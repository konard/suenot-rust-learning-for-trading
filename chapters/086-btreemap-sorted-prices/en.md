# Day 86: BTreeMap — Sorted Prices

## Trading Analogy

Imagine an **order book** on an exchange. Buy orders are sorted by descending price (best price on top), while sell orders are sorted by ascending price. The exchange must **instantly find** the best prices and maintain them in sorted order.

In Rust, **BTreeMap** is perfect for such tasks — a data structure that stores keys in sorted order and efficiently supports range-based operations on prices.

## What is BTreeMap?

`BTreeMap` is an associative array (dictionary) that:
- Stores key-value pairs
- **Automatically sorts** keys
- Allows range-based searches
- Quickly finds minimum and maximum

```rust
use std::collections::BTreeMap;

fn main() {
    // Order book: price -> volume
    let mut order_book: BTreeMap<u64, f64> = BTreeMap::new();

    // Add orders (price in cents for precision)
    order_book.insert(4200000, 1.5);  // $42,000.00 -> 1.5 BTC
    order_book.insert(4201000, 2.3);  // $42,010.00 -> 2.3 BTC
    order_book.insert(4199500, 0.8);  // $41,995.00 -> 0.8 BTC

    // Iteration is in sorted order!
    for (price, volume) in &order_book {
        println!("${:.2}: {} BTC", *price as f64 / 100.0, volume);
    }
}
```

## BTreeMap vs HashMap

| Feature | BTreeMap | HashMap |
|---------|----------|---------|
| Key order | Sorted | Arbitrary |
| Key lookup | O(log n) | O(1) |
| Insertion | O(log n) | O(1) |
| Range queries | Yes | No |
| Min/Max | O(log n) | O(n) |

**Use BTreeMap** when order matters or you need range queries.

## Basic Operations

### Creation and Insertion

```rust
use std::collections::BTreeMap;

fn main() {
    // Empty map
    let mut prices: BTreeMap<String, f64> = BTreeMap::new();

    // Insertion
    prices.insert("BTC".to_string(), 42000.0);
    prices.insert("ETH".to_string(), 2800.0);
    prices.insert("SOL".to_string(), 98.5);

    println!("{:?}", prices);
    // Prints in alphabetical order: {"BTC": 42000.0, "ETH": 2800.0, "SOL": 98.5}
}
```

### Accessing Elements

```rust
use std::collections::BTreeMap;

fn main() {
    let mut prices: BTreeMap<&str, f64> = BTreeMap::new();
    prices.insert("BTC", 42000.0);
    prices.insert("ETH", 2800.0);

    // get() returns Option<&V>
    if let Some(btc_price) = prices.get("BTC") {
        println!("BTC: ${}", btc_price);
    }

    // get_mut() for modification
    if let Some(eth_price) = prices.get_mut("ETH") {
        *eth_price = 2850.0;
    }

    // entry() API for conditional insertion
    prices.entry("XRP").or_insert(0.55);

    println!("{:?}", prices);
}
```

### First and Last Elements

```rust
use std::collections::BTreeMap;

fn main() {
    let mut bid_book: BTreeMap<u64, f64> = BTreeMap::new();

    // Buy orders (bids)
    bid_book.insert(4195000, 2.0);
    bid_book.insert(4198000, 1.5);
    bid_book.insert(4200000, 3.0);  // Best bid
    bid_book.insert(4190000, 5.0);

    // Best bid price — maximum
    if let Some((&best_price, &volume)) = bid_book.last_key_value() {
        println!("Best Bid: ${:.2} x {}", best_price as f64 / 100.0, volume);
    }

    // Worst bid price — minimum
    if let Some((&worst_price, &volume)) = bid_book.first_key_value() {
        println!("Worst Bid: ${:.2} x {}", worst_price as f64 / 100.0, volume);
    }
}
```

## Range Queries

The main advantage of `BTreeMap` — working with ranges:

```rust
use std::collections::BTreeMap;

fn main() {
    let mut price_history: BTreeMap<u64, f64> = BTreeMap::new();

    // Timestamps (unix timestamp) -> price
    price_history.insert(1700000000, 42000.0);
    price_history.insert(1700000060, 42050.0);
    price_history.insert(1700000120, 42100.0);
    price_history.insert(1700000180, 42080.0);
    price_history.insert(1700000240, 42150.0);
    price_history.insert(1700000300, 42200.0);

    // Get prices for the last 2 minutes
    let start = 1700000180;
    let end = 1700000300;

    println!("Prices from {} to {}:", start, end);
    for (ts, price) in price_history.range(start..=end) {
        println!("  {}: ${}", ts, price);
    }
}
```

### Range Types

```rust
use std::collections::BTreeMap;
use std::ops::Bound;

fn main() {
    let mut levels: BTreeMap<i32, &str> = BTreeMap::new();
    for i in 1..=10 {
        levels.insert(i, "level");
    }

    // range(start..end) — from start to end (excluding end)
    println!("1..5: {:?}", levels.range(1..5).collect::<Vec<_>>());

    // range(start..=end) — from start to end (inclusive)
    println!("1..=5: {:?}", levels.range(1..=5).collect::<Vec<_>>());

    // range(start..) — from start to the end
    println!("8..: {:?}", levels.range(8..).collect::<Vec<_>>());

    // range(..end) — from the beginning to end
    println!("..3: {:?}", levels.range(..3).collect::<Vec<_>>());

    // Using Bound for complex queries
    use std::ops::Bound::{Excluded, Included};
    println!("(3, 7]: {:?}",
        levels.range((Excluded(3), Included(7))).collect::<Vec<_>>());
}
```

## Practical Example: Order Book

```rust
use std::collections::BTreeMap;

struct OrderBook {
    // For bids, use negative prices for reverse sorting
    bids: BTreeMap<i64, f64>,  // -price -> volume (for descending sort)
    asks: BTreeMap<i64, f64>,  // price -> volume
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_bid(&mut self, price: i64, volume: f64) {
        *self.bids.entry(-price).or_insert(0.0) += volume;
    }

    fn add_ask(&mut self, price: i64, volume: f64) {
        *self.asks.entry(price).or_insert(0.0) += volume;
    }

    fn best_bid(&self) -> Option<(i64, f64)> {
        self.bids.first_key_value().map(|(&p, &v)| (-p, v))
    }

    fn best_ask(&self) -> Option<(i64, f64)> {
        self.asks.first_key_value().map(|(&p, &v)| (p, v))
    }

    fn spread(&self) -> Option<i64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }

    fn display_top(&self, depth: usize) {
        println!("╔══════════════════════════════════════╗");
        println!("║           ORDER BOOK                 ║");
        println!("╠══════════════════════════════════════╣");

        // Top asks (in reverse order)
        let asks: Vec<_> = self.asks.iter().take(depth).collect();
        for (&price, &volume) in asks.iter().rev() {
            println!("║  ASK: ${:>10.2} | {:>8.4} BTC     ║",
                price as f64 / 100.0, volume);
        }

        println!("╠══════════════════════════════════════╣");

        if let Some(spread) = self.spread() {
            println!("║  SPREAD: ${:.2}                      ║",
                spread as f64 / 100.0);
        }

        println!("╠══════════════════════════════════════╣");

        // Top bids
        for (&neg_price, &volume) in self.bids.iter().take(depth) {
            println!("║  BID: ${:>10.2} | {:>8.4} BTC     ║",
                -neg_price as f64 / 100.0, volume);
        }

        println!("╚══════════════════════════════════════╝");
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Add orders
    book.add_bid(4200000, 2.5);   // $42,000.00
    book.add_bid(4199500, 1.8);   // $41,995.00
    book.add_bid(4199000, 3.2);   // $41,990.00
    book.add_bid(4198000, 5.0);   // $41,980.00

    book.add_ask(4200500, 1.2);   // $42,005.00
    book.add_ask(4201000, 2.0);   // $42,010.00
    book.add_ask(4201500, 0.8);   // $42,015.00
    book.add_ask(4202000, 4.5);   // $42,020.00

    book.display_top(4);

    if let (Some((bid, _)), Some((ask, _))) = (book.best_bid(), book.best_ask()) {
        let mid = (bid + ask) as f64 / 2.0 / 100.0;
        println!("\nMid Price: ${:.2}", mid);
    }
}
```

## Practical Example: Price Levels

