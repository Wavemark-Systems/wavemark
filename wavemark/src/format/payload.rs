#![allow(dead_code)]

//! Payload metadata data model.
//!
//! The types in this module model the logical metadata that will eventually be
//! serialized into watermark payloads. They intentionally avoid encoding or
//! cryptographic details so callers can focus on expressing domain data.
//!
//! # Constructing Metadata
//!
//! Application code typically uses [`PayloadBuilder`] to assemble fields while
//! constraints are enforced automatically. The builder injects a default
//! `issued_at` timestamp so pipeline components can rely on it being present.
//!
//! ```ignore
//! use wavemark::format::payload::{PayloadBuilder, MetadataKey};
//!
//! let mut builder = PayloadBuilder::new();
//! builder
//!     .account_id("acct_demo")?
//!     .text_field(MetadataKey::custom("content.title")?, "Demo Track")?
//!     .int_field("content.duration_seconds", 185)?;
//! let frame = builder.build()?;
//!
//! assert!(frame.issued_at().is_some());
//! ```

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Public wrapper around the metadata stored in a watermark payload.
///
/// Frames are immutable once built. Callers query fields via typed helpers or by
/// iterating the key-value map directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadFrame {
    metadata: BTreeMap<MetadataKey, MetadataValue>,
    constraints: PayloadConstraints,
}

impl PayloadFrame {
    /// Creates an empty frame with default constraints and default fields.
    pub fn new() -> Result<Self, PayloadError> {
        PayloadBuilder::new().build()
    }

    /// Builds a frame from an iterator of metadata fields.
    pub fn from_fields<I>(fields: I) -> Result<Self, PayloadError>
    where
        I: IntoIterator<Item = MetadataField>,
    {
        let mut builder = PayloadBuilder::new();
        builder.extend_fields(fields)?;
        builder.build()
    }

    /// Returns the constraints that were in effect when the frame was created.
    pub fn constraints(&self) -> &PayloadConstraints {
        &self.constraints
    }

    /// Returns the metadata value associated with a key, if present.
    pub fn get(&self, key: &MetadataKey) -> Option<&MetadataValue> {
        self.metadata.get(key)
    }

    /// Returns an iterator over the stored metadata entries in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = (&MetadataKey, &MetadataValue)> {
        self.metadata.iter()
    }

    /// Returns the well-known account identifier field, if present.
    pub fn account_id(&self) -> Option<&AccountId> {
        self.get(&MetadataKey::well_known(WellKnownField::AccountId))
            .and_then(MetadataValue::as_account_id)
    }

    /// Returns the issuance timestamp if a caller did not override the default.
    pub fn issued_at(&self) -> Option<&MetadataTimestamp> {
        self.get(&MetadataKey::well_known(WellKnownField::IssuedAt))
            .and_then(MetadataValue::as_timestamp)
    }
}

/// Builder that enforces payload constraints when assembling metadata fields.
///
/// Callers add fields via typed setters (e.g. [`PayloadBuilder::account_id`]) or
/// through [`PayloadBuilder::put_field`]. Defaults such as the issued-at
/// timestamp are injected automatically but can be overridden explicitly.
#[derive(Debug, Clone)]
pub struct PayloadBuilder {
    constraints: PayloadConstraints,
    metadata: BTreeMap<MetadataKey, MetadataValue>,
}

impl PayloadBuilder {
    /// Construct a builder with default constraints and default fields (issued_at).
    pub fn new() -> Self {
        Self::with_constraints(PayloadConstraints::default())
    }

    /// Construct a builder with custom constraints.
    pub fn with_constraints(constraints: PayloadConstraints) -> Self {
        let mut builder = Self {
            constraints,
            metadata: BTreeMap::new(),
        };
        // Default issued_at helps keep downstream pipelines consistent. Callers can override.
        let _ = builder.metadata.insert(
            MetadataKey::well_known(WellKnownField::IssuedAt),
            MetadataValue::from(MetadataTimestamp::now()),
        );
        builder
    }

