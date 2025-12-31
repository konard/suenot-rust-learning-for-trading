# Day 52: Slice Practice: Partial History Analysis

## Trading Analogy

In algorithmic trading, you often need to analyze **part of the history**, not all data:
- Last 14 candles for RSI
- Window of 20 periods for Bollinger Bands
- Last 50 prices for moving average
- Specific trading hour for volume analysis

A **slice** is a "window" into an array of data, allowing you to work with a portion without copying.

## Theory: What is a Slice?

A slice is a reference to a contiguous sequence of elements in an array or vector. Unlike an array:
- A slice **does not own** the data (it's a reference)
- Slice size is **not fixed** at compile time
- A slice is written as `&[T]`

```rust
fn main() {
    let prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    // Full array: [f64; 5] — fixed size
    // Slice: &[f64] — dynamic size

    let slice: &[f64] = &prices[1..4];
    println!("Slice: {:?}", slice);  // [200.0, 300.0, 400.0]
}
```

## Slice Syntax

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // [start..end] — from start to end (not including end)
    let middle = &closes[3..7];
    println!("[3..7]: {:?}", middle);  // 4 elements

    // [..end] — from beginning to end
    let first_five = &closes[..5];
    println!("[..5]: {:?}", first_five);

    // [start..] — from start to end
    let last_five = &closes[5..];
    println!("[5..]: {:?}", last_five);

    // [..] — entire array as slice
    let all = &closes[..];
    println!("All: {:?}", all);

    // [start..=end] — including end
    let inclusive = &closes[0..=2];
    println!("[0..=2]: {:?}", inclusive);  // 3 elements: indices 0, 1, 2
}
```

## Slices and Functions

The main advantage of slices is function universality:

```rust
fn main() {
    let array: [f64; 5] = [100.0, 200.0, 300.0, 400.0, 500.0];
    let vec: Vec<f64> = vec![100.0, 200.0, 300.0, 400.0, 500.0];

    // One function works with both array and vector
    println!("Array SMA: {:.2}", calculate_sma(&array));
    println!("Vec SMA: {:.2}", calculate_sma(&vec));

    // And with part of the data
    println!("First 3 SMA: {:.2}", calculate_sma(&array[..3]));
    println!("Last 3 SMA: {:.2}", calculate_sma(&vec[2..]));
}

// Accepts a slice — works with any data source
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Practical Example 1: Sliding Window

Price analysis using a sliding window:

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    let window_size = 3;

    println!("=== Simple Moving Average (SMA-{}) ===", window_size);

    // Calculate SMA for each window
    for i in 0..=(closes.len() - window_size) {
        let window = &closes[i..i + window_size];
        let sma = calculate_sma(window);
        println!("Window [{}-{}]: {:?} -> SMA: {:.2}",
                 i, i + window_size - 1, window, sma);
    }
}

fn calculate_sma(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Practical Example 2: Trading Session Analysis

Splitting daily data into trading sessions:

```rust
fn main() {
    // 24 hourly prices (simulating daily data)
    let hourly_prices: [f64; 24] = [
        // Asian session (0:00-8:00 UTC)
        42000.0, 42050.0, 42100.0, 42080.0, 42120.0, 42150.0, 42130.0, 42180.0,
        // European session (8:00-16:00 UTC)
        42200.0, 42250.0, 42300.0, 42280.0, 42350.0, 42400.0, 42380.0, 42450.0,
        // American session (16:00-24:00 UTC)
        42500.0, 42480.0, 42550.0, 42600.0, 42580.0, 42650.0, 42700.0, 42680.0,
    ];

    // Slices for each session
    let asian = &hourly_prices[0..8];
    let european = &hourly_prices[8..16];
    let american = &hourly_prices[16..24];

    println!("=== Trading Session Analysis ===\n");

    analyze_session("Asian", asian);
    analyze_session("European", european);
    analyze_session("American", american);

    // Comparing average prices
    println!("\n=== Session Comparison ===");
    let asian_avg = calculate_avg(asian);
    let european_avg = calculate_avg(european);
    let american_avg = calculate_avg(american);

    println!("Asia -> Europe: {:+.2}%",
             (european_avg - asian_avg) / asian_avg * 100.0);
    println!("Europe -> America: {:+.2}%",
             (american_avg - european_avg) / european_avg * 100.0);
}

fn analyze_session(name: &str, prices: &[f64]) {
    let min = find_min(prices);
    let max = find_max(prices);
    let avg = calculate_avg(prices);
    let range = max - min;

    println!("{} session:", name);
    println!("  Min: ${:.2}", min);
    println!("  Max: ${:.2}", max);
    println!("  Avg: ${:.2}", avg);
    println!("  Range: ${:.2} ({:.2}%)", range, range / min * 100.0);
}

fn find_min(prices: &[f64]) -> f64 {
    *prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn find_max(prices: &[f64]) -> f64 {
    *prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn calculate_avg(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Practical Example 3: RSI with Slices

Calculating RSI (Relative Strength Index) using slices:

```rust
fn main() {
    let closes = [
        44000.0, 44200.0, 44100.0, 44400.0, 44300.0,
        44600.0, 44500.0, 44700.0, 44650.0, 44800.0,
        44750.0, 44900.0, 44850.0, 45000.0, 45100.0,
        44900.0, 44800.0, 44950.0, 45050.0, 45200.0,
    ];

    let period = 14;

    println!("=== RSI Analysis (period={}) ===\n", period);

    // Calculate RSI for the last points
    for i in period..closes.len() {
        let window = &closes[i - period..=i];
        let rsi = calculate_rsi(window);

        let signal = if rsi > 70.0 {
            "OVERBOUGHT"
        } else if rsi < 30.0 {
            "OVERSOLD"
        } else {
            "NEUTRAL"
        };

        println!("Index {}: Price ${:.0}, RSI: {:.2} [{}]",
                 i, closes[i], rsi, signal);
    }
}

fn calculate_rsi(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 50.0;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;
    let mut gain_count = 0;
    let mut loss_count = 0;

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
            gain_count += 1;
        } else if change < 0.0 {
            losses += change.abs();
            loss_count += 1;
        }
    }

    let avg_gain = if gain_count > 0 { gains / gain_count as f64 } else { 0.0 };
    let avg_loss = if loss_count > 0 { losses / loss_count as f64 } else { 0.0 };

    if avg_loss == 0.0 {
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}
```

## Practical Example 4: Pattern Detection

Finding trading patterns using slices:

```rust
fn main() {
    let closes = [
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,  // Uptrend
        42250.0, 42150.0, 42100.0, 42000.0, 41900.0,  // Downtrend
        41950.0, 42000.0, 42100.0, 42200.0, 42350.0,  // Uptrend
        42400.0, 42380.0, 42360.0, 42340.0, 42320.0,  // Consolidation
    ];

    println!("=== Pattern Detection (window = 5 candles) ===\n");

    let window_size = 5;

    for i in 0..=(closes.len() - window_size) {
        let window = &closes[i..i + window_size];
        let pattern = detect_pattern(window);

        println!("Window [{:2}-{:2}]: {:?}",
                 i, i + window_size - 1, pattern);
    }
}

