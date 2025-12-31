# Day 354: Production Logging

## Trading Analogy

Imagine you're managing the trading floor of a major exchange. You have hundreds of traders, thousands of trades per second, and you need to know everything that happens — but you can't personally watch every screen.

**Logging is like a system for recording all activities on the trading floor:**

| Trading Floor | Production Logging |
|---------------|-------------------|
| **Security cameras** | DEBUG logs — detailed recording of everything |
| **Voice announcements** | INFO logs — important events for everyone |
| **Warning alarms** | WARN logs — potential problems |
| **Fire alarm** | ERROR logs — critical failures |
| **Trade journal** | Structured logs for audit |

A good trader always keeps a trade journal. A good trading system always keeps logs.

## Log Levels

In Rust, the `tracing` crate is used for logging — it's much more powerful than the standard `log`:

```rust
use tracing::{trace, debug, info, warn, error, Level};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

/// Log levels from least to most critical:
/// TRACE < DEBUG < INFO < WARN < ERROR

fn demonstrate_log_levels() {
    // TRACE — the most detailed level, for debugging internal logic
    trace!(
        "Iteration {}: checking entry condition, price={}, threshold={}",
        42, 50000.0, 49500.0
    );

    // DEBUG — debugging information for developers
    debug!(
        price = 50000.0,
        volume = 1.5,
        "Market data received"
    );

    // INFO — important events during normal system operation
    info!(
        order_id = "ORD-12345",
        symbol = "BTCUSDT",
        side = "BUY",
        "Order executed"
    );

    // WARN — potential problems that didn't stop the system
    warn!(
        latency_ms = 250,
        threshold_ms = 100,
        "High API latency"
    );

    // ERROR — errors that affected system operation
    error!(
        error = "Connection refused",
        exchange = "binance",
        "Failed to connect to exchange"
    );
}

fn main() {
    // Initialize subscriber with level filter
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_target(true)
                .with_thread_ids(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
        .init();

    demonstrate_log_levels();
}
```

## Structured Logging for Trading

Structured logs are logs with metadata that are easy to parse and analyze:

```rust
use tracing::{info, warn, error, instrument, span, Level};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trade {
    id: String,
    order_id: String,
    symbol: String,
    price: f64,
    quantity: f64,
    commission: f64,
    timestamp: u64,
}

/// Use #[instrument] for automatic span creation with parameters
#[instrument(
    level = "info",
    name = "place_order",
    skip(order),
    fields(
        order_id = %order.id,
        symbol = %order.symbol,
        side = %order.side,
    )
)]
async fn place_order(order: Order) -> Result<Trade, String> {
    info!(
        price = order.price,
        quantity = order.quantity,
        "Sending order to exchange"
    );

    // Simulating order submission
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Risk management check
    if order.quantity * order.price > 100000.0 {
        warn!(
            order_value = order.quantity * order.price,
            max_allowed = 100000.0,
            "Order size exceeds limit"
        );
        return Err("Maximum order size exceeded".to_string());
    }

    let trade = Trade {
        id: format!("TRD-{}", uuid::Uuid::new_v4()),
        order_id: order.id.clone(),
        symbol: order.symbol.clone(),
        price: order.price,
        quantity: order.quantity,
        commission: order.price * order.quantity * 0.001,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    info!(
        trade_id = %trade.id,
        commission = trade.commission,
        "Order executed"
    );

    Ok(trade)
}

/// Logging with session context
#[instrument(
    level = "info",
    name = "trading_session",
    fields(session_id = %session_id, user = %user)
)]
async fn run_trading_session(session_id: &str, user: &str) {
    info!("Trading session started");

    let orders = vec![
        Order {
            id: "ORD-001".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 50000.0,
            quantity: 0.1,
            status: "NEW".to_string(),
        },
        Order {
            id: "ORD-002".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            price: 3000.0,
            quantity: 1.0,
            status: "NEW".to_string(),
        },
    ];

    for order in orders {
        match place_order(order).await {
            Ok(trade) => {
                info!(
                    trade_id = %trade.id,
                    pnl = (trade.price - 49500.0) * trade.quantity - trade.commission,
                    "Trade recorded"
                );
            }
            Err(e) => {
                error!(error = %e, "Order execution error");
            }
        }
    }

    info!("Trading session ended");
}

#[tokio::main]
async fn main() {
    // Initialize JSON logging for production
    tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_current_span(true)
        .with_span_list(true)
        .init();

    run_trading_session("SESSION-2024-001", "trader_bot_1").await;
}
```

## Log Rotation and Performance

In production, logs must be rotated to avoid filling up disk space:

