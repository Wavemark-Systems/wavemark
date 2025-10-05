#![allow(dead_code)]

//! Binary codec for watermark metadata payloads.
//!
//! The codec emitted here defines how a [`PayloadFrame`](crate::format::payload::PayloadFrame)
//! is transformed into bytes for embedding. The format is intentionally
//! conservative: all multi-byte integers use little-endian byte order, and a
//! compact static header advertises protocol version and encryption envelope.
//!
//! # Byte Layout (Version 1)
//!
//! ```text
//! +------------+-----------------------------------------------------------+
//! | Offset     | Contents                                                  |
//! +==========++===========================================================+
//! | 0..=1      | Magic literal `0x57 0x4D` (ASCII "WM")                    |
//! | 2          | Major version (currently `1`)                             |
//! | 3          | Minor version (`0` for initial release)                  |
//! | 4          | Envelope flag (`0` = plain, `1` = encrypted hash)         |
//! | 5..=7      | Reserved for future extensions (zeroed)                   |
//! | 8..        | Envelope payload (see below)                              |
//! +------------+-----------------------------------------------------------+
//! ```
//!
//! Plain envelopes store the field count followed by key/value records. Each
//! key is encoded as a length-prefixed UTF-8 string and every value carries a
//! type tag so future versions can introduce new representations without
//! breaking older readers. Encrypted envelopes prepend authentication metadata
//! before the sealed bytes and reuse the same inner plain encoding once the
//! ciphertext is opened.
//!
//! # Versioning and Extensibility
//!
//! [`FormatVersion`] tracks major/minor revisions. The codec currently targets
//! `1.0` and rejects frames whose major version differs, leaving room for future
//! backwards-compatibility rules. Reserved header bytes and per-field type tags
//! make it easy to append optional data without perturbing existing decoders.
//!
//! # Failure Modes
//!
//! Decoding can fail with [`CodecError`] when headers are malformed, byte slices
//! truncate early, unsupported type tags appear, or when lower layers (payload
//! validation or encryption) report issues. Encryption failures bubble up as
//! [`EncryptionError`](crate::format::encryption::EncryptionError), allowing
//! callers to distinguish configuration problems from cryptographic integrity
//! failures.
//!
//! # Example
//!
//! ```ignore
//! use wavemark::format::codec::{CodecOptions, FrameCodec};
//! use wavemark::format::payload::{PayloadBuilder, PayloadFrame};
//!
//! let frame: PayloadFrame = PayloadBuilder::new()
//!     .account_id("acct_demo")?
//!     .text_field("content.title", "Demo Track")?
//!     .build()?;
//!
//! let codec = FrameCodec::new(CodecOptions::default());
//! let bytes = codec.encode(&frame, &Default::default())?;
//! let decoded = codec.decode(&bytes, &Default::default())?;
//! assert_eq!(frame, decoded);
//! ```

use crate::format::encryption::{
    EncryptedHashConfig, EncryptionArtifacts, EncryptionContext, EncryptionError, EncryptionMode,
};
use crate::format::payload::{
    AccountId, MetadataField, MetadataKey, MetadataTimestamp, MetadataValue, PayloadBuilder,
    PayloadConstraints, PayloadError, PayloadFrame,
};
use std::convert::TryFrom;
use std::fmt;

const MAGIC: &[u8; 2] = b"WM";
const HEADER_LEN: usize = 8;

/// Semantic codec version (major.minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatVersion {
    pub major: u8,
    pub minor: u8,
}

impl FormatVersion {
    /// Latest codec version supported by the library.
    pub const LATEST: FormatVersion = FormatVersion { major: 1, minor: 0 };

    fn write_header(self, buffer: &mut Vec<u8>, envelope: FrameEnvelope) {
        buffer.extend_from_slice(MAGIC);
        buffer.push(self.major);
        buffer.push(self.minor);
        buffer.push(envelope as u8);
        buffer.extend_from_slice(&[0u8; 3]);
    }
}

/// Envelope variants describing how payload bytes are wrapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameEnvelope {
    Plain = 0,
    EncryptedHash = 1,
}

impl FrameEnvelope {
    fn from_flag(flag: u8) -> Option<Self> {
        match flag {
            0 => Some(FrameEnvelope::Plain),
            1 => Some(FrameEnvelope::EncryptedHash),
            _ => None,
        }
    }
}

/// Codec configuration shared between encoding and decoding.
#[derive(Clone, Debug)]
pub struct CodecOptions {
    pub version: FormatVersion,
    pub constraints: PayloadConstraints,
    pub encryption: EncryptionMode,
}

impl Default for CodecOptions {
    fn default() -> Self {
        Self {
            version: FormatVersion::LATEST,
            constraints: PayloadConstraints::default(),
            encryption: EncryptionMode::None,
        }
    }
}

