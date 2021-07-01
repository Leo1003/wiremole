//! Types related to Wireguard devices
use crate::types::*;
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
            .then(|api| {
                api.list_interfaces().map_ok(move |l| {
                    l.into_iter()
                        .map(|ifname| WgInterface { ifname, wgapi: api })
                        .collect()
                })
            })
            .try_concat()
            .await
    }

    pub async fn list_interfaces() -> Result<Vec<String>, WireCtlError> {
        futures::stream::iter(&AVAILABLE_WG_APIS)
            .then(|api| api.list_interfaces())
            .try_concat()
            .await
    }

    pub async fn get_config(&self) -> Result<WgDevice, WireCtlError> {
        self.wgapi.get_config(&self.ifname).await
    }

    pub async fn set_config(&self, conf: WgDeviceSettings) -> Result<(), WireCtlError> {
        if conf.devname != self.ifname {
            return Err(WireCtlError::InvalidConfig);
        }
        self.wgapi.set_config(&self.ifname, conf).await
    }
}
