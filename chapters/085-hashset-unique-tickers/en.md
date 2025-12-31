# Day 85: HashSet — Unique Tickers in Portfolio

## Trading Analogy

In a trader's portfolio, each asset appears **only once**:
- You can't add AAPL twice — it's either there or not
- The list of tracked tickers shouldn't have duplicates
- Unique exchanges where we trade

`HashSet` is a collection of **unique** values. Like a ticker watchlist: each ticker is either in the list or not.

## Creating a HashSet

```rust
use std::collections::HashSet;

fn main() {
    // Empty HashSet
    let mut watchlist: HashSet<String> = HashSet::new();

    // HashSet with initial values
    let exchanges: HashSet<&str> = HashSet::from(["Binance", "Coinbase", "Kraken"]);

    // Creating via collect()
    let tickers: HashSet<&str> = vec!["BTC", "ETH", "BTC", "SOL", "ETH"]
        .into_iter()
        .collect();

    println!("Watchlist: {:?}", watchlist);
    println!("Exchanges: {:?}", exchanges);
    println!("Unique tickers: {:?}", tickers);  // BTC and ETH appear only once!
}
```

## Adding Elements: insert()

```rust
use std::collections::HashSet;

fn main() {
    let mut portfolio_tickers: HashSet<String> = HashSet::new();

    // insert() returns bool: true if element was added
    let added = portfolio_tickers.insert("BTC".to_string());
    println!("BTC added: {}", added);  // true

    portfolio_tickers.insert("ETH".to_string());
    portfolio_tickers.insert("SOL".to_string());

    // Attempting to add a duplicate
    let duplicate = portfolio_tickers.insert("BTC".to_string());
    println!("BTC added again: {}", duplicate);  // false — already exists!

    println!("Portfolio: {:?}", portfolio_tickers);
    println!("Unique assets: {}", portfolio_tickers.len());  // 3
}
```

## Checking Presence: contains()

```rust
use std::collections::HashSet;

fn main() {
    let allowed_pairs: HashSet<&str> = HashSet::from([
        "BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT"
    ]);

    // Check allowed pairs for trading
    let pair_to_trade = "BTC/USDT";

    if allowed_pairs.contains(pair_to_trade) {
        println!("Trading {} is allowed", pair_to_trade);
    } else {
        println!("Trading {} is NOT allowed", pair_to_trade);
    }

    // Checking multiple pairs
    let orders = vec!["BTC/USDT", "DOGE/USDT", "ETH/USDT"];

    for pair in orders {
        if allowed_pairs.contains(pair) {
            println!("  [OK] {} - order accepted", pair);
        } else {
            println!("  [REJECT] {} - pair not in whitelist", pair);
        }
    }
}
```

## Removing Elements: remove()

```rust
use std::collections::HashSet;

fn main() {
    let mut active_positions: HashSet<String> = HashSet::from([
        "BTC".to_string(),
        "ETH".to_string(),
        "SOL".to_string(),
    ]);

    println!("Active positions: {:?}", active_positions);

    // Close ETH position
    let closed = active_positions.remove("ETH");
    println!("ETH position closed: {}", closed);  // true

    // Attempting to close non-existent position
    let not_found = active_positions.remove("DOGE");
    println!("DOGE position closed: {}", not_found);  // false

    println!("Remaining positions: {:?}", active_positions);
}
```

## Set Operations

HashSet supports mathematical set operations — very useful for portfolio analysis.

### Union

```rust
use std::collections::HashSet;

fn main() {
    // Assets on Binance
    let binance: HashSet<&str> = HashSet::from(["BTC", "ETH", "BNB", "SOL"]);

    // Assets on Coinbase
    let coinbase: HashSet<&str> = HashSet::from(["BTC", "ETH", "MATIC", "AVAX"]);

    // All unique assets we can trade
    let all_assets: HashSet<_> = binance.union(&coinbase).collect();

    println!("Binance: {:?}", binance);
    println!("Coinbase: {:?}", coinbase);
    println!("All tradeable assets: {:?}", all_assets);
}
```

### Intersection

```rust
use std::collections::HashSet;

fn main() {
    // Top 10 by volume on Binance
    let binance_top: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "BNB", "SOL", "XRP"
    ]);

    // Top 10 by volume on Coinbase
    let coinbase_top: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "MATIC", "SOL", "DOGE"
    ]);

    // Common leaders on both exchanges — potentially most liquid
    let common_leaders: HashSet<_> = binance_top.intersection(&coinbase_top).collect();

    println!("Common top assets: {:?}", common_leaders);
    // {"BTC", "ETH", "SOL"}
}
```

