# Day 352: Publishing to crates.io

## Trading Analogy

Imagine you've developed a powerful algorithm for calculating trading indicators. Now you have two options:

**Private Tool:**
You keep the code to yourself and only use it personally. Every time you need this functionality in a new project, you copy the code manually.

**Publishing on the Code Exchange (crates.io):**
You "list" your algorithm on a public platform. Now any developer (including you) can integrate it with a single line in `Cargo.toml`. It's like an IPO for your code — it becomes available to the entire community.

| Aspect | Private Code | Publishing to crates.io |
|--------|--------------|------------------------|
| **Availability** | Only you | Worldwide |
| **Updates** | Manual copying | `cargo update` |
| **Versioning** | None | Semantic versioning |
| **Documentation** | Local | Automatic on docs.rs |
| **Dependencies** | Manual management | Automatic resolution |

## Preparing a Crate for Publishing

### Cargo.toml Structure

```toml
[package]
name = "trading-indicators"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "High-performance trading indicators for algorithmic trading"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/trading-indicators"
homepage = "https://github.com/yourusername/trading-indicators"
documentation = "https://docs.rs/trading-indicators"
readme = "README.md"
keywords = ["trading", "indicators", "finance", "algorithms", "rust"]
categories = ["finance", "algorithms", "mathematics"]
exclude = ["tests/data/*", ".github/*"]

[dependencies]
# Dependencies for calculations
```

### Required Fields

```rust
/// Example library structure for trading indicators
///
/// # Example Usage
///
/// ```rust
/// use trading_indicators::{SMA, TradingIndicator};
///
/// let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];
/// let sma = SMA::new(3);
/// let result = sma.calculate(&prices);
/// println!("SMA: {:?}", result);
/// ```

/// Trait for all trading indicators
pub trait TradingIndicator {
    /// Calculates indicator values from prices
    fn calculate(&self, prices: &[f64]) -> Vec<f64>;

    /// Returns the indicator name
    fn name(&self) -> &str;

    /// Minimum number of data points required
    fn min_periods(&self) -> usize;
}

/// Simple Moving Average (SMA)
#[derive(Debug, Clone)]
pub struct SMA {
    period: usize,
}

impl SMA {
    /// Creates a new SMA with the given period
    ///
    /// # Arguments
    ///
    /// * `period` - Averaging period (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if period equals 0
    ///
    /// # Example
    ///
    /// ```rust
    /// use trading_indicators::SMA;
    ///
    /// let sma = SMA::new(20);
    /// assert_eq!(sma.period(), 20);
    /// ```
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        SMA { period }
    }

    /// Returns the SMA period
    pub fn period(&self) -> usize {
        self.period
    }
}

impl TradingIndicator for SMA {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period {
            return vec![];
        }

        prices
            .windows(self.period)
            .map(|window| window.iter().sum::<f64>() / self.period as f64)
            .collect()
    }

    fn name(&self) -> &str {
        "SMA"
    }

    fn min_periods(&self) -> usize {
        self.period
    }
}

/// Exponential Moving Average (EMA)
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
}

impl EMA {
    /// Creates a new EMA with the given period
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA { period, multiplier }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl TradingIndicator for EMA {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period {
            return vec![];
        }

        let mut result = Vec::with_capacity(prices.len() - self.period + 1);

        // First value is SMA
        let first_ema: f64 = prices[..self.period].iter().sum::<f64>()
            / self.period as f64;
        result.push(first_ema);

        // Subsequent values use EMA formula
        let mut prev_ema = first_ema;
        for price in &prices[self.period..] {
            let ema = (price - prev_ema) * self.multiplier + prev_ema;
            result.push(ema);
            prev_ema = ema;
        }

        result
    }

    fn name(&self) -> &str {
        "EMA"
    }

    fn min_periods(&self) -> usize {
        self.period
    }
}
```

## Documentation for Publishing

### Writing README.md

```markdown
# Trading Indicators

