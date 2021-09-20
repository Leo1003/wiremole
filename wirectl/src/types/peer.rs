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
    pub(crate) pubkey: PublicKey,
    pub(crate) preshared: PresharedKey,
    pub(crate) endpoint: SocketAddr,
    pub(crate) last_handshake: SystemTime,
    pub(crate) rx_bytes: u64,
    pub(crate) tx_bytes: u64,
    pub(crate) persistent_keepalive: u16,
    pub(crate) allow_ips: Vec<IpNetwork>,
}

impl Peer {
    pub(crate) fn new(pubkey: PublicKey) -> Self {
        Self {
            pubkey,
            preshared: PresharedKey::default(),
            endpoint: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            last_handshake: SystemTime::UNIX_EPOCH,
            rx_bytes: 0,
            tx_bytes: 0,
            persistent_keepalive: 0,
            allow_ips: Vec::new(),
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.pubkey
    }

    pub fn preshared_key(&self) -> Option<&PresharedKey> {
        if self.has_preshared_key() {
            Some(&self.preshared)
        } else {
            None
        }
    }

    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    pub fn last_handshake(&self) -> SystemTime {
        self.last_handshake
    }

    pub fn rx_bytes(&self) -> u64 {
        self.rx_bytes
    }

    pub fn tx_bytes(&self) -> u64 {
        self.tx_bytes
    }

    pub fn persistent_keepalive(&self) -> u16 {
        self.persistent_keepalive
    }

    pub fn allow_ips(&self) -> &[IpNetwork] {
        &self.allow_ips
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
        !self.pubkey.is_empty()
    }

    pub fn has_preshared_key(&self) -> bool {
        !self.preshared.is_empty()
    }

    pub fn has_persistent_keepalive(&self) -> bool {
        self.persistent_keepalive != 0
    }
}

#[derive(Debug)]
pub struct PeerSettings {
    pub(crate) pubkey: PublicKey,
    pub(crate) preshared_key: Option<PresharedKey>,
    pub(crate) endpoint: Option<SocketAddr>,
    pub(crate) persistent_keepalive: Option<u16>,
    pub(crate) replace_allowed_ips: bool,
    pub(crate) allowed_ips: Vec<IpNetwork>,
    pub(crate) update_only: bool,
    pub(crate) remove: bool,
}

impl PeerSettings {
    pub fn new(public_key: PublicKey) -> Self {
        PeerSettings {
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

impl From<Peer> for PeerSettings {
    fn from(peer: Peer) -> Self {
        Self::new(peer.pubkey)
    }
}
impl From<&Peer> for PeerSettings {
    fn from(peer: &Peer) -> Self {
        Self::new(peer.pubkey.clone())
    }
}
