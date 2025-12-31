# Day 307: Benchmarks: criterion Crate

## Trading Analogy

Imagine a trader testing two order execution algorithms:
- **Algorithm A**: Submits orders directly to the exchange
- **Algorithm B**: Uses smart order routing with multiple exchanges

Both seem to work, but which is faster? The trader needs precise measurements:
- How long does each order take to execute?
- Which algorithm is more consistent?
- Does performance degrade with higher order volumes?

Simply running each algorithm once isn't enough — network latency varies, exchange responses fluctuate, and random factors introduce noise. You need **statistical benchmarking** to get reliable measurements.

This is exactly what the **criterion** crate does for Rust code: it runs your functions thousands of times, collects statistical data, analyzes performance, detects regressions, and generates detailed reports with graphs.

## Why Do We Need Benchmarks?

In algorithmic trading, performance is critical:

| Scenario | Why Benchmarking Matters |
|----------|-------------------------|
| **Order execution** | Microseconds can mean the difference between profit and loss |
| **Price calculations** | Processing thousands of ticks per second requires optimization |
| **Strategy backtesting** | Faster backtests = more iterations = better strategies |
| **Risk checks** | Position limits must be verified in real-time |
| **Data processing** | Market data streams need efficient parsing |

### What criterion Provides

```rust
// Without criterion: manual timing (unreliable)
let start = std::time::Instant::now();
calculate_order_price(&order);
println!("Took: {:?}", start.elapsed());
// Problem: Single measurement, affected by noise, no statistical analysis

// With criterion: professional benchmarking
c.bench_function("order_price_calculation", |b| {
    b.iter(|| calculate_order_price(black_box(&order)))
});
// Benefits: Statistical analysis, regression detection, HTML reports
```

## Setting Up criterion

### 1. Add to Cargo.toml

```toml
[dev-dependencies]
criterion = { version = "0.7", features = ["html_reports"] }

[[bench]]
name = "trading_benchmarks"
harness = false
```

### 2. Create Benchmark File

Create `benches/trading_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Simple order structure
#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

// Function to benchmark: calculate total order value
fn calculate_order_value(order: &Order) -> f64 {
    order.quantity * order.price
}

// Function to benchmark: calculate order value with commission
fn calculate_order_value_with_commission(order: &Order, commission_rate: f64) -> f64 {
    let base_value = order.quantity * order.price;
    base_value * (1.0 + commission_rate)
}

// Basic benchmark
fn bench_order_value(c: &mut Criterion) {
    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.5,
        price: 42000.0,
    };

    c.bench_function("calculate_order_value", |b| {
        b.iter(|| calculate_order_value(black_box(&order)))
    });
}

// Benchmark with different inputs
fn bench_order_value_with_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_value_by_size");

    for quantity in [0.1, 1.0, 10.0, 100.0].iter() {
        let order = Order {
            symbol: "BTCUSDT".to_string(),
            quantity: *quantity,
            price: 42000.0,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(quantity),
            &order,
            |b, order| {
                b.iter(|| calculate_order_value(black_box(order)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_order_value, bench_order_value_with_sizes);
criterion_main!(benches);
```

### 3. Run Benchmarks

```bash
cargo bench
```

Output:
```
calculate_order_value  time:   [1.2345 ns 1.2567 ns 1.2789 ns]
order_value_by_size/0.1 time:   [1.2234 ns 1.2456 ns 1.2678 ns]
order_value_by_size/1   time:   [1.2345 ns 1.2567 ns 1.2789 ns]
order_value_by_size/10  time:   [1.2456 ns 1.2678 ns 1.2900 ns]
order_value_by_size/100 time:   [1.2567 ns 1.2789 ns 1.3011 ns]
```

## Real-World Example: Position Calculation Methods

Let's benchmark different approaches to calculating portfolio positions:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    average_price: f64,
}

// Method 1: Using Vec (simple iteration)
fn calculate_total_value_vec(positions: &Vec<Position>) -> f64 {
    positions.iter()
        .map(|p| p.quantity * p.average_price)
        .sum()
}

// Method 2: Using HashMap (faster lookups)
fn calculate_total_value_hashmap(positions: &HashMap<String, Position>) -> f64 {
    positions.values()
        .map(|p| p.quantity * p.average_price)
        .sum()
}

// Method 3: Pre-calculated cache
struct PortfolioCache {
    positions: Vec<Position>,
    cached_total: f64,
    dirty: bool,
}

impl PortfolioCache {
    fn new(positions: Vec<Position>) -> Self {
        Self {
            positions,
            cached_total: 0.0,
            dirty: true,
        }
    }

