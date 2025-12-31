# Day 96: if let with Result — When You Only Need Success

## Trading Analogy

Imagine you're waiting for an order to execute on an exchange. Sometimes you're only interested in the **successful** outcome — if the order filled, you want to see the trade details. If it didn't fill — you just move on, without analyzing the reasons.

This is like the difference between:
- **match** — "Show me everything: both success and failure reason"
- **if let** — "I only care about success, skip the rest"

## The Problem with match: Sometimes Too Much Code

Yesterday we learned to use `match` to handle `Result`:

```rust
fn main() {
    let order_result = execute_order("BTC/USDT", 0.1, 42000.0);

    match order_result {
        Ok(order_id) => println!("Order executed! ID: {}", order_id),
        Err(_) => {} // We don't care about the error — just ignore
    }
}

fn execute_order(pair: &str, amount: f64, price: f64) -> Result<String, String> {
    if price > 0.0 && amount > 0.0 {
        Ok(format!("ORD-{}-{}", pair, price as u64))
    } else {
        Err(String::from("Invalid parameters"))
    }
}
```

See that `Err(_) => {}`? That's an empty branch! We're forced to write it due to exhaustive matching, but it does nothing.

## if let: The Elegant Solution

`if let` allows handling **only one variant**, ignoring the rest:

```rust
fn main() {
    let order_result = execute_order("BTC/USDT", 0.1, 42000.0);

    // Handle ONLY success
    if let Ok(order_id) = order_result {
        println!("Order executed! ID: {}", order_id);
    }
    // Error? Just continue
}

fn execute_order(pair: &str, amount: f64, price: f64) -> Result<String, String> {
    if price > 0.0 && amount > 0.0 {
        Ok(format!("ORD-{}-{}", pair, price as u64))
    } else {
        Err(String::from("Invalid parameters"))
    }
}
```

Syntax: `if let Ok(variable) = result { ... }`

## if let with else: Handling Both Cases

Sometimes you need to do something on error too:

```rust
fn main() {
    let balance_result = get_balance("USDT");

    if let Ok(balance) = balance_result {
        println!("Balance: ${:.2}", balance);

        if balance > 1000.0 {
            println!("Enough for trading!");
        }
    } else {
        println!("Could not get balance, using cache");
        use_cached_balance();
    }
}

fn get_balance(asset: &str) -> Result<f64, String> {
    // Simulating API request
    if asset == "USDT" {
        Ok(5000.0)
    } else {
        Err(format!("Unknown asset: {}", asset))
    }
}

fn use_cached_balance() {
    println!("Using cached value: $4800.00");
}
```

## Getting the Error Value in else

If you need to know **what error occurred**, use `else if let`:

```rust
fn main() {
    let price_result = fetch_price("BTC/USDT");

    if let Ok(price) = price_result {
        println!("Current BTC price: ${:.2}", price);
    } else if let Err(error) = price_result {
        println!("Error fetching price: {}", error);
        // We can try an alternative source
    }
}

fn fetch_price(pair: &str) -> Result<f64, String> {
    Err(String::from("API timeout"))
}
```

Or a more idiomatic variant:

```rust
fn main() {
    let price_result: Result<f64, String> = fetch_price("BTC/USDT");

    if let Ok(price) = &price_result {
        println!("Current BTC price: ${:.2}", price);
    } else {
        // price_result is still available because we borrowed
        println!("Error: {:?}", price_result.err().unwrap());
    }
}

fn fetch_price(pair: &str) -> Result<f64, String> {
    Err(String::from("API timeout"))
}
```

## Practical Examples

### Example 1: Updating Portfolio Only on Success

```rust
fn main() {
    let mut portfolio_value = 10000.0;

    // Try to get current prices
    if let Ok(btc_price) = get_price("BTC") {
        let btc_holding = 0.5;
        portfolio_value += btc_price * btc_holding;
        println!("Added BTC value: ${:.2}", btc_price * btc_holding);
    }

    if let Ok(eth_price) = get_price("ETH") {
        let eth_holding = 10.0;
        portfolio_value += eth_price * eth_holding;
        println!("Added ETH value: ${:.2}", eth_price * eth_holding);
    }

    // SOL unavailable — just skip
    if let Ok(sol_price) = get_price("SOL") {
        let sol_holding = 100.0;
        portfolio_value += sol_price * sol_holding;
        println!("Added SOL value: ${:.2}", sol_price * sol_holding);
    }

    println!("Total portfolio value: ${:.2}", portfolio_value);
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Price not available for {}", symbol)),
    }
}
```

