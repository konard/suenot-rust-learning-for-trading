# Day 311: Iterators vs Loops: What's Faster

## Trading Analogy

Imagine two traders analyzing thousands of trades:

**Trader with loops** (classical approach):
- Takes an Excel file with trade history
- Manually iterates through each row
- Calculates profit for each trade
- Records results in a new column
- Then iterates through all rows again to calculate average profit
- And once more to filter losing trades

**Trader with iterators** (functional approach):
- Loads data once
- Builds a chain of operations: "calculate profit → filter profitable → calculate average"
- Computer optimizes the entire chain and executes in one pass
- No intermediate data copies
- Compiler can apply additional optimizations

In Rust, iterators don't just make code cleaner and more expressive — they're often **faster** than hand-written loops!

## Why can iterators be faster?

### 1. Zero-cost abstractions

Rust compiles iterators into machine code as efficient as loops (sometimes even better):

```rust
// for loop
let mut sum = 0;
for price in prices.iter() {
    sum += price;
}

// Iterator
let sum: i32 = prices.iter().sum();
```

Both compile to **identical** machine code! But the iterator:
- Is shorter and clearer
- Is protected from errors (no indices)
- Gives the compiler more information for optimization

### 2. Lazy evaluation

Iterators don't do work until the result is needed:

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: i32,
    profit: f64,
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), price: 42000.0, quantity: 1, profit: 500.0 },
        Trade { symbol: "ETH".to_string(), price: 2500.0, quantity: 10, profit: -200.0 },
        Trade { symbol: "BTC".to_string(), price: 43000.0, quantity: 2, profit: 1000.0 },
        Trade { symbol: "SOL".to_string(), price: 100.0, quantity: 50, profit: 300.0 },
    ];

    // Iterator creates a plan but computes nothing!
    let profitable_btc = trades.iter()
        .filter(|t| t.profit > 0.0)         // Plan: "only profitable"
        .filter(|t| t.symbol == "BTC");     // Plan: "only BTC"

    // Work is done only here, when result is needed
    let count = profitable_btc.count();
    println!("Profitable BTC trades: {}", count);
}
```

### 3. Compiler optimizations

The compiler sees the entire iterator chain and can:
- Combine multiple passes into one
- Remove array bounds checks
- Use SIMD instructions (parallel processing)
- Unroll loops

## Example 1: Calculating Average Price

### Loop version

```rust
fn average_price_loop(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let mut sum = 0.0;
    for price in prices {
        sum += price;
    }

    Some(sum / prices.len() as f64)
}

fn main() {
    let btc_prices = vec![42000.0, 43500.0, 41800.0, 44200.0, 43000.0];

    match average_price_loop(&btc_prices) {
        Some(avg) => println!("Average price (loop): ${:.2}", avg),
        None => println!("No data"),
    }
}
```

### Iterator version

```rust
fn average_price_iter(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    Some(prices.iter().sum::<f64>() / prices.len() as f64)
}

fn main() {
    let btc_prices = vec![42000.0, 43500.0, 41800.0, 44200.0, 43000.0];

    match average_price_iter(&btc_prices) {
        Some(avg) => println!("Average price (iterator): ${:.2}", avg),
        None => println!("No data"),
    }
}
```

**Performance**: Same! But the iterator is shorter and more expressive.

## Example 2: Filtering and Transformation

### Loop version

```rust
#[derive(Debug)]
struct Order {
    id: u32,
    symbol: String,
    price: f64,
    quantity: i32,
}

