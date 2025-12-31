# Day 49: Array Slices — Part of Price Series

## Trading Analogy

In algorithmic trading, we constantly work with **portions of data**:
- Analyzing the last 14 candles for RSI
- 20-period moving average (SMA-20)
- Comparing morning and evening trading sessions
- Extracting price ranges for specific periods

A **slice** is a reference to a contiguous sequence of elements. It's like a "window" into data without copying.

## What is a Slice?

A slice is a **borrowed portion** of an array or vector:

```rust
fn main() {
    let prices = [100.0, 105.0, 103.0, 108.0, 110.0, 107.0, 112.0];

    // Slice: elements from index 2 to 5 (not including 5)
    let window: &[f64] = &prices[2..5];

    println!("Full array: {:?}", prices);
    println!("Slice [2..5]: {:?}", window);  // [103.0, 108.0, 110.0]
    println!("Slice length: {}", window.len());  // 3
}
```

**Key features:**
- A slice doesn't own the data — it only references it
- Slice type: `&[T]` (reference to slice)
- No memory allocation required — it's just a pointer + length

## Slice Syntax

```rust
fn main() {
    let candles = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    // Full syntax: [start..end]
    let slice1 = &candles[2..5];     // [3.0, 4.0, 5.0]

    // From beginning: [..end]
    let first_three = &candles[..3]; // [1.0, 2.0, 3.0]

    // To end: [start..]
    let last_four = &candles[6..];   // [7.0, 8.0, 9.0, 10.0]

    // Entire array: [..]
    let all = &candles[..];          // All 10 elements

    // Inclusive range: [start..=end]
    let inclusive = &candles[2..=5]; // [3.0, 4.0, 5.0, 6.0]

    println!("slice1: {:?}", slice1);
    println!("first_three: {:?}", first_three);
    println!("last_four: {:?}", last_four);
    println!("inclusive [2..=5]: {:?}", inclusive);
}
```

## Slices and Functions

The main advantage of slices is **function flexibility**:

```rust
fn main() {
    // Fixed-size array
    let daily_closes: [f64; 5] = [42000.0, 42500.0, 42300.0, 42800.0, 43000.0];

    // Dynamic-size vector
    let hourly_closes: Vec<f64> = vec![42100.0, 42150.0, 42200.0, 42180.0];

    // One function works with both!
    println!("Daily SMA: {:.2}", calculate_sma(&daily_closes));
    println!("Hourly SMA: {:.2}", calculate_sma(&hourly_closes));

    // And with partial data
    println!("Last 3 daily: {:.2}", calculate_sma(&daily_closes[2..]));
    println!("First 2 hourly: {:.2}", calculate_sma(&hourly_closes[..2]));
}

// Function accepts a slice — works with arrays, vectors, partial data
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Practical Example: Trading Session Analysis

```rust
fn main() {
    // Hourly prices for a trading day (24 candles)
    let hourly_prices: [f64; 24] = [
        // Asian session (00:00 - 08:00)
        42000.0, 42050.0, 42100.0, 42080.0, 42120.0, 42150.0, 42200.0, 42180.0,
        // European session (08:00 - 16:00)
        42250.0, 42300.0, 42280.0, 42350.0, 42400.0, 42380.0, 42420.0, 42450.0,
        // American session (16:00 - 24:00)
        42500.0, 42550.0, 42480.0, 42520.0, 42600.0, 42650.0, 42700.0, 42680.0,
    ];

    // Extract slices for each session
    let asian_session = &hourly_prices[0..8];
    let european_session = &hourly_prices[8..16];
    let american_session = &hourly_prices[16..24];

    println!("=== Trading Session Analysis ===\n");

    println!("Asian Session:");
    analyze_session(asian_session);

    println!("\nEuropean Session:");
    analyze_session(european_session);

    println!("\nAmerican Session:");
    analyze_session(american_session);

    // Compare volatility
    println!("\n=== Volatility Comparison ===");
    println!("Asia: {:.2}%", calculate_volatility(asian_session));
    println!("Europe: {:.2}%", calculate_volatility(european_session));
    println!("America: {:.2}%", calculate_volatility(american_session));
}

