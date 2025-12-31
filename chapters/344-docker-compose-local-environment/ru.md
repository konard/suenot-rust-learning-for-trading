# День 344: Docker Compose: локальное окружение

## Аналогия из трейдинга

Представь, что ты запускаешь торговый деск. Тебе нужно:
- Сам торговый бот (исполняет стратегию)
- База данных (хранит историю сделок)
- Redis (кэширует цены в реальном времени)
- Grafana (мониторит прибыль и убытки)
- Prometheus (собирает метрики производительности)

Запускать каждый сервис вручную — как управлять торговым залом, где каждый трейдер использует разные инструменты и системы. **Docker Compose** — это твой операционный менеджер, который гарантирует, что все системы запускаются правильно, могут общаться друг с другом и корректно завершают работу.

| Концепция | Торговая аналогия |
|-----------|-------------------|
| **Контейнер** | Отдельный торговый терминал с настроенным окружением |
| **Сервис** | Торговая функция (исполнение, риск-менеджмент, данные) |
| **Сеть** | Внутренняя коммуникационная система торгового деска |
| **Том** | Безопасное хранилище для торговой истории и конфигурации |
| **Compose файл** | Операционный чек-лист для запуска всего деска |

## Основы Docker Compose

Docker Compose позволяет определить многоконтейнерные приложения в одном YAML-файле.

### Базовая структура docker-compose.yml

```yaml
# docker-compose.yml
version: '3.8'

services:
  # Торговый бот — основное приложение
  trading-bot:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: trading-bot
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://trader:secret@postgres:5432/trading
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    networks:
      - trading-network
    volumes:
      - ./config:/app/config:ro
      - trading-logs:/app/logs
    restart: unless-stopped

  # PostgreSQL для истории сделок
  postgres:
    image: postgres:15-alpine
    container_name: trading-postgres
    environment:
      POSTGRES_USER: trader
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: trading
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    networks:
      - trading-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U trader -d trading"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis для кэширования цен в реальном времени
  redis:
    image: redis:7-alpine
    container_name: trading-redis
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - trading-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

networks:
  trading-network:
    driver: bridge

volumes:
  postgres-data:
  redis-data:
  trading-logs:
```

### Dockerfile для Rust-приложения

```dockerfile
# Dockerfile
# Этап сборки
FROM rust:1.75-alpine AS builder

WORKDIR /app

# Устанавливаем зависимости для сборки
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Кэшируем зависимости
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Собираем приложение
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Финальный образ
FROM alpine:3.19

WORKDIR /app

# Устанавливаем рантайм-зависимости
RUN apk add --no-cache ca-certificates libgcc

# Копируем собранный бинарник
COPY --from=builder /app/target/release/trading-bot /app/trading-bot

# Создаём пользователя без root-прав
RUN addgroup -S trader && adduser -S trader -G trader
USER trader

EXPOSE 8080

CMD ["./trading-bot"]
```

## Торговая инфраструктура с Docker Compose

### Полный стек для алготрейдинга

```yaml
# docker-compose.trading.yml
version: '3.8'

services:
  # === Основные сервисы ===

  trading-bot:
    build:
      context: ./trading-bot
      dockerfile: Dockerfile
    container_name: algo-trading-bot
    environment:
      - RUST_LOG=info,trading_bot=debug
      - DATABASE_URL=postgres://trader:${DB_PASSWORD}@postgres:5432/trading
      - REDIS_URL=redis://redis:6379
      - EXCHANGE_API_KEY=${EXCHANGE_API_KEY}
      - EXCHANGE_API_SECRET=${EXCHANGE_API_SECRET}
    env_file:
      - .env
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - trading-network
    volumes:
      - ./config:/app/config:ro
      - trading-logs:/app/logs
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M

  # Сервис рыночных данных
  market-data:
    build:
      context: ./market-data
      dockerfile: Dockerfile
    container_name: market-data-service
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - SYMBOLS=BTCUSDT,ETHUSDT,SOLUSDT
    depends_on:
      redis:
        condition: service_healthy
    networks:
      - trading-network
    restart: unless-stopped

  # Сервис управления рисками
  risk-manager:
    build:
      context: ./risk-manager
      dockerfile: Dockerfile
    container_name: risk-manager
    environment:
      - RUST_LOG=info,risk_manager=debug
      - DATABASE_URL=postgres://trader:${DB_PASSWORD}@postgres:5432/trading
      - MAX_POSITION_SIZE=10000
      - MAX_DAILY_LOSS=1000
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - trading-network
    restart: unless-stopped

  # === Хранение данных ===

  postgres:
    image: postgres:15-alpine
    container_name: trading-postgres
    environment:
      POSTGRES_USER: trader
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: trading
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/01-init.sql:ro
      - ./sql/schema.sql:/docker-entrypoint-initdb.d/02-schema.sql:ro
    networks:
      - trading-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U trader -d trading"]
      interval: 10s
      timeout: 5s
      retries: 5
    deploy:
      resources:
        limits:
          memory: 1G

  redis:
    image: redis:7-alpine
    container_name: trading-redis
    command: redis-server --appendonly yes --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redis-data:/data
    networks:
      - trading-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  # === Мониторинг ===

  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: trading-prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    networks:
      - trading-network
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:10.2.2
    container_name: trading-grafana
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    depends_on:
      - prometheus
    networks:
      - trading-network
    ports:
      - "3000:3000"

  # Алерт-менеджер для уведомлений
  alertmanager:
    image: prom/alertmanager:v0.26.0
    container_name: trading-alertmanager
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
    networks:
      - trading-network
    ports:
      - "9093:9093"

networks:
  trading-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

volumes:
  postgres-data:
  redis-data:
  prometheus-data:
  grafana-data:
  trading-logs:
```

