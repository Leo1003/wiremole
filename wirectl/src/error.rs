use thiserror::Error;

#[derive(Debug, Error)]
pub enum WireCtlError {
    #[error("Invalid base64 string")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Invalid hex string")]
    HexDecode(#[from] hex::FromHexError),
    #[error("Invalid key length")]
    InvalidKeyLength,
}
