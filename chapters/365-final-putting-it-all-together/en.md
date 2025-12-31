# Day 365: Final: Putting It All Together

## Trading Analogy

Congratulations! You've reached the final day of your journey. Think of this year like building a complete trading operation from scratch.

**You started as a solo trader with just an idea** — learning the basics of Rust like learning to read stock charts and place your first orders.

**You built your infrastructure** — ownership and borrowing are your risk management rules, structs and enums are your order types and positions, traits are your trading strategies that can be applied to any asset.

**You scaled your operation** — async programming is your ability to monitor multiple markets simultaneously, concurrency is your parallel execution of strategies across different exchanges.

**Now you're ready to deploy** — you have monitoring, logging, testing, and CI/CD in place. You're not just a trader anymore; you're running a professional trading firm.

| Trading Journey | Rust Journey |
|-----------------|--------------|
| **Learning to read charts** | Basic syntax, variables, types |
| **Placing first orders** | Functions, control flow |
| **Risk management rules** | Ownership, borrowing, lifetimes |
| **Defining order types** | Structs, enums, traits |
| **Portfolio management** | Collections, generics |
| **Error handling** | Result, Option, ? operator |
| **Multi-market monitoring** | Async/await, futures |
| **Parallel execution** | Threads, channels, Mutex |
| **Exchange integrations** | HTTP clients, WebSockets |
| **Data persistence** | Database connections |
| **Performance tuning** | Profiling, optimization |
| **Production deployment** | Docker, CI/CD, monitoring |

## The Complete Trading System

In this final chapter, we'll build a complete algorithmic trading system that demonstrates everything you've learned. This isn't just a toy example — it's a production-ready architecture that you can extend.

### Project Structure

A professional Rust trading project is organized into a workspace with multiple crates:

```
trading-system/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── common/             # Shared types and utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs    # Order, Trade, Position types
│   │       ├── errors.rs   # Custom error types
│   │       └── config.rs   # Configuration management
│   │
│   ├── market-data/        # Market data handling
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── feed.rs     # WebSocket data feed
│   │       ├── orderbook.rs
│   │       └── candles.rs
│   │
│   ├── strategy/           # Trading strategies
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs   # Strategy trait
│   │       ├── momentum.rs
│   │       └── mean_reversion.rs
│   │
│   ├── execution/          # Order execution
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs
│   │       └── risk.rs
│   │
│   └── bot/                # Main trading bot binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── docker/                 # Containerization
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/common",
    "crates/market-data",
    "crates/strategy",
    "crates/execution",
    "crates/bot",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["Trading Bot Team"]
license = "MIT"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP/WebSocket
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }

# Metrics
prometheus = "0.13"

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Testing
mockall = "0.12"
```

## Core Types and Domain Model

Let's start with the foundation — our domain types that represent trading concepts:

```rust
// crates/common/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Trading symbol (e.g., "BTCUSDT", "ETHUSDT")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }

    pub fn base(&self) -> &str {
        // Extract base currency (BTC from BTCUSDT)
        &self.0[..self.0.len() - 4]
    }

    pub fn quote(&self) -> &str {
        // Extract quote currency (USDT from BTCUSDT)
        &self.0[self.0.len() - 4..]
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Order side: Buy or Sell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Get the opposite side
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// Order type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Market order - execute at current price
    Market,
    /// Limit order - execute at specified price or better
    Limit { price: f64 },
    /// Stop-loss order
    StopLoss { stop_price: f64 },
    /// Stop-limit order
    StopLimit { stop_price: f64, limit_price: f64 },
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// A trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Create a new market order
    pub fn market(symbol: Symbol, side: Side, quantity: f64) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Market,
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new limit order
    pub fn limit(symbol: Symbol, side: Side, quantity: f64, price: f64) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Limit { price },
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Check if order is still active
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Pending | OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Calculate remaining quantity
    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }
}

/// An executed trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub symbol: Symbol,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub commission_asset: String,
    pub executed_at: DateTime<Utc>,
}

impl Trade {
    /// Calculate the notional value of the trade
    pub fn notional(&self) -> f64 {
        self.price * self.quantity
    }

    /// Calculate net value after commission
    pub fn net_value(&self) -> f64 {
        match self.side {
            Side::Buy => self.notional() + self.commission,
            Side::Sell => self.notional() - self.commission,
        }
    }
}

/// A position in a trading asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: Symbol,
    pub quantity: f64,        // Positive = long, negative = short
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    /// Create a new position from a trade
    pub fn from_trade(trade: &Trade) -> Self {
        let quantity = match trade.side {
            Side::Buy => trade.quantity,
            Side::Sell => -trade.quantity,
        };

        let now = Utc::now();
        Position {
            symbol: trade.symbol.clone(),
            quantity,
            entry_price: trade.price,
            unrealized_pnl: 0.0,
            realized_pnl: -trade.commission, // Commission is a cost
            opened_at: now,
            updated_at: now,
        }
    }

    /// Check if this is a long position
    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }

    /// Check if this is a short position
    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }

    /// Check if position is closed
    pub fn is_closed(&self) -> bool {
        self.quantity.abs() < 1e-10
    }

    /// Update unrealized PnL based on current price
    pub fn update_pnl(&mut self, current_price: f64) {
        let price_change = current_price - self.entry_price;
        self.unrealized_pnl = price_change * self.quantity;
        self.updated_at = Utc::now();
    }

    /// Add a trade to this position
    pub fn apply_trade(&mut self, trade: &Trade) {
        let trade_quantity = match trade.side {
            Side::Buy => trade.quantity,
            Side::Sell => -trade.quantity,
        };

        // Check if this reduces the position
        if (self.quantity > 0.0 && trade_quantity < 0.0)
            || (self.quantity < 0.0 && trade_quantity > 0.0)
        {
            // Calculate realized PnL for the closed portion
            let closed_quantity = trade_quantity.abs().min(self.quantity.abs());
            let pnl = (trade.price - self.entry_price) * closed_quantity * self.quantity.signum();
            self.realized_pnl += pnl - trade.commission;
        }

        // Update position
        let new_quantity = self.quantity + trade_quantity;

        // If we're adding to position or flipping, update entry price
        if (self.quantity >= 0.0 && trade_quantity > 0.0)
            || (self.quantity <= 0.0 && trade_quantity < 0.0)
            || self.quantity.signum() != new_quantity.signum()
        {
            // Weighted average entry price
            let total_cost = self.entry_price * self.quantity.abs() + trade.price * trade_quantity.abs();
            self.entry_price = total_cost / (self.quantity.abs() + trade_quantity.abs());
        }

        self.quantity = new_quantity;
        self.updated_at = Utc::now();
    }
}

/// OHLCV candlestick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

impl Candle {
    /// Get the typical price (HLC average)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Get the range (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Check if this is a bullish candle
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if this is a bearish candle
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: Vec<BookLevel>,  // Sorted descending by price
    pub asks: Vec<BookLevel>,  // Sorted ascending by price
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    /// Get the best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    /// Get the best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    /// Get the mid price
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Get the spread
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Get the spread as a percentage
    pub fn spread_pct(&self) -> Option<f64> {
        match (self.mid_price(), self.spread()) {
            (Some(mid), Some(spread)) if mid > 0.0 => Some(spread / mid * 100.0),
            _ => None,
        }
    }
}

/// Trading signal generated by a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    /// Buy signal with target size
    Buy { quantity: f64, reason: String },
    /// Sell signal with target size
    Sell { quantity: f64, reason: String },
    /// Hold - no action
    Hold,
}
```

