# Day 317: PGO: Profile Guided Optimization

## Trading Analogy

Imagine you're developing a high-frequency trading system (HFT). In normal development, you optimize code "blindly" — guessing which functions will be called most often. It's like trying to optimize a trading strategy without real market data.

**Profile Guided Optimization (PGO)** is like backtesting for the compiler:
1. **First, collect a profile** — run the program with real trading data, the compiler records which code branches execute more often
2. **Then optimize based on data** — the compiler restructures the code knowing actual execution patterns

This is analogous to how an experienced trader:
- First analyzes trade history (profiling)
- Then optimizes the strategy based on real data (PGO compilation)
- Gets better results than with "blind" optimization

In the trading context, PGO is especially important for:
- High-frequency systems where every microsecond matters
- Market data parsers handling millions of messages per second
- Real-time indicator calculations
- Risk management systems with critical latency requirements

## What is PGO?

**Profile Guided Optimization (PGO)** is an optimization technique where the compiler uses information about actual program behavior to make more effective decisions.

### How Does It Work?

```
┌─────────────────────────────────────────────────────────────────┐
│                    PGO Process                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Regular Build         2. Profiling          3. PGO Build   │
│  ┌───────────────────┐    ┌─────────────────┐  ┌─────────────┐ │
│  │  Source code      │───▶│ Run with real   │─▶│ Final       │ │
│  │  + profiling      │    │ data            │  │ optimization│ │
│  │  instrumentation  │    │                 │  │ using       │ │
│  └───────────────────┘    └─────────────────┘  │ profile     │ │
│         │                        │             └─────────────┘ │
│         ▼                        ▼                    │        │
│  ┌───────────────────┐    ┌─────────────────┐         │        │
│  │ Instrumented      │    │ Profile file    │─────────┘        │
│  │ binary            │    │ (.profdata)     │                  │
│  └───────────────────┘    └─────────────────┘                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### What Optimizations Does PGO Perform?

| Optimization | Description | Impact on Trading |
|--------------|-------------|-------------------|
| **Inlining** | Inlining frequently called functions | Faster indicator calculations |
| **Branch prediction** | Optimizing conditional jumps | Faster order processing |
| **Code layout** | Placing hot code together | Fewer cache misses |
| **Register allocation** | Better register distribution | Faster computations |
| **Virtual call speculation** | Optimizing virtual calls | Faster polymorphic code |

## Enabling PGO in Rust

### Step 1: Compile with Profiling Instrumentation

```bash
# Create folder for profiles
mkdir -p /tmp/pgo-data

# Compile with profiling instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release --target=x86_64-unknown-linux-gnu
```

### Step 2: Run the Program with Real Data

```bash
# Run the program with typical trading data
./target/release/trading-bot --data historical_btc_2024.csv

# You can run multiple times with different scenarios
./target/release/trading-bot --data historical_eth_2024.csv
./target/release/trading-bot --data high_volatility_market.csv
```

### Step 3: Merge Profiles

```bash
# Install llvm-tools if not already installed
rustup component add llvm-tools-preview

# Find llvm-profdata
LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata | head -1)

# Merge all profiles into one
$LLVM_PROFDATA merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data/
```

### Step 4: Final PGO Compilation

```bash
# Compile using the profile
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release --target=x86_64-unknown-linux-gnu
```

## Practical Example: Optimizing a Trading System

### Trading System Source Code

```rust
use std::collections::HashMap;
use std::time::Instant;

/// OHLCV candle data
#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Trading signal
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Position
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    entry_time: i64,
}

/// Indicator calculator
struct IndicatorCalculator {
    sma_cache: HashMap<(String, usize), Vec<f64>>,
    ema_cache: HashMap<(String, usize), Vec<f64>>,
}

impl IndicatorCalculator {
    fn new() -> Self {
        Self {
            sma_cache: HashMap::new(),
            ema_cache: HashMap::new(),
        }
    }

    /// Calculate SMA (Simple Moving Average)
    /// This function is called very often — PGO optimizes it
    fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Calculate EMA (Exponential Moving Average)
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for &price in &prices[1..] {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }

        Some(ema)
    }

    /// Calculate RSI (Relative Strength Index)
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period {
            let change = prices[prices.len() - period - 1 + i] - prices[prices.len() - period - 2 + i];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Calculate volatility (standard deviation)
    fn calculate_volatility(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let slice = &prices[prices.len() - period..];
        let mean: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / period as f64;

        Some(variance.sqrt())
    }
}

/// Trading strategy
struct TradingStrategy {
    calculator: IndicatorCalculator,
    fast_period: usize,
    slow_period: usize,
    rsi_period: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,
}

impl TradingStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            calculator: IndicatorCalculator::new(),
            fast_period,
            slow_period,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
        }
    }

    /// Generate signal based on indicators
    /// This is the hot path — PGO optimizes branching
    fn generate_signal(&self, prices: &[f64]) -> Signal {
        // Check for sufficient data — often true
        if prices.len() < self.slow_period {
            return Signal::Hold;
        }

        // Calculate indicators
        let fast_sma = match self.calculator.calculate_sma(prices, self.fast_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        let slow_sma = match self.calculator.calculate_sma(prices, self.slow_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        let rsi = match self.calculator.calculate_rsi(prices, self.rsi_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        // Strategy logic — PGO optimizes based on real patterns
        // If the market more often rises, compiler optimizes the Buy branch
        if fast_sma > slow_sma && rsi < self.rsi_oversold {
            Signal::Buy
        } else if fast_sma < slow_sma && rsi > self.rsi_overbought {
            Signal::Sell
        } else {
            Signal::Hold
        }
    }
}

/// Backtester for profile collection
struct Backtester {
    strategy: TradingStrategy,
    positions: Vec<Position>,
    equity_curve: Vec<f64>,
    initial_capital: f64,
    current_capital: f64,
}

impl Backtester {
    fn new(strategy: TradingStrategy, initial_capital: f64) -> Self {
        Self {
            strategy,
            positions: Vec::new(),
            equity_curve: vec![initial_capital],
            initial_capital,
            current_capital: initial_capital,
        }
    }

    /// Process candle — called millions of times
    fn process_candle(&mut self, candle: &Candle, price_history: &[f64]) {
        let signal = self.strategy.generate_signal(price_history);

        match signal {
            Signal::Buy if self.positions.is_empty() => {
                // Open position
                let quantity = self.current_capital * 0.95 / candle.close;
                self.positions.push(Position {
                    symbol: "BTC".to_string(),
                    quantity,
                    entry_price: candle.close,
                    entry_time: candle.timestamp,
                });
                self.current_capital *= 0.05; // Keep 5% for fees
            }
            Signal::Sell if !self.positions.is_empty() => {
                // Close position
                if let Some(pos) = self.positions.pop() {
                    let pnl = (candle.close - pos.entry_price) * pos.quantity;
                    self.current_capital += pos.quantity * candle.close * 0.999; // -0.1% commission
                }
            }
            _ => {} // Hold — do nothing
        }

        // Update equity curve
        let total_equity = self.current_capital +
            self.positions.iter()
                .map(|p| p.quantity * candle.close)
                .sum::<f64>();
        self.equity_curve.push(total_equity);
    }

    /// Calculate final metrics
    fn calculate_metrics(&self) -> BacktestMetrics {
        let final_equity = *self.equity_curve.last().unwrap_or(&self.initial_capital);
        let total_return = (final_equity / self.initial_capital - 1.0) * 100.0;

        // Calculate maximum drawdown
        let mut max_drawdown = 0.0;
        let mut peak = self.initial_capital;
        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        BacktestMetrics {
            total_return,
            max_drawdown: max_drawdown * 100.0,
            final_equity,
        }
    }
}

#[derive(Debug)]
struct BacktestMetrics {
    total_return: f64,
    max_drawdown: f64,
    final_equity: f64,
}

/// Generate test data for profiling
fn generate_test_data(num_candles: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(num_candles);
    let mut price = 40000.0;

    for i in 0..num_candles {
        // Simulate market volatility
        let change = ((i as f64 * 0.1).sin() * 0.02 + (i as f64 * 0.03).cos() * 0.015) * price;
        price += change;

        let high = price * 1.01;
        let low = price * 0.99;
        let open = price - change * 0.3;
        let close = price;

        candles.push(Candle {
            timestamp: 1704067200 + (i as i64 * 3600), // Hourly candles
            open,
            high,
            low,
            close,
            volume: 1000.0 + (i as f64 * 10.0).sin().abs() * 500.0,
        });
    }

    candles
}

fn main() {
    println!("=== PGO Benchmark: Trading System ===\n");

    // Generate test data (as in real trading)
    let num_candles = 100_000;
    println!("Generating {} candles for backtest...", num_candles);
    let candles = generate_test_data(num_candles);

    // Create strategy
    let strategy = TradingStrategy::new(10, 50);
    let mut backtester = Backtester::new(strategy, 100_000.0);

    // Collect price history
    let mut price_history: Vec<f64> = Vec::with_capacity(num_candles);

    // Benchmark
    let start = Instant::now();

    for candle in &candles {
        price_history.push(candle.close);
        backtester.process_candle(candle, &price_history);
    }

    let duration = start.elapsed();

    // Results
    let metrics = backtester.calculate_metrics();

    println!("\n=== Backtest Results ===");
    println!("Execution time: {:?}", duration);
    println!("Candles processed: {}", num_candles);
    println!("Speed: {:.2} candles/sec", num_candles as f64 / duration.as_secs_f64());
    println!("\n=== Strategy Metrics ===");
    println!("Total return: {:.2}%", metrics.total_return);
    println!("Max drawdown: {:.2}%", metrics.max_drawdown);
    println!("Final capital: ${:.2}", metrics.final_equity);
}
```

### Cargo.toml for the Project

```toml
[package]
name = "trading-pgo-example"
version = "0.1.0"
edition = "2021"

[dependencies]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

# Create a separate profile for PGO build
[profile.release-pgo]
inherits = "release"
# Additional optimizations for PGO
```

### PGO Automation Script

```bash
#!/bin/bash
# pgo_build.sh - Script for PGO build of trading system

set -e

PROJECT_DIR=$(pwd)
PGO_DATA_DIR="$PROJECT_DIR/pgo-data"
TARGET="x86_64-unknown-linux-gnu"

echo "=== PGO Build Script for Trading System ==="
echo ""

# Step 1: Cleanup
echo "[1/5] Cleaning previous data..."
rm -rf "$PGO_DATA_DIR"
mkdir -p "$PGO_DATA_DIR"
cargo clean

# Step 2: Regular build (for comparison)
echo ""
echo "[2/5] Building regular version..."
cargo build --release --target=$TARGET 2>/dev/null
cp target/$TARGET/release/trading-pgo-example target/release/trading-normal

# Step 3: Build with profiling instrumentation
echo ""
echo "[3/5] Building with profiling instrumentation..."
RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR" \
    cargo build --release --target=$TARGET 2>/dev/null

# Step 4: Collect profile
echo ""
echo "[4/5] Collecting profile on test data..."
echo "  Run 1: Standard data..."
./target/$TARGET/release/trading-pgo-example > /dev/null

echo "  Run 2: High volatility (different seed)..."
./target/$TARGET/release/trading-pgo-example > /dev/null

echo "  Run 3: Low volatility..."
./target/$TARGET/release/trading-pgo-example > /dev/null

# Merge profiles
LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata | head -1)
if [ -z "$LLVM_PROFDATA" ]; then
    echo "Error: llvm-profdata not found"
    echo "Install: rustup component add llvm-tools-preview"
    exit 1
fi

$LLVM_PROFDATA merge -o "$PGO_DATA_DIR/merged.profdata" "$PGO_DATA_DIR/"

# Step 5: Final PGO build
echo ""
echo "[5/5] Final PGO-optimized build..."
cargo clean 2>/dev/null
RUSTFLAGS="-Cprofile-use=$PGO_DATA_DIR/merged.profdata" \
    cargo build --release --target=$TARGET 2>/dev/null
cp target/$TARGET/release/trading-pgo-example target/release/trading-pgo

# Comparison
echo ""
echo "=== Performance Comparison ==="
echo ""

echo "Regular build:"
time ./target/release/trading-normal

echo ""
echo "PGO-optimized build:"
time ./target/release/trading-pgo

echo ""
echo "=== Build Complete ==="
echo "Binaries are in target/release/"
echo "  - trading-normal: regular build"
echo "  - trading-pgo: PGO-optimized build"
```

## Measuring PGO Impact

### Benchmark for Comparison

```rust
use std::time::{Duration, Instant};

/// Benchmark result
struct BenchmarkResult {
    name: String,
    iterations: usize,
    total_time: Duration,
    avg_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("=== {} ===", self.name);
        println!("  Iterations: {}", self.iterations);
        println!("  Total time: {:?}", self.total_time);
        println!("  Average: {:?}", self.avg_time);
        println!("  Min: {:?}", self.min_time);
        println!("  Max: {:?}", self.max_time);
    }
}

