# Day 188: tokio::select!: First to Respond

## Trading Analogy

Imagine you're a trader who simultaneously sends price requests to multiple exchanges: Binance, Bybit, and OKX. You don't need all three responses — getting the first available price is enough to make a trading decision. As soon as one exchange responds, you act without waiting for the others.

This is exactly how `tokio::select!` works — it runs multiple async operations in parallel and returns the result of **the first one to complete**. The remaining operations are cancelled.

This is critical in algorithmic trading:
- Get the first price from multiple sources
- Execute an order with timeout — or cancel it
- React to the first of several market events
- Manage multiple WebSocket connections simultaneously

## What is tokio::select!?

`tokio::select!` is a macro that allows you to await multiple async operations simultaneously and react to the first one that completes.

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = sleep(Duration::from_secs(1)) => {
            println!("1 second passed");
        }
        _ = sleep(Duration::from_secs(2)) => {
            println!("2 seconds passed");
        }
    }
    // Prints: "1 second passed"
    // The second sleep will be cancelled
}
```

## Basic Syntax

```rust
tokio::select! {
    result1 = async_operation1 => {
        // Handle result1
    }
    result2 = async_operation2 => {
        // Handle result2
    }
    // Can add more branches
}
```

## Fetching Prices from Multiple Exchanges

Let's look at a practical example — requesting prices from different exchanges:

```rust
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    price: f64,
    latency_ms: u128,
}

// Simulating exchange API call
async fn fetch_price(exchange: &str, symbol: &str, delay_ms: u64) -> PriceQuote {
    let start = Instant::now();

    // Simulate network latency
    sleep(Duration::from_millis(delay_ms)).await;

    // Simulate different prices on different exchanges
    let price = match exchange {
        "Binance" => 42150.50,
        "Bybit" => 42148.00,
        "OKX" => 42152.25,
        _ => 42150.00,
    };

    PriceQuote {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        price,
        latency_ms: start.elapsed().as_millis(),
    }
}

#[tokio::main]
async fn main() {
    println!("Fetching BTC price from multiple exchanges...\n");

    let start = Instant::now();

    // select! returns the result of the first exchange to respond
    let fastest_quote = tokio::select! {
        quote = fetch_price("Binance", "BTC/USDT", 150) => quote,
        quote = fetch_price("Bybit", "BTC/USDT", 100) => quote,   // Fastest
        quote = fetch_price("OKX", "BTC/USDT", 200) => quote,
    };

    println!("First response from: {}", fastest_quote.exchange);
    println!("Price: ${:.2}", fastest_quote.price);
    println!("Latency: {}ms", fastest_quote.latency_ms);
    println!("Total time: {}ms", start.elapsed().as_millis());
}
```

## Order Execution with Timeout

One of the most important patterns — timeouts:

```rust
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderResult {
    order_id: u64,
    status: String,
    filled_price: f64,
    filled_quantity: f64,
}

// Simulating order submission to exchange
async fn submit_order(order: &Order) -> OrderResult {
    // Simulate execution delay (can be long with low liquidity)
    sleep(Duration::from_millis(500)).await;

    OrderResult {
        order_id: order.id,
        status: "FILLED".to_string(),
        filled_price: order.price,
        filled_quantity: order.quantity,
    }
}

