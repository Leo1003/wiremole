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
