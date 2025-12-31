# Day 177: Pattern: Producer-Consumer

## Trading Analogy

Imagine how a real exchange works: one department continuously receives market data — quotes, trades, order book changes (this is the **Producer**). Another department analyzes this data and makes trading decisions (this is the **Consumer**). Between them is a queue — a buffer where data is placed and from which it's taken for processing.

Why this matters in trading:
- **Market data arrives faster** than we can process it — we need a buffer
- **Different speeds**: receiving data — microseconds, analysis — milliseconds
- **Component decoupling**: data source doesn't depend on processing speed

The Producer-Consumer pattern solves a classic multithreading problem: how to safely transfer data between threads working at different speeds.

## What is Producer-Consumer?

**Producer-Consumer** is a concurrent programming pattern where:
- **Producer** — creates data and places it in a shared buffer
- **Consumer** — takes data from the buffer and processes it
- **Buffer** (queue) — synchronized structure between them

```
┌──────────┐     ┌─────────────┐     ┌──────────┐
│ Producer │ --> │   Buffer    │ --> │ Consumer │
│  (data)  │     │   (queue)   │     │(process) │
└──────────┘     └─────────────┘     └──────────┘
```

### Key Characteristics

| Characteristic | Description |
|----------------|-------------|
| Decoupling | Producer and Consumer don't depend on each other directly |
| Buffering | Smooths out load spikes |
| Scalability | Can add multiple producers and consumers |
| Thread Safety | Buffer is synchronized for safe access |

## Implementation with Channels (mpsc)

