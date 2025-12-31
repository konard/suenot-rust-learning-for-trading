# Day 145: Logging Levels for Trading

## Trading Analogy

Imagine a trading terminal with different notification levels:

- **ERROR** — stop-loss triggered, exchange unavailable, order rejected. You must know immediately!
- **WARN (warning)** — margin is close to limit, volatility above normal. Needs attention.
- **INFO (information)** — order executed, position opened/closed. Normal flow of events.
- **DEBUG** — position size calculation details, intermediate indicator values. For developers.
- **TRACE** — every price tick, every function call. Maximum detail.

Logging is a system for recording events in your trading program.

## Adding the log Crate

In Rust, the `log` crate is used for logging — it's a facade (abstraction), while the actual implementation is provided by crates like `env_logger`, `tracing-subscriber`, and others.

```toml
# Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

## Basic Usage

```rust
use log::{error, warn, info, debug, trace};

fn main() {
    env_logger::init();

    let btc_price = 42000.0;
    let order_size = 0.5;

    trace!("main() function called");
    debug!("Current BTC price: {}", btc_price);
    info!("Opening position: {} BTC @ ${}", order_size, btc_price);
    warn!("Volatility above average: 15%");
    error!("Connection to exchange lost!");
}
```

Running with different levels:
```bash
# Errors only
RUST_LOG=error cargo run

# Errors and warnings
RUST_LOG=warn cargo run

# Everything including debug
RUST_LOG=debug cargo run

# Maximum detail
RUST_LOG=trace cargo run
```

## Logging Levels

```rust
use log::{error, warn, info, debug, trace, Level};

fn demonstrate_levels() {
    // ERROR — critical errors requiring immediate attention
    error!("Critical error: balance is negative!");

    // WARN — potential problems
    warn!("Warning: position size exceeds recommended");

    // INFO — important normal operation events
    info!("Order #12345 executed successfully");

    // DEBUG — debugging information
    debug!("Calculated RSI: 67.5");

    // TRACE — maximum detail information
    trace!("Entering calculate_position_size()");
}

fn main() {
    env_logger::init();
    demonstrate_levels();
}
```

## Logging in Trading Functions

```rust
use log::{error, warn, info, debug, trace};

fn main() {
    env_logger::init();

    let result = execute_trade("BTCUSDT", 42000.0, 0.5, "BUY");
    match result {
        Ok(order_id) => info!("Trade executed: {}", order_id),
        Err(e) => error!("Trade failed: {}", e),
    }
}

fn execute_trade(
    symbol: &str,
    price: f64,
    quantity: f64,
    side: &str,
) -> Result<String, String> {
    trace!("execute_trade() called: {} {} {} @ {}", side, quantity, symbol, price);

    // Validation
    if quantity <= 0.0 {
        error!("Invalid quantity: {}", quantity);
        return Err("Invalid quantity".to_string());
    }

    if price <= 0.0 {
        error!("Invalid price: {}", price);
        return Err("Invalid price".to_string());
    }

    let value = price * quantity;
    debug!("Total order value: ${:.2}", value);

    // Risk check
    if value > 100_000.0 {
        warn!("Large order: ${:.2} — additional verification required", value);
    }

    // Simulate sending order
    let order_id = format!("ORD-{}", chrono_like_id());
    info!("Order sent: {} {} {} @ {} = ${:.2}",
          side, quantity, symbol, price, value);

    Ok(order_id)
}

fn chrono_like_id() -> u64 {
    1234567890 // In reality — timestamp
}
```

## Structured Logging

```rust
use log::{info, warn, error};

#[derive(Debug)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct Position {
    symbol: String,
    entry_price: f64,
    current_price: f64,
    quantity: f64,
    pnl: f64,
}

fn main() {
    env_logger::init();

    let order = Order {
        id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    // Logging structs via Debug
    info!("Order created: {:?}", order);

    let position = Position {
        symbol: "BTCUSDT".to_string(),
        entry_price: 42000.0,
        current_price: 43500.0,
        quantity: 0.5,
        pnl: 750.0,
    };

    if position.pnl < 0.0 {
        warn!("Position in loss: {:?}", position);
    } else {
        info!("Position in profit: {:?}", position);
    }
}
```

## Filtering by Module

```rust
// main.rs
mod trading_engine;
mod risk_manager;

use log::info;

fn main() {
    // Configuration: only warn for trading_engine, debug for risk_manager
    // RUST_LOG=warn,risk_manager=debug cargo run
    env_logger::init();

    info!("Starting trading system");
    trading_engine::process_order();
    risk_manager::check_risk();
}

// trading_engine.rs
use log::{info, debug, trace};

pub fn process_order() {
    trace!("Entering process_order");
    debug!("Processing order...");
    info!("Order processed");
}

// risk_manager.rs
use log::{info, debug, warn};

pub fn check_risk() {
    debug!("Checking risk limits");
    warn!("Risk close to limit");
    info!("Check completed");
}
```

## Customizing Log Format

```rust
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use chrono::Local;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    log::info!("Trading system started");
    log::warn!("High market volatility");
    log::error!("Exchange connection error");
}
```

Output:
```
2024-01-15 10:30:45.123 [INFO] trading_bot - Trading system started
2024-01-15 10:30:45.124 [WARN] trading_bot - High market volatility
2024-01-15 10:30:45.124 [ERROR] trading_bot - Exchange connection error
```

## Logging for Position Monitoring

```rust
use log::{error, warn, info, debug};

