//! Inter-process API (For userspace)
//!
//! This is the API to communicate with userspace wireguard implementation.
//! It communicates with programs such as `wireguard-go` by unix domain socket.
//!
//! For more detail protocol definition, read the [documentation](https://www.wireguard.com/xplatform/) by wireguard.
use crate::{
    types::{PresharedKey, PrivateKey, PublicKey, WgDevice},
    WireCtlError,
};
use async_fs::{read_dir, remove_file};
use async_net::unix::UnixStream;
use futures::io::BufReader;
use futures::prelude::*;
use std::path::PathBuf;
use std::{ffi::OsStr, os::unix::fs::FileTypeExt};
use std::{io::ErrorKind, str::FromStr};

pub const WG_SOCKET_PATH: &str = "/var/run/wireguard";
pub const WG_SOCKET_SUFFIX: &str = "sock";

pub async fn list_devices() -> Result<Vec<String>, WireCtlError> {
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
            if check_device(ifname).await {
                interfaces.push(ifname.to_string_lossy().into_owned());
            }
        }
    }

    Ok(interfaces)
}

async fn open_device<S: AsRef<OsStr> + ?Sized>(ifname: &S) -> Result<UnixStream, WireCtlError> {
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

async fn check_device<S: AsRef<OsStr> + ?Sized>(ifname: &S) -> bool {
    let rslt = open_device(ifname).await;
    rslt.is_ok()
}

pub async fn get_device(ifname: &str) -> Result<WgDevice, WireCtlError> {
    let mut ctrl_sock = BufReader::new(open_device(ifname).await?);

    let mut device = WgDevice::new(ifname);
    ctrl_sock.write_all(b"get=1\n\n").await?;

    let mut buf = String::with_capacity(1024);
    loop {
        ctrl_sock.read_line(&mut buf).await?;
        let line = buf.trim_end();
        if line.is_empty() {
            break;
        }

        let (key, value) = line.split_once('=').ok_or(WireCtlError::InvalidProtocol)?;

        match key {
            "private_key" => {
                let privkey = PrivateKey::from_hex(value)?;
                device.pubkey = Some(PublicKey::from(&privkey));
                device.privkey = Some(privkey);
            }
            "listen_port" => {
                device.listen_port = value.parse().map_err(|_| WireCtlError::InvalidProtocol)?;
            }
            "fwmark" => device.fwmark = value.parse().map_err(|_| WireCtlError::InvalidProtocol)?,
            "public_key" => {
                let pubkey = PublicKey::from_hex(value)?;
                // TODO
            }
            "errno" => {
                let errno: i32 = value.parse().map_err(|_| WireCtlError::InvalidProtocol)?;
                if errno != 0 {
                    return Err(WireCtlError::DeviceError(errno));
                }
            }
            _ => return Err(WireCtlError::InvalidProtocol),
        }
    }

    Ok(device)
}
