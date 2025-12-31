# Day 230: ORM vs Raw SQL

## Trading Analogy

Imagine you're managing a trading terminal. You have two ways to interact with the exchange:

1. **Raw SQL** — like direct access to the exchange's trading API: you write precise commands, have full control over every operation, but must know all protocol details. This is like a professional algo-trader who writes queries manually for maximum speed.

2. **ORM (Object-Relational Mapping)** — like a trading terminal with a graphical interface: you work with intuitive objects (orders, positions, balances), and the terminal converts your actions into exchange commands. Convenient and safe, but sometimes slower.

In trading, the choice between ORM and Raw SQL can affect:
- Order execution speed (latency)
- Convenience when working with historical data
- System reliability under high load

## What is ORM?

**ORM (Object-Relational Mapping)** is a technique that allows you to work with databases through programming language objects without writing SQL queries manually.

### Popular ORMs in Rust

| ORM | Description | Features |
|-----|-------------|----------|
| **Diesel** | Type-safe ORM | Compile-time query validation |
| **SeaORM** | Asynchronous ORM | async/await support, dynamic queries |
| **SQLx** | Not quite ORM | Compile-time SQL checking, direct queries |

## Raw SQL: Direct Queries

### Advantages of Raw SQL

```rust
use sqlx::{postgres::PgPoolOptions, Row};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn get_trades_raw(pool: &sqlx::PgPool, symbol: &str) -> Result<Vec<Trade>, sqlx::Error> {
    // Raw SQL — full control over the query
    let trades = sqlx::query(
        r#"
        SELECT id, symbol, price, quantity, side, timestamp
        FROM trades
        WHERE symbol = $1
        ORDER BY timestamp DESC
        LIMIT 100
        "#
    )
    .bind(symbol)
    .fetch_all(pool)
    .await?
    .iter()
    .map(|row| Trade {
        id: row.get("id"),
        symbol: row.get("symbol"),
        price: row.get("price"),
        quantity: row.get("quantity"),
        side: row.get("side"),
        timestamp: row.get("timestamp"),
    })
    .collect();

    Ok(trades)
}

// Complex analytical query — ideal for Raw SQL
async fn calculate_vwap(
    pool: &sqlx::PgPool,
    symbol: &str,
    hours: i32,
) -> Result<f64, sqlx::Error> {
    // VWAP = Volume Weighted Average Price
    let result = sqlx::query_scalar::<_, f64>(
        r#"
        SELECT SUM(price * quantity) / SUM(quantity) as vwap
        FROM trades
        WHERE symbol = $1
          AND timestamp > NOW() - INTERVAL '$2 hours'
          AND quantity > 0
        "#
    )
    .bind(symbol)
    .bind(hours)
    .fetch_one(pool)
    .await?;

    Ok(result)
}
```

### When to Use Raw SQL

```rust
use sqlx::postgres::PgPool;

// 1. Complex analytical queries
async fn analyze_trading_patterns(pool: &PgPool) -> Result<Vec<PatternResult>, sqlx::Error> {
    sqlx::query_as::<_, PatternResult>(
        r#"
        WITH hourly_stats AS (
            SELECT
                symbol,
                DATE_TRUNC('hour', timestamp) as hour,
                AVG(price) as avg_price,
                SUM(quantity) as total_volume,
                COUNT(*) as trade_count
            FROM trades
            WHERE timestamp > NOW() - INTERVAL '24 hours'
            GROUP BY symbol, DATE_TRUNC('hour', timestamp)
        ),
        price_changes AS (
            SELECT
                symbol,
                hour,
                avg_price,
                total_volume,
                LAG(avg_price) OVER (PARTITION BY symbol ORDER BY hour) as prev_price
            FROM hourly_stats
        )
        SELECT
            symbol,
            hour,
            avg_price,
            total_volume,
            COALESCE((avg_price - prev_price) / prev_price * 100, 0) as price_change_pct
        FROM price_changes
        WHERE prev_price IS NOT NULL
        ORDER BY ABS(price_change_pct) DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, sqlx::FromRow)]
struct PatternResult {
    symbol: String,
    hour: chrono::DateTime<chrono::Utc>,
    avg_price: f64,
    total_volume: f64,
    price_change_pct: f64,
}

// 2. Optimized queries for high-load systems
async fn get_order_book_snapshot(
    pool: &PgPool,
    symbol: &str,
    depth: i32,
) -> Result<OrderBookSnapshot, sqlx::Error> {
    // Using efficient grouping for top-N retrieval
    let rows = sqlx::query(
        r#"
        SELECT
            side,
            price,
            SUM(quantity) as total_quantity,
            COUNT(*) as order_count
        FROM orders
        WHERE symbol = $1 AND status = 'active'
        GROUP BY side, price
        ORDER BY
            CASE WHEN side = 'buy' THEN price END DESC,
            CASE WHEN side = 'sell' THEN price END ASC
        LIMIT $2
        "#
    )
    .bind(symbol)
    .bind(depth * 2) // Bids + Asks
    .fetch_all(pool)
    .await?;

    // Split into bids and asks
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for row in rows {
        let level = PriceLevel {
            price: row.get("price"),
            quantity: row.get("total_quantity"),
            order_count: row.get("order_count"),
        };

        match row.get::<String, _>("side").as_str() {
            "buy" => bids.push(level),
            "sell" => asks.push(level),
            _ => {}
        }
    }

    Ok(OrderBookSnapshot { symbol: symbol.to_string(), bids, asks })
}

#[derive(Debug)]
struct PriceLevel {
    price: f64,
    quantity: f64,
    order_count: i64,
}

#[derive(Debug)]
struct OrderBookSnapshot {
    symbol: String,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}
```

