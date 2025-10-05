#![allow(dead_code)]

//! Encryption abstractions for watermark payloads.
//!
//! This module intentionally keeps cryptographic details abstract. Callers pick a
//! [`EncryptionMode`] at configuration time and supply strategy implementations
//! through the [`EncryptedHashStrategy`] trait when opting into encrypted-hash
//! protection. The default mode is [`EncryptionMode::None`], which leaves
//! payload bytes untouched.
//!
//! ```ignore
//! use std::sync::Arc;
//! use wavemark::format::encryption::{EncryptionMode, EncryptedHashConfig, EncryptionContext};
//!
//! // Application-defined strategy implementing `EncryptedHashStrategy`.
//! let strategy = Arc::new(MyStrategy::new());
//! let mode = EncryptionMode::EncryptedHash(EncryptedHashConfig {
//!     strategy,
//!     key_id: Some("account-key-1".into()),
//!     nonce: None,
//! });
//! ```
//!
//! Strategies communicate failures via [`EncryptionError`], allowing callers to
//! surface configuration mistakes (`InvalidConfiguration`), payload issues
//! (`RejectedPayload`), or low-level cryptographic faults (`CryptoFailure`).

use std::fmt;
use std::sync::Arc;

/// High-level encryption selector used by the format layer.
#[derive(Debug, Clone)]
pub enum EncryptionMode {
    /// No encryption or hashing is applied; plaintext payload flows through.
    None,
    /// Wrap payload bytes using a user-supplied encrypted hash strategy.
    EncryptedHash(EncryptedHashConfig),
}

impl EncryptionMode {
    /// Returns `true` when payload bytes will remain unprotected.
    pub fn is_none(&self) -> bool {
        matches!(self, EncryptionMode::None)
    }

    /// Returns `true` when payload bytes require an encrypted-hash transform.
    pub fn is_encrypted_hash(&self) -> bool {
        matches!(self, EncryptionMode::EncryptedHash(_))
    }
}

/// Configuration for the encrypted-hash mode.
#[derive(Clone)]
pub struct EncryptedHashConfig {
    /// Strategy responsible for hashing, key usage, and encryption steps.
    pub strategy: Arc<dyn EncryptedHashStrategy>,
    /// Optional identifier for the key material used by the strategy.
    pub key_id: Option<String>,
    /// Optional nonce/IV if the strategy requires caller-provided randomness.
    pub nonce: Option<Vec<u8>>,
}

impl fmt::Debug for EncryptedHashConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedHashConfig")
            .field("strategy", &self.strategy.algorithm_id())
            .field("key_id", &self.key_id)
            .field("nonce", &self.nonce.as_ref().map(|n| n.len()))
            .finish()
    }
}

/// Runtime context passed to encryption strategies.
#[derive(Debug, Clone, Default)]
pub struct EncryptionContext {
    /// Optional channel identifier (e.g., stream session) for domain separation.
    pub channel_id: Option<String>,
    /// Additional authenticated data to bind the ciphertext to higher-level state.
    pub associated_data: Option<Vec<u8>>,
}

/// Result returned after sealing payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionArtifacts {
    /// Bytes that should be transported in place of the plaintext payload.
    pub sealed_payload: Vec<u8>,
    /// Detached authentication tag or digest when produced by the strategy.
    pub tag: Option<Vec<u8>>,
    /// Strategy-specific metadata that consumers might need to persist.
    pub metadata: Option<Vec<u8>>,
}

impl EncryptionArtifacts {
    /// Helper for producing artifacts when no encryption takes place.
    pub fn passthrough(payload: Vec<u8>) -> Self {
        Self {
            sealed_payload: payload,
            tag: None,
            metadata: None,
        }
    }
}

/// Trait implemented by encryption providers registered with the format layer.
///
/// Custom strategies typically implement this trait indirectly via
/// [`EncryptedHashStrategy`]. `seal` should return a deterministic encoding of
/// the sealed payload plus any auxiliary metadata, while `open` must validate
/// tags or authentication data and return the original bytes.
pub trait PayloadEncryption {
    /// Applies the provider's protection to `payload`, returning sealed bytes.
    fn seal(
        &self,
        payload: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptionArtifacts, EncryptionError>;

    /// Reverses `seal`, verifying tags and recovering the original payload.
    fn open(
        &self,
        sealed: &[u8],
        artifacts: &EncryptionArtifacts,
        context: &EncryptionContext,
    ) -> Result<Vec<u8>, EncryptionError>;

    /// Human-readable identifier for logging and debugging purposes.
    fn scheme_name(&self) -> &'static str;
}

/// Convenience trait that specializes [`PayloadEncryption`] for the encrypted-hash mode.
///
/// To plug in a new strategy, implement this trait for a type that wraps your
/// hashing and encryption primitives. The format layer will hold the type in an
/// [`Arc`], allowing shared use across threads.
pub trait EncryptedHashStrategy: PayloadEncryption + Send + Sync {
    /// Identifier describing the hashing/encryption construction.
    fn algorithm_id(&self) -> &'static str;
}

/// Default payload encryption provider covering the `None` mode.
#[derive(Debug, Default)]
pub struct NoEncryption;

impl PayloadEncryption for NoEncryption {
    fn seal(
        &self,
        payload: &[u8],
        _context: &EncryptionContext,
    ) -> Result<EncryptionArtifacts, EncryptionError> {
        Ok(EncryptionArtifacts::passthrough(payload.to_vec()))
    }

    fn open(
        &self,
        sealed: &[u8],
        _artifacts: &EncryptionArtifacts,
        _context: &EncryptionContext,
    ) -> Result<Vec<u8>, EncryptionError> {
        Ok(sealed.to_vec())
    }

    fn scheme_name(&self) -> &'static str {
        "none"
    }
}

/// Errors surfaced by encryption strategies when sealing or opening payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionError {
    /// Requested mode is not available (e.g., feature gate or missing key).
    UnsupportedMode(&'static str),
    /// Caller supplied an invalid configuration parameter.
    InvalidConfiguration(String),
    /// Strategy rejected the input payload (size, format, etc.).
    RejectedPayload(String),
    /// Underlying cryptographic operation failed.
    CryptoFailure(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionError::UnsupportedMode(mode) => {
                write!(f, "encryption mode '{}' is not supported", mode)
            }
            EncryptionError::InvalidConfiguration(reason) => {
                write!(f, "invalid encryption configuration: {}", reason)
            }
            EncryptionError::RejectedPayload(reason) => {
                write!(f, "payload rejected by encryption strategy: {}", reason)
            }
            EncryptionError::CryptoFailure(reason) => {
                write!(f, "cryptographic operation failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for EncryptionError {}
