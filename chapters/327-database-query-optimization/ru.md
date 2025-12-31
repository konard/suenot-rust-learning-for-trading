# День 327: Оптимизация запросов к БД

## Аналогия из трейдинга

Представь, что ты анализируешь историю сделок за последний год. У тебя миллионы записей: цены, объёмы, временные метки. Если каждый раз при анализе ты будешь перебирать все записи подряд — это как искать конкретную сделку, листая все выписки вручную. **Медленно и неэффективно.**

Оптимизация запросов к БД — это как создание умной системы каталогов для твоих торговых данных:
- **Индексы** — как закладки в книге сделок, позволяющие мгновенно находить нужные записи
- **Prepared statements** — как шаблоны ордеров, которые не нужно каждый раз составлять заново
- **Пакетные операции** — как групповая отправка ордеров вместо отдельных запросов
- **Кэширование** — как запоминание часто используемых котировок

В высокочастотной торговле каждая миллисекунда на счету. Неоптимизированные запросы к базе — это упущенные торговые возможности.

## Основы работы с БД в Rust

### Экосистема баз данных

| Библиотека | Описание | Применение в трейдинге |
|------------|----------|------------------------|
| `sqlx` | Асинхронный, compile-time проверка SQL | Основные торговые данные |
| `diesel` | ORM с типобезопасностью | Сложные бизнес-модели |
| `rusqlite` | SQLite для локальных данных | Локальный кэш котировок |
| `tokio-postgres` | Асинхронный PostgreSQL | Высоконагруженные системы |
| `redis` | Кэш и pub/sub | Реал-тайм котировки |

### Cargo.toml для примеров

```toml
[package]
name = "trading_db_optimization"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
serde = { version = "1", features = ["derive"] }
```

## Проблема N+1 запросов

### Плохой паттерн: N+1 запросы

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct Trade {
    id: Uuid,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct OrderFill {
    id: Uuid,
    trade_id: Uuid,
    fill_price: f64,
    fill_quantity: f64,
}

/// ПЛОХО: N+1 запросов — для каждой сделки отдельный запрос на fills
async fn get_trades_with_fills_slow(pool: &PgPool) -> Result<Vec<(Trade, Vec<OrderFill>)>, sqlx::Error> {
    // 1 запрос на все сделки
    let trades: Vec<Trade> = sqlx::query_as("SELECT * FROM trades ORDER BY timestamp DESC LIMIT 100")
        .fetch_all(pool)
        .await?;

    let mut result = Vec::with_capacity(trades.len());

    // N запросов — по одному на каждую сделку
    for trade in trades {
        let fills: Vec<OrderFill> = sqlx::query_as(
            "SELECT * FROM order_fills WHERE trade_id = $1"
        )
        .bind(&trade.id)
        .fetch_all(pool)
        .await?;

        result.push((trade, fills));
    }

    Ok(result)
}
```

### Решение: JOIN или пакетный запрос

```rust
#[derive(Debug, FromRow)]
struct TradeWithFill {
    // Trade fields
    trade_id: Uuid,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,
    // Fill fields (nullable, если нет fills)
    fill_id: Option<Uuid>,
    fill_price: Option<f64>,
    fill_quantity: Option<f64>,
}

/// ХОРОШО: Один запрос с JOIN
async fn get_trades_with_fills_fast(pool: &PgPool) -> Result<Vec<TradeWithFill>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            t.id as trade_id,
            t.symbol,
            t.price,
            t.quantity,
            t.timestamp,
            f.id as fill_id,
            f.fill_price,
            f.fill_quantity
        FROM trades t
        LEFT JOIN order_fills f ON t.id = f.trade_id
        ORDER BY t.timestamp DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool)
    .await
}

/// Альтернатива: пакетный запрос с IN
async fn get_fills_batch(pool: &PgPool, trade_ids: &[Uuid]) -> Result<Vec<OrderFill>, sqlx::Error> {
    // Один запрос для всех trade_id
    sqlx::query_as(
        r#"
        SELECT * FROM order_fills
        WHERE trade_id = ANY($1)
        ORDER BY trade_id
        "#
    )
    .bind(trade_ids)
    .fetch_all(pool)
    .await
}
```

## Индексы для торговых данных

### Типы индексов и их применение

```sql
-- Создание таблицы сделок с оптимальными индексами
CREATE TABLE trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL CHECK (side IN ('BUY', 'SELL')),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    strategy_id UUID,
    pnl DECIMAL(20, 8)
);

