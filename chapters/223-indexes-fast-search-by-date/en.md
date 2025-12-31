# Day 223: Indexes: Fast Search by Date

## Trading Analogy

Imagine you're a trader and want to find all your trades from the last week. If all your trades are written in a thick notebook without any organization, you'd have to look through every page from start to finish. This is slow and inefficient.

Now imagine this notebook has a **table of contents by date** — an index that shows: "January — pages 1-30, February — pages 31-58...". Now searching becomes lightning fast: you can jump directly to the right section!

In databases, an **index** works exactly the same way — it's a special data structure that speeds up searches on a specific field. Date indexes are especially important in trading because:
- Historical price data is always tied to time
- Analysis often requires selecting data for a specific period
- P&L reports are built by date

## What is an Index?

A database index is a sorted data structure that stores pointers to table rows. The most common types of indexes are:

| Index Type | Description | Use Case |
|------------|-------------|----------|
| B-Tree | Balanced tree | Equality and ranges |
| Hash | Hash table | Exact equality only |
| GiST | Generalized Search Tree | Geodata, full-text search |
| BRIN | Block Range Index | Very large ordered tables |

For date searches, the **B-Tree index** is most commonly used as it works excellently with date ranges.

## Project Setup

To work with databases in Rust, we'll use the `sqlx` and `tokio` libraries:

```toml
# Cargo.toml
[package]
name = "trading-indexes"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Creating Tables with Trading Data

```rust
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,        // "buy" or "sell"
    executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct PriceCandle {
    id: i64,
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    timestamp: DateTime<Utc>,
}

