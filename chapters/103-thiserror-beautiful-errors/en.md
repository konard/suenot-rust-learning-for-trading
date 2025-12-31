# Day 103: thiserror — Creating Beautiful Errors

## Trading Analogy

Imagine you work at a major brokerage firm. When something goes wrong, it's important not just to say "Error!" but to provide a clear, understandable message:

❌ **Bad:** "Error 500"
✅ **Good:** "Insufficient funds to buy 100 AAPL shares at $150.00. Available: $10,000. Required: $15,000"

In trading, clear error messages can save millions. The `thiserror` library helps create exactly that — informative and structured errors.

## What is thiserror?

`thiserror` is a library for conveniently creating custom error types using derive macros. It automatically implements the `std::error::Error` trait and message formatting.

### Advantages of thiserror

| Without thiserror | With thiserror |
|-------------------|----------------|
| Lots of boilerplate code | Minimal code |
| Manual Display implementation | Automatic Display |
| Complex source handling | Simple #[source] attribute |
| Hard to maintain | Easy to read and modify |

## Adding the Library

Add to your `Cargo.toml`:

```toml
[dependencies]
thiserror = "1.0"
```

## Basic Syntax

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds {
        required: f64,
        available: f64,
    },

    #[error("Invalid ticker: {0}")]
    InvalidTicker(String),

    #[error("Market closed")]
    MarketClosed,
}
```

## Message Formatting

`thiserror` supports several formatting methods:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderError {
    // Positional arguments
    #[error("Order {0} not found")]
    NotFound(u64),

    // Named fields
    #[error("Limit exceeded: {current}/{max} orders")]
    LimitExceeded { current: usize, max: usize },

    // Method calls
    #[error("Invalid price: {price:.2} (min: {min:.2})")]
    InvalidPrice { price: f64, min: f64 },

    // Using Display of inner type
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
```

## Practical Example: Trading System

```rust
use thiserror::Error;
use std::collections::HashMap;

// Define all possible trading system errors
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Insufficient funds to buy {quantity} {ticker}: required ${required:.2}, available ${available:.2}")]
    InsufficientFunds {
        ticker: String,
        quantity: u32,
        required: f64,
        available: f64,
    },

    #[error("Asset {0} not found on exchange")]
    AssetNotFound(String),

    #[error("Cannot sell {quantity} {ticker}: only {available} in portfolio")]
    InsufficientShares {
        ticker: String,
        quantity: u32,
        available: u32,
    },

    #[error("Market {0} is currently closed")]
    MarketClosed(String),

    #[error("Order rejected: {reason}")]
    OrderRejected { reason: String },

    #[error("Daily trading limit exceeded: ${current:.2} / ${limit:.2}")]
    DailyLimitExceeded { current: f64, limit: f64 },
}

// Portfolio structure
struct Portfolio {
    balance: f64,
    holdings: HashMap<String, u32>,
    daily_traded: f64,
    daily_limit: f64,
}

impl Portfolio {
    fn new(balance: f64, daily_limit: f64) -> Self {
        Portfolio {
            balance,
            holdings: HashMap::new(),
            daily_traded: 0.0,
            daily_limit,
        }
    }

    fn buy(&mut self, ticker: &str, quantity: u32, price: f64) -> Result<(), TradingError> {
        let total_cost = price * quantity as f64;

        // Check daily limit
        if self.daily_traded + total_cost > self.daily_limit {
            return Err(TradingError::DailyLimitExceeded {
                current: self.daily_traded + total_cost,
                limit: self.daily_limit,
            });
        }

        // Check balance
        if total_cost > self.balance {
            return Err(TradingError::InsufficientFunds {
                ticker: ticker.to_string(),
                quantity,
                required: total_cost,
                available: self.balance,
            });
        }

        // Execute purchase
        self.balance -= total_cost;
        self.daily_traded += total_cost;
        *self.holdings.entry(ticker.to_string()).or_insert(0) += quantity;

        println!("✅ Bought {} {} at ${:.2}", quantity, ticker, price);
        Ok(())
    }

    fn sell(&mut self, ticker: &str, quantity: u32, price: f64) -> Result<(), TradingError> {
        let available = *self.holdings.get(ticker).unwrap_or(&0);

        if available < quantity {
            return Err(TradingError::InsufficientShares {
                ticker: ticker.to_string(),
                quantity,
                available,
            });
        }

        // Execute sale
        let total = price * quantity as f64;
        self.balance += total;
        self.daily_traded += total;
        *self.holdings.get_mut(ticker).unwrap() -= quantity;

        println!("✅ Sold {} {} at ${:.2}", quantity, ticker, price);
        Ok(())
    }
}

fn main() {
    let mut portfolio = Portfolio::new(10_000.0, 50_000.0);

    // Successful purchase
    match portfolio.buy("AAPL", 10, 150.0) {
        Ok(()) => println!("Balance: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Attempt to buy too much
    match portfolio.buy("TSLA", 100, 200.0) {
        Ok(()) => println!("Balance: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Attempt to sell what we don't have
    match portfolio.sell("GOOGL", 5, 140.0) {
        Ok(()) => println!("Balance: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Error: {}", e),
    }
}
```

