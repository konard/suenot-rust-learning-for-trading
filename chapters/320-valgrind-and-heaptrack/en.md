# Day 320: Valgrind and Heaptrack

## Trading Analogy

Imagine you're running a hedge fund that manages billions in assets. Your trading system processes thousands of orders per second. One day you notice the system is slowing down — at first subtly, then dramatically. Memory usage keeps climbing until the system crashes during critical market hours.

This is like having a warehouse with thousands of trading positions:
- **Memory leak** — boxes (allocations) come in but never leave. The warehouse fills up until there's no space
- **Excessive allocations** — workers constantly moving boxes back and forth, wasting time on logistics instead of trading
- **Memory fragmentation** — like storing small items in large boxes scattered randomly, making it hard to find space for new items

**Valgrind** and **Heaptrack** are like warehouse auditors:
- **Valgrind** — a thorough inspector who tracks every box movement, finds leaks, and detects invalid access
- **Heaptrack** — a fast analyst who provides allocation statistics and identifies memory hotspots

In production trading systems, memory issues can cause:
- **Slippage**: Slow memory operations mean delayed order execution
- **System crashes**: Out-of-memory during volatile markets = missed trades
- **Unpredictable latency**: GC-like pauses from fragmentation affect timing

## What is Valgrind?

**Valgrind** is a powerful instrumentation framework for memory debugging. Its primary tool, **Memcheck**, detects:

| Problem | Description | Trading Impact |
|---------|-------------|----------------|
| **Memory leaks** | Allocated memory never freed | System crashes during long runs |
| **Invalid reads/writes** | Accessing freed or uninitialized memory | Data corruption, wrong prices |
| **Use after free** | Using memory after deallocation | Unpredictable behavior |
| **Double free** | Freeing the same memory twice | Crashes |
| **Invalid memory access** | Buffer overflows, null pointer dereferences | Security vulnerabilities |

### Installing Valgrind

```bash
# Ubuntu/Debian
sudo apt-get install valgrind

# Fedora
sudo dnf install valgrind

# macOS (Intel only, not supported on Apple Silicon)
brew install valgrind
```

## What is Heaptrack?

**Heaptrack** is a modern heap memory profiler. Unlike Valgrind, it focuses on:

| Feature | Description |
|---------|-------------|
| **Speed** | Much faster than Valgrind (10-100x) |
| **Allocation tracking** | Counts and sizes of all allocations |
| **Call stacks** | Which functions allocate the most |
| **Flame graphs** | Visual representation of memory hotspots |
| **Leak detection** | Identifies memory that's never freed |

### Installing Heaptrack

```bash
# Ubuntu/Debian
sudo apt-get install heaptrack heaptrack-gui

# Fedora
sudo dnf install heaptrack

# Build from source (if not in repos)
git clone https://github.com/KDE/heaptrack.git
cd heaptrack && mkdir build && cd build
cmake .. && make && sudo make install
```

## Basic Usage with Rust

### Setting Up a Test Project

First, let's create a trading system with intentional memory issues:

```rust
// src/main.rs
use std::collections::HashMap;
use std::time::Instant;

/// Represents a trading order with allocation overhead
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,      // Heap allocated
    price: f64,
    quantity: f64,
    timestamp: u64,
    metadata: HashMap<String, String>,  // Additional heap allocations
}

impl Order {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "api".to_string());
        metadata.insert("status".to_string(), "pending".to_string());

        Order {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        }
    }
}

/// Order book with potential memory issues
struct OrderBook {
    orders: Vec<Order>,
    history: Vec<Order>,  // Potential memory leak if not managed
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: Vec::new(),
            history: Vec::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.push(order.clone());
        // BUG: History grows unbounded - memory leak pattern
        self.history.push(order);
    }

    fn execute_order(&mut self, id: u64) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            let order = self.orders.remove(pos);
            // History keeps growing even after execution
            self.history.push(order.clone());
            Some(order)
        } else {
            None
        }
    }

    fn memory_stats(&self) {
        println!("Active orders: {}", self.orders.len());
        println!("History size: {}", self.history.len());
        println!("Estimated history memory: {} KB",
            self.history.len() * std::mem::size_of::<Order>() / 1024);
    }
}

/// Simulates high-frequency trading with memory allocations
fn simulate_trading(iterations: usize) {
    let mut order_book = OrderBook::new();
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"];

    println!("Starting trading simulation with {} iterations", iterations);
    let start = Instant::now();

    for i in 0..iterations {
        // Create new order (allocates memory)
        let symbol = symbols[i % symbols.len()];
        let order = Order::new(
            i as u64,
            symbol,
            50000.0 + (i as f64 * 0.01),
            0.1 + (i as f64 * 0.001),
        );

        order_book.add_order(order);

        // Execute some orders (but history keeps growing)
        if i > 0 && i % 10 == 0 {
            order_book.execute_order((i - 5) as u64);
        }

        // Print stats periodically
        if i > 0 && i % 10000 == 0 {
            order_book.memory_stats();
        }
    }

    let duration = start.elapsed();
    println!("\nSimulation completed in {:?}", duration);
    order_book.memory_stats();
}

fn main() {
    // Run with fewer iterations for Valgrind (it's slow)
    // Increase for realistic profiling with Heaptrack
    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);

    simulate_trading(iterations);
}
```

