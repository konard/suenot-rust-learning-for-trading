# Day 159: sync_channel — Bounded Queue

## Trading Analogy

Imagine an order book with limited depth. A market maker can only place a certain number of orders. When the book is full, new orders aren't accepted until previous ones are executed. This is exactly what **sync_channel** is — a channel with limited capacity where the sender blocks if the buffer is full.

Unlike regular `channel()` which creates an unbounded queue, `sync_channel(n)` creates a queue with fixed size `n`. This is critical for trading systems that need **back-pressure** — a mechanism to slow down fast data sources.

## Basic sync_channel Usage

```rust
use std::sync::mpsc::sync_channel;
use std::thread;

fn main() {
    // Create a channel with buffer for 3 messages
    let (sender, receiver) = sync_channel::<f64>(3);

    // Producer thread: sends prices
    let producer = thread::spawn(move || {
        let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

        for price in prices {
            println!("[Producer] Sending price: {}", price);
            sender.send(price).unwrap();
            println!("[Producer] Price {} sent", price);
        }
    });

    // Consumer thread: processes prices
    let consumer = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(100));

        while let Ok(price) = receiver.recv() {
            println!("[Consumer] Received price: {}", price);
            // Simulate processing
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

**Important:** When the buffer is full (3 messages), `send()` blocks until space becomes available.

## Difference Between channel and sync_channel

```rust
use std::sync::mpsc::{channel, sync_channel};
use std::thread;
use std::time::Instant;

fn main() {
    // Regular channel — unbounded buffer
    println!("=== Regular channel ===");
    let (tx, rx) = channel::<i32>();
    let start = Instant::now();

    for i in 0..1000 {
        tx.send(i).unwrap();  // Never blocks
    }
    println!("1000 messages sent in {:?}", start.elapsed());
    drop(tx);
    drop(rx);

    // Synchronous channel — bounded buffer
    println!("\n=== sync_channel(10) ===");
    let (tx, rx) = sync_channel::<i32>(10);

    let sender = thread::spawn(move || {
        let start = Instant::now();
        for i in 0..100 {
            tx.send(i).unwrap();  // Blocks when buffer is full
        }
        println!("100 messages sent in {:?}", start.elapsed());
    });

    let receiver = thread::spawn(move || {
        while let Ok(_) = rx.recv() {
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();
}
```

## sync_channel(0) — Rendezvous Channel

```rust
use std::sync::mpsc::sync_channel;
use std::thread;

fn main() {
    // Zero-buffer channel — synchronous transfer
    let (sender, receiver) = sync_channel::<(String, f64)>(0);

    let order_executor = thread::spawn(move || {
        while let Ok((symbol, price)) = receiver.recv() {
            println!("[Executor] Executing order: {} @ {:.2}", symbol, price);
            thread::sleep(std::time::Duration::from_millis(100));
            println!("[Executor] Order {} executed", symbol);
        }
    });

    let order_sender = thread::spawn(move || {
        let orders = [
            ("BTC".to_string(), 42000.0),
            ("ETH".to_string(), 2200.0),
            ("SOL".to_string(), 95.0),
        ];

        for (symbol, price) in orders {
            println!("[Sender] Sending order: {} @ {:.2}", symbol, price);
            // Blocks until receiver gets the message
            sender.send((symbol.clone(), price)).unwrap();
            println!("[Sender] Order {} accepted by executor", symbol);
        }
    });

    order_sender.join().unwrap();
    order_executor.join().unwrap();
}
```

A **rendezvous channel** guarantees that sender and receiver are synchronized — send completes only when the receiver has taken the message.

## Practical Example: Order Rate Limiter

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    // Limit orders in processing to 5
    let (order_tx, order_rx) = sync_channel::<Order>(5);
    let (result_tx, result_rx) = sync_channel::<(u64, String)>(5);

    // Order processor
    let processor = thread::spawn(move || {
        while let Ok(order) = order_rx.recv() {
            println!("[Processor] Processing order #{}", order.id);

            // Simulate order processing
            thread::sleep(Duration::from_millis(200));

            let result = format!(
                "{} {} {} @ {:.2} - FILLED",
                order.side, order.quantity, order.symbol, order.price
            );
            result_tx.send((order.id, result)).unwrap();
        }
    });

    // Result collector thread
    let collector = thread::spawn(move || {
        while let Ok((id, result)) = result_rx.recv() {
            println!("[Result] Order #{}: {}", id, result);
        }
    });

    // Order generator — will slow down due to back-pressure
    let generator = thread::spawn(move || {
        let start = Instant::now();

        for i in 0..15 {
            let order = Order {
                id: i,
                symbol: "BTCUSDT".to_string(),
                side: if i % 2 == 0 { "BUY".to_string() } else { "SELL".to_string() },
                price: 42000.0 + (i as f64 * 10.0),
                quantity: 0.1,
            };

            let send_start = Instant::now();
            println!("[Generator] Sending order #{}...", i);
            order_tx.send(order).unwrap();
            println!(
                "[Generator] Order #{} accepted in {:?} (total: {:?})",
                i, send_start.elapsed(), start.elapsed()
            );
        }

        println!("\n[Generator] All 15 orders sent in {:?}", start.elapsed());
    });

    generator.join().unwrap();
    drop(order_tx);  // Close channel so processor exits
    processor.join().unwrap();
    drop(result_tx);  // Close result channel
    collector.join().unwrap();
}
```

## Example: Order Book with Limited Depth

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderBookUpdate {
    Bid { price: u64, quantity: f64 },
    Ask { price: u64, quantity: f64 },
    Clear,
}

struct OrderBook {
    bids: BTreeMap<u64, f64>,  // price -> quantity
    asks: BTreeMap<u64, f64>,
    max_depth: usize,
}

impl OrderBook {
    fn new(max_depth: usize) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            max_depth,
        }
    }

    fn update(&mut self, update: OrderBookUpdate) {
        match update {
            OrderBookUpdate::Bid { price, quantity } => {
                if quantity > 0.0 {
                    self.bids.insert(price, quantity);
                } else {
                    self.bids.remove(&price);
                }
                // Limit depth
                while self.bids.len() > self.max_depth {
                    if let Some((&lowest, _)) = self.bids.iter().next() {
                        self.bids.remove(&lowest);
                    }
                }
            }
            OrderBookUpdate::Ask { price, quantity } => {
                if quantity > 0.0 {
                    self.asks.insert(price, quantity);
                } else {
                    self.asks.remove(&price);
                }
                while self.asks.len() > self.max_depth {
                    if let Some((&highest, _)) = self.asks.iter().next_back() {
                        self.asks.remove(&highest);
                    }
                }
            }
            OrderBookUpdate::Clear => {
                self.bids.clear();
                self.asks.clear();
            }
        }
    }

    fn best_bid(&self) -> Option<(u64, f64)> {
        self.bids.iter().next_back().map(|(&p, &q)| (p, q))
    }

    fn best_ask(&self) -> Option<(u64, f64)> {
        self.asks.iter().next().map(|(&p, &q)| (p, q))
    }

    fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }
}

