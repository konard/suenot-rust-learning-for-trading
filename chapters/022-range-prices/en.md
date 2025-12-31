# Day 22: Range — Prices from 100 to 200

## Trading Analogy

In trading, we constantly work with **ranges**:
- Price in the range from 100 to 200
- Last 10 candles
- Indices from 0 to portfolio size
- Support and resistance levels

Range in Rust is a way to express a **sequence of values** between a starting and ending point.

## Range Types

```rust
fn main() {
    // Range (excludes end): a..b
    let price_range = 100..200;
    println!("Range 100..200");

    // RangeInclusive (includes end): a..=b
    let inclusive_range = 100..=200;
    println!("Inclusive range 100..=200");

    // RangeFrom (from a to infinity): a..
    let from_100 = 100..;
    println!("From 100..");

    // RangeTo (from beginning to b, excluding): ..b
    let to_100 = ..100;
    println!("To ..100");

    // RangeToInclusive (from beginning to b, including): ..=b
    let to_100_incl = ..=100;
    println!("To inclusive ..=100");

    // RangeFull (entire range): ..
    let full = ..;
    println!("Full range ..");
}
```

## Range in for Loops

```rust
fn main() {
    println!("=== Analyzing 10 Candles ===");

    // Indices 0..10 (0,1,2,...,9)
    for i in 0..10 {
        println!("Candle {}: analyzing...", i);
    }

    println!("\n=== Prices from 100 to 105 inclusive ===");

    // 100..=105 (100,101,102,103,104,105)
    for price in 100..=105 {
        println!("Price level: ${}", price);
    }
}
```

