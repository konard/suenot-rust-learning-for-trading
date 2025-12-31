# Day 99: map_err — Transforming Error Type

## Trading Analogy

Imagine you have several market data sources: an exchange, a broker, and a news service. Each reports errors in its own format:
- Exchange: "CONNECTION_TIMEOUT_ERR_5001"
- Broker: "Session expired, code 401"
- News service: "Rate limit exceeded"

When building a unified trading system, you need to **transform** all these different errors into a single format that your system understands. The `map_err` method in Rust does exactly this — it transforms one error type into another.

## map_err Signature

```rust
impl<T, E> Result<T, E> {
    fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F
}
```

**Important:** `map_err` only transforms the error (`Err`), leaving the success value (`Ok`) unchanged.

## Basic Usage

```rust
fn main() {
    // Parsing a price may return ParseFloatError
    let price_str = "42500.50";
    let result: Result<f64, String> = price_str
        .parse::<f64>()
        .map_err(|e| format!("Price parsing error: {}", e));

    println!("{:?}", result); // Ok(42500.5)

    // Invalid format
    let bad_price = "not_a_price";
    let result: Result<f64, String> = bad_price
        .parse::<f64>()
        .map_err(|e| format!("Price parsing error: {}", e));

    println!("{:?}", result); // Err("Price parsing error: invalid float literal")
}
```

## Transforming Exchange API Errors

```rust
use std::fmt;

// Errors from different sources
#[derive(Debug)]
enum ExchangeError {
    ConnectionFailed(String),
    RateLimited(u32),
    InvalidSymbol(String),
}

#[derive(Debug)]
enum BrokerError {
    SessionExpired,
    InsufficientFunds(f64),
    OrderRejected(String),
}

// Our unified error type for the trading system
#[derive(Debug)]
enum TradingError {
    DataSource(String),
    Execution(String),
    Validation(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::DataSource(msg) => write!(f, "Data source error: {}", msg),
            TradingError::Execution(msg) => write!(f, "Execution error: {}", msg),
            TradingError::Validation(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

// Transform exchange errors
fn exchange_error_to_trading(e: ExchangeError) -> TradingError {
    match e {
        ExchangeError::ConnectionFailed(msg) =>
            TradingError::DataSource(format!("Exchange unavailable: {}", msg)),
        ExchangeError::RateLimited(seconds) =>
            TradingError::DataSource(format!("Rate limit exceeded, wait {} sec", seconds)),
        ExchangeError::InvalidSymbol(sym) =>
            TradingError::Validation(format!("Unknown ticker: {}", sym)),
    }
}

// Transform broker errors
fn broker_error_to_trading(e: BrokerError) -> TradingError {
    match e {
        BrokerError::SessionExpired =>
            TradingError::DataSource("Session expired, re-authentication required".to_string()),
        BrokerError::InsufficientFunds(required) =>
            TradingError::Execution(format!("Insufficient funds, required: ${:.2}", required)),
        BrokerError::OrderRejected(reason) =>
            TradingError::Execution(format!("Order rejected: {}", reason)),
    }
}

fn get_price_from_exchange(symbol: &str) -> Result<f64, ExchangeError> {
    if symbol == "INVALID" {
        Err(ExchangeError::InvalidSymbol(symbol.to_string()))
    } else {
        Ok(42500.0)
    }
}

fn place_order_at_broker(symbol: &str, qty: f64, balance: f64) -> Result<String, BrokerError> {
    let required = 42500.0 * qty;
    if balance < required {
        Err(BrokerError::InsufficientFunds(required))
    } else {
        Ok(format!("ORDER-{}-{}", symbol, qty))
    }
}

fn main() {
    // Use map_err for transformation
    let price_result: Result<f64, TradingError> = get_price_from_exchange("BTCUSD")
        .map_err(exchange_error_to_trading);
    println!("Price: {:?}", price_result);

    let invalid_result: Result<f64, TradingError> = get_price_from_exchange("INVALID")
        .map_err(exchange_error_to_trading);
    println!("Error: {:?}", invalid_result);

    let order_result: Result<String, TradingError> = place_order_at_broker("BTCUSD", 1.0, 1000.0)
        .map_err(broker_error_to_trading);
    println!("Order: {:?}", order_result);
}
```

## Chaining with map_err