fn analyze_session(prices: &[f64]) {
    let open = prices.first().unwrap();
    let close = prices.last().unwrap();
    let high = prices.iter().cloned().fold(f64::MIN, f64::max);
    let low = prices.iter().cloned().fold(f64::MAX, f64::min);
    let change = (close - open) / open * 100.0;

    println!("  Open: ${:.2}, Close: ${:.2}", open, close);
    println!("  High: ${:.2}, Low: ${:.2}", high, low);
    println!("  Change: {:+.2}%", change);
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let high = prices.iter().cloned().fold(f64::MIN, f64::max);
    let low = prices.iter().cloned().fold(f64::MAX, f64::min);
    let avg = prices.iter().sum::<f64>() / prices.len() as f64;

    (high - low) / avg * 100.0
}
```

## Practical Example: Sliding Window for Indicators

```rust
fn main() {
    let closes = [
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42250.0, 42400.0, 42350.0, 42500.0, 42450.0,
        42600.0, 42550.0, 42700.0, 42650.0, 42800.0,
    ];

    let period = 5;

    println!("=== SMA-{} Using Slices ===\n", period);

    // Sliding window through slices
    for i in 0..=(closes.len() - period) {
        let window = &closes[i..i + period];
        let sma = window.iter().sum::<f64>() / period as f64;

        println!(
            "Window [{:2}..{:2}]: {:?} => SMA: {:.2}",
            i, i + period, window, sma
        );
    }

    // Latest SMA value
    let last_window = &closes[closes.len() - period..];
    let current_sma = last_window.iter().sum::<f64>() / period as f64;
    println!("\nCurrent SMA-{}: {:.2}", period, current_sma);
}
```

## Mutable Slices

Slices can be mutable:

```rust
fn main() {
    let mut order_book = [100.0, 200.0, 150.0, 180.0, 220.0];

    println!("Before: {:?}", order_book);

    // Mutable slice of first three elements
    let top_orders = &mut order_book[..3];

    // Apply price adjustment
    adjust_prices(top_orders, 1.05);  // +5%

    println!("After adjustment: {:?}", order_book);
}

fn adjust_prices(prices: &mut [f64], multiplier: f64) {
    for price in prices.iter_mut() {
        *price *= multiplier;
    }
}
```

## Practical Example: Portfolio Splitting

```rust
fn main() {
    // Portfolio: [symbol_id, quantity, price]
    let mut portfolio: [(u32, f64, f64); 6] = [
        (1, 10.0, 42000.0),   // BTC
        (2, 100.0, 2800.0),   // ETH
        (3, 1000.0, 0.35),    // XRP
        (4, 50.0, 320.0),     // BNB
        (5, 200.0, 28.0),     // LINK
        (6, 500.0, 0.12),     // DOGE
    ];

    // Split into two parts: high-cap and altcoins
    let (high_cap, alt_coins) = portfolio.split_at_mut(3);

    println!("=== High Cap Assets ===");
    for (id, qty, price) in high_cap.iter() {
        println!("Asset {}: {} @ ${:.2} = ${:.2}", id, qty, price, qty * price);
    }

    println!("\n=== Altcoins ===");
    for (id, qty, price) in alt_coins.iter() {
        println!("Asset {}: {} @ ${:.2} = ${:.2}", id, qty, price, qty * price);
    }

    // Update prices in each part independently
    update_prices(high_cap, 1.02);   // +2%
    update_prices(alt_coins, 0.95);  // -5%

    println!("\n=== After Price Update ===");
    for (id, qty, price) in portfolio.iter() {
        println!("Asset {}: ${:.2}", id, qty * price);
    }
}

fn update_prices(assets: &mut [(u32, f64, f64)], multiplier: f64) {
    for (_, _, price) in assets.iter_mut() {
        *price *= multiplier;
    }
}
```

## Slice Methods

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0, 42800.0];

    // Basic methods
    println!("Length: {}", prices.len());
    println!("Is empty: {}", prices.is_empty());

    // First and last elements
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // Splitting
    let (left, right) = prices.split_at(3);
    println!("Left half: {:?}", left);
    println!("Right half: {:?}", right);

    // Get first/last + remainder
    if let Some((first, rest)) = prices.split_first() {
        println!("First: {}, Rest: {:?}", first, rest);
    }

    if let Some((last, init)) = prices.split_last() {
        println!("Last: {}, Init: {:?}", last, init);
    }

    // Chunks (fixed-size splitting)
    println!("\nChunks of 2:");
    for chunk in prices.chunks(2) {
        println!("  {:?}", chunk);
    }

    // Windows (sliding)
    println!("\nWindows of 3:");
    for window in prices.windows(3) {
        println!("  {:?}", window);
    }
}
```

## Practical Example: Pattern Detection

