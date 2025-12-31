# Day 331: Latency vs Throughput

## Trading Analogy

Imagine two different types of trading systems:

**System A (High-Frequency Trading — HFT):**
- Processes a single order in 10 microseconds
- But can only handle 1,000 orders per second
- Priority: **minimum latency**

**System B (Batch Trading):**
- Processes a single order in 100 milliseconds
- But can handle 100,000 orders per second
- Priority: **maximum throughput**

This is like the difference between:
- **An F1 racing car** — incredibly fast on a single lap, but can only carry one person
- **A freight train** — slower, but transports thousands of tons per trip

In trading, this is a critical architectural decision:
- HFT systems sacrifice throughput for minimum latency
- Batch processing systems sacrifice latency for maximum throughput

## What are Latency and Throughput?

### Latency
Time from sending a request to receiving a response.

```
┌─────────────────────────────────────────────────────────────┐
│                    Latency                                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Send order → [Processing] → Confirmation                   │
│       │                            │                         │
│       └──────── 10 μs ─────────────┘                         │
│                                                              │
│  Measured in: microseconds (μs), milliseconds (ms)          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Throughput
Number of operations processed per unit of time.

```
┌─────────────────────────────────────────────────────────────┐
│                    Throughput                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                    │
│  │Order│ │Order│ │Order│ │Order│ │Order│  ──→ 5 orders/sec │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                    │
│                                                              │
│  Measured in: operations/second, requests/second            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Relationship Between Them

| Characteristic | Low Latency | High Throughput |
|----------------|-------------|-----------------|
| **Architecture** | Sequential processing | Parallel processing |
| **Buffering** | Minimal | Aggressive |
| **Batching** | No | Yes |
| **Trading example** | HFT, arbitrage | Reports, backtesting |
| **Priority** | Response time | Processing volume |

## Measuring Latency and Throughput in Rust

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Trading system performance metrics
struct PerformanceMetrics {
    latencies: VecDeque<Duration>,
    window_size: usize,
    total_operations: u64,
    start_time: Instant,
}

impl PerformanceMetrics {
    fn new(window_size: usize) -> Self {
        PerformanceMetrics {
            latencies: VecDeque::with_capacity(window_size),
            window_size,
            total_operations: 0,
            start_time: Instant::now(),
        }
    }

    /// Record operation latency
    fn record_latency(&mut self, latency: Duration) {
        if self.latencies.len() >= self.window_size {
            self.latencies.pop_front();
        }
        self.latencies.push_back(latency);
        self.total_operations += 1;
    }

