# Day 84: Entry API — Update or Insert

## Trading Analogy

Imagine you're maintaining an order book. When a new order arrives:
- If the ticker **already exists** in the book — **update** the volume
- If the ticker **doesn't exist** — **add** a new entry

This is the classic "update or insert" (upsert) pattern. Rust provides a powerful **Entry API** for this, allowing efficient HashMap operations without double lookups.

## The Problem Without Entry API

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Bad approach — double lookup!
    let ticker = String::from("BTC");
    let amount = 0.5;

    if portfolio.contains_key(&ticker) {
        // First lookup: check existence
        let current = portfolio.get(&ticker).unwrap();
        portfolio.insert(ticker, current + amount);  // Second lookup: insert
    } else {
        portfolio.insert(ticker, amount);
    }

    println!("{:?}", portfolio);
}
```

**Problem:** We're looking up the key twice — inefficient!

## Entry API — The Elegant Solution

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Good approach — single lookup!
    portfolio
        .entry(String::from("BTC"))
        .and_modify(|qty| *qty += 0.5)
        .or_insert(0.5);

    portfolio
        .entry(String::from("BTC"))
        .and_modify(|qty| *qty += 0.3)
        .or_insert(0.3);

    println!("{:?}", portfolio);  // {"BTC": 0.8}
}
```

## Entry API Methods

### or_insert — Insert Default Value

```rust
use std::collections::HashMap;

fn main() {
    let mut prices: HashMap<String, f64> = HashMap::new();

    // If key doesn't exist — insert value
    prices.entry(String::from("BTC")).or_insert(42000.0);
    prices.entry(String::from("ETH")).or_insert(2800.0);

    // If key already exists — do nothing
    prices.entry(String::from("BTC")).or_insert(99999.0);  // Ignored!

    println!("BTC: ${}", prices.get("BTC").unwrap());  // 42000, not 99999
    println!("ETH: ${}", prices.get("ETH").unwrap());  // 2800
}
```

### or_insert_with — Lazy Initialization

```rust
use std::collections::HashMap;

fn main() {
    let mut order_books: HashMap<String, Vec<(f64, f64)>> = HashMap::new();

    // Create empty vector only if key doesn't exist
    // More efficient than always creating Vec::new()
    order_books
        .entry(String::from("BTC"))
        .or_insert_with(Vec::new)
        .push((42000.0, 1.5));

    order_books
        .entry(String::from("BTC"))
        .or_insert_with(Vec::new)
        .push((42100.0, 0.8));

    println!("BTC orders: {:?}", order_books.get("BTC"));
}
```

### or_default — Insert Default::default()

```rust
use std::collections::HashMap;

fn main() {
    let mut trade_counts: HashMap<String, i32> = HashMap::new();

    // i32::default() = 0
    *trade_counts.entry(String::from("BTC")).or_default() += 1;
    *trade_counts.entry(String::from("BTC")).or_default() += 1;
    *trade_counts.entry(String::from("ETH")).or_default() += 1;

    println!("BTC trades: {}", trade_counts.get("BTC").unwrap());  // 2
    println!("ETH trades: {}", trade_counts.get("ETH").unwrap());  // 1
}
```

### and_modify — Modify Existing Value

```rust
use std::collections::HashMap;

fn main() {
    let mut positions: HashMap<String, f64> = HashMap::new();
    positions.insert(String::from("BTC"), 1.0);

    // Double the position if it exists
    positions
        .entry(String::from("BTC"))
        .and_modify(|qty| *qty *= 2.0);

    // For ETH nothing happens — key doesn't exist
    positions
        .entry(String::from("ETH"))
        .and_modify(|qty| *qty *= 2.0);

    println!("{:?}", positions);  // {"BTC": 2.0}
}
```

### Combining and_modify + or_insert

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    let trades = vec![
        ("BTC", 0.5),
        ("ETH", 2.0),
        ("BTC", 0.3),
        ("BTC", -0.2),
        ("SOL", 10.0),
        ("ETH", 1.0),
    ];

    for (ticker, amount) in trades {
        portfolio
            .entry(String::from(ticker))
            .and_modify(|qty| *qty += amount)
            .or_insert(amount);
    }

    println!("Portfolio:");
    for (ticker, qty) in &portfolio {
        println!("  {}: {:.2}", ticker, qty);
    }
}
```

## Practical Example: Order Book Aggregator

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct OrderLevel {
    price: f64,
    total_quantity: f64,
    order_count: u32,
}

fn main() {
    let mut order_book: HashMap<String, Vec<OrderLevel>> = HashMap::new();

    // Simulating incoming orders
    let orders = vec![
        ("BTC", 42000.0, 1.5),
        ("BTC", 42000.0, 0.8),  // Same price level
        ("BTC", 42100.0, 2.0),
        ("ETH", 2800.0, 10.0),
        ("BTC", 42000.0, 0.5),  // Another at the same level
    ];

    for (ticker, price, quantity) in orders {
        let levels = order_book
            .entry(String::from(ticker))
            .or_insert_with(Vec::new);

        // Find existing price level
        if let Some(level) = levels.iter_mut().find(|l| l.price == price) {
            level.total_quantity += quantity;
            level.order_count += 1;
        } else {
            levels.push(OrderLevel {
                price,
                total_quantity: quantity,
                order_count: 1,
            });
        }
    }

    println!("Order Book:");
    for (ticker, levels) in &order_book {
        println!("\n{}:", ticker);
        for level in levels {
            println!(
                "  Price: ${:.2} | Qty: {:.2} | Orders: {}",
                level.price, level.total_quantity, level.order_count
            );
        }
    }
}
```

