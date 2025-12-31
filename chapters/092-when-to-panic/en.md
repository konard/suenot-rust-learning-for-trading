# Day 92: When to Panic — Unrecoverable Errors

## Trading Analogy

Imagine a trading bot that suddenly discovers its configuration contains a negative position size or that the API key is missing at startup. This isn't a situation that can be "handled" — it's a **critical error** where continuing operation is dangerous.

In real trading: if a risk management system detects a violation of a critical invariant (e.g., position exceeds 100% of the deposit), it **immediately halts trading**. This is what `panic!` is in Rust — an emergency program termination upon an unrecoverable error.

## What is panic!

`panic!` is a macro that immediately terminates the program (or current thread) when an unrecoverable error is detected.

```rust
fn main() {
    panic!("Critical error: cannot continue!");
}
```

When `panic!` is called:
1. An error message is printed
2. The stack is unwound (unwinding) or the program aborts immediately (abort)
3. The program exits with a non-zero exit code

## When to Use panic!

### 1. Program Invariant Violation

```rust
fn main() {
    let balance = -1000.0;
    validate_balance(balance);
}

fn validate_balance(balance: f64) {
    if balance < 0.0 {
        panic!(
            "CRITICAL ERROR: Negative balance {} is impossible! \
             Possible calculation error.",
            balance
        );
    }
    println!("Balance is valid: ${:.2}", balance);
}
```

### 2. Configuration Errors at Startup

```rust
fn main() {
    let config = TradingConfig::load();
    println!("Configuration loaded: {:?}", config);
}

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    max_position_size: f64,
    risk_per_trade: f64,
}

impl TradingConfig {
    fn load() -> Self {
        // Simulating configuration loading
        let api_key = std::env::var("API_KEY").unwrap_or_default();
        let max_position = 10000.0;
        let risk = 2.0;

        // Critical checks at startup
        if api_key.is_empty() {
            panic!("API_KEY is not set! Trading is impossible.");
        }

        if max_position <= 0.0 {
            panic!(
                "Invalid max_position_size: {}. Must be positive.",
                max_position
            );
        }

        if risk <= 0.0 || risk > 100.0 {
            panic!(
                "Invalid risk_per_trade: {}%. Valid range: 0-100%.",
                risk
            );
        }

        TradingConfig {
            api_key,
            max_position_size: max_position,
            risk_per_trade: risk,
        }
    }
}
```

### 3. Unreachable Code

```rust
fn main() {
    let side = "BUY";
    let result = calculate_pnl(side, 100.0, 110.0, 1.0);
    println!("PnL: ${:.2}", result);
}

fn calculate_pnl(side: &str, entry: f64, exit: f64, qty: f64) -> f64 {
    match side {
        "BUY" => (exit - entry) * qty,
        "SELL" => (entry - exit) * qty,
        _ => panic!("Invalid trade side: '{}'. Expected BUY or SELL.", side),
    }
}
```

### 4. Using unreachable!()

```rust
fn main() {
    let order_type = OrderType::Market;
    process_order(order_type, 42000.0);
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
}

fn process_order(order_type: OrderType, price: f64) {
    match order_type {
        OrderType::Market => {
            println!("Executing market order at current price");
        }
        OrderType::Limit => {
            println!("Placing limit order at ${:.2}", price);
        }
        OrderType::StopLoss => {
            println!("Setting stop-loss at ${:.2}", price);
        }
        // If we add a new variant and forget to handle it:
        #[allow(unreachable_patterns)]
        _ => unreachable!("All order types must be handled!"),
    }
}
```

## panic! vs Result: When to Use Which

```rust
use std::collections::HashMap;

fn main() {
    // Example 1: Result for recoverable errors
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert("BTC".to_string(), 1.5);
    portfolio.insert("ETH".to_string(), 10.0);

    match get_position(&portfolio, "BTC") {
        Ok(qty) => println!("BTC position: {} units", qty),
        Err(e) => println!("Error: {}", e),
    }

    match get_position(&portfolio, "DOGE") {
        Ok(qty) => println!("DOGE position: {} units", qty),
        Err(e) => println!("Warning: {}", e),
    }

    // Example 2: panic! for unrecoverable errors
    let critical_config = CriticalConfig {
        max_drawdown_percent: 25.0,
        emergency_stop: true,
    };
    validate_critical_config(&critical_config);
}

// Recoverable error - use Result
fn get_position(portfolio: &HashMap<String, f64>, ticker: &str) -> Result<f64, String> {
    portfolio
        .get(ticker)
        .copied()
        .ok_or_else(|| format!("Ticker '{}' not found in portfolio", ticker))
}

struct CriticalConfig {
    max_drawdown_percent: f64,
    emergency_stop: bool,
}

// Unrecoverable error - use panic!
fn validate_critical_config(config: &CriticalConfig) {
    if config.max_drawdown_percent <= 0.0 || config.max_drawdown_percent > 100.0 {
        panic!(
            "CRITICAL CONFIG ERROR: \
             max_drawdown_percent = {}% is invalid!",
            config.max_drawdown_percent
        );
    }

    if !config.emergency_stop {
        panic!(
            "CRITICAL SECURITY ERROR: \
             emergency_stop must be enabled!"
        );
    }

    println!("Critical configuration is valid.");
}
```

