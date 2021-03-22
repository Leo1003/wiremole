//! Inter-process API (For userspace)
//!
//! This is the API to communicate with userspace wireguard implementation.
//! It communicates with programs such as `wireguard-go` by unix domain socket.
//!
//! For more detail protocol definition, read the [documentation](https://www.wireguard.com/xplatform/) by wireguard.
use crate::WireCtlError;
use async_fs::{read_dir, remove_file};
use async_net::unix::UnixStream;
use futures::prelude::*;
use std::path::PathBuf;
use std::{ffi::OsStr, os::unix::fs::FileTypeExt};
use std::{io::ErrorKind, str::FromStr};

pub const WG_SOCKET_PATH: &str = "/var/run/wireguard";
pub const WG_SOCKET_SUFFIX: &str = "sock";

pub async fn get_interfaces() -> Result<Vec<String>, WireCtlError> {
    let mut sockdir = match read_dir(WG_SOCKET_PATH).await {
        Ok(data) => data,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Ok(Vec::new());
            }
            return Err(e.into());
        }
    };

    let mut interfaces = Vec::new();
    while let Some(entry) = sockdir.try_next().await? {
        let meta = entry.metadata().await?;
        if meta.file_type().is_socket() {
            let sockname = PathBuf::from(entry.file_name());
            if sockname.extension() != Some(OsStr::new(WG_SOCKET_SUFFIX)) {
                continue;
            }

            let ifname = sockname.file_stem().unwrap();
            if check_interface(ifname).await {
                interfaces.push(ifname.to_string_lossy().into_owned());
            }
        }
    }

    Ok(interfaces)
}

async fn open_interface<S: AsRef<OsStr> + ?Sized>(ifname: &S) -> Result<UnixStream, WireCtlError> {
    let mut socket_path = PathBuf::from_str(WG_SOCKET_PATH).unwrap();
    socket_path.push(ifname.as_ref());
    socket_path.set_extension(WG_SOCKET_SUFFIX);

    let socket = match UnixStream::connect(&socket_path).await {
        Ok(s) => s,
        Err(e) => {
            // Try to clean up the unused socket
            if e.kind() == ErrorKind::ConnectionRefused {
                remove_file(&socket_path).await.ok();
            }

            return Err(e.into());
        }
    };

    Ok(socket)
}

async fn check_interface<S: AsRef<OsStr> + ?Sized>(ifname: &S) -> bool {
    let rslt = open_interface(ifname).await;
    rslt.is_ok()
}
