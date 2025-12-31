# Day 195: Broadcast Channel: To All Subscribers

## Trading Analogy

Imagine a trading terminal that streams quotes in real-time. When Bitcoin's price changes, this information must reach **all** connected clients simultaneously: trading bots, risk managers, analytical systems, and mobile apps for traders. This is the classic **broadcast** pattern — one sender, many receivers.

Tokio provides the `broadcast` channel for this purpose — a special channel for broadcasting messages to all subscribers. Unlike a regular `mpsc` channel where each message is received by only one receiver, in a broadcast channel **every** subscriber receives **every** message.

Real-world trading use cases:
- Broadcasting price updates to all trading strategies
- Notifying about stop-loss triggers
- Streaming market news and events
- Synchronizing state between system components

## What is a Broadcast Channel?

`tokio::sync::broadcast` is a multi-producer, multi-consumer channel where **every** sent message is delivered to **all** active receivers.

Key features:
1. **Message cloning** — each subscriber receives their own copy of the message
2. **Bounded capacity** — the channel has a fixed buffer size
3. **Lagging** — slow receivers may miss messages
4. **Dynamic subscription** — new subscribers can join at any time

```rust
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    // Create a broadcast channel with a buffer for 16 messages
    let (tx, mut rx1) = broadcast::channel::<f64>(16);

    // Create a second subscriber
    let mut rx2 = tx.subscribe();

    // Send a price — BOTH subscribers will receive it
    tx.send(42000.0).unwrap();

    // Each receiver gets their own message
    println!("Subscriber 1: {}", rx1.recv().await.unwrap());
    println!("Subscriber 2: {}", rx2.recv().await.unwrap());
}
```

## Simple Example: Quote Broadcasting

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    // Channel for broadcasting price updates
    let (tx, _) = broadcast::channel::<PriceUpdate>(100);

    // Create subscribers for different components
    let mut trading_bot_rx = tx.subscribe();
    let mut risk_manager_rx = tx.subscribe();
    let mut logger_rx = tx.subscribe();

    // Trading bot
    let trading_bot = tokio::spawn(async move {
        while let Ok(update) = trading_bot_rx.recv().await {
            println!("[Bot] Received price {}: ${:.2}",
                update.symbol, update.price);
            // Trading decision logic goes here
        }
    });

    // Risk manager
    let risk_manager = tokio::spawn(async move {
        while let Ok(update) = risk_manager_rx.recv().await {
            println!("[Risk] Checking position for {}: ${:.2}",
                update.symbol, update.price);
            // Risk and limits checking goes here
        }
    });

    // Logger
    let logger = tokio::spawn(async move {
        while let Ok(update) = logger_rx.recv().await {
            println!("[Log] {} @ {} = ${:.2}",
                update.symbol, update.timestamp, update.price);
        }
    });

    // Simulate quote stream
    for i in 0..5 {
        let update = PriceUpdate {
            symbol: "BTC".to_string(),
            price: 42000.0 + (i as f64 * 100.0),
            timestamp: i,
        };

        tx.send(update).unwrap();
        sleep(Duration::from_millis(100)).await;
    }

    // Close the channel (drop the sender)
    drop(tx);

    // Wait for all tasks to complete
    let _ = tokio::join!(trading_bot, risk_manager, logger);
}
```

## Handling Lag

If a subscriber processes messages slower than they arrive, it may miss some messages. This is important to handle in trading systems:

```rust
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
}