```rust
use tracing::{info, Level};
use tracing_subscriber::{self, prelude::*};
use tracing_appender::{
    rolling::{RollingFileAppender, Rotation},
    non_blocking,
};

fn setup_production_logging() {
    // Log rotation by size or time
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,  // New file every day
        "/var/log/trading",
        "trading-bot.log"
    );

    // Non-blocking logging for high performance
    let (non_blocking_appender, _guard) = non_blocking(file_appender);

    // Configure subscriber with multiple layers
    tracing_subscriber::registry()
        // Console output for development
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        )
        // File output in JSON
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking_appender)
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG)
        )
        .init();

    info!("Logging initialized");
}

/// Demonstrate logging impact on performance
fn benchmark_logging() {
    use std::time::Instant;

    let iterations = 100_000;

    // Timing with logging
    let start = Instant::now();
    for i in 0..iterations {
        tracing::trace!(iteration = i, "High-frequency operation");
    }
    let with_trace = start.elapsed();

    // Timing with info (fewer logs)
    let start = Instant::now();
    for i in 0..iterations {
        if i % 1000 == 0 {
            tracing::info!(iteration = i, "Periodic report");
        }
    }
    let with_info = start.elapsed();

    println!("With TRACE logs: {:?}", with_trace);
    println!("With INFO logs (every 1000): {:?}", with_info);
}

fn main() {
    setup_production_logging();
    benchmark_logging();
}
```

## Contextual Logging for Order Tracking

When an order passes through multiple components, it's important to track its path:

```rust
use tracing::{info, warn, error, instrument, Span};
use tracing::field::Empty;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique ID generator for request tracking
fn generate_trace_id() -> String {
    format!("trace-{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
}

/// Order tracking context
#[derive(Clone, Debug)]
struct OrderContext {
    trace_id: String,
    order_id: String,
    symbol: String,
    user_id: String,
}

/// Order validation module
#[instrument(
    name = "validate_order",
    skip(ctx),
    fields(
        trace_id = %ctx.trace_id,
        order_id = %ctx.order_id,
        validation_result = Empty
    )
)]
async fn validate_order(ctx: &OrderContext, price: f64, quantity: f64) -> Result<(), String> {
    info!("Starting order validation");

    // Check minimum volume
    if quantity < 0.001 {
        warn!(quantity = quantity, min_quantity = 0.001, "Quantity below minimum");
        Span::current().record("validation_result", "REJECTED_MIN_QTY");
        return Err("Quantity below minimum".to_string());
    }

    // Check maximum price
    if price > 1_000_000.0 {
        warn!(price = price, max_price = 1_000_000.0, "Price above maximum");
        Span::current().record("validation_result", "REJECTED_MAX_PRICE");
        return Err("Price above maximum".to_string());
    }

    Span::current().record("validation_result", "PASSED");
    info!("Validation passed");
    Ok(())
}

/// Balance checking module
#[instrument(
    name = "check_balance",
    skip(ctx),
    fields(trace_id = %ctx.trace_id, order_id = %ctx.order_id)
)]
async fn check_balance(ctx: &OrderContext, required: f64) -> Result<(), String> {
    info!(required_balance = required, "Checking balance");

    // Simulating database query
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let available_balance = 100_000.0; // Simulation

    if required > available_balance {
        error!(
            required = required,
            available = available_balance,
            "Insufficient funds"
        );
        return Err("Insufficient funds".to_string());
    }

    info!(available_balance = available_balance, "Balance sufficient");
    Ok(())
}

/// Exchange submission module
#[instrument(
    name = "send_to_exchange",
    skip(ctx),
    fields(
        trace_id = %ctx.trace_id,
        order_id = %ctx.order_id,
        exchange_order_id = Empty
    )
)]
async fn send_to_exchange(
    ctx: &OrderContext,
    price: f64,
    quantity: f64
) -> Result<String, String> {
    info!(exchange = "binance", "Sending order to exchange");

    // Simulating network request
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let exchange_order_id = format!("EX-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    Span::current().record("exchange_order_id", &exchange_order_id);

    info!(
        exchange_order_id = %exchange_order_id,
        latency_ms = 50,
        "Order accepted by exchange"
    );

    Ok(exchange_order_id)
}

/// Full order processing cycle with tracing
#[instrument(
    name = "process_order",
    fields(
        trace_id = %generate_trace_id(),
        order_id = %order_id,
        symbol = %symbol,
        total_latency_ms = Empty
    )
)]
async fn process_order(
    order_id: &str,
    symbol: &str,
    user_id: &str,
    price: f64,
    quantity: f64,
) -> Result<String, String> {
    let start = std::time::Instant::now();

    let ctx = OrderContext {
        trace_id: Span::current()
            .field("trace_id")
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        order_id: order_id.to_string(),
        symbol: symbol.to_string(),
        user_id: user_id.to_string(),
    };

    info!(
        user_id = %user_id,
        price = price,
        quantity = quantity,
        "Starting order processing"
    );

    // Stage 1: Validation
    validate_order(&ctx, price, quantity).await?;

    // Stage 2: Balance check
    let required = price * quantity;
    check_balance(&ctx, required).await?;

    // Stage 3: Send to exchange
    let exchange_id = send_to_exchange(&ctx, price, quantity).await?;

    let total_latency = start.elapsed().as_millis();
    Span::current().record("total_latency_ms", total_latency);

    info!(
        exchange_id = %exchange_id,
        total_latency_ms = total_latency,
        "Order successfully processed"
    );

    Ok(exchange_id)
}

#[tokio::main]
async fn main() {
    // Initialize with analysis-friendly format
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    println!("=== Processing Order with Full Tracing ===\n");

    match process_order(
        "ORD-001",
        "BTCUSDT",
        "trader_123",
        50000.0,
        0.1
    ).await {
        Ok(exchange_id) => println!("\nSuccess! Exchange ID: {}", exchange_id),
        Err(e) => println!("\nError: {}", e),
    }
}
```

