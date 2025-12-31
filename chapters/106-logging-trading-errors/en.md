# Day 106: Logging Trading Errors

## Trading Analogy

Imagine a stock exchange trading floor. Every transaction, every error, every warning is recorded in a journal. If something goes wrong — auditors can reconstruct the entire chain of events. Logging in programming is the same transaction journal: it helps you understand what happened, when, and why.

Debugging a trading bot without logs is like searching for a needle in a haystack blindfolded. With logs — you see every step your system takes.

## Why Log Errors?

In trading, it's critical to know:
- **When** an error occurred (timing is crucial for trading)
- **What** went wrong (which request, what data)
- **Where** in the code the problem arose
- **Context**: which order, which instrument, what price

## The log Crate: Standard Logging Interface

```rust
// Cargo.toml:
// [dependencies]
// log = "0.4"
// env_logger = "0.10"

use log::{debug, info, warn, error, trace};

fn main() {
    // Initialize the logger
    env_logger::init();

    info!("Trading bot started");

    let ticker = "BTC/USDT";
    let price = 42000.0;

    debug!("Received price {} for {}", price, ticker);

    if price > 50000.0 {
        warn!("Price {} above threshold for {}", price, ticker);
    }

    match execute_order(ticker, price) {
        Ok(order_id) => info!("Order {} executed", order_id),
        Err(e) => error!("Order execution error: {}", e),
    }
}

fn execute_order(ticker: &str, price: f64) -> Result<u64, String> {
    if price <= 0.0 {
        return Err("Invalid price".to_string());
    }
    Ok(12345)
}
```

**Running with different log levels:**
```bash
RUST_LOG=debug cargo run    # Shows debug, info, warn, error
RUST_LOG=info cargo run     # Shows info, warn, error
RUST_LOG=warn cargo run     # Shows only warn and error
```

## Log Levels

| Level | When to Use | Trading Example |
|-------|-------------|-----------------|
| `trace!` | Algorithm details | Every price tick |
| `debug!` | Debug information | Indicator calculations |
| `info!` | Important events | Order execution |
| `warn!` | Potential issues | API rate limit approaching |
| `error!` | Operation failures | Order rejected |

```rust
use log::{trace, debug, info, warn, error};

fn process_market_data(prices: &[f64]) {
    trace!("Input data: {:?}", prices);

    let avg = prices.iter().sum::<f64>() / prices.len() as f64;
    debug!("Calculated average price: {:.2}", avg);

    if prices.len() < 10 {
        warn!("Insufficient data for reliable analysis: {} points", prices.len());
    }

    info!("Processed {} price points", prices.len());
}
```

## Logging with Error Context

```rust
use log::{error, warn, info};

#[derive(Debug)]
struct TradeError {
    code: String,
    message: String,
    ticker: String,
    attempted_price: f64,
}

fn place_order(ticker: &str, side: &str, quantity: f64, price: f64) -> Result<u64, TradeError> {
    info!("Placing order: {} {} {} at {}", side, quantity, ticker, price);

    // Simulation of validation
    if quantity <= 0.0 {
        let err = TradeError {
            code: "INVALID_QTY".to_string(),
            message: "Quantity must be positive".to_string(),
            ticker: ticker.to_string(),
            attempted_price: price,
        };
        error!(
            "Order error [{}]: {} | ticker={}, price={}, qty={}",
            err.code, err.message, ticker, price, quantity
        );
        return Err(err);
    }

    if price <= 0.0 {
        let err = TradeError {
            code: "INVALID_PRICE".to_string(),
            message: "Price must be positive".to_string(),
            ticker: ticker.to_string(),
            attempted_price: price,
        };
        error!(
            "Order error [{}]: {} | ticker={}, price={}",
            err.code, err.message, ticker, price
        );
        return Err(err);
    }

    // Successful placement
    let order_id = 98765;
    info!("Order placed: id={}, ticker={}, side={}, qty={}, price={}",
          order_id, ticker, side, quantity, price);
    Ok(order_id)
}

fn main() {
    env_logger::init();

    let _ = place_order("BTC/USDT", "BUY", 0.5, 42000.0);  // OK
    let _ = place_order("ETH/USDT", "BUY", -1.0, 3000.0);  // Error
    let _ = place_order("SOL/USDT", "SELL", 10.0, 0.0);    // Error
}
```

