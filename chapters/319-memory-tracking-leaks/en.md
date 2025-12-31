# Day 319: Memory: Tracking Leaks

## Trading Analogy

Imagine you're managing an investment portfolio. Every day you buy new assets, but sometimes you forget to sell those that are no longer needed. Over time, the portfolio bloats: money is frozen in forgotten positions, liquidity drops, and maintenance costs rise.

**Memory leak** is the same problem, but with RAM:
- **Buying an asset** = allocating memory for an object
- **Selling an asset** = freeing memory
- **Forgotten position** = memory leak

In a trading system, this is critical:
- The bot runs 24/7, accumulating leaks
- Memory runs out → system crashes during trading
- Money loss due to missed trades

Tracking leaks is like a regular portfolio audit: finding "dead" positions and closing them.

## What is a Memory Leak in Rust?

Although Rust guarantees memory safety, leaks are still possible:

| Leak Type | Description | Trading Example |
|-----------|-------------|-----------------|
| **Cyclic references** | `Rc`/`Arc` form a cycle | Strategy references manager, manager references strategy |
| **Forgotten channels** | Sender/Receiver not closed | Price update channel without subscribers |
| **Infinitely growing collections** | HashMap/Vec without cleanup | Years of trade history in memory |
| **mem::forget** | Explicit Drop skip | Intentional resource leaking |

### Why Doesn't Rust Protect Against Leaks?

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct TradingStrategy {
    name: String,
    // Cyclic reference to manager
    manager: RefCell<Option<Rc<StrategyManager>>>,
}

struct StrategyManager {
    strategies: Vec<Rc<TradingStrategy>>,
}

fn create_leak() {
    let strategy = Rc::new(TradingStrategy {
        name: "Scalper".to_string(),
        manager: RefCell::new(None),
    });

    let manager = Rc::new(StrategyManager {
        strategies: vec![Rc::clone(&strategy)],
    });

    // Create cycle: strategy -> manager -> strategy
    *strategy.manager.borrow_mut() = Some(Rc::clone(&manager));

    // On exit: Rc count > 0, memory not freed!
    println!("Strategy: {}", strategy.name);
}

fn main() {
    create_leak();
    println!("Function ended, but memory not freed!");
}
```

## Tools for Tracking Leaks

### 1. Valgrind (Linux)

Classic tool for detecting memory leaks:

```rust
// trading_bot.rs
use std::collections::HashMap;

struct OrderBook {
    orders: HashMap<u64, Order>,
}

struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: HashMap::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.insert(order.id, order);
    }

    // Leak: forgot to implement removal of old orders!
    fn process_fills(&mut self, filled_ids: &[u64]) {
        for id in filled_ids {
            // Should be: self.orders.remove(id);
            // But we just log and forget to remove
            if let Some(order) = self.orders.get(id) {
                println!("Order {} filled: {} @ {}", id, order.symbol, order.price);
            }
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Simulation: add orders every second
    for i in 0..10000 {
        book.add_order(Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01,
        });

        // Orders "fill" but aren't removed
        if i > 0 && i % 100 == 0 {
            let filled: Vec<u64> = ((i - 100)..i).collect();
            book.process_fills(&filled);
        }
    }

    println!("Active orders: {}", book.orders.len());
    // Expect 100, but have 10000!
}
```

Running with Valgrind:
```bash
cargo build --release
valgrind --leak-check=full ./target/release/trading_bot
```

### 2. Heaptrack (Linux)

Memory usage visualization over time:

```bash
heaptrack ./target/release/trading_bot
heaptrack_gui heaptrack.trading_bot.*.gz
```

### 3. Instruments (macOS)

```bash
cargo build --release
instruments -t "Leaks" ./target/release/trading_bot
```

### 4. Built-in Allocator with Tracing

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocator with memory counting
struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn memory_stats() -> (usize, usize, usize) {
    let alloc = ALLOCATED.load(Ordering::SeqCst);
    let dealloc = DEALLOCATED.load(Ordering::SeqCst);
    (alloc, dealloc, alloc.saturating_sub(dealloc))
}

fn print_memory_stats(label: &str) {
    let (alloc, dealloc, in_use) = memory_stats();
    println!(
        "[{}] Allocated: {} KB, Deallocated: {} KB, In Use: {} KB",
        label,
        alloc / 1024,
        dealloc / 1024,
        in_use / 1024
    );
}

fn main() {
    print_memory_stats("Start");

    // Simulate trading activity
    let mut price_history: Vec<f64> = Vec::new();

    for i in 0..100_000 {
        price_history.push(50000.0 + (i as f64 * 0.01));

        if i % 10_000 == 0 {
            print_memory_stats(&format!("After {} prices", i));
        }
    }

    // Free history
    drop(price_history);
    print_memory_stats("After cleanup");
}
```

