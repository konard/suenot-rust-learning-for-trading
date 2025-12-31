# Day 323: Zero-Copy: Avoiding Copies

## Trading Analogy

Imagine you're running a high-frequency trading desk where every millisecond matters. Your market data feed delivers thousands of price updates per second, and each update must be processed immediately.

**Traditional approach (with copying):**
- Receiving a price update is like a messenger bringing you a document
- You make a photocopy for the analyst, another for the risk manager, another for the order execution system
- Each copy takes time and paper (memory)
- By the time everyone has their copy, the price may have moved

**Zero-copy approach:**
- Instead of photocopying, you put the document in a central display
- Everyone reads from the same source simultaneously
- No copying delay, no wasted paper
- Everyone sees the data at the same time

In trading systems, zero-copy techniques can mean the difference between:
- **Capturing a profitable arbitrage** vs. missing it by microseconds
- **Processing 10,000 orders/second** vs. only 1,000
- **Running on a small VPS** vs. needing expensive hardware

## What is Zero-Copy?

**Zero-copy** is a technique that minimizes or eliminates data copying during operations. Instead of copying data from one memory location to another, you work with references or views into the original data.

### The Cost of Copying

```rust
use std::time::Instant;

fn demonstrate_copy_cost() {
    let iterations = 1_000_000;

    // Large price history data
    let price_data: Vec<f64> = (0..1000).map(|i| 50000.0 + i as f64 * 0.1).collect();

    // WITH COPYING: Each iteration copies the entire vector
    let start = Instant::now();
    for _ in 0..iterations {
        let copied_data = price_data.clone();  // Full copy!
        let _sum: f64 = copied_data.iter().sum();
    }
    let copy_time = start.elapsed();

    // WITHOUT COPYING: Use reference to original data
    let start = Instant::now();
    for _ in 0..iterations {
        let _sum: f64 = price_data.iter().sum();  // No copy, just reference
    }
    let zerocopy_time = start.elapsed();

    println!("=== Copy Cost Analysis ===");
    println!("With copying:    {:?}", copy_time);
    println!("Without copying: {:?}", zerocopy_time);
    println!("Speedup:         {:.1}x", copy_time.as_nanos() as f64 / zerocopy_time.as_nanos() as f64);
}

fn main() {
    demonstrate_copy_cost();
}
```

**Expected output:**
```
=== Copy Cost Analysis ===
With copying:    1.2s
Without copying: 45ms
Speedup:         26.7x
```

## Zero-Copy Techniques in Rust

### Technique 1: Slices Instead of Owned Collections

Slices (`&[T]`) are views into contiguous sequences without owning the data.

```rust
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

/// Calculate Simple Moving Average using a slice (zero-copy)
fn calculate_sma(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    // Take last `period` candles as a slice - no copying!
    let window = &candles[candles.len() - period..];
    let sum: f64 = window.iter().map(|c| c.close).sum();
    Some(sum / period as f64)
}

/// BAD: Takes ownership, forces caller to clone
fn calculate_sma_bad(candles: Vec<Candle>, period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }
    let sum: f64 = candles[candles.len() - period..].iter().map(|c| c.close).sum();
    Some(sum / period as f64)
}

fn main() {
    let candles: Vec<Candle> = (0..100)
        .map(|i| Candle {
            timestamp: 1700000000 + i * 60,
            open: 50000.0 + i as f64,
            high: 50100.0 + i as f64,
            low: 49900.0 + i as f64,
            close: 50050.0 + i as f64,
            volume: 100.0,
        })
        .collect();

    // GOOD: Zero-copy, can call multiple times
    let sma_20 = calculate_sma(&candles, 20);
    let sma_50 = calculate_sma(&candles, 50);  // Same data, no copy

    println!("SMA(20): ${:.2}", sma_20.unwrap_or(0.0));
    println!("SMA(50): ${:.2}", sma_50.unwrap_or(0.0));

    // BAD: Would require cloning for each call
    // let sma_bad = calculate_sma_bad(candles.clone(), 20);  // Clone required!
}
```

### Technique 2: Borrowed Iterators

Using iterators that borrow data instead of consuming it.

