# Day 348: Linters: clippy

## Trading Analogy

Imagine you're managing a trading desk, and you have an experienced senior trader who reviews all orders before they're sent to the exchange. They don't just check syntax (is the ticker correct?), but also look for patterns:

- "Are you sure you want to buy an asset at 5% above market price?"
- "This order duplicates the previous one — might be an error?"
- "You're checking a value but not using the result — that's suspicious"
- "This code could be written more simply and efficiently"

**Clippy** is like that senior developer for Rust. It analyzes your code and finds:
- Potential bugs
- Inefficient patterns
- Outdated coding style
- Places where code can be simplified

| Tool | Trading Analogy |
|------|-----------------|
| **Compiler** | Checks that order is syntactically correct |
| **Clippy** | Senior trader reviews order logic |
| **rustfmt** | Formats reports to standard |

## Installing and Running Clippy

Clippy is installed with Rust via rustup:

```bash
# Clippy is usually already installed, but you can update
rustup component add clippy

# Run clippy
cargo clippy

# Run with stricter checks
cargo clippy -- -W clippy::pedantic

# Run with all warnings as errors (CI mode)
cargo clippy -- -D warnings
```

## Clippy Lint Categories

Clippy organizes its checks into categories:

```rust
// Setting lint levels in code
#![warn(clippy::all)]           // Standard checks
#![warn(clippy::pedantic)]      // Stricter style checks
#![warn(clippy::nursery)]       // Experimental checks
#![warn(clippy::cargo)]         // Cargo.toml checks
#![deny(clippy::correctness)]   // Critical errors — deny
```

### Strictness Levels

| Category | Description | Use Case |
|----------|-------------|----------|
| `clippy::all` | Core checks | Always recommended |
| `clippy::pedantic` | Strict style checks | For clean code |
| `clippy::nursery` | New/experimental | For enthusiasts |
| `clippy::restriction` | Restrictive rules | For specific cases |
| `clippy::correctness` | Likely bugs | Critically important |

## Common Warnings and How to Fix Them

### 1. Unnecessary Cloning (Performance)

```rust
// Clippy warns: unnecessary clone
fn process_trade_bad(trade: Trade) {
    let trade_copy = trade.clone();  // Warning!
    println!("Processing: {:?}", trade_copy);
}

// Correct: use a reference
fn process_trade_good(trade: &Trade) {
    println!("Processing: {:?}", trade);
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}
```

### 2. Unused Result (Correctness)

```rust
use std::collections::HashMap;

struct OrderBook {
    bids: HashMap<u64, f64>,
    asks: HashMap<u64, f64>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    fn add_bid(&mut self, price: u64, quantity: f64) {
        // Clippy warns: ignoring Result from insert
        // Here it's fine, but clippy wants explicitness
        let _ = self.bids.insert(price, quantity);  // Correct: explicitly ignore
    }

    // Bad example:
    fn add_ask_bad(&mut self, price: u64, quantity: f64) {
        self.asks.insert(price, quantity);  // Warning!
    }
}
```

### 3. Non-optimal Iterators

```rust
fn calculate_total_volume_bad(prices: &[f64], quantities: &[f64]) -> f64 {
    // Clippy warns: could use zip
    let mut total = 0.0;
    for i in 0..prices.len() {
        total += prices[i] * quantities[i];  // Warning!
    }
    total
}

// Correct: idiomatic Rust with zip
fn calculate_total_volume_good(prices: &[f64], quantities: &[f64]) -> f64 {
    prices
        .iter()
        .zip(quantities.iter())
        .map(|(p, q)| p * q)
        .sum()
}
```

## Practical Application in Trading Code

### Example: Position Management System with clippy

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Disable some pedantic checks for readability
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

use std::collections::HashMap;

/// Trading position
#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    pub fn new(symbol: &str, quantity: f64, entry_price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            entry_price,
            current_price: entry_price,
        }
    }

    /// Calculates unrealized profit/loss
    pub fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    /// Updates current price
    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
    }

    /// Returns market value of position
    pub fn market_value(&self) -> f64 {
        self.current_price * self.quantity.abs()
    }
}

