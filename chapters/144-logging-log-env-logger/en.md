# Day 144: Logging: log and env_logger

## Trading Analogy

Imagine a trader's journal. It records all events: opening positions, price changes, stop-loss triggers, exchange connection errors. Without such a journal, it's impossible to understand why a strategy lost money overnight or why an order wasn't executed.

**Logging** in programming is the same journal, but automated. The `log` crate provides importance levels (from debug to critical errors), and `env_logger` outputs these records to the console with filtering capabilities.

## Adding Dependencies

```toml
# Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

## Basic Usage

```rust
use log::{debug, info, warn, error};

fn main() {
    // Initialize logger (once at program start)
    env_logger::init();

    info!("Trading bot started");
    debug!("Loading configuration...");

    let balance = 10000.0;
    info!("Initial balance: ${:.2}", balance);

    // Trading simulation
    if balance < 1000.0 {
        warn!("Low balance! Deposit recommended");
    }

    // Error simulation
    let api_response: Result<f64, &str> = Err("Connection timeout");
    if let Err(e) = api_response {
        error!("API error: {}", e);
    }

    info!("Trading bot stopped");
}
```

**Running with different levels:**

```bash
# Show only errors
RUST_LOG=error cargo run

# Show warnings and above
RUST_LOG=warn cargo run

# Show info and above (recommended for production)
RUST_LOG=info cargo run

# Show everything including debug
RUST_LOG=debug cargo run
```

## Logging Levels

```rust
use log::{trace, debug, info, warn, error};

fn main() {
    env_logger::init();

    // From most detailed to most important:
    trace!("Calculation details: step=1, value=0.0001");  // Deep debugging
    debug!("Received BTC quotes: 42000.0");               // For development
    info!("Order placed: BUY 0.5 BTC @ 42000");           // Normal operations
    warn!("Spread too wide: 0.5%");                       // Warnings
    error!("Failed to connect to exchange");              // Errors
}
```

| Level | Purpose | Trading Example |
|-------|---------|-----------------|
| `trace` | Finest details | Every quote tick |
| `debug` | Debug information | Indicator calculations |
| `info` | Important events | Opening/closing trades |
| `warn` | Warnings | High volatility |
| `error` | Errors | Rejected order |

## Logging Trading Operations

```rust
use log::{info, warn, error, debug};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // Trading day simulation
    let trades = simulate_trading_day();

    info!("Trading day complete. Total trades: {}", trades.len());
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn simulate_trading_day() -> Vec<Trade> {
    let mut trades = Vec::new();

    info!("=== Trading session started ===");

    // Attempting to open position
    let btc_price = 42000.0;
    let quantity = 0.5;

    debug!("Analyzing BTC market...");
    debug!("Current price: ${:.2}", btc_price);

    if btc_price > 40000.0 && btc_price < 50000.0 {
        info!("Buy signal: BTC @ ${:.2}", btc_price);

        let trade = Trade {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity,
            price: btc_price,
        };

        info!("Order executed: {} {} {} @ ${:.2}",
              trade.side, trade.quantity, trade.symbol, trade.price);
        trades.push(trade);
    }

    // Risk check
    let portfolio_value = 10000.0;
    let position_value = btc_price * quantity;
    let risk_percent = (position_value / portfolio_value) * 100.0;

    debug!("Position size: ${:.2} ({:.1}% of portfolio)", position_value, risk_percent);

    if risk_percent > 20.0 {
        warn!("Risk limit exceeded! Position: {:.1}% > 20%", risk_percent);
    }

    // Connection error simulation
    let exchange_status = check_exchange_connection();
    if let Err(e) = exchange_status {
        error!("Lost connection to exchange: {}", e);
    }

    info!("=== Trading session ended ===");

    trades
}

fn check_exchange_connection() -> Result<(), String> {
    // Random error simulation
    Ok(())
}
```

## Module Filtering

```rust
use log::{info, debug};

mod order_manager {
    use log::{info, debug, warn};

    pub fn place_order(symbol: &str, qty: f64, price: f64) {
        debug!("Preparing order...");
        info!("Order placed: {} {:.4} @ ${:.2}", symbol, qty, price);
    }

    pub fn cancel_order(order_id: u64) {
        warn!("Canceling order #{}", order_id);
    }
}

mod price_feed {
    use log::{debug, trace};

    pub fn on_price_update(symbol: &str, price: f64) {
        trace!("Tick: {} = ${:.2}", symbol, price);
    }

    pub fn calculate_sma(prices: &[f64]) -> f64 {
        debug!("Calculating SMA for {} values", prices.len());
        prices.iter().sum::<f64>() / prices.len() as f64
    }
}

fn main() {
    env_logger::init();

    info!("System starting");

    order_manager::place_order("BTC/USDT", 0.5, 42000.0);
    price_feed::on_price_update("BTC/USDT", 42100.0);

    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];
    let sma = price_feed::calculate_sma(&prices);
    info!("SMA: ${:.2}", sma);

    order_manager::cancel_order(12345);
}
```

**Module filtering:**

```bash
# Only orders
RUST_LOG=order_manager=info cargo run

# Price feed in trace mode, everything else — info
RUST_LOG=info,price_feed=trace cargo run

# Disable noisy module
RUST_LOG=info,price_feed=off cargo run
```

## Custom Output Format

```rust
use log::{info, warn};
use std::io::Write;

fn main() {
    // Custom format with timestamps
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    info!("Trading bot started");
    warn!("Test warning");
}
```

**Note:** To use `chrono`, add to `Cargo.toml`:

```toml
[dependencies]
chrono = "0.4"
```

## Structured Logging

```rust
use log::{info, warn, error};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // Logging with context
    log_trade_event("BTC/USDT", "BUY", 0.5, 42000.0, "market");
    log_trade_event("ETH/USDT", "SELL", 2.0, 2200.0, "limit");

    // Error logging with details
    log_order_error(12345, "Insufficient balance", 1000.0, 5000.0);
}

