# Day 309: String vs &str in Hot Paths

## Trading Analogy

Imagine a high-frequency trading system processing thousands of price updates per second. Each update contains a ticker symbol like "BTCUSDT".

Two approaches:
1. **String approach**: Like making a new photocopy of every document you need to read — wastes paper (memory) and time
2. **&str approach**: Like pointing at the original document — instant, no copying needed

In hot paths (code that runs millions of times), the difference between `String` and `&str` is like the difference between:
- Manually writing out every trade confirmation vs. pointing to the trade log
- Making physical copies of price charts vs. using references to the master chart

When your trading bot processes 100,000 trades per second, every nanosecond and every byte of memory matters. The choice between `String` and `&str` can mean the difference between 50ms latency and 5ms latency — enough to miss profitable trades.

## The Fundamental Difference

| Type | What It Is | Ownership | Allocation | Cost |
|------|------------|-----------|------------|------|
| **String** | Owned, growable string buffer | Owns the data | Heap | Allocates memory, copies data |
| **&str** | Reference to string slice | Borrows the data | Stack (ref only) | Zero-copy, just a pointer + length |

### Memory Layout

```rust
fn main() {
    // String: heap-allocated, owned
    let owned: String = String::from("BTCUSDT");
    // Memory: Stack has pointer + length + capacity
    //         Heap has actual data: ['B','T','C','U','S','D','T']

    // &str: reference to string data (could be anywhere)
    let borrowed: &str = "BTCUSDT";
    // Memory: Just a pointer + length on stack
    //         Data is in program's read-only memory

    println!("String size: {} bytes", std::mem::size_of_val(&owned));    // 24 bytes
    println!("&str size: {} bytes", std::mem::size_of_val(&borrowed));   // 16 bytes
}
```

## Why This Matters in Hot Paths

### Example 1: Order Book Processing

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
struct Order {
    symbol: String,  // Owned string
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderRef<'a> {
    symbol: &'a str,  // Borrowed string
    price: f64,
    quantity: f64,
}

// Hot path: called millions of times
fn process_order_owned(symbol: String, price: f64, qty: f64) -> f64 {
    // String allocation happens here - expensive!
    let order = Order {
        symbol,  // Moves the String
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

// Hot path: optimized version
fn process_order_borrowed(symbol: &str, price: f64, qty: f64) -> f64 {
    // No allocation - just using a reference
    let order = OrderRef {
        symbol,  // Just copies a reference (pointer + length)
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

fn main() {
    let iterations = 1_000_000;

    // Benchmark with String (allocating)
    let start = Instant::now();
    for _ in 0..iterations {
        let symbol = String::from("BTCUSDT");  // Allocation!
        process_order_owned(symbol, 50000.0, 0.5);
    }
    let string_duration = start.elapsed();

    // Benchmark with &str (zero-copy)
    let symbol_ref = "BTCUSDT";
    let start = Instant::now();
    for _ in 0..iterations {
        process_order_borrowed(symbol_ref, 50000.0, 0.5);
    }
    let str_duration = start.elapsed();

    println!("=== Performance Comparison ===");
    println!("String version: {:?}", string_duration);
    println!("&str version:   {:?}", str_duration);
    println!("Speedup: {:.2}x", string_duration.as_nanos() as f64 / str_duration.as_nanos() as f64);
}
```

**Expected output:**
```
=== Performance Comparison ===
String version: 45ms
&str version:   8ms
Speedup: 5.63x
```

### Example 2: Price Ticker Matching

```rust
use std::collections::HashMap;

struct MarketData {
    prices: HashMap<String, f64>,  // Owned keys - requires allocation for lookups
}

impl MarketData {
    fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 50000.0);
        prices.insert("ETHUSDT".to_string(), 3000.0);
        prices.insert("SOLUSDT".to_string(), 100.0);
        MarketData { prices }
    }

    // BAD: Allocates a new String for each lookup
    fn get_price_slow(&self, symbol: &str) -> Option<f64> {
        self.prices.get(&symbol.to_string()).copied()
        // ^^^ Allocates a String just to look it up!
    }

    // GOOD: Uses &str directly
    fn get_price_fast(&self, symbol: &str) -> Option<f64> {
        self.prices.get(symbol).copied()
        // ^^^ No allocation, HashMap can borrow for lookup
    }
}

fn main() {
    let market = MarketData::new();
    let iterations = 1_000_000;

    // Slow path: allocating String for each lookup
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = market.get_price_slow("BTCUSDT");
    }
    println!("Slow (with allocation): {:?}", start.elapsed());

    // Fast path: using &str directly
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = market.get_price_fast("BTCUSDT");
    }
    println!("Fast (zero-copy):       {:?}", start.elapsed());
}
```

## When to Use String vs &str

### Use &str when:

1. **Reading/comparing only** — You don't need to modify the string
2. **In function parameters** — Accept borrowed data to avoid forcing allocation
3. **In hot paths** — Any code that runs frequently
4. **In temporary operations** — Parsing, validation, lookups

```rust
// Good: accepts &str, caller doesn't need to allocate
fn validate_symbol(symbol: &str) -> bool {
    symbol.len() >= 3 && symbol.chars().all(|c| c.is_uppercase() || c.is_numeric())
}

