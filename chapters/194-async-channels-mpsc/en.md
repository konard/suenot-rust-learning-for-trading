# Day 194: Async Channels: tokio::sync::mpsc

## Trading Analogy

Imagine a trading floor on an exchange. Multiple traders (producers) are simultaneously shouting their orders, while one clerk (consumer) records them in the order book. Traders don't wait for the clerk to process each order — they continue trading. The clerk processes orders as they arrive.

This is what an **mpsc** (multi-producer, single-consumer) channel is — multiple senders can asynchronously send messages to a single receiver. In the context of `tokio::sync::mpsc`, this happens in an asynchronous environment where neither senders nor receivers block the execution of other tasks.

In a real trading system, this could be:
- Multiple WebSocket connections sending price updates to a single handler
- Several strategies generating signals for a single order executor
- Different data sources forwarding events to a unified logger

## What is tokio::sync::mpsc?

`tokio::sync::mpsc` is an asynchronous channel for passing messages between tasks in tokio. Unlike `std::sync::mpsc`:

| Characteristic | std::sync::mpsc | tokio::sync::mpsc |
|----------------|-----------------|-------------------|
| Environment | Synchronous | Asynchronous |
| Blocking | Blocks thread | Suspends task |
| Usage | OS threads | async/await tasks |
| Buffer | Unbounded | Bounded or unbounded |

### Creating a Channel

```rust
use tokio::sync::mpsc;

// Bounded channel with buffer for 100 messages
let (tx, rx) = mpsc::channel::<TradeSignal>(100);

// Unbounded channel (use with caution!)
let (tx, rx) = mpsc::unbounded_channel::<TradeSignal>();
```

## Simple Example: Trade Signal Stream

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct TradeSignal {
    symbol: String,
    action: String,    // "BUY" or "SELL"
    price: f64,
    quantity: f64,
}