Program output:
```
✅ Bought 10 AAPL at $150.00
Balance: $8500.00
❌ Error: Insufficient funds to buy 100 TSLA: required $20000.00, available $8500.00
❌ Error: Cannot sell 5 GOOGL: only 0 in portfolio
```

## Working with Nested Errors

The `#[source]` attribute allows preserving error chains:

```rust
use thiserror::Error;
use std::num::ParseFloatError;

#[derive(Error, Debug)]
pub enum PriceParseError {
    #[error("Invalid price format '{input}'")]
    InvalidFormat {
        input: String,
        #[source]
        source: ParseFloatError,
    },

    #[error("Price cannot be negative: {0}")]
    NegativePrice(f64),

    #[error("Price too high: {price} > {max}")]
    PriceTooHigh { price: f64, max: f64 },
}

fn parse_price(input: &str, max_price: f64) -> Result<f64, PriceParseError> {
    let price: f64 = input.trim().parse().map_err(|e| {
        PriceParseError::InvalidFormat {
            input: input.to_string(),
            source: e,
        }
    })?;

    if price < 0.0 {
        return Err(PriceParseError::NegativePrice(price));
    }

    if price > max_price {
        return Err(PriceParseError::PriceTooHigh {
            price,
            max: max_price,
        });
    }

    Ok(price)
}

fn main() {
    // Valid price
    println!("Price: {:?}", parse_price("150.50", 1000.0));

    // Invalid format
    println!("Error: {:?}", parse_price("abc", 1000.0));

    // Negative price
    println!("Error: {:?}", parse_price("-50.0", 1000.0));

    // Price too high
    println!("Error: {:?}", parse_price("5000.0", 1000.0));
}
```

## Automatic Conversion with #[from]

```rust
use thiserror::Error;
use std::io;
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum DataLoadError {
    #[error("File read error")]
    Io(#[from] io::Error),

    #[error("Number parsing error")]
    Parse(#[from] ParseIntError),

    #[error("File is empty")]
    EmptyFile,
}

fn load_prices(filename: &str) -> Result<Vec<i32>, DataLoadError> {
    let content = std::fs::read_to_string(filename)?; // Auto io::Error -> DataLoadError

    if content.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    let prices: Result<Vec<i32>, _> = content
        .lines()
        .map(|line| line.parse())
        .collect();

    Ok(prices?) // Auto ParseIntError -> DataLoadError
}
```

## Example: Risk Management System

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Position {ticker} exceeds limit: {current:.1}% > {max:.1}%")]
    PositionLimitExceeded {
        ticker: String,
        current: f64,
        max: f64,
    },

    #[error("Portfolio risk too high: VaR = ${var:.2} (limit: ${limit:.2})")]
    VaRExceeded { var: f64, limit: f64 },

    #[error("Correlation too high between {asset1} and {asset2}: {correlation:.2}")]
    HighCorrelation {
        asset1: String,
        asset2: String,
        correlation: f64,
    },

    #[error("Excessive leverage: {leverage}x (max: {max_leverage}x)")]
    ExcessiveLeverage { leverage: f64, max_leverage: f64 },
}

struct RiskManager {
    max_position_pct: f64,
    max_var: f64,
    max_leverage: f64,
}

