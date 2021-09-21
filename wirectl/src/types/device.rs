use super::{Peer, PeerSetter};
use super::{PrivateKey, PublicKey};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WgDevice {
    pub device_name: String,
    pub ifindex: u32,
    pub public_key: Option<PublicKey>,
    pub private_key: Option<PrivateKey>,
    pub fwmark: u32,
    pub listen_port: u16,
    pub peers: Vec<Peer>,
}

impl WgDevice {
    pub fn new(devname: &str) -> Self {
        Self {
            device_name: devname.to_owned(),
            ifindex: 0,
            public_key: None,
            private_key: None,
            fwmark: 0,
            listen_port: 0,
            peers: Vec::new(),
        }
    }

    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn has_public_key(&self) -> bool {
        self.public_key.is_some()
    }

    pub fn has_listen_port(&self) -> bool {
        self.listen_port != 0
    }

    pub fn has_fwmark(&self) -> bool {
        self.fwmark != 0
    }
}

#[derive(Debug)]
pub struct WgDeviceSetter {
    pub(crate) devname: String,
    pub(crate) privkey: Option<PrivateKey>,
    pub(crate) fwmark: Option<u32>,
    pub(crate) listen_port: Option<u16>,
    pub(crate) replace_peers: bool,
    pub(crate) peers: Vec<PeerSetter>,
}

impl WgDeviceSetter {
    pub(crate) fn new(devname: &str) -> Self {
        WgDeviceSetter {
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

    pub fn set_peer(mut self, peer: PeerSetter) -> Self {
        self.peers.push(peer);
        self
    }
}

impl From<WgDevice> for WgDeviceSetter {
    fn from(device: WgDevice) -> Self {
        WgDeviceSetter::new(&device.device_name)
    }
}
impl From<&WgDevice> for WgDeviceSetter {
    fn from(device: &WgDevice) -> Self {
        WgDeviceSetter::new(&device.device_name)
    }
}