## Logging Metrics for Monitoring

Logs can be used to collect performance metrics:

```rust
use tracing::{info, warn, Span};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

/// Counters for metrics
struct TradingMetrics {
    orders_total: AtomicU64,
    orders_success: AtomicU64,
    orders_failed: AtomicU64,
    total_volume: AtomicU64, // In cents
    total_latency_ms: AtomicU64,
}

impl TradingMetrics {
    fn new() -> Self {
        TradingMetrics {
            orders_total: AtomicU64::new(0),
            orders_success: AtomicU64::new(0),
            orders_failed: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    fn record_order(&self, success: bool, volume: f64, latency_ms: u64) {
        self.orders_total.fetch_add(1, Ordering::Relaxed);

        if success {
            self.orders_success.fetch_add(1, Ordering::Relaxed);
            // Convert to cents for atomic storage
            self.total_volume.fetch_add((volume * 100.0) as u64, Ordering::Relaxed);
        } else {
            self.orders_failed.fetch_add(1, Ordering::Relaxed);
        }

        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    fn log_summary(&self) {
        let total = self.orders_total.load(Ordering::Relaxed);
        let success = self.orders_success.load(Ordering::Relaxed);
        let failed = self.orders_failed.load(Ordering::Relaxed);
        let volume = self.total_volume.load(Ordering::Relaxed) as f64 / 100.0;
        let latency = self.total_latency_ms.load(Ordering::Relaxed);

        let avg_latency = if total > 0 { latency / total } else { 0 };
        let success_rate = if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 };

        info!(
            orders_total = total,
            orders_success = success,
            orders_failed = failed,
            success_rate_pct = format!("{:.2}", success_rate),
            total_volume_usd = format!("{:.2}", volume),
            avg_latency_ms = avg_latency,
            "Trading metrics summary"
        );

        // Warning on low success rate
        if total > 10 && success_rate < 95.0 {
            warn!(
                success_rate = format!("{:.2}%", success_rate),
                threshold = "95%",
                "Low order success rate"
            );
        }

        // Warning on high latency
        if avg_latency > 200 {
            warn!(
                avg_latency_ms = avg_latency,
                threshold_ms = 200,
                "High average latency"
            );
        }
    }
}

/// Wrapper for measuring execution time
struct TimedOperation {
    name: String,
    start: Instant,
}

impl TimedOperation {
    fn start(name: &str) -> Self {
        TimedOperation {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    fn finish(self) -> u64 {
        let elapsed = self.start.elapsed().as_millis() as u64;
        info!(
            operation = %self.name,
            duration_ms = elapsed,
            "Operation completed"
        );
        elapsed
    }
}

async fn simulate_trading(metrics: Arc<TradingMetrics>) {
    for i in 0..20 {
        let timer = TimedOperation::start(&format!("order_{}", i));

        // Simulating order processing
        tokio::time::sleep(std::time::Duration::from_millis(20 + (i * 5) as u64)).await;

        let latency = timer.finish();

        // Simulation: 90% successful orders
        let success = i % 10 != 0;
        let volume = 1000.0 + (i as f64 * 100.0);

        metrics.record_order(success, volume, latency);

        // Log summary every 5 orders
        if (i + 1) % 5 == 0 {
            metrics.log_summary();
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Trading Metrics Logging ===\n");

    let metrics = Arc::new(TradingMetrics::new());

    simulate_trading(metrics.clone()).await;

    println!("\n=== Final Summary ===\n");
    metrics.log_summary();
}
```

