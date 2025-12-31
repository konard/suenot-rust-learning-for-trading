# Day 25: match — Determining Order Type

## Trading Analogy

When an order arrives at a trading system, the first thing to do is determine its type: **Market**, **Limit**, **Stop**, or **Stop-Limit**. Depending on the order type, the system performs different actions. It's like a sorter at an exchange — it looks at the order and routes it to the appropriate handler.

In Rust, we use the `match` expression for such "sorting" — a powerful pattern matching tool.

## Basic match Syntax

```rust
fn main() {
    let order_type = "limit";

    match order_type {
        "market" => println!("Execute immediately at market price"),
        "limit" => println!("Place in order book and wait"),
        "stop" => println!("Activate when price is reached"),
        _ => println!("Unknown order type"),
    }
}
```

**Important:** `_` is a "wildcard" that catches all remaining variants. Rust requires match to be *exhaustive* — covering all possible cases.

## match Returns a Value

```rust
fn main() {
    let side = "buy";

    let direction = match side {
        "buy" => 1,
        "sell" => -1,
        _ => 0,
    };

    println!("Direction: {}", direction);

    // Using it for PnL calculation
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;

    let pnl = (exit - entry) * quantity * direction as f64;
    println!("PnL: ${:.2}", pnl);
}
```

## match with Numbers

```rust
fn main() {
    let rsi = 75;

    let signal = match rsi {
        0..=30 => "Oversold — buy signal",
        31..=70 => "Neutral zone — hold",
        71..=100 => "Overbought — sell signal",
        _ => "Invalid RSI value",
    };

    println!("RSI = {}: {}", rsi, signal);
}
```

## match with enum — Perfect Pair

```rust
enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

fn main() {
    let order = OrderType::Limit;

    let description = match order {
        OrderType::Market => "Executes immediately at best price",
        OrderType::Limit => "Executes at specified price or better",
        OrderType::Stop => "Becomes market order when stop price is reached",
        OrderType::StopLimit => "Becomes limit order when stop price is reached",
    };

    println!("{}", description);
}
```

**Advantage:** When using enum, the compiler checks that you've handled ALL variants. If you add a new order type, the code won't compile until you handle it.

## match with Data Inside enum

```rust
enum OrderCommand {
    Buy { ticker: String, quantity: f64 },
    Sell { ticker: String, quantity: f64 },
    Cancel { order_id: u64 },
    ModifyPrice { order_id: u64, new_price: f64 },
}

fn main() {
    let command = OrderCommand::Buy {
        ticker: String::from("BTC"),
        quantity: 0.5,
    };

    match command {
        OrderCommand::Buy { ticker, quantity } => {
            println!("Buying {} {} units", ticker, quantity);
        }
        OrderCommand::Sell { ticker, quantity } => {
            println!("Selling {} {} units", ticker, quantity);
        }
        OrderCommand::Cancel { order_id } => {
            println!("Cancelling order #{}", order_id);
        }
        OrderCommand::ModifyPrice { order_id, new_price } => {
            println!("Changing price of order #{} to ${:.2}", order_id, new_price);
        }
    }
}
```

## Value Binding with @

```rust
fn main() {
    let price_change_percent = 7.5;

    let alert = match price_change_percent {
        x @ 0.0..=2.0 => format!("Normal movement: {:.1}%", x),
        x @ 2.0..=5.0 => format!("Elevated volatility: {:.1}%", x),
        x @ 5.0..=10.0 => format!("WARNING! Strong movement: {:.1}%", x),
        x if x > 10.0 => format!("CRITICAL! Extreme movement: {:.1}%", x),
        x => format!("Decline: {:.1}%", x),
    };

    println!("{}", alert);
}
```

## Guards — Additional Conditions

```rust
fn main() {
    let price = 42500.0;
    let volume = 1000000.0;

    let market_condition = match (price, volume) {
        (p, v) if p > 50000.0 && v > 500000.0 => "Bull market with high volume",
        (p, v) if p > 50000.0 && v <= 500000.0 => "Bull market with low volume",
        (p, v) if p <= 50000.0 && v > 500000.0 => "Bear market with high volume",
        (p, v) if p <= 50000.0 && v <= 500000.0 => "Bear market with low volume",
        _ => "Undefined state",
    };

    println!("Market condition: {}", market_condition);
}
```

## match with Option

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0];
    let target_price = 42500.0;

    let found = prices.iter().find(|&&p| p == target_price);

    match found {
        Some(price) => println!("Found price: ${:.2}", price),
        None => println!("Price not found"),
    }
}

fn get_stop_loss(entry: f64, risk_percent: f64) -> Option<f64> {
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return None;
    }
    Some(entry * (1.0 - risk_percent / 100.0))
}
```

## match with Result

```rust
fn main() {
    let order_result = execute_order("BTC", 0.5, 42000.0);

    match order_result {
        Ok(order_id) => println!("Order executed! ID: {}", order_id),
        Err(error) => println!("Error: {}", error),
    }
}

fn execute_order(ticker: &str, quantity: f64, price: f64) -> Result<u64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    // Simulate successful execution
    Ok(12345)
}
```

## Practical Example: Trade Classification

```rust
enum TradeResult {
    Profit(f64),
    Loss(f64),
    Breakeven,
}

