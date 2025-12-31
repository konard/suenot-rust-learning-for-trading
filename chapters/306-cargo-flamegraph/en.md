# Day 306: cargo flamegraph: Performance Visualization

## Trading Analogy

Imagine you're running a high-frequency trading system that processes thousands of market orders per second. Everything works fine during testing, but in production, you notice occasional delays that cause you to miss profitable opportunities. Where is the bottleneck?

It's like trying to find the slowest trader in a large trading floor during peak hours — you need to see who spends the most time on each operation. **cargo flamegraph** is like a heat map of your trading floor showing exactly where time is being spent.

A flamegraph is a visualization where:
- **Width** represents how much time is spent in a function (wider = more time)
- **Height** shows the call stack (how deeply functions are nested)
- **Hot colors** (red, orange) highlight the most time-consuming operations

Just as you'd identify that one trader who manually processes orders instead of using automated tools, flamegraph helps you spot the slow functions in your code that need optimization.

## Why is Performance Profiling Critical in Trading?

In algorithmic trading, milliseconds matter:

| Problem | Impact | Solution |
|---------|--------|----------|
| Slow order processing | Missed arbitrage opportunities | Profile and optimize hot paths |
| Inefficient price calculations | Delayed trading signals | Identify computational bottlenecks |
| Memory allocations in loops | Increased latency | Find allocation hotspots |
| Redundant data parsing | Wasted CPU cycles | Visualize unnecessary work |
| Blocking I/O operations | System-wide slowdowns | Detect blocking calls |

## What is cargo flamegraph?

`cargo flamegraph` is a Rust tool that generates flame graphs showing where your program spends time during execution. It:

1. **Profiles** your application using system tools (perf on Linux, DTrace on macOS)
2. **Samples** the call stack hundreds of times per second
3. **Generates** an interactive SVG visualization
4. **Shows** which functions consume the most CPU time

### Installation

```bash
# Install the tool
cargo install flamegraph

# On Linux, you may need to adjust perf permissions
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

### Basic Usage

```bash
# Profile your application
cargo flamegraph

# Profile release build (recommended)
cargo flamegraph --release

# Profile with specific arguments
cargo flamegraph --release -- --data-file market_data.csv

# Profile a specific binary
cargo flamegraph --bin trading_engine
```

## Example 1: Finding Bottlenecks in Price Calculation

Let's create a trading system that calculates various price metrics and use flamegraph to find performance issues.

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
struct PriceData {
    timestamp: u64,
    price: f64,
    volume: f64,
}

/// Calculate Simple Moving Average (slow version)
fn calculate_sma_slow(prices: &[f64], window: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in window..=prices.len() {
        let sum: f64 = prices[i - window..i].iter().sum();
        result.push(sum / window as f64);
    }

    result
}

/// Calculate Exponential Moving Average (slow version)
fn calculate_ema_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // First EMA is SMA
    let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);

    for i in period..prices.len() {
        let ema = (prices[i] - result.last().unwrap()) * multiplier + result.last().unwrap();
        result.push(ema);
    }

    result
}

/// Calculate RSI (slow version with repeated allocations)
fn calculate_rsi_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsi_values = Vec::new();

    for i in period..prices.len() {
        let mut gains = Vec::new();  // Allocation in hot loop!
        let mut losses = Vec::new(); // Allocation in hot loop!

        for j in i - period + 1..=i {
            let change = prices[j] - prices[j - 1];
            if change > 0.0 {
                gains.push(change);
            } else {
                losses.push(change.abs());
            }
        }

        let avg_gain = if !gains.is_empty() {
            gains.iter().sum::<f64>() / gains.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };

        rsi_values.push(rsi);
    }

    rsi_values
}

/// Process market data with multiple indicators
fn analyze_market(prices: &[f64]) {
    let start = Instant::now();

    println!("Calculating indicators for {} price points...", prices.len());

    // These calculations will show up in flamegraph
    let sma_20 = calculate_sma_slow(prices, 20);
    let sma_50 = calculate_sma_slow(prices, 50);
    let ema_12 = calculate_ema_slow(prices, 12);
    let ema_26 = calculate_ema_slow(prices, 26);
    let rsi_14 = calculate_rsi_slow(prices, 14);

    println!("SMA(20): {} values", sma_20.len());
    println!("SMA(50): {} values", sma_50.len());
    println!("EMA(12): {} values", ema_12.len());
    println!("EMA(26): {} values", ema_26.len());
    println!("RSI(14): {} values", rsi_14.len());

    println!("Analysis completed in {:?}", start.elapsed());
}

/// Generate test price data
fn generate_price_data(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 50000.0;

    for i in 0..count {
        // Simulate price movement
        let change = ((i * 7) % 100) as f64 - 50.0;
        price += change;
        prices.push(price);
    }

    prices
}

fn main() {
    // Generate a large dataset to make profiling visible
    let prices = generate_price_data(10000);

    // Run analysis multiple times to get clear profiling data
    for iteration in 1..=5 {
        println!("\n=== Iteration {} ===", iteration);
        analyze_market(&prices);
    }
}
```