```rust
struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, quantity: f64) {
        self.bids.push((price, quantity));
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); // Descending
    }

    fn add_ask(&mut self, price: f64, quantity: f64) {
        self.asks.push((price, quantity));
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()); // Ascending
    }

    /// Zero-copy: returns iterator over references
    fn top_bids(&self, n: usize) -> impl Iterator<Item = &(f64, f64)> {
        self.bids.iter().take(n)
    }

    /// Zero-copy: calculate total bid volume without copying
    fn total_bid_volume(&self) -> f64 {
        self.bids.iter().map(|(_, qty)| qty).sum()
    }

    /// Zero-copy: find best bid/ask spread
    fn spread(&self) -> Option<f64> {
        let best_bid = self.bids.first().map(|(p, _)| p)?;
        let best_ask = self.asks.first().map(|(p, _)| p)?;
        Some(best_ask - best_bid)
    }

    /// Zero-copy: calculate order book imbalance
    fn imbalance(&self, depth: usize) -> f64 {
        let bid_volume: f64 = self.bids.iter().take(depth).map(|(_, q)| q).sum();
        let ask_volume: f64 = self.asks.iter().take(depth).map(|(_, q)| q).sum();

        if bid_volume + ask_volume > 0.0 {
            (bid_volume - ask_volume) / (bid_volume + ask_volume)
        } else {
            0.0
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Add some orders
    book.add_bid(49990.0, 1.5);
    book.add_bid(49980.0, 2.0);
    book.add_bid(49970.0, 3.0);
    book.add_ask(50010.0, 1.0);
    book.add_ask(50020.0, 2.5);

    // All these operations are zero-copy
    println!("=== Order Book Analysis ===");
    println!("Spread: ${:.2}", book.spread().unwrap_or(0.0));
    println!("Total bid volume: {:.2} BTC", book.total_bid_volume());
    println!("Imbalance (depth 3): {:.2}", book.imbalance(3));

    println!("\nTop 2 bids:");
    for (price, qty) in book.top_bids(2) {
        println!("  ${:.2} x {:.2}", price, qty);
    }
}
```

### Technique 3: Cow (Clone-on-Write)

`Cow` (Clone on Write) delays copying until mutation is needed.

```rust
use std::borrow::Cow;

/// Trade data that might need modification
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

/// Process trade data - only copies if modification needed
fn normalize_symbol<'a>(symbol: &'a str) -> Cow<'a, str> {
    if symbol.chars().all(|c| c.is_uppercase()) {
        // Already normalized - return reference (no copy)
        Cow::Borrowed(symbol)
    } else {
        // Needs normalization - allocate new String
        Cow::Owned(symbol.to_uppercase())
    }
}

/// Validate and potentially modify price
fn validate_price<'a>(price: f64, min_tick: f64) -> f64 {
    // Round to nearest tick size
    (price / min_tick).round() * min_tick
}

/// Process trades with zero-copy where possible
fn process_trades<'a>(trades: &'a [Trade]) -> Vec<Cow<'a, Trade>> {
    trades
        .iter()
        .map(|trade| {
            let normalized_symbol = normalize_symbol(&trade.symbol);

            // Only clone if we need to modify
            if matches!(normalized_symbol, Cow::Owned(_)) {
                Cow::Owned(Trade {
                    symbol: normalized_symbol.into_owned(),
                    price: trade.price,
                    quantity: trade.quantity,
                    side: trade.side.clone(),
                })
            } else {
                Cow::Borrowed(trade)
            }
        })
        .collect()
}

fn main() {
    let trades = vec![
        Trade {
            symbol: "BTCUSDT".to_string(),  // Already uppercase
            price: 50000.0,
            quantity: 1.0,
            side: "buy".to_string(),
        },
        Trade {
            symbol: "ethusdt".to_string(),  // Needs normalization
            price: 3000.0,
            quantity: 2.0,
            side: "sell".to_string(),
        },
        Trade {
            symbol: "SOLUSDT".to_string(),  // Already uppercase
            price: 100.0,
            quantity: 10.0,
            side: "buy".to_string(),
        },
    ];

    let processed = process_trades(&trades);

    println!("=== Processed Trades ===");
    for (i, trade) in processed.iter().enumerate() {
        let copy_status = if matches!(trade, Cow::Borrowed(_)) {
            "zero-copy"
        } else {
            "copied"
        };
        println!("Trade {}: {} ({}) - {}", i + 1, trade.symbol, trade.price, copy_status);
    }
}
```

