# Day 94: Result Deeper — Ok and Err

## Trading Analogy

Imagine you're sending an order to an exchange. There are two possible outcomes: **success** (order accepted, you receive an ID) or **error** (insufficient funds, exchange unavailable, invalid parameters). Rust's `Result<T, E>` type models exactly this pattern: an operation either returns a successful value of type `T`, or an error of type `E`.

## Anatomy of Result

```rust
enum Result<T, E> {
    Ok(T),   // Success: contains value of type T
    Err(E),  // Error: contains error of type E
}
```

`Result` is a generic enum with two variants:
- `Ok(T)` — operation succeeded, result inside
- `Err(E)` — operation failed, error reason inside

## Creating Result Values

```rust
fn main() {
    // Successful results
    let order_id: Result<u64, String> = Ok(12345);
    let balance: Result<f64, &str> = Ok(10000.50);

    // Errors
    let failed_order: Result<u64, String> = Err(String::from("Insufficient funds"));
    let api_error: Result<f64, &str> = Err("Connection timeout");

    println!("Order: {:?}", order_id);
    println!("Balance: {:?}", balance);
    println!("Failed: {:?}", failed_order);
    println!("API Error: {:?}", api_error);
}
```

## Handling Result with match

The basic and most explicit way to handle Results:

```rust
fn main() {
    let order_result = place_order("BTCUSDT", 0.1, 42000.0);

    match order_result {
        Ok(order_id) => {
            println!("Order placed successfully! ID: {}", order_id);
        }
        Err(error) => {
            println!("Failed to place order: {}", error);
        }
    }
}

fn place_order(symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    if symbol.is_empty() {
        return Err(String::from("Symbol cannot be empty"));
    }

    // Simulate successful order placement
    Ok(1234567890)
}
```

## Result Methods

### is_ok() and is_err()

```rust
fn main() {
    let success: Result<i32, &str> = Ok(42);
    let failure: Result<i32, &str> = Err("error");

    println!("Success is_ok: {}", success.is_ok());   // true
    println!("Success is_err: {}", success.is_err()); // false
    println!("Failure is_ok: {}", failure.is_ok());   // false
    println!("Failure is_err: {}", failure.is_err()); // true

    // Trading application
    let order = execute_trade("ETHUSDT", 1.0);
    if order.is_ok() {
        println!("Trade executed, proceeding with portfolio update");
    }
}

fn execute_trade(symbol: &str, qty: f64) -> Result<u64, String> {
    if qty > 0.0 {
        Ok(999)
    } else {
        Err(String::from("Invalid quantity"))
    }
}
```

### unwrap() and expect()

```rust
fn main() {
    // unwrap — panics on Err, returns value on Ok
    let price: Result<f64, &str> = Ok(42500.0);
    let value = price.unwrap();
    println!("Price: {}", value);

    // expect — same thing, but with custom message
    let balance: Result<f64, &str> = Ok(10000.0);
    let bal = balance.expect("Failed to get balance");
    println!("Balance: {}", bal);

    // DANGEROUS! This will panic:
    // let error: Result<f64, &str> = Err("API down");
    // error.unwrap(); // panic!
}
```

**Important:** Only use `unwrap()` when you're 100% sure of success or in tests!

### unwrap_or() and unwrap_or_else()

```rust
fn main() {
    let price: Result<f64, &str> = Err("Price unavailable");

    // Default value
    let safe_price = price.unwrap_or(0.0);
    println!("Safe price: {}", safe_price); // 0.0

    // Lazy evaluation of default value
    let calculated: Result<f64, &str> = Err("No data");
    let default = calculated.unwrap_or_else(|err| {
        println!("Error occurred: {}, using fallback", err);
        get_fallback_price()
    });
    println!("Default price: {}", default);
}

fn get_fallback_price() -> f64 {
    40000.0 // Fallback price
}
```

### unwrap_or_default()