// Get IDs of all large BTC orders
fn get_large_btc_orders_loop(orders: &[Order], min_quantity: i32) -> Vec<u32> {
    let mut result = Vec::new();

    for order in orders {
        if order.symbol == "BTC" && order.quantity >= min_quantity {
            result.push(order.id);
        }
    }

    result
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC".to_string(), price: 42000.0, quantity: 5 },
        Order { id: 2, symbol: "ETH".to_string(), price: 2500.0, quantity: 50 },
        Order { id: 3, symbol: "BTC".to_string(), price: 43000.0, quantity: 2 },
        Order { id: 4, symbol: "BTC".to_string(), price: 41500.0, quantity: 10 },
    ];

    let large_btc = get_large_btc_orders_loop(&orders, 5);
    println!("Large BTC orders (loop): {:?}", large_btc);
}
```

### Iterator version

```rust
fn get_large_btc_orders_iter(orders: &[Order], min_quantity: i32) -> Vec<u32> {
    orders.iter()
        .filter(|o| o.symbol == "BTC")
        .filter(|o| o.quantity >= min_quantity)
        .map(|o| o.id)
        .collect()
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC".to_string(), price: 42000.0, quantity: 5 },
        Order { id: 2, symbol: "ETH".to_string(), price: 2500.0, quantity: 50 },
        Order { id: 3, symbol: "BTC".to_string(), price: 43000.0, quantity: 2 },
        Order { id: 4, symbol: "BTC".to_string(), price: 41500.0, quantity: 10 },
    ];

    let large_btc = get_large_btc_orders_iter(&orders, 5);
    println!("Large BTC orders (iterator): {:?}", large_btc);
}
```

**Performance**: Iterator makes **one pass**, so does the loop. But the iterator:
- Is more readable (declarative style)
- Is easier to modify (add another `filter`)
- Compiler can optimize better

## Example 3: Benchmark — where iterators are actually faster

```rust
use std::time::Instant;

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
    fn new(timestamp: u64, close: f64) -> Self {
        Self {
            timestamp,
            open: close,
            high: close * 1.01,
            low: close * 0.99,
            close,
            volume: 1000.0,
        }
    }
}

// Calculate percentage change from previous candle
fn calculate_returns_loop(candles: &[Candle]) -> Vec<f64> {
    let mut returns = Vec::with_capacity(candles.len() - 1);

    for i in 1..candles.len() {
        let return_pct = (candles[i].close - candles[i - 1].close) / candles[i - 1].close;
        returns.push(return_pct);
    }

    returns
}

fn calculate_returns_iter(candles: &[Candle]) -> Vec<f64> {
    candles.windows(2)
        .map(|pair| (pair[1].close - pair[0].close) / pair[0].close)
        .collect()
}

fn main() {
    // Generate data
    let candles: Vec<Candle> = (0..100_000)
        .map(|i| Candle::new(i, 42000.0 + (i as f64 * 0.5)))
        .collect();

    // Benchmark loop
    let start = Instant::now();
    let returns_loop = calculate_returns_loop(&candles);
    let duration_loop = start.elapsed();

    // Benchmark iterator
    let start = Instant::now();
    let returns_iter = calculate_returns_iter(&candles);
    let duration_iter = start.elapsed();

    println!("=== Benchmark: returns calculation ===");
    println!("Number of candles: {}", candles.len());
    println!("for loop:  {:?} ({} results)", duration_loop, returns_loop.len());
    println!("Iterator:  {:?} ({} results)", duration_iter, returns_iter.len());

    if duration_iter < duration_loop {
        let speedup = duration_loop.as_nanos() as f64 / duration_iter.as_nanos() as f64;
        println!("✅ Iterator is {:.2}x faster", speedup);
    } else {
        println!("⚖️  Performance is roughly the same");
    }
}
```

## When are iterators slower?

### 1. Complex logic with early exit

```rust
// Find first losing trade with early exit
fn find_first_loss_loop(trades: &[Trade]) -> Option<&Trade> {
    for trade in trades {
        if trade.profit < 0.0 {
            return Some(trade);  // Early exit!
        }
    }
    None
}

