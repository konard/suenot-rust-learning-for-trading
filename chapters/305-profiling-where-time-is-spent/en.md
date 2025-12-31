# Day 305: Profiling: Where Time is Spent

## Trading Analogy

Imagine your trading strategy works, but slowly. You process 1000 ticks per second, but competitors process 10000. Where's the problem? Maybe you're spending too much time calculating indicators? Or serializing data? Perhaps the bottleneck is in database access?

It's like trying to understand why your trading bot lags behind the market. You see the symptom (slow performance), but don't know the cause. **Profiling** is a diagnostic tool that shows exactly where time is spent:

- Which functions are called most frequently?
- Which operations take the most time?
- Where do unexpected delays occur?
- Which parts of the code can be optimized?

Just as a trader analyzes their journal of trades to understand where they're losing money, a programmer uses a profiler to understand where time is being lost.

## What is Profiling?

Profiling is measuring program performance to identify bottlenecks. A profiler collects data about:

1. **Execution time** — how long each function takes
2. **Call frequency** — how often each function is called
3. **Call tree** — which functions call other functions
4. **Memory consumption** — how much memory is allocated and where

## Types of Profiling

| Type | Description | Trading Application |
|------|-------------|---------------------|
| CPU profiling | Where processor time is spent | Optimizing indicator calculations |
| Memory profiling | How memory is used | Finding memory leaks in long-running bots |
| I/O profiling | Time spent on input-output | Optimizing database operations |
| Sampling profiling | Periodic stack snapshots | Low overhead |
| Instrumentation profiling | Precise measurement of each call | Detailed analysis of critical paths |

## Basic Manual Profiling

```rust
use std::time::Instant;

#[derive(Debug)]
struct PriceData {
    symbol: String,
    price: f64,
    volume: f64,
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }

    sma
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut ema = Vec::new();
    let multiplier = 2.0 / (period + 1) as f64;

    if prices.is_empty() {
        return ema;
    }

    // First value - simple average
    if prices.len() >= period {
        let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
        ema.push(initial_sma);

        // Remaining values - exponential average
        for i in period..prices.len() {
            let new_ema = (prices[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }
    }

    ema
}

fn analyze_market(prices: &[f64]) -> f64 {
    let sma_20 = calculate_sma(prices, 20);
    let sma_50 = calculate_sma(prices, 50);
    let ema_12 = calculate_ema(prices, 12);
    let ema_26 = calculate_ema(prices, 26);

    // Simple logic: if fast MA is above slow MA, buy signal
    if let (Some(&last_sma_20), Some(&last_sma_50)) = (sma_20.last(), sma_50.last()) {
        if last_sma_20 > last_sma_50 {
            return 1.0; // Buy signal
        }
    }

    0.0 // No signal
}

fn main() {
    // Generate test data
    let prices: Vec<f64> = (0..10000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    println!("=== Manual Profiling ===\n");

    // Profile SMA
    let start = Instant::now();
    let _sma = calculate_sma(&prices, 20);
    let sma_duration = start.elapsed();
    println!("SMA(20): {:?}", sma_duration);

    // Profile EMA
    let start = Instant::now();
    let _ema = calculate_ema(&prices, 20);
    let ema_duration = start.elapsed();
    println!("EMA(20): {:?}", ema_duration);

    // Profile full analysis
    let start = Instant::now();
    let _signal = analyze_market(&prices);
    let analysis_duration = start.elapsed();
    println!("Full analysis: {:?}", analysis_duration);

    // Compare
    println!("\nComparison:");
    println!("EMA is {:.2}x slower than SMA",
        ema_duration.as_nanos() as f64 / sma_duration.as_nanos() as f64);
}
```

