# Day 237: Redis Pub/Sub — Real-Time Notifications

## Trading Analogy

Imagine a trading floor at an exchange: when an important event occurs — a sharp price change, a large trade, or a stop-loss trigger — all interested participants need to be notified instantly. Traders subscribe to specific instruments and receive notifications only about events that interest them.

Redis Pub/Sub works on the same principle:
- **Publisher** — a system that publishes events (e.g., BTC price changes)
- **Subscriber** — a client subscribed to specific channels receiving notifications
- **Channel** — a named message stream (e.g., `prices:BTC`, `orders:filled`, `alerts:risk`)

In real trading, this is used for:
- Instant price change notifications
- Order execution alerts
- Trading strategy signals
- Risk management alerts

## What is Redis Pub/Sub?

Redis Pub/Sub is a messaging mechanism based on the publish-subscribe pattern:

1. **Subscribers** register to one or more channels
2. **Publishers** send messages to channels
3. Redis **delivers** messages to all active subscribers of the channel
4. Messages are **not persisted** — if a subscriber is offline, they won't receive the message

```
┌─────────────┐     ┌───────────┐     ┌──────────────┐
│  Publisher  │────▶│   Redis   │────▶│  Subscriber  │
│  (Prices)   │     │  Channel  │     │  (Trader 1)  │
└─────────────┘     │ prices:BTC│     └──────────────┘
                    │           │     ┌──────────────┐
                    │           │────▶│  Subscriber  │
                    │           │     │  (Trader 2)  │
                    └───────────┘     └──────────────┘
```

## Project Setup

```toml
# Cargo.toml
[package]
name = "trading-notifications"
version = "0.1.0"
edition = "2021"

[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "aio"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## Basic Example: Subscribing to a Channel

```rust
use redis::{Client, Commands, PubSubCommands};
use std::thread;
use std::time::Duration;

