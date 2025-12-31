# Day 75: Result — Operation Can Fail

## Trading Analogy

Imagine you're sending an order to an exchange. What could go wrong?

- **Insufficient funds** in your balance
- **Market is closed** at the moment
- **Price has moved** too far from your requested price
- **Position limit** exceeded
- **Network error** while sending

In real trading, every operation can either **succeed** or **fail with an error**. Rust's `Result<T, E>` type perfectly models this situation: an operation returns either a successful result of type `T`, or an error of type `E`.

## Theory

### Result Definition

```rust
enum Result<T, E> {
    Ok(T),    // Success, contains value of type T
    Err(E),   // Error, contains value of type E
}
```

Unlike `Option`, which says "value exists or not", `Result` says "operation succeeded or failed with a specific reason".

### Basic Usage

```rust
fn main() {
    // Attempt to execute an order
    match execute_order("BTC/USDT", 0.5, 42000.0, 1000.0) {
        Ok(order_id) => println!("Order executed! ID: {}", order_id),
        Err(error) => println!("Error: {}", error),
    }
}

fn execute_order(
    pair: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    let required = quantity * price;

    if required > balance {
        return Err(format!(
            "Insufficient funds: need {:.2}, available {:.2}",
            required, balance
        ));
    }

    // Successful execution
    Ok(format!("ORD-{}-{}", pair.replace("/", ""), 12345))
}
```

## Creating Result

### Returning Ok and Err

```rust
fn validate_order_price(price: f64, min: f64, max: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    if price < min {
        return Err(format!("Price {:.2} is below minimum {:.2}", price, min));
    }
    if price > max {
        return Err(format!("Price {:.2} is above maximum {:.2}", price, max));
    }
    Ok(price)
}

fn main() {
    println!("{:?}", validate_order_price(42000.0, 40000.0, 45000.0)); // Ok(42000.0)
    println!("{:?}", validate_order_price(-100.0, 40000.0, 45000.0));  // Err(...)
    println!("{:?}", validate_order_price(50000.0, 40000.0, 45000.0)); // Err(...)
}
```

### Typed Errors

```rust
#[derive(Debug)]
enum OrderError {
    InsufficientFunds { required: f64, available: f64 },
    InvalidPrice { price: f64, reason: String },
    MarketClosed,
    RateLimitExceeded,
    NetworkError(String),
}

fn place_market_order(
    symbol: &str,
    quantity: f64,
    balance: f64,
    market_open: bool,
) -> Result<u64, OrderError> {
    if !market_open {
        return Err(OrderError::MarketClosed);
    }

    // Assume current price is 42000
    let price = 42000.0;
    let required = quantity * price;

    if required > balance {
        return Err(OrderError::InsufficientFunds {
            required,
            available: balance,
        });
    }

    // Return order ID
    Ok(123456789)
}

fn main() {
    match place_market_order("BTCUSDT", 1.0, 50000.0, true) {
        Ok(id) => println!("Order placed: {}", id),
        Err(OrderError::InsufficientFunds { required, available }) => {
            println!("Insufficient funds: need {}, have {}", required, available);
        }
        Err(OrderError::MarketClosed) => {
            println!("Market is closed, try again later");
        }
        Err(e) => println!("Other error: {:?}", e),
    }
}
```

## Handling Result

### Using match

```rust
fn calculate_position_value(
    quantity: f64,
    price: f64,
) -> Result<f64, String> {
    if quantity < 0.0 {
        return Err(String::from("Quantity cannot be negative"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    Ok(quantity * price)
}

fn main() {
    let result = calculate_position_value(0.5, 42000.0);

    match result {
        Ok(value) => println!("Position value: ${:.2}", value),
        Err(msg) => println!("Calculation error: {}", msg),
    }
}
```

### unwrap and expect Methods

```rust
fn main() {
    // unwrap — panics on error (use carefully!)
    let value = calculate_position_value(0.5, 42000.0).unwrap();
    println!("Value: {}", value);

    // expect — panics with custom message
    let value = calculate_position_value(0.5, 42000.0)
        .expect("Failed to calculate position value");
    println!("Value: {}", value);
}

fn calculate_position_value(qty: f64, price: f64) -> Result<f64, String> {
    if qty <= 0.0 || price <= 0.0 {
        return Err(String::from("Invalid input"));
    }
    Ok(qty * price)
}
```

### unwrap_or and unwrap_or_else Methods

