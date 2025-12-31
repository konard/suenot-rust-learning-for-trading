# Day 214: SQLite: Embedded Database for Bot

## Trading Analogy

Imagine you keep a trading journal. Every day you write in your notebook: what trades you made, at what prices you bought and sold, what the results were. But a notebook is easy to lose, it's hard to quickly find a specific entry, and it's impossible to query "all profitable trades from last month."

**SQLite** is like a smart electronic journal that:
- Stores all data in a single file (like a notebook)
- Allows instant searching for entries (indexes)
- Can filter and group data (SQL queries)
- Doesn't require a separate server (embedded database)

In real trading, SQLite is used for:
- Storing trade history locally
- Caching data from exchanges
- Saving bot settings and state
- Logging events for analysis

## What is SQLite?

SQLite is an **embedded** relational database. This means:

1. **Serverless** — the database works as a library inside your application
2. **Single file** — the entire database is stored in one `.db` file
3. **Zero configuration** — no installation or setup required
4. **ACID-compliant** — reliability like "grown-up" databases
5. **Cross-platform** — database file can be transferred between systems

## Why SQLite is Perfect for a Trading Bot?

```
┌─────────────────────────────────────────────────────────┐
│                     Trading Bot                         │
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Trading   │    │    Risk     │    │   Reports   │ │
│  │   Strategy  │    │ Management  │    │ & Analysis  │ │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘ │
│         │                  │                  │         │
│         └──────────────────┼──────────────────┘         │
│                            │                            │
│                   ┌────────▼────────┐                   │
│                   │     SQLite      │                   │
│                   │   trades.db     │                   │
│                   └─────────────────┘                   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Comparing SQLite with Other Solutions

| Feature | File/JSON | SQLite | PostgreSQL |
|---------|-----------|--------|------------|
| Setup | None | None | Complex |
| Server | No | No | Yes |
| Search | Slow | Fast | Very Fast |
| Transactions | No | Yes | Yes |
| Concurrent Writes | No | Limited | Yes |
| Data Volume | Small | Medium | Any |
| For Bot | ❌ | ✅ | For Production |

## Installing rusqlite

Add to `Cargo.toml`:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

The `bundled` flag means SQLite will be compiled with your program — no need to install SQLite on the system.

## First Connection to Database

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Create or open a database
    let conn = Connection::open("trading.db")?;

    println!("Database opened successfully!");
    println!("SQLite version: {}", rusqlite::version());

    Ok(())
}
```

## In-Memory Database

For testing, you can use an in-memory database:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Database only in RAM — will disappear after program exits
    let conn = Connection::open_in_memory()?;

    println!("In-memory database created!");

    Ok(())
}
```

## Creating a Trades Table

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Create a table to store trades
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol      TEXT NOT NULL,
            side        TEXT NOT NULL CHECK(side IN ('buy', 'sell')),
            price       REAL NOT NULL,
            quantity    REAL NOT NULL,
            timestamp   INTEGER NOT NULL,
            pnl         REAL,
            status      TEXT DEFAULT 'open'
        )",
        (), // No parameters
    )?;

    println!("Trades table created!");

    Ok(())
}
```

## Inserting a Trade

```rust
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn insert_trade(conn: &Connection, trade: &Trade) -> Result<i64> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![trade.symbol, trade.side, trade.price, trade.quantity, timestamp],
    )?;

    // Return the ID of the inserted record
    Ok(conn.last_insert_rowid())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Create table (if not exists)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol      TEXT NOT NULL,
            side        TEXT NOT NULL,
            price       REAL NOT NULL,
            quantity    REAL NOT NULL,
            timestamp   INTEGER NOT NULL,
            pnl         REAL,
            status      TEXT DEFAULT 'open'
        )",
        (),
    )?;

    // Insert a trade
    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: "buy".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    let trade_id = insert_trade(&conn, &trade)?;
    println!("Trade recorded with ID: {}", trade_id);

    Ok(())
}
```