    /// Insert a general metadata field after validating constraints.
    pub fn put_field(&mut self, field: MetadataField) -> Result<&mut Self, PayloadError> {
        self.validate(&field)?;
        self.metadata.insert(field.key, field.value);
        Ok(self)
    }

    /// Extend the builder with multiple fields at once.
    pub fn extend_fields<I>(&mut self, fields: I) -> Result<&mut Self, PayloadError>
    where
        I: IntoIterator<Item = MetadataField>,
    {
        for field in fields {
            self.put_field(field)?;
        }
        Ok(self)
    }

    /// Convenience setter for the account identifier.
    pub fn account_id<T>(&mut self, account_id: T) -> Result<&mut Self, PayloadError>
    where
        T: TryInto<AccountId, Error = PayloadError>,
    {
        let account_id = account_id.try_into()?;
        let field = MetadataField::new(
            MetadataKey::well_known(WellKnownField::AccountId),
            MetadataValue::Account(account_id),
        );
        self.put_field(field)
    }

    /// Convenience setter for issuance timestamps when callers want to override the default.
    pub fn issued_at(&mut self, timestamp: MetadataTimestamp) -> Result<&mut Self, PayloadError> {
        let field = MetadataField::new(
            MetadataKey::well_known(WellKnownField::IssuedAt),
            MetadataValue::Timestamp(timestamp),
        );
        self.put_field(field)
    }

    /// Convenience setter for expiration timestamps.
    pub fn expires_at(&mut self, timestamp: MetadataTimestamp) -> Result<&mut Self, PayloadError> {
        let field = MetadataField::new(
            MetadataKey::well_known(WellKnownField::ExpiresAt),
            MetadataValue::Timestamp(timestamp),
        );
        self.put_field(field)
    }

    /// Attach an arbitrary UTF-8 metadata field.
    pub fn text_field<K>(
        &mut self,
        key: K,
        value: impl Into<String>,
    ) -> Result<&mut Self, PayloadError>
    where
        K: TryInto<MetadataKey, Error = PayloadError>,
    {
        let key = key.try_into()?;
        self.put_field(MetadataField::new(key, MetadataValue::Text(value.into())))
    }

    /// Attach metadata as raw bytes for future specialized handling.
    pub fn binary_field<K>(
        &mut self,
        key: K,
        value: impl Into<Vec<u8>>,
    ) -> Result<&mut Self, PayloadError>
    where
        K: TryInto<MetadataKey, Error = PayloadError>,
    {
        let key = key.try_into()?;
        self.put_field(MetadataField::new(key, MetadataValue::Blob(value.into())))
    }

    /// Attach a boolean field.
    pub fn bool_field<K>(&mut self, key: K, value: bool) -> Result<&mut Self, PayloadError>
    where
        K: TryInto<MetadataKey, Error = PayloadError>,
    {
        let key = key.try_into()?;
        self.put_field(MetadataField::new(key, MetadataValue::Bool(value)))
    }

    /// Attach an integer field.
    pub fn int_field<K>(&mut self, key: K, value: i64) -> Result<&mut Self, PayloadError>
    where
        K: TryInto<MetadataKey, Error = PayloadError>,
    {
        let key = key.try_into()?;
        self.put_field(MetadataField::new(key, MetadataValue::Integer(value)))
    }

    /// Finalize the builder, returning a validated payload frame.
    pub fn build(self) -> Result<PayloadFrame, PayloadError> {
        if self.metadata.len() > self.constraints.max_fields {
            return Err(PayloadError::TooManyFields {
                limit: self.constraints.max_fields,
            });
        }

        Ok(PayloadFrame {
            metadata: self.metadata,
            constraints: self.constraints,
        })
    }

