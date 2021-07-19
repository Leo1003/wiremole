//! Types related to Wireguard devices
use crate::types::*;
use futures::{StreamExt, TryFutureExt, TryStreamExt};

use crate::{
    api::{WgApi, AVAILABLE_WG_APIS},
    WireCtlError,
};

#[derive(Clone, Debug)]
pub struct WgInterface {
    ifname: String,
    wgapi: WgApi,
}

impl WgInterface {
    pub async fn create_interface(ifname: &str) -> Result<WgInterface, WireCtlError> {
        Self::create_interface_with(AVAILABLE_WG_APIS[0], ifname).await
    }

    pub async fn create_interface_with(
        api: WgApi,
        ifname: &str,
    ) -> Result<WgInterface, WireCtlError> {
        api.add_interface(ifname).await?;

        Ok(WgInterface {
            ifname: ifname.to_owned(),
            wgapi: api,
        })
    }

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

    pub async fn remove_interfaces(self) -> Result<(), WireCtlError> {
        self.wgapi.del_interface(&self.ifname).await
    }

    pub fn ifname(&self) -> &str {
        &self.ifname
    }

    pub fn api(&self) -> WgApi {
        self.wgapi
    }
}