fn main() {
    // Limit update queue to 10
    let (update_tx, update_rx) = sync_channel::<OrderBookUpdate>(10);

    // Order book update thread
    let book_handler = thread::spawn(move || {
        let mut book = OrderBook::new(5);  // 5 levels depth

        while let Ok(update) = update_rx.recv() {
            book.update(update);

            if let (Some((bid, bid_qty)), Some((ask, ask_qty))) = (book.best_bid(), book.best_ask()) {
                println!(
                    "BBO: {} x {:.4} | {} x {:.4} | Spread: {}",
                    bid, bid_qty, ask, ask_qty,
                    book.spread().unwrap_or(0)
                );
            }
        }
    });

    // Update source (WebSocket simulation)
    let feed = thread::spawn(move || {
        let updates = [
            OrderBookUpdate::Bid { price: 42000, quantity: 1.5 },
            OrderBookUpdate::Ask { price: 42010, quantity: 2.0 },
            OrderBookUpdate::Bid { price: 41990, quantity: 0.8 },
            OrderBookUpdate::Ask { price: 42020, quantity: 1.2 },
            OrderBookUpdate::Bid { price: 42000, quantity: 2.0 },  // Update
            OrderBookUpdate::Ask { price: 42005, quantity: 0.5 },  // New best ask
            OrderBookUpdate::Bid { price: 42003, quantity: 3.0 },  // New best bid
        ];

        for update in updates {
            update_tx.send(update).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    feed.join().unwrap();
    drop(update_tx);
    book_handler.join().unwrap();
}
```

## try_send — Non-blocking Send

```rust
use std::sync::mpsc::{sync_channel, TrySendError};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct MarketTick {
    symbol: String,
    price: f64,
    timestamp: u64,
}

fn main() {
    // Buffer for 3 ticks — old data is dropped
    let (tx, rx) = sync_channel::<MarketTick>(3);

    // Fast data source
    let producer = thread::spawn(move || {
        let mut timestamp = 0u64;

        for i in 0..20 {
            let tick = MarketTick {
                symbol: "BTCUSDT".to_string(),
                price: 42000.0 + (i as f64 * 5.0),
                timestamp,
            };
            timestamp += 1;

            match tx.try_send(tick) {
                Ok(()) => println!("[Feed] Tick {} sent", i),
                Err(TrySendError::Full(tick)) => {
                    println!("[Feed] Buffer full, tick {} dropped (price: {})", i, tick.price);
                }
                Err(TrySendError::Disconnected(_)) => {
                    println!("[Feed] Channel closed");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    // Slow consumer
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(tick) = rx.recv() {
            println!(
                "[Strategy] Processing tick: {} @ {:.2} (ts: {})",
                tick.symbol, tick.price, tick.timestamp
            );
            count += 1;
            thread::sleep(Duration::from_millis(50));  // Slow processing
        }
        println!("[Strategy] Processed {} ticks", count);
    });

    producer.join().unwrap();
    drop(tx);
    consumer.join().unwrap();
}
```

## Example: Trading Data Processing Pipeline

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct RawTick {
    symbol: String,
    price: f64,
    volume: f64,
}

#[derive(Debug)]
struct NormalizedTick {
    symbol: String,
    price_usd: f64,
    volume_usd: f64,
}

#[derive(Debug)]
struct Signal {
    symbol: String,
    action: String,
    strength: f64,
}

fn main() {
    // Pipeline with bounded buffers
    let (raw_tx, raw_rx) = sync_channel::<RawTick>(5);
    let (normalized_tx, normalized_rx) = sync_channel::<NormalizedTick>(3);
    let (signal_tx, signal_rx) = sync_channel::<Signal>(2);

    // Stage 1: Data normalization
    let normalizer = thread::spawn(move || {
        let btc_usd = 42000.0;  // Exchange rate for conversion

        while let Ok(tick) = raw_rx.recv() {
            let normalized = NormalizedTick {
                symbol: tick.symbol,
                price_usd: tick.price * btc_usd,
                volume_usd: tick.volume * tick.price * btc_usd,
            };
            println!("[Normalizer] {} -> {:.2} USD", normalized.symbol, normalized.price_usd);
            normalized_tx.send(normalized).unwrap();
        }
    });

    // Stage 2: Signal generation
    let signal_generator = thread::spawn(move || {
        let mut last_price = 0.0;

        while let Ok(tick) = normalized_rx.recv() {
            let strength = if last_price > 0.0 {
                (tick.price_usd - last_price) / last_price
            } else {
                0.0
            };

            let action = if strength > 0.001 {
                "BUY"
            } else if strength < -0.001 {
                "SELL"
            } else {
                "HOLD"
            };

            let signal = Signal {
                symbol: tick.symbol,
                action: action.to_string(),
                strength: strength.abs(),
            };

            println!("[SignalGen] {} -> {} ({:.4})", signal.symbol, signal.action, signal.strength);
            signal_tx.send(signal).unwrap();
            last_price = tick.price_usd;
        }
    });

    // Stage 3: Signal execution
    let executor = thread::spawn(move || {
        while let Ok(signal) = signal_rx.recv() {
            if signal.action != "HOLD" && signal.strength > 0.002 {
                println!(
                    "[Executor] EXECUTING: {} {} (strength: {:.4})",
                    signal.action, signal.symbol, signal.strength
                );
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Data source
    let ticks = vec![
        RawTick { symbol: "ETHBTC".to_string(), price: 0.052, volume: 10.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.053, volume: 15.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.051, volume: 20.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.054, volume: 12.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.055, volume: 8.0 },
    ];

    for tick in ticks {
        raw_tx.send(tick).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    drop(raw_tx);
    normalizer.join().unwrap();
    drop(normalized_tx);
    signal_generator.join().unwrap();
    drop(signal_tx);
    executor.join().unwrap();
}
```

## Choosing Buffer Size

```rust
use std::sync::mpsc::sync_channel;

fn main() {
    // Buffer size depends on the scenario:

    // 0 — Rendezvous: strict synchronization between sender and receiver
    // Use for critical operations where order matters
    let (_tx, _rx) = sync_channel::<i32>(0);

    // 1-10 — Small buffer: minimal latency, strict back-pressure
    // Use for real-time trading signals
    let (_tx, _rx) = sync_channel::<i32>(5);

    // 10-100 — Medium buffer: balance between latency and throughput
    // Use for market data processing
    let (_tx, _rx) = sync_channel::<i32>(50);

    // 100+ — Large buffer: high throughput, possible latency
    // Use for logging, analytics
    let (_tx, _rx) = sync_channel::<i32>(1000);

    println!("Buffer size is chosen based on latency and throughput requirements");
}
```

## Comparison: channel vs sync_channel

| Characteristic | `channel()` | `sync_channel(n)` |
|---------------|-------------|-------------------|
| Buffer size | Unbounded | Fixed (n) |
| `send()` blocks | Never | When buffer is full |
| Memory | Grows indefinitely | Limited |
| Back-pressure | No | Yes |
| Use case | Logging, events | Trading data, streams |

## What We Learned

- `sync_channel(n)` creates a channel with buffer size `n`
- `sync_channel(0)` creates a rendezvous channel — sender waits for receiver
- `send()` blocks when the buffer is full
- `try_send()` returns an error instead of blocking
- Back-pressure helps control data flow in trading systems

## Homework

1. Implement an API rate limiting system for exchange requests using `sync_channel(10)` — no more than 10 requests in queue

2. Create a candlestick (OHLCV) processing pipeline: fetching -> indicator calculation -> signal generation, where each stage uses `sync_channel` with different buffer sizes

3. Implement an order routing system: one source, multiple handlers for different exchanges, using `try_send()` to skip overloaded destinations

4. Create a trading bot simulation with rendezvous channel (`sync_channel(0)`) for synchronizing order submission and confirmation

## Navigation

[← Previous day](../158-channel-producer-consumer/en.md) | [Next day →](../160-select-macro-multiplexing/en.md)
