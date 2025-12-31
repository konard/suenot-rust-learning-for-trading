# Day 324: Custom Allocators

## Trading Analogy

Imagine you're running a high-frequency trading firm with thousands of trades per second. A standard banking service processes each transaction individually — with verification, logging, and queuing. It's reliable, but slow.

Now imagine you create **your own internal settlement center**:
- **Capital pools** — pre-reserved money for fast settlements
- **Trading arena** — a memory region where all session trades live together
- **Fast operations stack** — instant allocation and deallocation for temporary calculations

**Custom allocator** in Rust is your own settlement center for memory:
- **Standard allocator** = bank with general rules
- **Pool allocator** = reserved capital for fast operations
- **Arena allocator** = trading session where all objects are freed at once
- **Bump allocator** = stack for ultra-fast temporary calculations

In high-frequency trading, every microsecond counts. The standard allocator can cause unpredictable pauses due to memory fragmentation or system calls. Custom allocators provide control and predictability.

## Why Custom Allocators?

| Problem | Solution | Trading Example |
|---------|----------|-----------------|
| **Fragmentation** | Pool of fixed blocks | Orders of the same size |
| **Allocation pauses** | Pre-allocated arena | All session trades |
| **Slow deallocation** | Bump allocator | Temporary indicator calculations |
| **Debugging leaks** | Tracing allocator | Memory monitoring in production |
| **Determinism** | Static buffer | Critical execution paths |

## Basic Allocator Structure in Rust

Rust provides the `GlobalAlloc` trait for creating custom allocators:

```rust
use std::alloc::{GlobalAlloc, Layout};

/// Trait for global allocator
unsafe trait GlobalAlloc {
    /// Allocates memory
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;

    /// Deallocates memory
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    /// Reallocates memory (optional)
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Default implementation
    }

    /// Allocates zero-initialized memory (optional)
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Default implementation
    }
}
```

