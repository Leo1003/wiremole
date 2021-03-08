use super::Peer;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Clone)]
pub struct WgDevice {
    devname: String,
    ifindex: u32,
    pubkey: Option<PublicKey>,
    privkey: Option<StaticSecret>,
    fwmark: u32,
    listen_port: u16,
    peers: Vec<Peer>,
}

impl Debug for WgDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("WgDevice")
            .field("devname", &self.devname)
            .field("ifindex", &self.ifindex)
            .field("pubkey", &self.pubkey)
            .field("privkey", &"<omitted>")
            .field("fwmark", &self.fwmark)
            .field("listen_port", &self.listen_port)
            .field("peers", &self.peers)
            .finish()
    }
}

impl WgDevice {
    pub fn device_name(&self) -> &str {
        &self.devname
    }

    pub fn public_key(&self) -> Option<PublicKey> {
        self.pubkey
    }

    pub fn private_key(&self) -> Option<StaticSecret> {
        self.privkey.clone()
    }

    pub fn fwmark(&self) -> u32 {
        self.fwmark
    }

    pub fn listen_port(&self) -> u16 {
        self.listen_port
    }

    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub fn has_private_key(&self) -> bool {
        self.privkey.is_some()
    }

    pub fn has_public_key(&self) -> bool {
        self.pubkey.is_some()
    }

    pub fn has_listen_port(&self) -> bool {
        self.listen_port != 0
    }

    pub fn has_fwmark(&self) -> bool {
        self.fwmark != 0
    }
}
