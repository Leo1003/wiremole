use crate::{ipc, types::WgDevice};
use crate::WireCtlError;

/// Used by [`crate::interface::WgInterface`] to determine which underlaying API to use
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgApi {
    IPC,
    #[cfg(target_os = "linux")]
    Linux,
    #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
    BSD,
}

impl WgApi {
    pub(crate) async fn list_devices(&self) -> Result<Vec<String>, WireCtlError> {
        match self {
            WgApi::IPC => ipc::list_devices().await,
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }

    pub(crate) async fn get_device(&self, ifname: &str) -> Result<WgDevice, WireCtlError> {
        match self {
            WgApi::IPC => ipc::get_device(ifname).await,
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }

    pub(crate) async fn set_device(&self) -> Result<Vec<String>, WireCtlError> {
        match self {
            WgApi::IPC => todo!(),
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }
}
