# Day 315: Compiling in Release Mode

## Trading Analogy

Imagine you've developed a trading bot that analyzes thousands of candles in real-time. In development mode (debug mode), your bot runs slowly — it's like trading on a simulator with delays: convenient for debugging, but unsuitable for real trading.

**Release mode** is the transition from a simulator to a real exchange:

| Debug mode (development) | Release mode (production) |
|--------------------------|---------------------------|
| Many checks and debug info | Maximum speed |
| Slow execution | Optimized code |
| Large binary size | Compact binary |
| Detailed error information | Minimal overhead |

This is like the difference between test-running a strategy on historical data (where speed isn't critical) and real trading, where every millisecond can cost money.

## What is Release Mode?

**Release mode** is a compilation mode in Rust that enables compiler optimizations for maximum performance.

### Comparing Compilation Modes

```bash
# Debug mode (default)
cargo build

# Release mode (with optimizations)
cargo build --release
```

### Key Differences

| Aspect | Debug | Release |
|--------|-------|---------|
| **Optimization** | Minimal (opt-level = 0) | Maximum (opt-level = 3) |
| **Compile time** | Faster | Slower |
| **Execution speed** | Slow | Up to 10-100x faster |
| **Binary size** | Larger | Smaller |
| **Debug information** | Full | None |
| **Bounds checks** | Full | May be optimized out |
| **Panic messages** | Detailed | Minimal |

## Practical Example: SMA Calculation

Let's measure the performance difference on a real task — calculating a moving average for a large volume of data:

```rust
use std::time::Instant;

/// Calculate Simple Moving Average
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    // First SMA value
    let first_sum: f64 = prices[..period].iter().sum();
    sma_values.push(first_sum / period as f64);

    // Optimized rolling calculation
    for i in period..prices.len() {
        let prev_sma = sma_values.last().unwrap();
        let new_sma = prev_sma + (prices[i] - prices[i - period]) / period as f64;
        sma_values.push(new_sma);
    }

    sma_values
}

/// Generate test data (simulating BTC prices)
fn generate_price_data(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 42000.0;

    for i in 0..count {
        // Simulate random price movement
        let change = ((i * 17 + 13) % 200) as f64 / 100.0 - 1.0;
        price += change * 50.0;
        prices.push(price);
    }

    prices
}

fn main() {
    let data_sizes = [1_000, 10_000, 100_000, 1_000_000];
    let period = 20;

    println!("=== SMA(20) Calculation Benchmark ===\n");
    println!("Compilation mode: {}",
             if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });
    println!();

    for &size in &data_sizes {
        let prices = generate_price_data(size);

        let start = Instant::now();
        let sma = calculate_sma(&prices, period);
        let elapsed = start.elapsed();

        println!("Data: {:>10} points", size);
        println!("  Time: {:>10.3} ms", elapsed.as_secs_f64() * 1000.0);
        println!("  SMA values: {}", sma.len());
        println!("  Last SMA: ${:.2}", sma.last().unwrap_or(&0.0));
        println!();
    }
}
```

### Comparison Results

Run this code in both modes:

```bash
# Debug mode
cargo run

# Release mode
cargo run --release
```

Typical results:

| Data | Debug | Release | Speedup |
|------|-------|---------|---------|
| 1,000 | 0.5 ms | 0.02 ms | 25x |
| 10,000 | 5 ms | 0.15 ms | 33x |
| 100,000 | 50 ms | 1.2 ms | 42x |
| 1,000,000 | 500 ms | 10 ms | 50x |

## Configuring Profiles in Cargo.toml

Rust allows fine-tuning compilation parameters through `Cargo.toml`:

```toml
[package]
name = "trading-bot"
version = "1.0.0"

# Debug build settings
[profile.dev]
opt-level = 0          # No optimizations (fast compilation)
debug = true           # Full debug information
overflow-checks = true # Overflow checks

# Release build settings
[profile.release]
opt-level = 3          # Maximum optimizations
lto = true             # Link-Time Optimization
codegen-units = 1      # Single codegen for better optimization
panic = "abort"        # Abort instead of unwind (smaller binary)
strip = true           # Remove debug symbols

# Custom profile for benchmarks
[profile.bench]
opt-level = 3
lto = true
debug = false

# Profile for testing with optimizations
[profile.test-release]
inherits = "release"
debug = true           # Debug info for profiling
```

### Optimization Levels

| opt-level | Description | Use Case |
|-----------|-------------|----------|
| 0 | No optimizations | Fast development |
| 1 | Basic optimizations | Speed/size balance |
| 2 | Most optimizations | Good compromise |
| 3 | All optimizations | Maximum speed |
| "s" | Size optimization | Embedded systems |
| "z" | Minimum size | Size-critical |

## Link-Time Optimization (LTO)

LTO is a powerful optimization technique that analyzes all project code at link time:

```toml
[profile.release]
lto = true          # Full LTO (slowest compilation, best results)
# lto = "thin"      # Thin LTO (faster compilation, slightly worse results)
# lto = false       # No LTO (default)
```

### Example: Optimizing Trading Calculations

```rust
/// Calculate Maximum Drawdown
/// LTO allows inlining this function into calling code
#[inline(always)]
fn calculate_running_max(prices: &[f64]) -> Vec<f64> {
    let mut running_max = Vec::with_capacity(prices.len());
    let mut max = f64::MIN;

    for &price in prices {
        if price > max {
            max = price;
        }
        running_max.push(max);
    }

    running_max
}

/// Calculate drawdown from peak
#[inline(always)]
fn calculate_drawdown(prices: &[f64], running_max: &[f64]) -> Vec<f64> {
    prices.iter()
        .zip(running_max.iter())
        .map(|(price, max)| (price - max) / max * 100.0)
        .collect()
}

/// Maximum Drawdown of portfolio
pub fn max_drawdown(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    let running_max = calculate_running_max(prices);
    let drawdowns = calculate_drawdown(prices, &running_max);

    drawdowns.iter()
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
}

fn main() {
    // Simulating equity curve of a trading strategy
    let equity: Vec<f64> = vec![
        10000.0, 10200.0, 10500.0, 10300.0, 10100.0,
        9800.0, 9500.0, 9700.0, 10000.0, 10500.0,
        11000.0, 10800.0, 10600.0, 11200.0, 11500.0,
    ];

    let mdd = max_drawdown(&equity);
    println!("Maximum Drawdown: {:.2}%", mdd);
}
```

## Conditional Compilation: Debug vs Release

Rust allows writing different code for debug and release modes:

```rust
/// Structure for logging trades
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: TradeSide,
}

#[derive(Debug)]
enum TradeSide {
    Buy,
    Sell,
}

impl Trade {
    fn execute(&self) {
        // Detailed logging only in debug mode
        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Executing trade: {} {} {} @ ${:.2}",
            match self.side { TradeSide::Buy => "BUY", TradeSide::Sell => "SELL" },
            self.quantity,
            self.symbol,
            self.price
        );

        // Real execution logic
        self.send_to_exchange();

        // Verification only in debug
        #[cfg(debug_assertions)]
        self.verify_execution();
    }

    fn send_to_exchange(&self) {
        // Send to exchange
        println!("Order sent: {} {}", self.symbol, self.quantity);
    }

    #[cfg(debug_assertions)]
    fn verify_execution(&self) {
        println!("[DEBUG] Verifying execution...");
        // Additional checks for development
    }
}

/// Function available only in debug mode
#[cfg(debug_assertions)]
fn debug_assert_valid_price(price: f64) {
    assert!(price > 0.0, "Price must be positive!");
    assert!(price < 1_000_000.0, "Price too high!");
}

/// Calculate position with additional checks in debug
fn calculate_position_size(capital: f64, price: f64, risk_percent: f64) -> f64 {
    #[cfg(debug_assertions)]
    debug_assert_valid_price(price);

    let position_value = capital * risk_percent / 100.0;
    position_value / price
}

fn main() {
    println!("Mode: {}",
             if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });

    let trade = Trade {
        symbol: "BTC/USD".to_string(),
        price: 42500.0,
        quantity: 0.5,
        side: TradeSide::Buy,
    };

    trade.execute();

    let position = calculate_position_size(10000.0, 42500.0, 2.0);
    println!("Position size: {} BTC", position);
}
```

## Platform-Specific Optimization

For maximum performance, you can specify the target processor:

```bash
# Use all features of current CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Specify a specific architecture
RUSTFLAGS="-C target-cpu=skylake" cargo build --release
```

### Cargo.toml for Cross-Compilation

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-cpu=native"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=native"]
```

## Performance Measurement: Benchmarks

For accurate performance measurement, use built-in benchmarks:

```rust
#![feature(test)]

