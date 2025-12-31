# Day 325: jemalloc and mimalloc

## Trading Analogy

Imagine you're managing a warehouse for trading goods. You have two options for organizing your warehouse:

**Standard warehouse (system allocator)**:
- Universal, works for everything
- Sometimes workers spend time finding space for new items
- Queues form under high load
- Works well for small volumes

**Specialized warehouse (jemalloc/mimalloc)**:
- Optimized for specific types of goods
- Each worker has their own zone (thread-local cache)
- Minimal waiting even under high load
- Ideal for large-scale operations

In a trading system, this is critical:
- **High-frequency trading** — thousands of orders per second, every microsecond counts
- **Market data processing** — data streams require fast memory allocation
- **Multithreading** — each thread processes its own symbol or exchange

## What Are Alternative Allocators?

The standard memory allocator in Rust uses the system allocator (glibc malloc on Linux, Windows Heap on Windows). Alternative allocators provide improved characteristics:

| Allocator | Features | When to Use |
|-----------|----------|-------------|
| **System** | Universal, no extra dependencies | Small applications |
| **jemalloc** | Excellent multithreading, low fragmentation | Servers, long-running processes |
| **mimalloc** | Maximum speed, small size | HFT, microservices |
| **tcmalloc** | Thread-caching, profiling | Google-style infrastructure |

### jemalloc

Developed by Facebook for server applications:
- **Thread-local caches** — each thread has its own cache for fast allocations
- **Arena-based allocation** — memory is divided into arenas to minimize contention
- **Low fragmentation** — efficient reuse of freed memory
- **Introspection** — built-in tools for memory usage analysis

### mimalloc

Developed by Microsoft Research:
- **Segment-based** — memory is organized into segments by object size
- **Free list sharding** — distributed free lists
- **Very low overhead** — minimal metadata per allocation
- **Excellent scalability** — nearly linear performance with thread count

## Integrating jemalloc

### Cargo.toml

```toml
[dependencies]
tikv-jemallocator = "0.5"

[features]
default = []
jemalloc = ["tikv-jemallocator"]
```

### Using in Code

```rust
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::collections::HashMap;
use std::time::Instant;

/// Order book for a trading system
struct OrderBook {
    bids: HashMap<u64, Order>,
    asks: HashMap<u64, Order>,
}

#[derive(Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    symbol: String,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    fn add_bid(&mut self, order: Order) {
        self.bids.insert(order.id, order);
    }

    fn add_ask(&mut self, order: Order) {
        self.asks.insert(order.id, order);
    }

    fn remove_order(&mut self, id: u64) -> bool {
        self.bids.remove(&id).is_some() || self.asks.remove(&id).is_some()
    }
}

fn benchmark_order_operations(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut book = OrderBook::new();

    for i in 0..iterations {
        // Add orders
        book.add_bid(Order {
            id: i as u64,
            price: 50000.0 + (i as f64 * 0.01),
            quantity: 0.1,
            symbol: "BTCUSDT".to_string(),
        });

        book.add_ask(Order {
            id: (i + iterations) as u64,
            price: 50001.0 + (i as f64 * 0.01),
            quantity: 0.1,
            symbol: "BTCUSDT".to_string(),
        });

        // Remove some orders
        if i > 100 {
            book.remove_order((i - 100) as u64);
        }
    }

    start.elapsed()
}

fn main() {
    println!("=== Allocator Performance Test ===\n");

    #[cfg(feature = "jemalloc")]
    println!("Using: jemalloc");

    #[cfg(not(feature = "jemalloc"))]
    println!("Using: system allocator");

    let iterations = 100_000;

    // Warmup
    let _ = benchmark_order_operations(1000);

    // Measurement
    let duration = benchmark_order_operations(iterations);

    println!("\nResults:");
    println!("  Iterations: {}", iterations);
    println!("  Time: {:?}", duration);
    println!("  Operations/sec: {:.0}", iterations as f64 / duration.as_secs_f64());
}
```

Running:
```bash
# With system allocator
cargo run --release

# With jemalloc
cargo run --release --features jemalloc
```

## Integrating mimalloc

### Cargo.toml

```toml
[dependencies]
mimalloc = "0.1"

[features]
default = []
mimalloc = ["mimalloc"]
```

### Using in Code

