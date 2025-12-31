# Day 20: while — Waiting for Target Price

## Trading Analogy

Imagine you've placed a limit buy order for Bitcoin at $40,000. The market is currently at $42,000, and you're **waiting** for the price to drop to your target. You won't check the price forever — only **while** it's above your target.

The `while` loop works exactly the same way: perform an action **while** a condition is true.

## Basic while Syntax

```rust
fn main() {
    let target_price = 40000.0;
    let mut current_price = 42000.0;

    // Simulate price dropping
    while current_price > target_price {
        println!("Current price: ${:.2} — waiting for drop...", current_price);
        current_price -= 500.0; // Price drops by $500
    }

    println!("Price reached ${:.2} — BUY!", current_price);
}
```

**Important:** The condition is checked **before** each iteration. If the condition is initially false — the loop won't execute at all.

## while vs loop

```rust
fn main() {
    // loop — infinite loop, requires break
    // while — checks condition, terminates automatically

    let mut price = 100.0;
    let take_profit = 110.0;

    // while: check condition on entry
    while price < take_profit {
        price += 2.0;
        println!("Price rising: ${:.2}", price);
    }

    println!("Take profit reached!");
}
```

## Waiting for Trading Signal

```rust
fn main() {
    let prices = [42000.0, 41800.0, 41500.0, 41200.0, 40800.0, 40500.0, 40000.0];
    let mut index = 0;
    let buy_signal_price = 41000.0;

    println!("Waiting for buy signal (price < ${:.2})", buy_signal_price);

    while index < prices.len() && prices[index] >= buy_signal_price {
        println!("  Price ${:.2} — no signal", prices[index]);
        index += 1;
    }

    if index < prices.len() {
        println!("SIGNAL! Price ${:.2} — opening position", prices[index]);
    } else {
        println!("No signal received during observation period");
    }
}
```

## Position Accumulation (DCA)

Dollar Cost Averaging — buying an asset in portions:

```rust
fn main() {
    let total_budget = 10000.0;   // Total budget $10,000
    let order_size = 2000.0;      // Size of each purchase
    let mut spent = 0.0;
    let mut btc_accumulated = 0.0;
    let mut order_number = 1;

    // Simulate prices at each purchase
    let prices = [42000.0, 41500.0, 40000.0, 39500.0, 41000.0];
    let mut price_index = 0;

    while spent < total_budget && price_index < prices.len() {
        let price = prices[price_index];
        let btc_bought = order_size / price;

        println!(
            "Purchase #{}: ${:.2} at ${:.2} = {:.6} BTC",
            order_number, order_size, price, btc_bought
        );

        btc_accumulated += btc_bought;
        spent += order_size;
        order_number += 1;
        price_index += 1;
    }

    let avg_price = spent / btc_accumulated;
    println!("\n=== TOTAL ===");
    println!("Spent: ${:.2}", spent);
    println!("Accumulated: {:.6} BTC", btc_accumulated);
    println!("Average price: ${:.2}", avg_price);
}
```

## Stop-Loss Monitoring

```rust
fn main() {
    let entry_price = 42000.0;
    let stop_loss = 40000.0;     // -4.76% from entry
    let mut current_price = entry_price;

    // Simulate price movement
    let price_changes = [-100.0, -200.0, 50.0, -500.0, -300.0, -400.0, -600.0];
    let mut change_index = 0;

    println!("Position opened at ${:.2}", entry_price);
    println!("Stop-loss: ${:.2}\n", stop_loss);

    while current_price > stop_loss && change_index < price_changes.len() {
        current_price += price_changes[change_index];
        let pnl_percent = ((current_price - entry_price) / entry_price) * 100.0;

        println!(
            "Price: ${:.2} | PnL: {:+.2}%",
            current_price, pnl_percent
        );

        change_index += 1;
    }

    if current_price <= stop_loss {
        let loss = entry_price - current_price;
        let loss_percent = (loss / entry_price) * 100.0;
        println!("\nSTOP-LOSS TRIGGERED!");
        println!("Loss: ${:.2} ({:.2}%)", loss, loss_percent);
    } else {
        println!("\nPosition still open");
    }
}
```

## Simple Moving Average (SMA) Calculation

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42300.0, 42500.0,
                  42400.0, 42600.0, 42800.0, 42700.0, 43000.0];
    let period = 5;

    println!("Calculating SMA-{} for {} prices\n", period, prices.len());

    let mut i = period - 1;

    while i < prices.len() {
        // Sum the last `period` prices
        let mut sum = 0.0;
        let mut j = 0;

        while j < period {
            sum += prices[i - j];
            j += 1;
        }

        let sma = sum / period as f64;
        println!("Index {}: Price ${:.2} | SMA-{}: ${:.2}", i, prices[i], period, sma);

        i += 1;
    }
}
```

## Waiting for Trend Confirmation

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42200.0, 42150.0, 42300.0, 42500.0, 42700.0];
    let confirmations_needed = 3;
    let mut consecutive_up = 0;
    let mut i = 1;

    println!("Waiting for {} consecutive up candles...\n", confirmations_needed);

    while i < prices.len() && consecutive_up < confirmations_needed {
        if prices[i] > prices[i - 1] {
            consecutive_up += 1;
            println!(
                "Candle {}: ${:.2} > ${:.2} — up #{}/{}",
                i, prices[i], prices[i - 1], consecutive_up, confirmations_needed
            );
        } else {
            consecutive_up = 0;
            println!(
                "Candle {}: ${:.2} <= ${:.2} — counter reset",
                i, prices[i], prices[i - 1]
            );
        }
        i += 1;
    }

    if consecutive_up >= confirmations_needed {
        println!("\nTREND CONFIRMED! Opening LONG position.");
    } else {
        println!("\nTrend not confirmed.");
    }
}
```

