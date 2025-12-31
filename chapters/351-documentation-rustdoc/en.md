# Day 351: Documentation: rustdoc

## Trading Analogy

Imagine you've created a complex trading system. It works great, generates profits, but six months later you return to the code and can't understand:
- What does the `calculate_signal()` function do?
- What parameters does `execute_order()` accept?
- Why is there such complex logic in `risk_manager`?

It's like a trading strategy without a trade journal — impossible to understand why certain decisions were made.

**Rustdoc** is Rust's built-in documentation tool:
- Automatically generates HTML documentation from code
- Supports code examples that are verified during compilation
- Creates navigation for modules, structures, and functions

Like a broker's API documentation: without it, you can't understand what requests to send and what responses to expect.

## Why Document Code?

In professional algorithmic trading, documentation is:

| Reason | Description | Example |
|--------|-------------|---------|
| **For yourself** | Remember the logic after a month | Why is stop-loss 2.5%? |
| **For the team** | Other developers understand the API | New developer onboards quickly |
| **For audit** | Regulator requires system understanding | Explain decision-making logic |
| **For tests** | Examples in docs are tests | Automatic correctness verification |

## Rustdoc Basics

### Documentation Comments

Rust has three types of comments:

```rust
// Regular comment — doesn't go into documentation

/// Documentation comment for the following item
/// Used for functions, structures, modules

//! Documentation comment for the current module
//! Usually at the beginning of lib.rs or mod.rs
```

### Documenting a Trading Structure

```rust
/// A trading order to be sent to the exchange.
///
/// Represents a buy or sell request for an asset with specified parameters.
/// Used for interaction with the exchange API.
///
/// # Examples
///
/// ```
/// use trading_lib::Order;
///
/// let order = Order::new("BTCUSDT", Side::Buy, 0.1, 50000.0);
/// assert_eq!(order.symbol(), "BTCUSDT");
/// ```
///
/// # Fields
///
/// * `symbol` - Trading pair (e.g., "BTCUSDT")
/// * `side` - Trade direction (buy or sell)
/// * `quantity` - Asset quantity
/// * `price` - Execution price
#[derive(Debug, Clone)]
pub struct Order {
    /// Trading pair (e.g., "BTCUSDT", "ETHUSDT")
    pub symbol: String,
    /// Trade direction
    pub side: Side,
    /// Asset quantity to trade
    pub quantity: f64,
    /// Limit execution price (None for market orders)
    pub price: Option<f64>,
    /// Order creation timestamp
    pub created_at: u64,
}

/// Direction of the trading operation.
///
/// Determines whether we're buying or selling an asset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    /// Buy asset (open long position)
    Buy,
    /// Sell asset (open short position or close long)
    Sell,
}

impl Order {
    /// Creates a new limit order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair (e.g., "BTCUSDT")
    /// * `side` - Trade direction
    /// * `quantity` - Asset quantity
    /// * `price` - Limit execution price
    ///
    /// # Examples
    ///
    /// ```
    /// let buy_order = Order::new("BTCUSDT", Side::Buy, 0.5, 45000.0);
    /// let sell_order = Order::new("ETHUSDT", Side::Sell, 2.0, 3000.0);
    /// ```
    ///
    /// # Panics
    ///
    /// The function doesn't panic but returns an order with zero timestamp.
    pub fn new(symbol: &str, side: Side, quantity: f64, price: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            side,
            quantity,
            price: Some(price),
            created_at: 0, // In reality: current timestamp
        }
    }

    /// Creates a market order (executes at current price).
    ///
    /// Market orders execute immediately at the best available price.
    /// Used when execution speed matters more than exact price.
    ///
    /// # Examples
    ///
    /// ```
    /// let market_order = Order::market("BTCUSDT", Side::Buy, 1.0);
    /// assert!(market_order.price.is_none());
    /// ```
    ///
    /// # Warning
    ///
    /// In volatile markets, the execution price may significantly
    /// differ from expected (slippage).
    pub fn market(symbol: &str, side: Side, quantity: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            side,
            quantity,
            price: None,
            created_at: 0,
        }
    }

    /// Returns the order's trading pair.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

