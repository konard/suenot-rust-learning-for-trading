# Day 76: Result Methods — Handling Errors in Trading

## Trading Analogy

Imagine you send an order to an exchange. The exchange can respond with success (order executed) or an error (insufficient funds, invalid price, exchange unavailable). The `Result` type in Rust is like a response from the exchange: either `Ok(data)` or `Err(error)`. Result methods are different strategies for handling these responses:

- **unwrap** — "I'm sure the order will execute, otherwise — panic!"
- **unwrap_or** — "If the order fails, use a fallback value"
- **map** — "Transform the successful result"
- **?** — "Pass the error to the calling function"

## Basic Methods: is_ok() and is_err()

```rust
fn main() {
    let order_result: Result<f64, String> = execute_order("BTC", 0.1, 42000.0);

    // Status check
    if order_result.is_ok() {
        println!("Order successfully executed!");
    }

    if order_result.is_err() {
        println!("Order rejected!");
    }

    // Another example
    let balance_check = check_balance(1000.0, 500.0);
    println!("Balance sufficient: {}", balance_check.is_ok());
}

fn execute_order(symbol: &str, qty: f64, price: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    Ok(qty * price)  // Return order value
}

fn check_balance(required: f64, available: f64) -> Result<(), String> {
    if available >= required {
        Ok(())
    } else {
        Err(format!("Insufficient funds: need {}, have {}", required, available))
    }
}
```

## unwrap() and expect() — When Confident of Success

```rust
fn main() {
    // unwrap — panics if Err
    let price = parse_price("42000.50").unwrap();
    println!("Price: {}", price);

    // expect — panics with custom message
    let volume = parse_volume("1.5")
        .expect("Critical error: failed to parse volume");
    println!("Volume: {}", volume);

    // DANGEROUS! This will panic:
    // let bad_price = parse_price("invalid").unwrap();
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", s))
}

fn parse_volume(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as volume", s))
}
```

**When to use:**
- In tests
- When absolutely certain error is impossible
- In prototypes (but replace with proper handling later!)

## unwrap_or() — Default Value

```rust
fn main() {
    // If error — use default value
    let price = fetch_price("BTC").unwrap_or(0.0);
    println!("BTC price: ${}", price);

    // Useful for optional settings
    let risk_percent = parse_config_value("risk_percent").unwrap_or(2.0);
    println!("Risk: {}%", risk_percent);

    // For orders with fallback
    let filled_qty = execute_market_order("ETH", 1.0).unwrap_or(0.0);
    if filled_qty == 0.0 {
        println!("Order not executed, skipping");
    } else {
        println!("Filled: {} ETH", filled_qty);
    }
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Simulating price fetch from exchange
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}

fn parse_config_value(key: &str) -> Result<f64, String> {
    // Simulating config read
    Err(format!("Key '{}' not found", key))
}

fn execute_market_order(symbol: &str, qty: f64) -> Result<f64, String> {
    if symbol == "ETH" {
        Ok(qty)
    } else {
        Err(String::from("Pair unavailable"))
    }
}
```

## unwrap_or_else() — Lazy Evaluation

```rust
fn main() {
    // Closure only executed on error
    let price = fetch_live_price("BTC")
        .unwrap_or_else(|e| {
            println!("Error fetching price: {}, using cache", e);
            get_cached_price("BTC")
        });
    println!("Price: ${}", price);

    // Useful when default is expensive to compute
    let position_size = calculate_position(10000.0, 42000.0)
        .unwrap_or_else(|_| calculate_safe_position());
    println!("Position size: {}", position_size);
}

fn fetch_live_price(_symbol: &str) -> Result<f64, String> {
    Err(String::from("Exchange unavailable"))
}

fn get_cached_price(_symbol: &str) -> f64 {
    41500.0  // Cached value
}

fn calculate_position(balance: f64, price: f64) -> Result<f64, String> {
    if price == 0.0 {
        return Err(String::from("Price cannot be zero"));
    }
    Ok(balance / price)
}

fn calculate_safe_position() -> f64 {
    println!("Computing safe position...");
    0.01  // Minimum safe position
}
```

## unwrap_or_default() — For Types with Default