    fn get_total_value(&mut self) -> f64 {
        if self.dirty {
            self.cached_total = self.positions.iter()
                .map(|p| p.quantity * p.average_price)
                .sum();
            self.dirty = false;
        }
        self.cached_total
    }
}

fn bench_position_calculations(c: &mut Criterion) {
    // Create test data
    let positions_vec: Vec<Position> = (0..100)
        .map(|i| Position {
            symbol: format!("SYM{}", i),
            quantity: 100.0 + i as f64,
            average_price: 50.0 + (i as f64 * 0.5),
        })
        .collect();

    let positions_hashmap: HashMap<String, Position> = positions_vec
        .iter()
        .map(|p| (p.symbol.clone(), p.clone()))
        .collect();

    let mut cache = PortfolioCache::new(positions_vec.clone());

    let mut group = c.benchmark_group("position_calculations");

    group.bench_function("vec_iteration", |b| {
        b.iter(|| calculate_total_value_vec(black_box(&positions_vec)))
    });

    group.bench_function("hashmap_iteration", |b| {
        b.iter(|| calculate_total_value_hashmap(black_box(&positions_hashmap)))
    });

    group.bench_function("cached_calculation", |b| {
        b.iter(|| cache.get_total_value())
    });

    group.finish();
}

criterion_group!(benches, bench_position_calculations);
criterion_main!(benches);
```

## Benchmarking Price Indicator Calculations

A common task in trading is calculating technical indicators. Let's benchmark different implementations:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Simple Moving Average - Naive approach
fn sma_naive(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in period - 1..prices.len() {
        let sum: f64 = prices[i - period + 1..=i].iter().sum();
        result.push(sum / period as f64);
    }

    result
}

// Simple Moving Average - Optimized with running sum
fn sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len() - period + 1);

    // Calculate first SMA
    let mut sum: f64 = prices[0..period].iter().sum();
    result.push(sum / period as f64);

    // Rolling calculation: add new price, remove old price
    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

// Exponential Moving Average
fn ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    // First EMA is the SMA
    let first_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(first_sma);

    // Calculate subsequent EMAs
    for i in period..prices.len() {
        let ema_value = (prices[i] - result[result.len() - 1]) * multiplier
            + result[result.len() - 1];
        result.push(ema_value);
    }

    result
}

fn bench_indicators(c: &mut Criterion) {
    // Generate sample price data
    let prices: Vec<f64> = (0..1000)
        .map(|i| 42000.0 + (i as f64 * 0.5).sin() * 1000.0)
        .collect();

    let mut group = c.benchmark_group("moving_averages");

    for period in [10, 20, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("sma_naive", period),
            period,
            |b, &period| {
                b.iter(|| sma_naive(black_box(&prices), black_box(period)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sma_optimized", period),
            period,
            |b, &period| {
                b.iter(|| sma_optimized(black_box(&prices), black_box(period)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ema", period),
            period,
            |b, &period| {
                b.iter(|| ema(black_box(&prices), black_box(period)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_indicators);
criterion_main!(benches);
```

## Understanding black_box()

The `black_box()` function is crucial for accurate benchmarks:

```rust
// WITHOUT black_box - compiler might optimize away the calculation
c.bench_function("bad_benchmark", |b| {
    b.iter(|| calculate_price(&order))  // Compiler might detect result is unused
});

// WITH black_box - prevents compiler optimizations
c.bench_function("good_benchmark", |b| {
    b.iter(|| calculate_price(black_box(&order)))  // Ensures calculation happens
});
```

**Why it matters**: The compiler is smart and might:
- Detect the result is unused and skip the calculation entirely
- Pre-compute constant values at compile time
- Inline and optimize away the function call

`black_box()` tells the compiler: "Treat this value as if it came from external input — don't optimize it away."

## Benchmark Groups and Comparisons

Compare multiple implementations side-by-side:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn calculate_vwap_v1(prices: &[f64], volumes: &[f64]) -> f64 {
    let total_volume: f64 = volumes.iter().sum();
    let weighted_sum: f64 = prices.iter()
        .zip(volumes.iter())
        .map(|(p, v)| p * v)
        .sum();
    weighted_sum / total_volume
}

fn calculate_vwap_v2(prices: &[f64], volumes: &[f64]) -> f64 {
    let (weighted_sum, total_volume) = prices.iter()
        .zip(volumes.iter())
        .fold((0.0, 0.0), |(sum, vol), (p, v)| {
            (sum + p * v, vol + v)
        });
    weighted_sum / total_volume
}

