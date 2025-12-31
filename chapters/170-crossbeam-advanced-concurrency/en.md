# Day 170: crossbeam: Advanced Concurrency

## Trading Analogy

Imagine you're building a high-frequency trading platform that needs to:
- Collect price feeds from multiple exchanges simultaneously
- Process thousands of orders per second
- Coordinate multiple trading strategies running in parallel
- Share market data between analysis threads without copying

The standard library's concurrency tools (`std::sync`, `mpsc` channels) work well, but for high-performance trading systems, you need something faster and more flexible. This is where **crossbeam** comes in — a collection of tools for concurrent programming that are faster, more ergonomic, and more powerful than the standard library equivalents.

Think of it like upgrading from a regular trading terminal to a professional low-latency trading system — same concepts, but optimized for performance.

## What is crossbeam?

`crossbeam` is a crate (actually, a family of crates) that provides:

| Component | Description | Use Case in Trading |
|-----------|-------------|---------------------|
| `crossbeam-channel` | Fast multi-producer, multi-consumer channels | Order queues, price feeds |
| `crossbeam-utils` | Scoped threads, caching utilities | Parallel data analysis |
| `crossbeam-epoch` | Epoch-based memory reclamation | Lock-free order books |
| `crossbeam-deque` | Work-stealing deques | Load balancing strategies |
| `crossbeam-queue` | Lock-free queues | High-throughput message passing |

## Setting Up crossbeam

Add to your `Cargo.toml`:

```toml
[dependencies]
crossbeam = "0.8"
# Or individual crates:
# crossbeam-channel = "0.5"
# crossbeam-utils = "0.8"
```

## crossbeam vs Standard Library

### Standard Library Channels

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // mpsc = multi-producer, single-consumer
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        tx1.send("Price from Binance").unwrap();
    });

    thread::spawn(move || {
        tx.send("Price from Coinbase").unwrap();
    });

    // Only ONE receiver possible
    println!("Received: {}", rx.recv().unwrap());
    println!("Received: {}", rx.recv().unwrap());
}
```

### crossbeam Channels: Multi-Producer, Multi-Consumer

```rust
use crossbeam::channel;
use std::thread;

fn main() {
    // crossbeam = multi-producer, multi-consumer!
    let (tx, rx) = channel::unbounded();

    // Multiple senders
    let tx1 = tx.clone();
    let tx2 = tx.clone();

    // Multiple receivers!
    let rx1 = rx.clone();
    let rx2 = rx.clone();

    thread::spawn(move || {
        tx1.send("Price from Binance: 42000").unwrap();
    });

    thread::spawn(move || {
        tx2.send("Price from Coinbase: 42005").unwrap();
    });

    // Different consumers can receive from same channel
    let consumer1 = thread::spawn(move || {
        if let Ok(msg) = rx1.recv() {
            println!("Consumer 1 got: {}", msg);
        }
    });

    let consumer2 = thread::spawn(move || {
        if let Ok(msg) = rx2.recv() {
            println!("Consumer 2 got: {}", msg);
        }
    });

    consumer1.join().unwrap();
    consumer2.join().unwrap();
}
```

## Bounded vs Unbounded Channels

```rust
use crossbeam::channel;
use std::thread;
use std::time::Duration;

fn main() {
    // Unbounded: unlimited capacity (careful with memory!)
    let (tx_unbounded, rx_unbounded) = channel::unbounded::<f64>();

    // Bounded: limited capacity (back-pressure)
    let (tx_bounded, rx_bounded) = channel::bounded::<f64>(100);

    // Producer: order processor
    let tx = tx_bounded.clone();
    thread::spawn(move || {
        for i in 0..1000 {
            let price = 42000.0 + i as f64;
            // Will block if channel is full — natural back-pressure!
            tx.send(price).unwrap();
            println!("Sent price: {}", price);
        }
    });

    // Slow consumer
    thread::spawn(move || {
        while let Ok(price) = rx_bounded.recv() {
            println!("Processing price: {}", price);
            thread::sleep(Duration::from_millis(10)); // Slow processing
        }
    });

    thread::sleep(Duration::from_secs(2));
}
```

## select! Macro: Waiting on Multiple Channels

One of crossbeam's most powerful features is `select!` — it lets you wait on multiple channels simultaneously:

```rust
use crossbeam::channel::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    price: f64,
}

#[derive(Debug)]
struct OrderFill {
    order_id: u64,
    filled_price: f64,
    quantity: f64,
}