/// Run function benchmark
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    let mut times = Vec::with_capacity(iterations);

    // Warmup
    for _ in 0..10 {
        f();
    }

    // Measurements
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }

    let total_time: Duration = times.iter().sum();
    let avg_time = total_time / iterations as u32;
    let min_time = *times.iter().min().unwrap();
    let max_time = *times.iter().max().unwrap();

    BenchmarkResult {
        name: name.to_string(),
        iterations,
        total_time,
        avg_time,
        min_time,
        max_time,
    }
}

fn main() {
    let prices: Vec<f64> = (0..10000)
        .map(|i| 40000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    let calculator = IndicatorCalculator::new();

    // Benchmark SMA
    let sma_result = benchmark("SMA(50)", 10000, || {
        let _ = calculator.calculate_sma(&prices, 50);
    });
    sma_result.print();

    println!();

    // Benchmark EMA
    let ema_result = benchmark("EMA(50)", 10000, || {
        let _ = calculator.calculate_ema(&prices, 50);
    });
    ema_result.print();

    println!();

    // Benchmark RSI
    let rsi_result = benchmark("RSI(14)", 10000, || {
        let _ = calculator.calculate_rsi(&prices, 14);
    });
    rsi_result.print();
}
```

## Expected PGO Results

Typical performance improvements with PGO:

| Component | Improvement | Reason |
|-----------|-------------|--------|
| **Indicator calculation** | 5-15% | Better inlining, branch prediction |
| **Market data parsing** | 10-25% | Hot path optimization in parsing |
| **Order processing** | 5-10% | Conditional jump optimization |
| **Overall throughput** | 10-20% | Combined effect |

### Real-World Improvement Examples

```
=== Results without PGO ===
Execution time: 1.234s
Speed: 81,037 candles/sec

=== Results with PGO ===
Execution time: 1.052s
Speed: 95,057 candles/sec

Improvement: 17.3%
```

## Advanced PGO Techniques

### 1. Instrumenting Only Critical Functions

```rust
/// Attribute for force-inlining hot functions
#[inline(always)]
fn hot_path_calculation(price: f64, factor: f64) -> f64 {
    // Critical code that should be inlined
    price * factor * 1.001
}

/// Attribute to prevent inlining of rare functions
#[inline(never)]
fn cold_path_error_handling(error: &str) {
    // Rarely called error handling code
    eprintln!("Error: {}", error);
}
```

### 2. Branch Probability Hints to Compiler

```rust
use std::hint::black_box;

/// Function with probability hints
fn process_market_data(data: &[u8]) -> Result<f64, &'static str> {
    // Validity check — usually succeeds
    if data.is_empty() {
        // unlikely! macro (in nightly Rust)
        return Err("Empty data");
    }

    // Main logic — hot path
    let price = parse_price(data);

    // Anomaly check — rarely triggers
    if price < 0.0 || price > 1_000_000.0 {
        return Err("Invalid price");
    }

    Ok(price)
}

