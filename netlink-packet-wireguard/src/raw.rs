use libc::{in6_addr, in_addr, sockaddr, sockaddr_in, sockaddr_in6, timespec, AF_INET, AF_INET6};
use netlink_packet_utils::DecodeError;
use std::{
    mem::size_of,
    mem::size_of_val,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    slice::from_raw_parts,
    time::{Duration, SystemTime},
};

pub fn emit_in_addr(addr: &Ipv4Addr, buf: &mut [u8]) {
    let caddr = in_addr {
        s_addr: u32::from(*addr),
    };

    copy_raw_slice(buf, &caddr);
}

pub fn parse_in_addr(buf: &[u8]) -> Result<Ipv4Addr, DecodeError> {
    if buf.len() != size_of::<in_addr>() {
        return Err(DecodeError::from("Invalid buffer length"));
    }

    let caddr: &in_addr = unsafe { from_raw_slice(buf)? };
    Ok(Ipv4Addr::from(caddr.s_addr))
}

pub fn emit_in6_addr(addr: &Ipv6Addr, buf: &mut [u8]) {
    let caddr = in6_addr {
        s6_addr: addr.octets(),
    };

    copy_raw_slice(buf, &caddr);
}

pub fn parse_in6_addr(buf: &[u8]) -> Result<Ipv6Addr, DecodeError> {
    if buf.len() != size_of::<in6_addr>() {
        return Err(DecodeError::from("Invalid buffer length"));
    }

    let caddr: &in6_addr = unsafe { from_raw_slice(buf)? };
    Ok(Ipv6Addr::from(caddr.s6_addr))
}

pub fn emit_sockaddr_in(addr: &SocketAddrV4, buf: &mut [u8]) {
    let csockaddr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: addr.port(),
        sin_addr: in_addr {
            s_addr: u32::from(*addr.ip()),
        },
        sin_zero: [0u8; 8],
    };

    copy_raw_slice(buf, &csockaddr);
}

fn parse_sockaddr_in(buf: &[u8]) -> Result<SocketAddrV4, DecodeError> {
    let csockaddr: &sockaddr_in = unsafe { from_raw_slice(buf)? };

    let ipaddr = Ipv4Addr::from(csockaddr.sin_addr.s_addr);
    Ok(SocketAddrV4::new(ipaddr, csockaddr.sin_port))
}

pub fn emit_sockaddr_in6(addr: &SocketAddrV6, buf: &mut [u8]) {
    let csockaddr = sockaddr_in6 {
        sin6_family: AF_INET6 as u16,
        sin6_port: addr.port(),
        sin6_flowinfo: addr.flowinfo(),
        sin6_addr: in6_addr {
            s6_addr: addr.ip().octets(),
        },
        sin6_scope_id: addr.scope_id(),
    };

    copy_raw_slice(buf, &csockaddr);
}

fn parse_sockaddr_in6(buf: &[u8]) -> Result<SocketAddrV6, DecodeError> {
    let csockaddr: &sockaddr_in6 = unsafe { from_raw_slice(buf)? };

    let ipaddr = Ipv6Addr::from(csockaddr.sin6_addr.s6_addr);
    Ok(SocketAddrV6::new(
        ipaddr,
        csockaddr.sin6_port,
        csockaddr.sin6_flowinfo,
        csockaddr.sin6_scope_id,
    ))
}

pub fn parse_sockaddr(buf: &[u8]) -> Result<SocketAddr, DecodeError> {
    let csockaddr: &sockaddr = unsafe { from_raw_slice(buf)? };

    if csockaddr.sa_family == AF_INET as u16 {
        Ok(SocketAddr::V4(parse_sockaddr_in(buf)?))
    } else if csockaddr.sa_family == AF_INET6 as u16 {
        Ok(SocketAddr::V6(parse_sockaddr_in6(buf)?))
    } else {
        Err(DecodeError::from("Unknown address family"))
    }
}

pub fn emit_timespec(time: &SystemTime, buf: &mut [u8]) {
    let epoch_elapsed = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let ctimespec = timespec {
        tv_sec: epoch_elapsed.as_secs() as i64,
        tv_nsec: epoch_elapsed.subsec_nanos() as i64,
    };

    copy_raw_slice(buf, &ctimespec);
}

pub fn parse_timespec(buf: &[u8]) -> Result<SystemTime, DecodeError> {
    if buf.len() != size_of::<timespec>() {
        return Err(DecodeError::from("Invalid buffer length"));
    }

    let ctimespec: &timespec = unsafe { from_raw_slice(buf)? };
    let epoch_elapsed_s = Duration::from_secs(ctimespec.tv_sec as u64);
    let epoch_elapsed_ns = Duration::from_nanos(ctimespec.tv_nsec as u64);
    Ok(SystemTime::UNIX_EPOCH + epoch_elapsed_s + epoch_elapsed_ns)
}

fn copy_raw_slice<T: Sized>(dst: &mut [u8], src: &T) {
    let src_slice = unsafe { as_raw_slice(src) };
    dst[..size_of_val(src)].copy_from_slice(src_slice);
}

unsafe fn from_raw_slice<T: Sized>(src: &[u8]) -> Result<&T, DecodeError> {
    if src.len() < size_of::<T>() {
        return Err(DecodeError::from("Buffer too small"));
    }
    let buf = &src[..size_of::<T>()];

    let (prefix, data, _) = buf.align_to::<T>();
    if prefix.is_empty() {
        Ok(&data[0])
    } else {
        Err(DecodeError::from("Buffer not aligned"))
    }
}

unsafe fn as_raw_slice<T: Sized>(src: &T) -> &[u8] {
    from_raw_parts((src as *const T) as *const u8, size_of::<T>())
}