### Example 2: Validating Order Before Submission

```rust
fn main() {
    let order = Order {
        symbol: String::from("ETH/USDT"),
        side: String::from("BUY"),
        amount: 5.0,
        price: 2200.0,
    };

    // Validate and submit only if valid
    if let Ok(validated) = validate_order(&order) {
        println!("Order valid, submitting...");

        if let Ok(order_id) = send_to_exchange(&validated) {
            println!("Success! Order ID: {}", order_id);
        } else {
            println!("Failed to submit to exchange");
        }
    }
    // Invalid order? Do nothing
}

struct Order {
    symbol: String,
    side: String,
    amount: f64,
    price: f64,
}

struct ValidatedOrder {
    symbol: String,
    side: String,
    amount: f64,
    price: f64,
    timestamp: u64,
}

fn validate_order(order: &Order) -> Result<ValidatedOrder, String> {
    if order.amount <= 0.0 {
        return Err(String::from("Amount must be positive"));
    }
    if order.price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }

    Ok(ValidatedOrder {
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        amount: order.amount,
        price: order.price,
        timestamp: 1234567890,
    })
}

fn send_to_exchange(order: &ValidatedOrder) -> Result<String, String> {
    Ok(format!("ORD-{}-{}", order.symbol, order.timestamp))
}
```

### Example 3: Conditional Logging of Successful Trades

```rust
fn main() {
    let trades = vec![
        execute_trade("BTC", 0.1, 43000.0),
        execute_trade("ETH", 0.0, 2200.0),  // Error: zero volume
        execute_trade("SOL", 10.0, 95.0),
        execute_trade("DOGE", 1000.0, -0.1), // Error: negative price
    ];

    println!("=== Successful Trades ===");
    for (i, trade) in trades.iter().enumerate() {
        if let Ok(details) = trade {
            println!("Trade #{}: {}", i + 1, details);
        }
    }

    println!("\n=== Failed Trades ===");
    for (i, trade) in trades.iter().enumerate() {
        if let Err(error) = trade {
            println!("Trade #{} failed: {}", i + 1, error);
        }
    }
}

fn execute_trade(symbol: &str, amount: f64, price: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err(format!("{}: Invalid amount", symbol));
    }
    if price <= 0.0 {
        return Err(format!("{}: Invalid price", symbol));
    }

    Ok(format!("{} {} @ ${:.2}", symbol, amount, price))
}
```

### Example 4: Parsing Trading Data

```rust
fn main() {
    let raw_prices = vec!["42500.50", "invalid", "43100.00", "N/A", "42800.75"];

    let mut valid_prices = Vec::new();

    for raw in &raw_prices {
        if let Ok(price) = parse_price(raw) {
            valid_prices.push(price);
        }
        // Invalid prices are simply skipped
    }

    if !valid_prices.is_empty() {
        let avg: f64 = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("Valid prices: {}", valid_prices.len());
        println!("Average price: ${:.2}", avg);
    }
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", s))
}
```

## if let vs match: When to Use What

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    // Use if let when:
    // 1. You only need one variant
    if let Ok(price) = result {
        println!("Price: {}", price);
    }

    // 2. Error doesn't matter — just skip
    if let Ok(price) = get_optional_indicator() {
        apply_indicator(price);
    }

    // Use match when:
    // 1. You need both variants with different logic
    match validate_trade(100.0, 42000.0) {
        Ok(trade) => execute(trade),
        Err(e) => log_error(e),
    }

    // 2. You need to return a value
    let status = match process_order() {
        Ok(_) => "SUCCESS",
        Err(_) => "FAILED",
    };
    println!("Status: {}", status);
}

fn get_optional_indicator() -> Result<f64, String> {
    Ok(50.0) // RSI value
}

fn apply_indicator(_value: f64) {
    println!("Applying indicator...");
}

fn validate_trade(_amount: f64, _price: f64) -> Result<String, String> {
    Ok(String::from("Trade validated"))
}

