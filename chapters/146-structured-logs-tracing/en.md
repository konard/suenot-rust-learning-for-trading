# Day 146: Structured Logs: tracing

## Trading Analogy

Imagine you're a professional trader keeping a detailed **trading journal**. Each entry contains not just text like "bought bitcoin", but structured information:
- Operation timestamp
- Instrument ticker
- Entry price
- Position size
- Strategy that generated the signal
- Importance level (information, warning, error)

The `tracing` library in Rust works exactly the same way — it's not just logging, but a **structured diagnostics system** where every event carries context and can be analyzed automatically.

## Why tracing over log?

| Characteristic | `log` | `tracing` |
|---------------|-------|-----------|
| Structured data | Limited | Full support |
| Async code | Context issues | Native support |
| Spans (time intervals) | No | Yes |
| Performance | Good | Excellent (zero-cost) |
| Filtering | By level | By level, target, fields |

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Basic Usage

```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber;

fn main() {
    // Initialize subscriber
    tracing_subscriber::fmt::init();

    info!("Trading bot started");
    debug!("Loading configuration...");

    let btc_price = 42000.0;
    let position_size = 0.5;

    // Structured fields
    info!(
        ticker = "BTC/USDT",
        price = btc_price,
        size = position_size,
        "Position opened"
    );

    warn!(
        ticker = "BTC/USDT",
        current_pnl = -50.0,
        threshold = -100.0,
        "PnL approaching stop-loss threshold"
    );

    error!(
        ticker = "BTC/USDT",
        error_code = "INSUFFICIENT_BALANCE",
        required = 5000.0,
        available = 3000.0,
        "Insufficient funds to open position"
    );
}
```

## Log Levels

```rust
use tracing::{trace, debug, info, warn, error, Level};

fn log_levels_demo() {
    // From least to most important:
    trace!("Detailed debugging — every price tick");
    debug!("Debug information — indicator calculations");
    info!("Informational events — opening/closing positions");
    warn!("Warnings — high volatility, low balance");
    error!("Errors — failed orders, connection loss");
}
```

## Structured Fields

```rust
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    let order_id = "ORD-12345";
    let ticker = "ETH/USDT";
    let side = "BUY";
    let price = 2500.0_f64;
    let quantity = 1.5_f64;

    // Named fields
    info!(
        order_id = order_id,
        ticker = ticker,
        side = side,
        price = price,
        quantity = quantity,
        total_value = price * quantity,
        "Order created"
    );

    // Shorthand form (variable name = field name)
    info!(%order_id, %ticker, %side, price, quantity, "Order created");

    // Field formats:
    // ? — Debug formatting
    // % — Display formatting
    let prices = vec![42000.0, 42100.0, 42050.0];
    info!(prices = ?prices, "Recent prices");
}
```

## Spans — Temporal Contexts

A span is a named time interval. In trading, this is perfect for tracking the lifecycle of orders or trades.

```rust
use tracing::{info, info_span, warn, Instrument};

fn main() {
    tracing_subscriber::fmt::init();

    // Create span for trade tracking
    let trade_span = info_span!(
        "trade",
        trade_id = "TRD-001",
        ticker = "BTC/USDT",
        strategy = "momentum"
    );

    // Enter the span
    let _guard = trade_span.enter();

    info!(price = 42000.0, side = "BUY", "Opening position");

    // Simulate work
    analyze_market();
    check_stop_loss(42000.0, 41500.0);

    info!(price = 43000.0, pnl = 500.0, "Closing position");
    // Span automatically closes when guard goes out of scope
}

fn analyze_market() {
    let span = info_span!("market_analysis");
    let _guard = span.enter();

    info!("Calculating RSI...");
    info!("Calculating MACD...");
    info!(rsi = 65.0, macd = 0.5, "Indicators calculated");
}

fn check_stop_loss(entry: f64, stop: f64) {
    let span = info_span!("stop_loss_check", entry = entry, stop = stop);
    let _guard = span.enter();

    let risk = ((entry - stop) / entry) * 100.0;
    if risk > 2.0 {
        warn!(risk_percent = risk, "Risk exceeds 2%");
    } else {
        info!(risk_percent = risk, "Risk within acceptable range");
    }
}
```

## The #[instrument] Attribute

Automatically creates a span for a function:

```rust
use tracing::{info, instrument, warn};

fn main() {
    tracing_subscriber::fmt::init();

    let result = execute_trade("BTC/USDT", 42000.0, 0.5, "BUY");
    info!(result = ?result, "Trade result");
}

#[instrument(
    name = "execute_trade",
    skip(ticker),  // Don't include in span
    fields(
        ticker = %ticker,
        trade_type = "market"
    )
)]
fn execute_trade(ticker: &str, price: f64, quantity: f64, side: &str) -> Result<String, String> {
    info!("Checking balance...");

    if quantity <= 0.0 {
        warn!("Invalid quantity");
        return Err("Invalid quantity".to_string());
    }

    info!(
        value = price * quantity,
        "Executing order"
    );

    Ok(format!("ORD-{}", rand_id()))
}

fn rand_id() -> u32 {
    42 // Simplified for example
}
```

## Async Code with tracing

`tracing` works excellently with async/await:

```rust
use tracing::{info, instrument, Instrument};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Method 1: via attribute
    fetch_price("BTC/USDT").await;

    // Method 2: via .instrument()
    let span = tracing::info_span!("price_monitor");
    monitor_prices().instrument(span).await;
}

#[instrument]
async fn fetch_price(ticker: &str) -> f64 {
    info!("Fetching price...");

    // Simulate API call
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let price = 42000.0;
    info!(price = price, "Price received");
    price
}

async fn monitor_prices() {
    info!("Monitoring started");

    for i in 0..3 {
        let price = fetch_price("BTC/USDT").await;
        info!(iteration = i, price = price, "Price update");
    }

    info!("Monitoring completed");
}
```

