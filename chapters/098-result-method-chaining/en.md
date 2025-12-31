# Day 98: Result Method Chaining

## Trading Analogy

Imagine executing a complex order on an exchange: first you need to check the balance, then validate the price, then verify limits, and finally place the order. Each step can fail. Instead of writing nested `match` blocks for each check, we can **chain** operations together — if any step fails, the entire chain stops and returns an error.

It's like an assembly line at a factory: if a part is defective at any stage, it gets removed from the line and doesn't proceed further.

## Basic Result Methods for Chaining

### `and_then` — Continue on Success

```rust
fn main() {
    let result = parse_price("42000.50")
        .and_then(|price| validate_price(price))
        .and_then(|price| check_balance(price, 50000.0));

    match result {
        Ok(price) => println!("Price is valid: ${:.2}", price),
        Err(e) => println!("Error: {}", e),
    }
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", s))
}

fn validate_price(price: f64) -> Result<f64, String> {
    if price <= 0.0 {
        Err(String::from("Price must be positive"))
    } else if price > 1_000_000.0 {
        Err(String::from("Price is too high"))
    } else {
        Ok(price)
    }
}

fn check_balance(price: f64, balance: f64) -> Result<f64, String> {
    if price > balance {
        Err(String::from("Insufficient funds"))
    } else {
        Ok(price)
    }
}
```

### `map` — Transform the Success Value

```rust
fn main() {
    let result = parse_quantity("100")
        .map(|qty| qty * 1.1)  // Add 10% buffer
        .map(|qty| qty.ceil() as u64);  // Round up

    println!("Quantity with buffer: {:?}", result);
}

fn parse_quantity(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}'", s))
}
```

### `map_err` — Transform the Error Value

```rust
fn main() {
    let result: Result<f64, TradingError> = "invalid"
        .parse::<f64>()
        .map_err(|_| TradingError::InvalidPrice);

    match result {
        Ok(price) => println!("Price: {}", price),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[derive(Debug)]
enum TradingError {
    InvalidPrice,
    InsufficientFunds,
    OrderRejected,
}
```

## Complete Order Processing Pipeline

```rust
fn main() {
    // Successful scenario
    let order_result = process_order("42000.50", "0.5", 25000.0);
    println!("Result 1: {:?}", order_result);

    // Parse error
    let order_result = process_order("invalid", "0.5", 25000.0);
    println!("Result 2: {:?}", order_result);

    // Insufficient funds
    let order_result = process_order("42000.50", "0.5", 1000.0);
    println!("Result 3: {:?}", order_result);
}

#[derive(Debug)]
struct Order {
    price: f64,
    quantity: f64,
    total: f64,
    fee: f64,
}

#[derive(Debug)]
enum OrderError {
    ParseError(String),
    ValidationError(String),
    InsufficientFunds { required: f64, available: f64 },
    RiskLimitExceeded,
}

fn process_order(price_str: &str, qty_str: &str, balance: f64) -> Result<Order, OrderError> {
    parse_order_params(price_str, qty_str)
        .and_then(|(price, qty)| validate_order(price, qty))
        .and_then(|(price, qty)| check_funds(price, qty, balance))
        .and_then(|(price, qty)| check_risk_limits(price, qty))
        .map(|(price, qty)| create_order(price, qty))
}

fn parse_order_params(price_str: &str, qty_str: &str) -> Result<(f64, f64), OrderError> {
    let price = price_str.parse::<f64>()
        .map_err(|_| OrderError::ParseError(format!("Invalid price: {}", price_str)))?;

    let quantity = qty_str.parse::<f64>()
        .map_err(|_| OrderError::ParseError(format!("Invalid quantity: {}", qty_str)))?;

    Ok((price, quantity))
}

fn validate_order(price: f64, quantity: f64) -> Result<(f64, f64), OrderError> {
    if price <= 0.0 {
        return Err(OrderError::ValidationError("Price must be positive".into()));
    }
    if quantity <= 0.0 {
        return Err(OrderError::ValidationError("Quantity must be positive".into()));
    }
    if quantity < 0.001 {
        return Err(OrderError::ValidationError("Minimum quantity: 0.001".into()));
    }
    Ok((price, quantity))
}

fn check_funds(price: f64, quantity: f64, balance: f64) -> Result<(f64, f64), OrderError> {
    let required = price * quantity * 1.001;  // +0.1% for fees
    if required > balance {
        Err(OrderError::InsufficientFunds { required, available: balance })
    } else {
        Ok((price, quantity))
    }
}

fn check_risk_limits(price: f64, quantity: f64) -> Result<(f64, f64), OrderError> {
    let position_value = price * quantity;
    if position_value > 100_000.0 {
        Err(OrderError::RiskLimitExceeded)
    } else {
        Ok((price, quantity))
    }
}

fn create_order(price: f64, quantity: f64) -> Order {
    let total = price * quantity;
    let fee = total * 0.001;
    Order { price, quantity, total, fee }
}
```