## Logging Error Chains with anyhow

```rust
use anyhow::{Context, Result};
use log::{error, info};

fn fetch_price(ticker: &str) -> Result<f64> {
    // Simulating API error
    if ticker == "INVALID" {
        anyhow::bail!("Ticker not found");
    }
    Ok(42000.0)
}

fn calculate_position_value(ticker: &str, quantity: f64) -> Result<f64> {
    let price = fetch_price(ticker)
        .with_context(|| format!("Failed to fetch price for {}", ticker))?;

    Ok(price * quantity)
}

fn process_portfolio(positions: &[(&str, f64)]) -> Result<f64> {
    let mut total = 0.0;

    for (ticker, qty) in positions {
        match calculate_position_value(ticker, *qty) {
            Ok(value) => {
                info!("Position {}: ${:.2}", ticker, value);
                total += value;
            }
            Err(e) => {
                // Log the full error chain
                error!("Error processing position {}: {:?}", ticker, e);

                // Can continue with remaining positions
                // or return error — depends on your logic
            }
        }
    }

    Ok(total)
}

fn main() -> Result<()> {
    env_logger::init();

    let positions = vec![
        ("BTC/USDT", 0.5),
        ("ETH/USDT", 2.0),
        ("INVALID", 1.0),
    ];

    let total = process_portfolio(&positions)?;
    info!("Total portfolio value: ${:.2}", total);

    Ok(())
}
```

## Structured Logging

For production log analysis, structured formats are useful:

```rust
use log::info;
use serde::Serialize;

#[derive(Serialize)]
struct OrderLog {
    event: &'static str,
    order_id: u64,
    ticker: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

fn log_order_event(order: &OrderLog) {
    // Log as JSON for parsing by monitoring tools
    info!(
        target: "orders",
        "{}",
        serde_json::to_string(order).unwrap_or_else(|_| format!("{:?}", order.order_id))
    );
}

fn main() {
    env_logger::init();

    let order_log = OrderLog {
        event: "ORDER_PLACED",
        order_id: 12345,
        ticker: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 42000.0,
        status: "PENDING".to_string(),
    };

    log_order_event(&order_log);
}
```

## Logging to File

```rust
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

fn setup_file_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}\n"
        )))
        .build("trading_bot.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
```

## Practical Example: Trading Session Logging