-- B-Tree индекс: для точных совпадений и диапазонов
CREATE INDEX idx_trades_symbol ON trades(symbol);
CREATE INDEX idx_trades_timestamp ON trades(timestamp DESC);

-- Составной индекс: для частых комбинаций фильтров
CREATE INDEX idx_trades_symbol_timestamp ON trades(symbol, timestamp DESC);

-- Частичный индекс: только для интересующего подмножества
CREATE INDEX idx_trades_profitable ON trades(symbol, pnl) WHERE pnl > 0;

-- BRIN индекс: для временных рядов (компактнее B-Tree)
CREATE INDEX idx_trades_timestamp_brin ON trades USING BRIN(timestamp);
```

### Мониторинг использования индексов в Rust

```rust
#[derive(Debug, FromRow)]
struct IndexUsage {
    indexrelname: String,
    idx_scan: i64,
    idx_tup_read: i64,
    idx_tup_fetch: i64,
}

/// Проверка эффективности индексов
async fn check_index_usage(pool: &PgPool, table_name: &str) -> Result<Vec<IndexUsage>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            indexrelname,
            idx_scan,
            idx_tup_read,
            idx_tup_fetch
        FROM pg_stat_user_indexes
        WHERE relname = $1
        ORDER BY idx_scan DESC
        "#
    )
    .bind(table_name)
    .fetch_all(pool)
    .await
}

/// Анализ медленных запросов
async fn analyze_slow_queries(pool: &PgPool) -> Result<Vec<SlowQuery>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            query,
            calls,
            mean_exec_time,
            total_exec_time
        FROM pg_stat_statements
        WHERE mean_exec_time > 100  -- более 100ms
        ORDER BY total_exec_time DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, FromRow)]
struct SlowQuery {
    query: String,
    calls: i64,
    mean_exec_time: f64,
    total_exec_time: f64,
}
```

## Prepared Statements и пул соединений

### Переиспользование prepared statements

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Оптимальная конфигурация пула соединений
async fn create_optimized_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)  // Не больше CPU cores * 2-3
        .min_connections(5)   // Держим готовые соединения
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Кэширование prepared statements
pub struct TradingQueries {
    pool: PgPool,
}

impl TradingQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Часто используемый запрос — statement кэшируется автоматически в sqlx
    pub async fn get_latest_price(&self, symbol: &str) -> Result<Option<f64>, sqlx::Error> {
        // sqlx автоматически кэширует prepared statement по тексту запроса
        sqlx::query_scalar(
            r#"
            SELECT price FROM trades
            WHERE symbol = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#
        )
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
    }

    /// Запрос с параметризованным количеством свечей
    pub async fn get_candles(
        &self,
        symbol: &str,
        timeframe_seconds: i32,
        limit: i32
    ) -> Result<Vec<Candle>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                time_bucket($2 * INTERVAL '1 second', timestamp) as bucket,
                FIRST(price, timestamp) as open,
                MAX(price) as high,
                MIN(price) as low,
                LAST(price, timestamp) as close,
                SUM(quantity) as volume
            FROM trades
            WHERE symbol = $1
            GROUP BY bucket
            ORDER BY bucket DESC
            LIMIT $3
            "#
        )
        .bind(symbol)
        .bind(timeframe_seconds)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

#[derive(Debug, FromRow)]
struct Candle {
    bucket: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}
```

## Пакетные операции

### Bulk Insert для торговых данных

