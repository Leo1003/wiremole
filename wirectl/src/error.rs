use std::num::ParseIntError;

use ipnetwork::IpNetworkError;
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
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Device Error: {0}")]
    DeviceError(i32),
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