### Файл переменных окружения

```bash
# .env
# Учётные данные базы данных
DB_PASSWORD=your_secure_password_here

# API-ключи биржи (не коммитить в git!)
EXCHANGE_API_KEY=your_api_key
EXCHANGE_API_SECRET=your_api_secret

# Мониторинг
GRAFANA_PASSWORD=admin_password

# Настройки окружения
ENVIRONMENT=development
LOG_LEVEL=debug
```

## Rust-код для интеграции с Docker-сервисами

### Конфигурация приложения

```rust
use std::env;
use serde::Deserialize;

/// Конфигурация торгового бота из переменных окружения
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub exchange_api_key: String,
    pub exchange_api_secret: String,
    pub log_level: String,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
}

impl Config {
    /// Загрузка конфигурации из переменных окружения
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            redis_url: env::var("REDIS_URL")
                .map_err(|_| ConfigError::Missing("REDIS_URL"))?,
            exchange_api_key: env::var("EXCHANGE_API_KEY")
                .map_err(|_| ConfigError::Missing("EXCHANGE_API_KEY"))?,
            exchange_api_secret: env::var("EXCHANGE_API_SECRET")
                .map_err(|_| ConfigError::Missing("EXCHANGE_API_SECRET"))?,
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000.0),
            max_daily_loss: env::var("MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000.0),
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => write!(f, "Отсутствует переменная окружения: {}", var),
            ConfigError::Invalid(var) => write!(f, "Некорректное значение переменной: {}", var),
        }
    }
}
```

### Проверка состояния сервисов

```rust
use std::time::Duration;
use tokio::time::timeout;

/// Проверка состояния сервисов
pub struct HealthChecker {
    database_url: String,
    redis_url: String,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub database: ServiceStatus,
    pub redis: ServiceStatus,
    pub overall: bool,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy(String),
    Unknown,
}

impl HealthChecker {
    pub fn new(database_url: String, redis_url: String) -> Self {
        HealthChecker {
            database_url,
            redis_url,
        }
    }

    /// Проверка здоровья всех сервисов
    pub async fn check_all(&self) -> HealthStatus {
        let (db_status, redis_status) = tokio::join!(
            self.check_database(),
            self.check_redis()
        );

        let overall = matches!(db_status, ServiceStatus::Healthy)
            && matches!(redis_status, ServiceStatus::Healthy);

        HealthStatus {
            database: db_status,
            redis: redis_status,
            overall,
        }
    }

    async fn check_database(&self) -> ServiceStatus {
        // Имитация проверки соединения с PostgreSQL
        match timeout(Duration::from_secs(5), self.ping_database()).await {
            Ok(Ok(())) => ServiceStatus::Healthy,
            Ok(Err(e)) => ServiceStatus::Unhealthy(e),
            Err(_) => ServiceStatus::Unhealthy("Таймаут соединения".to_string()),
        }
    }

    async fn check_redis(&self) -> ServiceStatus {
        // Имитация проверки соединения с Redis
        match timeout(Duration::from_secs(5), self.ping_redis()).await {
            Ok(Ok(())) => ServiceStatus::Healthy,
            Ok(Err(e)) => ServiceStatus::Unhealthy(e),
            Err(_) => ServiceStatus::Unhealthy("Таймаут соединения".to_string()),
        }
    }

    async fn ping_database(&self) -> Result<(), String> {
        // В реальном коде здесь будет подключение к PostgreSQL
        // sqlx::PgPool::connect(&self.database_url).await
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    async fn ping_redis(&self) -> Result<(), String> {
        // В реальном коде здесь будет подключение к Redis
        // redis::Client::open(&self.redis_url)?.get_connection()
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("=== Проверка состояния торговой инфраструктуры ===\n");

    let checker = HealthChecker::new(
        "postgres://trader:secret@localhost:5432/trading".to_string(),
        "redis://localhost:6379".to_string(),
    );

    let status = checker.check_all().await;

    println!("База данных: {:?}", status.database);
    println!("Redis: {:?}", status.redis);
    println!("Общее состояние: {}", if status.overall { "Здоров" } else { "Проблемы" });
}
```

