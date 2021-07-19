use std::num::ParseIntError;

use async_process::ExitStatus;
use ipnetwork::IpNetworkError;
use rtnetlink::Error as NlError;
use std::net::AddrParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WireCtlError {
    #[error("Invalid base64 string")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Invalid hex string")]
    HexDecode(#[from] hex::FromHexError),
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Invalid UTF-8 string")]
    InvalidString,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Interface not found")]
    NotFound,
    #[error("Device Error: {0}")]
    DeviceError(i32),
    #[error("Failed to launch userspace implementation. Exit status: {0}")]
    UserspaceLaunch(ExitStatus),
    #[error("Unknown Error")]
    Unknown,
}

impl From<ParseIntError> for WireCtlError {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidProtocol
    }
}

impl From<AddrParseError> for WireCtlError {
    fn from(_: AddrParseError) -> Self {
        Self::InvalidProtocol
    }
}

impl From<IpNetworkError> for WireCtlError {
    fn from(_: IpNetworkError) -> Self {
        Self::InvalidProtocol
    }
}

impl From<NlError> for WireCtlError {
    fn from(e: NlError) -> Self {
        match e {
            NlError::NetlinkError(errmsg) => errmsg.to_io().into(),
            _ => Self::Unknown,
        }
    }
}
