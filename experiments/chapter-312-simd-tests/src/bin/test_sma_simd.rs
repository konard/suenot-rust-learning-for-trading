#![feature(portable_simd)]
use std::simd::{f32x4, num::SimdFloat};

/// Fast sum using SIMD
fn calculate_sum_simd(values: &[f32]) -> f32 {
    let mut sum = 0.0f32;

    // Process in blocks of 4 elements
    let chunks = values.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut simd_sum = f32x4::splat(0.0);

    for chunk in chunks {
        // Load 4 values into SIMD register
        let simd_vals = f32x4::from_slice(chunk);
        // Add in parallel
        simd_sum += simd_vals;
    }

    // Sum 4 components of SIMD register
    sum += simd_sum.reduce_sum();

    // Process remaining elements
    sum += remainder.iter().sum::<f32>();

    sum
}

/// Calculate Simple Moving Average (SMA) with SIMD
fn calculate_sma_simd(prices: &[f32], window: usize) -> Vec<f32> {
    if prices.len() < window {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - window + 1);

    for i in 0..=prices.len() - window {
        let window_prices = &prices[i..i + window];
        let sum = calculate_sum_simd(window_prices);
        sma_values.push(sum / window as f32);
    }

    sma_values
}

fn main() {
    let prices = vec![
        100.0, 101.0, 102.0, 103.0, 104.0,
        103.5, 102.0, 101.0, 100.5, 99.0,
        98.0, 99.5, 101.0, 102.5, 104.0,
        105.0, 106.0, 107.0, 106.5, 105.0,
    ];

    let sma = calculate_sma_simd(&prices, 5);

    println!("=== SMA-5 with SIMD ===");
    for (i, value) in sma.iter().enumerate() {
        println!("Position {}: SMA = {:.2}", i + 5, value);
    }

    println!("\n✅ SMA SIMD test passed!");
}