impl RiskManager {
    fn check_position(&self, ticker: &str, position_pct: f64) -> Result<(), RiskError> {
        if position_pct > self.max_position_pct {
            return Err(RiskError::PositionLimitExceeded {
                ticker: ticker.to_string(),
                current: position_pct,
                max: self.max_position_pct,
            });
        }
        Ok(())
    }

    fn check_var(&self, var: f64) -> Result<(), RiskError> {
        if var > self.max_var {
            return Err(RiskError::VaRExceeded {
                var,
                limit: self.max_var,
            });
        }
        Ok(())
    }

    fn check_leverage(&self, leverage: f64) -> Result<(), RiskError> {
        if leverage > self.max_leverage {
            return Err(RiskError::ExcessiveLeverage {
                leverage,
                max_leverage: self.max_leverage,
            });
        }
        Ok(())
    }
}

fn main() {
    let risk_manager = RiskManager {
        max_position_pct: 10.0,
        max_var: 50_000.0,
        max_leverage: 3.0,
    };

    // Check position
    if let Err(e) = risk_manager.check_position("BTC", 25.0) {
        println!("⚠️ Risk: {}", e);
    }

    // Check VaR
    if let Err(e) = risk_manager.check_var(75_000.0) {
        println!("⚠️ Risk: {}", e);
    }

    // Check leverage
    if let Err(e) = risk_manager.check_leverage(5.0) {
        println!("⚠️ Risk: {}", e);
    }
}
```

## Exercises

### Exercise 1: Order Validation Errors
Create an `OrderValidationError` enum with variants:
- `InvalidQuantity` — quantity must be > 0
- `InvalidPrice` — price must be > 0
- `InvalidSide` — side must be "buy" or "sell"

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderValidationError {
    // Your code here
}

fn validate_order(quantity: i32, price: f64, side: &str) -> Result<(), OrderValidationError> {
    // Your code here
    Ok(())
}
```

### Exercise 2: Exchange API Errors
Create an enum for exchange API errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeApiError {
    // Variants:
    // - RateLimited { retry_after: u64 } — rate limit exceeded
    // - AuthenticationFailed — invalid API key
    // - NetworkError with nested std::io::Error
    // - InvalidResponse { status: u16, body: String }
}
```

### Exercise 3: Combining Errors
Create a function that reads prices from a file and validates them:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceDataError {
    // Combine file reading and parsing errors
}

fn load_and_validate_prices(filename: &str) -> Result<Vec<f64>, PriceDataError> {
    // Read file, parse lines to f64, check all prices > 0
    todo!()
}
```

### Exercise 4: Error Hierarchy
Create an error hierarchy for a trading bot:

```rust
#[derive(Error, Debug)]
pub enum StrategyError { /* ... */ }

#[derive(Error, Debug)]
pub enum ExecutionError { /* ... */ }

#[derive(Error, Debug)]
pub enum TradingBotError {
    #[error("Strategy error")]
    Strategy(#[from] StrategyError),

    #[error("Execution error")]
    Execution(#[from] ExecutionError),

    // Add more variants
}
```

## Homework

1. **Create a complete error system** for a trading application, including:
   - Exchange connection errors
   - Order validation errors
   - Risk management errors
   - Portfolio management errors

2. **Implement the From trait** for converting between error levels

3. **Add context** to errors using `.context()` methods (will require `anyhow` from the next chapter)

4. **Write tests** that verify error messages:
   ```rust
   #[test]
   fn test_error_messages() {
       let err = TradingError::InsufficientFunds {
           ticker: "AAPL".to_string(),
           quantity: 10,
           required: 1500.0,
           available: 1000.0,
       };
       assert!(err.to_string().contains("AAPL"));
       assert!(err.to_string().contains("1500"));
   }
   ```

## Key Takeaways

| Concept | Description |
|---------|-------------|
| `#[derive(Error)]` | Automatically implements the Error trait |
| `#[error("...")]` | Sets the message for Display |
| `#[from]` | Automatic conversion from another error |
| `#[source]` | Points to the error cause |
| `#[error(transparent)]` | Forwards Display of nested error |

## Navigation

[← Day 102: Box<dyn Error>](../102-box-dyn-error/en.md) | [Day 104: anyhow →](../104-anyhow-simple-errors/en.md)
