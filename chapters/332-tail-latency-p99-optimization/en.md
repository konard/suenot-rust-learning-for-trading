# Day 332: Tail Latency: p99 Optimization

## Trading Analogy

Imagine you run a high-frequency trading firm. Your average order execution takes 1ms — impressive! But here's the problem: once in every 100 orders (p99), execution takes 50ms. During these "tail" events, the market has already moved, and you're left with unfavorable fills or missed opportunities.

**Tail latency** is like the slowest trader in your team. Even if 99 traders are lightning fast, that one slow trader can ruin the entire day's profits when they handle a critical order.

| Metric | Description | Trading Impact |
|--------|-------------|----------------|
| **p50 (median)** | 50% of requests are faster | Average experience |
| **p90** | 90% of requests are faster | Most orders are fine |
| **p95** | 95% of requests are faster | Occasional slowdowns |
| **p99** | 99% of requests are faster | Critical for HFT |
| **p99.9** | 99.9% of requests are faster | Ultra-low latency systems |

In trading, p99 matters because:
- A single slow order can cause significant slippage
- Arbitrage opportunities disappear in milliseconds
- Risk management decisions must be instant
- Market makers need consistent response times

## Understanding Tail Latency

### What Causes Tail Latency?

```rust
use std::time::{Duration, Instant};
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// Order execution with various latency sources
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: Instant,
}

/// Latency measurement with percentile tracking
struct LatencyTracker {
    latencies: Vec<Duration>,
}

impl LatencyTracker {
    fn new() -> Self {
        LatencyTracker {
            latencies: Vec::with_capacity(10000),
        }
    }

    fn record(&mut self, latency: Duration) {
        self.latencies.push(latency);
    }

    fn percentile(&mut self, p: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }

        self.latencies.sort();
        let index = ((self.latencies.len() as f64 * p / 100.0) as usize)
            .min(self.latencies.len() - 1);
        self.latencies[index]
    }

    fn report(&mut self) {
        println!("Latency Report:");
        println!("  p50:  {:?}", self.percentile(50.0));
        println!("  p90:  {:?}", self.percentile(90.0));
        println!("  p95:  {:?}", self.percentile(95.0));
        println!("  p99:  {:?}", self.percentile(99.0));
        println!("  p99.9: {:?}", self.percentile(99.9));
        println!("  max:  {:?}", self.latencies.iter().max().unwrap_or(&Duration::ZERO));
    }
}

/// Simulate various causes of tail latency
fn process_order_with_latency_sources(order: &Order, iteration: u64) -> Duration {
    let start = Instant::now();

    // Simulate normal processing
    std::hint::black_box(order.price * order.quantity);

    // Cause 1: Occasional GC-like pauses (memory allocation)
    if iteration % 100 == 0 {
        let _allocations: Vec<Vec<u8>> = (0..1000)
            .map(|_| vec![0u8; 1024])
            .collect();
    }

    // Cause 2: Lock contention simulation
    if iteration % 50 == 0 {
        std::thread::sleep(Duration::from_micros(100));
    }

    // Cause 3: Cache miss simulation
    if iteration % 200 == 0 {
        let data: Vec<u64> = (0..10000).collect();
        let sum: u64 = data.iter().step_by(64).sum();
        std::hint::black_box(sum);
    }

    start.elapsed()
}

fn main() {
    println!("=== Understanding Tail Latency Sources ===\n");

    let mut tracker = LatencyTracker::new();

    for i in 0..10000 {
        let order = Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            quantity: 0.1,
            timestamp: Instant::now(),
        };

        let latency = process_order_with_latency_sources(&order, i);
        tracker.record(latency);
    }

    tracker.report();

    println!("\nKey insight: p99 can be 10-100x higher than p50!");
}
```

## Strategy 1: Pre-allocation and Object Pooling

One major source of tail latency is memory allocation. Pre-allocating buffers eliminates this:

```rust
use std::time::{Duration, Instant};

/// Pre-allocated buffer for order processing
struct OrderBuffer {
    data: Vec<u8>,
    position: usize,
}

impl OrderBuffer {
    fn with_capacity(size: usize) -> Self {
        OrderBuffer {
            data: vec![0u8; size],
            position: 0,
        }
    }

    fn reset(&mut self) {
        self.position = 0;
    }

    fn write(&mut self, bytes: &[u8]) -> bool {
        if self.position + bytes.len() > self.data.len() {
            return false;
        }
        self.data[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        true
    }
}

/// Object pool for reusing order structures
struct OrderPool {
    available: Vec<Box<OrderData>>,
    in_use: usize,
}

#[derive(Default)]
struct OrderData {
    id: u64,
    symbol: [u8; 16],
    symbol_len: usize,
    price: f64,
    quantity: f64,
    side: u8, // 0 = buy, 1 = sell
}

impl OrderPool {
    fn new(initial_size: usize) -> Self {
        let available = (0..initial_size)
            .map(|_| Box::new(OrderData::default()))
            .collect();

        OrderPool {
            available,
            in_use: 0,
        }
    }

    fn acquire(&mut self) -> Option<Box<OrderData>> {
        self.available.pop().map(|order| {
            self.in_use += 1;
            order
        })
    }

    fn release(&mut self, mut order: Box<OrderData>) {
        // Reset the order data
        order.id = 0;
        order.symbol_len = 0;
        order.price = 0.0;
        order.quantity = 0.0;
        order.side = 0;

        self.available.push(order);
        self.in_use -= 1;
    }

    fn stats(&self) -> (usize, usize) {
        (self.available.len(), self.in_use)
    }
}

fn benchmark_allocation_strategies() {
    println!("=== Pre-allocation vs Dynamic Allocation ===\n");

    const ITERATIONS: usize = 100_000;

    // Dynamic allocation (causes tail latency)
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        let buffer = vec![0u8; 1024];
        std::hint::black_box(&buffer);
        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("Dynamic allocation:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);

    // Pre-allocated buffer (consistent latency)
    let mut buffer = OrderBuffer::with_capacity(1024);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        buffer.reset();
        buffer.write(&[0u8; 100]);
        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("\nPre-allocated buffer:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);

    // Object pooling
    let mut pool = OrderPool::new(1000);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        if let Some(mut order) = pool.acquire() {
            order.id = i as u64;
            order.price = 50000.0;
            std::hint::black_box(&order);
            pool.release(order);
        }

        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("\nObject pooling:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);
}

fn main() {
    benchmark_allocation_strategies();
}
```

## Strategy 2: Lock-Free Data Structures

Locks are a major source of tail latency due to contention. Lock-free alternatives provide more consistent performance:

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Lock-free order counter using atomics
struct AtomicOrderCounter {
    buy_count: AtomicU64,
    sell_count: AtomicU64,
    total_volume: AtomicU64, // Stored as fixed-point (multiply by 1000)
}

impl AtomicOrderCounter {
    fn new() -> Self {
        AtomicOrderCounter {
            buy_count: AtomicU64::new(0),
            sell_count: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
        }
    }

    fn record_buy(&self, volume: f64) {
        self.buy_count.fetch_add(1, Ordering::Relaxed);
        let volume_fixed = (volume * 1000.0) as u64;
        self.total_volume.fetch_add(volume_fixed, Ordering::Relaxed);
    }

    fn record_sell(&self, volume: f64) {
        self.sell_count.fetch_add(1, Ordering::Relaxed);
        let volume_fixed = (volume * 1000.0) as u64;
        self.total_volume.fetch_add(volume_fixed, Ordering::Relaxed);
    }

    fn stats(&self) -> (u64, u64, f64) {
        let buys = self.buy_count.load(Ordering::Relaxed);
        let sells = self.sell_count.load(Ordering::Relaxed);
        let volume = self.total_volume.load(Ordering::Relaxed) as f64 / 1000.0;
        (buys, sells, volume)
    }
}