## Reading Trades

```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
    pnl: Option<f64>,
    status: String,
}

fn get_all_trades(conn: &Connection) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
         FROM trades
         ORDER BY timestamp DESC"
    )?;

    let trade_iter = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            timestamp: row.get(5)?,
            pnl: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    let mut trades = Vec::new();
    for trade in trade_iter {
        trades.push(trade?);
    }

    Ok(trades)
}

fn get_trades_by_symbol(conn: &Connection, symbol: &str) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
         FROM trades
         WHERE symbol = ?1
         ORDER BY timestamp DESC"
    )?;

    let trade_iter = stmt.query_map(params![symbol], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            timestamp: row.get(5)?,
            pnl: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    let mut trades = Vec::new();
    for trade in trade_iter {
        trades.push(trade?);
    }

    Ok(trades)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Get all trades
    let all_trades = get_all_trades(&conn)?;
    println!("Total trades: {}", all_trades.len());

    for trade in &all_trades {
        println!("{:?}", trade);
    }

    // Get trades for a specific symbol
    let btc_trades = get_trades_by_symbol(&conn, "BTC/USDT")?;
    println!("\nBTC/USDT trades: {}", btc_trades.len());

    Ok(())
}
```

## Updating a Trade

```rust
use rusqlite::{Connection, Result, params};

fn close_trade(conn: &Connection, trade_id: i64, pnl: f64) -> Result<usize> {
    let rows_updated = conn.execute(
        "UPDATE trades
         SET status = 'closed', pnl = ?1
         WHERE id = ?2",
        params![pnl, trade_id],
    )?;

    Ok(rows_updated)
}

fn update_trade_price(conn: &Connection, trade_id: i64, new_price: f64) -> Result<usize> {
    let rows_updated = conn.execute(
        "UPDATE trades
         SET price = ?1
         WHERE id = ?2 AND status = 'open'",
        params![new_price, trade_id],
    )?;

    Ok(rows_updated)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Close a trade with profit
    let updated = close_trade(&conn, 1, 150.50)?;
    println!("Rows updated: {}", updated);

    Ok(())
}
```

## Deleting a Trade

```rust
use rusqlite::{Connection, Result, params};

fn delete_trade(conn: &Connection, trade_id: i64) -> Result<usize> {
    let rows_deleted = conn.execute(
        "DELETE FROM trades WHERE id = ?1",
        params![trade_id],
    )?;

    Ok(rows_deleted)
}

fn delete_old_trades(conn: &Connection, before_timestamp: i64) -> Result<usize> {
    let rows_deleted = conn.execute(
        "DELETE FROM trades WHERE timestamp < ?1 AND status = 'closed'",
        params![before_timestamp],
    )?;

    Ok(rows_deleted)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Delete a specific trade
    let deleted = delete_trade(&conn, 5)?;
    println!("Rows deleted: {}", deleted);

    Ok(())
}
```

## Practical Example: Trade Tracker

