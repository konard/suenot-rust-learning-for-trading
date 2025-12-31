# Day 166: Atomics: Atomic Operations

## Trading Analogy

Imagine an exchange terminal where thousands of traders simultaneously view the current asset price. Every millisecond the price updates, and everyone must see the same value. If one trader sees a price of 42000 while another sees 42100 at the same moment, this would lead to chaos and arbitrage exploits.

**Atomic operations** are like an electronic price board on the exchange. When the price changes, it changes instantly and completely — it's impossible to see "half" of the new value. Either the old price or the new one, no intermediate states.

In real algorithmic trading, atomic operations are used for:
- Order execution counters
- Trading halt flags
- Current market prices
- Connected API client counters

## What are Atomics?

Atomic types (Atomics) are primitives that guarantee read and write operations are performed completely and indivisibly. In Rust, they are found in the `std::sync::atomic` module.

### Advantages over Mutex:

| Characteristic | Mutex | Atomic |
|---------------|-------|--------|
| Blocking | Yes, waits for release | No blocking |
| Performance | Slower | Very fast |
| Data protection | Any types | Only primitives |
| Deadlock | Possible | Impossible |
| Complexity | Simple API | Requires understanding Ordering |

### Main Atomic Types:

```rust
use std::sync::atomic::{
    AtomicBool,    // Atomic boolean
    AtomicI32,     // Atomic i32
    AtomicI64,     // Atomic i64
    AtomicU32,     // Atomic u32
    AtomicU64,     // Atomic u64
    AtomicUsize,   // Atomic usize
    AtomicPtr,     // Atomic pointer
    Ordering,      // Memory ordering
};
```

