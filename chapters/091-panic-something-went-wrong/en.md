# Day 91: panic! — Something Went Very Wrong

## Trading Analogy

Imagine this situation: your trading system discovers that the account balance has become **negative** when operating without margin. This is impossible under normal conditions — meaning a critical error has occurred. You cannot continue trading — you must **immediately stop everything** and investigate.

The `panic!` macro in Rust does exactly this: it immediately stops the program when detecting a situation that should never have happened.

## Basic Usage of panic!

```rust
fn main() {
    println!("Starting trading system...");

    panic!("Critical error: lost connection to exchange!");

    println!("This code will never execute");
}
```

Output:
```
Starting trading system...
thread 'main' panicked at 'Critical error: lost connection to exchange!'
```

## Panic with Formatting

```rust
fn main() {
    let account_balance = -1500.0;
    let symbol = "BTC/USDT";

    if account_balance < 0.0 {
        panic!(
            "CRITICAL ERROR: Negative balance {} for {}!",
            account_balance,
            symbol
        );
    }
}
```

## When to Use panic!

### 1. Violation of System Invariants

```rust
fn main() {
    let mut portfolio = Portfolio::new(10000.0);

    // Simulating a critical error
    portfolio.balance = -500.0;

    portfolio.validate();
}

struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn validate(&self) {
        // Balance should never be negative without margin
        if self.balance < 0.0 {
            panic!(
                "INVARIANT VIOLATED: Portfolio balance is negative: {}",
                self.balance
            );
        }

        // Check each position
        for position in &self.positions {
            if position.quantity <= 0.0 {
                panic!(
                    "INVARIANT VIOLATED: Zero or negative quantity for {}",
                    position.symbol
                );
            }
            if position.entry_price <= 0.0 {
                panic!(
                    "INVARIANT VIOLATED: Invalid entry price for {}",
                    position.symbol
                );
            }
        }
    }
}
```

### 2. Impossible States in Trading Logic

```rust
fn main() {
    let order_type = "INVALID";
    process_order(order_type, 100.0, 42000.0);
}

fn process_order(order_type: &str, quantity: f64, price: f64) {
    match order_type {
        "BUY" => {
            println!("Buying {} at price {}", quantity, price);
        }
        "SELL" => {
            println!("Selling {} at price {}", quantity, price);
        }
        "LIMIT_BUY" | "LIMIT_SELL" => {
            println!("Limit order: {} at {}", quantity, price);
        }
        _ => {
            // Unknown order type — this is a bug in the code
            panic!("Unknown order type: '{}'. This is a bug!", order_type);
        }
    }
}
```

### 3. Critical Errors During Initialization

```rust
fn main() {
    let config = TradingConfig::load();
    println!("Configuration loaded: {:?}", config);
}

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    api_secret: String,
    max_position_size: f64,
    risk_per_trade: f64,
}

impl TradingConfig {
    fn load() -> Self {
        // Simulating config loading
        let api_key = ""; // Empty key
        let api_secret = "secret123";
        let max_position_size = 10000.0;
        let risk_per_trade = 2.0;

        // Critical parameter checks
        if api_key.is_empty() {
            panic!("CRITICAL ERROR: API key not set! Trading impossible.");
        }

        if api_secret.is_empty() {
            panic!("CRITICAL ERROR: API secret not set!");
        }

        if max_position_size <= 0.0 {
            panic!(
                "CRITICAL ERROR: Invalid max_position_size: {}",
                max_position_size
            );
        }

        if risk_per_trade <= 0.0 || risk_per_trade > 100.0 {
            panic!(
                "CRITICAL ERROR: Invalid risk_per_trade: {}%",
                risk_per_trade
            );
        }

        TradingConfig {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            max_position_size,
            risk_per_trade,
        }
    }
}
```

## Panic in Defensive Programming

### Checking Preconditions

```rust
fn main() {
    // This will cause a panic
    let size = calculate_position_size(10000.0, 0.0, 42000.0, 41000.0);
    println!("Position size: {}", size);
}

fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
) -> f64 {
    // Preconditions — if violated, it's a bug in the calling code
    assert!(balance > 0.0, "Balance must be positive");
    assert!(
        risk_percent > 0.0 && risk_percent <= 100.0,
        "Risk must be between 0 and 100%"
    );
    assert!(entry_price > 0.0, "Entry price must be positive");
    assert!(stop_loss > 0.0, "Stop loss must be positive");
    assert!(
        entry_price != stop_loss,
        "Entry price and stop loss cannot be equal"
    );

    let risk_amount = balance * (risk_percent / 100.0);
    let risk_per_unit = (entry_price - stop_loss).abs();

    risk_amount / risk_per_unit
}
```

### assert! vs panic!

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42050.0];

    // assert! — for condition checks (disabled in release with debug_assertions)
    assert!(!prices.is_empty(), "Price array cannot be empty");

    // debug_assert! — only in debug mode
    debug_assert!(prices.len() >= 2, "Need at least 2 prices for analysis");

    // panic! — always executes
    if prices.iter().any(|&p| p < 0.0) {
        panic!("Negative price detected!");
    }

    let avg = calculate_average(&prices);
    println!("Average price: {:.2}", avg);
}

