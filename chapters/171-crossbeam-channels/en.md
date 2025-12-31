# Day 171: crossbeam channels: Faster Than mpsc

## Trading Analogy

Imagine a high-frequency trading system. Standard `std::sync::mpsc` channels are like a post office: reliable, but slow. While `crossbeam-channel` is like a direct fiber optic line between the exchange and your trading server: data flies with minimal latency.

In real trading, every microsecond counts:
- Market data feed generates thousands of price updates per second
- Trading engine must receive this data instantly
- Strategy analyzes and sends orders without delays

Standard `mpsc` can become a bottleneck, while `crossbeam-channel` solves this problem thanks to lock-free algorithms.

## Why is crossbeam-channel faster?

| Feature | std::sync::mpsc | crossbeam-channel |
|---------|-----------------|-------------------|
| Algorithm | Blocking | Lock-free |
| Producers | Many (mpsc) | Many (mpmc) |
| Consumers | One | Many |
| Bounded/Unbounded | Unbounded only | Both |
| Select | No | Yes |
| Zero-capacity | No | Yes |

## Installing crossbeam-channel

Add to your `Cargo.toml`:

```toml
[dependencies]
crossbeam-channel = "0.5"
```

## Basic Usage

### Creating Channels

```rust
use crossbeam_channel::{unbounded, bounded};

fn main() {
    // Unbounded channel (like std::sync::mpsc::channel)
    let (tx, rx) = unbounded::<f64>();

    // Bounded channel with buffer for 100 messages
    let (tx_bounded, rx_bounded) = bounded::<f64>(100);

    // Zero-capacity channel (rendezvous) — sender waits for receiver
    let (tx_zero, rx_zero) = bounded::<f64>(0);

    // Send BTC price
    tx.send(42500.0).unwrap();

    // Receive price
    let price = rx.recv().unwrap();
    println!("Received BTC price: ${}", price);
}
```

## Example: Market Data Feed

```rust
use crossbeam_channel::{bounded, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

fn market_data_producer(tx: Sender<MarketTick>, symbol: &str) {
    let mut timestamp = 0u64;
    let base_price = match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    for i in 0..1000 {
        let spread = 0.01 * base_price; // 1% spread
        let variation = (i as f64 * 0.1).sin() * base_price * 0.001;

        let tick = MarketTick {
            symbol: symbol.to_string(),
            bid: base_price + variation,
            ask: base_price + variation + spread,
            timestamp,
        };

        if tx.send(tick).is_err() {
            break; // Channel closed
        }
        timestamp += 1;
    }
}

fn trading_strategy(rx: Receiver<MarketTick>) -> Vec<String> {
    let mut signals = Vec::new();
    let mut last_prices: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    while let Ok(tick) = rx.recv() {
        let mid_price = (tick.bid + tick.ask) / 2.0;

        if let Some(&prev_price) = last_prices.get(&tick.symbol) {
            let change_pct = (mid_price - prev_price) / prev_price * 100.0;

            if change_pct > 0.05 {
                signals.push(format!("BUY {} @ {:.2}", tick.symbol, tick.ask));
            } else if change_pct < -0.05 {
                signals.push(format!("SELL {} @ {:.2}", tick.symbol, tick.bid));
            }
        }

        last_prices.insert(tick.symbol.clone(), mid_price);
    }

    signals
}

fn main() {
    // Bounded channel — don't let buffer grow uncontrollably
    let (tx, rx) = bounded::<MarketTick>(1000);

    let start = Instant::now();

    // Start producers for different symbols
    let tx_btc = tx.clone();
    let btc_producer = thread::spawn(move || {
        market_data_producer(tx_btc, "BTC");
    });

    let tx_eth = tx.clone();
    let eth_producer = thread::spawn(move || {
        market_data_producer(tx_eth, "ETH");
    });

    // Close original sender — otherwise channel never closes
    drop(tx);

    // Start strategy
    let strategy = thread::spawn(move || {
        trading_strategy(rx)
    });

    btc_producer.join().unwrap();
    eth_producer.join().unwrap();

    let signals = strategy.join().unwrap();

    let elapsed = start.elapsed();

    println!("Processed in {:?}", elapsed);
    println!("Total signals: {}", signals.len());
    println!("Sample signals:");
    for signal in signals.iter().take(5) {
        println!("  {}", signal);
    }
}
```

## MPMC: Multiple Consumers

The main advantage of `crossbeam-channel` — support for multiple consumers (Multi-Producer Multi-Consumer):