extern crate test;

use test::Bencher;

/// Naive SMA implementation
fn sma_naive(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..=prices.len() - period {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

/// Optimized SMA implementation
fn sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();
    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    fn generate_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 42000.0 + (i as f64) * 0.1).collect()
    }

    #[bench]
    fn bench_sma_naive_1000(b: &mut Bencher) {
        let prices = generate_prices(1000);
        b.iter(|| sma_naive(&prices, 20));
    }

    #[bench]
    fn bench_sma_optimized_1000(b: &mut Bencher) {
        let prices = generate_prices(1000);
        b.iter(|| sma_optimized(&prices, 20));
    }

    #[bench]
    fn bench_sma_naive_10000(b: &mut Bencher) {
        let prices = generate_prices(10000);
        b.iter(|| sma_naive(&prices, 20));
    }

    #[bench]
    fn bench_sma_optimized_10000(b: &mut Bencher) {
        let prices = generate_prices(10000);
        b.iter(|| sma_optimized(&prices, 20));
    }
}

fn main() {
    println!("Run benchmarks with: cargo bench");
}
```

Running benchmarks:

```bash
cargo bench
```

## Reducing Binary Size

For production deployment, binary size matters:

```toml
[profile.release]
opt-level = "z"        # Size optimization
lto = true             # LTO also reduces size
codegen-units = 1      # Better optimization
panic = "abort"        # Remove unwind code
strip = true           # Remove debug symbols
```

### Additional Steps

```bash
# Build with minimum size
cargo build --release