## ORM: Diesel

### Setting Up Diesel

```rust
// Cargo.toml
// [dependencies]
// diesel = { version = "2.1", features = ["postgres", "chrono"] }
// dotenvy = "0.15"

// schema.rs (generated by diesel)
diesel::table! {
    trades (id) {
        id -> Int8,
        symbol -> Varchar,
        price -> Float8,
        quantity -> Float8,
        side -> Varchar,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    orders (id) {
        id -> Int8,
        symbol -> Varchar,
        side -> Varchar,
        order_type -> Varchar,
        price -> Nullable<Float8>,
        quantity -> Float8,
        filled_quantity -> Float8,
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    portfolios (id) {
        id -> Int8,
        user_id -> Int8,
        symbol -> Varchar,
        quantity -> Float8,
        avg_price -> Float8,
        updated_at -> Timestamptz,
    }
}
```

### CRUD Operations with Diesel

```rust
use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::{DateTime, Utc};

// Models
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = trades)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = trades)]
struct NewTrade<'a> {
    symbol: &'a str,
    price: f64,
    quantity: f64,
    side: &'a str,
    timestamp: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Debug, AsChangeset)]
#[diesel(table_name = orders)]
struct Order {
    id: i64,
    symbol: String,
    side: String,
    order_type: String,
    price: Option<f64>,
    quantity: f64,
    filled_quantity: f64,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Operations
fn create_trade(conn: &mut PgConnection, trade: NewTrade) -> QueryResult<Trade> {
    use crate::schema::trades;

    diesel::insert_into(trades::table)
        .values(&trade)
        .returning(Trade::as_returning())
        .get_result(conn)
}

fn get_recent_trades(
    conn: &mut PgConnection,
    symbol_filter: &str,
    limit: i64,
) -> QueryResult<Vec<Trade>> {
    use crate::schema::trades::dsl::*;

    trades
        .filter(symbol.eq(symbol_filter))
        .order(timestamp.desc())
        .limit(limit)
        .select(Trade::as_select())
        .load(conn)
}

fn update_order_status(
    conn: &mut PgConnection,
    order_id: i64,
    new_status: &str,
    filled: f64,
) -> QueryResult<Order> {
    use crate::schema::orders::dsl::*;

    diesel::update(orders.find(order_id))
        .set((
            status.eq(new_status),
            filled_quantity.eq(filled),
            updated_at.eq(Utc::now()),
        ))
        .returning(Order::as_returning())
        .get_result(conn)
}

fn cancel_pending_orders(conn: &mut PgConnection, user_symbol: &str) -> QueryResult<usize> {
    use crate::schema::orders::dsl::*;

    diesel::update(orders)
        .filter(symbol.eq(user_symbol))
        .filter(status.eq("pending"))
        .set((
            status.eq("cancelled"),
            updated_at.eq(Utc::now()),
        ))
        .execute(conn)
}
```

