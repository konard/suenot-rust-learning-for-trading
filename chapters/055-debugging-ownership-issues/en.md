# Day 55: Debugging Ownership Issues

## Trading Analogy

Imagine you're a fund manager and discover an anomaly: the same stock is assigned to two different traders. Or worse — someone is trying to sell an asset that was already sold. To investigate, you need to: check operation logs, find the moment of error, understand who transferred rights and when. In Rust, the compiler is your auditor that catches such problems **before** they reach production.

## Common Ownership Errors and How to Read Them

### 1. Use after move

```rust
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = portfolio;  // Ownership moved

    // Error! portfolio no longer owns the data
    println!("Active portfolio: {:?}", portfolio);
}
```

**Compiler message:**
```
error[E0382]: borrow of moved value: `portfolio`
 --> src/main.rs:6:43
  |
2 |     let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
  |         --------- move occurs because `portfolio` has type `Vec<&str>`
3 |     let archived = portfolio;
  |                    --------- value moved here
...
6 |     println!("Active portfolio: {:?}", portfolio);
  |                                        ^^^^^^^^^ value borrowed here after move
```

**How to read it:**
- `move occurs because` — type doesn't implement Copy, so move happens
- `value moved here` — this is where the value was moved
- `value borrowed here after move` — attempting to use after move

**Solutions:**
```rust
// Solution 1: Clone — create a copy
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = portfolio.clone();  // Copy
    println!("Active: {:?}", portfolio);
    println!("Archived: {:?}", archived);
}

// Solution 2: Reference — look, don't own
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = &portfolio;  // Just a reference
    println!("Active: {:?}", portfolio);
    println!("Archived: {:?}", archived);
}
```

### 2. Borrowing after moving to a function

```rust
fn archive_portfolio(portfolio: Vec<String>) {
    println!("Archiving: {:?}", portfolio);
}

fn main() {
    let portfolio = vec!["BTC".to_string(), "ETH".to_string()];
    archive_portfolio(portfolio);  // Ownership transferred

    // Error! portfolio is no longer ours
    println!("Size: {}", portfolio.len());
}
```

**Solution: accept a reference instead of ownership**
```rust
fn archive_portfolio(portfolio: &Vec<String>) {
    println!("Archiving: {:?}", portfolio);
}

fn main() {
    let portfolio = vec!["BTC".to_string(), "ETH".to_string()];
    archive_portfolio(&portfolio);  // Pass reference
    println!("Size: {}", portfolio.len());  // Works!
}
```

### 3. Simultaneous mutable and immutable borrowing

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    let first = &prices[0];  // Immutable borrow
    prices.push(42200.0);    // Mutable borrow

    println!("First price: {}", first);  // Error!
}
```

**Compiler message:**
```
error[E0502]: cannot borrow `prices` as mutable because it is also borrowed as immutable
 --> src/main.rs:5:5
  |
4 |     let first = &prices[0];
  |                  ------ immutable borrow occurs here
5 |     prices.push(42200.0);
  |     ^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
6 |
7 |     println!("First price: {}", first);
  |                                 ----- immutable borrow later used here
```

**How to read it:**
- `cannot borrow as mutable because it is also borrowed as immutable` — can't mutate while immutable reference exists
- `immutable borrow occurs here` — immutable reference taken here
- `immutable borrow later used here` — and it's still used here

**Solution: separate in time**
```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // First use the reference
    let first = &prices[0];
    println!("First price: {}", first);
    // Reference no longer needed

    // Now we can mutate
    prices.push(42200.0);
    println!("Prices after addition: {:?}", prices);
}
```

### 4. Dangling reference

```rust
fn get_best_price() -> &f64 {
    let price = 42000.0;
    &price  // Error! price will be destroyed
}

fn main() {
    let best = get_best_price();
    println!("Best price: {}", best);
}
```

**Compiler message:**
```
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:24
  |
