# День 353: Мониторинг в продакшене

## Аналогия из трейдинга

Представь, что у тебя есть торговый робот, который работает 24/7 на реальном рынке. Он как опытный трейдер, который должен постоянно следить за:

**Здоровье системы = Здоровье трейдера:**
- Пульс (heartbeat) — система работает и отвечает
- Температура (CPU/Memory) — нагрузка в норме
- Давление (latency) — задержки в пределах допустимого

**Торговые метрики = Показатели эффективности:**
- Количество сделок в минуту
- Средняя прибыль на сделку
- Время исполнения ордеров
- Проскальзывание (slippage)

**Алерты = Сигналы тревоги:**
- Потеря соединения с биржей — немедленно оповестить
- Аномальные потери — остановить торговлю
- Исчерпание памяти — предупредить заранее

| Аспект | Без мониторинга | С мониторингом |
|--------|-----------------|----------------|
| **Обнаружение проблем** | Когда деньги потеряны | Заранее |
| **Время реакции** | Часы/дни | Секунды/минуты |
| **Диагностика** | Гадание | Точные данные |
| **Оптимизация** | Вслепую | На основе метрик |

## Основы мониторинга в Rust

### Типы метрик

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Типы метрик для торговой системы
#[derive(Debug)]
pub struct TradingMetrics {
    // Счётчики (Counter) — только увеличиваются
    pub orders_placed: AtomicU64,
    pub orders_filled: AtomicU64,
    pub orders_cancelled: AtomicU64,
    pub orders_rejected: AtomicU64,

    // Измерители (Gauge) — текущее значение
    pub open_positions: AtomicUsize,
    pub active_orders: AtomicUsize,
    pub available_balance_cents: AtomicU64,

    // Гистограммы (Histogram) — распределение значений
    // Для упрощения храним сумму и количество
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

    // Симуляция торговой активности
    for i in 0..100 {
        metrics.record_order_placed();

        // 90% ордеров исполняются
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

### Health Checks (Проверки здоровья)

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// Статус компонента
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Результат проверки здоровья
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub last_check: u64,  // Unix timestamp
    pub latency_ms: u64,
}

/// Менеджер проверок здоровья
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

/// Проверка подключения к бирже
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

/// Проверка памяти
fn memory_health_check() -> HealthCheck {
    let start = Instant::now();

    // В реальном коде используйте sys-info или procfs
    // Здесь симуляция
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

    // Регистрируем проверки
    health_manager.register("exchange", move || exchange_clone.health_check());
    health_manager.register("memory", memory_health_check);

    // Проверяем здоровье системы
    println!("=== Health Check (все хорошо) ===");
    for check in health_manager.check_all() {
        println!("{}: {:?} ({}ms)", check.name, check.status, check.latency_ms);
    }
    println!("System healthy: {}\n", health_manager.is_healthy());

    // Симуляция проблемы
    exchange.disconnect();

    println!("=== Health Check (после отключения биржи) ===");
    for check in health_manager.check_all() {
        println!("{}: {:?} ({}ms)", check.name, check.status, check.latency_ms);
    }
    println!("System healthy: {}", health_manager.is_healthy());
}
```

## Интеграция с Prometheus

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Формат метрик совместимый с Prometheus
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
        // Безопасно: мы только добавляем, не удаляем
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

    /// Экспорт в формате Prometheus
    pub fn export(&self) -> String {
        let mut output = String::new();

        // Экспорт счётчиков
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

        // Экспорт gauges
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

/// Метрики торговой системы для Prometheus
struct TradingPrometheusMetrics {
    registry: Arc<PrometheusRegistry>,
}

impl TradingPrometheusMetrics {
    fn new() -> Self {
        let registry = Arc::new(PrometheusRegistry::new());

        // Инициализация метрик с лейблами
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
        // Храним в сотых долях для точности
        self.registry.set_gauge("trading_position_size", (size * 100.0) as u64);
    }

    fn set_pnl(&self, pnl: f64) {
        // Храним P&L в центах (может быть отрицательным, поэтому добавляем offset)
        let offset_pnl = ((pnl + 1_000_000.0) * 100.0) as u64;
        self.registry.set_gauge("trading_pnl_cents", offset_pnl);
    }

    fn export(&self) -> String {
        self.registry.export()
    }
}

fn main() {
    let metrics = TradingPrometheusMetrics::new();

    // Симуляция торговли
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

## Логирование для продакшена

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Уровни логирования
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Структурированная запись лога
#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Простой структурированный логгер
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

        // Вывод в JSON формате для обработки
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

/// Макрос для удобного создания полей
macro_rules! log_fields {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

/// Логирование торговых событий
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

    // Симуляция торговой активности
    log_order_placed(&logger, "ORD-001", "BTCUSDT", "BUY", 50000.0, 0.1);
    log_order_filled(&logger, "ORD-001", 50001.50, 45);

    log_order_placed(&logger, "ORD-002", "ETHUSDT", "SELL", 3000.0, 1.0);
    log_error(&logger, "Insufficient balance", "ORD-002");

    log_order_placed(&logger, "ORD-003", "BTCUSDT", "BUY", 49950.0, 0.05);
    log_order_filled(&logger, "ORD-003", 49951.00, 38);

    println!("\nTotal log entries: {}", logger.log_count());
}
```

## Алертинг (Оповещения)

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Уровень важности алерта
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Алерт
#[derive(Debug, Clone)]
pub struct Alert {
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: Instant,
}

/// Правило алертинга
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
        // Проверяем cooldown
        if let Some(last) = *self.last_fired.read().unwrap() {
            if last.elapsed() < self.cooldown {
                return None;
            }
        }

