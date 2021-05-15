//! Inter-process API (For userspace)
//!
//! This is the API to communicate with userspace wireguard implementation.
//! It communicates with programs such as `wireguard-go` by unix domain socket.
//!
//! For more detail protocol definition, read the [documentation](https://www.wireguard.com/xplatform/) by wireguard.
use crate::{
    types::{Peer, PresharedKey, PrivateKey, PublicKey, WgDevice},
    WireCtlError,
};
use async_fs::{read_dir, remove_file};
use async_net::unix::UnixStream;
use futures::io::BufReader;
use futures::prelude::*;
use std::{ffi::OsStr, os::unix::fs::FileTypeExt, time::SystemTime};
use std::{io::ErrorKind, str::FromStr};
use std::{path::PathBuf, time::Duration};

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

    let mut errno = None;
    let mut device = WgDevice::new(ifname);
    ctrl_sock.write_all(b"get=1\n\n").await?;

    let mut curr_line = String::with_capacity(1024);
    ctrl_sock.read_line(&mut curr_line).await?;

    loop {
        let line = curr_line.trim_end();
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
                device.listen_port = value.parse()?;
            }
            "fwmark" => device.fwmark = value.parse()?,
            "public_key" => {
                let pubkey = PublicKey::from_hex(value)?;
                let peer = read_peer_info(&mut ctrl_sock, &mut curr_line, pubkey).await?;
                device.peers.push(peer);

                // The next line has already been read into `curr_line` by `read_peer_info()`
                continue;
            }
            "errno" => {
                if errno.is_some() {
                    // errno is alreadys set
                    return Err(WireCtlError::InvalidProtocol);
                } else {
                    errno = Some(value.parse::<i32>()?);
                }
            }
            _ => return Err(WireCtlError::InvalidProtocol),
        }

        // Read next line
        ctrl_sock.read_line(&mut curr_line).await?;
    }

    if let Some(errno) = errno {
        if errno == 0 {
            Ok(device)
        } else {
            Err(WireCtlError::DeviceError(errno))
        }
    } else {
        // If the peer doesn't send errno, treat as invalid protocol
        Err(WireCtlError::InvalidProtocol)
    }
}

async fn read_peer_info<S>(
    ctrl_sock: &mut S,
    curr_line: &mut String,
    pubkey: PublicKey,
) -> Result<Peer, WireCtlError>
where
    S: AsyncBufRead + AsyncRead + Unpin + ?Sized,
{
    let mut peer = Peer::new(pubkey);
    let mut last_handshake_s = Duration::default();
    let mut last_handshake_ns = Duration::default();

    loop {
        let line = curr_line.trim_end();
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once('=').ok_or(WireCtlError::InvalidProtocol)?;

        match key {
            "preshared_key" => peer.preshared = PresharedKey::from_hex(value)?,
            "allowed_ip" => {
                let allowed_ip = value.parse()?;
                peer.allow_ips.push(allowed_ip);
            }
            "endpoint" => {
                peer.endpoint = value.parse()?;
            }
            "tx_bytes" => {
                peer.tx_bytes = value.parse()?;
            }
            "rx_bytes" => {
                peer.rx_bytes = value.parse()?;
            }
            "persistent_keepalive_interval" => {
                peer.persistent_keepalive = value.parse()?;
            }
            "last_handshake_time_sec" => {
                last_handshake_s = Duration::from_secs(value.parse()?);
            }
            "last_handshake_time_nsec" => {
                last_handshake_ns = Duration::from_nanos(value.parse()?);
            }
            "protocol_version" => (), // Currently, we don't care the protocol_version
            _ => break,
        }

        // Read next line
        ctrl_sock.read_line(curr_line).await?;
    }

    peer.last_handshake = SystemTime::UNIX_EPOCH + last_handshake_s + last_handshake_ns;
    Ok(peer)
}