```rust
use log::{debug, info, warn, error};
use std::time::Instant;

struct TradingSession {
    start_time: Instant,
    trades_count: u32,
    errors_count: u32,
}

impl TradingSession {
    fn new() -> Self {
        info!("═══════════════════════════════════════");
        info!("Trading session started");
        info!("═══════════════════════════════════════");

        TradingSession {
            start_time: Instant::now(),
            trades_count: 0,
            errors_count: 0,
        }
    }

    fn execute_trade(&mut self, ticker: &str, side: &str, qty: f64, price: f64) -> Result<(), String> {
        debug!("Attempting execution: {} {} {} @ {}", side, qty, ticker, price);

        // Simulation of validations
        if qty <= 0.0 {
            self.errors_count += 1;
            error!("Rejected: invalid quantity {} for {}", qty, ticker);
            return Err("Invalid quantity".to_string());
        }

        if price <= 0.0 {
            self.errors_count += 1;
            error!("Rejected: invalid price {} for {}", price, ticker);
            return Err("Invalid price".to_string());
        }

        // Simulating random failure (10% chance)
        if (self.trades_count % 10) == 7 {
            self.errors_count += 1;
            warn!("Temporary API failure while executing order {}", ticker);
            return Err("API timeout".to_string());
        }

        self.trades_count += 1;
        info!("✓ Executed: {} {} {} @ {} | Trade #{}",
              side, qty, ticker, price, self.trades_count);

        Ok(())
    }

    fn end_session(&self) {
        let duration = self.start_time.elapsed();

        info!("═══════════════════════════════════════");
        info!("Trading session ended");
        info!("Duration: {:.2} sec", duration.as_secs_f64());
        info!("Trades: {}", self.trades_count);
        info!("Errors: {}", self.errors_count);

        if self.errors_count > 0 {
            warn!("Session ended with {} errors", self.errors_count);
        }

        let success_rate = if self.trades_count + self.errors_count > 0 {
            (self.trades_count as f64) / ((self.trades_count + self.errors_count) as f64) * 100.0
        } else {
            0.0
        };

        info!("Success rate: {:.1}%", success_rate);
        info!("═══════════════════════════════════════");
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug")
    ).init();

    let mut session = TradingSession::new();

    // Series of trading operations
    let orders = vec![
        ("BTC/USDT", "BUY", 0.1, 42000.0),
        ("ETH/USDT", "BUY", 1.0, 3000.0),
        ("SOL/USDT", "SELL", -5.0, 100.0),  // Error: negative quantity
        ("DOGE/USDT", "BUY", 1000.0, 0.0),  // Error: zero price
        ("ADA/USDT", "BUY", 500.0, 0.5),
        ("DOT/USDT", "SELL", 20.0, 7.0),
        ("LINK/USDT", "BUY", 10.0, 15.0),
        ("BTC/USDT", "SELL", 0.05, 42500.0),  // May trigger API timeout
    ];

    for (ticker, side, qty, price) in orders {
        let _ = session.execute_trade(ticker, side, qty, price);
    }

    session.end_session();
}
```

## Logging Patterns for Trading

### 1. Logging with Correlation ID

```rust
use uuid::Uuid;
use log::info;

fn process_order_with_correlation(ticker: &str, qty: f64) {
    let correlation_id = Uuid::new_v4();

    info!("[{}] Starting order processing {} qty={}", correlation_id, ticker, qty);
    // ... operations ...
    info!("[{}] Order processed", correlation_id);
}
```

### 2. Execution Time Logging

```rust
use std::time::Instant;
use log::{debug, warn};

fn timed_operation<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    if duration.as_millis() > 100 {
        warn!("{} took {:.2}ms (slow!)", name, duration.as_secs_f64() * 1000.0);
    } else {
        debug!("{} took {:.2}ms", name, duration.as_secs_f64() * 1000.0);
    }

    result
}
```

### 3. Error Counter with Logging

```rust
use std::collections::HashMap;
use log::{error, warn};

struct ErrorTracker {
    counts: HashMap<String, u32>,
    threshold: u32,
}

impl ErrorTracker {
    fn new(threshold: u32) -> Self {
        ErrorTracker {
            counts: HashMap::new(),
            threshold,
        }
    }

    fn record_error(&mut self, error_type: &str, message: &str) {
        let count = self.counts.entry(error_type.to_string()).or_insert(0);
        *count += 1;

        if *count == self.threshold {
            warn!(
                "Error threshold reached for '{}': {} per session",
                error_type, self.threshold
            );
        }

        error!("[{}] (#{}) {}", error_type, count, message);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `log` crate | Standard Rust logging interface |
| Log levels | trace, debug, info, warn, error |
| `env_logger` | Simple logger with environment variable configuration |
| Context | Add ticker, price, order_id to every message |
| Structured logs | JSON format for tool analysis |
| Execution time | Log slow operations |

## Homework

1. Create a logger for a trading bot that:
   - Records all orders at `info` level
   - Logs API errors at `error` level
   - Outputs indicator calculation details at `debug` level

2. Implement a `log_trade_chain()` function that logs the entire chain: from receiving a signal to order execution, with timestamps for each stage

3. Write an `ErrorAggregator` that collects errors during a session and outputs statistics at the end: what errors, how many times, what percentage of total operations

4. Implement a structured logger that outputs events in JSON format for subsequent analysis in monitoring systems

## Navigation

[← Previous day](../105-error-context/en.md) | [Next day →](../107-recovering-from-errors/en.md)