fn main() {
    let order = Order::new("BTCUSDT", Side::Buy, 0.5, 50000.0);
    println!("Created order: {:?}", order);
}
```

## Documentation Sections

Rustdoc supports special sections:

```rust
/// Calculates the RSI (Relative Strength Index) indicator.
///
/// RSI measures the speed and change of price movements.
/// Values above 70 indicate overbought conditions,
/// below 30 — oversold conditions.
///
/// # Arguments
///
/// * `prices` - Slice of closing prices
/// * `period` - Calculation period (usually 14)
///
/// # Returns
///
/// A vector of RSI values. First `period` values will be None
/// since history is needed for calculation.
///
/// # Examples
///
/// ```
/// let prices = vec![44.0, 44.5, 44.2, 44.8, 45.1, 45.3, 45.0];
/// let rsi = calculate_rsi(&prices, 14);
/// ```
///
/// # Errors
///
/// Returns an empty vector if:
/// * `prices` contains fewer than `period + 1` elements
/// * `period` is zero
///
/// # Panics
///
/// The function doesn't panic.
///
/// # Safety
///
/// The function is safe to use in multithreaded context.
///
/// # Performance
///
/// Complexity: O(n), where n is the number of prices.
/// Uses a single pass through the data.
///
/// # See Also
///
/// * [`calculate_macd`] - Another popular indicator
/// * [`calculate_bollinger_bands`] - Bollinger Bands
pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    if prices.len() < period + 1 || period == 0 {
        return vec![];
    }

    let mut result = vec![None; period];

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    if gains.len() < period {
        return result;
    }

    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };
        result.push(Some(rsi));
    }

    result
}

fn main() {
    let prices = vec![44.0, 44.5, 44.2, 44.8, 45.1, 45.3, 45.0, 44.8, 44.5, 44.9,
                      45.2, 45.5, 45.8, 46.0, 45.7, 45.5, 45.3, 45.1, 45.4, 45.6];
    let rsi = calculate_rsi(&prices, 14);

    println!("RSI values:");
    for (i, value) in rsi.iter().enumerate() {
        if let Some(v) = value {
            println!("  Day {}: {:.2}", i + 1, v);
        }
    }
}
```

## Module Documentation

For modules, use `//!`:

```rust
//! # Trading Library
//!
//! This library provides tools for algorithmic trading.
//!
//! ## Main Features
//!
//! * Technical indicators (RSI, MACD, Bollinger Bands)
//! * Order management
//! * Risk management
//! * Strategy backtesting
//!
//! ## Quick Start
//!
//! ```rust
//! use trading_lib::{Order, Side, RiskManager};
//!
//! // Create an order
//! let order = Order::new("BTCUSDT", Side::Buy, 0.1, 50000.0);
//!
//! // Check risks
//! let risk_manager = RiskManager::new(10000.0, 0.02);
//! if risk_manager.approve(&order) {
//!     println!("Order approved!");
//! }
//! ```
//!
//! ## Modules
//!
//! * [`indicators`] - Technical indicators
//! * [`orders`] - Order management
//! * [`risk`] - Risk management
//!
//! ## Dependencies
//!
//! The library uses minimal dependencies for high performance.

/// Technical indicators module.
///
/// Contains implementations of popular market analysis indicators:
/// * RSI — Relative Strength Index
/// * MACD — Moving Average Convergence/Divergence
/// * Bollinger Bands — Volatility bands
pub mod indicators {
    /// Calculates Simple Moving Average (SMA).
    ///
    /// # Examples
    ///
    /// ```
    /// let prices = vec![10.0, 11.0, 12.0, 11.5, 12.5];
    /// let sma = calculate_sma(&prices, 3);
    /// ```
    pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.len() < period || period == 0 {
            return vec![];
        }