#[tokio::main]
async fn main() {
    // Small buffer to demonstrate lagging
    let (tx, mut rx) = broadcast::channel::<MarketData>(4);

    // Slow consumer
    let slow_consumer = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(data) => {
                    println!("Received: {} bid={:.2} ask={:.2}",
                        data.symbol, data.bid, data.ask);
                    // Slow processing
                    sleep(Duration::from_millis(200)).await;
                }
                Err(RecvError::Lagged(skipped)) => {
                    // Missed messages — important to log!
                    println!("WARNING: Skipped {} messages!", skipped);
                    // In a real system, synchronization is needed here
                }
                Err(RecvError::Closed) => {
                    println!("Channel closed");
                    break;
                }
            }
        }
    });

    // Fast sender
    for i in 0..10 {
        let data = MarketData {
            symbol: "ETH".to_string(),
            bid: 2500.0 + i as f64,
            ask: 2501.0 + i as f64,
        };

        match tx.send(data) {
            Ok(receivers) => println!("Sent to {} receivers", receivers),
            Err(_) => println!("No active receivers"),
        }

        sleep(Duration::from_millis(50)).await;
    }

    drop(tx);
    let _ = slow_consumer.await;
}
```

## Practical Example: Trading Signal System

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum TradingSignal {
    Buy { symbol: String, price: f64, reason: String },
    Sell { symbol: String, price: f64, reason: String },
    StopLoss { symbol: String, trigger_price: f64 },
    TakeProfit { symbol: String, trigger_price: f64 },
    MarketAlert { message: String },
}

struct SignalBroadcaster {
    sender: broadcast::Sender<TradingSignal>,
}

impl SignalBroadcaster {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        SignalBroadcaster { sender }
    }

    fn subscribe(&self) -> broadcast::Receiver<TradingSignal> {
        self.sender.subscribe()
    }

    fn broadcast(&self, signal: TradingSignal) -> Result<usize, String> {
        self.sender.send(signal)
            .map_err(|_| "No active subscribers".to_string())
    }
}

// Trade executor
async fn order_executor(
    mut rx: broadcast::Receiver<TradingSignal>,
    name: String,
) {
    println!("[{}] Started", name);

    while let Ok(signal) = rx.recv().await {
        match signal {
            TradingSignal::Buy { symbol, price, reason } => {
                println!("[{}] BUY {} @ ${:.2} ({})",
                    name, symbol, price, reason);
            }
            TradingSignal::Sell { symbol, price, reason } => {
                println!("[{}] SELL {} @ ${:.2} ({})",
                    name, symbol, price, reason);
            }
            TradingSignal::StopLoss { symbol, trigger_price } => {
                println!("[{}] STOP-LOSS set {} @ ${:.2}",
                    name, symbol, trigger_price);
            }
            TradingSignal::TakeProfit { symbol, trigger_price } => {
                println!("[{}] TAKE-PROFIT set {} @ ${:.2}",
                    name, symbol, trigger_price);
            }
            TradingSignal::MarketAlert { message } => {
                println!("[{}] ALERT: {}", name, message);
            }
        }
    }

    println!("[{}] Stopped", name);
}

#[tokio::main]
async fn main() {
    let broadcaster = SignalBroadcaster::new(100);

    // Create multiple executors for different strategies
    let scalper = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Scalper".to_string(),
    ));

    let swing_trader = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Swing Trader".to_string(),
    ));

    let risk_monitor = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Risk Monitor".to_string(),
    ));

    // Small delay for receivers to start
    sleep(Duration::from_millis(50)).await;

    // Send a series of signals
    let signals = vec![
        TradingSignal::MarketAlert {
            message: "High market volatility!".to_string()
        },
        TradingSignal::Buy {
            symbol: "BTC".to_string(),
            price: 42000.0,
            reason: "Resistance breakout".to_string()
        },
        TradingSignal::StopLoss {
            symbol: "BTC".to_string(),
            trigger_price: 41000.0
        },
        TradingSignal::TakeProfit {
            symbol: "BTC".to_string(),
            trigger_price: 45000.0
        },
        TradingSignal::Sell {
            symbol: "ETH".to_string(),
            price: 2500.0,
            reason: "Take-profit reached".to_string()
        },
    ];

    for signal in signals {
        match broadcaster.broadcast(signal) {
            Ok(count) => println!("--- Signal sent to {} subscribers ---", count),
            Err(e) => println!("Error: {}", e),
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Close the channel
    drop(broadcaster);

    // Wait for all tasks to complete
    let _ = tokio::join!(scalper, swing_trader, risk_monitor);
}
```

## Broadcast vs MPSC: When to Use Which

| Feature | broadcast | mpsc |
|---------|-----------|------|
| Receivers | Many (each gets everything) | Many (each gets a portion) |
| Cloning | Message is cloned | Message is moved |
| Message loss | Possible (Lagged) | Not possible |
| Use case | Quotes, events, notifications | Task queue, pipeline |

## Advanced Example: Multi-Currency Terminal

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, interval};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Quote {
    symbol: String,
    bid: f64,
    ask: f64,
    volume: f64,
    timestamp: u64,
}

struct QuoteFeed {
    sender: broadcast::Sender<Quote>,
}

impl QuoteFeed {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        QuoteFeed { sender }
    }

    fn subscribe(&self) -> broadcast::Receiver<Quote> {
        self.sender.subscribe()
    }

    fn publish(&self, quote: Quote) {
        // Ignore error if no subscribers
        let _ = self.sender.send(quote);
    }
}

// Quote generator (exchange simulation)
async fn quote_generator(feed: Arc<QuoteFeed>) {
    let symbols = vec![
        ("BTC", 42000.0),
        ("ETH", 2500.0),
        ("SOL", 100.0),
    ];

    let mut tick = 0u64;
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        for (symbol, base_price) in &symbols {
            // Simulate random price movement
            let variation = (tick as f64 * 0.1).sin() * base_price * 0.01;
            let bid = base_price + variation;
            let ask = bid + base_price * 0.001; // 0.1% spread

            let quote = Quote {
                symbol: symbol.to_string(),
                bid,
                ask,
                volume: 100.0 + (tick as f64 % 50.0),
                timestamp: tick,
            };

            feed.publish(quote);
        }

        tick += 1;

        if tick >= 20 {
            break;
        }
    }
}

