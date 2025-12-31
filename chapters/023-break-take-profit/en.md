# Day 23: break — Exit at Take Profit

## Trading Analogy

Imagine you're monitoring an asset's price in a loop, waiting for the moment to close your position. When the price reaches your **Take Profit** level — you immediately exit the trade, without waiting for the trading session to end. This is exactly how `break` works in Rust: it allows you to **immediately exit a loop** when a condition is met.

## Basic break Syntax

```rust
fn main() {
    let prices = [42000.0, 42500.0, 43000.0, 43500.0, 44000.0];
    let take_profit = 43200.0;

    for price in prices {
        println!("Current price: ${:.2}", price);

        if price >= take_profit {
            println!("🎯 Take Profit reached! Closing position.");
            break;  // Exit the loop
        }
    }

    println!("Monitoring complete");
}
```

**Output:**
```
Current price: $42000.00
Current price: $42500.00
Current price: $43000.00
Current price: $43500.00
🎯 Take Profit reached! Closing position.
Monitoring complete
```

## break in loop

```rust
fn main() {
    let mut price = 42000.0;
    let take_profit = 43500.0;
    let stop_loss = 41000.0;

    loop {
        // Simulate price change
        price += (price * 0.005) * if price > 42500.0 { 1.0 } else { -0.5 };

        println!("Price: ${:.2}", price);

        if price >= take_profit {
            println!("✅ Take Profit! Profit locked in.");
            break;
        }

        if price <= stop_loss {
            println!("❌ Stop Loss! Loss limited.");
            break;
        }
    }
}
```

## break with Return Value

In Rust, `loop` can return a value through `break`:

```rust
fn main() {
    let prices = [41500.0, 42000.0, 42800.0, 43200.0, 43000.0];
    let entry_price = 42000.0;

    let exit_price = loop {
        // In reality, this would be a data stream
        static mut INDEX: usize = 0;

        let price = unsafe {
            let p = prices.get(INDEX).copied();
            INDEX += 1;
            p
        };

        match price {
            Some(p) if p >= entry_price * 1.03 => {
                println!("Take Profit at ${:.2}", p);
                break p;  // Return the exit price
            }
            Some(p) if p <= entry_price * 0.98 => {
                println!("Stop Loss at ${:.2}", p);
                break p;
            }
            Some(p) => println!("Waiting... price ${:.2}", p),
            None => {
                println!("Data exhausted");
                break entry_price;  // Exit at entry price
            }
        }
    };

    let pnl = exit_price - entry_price;
    println!("PnL: ${:.2}", pnl);
}
```

## Safer Example with Return Value

```rust
fn main() {
    let orders = [
        ("BTC", 100.0),
        ("ETH", 50.0),
        ("INVALID", -10.0),  // Invalid order
        ("SOL", 25.0),
    ];

    let result = loop {
        let mut found_invalid = false;
        let mut invalid_ticker = "";

        for (ticker, amount) in &orders {
            if *amount < 0.0 {
                found_invalid = true;
                invalid_ticker = ticker;
                break;  // Inner break
            }
        }

        if found_invalid {
            break format!("Error: invalid order {}", invalid_ticker);
        } else {
            break String::from("All orders are valid");
        }
    };

    println!("{}", result);
}
```

## Finding the First Profitable Signal

```rust
fn main() {
    let signals = [
        ("BTC", -2.5),   // Loss
        ("ETH", -1.2),   // Loss
        ("SOL", 3.8),    // Profitable!
        ("ADA", 1.5),    // Profitable
    ];

    let mut found_signal: Option<(&str, f64)> = None;

    for (ticker, profit_pct) in signals {
        if profit_pct > 0.0 {
            found_signal = Some((ticker, profit_pct));
            break;  // Found it — exit
        }
        println!("Skipping {}: loss {:.1}%", ticker, profit_pct);
    }

    match found_signal {
        Some((ticker, pct)) => {
            println!("First profitable signal: {} (+{:.1}%)", ticker, pct);
        }
        None => println!("No profitable signals found"),
    }
}
```

## Monitoring Until Goal Reached

```rust
fn main() {
    let target_balance = 11000.0;
    let mut balance = 10000.0;
    let mut trades = 0;

    let trade_results = [150.0, -50.0, 200.0, -30.0, 300.0, 100.0, -80.0, 500.0];

    for pnl in trade_results {
        trades += 1;
        balance += pnl;

        println!("Trade #{}: PnL ${:.2}, Balance: ${:.2}", trades, pnl, balance);

        if balance >= target_balance {
            println!("🎉 Goal reached in {} trades!", trades);
            break;
        }
    }

    if balance < target_balance {
        println!("Goal not reached. Final balance: ${:.2}", balance);
    }
}
```

## Order Validation with Early Exit

```rust
fn main() {
    let order_price = 42000.0;
    let order_quantity = 0.5;
    let balance = 20000.0;

    let validation_passed = validate_order(order_price, order_quantity, balance);

    if validation_passed {
        println!("✅ Order accepted for execution");
    } else {
        println!("❌ Order rejected");
    }
}

fn validate_order(price: f64, quantity: f64, balance: f64) -> bool {
    let checks = [
        (price > 0.0, "Price must be positive"),
        (quantity > 0.0, "Quantity must be positive"),
        (price * quantity <= balance, "Insufficient funds"),
        (quantity >= 0.001, "Minimum order size is 0.001"),
    ];

    for (condition, error_msg) in checks {
        if !condition {
            println!("Validation error: {}", error_msg);
            return false;  // Early exit from function (not break!)
        }
    }

    true
}
```

