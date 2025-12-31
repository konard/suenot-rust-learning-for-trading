# Day 318: Binary Size: Reducing

## Trading Analogy

Imagine you're deploying a trading bot to a small VPS server or an embedded device at a remote location (like a co-located server close to an exchange). Every megabyte of your binary costs money:

- **Storage costs**: Cloud providers charge for disk space
- **Deployment time**: Larger binaries take longer to transfer over networks
- **Memory overhead**: Bigger binaries often consume more RAM at runtime
- **Startup time**: Loading a 100MB binary is slower than loading a 10MB one

This is similar to how traders optimize their portfolios:
- **Lean portfolios**: Only hold assets that serve a purpose
- **Efficient capital use**: Don't tie up money in unnecessary positions
- **Quick execution**: Smaller, focused positions can be adjusted faster

In high-frequency trading, every millisecond matters. A bloated binary can mean the difference between capturing and missing a trade. When deploying to edge devices or containers, size directly impacts costs and performance.

## Why Rust Binaries Can Be Large

By default, Rust prioritizes:
- **Debug information** for development
- **Monomorphization** (generating specialized code for each generic type)
- **Static linking** (including all dependencies in one binary)
- **Panic handling** with full stack traces

This results in larger but faster and easier-to-debug binaries. For production trading systems, we often need to balance these trade-offs.

## Basic Size Reduction Techniques

### 1. Release Mode Compilation

The most fundamental optimization:

```bash
# Debug build (default) - large, slow, debuggable
cargo build

# Release build - smaller, faster, optimized
cargo build --release
```

```rust
fn main() {
    let prices = vec![100.0, 105.0, 102.0, 108.0, 110.0];

    // This code is optimized differently in debug vs release
    let sma: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

    println!("SMA(5): ${:.2}", sma);
}
```

**Typical size difference:**
- Debug: ~10-50 MB
- Release: ~2-10 MB

### 2. Cargo.toml Optimization Settings

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[profile.release]
# Optimization level (0-3, "s" for size, "z" for minimal size)
opt-level = "z"         # Optimize for size

# Link-Time Optimization - allows optimization across crate boundaries
lto = true              # Enable LTO

# Reduce parallel code generation units (better optimization, slower compile)
codegen-units = 1       # Better optimization

# Strip symbols from binary
strip = true            # Remove debug symbols

# Panic handling - "abort" is smaller than "unwind"
panic = "abort"         # Smaller panic handling

[dependencies]
# Use minimal feature sets
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

### 3. Understanding opt-level

```toml
[profile.release]
# opt-level values:
# 0 = no optimization (fast compile, large binary)
# 1 = basic optimization
# 2 = moderate optimization (default for release)
# 3 = aggressive optimization (may increase size!)
# "s" = optimize for size
# "z" = optimize for minimal size (most aggressive)
opt-level = "z"
```

## Example: Trading Bot Size Optimization

Let's build a simple trading bot and optimize its size:

```rust
use std::collections::HashMap;

/// Simple price tracker for portfolio monitoring
struct PortfolioTracker {
    positions: HashMap<String, Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl PortfolioTracker {
    fn new() -> Self {
        PortfolioTracker {
            positions: HashMap::new(),
        }
    }

    fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        let position = Position {
            symbol: symbol.to_string(),
            quantity,
            avg_price: price,
        };
        self.positions.insert(symbol.to_string(), position);
    }

    fn get_total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.values().map(|pos| {
            let current_price = current_prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * current_price
        }).sum()
    }

    fn calculate_pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.values().map(|pos| {
            let current_price = current_prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * (current_price - pos.avg_price)
        }).sum()
    }
}

fn main() {
    let mut tracker = PortfolioTracker::new();

    // Add some positions
    tracker.add_position("BTCUSD", 0.5, 45000.0);
    tracker.add_position("ETHUSD", 2.0, 2800.0);
    tracker.add_position("SOLUSD", 10.0, 95.0);

    // Current market prices
    let mut prices = HashMap::new();
    prices.insert("BTCUSD".to_string(), 48000.0);
    prices.insert("ETHUSD".to_string(), 3100.0);
    prices.insert("SOLUSD".to_string(), 105.0);

    let total_value = tracker.get_total_value(&prices);
    let pnl = tracker.calculate_pnl(&prices);

    println!("=== Portfolio Summary ===");
    println!("Total Value: ${:.2}", total_value);
    println!("Unrealized PnL: ${:+.2}", pnl);
}
```

## Link-Time Optimization (LTO)

LTO allows the compiler to optimize across crate boundaries:

