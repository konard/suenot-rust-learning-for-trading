# Day 308: Micro-Optimizations: Allocations

## Trading Analogy

Imagine a trader who constantly transfers money between different exchanges for every trade. Even if the strategy is profitable, the overhead costs of transfers eat up all the profit. A 0.1% transfer fee seems trivial, but with 10,000 operations per day, it becomes serious losses.

In programming, **memory allocation** is like transferring funds between accounts. Every time we create a `Vec`, `String`, `Box`, or `HashMap`, Rust calls the system allocator, which takes time. For most programs this is negligible, but in high-frequency trading (HFT) where we count microseconds, unnecessary allocations can cost money.

## Why are allocations important in algorithmic trading?

In trading systems, **latency** is critical — the time between receiving market data and sending an order. Even a 100-microsecond delay can mean losing to competitors.

| Operation | Typical Time | Impact |
|-----------|-------------|---------|
| Heap allocation (malloc) | ~100-500 ns | Slows processing |
| Data copying | ~10-50 ns/KB | Grows with size |
| Stack operations | ~1-5 ns | Almost invisible |
| System call | ~1-10 μs | Very expensive |

**Problems with excessive allocations:**
1. **Latency spikes** — unpredictable delays
2. **CPU cache misses** — data scattered in memory
3. **Memory fragmentation** — inefficient usage
4. **Garbage collection pauses** (not in Rust, but concept is important)

## Where do allocations happen in Rust?

```rust
// ❌ Heap allocations
let prices = Vec::new();           // Allocation on push
let symbol = String::from("BTCUSDT"); // String allocation
let data = Box::new(MarketData {});   // Heap allocation
let map = HashMap::new();           // Allocation + internal buffers

// ✅ Stack data (fast)
let price = 42000.0;
let array = [0.0; 100];  // Fixed size
let tuple = (price, quantity);
```

## Example: Measuring Allocations

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocator wrapper for counting allocations
struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn get_allocation_stats() -> (usize, usize, usize) {
    let allocated = ALLOCATED.load(Ordering::SeqCst);
    let deallocated = DEALLOCATED.load(Ordering::SeqCst);
    let count = ALLOCATION_COUNT.load(Ordering::SeqCst);
    (allocated, deallocated, count)
}

fn reset_allocation_stats() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
    ALLOCATION_COUNT.store(0, Ordering::SeqCst);
}

// Structure for storing price and volume
#[derive(Debug, Clone)]
struct PriceLevel {
    price: f64,
    quantity: f64,
}

// ❌ Non-optimal version: many allocations
fn process_orderbook_slow(levels: &[(f64, f64)]) -> Vec<PriceLevel> {
    let mut result = Vec::new(); // Allocation 1

    for &(price, qty) in levels {
        let level = PriceLevel { // Each iteration — potential allocation
            price,
            quantity: qty,
        };
        result.push(level); // May trigger reallocation
    }

    result
}

// ✅ Optimized version: one allocation
fn process_orderbook_fast(levels: &[(f64, f64)]) -> Vec<PriceLevel> {
    let mut result = Vec::with_capacity(levels.len()); // Exact size

    for &(price, qty) in levels {
        result.push(PriceLevel { price, quantity: qty });
    }

    result
}

fn main() {
    let levels = vec![
        (42000.0, 1.5),
        (42001.0, 2.3),
        (42002.0, 0.8),
        (42003.0, 1.2),
        (42004.0, 3.1),
    ];

    println!("=== Allocation Comparison ===\n");

    // Test non-optimal version
    reset_allocation_stats();
    let _ = process_orderbook_slow(&levels);
    let (alloc1, dealloc1, count1) = get_allocation_stats();
    println!("❌ Slow version (without with_capacity):");
    println!("   Allocated: {} bytes", alloc1);
    println!("   Number of allocations: {}", count1);

    // Test optimized version
    reset_allocation_stats();
    let _ = process_orderbook_fast(&levels);
    let (alloc2, dealloc2, count2) = get_allocation_stats();
    println!("\n✅ Fast version (with with_capacity):");
    println!("   Allocated: {} bytes", alloc2);
    println!("   Number of allocations: {}", count2);

    println!("\n📊 Difference:");
    println!("   Allocation savings: {:.1}x", count1 as f64 / count2 as f64);
}
```

## Optimization 1: Pre-allocating Memory

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}

// ❌ Bad: growing allocations
fn collect_trades_bad(count: usize) -> Vec<Trade> {
    let mut trades = Vec::new(); // Capacity = 0

    for i in 0..count {
        trades.push(Trade {  // Reallocation at 1, 2, 4, 8, 16...
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }

    trades
}

// ✅ Good: one allocation
fn collect_trades_good(count: usize) -> Vec<Trade> {
    let mut trades = Vec::with_capacity(count); // Exact size

    for i in 0..count {
        trades.push(Trade {
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }

    trades
}

// ✅ Even better: buffer reuse
fn collect_trades_reuse(count: usize, buffer: &mut Vec<Trade>) {
    buffer.clear(); // Doesn't deallocate memory
    buffer.reserve(count); // Expands capacity if needed

    for i in 0..count {
        buffer.push(Trade {
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }
}

fn main() {
    println!("=== Pre-allocation Strategies ===\n");

    let count = 1000;

    // Version with reallocations
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let trades1 = collect_trades_bad(count);
    let time1 = start.elapsed();
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ Without capacity:");
    println!("   Time: {:?}", time1);
    println!("   Allocations: {}", count1);

    // Version with with_capacity
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let trades2 = collect_trades_good(count);
    let time2 = start.elapsed();
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ With with_capacity:");
    println!("   Time: {:?}", time2);
    println!("   Allocations: {}", count2);

    // Version with buffer reuse
    let mut buffer = Vec::new();
    reset_allocation_stats();
    let start = std::time::Instant::now();
    collect_trades_reuse(count, &mut buffer);
    let time3 = start.elapsed();
    let (alloc3, _, count3) = get_allocation_stats();
    println!("\n✅ With buffer reuse:");
    println!("   Time: {:?}", time3);
    println!("   Allocations: {}", count3);

    println!("\n📊 Speedup: {:.2}x", time1.as_nanos() as f64 / time3.as_nanos() as f64);
}
```

