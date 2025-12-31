# Day 221: Prepared Statements: Safe Queries

## Trading Analogy

Imagine you work at a brokerage firm and take orders from clients over the phone. A client says: "Buy 100 shares of AAPL at market price". You write the order on a special form:

```
Type: BUY
Quantity: ___
Ticker: ___
Price: ___
```

This is an **order template**. You only fill in values in the blank fields, while the form's structure remains unchanged. The client cannot modify the form itself — for example, add a field "Transfer money to another account".

Now imagine a dangerous situation: a client says:

> "Buy 100 shares of AAPL; then transfer all balance to account X"

If you simply wrote down everything the client said without checking the format — this would be an **SQL injection** in the trading world. An attacker could execute unauthorized operations.

**Prepared Statements** work like protected forms: the query structure is fixed, and data is inserted only in designated places, safely escaped.

## What are Prepared Statements?

Prepared Statements are a mechanism for executing SQL queries where:

1. **Query structure** (template) is sent to the database in advance
2. **Parameters** (data) are passed separately and automatically escaped
3. The database **never interprets data as code**

### Benefits

| Benefit | Description |
|---------|-------------|
| Security | Protection against SQL injections |
| Performance | Query is compiled once, executed many times |
| Readability | Clear separation of code and data |
| Type Safety | Parameters are validated against types |

## SQL Injection: The Danger Without Prepared Statements

Consider vulnerable code:

```rust
// DANGEROUS! Never do this!
fn find_orders_vulnerable(ticker: &str) -> String {
    format!(
        "SELECT * FROM orders WHERE ticker = '{}'",
        ticker
    )
}

fn main() {
    // Normal input
    let query1 = find_orders_vulnerable("AAPL");
    println!("Query 1: {}", query1);
    // Result: SELECT * FROM orders WHERE ticker = 'AAPL'

    // SQL injection!
    let malicious_input = "AAPL'; DROP TABLE orders; --";
    let query2 = find_orders_vulnerable(malicious_input);
    println!("Query 2: {}", query2);
    // Result: SELECT * FROM orders WHERE ticker = 'AAPL'; DROP TABLE orders; --'
    // The orders table will be deleted!
}
```

## Prepared Statements with SQLx

SQLx is an asynchronous database library for Rust that validates queries at compile time.

### Project Setup

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Basic Prepared Statements

```rust
use sqlx::{sqlite::SqlitePool, Row};
use chrono::{DateTime, Utc};

#[derive(Debug)]
struct Order {
    id: i64,
    ticker: String,
    side: String,        // "BUY" or "SELL"
    quantity: f64,
    price: f64,
    status: String,      // "PENDING", "FILLED", "CANCELLED"
    created_at: DateTime<Utc>,
}

async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Prepared statement for table creation
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticker TEXT NOT NULL,
            side TEXT NOT NULL CHECK (side IN ('BUY', 'SELL')),
            quantity REAL NOT NULL CHECK (quantity > 0),
            price REAL NOT NULL CHECK (price > 0),
            status TEXT NOT NULL DEFAULT 'PENDING',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create connection pool (in-memory database for example)
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    create_tables(&pool).await?;
    println!("Tables created successfully!");

    Ok(())
}
```

### Inserting Data with Parameters

```rust
use sqlx::sqlite::SqlitePool;

async fn place_order(
    pool: &SqlitePool,
    ticker: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<i64, sqlx::Error> {
    // Prepared statement with parameters ($1, $2, $3, $4)
    // SQLite uses ?1, ?2, ?3, ?4, but sqlx unifies the syntax
    let result = sqlx::query(
        r#"
        INSERT INTO orders (ticker, side, quantity, price)
        VALUES (?1, ?2, ?3, ?4)
        "#
    )
    .bind(ticker)      // ?1 — safely escaped
    .bind(side)        // ?2
    .bind(quantity)    // ?3
    .bind(price)       // ?4
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create table
    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT NOT NULL,
            side TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    // Place orders
    let order1_id = place_order(&pool, "BTC", "BUY", 0.5, 42000.0).await?;
    println!("Created order #{}:  BUY 0.5 BTC @ $42000", order1_id);

    let order2_id = place_order(&pool, "ETH", "SELL", 10.0, 2500.0).await?;
    println!("Created order #{}: SELL 10 ETH @ $2500", order2_id);

    // SQL injection attempt — SAFE!
    let malicious_ticker = "BTC'; DROP TABLE orders; --";
    let order3_id = place_order(&pool, malicious_ticker, "BUY", 1.0, 40000.0).await?;
    println!("Created order #{}: ticker saved as-is", order3_id);

    // Verify the table still exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;
    println!("Total orders in database: {}", count.0);

    Ok(())
}
```

