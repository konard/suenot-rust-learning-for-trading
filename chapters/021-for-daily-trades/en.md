# Day 21: for — Iterating Through Daily Trades

## Trading Analogy

Imagine at the end of the day you review **each trade** in your journal to calculate total profit. You take the first trade, note the PnL, then the second, third... and so on until the end of the list. The `for` loop in Rust works exactly the same way — it **sequentially iterates** through each element of a collection.

## Basic for Syntax

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    for price in prices {
        println!("Price: ${:.2}", price);
    }
}
```

## Iterating Over Price Array

```rust
fn main() {
    let daily_closes = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    let mut total = 0.0;
    for price in daily_closes {
        total += price;
    }

    let average = total / daily_closes.len() as f64;
    println!("Average closing price: ${:.2}", average);
}
```

## Iterating by Reference

When you need to keep ownership of the array:

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0];

    // Use reference to avoid taking ownership
    for price in &prices {
        println!("Price: ${:.2}", price);
    }

    // prices is still available
    println!("Total prices: {}", prices.len());
}
```

## Iterating with Indices (enumerate)

```rust
fn main() {
    let trades_pnl = [150.0, -75.0, 200.0, -30.0, 180.0];

    println!("Trade Journal:");
    println!("─────────────────────────");

    for (index, pnl) in trades_pnl.iter().enumerate() {
        let status = if *pnl >= 0.0 { "PROFIT" } else { "LOSS" };
        println!("Trade #{}: {:>8.2} [{}]", index + 1, pnl, status);
    }
}
```

## Iterating Over Ranges

```rust
fn main() {
    // Range from 1 to 5 (excluding 5)
    println!("Trading days of the week:");
    for day in 1..5 {
        println!("  Day {}", day);
    }

    // Range from 1 to 5 inclusive
    println!("\nAll 5 days:");
    for day in 1..=5 {
        println!("  Day {}", day);
    }
}
```

### Generating Timestamps

```rust
fn main() {
    let start_hour = 9;   // Market open
    let end_hour = 16;    // Market close

    println!("Trading hours:");
    for hour in start_hour..=end_hour {
        println!("  {:02}:00 - market open", hour);
    }
}
```

## Finding Maximum and Minimum

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 43100.0, 41500.0];

    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;

    for &price in &prices {
        if price < min_price {
            min_price = price;
        }
        if price > max_price {
            max_price = price;
        }
    }

    println!("Minimum price: ${:.2}", min_price);
    println!("Maximum price: ${:.2}", max_price);
    println!("Range: ${:.2}", max_price - min_price);
}
```

## Counting Winning and Losing Trades

```rust
fn main() {
    let trades = [150.0, -75.0, 200.0, -30.0, 180.0, -50.0, 90.0];

    let mut wins = 0;
    let mut losses = 0;
    let mut total_profit = 0.0;
    let mut total_loss = 0.0;

    for &pnl in &trades {
        if pnl >= 0.0 {
            wins += 1;
            total_profit += pnl;
        } else {
            losses += 1;
            total_loss += pnl.abs();
        }
    }

    let win_rate = (wins as f64 / trades.len() as f64) * 100.0;
    let profit_factor = if total_loss > 0.0 {
        total_profit / total_loss
    } else {
        f64::INFINITY
    };

    println!("╔═══════════════════════════════╗");
    println!("║       TRADE STATISTICS        ║");
    println!("╠═══════════════════════════════╣");
    println!("║ Winners:       {:>14} ║", wins);
    println!("║ Losers:        {:>14} ║", losses);
    println!("║ Win Rate:      {:>13.1}% ║", win_rate);
    println!("║ Profit Factor: {:>14.2} ║", profit_factor);
    println!("╚═══════════════════════════════╝");
}
```

## Calculating Simple Moving Average (SMA)

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0, 42300.0, 42250.0];
    let period = 3;

    println!("SMA-{} for daily prices:", period);
    println!("─────────────────────────────");

    for i in 0..prices.len() {
        if i + 1 >= period {
            let mut sum = 0.0;
            for j in (i + 1 - period)..=i {
                sum += prices[j];
            }
            let sma = sum / period as f64;
            println!("Day {}: price = ${:.2}, SMA = ${:.2}", i + 1, prices[i], sma);
        } else {
            println!("Day {}: price = ${:.2}, SMA = N/A (insufficient data)", i + 1, prices[i]);
        }
    }
}
```

## Filtering Trades

```rust
fn main() {
    let trades = [
        ("BTC", 150.0),
        ("ETH", -75.0),
        ("BTC", 200.0),
        ("SOL", -30.0),
        ("BTC", 180.0),
    ];

    println!("BTC trades only:");

    let mut btc_total = 0.0;
    for (symbol, pnl) in &trades {
        if *symbol == "BTC" {
            println!("  {} -> ${:.2}", symbol, pnl);
            btc_total += pnl;
        }
    }

    println!("BTC total: ${:.2}", btc_total);
}
```

## Processing Portfolio

```rust
fn main() {
    let portfolio = [
        ("BTC", 0.5, 42000.0),    // (ticker, quantity, price)
        ("ETH", 10.0, 2200.0),
        ("SOL", 100.0, 25.0),
        ("AVAX", 50.0, 35.0),
    ];

    println!("╔════════════════════════════════════════════╗");
    println!("║              MY PORTFOLIO                  ║");
    println!("╠════════════════════════════════════════════╣");
    println!("║ Ticker    Qty       Price          Value   ║");
    println!("╠════════════════════════════════════════════╣");

    let mut total_value = 0.0;

    for (ticker, quantity, price) in &portfolio {
        let value = quantity * price;
        total_value += value;
        println!("║ {:6} {:>7.2}  ${:>9.2}  ${:>12.2} ║",
                 ticker, quantity, price, value);
    }

    println!("╠════════════════════════════════════════════╣");
    println!("║ TOTAL                       ${:>12.2} ║", total_value);
    println!("╚════════════════════════════════════════════╝");
}
```

