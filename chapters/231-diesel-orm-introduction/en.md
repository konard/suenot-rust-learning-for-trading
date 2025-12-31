# Day 231: Diesel ORM: Introduction

## Trading Analogy

Imagine you're managing a trading terminal that needs to store the history of all trades, quotes, and portfolio state. You could write everything to text files, but that's slow and unreliable. It's better to use a database — it's like a professional exchange archive where every transaction is recorded, indexed, and can be quickly retrieved.

**Diesel ORM** is a "translator" between your Rust code and the database. Instead of writing raw SQL queries manually, you describe your data as Rust structs, and Diesel automatically generates the correct queries. It's like having a personal broker who understands your intentions and properly handles all the paperwork.

In trading, this is critical:
- **Reliability**: The compiler checks your queries at build time
- **Speed**: Optimized queries with zero overhead
- **Security**: SQL injection protection "out of the box"

## What is an ORM?

**ORM (Object-Relational Mapping)** is a programming technique that links objects in code to tables in the database:

| Rust Concept | Database Concept |
|--------------|------------------|
| Struct | Table |
| Struct field | Column |
| Struct instance | Row |
| Vec<Struct> | SELECT result |

## Why Diesel?

Diesel stands out among other ORMs with several features:

1. **Compile-time checking** — query errors are detected before running the program
2. **Zero-cost abstractions** — performance equivalent to raw SQL
3. **Type safety** — impossible to accidentally mix data types
4. **Migration support** — database schema versioning

## Installing Diesel

To work with Diesel, you need to install the CLI tool and add dependencies:

```bash
# Install diesel_cli (for SQLite)
cargo install diesel_cli --no-default-features --features sqlite

# Or for PostgreSQL (recommended for production)
cargo install diesel_cli --no-default-features --features postgres
```

## Project Setup

### Cargo.toml

```toml
[package]
name = "trading_db"
version = "0.1.0"
edition = "2021"

[dependencies]
diesel = { version = "2.1", features = ["sqlite", "r2d2"] }
dotenvy = "0.15"
chrono = { version = "0.4", features = ["serde"] }
```

### Database Initialization

```bash
# Create .env file with database path
echo DATABASE_URL=trading.db > .env

# Initialize Diesel
diesel setup
```

This command creates:
- A `migrations/` directory for migrations
- A `diesel.toml` file with settings

## Creating Your First Migration

A migration is a versioned change to the database schema:

```bash
diesel migration generate create_trades
```

This creates a directory `migrations/YYYY-MM-DD-HHMMSS_create_trades/` with two files:
- `up.sql` — applies the migration
- `down.sql` — rolls back the migration

### up.sql — Creating the Trades Table

```sql
-- Table for storing trading transactions
CREATE TABLE trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol VARCHAR(10) NOT NULL,
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    quantity REAL NOT NULL CHECK (quantity > 0),
    price REAL NOT NULL CHECK (price > 0),
    executed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    commission REAL NOT NULL DEFAULT 0.0
);

-- Index for fast symbol lookup
CREATE INDEX idx_trades_symbol ON trades(symbol);

-- Index for time-based queries
CREATE INDEX idx_trades_executed_at ON trades(executed_at);
```

### down.sql — Rolling Back the Migration

```sql
DROP TABLE trades;
```

### Applying the Migration

```bash
diesel migration run
```

## Schema and Models

After applying the migration, Diesel generates the `src/schema.rs` file:

```rust
// src/schema.rs (auto-generated)
diesel::table! {
    trades (id) {
        id -> Integer,
        symbol -> Text,
        side -> Text,
        quantity -> Float,
        price -> Float,
        executed_at -> Timestamp,
        commission -> Float,
    }
}
```

Now let's create models for working with data:

```rust
// src/models.rs
use diesel::prelude::*;
use chrono::NaiveDateTime;

// Model for reading from the database
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::trades)]
pub struct Trade {
    pub id: i32,
    pub symbol: String,
    pub side: String,
    pub quantity: f32,
    pub price: f32,
    pub executed_at: NaiveDateTime,
    pub commission: f32,
}

// Model for inserting into the database
#[derive(Insertable)]
#[diesel(table_name = crate::schema::trades)]
pub struct NewTrade<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub quantity: f32,
    pub price: f32,
    pub commission: f32,
}

impl Trade {
    /// Calculates the total trade value
    pub fn total_value(&self) -> f32 {
        self.quantity * self.price
    }

    /// Calculates net value including commission
    pub fn net_value(&self) -> f32 {
        let value = self.total_value();
        match self.side.as_str() {
            "buy" => value + self.commission,
            "sell" => value - self.commission,
            _ => value,
        }
    }
}
```

## Connecting to the Database

```rust
// src/lib.rs
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

/// Establishes a connection to the database
pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
```

## Basic CRUD Operations

### Create