    fn validate(&self, field: &MetadataField) -> Result<(), PayloadError> {
        let payload_key = &field.key;
        let key_len = payload_key.as_str().len();
        if key_len == 0 {
            return Err(PayloadError::EmptyKey);
        }
        if key_len > self.constraints.max_key_bytes {
            return Err(PayloadError::KeyTooLong {
                key: payload_key.clone(),
                limit: self.constraints.max_key_bytes,
            });
        }

        let value_len = field.value.estimated_size_bytes();
        match &field.value {
            MetadataValue::Text(_) if value_len > self.constraints.max_text_bytes => {
                Err(PayloadError::ValueTooLarge {
                    key: payload_key.clone(),
                    limit: self.constraints.max_text_bytes,
                })
            }
            MetadataValue::Blob(_) if value_len > self.constraints.max_blob_bytes => {
                Err(PayloadError::ValueTooLarge {
                    key: payload_key.clone(),
                    limit: self.constraints.max_blob_bytes,
                })
            }
            MetadataValue::Account(account_id) => account_id.validate(),
            _ => Ok(()),
        }
    }
}

/// Domain-specific metadata constraints that keep payloads manageable.
///
/// These values cap the size and shape of metadata so it can be transported in
/// watermark payloads with predictable bounds. Callers can customize the limits
/// if their deployment requires more flexibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadConstraints {
    pub max_fields: usize,
    pub max_key_bytes: usize,
    pub max_text_bytes: usize,
    pub max_blob_bytes: usize,
}

impl Default for PayloadConstraints {
    fn default() -> Self {
        Self {
            max_fields: 32,
            max_key_bytes: 64,
            max_text_bytes: 512,
            max_blob_bytes: 1024,
        }
    }
}

/// Errors raised when metadata does not satisfy payload constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadError {
    EmptyKey,
    KeyTooLong { key: MetadataKey, limit: usize },
    ValueTooLarge { key: MetadataKey, limit: usize },
    TooManyFields { limit: usize },
    InvalidAccountId(Cow<'static, str>),
    InvalidCustomKey(Cow<'static, str>),
    InvalidTimestamp(Cow<'static, str>),
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadError::EmptyKey => write!(f, "metadata keys cannot be empty"),
            PayloadError::KeyTooLong { key, limit } => {
                write!(f, "metadata key '{}' exceeds maximum length {}", key, limit)
            }
            PayloadError::ValueTooLarge { key, limit } => {
                write!(f, "metadata value for '{}' exceeds {} bytes", key, limit)
            }
            PayloadError::TooManyFields { limit } => {
                write!(
                    f,
                    "payload exceeds the maximum of {} metadata fields",
                    limit
                )
            }
            PayloadError::InvalidAccountId(reason) => {
                write!(f, "account id is invalid: {}", reason)
            }
            PayloadError::InvalidCustomKey(reason) => {
                write!(f, "custom key is invalid: {}", reason)
            }
            PayloadError::InvalidTimestamp(reason) => {
                write!(f, "timestamp is invalid: {}", reason)
            }
        }
    }
}

impl std::error::Error for PayloadError {}

/// Enumerates the well-known metadata fields that the library understands natively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WellKnownField {
    AccountId,
    SessionId,
    ContentId,
    IssuedAt,
    ExpiresAt,
}

impl WellKnownField {
    /// Returns the canonical lowercase wire key for the field.
    pub fn as_str(&self) -> &'static str {
        match self {
            WellKnownField::AccountId => "account_id",
            WellKnownField::SessionId => "session_id",
            WellKnownField::ContentId => "content_id",
            WellKnownField::IssuedAt => "issued_at",
            WellKnownField::ExpiresAt => "expires_at",
        }
    }
}

/// Structured metadata key that can reference either well-known or custom fields.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetadataKey {
    WellKnown(WellKnownField),
    Custom(String),
}

impl MetadataKey {
    /// Create a well-known key.
    pub fn well_known(field: WellKnownField) -> Self {
        MetadataKey::WellKnown(field)
    }

    /// Creates a custom key after validating it with the default constraints.
    pub fn custom(key: impl Into<String>) -> Result<Self, PayloadError> {
        let key = key.into();
        if key.is_empty() {
            return Err(PayloadError::EmptyKey);
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
        {
            return Err(PayloadError::InvalidCustomKey(Cow::from(
                "keys must be lowercase ASCII alphanumerics, '.', or '_'",
            )));
        }
        Ok(MetadataKey::Custom(key))
    }

    /// Returns the canonical string representation for serialization.
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            MetadataKey::WellKnown(field) => Cow::Borrowed(field.as_str()),
            MetadataKey::Custom(ref key) => Cow::Borrowed(key.as_str()),
        }
    }
}

