# Day 312: SIMD: Parallel Computations

## Trading Analogy

Imagine you need to process an order book with thousands of orders — calculate the volume-weighted average price for each of 1000 trading instruments. The usual approach: take an instrument, go through all its orders, calculate the average price, move to the next. It's like a bank teller serving customers strictly one by one.

**SIMD (Single Instruction, Multiple Data)** is like having 4 tellers at the bank who simultaneously perform the same operation on different customers: all four check documents at once, all count money simultaneously, all issue receipts together.

In trading context:
- Regular processor: processes prices one at a time
- SIMD processor: processes 4-8-16 prices simultaneously in one instruction
- Speedup: 4-8x faster when computing indicators, processing ticks, calculating volatility

Typical example: calculating moving average over 1000 prices:
- Without SIMD: 1000 addition operations
- With SIMD (AVX-256): ~125 operations (8 prices at once)
- Result: 8x speedup!

## What is SIMD?

SIMD is a processor technology that allows executing one instruction on multiple data elements simultaneously:

| Feature | Description |
|---------|-------------|
| **SSE** | 128-bit registers, 4 × f32 or 2 × f64 |
| **AVX** | 256-bit registers, 8 × f32 or 4 × f64 |
| **AVX-512** | 512-bit registers, 16 × f32 or 8 × f64 |

### When is SIMD Effective?

✅ **Good for:**
- Processing arrays of prices, volumes, indicators
- Mathematical operations: addition, multiplication, min/max
- Identical operations over large amounts of data
- Computing technical indicators (SMA, EMA, RSI)

❌ **Not suitable for:**
- Different operations for each element
- Complex logic with branches
- Small data volumes (overhead exceeds benefits)

## Basic Example: PnL Calculation

```rust
// Without SIMD: processing one trade at a time
fn calculate_pnl_scalar(entry_prices: &[f32], exit_prices: &[f32]) -> Vec<f32> {
    entry_prices.iter()
        .zip(exit_prices.iter())
        .map(|(entry, exit)| exit - entry)
        .collect()
}

// With SIMD: processing 4 trades simultaneously (auto-vectorized through iterators)
fn calculate_pnl_simd(entry_prices: &[f32], exit_prices: &[f32]) -> Vec<f32> {
    // Rust can automatically vectorize simple operations
    entry_prices.iter()
        .zip(exit_prices.iter())
        .map(|(entry, exit)| exit - entry)
        .collect()
}

fn main() {
    let entries = vec![100.0, 101.5, 99.8, 102.3];
    let exits = vec![105.0, 100.0, 103.2, 101.5];

    let pnl = calculate_pnl_scalar(&entries, &exits);

    println!("=== Profit/Loss per Trade ===");
    for (i, profit) in pnl.iter().enumerate() {
        println!("Trade {}: {:.2}", i + 1, profit);
    }
}
```

## Explicit SIMD Usage with `std::simd`

Rust provides a portable interface for SIMD through the experimental `std::simd` module:

```rust
#![feature(portable_simd)]
use std::simd::f32x4;

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
}
```

## Practical Example: Volatility Calculation

```rust
#![feature(portable_simd)]
use std::simd::f32x8;

#[derive(Debug)]
struct VolatilityMetrics {
    std_dev: f32,
    variance: f32,
    mean: f32,
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

fn calculate_mean_simd(values: &[f32]) -> f32 {
    let sum = calculate_sum_simd_f32x8(values);
    sum / values.len() as f32
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
}
```

## Computing RSI Indicator with SIMD

```rust
#![feature(portable_simd)]
use std::simd::f32x4;

#[derive(Debug)]
struct RsiResult {
    rsi_values: Vec<f32>,
    avg_gain: f32,
    avg_loss: f32,
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
    println!("\nRSI values:");

    for (i, rsi) in rsi_result.rsi_values.iter().enumerate() {
        let idx = i + 14;
        println!("Day {}: RSI = {:.2}", idx + 1, rsi);

        if *rsi > 70.0 {
            println!("  ⚠️  Overbought!");
        } else if *rsi < 30.0 {
            println!("  ⚠️  Oversold!");
        }
    }
}
```

## Performance Comparison: SIMD vs Scalar

```rust
use std::time::Instant;

/// Calculate moving average without SIMD
fn sma_scalar(prices: &[f32], window: usize) -> Vec<f32> {
    let mut result = Vec::new();

    for i in 0..=prices.len() - window {
        let sum: f32 = prices[i..i + window].iter().sum();
        result.push(sum / window as f32);
    }

    result
}

/// Generate test data
fn generate_prices(count: usize, start: f32) -> Vec<f32> {
    let mut prices = Vec::with_capacity(count);
    let mut price = start;

    for i in 0..count {
        price += ((i % 17) as f32 - 8.0) * 0.1;
        prices.push(price);
    }

    prices
}

fn main() {
    let sizes = [1000, 10_000, 100_000, 1_000_000];

    println!("=== Performance Comparison: SIMD vs Scalar ===\n");

    for &size in &sizes {
        let prices = generate_prices(size, 100.0);
        let window = 20;

        // Scalar version
        let start = Instant::now();
        let result_scalar = sma_scalar(&prices, window);
        let time_scalar = start.elapsed();

        // SIMD version (using calculate_sma_simd from previous examples)
        let start = Instant::now();
        let result_simd = calculate_sma_simd(&prices, window);
        let time_simd = start.elapsed();

        let speedup = time_scalar.as_secs_f64() / time_simd.as_secs_f64();

        println!("Data size: {} prices", size);
        println!("  Scalar: {:?}", time_scalar);
        println!("  SIMD:   {:?}", time_simd);
        println!("  Speedup: {:.2}x", speedup);
        println!();

        // Verify correctness (first few values)
        assert_eq!(result_scalar.len(), result_simd.len());
        for i in 0..5.min(result_scalar.len()) {
            let diff = (result_scalar[i] - result_simd[i]).abs();
            assert!(diff < 0.001, "Values differ!");
        }
    }
}
```