```rust
fn main() {
    let result = parse_and_validate_order("BTCUSD", "0.5", "42000.50");
    println!("{:?}", result);

    let bad_result = parse_and_validate_order("BTCUSD", "abc", "42000.50");
    println!("{:?}", bad_result);
}

#[derive(Debug)]
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidQuantity(String),
    InvalidPrice(String),
    InvalidSymbol(String),
}

fn parse_and_validate_order(
    symbol: &str,
    qty_str: &str,
    price_str: &str,
) -> Result<Order, OrderError> {
    // Each parse transforms its error to the appropriate variant
    let quantity: f64 = qty_str
        .parse()
        .map_err(|e| OrderError::InvalidQuantity(format!("{}: {}", qty_str, e)))?;

    let price: f64 = price_str
        .parse()
        .map_err(|e| OrderError::InvalidPrice(format!("{}: {}", price_str, e)))?;

    if symbol.is_empty() {
        return Err(OrderError::InvalidSymbol("Empty ticker".to_string()));
    }

    Ok(Order {
        symbol: symbol.to_string(),
        quantity,
        price,
    })
}
```

## Transforming to String with Context

```rust
fn main() {
    let prices = vec!["42500.0", "invalid", "43000.0"];

    for (i, price_str) in prices.iter().enumerate() {
        let result = parse_price_with_context(price_str, i);
        match result {
            Ok(price) => println!("Price {}: ${:.2}", i, price),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn parse_price_with_context(s: &str, index: usize) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|e| format!("Failed to parse price #{} '{}': {}", index, s, e))
}
```

## map_err with Closures and Context Capture

```rust
fn main() {
    let symbol = "ETHUSD";
    let source = "Binance";

    let result = fetch_price_with_context(symbol, source);
    println!("{:?}", result);
}

fn fetch_price(symbol: &str) -> Result<f64, std::io::Error> {
    // Simulate an I/O error
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Server unavailable"
    ))
}

fn fetch_price_with_context(symbol: &str, source: &str) -> Result<f64, String> {
    // Closure captures symbol and source to form an informative error
    fetch_price(symbol)
        .map_err(|e| format!(
            "[{}] Error fetching price for {}: {} ({})",
            source, symbol, e, e.kind()
        ))
}
```

## Practical Example: Loading a Portfolio

```rust
use std::collections::HashMap;

fn main() {
    let portfolio_data = r#"
        BTCUSD:0.5
        ETHUSD:2.0
        SOLUSD:10.0
    "#;

    match load_portfolio(portfolio_data) {
        Ok(portfolio) => {
            println!("Portfolio loaded:");
            for (symbol, qty) in &portfolio {
                println!("  {}: {} units", symbol, qty);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example with error
    let bad_data = "BTCUSD:not_a_number";
    match load_portfolio(bad_data) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}

#[derive(Debug)]
enum PortfolioError {
    ParseError { line: usize, details: String },
    EmptyPortfolio,
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioError::ParseError { line, details } =>
                write!(f, "Parse error on line {}: {}", line, details),
            PortfolioError::EmptyPortfolio =>
                write!(f, "Portfolio is empty"),
        }
    }
}

fn load_portfolio(data: &str) -> Result<HashMap<String, f64>, PortfolioError> {
    let mut portfolio = HashMap::new();

    for (line_num, line) in data.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (symbol, qty) = parse_portfolio_line(line, line_num + 1)?;
        portfolio.insert(symbol, qty);
    }

    if portfolio.is_empty() {
        return Err(PortfolioError::EmptyPortfolio);
    }

    Ok(portfolio)
}

fn parse_portfolio_line(line: &str, line_num: usize) -> Result<(String, f64), PortfolioError> {
    let parts: Vec<&str> = line.split(':').collect();

    if parts.len() != 2 {
        return Err(PortfolioError::ParseError {
            line: line_num,
            details: format!("Expected format 'SYMBOL:QTY', got '{}'", line),
        });
    }

    let symbol = parts[0].to_string();
    let qty = parts[1]
        .parse::<f64>()
        // Transform ParseFloatError to PortfolioError with context
        .map_err(|e| PortfolioError::ParseError {
            line: line_num,
            details: format!("Invalid quantity '{}': {}", parts[1], e),
        })?;

    Ok((symbol, qty))
}
```

## map_err vs and_then for Errors

