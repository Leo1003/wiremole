use crate::WireCtlError;
use crate::{ipc, types::*};

cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub const AVAILABLE_WG_APIS: [WgApi; 1] = [WgApi::IPC];
    } else if #[cfg(any(target_os = "openbsd", target_os = "freebsd"))] {
        pub const AVAILABLE_WG_APIS: [WgApi; 1] = [WgApi::IPC];
    } else {
        pub const AVAILABLE_WG_APIS: [WgApi; 1] = [WgApi::IPC];
    }
}

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
    pub(crate) async fn list_interfaces(self) -> Result<Vec<String>, WireCtlError> {
        match self {
            WgApi::IPC => ipc::list_interfaces().await,
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }

    pub(crate) async fn get_config(self, ifname: &str) -> Result<WgDevice, WireCtlError> {
        match self {
            WgApi::IPC => ipc::get_config(ifname).await,
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }

    pub(crate) async fn set_config(
        self,
        ifname: &str,
        conf: WgDeviceSettings,
    ) -> Result<(), WireCtlError> {
        match self {
            WgApi::IPC => ipc::set_config(ifname, conf).await,
            #[cfg(target_os = "linux")]
            WgApi::Linux => todo!(),
            #[cfg(any(target_os = "openbsd", target_os = "freebsd"))]
            WgApi::BSD => todo!(),
        }
    }
}
