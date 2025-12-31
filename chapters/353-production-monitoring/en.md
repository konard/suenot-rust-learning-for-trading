# Day 353: Production Monitoring

## Trading Analogy

Imagine you have a trading bot running 24/7 on a real market. It's like an experienced trader who must constantly monitor:

**System Health = Trader's Health:**
- Pulse (heartbeat) — system is running and responsive
- Temperature (CPU/Memory) — load is within normal range
- Blood pressure (latency) — delays are acceptable

**Trading Metrics = Performance Indicators:**
- Number of trades per minute
- Average profit per trade
- Order execution time
- Slippage

**Alerts = Alarm Signals:**
- Lost connection to exchange — notify immediately
- Abnormal losses — stop trading
- Memory exhaustion — warn in advance

| Aspect | Without Monitoring | With Monitoring |
|--------|-------------------|-----------------|
| **Problem Detection** | When money is lost | In advance |
| **Response Time** | Hours/days | Seconds/minutes |
| **Diagnostics** | Guessing | Precise data |
| **Optimization** | Blind | Based on metrics |

## Monitoring Basics in Rust

### Types of Metrics

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Types of metrics for a trading system
#[derive(Debug)]
pub struct TradingMetrics {
    // Counters — only increase
    pub orders_placed: AtomicU64,
    pub orders_filled: AtomicU64,
    pub orders_cancelled: AtomicU64,
    pub orders_rejected: AtomicU64,

    // Gauges — current value
    pub open_positions: AtomicUsize,
    pub active_orders: AtomicUsize,
    pub available_balance_cents: AtomicU64,

    // Histograms — distribution of values
    // For simplicity, we store sum and count
    pub order_latency_sum_us: AtomicU64,
    pub order_latency_count: AtomicU64,
}

impl TradingMetrics {
    pub fn new() -> Self {
        TradingMetrics {
            orders_placed: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_cancelled: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            open_positions: AtomicUsize::new(0),
            active_orders: AtomicUsize::new(0),
            available_balance_cents: AtomicU64::new(0),
            order_latency_sum_us: AtomicU64::new(0),
            order_latency_count: AtomicU64::new(0),
        }
    }