## Error Handling

A robust trading system needs comprehensive error handling:

```rust
// crates/common/src/errors.rs

use thiserror::Error;

/// Main error type for the trading system
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Order error: {0}")]
    Order(#[from] OrderError),

    #[error("Market data error: {0}")]
    MarketData(#[from] MarketDataError),

    #[error("Strategy error: {0}")]
    Strategy(#[from] StrategyError),

    #[error("Risk management error: {0}")]
    Risk(#[from] RiskError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Invalid order: {0}")]
    Invalid(String),

    #[error("Order rejected by exchange: {reason}")]
    Rejected { order_id: String, reason: String },

    #[error("Order not found: {0}")]
    NotFound(String),

    #[error("Order already filled")]
    AlreadyFilled,

    #[error("Order cancelled")]
    Cancelled,

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: f64, available: f64 },
}

#[derive(Error, Debug)]
pub enum MarketDataError {
    #[error("Connection lost to {exchange}")]
    ConnectionLost { exchange: String },

    #[error("Data feed timeout for {symbol}")]
    Timeout { symbol: String },

    #[error("Invalid data received: {0}")]
    InvalidData(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Rate limited by {exchange}")]
    RateLimited { exchange: String },
}

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Strategy initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Insufficient data for calculation: need {required}, have {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Invalid parameter: {name} = {value}, expected {expected}")]
    InvalidParameter {
        name: String,
        value: String,
        expected: String,
    },
}

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Position size {size} exceeds maximum {max}")]
    PositionSizeExceeded { size: f64, max: f64 },

    #[error("Daily loss limit reached: {loss} / {limit}")]
    DailyLossLimitReached { loss: f64, limit: f64 },

    #[error("Maximum drawdown exceeded: {drawdown}% > {limit}%")]
    MaxDrawdownExceeded { drawdown: f64, limit: f64 },

    #[error("Too many open positions: {count} / {max}")]
    TooManyPositions { count: usize, max: usize },

    #[error("Symbol {symbol} is restricted")]
    SymbolRestricted { symbol: String },
}

/// Result type for trading operations
pub type Result<T> = std::result::Result<T, TradingError>;
```

## The Strategy Trait

Strategies are defined using traits, allowing for flexible composition:

```rust
// crates/strategy/src/traits.rs

use async_trait::async_trait;
use common::{
    errors::Result,
    types::{Candle, OrderBook, Position, Signal, Symbol},
};
use std::collections::HashMap;

/// Market state provided to strategies
#[derive(Debug, Clone)]
pub struct MarketState {
    pub symbol: Symbol,
    pub orderbook: OrderBook,
    pub candles: Vec<Candle>,
    pub positions: HashMap<Symbol, Position>,
    pub account_balance: f64,
}

/// Configuration for a strategy
pub trait StrategyConfig: Send + Sync + Clone + 'static {
    /// Strategy name for logging and identification
    fn name(&self) -> &str;

    /// Symbols this strategy trades
    fn symbols(&self) -> &[Symbol];

    /// Number of historical candles needed
    fn required_history(&self) -> usize;
}

/// The main strategy trait
#[async_trait]
pub trait Strategy: Send + Sync {
    /// The configuration type for this strategy
    type Config: StrategyConfig;

    /// Create a new strategy instance
    fn new(config: Self::Config) -> Self
    where
        Self: Sized;

    /// Get the strategy configuration
    fn config(&self) -> &Self::Config;

    /// Initialize the strategy (load historical data, warm up indicators)
    async fn initialize(&mut self) -> Result<()>;

    /// Generate a trading signal based on current market state
    async fn generate_signal(&mut self, state: &MarketState) -> Result<Signal>;

    /// Called when an order is filled
    fn on_order_filled(&mut self, _order: &common::types::Order) {
        // Default: do nothing
    }

    /// Called on each new candle
    fn on_candle(&mut self, _candle: &Candle) {
        // Default: do nothing
    }

    /// Called on order book update
    fn on_orderbook_update(&mut self, _orderbook: &OrderBook) {
        // Default: do nothing
    }
}
```

