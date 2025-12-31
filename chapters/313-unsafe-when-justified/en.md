# Day 313: unsafe: When Justified

## Trading Analogy

Imagine an automated trading system with two operating modes:

**Safe Mode:**
- All trades go through a verification system
- Position limits are checked
- Orders are validated before sending
- Risk is controlled at every step
- Slower, but guaranteed to be safe

**Direct Access Mode (unsafe):**
- Direct interaction with exchange API without intermediate checks
- Orders sent directly to the execution queue
- No additional validations — maximum speed
- Used in high-frequency trading (HFT)
- One mistake = money loss

In Rust, `unsafe` is exactly this direct access mode. The compiler trusts the programmer to know what they're doing and removes some safety checks. It's like removing a climber's safety rope — gives freedom of movement but requires absolute confidence in every step.

## What is unsafe in Rust?

Rust guarantees memory safety at compile time through the ownership system, lifetime checking, and borrowing rules. But sometimes you need to do what the compiler cannot verify:

### Five superpowers of unsafe:

1. **Dereferencing raw pointers** (`*const T`, `*mut T`)
2. **Calling unsafe functions and methods**
3. **Accessing or modifying mutable static variables**
4. **Implementing unsafe traits**
5. **Accessing fields of unions**

```rust
fn main() {
    let x = 42;
    let raw_ptr = &x as *const i32;

    // Safe: creating a raw pointer
    println!("Created pointer: {:p}", raw_ptr);

    // UNSAFE: dereferencing a raw pointer
    unsafe {
        println!("Value at pointer: {}", *raw_ptr);
    }
}
```

## When is unsafe justified?

### 1. Interacting with FFI (Foreign Function Interface)

When working with external C/C++ libraries for high-performance computations:

```rust
// Linking C library for technical analysis
#[link(name = "ta_lib")]
extern "C" {
    // Calculate RSI (Relative Strength Index)
    fn TA_RSI(
        start_idx: i32,
        end_idx: i32,
        in_real: *const f64,     // Input prices
        opt_period: i32,          // RSI period
        out_begin: *mut i32,      // Output start index
        out_nb_element: *mut i32, // Number of output elements
        out_real: *mut f64,       // Output RSI values
    ) -> i32;
}

fn calculate_rsi_safe(prices: &[f64], period: usize) -> Vec<f64> {
    let mut out_begin: i32 = 0;
    let mut out_nb_element: i32 = 0;
    let mut output = vec![0.0; prices.len()];

    unsafe {
        let result = TA_RSI(
            0,
            (prices.len() - 1) as i32,
            prices.as_ptr(),
            period as i32,
            &mut out_begin,
            &mut out_nb_element,
            output.as_mut_ptr(),
        );

        if result != 0 {
            panic!("RSI calculation error: code {}", result);
        }
    }

    // Truncate vector to actual output size
    output.truncate(out_nb_element as usize);
    output
}

fn main() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33,
        44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28,
    ];

    let rsi = calculate_rsi_safe(&prices, 14);
    println!("RSI values: {:?}", rsi);
}
```

### 2. Optimizing critical code sections

High-frequency trading requires microsecond precision. Sometimes `unsafe` allows avoiding array bounds checks:

```rust
#[derive(Debug, Clone)]
struct OrderBookLevel {
    price: f64,
    volume: f64,
}

struct FastOrderBook {
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
}

impl FastOrderBook {
    /// SAFE version: with bounds checking
    fn get_best_bid_safe(&self) -> Option<&OrderBookLevel> {
        self.bids.first()
    }

    /// UNSAFE version: no bounds checking for maximum speed
    /// REQUIREMENT: guarantee bids is not empty through structure invariant
    unsafe fn get_best_bid_unchecked(&self) -> &OrderBookLevel {
        // Avoid array bounds check
        self.bids.get_unchecked(0)
    }

    /// Calculate spread using unsafe for speed
    fn calculate_spread_fast(&self) -> f64 {
        // Invariant: OrderBook always contains at least 1 bid and 1 ask
        unsafe {
            let best_ask = self.asks.get_unchecked(0).price;
            let best_bid = self.bids.get_unchecked(0).price;
            best_ask - best_bid
        }
    }
}

fn main() {
    let order_book = FastOrderBook {
        bids: vec![
            OrderBookLevel { price: 42150.50, volume: 1.5 },
            OrderBookLevel { price: 42150.00, volume: 2.3 },
        ],
        asks: vec![
            OrderBookLevel { price: 42151.00, volume: 0.8 },
            OrderBookLevel { price: 42151.50, volume: 1.2 },
        ],
    };

    println!("Spread: {:.2}", order_book.calculate_spread_fast());
}
```