```rust
use crossbeam_channel::bounded;
use std::thread;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn order_processor(id: u32, rx: crossbeam_channel::Receiver<Order>) {
    while let Ok(order) = rx.recv() {
        println!(
            "Processor {}: Processing order #{} - {} {} {} @ {:.2}",
            id, order.id, order.side, order.quantity, order.symbol, order.price
        );
        // Simulate processing
        thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("Processor {}: Shutting down", id);
}

fn main() {
    let (tx, rx) = bounded::<Order>(100);

    // Start 4 order processors
    let mut processors = vec![];
    for i in 0..4 {
        let rx_clone = rx.clone();
        processors.push(thread::spawn(move || {
            order_processor(i, rx_clone);
        }));
    }

    // Send orders
    for id in 0..20 {
        let order = Order {
            id,
            symbol: if id % 2 == 0 { "BTC".to_string() } else { "ETH".to_string() },
            side: if id % 3 == 0 { "BUY".to_string() } else { "SELL".to_string() },
            price: 42000.0 + id as f64 * 100.0,
            quantity: 0.1 + (id as f64 * 0.01),
        };
        tx.send(order).unwrap();
    }

    // Close channel
    drop(tx);

    // Wait for all processors to finish
    for p in processors {
        p.join().unwrap();
    }

    println!("All orders processed!");
}
```

## Select: Waiting on Multiple Channels

`crossbeam_channel::select!` allows waiting on multiple channels simultaneously:

```rust
use crossbeam_channel::{bounded, select, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    RiskAlert { message: String },
}

fn price_feed(tx: Sender<TradingEvent>) {
    for i in 0..10 {
        thread::sleep(Duration::from_millis(100));
        tx.send(TradingEvent::PriceUpdate {
            symbol: "BTC".to_string(),
            price: 42000.0 + i as f64 * 10.0,
        }).ok();
    }
}

fn order_executor(tx: Sender<TradingEvent>) {
    for id in 1..=3 {
        thread::sleep(Duration::from_millis(300));
        tx.send(TradingEvent::OrderFilled {
            order_id: id,
            price: 42000.0 + id as f64 * 50.0,
        }).ok();
    }
}

fn risk_monitor(tx: Sender<TradingEvent>) {
    thread::sleep(Duration::from_millis(500));
    tx.send(TradingEvent::RiskAlert {
        message: "High volatility detected!".to_string(),
    }).ok();
}

fn main() {
    let (price_tx, price_rx) = bounded::<TradingEvent>(10);
    let (order_tx, order_rx) = bounded::<TradingEvent>(10);
    let (risk_tx, risk_rx) = bounded::<TradingEvent>(10);

    // Start event sources
    let h1 = thread::spawn(move || price_feed(price_tx));
    let h2 = thread::spawn(move || order_executor(order_tx));
    let h3 = thread::spawn(move || risk_monitor(risk_tx));

    // Main event processing loop
    let mut running = true;
    let mut event_count = 0;

    while running && event_count < 20 {
        select! {
            recv(price_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("Price: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            recv(order_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("Order: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            recv(risk_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("Risk: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            default(Duration::from_millis(1000)) => {
                println!("Timeout — no events");
                running = false;
            }
        }
    }

    h1.join().ok();
    h2.join().ok();
    h3.join().ok();

    println!("\nEvents processed: {}", event_count);
}
```

## Bounded vs Unbounded: Memory Control