```rust
fn get_current_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTCUSDT" => Ok(42000.0),
        "ETHUSDT" => Ok(2500.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}

fn main() {
    // unwrap_or — default value
    let btc_price = get_current_price("BTCUSDT").unwrap_or(0.0);
    let unknown_price = get_current_price("UNKNOWN").unwrap_or(0.0);

    println!("BTC: {}, Unknown: {}", btc_price, unknown_price);

    // unwrap_or_else — lazy default value computation
    let price = get_current_price("UNKNOWN").unwrap_or_else(|err| {
        println!("Warning: {}", err);
        0.0 // Return default value
    });
}
```

### The map Method

```rust
fn parse_price(input: &str) -> Result<f64, String> {
    input.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", input))
}

fn main() {
    // map — transform successful value
    let doubled = parse_price("42000.0")
        .map(|price| price * 2.0);

    println!("{:?}", doubled); // Ok(84000.0)

    // Chaining map
    let formatted = parse_price("42000.0")
        .map(|price| price * 1.1)  // Add 10%
        .map(|price| format!("${:.2}", price));

    println!("{:?}", formatted); // Ok("$46200.00")
}
```

### The and_then Method (flatMap)

```rust
fn get_balance(account: &str) -> Result<f64, String> {
    match account {
        "main" => Ok(10000.0),
        "trading" => Ok(5000.0),
        _ => Err(format!("Account '{}' not found", account)),
    }
}

fn calculate_max_position(balance: f64, risk_percent: f64) -> Result<f64, String> {
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Risk percent must be between 0 and 100"));
    }
    Ok(balance * (risk_percent / 100.0))
}

fn main() {
    // and_then — chain operations, each can fail
    let max_position = get_balance("trading")
        .and_then(|balance| calculate_max_position(balance, 5.0));

    println!("{:?}", max_position); // Ok(250.0)

    // If first operation fails
    let max_position = get_balance("unknown")
        .and_then(|balance| calculate_max_position(balance, 5.0));

    println!("{:?}", max_position); // Err("Account 'unknown' not found")
}
```

## The ? Operator — Error Propagation

```rust
#[derive(Debug)]
struct TradeResult {
    order_id: u64,
    executed_price: f64,
    quantity: f64,
    fees: f64,
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    if symbol.is_empty() {
        return Err(String::from("Symbol cannot be empty"));
    }
    if !symbol.chars().all(|c| c.is_alphanumeric()) {
        return Err(String::from("Symbol contains invalid characters"));
    }
    Ok(())
}

fn validate_quantity(quantity: f64) -> Result<(), String> {
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    if quantity > 1000.0 {
        return Err(String::from("Maximum order size exceeded"));
    }
    Ok(())
}

fn get_market_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTCUSDT" => Ok(42000.0),
        "ETHUSDT" => Ok(2500.0),
        _ => Err(format!("No price data for {}", symbol)),
    }
}

fn execute_trade(
    symbol: &str,
    quantity: f64,
    balance: f64,
) -> Result<TradeResult, String> {
    // The ? operator automatically returns Err if operation fails
    validate_symbol(symbol)?;
    validate_quantity(quantity)?;

    let price = get_market_price(symbol)?;
    let total_cost = price * quantity;

    if total_cost > balance {
        return Err(format!(
            "Insufficient funds: need {:.2}, available {:.2}",
            total_cost, balance
        ));
    }

    let fees = total_cost * 0.001; // 0.1% fee

    Ok(TradeResult {
        order_id: 123456,
        executed_price: price,
        quantity,
        fees,
    })
}

fn main() {
    match execute_trade("BTCUSDT", 0.5, 50000.0) {
        Ok(result) => {
            println!("Trade executed!");
            println!("Order ID: {}", result.order_id);
            println!("Price: ${:.2}", result.executed_price);
            println!("Quantity: {}", result.quantity);
            println!("Fees: ${:.2}", result.fees);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

## Combining Result with Option

```rust
fn find_best_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }
    prices.iter().cloned().reduce(f64::min)
}

fn calculate_profit(
    entry: f64,
    exit: f64,
    quantity: f64,
) -> Result<f64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    Ok((exit - entry) * quantity)
}

fn analyze_trade_opportunity(
    prices: &[f64],
    current_price: f64,
    quantity: f64,
) -> Result<f64, String> {
    // ok_or converts Option to Result
    let best_entry = find_best_price(prices)
        .ok_or_else(|| String::from("No historical prices for analysis"))?;

    // Calculate potential profit
    calculate_profit(best_entry, current_price, quantity)
}

