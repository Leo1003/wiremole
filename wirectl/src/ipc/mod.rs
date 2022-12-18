//! Inter-process API (For userspace)
//!
//! This is the API to communicate with userspace wireguard implementation.
//! It communicates with programs such as `wireguard-go` by unix domain socket.
//!
//! For more detail protocol definition, read the [documentation](https://www.wireguard.com/xplatform/) by wireguard.
use crate::{implementations::WgImpl, types::*, WireCtlError};
use async_fs::{read_dir, remove_file};
use async_net::unix::UnixStream;
use async_process::Command;
use futures::io::BufReader;
use futures::prelude::*;
use once_cell::sync::Lazy;
use std::{
    borrow::Cow,
    env,
    ffi::OsStr,
    io::{Error, ErrorKind},
    os::unix::fs::FileTypeExt,
    path::PathBuf,
    str::FromStr,
    time::Duration,
    time::SystemTime,
};

pub const WG_SOCKET_PATH: &str = "/var/run/wireguard";
pub const WG_SOCKET_SUFFIX: &str = "sock";
pub const DEFAULT_WG_USERSPACE_IMPL: &str = "wireguard-go";

static WG_USERSPACE_EXEC: Lazy<Cow<OsStr>> = Lazy::new(|| {
    let exec = env::var_os("WG_USERSPACE_IMPLEMENTATION")
        .or_else(|| env::var_os("WG_QUICK_USERSPACE_IMPLEMENTATION"))
        .map_or_else(
            || OsStr::new(DEFAULT_WG_USERSPACE_IMPL).into(),
            |s| s.into(),
        );

    debug!("Using {:?} as userspace Wireguard implementation", &exec);

    exec
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ipc;

#[async_trait]
impl WgImpl for Ipc {
    async fn create_interface<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync,
    {
        let program: &OsStr = WG_USERSPACE_EXEC.as_ref();
        let status = Command::new(program)
            .arg(ifname)
            .env_clear()
            .status()
            .await?;

        if !status.success() {
            return Err(WireCtlError::UserspaceLaunch(status));
        }

        Ok(())
    }

    async fn list_interfaces() -> Result<Vec<String>, WireCtlError> {
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
                if check_device(ifname).await.is_ok() {
                    interfaces.push(ifname.to_string_lossy().into_owned());
                }
            }
        }

        Ok(interfaces)
    }

    async fn remove_interface<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync,
    {
        todo!();
    }

    async fn check_device<S>(ifname: &S) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync,
    {
        let rslt = open_device(ifname).await;
        rslt.map(|_| ())
    }

    async fn get_config<S>(ifname: &S) -> Result<WgDevice, WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync,
    {
        let mut ctrl_sock = BufReader::new(open_device(ifname).await?);
        ctrl_sock.write_all(b"get=1\n\n").await?;

        parse_device_config(&mut ctrl_sock, ifname).await
    }

    async fn set_config<S>(ifname: &S, conf: WgDeviceSetter) -> Result<(), WireCtlError>
    where
        S: AsRef<OsStr> + ?Sized + Send + Sync,
    {
        let mut ctrl_sock = BufReader::new(open_device(ifname).await?);

        emit_device_config(&mut ctrl_sock, conf).await?;
        ctrl_sock.flush().await?;

        let mut curr_line = String::new();

        // Read return errno
        // Format:
        // `errno=0`
        ctrl_sock.read_line(&mut curr_line).await?;
        let line = curr_line.trim_end();
        let (key, value) = line.split_once('=').ok_or(WireCtlError::InvalidProtocol)?;
        let errno = if key == "errno" {
            value.parse::<i32>()?
        } else {
            return Err(WireCtlError::InvalidProtocol);
        };

        // Next line should be empty
        curr_line.clear();
        ctrl_sock.read_line(&mut curr_line).await?;
        let line = curr_line.trim_end();
        if !line.is_empty() {
            return Err(WireCtlError::InvalidProtocol);
        }

        if errno == 0 {
            Ok(())
        } else {
            Err(WireCtlError::DeviceError(errno))
        }
    }
}

pub async fn create_interface(ifname: &str) -> Result<(), WireCtlError> {
    Ipc::create_interface(ifname).await
}

