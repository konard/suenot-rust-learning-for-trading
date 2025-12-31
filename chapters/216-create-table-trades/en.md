# Day 216: CREATE TABLE: Trades Table

## Trading Analogy

Imagine you're a trader keeping a trade journal. Each trade has specific characteristics: date, ticker, direction (buy or sell), price, quantity, commission. In the past, traders wrote everything in a paper journal where each column represented a specific data type.

In a database, **CREATE TABLE** is like creating that journal. You define in advance:
- Which columns will exist (name, ticker, price...)
- What data type each column holds (number, text, date...)
- What constraints apply (price can't be negative, quantity must be a whole number)

It's like creating the perfect trading journal where every entry follows the same structure.

## What is CREATE TABLE?

`CREATE TABLE` is an SQL command for creating a new table in a database. A table is a structured data store with predefined columns (fields) and their types.

### Basic SQL Syntax

```sql
CREATE TABLE table_name (
    column1 DATA_TYPE CONSTRAINTS,
    column2 DATA_TYPE CONSTRAINTS,
    ...
);
```

## Data Types in SQLite

| SQLite Type | Description | Trading Example |
|-------------|-------------|-----------------|
| INTEGER | Whole number | Number of shares |
| REAL | Floating-point number | Asset price |
| TEXT | String | Ticker symbol |
| BLOB | Binary data | Serialized data |
| NULL | Absence of value | Unrealized profit |

## Creating a Trades Table in Rust

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Create or open database
    let conn = Connection::open("trading.db")?;

    // Create trades table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL CHECK(price > 0),
            quantity REAL NOT NULL CHECK(quantity > 0),
            commission REAL DEFAULT 0.0,
            executed_at TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    println!("Trades table created successfully!");
    Ok(())
}
```

## Table Fields Breakdown

| Field | Type | Purpose |
|-------|------|---------|
| `id` | INTEGER PRIMARY KEY | Unique trade identifier |
| `symbol` | TEXT NOT NULL | Instrument ticker (BTC, AAPL) |
| `side` | TEXT NOT NULL | Direction: BUY or SELL |
| `price` | REAL NOT NULL | Execution price |
| `quantity` | REAL NOT NULL | Trade volume |
| `commission` | REAL DEFAULT 0.0 | Exchange commission |
| `executed_at` | TEXT NOT NULL | Execution time |
| `created_at` | TEXT | Record creation time |

## Constraints

### PRIMARY KEY — Unique Identifier

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            client_order_id TEXT UNIQUE,
            symbol TEXT NOT NULL
        )",
        [],
    )?;

    // AUTOINCREMENT guarantees uniqueness even after deletion
    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER,
            price REAL
        )",
        [],
    )?;

    println!("Tables with PRIMARY KEY created!");
    Ok(())
}
```

### NOT NULL — Required Field

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE positions (
            symbol TEXT NOT NULL,
            quantity REAL NOT NULL,
            entry_price REAL NOT NULL,
            stop_loss REAL,  -- can be NULL
            take_profit REAL -- can be NULL
        )",
        [],
    )?;

    // Attempting to insert NULL into a NOT NULL field will cause an error
    let result = conn.execute(
        "INSERT INTO positions (symbol, quantity, entry_price) VALUES (NULL, 100.0, 50000.0)",
        [],
    );

    match result {
        Ok(_) => println!("Insert successful"),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
```

### CHECK — Condition Validation

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // CHECK constraints for trading data
    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT CHECK(side IN ('BUY', 'SELL')),
            price REAL CHECK(price > 0),
            quantity REAL CHECK(quantity > 0),
            leverage INTEGER CHECK(leverage >= 1 AND leverage <= 100)
        )",
        [],
    )?;

    // This insert succeeds
    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, leverage)
         VALUES ('BTC', 'BUY', 50000.0, 0.1, 10)",
        [],
    )?;
    println!("Valid trade added");

    // This insert will cause an error — negative price
    let result = conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, leverage)
         VALUES ('BTC', 'BUY', -100.0, 0.1, 10)",
        [],
    );

    match result {
        Ok(_) => println!("Insert successful"),
        Err(e) => println!("CHECK error: {}", e),
    }

    Ok(())
}
```

### DEFAULT — Default Value

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            status TEXT DEFAULT 'PENDING',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            commission_rate REAL DEFAULT 0.001
        )",
        [],
    )?;

    // Insert only required fields
    conn.execute(
        "INSERT INTO orders (symbol, side, price, quantity)
         VALUES ('ETH', 'BUY', 3000.0, 1.0)",
        [],
    )?;

    // Verify that default values were applied
    let mut stmt = conn.prepare("SELECT symbol, status, commission_rate FROM orders")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let symbol: String = row.get(0)?;
        let status: String = row.get(1)?;
        let commission: f64 = row.get(2)?;
        println!("Symbol: {}, Status: {}, Commission: {}", symbol, status, commission);
    }

    Ok(())
}
```

### UNIQUE — Unique Values

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE api_keys (
            id INTEGER PRIMARY KEY,
            exchange TEXT NOT NULL,
            api_key TEXT UNIQUE NOT NULL,
            secret_hash TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // First key is added successfully
    conn.execute(
        "INSERT INTO api_keys (exchange, api_key, secret_hash)
         VALUES ('Binance', 'key123', 'hash456')",
        [],
    )?;
    println!("First API key added");

    // Attempting to add a duplicate will cause an error
    let result = conn.execute(
        "INSERT INTO api_keys (exchange, api_key, secret_hash)
         VALUES ('Binance', 'key123', 'hash789')",
        [],
    );

    match result {
        Ok(_) => println!("Duplicate added"),
        Err(e) => println!("UNIQUE error: {}", e),
    }

    Ok(())
}
```