[![Crates.io](https://img.shields.io/crates/v/trading-indicators.svg)](https://crates.io/crates/trading-indicators)
[![Documentation](https://docs.rs/trading-indicators/badge.svg)](https://docs.rs/trading-indicators)
[![License](https://img.shields.io/crates/l/trading-indicators.svg)](LICENSE)

High-performance trading indicators for algorithmic trading in Rust.

## Features

- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- Relative Strength Index (RSI)
- MACD
- Bollinger Bands

## Installation

Add to your `Cargo.toml`:

\```toml
[dependencies]
trading-indicators = "0.1"
\```

## Quick Start

\```rust
use trading_indicators::{SMA, EMA, TradingIndicator};

fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];

    // Simple Moving Average
    let sma = SMA::new(3);
    let sma_values = sma.calculate(&prices);
    println!("SMA(3): {:?}", sma_values);

    // Exponential Moving Average
    let ema = EMA::new(3);
    let ema_values = ema.calculate(&prices);
    println!("EMA(3): {:?}", ema_values);
}
\```

## License

Licensed under either of Apache License, Version 2.0 or MIT license.
```

### Code Documentation (doc comments)

```rust
//! # Trading Indicators Library
//!
//! A library for calculating trading indicators.
//!
//! ## Overview
//!
//! This library provides high-performance implementations
//! of popular trading indicators:
//!
//! - [`SMA`] - Simple Moving Average
//! - [`EMA`] - Exponential Moving Average
//! - [`RSI`] - Relative Strength Index
//! - [`MACD`] - Moving Average Convergence Divergence
//!
//! ## Example Usage
//!
//! ```rust
//! use trading_indicators::{SMA, TradingIndicator};
//!
//! let btc_prices = vec![
//!     50000.0, 50500.0, 51000.0, 50800.0, 51200.0,
//!     51500.0, 51300.0, 52000.0, 52500.0, 52300.0,
//! ];
//!
//! let sma = SMA::new(5);
//! let sma_values = sma.calculate(&btc_prices);
//!
//! // Trend detection
//! if let (Some(&last_price), Some(&last_sma)) =
//!     (btc_prices.last(), sma_values.last())
//! {
//!     if last_price > last_sma {
//!         println!("Uptrend: price above SMA");
//!     } else {
//!         println!("Downtrend: price below SMA");
//!     }
//! }
//! ```
//!
//! ## Performance
//!
//! All indicators are optimized for working with large data arrays
//! and use Rust iterators to minimize allocations.

/// Relative Strength Index (RSI)
///
/// RSI measures the speed and magnitude of price changes
/// to identify overbought or oversold conditions.
///
/// # Formula
///
/// RSI = 100 - (100 / (1 + RS))
/// where RS = average gain / average loss
///
/// # Interpretation
///
/// - RSI > 70: Asset is overbought (possible downward correction)
/// - RSI < 30: Asset is oversold (possible upward bounce)
///
/// # Example
///
/// ```rust
/// use trading_indicators::{RSI, TradingIndicator};
///
/// let prices = vec![
///     44.0, 44.25, 44.50, 43.75, 44.50,
///     44.25, 44.50, 44.00, 43.50, 43.25,
///     43.50, 44.25, 44.75, 45.00, 45.50,
/// ];
///
/// let rsi = RSI::new(14);
/// if let Some(&current_rsi) = rsi.calculate(&prices).last() {
///     match current_rsi {
///         r if r > 70.0 => println!("Overbought: {:.1}", r),
///         r if r < 30.0 => println!("Oversold: {:.1}", r),
///         r => println!("Neutral: {:.1}", r),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        RSI { period }
    }
}

impl TradingIndicator for RSI {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period + 1 {
            return vec![];
        }

        // Calculate price changes
        let changes: Vec<f64> = prices
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        let mut gains: Vec<f64> = Vec::new();
        let mut losses: Vec<f64> = Vec::new();

        for change in &changes {
            if *change > 0.0 {
                gains.push(*change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let mut result = Vec::new();

        // First RSI
        let mut avg_gain: f64 = gains[..self.period].iter().sum::<f64>()
            / self.period as f64;
        let mut avg_loss: f64 = losses[..self.period].iter().sum::<f64>()
            / self.period as f64;

        for i in self.period..gains.len() {
            avg_gain = (avg_gain * (self.period - 1) as f64 + gains[i])
                / self.period as f64;
            avg_loss = (avg_loss * (self.period - 1) as f64 + losses[i])
                / self.period as f64;

            let rs = if avg_loss != 0.0 {
                avg_gain / avg_loss
            } else {
                100.0
            };

            result.push(100.0 - (100.0 / (1.0 + rs)));
        }

        result
    }

    fn name(&self) -> &str {
        "RSI"
    }

    fn min_periods(&self) -> usize {
        self.period + 1
    }
}
```

## Publishing Process

### Step 1: Register on crates.io

```bash
# Create an account at https://crates.io via GitHub

# Get your API token from profile settings
# https://crates.io/settings/tokens

# Authorize with Cargo
cargo login <your-api-token>
```

### Step 2: Pre-publication Checks

```bash
# Check for errors
cargo check

# Run tests
cargo test

# Verify documentation
cargo doc --no-deps --open

# Dry run of publishing (without actual publishing)
cargo publish --dry-run

# Verify crate packages correctly
cargo package --list
```

### Step 3: Publishing

```rust
// Demonstrating the publishing process through code
fn demonstrate_publish_process() {
    println!("=== Publishing Process to crates.io ===\n");

    // Step 1: Version check
    println!("1. Checking version in Cargo.toml...");
    println!("   version = \"0.1.0\"");

    // Step 2: Metadata check
    println!("\n2. Required fields:");
    println!("   - name: crate name (unique)");
    println!("   - version: SemVer version");
    println!("   - license: MIT OR Apache-2.0");
    println!("   - description: brief description");

    // Step 3: Publishing
    println!("\n3. Publish command:");
    println!("   cargo publish");

    // After publishing
    println!("\n4. After successful publishing:");
    println!("   - Crate available on crates.io");
    println!("   - Documentation on docs.rs");
    println!("   - Cannot delete or overwrite version!");
}

fn main() {
    demonstrate_publish_process();
}
```

## Versioning (SemVer)

```rust
/// Demonstrating semantic versioning for a trading library
///
/// MAJOR.MINOR.PATCH
///
/// - MAJOR: Incompatible API changes
/// - MINOR: New functionality with backward compatibility
/// - PATCH: Bug fixes

// Version 0.1.0 - Initial version
mod v0_1_0 {
    pub struct SMA {
        pub period: usize,
    }

    impl SMA {
        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect()
        }
    }
}

// Version 0.1.1 - PATCH: Fixed bug with empty array
mod v0_1_1 {
    pub struct SMA {
        pub period: usize,
    }

    impl SMA {
        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            // Added check for empty array (PATCH)
            if prices.len() < self.period {
                return vec![];
            }

            prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect()
        }
    }
}

// Version 0.2.0 - MINOR: Added new indicator
mod v0_2_0 {
    pub struct SMA {
        period: usize,
    }

    // New functionality (MINOR)
    pub struct EMA {
        period: usize,
    }

    impl EMA {
        pub fn new(period: usize) -> Self {
            EMA { period }
        }

        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            // EMA implementation
            vec![]
        }
    }
}

// Version 1.0.0 - MAJOR: Changed API
mod v1_0_0 {
    /// Now uses Result instead of panic
    pub struct SMA {
        period: usize,
    }

    #[derive(Debug)]
    pub enum IndicatorError {
        InvalidPeriod,
        InsufficientData,
    }

    impl SMA {
        // BREAKING CHANGE: returns Result
        pub fn new(period: usize) -> Result<Self, IndicatorError> {
            if period == 0 {
                return Err(IndicatorError::InvalidPeriod);
            }
            Ok(SMA { period })
        }

        // BREAKING CHANGE: returns Result
        pub fn calculate(&self, prices: &[f64]) -> Result<Vec<f64>, IndicatorError> {
            if prices.len() < self.period {
                return Err(IndicatorError::InsufficientData);
            }

            Ok(prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect())
        }
    }
}

fn main() {
    println!("=== Semantic Versioning ===\n");

    println!("0.1.0 -> 0.1.1: PATCH");
    println!("  Bug fixed, API unchanged\n");

    println!("0.1.1 -> 0.2.0: MINOR");
    println!("  EMA added, old code still works\n");

    println!("0.2.0 -> 1.0.0: MAJOR");
    println!("  API changed (Result instead of panic)");
    println!("  Users need to update their code");
}
```

## Dependency Management

```toml
# Cargo.toml with proper dependency management

[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# Exact version (not recommended for most cases)
serde = "=1.0.193"

# Compatible version (recommended)
tokio = "1.35"          # Any 1.x.y where x >= 35

# With features specified
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

# From git repository (for development)
# trading-indicators = { git = "https://github.com/user/trading-indicators" }

# Local path (for development)
# trading-indicators = { path = "../trading-indicators" }

[dev-dependencies]
# Dependencies for tests only
criterion = "0.5"

[build-dependencies]
# Dependencies for build.rs
# cc = "1.0"

# Optional features
[features]
default = ["sma", "ema"]
sma = []
ema = []
rsi = []
macd = []
all-indicators = ["sma", "ema", "rsi", "macd"]
```

### Example Using Features

```rust
//! Library with optional indicators

#[cfg(feature = "sma")]
mod sma;

#[cfg(feature = "ema")]
mod ema;

#[cfg(feature = "rsi")]
mod rsi;

#[cfg(feature = "macd")]
mod macd;

// Re-export enabled modules
#[cfg(feature = "sma")]
pub use sma::SMA;

#[cfg(feature = "ema")]
pub use ema::EMA;

#[cfg(feature = "rsi")]
pub use rsi::RSI;

#[cfg(feature = "macd")]
pub use macd::MACD;

/// Check enabled features
pub fn available_indicators() -> Vec<&'static str> {
    let mut indicators = Vec::new();

    #[cfg(feature = "sma")]
    indicators.push("SMA");

    #[cfg(feature = "ema")]
    indicators.push("EMA");

    #[cfg(feature = "rsi")]
    indicators.push("RSI");

    #[cfg(feature = "macd")]
    indicators.push("MACD");

    indicators
}

fn main() {
    println!("Available indicators: {:?}", available_indicators());
}
```

## Practical Example: Publishing a Trading Library

```rust
//! # Trading Strategy Library
//!
//! A full-featured library for algorithmic trading.

use std::collections::HashMap;

/// Trading signal
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy { price: f64, quantity: f64 },
    Sell { price: f64, quantity: f64 },
    Hold,
}

/// Trait for trading strategies
pub trait Strategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;

    /// Generate signal based on data
    fn generate_signal(&self, data: &MarketData) -> Signal;

    /// Strategy parameters
    fn parameters(&self) -> HashMap<String, f64>;
}

/// Market data
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub prices: Vec<f64>,
    pub volumes: Vec<f64>,
    pub timestamps: Vec<i64>,
}