## Range for Array Element Access

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // First 5 candles (indices 0,1,2,3,4)
    let first_five = &closes[0..5];
    println!("First 5: {:?}", first_five);

    // Last 3 candles (indices 7,8,9)
    let last_three = &closes[7..10];
    println!("Last 3: {:?}", last_three);

    // From index 3 to end
    let from_third = &closes[3..];
    println!("From index 3: {:?}", from_third);

    // From beginning to index 4 (not including)
    let to_fourth = &closes[..4];
    println!("Up to index 4: {:?}", to_fourth);

    // Entire array
    let all = &closes[..];
    println!("All: {:?}", all);
}
```

## Practical Example: Sliding Window for SMA

```rust
fn main() {
    let prices = [100.0, 102.0, 104.0, 103.0, 105.0,
                  107.0, 106.0, 108.0, 110.0, 109.0];

    let window_size = 3;

    println!("=== SMA-{} Calculation ===", window_size);

    // Iterate through all possible windows
    for i in 0..=(prices.len() - window_size) {
        let window = &prices[i..i + window_size];
        let sma: f64 = window.iter().sum::<f64>() / window_size as f64;
        println!("Window [{}-{}]: {:?} -> SMA: {:.2}",
                 i, i + window_size - 1, window, sma);
    }
}
```

## Practical Example: Finding Support/Resistance Levels

```rust
fn main() {
    let prices = [100.0, 105.0, 103.0, 108.0, 106.0,
                  110.0, 107.0, 112.0, 109.0, 115.0];

    // Define price range
    let min_price = 100;
    let max_price = 120;
    let step = 5;

    println!("=== Price Level Analysis ===");

    // Analyze each level
    for level in (min_price..=max_price).step_by(step) {
        let level_f = level as f64;
        let touches = prices.iter()
            .filter(|&&p| (p - level_f).abs() < 2.0)
            .count();

        if touches > 0 {
            println!("Level ${}: {} touches", level, touches);
        }
    }
}
```

## Practical Example: Trading Sessions

```rust
fn main() {
    // Trading session hours (0-23)
    let asian_session = 0..9;      // 00:00 - 08:59
    let european_session = 7..16;  // 07:00 - 15:59
    let american_session = 13..22; // 13:00 - 21:59

    let current_hour = 14;

    println!("Current hour: {}:00", current_hour);

    if asian_session.contains(&current_hour) {
        println!("Asian session is active");
    }
    if european_session.contains(&current_hour) {
        println!("European session is active");
    }
    if american_session.contains(&current_hour) {
        println!("American session is active");
    }
}
```

## Range Methods

```rust
fn main() {
    let range = 1..10;

    // contains() - check membership
    println!("Range 1..10 contains 5: {}", range.contains(&5));
    println!("Range 1..10 contains 10: {}", range.contains(&10));

    let inclusive = 1..=10;
    println!("Range 1..=10 contains 10: {}", inclusive.contains(&10));

    // is_empty() - check if empty
    let empty_range = 5..5;
    println!("Range 5..5 is empty: {}", empty_range.is_empty());

    let normal_range = 5..10;
    println!("Range 5..10 is empty: {}", normal_range.is_empty());
}
```

## Practical Example: Filtering Orders by Price

```rust
fn main() {
    let orders = [
        ("BUY", 100.0),
        ("SELL", 150.0),
        ("BUY", 120.0),
        ("SELL", 200.0),
        ("BUY", 180.0),
        ("SELL", 130.0),
    ];

    // Price range of interest
    let price_range = 100.0..=150.0;

    println!("=== Orders in range ${:.0} - ${:.0} ===", 100.0, 150.0);

    for (i, (side, price)) in orders.iter().enumerate() {
        // Check if price falls within range
        if *price >= 100.0 && *price <= 150.0 {
            println!("Order {}: {} @ ${:.2}", i, side, price);
        }
    }

    // Count orders in range
    let count = orders.iter()
        .filter(|(_, price)| *price >= 100.0 && *price <= 150.0)
        .count();
    println!("\nTotal orders in range: {}", count);
}
```

## Practical Example: Volatility Analysis by Periods

```rust
fn main() {
    let hourly_volatility = [
        0.5, 0.4, 0.3, 0.2, 0.2, 0.3,  // 00:00 - 05:59
        0.6, 0.8, 1.2, 1.5, 1.3, 1.1,  // 06:00 - 11:59
        0.9, 1.4, 1.8, 2.0, 1.7, 1.5,  // 12:00 - 17:59
        1.2, 0.9, 0.7, 0.5, 0.4, 0.3,  // 18:00 - 23:59
    ];

    // Analyze different sessions
    let sessions = [
        ("Night", 0..6),
        ("Morning", 6..12),
        ("Afternoon", 12..18),
        ("Evening", 18..24),
    ];

    println!("=== Volatility by Session ===");

    for (name, range) in sessions {
        let session_vol = &hourly_volatility[range.clone()];
        let avg_vol: f64 = session_vol.iter().sum::<f64>() / session_vol.len() as f64;
        let max_vol = session_vol.iter().cloned().fold(0.0_f64, f64::max);

        println!("{:10}: Avg={:.2}%, Max={:.2}%", name, avg_vol, max_vol);
    }
}
```

## Practical Example: Splitting Data into Periods

```rust
fn main() {
    let daily_closes: [f64; 20] = [
        100.0, 102.0, 101.0, 103.0, 105.0,  // Week 1
        104.0, 106.0, 108.0, 107.0, 109.0,  // Week 2
        110.0, 108.0, 109.0, 111.0, 113.0,  // Week 3
        112.0, 114.0, 115.0, 113.0, 116.0,  // Week 4
    ];

    println!("=== Weekly Performance ===");

    // Split into weeks of 5 days
    for week in 0..4 {
        let start = week * 5;
        let end = start + 5;
        let week_data = &daily_closes[start..end];

        let open = week_data[0];
        let close = week_data[4];
        let change = (close - open) / open * 100.0;

        let max = week_data.iter().cloned().fold(f64::MIN, f64::max);
        let min = week_data.iter().cloned().fold(f64::MAX, f64::min);

        println!("Week {}: Open={:.0}, Close={:.0}, Change={:+.2}%, Range={:.0}-{:.0}",
                 week + 1, open, close, change, min, max);
    }
}
```

## Reverse Range with rev()

```rust
fn main() {
    println!("=== Countdown to Market Open ===");

    // Countdown
    for seconds in (1..=10).rev() {
        println!("{}...", seconds);
    }
    println!("Market is OPEN!");

    println!("\n=== Last 5 Trades (newest first) ===");

    let trades = ["Trade A", "Trade B", "Trade C", "Trade D", "Trade E"];

    for i in (0..trades.len()).rev() {
        println!("{}: {}", trades.len() - i, trades[i]);
    }
}
```

## step_by() for Custom Step

```rust
fn main() {
    // Fibonacci-style levels (every 10%)
    println!("=== Fibonacci-style Levels ===");
    for level in (0..=100).step_by(10) {
        println!("{}% retracement", level);
    }

    // Time intervals (every 15 minutes)
    println!("\n=== 15-minute Candles in 1 Hour ===");
    for minute in (0..60).step_by(15) {
        println!("Candle at :{:02}", minute);
    }

    // Price levels with step of 50
    println!("\n=== Price Levels ===");
    for price in (1000..=1500).step_by(50) {
        println!("Support/Resistance at ${}", price);
    }
}
```

## Range with Different Types

```rust
fn main() {
    // Range with i32
    let int_range: std::ops::Range<i32> = -10..10;
    println!("Integer range: {:?}", int_range);

    // Range with char
    for c in 'A'..='Z' {
        print!("{}", c);
    }
    println!(" <- Ticker symbols");

    // Important: Range<f64> does not support iteration!
    // For floats, use other approaches

    let start = 100.0_f64;
    let end = 110.0_f64;
    let step = 0.5_f64;

    println!("\nPrice levels (float):");
    let mut price = start;
    while price <= end {
        println!("  ${:.1}", price);
        price += step;
    }
}
```

## What We Learned

| Syntax | Type | Description |
|--------|------|-------------|
| `a..b` | `Range` | From a to b (excluding b) |
| `a..=b` | `RangeInclusive` | From a to b (including b) |
| `a..` | `RangeFrom` | From a to infinity |
| `..b` | `RangeTo` | From beginning to b |
| `..=b` | `RangeToInclusive` | From beginning to b inclusive |
| `..` | `RangeFull` | Entire range |

| Method | Description |
|--------|-------------|
| `contains(&x)` | Check if range contains x |
| `is_empty()` | Check if range is empty |
| `rev()` | Reverse iteration |
| `step_by(n)` | Iteration with step n |

## Homework

1. **Trading Windows**: Create an array of 24 volume values (by hour). Find the 3 most active 4-hour windows using Range to select data.

2. **Trend Analysis**: Given an array of 50 closing prices. Using Range, divide it into 5 periods of 10 days each. Determine which period showed the best growth.

3. **Order Grid**: Write a function that creates a grid of limit orders from price A to price B with step S. Use Range and step_by().

4. **Pattern Search**: Given an array of OHLC data. Using Range, find all candles where the closing price was within ±1% of the opening price (doji candles).

## Navigation

[← Previous day](../021-loops-market-scanner/en.md) | [Next day →](../023-if-let-option/en.md)