## Complete Example: Trading System

```rust
use rusqlite::{Connection, Result};

fn create_trading_database() -> Result<Connection> {
    let conn = Connection::open("trading_system.db")?;

    // Instruments table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS instruments (
            id INTEGER PRIMARY KEY,
            symbol TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            instrument_type TEXT CHECK(instrument_type IN ('SPOT', 'FUTURES', 'OPTION')),
            base_currency TEXT NOT NULL,
            quote_currency TEXT NOT NULL,
            tick_size REAL NOT NULL CHECK(tick_size > 0),
            lot_size REAL NOT NULL CHECK(lot_size > 0),
            is_active INTEGER DEFAULT 1
        )",
        [],
    )?;

    // Accounts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            exchange TEXT NOT NULL,
            balance REAL DEFAULT 0.0,
            currency TEXT DEFAULT 'USDT',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Orders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            client_order_id TEXT UNIQUE,
            exchange_order_id TEXT,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            order_type TEXT NOT NULL CHECK(order_type IN ('MARKET', 'LIMIT', 'STOP', 'STOP_LIMIT')),
            price REAL,
            quantity REAL NOT NULL CHECK(quantity > 0),
            filled_quantity REAL DEFAULT 0.0,
            status TEXT DEFAULT 'NEW' CHECK(status IN ('NEW', 'PARTIALLY_FILLED', 'FILLED', 'CANCELLED', 'REJECTED')),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id)
        )",
        [],
    )?;

    // Trades table (executed orders)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL CHECK(price > 0),
            quantity REAL NOT NULL CHECK(quantity > 0),
            commission REAL DEFAULT 0.0,
            commission_asset TEXT,
            realized_pnl REAL,
            executed_at TEXT NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id),
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id)
        )",
        [],
    )?;

    // Positions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            side TEXT CHECK(side IN ('LONG', 'SHORT')),
            quantity REAL NOT NULL DEFAULT 0.0,
            entry_price REAL,
            unrealized_pnl REAL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id),
            UNIQUE(account_id, instrument_id)
        )",
        [],
    )?;

    // Price history table (OHLCV)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS price_history (
            id INTEGER PRIMARY KEY,
            instrument_id INTEGER NOT NULL,
            timeframe TEXT NOT NULL CHECK(timeframe IN ('1m', '5m', '15m', '1h', '4h', '1d')),
            open_time TEXT NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            FOREIGN KEY (instrument_id) REFERENCES instruments(id),
            UNIQUE(instrument_id, timeframe, open_time)
        )",
        [],
    )?;

    println!("Trading system database created successfully!");
    Ok(conn)
}

fn main() -> Result<()> {
    let conn = create_trading_database()?;

    // Check created tables
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )?;

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("\nCreated tables:");
    for table in tables {
        println!("  - {}", table);
    }

    Ok(())
}
```

## Foreign Keys (FOREIGN KEY)

