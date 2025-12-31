# Day 81: Iterating Vec — Going Through All Orders

## Trading Analogy

Every day a trader works with multiple orders:
- **Reviewing all open positions** — to assess current portfolio state
- **Analyzing trade history** — to calculate P&L
- **Checking all stop-losses** — for risk management
- **Calculating total commissions** — across all trades

Iterating through a vector is like going through all orders in an order book, one by one.

## Basic Iteration Methods

### Simple for Loop

```rust
fn main() {
    let orders = vec!["BUY BTC", "SELL ETH", "BUY SOL", "SELL BTC"];

    println!("=== All Orders ===");
    for order in &orders {
        println!("Order: {}", order);
    }

    // orders is still available thanks to &
    println!("\nTotal orders: {}", orders.len());
}
```

### Consuming Iteration

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 42200.0];

    // into_iter() takes ownership
    for price in prices.into_iter() {
        println!("Price: ${}", price);
    }

    // prices is no longer available!
    // println!("{:?}", prices);  // Compilation error!
}
```

### Iteration with Index (enumerate)

```rust
fn main() {
    let trades = vec![
        ("BTC", 100.0, 42000.0),
        ("ETH", 50.0, 2200.0),
        ("SOL", 200.0, 25.0),
    ];

    println!("=== Trade History ===");
    for (index, (symbol, amount, price)) in trades.iter().enumerate() {
        let value = amount * price;
        println!("#{}: {} {} units @ ${} = ${:.2}",
                 index + 1, symbol, amount, price, value);
    }
}
```

## Iterator Methods for Trading

### Filtering Orders

```rust
fn main() {
    let orders = vec![
        ("BTC", "BUY", 42000.0),
        ("ETH", "SELL", 2200.0),
        ("BTC", "SELL", 42500.0),
        ("SOL", "BUY", 25.0),
        ("BTC", "BUY", 41800.0),
    ];

    // Only BTC orders
    println!("=== BTC Orders ===");
    for order in orders.iter().filter(|(symbol, _, _)| *symbol == "BTC") {
        println!("{:?}", order);
    }

    // Only buy orders
    println!("\n=== All Buy Orders ===");
    let buys: Vec<_> = orders.iter()
        .filter(|(_, side, _)| *side == "BUY")
        .collect();

    for buy in &buys {
        println!("{:?}", buy);
    }
}
```

### Calculating Sum

```rust
fn main() {
    let trade_profits = vec![150.0, -50.0, 200.0, -30.0, 180.0, -20.0];

    // Total P&L
    let total_pnl: f64 = trade_profits.iter().sum();
    println!("Total P&L: ${:.2}", total_pnl);

    // Sum of profitable trades
    let total_profit: f64 = trade_profits.iter()
        .filter(|&&p| p > 0.0)
        .sum();
    println!("Total profit: ${:.2}", total_profit);

    // Sum of losing trades
    let total_loss: f64 = trade_profits.iter()
        .filter(|&&p| p < 0.0)
        .sum();
    println!("Total loss: ${:.2}", total_loss);

    // Profit factor
    if total_loss != 0.0 {
        let profit_factor = total_profit / total_loss.abs();
        println!("Profit factor: {:.2}", profit_factor);
    }
}
```

### Transformation (map)

```rust
fn main() {
    let prices_usd = vec![42000.0, 2200.0, 25.0, 0.35];
    let usd_to_eur = 0.92;

    // Convert to EUR
    let prices_eur: Vec<f64> = prices_usd.iter()
        .map(|price| price * usd_to_eur)
        .collect();

    println!("USD prices: {:?}", prices_usd);
    println!("EUR prices: {:?}", prices_eur);

    // Calculate percentage changes
    let closes = vec![42000.0, 42500.0, 42200.0, 42800.0, 43000.0];

    let returns: Vec<f64> = closes.windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect();

    println!("\nDaily returns:");
    for (i, ret) in returns.iter().enumerate() {
        let sign = if *ret >= 0.0 { "+" } else { "" };
        println!("  Day {}: {}{}%", i + 1, sign, ret);
    }
}
```

### Finding Extremes

```rust
fn main() {
    let daily_prices = vec![
        ("2024-01-01", 42000.0),
        ("2024-01-02", 42500.0),
        ("2024-01-03", 41800.0),
        ("2024-01-04", 43200.0),
        ("2024-01-05", 42900.0),
    ];

    // Maximum price
    if let Some((date, price)) = daily_prices.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("Maximum: {} - ${}", date, price);
    }

    // Minimum price
    if let Some((date, price)) = daily_prices.iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("Minimum: {} - ${}", date, price);
    }

    // Price range
    let prices: Vec<f64> = daily_prices.iter().map(|(_, p)| *p).collect();
    let max = prices.iter().cloned().fold(f64::MIN, f64::max);
    let min = prices.iter().cloned().fold(f64::MAX, f64::min);
    println!("Range: ${:.2}", max - min);
}
```

## Mutable Iteration

```rust
fn main() {
    let mut portfolio = vec![
        ("BTC", 1.5, 42000.0),   // (symbol, amount, entry price)
        ("ETH", 10.0, 2200.0),
        ("SOL", 100.0, 25.0),
    ];

    println!("=== Before Price Update ===");
    for (symbol, amount, price) in &portfolio {
        println!("{}: {} units @ ${}", symbol, amount, price);
    }

    // Update prices (simulating market movement)
    let price_changes = [1.05, 0.98, 1.12];  // +5%, -2%, +12%

    for (i, (_, _, price)) in portfolio.iter_mut().enumerate() {
        *price *= price_changes[i];
    }

    println!("\n=== After Price Update ===");
    for (symbol, amount, price) in &portfolio {
        println!("{}: {} units @ ${:.2}", symbol, amount, price);
    }
}
```

## Practical Example: Order Book Analysis

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,      // "BUY" or "SELL"
    price: f64,
    amount: f64,
    status: String,    // "OPEN", "FILLED", "CANCELLED"
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC".into(), side: "BUY".into(),
                price: 42000.0, amount: 0.5, status: "FILLED".into() },
        Order { id: 2, symbol: "ETH".into(), side: "SELL".into(),
                price: 2200.0, amount: 5.0, status: "OPEN".into() },
        Order { id: 3, symbol: "BTC".into(), side: "BUY".into(),
                price: 41500.0, amount: 0.3, status: "OPEN".into() },
        Order { id: 4, symbol: "BTC".into(), side: "SELL".into(),
                price: 43000.0, amount: 0.2, status: "CANCELLED".into() },
        Order { id: 5, symbol: "SOL".into(), side: "BUY".into(),
                price: 25.0, amount: 100.0, status: "FILLED".into() },
    ];

    // Statistics by status
    let filled_count = orders.iter().filter(|o| o.status == "FILLED").count();
    let open_count = orders.iter().filter(|o| o.status == "OPEN").count();
    let cancelled_count = orders.iter().filter(|o| o.status == "CANCELLED").count();

    println!("=== Order Statistics ===");
    println!("Filled: {}", filled_count);
    println!("Open: {}", open_count);
    println!("Cancelled: {}", cancelled_count);

    // Total volume of filled orders by symbol
    println!("\n=== Volumes by Symbol ===");
    for symbol in ["BTC", "ETH", "SOL"] {
        let volume: f64 = orders.iter()
            .filter(|o| o.symbol == symbol && o.status == "FILLED")
            .map(|o| o.price * o.amount)
            .sum();
        if volume > 0.0 {
            println!("{}: ${:.2}", symbol, volume);
        }
    }

    // Open buy orders
    println!("\n=== Open BUY Orders ===");
    for order in orders.iter().filter(|o| o.side == "BUY" && o.status == "OPEN") {
        println!("#{}: {} {} @ ${}", order.id, order.symbol, order.amount, order.price);
    }
}
```