```rust
use std::time::Instant;

/// Медленная вставка: по одной записи
async fn insert_trades_slow(pool: &PgPool, trades: &[Trade]) -> Result<(), sqlx::Error> {
    for trade in trades {
        sqlx::query(
            "INSERT INTO trades (symbol, side, price, quantity, timestamp) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&trade.symbol)
        .bind(&trade.side)
        .bind(trade.price)
        .bind(trade.quantity)
        .bind(trade.timestamp)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Быстрая пакетная вставка с UNNEST
async fn insert_trades_fast(pool: &PgPool, trades: &[Trade]) -> Result<u64, sqlx::Error> {
    if trades.is_empty() {
        return Ok(0);
    }

    let symbols: Vec<&str> = trades.iter().map(|t| t.symbol.as_str()).collect();
    let sides: Vec<&str> = trades.iter().map(|t| t.side.as_str()).collect();
    let prices: Vec<f64> = trades.iter().map(|t| t.price).collect();
    let quantities: Vec<f64> = trades.iter().map(|t| t.quantity).collect();
    let timestamps: Vec<DateTime<Utc>> = trades.iter().map(|t| t.timestamp).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO trades (symbol, side, price, quantity, timestamp)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::float8[], $4::float8[], $5::timestamptz[])
        "#
    )
    .bind(&symbols)
    .bind(&sides)
    .bind(&prices)
    .bind(&quantities)
    .bind(&timestamps)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Ещё быстрее: COPY для очень больших объёмов
async fn insert_trades_copy(pool: &PgPool, trades: &[Trade]) -> Result<u64, sqlx::Error> {
    use sqlx::postgres::PgCopyIn;

    let mut copy = pool.copy_in_raw(
        "COPY trades (symbol, side, price, quantity, timestamp) FROM STDIN WITH (FORMAT csv)"
    ).await?;

    for trade in trades {
        let line = format!(
            "{},{},{},{},{}\n",
            trade.symbol,
            trade.side,
            trade.price,
            trade.quantity,
            trade.timestamp.to_rfc3339()
        );
        copy.send(line.as_bytes()).await?;
    }

    let rows = copy.finish().await?;
    Ok(rows)
}

/// Сравнение производительности
async fn benchmark_inserts(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Генерируем тестовые данные
    let trades: Vec<Trade> = (0..10_000)
        .map(|i| Trade {
            id: Uuid::new_v4(),
            symbol: "BTC/USD".to_string(),
            side: if i % 2 == 0 { "BUY" } else { "SELL" }.to_string(),
            price: 42000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.001),
            timestamp: Utc::now(),
        })
        .collect();

    // Медленный способ
    let start = Instant::now();
    // insert_trades_slow(pool, &trades[..100]).await?; // Только 100 для демонстрации
    println!("Slow insert (100 records): {:?}", start.elapsed());

    // Быстрый способ
    let start = Instant::now();
    let rows = insert_trades_fast(pool, &trades).await?;
    println!("Fast insert ({} records): {:?}", rows, start.elapsed());

    Ok(())
}
```

## Оптимизация чтения: партиционирование и материализованные представления

### Партиционирование по времени

```sql
-- Создание партиционированной таблицы для сделок
CREATE TABLE trades_partitioned (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- Партиции по месяцам
CREATE TABLE trades_2024_01 PARTITION OF trades_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE trades_2024_02 PARTITION OF trades_partitioned
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Автоматическое создание партиций (через pg_partman или вручную)
```

### Материализованные представления для агрегаций

```sql
-- Материализованное представление для дневных OHLCV
CREATE MATERIALIZED VIEW daily_candles AS
SELECT
    symbol,
    date_trunc('day', timestamp) as day,
    (array_agg(price ORDER BY timestamp ASC))[1] as open,
    MAX(price) as high,
    MIN(price) as low,
    (array_agg(price ORDER BY timestamp DESC))[1] as close,
    SUM(quantity) as volume,
    COUNT(*) as trade_count
FROM trades
GROUP BY symbol, date_trunc('day', timestamp);

-- Индекс для быстрого доступа
CREATE UNIQUE INDEX ON daily_candles (symbol, day);

-- Обновление (можно по расписанию через cron или pg_cron)
REFRESH MATERIALIZED VIEW CONCURRENTLY daily_candles;
```

### Работа с партициями в Rust

```rust
/// Запрос к партиционированной таблице с явным указанием диапазона
async fn get_trades_for_month(
    pool: &PgPool,
    symbol: &str,
    year: i32,
    month: u32,
) -> Result<Vec<Trade>, sqlx::Error> {
    // PostgreSQL автоматически выбирает нужную партицию
    sqlx::query_as(
        r#"
        SELECT * FROM trades_partitioned
        WHERE symbol = $1
          AND timestamp >= make_date($2, $3, 1)
          AND timestamp < make_date($2, $3, 1) + INTERVAL '1 month'
        ORDER BY timestamp
        "#
    )
    .bind(symbol)
    .bind(year)
    .bind(month as i32)
    .fetch_all(pool)
    .await
}

/// Быстрый запрос к материализованному представлению
async fn get_daily_candles(
    pool: &PgPool,
    symbol: &str,
    days: i32,
) -> Result<Vec<DailyCandle>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT * FROM daily_candles
        WHERE symbol = $1
          AND day >= NOW() - $2 * INTERVAL '1 day'
        ORDER BY day DESC
        "#
    )
    .bind(symbol)
    .bind(days)
    .fetch_all(pool)
    .await
}

#[derive(Debug, FromRow)]
struct DailyCandle {
    symbol: String,
    day: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_count: i64,
}
```