        // Проверяем условие
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

/// Менеджер алертов
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

        // В реальной системе здесь отправка в Slack, PagerDuty, и т.д.
    }

    pub fn silence(&self) {
        self.is_silenced.store(true, Ordering::Relaxed);
    }

    pub fn unsilence(&self) {
        self.is_silenced.store(false, Ordering::Relaxed);
    }
}

/// Торговые метрики для алертинга
struct TradingState {
    pnl_cents: AtomicU64,       // Смещённый P&L (+ 1_000_000_00 для отрицательных)
    position_size: AtomicU64,   // В сотых долях
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

    // Правило: Большие убытки
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

    // Правило: Серия убыточных сделок
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

    // Правило: Слишком большая позиция
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

    // Симуляция: всё нормально
    state.set_pnl(1500.0);
    state.set_position(2.5);
    state.heartbeat();
    println!("State: P&L=$1500, Position=2.5 BTC");
    let alerts = alert_manager.check_all();
    if alerts.is_empty() {
        println!("No alerts triggered\n");
    }

    // Симуляция: серия убытков
    println!("Simulating 5 consecutive losses...");
    for _ in 0..5 {
        state.add_loss();
    }
    alert_manager.check_all();

    // Симуляция: большой убыток
    println!("\nSimulating large loss...");
    state.set_pnl(-6000.0);
    alert_manager.check_all();