## Typical Leak Patterns in Trading Systems

### Pattern 1: Infinitely Growing Cache

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct PriceCache {
    cache: HashMap<String, Vec<(Instant, f64)>>,
    // No size limit!
}

impl PriceCache {
    fn new() -> Self {
        PriceCache {
            cache: HashMap::new(),
        }
    }

    // BAD: data accumulates infinitely
    fn add_price_bad(&mut self, symbol: &str, price: f64) {
        self.cache
            .entry(symbol.to_string())
            .or_insert_with(Vec::new)
            .push((Instant::now(), price));
    }
}

// FIX: bounded cache

struct BoundedPriceCache {
    cache: HashMap<String, Vec<(Instant, f64)>>,
    max_age: Duration,
    max_entries_per_symbol: usize,
}

impl BoundedPriceCache {
    fn new(max_age: Duration, max_entries: usize) -> Self {
        BoundedPriceCache {
            cache: HashMap::new(),
            max_age,
            max_entries_per_symbol: max_entries,
        }
    }

    fn add_price(&mut self, symbol: &str, price: f64) {
        let now = Instant::now();
        let prices = self.cache
            .entry(symbol.to_string())
            .or_insert_with(Vec::new);

        // Add new price
        prices.push((now, price));

        // Remove outdated entries
        prices.retain(|(time, _)| now.duration_since(*time) < self.max_age);

        // Limit size
        if prices.len() > self.max_entries_per_symbol {
            let drain_count = prices.len() - self.max_entries_per_symbol;
            prices.drain(..drain_count);
        }
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        for prices in self.cache.values_mut() {
            prices.retain(|(time, _)| now.duration_since(*time) < self.max_age);
        }
        // Remove empty entries
        self.cache.retain(|_, prices| !prices.is_empty());
    }
}

fn main() {
    let mut cache = BoundedPriceCache::new(
        Duration::from_secs(3600),  // Store prices for 1 hour
        1000,                        // Max 1000 entries per symbol
    );

    // Simulate data stream
    for i in 0..10000 {
        cache.add_price("BTCUSDT", 50000.0 + i as f64);

        if i % 1000 == 0 {
            cache.cleanup();
            println!("Cache size after cleanup: {} entries",
                     cache.cache.values().map(|v| v.len()).sum::<usize>());
        }
    }
}
```

### Pattern 2: Breaking Cycles with Weak

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Exchange {
    name: String,
    strategies: RefCell<Vec<Rc<Strategy>>>,
}

struct Strategy {
    name: String,
    // Use Weak instead of Rc to prevent cycle
    exchange: Weak<Exchange>,
}

impl Strategy {
    fn execute(&self) {
        // Try to get a strong reference
        if let Some(exchange) = self.exchange.upgrade() {
            println!("Strategy '{}' running on exchange '{}'",
                     self.name, exchange.name);
        } else {
            println!("Exchange no longer available");
        }
    }
}

impl Drop for Strategy {
    fn drop(&mut self) {
        println!("Strategy '{}' dropped", self.name);
    }
}

impl Drop for Exchange {
    fn drop(&mut self) {
        println!("Exchange '{}' dropped", self.name);
    }
}

fn main() {
    let exchange = Rc::new(Exchange {
        name: "Binance".to_string(),
        strategies: RefCell::new(Vec::new()),
    });

    let scalper = Rc::new(Strategy {
        name: "Scalper".to_string(),
        exchange: Rc::downgrade(&exchange),  // Weak reference
    });

    let arbitrage = Rc::new(Strategy {
        name: "Arbitrage".to_string(),
        exchange: Rc::downgrade(&exchange),  // Weak reference
    });

    exchange.strategies.borrow_mut().push(Rc::clone(&scalper));
    exchange.strategies.borrow_mut().push(Rc::clone(&arbitrage));

    scalper.execute();
    arbitrage.execute();

    println!("\nDropping strategy references...");
    drop(scalper);
    drop(arbitrage);

    println!("Exchange Rc count: {}", Rc::strong_count(&exchange));
    println!("\nDropping exchange...");
    drop(exchange);
    // Everything correctly freed!
}
```