```rust
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Price data for market data feed
#[derive(Clone)]
struct PriceTick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// Market data stream processor
struct MarketDataProcessor {
    buffer: Vec<PriceTick>,
    capacity: usize,
}

impl MarketDataProcessor {
    fn new(capacity: usize) -> Self {
        MarketDataProcessor {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn process_tick(&mut self, tick: PriceTick) {
        if self.buffer.len() >= self.capacity {
            // Clear old data
            self.buffer.drain(..self.capacity / 2);
        }
        self.buffer.push(tick);
    }

    fn get_latest_price(&self, symbol: &str) -> Option<(f64, f64)> {
        self.buffer
            .iter()
            .rev()
            .find(|t| t.symbol == symbol)
            .map(|t| (t.bid, t.ask))
    }
}

fn benchmark_multithreaded(threads: usize, ticks_per_thread: usize) -> std::time::Duration {
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let mut processor = MarketDataProcessor::new(10000);

                for i in 0..ticks_per_thread {
                    let tick = PriceTick {
                        symbol: format!("SYM{}", i % 100),
                        bid: 100.0 + (i as f64 * 0.001),
                        ask: 100.01 + (i as f64 * 0.001),
                        timestamp: i as u64,
                    };
                    processor.process_tick(tick);
                }

                processor.buffer.len()
            })
        })
        .collect();

    let total_processed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let duration = start.elapsed();

    println!(
        "  Threads: {}, Processed ticks: {}",
        threads, total_processed
    );

    duration
}

fn main() {
    println!("=== Multithreaded mimalloc Test ===\n");

    #[cfg(feature = "mimalloc")]
    println!("Using: mimalloc");

    #[cfg(not(feature = "mimalloc"))]
    println!("Using: system allocator");

    let ticks_per_thread = 100_000;

    for threads in [1, 2, 4, 8] {
        println!("\nTest with {} threads:", threads);
        let duration = benchmark_multithreaded(threads, ticks_per_thread);
        let total_ticks = threads * ticks_per_thread;
        println!(
            "  Time: {:?}, Ticks/sec: {:.0}",
            duration,
            total_ticks as f64 / duration.as_secs_f64()
        );
    }
}
```

## Performance Comparison

### Benchmark for Trading Systems

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::collections::HashMap;

// Allocation counter for analysis
struct CountingAllocator<A: GlobalAlloc> {
    inner: A,
}

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES_ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        self.inner.dealloc(ptr, layout)
    }
}

fn reset_counters() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    DEALLOC_COUNT.store(0, Ordering::Relaxed);
    BYTES_ALLOCATED.store(0, Ordering::Relaxed);
}

fn get_stats() -> (usize, usize, usize) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        DEALLOC_COUNT.load(Ordering::Relaxed),
        BYTES_ALLOCATED.load(Ordering::Relaxed),
    )
}

/// Trading engine simulation
struct TradingEngine {
    positions: HashMap<String, f64>,
    order_history: Vec<TradeOrder>,
    price_cache: HashMap<String, Vec<f64>>,
}

#[derive(Clone)]
struct TradeOrder {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine {
            positions: HashMap::new(),
            order_history: Vec::new(),
            price_cache: HashMap::new(),
        }
    }

    fn process_order(&mut self, order: TradeOrder) {
        // Update position
        let position = self.positions.entry(order.symbol.clone()).or_insert(0.0);
        match order.side {
            OrderSide::Buy => *position += order.quantity,
            OrderSide::Sell => *position -= order.quantity,
        }

        // Cache price
        self.price_cache
            .entry(order.symbol.clone())
            .or_insert_with(Vec::new)
            .push(order.price);

        // Save to history
        self.order_history.push(order);

        // Clean old price cache (limit size)
        for prices in self.price_cache.values_mut() {
            if prices.len() > 1000 {
                prices.drain(..500);
            }
        }
    }

    fn get_position(&self, symbol: &str) -> f64 {
        *self.positions.get(symbol).unwrap_or(&0.0)
    }
}

fn run_trading_simulation(orders: usize) -> (std::time::Duration, (usize, usize, usize)) {
    reset_counters();

    let start = Instant::now();
    let mut engine = TradingEngine::new();

    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT"];

    for i in 0..orders {
        let order = TradeOrder {
            id: i as u64,
            symbol: symbols[i % symbols.len()].to_string(),
            side: if i % 2 == 0 {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            },
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.001) % 1.0,
        };
        engine.process_order(order);
    }

    let duration = start.elapsed();
    let stats = get_stats();

    (duration, stats)
}

fn main() {
    println!("=== Allocator Performance Comparison ===\n");

    let test_sizes = [10_000, 50_000, 100_000];

    for &size in &test_sizes {
        println!("Test with {} orders:", size);

        let (duration, (allocs, deallocs, bytes)) = run_trading_simulation(size);

        println!("  Time: {:?}", duration);
        println!("  Allocations: {}", allocs);
        println!("  Deallocations: {}", deallocs);
        println!("  Bytes allocated: {} KB", bytes / 1024);
        println!(
            "  Orders/sec: {:.0}",
            size as f64 / duration.as_secs_f64()
        );
        println!();
    }
}
```

## Advanced Configuration

### Configuring jemalloc via Environment Variables

```bash
# Enable profiling
export MALLOC_CONF="prof:true,prof_prefix:jeprof.out"