        prices
            .windows(period)
            .map(|window| window.iter().sum::<f64>() / period as f64)
            .collect()
    }

    /// Calculates Exponential Moving Average (EMA).
    ///
    /// EMA gives more weight to recent prices compared to SMA.
    ///
    /// # Formula
    ///
    /// `EMA = Price * k + EMA_prev * (1 - k)`
    /// where `k = 2 / (period + 1)`
    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.is_empty() || period == 0 {
            return vec![];
        }

        let k = 2.0 / (period as f64 + 1.0);
        let mut ema = vec![prices[0]];

        for price in prices.iter().skip(1) {
            let prev_ema = *ema.last().unwrap();
            ema.push(price * k + prev_ema * (1.0 - k));
        }

        ema
    }
}

/// Risk management module.
pub mod risk {
    /// Risk manager for trading operations.
    ///
    /// Controls position sizes and maximum losses.
    pub struct RiskManager {
        /// Total capital
        pub capital: f64,
        /// Maximum risk per trade (fraction of capital)
        pub max_risk_per_trade: f64,
    }

    impl RiskManager {
        /// Creates a new risk manager.
        pub fn new(capital: f64, max_risk_per_trade: f64) -> Self {
            RiskManager {
                capital,
                max_risk_per_trade,
            }
        }

        /// Calculates maximum position size.
        ///
        /// # Formula
        ///
        /// `position_size = (capital * max_risk) / (entry_price * stop_loss_pct)`
        ///
        /// # Examples
        ///
        /// ```
        /// let rm = RiskManager::new(10000.0, 0.02);
        /// let size = rm.calculate_position_size(50000.0, 0.02);
        /// assert!(size > 0.0);
        /// ```
        pub fn calculate_position_size(&self, entry_price: f64, stop_loss_pct: f64) -> f64 {
            if entry_price <= 0.0 || stop_loss_pct <= 0.0 {
                return 0.0;
            }
            (self.capital * self.max_risk_per_trade) / (entry_price * stop_loss_pct)
        }
    }
}

fn main() {
    use indicators::{calculate_sma, calculate_ema};
    use risk::RiskManager;

    let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5];

    let sma = calculate_sma(&prices, 3);
    println!("SMA(3): {:?}", sma);

    let ema = calculate_ema(&prices, 3);
    println!("EMA(3): {:?}", ema);

    let rm = RiskManager::new(10000.0, 0.02);
    let position_size = rm.calculate_position_size(50000.0, 0.02);
    println!("Max position size: {:.4} BTC", position_size);
}
```

## Code Examples in Documentation

Examples in documentation are compiled and tested:

```rust
/// Validates a trading signal.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.75);
/// assert!(signal.is_valid());
/// ```
///
/// Signal with low confidence:
///
/// ```
/// let weak_signal = TradingSignal::new("ETHUSDT", Action::Sell, 0.3);
/// assert!(!weak_signal.is_strong());
/// ```
///
/// Example with panic (uses should_panic):
///
/// ```should_panic
/// let invalid = TradingSignal::new("", Action::Buy, 1.5);
/// invalid.validate().unwrap();
/// ```
///
/// Example that doesn't compile (for demonstration):
///
/// ```compile_fail
/// let signal = TradingSignal::new("BTC", Action::Buy, 0.5);
/// signal.private_method(); // Private method is inaccessible
/// ```
///
/// Example that doesn't run:
///
/// ```no_run
/// let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.9);
/// signal.execute().await; // Requires async runtime
/// ```
///
/// Hidden lines (not shown in documentation):
///
/// ```
/// # use std::collections::HashMap;
/// # let mut cache = HashMap::new();
/// # cache.insert("BTCUSDT", 50000.0);
/// let price = cache.get("BTCUSDT").unwrap();
/// assert_eq!(*price, 50000.0);
/// ```
#[derive(Debug)]
pub struct TradingSignal {
    symbol: String,
    action: Action,
    confidence: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Buy,
    Sell,
    Hold,
}

