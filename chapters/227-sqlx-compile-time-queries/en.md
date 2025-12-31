# Day 227: sqlx: Compile-time Checked Queries

## Trading Analogy

Imagine you work at an investment bank where every trade order goes through a risk management system **before** it reaches the exchange. If the order is invalid — wrong ticker, incorrect price format, non-existent account — the system rejects it **instantly**, before it's ever sent to the market.

This is exactly what **sqlx** does for SQL queries in Rust. Instead of discovering errors in production (when a query can't find a column or returns an unexpected type), sqlx verifies all queries **at compile time**:

- Does the `trades` table exist?
- Is there a `price` column with type `DECIMAL`?
- Is the result type compatible with your Rust struct?

If something is wrong — the code simply **won't compile**. It's like having a risk manager who checks every order before you hit the "Submit" button.

## What is sqlx?

**sqlx** is an async SQL crate for Rust with these features:

1. **Compile-time query verification** — sqlx connects to your database during compilation and verifies query correctness
2. **Pure Rust** — no ORM, no magic, just SQL
3. **Async-first** — works with tokio, async-std
4. **Supports PostgreSQL, MySQL, SQLite**

```rust
// Cargo.toml
// [dependencies]
// sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros", "chrono", "rust_decimal"] }
// tokio = { version = "1", features = ["full"] }
// chrono = { version = "0.4", features = ["serde"] }
// rust_decimal = { version = "1", features = ["db-postgres"] }
```

## The query! Macro — Compile-time Verification

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// This struct will be populated with data from the DB
#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,        // "BUY" or "SELL"
    price: Decimal,
    quantity: Decimal,
    executed_at: DateTime<Utc>,
}

async fn get_recent_trades(pool: &PgPool, symbol: &str) -> Result<Vec<Trade>, sqlx::Error> {
    // The query_as! macro verifies the query at compile time!
    // If the trades table doesn't exist or columns don't match —
    // the code WON'T compile
    let trades = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, side, price, quantity, executed_at
        FROM trades
        WHERE symbol = $1
        ORDER BY executed_at DESC
        LIMIT 100
        "#,
        symbol
    )
    .fetch_all(pool)
    .await?;

    Ok(trades)
}
```

### How Does It Work?

1. During compilation, sqlx reads the `DATABASE_URL` environment variable
2. Connects to the database
3. Runs `EXPLAIN` for each query
4. Checks column types and maps them to Rust types
5. If something is wrong — compilation error!

```bash
# Set DATABASE_URL for query verification
export DATABASE_URL="postgres://user:pass@localhost:5432/trading_db"

# Build the project
cargo build
```

## Example: Trade Recording System

```rust
use sqlx::{PgPool, postgres::PgPoolOptions, FromRow};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, FromRow)]
struct Position {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
    unrealized_pnl: Option<Decimal>,
}

#[derive(Debug, FromRow)]
struct OrderRecord {
    id: i64,
    symbol: String,
    order_type: String,
    side: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
    created_at: DateTime<Utc>,
}

struct TradingDatabase {
    pool: PgPool,
}

impl TradingDatabase {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    // All queries are verified at compile time!