```rust
fn main() {
    // For numbers default = 0
    let volume: f64 = parse_volume("invalid").unwrap_or_default();
    println!("Volume: {}", volume);  // 0.0

    // For String default = ""
    let symbol: String = get_symbol(999).unwrap_or_default();
    println!("Symbol: '{}'", symbol);  // ""

    // For Vec default = []
    let trades: Vec<f64> = get_recent_trades("UNKNOWN").unwrap_or_default();
    println!("Trades: {:?}", trades);  // []

    // For bool default = false
    let is_active: bool = check_market_active("NYSE").unwrap_or_default();
    println!("Market active: {}", is_active);  // false
}

fn parse_volume(s: &str) -> Result<f64, String> {
    s.parse().map_err(|_| String::from("Parse error"))
}

fn get_symbol(id: u32) -> Result<String, String> {
    Err(format!("Symbol with id {} not found", id))
}

fn get_recent_trades(_symbol: &str) -> Result<Vec<f64>, String> {
    Err(String::from("No data"))
}

fn check_market_active(_market: &str) -> Result<bool, String> {
    Err(String::from("Market unavailable"))
}
```

## map() — Transforming Successful Result

```rust
fn main() {
    // Transform price to string for display
    let formatted = fetch_price("BTC")
        .map(|price| format!("${:.2}", price));
    println!("{:?}", formatted);  // Ok("$42000.00")

    // Chain of transformations
    let trade_value = parse_trade_data("100.0,42000.0")
        .map(|(qty, price)| qty * price)
        .map(|value| value * 1.001);  // Add commission
    println!("Value with commission: {:?}", trade_value);

    // PnL calculation
    let pnl = calculate_exit_value(42000.0, 43500.0, 0.5)
        .map(|gross| gross - 50.0);  // Minus commission
    println!("Net PnL: {:?}", pnl);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(String::from("Unknown symbol")),
    }
}

fn parse_trade_data(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(String::from("Invalid format"));
    }
    let qty = parts[0].parse().map_err(|_| String::from("Qty error"))?;
    let price = parts[1].parse().map_err(|_| String::from("Price error"))?;
    Ok((qty, price))
}

fn calculate_exit_value(entry: f64, exit: f64, qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    Ok((exit - entry) * qty)
}
```

## map_err() — Transforming Error

```rust
fn main() {
    // Add context to error
    let result = fetch_order_book("BTC")
        .map_err(|e| format!("Order book loading error: {}", e));
    println!("{:?}", result);

    // Transform error type
    let trade_result = execute_trade("ETH", 1.0)
        .map_err(TradeError::from);
    println!("{:?}", trade_result);

    // Error logging
    let price = get_market_price("DOGE")
        .map_err(|e| {
            eprintln!("[ERROR] {}", e);
            e
        });
    println!("Result: {:?}", price);
}

#[derive(Debug)]
enum TradeError {
    NetworkError(String),
    ValidationError(String),
    InsufficientFunds(String),
}

impl From<String> for TradeError {
    fn from(s: String) -> Self {
        TradeError::NetworkError(s)
    }
}

fn fetch_order_book(_symbol: &str) -> Result<Vec<(f64, f64)>, String> {
    Err(String::from("Connection timeout"))
}

fn execute_trade(_symbol: &str, _qty: f64) -> Result<String, String> {
    Err(String::from("Exchange unavailable"))
}

fn get_market_price(_symbol: &str) -> Result<f64, String> {
    Err(String::from("Symbol not found"))
}
```

## and_then() — Chaining Result Operations