### Complex Queries with Diesel

```rust
use diesel::prelude::*;
use diesel::dsl::{sum, avg, count};

// Aggregation: trading volume by symbol
fn get_volume_by_symbol(
    conn: &mut PgConnection,
    since: DateTime<Utc>,
) -> QueryResult<Vec<(String, f64)>> {
    use crate::schema::trades::dsl::*;

    trades
        .filter(timestamp.gt(since))
        .group_by(symbol)
        .select((symbol, sum(quantity)))
        .load::<(String, Option<f64>)>(conn)
        .map(|results| {
            results.into_iter()
                .map(|(s, q)| (s, q.unwrap_or(0.0)))
                .collect()
        })
}

// Portfolio summary
#[derive(Debug, Queryable)]
struct PortfolioSummary {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_value: f64,
}

fn get_portfolio_with_current_prices(
    conn: &mut PgConnection,
    user_id_filter: i64,
) -> QueryResult<Vec<PortfolioSummary>> {
    use crate::schema::portfolios::dsl::*;

    // Subquery for latest prices
    let latest_prices = trades
        .group_by(symbol)
        .select((symbol, diesel::dsl::max(price)));

    // Complex JOINs in Diesel require raw SQL or subqueries
    // Showing simplified query for clarity
    portfolios
        .filter(user_id.eq(user_id_filter))
        .filter(quantity.gt(0.0))
        .select((symbol, quantity, avg_price, quantity * avg_price))
        .load(conn)
}

// Transaction: order execution
fn execute_order(
    conn: &mut PgConnection,
    order_id: i64,
    execution_price: f64,
    execution_qty: f64,
) -> QueryResult<()> {
    use crate::schema::{orders, trades, portfolios};

    conn.transaction(|conn| {
        // 1. Get the order
        let order: Order = orders::table
            .find(order_id)
            .select(Order::as_select())
            .first(conn)?;

        // 2. Create trade record
        let new_trade = NewTrade {
            symbol: &order.symbol,
            price: execution_price,
            quantity: execution_qty,
            side: &order.side,
            timestamp: Utc::now(),
        };

        diesel::insert_into(trades::table)
            .values(&new_trade)
            .execute(conn)?;

        // 3. Update order
        let new_filled = order.filled_quantity + execution_qty;
        let new_status = if new_filled >= order.quantity {
            "filled"
        } else {
            "partial"
        };

        diesel::update(orders::table.find(order_id))
            .set((
                orders::filled_quantity.eq(new_filled),
                orders::status.eq(new_status),
                orders::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;

        // 4. Update portfolio
        let qty_change = if order.side == "buy" {
            execution_qty
        } else {
            -execution_qty
        };

        diesel::insert_into(portfolios::table)
            .values((
                portfolios::user_id.eq(1i64), // simplified
                portfolios::symbol.eq(&order.symbol),
                portfolios::quantity.eq(qty_change),
                portfolios::avg_price.eq(execution_price),
                portfolios::updated_at.eq(Utc::now()),
            ))
            .on_conflict((portfolios::user_id, portfolios::symbol))
            .do_update()
            .set((
                portfolios::quantity.eq(portfolios::quantity + qty_change),
                portfolios::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;

        Ok(())
    })
}
```

## ORM: SeaORM (Asynchronous)

### Setting Up SeaORM

```rust
// Cargo.toml
// [dependencies]
// sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls"] }

// entity/trade.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub side: String,
    pub timestamp: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// entity/order.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<f64>,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### CRUD with SeaORM

```rust
use sea_orm::*;
use entity::{trade, order};

// Create trade
async fn create_trade(
    db: &DatabaseConnection,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
) -> Result<trade::Model, DbErr> {
    let new_trade = trade::ActiveModel {
        symbol: Set(symbol),
        price: Set(price),
        quantity: Set(quantity),
        side: Set(side),
        timestamp: Set(chrono::Utc::now()),
        ..Default::default()
    };

    new_trade.insert(db).await
}

// Find with filtering
async fn find_trades_by_symbol(
    db: &DatabaseConnection,
    symbol: &str,
    min_quantity: f64,
) -> Result<Vec<trade::Model>, DbErr> {
    trade::Entity::find()
        .filter(trade::Column::Symbol.eq(symbol))
        .filter(trade::Column::Quantity.gte(min_quantity))
        .order_by_desc(trade::Column::Timestamp)
        .limit(100)
        .all(db)
        .await
}

