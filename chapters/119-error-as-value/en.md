# Day 119: Pattern: Error as Value

## Trading Analogy

Imagine you're requesting data from an exchange. In most programming languages, if something goes wrong — server unavailable, invalid ticker, expired token — the program "explodes" with an exception. It's like your trading terminal just closing on any error.

In Rust, the approach is different. When you call a function to get a price, it returns a **result in an envelope**: either the price or a description of the problem. You open the envelope and decide what to do. No sudden "explosions" — everything is under control.

It's like professional risk management: instead of panicking on every problem, you get a clear report and make a considered decision.

## The "Error as Value" Philosophy

In Rust, errors are not exceptional situations but **regular values** returned from functions:

```rust
// In other languages:
// price = get_price("BTC")  // May "explode" at any moment!

// In Rust:
// result = get_price("BTC")  // Returns Result<f64, Error>
// You MUST handle both variants
```

## Result: Success or Error

`Result<T, E>` is an enum with two variants:

```rust
enum Result<T, E> {
    Ok(T),    // Success — contains value of type T
    Err(E),   // Error — contains error of type E
}
```

### Example: Getting Asset Price

```rust
fn main() {
    let symbols = ["BTC", "ETH", "INVALID", "SOL"];

    for symbol in symbols {
        match get_price(symbol) {
            Ok(price) => println!("{}: ${:.2}", symbol, price),
            Err(error) => println!("{}: Error - {}", symbol, error),
        }
    }
}

fn get_price(symbol: &str) -> Result<f64, String> {
    // Simulating price retrieval from exchange
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        "SOL" => Ok(100.0),
        _ => Err(format!("Unknown ticker: {}", symbol)),
    }
}
```

## Option: Value Present or Not

`Option<T>` — for cases when absence of value is not an error but a normal situation:

```rust
enum Option<T> {
    Some(T),  // Value exists
    None,     // No value
}
```

### Example: Finding the Best Trade

```rust
fn main() {
    let trades = vec![
        ("BTC", -150.0),
        ("ETH", 200.0),
        ("SOL", -50.0),
        ("DOGE", 500.0),
    ];

    match find_best_trade(&trades) {
        Some((symbol, pnl)) => {
            println!("Best trade: {} with profit ${:.2}", symbol, pnl);
        }
        None => println!("No profitable trades"),
    }

    let empty_trades: Vec<(&str, f64)> = vec![];
    match find_best_trade(&empty_trades) {
        Some((symbol, pnl)) => println!("Best: {} ${:.2}", symbol, pnl),
        None => println!("No trades to analyze"),
    }
}

fn find_best_trade(trades: &[(&str, f64)]) -> Option<(&str, f64)> {
    trades
        .iter()
        .filter(|(_, pnl)| *pnl > 0.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .copied()
}
```

## Advantages of the "Error as Value" Pattern

### 1. The Compiler Forces You to Handle Errors

```rust
fn main() {
    let price = get_price("BTC");

    // Compilation error! Cannot use Result directly
    // let doubled = price * 2.0;  // Won't compile

    // Correct: explicit handling
    let doubled = match price {
        Ok(p) => p * 2.0,
        Err(_) => 0.0,  // Or other handling logic
    };

    println!("Doubled price: ${:.2}", doubled);
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(String::from("Unknown ticker")),
    }
}
```

### 2. Errors Are Visible in the Function Signature

```rust
// Immediately visible that function may return an error
fn execute_order(
    symbol: &str,
    quantity: f64,
    price: f64,
) -> Result<OrderConfirmation, TradingError> {
    // ...
}

// Immediately visible that result may be absent
fn find_stop_loss(position: &Position) -> Option<f64> {
    // ...
}

struct OrderConfirmation {
    order_id: String,
    filled_price: f64,
}

struct Position {
    symbol: String,
    size: f64,
}

#[derive(Debug)]
enum TradingError {
    InsufficientBalance,
    MarketClosed,
    InvalidQuantity,
}

fn execute_order(
    symbol: &str,
    quantity: f64,
    _price: f64,
) -> Result<OrderConfirmation, TradingError> {
    if quantity <= 0.0 {
        return Err(TradingError::InvalidQuantity);
    }
    Ok(OrderConfirmation {
        order_id: format!("ORD-{}-001", symbol),
        filled_price: 42000.0,
    })
}

fn find_stop_loss(position: &Position) -> Option<f64> {
    if position.size > 0.0 {
        Some(position.size * 0.95)  // 5% stop loss
    } else {
        None
    }
}

fn main() {
    // Usage examples
    match execute_order("BTC", 0.5, 42000.0) {
        Ok(conf) => println!("Order executed: {}", conf.order_id),
        Err(e) => println!("Error: {:?}", e),
    }

    let pos = Position { symbol: String::from("BTC"), size: 1.0 };
    if let Some(sl) = find_stop_loss(&pos) {
        println!("Stop Loss: {:.2}", sl);
    }
}
```

