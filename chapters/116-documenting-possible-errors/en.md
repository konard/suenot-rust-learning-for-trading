# Day 116: Documenting Possible Errors

## Trading Analogy

Imagine you're creating a trading bot and handing it over to another trader. They need to know:
- What errors can the order submission function return?
- Under what conditions might the program crash?
- What data is considered invalid?

It's like an instruction manual for a trading terminal: "If internet connection is lost, the order won't be sent", "If there are insufficient funds, an InsufficientBalance error will be returned".

In Rust, we use special sections in documentation comments (`///`) to describe possible errors: `# Errors`, `# Panics`, and `# Safety`.

## The # Errors Section

Used for functions that return `Result`. Describes under what conditions the function will return `Err`.

```rust
use std::fmt;

/// Errors when sending an order
#[derive(Debug)]
enum OrderError {
    InsufficientBalance,
    InvalidPrice,
    MarketClosed,
    ConnectionFailed,
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::InsufficientBalance => write!(f, "Insufficient funds"),
            OrderError::InvalidPrice => write!(f, "Invalid price"),
            OrderError::MarketClosed => write!(f, "Market is closed"),
            OrderError::ConnectionFailed => write!(f, "Connection failed"),
        }
    }
}

/// Sends a market order to the exchange.
///
/// # Arguments
///
/// * `symbol` - Trading symbol (e.g., "BTCUSDT")
/// * `quantity` - Quantity to buy
/// * `balance` - Available balance
/// * `market_open` - Whether the market is open
///
/// # Returns
///
/// ID of the successfully created order
///
/// # Errors
///
/// Returns an error in the following cases:
///
/// * [`OrderError::InsufficientBalance`] - if `balance` is less than the order value
/// * [`OrderError::InvalidPrice`] - if the current price is zero or negative
/// * [`OrderError::MarketClosed`] - if `market_open` is `false`
/// * [`OrderError::ConnectionFailed`] - if connection to the exchange failed
fn send_market_order(
    symbol: &str,
    quantity: f64,
    balance: f64,
    market_open: bool,
) -> Result<u64, OrderError> {
    if !market_open {
        return Err(OrderError::MarketClosed);
    }

    // Simulating price retrieval
    let current_price = get_current_price(symbol);

    if current_price <= 0.0 {
        return Err(OrderError::InvalidPrice);
    }

    let order_value = current_price * quantity;

    if balance < order_value {
        return Err(OrderError::InsufficientBalance);
    }

    // Simulating successful order
    Ok(12345)
}

fn get_current_price(_symbol: &str) -> f64 {
    42000.0  // Simulation
}

fn main() {
    match send_market_order("BTCUSDT", 0.1, 5000.0, true) {
        Ok(order_id) => println!("Order created: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }
}
```

## The # Panics Section

Describes conditions under which the function will call `panic!`. This is important because a panic terminates the program (or thread).

```rust
/// Calculates profit as a percentage.
///
/// # Arguments
///
/// * `entry_price` - Position entry price
/// * `exit_price` - Position exit price
///
/// # Returns
///
/// Profit or loss percentage
///
/// # Panics
///
/// Panics if `entry_price` is zero, as this would cause
/// division by zero. Always validate entry price before calling.
///
/// # Examples
///
/// ```
/// let profit = calculate_profit_percent(100.0, 110.0);
/// assert_eq!(profit, 10.0);
/// ```
fn calculate_profit_percent(entry_price: f64, exit_price: f64) -> f64 {
    if entry_price == 0.0 {
        panic!("Entry price cannot be zero!");
    }

    ((exit_price - entry_price) / entry_price) * 100.0
}

fn main() {
    println!("Profit: {:.2}%", calculate_profit_percent(42000.0, 44100.0));
}
```

## Prefer Result Over Panic

In trading, reliability is critical. Prefer `Result` over `panic!`:

```rust
use std::fmt;

#[derive(Debug)]
enum CalculationError {
    DivisionByZero,
    NegativePrice,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationError::DivisionByZero => write!(f, "Division by zero"),
            CalculationError::NegativePrice => write!(f, "Negative price"),
        }
    }
}

/// Safely calculates profit as a percentage.
///
/// # Arguments
///
/// * `entry_price` - Position entry price
/// * `exit_price` - Position exit price
///
/// # Returns
///
/// Profit or loss percentage, wrapped in `Result`
///
/// # Errors
///
/// * [`CalculationError::DivisionByZero`] - if `entry_price` is zero
/// * [`CalculationError::NegativePrice`] - if any price is negative
///
/// # Examples
///
/// ```
/// let profit = calculate_profit_percent_safe(100.0, 110.0);
/// assert_eq!(profit.unwrap(), 10.0);
/// ```
fn calculate_profit_percent_safe(
    entry_price: f64,
    exit_price: f64,
) -> Result<f64, CalculationError> {
    if entry_price < 0.0 || exit_price < 0.0 {
        return Err(CalculationError::NegativePrice);
    }

    if entry_price == 0.0 {
        return Err(CalculationError::DivisionByZero);
    }

    Ok(((exit_price - entry_price) / entry_price) * 100.0)
}

