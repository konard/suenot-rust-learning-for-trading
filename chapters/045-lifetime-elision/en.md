# Day 45: Lifetime Elision — When Rust Figures It Out

## Trading Analogy

Imagine an experienced trader who looks at an order and immediately understands which trade it belongs to — without needing to explicitly specify it every time. Rust works the same way: the compiler is smart enough to **automatically infer** reference lifetimes in most cases, without requiring explicit annotations.

This is called **lifetime elision** — a set of rules the compiler applies automatically.

## Why Do We Need Lifetime Elision?

Without elision, we'd have to write:

```rust
// Without elision — very verbose
fn get_ticker<'a>(trade: &'a Trade) -> &'a str {
    &trade.ticker
}

fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}
```

With elision, code becomes cleaner:

```rust
// With elision — compiler infers lifetimes
fn get_ticker(trade: &Trade) -> &str {
    &trade.ticker
}

// This works because Rust applies elision rules
```

## Three Rules of Lifetime Elision

The compiler applies three rules in order:

### Rule 1: Each Reference Parameter Gets Its Own Lifetime

```rust
// What we write:
fn analyze_price(price: &f64) -> bool { *price > 0.0 }

// What the compiler sees:
fn analyze_price<'a>(price: &'a f64) -> bool { *price > 0.0 }
```

```rust
// Two parameters — two different lifetimes
fn compare_prices(bid: &f64, ask: &f64) -> bool { bid < ask }

// Compiler sees:
fn compare_prices<'a, 'b>(bid: &'a f64, ask: &'b f64) -> bool { bid < ask }
```

### Rule 2: If Exactly One Input Lifetime — It's Assigned to All Output References

```rust
// One input parameter
fn get_ticker(trade: &Trade) -> &str {
    &trade.ticker
}

// Compiler infers:
fn get_ticker<'a>(trade: &'a Trade) -> &'a str {
    &trade.ticker
}
```

```rust
// Practical example with price data
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: String,
}

fn get_symbol(data: &PriceData) -> &str {
    &data.symbol
}

fn get_timestamp(data: &PriceData) -> &str {
    &data.timestamp
}

fn main() {
    let data = PriceData {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        timestamp: String::from("2024-01-15 10:30:00"),
    };

    println!("Symbol: {}", get_symbol(&data));
    println!("Time: {}", get_timestamp(&data));
}
```

### Rule 3: For Methods with &self — The self Lifetime Is Assigned to Output References

```rust
struct OrderBook {
    symbol: String,
    bids: Vec<f64>,
    asks: Vec<f64>,
}

impl OrderBook {
    // &self is present, so returned reference gets its lifetime
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn best_bid(&self) -> Option<&f64> {
        self.bids.first()
    }

    fn best_ask(&self) -> Option<&f64> {
        self.asks.first()
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}

fn main() {
    let book = OrderBook {
        symbol: String::from("ETH/USDT"),
        bids: vec![2000.0, 1999.5, 1999.0],
        asks: vec![2001.0, 2001.5, 2002.0],
    };

    println!("Order Book: {}", book.symbol());
    println!("Best Bid: {:?}", book.best_bid());
    println!("Best Ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
}
```

## When Elision Does NOT Work

Sometimes the compiler cannot infer lifetimes automatically:

### Multiple Input References and Returning a Reference

```rust
// Compilation error! Compiler doesn't know which lifetime to use
// fn get_better_price(bid: &f64, ask: &f64) -> &f64 {
//     if bid > ask { bid } else { ask }
// }

// Need explicit annotation
fn get_better_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42050.0;
    let better = get_better_price(&bid, &ask);
    println!("Better price: {}", better);
}
```

### Different Lifetimes for Different Parameters

