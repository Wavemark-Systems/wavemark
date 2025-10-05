//! Payload formatting façade.
//!
//! This module provides a high-level API that composes schema definitions
//! ([`payload`]), encryption abstraction ([`encryption`]), and the binary codec
//! ([`codec`]) into an ergonomic builder for downstream consumers.

pub mod codec;
pub mod encryption;
pub mod payload;

use codec::{CodecError, CodecOptions, FrameCodec};
use encryption::{EncryptionContext, EncryptionMode};
use payload::{MetadataField, PayloadBuilder, PayloadError, PayloadFrame};

/// Builder that collects metadata fields, configures encryption, and yields
/// ready-to-embed byte payloads.
///
/// ```ignore
/// use wavemark::format::{FormatBuilder, encryption::EncryptionMode};
///
/// let mut builder = FormatBuilder::new();
/// builder
///     .payload_builder()
///     .account_id("acct_demo")?
///     .text_field("content.title", "Demo Track")?;
/// let output = builder
///     .encryption_mode(EncryptionMode::None)
///     .build()?;
///
/// assert!(!output.bytes.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct FormatBuilder {
    payload: PayloadBuilder,
    codec: FrameCodec,
    encryption_context: EncryptionContext,
}

impl FormatBuilder {
    /// Start a builder with default payload constraints and no encryption.
    pub fn new() -> Self {
        Self::with_options(CodecOptions::default())
    }

    /// Start a builder with explicit codec options.
    pub fn with_options(options: CodecOptions) -> Self {
        Self {
            payload: PayloadBuilder::with_constraints(options.constraints),
            codec: FrameCodec::new(options),
            encryption_context: EncryptionContext::default(),
        }
    }

    /// Specify the encryption mode to use when serializing the payload.
    pub fn encryption_mode(mut self, mode: EncryptionMode) -> Self {
        let mut options = self.codec.options().clone();
        options.encryption = mode;
        self.codec = FrameCodec::new(options);
        self
    }

    /// Provide ambient encryption context (AAD, channel identifiers, etc.).
    pub fn encryption_context(mut self, context: EncryptionContext) -> Self {
        self.encryption_context = context;
        self
    }

    /// Insert a metadata field directly.
    pub fn field(mut self, field: MetadataField) -> Result<Self, PayloadError> {
        self.payload.put_field(field)?;
        Ok(self)
    }

    /// Extend the builder with multiple fields.
    pub fn fields<I>(mut self, fields: I) -> Result<Self, PayloadError>
    where
        I: IntoIterator<Item = MetadataField>,
    {
        self.payload.extend_fields(fields)?;
        Ok(self)
    }

    /// Access the underlying payload builder to use typed helpers.
    pub fn payload_builder(&mut self) -> &mut PayloadBuilder {
        &mut self.payload
    }

    /// Consume the builder, returning both the `PayloadFrame` and serialized bytes.
    pub fn build(self) -> Result<FormatOutput, CodecError> {
        let frame = self.payload.build().map_err(CodecError::from)?;
        let bytes = self
            .codec
            .encode(&frame, &self.encryption_context)
            .map_err(CodecError::from)?;
        Ok(FormatOutput { frame, bytes })
    }
}

impl Default for FormatBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Output of [`FormatBuilder::build`], containing both logical metadata and bytes.
#[derive(Debug, Clone)]
pub struct FormatOutput {
    pub frame: PayloadFrame,
    pub bytes: Vec<u8>,
}

impl FormatOutput {
    /// Consumes the output, returning just the byte payload when the logical frame is no longer needed.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