### Compiling for Profiling

```bash
# Build with debug symbols for better stack traces
# Use release for realistic performance, but keep debug info
cargo build --release

# Or with maximum debug info
RUSTFLAGS="-g" cargo build --release
```

## Using Valgrind with Rust

### Running Memcheck

```bash
# Basic memory check
valgrind --leak-check=full ./target/release/trading_profiler

# Detailed leak analysis
valgrind --leak-check=full --show-leak-kinds=all \
         --track-origins=yes ./target/release/trading_profiler

# With reduced iterations (Valgrind is ~20x slower)
ITERATIONS=5000 valgrind --leak-check=full ./target/release/trading_profiler
```

### Understanding Valgrind Output

```
==12345== HEAP SUMMARY:
==12345==     in use at exit: 1,234,567 bytes in 8,901 blocks
==12345==   total heap usage: 123,456 allocs, 114,555 frees, 12,345,678 bytes allocated
==12345==
==12345== 567,890 bytes in 1,234 blocks are definitely lost in loss record 42 of 50
==12345==    at 0x4C2FB0F: malloc (vg_replace_malloc.c:299)
==12345==    by 0x55AABCD: alloc::alloc::alloc (alloc.rs:81)
==12345==    by 0x55AACDE: <alloc::vec::Vec<T>>::push (vec.rs:1234)
==12345==    by 0x123456: trading_profiler::OrderBook::add_order (main.rs:45)
==12345==    by 0x234567: trading_profiler::simulate_trading (main.rs:78)
```

Key metrics:
- **definitely lost**: Memory that's leaked (no pointers to it)
- **indirectly lost**: Memory reachable only through leaked memory
- **possibly lost**: Memory with interior pointers (usually false positives in Rust)
- **still reachable**: Memory reachable at exit (often intentional)

## Using Heaptrack with Rust

### Running Heaptrack

```bash
# Profile the application
heaptrack ./target/release/trading_profiler

# With more iterations (Heaptrack is much faster)
ITERATIONS=100000 heaptrack ./target/release/trading_profiler

# Output: heaptrack.trading_profiler.12345.gz
```

### Analyzing Results

```bash
# Text-based analysis
heaptrack_print heaptrack.trading_profiler.12345.gz

# GUI analysis (if available)
heaptrack_gui heaptrack.trading_profiler.12345.gz
```

### Understanding Heaptrack Output

```
SUMMARY
=======
Total runtime: 2.5s
Total memory allocated: 156.78 MB
Peak heap memory: 45.23 MB
Peak RSS: 67.89 MB
Total allocations: 234,567
Calls to malloc: 234,567

MOST MEMORY ALLOCATED
=====================
  1. 89.45 MB from 123,456 allocations
     alloc::vec::Vec<T>::push
       at /rustc/xxx/library/alloc/src/vec/mod.rs:1234
     trading_profiler::OrderBook::add_order
       at src/main.rs:45

  2. 34.56 MB from 67,890 allocations
     alloc::string::String::from
       at /rustc/xxx/library/alloc/src/string.rs:567
     trading_profiler::Order::new
       at src/main.rs:23

LEAKED MEMORY
=============
  Total: 23.45 MB

  1. 18.90 MB leaked from:
     trading_profiler::OrderBook::history
       at src/main.rs:38
```

