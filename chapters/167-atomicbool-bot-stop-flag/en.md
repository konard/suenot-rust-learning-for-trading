# Day 167: AtomicBool: Bot Stop Flag

## Trading Analogy

Imagine you have a trading bot that checks the market every second and executes trades. You want to be able to safely stop it at any moment — for example, during a sudden market crash, when reaching a loss limit, or simply at the end of a trading session.

How do you implement this? You need a **stop flag** — a variable that one thread (main) can set to `true`, while another thread (bot) constantly checks it. If the flag becomes `true`, the bot stops working.

The problem is that a regular `bool` variable is not safe for multi-threaded access:
- One thread may read a stale value while another is changing it
- The compiler may cache the value in a register
- The CPU may reorder operations

**AtomicBool** solves these problems — it's an atomic (indivisible) boolean type that's safe to use from multiple threads without mutexes.

## What is AtomicBool?

`AtomicBool` is a synchronization primitive from the `std::sync::atomic` module. It guarantees:

1. **Atomicity** — read and write operations are indivisible
2. **Visibility** — changes are immediately visible to all threads
3. **No data races** — safe without locks

```rust
use std::sync::atomic::{AtomicBool, Ordering};

// Creating an AtomicBool
let flag = AtomicBool::new(false);

// Reading the value
let value = flag.load(Ordering::Relaxed);

// Writing a value
flag.store(true, Ordering::Relaxed);
```

## Ordering: Memory Ordering

When working with atomic types, you need to specify `Ordering` — this tells the compiler and CPU what ordering guarantees are needed:

| Ordering | Description | When to use |
|----------|-------------|-------------|
| `Relaxed` | No ordering guarantees | Simple counters, flags without dependencies |
| `Acquire` | All writes before Release are visible | Reading data protected by a flag |
| `Release` | Writes are visible after Acquire | Writing data protected by a flag |
| `SeqCst` | Full sequential consistency | When maximum strictness is needed |

For a simple stop flag, `Relaxed` is usually sufficient.

## Simple Example: Stop Flag

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Stop flag shared between threads
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Clone for the worker thread
    let worker_flag = Arc::clone(&stop_flag);

    // Worker thread (trading bot)
    let worker = thread::spawn(move || {
        let mut iteration = 0;

        // Work while the flag is not set
        while !worker_flag.load(Ordering::Relaxed) {
            iteration += 1;
            println!("Bot: iteration {}, checking market...", iteration);

            // Simulate work
            thread::sleep(Duration::from_millis(200));
        }

        println!("Bot: stop signal received, shutting down");
        iteration
    });

    // Main thread waits a bit, then stops the bot
    thread::sleep(Duration::from_secs(1));
    println!("Main: sending stop signal");
    stop_flag.store(true, Ordering::Relaxed);

    // Wait for the bot to finish
    let iterations = worker.join().unwrap();
    println!("Bot completed {} iterations", iterations);
}
```

## Trading Bot with Stop Flag

Let's look at a more realistic example:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct TradingBot {
    stop_flag: Arc<AtomicBool>,
    symbol: String,
    max_trades: u32,
    trades_executed: u32,
    total_pnl: f64,
}

impl TradingBot {
    fn new(stop_flag: Arc<AtomicBool>, symbol: &str, max_trades: u32) -> Self {
        TradingBot {
            stop_flag,
            symbol: symbol.to_string(),
            max_trades,
            trades_executed: 0,
            total_pnl: 0.0,
        }
    }

    fn run(&mut self) {
        println!("[{}] Bot started", self.symbol);

        while !self.should_stop() {
            // Check market conditions (simulation)
            let price = self.get_market_price();

            // Make trading decision
            if self.should_trade(price) {
                self.execute_trade(price);
            }

            // Pause between iterations
            thread::sleep(Duration::from_millis(100));
        }

        println!("[{}] Bot stopped. Trades: {}, PnL: ${:.2}",
            self.symbol, self.trades_executed, self.total_pnl);
    }

    fn should_stop(&self) -> bool {
        // Stop if:
        // 1. Stop signal received
        // 2. Trade limit reached
        self.stop_flag.load(Ordering::Relaxed) ||
        self.trades_executed >= self.max_trades
    }

    fn get_market_price(&self) -> f64 {
        // Simulate getting price
        42000.0 + (rand_simple() * 1000.0 - 500.0)
    }

    fn should_trade(&self, price: f64) -> bool {
        // Simple strategy: trade with 30% probability
        rand_simple() < 0.3
    }

    fn execute_trade(&mut self, price: f64) {
        self.trades_executed += 1;

        // Simulate trade result
        let pnl = (rand_simple() - 0.5) * 100.0;
        self.total_pnl += pnl;

        println!("[{}] Trade #{} at ${:.2}, PnL: ${:.2}",
            self.symbol, self.trades_executed, price, pnl);
    }
}

// Simple random number generator
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Launch multiple bots for different pairs
    let mut handles = vec![];

    for symbol in ["BTC/USDT", "ETH/USDT", "SOL/USDT"] {
        let flag = Arc::clone(&stop_flag);
        let sym = symbol.to_string();

        let handle = thread::spawn(move || {
            let mut bot = TradingBot::new(flag, &sym, 10);
            bot.run();
        });

        handles.push(handle);
    }

    // Wait 2 seconds and stop all bots
    thread::sleep(Duration::from_secs(2));
    println!("\n=== Sending stop signal to all bots ===\n");
    stop_flag.store(true, Ordering::Relaxed);

    // Wait for all bots to finish
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nAll bots stopped");
}
```