    pub fn record_order_placed(&self) {
        self.orders_placed.fetch_add(1, Ordering::Relaxed);
        self.active_orders.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_order_filled(&self, latency: Duration) {
        self.orders_filled.fetch_add(1, Ordering::Relaxed);
        self.active_orders.fetch_sub(1, Ordering::Relaxed);

        let latency_us = latency.as_micros() as u64;
        self.order_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.order_latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_order_cancelled(&self) {
        self.orders_cancelled.fetch_add(1, Ordering::Relaxed);
        self.active_orders.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_order_rejected(&self) {
        self.orders_rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_open_positions(&self, count: usize) {
        self.open_positions.store(count, Ordering::Relaxed);
    }

    pub fn set_balance(&self, balance_cents: u64) {
        self.available_balance_cents.store(balance_cents, Ordering::Relaxed);
    }

    pub fn average_order_latency(&self) -> Option<Duration> {
        let count = self.order_latency_count.load(Ordering::Relaxed);
        if count == 0 {
            return None;
        }
        let sum = self.order_latency_sum_us.load(Ordering::Relaxed);
        Some(Duration::from_micros(sum / count))
    }

    pub fn report(&self) {
        println!("=== Trading Metrics ===");
        println!("Orders: placed={}, filled={}, cancelled={}, rejected={}",
            self.orders_placed.load(Ordering::Relaxed),
            self.orders_filled.load(Ordering::Relaxed),
            self.orders_cancelled.load(Ordering::Relaxed),
            self.orders_rejected.load(Ordering::Relaxed));
        println!("Active orders: {}", self.active_orders.load(Ordering::Relaxed));
        println!("Open positions: {}", self.open_positions.load(Ordering::Relaxed));
        println!("Balance: ${:.2}",
            self.available_balance_cents.load(Ordering::Relaxed) as f64 / 100.0);
        if let Some(latency) = self.average_order_latency() {
            println!("Avg order latency: {:?}", latency);
        }
    }
}

fn main() {
    let metrics = Arc::new(TradingMetrics::new());
    metrics.set_balance(100_000_00); // $100,000

    // Simulate trading activity
    for i in 0..100 {
        metrics.record_order_placed();

        // 90% of orders fill
        if i % 10 != 0 {
            let latency = Duration::from_millis(50 + (i % 30) as u64);
            metrics.record_order_filled(latency);
        } else {
            metrics.record_order_cancelled();
        }
    }

    metrics.set_open_positions(5);
    metrics.report();
}
```

### Health Checks

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// Component status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: u64,  // Unix timestamp
    pub latency_ms: u64,
}

/// Health check manager
pub struct HealthManager {
    checks: HashMap<String, Box<dyn Fn() -> HealthCheck + Send + Sync>>,
}

impl HealthManager {
    pub fn new() -> Self {
        HealthManager {
            checks: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, check: F)
    where
        F: Fn() -> HealthCheck + Send + Sync + 'static,
    {
        self.checks.insert(name.to_string(), Box::new(check));
    }

    pub fn check_all(&self) -> Vec<HealthCheck> {
        self.checks.values().map(|check| check()).collect()
    }

    pub fn is_healthy(&self) -> bool {
        self.check_all().iter().all(|c| matches!(c.status, HealthStatus::Healthy))
    }
}

/// Exchange connection check
struct ExchangeConnection {
    is_connected: AtomicBool,
    last_message_time: AtomicU64,
}

impl ExchangeConnection {
    fn new() -> Self {
        ExchangeConnection {
            is_connected: AtomicBool::new(true),
            last_message_time: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
        }
    }

    fn health_check(&self) -> HealthCheck {
        let start = Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let is_connected = self.is_connected.load(Ordering::Relaxed);
        let last_msg = self.last_message_time.load(Ordering::Relaxed);
        let seconds_since_message = now.saturating_sub(last_msg);

        let status = if !is_connected {
            HealthStatus::Unhealthy("Disconnected from exchange".to_string())
        } else if seconds_since_message > 30 {
            HealthStatus::Unhealthy(format!("No data for {} seconds", seconds_since_message))
        } else if seconds_since_message > 10 {
            HealthStatus::Degraded(format!("Slow data: {} seconds since last update", seconds_since_message))
        } else {
            HealthStatus::Healthy
        };

        HealthCheck {
            name: "exchange_connection".to_string(),
            status,
            last_check: now,
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn simulate_message(&self) {
        self.last_message_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    fn disconnect(&self) {
        self.is_connected.store(false, Ordering::Relaxed);
    }
}

/// Memory health check
fn memory_health_check() -> HealthCheck {
    let start = Instant::now();

    // In real code, use sys-info or procfs
    // Here's a simulation
    let used_mb = 512;
    let total_mb = 2048;
    let usage_percent = (used_mb * 100) / total_mb;

    let status = if usage_percent > 90 {
        HealthStatus::Unhealthy(format!("Memory critical: {}%", usage_percent))
    } else if usage_percent > 75 {
        HealthStatus::Degraded(format!("Memory high: {}%", usage_percent))
    } else {
        HealthStatus::Healthy
    };

    HealthCheck {
        name: "memory".to_string(),
        status,
        last_check: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        latency_ms: start.elapsed().as_millis() as u64,
    }
}

fn main() {
    let exchange = Arc::new(ExchangeConnection::new());
    let exchange_clone = Arc::clone(&exchange);

    let mut health_manager = HealthManager::new();

    // Register checks
    health_manager.register("exchange", move || exchange_clone.health_check());
    health_manager.register("memory", memory_health_check);

    // Check system health
    println!("=== Health Check (all good) ===");
    for check in health_manager.check_all() {
        println!("{}: {:?} ({}ms)", check.name, check.status, check.latency_ms);
    }
    println!("System healthy: {}\n", health_manager.is_healthy());

    // Simulate a problem
    exchange.disconnect();

    println!("=== Health Check (after exchange disconnect) ===");
    for check in health_manager.check_all() {
        println!("{}: {:?} ({}ms)", check.name, check.status, check.latency_ms);
    }
    println!("System healthy: {}", health_manager.is_healthy());
}
```

## Prometheus Integration

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Prometheus-compatible metrics format
pub struct PrometheusRegistry {
    counters: RwLock<HashMap<String, AtomicU64>>,
    gauges: RwLock<HashMap<String, AtomicU64>>,
    labels: RwLock<HashMap<String, HashMap<String, String>>>,
}

impl PrometheusRegistry {
    pub fn new() -> Self {
        PrometheusRegistry {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            labels: RwLock::new(HashMap::new()),
        }
    }

    pub fn counter(&self, name: &str) -> &AtomicU64 {
        let mut counters = self.counters.write().unwrap();
        if !counters.contains_key(name) {
            counters.insert(name.to_string(), AtomicU64::new(0));
        }
        // Safe: we only add, never remove
        unsafe {
            let ptr = counters.get(name).unwrap() as *const AtomicU64;
            &*ptr
        }
    }

    pub fn inc_counter(&self, name: &str) {
        let counters = self.counters.read().unwrap();
        if let Some(counter) = counters.get(name) {
            counter.fetch_add(1, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.counters.write().unwrap();
            counters.entry(name.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn set_gauge(&self, name: &str, value: u64) {
        let mut gauges = self.gauges.write().unwrap();
        gauges.entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::Relaxed);
    }

    pub fn set_labels(&self, name: &str, labels: HashMap<String, String>) {
        let mut all_labels = self.labels.write().unwrap();
        all_labels.insert(name.to_string(), labels);
    }

    /// Export in Prometheus format
    pub fn export(&self) -> String {
        let mut output = String::new();

        // Export counters
        let counters = self.counters.read().unwrap();
        let labels = self.labels.read().unwrap();

        for (name, value) in counters.iter() {
            let label_str = if let Some(l) = labels.get(name) {
                let pairs: Vec<String> = l.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            } else {
                String::new()
            };

            output.push_str(&format!(
                "# TYPE {} counter\n{}{} {}\n",
                name, name, label_str, value.load(Ordering::Relaxed)
            ));
        }

        // Export gauges
        let gauges = self.gauges.read().unwrap();
        for (name, value) in gauges.iter() {
            let label_str = if let Some(l) = labels.get(name) {
                let pairs: Vec<String> = l.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            } else {
                String::new()
            };

            output.push_str(&format!(
                "# TYPE {} gauge\n{}{} {}\n",
                name, name, label_str, value.load(Ordering::Relaxed)
            ));
        }

        output
    }
}

/// Trading system metrics for Prometheus
struct TradingPrometheusMetrics {
    registry: Arc<PrometheusRegistry>,
}

impl TradingPrometheusMetrics {
    fn new() -> Self {
        let registry = Arc::new(PrometheusRegistry::new());

        // Initialize metrics with labels
        let mut labels = HashMap::new();
        labels.insert("exchange".to_string(), "binance".to_string());
        labels.insert("symbol".to_string(), "BTCUSDT".to_string());
        registry.set_labels("trading_orders_total", labels);

        TradingPrometheusMetrics { registry }
    }

    fn record_order(&self, order_type: &str) {
        let metric_name = format!("trading_orders_{}", order_type);
        self.registry.inc_counter(&metric_name);
        self.registry.inc_counter("trading_orders_total");
    }

    fn set_position_size(&self, size: f64) {
        // Store in hundredths for precision
        self.registry.set_gauge("trading_position_size", (size * 100.0) as u64);
    }

    fn set_pnl(&self, pnl: f64) {
        // Store P&L in cents (can be negative, so add offset)
        let offset_pnl = ((pnl + 1_000_000.0) * 100.0) as u64;
        self.registry.set_gauge("trading_pnl_cents", offset_pnl);
    }

    fn export(&self) -> String {
        self.registry.export()
    }
}

fn main() {
    let metrics = TradingPrometheusMetrics::new();

    // Simulate trading
    for _ in 0..50 {
        metrics.record_order("filled");
    }
    for _ in 0..10 {
        metrics.record_order("cancelled");
    }
    for _ in 0..5 {
        metrics.record_order("rejected");
    }

    metrics.set_position_size(1.5);
    metrics.set_pnl(2500.50);

    println!("=== Prometheus Export ===\n");
    println!("{}", metrics.export());
}
```

## Production Logging

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Structured log entry
#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Simple structured logger
pub struct StructuredLogger {
    min_level: LogLevel,
    log_count: AtomicUsize,
}

impl StructuredLogger {
    pub fn new(min_level: LogLevel) -> Self {
        StructuredLogger {
            min_level,
            log_count: AtomicUsize::new(0),
        }
    }

    pub fn log(&self, level: LogLevel, message: &str, fields: HashMap<String, String>) {
        if level < self.min_level {
            return;
        }

        self.log_count.fetch_add(1, Ordering::Relaxed);

        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            level,
            message: message.to_string(),
            fields,
        };

        // Output in JSON format for processing
        println!("{}", self.format_json(&entry));
    }

    fn format_json(&self, entry: &LogEntry) -> String {
        let level_str = match entry.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let fields_json: Vec<String> = entry.fields.iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect();

        format!(
            "{{\"ts\":{},\"level\":\"{}\",\"msg\":\"{}\",{}}}",
            entry.timestamp,
            level_str,
            entry.message.replace("\"", "\\\""),
            fields_json.join(",")
        )
    }

    pub fn debug(&self, message: &str, fields: HashMap<String, String>) {
        self.log(LogLevel::Debug, message, fields);
    }

    pub fn info(&self, message: &str, fields: HashMap<String, String>) {
        self.log(LogLevel::Info, message, fields);
    }

    pub fn warn(&self, message: &str, fields: HashMap<String, String>) {
        self.log(LogLevel::Warn, message, fields);
    }

    pub fn error(&self, message: &str, fields: HashMap<String, String>) {
        self.log(LogLevel::Error, message, fields);
    }

    pub fn log_count(&self) -> usize {
        self.log_count.load(Ordering::Relaxed)
    }
}

/// Macro for convenient field creation
macro_rules! log_fields {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

/// Log trading events
fn log_order_placed(logger: &StructuredLogger, order_id: &str, symbol: &str, side: &str, price: f64, qty: f64) {
    logger.info("Order placed", log_fields! {
        "order_id" => order_id,
        "symbol" => symbol,
        "side" => side,
        "price" => format!("{:.2}", price),
        "quantity" => format!("{:.4}", qty),
    });
}

fn log_order_filled(logger: &StructuredLogger, order_id: &str, fill_price: f64, latency_ms: u64) {
    logger.info("Order filled", log_fields! {
        "order_id" => order_id,
        "fill_price" => format!("{:.2}", fill_price),
        "latency_ms" => latency_ms,
    });
}

fn log_error(logger: &StructuredLogger, error: &str, order_id: &str) {
    logger.error("Order error", log_fields! {
        "error" => error,
        "order_id" => order_id,
    });
}

fn main() {
    let logger = StructuredLogger::new(LogLevel::Info);

    println!("=== Structured Trading Logs ===\n");

    // Simulate trading activity
    log_order_placed(&logger, "ORD-001", "BTCUSDT", "BUY", 50000.0, 0.1);
    log_order_filled(&logger, "ORD-001", 50001.50, 45);

    log_order_placed(&logger, "ORD-002", "ETHUSDT", "SELL", 3000.0, 1.0);
    log_error(&logger, "Insufficient balance", "ORD-002");

    log_order_placed(&logger, "ORD-003", "BTCUSDT", "BUY", 49950.0, 0.05);
    log_order_filled(&logger, "ORD-003", 49951.00, 38);

    println!("\nTotal log entries: {}", logger.log_count());
}
```

## Alerting System

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: Instant,
}

/// Alerting rule
pub struct AlertRule {
    pub name: String,
    pub severity: AlertSeverity,
    pub condition: Box<dyn Fn() -> Option<String> + Send + Sync>,
    pub cooldown: Duration,
    pub last_fired: RwLock<Option<Instant>>,
}

impl AlertRule {
    pub fn new<F>(name: &str, severity: AlertSeverity, cooldown: Duration, condition: F) -> Self
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        AlertRule {
            name: name.to_string(),
            severity,
            condition: Box::new(condition),
            cooldown,
            last_fired: RwLock::new(None),
        }
    }

    pub fn check(&self) -> Option<Alert> {
        // Check cooldown
        if let Some(last) = *self.last_fired.read().unwrap() {
            if last.elapsed() < self.cooldown {
                return None;
            }
        }

        // Check condition
        if let Some(message) = (self.condition)() {
            *self.last_fired.write().unwrap() = Some(Instant::now());
            return Some(Alert {
                name: self.name.clone(),
                severity: self.severity,
                message,
                timestamp: Instant::now(),
            });
        }

        None
    }
}

/// Alert manager
pub struct AlertManager {
    rules: Vec<Arc<AlertRule>>,
    alerts_fired: AtomicU64,
    is_silenced: AtomicBool,
}

impl AlertManager {
    pub fn new() -> Self {
        AlertManager {
            rules: Vec::new(),
            alerts_fired: AtomicU64::new(0),
            is_silenced: AtomicBool::new(false),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(Arc::new(rule));
    }

    pub fn check_all(&self) -> Vec<Alert> {
        if self.is_silenced.load(Ordering::Relaxed) {
            return Vec::new();
        }

        let mut alerts = Vec::new();
        for rule in &self.rules {
            if let Some(alert) = rule.check() {
                self.alerts_fired.fetch_add(1, Ordering::Relaxed);
                self.notify(&alert);
                alerts.push(alert);
            }
        }
        alerts
    }

    fn notify(&self, alert: &Alert) {
        let severity_str = match alert.severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Critical => "CRITICAL",
        };

        println!("[ALERT][{}] {}: {}", severity_str, alert.name, alert.message);

        // In a real system, send to Slack, PagerDuty, etc.
    }

    pub fn silence(&self) {
        self.is_silenced.store(true, Ordering::Relaxed);
    }

    pub fn unsilence(&self) {
        self.is_silenced.store(false, Ordering::Relaxed);
    }
}

/// Trading metrics for alerting
struct TradingState {
    pnl_cents: AtomicU64,       // Offset P&L (+ 1_000_000_00 for negatives)
    position_size: AtomicU64,   // In hundredths
    consecutive_losses: AtomicU64,
    last_exchange_heartbeat: AtomicU64,
}

impl TradingState {
    fn new() -> Self {
        TradingState {
            pnl_cents: AtomicU64::new(1_000_000_00), // 0 P&L
            position_size: AtomicU64::new(0),
            consecutive_losses: AtomicU64::new(0),
            last_exchange_heartbeat: AtomicU64::new(0),
        }
    }

    fn set_pnl(&self, pnl: f64) {
        let cents = ((pnl + 1_000_000.0) * 100.0) as u64;
        self.pnl_cents.store(cents, Ordering::Relaxed);
    }

    fn get_pnl(&self) -> f64 {
        let cents = self.pnl_cents.load(Ordering::Relaxed);
        (cents as f64 / 100.0) - 1_000_000.0
    }

    fn set_position(&self, size: f64) {
        self.position_size.store((size * 100.0) as u64, Ordering::Relaxed);
    }

    fn get_position(&self) -> f64 {
        self.position_size.load(Ordering::Relaxed) as f64 / 100.0
    }

    fn add_loss(&self) {
        self.consecutive_losses.fetch_add(1, Ordering::Relaxed);
    }

    fn reset_losses(&self) {
        self.consecutive_losses.store(0, Ordering::Relaxed);
    }

    fn heartbeat(&self) {
        self.last_exchange_heartbeat.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
    }
}

fn main() {
    let state = Arc::new(TradingState::new());
    let mut alert_manager = AlertManager::new();

    // Rule: Large losses
    let state_clone = Arc::clone(&state);
    alert_manager.add_rule(AlertRule::new(
        "large_loss",
        AlertSeverity::Critical,
        Duration::from_secs(60),
        move || {
            let pnl = state_clone.get_pnl();
            if pnl < -5000.0 {
                Some(format!("Daily P&L: ${:.2}", pnl))
            } else {
                None
            }
        },
    ));

    // Rule: Consecutive losing trades
    let state_clone = Arc::clone(&state);
    alert_manager.add_rule(AlertRule::new(
        "consecutive_losses",
        AlertSeverity::Warning,
        Duration::from_secs(30),
        move || {
            let losses = state_clone.consecutive_losses.load(Ordering::Relaxed);
            if losses >= 5 {
                Some(format!("{} consecutive losing trades", losses))
            } else {
                None
            }
        },
    ));

    // Rule: Position too large
    let state_clone = Arc::clone(&state);
    alert_manager.add_rule(AlertRule::new(
        "large_position",
        AlertSeverity::Warning,
        Duration::from_secs(60),
        move || {
            let position = state_clone.get_position();
            if position > 10.0 {
                Some(format!("Position size: {:.2} BTC", position))
            } else {
                None
            }
        },
    ));

    println!("=== Alert System Demo ===\n");

    // Simulation: everything normal
    state.set_pnl(1500.0);
    state.set_position(2.5);
    state.heartbeat();
    println!("State: P&L=$1500, Position=2.5 BTC");
    let alerts = alert_manager.check_all();
    if alerts.is_empty() {
        println!("No alerts triggered\n");
    }

    // Simulation: series of losses
    println!("Simulating 5 consecutive losses...");
    for _ in 0..5 {
        state.add_loss();
    }
    alert_manager.check_all();

    // Simulation: large loss
    println!("\nSimulating large loss...");
    state.set_pnl(-6000.0);
    alert_manager.check_all();

    // Simulation: large position
    println!("\nSimulating large position...");
    state.set_position(15.0);
    alert_manager.check_all();
}
```

## Complete Monitoring System

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Comprehensive trading bot monitoring system
pub struct TradingMonitor {
    // Metrics
    orders_total: AtomicU64,
    orders_filled: AtomicU64,
    orders_rejected: AtomicU64,
    total_pnl_cents: AtomicU64,  // Offset for negative values

    // Health
    is_connected: AtomicBool,
    last_heartbeat: AtomicU64,

    // Performance statistics
    order_latency_sum_us: AtomicU64,
    order_latency_count: AtomicU64,

    // State
    start_time: Instant,
}

impl TradingMonitor {
    pub fn new() -> Self {
        TradingMonitor {
            orders_total: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            total_pnl_cents: AtomicU64::new(1_000_000_00),
            is_connected: AtomicBool::new(true),
            last_heartbeat: AtomicU64::new(Self::current_timestamp()),
            order_latency_sum_us: AtomicU64::new(0),
            order_latency_count: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn record_order(&self) {
        self.orders_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_fill(&self, latency: Duration, pnl: f64) {
        self.orders_filled.fetch_add(1, Ordering::Relaxed);

        let latency_us = latency.as_micros() as u64;
        self.order_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.order_latency_count.fetch_add(1, Ordering::Relaxed);

        // Update P&L
        let pnl_cents = (pnl * 100.0) as i64;
        if pnl_cents >= 0 {
            self.total_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.total_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }
    }

    pub fn record_rejection(&self) {
        self.orders_rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn heartbeat(&self) {
        self.last_heartbeat.store(Self::current_timestamp(), Ordering::Relaxed);
        self.is_connected.store(true, Ordering::Relaxed);
    }

    pub fn disconnect(&self) {
        self.is_connected.store(false, Ordering::Relaxed);
    }

    pub fn get_pnl(&self) -> f64 {
        let cents = self.total_pnl_cents.load(Ordering::Relaxed);
        (cents as f64 / 100.0) - 1_000_000.0
    }

    pub fn get_fill_rate(&self) -> f64 {
        let total = self.orders_total.load(Ordering::Relaxed);
        let filled = self.orders_filled.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        (filled as f64 / total as f64) * 100.0
    }

    pub fn get_avg_latency(&self) -> Option<Duration> {
        let count = self.order_latency_count.load(Ordering::Relaxed);
        if count == 0 {
            return None;
        }
        let sum = self.order_latency_sum_us.load(Ordering::Relaxed);
        Some(Duration::from_micros(sum / count))
    }

    pub fn is_healthy(&self) -> bool {
        let connected = self.is_connected.load(Ordering::Relaxed);
        let last_hb = self.last_heartbeat.load(Ordering::Relaxed);
        let now = Self::current_timestamp();

        connected && (now - last_hb) < 30
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Generate dashboard report
    pub fn dashboard_report(&self) -> String {
        let mut report = String::new();

        report.push_str("╔══════════════════════════════════════╗\n");
        report.push_str("║     TRADING BOT MONITOR DASHBOARD    ║\n");
        report.push_str("╠══════════════════════════════════════╣\n");

        // Status
        let status = if self.is_healthy() { "🟢 HEALTHY" } else { "🔴 UNHEALTHY" };
        report.push_str(&format!("║ Status: {:27} ║\n", status));

        // Uptime
        let uptime = self.uptime();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        report.push_str(&format!("║ Uptime: {:02}h {:02}m {:23} ║\n", hours, minutes, ""));

        report.push_str("╠══════════════════════════════════════╣\n");

        // Orders
        let total = self.orders_total.load(Ordering::Relaxed);
        let filled = self.orders_filled.load(Ordering::Relaxed);
        let rejected = self.orders_rejected.load(Ordering::Relaxed);
        report.push_str(&format!("║ Orders: {} total, {} filled, {} rej  ║\n",
            total, filled, rejected));
        report.push_str(&format!("║ Fill Rate: {:6.2}% {:19} ║\n",
            self.get_fill_rate(), ""));

        // Latency
        if let Some(latency) = self.get_avg_latency() {
            report.push_str(&format!("║ Avg Latency: {:6.2}ms {:15} ║\n",
                latency.as_secs_f64() * 1000.0, ""));
        }

        report.push_str("╠══════════════════════════════════════╣\n");

        // P&L
        let pnl = self.get_pnl();
        let pnl_indicator = if pnl >= 0.0 { "📈" } else { "📉" };
        report.push_str(&format!("║ {} P&L: ${:>10.2} {:15} ║\n",
            pnl_indicator, pnl, ""));

        report.push_str("╚══════════════════════════════════════╝\n");

        report
    }

    /// Export metrics in Prometheus format
    pub fn prometheus_export(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "# TYPE trading_orders_total counter\ntrading_orders_total {}\n",
            self.orders_total.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# TYPE trading_orders_filled counter\ntrading_orders_filled {}\n",
            self.orders_filled.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# TYPE trading_orders_rejected counter\ntrading_orders_rejected {}\n",
            self.orders_rejected.load(Ordering::Relaxed)
        ));

        output.push_str(&format!(
            "# TYPE trading_pnl_dollars gauge\ntrading_pnl_dollars {:.2}\n",
            self.get_pnl()
        ));

        output.push_str(&format!(
            "# TYPE trading_connected gauge\ntrading_connected {}\n",
            if self.is_connected.load(Ordering::Relaxed) { 1 } else { 0 }
        ));

        if let Some(latency) = self.get_avg_latency() {
            output.push_str(&format!(
                "# TYPE trading_order_latency_ms gauge\ntrading_order_latency_ms {:.2}\n",
                latency.as_secs_f64() * 1000.0
            ));
        }

        output.push_str(&format!(
            "# TYPE trading_uptime_seconds gauge\ntrading_uptime_seconds {}\n",
            self.uptime().as_secs()
        ));

        output
    }
}

fn main() {
    let monitor = Arc::new(TradingMonitor::new());

    println!("=== Trading Monitor Demo ===\n");

    // Simulate trading activity
    for i in 0..100 {
        monitor.record_order();
        monitor.heartbeat();

        if i % 10 == 9 {
            // 10% rejections
            monitor.record_rejection();
        } else {
            // 90% fills
            let latency = Duration::from_millis(30 + (i % 50) as u64);
            let pnl = if i % 3 == 0 { -10.0 } else { 15.0 };
            monitor.record_fill(latency, pnl);
        }
    }

    // Display dashboard
    println!("{}", monitor.dashboard_report());

    // Prometheus metrics
    println!("\n=== Prometheus Metrics ===\n");
    println!("{}", monitor.prometheus_export());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Counter** | Metric that only increases (orders, errors) |
| **Gauge** | Current value (balance, open positions) |
| **Histogram** | Distribution of values (order latencies) |
| **Health Check** | Verification of system component status |
| **Alerting** | Notifications when thresholds are violated |
| **Structured Logging** | JSON-formatted logs for automated processing |
| **Prometheus** | Standard for exporting metrics to monitoring systems |

## Practical Exercises

1. **WebSocket Connection Monitoring**: Create a system that:
   - Tracks connection status to multiple exchanges
   - Measures data reception latency
   - Automatically reconnects on connection loss
   - Alerts on prolonged disconnections

2. **Order Tracing**: Implement a system:
   - Tracks the complete order lifecycle
   - Measures time for each stage (creation → sending → confirmation → execution)
   - Identifies bottlenecks
   - Generates performance reports

3. **Trading Anomaly Detection**: Create a detector:
   - Identifies unusual patterns (many rejections, high latency)
   - Compares current metrics with historical data
   - Automatically reduces activity during anomalies
   - Logs all detected issues

4. **Real-time Dashboard**: Implement:
   - Web server with real-time metrics
   - Charts for P&L, orders, latencies
   - Grafana integration via Prometheus
   - Alerts to Telegram/Slack

## Homework

1. **Complete Monitoring System**: Develop a system:
   - Collects all trading bot metrics
   - Exports to Prometheus
   - Has Health endpoint for Kubernetes
   - Supports graceful shutdown
   - Stores metrics history

2. **Intelligent Alerting**: Create a system:
   - Automatically determines baseline metrics
   - Adapts thresholds to time of day and volatility
   - Groups similar alerts
   - Escalates critical issues
   - Maintains incident history

3. **Distributed Tracing**: Implement:
   - Request tracking across all components
   - Correlation ID for linking events
   - Data flow visualization
   - Component dependency analysis
   - Bottleneck identification

4. **Chaos Engineering for Trading**: Create a tool:
   - Simulates failures (exchange disconnect, network delays)
   - Tests system behavior during failures
   - Validates alerts and recovery
   - Generates reliability reports
   - Recommends improvements

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../354-*/en.md)