```rust
use std::collections::BTreeMap;

fn main() {
    // Store volumes at price levels
    let mut volume_profile: BTreeMap<u64, f64> = BTreeMap::new();

    // Add trades
    let trades = vec![
        (4200000u64, 1.5f64),
        (4200500, 2.0),
        (4200000, 0.8),
        (4199500, 1.2),
        (4200500, 3.0),
        (4201000, 0.5),
    ];

    for (price, volume) in trades {
        *volume_profile.entry(price).or_insert(0.0) += volume;
    }

    // Find Point of Control (level with maximum volume)
    let poc = volume_profile
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(&price, &vol)| (price, vol));

    println!("=== Volume Profile ===");
    for (price, volume) in &volume_profile {
        let bar = "█".repeat((volume * 10.0) as usize);
        println!("${:.2}: {:>5.2} {}",
            *price as f64 / 100.0, volume, bar);
    }

    if let Some((price, vol)) = poc {
        println!("\nPoint of Control: ${:.2} ({:.2} BTC)",
            price as f64 / 100.0, vol);
    }

    // Find Value Area (70% of volume around POC)
    let total_volume: f64 = volume_profile.values().sum();
    println!("Total Volume: {:.2} BTC", total_volume);
}
```

## Practical Example: Price History with Timestamps

```rust
use std::collections::BTreeMap;

struct PriceHistory {
    data: BTreeMap<u64, f64>,  // timestamp -> price
}

impl PriceHistory {
    fn new() -> Self {
        PriceHistory {
            data: BTreeMap::new(),
        }
    }

    fn add(&mut self, timestamp: u64, price: f64) {
        self.data.insert(timestamp, price);
    }

    fn get_range(&self, from: u64, to: u64) -> Vec<(u64, f64)> {
        self.data
            .range(from..=to)
            .map(|(&ts, &price)| (ts, price))
            .collect()
    }

    fn get_latest(&self, n: usize) -> Vec<(u64, f64)> {
        self.data
            .iter()
            .rev()
            .take(n)
            .map(|(&ts, &price)| (ts, price))
            .collect()
    }

    fn high_low(&self, from: u64, to: u64) -> Option<(f64, f64)> {
        let prices: Vec<f64> = self.data
            .range(from..=to)
            .map(|(_, &p)| p)
            .collect();

        if prices.is_empty() {
            return None;
        }

        let high = prices.iter().cloned().fold(f64::MIN, f64::max);
        let low = prices.iter().cloned().fold(f64::MAX, f64::min);

        Some((high, low))
    }

    fn price_at_or_before(&self, timestamp: u64) -> Option<f64> {
        self.data
            .range(..=timestamp)
            .last()
            .map(|(_, &price)| price)
    }
}

fn main() {
    let mut history = PriceHistory::new();

    // Add minute data
    let base_ts = 1700000000u64;
    let prices = [42000.0, 42050.0, 42100.0, 42080.0, 42150.0,
                  42200.0, 42180.0, 42250.0, 42300.0, 42280.0];

    for (i, price) in prices.iter().enumerate() {
        history.add(base_ts + (i as u64 * 60), *price);
    }

    // Last 5 prices
    println!("=== Last 5 prices ===");
    for (ts, price) in history.get_latest(5) {
        println!("  {}: ${:.2}", ts, price);
    }

    // High/Low for period
    let from = base_ts + 120;
    let to = base_ts + 420;
    if let Some((high, low)) = history.high_low(from, to) {
        println!("\nHigh/Low from {} to {}:", from, to);
        println!("  High: ${:.2}", high);
        println!("  Low: ${:.2}", low);
        println!("  Range: ${:.2}", high - low);
    }

    // Price at a specific time
    let query_ts = base_ts + 150;  // Between data points
    if let Some(price) = history.price_at_or_before(query_ts) {
        println!("\nPrice at or before {}: ${:.2}", query_ts, price);
    }
}
```

## Practical Example: Risk Management by Levels