## AtomicBool Methods

AtomicBool provides several useful methods:

### Basic Operations

```rust
use std::sync::atomic::{AtomicBool, Ordering};

let flag = AtomicBool::new(false);

// load — read the value
let value = flag.load(Ordering::Relaxed);

// store — write a value
flag.store(true, Ordering::Relaxed);

// swap — write new value and return the old one
let old = flag.swap(false, Ordering::Relaxed);
println!("Was: {}, now: false", old);
```

### Conditional Operations

```rust
use std::sync::atomic::{AtomicBool, Ordering};

let flag = AtomicBool::new(false);

// compare_exchange — atomic compare and swap
// If flag == false, write true and return Ok(false)
// Otherwise return Err(current value)
match flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
    Ok(old) => println!("Successfully changed from {} to true", old),
    Err(current) => println!("Not changed, current value: {}", current),
}

// fetch_and — atomic AND
let result = flag.fetch_and(true, Ordering::Relaxed);

// fetch_or — atomic OR
let result = flag.fetch_or(false, Ordering::Relaxed);

// fetch_xor — atomic XOR
let result = flag.fetch_xor(true, Ordering::Relaxed);

// fetch_nand — atomic NAND
let result = flag.fetch_nand(true, Ordering::Relaxed);
```

## Practical Example: Graceful Shutdown

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct GracefulBot {
    running: Arc<AtomicBool>,
    shutdown_requested: Arc<AtomicBool>,
}