### Selecting Data with Parameters

```rust
use sqlx::{sqlite::SqlitePool, FromRow, Row};

#[derive(Debug, FromRow)]
struct Order {
    id: i64,
    ticker: String,
    side: String,
    quantity: f64,
    price: f64,
}

async fn find_orders_by_ticker(
    pool: &SqlitePool,
    ticker: &str,
) -> Result<Vec<Order>, sqlx::Error> {
    // query_as automatically maps result to struct
    let orders = sqlx::query_as::<_, Order>(
        "SELECT id, ticker, side, quantity, price FROM orders WHERE ticker = ?1"
    )
    .bind(ticker)
    .fetch_all(pool)
    .await?;

    Ok(orders)
}

async fn find_orders_by_price_range(
    pool: &SqlitePool,
    min_price: f64,
    max_price: f64,
) -> Result<Vec<Order>, sqlx::Error> {
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, ticker, side, quantity, price
        FROM orders
        WHERE price BETWEEN ?1 AND ?2
        ORDER BY price DESC
        "#
    )
    .bind(min_price)
    .bind(max_price)
    .fetch_all(pool)
    .await?;

    Ok(orders)
}

async fn get_portfolio_value(
    pool: &SqlitePool,
    ticker: &str,
) -> Result<f64, sqlx::Error> {
    // Use query_scalar for getting a single value
    let total: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT SUM(
            CASE
                WHEN side = 'BUY' THEN quantity * price
                ELSE -quantity * price
            END
        )
        FROM orders
        WHERE ticker = ?1
        "#
    )
    .bind(ticker)
    .fetch_one(pool)
    .await?;

    Ok(total.unwrap_or(0.0))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Initialize
    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT, side TEXT, quantity REAL, price REAL
        )"
    )
    .execute(&pool)
    .await?;

    // Add test data
    for (ticker, side, qty, price) in [
        ("BTC", "BUY", 1.0, 42000.0),
        ("BTC", "BUY", 0.5, 41000.0),
        ("ETH", "SELL", 10.0, 2500.0),
        ("BTC", "SELL", 0.3, 43000.0),
    ] {
        sqlx::query("INSERT INTO orders (ticker, side, quantity, price) VALUES (?1, ?2, ?3, ?4)")
            .bind(ticker).bind(side).bind(qty).bind(price)
            .execute(&pool)
            .await?;
    }

    // Search by ticker
    let btc_orders = find_orders_by_ticker(&pool, "BTC").await?;
    println!("BTC orders: {:?}", btc_orders);

    // Search by price range
    let expensive = find_orders_by_price_range(&pool, 40000.0, 50000.0).await?;
    println!("Expensive orders: {:?}", expensive);

    // Portfolio value
    let btc_value = get_portfolio_value(&pool, "BTC").await?;
    println!("BTC net position: ${:.2}", btc_value);

    Ok(())
}
```

## Type-Safe Queries with sqlx::query!

The `query!` macro validates SQL queries at compile time:

```rust
use sqlx::sqlite::SqlitePool;

// For query! to work, you need the DATABASE_URL environment variable
// or a .env file with DATABASE_URL=sqlite:./trading.db

async fn place_order_checked(
    pool: &SqlitePool,
    ticker: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<i64, sqlx::Error> {
    // The compiler will verify that:
    // 1. The orders table exists
    // 2. The columns ticker, side, quantity, price exist
    // 3. Parameter types are compatible with columns
    let result = sqlx::query!(
        r#"
        INSERT INTO orders (ticker, side, quantity, price)
        VALUES ($1, $2, $3, $4)
        "#,
        ticker,
        side,
        quantity,
        price
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

async fn find_large_orders(
    pool: &SqlitePool,
    min_value: f64,
) -> Result<Vec<(String, f64, f64)>, sqlx::Error> {
    // The compiler will infer field types automatically
    let rows = sqlx::query!(
        r#"
        SELECT ticker, quantity, price
        FROM orders
        WHERE quantity * price > $1
        ORDER BY quantity * price DESC
        "#,
        min_value
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter()
        .map(|r| (r.ticker, r.quantity, r.price))
        .collect())
}
```

