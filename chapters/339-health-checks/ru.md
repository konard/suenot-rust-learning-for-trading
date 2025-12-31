# День 339: Health Checks: проверка здоровья

## Аналогия из трейдинга

Представь, что ты управляешь торговым центром с несколькими филиалами. Каждое утро ты отправляешь инспектора проверить каждый филиал:

- **Открыт ли магазин?** — базовая проверка доступности
- **Работает ли касса?** — проверка критических систем
- **Есть ли товар на складе?** — проверка зависимостей
- **Работает ли интернет для терминалов?** — проверка внешних подключений

В торговых системах Health Checks работают так же:

| Аналогия из торговли | Health Check компонент |
|---------------------|------------------------|
| Магазин открыт | Сервис запущен и отвечает |
| Касса работает | База данных доступна |
| Товар на складе | Кэш данных актуален |
| Интернет работает | Биржа API доступна |
| Персонал на месте | Воркеры обрабатывают задачи |

## Зачем нужны Health Checks?

В production-системах Health Checks критически важны для:

1. **Load Balancer** — направляет трафик только на здоровые инстансы
2. **Kubernetes** — решает, когда перезапустить pod
3. **Мониторинг** — оповещает о проблемах до их эскалации
4. **CI/CD** — проверяет, что деплой прошёл успешно

## Типы Health Checks

### 1. Liveness Probe — "Жив ли сервис?"

Простейшая проверка: отвечает ли сервис вообще.