### Difference

```rust
use std::collections::HashSet;

fn main() {
    // Current portfolio
    let portfolio: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL", "AVAX"]);

    // Target portfolio
    let target: HashSet<&str> = HashSet::from(["BTC", "ETH", "MATIC", "DOT"]);

    // What to sell (in portfolio but not in target)
    let to_sell: HashSet<_> = portfolio.difference(&target).collect();

    // What to buy (not in portfolio but in target)
    let to_buy: HashSet<_> = target.difference(&portfolio).collect();

    println!("Current portfolio: {:?}", portfolio);
    println!("Target portfolio: {:?}", target);
    println!("Assets to SELL: {:?}", to_sell);   // {"SOL", "AVAX"}
    println!("Assets to BUY: {:?}", to_buy);     // {"MATIC", "DOT"}
}
```

### Symmetric Difference

```rust
use std::collections::HashSet;

fn main() {
    let yesterday: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL"]);
    let today: HashSet<&str> = HashSet::from(["BTC", "ETH", "AVAX"]);

    // Assets that changed (were yesterday but not today + are today but weren't yesterday)
    let changed: HashSet<_> = yesterday.symmetric_difference(&today).collect();

    println!("Portfolio changes: {:?}", changed);  // {"SOL", "AVAX"}
}
```

## Iterating Over HashSet

```rust
use std::collections::HashSet;

fn main() {
    let watchlist: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "SOL", "AVAX", "MATIC"
    ]);

    println!("=== Watchlist ===");
    for ticker in &watchlist {
        println!("  Monitoring: {}", ticker);
    }

    // Filtering
    let eth_based: Vec<_> = watchlist
        .iter()
        .filter(|t| **t == "ETH" || t.starts_with("ETH"))
        .collect();

    println!("ETH-related: {:?}", eth_based);
}
```

## Practical Example: Duplicate Order Filter

```rust
use std::collections::HashSet;

#[derive(Debug)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    quantity: f64,
}

fn main() {
    // Incoming order stream (may have duplicate ids due to retries)
    let incoming_orders = vec![
        Order { id: "ORD001".to_string(), symbol: "BTC".to_string(), side: "BUY".to_string(), quantity: 0.5 },
        Order { id: "ORD002".to_string(), symbol: "ETH".to_string(), side: "BUY".to_string(), quantity: 2.0 },
        Order { id: "ORD001".to_string(), symbol: "BTC".to_string(), side: "BUY".to_string(), quantity: 0.5 }, // Duplicate!
        Order { id: "ORD003".to_string(), symbol: "SOL".to_string(), side: "SELL".to_string(), quantity: 10.0 },
        Order { id: "ORD002".to_string(), symbol: "ETH".to_string(), side: "BUY".to_string(), quantity: 2.0 }, // Duplicate!
    ];

    let mut processed_ids: HashSet<String> = HashSet::new();
    let mut unique_orders: Vec<&Order> = Vec::new();

    for order in &incoming_orders {
        // If id hasn't been processed yet
        if processed_ids.insert(order.id.clone()) {
            unique_orders.push(order);
            println!("[PROCESS] Order {}: {} {} {}",
                order.id, order.side, order.quantity, order.symbol);
        } else {
            println!("[SKIP] Duplicate order {}", order.id);
        }
    }

    println!("\nTotal incoming: {}", incoming_orders.len());
    println!("Unique processed: {}", unique_orders.len());
}
```

## Practical Example: Ticker Activity Analysis

```rust
use std::collections::HashSet;

fn main() {
    // Trades for the week (symbols)
    let monday_trades = vec!["BTC", "ETH", "SOL", "BTC", "ETH"];
    let tuesday_trades = vec!["ETH", "AVAX", "ETH", "BTC"];
    let wednesday_trades = vec!["SOL", "MATIC", "DOT", "SOL"];

    // Unique tickers for each day
    let monday: HashSet<_> = monday_trades.into_iter().collect();
    let tuesday: HashSet<_> = tuesday_trades.into_iter().collect();
    let wednesday: HashSet<_> = wednesday_trades.into_iter().collect();

    println!("Monday tickers: {:?}", monday);
    println!("Tuesday tickers: {:?}", tuesday);
    println!("Wednesday tickers: {:?}", wednesday);

    // Tickers traded every day
    let all_days: HashSet<_> = monday
        .intersection(&tuesday)
        .cloned()
        .collect::<HashSet<_>>()
        .intersection(&wednesday)
        .cloned()
        .collect();

    println!("\nTraded every day: {:?}", all_days);

    // All unique tickers for the week
    let all_week: HashSet<_> = monday
        .union(&tuesday)
        .cloned()
        .collect::<HashSet<_>>()
        .union(&wednesday)
        .cloned()
        .collect();

    println!("All tickers this week: {:?}", all_week);
    println!("Total unique tickers: {}", all_week.len());
}
```

