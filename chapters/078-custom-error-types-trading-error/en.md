# Day 78: Custom Error Types: TradingError

## Trading Analogy

Imagine you're working with different exchanges. Each exchange reports errors differently: one says "Insufficient funds", another "Balance too low", a third returns error code 403. How can a trader make sense of all this chaos?

The solution is to create a **unified error standard** for your trading system. Instead of handling dozens of different messages, you define your own `TradingError` type that unifies all possible problems.

## Why Custom Error Types?

```rust
// Bad: returning String — unclear what errors are possible
fn execute_order(order: &Order) -> Result<Trade, String> {
    Err(String::from("Something went wrong"))
}

// Good: clear enumeration of all possible errors
fn execute_order(order: &Order) -> Result<Trade, TradingError> {
    Err(TradingError::InsufficientBalance {
        required: 1000.0,
        available: 500.0
    })
}
```

Custom error types provide:
- **Complete list** of possible errors (visible in code)
- **Typed information** about the error (not just text)
- **Ability to match** — different reactions to different errors
- **Compatibility with `?`** — easy error propagation

## Creating TradingError

```rust
#[derive(Debug)]
enum TradingError {
    // Balance errors
    InsufficientBalance { required: f64, available: f64 },

    // Order errors
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    InvalidPrice { price: f64, reason: String },
    OrderNotFound { order_id: u64 },

    // Market errors
    MarketClosed,
    SymbolNotFound { symbol: String },

    // Connection errors
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },

    // Wrapper for external errors
    ApiError { code: i32, message: String },
}
```

## Implementing Display for Pretty Output

```rust
use std::fmt;

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: need ${:.2}, available ${:.2}",
                       required, available)
            }
            TradingError::InvalidOrderSize { size, min, max } => {
                write!(f, "Invalid order size {}: allowed range {} to {}",
                       size, min, max)
            }
            TradingError::InvalidPrice { price, reason } => {
                write!(f, "Invalid price {}: {}", price, reason)
            }
            TradingError::OrderNotFound { order_id } => {
                write!(f, "Order #{} not found", order_id)
            }
            TradingError::MarketClosed => {
                write!(f, "Market is closed")
            }
            TradingError::SymbolNotFound { symbol } => {
                write!(f, "Symbol '{}' not found", symbol)
            }
            TradingError::ConnectionLost => {
                write!(f, "Connection lost")
            }
            TradingError::Timeout { operation, seconds } => {
                write!(f, "Operation '{}' timed out after {} sec", operation, seconds)
            }
            TradingError::ApiError { code, message } => {
                write!(f, "API error [{}]: {}", code, message)
            }
        }
    }
}
```

## Implementing std::error::Error

```rust
use std::error::Error;

impl Error for TradingError {}

fn main() {
    let err = TradingError::InsufficientBalance {
        required: 10000.0,
        available: 5000.0
    };

    // Display — for the user
    println!("Error: {}", err);

    // Debug — for the developer
    println!("Debug: {:?}", err);
}
```

## Practical Example: Trading System