/// Binary encoder/decoder for [`PayloadFrame`]s.
#[derive(Clone, Debug)]
pub struct FrameCodec {
    options: CodecOptions,
}

impl FrameCodec {
    /// Construct a codec with the supplied options.
    pub fn new(options: CodecOptions) -> Self {
        Self { options }
    }

    /// Returns the active codec options.
    pub fn options(&self) -> &CodecOptions {
        &self.options
    }

    /// Serializes a payload frame into bytes, applying encryption based on the
    /// configured [`EncryptionMode`].
    pub fn encode(
        &self,
        frame: &PayloadFrame,
        context: &EncryptionContext,
    ) -> Result<Vec<u8>, CodecError> {
        let plain_body = self.encode_plain(frame)?;
        match &self.options.encryption {
            EncryptionMode::None => Ok(self.wrap_plain(plain_body)),
            EncryptionMode::EncryptedHash(config) => {
                self.wrap_encrypted(plain_body, config, context)
            }
        }
    }

    /// Decodes bytes into a payload frame, verifying headers, version, and
    /// optionally decrypting/enforcing integrity.
    pub fn decode(
        &self,
        bytes: &[u8],
        context: &EncryptionContext,
    ) -> Result<PayloadFrame, CodecError> {
        if bytes.len() < HEADER_LEN {
            return Err(CodecError::UnexpectedEof);
        }

        let (magic, rest) = bytes.split_at(2);
        if magic != MAGIC {
            return Err(CodecError::InvalidHeader("magic mismatch"));
        }

        let version = FormatVersion {
            major: rest[0],
            minor: rest[1],
        };
        if version.major != self.options.version.major {
            return Err(CodecError::UnsupportedVersion {
                expected_major: self.options.version.major,
                found: version,
            });
        }

        let envelope = FrameEnvelope::from_flag(rest[2])
            .ok_or(CodecError::InvalidHeader("unknown envelope flag"))?;

        let payload = &bytes[HEADER_LEN..];
        if matches!(envelope, FrameEnvelope::Plain)
            && matches!(self.options.encryption, EncryptionMode::EncryptedHash(_))
        {
            return Err(CodecError::InvalidHeader(
                "plaintext payload encountered but codec expects encrypted hash",
            ));
        }

        let plain_body = match envelope {
            FrameEnvelope::Plain => payload.to_vec(),
            FrameEnvelope::EncryptedHash => self.unwrap_encrypted(payload, context)?,
        };

        self.decode_plain(&plain_body)
    }

