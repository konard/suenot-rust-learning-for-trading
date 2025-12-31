# Day 245: Calculating Candles from Ticks

## Trading Analogy

Imagine you're standing on a trading floor watching a display where prices change every second. Each trade is a **tick**: price, volume, time. There can be thousands of such ticks per minute. Looking at a stream of raw data is impossible — your eyes would glaze over.

That's why traders invented **candlesticks**. A candle groups all ticks within a specific period (minute, hour, day) and shows four key values:
- **Open** (O) — price of the first trade in the period
- **High** (H) — maximum price during the period
- **Low** (L) — minimum price during the period
- **Close** (C) — price of the last trade in the period

Additionally, a candle may contain **Volume** — the total volume of all trades during the period.

It's like compressing a thousand-page book into a summary — we lose details, but gain a clear picture of price movement.

## Theory: Data Aggregation in Rust

Calculating candles from ticks is a classic example of **data aggregation**. We:
1. Group data by time intervals
2. Apply aggregating functions (min, max, first, last, sum)
3. Form a new data structure

In Rust, we use:
- **Structs** to represent ticks and candles
- **Iterators** to process the data stream
- **HashMap** or grouping for time-based aggregation
- **Chrono** for working with dates and times

## Basic Data Structures

```rust
use std::time::{SystemTime, UNIX_EPOCH};

/// Tick — one trade on the exchange
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,      // Trading symbol (BTC/USDT)
    price: f64,          // Trade price
    quantity: f64,       // Trade volume
    timestamp: u64,      // Unix timestamp in milliseconds
}

/// OHLCV candle
#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,           // Opening price
    high: f64,           // Maximum
    low: f64,            // Minimum
    close: f64,          // Closing price
    volume: f64,         // Total volume
    open_time: u64,      // Period start
    close_time: u64,     // Period end
    trade_count: u32,    // Number of trades
}

impl Candle {
    /// Creates a new candle from the first tick
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    /// Updates the candle with a new tick
    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    /// Checks if the tick belongs to this candle
    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}

fn main() {
    // Create test ticks
    let base_time: u64 = 1700000000000; // Base time in milliseconds

    let ticks = vec![
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 0.5, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42050.0, quantity: 0.3, timestamp: base_time + 1000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41980.0, quantity: 1.2, timestamp: base_time + 2000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42100.0, quantity: 0.8, timestamp: base_time + 3000 },
    ];

    // Create a 1-minute candle (60000 ms = 1 minute)
    let interval_ms: u64 = 60000;
    let mut candle = Candle::new(&ticks[0], interval_ms);

    for tick in ticks.iter().skip(1) {
        if candle.contains(tick) {
            candle.update(tick);
        }
    }

    println!("Candle: {:?}", candle);
    println!("\nOHLCV: O={} H={} L={} C={} V={}",
        candle.open, candle.high, candle.low, candle.close, candle.volume);
}
```

## Real-Time Candle Aggregator

```rust
use std::collections::HashMap;

/// Aggregator for building candles from a tick stream
struct CandleAggregator {
    interval_ms: u64,                        // Candle interval in milliseconds
    current_candles: HashMap<String, Candle>, // Current unclosed candles
    completed_candles: Vec<Candle>,          // Completed candles
}

impl CandleAggregator {
    fn new(interval_ms: u64) -> Self {
        CandleAggregator {
            interval_ms,
            current_candles: HashMap::new(),
            completed_candles: Vec::new(),
        }
    }

    /// Processes a new tick
    fn process_tick(&mut self, tick: &Tick) {
        let candle_key = tick.symbol.clone();

        match self.current_candles.get_mut(&candle_key) {
            Some(candle) => {
                if candle.contains(tick) {
                    // Tick belongs to the current candle
                    candle.update(tick);
                } else {
                    // Candle closed, save it
                    let completed = candle.clone();
                    self.completed_candles.push(completed);

                    // Create a new candle
                    *candle = Candle::new(tick, self.interval_ms);
                }
            }
            None => {
                // First tick for this symbol
                let candle = Candle::new(tick, self.interval_ms);
                self.current_candles.insert(candle_key, candle);
            }
        }
    }

    /// Returns the current unclosed candle for a symbol
    fn get_current_candle(&self, symbol: &str) -> Option<&Candle> {
        self.current_candles.get(symbol)
    }

    /// Returns all completed candles
    fn get_completed_candles(&self) -> &[Candle] {
        &self.completed_candles
    }
}

fn main() {
    let base_time: u64 = 1700000000000;

    // Simulate a tick stream over several minutes
    let ticks = vec![
        // First minute
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 0.5, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42100.0, quantity: 0.3, timestamp: base_time + 15000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41900.0, quantity: 1.0, timestamp: base_time + 30000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42050.0, quantity: 0.7, timestamp: base_time + 45000 },
        // Second minute
        Tick { symbol: "BTC/USDT".to_string(), price: 42080.0, quantity: 0.4, timestamp: base_time + 60000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42200.0, quantity: 0.6, timestamp: base_time + 75000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42150.0, quantity: 0.9, timestamp: base_time + 90000 },
        // Third minute
        Tick { symbol: "BTC/USDT".to_string(), price: 42300.0, quantity: 0.5, timestamp: base_time + 120000 },
    ];

    let mut aggregator = CandleAggregator::new(60000); // 1 minute

    for tick in &ticks {
        aggregator.process_tick(tick);
        println!("Processed tick: price={}, time={}",
            tick.price, tick.timestamp - base_time);
    }

    println!("\n--- Completed Candles ---");
    for (i, candle) in aggregator.get_completed_candles().iter().enumerate() {
        println!("Candle {}: O={} H={} L={} C={} V={:.1} Trades={}",
            i + 1, candle.open, candle.high, candle.low,
            candle.close, candle.volume, candle.trade_count);
    }

    if let Some(current) = aggregator.get_current_candle("BTC/USDT") {
        println!("\n--- Current Candle (unclosed) ---");
        println!("O={} H={} L={} C={} V={:.1} Trades={}",
            current.open, current.high, current.low,
            current.close, current.volume, current.trade_count);
    }
}

#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}

impl Candle {
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}
```

