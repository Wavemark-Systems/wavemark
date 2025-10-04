//! Wavemark - Audio Watermarking Library
//! 
//! This library provides functionality for embedding imperceptible, verifiable information
//! within audio signals for AI voice generators and other audio applications.

pub mod encoder;
pub mod decoder;
pub mod fourier;
pub mod api;
pub mod key;
pub mod streaming;

// Re-export main functionality
pub use encoder::*;
pub use decoder::*;
pub use fourier::*;
pub use api::*;
pub use key::*;
pub use streaming::*;
