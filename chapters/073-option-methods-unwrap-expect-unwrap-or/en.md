# Day 73: Option Methods — unwrap, expect, unwrap_or

## Trading Analogy

Imagine you're requesting an asset price from an exchange. Sometimes the price exists, and sometimes it doesn't (trading halted, asset delisted, server not responding). The `Option` type in Rust represents exactly this situation: a value either exists (`Some`) or doesn't (`None`).

But what do you do when you need to **extract** the value from an `Option`? There are several methods for this:

- **unwrap** — "I'm certain the price exists. If not — let the program crash!"
- **expect** — "I'm certain the price exists. If not — crash with a clear message!"
- **unwrap_or** — "Give me the price, or if missing — use a default value"

## The unwrap Method — Dangerous but Fast

`unwrap()` extracts the value from `Some` or **panics** on `None`:

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);
    let value = price.unwrap();  // 42000.0
    println!("BTC price: ${}", value);

    let missing_price: Option<f64> = None;
    // let crash = missing_price.unwrap();  // PANIC! thread 'main' panicked
}
```

### When to Use unwrap

```rust
fn main() {
    // 1. In tests — test panic = test failure
    let test_price: Option<f64> = get_test_price();
    assert_eq!(test_price.unwrap(), 42000.0);

    // 2. When None is logically impossible
    let prices = vec![42000.0, 42100.0, 42050.0];
    let first = prices.first().unwrap();  // Vector is not empty — first element 100% exists
    println!("First price: ${}", first);

    // 3. In prototypes/quick scripts
    let quick_calc = Some(100.0 * 1.05).unwrap();
    println!("Result: {}", quick_calc);
}

fn get_test_price() -> Option<f64> {
    Some(42000.0)
}
```

### Dangers of unwrap in Trading

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH"];

    // DANGEROUS: what if index is out of bounds?
    // let asset = portfolio.get(5).unwrap();  // PANIC!

    // DANGEROUS in production: API might return None
    // let price = fetch_price("UNKNOWN_COIN").unwrap();  // PANIC!
}
```

## The expect Method — unwrap with a Message

`expect(msg)` works like `unwrap`, but outputs your message on panic:

```rust
fn main() {
    let btc_price: Option<f64> = Some(42000.0);
    let price = btc_price.expect("BTC price should exist in portfolio");
    println!("BTC: ${}", price);

    // Useful for debugging
    let eth_price: Option<f64> = None;
    // let price = eth_price.expect("ETH price missing - check API connection");
    // thread 'main' panicked at 'ETH price missing - check API connection'
}
```

### Example: Loading Configuration

```rust
fn main() {
    let config = load_trading_config();

    let api_key = config.api_key
        .expect("API key must be set in config.toml");

    let max_position = config.max_position_size
        .expect("Max position size required for risk management");

    println!("Config loaded: API key exists, max position: {}", max_position);
}

struct TradingConfig {
    api_key: Option<String>,
    max_position_size: Option<f64>,
}

fn load_trading_config() -> TradingConfig {
    TradingConfig {
        api_key: Some(String::from("secret_key_123")),
        max_position_size: Some(10000.0),
    }
}
```

## The unwrap_or Method — Default Value

`unwrap_or(default)` returns the value from `Some` or the specified default value:

```rust
fn main() {
    // Price exists
    let btc_price: Option<f64> = Some(42000.0);
    let price = btc_price.unwrap_or(0.0);
    println!("BTC: ${}", price);  // 42000.0

    // Price doesn't exist — use default
    let unknown_price: Option<f64> = None;
    let price = unknown_price.unwrap_or(0.0);
    println!("Unknown: ${}", price);  // 0.0
}
```

### Practical Applications

```rust
fn main() {
    // Exchange fee: if not specified, use standard 0.1%
    let custom_fee: Option<f64> = None;
    let fee_percent = custom_fee.unwrap_or(0.1);
    println!("Fee: {}%", fee_percent);

    // Stop-loss: if not set, use 2% below entry price
    let entry_price = 42000.0;
    let stop_loss: Option<f64> = None;
    let actual_stop = stop_loss.unwrap_or(entry_price * 0.98);
    println!("Stop-loss: ${}", actual_stop);  // 41160.0

    // Quantity: minimum 1 if not specified
    let quantity: Option<u32> = None;
    let qty = quantity.unwrap_or(1);
    println!("Quantity: {}", qty);
}
```

