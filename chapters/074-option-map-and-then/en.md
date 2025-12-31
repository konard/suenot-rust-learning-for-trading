# Day 74: Option with map and and_then

## Trading Analogy

Imagine analyzing a trade through a chain of operations:
1. Get the current asset price (might be unavailable)
2. If price exists — calculate position size
3. If size is calculated — determine potential profit
4. If everything succeeds — display the result

Each step can fail: exchange not responding, insufficient data, calculation error. Instead of endless `if let Some(x)` checks, Rust offers elegant methods **map** and **and_then** for chaining transformations.

**map** — transforms the value inside Option if it exists (like converting price from one currency to another).

**and_then** — applies a function that itself returns an Option (like checking if data exists for the next step).

## The map Method

`map` applies a function to the value inside `Some`, leaving `None` unchanged:

```rust
fn main() {
    let btc_price: Option<f64> = Some(42000.0);
    let no_price: Option<f64> = None;

    // Convert price to EUR (rate 0.92)
    let btc_eur = btc_price.map(|price| price * 0.92);
    let no_eur = no_price.map(|price| price * 0.92);

    println!("BTC in EUR: {:?}", btc_eur);  // Some(38640.0)
    println!("No price in EUR: {:?}", no_eur);  // None
}
```

### Chaining map

```rust
fn main() {
    let entry_price: Option<f64> = Some(42000.0);

    // Chain of transformations
    let result = entry_price
        .map(|price| price * 1.05)      // +5% take profit
        .map(|tp| tp * 0.5)             // Position size
        .map(|value| value - 50.0);     // Minus fee

    println!("Result: {:?}", result);  // Some(21975.0)

    // If initial value is None, entire chain returns None
    let no_price: Option<f64> = None;
    let no_result = no_price
        .map(|price| price * 1.05)
        .map(|tp| tp * 0.5);

    println!("No result: {:?}", no_result);  // None
}
```

## The and_then Method

`and_then` (also known as flatMap in other languages) applies a function that itself returns an `Option`:

```rust
fn main() {
    let balance: Option<f64> = Some(10000.0);

    // Function that returns Option
    fn calculate_position(balance: f64) -> Option<f64> {
        if balance >= 1000.0 {
            Some(balance * 0.02)  // 2% of balance
        } else {
            None  // Insufficient funds
        }
    }

    // and_then "unwraps" the nested Option
    let position = balance.and_then(calculate_position);
    println!("Position: {:?}", position);  // Some(200.0)

    // With small balance
    let small_balance: Option<f64> = Some(500.0);
    let no_position = small_balance.and_then(calculate_position);
    println!("No position: {:?}", no_position);  // None
}
```

### Difference Between map and and_then

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Function returning Option
    fn get_stop_loss(price: f64) -> Option<f64> {
        if price > 0.0 {
            Some(price * 0.95)  // -5%
        } else {
            None
        }
    }

    // map creates nested Option<Option<f64>>
    let nested = price.map(get_stop_loss);
    println!("With map: {:?}", nested);  // Some(Some(39900.0))

    // and_then "flattens" the result
    let flat = price.and_then(get_stop_loss);
    println!("With and_then: {:?}", flat);  // Some(39900.0)
}
```

## Combining map and and_then

```rust
fn main() {
    let ticker: Option<&str> = Some("BTCUSDT");

    // Simulate getting price by ticker
    fn get_price(ticker: &str) -> Option<f64> {
        match ticker {
            "BTCUSDT" => Some(42000.0),
            "ETHUSDT" => Some(2500.0),
            _ => None,
        }
    }

    // Simulate liquidity check
    fn check_liquidity(price: f64) -> Option<f64> {
        if price > 1000.0 {
            Some(price)
        } else {
            None  // Insufficient liquidity
        }
    }

    let result = ticker
        .and_then(get_price)           // Option<f64>
        .and_then(check_liquidity)     // Option<f64>
        .map(|price| price * 0.02)     // Position size
        .map(|pos| format!("Position: ${:.2}", pos));

    println!("{:?}", result);  // Some("Position: $840.00")
}
```

## Practical Example: Order Analysis

```rust
fn main() {
    // Order structure
    struct Order {
        symbol: String,
        quantity: Option<f64>,
        price: Option<f64>,
    }

    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: Some(0.5),
        price: Some(42000.0),
    };

    // Calculate order value
    let order_value = order.quantity
        .and_then(|qty| {
            order.price.map(|price| qty * price)
        });

    println!("Order value: {:?}", order_value);  // Some(21000.0)

    // More elegant way with zip (Rust 1.46+)
    let order2 = Order {
        symbol: "ETHUSDT".to_string(),
        quantity: Some(10.0),
        price: Some(2500.0),
    };

    let value2 = order2.quantity
        .zip(order2.price)
        .map(|(qty, price)| qty * price);

    println!("Value: {:?}", value2);  // Some(25000.0)
}
```

## Portfolio Processing

```rust
fn main() {
    struct Position {
        symbol: String,
        quantity: f64,
        entry_price: Option<f64>,
    }

    fn get_current_price(symbol: &str) -> Option<f64> {
        match symbol {
            "BTC" => Some(43000.0),
            "ETH" => Some(2600.0),
            "SOL" => Some(100.0),
            _ => None,
        }
    }

    fn calculate_pnl(position: &Position) -> Option<f64> {
        position.entry_price.and_then(|entry| {
            get_current_price(&position.symbol).map(|current| {
                (current - entry) * position.quantity
            })
        })
    }

    let positions = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: Some(42000.0) },
        Position { symbol: "ETH".to_string(), quantity: 5.0, entry_price: Some(2500.0) },
        Position { symbol: "UNKNOWN".to_string(), quantity: 100.0, entry_price: Some(10.0) },
    ];

    println!("╔═══════════════════════════════════════╗");
    println!("║          PORTFOLIO P&L                ║");
    println!("╚═══════════════════════════════════════╝\n");

    for pos in &positions {
        let pnl = calculate_pnl(pos);
        match pnl {
            Some(value) => {
                let sign = if value >= 0.0 { "+" } else { "" };
                println!("{}: {}${:.2}", pos.symbol, sign, value);
            }
            None => println!("{}: No data", pos.symbol),
        }
    }
}
```

## filter Combined with map

```rust
fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(42000.0),
        None,
        Some(2500.0),
        Some(50.0),
        None,
        Some(100.0),
    ];

    // Filter and transform prices
    let high_value_positions: Vec<f64> = prices
        .into_iter()
        .filter_map(|price| {
            price
                .filter(|&p| p >= 100.0)  // Only prices >= 100
                .map(|p| p * 0.01)         // 1% position
        })
        .collect();

    println!("Positions: {:?}", high_value_positions);
    // [420.0, 25.0, 1.0]
}
```

## ok_or and ok_or_else Methods

Converting `Option` to `Result`:

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);
    let no_price: Option<f64> = None;

    // ok_or converts Option to Result
    let result1 = price.ok_or("Price unavailable");
    let result2 = no_price.ok_or("Price unavailable");

    println!("Result 1: {:?}", result1);  // Ok(42000.0)
    println!("Result 2: {:?}", result2);  // Err("Price unavailable")

    // ok_or_else with lazy error computation
    fn create_error() -> String {
        println!("Creating error message...");
        "API timeout".to_string()
    }

    let lazy_result = no_price.ok_or_else(create_error);
    println!("Lazy: {:?}", lazy_result);
}
```

