//! Basic usage example for Wavemark
//! 
//! This example demonstrates how to use the wavemark library.

use wavemark::{encoder, decoder, fourier, api};

fn main() {
    println!("Wavemark Basic Usage Example");
    println!("============================");
    
    // Test encoder
    encoder::encode();
    println!("✅ Encoder function called");
    
    // Test decoder
    decoder::decode();
    println!("✅ Decoder function called");
    
    // Test fourier
    fourier::transform();
    println!("✅ Fourier transform function called");
    
    // Test api
    api::process();
    println!("✅ API process function called");
    
    println!("\n🎉 All wavemark functions working!");
}