fn execute(_trade: String) {
    println!("Executing trade");
}

fn log_error(_e: String) {
    println!("Logging error");
}

fn process_order() -> Result<(), String> {
    Ok(())
}
```

## Chaining if let

```rust
fn main() {
    // Sequential checks
    if let Ok(balance) = get_balance() {
        if let Ok(price) = get_current_price() {
            if let Ok(max_amount) = calculate_max_position(balance, price) {
                println!("Can buy up to {} BTC", max_amount);
            }
        }
    }

    // This can get verbose — then it's better to use ?
    // or combinators (more on that later)
}

fn get_balance() -> Result<f64, String> {
    Ok(10000.0)
}

fn get_current_price() -> Result<f64, String> {
    Ok(43000.0)
}

fn calculate_max_position(balance: f64, price: f64) -> Result<f64, String> {
    if price > 0.0 {
        Ok(balance / price)
    } else {
        Err(String::from("Invalid price"))
    }
}
```

## Destructuring in if let

```rust
fn main() {
    let trade_result = get_trade_details();

    // Destructure tuple inside Ok
    if let Ok((symbol, amount, price)) = trade_result {
        let value = amount * price;
        println!("{}: {} @ ${:.2} = ${:.2}", symbol, amount, price, value);
    }
}

fn get_trade_details() -> Result<(String, f64, f64), String> {
    Ok((String::from("BTC"), 0.5, 43000.0))
}
```

## What We Learned

| Construct | When to Use |
|-----------|-------------|
| `if let Ok(x) = result` | Only interested in success |
| `if let Err(e) = result` | Only interested in error |
| `if let Ok(x) = result { } else { }` | Two cases, but error logic is simple |
| `match result { Ok => ..., Err => ... }` | Need different logic for both cases |

## Exercises

### Exercise 1: Loading Configuration
Write a program that tries to load configuration. If successful — use it, if not — apply defaults.

```rust
fn main() {
    // Your code here
    // Use if let with else to handle load_config()
}

fn load_config() -> Result<TradingConfig, String> {
    // Implement
    todo!()
}

struct TradingConfig {
    max_position_size: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}
```

### Exercise 2: Filtering Valid Orders
Given a vector of order parsing results. Using `if let`, collect only successfully parsed orders.

```rust
fn main() {
    let raw_orders = vec![
        "BUY,BTC,0.5,43000",
        "invalid order",
        "SELL,ETH,10,2200",
        "BUY,SOL,-5,95", // Invalid: negative amount
    ];

    // Your code: collect valid orders into a vector
}
```

### Exercise 3: Updating Positions
Write a function that updates portfolio positions. If price fetching fails — the position is not updated (but the program continues running).

```rust
fn main() {
    let mut positions = vec![
        Position { symbol: "BTC".to_string(), amount: 0.5, value: 0.0 },
        Position { symbol: "ETH".to_string(), amount: 10.0, value: 0.0 },
        Position { symbol: "UNKNOWN".to_string(), amount: 100.0, value: 0.0 },
    ];

    update_portfolio_values(&mut positions);

    for pos in &positions {
        println!("{}: {} units = ${:.2}", pos.symbol, pos.amount, pos.value);
    }
}

struct Position {
    symbol: String,
    amount: f64,
    value: f64,
}

fn update_portfolio_values(positions: &mut Vec<Position>) {
    // Your code here
    // Use if let to update only those positions
    // for which price fetching succeeded
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43000.0),
        "ETH" => Ok(2200.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}
```

## Homework

1. Create a multi-exchange monitoring system. Use `if let` to handle responses — if an exchange responds, display the data; if not — skip without errors.

2. Implement a function to calculate average PnL that takes a vector of `Result<f64, String>` (trade results) and returns the average of only successful trades.

3. Write a trading log parser that reads lines in format "TIMESTAMP,SYMBOL,SIDE,AMOUNT,PRICE" and collects only valid entries, ignoring corrupted lines.

4. Create an automatic portfolio rebalancer function that uses `if let` to verify each step: fetch balances → calculate shares → generate orders.

## Navigation

[← Previous day](../095-match-on-result/en.md) | [Next day →](../097-while-let/en.md)