# Configure arena count
export MALLOC_CONF="narenas:4"

# Enable statistics
export MALLOC_CONF="stats_print:true"
```

### Programmatic jemalloc Configuration

```rust
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(feature = "jemalloc")]
use tikv_jemalloc_ctl::{epoch, stats};

fn print_jemalloc_stats() {
    #[cfg(feature = "jemalloc")]
    {
        // Update statistics
        epoch::advance().unwrap();

        let allocated = stats::allocated::read().unwrap();
        let resident = stats::resident::read().unwrap();

        println!("=== jemalloc Statistics ===");
        println!("  Allocated: {} KB", allocated / 1024);
        println!("  Resident memory: {} KB", resident / 1024);
        println!("  Fragmentation: {:.1}%",
                 (resident - allocated) as f64 / resident as f64 * 100.0);
    }

    #[cfg(not(feature = "jemalloc"))]
    println!("jemalloc not enabled");
}

fn main() {
    println!("=== Trading System Memory Monitoring ===\n");

    // Simulate work
    let mut data: Vec<Vec<f64>> = Vec::new();

    for i in 0..100 {
        data.push((0..10000).map(|x| x as f64 * 0.001).collect());

        if i % 10 == 0 {
            print_jemalloc_stats();
            // Free some memory
            if data.len() > 50 {
                data.drain(..25);
            }
        }
    }

    println!("\nFinal statistics:");
    print_jemalloc_stats();
}
```

### Configuring mimalloc

```rust
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    // mimalloc is configured via environment variables:
    // MIMALLOC_VERBOSE=1 - enable logs
    // MIMALLOC_SHOW_STATS=1 - show statistics on exit
    // MIMALLOC_ARENA_EAGER_COMMIT=1 - eager arena allocation
    // MIMALLOC_LARGE_OS_PAGES=1 - use huge pages

    println!("=== mimalloc Test ===\n");

    let mut buffers: Vec<Vec<u8>> = Vec::new();

    // Create load with different allocation sizes
    for size in [64, 256, 1024, 4096, 16384, 65536] {
        println!("Allocations of {} bytes:", size);

        for _ in 0..1000 {
            buffers.push(vec![0u8; size]);
        }

        println!("  Created {} buffers", buffers.len());

        // Free half
        buffers.truncate(buffers.len() / 2);
    }

    println!("\nFinal buffer count: {}", buffers.len());
}
```

## Choosing an Allocator for Trading Systems

### Selection Recommendations

```rust
/// Allocator selection based on use case
///
/// | Scenario                    | Recommended Allocator |
/// |-----------------------------|----------------------|
/// | HFT (< 1ms latency)         | mimalloc             |
/// | Market data processing      | jemalloc             |
/// | Backtesting (lots of memory)| jemalloc             |
/// | Microservices               | mimalloc             |
/// | Long-running processes      | jemalloc             |
/// | Embedded/Edge               | mimalloc (smaller)   |

// Example of conditional compilation for different environments
#[cfg(all(feature = "mimalloc", target_os = "linux"))]
use mimalloc::MiMalloc;

#[cfg(all(feature = "jemalloc", target_os = "linux"))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "mimalloc", target_os = "linux"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(all(feature = "jemalloc", target_os = "linux", not(feature = "mimalloc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn get_allocator_name() -> &'static str {
    #[cfg(feature = "mimalloc")]
    return "mimalloc";

    #[cfg(all(feature = "jemalloc", not(feature = "mimalloc")))]
    return "jemalloc";

    #[cfg(not(any(feature = "mimalloc", feature = "jemalloc")))]
    return "system";
}

fn main() {
    println!("Active allocator: {}", get_allocator_name());
}
```

### Benchmark for Choosing the Optimal Allocator

```rust
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Benchmark of allocation patterns in trading systems
struct AllocationBenchmark {
    small_allocs: u64,    // < 256 bytes (orders, ticks)
    medium_allocs: u64,   // 256 - 4KB (message buffers)
    large_allocs: u64,    // > 4KB (historical data)
}

impl AllocationBenchmark {
    fn new() -> Self {
        AllocationBenchmark {
            small_allocs: 0,
            medium_allocs: 0,
            large_allocs: 0,
        }
    }

    fn run_small_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut items: Vec<Box<[u8; 64]>> = Vec::new();
        for _ in 0..count {
            items.push(Box::new([0u8; 64]));
        }

        // Random deallocation
        for i in (0..items.len()).step_by(3) {
            if i < items.len() {
                items.swap_remove(i.min(items.len() - 1));
            }
        }

