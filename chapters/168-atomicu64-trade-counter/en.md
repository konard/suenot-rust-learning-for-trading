# Day 168: AtomicU64: Trade Counter

## Trading Analogy

Imagine a trading platform processing thousands of trades per second. Each trade needs a unique identifier — a transaction number. If you use a regular counter with `Mutex`, every thread would wait in line to increment the counter. This creates a bottleneck — threads spend time waiting instead of processing trades.

**AtomicU64** is like an electronic scoreboard on an exchange that updates instantly. Each trader can "atomically" (indivisibly) increment the counter in a single operation without blocking other traders. This allows generating millions of unique IDs per second without conflicts.

In real trading, `AtomicU64` is used for:
- Generating unique IDs for orders and transactions
- Counting executed trades
- Tracking trading volume in real-time
- Collecting statistics: API request counts, latency metrics

## What is AtomicU64?

`AtomicU64` is an atomic 64-bit unsigned integer type from the `std::sync::atomic` module. "Atomic" means that operations on it are performed as a single, indivisible action — no other thread can see an intermediate state.

### Core Operations

| Method | Description |
|--------|-------------|
| `new(value)` | Create with initial value |
| `load(ordering)` | Read current value |
| `store(value, ordering)` | Write new value |
| `fetch_add(value, ordering)` | Add and return old value |
| `fetch_sub(value, ordering)` | Subtract and return old value |
| `swap(value, ordering)` | Replace and return old value |
| `compare_exchange(current, new, success, failure)` | Conditional replacement |

### Memory Ordering

The `Ordering` parameter defines memory synchronization guarantees:

| Ordering | Description | Use Case |
|----------|-------------|----------|
| `Relaxed` | Minimal guarantees, only atomicity | Counters, statistics |
| `Acquire` | All subsequent operations see changes | Reading during synchronization |
| `Release` | All previous operations visible to others | Writing during synchronization |
| `AcqRel` | Combination of Acquire and Release | Read-modify-write |
| `SeqCst` | Strict ordering across all threads | When global order matters |

For simple counters, `Relaxed` is usually sufficient. For synchronization between threads, use stricter guarantees.

