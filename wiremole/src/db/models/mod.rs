use std::{net::IpAddr, net::SocketAddr};
use wirectl::types::{PublicKey, PresharedKey, PrivateKey};
use ipnetwork::IpNetwork;

pub mod db_mysql;

#[derive(Debug)]
pub struct Interface {
    pub id: i32,
    pub devname: String,
    pub mtu: Option<u32>,
    pub privkey: Option<PrivateKey>,
    pub fwmark: u32,
    pub listen_port: u16,
}

#[derive(Debug)]
pub struct InterfaceIp {
    pub id: i32,
    pub interface_id: i32,
    pub ipnetwork: IpNetwork,
}

#[derive(Debug)]
pub struct Peer {
    pub id: i32,
    pub interface_id: i32,
    pub pubkey: PublicKey,
    pub preshared_key: Option<PresharedKey>,
    pub endpoint: Option<SocketAddr>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug)]
pub struct AllowedIp {
    pub id: i32,
    pub peer_id: i32,
    pub ipnetwork: IpNetwork,
}