```rust
use diesel::prelude::*;
use crate::models::{NewTrade, Trade};
use crate::schema::trades;

/// Saves a new trade to the database
pub fn create_trade(
    conn: &mut SqliteConnection,
    symbol: &str,
    side: &str,
    quantity: f32,
    price: f32,
    commission: f32,
) -> Trade {
    let new_trade = NewTrade {
        symbol,
        side,
        quantity,
        price,
        commission,
    };

    diesel::insert_into(trades::table)
        .values(&new_trade)
        .returning(Trade::as_returning())
        .get_result(conn)
        .expect("Error saving trade")
}

fn main() {
    let mut conn = establish_connection();

    // Record a Bitcoin purchase
    let trade = create_trade(
        &mut conn,
        "BTC/USDT",
        "buy",
        0.5,
        42000.0,
        10.5, // $10.50 commission
    );

    println!("Created trade #{}: {} {} {} @ {}",
        trade.id,
        trade.side,
        trade.quantity,
        trade.symbol,
        trade.price
    );
    println!("Total value: ${:.2}", trade.total_value());
    println!("With commission: ${:.2}", trade.net_value());
}
```

### Read

```rust
use diesel::prelude::*;
use crate::models::Trade;
use crate::schema::trades::dsl::*;

/// Gets all trades for a specific symbol
pub fn get_trades_by_symbol(
    conn: &mut SqliteConnection,
    ticker: &str,
) -> Vec<Trade> {
    trades
        .filter(symbol.eq(ticker))
        .order(executed_at.desc())
        .load::<Trade>(conn)
        .expect("Error loading trades")
}

/// Gets the last N trades
pub fn get_recent_trades(
    conn: &mut SqliteConnection,
    limit: i64,
) -> Vec<Trade> {
    trades
        .order(executed_at.desc())
        .limit(limit)
        .load::<Trade>(conn)
        .expect("Error loading trades")
}

/// Gets a trade by ID
pub fn get_trade_by_id(
    conn: &mut SqliteConnection,
    trade_id: i32,
) -> Option<Trade> {
    trades
        .find(trade_id)
        .first(conn)
        .optional()
        .expect("Error querying database")
}

fn main() {
    let mut conn = establish_connection();

    // Get all BTC trades
    let btc_trades = get_trades_by_symbol(&mut conn, "BTC/USDT");
    println!("Found {} BTC/USDT trades", btc_trades.len());

    for trade in &btc_trades {
        println!("  #{}: {} {} @ ${:.2}",
            trade.id,
            trade.side,
            trade.quantity,
            trade.price
        );
    }

    // Get last 5 trades
    let recent = get_recent_trades(&mut conn, 5);
    println!("\nLast 5 trades:");
    for trade in recent {
        println!("  {} {} {}", trade.symbol, trade.side, trade.quantity);
    }
}
```

### Update

```rust
use diesel::prelude::*;
use crate::schema::trades::dsl::*;

/// Updates the commission for a trade
pub fn update_trade_commission(
    conn: &mut SqliteConnection,
    trade_id: i32,
    new_commission: f32,
) -> usize {
    diesel::update(trades.find(trade_id))
        .set(commission.eq(new_commission))
        .execute(conn)
        .expect("Error updating trade")
}

/// Marks all trades for a symbol as closed (updates commission)
pub fn close_all_positions(
    conn: &mut SqliteConnection,
    ticker: &str,
    closing_commission: f32,
) -> usize {
    diesel::update(trades.filter(symbol.eq(ticker)))
        .set(commission.eq(closing_commission))
        .execute(conn)
        .expect("Error closing positions")
}

fn main() {
    let mut conn = establish_connection();

    // Update commission for trade #1
    let updated = update_trade_commission(&mut conn, 1, 15.0);
    println!("Updated records: {}", updated);
}
```

### Delete

```rust
use diesel::prelude::*;
use crate::schema::trades::dsl::*;

/// Deletes a trade by ID
pub fn delete_trade(
    conn: &mut SqliteConnection,
    trade_id: i32,
) -> usize {
    diesel::delete(trades.find(trade_id))
        .execute(conn)
        .expect("Error deleting trade")
}

/// Deletes all trades older than the specified date
pub fn delete_old_trades(
    conn: &mut SqliteConnection,
    before: NaiveDateTime,
) -> usize {
    diesel::delete(trades.filter(executed_at.lt(before)))
        .execute(conn)
        .expect("Error deleting old trades")
}

fn main() {
    let mut conn = establish_connection();

    // Delete trade #1
    let deleted = delete_trade(&mut conn, 1);
    println!("Deleted records: {}", deleted);
}
```

## Practical Example: Trading Journal

