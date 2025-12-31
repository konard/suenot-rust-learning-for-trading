# Day 101: Multiple Error Types in Function

## Trading Analogy

Imagine you're validating a trade order before sending it to the exchange. The order can be rejected for **different reasons**:
- Insufficient balance (balance error)
- Invalid ticker (validation error)
- Exchange unavailable (network error)
- Invalid price format (parsing error)

Each reason is a **different error type**, but the function must handle them all. Rust offers several approaches for this.

## The Problem: Different Error Types

```rust
use std::fs::File;
use std::io::{self, Read};

fn load_portfolio(path: &str) -> Result<f64, ???> {  // What error type?
    let mut file = File::open(path)?;  // io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // io::Error
    let balance: f64 = contents.trim().parse()?;  // ParseFloatError
    Ok(balance)
}
```

Here `File::open` returns `io::Error`, while `parse()` returns `ParseFloatError`. How do we combine them in one `Result`?

## Solution 1: Box<dyn Error>

A universal container for any error:

```rust
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn load_portfolio(path: &str) -> Result<f64, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let balance: f64 = contents.trim().parse()?;
    Ok(balance)
}

fn main() {
    match load_portfolio("portfolio.txt") {
        Ok(balance) => println!("Portfolio balance: ${:.2}", balance),
        Err(e) => println!("Failed to load portfolio: {}", e),
    }
}
```

**Pros:** Simple, works with any error type.
**Cons:** Lose information about the specific error type.

## Solution 2: Custom Error Enum

Create your own type that combines all possible errors:

```rust
use std::fmt;
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradeError {
    IoError(io::Error),
    ParseError(ParseFloatError),
    ValidationError(String),
    InsufficientBalance { required: f64, available: f64 },
    InvalidTicker(String),
}

impl fmt::Display for TradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeError::IoError(e) => write!(f, "IO error: {}", e),
            TradeError::ParseError(e) => write!(f, "Parse error: {}", e),
            TradeError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TradeError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need ${:.2}, have ${:.2}", required, available)
            }
            TradeError::InvalidTicker(ticker) => write!(f, "Invalid ticker: {}", ticker),
        }
    }
}

impl std::error::Error for TradeError {}
```

## Implementing From for Automatic Conversion

To make the `?` operator work automatically:

```rust
use std::io;
use std::num::ParseFloatError;

impl From<io::Error> for TradeError {
    fn from(error: io::Error) -> Self {
        TradeError::IoError(error)
    }
}

impl From<ParseFloatError> for TradeError {
    fn from(error: ParseFloatError) -> Self {
        TradeError::ParseError(error)
    }
}
```

Now you can use `?` without explicit conversion:

```rust
use std::fs::File;
use std::io::Read;

fn load_portfolio(path: &str) -> Result<f64, TradeError> {
    let mut file = File::open(path)?;  // io::Error -> TradeError automatically
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let balance: f64 = contents.trim().parse()?;  // ParseFloatError -> TradeError
    Ok(balance)
}
```

## Practical Example: Trade Order Validation

```rust
use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
enum OrderError {
    InvalidTicker(String),
    InvalidQuantity(String),
    InvalidPrice(String),
    InsufficientBalance { required: f64, available: f64 },
    MarketClosed(String),
    ParseError(String),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::InvalidTicker(t) => write!(f, "Invalid ticker: {}", t),
            OrderError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {}", msg),
            OrderError::InvalidPrice(msg) => write!(f, "Invalid price: {}", msg),
            OrderError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need ${:.2}, have ${:.2}", required, available)
            }
            OrderError::MarketClosed(market) => write!(f, "Market {} is closed", market),
            OrderError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for OrderError {}

struct Order {
    ticker: String,
    quantity: f64,
    price: f64,
    side: String,
}

fn validate_order(
    ticker: &str,
    quantity: f64,
    price: f64,
    balance: f64,
    valid_tickers: &HashMap<String, bool>,
    market_open: bool,
) -> Result<Order, OrderError> {
    // Validate ticker
    if !valid_tickers.contains_key(ticker) {
        return Err(OrderError::InvalidTicker(ticker.to_string()));
    }

    // Validate quantity
    if quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(
            "Quantity must be positive".to_string()
        ));
    }

    // Validate price
    if price <= 0.0 {
        return Err(OrderError::InvalidPrice(
            "Price must be positive".to_string()
        ));
    }

    // Check balance
    let required = quantity * price;
    if required > balance {
        return Err(OrderError::InsufficientBalance { required, available: balance });
    }

    // Check market status
    if !market_open {
        return Err(OrderError::MarketClosed("NASDAQ".to_string()));
    }

    Ok(Order {
        ticker: ticker.to_string(),
        quantity,
        price,
        side: "BUY".to_string(),
    })
}

fn main() {
    let mut valid_tickers = HashMap::new();
    valid_tickers.insert("AAPL".to_string(), true);
    valid_tickers.insert("GOOGL".to_string(), true);
    valid_tickers.insert("BTC".to_string(), true);

    let balance = 10000.0;
    let market_open = true;

    // Test 1: Valid order
    match validate_order("AAPL", 10.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(order) => println!("Order created: {} {} @ ${:.2}",
            order.quantity, order.ticker, order.price),
        Err(e) => println!("Order failed: {}", e),
    }

    // Test 2: Invalid ticker
    match validate_order("INVALID", 10.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Order failed: {}", e),
    }

    // Test 3: Insufficient balance
    match validate_order("AAPL", 1000.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Order failed: {}", e),
    }
}
```

