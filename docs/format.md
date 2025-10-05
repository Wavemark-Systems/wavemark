# Format Layer Guide

This guide explains how to configure metadata payloads with the `wavemark::format`
module. It covers standard metadata fields, adding custom fields, enabling
encryption, and wiring the output into the watermarking pipeline. You will also
find advice on handling failures and preparing for future format versions.

## Overview

`wavemark::format` exposes a high-level [`FormatBuilder`](../wavemark/src/format/mod.rs)
that composes three subsystems:

- **Schema & data model**: `format::payload` defines well-known metadata keys,
  type-safe values, and the `PayloadBuilder` used internally by `FormatBuilder`.
- **Encryption abstraction**: `format::encryption` offers pluggable strategies
  so you can ship plaintext payloads or wrap them in authenticated encryption.
- **Serialization**: `format::codec` turns logical payloads into bytes using a
  versioned framing format shared by embedding and detection pipelines.

The builder gives application authors a single entry point for assembling
metadata and producing ready-to-embed byte payloads.

## Registering Standard Fields

Well-known fields such as `account_id`, `issued_at`, and `expires_at` have
dedicated convenience methods on `PayloadBuilder`. `FormatBuilder` exposes the
underlying builder through `payload_builder()` so you can use those helpers.

```rust
use wavemark::format::FormatBuilder;

let mut builder = FormatBuilder::new();
builder
    .payload_builder()
    .account_id("acct_demo")?
    .issued_at(wavemark::format::payload::MetadataTimestamp::now())?
    .expires_at(wavemark::format::payload::MetadataTimestamp::from_unix_seconds(1_700_000_000)?);

let output = builder.build()?;
assert!(output.frame.account_id().is_some());
```

`PayloadBuilder` injects a default `issued_at` timestamp automatically, so
pipelines relying on this field can assume it is present even when the caller
does nothing. Callers may override the value explicitly as shown above.

## Injecting Custom Fields

Custom metadata keys support lowercase ASCII characters, digits, `.` and `_`.
Builder helpers convert basic Rust types into the appropriate
[`MetadataValue`](../wavemark/src/format/payload.rs).

```rust
use wavemark::format::FormatBuilder;
use wavemark::format::payload::MetadataKey;

let mut builder = FormatBuilder::new();
builder
    .payload_builder()
    .account_id("acct_demo")?
    .text_field(MetadataKey::custom("content.title")?, "Demo Track")?
    .int_field("content.duration_seconds", 185)?
    .bool_field("content.preview", true)?;

let output = builder.build()?;
assert_eq!(output.frame.get(&MetadataKey::custom("content.preview")?), Some(&wavemark::format::payload::MetadataValue::Bool(true)));
```

If you need to construct fields elsewhere, combine `MetadataKey` and
`MetadataValue` manually, then call `FormatBuilder::field` or
`FormatBuilder::fields`.

## Enabling Encryption

`EncryptionMode` determines how payload bytes are wrapped. By default, the
builder emits plaintext payloads. To enable encrypted-hash envelopes, supply an
[`EncryptedHashConfig`](../wavemark/src/format/encryption.rs) containing your
strategy implementation.

```rust
use std::sync::Arc;
use wavemark::format::encryption::{EncryptionContext, EncryptionMode, EncryptedHashConfig, EncryptedHashStrategy};
use wavemark::format::FormatBuilder;

struct MyStrategy;

impl wavemark::format::encryption::PayloadEncryption for MyStrategy {
    // implement `seal`/`open` using your crypto primitives
    # fn seal(&self, payload: &[u8], _ctx: &EncryptionContext) -> Result<wavemark::format::encryption::EncryptionArtifacts, wavemark::format::encryption::EncryptionError> {
    #     Ok(wavemark::format::encryption::EncryptionArtifacts::passthrough(payload.to_vec()))
    # }
    # fn open(&self, sealed: &[u8], _artifacts: &wavemark::format::encryption::EncryptionArtifacts, _ctx: &EncryptionContext) -> Result<Vec<u8>, wavemark::format::encryption::EncryptionError> {
    #     Ok(sealed.to_vec())
    # }
    # fn scheme_name(&self) -> &'static str { "my-strategy" }
}

impl EncryptedHashStrategy for MyStrategy {
    fn algorithm_id(&self) -> &'static str { "my-strategy" }
}

let strategy = Arc::new(MyStrategy);
let config = EncryptedHashConfig {
    strategy,
    key_id: Some("customer-key-1".into()),
    nonce: None,
};

let mut builder = FormatBuilder::new();
builder
    .payload_builder()
    .account_id("acct_secure")?
    .text_field("content.id", "track-42")?;

let bytes = builder
    .encryption_mode(EncryptionMode::EncryptedHash(config))
    .encryption_context(EncryptionContext {
        channel_id: Some("session-01".into()),
        associated_data: Some(b"pipeline-AAD".to_vec()),
    })
    .build()?;
```

