//! Wireguard interface types
use zeroize::Zeroizing;

mod device;
mod peer;

pub use device::*;
pub use peer::*;

pub type PresharedKey = Zeroizing<[u8; 32]>;