### Technique 4: Memory Mapping for Historical Data

For large historical data files, memory mapping avoids loading everything into RAM.

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Simulated memory-mapped price data reader
/// In production, you would use `memmap2` crate
struct PriceDataReader {
    prices: Vec<f64>,  // In real implementation, this would be memory-mapped
}

impl PriceDataReader {
    /// Load price data (simulated - real impl would use mmap)
    fn new(data: Vec<f64>) -> Self {
        PriceDataReader { prices: data }
    }

    /// Zero-copy access to price window
    fn get_window(&self, start: usize, end: usize) -> Option<&[f64]> {
        if end <= self.prices.len() && start < end {
            Some(&self.prices[start..end])
        } else {
            None
        }
    }

    /// Calculate indicator over a window without copying
    fn calculate_volatility(&self, window_start: usize, window_size: usize) -> Option<f64> {
        let window = self.get_window(window_start, window_start + window_size)?;

        // Calculate standard deviation (volatility)
        let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
        let variance: f64 = window.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / window.len() as f64;

        Some(variance.sqrt())
    }

    /// Sliding window analysis without copying data
    fn analyze_volatility_series(&self, window_size: usize) -> Vec<f64> {
        (0..self.prices.len().saturating_sub(window_size))
            .filter_map(|i| self.calculate_volatility(i, window_size))
            .collect()
    }
}