fn main() {
    let (price_tx, price_rx): (Sender<PriceUpdate>, Receiver<PriceUpdate>) = channel::unbounded();
    let (order_tx, order_rx): (Sender<OrderFill>, Receiver<OrderFill>) = channel::unbounded();
    let (shutdown_tx, shutdown_rx) = channel::bounded::<()>(1);

    // Price feed simulator
    let price_sender = price_tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            thread::sleep(Duration::from_millis(100));
            price_sender.send(PriceUpdate {
                exchange: "Binance".to_string(),
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            }).unwrap();
        }
    });

    // Order fill simulator
    let order_sender = order_tx.clone();
    thread::spawn(move || {
        for i in 0..3 {
            thread::sleep(Duration::from_millis(150));
            order_sender.send(OrderFill {
                order_id: i + 1,
                filled_price: 41995.0 + i as f64 * 5.0,
                quantity: 0.1,
            }).unwrap();
        }
    });

    // Shutdown after 1 second
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        let _ = shutdown_tx.send(());
    });

    // Main event loop using select!
    loop {
        crossbeam::channel::select! {
            recv(price_rx) -> msg => {
                match msg {
                    Ok(update) => println!("Price: {} {} = ${}",
                        update.exchange, update.symbol, update.price),
                    Err(_) => println!("Price channel closed"),
                }
            }
            recv(order_rx) -> msg => {
                match msg {
                    Ok(fill) => println!("Order #{} filled: {} @ ${}",
                        fill.order_id, fill.quantity, fill.filled_price),
                    Err(_) => println!("Order channel closed"),
                }
            }
            recv(shutdown_rx) -> _ => {
                println!("Shutdown signal received!");
                break;
            }
        }
    }

    println!("Trading system stopped.");
}
```

## Scoped Threads: Borrowing Data Without Arc

Standard threads require `'static` lifetime — you must `move` data or use `Arc`. Crossbeam's scoped threads let you borrow data directly:

### Standard Threads (Requires Arc)

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let prices = Arc::new(vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0]);

    let prices1 = Arc::clone(&prices);
    let prices2 = Arc::clone(&prices);

    let h1 = thread::spawn(move || {
        let sum: f64 = prices1.iter().sum();
        sum / prices1.len() as f64
    });

    let h2 = thread::spawn(move || {
        prices2.iter().cloned().fold(f64::MIN, f64::max)
    });

    println!("Average: {}", h1.join().unwrap());
    println!("Max: {}", h2.join().unwrap());
}
```

### Scoped Threads (Direct Borrowing)

```rust
use crossbeam::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0];
    let volumes = vec![10.5, 20.0, 15.3, 8.7, 25.1];

    // Scoped threads can borrow local data!
    thread::scope(|s| {
        // Thread 1: Calculate average price
        let avg_handle = s.spawn(|_| {
            let sum: f64 = prices.iter().sum();
            sum / prices.len() as f64
        });

        // Thread 2: Calculate total volume
        let vol_handle = s.spawn(|_| {
            volumes.iter().sum::<f64>()
        });

        // Thread 3: Calculate VWAP (needs both!)
        let vwap_handle = s.spawn(|_| {
            let total_value: f64 = prices.iter()
                .zip(volumes.iter())
                .map(|(p, v)| p * v)
                .sum();
            let total_volume: f64 = volumes.iter().sum();
            total_value / total_volume
        });

        // Scoped threads are automatically joined at the end of scope
        println!("Average Price: ${:.2}", avg_handle.join().unwrap());
        println!("Total Volume: {:.2}", vol_handle.join().unwrap());
        println!("VWAP: ${:.2}", vwap_handle.join().unwrap());
    }).unwrap();

    // prices and volumes are still usable here!
    println!("Original data still available: {} prices", prices.len());
}
```

## Lock-Free Queue: ArrayQueue

For high-performance scenarios where you can't afford mutex overhead:

```rust
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    // Fixed-size lock-free queue
    let tick_queue = Arc::new(ArrayQueue::new(1000));

    // Producer: Market data feed
    let producer_queue = Arc::clone(&tick_queue);
    let producer = thread::spawn(move || {
        for i in 0..100 {
            let tick = MarketTick {
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + (i as f64 * 0.5),
                volume: 0.1 + (i as f64 * 0.01),
                timestamp: 1000000 + i,
            };

            match producer_queue.push(tick) {
                Ok(_) => {}
                Err(tick) => println!("Queue full, dropped: {:?}", tick),
            }
        }
        println!("Producer done: sent 100 ticks");
    });

    // Consumer: Strategy processor
    let consumer_queue = Arc::clone(&tick_queue);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        let mut total_volume = 0.0;

        loop {
            match consumer_queue.pop() {
                Some(tick) => {
                    count += 1;
                    total_volume += tick.volume;
                }
                None => {
                    if count >= 100 {
                        break;
                    }
                    thread::yield_now(); // Let producer run
                }
            }
        }

        println!("Consumer done: processed {} ticks, total volume: {:.2}",
            count, total_volume);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Work-Stealing Deque: Load Balancing

