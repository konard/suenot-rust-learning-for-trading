# День 345: CI/CD: автоматический деплой

## Аналогия из трейдинга

Представь, что ты разработал новую торговую стратегию. Прежде чем запустить её на реальные деньги, ты хочешь убедиться, что она работает правильно:

**Ручной деплой (старый подход):**
Ты каждый раз вручную проверяешь код стратегии, запускаешь тесты, копируешь файлы на сервер, перезапускаешь бота. Это занимает много времени и подвержено ошибкам — можно забыть запустить тесты или скопировать не тот файл.

**Автоматический CI/CD (современный подход):**
У тебя есть автоматизированный конвейер. Когда ты делаешь изменения в коде:
1. **CI (Continuous Integration)** — автоматически запускаются тесты, проверяется код
2. **CD (Continuous Deployment)** — если всё прошло успешно, код автоматически деплоится на сервер

| Этап | Аналогия в трейдинге | Действие |
|------|----------------------|----------|
| **Commit** | Записать изменения стратегии | Сохранение кода |
| **Build** | Скомпилировать торгового бота | Сборка приложения |
| **Test** | Прогнать бэктест на исторических данных | Запуск тестов |
| **Stage** | Протестировать на демо-счёте | Развёртывание в staging |
| **Deploy** | Запустить на реальном счёте | Развёртывание в production |

## Основы CI/CD для Rust

### Структура GitHub Actions

GitHub Actions — популярный инструмент для CI/CD. Конфигурация хранится в `.github/workflows/`:

```yaml
# .github/workflows/ci.yml
name: Trading Bot CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Этап 1: Проверка кода
  lint:
    name: Lint & Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  # Этап 2: Тестирование
  test:
    name: Test
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Run doc tests
        run: cargo test --doc

  # Этап 3: Сборка
  build:
    name: Build Release
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Build release
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: trading-bot
          path: target/release/trading-bot

  # Этап 4: Деплой
  deploy:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: trading-bot

      - name: Deploy to server
        run: |
          echo "Deploying trading bot to production..."
          # Здесь команды для деплоя
```