```rust
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Trade {
    id: Option<i64>,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
    pnl: Option<f64>,
    status: String,
}

struct TradeTracker {
    conn: Connection,
}

impl TradeTracker {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol      TEXT NOT NULL,
                side        TEXT NOT NULL,
                price       REAL NOT NULL,
                quantity    REAL NOT NULL,
                timestamp   INTEGER NOT NULL,
                pnl         REAL,
                status      TEXT DEFAULT 'open'
            )",
            (),
        )?;

        // Create indexes for fast lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_symbol
             ON trades(symbol)",
            (),
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_status
             ON trades(status)",
            (),
        )?;

        Ok(TradeTracker { conn })
    }

    fn record_trade(&self, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<i64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO trades (symbol, side, price, quantity, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![symbol, side, price, quantity, timestamp],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn close_trade(&self, trade_id: i64, exit_price: f64) -> Result<f64> {
        // Get trade information
        let mut stmt = self.conn.prepare(
            "SELECT side, price, quantity FROM trades WHERE id = ?1 AND status = 'open'"
        )?;

        let trade_info: (String, f64, f64) = stmt.query_row(params![trade_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let (side, entry_price, quantity) = trade_info;

        // Calculate PnL
        let pnl = if side == "buy" {
            (exit_price - entry_price) * quantity
        } else {
            (entry_price - exit_price) * quantity
        };

        // Update the trade
        self.conn.execute(
            "UPDATE trades SET status = 'closed', pnl = ?1 WHERE id = ?2",
            params![pnl, trade_id],
        )?;

        Ok(pnl)
    }

    fn get_open_trades(&self) -> Result<Vec<Trade>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
             FROM trades
             WHERE status = 'open'
             ORDER BY timestamp DESC"
        )?;

        let trades = stmt.query_map([], |row| {
            Ok(Trade {
                id: Some(row.get(0)?),
                symbol: row.get(1)?,
                side: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
                timestamp: row.get(5)?,
                pnl: row.get(6)?,
                status: row.get(7)?,
            })
        })?;

        trades.collect()
    }

    fn get_total_pnl(&self) -> Result<f64> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(pnl), 0.0) FROM trades WHERE status = 'closed'"
        )?;

        let total: f64 = stmt.query_row([], |row| row.get(0))?;
        Ok(total)
    }

    fn get_statistics(&self) -> Result<TradingStats> {
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), 0) as winning,
                COALESCE(SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END), 0) as losing,
                COALESCE(SUM(pnl), 0.0) as total_pnl,
                COALESCE(MAX(pnl), 0.0) as best_trade,
                COALESCE(MIN(pnl), 0.0) as worst_trade
             FROM trades
             WHERE status = 'closed'"
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(TradingStats {
                total_trades: row.get(0)?,
                winning_trades: row.get(1)?,
                losing_trades: row.get(2)?,
                total_pnl: row.get(3)?,
                best_trade: row.get(4)?,
                worst_trade: row.get(5)?,
            })
        })?;

        Ok(stats)
    }
}

#[derive(Debug)]
struct TradingStats {
    total_trades: i64,
    winning_trades: i64,
    losing_trades: i64,
    total_pnl: f64,
    best_trade: f64,
    worst_trade: f64,
}

impl TradingStats {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }
}

fn main() -> Result<()> {
    let tracker = TradeTracker::new("trading_bot.db")?;

    // Record some trades
    println!("=== Recording trades ===");

    let trade1 = tracker.record_trade("BTC/USDT", "buy", 42000.0, 0.5)?;
    println!("Opened trade #{}: BTC/USDT buy @ 42000", trade1);

    let trade2 = tracker.record_trade("ETH/USDT", "buy", 2800.0, 2.0)?;
    println!("Opened trade #{}: ETH/USDT buy @ 2800", trade2);

    let trade3 = tracker.record_trade("BTC/USDT", "sell", 43000.0, 0.3)?;
    println!("Opened trade #{}: BTC/USDT sell @ 43000", trade3);

    // Show open positions
    println!("\n=== Open positions ===");
    for trade in tracker.get_open_trades()? {
        println!("  #{}: {} {} {} @ {} (qty: {})",
            trade.id.unwrap(),
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.quantity
        );
    }

    // Close trades
    println!("\n=== Closing trades ===");

    let pnl1 = tracker.close_trade(trade1, 43500.0)?;
    println!("Trade #{} closed, PnL: ${:.2}", trade1, pnl1);

    let pnl2 = tracker.close_trade(trade2, 2750.0)?;
    println!("Trade #{} closed, PnL: ${:.2}", trade2, pnl2);

    let pnl3 = tracker.close_trade(trade3, 42500.0)?;
    println!("Trade #{} closed, PnL: ${:.2}", trade3, pnl3);

    // Show statistics
    println!("\n=== Trading statistics ===");
    let stats = tracker.get_statistics()?;
    println!("Total trades: {}", stats.total_trades);
    println!("Winning trades: {}", stats.winning_trades);
    println!("Losing trades: {}", stats.losing_trades);
    println!("Win Rate: {:.1}%", stats.win_rate());
    println!("Total PnL: ${:.2}", stats.total_pnl);
    println!("Best trade: ${:.2}", stats.best_trade);
    println!("Worst trade: ${:.2}", stats.worst_trade);

    Ok(())
}
```