```rust
use std::collections::BTreeMap;

fn main() {
    // Stop-loss levels for different position sizes
    let mut risk_levels: BTreeMap<f64, f64> = BTreeMap::new();

    // Price -> percentage of capital to exit
    risk_levels.insert(41000.0, 50.0);  // If drops to 41K - exit 50%
    risk_levels.insert(40000.0, 30.0);  // If to 40K - another 30%
    risk_levels.insert(39000.0, 20.0);  // Last 20%

    let current_price = 40500.0;

    println!("Current price: ${:.2}", current_price);
    println!("\nActive stop-loss levels:");

    // Find all levels below current price
    for (level, pct) in risk_levels.range(..current_price) {
        println!("  ${:.2}: exit {:.0}% of position", level, pct);
    }

    // Nearest stop-loss level
    if let Some((level, pct)) = risk_levels.range(..current_price).last() {
        let distance = current_price - level;
        let distance_pct = (distance / current_price) * 100.0;
        println!("\nNearest stop: ${:.2} ({:.2}% away)", level, distance_pct);
        println!("Will exit: {:.0}% of position", pct);
    }
}
```

## Removing Elements

```rust
use std::collections::BTreeMap;

fn main() {
    let mut orders: BTreeMap<u64, f64> = BTreeMap::new();
    orders.insert(100, 1.0);
    orders.insert(200, 2.0);
    orders.insert(300, 3.0);
    orders.insert(400, 4.0);
    orders.insert(500, 5.0);

    // Remove specific order
    if let Some(volume) = orders.remove(&300) {
        println!("Removed order at 300 with volume {}", volume);
    }

    // Remove first element
    if let Some((price, volume)) = orders.pop_first() {
        println!("Popped first: {} -> {}", price, volume);
    }

    // Remove last element
    if let Some((price, volume)) = orders.pop_last() {
        println!("Popped last: {} -> {}", price, volume);
    }

    println!("Remaining: {:?}", orders);
}
```

## Merging BTreeMaps

```rust
use std::collections::BTreeMap;

fn main() {
    // Two price data sources
    let mut exchange_a: BTreeMap<&str, f64> = BTreeMap::new();
    exchange_a.insert("BTC", 42000.0);
    exchange_a.insert("ETH", 2800.0);

    let mut exchange_b: BTreeMap<&str, f64> = BTreeMap::new();
    exchange_b.insert("ETH", 2810.0);  // Different price!
    exchange_b.insert("SOL", 98.0);

    // Merge (exchange_b overwrites exchange_a)
    for (symbol, price) in exchange_b {
        exchange_a.insert(symbol, price);
    }

    println!("Merged prices: {:?}", exchange_a);

    // Or with price averaging
    let mut prices_a: BTreeMap<&str, f64> = BTreeMap::new();
    prices_a.insert("BTC", 42000.0);
    prices_a.insert("ETH", 2800.0);

    let prices_b: BTreeMap<&str, f64> = BTreeMap::from([
        ("ETH", 2810.0),
        ("SOL", 98.0),
    ]);

    for (symbol, price_b) in &prices_b {
        prices_a
            .entry(symbol)
            .and_modify(|p| *p = (*p + price_b) / 2.0)
            .or_insert(*price_b);
    }

    println!("Averaged prices: {:?}", prices_a);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `BTreeMap::new()` | Create empty map |
| `insert(k, v)` | Insert key-value pair |
| `get(&k)` | Get value by key |
| `first_key_value()` | Minimum key and value |
| `last_key_value()` | Maximum key and value |
| `range(start..end)` | Iterate over range |
| `pop_first()` / `pop_last()` | Remove first/last |
| `entry().or_insert()` | Conditional insertion |

## Exercises

1. Create a `BTreeMap` to store closing prices by date. Implement a function to find the maximum price for a given period.

2. Implement a simple order book with `add_order`, `remove_order`, `get_best_bid`, `get_best_ask` functions.

3. Create a support/resistance level system. Find the nearest level above and below the current price.

4. Implement a historical data cache with automatic removal of old entries (keep only the last N minutes).

## Homework

1. **Order Matching Engine**: Implement simple order matching. When bid >= ask, a trade should occur.

2. **VWAP Calculator**: Using BTreeMap<timestamp, (price, volume)>, calculate Volume Weighted Average Price for a period.

3. **Support/Resistance Finder**: Based on price history, find levels where the price frequently reversed (levels with many touches).

4. **Multi-Exchange Aggregator**: Combine order books from multiple exchanges into one aggregated book.

## Navigation

[← Previous day](../085-hashset-unique-tickers/en.md) | [Next day →](../087-vecdeque-order-queue/en.md)
