# Day 24: continue — Skip Losing Trades in Report

## Trading Analogy

Imagine you're preparing a monthly trading report. You need to output only profitable trades, skipping the losing ones. You don't stop working on the report (like with `break`), you simply **skip** the unsuitable rows and move to the next trade.

The `continue` operator does exactly that — it skips the current loop iteration and immediately jumps to the next one.

## Basic continue Syntax

```rust
fn main() {
    let pnl_list = [150.0, -50.0, 200.0, -30.0, 80.0, -10.0, 300.0];

    println!("=== Profitable Trades ===");

    for pnl in pnl_list {
        if pnl < 0.0 {
            continue;  // Skip losing trades
        }
        println!("Profit: ${:.2}", pnl);
    }
}
```

**Output:**
```
=== Profitable Trades ===
Profit: $150.00
Profit: $200.00
Profit: $80.00
Profit: $300.00
```

## Difference Between break and continue

```rust
fn main() {
    let prices = [100.0, 102.0, 98.0, 105.0, 95.0, 110.0];

    println!("=== With break (stop on first drop) ===");
    for price in prices {
        if price < 100.0 {
            println!("Price dropped below 100, stop!");
            break;  // Exit the loop completely
        }
        println!("Price: ${}", price);
    }

    println!("\n=== With continue (skip drops) ===");
    for price in prices {
        if price < 100.0 {
            continue;  // Skip this iteration
        }
        println!("Price: ${}", price);
    }
}
```

**Output:**
```
=== With break (stop on first drop) ===
Price: $100
Price: $102
Price dropped below 100, stop!

=== With continue (skip drops) ===
Price: $100
Price: $102
Price: $105
Price: $110
```

## Filtering Trades by Criteria

```rust
fn main() {
    let trades = [
        ("BTC", 500.0, 0.1),    // ticker, PnL, volume
        ("ETH", -100.0, 0.5),
        ("BTC", 300.0, 0.05),   // small volume
        ("SOL", 200.0, 0.3),
        ("BTC", -50.0, 0.2),
        ("ETH", 400.0, 0.4),
    ];

    println!("=== Report: Profitable BTC trades with volume >= 0.1 ===\n");

    let mut total_profit = 0.0;
    let mut count = 0;

    for (ticker, pnl, volume) in trades {
        // Skip non-BTC
        if ticker != "BTC" {
            continue;
        }

        // Skip losing trades
        if pnl <= 0.0 {
            continue;
        }

        // Skip small volume
        if volume < 0.1 {
            continue;
        }

        println!("{}: +${:.2} (volume: {})", ticker, pnl, volume);
        total_profit += pnl;
        count += 1;
    }

    println!("\nTotal: {} trades, profit: ${:.2}", count, total_profit);
}
```

## Processing Orders with Validation

```rust
fn main() {
    let orders = [
        (100.0, 10),    // price, quantity
        (0.0, 5),       // invalid price
        (50.0, 0),      // invalid quantity
        (75.0, -3),     // negative quantity
        (200.0, 20),    // valid
        (150.0, 15),    // valid
    ];

    println!("=== Processing Valid Orders ===\n");

    let mut processed = 0;
    let mut total_value = 0.0;

    for (i, (price, quantity)) in orders.iter().enumerate() {
        // Validity checks
        if *price <= 0.0 {
            println!("Order #{}: skipped (invalid price)", i + 1);
            continue;
        }

        if *quantity <= 0 {
            println!("Order #{}: skipped (invalid quantity)", i + 1);
            continue;
        }

        let value = price * (*quantity as f64);
        println!("Order #{}: {} units at ${:.2} = ${:.2}", i + 1, quantity, price, value);

        total_value += value;
        processed += 1;
    }

    println!("\nProcessed: {} orders worth ${:.2}", processed, total_value);
}
```

## continue in while Loop

```rust
fn main() {
    let mut prices = vec![100.0, 0.0, 105.0, -5.0, 110.0, 103.0];
    let mut index = 0;

    println!("=== Filtering Anomalous Prices ===\n");

    while index < prices.len() {
        let price = prices[index];

        // Skip invalid prices
        if price <= 0.0 {
            println!("Index {}: anomalous price {}, skipping", index, price);
            index += 1;  // IMPORTANT: increment index before continue!
            continue;
        }

        println!("Index {}: price ${:.2} — OK", index, price);
        index += 1;
    }
}
```