## Transactions for Atomic Operations

```rust
use rusqlite::{Connection, Result, params, Transaction};

fn transfer_funds(conn: &mut Connection, from_asset: &str, to_asset: &str, amount: f64) -> Result<()> {
    // Begin transaction
    let tx = conn.transaction()?;

    // Deduct from one asset
    tx.execute(
        "UPDATE portfolio SET balance = balance - ?1 WHERE asset = ?2",
        params![amount, from_asset],
    )?;

    // Credit to another
    tx.execute(
        "UPDATE portfolio SET balance = balance + ?1 WHERE asset = ?2",
        params![amount, to_asset],
    )?;

    // Commit only if both queries succeed
    tx.commit()?;

    println!("Transfer of {} from {} to {} completed", amount, from_asset, to_asset);
    Ok(())
}

fn execute_order_atomic(conn: &mut Connection, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<i64> {
    let tx = conn.transaction()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Record the trade
    tx.execute(
        "INSERT INTO trades (symbol, side, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![symbol, side, price, quantity, timestamp],
    )?;

    let trade_id = tx.last_insert_rowid();

    // Update position
    let qty_change = if side == "buy" { quantity } else { -quantity };

    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        params![symbol, qty_change],
    )?;

    // Log the event
    tx.execute(
        "INSERT INTO trade_log (trade_id, event, timestamp) VALUES (?1, 'executed', ?2)",
        params![trade_id, timestamp],
    )?;

    tx.commit()?;

    Ok(trade_id)
}

fn main() -> Result<()> {
    let mut conn = Connection::open("trading.db")?;

    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS portfolio (
            asset TEXT PRIMARY KEY,
            balance REAL NOT NULL DEFAULT 0
        )",
        (),
    )?;

    // Initialize balance
    conn.execute("INSERT OR REPLACE INTO portfolio (asset, balance) VALUES ('USDT', 10000)", ())?;
    conn.execute("INSERT OR REPLACE INTO portfolio (asset, balance) VALUES ('BTC', 0)", ())?;

    // Perform transfer
    transfer_funds(&mut conn, "USDT", "BTC", 1000.0)?;

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SQLite | Embedded database in a single file |
| rusqlite | Rust library for working with SQLite |
| Connection | Connection to the database |
| execute() | Execute SQL without result |
| prepare() + query_map() | Execute SELECT queries |
| params![] | Safe parameter passing |
| Transaction | Atomic operations |

## Homework

1. **Order Journal**: Create an `orders` table with fields: id, symbol, side, type (market/limit), price, quantity, status, created_at, filled_at. Implement CRUD operations.

2. **Portfolio with History**: Create a system with two tables:
   - `portfolio` — current asset balances
   - `portfolio_history` — history of changes (trigger on INSERT/UPDATE)

   Implement a function to get balance at a specific date.

3. **Trade Analysis**: Write SQL queries for:
   - Top 5 most profitable symbols
   - Average PnL by day of week
   - Trades with anomalously large volume (> 2 standard deviations)

4. **Order Book Cache**: Create a structure for storing order book in SQLite:
   - Table `order_book` with fields: symbol, side, price, quantity, updated_at
   - Functions to update and get top-N levels
   - Cleanup of stale records (older than 1 minute)

## Navigation

[← Previous day](../213-why-database-persistence/en.md) | [Next day →](../215-rusqlite-connecting/en.md)
