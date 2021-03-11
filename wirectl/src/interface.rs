//! Types related to Wireguard devices
use crate::api::WgApi;

#[derive(Debug)]
pub struct WgInterface {
    ifname: String,
    wgapi: WgApi,
}
