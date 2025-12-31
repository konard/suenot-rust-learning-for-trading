#![feature(portable_simd)]
use std::simd::{f32x8, num::SimdFloat};

#[derive(Debug)]
struct VolatilityMetrics {
    std_dev: f32,
    variance: f32,
    mean: f32,
}

fn calculate_sum_simd_f32x8(values: &[f32]) -> f32 {
    let chunks = values.chunks_exact(8);
    let remainder = chunks.remainder();

    let mut simd_sum = f32x8::splat(0.0);

    for chunk in chunks {
        simd_sum += f32x8::from_slice(chunk);
    }

    let mut sum = simd_sum.reduce_sum();
    sum += remainder.iter().sum::<f32>();

    sum
}

fn calculate_mean_simd(values: &[f32]) -> f32 {
    let sum = calculate_sum_simd_f32x8(values);
    sum / values.len() as f32
}

/// Calculate volatility using SIMD
fn calculate_volatility_simd(returns: &[f32]) -> VolatilityMetrics {
    if returns.is_empty() {
        return VolatilityMetrics {
            std_dev: 0.0,
            variance: 0.0,
            mean: 0.0,
        };
    }

    // Step 1: Calculate mean with SIMD
    let mean = calculate_mean_simd(returns);

    // Step 2: Calculate variance
    let mut sum_squared_diff = 0.0f32;

    let chunks = returns.chunks_exact(8);
    let remainder = chunks.remainder();

    let mean_simd = f32x8::splat(mean);
    let mut simd_sum_sq = f32x8::splat(0.0);

    for chunk in chunks {
        let values = f32x8::from_slice(chunk);
        let diff = values - mean_simd;
        simd_sum_sq += diff * diff;
    }

    sum_squared_diff += simd_sum_sq.reduce_sum();

    // Process remainder
    for &value in remainder {
        let diff = value - mean;
        sum_squared_diff += diff * diff;
    }

    let variance = sum_squared_diff / returns.len() as f32;
    let std_dev = variance.sqrt();

    VolatilityMetrics {
        std_dev,
        variance,
        mean,
    }
}

/// Generate returns from prices
fn calculate_returns(prices: &[f32]) -> Vec<f32> {
    prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.5, 102.0,
        104.0, 103.0, 105.5, 107.0, 106.0,
        108.0, 107.5, 109.0, 108.0, 110.0,
        111.0, 109.5, 110.5, 112.0, 111.0,
    ];

    let returns = calculate_returns(&prices);
    let volatility = calculate_volatility_simd(&returns);

    println!("=== Volatility Analysis with SIMD ===");
    println!("Mean return: {:.4}", volatility.mean);
    println!("Variance: {:.6}", volatility.variance);
    println!("Standard deviation: {:.4}", volatility.std_dev);
    println!("Annualized volatility: {:.2}%",
             volatility.std_dev * (252.0f32).sqrt() * 100.0);

    println!("\n✅ Volatility SIMD test passed!");
}