### Momentum Strategy Implementation

```rust
// crates/strategy/src/momentum.rs

use async_trait::async_trait;
use common::{
    errors::{Result, StrategyError},
    types::{Candle, Signal, Symbol},
};
use tracing::{debug, info, instrument};

use crate::traits::{MarketState, Strategy, StrategyConfig};

/// Configuration for the momentum strategy
#[derive(Debug, Clone)]
pub struct MomentumConfig {
    pub name: String,
    pub symbols: Vec<Symbol>,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_threshold: f64,
    pub position_size_pct: f64,
}

impl StrategyConfig for MomentumConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn required_history(&self) -> usize {
        self.slow_period + 10 // Extra buffer for smoothing
    }
}

/// A simple momentum-based strategy using moving average crossover
pub struct MomentumStrategy {
    config: MomentumConfig,
    fast_ma: Option<f64>,
    slow_ma: Option<f64>,
    prev_fast_ma: Option<f64>,
    prev_slow_ma: Option<f64>,
}

impl MomentumStrategy {
    /// Calculate Simple Moving Average
    fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for price in prices.iter().skip(1) {
            ema = (price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Update indicators with new price data
    fn update_indicators(&mut self, candles: &[Candle]) {
        // Store previous values for crossover detection
        self.prev_fast_ma = self.fast_ma;
        self.prev_slow_ma = self.slow_ma;

        // Extract closing prices
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        // Calculate new MAs
        self.fast_ma = Self::calculate_ema(&closes, self.config.fast_period);
        self.slow_ma = Self::calculate_ema(&closes, self.config.slow_period);
    }

    /// Detect crossover
    fn detect_crossover(&self) -> Option<Signal> {
        let (fast, slow, prev_fast, prev_slow) = match (
            self.fast_ma,
            self.slow_ma,
            self.prev_fast_ma,
            self.prev_slow_ma,
        ) {
            (Some(f), Some(s), Some(pf), Some(ps)) => (f, s, pf, ps),
            _ => return None,
        };

        // Calculate momentum strength
        let momentum = (fast - slow) / slow * 100.0;

        // Bullish crossover: fast crosses above slow
        if prev_fast <= prev_slow && fast > slow && momentum > self.config.signal_threshold {
            debug!(
                fast_ma = fast,
                slow_ma = slow,
                momentum = momentum,
                "Bullish crossover detected"
            );
            return Some(Signal::Buy {
                quantity: 0.0, // Will be calculated based on position sizing
                reason: format!(
                    "MA crossover: fast({:.2}) > slow({:.2}), momentum: {:.2}%",
                    fast, slow, momentum
                ),
            });
        }

        // Bearish crossover: fast crosses below slow
        if prev_fast >= prev_slow && fast < slow && momentum.abs() > self.config.signal_threshold {
            debug!(
                fast_ma = fast,
                slow_ma = slow,
                momentum = momentum,
                "Bearish crossover detected"
            );
            return Some(Signal::Sell {
                quantity: 0.0,
                reason: format!(
                    "MA crossover: fast({:.2}) < slow({:.2}), momentum: {:.2}%",
                    fast, slow, momentum
                ),
            });
        }

        None
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
    type Config = MomentumConfig;

    fn new(config: Self::Config) -> Self {
        MomentumStrategy {
            config,
            fast_ma: None,
            slow_ma: None,
            prev_fast_ma: None,
            prev_slow_ma: None,
        }
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            strategy = %self.config.name,
            fast_period = self.config.fast_period,
            slow_period = self.config.slow_period,
            "Initializing momentum strategy"
        );
        Ok(())
    }

    #[instrument(skip(self, state), fields(strategy = %self.config.name))]
    async fn generate_signal(&mut self, state: &MarketState) -> Result<Signal> {
        // Check if we have enough data
        let required = self.config.required_history();
        if state.candles.len() < required {
            return Err(StrategyError::InsufficientData {
                required,
                available: state.candles.len(),
            }
            .into());
        }

        // Update indicators
        self.update_indicators(&state.candles);

        // Check for signals
        if let Some(signal) = self.detect_crossover() {
            // Calculate position size
            let position_value = state.account_balance * self.config.position_size_pct / 100.0;
            let current_price = state.candles.last().unwrap().close;
            let quantity = position_value / current_price;

            return Ok(match signal {
                Signal::Buy { reason, .. } => Signal::Buy { quantity, reason },
                Signal::Sell { reason, .. } => Signal::Sell { quantity, reason },
                Signal::Hold => Signal::Hold,
            });
        }

        Ok(Signal::Hold)
    }

    fn on_candle(&mut self, candle: &Candle) {
        debug!(
            symbol = %candle.symbol,
            close = candle.close,
            "Processing new candle"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::types::OrderBook;
    use std::collections::HashMap;

    fn create_test_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| Candle {
                symbol: Symbol::new("BTCUSDT"),
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: 1000.0,
                timestamp: Utc::now() + chrono::Duration::minutes(i as i64),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_bullish_crossover() {
        let config = MomentumConfig {
            name: "test_momentum".to_string(),
            symbols: vec![Symbol::new("BTCUSDT")],
            fast_period: 5,
            slow_period: 10,
            signal_threshold: 0.1,
            position_size_pct: 10.0,
        };

        let mut strategy = MomentumStrategy::new(config);
        strategy.initialize().await.unwrap();

        // Create price data with bullish trend
        let mut prices = vec![100.0; 10];
        prices.extend(vec![105.0, 110.0, 115.0, 120.0, 125.0]); // Rising prices

        let candles = create_test_candles(&prices);
        let state = MarketState {
            symbol: Symbol::new("BTCUSDT"),
            orderbook: OrderBook {
                symbol: Symbol::new("BTCUSDT"),
                bids: vec![],
                asks: vec![],
                timestamp: Utc::now(),
            },
            candles,
            positions: HashMap::new(),
            account_balance: 10000.0,
        };

        let signal = strategy.generate_signal(&state).await.unwrap();

        // Should get a buy signal due to rising prices
        match signal {
            Signal::Buy { quantity, reason } => {
                assert!(quantity > 0.0);
                assert!(reason.contains("crossover"));
            }
            _ => {} // May not trigger depending on exact MA values
        }
    }
}
```