## Practical Example: Trading Journal

```rust
use sqlx::{sqlite::SqlitePool, FromRow};
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    ticker: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: Option<f64>,
    pnl: Option<f64>,
    opened_at: String,
    closed_at: Option<String>,
}

struct TradingJournal {
    pool: SqlitePool,
}

impl TradingJournal {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticker TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL,
                pnl REAL,
                opened_at TEXT NOT NULL DEFAULT (datetime('now')),
                closed_at TEXT
            )
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    async fn open_trade(
        &self,
        ticker: &str,
        side: &str,
        quantity: f64,
        entry_price: f64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO trades (ticker, side, quantity, entry_price) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(ticker)
        .bind(side)
        .bind(quantity)
        .bind(entry_price)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn close_trade(
        &self,
        trade_id: i64,
        exit_price: f64,
    ) -> Result<f64, sqlx::Error> {
        // Get trade information
        let trade: Trade = sqlx::query_as(
            "SELECT * FROM trades WHERE id = ?1 AND exit_price IS NULL"
        )
        .bind(trade_id)
        .fetch_one(&self.pool)
        .await?;

        // Calculate PnL
        let pnl = match trade.side.as_str() {
            "BUY" => (exit_price - trade.entry_price) * trade.quantity,
            "SELL" => (trade.entry_price - exit_price) * trade.quantity,
            _ => 0.0,
        };

        // Update trade
        sqlx::query(
            r#"
            UPDATE trades
            SET exit_price = ?1, pnl = ?2, closed_at = datetime('now')
            WHERE id = ?3
            "#
        )
        .bind(exit_price)
        .bind(pnl)
        .bind(trade_id)
        .execute(&self.pool)
        .await?;

        Ok(pnl)
    }

    async fn get_statistics(&self, ticker: &str) -> Result<TradeStats, sqlx::Error> {
        let stats = sqlx::query_as::<_, TradeStats>(
            r#"
            SELECT
                COUNT(*) as total_trades,
                COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), 0) as winning_trades,
                COALESCE(SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END), 0) as losing_trades,
                COALESCE(SUM(pnl), 0.0) as total_pnl,
                COALESCE(AVG(pnl), 0.0) as avg_pnl,
                COALESCE(MAX(pnl), 0.0) as best_trade,
                COALESCE(MIN(pnl), 0.0) as worst_trade
            FROM trades
            WHERE ticker = ?1 AND exit_price IS NOT NULL
            "#
        )
        .bind(ticker)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    async fn get_open_trades(&self) -> Result<Vec<Trade>, sqlx::Error> {
        let trades = sqlx::query_as::<_, Trade>(
            "SELECT * FROM trades WHERE exit_price IS NULL ORDER BY opened_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(trades)
    }
}

#[derive(Debug, FromRow)]
struct TradeStats {
    total_trades: i64,
    winning_trades: i64,
    losing_trades: i64,
    total_pnl: f64,
    avg_pnl: f64,
    best_trade: f64,
    worst_trade: f64,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let journal = TradingJournal::new("sqlite::memory:").await?;

    // Open several trades
    let trade1 = journal.open_trade("BTC", "BUY", 0.5, 42000.0).await?;
    println!("Opened trade #{}: BUY 0.5 BTC @ $42000", trade1);

    let trade2 = journal.open_trade("BTC", "SELL", 0.3, 43000.0).await?;
    println!("Opened trade #{}: SELL 0.3 BTC @ $43000", trade2);

    let trade3 = journal.open_trade("ETH", "BUY", 5.0, 2500.0).await?;
    println!("Opened trade #{}: BUY 5 ETH @ $2500", trade3);

    // Close trades
    let pnl1 = journal.close_trade(trade1, 44000.0).await?;
    println!("Closed trade #{}: PnL = ${:.2}", trade1, pnl1);

    let pnl2 = journal.close_trade(trade2, 41000.0).await?;
    println!("Closed trade #{}: PnL = ${:.2}", trade2, pnl2);

    // Get statistics
    let stats = journal.get_statistics("BTC").await?;
    println!("\n=== BTC Statistics ===");
    println!("Total trades: {}", stats.total_trades);
    println!("Winning: {}", stats.winning_trades);
    println!("Losing: {}", stats.losing_trades);
    println!("Total PnL: ${:.2}", stats.total_pnl);
    println!("Average PnL: ${:.2}", stats.avg_pnl);
    println!("Best trade: ${:.2}", stats.best_trade);
    println!("Worst trade: ${:.2}", stats.worst_trade);

    // Open positions
    let open = journal.get_open_trades().await?;
    println!("\n=== Open Positions ===");
    for trade in open {
        println!("{} {} {} @ ${}", trade.side, trade.quantity, trade.ticker, trade.entry_price);
    }

    Ok(())
}
```