// Update order
async fn fill_order(
    db: &DatabaseConnection,
    order_id: i64,
    filled_qty: f64,
) -> Result<order::Model, DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound("Order not found".to_string()))?;

    let new_filled = order.filled_quantity + filled_qty;
    let new_status = if new_filled >= order.quantity {
        "filled".to_string()
    } else {
        "partial".to_string()
    };

    let mut active_order: order::ActiveModel = order.into();
    active_order.filled_quantity = Set(new_filled);
    active_order.status = Set(new_status);
    active_order.updated_at = Set(chrono::Utc::now());

    active_order.update(db).await
}

// Aggregation
async fn get_trading_stats(
    db: &DatabaseConnection,
    symbol: &str,
) -> Result<TradingStats, DbErr> {
    use sea_orm::sea_query::{Expr, Func};

    #[derive(Debug, FromQueryResult)]
    struct StatsResult {
        total_volume: Option<f64>,
        trade_count: Option<i64>,
        avg_price: Option<f64>,
        max_price: Option<f64>,
        min_price: Option<f64>,
    }

    let result: StatsResult = trade::Entity::find()
        .filter(trade::Column::Symbol.eq(symbol))
        .select_only()
        .column_as(Expr::col(trade::Column::Quantity).sum(), "total_volume")
        .column_as(Expr::col(trade::Column::Id).count(), "trade_count")
        .column_as(Expr::col(trade::Column::Price).avg(), "avg_price")
        .column_as(Expr::col(trade::Column::Price).max(), "max_price")
        .column_as(Expr::col(trade::Column::Price).min(), "min_price")
        .into_model::<StatsResult>()
        .one(db)
        .await?
        .unwrap_or(StatsResult {
            total_volume: None,
            trade_count: None,
            avg_price: None,
            max_price: None,
            min_price: None,
        });

    Ok(TradingStats {
        symbol: symbol.to_string(),
        total_volume: result.total_volume.unwrap_or(0.0),
        trade_count: result.trade_count.unwrap_or(0),
        avg_price: result.avg_price.unwrap_or(0.0),
        price_range: (
            result.min_price.unwrap_or(0.0),
            result.max_price.unwrap_or(0.0),
        ),
    })
}

#[derive(Debug)]
struct TradingStats {
    symbol: String,
    total_volume: f64,
    trade_count: i64,
    avg_price: f64,
    price_range: (f64, f64),
}
```

## Comparison: ORM vs Raw SQL

### Practical Example: Trading Report

```rust
use sqlx::PgPool;
use diesel::prelude::*;