// Cancel order
async fn cancel_order(order_id: u64) -> bool {
    println!("Cancelling order #{}", order_id);
    sleep(Duration::from_millis(50)).await;
    true
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: 12345,
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.1,
    };

    println!("Submitting order: {:?}\n", order);

    // Option 1: using select! with sleep
    let result = tokio::select! {
        result = submit_order(&order) => {
            println!("Order filled!");
            Some(result)
        }
        _ = sleep(Duration::from_millis(300)) => {
            println!("Timeout! Order not filled within 300ms");
            cancel_order(order.id).await;
            None
        }
    };

    match result {
        Some(r) => println!("Result: {:?}", r),
        None => println!("Order was cancelled"),
    }

    // Option 2: using tokio::time::timeout (more idiomatic)
    println!("\n--- Alternative with timeout ---\n");

    let order2 = Order {
        id: 12346,
        symbol: "ETH/USDT".to_string(),
        side: "SELL".to_string(),
        price: 2500.0,
        quantity: 1.0,
    };

    match timeout(Duration::from_millis(300), submit_order(&order2)).await {
        Ok(result) => println!("Success: {:?}", result),
        Err(_) => {
            println!("Timeout!");
            cancel_order(order2.id).await;
        }
    }
}
```

## Handling Multiple Data Sources

In real trading, you need to simultaneously listen to:
- Price updates
- Order executions
- Strategy signals
- Control commands

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
enum MarketEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    Signal { action: String, symbol: String },
}

#[derive(Debug)]
enum Command {
    Pause,
    Resume,
    Shutdown,
}

async fn price_feed(tx: mpsc::Sender<MarketEvent>) {
    let mut price = 42000.0;
    loop {
        sleep(Duration::from_millis(100)).await;
        price += (rand_simple() - 0.5) * 10.0;

        let event = MarketEvent::PriceUpdate {
            symbol: "BTC/USDT".to_string(),
            price,
        };

        if tx.send(event).await.is_err() {
            break;
        }
    }
}

async fn order_updates(tx: mpsc::Sender<MarketEvent>) {
    let mut order_id = 1000;
    loop {
        sleep(Duration::from_millis(500)).await;
        order_id += 1;

        let event = MarketEvent::OrderFilled {
            order_id,
            price: 42000.0 + rand_simple() * 100.0,
        };

        if tx.send(event).await.is_err() {
            break;
        }
    }
}

// Simple pseudo-random number generator
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let (market_tx, mut market_rx) = mpsc::channel::<MarketEvent>(100);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(10);

    // Start data sources
    let market_tx_clone = market_tx.clone();
    tokio::spawn(async move {
        price_feed(market_tx_clone).await;
    });

    tokio::spawn(async move {
        order_updates(market_tx).await;
    });

    // Simulate shutdown command after 1 second
    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        let _ = cmd_tx.send(Command::Shutdown).await;
    });

    println!("Starting trading engine...\n");

    let mut running = true;
    let mut event_count = 0;

    while running {
        tokio::select! {
            // Handle market events
            Some(event) = market_rx.recv() => {
                event_count += 1;
                match event {
                    MarketEvent::PriceUpdate { symbol, price } => {
                        println!("[PRICE] {}: ${:.2}", symbol, price);
                    }
                    MarketEvent::OrderFilled { order_id, price } => {
                        println!("[ORDER] #{} filled at ${:.2}", order_id, price);
                    }
                    MarketEvent::Signal { action, symbol } => {
                        println!("[SIGNAL] {} {}", action, symbol);
                    }
                }
            }

            // Handle control commands
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    Command::Pause => println!("\n⏸ Paused"),
                    Command::Resume => println!("\n▶ Resumed"),
                    Command::Shutdown => {
                        println!("\n🛑 Shutting down...");
                        running = false;
                    }
                }
            }

            // If all channels are closed
            else => {
                println!("All data sources closed");
                running = false;
            }
        }
    }

    println!("\nEvents processed: {}", event_count);
}
```

## Racing Between Strategies

Running multiple strategies where the first signal wins:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct TradeSignal {
    strategy: String,
    action: String,  // "BUY" or "SELL"
    symbol: String,
    confidence: f64,
}

async fn momentum_strategy(symbol: &str) -> Option<TradeSignal> {
    // Simulate momentum analysis
    sleep(Duration::from_millis(150)).await;

    // Assume strategy found a signal
    Some(TradeSignal {
        strategy: "Momentum".to_string(),
        action: "BUY".to_string(),
        symbol: symbol.to_string(),
        confidence: 0.75,
    })
}

async fn mean_reversion_strategy(symbol: &str) -> Option<TradeSignal> {
    // Simulate mean reversion analysis
    sleep(Duration::from_millis(200)).await;

    Some(TradeSignal {
        strategy: "Mean Reversion".to_string(),
        action: "SELL".to_string(),
        symbol: symbol.to_string(),
        confidence: 0.65,
    })
}

async fn breakout_strategy(symbol: &str) -> Option<TradeSignal> {
    // Simulate breakout analysis
    sleep(Duration::from_millis(100)).await;

    // This strategy found no signal
    None
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";

    println!("Running strategies for {}...\n", symbol);

    // Wait for first valid signal
    let signal = tokio::select! {
        result = momentum_strategy(symbol) => {
            println!("Momentum finished first");
            result
        }
        result = mean_reversion_strategy(symbol) => {
            println!("Mean Reversion finished first");
            result
        }
        result = breakout_strategy(symbol) => {
            println!("Breakout finished first");
            result
        }
    };

    match signal {
        Some(s) => {
            println!("\nSignal received:");
            println!("  Strategy: {}", s.strategy);
            println!("  Action: {}", s.action);
            println!("  Confidence: {:.0}%", s.confidence * 100.0);
        }
        None => {
            println!("\nNo signal found");
        }
    }
}
```

## Biased select — Branch Priority

By default, `select!` chooses branches randomly if multiple are ready simultaneously. For prioritization, use `biased`:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Priority {
    High,   // Risk management, stop-losses
    Medium, // Order execution
    Low,    // Logging, analytics
}

#[derive(Debug)]
struct Task {
    priority: Priority,
    description: String,
}

#[tokio::main]
async fn main() {
    let (high_tx, mut high_rx) = mpsc::channel::<Task>(10);
    let (medium_tx, mut medium_rx) = mpsc::channel::<Task>(10);
    let (low_tx, mut low_rx) = mpsc::channel::<Task>(10);

    // Send tasks
    tokio::spawn(async move {
        high_tx.send(Task {
            priority: Priority::High,
            description: "STOP-LOSS triggered!".to_string(),
        }).await.unwrap();
    });

    tokio::spawn(async move {
        medium_tx.send(Task {
            priority: Priority::Medium,
            description: "Execute order #123".to_string(),
        }).await.unwrap();
    });

    tokio::spawn(async move {
        low_tx.send(Task {
            priority: Priority::Low,
            description: "Record statistics".to_string(),
        }).await.unwrap();
    });

    // Give time for sending
    sleep(Duration::from_millis(10)).await;

    // biased guarantees checking branches in declaration order
    for _ in 0..3 {
        tokio::select! {
            biased;  // Check branches in order!

            Some(task) = high_rx.recv() => {
                println!("🔴 HIGH: {}", task.description);
            }
            Some(task) = medium_rx.recv() => {
                println!("🟡 MEDIUM: {}", task.description);
            }
            Some(task) = low_rx.recv() => {
                println!("🟢 LOW: {}", task.description);
            }
            else => break,
        }
    }
}
```

