# Day 244: OHLCV Candles: Basic Structure

## Trading Analogy

Imagine you're watching a busy trading floor. Every second, prices fluctuate — sometimes up, sometimes down. But how do you capture all this chaos in a meaningful way? You can't track every single tick, so traders use a brilliant abstraction: the **OHLCV candle**.

Think of an OHLCV candle as a "summary report" for a period of trading activity:
- **Open**: Where did we start? (The first trade of the period)
- **High**: How high did we reach? (The maximum price)
- **Low**: How low did we fall? (The minimum price)
- **Close**: Where did we end up? (The last trade of the period)
- **Volume**: How much activity was there? (Total traded quantity)

It's like a weather report for prices: instead of saying "the temperature was 20°C at 9:00, then 21°C at 9:01, then 20.5°C at 9:02...", you simply say "Temperature ranged from 18°C to 25°C, starting at 20°C and ending at 22°C."

In algorithmic trading, OHLCV candles are the fundamental building blocks for:
- Technical analysis and indicators (RSI, MACD, Bollinger Bands)
- Pattern recognition (engulfing patterns, doji, hammers)
- Backtesting trading strategies
- Building charts and visualizations

## What is an OHLCV Candle?

An OHLCV candle represents aggregated price data over a specific time interval. Common intervals include:
- **1 minute (1m)**: For scalping and high-frequency strategies
- **5 minutes (5m)**: Short-term trading
- **1 hour (1h)**: Intraday analysis
- **1 day (1D)**: Swing trading and daily analysis
- **1 week (1W)**: Long-term trends

```
        │
        │ ← High (highest price during period)
    ┌───┴───┐
    │       │
    │       │ ← Body (shows Open vs Close)
    │       │
    └───┬───┘
        │
        │ ← Low (lowest price during period)
        │

    Green candle: Close > Open (price went up)
    Red candle: Close < Open (price went down)
```

## Basic OHLCV Structure in Rust

Let's define the fundamental structure:

```rust
/// Represents a single OHLCV candlestick
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    /// Opening price of the period
    pub open: f64,
    /// Highest price during the period
    pub high: f64,
    /// Lowest price during the period
    pub low: f64,
    /// Closing price of the period
    pub close: f64,
    /// Trading volume during the period
    pub volume: f64,
    /// Timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
}

impl Candle {
    /// Creates a new candle with validation
    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: u64,
    ) -> Result<Self, String> {
        // Validate OHLCV data
        if high < low {
            return Err("High price cannot be less than low price".to_string());
        }
        if high < open || high < close {
            return Err("High must be >= both open and close".to_string());
        }
        if low > open || low > close {
            return Err("Low must be <= both open and close".to_string());
        }
        if volume < 0.0 {
            return Err("Volume cannot be negative".to_string());
        }

        Ok(Candle {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        })
    }

    /// Returns true if this is a bullish (green) candle
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Returns true if this is a bearish (red) candle
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Returns the body size (difference between open and close)
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Returns the full range (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Returns the upper wick/shadow size
    pub fn upper_wick(&self) -> f64 {
        self.high - self.close.max(self.open)
    }

    /// Returns the lower wick/shadow size
    pub fn lower_wick(&self) -> f64 {
        self.close.min(self.open) - self.low
    }
}

fn main() {
    // Create a bullish BTC/USDT candle
    let btc_candle = Candle::new(
        42000.0,  // Open
        42500.0,  // High
        41800.0,  // Low
        42300.0,  // Close
        150.5,    // Volume in BTC
        1703980800000, // Timestamp: Dec 31, 2024 00:00:00 UTC
    ).expect("Invalid candle data");

    println!("BTC/USDT Candle: {:?}", btc_candle);
    println!("Is bullish: {}", btc_candle.is_bullish());
    println!("Body size: ${:.2}", btc_candle.body_size());
    println!("Range: ${:.2}", btc_candle.range());
    println!("Upper wick: ${:.2}", btc_candle.upper_wick());
    println!("Lower wick: ${:.2}", btc_candle.lower_wick());
}
```

## Using Decimal for Financial Precision

For real trading applications, floating-point precision issues can be problematic. Consider using the `rust_decimal` crate:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq)]
pub struct PreciseCandle {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub timestamp: u64,
}