/// Lock-free SPSC (Single Producer Single Consumer) ring buffer
/// Ideal for order queues between threads
struct SpscQueue<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: AtomicUsize, // Consumer reads from here
    tail: AtomicUsize, // Producer writes here
}

impl<T> SpscQueue<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        SpscQueue {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    fn push(&mut self, item: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;

        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Queue is full
        }

        self.buffer[tail] = Some(item);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Queue is empty
        }

        let item = self.buffer[head].take();
        self.head.store((head + 1) % self.capacity, Ordering::Release);
        item
    }
}

/// Benchmark lock-free vs mutex-based counter
fn benchmark_lock_free() {
    println!("=== Lock-Free vs Mutex Performance ===\n");

    use std::sync::Mutex;

    const ITERATIONS: u64 = 1_000_000;
    const THREADS: usize = 4;

    // Mutex-based counter
    let mutex_counter = Arc::new(Mutex::new((0u64, 0u64, 0.0f64)));
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let counter = Arc::clone(&mutex_counter);
            thread::spawn(move || {
                let mut max_latency = Duration::ZERO;
                for i in 0..ITERATIONS / THREADS as u64 {
                    let iter_start = Instant::now();
                    let mut guard = counter.lock().unwrap();
                    if i % 2 == 0 {
                        guard.0 += 1;
                    } else {
                        guard.1 += 1;
                    }
                    guard.2 += 0.1;
                    drop(guard);
                    max_latency = max_latency.max(iter_start.elapsed());
                }
                max_latency
            })
        })
        .collect();

    let max_mutex_latency = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .max()
        .unwrap();

    println!("Mutex-based counter:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_mutex_latency);

    // Lock-free counter
    let atomic_counter = Arc::new(AtomicOrderCounter::new());
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let counter = Arc::clone(&atomic_counter);
            thread::spawn(move || {
                let mut max_latency = Duration::ZERO;
                for i in 0..ITERATIONS / THREADS as u64 {
                    let iter_start = Instant::now();
                    if i % 2 == 0 {
                        counter.record_buy(0.1);
                    } else {
                        counter.record_sell(0.1);
                    }
                    max_latency = max_latency.max(iter_start.elapsed());
                }
                max_latency
            })
        })
        .collect();

    let max_atomic_latency = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .max()
        .unwrap();

    println!("\nLock-free counter:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_atomic_latency);

    let (buys, sells, volume) = atomic_counter.stats();
    println!("  Final stats: {} buys, {} sells, {:.2} volume", buys, sells, volume);
}

fn main() {
    benchmark_lock_free();
}
```

## Strategy 3: Avoiding System Calls in Hot Paths

System calls can cause unpredictable latency spikes. Minimize them in critical paths:

```rust
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

/// Cached timestamp to avoid system calls
struct CachedTimestamp {
    cached_nanos: AtomicU64,
    last_update: AtomicU64,
}

impl CachedTimestamp {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        CachedTimestamp {
            cached_nanos: AtomicU64::new(now),
            last_update: AtomicU64::new(now),
        }
    }

    /// Fast path: return cached timestamp
    fn now_fast(&self) -> u64 {
        self.cached_nanos.load(Ordering::Relaxed)
    }

    /// Update cache (call periodically from a background thread)
    fn update(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        self.cached_nanos.store(now, Ordering::Relaxed);
        self.last_update.store(now, Ordering::Relaxed);
    }
}

/// Trade log that batches writes to avoid I/O syscalls
struct BatchedTradeLog {
    buffer: Vec<TradeRecord>,
    capacity: usize,
}

#[derive(Clone)]
struct TradeRecord {
    timestamp: u64,
    symbol_id: u32,
    price: f64,
    quantity: f64,
    side: u8,
}

impl BatchedTradeLog {
    fn new(batch_size: usize) -> Self {
        BatchedTradeLog {
            buffer: Vec::with_capacity(batch_size),
            capacity: batch_size,
        }
    }