## Error Handling in select!

```rust
use tokio::time::{sleep, Duration};
use std::io;

async fn fetch_from_primary() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(100)).await;
    // Simulate primary source failure
    Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Primary down"))
}

async fn fetch_from_backup() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(150)).await;
    Ok(42150.50)
}

async fn fetch_from_cache() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(10)).await;
    Ok(42000.0)  // Stale but available price
}

#[tokio::main]
async fn main() {
    println!("Attempting to get BTC price...\n");

    // Get first successful result
    let price = tokio::select! {
        result = fetch_from_primary() => {
            match result {
                Ok(p) => {
                    println!("✓ Got from primary");
                    Some(p)
                }
                Err(e) => {
                    println!("✗ Primary error: {}", e);
                    None
                }
            }
        }
        result = fetch_from_backup() => {
            match result {
                Ok(p) => {
                    println!("✓ Got from backup");
                    Some(p)
                }
                Err(e) => {
                    println!("✗ Backup error: {}", e);
                    None
                }
            }
        }
    };

    // If both primary sources failed — use cache
    let final_price = match price {
        Some(p) => p,
        None => {
            println!("\nUsing cache...");
            fetch_from_cache().await.unwrap_or(0.0)
        }
    };

    println!("\nFinal price: ${:.2}", final_price);
}
```

## select! in a Loop — Event Loop

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[tokio::main]
async fn main() {
    let (price_tx, mut price_rx) = mpsc::channel::<f64>(100);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // Simulate price feed
    tokio::spawn(async move {
        let mut price = 42000.0;
        let mut interval = interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            price += (rand_simple() - 0.5) * 20.0;
            if price_tx.send(price).await.is_err() {
                break;
            }
        }
    });

    // Shutdown after 2 seconds
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = shutdown_tx.send(()).await;
    });

    let mut position = Position {
        symbol: "BTC/USDT".to_string(),
        quantity: 0.5,
        entry_price: 42000.0,
        current_price: 42000.0,
    };

    // Interval for periodic checks
    let mut check_interval = interval(Duration::from_millis(500));
    let start = Instant::now();

    println!("Monitoring position: {} @ ${:.2}\n",
             position.symbol, position.entry_price);

    loop {
        tokio::select! {
            // Price update
            Some(price) = price_rx.recv() => {
                position.current_price = price;
            }

            // Periodic position check
            _ = check_interval.tick() => {
                let elapsed = start.elapsed().as_secs_f64();
                println!("[{:.1}s] Price: ${:.2} | PnL: ${:.2} ({:+.2}%)",
                    elapsed,
                    position.current_price,
                    position.pnl(),
                    position.pnl_percent()
                );

                // Check stop-loss
                if position.pnl_percent() < -2.0 {
                    println!("⚠️  STOP-LOSS! Closing position");
                    break;
                }

                // Check take-profit
                if position.pnl_percent() > 2.0 {
                    println!("🎯 TAKE-PROFIT! Closing position");
                    break;
                }
            }

            // Shutdown command
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Shutdown signal received");
                break;
            }
        }
    }

    println!("\nSummary:");
    println!("  Final price: ${:.2}", position.current_price);
    println!("  PnL: ${:.2} ({:+.2}%)", position.pnl(), position.pnl_percent());
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::select!` | Wait for the first of multiple async events |
| Cancellation | Remaining branches are cancelled after one completes |
| `biased` | Enforce branch checking order |
| Timeout pattern | Combine select! with sleep for timeouts |
| Event loop | select! in a loop for continuous event processing |
| Multiple channels | Listen to multiple channels simultaneously |

## Homework

1. **Price Aggregator**: Implement a `get_best_price()` function that queries price from 5 exchanges in parallel and returns the first received price. Add a 500ms timeout — if no one responds, return an error.

2. **Priority Order Handler**: Create a system with three order queues (market, limit, stop). Use `biased` select so that market orders are processed first.

3. **Trading Bot with Graceful Shutdown**: Write a bot that:
   - Listens to price feed
   - Handles strategy signals
   - Shuts down gracefully on Ctrl+C (use `tokio::signal::ctrl_c()`)
   - Saves state before exit

4. **Multi-Exchange Arbitrage**: Implement a function that simultaneously:
   - Waits for price from exchange A
   - Waits for price from exchange B
   - Compares prices and logs arbitrage opportunity
   - Use `tokio::join!` to get both prices, or `select!` for the first one

## Navigation

[← Previous day](../187-async-await-basic/en.md) | [Next day →](../189-tokio-spawn-concurrent-tasks/en.md)
