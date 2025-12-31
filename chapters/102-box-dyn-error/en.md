# Day 102: Box<dyn Error> — Any Error

## Trading Analogy

Imagine you're managing a trading system that receives data from **multiple sources**: exchange API, historical data files, positions database, order network streams. Each source can produce **its own error type**: network error, JSON parsing error, file reading error, database error.

Instead of handling each type separately, you need a **universal container for any error** — like a unified monitoring dashboard that shows problems from all sources in one format.

`Box<dyn Error>` is exactly that container. It can hold **any error** that implements the `Error` trait.

## Why Do We Need Box<dyn Error>?

### The Problem: Different Error Types

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseFloatError;

// This function can return different error types
fn load_prices_problem(path: &str) -> Vec<f64> {
    // io::Error when reading file
    // ParseFloatError when parsing price
    // How to return Result with both types?
    vec![]
}
```

### The Solution: Box<dyn Error>

```rust
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn load_prices(path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let file = File::open(path)?;  // io::Error automatically converts
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for line in reader.lines() {
        let line = line?;  // io::Error
        let price: f64 = line.trim().parse()?;  // ParseFloatError
        prices.push(price);
    }

    Ok(prices)
}

fn main() {
    match load_prices("prices.txt") {
        Ok(prices) => println!("Loaded {} prices", prices.len()),
        Err(e) => println!("Error: {}", e),
    }
}
```

## How It Works

`Box<dyn Error>` consists of two parts:

1. **Box** — a smart pointer that stores data on the heap
2. **dyn Error** — a dynamic trait object representing any type with the `Error` trait

```rust
use std::error::Error;
use std::fmt;

// Example of creating a custom error
#[derive(Debug)]
struct InsufficientBalanceError {
    required: f64,
    available: f64,
}

impl fmt::Display for InsufficientBalanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Insufficient balance: need ${:.2}, have ${:.2}",
            self.required, self.available
        )
    }
}

impl Error for InsufficientBalanceError {}

fn check_balance(required: f64, available: f64) -> Result<(), Box<dyn Error>> {
    if required > available {
        return Err(Box::new(InsufficientBalanceError { required, available }));
    }
    Ok(())
}

