use crate::WireCtlError;
use rand::{CryptoRng, RngCore};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Formatter, Result as FmtResult},
};
use x25519_dalek::StaticSecret;
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
    let keylen = base64::decode_engine_slice(input, buf, &base64::engine::DEFAULT_ENGINE)?;
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
        *self.0.as_bytes() == [0u8; WG_KEY_LEN]
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

impl From<[u8; WG_KEY_LEN]> for PublicKey {
    fn from(bytes: [u8; WG_KEY_LEN]) -> Self {
        PublicKey(x25519_dalek::PublicKey::from(bytes))
    }
}

impl From<PublicKey> for [u8; WG_KEY_LEN] {
    fn from(pubkey: PublicKey) -> Self {
        pubkey.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        <[u8; WG_KEY_LEN]>::try_from(value)
            .map(Self::from)
            .map_err(|_| WireCtlError::InvalidKeyLength)
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
        Ok(Self(x25519_dalek::StaticSecret::from(*buf)))
    }

    pub fn from_hex(input: &str) -> Result<Self, WireCtlError> {
        let mut buf = Zeroizing::new([0u8; WG_KEY_LEN]);
        hex_decode_checklen(input, &mut buf)?;
        Ok(Self(x25519_dalek::StaticSecret::from(*buf)))
    }

    pub fn generate<R>(csprng: R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        Self(StaticSecret::new(csprng))
    }

    pub fn to_base64(&self) -> String {
        let buf = Zeroizing::new(self.0.to_bytes());
        base64::encode(*buf)
    }

    pub fn to_hex(&self) -> String {
        let buf = Zeroizing::new(self.0.to_bytes());
        hex::encode(*buf)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey((&self.0).into())
    }
}

impl From<[u8; WG_KEY_LEN]> for PrivateKey {
    fn from(bytes: [u8; WG_KEY_LEN]) -> Self {
        PrivateKey(x25519_dalek::StaticSecret::from(bytes))
    }
}

impl From<PrivateKey> for [u8; WG_KEY_LEN] {
    fn from(privkey: PrivateKey) -> Self {
        privkey.0.to_bytes()
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        <[u8; WG_KEY_LEN]>::try_from(value)
            .map(Self::from)
            .map_err(|_| WireCtlError::InvalidKeyLength)
    }
}

#[derive(Clone, Default, Zeroize)]
#[zeroize(drop)]
pub struct PresharedKey([u8; WG_KEY_LEN]);

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

    pub fn generate<R>(mut csprng: R) -> Self
    where
        R: RngCore + CryptoRng,
    {
        let mut psk = Self::default();
        csprng.fill_bytes(&mut psk.0);
        psk
    }

    pub fn to_base64(&self) -> String {
        base64::encode(self.0)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [0u8; WG_KEY_LEN]
    }
}

impl AsRef<[u8]> for PresharedKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; WG_KEY_LEN]> for PresharedKey {
    fn from(bytes: [u8; WG_KEY_LEN]) -> Self {
        PresharedKey(bytes)
    }
}

impl From<PresharedKey> for [u8; WG_KEY_LEN] {
    fn from(key: PresharedKey) -> Self {
        key.0
    }
}

impl TryFrom<&[u8]> for PresharedKey {
    type Error = WireCtlError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        <[u8; WG_KEY_LEN]>::try_from(value)
            .map(Self::from)
            .map_err(|_| WireCtlError::InvalidKeyLength)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{
        de::{Error as DeError, Visitor},
        Deserialize, Serialize,
    };
    pub const SERDE_EXPECTED_KEY_LEN: &str = "32 bytes key buffer";

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BufferVisitor;

    impl<'de> Visitor<'de> for BufferVisitor {
        type Value = [u8; WG_KEY_LEN];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "{}", SERDE_EXPECTED_KEY_LEN)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Self::Value::try_from(v)
                .map_err(|_| E::invalid_length(v.len(), &SERDE_EXPECTED_KEY_LEN))
        }
    }

    impl Serialize for PublicKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(self.0.as_bytes())
        }
    }

    impl<'de> Deserialize<'de> for PublicKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let buf = deserializer.deserialize_bytes(BufferVisitor)?;
            Ok(Self::from(buf))
        }
    }

    impl Serialize for PrivateKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(&self.0.to_bytes())
        }
    }

    impl<'de> Deserialize<'de> for PrivateKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let buf = deserializer.deserialize_bytes(BufferVisitor)?;
            Ok(Self::from(buf))
        }
    }

    impl Serialize for PresharedKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_bytes(&self.0)
        }
    }

    impl<'de> Deserialize<'de> for PresharedKey {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let buf = deserializer.deserialize_bytes(BufferVisitor)?;
            Ok(Self::from(buf))
        }
    }
}