## Methods That Cause panic!

```rust
fn main() {
    // unwrap() - panics on None or Err
    let prices = vec![42000.0, 42500.0, 41800.0];

    // Dangerous! Panics if index is out of bounds
    // let price = prices[10]; // panic: index out of bounds

    // Safe alternative
    match prices.get(10) {
        Some(price) => println!("Price: {}", price),
        None => println!("Index out of bounds"),
    }

    // unwrap() - use only when you're certain
    let valid_price: Option<f64> = Some(42000.0);
    let price = valid_price.unwrap(); // OK, we know it's Some
    println!("Price: {}", price);

    // expect() - panic with custom message
    let api_response: Option<f64> = Some(42500.0);
    let current_price = api_response
        .expect("API must return current price");
    println!("Current price: ${:.2}", current_price);
}
```

## unwrap_or and Alternatives to panic

```rust
fn main() {
    let maybe_price: Option<f64> = None;

    // Instead of panic - default value
    let price1 = maybe_price.unwrap_or(0.0);
    println!("Price (or 0): {}", price1);

    // Lazy evaluation of default value
    let price2 = maybe_price.unwrap_or_else(|| {
        println!("Computing default value...");
        fetch_default_price()
    });
    println!("Price (or default): {}", price2);

    // unwrap_or_default for types with Default
    let maybe_qty: Option<f64> = None;
    let qty = maybe_qty.unwrap_or_default(); // 0.0 for f64
    println!("Quantity: {}", qty);
}

fn fetch_default_price() -> f64 {
    42000.0 // Simulating price fetch
}
```

## Practical Example: Trading System Validation

```rust
fn main() {
    // Create trading system with validation
    let system = TradingSystem::new(
        10000.0,  // initial balance
        2.0,      // risk per trade %
        10.0,     // max drawdown %
    );

    println!("{:?}", system);

    // Attempt to create system with invalid parameters
    // Uncomment to test panic:
    // let invalid_system = TradingSystem::new(-1000.0, 2.0, 10.0);
}

#[derive(Debug)]
struct TradingSystem {
    balance: f64,
    risk_per_trade: f64,
    max_drawdown: f64,
    is_active: bool,
}

impl TradingSystem {
    fn new(balance: f64, risk_per_trade: f64, max_drawdown: f64) -> Self {
        // Critical checks - violation = panic
        if balance <= 0.0 {
            panic!(
                "CRITICAL ERROR: Initial balance must be positive. \
                 Got: ${:.2}",
                balance
            );
        }

        if risk_per_trade <= 0.0 {
            panic!(
                "CRITICAL ERROR: Risk per trade must be positive. \
                 Got: {:.2}%",
                risk_per_trade
            );
        }

        if risk_per_trade > 10.0 {
            panic!(
                "CRITICAL ERROR: Risk per trade exceeds safe limit of 10%. \
                 Got: {:.2}%",
                risk_per_trade
            );
        }

        if max_drawdown <= 0.0 || max_drawdown > 50.0 {
            panic!(
                "CRITICAL ERROR: Max drawdown must be in range 0-50%. \
                 Got: {:.2}%",
                max_drawdown
            );
        }

        println!("Trading system initialized successfully");
        println!("  Balance: ${:.2}", balance);
        println!("  Risk per trade: {:.2}%", risk_per_trade);
        println!("  Max drawdown: {:.2}%", max_drawdown);

        TradingSystem {
            balance,
            risk_per_trade,
            max_drawdown,
            is_active: true,
        }
    }
}
```

## Catching panic with catch_unwind