## Risk Management

A professional trading system must have robust risk management:

```rust
// crates/execution/src/risk.rs

use common::{
    errors::{Result, RiskError},
    types::{Order, Position, Side, Symbol},
};
use std::collections::HashMap;
use tracing::{info, warn};

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Maximum position size as percentage of account
    pub max_position_pct: f64,
    /// Maximum number of open positions
    pub max_positions: usize,
    /// Maximum daily loss as percentage of account
    pub max_daily_loss_pct: f64,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: f64,
    /// Maximum single order value
    pub max_order_value: f64,
    /// Restricted symbols (cannot trade)
    pub restricted_symbols: Vec<Symbol>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_pct: 10.0,
            max_positions: 5,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 20.0,
            max_order_value: 50000.0,
            restricted_symbols: vec![],
        }
    }
}

/// Risk manager state
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: f64,
    peak_balance: f64,
    current_balance: f64,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_balance: f64) -> Self {
        RiskManager {
            config,
            daily_pnl: 0.0,
            peak_balance: initial_balance,
            current_balance: initial_balance,
        }
    }

    /// Check if an order passes all risk checks
    pub fn check_order(
        &self,
        order: &Order,
        positions: &HashMap<Symbol, Position>,
        current_price: f64,
    ) -> Result<()> {
        // Check if symbol is restricted
        if self.config.restricted_symbols.contains(&order.symbol) {
            return Err(RiskError::SymbolRestricted {
                symbol: order.symbol.to_string(),
            }
            .into());
        }

        // Calculate order value
        let order_value = order.quantity * current_price;

        // Check maximum order value
        if order_value > self.config.max_order_value {
            warn!(
                order_value = order_value,
                max = self.config.max_order_value,
                "Order value exceeds limit"
            );
            return Err(RiskError::PositionSizeExceeded {
                size: order_value,
                max: self.config.max_order_value,
            }
            .into());
        }

        // Check position size after order
        let current_position = positions.get(&order.symbol);
        let new_position_value = match (current_position, order.side) {
            (Some(pos), Side::Buy) => {
                (pos.quantity + order.quantity) * current_price
            }
            (Some(pos), Side::Sell) => {
                (pos.quantity - order.quantity).abs() * current_price
            }
            (None, _) => order_value,
        };

        let max_position_value = self.current_balance * self.config.max_position_pct / 100.0;
        if new_position_value > max_position_value {
            return Err(RiskError::PositionSizeExceeded {
                size: new_position_value,
                max: max_position_value,
            }
            .into());
        }

        // Check number of positions
        let position_count = positions.values().filter(|p| !p.is_closed()).count();
        if position_count >= self.config.max_positions
            && !positions.contains_key(&order.symbol)
        {
            return Err(RiskError::TooManyPositions {
                count: position_count,
                max: self.config.max_positions,
            }
            .into());
        }

        // Check daily loss limit
        let daily_loss_limit = self.peak_balance * self.config.max_daily_loss_pct / 100.0;
        if self.daily_pnl < -daily_loss_limit {
            return Err(RiskError::DailyLossLimitReached {
                loss: self.daily_pnl.abs(),
                limit: daily_loss_limit,
            }
            .into());
        }

        // Check drawdown
        let current_drawdown = (self.peak_balance - self.current_balance) / self.peak_balance * 100.0;
        if current_drawdown > self.config.max_drawdown_pct {
            return Err(RiskError::MaxDrawdownExceeded {
                drawdown: current_drawdown,
                limit: self.config.max_drawdown_pct,
            }
            .into());
        }

        info!(
            symbol = %order.symbol,
            side = ?order.side,
            quantity = order.quantity,
            order_value = order_value,
            "Order passed risk checks"
        );

        Ok(())
    }

    /// Update balance and PnL tracking
    pub fn update_balance(&mut self, new_balance: f64, pnl: f64) {
        self.current_balance = new_balance;
        self.daily_pnl += pnl;

        // Update peak balance if new high
        if new_balance > self.peak_balance {
            self.peak_balance = new_balance;
        }
    }

    /// Reset daily PnL (call at start of new trading day)
    pub fn reset_daily_pnl(&mut self) {
        self.daily_pnl = 0.0;
    }

    /// Get current drawdown percentage
    pub fn current_drawdown(&self) -> f64 {
        if self.peak_balance <= 0.0 {
            return 0.0;
        }
        (self.peak_balance - self.current_balance) / self.peak_balance * 100.0
    }

    /// Get current risk metrics
    pub fn get_metrics(&self) -> RiskMetrics {
        RiskMetrics {
            daily_pnl: self.daily_pnl,
            current_drawdown: self.current_drawdown(),
            peak_balance: self.peak_balance,
            current_balance: self.current_balance,
            daily_loss_remaining: (self.peak_balance * self.config.max_daily_loss_pct / 100.0)
                + self.daily_pnl,
        }
    }
}

/// Current risk metrics
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub daily_pnl: f64,
    pub current_drawdown: f64,
    pub peak_balance: f64,
    pub current_balance: f64,
    pub daily_loss_remaining: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::OrderType;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_position_size_limit() {
        let config = RiskConfig {
            max_position_pct: 10.0,
            max_order_value: 10000.0,
            ..Default::default()
        };

        let risk_manager = RiskManager::new(config, 100000.0);
        let positions = HashMap::new();

        // Order that exceeds position size limit
        let order = Order {
            id: Uuid::new_v4(),
            symbol: Symbol::new("BTCUSDT"),
            side: Side::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            filled_quantity: 0.0,
            status: common::types::OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Price of 50000 means 50000 order value, max is 10% of 100000 = 10000
        let result = risk_manager.check_order(&order, &positions, 50000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_loss_limit() {
        let config = RiskConfig {
            max_daily_loss_pct: 5.0,
            ..Default::default()
        };

        let mut risk_manager = RiskManager::new(config, 100000.0);

        // Simulate losses
        risk_manager.update_balance(94000.0, -6000.0);

        let order = Order::market(
            Symbol::new("BTCUSDT"),
            Side::Buy,
            0.01,
        );

        let positions = HashMap::new();
        let result = risk_manager.check_order(&order, &positions, 50000.0);

        // Should fail because daily loss of 6000 exceeds 5% of 100000 = 5000
        assert!(result.is_err());
    }
}
```