## Log Filtering

```rust
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() {
    // Filter via RUST_LOG environment variable
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Or programmatically:
    // tracing_subscriber::fmt()
    //     .with_max_level(Level::DEBUG)
    //     .init();

    // Run: RUST_LOG=debug cargo run
    // Or: RUST_LOG=trading_bot=debug,hyper=warn cargo run
}
```

## Practical Example: Trading Bot with Logging

```rust
use tracing::{info, warn, error, instrument, info_span};
use tracing_subscriber::fmt;
use std::collections::HashMap;

fn main() {
    // Initialize with pretty format
    fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("=== Trading Bot v1.0 ===");

    let mut bot = TradingBot::new(10000.0);

    bot.process_signal("BTC/USDT", Signal::Buy, 42000.0, 0.1);
    bot.process_signal("ETH/USDT", Signal::Buy, 2500.0, 1.0);
    bot.process_signal("BTC/USDT", Signal::Sell, 43000.0, 0.1);

    bot.print_summary();
}

#[derive(Debug, Clone)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

struct TradingBot {
    balance: f64,
    positions: HashMap<String, Position>,
    trade_count: u32,
}

#[derive(Debug)]
struct Position {
    quantity: f64,
    entry_price: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        info!(balance = initial_balance, "Initializing bot");
        Self {
            balance: initial_balance,
            positions: HashMap::new(),
            trade_count: 0,
        }
    }

    #[instrument(skip(self), fields(trade_id = self.trade_count + 1))]
    fn process_signal(&mut self, ticker: &str, signal: Signal, price: f64, quantity: f64) {
        info!(signal = ?signal, "Processing signal");

        match signal {
            Signal::Buy => self.open_position(ticker, price, quantity),
            Signal::Sell => self.close_position(ticker, price, quantity),
            Signal::Hold => info!("Holding position"),
        }
    }

    fn open_position(&mut self, ticker: &str, price: f64, quantity: f64) {
        let span = info_span!("open_position", ticker = ticker);
        let _guard = span.enter();

        let cost = price * quantity;

        if cost > self.balance {
            error!(
                required = cost,
                available = self.balance,
                "Insufficient funds"
            );
            return;
        }

        self.balance -= cost;
        self.positions.insert(
            ticker.to_string(),
            Position {
                quantity,
                entry_price: price,
            },
        );
        self.trade_count += 1;

        info!(
            cost = cost,
            new_balance = self.balance,
            "Position opened"
        );
    }

    fn close_position(&mut self, ticker: &str, price: f64, quantity: f64) {
        let span = info_span!("close_position", ticker = ticker);
        let _guard = span.enter();

        if let Some(position) = self.positions.remove(ticker) {
            let sell_quantity = quantity.min(position.quantity);
            let revenue = price * sell_quantity;
            let cost = position.entry_price * sell_quantity;
            let pnl = revenue - cost;

            self.balance += revenue;
            self.trade_count += 1;

            if pnl >= 0.0 {
                info!(
                    pnl = pnl,
                    roi_percent = (pnl / cost) * 100.0,
                    "Profitable trade"
                );
            } else {
                warn!(
                    pnl = pnl,
                    roi_percent = (pnl / cost) * 100.0,
                    "Loss-making trade"
                );
            }
        } else {
            warn!("Position not found");
        }
    }

    fn print_summary(&self) {
        info!(
            balance = self.balance,
            open_positions = self.positions.len(),
            total_trades = self.trade_count,
            "=== Session Summary ==="
        );
    }
}
```

## Output Formats

```rust
use tracing_subscriber::fmt;

fn setup_json_logging() {
    // JSON format for machine processing
    fmt()
        .json()
        .init();
}

fn setup_compact_logging() {
    // Compact format for console
    fmt()
        .compact()
        .init();
}

fn setup_pretty_logging() {
    // Pretty format for development
    fmt()
        .pretty()
        .init();
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| Events | Point-in-time records | Order execution, price receipt |
| Spans | Time intervals | Trade lifecycle |
| Fields | Structured data | Price, volume, ticker, PnL |
| Levels | Event importance | Errors > Warnings > Info |
| Subscribers | Event processing | Console, file, monitoring system |

## Practical Exercises

1. **Basic Logging**: Add structured logging to a PnL calculation function with fields: entry_price, exit_price, quantity, gross_pnl, fees, net_pnl.

2. **Trade Spans**: Create a function that opens a span for each trade and logs all stages: validation -> execution -> confirmation.

3. **Module Filtering**: Configure logging so the `risk_management` module outputs DEBUG while others only output INFO.

4. **Async Monitoring**: Write an async portfolio monitoring function that logs current value every N seconds using `#[instrument]`.

## Homework

1. Create a `TradeLogger` struct that wraps tracing and provides methods: `log_order_placed`, `log_order_filled`, `log_order_cancelled`, `log_position_opened`, `log_position_closed`.

2. Implement middleware for logging all incoming market data (price, volume, time) with automatic span creation for each ticker.

3. Write a function `setup_production_logging()` that configures:
   - JSON format for stdout
   - Filtering via environment variable
   - Including thread_id and timestamp

4. Create an alert system based on tracing: if PnL drops below a threshold, an ERROR level event should be generated with full position information.

## Navigation

← Previous day | Next day →

*Note: Links to adjacent chapters will be added once they are created.*