Strategies report issues through `EncryptionError`. Surface these back to your
callers to highlight configuration or integrity problems.

## Integrating with the Watermarking Pipeline

`FormatBuilder::build` returns a `FormatOutput` containing both the logical
`PayloadFrame` and serialized bytes. Pass `FormatOutput::bytes` into the embed
pipeline and persist the `PayloadFrame` or selected fields for indexing.

```rust
let format_output = FormatBuilder::new()
    .payload_builder()
    .account_id("acct_pipeline")?
    .text_field("content.title", "Pipeline Demo")?
    .build()?;

embed::payload_mapper::map_payload_audio(audio_buffer, &format_output.bytes);
```

When detecting watermarks, feed recovered bytes into `FrameCodec::decode` and
reconstruct the `PayloadFrame` for verification.

```rust
use wavemark::format::codec::{CodecOptions, FrameCodec};
use wavemark::format::encryption::EncryptionContext;

let codec = FrameCodec::new(CodecOptions::default());
let recovered_frame = codec.decode(&recovered_bytes, &EncryptionContext::default())?;
```

## Handling Failures

Three error types bubble up from the format layer:

- `PayloadError` signals schema violations (empty keys, oversized values,
  invalid account IDs). Handle these during request validation.
- `EncryptionError` indicates encryption configuration issues or integrity
  failures during decrypt/verify operations. Retry with fresh keys or alert
  operators.
- `CodecError` represents framing/serialization problems. Many variants wrap
  the other two error types; inspect the enum to decide whether to retry or
  reject the payload.

Example handling pattern:

```rust
match FormatBuilder::new().build() {
    Ok(output) => embed_bytes(output.bytes),
    Err(err) => match err {
        CodecError::Payload(payload_err) => {
            log::warn!("bad metadata: {payload_err}");
            return Err(UserFacingError::InvalidMetadata);
        }
        CodecError::Encryption(enc_err) => {
            log::error!("encryption failed: {enc_err}");
            retry_with_fallback_config();
        }
        other => {
            log::error!("serialization failed: {other}");
            return Err(UserFacingError::InternalError);
        }
    },
}
```

## Version Migration Guidance

The codec header includes a `FormatVersion` (major/minor). The library accepts
payloads whose **major** version matches `FormatVersion::LATEST.major`. Minor
updates are treated as compatible.

To prepare for future migrations:

1. **Persist version metadata** alongside stored payload bytes. Retain the
   header or record `FormatVersion` explicitly.
2. **Handle `UnsupportedVersion`** errors from `FrameCodec::decode` by providing
   a fallback decoder or asking clients to upgrade.
3. **Avoid custom binary parsing** of payload bytes. Always use `FrameCodec`
   utilities so you automatically benefit from compatibility fixes.
4. **Reserve space for future fields** in your downstream storage. The payload
   model supports up to 32 fields by default, and new optional fields can appear
   in later versions.

When upgrading to a new major release, look for migration notes in this document
and in the crate changelog. Library-provided tooling will offer re-encoding
utilities if the on-disk format changes.

## Additional Resources

- [`src/format/payload.rs`](../wavemark/src/format/payload.rs): Full metadata
  data model, including constraints and well-known keys.
- [`src/format/encryption.rs`](../wavemark/src/format/encryption.rs): Extension
  points for custom encrypted-hash strategies.
- [`src/format/codec.rs`](../wavemark/src/format/codec.rs): Byte layout and
  serialization internals.
- [`tests/format_payload.rs`](../wavemark/tests/format_payload.rs): Working
  examples of builder usage, encryption mocks, and error assertions.