```rust
// Return reference only to one of the parameters
fn get_longer_symbol<'a, 'b>(s1: &'a str, s2: &'b str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s1 } // Always return s1
}

// Or with same lifetime if we can return either
fn get_longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s2 }
}

fn main() {
    let btc = "BTC/USDT";
    let eth = "ETH";

    println!("Longer: {}", get_longer(btc, eth));
}
```

## Practical Trading Examples

### Portfolio Analyzer

```rust
struct Portfolio {
    name: String,
    assets: Vec<Asset>,
    base_currency: String,
}

struct Asset {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Portfolio {
    fn new(name: &str, base_currency: &str) -> Self {
        Portfolio {
            name: name.to_string(),
            assets: Vec::new(),
            base_currency: base_currency.to_string(),
        }
    }

    // Rule 3: lifetime from &self
    fn name(&self) -> &str {
        &self.name
    }

    fn base_currency(&self) -> &str {
        &self.base_currency
    }

    fn get_asset(&self, symbol: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.symbol == symbol)
    }

    fn largest_position(&self) -> Option<&Asset> {
        self.assets.iter()
            .max_by(|a, b| {
                let value_a = a.quantity * a.avg_price;
                let value_b = b.quantity * b.avg_price;
                value_a.partial_cmp(&value_b).unwrap()
            })
    }

    fn add_asset(&mut self, symbol: &str, quantity: f64, price: f64) {
        self.assets.push(Asset {
            symbol: symbol.to_string(),
            quantity,
            avg_price: price,
        });
    }

    fn total_value(&self) -> f64 {
        self.assets.iter()
            .map(|a| a.quantity * a.avg_price)
            .sum()
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading", "USDT");

    portfolio.add_asset("BTC", 0.5, 42000.0);
    portfolio.add_asset("ETH", 5.0, 2200.0);
    portfolio.add_asset("SOL", 100.0, 95.0);

    println!("Portfolio: {}", portfolio.name());
    println!("Base currency: {}", portfolio.base_currency());
    println!("Total value: ${:.2}", portfolio.total_value());

    if let Some(largest) = portfolio.largest_position() {
        println!("Largest position: {} ({} units)",
                 largest.symbol, largest.quantity);
    }

    if let Some(btc) = portfolio.get_asset("BTC") {
        println!("BTC position: {} @ ${:.2}", btc.quantity, btc.avg_price);
    }
}
```

### Trade Data Parser

```rust
struct TradeRecord<'a> {
    raw_data: &'a str,
}

impl<'a> TradeRecord<'a> {
    fn new(data: &'a str) -> Self {
        TradeRecord { raw_data: data }
    }

    // Elision works — one input lifetime from &self
    fn get_field(&self, index: usize) -> Option<&str> {
        self.raw_data.split(',').nth(index)
    }

    fn symbol(&self) -> Option<&str> {
        self.get_field(0)
    }

    fn side(&self) -> Option<&str> {
        self.get_field(1)
    }

    fn price(&self) -> Option<f64> {
        self.get_field(2)?.parse().ok()
    }

    fn quantity(&self) -> Option<f64> {
        self.get_field(3)?.parse().ok()
    }
}

fn main() {
    let csv_line = "BTC/USDT,BUY,42150.50,0.5";
    let record = TradeRecord::new(csv_line);

    println!("Symbol: {:?}", record.symbol());
    println!("Side: {:?}", record.side());
    println!("Price: {:?}", record.price());
    println!("Quantity: {:?}", record.quantity());
}
```

### Market Data Cache

