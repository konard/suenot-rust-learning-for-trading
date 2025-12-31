# Day 82: HashMap — Portfolio: Asset → Quantity

## Trading Analogy

Imagine your trading portfolio. You have different assets:
- **BTC**: 0.5 units
- **ETH**: 10.0 units
- **USDT**: 50000.0 units

This is a classic example of a "key-value" structure:
- **Key**: asset name (ticker)
- **Value**: asset quantity

In Rust, `HashMap` is used for such tasks — a hash table that allows quick value lookup by key.

## What is HashMap?

`HashMap<K, V>` is a collection of key-value pairs:
- `K` — key type (must implement `Eq` and `Hash`)
- `V` — value type

**Advantages:**
- O(1) average lookup time
- Fast insertion and deletion
- Dynamic size

```rust
use std::collections::HashMap;

fn main() {
    // Create portfolio: asset -> quantity
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Add assets
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("USDT"), 50000.0);

    println!("Portfolio: {:?}", portfolio);
}
```

## Creating HashMap

### Empty HashMap

```rust
use std::collections::HashMap;

fn main() {
    // Explicit type annotation
    let portfolio: HashMap<String, f64> = HashMap::new();

    // Rust infers types from first insertion
    let mut balances = HashMap::new();
    balances.insert("BTC", 1.5);  // HashMap<&str, f64>

    println!("Portfolio: {:?}", portfolio);
    println!("Balances: {:?}", balances);
}
```

### HashMap with Initial Capacity

```rust
use std::collections::HashMap;

fn main() {
    // Pre-allocate space for 100 assets
    let mut large_portfolio: HashMap<String, f64> = HashMap::with_capacity(100);

    large_portfolio.insert(String::from("BTC"), 1.0);

    println!("Capacity: {}", large_portfolio.capacity());
}
```

### Creating from Iterator

```rust
use std::collections::HashMap;

fn main() {
    // From array of tuples
    let assets = [
        ("BTC", 0.5),
        ("ETH", 10.0),
        ("SOL", 100.0),
    ];

    let portfolio: HashMap<&str, f64> = assets.into_iter().collect();

    println!("Portfolio: {:?}", portfolio);

    // From two vectors using zip
    let tickers = vec!["DOGE", "ADA", "DOT"];
    let amounts = vec![10000.0, 500.0, 50.0];

    let portfolio2: HashMap<&str, f64> = tickers
        .into_iter()
        .zip(amounts.into_iter())
        .collect();

    println!("Portfolio 2: {:?}", portfolio2);
}
```

## Accessing Elements

### Getting Value by Key

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);

    // get() returns Option<&V>
    let btc_amount = portfolio.get("BTC");
    match btc_amount {
        Some(amount) => println!("BTC balance: {}", amount),
        None => println!("BTC not found in portfolio"),
    }

    // Using if let
    if let Some(amount) = portfolio.get("ETH") {
        println!("ETH balance: {}", amount);
    }

    // unwrap_or for default value
    let xrp_amount = portfolio.get("XRP").unwrap_or(&0.0);
    println!("XRP balance: {}", xrp_amount);

    // copied() to get value copy
    let eth_balance: f64 = portfolio.get("ETH").copied().unwrap_or(0.0);
    println!("ETH balance (copied): {}", eth_balance);
}
```

### Checking Key Existence

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);

    // contains_key checks for key presence
    if portfolio.contains_key("BTC") {
        println!("You have Bitcoin!");
    }

    if !portfolio.contains_key("DOGE") {
        println!("Dogecoin is not in portfolio");
    }
}
```

## Practical Example: Portfolio Tracker

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Initial positions
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("USDT"), 10000.0);

    // Asset prices (in USDT)
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2500.0);
    prices.insert(String::from("USDT"), 1.0);

    // Calculate total portfolio value
    let mut total_value = 0.0;

    println!("=== Portfolio Composition ===");
    for (asset, amount) in &portfolio {
        let price = prices.get(asset).unwrap_or(&0.0);
        let value = amount * price;
        total_value += value;

        println!("{}: {} × ${:.2} = ${:.2}", asset, amount, price, value);
    }

    println!("=============================");
    println!("Total value: ${:.2}", total_value);
}
```

## Modifying HashMap

### Updating Value

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    println!("Before: {:?}", portfolio);

    // insert overwrites existing value
    let old_value = portfolio.insert(String::from("BTC"), 1.0);
    println!("Old value: {:?}", old_value);  // Some(0.5)

    println!("After: {:?}", portfolio);
}
```

### Modification via get_mut

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    // Get mutable reference and increase
    if let Some(amount) = portfolio.get_mut("BTC") {
        *amount += 0.25;  // Bought 0.25 more BTC
    }

    println!("BTC after purchase: {:?}", portfolio.get("BTC"));
}
```

### Entry API — Conditional Insertion

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    // or_insert: insert only if key doesn't exist
    portfolio.entry(String::from("BTC")).or_insert(10.0);  // Won't change
    portfolio.entry(String::from("ETH")).or_insert(5.0);   // Will insert

    println!("Portfolio: {:?}", portfolio);

    // or_insert_with: lazy evaluation
    portfolio
        .entry(String::from("SOL"))
        .or_insert_with(|| {
            println!("Computing initial SOL value...");
            100.0
        });

    println!("Portfolio: {:?}", portfolio);
}
```

### Entry API — Update Based on Old Value