## SIMD Limitations and Pitfalls

### 1. Data Alignment

```rust
#![feature(portable_simd)]
use std::simd::f32x8;

fn demonstrate_alignment() {
    // Good: data is aligned
    let aligned = vec![1.0f32; 32];

    // Bad: might not be aligned
    let unaligned = &aligned[1..25]; // doesn't start at boundary

    // SIMD requires alignment for optimal performance
    println!("Aligned length: {}", aligned.len());
    println!("Unaligned length: {}", unaligned.len());
}
```

### 2. Remainder Elements

```rust
fn handle_remainder_correctly(data: &[f32]) -> f32 {
    let mut sum = 0.0f32;

    // Process in blocks
    let chunks = data.chunks_exact(8);
    let remainder = chunks.remainder();

    // SIMD processing
    for chunk in chunks {
        sum += chunk.iter().sum::<f32>();
    }

    // Important: don't forget to process the remainder!
    sum += remainder.iter().sum::<f32>();

    sum
}
```

### 3. Branches in SIMD

```rust
#![feature(portable_simd)]
use std::simd::{f32x4, Mask};

/// Example: calculate fees with conditions
fn calculate_fees_simd(volumes: &[f32], threshold: f32) -> Vec<f32> {
    let mut fees = vec![0.0f32; volumes.len()];

    let chunks = volumes.chunks_exact(4);
    let remainder = chunks.remainder();

    let threshold_simd = f32x4::splat(threshold);
    let high_fee = f32x4::splat(0.001); // 0.1%
    let low_fee = f32x4::splat(0.002);  // 0.2%

    for (i, chunk) in chunks.enumerate() {
        let vols = f32x4::from_slice(chunk);

        // Mask: volume >= threshold?
        let is_high_volume = vols.simd_ge(threshold_simd);

        // Select fee based on mask
        let fee_rate = is_high_volume.select(high_fee, low_fee);
        let fee_values = vols * fee_rate;

        let idx = i * 4;
        fee_values.copy_to_slice(&mut fees[idx..idx + 4]);
    }

    // Process remainder
    for (i, &vol) in remainder.iter().enumerate() {
        let idx = volumes.len() - remainder.len() + i;
        let rate = if vol >= threshold { 0.001 } else { 0.002 };
        fees[idx] = vol * rate;
    }

    fees
}

fn main() {
    let volumes = vec![
        10000.0, 5000.0, 50000.0, 3000.0,
        75000.0, 8000.0, 100000.0, 2000.0,
    ];

    let fees = calculate_fees_simd(&volumes, 10000.0);

    println!("=== Fee Calculation with SIMD ===");
    for (i, (vol, fee)) in volumes.iter().zip(fees.iter()).enumerate() {
        let rate = if *vol >= 10000.0 { "0.1%" } else { "0.2%" };
        println!("Trade {}: volume = {:.0}, fee = {:.2} ({})",
                 i + 1, vol, fee, rate);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **SIMD** | Single Instruction, Multiple Data — one instruction, many data |
| **SSE/AVX** | SIMD instruction sets in Intel/AMD processors |
| **f32x4, f32x8** | SIMD vectors for 4 and 8 floating-point numbers |
| **Vectorization** | Converting scalar code to SIMD |
| **Chunks** | Splitting data into blocks for SIMD processing |
| **Masks** | Conditional operations in SIMD through bit masks |
| **Alignment** | Placing data at memory boundaries for SIMD |
| **Remainder** | Processing residual elements not divisible by SIMD size |

## Practical Exercises

1. **Technical Indicators Calculator**: Implement with SIMD:
   - Bollinger Bands (moving average + standard deviation)
   - MACD (difference of two exponential moving averages)
   - ATR (Average True Range)

   Compare performance of SIMD and scalar versions.

2. **Arbitrage Opportunity Finder**: Write a function with SIMD:
   - Takes arrays of bid/ask prices from different exchanges
   - Finds moments when bid(exchange A) > ask(exchange B)
   - Calculates potential profit including fees
   - Uses SIMD masks for conditions

3. **Portfolio Optimization**: Create SIMD version:
   - Calculate correlation matrix between assets
   - Compute portfolio weights
   - Monte Carlo simulation for risk assessment

   Measure speedup for portfolio of 50-100 assets.

## Homework

1. **SIMD Indicators Library**: Create a module with:
   - At least 5 technical indicators
   - Each indicator in two versions: scalar and SIMD
   - Benchmarks for performance comparison
   - Documentation with usage examples

2. **Candlestick Pattern Analyzer**: Implement with SIMD:
   - Search for patterns: doji, hammer, engulfing, etc.
   - Process OHLC data in parallel
   - Vector comparisons for pattern detection
   - Statistics on found patterns

3. **High-Frequency Data Processor**: Build a system:
   - Process tick data in real-time
   - Aggregate into OHLC candles with SIMD
   - Calculate volume profiles
   - Detect anomalies in prices/volumes
   - Output performance metrics (ticks/sec)

4. **Comparative Analysis**: Test:
   - Different SIMD vector sizes (f32x4, f32x8, f32x16)
   - Impact of data alignment
   - Overhead with small data volumes
   - Scaling on large datasets (millions of records)

   Plot performance graphs vs data size.

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