Foreign keys link tables together, ensuring data integrity:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Enable foreign key support (disabled by default in SQLite!)
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create parent table
    conn.execute(
        "CREATE TABLE portfolios (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        )",
        [],
    )?;

    // Create child table with foreign key
    conn.execute(
        "CREATE TABLE holdings (
            id INTEGER PRIMARY KEY,
            portfolio_id INTEGER NOT NULL,
            symbol TEXT NOT NULL,
            quantity REAL NOT NULL,
            avg_price REAL NOT NULL,
            FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
                ON DELETE CASCADE
                ON UPDATE CASCADE
        )",
        [],
    )?;

    // Add portfolio
    conn.execute(
        "INSERT INTO portfolios (id, name, description) VALUES (1, 'Main', 'Long-term investments')",
        [],
    )?;

    // Add assets to portfolio
    conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (1, 'BTC', 0.5, 45000.0)",
        [],
    )?;

    conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (1, 'ETH', 5.0, 3000.0)",
        [],
    )?;

    println!("Portfolio and assets added!");

    // Attempting to add an asset to a non-existent portfolio will cause an error
    let result = conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (999, 'SOL', 10.0, 100.0)",
        [],
    );

    match result {
        Ok(_) => println!("Asset added"),
        Err(e) => println!("FOREIGN KEY error: {}", e),
    }

    Ok(())
}
```

## IF NOT EXISTS — Safe Creation

```rust
use rusqlite::{Connection, Result};

fn ensure_tables_exist(conn: &Connection) -> Result<()> {
    // Table will only be created if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            executed_at TEXT NOT NULL
        )",
        [],
    )?;

    // Can be called multiple times without errors
    println!("Trades table ready for use");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("safe_trading.db")?;

    // First call — creates table
    ensure_tables_exist(&conn)?;

    // Second call — just checks, does nothing
    ensure_tables_exist(&conn)?;

    // Third call — also no errors
    ensure_tables_exist(&conn)?;

    println!("All calls successful!");
    Ok(())
}
```

## DROP TABLE — Deleting a Table

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Create table
    conn.execute(
        "CREATE TABLE temp_trades (
            id INTEGER PRIMARY KEY,
            data TEXT
        )",
        [],
    )?;
    println!("Table created");

    // Delete table (caution! All data will be lost)
    conn.execute("DROP TABLE temp_trades", [])?;
    println!("Table deleted");

    // DROP TABLE IF EXISTS — safe deletion
    conn.execute("DROP TABLE IF EXISTS temp_trades", [])?;
    println!("Safe deletion executed (table no longer exists)");

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| CREATE TABLE | Creating a new table with defined structure |
| Data Types | INTEGER, REAL, TEXT, BLOB, NULL |
| PRIMARY KEY | Unique record identifier |
| NOT NULL | Required field |
| CHECK | Condition validation on insert |
| DEFAULT | Default value |
| UNIQUE | Unique values in column |
| FOREIGN KEY | Relationship between tables |
| IF NOT EXISTS | Safe table creation |

## Homework

1. **Exchanges Table**: Create an `exchanges` table with fields:
   - `id` — primary key
   - `name` — exchange name (unique, required)
   - `api_url` — API URL
   - `is_active` — whether exchange is active (default 1)
   - `created_at` — date added

2. **Strategies Table**: Create a `strategies` table with fields:
   - `id` — primary key
   - `name` — strategy name
   - `description` — description
   - `risk_level` — risk level (CHECK: LOW, MEDIUM, HIGH)
   - `max_position_size` — maximum position size (CHECK: > 0)
   - `is_active` — whether strategy is active

3. **Linked Tables**: Create two linked tables:
   - `watchlists` — watch lists (id, name, created_at)
   - `watchlist_items` — list items (id, watchlist_id, symbol, added_at)

   Implement the relationship via FOREIGN KEY with cascade delete.

4. **Signals Journal**: Create a `signals` table for storing trading signals:
   - `id`, `strategy_id`, `symbol`, `side` (BUY/SELL)
   - `entry_price`, `stop_loss`, `take_profit`
   - `confidence` (CHECK: from 0.0 to 1.0)
   - `created_at`, `expired_at`

   Add all necessary constraints to ensure data validity.

## Navigation

[← Previous day](../215-rusqlite-connecting/en.md) | [Next day →](../217-insert-recording-trade/en.md)