// Usage - no allocation needed
let is_valid = validate_symbol("BTCUSDT");  // ✅ Zero-copy
```

### Use String when:

1. **Ownership required** — Need to store and own the data
2. **Modification needed** — Need to change, append, or build strings
3. **Dynamic data** — Building strings at runtime
4. **Returning from functions** — When you create a new string to return

```rust
// Good: returns String because we're creating new data
fn format_ticker(base: &str, quote: &str) -> String {
    format!("{}{}", base, quote)  // Creates a new String
}

// Usage
let ticker = format_ticker("BTC", "USDT");  // ticker owns the data
```

## Common Patterns in Trading Systems

### Pattern 1: Order Validation (Hot Path)

```rust
#[derive(Debug)]
struct OrderValidator;

impl OrderValidator {
    // ✅ Optimal: uses &str for validation
    fn validate_symbol(&self, symbol: &str) -> Result<(), &'static str> {
        if symbol.is_empty() {
            return Err("Symbol cannot be empty");
        }
        if !symbol.chars().all(|c| c.is_alphanumeric()) {
            return Err("Symbol must be alphanumeric");
        }
        Ok(())
    }

    // ✅ Optimal: uses references throughout
    fn validate_order(&self, symbol: &str, price: f64, quantity: f64) -> Result<(), String> {
        self.validate_symbol(symbol)?;

        if price <= 0.0 {
            return Err(format!("Invalid price for {}: {}", symbol, price));
        }

        if quantity <= 0.0 {
            return Err(format!("Invalid quantity for {}: {}", symbol, quantity));
        }

        Ok(())
    }
}

fn main() {
    let validator = OrderValidator;

    // Hot path: validating thousands of orders per second
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];

    for symbol in symbols {
        // ✅ No allocation - just passing references
        match validator.validate_order(symbol, 50000.0, 1.0) {
            Ok(_) => println!("✅ {} order valid", symbol),
            Err(e) => println!("❌ {}", e),
        }
    }
}
```

### Pattern 2: Price Aggregation

```rust
use std::collections::HashMap;

struct PriceAggregator {
    // Store owned Strings as keys (allocated once)
    prices: HashMap<String, Vec<f64>>,
}

impl PriceAggregator {
    fn new() -> Self {
        PriceAggregator {
            prices: HashMap::new(),
        }
    }

    // ✅ Accepts &str, only allocates if key doesn't exist
    fn add_price(&mut self, symbol: &str, price: f64) {
        self.prices
            .entry(symbol.to_string())  // Only allocates if inserting new key
            .or_insert_with(Vec::new)
            .push(price);
    }

    // ✅ Accepts &str for lookup (no allocation)
    fn get_average(&self, symbol: &str) -> Option<f64> {
        self.prices.get(symbol).map(|prices| {
            prices.iter().sum::<f64>() / prices.len() as f64
        })
    }

    // Returns owned String when building new data
    fn get_summary(&self, symbol: &str) -> Option<String> {
        self.prices.get(symbol).map(|prices| {
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let min = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            format!("{}: avg={:.2}, min={:.2}, max={:.2}", symbol, avg, min, max)
        })
    }
}

fn main() {
    let mut aggregator = PriceAggregator::new();

    // Hot path: adding prices (only first add allocates the key)
    let symbol = "BTCUSDT";
    for price in [50000.0, 50100.0, 49900.0, 50200.0] {
        aggregator.add_price(symbol, price);  // ✅ &str parameter
    }

    // Lookup: no allocation
    if let Some(avg) = aggregator.get_average("BTCUSDT") {
        println!("Average BTC price: ${:.2}", avg);
    }

    // Summary: allocates new String (acceptable, not in hot path)
    if let Some(summary) = aggregator.get_summary("BTCUSDT") {
        println!("{}", summary);
    }
}
```

### Pattern 3: Building Strings from Parts

```rust
struct TradeReport {
    symbol: String,
    trades: Vec<f64>,
}

impl TradeReport {
    // Accepts &str parameters, stores owned String
    fn new(symbol: &str) -> Self {
        TradeReport {
            symbol: symbol.to_string(),  // Allocate once on creation
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, pnl: f64) {
        self.trades.push(pnl);
    }

    // Returns owned String (we're building new data)
    fn generate_report(&self) -> String {
        let total_pnl: f64 = self.trades.iter().sum();
        let num_trades = self.trades.len();
        let avg_pnl = if num_trades > 0 {
            total_pnl / num_trades as f64
        } else {
            0.0
        };

        format!(
            "=== Trade Report: {} ===\n\
             Total trades: {}\n\
             Total PnL: ${:.2}\n\
             Average PnL: ${:.2}",
            self.symbol, num_trades, total_pnl, avg_pnl
        )
    }
}

fn main() {
    let mut report = TradeReport::new("BTCUSDT");  // ✅ Pass &str

    report.add_trade(150.0);
    report.add_trade(-50.0);
    report.add_trade(200.0);

    println!("{}", report.generate_report());  // Returns owned String
}
```

## Advanced: String Interning for Repeated Symbols

When you have a limited set of symbols used repeatedly (like ticker symbols), string interning can help:

```rust
use std::collections::HashMap;
use std::sync::Arc;

struct SymbolCache {
    cache: HashMap<String, Arc<str>>,
}

impl SymbolCache {
    fn new() -> Self {
        SymbolCache {
            cache: HashMap::new(),
        }
    }