### 3. Easy to Combine Checks

```rust
fn main() {
    match validate_and_execute_trade("BTC", 0.5, 42000.0, 50000.0) {
        Ok(result) => println!("Trade executed: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}

fn validate_and_execute_trade(
    symbol: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    // Chain of checks with the ? operator
    validate_symbol(symbol)?;
    validate_quantity(quantity)?;
    validate_balance(price * quantity, balance)?;

    // If all checks passed — execute the trade
    Ok(format!("Bought {} {} at ${:.2}", quantity, symbol, price))
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    let valid_symbols = ["BTC", "ETH", "SOL"];
    if valid_symbols.contains(&symbol) {
        Ok(())
    } else {
        Err(format!("Unsupported ticker: {}", symbol))
    }
}

fn validate_quantity(quantity: f64) -> Result<(), String> {
    if quantity > 0.0 {
        Ok(())
    } else {
        Err(String::from("Quantity must be positive"))
    }
}

fn validate_balance(cost: f64, balance: f64) -> Result<(), String> {
    if cost <= balance {
        Ok(())
    } else {
        Err(format!("Insufficient funds: need ${:.2}, have ${:.2}", cost, balance))
    }
}
```

## Comparison with Exceptions

```rust
fn main() {
    // Rust: explicit handling via Result
    let result = divide_safely(100.0, 0.0);
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(msg) => println!("Error: {}", msg),
    }

    // We KNOW FOR CERTAIN that the error is handled
    // No hidden execution paths
}

fn divide_safely(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// In languages with exceptions it would be:
// try {
//     result = divide(100, 0)  // May "throw" an exception
// } catch (e) {
//     // Easy to forget to handle
// }
// // Exception may "escape" many levels up
```

## Methods for Working with Result

```rust
fn main() {
    let prices: Vec<Result<f64, String>> = vec![
        Ok(42000.0),
        Err(String::from("API unavailable")),
        Ok(2500.0),
    ];

    // unwrap_or: default value
    for price in &prices {
        let value = price.clone().unwrap_or(0.0);
        println!("Price: ${:.2}", value);
    }

    println!("---");

    // map: transform successful value
    for price in &prices {
        let doubled = price.clone().map(|p| p * 2.0);
        println!("Doubled: {:?}", doubled);
    }

    println!("---");

    // is_ok / is_err: type check
    let success_count = prices.iter().filter(|p| p.is_ok()).count();
    println!("Successful requests: {}", success_count);
}
```

## Methods for Working with Option

```rust
fn main() {
    let stop_losses: Vec<Option<f64>> = vec![
        Some(41000.0),
        None,
        Some(2400.0),
    ];

    // unwrap_or: default value
    for sl in &stop_losses {
        let value = sl.unwrap_or(0.0);
        println!("Stop Loss: ${:.2}", value);
    }

    println!("---");

    // map: transformation
    for sl in &stop_losses {
        let adjusted = sl.map(|s| s * 0.99);  // Reduce by 1%
        println!("Adjusted SL: {:?}", adjusted);
    }

    println!("---");

    // filter: conditional filtering
    for sl in &stop_losses {
        let significant = sl.filter(|&s| s > 10000.0);
        println!("Significant SL: {:?}", significant);
    }
}
```

## Practical Example: Portfolio Processing

