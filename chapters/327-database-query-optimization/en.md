# Day 327: Database Query Optimization

## Trading Analogy

Imagine you're analyzing your trade history for the past year. You have millions of records: prices, volumes, timestamps. If every time you analyze you iterate through all records sequentially — it's like searching for a specific trade by flipping through all statements manually. **Slow and inefficient.**

Database query optimization is like creating a smart catalog system for your trading data:
- **Indexes** — like bookmarks in your trade journal, allowing instant access to specific records
- **Prepared statements** — like order templates that don't need to be composed from scratch each time
- **Batch operations** — like sending orders in groups instead of individual requests
- **Caching** — like remembering frequently used quotes

In high-frequency trading, every millisecond counts. Unoptimized database queries mean missed trading opportunities.

## Database Basics in Rust

### Database Ecosystem

| Library | Description | Trading Application |
|---------|-------------|---------------------|
| `sqlx` | Async, compile-time SQL verification | Core trading data |
| `diesel` | Type-safe ORM | Complex business models |
| `rusqlite` | SQLite for local data | Local quote cache |
| `tokio-postgres` | Async PostgreSQL | High-load systems |
| `redis` | Cache and pub/sub | Real-time quotes |

### Cargo.toml for Examples

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

## The N+1 Query Problem

### Bad Pattern: N+1 Queries

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

/// BAD: N+1 queries — separate query for fills for each trade
async fn get_trades_with_fills_slow(pool: &PgPool) -> Result<Vec<(Trade, Vec<OrderFill>)>, sqlx::Error> {
    // 1 query for all trades
    let trades: Vec<Trade> = sqlx::query_as("SELECT * FROM trades ORDER BY timestamp DESC LIMIT 100")
        .fetch_all(pool)
        .await?;

    let mut result = Vec::with_capacity(trades.len());

    // N queries — one for each trade
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

### Solution: JOIN or Batch Query

```rust
#[derive(Debug, FromRow)]
struct TradeWithFill {
    // Trade fields
    trade_id: Uuid,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,
    // Fill fields (nullable if no fills)
    fill_id: Option<Uuid>,
    fill_price: Option<f64>,
    fill_quantity: Option<f64>,
}

/// GOOD: Single query with JOIN
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

/// Alternative: batch query with IN
async fn get_fills_batch(pool: &PgPool, trade_ids: &[Uuid]) -> Result<Vec<OrderFill>, sqlx::Error> {
    // Single query for all trade_ids
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

## Indexes for Trading Data

### Index Types and Their Applications

```sql
-- Creating trades table with optimal indexes
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

-- B-Tree index: for exact matches and ranges
CREATE INDEX idx_trades_symbol ON trades(symbol);
CREATE INDEX idx_trades_timestamp ON trades(timestamp DESC);

-- Composite index: for frequent filter combinations
CREATE INDEX idx_trades_symbol_timestamp ON trades(symbol, timestamp DESC);

-- Partial index: only for subset of interest
CREATE INDEX idx_trades_profitable ON trades(symbol, pnl) WHERE pnl > 0;

-- BRIN index: for time series (more compact than B-Tree)
CREATE INDEX idx_trades_timestamp_brin ON trades USING BRIN(timestamp);
```

### Monitoring Index Usage in Rust

```rust
#[derive(Debug, FromRow)]
struct IndexUsage {
    indexrelname: String,
    idx_scan: i64,
    idx_tup_read: i64,
    idx_tup_fetch: i64,
}

/// Check index efficiency
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

/// Analyze slow queries
async fn analyze_slow_queries(pool: &PgPool) -> Result<Vec<SlowQuery>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            query,
            calls,
            mean_exec_time,
            total_exec_time
        FROM pg_stat_statements
        WHERE mean_exec_time > 100  -- more than 100ms
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

## Prepared Statements and Connection Pooling

### Reusing Prepared Statements

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Optimal connection pool configuration
async fn create_optimized_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)  // No more than CPU cores * 2-3
        .min_connections(5)   // Keep connections ready
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Caching prepared statements
pub struct TradingQueries {
    pool: PgPool,
}