```rust
use std::collections::HashMap;

struct MarketCache {
    prices: HashMap<String, f64>,
    volumes: HashMap<String, f64>,
}

impl MarketCache {
    fn new() -> Self {
        MarketCache {
            prices: HashMap::new(),
            volumes: HashMap::new(),
        }
    }

    fn update_price(&mut self, symbol: &str, price: f64) {
        self.prices.insert(symbol.to_string(), price);
    }

    fn update_volume(&mut self, symbol: &str, volume: f64) {
        self.volumes.insert(symbol.to_string(), volume);
    }

    // Elision: &self -> returned reference
    fn get_price(&self, symbol: &str) -> Option<&f64> {
        self.prices.get(symbol)
    }

    fn get_volume(&self, symbol: &str) -> Option<&f64> {
        self.volumes.get(symbol)
    }

    fn symbols(&self) -> Vec<&String> {
        self.prices.keys().collect()
    }
}

fn main() {
    let mut cache = MarketCache::new();

    cache.update_price("BTC/USDT", 42000.0);
    cache.update_price("ETH/USDT", 2200.0);
    cache.update_volume("BTC/USDT", 1500000.0);
    cache.update_volume("ETH/USDT", 800000.0);

    println!("Cached symbols: {:?}", cache.symbols());

    if let Some(btc_price) = cache.get_price("BTC/USDT") {
        println!("BTC price: ${}", btc_price);
    }

    if let Some(btc_vol) = cache.get_volume("BTC/USDT") {
        println!("BTC volume: ${}", btc_vol);
    }
}
```

## Elision in Closures

Closures also use elision:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // Elision in closure
    let above_threshold: Vec<&f64> = prices.iter()
        .filter(|p| **p > 42000.0)
        .collect();

    println!("Prices above 42000: {:?}", above_threshold);

    // More complex example
    let trades = vec![
        ("BTC", 0.5, 42000.0),
        ("ETH", 5.0, 2200.0),
        ("SOL", 100.0, 95.0),
    ];

    let large_positions: Vec<_> = trades.iter()
        .filter(|(_, qty, price)| qty * price > 5000.0)
        .collect();

    println!("Large positions: {:?}", large_positions);
}
```

## Lifetime Elision Cheat Sheet

| Situation | Elision Works? | Example |
|-----------|----------------|---------|
| One input, no output reference | Yes | `fn log(msg: &str)` |
| One input, output reference | Yes | `fn first(s: &str) -> &str` |
| Method with &self, output reference | Yes | `fn name(&self) -> &str` |
| Two inputs, output reference | No | `fn pick<'a>(a: &'a T, b: &'a T) -> &'a T` |
| No input references, output reference | No | Needs 'static or another source |

## What We Learned

1. **Lifetime elision** — automatic lifetime inference by the compiler
2. **Three rules**: each input gets its own lifetime → one input = one output → &self determines output
3. **When it doesn't work**: multiple input references with a reference return
4. **In practice**: most functions don't require explicit annotations

## Exercises

1. Determine where explicit lifetime annotations are needed:
```rust
fn get_symbol(trade: &Trade) -> &str { ... }
fn longer(a: &str, b: &str) -> &str { ... }
fn process(data: &Data) { ... }
fn best_of_two(x: &f64, y: &f64) -> &f64 { ... }
```

2. Fix the compilation error:
```rust
fn get_max_price(prices: &[f64], threshold: &f64) -> &f64 {
    prices.iter()
        .filter(|p| *p > threshold)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(threshold)
}
```

3. Write a `TradeAnalyzer` struct with methods that use elision.

## Homework

1. Create a `MarketDataProvider` struct with methods:
   - `get_last_price(&self, symbol: &str) -> Option<&f64>`
   - `get_symbol_info(&self, symbol: &str) -> Option<&SymbolInfo>`
   - `all_symbols(&self) -> Vec<&String>`

2. Implement functions for working with trade history:
   - `find_trade_by_id(trades: &[Trade], id: &str) -> Option<&Trade>` (elision works)
   - `find_best_trade<'a>(t1: &'a Trade, t2: &'a Trade) -> &'a Trade` (explicit annotation needed)

3. Create a configuration parser with methods that return references to internal strings.

4. Write unit tests verifying that reference lifetimes are correct.

## Navigation

[← Previous day](../044-lifetimes-and-functions/en.md) | [Next day →](../046-static-lifetime/en.md)
