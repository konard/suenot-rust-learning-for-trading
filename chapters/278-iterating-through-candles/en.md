# Day 278: Iterating Through Candles

## Trading Analogy

Imagine you're analyzing a Bitcoin chart over the last year. You have 365 daily candles, and you need to go through each one to find patterns, calculate indicators, or test a trading strategy. This is **iterating through candles** — sequentially traversing each data element.

In real trading, iteration is used everywhere:
- Backtesting strategies — walking through historical data
- Calculating moving averages — summing the last N candles
- Finding extremes — locating maximum/minimum prices
- Volume analysis — counting total volume over a period

## What is an Iterator in Rust?

An iterator is an object that allows you to sequentially traverse elements of a collection. In Rust, iterators are:

1. **Lazy** — they don't perform computations until results are needed
2. **Safe** — the compiler checks bounds at compile time
3. **Efficient** — often optimized to the level of manual loops
4. **Composable** — you can chain multiple operations together

```rust
// Simple iteration example
let prices = vec![42000.0, 42500.0, 41800.0, 43000.0];

for price in prices.iter() {
    println!("Price: {}", price);
}
```

## Candle Structure

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,      // Unix timestamp
    open: f64,           // Opening price
    high: f64,           // Highest price
    low: f64,            // Lowest price
    close: f64,          // Closing price
    volume: f64,         // Trading volume
}

impl Candle {
    fn new(timestamp: u64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Candle { timestamp, open, high, low, close, volume }
    }

    // Is the candle bullish (green)?
    fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    // Is the candle bearish (red)?
    fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    // Size of candle body
    fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    // Full range of the candle
    fn range(&self) -> f64 {
        self.high - self.low
    }
}
```

## Core Iteration Methods

### 1. iter() — Immutable Iteration

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Count bullish candles
    let mut bullish_count = 0;
    for candle in candles.iter() {
        if candle.is_bullish() {
            bullish_count += 1;
        }
    }
    println!("Bullish candles: {}", bullish_count);

    // candles is still available!
    println!("Total candles: {}", candles.len());
}
```

### 2. iter_mut() — Mutable Iteration

```rust
fn main() {
    let mut candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
    ];

    // Adjust volume by a factor
    let adjustment_factor = 1.1;
    for candle in candles.iter_mut() {
        candle.volume *= adjustment_factor;
    }

    for candle in candles.iter() {
        println!("Adjusted volume: {:.2}", candle.volume);
    }
}
```

### 3. into_iter() — Consuming Iteration

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
    ];

    // Consume the vector
    let closes: Vec<f64> = candles.into_iter()
        .map(|c| c.close)
        .collect();

    println!("Closing prices: {:?}", closes);

    // candles is no longer available — ownership was transferred!
    // println!("{:?}", candles); // Compilation error!
}
```

## Iterator Adapters for Trading

### map() — Transforming Data

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Extract only closing prices
    let closes: Vec<f64> = candles.iter()
        .map(|c| c.close)
        .collect();

    println!("Closing prices: {:?}", closes);

    // Calculate typical price
    let typical_prices: Vec<f64> = candles.iter()
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();

    println!("Typical prices: {:?}", typical_prices);
}
```

### filter() — Filtering Candles

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
    ];

    // Only bullish candles
    let bullish: Vec<&Candle> = candles.iter()
        .filter(|c| c.is_bullish())
        .collect();

    println!("Bullish candles: {}", bullish.len());

    // High volume candles
    let high_volume: Vec<&Candle> = candles.iter()
        .filter(|c| c.volume > 130.0)
        .collect();

    println!("High volume candles: {}", high_volume.len());
}
```

### enumerate() — Iteration with Index

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    for (index, candle) in candles.iter().enumerate() {
        println!(
            "Candle {}: Open={}, Close={}, {}",
            index + 1,
            candle.open,
            candle.close,
            if candle.is_bullish() { "Bullish" } else { "Bearish" }
        );
    }
}
```

### windows() — Sliding Window

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
        Candle::new(5, 42350.0, 42600.0, 42000.0, 42500.0, 180.0),
    ];

    // Simple Moving Average (SMA) with period 3
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    println!("SMA(3):");
    for (i, window) in closes.windows(3).enumerate() {
        let sma: f64 = window.iter().sum::<f64>() / window.len() as f64;
        println!("  Period {}-{}: {:.2}", i + 1, i + 3, sma);
    }
}
```

### zip() — Combining Data

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Compare current candle with previous
    for (prev, curr) in candles.iter().zip(candles.iter().skip(1)) {
        let change = ((curr.close - prev.close) / prev.close) * 100.0;
        println!(
            "Candle {} -> {}: change {:.2}%",
            prev.timestamp, curr.timestamp, change
        );
    }
}
```

## Consuming Adapters

### sum() and product()

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Total volume
    let total_volume: f64 = candles.iter()
        .map(|c| c.volume)
        .sum();

    println!("Total volume: {}", total_volume);

    // Average volume
    let avg_volume = total_volume / candles.len() as f64;
    println!("Average volume: {:.2}", avg_volume);
}
```

### fold() — Accumulation

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Find the minimum and maximum price for the period
    let (min_low, max_high) = candles.iter().fold(
        (f64::MAX, f64::MIN),
        |(min, max), candle| {
            (min.min(candle.low), max.max(candle.high))
        }
    );

    println!("Period range: Low={}, High={}", min_low, max_high);
}
```