impl fmt::Display for MetadataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for MetadataKey {
    type Error = PayloadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "account_id" => Ok(MetadataKey::well_known(WellKnownField::AccountId)),
            "session_id" => Ok(MetadataKey::well_known(WellKnownField::SessionId)),
            "content_id" => Ok(MetadataKey::well_known(WellKnownField::ContentId)),
            "issued_at" => Ok(MetadataKey::well_known(WellKnownField::IssuedAt)),
            "expires_at" => Ok(MetadataKey::well_known(WellKnownField::ExpiresAt)),
            other => MetadataKey::custom(other),
        }
    }
}

impl TryFrom<String> for MetadataKey {
    type Error = PayloadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        MetadataKey::try_from(value.as_str())
    }
}

/// Structured representation of metadata values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValue {
    Account(AccountId),
    Timestamp(MetadataTimestamp),
    Text(String),
    Integer(i64),
    Bool(bool),
    Blob(Vec<u8>),
}

impl MetadataValue {
    fn estimated_size_bytes(&self) -> usize {
        match self {
            MetadataValue::Account(account) => account.as_str().len(),
            MetadataValue::Timestamp(_) => 16,
            MetadataValue::Text(text) => text.len(),
            MetadataValue::Integer(_) => 8,
            MetadataValue::Bool(_) => 1,
            MetadataValue::Blob(bytes) => bytes.len(),
        }
    }

    fn as_account_id(&self) -> Option<&AccountId> {
        if let MetadataValue::Account(account) = self {
            Some(account)
        } else {
            None
        }
    }

    fn as_timestamp(&self) -> Option<&MetadataTimestamp> {
        if let MetadataValue::Timestamp(ts) = self {
            Some(ts)
        } else {
            None
        }
    }
}

impl From<MetadataTimestamp> for MetadataValue {
    fn from(value: MetadataTimestamp) -> Self {
        MetadataValue::Timestamp(value)
    }
}

/// Single metadata entry tying a key to a typed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataField {
    pub key: MetadataKey,
    pub value: MetadataValue,
}

impl MetadataField {
    /// Construct a metadata field.
    pub fn new(key: MetadataKey, value: MetadataValue) -> Self {
        Self { key, value }
    }
}

/// Account identifier used by watermark operators to scope requests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(String);

impl AccountId {
    const MAX_LEN: usize = 64;

    /// Create a new account identifier from a string.
    pub fn new(id: impl Into<String>) -> Result<Self, PayloadError> {
        let value = id.into();
        if value.trim().is_empty() {
            return Err(PayloadError::InvalidAccountId(Cow::from(
                "account identifiers cannot be empty",
            )));
        }
        if value.len() > Self::MAX_LEN {
            return Err(PayloadError::InvalidAccountId(Cow::from(
                "account identifier exceeds maximum length",
            )));
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(PayloadError::InvalidAccountId(Cow::from(
                "account identifier must be alphanumeric plus '-' or '_'",
            )));
        }
        Ok(AccountId(value))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), PayloadError> {
        // Validation happens on construction, so this is always OK.
        Ok(())
    }
}

impl TryFrom<&str> for AccountId {
    type Error = PayloadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        AccountId::new(value)
    }
}

impl TryFrom<String> for AccountId {
    type Error = PayloadError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        AccountId::new(value)
    }
}

/// Timestamp wrapper that keeps conversions localized.
///
/// Timestamps are stored as [`SystemTime`] values but can be created from raw
/// Unix epoch seconds to accommodate external metadata sources.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetadataTimestamp(SystemTime);

impl MetadataTimestamp {
    /// Returns the current system time.
    pub fn now() -> Self {
        MetadataTimestamp(SystemTime::now())
    }