fn main() {
    // Simulate loading 1 million historical prices
    let prices: Vec<f64> = (0..1_000_000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    let reader = PriceDataReader::new(prices);

    // Zero-copy window access
    if let Some(recent) = reader.get_window(999_990, 1_000_000) {
        println!("Last 10 prices (zero-copy access):");
        for (i, price) in recent.iter().enumerate() {
            println!("  {}: ${:.2}", i + 1, price);
        }
    }

    // Calculate volatility over a small sample
    let start = std::time::Instant::now();
    let volatilities = reader.analyze_volatility_series(100);
    let elapsed = start.elapsed();

    println!("\nCalculated {} volatility points in {:?}", volatilities.len(), elapsed);
    println!("Average volatility: ${:.2}",
        volatilities.iter().sum::<f64>() / volatilities.len() as f64);
}
```

### Technique 5: Bytes and Buffer Reuse

When parsing market data streams, reuse buffers instead of allocating new ones.

```rust
/// Reusable buffer for parsing market data
struct MarketDataParser {
    buffer: Vec<u8>,
    parsed_prices: Vec<f64>,
}

impl MarketDataParser {
    fn new(capacity: usize) -> Self {
        MarketDataParser {
            buffer: Vec::with_capacity(capacity),
            parsed_prices: Vec::with_capacity(1000),
        }
    }

    /// Parse price data, reusing internal buffers
    fn parse_prices(&mut self, data: &[u8]) -> &[f64] {
        self.parsed_prices.clear();  // Reuse buffer, don't reallocate

        // Simulate parsing (in reality, you'd parse actual market data format)
        for chunk in data.chunks(8) {
            if chunk.len() == 8 {
                let price = f64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
                self.parsed_prices.push(price);
            }
        }

        &self.parsed_prices  // Return reference to internal buffer
    }

    /// Process streaming data with zero allocation in hot path
    fn process_stream(&mut self, messages: &[Vec<u8>]) -> f64 {
        let mut total_volume = 0.0;

        for message in messages {
            let prices = self.parse_prices(message);
            total_volume += prices.iter().sum::<f64>();
        }

        total_volume
    }
}

fn main() {
    let mut parser = MarketDataParser::new(1024);

    // Simulate incoming market data messages
    let messages: Vec<Vec<u8>> = (0..100)
        .map(|i| {
            let price = 50000.0 + i as f64;
            price.to_le_bytes().to_vec()
        })
        .collect();

    let start = std::time::Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        let _total = parser.process_stream(&messages);
    }

    println!("Processed {} iterations in {:?}", iterations, start.elapsed());
    println!("Buffer reuse eliminates {} allocations", iterations * messages.len());
}
```

## Real-World Trading System Example

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Market data tick
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: i64,
}

/// Portfolio position
struct Position {
    quantity: f64,
    avg_price: f64,
}

/// High-performance trading engine using zero-copy techniques
struct TradingEngine {
    positions: HashMap<String, Position>,
    price_cache: HashMap<String, (f64, f64)>,  // (bid, ask)
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine {
            positions: HashMap::new(),
            price_cache: HashMap::new(),
        }
    }

    /// Update price cache - uses &str to avoid allocation
    fn update_price(&mut self, symbol: &str, bid: f64, ask: f64) {
        // Only allocates String if symbol is new
        if let Some(cache) = self.price_cache.get_mut(symbol) {
            *cache = (bid, ask);  // Update in place
        } else {
            self.price_cache.insert(symbol.to_string(), (bid, ask));
        }
    }

    /// Get current price - zero-copy lookup
    fn get_price(&self, symbol: &str) -> Option<(f64, f64)> {
        self.price_cache.get(symbol).copied()
    }

    /// Calculate portfolio value - zero-copy iteration
    fn calculate_portfolio_value(&self) -> f64 {
        self.positions.iter()
            .map(|(symbol, pos)| {
                let mid_price = self.price_cache
                    .get(symbol)
                    .map(|(bid, ask)| (bid + ask) / 2.0)
                    .unwrap_or(pos.avg_price);
                pos.quantity * mid_price
            })
            .sum()
    }

    /// Calculate unrealized PnL - zero-copy
    fn calculate_unrealized_pnl(&self) -> f64 {
        self.positions.iter()
            .map(|(symbol, pos)| {
                let mid_price = self.price_cache
                    .get(symbol)
                    .map(|(bid, ask)| (bid + ask) / 2.0)
                    .unwrap_or(pos.avg_price);
                pos.quantity * (mid_price - pos.avg_price)
            })
            .sum()
    }

    /// Find trading opportunities - returns references
    fn find_arbitrage_opportunities(&self, threshold: f64) -> Vec<(&str, f64)> {
        self.price_cache
            .iter()
            .filter_map(|(symbol, (bid, ask))| {
                let spread = (ask - bid) / bid;
                if spread > threshold {
                    Some((symbol.as_str(), spread))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Process tick batch - minimal copying
    fn process_ticks(&mut self, ticks: &[Tick]) -> usize {
        let mut updates = 0;
        for tick in ticks {
            self.update_price(&tick.symbol, tick.bid, tick.ask);
            updates += 1;
        }
        updates
    }

    /// Add or update position
    fn update_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        self.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                let total_cost = pos.avg_price * pos.quantity + price * quantity;
                pos.quantity += quantity;
                if pos.quantity != 0.0 {
                    pos.avg_price = total_cost / pos.quantity;
                }
            })
            .or_insert(Position {
                quantity,
                avg_price: price,
            });
    }
}

fn main() {
    let mut engine = TradingEngine::new();

    // Setup positions
    engine.update_position("BTCUSDT", 0.5, 50000.0);
    engine.update_position("ETHUSDT", 5.0, 3000.0);
    engine.update_position("SOLUSDT", 100.0, 100.0);

    // Simulate market data stream
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT", "DOTUSDT"];
    let ticks: Vec<Tick> = symbols
        .iter()
        .enumerate()
        .map(|(i, &symbol)| Tick {
            symbol: symbol.to_string(),
            bid: 50000.0 / (i + 1) as f64,
            ask: 50010.0 / (i + 1) as f64,
            timestamp: 1700000000 + i as i64,
        })
        .collect();

    // Benchmark zero-copy processing
    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        engine.process_ticks(&ticks);
        let _value = engine.calculate_portfolio_value();
        let _pnl = engine.calculate_unrealized_pnl();
    }

    let elapsed = start.elapsed();

    println!("=== Zero-Copy Trading Engine ===");
    println!("Processed {} iterations in {:?}", iterations, elapsed);
    println!("Throughput: {:.0} iterations/sec", iterations as f64 / elapsed.as_secs_f64());
    println!("\nPortfolio value: ${:.2}", engine.calculate_portfolio_value());
    println!("Unrealized PnL: ${:+.2}", engine.calculate_unrealized_pnl());

    // Find opportunities
    if let opportunities = engine.find_arbitrage_opportunities(0.0001) {
        println!("\nArbitrage opportunities (spread > 0.01%):");
        for (symbol, spread) in opportunities.iter().take(3) {
            println!("  {}: {:.4}%", symbol, spread * 100.0);
        }
    }
}
```