fn log_trade_event(symbol: &str, side: &str, qty: f64, price: f64, order_type: &str) {
    info!(
        "trade_event | symbol={} side={} qty={:.4} price={:.2} type={} value={:.2}",
        symbol, side, qty, price, order_type, qty * price
    );
}

fn log_order_error(order_id: u64, reason: &str, available: f64, required: f64) {
    error!(
        "order_error | order_id={} reason=\"{}\" available={:.2} required={:.2}",
        order_id, reason, available, required
    );
}
```

## Practical Example: Trading Bot with Logging

```rust
use log::{info, warn, error, debug, trace};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("=== Trading Bot v1.0 ===");

    let mut bot = TradingBot::new(10000.0);

    // Market data simulation
    let price_updates = vec![42000.0, 42100.0, 41950.0, 42200.0, 42150.0];

    for price in price_updates {
        bot.on_price_update("BTC/USDT", price);
    }

    bot.print_summary();
}

struct TradingBot {
    balance: f64,
    position: f64,
    entry_price: Option<f64>,
    trades_count: u32,
    pnl: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        info!("Initializing bot with balance: ${:.2}", initial_balance);
        TradingBot {
            balance: initial_balance,
            position: 0.0,
            entry_price: None,
            trades_count: 0,
            pnl: 0.0,
        }
    }

    fn on_price_update(&mut self, symbol: &str, price: f64) {
        trace!("Price received: {} = ${:.2}", symbol, price);

        // Simple strategy: buy on 0.1%+ dip
        if let Some(entry) = self.entry_price {
            let change = (price - entry) / entry * 100.0;
            debug!("Position change: {:.2}%", change);

            if change >= 0.2 {
                self.close_position(symbol, price);
            } else if change <= -0.5 {
                warn!("Stop-loss triggered at {:.2}% change", change);
                self.close_position(symbol, price);
            }
        } else {
            // No open position — looking for entry
            self.try_open_position(symbol, price);
        }
    }

    fn try_open_position(&mut self, symbol: &str, price: f64) {
        let position_size = self.balance * 0.1; // 10% of balance
        let quantity = position_size / price;

        if position_size < 100.0 {
            warn!("Insufficient funds to open position");
            return;
        }

        debug!("Entry analysis: size=${:.2}, quantity={:.4}", position_size, quantity);

        self.position = quantity;
        self.entry_price = Some(price);
        self.balance -= position_size;
        self.trades_count += 1;

        info!("OPEN {} | qty={:.4} @ ${:.2} | value=${:.2}",
              symbol, quantity, price, position_size);
    }

    fn close_position(&mut self, symbol: &str, price: f64) {
        if let Some(entry) = self.entry_price {
            let exit_value = self.position * price;
            let entry_value = self.position * entry;
            let trade_pnl = exit_value - entry_value;

            self.balance += exit_value;
            self.pnl += trade_pnl;

            let pnl_percent = (trade_pnl / entry_value) * 100.0;

            if trade_pnl >= 0.0 {
                info!("CLOSE {} | qty={:.4} @ ${:.2} | PnL=${:.2} ({:+.2}%)",
                      symbol, self.position, price, trade_pnl, pnl_percent);
            } else {
                warn!("CLOSE {} | qty={:.4} @ ${:.2} | PnL=${:.2} ({:+.2}%)",
                      symbol, self.position, price, trade_pnl, pnl_percent);
            }

            self.position = 0.0;
            self.entry_price = None;
        }
    }

    fn print_summary(&self) {
        info!("=== Trading Summary ===");
        info!("Trades: {}", self.trades_count);
        info!("Balance: ${:.2}", self.balance);
        info!("Total PnL: ${:.2}", self.pnl);

        if self.pnl >= 0.0 {
            info!("Result: PROFIT");
        } else {
            warn!("Result: LOSS");
        }
    }
}
```

## Logging to File

```rust
use log::{info, LevelFilter};
use std::fs::File;
use std::io::Write;

fn main() {
    // Configure file logging
    let target = Box::new(File::create("trading.log").expect("Can't create file"));

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    info!("Logging to file trading.log");
    info!("This message will be written to the file");
}
```

## What We Learned

| Concept | Description | Usage |
|---------|-------------|-------|
| `log` crate | Logging facade | Level macros |
| `env_logger` | Logger implementation | Output and filtering |
| Levels | trace → debug → info → warn → error | Different verbosity |
| RUST_LOG | Environment variable | Filter configuration |
| Modules | Path-based filtering | Noise isolation |

## Practical Exercises

1. **Order Journal**: Create a logging system for the order book with levels:
   - `trace`: every order book change
   - `debug`: aggregated updates
   - `info`: significant changes (>1% of volume)
   - `warn`: abnormally large orders
   - `error`: data inconsistencies

2. **Risk Monitoring**: Implement a risk control module with logging:
   - Position limit violations
   - Approaching margin call
   - High volatility alerts

3. **Trade Analysis**: Add statistics logging:
   - Win rate per session
   - Average PnL
   - Maximum drawdown

## Homework

1. Write a trading bot with full logging of all operations and ability to analyze the log file after the session

2. Create a `risk_manager` module with separate logging settings and alerts when limits are exceeded

3. Implement a log file parser that extracts trading statistics: total PnL, number of profitable/losing trades, longest position duration

4. Add log rotation: new file each day named `trading_YYYY-MM-DD.log`

## Navigation

[← Previous day](../143-clap-command-line/en.md) | [Next day →](../145-logging-levels-trading/en.md)