## The unwrap_or_else Method — Lazy Evaluation

`unwrap_or_else(|| ...)` computes the default value only if `None`:

```rust
fn main() {
    let cached_price: Option<f64> = None;

    // fetch_live_price is called ONLY if cached_price == None
    let price = cached_price.unwrap_or_else(|| fetch_live_price("BTC"));
    println!("Price: ${}", price);

    // With unwrap_or the function is ALWAYS called (even if there's Some)
    let cached = Some(42000.0);
    let price = cached.unwrap_or(fetch_live_price("BTC"));  // fetch IS called!
    let price = cached.unwrap_or_else(|| fetch_live_price("BTC"));  // fetch NOT called
}

fn fetch_live_price(symbol: &str) -> f64 {
    println!("Fetching price for {}...", symbol);
    42500.0  // Simulating API request
}
```

### When to Use unwrap_or_else

```rust
fn main() {
    // 1. Expensive computations
    let cached_sma: Option<f64> = Some(42100.0);
    let sma = cached_sma.unwrap_or_else(|| calculate_sma(&get_prices(), 20));

    // 2. Side effects (logging)
    let price: Option<f64> = None;
    let value = price.unwrap_or_else(|| {
        log_missing_price();
        0.0
    });

    // 3. Dependency on other data
    let balance = 10000.0;
    let position_size: Option<f64> = None;
    let size = position_size.unwrap_or_else(|| balance * 0.02);  // 2% of balance
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return 0.0;
    }
    prices.iter().rev().take(period).sum::<f64>() / period as f64
}

fn get_prices() -> Vec<f64> {
    vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0]
}

fn log_missing_price() {
    println!("[WARN] Price is missing, using default");
}
```

## The unwrap_or_default Method — Default Trait

`unwrap_or_default()` uses the type's default value:

```rust
fn main() {
    // For numbers default = 0
    let volume: Option<f64> = None;
    let vol = volume.unwrap_or_default();  // 0.0
    println!("Volume: {}", vol);

    // For strings default = ""
    let symbol: Option<String> = None;
    let sym = symbol.unwrap_or_default();  // ""
    println!("Symbol: '{}'", sym);

    // For vectors default = []
    let trades: Option<Vec<f64>> = None;
    let t = trades.unwrap_or_default();  // []
    println!("Trades: {:?}", t);

    // For bool default = false
    let is_active: Option<bool> = None;
    let active = is_active.unwrap_or_default();  // false
    println!("Active: {}", active);
}
```

## Comparing Methods

```rust
fn main() {
    let price: Option<f64> = None;

    // unwrap — panics on None
    // let v = price.unwrap();  // PANIC!

    // expect — panic with message
    // let v = price.expect("Price required");  // PANIC with message

    // unwrap_or — specific value
    let v = price.unwrap_or(0.0);  // 0.0

    // unwrap_or_else — lazy evaluation
    let v = price.unwrap_or_else(|| calculate_default());  // function call

    // unwrap_or_default — Default trait
    let v = price.unwrap_or_default();  // 0.0 (default for f64)
}

fn calculate_default() -> f64 {
    println!("Calculating default...");
    42000.0
}
```

## Practical Example: Portfolio Calculator