pub async fn list_interfaces() -> Result<Vec<String>, WireCtlError> {
    Ipc::list_interfaces().await
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
                return Err(Error::from(ErrorKind::NotFound).into());
            }

            return Err(e.into());
        }
    };

    Ok(socket)
}

pub async fn check_device<S>(ifname: &S) -> Result<(), WireCtlError>
where
    S: AsRef<OsStr> + ?Sized + Send + Sync,
{
    Ipc::check_device(ifname).await
}

pub async fn get_config(ifname: &str) -> Result<WgDevice, WireCtlError> {
    Ipc::get_config(ifname).await
}

async fn parse_device_config<R, S>(ctrl_sock: &mut R, ifname: &S) -> Result<WgDevice, WireCtlError>
where
    R: AsyncBufRead + AsyncRead + Unpin + ?Sized,
    S: AsRef<OsStr> + ?Sized + Send + Sync,
{
    let mut errno = None;
    let mut device = WgDevice::new(ifname.as_ref().to_string_lossy().as_ref());

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
                device.public_key = Some(PublicKey::from(&privkey));
                device.private_key = Some(privkey);
            }
            "listen_port" => {
                device.listen_port = value.parse()?;
            }
            "fwmark" => device.fwmark = value.parse()?,
            "public_key" => {
                let pubkey = PublicKey::from_hex(value)?;
                let peer = parse_peer_config(ctrl_sock, &mut curr_line, pubkey).await?;
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
        curr_line.clear();
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

async fn parse_peer_config<S>(
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

    // Read next line
    curr_line.clear();
    ctrl_sock.read_line(curr_line).await?;

    loop {
        let line = curr_line.trim_end();
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once('=').ok_or(WireCtlError::InvalidProtocol)?;

        match key {
            "preshared_key" => peer.preshared_key = PresharedKey::from_hex(value)?,
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
        curr_line.clear();
        ctrl_sock.read_line(curr_line).await?;
    }

    peer.last_handshake = SystemTime::UNIX_EPOCH + last_handshake_s + last_handshake_ns;
    Ok(peer)
}

pub async fn set_config(ifname: &str, conf: WgDeviceSetter) -> Result<(), WireCtlError> {
    Ipc::set_config(ifname, conf).await
}

async fn emit_device_config<S>(ctrl_sock: &mut S, conf: WgDeviceSetter) -> Result<(), WireCtlError>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    ctrl_sock.write_all(b"set=1\n").await?;

    if let Some(privkey) = conf.privkey {
        let line = format!("private_key={}\n", privkey.to_hex());
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if let Some(fwmark) = conf.fwmark {
        let line = format!("fwmark={}\n", fwmark);
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if let Some(listen_port) = conf.listen_port {
        let line = format!("listen_port={}\n", listen_port);
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if conf.replace_peers {
        ctrl_sock.write_all(b"replace_peers=true\n").await?;
    }
    for peer in conf.peers {
        emit_peer_config(ctrl_sock, peer).await?;
    }
    // End with empty line
    ctrl_sock.write_all(b"\n").await?;

    Ok(())
}

async fn emit_peer_config<S>(ctrl_sock: &mut S, conf: PeerSetter) -> Result<(), WireCtlError>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    let line = format!("public_key={}\n", conf.pubkey.to_hex());
    ctrl_sock.write_all(line.as_bytes()).await?;

    if conf.remove {
        ctrl_sock.write_all(b"remove=true\n").await?;
        return Ok(());
    }
    if conf.update_only {
        ctrl_sock.write_all(b"update_only=true\n").await?;
    }
    if let Some(preshared_key) = conf.preshared_key {
        let line = format!("preshared_key={}\n", preshared_key.to_hex());
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if let Some(endpoint) = conf.endpoint {
        let line = format!("endpoint={}\n", endpoint);
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if let Some(keepalive) = conf.persistent_keepalive {
        let line = format!("persistent_keepalive_interval={}\n", keepalive);
        ctrl_sock.write_all(line.as_bytes()).await?;
    }
    if conf.replace_allowed_ips {
        ctrl_sock.write_all(b"replace_allowed_ips=true\n").await?;
    }
    for allowed_ip in conf.allowed_ips {
        let line = format!("allowed_ip={}\n", allowed_ip);
        ctrl_sock.write_all(line.as_bytes()).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