**Running with flamegraph:**

```bash
# Save the code to src/main.rs
cargo flamegraph --release

# This will:
# 1. Build your code in release mode
# 2. Run it while profiling
# 3. Generate flamegraph.svg in your project root
# 4. Open it in your browser
```

**What you'll see in the flamegraph:**
- `calculate_rsi_slow` will likely be the widest (most time spent)
- You'll see repeated allocations in the hot loop
- `calculate_sma_slow` may show inefficient iteration patterns

## Example 2: Optimized Version After Profiling

After analyzing the flamegraph, we can optimize the hot paths:

```rust
/// Calculate SMA (optimized version)
fn calculate_sma_optimized(prices: &[f64], window: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len() - window + 1);

    if prices.len() < window {
        return result;
    }

    // Calculate first SMA
    let mut sum: f64 = prices[..window].iter().sum();
    result.push(sum / window as f64);

    // Use sliding window: remove oldest, add newest
    for i in window..prices.len() {
        sum = sum - prices[i - window] + prices[i];
        result.push(sum / window as f64);
    }

    result
}

/// Calculate RSI (optimized version - pre-allocated buffers)
fn calculate_rsi_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsi_values = Vec::with_capacity(prices.len() - period);

    // Pre-calculate all price changes
    let mut changes: Vec<f64> = Vec::with_capacity(prices.len() - 1);
    for i in 1..prices.len() {
        changes.push(prices[i] - prices[i - 1]);
    }

    // Use running averages instead of recalculating
    for i in period - 1..changes.len() {
        let window = &changes[i - period + 1..=i];

        let (gain_sum, loss_sum) = window.iter().fold((0.0, 0.0), |(g, l), &change| {
            if change > 0.0 {
                (g + change, l)
            } else {
                (g, l + change.abs())
            }
        });

        let avg_gain = gain_sum / period as f64;
        let avg_loss = loss_sum / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };

        rsi_values.push(rsi);
    }

    rsi_values
}

/// Benchmark comparison
fn compare_performance(prices: &[f64]) {
    use std::time::Instant;

    println!("=== Performance Comparison ===\n");

    // Test slow version
    let start = Instant::now();
    let _rsi_slow = calculate_rsi_slow(prices, 14);
    let slow_time = start.elapsed();
    println!("RSI (slow):      {:?}", slow_time);

    // Test optimized version
    let start = Instant::now();
    let _rsi_fast = calculate_rsi_optimized(prices, 14);
    let fast_time = start.elapsed();
    println!("RSI (optimized): {:?}", fast_time);

    let speedup = slow_time.as_secs_f64() / fast_time.as_secs_f64();
    println!("\nSpeedup: {:.2}x faster", speedup);
}

fn main() {
    let prices = generate_price_data(10000);
    compare_performance(&prices);
}
```

## Example 3: Profiling Order Matching Engine

Let's profile a simple order matching system:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

struct OrderBook {
    bids: BTreeMap<u64, Order>,  // Buy orders (price as key * 100)
    asks: BTreeMap<u64, Order>,  // Sell orders (price as key * 100)
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Add order and try to match
    fn add_order(&mut self, order: Order) -> Vec<(u64, u64, f64, f64)> {
        let mut matches = Vec::new();

        match order.side {
            OrderSide::Buy => {
                // Try to match with asks
                let price_key = (order.price * 100.0) as u64;

                // Find matching sell orders
                let matching_asks: Vec<_> = self.asks
                    .iter()
                    .filter(|(_, ask)| ask.price <= order.price)
                    .map(|(_, ask)| ask.clone())
                    .collect();

                for ask in matching_asks {
                    let ask_key = (ask.price * 100.0) as u64;
                    matches.push((order.id, ask.id, ask.price, ask.quantity.min(order.quantity)));
                    self.asks.remove(&ask_key);
                }

                // Add to order book if not fully matched
                if matches.is_empty() {
                    self.bids.insert(price_key, order);
                }
            }
            OrderSide::Sell => {
                // Similar logic for sell orders
                let price_key = (order.price * 100.0) as u64;

                let matching_bids: Vec<_> = self.bids
                    .iter()
                    .filter(|(_, bid)| bid.price >= order.price)
                    .map(|(_, bid)| bid.clone())
                    .collect();

                for bid in matching_bids {
                    let bid_key = (bid.price * 100.0) as u64;
                    matches.push((order.id, bid.id, bid.price, bid.quantity.min(order.quantity)));
                    self.bids.remove(&bid_key);
                }

                if matches.is_empty() {
                    self.asks.insert(price_key, order);
                }
            }
        }

        matches
    }
}