```rust
use crossbeam_channel::{bounded, unbounded, TrySendError};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceQuote {
    symbol: String,
    price: f64,
}

fn main() {
    // Bounded channel — protection from memory overflow
    let (tx, rx) = bounded::<PriceQuote>(5);

    // Fast producer
    let producer = thread::spawn(move || {
        for i in 0..20 {
            let quote = PriceQuote {
                symbol: "BTC".to_string(),
                price: 42000.0 + i as f64,
            };

            // try_send doesn't block if buffer is full
            match tx.try_send(quote) {
                Ok(_) => println!("Sent: price #{}", i),
                Err(TrySendError::Full(q)) => {
                    println!("Buffer full! Skipping price: {:.2}", q.price);
                }
                Err(TrySendError::Disconnected(_)) => {
                    println!("Channel closed!");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    });

    // Slow consumer
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(quote) = rx.recv() {
            println!("Received: {} @ {:.2}", quote.symbol, quote.price);
            count += 1;
            // Slow processing
            thread::sleep(Duration::from_millis(200));
        }
        println!("Total processed: {}", count);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Practical Example: Trading Engine with Priorities

```rust
use crossbeam_channel::{bounded, select, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum Priority {
    High,   // Stop-losses, risk alerts
    Normal, // Regular orders
    Low,    // Reports, logging
}

#[derive(Debug)]
struct Task {
    id: u64,
    priority: Priority,
    description: String,
}

struct PriorityTaskQueue {
    high_rx: Receiver<Task>,
    normal_rx: Receiver<Task>,
    low_rx: Receiver<Task>,
}

impl PriorityTaskQueue {
    fn recv(&self) -> Option<Task> {
        // Priority select — first check high, then normal, then low
        select! {
            recv(self.high_rx) -> task => task.ok(),
            recv(self.normal_rx) -> task => task.ok(),
            recv(self.low_rx) -> task => task.ok(),
            default(Duration::from_millis(100)) => None,
        }
    }
}

fn main() {
    let (high_tx, high_rx) = bounded::<Task>(10);
    let (normal_tx, normal_rx) = bounded::<Task>(100);
    let (low_tx, low_rx) = bounded::<Task>(100);

    let queue = PriorityTaskQueue {
        high_rx,
        normal_rx,
        low_rx,
    };

    // Task generator
    let task_generator = {
        let high_tx = high_tx.clone();
        let normal_tx = normal_tx.clone();
        let low_tx = low_tx.clone();

        thread::spawn(move || {
            for id in 0..15 {
                let (tx, priority, desc) = match id % 5 {
                    0 => (&high_tx, Priority::High, "STOP-LOSS TRIGGERED!"),
                    1 | 2 => (&normal_tx, Priority::Normal, "Regular order"),
                    _ => (&low_tx, Priority::Low, "Log entry"),
                };

                tx.send(Task {
                    id,
                    priority: priority.clone(),
                    description: desc.to_string(),
                }).ok();

                thread::sleep(Duration::from_millis(50));
            }

            drop(high_tx);
            drop(normal_tx);
            drop(low_tx);
        })
    };

    // Close original senders
    drop(high_tx);
    drop(normal_tx);
    drop(low_tx);

    // Task processor
    let processor = thread::spawn(move || {
        let mut processed = 0;
        let start = Instant::now();

        loop {
            match queue.recv() {
                Some(task) => {
                    let priority_str = match task.priority {
                        Priority::High => "HIGH",
                        Priority::Normal => "NORMAL",
                        Priority::Low => "LOW",
                    };
                    println!("[{}] Task #{}: {}", priority_str, task.id, task.description);
                    processed += 1;
                    thread::sleep(Duration::from_millis(30));
                }
                None => {
                    if start.elapsed() > Duration::from_secs(2) {
                        break;
                    }
                }
            }
        }

        processed
    });

    task_generator.join().unwrap();
    let total = processor.join().unwrap();

    println!("\nTotal tasks processed: {}", total);
}
```

## Performance Comparison

```rust
use crossbeam_channel::bounded;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const NUM_MESSAGES: usize = 1_000_000;
const NUM_PRODUCERS: usize = 4;

fn bench_std_mpsc() -> Duration {
    let (tx, rx) = mpsc::channel::<u64>();
    let start = Instant::now();

    let producers: Vec<_> = (0..NUM_PRODUCERS)
        .map(|_| {
            let tx = tx.clone();
            thread::spawn(move || {
                for i in 0..(NUM_MESSAGES / NUM_PRODUCERS) as u64 {
                    tx.send(i).unwrap();
                }
            })
        })
        .collect();

    drop(tx);

    let consumer = thread::spawn(move || {
        let mut count = 0u64;
        while rx.recv().is_ok() {
            count += 1;
        }
        count
    });

    for p in producers {
        p.join().unwrap();
    }
    consumer.join().unwrap();

    start.elapsed()
}

fn bench_crossbeam() -> Duration {
    let (tx, rx) = bounded::<u64>(10000);
    let start = Instant::now();

    let producers: Vec<_> = (0..NUM_PRODUCERS)
        .map(|_| {
            let tx = tx.clone();
            thread::spawn(move || {
                for i in 0..(NUM_MESSAGES / NUM_PRODUCERS) as u64 {
                    tx.send(i).unwrap();
                }
            })
        })
        .collect();

    drop(tx);

    let consumer = thread::spawn(move || {
        let mut count = 0u64;
        while rx.recv().is_ok() {
            count += 1;
        }
        count
    });

    for p in producers {
        p.join().unwrap();
    }
    consumer.join().unwrap();

    start.elapsed()
}

use std::time::Duration;

fn main() {
    println!("Benchmark: {} messages, {} producers", NUM_MESSAGES, NUM_PRODUCERS);
    println!();

    let std_time = bench_std_mpsc();
    println!("std::sync::mpsc:     {:?}", std_time);

    let crossbeam_time = bench_crossbeam();
    println!("crossbeam-channel:   {:?}", crossbeam_time);

    let speedup = std_time.as_nanos() as f64 / crossbeam_time.as_nanos() as f64;
    println!();
    println!("crossbeam is {:.2}x faster", speedup);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `crossbeam-channel` | Fast lock-free alternative to std::sync::mpsc |
| MPMC | Support for multiple producers AND multiple consumers |
| `bounded(n)` | Channel with limited buffer |
| `unbounded()` | Channel without buffer limit |
| `select!` | Macro for waiting on multiple channels |
| `try_send` | Non-blocking send |
| Zero-capacity | Rendezvous channel for synchronization |

## Homework

1. **Market Data Aggregator**: Implement a system that:
   - Receives data from 5 different sources (threads)
   - Aggregates prices for each symbol
   - Sends averaged price to the strategy
   - Uses bounded channels for overload protection

2. **Order Router**: Create an order router with:
   - Channel for incoming orders
   - 3 channels for different exchanges (by instrument type)
   - Best price selection logic
   - Performance metrics (orders/sec)

3. **Performance Comparison**: Write a benchmark comparing:
   - `std::sync::mpsc` vs `crossbeam-channel`
   - Bounded vs Unbounded channels
   - Different buffer sizes (10, 100, 1000, 10000)

4. **Event Sourcing**: Implement an event system for trading:
   - All state changes through events in channel
   - Multiple handlers subscribed to events
   - Event persistence to file
   - State recovery on startup

## Navigation

[← Previous day](../170-crossbeam-advanced-concurrency/en.md) | [Next day →](../172-crossbeam-scope/en.md)