```toml
[profile.release]
# LTO options:
# false = no LTO (fast compile)
# true = full LTO (slow compile, best optimization)
# "thin" = thin LTO (balance between speed and optimization)
# "fat" = same as true
lto = true
```

### How LTO helps trading systems:

```rust
// crate: indicators
pub fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let sum: f64 = prices.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

// crate: trading-bot (uses indicators)
fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];

    // With LTO, the compiler can inline calculate_sma
    // and optimize the entire call chain
    if let Some(sma) = calculate_sma(&prices, 3) {
        println!("SMA(3): {:.2}", sma);
    }
}
```

Without LTO, the compiler can't optimize across crate boundaries. With LTO enabled, it can inline and optimize the entire program as a unit.

## Strip Symbols and Debug Info

Debug symbols add significant size but are essential for debugging:

```toml
[profile.release]
# Strip options:
# "none" = keep all symbols
# "debuginfo" = strip debug info, keep symbol names
# "symbols" = strip everything
strip = "symbols"
```

### Alternative: Using strip command

```bash
# Build release binary
cargo build --release

# Check size before stripping
ls -lh target/release/trading-bot

# Strip symbols manually (if not using Cargo strip)
strip target/release/trading-bot

# Check size after stripping
ls -lh target/release/trading-bot
```

## Panic Strategy: Abort vs Unwind

```toml
[profile.release]
# Panic behavior:
# "unwind" = stack unwinding, allows catch_unwind, larger binary
# "abort" = immediate abort, smaller binary
panic = "abort"
```

### Trade-offs for trading systems:

```rust
use std::panic;

fn risky_calculation(data: &[f64]) -> f64 {
    // With panic = "unwind", we can catch panics
    let result = panic::catch_unwind(|| {
        data.iter().sum::<f64>() / data.len() as f64
    });

    match result {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Calculation failed, returning default");
            0.0
        }
    }
}

fn main() {
    let prices: Vec<f64> = vec![];

    // With panic = "abort", any panic terminates immediately
    // This is often acceptable for trading bots that should
    // restart cleanly rather than continue in undefined state
    let avg = risky_calculation(&prices);
    println!("Average: {}", avg);
}
```

**Recommendation for trading systems:** Use `panic = "abort"` in production. It's better for a trading bot to crash and restart cleanly than to continue in a potentially corrupted state.

## Feature Flags: Using Only What You Need

Many crates have optional features. Only enable what you need:

```toml
[dependencies]
# Full serde (large)
# serde = "1.0"

# Minimal serde (smaller)
serde = { version = "1.0", default-features = false, features = ["derive"] }

# Full tokio (large)
# tokio = { version = "1.0", features = ["full"] }

# Minimal tokio for trading bot (smaller)
tokio = { version = "1.0", features = ["rt", "net", "time"] }

# reqwest without default features
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }
```

### Example: Minimal HTTP Client for Price Fetching

```rust
use std::collections::HashMap;

// Using reqwest with minimal features
// reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }

async fn fetch_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Simulated API response
    let mock_prices: HashMap<&str, f64> = [
        ("BTCUSD", 48000.0),
        ("ETHUSD", 3100.0),
    ].iter().cloned().collect();

    mock_prices.get(symbol)
        .copied()
        .ok_or_else(|| format!("Price not found for {}", symbol).into())
}

#[tokio::main]
async fn main() {
    match fetch_price("BTCUSD").await {
        Ok(price) => println!("BTC Price: ${:.2}", price),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Codegen Units

Reducing codegen units allows better optimization but increases compile time:

```toml
[profile.release]
# Default is 16, which allows parallel compilation
# Setting to 1 enables better optimization
codegen-units = 1
```

## Advanced: UPX Compression

UPX (Ultimate Packer for eXecutables) can further compress binaries:

```bash
# Install UPX
# Ubuntu/Debian: sudo apt install upx
# macOS: brew install upx

# Compress the binary
upx --best target/release/trading-bot

# Or with aggressive compression (slower decompression)
upx --ultra-brute target/release/trading-bot
```

**Note:** UPX adds a small decompression overhead at startup. For HFT systems where startup time matters, test the trade-off.

## Size Analysis Tools

### Using cargo-bloat

```bash
# Install cargo-bloat
cargo install cargo-bloat

# Analyze binary size by function
cargo bloat --release -n 20

# Analyze by crate
cargo bloat --release --crates
```

### Sample Output

```
File  .text     Size Crate
0.8%   4.5%  43.5KiB std
0.5%   2.8%  27.1KiB serde_json
0.3%   1.7%  16.4KiB regex
0.2%   1.2%  11.5KiB trading_bot
...
```

### Using cargo-size

```bash
# Check binary size
cargo size --release