## break vs return

```rust
fn main() {
    let result = process_trades();
    println!("Result: {}", result);
}

fn process_trades() -> String {
    let trades = [100.0, -50.0, 200.0, -500.0, 150.0];
    let mut total_pnl = 0.0;

    for pnl in trades {
        // return - exits the function
        if pnl < -400.0 {
            return String::from("Critical loss! Trading stopped.");
        }

        total_pnl += pnl;

        // break - exits only the loop
        if total_pnl >= 200.0 {
            println!("Daily goal reached");
            break;
        }
    }

    format!("Total PnL: ${:.2}", total_pnl)
}
```

## Finding Optimal Entry Point

```rust
fn main() {
    let candles = [
        (42000.0, 42500.0, 41800.0, 42300.0),  // (open, high, low, close)
        (42300.0, 42400.0, 41900.0, 42000.0),
        (42000.0, 42100.0, 41500.0, 41600.0),  // Strong drop
        (41600.0, 42000.0, 41400.0, 41900.0),
        (41900.0, 42200.0, 41800.0, 42100.0),
    ];

    let mut entry_candle: Option<usize> = None;

    for (i, &(open, high, low, close)) in candles.iter().enumerate() {
        let body_size = (close - open).abs();
        let lower_wick = open.min(close) - low;

        // Looking for a "hammer" — reversal candle
        if lower_wick > body_size * 2.0 && close > open {
            println!("Candle #{}: Found 'Hammer' pattern!", i + 1);
            entry_candle = Some(i);
            break;
        }

        println!("Candle #{}: No signal", i + 1);
    }

    match entry_candle {
        Some(i) => println!("Recommendation: enter after candle #{}", i + 1),
        None => println!("No entry signal found"),
    }
}
```

## Processing Order Stream

```rust
fn main() {
    let order_book = [
        ("BUY", 42000.0, 0.5),
        ("BUY", 41950.0, 1.0),
        ("SELL", 42100.0, 0.3),
        ("CANCEL", 0.0, 0.0),  // Stop signal
        ("BUY", 42050.0, 0.8),  // Won't be processed
    ];

    let mut total_buy_volume = 0.0;
    let mut total_sell_volume = 0.0;

    for (order_type, price, quantity) in order_book {
        if order_type == "CANCEL" {
            println!("Cancel signal received. Stopping processing.");
            break;
        }

        match order_type {
            "BUY" => {
                total_buy_volume += price * quantity;
                println!("BUY: {} @ ${:.2}", quantity, price);
            }
            "SELL" => {
                total_sell_volume += price * quantity;
                println!("SELL: {} @ ${:.2}", quantity, price);
            }
            _ => println!("Unknown order type: {}", order_type),
        }
    }

    println!("\nSummary:");
    println!("Buy volume: ${:.2}", total_buy_volume);
    println!("Sell volume: ${:.2}", total_sell_volume);
}
```

## Practice Exercises

### Exercise 1: Stop Loss Monitoring

```rust
fn main() {
    // Find the price at which stop loss triggers
    let prices = [42000.0, 41800.0, 41500.0, 41200.0, 40800.0, 41000.0];
    let stop_loss = 41000.0;

    // Your code here
    // Print the stop loss trigger price and candle index
}
```

### Exercise 2: Finding Trend Reversal

```rust
fn main() {
    // Find the first candle where direction changed
    let closes = [100.0, 102.0, 104.0, 103.0, 101.0, 99.0];

    // Your code here
    // Determine the reversal moment (when price started falling after rising)
}
```

### Exercise 3: Loss Limit

```rust
fn main() {
    // Stop when cumulative loss exceeds $500
    let trades = [-100.0, 50.0, -200.0, 100.0, -300.0, -150.0, 200.0];

    // Your code here
    // Print which trade we stopped at and cumulative loss
}
```

## What We Learned

| Concept | Description | Example |
|---------|-------------|---------|
| `break` | Immediate loop exit | `break;` |
| `break value` | Exit with return value | `break price;` |
| break vs return | break — from loop, return — from function | — |
| Conditional break | Exit when condition is met | `if price >= tp { break; }` |

## Homework

1. **Trailing Stop**: Write a function that monitors prices and exits when the price drops 2% from the maximum. Return the maximum price reached and the exit price.

2. **Pattern Search**: Implement a search for the "three white soldiers" pattern (three consecutive rising candles). Use `break` to exit when found.

3. **Risk Manager**: Create a function that processes trades and stops when:
   - Daily profit limit is reached ($1000)
   - Daily loss limit is reached ($500)
   - Maximum number of trades is exceeded (10)

   Return the stop reason and final PnL.

4. **Order Validator**: Write a validator for a series of orders that stops processing at the first invalid order and returns error information.

## Navigation

[← Previous day](../022-loop-infinite-market/en.md) | [Next day →](../024-continue-skip-trade/en.md)
