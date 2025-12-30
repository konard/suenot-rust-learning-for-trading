# Day 12: Arrays — Last 10 Closing Prices

## Trading Analogy

For market analysis, we need **historical data**:
- Last 10 closing prices
- 20 candles for SMA calculation
- 14 values for RSI

An array is a list of values of **the same type** with a **fixed size**.

## Creating Arrays

```rust
fn main() {
    // Last 5 closing prices
    let closes: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Array of zeros (10 elements)
    let zeros: [f64; 10] = [0.0; 10];

    // Type inferred automatically
    let volumes = [1500.0, 2300.0, 1800.0, 2100.0, 1950.0];

    println!("Closes: {:?}", closes);
    println!("Zeros: {:?}", zeros);
    println!("Volumes: {:?}", volumes);
}
```

Syntax: `[type; size]` or `[value; count]`

## Accessing Elements

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Indices start at 0
    println!("First: {}", closes[0]);   // 42000.0
    println!("Second: {}", closes[1]);  // 42100.0
    println!("Last: {}", closes[4]);    // 42150.0

    // Array length
    println!("Length: {}", closes.len());  // 5

    // RUNTIME ERROR!
    // println!("{}", closes[10]);  // panic: index out of bounds
}
```

## Safe Access with get()

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0];

    // get() returns Option
    match closes.get(1) {
        Some(price) => println!("Price at index 1: {}", price),
        None => println!("Index out of bounds"),
    }

    // Safe for any index
    match closes.get(10) {
        Some(price) => println!("Price: {}", price),
        None => println!("No price at index 10"),
    }
}
```

## Iterating Over Arrays

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Simple for
    println!("All prices:");
    for price in closes {
        println!("  ${}", price);
    }

    // With index
    println!("\nWith index:");
    for (i, price) in closes.iter().enumerate() {
        println!("  [{}] ${}", i, price);
    }

    // Only first 3
    for price in &closes[0..3] {
        println!("First 3: ${}", price);
    }
}
```

## Mutable Arrays

```rust
fn main() {
    let mut prices = [0.0; 5];

    println!("Before: {:?}", prices);

    // Fill with data
    prices[0] = 42000.0;
    prices[1] = 42100.0;
    prices[2] = 41900.0;
    prices[3] = 42200.0;
    prices[4] = 42150.0;

    println!("After: {:?}", prices);

    // Update last price
    prices[4] = 42300.0;
    println!("Updated: {:?}", prices);
}
```

## Slices

A slice is a "view" into part of an array:

```rust
fn main() {
    let prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    // Slice from index 1 to 4 (not including 4)
    let slice = &prices[1..4];
    println!("Slice [1..4]: {:?}", slice);  // [200.0, 300.0, 400.0]

    // From beginning to index 3
    let first_three = &prices[..3];
    println!("First 3: {:?}", first_three);  // [100.0, 200.0, 300.0]

    // From index 2 to end
    let last_three = &prices[2..];
    println!("Last 3: {:?}", last_three);  // [300.0, 400.0, 500.0]

    // Entire array as slice
    let all = &prices[..];
    println!("All: {:?}", all);
}
```

## Practical Example: SMA Calculation

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // SMA-5 (simple moving average for 5 periods)
    let sma5 = calculate_sma(&closes[5..]);  // Last 5
    println!("SMA-5 (last 5): {:.2}", sma5);

    // SMA for different windows
    let sma3 = calculate_sma(&closes[7..]);
    let sma10 = calculate_sma(&closes);

    println!("SMA-3: {:.2}", sma3);
    println!("SMA-10: {:.2}", sma10);
}

fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    let sum: f64 = prices.iter().sum();
    sum / prices.len() as f64
}
```

## Practical Example: Finding Min/Max

