use std::error::Error;
use std::sync::Arc;

use wavemark::format::codec::{CodecError, CodecOptions, FrameCodec};
use wavemark::format::encryption::{
    EncryptedHashConfig, EncryptedHashStrategy, EncryptionArtifacts, EncryptionContext,
    EncryptionError, EncryptionMode, PayloadEncryption,
};
use wavemark::format::payload::{MetadataKey, MetadataTimestamp, MetadataValue, PayloadError};
use wavemark::format::FormatBuilder;

#[derive(Debug, Clone)]
struct TestStrategy {
    seed: Vec<u8>,
}

impl TestStrategy {
    fn new(seed: impl Into<Vec<u8>>) -> Self {
        Self { seed: seed.into() }
    }

    fn transform(&self, input: &[u8]) -> Vec<u8> {
        if self.seed.is_empty() {
            return input.to_vec();
        }
        input
            .iter()
            .enumerate()
            .map(|(idx, byte)| byte ^ self.seed[idx % self.seed.len()])
            .collect()
    }
}

impl PayloadEncryption for TestStrategy {
    fn seal(
        &self,
        payload: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptionArtifacts, EncryptionError> {
        Ok(EncryptionArtifacts {
            sealed_payload: self.transform(payload),
            tag: Some(vec![self.seed.len() as u8]),
            metadata: context.associated_data.clone(),
        })
    }

    fn open(
        &self,
        sealed: &[u8],
        artifacts: &EncryptionArtifacts,
        context: &EncryptionContext,
    ) -> Result<Vec<u8>, EncryptionError> {
        let expected_tag = self.seed.len() as u8;
        match artifacts.tag.as_deref() {
            Some([tag]) if *tag == expected_tag => {}
            _ => return Err(EncryptionError::CryptoFailure("tag mismatch".into())),
        }

        if artifacts.metadata != context.associated_data {
            return Err(EncryptionError::CryptoFailure("aad mismatch".into()));
        }

        Ok(self.transform(sealed))
    }

    fn scheme_name(&self) -> &'static str {
        "test-xor"
    }
}

impl EncryptedHashStrategy for TestStrategy {
    fn algorithm_id(&self) -> &'static str {
        "test-xor"
    }
}

#[test]
fn build_payload_with_multiple_fields() -> Result<(), Box<dyn Error>> {
    let mut builder = FormatBuilder::new();
    builder
        .payload_builder()
        .account_id("acct_demo")?
        .text_field("content.title", "Demo Track")?
        .int_field("content.duration_seconds", 185)?;

    let output = builder.build()?;
    let frame = output.frame;

    let title_key = MetadataKey::custom("content.title")?;
    let duration_key = MetadataKey::custom("content.duration_seconds")?;

    assert_eq!(frame.account_id().unwrap().as_str(), "acct_demo");
    assert_eq!(
        frame.get(&title_key),
        Some(&MetadataValue::Text("Demo Track".to_owned()))
    );
    assert_eq!(frame.get(&duration_key), Some(&MetadataValue::Integer(185)));

    // Default issued_at should always be present.
    assert!(frame.issued_at().is_some());

    Ok(())
}

#[test]
fn encrypted_hashes_are_deterministic_with_seed() -> Result<(), Box<dyn Error>> {
    let strategy: Arc<dyn EncryptedHashStrategy> = Arc::new(TestStrategy::new([0xAA, 0x55]));
    let config = EncryptedHashConfig {
        strategy: strategy.clone(),
        key_id: Some("test-key".into()),
        nonce: Some(vec![0x01, 0x02, 0x03]),
    };

    let mut options = CodecOptions::default();
    options.encryption = EncryptionMode::EncryptedHash(config.clone());

    let context = EncryptionContext {
        channel_id: Some("channel-42".into()),
        associated_data: Some(b"aad".to_vec()),
    };

    let mut builder_a = FormatBuilder::with_options(options.clone());
    builder_a
        .payload_builder()
        .account_id("acct_det")?
        .text_field("content.label", "Encrypted")?;
    let output_a = builder_a
        .encryption_context(context.clone())
        .encryption_mode(EncryptionMode::EncryptedHash(config.clone()))
        .build()?;

    let mut builder_b = FormatBuilder::with_options(options.clone());
    builder_b
        .payload_builder()
        .account_id("acct_det")?
        .text_field("content.label", "Encrypted")?;
    let output_b = builder_b
        .encryption_context(context.clone())
        .encryption_mode(EncryptionMode::EncryptedHash(config))
        .build()?;

    assert_eq!(output_a.bytes, output_b.bytes);
    // Encrypted envelope flag should be set to 1.
    assert_eq!(output_a.bytes[4], 1);

    Ok(())
}

#[test]
fn round_trip_serialization_plaintext() -> Result<(), Box<dyn Error>> {
    let mut builder = FormatBuilder::new();
    let issued_at = MetadataTimestamp::from_unix_seconds(1_700_000_000)?;
    builder
        .payload_builder()
        .account_id("acct_round")?
        .issued_at(issued_at.clone())?
        .bool_field("content.flag", true)?;

    let output = builder.build()?;
    let frame = output.frame.clone();
    let bytes = output.bytes.clone();

    let codec = FrameCodec::new(CodecOptions::default());
    let decoded = codec.decode(&bytes, &EncryptionContext::default())?;

    assert_eq!(frame, decoded);

    Ok(())
}

#[test]
fn builder_and_codec_error_conditions() {
    // Missing required field / invalid account id.
    let mut builder = FormatBuilder::new();
    let err = builder.payload_builder().account_id("").unwrap_err();
    assert!(matches!(err, PayloadError::InvalidAccountId(_)));

    // Unsupported encryption expectation during decode.
    let output = FormatBuilder::new().build().expect("payload should build");

    let strategy: Arc<dyn EncryptedHashStrategy> = Arc::new(TestStrategy::new([0xAA]));
    let mut options = CodecOptions::default();
    options.encryption = EncryptionMode::EncryptedHash(EncryptedHashConfig {
        strategy,
        key_id: None,
        nonce: None,
    });
    let codec = FrameCodec::new(options);

    let err = codec
        .decode(&output.bytes, &EncryptionContext::default())
        .unwrap_err();
    assert!(matches!(
        err,
        CodecError::InvalidHeader(message)
            if message == "plaintext payload encountered but codec expects encrypted hash"
    ));
}