In Rust, the most natural way to implement Producer-Consumer is using `std::sync::mpsc` channels:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    // Create a channel (unbounded buffer)
    let (tx, rx) = mpsc::channel();

    // Producer: generates market data
    let producer = thread::spawn(move || {
        let ticks = vec![
            MarketTick { symbol: "BTC".to_string(), price: 42000.0, volume: 1.5, timestamp: 1 },
            MarketTick { symbol: "ETH".to_string(), price: 2200.0, volume: 10.0, timestamp: 2 },
            MarketTick { symbol: "BTC".to_string(), price: 42050.0, volume: 0.8, timestamp: 3 },
            MarketTick { symbol: "ETH".to_string(), price: 2205.0, volume: 5.0, timestamp: 4 },
            MarketTick { symbol: "BTC".to_string(), price: 42100.0, volume: 2.0, timestamp: 5 },
        ];

        for tick in ticks {
            println!("[Producer] Sending: {} @ {}", tick.symbol, tick.price);
            tx.send(tick).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        println!("[Producer] Finished");
    });

    // Consumer: processes market data
    let consumer = thread::spawn(move || {
        let mut total_btc_volume = 0.0;
        let mut total_eth_volume = 0.0;

        // recv() blocks until data arrives or channel closes
        while let Ok(tick) = rx.recv() {
            println!("[Consumer] Received: {} @ {} (volume: {})",
                tick.symbol, tick.price, tick.volume);

            match tick.symbol.as_str() {
                "BTC" => total_btc_volume += tick.volume,
                "ETH" => total_eth_volume += tick.volume,
                _ => {}
            }

            // Simulate processing
            thread::sleep(Duration::from_millis(150));
        }

        println!("[Consumer] Total BTC: {}, ETH: {}", total_btc_volume, total_eth_volume);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Bounded Buffer with sync_channel

To prevent memory overflow, use `sync_channel` with a bounded buffer:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct TradeSignal {
    symbol: String,
    action: String,    // "BUY" or "SELL"
    price: f64,
    quantity: f64,
}

fn main() {
    // Buffer for 3 elements — producer blocks if buffer is full
    let (tx, rx) = mpsc::sync_channel::<TradeSignal>(3);

    let producer = thread::spawn(move || {
        let signals = vec![
            TradeSignal { symbol: "BTC".into(), action: "BUY".into(), price: 42000.0, quantity: 0.5 },
            TradeSignal { symbol: "ETH".into(), action: "SELL".into(), price: 2200.0, quantity: 5.0 },
            TradeSignal { symbol: "BTC".into(), action: "SELL".into(), price: 42100.0, quantity: 0.3 },
            TradeSignal { symbol: "SOL".into(), action: "BUY".into(), price: 100.0, quantity: 10.0 },
            TradeSignal { symbol: "ETH".into(), action: "BUY".into(), price: 2180.0, quantity: 8.0 },
            TradeSignal { symbol: "BTC".into(), action: "BUY".into(), price: 41900.0, quantity: 1.0 },
        ];

        for signal in signals {
            let start = Instant::now();
            println!("[Producer] Sending signal: {} {} @ {}",
                signal.action, signal.symbol, signal.price);

            // send() blocks if buffer is full
            tx.send(signal).unwrap();

            let elapsed = start.elapsed();
            if elapsed > Duration::from_millis(10) {
                println!("[Producer] Waited {} ms (buffer was full)", elapsed.as_millis());
            }
        }
        println!("[Producer] All signals sent");
    });

    let consumer = thread::spawn(move || {
        let mut executed = 0;

        while let Ok(signal) = rx.recv() {
            println!("[Consumer] Executing: {} {} {} @ {}",
                signal.action, signal.quantity, signal.symbol, signal.price);

            // Slow processing — simulates sending order to exchange
            thread::sleep(Duration::from_millis(500));
            executed += 1;
        }

        println!("[Consumer] Orders executed: {}", executed);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Multiple Producers, Single Consumer

The `mpsc` (multiple producer, single consumer) pattern is ideal for collecting data from multiple sources:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Producer 1: Binance
    let tx1 = tx.clone();
    let binance = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Binance".to_string(),
                symbol: "BTC/USDT".to_string(),
                bid: 42000.0 + i as f64 * 10.0,
                ask: 42001.0 + i as f64 * 10.0,
            };
            tx1.send(update).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Producer 2: Coinbase
    let tx2 = tx.clone();
    let coinbase = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Coinbase".to_string(),
                symbol: "BTC/USD".to_string(),
                bid: 42005.0 + i as f64 * 8.0,
                ask: 42008.0 + i as f64 * 8.0,
            };
            tx2.send(update).unwrap();
            thread::sleep(Duration::from_millis(120));
        }
    });

    // Producer 3: Kraken
    let tx3 = tx;
    let kraken = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Kraken".to_string(),
                symbol: "XBTUSD".to_string(),
                bid: 41998.0 + i as f64 * 12.0,
                ask: 42002.0 + i as f64 * 12.0,
            };
            tx3.send(update).unwrap();
            thread::sleep(Duration::from_millis(80));
        }
    });

    // Consumer: aggregates prices and looks for arbitrage
    let aggregator = thread::spawn(move || {
        let mut best_bid = 0.0;
        let mut best_ask = f64::MAX;
        let mut bid_exchange = String::new();
        let mut ask_exchange = String::new();

        while let Ok(update) = rx.recv() {
            println!("[{}] {} bid: {:.2}, ask: {:.2}",
                update.exchange, update.symbol, update.bid, update.ask);

            if update.bid > best_bid {
                best_bid = update.bid;
                bid_exchange = update.exchange.clone();
            }
            if update.ask < best_ask {
                best_ask = update.ask;
                ask_exchange = update.exchange.clone();
            }

            // Check for arbitrage opportunity
            if best_bid > best_ask {
                println!(">>> ARBITRAGE! Buy on {} @ {:.2}, sell on {} @ {:.2}",
                    ask_exchange, best_ask, bid_exchange, best_bid);
            }
        }

        println!("\nBest bid: {} on {}", best_bid, bid_exchange);
        println!("Best ask: {} on {}", best_ask, ask_exchange);
    });

    binance.join().unwrap();
    coinbase.join().unwrap();
    kraken.join().unwrap();
    aggregator.join().unwrap();
}
```

## Practical Example: Trading Pipeline

Let's implement a complete data processing pipeline:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

// Stage 1: Raw market data
#[derive(Debug, Clone)]
struct RawTick {
    symbol: String,
    price: f64,
    volume: f64,
}

// Stage 2: Enriched data with indicators
#[derive(Debug)]
struct EnrichedTick {
    symbol: String,
    price: f64,
    volume: f64,
    sma_5: f64,        // 5-tick simple moving average
    price_change: f64,
}

// Stage 3: Trading signal
#[derive(Debug)]
struct Signal {
    symbol: String,
    action: String,
    confidence: f64,
    reason: String,
}

fn main() {
    // Channel 1: raw data -> enrichment
    let (raw_tx, raw_rx) = mpsc::channel::<RawTick>();

    // Channel 2: enriched data -> signal generation
    let (enriched_tx, enriched_rx) = mpsc::channel::<EnrichedTick>();

    // Channel 3: signals -> execution
    let (signal_tx, signal_rx) = mpsc::channel::<Signal>();

    // Producer: market data source
    let data_source = thread::spawn(move || {
        let ticks = vec![
            RawTick { symbol: "BTC".into(), price: 42000.0, volume: 1.0 },
            RawTick { symbol: "BTC".into(), price: 42050.0, volume: 1.5 },
            RawTick { symbol: "BTC".into(), price: 42100.0, volume: 2.0 },
            RawTick { symbol: "BTC".into(), price: 42080.0, volume: 0.8 },
            RawTick { symbol: "BTC".into(), price: 42150.0, volume: 3.0 },
            RawTick { symbol: "BTC".into(), price: 42200.0, volume: 2.5 },
            RawTick { symbol: "BTC".into(), price: 42180.0, volume: 1.2 },
        ];

        for tick in ticks {
            println!("[DataSource] New tick: {} @ {}", tick.symbol, tick.price);
            raw_tx.send(tick).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Consumer/Producer: data enrichment
    let enricher = thread::spawn(move || {
        let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();
        let mut last_price: HashMap<String, f64> = HashMap::new();

        while let Ok(tick) = raw_rx.recv() {
            let history = price_history.entry(tick.symbol.clone()).or_insert_with(Vec::new);
            history.push(tick.price);

            // Calculate SMA for last 5 ticks
            let sma_5 = if history.len() >= 5 {
                history.iter().rev().take(5).sum::<f64>() / 5.0
            } else {
                history.iter().sum::<f64>() / history.len() as f64
            };

            // Calculate price change
            let prev = last_price.get(&tick.symbol).copied().unwrap_or(tick.price);
            let price_change = (tick.price - prev) / prev * 100.0;
            last_price.insert(tick.symbol.clone(), tick.price);

            let enriched = EnrichedTick {
                symbol: tick.symbol,
                price: tick.price,
                volume: tick.volume,
                sma_5,
                price_change,
            };

            println!("[Enricher] SMA: {:.2}, Change: {:.3}%",
                enriched.sma_5, enriched.price_change);
            enriched_tx.send(enriched).unwrap();
        }
    });

    // Consumer/Producer: signal generation
    let signal_generator = thread::spawn(move || {
        while let Ok(tick) = enriched_rx.recv() {
            // Simple strategy: price above SMA and rising = buy
            let signal = if tick.price > tick.sma_5 && tick.price_change > 0.1 {
                Some(Signal {
                    symbol: tick.symbol,
                    action: "BUY".into(),
                    confidence: tick.price_change.abs().min(1.0),
                    reason: format!("Price above SMA, up {:.2}%", tick.price_change),
                })
            } else if tick.price < tick.sma_5 && tick.price_change < -0.1 {
                Some(Signal {
                    symbol: tick.symbol,
                    action: "SELL".into(),
                    confidence: tick.price_change.abs().min(1.0),
                    reason: format!("Price below SMA, down {:.2}%", tick.price_change),
                })
            } else {
                None
            };

            if let Some(sig) = signal {
                println!("[SignalGen] Signal: {} {} (confidence: {:.2})",
                    sig.action, sig.symbol, sig.confidence);
                signal_tx.send(sig).unwrap();
            }
        }
    });

    // Consumer: order execution
    let executor = thread::spawn(move || {
        let mut orders = 0;

        while let Ok(signal) = signal_rx.recv() {
            if signal.confidence > 0.15 {
                println!("[Executor] === EXECUTING: {} {} ===", signal.action, signal.symbol);
                println!("           Reason: {}", signal.reason);
                orders += 1;
            } else {
                println!("[Executor] Skipping signal (low confidence)");
            }
        }

        println!("\n[Executor] Total orders executed: {}", orders);
    });

    data_source.join().unwrap();
    enricher.join().unwrap();
    signal_generator.join().unwrap();
    executor.join().unwrap();
}
```