```rust
fn main() {
    let result: Result<String, &str> = Err("error");
    let value = result.unwrap_or_default(); // Empty string
    println!("Value: '{}'", value);

    let num_result: Result<i32, &str> = Err("error");
    let num = num_result.unwrap_or_default(); // 0
    println!("Number: {}", num);

    // In trading: default quantity
    let qty: Result<f64, String> = fetch_position_size("BTCUSDT");
    let position = qty.unwrap_or_default(); // 0.0 on error
    println!("Position size: {}", position);
}

fn fetch_position_size(symbol: &str) -> Result<f64, String> {
    Err(format!("No position for {}", symbol))
}
```

## Transforming Result

### map() — Transform Ok

```rust
fn main() {
    let price_result: Result<f64, String> = Ok(42000.0);

    // Convert to EUR
    let eur_price = price_result.map(|usd| usd * 0.92);
    println!("Price in EUR: {:?}", eur_price); // Ok(38640.0)

    // On error, map doesn't execute
    let error: Result<f64, String> = Err(String::from("No price"));
    let mapped = error.map(|p| p * 2.0);
    println!("Mapped error: {:?}", mapped); // Err("No price")
}
```

### map_err() — Transform Err

```rust
fn main() {
    let error: Result<f64, &str> = Err("timeout");

    // Transform error type
    let detailed: Result<f64, String> = error.map_err(|e| {
        format!("API Error: {} - please retry", e)
    });
    println!("{:?}", detailed);

    // Add context to error
    let result = fetch_price("BTCUSDT")
        .map_err(|e| format!("[BTCUSDT] {}", e));
    println!("{:?}", result);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    Err(String::from("Connection refused"))
}
```

### and_then() — Chain Operations

```rust
fn main() {
    let result = validate_symbol("BTCUSDT")
        .and_then(|symbol| fetch_balance(&symbol))
        .and_then(|balance| calculate_position_size(balance, 2.0));

    match result {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error in chain: {}", e),
    }
}

fn validate_symbol(symbol: &str) -> Result<String, String> {
    if symbol.len() >= 6 {
        Ok(symbol.to_string())
    } else {
        Err(String::from("Invalid symbol format"))
    }
}

fn fetch_balance(symbol: &str) -> Result<f64, String> {
    // Simulate fetching balance
    Ok(10000.0)
}

fn calculate_position_size(balance: f64, risk_pct: f64) -> Result<f64, String> {
    if risk_pct > 0.0 && risk_pct <= 100.0 {
        Ok(balance * (risk_pct / 100.0))
    } else {
        Err(String::from("Invalid risk percentage"))
    }
}
```

### or_else() — Recover from Error

```rust
fn main() {
    let price = fetch_from_primary()
        .or_else(|_| fetch_from_secondary())
        .or_else(|_| fetch_from_cache());

    println!("Price: {:?}", price);
}

fn fetch_from_primary() -> Result<f64, String> {
    Err(String::from("Primary API down"))
}

fn fetch_from_secondary() -> Result<f64, String> {
    Err(String::from("Secondary API down"))
}

fn fetch_from_cache() -> Result<f64, String> {
    Ok(41500.0) // Cached price
}
```

## The ? Operator — Error Propagation

```rust
fn main() {
    match process_trade("BTCUSDT", 0.5, 42000.0) {
        Ok(result) => println!("Trade result: {}", result),
        Err(e) => println!("Trade failed: {}", e),
    }
}

fn process_trade(symbol: &str, qty: f64, price: f64) -> Result<String, String> {
    // The ? operator automatically returns Err if the result is an error
    let validated_symbol = validate_trading_pair(symbol)?;
    let order_id = place_limit_order(&validated_symbol, qty, price)?;
    let confirmation = confirm_order(order_id)?;

    Ok(format!("Trade confirmed: {}", confirmation))
}

fn validate_trading_pair(symbol: &str) -> Result<String, String> {
    if symbol.ends_with("USDT") || symbol.ends_with("BTC") {
        Ok(symbol.to_uppercase())
    } else {
        Err(format!("Unsupported trading pair: {}", symbol))
    }
}

fn place_limit_order(symbol: &str, qty: f64, price: f64) -> Result<u64, String> {
    if qty > 0.0 && price > 0.0 {
        Ok(1234567890)
    } else {
        Err(String::from("Invalid order parameters"))
    }
}

fn confirm_order(order_id: u64) -> Result<String, String> {
    Ok(format!("ORD-{}", order_id))
}
```