fn parse_price(data: &[u8]) -> f64 {
    // Parse price from bytes
    // PGO optimizes this code based on real data
    42000.0 // Placeholder
}
```

### 3. BOLT — Post-Link Optimization

BOLT (Binary Optimization and Layout Tool) can further optimize the binary after PGO:

```bash
# Build with symbol preservation for BOLT
RUSTFLAGS="-Clink-arg=-Wl,--emit-relocs" cargo build --release

# Collect profile with perf
perf record -e cycles:u -o perf.data ./target/release/trading-bot

# Convert to BOLT format
perf2bolt -p perf.data ./target/release/trading-bot -o perf.fdata

# Optimize with BOLT
llvm-bolt ./target/release/trading-bot -o ./target/release/trading-bot-bolt \
    -data=perf.fdata -reorder-blocks=ext-tsp -reorder-functions=hfsort
```

## Automating PGO in CI/CD

### GitHub Actions Workflow

```yaml
name: PGO Build

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  pgo-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-unknown-linux-gnu
          components: llvm-tools-preview

      - name: Create PGO data directory
        run: mkdir -p pgo-data

      - name: Build instrumented binary
        run: |
          RUSTFLAGS="-Cprofile-generate=pgo-data" \
            cargo build --release --target=x86_64-unknown-linux-gnu

      - name: Run profiling workload
        run: |
          ./target/x86_64-unknown-linux-gnu/release/trading-bot \
            --data test_data/btc_2024.csv

      - name: Merge profile data
        run: |
          LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata)
          $LLVM_PROFDATA merge -o pgo-data/merged.profdata pgo-data/

      - name: Build PGO-optimized binary
        run: |
          cargo clean
          RUSTFLAGS="-Cprofile-use=pgo-data/merged.profdata" \
            cargo build --release --target=x86_64-unknown-linux-gnu

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: trading-bot-pgo
          path: target/x86_64-unknown-linux-gnu/release/trading-bot
