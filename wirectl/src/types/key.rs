use crate::WireCtlError;
use std::{convert::TryFrom, fmt::{Debug, Formatter, Result as FmtResult}};
use zeroize::{Zeroize, Zeroizing};

pub const WG_KEY_LEN: usize = 32;
pub const WG_KEY_BASE64_LEN: usize = ((WG_KEY_LEN + 2) / 3) * 4;
pub const WG_KEY_HEX_LEN: usize = WG_KEY_LEN * 2;

fn base64_decode_checklen(input: &str, buf: &mut [u8; WG_KEY_LEN]) -> Result<(), WireCtlError> {
    // Check base64 length won't exceed the buffer after decoded.
    if input.len() != WG_KEY_BASE64_LEN || input.as_bytes()[WG_KEY_BASE64_LEN - 1] != b'=' {
        return Err(WireCtlError::InvalidKeyLength);
    }

    // We have checked the base64 length before.
    // So the following operation should not panic.
    let keylen = base64::decode_config_slice(input, base64::STANDARD, buf)?;
    if keylen != WG_KEY_LEN {
        return Err(WireCtlError::InvalidKeyLength);
    }
    Ok(())
}

fn hex_decode_checklen(input: &str, buf: &mut [u8; WG_KEY_LEN]) -> Result<(), WireCtlError> {
    // Check hexadecimal string length won't exceed the buffer after decoded.
    if input.len() != WG_KEY_HEX_LEN {
        return Err(WireCtlError::InvalidKeyLength);
    }

    hex::decode_to_slice(input, buf)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKey(x25519_dalek::PublicKey);

impl PublicKey {
    pub fn from_base64(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = [0u8; WG_KEY_LEN];
        base64_decode_checklen(input, &mut buf)?;
        Ok(PublicKey(x25519_dalek::PublicKey::from(buf)))
    }

    pub fn from_hex(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = [0u8; WG_KEY_LEN];
        hex_decode_checklen(input, &mut buf)?;
        Ok(PublicKey(x25519_dalek::PublicKey::from(buf)))
    }

    pub fn to_base64(&self) -> String {
        base64::encode(self.0.as_bytes())
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0.as_bytes())
    }

    pub fn is_empty(&self) -> bool {
        *self.0.as_bytes() == [0u8; 32]
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<&PrivateKey> for PublicKey {
    fn from(key: &PrivateKey) -> Self {
        key.public_key()
    }
}

impl From<[u8; 32]> for PublicKey {
    fn from(bytes: [u8; 32]) -> Self {
        PublicKey(x25519_dalek::PublicKey::from(bytes))
    }
}

impl From<PublicKey> for [u8; 32] {
    fn from(pubkey: PublicKey) -> Self {
        pubkey.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == WG_KEY_LEN {
            Ok(Self::from(<[u8; 32]>::try_from(value).unwrap()))
        } else {
            Err(WireCtlError::InvalidKeyLength)
        }
    }
}

#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct PrivateKey(x25519_dalek::StaticSecret);

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("PrivateKey")
            .field(&"<value omitted>")
            .finish()
    }
}

impl PrivateKey {
    pub fn from_base64(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = Zeroizing::new([0u8; WG_KEY_LEN]);
        base64_decode_checklen(input, &mut buf)?;
        Ok(PrivateKey(x25519_dalek::StaticSecret::from(*buf)))
    }

    pub fn from_hex(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = Zeroizing::new([0u8; WG_KEY_LEN]);
        hex_decode_checklen(input, &mut buf)?;
        Ok(PrivateKey(x25519_dalek::StaticSecret::from(*buf)))
    }

    pub fn to_base64(&self) -> String {
        let buf = Zeroizing::new(self.0.to_bytes());
        base64::encode(&*buf)
    }

    pub fn to_hex(&self) -> String {
        let buf = Zeroizing::new(self.0.to_bytes());
        hex::encode(&*buf)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey((&self.0).into())
    }
}

impl From<[u8; 32]> for PrivateKey {
    fn from(bytes: [u8; 32]) -> Self {
        PrivateKey(x25519_dalek::StaticSecret::from(bytes))
    }
}

impl From<PrivateKey> for [u8; 32] {
    fn from(privkey: PrivateKey) -> Self {
        privkey.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == WG_KEY_LEN {
            Ok(Self::from(<[u8; 32]>::try_from(value).unwrap()))
        } else {
            Err(WireCtlError::InvalidKeyLength)
        }
    }
}

#[derive(Clone, Default, Zeroize)]
#[zeroize(drop)]
pub struct PresharedKey([u8; 32]);

impl Debug for PresharedKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("PresharedKey")
            .field(&"<value omitted>")
            .finish()
    }
}

impl PresharedKey {
    pub fn from_base64(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = Zeroizing::new([0u8; WG_KEY_LEN]);
        base64_decode_checklen(input, &mut buf)?;
        Ok(PresharedKey(*buf))
    }

    pub fn from_hex(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = Zeroizing::new([0u8; WG_KEY_LEN]);
        hex_decode_checklen(input, &mut buf)?;
        Ok(PresharedKey(*buf))
    }

    pub fn to_base64(&self) -> String {
        base64::encode(self.0)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl AsRef<[u8]> for PresharedKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 32]> for PresharedKey {
    fn from(bytes: [u8; 32]) -> Self {
        PresharedKey(bytes)
    }
}

impl From<PresharedKey> for [u8; 32] {
    fn from(key: PresharedKey) -> Self {
        key.0
    }
}

impl TryFrom<&[u8]> for PresharedKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() == WG_KEY_LEN {
            Ok(Self::from(<[u8; 32]>::try_from(value).unwrap()))
        } else {
            Err(WireCtlError::InvalidKeyLength)
        }
    }
}