```rust
fn main() {
    let portfolio = Portfolio {
        btc: Some(0.5),
        eth: Some(10.0),
        sol: None,
        dot: Some(100.0),
    };

    let prices = Prices {
        btc: Some(42000.0),
        eth: Some(2200.0),
        sol: None,  // No price data
        dot: Some(7.5),
    };

    let total = calculate_portfolio_value(&portfolio, &prices);
    println!("Total portfolio value: ${:.2}", total);
}

struct Portfolio {
    btc: Option<f64>,
    eth: Option<f64>,
    sol: Option<f64>,
    dot: Option<f64>,
}

struct Prices {
    btc: Option<f64>,
    eth: Option<f64>,
    sol: Option<f64>,
    dot: Option<f64>,
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &Prices) -> f64 {
    let btc_value = portfolio.btc.unwrap_or(0.0) * prices.btc.unwrap_or(0.0);
    let eth_value = portfolio.eth.unwrap_or(0.0) * prices.eth.unwrap_or(0.0);
    let sol_value = portfolio.sol.unwrap_or(0.0) * prices.sol.unwrap_or(0.0);
    let dot_value = portfolio.dot.unwrap_or(0.0) * prices.dot.unwrap_or(0.0);

    btc_value + eth_value + sol_value + dot_value
}
```

## Practical Example: Order System

```rust
fn main() {
    let order1 = Order {
        symbol: String::from("BTC"),
        price: Some(42000.0),
        quantity: 0.5,
        stop_loss: Some(41000.0),
        take_profit: None,
    };

    let order2 = Order {
        symbol: String::from("ETH"),
        price: None,  // Market order
        quantity: 10.0,
        stop_loss: None,
        take_profit: Some(2500.0),
    };

    process_order(&order1);
    process_order(&order2);
}

struct Order {
    symbol: String,
    price: Option<f64>,      // None = market order
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

fn process_order(order: &Order) {
    println!("\n=== Processing Order: {} ===", order.symbol);

    // Price: use market price if not specified
    let execution_price = order.price.unwrap_or_else(|| {
        println!("No limit price, fetching market price...");
        get_market_price(&order.symbol)
    });
    println!("Execution price: ${:.2}", execution_price);

    // Stop-loss: 5% below price if not specified
    let stop = order.stop_loss.unwrap_or(execution_price * 0.95);
    println!("Stop-loss: ${:.2}", stop);

    // Take-profit: 10% above price if not specified
    let tp = order.take_profit.unwrap_or(execution_price * 1.10);
    println!("Take-profit: ${:.2}", tp);

    // Trade risk
    let risk = (execution_price - stop) * order.quantity;
    println!("Risk: ${:.2}", risk);
}

fn get_market_price(symbol: &str) -> f64 {
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        _ => 0.0,
    }
}
```

## What We Learned

| Method | Behavior on None | When to Use |
|--------|-----------------|-------------|
| `unwrap()` | Panic | Tests, prototypes, 100% certainty |
| `expect(msg)` | Panic with message | Required values, debugging |
| `unwrap_or(val)` | Returns `val` | Simple defaults |
| `unwrap_or_else(f)` | Calls `f()` | Expensive computations, side effects |
| `unwrap_or_default()` | `Default::default()` | Standard zero values |

## Exercises

### Exercise 1: Fee Calculator
Write a function that calculates trade fee. If fee is not specified in the order, use standard 0.1%:

```rust
fn calculate_fee(trade_value: f64, custom_fee: Option<f64>) -> f64 {
    // Your code here
}
```

### Exercise 2: Safe Price Retrieval
Create a function that returns asset price from cache or fetches from API:

```rust
fn get_price(symbol: &str, cache: &HashMap<String, f64>) -> f64 {
    // Use unwrap_or_else for lazy loading
}
```

### Exercise 3: Order Validation
Write an order validation function that uses expect for required fields:

```rust
fn validate_order(order: &Order) -> bool {
    // Use expect for critical fields
}
```

### Exercise 4: Portfolio Statistics
Create a function to calculate portfolio statistics with default values:

```rust
fn portfolio_stats(assets: &[Option<f64>]) -> (f64, f64, f64) {
    // Return (total, average, count), ignoring None
}
```

## Homework

1. Implement a price caching system with `unwrap_or_else` for lazy loading

2. Create a trading bot configurator with `expect` for required parameters and `unwrap_or` for optional ones

3. Write a portfolio PnL calculation function where prices and quantities may be missing

4. Implement a trading log parser where some fields may be empty

## Navigation

[← Previous day: Option — Price Might Be Missing](../072-option-price-might-be-missing/en.md) | [Next day: Option with map and and_then →](../074-option-map-and-then/en.md)