impl TradingSignal {
    pub fn new(symbol: &str, action: Action, confidence: f64) -> Self {
        TradingSignal {
            symbol: symbol.to_string(),
            action,
            confidence,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.symbol.is_empty() && self.confidence >= 0.0 && self.confidence <= 1.0
    }

    pub fn is_strong(&self) -> bool {
        self.confidence >= 0.7
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err("Confidence must be between 0 and 1".to_string());
        }
        Ok(())
    }
}

fn main() {
    let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.85);
    println!("Signal: {:?}", signal);
    println!("Is valid: {}", signal.is_valid());
    println!("Is strong: {}", signal.is_strong());
}
```

## Generating Documentation

### cargo doc Commands

```bash
# Generate documentation
cargo doc

# Generate and open in browser
cargo doc --open

# Include dependency documentation
cargo doc --document-private-items

# For a specific package
cargo doc --package my_trading_lib

# Test examples in documentation
cargo test --doc
```

### Configuration in Cargo.toml

```toml
[package]
name = "trading_lib"
version = "0.1.0"
edition = "2021"
authors = ["Trading Team <team@example.com>"]
description = "Library for algorithmic trading"
documentation = "https://docs.rs/trading_lib"
repository = "https://github.com/example/trading_lib"
license = "MIT"
keywords = ["trading", "finance", "algorithms"]
categories = ["finance", "algorithms"]

# Documentation settings
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[features]
default = []
# Enable extended documentation
full-docs = []
```

## Links and Navigation

```rust
/// Trading event handler.
///
/// Related types:
/// * [`Order`] — trading order
/// * [`Trade`](crate::trades::Trade) — executed trade
/// * [`Position`](super::Position) — open position
///
/// External links:
/// * [Binance API](https://binance-docs.github.io/apidocs/)
/// * [Trading View](https://www.tradingview.com/)
///
/// Methods:
/// * [`Self::process`] — process event
/// * [`Self::new`] — create handler
///
/// Modules:
/// * [`crate::indicators`] — technical indicators
/// * [`crate::risk`] — risk management
pub struct EventHandler {
    orders: Vec<Order>,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
}

impl EventHandler {
    /// Creates a new event handler.
    pub fn new() -> Self {
        EventHandler { orders: Vec::new() }
    }

    /// Processes a trading event.
    ///
    /// See also: [`Order`], [`Self::new`]
    pub fn process(&mut self, order: Order) {
        self.orders.push(order);
    }
}

fn main() {
    let mut handler = EventHandler::new();
    handler.process(Order { id: "ORD-001".to_string() });
    println!("Orders processed: {}", handler.orders.len());
}
```

## Practical Example: Documenting a Trading Library

```rust
//! # Trading Strategy Library
//!
//! Library for developing and testing trading strategies.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │  Market     │────▶│  Strategy   │────▶│  Orders     │
//! │  Data       │     │  Engine     │     │  Manager    │
//! └─────────────┘     └─────────────┘     └─────────────┘
//!                           │
//!                           ▼
//!                     ┌─────────────┐
//!                     │    Risk     │
//!                     │  Manager    │
//!                     └─────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use trading_lib::{Strategy, MarketData, RiskManager};
//!
//! fn main() {
//!     // Initialization
//!     let mut strategy = MACrossoverStrategy::new(10, 50);
//!     let risk_manager = RiskManager::new(10000.0, 0.02);
//!
//!     // Get data
//!     let data = MarketData::fetch("BTCUSDT").await?;
//!
//!     // Generate signal
//!     if let Some(signal) = strategy.analyze(&data) {
//!         if risk_manager.approve(&signal) {
//!             strategy.execute(signal).await?;
//!         }
//!     }
//! }
//! ```