fn main() {
    match check_balance(10000.0, 5000.0) {
        Ok(()) => println!("Balance OK"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Practical Example: Loading Trading Data

```rust
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn parse_trade(line: &str) -> Result<Trade, Box<dyn Error>> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 4 {
        return Err(format!("Invalid trade format: expected 4 fields, got {}", parts.len()).into());
    }

    Ok(Trade {
        symbol: parts[0].trim().to_string(),
        price: parts[1].trim().parse()?,
        quantity: parts[2].trim().parse()?,
        side: parts[3].trim().to_string(),
    })
}

fn load_trades(path: &str) -> Result<Vec<Trade>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;  // Skip empty lines and comments
        }

        match parse_trade(&line) {
            Ok(trade) => trades.push(trade),
            Err(e) => {
                return Err(format!("Error on line {}: {}", line_num + 1, e).into());
            }
        }
    }

    Ok(trades)
}

fn calculate_portfolio_value(trades: &[Trade]) -> f64 {
    trades.iter()
        .filter(|t| t.side == "BUY")
        .map(|t| t.price * t.quantity)
        .sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let trades = load_trades("trades.csv")?;

    println!("Loaded {} trades", trades.len());
    println!("Portfolio value: ${:.2}", calculate_portfolio_value(&trades));

    for trade in &trades {
        println!(
            "  {} {} {} @ ${:.2}",
            trade.side, trade.quantity, trade.symbol, trade.price
        );
    }

    Ok(())
}
```

## Converting String to Box<dyn Error>

```rust
use std::error::Error;

fn validate_order(
    symbol: &str,
    price: f64,
    quantity: f64,
) -> Result<(), Box<dyn Error>> {
    if symbol.is_empty() {
        return Err("Symbol cannot be empty".into());  // &str -> Box<dyn Error>
    }

    if price <= 0.0 {
        return Err(format!("Invalid price: {}", price).into());  // String -> Box<dyn Error>
    }

    if quantity <= 0.0 {
        return Err("Quantity must be positive".into());
    }

    Ok(())
}

fn main() {
    match validate_order("", 100.0, 10.0) {
        Ok(()) => println!("Order valid"),
        Err(e) => println!("Validation error: {}", e),
    }
}
```

## Working with Multiple Data Sources

```rust
use std::error::Error;
use std::collections::HashMap;

// Simulating different data sources
fn fetch_price_from_exchange(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // May return network error
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        _ => Err(format!("Unknown symbol: {}", symbol).into()),
    }
}

fn fetch_price_from_cache(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // May return cache error
    let cache: HashMap<&str, f64> = [("BTC", 41950.0), ("ETH", 2790.0)]
        .into_iter()
        .collect();

    cache
        .get(symbol)
        .copied()
        .ok_or_else(|| format!("Symbol {} not in cache", symbol).into())
}

fn fetch_price_from_file(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // May return io::Error or ParseFloatError
    // Simplified version
    match symbol {
        "BTC" => Ok(41900.0),
        _ => Err("Price file not found".into()),
    }
}

fn get_best_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Try different sources with different error types
    let exchange_result = fetch_price_from_exchange(symbol);
    let cache_result = fetch_price_from_cache(symbol);
    let file_result = fetch_price_from_file(symbol);

    // Collect all successful results
    let mut prices = Vec::new();

    if let Ok(p) = exchange_result { prices.push(("exchange", p)); }
    if let Ok(p) = cache_result { prices.push(("cache", p)); }
    if let Ok(p) = file_result { prices.push(("file", p)); }

    if prices.is_empty() {
        return Err(format!("No price available for {}", symbol).into());
    }

    // Return average price
    let avg: f64 = prices.iter().map(|(_, p)| p).sum::<f64>() / prices.len() as f64;

    println!("Price sources for {}:", symbol);
    for (source, price) in &prices {
        println!("  {}: ${:.2}", source, price);
    }
    println!("  Average: ${:.2}", avg);

    Ok(avg)
}

fn main() -> Result<(), Box<dyn Error>> {
    let btc_price = get_best_price("BTC")?;
    println!("\nFinal BTC price: ${:.2}", btc_price);

    let eth_price = get_best_price("ETH")?;
    println!("Final ETH price: ${:.2}", eth_price);

    // This will return an error
    match get_best_price("UNKNOWN") {
        Ok(p) => println!("Price: {}", p),
        Err(e) => println!("\nError: {}", e),
    }

    Ok(())
}
```

## The ? Operator with Box<dyn Error>

```rust
use std::error::Error;

fn process_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    balance: f64,
) -> Result<f64, Box<dyn Error>> {
    // Each line with ? can return a different error type
    validate_symbol(symbol)?;           // Validation error
    validate_side(side)?;               // Validation error
    let price = get_market_price(symbol)?;  // Network/data error
    let cost = calculate_cost(price, quantity)?;  // Calculation error
    check_sufficient_balance(cost, balance)?;     // Business logic error

    Ok(cost)
}

fn validate_symbol(symbol: &str) -> Result<(), Box<dyn Error>> {
    if symbol.len() < 2 || symbol.len() > 10 {
        return Err("Symbol must be 2-10 characters".into());
    }
    Ok(())
}

fn validate_side(side: &str) -> Result<(), Box<dyn Error>> {
    match side {
        "BUY" | "SELL" => Ok(()),
        _ => Err(format!("Invalid side: {}. Must be BUY or SELL", side).into()),
    }
}

fn get_market_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        "SOL" => Ok(95.0),
        _ => Err(format!("No market data for {}", symbol).into()),
    }
}

fn calculate_cost(price: f64, quantity: f64) -> Result<f64, Box<dyn Error>> {
    if quantity <= 0.0 {
        return Err("Quantity must be positive".into());
    }
    Ok(price * quantity)
}

fn check_sufficient_balance(cost: f64, balance: f64) -> Result<(), Box<dyn Error>> {
    if cost > balance {
        return Err(format!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            cost, balance
        ).into());
    }
    Ok(())
}

fn main() {
    let test_cases = [
        ("BTC", "BUY", 0.5, 50000.0),
        ("ETH", "SELL", 10.0, 1000.0),
        ("X", "BUY", 1.0, 1000.0),      // Invalid symbol
        ("BTC", "HOLD", 1.0, 50000.0),  // Invalid side
        ("BTC", "BUY", 1.0, 100.0),     // Insufficient balance
    ];

    for (symbol, side, qty, balance) in test_cases {
        println!("\nOrder: {} {} {} with balance ${:.2}", side, qty, symbol, balance);
        match process_order(symbol, side, qty, balance) {
            Ok(cost) => println!("  ✓ Order cost: ${:.2}", cost),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }
}
```

## main() with Box<dyn Error>

```rust
use std::error::Error;

fn run_trading_bot() -> Result<(), Box<dyn Error>> {
    println!("Starting trading bot...");

    // Initialization
    let config = load_config()?;
    let positions = load_positions()?;

    println!("Config loaded: {} pairs", config.len());
    println!("Positions loaded: {} open", positions.len());

    // Trading logic
    for symbol in &config {
        let price = fetch_price(symbol)?;
        println!("{}: ${:.2}", symbol, price);
    }

    println!("Trading bot finished successfully");
    Ok(())
}

fn load_config() -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()])
}

fn load_positions() -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    Ok(vec![
        ("BTC".to_string(), 0.5),
        ("ETH".to_string(), 5.0),
    ])
}

fn fetch_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        "SOL" => Ok(95.0),
        _ => Err(format!("Unknown symbol: {}", symbol).into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    run_trading_bot()
}
```

## Comparing Error Handling Approaches

| Approach | When to Use | Example |
|----------|-------------|---------|
| `Result<T, ConcreteError>` | Single known error type | `Result<f64, ParseFloatError>` |
| `Result<T, String>` | Simple error messages | `Result<f64, String>` |
| `Result<T, Box<dyn Error>>` | Multiple error types | File reading + parsing |
| Custom enum | Full control over errors | `enum TradingError { ... }` |

## Limitations of Box<dyn Error>

```rust
use std::error::Error;

fn example() -> Result<(), Box<dyn Error>> {
    // Box<dyn Error> erases the concrete error type
    // We cannot pattern match on the type

    let result: Result<i32, Box<dyn Error>> = Err("some error".into());

    match result {
        Ok(v) => println!("Value: {}", v),
        Err(e) => {
            // Can only get the message
            println!("Error: {}", e);

            // Can check source (if any)
            if let Some(source) = e.source() {
                println!("Caused by: {}", source);
            }
        }
    }

    Ok(())
}

fn main() {
    let _ = example();
}
```

## Exercises

### Exercise 1: Exchange Data Parser

```rust
use std::error::Error;

// TODO: Implement a function that parses a line in format
// "BTC,42000.50,0.5,BUY,2024-01-15T10:30:00"
// and returns a Trade struct

struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: String,
}

fn parse_trade_line(line: &str) -> Result<Trade, Box<dyn Error>> {
    // Your code here
    todo!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let line = "BTC,42000.50,0.5,BUY,2024-01-15T10:30:00";
    let trade = parse_trade_line(line)?;
    println!("Parsed: {} {} {} @ {}",
        trade.side, trade.quantity, trade.symbol, trade.price);
    Ok(())
}
```

### Exercise 2: Multi-Source Data

```rust
use std::error::Error;

// TODO: Implement a function that tries to get price from multiple
// sources and returns the first successful result

fn get_price_with_fallback(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Source 1: API (may fail)
    // Source 2: Cache (may be stale)
    // Source 3: File (may not exist)
    // If all fail — return error
    todo!()
}

fn main() {
    match get_price_with_fallback("BTC") {
        Ok(price) => println!("Got price: ${:.2}", price),
        Err(e) => println!("All sources failed: {}", e),
    }
}
```

### Exercise 3: Portfolio Validator

```rust
use std::error::Error;

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

// TODO: Implement a portfolio validation function that checks:
// 1. All symbols are valid (2-10 chars, letters only)
// 2. All quantities are positive
// 3. All prices are positive
// 4. Total value doesn't exceed limit

fn validate_portfolio(
    positions: &[Position],
    max_value: f64,
) -> Result<f64, Box<dyn Error>> {
    todo!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 10.0, entry_price: 2800.0 },
    ];

    let total = validate_portfolio(&portfolio, 100000.0)?;
    println!("Portfolio value: ${:.2}", total);
    Ok(())
}
```

## Homework

1. **Configuration Loader**: Write a function `load_trading_config(path: &str) -> Result<TradingConfig, Box<dyn Error>>` that reads a JSON/TOML config file and validates all fields.

2. **Order Processor**: Create an order processing system where `execute_order(order: Order) -> Result<ExecutionReport, Box<dyn Error>>` can return validation errors, network errors, and insufficient balance errors.

3. **Market Data Aggregator**: Implement `aggregate_market_data(symbols: &[&str]) -> Result<MarketSnapshot, Box<dyn Error>>` that collects data from multiple sources and handles partial failures.

4. **Risk Manager**: Write `check_risk_limits(portfolio: &Portfolio, new_order: &Order) -> Result<(), Box<dyn Error>>` that checks position limits, exposure, drawdown, and returns clear error messages.

## What We Learned

- `Box<dyn Error>` allows returning any error type
- The `?` operator automatically converts errors to `Box<dyn Error>`
- Strings easily convert via `.into()`
- `main()` can return `Result<(), Box<dyn Error>>`
- This is convenient for prototypes and scripts, but for production it's better to use concrete error types

## Navigation

[← Previous day](../101-custom-error-types/en.md) | [Next day →](../103-thiserror-crate/en.md)