impl GracefulBot {
    fn new() -> Self {
        GracefulBot {
            running: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start(&self) -> thread::JoinHandle<()> {
        let running = Arc::clone(&self.running);
        let shutdown = Arc::clone(&self.shutdown_requested);

        thread::spawn(move || {
            // Mark that the bot is running
            running.store(true, Ordering::Release);
            println!("Bot: started and ready to work");

            let mut pending_orders = vec![];

            loop {
                // Check for shutdown request
                if shutdown.load(Ordering::Acquire) {
                    println!("Bot: shutdown request received");

                    // Graceful shutdown: complete current operations
                    if !pending_orders.is_empty() {
                        println!("Bot: completing {} pending orders", pending_orders.len());
                        for order in pending_orders.drain(..) {
                            println!("  - Cancelled order: {}", order);
                            thread::sleep(Duration::from_millis(50));
                        }
                    }

                    break;
                }

                // Simulate work
                let order_id = rand_simple() as u64;
                if rand_simple() < 0.3 {
                    pending_orders.push(format!("ORD-{}", order_id));
                    println!("Bot: created order ORD-{}", order_id);
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Mark that the bot is stopped
            running.store(false, Ordering::Release);
            println!("Bot: fully stopped");
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    fn request_shutdown(&self) {
        println!("Controller: sending shutdown request");
        self.shutdown_requested.store(true, Ordering::Release);
    }
}

// Simple random number generator
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    let bot = GracefulBot::new();

    let handle = bot.start();

    // Wait for the bot to start
    while !bot.is_running() {
        thread::sleep(Duration::from_millis(10));
    }

    println!("Controller: bot is working, waiting 1 second...\n");
    thread::sleep(Duration::from_secs(1));

    // Request shutdown
    bot.request_shutdown();

    // Wait for completion
    handle.join().unwrap();

    println!("\nController: bot successfully stopped");
}
```

## Example: Multiple State Flags

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingState {
    // System state flags
    market_open: AtomicBool,
    trading_enabled: AtomicBool,
    risk_limit_hit: AtomicBool,
    maintenance_mode: AtomicBool,
}

impl TradingState {
    fn new() -> Self {
        TradingState {
            market_open: AtomicBool::new(true),
            trading_enabled: AtomicBool::new(true),
            risk_limit_hit: AtomicBool::new(false),
            maintenance_mode: AtomicBool::new(false),
        }
    }

    fn can_trade(&self) -> bool {
        self.market_open.load(Ordering::Relaxed) &&
        self.trading_enabled.load(Ordering::Relaxed) &&
        !self.risk_limit_hit.load(Ordering::Relaxed) &&
        !self.maintenance_mode.load(Ordering::Relaxed)
    }

    fn get_status(&self) -> String {
        format!(
            "Market: {}, Trading: {}, Risk: {}, Maintenance: {}",
            if self.market_open.load(Ordering::Relaxed) { "open" } else { "closed" },
            if self.trading_enabled.load(Ordering::Relaxed) { "on" } else { "off" },
            if self.risk_limit_hit.load(Ordering::Relaxed) { "limit!" } else { "ok" },
            if self.maintenance_mode.load(Ordering::Relaxed) { "on" } else { "off" }
        )
    }
}

fn main() {
    let state = Arc::new(TradingState::new());

    // Trader thread
    let trader_state = Arc::clone(&state);
    let trader = thread::spawn(move || {
        for i in 1..=10 {
            if trader_state.can_trade() {
                println!("Trader: executing trade #{}", i);
            } else {
                println!("Trader: trading unavailable - {}", trader_state.get_status());
            }
            thread::sleep(Duration::from_millis(200));
        }
    });

    // Risk controller thread
    let risk_state = Arc::clone(&state);
    let risk_manager = thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        println!("\nRisk Manager: risk limit reached!");
        risk_state.risk_limit_hit.store(true, Ordering::Relaxed);

        thread::sleep(Duration::from_millis(800));
        println!("Risk Manager: risk limit reset\n");
        risk_state.risk_limit_hit.store(false, Ordering::Relaxed);
    });

    trader.join().unwrap();
    risk_manager.join().unwrap();

    println!("\nFinal state: {}", state.get_status());
}
```

## AtomicBool vs Mutex<bool>

| Characteristic | AtomicBool | Mutex\<bool\> |
|----------------|------------|---------------|
| Blocking | No (lock-free) | Yes |
| Performance | Very high | Lower |
| Complexity | Simple | More complex |
| Capabilities | Boolean value only | Any operations |
| Deadlock | Impossible | Possible |

Use `AtomicBool` when:
- You need a simple flag (on/off)
- Maximum performance is important
- You don't need compound operations

Use `Mutex<bool>` when:
- You need to protect multiple related variables
- You need compound operations (check + modify multiple values)

## What We Learned

| Concept | Description |
|---------|-------------|
| AtomicBool | Atomic boolean type for multi-threaded access |
| Ordering | Memory ordering guarantees |
| load/store | Basic read and write operations |
| swap | Atomic value exchange |
| compare_exchange | Conditional atomic modification |
| Lock-free | Working without locks (mutexes) |

## Homework

1. **Trading Bot with Multiple Modes**: Implement a bot with three states (AtomicBool each):
   - `aggressive_mode` — aggressive trading
   - `safe_mode` — conservative trading
   - `paused` — paused

   The bot should check all flags and change behavior accordingly.

2. **Health Monitoring System**: Create a `HealthMonitor` struct with methods:
   - `set_healthy(service: &str, healthy: bool)` — set service status
   - `is_all_healthy()` — check if all services are working
   - `get_unhealthy_services()` — get list of non-working services

3. **Atomic Toggle**: Using `compare_exchange`, implement a `toggle_once` function that:
   - Toggles the flag only if it's in the initial state
   - Returns `true` if toggle succeeded, `false` if already toggled
   - Guarantees that only one thread successfully toggles the flag

4. **Rate Limiter**: Implement a `RateLimiter` struct with AtomicBool for a "too many requests" flag:
   - A monitor thread counts requests and sets the flag when limit is exceeded
   - Worker threads check the flag before performing operations
   - Automatic flag reset after a specified interval

## Navigation

[← Previous day](../166-atomics-lock-free-counters/en.md) | [Next day →](../168-atomic-ordering-memory-barriers/en.md)