```rust
use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc};

mod schema;
mod models;

use models::{Trade, NewTrade};
use schema::trades;

/// Structure for managing a trading journal
pub struct TradingJournal {
    conn: SqliteConnection,
}

impl TradingJournal {
    pub fn new(database_url: &str) -> Self {
        let conn = SqliteConnection::establish(database_url)
            .expect("Failed to connect to database");
        TradingJournal { conn }
    }

    /// Records a buy trade
    pub fn record_buy(
        &mut self,
        symbol: &str,
        quantity: f32,
        price: f32,
        commission: f32,
    ) -> Trade {
        let new_trade = NewTrade {
            symbol,
            side: "buy",
            quantity,
            price,
            commission,
        };

        diesel::insert_into(trades::table)
            .values(&new_trade)
            .returning(Trade::as_returning())
            .get_result(&mut self.conn)
            .expect("Error recording buy")
    }

    /// Records a sell trade
    pub fn record_sell(
        &mut self,
        symbol: &str,
        quantity: f32,
        price: f32,
        commission: f32,
    ) -> Trade {
        let new_trade = NewTrade {
            symbol,
            side: "sell",
            quantity,
            price,
            commission,
        };

        diesel::insert_into(trades::table)
            .values(&new_trade)
            .returning(Trade::as_returning())
            .get_result(&mut self.conn)
            .expect("Error recording sell")
    }

    /// Calculates total P&L for a symbol
    pub fn calculate_pnl(&mut self, ticker: &str) -> f32 {
        use schema::trades::dsl::*;

        let ticker_trades: Vec<Trade> = trades
            .filter(symbol.eq(ticker))
            .load(&mut self.conn)
            .expect("Error loading trades");

        let mut pnl = 0.0;
        for trade in ticker_trades {
            match trade.side.as_str() {
                "buy" => pnl -= trade.net_value(),
                "sell" => pnl += trade.net_value(),
                _ => {}
            }
        }
        pnl
    }

    /// Gets statistics for all trades
    pub fn get_statistics(&mut self) -> TradingStats {
        use schema::trades::dsl::*;

        let all_trades: Vec<Trade> = trades
            .load(&mut self.conn)
            .expect("Error loading trades");

        let total_trades = all_trades.len();
        let buy_trades = all_trades.iter().filter(|t| t.side == "buy").count();
        let sell_trades = all_trades.iter().filter(|t| t.side == "sell").count();

        let total_volume: f32 = all_trades.iter()
            .map(|t| t.total_value())
            .sum();

        let total_commission: f32 = all_trades.iter()
            .map(|t| t.commission)
            .sum();

        TradingStats {
            total_trades,
            buy_trades,
            sell_trades,
            total_volume,
            total_commission,
        }
    }
}

#[derive(Debug)]
pub struct TradingStats {
    pub total_trades: usize,
    pub buy_trades: usize,
    pub sell_trades: usize,
    pub total_volume: f32,
    pub total_commission: f32,
}

fn main() {
    let mut journal = TradingJournal::new("trading.db");

    // Record a series of trades
    println!("=== Recording Trades ===\n");

    let buy1 = journal.record_buy("BTC/USDT", 0.5, 40000.0, 10.0);
    println!("Buy: {} BTC @ ${}", buy1.quantity, buy1.price);

    let buy2 = journal.record_buy("BTC/USDT", 0.3, 41000.0, 6.15);
    println!("Buy: {} BTC @ ${}", buy2.quantity, buy2.price);

    let sell1 = journal.record_sell("BTC/USDT", 0.8, 43000.0, 17.2);
    println!("Sell: {} BTC @ ${}", sell1.quantity, sell1.price);

    // Calculate P&L
    println!("\n=== Position Analysis ===\n");

    let pnl = journal.calculate_pnl("BTC/USDT");
    println!("BTC/USDT P&L: ${:.2}", pnl);

    // Get statistics
    let stats = journal.get_statistics();
    println!("\n=== Trading Statistics ===\n");
    println!("Total trades: {}", stats.total_trades);
    println!("Buys: {}", stats.buy_trades);
    println!("Sells: {}", stats.sell_trades);
    println!("Total volume: ${:.2}", stats.total_volume);
    println!("Total commission: ${:.2}", stats.total_commission);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| ORM | Object-Relational Mapping — linking code objects to DB tables |
| Diesel | Type-safe ORM for Rust with compile-time checking |
| Migrations | Versioned changes to the database schema |
| `Queryable` | Trait for reading data from DB |
| `Insertable` | Trait for inserting data into DB |
| `diesel::insert_into` | Function to create INSERT query |
| `diesel::update` | Function to create UPDATE query |
| `diesel::delete` | Function to create DELETE query |

## Practical Exercises

1. **Create a quotes table**: Write a migration for a `quotes` table with fields: `id`, `symbol`, `bid`, `ask`, `timestamp`. Create the corresponding models.

2. **Extend the trading journal**: Add a method `get_trades_in_range(start: NaiveDateTime, end: NaiveDateTime)` to get trades within a specified time range.

3. **Data aggregation**: Write a function that groups trades by symbol and returns the total volume for each.

## Homework

1. **Trader's Portfolio**: Create a `portfolio` table to store current positions (symbol, quantity, avg_price). Implement methods to update the position with each trade.

2. **Quote History**: Create a system for saving and loading historical OHLCV data (Open, High, Low, Close, Volume). Implement a query to get candles for a specified period.

3. **Risk Management**: Create a `risk_limits` table with fields: symbol, max_position, max_loss_daily. Write a function to check if a new trade exceeds the established limits.

4. **Order Journal**: Extend the system by adding an `orders` table (id, symbol, side, quantity, price, status, created_at, filled_at). Implement the order lifecycle: creation → execution → trade recording.

## Navigation

[← Previous day](../230-database-introduction/en.md) | [Next day →](../232-diesel-queries/en.md)
