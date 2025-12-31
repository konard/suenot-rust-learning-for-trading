# Day 169: Ordering: Visibility Guarantees

## Trading Analogy

Imagine an exchange where two traders work. Trader A updates the asset price on the screen, and Trader B makes decisions based on that price. It's critical that B sees the current price, not stale data from the cache.

In the world of processors, there's a similar problem: each core has its own cache, and changes to a variable in one thread may not be immediately visible in another. **Memory Ordering** defines what visibility guarantees we get when working with atomic operations.

In trading, this can manifest as:
- Thread A writes the new BTC price
- Thread B checks the "price updated" flag and reads the price
- Without proper ordering, Thread B might see flag = true but read the old price!

## What is Memory Ordering?

Memory Ordering is a set of guarantees about the order of memory operations between different threads. In Rust, the `std::sync::atomic::Ordering` enum is used for atomic types:

```rust
use std::sync::atomic::Ordering;

// Available options:
// Ordering::Relaxed   — minimal guarantees
// Ordering::Acquire   — guarantees when reading
// Ordering::Release   — guarantees when writing
// Ordering::AcqRel    — combination of Acquire + Release
// Ordering::SeqCst    — sequential consistency (maximum guarantees)
```

## Ordering::Relaxed — Minimal Guarantees

`Relaxed` guarantees only the atomicity of the operation — no guarantees about ordering relative to other operations. Suitable for counters where only the final value matters.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Counter of executed orders — we only care about the final number
    let orders_executed = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for trader_id in 0..4 {
        let counter = Arc::clone(&orders_executed);

        let handle = thread::spawn(move || {
            for order in 0..100 {
                // Relaxed is sufficient — we only need atomicity of the increment
                counter.fetch_add(1, Ordering::Relaxed);

                if order % 25 == 0 {
                    println!("Trader {}: executed order #{}", trader_id, order);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Final value is always correct
    println!("Total orders executed: {}",
        orders_executed.load(Ordering::Relaxed));
}
```

### When to Use Relaxed

- Simple counters (number of trades, requests)
- Statistics where only the final result matters
- Cases where operation order doesn't matter

## Ordering::Acquire and Ordering::Release — Data Synchronization

`Release` and `Acquire` work as a pair and provide a "happens-before" guarantee:
- **Release** on write: all previous writes are completed
- **Acquire** on read: all subsequent reads will see data after the Release

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Structure for publishing prices
struct PricePublisher {
    price: AtomicU64,        // Price in cents (to avoid f64)
    ready: AtomicBool,       // Ready flag
}

fn main() {
    let publisher = Arc::new(PricePublisher {
        price: AtomicU64::new(0),
        ready: AtomicBool::new(false),
    });

    let pub_clone = Arc::clone(&publisher);
    let sub_clone = Arc::clone(&publisher);

    // Publisher thread: updates the price
    let publisher_handle = thread::spawn(move || {
        // Simulate receiving a new price from the exchange
        thread::sleep(Duration::from_millis(50));

        let new_price = 42_500_00_u64; // $42,500.00 in cents

        // First write the price (regular atomic write)
        pub_clone.price.store(new_price, Ordering::Relaxed);

        // Release guarantees: ALL previous writes (including price)
        // will be visible after another thread reads ready = true with Acquire
        pub_clone.ready.store(true, Ordering::Release);

        println!("Publisher: Price updated to ${:.2}", new_price as f64 / 100.0);
    });

    // Subscriber thread: reads the price
    let subscriber_handle = thread::spawn(move || {
        // Wait until the price is ready
        loop {
            // Acquire guarantees: if we see ready = true,
            // we will also see ALL writes made BEFORE the Release
            if sub_clone.ready.load(Ordering::Acquire) {
                // Now it's safe to read the price — it's definitely current
                let price = sub_clone.price.load(Ordering::Relaxed);
                println!("Subscriber: Received price ${:.2}", price as f64 / 100.0);
                break;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    publisher_handle.join().unwrap();
    subscriber_handle.join().unwrap();
}
```

### Acquire-Release Visualization

```
Thread A (Publisher)          Thread B (Subscriber)
       |                            |
  price = 42500                     |
       |                            |
  [RELEASE]                         |
  ready = true  --------→    ready == true?
       |                     [ACQUIRE]
       |                            |
       |                     price = 42500 ✓
       |                            |
```

## Example: Trade Signal with Data

In HFT systems, you often need to pass not just a signal, but also associated data:

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradeSignal {
    // Signal data
    symbol_id: AtomicU64,
    price: AtomicU64,         // In cents
    quantity: AtomicI64,      // Positive = buy, negative = sell

    // Signal ready flag
    signal_ready: AtomicBool,
}

impl TradeSignal {
    fn new() -> Self {
        TradeSignal {
            symbol_id: AtomicU64::new(0),
            price: AtomicU64::new(0),
            quantity: AtomicI64::new(0),
            signal_ready: AtomicBool::new(false),
        }
    }

    // Publish signal (called from analysis thread)
    fn publish(&self, symbol: u64, price: u64, qty: i64) {
        // Write data (order between them doesn't matter)
        self.symbol_id.store(symbol, Ordering::Relaxed);
        self.price.store(price, Ordering::Relaxed);
        self.quantity.store(qty, Ordering::Relaxed);

        // Release — all writes above become visible
        self.signal_ready.store(true, Ordering::Release);
    }

    // Consume signal (called from execution thread)
    fn consume(&self) -> Option<(u64, u64, i64)> {
        // Acquire — if we see true, all data is current
        if self.signal_ready.load(Ordering::Acquire) {
            let symbol = self.symbol_id.load(Ordering::Relaxed);
            let price = self.price.load(Ordering::Relaxed);
            let qty = self.quantity.load(Ordering::Relaxed);

            // Reset flag (Relaxed is fine since we're the only reader)
            self.signal_ready.store(false, Ordering::Relaxed);

            Some((symbol, price, qty))
        } else {
            None
        }
    }
}

fn main() {
    let signal = Arc::new(TradeSignal::new());

    let signal_producer = Arc::clone(&signal);
    let signal_consumer = Arc::clone(&signal);

    // Market analysis thread
    let analyst = thread::spawn(move || {
        // Simulate analysis
        thread::sleep(Duration::from_millis(100));

        // BTC (id=1), price $42,500, buy 10 units
        signal_producer.publish(1, 42_500_00, 10);
        println!("Analyst: Buy signal published");

        thread::sleep(Duration::from_millis(100));

        // ETH (id=2), price $2,200, sell 50 units
        signal_producer.publish(2, 2_200_00, -50);
        println!("Analyst: Sell signal published");
    });

    // Order execution thread
    let executor = thread::spawn(move || {
        let mut executed = 0;

        while executed < 2 {
            if let Some((symbol, price, qty)) = signal_consumer.consume() {
                let action = if qty > 0 { "BUY" } else { "SELL" };
                let symbol_name = match symbol {
                    1 => "BTC",
                    2 => "ETH",
                    _ => "UNKNOWN",
                };

                println!(
                    "Executor: {} {} {} at ${:.2}",
                    action,
                    qty.abs(),
                    symbol_name,
                    price as f64 / 100.0
                );

                executed += 1;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    analyst.join().unwrap();
    executor.join().unwrap();
}
```

## Ordering::AcqRel — For Read-Modify-Write Operations

`AcqRel` combines `Acquire` and `Release` for operations like `fetch_add`, `compare_exchange`:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

struct OrderBook {
    // Best buy price (bid) and sell price (ask)
    best_bid: AtomicU64,
    best_ask: AtomicU64,
}

impl OrderBook {
    fn new(bid: u64, ask: u64) -> Self {
        OrderBook {
            best_bid: AtomicU64::new(bid),
            best_ask: AtomicU64::new(ask),
        }
    }

    // Attempt to improve bid (concurrent update)
    fn try_improve_bid(&self, new_bid: u64) -> bool {
        let mut current = self.best_bid.load(Ordering::Relaxed);

        loop {
            // New bid must be higher than current
            if new_bid <= current {
                return false;
            }

            // AcqRel: read current value (Acquire) and write new one (Release)
            match self.best_bid.compare_exchange_weak(
                current,
                new_bid,
                Ordering::AcqRel,  // Success: both Acquire and Release
                Ordering::Relaxed  // Failure: just read again
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    // Attempt to improve ask (must be lower)
    fn try_improve_ask(&self, new_ask: u64) -> bool {
        let mut current = self.best_ask.load(Ordering::Relaxed);

        loop {
            if new_ask >= current {
                return false;
            }

            match self.best_ask.compare_exchange_weak(
                current,
                new_ask,
                Ordering::AcqRel,
                Ordering::Relaxed
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    fn get_spread(&self) -> i64 {
        let ask = self.best_ask.load(Ordering::Acquire) as i64;
        let bid = self.best_bid.load(Ordering::Acquire) as i64;
        ask - bid
    }
}

fn main() {
    let book = Arc::new(OrderBook::new(42_000_00, 42_010_00));

    let mut handles = vec![];

    // Several market makers compete for best prices
    for mm_id in 0..4 {
        let book_clone = Arc::clone(&book);

        let handle = thread::spawn(move || {
            for i in 0..10 {
                // Each MM tries to improve bid and ask
                let new_bid = 42_000_00 + mm_id * 100 + i * 10;
                let new_ask = 42_010_00 - mm_id * 100 - i * 10;

                if book_clone.try_improve_bid(new_bid) {
                    println!(
                        "MM{}: Improved bid to ${:.2}",
                        mm_id,
                        new_bid as f64 / 100.0
                    );
                }

                if book_clone.try_improve_ask(new_ask) {
                    println!(
                        "MM{}: Improved ask to ${:.2}",
                        mm_id,
                        new_ask as f64 / 100.0
                    );
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "\nFinal spread: ${:.2}",
        book.get_spread() as f64 / 100.0
    );
}
```

## Ordering::SeqCst — Sequential Consistency

`SeqCst` (Sequentially Consistent) — the strictest mode. Guarantees a single global order of all operations for all threads. Use when order between different atomic variables is critically important.

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingHalt {
    halt_flag: AtomicBool,
    last_price: AtomicU64,
    halt_reason: AtomicU64, // 1 = volatility, 2 = news, 3 = technical failure
}

impl TradingHalt {
    fn new() -> Self {
        TradingHalt {
            halt_flag: AtomicBool::new(false),
            last_price: AtomicU64::new(0),
            halt_reason: AtomicU64::new(0),
        }
    }

    // Halt trading — all threads must see consistent state
    fn halt_trading(&self, price: u64, reason: u64) {
        // SeqCst guarantees that all threads see these operations in the same order
        self.last_price.store(price, Ordering::SeqCst);
        self.halt_reason.store(reason, Ordering::SeqCst);
        self.halt_flag.store(true, Ordering::SeqCst);
    }

    fn is_halted(&self) -> bool {
        self.halt_flag.load(Ordering::SeqCst)
    }

    fn get_halt_info(&self) -> (u64, u64) {
        // SeqCst guarantees consistent reads
        let price = self.last_price.load(Ordering::SeqCst);
        let reason = self.halt_reason.load(Ordering::SeqCst);
        (price, reason)
    }

    fn resume_trading(&self) {
        self.halt_flag.store(false, Ordering::SeqCst);
    }
}

fn main() {
    let halt_system = Arc::new(TradingHalt::new());

    // Monitoring thread — watches for volatility
    let monitor = {
        let system = Arc::clone(&halt_system);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));

            // High volatility detected!
            println!("Monitor: High volatility detected, halting trading!");
            system.halt_trading(41_500_00, 1);
        })
    };

    // Multiple trading threads
    let mut traders = vec![];
    for id in 0..3 {
        let system = Arc::clone(&halt_system);
        traders.push(thread::spawn(move || {
            for i in 0..10 {
                if system.is_halted() {
                    let (price, reason) = system.get_halt_info();
                    let reason_str = match reason {
                        1 => "volatility",
                        2 => "news",
                        3 => "tech failure",
                        _ => "unknown",
                    };
                    println!(
                        "Trader {}: Trading halted! Reason: {}, price: ${:.2}",
                        id,
                        reason_str,
                        price as f64 / 100.0
                    );
                    break;
                }

                println!("Trader {}: Working, iteration {}", id, i);
                thread::sleep(Duration::from_millis(30));
            }
        }));
    }

    monitor.join().unwrap();
    for t in traders {
        t.join().unwrap();
    }
}
```

## Ordering Comparison

| Ordering | Guarantees | Performance | Use Case |
|----------|------------|-------------|----------|
| Relaxed | Atomicity only | Maximum | Counters, statistics |
| Acquire | Visibility of writes before Release | High | Reading flags, pointers |
| Release | Previous writes visible after Acquire | High | Publishing data |
| AcqRel | Acquire + Release | Medium | compare_exchange, fetch_* |
| SeqCst | Global ordering | Low | Critical sections, synchronization algorithms |

## Practical Example: Lock-free Price Queue

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const QUEUE_SIZE: usize = 16;

struct PriceQueue {
    buffer: [AtomicU64; QUEUE_SIZE],
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

impl PriceQueue {
    fn new() -> Self {
        PriceQueue {
            buffer: std::array::from_fn(|_| AtomicU64::new(0)),
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
        }
    }

    // Producer writes a price
    fn push(&self, price: u64) -> bool {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Acquire);

        // Check if there's room
        if write.wrapping_sub(read) >= QUEUE_SIZE {
            return false; // Queue is full
        }

        let index = write % QUEUE_SIZE;

        // Write the price
        self.buffer[index].store(price, Ordering::Relaxed);

        // Release: price write is visible after position increment
        self.write_pos.store(write.wrapping_add(1), Ordering::Release);

        true
    }

    // Consumer reads a price
    fn pop(&self) -> Option<u64> {
        let read = self.read_pos.load(Ordering::Relaxed);

        // Acquire: see all writes before Release in write_pos
        let write = self.write_pos.load(Ordering::Acquire);

        if read == write {
            return None; // Queue is empty
        }

        let index = read % QUEUE_SIZE;
        let price = self.buffer[index].load(Ordering::Relaxed);

        // Release: next consumer will see the updated position
        self.read_pos.store(read.wrapping_add(1), Ordering::Release);

        Some(price)
    }
}

fn main() {
    let queue = Arc::new(PriceQueue::new());

    // Producer: feeds prices from the exchange
    let producer = {
        let q = Arc::clone(&queue);
        thread::spawn(move || {
            for i in 0..20 {
                let price = 42_000_00 + i * 100; // Price is rising

                while !q.push(price) {
                    // Queue is full, wait
                    thread::sleep(Duration::from_micros(100));
                }

                println!("Producer: wrote price ${:.2}", price as f64 / 100.0);
                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    // Consumer: processes prices
    let consumer = {
        let q = Arc::clone(&queue);
        thread::spawn(move || {
            let mut count = 0;
            while count < 20 {
                if let Some(price) = q.pop() {
                    println!("Consumer: processed price ${:.2}", price as f64 / 100.0);
                    count += 1;
                } else {
                    thread::sleep(Duration::from_micros(100));
                }
            }
        })
    };

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("All prices processed!");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Memory Ordering | Guarantees of change visibility between threads |
| Relaxed | Atomicity only, no ordering guarantees |
| Acquire | Guarantees visibility of all writes before Release |
| Release | Makes previous writes visible to Acquire |
| AcqRel | Combination for read-modify-write operations |
| SeqCst | Global ordering, maximum guarantees |

## Homework

1. **Counter with Guarantees**: Create a `TradeCounter` structure with atomic counters for buys and sells. Implement a `get_net_position()` method that returns the difference. What Ordering is needed for each operation?

2. **OHLCV Publication**: Implement an `OHLCVPublisher` structure with Open, High, Low, Close, Volume fields (all AtomicU64). One thread updates data, another reads. Use Acquire-Release to guarantee consistency.

3. **Emergency Stop Flag**: Create a system with three threads:
   - Thread A sets the stop flag
   - Thread B checks the flag and reads the stop reason
   - Thread C checks the flag and reads the stop time
   Use SeqCst so all threads see consistent state.

4. **Lock-free Stack**: Modify the queue example to implement a LIFO price stack. Use compare_exchange_weak with proper Ordering.

## Navigation

[← Previous Day](../168-atomic-compare-exchange/en.md) | [Next Day →](../170-memory-barriers/en.md)