    async fn insert_order(
        &self,
        symbol: &str,
        order_type: &str,
        side: &str,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> Result<i64, sqlx::Error> {
        // query_scalar! returns a single value
        let order_id = sqlx::query_scalar!(
            r#"
            INSERT INTO orders (symbol, order_type, side, price, quantity, status, created_at)
            VALUES ($1, $2, $3, $4, $5, 'NEW', NOW())
            RETURNING id
            "#,
            symbol,
            order_type,
            side,
            price,
            quantity
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(order_id)
    }

    async fn get_open_positions(&self) -> Result<Vec<Position>, sqlx::Error> {
        let positions = sqlx::query_as!(
            Position,
            r#"
            SELECT
                symbol,
                quantity,
                avg_price,
                unrealized_pnl
            FROM positions
            WHERE quantity != 0
            ORDER BY symbol
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(positions)
    }

    async fn get_orders_by_status(&self, status: &str) -> Result<Vec<OrderRecord>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderRecord,
            r#"
            SELECT id, symbol, order_type, side, price, quantity, status, created_at
            FROM orders
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    async fn update_order_status(
        &self,
        order_id: i64,
        new_status: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE orders
            SET status = $2, updated_at = NOW()
            WHERE id = $1 AND status != 'FILLED' AND status != 'CANCELLED'
            "#,
            order_id,
            new_status
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_daily_pnl(&self, date: chrono::NaiveDate) -> Result<Option<Decimal>, sqlx::Error> {
        let pnl = sqlx::query_scalar!(
            r#"
            SELECT SUM(realized_pnl) as "total_pnl"
            FROM trades
            WHERE DATE(executed_at) = $1
            "#,
            date
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(pnl)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DATABASE_URL must be set for compilation
    let db = TradingDatabase::new(&std::env::var("DATABASE_URL")?).await?;

    // Place a limit order
    let order_id = db.insert_order(
        "BTC/USDT",
        "LIMIT",
        "BUY",
        Some(Decimal::new(4200000, 2)), // 42000.00
        Decimal::new(1, 1),             // 0.1 BTC
    ).await?;

    println!("Created order #{}", order_id);

    // Get open positions
    let positions = db.get_open_positions().await?;
    for pos in positions {
        println!("{}: {} @ {} (PnL: {:?})",
            pos.symbol,
            pos.quantity,
            pos.avg_price,
            pos.unrealized_pnl
        );
    }

    // Get active orders
    let pending_orders = db.get_orders_by_status("NEW").await?;
    println!("Active orders: {}", pending_orders.len());

    Ok(())
}
```

## Working with Transactions

Transactions are critical in trading — order execution must be atomic:

```rust
use sqlx::{PgPool, Transaction, Postgres};
use rust_decimal::Decimal;

struct TradeExecution {
    order_id: i64,
    executed_price: Decimal,
    executed_quantity: Decimal,
}

async fn execute_trade(
    pool: &PgPool,
    execution: TradeExecution,
) -> Result<(), sqlx::Error> {
    // Begin transaction
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 1. Get order information
    let order = sqlx::query!(
        r#"
        SELECT symbol, side, quantity
        FROM orders
        WHERE id = $1 AND status = 'NEW'
        FOR UPDATE  -- Lock the row
        "#,
        execution.order_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // 2. Update order status
    sqlx::query!(
        r#"
        UPDATE orders
        SET status = 'FILLED',
            filled_quantity = $2,
            filled_price = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
        execution.order_id,
        execution.executed_quantity,
        execution.executed_price
    )
    .execute(&mut *tx)
    .await?;

    // 3. Create trade record
    sqlx::query!(
        r#"
        INSERT INTO trades (order_id, symbol, side, price, quantity, executed_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        execution.order_id,
        order.symbol,
        order.side,
        execution.executed_price,
        execution.executed_quantity
    )
    .execute(&mut *tx)
    .await?;

    // 4. Update position
    let position_change = if order.side == "BUY" {
        execution.executed_quantity
    } else {
        -execution.executed_quantity
    };

    sqlx::query!(
        r#"
        INSERT INTO positions (symbol, quantity, avg_price, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (symbol)
        DO UPDATE SET
            quantity = positions.quantity + $2,
            avg_price = CASE
                WHEN $2 > 0 THEN
                    (positions.quantity * positions.avg_price + $2 * $3) / (positions.quantity + $2)
                ELSE positions.avg_price
            END,
            updated_at = NOW()
        "#,
        order.symbol,
        position_change,
        execution.executed_price
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction — all changes are atomic
    tx.commit().await?;

    println!("Trade executed: {} {} @ {}",
        order.symbol,
        execution.executed_quantity,
        execution.executed_price
    );

    Ok(())
}
```

## Offline Mode with sqlx-data.json

For CI/CD pipelines where database access is unavailable, sqlx supports offline mode:

```bash
# Generate query metadata file
cargo sqlx prepare

# This creates .sqlx/query-*.json for each query
# and sqlx-data.json in the project root
```

```rust
// In Cargo.toml add:
// [features]
// offline = ["sqlx/offline"]

// Now you can compile without database connection:
// SQLX_OFFLINE=true cargo build --features offline
```

## Dynamic Queries with QueryBuilder

Sometimes you need dynamic queries (filters, sorting). Use `QueryBuilder` for this:

```rust
use sqlx::{PgPool, QueryBuilder, postgres::Postgres};
use rust_decimal::Decimal;

#[derive(Debug)]
struct TradeFilter {
    symbol: Option<String>,
    side: Option<String>,
    min_price: Option<Decimal>,
    max_price: Option<Decimal>,
    limit: i64,
}

async fn search_trades(
    pool: &PgPool,
    filter: TradeFilter,
) -> Result<Vec<(i64, String, String, Decimal, Decimal)>, sqlx::Error> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, symbol, side, price, quantity FROM trades WHERE 1=1"
    );

    if let Some(symbol) = &filter.symbol {
        builder.push(" AND symbol = ");
        builder.push_bind(symbol);
    }

    if let Some(side) = &filter.side {
        builder.push(" AND side = ");
        builder.push_bind(side);
    }

    if let Some(min_price) = &filter.min_price {
        builder.push(" AND price >= ");
        builder.push_bind(min_price);
    }

    if let Some(max_price) = &filter.max_price {
        builder.push(" AND price <= ");
        builder.push_bind(max_price);
    }

    builder.push(" ORDER BY executed_at DESC LIMIT ");
    builder.push_bind(filter.limit);

    let query = builder.build_query_as::<(i64, String, String, Decimal, Decimal)>();

    query.fetch_all(pool).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let filter = TradeFilter {
        symbol: Some("BTC/USDT".to_string()),
        side: Some("BUY".to_string()),
        min_price: Some(Decimal::new(4000000, 2)),
        max_price: None,
        limit: 50,
    };

    let trades = search_trades(&pool, filter).await?;

    for (id, symbol, side, price, qty) in trades {
        println!("#{}: {} {} {} @ {}", id, side, qty, symbol, price);
    }

    Ok(())
}
```

## Benefits of Compile-time Verification

| Without verification (runtime) | With verification (compile-time) |
|-------------------------------|----------------------------------|
| Errors in production | Errors at compilation |
| Refactoring is dangerous | Refactoring is safe |
| Requires test coverage | Compiler is the best test |
| Typos in SQL break code | Typos don't compile |
| Type mismatches at runtime | Types checked immediately |

## What We Learned

| Concept | Description |
|---------|-------------|
| `query!` | Macro for compile-time SQL verification |
| `query_as!` | Maps results to a struct with type checking |
| `query_scalar!` | Returns a single value |
| Transactions | Atomic operations with `begin()` / `commit()` |
| QueryBuilder | Dynamic query construction |
| Offline mode | Compile without DB access via `sqlx prepare` |

## Practical Exercises

1. **Order System**: Create an `OrderBook` struct with methods for placing, canceling, and retrieving orders. All queries should use `query_as!` for compile-time verification.

2. **Position Report**: Write a function that returns all positions with their current market value and unrealized PnL. Use JOIN between `positions` and `market_prices` tables.

3. **Trade History**: Implement pagination for trade history using `OFFSET` and `LIMIT`. Add filtering by date and symbol.

4. **Atomic Swap**: Write a transaction that simultaneously closes a position in one instrument and opens another (e.g., portfolio rebalancing BTC -> ETH).

## Homework

1. **Risk Management with DB**: Create a system that:
   - Stores risk limits in a `risk_limits` table (max_position_size, max_daily_loss, etc.)
   - Checks current positions against limits before each order
   - Records all checks in a `risk_checks` table for audit
   - Uses transactions for atomic verification and order creation

2. **Performance Monitoring**: Add query execution time logging. Create a wrapper function that measures time and records slow queries (>100ms) in a `slow_queries` table.

3. **Position Caching**: Implement an in-memory position cache with invalidation on DB changes. Use PostgreSQL's `LISTEN/NOTIFY` for change notifications.

4. **Offline Preparation**: Configure your project for offline mode:
   - Run `cargo sqlx prepare`
   - Add generated files to git
   - Verify the project compiles with `SQLX_OFFLINE=true`

## Navigation

[← Previous day](../226-tokio-postgres-async-client/en.md) | [Next day →](../228-sqlx-migrations/en.md)