// ===== RAW SQL (SQLx) =====
async fn generate_trading_report_raw(
    pool: &PgPool,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> Result<TradingReport, sqlx::Error> {
    // Single complex query — maximum efficiency
    let report = sqlx::query_as::<_, TradingReport>(
        r#"
        WITH daily_stats AS (
            SELECT
                DATE(timestamp) as trade_date,
                symbol,
                SUM(CASE WHEN side = 'buy' THEN quantity ELSE 0 END) as buy_volume,
                SUM(CASE WHEN side = 'sell' THEN quantity ELSE 0 END) as sell_volume,
                SUM(CASE WHEN side = 'buy' THEN quantity * price ELSE 0 END) as buy_value,
                SUM(CASE WHEN side = 'sell' THEN quantity * price ELSE 0 END) as sell_value,
                COUNT(*) as trade_count,
                MAX(price) as high_price,
                MIN(price) as low_price,
                (ARRAY_AGG(price ORDER BY timestamp ASC))[1] as open_price,
                (ARRAY_AGG(price ORDER BY timestamp DESC))[1] as close_price
            FROM trades
            WHERE DATE(timestamp) BETWEEN $1 AND $2
            GROUP BY DATE(timestamp), symbol
        )
        SELECT
            symbol,
            SUM(buy_volume) as total_buy_volume,
            SUM(sell_volume) as total_sell_volume,
            SUM(buy_value) as total_buy_value,
            SUM(sell_value) as total_sell_value,
            SUM(trade_count) as total_trades,
            AVG(high_price - low_price) as avg_daily_range,
            (SELECT close_price FROM daily_stats ds2
             WHERE ds2.symbol = daily_stats.symbol
             ORDER BY trade_date DESC LIMIT 1) -
            (SELECT open_price FROM daily_stats ds3
             WHERE ds3.symbol = daily_stats.symbol
             ORDER BY trade_date ASC LIMIT 1) as price_change
        FROM daily_stats
        GROUP BY symbol
        "#
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;

    Ok(report)
}

#[derive(Debug, sqlx::FromRow)]
struct TradingReport {
    symbol: String,
    total_buy_volume: f64,
    total_sell_volume: f64,
    total_buy_value: f64,
    total_sell_value: f64,
    total_trades: i64,
    avg_daily_range: f64,
    price_change: f64,
}

// ===== ORM (Diesel) — requires more code =====
fn generate_trading_report_orm(
    conn: &mut PgConnection,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
) -> QueryResult<Vec<SimpleReport>> {
    use crate::schema::trades::dsl::*;
    use diesel::dsl::{sum, count};

    // ORM often requires multiple queries for complex logic
    // or using raw SQL internally

    // Simplified report
    trades
        .filter(timestamp.ge(start_date.and_hms_opt(0, 0, 0).unwrap()))
        .filter(timestamp.lt(end_date.and_hms_opt(23, 59, 59).unwrap()))
        .group_by(symbol)
        .select((
            symbol,
            sum(quantity),
            count(id),
        ))
        .load::<(String, Option<f64>, i64)>(conn)
        .map(|results| {
            results.into_iter()
                .map(|(s, v, c)| SimpleReport {
                    symbol: s,
                    total_volume: v.unwrap_or(0.0),
                    trade_count: c,
                })
                .collect()
        })
}

#[derive(Debug)]
struct SimpleReport {
    symbol: String,
    total_volume: f64,
    trade_count: i64,
}
```

### Comparison Table

| Criterion | Raw SQL | ORM (Diesel) | ORM (SeaORM) |
|-----------|---------|--------------|--------------|
| **Performance** | Maximum | High | High |
| **Type Safety** | Partial (SQLx) | Full | Full |
| **Complex Queries** | Excellent | Limited | Limited |
| **Development Speed** | Slower | Faster | Faster |
| **Migration Support** | Manual | Built-in | Built-in |
| **Async/Await** | SQLx — yes | No | Yes |
| **Learning Curve** | SQL knowledge | ORM + SQL | ORM + SQL |
| **Flexibility** | Maximum | Medium | Medium |

## Hybrid Approach

Real trading systems often use both approaches:

```rust
use sqlx::PgPool;
use sea_orm::DatabaseConnection;

struct TradingRepository {
    raw_pool: PgPool,            // For complex queries
    orm_db: DatabaseConnection,  // For CRUD operations
}

impl TradingRepository {
    // CRUD — via ORM (convenience and safety)
    async fn create_order(&self, order: NewOrder) -> Result<Order, Error> {
        let active_model = order::ActiveModel {
            symbol: Set(order.symbol),
            side: Set(order.side),
            order_type: Set(order.order_type),
            price: Set(order.price),
            quantity: Set(order.quantity),
            filled_quantity: Set(0.0),
            status: Set("pending".to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        Ok(active_model.insert(&self.orm_db).await?)
    }

    async fn get_order(&self, id: i64) -> Result<Option<Order>, Error> {
        Ok(order::Entity::find_by_id(id)
            .one(&self.orm_db)
            .await?)
    }

    async fn update_order_status(&self, id: i64, status: &str) -> Result<Order, Error> {
        let order = self.get_order(id).await?
            .ok_or(Error::NotFound)?;

        let mut active: order::ActiveModel = order.into();
        active.status = Set(status.to_string());
        active.updated_at = Set(chrono::Utc::now());

        Ok(active.update(&self.orm_db).await?)
    }

    // Analytics — via Raw SQL (performance)
    async fn get_market_depth(&self, symbol: &str, levels: i32) -> Result<MarketDepth, Error> {
        let bids = sqlx::query_as::<_, PriceLevel>(
            r#"
            SELECT price, SUM(quantity - filled_quantity) as quantity
            FROM orders
            WHERE symbol = $1 AND side = 'buy' AND status = 'pending'
            GROUP BY price
            ORDER BY price DESC
            LIMIT $2
            "#
        )
        .bind(symbol)
        .bind(levels)
        .fetch_all(&self.raw_pool)
        .await?;

        let asks = sqlx::query_as::<_, PriceLevel>(
            r#"
            SELECT price, SUM(quantity - filled_quantity) as quantity
            FROM orders
            WHERE symbol = $1 AND side = 'sell' AND status = 'pending'
            GROUP BY price
            ORDER BY price ASC
            LIMIT $2
            "#
        )
        .bind(symbol)
        .bind(levels)
        .fetch_all(&self.raw_pool)
        .await?;

        Ok(MarketDepth { symbol: symbol.to_string(), bids, asks })
    }

    // Complex analytics — Raw SQL only
    async fn calculate_portfolio_pnl(&self, user_id: i64) -> Result<PortfolioPnL, Error> {
        let pnl = sqlx::query_as::<_, PortfolioPnL>(
            r#"
            WITH current_prices AS (
                SELECT DISTINCT ON (symbol)
                    symbol,
                    price as current_price
                FROM trades
                ORDER BY symbol, timestamp DESC
            ),
            portfolio_values AS (
                SELECT
                    p.symbol,
                    p.quantity,
                    p.avg_price as entry_price,
                    cp.current_price,
                    p.quantity * p.avg_price as cost_basis,
                    p.quantity * cp.current_price as market_value,
                    p.quantity * (cp.current_price - p.avg_price) as unrealized_pnl
                FROM portfolios p
                JOIN current_prices cp ON p.symbol = cp.symbol
                WHERE p.user_id = $1 AND p.quantity > 0
            )
            SELECT
                COALESCE(SUM(cost_basis), 0) as total_cost,
                COALESCE(SUM(market_value), 0) as total_value,
                COALESCE(SUM(unrealized_pnl), 0) as total_pnl,
                CASE
                    WHEN SUM(cost_basis) > 0
                    THEN SUM(unrealized_pnl) / SUM(cost_basis) * 100
                    ELSE 0
                END as pnl_percentage
            FROM portfolio_values
            "#
        )
        .bind(user_id)
        .fetch_one(&self.raw_pool)
        .await?;

        Ok(pnl)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PriceLevel {
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct MarketDepth {
    symbol: String,
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

#[derive(Debug, sqlx::FromRow)]
struct PortfolioPnL {
    total_cost: f64,
    total_value: f64,
    total_pnl: f64,
    pnl_percentage: f64,
}

#[derive(Debug)]
struct NewOrder {
    symbol: String,
    side: String,
    order_type: String,
    price: Option<f64>,
    quantity: f64,
}

#[derive(Debug)]
struct Order {
    id: i64,
    symbol: String,
    side: String,
    order_type: String,
    price: Option<f64>,
    quantity: f64,
    filled_quantity: f64,
    status: String,
}

#[derive(Debug)]
enum Error {
    NotFound,
    Database(String),
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Raw SQL | Direct SQL queries with maximum control |
| ORM | Object-Relational Mapping for convenient database work |
| Diesel | Type-safe synchronous ORM for Rust |
| SeaORM | Asynchronous ORM with async/await support |
| SQLx | Compile-time SQL query validation |
| Hybrid Approach | Combining ORM for CRUD and Raw SQL for analytics |
| Transactions | Atomic operations in both approaches |

## Homework

1. **CRUD with Diesel**: Create a module for managing trading positions with operations:
   - Opening a position
   - Closing a position
   - Updating stop-loss/take-profit
   - Getting all open positions

2. **Analytics with Raw SQL**: Write queries for:
   - Calculating Simple Moving Average (SMA) for the last N candles
   - Identifying support/resistance levels
   - Finding volume patterns (volume spikes)

3. **Hybrid Repository**: Create a `TradingSystem` struct that:
   - Uses SeaORM for order management
   - Uses SQLx for real-time data aggregation
   - Supports transactions across both approaches

4. **Benchmark**: Compare performance:
   - Inserting 10,000 trades via ORM vs Raw SQL
   - Aggregation query on 1,000,000 records
   - Measure time and memory consumption

## Navigation

[← Previous day](../229-database-migrations/en.md) | [Next day →](../231-database-connection-pooling/en.md)