impl MarketData {
    pub fn new(symbol: &str) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            prices: Vec::new(),
            volumes: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    pub fn add_candle(&mut self, price: f64, volume: f64, timestamp: i64) {
        self.prices.push(price);
        self.volumes.push(volume);
        self.timestamps.push(timestamp);
    }

    pub fn last_price(&self) -> Option<f64> {
        self.prices.last().copied()
    }
}

/// Moving Average Crossover Strategy
#[derive(Debug, Clone)]
pub struct CrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
}

impl CrossoverStrategy {
    /// Creates a new MA crossover strategy
    ///
    /// # Arguments
    ///
    /// * `fast_period` - Fast MA period
    /// * `slow_period` - Slow MA period
    ///
    /// # Example
    ///
    /// ```rust
    /// use trading_strategy::{CrossoverStrategy, Strategy, MarketData};
    ///
    /// let strategy = CrossoverStrategy::new(10, 20);
    ///
    /// let mut data = MarketData::new("BTCUSDT");
    /// for i in 0..30 {
    ///     data.add_candle(50000.0 + (i as f64 * 100.0), 1000.0, i);
    /// }
    ///
    /// let signal = strategy.generate_signal(&data);
    /// println!("{:?}", signal);
    /// ```
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        CrossoverStrategy { fast_period, slow_period }
    }

    fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }
}