use std::collections::HashMap;

/// Market data for analysis.
///
/// Contains OHLCV data (Open, High, Low, Close, Volume)
/// for a single trading instrument.
///
/// # Data Structure
///
/// | Field | Description | Example |
/// |-------|-------------|---------|
/// | symbol | Trading pair | "BTCUSDT" |
/// | prices | Closing prices | [50000.0, 50100.0, ...] |
/// | volumes | Volumes | [100.5, 150.2, ...] |
///
/// # Performance
///
/// Stores data in vectors for fast sequential access.
/// For random access, use indices.
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Trading pair
    pub symbol: String,
    /// Opening prices
    pub open: Vec<f64>,
    /// High prices
    pub high: Vec<f64>,
    /// Low prices
    pub low: Vec<f64>,
    /// Closing prices
    pub close: Vec<f64>,
    /// Trading volumes
    pub volume: Vec<f64>,
}

impl MarketData {
    /// Creates an empty market data container.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = MarketData::new("BTCUSDT");
    /// assert_eq!(data.len(), 0);
    /// ```
    pub fn new(symbol: &str) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            open: Vec::new(),
            high: Vec::new(),
            low: Vec::new(),
            close: Vec::new(),
            volume: Vec::new(),
        }
    }

    /// Returns the number of candles.
    pub fn len(&self) -> usize {
        self.close.len()
    }

    /// Checks if data is empty.
    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }

    /// Adds a new candle.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut data = MarketData::new("BTCUSDT");
    /// data.add_candle(50000.0, 50500.0, 49800.0, 50200.0, 1000.0);
    /// assert_eq!(data.len(), 1);
    /// ```
    pub fn add_candle(&mut self, open: f64, high: f64, low: f64, close: f64, volume: f64) {
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
        self.volume.push(volume);
    }

    /// Returns the last closing price.
    ///
    /// # Returns
    ///
    /// `Some(price)` if data exists, otherwise `None`.
    pub fn last_close(&self) -> Option<f64> {
        self.close.last().copied()
    }
}

/// Moving average crossover strategy.
///
/// Generates a buy signal when the short MA
/// crosses above the long MA, and vice versa for sell.
///
/// # Parameters
///
/// * `short_period` — short MA period (usually 10-20)
/// * `long_period` — long MA period (usually 50-200)
///
/// # Example
///
/// ```
/// let strategy = MACrossoverStrategy::new(10, 50);
/// let signal = strategy.generate_signal(&market_data);
/// ```
///
/// # Signal Logic
///
/// | Condition | Signal |
/// |-----------|--------|
/// | Short MA > Long MA (crossover) | Buy |
/// | Short MA < Long MA (crossover) | Sell |
/// | No crossover | Hold |
pub struct MACrossoverStrategy {
    short_period: usize,
    long_period: usize,
    prev_short_above: Option<bool>,
}

/// Trading signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Buy signal
    Buy,
    /// Sell signal
    Sell,
    /// Hold position
    Hold,
}

impl MACrossoverStrategy {
    /// Creates a new strategy with specified periods.
    ///
    /// # Panics
    ///
    /// Panics if `short_period >= long_period`.
    ///
    /// # Examples
    ///
    /// ```
    /// let strategy = MACrossoverStrategy::new(10, 50);
    /// ```
    ///
    /// ```should_panic
    /// let invalid = MACrossoverStrategy::new(50, 10);
    /// ```
    pub fn new(short_period: usize, long_period: usize) -> Self {
        assert!(short_period < long_period, "Short period must be less than long period");
        MACrossoverStrategy {
            short_period,
            long_period,
            prev_short_above: None,
        }
    }

