use super::{Peer, PeerSettings};
use super::{PrivateKey, PublicKey};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WgDevice {
    pub(crate) devname: String,
    pub(crate) ifindex: u32,
    pub(crate) pubkey: Option<PublicKey>,
    pub(crate) privkey: Option<PrivateKey>,
    pub(crate) fwmark: u32,
    pub(crate) listen_port: u16,
    pub(crate) peers: Vec<Peer>,
}

impl WgDevice {
    pub(crate) fn new(devname: &str) -> Self {
        Self {
            devname: devname.to_owned(),
            ifindex: 0,
            pubkey: None,
            privkey: None,
            fwmark: 0,
            listen_port: 0,
            peers: Vec::new(),
        }
    }

    pub fn device_name(&self) -> &str {
        &self.devname
    }

    pub fn public_key(&self) -> Option<&PublicKey> {
        self.pubkey.as_ref()
    }

    pub fn private_key(&self) -> Option<&PrivateKey> {
        self.privkey.as_ref()
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

#[derive(Debug)]
pub struct WgDeviceSettings {
    pub(crate) devname: String,
    pub(crate) privkey: Option<PrivateKey>,
    pub(crate) fwmark: Option<u32>,
    pub(crate) listen_port: Option<u16>,
    pub(crate) replace_peers: bool,
    pub(crate) peers: Vec<PeerSettings>,
}

impl WgDeviceSettings {
    pub(crate) fn new(devname: &str) -> Self {
        WgDeviceSettings {
            devname: devname.to_owned(),
            privkey: None,
            fwmark: None,
            listen_port: None,
            replace_peers: false,
            peers: Vec::new(),
        }
    }

    pub fn set_private_key(mut self, private_key: PrivateKey) -> Self {
        self.privkey = Some(private_key);
        self
    }

    pub fn set_fwmark(mut self, fwmark: u32) -> Self {
        self.fwmark = Some(fwmark);
        self
    }

    pub fn set_listen_port(mut self, listen_port: u16) -> Self {
        self.listen_port = Some(listen_port);
        self
    }

    pub fn set_replace_peers(mut self) -> Self {
        self.replace_peers = true;
        self
    }

    pub fn set_peer(mut self, peer: PeerSettings) -> Self {
        self.peers.push(peer);
        self
    }
}

impl From<WgDevice> for WgDeviceSettings {
    fn from(device: WgDevice) -> Self {
        WgDeviceSettings::new(device.device_name())
    }
}
impl From<&WgDevice> for WgDeviceSettings {
    fn from(device: &WgDevice) -> Self {
        WgDeviceSettings::new(device.device_name())
    }
}