    fn wrap_plain(&self, body: Vec<u8>) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(HEADER_LEN + body.len());
        self.options
            .version
            .write_header(&mut buffer, FrameEnvelope::Plain);
        buffer.extend_from_slice(&body);
        buffer
    }

    fn wrap_encrypted(
        &self,
        body: Vec<u8>,
        config: &EncryptedHashConfig,
        context: &EncryptionContext,
    ) -> Result<Vec<u8>, CodecError> {
        let artifacts = config.strategy.seal(&body, context)?;
        let tag_len = artifacts.tag.as_ref().map(|tag| tag.len()).unwrap_or(0);
        if tag_len > u16::MAX as usize {
            return Err(CodecError::LengthOverflow("tag"));
        }
        let metadata_len = artifacts
            .metadata
            .as_ref()
            .map(|meta| meta.len())
            .unwrap_or(0);
        if metadata_len > u16::MAX as usize {
            return Err(CodecError::LengthOverflow("encryption metadata"));
        }
        if artifacts.sealed_payload.len() > u32::MAX as usize {
            return Err(CodecError::LengthOverflow("sealed payload"));
        }

        let mut buffer = Vec::with_capacity(
            HEADER_LEN + 2 + 2 + 4 + tag_len + metadata_len + artifacts.sealed_payload.len(),
        );
        self.options
            .version
            .write_header(&mut buffer, FrameEnvelope::EncryptedHash);

        buffer.extend_from_slice(&(tag_len as u16).to_le_bytes());
        buffer.extend_from_slice(&(metadata_len as u16).to_le_bytes());
        buffer.extend_from_slice(&(artifacts.sealed_payload.len() as u32).to_le_bytes());
        if let Some(tag) = artifacts.tag.as_ref() {
            buffer.extend_from_slice(tag);
        }
        if let Some(meta) = artifacts.metadata.as_ref() {
            buffer.extend_from_slice(meta);
        }
        buffer.extend_from_slice(&artifacts.sealed_payload);

        Ok(buffer)
    }

    fn unwrap_encrypted(
        &self,
        payload: &[u8],
        context: &EncryptionContext,
    ) -> Result<Vec<u8>, CodecError> {
        let config = match &self.options.encryption {
            EncryptionMode::EncryptedHash(config) => config,
            EncryptionMode::None => {
                return Err(CodecError::InvalidHeader(
                    "received encrypted payload but codec is in plaintext mode",
                ))
            }
        };

        if payload.len() < 8 {
            return Err(CodecError::UnexpectedEof);
        }

        let tag_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
        let metadata_len = u16::from_le_bytes([payload[2], payload[3]]) as usize;
        let sealed_len =
            u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;

        let expected_len = 8 + tag_len + metadata_len + sealed_len;
        if payload.len() < expected_len {
            return Err(CodecError::UnexpectedEof);
        }

        let mut offset = 8;
        let tag_slice = &payload[offset..offset + tag_len];
        offset += tag_len;
        let metadata_slice = &payload[offset..offset + metadata_len];
        offset += metadata_len;
        let sealed_slice = &payload[offset..offset + sealed_len];

        let artifacts = EncryptionArtifacts {
            sealed_payload: sealed_slice.to_vec(),
            tag: if tag_len > 0 {
                Some(tag_slice.to_vec())
            } else {
                None
            },
            metadata: if metadata_len > 0 {
                Some(metadata_slice.to_vec())
            } else {
                None
            },
        };

        let plain = config
            .strategy
            .open(&artifacts.sealed_payload, &artifacts, context)?;
        Ok(plain)
    }

    fn decode_plain(&self, body: &[u8]) -> Result<PayloadFrame, CodecError> {
        if body.len() < 2 {
            return Err(CodecError::UnexpectedEof);
        }

        let field_count = u16::from_le_bytes([body[0], body[1]]) as usize;
        let mut offset = 2;
        let mut builder = PayloadBuilder::with_constraints(self.options.constraints);

        for _ in 0..field_count {
            if offset >= body.len() {
                return Err(CodecError::UnexpectedEof);
            }
            let key_len = body[offset] as usize;
            offset += 1;
            if offset + key_len > body.len() {
                return Err(CodecError::UnexpectedEof);
            }
            let key_bytes = &body[offset..offset + key_len];
            offset += key_len;
            let key_str = String::from_utf8(key_bytes.to_vec())
                .map_err(|_| CodecError::InvalidUtf8("metadata key".into()))?;
            let key = MetadataKey::try_from(key_str.as_str())?;

            if offset >= body.len() {
                return Err(CodecError::UnexpectedEof);
            }
            let tag = body[offset];
            offset += 1;
            let kind =
                ValueKind::from_tag(tag).ok_or_else(|| CodecError::UnsupportedFieldType(tag))?;

            let value = match kind {
                ValueKind::AccountId => {
                    if offset >= body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let len = body[offset] as usize;
                    offset += 1;
                    if offset + len > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let account_str = String::from_utf8(body[offset..offset + len].to_vec())
                        .map_err(|_| CodecError::InvalidUtf8("account_id".into()))?;
                    offset += len;
                    MetadataValue::Account(AccountId::new(account_str)?)
                }
                ValueKind::Timestamp => {
                    if offset + 8 > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&body[offset..offset + 8]);
                    offset += 8;
                    let seconds = i64::from_le_bytes(bytes);
                    MetadataValue::Timestamp(MetadataTimestamp::from_unix_seconds(seconds)?)
                }
                ValueKind::Text => {
                    if offset + 2 > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let len = u16::from_le_bytes([body[offset], body[offset + 1]]) as usize;
                    offset += 2;
                    if offset + len > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let text = String::from_utf8(body[offset..offset + len].to_vec())
                        .map_err(|_| CodecError::InvalidUtf8("text value".into()))?;
                    offset += len;
                    MetadataValue::Text(text)
                }
                ValueKind::Integer => {
                    if offset + 8 > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&body[offset..offset + 8]);
                    offset += 8;
                    MetadataValue::Integer(i64::from_le_bytes(bytes))
                }
                ValueKind::Bool => {
                    if offset >= body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let byte = body[offset];
                    offset += 1;
                    match byte {
                        0 => MetadataValue::Bool(false),
                        1 => MetadataValue::Bool(true),
                        _ => return Err(CodecError::InvalidHeader("boolean value must be 0 or 1")),
                    }
                }
                ValueKind::Blob => {
                    if offset + 2 > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let len = u16::from_le_bytes([body[offset], body[offset + 1]]) as usize;
                    offset += 2;
                    if offset + len > body.len() {
                        return Err(CodecError::UnexpectedEof);
                    }
                    let blob = body[offset..offset + len].to_vec();
                    offset += len;
                    MetadataValue::Blob(blob)
                }
            };

            builder.put_field(MetadataField::new(key, value))?;
        }

        builder.build().map_err(CodecError::from)
    }

    fn encode_plain(&self, frame: &PayloadFrame) -> Result<Vec<u8>, CodecError> {
        let field_count = frame.iter().count();
        if field_count > u16::MAX as usize {
            return Err(CodecError::LengthOverflow("field count"));
        }

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&(field_count as u16).to_le_bytes());

        for (key, value) in frame.iter() {
            let key_bytes = key.as_str();
            if key_bytes.len() > u8::MAX as usize {
                return Err(CodecError::LengthOverflow("metadata key"));
            }
            buffer.push(key_bytes.len() as u8);
            buffer.extend_from_slice(key_bytes.as_bytes());

            let kind = ValueKind::from_value(value);
            buffer.push(kind as u8);

            match value {
                MetadataValue::Account(account) => {
                    let bytes = account.as_str().as_bytes();
                    if bytes.len() > u8::MAX as usize {
                        return Err(CodecError::LengthOverflow("account_id"));
                    }
                    buffer.push(bytes.len() as u8);
                    buffer.extend_from_slice(bytes);
                }
                MetadataValue::Timestamp(ts) => {
                    let seconds = ts.to_unix_seconds()?;
                    buffer.extend_from_slice(&seconds.to_le_bytes());
                }
                MetadataValue::Text(text) => {
                    if text.len() > u16::MAX as usize {
                        return Err(CodecError::LengthOverflow("text value"));
                    }
                    buffer.extend_from_slice(&(text.len() as u16).to_le_bytes());
                    buffer.extend_from_slice(text.as_bytes());
                }
                MetadataValue::Integer(value) => {
                    buffer.extend_from_slice(&value.to_le_bytes());
                }
                MetadataValue::Bool(value) => {
                    buffer.push(if *value { 1 } else { 0 });
                }
                MetadataValue::Blob(bytes) => {
                    if bytes.len() > u16::MAX as usize {
                        return Err(CodecError::LengthOverflow("blob value"));
                    }
                    buffer.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                    buffer.extend_from_slice(bytes);
                }
            }
        }

        Ok(buffer)
    }
}