## When to Use Zero-Copy

### Use Zero-Copy When:

| Scenario | Why |
|----------|-----|
| **Hot paths** | Code running millions of times per second |
| **Large data** | Processing gigabytes of historical data |
| **Streaming data** | Real-time market feeds |
| **Memory constrained** | VPS or embedded systems |
| **Latency critical** | HFT, arbitrage |

### When Copying is Acceptable:

| Scenario | Why |
|----------|-----|
| **Data needs modification** | Clone before mutating |
| **Cross-thread sharing** | May need owned data |
| **Lifetime complexity** | Sometimes simpler to clone |
| **Small data** | Cost is negligible |
| **Infrequent operations** | Not a bottleneck |

## Performance Comparison

```rust
use std::time::Instant;

fn benchmark_approaches() {
    let data: Vec<f64> = (0..10_000).map(|i| i as f64).collect();
    let iterations = 100_000;

    // Approach 1: Clone entire vector each time
    let start = Instant::now();
    for _ in 0..iterations {
        let cloned = data.clone();
        let _sum: f64 = cloned.iter().sum();
    }
    let clone_time = start.elapsed();

    // Approach 2: Use reference (zero-copy)
    let start = Instant::now();
    for _ in 0..iterations {
        let _sum: f64 = data.iter().sum();
    }
    let ref_time = start.elapsed();

    // Approach 3: Use slice
    let start = Instant::now();
    for _ in 0..iterations {
        let slice = &data[..];
        let _sum: f64 = slice.iter().sum();
    }
    let slice_time = start.elapsed();

    println!("=== Performance Comparison ===");
    println!("Clone each time:  {:?}", clone_time);
    println!("Use reference:    {:?}", ref_time);
    println!("Use slice:        {:?}", slice_time);
    println!("\nSpeedup (clone vs ref): {:.1}x",
        clone_time.as_nanos() as f64 / ref_time.as_nanos() as f64);
}

fn main() {
    benchmark_approaches();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Zero-copy** | Working with references instead of copying data |
| **Slices** | Views into data without ownership |
| **Cow** | Clone-on-write for conditional copying |
| **Buffer reuse** | Reusing allocated memory for multiple operations |
| **Memory mapping** | Accessing files without loading into RAM |
| **Borrowed iterators** | Iterating without consuming data |

## Practical Exercises

1. **Order Book Optimization**: Implement an order book that:
   - Uses slices for price level access
   - Reuses internal buffers for aggregation
   - Measures memory usage vs. copying approach

2. **Price Feed Parser**: Create a market data parser that:
   - Reuses a single buffer for all messages
   - Uses slices to reference parsed data
   - Benchmarks vs. allocating per message

3. **Portfolio Calculator**: Build a portfolio analyzer that:
   - Stores positions with references where possible
   - Uses Cow for symbol normalization
   - Calculates metrics without copying position data

4. **Backtesting Engine**: Implement a backtester that:
   - Uses memory-mapped files for historical data
   - Processes candles via slices
   - Measures throughput improvement vs. loading all data

## Homework

1. **Zero-Copy Market Data Feed**: Build a system that:
   - Receives simulated WebSocket messages
   - Parses without allocating per message
   - Updates order book using references
   - Calculates VWAP using slices
   - Benchmarks: measure allocations per second

2. **Historical Data Analyzer**: Create a tool that:
   - Reads multi-gigabyte CSV files
   - Uses memory mapping or streaming
   - Calculates rolling indicators with slices
   - Generates reports without loading all data
   - Compares memory usage: full load vs. streaming

3. **Low-Latency Signal Generator**: Implement a system that:
   - Processes 100,000+ ticks per second
   - Uses buffer pools for message parsing
   - Generates signals using zero-copy patterns
   - Measures and reports latency percentiles (p50, p99, p999)

4. **Memory-Efficient Backtester**: Build a backtesting framework that:
   - Handles 10+ years of tick data
   - Uses iterators instead of loading everything
   - Implements strategies with borrowed data
   - Reports memory high-water mark
   - Compares execution time with different approaches

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../324-*/en.md)