impl Strategy for CrossoverStrategy {
    fn name(&self) -> &str {
        "MA Crossover"
    }

    fn generate_signal(&self, data: &MarketData) -> Signal {
        let fast_ma = match self.calculate_sma(&data.prices, self.fast_period) {
            Some(ma) => ma,
            None => return Signal::Hold,
        };

        let slow_ma = match self.calculate_sma(&data.prices, self.slow_period) {
            Some(ma) => ma,
            None => return Signal::Hold,
        };

        let current_price = match data.last_price() {
            Some(p) => p,
            None => return Signal::Hold,
        };

        // Buy signal: fast MA crosses above slow MA
        if fast_ma > slow_ma {
            Signal::Buy {
                price: current_price,
                quantity: 1.0,
            }
        }
        // Sell signal: fast MA crosses below slow MA
        else if fast_ma < slow_ma {
            Signal::Sell {
                price: current_price,
                quantity: 1.0,
            }
        } else {
            Signal::Hold
        }
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("fast_period".to_string(), self.fast_period as f64);
        params.insert("slow_period".to_string(), self.slow_period as f64);
        params
    }
}

/// Strategy manager
pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn new() -> Self {
        StrategyManager {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub fn generate_signals(&self, data: &MarketData) -> Vec<(&str, Signal)> {
        self.strategies
            .iter()
            .map(|s| (s.name(), s.generate_signal(data)))
            .collect()
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("=== Trading Library Demo ===\n");

    // Create market data
    let mut data = MarketData::new("BTCUSDT");

    // Simulate uptrend
    let base_price = 50000.0;
    for i in 0..50 {
        let price = base_price + (i as f64 * 50.0) + (i as f64).sin() * 100.0;
        data.add_candle(price, 1000.0, i);
    }

    // Create strategy
    let strategy = CrossoverStrategy::new(5, 20);

    println!("Strategy: {}", strategy.name());
    println!("Parameters: {:?}", strategy.parameters());
    println!("Last price: ${:.2}", data.last_price().unwrap());

    // Generate signal
    let signal = strategy.generate_signal(&data);
    println!("\nSignal: {:?}", signal);

    // Use strategy manager
    let mut manager = StrategyManager::new();
    manager.add_strategy(Box::new(CrossoverStrategy::new(5, 20)));
    manager.add_strategy(Box::new(CrossoverStrategy::new(10, 30)));

    println!("\n=== Signals from All Strategies ===");
    for (name, signal) in manager.generate_signals(&data) {
        println!("{}: {:?}", name, signal);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **crates.io** | Official Rust package registry |
| **Cargo.toml** | Crate configuration with metadata |
| **SemVer** | Semantic versioning (MAJOR.MINOR.PATCH) |
| **cargo publish** | Command to publish a crate |
| **docs.rs** | Automatic documentation for published crates |
| **Features** | Optional crate capabilities |
| **API Token** | Token for publishing authorization |

## Practical Exercises

1. **Create an Indicators Library**: Create a crate with:
   - Implementation of SMA, EMA, RSI
   - Full documentation for each function
   - Usage examples in doc-tests
   - README.md with instructions
   - Tests for all public functions

2. **Library Versioning**: Develop:
   - Version 0.1.0 with basic functionality
   - Version 0.2.0 with new indicators
   - Version 1.0.0 with improved API
   - CHANGELOG.md describing changes

3. **Features for Crate**: Add:
   - Optional heavy dependencies
   - Configurable components
   - Feature for async API
   - Documentation for using features

4. **CI/CD for Publishing**: Set up:
   - Automatic testing on PR
   - Formatting checks (rustfmt)
   - Style checks (clippy)
   - Automatic publishing on tag creation

## Homework

1. **Full-featured Trading Library**: Create a crate:
   - With at least 5 indicators (SMA, EMA, RSI, MACD, Bollinger Bands)
   - With documentation in Russian and English
   - With examples in examples/ directory
   - With performance benchmarks
   - Publish to crates.io (or prepare for publishing)

2. **Workspace with Multiple Crates**: Create:
   - trading-core: base types and traits
   - trading-indicators: indicator implementations
   - trading-strategies: trading strategies
   - trading-bot: CLI application
   - Configure versioning between crates

3. **Version Migration**: Write:
   - Migration guide from 0.x to 1.0
   - Deprecation warnings for outdated API
   - Tools for automatic migration
   - Backward compatibility tests

4. **Documentation Like Top Crates**: Study:
   - Documentation of serde, tokio, reqwest
   - Create similar structure for your crate
   - Add cookbook with recipes
   - Create interactive examples

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../353-*/en.md)