## Simple Trade Counter

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Atomic trade counter
    let trade_counter = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    // 10 trading threads
    for trader_id in 0..10 {
        let counter = Arc::clone(&trade_counter);

        let handle = thread::spawn(move || {
            // Each trader makes 100 trades
            for _ in 0..100 {
                // fetch_add returns the previous value
                let trade_id = counter.fetch_add(1, Ordering::Relaxed);

                // Use trade_id for logging (print every 50th)
                if trade_id % 50 == 0 {
                    println!("Trader {}: trade #{}", trader_id, trade_id);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Read final value
    let total = trade_counter.load(Ordering::Relaxed);
    println!("\nTotal trades executed: {}", total);
}
```

## Unique Order ID Generator

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique order ID generator
struct OrderIdGenerator {
    /// Counter in lower 32 bits
    counter: AtomicU64,
    /// Prefix (timestamp at startup) in upper 32 bits
    prefix: u64,
}

impl OrderIdGenerator {
    fn new() -> Self {
        // Use timestamp as prefix for uniqueness across restarts
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        OrderIdGenerator {
            counter: AtomicU64::new(0),
            prefix: (timestamp & 0xFFFFFFFF) << 32,
        }
    }

    /// Generates a unique ID for a new order
    fn next_order_id(&self) -> u64 {
        let sequence = self.counter.fetch_add(1, Ordering::Relaxed);
        self.prefix | (sequence & 0xFFFFFFFF)
    }

    /// Returns the count of generated IDs
    fn generated_count(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: &'static str,
    quantity: f64,
    price: f64,
}

fn main() {
    let generator = Arc::new(OrderIdGenerator::new());

    let mut handles = vec![];

    // Simulate multiple trading strategies
    for strategy in 0..4 {
        let gen = Arc::clone(&generator);

        let handle = thread::spawn(move || {
            let symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT"];
            let mut orders = Vec::new();

            for i in 0..25 {
                let order = Order {
                    id: gen.next_order_id(),
                    symbol: symbols[i % symbols.len()],
                    quantity: (i as f64 + 1.0) * 0.1,
                    price: 40000.0 + (i as f64 * 100.0),
                };
                orders.push(order);
            }

            println!("Strategy {} created {} orders", strategy, orders.len());
            orders
        });

        handles.push(handle);
    }

    // Collect all orders
    let mut all_orders = Vec::new();
    for handle in handles {
        let orders = handle.join().unwrap();
        all_orders.extend(orders);
    }

    println!("\nTotal orders created: {}", all_orders.len());
    println!("IDs generated: {}", generator.generated_count());

    // Verify ID uniqueness
    let mut ids: Vec<u64> = all_orders.iter().map(|o| o.id).collect();
    ids.sort();
    ids.dedup();

    if ids.len() == all_orders.len() {
        println!("All IDs are unique!");
    } else {
        println!("ERROR: duplicate IDs detected!");
    }

    // Show sample orders
    println!("\nSample orders:");
    for order in all_orders.iter().take(5) {
        println!("  {:?}", order);
    }
}
```

## Multithreaded Trading Statistics

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Trading system statistics
struct TradingStats {
    /// Number of executed buy orders
    buy_orders: AtomicU64,
    /// Number of executed sell orders
    sell_orders: AtomicU64,
    /// Total trading volume (in cents for precision)
    total_volume_cents: AtomicU64,
    /// Number of cancelled orders
    cancelled_orders: AtomicU64,
    /// Number of errors
    errors: AtomicU64,
}

impl TradingStats {
    fn new() -> Self {
        TradingStats {
            buy_orders: AtomicU64::new(0),
            sell_orders: AtomicU64::new(0),
            total_volume_cents: AtomicU64::new(0),
            cancelled_orders: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    fn record_buy(&self, volume: f64) {
        self.buy_orders.fetch_add(1, Ordering::Relaxed);
        let cents = (volume * 100.0) as u64;
        self.total_volume_cents.fetch_add(cents, Ordering::Relaxed);
    }

    fn record_sell(&self, volume: f64) {
        self.sell_orders.fetch_add(1, Ordering::Relaxed);
        let cents = (volume * 100.0) as u64;
        self.total_volume_cents.fetch_add(cents, Ordering::Relaxed);
    }

    fn record_cancel(&self) {
        self.cancelled_orders.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            buy_orders: self.buy_orders.load(Ordering::Relaxed),
            sell_orders: self.sell_orders.load(Ordering::Relaxed),
            total_volume: self.total_volume_cents.load(Ordering::Relaxed) as f64 / 100.0,
            cancelled_orders: self.cancelled_orders.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct StatsSnapshot {
    buy_orders: u64,
    sell_orders: u64,
    total_volume: f64,
    cancelled_orders: u64,
    errors: u64,
}

impl StatsSnapshot {
    fn total_orders(&self) -> u64 {
        self.buy_orders + self.sell_orders
    }

    fn buy_ratio(&self) -> f64 {
        if self.total_orders() == 0 {
            0.0
        } else {
            self.buy_orders as f64 / self.total_orders() as f64 * 100.0
        }
    }
}

fn main() {
    let stats = Arc::new(TradingStats::new());
    let start = Instant::now();

    let mut handles = vec![];

    // Monitoring thread
    let stats_monitor = Arc::clone(&stats);
    let monitor_handle = thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            let snap = stats_monitor.snapshot();
            println!(
                "[Monitor] Orders: {} (Buy: {:.1}%), Volume: ${:.2}",
                snap.total_orders(),
                snap.buy_ratio(),
                snap.total_volume
            );
        }
    });

    // Trading threads
    for trader_id in 0..4 {
        let stats_clone = Arc::clone(&stats);

        let handle = thread::spawn(move || {
            for i in 0..500 {
                let volume = 100.0 + (i as f64 * 0.5);

                match i % 10 {
                    0..=5 => stats_clone.record_buy(volume),
                    6..=8 => stats_clone.record_sell(volume),
                    9 => {
                        if i % 20 == 9 {
                            stats_clone.record_error();
                        } else {
                            stats_clone.record_cancel();
                        }
                    }
                    _ => {}
                }
            }
            println!("Trader {} finished", trader_id);
        });

        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
    monitor_handle.join().unwrap();

    let elapsed = start.elapsed();
    let final_stats = stats.snapshot();

    println!("\n===== Final Statistics =====");
    println!("Elapsed time: {:?}", elapsed);
    println!("Buys: {}", final_stats.buy_orders);
    println!("Sells: {}", final_stats.sell_orders);
    println!("Total trades: {}", final_stats.total_orders());
    println!("Total volume: ${:.2}", final_stats.total_volume);
    println!("Cancelled: {}", final_stats.cancelled_orders);
    println!("Errors: {}", final_stats.errors);
    println!("Buy/Sell ratio: {:.1}%", final_stats.buy_ratio());
}
```

## High-Performance Tick Counter

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Market tick counter with statistics support
struct TickCounter {
    /// Number of processed ticks
    ticks: AtomicU64,
    /// Last price (multiplied by 100 for precision)
    last_price_cents: AtomicU64,
    /// Maximum price for the session
    max_price_cents: AtomicU64,
    /// Minimum price for the session
    min_price_cents: AtomicU64,
}

impl TickCounter {
    fn new(initial_price: f64) -> Self {
        let price_cents = (initial_price * 100.0) as u64;
        TickCounter {
            ticks: AtomicU64::new(0),
            last_price_cents: AtomicU64::new(price_cents),
            max_price_cents: AtomicU64::new(price_cents),
            min_price_cents: AtomicU64::new(price_cents),
        }
    }

    /// Processes a new tick with price
    fn record_tick(&self, price: f64) {
        self.ticks.fetch_add(1, Ordering::Relaxed);

        let price_cents = (price * 100.0) as u64;
        self.last_price_cents.store(price_cents, Ordering::Relaxed);

        // Atomically update maximum
        let mut current_max = self.max_price_cents.load(Ordering::Relaxed);
        while price_cents > current_max {
            match self.max_price_cents.compare_exchange_weak(
                current_max,
                price_cents,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Atomically update minimum
        let mut current_min = self.min_price_cents.load(Ordering::Relaxed);
        while price_cents < current_min {
            match self.min_price_cents.compare_exchange_weak(
                current_min,
                price_cents,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
    }

    fn tick_count(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }

    fn last_price(&self) -> f64 {
        self.last_price_cents.load(Ordering::Relaxed) as f64 / 100.0
    }

    fn price_range(&self) -> (f64, f64) {
        let min = self.min_price_cents.load(Ordering::Relaxed) as f64 / 100.0;
        let max = self.max_price_cents.load(Ordering::Relaxed) as f64 / 100.0;
        (min, max)
    }
}

fn main() {
    let counter = Arc::new(TickCounter::new(42000.0));
    let start = Instant::now();

    let mut handles = vec![];

    // Simulate multiple data sources
    for source_id in 0..4 {
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            let base_price = 42000.0 + (source_id as f64 * 10.0);

            for i in 0..10000 {
                // Simulate price fluctuations
                let variation = ((i as f64 * 0.1).sin() * 500.0) +
                               ((source_id as f64 + i as f64) * 0.01);
                let price = base_price + variation;

                counter_clone.record_tick(price);
            }
        });

        handles.push(handle);
    }

    // Real-time monitoring
    let counter_monitor = Arc::clone(&counter);
    let monitor = thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(10));
            let ticks = counter_monitor.tick_count();
            let price = counter_monitor.last_price();
            let (min, max) = counter_monitor.price_range();
            println!(
                "Ticks: {:6} | Price: ${:.2} | Range: ${:.2} - ${:.2}",
                ticks, price, min, max
            );
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    monitor.join().unwrap();

    let elapsed = start.elapsed();
    let total_ticks = counter.tick_count();
    let (min, max) = counter.price_range();

    println!("\n===== Results =====");
    println!("Ticks processed: {}", total_ticks);
    println!("Time: {:?}", elapsed);
    println!("Speed: {:.0} ticks/sec", total_ticks as f64 / elapsed.as_secs_f64());
    println!("Last price: ${:.2}", counter.last_price());
    println!("Price range: ${:.2} - ${:.2}", min, max);
    println!("Volatility: ${:.2}", max - min);
}
```

## Comparing AtomicU64 and Mutex

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn benchmark_atomic(iterations: u64, threads: usize) -> std::time::Duration {
    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..iterations {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_mutex(iterations: u64, threads: usize) -> std::time::Duration {
    let counter = Arc::new(Mutex::new(0u64));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..iterations {
                    let mut guard = c.lock().unwrap();
                    *guard += 1;
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

fn main() {
    let iterations = 100_000;
    let thread_counts = [1, 2, 4, 8];

    println!("Performance Comparison: AtomicU64 vs Mutex");
    println!("Iterations per thread: {}\n", iterations);
    println!("{:>8} {:>12} {:>12} {:>10}", "Threads", "Atomic", "Mutex", "Speedup");
    println!("{}", "-".repeat(46));

    for &threads in &thread_counts {
        let atomic_time = benchmark_atomic(iterations, threads);
        let mutex_time = benchmark_mutex(iterations, threads);

        let speedup = mutex_time.as_nanos() as f64 / atomic_time.as_nanos() as f64;

        println!(
            "{:>8} {:>10.2}ms {:>10.2}ms {:>9.1}x",
            threads,
            atomic_time.as_secs_f64() * 1000.0,
            mutex_time.as_secs_f64() * 1000.0,
            speedup
        );
    }

    println!("\nConclusion: AtomicU64 is significantly faster than Mutex for simple counters,");
    println!("especially under high thread contention.");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `AtomicU64` | Atomic 64-bit unsigned counter |
| `fetch_add` | Atomic increment, returns old value |
| `compare_exchange` | Conditional replacement for complex operations |
| `Ordering::Relaxed` | Minimal guarantees, maximum speed |
| Lock-free | Atomic operations don't block threads |
| Performance | AtomicU64 is faster than Mutex for simple operations |

## Homework

1. **Trade Type Counter**: Create a `TradeTypeCounter` struct with separate `AtomicU64` counters for:
   - Market orders
   - Limit orders
   - Stop orders

   Implement methods for counting each type and getting overall statistics.

2. **ID Generator with Validation**: Modify `OrderIdGenerator` to:
   - Use `compare_exchange` to prevent counter overflow
   - Return `Option<u64>` instead of `u64`
   - Log a warning when approaching the limit (e.g., 90% of `u32::MAX`)

3. **Moving Average**: Implement an `AtomicMovingAverage` struct for calculating the average of the last N values:
   - Use a ring buffer with an atomic index
   - Atomically update the sum when adding a new value
   - Implement a thread-safe `average()` method

4. **Rate Limiter**: Create a `TradeRateLimiter` struct that:
   - Limits the number of trades per second
   - Uses atomic counters for tracking
   - Returns `true`/`false` when attempting a trade
   - Automatically resets the counter every second

## Navigation

[← Previous day](../167-atomicbool-bot-stop-flag/en.md) | [Next day →](../169-ordering-visibility-guarantees/en.md)