    /// Generates a trading signal based on market data.
    ///
    /// # Arguments
    ///
    /// * `data` — market data with closing prices
    ///
    /// # Returns
    ///
    /// Trading signal (`Buy`, `Sell`, or `Hold`).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut strategy = MACrossoverStrategy::new(5, 10);
    /// let mut data = MarketData::new("BTCUSDT");
    ///
    /// // Add data...
    /// for i in 0..20 {
    ///     data.add_candle(100.0, 101.0, 99.0, 100.0 + i as f64, 1000.0);
    /// }
    ///
    /// let signal = strategy.generate_signal(&data);
    /// println!("Signal: {:?}", signal);
    /// ```
    pub fn generate_signal(&mut self, data: &MarketData) -> Signal {
        if data.close.len() < self.long_period {
            return Signal::Hold;
        }

        let short_ma = self.calculate_sma(&data.close, self.short_period);
        let long_ma = self.calculate_sma(&data.close, self.long_period);

        let short_above = short_ma > long_ma;

        let signal = match self.prev_short_above {
            Some(prev) if prev != short_above => {
                if short_above { Signal::Buy } else { Signal::Sell }
            }
            _ => Signal::Hold,
        };

        self.prev_short_above = Some(short_above);
        signal
    }

    fn calculate_sma(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        sum / period as f64
    }
}

fn main() {
    // Usage demonstration
    let mut data = MarketData::new("BTCUSDT");

    // Add test data
    let prices = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 105.0,
                  104.5, 106.0, 107.0, 108.0, 107.5, 109.0, 110.0,
                  111.0, 112.0, 113.0, 114.0, 115.0, 116.0];

    for price in prices.iter() {
        data.add_candle(*price, price + 1.0, price - 1.0, *price, 1000.0);
    }

    let mut strategy = MACrossoverStrategy::new(5, 10);
    let signal = strategy.generate_signal(&data);

    println!("Market Data: {} candles", data.len());
    println!("Strategy Signal: {:?}", signal);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **`///`** | Documentation comment for an item |
| **`//!`** | Documentation comment for a module |
| **Sections** | `# Examples`, `# Arguments`, `# Returns`, `# Panics`, `# Errors` |
| **Code examples** | Compiled and tested automatically |
| **Attributes** | `should_panic`, `no_run`, `compile_fail`, `ignore` |
| **Links** | `[`Name`]`, `[`crate::path`]`, `[`Self::method`]` |
| **cargo doc** | HTML documentation generation |
| **cargo test --doc** | Testing examples in documentation |

## Practical Exercises

1. **Indicator Documentation**: Write complete documentation for a MACD calculation function:
   - Algorithm description
   - All parameters with example values
   - Usage examples
   - Edge case handling

2. **Module Documentation**: Create a `position_manager` module with full documentation:
   - Module description with diagram
   - Documentation for all public types
   - Usage examples
   - Links between types

3. **Testable Examples**: Write documentation with examples for:
   - Successful scenario
   - Error handling
   - Edge cases
   - Example with panic

4. **CI Integration**: Set up automatic:
   - Documentation generation on push
   - Testing examples in documentation
   - Publishing to GitHub Pages

## Homework

1. **Complete Library Documentation**: Create a documented trading library:
   - At least 5 public structures with complete documentation
   - Each function with code examples
   - Modular documentation with overview
   - Architecture diagram in README
   - All examples must compile

2. **Changelog and Versioning**: Implement a change documentation system:
   - Structure for storing changes by version
   - Automatic CHANGELOG.md generation
   - Breaking changes documentation
   - Migration examples between versions

3. **Interactive Documentation**: Create extended documentation:
   - Examples for each public API
   - Tutorials as doc-tests
   - FAQ in module documentation
   - Links to external resources

4. **Documentation with Benchmarks**: Add to documentation:
   - Performance information
   - Comparison of different approaches
   - Optimization recommendations
   - Big-O notation for algorithms

## Navigation

[← Previous Day](../326-async-vs-threading/en.md) | [Next Day →](../352-*/en.md)