```rust
use axum::{routing::get, Router, Json};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct LivenessResponse {
    status: String,
    timestamp: u64,
}

/// Liveness endpoint — просто подтверждает, что сервис запущен
async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "alive".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health/live", get(liveness));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Сервис запущен на {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 2. Readiness Probe — "Готов ли принимать запросы?"

Проверяет, что все зависимости работают.

```rust
use axum::{routing::get, Router, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Состояние здоровья торговой системы
#[derive(Clone)]
struct TradingSystemHealth {
    database_connected: bool,
    exchange_api_available: bool,
    market_data_fresh: bool,
    order_queue_healthy: bool,
}

impl TradingSystemHealth {
    fn is_ready(&self) -> bool {
        self.database_connected
            && self.exchange_api_available
            && self.market_data_fresh
            && self.order_queue_healthy
    }
}

#[derive(Serialize)]
struct ReadinessResponse {
    ready: bool,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
struct ReadinessChecks {
    database: CheckResult,
    exchange_api: CheckResult,
    market_data: CheckResult,
    order_queue: CheckResult,
}

#[derive(Serialize)]
struct CheckResult {
    healthy: bool,
    message: String,
}

async fn readiness(
    health: Arc<RwLock<TradingSystemHealth>>,
) -> impl IntoResponse {
    let state = health.read().await;

    let response = ReadinessResponse {
        ready: state.is_ready(),
        checks: ReadinessChecks {
            database: CheckResult {
                healthy: state.database_connected,
                message: if state.database_connected {
                    "Connected".to_string()
                } else {
                    "Connection failed".to_string()
                },
            },
            exchange_api: CheckResult {
                healthy: state.exchange_api_available,
                message: if state.exchange_api_available {
                    "API responding".to_string()
                } else {
                    "API timeout".to_string()
                },
            },
            market_data: CheckResult {
                healthy: state.market_data_fresh,
                message: if state.market_data_fresh {
                    "Data fresh (<5s)".to_string()
                } else {
                    "Stale data (>30s)".to_string()
                },
            },
            order_queue: CheckResult {
                healthy: state.order_queue_healthy,
                message: if state.order_queue_healthy {
                    "Queue processing".to_string()
                } else {
                    "Queue blocked".to_string()
                },
            },
        },
    };

    let status = if state.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

#[tokio::main]
async fn main() {
    // Инициализация состояния здоровья
    let health = Arc::new(RwLock::new(TradingSystemHealth {
        database_connected: true,
        exchange_api_available: true,
        market_data_fresh: true,
        order_queue_healthy: true,
    }));

    let health_for_route = Arc::clone(&health);

    let app = Router::new()
        .route("/health/ready", get(move || {
            let h = Arc::clone(&health_for_route);
            async move { readiness(h).await }
        }));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Readiness endpoint: http://{}/health/ready", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 3. Startup Probe — "Завершилась ли инициализация?"

Особенно важна для торговых систем, которым нужно загрузить исторические данные.

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Состояние запуска торговой системы
struct StartupState {
    config_loaded: AtomicBool,
    historical_data_loaded: AtomicBool,
    strategies_initialized: AtomicBool,
    exchange_connections_established: AtomicBool,
    ready_for_trading: AtomicBool,
}

impl StartupState {
    fn new() -> Self {
        StartupState {
            config_loaded: AtomicBool::new(false),
            historical_data_loaded: AtomicBool::new(false),
            strategies_initialized: AtomicBool::new(false),
            exchange_connections_established: AtomicBool::new(false),
            ready_for_trading: AtomicBool::new(false),
        }
    }

    fn is_started(&self) -> bool {
        self.config_loaded.load(Ordering::Relaxed)
            && self.historical_data_loaded.load(Ordering::Relaxed)
            && self.strategies_initialized.load(Ordering::Relaxed)
            && self.exchange_connections_established.load(Ordering::Relaxed)
    }

    fn startup_progress(&self) -> StartupProgress {
        StartupProgress {
            config_loaded: self.config_loaded.load(Ordering::Relaxed),
            historical_data_loaded: self.historical_data_loaded.load(Ordering::Relaxed),
            strategies_initialized: self.strategies_initialized.load(Ordering::Relaxed),
            exchange_connections: self.exchange_connections_established.load(Ordering::Relaxed),
            ready_for_trading: self.ready_for_trading.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct StartupProgress {
    config_loaded: bool,
    historical_data_loaded: bool,
    strategies_initialized: bool,
    exchange_connections: bool,
    ready_for_trading: bool,
}

/// Симуляция запуска торговой системы
async fn simulate_startup(state: Arc<StartupState>) {
    println!("=== Запуск торговой системы ===\n");

    // Этап 1: Загрузка конфигурации
    println!("[1/5] Загрузка конфигурации...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    state.config_loaded.store(true, Ordering::Relaxed);
    println!("      Конфигурация загружена");

    // Этап 2: Загрузка исторических данных
    println!("[2/5] Загрузка исторических данных...");
    for i in 1..=5 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("      Загружено {} из 5 инструментов", i);
    }
    state.historical_data_loaded.store(true, Ordering::Relaxed);
    println!("      Исторические данные загружены");

    // Этап 3: Инициализация стратегий
    println!("[3/5] Инициализация торговых стратегий...");
    tokio::time::sleep(Duration::from_millis(400)).await;
    state.strategies_initialized.store(true, Ordering::Relaxed);
    println!("      Стратегии инициализированы");

    // Этап 4: Подключение к биржам
    println!("[4/5] Установка подключений к биржам...");
    tokio::time::sleep(Duration::from_millis(600)).await;
    state.exchange_connections_established.store(true, Ordering::Relaxed);
    println!("      Подключения установлены");

    // Этап 5: Готовность к торговле
    println!("[5/5] Финальная проверка...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    state.ready_for_trading.store(true, Ordering::Relaxed);
    println!("      Система готова к торговле!");

    println!("\n=== Система полностью запущена ===");
}

#[tokio::main]
async fn main() {
    let state = Arc::new(StartupState::new());
    let state_clone = Arc::clone(&state);

    // Запускаем процесс инициализации
    let startup_handle = tokio::spawn(simulate_startup(state_clone));

    // Мониторим прогресс запуска
    let state_monitor = Arc::clone(&state);
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let progress = state_monitor.startup_progress();

            if state_monitor.is_started() {
                println!("\n[Health Check] Startup complete: {:?}", progress);
                break;
            } else {
                println!("[Health Check] Startup in progress: {:?}", progress);
            }
        }
    });

    startup_handle.await.unwrap();
    monitor_handle.await.unwrap();
}
```

## Комплексная система Health Checks для торгового бота

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Статус проверки здоровья
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Результат проверки компонента
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub latency_ms: u64,
    pub last_check: u64,
}

/// Общий отчёт о здоровье системы
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Проверка здоровья компонента
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> ComponentHealth;
}

/// Проверка подключения к базе данных
pub struct DatabaseHealthCheck {
    connection_string: String,
}

impl DatabaseHealthCheck {
    pub fn new(connection_string: &str) -> Self {
        DatabaseHealthCheck {
            connection_string: connection_string.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Симуляция проверки подключения к БД
        tokio::time::sleep(Duration::from_millis(10)).await;

        let latency = start.elapsed().as_millis() as u64;
        let (status, message) = if latency < 100 {
            (HealthStatus::Healthy, "Connection OK".to_string())
        } else if latency < 500 {
            (HealthStatus::Degraded, format!("Slow response: {}ms", latency))
        } else {
            (HealthStatus::Unhealthy, "Connection timeout".to_string())
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: latency,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Проверка подключения к бирже
pub struct ExchangeHealthCheck {
    exchange_name: String,
    api_endpoint: String,
}

impl ExchangeHealthCheck {
    pub fn new(exchange_name: &str, api_endpoint: &str) -> Self {
        ExchangeHealthCheck {
            exchange_name: exchange_name.to_string(),
            api_endpoint: api_endpoint.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ExchangeHealthCheck {
    fn name(&self) -> &str {
        &self.exchange_name
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Симуляция запроса к API биржи
        tokio::time::sleep(Duration::from_millis(50)).await;

        let latency = start.elapsed().as_millis() as u64;

        ComponentHealth {
            name: format!("exchange_{}", self.exchange_name),
            status: HealthStatus::Healthy,
            message: format!("API {} responding", self.api_endpoint),
            latency_ms: latency,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Проверка актуальности рыночных данных
pub struct MarketDataHealthCheck {
    max_staleness_seconds: u64,
    last_update: Arc<RwLock<Instant>>,
}

impl MarketDataHealthCheck {
    pub fn new(max_staleness_seconds: u64) -> Self {
        MarketDataHealthCheck {
            max_staleness_seconds,
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_timestamp(&self) {
        let mut last = self.last_update.write().await;
        *last = Instant::now();
    }
}

#[async_trait::async_trait]
impl HealthCheck for MarketDataHealthCheck {
    fn name(&self) -> &str {
        "market_data"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let last_update = self.last_update.read().await;
        let staleness = last_update.elapsed().as_secs();

        let (status, message) = if staleness < self.max_staleness_seconds {
            (HealthStatus::Healthy, format!("Data fresh ({}s ago)", staleness))
        } else if staleness < self.max_staleness_seconds * 2 {
            (HealthStatus::Degraded, format!("Data aging ({}s ago)", staleness))
        } else {
            (HealthStatus::Unhealthy, format!("Data stale ({}s ago)", staleness))
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Проверка очереди ордеров
pub struct OrderQueueHealthCheck {
    queue_size: Arc<RwLock<usize>>,
    max_queue_size: usize,
}

impl OrderQueueHealthCheck {
    pub fn new(max_queue_size: usize) -> Self {
        OrderQueueHealthCheck {
            queue_size: Arc::new(RwLock::new(0)),
            max_queue_size,
        }
    }

    pub async fn set_queue_size(&self, size: usize) {
        let mut q = self.queue_size.write().await;
        *q = size;
    }
}

#[async_trait::async_trait]
impl HealthCheck for OrderQueueHealthCheck {
    fn name(&self) -> &str {
        "order_queue"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let queue_size = *self.queue_size.read().await;
        let queue_percent = (queue_size as f64 / self.max_queue_size as f64) * 100.0;

        let (status, message) = if queue_percent < 50.0 {
            (HealthStatus::Healthy, format!("Queue OK ({}/{})", queue_size, self.max_queue_size))
        } else if queue_percent < 80.0 {
            (HealthStatus::Degraded, format!("Queue filling ({:.0}%)", queue_percent))
        } else {
            (HealthStatus::Unhealthy, format!("Queue critical ({:.0}%)", queue_percent))
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Менеджер проверок здоровья
pub struct HealthCheckManager {
    checks: Vec<Box<dyn HealthCheck>>,
    start_time: Instant,
    version: String,
}

impl HealthCheckManager {
    pub fn new(version: &str) -> Self {
        HealthCheckManager {
            checks: Vec::new(),
            start_time: Instant::now(),
            version: version.to_string(),
        }
    }

    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub async fn run_all_checks(&self) -> HealthReport {
        let mut components = Vec::new();
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for check in &self.checks {
            let result = check.check().await;
            match result.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
            components.push(result);
        }

        let overall_status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthReport {
            overall_status,
            components,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: self.version.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Health Check система для торгового бота ===\n");

    // Создаём менеджер проверок
    let mut manager = HealthCheckManager::new("1.0.0");

    // Добавляем проверки
    manager.add_check(Box::new(DatabaseHealthCheck::new("postgres://localhost/trading")));
    manager.add_check(Box::new(ExchangeHealthCheck::new("binance", "api.binance.com")));
    manager.add_check(Box::new(ExchangeHealthCheck::new("kraken", "api.kraken.com")));

    let market_data_check = Arc::new(MarketDataHealthCheck::new(5));
    manager.add_check(Box::new(MarketDataHealthCheck::new(5)));

    let order_queue_check = Arc::new(OrderQueueHealthCheck::new(1000));
    manager.add_check(Box::new(OrderQueueHealthCheck::new(1000)));

    // Запускаем проверку
    let report = manager.run_all_checks().await;

    println!("Overall Status: {:?}", report.overall_status);
    println!("Uptime: {}s", report.uptime_seconds);
    println!("Version: {}\n", report.version);

    println!("Components:");
    for component in &report.components {
        println!(
            "  {} [{:?}]: {} ({}ms)",
            component.name, component.status, component.message, component.latency_ms
        );
    }

    // Вывод в JSON формате
    println!("\n=== JSON Response ===");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
```

## Интеграция с Kubernetes

```rust
use axum::{routing::get, Router, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Конфигурация для Kubernetes probes
#[derive(Clone)]
pub struct K8sProbeConfig {
    /// Время на запуск (для startup probe)
    pub startup_timeout_seconds: u64,
    /// Порог неудачных проверок liveness
    pub liveness_failure_threshold: u32,
    /// Порог неудачных проверок readiness
    pub readiness_failure_threshold: u32,
}

/// Состояние приложения для K8s
pub struct AppState {
    pub started: bool,
    pub ready: bool,
    pub live: bool,
    pub consecutive_failures: u32,
}

#[derive(Serialize)]
struct ProbeResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

/// Startup probe — проверяет, завершилась ли инициализация
async fn startup_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.started {
        (StatusCode::OK, Json(ProbeResponse {
            status: "started".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(ProbeResponse {
            status: "starting".to_string(),
            details: Some("Initialization in progress".to_string()),
        }))
    }
}

/// Liveness probe — проверяет, не завис ли процесс
async fn liveness_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.live {
        (StatusCode::OK, Json(ProbeResponse {
            status: "alive".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ProbeResponse {
            status: "dead".to_string(),
            details: Some("Process unresponsive".to_string()),
        }))
    }
}

/// Readiness probe — проверяет готовность принимать трафик
async fn readiness_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.ready {
        (StatusCode::OK, Json(ProbeResponse {
            status: "ready".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(ProbeResponse {
            status: "not_ready".to_string(),
            details: Some("Dependencies unavailable".to_string()),
        }))
    }
}

/*
Пример манифеста Kubernetes:

apiVersion: v1
kind: Pod
metadata:
  name: trading-bot
spec:
  containers:
  - name: trading-bot
    image: trading-bot:latest
    ports:
    - containerPort: 8080

    # Startup probe — даёт время на инициализацию
    startupProbe:
      httpGet:
        path: /health/startup
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
      failureThreshold: 30  # 30 * 5 = 150 секунд на запуск

    # Liveness probe — перезапускает зависший pod
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      periodSeconds: 10
      failureThreshold: 3

    # Readiness probe — убирает pod из Service
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      periodSeconds: 5
      failureThreshold: 2
*/

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState {
        started: false,
        ready: false,
        live: true,
        consecutive_failures: 0,
    }));

    let state_startup = Arc::clone(&state);
    let state_liveness = Arc::clone(&state);
    let state_readiness = Arc::clone(&state);

    let app = Router::new()
        .route("/health/startup", get(move || {
            let s = Arc::clone(&state_startup);
            async move { startup_probe(s).await }
        }))
        .route("/health/live", get(move || {
            let s = Arc::clone(&state_liveness);
            async move { liveness_probe(s).await }
        }))
        .route("/health/ready", get(move || {
            let s = Arc::clone(&state_readiness);
            async move { readiness_probe(s).await }
        }));

    // Симуляция процесса запуска
    let state_init = Arc::clone(&state);
    tokio::spawn(async move {
        println!("Начало инициализации...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        {
            let mut app = state_init.write().await;
            app.started = true;
            println!("Инициализация завершена");
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        {
            let mut app = state_init.write().await;
            app.ready = true;
            println!("Готов принимать трафик");
        }
    });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("K8s probes доступны:");
    println!("  Startup:   http://{}/health/startup", addr);
    println!("  Liveness:  http://{}/health/live", addr);
    println!("  Readiness: http://{}/health/ready", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Deep Health Check для критических систем

```rust
use std::time::{Duration, Instant};
use serde::Serialize;

/// Глубокая проверка торговой системы
#[derive(Debug, Serialize)]
pub struct DeepHealthCheck {
    pub timestamp: u64,
    pub overall_healthy: bool,
    pub checks: DeepChecks,
    pub performance: PerformanceMetrics,
    pub trading_status: TradingStatus,
}

#[derive(Debug, Serialize)]
pub struct DeepChecks {
    pub database_write_test: CheckDetail,
    pub database_read_test: CheckDetail,
    pub exchange_order_test: CheckDetail,
    pub market_data_freshness: CheckDetail,
    pub strategy_execution: CheckDetail,
    pub risk_limits: CheckDetail,
}

#[derive(Debug, Serialize)]
pub struct CheckDetail {
    pub passed: bool,
    pub latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub avg_order_latency_ms: f64,
    pub orders_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct TradingStatus {
    pub trading_enabled: bool,
    pub active_positions: u32,
    pub pending_orders: u32,
    pub daily_pnl: f64,
    pub risk_utilization_percent: f64,
}

impl DeepHealthCheck {
    pub async fn run() -> Self {
        let start = Instant::now();

        // Проверка записи в БД
        let db_write = Self::check_database_write().await;

        // Проверка чтения из БД
        let db_read = Self::check_database_read().await;

        // Проверка API биржи (тестовый ордер)
        let exchange_test = Self::check_exchange_api().await;

        // Проверка актуальности данных
        let market_data = Self::check_market_data().await;

        // Проверка выполнения стратегии
        let strategy = Self::check_strategy_execution().await;

        // Проверка риск-лимитов
        let risk = Self::check_risk_limits().await;

        let all_passed = db_write.passed
            && db_read.passed
            && exchange_test.passed
            && market_data.passed
            && strategy.passed
            && risk.passed;

        DeepHealthCheck {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            overall_healthy: all_passed,
            checks: DeepChecks {
                database_write_test: db_write,
                database_read_test: db_read,
                exchange_order_test: exchange_test,
                market_data_freshness: market_data,
                strategy_execution: strategy,
                risk_limits: risk,
            },
            performance: Self::collect_performance_metrics(),
            trading_status: Self::get_trading_status(),
        }
    }

    async fn check_database_write() -> CheckDetail {
        let start = Instant::now();
        // Симуляция записи тестовой строки
        tokio::time::sleep(Duration::from_millis(5)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Write test passed".to_string(),
        }
    }

    async fn check_database_read() -> CheckDetail {
        let start = Instant::now();
        // Симуляция чтения тестовой строки
        tokio::time::sleep(Duration::from_millis(3)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Read test passed".to_string(),
        }
    }

    async fn check_exchange_api() -> CheckDetail {
        let start = Instant::now();
        // Симуляция проверки API (получение баланса)
        tokio::time::sleep(Duration::from_millis(50)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "API responding, balance retrieved".to_string(),
        }
    }

    async fn check_market_data() -> CheckDetail {
        let start = Instant::now();
        // Симуляция проверки актуальности данных
        let staleness_seconds = 2;

        CheckDetail {
            passed: staleness_seconds < 5,
            latency_ms: start.elapsed().as_millis() as u64,
            message: format!("Data age: {}s", staleness_seconds),
        }
    }

    async fn check_strategy_execution() -> CheckDetail {
        let start = Instant::now();
        // Симуляция проверки выполнения стратегии
        tokio::time::sleep(Duration::from_millis(10)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Strategy engine responsive".to_string(),
        }
    }

    async fn check_risk_limits() -> CheckDetail {
        let start = Instant::now();
        let risk_percent = 45.0;

        CheckDetail {
            passed: risk_percent < 80.0,
            latency_ms: start.elapsed().as_millis() as u64,
            message: format!("Risk utilization: {:.1}%", risk_percent),
        }
    }

    fn collect_performance_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            avg_order_latency_ms: 45.5,
            orders_per_second: 150.0,
            memory_usage_mb: 512.0,
            cpu_usage_percent: 35.0,
        }
    }

    fn get_trading_status() -> TradingStatus {
        TradingStatus {
            trading_enabled: true,
            active_positions: 5,
            pending_orders: 12,
            daily_pnl: 1250.50,
            risk_utilization_percent: 45.0,
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Deep Health Check для торговой системы ===\n");

    let report = DeepHealthCheck::run().await;

    println!("Overall Healthy: {}", report.overall_healthy);
    println!("\nChecks:");
    println!("  Database Write: {} ({} ms)",
        if report.checks.database_write_test.passed { "PASS" } else { "FAIL" },
        report.checks.database_write_test.latency_ms);
    println!("  Database Read: {} ({} ms)",
        if report.checks.database_read_test.passed { "PASS" } else { "FAIL" },
        report.checks.database_read_test.latency_ms);
    println!("  Exchange API: {} ({} ms)",
        if report.checks.exchange_order_test.passed { "PASS" } else { "FAIL" },
        report.checks.exchange_order_test.latency_ms);
    println!("  Market Data: {} - {}",
        if report.checks.market_data_freshness.passed { "PASS" } else { "FAIL" },
        report.checks.market_data_freshness.message);
    println!("  Strategy: {} ({} ms)",
        if report.checks.strategy_execution.passed { "PASS" } else { "FAIL" },
        report.checks.strategy_execution.latency_ms);
    println!("  Risk Limits: {} - {}",
        if report.checks.risk_limits.passed { "PASS" } else { "FAIL" },
        report.checks.risk_limits.message);

    println!("\nPerformance:");
    println!("  Avg Order Latency: {:.1} ms", report.performance.avg_order_latency_ms);
    println!("  Orders/sec: {:.0}", report.performance.orders_per_second);
    println!("  Memory: {:.0} MB", report.performance.memory_usage_mb);
    println!("  CPU: {:.0}%", report.performance.cpu_usage_percent);

    println!("\nTrading Status:");
    println!("  Trading Enabled: {}", report.trading_status.trading_enabled);
    println!("  Active Positions: {}", report.trading_status.active_positions);
    println!("  Pending Orders: {}", report.trading_status.pending_orders);
    println!("  Daily PnL: ${:.2}", report.trading_status.daily_pnl);
    println!("  Risk Utilization: {:.1}%", report.trading_status.risk_utilization_percent);

    println!("\n=== JSON Response ===");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Liveness Probe** | Базовая проверка "жив ли процесс" |
| **Readiness Probe** | Проверка готовности принимать трафик |
| **Startup Probe** | Проверка завершения инициализации |
| **Deep Health Check** | Комплексная проверка всех компонентов |
| **Health Status** | Healthy / Degraded / Unhealthy |
| **Component Health** | Статус отдельного компонента системы |

## Практические задания

1. **Health Check для WebSocket**: Создай проверку здоровья для WebSocket соединения с биржей:
   - Проверка активности соединения
   - Измерение задержки (ping/pong)
   - Статус подписок на инструменты
   - Автоматическое переподключение

2. **Cascade Health Check**: Реализуй каскадную проверку зависимостей:
   - Если БД недоступна → система unhealthy
   - Если API биржи медленный → система degraded
   - Если кэш устарел → warning
   - Агрегация статусов всех зависимостей

3. **Health Check Dashboard**: Создай веб-интерфейс для мониторинга:
   - Отображение статуса всех компонентов
   - История проверок за последний час
   - Графики латентности
   - Алерты при изменении статуса

4. **Self-Healing Health Check**: Реализуй систему самовосстановления:
   - При обнаружении проблемы → попытка автоматического исправления
   - Перезапуск зависших компонентов
   - Переключение на резервные подключения
   - Логирование всех действий

## Домашнее задание

1. **Production Health Check System**: Разработай полноценную систему проверки здоровья:
   - Поддержка HTTP и gRPC endpoints
   - Кастомные проверки через plugin-систему
   - Интеграция с Prometheus для метрик
   - Alerting через webhook
   - Поддержка graceful degradation

2. **Multi-Region Health Checker**: Создай систему для проверки здоровья в нескольких регионах:
   - Проверка доступности из разных точек
   - Сравнение латентности между регионами
   - Автоматический failover при проблемах
   - Dashboard с географической картой

3. **Trading-Specific Health Checks**: Реализуй специализированные проверки:
   - Проверка синхронизации ордербука
   - Валидация баланса на бирже
   - Проверка исполнения тестового ордера
   - Мониторинг slippage
   - Проверка лимитов API

4. **Health Check Automation**: Создай систему автоматизации:
   - Генерация health check кода из OpenAPI spec
   - Автоматическое обнаружение зависимостей
   - CI/CD интеграция для проверки перед деплоем
   - Canary deployment с проверкой здоровья

## Навигация

[← Предыдущий день](../338-graceful-shutdown/ru.md) | [Следующий день →](../340-metrics-prometheus/ru.md)