## Practical Example: Optimizing a Price Feed Processor

Let's create a more realistic example with memory optimization:

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Price tick from market feed
#[derive(Debug, Clone)]
struct PriceTick {
    symbol: Arc<str>,  // Interned symbol - shared across ticks
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// INEFFICIENT: String allocation per tick
#[derive(Debug, Clone)]
struct PriceTickBad {
    symbol: String,  // Allocates every time
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// Price aggregator with bounded memory
struct PriceAggregator {
    // Ring buffer - bounded memory usage
    prices: HashMap<Arc<str>, VecDeque<f64>>,
    max_history: usize,
    symbol_cache: HashMap<String, Arc<str>>,
}

impl PriceAggregator {
    fn new(max_history: usize) -> Self {
        PriceAggregator {
            prices: HashMap::new(),
            max_history,
            symbol_cache: HashMap::new(),
        }
    }

    /// Intern a symbol string (allocate once, reuse everywhere)
    fn intern_symbol(&mut self, symbol: &str) -> Arc<str> {
        if let Some(cached) = self.symbol_cache.get(symbol) {
            return Arc::clone(cached);
        }
        let interned: Arc<str> = Arc::from(symbol);
        self.symbol_cache.insert(symbol.to_string(), Arc::clone(&interned));
        interned
    }

    /// Add price with bounded memory
    fn add_price(&mut self, symbol: Arc<str>, price: f64) {
        let history = self.prices
            .entry(symbol)
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));

        // Bounded buffer - old prices are dropped
        if history.len() >= self.max_history {
            history.pop_front();  // Remove oldest
        }
        history.push_back(price);
    }

    /// Calculate moving average
    fn get_sma(&self, symbol: &Arc<str>, period: usize) -> Option<f64> {
        self.prices.get(symbol).map(|history| {
            let count = history.len().min(period);
            if count == 0 {
                return 0.0;
            }
            history.iter().rev().take(count).sum::<f64>() / count as f64
        })
    }

    fn memory_stats(&self) {
        let total_prices: usize = self.prices.values().map(|v| v.len()).sum();
        let symbols = self.prices.len();
        println!("Symbols tracked: {}", symbols);
        println!("Total prices in memory: {}", total_prices);
        println!("Symbol cache size: {}", self.symbol_cache.len());
    }
}

/// INEFFICIENT version for comparison
struct PriceAggregatorBad {
    prices: HashMap<String, Vec<f64>>,  // Unbounded + String keys
}

impl PriceAggregatorBad {
    fn new() -> Self {
        PriceAggregatorBad {
            prices: HashMap::new(),
        }
    }

    fn add_price(&mut self, symbol: &str, price: f64) {
        // Allocates String for every lookup!
        self.prices
            .entry(symbol.to_string())  // Allocation!
            .or_insert_with(Vec::new)
            .push(price);  // Unbounded growth
    }
}

fn benchmark_optimized(iterations: usize) -> std::time::Duration {
    let mut aggregator = PriceAggregator::new(1000);  // Bounded to 1000 prices
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];

    let start = Instant::now();

    for i in 0..iterations {
        let symbol_str = symbols[i % symbols.len()];
        let symbol = aggregator.intern_symbol(symbol_str);
        let price = 50000.0 + (i as f64 * 0.001);

        aggregator.add_price(symbol.clone(), price);

        // Occasionally compute SMA
        if i % 100 == 0 {
            let _ = aggregator.get_sma(&symbol, 20);
        }
    }

    let duration = start.elapsed();
    println!("\nOptimized version:");
    aggregator.memory_stats();
    duration
}

fn benchmark_inefficient(iterations: usize) -> std::time::Duration {
    let mut aggregator = PriceAggregatorBad::new();
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];

    let start = Instant::now();

    for i in 0..iterations {
        let symbol = symbols[i % symbols.len()];
        let price = 50000.0 + (i as f64 * 0.001);

        aggregator.add_price(symbol, price);
    }

    let duration = start.elapsed();
    let total_prices: usize = aggregator.prices.values().map(|v| v.len()).sum();
    println!("\nInefficient version:");
    println!("Total prices in memory: {}", total_prices);
    duration
}