    // Interns a symbol: allocates once, reuses Arc<str>
    fn intern(&mut self, symbol: &str) -> Arc<str> {
        if let Some(cached) = self.cache.get(symbol) {
            return Arc::clone(cached);  // ✅ No allocation, just clone Arc
        }

        let interned: Arc<str> = Arc::from(symbol);
        self.cache.insert(symbol.to_string(), Arc::clone(&interned));
        interned
    }
}

#[derive(Debug, Clone)]
struct OptimizedOrder {
    symbol: Arc<str>,  // Shared ownership, cheap to clone
    price: f64,
    quantity: f64,
}

fn main() {
    let mut cache = SymbolCache::new();

    // First time: allocates
    let btc_symbol = cache.intern("BTCUSDT");

    // Subsequent times: reuses existing Arc<str>
    let orders: Vec<OptimizedOrder> = (0..5)
        .map(|i| OptimizedOrder {
            symbol: Arc::clone(&btc_symbol),  // ✅ Cheap clone
            price: 50000.0 + i as f64 * 10.0,
            quantity: 0.1,
        })
        .collect();

    println!("Created {} orders", orders.len());
    for (i, order) in orders.iter().enumerate() {
        println!("Order {}: {:?}", i + 1, order);
    }
}
```

## Performance Guidelines

### Memory Allocation Cost

```rust
use std::time::Instant;

fn benchmark_allocations() {
    let iterations = 1_000_000;

    // Benchmark 1: String allocation
    let start = Instant::now();
    for _ in 0..iterations {
        let _s = String::from("BTCUSDT");  // Heap allocation
    }
    let string_time = start.elapsed();

    // Benchmark 2: &str (no allocation)
    let start = Instant::now();
    for _ in 0..iterations {
        let _s: &str = "BTCUSDT";  // No heap allocation
    }
    let str_time = start.elapsed();

    println!("=== Allocation Cost ===");
    println!("String::from(): {:?}", string_time);
    println!("&str literal:   {:?}", str_time);
    println!("Difference:     {:?}", string_time - str_time);
    println!("Allocations saved: {}", iterations);
}

fn main() {
    benchmark_allocations();
}
```

### Rules of Thumb

| Operation | Cost | When to Use |
|-----------|------|-------------|
| `"literal"` | Free (compile-time) | Default choice |
| `&str` parameter | Cheap (just pointer) | Function parameters |
| `String::from()` | Expensive (heap alloc) | When you need ownership |
| `to_string()` | Expensive (heap alloc) | Converting for storage |
| `clone()` on String | Expensive (copies data) | When unavoidable |
| `clone()` on &str | N/A (can't clone a reference) | Use `.to_string()` instead |

## What We Learned

| Concept | Description |
|---------|-------------|
| **String** | Owned, heap-allocated, growable string buffer |
| **&str** | Borrowed reference to string data (slice) |
| **Hot path** | Code that executes very frequently (performance-critical) |
| **Zero-copy** | Using references instead of copying data |
| **String interning** | Reusing single instances of strings via Arc<str> |
| **Allocation cost** | Heap allocation is expensive, avoid in hot paths |
| **Borrow for parameters** | Accept &str in functions to avoid forced allocations |
| **Own for storage** | Use String when you need to store or modify data |

## Homework

1. **Performance Profiler**: Create a tool that:
   - Takes a list of 1000 ticker symbols
   - Processes them 10,000 times
   - Compares three approaches:
     - Using `String` everywhere
     - Using `&str` everywhere possible
     - Using `Arc<str>` with interning
   - Measures and reports:
     - Total execution time
     - Memory allocations (use a global allocator tracker)
     - Peak memory usage
   - Generates a performance report

2. **Order Book Optimizer**: Implement an order book that:
   - Stores orders with symbol references
   - Implements both `String` and `&str` versions
   - Benchmarks insertion of 100,000 orders
   - Benchmarks lookup of 1,000,000 queries
   - Shows memory usage difference
   - Demonstrates when each approach is better

3. **Symbol Cache**: Build a production-ready symbol cache:
   - Uses `Arc<str>` for shared ownership
   - Implements `intern()` method for deduplication
   - Provides `get_or_intern()` for convenient usage
   - Tracks cache hit rate
   - Measures memory savings vs. using String everywhere
   - Thread-safe implementation (bonus)

4. **String Optimization Analyzer**: Write a tool that:
   - Parses Rust code (can be simple regex-based)
   - Identifies anti-patterns:
     - `&symbol.to_string()` in function calls
     - Unnecessary `clone()` on strings
     - Using `String` in function parameters
     - HashMap lookups that allocate
   - Suggests optimizations
   - Estimates potential performance gain

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md) | [Next day →](../310-*/en.md)