```rust
fn main() {
    let prices = [
        100.0, 102.0, 101.0, 103.0, 105.0,  // Rising
        104.0, 103.0, 101.0, 99.0, 97.0,    // Falling
        98.0, 100.0, 102.0, 104.0, 106.0,   // Rising
    ];

    println!("=== Trend Detection Using Windows ===\n");

    // Analyze with 3-candle windows
    for (i, window) in prices.windows(3).enumerate() {
        let trend = detect_trend(window);
        println!(
            "Candles [{:2}-{:2}]: {:?} => {}",
            i, i + 2, window, trend
        );
    }

    // Count trends
    let mut uptrends = 0;
    let mut downtrends = 0;

    for window in prices.windows(3) {
        match detect_trend(window) {
            "Uptrend" => uptrends += 1,
            "Downtrend" => downtrends += 1,
            _ => {}
        }
    }

    println!("\n=== Statistics ===");
    println!("Uptrends: {}", uptrends);
    println!("Downtrends: {}", downtrends);
    println!("Trend ratio: {:.2}", uptrends as f64 / downtrends as f64);
}

fn detect_trend(window: &[f64]) -> &'static str {
    if window.len() < 2 {
        return "Unknown";
    }

    let is_uptrend = window.windows(2).all(|w| w[1] > w[0]);
    let is_downtrend = window.windows(2).all(|w| w[1] < w[0]);

    if is_uptrend {
        "Uptrend"
    } else if is_downtrend {
        "Downtrend"
    } else {
        "Sideways"
    }
}
```

## Slice Safety

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42200.0];

    // Safe access via get()
    match prices.get(1..4) {
        Some(slice) => println!("Slice: {:?}", slice),
        None => println!("Index out of bounds!"),
    }

    // Check before creating slice
    let start = 0;
    let end = 5;

    if end <= prices.len() {
        let safe_slice = &prices[start..end];
        println!("Safe slice: {:?}", safe_slice);
    } else {
        println!("Cannot create slice: end ({}) > len ({})", end, prices.len());

        // Use available portion
        let available = &prices[start..];
        println!("Available data: {:?}", available);
    }
}
```

## Exercises

### Exercise 1: Period Comparison

```rust
fn main() {
    let monthly_returns = [
        2.5, -1.2, 3.8, 0.5, -2.1, 4.2,   // First half
        1.8, -0.5, 2.9, -1.5, 3.1, 2.0,   // Second half
    ];

    // TODO: Split the data into two halves and compare:
    // - Average return
    // - Maximum drawdown (minimum value)
    // - Number of profitable months
}
```

### Exercise 2: Rolling Maximum

```rust
fn main() {
    let highs = [42500.0, 42800.0, 42300.0, 43100.0, 42900.0, 43500.0, 43200.0];

    // TODO: Find the maximum for each 3-element window
    // Use the windows() method
}
```

### Exercise 3: Data Normalization

```rust
fn main() {
    let mut prices = [100.0, 150.0, 120.0, 180.0, 140.0];

    // TODO: Normalize prices to [0, 1] range
    // Use mutable slice
    // Formula: (price - min) / (max - min)
}
```

### Exercise 4: Order Batching

```rust
fn main() {
    let orders = [
        ("BTC", 1.5, "buy"),
        ("ETH", 10.0, "sell"),
        ("BTC", 0.5, "buy"),
        ("SOL", 100.0, "sell"),
        ("ETH", 5.0, "buy"),
    ];

    // TODO: Using chunks(), process orders in batches of 2
    // and print information about each batch
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&[T]` | Immutable slice of type T |
| `&mut [T]` | Mutable slice |
| `[a..b]` | Range from a to b (excluding b) |
| `[a..=b]` | Range from a to b (including b) |
| `[..b]` / `[a..]` / `[..]` | Shorthand forms |
| `split_at()` | Split into two parts |
| `chunks()` | Split into fixed-size parts |
| `windows()` | Sliding windows |

## Homework

1. **RSI with Slices**: Implement RSI (Relative Strength Index) calculation using slices to analyze price changes over 14 periods.

2. **Bollinger Bands**: Create a function that takes a price slice and period, and returns upper and lower Bollinger Bands.

3. **Volume Analysis**: Given an array of trading volumes for a month, split into weeks and find the week with the highest average volume.

4. **Pattern Detector**: Write a function that searches for the "three white soldiers" pattern (three consecutive bullish candles) in a closing price array.

## Navigation

[← Day 48](../048-ownership-transfer/en.md) | [Day 50 →](../050-string-slices/en.md)