## Practical Example: Whitelist/Blacklist

```rust
use std::collections::HashSet;

struct TradingFilter {
    whitelist: HashSet<String>,
    blacklist: HashSet<String>,
}

impl TradingFilter {
    fn new() -> Self {
        TradingFilter {
            whitelist: HashSet::new(),
            blacklist: HashSet::new(),
        }
    }

    fn allow(&mut self, symbol: &str) {
        self.whitelist.insert(symbol.to_string());
        self.blacklist.remove(symbol);
    }

    fn block(&mut self, symbol: &str) {
        self.blacklist.insert(symbol.to_string());
        self.whitelist.remove(symbol);
    }

    fn can_trade(&self, symbol: &str) -> bool {
        // If whitelist exists — only from whitelist
        // If no whitelist — everything except blacklist
        if !self.whitelist.is_empty() {
            self.whitelist.contains(symbol)
        } else {
            !self.blacklist.contains(symbol)
        }
    }
}

fn main() {
    let mut filter = TradingFilter::new();

    // Allow only specific assets
    filter.allow("BTC");
    filter.allow("ETH");
    filter.allow("SOL");

    let orders = vec!["BTC", "DOGE", "ETH", "SHIB", "SOL"];

    println!("=== Trading Filter ===");
    for symbol in orders {
        if filter.can_trade(symbol) {
            println!("  [ALLOW] {} - order accepted", symbol);
        } else {
            println!("  [BLOCK] {} - not in whitelist", symbol);
        }
    }
}
```

## HashSet vs Vec: When to Use Which

```rust
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    // Create a large set of tickers
    let tickers: Vec<String> = (0..10000)
        .map(|i| format!("TICKER{}", i))
        .collect();

    let ticker_set: HashSet<String> = tickers.iter().cloned().collect();

    let search_for = "TICKER9999";

    // Search in Vec — O(n)
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = tickers.contains(&search_for.to_string());
    }
    let vec_time = start.elapsed();

    // Search in HashSet — O(1)
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = ticker_set.contains(search_for);
    }
    let set_time = start.elapsed();

    println!("Vec search (1000x): {:?}", vec_time);
    println!("HashSet search (1000x): {:?}", set_time);
    println!("HashSet is ~{:.0}x faster",
        vec_time.as_nanos() as f64 / set_time.as_nanos() as f64);
}
```

## Useful Methods

```rust
use std::collections::HashSet;

fn main() {
    let mut set: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL"]);

    // Size and empty check
    println!("Size: {}", set.len());         // 3
    println!("Is empty: {}", set.is_empty()); // false

    // Clear
    set.clear();
    println!("After clear: {:?}", set);       // {}

    // Create with capacity
    let big_set: HashSet<String> = HashSet::with_capacity(1000);
    println!("Capacity: {}", big_set.capacity());

    // Subset check
    let small: HashSet<i32> = HashSet::from([1, 2]);
    let large: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5]);

    println!("small is subset of large: {}", small.is_subset(&large));     // true
    println!("large is superset of small: {}", large.is_superset(&small)); // true
    println!("Are disjoint: {}", small.is_disjoint(&HashSet::from([6, 7]))); // true
}
```

## What We Learned

| Method | Description |
|--------|-------------|
| `HashSet::new()` | Creates an empty HashSet |
| `insert(value)` | Adds element, returns `true` if new |
| `remove(&value)` | Removes element |
| `contains(&value)` | Checks presence |
| `union(&other)` | Set union |
| `intersection(&other)` | Set intersection |
| `difference(&other)` | Set difference |
| `len()`, `is_empty()` | Size and empty check |

## Homework

1. Create a `find_common_assets` function that takes portfolios from multiple traders and returns assets that all of them hold

2. Implement a unique trading pairs tracking system: adding new pairs, checking existing ones, statistics

3. Write a portfolio rebalancing function: takes current and target portfolios, returns lists of assets to buy and sell

4. Create a "new assets" detector: compares yesterday's and today's lists of tradeable assets on an exchange, finds new listings

## Navigation

[← Previous day](../084-entry-api/en.md) | [Next day →](../086-btreemap-sorted-prices/en.md)