## Simple Example: Order Counter

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Atomic order counter
    let order_counter = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    // 10 threads, each creates 100 orders
    for thread_id in 0..10 {
        let counter = Arc::clone(&order_counter);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Atomic increment — no blocking!
                let order_id = counter.fetch_add(1, Ordering::SeqCst);

                // order_id is unique for each order
                if order_id % 100 == 0 {
                    println!("Thread {}: created order #{}", thread_id, order_id);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Guaranteed to get 1000!
    let total = order_counter.load(Ordering::SeqCst);
    println!("Total orders created: {}", total);
}
```

## Ordering (Memory Ordering)

The most complex part of atomic operations is understanding `Ordering`. It tells the compiler and processor how to synchronize memory between threads.

### Types of Ordering:

```rust
use std::sync::atomic::Ordering;

// Relaxed — minimal guarantees, maximum performance
// Use for counters where order doesn't matter
Ordering::Relaxed

// Acquire — all reads after this operation see writes before Release
// Use when reading a flag or pointer
Ordering::Acquire

// Release — all writes before this operation are visible after Acquire
// Use when writing a flag or pointer
Ordering::Release

// AcqRel — combination of Acquire and Release
// Use for read-modify-write operations
Ordering::AcqRel

// SeqCst — strictest guarantees, all threads see the same order
// Use when unsure — this is the safest option
Ordering::SeqCst
```

### Practical Rule:

```rust
// If unsure — use SeqCst
let value = atomic.load(Ordering::SeqCst);
atomic.store(new_value, Ordering::SeqCst);

// For simple counters, Relaxed is fine
counter.fetch_add(1, Ordering::Relaxed);

// For stop flags: Release when writing, Acquire when reading
stop_flag.store(true, Ordering::Release);
if stop_flag.load(Ordering::Acquire) { /* ... */ }
```

## Trading Bot Stop Flag

A classic pattern — using `AtomicBool` for safe thread shutdown:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingBot {
    should_stop: AtomicBool,
}

impl TradingBot {
    fn new() -> Self {
        TradingBot {
            should_stop: AtomicBool::new(false),
        }
    }

    fn stop(&self) {
        println!("Stop signal received!");
        self.should_stop.store(true, Ordering::Release);
    }

    fn is_running(&self) -> bool {
        !self.should_stop.load(Ordering::Acquire)
    }

    fn run_strategy(&self, name: &str) {
        println!("Strategy '{}' started", name);
        let mut tick = 0;

        while self.is_running() {
            tick += 1;

            // Simulate trading logic
            if tick % 10 == 0 {
                println!("[{}] Tick #{}: analyzing market...", name, tick);
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("Strategy '{}' stopped at tick #{}", name, tick);
    }
}

fn main() {
    let bot = Arc::new(TradingBot::new());

    let bot1 = Arc::clone(&bot);
    let bot2 = Arc::clone(&bot);

    // Start two strategies
    let strategy1 = thread::spawn(move || {
        bot1.run_strategy("Momentum");
    });

    let strategy2 = thread::spawn(move || {
        bot2.run_strategy("MeanReversion");
    });

    // Wait 2 seconds and stop
    thread::sleep(Duration::from_secs(2));
    bot.stop();

    strategy1.join().unwrap();
    strategy2.join().unwrap();

    println!("All strategies stopped");
}
```

## Atomic Asset Price

Example of updating and reading price from multiple threads:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Store price as u64 (cents) for atomic operations
// $42000.50 = 4200050 cents
struct AtomicPrice {
    // Price in cents (for precision)
    price_cents: AtomicU64,
}

impl AtomicPrice {
    fn new(price: f64) -> Self {
        AtomicPrice {
            price_cents: AtomicU64::new((price * 100.0) as u64),
        }
    }

    fn get(&self) -> f64 {
        self.price_cents.load(Ordering::Acquire) as f64 / 100.0
    }

    fn set(&self, price: f64) {
        self.price_cents.store((price * 100.0) as u64, Ordering::Release);
    }

    // Atomic update only if price changed significantly
    fn update_if_significant(&self, new_price: f64, threshold_percent: f64) -> bool {
        let new_cents = (new_price * 100.0) as u64;

        loop {
            let current = self.price_cents.load(Ordering::Acquire);
            let current_price = current as f64 / 100.0;

            let change_percent = ((new_price - current_price) / current_price * 100.0).abs();

            if change_percent < threshold_percent {
                return false; // Change is insignificant
            }

            // CAS (Compare-And-Swap) — atomic check and replace
            match self.price_cents.compare_exchange(
                current,
                new_cents,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,  // Successfully updated
                Err(_) => continue,    // Someone else changed it, try again
            }
        }
    }
}

fn main() {
    let btc_price = Arc::new(AtomicPrice::new(42000.0));

    let price_reader = Arc::clone(&btc_price);
    let price_writer = Arc::clone(&btc_price);

    // Price update thread (simulating exchange)
    let writer = thread::spawn(move || {
        let prices = [42100.0, 41950.0, 42300.0, 41800.0, 42500.0];

        for price in prices {
            price_writer.set(price);
            println!("[Exchange] New BTC price: ${:.2}", price);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // Price reading thread (trader)
    let reader = thread::spawn(move || {
        for i in 0..10 {
            let price = price_reader.get();
            println!("[Trader] Tick #{}: BTC = ${:.2}", i + 1, price);
            thread::sleep(Duration::from_millis(300));
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
```

## Compare-And-Swap (CAS)

CAS is a fundamental atomic operation. It checks the current value and replaces it with a new one, only if the current matches the expected value.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

// Atomic liquidity reservation for an order
struct LiquidityPool {
    available: AtomicU64, // Available liquidity in cents
}

impl LiquidityPool {
    fn new(amount: f64) -> Self {
        LiquidityPool {
            available: AtomicU64::new((amount * 100.0) as u64),
        }
    }

    fn get_available(&self) -> f64 {
        self.available.load(Ordering::Acquire) as f64 / 100.0
    }

    // Atomic funds reservation
    fn reserve(&self, amount: f64) -> Result<(), String> {
        let amount_cents = (amount * 100.0) as u64;

        loop {
            let current = self.available.load(Ordering::Acquire);

            if current < amount_cents {
                return Err(format!(
                    "Insufficient liquidity: need ${:.2}, available ${:.2}",
                    amount, current as f64 / 100.0
                ));
            }

            let new_value = current - amount_cents;

            // CAS: if value hasn't changed — reserve
            match self.available.compare_exchange(
                current,
                new_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    println!("Reserved ${:.2}, remaining ${:.2}",
                        amount, new_value as f64 / 100.0);
                    return Ok(());
                }
                Err(_) => {
                    // Someone changed the value, try again
                    continue;
                }
            }
        }
    }

    // Return funds
    fn release(&self, amount: f64) {
        let amount_cents = (amount * 100.0) as u64;
        self.available.fetch_add(amount_cents, Ordering::AcqRel);
        println!("Released ${:.2}", amount);
    }
}

fn main() {
    let pool = Arc::new(LiquidityPool::new(10000.0)); // $10,000

    let mut handles = vec![];

    // 5 traders trying to reserve funds
    for trader_id in 0..5 {
        let pool_clone = Arc::clone(&pool);

        let handle = thread::spawn(move || {
            let amounts = [1500.0, 2000.0, 1000.0];

            for amount in amounts {
                match pool_clone.reserve(amount) {
                    Ok(_) => println!("Trader {}: successfully reserved ${}", trader_id, amount),
                    Err(e) => println!("Trader {}: {}", trader_id, e),
                }
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nFinal available liquidity: ${:.2}", pool.get_available());
}
```

## Trading Statistics with Atomics

```rust
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingStats {
    total_trades: AtomicU64,
    winning_trades: AtomicU64,
    losing_trades: AtomicU64,
    total_pnl_cents: AtomicI64, // Can be negative!
}

impl TradingStats {
    fn new() -> Self {
        TradingStats {
            total_trades: AtomicU64::new(0),
            winning_trades: AtomicU64::new(0),
            losing_trades: AtomicU64::new(0),
            total_pnl_cents: AtomicI64::new(0),
        }
    }

    fn record_trade(&self, pnl: f64) {
        self.total_trades.fetch_add(1, Ordering::Relaxed);

        let pnl_cents = (pnl * 100.0) as i64;
        self.total_pnl_cents.fetch_add(pnl_cents, Ordering::Relaxed);

        if pnl >= 0.0 {
            self.winning_trades.fetch_add(1, Ordering::Relaxed);
        } else {
            self.losing_trades.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn get_summary(&self) -> TradingSummary {
        TradingSummary {
            total: self.total_trades.load(Ordering::Acquire),
            wins: self.winning_trades.load(Ordering::Acquire),
            losses: self.losing_trades.load(Ordering::Acquire),
            pnl: self.total_pnl_cents.load(Ordering::Acquire) as f64 / 100.0,
        }
    }
}

struct TradingSummary {
    total: u64,
    wins: u64,
    losses: u64,
    pnl: f64,
}

impl TradingSummary {
    fn win_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.wins as f64 / self.total as f64 * 100.0
        }
    }
}

fn main() {
    let stats = Arc::new(TradingStats::new());

    let stats1 = Arc::clone(&stats);
    let stats2 = Arc::clone(&stats);
    let stats3 = Arc::clone(&stats);

    // Three strategies trading in parallel
    let strategy1 = thread::spawn(move || {
        let results = [150.0, -50.0, 200.0, -30.0, 80.0];
        for pnl in results {
            stats1.record_trade(pnl);
            thread::sleep(Duration::from_millis(50));
        }
    });

    let strategy2 = thread::spawn(move || {
        let results = [-100.0, 300.0, -20.0, 150.0, -80.0];
        for pnl in results {
            stats2.record_trade(pnl);
            thread::sleep(Duration::from_millis(40));
        }
    });

    let strategy3 = thread::spawn(move || {
        let results = [50.0, 100.0, -200.0, 75.0, 25.0];
        for pnl in results {
            stats3.record_trade(pnl);
            thread::sleep(Duration::from_millis(60));
        }
    });

    // Real-time monitoring
    let stats_monitor = Arc::clone(&stats);
    let monitor = thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            let summary = stats_monitor.get_summary();
            println!(
                "Stats: trades={}, winrate={:.1}%, PnL=${:.2}",
                summary.total, summary.win_rate(), summary.pnl
            );
        }
    });

    strategy1.join().unwrap();
    strategy2.join().unwrap();
    strategy3.join().unwrap();
    monitor.join().unwrap();

    let final_summary = stats.get_summary();
    println!("\n=== Final Statistics ===");
    println!("Total trades: {}", final_summary.total);
    println!("Winning: {}", final_summary.wins);
    println!("Losing: {}", final_summary.losses);
    println!("Win Rate: {:.1}%", final_summary.win_rate());
    println!("Total PnL: ${:.2}", final_summary.pnl);
}
```

## Practical Example: API Rate Limiter

```rust
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct RateLimiter {
    requests_this_second: AtomicU32,
    max_requests_per_second: u32,
    window_start_ms: AtomicU64,
}

impl RateLimiter {
    fn new(max_rps: u32) -> Self {
        RateLimiter {
            requests_this_second: AtomicU32::new(0),
            max_requests_per_second: max_rps,
            window_start_ms: AtomicU64::new(0),
        }
    }

    fn try_acquire(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let current_window = now_ms / 1000 * 1000; // Start of current second

        // Check if a new window has started
        let last_window = self.window_start_ms.load(Ordering::Acquire);

        if current_window > last_window {
            // Try to update the window
            if self.window_start_ms
                .compare_exchange(last_window, current_window, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Successfully updated window, reset counter
                self.requests_this_second.store(0, Ordering::Release);
            }
        }

        // Try to increment counter
        loop {
            let current = self.requests_this_second.load(Ordering::Acquire);

            if current >= self.max_requests_per_second {
                return false; // Limit reached
            }

            if self.requests_this_second
                .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return true; // Successfully acquired a slot
            }
            // Otherwise — someone else changed it, try again
        }
    }
}

fn main() {
    let limiter = Arc::new(RateLimiter::new(10)); // 10 requests per second

    let mut handles = vec![];

    // 5 threads trying to make requests
    for client_id in 0..5 {
        let limiter_clone = Arc::clone(&limiter);

        let handle = thread::spawn(move || {
            let mut success = 0;
            let mut rejected = 0;

            for _ in 0..10 {
                if limiter_clone.try_acquire() {
                    success += 1;
                } else {
                    rejected += 1;
                }
                thread::sleep(Duration::from_millis(50));
            }

            println!("Client {}: {} successful, {} rejected",
                client_id, success, rejected);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Atomics | Non-blocking thread-safe primitives |
| `AtomicBool/U64/I64` | Main atomic types |
| `load()` / `store()` | Reading and writing atomic values |
| `fetch_add()` | Atomic addition |
| `compare_exchange()` | CAS — conditional replacement |
| `Ordering` | Memory synchronization order |
| `SeqCst` | Strictest and safest Ordering |
| `Relaxed` | Minimal Ordering for counters |

## Practice Exercises

1. **Connection Counter**: Create a `ConnectionPool` structure with an atomic counter of active connections. Implement `connect()` (increments counter) and `disconnect()` (decrements) methods. Add a maximum connections limit.

2. **Atomic Best Bid/Ask**: Implement a `BestQuotes` structure with atomic fields for best buy and sell prices. Multiple threads should update prices, while others read the spread.

3. **Lock-free Order Queue**: Using `AtomicUsize` for indices, implement a simple fixed-size ring buffer for orders.

4. **Latency Monitoring**: Create a structure for tracking minimum, maximum, and average order execution times using only atomic operations.

## Homework

1. **Atomic Order ID Generator**: Implement a global unique ID generator for orders that safely works from any thread. The ID should include a timestamp and sequence number.

2. **Trading Circuit Breaker**: Create a "circuit breaker" that automatically stops trading if:
   - Per-minute loss exceeds a threshold
   - Number of API errors exceeds a limit
   Use atomic counters for all metrics.

3. **Lock-free Price Cache**: Implement a price cache for multiple assets where updates happen atomically. Support a "get all prices not older than N milliseconds" operation.

4. **Performance Comparison**: Write a benchmark comparing the performance of `AtomicU64` and `Mutex<u64>` for a counter with different thread counts (1, 2, 4, 8, 16). Plot the dependency graph.

## Navigation

[← Previous day](../165-avoiding-deadlock-lock-ordering/en.md) | [Next day →](../167-arc-atomic-reference-counting/en.md)