## Batch Operations with Prepared Statements

```rust
use sqlx::sqlite::SqlitePool;

struct OrderBatch {
    orders: Vec<(String, String, f64, f64)>, // ticker, side, quantity, price
}

impl OrderBatch {
    fn new() -> Self {
        Self { orders: Vec::new() }
    }

    fn add(&mut self, ticker: &str, side: &str, quantity: f64, price: f64) {
        self.orders.push((
            ticker.to_string(),
            side.to_string(),
            quantity,
            price,
        ));
    }

    async fn execute(&self, pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
        let mut ids = Vec::new();

        // Use transaction for atomicity
        let mut tx = pool.begin().await?;

        for (ticker, side, quantity, price) in &self.orders {
            // The same prepared statement is reused
            let result = sqlx::query(
                "INSERT INTO orders (ticker, side, quantity, price) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(ticker)
            .bind(side)
            .bind(quantity)
            .bind(price)
            .execute(&mut *tx)
            .await?;

            ids.push(result.last_insert_rowid());
        }

        tx.commit().await?;
        Ok(ids)
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT, side TEXT, quantity REAL, price REAL
        )"
    )
    .execute(&pool)
    .await?;

    // Create order batch
    let mut batch = OrderBatch::new();
    batch.add("BTC", "BUY", 0.1, 42000.0);
    batch.add("BTC", "BUY", 0.2, 41500.0);
    batch.add("ETH", "BUY", 1.0, 2500.0);
    batch.add("SOL", "SELL", 10.0, 100.0);

    // Execute batch insert
    let ids = batch.execute(&pool).await?;
    println!("Created {} orders: {:?}", ids.len(), ids);

    // Verify result
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;
    println!("Total orders in database: {}", count.0);

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Prepared Statements | Queries with separated parameters |
| SQL Injection | Attack through code injection in queries |
| Parameterization | Safe data passing through bind() |
| query_as | Mapping results to structs |
| query! | Compile-time query validation |
| Batch Operations | Executing multiple queries efficiently |

## Practice Exercises

1. **Injection Protection**: Write a function `search_orders(ticker: &str, min_qty: f64)` that safely searches for orders by ticker and minimum quantity. Test it with malicious input data.

2. **Portfolio CRUD**: Implement a `PortfolioManager` struct with methods:
   - `add_position(ticker: &str, quantity: f64, avg_price: f64)`
   - `update_position(ticker: &str, quantity_delta: f64, new_avg_price: f64)`
   - `remove_position(ticker: &str)`
   - `get_position(ticker: &str) -> Option<Position>`

3. **Data Aggregation**: Write prepared statements to get:
   - Top 5 most profitable trades
   - Total trading volume by ticker
   - Average entry price for open positions

4. **Search with Filters**: Implement a search function with dynamic filters:
   ```rust
   fn search_trades(
       ticker: Option<&str>,
       side: Option<&str>,
       min_pnl: Option<f64>,
       max_pnl: Option<f64>,
   ) -> Vec<Trade>
   ```

## Homework

1. **Risk Management System**: Create a `risk_limits` table and implement limit checks before order creation. Use prepared statements for all operations.

2. **Price History**: Implement a `price_history` table with columns (ticker, price, timestamp) and write efficient prepared statements for:
   - Inserting new prices (batch insert)
   - Getting the latest price
   - Calculating average price over a period

3. **Operations Audit**: Add an `audit_log` table and triggers (or manual logging) to track all changes in the orders table. All queries must use prepared statements.

## Navigation

[← Previous day](../220-delete-removing-cancelled/en.md) | [Next day →](../222-transactions-atomic-operations/en.md)