    /// Average latency
    fn avg_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.latencies.iter().sum();
        total / self.latencies.len() as u32
    }

    /// Percentile latency (p50, p95, p99)
    fn percentile_latency(&self, percentile: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted: Vec<Duration> = self.latencies.iter().copied().collect();
        sorted.sort();

        let index = ((percentile / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[index]
    }

    /// Current throughput (operations per second)
    fn throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_operations as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Minimum latency
    fn min_latency(&self) -> Duration {
        self.latencies.iter().copied().min().unwrap_or(Duration::ZERO)
    }

    /// Maximum latency
    fn max_latency(&self) -> Duration {
        self.latencies.iter().copied().max().unwrap_or(Duration::ZERO)
    }

    /// Print statistics
    fn print_stats(&self) {
        println!("=== Performance Metrics ===");
        println!("Total operations: {}", self.total_operations);
        println!("Throughput: {:.2} ops/sec", self.throughput());
        println!();
        println!("Latency:");
        println!("  Min: {:?}", self.min_latency());
        println!("  Avg: {:?}", self.avg_latency());
        println!("  p50: {:?}", self.percentile_latency(50.0));
        println!("  p95: {:?}", self.percentile_latency(95.0));
        println!("  p99: {:?}", self.percentile_latency(99.0));
        println!("  Max: {:?}", self.max_latency());
    }
}

fn main() {
    let mut metrics = PerformanceMetrics::new(1000);

    // Simulate order processing
    for i in 0..1000 {
        let start = Instant::now();

        // Simulate order processing
        process_order(i);

        let latency = start.elapsed();
        metrics.record_latency(latency);
    }

    metrics.print_stats();
}

fn process_order(order_id: u32) {
    // Simulate work with varying load
    let work = (order_id % 100) as u64 * 10;
    std::hint::black_box(work);
}
```

## Optimizing for Low Latency

### Avoiding Allocations in the Hot Path

```rust
use std::time::Instant;

/// Order without dynamic allocations
#[derive(Debug, Clone, Copy)]
struct LowLatencyOrder {
    id: u64,
    symbol_id: u32,      // Instead of String
    price: f64,
    quantity: f64,
    side: OrderSide,
    timestamp_ns: u64,   // Nanoseconds instead of SystemTime
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

/// Pool of pre-allocated orders
struct OrderPool {
    orders: Vec<LowLatencyOrder>,
    free_indices: Vec<usize>,
}

impl OrderPool {
    fn new(capacity: usize) -> Self {
        let orders = (0..capacity)
            .map(|i| LowLatencyOrder {
                id: i as u64,
                symbol_id: 0,
                price: 0.0,
                quantity: 0.0,
                side: OrderSide::Buy,
                timestamp_ns: 0,
            })
            .collect();

        let free_indices = (0..capacity).rev().collect();

        OrderPool {
            orders,
            free_indices,
        }
    }

    /// Acquire order from pool (O(1), no allocation)
    #[inline(always)]
    fn acquire(&mut self) -> Option<&mut LowLatencyOrder> {
        self.free_indices.pop().map(|idx| &mut self.orders[idx])
    }

    /// Release order back to pool (O(1))
    #[inline(always)]
    fn release(&mut self, order: &LowLatencyOrder) {
        self.free_indices.push(order.id as usize);
    }

    fn available(&self) -> usize {
        self.free_indices.len()
    }
}

fn main() {
    let mut pool = OrderPool::new(10000);
    let mut latencies = Vec::with_capacity(1000);

    println!("=== Low Latency Order Pool ===\n");
    println!("Pool capacity: {}", pool.available());

    // Performance test
    for _ in 0..1000 {
        let start = Instant::now();

        if let Some(order) = pool.acquire() {
            order.symbol_id = 1;
            order.price = 42500.0;
            order.quantity = 0.1;
            order.side = OrderSide::Buy;
            order.timestamp_ns = start.elapsed().as_nanos() as u64;

            // Processing...

            pool.release(order);
        }

        latencies.push(start.elapsed());
    }

    // Statistics
    latencies.sort();
    let avg: std::time::Duration = latencies.iter().sum::<std::time::Duration>() / 1000;

    println!("\nLatency statistics:");
    println!("  Min: {:?}", latencies.first().unwrap());
    println!("  Avg: {:?}", avg);
    println!("  p99: {:?}", latencies[990]);
    println!("  Max: {:?}", latencies.last().unwrap());
}
```

### Lock-free Data Structures

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Lock-free order counter
struct OrderCounter {
    count: AtomicU64,
    total_value: AtomicU64, // In cents for atomicity
}

impl OrderCounter {
    fn new() -> Self {
        OrderCounter {
            count: AtomicU64::new(0),
            total_value: AtomicU64::new(0),
        }
    }

    /// Add order (lock-free)
    #[inline(always)]
    fn add_order(&self, value_cents: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_value.fetch_add(value_cents, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (u64, f64) {
        let count = self.count.load(Ordering::Relaxed);
        let value = self.total_value.load(Ordering::Relaxed) as f64 / 100.0;
        (count, value)
    }
}

/// Single-Producer Single-Consumer queue (SPSC)
struct SpscQueue<T> {
    buffer: Vec<Option<T>>,
    head: AtomicU64,
    tail: AtomicU64,
    capacity: usize,
}

impl<T: Clone> SpscQueue<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        SpscQueue {
            buffer,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            capacity,
        }
    }

    /// Push element (producer only)
    fn push(&mut self, item: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity as u64;

        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Queue full
        }

        self.buffer[tail as usize] = Some(item);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    /// Pop element (consumer only)
    fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Queue empty
        }

        let item = self.buffer[head as usize].take();
        self.head.store((head + 1) % self.capacity as u64, Ordering::Release);
        item
    }
}

fn main() {
    let counter = Arc::new(OrderCounter::new());
    let running = Arc::new(AtomicBool::new(true));

    println!("=== Lock-free Order Counter ===\n");

    // Launch multiple producer threads
    let mut handles = vec![];

    for thread_id in 0..4 {
        let counter = Arc::clone(&counter);
        let running = Arc::clone(&running);

        handles.push(thread::spawn(move || {
            let mut count = 0u64;
            while running.load(Ordering::Relaxed) {
                counter.add_order(100 + thread_id as u64);
                count += 1;
                if count >= 250000 {
                    break;
                }
            }
            count
        }));
    }

    // Let them work
    thread::sleep(Duration::from_secs(1));
    running.store(false, Ordering::Relaxed);

    // Collect results
    let mut total_produced = 0u64;
    for handle in handles {
        total_produced += handle.join().unwrap();
    }

    let (count, value) = counter.get_stats();
    println!("Orders processed: {}", count);
    println!("Total value: ${:.2}", value);
    println!("Throughput: {:.0} orders/sec", count as f64);
}
```

## Optimizing for High Throughput

### Batch Processing

```rust
use std::time::{Duration, Instant};

/// Order for batch processing
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

/// Processing result
#[derive(Debug)]
struct ProcessingResult {
    order_id: u64,
    success: bool,
    execution_price: f64,
}

/// Batch order processor
struct BatchProcessor {
    batch_size: usize,
    pending_orders: Vec<Order>,
    processed_count: u64,
}

impl BatchProcessor {
    fn new(batch_size: usize) -> Self {
        BatchProcessor {
            batch_size,
            pending_orders: Vec::with_capacity(batch_size),
            processed_count: 0,
        }
    }

    /// Add order to batch
    fn add_order(&mut self, order: Order) -> Option<Vec<ProcessingResult>> {
        self.pending_orders.push(order);

        if self.pending_orders.len() >= self.batch_size {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Process accumulated batch
    fn flush(&mut self) -> Vec<ProcessingResult> {
        let orders = std::mem::take(&mut self.pending_orders);
        self.pending_orders = Vec::with_capacity(self.batch_size);

        // Batch processing is more efficient for several reasons:
        // 1. Fewer system calls
        // 2. Better CPU cache utilization
        // 3. Opportunity for SIMD optimizations
        // 4. Amortized overhead costs

        let results: Vec<ProcessingResult> = orders
            .iter()
            .map(|order| {
                self.processed_count += 1;
                ProcessingResult {
                    order_id: order.id,
                    success: true,
                    execution_price: order.price * 1.001, // Simulate slippage
                }
            })
            .collect();

        results
    }

    fn pending_count(&self) -> usize {
        self.pending_orders.len()
    }

    fn processed_count(&self) -> u64 {
        self.processed_count
    }
}

/// Comparison: individual vs batch processing
fn compare_processing_methods() {
    println!("=== Throughput Comparison ===\n");

    let order_count = 100_000;

    // Generate test orders
    let orders: Vec<Order> = (0..order_count)
        .map(|i| Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 42500.0 + (i as f64 * 0.01).sin() * 100.0,
            quantity: 0.1,
        })
        .collect();

    // Method 1: Individual processing
    let start = Instant::now();
    let mut individual_results = Vec::with_capacity(order_count as usize);

    for order in &orders {
        // Simulate per-call overhead
        let result = ProcessingResult {
            order_id: order.id,
            success: true,
            execution_price: order.price * 1.001,
        };
        individual_results.push(result);
    }

    let individual_time = start.elapsed();
    let individual_throughput = order_count as f64 / individual_time.as_secs_f64();

    println!("Individual processing:");
    println!("  Time: {:?}", individual_time);
    println!("  Throughput: {:.0} orders/sec", individual_throughput);

    // Method 2: Batch processing
    let start = Instant::now();
    let mut batch_processor = BatchProcessor::new(1000);
    let mut batch_results = Vec::new();

    for order in orders {
        if let Some(results) = batch_processor.add_order(order) {
            batch_results.extend(results);
        }
    }
    // Process remaining
    batch_results.extend(batch_processor.flush());

    let batch_time = start.elapsed();
    let batch_throughput = order_count as f64 / batch_time.as_secs_f64();

    println!("\nBatch processing (batch_size=1000):");
    println!("  Time: {:?}", batch_time);
    println!("  Throughput: {:.0} orders/sec", batch_throughput);
    println!("\nImprovement: {:.1}x", batch_throughput / individual_throughput);
}

fn main() {
    compare_processing_methods();
}
```

### Parallel Processing with Rayon

```rust
use std::time::Instant;

// Add to Cargo.toml: rayon = "1.8"
// use rayon::prelude::*;

/// OHLCV candle
#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Calculate SMA
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();
    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

/// Calculate RSI
fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period);

    for i in period..prices.len() {
        let mut gains = 0.0;
        let mut losses = 0.0;

        for j in (i - period + 1)..=i {
            let change = prices[j] - prices[j - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };

        result.push(rsi);
    }

    result
}

/// Sequential processing of multiple symbols
fn process_sequential(symbols_data: &[(String, Vec<f64>)]) -> Vec<(String, Vec<f64>, Vec<f64>)> {
    symbols_data
        .iter()
        .map(|(symbol, prices)| {
            let sma = calculate_sma(prices, 20);
            let rsi = calculate_rsi(prices, 14);
            (symbol.clone(), sma, rsi)
        })
        .collect()
}

/// Parallel processing (conceptual — without rayon for compilation)
fn process_parallel_concept(symbols_data: &[(String, Vec<f64>)]) -> Vec<(String, Vec<f64>, Vec<f64>)> {
    // With rayon this would look like:
    // symbols_data
    //     .par_iter()
    //     .map(|(symbol, prices)| {
    //         let sma = calculate_sma(prices, 20);
    //         let rsi = calculate_rsi(prices, 14);
    //         (symbol.clone(), sma, rsi)
    //     })
    //     .collect()

    // For demonstration, using sequential version
    process_sequential(symbols_data)
}

fn main() {
    println!("=== Parallel Processing Throughput ===\n");

    // Generate data for 100 symbols
    let num_symbols = 100;
    let candles_per_symbol = 10_000;

    let symbols_data: Vec<(String, Vec<f64>)> = (0..num_symbols)
        .map(|i| {
            let symbol = format!("SYMBOL{:03}", i);
            let prices: Vec<f64> = (0..candles_per_symbol)
                .map(|j| 100.0 + (j as f64 * 0.01 + i as f64 * 0.1).sin() * 10.0)
                .collect();
            (symbol, prices)
        })
        .collect();

    println!("Data: {} symbols x {} candles = {} total candles\n",
             num_symbols, candles_per_symbol, num_symbols * candles_per_symbol);

    // Sequential processing
    let start = Instant::now();
    let results_seq = process_sequential(&symbols_data);
    let seq_time = start.elapsed();

    println!("Sequential processing:");
    println!("  Time: {:?}", seq_time);
    println!("  Throughput: {:.0} symbols/sec",
             num_symbols as f64 / seq_time.as_secs_f64());

    // Parallel processing (conceptual)
    let start = Instant::now();
    let results_par = process_parallel_concept(&symbols_data);
    let par_time = start.elapsed();

    println!("\nParallel processing (conceptual):");
    println!("  Time: {:?}", par_time);
    println!("  Throughput: {:.0} symbols/sec",
             num_symbols as f64 / par_time.as_secs_f64());

    // Verify correctness
    assert_eq!(results_seq.len(), results_par.len());
    println!("\nResults validated: {} symbols processed", results_seq.len());

    println!("\nNote: With rayon, parallel processing would be ~4-8x faster");
    println!("      on a multi-core CPU depending on core count.");
}
```

## Trade-off: Latency vs Throughput

### Choosing Strategy for Different Scenarios

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Operation type
#[derive(Debug, Clone, Copy)]
enum OperationType {
    /// Urgent arbitrage — needs minimum latency
    Arbitrage,
    /// Market making — balance latency/throughput
    MarketMaking,
    /// Reports and analytics — throughput priority
    Analytics,
    /// Backtesting — maximum throughput
    Backtesting,
}

impl OperationType {
    fn recommended_batch_size(&self) -> usize {
        match self {
            OperationType::Arbitrage => 1,      // No batching
            OperationType::MarketMaking => 10,  // Small batches
            OperationType::Analytics => 1000,   // Medium batches
            OperationType::Backtesting => 10000, // Large batches
        }
    }

    fn max_latency(&self) -> Duration {
        match self {
            OperationType::Arbitrage => Duration::from_micros(100),
            OperationType::MarketMaking => Duration::from_millis(10),
            OperationType::Analytics => Duration::from_secs(1),
            OperationType::Backtesting => Duration::from_secs(60),
        }
    }
}

/// Adaptive processor with configurable balance
struct AdaptiveProcessor {
    operation_type: OperationType,
    pending: VecDeque<u64>,
    batch_size: usize,
    processed: u64,
    total_latency: Duration,
}

impl AdaptiveProcessor {
    fn new(operation_type: OperationType) -> Self {
        let batch_size = operation_type.recommended_batch_size();
        AdaptiveProcessor {
            operation_type,
            pending: VecDeque::with_capacity(batch_size),
            batch_size,
            processed: 0,
            total_latency: Duration::ZERO,
        }
    }

    /// Submit operation
    fn submit(&mut self, operation_id: u64) -> Option<Vec<u64>> {
        let start = Instant::now();
        self.pending.push_back(operation_id);

        // Check if batch should be processed
        let should_process = match self.operation_type {
            OperationType::Arbitrage => true, // Always immediate
            _ => self.pending.len() >= self.batch_size,
        };

        if should_process {
            let results = self.process_batch();
            self.total_latency += start.elapsed();
            Some(results)
        } else {
            None
        }
    }

    /// Process batch
    fn process_batch(&mut self) -> Vec<u64> {
        let batch: Vec<u64> = self.pending.drain(..).collect();
        self.processed += batch.len() as u64;

        // Simulate processing
        let work_per_item = match self.operation_type {
            OperationType::Arbitrage => 1,
            OperationType::MarketMaking => 10,
            OperationType::Analytics => 100,
            OperationType::Backtesting => 5,
        };

        for _ in 0..batch.len() * work_per_item {
            std::hint::black_box(0);
        }

        batch
    }

    /// Force process remaining
    fn flush(&mut self) -> Vec<u64> {
        if self.pending.is_empty() {
            return vec![];
        }
        self.process_batch()
    }

    fn stats(&self) -> (u64, Duration) {
        (self.processed, self.total_latency)
    }
}

fn benchmark_operation_type(op_type: OperationType, num_operations: u64) {
    let mut processor = AdaptiveProcessor::new(op_type);
    let start = Instant::now();

    for i in 0..num_operations {
        let _ = processor.submit(i);
    }
    processor.flush();

    let elapsed = start.elapsed();
    let (processed, _) = processor.stats();
    let throughput = processed as f64 / elapsed.as_secs_f64();
    let avg_latency = elapsed / processed as u32;

    println!("{:?}:", op_type);
    println!("  Batch size: {}", op_type.recommended_batch_size());
    println!("  Throughput: {:.0} ops/sec", throughput);
    println!("  Avg latency: {:?}", avg_latency);
    println!("  Max acceptable latency: {:?}", op_type.max_latency());
    println!();
}

fn main() {
    println!("=== Latency vs Throughput Trade-offs ===\n");

    let num_operations = 100_000;

    benchmark_operation_type(OperationType::Arbitrage, num_operations);
    benchmark_operation_type(OperationType::MarketMaking, num_operations);
    benchmark_operation_type(OperationType::Analytics, num_operations);
    benchmark_operation_type(OperationType::Backtesting, num_operations);
}
```

## Practical Recommendations

### When to Optimize for Latency

| Scenario | Reason | Techniques |
|----------|--------|------------|
| **HFT** | Speed advantage = profit | Avoid allocations, lock-free |
| **Arbitrage** | Opportunity window — milliseconds | Pre-allocated buffers |
| **Market Making** | Fast quote updates | SPSC queues |
| **Stop-losses** | Minimize losses | Inline critical code |

### When to Optimize for Throughput

| Scenario | Reason | Techniques |
|----------|--------|------------|
| **Backtesting** | Millions of operations | Batching, parallelism |
| **EOD Reports** | Large data volumes | Bulk operations |
| **Risk calculations** | Many instruments | SIMD, rayon |
| **Data aggregation** | Stream processing | Buffering |

```rust
use std::time::{Duration, Instant};

/// Optimization recommendations
struct OptimizationGuide;

impl OptimizationGuide {
    /// Analyze requirements and choose strategy
    fn analyze_requirements(
        target_latency: Duration,
        target_throughput: u64,
        operation_complexity: f64,
    ) -> String {
        let latency_critical = target_latency < Duration::from_millis(10);
        let throughput_critical = target_throughput > 100_000;

        let strategy = match (latency_critical, throughput_critical) {
            (true, false) => {
                "LATENCY-FOCUSED:\n\
                 - Use object pools to avoid allocations\n\
                 - Prefer lock-free data structures\n\
                 - Inline hot paths with #[inline(always)]\n\
                 - Avoid dynamic dispatch (dyn Trait)\n\
                 - Pre-compute what's possible\n\
                 - Use thread-per-core architecture"
            }
            (false, true) => {
                "THROUGHPUT-FOCUSED:\n\
                 - Use batch processing\n\
                 - Enable parallel processing (rayon)\n\
                 - Use async I/O for network operations\n\
                 - Buffer aggressively\n\
                 - Optimize for cache locality\n\
                 - Consider SIMD operations"
            }
            (true, true) => {
                "BALANCED (Hard!):\n\
                 - Use bounded queues between stages\n\
                 - Implement backpressure\n\
                 - Consider separate paths for fast/slow\n\
                 - Profile extensively\n\
                 - May need to relax one requirement"
            }
            (false, false) => {
                "RELAXED:\n\
                 - Focus on code clarity first\n\
                 - Standard Rust idioms are fine\n\
                 - Optimize only proven bottlenecks\n\
                 - Use profiler to find hot spots"
            }
        };

        format!("Requirements Analysis:\n\
                 - Target latency: {:?} (critical: {})\n\
                 - Target throughput: {} ops/sec (critical: {})\n\
                 - Operation complexity: {:.1}\n\n\
                 Recommended Strategy:\n{}",
                target_latency, latency_critical,
                target_throughput, throughput_critical,
                operation_complexity, strategy)
    }
}

fn main() {
    println!("=== Optimization Guide ===\n");

    // HFT scenario
    println!("1. HFT Trading System:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_micros(100),
        10_000,
        0.5,
    ));

    // Backtesting
    println!("2. Backtesting Engine:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_secs(1),
        1_000_000,
        1.0,
    ));

    // Critical system
    println!("3. Real-time Risk Engine:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_millis(5),
        500_000,
        2.0,
    ));
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Latency** | Time to process a single operation |
| **Throughput** | Number of operations per time unit |
| **Trade-off** | Improving one often degrades the other |
| **Object Pool** | Reusing objects to avoid allocation |
| **Lock-free** | Data structures without locks |
| **Batching** | Batch processing for throughput |
| **Percentile** | p50, p95, p99 — more important than average |