impl TradingQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Frequently used query — statement is cached automatically by sqlx
    pub async fn get_latest_price(&self, symbol: &str) -> Result<Option<f64>, sqlx::Error> {
        // sqlx automatically caches prepared statement by query text
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

    /// Query with parameterized candle count
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

## Batch Operations

### Bulk Insert for Trading Data

```rust
use std::time::Instant;

/// Slow insert: one record at a time
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

/// Fast batch insert with UNNEST
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

/// Even faster: COPY for very large volumes
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

/// Performance comparison
async fn benchmark_inserts(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Generate test data
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

    // Slow method
    let start = Instant::now();
    // insert_trades_slow(pool, &trades[..100]).await?; // Only 100 for demo
    println!("Slow insert (100 records): {:?}", start.elapsed());

    // Fast method
    let start = Instant::now();
    let rows = insert_trades_fast(pool, &trades).await?;
    println!("Fast insert ({} records): {:?}", rows, start.elapsed());

    Ok(())
}
```

## Read Optimization: Partitioning and Materialized Views

### Time-Based Partitioning

```sql
-- Creating partitioned table for trades
CREATE TABLE trades_partitioned (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- Monthly partitions
CREATE TABLE trades_2024_01 PARTITION OF trades_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE trades_2024_02 PARTITION OF trades_partitioned
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Automatic partition creation (via pg_partman or manually)
```

### Materialized Views for Aggregations

```sql
-- Materialized view for daily OHLCV
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

-- Index for fast access
CREATE UNIQUE INDEX ON daily_candles (symbol, day);

-- Refresh (can be scheduled via cron or pg_cron)
REFRESH MATERIALIZED VIEW CONCURRENTLY daily_candles;
```

### Working with Partitions in Rust

```rust
/// Query partitioned table with explicit range
async fn get_trades_for_month(
    pool: &PgPool,
    symbol: &str,
    year: i32,
    month: u32,
) -> Result<Vec<Trade>, sqlx::Error> {
    // PostgreSQL automatically selects the correct partition
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

/// Fast query to materialized view
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

## Query Result Caching

### Multi-Level Cache for Quotes

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Cache entry with TTL
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

/// Quote cache with TTL
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

    /// Get price from cache or load from DB
    pub async fn get_or_load(
        &self,
        symbol: &str,
        loader: impl std::future::Future<Output = Result<f64, sqlx::Error>>,
    ) -> Result<f64, sqlx::Error> {
        // Try to read from cache
        {
            let cache = self.data.read().await;
            if let Some(entry) = cache.get(symbol) {
                if !entry.is_expired() {
                    return Ok(entry.value);
                }
            }
        }

        // Load from DB
        let value = loader.await?;

        // Save to cache
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

    /// Invalidate cache for symbol
    pub async fn invalidate(&self, symbol: &str) {
        let mut cache = self.data.write().await;
        cache.remove(symbol);
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.data.write().await;
        cache.retain(|_, entry| !entry.is_expired());
    }
}

/// Quote service with caching
pub struct QuoteService {
    pool: PgPool,
    cache: PriceCache,
}

impl QuoteService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: PriceCache::new(Duration::from_secs(1)), // 1 second TTL
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

## Async Queries and Parallelism

### Parallel Execution of Independent Queries

```rust
use tokio::try_join;

/// Get dashboard data — multiple independent queries in parallel
async fn get_trading_dashboard(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<TradingDashboard, sqlx::Error> {
    // All queries execute in parallel
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

## Query Monitoring and Profiling

### Query Logging Wrapper

```rust
use std::time::Instant;
use tracing::{info, warn, instrument};

/// Slow query logging
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

/// Query metrics
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

## What We Learned

| Concept | Description |
|---------|-------------|
| **N+1 problem** | Multiple small queries instead of one large query |
| **Indexes** | Speed up searches with additional data structures |
| **Prepared statements** | Reuse compiled queries |
| **Batch operations** | Insert/update multiple records in one query |
| **Partitioning** | Split table into parts for faster access |
| **Materialized views** | Pre-computed aggregates |
| **Caching** | Store frequently requested data in memory |
| **Parallel queries** | Execute independent queries simultaneously |

## Practical Exercises

1. **Profile Real Queries**: Connect `pg_stat_statements` to your database and find the top 10 slowest queries. Optimize at least 3 of them.

2. **Index Benchmark**: Create a table with 1 million trading records. Compare query execution time with and without indexes. Measure the impact of different index types (B-Tree, BRIN, Hash).

3. **Cache Implementation**: Create a multi-level cache for trading data:
   - L1: In-memory cache with 1 second TTL (for hot data)
   - L2: Redis with 1 minute TTL
   - L3: PostgreSQL

4. **Batch Import**: Implement historical data import (1 million candles) using COPY. Compare with row-by-row insertion.

## Homework

1. **Backtesting Optimization**: Design a database schema for storing backtesting results (trades, metrics, parameters). Optimize for:
   - Fast addition of new results
   - Fast search for best strategies by metrics
   - Comparison of results across different parameters

2. **Real-time Aggregations**: Implement a system for real-time trade aggregation into candles of different timeframes (1m, 5m, 15m, 1h, 1d). Use:
   - Materialized views or
   - TimescaleDB continuous aggregates or
   - Redis caching

3. **Performance Monitoring**: Create a dashboard for monitoring:
   - Average query time
   - Number of slow queries
   - Cache hit rate
   - Index usage
   Use Prometheus + Grafana or alternatives.

4. **High-Load Optimization**: Conduct load testing of your trading system:
   - 10,000 requests/sec for reading quotes
   - 1,000 trade inserts/sec
   Find bottlenecks and eliminate them.

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../328-*/en.md)