## Practical Example: Order Execution System

```rust
fn main() {
    let orders = vec![
        ("BTCUSDT", 0.1, 42000.0),
        ("ETHUSDT", 2.0, 2500.0),
        ("INVALID", 1.0, 100.0),  // Invalid symbol
        ("SOLUSDT", -1.0, 50.0),  // Negative quantity
    ];

    for (symbol, qty, price) in orders {
        match execute_order(symbol, qty, price) {
            Ok(order) => println!("{}", order),
            Err(e) => println!("REJECTED: {} - {}", symbol, e),
        }
    }
}

#[derive(Debug)]
struct OrderConfirmation {
    order_id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    total: f64,
    status: String,
}

impl std::fmt::Display for OrderConfirmation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FILLED: {} | {} @ ${:.2} | Total: ${:.2} | ID: {}",
            self.symbol, self.quantity, self.price, self.total, self.order_id
        )
    }
}

fn execute_order(symbol: &str, qty: f64, price: f64) -> Result<OrderConfirmation, String> {
    // Validate symbol
    if !symbol.ends_with("USDT") && !symbol.ends_with("BTC") {
        return Err(format!("Invalid trading pair: {}", symbol));
    }

    // Validate quantity
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }

    // Validate price
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }

    // Check minimum order size
    let total = qty * price;
    if total < 10.0 {
        return Err(format!("Order too small: ${:.2} (min: $10)", total));
    }

    // Successful execution
    Ok(OrderConfirmation {
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        quantity: qty,
        price,
        total,
        status: String::from("FILLED"),
    })
}

fn generate_order_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
```

## Custom Error Types

```rust
use std::fmt;

#[derive(Debug)]
enum TradingError {
    InsufficientFunds { required: f64, available: f64 },
    InvalidQuantity(f64),
    SymbolNotFound(String),
    RiskLimitExceeded { current: f64, max: f64 },
    NetworkError(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::InsufficientFunds { required, available } => {
                write!(f, "Insufficient funds: need ${:.2}, have ${:.2}", required, available)
            }
            TradingError::InvalidQuantity(qty) => {
                write!(f, "Invalid quantity: {}", qty)
            }
            TradingError::SymbolNotFound(symbol) => {
                write!(f, "Symbol not found: {}", symbol)
            }
            TradingError::RiskLimitExceeded { current, max } => {
                write!(f, "Risk limit exceeded: {:.2}% > {:.2}%", current, max)
            }
            TradingError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
        }
    }
}

fn main() {
    let result = open_position("BTCUSDT", 1.0, 50000.0, 5000.0);

    match result {
        Ok(position_id) => println!("Position opened: {}", position_id),
        Err(e) => println!("Error: {}", e),
    }
}

fn open_position(
    symbol: &str,
    qty: f64,
    price: f64,
    balance: f64,
) -> Result<u64, TradingError> {
    // Check symbol
    let valid_symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    if !valid_symbols.contains(&symbol) {
        return Err(TradingError::SymbolNotFound(symbol.to_string()));
    }

    // Check quantity
    if qty <= 0.0 {
        return Err(TradingError::InvalidQuantity(qty));
    }

    // Check balance
    let required = qty * price;
    if required > balance {
        return Err(TradingError::InsufficientFunds {
            required,
            available: balance,
        });
    }

    // Check risk
    let risk_percent = (required / balance) * 100.0;
    if risk_percent > 10.0 {
        return Err(TradingError::RiskLimitExceeded {
            current: risk_percent,
            max: 10.0,
        });
    }

    Ok(12345)
}
```