## Filtering Logs by Component

In a large system, it's important to be able to filter logs by module:

```rust
use tracing::{info, debug, warn, Level};
use tracing_subscriber::{self, filter::EnvFilter, prelude::*};

mod market_data {
    use tracing::{info, debug};

    pub fn process_tick(symbol: &str, price: f64) {
        debug!(symbol = %symbol, price = price, "Tick received");
        // Processing...
        info!(symbol = %symbol, "Tick processed");
    }
}

mod order_manager {
    use tracing::{info, warn};

    pub fn place_order(order_id: &str, price: f64) {
        info!(order_id = %order_id, price = price, "Placing order");

        if price > 100000.0 {
            warn!(order_id = %order_id, "High order price");
        }
    }
}

mod risk_manager {
    use tracing::{info, warn, error};

    pub fn check_risk(position_size: f64) -> bool {
        info!(position_size = position_size, "Checking risk");

        if position_size > 10.0 {
            warn!(position_size = position_size, max = 10.0, "Position size exceeds limit");
            return false;
        }

        true
    }
}

fn main() {
    // Log filtering through RUST_LOG environment variable
    // Examples:
    // RUST_LOG=debug - all debug logs
    // RUST_LOG=market_data=debug,order_manager=info - different levels for modules
    // RUST_LOG=warn,risk_manager=debug - warn by default, debug for risk_manager

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default value if RUST_LOG is not set
            EnvFilter::new("info")
                .add_directive("market_data=debug".parse().unwrap())
                .add_directive("risk_manager=debug".parse().unwrap())
        });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    println!("=== Filtering Logs by Module ===\n");
    println!("Set RUST_LOG to change filtering");
    println!("Example: RUST_LOG=market_data=debug,order_manager=warn\n");

    // Demonstrate logging from different modules
    market_data::process_tick("BTCUSDT", 50000.0);
    order_manager::place_order("ORD-001", 50000.0);
    order_manager::place_order("ORD-002", 150000.0); // High price

    let risk_ok = risk_manager::check_risk(5.0);
    info!(approved = risk_ok, "Risk check result");

    let risk_exceeded = risk_manager::check_risk(15.0);
    info!(approved = risk_exceeded, "Risk check result");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Log levels** | TRACE, DEBUG, INFO, WARN, ERROR — from detailed to critical |
| **tracing** | Modern framework for structured logging in Rust |
| **Span** | Execution context that groups related logs |
| **#[instrument]** | Macro for automatic span creation with function parameters |
| **Structured logs** | Logs with metadata for machine parsing |
| **Log rotation** | Automatic creation of new files by time or size |
| **Non-blocking logging** | Writing logs without blocking the main thread |
| **EnvFilter** | Log filtering through the RUST_LOG environment variable |

## Practical Exercises

1. **Trade Audit System**: Create a system that:
   - Logs every trade with all details
   - Uses spans to group related events
   - Outputs logs in JSON format for analysis
   - Includes trace_id for request tracking

2. **Performance Monitoring**: Implement a system that:
   - Measures execution time of each operation
   - Logs warnings when thresholds are exceeded
   - Collects statistics in structured format
   - Generates periodic reports

3. **Multi-level Logging**: Create a configuration that:
   - Console output for development (colored, readable)
   - File output in JSON for production
   - Separate files for errors
   - Dynamic level changes through RUST_LOG

4. **Monitoring Integration**: Implement:
   - Export logs in Elasticsearch format
   - Add custom metrics to logs
   - Alerts on critical events
   - Dashboard with aggregated statistics

## Homework

1. **Logging System for Trading Bot**: Create a system that:
   - Full audit of all trading operations
   - Different levels for different modules (market_data=debug, risk=info)
   - Daily log rotation with compression of old files
   - Search logs by trace_id
   - Metrics: order count, success rate, average latency

2. **Log Analyzer**: Write a tool that:
   - Parses JSON logs from the trading system
   - Finds error patterns (recurring problems)
   - Builds a timeline of events for a specific order
   - Identifies latency anomalies
   - Generates a report in markdown format

3. **Real-time Monitoring**: Create a dashboard that:
   - Streams logs in real-time via WebSocket
   - Client-side filtering by level and module
   - Highlighting errors and warnings
   - Metric graphs (orders/sec, latency, errors)
   - Alerts to Telegram/Slack for critical errors

4. **Distributed Tracing**: Implement a system that:
   - Passes trace_id between microservices
   - Collects logs from all services in a single store
   - Visualizes request path through the system
   - Measures latency at each stage
   - Integration with OpenTelemetry

## Navigation

[← Previous day](../353-production-monitoring/en.md) | [Next day →](../359-kubernetes-orchestration/en.md)
