//! Types related to Wireguard devices
use futures::{StreamExt, TryFutureExt, TryStreamExt};

use crate::{
    api::{WgApi, AVAILABLE_WG_APIS},
    WireCtlError,
};

#[derive(Debug)]
pub struct WgInterface {
    ifname: String,
    wgapi: WgApi,
}

impl WgInterface {
    pub async fn get_interfaces() -> Result<Vec<WgInterface>, WireCtlError> {
        futures::stream::iter(AVAILABLE_WG_APIS.iter().copied())
            .then(|api| api.list_devices().map_ok(move |l| {
                l.into_iter()
                    .map(|ifname| WgInterface { ifname, wgapi: api })
                    .collect()
            }))
            .try_concat()
            .await
    }

    pub async fn list_interfaces() -> Result<Vec<String>, WireCtlError> {
        futures::stream::iter(&AVAILABLE_WG_APIS)
            .then(|api| api.list_devices())
            .try_concat()
            .await
    }
}