```rust
fn main() {
    // Chain of validations and operations
    let result = validate_symbol("BTC")
        .and_then(|s| fetch_price(&s))
        .and_then(|price| calculate_order_value(price, 0.1));

    match result {
        Ok(value) => println!("Order value: ${:.2}", value),
        Err(e) => println!("Error: {}", e),
    }

    // Full order processing pipeline
    let order_result = parse_order_request("BTC,0.5,42000")
        .and_then(|(symbol, qty, price)| validate_order(&symbol, qty, price))
        .and_then(|(symbol, qty, price)| check_balance_for_order(&symbol, qty, price))
        .and_then(|(symbol, qty, price)| place_order(&symbol, qty, price));

    println!("Order result: {:?}", order_result);
}

fn validate_symbol(symbol: &str) -> Result<String, String> {
    if symbol.len() >= 2 && symbol.len() <= 10 {
        Ok(symbol.to_uppercase())
    } else {
        Err(String::from("Invalid symbol format"))
    }
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Price for {} unavailable", symbol)),
    }
}

fn calculate_order_value(price: f64, qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    Ok(price * qty)
}

fn parse_order_request(s: &str) -> Result<(String, f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(String::from("Format: SYMBOL,QTY,PRICE"));
    }
    let symbol = parts[0].to_string();
    let qty = parts[1].parse().map_err(|_| String::from("Invalid quantity"))?;
    let price = parts[2].parse().map_err(|_| String::from("Invalid price"))?;
    Ok((symbol, qty, price))
}

fn validate_order(symbol: &str, qty: f64, price: f64) -> Result<(String, f64, f64), String> {
    if qty <= 0.0 || price <= 0.0 {
        return Err(String::from("Quantity and price must be positive"));
    }
    Ok((symbol.to_string(), qty, price))
}

fn check_balance_for_order(_symbol: &str, qty: f64, price: f64) -> Result<(String, f64, f64), String> {
    let required = qty * price;
    let balance = 50000.0;  // Simulated balance
    if required > balance {
        return Err(format!("Insufficient funds: need {}, have {}", required, balance));
    }
    Ok((_symbol.to_string(), qty, price))
}

fn place_order(symbol: &str, qty: f64, price: f64) -> Result<String, String> {
    Ok(format!("ORDER_{}_{:.4}@{:.2}", symbol, qty, price))
}
```

## The ? Operator — Elegant Error Propagation

```rust
fn main() {
    match execute_trading_strategy() {
        Ok(pnl) => println!("Strategy completed. PnL: ${:.2}", pnl),
        Err(e) => println!("Strategy error: {}", e),
    }
}

fn execute_trading_strategy() -> Result<f64, String> {
    // ? automatically returns Err if operation fails
    let btc_price = get_price("BTC")?;
    let eth_price = get_price("ETH")?;

    let signal = analyze_spread(btc_price, eth_price)?;

    let position = if signal > 0.0 {
        open_position("BTC", 0.1, btc_price)?
    } else {
        open_position("ETH", 1.0, eth_price)?
    };

    let exit_price = get_exit_price(&position.symbol)?;
    let pnl = calculate_pnl(&position, exit_price)?;

    Ok(pnl)
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Price for {} unavailable", symbol)),
    }
}

fn analyze_spread(price1: f64, price2: f64) -> Result<f64, String> {
    if price1 == 0.0 || price2 == 0.0 {
        return Err(String::from("Prices cannot be zero"));
    }
    Ok(price1 / price2 - 16.8)  // Deviation from mean signal
}

struct Position {
    symbol: String,
    qty: f64,
    entry_price: f64,
}

fn open_position(symbol: &str, qty: f64, price: f64) -> Result<Position, String> {
    Ok(Position {
        symbol: symbol.to_string(),
        qty,
        entry_price: price,
    })
}

fn get_exit_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43000.0),
        "ETH" => Ok(2600.0),
        _ => Err(format!("Exit price for {} unavailable", symbol)),
    }
}

fn calculate_pnl(position: &Position, exit_price: f64) -> Result<f64, String> {
    Ok((exit_price - position.entry_price) * position.qty)
}
```

## ok() and err() — Converting to Option

```rust
fn main() {
    // ok() — extracts Ok value as Some, Err becomes None
    let price: Option<f64> = fetch_price("BTC").ok();
    println!("Price: {:?}", price);  // Some(42000.0)

    let bad_price: Option<f64> = fetch_price("INVALID").ok();
    println!("Bad price: {:?}", bad_price);  // None

    // err() — extracts Err value as Some, Ok becomes None
    let error: Option<String> = fetch_price("INVALID").err();
    println!("Error: {:?}", error);  // Some("...")

    // Useful with Option methods
    let default_price = fetch_price("UNKNOWN")
        .ok()
        .unwrap_or(0.0);
    println!("Price with default: {}", default_price);

    // Combining with filter
    let valid_price = fetch_price("BTC")
        .ok()
        .filter(|&p| p > 10000.0);
    println!("Valid price: {:?}", valid_price);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}
```

## Practical Example: Trading Engine with Error Handling

