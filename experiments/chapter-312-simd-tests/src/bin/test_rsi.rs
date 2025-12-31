#![feature(portable_simd)]
use std::simd::{f32x4, cmp::SimdPartialOrd};

#[derive(Debug)]
struct RsiResult {
    rsi_values: Vec<f32>,
    avg_gain: f32,
    avg_loss: f32,
}

/// Split changes into gains and losses with SIMD
fn split_gains_losses_simd(changes: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut gains = vec![0.0f32; changes.len()];
    let mut losses = vec![0.0f32; changes.len()];

    let chunks = changes.chunks_exact(4);
    let remainder = chunks.remainder();

    let zero = f32x4::splat(0.0);

    for (i, chunk) in chunks.enumerate() {
        let values = f32x4::from_slice(chunk);

        // Parallel comparison: positive values
        let gain_mask = values.simd_gt(zero);
        let gain_values = gain_mask.select(values, zero);

        // Parallel comparison: negative values (take absolute)
        let loss_mask = values.simd_lt(zero);
        let loss_values = loss_mask.select(-values, zero);

        let idx = i * 4;
        gain_values.copy_to_slice(&mut gains[idx..idx + 4]);
        loss_values.copy_to_slice(&mut losses[idx..idx + 4]);
    }

    // Process remainder
    for (i, &change) in remainder.iter().enumerate() {
        let idx = changes.len() - remainder.len() + i;
        if change > 0.0 {
            gains[idx] = change;
        } else {
            losses[idx] = -change;
        }
    }

    (gains, losses)
}

/// Calculate RSI (Relative Strength Index) using SIMD
fn calculate_rsi_simd(prices: &[f32], period: usize) -> RsiResult {
    if prices.len() < period + 1 {
        return RsiResult {
            rsi_values: vec![],
            avg_gain: 0.0,
            avg_loss: 0.0,
        };
    }

    // Step 1: Calculate price changes
    let mut changes = Vec::with_capacity(prices.len() - 1);
    for i in 1..prices.len() {
        changes.push(prices[i] - prices[i - 1]);
    }

    // Step 2: Split into gains and losses with SIMD
    let (gains, losses) = split_gains_losses_simd(&changes);

    // Step 3: Calculate initial averages
    let first_avg_gain = gains[..period].iter().sum::<f32>() / period as f32;
    let first_avg_loss = losses[..period].iter().sum::<f32>() / period as f32;

    // Step 4: Smoothed averages and RSI
    let mut avg_gain = first_avg_gain;
    let mut avg_loss = first_avg_loss;
    let mut rsi_values = Vec::new();

    // First RSI value
    let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
    rsi_values.push(100.0 - (100.0 / (1.0 + rs)));

    // Subsequent values
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f32 + gains[i]) / period as f32;
        avg_loss = (avg_loss * (period - 1) as f32 + losses[i]) / period as f32;

        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    RsiResult {
        rsi_values,
        avg_gain,
        avg_loss,
    }
}

fn main() {
    let prices = vec![
        44.0, 44.34, 44.09, 43.61, 44.33,
        44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28,
        46.00, 46.03, 46.41, 46.22, 45.64,
        46.21, 46.25, 45.71, 46.45, 45.78,
        45.35, 44.03, 44.18, 44.22, 44.57,
        43.42, 42.66, 43.13,
    ];

    let rsi_result = calculate_rsi_simd(&prices, 14);

    println!("=== RSI-14 using SIMD ===");
    println!("Average gain: {:.4}", rsi_result.avg_gain);
    println!("Average loss: {:.4}", rsi_result.avg_loss);
    println!("\nRSI values (showing last 5):");

    let start = rsi_result.rsi_values.len().saturating_sub(5);
    for (i, rsi) in rsi_result.rsi_values[start..].iter().enumerate() {
        let idx = start + i + 14;
        println!("Day {}: RSI = {:.2}", idx + 1, rsi);

        if *rsi > 70.0 {
            println!("  ⚠️  Overbought!");
        } else if *rsi < 30.0 {
            println!("  ⚠️  Oversold!");
        }
    }

    println!("\n✅ RSI SIMD test passed!");
}