## Combinators for Complex Scenarios

### `or_else` — Try an Alternative on Error

```rust
fn main() {
    // Try to get price from different sources
    let price = get_price_from_exchange("binance")
        .or_else(|_| get_price_from_exchange("coinbase"))
        .or_else(|_| get_cached_price());

    println!("Price: {:?}", price);
}

fn get_price_from_exchange(exchange: &str) -> Result<f64, String> {
    match exchange {
        "binance" => Err("Binance unavailable".into()),
        "coinbase" => Ok(42150.0),
        _ => Err("Unknown exchange".into()),
    }
}

fn get_cached_price() -> Result<f64, String> {
    Ok(42000.0)  // Cached price as fallback
}
```

### `unwrap_or_else` — Default Value with Lazy Evaluation

```rust
fn main() {
    let price = parse_price("invalid")
        .unwrap_or_else(|_| get_default_price());

    println!("Price: {}", price);
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse().map_err(|_| "Parse error".into())
}

fn get_default_price() -> f64 {
    println!("Getting default price...");
    42000.0
}
```

### Combining with the `?` Operator

```rust
fn main() {
    match execute_trading_strategy() {
        Ok(profit) => println!("Profit: ${:.2}", profit),
        Err(e) => println!("Strategy failed: {}", e),
    }
}

fn execute_trading_strategy() -> Result<f64, String> {
    let prices = fetch_prices()?;
    let signal = analyze_prices(&prices)?;
    let order = create_trade_order(signal)?;
    let result = execute_order(&order)?;
    Ok(result)
}

fn fetch_prices() -> Result<Vec<f64>, String> {
    Ok(vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0])
}

fn analyze_prices(prices: &[f64]) -> Result<TradeSignal, String> {
    if prices.is_empty() {
        return Err("No data for analysis".into());
    }

    let avg: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    let last = prices.last().unwrap();

    if *last > avg * 1.01 {
        Ok(TradeSignal::Buy { price: *last })
    } else if *last < avg * 0.99 {
        Ok(TradeSignal::Sell { price: *last })
    } else {
        Err("No clear signal".into())
    }
}

#[derive(Debug)]
enum TradeSignal {
    Buy { price: f64 },
    Sell { price: f64 },
}

fn create_trade_order(signal: TradeSignal) -> Result<TradeOrder, String> {
    match signal {
        TradeSignal::Buy { price } => Ok(TradeOrder {
            side: "BUY".into(),
            price,
            quantity: 0.1,
        }),
        TradeSignal::Sell { price } => Ok(TradeOrder {
            side: "SELL".into(),
            price,
            quantity: 0.1,
        }),
    }
}

#[derive(Debug)]
struct TradeOrder {
    side: String,
    price: f64,
    quantity: f64,
}

fn execute_order(order: &TradeOrder) -> Result<f64, String> {
    println!("Executing order: {:?}", order);
    // Simulation: small profit
    Ok(order.price * order.quantity * 0.02)
}
```

## Practical Example: Portfolio Analysis

```rust
fn main() {
    let portfolio_data = vec![
        ("BTC", "42000.50", "0.5"),
        ("ETH", "2200.00", "5.0"),
        ("INVALID", "abc", "1.0"),  // Bad data
    ];

    let results: Vec<_> = portfolio_data
        .iter()
        .map(|(symbol, price, qty)| {
            analyze_position(symbol, price, qty)
        })
        .collect();

    for (symbol, result) in portfolio_data.iter().zip(results.iter()) {
        match result {
            Ok(analysis) => println!("{}: ${:.2}", symbol.0, analysis.value),
            Err(e) => println!("{}: Error - {}", symbol.0, e),
        }
    }
}

#[derive(Debug)]
struct PositionAnalysis {
    symbol: String,
    price: f64,
    quantity: f64,
    value: f64,
    weight: f64,
}

fn analyze_position(symbol: &str, price_str: &str, qty_str: &str) -> Result<PositionAnalysis, String> {
    let price = price_str.parse::<f64>()
        .map_err(|_| format!("Invalid price for {}", symbol))?;

    let quantity = qty_str.parse::<f64>()
        .map_err(|_| format!("Invalid quantity for {}", symbol))?;

    let value = price * quantity;
    let weight = 0.0;  // Will be calculated later

    Ok(PositionAnalysis {
        symbol: symbol.to_string(),
        price,
        quantity,
        value,
        weight,
    })
}
```