⚠️ **IMPORTANT:** In real code, you must guarantee the invariant (non-empty bids/asks) through constructor and other methods!

### 3. Implementing low-level data structures

Lock-free data structures for multi-threaded market data processing:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;

#[derive(Debug)]
struct Trade {
    price: f64,
    volume: f64,
    timestamp: u64,
}

/// Lock-free stack for storing recent trades
/// Used in high-frequency systems to minimize latency
struct LockFreeTradeStack {
    head: AtomicPtr<Node>,
    len: AtomicUsize,
}

struct Node {
    trade: Trade,
    next: *mut Node,
}

impl LockFreeTradeStack {
    fn new() -> Self {
        LockFreeTradeStack {
            head: AtomicPtr::new(ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    /// Add trade to stack (lock-free)
    fn push(&self, trade: Trade) {
        unsafe {
            // Create new node on heap
            let new_node = Box::into_raw(Box::new(Node {
                trade,
                next: ptr::null_mut(),
            }));

            loop {
                // Read current head
                let old_head = self.head.load(Ordering::Acquire);

                // Set new node's next to old head
                (*new_node).next = old_head;

                // Try to atomically replace head
                if self.head.compare_exchange(
                    old_head,
                    new_node,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    self.len.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                // If failed — retry (someone else changed head)
            }
        }
    }

    /// Extract last trade (lock-free)
    fn pop(&self) -> Option<Trade> {
        unsafe {
            loop {
                let old_head = self.head.load(Ordering::Acquire);

                if old_head.is_null() {
                    return None;
                }

                let next = (*old_head).next;

                // Try to atomically replace head with next element
                if self.head.compare_exchange(
                    old_head,
                    next,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    self.len.fetch_sub(1, Ordering::Relaxed);

                    // Extract trade and free node memory
                    let boxed_node = Box::from_raw(old_head);
                    return Some(boxed_node.trade);
                }
                // If failed — retry
            }
        }
    }

    fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

impl Drop for LockFreeTradeStack {
    fn drop(&mut self) {
        // Clean up all nodes when dropping stack
        while self.pop().is_some() {}
    }
}

fn main() {
    let stack = LockFreeTradeStack::new();

    // Add trades
    stack.push(Trade { price: 42100.0, volume: 0.5, timestamp: 1000 });
    stack.push(Trade { price: 42105.0, volume: 1.2, timestamp: 1001 });
    stack.push(Trade { price: 42103.0, volume: 0.8, timestamp: 1002 });

    println!("Number of trades in stack: {}", stack.len());

    // Extract trades (in reverse order — stack is LIFO)
    while let Some(trade) = stack.pop() {
        println!("Trade: ${:.2}, Vol: {:.2}, Time: {}",
            trade.price, trade.volume, trade.timestamp);
    }
}
```

### 4. Working with mutable global variables

Caching configuration for fast access from different threads:

```rust
use std::sync::Mutex;

/// Global trading system configuration
struct TradingConfig {
    max_position_size: f64,
    max_order_value: f64,
    risk_limit_percent: f64,
}

// Static mutable variable (requires unsafe for access)
static mut TRADING_CONFIG: Option<TradingConfig> = None;
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

/// Initialize configuration (called once at startup)
fn init_config(config: TradingConfig) {
    let _lock = CONFIG_LOCK.lock().unwrap();
    unsafe {
        TRADING_CONFIG = Some(config);
    }
}

/// Safe configuration reading
fn get_max_position() -> f64 {
    unsafe {
        TRADING_CONFIG
            .as_ref()
            .map(|c| c.max_position_size)
            .unwrap_or(0.0)
    }
}

fn main() {
    // Initialize configuration at startup
    init_config(TradingConfig {
        max_position_size: 100_000.0,
        max_order_value: 50_000.0,
        risk_limit_percent: 2.0,
    });

    // Use configuration
    println!("Maximum position size: ${:.2}", get_max_position());
}
```

**Better alternative without unsafe:**

```rust
use std::sync::OnceLock;

static TRADING_CONFIG: OnceLock<TradingConfig> = OnceLock::new();

fn init_config(config: TradingConfig) {
    TRADING_CONFIG.set(config).expect("Configuration already initialized");
}

fn get_max_position() -> f64 {
    TRADING_CONFIG
        .get()
        .map(|c| c.max_position_size)
        .unwrap_or(0.0)
}
```

## Rules for safe usage of unsafe

### 1. Minimize unsafe blocks

```rust
// ❌ BAD: large unsafe block
unsafe {
    let ptr = data.as_ptr();
    let value = *ptr;
    process_value(value);
    another_safe_operation();
    yet_another_safe_operation();
}

// ✅ GOOD: only critical operation in unsafe
let value = unsafe { *data.as_ptr() };
process_value(value);
another_safe_operation();
yet_another_safe_operation();
```

### 2. Wrap unsafe in a safe API

```rust
/// Unsafe function for direct memory access
unsafe fn raw_memory_access(ptr: *const f64, len: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..len {
        sum += *ptr.add(i);
    }
    sum
}

/// Safe wrapper
fn calculate_sum(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    unsafe {
        // Guarantee correctness through slice validation
        raw_memory_access(prices.as_ptr(), prices.len())
    }
}

fn main() {
    let prices = vec![100.0, 101.5, 99.8, 102.3];
    println!("Sum of prices: {:.2}", calculate_sum(&prices));
}
```

### 3. Document invariants

```rust
/// Fast array element access without bounds checking
///
/// # Safety
///
/// Caller MUST guarantee:
/// - `index < data.len()`
/// - `data` is not empty
unsafe fn get_price_unchecked(data: &[f64], index: usize) -> f64 {
    *data.get_unchecked(index)
}

fn calculate_price_change(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    // Safe: verified length, indices are correct
    unsafe {
        let last = get_price_unchecked(prices, prices.len() - 1);
        let first = get_price_unchecked(prices, 0);
        ((last - first) / first) * 100.0
    }
}

fn main() {
    let btc_prices = vec![40000.0, 41000.0, 39500.0, 42000.0];
    println!("Price change: {:.2}%", calculate_price_change(&btc_prices));
}
```

## Example: optimized moving average calculation

Let's compare safe and unsafe versions:

```rust
/// Safe SMA version
fn sma_safe(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in period - 1..prices.len() {
        let sum: f64 = prices[i - period + 1..=i].iter().sum();
        result.push(sum / period as f64);
    }

    result
}

/// Unsafe optimized SMA version
fn sma_unsafe_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);

    // Calculate first window
    let mut sum = 0.0;
    for i in 0..period {
        sum += prices[i];
    }
    result.push(sum / period as f64);

    // Sliding window: remove first element, add next
    for i in period..prices.len() {
        unsafe {
            // SAFE:
            // - i >= period (by loop condition)
            // - i < prices.len() (by loop condition)
            // - i - period < prices.len() (since i < prices.len())
            sum -= prices.get_unchecked(i - period);
            sum += prices.get_unchecked(i);
        }
        result.push(sum / period as f64);
    }

    result
}

fn main() {
    let prices: Vec<f64> = (0..1000).map(|i| 40000.0 + (i as f64 * 0.1).sin() * 100.0).collect();
    let period = 20;

    // Benchmark
    use std::time::Instant;

    let start = Instant::now();
    let _sma_safe_result = sma_safe(&prices, period);
    let safe_duration = start.elapsed();

    let start = Instant::now();
    let _sma_unsafe_result = sma_unsafe_optimized(&prices, period);
    let unsafe_duration = start.elapsed();

    println!("Safe SMA: {:?}", safe_duration);
    println!("Unsafe SMA: {:?}", unsafe_duration);
    println!("Speedup: {:.2}x", safe_duration.as_nanos() as f64 / unsafe_duration.as_nanos() as f64);
}
```

## Practical Exercises

### Exercise 1: Safe FFI wrapper

Create a safe wrapper for a C function calculating VWAP (Volume Weighted Average Price):

```rust
extern "C" {
    fn calculate_vwap_c(
        prices: *const f64,
        volumes: *const f64,
        len: usize,
        output: *mut f64,
    ) -> i32;
}

// Your task: implement a safe function
fn calculate_vwap_safe(prices: &[f64], volumes: &[f64]) -> Result<f64, String> {
    // TODO: implement checks and unsafe function call
    todo!()
}
```

### Exercise 2: Lock-free trade counter

Implement a thread-safe counter for total number of trades using atomic operations:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct TradeCounter {
    total_trades: AtomicU64,
    total_volume: AtomicU64, // Multiply by 1000 to store 3 decimal places
}

impl TradeCounter {
    fn new() -> Self {
        todo!()
    }

    fn add_trade(&self, volume: f64) {
        // TODO: atomically increment counters
        todo!()
    }

    fn get_stats(&self) -> (u64, f64) {
        // TODO: return (number of trades, total volume)
        todo!()
    }
}
```

### Exercise 3: Optimization through get_unchecked

Optimize the maximum drawdown calculation function:

```rust
/// Safe version
fn max_drawdown_safe(equity_curve: &[f64]) -> f64 {
    let mut max_dd = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        if value > peak {
            peak = value;
        }
        let dd = (peak - value) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

/// Your task: create an unsafe optimized version
fn max_drawdown_unsafe(equity_curve: &[f64]) -> f64 {
    // TODO: use get_unchecked for speedup
    todo!()
}
```

### Exercise 4: Safe static configuration

Implement a configuration system without using `static mut`:

```rust
// Use OnceLock or lazy_static
use std::sync::OnceLock;

struct ExchangeConfig {
    api_url: String,
    max_reconnects: u32,
    timeout_ms: u64,
}

// TODO: create global configuration and functions to work with it
```

## Homework

1. **Performance analysis of unsafe:**
   - Implement EMA (Exponential Moving Average) calculation in two versions: safe and unsafe
   - Create a benchmark on an array of 1,000,000 elements
   - Compare performance
   - Assess whether the optimization is worth the additional risk

2. **FFI for TA-Lib:**
   - Install the TA-Lib (Technical Analysis Library)
   - Create safe wrappers for 5 indicators: SMA, EMA, RSI, MACD, Bollinger Bands
   - Write tests to verify correctness
   - Add handling for all possible errors

3. **Lock-free order queue:**
   - Implement a lock-free queue for storing orders (FIFO)
   - Support operations: enqueue, dequeue, len
   - Testing in multi-threaded environment (10 threads)
   - Performance comparison with `std::sync::mpsc`

4. **Unsafe code detector:**
   - Write a utility that analyzes a Rust project
   - Finds all `unsafe` blocks and functions
   - Checks for documentation with `# Safety` section
   - Generates a report on potential risks

5. **Safe abstractions:**
   - Take any unsafe function from the standard library
   - Create the safest possible wrapper
   - Prove through types that incorrect usage is impossible
   - Write documentation with examples

## What We Learned

| Concept | Description |
|---------|-------------|
| **unsafe** | Mode that allows bypassing some compiler checks |
| **Raw pointers** | `*const T` and `*mut T` — pointers without ownership guarantees |
| **FFI** | Interaction with code in other languages (C/C++) |
| **get_unchecked** | Access to elements without array bounds checking |
| **Atomic operations** | Lock-free synchronization via `std::sync::atomic` |
| **static mut** | Mutable global variables (better to avoid) |
| **OnceLock** | Safe alternative for global variables |
| **Invariants** | Conditions that must hold for unsafe code correctness |

## Important Principles

1. **unsafe ≠ dangerous** — it's a tool for experts, not a sign of bad code
2. **Minimize scope** — make unsafe blocks as small as possible
3. **Document** — always write `# Safety` with correctness conditions
4. **Wrap** — create safe APIs on top of unsafe code
5. **Verify** — use tools like Miri to find undefined behavior
6. **Justify** — unsafe should provide real benefits (performance, FFI)

## When NOT to use unsafe

❌ **Don't use unsafe for:**
- Bypassing type system "because it's convenient"
- Mutable global variables without extreme necessity
- Premature optimization ("in case it's faster")
- Code that can be written safely with minimal losses

✅ **Use unsafe when:**
- Interacting with external libraries through FFI
- Optimizing critical sections after profiling
- Implementing low-level data structures
- Writing abstractions for a safe interface

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