## Optimization 2: Avoiding String Cloning

```rust
use std::borrow::Cow;

#[derive(Debug)]
struct Order<'a> {
    symbol: Cow<'a, str>,  // Copy-on-write: borrows or owns
    price: f64,
    quantity: f64,
}

// ❌ Bad: clones string every time
fn create_order_bad(symbol: &str, price: f64, qty: f64) -> Order<'static> {
    Order {
        symbol: Cow::Owned(symbol.to_string()), // Allocation
        price,
        quantity: qty,
    }
}

// ✅ Good: borrows string
fn create_order_good(symbol: &str, price: f64, qty: f64) -> Order {
    Order {
        symbol: Cow::Borrowed(symbol), // No allocation
        price,
        quantity: qty,
    }
}

// ✅ Smart solution: clones only when modifying
fn normalize_symbol(symbol: &str) -> Cow<str> {
    if symbol.contains('-') {
        // Need to modify — clone
        Cow::Owned(symbol.replace('-', ""))
    } else {
        // No changes — borrow
        Cow::Borrowed(symbol)
    }
}

fn main() {
    println!("=== String Optimization with Cow ===\n");

    let symbol = "BTCUSDT";

    // Bad variant
    reset_allocation_stats();
    let order1 = create_order_bad(symbol, 42000.0, 1.0);
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ With cloning: {} allocations, {} bytes", count1, alloc1);

    // Good variant
    reset_allocation_stats();
    let order2 = create_order_good(symbol, 42000.0, 1.0);
    let (alloc2, _, count2) = get_allocation_stats();
    println!("✅ With borrowing: {} allocations, {} bytes", count2, alloc2);

    println!("\n=== Smart Cloning ===");

    reset_allocation_stats();
    let normalized1 = normalize_symbol("BTCUSDT");
    let (a1, _, c1) = get_allocation_stats();
    println!("Without '-': {} allocations (borrowing)", c1);

    reset_allocation_stats();
    let normalized2 = normalize_symbol("BTC-USDT");
    let (a2, _, c2) = get_allocation_stats();
    println!("With '-': {} allocations (cloning for modification)", c2);
}
```

## Optimization 3: Object Pools

```rust
use std::collections::VecDeque;

/// Pool for buffer reuse
struct BufferPool {
    pool: VecDeque<Vec<f64>>,
    capacity: usize,
}

impl BufferPool {
    fn new(capacity: usize) -> Self {
        BufferPool {
            pool: VecDeque::new(),
            capacity,
        }
    }

    /// Take buffer from pool or create new
    fn acquire(&mut self) -> Vec<f64> {
        self.pool.pop_front().unwrap_or_else(|| Vec::with_capacity(self.capacity))
    }

    /// Return buffer to pool
    fn release(&mut self, mut buffer: Vec<f64>) {
        buffer.clear(); // Clear data but keep capacity
        if self.pool.len() < 10 { // Limit pool size
            self.pool.push_back(buffer);
        }
        // Otherwise buffer will be dropped
    }
}

/// Calculate simple moving average
fn calculate_sma(prices: &[f64], period: usize, buffer: &mut Vec<f64>) -> f64 {
    buffer.clear();
    buffer.extend_from_slice(&prices[prices.len() - period..]);
    buffer.iter().sum::<f64>() / period as f64
}

fn main() {
    let prices: Vec<f64> = (0..1000).map(|i| 42000.0 + i as f64 * 0.5).collect();
    let iterations = 10000;

    println!("=== Comparison with and without pool ===\n");

    // ❌ Without pool: new allocation each time
    reset_allocation_stats();
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut buffer = Vec::new();
        let _sma = calculate_sma(&prices, 20, &mut buffer);
    }
    let time1 = start.elapsed();
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ Without pool:");
    println!("   Time: {:?}", time1);
    println!("   Allocations: {}", count1);

    // ✅ With pool: reuse buffers
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let mut pool = BufferPool::new(100);
    for _ in 0..iterations {
        let mut buffer = pool.acquire();
        let _sma = calculate_sma(&prices, 20, &mut buffer);
        pool.release(buffer);
    }
    let time2 = start.elapsed();
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ With pool:");
    println!("   Time: {:?}", time2);
    println!("   Allocations: {}", count2);

    println!("\n📊 Speedup: {:.2}x", time1.as_nanos() as f64 / time2.as_nanos() as f64);
    println!("📊 Allocation reduction: {:.1}%", (1.0 - count2 as f64 / count1 as f64) * 100.0);
}
```