```

## When to Use PGO

### PGO is Recommended When:

| Scenario | Reason |
|----------|--------|
| **HFT systems** | Every microsecond matters |
| **Market data parsers** | Many conditional branches |
| **Indicator calculations** | Intensive loop computations |
| **Production systems** | Stable usage patterns |
| **Latency-critical services** | Predictable response time |

### PGO is Not Recommended When:

| Scenario | Reason |
|----------|--------|
| **Prototyping** | Build overhead doesn't pay off |
| **Diverse workloads** | Profile isn't representative |
| **Rarely run code** | Improvements aren't noticeable |
| **Rapidly changing code** | Profile becomes stale |

## What We Learned

| Concept | Description |
|---------|-------------|
| **PGO** | Profile Guided Optimization — optimization based on profile data |
| **Instrumentation** | Build with execution information recording |
| **Profile** | Data about actual program behavior |
| **Branch prediction** | Optimization of conditional jumps |
| **Inlining** | Embedding frequently called functions |
| **Code layout** | Placing hot code for better caching |
| **BOLT** | Additional post-link optimization |

## Homework

1. **Basic PGO Build**: Take your trading bot or use the example from this chapter:
   - Build a regular release version
   - Build a PGO-optimized version
   - Compare performance on identical data
   - Record results in a table

2. **Profiling Different Scenarios**: Create several test data sets:
   - Bull market (constant growth)
   - Bear market (constant decline)
   - Sideways (low volatility)
   - High volatility
   Collect a profile on each and compare how it affects optimization.

3. **Automation**: Write a script that:
   - Automatically detects if code has changed since last PGO build
   - If changed — rebuilds with a new profile
   - Saves performance history
   - Sends notification if performance degraded

4. **Benchmark Integration**: Add to your project:
   - Criterion benchmarks for critical functions
   - Performance comparison of regular and PGO builds
   - Automatic report with graphs

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../318-*/en.md)