// With iterator — also efficient thanks to .find()
fn find_first_loss_iter(trades: &[Trade]) -> Option<&Trade> {
    trades.iter().find(|t| t.profit < 0.0)
}
```

**Both are equally efficient** thanks to lazy evaluation of iterators.

### 2. Mutating state inside loop

```rust
// Count winning streaks
fn count_winning_streaks_loop(trades: &[Trade]) -> Vec<usize> {
    let mut streaks = Vec::new();
    let mut current_streak = 0;

    for trade in trades {
        if trade.profit > 0.0 {
            current_streak += 1;
        } else {
            if current_streak > 0 {
                streaks.push(current_streak);
                current_streak = 0;
            }
        }
    }

    if current_streak > 0 {
        streaks.push(current_streak);
    }

    streaks
}
```

Here the loop is **simpler and clearer** than trying to use `fold()` or `scan()`.

## Comparison: expressiveness vs performance

| Operation | for loop | Iterator | Winner |
|-----------|----------|----------|--------|
| Simple iteration | `for x in iter` | `.iter()` | 🤝 Same |
| Sum | Manual counter | `.sum()` | ✅ Iterator (shorter) |
| Filtering | `if` inside | `.filter()` | ✅ Iterator (readability) |
| Transformation | Manual `Vec::push` | `.map().collect()` | ✅ Iterator |
| Complex logic | Explicit conditions | `fold()`/`scan()` | ✅ Loop (clearer) |
| Early exit | `break`/`return` | `.find()`/`.any()` | 🤝 Same |

## Practical Example: Portfolio Analysis

```rust
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn profit(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn profit_pct(&self) -> f64 {
        (self.current_price - self.entry_price) / self.entry_price
    }
}

fn analyze_portfolio(positions: &[Position]) {
    // 1. Total profit/loss (iterator)
    let total_pnl: f64 = positions.iter()
        .map(|p| p.profit())
        .sum();

    // 2. Number of profitable positions
    let profitable_count = positions.iter()
        .filter(|p| p.profit() > 0.0)
        .count();

    // 3. Average profit percentage (profitable only)
    let avg_profit_pct = positions.iter()
        .filter(|p| p.profit() > 0.0)
        .map(|p| p.profit_pct())
        .sum::<f64>() / profitable_count.max(1) as f64;

    // 4. Worst position
    let worst_position = positions.iter()
        .min_by(|a, b| a.profit().partial_cmp(&b.profit()).unwrap());

    // 5. Symbols with loss > 10%
    let heavy_losses: Vec<&str> = positions.iter()
        .filter(|p| p.profit_pct() < -0.10)
        .map(|p| p.symbol.as_str())
        .collect();

    println!("=== Portfolio Analysis ===");
    println!("Total P&L: ${:.2}", total_pnl);
    println!("Profitable positions: {} out of {}", profitable_count, positions.len());
    println!("Average profit: {:.2}%", avg_profit_pct * 100.0);

    if let Some(worst) = worst_position {
        println!("Worst position: {} (${:.2})", worst.symbol, worst.profit());
    }

    if !heavy_losses.is_empty() {
        println!("Losses > 10%: {:?}", heavy_losses);
    }
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.0, entry_price: 40000.0, current_price: 43000.0 },
        Position { symbol: "ETH".to_string(), quantity: 50.0, entry_price: 2800.0, current_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, entry_price: 120.0, current_price: 105.0 },
        Position { symbol: "AAPL".to_string(), quantity: 10.0, entry_price: 180.0, current_price: 185.0 },
    ];

    analyze_portfolio(&portfolio);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Zero-cost abstractions** | Iterators compile to efficient machine code |
| **Lazy evaluation** | Iterators don't execute until consuming method is called |
| **Method chains** | `.filter().map().collect()` — more readable than nested loops |
| **Compiler optimizations** | Compiler can better optimize iterators |
| **Expressiveness** | Declarative style: "what to do", not "how to do" |
| **Safety** | No manual indices — no out-of-bounds errors |
| **windows()** | Iterator over sliding window — convenient for pairs of elements |
| **When to use loops** | Complex logic with mutable state |

## Homework

1. **SMA Benchmark**: Write two versions of simple moving average calculation:
   - With `for` loop
   - With iterators (use `windows()`)

   Measure performance on an array of 1 million prices.

2. **Trade Analysis**: For an array of trades, implement using iterators:
   - Sharpe ratio
   - Maximum drawdown
   - Win rate (percentage of profitable trades)
   - Average win vs average loss

3. **Approach Comparison**: Implement a `find_best_strategy()` function that:
   - Takes an array of strategies with results
   - Filters by Sharpe > 1.0 and drawdown < 20%
   - Sorts by profit
   - Returns top-3 strategies

   Implement **both ways**: with loops and with iterators. Which code is clearer?

4. **Chain Optimization**: Find places in your code where you can replace multiple loops with one iterator chain. Measure improvements in readability and performance.

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