```rust
#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Trade {
    order_id: u64,
    executed_price: f64,
    executed_quantity: f64,
}

#[derive(Debug)]
struct TradingAccount {
    balance: f64,
    positions: Vec<(String, f64)>, // (symbol, quantity)
}

impl TradingAccount {
    fn validate_order(&self, order: &Order) -> Result<(), TradingError> {
        // Check minimum order size
        let min_size = 0.001;
        let max_size = 1000.0;

        if order.quantity < min_size || order.quantity > max_size {
            return Err(TradingError::InvalidOrderSize {
                size: order.quantity,
                min: min_size,
                max: max_size,
            });
        }

        // Check price
        if order.price <= 0.0 {
            return Err(TradingError::InvalidPrice {
                price: order.price,
                reason: String::from("Price must be positive"),
            });
        }

        // Check balance for buy orders
        if matches!(order.side, OrderSide::Buy) {
            let required = order.price * order.quantity;
            if required > self.balance {
                return Err(TradingError::InsufficientBalance {
                    required,
                    available: self.balance,
                });
            }
        }

        Ok(())
    }

    fn execute_order(&mut self, order: &Order) -> Result<Trade, TradingError> {
        // First validate
        self.validate_order(order)?;

        // Simulate execution
        match order.side {
            OrderSide::Buy => {
                let cost = order.price * order.quantity;
                self.balance -= cost;
            }
            OrderSide::Sell => {
                // Check if we have position to sell
                let position = self.positions.iter_mut()
                    .find(|(s, _)| s == &order.symbol);

                match position {
                    Some((_, qty)) if *qty >= order.quantity => {
                        *qty -= order.quantity;
                        self.balance += order.price * order.quantity;
                    }
                    _ => {
                        return Err(TradingError::InsufficientBalance {
                            required: order.quantity,
                            available: 0.0,
                        });
                    }
                }
            }
        }

        Ok(Trade {
            order_id: order.id,
            executed_price: order.price,
            executed_quantity: order.quantity,
        })
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 1000.0,
        positions: vec![("BTC".to_string(), 0.5)],
    };

    // Attempt to buy too much
    let big_order = Order {
        id: 1,
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        price: 50000.0,
        quantity: 1.0,
    };

    match account.execute_order(&big_order) {
        Ok(trade) => println!("Trade executed: {:?}", trade),
        Err(e) => println!("Error: {}", e),
    }

    // Successful purchase
    let small_order = Order {
        id: 2,
        symbol: "ETH".to_string(),
        side: OrderSide::Buy,
        price: 100.0,
        quantity: 5.0,
    };

    match account.execute_order(&small_order) {
        Ok(trade) => println!("Trade executed: {:?}", trade),
        Err(e) => println!("Error: {}", e),
    }

    println!("Balance after trades: ${:.2}", account.balance);
}
```

## Error Conversion: From Trait

Often you need to convert errors from external libraries to your own type:

```rust
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidPrice { price: f64, reason: String },
    IoError(io::Error),
    ParseError(String),
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: need {}, available {}", required, available)
            }
            TradingError::InvalidPrice { price, reason } => {
                write!(f, "Invalid price {}: {}", price, reason)
            }
            TradingError::IoError(e) => write!(f, "I/O error: {}", e),
            TradingError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for TradingError {}

// Automatic conversion from io::Error to TradingError
impl From<io::Error> for TradingError {
    fn from(err: io::Error) -> Self {
        TradingError::IoError(err)
    }
}

// Automatic conversion from ParseFloatError to TradingError
impl From<ParseFloatError> for TradingError {
    fn from(err: ParseFloatError) -> Self {
        TradingError::ParseError(err.to_string())
    }
}

// Now we can use ? with different error types
fn load_price_from_file(path: &str) -> Result<f64, TradingError> {
    let content = std::fs::read_to_string(path)?; // io::Error -> TradingError
    let price: f64 = content.trim().parse()?;     // ParseFloatError -> TradingError

    if price <= 0.0 {
        return Err(TradingError::InvalidPrice {
            price,
            reason: String::from("Price from file must be positive"),
        });
    }

    Ok(price)
}

fn main() {
    match load_price_from_file("price.txt") {
        Ok(price) => println!("Price: ${:.2}", price),
        Err(e) => println!("Load error: {}", e),
    }
}
```

## Handling Different Errors Differently