fn main() {
    match calculate_profit_percent_safe(42000.0, 44100.0) {
        Ok(profit) => println!("Profit: {:.2}%", profit),
        Err(e) => println!("Calculation error: {}", e),
    }
}
```

## Documentation for Option

For functions returning `Option`, use `# Returns` with an explanation of when `None` is returned:

```rust
/// Finds an order by ID in the list.
///
/// # Arguments
///
/// * `orders` - List of tuples (order_id, price, quantity)
/// * `order_id` - ID of the order to find
///
/// # Returns
///
/// Returns `Some((price, quantity))` if the order is found,
/// or `None` if no order with the given ID exists in the list.
fn find_order(orders: &[(u64, f64, f64)], order_id: u64) -> Option<(f64, f64)> {
    orders
        .iter()
        .find(|(id, _, _)| *id == order_id)
        .map(|(_, price, quantity)| (*price, *quantity))
}

fn main() {
    let orders = vec![
        (1, 42000.0, 0.5),
        (2, 42100.0, 0.3),
        (3, 41900.0, 0.7),
    ];

    match find_order(&orders, 2) {
        Some((price, qty)) => println!("Order found: {} @ {}", qty, price),
        None => println!("Order not found"),
    }
}
```

## Comprehensive Example: Exchange API Client

```rust
use std::fmt;

/// Order types
#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
}

/// Trading API errors
#[derive(Debug)]
enum TradingApiError {
    /// Insufficient balance to complete the trade
    InsufficientBalance { required: f64, available: f64 },
    /// Market is currently closed
    MarketClosed,
    /// Invalid order parameters
    InvalidOrder(String),
    /// Network error
    NetworkError(String),
    /// Order rejected by the exchange
    OrderRejected { reason: String },
}

impl fmt::Display for TradingApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingApiError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: need {}, have {}", required, available)
            }
            TradingApiError::MarketClosed => write!(f, "Market is closed"),
            TradingApiError::InvalidOrder(msg) => write!(f, "Invalid order: {}", msg),
            TradingApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TradingApiError::OrderRejected { reason } => {
                write!(f, "Order rejected: {}", reason)
            }
        }
    }
}

struct TradingClient {
    balance: f64,
    connected: bool,
    market_open: bool,
}

impl TradingClient {
    /// Creates a new trading API client.
    ///
    /// # Arguments
    ///
    /// * `initial_balance` - Initial account balance
    ///
    /// # Panics
    ///
    /// Panics if `initial_balance` is negative.
    /// Use [`TradingClient::try_new`] for safe creation.
    fn new(initial_balance: f64) -> Self {
        if initial_balance < 0.0 {
            panic!("Initial balance cannot be negative");
        }

        TradingClient {
            balance: initial_balance,
            connected: true,
            market_open: true,
        }
    }

    /// Safely creates a new trading API client.
    ///
    /// # Arguments
    ///
    /// * `initial_balance` - Initial account balance
    ///
    /// # Returns
    ///
    /// `Some(TradingClient)` if balance is valid, `None` if negative.
    fn try_new(initial_balance: f64) -> Option<Self> {
        if initial_balance < 0.0 {
            return None;
        }

        Some(TradingClient {
            balance: initial_balance,
            connected: true,
            market_open: true,
        })
    }

    /// Places a new order on the exchange.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading symbol (e.g., "BTCUSDT", "ETHUSDT")
    /// * `quantity` - Amount of asset to buy/sell
    /// * `order_type` - Order type: Market, Limit, or StopLoss
    ///
    /// # Returns
    ///
    /// ID of the successfully created order
    ///
    /// # Errors
    ///
    /// Returns an error in the following cases:
    ///
    /// * [`TradingApiError::NetworkError`] - if the client is not connected
    ///   to the exchange. Call `reconnect()` before retrying.
    ///
    /// * [`TradingApiError::MarketClosed`] - if the market is closed.
    ///   Market orders cannot be placed during off-hours.
    ///
    /// * [`TradingApiError::InvalidOrder`] - if:
    ///   - `symbol` is empty or contains invalid characters
    ///   - `quantity` is less than or equal to zero
    ///   - For Limit orders: price is less than or equal to zero
    ///   - For StopLoss: trigger_price is less than or equal to zero
    ///
    /// * [`TradingApiError::InsufficientBalance`] - if balance is less than
    ///   the order value. Contains required and available amounts.
    ///
    /// * [`TradingApiError::OrderRejected`] - if the exchange rejected the
    ///   order for internal reasons (e.g., position limit exceeded).
    ///
    /// # Examples
    ///
    /// ```
    /// let client = TradingClient::new(10000.0);
    ///
    /// // Market order
    /// match client.place_order("BTCUSDT", 0.1, OrderType::Market) {
    ///     Ok(id) => println!("Order created: {}", id),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    ///
    /// // Limit order
    /// let limit = OrderType::Limit { price: 42000.0 };
    /// client.place_order("BTCUSDT", 0.5, limit)?;
    /// ```
    fn place_order(
        &self,
        symbol: &str,
        quantity: f64,
        order_type: OrderType,
    ) -> Result<u64, TradingApiError> {
        // Check connection
        if !self.connected {
            return Err(TradingApiError::NetworkError(
                "Not connected to exchange".to_string()
            ));
        }

        // Check market status
        if !self.market_open {
            return Err(TradingApiError::MarketClosed);
        }

        // Validate symbol
        if symbol.is_empty() {
            return Err(TradingApiError::InvalidOrder(
                "Symbol cannot be empty".to_string()
            ));
        }

        // Validate quantity
        if quantity <= 0.0 {
            return Err(TradingApiError::InvalidOrder(
                format!("Quantity must be positive, got: {}", quantity)
            ));
        }

        // Validate order type
        match &order_type {
            OrderType::Limit { price } if *price <= 0.0 => {
                return Err(TradingApiError::InvalidOrder(
                    format!("Limit order price must be positive: {}", price)
                ));
            }
            OrderType::StopLoss { trigger_price } if *trigger_price <= 0.0 => {
                return Err(TradingApiError::InvalidOrder(
                    format!("Trigger price must be positive: {}", trigger_price)
                ));
            }
            _ => {}
        }

        // Calculate order value (simulation)
        let price = match &order_type {
            OrderType::Market => 42000.0,  // Current market price
            OrderType::Limit { price } => *price,
            OrderType::StopLoss { trigger_price } => *trigger_price,
        };

        let order_value = price * quantity;

        // Check balance
        if self.balance < order_value {
            return Err(TradingApiError::InsufficientBalance {
                required: order_value,
                available: self.balance,
            });
        }

        // Successful order creation
        Ok(12345)
    }

    /// Gets the current price of an asset.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading symbol
    ///
    /// # Returns
    ///
    /// Current asset price, or `None` if:
    /// - Symbol is not found on the exchange
    /// - No quotes are available
    /// - Client is not connected to the exchange
    fn get_price(&self, symbol: &str) -> Option<f64> {
        if !self.connected || symbol.is_empty() {
            return None;
        }

        // Simulating price retrieval
        match symbol {
            "BTCUSDT" => Some(42000.0),
            "ETHUSDT" => Some(2500.0),
            _ => None,
        }
    }
}