## Structured Profiling with Metrics

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ProfileMetrics {
    name: String,
    call_count: u64,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl ProfileMetrics {
    fn new(name: &str) -> Self {
        ProfileMetrics {
            name: name.to_string(),
            call_count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_duration += duration;

        if duration < self.min_duration {
            self.min_duration = duration;
        }

        if duration > self.max_duration {
            self.max_duration = duration;
        }
    }

    fn avg_duration(&self) -> Duration {
        if self.call_count > 0 {
            self.total_duration / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }

    fn print_report(&self) {
        println!("Function: {}", self.name);
        println!("  Calls: {}", self.call_count);
        println!("  Total time: {:?}", self.total_duration);
        println!("  Average: {:?}", self.avg_duration());
        println!("  Min: {:?}, Max: {:?}", self.min_duration, self.max_duration);

        if self.call_count > 0 {
            let total_micros = self.total_duration.as_micros();
            let calls_per_sec = if total_micros > 0 {
                (self.call_count as f64 * 1_000_000.0) / total_micros as f64
            } else {
                0.0
            };
            println!("  Throughput: {:.0} calls/sec", calls_per_sec);
        }
        println!();
    }
}

struct Profiler {
    metrics: HashMap<String, ProfileMetrics>,
}

impl Profiler {
    fn new() -> Self {
        Profiler {
            metrics: HashMap::new(),
        }
    }

    fn profile<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.metrics
            .entry(name.to_string())
            .or_insert_with(|| ProfileMetrics::new(name))
            .record(duration);

        result
    }

    fn report(&self) {
        println!("\n=== Profiling Report ===\n");

        // Sort by total time
        let mut metrics: Vec<_> = self.metrics.values().collect();
        metrics.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

        let total_time: Duration = metrics.iter().map(|m| m.total_duration).sum();

        for metric in metrics {
            metric.print_report();

            // Percentage of total time
            if total_time.as_nanos() > 0 {
                let percentage = (metric.total_duration.as_nanos() as f64
                    / total_time.as_nanos() as f64) * 100.0;
                println!("  Share of total time: {:.2}%", percentage);
                println!();
            }
        }

        println!("TOTAL: {:?}", total_time);
    }
}

// Trading strategy with profiling
struct TradingStrategy {
    profiler: Profiler,
}

impl TradingStrategy {
    fn new() -> Self {
        TradingStrategy {
            profiler: Profiler::new(),
        }
    }

    fn fetch_market_data(&mut self) -> Vec<f64> {
        self.profiler.profile("fetch_market_data", || {
            // Simulate data fetching
            std::thread::sleep(Duration::from_micros(100));
            (0..1000).map(|i| 50000.0 + (i as f64).sin() * 100.0).collect()
        })
    }

    fn calculate_indicators(&mut self, prices: &[f64]) -> (Vec<f64>, Vec<f64>) {
        self.profiler.profile("calculate_indicators", || {
            let sma = self.profiler.profile("calculate_sma", || {
                calculate_sma(prices, 20)
            });

            let ema = self.profiler.profile("calculate_ema", || {
                calculate_ema(prices, 20)
            });

            (sma, ema)
        })
    }

    fn generate_signal(&mut self, sma: &[f64], ema: &[f64]) -> i32 {
        self.profiler.profile("generate_signal", || {
            if let (Some(&last_sma), Some(&last_ema)) = (sma.last(), ema.last()) {
                if last_ema > last_sma {
                    return 1; // Buy
                } else if last_ema < last_sma {
                    return -1; // Sell
                }
            }
            0 // Hold
        })
    }

    fn execute_trade(&mut self, signal: i32) {
        self.profiler.profile("execute_trade", || {
            if signal != 0 {
                // Simulate order sending
                std::thread::sleep(Duration::from_micros(50));
            }
        });
    }

    fn run_strategy(&mut self, iterations: usize) {
        for _ in 0..iterations {
            let prices = self.fetch_market_data();
            let (sma, ema) = self.calculate_indicators(&prices);
            let signal = self.generate_signal(&sma, &ema);
            self.execute_trade(signal);
        }
    }

    fn report(&self) {
        self.profiler.report();
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }
    sma
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut ema = Vec::new();
    let multiplier = 2.0 / (period + 1) as f64;

    if prices.len() >= period {
        let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
        ema.push(initial_sma);

        for i in period..prices.len() {
            let new_ema = (prices[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }
    }

    ema
}

fn main() {
    let mut strategy = TradingStrategy::new();

    println!("Running strategy with profiling...\n");

    let start = Instant::now();
    strategy.run_strategy(100);
    let total_time = start.elapsed();

    println!("Total execution time: {:?}\n", total_time);

    strategy.report();
}
```

## Profiling with Flamegraph

Flamegraph is a visualization of where time is spent in a program. Each "bar" represents a function, the width of the bar represents execution time.

### Installation and Usage

```bash
# Install flamegraph
cargo install flamegraph

# For Linux, perf permissions are needed
sudo sysctl -w kernel.perf_event_paranoid=-1

# Create flamegraph
cargo flamegraph --bin trading_bot

# Opens flamegraph.svg in browser
```

### Example Code for Profiling

```rust
// trading_bot/src/main.rs
use std::time::Duration;

fn calculate_heavy_indicator(prices: &[f64]) -> Vec<f64> {
    // Complex calculations we want to profile
    prices
        .windows(50)
        .map(|window| {
            let sum: f64 = window.iter().sum();
            let mean = sum / window.len() as f64;
            let variance: f64 = window.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            variance.sqrt()
        })
        .collect()
}

fn process_market_data(data: &[f64]) -> f64 {
    let volatility = calculate_heavy_indicator(data);
    let avg_volatility: f64 = volatility.iter().sum::<f64>() / volatility.len() as f64;
    avg_volatility
}

fn main() {
    let prices: Vec<f64> = (0..10000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    // Run many times so profiler collects statistics
    for _ in 0..1000 {
        let _result = process_market_data(&prices);
    }
}
```

## Profiling with Criterion (Benchmarks)

Criterion is a library for precise performance measurements.

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "indicator_benchmarks"
harness = false
```

```rust
// benches/indicator_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }
    sma
}

fn calculate_sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::with_capacity(prices.len().saturating_sub(period - 1));

    if prices.len() < period {
        return sma;
    }

    // First value
    let mut sum: f64 = prices[..period].iter().sum();
    sma.push(sum / period as f64);

    // Remaining values use sliding window
    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        sma.push(sum / period as f64);
    }

    sma
}

fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA");

    for size in [100, 1000, 10000].iter() {
        let prices: Vec<f64> = (0..*size)
            .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
            .collect();

        group.bench_with_input(
            BenchmarkId::new("naive", size),
            &prices,
            |b, prices| {
                b.iter(|| calculate_sma(black_box(prices), black_box(20)))
            }
        );

        group.bench_with_input(
            BenchmarkId::new("optimized", size),
            &prices,
            |b, prices| {
                b.iter(|| calculate_sma_optimized(black_box(prices), black_box(20)))
            }
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_sma);
criterion_main!(benches);
```

Running benchmarks:

```bash
cargo bench
```

Criterion creates a detailed report with:
- Average execution time
- Standard deviation
- Comparison with previous runs
- HTML report with graphs

## Memory Profiling

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct OrderBookEntry {
    price: f64,
    quantity: f64,
    timestamp: u64,
}

struct OrderBook {
    bids: Vec<OrderBookEntry>,
    asks: Vec<OrderBookEntry>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    // Suboptimal version - creates many temporary objects
    fn get_best_bid_naive(&self) -> Option<f64> {
        let sorted_bids: Vec<_> = self.bids
            .iter()
            .map(|entry| entry.price)
            .collect();

        sorted_bids.iter().max().copied()
    }

    // Optimized version - doesn't create intermediate collections
    fn get_best_bid_optimized(&self) -> Option<f64> {
        self.bids
            .iter()
            .map(|entry| entry.price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Fill order book
    for i in 0..10000 {
        book.bids.push(OrderBookEntry {
            price: 50000.0 + i as f64,
            quantity: 1.0,
            timestamp: i,
        });
    }

    println!("Order book size: {} entries", book.bids.len());
    println!("Memory per entry: {} bytes",
        std::mem::size_of::<OrderBookEntry>());
    println!("Total memory: {} KB",
        book.bids.len() * std::mem::size_of::<OrderBookEntry>() / 1024);
}
```

For detailed memory profiling use:

```bash
# valgrind (Linux)
cargo build --release
valgrind --tool=massif ./target/release/trading_bot

# heaptrack (Linux)
heaptrack ./target/release/trading_bot

# Instruments (macOS)
# Use Xcode Instruments -> Allocations
```

## Practical Profiling: Optimizing JSON Parsing

```rust
use std::time::Instant;

// Example market data in JSON
const MARKET_DATA_JSON: &str = r#"
{
    "symbol": "BTCUSDT",
    "price": 50000.0,
    "volume": 123.45,
    "timestamp": 1234567890
}
"#;

// Version 1: Use serde_json (slower, but more convenient)
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "use_serde")]
#[derive(Deserialize, Serialize, Debug)]
struct MarketDataSerde {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

// Version 2: Manual parsing (faster, but more fragile)
#[derive(Debug)]
struct MarketDataManual {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

impl MarketDataManual {
    fn parse_simple(json: &str) -> Option<Self> {
        // Simplified parser for demonstration
        // In reality use a proper JSON parser
        let symbol = json.split("\"symbol\":").nth(1)?
            .split('"').nth(1)?
            .to_string();

        let price = json.split("\"price\":").nth(1)?
            .split(',').next()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let volume = json.split("\"volume\":").nth(1)?
            .split(',').next()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let timestamp = json.split("\"timestamp\":").nth(1)?
            .split('}').next()?
            .trim()
            .parse::<u64>()
            .ok()?;

        Some(MarketDataManual {
            symbol,
            price,
            volume,
            timestamp,
        })
    }
}

fn main() {
    let iterations = 100000;

    // Profile manual parsing
    let start = Instant::now();
    for _ in 0..iterations {
        let _data = MarketDataManual::parse_simple(MARKET_DATA_JSON);
    }
    let manual_duration = start.elapsed();

    println!("=== Parsing Profiling ===");
    println!("Manual parsing: {:?} ({:.0} ops/sec)",
        manual_duration,
        iterations as f64 / manual_duration.as_secs_f64()
    );

    // In real code this would be a serde_json benchmark
    println!("\nFor full comparison run with use_serde flag");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Manual profiling | Using `Instant` to measure execution time |
| Structured metrics | Collecting statistics on function calls |
| Flamegraph | Visualizing hot paths in code |
| Criterion | Precise benchmarks with statistical analysis |
| Memory profiling | Identifying leaks and inefficient memory usage |
| Data-driven optimization | Measure before optimizing |

## Practical Exercises

1. **Hot Function Profiler**: Create a `profile!` macro that automatically measures function execution time and collects statistics in a global profiler.

2. **Comparative Benchmark**: Implement several variants of Bollinger Bands calculation and compare their performance using criterion:
   - Naive version with multiple passes through data
   - Optimized version with single pass
   - SIMD version (if possible)

3. **Allocation Profiler**: Create a wrapper around `Vec` that tracks all memory allocations and outputs a report on where the program allocates memory most.

## Homework

1. **Real-time Strategy Profiling**: Create a trading strategy and profile it:
   - Measure latency from tick receipt to order sending
   - Find the slowest components
   - Optimize bottlenecks
   - Document performance improvements

2. **Flamegraph Analysis**: Take any complex algorithm (e.g., strategy backtester):
   - Create a flamegraph
   - Analyze where time is spent
   - Optimize the top-3 slowest functions
   - Create a new flamegraph and compare results

3. **Benchmark Suite**: Create a set of benchmarks for typical trading operations:
   - Parsing market data (JSON, MessagePack, Protocol Buffers)
   - Calculating indicators (SMA, EMA, RSI, MACD)
   - Working with order book (insert, delete, find best price)
   - Compare different approaches and choose the fastest

4. **Memory Profiler**: Implement a memory usage monitoring system:
   - Tracking current heap usage
   - Detecting memory leaks
   - Warnings when threshold is exceeded
   - Reports on the "heaviest" data structures

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