/// Trader's portfolio
pub struct Portfolio {
    positions: HashMap<String, Position>,
    cash: f64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_cash,
        }
    }

    /// Adds a new position or increases existing one
    pub fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        // Clippy approves: using entry API
        self.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                // Weighted average entry price
                let total_quantity = pos.quantity + quantity;
                if total_quantity.abs() > f64::EPSILON {
                    pos.entry_price = (pos.entry_price * pos.quantity + price * quantity)
                        / total_quantity;
                }
                pos.quantity = total_quantity;
            })
            .or_insert_with(|| Position::new(symbol, quantity, price));

        self.cash -= quantity * price;
    }

    /// Closes position completely
    pub fn close_position(&mut self, symbol: &str) -> Option<f64> {
        // Clippy approves: using remove instead of get + remove
        self.positions.remove(symbol).map(|pos| {
            let pnl = pos.unrealized_pnl();
            self.cash += pos.market_value() + pnl;
            pnl
        })
    }

    /// Updates prices for all positions
    pub fn update_prices(&mut self, prices: &HashMap<String, f64>) {
        // Clippy approves: values_mut for in-place modification
        for position in self.positions.values_mut() {
            if let Some(&price) = prices.get(&position.symbol) {
                position.update_price(price);
            }
        }
    }

    /// Calculates total unrealized PnL
    pub fn total_unrealized_pnl(&self) -> f64 {
        // Clippy approves: using sum()
        self.positions.values().map(Position::unrealized_pnl).sum()
    }

    /// Returns total portfolio value
    pub fn total_value(&self) -> f64 {
        self.cash + self.positions.values().map(Position::market_value).sum::<f64>()
    }

    /// Returns positions sorted by PnL
    pub fn positions_by_pnl(&self) -> Vec<&Position> {
        // Clippy may suggest sorted_by instead of sort_by on clone
        let mut positions: Vec<_> = self.positions.values().collect();
        positions.sort_by(|a, b| {
            b.unrealized_pnl()
                .partial_cmp(&a.unrealized_pnl())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        positions
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100_000.0);

    // Open positions
    portfolio.add_position("BTCUSDT", 0.5, 50_000.0);
    portfolio.add_position("ETHUSDT", 5.0, 3_000.0);
    portfolio.add_position("SOLUSDT", 100.0, 100.0);

    println!("=== Initial Portfolio ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Cash: ${:.2}", portfolio.cash);

    // Update prices
    let mut new_prices = HashMap::new();
    new_prices.insert("BTCUSDT".to_string(), 52_000.0);
    new_prices.insert("ETHUSDT".to_string(), 3_200.0);
    new_prices.insert("SOLUSDT".to_string(), 95.0);

    portfolio.update_prices(&new_prices);

    println!("\n=== After Price Update ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Unrealized PnL: ${:.2}", portfolio.total_unrealized_pnl());

    println!("\n=== Positions by PnL ===");
    for pos in portfolio.positions_by_pnl() {
        println!(
            "{}: quantity={:.2}, PnL=${:.2}",
            pos.symbol,
            pos.quantity,
            pos.unrealized_pnl()
        );
    }

    // Close profitable position
    if let Some(pnl) = portfolio.close_position("ETHUSDT") {
        println!("\nClosed ETHUSDT position with PnL: ${:.2}", pnl);
    }

    println!("\n=== Final Portfolio ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Cash: ${:.2}", portfolio.cash);
}
```

## Clippy Configuration via File

Create a `clippy.toml` file in the project root:

```toml
# clippy.toml

# Maximum function complexity
cognitive-complexity-threshold = 25

# Minimum length for magic number warnings
trivial-copy-size-limit = 8

# Allowed variable names (to disable warnings)
allowed-idents-below-min-chars = ["i", "j", "x", "y", "id"]

# Threshold for too many arguments
too-many-arguments-threshold = 7

# Threshold for too many lines in a function
too-many-lines-threshold = 100
```

## Automatic Fixing with cargo clippy --fix

```bash
# Automatically fix simple issues
cargo clippy --fix

# Fix even with uncommitted changes
cargo clippy --fix --allow-dirty

# Fix with permission for unstaged files
cargo clippy --fix --allow-staged
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/clippy.yml
name: Clippy

on: [push, pull_request]

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Run Clippy
        run: cargo clippy -- -D warnings
```

## Specific Lints for Trading Code

### Floating Point Precision Checks

```rust
#![warn(clippy::float_cmp)]

fn check_price_equal_bad(price1: f64, price2: f64) -> bool {
    price1 == price2  // Warning! Unsafe float comparison
}

fn check_price_equal_good(price1: f64, price2: f64) -> bool {
    (price1 - price2).abs() < f64::EPSILON * 100.0  // Correct
}

// Or use a special type for money
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Price(i64);  // Store in cents/satoshi

impl Price {
    fn from_float(value: f64, precision: u32) -> Self {
        Price((value * 10_f64.powi(precision as i32)).round() as i64)
    }

    fn to_float(self, precision: u32) -> f64 {
        self.0 as f64 / 10_f64.powi(precision as i32)
    }
}
```

### Overflow Checks for Large Volumes

```rust
#![warn(clippy::integer_arithmetic)]
#![warn(clippy::cast_possible_truncation)]

fn calculate_total_value_bad(quantity: u64, price: u64) -> u64 {
    quantity * price  // Warning! Possible overflow
}

fn calculate_total_value_good(quantity: u64, price: u64) -> Option<u64> {
    quantity.checked_mul(price)  // Correct: explicit overflow handling
}

// Or use saturating operations
fn calculate_total_value_safe(quantity: u64, price: u64) -> u64 {
    quantity.saturating_mul(price)  // Returns MAX on overflow
}
```

## Suppressing Warnings

Sometimes you need to disable specific warnings:

```rust
// For a specific line
#[allow(clippy::needless_return)]
fn get_price() -> f64 {
    return 100.0;  // Explicit return needed for clarity
}

// For a function
#[allow(clippy::too_many_arguments)]
fn create_complex_order(
    symbol: &str,
    side: &str,
    order_type: &str,
    price: f64,
    quantity: f64,
    stop_price: f64,
    take_profit: f64,
    time_in_force: &str,
) -> Order {
    // ...
    Order::default()
}

#[derive(Default)]
struct Order;

// For a module
#[allow(clippy::module_inception)]
mod order {
    pub mod order {
        // ...
    }
}
```

## Custom Lints for Trading Systems

While Clippy doesn't allow creating custom lints directly, you can use combinations of existing ones:

```rust
// Recommended configuration for trading systems
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

// Critical for finance
#![deny(clippy::float_cmp)]           // Deny unsafe float comparison
#![deny(clippy::integer_arithmetic)]   // Warn about possible overflows
#![deny(clippy::unwrap_used)]         // Deny .unwrap() in production code
#![deny(clippy::expect_used)]         // Deny .expect() in production code
#![deny(clippy::panic)]               // Deny panic! in production code

// Disable overly strict for our case
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **clippy::all** | Basic code quality checks |
| **clippy::pedantic** | Strict stylistic checks |
| **clippy::correctness** | Checks for likely bugs |
| **cargo clippy --fix** | Automatic problem fixing |
| **#[allow(clippy::...)]** | Suppressing specific warnings |
| **clippy.toml** | Configuring thresholds and settings |
| **CI integration** | Automatic checks in pipeline |

## Practical Exercises

1. **Audit existing code**: Take one of your projects and:
   - Run `cargo clippy -- -W clippy::pedantic`
   - Fix all warnings in the `correctness` category
   - Analyze `pedantic` warnings and decide which are worth fixing
   - Add suppressions for those that don't apply

2. **CI Setup**: Create a GitHub Actions workflow:
   - Runs clippy on every PR
   - Blocks merge when warnings exist
   - Sends report as PR comment

3. **Trading module refactoring**: Take the example code and:
   - Add overflow handling for all arithmetic operations
   - Replace float comparisons with safe alternatives
   - Remove all .unwrap() and replace with proper error handling

4. **Custom configuration**: Create `clippy.toml` for a trading project:
   - Configure complexity thresholds for financial code specifics
   - Define list of allowed short identifiers
   - Document reasons for disabling certain lints

## Homework

1. **Code quality monitoring system**: Implement a system that:
   - Runs clippy with different strictness levels
   - Groups warnings by category
   - Tracks trends (increase/decrease in warnings)
   - Generates report with fix priorities
   - Integrates with ticketing system

2. **Safe trading calculator**: Write a library:
   - All arithmetic operations with overflow checking
   - Type-safe representations for money and prices
   - Zero warnings from `clippy::pedantic`
   - Documentation for all public functions
   - 100% test coverage

3. **Trading logic linter**: Using clippy and custom checks:
   - Find patterns specific to trading errors
   - Implement checks through attributes and macros
   - Add business logic validation (order validation, risk limits)
   - Integrate with existing CI/CD

4. **Legacy code migration**: Take old trading code and:
   - Conduct full audit with clippy
   - Create migration plan with priorities
   - Implement automatic fixes where possible
   - Write documentation for changes
   - Add regression tests

## Navigation

[← Previous day](../347-testing-in-ci/en.md) | [Next day →](../349-formatting-rustfmt/en.md)