For parallel processing where some tasks take longer than others:

```rust
use crossbeam::deque::{Injector, Stealer, Worker};
use std::thread;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct AnalysisTask {
    symbol: String,
    data_points: usize,
}

fn main() {
    // Global queue for new tasks
    let injector: Arc<Injector<AnalysisTask>> = Arc::new(Injector::new());

    // Create workers
    let worker1 = Worker::new_fifo();
    let worker2 = Worker::new_fifo();

    // Stealers allow workers to steal from each other
    let stealer1 = worker1.stealer();
    let stealer2 = worker2.stealer();

    // Add tasks to global queue
    for i in 0..20 {
        injector.push(AnalysisTask {
            symbol: format!("ASSET_{}", i),
            data_points: 100 + i * 50, // Varying complexity
        });
    }

    let inj1 = Arc::clone(&injector);
    let inj2 = Arc::clone(&injector);

    // Worker 1
    let handle1 = thread::spawn(move || {
        let mut processed = 0;
        loop {
            // Try local queue first
            let task = worker1.pop()
                // Then try global queue
                .or_else(|| inj1.steal().success())
                // Then try stealing from worker 2
                .or_else(|| stealer2.steal().success());

            match task {
                Some(task) => {
                    // Simulate analysis
                    thread::sleep(std::time::Duration::from_micros(task.data_points as u64));
                    println!("Worker 1 analyzed {} ({} points)", task.symbol, task.data_points);
                    processed += 1;
                }
                None => {
                    if processed >= 10 {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }
        println!("Worker 1 completed {} tasks", processed);
    });

    // Worker 2
    let handle2 = thread::spawn(move || {
        let mut processed = 0;
        loop {
            let task = worker2.pop()
                .or_else(|| inj2.steal().success())
                .or_else(|| stealer1.steal().success());

            match task {
                Some(task) => {
                    thread::sleep(std::time::Duration::from_micros(task.data_points as u64));
                    println!("Worker 2 analyzed {} ({} points)", task.symbol, task.data_points);
                    processed += 1;
                }
                None => {
                    if processed >= 10 {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }
        println!("Worker 2 completed {} tasks", processed);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Practical Example: Multi-Exchange Price Aggregator

```rust
use crossbeam::channel::{self, Sender, Receiver};
use crossbeam::thread;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: Instant,
}

#[derive(Debug)]
struct AggregatedPrice {
    symbol: String,
    best_bid: f64,
    best_bid_exchange: String,
    best_ask: f64,
    best_ask_exchange: String,
    spread: f64,
}