struct PositionMonitor {
    symbol: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    quantity: f64,
}

impl PositionMonitor {
    fn new(symbol: &str, entry: f64, stop: f64, take: f64, qty: f64) -> Self {
        info!("Position monitor created: {} entry={} SL={} TP={}",
              symbol, entry, stop, take);

        Self {
            symbol: symbol.to_string(),
            entry_price: entry,
            stop_loss: stop,
            take_profit: take,
            quantity: qty,
        }
    }

    fn check_price(&self, current_price: f64) {
        let pnl = (current_price - self.entry_price) * self.quantity;
        let pnl_percent = ((current_price / self.entry_price) - 1.0) * 100.0;

        debug!("{}: price={:.2} PnL={:.2} ({:.2}%)",
               self.symbol, current_price, pnl, pnl_percent);

        if current_price <= self.stop_loss {
            error!("STOP-LOSS TRIGGERED! {} price={} <= SL={}",
                   self.symbol, current_price, self.stop_loss);
        } else if current_price >= self.take_profit {
            info!("TAKE-PROFIT REACHED! {} price={} >= TP={}",
                  self.symbol, current_price, self.take_profit);
        } else if pnl_percent < -5.0 {
            warn!("{}: loss {:.2}% — approaching stop-loss",
                  self.symbol, pnl_percent);
        }
    }
}

fn main() {
    env_logger::init();

    let monitor = PositionMonitor::new("BTCUSDT", 42000.0, 40000.0, 45000.0, 0.5);

    // Simulate price changes
    let prices = [42500.0, 41500.0, 40500.0, 39500.0, 45500.0];

    for price in prices {
        monitor.check_price(price);
    }
}
```

## Performance Logging

```rust
use log::{info, debug, trace};
use std::time::Instant;

fn main() {
    env_logger::init();

    let prices: Vec<f64> = (0..10000)
        .map(|i| 42000.0 + (i as f64 * 0.1))
        .collect();

    let sma = timed_sma(&prices, 20);
    info!("SMA-20 calculated: {:.2}", sma.unwrap_or(0.0));
}

fn timed_sma(prices: &[f64], period: usize) -> Option<f64> {
    let start = Instant::now();
    trace!("Starting SMA-{} calculation", period);

    if prices.len() < period {
        debug!("Not enough data for SMA-{}: {} < {}",
               period, prices.len(), period);
        return None;
    }

    let sum: f64 = prices.iter().rev().take(period).sum();
    let result = sum / period as f64;

    let elapsed = start.elapsed();
    debug!("SMA-{} calculated in {:?}: {:.4}", period, elapsed, result);

    if elapsed.as_millis() > 100 {
        log::warn!("Slow SMA calculation: {:?}", elapsed);
    }

    Some(result)
}
```

## Conditional Logging

```rust
use log::{info, debug, warn, log_enabled, Level};

fn calculate_signals(prices: &[f64]) -> Vec<String> {
    let mut signals = Vec::new();

    // Check if debug level is enabled
    if log_enabled!(Level::Debug) {
        debug!("Analyzing {} price points", prices.len());
        for (i, price) in prices.iter().enumerate() {
            debug!("  [{}] {:.2}", i, price);
        }
    }

    // Expensive calculations only if trace is enabled
    if log_enabled!(Level::Trace) {
        let avg: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance: f64 = prices.iter()
            .map(|p| (p - avg).powi(2))
            .sum::<f64>() / prices.len() as f64;
        log::trace!("Statistics: avg={:.2}, var={:.2}", avg, variance);
    }

    // Main logic
    if prices.len() >= 2 {
        let last = prices[prices.len() - 1];
        let prev = prices[prices.len() - 2];

        if last > prev * 1.01 {
            let signal = "BUY".to_string();
            info!("Signal: {} (growth {:.2}%)", signal, (last/prev - 1.0) * 100.0);
            signals.push(signal);
        } else if last < prev * 0.99 {
            let signal = "SELL".to_string();
            warn!("Signal: {} (drop {:.2}%)", signal, (1.0 - last/prev) * 100.0);
            signals.push(signal);
        }
    }

    signals
}

fn main() {
    env_logger::init();

    let prices = vec![42000.0, 42100.0, 42500.0, 42300.0, 43000.0];
    let signals = calculate_signals(&prices);

    info!("Total signals: {}", signals.len());
}
```

## Practical Example: Trading Bot with Logging

```rust
use log::{error, warn, info, debug, trace, LevelFilter};
use env_logger::Builder;
use std::io::Write;