#[derive(Debug)]
enum Pattern {
    Uptrend,
    Downtrend,
    Consolidation,
    Reversal,
}

fn detect_pattern(prices: &[f64]) -> Pattern {
    if prices.len() < 2 {
        return Pattern::Consolidation;
    }

    let mut ups = 0;
    let mut downs = 0;

    for i in 1..prices.len() {
        if prices[i] > prices[i - 1] {
            ups += 1;
        } else if prices[i] < prices[i - 1] {
            downs += 1;
        }
    }

    let total_moves = ups + downs;
    if total_moves == 0 {
        return Pattern::Consolidation;
    }

    let up_ratio = ups as f64 / total_moves as f64;

    // First half vs second half
    let mid = prices.len() / 2;
    let first_half_trend = prices[mid] - prices[0];
    let second_half_trend = prices[prices.len() - 1] - prices[mid];

    // Reversal: trends in different directions
    if (first_half_trend > 0.0 && second_half_trend < 0.0) ||
       (first_half_trend < 0.0 && second_half_trend > 0.0) {
        return Pattern::Reversal;
    }

    if up_ratio >= 0.7 {
        Pattern::Uptrend
    } else if up_ratio <= 0.3 {
        Pattern::Downtrend
    } else {
        Pattern::Consolidation
    }
}
```

## Mutable Slices

Slices can be mutable for data modification:

```rust
fn main() {
    let mut prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    println!("Before: {:?}", prices);

    // Mutable slice
    let slice = &mut prices[1..4];

    // Apply 1% commission to part of the prices
    apply_commission(slice, 0.01);

    println!("After commission: {:?}", prices);
}

fn apply_commission(prices: &mut [f64], rate: f64) {
    for price in prices.iter_mut() {
        *price *= 1.0 - rate;
    }
}
```

## Useful Slice Methods

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let slice = &prices[..];

    // Basic methods
    println!("len: {}", slice.len());
    println!("is_empty: {}", slice.is_empty());
    println!("first: {:?}", slice.first());
    println!("last: {:?}", slice.last());

    // Splitting a slice
    let (left, right) = slice.split_at(2);
    println!("Left: {:?}", left);
    println!("Right: {:?}", right);

    // Windows
    println!("\nWindows of 3:");
    for window in slice.windows(3) {
        println!("  {:?}", window);
    }

    // Chunks
    println!("\nChunks of 2:");
    for chunk in slice.chunks(2) {
        println!("  {:?}", chunk);
    }

    // Iterators
    let sum: f64 = slice.iter().sum();
    let count = slice.iter().filter(|&&p| p > 42000.0).count();

    println!("\nSum: {}", sum);
    println!("Count > 42000: {}", count);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&[T]` | Immutable slice |
| `&mut [T]` | Mutable slice |
| `&arr[a..b]` | Slice from a to b (not including b) |
| `&arr[..b]` | From beginning to b |
| `&arr[a..]` | From a to end |
| `&arr[a..=b]` | From a to b (including b) |
| `.windows(n)` | Sliding windows of size n |
| `.chunks(n)` | Split into chunks of size n |
| `.split_at(n)` | Split into two parts |

## Homework

1. **Bollinger Bands**: Implement Bollinger Bands calculation (SMA ± 2*standard deviation) using slices for the sliding window.

2. **Period Comparison**: Write a function that takes a price array and compares the first half with the second (average price, volatility, growth).

3. **Finding Local Extrema**: Create a function that finds local maxima and minima in a price array using a window of 3 elements.

4. **Hourly Volume Analysis**: Given an array of 168 volume values (a week by hours), find:
   - The most active day of the week
   - The most active hour
   - Compare weekend and weekday volumes

## Navigation

[← Previous day](../051-reference-practice/en.md) | [Next day →](../053-pattern-data-in-out/en.md)