#[tokio::main]
async fn main() {
    // Create a channel with buffer for 32 signals
    let (tx, mut rx) = mpsc::channel::<TradeSignal>(32);

    // Signal generator (producer)
    let signal_generator = tokio::spawn(async move {
        let signals = vec![
            TradeSignal {
                symbol: "BTC/USDT".to_string(),
                action: "BUY".to_string(),
                price: 42000.0,
                quantity: 0.5,
            },
            TradeSignal {
                symbol: "ETH/USDT".to_string(),
                action: "SELL".to_string(),
                price: 2800.0,
                quantity: 2.0,
            },
            TradeSignal {
                symbol: "BTC/USDT".to_string(),
                action: "SELL".to_string(),
                price: 42500.0,
                quantity: 0.5,
            },
        ];

        for signal in signals {
            println!("Generating signal: {:?}", signal);
            tx.send(signal).await.expect("Receiver closed");
            sleep(Duration::from_millis(500)).await;
        }

        println!("All signals sent");
    });

    // Order executor (consumer)
    let order_executor = tokio::spawn(async move {
        while let Some(signal) = rx.recv().await {
            println!("Executing: {} {} {} @ {}",
                signal.action, signal.quantity, signal.symbol, signal.price);

            // Simulate order execution
            sleep(Duration::from_millis(100)).await;
        }

        println!("Channel closed, executor finished");
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(signal_generator, order_executor);
}
```

## Multiple Senders: Monitoring Multiple Exchanges

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn monitor_exchange(
    exchange: &str,
    symbol: &str,
    base_price: f64,
    tx: mpsc::Sender<PriceUpdate>,
) {
    for i in 0..5 {
        let spread = 0.001; // 0.1% spread
        let price_change = (i as f64 * 10.0) - 20.0;
        let mid_price = base_price + price_change;

        let update = PriceUpdate {
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            bid: mid_price * (1.0 - spread / 2.0),
            ask: mid_price * (1.0 + spread / 2.0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        if tx.send(update).await.is_err() {
            println!("{}: Receiver closed, exiting", exchange);
            break;
        }

        // Different exchanges update at different rates
        sleep(Duration::from_millis(100 + (exchange.len() * 50) as u64)).await;
    }

    println!("{}: Monitoring finished", exchange);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<PriceUpdate>(100);

    // Start monitoring multiple exchanges in parallel
    let binance_tx = tx.clone();
    let binance = tokio::spawn(async move {
        monitor_exchange("Binance", "BTC/USDT", 42000.0, binance_tx).await;
    });

    let coinbase_tx = tx.clone();
    let coinbase = tokio::spawn(async move {
        monitor_exchange("Coinbase", "BTC/USD", 42050.0, coinbase_tx).await;
    });

    let kraken_tx = tx.clone();
    let kraken = tokio::spawn(async move {
        monitor_exchange("Kraken", "XBT/USD", 41980.0, kraken_tx).await;
    });

    // Important: drop the original sender so the channel closes
    // when all clones are destroyed
    drop(tx);

    // Price aggregator
    let aggregator = tokio::spawn(async move {
        let mut best_bid: Option<(String, f64)> = None;
        let mut best_ask: Option<(String, f64)> = None;

        while let Some(update) = rx.recv().await {
            println!("[{}] {} Bid: {:.2} Ask: {:.2}",
                update.exchange, update.symbol, update.bid, update.ask);

            // Update best prices
            if best_bid.is_none() || update.bid > best_bid.as_ref().unwrap().1 {
                best_bid = Some((update.exchange.clone(), update.bid));
            }
            if best_ask.is_none() || update.ask < best_ask.as_ref().unwrap().1 {
                best_ask = Some((update.exchange.clone(), update.ask));
            }
        }

        if let (Some((bid_ex, bid)), Some((ask_ex, ask))) = (&best_bid, &best_ask) {
            println!("\n=== Best Prices ===");
            println!("Best BID: {:.2} on {}", bid, bid_ex);
            println!("Best ASK: {:.2} on {}", ask, ask_ex);
            println!("Arbitrage spread: {:.2}%", (ask - bid) / bid * 100.0);
        }
    });

    let _ = tokio::join!(binance, coinbase, kraken, aggregator);
}
```

## Bounded vs Unbounded Channel

### Bounded Channel

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
}

#[tokio::main]
async fn main() {
    // Buffer for 3 orders — protection against overload
    let (tx, mut rx) = mpsc::channel::<Order>(3);

    let producer = tokio::spawn(async move {
        for i in 1..=10 {
            let order = Order {
                id: i,
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            };

            println!("Trying to send order #{}", i);

            // send() will wait if buffer is full
            match tx.send(order).await {
                Ok(_) => println!("Order #{} sent", i),
                Err(e) => {
                    println!("Send error: {}", e);
                    break;
                }
            }
        }
    });

    let consumer = tokio::spawn(async move {
        // Simulate slow processing
        while let Some(order) = rx.recv().await {
            println!("Processing order #{}: {} @ {}",
                order.id, order.symbol, order.price);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(producer, consumer);
}
```

### Unbounded Channel

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
struct MarketEvent {
    event_type: String,
    data: String,
}

#[tokio::main]
async fn main() {
    // Unbounded channel — doesn't block sender
    // Use with caution: can lead to memory exhaustion!
    let (tx, mut rx) = mpsc::unbounded_channel::<MarketEvent>();

    let producer = tokio::spawn(async move {
        for i in 1..=1000 {
            let event = MarketEvent {
                event_type: "TRADE".to_string(),
                data: format!("Trade #{}", i),
            };

            // unbounded_send doesn't require await
            if tx.send(event).is_err() {
                println!("Receiver closed");
                break;
            }
        }
        println!("Sent 1000 events");
    });

    let consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(event) = rx.recv().await {
            count += 1;
            if count % 100 == 0 {
                println!("Processed {} events", count);
            }
        }
        println!("Total processed: {}", count);
    });

    let _ = tokio::join!(producer, consumer);
}
```

## Practical Example: Order Execution System

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
}

#[derive(Debug)]
enum ExecutionResult {
    Filled { order_id: u64, fill_price: f64 },
    Rejected { order_id: u64, reason: String },
    PartialFill { order_id: u64, filled_qty: f64, remaining_qty: f64 },
}

struct OrderExecutor {
    current_prices: HashMap<String, f64>,
}

impl OrderExecutor {
    fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert("BTC/USDT".to_string(), 42000.0);
        prices.insert("ETH/USDT".to_string(), 2800.0);
        OrderExecutor { current_prices: prices }
    }

    async fn execute(&self, order: Order) -> ExecutionResult {
        // Simulate execution delay
        sleep(Duration::from_millis(50)).await;

        let current_price = match self.current_prices.get(&order.symbol) {
            Some(price) => *price,
            None => return ExecutionResult::Rejected {
                order_id: order.id,
                reason: format!("Unknown symbol: {}", order.symbol),
            },
        };

        match order.order_type {
            OrderType::Market => {
                ExecutionResult::Filled {
                    order_id: order.id,
                    fill_price: current_price,
                }
            }
            OrderType::Limit { price } => {
                let can_fill = match order.side {
                    OrderSide::Buy => current_price <= price,
                    OrderSide::Sell => current_price >= price,
                };

                if can_fill {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        fill_price: price,
                    }
                } else {
                    ExecutionResult::Rejected {
                        order_id: order.id,
                        reason: format!(
                            "Price {} did not reach limit {}",
                            current_price, price
                        ),
                    }
                }
            }
            OrderType::StopLoss { trigger_price } => {
                let triggered = match order.side {
                    OrderSide::Sell => current_price <= trigger_price,
                    OrderSide::Buy => current_price >= trigger_price,
                };

                if triggered {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        fill_price: current_price,
                    }
                } else {
                    ExecutionResult::Rejected {
                        order_id: order.id,
                        reason: "Stop not triggered".to_string(),
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (order_tx, mut order_rx) = mpsc::channel::<Order>(100);
    let (result_tx, mut result_rx) = mpsc::channel::<ExecutionResult>(100);

    // Strategy 1: Aggressive trader
    let tx1 = order_tx.clone();
    let strategy1 = tokio::spawn(async move {
        for i in 1..=3 {
            let order = Order {
                id: i,
                symbol: "BTC/USDT".to_string(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: 0.1,
            };
            println!("Strategy 1: Sending market order #{}", i);
            tx1.send(order).await.ok();
            sleep(Duration::from_millis(100)).await;
        }
    });

    // Strategy 2: Limit trader
    let tx2 = order_tx.clone();
    let strategy2 = tokio::spawn(async move {
        for i in 4..=6 {
            let order = Order {
                id: i,
                symbol: "ETH/USDT".to_string(),
                side: OrderSide::Sell,
                order_type: OrderType::Limit { price: 2850.0 },
                quantity: 1.0,
            };
            println!("Strategy 2: Sending limit order #{}", i);
            tx2.send(order).await.ok();
            sleep(Duration::from_millis(150)).await;
        }
    });

    drop(order_tx);

    // Order executor
    let result_tx_clone = result_tx.clone();
    let executor = tokio::spawn(async move {
        let executor = OrderExecutor::new();

        while let Some(order) = order_rx.recv().await {
            println!("Executor: Processing order #{}", order.id);
            let result = executor.execute(order).await;
            result_tx_clone.send(result).await.ok();
        }
        println!("Executor: All orders processed");
    });

    drop(result_tx);

    // Result handler
    let result_handler = tokio::spawn(async move {
        let mut filled = 0;
        let mut rejected = 0;

        while let Some(result) = result_rx.recv().await {
            match result {
                ExecutionResult::Filled { order_id, fill_price } => {
                    println!("Result: Order #{} filled at {}",
                        order_id, fill_price);
                    filled += 1;
                }
                ExecutionResult::Rejected { order_id, reason } => {
                    println!("Result: Order #{} rejected: {}",
                        order_id, reason);
                    rejected += 1;
                }
                ExecutionResult::PartialFill { order_id, filled_qty, remaining_qty } => {
                    println!("Result: Order #{} partially filled: {} / {}",
                        order_id, filled_qty, filled_qty + remaining_qty);
                }
            }
        }

        println!("\n=== Summary ===");
        println!("Filled: {}", filled);
        println!("Rejected: {}", rejected);
    });

    let _ = tokio::join!(strategy1, strategy2, executor, result_handler);
}
```

## try_send and try_recv: Non-blocking Operations

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct QuickSignal {
    symbol: String,
    urgency: u8,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<QuickSignal>(3);

    // Fast sender — doesn't want to wait
    let sender = tokio::spawn(async move {
        for i in 1..=10 {
            let signal = QuickSignal {
                symbol: format!("SYM{}", i),
                urgency: (i % 3) as u8,
            };

            // try_send doesn't wait, returns error if channel is full
            match tx.try_send(signal) {
                Ok(_) => println!("Signal {} sent instantly", i),
                Err(mpsc::error::TrySendError::Full(s)) => {
                    println!("Channel full! Signal {} ({}) dropped", i, s.symbol);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    println!("Channel closed");
                    break;
                }
            }

            sleep(Duration::from_millis(50)).await;
        }
    });

    // Slow receiver
    let receiver = tokio::spawn(async move {
        loop {
            // try_recv doesn't wait, returns error if channel is empty
            match rx.try_recv() {
                Ok(signal) => {
                    println!("Received signal: {} (urgency: {})",
                        signal.symbol, signal.urgency);
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    println!("Channel empty, doing other work...");
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    println!("All senders closed");
                    break;
                }
            }

            sleep(Duration::from_millis(200)).await;
        }
    });

    let _ = tokio::join!(sender, receiver);
}
```

## Graceful Shutdown with Channels

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Command {
    Process(String),
    Shutdown,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(10);

    // Worker process
    let worker = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(Command::Process(data)) => {
                    println!("Processing: {}", data);
                    sleep(Duration::from_millis(100)).await;
                }
                Some(Command::Shutdown) => {
                    println!("Shutdown command received");

                    // Process remaining messages
                    while let Ok(cmd) = rx.try_recv() {
                        if let Command::Process(data) = cmd {
                            println!("Processing remaining: {}", data);
                        }
                    }

                    println!("Worker finished gracefully");
                    break;
                }
                None => {
                    println!("Channel closed");
                    break;
                }
            }
        }
    });

    // Main process
    for i in 1..=5 {
        tx.send(Command::Process(format!("Task {}", i))).await.ok();
    }

    println!("Sending shutdown command...");
    tx.send(Command::Shutdown).await.ok();

    worker.await.ok();
    println!("Program finished");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| mpsc channel | Multiple senders, single receiver |
| Bounded channel | Limited buffer, sender waits when full |
| Unbounded channel | Unlimited buffer, can exhaust memory |
| tx.clone() | Create additional sender |
| send().await | Async send with waiting |
| try_send() | Non-blocking send |
| recv().await | Async receive with waiting |
| try_recv() | Non-blocking receive |
| drop(tx) | Close channel (termination signal) |

## Homework

1. **Price Aggregator**: Create a system where multiple "exchanges" (async tasks) send price updates to one aggregator. The aggregator should output the best bid/ask price every second.

2. **Order Queue with Priorities**: Modify the order execution system so that market orders are processed before limit orders. Use two channels or a priority field.

3. **Rate Limiter**: Create a component that receives requests through a channel and allows no more than N requests per second. Others should either be queued or rejected.

4. **Channel Monitoring**: Add metrics to the channel:
   - Number of messages sent
   - Number of messages received
   - Current queue size (you can use `capacity()` and `len()` for some channels)
   - Number of dropped messages (when using `try_send`)

## Navigation

[← Previous Day](../193-async-mutex-tokio/en.md) | [Next Day →](../195-broadcast-channel/en.md)