## Chaining for Risk Management

```rust
fn main() {
    let trade = TradeRequest {
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
        side: Side::Buy,
    };

    let account = Account {
        balance: 25000.0,
        max_position_size: 20000.0,
        daily_loss_limit: 1000.0,
        current_daily_loss: 200.0,
    };

    let result = validate_trade(&trade, &account)
        .and_then(|t| check_position_size(t, &account))
        .and_then(|t| check_daily_loss_limit(t, &account))
        .and_then(|t| calculate_risk_metrics(t))
        .map(|metrics| format_risk_report(&trade, &metrics));

    match result {
        Ok(report) => println!("{}", report),
        Err(e) => println!("Trade rejected: {:?}", e),
    }
}

#[derive(Debug, Clone)]
struct TradeRequest {
    symbol: String,
    price: f64,
    quantity: f64,
    side: Side,
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

struct Account {
    balance: f64,
    max_position_size: f64,
    daily_loss_limit: f64,
    current_daily_loss: f64,
}

#[derive(Debug)]
enum RiskError {
    InsufficientBalance,
    PositionTooLarge,
    DailyLossLimitReached,
    InvalidTrade(String),
}

struct RiskMetrics {
    position_value: f64,
    risk_percentage: f64,
    max_loss: f64,
}

fn validate_trade(trade: &TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let required = trade.price * trade.quantity;
    if required > account.balance {
        Err(RiskError::InsufficientBalance)
    } else {
        Ok(trade.clone())
    }
}

fn check_position_size(trade: TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let position_value = trade.price * trade.quantity;
    if position_value > account.max_position_size {
        Err(RiskError::PositionTooLarge)
    } else {
        Ok(trade)
    }
}

fn check_daily_loss_limit(trade: TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let remaining = account.daily_loss_limit - account.current_daily_loss;
    let potential_loss = trade.price * trade.quantity * 0.02;  // Max 2% loss

    if potential_loss > remaining {
        Err(RiskError::DailyLossLimitReached)
    } else {
        Ok(trade)
    }
}

fn calculate_risk_metrics(trade: TradeRequest) -> Result<RiskMetrics, RiskError> {
    let position_value = trade.price * trade.quantity;
    let risk_percentage = 2.0;  // Fixed risk
    let max_loss = position_value * (risk_percentage / 100.0);

    Ok(RiskMetrics {
        position_value,
        risk_percentage,
        max_loss,
    })
}

fn format_risk_report(trade: &TradeRequest, metrics: &RiskMetrics) -> String {
    format!(
        "╔══════════════════════════════════╗\n\
         ║        RISK REPORT               ║\n\
         ╠══════════════════════════════════╣\n\
         ║ Symbol:         {:>17} ║\n\
         ║ Position Size:  ${:>14.2} ║\n\
         ║ Risk:           {:>14.1}% ║\n\
         ║ Max Loss:       ${:>14.2} ║\n\
         ║ Status:         {:>17} ║\n\
         ╚══════════════════════════════════╝",
        trade.symbol,
        metrics.position_value,
        metrics.risk_percentage,
        metrics.max_loss,
        "APPROVED"
    )
}
```

## What We Learned

| Method | Description | When to Use |
|--------|-------------|-------------|
| `and_then` | Chain Result-returning functions | Sequential operations |
| `map` | Transform Ok value | Simple transformations |
| `map_err` | Transform Err value | Error type conversion |
| `or_else` | Alternative on error | Fallback strategies |
| `unwrap_or_else` | Default value | Lazy defaults |
| `?` | Early return on error | Clean chains in functions |

## Exercises

1. **Order Validator**: Create a chain for validating an exchange order with checks: parsing → price validation → quantity validation → balance check → limit check.

2. **Multi-Exchange Parser**: Write a function that tries to get a price from multiple exchanges in sequence using `or_else`.

3. **Position Calculator**: Create a chain for calculating position size: parse parameters → calculate risk → determine size → validate result.

4. **Portfolio Processor**: Write a function that processes a list of positions, collecting successful results and logging errors.

## Homework

1. Implement a complete trading strategy execution pipeline with method chaining: fetch data → analyze → generate signal → calculate size → execute → report.

2. Create a fallback data source system that sequentially tries: primary API → secondary API → cache → default value.

3. Write a `batch_process_orders` function that processes a vector of orders, separating results into successful and failed.

4. Implement a chain of checks for a risk management system where each check can add a warning even on success.

## Navigation

[← Previous day](../097-result-advanced/en.md) | [Next day →](../099-custom-error-types/en.md)