fn main() {
    let client = TradingClient::new(10000.0);

    // Example of successful order
    match client.place_order("BTCUSDT", 0.1, OrderType::Market) {
        Ok(id) => println!("Order created: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    // Example of error - insufficient funds
    match client.place_order("BTCUSDT", 1.0, OrderType::Market) {
        Ok(id) => println!("Order created: {}", id),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Generating Documentation

To see your documentation in a nice format:

```bash
cargo doc --open
```

This will create HTML documentation with all # Errors, # Panics sections and examples.

## What We Learned

| Section | Purpose | When to Use |
|---------|---------|-------------|
| `# Errors` | Describe Result errors | Functions returning Result |
| `# Panics` | Panic conditions | Functions with assert!/panic! |
| `# Returns` + None | When Option = None | Functions returning Option |
| `# Safety` | Unsafe invariants | unsafe functions |

## Practical Exercises

1. **Documenting a Trading Validator**

   Write a function `validate_order(price, quantity, side)` and document all possible validation errors.

2. **Risk Calculator Documentation**

   Create a function `calculate_position_size(balance, risk_percent, entry, stop_loss)` with complete error documentation.

3. **API Methods with Documentation**

   Implement methods `get_balance()`, `cancel_order(id)`, `get_open_orders()` for `TradingClient` with documentation of all possible errors.

## Homework

1. Create a `Portfolio` struct with methods:
   - `add_position(symbol, quantity, price)` - with error documentation
   - `remove_position(symbol)` - with documentation for when `None` is returned
   - `total_value()` - with documentation of possible panics

2. Write a function `parse_trade_signal(signal_str)` that parses trading signals like "BUY:BTCUSDT:0.5" and document all parsing errors.

3. Implement an `OrderBook` with methods `get_best_bid()` and `get_best_ask()`, documenting when `None` is returned.

4. Use `cargo doc --open` to view your documentation and ensure it's understandable to other developers.

## Navigation

[← Previous day](../115-mocking-errors-in-tests/en.md) | [Next day →](../117-async-errors-preview/en.md)