## Practical Example: Trade Validator

```rust
fn main() {
    struct TradeRequest {
        symbol: Option<String>,
        side: Option<String>,
        quantity: Option<f64>,
        price: Option<f64>,
    }

    fn validate_symbol(symbol: &str) -> Option<String> {
        let valid_symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
        if valid_symbols.contains(&symbol) {
            Some(symbol.to_string())
        } else {
            None
        }
    }

    fn validate_quantity(qty: f64) -> Option<f64> {
        if qty > 0.0 && qty <= 1000.0 {
            Some(qty)
        } else {
            None
        }
    }

    fn validate_price(price: f64) -> Option<f64> {
        if price > 0.0 {
            Some(price)
        } else {
            None
        }
    }

    let request = TradeRequest {
        symbol: Some("BTCUSDT".to_string()),
        side: Some("BUY".to_string()),
        quantity: Some(0.5),
        price: Some(42000.0),
    };

    // Validation through and_then chain
    let validated_order = request.symbol
        .as_ref()
        .and_then(|s| validate_symbol(s))
        .and_then(|symbol| {
            request.quantity
                .and_then(validate_quantity)
                .and_then(|qty| {
                    request.price
                        .and_then(validate_price)
                        .map(|price| {
                            format!(
                                "Order: {} {} @ ${:.2} (value: ${:.2})",
                                symbol, qty, price, qty * price
                            )
                        })
                })
        });

    match validated_order {
        Some(order) => println!("✓ Valid: {}", order),
        None => println!("✗ Invalid order"),
    }
}
```

## What We Learned

| Method | Description | Returns |
|--------|-------------|---------|
| `map(f)` | Transforms value inside Some | `Option<U>` |
| `and_then(f)` | Applies function returning Option | `Option<U>` |
| `filter(predicate)` | Keeps Some only if predicate is true | `Option<T>` |
| `ok_or(err)` | Converts to Result | `Result<T, E>` |
| `ok_or_else(f)` | Lazy conversion to Result | `Result<T, E>` |
| `zip(other)` | Combines two Options into tuple | `Option<(T, U)>` |

## Practical Exercises

### Exercise 1: Risk Calculator

Implement a function that:
- Takes `Option<f64>` for balance
- Takes `Option<f64>` for risk percentage
- Returns `Option<f64>` with risk amount

```rust
fn calculate_risk_amount(
    balance: Option<f64>,
    risk_percent: Option<f64>
) -> Option<f64> {
    // Your code here
    todo!()
}
```

### Exercise 2: Analysis Chain

Create a function to analyze entry possibility:
1. Get price (may be None)
2. Check that price > 0
3. Calculate position size (2% of 10000 balance)
4. Check that size >= 100
5. Return final value

### Exercise 3: Order List Processing

```rust
struct Order {
    id: u64,
    price: Option<f64>,
    quantity: Option<f64>,
}

fn calculate_total_value(orders: Vec<Order>) -> f64 {
    // Sum the value of only valid orders
    // (where both price and quantity are Some)
    todo!()
}
```

### Exercise 4: Currency Converter

Create a function that:
- Takes amount in USD (Option<f64>)
- Gets conversion rate (may return None)
- Applies fee (may be None = no fee)
- Returns final amount

## Homework

1. **Signal Analyzer**: Create a system that receives a trading signal (Option), validates it, calculates entry parameters, and returns a ready order.

2. **Portfolio Calculator**: Implement a function that calculates total P&L for a list of positions, skipping positions with missing data.

3. **API Wrapper**: Write wrapper functions for exchange API where each call may return None on error, using and_then chains.

4. **Strategy Validator**: Create a trading strategy validator that checks presence and correctness of all parameters (entry, stop_loss, take_profit, position_size).

## Navigation

[← Previous day](../073-option-unwrap-expect/en.md) | [Next day →](../075-result-type/en.md)