fn main() -> redis::RedisResult<()> {
    // Create two clients: one for publishing, one for subscribing
    let publisher_client = Client::open("redis://127.0.0.1/")?;
    let subscriber_client = Client::open("redis://127.0.0.1/")?;

    // Subscriber thread
    let subscriber_handle = thread::spawn(move || {
        let mut con = subscriber_client.get_connection().unwrap();

        // Subscribe to BTC price channel
        con.subscribe(&["prices:BTC"], |msg| {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            println!("[Subscriber] Channel: {}, Message: {}", channel, payload);

            // Return ControlFlow to manage subscription
            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Give subscriber time to connect
    thread::sleep(Duration::from_millis(100));

    // Publisher thread
    let publisher_handle = thread::spawn(move || {
        let mut con = publisher_client.get_connection().unwrap();

        // Publish several price updates
        for i in 0..5 {
            let price = 42000.0 + (i as f64 * 100.0);
            let message = format!("BTC: ${:.2}", price);

            let subscribers: i32 = con.publish("prices:BTC", &message).unwrap();
            println!("[Publisher] Sent: {} ({} subscribers)", message, subscribers);

            thread::sleep(Duration::from_millis(500));
        }
    });

    publisher_handle.join().unwrap();
    // Note: subscriber_handle will run indefinitely

    Ok(())
}
```

## Price Notifications in a Trading System

```rust
use redis::{Client, Commands, PubSubCommands};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: i64,
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeAlert {
    alert_type: String,
    symbol: String,
    message: String,
    severity: String,
    timestamp: i64,
}

fn main() -> redis::RedisResult<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Price updates subscriber
    let price_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        println!("[Price Subscriber] Connecting to channels...");

        con.subscribe(&["prices:BTC", "prices:ETH", "prices:SOL"], |msg| {
            let payload: String = msg.get_payload().unwrap();

            if let Ok(update) = serde_json::from_str::<PriceUpdate>(&payload) {
                println!(
                    "[Price] {} = ${:.2} (volume: {:.4}, source: {})",
                    update.symbol, update.price, update.volume, update.source
                );

                // Check for sharp price changes
                if update.price > 50000.0 {
                    println!("  ⚠️  {} price above $50,000!", update.symbol);
                }
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Trading alerts subscriber
    let alert_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        println!("[Alert Subscriber] Connecting to alerts channel...");

        con.subscribe(&["alerts:trading"], |msg| {
            let payload: String = msg.get_payload().unwrap();

            if let Ok(alert) = serde_json::from_str::<TradeAlert>(&payload) {
                let icon = match alert.severity.as_str() {
                    "critical" => "🔴",
                    "warning" => "🟡",
                    "info" => "🔵",
                    _ => "⚪",
                };

                println!(
                    "[Alert] {} {} [{}]: {}",
                    icon, alert.alert_type, alert.symbol, alert.message
                );
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Price publisher (simulating market data feed)
    let publisher = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        let symbols = vec![
            ("BTC", 42000.0, "prices:BTC"),
            ("ETH", 2800.0, "prices:ETH"),
            ("SOL", 98.0, "prices:SOL"),
        ];

        for i in 0..10 {
            for (symbol, base_price, channel) in &symbols {
                let price_change = (i as f64 * 50.0) * if i % 2 == 0 { 1.0 } else { -1.0 };
                let price = base_price + price_change;

                let update = PriceUpdate {
                    symbol: symbol.to_string(),
                    price,
                    volume: 1.5 + (i as f64 * 0.1),
                    timestamp: chrono::Utc::now().timestamp(),
                    source: "Binance".to_string(),
                };

                let json = serde_json::to_string(&update).unwrap();
                let _: i32 = con.publish(*channel, &json).unwrap();

                // Generate alert under certain conditions
                if price > 42500.0 && *symbol == "BTC" {
                    let alert = TradeAlert {
                        alert_type: "PRICE_SPIKE".to_string(),
                        symbol: symbol.to_string(),
                        message: format!("Price exceeded $42,500 (current: ${:.2})", price),
                        severity: "warning".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                    };

                    let alert_json = serde_json::to_string(&alert).unwrap();
                    let _: i32 = con.publish("alerts:trading", &alert_json).unwrap();
                }
            }

            thread::sleep(Duration::from_millis(1000));
        }

        running_clone.store(false, Ordering::SeqCst);
    });

    publisher.join().unwrap();

    Ok(())
}
```

## Async Pub/Sub with Tokio

```rust
use redis::aio::PubSub;
use redis::{AsyncCommands, Client};
use tokio::sync::mpsc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OrderNotification {
    order_id: u64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
    filled_qty: f64,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RiskAlert {
    portfolio_id: String,
    metric: String,
    current_value: f64,
    threshold: f64,
    message: String,
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1/")?;

    // Channel for passing notifications to main handler
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Order subscriber task
    let order_subscriber = {
        let client = client.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("orders:filled").await.unwrap();
            pubsub.subscribe("orders:cancelled").await.unwrap();
            pubsub.subscribe("orders:rejected").await.unwrap();

            println!("[Orders] Subscribed to order channels");

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let channel: String = msg.get_channel_name().to_string();
                let payload: String = msg.get_payload().unwrap();

                if let Ok(order) = serde_json::from_str::<OrderNotification>(&payload) {
                    let notification = format!(
                        "[{}] Order #{}: {} {} {} @ ${:.2} (filled: {:.4})",
                        order.status.to_uppercase(),
                        order.order_id,
                        order.side.to_uppercase(),
                        order.quantity,
                        order.symbol,
                        order.price,
                        order.filled_qty
                    );
                    tx.send(notification).await.unwrap();
                }
            }
        })
    };

    // Risk alert subscriber task
    let risk_subscriber = {
        let client = client.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("risk:alerts").await.unwrap();

            println!("[Risk] Subscribed to risk alerts channel");

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap();

                if let Ok(alert) = serde_json::from_str::<RiskAlert>(&payload) {
                    let notification = format!(
                        "[RISK] Portfolio {}: {} = {:.2}% (threshold: {:.2}%) - {}",
                        alert.portfolio_id,
                        alert.metric,
                        alert.current_value,
                        alert.threshold,
                        alert.message
                    );
                    tx.send(notification).await.unwrap();
                }
            }
        })
    };

    // Event publisher (simulating trading engine)
    let publisher = {
        let client = client.clone();

        tokio::spawn(async move {
            let mut con = client.get_multiplexed_async_connection().await.unwrap();

            // Simulate order executions
            for i in 1..=5 {
                let order = OrderNotification {
                    order_id: 1000 + i,
                    symbol: "BTC/USDT".to_string(),
                    side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                    quantity: 0.1 * i as f64,
                    price: 42000.0 + (i as f64 * 100.0),
                    status: "filled".to_string(),
                    filled_qty: 0.1 * i as f64,
                    timestamp: chrono::Utc::now().timestamp(),
                };

                let json = serde_json::to_string(&order).unwrap();
                let _: i32 = con.publish("orders:filled", &json).await.unwrap();

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // Simulate risk alert
            let risk_alert = RiskAlert {
                portfolio_id: "PORT-001".to_string(),
                metric: "drawdown".to_string(),
                current_value: 12.5,
                threshold: 10.0,
                message: "Drawdown exceeded acceptable level!".to_string(),
            };

            let json = serde_json::to_string(&risk_alert).unwrap();
            let _: i32 = con.publish("risk:alerts", &json).await.unwrap();
        })
    };

    // Notification handler
    let handler = tokio::spawn(async move {
        while let Some(notification) = rx.recv().await {
            println!("{}", notification);
        }
    });

    // Wait for publisher to complete
    publisher.await.unwrap();

    // Allow time to receive all messages
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
```

## Pattern Subscribe

```rust
use redis::{Client, PubSubCommands};
use std::thread;
use std::time::Duration;

fn main() -> redis::RedisResult<()> {
    let subscriber_client = Client::open("redis://127.0.0.1/")?;
    let publisher_client = Client::open("redis://127.0.0.1/")?;

    // Pattern subscriber — receives all price messages
    let pattern_subscriber = thread::spawn(move || {
        let mut con = subscriber_client.get_connection().unwrap();

        println!("[Pattern] Subscribing to prices:* ...");

        // psubscribe allows using patterns
        con.psubscribe(&["prices:*", "orders:*"], |msg| {
            let pattern: String = msg.get_pattern().unwrap_or_default().to_string();
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            println!(
                "[Pattern: {}] Channel: {} -> {}",
                pattern, channel, payload
            );

            redis::ControlFlow::Continue
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(100));

    // Publisher sends to different channels
    let publisher = thread::spawn(move || {
        let mut con = publisher_client.get_connection().unwrap();

        let channels = vec![
            "prices:BTC",
            "prices:ETH",
            "prices:SOL",
            "orders:filled",
            "orders:cancelled",
        ];

        for (i, channel) in channels.iter().enumerate() {
            let message = format!("Message #{} for {}", i + 1, channel);
            let _: i32 = con.publish(*channel, &message).unwrap();
            println!("[Publisher] {} -> {}", channel, message);
            thread::sleep(Duration::from_millis(200));
        }
    });

    publisher.join().unwrap();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
```

## Trading Strategy Notification System

```rust
use redis::{Client, Commands, PubSubCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StrategySignal {
    strategy_id: String,
    signal_type: String, // "entry", "exit", "adjust"
    symbol: String,
    direction: String,   // "long", "short"
    confidence: f64,
    price_target: Option<f64>,
    stop_loss: Option<f64>,
    timestamp: i64,
}

#[derive(Debug, Clone)]
struct SignalAggregator {
    signals: Arc<Mutex<HashMap<String, Vec<StrategySignal>>>>,
}

impl SignalAggregator {
    fn new() -> Self {
        SignalAggregator {
            signals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_signal(&self, signal: StrategySignal) {
        let mut signals = self.signals.lock().unwrap();
        signals
            .entry(signal.symbol.clone())
            .or_insert_with(Vec::new)
            .push(signal);
    }

    fn get_consensus(&self, symbol: &str) -> Option<String> {
        let signals = self.signals.lock().unwrap();

        if let Some(symbol_signals) = signals.get(symbol) {
            if symbol_signals.is_empty() {
                return None;
            }

            let long_confidence: f64 = symbol_signals
                .iter()
                .filter(|s| s.direction == "long")
                .map(|s| s.confidence)
                .sum();

            let short_confidence: f64 = symbol_signals
                .iter()
                .filter(|s| s.direction == "short")
                .map(|s| s.confidence)
                .sum();

            if long_confidence > short_confidence && long_confidence > 0.5 {
                Some(format!("LONG (confidence: {:.1}%)", long_confidence * 100.0))
            } else if short_confidence > long_confidence && short_confidence > 0.5 {
                Some(format!("SHORT (confidence: {:.1}%)", short_confidence * 100.0))
            } else {
                Some("NEUTRAL".to_string())
            }
        } else {
            None
        }
    }
}

fn main() -> redis::RedisResult<()> {
    let aggregator = SignalAggregator::new();
    let aggregator_clone = aggregator.clone();

    // Strategy signals subscriber
    let signal_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        con.psubscribe(&["strategy:*:signals"], |msg| {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            if let Ok(signal) = serde_json::from_str::<StrategySignal>(&payload) {
                println!(
                    "[Signal] {} from {}: {} {} (confidence: {:.0}%)",
                    signal.signal_type.to_uppercase(),
                    signal.strategy_id,
                    signal.direction.to_uppercase(),
                    signal.symbol,
                    signal.confidence * 100.0
                );

                if let Some(target) = signal.price_target {
                    println!("  Target: ${:.2}", target);
                }
                if let Some(sl) = signal.stop_loss {
                    println!("  Stop-loss: ${:.2}", sl);
                }

                aggregator_clone.add_signal(signal);
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(100));

    // Publishing signals from different strategies
    let publisher = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        let strategies = vec![
            ("momentum", "long", 0.75),
            ("mean_reversion", "short", 0.60),
            ("breakout", "long", 0.85),
            ("ml_predictor", "long", 0.70),
        ];

        for (strategy, direction, confidence) in strategies {
            let signal = StrategySignal {
                strategy_id: strategy.to_string(),
                signal_type: "entry".to_string(),
                symbol: "BTC/USDT".to_string(),
                direction: direction.to_string(),
                confidence,
                price_target: Some(45000.0),
                stop_loss: Some(40000.0),
                timestamp: chrono::Utc::now().timestamp(),
            };

            let channel = format!("strategy:{}:signals", strategy);
            let json = serde_json::to_string(&signal).unwrap();
            let _: i32 = con.publish(&channel, &json).unwrap();

            thread::sleep(Duration::from_millis(300));
        }
    });

    publisher.join().unwrap();
    thread::sleep(Duration::from_secs(1));

    // Show consensus
    if let Some(consensus) = aggregator.get_consensus("BTC/USDT") {
        println!("\n[Consensus] BTC/USDT: {}", consensus);
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Pub/Sub | Publish-subscribe messaging pattern |
| Publisher | Client that publishes messages to channels |
| Subscriber | Client that receives messages from channels |
| Channel | Named message stream |
| `subscribe` | Subscribe to specific channels |
| `psubscribe` | Subscribe using patterns (wildcards) |
| Fire-and-forget | Messages are not persisted — only delivered to online subscribers |

## Exercises

1. **Price Monitor**: Create a system that subscribes to price updates for multiple cryptocurrencies and displays a notification when the price changes by more than 1% in the last 5 minutes.

2. **Order Notifications**: Implement a notification system that:
   - Subscribes to `orders:pending`, `orders:filled`, `orders:cancelled` channels
   - Maintains statistics for each event type
   - Sends an alert if cancelled orders exceed 10% of total orders

3. **Signal Router**: Write a program that:
   - Receives signals from multiple trading strategies via Pub/Sub
   - Aggregates signals by instrument
   - Publishes consolidated signals to a separate channel

4. **Channel Multiplexer**: Create an async system using Tokio that:
   - Subscribes to multiple channels simultaneously
   - Processes messages from different channels in separate tasks
   - Implements graceful shutdown upon receiving a termination signal

## Homework

Implement a complete notification system for a trading bot:

1. **Market Data Publisher**: Publishes price updates every 100ms
2. **Order Publisher**: Publishes order statuses when they change
3. **Aggregator Subscriber**: Collects data and calculates metrics
4. **Alert Subscriber**: Sends notifications when conditions are met:
   - Sharp price change (> 2% per minute)
   - Large order execution (> 1 BTC)
   - Portfolio drawdown (> 5%)

Add connection error handling with automatic reconnection.

## Navigation

[← Previous day](../236-redis-sorted-sets-leaderboard/en.md) | [Next day →](../238-redis-streams-event-sourcing/en.md)