### Пример торгового бота с CI/CD

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Конфигурация деплоя
#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub environment: Environment,
    pub api_endpoint: String,
    pub max_position_size: f64,
    pub risk_limit_pct: f64,
    pub enable_trading: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl DeployConfig {
    /// Загружает конфигурацию из переменных окружения
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("DEPLOY_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let environment = match env.as_str() {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            _ => Environment::Development,
        };

        let api_endpoint = std::env::var("API_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let max_position_size: f64 = std::env::var("MAX_POSITION_SIZE")
            .unwrap_or_else(|_| "1000.0".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("MAX_POSITION_SIZE"))?;

        let risk_limit_pct: f64 = std::env::var("RISK_LIMIT_PCT")
            .unwrap_or_else(|_| "2.0".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("RISK_LIMIT_PCT"))?;

        // В production торговля включена только явно
        let enable_trading = std::env::var("ENABLE_TRADING")
            .map(|v| v == "true")
            .unwrap_or(environment != Environment::Production);

        Ok(DeployConfig {
            environment,
            api_endpoint,
            max_position_size,
            risk_limit_pct,
            enable_trading,
        })
    }

    /// Проверяет конфигурацию перед деплоем
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_position_size <= 0.0 {
            return Err(ConfigError::InvalidValue("max_position_size must be positive"));
        }

        if self.risk_limit_pct <= 0.0 || self.risk_limit_pct > 100.0 {
            return Err(ConfigError::InvalidValue("risk_limit_pct must be between 0 and 100"));
        }

        // Дополнительные проверки для production
        if self.environment == Environment::Production {
            if self.api_endpoint.contains("localhost") {
                return Err(ConfigError::InvalidValue("localhost not allowed in production"));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidValue(&'static str),
    MissingEnvVar(&'static str),
}

/// Торговый бот с поддержкой горячего обновления
pub struct TradingBot {
    config: Arc<RwLock<DeployConfig>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    version: String,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
}

impl Position {
    pub fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }
}

impl TradingBot {
    pub fn new(config: DeployConfig) -> Self {
        TradingBot {
            config: Arc::new(RwLock::new(config)),
            positions: Arc::new(RwLock::new(HashMap::new())),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Проверка здоровья для деплоя
    pub async fn health_check(&self) -> HealthStatus {
        let config = self.config.read().await;
        let positions = self.positions.read().await;

        let total_pnl: f64 = positions.values()
            .map(|p| p.unrealized_pnl())
            .sum();

        HealthStatus {
            healthy: true,
            version: self.version.clone(),
            environment: format!("{:?}", config.environment),
            open_positions: positions.len(),
            total_unrealized_pnl: total_pnl,
            trading_enabled: config.enable_trading,
        }
    }

    /// Горячее обновление конфигурации без остановки бота
    pub async fn hot_reload_config(&self, new_config: DeployConfig) -> Result<(), ConfigError> {
        new_config.validate()?;

        let mut config = self.config.write().await;

        println!("=== Горячее обновление конфигурации ===");
        println!("Старая: {:?}", config.environment);
        println!("Новая: {:?}", new_config.environment);

        *config = new_config;

        Ok(())
    }

    /// Graceful shutdown — безопасное завершение
    pub async fn graceful_shutdown(&self) -> ShutdownResult {
        println!("=== Начало graceful shutdown ===");

        let positions = self.positions.read().await;

        if !positions.is_empty() {
            println!("Открытые позиции: {}", positions.len());
            println!("Закрываем все позиции...");

            // В реальной системе здесь были бы вызовы API биржи
            for (symbol, pos) in positions.iter() {
                println!("  Закрытие {}: {} @ {:.2}", symbol, pos.quantity, pos.current_price);
            }
        }

        println!("=== Shutdown завершён ===");

        ShutdownResult {
            positions_closed: positions.len(),
            success: true,
        }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: String,
    pub environment: String,
    pub open_positions: usize,
    pub total_unrealized_pnl: f64,
    pub trading_enabled: bool,
}

#[derive(Debug)]
pub struct ShutdownResult {
    pub positions_closed: usize,
    pub success: bool,
}

fn main() {
    println!("=== CI/CD: Автоматический деплой ===\n");

    // Демонстрация загрузки конфигурации
    std::env::set_var("DEPLOY_ENV", "staging");
    std::env::set_var("MAX_POSITION_SIZE", "5000.0");
    std::env::set_var("RISK_LIMIT_PCT", "1.5");

    let config = DeployConfig::from_env().expect("Failed to load config");
    println!("Загружена конфигурация: {:?}", config.environment);
    println!("  API: {}", config.api_endpoint);
    println!("  Max position: ${:.2}", config.max_position_size);
    println!("  Risk limit: {:.1}%", config.risk_limit_pct);
    println!("  Trading: {}", if config.enable_trading { "enabled" } else { "disabled" });

    // Валидация
    match config.validate() {
        Ok(()) => println!("\nКонфигурация валидна!"),
        Err(e) => println!("\nОшибка валидации: {:?}", e),
    }
}
```

## Blue-Green Deployment

Blue-Green deployment — стратегия, при которой поддерживаются две идентичные среды:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Blue-Green деплой для торговой системы
pub struct BlueGreenDeployment {
    blue_active: Arc<AtomicBool>,
    blue_version: String,
    green_version: String,
}

#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    pub active_env: &'static str,
    pub active_version: String,
    pub standby_version: String,
}

impl BlueGreenDeployment {
    pub fn new(initial_version: String) -> Self {
        BlueGreenDeployment {
            blue_active: Arc::new(AtomicBool::new(true)),
            blue_version: initial_version.clone(),
            green_version: initial_version,
        }
    }

    /// Развёртывание новой версии в standby среду
    pub fn deploy_to_standby(&mut self, new_version: String) -> Result<(), DeployError> {
        println!("Деплой версии {} в standby среду...", new_version);

        if self.blue_active.load(Ordering::SeqCst) {
            // Blue активен, деплоим в Green
            self.green_version = new_version;
            println!("  -> Развёрнуто в Green");
        } else {
            // Green активен, деплоим в Blue
            self.blue_version = new_version;
            println!("  -> Развёрнуто в Blue");
        }

        Ok(())
    }

    /// Переключение трафика на новую версию
    pub fn switch_traffic(&self) -> DeploymentStatus {
        let was_blue = self.blue_active.fetch_xor(true, Ordering::SeqCst);

        if was_blue {
            println!("Трафик переключён: Blue -> Green");
        } else {
            println!("Трафик переключён: Green -> Blue");
        }

        self.status()
    }

    /// Откат на предыдущую версию
    pub fn rollback(&self) -> DeploymentStatus {
        println!("Откат на предыдущую версию...");
        self.switch_traffic()
    }

    /// Текущий статус деплоя
    pub fn status(&self) -> DeploymentStatus {
        let blue_active = self.blue_active.load(Ordering::SeqCst);

        DeploymentStatus {
            active_env: if blue_active { "Blue" } else { "Green" },
            active_version: if blue_active {
                self.blue_version.clone()
            } else {
                self.green_version.clone()
            },
            standby_version: if blue_active {
                self.green_version.clone()
            } else {
                self.blue_version.clone()
            },
        }
    }
}

#[derive(Debug)]
pub enum DeployError {
    ValidationFailed(String),
    DeploymentFailed(String),
}

fn main() {
    println!("=== Blue-Green Deployment ===\n");

    let mut deployment = BlueGreenDeployment::new("v1.0.0".to_string());

    println!("Начальный статус: {:?}\n", deployment.status());

    // Деплоим новую версию
    deployment.deploy_to_standby("v1.1.0".to_string()).unwrap();
    println!("После деплоя: {:?}\n", deployment.status());

    // Переключаем трафик
    let status = deployment.switch_traffic();
    println!("После переключения: {:?}\n", status);

    // Откат (если что-то пошло не так)
    let status = deployment.rollback();
    println!("После отката: {:?}", status);
}
```

## Canary Deployment

Canary deployment — постепенное развёртывание новой версии на часть трафика:

```rust
use std::collections::HashMap;

/// Canary деплой — постепенное развёртывание
pub struct CanaryDeployment {
    stable_version: String,
    canary_version: Option<String>,
    canary_percentage: u8,  // 0-100
    metrics: DeployMetrics,
}

#[derive(Debug, Default)]
pub struct DeployMetrics {
    pub stable_requests: u64,
    pub canary_requests: u64,
    pub stable_errors: u64,
    pub canary_errors: u64,
    pub stable_latency_sum_ms: u64,
    pub canary_latency_sum_ms: u64,
}

impl DeployMetrics {
    pub fn stable_error_rate(&self) -> f64 {
        if self.stable_requests == 0 {
            0.0
        } else {
            self.stable_errors as f64 / self.stable_requests as f64 * 100.0
        }
    }

    pub fn canary_error_rate(&self) -> f64 {
        if self.canary_requests == 0 {
            0.0
        } else {
            self.canary_errors as f64 / self.canary_requests as f64 * 100.0
        }
    }

    pub fn stable_avg_latency(&self) -> f64 {
        if self.stable_requests == 0 {
            0.0
        } else {
            self.stable_latency_sum_ms as f64 / self.stable_requests as f64
        }
    }

    pub fn canary_avg_latency(&self) -> f64 {
        if self.canary_requests == 0 {
            0.0
        } else {
            self.canary_latency_sum_ms as f64 / self.canary_requests as f64
        }
    }
}

impl CanaryDeployment {
    pub fn new(stable_version: String) -> Self {
        CanaryDeployment {
            stable_version,
            canary_version: None,
            canary_percentage: 0,
            metrics: DeployMetrics::default(),
        }
    }

    /// Запуск canary с указанным процентом трафика
    pub fn start_canary(&mut self, version: String, percentage: u8) {
        println!(
            "Запуск canary: версия {} с {}% трафика",
            version, percentage
        );
        self.canary_version = Some(version);
        self.canary_percentage = percentage.min(100);
        self.metrics = DeployMetrics::default();
    }

    /// Определение версии для обработки запроса
    pub fn route_request(&self) -> &str {
        let random: u8 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() % 100) as u8;

        if let Some(ref canary) = self.canary_version {
            if random < self.canary_percentage {
                return canary;
            }
        }
        &self.stable_version
    }

    /// Увеличение процента canary трафика
    pub fn increase_canary(&mut self, new_percentage: u8) {
        let old = self.canary_percentage;
        self.canary_percentage = new_percentage.min(100);
        println!(
            "Canary трафик увеличен: {}% -> {}%",
            old, self.canary_percentage
        );
    }

    /// Промоутинг canary в stable
    pub fn promote_canary(&mut self) -> Result<(), &'static str> {
        match &self.canary_version {
            Some(version) => {
                println!("Промоутинг canary {} в stable", version);
                self.stable_version = version.clone();
                self.canary_version = None;
                self.canary_percentage = 0;
                Ok(())
            }
            None => Err("No canary version to promote"),
        }
    }

    /// Откат canary
    pub fn rollback_canary(&mut self) {
        if self.canary_version.is_some() {
            println!("Откат canary версии");
            self.canary_version = None;
            self.canary_percentage = 0;
        }
    }

    /// Проверка метрик для автоматического решения
    pub fn should_promote(&self) -> bool {
        // Promote если canary не хуже stable
        let error_diff = self.metrics.canary_error_rate() - self.metrics.stable_error_rate();
        let latency_diff = self.metrics.canary_avg_latency() - self.metrics.stable_avg_latency();

        // Допускаем небольшое ухудшение
        error_diff < 0.5 && latency_diff < 10.0
    }

    /// Проверка метрик для автоматического отката
    pub fn should_rollback(&self) -> bool {
        // Rollback если canary значительно хуже
        let error_rate = self.metrics.canary_error_rate();
        let stable_error_rate = self.metrics.stable_error_rate();

        // Откат если ошибок вдвое больше или более 5%
        error_rate > stable_error_rate * 2.0 || error_rate > 5.0
    }

    /// Симуляция запросов для демонстрации
    pub fn record_request(&mut self, is_canary: bool, success: bool, latency_ms: u64) {
        if is_canary {
            self.metrics.canary_requests += 1;
            self.metrics.canary_latency_sum_ms += latency_ms;
            if !success {
                self.metrics.canary_errors += 1;
            }
        } else {
            self.metrics.stable_requests += 1;
            self.metrics.stable_latency_sum_ms += latency_ms;
            if !success {
                self.metrics.stable_errors += 1;
            }
        }
    }
}

fn main() {
    println!("=== Canary Deployment ===\n");

    let mut deployment = CanaryDeployment::new("v1.0.0".to_string());

    // Запускаем canary с 10% трафика
    deployment.start_canary("v1.1.0".to_string(), 10);

    // Симуляция запросов
    println!("\nСимуляция 100 запросов...");
    for i in 0..100 {
        let version = deployment.route_request();
        let is_canary = version == "v1.1.0";
        let success = i % 20 != 0;  // 95% успешных запросов
        let latency = if is_canary { 45 } else { 50 };  // Canary чуть быстрее
        deployment.record_request(is_canary, success, latency);
    }

    // Анализ метрик
    println!("\nМетрики:");
    println!(
        "  Stable: {} запросов, {:.1}% ошибок, {:.1}ms latency",
        deployment.metrics.stable_requests,
        deployment.metrics.stable_error_rate(),
        deployment.metrics.stable_avg_latency()
    );
    println!(
        "  Canary: {} запросов, {:.1}% ошибок, {:.1}ms latency",
        deployment.metrics.canary_requests,
        deployment.metrics.canary_error_rate(),
        deployment.metrics.canary_avg_latency()
    );

    // Решение о продвижении
    if deployment.should_rollback() {
        println!("\nРешение: ОТКАТ (метрики ухудшились)");
        deployment.rollback_canary();
    } else if deployment.should_promote() {
        println!("\nРешение: ПРОМОУТ (метрики в норме)");
        deployment.promote_canary().unwrap();
    } else {
        println!("\nРешение: Увеличить canary трафик");
        deployment.increase_canary(25);
    }
}
```

## Docker и Kubernetes для торговых систем

### Dockerfile для Rust бота

```dockerfile
# Dockerfile
# Этап 1: Сборка
FROM rust:1.75 as builder

WORKDIR /app

# Кешируем зависимости
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Собираем приложение
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# Этап 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trading-bot /usr/local/bin/

# Не root пользователь для безопасности
RUN useradd -m -u 1000 trader
USER trader

ENV RUST_LOG=info

ENTRYPOINT ["trading-bot"]
```

### Kubernetes манифесты

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-bot
  labels:
    app: trading-bot
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  selector:
    matchLabels:
      app: trading-bot
  template:
    metadata:
      labels:
        app: trading-bot
    spec:
      containers:
      - name: trading-bot
        image: trading-bot:latest
        ports:
        - containerPort: 8080
        env:
        - name: DEPLOY_ENV
          value: "production"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 20
---
apiVersion: v1
kind: Service
metadata:
  name: trading-bot
spec:
  selector:
    app: trading-bot
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

## Health Checks и Readiness Probes

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Состояние здоровья сервиса
#[derive(Debug, Clone)]
pub struct HealthState {
    pub ready: bool,
    pub live: bool,
    pub exchange_connected: bool,
    pub database_connected: bool,
    pub last_heartbeat: std::time::Instant,
}

impl Default for HealthState {
    fn default() -> Self {
        HealthState {
            ready: false,
            live: true,
            exchange_connected: false,
            database_connected: false,
            last_heartbeat: std::time::Instant::now(),
        }
    }
}

/// Компонент проверки здоровья
pub struct HealthChecker {
    state: Arc<RwLock<HealthState>>,
    startup_timeout_secs: u64,
    heartbeat_timeout_secs: u64,
}

impl HealthChecker {
    pub fn new() -> Self {
        HealthChecker {
            state: Arc::new(RwLock::new(HealthState::default())),
            startup_timeout_secs: 30,
            heartbeat_timeout_secs: 60,
        }
    }

    /// Liveness check — процесс жив?
    pub async fn liveness(&self) -> LivenessResponse {
        let state = self.state.read().await;

        let heartbeat_age = state.last_heartbeat.elapsed().as_secs();
        let is_live = state.live && heartbeat_age < self.heartbeat_timeout_secs;

        LivenessResponse {
            status: if is_live { "ok" } else { "error" },
            live: is_live,
            heartbeat_age_secs: heartbeat_age,
        }
    }

    /// Readiness check — готов принимать трафик?
    pub async fn readiness(&self) -> ReadinessResponse {
        let state = self.state.read().await;

        let is_ready = state.ready
            && state.exchange_connected
            && state.database_connected;

        ReadinessResponse {
            status: if is_ready { "ok" } else { "not_ready" },
            ready: is_ready,
            checks: HealthChecks {
                exchange: state.exchange_connected,
                database: state.database_connected,
            },
        }
    }

    /// Обновление состояния компонентов
    pub async fn update_component(&self, component: &str, healthy: bool) {
        let mut state = self.state.write().await;

        match component {
            "exchange" => state.exchange_connected = healthy,
            "database" => state.database_connected = healthy,
            _ => {}
        }

        state.last_heartbeat = std::time::Instant::now();

        // Готов, если все компоненты работают
        state.ready = state.exchange_connected && state.database_connected;
    }

    /// Пометить сервис как unhealthy
    pub async fn mark_unhealthy(&self, reason: &str) {
        let mut state = self.state.write().await;
        state.live = false;
        println!("Service marked unhealthy: {}", reason);
    }
}

#[derive(Debug)]
pub struct LivenessResponse {
    pub status: &'static str,
    pub live: bool,
    pub heartbeat_age_secs: u64,
}

#[derive(Debug)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub ready: bool,
    pub checks: HealthChecks,
}

#[derive(Debug)]
pub struct HealthChecks {
    pub exchange: bool,
    pub database: bool,
}

#[tokio::main]
async fn main() {
    println!("=== Health Checks Demo ===\n");

    let checker = HealthChecker::new();

    // Начальное состояние
    println!("Начальное состояние:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);

    // Симуляция запуска
    println!("\nЗапуск компонентов...");
    checker.update_component("database", true).await;
    println!("  Database connected");

    checker.update_component("exchange", true).await;
    println!("  Exchange connected");

    // Проверка после запуска
    println!("\nПосле запуска:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);

    // Симуляция проблемы с биржей
    println!("\nОтключение от биржи...");
    checker.update_component("exchange", false).await;

    println!("После отключения:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **CI (Continuous Integration)** | Автоматическая сборка и тестирование при каждом коммите |
| **CD (Continuous Deployment)** | Автоматический деплой после успешного CI |
| **Blue-Green Deployment** | Две идентичные среды для безопасного переключения |
| **Canary Deployment** | Постепенное развёртывание на часть трафика |
| **Health Checks** | Проверки жизнеспособности и готовности сервиса |
| **Graceful Shutdown** | Корректное завершение с закрытием позиций |
| **Hot Reload** | Обновление конфигурации без остановки |

## Практические задания

1. **CI Pipeline для торгового бота**: Создай GitHub Actions workflow, который:
   - Проверяет форматирование кода
   - Запускает clippy с максимальными проверками
   - Выполняет unit и integration тесты
   - Собирает Docker образ
   - Пушит образ в registry

2. **Blue-Green контроллер**: Реализуй систему, которая:
   - Управляет двумя средами (blue/green)
   - Автоматически переключает трафик
   - Мониторит метрики после переключения
   - Выполняет автоматический откат при проблемах

3. **Canary с автоматическим анализом**: Создай систему:
   - Автоматически увеличивает процент canary трафика
   - Анализирует метрики в реальном времени
   - Принимает решение о promote/rollback
   - Уведомляет о проблемах

4. **Health Check система**: Разработай компонент:
   - Проверяет все зависимости (база, биржа, кэш)
   - Поддерживает liveness и readiness probes
   - Собирает метрики здоровья
   - Интегрируется с Kubernetes

## Домашнее задание

1. **Полный CI/CD pipeline**: Настрой pipeline, который:
   - Запускается на push в develop и main
   - Включает все проверки качества кода
   - Деплоит в staging на develop
   - Деплоит в production на main с ручным подтверждением
   - Отправляет уведомления в Slack/Telegram
   - Хранит артефакты и отчёты о тестах

2. **Система версионирования**: Реализуй:
   - Semantic versioning для релизов
   - Автоматическое обновление changelog
   - Git tags для релизов
   - Отображение версии в health check
   - Проверку совместимости конфигурации

3. **Disaster Recovery план**: Создай систему:
   - Автоматический backup позиций и состояния
   - Восстановление из backup при старте
   - Проверка целостности данных
   - Режим "только чтение" при проблемах
   - Документация процедур восстановления

4. **Мониторинг деплоя**: Разработай dashboard:
   - Время деплоя и откатов
   - Частота деплоев по окружениям
   - Метрики успешности (MTTR, MTBF)
   - Алерты на аномалии
   - Интеграция с Prometheus/Grafana

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../346-*/ru.md)