fn simulate_trading_day() {
    let mut order_book = OrderBook::new();
    let mut order_id = 1u64;

    // Simulate 100,000 orders
    for i in 0..100_000 {
        let side = if i % 2 == 0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let base_price = 50000.0;
        let price_variation = ((i * 7) % 100) as f64 - 50.0;
        let price = base_price + price_variation;

        let order = Order {
            id: order_id,
            side,
            price,
            quantity: 0.1,
        };

        let matches = order_book.add_order(order);

        if !matches.is_empty() && i % 10000 == 0 {
            println!("Order {} matched {} orders", order_id, matches.len());
        }

        order_id += 1;
    }

    println!("Simulation complete!");
    println!("Remaining bids: {}", order_book.bids.len());
    println!("Remaining asks: {}", order_book.asks.len());
}

fn main() {
    println!("Starting order matching simulation...\n");
    simulate_trading_day();
}
```

## Reading a Flamegraph

When you open `flamegraph.svg`, look for:

### 1. **Wide Bars** = Hot Functions
The wider a bar, the more CPU time that function consumed:
```
[────── calculate_rsi_slow ──────]  ← This is slow!
[─ calculate_sma ─]                 ← This is faster
```

### 2. **Tall Stacks** = Deep Call Chains
Height shows call depth. Very tall stacks might indicate:
- Recursive functions
- Deep abstraction layers
- Potential for inlining

### 3. **Colors** = Different Modules/Crates
- Your code vs library code
- Different modules within your crate
- Standard library functions

### 4. **Tooltip Information**
Hover over a bar to see:
- Function name
- Percentage of total time
- Absolute time spent
- Number of samples

## Common Performance Issues Found

| Pattern in Flamegraph | Problem | Solution |
|----------------------|---------|----------|
| Wide allocation bars | Too many heap allocations | Pre-allocate or use stack |
| Repeated small functions | Lack of inlining | Use `#[inline]` or LTO |
| Large clone operations | Unnecessary copying | Use references or `Cow` |
| Format/parse functions | String conversions | Cache or use binary formats |
| Lock contention | Mutex blocking | Use lock-free structures or finer locking |

## Practical Tips

### 1. Always Profile Release Builds
```bash
cargo flamegraph --release
```
Debug builds include lots of extra checks that skew results.

### 2. Enable Debug Symbols in Release
Add to `Cargo.toml`:
```toml
[profile.release]
debug = true  # Enable debug symbols for profiling
```

### 3. Run Long Enough to Get Good Data
Short runs may not provide representative samples:
```rust
// Run operations multiple times
for _ in 0..1000 {
    analyze_market(&prices);
}
```

### 4. Profile Real Workloads
Use actual market data or realistic simulations:
```bash
cargo flamegraph --release -- --data-file real_market_data.csv
```

### 5. Compare Before/After
Generate flamegraphs before and after optimization:
```bash
# Before
cargo flamegraph --release -o flamegraph_before.svg

# After optimization
cargo flamegraph --release -o flamegraph_after.svg
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Flamegraph** | Visual representation of CPU time usage |
| **Sampling** | Periodic capture of call stacks during execution |
| **Hot Path** | Code sections that consume the most time |
| **Profiling** | Measuring where a program spends time |
| **Call Stack** | Chain of function calls at any point |
| **cargo flamegraph** | Tool for generating flamegraphs in Rust |
| **Bottleneck** | Slowest part that limits overall performance |

## Homework

1. **Profile Your Own Code**: Take any previous trading example and:
   - Add `cargo flamegraph` profiling
   - Identify the slowest function
   - Generate a flamegraph report showing before/after optimization

2. **Order Book Optimization**: Profile the order matching example:
   - Find the hottest function in the matching logic
   - Optimize it (consider using better data structures)
   - Measure the speedup with benchmarks

3. **Multi-Indicator Dashboard**: Create a system that calculates 10+ indicators:
   - SMA with 5 different periods
   - EMA with 3 different periods
   - RSI, MACD, Bollinger Bands
   - Profile to find which indicator is slowest
   - Optimize the top 3 bottlenecks

4. **Flamegraph Analysis Report**: Write a tool that:
   - Runs cargo flamegraph automatically
   - Parses the generated data
   - Identifies functions taking > 10% of time
   - Generates a text report with optimization suggestions

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