// Spread monitor
async fn spread_monitor(mut rx: broadcast::Receiver<Quote>) {
    let mut max_spreads: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Ok(quote) = rx.recv().await {
        let spread_pct = (quote.ask - quote.bid) / quote.bid * 100.0;

        let max_spread = max_spreads
            .entry(quote.symbol.clone())
            .or_insert(0.0);

        if spread_pct > *max_spread {
            *max_spread = spread_pct;
            println!("[Spread] {} max spread: {:.4}%",
                quote.symbol, spread_pct);
        }
    }

    println!("\n=== Final Maximum Spreads ===");
    for (symbol, spread) in max_spreads {
        println!("{}: {:.4}%", symbol, spread);
    }
}

// Volume detector
async fn volume_detector(mut rx: broadcast::Receiver<Quote>) {
    while let Ok(quote) = rx.recv().await {
        if quote.volume > 120.0 {
            println!("[Volume] {} high volume: {:.0}",
                quote.symbol, quote.volume);
        }
    }
}

// File logger (simulation)
async fn file_logger(mut rx: broadcast::Receiver<Quote>) {
    let mut count = 0;

    while let Ok(quote) = rx.recv().await {
        count += 1;
        // In reality, this would write to a file
        if count % 10 == 0 {
            println!("[Logger] Recorded {} quotes", count);
        }
    }

    println!("[Logger] Total recorded: {} quotes", count);
}

#[tokio::main]
async fn main() {
    let feed = Arc::new(QuoteFeed::new(256));

    // Start subscribers
    let spread_task = tokio::spawn(spread_monitor(feed.subscribe()));
    let volume_task = tokio::spawn(volume_detector(feed.subscribe()));
    let logger_task = tokio::spawn(file_logger(feed.subscribe()));

    // Start generator
    quote_generator(feed).await;

    // Wait for subscribers to finish
    let _ = tokio::join!(spread_task, volume_task, logger_task);
}
```

## Error Handling and Patterns

### Checking Subscriber Count

```rust
use tokio::sync::broadcast;

fn main() {
    let (tx, rx1) = broadcast::channel::<i32>(16);
    let rx2 = tx.subscribe();

    // Number of active receivers
    println!("Active receivers: {}", tx.receiver_count());

    // Remove one
    drop(rx1);
    println!("After drop: {}", tx.receiver_count());

    // Send returns number of receivers
    let sent_to = tx.send(42).unwrap();
    println!("Sent to {} receivers", sent_to);

    drop(rx2);

    // Error when sending with no subscribers
    match tx.send(43) {
        Ok(n) => println!("Sent to: {}", n),
        Err(_) => println!("No active subscribers!"),
    }
}
```

### Dynamic Subscriber Connection

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct Event {
    id: u64,
    data: String,
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Event>(100);
    let tx_clone = tx.clone();

    // Event sender
    let sender = tokio::spawn(async move {
        for i in 0..10 {
            let event = Event {
                id: i,
                data: format!("Event {}", i),
            };

            match tx.send(event) {
                Ok(n) => println!("Event {} sent to {} subscribers", i, n),
                Err(_) => println!("No subscribers for event {}", i),
            }

            sleep(Duration::from_millis(100)).await;
        }
    });

    // Subscriber joins later
    sleep(Duration::from_millis(350)).await;

    let mut late_subscriber = tx_clone.subscribe();
    println!("--- Late subscriber joined ---");

    let receiver = tokio::spawn(async move {
        while let Ok(event) = late_subscriber.recv().await {
            println!("Late subscriber received: {:?}", event);
        }
    });

    let _ = sender.await;
    drop(tx_clone);
    let _ = receiver.await;
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `broadcast::channel` | Channel for broadcasting messages to all subscribers |
| `tx.subscribe()` | Create a new subscriber |
| `tx.send()` | Send message to all |
| `rx.recv().await` | Asynchronously receive a message |
| `RecvError::Lagged` | Subscriber missed messages |
| `receiver_count()` | Number of active subscribers |
| Buffer | Fixed size, important for performance |

## Homework

1. **Quote System**: Create a quote broadcasting system with:
   - Price generator for 5 cryptocurrencies
   - Subscriber that calculates average price over the last 10 ticks
   - Subscriber that determines trend (rising/falling)
   - Subscriber that logs anomalous price jumps (>1%)

2. **Trading Alerts**: Implement an alert system for:
   - Reaching specified price levels
   - Volume exceeding threshold
   - Spread changes above threshold
   All alerts should be received by multiple components (Telegram bot, email, logger).

3. **Lag Handling**: Create a stress test:
   - Fast sender (1000 messages/sec)
   - Slow receiver with `Lagged` handling
   - Count of missed messages
   - Recovery mechanism (requesting missed data)

4. **Multi-Channel Subscription**: Implement a system with multiple channels:
   - BTC quotes channel
   - ETH quotes channel
   - Trading signals channel
   Create a subscriber that listens to all channels simultaneously using `tokio::select!`.

## Navigation

[← Previous day](../194-watch-channel-latest-value/en.md) | [Next day →](../196-oneshot-channel-single-response/en.md)