fn simulate_exchange(name: &str, tx: Sender<PriceQuote>, base_price: f64) {
    for i in 0..10 {
        let spread = 5.0 + (i as f64 * 0.5);
        let quote = PriceQuote {
            exchange: name.to_string(),
            symbol: "BTC/USD".to_string(),
            bid: base_price - spread + (i as f64 * 2.0),
            ask: base_price + spread + (i as f64 * 2.0),
            timestamp: Instant::now(),
        };
        tx.send(quote).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn aggregate_prices(quotes: &[PriceQuote]) -> Option<AggregatedPrice> {
    if quotes.is_empty() {
        return None;
    }

    let mut best_bid = f64::MIN;
    let mut best_bid_exchange = String::new();
    let mut best_ask = f64::MAX;
    let mut best_ask_exchange = String::new();

    for quote in quotes {
        if quote.bid > best_bid {
            best_bid = quote.bid;
            best_bid_exchange = quote.exchange.clone();
        }
        if quote.ask < best_ask {
            best_ask = quote.ask;
            best_ask_exchange = quote.exchange.clone();
        }
    }

    Some(AggregatedPrice {
        symbol: quotes[0].symbol.clone(),
        best_bid,
        best_bid_exchange,
        best_ask,
        best_ask_exchange,
        spread: best_ask - best_bid,
    })
}

fn main() {
    let (quote_tx, quote_rx): (Sender<PriceQuote>, Receiver<PriceQuote>) =
        channel::unbounded();
    let (result_tx, result_rx): (Sender<AggregatedPrice>, Receiver<AggregatedPrice>) =
        channel::unbounded();

    thread::scope(|s| {
        // Exchange simulators
        let tx1 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Binance", tx1, 42000.0);
        });

        let tx2 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Coinbase", tx2, 42010.0);
        });

        let tx3 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Kraken", tx3, 41995.0);
        });

        drop(quote_tx); // Close sender so receiver knows when done

        // Aggregator thread
        let agg_rx = quote_rx.clone();
        let agg_tx = result_tx.clone();
        s.spawn(move |_| {
            let mut quotes_by_exchange: HashMap<String, PriceQuote> = HashMap::new();

            while let Ok(quote) = agg_rx.recv() {
                quotes_by_exchange.insert(quote.exchange.clone(), quote);

                // Aggregate when we have quotes from all exchanges
                if quotes_by_exchange.len() >= 3 {
                    let quotes: Vec<_> = quotes_by_exchange.values().cloned().collect();
                    if let Some(aggregated) = aggregate_prices(&quotes) {
                        agg_tx.send(aggregated).unwrap();
                    }
                }
            }
        });

        drop(result_tx);

        // Result processor
        s.spawn(move |_| {
            println!("\n=== Aggregated Price Feed ===\n");
            while let Ok(agg) = result_rx.recv() {
                println!("{}: Best Bid ${:.2} ({}), Best Ask ${:.2} ({}), Spread: ${:.2}",
                    agg.symbol,
                    agg.best_bid, agg.best_bid_exchange,
                    agg.best_ask, agg.best_ask_exchange,
                    agg.spread
                );

                // Arbitrage opportunity?
                if agg.spread < 0.0 {
                    println!("  >>> ARBITRAGE OPPORTUNITY! <<<");
                }
            }
            println!("\n=== Feed Closed ===");
        });
    }).unwrap();
}
```

## Performance Comparison

| Feature | std::sync::mpsc | crossbeam-channel |
|---------|-----------------|-------------------|
| Producers | Multiple | Multiple |
| Consumers | Single | Multiple |
| select! | No (use recv_timeout) | Yes |
| Performance | Good | Better (2-10x faster) |
| Bounded channels | Yes (sync_channel) | Yes |
| Zero-capacity | No | Yes (rendezvous) |

## When to Use crossbeam

| Scenario | Use crossbeam |
|----------|---------------|
| Multiple consumers needed | Yes (MPMC channels) |
| High-throughput messaging | Yes (faster channels) |
| Borrowing data in threads | Yes (scoped threads) |
| Lock-free data structures | Yes (queues, deques) |
| Work-stealing parallelism | Yes (Deque) |
| Simple single-consumer queue | Maybe (std::mpsc is fine) |

## What We Learned

| Concept | Description |
|---------|-------------|
| crossbeam | High-performance concurrency toolkit |
| MPMC channels | Multiple producers AND multiple consumers |
| Bounded channels | Back-pressure with capacity limits |
| select! macro | Wait on multiple channels at once |
| Scoped threads | Borrow data without Arc/move |
| ArrayQueue | Lock-free fixed-size queue |
| Work-stealing | Load balancing with deques |

## Homework

1. **Price Feed Merger**: Create a system with 5 exchange simulators sending prices to a central aggregator. Use `select!` to handle all feeds and calculate the best bid/ask across exchanges.

2. **Order Router**: Implement an order routing system where:
   - Orders come in through a bounded channel
   - Multiple worker threads process orders
   - Each worker can steal work from others when idle
   - Track total orders processed per worker

3. **Scoped Analysis**: Using scoped threads, parallelize the calculation of:
   - Simple Moving Average (SMA)
   - Exponential Moving Average (EMA)
   - Relative Strength Index (RSI)
   All operating on the same price data without using Arc.

4. **Timeout Handler**: Create a trading system that uses `select!` with timeout:
   - If no price update in 500ms, log a warning
   - If no update in 2 seconds, trigger reconnection
   - Gracefully shutdown on CTRL+C signal

## Navigation

[← Previous day](../164-deadlock-threads-block/en.md) | [Next day →](../171-crossbeam-channels/en.md)