## Portfolio Management with Rebalancing

```rust
fn main() {
    let target_btc_percent = 50.0;
    let tolerance = 5.0; // Acceptable deviation ±5%

    let mut btc_value = 6000.0;  // $6,000 in BTC
    let mut usd_value = 4000.0;  // $4,000 in USD

    // Simulate BTC price changes
    let price_multipliers = [1.1, 1.15, 0.95, 1.2, 0.9];
    let mut step = 0;

    while step < price_multipliers.len() {
        btc_value *= price_multipliers[step];
        let total = btc_value + usd_value;
        let btc_percent = (btc_value / total) * 100.0;

        println!("\n=== Step {} ===", step + 1);
        println!("BTC: ${:.2} ({:.1}%)", btc_value, btc_percent);
        println!("USD: ${:.2} ({:.1}%)", usd_value, 100.0 - btc_percent);

        // Check if rebalancing is needed
        let deviation = (btc_percent - target_btc_percent).abs();

        if deviation > tolerance {
            println!("Deviation {:.1}% > {:.1}% — rebalancing needed!", deviation, tolerance);

            let target_btc_value = total * (target_btc_percent / 100.0);
            let adjustment = btc_value - target_btc_value;

            if adjustment > 0.0 {
                println!("Selling ${:.2} BTC", adjustment);
                btc_value -= adjustment;
                usd_value += adjustment;
            } else {
                println!("Buying ${:.2} BTC", -adjustment);
                btc_value -= adjustment; // adjustment is negative
                usd_value += adjustment;
            }
        } else {
            println!("Deviation {:.1}% — no rebalancing needed", deviation);
        }

        step += 1;
    }
}
```

## Volume Analysis

```rust
fn main() {
    let volumes = [1500.0, 2000.0, 2500.0, 1800.0, 3500.0, 4000.0, 3800.0];
    let volume_threshold = 3000.0;
    let mut high_volume_count = 0;
    let mut i = 0;

    println!("Volume analysis (threshold: ${:.0})\n", volume_threshold);

    while i < volumes.len() {
        let volume = volumes[i];
        let status = if volume >= volume_threshold {
            high_volume_count += 1;
            "HIGH"
        } else {
            "normal"
        };

        println!("Candle {}: volume ${:.0} — {}", i + 1, volume, status);
        i += 1;
    }

    let high_volume_percent = (high_volume_count as f64 / volumes.len() as f64) * 100.0;
    println!("\nHigh volume candles: {} of {} ({:.1}%)",
             high_volume_count, volumes.len(), high_volume_percent);
}
```

## Practical Example: Trading Session Simulation

```rust
fn main() {
    let mut balance = 10000.0;
    let position_size = 0.1; // 0.1 BTC
    let mut btc_holdings = 0.0;
    let mut in_position = false;

    // Simulate prices throughout the day
    let prices = [42000.0, 41800.0, 41500.0, 41200.0, 41500.0,
                  42000.0, 42500.0, 43000.0, 42800.0, 42500.0];

    let buy_threshold = 41300.0;
    let sell_threshold = 42800.0;

    let mut i = 0;

    println!("=== TRADING SESSION ===\n");
    println!("Initial balance: ${:.2}", balance);
    println!("Position size: {} BTC", position_size);
    println!("Buy below ${:.0}, sell above ${:.0}\n", buy_threshold, sell_threshold);

    while i < prices.len() {
        let price = prices[i];
        println!("Hour {}: Price ${:.2}", i + 1, price);

        // Buy logic
        if !in_position && price < buy_threshold {
            let cost = price * position_size;
            if balance >= cost {
                balance -= cost;
                btc_holdings += position_size;
                in_position = true;
                println!("  -> BUY {} BTC at ${:.2} (spent ${:.2})", position_size, price, cost);
            }
        }

        // Sell logic
        if in_position && price > sell_threshold {
            let revenue = price * btc_holdings;
            balance += revenue;
            println!("  -> SELL {} BTC at ${:.2} (received ${:.2})", btc_holdings, price, revenue);
            btc_holdings = 0.0;
            in_position = false;
        }

        i += 1;
    }

    // Results
    println!("\n=== SESSION RESULTS ===");
    println!("USD Balance: ${:.2}", balance);
    println!("BTC Holdings: {:.4}", btc_holdings);

    if btc_holdings > 0.0 {
        let btc_value = btc_holdings * prices[prices.len() - 1];
        println!("BTC Value: ${:.2}", btc_value);
        println!("Total Portfolio: ${:.2}", balance + btc_value);
    }

    let profit = balance - 10000.0 + (btc_holdings * prices[prices.len() - 1]);
    println!("Profit: ${:+.2}", profit);
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| `while condition {}` | Loop while condition is true | Waiting for target price |
| Condition checked first | Loop may not execute | Checking trade possibility |
| `while` with index | Iterating through array | Historical data analysis |
| Multiple conditions | `while a && b` | Stop-loss + take-profit |
| State modification | `mut` variables in loop | Position accumulation |

## Homework

1. **Entry Waiting:** Write a program that waits until RSI drops below 30 (oversold), using an array of RSI values.

2. **Trailing Stop:** Implement a trailing stop-loss that follows the price at a 2% distance and triggers on reversal.

3. **Volume Accumulation:** Create a simulation where the bot buys $1000 each time the price drops 5% from the previous purchase, until the entire budget is spent.

4. **Consolidation Analysis:** Write a program that identifies a consolidation period — when the price stays within ±2% range for N consecutive candles.

## Navigation

[<- Previous day](../019-loop/en.md) | [Next day ->](../021-for/en.md)
