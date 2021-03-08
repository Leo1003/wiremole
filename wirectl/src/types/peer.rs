use ipnetwork::IpNetwork;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;
use x25519_dalek::PublicKey;
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct Peer {
    pubkey: Option<PublicKey>,
    preshared: Zeroizing<[u8; 32]>,
    endpoint: SocketAddr,
    last_handshake: SystemTime,
    rx_bytes: u64,
    tx_bytes: u64,
    persistent_keepalive: u16,
    allow_ips: Vec<IpNetwork>,
}

impl Debug for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Peer")
            .field("pubkey", &self.pubkey)
            .field("preshared", &"<omitted>")
            .field("endpoint", &self.endpoint)
            .field("last_handshake", &self.last_handshake)
            .field("rx_bytes", &self.rx_bytes)
            .field("tx_bytes", &self.tx_bytes)
            .field("persistent_keepalive", &self.persistent_keepalive)
            .field("allow_ips", &self.allow_ips)
            .finish()
    }
}

impl Peer {
    pub fn public_key(&self) -> Option<PublicKey> {
        self.pubkey
    }

    pub fn preshared_key(&self) -> Option<&[u8; 32]> {
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
        self.pubkey.is_some()
    }

    pub fn has_preshared_key(&self) -> bool {
        *self.preshared != [0u8; 32]
    }

    pub fn has_persistent_keepalive(&self) -> bool {
        self.persistent_keepalive != 0
    }
}