    // Симуляция: большая позиция
    println!("\nSimulating large position...");
    state.set_position(15.0);
    alert_manager.check_all();
}
```

## Полная система мониторинга

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Комплексная система мониторинга торгового бота
pub struct TradingMonitor {
    // Метрики
    orders_total: AtomicU64,
    orders_filled: AtomicU64,
    orders_rejected: AtomicU64,
    total_pnl_cents: AtomicU64,  // Смещённый для отрицательных

    // Здоровье
    is_connected: AtomicBool,
    last_heartbeat: AtomicU64,

    // Статистика производительности
    order_latency_sum_us: AtomicU64,
    order_latency_count: AtomicU64,

    // Состояние
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

        // Обновляем P&L
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

    /// Генерация отчёта для дашборда
    pub fn dashboard_report(&self) -> String {
        let mut report = String::new();

        report.push_str("╔══════════════════════════════════════╗\n");
        report.push_str("║     TRADING BOT MONITOR DASHBOARD    ║\n");
        report.push_str("╠══════════════════════════════════════╣\n");

        // Статус
        let status = if self.is_healthy() { "🟢 HEALTHY" } else { "🔴 UNHEALTHY" };
        report.push_str(&format!("║ Status: {:27} ║\n", status));

        // Время работы
        let uptime = self.uptime();
        let hours = uptime.as_secs() / 3600;
        let minutes = (uptime.as_secs() % 3600) / 60;
        report.push_str(&format!("║ Uptime: {:02}h {:02}m {:23} ║\n", hours, minutes, ""));

        report.push_str("╠══════════════════════════════════════╣\n");

        // Ордера
        let total = self.orders_total.load(Ordering::Relaxed);
        let filled = self.orders_filled.load(Ordering::Relaxed);
        let rejected = self.orders_rejected.load(Ordering::Relaxed);
        report.push_str(&format!("║ Orders: {} total, {} filled, {} rej  ║\n",
            total, filled, rejected));
        report.push_str(&format!("║ Fill Rate: {:6.2}% {:19} ║\n",
            self.get_fill_rate(), ""));

        // Задержка
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

    /// Экспорт метрик в формате Prometheus
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

    // Симуляция торговой активности
    for i in 0..100 {
        monitor.record_order();
        monitor.heartbeat();

        if i % 10 == 9 {
            // 10% отклонений
            monitor.record_rejection();
        } else {
            // 90% исполнений
            let latency = Duration::from_millis(30 + (i % 50) as u64);
            let pnl = if i % 3 == 0 { -10.0 } else { 15.0 };
            monitor.record_fill(latency, pnl);
        }
    }

    // Отображение дашборда
    println!("{}", monitor.dashboard_report());

    // Prometheus метрики
    println!("\n=== Prometheus Metrics ===\n");
    println!("{}", monitor.prometheus_export());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Counter** | Счётчик, который только увеличивается (ордера, ошибки) |
| **Gauge** | Текущее значение (баланс, открытые позиции) |
| **Histogram** | Распределение значений (задержки ордеров) |
| **Health Check** | Проверка состояния компонентов системы |
| **Alerting** | Уведомления при нарушении пороговых значений |
| **Structured Logging** | Логи в формате JSON для автоматической обработки |
| **Prometheus** | Стандарт экспорта метрик для систем мониторинга |

## Практические задания

1. **Мониторинг WebSocket соединений**: Создай систему, которая:
   - Отслеживает состояние подключений к нескольким биржам
   - Измеряет задержку получения данных
   - Автоматически переподключается при потере связи
   - Алертит при длительных отключениях

2. **Трейсинг ордеров**: Реализуй систему:
   - Отслеживает полный жизненный цикл ордера
   - Измеряет время каждого этапа (создание → отправка → подтверждение → исполнение)
   - Выявляет узкие места
   - Генерирует отчёты по производительности

3. **Аномалии в торговле**: Создай детектор:
   - Определяет необычные паттерны (много отклонений, высокие задержки)
   - Сравнивает текущие метрики с историческими
   - Автоматически снижает активность при аномалиях
   - Логирует все обнаруженные проблемы

4. **Дашборд реального времени**: Реализуй:
   - Веб-сервер с метриками в реальном времени
   - Графики P&L, ордеров, задержек
   - Интеграцию с Grafana через Prometheus
   - Алерты в Telegram/Slack

## Домашнее задание

1. **Полная система мониторинга**: Разработай систему:
   - Собирает все метрики торгового бота
   - Экспортирует в Prometheus
   - Имеет Health endpoint для Kubernetes
   - Поддерживает graceful shutdown
   - Сохраняет историю метрик

2. **Интеллектуальный алертинг**: Создай систему:
   - Определяет базовые показатели автоматически
   - Адаптирует пороги к времени суток и волатильности
   - Группирует похожие алерты
   - Эскалирует критические проблемы
   - Ведёт историю инцидентов

3. **Распределённая трассировка**: Реализуй:
   - Отслеживание запросов через все компоненты
   - Correlation ID для связи событий
   - Визуализацию потока данных
   - Анализ зависимостей компонентов
   - Поиск bottlenecks

4. **Chaos Engineering для трейдинга**: Создай инструмент:
   - Симулирует сбои (отключение биржи, задержки сети)
   - Проверяет поведение системы при сбоях
   - Валидирует алерты и восстановление
   - Генерирует отчёты о надёжности
   - Рекомендует улучшения

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../354-*/ru.md)