```rust
fn handle_trading_error(error: &TradingError) {
    match error {
        TradingError::InsufficientBalance { required, available } => {
            let deficit = required - available;
            println!("Please deposit at least ${:.2}", deficit);
        }
        TradingError::InvalidOrderSize { size, min, max } => {
            if *size < *min {
                println!("Increase order size to at least {}", min);
            } else {
                println!("Decrease order size to at most {}", max);
            }
        }
        TradingError::MarketClosed => {
            println!("Wait for market to open");
        }
        TradingError::ConnectionLost => {
            println!("Check your internet connection");
        }
        TradingError::Timeout { operation, .. } => {
            println!("Try repeating operation '{}'", operation);
        }
        _ => {
            println!("Contact support: {}", error);
        }
    }
}

#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    InvalidPrice { price: f64, reason: String },
    OrderNotFound { order_id: u64 },
    MarketClosed,
    SymbolNotFound { symbol: String },
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },
    ApiError { code: i32, message: String },
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let errors = vec![
        TradingError::InsufficientBalance { required: 5000.0, available: 1000.0 },
        TradingError::InvalidOrderSize { size: 0.0001, min: 0.001, max: 1000.0 },
        TradingError::MarketClosed,
        TradingError::ConnectionLost,
    ];

    for err in &errors {
        println!("\n--- Error: {} ---", err);
        handle_trading_error(err);
    }
}
```

## Methods for TradingError

```rust
#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },
    ApiError { code: i32, message: String },
}

impl TradingError {
    /// Can the operation be retried?
    fn is_retryable(&self) -> bool {
        matches!(self,
            TradingError::ConnectionLost |
            TradingError::Timeout { .. } |
            TradingError::ApiError { code, .. } if *code >= 500
        )
    }

    /// Is this a critical error?
    fn is_critical(&self) -> bool {
        matches!(self, TradingError::InsufficientBalance { .. })
    }

    /// Error code for logging
    fn error_code(&self) -> &'static str {
        match self {
            TradingError::InsufficientBalance { .. } => "E001",
            TradingError::InvalidOrderSize { .. } => "E002",
            TradingError::ConnectionLost => "E003",
            TradingError::Timeout { .. } => "E004",
            TradingError::ApiError { .. } => "E005",
        }
    }
}

fn process_with_retry(operation: impl Fn() -> Result<(), TradingError>) {
    let max_retries = 3;
    let mut attempts = 0;

    loop {
        match operation() {
            Ok(()) => {
                println!("Operation successful!");
                break;
            }
            Err(e) if e.is_retryable() && attempts < max_retries => {
                attempts += 1;
                println!("[{}] Retry {}/{}: {}", e.error_code(), attempts, max_retries, e);
            }
            Err(e) => {
                println!("[{}] Fatal error: {}", e.error_code(), e);
                if e.is_critical() {
                    println!("Immediate intervention required!");
                }
                break;
            }
        }
    }
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let mut counter = 0;

    process_with_retry(|| {
        counter += 1;
        if counter < 3 {
            Err(TradingError::ConnectionLost)
        } else {
            Ok(())
        }
    });
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `enum` for errors | Enumeration of all possible errors |
| `Display` trait | Pretty output for users |
| `Error` trait | Compatibility with standard library |
| `From` trait | Automatic error conversion |
| `match` on errors | Different logic for different errors |
| Methods on enum | Useful functions like `is_retryable()` |

## Exercises

1. **Extend TradingError**: add variants `RateLimitExceeded { retry_after: u64 }` and `AuthenticationFailed { reason: String }`

2. **Error hierarchy**: create separate enums `OrderError`, `ConnectionError`, `AccountError` and combine them into `TradingError`

3. **Logging**: add a method `log_level(&self) -> &str` that returns "ERROR", "WARN", or "INFO" depending on error type

4. **Context**: implement a method `with_context(self, ctx: &str) -> Self` that adds context to the error

## Homework

Create a complete error system for a trading bot:

1. Define `TradingError` with at least 10 error variants
2. Implement `Display`, `Debug`, `Error`
3. Add `From` for `std::io::Error` and `serde_json::Error`
4. Implement methods `is_retryable()`, `is_critical()`, `suggested_action() -> String`
5. Write a function `execute_with_error_handling()` that handles all error types differently

## Navigation

[← Day 77: The ? Operator](../077-question-mark-operator/en.md) | [Day 79: Vec — Dynamic List of Trades →](../079-vec-dynamic-trades/en.md)