### Кэш цен с Redis

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Данные о цене
#[derive(Debug, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: f64,
    pub timestamp: Instant,
}

/// Локальный кэш цен с TTL (имитация Redis)
pub struct PriceCache {
    cache: HashMap<String, PriceData>,
    ttl: Duration,
}

impl PriceCache {
    pub fn new(ttl_seconds: u64) -> Self {
        PriceCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Сохранение цены в кэш
    pub fn set(&mut self, symbol: &str, bid: f64, ask: f64, last: f64, volume: f64) {
        let data = PriceData {
            symbol: symbol.to_string(),
            bid,
            ask,
            last,
            volume,
            timestamp: Instant::now(),
        };
        self.cache.insert(symbol.to_string(), data);
    }

    /// Получение цены из кэша
    pub fn get(&self, symbol: &str) -> Option<&PriceData> {
        self.cache.get(symbol).filter(|data| {
            data.timestamp.elapsed() < self.ttl
        })
    }

    /// Получение спреда
    pub fn get_spread(&self, symbol: &str) -> Option<f64> {
        self.get(symbol).map(|data| data.ask - data.bid)
    }

    /// Очистка устаревших записей
    pub fn cleanup_expired(&mut self) {
        self.cache.retain(|_, data| {
            data.timestamp.elapsed() < self.ttl
        });
    }

    /// Получение всех активных символов
    pub fn get_active_symbols(&self) -> Vec<String> {
        self.cache
            .iter()
            .filter(|(_, data)| data.timestamp.elapsed() < self.ttl)
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }
}

fn main() {
    println!("=== Демонстрация кэша цен ===\n");

    let mut cache = PriceCache::new(60); // TTL 60 секунд

    // Имитация обновления цен с биржи
    let prices = vec![
        ("BTCUSDT", 50000.0, 50010.0, 50005.0, 1500.5),
        ("ETHUSDT", 3000.0, 3002.0, 3001.0, 5000.0),
        ("SOLUSDT", 100.0, 100.1, 100.05, 10000.0),
    ];

    for (symbol, bid, ask, last, volume) in prices {
        cache.set(symbol, bid, ask, last, volume);
        println!("Обновлена цена: {} - bid: {}, ask: {}", symbol, bid, ask);
    }

    println!("\n=== Состояние кэша ===");
    for symbol in cache.get_active_symbols() {
        if let Some(data) = cache.get(&symbol) {
            println!(
                "{}: ${:.2} (спред: ${:.2})",
                symbol,
                data.last,
                cache.get_spread(&symbol).unwrap_or(0.0)
            );
        }
    }
}
```

## Профили для разных окружений

### Разработка (Development)

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  trading-bot:
    build:
      context: .
      dockerfile: Dockerfile.dev
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    volumes:
      - ./src:/app/src:ro          # Маунт исходного кода для hot-reload
      - ./target:/app/target        # Кэширование сборки
    ports:
      - "8080:8080"                 # Отладочный порт
      - "9229:9229"                 # Порт дебаггера

  postgres:
    ports:
      - "5432:5432"                 # Доступ к БД извне

  redis:
    ports:
      - "6379:6379"                 # Доступ к Redis извне

  # Инструменты разработки
  adminer:
    image: adminer:4
    ports:
      - "8081:8080"
    networks:
      - trading-network

  redis-commander:
    image: rediscommander/redis-commander:latest
    environment:
      - REDIS_HOSTS=local:redis:6379
    ports:
      - "8082:8081"
    networks:
      - trading-network
```

### Продакшн (Production)

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  trading-bot:
    image: ${REGISTRY}/trading-bot:${VERSION}
    environment:
      - RUST_LOG=info
    deploy:
      replicas: 2
      update_config:
        parallelism: 1
        delay: 30s
        failure_action: rollback
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      resources:
        limits:
          cpus: '4'
          memory: 4G
        reservations:
          cpus: '2'
          memory: 2G
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  postgres:
    deploy:
      resources:
        limits:
          memory: 2G
    # Не экспортируем порты в продакшне!
```

### Запуск с профилями

```bash
# Разработка
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# Продакшн
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# С конкретными сервисами
docker compose up trading-bot postgres redis

# Пересборка после изменений
docker compose build --no-cache trading-bot
docker compose up -d trading-bot
```

## Мониторинг и логирование

### Конфигурация Prometheus

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

rule_files:
  - "alerts/*.yml"

scrape_configs:
  - job_name: 'trading-bot'
    static_configs:
      - targets: ['trading-bot:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'market-data'
    static_configs:
      - targets: ['market-data:8080']

  - job_name: 'risk-manager'
    static_configs:
      - targets: ['risk-manager:8080']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

### Правила алертов для трейдинга

```yaml
# monitoring/alerts/trading.yml
groups:
  - name: trading_alerts
    rules:
      - alert: HighDrawdown
        expr: trading_drawdown_percent > 5
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Высокая просадка: {{ $value }}%"
          description: "Просадка превысила 5%. Требуется внимание."