fn bench_vwap_comparison(c: &mut Criterion) {
    let prices: Vec<f64> = (0..1000).map(|i| 42000.0 + i as f64).collect();
    let volumes: Vec<f64> = (0..1000).map(|i| 1.0 + (i % 100) as f64).collect();

    let mut group = c.benchmark_group("vwap_comparison");

    group.bench_function("version_1_two_passes", |b| {
        b.iter(|| calculate_vwap_v1(black_box(&prices), black_box(&volumes)))
    });

    group.bench_function("version_2_single_fold", |b| {
        b.iter(|| calculate_vwap_v2(black_box(&prices), black_box(&volumes)))
    });

    group.finish();
}

criterion_group!(benches, bench_vwap_comparison);
criterion_main!(benches);
```

## Customizing Benchmark Configuration

Fine-tune measurement parameters:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn bench_with_custom_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_config");

    // Set custom measurement time
    group.measurement_time(Duration::from_secs(10));

    // Set custom warm-up time
    group.warm_up_time(Duration::from_secs(3));

    // Set sample size
    group.sample_size(100);

    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.0,
        price: 42000.0,
    };

    group.bench_function("order_calculation", |b| {
        b.iter(|| calculate_order_value(black_box(&order)))
    });

    group.finish();
}

criterion_group!(benches, bench_with_custom_config);
criterion_main!(benches);
```

## Interpreting Results

Criterion provides rich output:

```
order_calculation       time:   [1.2345 ns 1.2567 ns 1.2789 ns]
                        change: [-2.5432% -1.2345% +0.1234%] (p = 0.42 > 0.05)
                        No change in performance detected.
```

Breaking it down:
- **time: [low median high]**: The confidence interval for execution time
- **change**: Percentage change from the previous benchmark run
- **p-value**: Statistical significance (< 0.05 means significant change detected)
- **Performance verdict**: Detected improvement/regression or no change

### HTML Reports

Criterion generates detailed HTML reports in `target/criterion/`:
- Line charts showing performance over time
- Probability density functions
- Comparison charts between implementations
- Statistical analysis details

## What We Learned

| Concept | Description |
|---------|-------------|
| **criterion** | Statistics-driven benchmarking library for Rust |
| **black_box()** | Prevents compiler optimizations in benchmarks |
| **criterion_group!** | Macro to group related benchmarks |
| **criterion_main!** | Entry point for benchmark binary |
| **BenchmarkId** | Identifies parametric benchmarks |
| **Statistical analysis** | Automatic detection of performance regressions |
| **HTML reports** | Visual representation of benchmark results |
| **Warm-up time** | Time spent stabilizing before measurement |

## Practical Exercises

### Exercise 1: Benchmark Order Book Operations

Create benchmarks for these order book operations:
- Adding a new order to the book
- Finding the best bid/ask price
- Matching an incoming order against the book
- Canceling an order by ID

Compare performance with 10, 100, 1000, and 10000 orders in the book.

### Exercise 2: Compare Sorting Algorithms

Benchmark different methods for sorting trading signals by priority:
- Standard `sort()` method
- `sort_unstable()` method (doesn't preserve equal element order)
- Heap-based priority queue
- Custom sorting with cached comparisons

Test with 100, 1000, and 10000 signals.

### Exercise 3: Optimize Price Formatting

Traders need prices displayed with correct precision. Benchmark:
- Using `format!("{:.2}", price)` for formatting
- Pre-calculating string representations
- Using a custom number formatter
- Caching recently formatted prices

### Exercise 4: Concurrent Order Processing

Benchmark sequential vs. parallel order validation:
- Process 1000 orders sequentially
- Process using `rayon` parallel iterator
- Process using manual thread pool
- Process using async/await with Tokio

## Homework

1. **Comprehensive Strategy Benchmark Suite**: Create a complete benchmark suite for a trading strategy that:
   - Benchmarks signal generation (indicator calculations)
   - Benchmarks order execution simulation
   - Benchmarks risk management calculations
   - Compares at least 3 different implementations of each component
   - Generates a report identifying the fastest implementation
   - Includes tests with small (100 candles), medium (1000), and large (10000) datasets

2. **Regression Detection System**: Set up automated performance monitoring:
   - Create a baseline benchmark for your trading functions
   - Run benchmarks after every code change
   - Automatically detect performance regressions > 10%
   - Generate alerts (print to console) when regressions are detected
   - Document which code changes caused performance improvements/degradations

3. **Memory vs. Speed Trade-off**: Benchmark caching strategies:
   - Implement a price indicator calculator with no caching
   - Implement version with LRU cache (limited size)
   - Implement version with unlimited cache
   - Benchmark memory usage vs. calculation speed
   - Find the optimal cache size for 10000 price updates

4. **Real-World Trading Scenario**: Benchmark a complete order flow:
   - Receive market data update
   - Calculate technical indicators
   - Generate trading signal
   - Validate order against risk rules
   - Format and submit order

   Optimize the entire pipeline to process under 1 millisecond. Identify the bottleneck using criterion benchmarks.

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