impl PreciseCandle {
    pub fn new(
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        timestamp: u64,
    ) -> Result<Self, String> {
        if high < low {
            return Err("High cannot be less than low".to_string());
        }
        if volume < Decimal::ZERO {
            return Err("Volume cannot be negative".to_string());
        }

        Ok(PreciseCandle {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        })
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn body_size(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Calculate the percentage change from open to close
    pub fn percent_change(&self) -> Decimal {
        if self.open == Decimal::ZERO {
            return Decimal::ZERO;
        }
        ((self.close - self.open) / self.open) * dec!(100)
    }
}

fn main() {
    let eth_candle = PreciseCandle::new(
        dec!(2250.50),
        dec!(2280.75),
        dec!(2240.25),
        dec!(2275.00),
        dec!(5000.123456),
        1703980800000,
    ).expect("Invalid candle");

    println!("ETH Candle: {:?}", eth_candle);
    println!("Percent change: {:.4}%", eth_candle.percent_change());
}
```

## Building Candles from Tick Data

In practice, you often receive individual trades (ticks) and need to aggregate them into candles:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct CandleBuilder {
    interval_ms: u64,        // Candle interval in milliseconds
    current_open: Option<f64>,
    current_high: f64,
    current_low: f64,
    current_close: f64,
    current_volume: f64,
    period_start: u64,
    completed_candles: VecDeque<Candle>,
}

impl CandleBuilder {
    /// Create a new candle builder for the specified interval
    pub fn new(interval_ms: u64) -> Self {
        CandleBuilder {
            interval_ms,
            current_open: None,
            current_high: f64::MIN,
            current_low: f64::MAX,
            current_close: 0.0,
            current_volume: 0.0,
            period_start: 0,
            completed_candles: VecDeque::new(),
        }
    }

    /// Add a trade and potentially complete a candle
    pub fn add_trade(&mut self, trade: &Trade) -> Option<Candle> {
        let trade_period = (trade.timestamp / self.interval_ms) * self.interval_ms;

        // Check if we need to close the current candle and start a new one
        if self.current_open.is_some() && trade_period > self.period_start {
            let completed = self.finalize_candle();
            self.start_new_candle(trade, trade_period);
            return completed;
        }

        // Start a new candle if none exists
        if self.current_open.is_none() {
            self.start_new_candle(trade, trade_period);
            return None;
        }

        // Update the current candle
        self.update_candle(trade);
        None
    }

    fn start_new_candle(&mut self, trade: &Trade, period_start: u64) {
        self.current_open = Some(trade.price);
        self.current_high = trade.price;
        self.current_low = trade.price;
        self.current_close = trade.price;
        self.current_volume = trade.quantity;
        self.period_start = period_start;
    }

    fn update_candle(&mut self, trade: &Trade) {
        self.current_high = self.current_high.max(trade.price);
        self.current_low = self.current_low.min(trade.price);
        self.current_close = trade.price;
        self.current_volume += trade.quantity;
    }

    fn finalize_candle(&mut self) -> Option<Candle> {
        if let Some(open) = self.current_open {
            let candle = Candle {
                open,
                high: self.current_high,
                low: self.current_low,
                close: self.current_close,
                volume: self.current_volume,
                timestamp: self.period_start,
            };
            self.current_open = None;
            return Some(candle);
        }
        None
    }
}

fn main() {
    // Create a 1-minute candle builder
    let mut builder = CandleBuilder::new(60_000); // 60 seconds in ms

    // Simulate incoming trades
    let trades = vec![
        Trade { price: 42000.0, quantity: 0.5, timestamp: 1703980800000 },
        Trade { price: 42050.0, quantity: 0.3, timestamp: 1703980815000 },
        Trade { price: 42100.0, quantity: 0.8, timestamp: 1703980830000 },
        Trade { price: 41950.0, quantity: 0.2, timestamp: 1703980845000 },
        Trade { price: 42020.0, quantity: 0.6, timestamp: 1703980858000 },
        // New minute starts
        Trade { price: 42030.0, quantity: 0.4, timestamp: 1703980860000 },
    ];

    println!("Processing trades...\n");

    for trade in &trades {
        println!("Trade: ${:.2} x {:.2} @ {}", trade.price, trade.quantity, trade.timestamp);

        if let Some(candle) = builder.add_trade(trade) {
            println!("\n=== Completed 1-minute candle ===");
            println!("  Open:   ${:.2}", candle.open);
            println!("  High:   ${:.2}", candle.high);
            println!("  Low:    ${:.2}", candle.low);
            println!("  Close:  ${:.2}", candle.close);
            println!("  Volume: {:.2}", candle.volume);
            println!("  Bullish: {}\n", candle.is_bullish());
        }
    }
}
```

## Practical Example: Simple Candle Analysis

Let's analyze a series of candles to identify patterns:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: u64,
}

impl Candle {
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Check if this candle is a doji (small body relative to range)
    pub fn is_doji(&self, threshold: f64) -> bool {
        if self.range() == 0.0 {
            return true;
        }
        self.body_size() / self.range() < threshold
    }

    /// Check if this is a hammer (long lower wick, small upper wick)
    pub fn is_hammer(&self, body_ratio: f64) -> bool {
        let body = self.body_size();
        let lower_wick = self.close.min(self.open) - self.low;
        let upper_wick = self.high - self.close.max(self.open);

        lower_wick > body * 2.0 && upper_wick < body * body_ratio
    }
}

/// Analyze a series of candles
pub struct CandleAnalyzer {
    candles: Vec<Candle>,
}

impl CandleAnalyzer {
    pub fn new(candles: Vec<Candle>) -> Self {
        CandleAnalyzer { candles }
    }