    /// Fast path: add to buffer, no syscalls
    fn log(&mut self, record: TradeRecord) -> bool {
        if self.buffer.len() >= self.capacity {
            return false; // Buffer full, need to flush
        }
        self.buffer.push(record);
        true
    }

    /// Slow path: flush buffer (call from background thread)
    fn flush(&mut self) -> Vec<TradeRecord> {
        std::mem::take(&mut self.buffer)
    }

    fn pending_count(&self) -> usize {
        self.buffer.len()
    }
}

fn benchmark_syscall_avoidance() {
    println!("=== System Call Avoidance ===\n");

    const ITERATIONS: usize = 1_000_000;

    // Direct system calls (slow, unpredictable)
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut timestamps = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        timestamps.push(ts);
        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("Direct SystemTime calls:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);
    std::hint::black_box(timestamps);

    // Cached timestamp (fast, consistent)
    let cached_ts = CachedTimestamp::new();
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut timestamps = Vec::with_capacity(ITERATIONS);

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        // Update cache occasionally (simulating background thread)
        if i % 10000 == 0 {
            cached_ts.update();
        }

        let ts = cached_ts.now_fast();
        timestamps.push(ts);
        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("\nCached timestamp:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);
    std::hint::black_box(timestamps);

    // Batched logging
    let mut log = BatchedTradeLog::new(1000);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut flushes = 0;

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        let record = TradeRecord {
            timestamp: i as u64,
            symbol_id: 1,
            price: 50000.0,
            quantity: 0.1,
            side: (i % 2) as u8,
        };

        if !log.log(record.clone()) {
            // Flush would happen here (in real code, send to background thread)
            let _records = log.flush();
            flushes += 1;
            log.log(record);
        }

        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("\nBatched logging:");
    println!("  Total time: {:?}", start.elapsed());
    println!("  Max latency: {:?}", max_latency);
    println!("  Flushes: {}", flushes);
}

fn main() {
    benchmark_syscall_avoidance();
}
```

## Strategy 4: CPU Affinity and NUMA Awareness

For ultra-low latency, bind threads to specific CPU cores:

```rust
use std::thread;
use std::time::{Duration, Instant};

/// Trading engine with CPU affinity considerations
struct TradingEngine {
    core_id: usize,
    name: String,
}

impl TradingEngine {
    fn new(name: &str, core_id: usize) -> Self {
        TradingEngine {
            core_id,
            name: name.to_string(),
        }
    }

    /// Simulate setting CPU affinity
    /// In real code, use core_affinity crate or libc directly
    fn describe_affinity(&self) {
        println!(
            "Engine '{}' should be pinned to core {}",
            self.name, self.core_id
        );

        // Example using core_affinity crate (pseudocode):
        // let core_ids = core_affinity::get_core_ids().unwrap();
        // if let Some(core_id) = core_ids.get(self.core_id) {
        //     core_affinity::set_for_current(*core_id);
        // }
    }

    fn process_orders(&self, count: usize) -> Duration {
        let start = Instant::now();

        for i in 0..count {
            // Simulate order processing
            let price = 50000.0 + (i as f64 * 0.01);
            let quantity = 0.1;
            let _value = price * quantity;
            std::hint::black_box(_value);
        }

        start.elapsed()
    }
}

/// NUMA-aware memory allocation strategy
struct NumaAwareBuffer {
    // In real code, use numa crate for NUMA-local allocation
    data: Vec<u8>,
    numa_node: usize,
}

impl NumaAwareBuffer {
    fn new(size: usize, numa_node: usize) -> Self {
        println!("Allocating {} bytes on NUMA node {}", size, numa_node);

        // In real code:
        // let ptr = numa::alloc_onnode(size, numa_node);
        // let data = unsafe { Vec::from_raw_parts(ptr, size, size) };

        NumaAwareBuffer {
            data: vec![0u8; size],
            numa_node,
        }
    }

    fn write(&mut self, offset: usize, value: u8) {
        if offset < self.data.len() {
            self.data[offset] = value;
        }
    }

    fn read(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0)
    }
}

/// Demonstrate cache-friendly data layout
#[repr(C)]
struct CacheFriendlyOrder {
    // Hot data (accessed frequently) - fits in one cache line (64 bytes)
    price: f64,         // 8 bytes
    quantity: f64,      // 8 bytes
    timestamp: u64,     // 8 bytes
    order_id: u64,      // 8 bytes
    symbol_id: u32,     // 4 bytes
    side: u8,           // 1 byte
    order_type: u8,     // 1 byte
    _padding: [u8; 26], // Padding to 64 bytes
}

#[repr(C)]
struct CacheUnfriendlyOrder {
    // Cold data mixed with hot data - poor cache utilization
    order_id: u64,
    notes: [u8; 256],      // Rarely accessed
    price: f64,
    customer_data: [u8; 128], // Rarely accessed
    quantity: f64,
    audit_log: [u8; 512],  // Rarely accessed
    timestamp: u64,
}

fn benchmark_cache_layout() {
    println!("=== Cache-Friendly Data Layout ===\n");

    const ITERATIONS: usize = 1_000_000;

    // Cache-friendly layout
    let orders: Vec<CacheFriendlyOrder> = (0..1000)
        .map(|i| CacheFriendlyOrder {
            price: 50000.0 + i as f64,
            quantity: 0.1,
            timestamp: i as u64,
            order_id: i as u64,
            symbol_id: 1,
            side: (i % 2) as u8,
            order_type: 0,
            _padding: [0; 26],
        })
        .collect();

    let start = Instant::now();
    let mut sum = 0.0f64;

    for _ in 0..ITERATIONS {
        for order in &orders {
            sum += order.price * order.quantity;
        }
    }

    println!("Cache-friendly layout:");
    println!("  Time: {:?}", start.elapsed());
    println!("  Structure size: {} bytes", std::mem::size_of::<CacheFriendlyOrder>());
    std::hint::black_box(sum);

    // Note: Cache-unfriendly benchmark omitted for brevity
    // The unfriendly layout would be significantly slower due to cache misses

    println!("\nKey insight: Keep hot data together in cache lines!");
}

fn main() {
    println!("=== CPU Affinity and NUMA Awareness ===\n");

    // Create engines for different cores
    let market_data_engine = TradingEngine::new("MarketData", 0);
    let order_engine = TradingEngine::new("OrderProcessing", 1);
    let risk_engine = TradingEngine::new("RiskManagement", 2);

    // Describe affinity settings
    market_data_engine.describe_affinity();
    order_engine.describe_affinity();
    risk_engine.describe_affinity();

    println!("\n--- Running benchmarks ---\n");

    // Benchmark order processing
    let duration = order_engine.process_orders(100_000);
    println!("Order processing (100k orders): {:?}", duration);

    // NUMA-aware buffer demo
    let mut buffer = NumaAwareBuffer::new(1024 * 1024, 0);
    buffer.write(0, 42);
    println!("NUMA buffer read: {}", buffer.read(0));

    println!();
    benchmark_cache_layout();
}
```

## Strategy 5: Measuring and Monitoring p99

You can't optimize what you don't measure. Here's a comprehensive latency monitoring system:

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// High-precision latency histogram using HDR Histogram principles
struct LatencyHistogram {
    // Buckets for different latency ranges (in nanoseconds)
    // 0-1us, 1-10us, 10-100us, 100us-1ms, 1-10ms, 10-100ms, 100ms+
    buckets: [AtomicU64; 7],
    total_count: AtomicU64,
    min_ns: AtomicU64,
    max_ns: AtomicU64,
    sum_ns: AtomicU64,
}

impl LatencyHistogram {
    fn new() -> Self {
        LatencyHistogram {
            buckets: [
                AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            total_count: AtomicU64::new(0),
            min_ns: AtomicU64::new(u64::MAX),
            max_ns: AtomicU64::new(0),
            sum_ns: AtomicU64::new(0),
        }
    }

    fn record(&self, latency: Duration) {
        let nanos = latency.as_nanos() as u64;

        // Update bucket
        let bucket_idx = match nanos {
            0..=1_000 => 0,           // 0-1us
            1_001..=10_000 => 1,       // 1-10us
            10_001..=100_000 => 2,     // 10-100us
            100_001..=1_000_000 => 3,  // 100us-1ms
            1_000_001..=10_000_000 => 4, // 1-10ms
            10_000_001..=100_000_000 => 5, // 10-100ms
            _ => 6,                     // 100ms+
        };

        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
        self.sum_ns.fetch_add(nanos, Ordering::Relaxed);

        // Update min/max (using compare-and-swap for thread safety)
        let mut current_min = self.min_ns.load(Ordering::Relaxed);
        while nanos < current_min {
            match self.min_ns.compare_exchange_weak(
                current_min, nanos, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        let mut current_max = self.max_ns.load(Ordering::Relaxed);
        while nanos > current_max {
            match self.max_ns.compare_exchange_weak(
                current_max, nanos, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    fn report(&self) {
        let total = self.total_count.load(Ordering::Relaxed);
        if total == 0 {
            println!("No data recorded");
            return;
        }

        let bucket_names = [
            "0-1us", "1-10us", "10-100us", "100us-1ms",
            "1-10ms", "10-100ms", "100ms+"
        ];

        println!("Latency Distribution:");
        println!("-".repeat(50));

        let mut cumulative = 0u64;
        for (i, name) in bucket_names.iter().enumerate() {
            let count = self.buckets[i].load(Ordering::Relaxed);
            cumulative += count;
            let pct = count as f64 / total as f64 * 100.0;
            let cumulative_pct = cumulative as f64 / total as f64 * 100.0;

            let bar_len = (pct / 2.0) as usize;
            let bar: String = "#".repeat(bar_len);

            println!(
                "{:>12}: {:>8} ({:>5.1}%) [p{:>5.1}] {}",
                name, count, pct, cumulative_pct, bar
            );
        }

        println!("-".repeat(50));

        let min_us = self.min_ns.load(Ordering::Relaxed) as f64 / 1000.0;
        let max_us = self.max_ns.load(Ordering::Relaxed) as f64 / 1000.0;
        let avg_us = (self.sum_ns.load(Ordering::Relaxed) as f64 / total as f64) / 1000.0;

        println!("Statistics:");
        println!("  Count: {}", total);
        println!("  Min:   {:.2}us", min_us);
        println!("  Max:   {:.2}us", max_us);
        println!("  Avg:   {:.2}us", avg_us);
    }
}

/// Latency monitor for different operations
struct OperationMonitor {
    histograms: HashMap<String, Arc<LatencyHistogram>>,
}

impl OperationMonitor {
    fn new() -> Self {
        OperationMonitor {
            histograms: HashMap::new(),
        }
    }

    fn get_or_create(&mut self, operation: &str) -> Arc<LatencyHistogram> {
        self.histograms
            .entry(operation.to_string())
            .or_insert_with(|| Arc::new(LatencyHistogram::new()))
            .clone()
    }

    fn report_all(&self) {
        for (name, histogram) in &self.histograms {
            println!("\n=== {} ===", name);
            histogram.report();
        }
    }
}

/// Latency-aware order processor
struct OrderProcessor {
    histogram: Arc<LatencyHistogram>,
}

impl OrderProcessor {
    fn new(histogram: Arc<LatencyHistogram>) -> Self {
        OrderProcessor { histogram }
    }

    fn process(&self, order_id: u64) {
        let start = Instant::now();

        // Simulate order processing
        let price = 50000.0 + (order_id % 1000) as f64;
        let quantity = 0.1 * (order_id % 10) as f64;
        let _value = price * quantity;
        std::hint::black_box(_value);

        // Occasional slow operation
        if order_id % 100 == 0 {
            std::thread::sleep(Duration::from_micros(50));
        }

        self.histogram.record(start.elapsed());
    }
}

fn main() {
    println!("=== Latency Monitoring System ===\n");

    let mut monitor = OperationMonitor::new();

    // Create processors for different operations
    let order_histogram = monitor.get_or_create("OrderProcessing");
    let market_histogram = monitor.get_or_create("MarketDataUpdate");

    let order_processor = OrderProcessor::new(order_histogram);

    // Process orders
    println!("Processing 10,000 orders...\n");
    for i in 0..10_000 {
        order_processor.process(i);
    }

    // Simulate market data updates
    let market_hist = market_histogram;
    for i in 0..5_000 {
        let start = Instant::now();

        // Simulate market data processing
        let _price = 50000.0 + (i as f64 * 0.001).sin() * 100.0;
        std::hint::black_box(_price);

        if i % 200 == 0 {
            std::thread::sleep(Duration::from_micros(100));
        }

        market_hist.record(start.elapsed());
    }

    // Print reports
    monitor.report_all();

    println!("\n=== Key Takeaways ===");
    println!("1. Monitor p99 continuously in production");
    println!("2. Set alerts for p99 threshold breaches");
    println!("3. Investigate any p99 > 10x p50");
    println!("4. Track percentile trends over time");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Tail Latency** | The latency at high percentiles (p99, p99.9) |
| **p99** | 99% of requests are faster than this value |
| **Pre-allocation** | Allocate memory upfront to avoid allocation spikes |
| **Object Pooling** | Reuse objects instead of allocating new ones |
| **Lock-free** | Data structures that avoid mutex contention |
| **CPU Affinity** | Binding threads to specific CPU cores |
| **NUMA** | Non-Uniform Memory Access awareness |
| **Cache Line** | 64-byte block of CPU cache |
| **Hot/Cold Split** | Separate frequently and rarely accessed data |

## Practical Exercises

1. **Latency Tracker**: Create a system that:
   - Records latency for each order type separately
   - Calculates running percentiles (p50, p90, p99)
   - Detects latency anomalies in real-time
   - Exports metrics for visualization

2. **Object Pool Implementation**: Build a generic object pool:
   - Thread-safe acquire and release
   - Automatic pool expansion when exhausted
   - Statistics on pool utilization
   - Memory-efficient storage

3. **Lock-free Order Queue**: Implement an MPSC queue:
   - Multiple producers (market data feeds)
   - Single consumer (order processor)
   - Bounded capacity with backpressure
   - Latency measurement per operation

4. **Cache-optimized Order Book**: Design an order book:
   - Cache-friendly data layout
   - Minimal allocations during updates
   - Pre-allocated price levels
   - Benchmark against naive implementation

## Homework

1. **Complete p99 Optimization System**: Build a trading system that:
   - Measures latency at every stage of order processing
   - Implements all 5 optimization strategies from this chapter
   - Compares p99 before and after each optimization
   - Generates a detailed report with graphs
   - Achieves p99 < 2x p50 for order processing

2. **Latency Budget Allocator**: Create a tool that:
   - Defines latency budgets for each component
   - Monitors actual latency against budgets
   - Alerts when components exceed their budget
   - Suggests optimizations based on patterns
   - Tracks budget utilization over time

3. **Adaptive Batch Processor**: Implement a processor that:
   - Dynamically adjusts batch sizes based on latency
   - Balances throughput vs latency
   - Maintains p99 within target threshold
   - Handles load spikes gracefully
   - Reports efficiency metrics

4. **Memory-Mapped Order Journal**: Build a journal that:
   - Uses memory-mapped files for persistence
   - Achieves consistent write latency
   - Supports crash recovery
   - Benchmarks p99 write latency
   - Compares with traditional file I/O

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../333-*/en.md)