## Practical Exercises

1. **Metrics Collection**: Add latency and throughput collection to existing trading code. Build a latency histogram.

2. **Object Pool**: Implement a pool for the `Order` structure with `acquire()` and `release()` methods. Compare performance with regular creation.

3. **Adaptive Batching**: Create a system that automatically adjusts batch size to achieve target latency/throughput balance.

4. **Lock-free Queue**: Implement an MPSC (Multiple Producer Single Consumer) queue without locks.

## Homework

1. **Profiling**: Take your trading system (or the example from this chapter) and profile it:
   - Measure p50, p95, p99 latency
   - Measure throughput
   - Identify bottlenecks
   - Propose optimizations

2. **Dual-Mode Processor**: Create an order processing system with two modes:
   - "Fast path" for VIP clients (minimum latency)
   - "Bulk path" for regular orders (maximum throughput)

3. **Load Simulator**: Write a tool that:
   - Generates orders with specified distribution (uniform, bursty)
   - Measures latency under different loads
   - Plots degradation graph (latency increase as load grows)

4. **Comparison Benchmark**: Compare three approaches to market data processing:
   - Sequential (baseline)
   - Batched (different batch sizes)
   - Parallel (rayon)
   Determine the crossover point where each approach becomes optimal.

## Navigation

[← Previous day](../330-*/en.md) | [Next day →](../332-*/en.md)