async fn create_tables(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Trades table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS trades (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(20, 8) NOT NULL,
            quantity DECIMAL(20, 8) NOT NULL,
            side VARCHAR(4) NOT NULL,
            executed_at TIMESTAMPTZ NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    // Candles table (OHLCV)
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS candles (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            open_price DECIMAL(20, 8) NOT NULL,
            high_price DECIMAL(20, 8) NOT NULL,
            low_price DECIMAL(20, 8) NOT NULL,
            close_price DECIMAL(20, 8) NOT NULL,
            volume DECIMAL(20, 8) NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    println!("Tables created successfully!");
    Ok(())
}
```

## Creating Date Indexes

```rust
async fn create_indexes(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Simple index on trade execution date
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_executed_at
        ON trades (executed_at)
    "#)
    .execute(pool)
    .await?;
    println!("Index idx_trades_executed_at created");

    // Composite index: symbol + date (for filtering by instrument and time)
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_symbol_date
        ON trades (symbol, executed_at)
    "#)
    .execute(pool)
    .await?;
    println!("Index idx_trades_symbol_date created");

    // Date index for candles
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_candles_timestamp
        ON candles (timestamp)
    "#)
    .execute(pool)
    .await?;
    println!("Index idx_candles_timestamp created");

    // Composite index for candles: symbol + time
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_candles_symbol_time
        ON candles (symbol, timestamp)
    "#)
    .execute(pool)
    .await?;
    println!("Index idx_candles_symbol_time created");

    // Unique index to prevent duplicate candles
    sqlx::query(r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_candles_unique
        ON candles (symbol, timestamp)
    "#)
    .execute(pool)
    .await
    .ok(); // Ignore error if index already exists
    println!("Unique index idx_candles_unique created");

    Ok(())
}
```

## Seeding Test Data

```rust
use rand::Rng;

async fn seed_test_data(pool: &Pool<Postgres>, num_trades: i32) -> Result<(), sqlx::Error> {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT", "DOGE/USDT"];
    let base_prices = vec![42000.0, 2500.0, 100.0, 0.08];
    let sides = vec!["buy", "sell"];

    let mut rng = rand::thread_rng();
    let now = Utc::now();

    println!("Adding {} test trades...", num_trades);

    for i in 0..num_trades {
        let symbol_idx = rng.gen_range(0..symbols.len());
        let symbol = symbols[symbol_idx];
        let base_price = base_prices[symbol_idx];

        // Random price deviation ±5%
        let price = base_price * (1.0 + rng.gen_range(-0.05..0.05));
        let quantity = rng.gen_range(0.001..10.0);
        let side = sides[rng.gen_range(0..2)];

        // Distribute trades over the last 30 days
        let days_ago = rng.gen_range(0..30);
        let hours_ago = rng.gen_range(0..24);
        let executed_at = now - Duration::days(days_ago) - Duration::hours(hours_ago);

        sqlx::query(r#"
            INSERT INTO trades (symbol, price, quantity, side, executed_at)
            VALUES ($1, $2, $3, $4, $5)
        "#)
        .bind(symbol)
        .bind(price)
        .bind(quantity)
        .bind(side)
        .bind(executed_at)
        .execute(pool)
        .await?;

        if (i + 1) % 1000 == 0 {
            println!("  Added {} trades", i + 1);
        }
    }

    println!("Test data added successfully!");
    Ok(())
}
```

## Searching Using the Index

```rust
use std::time::Instant;

async fn find_trades_by_date_range(
    pool: &Pool<Postgres>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<Trade>, sqlx::Error> {
    let start = Instant::now();

    let trades: Vec<Trade> = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, price as "price: f64", quantity as "quantity: f64",
               side, executed_at as "executed_at: DateTime<Utc>"
        FROM trades
        WHERE executed_at >= $1 AND executed_at < $2
        ORDER BY executed_at DESC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let elapsed = start.elapsed();
    println!(
        "Found {} trades in {:?} (from {} to {})",
        trades.len(),
        elapsed,
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d")
    );

    Ok(trades)
}

async fn find_trades_by_symbol_and_date(
    pool: &Pool<Postgres>,
    symbol: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<Trade>, sqlx::Error> {
    let start = Instant::now();

    let rows = sqlx::query(
        r#"
        SELECT id, symbol, price, quantity, side, executed_at
        FROM trades
        WHERE symbol = $1 AND executed_at >= $2 AND executed_at < $3
        ORDER BY executed_at DESC
        "#
    )
    .bind(symbol)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let trades: Vec<Trade> = rows.iter().map(|row| {
        Trade {
            id: row.get("id"),
            symbol: row.get("symbol"),
            price: row.get::<sqlx::types::BigDecimal, _>("price")
                .to_string().parse().unwrap_or(0.0),
            quantity: row.get::<sqlx::types::BigDecimal, _>("quantity")
                .to_string().parse().unwrap_or(0.0),
            side: row.get("side"),
            executed_at: row.get("executed_at"),
        }
    }).collect();

    let elapsed = start.elapsed();
    println!(
        "Found {} trades for {} in {:?}",
        trades.len(),
        symbol,
        elapsed
    );

    Ok(trades)
}
```

## Analyzing Query Execution Plan

To verify that the index is being used, you can analyze the query execution plan:

```rust
async fn analyze_query_plan(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Query Plan Analysis ===\n");

    // Query WITHOUT index usage (full table scan)
    let explain_no_index: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE price > 40000"
    )
    .fetch_all(pool)
    .await?;

    println!("Query by price (no index):");
    for row in &explain_no_index {
        println!("  {}", row.0);
    }

    // Query WITH date index usage
    let explain_with_index: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE executed_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_all(pool)
    .await?;

    println!("\nQuery by date (with index):");
    for row in &explain_with_index {
        println!("  {}", row.0);
    }

    // Query with composite index
    let explain_composite: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE symbol = 'BTC/USDT' AND executed_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_all(pool)
    .await?;

    println!("\nQuery by symbol + date (composite index):");
    for row in &explain_composite {
        println!("  {}", row.0);
    }

    Ok(())
}
```

## Types of Date Indexes

### 1. B-Tree (default) — Universal Index

```rust
async fn create_btree_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // B-Tree is excellent for:
    // - Exact search: WHERE date = '2024-01-15'
    // - Ranges: WHERE date BETWEEN '2024-01-01' AND '2024-01-31'
    // - Sorting: ORDER BY date DESC

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_btree_date
        ON trades USING btree (executed_at)
    "#)
    .execute(pool)
    .await?;

    println!("B-Tree index created");
    Ok(())
}
```

### 2. BRIN — For Large Ordered Tables

```rust
async fn create_brin_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // BRIN (Block Range Index) is ideal when:
    // - Data is added sequentially (by time)
    // - Table is very large (millions of rows)
    // - Perfect precision is not needed

    // BRIN takes much less space than B-Tree!
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_brin_date
        ON trades USING brin (executed_at)
        WITH (pages_per_range = 128)
    "#)
    .execute(pool)
    .await?;

    println!("BRIN index created");
    Ok(())
}
```

### 3. Partial Index — Only Needed Data

```rust
async fn create_partial_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Partial index indexes only part of the data
    // Useful for "hot" data — recent trades

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_recent
        ON trades (executed_at)
        WHERE executed_at > CURRENT_DATE - INTERVAL '30 days'
    "#)
    .execute(pool)
    .await?;

    println!("Partial index for last 30 days created");
    Ok(())
}
```

## Practical Example: Trading Journal

```rust
use std::collections::HashMap;

struct TradingJournal {
    pool: Pool<Postgres>,
}

impl TradingJournal {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(TradingJournal { pool })
    }

    /// Get P&L for a period
    async fn get_pnl_by_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<f64, sqlx::Error> {
        let row = sqlx::query(r#"
            SELECT
                COALESCE(SUM(
                    CASE
                        WHEN side = 'sell' THEN price * quantity
                        ELSE -price * quantity
                    END
                ), 0) as pnl
            FROM trades
            WHERE executed_at >= $1 AND executed_at < $2
        "#)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let pnl: sqlx::types::BigDecimal = row.get("pnl");
        Ok(pnl.to_string().parse().unwrap_or(0.0))
    }

    /// Get daily statistics
    async fn get_daily_stats(
        &self,
        date: DateTime<Utc>,
    ) -> Result<DailyStats, sqlx::Error> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_utc();
        let end = start + Duration::days(1);

        let row = sqlx::query(r#"
            SELECT
                COUNT(*) as trade_count,
                COALESCE(SUM(quantity), 0) as total_volume,
                COALESCE(SUM(CASE WHEN side = 'buy' THEN 1 ELSE 0 END), 0) as buy_count,
                COALESCE(SUM(CASE WHEN side = 'sell' THEN 1 ELSE 0 END), 0) as sell_count
            FROM trades
            WHERE executed_at >= $1 AND executed_at < $2
        "#)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(DailyStats {
            date: start,
            trade_count: row.get::<i64, _>("trade_count") as u32,
            total_volume: row.get::<sqlx::types::BigDecimal, _>("total_volume")
                .to_string().parse().unwrap_or(0.0),
            buy_count: row.get::<i64, _>("buy_count") as u32,
            sell_count: row.get::<i64, _>("sell_count") as u32,
        })
    }

    /// Get trades by symbol for the last N days
    async fn get_recent_trades(
        &self,
        symbol: &str,
        days: i64,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(r#"
            SELECT id, symbol, price, quantity, side, executed_at
            FROM trades
            WHERE symbol = $1 AND executed_at >= $2
            ORDER BY executed_at DESC
            LIMIT 100
        "#)
        .bind(symbol)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let trades: Vec<Trade> = rows.iter().map(|row| {
            Trade {
                id: row.get("id"),
                symbol: row.get("symbol"),
                price: row.get::<sqlx::types::BigDecimal, _>("price")
                    .to_string().parse().unwrap_or(0.0),
                quantity: row.get::<sqlx::types::BigDecimal, _>("quantity")
                    .to_string().parse().unwrap_or(0.0),
                side: row.get("side"),
                executed_at: row.get("executed_at"),
            }
        }).collect();

        Ok(trades)
    }

    /// Group trades by day using the index
    async fn get_trades_grouped_by_day(
        &self,
        symbol: &str,
        days: i64,
    ) -> Result<HashMap<String, Vec<Trade>>, sqlx::Error> {
        let trades = self.get_recent_trades(symbol, days).await?;

        let mut grouped: HashMap<String, Vec<Trade>> = HashMap::new();
        for trade in trades {
            let date_key = trade.executed_at.format("%Y-%m-%d").to_string();
            grouped.entry(date_key).or_insert_with(Vec::new).push(trade);
        }

        Ok(grouped)
    }
}

#[derive(Debug)]
struct DailyStats {
    date: DateTime<Utc>,
    trade_count: u32,
    total_volume: f64,
    buy_count: u32,
    sell_count: u32,
}
```

## Optimizing Queries with Indexes

### 1. Avoid Functions in WHERE Clauses

```rust
// BAD: index is not used
// WHERE DATE(executed_at) = '2024-01-15'

// GOOD: index is used
async fn find_by_date_efficient(
    pool: &Pool<Postgres>,
    date_str: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(r#"
        SELECT COUNT(*) as count
        FROM trades
        WHERE executed_at >= $1::date
          AND executed_at < ($1::date + INTERVAL '1 day')
    "#)
    .bind(date_str)
    .fetch_one(pool)
    .await?;

    Ok(row.get("count"))
}
```

### 2. Use Correct Column Order in Composite Indexes

```rust
// Composite index (symbol, executed_at) is efficient for:
// - WHERE symbol = 'BTC' AND executed_at > '2024-01-01'
// - WHERE symbol = 'BTC'

// But NOT efficient for:
// - WHERE executed_at > '2024-01-01' (without symbol)

async fn demonstrate_index_order(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // This query uses the composite index efficiently
    let _ = sqlx::query(
        "SELECT COUNT(*) FROM trades WHERE symbol = $1 AND executed_at > $2"
    )
    .bind("BTC/USDT")
    .bind(Utc::now() - Duration::days(7))
    .fetch_one(pool)
    .await?;

    // For queries only by date, a separate index is needed
    let _ = sqlx::query(
        "SELECT COUNT(*) FROM trades WHERE executed_at > $1"
    )
    .bind(Utc::now() - Duration::days(7))
    .fetch_one(pool)
    .await?;

    Ok(())
}
```

## Monitoring Indexes

```rust
async fn check_index_usage(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Index Usage Statistics ===\n");

    let rows = sqlx::query(r#"
        SELECT
            schemaname,
            tablename,
            indexname,
            idx_scan as index_scans,
            idx_tup_read as tuples_read,
            idx_tup_fetch as tuples_fetched
        FROM pg_stat_user_indexes
        WHERE tablename IN ('trades', 'candles')
        ORDER BY idx_scan DESC
    "#)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let index_name: &str = row.get("indexname");
        let scans: i64 = row.get("index_scans");
        let tuples: i64 = row.get("tuples_read");

        println!(
            "Index: {} | Scans: {} | Tuples read: {}",
            index_name, scans, tuples
        );
    }

    Ok(())
}

async fn check_index_sizes(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Index Sizes ===\n");

    let rows = sqlx::query(r#"
        SELECT
            indexname,
            pg_size_pretty(pg_relation_size(indexname::regclass)) as index_size
        FROM pg_indexes
        WHERE tablename IN ('trades', 'candles')
    "#)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let name: &str = row.get("indexname");
        let size: &str = row.get("index_size");
        println!("Index: {} | Size: {}", name, size);
    }

    Ok(())
}
```

## Complete Example

```rust
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use chrono::{DateTime, Utc, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/trading".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Database connection established!\n");

    // Create tables
    create_tables(&pool).await?;

    // Create indexes
    create_indexes(&pool).await?;

    // Seed test data (10000 trades)
    seed_test_data(&pool, 10000).await?;

    // Test searching
    println!("\n=== Testing Search ===\n");

    let end = Utc::now();
    let start = end - Duration::days(7);

    // Search by date range
    let trades = find_trades_by_date_range(&pool, start, end).await?;
    println!("First 3 trades:");
    for trade in trades.iter().take(3) {
        println!("  {:?}", trade);
    }

    // Search by symbol and date
    let btc_trades = find_trades_by_symbol_and_date(
        &pool, "BTC/USDT", start, end
    ).await?;
    println!("\nFirst 3 BTC trades:");
    for trade in btc_trades.iter().take(3) {
        println!("  {:?}", trade);
    }

    // Analyze query plans
    analyze_query_plan(&pool).await?;

    // Index statistics
    check_index_usage(&pool).await?;
    check_index_sizes(&pool).await?;

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Index | Data structure for speeding up searches |
| B-Tree | Universal index for equality and ranges |
| BRIN | Compact index for large ordered data |
| Composite Index | Index on multiple columns |
| Partial Index | Index for only part of the data |
| EXPLAIN ANALYZE | Query execution plan analysis |

## Homework

1. **Performance Comparison**: Create a table with 1 million rows and compare query execution time for a date range with and without an index. Use `EXPLAIN ANALYZE` for analysis.

2. **Composite Index**: Create a composite index `(symbol, executed_at, side)` and determine which queries it will be efficient for and which it won't.

3. **BRIN vs B-Tree**: Create both versions of the index on a table with one million rows. Compare:
   - Index size
   - Query execution time
   - When to use which index

4. **Trading Analytics**: Using indexes, write functions for:
   - Finding the most profitable trade of the month
   - Counting trading volume by hour
   - Finding "quiet periods" — gaps without trades for more than 1 hour

## Navigation

[← Previous day](../222-transactions-atomic-operations/en.md) | [Next day →](../224-migrations-schema-evolution/en.md)