# Check size
ls -lh target/release/trading-bot

# Additional compression (Linux)
strip target/release/trading-bot
upx --best target/release/trading-bot
```

## Debugging in Release Mode

Sometimes you need to debug an issue that only appears in release:

```toml
# Profile for debugging release
[profile.release-with-debug]
inherits = "release"
debug = true           # Add debug information
```

```bash
# Build with this profile
cargo build --profile release-with-debug
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Debug mode** | Mode for development with checks and debug info |
| **Release mode** | Optimized mode for production |
| **opt-level** | Optimization level (0-3, s, z) |
| **LTO** | Link-Time Optimization for cross-module optimization |
| **codegen-units** | Number of parallel codegen units |
| **panic = "abort"** | Simplified panic handling for smaller binary |
| **cfg!(debug_assertions)** | Conditional compilation for different modes |
| **target-cpu=native** | Optimization for specific processor |

## Practical Exercises

1. **Strategy Benchmark**: Create a trading strategy (e.g., SMA crossover) and measure the performance difference between debug and release modes on 1 million candles.

2. **Testing Profile**: Create a custom profile in `Cargo.toml` for testing with partial optimizations (opt-level = 1) and debug information.

3. **Conditional Compilation**: Add detailed logging of all operations to a trading bot that only works in debug mode, and measure the performance difference.

4. **Size Optimization**: Build a trading bot with minimum binary size and compare it with a regular release build. Measure the impact on performance.

## Homework

1. **Trading Bot Benchmark Suite**: Create a benchmark suite for main trading bot operations:
   - Indicator calculations (SMA, EMA, RSI, MACD)
   - Market data parsing
   - Order serialization/deserialization
   - Portfolio risk calculations

   Compare debug vs release results.

2. **Profiles for Different Scenarios**: Configure different compilation profiles for:
   - Development (fast compilation)
   - Testing (optimizations + debug)
   - Staging (near-production)
   - Production (maximum performance)

   Document compilation time and binary size for each.

3. **Cross-Platform Optimization**: Configure cross-compilation of a trading bot for:
   - Linux (server)
   - macOS (development)
   - Windows (clients)

   With optimizations for specific processors.

4. **Performance Regression Testing**: Create a system for automatic performance testing:
   - Benchmarks for critical paths
   - Comparison with baseline metrics
   - Automatic warnings on performance degradation

   Integrate into CI/CD pipeline.

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../316-*/en.md)