    /// Calculate Simple Moving Average of closing prices
    pub fn sma(&self, period: usize) -> Vec<f64> {
        if self.candles.len() < period {
            return vec![];
        }

        let mut result = Vec::new();
        for i in (period - 1)..self.candles.len() {
            let sum: f64 = self.candles[(i + 1 - period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            result.push(sum / period as f64);
        }
        result
    }

    /// Find bullish engulfing patterns
    pub fn find_bullish_engulfing(&self) -> Vec<usize> {
        let mut patterns = Vec::new();

        for i in 1..self.candles.len() {
            let prev = &self.candles[i - 1];
            let curr = &self.candles[i];

            // Bullish engulfing: previous bearish, current bullish,
            // current body completely covers previous body
            if prev.is_bearish() && curr.is_bullish()
                && curr.open < prev.close && curr.close > prev.open
            {
                patterns.push(i);
            }
        }
        patterns
    }

    /// Calculate average volume
    pub fn average_volume(&self) -> f64 {
        if self.candles.is_empty() {
            return 0.0;
        }
        self.candles.iter().map(|c| c.volume).sum::<f64>() / self.candles.len() as f64
    }

    /// Find high volume candles (above average * multiplier)
    pub fn high_volume_candles(&self, multiplier: f64) -> Vec<usize> {
        let avg = self.average_volume();
        self.candles
            .iter()
            .enumerate()
            .filter(|(_, c)| c.volume > avg * multiplier)
            .map(|(i, _)| i)
            .collect()
    }
}

fn main() {
    // Sample candle data (simulating BTC/USDT hourly candles)
    let candles = vec![
        Candle { open: 42000.0, high: 42200.0, low: 41800.0, close: 41900.0, volume: 100.0, timestamp: 0 },
        Candle { open: 41900.0, high: 42100.0, low: 41700.0, close: 41750.0, volume: 120.0, timestamp: 1 },
        Candle { open: 41750.0, high: 42300.0, low: 41600.0, close: 42250.0, volume: 200.0, timestamp: 2 },
        Candle { open: 42250.0, high: 42400.0, low: 42100.0, close: 42350.0, volume: 150.0, timestamp: 3 },
        Candle { open: 42350.0, high: 42600.0, low: 42300.0, close: 42550.0, volume: 180.0, timestamp: 4 },
    ];

    let analyzer = CandleAnalyzer::new(candles.clone());

    println!("=== Candle Analysis ===\n");

    // Individual candle analysis
    for (i, candle) in candles.iter().enumerate() {
        let trend = if candle.is_bullish() { "Bullish" } else { "Bearish" };
        let doji = if candle.is_doji(0.1) { " (Doji)" } else { "" };
        println!(
            "Candle {}: {} ${:.0} -> ${:.0} | Range: ${:.0} | Vol: {:.0}{}",
            i, trend, candle.open, candle.close, candle.range(), candle.volume, doji
        );
    }

    // SMA calculation
    println!("\n3-period SMA: {:?}", analyzer.sma(3));

    // Pattern detection
    let engulfing = analyzer.find_bullish_engulfing();
    println!("Bullish engulfing patterns at indices: {:?}", engulfing);

    // Volume analysis
    println!("Average volume: {:.2}", analyzer.average_volume());
    println!("High volume candles (>1.5x avg): {:?}", analyzer.high_volume_candles(1.5));
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| OHLCV | Open, High, Low, Close, Volume — the five key data points of a candle |
| Candlestick | A visual representation of price action over a time period |
| Bullish/Bearish | Indicates whether price went up (bullish) or down (bearish) |
| Body | The difference between open and close prices |
| Wicks/Shadows | The lines above and below the body showing price extremes |
| Tick Aggregation | Building candles from individual trades |
| Technical Indicators | Calculations derived from candle data (SMA, patterns) |

## Exercises

1. **Candle Validation**: Extend the `Candle::new()` function to also validate that prices are positive and that the timestamp is not zero.

2. **Candle Serialization**: Implement `serde::Serialize` and `serde::Deserialize` for the `Candle` struct to allow saving/loading candle data to/from JSON files.

3. **Timeframe Conversion**: Write a function that takes 1-minute candles and aggregates them into 5-minute candles. The function should properly calculate the new OHLCV values.

4. **Pattern Detection**: Implement detection for the "Three White Soldiers" pattern (three consecutive bullish candles with higher closes).

## Homework

1. **Complete Candle Builder**: Extend the `CandleBuilder` to handle gaps in data (missing trades) and emit empty candles when needed for continuous charts.

2. **Volume Profile**: Create a function that groups candles by price levels and calculates the total volume traded at each level. This is useful for identifying support/resistance zones.

3. **Candle Statistics**: Implement a `CandleStats` struct that calculates:
   - Average body size
   - Average range
   - Bullish/Bearish ratio
   - Most common price range

4. **Real-time Aggregation**: Modify the `CandleBuilder` to work with live WebSocket data. Simulate a stream of trades and display forming candles in real-time.

## Navigation

[← Previous day](../243-order-book-matching/en.md) | [Next day →](../245-ohlcv-timeframe-conversion/en.md)