```rust
use std::collections::HashMap;

fn main() {
    let trades = vec![
        ("BTC", 0.1),
        ("ETH", 5.0),
        ("BTC", 0.2),
        ("BTC", -0.05),  // sell
        ("ETH", 2.5),
    ];

    let mut portfolio: HashMap<&str, f64> = HashMap::new();

    // Aggregate all trades
    for (asset, amount) in trades {
        portfolio
            .entry(asset)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
    }

    println!("Final portfolio: {:?}", portfolio);
    // BTC: 0.1 + 0.2 - 0.05 = 0.25
    // ETH: 5.0 + 2.5 = 7.5
}
```

## Iterating Over HashMap

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);
    portfolio.insert("SOL", 100.0);

    // Iterate over (key, value) pairs
    println!("=== All Assets ===");
    for (asset, amount) in &portfolio {
        println!("{}: {}", asset, amount);
    }

    // Keys only
    println!("\n=== Tickers ===");
    for asset in portfolio.keys() {
        println!("{}", asset);
    }

    // Values only
    println!("\n=== Quantities ===");
    for amount in portfolio.values() {
        println!("{}", amount);
    }

    // Mutable iteration over values
    println!("\n=== Doubling positions ===");
    for amount in portfolio.values_mut() {
        *amount *= 2.0;
    }
    println!("After doubling: {:?}", portfolio);
}
```

## Practical Example: Order Processing

```rust
use std::collections::HashMap;

fn main() {
    // Order book simulation: price -> volume
    let mut order_book: HashMap<String, f64> = HashMap::new();

    // Add buy orders
    let buy_orders = vec![
        ("41000.00", 0.5),
        ("40500.00", 1.2),
        ("40000.00", 2.0),
        ("41000.00", 0.3),  // Add to existing level
    ];

    for (price, volume) in buy_orders {
        order_book
            .entry(String::from(price))
            .and_modify(|v| *v += volume)
            .or_insert(volume);
    }

    println!("=== Order Book (BID) ===");
    for (price, volume) in &order_book {
        println!("${}: {} BTC", price, volume);
    }

    // Total bid volume
    let total_bid_volume: f64 = order_book.values().sum();
    println!("\nTotal BID volume: {} BTC", total_bid_volume);
}
```

## Practical Example: Counting Trades by Asset

```rust
use std::collections::HashMap;

fn main() {
    let trades = vec![
        "BTC", "ETH", "BTC", "SOL", "BTC", "ETH",
        "DOGE", "BTC", "SOL", "ETH", "BTC", "BTC",
    ];

    let mut trade_counts: HashMap<&str, u32> = HashMap::new();

    // Count trades per asset
    for trade in &trades {
        let count = trade_counts.entry(trade).or_insert(0);
        *count += 1;
    }

    println!("=== Trade Statistics ===");
    for (asset, count) in &trade_counts {
        println!("{}: {} trades", asset, count);
    }

    // Find most traded asset
    if let Some((top_asset, top_count)) = trade_counts.iter().max_by_key(|&(_, count)| count) {
        println!("\nMost active: {} ({} trades)", top_asset, top_count);
    }
}
```

## Practical Example: Currency Conversion

```rust
use std::collections::HashMap;

fn main() {
    // Exchange rates to USD
    let mut exchange_rates: HashMap<&str, f64> = HashMap::new();
    exchange_rates.insert("BTC", 42000.0);
    exchange_rates.insert("ETH", 2500.0);
    exchange_rates.insert("SOL", 100.0);
    exchange_rates.insert("USDT", 1.0);
    exchange_rates.insert("EUR", 1.08);

    // Portfolio in different currencies
    let mut portfolio: HashMap<&str, f64> = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);
    portfolio.insert("EUR", 5000.0);

    // Convert everything to USD
    let mut total_usd = 0.0;

    println!("=== Conversion to USD ===");
    for (currency, amount) in &portfolio {
        if let Some(rate) = exchange_rates.get(currency) {
            let usd_value = amount * rate;
            total_usd += usd_value;
            println!("{} {} × ${:.2} = ${:.2}", amount, currency, rate, usd_value);
        } else {
            println!("{} {}: rate not found!", amount, currency);
        }
    }

    println!("=========================");
    println!("Total in USD: ${:.2}", total_usd);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `HashMap<K, V>` | Collection of key-value pairs |
| `insert()` | Insert or update element |
| `get()` | Get value (returns Option) |
| `get_mut()` | Get mutable reference |
| `contains_key()` | Check if key exists |
| `entry().or_insert()` | Conditional insertion |
| `keys()`, `values()` | Iterators over keys/values |

## Exercises

1. **P&L Tracker**: Create a HashMap to track profit/loss per asset. Implement functions to add trades and display total P&L.

2. **Balances by Exchange**: Create a nested HashMap `HashMap<String, HashMap<String, f64>>` to store balances by exchange (exchange -> asset -> quantity).

3. **Top 3 Positions**: Write a function that takes a portfolio and prices, returns the 3 largest positions by value.

4. **Price History**: Create a HashMap to store the last 10 prices for each asset. Implement a function to calculate average price.

## Homework

1. Implement a simple order book with price levels and volumes:
   ```rust
   // Structure: HashMap<price, volume>
   // Functions: add_order, remove_order, get_best_bid, get_best_ask
   ```

2. Create a position tracking system:
   - Add/remove positions
   - Calculate average entry price
   - Calculate unrealized P&L

3. Write a currency converter with support for:
   - Direct conversion (BTC → USD)
   - Reverse conversion (USD → BTC)
   - Cross rates (ETH → BTC via USD)

## Navigation

[← Previous day](../081-iterating-vec-orders/en.md) | [Next day →](../083-hashmap-methods/en.md)
