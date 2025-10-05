//! Wavemark - Audio Watermarking Library
//!
//! This library provides functionality for embedding imperceptible, verifiable information
//! within audio signals for AI voice generators and other audio applications.

pub mod api;
pub mod core;
pub mod detect;
pub mod embed;
pub mod format;
pub mod key;
pub mod pipeline;
pub mod streaming;
pub mod transforms;

// Re-export main placeholders for now.
pub use api::*;
pub use core::*;
pub use detect::*;
pub use embed::*;
pub use format::*;
pub use key::*;
pub use pipeline::*;
pub use streaming::*;
pub use transforms::*;