1 | fn get_best_price() -> &f64 {
  |                        ^ expected named lifetime parameter
```

**Solution: return ownership**
```rust
fn get_best_price() -> f64 {
    let price = 42000.0;
    price  // Return value, not reference
}

fn main() {
    let best = get_best_price();
    println!("Best price: {}", best);
}
```

## Debugging Tools

### 1. The cargo check command

Quick check without full compilation:

```bash
cargo check
```

Shows ownership errors without spending time on code generation.

### 2. Cargo clippy — extended analysis

```bash
cargo clippy
```

Clippy finds not only errors but also suboptimal patterns:

```rust
// Clippy will warn: unnecessary cloning
fn process_order(order: Order) {
    let backup = order.clone();  // Clippy: "unnecessary clone"
    // only using order, backup is unused
}
```

### 3. Type annotations for understanding

When you don't understand what's happening, add explicit types:

```rust
fn analyze_trades(trades: Vec<Trade>) -> AnalysisResult {
    let filtered: Vec<&Trade> = trades  // Explicit type helps understand
        .iter()                          // iter() gives &Trade
        .filter(|t| t.is_profitable())
        .collect();

    // Now visible: filtered contains references to trades
    // so trades must outlive filtered

    process(&filtered)
}
```

### 4. Ownership comments

```rust
struct TradingEngine {
    // Owns the list of orders
    orders: Vec<Order>,

    // Owns the configuration (cloned at creation)
    config: Config,

    // Does NOT own — just a reference to external logger
    // Lifetime: engine cannot outlive logger
    logger: &'static Logger,
}
```

## Practical Solution Patterns

### Pattern 1: Early reference release

```rust
fn update_portfolio(portfolio: &mut Portfolio, market_data: &MarketData) {
    // Problematic code:
    // let price = portfolio.get_price("BTC");  // &f64
    // portfolio.update("BTC", new_price);       // &mut — conflict!
    // println!("Was: {}", price);

    // Solution: copy the value
    let price = *portfolio.get_price("BTC");  // f64, not &f64
    portfolio.update("BTC", market_data.get("BTC"));
    println!("Was: {}", price);  // Works!
}
```

### Pattern 2: Indices instead of references

```rust
fn find_best_trade(trades: &mut Vec<Trade>) -> Option<usize> {
    // Instead of storing reference &Trade, store index
    let mut best_idx = None;
    let mut best_profit = 0.0;

    for (idx, trade) in trades.iter().enumerate() {
        if trade.profit > best_profit {
            best_profit = trade.profit;
            best_idx = Some(idx);
        }
    }

    // Now we can modify trades
    if let Some(idx) = best_idx {
        trades[idx].mark_as_best();
    }

    best_idx
}
```

### Pattern 3: Struct splitting

```rust
// Problem: want to read orders and write to stats simultaneously
struct TradingSystem {
    orders: Vec<Order>,
    stats: Statistics,
}

// Solution: split into parts
struct TradingSystem {
    orders: OrderBook,
    stats: Statistics,
}

impl TradingSystem {
    fn process(&mut self) {
        // Can borrow different fields independently
        let orders = &self.orders;
        let stats = &mut self.stats;

        for order in orders.iter() {
            stats.record(order);
        }
    }
}
```

### Pattern 4: Temporary variables

```rust
fn calculate_metrics(trades: &mut Vec<Trade>) {
    // Problem:
    // for trade in trades.iter() {
    //     trades.push(trade.generate_hedge());  // Error!
    // }

    // Solution: collect changes separately
    let hedges: Vec<Trade> = trades
        .iter()
        .map(|t| t.generate_hedge())
        .collect();

    trades.extend(hedges);
}
```

## Practical Exercises

### Exercise 1: Fix the ownership error

```rust
fn main() {
    let orders = vec!["BUY BTC", "SELL ETH", "BUY SOL"];
    process_orders(orders);
    println!("Processed {} orders", orders.len());  // Error!
}

fn process_orders(orders: Vec<&str>) {
    for order in orders {
        println!("Processing: {}", order);
    }
}
```

### Exercise 2: Fix the borrow conflict

```rust
fn main() {
    let mut balances = vec![1000.0, 2000.0, 500.0];
    let first = &balances[0];
    balances[0] = 1500.0;
    println!("First balance: {}", first);
}
```

### Exercise 3: Fix the dangling reference

```rust
fn get_ticker() -> &str {
    let ticker = String::from("BTC/USDT");
    &ticker
}
```

### Exercise 4: Complex case

```rust
struct Portfolio {
    assets: Vec<String>,
    total_value: f64,
}

impl Portfolio {
    fn get_asset(&self, idx: usize) -> &String {
        &self.assets[idx]
    }

    fn update_value(&mut self, new_value: f64) {
        self.total_value = new_value;
    }
}

fn main() {
    let mut portfolio = Portfolio {
        assets: vec!["BTC".to_string(), "ETH".to_string()],
        total_value: 50000.0,
    };

    let first_asset = portfolio.get_asset(0);
    portfolio.update_value(55000.0);
    println!("Asset: {}", first_asset);
}
```

## Homework

1. **Error Analyzer:** Create a function that takes a Rust compiler error code string (E0382, E0502, E0106) and returns an explanation of the problem and typical solution

2. **Safe Price Cache:** Implement a `PriceCache` struct that stores the last N prices and allows:
   - Adding new prices
   - Getting the average price
   - Getting the latest price
   Ensure all operations are ownership-safe

3. **Refactoring:** Take the following code with errors and fix it in three different ways:

```rust
fn analyze_and_update(data: Vec<f64>) -> Vec<f64> {
    let avg = data.iter().sum::<f64>() / data.len() as f64;
    let normalized = data;  // Problem here

    normalized.iter().map(|x| x - avg).collect()
}
```

4. **Documentation:** Add comments to your solution explaining why each change solves the ownership problem

## What We Learned

| Error | Code | Typical Cause | Solution |
|-------|------|---------------|----------|
| Use after move | E0382 | Value was moved | Clone or reference |
| Double borrow | E0502 | &mut + & simultaneously | Separate in time |
| Dangling ref | E0106 | Reference to local | Return ownership |
| Lifetime | E0597 | Reference outlives source | Extend source lifetime |

## Navigation

[← Previous day](../054-borrow-dont-own/en.md) | [Next day →](../056-rc-multiple-owners/en.md)