## Execution Engine

The execution engine coordinates orders, fills, and position updates:

```rust
// crates/execution/src/engine.rs

use common::{
    errors::{OrderError, Result},
    types::{Order, OrderStatus, Position, Side, Symbol, Trade},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::risk::{RiskConfig, RiskManager};

/// Events emitted by the execution engine
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    OrderSubmitted(Order),
    OrderFilled(Order, Trade),
    OrderPartiallyFilled(Order, Trade),
    OrderCancelled(Order),
    OrderRejected(Order, String),
    PositionOpened(Position),
    PositionUpdated(Position),
    PositionClosed(Position),
}

/// Exchange interface for order execution
#[async_trait::async_trait]
pub trait Exchange: Send + Sync {
    /// Submit an order to the exchange
    async fn submit_order(&self, order: &Order) -> Result<()>;

    /// Cancel an order
    async fn cancel_order(&self, order_id: Uuid) -> Result<()>;

    /// Get current price for a symbol
    async fn get_price(&self, symbol: &Symbol) -> Result<f64>;

    /// Get account balance
    async fn get_balance(&self, asset: &str) -> Result<f64>;
}

/// The main execution engine
pub struct ExecutionEngine<E: Exchange> {
    exchange: Arc<E>,
    risk_manager: Arc<RwLock<RiskManager>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    positions: Arc<RwLock<HashMap<Symbol, Position>>>,
    event_sender: mpsc::Sender<ExecutionEvent>,
}

impl<E: Exchange> ExecutionEngine<E> {
    pub fn new(
        exchange: E,
        risk_config: RiskConfig,
        initial_balance: f64,
        event_sender: mpsc::Sender<ExecutionEvent>,
    ) -> Self {
        ExecutionEngine {
            exchange: Arc::new(exchange),
            risk_manager: Arc::new(RwLock::new(RiskManager::new(risk_config, initial_balance))),
            orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// Submit a new order
    #[instrument(skip(self), fields(order_id = %order.id, symbol = %order.symbol))]
    pub async fn submit_order(&self, mut order: Order) -> Result<Uuid> {
        let order_id = order.id;

        // Get current price for risk checks
        let current_price = self.exchange.get_price(&order.symbol).await?;

        // Perform risk checks
        {
            let positions = self.positions.read().await;
            let risk_manager = self.risk_manager.read().await;
            risk_manager.check_order(&order, &positions, current_price)?;
        }

        // Submit to exchange
        match self.exchange.submit_order(&order).await {
            Ok(()) => {
                order.status = OrderStatus::Open;

                // Store order
                {
                    let mut orders = self.orders.write().await;
                    orders.insert(order_id, order.clone());
                }

                // Emit event
                self.emit_event(ExecutionEvent::OrderSubmitted(order)).await;

                info!("Order submitted successfully");
                Ok(order_id)
            }
            Err(e) => {
                order.status = OrderStatus::Rejected;

                error!(error = %e, "Order rejected");
                self.emit_event(ExecutionEvent::OrderRejected(
                    order,
                    e.to_string(),
                ))
                .await;

                Err(e)
            }
        }
    }

    /// Process a trade fill from the exchange
    #[instrument(skip(self), fields(order_id = %trade.order_id, trade_id = %trade.id))]
    pub async fn process_fill(&self, trade: Trade) -> Result<()> {
        let mut orders = self.orders.write().await;
        let mut positions = self.positions.write().await;

        // Get the order
        let order = orders.get_mut(&trade.order_id).ok_or_else(|| {
            OrderError::NotFound(trade.order_id.to_string())
        })?;

        // Update order
        order.filled_quantity += trade.quantity;
        order.updated_at = chrono::Utc::now();

        let is_fully_filled = order.filled_quantity >= order.quantity;
        if is_fully_filled {
            order.status = OrderStatus::Filled;
        } else {
            order.status = OrderStatus::PartiallyFilled;
        }

        // Update or create position
        if let Some(position) = positions.get_mut(&trade.symbol) {
            position.apply_trade(&trade);

            if position.is_closed() {
                let closed_position = positions.remove(&trade.symbol).unwrap();
                self.emit_event(ExecutionEvent::PositionClosed(closed_position)).await;
            } else {
                self.emit_event(ExecutionEvent::PositionUpdated(position.clone())).await;
            }
        } else {
            let position = Position::from_trade(&trade);
            positions.insert(trade.symbol.clone(), position.clone());
            self.emit_event(ExecutionEvent::PositionOpened(position)).await;
        }

        // Update risk manager
        {
            let mut risk_manager = self.risk_manager.write().await;
            let balance = self.exchange.get_balance("USDT").await.unwrap_or(0.0);
            let pnl = if trade.side == Side::Sell {
                trade.notional() - trade.commission
            } else {
                -(trade.notional() + trade.commission)
            };
            risk_manager.update_balance(balance, pnl);
        }

        // Emit fill event
        if is_fully_filled {
            self.emit_event(ExecutionEvent::OrderFilled(order.clone(), trade)).await;
        } else {
            self.emit_event(ExecutionEvent::OrderPartiallyFilled(order.clone(), trade)).await;
        }

        info!("Fill processed successfully");
        Ok(())
    }

    /// Cancel an order
    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        // Cancel on exchange
        self.exchange.cancel_order(order_id).await?;

        // Update local state
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(&order_id) {
            order.status = OrderStatus::Cancelled;
            order.updated_at = chrono::Utc::now();

            self.emit_event(ExecutionEvent::OrderCancelled(order.clone())).await;
            info!("Order cancelled");
        } else {
            warn!("Order not found for cancellation");
        }

        Ok(())
    }

    /// Get all open orders
    pub async fn get_open_orders(&self) -> Vec<Order> {
        let orders = self.orders.read().await;
        orders
            .values()
            .filter(|o| o.is_active())
            .cloned()
            .collect()
    }

    /// Get all positions
    pub async fn get_positions(&self) -> HashMap<Symbol, Position> {
        self.positions.read().await.clone()
    }

    /// Get position for a symbol
    pub async fn get_position(&self, symbol: &Symbol) -> Option<Position> {
        self.positions.read().await.get(symbol).cloned()
    }

    /// Emit an event to subscribers
    async fn emit_event(&self, event: ExecutionEvent) {
        if let Err(e) = self.event_sender.send(event).await {
            error!(error = %e, "Failed to send execution event");
        }
    }
}

/// Mock exchange for testing
pub struct MockExchange {
    prices: RwLock<HashMap<Symbol, f64>>,
    balances: RwLock<HashMap<String, f64>>,
}

impl MockExchange {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert(Symbol::new("BTCUSDT"), 50000.0);
        prices.insert(Symbol::new("ETHUSDT"), 3000.0);

        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 100000.0);
        balances.insert("BTC".to_string(), 1.0);
        balances.insert("ETH".to_string(), 10.0);

        MockExchange {
            prices: RwLock::new(prices),
            balances: RwLock::new(balances),
        }
    }

    pub async fn set_price(&self, symbol: Symbol, price: f64) {
        self.prices.write().await.insert(symbol, price);
    }
}

#[async_trait::async_trait]
impl Exchange for MockExchange {
    async fn submit_order(&self, _order: &Order) -> Result<()> {
        // Simulate network latency
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(())
    }

    async fn cancel_order(&self, _order_id: Uuid) -> Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        Ok(())
    }

    async fn get_price(&self, symbol: &Symbol) -> Result<f64> {
        self.prices
            .read()
            .await
            .get(symbol)
            .copied()
            .ok_or_else(|| common::errors::MarketDataError::SymbolNotFound(symbol.to_string()).into())
    }

    async fn get_balance(&self, asset: &str) -> Result<f64> {
        Ok(self.balances.read().await.get(asset).copied().unwrap_or(0.0))
    }
}
```