fn main() {
    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    println!("=== Memory Profiling Comparison ===");
    println!("Iterations: {}\n", iterations);

    let optimized_time = benchmark_optimized(iterations);
    let inefficient_time = benchmark_inefficient(iterations);

    println!("\n=== Performance Results ===");
    println!("Optimized:   {:?}", optimized_time);
    println!("Inefficient: {:?}", inefficient_time);
    println!("Speedup:     {:.2}x",
        inefficient_time.as_nanos() as f64 / optimized_time.as_nanos() as f64);
}
```

### Running the Comparison with Heaptrack

```bash
# Build
cargo build --release

# Profile optimized version
ITERATIONS=500000 heaptrack ./target/release/trading_profiler

# Compare allocations
heaptrack_print heaptrack.trading_profiler.*.gz | head -50
```

## Detecting Memory Leaks in Trading Systems

### Common Leak Patterns

```rust
use std::collections::HashMap;

/// Pattern 1: Unbounded cache
struct LeakyCache {
    cache: HashMap<String, Vec<f64>>,  // Never evicts old entries
}

impl LeakyCache {
    fn add(&mut self, key: &str, value: f64) {
        self.cache
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        // BUG: Cache grows forever
    }
}

/// Pattern 2: Event history without cleanup
struct LeakyEventLog {
    events: Vec<String>,  // Never cleared
}

impl LeakyEventLog {
    fn log(&mut self, event: &str) {
        self.events.push(format!("[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            event
        ));
        // BUG: Events accumulate forever
    }
}

/// Pattern 3: Circular references (rare in Rust, but possible with Rc)
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    value: i32,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,  // Creates cycle!
}

/// FIXED versions

/// Fixed Pattern 1: LRU Cache with eviction
use std::collections::BTreeMap;

struct BoundedCache {
    cache: HashMap<String, f64>,
    order: BTreeMap<u64, String>,  // For LRU eviction
    max_size: usize,
    counter: u64,
}

impl BoundedCache {
    fn new(max_size: usize) -> Self {
        BoundedCache {
            cache: HashMap::with_capacity(max_size),
            order: BTreeMap::new(),
            max_size,
            counter: 0,
        }
    }

    fn add(&mut self, key: &str, value: f64) {
        // Evict oldest if at capacity
        while self.cache.len() >= self.max_size {
            if let Some((&oldest_time, _)) = self.order.iter().next() {
                if let Some(oldest_key) = self.order.remove(&oldest_time) {
                    self.cache.remove(&oldest_key);
                }
            }
        }

        self.counter += 1;
        self.cache.insert(key.to_string(), value);
        self.order.insert(self.counter, key.to_string());
    }
}

/// Fixed Pattern 2: Rolling event log
struct RollingEventLog {
    events: std::collections::VecDeque<String>,
    max_events: usize,
}

impl RollingEventLog {
    fn new(max_events: usize) -> Self {
        RollingEventLog {
            events: std::collections::VecDeque::with_capacity(max_events),
            max_events,
        }
    }

    fn log(&mut self, event: &str) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();  // Remove oldest
        }
        self.events.push_back(format!("[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            event
        ));
    }
}

/// Fixed Pattern 3: Use Weak references for back-pointers
use std::rc::Weak;

struct SafeNode {
    value: i32,
    next: Option<Rc<RefCell<SafeNode>>>,
    prev: Weak<RefCell<SafeNode>>,  // Weak breaks the cycle
}
```

## Valgrind Suppression Files

Sometimes Valgrind reports false positives. Create suppression files for known-safe patterns:

```
# rust.supp - Valgrind suppression file for Rust
{
   rust_thread_local
   Memcheck:Leak
   match-leak-kinds: possible
   fun:malloc
   ...
   fun:*thread_local*
}

{
   rust_backtrace
   Memcheck:Leak
   match-leak-kinds: reachable
   ...
   fun:*backtrace*
}
```

Usage:
```bash
valgrind --suppressions=rust.supp --leak-check=full ./target/release/app
```

## Integrating Memory Profiling into CI/CD

### GitHub Actions Example

```yaml
name: Memory Profiling

on:
  push:
    branches: [main]
  pull_request:

jobs:
  memory-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Valgrind
        run: sudo apt-get install -y valgrind

      - name: Build
        run: cargo build --release

      - name: Run Valgrind
        run: |
          ITERATIONS=5000 valgrind --leak-check=full \
            --error-exitcode=1 \
            ./target/release/trading_profiler 2>&1 | tee valgrind.log

      - name: Check for leaks
        run: |
          if grep -q "definitely lost:" valgrind.log; then
            echo "Memory leaks detected!"
            grep -A5 "definitely lost:" valgrind.log
            exit 1
          fi

      - name: Upload report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: valgrind-report
          path: valgrind.log
```

## Performance Comparison Script

```rust
use std::process::Command;
use std::fs;

fn run_with_heaptrack(binary: &str, iterations: usize) -> String {
    let output = Command::new("heaptrack")
        .arg(binary)
        .env("ITERATIONS", iterations.to_string())
        .output()
        .expect("Failed to run heaptrack");

    String::from_utf8_lossy(&output.stderr).to_string()
}

fn parse_heaptrack_summary(output_file: &str) -> HeaptrackStats {
    let output = Command::new("heaptrack_print")
        .arg(output_file)
        .output()
        .expect("Failed to parse heaptrack output");

    let text = String::from_utf8_lossy(&output.stdout);

    // Parse key metrics
    HeaptrackStats {
        total_allocated: parse_bytes(&text, "Total memory allocated"),
        peak_memory: parse_bytes(&text, "Peak heap memory"),
        total_allocations: parse_count(&text, "Total allocations"),
    }
}

#[derive(Debug)]
struct HeaptrackStats {
    total_allocated: u64,
    peak_memory: u64,
    total_allocations: u64,
}

fn parse_bytes(text: &str, pattern: &str) -> u64 {
    // Implementation for parsing byte values
    0
}

fn parse_count(text: &str, pattern: &str) -> u64 {
    // Implementation for parsing counts
    0
}

fn main() {
    println!("=== Automated Memory Profiling ===\n");

    // Run profiling
    let stats = run_with_heaptrack("./target/release/trading_app", 100_000);

    println!("Profiling complete!");
    println!("{}", stats);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Valgrind** | Memory debugging tool that detects leaks, invalid access, and use-after-free |
| **Heaptrack** | Fast heap profiler showing allocation patterns and memory hotspots |
| **Memory leak** | Memory allocated but never freed, causing gradual resource exhaustion |
| **Bounded buffers** | Using VecDeque with max capacity to prevent unbounded growth |
| **String interning** | Sharing string data via Arc<str> to reduce allocations |
| **Suppression files** | Filtering out known false positives from Valgrind |
| **CI integration** | Automated memory checking in build pipelines |

## Practical Exercises

1. **Find the Leak**: Create a trading order manager that intentionally leaks memory. Use Valgrind to locate the leak, then fix it with bounded data structures.

2. **Allocation Audit**: Profile an existing trading application with Heaptrack. Identify the top 3 allocation hotspots and propose optimizations.

3. **Memory Budget**: Implement a price aggregator that operates within a fixed memory budget (e.g., 100MB). Use Heaptrack to verify it stays within bounds under heavy load.

4. **CI Pipeline**: Set up a GitHub Actions workflow that runs Valgrind on every PR and fails if any memory leaks are detected.

## Homework

1. **Trade History Manager**: Build a system that:
   - Records all trades with timestamps
   - Uses a rolling window (keeps last N trades)
   - Profile with both Valgrind and Heaptrack
   - Compare memory usage vs. unbounded version
   - Generate a report showing memory savings

2. **Order Book Memory Optimizer**: Create an order book that:
   - Handles 1 million orders per run
   - Uses string interning for symbols
   - Implements bounded order history
   - Profile memory patterns with Heaptrack
   - Achieve < 50MB peak memory usage

3. **Leak Detection Suite**: Write a test harness that:
   - Runs multiple test scenarios
   - Uses Valgrind to check each for leaks
   - Generates HTML report of findings
   - Integrates with cargo test
   - Supports CI/CD integration

4. **Memory Regression Tests**: Implement a system that:
   - Records baseline memory usage
   - Compares new commits against baseline
   - Alerts if memory usage increases by > 10%
   - Uses Heaptrack for measurements
   - Runs automatically on PR

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../321-*/en.md)