fn main() {
    let historical_prices = vec![41000.0, 41500.0, 40800.0, 41200.0];
    let current = 42000.0;

    match analyze_trade_opportunity(&historical_prices, current, 0.5) {
        Ok(profit) => println!("Potential profit: ${:.2}", profit),
        Err(e) => println!("Analysis error: {}", e),
    }

    // With empty array
    match analyze_trade_opportunity(&[], current, 0.5) {
        Ok(profit) => println!("Profit: ${:.2}", profit),
        Err(e) => println!("Error: {}", e), // "No historical prices..."
    }
}
```

## Practical Example: Trading Validator

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum ValidationError {
    EmptySymbol,
    InvalidSide(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance { required: f64, available: f64 },
    PositionLimitExceeded { current: f64, max: f64 },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptySymbol => write!(f, "Symbol not specified"),
            ValidationError::InvalidSide(s) => write!(f, "Invalid side: {}", s),
            ValidationError::InvalidQuantity(q) => write!(f, "Invalid quantity: {}", q),
            ValidationError::InvalidPrice(p) => write!(f, "Invalid price: {}", p),
            ValidationError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: need {:.2}, have {:.2}", required, available)
            }
            ValidationError::PositionLimitExceeded { current, max } => {
                write!(f, "Position limit exceeded: {} > {}", current, max)
            }
        }
    }
}

fn validate_order(
    order: &Order,
    balance: f64,
    current_position: f64,
    max_position: f64,
) -> Result<(), ValidationError> {
    // Check symbol
    if order.symbol.is_empty() {
        return Err(ValidationError::EmptySymbol);
    }

    // Check side
    if order.side != "BUY" && order.side != "SELL" {
        return Err(ValidationError::InvalidSide(order.side.clone()));
    }

    // Check quantity
    if order.quantity <= 0.0 {
        return Err(ValidationError::InvalidQuantity(order.quantity));
    }

    // Check price
    if order.price <= 0.0 {
        return Err(ValidationError::InvalidPrice(order.price));
    }

    // Check balance (only for buy)
    if order.side == "BUY" {
        let required = order.quantity * order.price;
        if required > balance {
            return Err(ValidationError::InsufficientBalance {
                required,
                available: balance,
            });
        }
    }

    // Check position limit
    let new_position = if order.side == "BUY" {
        current_position + order.quantity
    } else {
        current_position - order.quantity
    };

    if new_position.abs() > max_position {
        return Err(ValidationError::PositionLimitExceeded {
            current: new_position.abs(),
            max: max_position,
        });
    }

    Ok(())
}

fn main() {
    let order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 0.5,
        price: 42000.0,
    };

    let balance = 50000.0;
    let current_position = 0.3;
    let max_position = 1.0;

    match validate_order(&order, balance, current_position, max_position) {
        Ok(()) => {
            println!("Order passed validation");
            println!("Sending to exchange: {:?}", order);
        }
        Err(e) => {
            println!("Validation failed: {}", e);
        }
    }

    // Test with error
    let bad_order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 2.0, // Will exceed position limit
        price: 42000.0,
    };

    match validate_order(&bad_order, balance, current_position, max_position) {
        Ok(()) => println!("Order OK"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Result<T, E>` | Operation returns success `T` or error `E` |
| `Ok(value)` | Successful result |
| `Err(error)` | Error with reason description |
| `?` operator | Automatic error propagation |
| `unwrap()` | Extract value or panic |
| `unwrap_or(default)` | Extract value or return default |
| `map()` | Transform successful value |
| `and_then()` | Chain fallible operations |
| `ok_or()` | Convert Option to Result |

## Practice Exercises

1. **Price Validation**: Write a function `validate_price(price: f64, min: f64, max: f64) -> Result<f64, PriceError>`, where `PriceError` is an enum with variants `Negative`, `BelowMin`, `AboveMax`.

2. **Order Parser**: Create a function `parse_order(input: &str) -> Result<Order, ParseError>` that parses a string like "BUY BTCUSDT 0.5 42000.0" into an Order struct.

3. **Validation Chain**: Implement a function `process_trade(...)` that sequentially checks: balance → limits → market conditions, using the `?` operator.

4. **Error Aggregation**: Write a function that validates multiple fields and collects all errors into a `Vec<ValidationError>`.

## Homework

1. Create an enum `TradingError` with variants for all possible trading system errors (network, validation, execution, etc.) and implement `Display` for it.

2. Write a function `execute_strategy(...)` that:
   - Fetches market data (can fail)
   - Analyzes the signal (may not find a signal)
   - Validates the order (may be invalid)
   - Sends for execution (can be rejected)

   Use `Result` and `?` for elegant handling of all cases.

3. Implement retry logic: a function `execute_with_retry(order, max_attempts) -> Result<TradeResult, TradingError>` that attempts to execute an order multiple times on recoverable errors.

## Navigation

[← Previous day](../074-option-value-may-be-absent/en.md) | [Next day →](../076-question-mark-operator/en.md)