fn main() {
    // Configure logger with custom format
    Builder::new()
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // red
                log::Level::Warn => "\x1b[33m",  // yellow
                log::Level::Info => "\x1b[32m",  // green
                log::Level::Debug => "\x1b[36m", // cyan
                log::Level::Trace => "\x1b[90m", // gray
            };
            writeln!(
                buf,
                "{}[{}]\x1b[0m {} - {}",
                level_style,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();

    info!("=== Trading bot started ===");

    let mut bot = TradingBot::new(10000.0);

    // Simulate trading session
    let market_data = vec![
        ("BTCUSDT", 42000.0),
        ("BTCUSDT", 42500.0),
        ("BTCUSDT", 41800.0),
        ("BTCUSDT", 40000.0), // sharp drop
        ("BTCUSDT", 43000.0),
    ];

    for (symbol, price) in market_data {
        bot.on_price_update(symbol, price);
    }

    info!("=== Trading bot stopped ===");
    info!("Final balance: ${:.2}", bot.balance);
}

struct TradingBot {
    balance: f64,
    position: Option<Position>,
    trade_count: u32,
}

struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        debug!("Initializing bot with balance ${}", initial_balance);
        Self {
            balance: initial_balance,
            position: None,
            trade_count: 0,
        }
    }

    fn on_price_update(&mut self, symbol: &str, price: f64) {
        trace!("Price update received: {} = {}", symbol, price);

        match &self.position {
            None => self.check_entry(symbol, price),
            Some(_pos) => self.check_exit(price),
        }
    }

    fn check_entry(&mut self, symbol: &str, price: f64) {
        debug!("Checking entry conditions for {} @ {}", symbol, price);

        // Simple strategy: buy if price is "low"
        if price < 42000.0 {
            let quantity = (self.balance * 0.1) / price; // 10% of balance

            info!("OPENING POSITION: {} {} @ ${:.2}",
                  symbol, quantity, price);

            self.position = Some(Position {
                symbol: symbol.to_string(),
                entry_price: price,
                quantity,
            });

            self.balance -= quantity * price;
            self.trade_count += 1;
        }
    }

    fn check_exit(&mut self, price: f64) {
        if let Some(ref pos) = self.position {
            let pnl = (price - pos.entry_price) * pos.quantity;
            let pnl_percent = ((price / pos.entry_price) - 1.0) * 100.0;

            debug!("Position {}: PnL = ${:.2} ({:.2}%)",
                   pos.symbol, pnl, pnl_percent);

            // Stop-loss: -5%
            if pnl_percent <= -5.0 {
                error!("STOP-LOSS! Closing {} with loss {:.2}%",
                       pos.symbol, pnl_percent);
                self.close_position(price);
            }
            // Take-profit: +3%
            else if pnl_percent >= 3.0 {
                info!("TAKE-PROFIT! Closing {} with profit {:.2}%",
                      pos.symbol, pnl_percent);
                self.close_position(price);
            }
            // Warning when loss > 3%
            else if pnl_percent < -3.0 {
                warn!("Position {} in loss {:.2}% — close to stop-loss",
                      pos.symbol, pnl_percent);
            }
        }
    }

    fn close_position(&mut self, price: f64) {
        if let Some(pos) = self.position.take() {
            let proceeds = pos.quantity * price;
            self.balance += proceeds;

            let pnl = (price - pos.entry_price) * pos.quantity;
            info!("Position closed: {} @ ${:.2}, PnL: ${:.2}",
                  pos.symbol, price, pnl);
            debug!("New balance: ${:.2}", self.balance);
        }
    }
}
```

## What We Learned

| Level | Purpose | Trading Example |
|-------|---------|-----------------|
| `error!` | Critical errors | Stop-loss, API error |
| `warn!` | Warnings | High volatility, close to limits |
| `info!` | Important events | Order execution, position opening |
| `debug!` | Debugging | Indicator calculations, state |
| `trace!` | Tracing | Every tick, function calls |

## Homework

1. Create a trading bot with full logging of all operations. Use all 5 logging levels appropriately.

2. Implement a function `log_trade_summary(trades: &[Trade])` that outputs:
   - INFO: total number of trades
   - DEBUG: details of each trade
   - WARN: losing trades
   - ERROR: trades with loss > 10%

3. Write a portfolio monitoring system with logging:
   - TRACE: every price update
   - DEBUG: PnL recalculation
   - INFO: portfolio composition changes
   - WARN: drawdown > 5%
   - ERROR: drawdown > 10%

4. Implement a configurable logger that writes:
   - All levels to file `trading.log`
   - Only WARN and ERROR to console
   - Add timestamps and module name

## Navigation

[← Previous day](../144-file-serialization/en.md) | [Next day →](../146-tracing-spans/en.md)