## Practical Example: Moving Average Calculation

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let period = 5;

    // Calculate SMA
    let sma: Vec<f64> = prices.windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect();

    println!("=== SMA-{} ===", period);
    for (i, value) in sma.iter().enumerate() {
        let price = prices[i + period - 1];
        let signal = if price > *value { "above SMA" } else { "below SMA" };
        println!("Period {}: SMA={:.2}, Price={:.2} ({})",
                 i + period, value, price, signal);
    }
}
```

## Practical Example: Trading Signal Detection

```rust
fn main() {
    let candles = vec![
        (42000.0, 42500.0, 41800.0, 42300.0),  // (open, high, low, close)
        (42300.0, 42600.0, 42200.0, 42100.0),
        (42100.0, 42400.0, 42000.0, 42350.0),
        (42350.0, 43000.0, 42300.0, 42900.0),
        (42900.0, 43200.0, 42800.0, 43100.0),
    ];

    println!("=== Candle Analysis ===");

    for (i, (open, high, low, close)) in candles.iter().enumerate() {
        let body = (close - open).abs();
        let upper_shadow = high - f64::max(*open, *close);
        let lower_shadow = f64::min(*open, *close) - low;
        let range = high - low;

        let candle_type = if close > open { "bullish" } else { "bearish" };

        // Identify patterns
        let pattern = if body < range * 0.1 {
            "Doji"
        } else if lower_shadow > body * 2.0 && upper_shadow < body * 0.5 {
            "Hammer"
        } else if upper_shadow > body * 2.0 && lower_shadow < body * 0.5 {
            "Shooting Star"
        } else {
            "Regular candle"
        };

        println!("Candle {}: {} {} (body={:.0}, range={:.0})",
                 i + 1, candle_type, pattern, body, range);
    }

    // Search for reversal patterns
    println!("\n=== Signals ===");
    for i in 1..candles.len() {
        let prev = candles[i - 1];
        let curr = candles[i];

        // Bullish engulfing
        if prev.3 < prev.0 && curr.3 > curr.0 &&
           curr.0 < prev.3 && curr.3 > prev.0 {
            println!("Candle {}: Bullish Engulfing! Buy signal", i + 1);
        }

        // Bearish engulfing
        if prev.3 > prev.0 && curr.3 < curr.0 &&
           curr.0 > prev.3 && curr.3 < prev.0 {
            println!("Candle {}: Bearish Engulfing! Sell signal", i + 1);
        }
    }
}
```

## Iterator Chaining

```rust
fn main() {
    let trades = vec![
        ("BTC", 100.0),
        ("ETH", -50.0),
        ("BTC", 200.0),
        ("SOL", -30.0),
        ("BTC", 150.0),
        ("ETH", 80.0),
    ];

    // Complex chain: filter + transform + aggregate
    let btc_profit: f64 = trades.iter()
        .filter(|(symbol, _)| *symbol == "BTC")  // Only BTC
        .map(|(_, profit)| profit)                // Extract profit
        .filter(|&&p| p > 0.0)                    // Only positive
        .sum();                                    // Sum up

    println!("BTC profit: ${:.2}", btc_profit);

    // Top 3 profitable trades
    let mut sorted_trades = trades.clone();
    sorted_trades.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nTop 3 trades:");
    for (symbol, profit) in sorted_trades.iter().take(3) {
        println!("  {}: ${:.2}", symbol, profit);
    }

    // Count of profitable trades per asset
    println!("\nProfitable trades by asset:");
    for symbol in ["BTC", "ETH", "SOL"] {
        let count = trades.iter()
            .filter(|(s, p)| *s == symbol && *p > 0.0)
            .count();
        println!("  {}: {}", symbol, count);
    }
}
```

## What We Learned

| Method | Description |
|--------|-------------|
| `iter()` | Iterator over references |
| `iter_mut()` | Iterator for mutation |
| `into_iter()` | Consuming iterator |
| `enumerate()` | Adds indices |
| `filter()` | Filter elements |
| `map()` | Transform elements |
| `sum()` | Sum of elements |
| `collect()` | Collect into collection |
| `windows(n)` | Sliding window |
| `take(n)` | First n elements |

## Homework

1. **Portfolio Analysis**: Create a vector of positions with fields (symbol, amount, entry price, current price). Calculate:
   - Total portfolio P&L
   - Profitable and losing positions separately
   - Percentage of profitable positions

2. **Trade History**: Write a function that takes a vector of trades and returns:
   - Best trade
   - Worst trade
   - Average profit
   - Profit factor

3. **Moving Averages**: Implement EMA (Exponential Moving Average) calculation through iteration

4. **Pattern Detector**: Write a function to find the "Three White Soldiers" pattern (three consecutive bullish candles with increasing closes)

## Navigation

[← Previous day](../080-vec-modification/en.md) | [Next day →](../082-vec-slices-trading/en.md)