## Combining Result and Option

```rust
fn main() {
    // Option -> Result
    let maybe_price: Option<f64> = Some(42000.0);
    let result: Result<f64, &str> = maybe_price.ok_or("Price not found");
    println!("{:?}", result);

    // Result -> Option
    let result: Result<f64, &str> = Ok(42000.0);
    let maybe: Option<f64> = result.ok();
    println!("{:?}", maybe);

    // Practical example
    match find_and_execute("BTCUSDT") {
        Ok(price) => println!("Executed at: ${:.2}", price),
        Err(e) => println!("Failed: {}", e),
    }
}

fn find_and_execute(symbol: &str) -> Result<f64, String> {
    let price = get_best_price(symbol)
        .ok_or_else(|| format!("No price for {}", symbol))?;

    execute_at_price(symbol, price)
}

fn get_best_price(symbol: &str) -> Option<f64> {
    match symbol {
        "BTCUSDT" => Some(42000.0),
        "ETHUSDT" => Some(2500.0),
        _ => None,
    }
}

fn execute_at_price(symbol: &str, price: f64) -> Result<f64, String> {
    if price > 0.0 {
        Ok(price)
    } else {
        Err(String::from("Invalid price for execution"))
    }
}
```

## Exercises

### Exercise 1: Order Validator
Create a function `validate_order` that checks all order parameters and returns `Result<ValidatedOrder, OrderError>`.

```rust
#[derive(Debug)]
struct ValidatedOrder {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidSymbol(String),
    InvalidSide(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance,
}

fn validate_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<ValidatedOrder, OrderError> {
    // Your implementation
    todo!()
}
```

### Exercise 2: Trading Signal Parser
Write a function that parses a trading signal string into a struct:

```rust
#[derive(Debug)]
struct TradeSignal {
    action: String,
    symbol: String,
    price: f64,
}

fn parse_signal(input: &str) -> Result<TradeSignal, String> {
    // Format: "BUY BTCUSDT 42000.50"
    // Your implementation
    todo!()
}
```

### Exercise 3: Risk Management Check Chain
Implement a pre-trade check system:

```rust
fn pre_trade_checks(
    symbol: &str,
    quantity: f64,
    price: f64,
    balance: f64,
    max_risk: f64,
    max_position: f64,
) -> Result<(), String> {
    // Checks:
    // 1. Symbol is valid
    // 2. Sufficient funds
    // 3. Risk within limits
    // 4. Position size doesn't exceed maximum
    todo!()
}
```

## What We Learned

| Method | Description | Example Use Case |
|--------|-------------|------------------|
| `Ok(v)` | Create success result | `Ok(order_id)` |
| `Err(e)` | Create error | `Err("Invalid")` |
| `is_ok()` | Check for success | Conditional logic |
| `is_err()` | Check for error | Error logging |
| `unwrap()` | Extract or panic | Tests only! |
| `expect()` | Extract with message | Debugging |
| `unwrap_or()` | Default value | Safe extraction |
| `map()` | Transform Ok | Currency conversion |
| `map_err()` | Transform Err | Adding context |
| `and_then()` | Chain operations | Order workflow |
| `?` | Propagate error | Clean code |

## Homework

1. **Order Execution System**: Create a complete system with order types (market, limit, stop), validation, and execution. Each stage should return `Result`.

2. **Exchange API Parser**: Write functions to parse JSON responses from an exchange with handling for all possible errors (invalid format, missing fields, invalid values).

3. **Risk Calculator**: Implement a position sizing function with checks:
   - Input validation
   - Balance sufficiency
   - Risk limit compliance
   - Minimum order size

4. **Error Handler with Retry**: Create a wrapper that retries an operation N times for certain error types (network), but immediately returns error for others (validation).

## Navigation

[← Previous day](../093-result-introduction/en.md) | [Next day →](../095-question-mark-operator/en.md)