## Practical Example: Trade Statistics Counter

```rust
use std::collections::HashMap;

#[derive(Debug, Default)]
struct TradeStats {
    count: u32,
    total_volume: f64,
    total_pnl: f64,
    wins: u32,
    losses: u32,
}

fn main() {
    let trades = vec![
        ("BTC", 1000.0, 50.0),    // ticker, volume, pnl
        ("ETH", 500.0, -20.0),
        ("BTC", 1500.0, 75.0),
        ("SOL", 200.0, 10.0),
        ("BTC", 800.0, -30.0),
        ("ETH", 600.0, 40.0),
    ];

    let mut stats: HashMap<String, TradeStats> = HashMap::new();

    for (ticker, volume, pnl) in trades {
        let entry = stats.entry(String::from(ticker)).or_default();

        entry.count += 1;
        entry.total_volume += volume;
        entry.total_pnl += pnl;

        if pnl > 0.0 {
            entry.wins += 1;
        } else if pnl < 0.0 {
            entry.losses += 1;
        }
    }

    println!("╔════════════════════════════════════════════════════╗");
    println!("║              TRADE STATISTICS                      ║");
    println!("╠════════════════════════════════════════════════════╣");

    for (ticker, s) in &stats {
        let win_rate = if s.count > 0 {
            (s.wins as f64 / s.count as f64) * 100.0
        } else {
            0.0
        };

        println!("║ {} ", ticker);
        println!("║   Trades: {} | Volume: ${:.2}", s.count, s.total_volume);
        println!("║   PnL: ${:.2} | Win Rate: {:.1}%", s.total_pnl, win_rate);
        println!("╠════════════════════════════════════════════════════╣");
    }
    println!("╚════════════════════════════════════════════════════╝");
}
```

## Practical Example: Price Cache with Updates

```rust
use std::collections::HashMap;

struct PriceCache {
    prices: HashMap<String, f64>,
    update_count: HashMap<String, u32>,
}

impl PriceCache {
    fn new() -> Self {
        PriceCache {
            prices: HashMap::new(),
            update_count: HashMap::new(),
        }
    }

    fn update_price(&mut self, ticker: &str, price: f64) {
        self.prices
            .entry(String::from(ticker))
            .and_modify(|p| *p = price)
            .or_insert(price);

        *self.update_count
            .entry(String::from(ticker))
            .or_default() += 1;
    }

    fn get_price(&self, ticker: &str) -> Option<f64> {
        self.prices.get(ticker).copied()
    }

    fn get_update_count(&self, ticker: &str) -> u32 {
        *self.update_count.get(ticker).unwrap_or(&0)
    }
}

fn main() {
    let mut cache = PriceCache::new();

    // Simulating price updates
    let price_updates = vec![
        ("BTC", 42000.0),
        ("ETH", 2800.0),
        ("BTC", 42100.0),
        ("BTC", 42050.0),
        ("ETH", 2850.0),
    ];

    for (ticker, price) in price_updates {
        cache.update_price(ticker, price);
        println!(
            "{}: ${:.2} (updates: {})",
            ticker,
            cache.get_price(ticker).unwrap(),
            cache.get_update_count(ticker)
        );
    }
}
```

## Working with Entry Enum Directly

```rust
use std::collections::HashMap;
use std::collections::hash_map::Entry;

fn main() {
    let mut risk_limits: HashMap<String, f64> = HashMap::new();
    risk_limits.insert(String::from("BTC"), 10000.0);

    // Check Entry type
    match risk_limits.entry(String::from("BTC")) {
        Entry::Occupied(mut entry) => {
            println!("BTC limit exists: ${}", entry.get());
            entry.insert(15000.0);  // Update
            println!("Updated to: ${}", entry.get());
        }
        Entry::Vacant(entry) => {
            println!("BTC limit not set, creating...");
            entry.insert(5000.0);
        }
    }

    match risk_limits.entry(String::from("ETH")) {
        Entry::Occupied(entry) => {
            println!("ETH limit: ${}", entry.get());
        }
        Entry::Vacant(entry) => {
            println!("ETH limit not set, creating default...");
            entry.insert(5000.0);
        }
    }

    println!("\nFinal limits: {:?}", risk_limits);
}
```

## What We Learned

| Method | Description | When to Use |
|--------|-------------|-------------|
| `or_insert(v)` | Insert `v` if key absent | Known default value |
| `or_insert_with(f)` | Call `f()` if key absent | Expensive initialization |
| `or_default()` | Insert `Default::default()` | Types with Default (numbers, Vec, String) |
| `and_modify(f)` | Modify value if key exists | Updating existing entries |
| `Entry::Occupied` | Key exists | Full control over entry |
| `Entry::Vacant` | Key absent | Full control over insertion |

## Homework

1. Write a function `aggregate_trades(trades: Vec<(String, f64, f64)>) -> HashMap<String, TradeAggregate>` that aggregates trades by ticker using Entry API

2. Create a `PositionTracker` struct that:
   - Tracks positions by ticker
   - Updates average entry price when adding
   - Uses Entry API for efficient operations

3. Implement a `WordCounter` for counting word frequency in trading logs using `or_default()`

4. Write a function that groups orders by price levels and calculates total volume at each level

## Navigation

[← Previous day](../083-hashmap-methods-insert-get-remove/en.md) | [Next day →](../085-hashset-unique-tickers/en.md)