```rust
fn main() {
    let portfolio = vec![
        ("BTC", 0.5),
        ("INVALID", 10.0),
        ("ETH", 2.0),
        ("FAKE", 100.0),
    ];

    println!("=== Portfolio Analysis ===\n");

    let mut total_value = 0.0;
    let mut errors = Vec::new();

    for (symbol, quantity) in &portfolio {
        match get_position_value(symbol, *quantity) {
            Ok(value) => {
                println!("✓ {}: {} units × ${:.2} = ${:.2}",
                    symbol, quantity, value / quantity, value);
                total_value += value;
            }
            Err(error) => {
                println!("✗ {}: {}", symbol, error);
                errors.push((symbol, error));
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total value: ${:.2}", total_value);
    println!("Calculation errors: {}", errors.len());

    if !errors.is_empty() {
        println!("\nProblematic positions:");
        for (symbol, error) in errors {
            println!("  - {}: {}", symbol, error);
        }
    }
}

fn get_position_value(symbol: &str, quantity: f64) -> Result<f64, String> {
    let price = get_price(symbol)?;  // The ? operator propagates the error

    if quantity <= 0.0 {
        return Err(String::from("Invalid quantity"));
    }

    Ok(price * quantity)
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        "SOL" => Ok(100.0),
        _ => Err(format!("Unknown ticker: {}", symbol)),
    }
}
```

## Converting Between Result and Option

```rust
fn main() {
    // Result -> Option (discarding error information)
    let result: Result<f64, String> = Ok(42000.0);
    let option: Option<f64> = result.ok();
    println!("Result -> Option: {:?}", option);

    // Option -> Result (adding error information)
    let option: Option<f64> = Some(42000.0);
    let result: Result<f64, String> = option.ok_or(String::from("Value is missing"));
    println!("Option -> Result: {:?}", result);

    // Example in trading context
    let price = find_cached_price("BTC")           // Option<f64>
        .ok_or(String::from("Price not found in cache"));  // Result<f64, String>

    println!("Cached price: {:?}", price);
}

fn find_cached_price(symbol: &str) -> Option<f64> {
    match symbol {
        "BTC" => Some(42000.0),
        "ETH" => Some(2500.0),
        _ => None,
    }
}
```

## When to Use Result vs Option

```rust
// Result<T, E> — when it's important to know the REASON for absence
fn fetch_price_from_api(symbol: &str) -> Result<f64, ApiError> {
    // May return: NetworkError, TimeoutError, InvalidSymbol, etc.
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(ApiError::InvalidSymbol(symbol.to_string())),
    }
}

// Option<T> — when absence is normal and doesn't require explanation
fn find_cached_price(symbol: &str) -> Option<f64> {
    // Just not in cache — that's normal
    match symbol {
        "BTC" => Some(42000.0),
        "ETH" => Some(2500.0),
        _ => None,
    }
}

#[derive(Debug)]
enum ApiError {
    NetworkError,
    Timeout,
    InvalidSymbol(String),
}

fn main() {
    // Difference in handling

    // Result: handle specific error
    match fetch_price_from_api("INVALID") {
        Ok(price) => println!("Price: {}", price),
        Err(ApiError::NetworkError) => println!("Network issues, try again later"),
        Err(ApiError::Timeout) => println!("Request timed out"),
        Err(ApiError::InvalidSymbol(s)) => println!("Unknown ticker: {}", s),
    }

    // Option: simple presence check
    match find_cached_price("UNKNOWN") {
        Some(price) => println!("From cache: {}", price),
        None => println!("Requesting from API..."),
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Error as value | Errors are regular values, not exceptions |
| Result<T, E> | Success or error with reason information |
| Option<T> | Value present or absent |
| ? operator | Propagates error up the stack |
| Explicit handling | Compiler requires handling all variants |
| Combinators | map, unwrap_or, ok_or for transformations |

## Practice Exercises

1. Write a function `parse_trade_line(line: &str) -> Result<Trade, ParseError>` that parses a line in format "BTC,0.5,42000.0" into a Trade struct

2. Implement a function `find_trade_by_id(trades: &[Trade], id: u64) -> Option<&Trade>` to find a trade by ID

3. Create a processing chain: get price → validate → calculate position, where each step can return an error

## Homework

1. Implement an order processing system with different error types:
   - `InsufficientBalance`
   - `InvalidQuantity`
   - `MarketClosed`
   - `PriceTooFar` (price deviated from market)

2. Write a function `batch_get_prices(symbols: &[&str]) -> Vec<Result<f64, String>>` that returns prices for a list of tickers, where each result is independent

3. Create a portfolio handler that:
   - Collects all successful values
   - Logs all errors
   - Returns final statistics

4. Implement the "Circuit Breaker" pattern: after N failed API requests, the function starts returning cached data

## Navigation

[← Previous day](../118-fail-fast/en.md) | [Next day →](../120-robust-exchange-api-client/en.md)