## Multiple Timeframes

In real trading, you often need candles of different timeframes simultaneously:

```rust
use std::collections::HashMap;

/// Multi-timeframe aggregator
struct MultiTimeframeAggregator {
    aggregators: HashMap<String, CandleAggregator>, // Key: "symbol:interval"
}

impl MultiTimeframeAggregator {
    fn new() -> Self {
        MultiTimeframeAggregator {
            aggregators: HashMap::new(),
        }
    }

    /// Adds a timeframe for tracking
    fn add_timeframe(&mut self, symbol: &str, interval_ms: u64) {
        let key = format!("{}:{}", symbol, interval_ms);
        self.aggregators.insert(key, CandleAggregator::new(interval_ms));
    }

    /// Processes a tick for all timeframes
    fn process_tick(&mut self, tick: &Tick) {
        for (key, aggregator) in self.aggregators.iter_mut() {
            if key.starts_with(&tick.symbol) {
                aggregator.process_tick(tick);
            }
        }
    }

    /// Gets the current candle for a specific timeframe
    fn get_candle(&self, symbol: &str, interval_ms: u64) -> Option<&Candle> {
        let key = format!("{}:{}", symbol, interval_ms);
        self.aggregators.get(&key)?.get_current_candle(symbol)
    }
}

fn main() {
    let base_time: u64 = 1700000000000;

    let mut mtf = MultiTimeframeAggregator::new();

    // Add different timeframes for BTC
    mtf.add_timeframe("BTC/USDT", 60000);    // 1 minute
    mtf.add_timeframe("BTC/USDT", 300000);   // 5 minutes
    mtf.add_timeframe("BTC/USDT", 3600000);  // 1 hour

    // Generate ticks
    let ticks = vec![
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 1.0, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42150.0, quantity: 0.5, timestamp: base_time + 30000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41950.0, quantity: 0.8, timestamp: base_time + 60000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42200.0, quantity: 1.2, timestamp: base_time + 120000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42300.0, quantity: 0.6, timestamp: base_time + 300000 },
    ];

    for tick in &ticks {
        mtf.process_tick(tick);
    }

    println!("=== Multi-timeframe Analysis BTC/USDT ===\n");

    if let Some(candle) = mtf.get_candle("BTC/USDT", 60000) {
        println!("1 minute:  O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }

    if let Some(candle) = mtf.get_candle("BTC/USDT", 300000) {
        println!("5 minutes: O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }

    if let Some(candle) = mtf.get_candle("BTC/USDT", 3600000) {
        println!("1 hour:    O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }
}

// Structs repeated for compilation
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}

impl Candle {
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}

struct CandleAggregator {
    interval_ms: u64,
    current_candles: HashMap<String, Candle>,
    completed_candles: Vec<Candle>,
}

impl CandleAggregator {
    fn new(interval_ms: u64) -> Self {
        CandleAggregator {
            interval_ms,
            current_candles: HashMap::new(),
            completed_candles: Vec::new(),
        }
    }

    fn process_tick(&mut self, tick: &Tick) {
        let candle_key = tick.symbol.clone();
        match self.current_candles.get_mut(&candle_key) {
            Some(candle) => {
                if candle.contains(tick) {
                    candle.update(tick);
                } else {
                    let completed = candle.clone();
                    self.completed_candles.push(completed);
                    *candle = Candle::new(tick, self.interval_ms);
                }
            }
            None => {
                let candle = Candle::new(tick, self.interval_ms);
                self.current_candles.insert(candle_key, candle);
            }
        }
    }

    fn get_current_candle(&self, symbol: &str) -> Option<&Candle> {
        self.current_candles.get(symbol)
    }
}
```

## Calculating Indicators on Candles

After forming candles, you can calculate technical indicators:

```rust
/// Simple Moving Average (SMA)
fn calculate_sma(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    let sum: f64 = candles.iter()
        .rev()
        .take(period)
        .map(|c| c.close)
        .sum();

    Some(sum / period as f64)
}

/// Exponential Moving Average (EMA)
fn calculate_ema(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = candles[candles.len() - period].close;

    for candle in candles.iter().rev().take(period - 1) {
        ema = (candle.close - ema) * multiplier + ema;
    }

    Some(ema)
}

/// Trend detection using candles
fn detect_trend(candles: &[Candle], lookback: usize) -> String {
    if candles.len() < lookback {
        return "Insufficient data".to_string();
    }

    let recent: Vec<&Candle> = candles.iter().rev().take(lookback).collect();

    let bullish_count = recent.iter().filter(|c| c.close > c.open).count();
    let bearish_count = recent.iter().filter(|c| c.close < c.open).count();

    let avg_body_size: f64 = recent.iter()
        .map(|c| (c.close - c.open).abs())
        .sum::<f64>() / lookback as f64;

    if bullish_count > bearish_count * 2 {
        format!("Strong uptrend ({}% bullish candles)",
            bullish_count * 100 / lookback)
    } else if bearish_count > bullish_count * 2 {
        format!("Strong downtrend ({}% bearish candles)",
            bearish_count * 100 / lookback)
    } else {
        format!("Sideways movement (avg body size: {:.2})", avg_body_size)
    }
}

fn main() {
    // Create historical candles
    let candles = vec![
        Candle { symbol: "BTC/USDT".to_string(), open: 40000.0, high: 40500.0, low: 39800.0, close: 40300.0, volume: 100.0, open_time: 0, close_time: 0, trade_count: 50 },
        Candle { symbol: "BTC/USDT".to_string(), open: 40300.0, high: 41000.0, low: 40200.0, close: 40800.0, volume: 120.0, open_time: 0, close_time: 0, trade_count: 60 },
        Candle { symbol: "BTC/USDT".to_string(), open: 40800.0, high: 41500.0, low: 40700.0, close: 41200.0, volume: 150.0, open_time: 0, close_time: 0, trade_count: 70 },
        Candle { symbol: "BTC/USDT".to_string(), open: 41200.0, high: 42000.0, low: 41100.0, close: 41800.0, volume: 180.0, open_time: 0, close_time: 0, trade_count: 80 },
        Candle { symbol: "BTC/USDT".to_string(), open: 41800.0, high: 42500.0, low: 41600.0, close: 42200.0, volume: 200.0, open_time: 0, close_time: 0, trade_count: 90 },
    ];

    println!("=== BTC/USDT Candle Analysis ===\n");

    if let Some(sma) = calculate_sma(&candles, 3) {
        println!("SMA(3): {:.2}", sma);
    }

    if let Some(ema) = calculate_ema(&candles, 3) {
        println!("EMA(3): {:.2}", ema);
    }

    let trend = detect_trend(&candles, 5);
    println!("\nTrend: {}", trend);

    println!("\n--- Recent Candles ---");
    for (i, candle) in candles.iter().enumerate() {
        let direction = if candle.close > candle.open { "+" } else { "-" };
        println!("Candle {}: {} O={} C={} ({})",
            i + 1, direction, candle.open, candle.close,
            if candle.close > candle.open { "bullish" } else { "bearish" });
    }
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Tick | A single trade on the exchange with price, volume, and time |
| OHLCV Candle | Tick aggregation: Open, High, Low, Close, Volume |
| Candle Interval | Time period for grouping (1m, 5m, 1h, 1d) |
| Data Aggregation | Grouping and applying functions (min, max, sum) |
| Multi-timeframe | Simultaneously building candles of different intervals |
| Technical Indicators | SMA, EMA, and other calculations based on candles |

## Practice Exercises

1. **Simple Aggregator**: Implement a function `ticks_to_candle(ticks: &[Tick]) -> Candle` that converts a vector of ticks into a single candle. Test it with sample data.

2. **Tick Validation**: Add a method `is_valid()` to the `Tick` struct that checks:
   - Price > 0
   - Volume >= 0
   - Timestamp is within a reasonable range (not in the future)

3. **Volume-based Candles**: Create an aggregator that forms candles not by time, but by accumulated volume (e.g., every 100 BTC traded — new candle).

## Homework

1. **Anomaly Detector**: Write a function that analyzes a tick stream and identifies anomalous spikes:
   - Sharp price change (>1% in a single trade)
   - Abnormal volume (>10x the average)
   - Time gaps (>10 seconds without trades)

2. **VWAP Calculator**: Implement Volume Weighted Average Price (VWAP) calculation from ticks:
   ```
   VWAP = Sum(Price × Volume) / Sum(Volume)
   ```
   Track VWAP in real-time as new ticks arrive.

3. **Candlestick Pattern**: Implement recognition of the "Engulfing" pattern:
   - Bullish engulfing: small red candle, then large green candle
   - Bearish engulfing: small green candle, then large red candle

4. **Data Export**: Create a function to export candles to CSV format:
   ```
   timestamp,open,high,low,close,volume
   1700000000000,42000.0,42100.0,41900.0,42050.0,2.5
   ```

## Navigation

[← Previous day](../244-tick-data-processing/en.md) | [Next day →](../246-order-book-aggregation/en.md)