## Nested Loops: Analyzing by Days and Hours

```rust
fn main() {
    // Prices by days (rows) and hours (columns)
    let hourly_prices = [
        [42000.0, 42100.0, 42050.0, 42200.0],  // Day 1
        [42150.0, 42300.0, 42250.0, 42400.0],  // Day 2
        [42350.0, 42200.0, 42100.0, 42150.0],  // Day 3
    ];

    println!("Daily highs and lows:");
    println!("─────────────────────────────");

    for (day, prices) in hourly_prices.iter().enumerate() {
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for &price in prices {
            if price < min { min = price; }
            if price > max { max = price; }
        }

        println!("Day {}: min = ${:.2}, max = ${:.2}, range = ${:.2}",
                 day + 1, min, max, max - min);
    }
}
```

## Breaking the Loop with break

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41500.0, 42200.0, 42300.0];
    let stop_loss = 41800.0;

    println!("Price monitoring (stop-loss: ${:.2}):", stop_loss);

    for (i, &price) in prices.iter().enumerate() {
        println!("Hour {}: ${:.2}", i + 1, price);

        if price < stop_loss {
            println!("⚠️  STOP-LOSS TRIGGERED! Exiting position.");
            break;
        }
    }

    println!("Monitoring complete.");
}
```

## Skipping Iteration with continue

```rust
fn main() {
    let trades = [150.0, -75.0, 0.0, 200.0, -30.0, 0.0, 180.0];

    println!("Significant trades (skipping zero PnL):");

    let mut count = 0;
    for &pnl in &trades {
        if pnl == 0.0 {
            continue;  // Skip trades with zero PnL
        }

        count += 1;
        let sign = if pnl > 0.0 { "+" } else { "" };
        println!("  Trade: {}{:.2}", sign, pnl);
    }

    println!("Total significant trades: {}", count);
}
```

## Mutable Iteration (iter_mut)

```rust
fn main() {
    let mut prices = [42000.0, 42500.0, 41800.0, 42200.0];
    let commission_rate = 0.001;  // 0.1%

    println!("Prices before commission:  {:?}", prices);

    // Apply commission to each price
    for price in &mut prices {
        *price *= (1.0 - commission_rate);
    }

    println!("Prices after commission: {:?}", prices);
}
```

## Practical Example: Daily Report

```rust
fn main() {
    let daily_trades = [
        vec![100.0, -50.0, 75.0],           // Monday
        vec![-30.0, 120.0],                  // Tuesday
        vec![80.0, -40.0, 60.0, -20.0],     // Wednesday
        vec![150.0],                         // Thursday
        vec![-80.0, 90.0, -10.0, 50.0],     // Friday
    ];

    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];

    println!("╔═════════════════════════════════════════════╗");
    println!("║           WEEKLY TRADING REPORT             ║");
    println!("╠═════════════════════════════════════════════╣");

    let mut weekly_total = 0.0;
    let mut best_day = (0, f64::MIN);
    let mut worst_day = (0, f64::MAX);

    for (i, trades) in daily_trades.iter().enumerate() {
        let mut day_pnl = 0.0;
        let mut day_wins = 0;

        for &pnl in trades {
            day_pnl += pnl;
            if pnl > 0.0 { day_wins += 1; }
        }

        let win_rate = (day_wins as f64 / trades.len() as f64) * 100.0;
        weekly_total += day_pnl;

        if day_pnl > best_day.1 { best_day = (i, day_pnl); }
        if day_pnl < worst_day.1 { worst_day = (i, day_pnl); }

        let emoji = if day_pnl >= 0.0 { "📈" } else { "📉" };
        println!("║ {} {:11}: {:>+8.2} ({} trades, WR: {:.0}%)  ║",
                 emoji, days[i], day_pnl, trades.len(), win_rate);
    }

    println!("╠═════════════════════════════════════════════╣");
    println!("║ Weekly total:                 {:>+12.2} ║", weekly_total);
    println!("║ Best day:    {:11} ({:>+8.2})     ║", days[best_day.0], best_day.1);
    println!("║ Worst day:   {:11} ({:>+8.2})     ║", days[worst_day.0], worst_day.1);
    println!("╚═════════════════════════════════════════════╝");
}
```

## What We Learned

| Construct | Syntax | When to Use |
|-----------|--------|-------------|
| Simple for | `for x in collection` | Iterate with ownership |
| By reference | `for x in &collection` | Keep collection |
| Mutable | `for x in &mut coll` | Modify elements |
| With index | `.iter().enumerate()` | Need element number |
| Range | `for i in 0..n` | Iteration counter |
| break | `break;` | Early exit |
| continue | `continue;` | Skip iteration |

## Homework

1. Write a function `calculate_daily_returns(prices: &[f64]) -> Vec<f64>` that computes daily returns: `(price[i] - price[i-1]) / price[i-1]`

2. Create a function `find_best_trade(trades: &[(String, f64)]) -> Option<(String, f64)>` that finds the most profitable trade

3. Implement a function `calculate_drawdown(equity: &[f64]) -> f64` that computes the maximum portfolio drawdown

4. Write a program that simulates a trading week: generates random trades for each day and outputs final statistics using nested loops

## Navigation

[← Previous day](../020-loop-market-scanner/en.md) | [Next day →](../022-while-waiting-for-signal/en.md)