## Optimization 4: SmallVec — Hybrid Storage

```rust
// Requires: cargo add smallvec
use smallvec::{SmallVec, smallvec};

/// Processing small order lists
#[derive(Debug)]
struct OrderUpdate {
    // Up to 4 elements on stack, more — on heap
    price_levels: SmallVec<[(f64, f64); 4]>,
}

impl OrderUpdate {
    fn new() -> Self {
        OrderUpdate {
            price_levels: smallvec![],
        }
    }

    fn add_level(&mut self, price: f64, qty: f64) {
        self.price_levels.push((price, qty));
    }

    fn total_volume(&self) -> f64 {
        self.price_levels.iter().map(|(_, qty)| qty).sum()
    }
}

fn main() {
    println!("=== SmallVec: stack for small data ===\n");

    // Small volume — stays on stack
    reset_allocation_stats();
    let mut update1 = OrderUpdate::new();
    update1.add_level(42000.0, 1.0);
    update1.add_level(42001.0, 2.0);
    let (alloc1, _, count1) = get_allocation_stats();
    println!("✅ 2 elements (stack):");
    println!("   Allocations: {}", count1);
    println!("   Volume: {}", update1.total_volume());

    // More than 4 elements — moves to heap
    reset_allocation_stats();
    let mut update2 = OrderUpdate::new();
    for i in 0..10 {
        update2.add_level(42000.0 + i as f64, 1.0);
    }
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ 10 elements (heap after 4th):");
    println!("   Allocations: {}", count2);
    println!("   Volume: {}", update2.total_volume());

    println!("\n📝 SmallVec automatically switches between stack and heap");
}
```

## Practical Recommendations

### When to optimize allocations?

1. **Measure first** — use profilers (perf, valgrind, heaptrack)
2. **Hot paths** — optimize code executed thousands of times per second
3. **Latency-critical** — HFT, real-time market data processing
4. **Don't optimize prematurely** — readability is more important than micro-optimizations

### Allocation Optimization Checklist

```rust
// ✅ Good practices
Vec::with_capacity(n)        // Pre-allocation
HashMap::with_capacity(n)    // Same for maps
String::with_capacity(n)     // And for strings
buffer.clear()               // Reuse instead of new Vec::new()
&str instead of String       // Borrowing instead of ownership
Cow<str>                     // Copy-on-write
SmallVec                     // Stack for small data
arrayvec                     // Fixed-size Vec on stack
```

### Analysis Tools

```bash
# Allocation profiling
cargo install cargo-flamegraph
cargo flamegraph --bin my_trading_bot

# Heap usage analysis
valgrind --tool=massif ./target/release/my_trading_bot
ms_print massif.out.*

# Allocation counting
cargo add dhat
# Add #[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Heap allocation** | Memory allocation through system allocator (slow) |
| **Stack allocation** | Fast memory allocation on stack |
| **with_capacity** | Pre-allocation for Vec/HashMap/String |
| **Buffer reuse** | Reusing buffers via .clear() |
| **Cow** | Copy-on-write: borrows or clones as needed |
| **Object pool** | Object pool for reuse |
| **SmallVec** | Stack/heap hybrid for small collections |
| **Zero-copy** | Data processing without copying |

## Homework

1. **Allocation Profiling**: Write a program that:
   - Processes market data stream (1000 updates/sec)
   - Calculates top-10 active instruments
   - Measure allocations before and after optimization
   - Use custom allocator for counting

2. **Object Pool for Orders**: Implement object pool:
   - Pool of `Order` structs with capacity 1000
   - acquire/release methods
   - Automatic pool expansion when needed
   - Usage statistics (hit rate, miss rate)

3. **Zero-allocation Parser**: Create JSON market data parser:
   - Uses `&str` instead of `String` where possible
   - SmallVec for arrays
   - Reuses buffers between calls
   - Compare with naive version

4. **Benchmark Comparison**: Test performance:
   - `Vec` vs `SmallVec` vs `ArrayVec`
   - `String` vs `&str` vs `Cow<str>`
   - HashMap with and without `with_capacity`
   - Use criterion for accurate measurements

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