## Кэширование результатов запросов

### Многоуровневый кэш для котировок

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Запись в кэше с временем жизни
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Кэш котировок с TTL
pub struct PriceCache {
    data: Arc<RwLock<HashMap<String, CacheEntry<f64>>>>,
    default_ttl: Duration,
}

impl PriceCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Получить цену из кэша или загрузить из БД
    pub async fn get_or_load(
        &self,
        symbol: &str,
        loader: impl std::future::Future<Output = Result<f64, sqlx::Error>>,
    ) -> Result<f64, sqlx::Error> {
        // Пробуем прочитать из кэша
        {
            let cache = self.data.read().await;
            if let Some(entry) = cache.get(symbol) {
                if !entry.is_expired() {
                    return Ok(entry.value);
                }
            }
        }

        // Загружаем из БД
        let value = loader.await?;

        // Сохраняем в кэш
        {
            let mut cache = self.data.write().await;
            cache.insert(
                symbol.to_string(),
                CacheEntry {
                    value,
                    created_at: Instant::now(),
                    ttl: self.default_ttl,
                },
            );
        }

        Ok(value)
    }

    /// Инвалидировать кэш для символа
    pub async fn invalidate(&self, symbol: &str) {
        let mut cache = self.data.write().await;
        cache.remove(symbol);
    }

    /// Очистить просроченные записи
    pub async fn cleanup_expired(&self) {
        let mut cache = self.data.write().await;
        cache.retain(|_, entry| !entry.is_expired());
    }
}

/// Сервис котировок с кэшированием
pub struct QuoteService {
    pool: PgPool,
    cache: PriceCache,
}

impl QuoteService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: PriceCache::new(Duration::from_secs(1)), // TTL 1 секунда
        }
    }

    pub async fn get_price(&self, symbol: &str) -> Result<f64, sqlx::Error> {
        let pool = self.pool.clone();
        let sym = symbol.to_string();

        self.cache.get_or_load(
            symbol,
            async move {
                sqlx::query_scalar(
                    "SELECT price FROM trades WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
                )
                .bind(&sym)
                .fetch_one(&pool)
                .await
            }
        ).await
    }
}
```

## Асинхронные запросы и параллелизм

### Параллельное выполнение независимых запросов

```rust
use tokio::try_join;