### find() and position()

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
    ];

    // Find first candle with volume greater than 180
    if let Some(candle) = candles.iter().find(|c| c.volume > 180.0) {
        println!("Found high volume candle: timestamp={}", candle.timestamp);
    }

    // Find position of first bearish candle
    if let Some(pos) = candles.iter().position(|c| c.is_bearish()) {
        println!("First bearish candle at position: {}", pos);
    }
}
```

## Practical Example: Backtesting a Simple Strategy

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn new(timestamp: u64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Candle { timestamp, open, high, low, close, volume }
    }

    fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit: f64,
}

fn calculate_sma(closes: &[f64], period: usize) -> Vec<f64> {
    closes
        .windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

fn backtest_sma_crossover(candles: &[Candle], fast_period: usize, slow_period: usize) -> Vec<Trade> {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let fast_sma = calculate_sma(&closes, fast_period);
    let slow_sma = calculate_sma(&closes, slow_period);

    // Align arrays (slow_sma is shorter)
    let offset = slow_period - fast_period;
    let fast_sma: Vec<f64> = fast_sma.iter().skip(offset).cloned().collect();

    let mut trades = Vec::new();
    let mut position: Option<f64> = None;

    for (i, (fast, slow)) in fast_sma.iter().zip(slow_sma.iter()).enumerate() {
        let prev_fast = if i > 0 { fast_sma[i - 1] } else { *fast };
        let prev_slow = if i > 0 { slow_sma[i - 1] } else { *slow };

        // Crossover from below — buy
        if prev_fast <= prev_slow && fast > slow && position.is_none() {
            let price = closes[i + slow_period - 1];
            position = Some(price);
            println!("Buy at price: {:.2}", price);
        }

        // Crossover from above — sell
        if prev_fast >= prev_slow && fast < slow && position.is_some() {
            let entry_price = position.unwrap();
            let exit_price = closes[i + slow_period - 1];
            let profit = exit_price - entry_price;

            trades.push(Trade {
                entry_price,
                exit_price,
                profit,
            });

            println!("Sell at price: {:.2}, Profit: {:.2}", exit_price, profit);
            position = None;
        }
    }

    trades
}

fn main() {
    // Generate test data
    let candles: Vec<Candle> = (0..50)
        .map(|i| {
            let base = 42000.0 + (i as f64 * 50.0).sin() * 1000.0;
            Candle::new(
                i,
                base,
                base + 100.0,
                base - 100.0,
                base + 50.0 * (i as f64 * 0.3).cos(),
                100.0 + (i as f64 * 10.0) % 50.0,
            )
        })
        .collect();

    println!("=== Backtesting SMA Crossover ===\n");

    let trades = backtest_sma_crossover(&candles, 5, 10);

    println!("\n=== Results ===");
    println!("Number of trades: {}", trades.len());

    if !trades.is_empty() {
        let total_profit: f64 = trades.iter().map(|t| t.profit).sum();
        let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = (winning_trades as f64 / trades.len() as f64) * 100.0;

        println!("Total profit: {:.2}", total_profit);
        println!("Winning trades: {} ({:.1}%)", winning_trades, win_rate);
    }
}
```

## Practical Exercises

### Exercise 1: Pattern Detection

Implement a function that finds the "Three White Soldiers" pattern (three consecutive bullish candles):

```rust
fn find_three_white_soldiers(candles: &[Candle]) -> Vec<usize> {
    // Your code here
    // Return indices of each pattern's start
    todo!()
}
```

### Exercise 2: ATR Calculation

Implement Average True Range (ATR) calculation:

```rust
fn calculate_atr(candles: &[Candle], period: usize) -> Vec<f64> {
    // True Range = max(high - low, |high - prev_close|, |low - prev_close|)
    // ATR = SMA(True Range, period)
    todo!()
}
```

### Exercise 3: Anomaly Detector

Find candles with abnormally high volume (more than 2 standard deviations from the mean):

```rust
fn find_volume_anomalies(candles: &[Candle]) -> Vec<&Candle> {
    // Your code here
    todo!()
}
```

### Exercise 4: Daily Aggregation

Convert hourly candles to daily candles:

```rust
fn aggregate_to_daily(hourly_candles: &[Candle]) -> Vec<Candle> {
    // Group by 24 hours
    // Open = first open, Close = last close
    // High = max(highs), Low = min(lows)
    // Volume = sum(volumes)
    todo!()
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `iter()` | Immutable iteration, retains ownership |
| `iter_mut()` | Mutable iteration |
| `into_iter()` | Consuming iteration, transfers ownership |
| `map()` | Transform each element |
| `filter()` | Filter by condition |
| `enumerate()` | Add index to elements |
| `windows()` | Sliding window for calculating indicators |
| `zip()` | Combine two iterators |
| `fold()` | Accumulate with initial value |
| `find()` | Find first element matching a condition |

## Homework

1. **RSI Calculator**: Implement Relative Strength Index (RSI) calculation using iterators. RSI = 100 - (100 / (1 + RS)), where RS = average gain / average loss over the period.

2. **Bollinger Bands**: Implement Bollinger Bands calculation:
   - Middle band = SMA(close, 20)
   - Upper band = SMA + 2 * standard deviation
   - Lower band = SMA - 2 * standard deviation

3. **Pattern Scanner**: Create a scanner that finds various candlestick patterns (hammer, doji, engulfing) using iterator chains.

4. **Performance Analyzer**: Write a trading strategy performance analyzer that calculates:
   - Maximum Drawdown (Max Drawdown)
   - Sharpe Ratio
   - Profit Factor (sum of profits / sum of losses)

## Navigation

[← Previous day](../277-backtesting-data-structures/en.md) | [Next day →](../279-candle-pattern-detection/en.md)
