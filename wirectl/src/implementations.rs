use std::ffi::OsStr;

use crate::WireCtlError;
use crate::types::*;

#[async_trait]
pub trait WgImpl {
    async fn create_interface<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync;

    async fn list_interfaces() -> Result<Vec<String>, WireCtlError>;

    async fn remove_interface<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync;

    async fn check_device<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync;

    async fn get_config<S>(ifname: &S) -> Result<WgDevice, WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync;

    async fn set_config<S>(ifname: &S, conf: WgDeviceSetter) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync;
}