**Important:** In a `while` loop, don't forget to increment the counter **before** `continue`, otherwise you'll get an infinite loop!

## continue in loop

```rust
fn main() {
    let market_data = [
        Some(42000.0),
        None,           // data unavailable
        Some(42100.0),
        None,
        Some(42050.0),
        Some(42200.0),
    ];

    let mut index = 0;
    let mut valid_prices = Vec::new();

    println!("=== Collecting Valid Prices ===\n");

    loop {
        if index >= market_data.len() {
            break;
        }

        let data = market_data[index];
        index += 1;

        // Skip missing data
        let price = match data {
            Some(p) => p,
            None => {
                println!("Data #{}: unavailable, skipping", index);
                continue;
            }
        };

        println!("Data #{}: price ${:.2}", index, price);
        valid_prices.push(price);
    }

    println!("\nCollected {} valid prices: {:?}", valid_prices.len(), valid_prices);
}
```

## Loop Labels: continue with Nested Loops

```rust
fn main() {
    let exchanges = ["Binance", "Coinbase", "Kraken"];
    let tickers = ["BTC", "INVALID", "ETH", "ERROR", "SOL"];

    println!("=== Checking Tickers on Exchanges ===\n");

    'exchange: for exchange in exchanges {
        println!("Exchange: {}", exchange);

        for ticker in tickers {
            // Skip invalid tickers
            if ticker == "INVALID" || ticker == "ERROR" {
                println!("  {} — invalid ticker, skipping", ticker);
                continue;  // Continue with next ticker
            }

            // Skip entire exchange if Kraken and SOL
            if exchange == "Kraken" && ticker == "SOL" {
                println!("  SOL not traded on Kraken, moving to next exchange");
                continue 'exchange;  // Continue with next exchange
            }

            println!("  {} — OK", ticker);
        }
        println!();
    }
}
```

## Practical Example: Generating Trading Report

```rust
fn main() {
    let trades = [
        ("2024-01-15", "BTC", "BUY", 42000.0, 0.5, 150.0),
        ("2024-01-15", "ETH", "BUY", 2200.0, 2.0, -50.0),   // loss
        ("2024-01-16", "BTC", "SELL", 43000.0, 0.3, 300.0),
        ("2024-01-16", "SOL", "BUY", 95.0, 10.0, 0.0),      // breakeven
        ("2024-01-17", "ETH", "SELL", 2300.0, 1.5, 150.0),
        ("2024-01-17", "BTC", "BUY", 41500.0, 0.2, -80.0),  // loss
        ("2024-01-18", "SOL", "SELL", 100.0, 8.0, 40.0),
    ];

    println!("╔════════════════════════════════════════════════════╗");
    println!("║         REPORT: PROFITABLE TRADES                  ║");
    println!("╠════════════════════════════════════════════════════╣");

    let mut total_profit = 0.0;
    let mut trade_count = 0;

    for (date, ticker, side, price, qty, pnl) in trades {
        // Skip losing and breakeven trades
        if pnl <= 0.0 {
            continue;
        }

        println!("║ {} │ {:>4} │ {:>4} │ ${:>8.2} │ {:>4.1} │ +${:>6.2} ║",
                 date, ticker, side, price, qty, pnl);

        total_profit += pnl;
        trade_count += 1;
    }

    println!("╠════════════════════════════════════════════════════╣");
    println!("║ Total profitable trades: {:>3}                       ║", trade_count);
    println!("║ Total profit: ${:>10.2}                        ║", total_profit);
    println!("╚════════════════════════════════════════════════════╝");
}
```

## Portfolio Analysis with Filtering