# Detailed section analysis
cargo size --release -- -A
```

## Complete Optimization Profile for Trading Systems

```toml
[package]
name = "high-performance-trading-bot"
version = "1.0.0"
edition = "2021"

# Optimize dependencies in debug builds too
[profile.dev.package."*"]
opt-level = 2

[profile.release]
# Size optimization (use "3" if speed is more important than size)
opt-level = "z"

# Link-Time Optimization
lto = true

# Single codegen unit for best optimization
codegen-units = 1

# Strip all symbols
strip = "symbols"

# Abort on panic (smaller, cleaner for trading systems)
panic = "abort"

# Reduce debug info in release
debug = false

# Incremental compilation off for release (better optimization)
incremental = false

[dependencies]
# Use minimal feature sets
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false, features = ["std"] }
tokio = { version = "1.0", default-features = false, features = ["rt", "macros", "time"] }
```

## Measuring Impact

Here's a script to measure optimization impact:

```rust
//! Build script to report binary size
//! Save as: build_and_measure.rs

use std::process::Command;
use std::fs;

fn get_file_size(path: &str) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} bytes", bytes)
    }
}

fn main() {
    println!("=== Binary Size Optimization Report ===\n");

    // Build debug
    println!("Building debug...");
    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Failed to build debug");

    // Build release
    println!("Building release...");
    Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to build release");

    // Measure sizes
    let debug_size = get_file_size("target/debug/trading-bot").unwrap_or(0);
    let release_size = get_file_size("target/release/trading-bot").unwrap_or(0);

    println!("\n=== Results ===");
    println!("Debug binary:   {}", format_size(debug_size));
    println!("Release binary: {}", format_size(release_size));

    if debug_size > 0 && release_size > 0 {
        let reduction = (1.0 - release_size as f64 / debug_size as f64) * 100.0;
        println!("Size reduction: {:.1}%", reduction);
    }
}
```

## What We Learned

| Technique | Size Reduction | Trade-off |
|-----------|---------------|-----------|
| Release mode | 70-90% | Longer compile time |
| opt-level = "z" | 10-30% | Slightly slower execution |
| LTO = true | 10-20% | Much longer compile time |
| codegen-units = 1 | 5-15% | Longer compile time |
| strip = true | 20-50% | No debugging symbols |
| panic = "abort" | 5-10% | No panic recovery |
| Minimal features | Varies | May lose functionality |
| UPX compression | 30-60% | Startup overhead |

## Practical Exercises

1. **Size Comparison**: Create a simple trading bot and measure binary size with:
   - Default debug build
   - Default release build
   - Release with `opt-level = "z"`
   - Release with full optimization profile
   - After UPX compression

   Record all sizes and calculate percentage reductions.

2. **Feature Audit**: Take an existing Rust project and:
   - List all dependencies with `cargo tree`
   - Identify which features can be disabled
   - Create a minimal feature set
   - Measure size reduction

3. **Trade-off Analysis**: Build a trading signal calculator with:
   - Full optimization profile (smallest size)
   - Performance profile (opt-level = 3)
   - Measure and compare:
     - Binary size
     - Execution time for 1 million calculations
     - Memory usage

4. **Deployment Simulation**: Create a Docker container for a trading bot:
   - One with unoptimized binary
   - One with fully optimized binary
   - Compare container sizes and startup times

## Homework

1. **Trading Bot Optimizer**: Create a CLI tool that:
   - Analyzes a Cargo.toml file
   - Suggests optimization settings
   - Runs size comparison builds
   - Generates a report with recommendations
   - Includes specific advice for trading applications

2. **Binary Size Dashboard**: Build a monitoring system that:
   - Tracks binary size across git commits
   - Alerts when size increases significantly
   - Shows size breakdown by crate
   - Compares different optimization profiles
   - Generates historical size charts

3. **Feature Minimizer**: Write a tool that:
   - Parses Cargo.toml dependencies
   - Identifies unused features via static analysis
   - Suggests minimal feature sets
   - Tests compilation with reduced features
   - Reports size savings

4. **Deployment Pipeline**: Create a CI/CD pipeline that:
   - Builds with multiple optimization profiles
   - Runs performance benchmarks on each
   - Compares binary sizes
   - Selects optimal profile based on constraints (size vs speed)
   - Deploys to different environments (HFT = speed, edge = size)

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../319-*/en.md)
