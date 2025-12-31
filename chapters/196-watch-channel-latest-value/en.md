# Day 196: Watch Channel: Latest Value

## Trading Analogy

Imagine an information board at an exchange that shows the current Bitcoin price. When the price changes, the board updates — but it only shows the **latest** price. You don't care about the history of changes in the last second, you only care about the current value right now.

This is exactly what a **watch channel** does in Tokio:
- One sender updates the value (the exchange updates the price)
- Multiple receivers see the latest value (traders look at the board)
- Intermediate values may be skipped (if the price changed twice in a second, you'll only see the latest one)

In real trading, watch channels are perfect for:
- **Current asset price** — everyone only needs the latest price
- **Exchange connection status** — connected/disconnected
- **Risk management parameters** — current position limit
- **Trading mode** — active/paused/stopped

## What is a Watch Channel?

A watch channel is a communication channel between tasks with special properties:

| Property | Description |
|----------|-------------|
| Multi-producer | Multiple senders can subscribe via `subscribe()` |
| Multi-consumer | Multiple receivers can read the value |
| Latest only | Only the most recent value is stored |
| Notifications | Receivers are notified about changes |
| No buffer | Intermediate values are lost |

## Creating a Watch Channel

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    // Create a channel with initial BTC price
    let (tx, rx) = watch::channel(42000.0_f64);

    // tx — Sender for sending new values
    // rx — Receiver for reading the latest value

    println!("Initial BTC price: ${}", *rx.borrow());
}
```

## Basic Example: Price Monitoring

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Channel for current Bitcoin price
    let (price_tx, mut price_rx) = watch::channel(42000.0_f64);

    // Producer task: updates the price
    let producer = tokio::spawn(async move {
        let prices = [42100.0, 42050.0, 42200.0, 42150.0, 42300.0];

        for price in prices {
            sleep(Duration::from_millis(500)).await;
            println!("[Exchange] New BTC price: ${}", price);

            if price_tx.send(price).is_err() {
                println!("[Exchange] All receivers disconnected");
                break;
            }
        }
    });

    // Consumer task: watches for changes
    let consumer = tokio::spawn(async move {
        loop {
            // changed() waits for a new value
            if price_rx.changed().await.is_err() {
                println!("[Trader] Channel closed");
                break;
            }

            // borrow_and_update() gets the value and marks it as read
            let price = *price_rx.borrow_and_update();
            println!("[Trader] Seeing price: ${}", price);
        }
    });

    producer.await.unwrap();
    consumer.await.unwrap();
}
```

## Multiple Receivers: Strategies and Monitoring

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_trade: f64,
}

#[tokio::main]
async fn main() {
    let initial_data = MarketData {
        symbol: "BTC/USDT".to_string(),
        bid: 41950.0,
        ask: 42050.0,
        last_trade: 42000.0,
    };

    let (data_tx, data_rx) = watch::channel(initial_data);

    // Clone receiver for each task
    let mut strategy_rx = data_rx.clone();
    let mut risk_rx = data_rx.clone();
    let mut logger_rx = data_rx;

    // Trading strategy watches the spread
    let strategy = tokio::spawn(async move {
        loop {
            if strategy_rx.changed().await.is_err() {
                break;
            }
            let data = strategy_rx.borrow_and_update().clone();
            let spread = data.ask - data.bid;

            if spread < 50.0 {
                println!("[Strategy] Tight spread ${:.2} — good for market order!", spread);
            } else {
                println!("[Strategy] Wide spread ${:.2} — use limit order", spread);
            }
        }
    });

    // Risk management watches the price
    let risk_manager = tokio::spawn(async move {
        let mut last_price = 0.0;

        loop {
            if risk_rx.changed().await.is_err() {
                break;
            }
            let data = risk_rx.borrow_and_update().clone();

            if last_price > 0.0 {
                let change_pct = (data.last_trade - last_price) / last_price * 100.0;
                if change_pct.abs() > 0.5 {
                    println!("[Risk] Sharp movement: {:.2}%!", change_pct);
                }
            }
            last_price = data.last_trade;
        }
    });

    // Logger records all changes
    let logger = tokio::spawn(async move {
        loop {
            if logger_rx.changed().await.is_err() {
                break;
            }
            let data = logger_rx.borrow_and_update().clone();
            println!("[Log] {} bid={:.2} ask={:.2} last={:.2}",
                data.symbol, data.bid, data.ask, data.last_trade);
        }
    });

    // Producer updates market data
    let updates = vec![
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42000.0, ask: 42020.0, last_trade: 42010.0 },
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42100.0, ask: 42200.0, last_trade: 42150.0 },
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42400.0, ask: 42450.0, last_trade: 42420.0 },
    ];

    for data in updates {
        sleep(Duration::from_millis(300)).await;
        let _ = data_tx.send(data);
    }

    // Give receivers time to process
    sleep(Duration::from_millis(100)).await;
    drop(data_tx); // Close the channel

    let _ = tokio::join!(strategy, risk_manager, logger);
}
```

## Checking for Changes Without Waiting

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel("active");

    // has_changed() checks without blocking
    println!("Changed? {}", rx.has_changed().unwrap());

    // Send a new value
    tx.send("paused").unwrap();

    println!("Changed? {}", rx.has_changed().unwrap());

    // borrow() reads without marking as read
    println!("Status: {}", *rx.borrow());
    println!("Changed? {}", rx.has_changed().unwrap()); // Still true!

    // borrow_and_update() reads AND marks as read
    {
        let status = rx.borrow_and_update();
        println!("Status: {}", *status);
    }
    println!("Changed? {}", rx.has_changed().unwrap()); // Now false
}
```