```rust
fn main() {
    let engine = TradingEngine::new(10000.0);

    // Execute multiple orders
    let orders = vec![
        ("BTC", 0.1, 42000.0),
        ("ETH", 2.0, 2500.0),
        ("INVALID", 1.0, 100.0),
        ("BTC", 0.05, 42500.0),
    ];

    for (symbol, qty, price) in orders {
        match engine.process_order(symbol, qty, price) {
            Ok(order_id) => println!("✓ Order executed: {}", order_id),
            Err(e) => println!("✗ Error: {}", e),
        }
    }

    // Batch processing with collect
    let results: Vec<Result<String, String>> = vec![
        engine.process_order("BTC", 0.1, 42000.0),
        engine.process_order("ETH", 1.0, 2500.0),
    ];

    let successful: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    println!("\nSuccessful orders: {:?}", successful);
}

struct TradingEngine {
    balance: f64,
}

impl TradingEngine {
    fn new(balance: f64) -> Self {
        TradingEngine { balance }
    }

    fn process_order(&self, symbol: &str, qty: f64, price: f64) -> Result<String, String> {
        // Validate symbol
        self.validate_symbol(symbol)?;

        // Validate quantity and price
        self.validate_order_params(qty, price)?;

        // Check balance
        self.check_sufficient_balance(qty, price)?;

        // Check limits
        self.check_order_limits(qty, price)?;

        // Generate order ID
        Ok(format!("ORD-{}-{}", symbol, chrono_mock()))
    }

    fn validate_symbol(&self, symbol: &str) -> Result<(), String> {
        let valid_symbols = ["BTC", "ETH", "SOL", "DOGE"];
        if valid_symbols.contains(&symbol) {
            Ok(())
        } else {
            Err(format!("Unsupported symbol: {}", symbol))
        }
    }

    fn validate_order_params(&self, qty: f64, price: f64) -> Result<(), String> {
        if qty <= 0.0 {
            return Err(String::from("Quantity must be positive"));
        }
        if price <= 0.0 {
            return Err(String::from("Price must be positive"));
        }
        Ok(())
    }

    fn check_sufficient_balance(&self, qty: f64, price: f64) -> Result<(), String> {
        let required = qty * price;
        if required > self.balance {
            Err(format!(
                "Insufficient funds: need ${:.2}, available ${:.2}",
                required, self.balance
            ))
        } else {
            Ok(())
        }
    }

    fn check_order_limits(&self, qty: f64, price: f64) -> Result<(), String> {
        let value = qty * price;
        if value > 100000.0 {
            Err(String::from("Order limit exceeded ($100,000)"))
        } else {
            Ok(())
        }
    }
}

fn chrono_mock() -> u64 {
    1234567890  // Simulated timestamp
}
```

## Result Methods Comparison

| Method | Description | Use Case |
|--------|-------------|----------|
| `is_ok()` | Checks if contains Ok | Conditional checks |
| `is_err()` | Checks if contains Err | Conditional checks |
| `unwrap()` | Extracts Ok or panics | Tests, prototypes |
| `expect(msg)` | Extracts Ok or panics with message | Critical errors |
| `unwrap_or(val)` | Ok or default value | Simple fallbacks |
| `unwrap_or_else(f)` | Ok or computed value | Complex fallbacks |
| `unwrap_or_default()` | Ok or Default::default() | Zero values |
| `ok()` | Result -> Option (Ok) | Ignoring errors |
| `err()` | Result -> Option (Err) | Extracting errors |
| `map(f)` | Transforms Ok value | Data transformation |
| `map_err(f)` | Transforms Err value | Error enrichment |
| `and_then(f)` | Chain Result operations | Processing pipeline |
| `?` | Propagate error | Elegant code |

## Homework

1. **Order Validator**: Write a function `validate_order_chain(symbol: &str, qty: f64, price: f64, balance: f64) -> Result<Order, OrderError>` that uses `and_then` for a chain of validations (symbol, quantity, price, balance).

2. **Market Data Parser**: Create a function `parse_market_data(json: &str) -> Result<MarketData, ParseError>` that parses a data string and uses `map` and `map_err` for transformations.

3. **Trading Strategy with ?**: Implement a function `execute_strategy() -> Result<TradeResult, StrategyError>` that uses the `?` operator for sequential execution: get price -> analyze -> open position -> close.

4. **Portfolio Processor**: Write a function `process_portfolio(positions: Vec<&str>) -> Vec<Result<PositionValue, String>>` that processes a list of positions and returns results for each, using `map` for transforming successful ones and `filter_map` for collecting successes.

## Navigation

[← Previous day](../075-result-error-handling/en.md) | [Next day →](../077-custom-error-types/en.md)