fn main() {
    let trades = [
        calculate_trade_result(42000.0, 43500.0, 0.5),
        calculate_trade_result(42000.0, 41000.0, 0.3),
        calculate_trade_result(42000.0, 42000.0, 1.0),
        calculate_trade_result(50000.0, 52000.0, 0.2),
    ];

    let mut total_profit = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for trade in &trades {
        match trade {
            TradeResult::Profit(amount) => {
                println!("Profit: ${:.2}", amount);
                total_profit += amount;
                win_count += 1;
            }
            TradeResult::Loss(amount) => {
                println!("Loss: ${:.2}", amount);
                total_loss += amount;
                loss_count += 1;
            }
            TradeResult::Breakeven => {
                println!("Breakeven");
            }
        }
    }

    println!("\n=== Statistics ===");
    println!("Total profit: ${:.2}", total_profit);
    println!("Total loss: ${:.2}", total_loss);
    println!("Net result: ${:.2}", total_profit - total_loss);
    println!("Win rate: {:.1}%", (win_count as f64 / trades.len() as f64) * 100.0);
}

fn calculate_trade_result(entry: f64, exit: f64, quantity: f64) -> TradeResult {
    let pnl = (exit - entry) * quantity;

    match pnl {
        p if p > 0.0 => TradeResult::Profit(p),
        p if p < 0.0 => TradeResult::Loss(p.abs()),
        _ => TradeResult::Breakeven,
    }
}
```

## Example: Trading Signal

```rust
enum TrendDirection {
    Up,
    Down,
    Sideways,
}

enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

fn main() {
    let trend = TrendDirection::Up;
    let rsi = 35;
    let price_above_sma = true;

    let signal = generate_signal(&trend, rsi, price_above_sma);

    let action = match signal {
        Signal::StrongBuy => "Open long position (full size)",
        Signal::Buy => "Open long position (half size)",
        Signal::Hold => "Wait for better entry point",
        Signal::Sell => "Close part of position",
        Signal::StrongSell => "Close entire position",
    };

    println!("Action: {}", action);
}

fn generate_signal(trend: &TrendDirection, rsi: i32, price_above_sma: bool) -> Signal {
    match (trend, rsi, price_above_sma) {
        (TrendDirection::Up, r, true) if r < 30 => Signal::StrongBuy,
        (TrendDirection::Up, r, true) if r < 50 => Signal::Buy,
        (TrendDirection::Up, _, false) => Signal::Hold,
        (TrendDirection::Down, r, false) if r > 70 => Signal::StrongSell,
        (TrendDirection::Down, r, false) if r > 50 => Signal::Sell,
        (TrendDirection::Down, _, true) => Signal::Hold,
        (TrendDirection::Sideways, _, _) => Signal::Hold,
        _ => Signal::Hold,
    }
}
```

## if let — Simplified match for One Variant

```rust
fn main() {
    let order_status = Some("filled");

    // Instead of full match:
    // match order_status {
    //     Some(status) => println!("Status: {}", status),
    //     None => {},
    // }

    // Use if let:
    if let Some(status) = order_status {
        println!("Order status: {}", status);
    }

    // With else
    let price: Option<f64> = None;

    if let Some(p) = price {
        println!("Price: ${:.2}", p);
    } else {
        println!("Price unavailable");
    }
}
```

## let else — Early Exit

```rust
fn process_trade(trade_data: Option<(f64, f64, f64)>) {
    let Some((entry, exit, qty)) = trade_data else {
        println!("No trade data available");
        return;
    };

    let pnl = (exit - entry) * qty;
    println!("PnL: ${:.2}", pnl);
}

fn main() {
    process_trade(Some((42000.0, 43500.0, 0.5)));
    process_trade(None);
}
```

## What We Learned

| Construct | Usage |
|-----------|-------|
| `match value { ... }` | Full pattern matching |
| `_` | Wildcard — catches all remaining variants |
| `x @ pattern` | Bind value to variable |
| `if guard` | Additional condition in branch |
| `if let` | Simplified match for one variant |
| `let else` | Early exit if no match |

## Homework

1. Create an enum `OrderStatus` with variants: `Pending`, `PartiallyFilled(f64)`, `Filled`, `Cancelled`, `Rejected(String)`. Write a function that uses match to print status information.

2. Implement a function `classify_market_move(change_percent: f64) -> &'static str` that classifies price change:
   - < -5%: "Crash"
   - -5% to -2%: "Decline"
   - -2% to 2%: "Stable"
   - 2% to 5%: "Rally"
   - > 5%: "Moon"

3. Write a function `get_position_action(current: f64, target: f64, stop: f64, price: f64) -> String` that determines action:
   - If price >= target: "Take Profit"
   - If price <= stop: "Stop Loss"
   - If price > current: "In Profit"
   - If price < current: "In Loss"
   - Otherwise: "At Entry"

4. Create a commission determination system based on trading volume (match with ranges):
   - Up to $10,000: 0.1%
   - $10,000 - $50,000: 0.08%
   - $50,000 - $100,000: 0.06%
   - More than $100,000: 0.04%

## Navigation

[← Previous day](../024-continue-skip-losing-trades/en.md) | [Next day →](../026-constants-fixed-exchange-fee/en.md)