      - alert: OrderExecutionSlow
        expr: trading_order_execution_seconds > 2
        for: 30s
        labels:
          severity: warning
        annotations:
          summary: "Медленное исполнение ордеров"
          description: "Время исполнения ордера: {{ $value }} секунд"

      - alert: MarketDataStale
        expr: time() - trading_last_price_update_timestamp > 30
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Устаревшие рыночные данные"
          description: "Данные не обновлялись более 30 секунд"

      - alert: HighMemoryUsage
        expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Высокое использование памяти"
          description: "Контейнер использует более 85% памяти"
```

### Rust-код для экспорта метрик

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Метрики торгового бота для Prometheus
pub struct TradingMetrics {
    // Счётчики
    orders_total: AtomicU64,
    orders_filled: AtomicU64,
    orders_rejected: AtomicU64,

    // Гистограммы (упрощённая версия)
    execution_times_ms: Arc<std::sync::Mutex<Vec<u64>>>,

    // Текущие значения
    current_pnl: AtomicU64,        // Хранится как биты f64
    current_drawdown: AtomicU64,
    position_count: AtomicU64,

    // Время последнего обновления
    last_update: Arc<std::sync::Mutex<Instant>>,
}

impl TradingMetrics {
    pub fn new() -> Self {
        TradingMetrics {
            orders_total: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            execution_times_ms: Arc::new(std::sync::Mutex::new(Vec::new())),
            current_pnl: AtomicU64::new(0),
            current_drawdown: AtomicU64::new(0),
            position_count: AtomicU64::new(0),
            last_update: Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }

    /// Регистрация нового ордера
    pub fn record_order(&self, filled: bool, execution_time_ms: u64) {
        self.orders_total.fetch_add(1, Ordering::Relaxed);

        if filled {
            self.orders_filled.fetch_add(1, Ordering::Relaxed);
        } else {
            self.orders_rejected.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut times) = self.execution_times_ms.lock() {
            times.push(execution_time_ms);
            // Храним только последние 1000 измерений
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    /// Обновление PnL
    pub fn update_pnl(&self, pnl: f64) {
        self.current_pnl.store(pnl.to_bits(), Ordering::Relaxed);
        if let Ok(mut last) = self.last_update.lock() {
            *last = Instant::now();
        }
    }

    /// Обновление просадки
    pub fn update_drawdown(&self, drawdown_percent: f64) {
        self.current_drawdown.store(
            (drawdown_percent * 100.0) as u64,
            Ordering::Relaxed
        );
    }

    /// Генерация метрик в формате Prometheus
    pub fn to_prometheus_format(&self) -> String {
        let pnl = f64::from_bits(self.current_pnl.load(Ordering::Relaxed));
        let drawdown = self.current_drawdown.load(Ordering::Relaxed) as f64 / 100.0;

        let avg_execution = if let Ok(times) = self.execution_times_ms.lock() {
            if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<u64>() as f64 / times.len() as f64
            }
        } else {
            0.0
        };

        format!(
            r#"# HELP trading_orders_total Total number of orders
# TYPE trading_orders_total counter
trading_orders_total {}

# HELP trading_orders_filled Number of filled orders
# TYPE trading_orders_filled counter
trading_orders_filled {}

# HELP trading_orders_rejected Number of rejected orders
# TYPE trading_orders_rejected counter
trading_orders_rejected {}

# HELP trading_pnl_usd Current profit/loss in USD
# TYPE trading_pnl_usd gauge
trading_pnl_usd {:.2}

# HELP trading_drawdown_percent Current drawdown percentage
# TYPE trading_drawdown_percent gauge
trading_drawdown_percent {:.2}

# HELP trading_order_execution_avg_ms Average order execution time
# TYPE trading_order_execution_avg_ms gauge
trading_order_execution_avg_ms {:.2}
"#,
            self.orders_total.load(Ordering::Relaxed),
            self.orders_filled.load(Ordering::Relaxed),
            self.orders_rejected.load(Ordering::Relaxed),
            pnl,
            drawdown,
            avg_execution
        )
    }
}

fn main() {
    println!("=== Демонстрация метрик торгового бота ===\n");

    let metrics = TradingMetrics::new();

    // Имитация торговой активности
    for i in 0..10 {
        let filled = i % 3 != 0;  // 70% успешных ордеров
        let execution_time = 50 + (i * 10) as u64;
        metrics.record_order(filled, execution_time);
    }

    metrics.update_pnl(1234.56);
    metrics.update_drawdown(2.5);

    println!("{}", metrics.to_prometheus_format());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Docker Compose** | Оркестрация многоконтейнерных приложений |
| **Сервисы** | Определение контейнеров и их зависимостей |
| **Сети** | Изолированная коммуникация между сервисами |
| **Тома** | Персистентное хранение данных |
| **Health checks** | Проверка состояния сервисов |
| **Профили** | Конфигурации для разных окружений |
| **Переменные окружения** | Настройка без изменения кода |
| **Мониторинг** | Prometheus + Grafana для метрик |

## Практические задания

1. **Базовый торговый стек**: Создай docker-compose.yml с:
   - Rust-приложением торгового бота
   - PostgreSQL для хранения сделок
   - Redis для кэширования цен
   - Проверками здоровья для всех сервисов
   - Именованными томами для данных

2. **Мульти-окружение**: Настрой профили для:
   - Разработки с hot-reload и отладочными портами
   - Тестирования с изолированными базами
   - Продакшна с ограничениями ресурсов

3. **Мониторинг PnL**: Добавь:
   - Prometheus для сбора метрик
   - Grafana с дашбордом прибыли/убытков
   - Алерты при высокой просадке

4. **Сеть сервисов**: Создай микросервисную архитектуру:
   - Отдельный сервис рыночных данных
   - Сервис исполнения ордеров
   - Сервис риск-менеджмента
   - Общий Redis для коммуникации

## Домашнее задание

1. **Полный торговый стек**: Реализуй инфраструктуру с:
   - Минимум 3 Rust-микросервисами
   - PostgreSQL с репликацией
   - Redis кластер
   - Полным стеком мониторинга
   - Автоматическими бэкапами
   - Логированием в ELK/Loki

2. **CI/CD пайплайн**: Настрой:
   - Автоматическую сборку образов
   - Тесты в Docker-контейнерах
   - Развёртывание с docker compose
   - Откат при неудачных проверках здоровья
   - Уведомления в Telegram при проблемах

3. **Отказоустойчивость**: Добавь:
   - Репликацию PostgreSQL (primary/replica)
   - Redis Sentinel для отказоустойчивости
   - Nginx как балансировщик нагрузки
   - Автоматический рестарт упавших сервисов
   - Graceful shutdown для торговых сервисов

4. **Безопасность**: Реализуй:
   - Secrets management (Docker secrets или Vault)
   - Изолированные сети для разных сервисов
   - Read-only файловые системы где возможно
   - Пользователи без root-прав в контейнерах
   - Сканирование образов на уязвимости

## Навигация

[← Предыдущий день](../343-*/ru.md) | [Следующий день →](../345-*/ru.md)