## Practical Example: Trading Mode

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug, PartialEq)]
enum TradingMode {
    Active,      // Trading allowed
    Paused,      // Temporary pause
    CloseOnly,   // Only closing positions
    Stopped,     // Full stop
}

struct TradingEngine {
    mode_rx: watch::Receiver<TradingMode>,
}

impl TradingEngine {
    fn new(mode_rx: watch::Receiver<TradingMode>) -> Self {
        TradingEngine { mode_rx }
    }

    async fn can_open_position(&self) -> bool {
        *self.mode_rx.borrow() == TradingMode::Active
    }

    async fn can_close_position(&self) -> bool {
        let mode = self.mode_rx.borrow().clone();
        mode == TradingMode::Active || mode == TradingMode::CloseOnly
    }

    async fn run(&mut self) {
        println!("[Engine] Starting trading engine");

        loop {
            // Wait for mode change
            if self.mode_rx.changed().await.is_err() {
                println!("[Engine] Control channel closed, stopping");
                break;
            }

            let mode = self.mode_rx.borrow_and_update().clone();

            match mode {
                TradingMode::Active => {
                    println!("[Engine] Mode ACTIVE: full trading allowed");
                }
                TradingMode::Paused => {
                    println!("[Engine] Mode PAUSED: waiting for resumption");
                }
                TradingMode::CloseOnly => {
                    println!("[Engine] Mode CLOSE_ONLY: only closing positions");
                }
                TradingMode::Stopped => {
                    println!("[Engine] Mode STOPPED: full stop");
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (mode_tx, mode_rx) = watch::channel(TradingMode::Active);

    let mut engine = TradingEngine::new(mode_rx);

    let engine_task = tokio::spawn(async move {
        engine.run().await;
    });

    // Simulate mode changes
    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Market is volatile -> PAUSED");
    mode_tx.send(TradingMode::Paused).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Closing positions -> CLOSE_ONLY");
    mode_tx.send(TradingMode::CloseOnly).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Market stabilized -> ACTIVE");
    mode_tx.send(TradingMode::Active).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] End of trading session -> STOPPED");
    mode_tx.send(TradingMode::Stopped).unwrap();

    engine_task.await.unwrap();
}
```

## Pattern: Real-time Configuration

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    max_drawdown_pct: f64,
    stop_loss_pct: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_size: 10.0,
            max_daily_loss: 5000.0,
            max_drawdown_pct: 5.0,
            stop_loss_pct: 2.0,
        }
    }
}

struct RiskManager {
    config_rx: watch::Receiver<RiskConfig>,
}

impl RiskManager {
    async fn check_order(&self, symbol: &str, size: f64, price: f64) -> Result<(), String> {
        let config = self.config_rx.borrow().clone();

        if size > config.max_position_size {
            return Err(format!(
                "Size {} exceeds limit {}",
                size, config.max_position_size
            ));
        }

        let order_value = size * price;
        if order_value > config.max_daily_loss {
            return Err(format!(
                "Order value ${} exceeds daily limit ${}",
                order_value, config.max_daily_loss
            ));
        }

        println!("[Risk] Order {} x {} @ ${} approved", symbol, size, price);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (config_tx, config_rx) = watch::channel(RiskConfig::default());

    let risk_manager = RiskManager { config_rx };

    // Check order with initial settings
    println!("=== Initial Configuration ===");
    let _ = risk_manager.check_order("BTC", 5.0, 42000.0).await;
    let _ = risk_manager.check_order("BTC", 15.0, 42000.0).await; // Rejected

    // Update configuration
    println!("\n=== Updating Limits ===");
    let new_config = RiskConfig {
        max_position_size: 20.0,
        max_daily_loss: 10000.0,
        max_drawdown_pct: 3.0,
        stop_loss_pct: 1.5,
    };
    config_tx.send(new_config).unwrap();

    // Now the order will pass
    let _ = risk_manager.check_order("BTC", 15.0, 42000.0).await;
}
```

## Comparison with Broadcast Channel

| Characteristic | watch | broadcast |
|----------------|-------|-----------|
| Buffer | 1 (latest only) | Configurable |
| Message skipping | Yes | Only on overflow |
| Initial value | Required | No |
| Subscribe on the fly | `subscribe()` | `subscribe()` |
| Use case | State | Events |

## When to Use Watch Channel

**Use watch when:**
- Only the latest value matters (current price, status)
- Receivers can skip intermediate values
- You need to propagate configuration
- You need a state signal (on/off)

**Don't use watch when:**
- Every message must be processed (orders!)
- You need change history
- Order of processing all events matters

## Practical Example: Portfolio Monitoring

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct PortfolioSnapshot {
    timestamp: u64,
    total_value: f64,
    positions: HashMap<String, f64>,
    unrealized_pnl: f64,
    daily_pnl: f64,
}

#[tokio::main]
async fn main() {
    let initial_snapshot = PortfolioSnapshot {
        timestamp: 0,
        total_value: 100000.0,
        positions: HashMap::new(),
        unrealized_pnl: 0.0,
        daily_pnl: 0.0,
    };

    let (snapshot_tx, snapshot_rx) = watch::channel(initial_snapshot);

    // UI thread updates the dashboard
    let mut ui_rx = snapshot_rx.clone();
    let ui_task = tokio::spawn(async move {
        loop {
            if ui_rx.changed().await.is_err() {
                break;
            }
            let snap = ui_rx.borrow_and_update().clone();
            println!("[UI] Portfolio: ${:.2} | PnL: ${:.2} | Positions: {}",
                snap.total_value, snap.daily_pnl, snap.positions.len());
        }
    });

    // Risk monitoring checks limits
    let mut risk_rx = snapshot_rx.clone();
    let risk_task = tokio::spawn(async move {
        loop {
            if risk_rx.changed().await.is_err() {
                break;
            }
            let snap = risk_rx.borrow_and_update().clone();

            let drawdown_pct = -snap.unrealized_pnl / 100000.0 * 100.0;
            if drawdown_pct > 2.0 {
                println!("[RISK ALERT] Drawdown {:.2}% exceeds limit!", drawdown_pct);
            }
        }
    });

    // Simulate portfolio updates
    let mut time = 1u64;
    for i in 0..5 {
        sleep(Duration::from_millis(200)).await;

        let mut positions = HashMap::new();
        positions.insert("BTC".to_string(), 2.0 + i as f64 * 0.5);
        positions.insert("ETH".to_string(), 10.0);

        let pnl = (i as f64 - 2.0) * 1000.0; // Varies from -2000 to +2000

        let snapshot = PortfolioSnapshot {
            timestamp: time,
            total_value: 100000.0 + pnl,
            positions,
            unrealized_pnl: pnl,
            daily_pnl: pnl,
        };

        let _ = snapshot_tx.send(snapshot);
        time += 1;
    }

    sleep(Duration::from_millis(100)).await;
    drop(snapshot_tx);

    let _ = tokio::join!(ui_task, risk_task);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `watch::channel(T)` | Creates a channel with initial value of type T |
| `tx.send(value)` | Updates the value, notifies receivers |
| `rx.borrow()` | Reads value without marking as read |
| `rx.borrow_and_update()` | Reads and marks as read |
| `rx.changed().await` | Waits for a new value |
| `rx.has_changed()` | Checks for changes |
| `tx.subscribe()` | Creates a new receiver |

## Homework

1. **Real-time Price**: Create a system where one producer updates the BTC price every 100ms, and three consumers:
   - Print every change
   - Calculate moving average of the last 5 values
   - Generate a signal when price changes more than 1%

2. **Trading Configuration**: Implement a `TradingConfig` struct with parameters (leverage, max_orders, allowed_symbols) and a system that:
   - Allows updating configuration on the fly
   - Has multiple workers that check the config before each operation

3. **Connection Status**: Create an exchange connection monitor with states (Connected, Reconnecting, Disconnected) and:
   - A trading engine that pauses when Disconnected
   - A logger that records all status changes
   - An alerter that sends a notification after 3 reconnect attempts

4. **Portfolio Dashboard**: Implement a portfolio monitoring system where:
   - A producer updates positions every 500ms
   - A UI consumer shows top-3 positions
   - A Risk consumer checks total exposure
   - A Performance consumer calculates daily PnL

## Navigation

[← Previous day](../195-broadcast-channel-all-subscribers/en.md) | [Next day →](../197-http-basics-reqwest-get/en.md)