## Error Handling and Graceful Shutdown

In real systems, proper shutdown handling is important:

```rust
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct OrderUpdate {
    order_id: u64,
    status: String,
    filled_qty: f64,
}

fn main() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();

    let shutdown_producer = Arc::clone(&shutdown);
    let producer = thread::spawn(move || {
        let mut order_id = 1;

        while !shutdown_producer.load(Ordering::Relaxed) {
            let update = OrderUpdate {
                order_id,
                status: if order_id % 3 == 0 { "FILLED".into() } else { "PARTIAL".into() },
                filled_qty: (order_id as f64) * 0.1,
            };

            match tx.send(update) {
                Ok(_) => println!("[Producer] Sent order update #{}", order_id),
                Err(e) => {
                    println!("[Producer] Channel closed: {}", e);
                    break;
                }
            }

            order_id += 1;
            thread::sleep(Duration::from_millis(200));
        }

        println!("[Producer] Received shutdown signal");
    });

    let shutdown_consumer = Arc::clone(&shutdown);
    let consumer = thread::spawn(move || {
        let mut processed = 0;

        loop {
            // Use recv_timeout to check shutdown
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(update) => {
                    println!("[Consumer] Order #{}: {} (filled: {:.2})",
                        update.order_id, update.status, update.filled_qty);
                    processed += 1;
                }
                Err(RecvTimeoutError::Timeout) => {
                    // Check shutdown flag
                    if shutdown_consumer.load(Ordering::Relaxed) {
                        println!("[Consumer] Shutting down via flag");
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    println!("[Consumer] Channel closed");
                    break;
                }
            }
        }

        println!("[Consumer] Updates processed: {}", processed);
    });

    // Let it run for 2 seconds
    thread::sleep(Duration::from_secs(2));

    // Send shutdown signal
    println!("\n[Main] Sending shutdown signal...\n");
    shutdown.store(true, Ordering::Relaxed);

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("Program terminated gracefully");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Producer-Consumer | Pattern for separating data production and consumption |
| `mpsc::channel` | Unbounded channel, multiple producers -> single consumer |
| `mpsc::sync_channel` | Bounded buffer, blocks producer when full |
| Pipeline | Chain of producer-consumer stages for step-by-step processing |
| Graceful shutdown | Proper termination using flags and timeouts |

## Homework

1. **Quote Aggregator**: Implement a system with 3 producers (exchanges) and 1 consumer. The consumer should:
   - Store the latest prices from each exchange
   - Display the best bid/ask in real-time
   - Detect arbitrage opportunities (bid on one exchange > ask on another)

2. **Risk Manager**: Create a pipeline with three stages:
   - Producer: generates trading signals
   - Processor 1: filters signals by risk (rejects oversized positions)
   - Processor 2: calculates position size using Kelly criterion
   - Consumer: executes orders and tracks P&L

3. **Load Balancer**: Implement a system with 1 producer and 3 consumers:
   - Producer sends orders to a queue
   - Each consumer processes orders for its own group of symbols
   - Use separate channels for distribution by symbol

4. **Queue Monitoring**: Add monitoring to any example:
   - Queue element counter
   - Average processing time
   - Warning when queue accumulates > N elements

## Navigation

[← Previous day](../176-operation-cancellation/en.md) | [Next day →](../178-fan-out-fan-in/en.md)