/// Field value type tags encoded alongside metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ValueKind {
    AccountId = 0x01,
    Timestamp = 0x02,
    Text = 0x10,
    Integer = 0x11,
    Bool = 0x12,
    Blob = 0x13,
}

impl ValueKind {
    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(ValueKind::AccountId),
            0x02 => Some(ValueKind::Timestamp),
            0x10 => Some(ValueKind::Text),
            0x11 => Some(ValueKind::Integer),
            0x12 => Some(ValueKind::Bool),
            0x13 => Some(ValueKind::Blob),
            _ => None,
        }
    }

    fn from_value(value: &MetadataValue) -> Self {
        match value {
            MetadataValue::Account(_) => ValueKind::AccountId,
            MetadataValue::Timestamp(_) => ValueKind::Timestamp,
            MetadataValue::Text(_) => ValueKind::Text,
            MetadataValue::Integer(_) => ValueKind::Integer,
            MetadataValue::Bool(_) => ValueKind::Bool,
            MetadataValue::Blob(_) => ValueKind::Blob,
        }
    }
}

/// Errors produced during encoding/decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    InvalidHeader(&'static str),
    UnsupportedVersion {
        expected_major: u8,
        found: FormatVersion,
    },
    UnexpectedEof,
    LengthOverflow(&'static str),
    InvalidUtf8(String),
    UnsupportedFieldType(u8),
    Payload(PayloadError),
    Encryption(EncryptionError),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::InvalidHeader(reason) => write!(f, "invalid codec header: {}", reason),
            CodecError::UnsupportedVersion {
                expected_major,
                found,
            } => write!(
                f,
                "unsupported payload version: expected major {} but found {}.{}",
                expected_major, found.major, found.minor
            ),
            CodecError::UnexpectedEof => write!(f, "unexpected end of payload"),
            CodecError::LengthOverflow(field) => {
                write!(f, "{} exceeds representable length", field)
            }
            CodecError::InvalidUtf8(field) => write!(f, "{} is not valid UTF-8", field),
            CodecError::UnsupportedFieldType(tag) => {
                write!(f, "unsupported field type tag 0x{:02X}", tag)
            }
            CodecError::Payload(err) => err.fmt(f),
            CodecError::Encryption(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<PayloadError> for CodecError {
    fn from(err: PayloadError) -> Self {
        CodecError::Payload(err)
    }
}

impl From<EncryptionError> for CodecError {
    fn from(err: EncryptionError) -> Self {
        CodecError::Encryption(err)
    }
}