## Combining Errors from Different Sources

```rust
use std::fmt;
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum PortfolioError {
    Io(io::Error),
    Parse(ParseFloatError),
    Validation(String),
    Calculation(String),
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortfolioError::Io(e) => write!(f, "IO error: {}", e),
            PortfolioError::Parse(e) => write!(f, "Parse error: {}", e),
            PortfolioError::Validation(msg) => write!(f, "Validation: {}", msg),
            PortfolioError::Calculation(msg) => write!(f, "Calculation: {}", msg),
        }
    }
}

impl std::error::Error for PortfolioError {}

impl From<io::Error> for PortfolioError {
    fn from(e: io::Error) -> Self {
        PortfolioError::Io(e)
    }
}

impl From<ParseFloatError> for PortfolioError {
    fn from(e: ParseFloatError) -> Self {
        PortfolioError::Parse(e)
    }
}

fn calculate_portfolio_risk(positions: &[(String, f64, f64)]) -> Result<f64, PortfolioError> {
    if positions.is_empty() {
        return Err(PortfolioError::Validation(
            "Portfolio cannot be empty".to_string()
        ));
    }

    let total_value: f64 = positions
        .iter()
        .map(|(_, qty, price)| qty * price)
        .sum();

    if total_value <= 0.0 {
        return Err(PortfolioError::Calculation(
            "Total portfolio value must be positive".to_string()
        ));
    }

    // Simple risk calculation: sum of squared weights
    let risk: f64 = positions
        .iter()
        .map(|(_, qty, price)| {
            let weight = (qty * price) / total_value;
            weight * weight
        })
        .sum();

    Ok(risk.sqrt())
}

fn main() {
    let positions = vec![
        ("AAPL".to_string(), 10.0, 150.0),
        ("GOOGL".to_string(), 5.0, 2800.0),
        ("BTC".to_string(), 0.5, 42000.0),
    ];

    match calculate_portfolio_risk(&positions) {
        Ok(risk) => println!("Portfolio risk score: {:.4}", risk),
        Err(e) => println!("Error: {}", e),
    }

    // Test with empty portfolio
    match calculate_portfolio_risk(&[]) {
        Ok(risk) => println!("Risk: {:.4}", risk),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Error Handling in Operation Chains

```rust
use std::fmt;

