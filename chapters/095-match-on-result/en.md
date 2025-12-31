# Day 95: match on Result — Handling Both Cases

## Trading Analogy

Imagine you're sending an order to an exchange. There are two possible outcomes: the order is **executed** (success) or **rejected** (error). A good trader doesn't just hope for the best — they **plan for both scenarios** in advance. What to do if the order succeeds? What to do if it fails?

The `match` construct in Rust forces you to **explicitly handle both cases**. It's like a trader's checklist: "If success — do X, if failure — do Y". The compiler won't let you forget any scenario!

## match Syntax for Result

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    match result {
        Ok(price) => println!("Price received: ${:.2}", price),
        Err(error) => println!("Error: {}", error),
    }
}
```

**Key points:**
- `Ok(value)` — extract the value on success
- `Err(error)` — extract the error on failure
- The compiler requires handling **both** variants

## Fetching Price from Exchange

```rust
fn main() {
    // Successful request
    match fetch_price("BTC") {
        Ok(price) => {
            println!("Current BTC price: ${:.2}", price);
            if price > 50000.0 {
                println!("Price above 50k — bull market!");
            }
        }
        Err(e) => {
            println!("Failed to get price: {}", e);
            println!("Using last known price...");
        }
    }

    // Failed request
    match fetch_price("INVALID") {
        Ok(price) => println!("Price: {}", price),
        Err(e) => println!("Error: {}", e),
    }
}

fn fetch_price(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        "SOL" => Ok(98.5),
        _ => Err(format!("Ticker '{}' not found", ticker)),
    }
}
```

## Placing Order with Full Handling

```rust
fn main() {
    let balance = 10000.0;

    // Attempt to place order
    match place_order("BTC", 0.5, 42000.0, balance) {
        Ok(order_id) => {
            println!("Order placed successfully!");
            println!("Order ID: {}", order_id);
            println!("Waiting for execution...");
        }
        Err(error) => {
            println!("Order rejected!");
            println!("Reason: {}", error);
            println!("Please check balance and order parameters.");
        }
    }

    // Attempt to place an order that's too large
    match place_order("BTC", 10.0, 42000.0, balance) {
        Ok(order_id) => println!("Order {}", order_id),
        Err(error) => println!("Error: {}", error),
    }
}

fn place_order(
    ticker: &str,
    quantity: f64,
    price: f64,
    balance: f64
) -> Result<String, String> {
    let total_cost = quantity * price;

    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }

    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }

    if total_cost > balance {
        return Err(format!(
            "Insufficient funds. Required: ${:.2}, available: ${:.2}",
            total_cost, balance
        ));
    }

    // Generate order ID
    Ok(format!("ORD-{}-{}", ticker, 12345))
}
```

## Returning Values from match

`match` is an expression, so it returns a value:

```rust
fn main() {
    let result = calculate_profit(42000.0, 43500.0, 0.5);

    // match returns a value
    let message = match result {
        Ok(profit) => format!("Profit: ${:.2}", profit),
        Err(e) => format!("Calculation error: {}", e),
    };

    println!("{}", message);

    // Can use directly in println!
    println!("Status: {}", match result {
        Ok(_) => "Success",
        Err(_) => "Failure",
    });
}

fn calculate_profit(entry: f64, exit: f64, qty: f64) -> Result<f64, String> {
    if entry <= 0.0 || exit <= 0.0 {
        return Err(String::from("Prices must be positive"));
    }
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    Ok((exit - entry) * qty)
}
```

## Nested Logic in match Arms

```rust
fn main() {
    let prices = vec![
        fetch_price_with_validation("BTC"),
        fetch_price_with_validation("ETH"),
        fetch_price_with_validation("INVALID"),
    ];

    for (i, result) in prices.iter().enumerate() {
        println!("\nRequest {}:", i + 1);
        match result {
            Ok(price) => {
                // Nested logic for success case
                if *price > 10000.0 {
                    println!("  High price: ${:.2}", price);
                } else if *price > 1000.0 {
                    println!("  Medium price: ${:.2}", price);
                } else {
                    println!("  Low price: ${:.2}", price);
                }
            }
            Err(error) => {
                // Can analyze error type
                if error.contains("not found") {
                    println!("  Unknown ticker");
                } else if error.contains("server") {
                    println!("  Server issue, try again later");
                } else {
                    println!("  Unknown error: {}", error);
                }
            }
        }
    }
}