/// Получение dashboard данных — несколько независимых запросов параллельно
async fn get_trading_dashboard(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<TradingDashboard, sqlx::Error> {
    // Все запросы выполняются параллельно
    let (positions, recent_trades, daily_pnl, open_orders) = try_join!(
        get_open_positions(pool, user_id),
        get_recent_trades(pool, user_id, 10),
        get_daily_pnl(pool, user_id),
        get_open_orders(pool, user_id),
    )?;

    Ok(TradingDashboard {
        positions,
        recent_trades,
        daily_pnl,
        open_orders,
    })
}

async fn get_open_positions(pool: &PgPool, user_id: Uuid) -> Result<Vec<Position>, sqlx::Error> {
    sqlx::query_as(
        "SELECT * FROM positions WHERE user_id = $1 AND quantity != 0"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

async fn get_recent_trades(pool: &PgPool, user_id: Uuid, limit: i32) -> Result<Vec<Trade>, sqlx::Error> {
    sqlx::query_as(
        "SELECT * FROM trades WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2"
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn get_daily_pnl(pool: &PgPool, user_id: Uuid) -> Result<f64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(pnl), 0) FROM trades
        WHERE user_id = $1 AND timestamp >= CURRENT_DATE
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

async fn get_open_orders(pool: &PgPool, user_id: Uuid) -> Result<Vec<Order>, sqlx::Error> {
    sqlx::query_as(
        "SELECT * FROM orders WHERE user_id = $1 AND status = 'OPEN'"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[derive(Debug)]
struct TradingDashboard {
    positions: Vec<Position>,
    recent_trades: Vec<Trade>,
    daily_pnl: f64,
    open_orders: Vec<Order>,
}

#[derive(Debug, FromRow)]
struct Position {
    symbol: String,
    quantity: f64,
    average_price: f64,
}

#[derive(Debug, FromRow)]
struct Order {
    id: Uuid,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}
```

## Мониторинг и профилирование запросов

### Обёртка для логирования запросов

```rust
use std::time::Instant;
use tracing::{info, warn, instrument};

/// Логирование медленных запросов
pub struct QueryLogger {
    slow_threshold: Duration,
}

impl QueryLogger {
    pub fn new(slow_threshold: Duration) -> Self {
        Self { slow_threshold }
    }

    pub async fn execute<T, F, Fut>(&self, name: &str, query: F) -> Result<T, sqlx::Error>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        let start = Instant::now();
        let result = query().await;
        let elapsed = start.elapsed();

        if elapsed > self.slow_threshold {
            warn!(
                query = name,
                duration_ms = elapsed.as_millis(),
                "Slow query detected"
            );
        } else {
            info!(
                query = name,
                duration_ms = elapsed.as_millis(),
                "Query executed"
            );
        }

        result
    }
}

/// Метрики запросов
pub struct QueryMetrics {
    query_count: std::sync::atomic::AtomicU64,
    total_duration_us: std::sync::atomic::AtomicU64,
    slow_query_count: std::sync::atomic::AtomicU64,
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self {
            query_count: std::sync::atomic::AtomicU64::new(0),
            total_duration_us: std::sync::atomic::AtomicU64::new(0),
            slow_query_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record(&self, duration: Duration, is_slow: bool) {
        use std::sync::atomic::Ordering;

        self.query_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);

        if is_slow {
            self.slow_query_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn average_duration(&self) -> Duration {
        use std::sync::atomic::Ordering;

        let count = self.query_count.load(Ordering::Relaxed);
        if count == 0 {
            return Duration::ZERO;
        }

        let total_us = self.total_duration_us.load(Ordering::Relaxed);
        Duration::from_micros(total_us / count)
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **N+1 проблема** | Множество мелких запросов вместо одного большого |
| **Индексы** | Ускорение поиска за счёт дополнительных структур данных |
| **Prepared statements** | Переиспользование скомпилированных запросов |
| **Пакетные операции** | Вставка/обновление множества записей одним запросом |
| **Партиционирование** | Разбиение таблицы на части для быстрого доступа |
| **Материализованные представления** | Предварительно вычисленные агрегаты |
| **Кэширование** | Хранение часто запрашиваемых данных в памяти |
| **Параллельные запросы** | Одновременное выполнение независимых запросов |

## Практические задания

1. **Профилирование реальных запросов**: Подключи `pg_stat_statements` к своей БД и найди топ-10 самых медленных запросов. Оптимизируй хотя бы 3 из них.

2. **Benchmark индексов**: Создай таблицу с 1 миллионом торговых записей. Сравни время выполнения запросов с индексами и без них. Замерь влияние разных типов индексов (B-Tree, BRIN, Hash).

3. **Реализация кэша**: Создай многоуровневый кэш для торговых данных:
   - L1: In-memory кэш с TTL 1 секунда (для hot data)
   - L2: Redis с TTL 1 минута
   - L3: PostgreSQL

4. **Пакетный импорт**: Реализуй импорт исторических данных (1 миллион свечей) используя COPY. Сравни с построчной вставкой.

## Домашнее задание

1. **Оптимизация бэктестинга**: Спроектируй схему БД для хранения результатов бэктестинга (сделки, метрики, параметры). Оптимизируй для:
   - Быстрого добавления новых результатов
   - Быстрого поиска лучших стратегий по метрикам
   - Сравнения результатов разных параметров

2. **Real-time агрегации**: Реализуй систему real-time агрегации сделок в свечи разных таймфреймов (1m, 5m, 15m, 1h, 1d). Используй:
   - Материализованные представления или
   - TimescaleDB continuous aggregates или
   - Кэширование в Redis

3. **Мониторинг производительности**: Создай dashboard для мониторинга:
   - Среднее время запросов
   - Количество медленных запросов
   - Hit rate кэша
   - Использование индексов
   Используй Prometheus + Grafana или аналоги.

4. **Оптимизация под высокую нагрузку**: Проведи нагрузочное тестирование своей торговой системы:
   - 10,000 запросов/сек на чтение котировок
   - 1,000 вставок сделок/сек
   Найди bottlenecks и устрани их.

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../328-*/ru.md)