```rust
fn main() {
    let daily_highs = [42500.0, 42800.0, 42300.0, 43100.0, 42900.0];
    let daily_lows = [41800.0, 42100.0, 41500.0, 42400.0, 42200.0];

    // Find extremes
    let highest = find_max(&daily_highs);
    let lowest = find_min(&daily_lows);

    println!("Weekly High: ${:.2}", highest);
    println!("Weekly Low: ${:.2}", lowest);
    println!("Weekly Range: ${:.2}", highest - lowest);
}

fn find_max(prices: &[f64]) -> f64 {
    let mut max = prices[0];
    for &price in prices {
        if price > max {
            max = price;
        }
    }
    max
}

fn find_min(prices: &[f64]) -> f64 {
    let mut min = prices[0];
    for &price in prices {
        if price < min {
            min = price;
        }
    }
    min
}
```

## Practical Example: Returns

```rust
fn main() {
    let prices = [42000.0, 42500.0, 42200.0, 42800.0, 43000.0];

    // Calculate daily returns
    let mut returns: [f64; 4] = [0.0; 4];

    for i in 1..prices.len() {
        returns[i - 1] = (prices[i] - prices[i - 1]) / prices[i - 1] * 100.0;
    }

    println!("Prices: {:?}", prices);
    println!("Daily returns (%):");
    for (i, ret) in returns.iter().enumerate() {
        let sign = if *ret >= 0.0 { "+" } else { "" };
        println!("  Day {}: {}{:.2}%", i + 1, sign, ret);
    }

    // Total return
    let total_return = (prices[4] - prices[0]) / prices[0] * 100.0;
    println!("\nTotal return: {:.2}%", total_return);
}
```

## Two-Dimensional Arrays

```rust
fn main() {
    // OHLC data for 3 days
    // [day][value]: O, H, L, C
    let ohlc: [[f64; 4]; 3] = [
        [42000.0, 42500.0, 41800.0, 42200.0],  // Day 1
        [42200.0, 42800.0, 42100.0, 42600.0],  // Day 2
        [42600.0, 43000.0, 42400.0, 42900.0],  // Day 3
    ];

    println!("=== OHLC Data ===");
    for (day, candle) in ohlc.iter().enumerate() {
        println!("Day {}: O={}, H={}, L={}, C={}",
            day + 1, candle[0], candle[1], candle[2], candle[3]);
    }

    // Average close price
    let avg_close = (ohlc[0][3] + ohlc[1][3] + ohlc[2][3]) / 3.0;
    println!("\nAverage Close: {:.2}", avg_close);
}
```

## Useful Methods

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Checks
    println!("Is empty: {}", prices.is_empty());
    println!("Length: {}", prices.len());

    // First and last
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // Contains value
    println!("Contains 42000: {}", prices.contains(&42000.0));

    // Iterators
    let sum: f64 = prices.iter().sum();
    let max = prices.iter().cloned().fold(f64::MIN, f64::max);
    let min = prices.iter().cloned().fold(f64::MAX, f64::min);

    println!("Sum: {}", sum);
    println!("Max: {}", max);
    println!("Min: {}", min);
}
```

## Sorting

```rust
fn main() {
    let mut prices = [42500.0, 41800.0, 42200.0, 42000.0, 42100.0];

    println!("Original: {:?}", prices);

    // Sorting (for f64 needs special approach)
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("Sorted: {:?}", prices);

    // Reverse sort
    prices.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("Reversed: {:?}", prices);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `[T; N]` | Array of N elements of type T |
| `arr[i]` | Index access |
| `&arr[a..b]` | Slice |
| `arr.len()` | Array length |
| `arr.iter()` | Iterator |

## Homework

1. Create an array of 20 random prices and calculate SMA-5, SMA-10, SMA-20

2. Implement a function to find volatility (standard deviation) for a price array

3. Create a 2D OHLCV array for 5 days and find:
   - Day with maximum volume
   - Day with biggest candle (high - low)

4. Write a function that finds the crossover of two SMA arrays

## Navigation

[← Previous day](../011-tuples-bid-ask/en.md) | [Next day →](../013-functions-trade-profit/en.md)