### Pattern 3: Memory Tracking with RAII

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

static ACTIVE_ORDERS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_ORDERS_CREATED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct TrackedOrder {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl TrackedOrder {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        ACTIVE_ORDERS.fetch_add(1, Ordering::SeqCst);
        TOTAL_ORDERS_CREATED.fetch_add(1, Ordering::SeqCst);

        TrackedOrder {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
        }
    }
}

impl Drop for TrackedOrder {
    fn drop(&mut self) {
        ACTIVE_ORDERS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn get_order_stats() -> (usize, usize) {
    (
        ACTIVE_ORDERS.load(Ordering::SeqCst),
        TOTAL_ORDERS_CREATED.load(Ordering::SeqCst),
    )
}

fn main() {
    println!("=== Order Tracking Test ===\n");

    {
        let mut orders = Vec::new();

        for i in 0..1000 {
            orders.push(TrackedOrder::new(
                i,
                "BTCUSDT",
                50000.0 + i as f64,
                0.01,
            ));
        }

        let (active, total) = get_order_stats();
        println!("After creating 1000 orders:");
        println!("  Active: {}", active);
        println!("  Total created: {}", total);

        // Remove half of the orders
        orders.drain(..500);

        let (active, total) = get_order_stats();
        println!("\nAfter removing 500 orders:");
        println!("  Active: {}", active);
        println!("  Total created: {}", total);
    }

    let (active, total) = get_order_stats();
    println!("\nAfter exiting block:");
    println!("  Active: {}", active);
    println!("  Total created: {}", total);

    if active != 0 {
        println!("\n!!! MEMORY LEAK: {} orders not freed!", active);
    } else {
        println!("\nAll orders correctly freed!");
    }
}
```

## Advanced Tracking Techniques

### Allocation Profiling with dhat

```toml
# Cargo.toml
[dependencies]
dhat = "0.3"

[features]
dhat-heap = []
```

```rust
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Trading system code
    let mut prices: Vec<f64> = Vec::new();

    for i in 0..100_000 {
        prices.push(50000.0 + i as f64 * 0.01);
    }

    // On exit dhat will output a report
}
```

Running:
```bash
cargo run --release --features dhat-heap
```

### Integration with Prometheus Metrics

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MemoryMetrics {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
    high_water_mark: AtomicUsize,
}

impl MemoryMetrics {
    fn new() -> Self {
        MemoryMetrics {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            high_water_mark: AtomicUsize::new(0),
        }
    }

    fn record_alloc(&self, bytes: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let new_total = self.bytes_allocated.fetch_add(bytes, Ordering::Relaxed) + bytes;
        let current_in_use = new_total - self.bytes_deallocated.load(Ordering::Relaxed);

        // Update high water mark
        let mut current_hwm = self.high_water_mark.load(Ordering::Relaxed);
        while current_in_use > current_hwm {
            match self.high_water_mark.compare_exchange_weak(
                current_hwm,
                current_in_use,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new) => current_hwm = new,
            }
        }
    }

    fn record_dealloc(&self, bytes: usize) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_deallocated.fetch_add(bytes, Ordering::Relaxed);
    }

    fn report(&self) {
        let allocs = self.allocations.load(Ordering::Relaxed);
        let deallocs = self.deallocations.load(Ordering::Relaxed);
        let bytes_alloc = self.bytes_allocated.load(Ordering::Relaxed);
        let bytes_dealloc = self.bytes_deallocated.load(Ordering::Relaxed);
        let hwm = self.high_water_mark.load(Ordering::Relaxed);

        println!("=== Memory Report ===");
        println!("Total allocations: {}", allocs);
        println!("Total deallocations: {}", deallocs);
        println!("Outstanding allocations: {}", allocs - deallocs);
        println!("Bytes allocated: {} KB", bytes_alloc / 1024);
        println!("Bytes deallocated: {} KB", bytes_dealloc / 1024);
        println!("Current usage: {} KB", (bytes_alloc - bytes_dealloc) / 1024);
        println!("Peak usage: {} KB", hwm / 1024);
    }
}

fn main() {
    let metrics = Arc::new(MemoryMetrics::new());

    // Simulate trading system operation
    println!("Starting simulation...\n");

    // Simulate allocations
    for i in 0..1000 {
        let size = 1024 * ((i % 10) + 1);  // From 1KB to 10KB
        metrics.record_alloc(size);

        // Free 80% of allocations
        if i % 5 != 0 {
            metrics.record_dealloc(size);
        }
    }

    metrics.report();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Memory leak** | Allocated memory that is never freed |
| **Cyclic references** | Rc/Arc forming a cycle, preventing deallocation |
| **Weak** | Weak reference that doesn't affect the reference count |
| **Valgrind** | External tool for leak detection |
| **dhat** | Built-in allocation profiler for Rust |
| **RAII** | Pattern for automatic resource management via Drop |
| **Bounded cache** | Cache with size limit to prevent growth |
| **Memory metrics** | Collecting memory usage statistics |

## Practical Exercises

1. **Leak Detector**: Create a wrapper for `Box<T>` that:
   - Tracks all allocations and deallocations
   - Stores stack trace of creation location
   - On program exit, outputs list of non-freed objects
   - Shows where they were created

2. **Cycle Analyzer**: Write a utility that:
   - Builds a graph of connections between `Rc<T>` objects
   - Detects cyclic dependencies
   - Suggests replacement with `Weak<T>` where necessary
   - Generates graph visualization

3. **Real-time Monitoring**: Implement a monitoring system:
   - Tracks memory usage every second
   - Builds growth/decline graph
   - Warns on abnormal growth
   - Integrates with trading metrics

4. **Bounded Order Book**: Create an OrderBook with:
   - Automatic cleanup of old orders
   - Limit on number of orders per symbol
   - Memory usage metrics
   - Tests for leak absence

## Homework

1. **Memory Profiler for Trading Bot**: Develop a profiler that:
   - Integrates into an existing trading bot
   - Tracks allocations by category (orders, prices, indicators)
   - Generates reports every N seconds
   - Records memory usage history
   - Warns about potential leaks

2. **Stress-test for Leaks**: Write a test framework:
   - Runs the trading bot with different loads
   - Monitors memory over extended time
   - Determines if there is constant growth
   - Generates report with graphs
   - Automatically detects leaks

3. **Bounded Object Pool**: Implement a pool:
   - Reuses objects instead of constant allocations
   - Has maximum size limit
   - Automatically shrinks under low load
   - Collects efficiency metrics
   - Safe for multithreaded use

4. **CI Integration for Leak Checking**: Create a pipeline:
   - Runs tests with Valgrind/ASAN
   - Checks for leak absence
   - Blocks merge on detected problems
   - Generates reports for code review
   - Stores check history

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../320-*/en.md)
