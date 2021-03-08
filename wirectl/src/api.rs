use std::fmt::Debug;

/// Used by [`crate::interface::WgInterface`] to determine which underlaying API to use
pub trait WgApi: Debug {
    fn get(&self, ifname: &str);
}