### Minimal Example: Wrapper over System

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Allocator with counters for trading system
struct TradingAllocator {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl TradingAllocator {
    const fn new() -> Self {
        TradingAllocator {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    fn stats(&self) -> (usize, usize, usize, usize) {
        (
            self.allocations.load(Ordering::Relaxed),
            self.deallocations.load(Ordering::Relaxed),
            self.bytes_allocated.load(Ordering::Relaxed),
            self.peak_usage.load(Ordering::Relaxed),
        )
    }
}

unsafe impl GlobalAlloc for TradingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            self.allocations.fetch_add(1, Ordering::Relaxed);
            let new_total = self.bytes_allocated.fetch_add(layout.size(), Ordering::Relaxed)
                + layout.size();

            // Update peak usage
            let mut current_peak = self.peak_usage.load(Ordering::Relaxed);
            while new_total > current_peak {
                match self.peak_usage.compare_exchange_weak(
                    current_peak,
                    new_total,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => current_peak = p,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_sub(layout.size(), Ordering::Relaxed);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: TradingAllocator = TradingAllocator::new();

fn main() {
    // Simulate trading activity
    let mut orders: Vec<String> = Vec::new();

    for i in 0..1000 {
        orders.push(format!("Order-{}-BTCUSDT", i));
    }

    let (allocs, deallocs, bytes, peak) = ALLOCATOR.stats();
    println!("=== Allocator Statistics ===");
    println!("Allocations: {}", allocs);
    println!("Deallocations: {}", deallocs);
    println!("Current usage: {} KB", bytes / 1024);
    println!("Peak usage: {} KB", peak / 1024);
}
```

## Pool Allocator: Fixed Block Pool

Ideal for objects of the same size — like orders:

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Block size in the pool (enough for a typical order)
const BLOCK_SIZE: usize = 128;
/// Number of blocks in the pool
const POOL_SIZE: usize = 10000;

/// Block in the pool
#[repr(C, align(16))]
struct PoolBlock {
    data: [u8; BLOCK_SIZE],
}

/// Free list node
struct FreeNode {
    next: AtomicPtr<FreeNode>,
}

/// Pool allocator for orders
struct OrderPoolAllocator {
    // Static buffer
    pool: UnsafeCell<[PoolBlock; POOL_SIZE]>,
    // Free list head
    free_list: AtomicPtr<FreeNode>,
    // Statistics
    allocations: AtomicUsize,
    pool_hits: AtomicUsize,
    pool_misses: AtomicUsize,
}

unsafe impl Sync for OrderPoolAllocator {}

impl OrderPoolAllocator {
    const fn new() -> Self {
        OrderPoolAllocator {
            pool: UnsafeCell::new([PoolBlock { data: [0; BLOCK_SIZE] }; POOL_SIZE]),
            free_list: AtomicPtr::new(std::ptr::null_mut()),
            allocations: AtomicUsize::new(0),
            pool_hits: AtomicUsize::new(0),
            pool_misses: AtomicUsize::new(0),
        }
    }

    /// Initialize the free list
    unsafe fn init(&self) {
        let pool = &mut *self.pool.get();

        for i in 0..POOL_SIZE - 1 {
            let current = &mut pool[i] as *mut PoolBlock as *mut FreeNode;
            let next = &mut pool[i + 1] as *mut PoolBlock as *mut FreeNode;
            (*current).next = AtomicPtr::new(next);
        }

        let last = &mut pool[POOL_SIZE - 1] as *mut PoolBlock as *mut FreeNode;
        (*last).next = AtomicPtr::new(std::ptr::null_mut());

        self.free_list.store(&mut pool[0] as *mut PoolBlock as *mut FreeNode, Ordering::Release);
    }

    /// Check if pointer belongs to the pool
    fn is_from_pool(&self, ptr: *mut u8) -> bool {
        let pool_start = self.pool.get() as *mut u8;
        let pool_end = unsafe { pool_start.add(POOL_SIZE * std::mem::size_of::<PoolBlock>()) };
        ptr >= pool_start && ptr < pool_end
    }

    fn stats(&self) -> (usize, usize, usize) {
        (
            self.allocations.load(Ordering::Relaxed),
            self.pool_hits.load(Ordering::Relaxed),
            self.pool_misses.load(Ordering::Relaxed),
        )
    }
}

unsafe impl GlobalAlloc for OrderPoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::Relaxed);

        // If size fits the pool
        if layout.size() <= BLOCK_SIZE && layout.align() <= 16 {
            // Try to take block from free list (lock-free)
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                if head.is_null() {
                    break; // Pool is empty
                }

                let next = (*head).next.load(Ordering::Relaxed);

                if self.free_list.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    self.pool_hits.fetch_add(1, Ordering::Relaxed);
                    return head as *mut u8;
                }
            }
        }

        // Fallback to system allocator
        self.pool_misses.fetch_add(1, Ordering::Relaxed);
        std::alloc::System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // If block is from pool — return to free list
        if self.is_from_pool(ptr) {
            let node = ptr as *mut FreeNode;
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                (*node).next = AtomicPtr::new(head);

                if self.free_list.compare_exchange_weak(
                    head,
                    node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    return;
                }
            }
        }

        // Otherwise deallocate through system allocator
        std::alloc::System.dealloc(ptr, layout);
    }
}

// Usage example
fn main() {
    #[repr(C)]
    struct Order {
        id: u64,
        symbol: [u8; 16],
        price: f64,
        quantity: f64,
        side: u8,
        _padding: [u8; 7],
    }

    println!("Order size: {} bytes", std::mem::size_of::<Order>());

    // In real code:
    // static ALLOCATOR: OrderPoolAllocator = OrderPoolAllocator::new();
    // unsafe { ALLOCATOR.init(); }

    // Simulate high-frequency trading
    let mut orders = Vec::with_capacity(1000);

    for i in 0..1000 {
        orders.push(Box::new(Order {
            id: i,
            symbol: *b"BTCUSDT\0\0\0\0\0\0\0\0\0",
            price: 50000.0 + i as f64,
            quantity: 0.01,
            side: if i % 2 == 0 { 0 } else { 1 },
            _padding: [0; 7],
        }));
    }

    println!("Created {} orders", orders.len());
}
```

## Arena Allocator: Session Memory

Arena allocates memory sequentially and frees everything at once at the end of the session:

```rust
use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::ptr::NonNull;

/// Arena for trading session
struct TradingArena {
    /// Memory buffer
    buffer: UnsafeCell<Vec<u8>>,
    /// Current position
    offset: UnsafeCell<usize>,
    /// Capacity
    capacity: usize,
}

impl TradingArena {
    fn new(capacity: usize) -> Self {
        TradingArena {
            buffer: UnsafeCell::new(vec![0u8; capacity]),
            offset: UnsafeCell::new(0),
            capacity,
        }
    }

    /// Allocates memory in the arena
    fn alloc<T>(&self) -> Option<NonNull<T>> {
        let layout = Layout::new::<T>();
        self.alloc_layout(layout).map(|ptr| ptr.cast())
    }

    fn alloc_layout(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            let offset = &mut *self.offset.get();
            let buffer = &mut *self.buffer.get();

            // Alignment
            let align_offset = (*offset).wrapping_neg() & (layout.align() - 1);
            let new_offset = *offset + align_offset + layout.size();

            if new_offset > self.capacity {
                return None; // Arena is full
            }

            let ptr = buffer.as_mut_ptr().add(*offset + align_offset);
            *offset = new_offset;

            NonNull::new(ptr)
        }
    }

    /// Allocates memory for a slice
    fn alloc_slice<T>(&self, count: usize) -> Option<&mut [T]> {
        let layout = Layout::array::<T>(count).ok()?;
        let ptr = self.alloc_layout(layout)?;

        unsafe {
            Some(std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, count))
        }
    }

    /// Resets the arena for reuse
    fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Used memory
    fn used(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    /// Remaining memory
    fn remaining(&self) -> usize {
        self.capacity - self.used()
    }
}

// Example: trading session with arena
#[derive(Debug, Clone, Copy)]
struct Trade {
    timestamp: u64,
    price: f64,
    quantity: f64,
    is_buy: bool,
}

#[derive(Debug, Clone, Copy)]
struct PriceLevel {
    price: f64,
    volume: f64,
}

fn main() {
    // Create 1 MB arena for trading session
    let arena = TradingArena::new(1024 * 1024);

    println!("=== Trading Session ===");
    println!("Arena capacity: {} KB", arena.capacity / 1024);

    // Allocate array for trades
    let trades: &mut [Trade] = arena.alloc_slice(10000).expect("Out of memory");

    // Fill with data
    for (i, trade) in trades.iter_mut().enumerate() {
        *trade = Trade {
            timestamp: 1700000000000 + i as u64,
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.001),
            is_buy: i % 2 == 0,
        };
    }

    println!("Recorded {} trades", trades.len());
    println!("Used: {} KB", arena.used() / 1024);
    println!("Remaining: {} KB", arena.remaining() / 1024);

    // Allocate price levels for order book
    let bid_levels: &mut [PriceLevel] = arena.alloc_slice(100).expect("Out of memory");
    let ask_levels: &mut [PriceLevel] = arena.alloc_slice(100).expect("Out of memory");

    // Initialize order book
    for (i, level) in bid_levels.iter_mut().enumerate() {
        *level = PriceLevel {
            price: 50000.0 - i as f64,
            volume: 1.0 + i as f64 * 0.1,
        };
    }

    for (i, level) in ask_levels.iter_mut().enumerate() {
        *level = PriceLevel {
            price: 50000.0 + i as f64,
            volume: 1.0 + i as f64 * 0.1,
        };
    }

    println!("Order book: {} bid / {} ask levels", bid_levels.len(), ask_levels.len());
    println!("Total used: {} KB", arena.used() / 1024);

    // Calculate VWAP
    let vwap: f64 = trades.iter()
        .map(|t| t.price * t.quantity)
        .sum::<f64>()
        / trades.iter().map(|t| t.quantity).sum::<f64>();

    println!("VWAP: {:.2}", vwap);

    // End of session — reset arena
    arena.reset();
    println!("\n=== Session Ended ===");
    println!("Arena reset, used: {} bytes", arena.used());
}
```

## Bump Allocator: Ultra-Fast Allocation

Bump allocator is the simplest and fastest allocator:

```rust
use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::alloc::Layout;

/// Lifetime marker for safe borrowing
struct BumpMarker<'a> {
    _phantom: PhantomData<&'a ()>,
}

/// Ultra-fast bump allocator
struct BumpAllocator<'a> {
    start: *mut u8,
    end: *mut u8,
    ptr: Cell<*mut u8>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> BumpAllocator<'a> {
    /// Creates allocator from buffer
    fn from_slice(buffer: &'a mut [u8]) -> Self {
        let start = buffer.as_mut_ptr();
        let end = unsafe { start.add(buffer.len()) };

        BumpAllocator {
            start,
            end,
            ptr: Cell::new(start),
            _marker: PhantomData,
        }
    }

    /// Allocates memory
    fn alloc<T>(&self, value: T) -> Option<&'a mut T> {
        let layout = Layout::new::<T>();

        let current = self.ptr.get();

        // Align
        let aligned = (current as usize + layout.align() - 1) & !(layout.align() - 1);
        let new_ptr = aligned + layout.size();

        if new_ptr > self.end as usize {
            return None;
        }

        self.ptr.set(new_ptr as *mut u8);

        unsafe {
            let ptr = aligned as *mut T;
            ptr.write(value);
            Some(&mut *ptr)
        }
    }

    /// Allocates memory for a slice
    fn alloc_slice<T: Copy>(&self, values: &[T]) -> Option<&'a mut [T]> {
        let layout = Layout::array::<T>(values.len()).ok()?;

        let current = self.ptr.get();
        let aligned = (current as usize + layout.align() - 1) & !(layout.align() - 1);
        let new_ptr = aligned + layout.size();

        if new_ptr > self.end as usize {
            return None;
        }

        self.ptr.set(new_ptr as *mut u8);

        unsafe {
            let ptr = aligned as *mut T;
            std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len());
            Some(std::slice::from_raw_parts_mut(ptr, values.len()))
        }
    }

    /// Used memory
    fn used(&self) -> usize {
        self.ptr.get() as usize - self.start as usize
    }

    /// Remaining memory
    fn remaining(&self) -> usize {
        self.end as usize - self.ptr.get() as usize
    }

    /// Reset (frees everything at once)
    fn reset(&self) {
        self.ptr.set(self.start);
    }
}

// Example: fast indicator calculations
fn main() {
    // Static buffer for temporary calculations
    let mut buffer = [0u8; 64 * 1024]; // 64 KB

    let bump = BumpAllocator::from_slice(&mut buffer);

    println!("=== Fast Indicator Calculation ===");

    // Input data — prices
    let prices = [
        50000.0, 50100.0, 50050.0, 50200.0, 50150.0,
        50300.0, 50250.0, 50400.0, 50350.0, 50500.0,
    ];

    // Allocate memory for intermediate calculations
    let prices_copy = bump.alloc_slice(&prices).unwrap();
    println!("Prices copy: {:?}", &prices_copy[..5]);

    // Calculate returns
    let returns: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    let returns_slice = bump.alloc_slice(&returns).unwrap();
    println!("Returns: {:?}", &returns_slice[..5]);

    // SMA (Simple Moving Average)
    let period = 5;
    let mut sma_values = Vec::new();

    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma_values.push(sum / period as f64);
    }

    let sma = bump.alloc_slice(&sma_values).unwrap();
    println!("SMA(5): {:?}", sma);

    // Volatility (standard deviation)
    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / (returns.len() - 1) as f64;
    let volatility = variance.sqrt() * (252.0_f64).sqrt(); // Annualized volatility

    let vol = bump.alloc(volatility).unwrap();
    println!("Annualized volatility: {:.2}%", vol * 100.0);

    println!("\nMemory used: {} bytes", bump.used());
    println!("Remaining: {} bytes", bump.remaining());

    // Reset for next calculation
    bump.reset();
    println!("After reset: {} bytes", bump.used());
}
```

## Allocator Comparison

| Allocator | alloc Speed | dealloc Speed | Fragmentation | Use Case |
|-----------|-------------|---------------|---------------|----------|
| **System** | Medium | Medium | Yes | General purpose |
| **Pool** | Fast | Fast | No* | Same-size objects |
| **Arena** | Very fast | Instant (bulk) | No | Session data |
| **Bump** | Instant | Impossible | No | Temporary calculations |

*when using same-size blocks

## Practical Example: Trading Engine with Custom Allocators

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// Allocator metrics for production
struct AllocatorMetrics {
    total_allocations: AtomicU64,
    total_deallocations: AtomicU64,
    current_bytes: AtomicUsize,
    peak_bytes: AtomicUsize,
    alloc_time_ns: AtomicU64,
    dealloc_time_ns: AtomicU64,
}

impl AllocatorMetrics {
    const fn new() -> Self {
        AllocatorMetrics {
            total_allocations: AtomicU64::new(0),
            total_deallocations: AtomicU64::new(0),
            current_bytes: AtomicUsize::new(0),
            peak_bytes: AtomicUsize::new(0),
            alloc_time_ns: AtomicU64::new(0),
            dealloc_time_ns: AtomicU64::new(0),
        }
    }
}

/// Production-ready allocator for trading engine
struct TradingEngineAllocator {
    metrics: AllocatorMetrics,
}

impl TradingEngineAllocator {
    const fn new() -> Self {
        TradingEngineAllocator {
            metrics: AllocatorMetrics::new(),
        }
    }

    fn print_stats(&self) {
        let allocs = self.metrics.total_allocations.load(Ordering::Relaxed);
        let deallocs = self.metrics.total_deallocations.load(Ordering::Relaxed);
        let current = self.metrics.current_bytes.load(Ordering::Relaxed);
        let peak = self.metrics.peak_bytes.load(Ordering::Relaxed);
        let alloc_time = self.metrics.alloc_time_ns.load(Ordering::Relaxed);
        let dealloc_time = self.metrics.dealloc_time_ns.load(Ordering::Relaxed);

        println!("=== Trading Engine Allocator Stats ===");
        println!("Total allocations:   {}", allocs);
        println!("Total deallocations: {}", deallocs);
        println!("Outstanding:         {}", allocs - deallocs);
        println!("Current memory:      {} KB", current / 1024);
        println!("Peak memory:         {} KB", peak / 1024);
        if allocs > 0 {
            println!("Avg alloc time:      {} ns", alloc_time / allocs);
        }
        if deallocs > 0 {
            println!("Avg dealloc time:    {} ns", dealloc_time / deallocs);
        }
    }
}

unsafe impl GlobalAlloc for TradingEngineAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();

        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            self.metrics.total_allocations.fetch_add(1, Ordering::Relaxed);

            let new_size = self.metrics.current_bytes.fetch_add(layout.size(), Ordering::Relaxed)
                + layout.size();

            // Update peak
            let mut current_peak = self.metrics.peak_bytes.load(Ordering::Relaxed);
            while new_size > current_peak {
                match self.metrics.peak_bytes.compare_exchange_weak(
                    current_peak,
                    new_size,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(p) => current_peak = p,
                }
            }
        }

        let elapsed = start.elapsed().as_nanos() as u64;
        self.metrics.alloc_time_ns.fetch_add(elapsed, Ordering::Relaxed);

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let start = Instant::now();

        self.metrics.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.metrics.current_bytes.fetch_sub(layout.size(), Ordering::Relaxed);

        System.dealloc(ptr, layout);

        let elapsed = start.elapsed().as_nanos() as u64;
        self.metrics.dealloc_time_ns.fetch_add(elapsed, Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: TradingEngineAllocator = TradingEngineAllocator::new();

// Trading engine simulation
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: Side,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Buy,
    Sell,
}

fn main() {
    println!("Starting trading engine...\n");

    let start = Instant::now();

    // Simulate high-frequency trading
    let mut orders: Vec<Order> = Vec::with_capacity(10000);

    for i in 0..10000 {
        orders.push(Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.0001),
            side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
        });
    }

    println!("Created {} orders in {:?}", orders.len(), start.elapsed());

    // Process orders
    let matched: Vec<_> = orders.iter()
        .filter(|o| matches!(o.side, Side::Buy) && o.price > 50500.0)
        .collect();

    println!("Filled {} orders", matched.len());

    // Cleanup
    drop(orders);

    println!();
    ALLOCATOR.print_stats();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **GlobalAlloc** | Trait for creating global allocators |
| **Pool Allocator** | Pool of fixed blocks for same-size objects |
| **Arena Allocator** | Sequential allocation with bulk deallocation |
| **Bump Allocator** | Ultra-fast allocation without individual deallocation |
| **Lock-free** | Lock-free algorithms for multithreading |
| **Memory metrics** | Collecting statistics for production monitoring |

## Practical Exercises

1. **Thread-local arena**: Create an arena for each trading engine thread:
   - Each thread has its own arena
   - No locks during allocation
   - Reset at the end of each trading session
   - Per-thread usage metrics

2. **Slab allocator**: Implement a slab allocator:
   - Multiple pools for different object sizes
   - Automatic selection of appropriate pool
   - Fallback to system allocator for large objects
   - Size-based statistics collection

3. **Ring buffer allocator**: Create a circular allocator for data streams:
   - Fixed buffer size
   - FIFO semantics — old data gets overwritten
   - Optimized for tick data stream
   - No fragmentation

4. **Leak detector wrapper**: Develop a leak detector wrapper:
   - Wraps any allocator
   - Tracks all allocations with callstack
   - Reports unclosed allocations
   - CI/CD integration

## Homework

1. **Hybrid allocator**: Create an allocator for a trading bot:
   - Pool for orders (frequent, same size)
   - Arena for session data
   - System for rare large objects
   - Prometheus metrics for each type
   - Documentation with benchmarks

2. **Memory-mapped allocator**: Implement an mmap-based allocator:
   - Allocates memory via mmap for large blocks
   - Uses pools for small objects
   - Huge pages support for performance
   - Tests for alignment and correctness

3. **Deterministic allocator**: Create a deterministic allocator:
   - Predictable allocation time (no system calls)
   - Static pool with maximum size
   - Panic on overflow (fail-fast)
   - Ideal for trading engine hot path

4. **Allocation profiler**: Develop an allocation profiler:
   - Records all allocations with timestamps
   - Builds histogram by sizes
   - Identifies hot spots
   - Generates allocation flamegraph
   - Integration with existing trading bot

## Navigation

[← Previous day](../323-zero-copy-avoiding-copies/en.md) | [Next day →](../325-jemalloc-mimalloc/en.md)
