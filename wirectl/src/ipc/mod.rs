//! Inter-process API (For userspace)
//!
//! This is the API to communicate with userspace wireguard implementation.
//! It communicates with programs such as `wireguard-go` by unix domain socket.
//!
//! For more detail protocol definition, read the [documentation](https://www.wireguard.com/xplatform/) by wireguard.
use crate::{types::*, WireCtlError};
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
    fmt::Arguments,
    io::{ErrorKind, Write as _},
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

pub async fn create_interface(ifname: &str) -> Result<(), WireCtlError> {
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

pub async fn list_interfaces() -> Result<Vec<String>, WireCtlError> {
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

pub async fn check_device<S: AsRef<OsStr> + ?Sized>(ifname: &S) -> bool {
    let rslt = open_device(ifname).await;
    rslt.is_ok()
}

pub async fn get_config(ifname: &str) -> Result<WgDevice, WireCtlError> {
    let mut ctrl_sock = BufReader::new(open_device(ifname).await?);
    ctrl_sock.write_all(b"get=1\n\n").await?;

    parse_device_config(&mut ctrl_sock, ifname).await
}

async fn parse_device_config<S>(ctrl_sock: &mut S, ifname: &str) -> Result<WgDevice, WireCtlError>
where
    S: AsyncBufRead + AsyncRead + Unpin + ?Sized,
{
    let mut errno = None;
    let mut device = WgDevice::new(ifname);

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
        curr_line.clear();
        ctrl_sock.read_line(curr_line).await?;
    }

    peer.last_handshake = SystemTime::UNIX_EPOCH + last_handshake_s + last_handshake_ns;
    Ok(peer)
}

async fn write_fmt<S>(ctrl_sock: &mut S, args: Arguments<'_>) -> Result<(), WireCtlError>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    let mut buf = Vec::new();
    write!(&mut buf, "{}", args)
        .map_err(|_| WireCtlError::Io(std::io::Error::new(ErrorKind::Other, "formatter error")))?;
    ctrl_sock.write_all(buf.as_slice()).await?;
    Ok(())
}

pub async fn set_config(ifname: &str, conf: WgDeviceSettings) -> Result<(), WireCtlError> {
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

async fn emit_device_config<S>(
    ctrl_sock: &mut S,
    conf: WgDeviceSettings,
) -> Result<(), WireCtlError>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    write_fmt(ctrl_sock, format_args!("set=1\n")).await?;

    if let Some(privkey) = conf.privkey {
        write_fmt(
            ctrl_sock,
            format_args!("private_key={}\n", privkey.to_hex()),
        )
        .await?;
    }
    if let Some(fwmark) = conf.fwmark {
        write_fmt(ctrl_sock, format_args!("fwmark={}\n", fwmark)).await?;
    }
    if let Some(listen_port) = conf.listen_port {
        write_fmt(ctrl_sock, format_args!("listen_port={}\n", listen_port)).await?;
    }
    if conf.replace_peers {
        write_fmt(ctrl_sock, format_args!("replace_peers=true\n")).await?;
    }
    for peer in conf.peers {
        emit_peer_config(ctrl_sock, peer).await?;
    }
    // End with empty line
    write_fmt(ctrl_sock, format_args!("\n")).await?;

    Ok(())
}

async fn emit_peer_config<S>(ctrl_sock: &mut S, conf: PeerSettings) -> Result<(), WireCtlError>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    write_fmt(
        ctrl_sock,
        format_args!("public_key={}\n", conf.pubkey.to_hex()),
    )
    .await?;
    if conf.remove {
        write_fmt(ctrl_sock, format_args!("remove=true\n")).await?;
        return Ok(());
    }
    if conf.update_only {
        write_fmt(ctrl_sock, format_args!("update_only=true\n")).await?;
    }
    if let Some(preshared_key) = conf.preshared_key {
        write_fmt(
            ctrl_sock,
            format_args!("preshared_key={}\n", preshared_key.to_hex()),
        )
        .await?;
    }
    if let Some(endpoint) = conf.endpoint {
        write_fmt(ctrl_sock, format_args!("endpoint={}\n", endpoint)).await?;
    }
    if let Some(keepalive) = conf.persistent_keepalive {
        write_fmt(
            ctrl_sock,
            format_args!("persistent_keepalive_interval={}\n", keepalive),
        )
        .await?;
    }
    if conf.replace_allowed_ips {
        write_fmt(ctrl_sock, format_args!("replace_allowed_ips=true\n")).await?;
    }
    for allowed_ip in conf.allowed_ips {
        write_fmt(ctrl_sock, format_args!("allowed_ip={}\n", allowed_ip)).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