```rust
fn main() {
    // map_err — only transforms the error
    let result1: Result<i32, String> = "42".parse::<i32>()
        .map_err(|e| format!("Error: {}", e));

    // and_then — allows returning a new error based on the success value
    let result2: Result<i32, String> = "42".parse::<i32>()
        .map_err(|e| format!("Error: {}", e))
        .and_then(|n| {
            if n > 0 {
                Ok(n)
            } else {
                Err("Number must be positive".to_string())
            }
        });

    println!("result1: {:?}", result1);
    println!("result2: {:?}", result2);
}
```

## Example: Risk Management with Different Error Sources

```rust
fn main() {
    match execute_trade("BTCUSD", 0.5, 50000.0) {
        Ok(order_id) => println!("Trade executed: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }

    // Simulate validation error
    match execute_trade("BTCUSD", -0.5, 50000.0) {
        Ok(order_id) => println!("Trade executed: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }
}

#[derive(Debug)]
enum TradeError {
    Validation(String),
    RiskLimit(String),
    Execution(String),
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeError::Validation(msg) => write!(f, "Validation: {}", msg),
            TradeError::RiskLimit(msg) => write!(f, "Risk limit: {}", msg),
            TradeError::Execution(msg) => write!(f, "Execution: {}", msg),
        }
    }
}

fn validate_quantity(qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        Err(format!("Quantity must be positive, got: {}", qty))
    } else if qty > 100.0 {
        Err(format!("Maximum volume exceeded: {}", qty))
    } else {
        Ok(qty)
    }
}

fn check_risk_limits(symbol: &str, qty: f64, price: f64) -> Result<(), String> {
    let max_position_value = 100_000.0;
    let position_value = qty * price;

    if position_value > max_position_value {
        Err(format!(
            "Position ${:.2} exceeds limit ${:.2}",
            position_value, max_position_value
        ))
    } else {
        Ok(())
    }
}

fn send_order(symbol: &str, qty: f64, price: f64) -> Result<String, std::io::Error> {
    // Successful execution
    Ok(format!("ORD-{}-{}-{}", symbol, qty, price))
}

fn execute_trade(symbol: &str, qty: f64, price: f64) -> Result<String, TradeError> {
    // Validation — transform String to TradeError::Validation
    let validated_qty = validate_quantity(qty)
        .map_err(TradeError::Validation)?;

    // Risk check — transform String to TradeError::RiskLimit
    check_risk_limits(symbol, validated_qty, price)
        .map_err(TradeError::RiskLimit)?;

    // Send order — transform io::Error to TradeError::Execution
    send_order(symbol, validated_qty, price)
        .map_err(|e| TradeError::Execution(e.to_string()))
}
```

## What We Learned

| Aspect | Description |
|--------|-------------|
| `map_err` | Transforms error type in `Result` |
| Signature | `Result<T, E> -> Result<T, F>` |
| When to use | When combining different error sources |
| Closures | Can capture context for informative messages |
| Chaining | Easily combines with `?` operator |

## Practical Exercises

1. **Market Data Parser**: Write a function that parses a string in format `"BTCUSD,42500.00,0.5"` and transforms all parsing errors into a unified `MarketDataError` type with information about which field failed to parse.

2. **Error Unification**: Create three functions simulating calls to different APIs (exchange, broker, analytics). Each returns its own error type. Write a wrapper function that uses `map_err` to convert all errors to a unified type.

3. **Configuration Loader**: Implement loading of trading configuration from a file. Use `map_err` to add context (filename, line number) to parsing errors.

4. **Multi-step Validation**: Create an order validation function that checks symbol, quantity, price, and balance. Each check can return an error — transform them all to `OrderValidationError` with a specific description of the problem.

## Homework

1. Implement a system for loading historical data from multiple sources (file, API, cache). Each source has its own error type. Use `map_err` for unification and adding source information.

2. Create a position size calculator with full validation of all input parameters. All errors should be transformed to `PositionSizeError` with a detailed description of the problem.

3. Write a function that reads and parses a trade history file. Use `map_err` to:
   - Transform file reading errors
   - Add line numbers to parsing errors
   - Create clear error messages

4. Implement a trading signal processing chain: data retrieval → validation → risk check → execution. Each step can return a different error type — unify them using `map_err`.

## Navigation

[← Previous day](../098-result-method-chaining/en.md) | [Next day →](../100-ok-or-converting-option-to-result/en.md)
