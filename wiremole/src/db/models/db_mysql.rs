use crate::db::{schema::*, FromModel, IntoModel};
use anyhow::Context;
use ipnetwork::IpNetwork;
use std::{convert::{Infallible, TryFrom}, net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6}};
use wirectl::types::{PresharedKey, PrivateKey, PublicKey};

#[derive(Clone, Debug, Identifiable, Queryable, Insertable, AsChangeset)]
#[table_name = "interfaces"]
#[primary_key(id)]
pub struct Interface {
    pub id: i32,
    pub devname: String,
    pub mtu: Option<u32>,
    pub privkey: Option<Vec<u8>>,
    pub fwmark: u32,
    pub listen_port: u16,
}

impl IntoModel for Interface {
    type Output = super::Interface;
    type Error = anyhow::Error;

    fn into_model(self) -> Result<Self::Output, Self::Error> {
        Ok(super::Interface {
            id: self.id,
            devname: self.devname,
            mtu: self.mtu,
            privkey: self
                .privkey
                .map(|privkey| PrivateKey::try_from(privkey.as_slice()))
                .transpose()?,
            fwmark: self.fwmark,
            listen_port: self.listen_port,
        })
    }
}

impl FromModel<super::Interface> for Interface {
    type Error = Infallible;

    fn from_model(model: super::Interface) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            devname: model.devname,
            mtu: model.mtu,
            privkey: model
                .privkey
                .map(|privkey| Vec::from(<[u8; 32]>::from(privkey))),
            fwmark: model.fwmark,
            listen_port: model.listen_port,
        })
    }
}

#[derive(Clone, Debug, Associations, Identifiable, Queryable, Insertable, AsChangeset)]
#[table_name = "interface_ips"]
#[primary_key(id)]
#[belongs_to(Interface)]
pub struct InterfaceIp {
    pub id: i32,
    pub interface_id: i32,
    pub ipaddress: Vec<u8>,
    pub mask: u8,
}

impl IntoModel for InterfaceIp {
    type Output = super::InterfaceIp;
    type Error = anyhow::Error;

    fn into_model(self) -> Result<Self::Output, Self::Error> {
        let ipaddr = match self.ipaddress.len() {
            4 => IpAddr::from(<[u8; 4]>::try_from(self.ipaddress.as_slice()).unwrap()),
            16 => IpAddr::from(<[u8; 16]>::try_from(self.ipaddress.as_slice()).unwrap()),
            _ => bail!("Invalid IP address in the database"),
        };
        Ok(super::InterfaceIp {
            id: self.id,
            interface_id: self.interface_id,
            ipnetwork: IpNetwork::new(ipaddr, self.mask)
                .with_context(|| "Invalid network mask in the database")?,
        })
    }
}

impl FromModel<super::InterfaceIp> for InterfaceIp {
    type Error = Infallible;

    fn from_model(model: super::InterfaceIp) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            interface_id: model.interface_id,
            ipaddress: match model.ipnetwork.ip() {
                IpAddr::V4(addr) => addr.octets().into(),
                IpAddr::V6(addr) => addr.octets().into(),
            },
            mask: model.ipnetwork.prefix(),
        })
    }
}

#[derive(Clone, Debug, Associations, Identifiable, Queryable, Insertable, AsChangeset)]
#[table_name = "peers"]
#[primary_key(id)]
#[belongs_to(Interface)]
pub struct Peer {
    pub id: i32,
    pub interface_id: i32,
    pub pubkey: Vec<u8>,
    pub preshared_key: Option<Vec<u8>>,
    pub endpoint_ip: Option<Vec<u8>>,
    pub endpoint_port: Option<u16>,
    pub endpoint_flowinfo: Option<u32>,
    pub persistent_keepalive: Option<u16>,
}

impl IntoModel for Peer {
    type Output = super::Peer;
    type Error = anyhow::Error;

    fn into_model(self) -> Result<Self::Output, Self::Error> {
        let sockaddr = if let Some((ip, port)) = self.endpoint_ip.zip(self.endpoint_port) {
            Some(match ip.len() {
                4 => SocketAddrV4::new(
                    Ipv4Addr::from(<[u8; 4]>::try_from(ip.as_slice()).unwrap()),
                    port,
                )
                .into(),
                16 => SocketAddrV6::new(
                    Ipv6Addr::from(<[u8; 16]>::try_from(ip.as_slice()).unwrap()),
                    port,
                    self.endpoint_flowinfo.unwrap_or_default(),
                    0,
                )
                .into(),
                _ => bail!("Invalid IP address in the database"),
            })
        } else {
            None
        };
        Ok(super::Peer {
            id: self.id,
            interface_id: self.interface_id,
            pubkey: PublicKey::try_from(self.pubkey.as_slice())?,
            preshared_key: self
                .preshared_key
                .map(|preshared_key| PresharedKey::try_from(preshared_key.as_slice()))
                .transpose()?,
            endpoint: sockaddr,
            persistent_keepalive: self.persistent_keepalive,
        })
    }
}

#[derive(Clone, Debug, Associations, Identifiable, Queryable, Insertable, AsChangeset)]
#[table_name = "allowed_ips"]
#[primary_key(id)]
#[belongs_to(Peer)]
pub struct AllowedIp {
    pub id: i32,
    pub peer_id: i32,
    pub ipaddress: Vec<u8>,
    pub mask: u8,
}

impl IntoModel for AllowedIp {
    type Output = super::AllowedIp;
    type Error = anyhow::Error;

    fn into_model(self) -> Result<Self::Output, Self::Error> {
        let ipaddr = match self.ipaddress.len() {
            4 => IpAddr::from(<[u8; 4]>::try_from(self.ipaddress.as_slice()).unwrap()),
            16 => IpAddr::from(<[u8; 16]>::try_from(self.ipaddress.as_slice()).unwrap()),
            _ => bail!("Invalid IP address in the database"),
        };
        Ok(super::AllowedIp {
            id: self.id,
            peer_id: self.peer_id,
            ipnetwork: IpNetwork::new(ipaddr, self.mask)
                .with_context(|| "Invalid network mask in the database")?,
        })
    }
}

impl FromModel<super::AllowedIp> for AllowedIp {
    type Error = Infallible;

    fn from_model(model: super::AllowedIp) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            peer_id: model.peer_id,
            ipaddress: match model.ipnetwork.ip() {
                IpAddr::V4(addr) => addr.octets().into(),
                IpAddr::V6(addr) => addr.octets().into(),
            },
            mask: model.ipnetwork.prefix(),
        })
    }
}