    /// Create a timestamp from a raw `SystemTime`.
    pub fn from_system_time(time: SystemTime) -> Result<Self, PayloadError> {
        Self::validate(time)?;
        Ok(MetadataTimestamp(time))
    }

    /// Create a timestamp from seconds since the Unix epoch.
    pub fn from_unix_seconds(secs: i64) -> Result<Self, PayloadError> {
        let time = if secs >= 0 {
            UNIX_EPOCH + Duration::from_secs(secs as u64)
        } else {
            let magnitude = secs
                .checked_neg()
                .and_then(|v| u64::try_from(v).ok())
                .ok_or_else(|| {
                    PayloadError::InvalidTimestamp(Cow::from(
                        "seconds precede the representable range",
                    ))
                })?;
            UNIX_EPOCH
                .checked_sub(Duration::from_secs(magnitude))
                .ok_or_else(|| {
                    PayloadError::InvalidTimestamp(Cow::from("seconds precede the Unix epoch"))
                })?
        };
        Self::from_system_time(time)
    }

    /// Returns the inner `SystemTime` value.
    pub fn as_system_time(&self) -> SystemTime {
        self.0
    }

    /// Returns seconds relative to the Unix epoch, if representable.
    pub fn to_unix_seconds(&self) -> Result<i64, PayloadError> {
        match self.0.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration
                .as_secs()
                .try_into()
                .map_err(|_| PayloadError::InvalidTimestamp(Cow::from("timestamp too large"))),
            Err(err) => err
                .duration()
                .as_secs()
                .try_into()
                .map(|secs: i64| -secs)
                .map_err(|_| {
                    PayloadError::InvalidTimestamp(Cow::from("timestamp too far in the past"))
                }),
        }
    }

    fn validate(time: SystemTime) -> Result<(), PayloadError> {
        // The watermark payload format expects timestamps in a safe range around the Unix epoch.
        const MAX_FUTURE_SECS: u64 = 253402300800; // year 9999
        if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
            if duration.as_secs() > MAX_FUTURE_SECS {
                return Err(PayloadError::InvalidTimestamp(Cow::from(
                    "timestamp exceeds supported range",
                )));
            }
            Ok(())
        } else {
            // Negative durations are allowed down to 100 years before epoch.
            const MAX_PAST_SECS: u64 = 3_155_760_000; // approx 100 years
            let past = UNIX_EPOCH
                .duration_since(time)
                .map_err(|_| PayloadError::InvalidTimestamp(Cow::from("timestamp underflow")))?;
            if past.as_secs() > MAX_PAST_SECS {
                return Err(PayloadError::InvalidTimestamp(Cow::from(
                    "timestamp precedes supported range",
                )));
            }
            Ok(())
        }
    }
}

impl TryFrom<SystemTime> for MetadataTimestamp {
    type Error = PayloadError;

    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        MetadataTimestamp::from_system_time(value)
    }
}

/// Example helper so application code can construct metadata fields succinctly.
pub fn metadata_field<K, V>(key: K, value: V) -> Result<MetadataField, PayloadError>
where
    K: TryInto<MetadataKey, Error = PayloadError>,
    V: Into<MetadataValue>,
{
    Ok(MetadataField::new(key.try_into()?, value.into()))
}

impl From<AccountId> for MetadataValue {
    fn from(account_id: AccountId) -> Self {
        MetadataValue::Account(account_id)
    }
}

impl From<String> for MetadataValue {
    fn from(value: String) -> Self {
        MetadataValue::Text(value)
    }
}

impl From<&str> for MetadataValue {
    fn from(value: &str) -> Self {
        MetadataValue::Text(value.to_owned())
    }
}

impl From<bool> for MetadataValue {
    fn from(value: bool) -> Self {
        MetadataValue::Bool(value)
    }
}

impl From<i64> for MetadataValue {
    fn from(value: i64) -> Self {
        MetadataValue::Integer(value)
    }
}

impl From<Vec<u8>> for MetadataValue {
    fn from(value: Vec<u8>) -> Self {
        MetadataValue::Blob(value)
    }
}