#[derive(Debug)]
enum TradingError {
    DataFetch(String),
    Analysis(String),
    Execution(String),
    RiskLimit(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::DataFetch(msg) => write!(f, "Data fetch failed: {}", msg),
            TradingError::Analysis(msg) => write!(f, "Analysis failed: {}", msg),
            TradingError::Execution(msg) => write!(f, "Execution failed: {}", msg),
            TradingError::RiskLimit(msg) => write!(f, "Risk limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for TradingError {}

struct MarketData {
    price: f64,
    volume: f64,
}

struct Signal {
    action: String,
    confidence: f64,
}

struct TradeResult {
    executed_price: f64,
    quantity: f64,
}

fn fetch_market_data(ticker: &str) -> Result<MarketData, TradingError> {
    // Simulate data fetching
    if ticker == "INVALID" {
        return Err(TradingError::DataFetch(
            format!("Unknown ticker: {}", ticker)
        ));
    }
    Ok(MarketData { price: 42000.0, volume: 1000000.0 })
}

fn analyze_signal(data: &MarketData) -> Result<Signal, TradingError> {
    if data.volume < 1000.0 {
        return Err(TradingError::Analysis(
            "Insufficient volume for analysis".to_string()
        ));
    }
    Ok(Signal {
        action: "BUY".to_string(),
        confidence: 0.75,
    })
}

fn check_risk_limits(signal: &Signal, max_risk: f64) -> Result<(), TradingError> {
    if signal.confidence < max_risk {
        return Err(TradingError::RiskLimit(
            format!("Confidence {:.2} below threshold {:.2}",
                signal.confidence, max_risk)
        ));
    }
    Ok(())
}

fn execute_trade(signal: &Signal, quantity: f64) -> Result<TradeResult, TradingError> {
    if quantity <= 0.0 {
        return Err(TradingError::Execution(
            "Invalid quantity".to_string()
        ));
    }
    Ok(TradeResult {
        executed_price: 42000.0,
        quantity,
    })
}

fn trading_pipeline(ticker: &str, quantity: f64, max_risk: f64) -> Result<TradeResult, TradingError> {
    let data = fetch_market_data(ticker)?;
    let signal = analyze_signal(&data)?;
    check_risk_limits(&signal, max_risk)?;
    let result = execute_trade(&signal, quantity)?;
    Ok(result)
}

fn main() {
    // Successful trade
    match trading_pipeline("BTC", 0.5, 0.7) {
        Ok(result) => println!(
            "Trade executed: {} @ ${:.2}",
            result.quantity, result.executed_price
        ),
        Err(e) => println!("Trading failed: {}", e),
    }

    // Error: invalid ticker
    match trading_pipeline("INVALID", 0.5, 0.7) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }

    // Error: risk limit exceeded
    match trading_pipeline("BTC", 0.5, 0.9) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Pattern: Error Recovery

```rust
fn get_price_with_fallback(primary: &str, fallback: &str) -> Result<f64, String> {
    // Try primary source
    match fetch_price(primary) {
        Ok(price) => return Ok(price),
        Err(e) => println!("Primary source failed: {}, trying fallback...", e),
    }

    // Try fallback source
    fetch_price(fallback)
}

fn fetch_price(source: &str) -> Result<f64, String> {
    match source {
        "binance" => Ok(42000.0),
        "coinbase" => Ok(42100.0),
        _ => Err(format!("Unknown source: {}", source)),
    }
}

fn main() {
    match get_price_with_fallback("unknown", "binance") {
        Ok(price) => println!("Price: ${:.2}", price),
        Err(e) => println!("All sources failed: {}", e),
    }
}
```

## Exercises

### Exercise 1: Transaction Validator

Create an enum `TransactionError` with variants:
- `InvalidAmount` — invalid amount
- `InvalidAddress` — invalid address
- `NetworkError` — network error
- `InsufficientFunds` — insufficient funds

Implement a `validate_transaction()` function.

### Exercise 2: Trade Data Parser

Write a function that parses a string in format `"TICKER:PRICE:VOLUME"` and returns a struct or one of the error types.

### Exercise 3: Multi-stage Validation

Create a function `process_trade_request()` that:
1. Parses input data
2. Validates the ticker
3. Checks limits
4. Calculates fees

Each stage can return its own error type.

## Homework

1. **Custom Exchange Error Type**

   Create an `ExchangeError` with variants for different exchange API errors: authentication error, rate limit, service unavailable, order error.

2. **Order Processing Pipeline**

   Implement a complete order processing pipeline with 5 stages, each of which can return its own error type.

3. **Data Aggregator**

   Write a function that requests a price from three sources and returns the result or an aggregated error.

4. **Retry with Different Errors**

   Implement a `retry_operation()` function that retries the operation only for certain error types (e.g., network errors), but not for others (e.g., validation errors).

## What We Learned

| Approach | When to Use |
|----------|-------------|
| `Box<dyn Error>` | Quick prototyping, when error type doesn't matter |
| Error `enum` | When you need to handle errors differently |
| `From` trait | For automatic conversion with `?` operator |
| Nested errors | When preserving the original error is important |

## Navigation

[← Previous day](../100-result-type-trade-execution/en.md) | [Next day →](../102-question-mark-operator/en.md)