fn calculate_average(prices: &[f64]) -> f64 {
    // assert_eq! — equality check
    // assert_ne! — inequality check
    assert_ne!(prices.len(), 0, "Division by zero!");

    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## unreachable!() — Unreachable Code

```rust
fn main() {
    let signal = generate_signal(42000.0, 42500.0);
    execute_signal(&signal);
}

fn generate_signal(current_price: f64, target_price: f64) -> &'static str {
    if current_price < target_price {
        "BUY"
    } else if current_price > target_price {
        "SELL"
    } else {
        "HOLD"
    }
}

fn execute_signal(signal: &str) {
    match signal {
        "BUY" => println!("Executing buy order"),
        "SELL" => println!("Executing sell order"),
        "HOLD" => println!("Waiting"),
        _ => unreachable!("generate_signal only returns BUY, SELL, or HOLD"),
    }
}
```

## todo!() and unimplemented!()

```rust
fn main() {
    let mut engine = TradingEngine::new();
    engine.start();
}

struct TradingEngine {
    is_running: bool,
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine { is_running: false }
    }

    fn start(&mut self) {
        self.is_running = true;
        println!("Engine started");

        // This is not yet implemented
        self.connect_to_exchange();
    }

    fn connect_to_exchange(&self) {
        // todo! — reminder that implementation is needed
        todo!("Implement exchange connection");
    }

    #[allow(dead_code)]
    fn execute_trade(&self, _symbol: &str, _quantity: f64) {
        // unimplemented! — functionality intentionally not implemented
        unimplemented!("API trading will be added in the next version");
    }
}
```

## Practical Example: Order Validator

```rust
fn main() {
    let order1 = Order {
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: Some(42000.0),
        order_type: OrderType::Limit,
    };

    let order2 = Order {
        symbol: "".to_string(), // Empty symbol — error!
        side: OrderSide::Sell,
        quantity: 1.0,
        price: None,
        order_type: OrderType::Market,
    };

    validate_and_submit(&order1);
    validate_and_submit(&order2); // This will panic
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: Option<f64>,
    order_type: OrderType,
}

fn validate_and_submit(order: &Order) {
    // Critical checks — if they fail, it's a bug
    assert!(!order.symbol.is_empty(), "Order symbol cannot be empty");
    assert!(order.quantity > 0.0, "Quantity must be positive");

    // Check order type and price consistency
    match order.order_type {
        OrderType::Limit => {
            if order.price.is_none() {
                panic!("Limit order requires a price!");
            }
            if order.price.unwrap() <= 0.0 {
                panic!("Limit order price must be positive!");
            }
        }
        OrderType::Market => {
            // Market order — price not needed, OK
        }
    }

    println!("Order is valid: {:?}", order);
    println!("Submitting order to exchange...");
}
```

## Panic vs Result: When to Use Which

```rust
fn main() {
    // panic! — for bugs and impossible situations
    // Result — for expected errors

    // Example with Result (correct for user input)
    match parse_price("42000.50") {
        Ok(price) => println!("Price: {}", price),
        Err(e) => println!("Parse error: {}", e),
    }

    // Example with panic! (for internal checks)
    let prices = vec![42000.0, 42100.0];
    let avg = safe_average(&prices);
    println!("Average: {}", avg);

    // This will panic — internal error
    let empty: Vec<f64> = vec![];
    let _ = safe_average(&empty);
}

// Result — for expected errors (invalid user input)
fn parse_price(input: &str) -> Result<f64, String> {
    input.parse::<f64>()
        .map_err(|_| format!("'{}' — invalid price", input))
}

// panic! — for bugs (empty array is a bug in calling code)
fn safe_average(prices: &[f64]) -> f64 {
    assert!(!prices.is_empty(), "BUG: price array should not be empty");
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Informative Panic Messages

```rust
fn main() {
    let order_id = "ORD-12345";
    let expected_status = "FILLED";
    let actual_status = "REJECTED";

    check_order_status(order_id, expected_status, actual_status);
}

fn check_order_status(order_id: &str, expected: &str, actual: &str) {
    if expected != actual {
        panic!(
            "\n\
            ╔══════════════════════════════════════════╗\n\
            ║        CRITICAL ORDER ERROR              ║\n\
            ╠══════════════════════════════════════════╣\n\
            ║ Order ID: {:<30} ║\n\
            ║ Expected status: {:<23} ║\n\
            ║ Received status: {:<23} ║\n\
            ╚══════════════════════════════════════════╝",
            order_id, expected, actual
        );
    }
}
```

## What We Learned

| Macro | When to Use |
|-------|-------------|
| `panic!` | Critical error, program cannot continue |
| `assert!` | Condition check (disabled in release) |
| `assert_eq!` | Check equality of two values |
| `assert_ne!` | Check inequality of two values |
| `debug_assert!` | Check only in debug mode |
| `unreachable!` | Code that should never execute |
| `todo!` | Placeholder for unimplemented code |
| `unimplemented!` | Functionality intentionally not implemented |

## Homework

1. Write a `RiskManager` struct with a `validate_trade()` method that uses `panic!` when risk management rules are violated (exceeding max position, too much risk per trade)

2. Create a `process_market_data()` function that uses `assert!` to check input data (prices are positive, volumes are correct, timestamps are in order)

3. Implement a `TradingState` enum with states (Idle, Trading, Paused, Error) and a `transition()` function that uses `unreachable!` for impossible state transitions

4. Write a module with `todo!()` placeholders for future implementation: WebSocket connection, order book processing, order execution

## Navigation

[← Previous day](../090-project-order-book/en.md) | [Next day →](../092-when-to-panic/en.md)