fn fetch_price_with_validation(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Ticker '{}' not found", ticker)),
    }
}
```

## Handling Different Error Types

```rust
fn main() {
    // Test different scenarios
    let test_cases = vec![
        ("BTC", 1.0, 42000.0),    // Success
        ("BTC", -1.0, 42000.0),   // Invalid quantity
        ("BTC", 1.0, -100.0),     // Invalid price
        ("XXX", 1.0, 100.0),      // Invalid ticker
    ];

    for (ticker, qty, price) in test_cases {
        println!("\nAttempt: {} x {} @ ${}", ticker, qty, price);

        match validate_order(ticker, qty, price) {
            Ok(order) => {
                println!("  Order valid: {:?}", order);
            }
            Err(OrderError::InvalidTicker(t)) => {
                println!("  Ticker '{}' not supported", t);
            }
            Err(OrderError::InvalidQuantity(q)) => {
                println!("  Invalid quantity: {}", q);
            }
            Err(OrderError::InvalidPrice(p)) => {
                println!("  Invalid price: {}", p);
            }
        }
    }
}

#[derive(Debug)]
struct Order {
    ticker: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidTicker(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
}

fn validate_order(ticker: &str, quantity: f64, price: f64) -> Result<Order, OrderError> {
    // Check ticker
    let valid_tickers = ["BTC", "ETH", "SOL"];
    if !valid_tickers.contains(&ticker) {
        return Err(OrderError::InvalidTicker(ticker.to_string()));
    }

    // Check quantity
    if quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(quantity));
    }

    // Check price
    if price <= 0.0 {
        return Err(OrderError::InvalidPrice(price));
    }

    Ok(Order {
        ticker: ticker.to_string(),
        quantity,
        price,
    })
}
```

## Guards in match (Conditions)

```rust
fn main() {
    let results = vec![
        Ok(150.0),   // Good profit
        Ok(10.0),    // Small profit
        Ok(-50.0),   // Loss
        Err("Connection error".to_string()),
    ];

    for result in results {
        match result {
            Ok(pnl) if pnl > 100.0 => {
                println!("Excellent trade! PnL: ${:.2}", pnl);
            }
            Ok(pnl) if pnl > 0.0 => {
                println!("Profitable trade. PnL: ${:.2}", pnl);
            }
            Ok(pnl) if pnl < 0.0 => {
                println!("Losing trade. PnL: ${:.2}", pnl);
            }
            Ok(pnl) => {
                println!("Break-even trade. PnL: ${:.2}", pnl);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
```

## Practical Example: Position Size Calculator

```rust
fn main() {
    println!("=== Position Size Calculator ===\n");

    let scenarios = vec![
        (10000.0, 2.0, 42000.0, 41000.0),  // Normal
        (10000.0, 0.0, 42000.0, 41000.0),  // Zero risk
        (10000.0, 2.0, 42000.0, 42000.0),  // Entry = Stop
        (-5000.0, 2.0, 42000.0, 41000.0),  // Negative balance
    ];

    for (balance, risk_pct, entry, stop) in scenarios {
        println!("Balance: ${}, Risk: {}%, Entry: ${}, Stop: ${}",
                 balance, risk_pct, entry, stop);

        match calculate_position_size(balance, risk_pct, entry, stop) {
            Ok(size) => {
                let position_value = size * entry;
                println!("  Position size: {:.6} BTC", size);
                println!("  Position value: ${:.2}", position_value);
                println!("  Dollar risk: ${:.2}", balance * risk_pct / 100.0);
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
        println!();
    }
}

fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(String::from("Balance must be positive"));
    }

    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Risk must be between 0 and 100%"));
    }

    if entry_price <= 0.0 {
        return Err(String::from("Entry price must be positive"));
    }

    let risk_per_unit = (entry_price - stop_loss).abs();
    if risk_per_unit == 0.0 {
        return Err(String::from("Entry and stop loss cannot be equal"));
    }

    let risk_amount = balance * (risk_percent / 100.0);
    Ok(risk_amount / risk_per_unit)
}
```

## Combining Multiple Results

```rust
fn main() {
    // All operations successful
    match execute_trade_pipeline("BTC", 0.1, 10000.0) {
        Ok(trade) => {
            println!("Trade executed!");
            println!("  Ticker: {}", trade.ticker);
            println!("  Quantity: {}", trade.quantity);
            println!("  Price: ${:.2}", trade.price);
            println!("  ID: {}", trade.order_id);
        }
        Err(e) => {
            println!("Trade failed: {}", e);
        }
    }
}

#[derive(Debug)]
struct TradeResult {
    ticker: String,
    quantity: f64,
    price: f64,
    order_id: String,
}

fn execute_trade_pipeline(
    ticker: &str,
    quantity: f64,
    balance: f64,
) -> Result<TradeResult, String> {
    // Step 1: Get price
    let price = match fetch_current_price(ticker) {
        Ok(p) => p,
        Err(e) => return Err(format!("Price fetch error: {}", e)),
    };

    // Step 2: Check balance
    match check_balance(balance, quantity * price) {
        Ok(_) => {}
        Err(e) => return Err(format!("Balance error: {}", e)),
    };

    // Step 3: Submit order
    let order_id = match submit_order(ticker, quantity, price) {
        Ok(id) => id,
        Err(e) => return Err(format!("Order submission error: {}", e)),
    };

    Ok(TradeResult {
        ticker: ticker.to_string(),
        quantity,
        price,
        order_id,
    })
}

fn fetch_current_price(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2200.0),
        _ => Err(format!("Unknown ticker: {}", ticker)),
    }
}

fn check_balance(balance: f64, required: f64) -> Result<(), String> {
    if balance >= required {
        Ok(())
    } else {
        Err(format!("Insufficient funds: need {:.2}, have {:.2}", required, balance))
    }
}

fn submit_order(ticker: &str, quantity: f64, price: f64) -> Result<String, String> {
    if quantity > 0.0 && price > 0.0 {
        Ok(format!("ORD-{}-{}", ticker, 99999))
    } else {
        Err(String::from("Invalid order parameters"))
    }
}
```

## Comparison: match vs Other Approaches

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    // 1. match — full control
    match &result {
        Ok(price) => println!("match: Price ${:.2}", price),
        Err(e) => println!("match: Error {}", e),
    }

    // 2. if let — when only one case matters
    if let Ok(price) = &result {
        println!("if let: Price ${:.2}", price);
    }

    // 3. unwrap_or — default value
    let price = result.clone().unwrap_or(0.0);
    println!("unwrap_or: Price ${:.2}", price);

    // 4. map — transform successful value
    let doubled = result.clone().map(|p| p * 2.0);
    println!("map: {:?}", doubled);
}
```

## What We Learned

| Concept | Syntax | Use Case |
|---------|--------|----------|
| Basic match | `match result { Ok(v) => ..., Err(e) => ... }` | Full handling of both cases |
| Guard | `Ok(v) if v > 0.0 => ...` | Additional conditions |
| Nested match | `match { Ok(v) => match v {...} }` | Complex logic |
| Match as expression | `let x = match result {...}` | Getting a value |
| Error enums | `Err(MyError::Type)` | Typed errors |

## Exercises

1. **Order Validator**: Write a function that validates an order and returns `Result<Order, OrderError>`. Use `match` to handle all possible errors.

2. **Currency Converter**: Create a currency conversion function that can return an error for unknown currencies. Handle the result with `match`.

3. **Portfolio Analyzer**: Write a function that analyzes a portfolio and returns `Result<PortfolioStats, AnalysisError>`. Use guards to classify results.

## Homework

1. Implement a function `execute_trade_with_retry` that attempts to execute a trade and on failure retries up to 3 times, using `match` to analyze errors.

2. Create a `TradingError` type with variants: `NetworkError`, `InsufficientFunds`, `InvalidOrder`, `ExchangeError`. Write a function that returns `Result<Trade, TradingError>`, and handle all variants with `match`.

3. Write a function `analyze_multiple_trades(trades: Vec<Result<Trade, Error>>)` that counts successful and failed trades, total profit from successful trades, and outputs statistics.

4. Implement chain of responsibility: function `process_order` should sequentially call `validate_order`, `check_risk`, `check_balance`, `submit_to_exchange`, and on any error break the chain with an informative message.

## Navigation

[← Previous day](../094-result-deeper/en.md) | [Next day →](../096-if-let-result/en.md)