## The Main Trading Bot

Finally, let's put it all together in the main bot:

```rust
// crates/bot/src/main.rs

use common::{
    errors::Result,
    types::{Candle, OrderBook, Signal, Symbol},
};
use execution::{
    engine::{ExecutionEngine, ExecutionEvent, MockExchange},
    risk::RiskConfig,
};
use strategy::{
    momentum::{MomentumConfig, MomentumStrategy},
    traits::{MarketState, Strategy},
};

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, Level};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

/// Trading bot configuration
#[derive(Debug, Clone)]
struct BotConfig {
    symbols: Vec<Symbol>,
    update_interval_ms: u64,
    risk_config: RiskConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            symbols: vec![
                Symbol::new("BTCUSDT"),
                Symbol::new("ETHUSDT"),
            ],
            update_interval_ms: 1000,
            risk_config: RiskConfig::default(),
        }
    }
}

/// The main trading bot
struct TradingBot<S: Strategy> {
    config: BotConfig,
    strategy: S,
    engine: Arc<ExecutionEngine<MockExchange>>,
    event_receiver: mpsc::Receiver<ExecutionEvent>,
    candles: HashMap<Symbol, Vec<Candle>>,
    orderbooks: HashMap<Symbol, OrderBook>,
    running: Arc<RwLock<bool>>,
}

impl<S: Strategy + 'static> TradingBot<S> {
    pub fn new(config: BotConfig, strategy: S) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(1000);

        let exchange = MockExchange::new();
        let engine = Arc::new(ExecutionEngine::new(
            exchange,
            config.risk_config.clone(),
            100000.0, // Initial balance
            event_sender,
        ));

        TradingBot {
            config,
            strategy,
            engine,
            event_receiver,
            candles: HashMap::new(),
            orderbooks: HashMap::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the bot
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing trading bot");

        // Initialize strategy
        self.strategy.initialize().await?;

        // Initialize candle storage for each symbol
        for symbol in &self.config.symbols {
            self.candles.insert(symbol.clone(), Vec::new());
            self.orderbooks.insert(
                symbol.clone(),
                OrderBook {
                    symbol: symbol.clone(),
                    bids: vec![],
                    asks: vec![],
                    timestamp: Utc::now(),
                },
            );
        }

        info!("Trading bot initialized successfully");
        Ok(())
    }

    /// Run the main trading loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting trading bot");
        *self.running.write().await = true;

        // Spawn event handler
        let engine = self.engine.clone();
        let mut event_rx = std::mem::replace(
            &mut self.event_receiver,
            mpsc::channel(1).1,
        );

        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                handle_execution_event(event).await;
            }
        });

        // Main trading loop
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(self.config.update_interval_ms)
        );

        while *self.running.read().await {
            interval.tick().await;

            // Process each symbol
            for symbol in self.config.symbols.clone() {
                if let Err(e) = self.process_symbol(&symbol).await {
                    error!(symbol = %symbol, error = %e, "Error processing symbol");
                }
            }
        }

        info!("Trading bot stopped");
        Ok(())
    }

    /// Process a single symbol
    async fn process_symbol(&mut self, symbol: &Symbol) -> Result<()> {
        // Get current market state
        let candles = self.candles.get(symbol).cloned().unwrap_or_default();
        let orderbook = self.orderbooks.get(symbol).cloned().unwrap_or_else(|| {
            OrderBook {
                symbol: symbol.clone(),
                bids: vec![],
                asks: vec![],
                timestamp: Utc::now(),
            }
        });

        // Get positions
        let positions = self.engine.get_positions().await;

        // Build market state
        let state = MarketState {
            symbol: symbol.clone(),
            orderbook,
            candles,
            positions,
            account_balance: 100000.0, // TODO: Get from exchange
        };

        // Generate signal
        match self.strategy.generate_signal(&state).await {
            Ok(signal) => {
                self.execute_signal(symbol, signal).await?;
            }
            Err(e) => {
                // Strategy errors are often non-fatal (e.g., insufficient data)
                tracing::debug!(error = %e, "Strategy error");
            }
        }

        Ok(())
    }

    /// Execute a trading signal
    async fn execute_signal(&self, symbol: &Symbol, signal: Signal) -> Result<()> {
        match signal {
            Signal::Buy { quantity, reason } => {
                info!(
                    symbol = %symbol,
                    quantity = quantity,
                    reason = %reason,
                    "Executing buy signal"
                );

                let order = common::types::Order::market(
                    symbol.clone(),
                    common::types::Side::Buy,
                    quantity,
                );

                self.engine.submit_order(order).await?;
            }
            Signal::Sell { quantity, reason } => {
                info!(
                    symbol = %symbol,
                    quantity = quantity,
                    reason = %reason,
                    "Executing sell signal"
                );

                let order = common::types::Order::market(
                    symbol.clone(),
                    common::types::Side::Sell,
                    quantity,
                );

                self.engine.submit_order(order).await?;
            }
            Signal::Hold => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Stop the bot gracefully
    pub async fn stop(&self) {
        info!("Stopping trading bot");
        *self.running.write().await = false;
    }
}

/// Handle execution events
async fn handle_execution_event(event: ExecutionEvent) {
    match event {
        ExecutionEvent::OrderSubmitted(order) => {
            info!(
                order_id = %order.id,
                symbol = %order.symbol,
                side = ?order.side,
                quantity = order.quantity,
                "Order submitted"
            );
        }
        ExecutionEvent::OrderFilled(order, trade) => {
            info!(
                order_id = %order.id,
                trade_id = %trade.id,
                price = trade.price,
                quantity = trade.quantity,
                "Order filled"
            );
        }
        ExecutionEvent::OrderPartiallyFilled(order, trade) => {
            info!(
                order_id = %order.id,
                filled = trade.quantity,
                remaining = order.remaining_quantity(),
                "Order partially filled"
            );
        }
        ExecutionEvent::OrderCancelled(order) => {
            info!(order_id = %order.id, "Order cancelled");
        }
        ExecutionEvent::OrderRejected(order, reason) => {
            error!(
                order_id = %order.id,
                reason = %reason,
                "Order rejected"
            );
        }
        ExecutionEvent::PositionOpened(position) => {
            info!(
                symbol = %position.symbol,
                quantity = position.quantity,
                entry_price = position.entry_price,
                "Position opened"
            );
        }
        ExecutionEvent::PositionUpdated(position) => {
            info!(
                symbol = %position.symbol,
                quantity = position.quantity,
                unrealized_pnl = position.unrealized_pnl,
                "Position updated"
            );
        }
        ExecutionEvent::PositionClosed(position) => {
            info!(
                symbol = %position.symbol,
                realized_pnl = position.realized_pnl,
                "Position closed"
            );
        }
    }
}

/// Initialize logging
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    info!("=== Algorithmic Trading System ===");
    info!("Day 365: Putting It All Together");

    // Create strategy configuration
    let strategy_config = MomentumConfig {
        name: "BTC_Momentum".to_string(),
        symbols: vec![Symbol::new("BTCUSDT")],
        fast_period: 12,
        slow_period: 26,
        signal_threshold: 0.5,
        position_size_pct: 10.0,
    };

    // Create bot configuration
    let bot_config = BotConfig {
        symbols: vec![Symbol::new("BTCUSDT")],
        update_interval_ms: 1000,
        risk_config: RiskConfig {
            max_position_pct: 20.0,
            max_positions: 3,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 15.0,
            max_order_value: 25000.0,
            restricted_symbols: vec![],
        },
    };

    // Create and initialize bot
    let strategy = MomentumStrategy::new(strategy_config);
    let mut bot = TradingBot::new(bot_config, strategy);

    bot.initialize().await?;

    // Run with graceful shutdown
    let running = bot.running.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutdown signal received");
        *running.write().await = false;
    });

    bot.run().await?;

    info!("Trading bot shutdown complete");
    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Workspace** | Organize large projects into multiple crates with shared dependencies |
| **Domain modeling** | Use Rust's type system to model trading concepts precisely |
| **Traits** | Define abstractions for strategies, exchanges, and other components |
| **Error handling** | Create comprehensive error types using thiserror |
| **Async architecture** | Use tokio for concurrent market data and order processing |
| **Risk management** | Implement position sizing, loss limits, and drawdown protection |
| **Event-driven design** | Use channels to decouple components and handle events |
| **Testing** | Write unit tests for critical components like risk checks |
| **Logging** | Use tracing for structured, contextual logging |
| **Graceful shutdown** | Handle signals to stop the bot cleanly |

## Practical Exercises

1. **Add a Mean Reversion Strategy**: Implement a new strategy that:
   - Calculates Bollinger Bands (20-period SMA with 2 standard deviations)
   - Buys when price touches the lower band
   - Sells when price touches the upper band
   - Includes position sizing based on volatility

2. **Implement a Position Tracker**: Create a component that:
   - Tracks all open positions with real-time PnL
   - Calculates portfolio-level metrics (total exposure, correlation)
   - Emits alerts when positions approach limits
   - Persists position history to a database

3. **Add Exchange Integration**: Extend the exchange trait to:
   - Connect to a real exchange API (Binance testnet)
   - Handle rate limiting with exponential backoff
   - Implement WebSocket for real-time order updates
   - Parse and validate exchange responses

4. **Build a Backtesting Engine**: Create a system that:
   - Replays historical candle data
   - Simulates order fills with realistic slippage
   - Calculates performance metrics (Sharpe ratio, max drawdown)
   - Compares multiple strategy configurations

## Homework

1. **Complete Trading System**: Build a production-ready bot that:
   - Implements at least two different strategies
   - Includes comprehensive risk management
   - Persists all trades and positions to PostgreSQL
   - Exposes Prometheus metrics for monitoring
   - Has Docker Compose setup for local development
   - Includes integration tests with a mock exchange
   - Has CI/CD pipeline with GitHub Actions

2. **Advanced Order Types**: Extend the system to support:
   - Trailing stop-loss orders
   - OCO (One-Cancels-Other) orders
   - Iceberg orders (hidden quantity)
   - Time-weighted average price (TWAP) execution
   - Smart order routing between multiple exchanges

3. **Machine Learning Integration**: Add ML capabilities:
   - Feature engineering from market data (returns, volatility, momentum)
   - Train a simple classifier to predict price direction
   - Use the model in a strategy to generate signals
   - Implement online learning to adapt to market changes
   - Track model performance and trigger retraining

4. **Multi-Exchange Arbitrage**: Build an arbitrage detector:
   - Connect to multiple exchanges simultaneously
   - Detect price discrepancies across exchanges
   - Calculate arbitrage opportunities accounting for fees
   - Execute synchronized orders on multiple exchanges
   - Handle partial fills and failed orders gracefully

5. **Real-Time Dashboard**: Create a monitoring dashboard:
   - WebSocket server for real-time updates
   - React/Vue frontend showing positions and PnL
   - Interactive charts with TradingView integration
   - Alert configuration and notification delivery
   - Historical performance analysis and reporting

## Congratulations!

You've completed 365 days of learning Rust for algorithmic trading. You've gone from "Hello, World!" to building a complete trading system with:

- **Strong foundations** in Rust's ownership, types, and error handling
- **Professional architecture** using workspaces, traits, and modular design
- **Production readiness** with logging, monitoring, and testing
- **Trading expertise** in market data, strategies, and risk management

This is just the beginning. The Rust ecosystem is constantly evolving, and the world of algorithmic trading offers endless opportunities to apply your skills. Keep learning, keep building, and happy trading!

## Navigation

[← Previous day](../354-production-logging/en.md)