```rust
fn main() {
    let portfolio = [
        ("AAPL", 150.0, 10, 155.0),   // ticker, buy price, qty, current price
        ("GOOGL", 140.0, 5, 135.0),   // loss
        ("MSFT", 380.0, 8, 395.0),
        ("TSLA", 250.0, 3, 240.0),    // loss
        ("NVDA", 450.0, 6, 520.0),
        ("META", 330.0, 4, 325.0),    // loss
    ];

    println!("=== Positions in Profit ===\n");

    let mut total_unrealized = 0.0;

    for (ticker, buy_price, quantity, current_price) in portfolio {
        let pnl = (current_price - buy_price) * quantity as f64;

        // Skip losing positions
        if pnl <= 0.0 {
            continue;
        }

        let pnl_percent = ((current_price / buy_price) - 1.0) * 100.0;

        println!("{}: {} shares | Bought: ${:.2} | Now: ${:.2}",
                 ticker, quantity, buy_price, current_price);
        println!("     Unrealized profit: +${:.2} ({:+.2}%)\n",
                 pnl, pnl_percent);

        total_unrealized += pnl;
    }

    println!("Total unrealized profit: ${:.2}", total_unrealized);
}
```

## Calculating Statistics While Skipping Outliers

```rust
fn main() {
    let daily_returns = [
        0.5, 1.2, -0.3, 0.8, 15.0,   // 15% — outlier
        -0.5, 0.2, -20.0, 0.7, 1.1,  // -20% — outlier
        0.3, -0.8, 0.9, 0.4, -0.2,
    ];

    let outlier_threshold = 10.0;  // Outlier threshold: ±10%

    println!("=== Calculating Average Return (excluding outliers) ===\n");

    let mut sum = 0.0;
    let mut count = 0;
    let mut outliers = 0;

    for ret in daily_returns {
        // Skip outliers
        if ret.abs() > outlier_threshold {
            println!("Outlier: {:+.1}% — skipping", ret);
            outliers += 1;
            continue;
        }

        sum += ret;
        count += 1;
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };

    println!("\nTotal values: {}", daily_returns.len());
    println!("Outliers skipped: {}", outliers);
    println!("Values counted: {}", count);
    println!("Average return: {:+.2}%", average);
}
```

## Pattern: Early continue for Clean Code

```rust
fn main() {
    let candles = [
        (100.0, 105.0, 98.0, 103.0, 1000),  // open, high, low, close, volume
        (103.0, 103.0, 103.0, 103.0, 0),    // zero volume — skip
        (103.0, 108.0, 102.0, 107.0, 1500),
        (107.0, 107.0, 100.0, 101.0, 800),
        (101.0, 101.0, 101.0, 101.0, 50),   // doji with small volume — skip
        (101.0, 110.0, 100.0, 109.0, 2000),
    ];

    let min_volume = 100;

    println!("=== Analyzing Significant Candles ===\n");

    for (i, (open, high, low, close, volume)) in candles.iter().enumerate() {
        // Early checks with continue
        if *volume < min_volume {
            continue;  // Skip low volume candles
        }

        if open == close && high == low {
            continue;  // Skip "empty" candles
        }

        // Main logic for valid candles
        let body = (close - open).abs();
        let range = high - low;
        let body_ratio = if range > 0.0 { body / range * 100.0 } else { 0.0 };

        let candle_type = if close > open { "bullish" } else { "bearish" };

        println!("Candle #{}: {} | Range: {:.2} | Body: {:.1}%",
                 i + 1, candle_type, range, body_ratio);
    }
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| `continue` | Skip current iteration | Skip losing trades |
| `break` vs `continue` | Exit vs skip | Stop-loss vs filtering |
| `continue` in `while` | Don't forget increment! | Stream data processing |
| Loop labels | `continue 'label` | Skip entire exchange |
| Early continue | Clean code with checks | Data validation |

## Homework

1. Write a program that iterates through a list of trades and outputs only those where profit exceeds $100 and volume is greater than 0.5

2. Create a market data processing loop that skips:
   - Prices equal to zero
   - Negative prices
   - Prices with anomalous spread (> 5%)

3. Implement portfolio analysis using `continue`:
   - Skip positions with zero quantity
   - Skip positions where loss is less than 1%
   - Output only positions requiring attention (loss > 5%)

4. Using loop labels, write a program that:
   - Iterates through multiple exchanges
   - Checks several tickers on each exchange
   - Skips the entire exchange if an invalid API response is detected

## Navigation

[← Previous day](../023-break-take-profit/en.md) | [Next day →](../025-match-order-type/en.md)