        self.small_allocs = start.elapsed().as_nanos() as u64;
    }

    fn run_medium_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut buffers: Vec<Vec<u8>> = Vec::new();
        for i in 0..count {
            buffers.push(vec![0u8; 256 + (i % 3840)]);
        }

        // FIFO deallocation (typical for message buffers)
        while buffers.len() > count / 2 {
            buffers.remove(0);
        }

        self.medium_allocs = start.elapsed().as_nanos() as u64;
    }

    fn run_large_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut data: Vec<Vec<f64>> = Vec::new();
        for _ in 0..count {
            // Historical data: 1000 candles
            data.push(vec![0.0; 1000]);
        }

        // Clean old data
        data.retain(|v| v.len() > 500);

        self.large_allocs = start.elapsed().as_nanos() as u64;
    }

    fn report(&self) {
        println!("  Small allocations: {} µs", self.small_allocs / 1000);
        println!("  Medium allocations: {} µs", self.medium_allocs / 1000);
        println!("  Large allocations: {} µs", self.large_allocs / 1000);
        println!("  Total: {} µs",
                 (self.small_allocs + self.medium_allocs + self.large_allocs) / 1000);
    }
}

fn run_benchmark() {
    let mut bench = AllocationBenchmark::new();

    bench.run_small_allocations(50_000);
    bench.run_medium_allocations(10_000);
    bench.run_large_allocations(1_000);

    bench.report();
}

fn run_multithreaded_benchmark(threads: usize) {
    let start = Instant::now();
    let total_ops = Arc::new(AtomicU64::new(0));

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let ops = Arc::clone(&total_ops);
            thread::spawn(move || {
                let mut local_ops = 0u64;

                for _ in 0..10_000 {
                    // Typical pattern: create-process-delete
                    let order = Box::new([0u8; 128]);
                    let _ = order.len();
                    drop(order);
                    local_ops += 1;
                }

                ops.fetch_add(local_ops, Ordering::Relaxed);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let duration = start.elapsed();
    let ops = total_ops.load(Ordering::Relaxed);

    println!(
        "  {} threads: {} ops, {:?}, {:.0} ops/sec",
        threads,
        ops,
        duration,
        ops as f64 / duration.as_secs_f64()
    );
}

fn main() {
    println!("=== Benchmark for Allocator Selection ===\n");

    println!("Single-threaded test:");
    run_benchmark();

    println!("\nMulti-threaded test:");
    for threads in [1, 2, 4, 8] {
        run_multithreaded_benchmark(threads);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **jemalloc** | Allocator with thread-local caches and arenas for server applications |
| **mimalloc** | Fast allocator from Microsoft with low overhead |
| **Thread-local cache** | Local cache for each thread, minimizing locks |
| **Arena** | Isolated memory region for a group of allocations |
| **Fragmentation** | Inefficient memory use due to scattered free blocks |
| **Global allocator** | Rust attribute for replacing the standard allocator |

## Practical Exercises

1. **Comparative Benchmark**: Create a program that:
   - Measures performance of system, jemalloc, and mimalloc allocators
   - Tests different allocation patterns (small, medium, large)
   - Generates a report with recommendations
   - Accounts for multithreading

2. **Memory Monitoring**: Implement a monitoring system that:
   - Tracks memory usage in real-time
   - Detects fragmentation
   - Warns about potential problems
   - Integrates with trading metrics

3. **Adaptive Allocator**: Write a wrapper that:
   - Switches between allocators based on load
   - Optimizes for specific usage patterns
   - Collects statistics for analysis
   - Provides an API for configuration

4. **Trading System Profiling**: Create a tool that:
   - Profiles allocations by system components
   - Identifies hotspots
   - Suggests optimizations
   - Generates visualizations

## Homework

1. **HFT Bot Optimization**: Take an existing trading bot and:
   - Measure baseline performance
   - Try jemalloc and mimalloc
   - Find the optimal configuration
   - Document results with graphs
   - Achieve at least 20% latency improvement

2. **Memory Pool for Orders**: Implement an object pool:
   - Pre-allocates N Order objects
   - Reuses objects instead of allocating/deallocating
   - Measure the performance difference
   - Compare with alternative allocators
   - Determine the break-even point (when pool is more efficient)

3. **CI Pipeline with Benchmarks**: Create a pipeline that:
   - Runs benchmarks on each PR
   - Compares with baseline
   - Warns about performance regressions
   - Stores history of results
   - Generates reports for code review

4. **Hybrid Allocation Strategy**: Develop a system that:
   - Uses different allocators for different data types
   - Hot path uses mimalloc
   - Cold data uses jemalloc
   - Automatically profiles and adapts
   - Documents trade-offs

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../326-*/en.md)
