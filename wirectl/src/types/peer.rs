use super::{PresharedKey, PublicKey};
use ipnetwork::IpNetwork;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::SystemTime,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Peer {
    pub public_key: PublicKey,
    pub preshared_key: PresharedKey,
    pub endpoint: SocketAddr,
    pub last_handshake: SystemTime,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub persistent_keepalive: u16,
    pub allow_ips: Vec<IpNetwork>,
}

impl Peer {
    pub fn new(pubkey: PublicKey) -> Self {
        Self {
            public_key: pubkey,
            preshared_key: PresharedKey::default(),
            endpoint: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            last_handshake: SystemTime::UNIX_EPOCH,
            rx_bytes: 0,
            tx_bytes: 0,
            persistent_keepalive: 0,
            allow_ips: Vec::new(),
        }
    }

    pub fn preshared_key_option(&self) -> Option<&PresharedKey> {
        if self.has_preshared_key() {
            Some(&self.preshared_key)
        } else {
            None
        }
    }

    pub fn is_address_allowed(&self, addr: IpAddr) -> bool {
        for network in &self.allow_ips {
            match network {
                IpNetwork::V4(net) => {
                    if let IpAddr::V4(addr) = addr {
                        if net.contains(addr) {
                            return true;
                        }
                    }
                }
                IpNetwork::V6(net) => {
                    if let IpAddr::V6(addr) = addr {
                        if net.contains(addr) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn has_public_key(&self) -> bool {
        !self.public_key.is_empty()
    }

    pub fn has_preshared_key(&self) -> bool {
        !self.preshared_key.is_empty()
    }

    pub fn has_persistent_keepalive(&self) -> bool {
        self.persistent_keepalive != 0
    }

    pub fn has_endpoint(&self) -> bool {
        !self.endpoint.ip().is_unspecified()
    }
}

#[derive(Debug)]
pub struct PeerSetter {
    pub(crate) pubkey: PublicKey,
    pub(crate) preshared_key: Option<PresharedKey>,
    pub(crate) endpoint: Option<SocketAddr>,
    pub(crate) persistent_keepalive: Option<u16>,
    pub(crate) replace_allowed_ips: bool,
    pub(crate) allowed_ips: Vec<IpNetwork>,
    pub(crate) update_only: bool,
    pub(crate) remove: bool,
}

impl PeerSetter {
    pub fn new(public_key: PublicKey) -> Self {
        PeerSetter {
            pubkey: public_key,
            preshared_key: None,
            endpoint: None,
            persistent_keepalive: None,
            replace_allowed_ips: false,
            allowed_ips: Vec::new(),
            update_only: false,
            remove: false,
        }
    }

    pub fn set_preshared_key(mut self, preshare_key: PresharedKey) -> Self {
        self.preshared_key = Some(preshare_key);
        self
    }

    pub fn set_endpoint(mut self, endpoint: SocketAddr) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn set_persistent_keepalive(mut self, keepalive: u16) -> Self {
        self.persistent_keepalive = Some(keepalive);
        self
    }

    pub fn add_allowed_ip(mut self, allow_ip: IpNetwork) -> Self {
        self.allowed_ips.push(allow_ip);
        self
    }

    pub fn add_allowed_ips(mut self, allow_ips: &[IpNetwork]) -> Self {
        self.allowed_ips.extend_from_slice(allow_ips);
        self
    }

    pub fn set_replace_allowed_ips(mut self) -> Self {
        self.replace_allowed_ips = true;
        self
    }
}

impl From<Peer> for PeerSetter {
    fn from(peer: Peer) -> Self {
        Self::new(peer.public_key)
    }
}
impl From<&Peer> for PeerSetter {
    fn from(peer: &Peer) -> Self {
        Self::new(peer.public_key.clone())
    }
}