```rust
use std::panic;

fn main() {
    println!("Testing panic catching...\n");

    // Catch panic to isolate errors
    let result = panic::catch_unwind(|| {
        risky_calculation(-100.0)
    });

    match result {
        Ok(value) => println!("Result: {}", value),
        Err(_) => println!("Function panicked, but program continues"),
    }

    println!("\nProgram continues after catching panic!");

    // Normal call
    let safe_result = panic::catch_unwind(|| {
        risky_calculation(100.0)
    });

    match safe_result {
        Ok(value) => println!("Safe result: {}", value),
        Err(_) => println!("Unexpected panic"),
    }
}

fn risky_calculation(value: f64) -> f64 {
    if value < 0.0 {
        panic!("Negative value not allowed: {}", value);
    }
    value * 2.0
}
```

## assert! Macros for Invariant Testing

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    // assert! - condition check
    assert!(entry_price > 0.0, "Entry price must be positive");
    assert!(quantity > 0.0, "Quantity must be positive");

    let pnl = calculate_pnl(entry_price, exit_price, quantity);
    println!("PnL: ${:.2}", pnl);

    // assert_eq! - equality check
    let expected_pnl = 750.0;
    assert_eq!(pnl, expected_pnl, "PnL should equal $750");

    // assert_ne! - inequality check
    assert_ne!(pnl, 0.0, "PnL should not be zero");

    println!("All assertions passed!");
}

fn calculate_pnl(entry: f64, exit: f64, qty: f64) -> f64 {
    (exit - entry) * qty
}
```

## debug_assert! for Debug-Only Checks

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42300.0, 42200.0];

    let sma = calculate_sma(&prices, 3);
    println!("SMA-3: ${:.2}", sma);
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    // These checks only run in debug mode
    debug_assert!(
        !prices.is_empty(),
        "Price array must not be empty"
    );
    debug_assert!(
        period > 0,
        "Period must be positive"
    );
    debug_assert!(
        period <= prices.len(),
        "Period ({}) cannot exceed number of prices ({})",
        period,
        prices.len()
    );

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    sum / period as f64
}
```

## Guidelines: When to panic, When to Result

```rust
fn main() {
    println!("=== Guide for Choosing Between panic! and Result ===\n");

    // USE panic! WHEN:
    println!("Use panic! when:");
    println!("  - Critical program invariant is violated");
    println!("  - Configuration error at startup");
    println!("  - Code is theoretically unreachable");
    println!("  - In tests for condition verification");
    println!("  - Programmer error (bug), not user error\n");

    // USE Result WHEN:
    println!("Use Result when:");
    println!("  - Error is expected and recoverable");
    println!("  - User can correct the input");
    println!("  - Network request may fail");
    println!("  - File may not exist");
    println!("  - Parsing may fail\n");

    // Examples
    println!("=== Examples ===\n");

    // This should be panic! - invariant violation
    let portfolio_value = 50000.0;
    let position_value = 45000.0;
    let exposure = position_value / portfolio_value;

    if exposure > 1.0 {
        panic!("Impossible: exposure > 100%");
    }
    println!("Exposure: {:.1}% - OK", exposure * 100.0);

    // This should be Result - user input
    match parse_order_quantity("abc") {
        Ok(qty) => println!("Quantity: {}", qty),
        Err(e) => println!("Input error (expected): {}", e),
    }
}

fn parse_order_quantity(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("'{}' is not a number", input))
}
```

## Practical Exercises

### Exercise 1: Order Validation
Write a function `validate_order` that panics on critical errors and returns `Result` for recoverable ones.

### Exercise 2: Safe Data Access
Implement a function for getting historical data with proper use of `panic!` vs `Option`.

### Exercise 3: Assertion System
Create a set of `assert!` checks for validating a trading strategy.

### Exercise 4: Panic Isolation
Use `catch_unwind` to isolate potentially panicking code in a trading bot.

## What We Learned

| Tool | When to Use |
|------|-------------|
| `panic!()` | Unrecoverable error, invariant violation |
| `unreachable!()` | Code that should never execute |
| `assert!()` | Condition check (panics if false) |
| `assert_eq!()` | Equality check |
| `debug_assert!()` | Debug-mode only check |
| `unwrap()` | Extract value (panics on error) |
| `expect()` | `unwrap` with custom message |
| `catch_unwind()` | Catch panic for isolation |

## Homework

1. Create a `RiskManager` struct with a constructor that panics on invalid risk parameters

2. Write a portfolio validation function using a combination of `panic!` for critical errors and `Result` for recoverable ones

3. Implement a set of `debug_assert!` checks for a position sizing function that don't affect performance in release builds

4. Create a strategy testing "sandbox" using `catch_unwind` to isolate panicking strategies

## Navigation

[← Previous day](../091-propagating-errors/en.md) | [Next day →](../093-custom-error-types/en.md)
