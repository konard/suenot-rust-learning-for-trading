# Day 224: Migrations: Schema Evolution

## Trading Analogy

Imagine you've launched a trading bot that records trades in a database. Initially, you only need price and volume. But then you realize you want to add commission, order type, and execution time. You can't just delete the old database — it contains valuable trade history! You need to **migrate** the schema while preserving all data.

It's like upgrading a stock exchange's trading system: you need to add new fields for new order types, but you can't stop trading or lose historical data. Migrations are a mechanism for safely updating database structure.

In real trading, this happens constantly:
- Adding a `stop_loss` field to the orders table
- Splitting the trades table into `trades` and `executions`
- Adding new indexes to speed up analytical queries
- Changing price data type from `REAL` to `DECIMAL` for precision

## What Are Migrations?

A **migration** is a script that modifies the database schema. Migrations are:

1. **Versioned** — each migration has a unique identifier
2. **Sequential** — applied strictly in order
3. **Reversible** — changes can be rolled back
4. **Idempotent** — reapplying doesn't break the system

```
Migration 001: CREATE TABLE trades
Migration 002: ADD COLUMN commission
Migration 003: CREATE INDEX idx_trades_date
Migration 004: ADD COLUMN order_type
```

## Simple Migrations with rusqlite

Let's start with a manual migration implementation:

```rust
use rusqlite::{Connection, Result};

/// Structure for tracking schema version
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Create version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    // Get current version
    let version: Result<i32> = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    );

    version.or(Ok(0))
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}

/// Migration 1: Create basic trades table
fn migration_001_create_trades(conn: &Connection) -> Result<()> {
    println!("Applying migration 001: creating trades table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

/// Migration 2: Add commission field
fn migration_002_add_commission(conn: &Connection) -> Result<()> {
    println!("Applying migration 002: adding commission field");

    conn.execute(
        "ALTER TABLE trades ADD COLUMN commission REAL DEFAULT 0.0",
        [],
    )?;

    Ok(())
}

/// Migration 3: Add date index
fn migration_003_add_date_index(conn: &Connection) -> Result<()> {
    println!("Applying migration 003: creating date index");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_trades_created_at
         ON trades(created_at)",
        [],
    )?;

    Ok(())
}

/// Migration 4: Add order type
fn migration_004_add_order_type(conn: &Connection) -> Result<()> {
    println!("Applying migration 004: adding order_type field");

    conn.execute(
        "ALTER TABLE trades ADD COLUMN order_type TEXT DEFAULT 'MARKET'",
        [],
    )?;

    Ok(())
}

/// Apply all migrations sequentially
fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_schema_version(conn)?;
    println!("Current schema version: {}", current_version);

    let migrations: Vec<(i32, fn(&Connection) -> Result<()>)> = vec![
        (1, migration_001_create_trades),
        (2, migration_002_add_commission),
        (3, migration_003_add_date_index),
        (4, migration_004_add_order_type),
    ];

    for (version, migration_fn) in migrations {
        if version > current_version {
            // Execute migration in transaction
            let tx = conn.unchecked_transaction()?;
            migration_fn(conn)?;
            set_schema_version(conn, version)?;
            tx.commit()?;
            println!("Migration {} applied successfully", version);
        }
    }

    println!("All migrations applied. Schema version: {}",
             get_schema_version(conn)?);
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    run_migrations(&conn)?;

    // Test data insertion
    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, commission, order_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        ("BTC/USDT", "BUY", 42000.0, 0.5, 21.0, "LIMIT"),
    )?;

    println!("Trade added successfully!");

    Ok(())
}
```

## Migrations with Transactions and Rollback

For safe migrations, it's important to use transactions:

```rust
use rusqlite::{Connection, Result, Transaction};

struct Migration {
    version: i32,
    name: String,
    up: String,    // SQL to apply
    down: String,  // SQL to rollback
}

impl Migration {
    fn new(version: i32, name: &str, up: &str, down: &str) -> Self {
        Migration {
            version,
            name: name.to_string(),
            up: up.to_string(),
            down: down.to_string(),
        }
    }
}

struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    fn new() -> Self {
        MigrationRunner {
            migrations: Vec::new(),
        }
    }

    fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    fn init_schema_tracking(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    fn get_applied_versions(&self, conn: &Connection) -> Result<Vec<i32>> {
        let mut stmt = conn.prepare(
            "SELECT version FROM schema_migrations ORDER BY version"
        )?;

        let versions = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<i32>>>()?;

        Ok(versions)
    }

    fn apply_migration(&self, conn: &Connection, migration: &Migration) -> Result<()> {
        println!("Applying migration {}: {}", migration.version, migration.name);

        // Execute migration SQL
        conn.execute_batch(&migration.up)?;

        // Record applied migration
        conn.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            (&migration.version, &migration.name),
        )?;

        Ok(())
    }

    fn rollback_migration(&self, conn: &Connection, migration: &Migration) -> Result<()> {
        println!("Rolling back migration {}: {}", migration.version, migration.name);

        // Execute rollback
        conn.execute_batch(&migration.down)?;

        // Remove migration record
        conn.execute(
            "DELETE FROM schema_migrations WHERE version = ?1",
            [migration.version],
        )?;

        Ok(())
    }

    fn migrate(&self, conn: &mut Connection) -> Result<()> {
        self.init_schema_tracking(conn)?;
        let applied = self.get_applied_versions(conn)?;

        for migration in &self.migrations {
            if !applied.contains(&migration.version) {
                let tx = conn.transaction()?;
                self.apply_migration(&tx, migration)?;
                tx.commit()?;
            }
        }

        Ok(())
    }

    fn rollback(&self, conn: &mut Connection, steps: usize) -> Result<()> {
        self.init_schema_tracking(conn)?;
        let mut applied = self.get_applied_versions(conn)?;
        applied.reverse(); // From newest to oldest

        for (i, version) in applied.iter().enumerate() {
            if i >= steps {
                break;
            }

            if let Some(migration) = self.migrations.iter()
                .find(|m| m.version == *version)
            {
                let tx = conn.transaction()?;
                self.rollback_migration(&tx, migration)?;
                tx.commit()?;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut runner = MigrationRunner::new();

    // Add migrations for trading system
    runner.add_migration(Migration::new(
        1,
        "create_orders_table",
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            order_type TEXT NOT NULL,
            price REAL,
            quantity REAL NOT NULL,
            status TEXT DEFAULT 'NEW',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        "DROP TABLE orders",
    ));

    runner.add_migration(Migration::new(
        2,
        "create_positions_table",
        "CREATE TABLE positions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT UNIQUE NOT NULL,
            quantity REAL NOT NULL DEFAULT 0,
            avg_price REAL NOT NULL DEFAULT 0,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        "DROP TABLE positions",
    ));

    runner.add_migration(Migration::new(
        3,
        "add_order_client_id",
        "ALTER TABLE orders ADD COLUMN client_order_id TEXT",
        // SQLite can't drop columns directly
        // So rollback here is complex - we recreate the table
        "CREATE TABLE orders_backup AS SELECT
            id, symbol, side, order_type, price, quantity, status, created_at
         FROM orders;
         DROP TABLE orders;
         ALTER TABLE orders_backup RENAME TO orders",
    ));

    runner.add_migration(Migration::new(
        4,
        "add_indexes",
        "CREATE INDEX idx_orders_symbol ON orders(symbol);
         CREATE INDEX idx_orders_status ON orders(status);
         CREATE INDEX idx_positions_symbol ON positions(symbol)",
        "DROP INDEX idx_orders_symbol;
         DROP INDEX idx_orders_status;
         DROP INDEX idx_positions_symbol",
    ));

    let mut conn = Connection::open("trading_advanced.db")?;

    // Apply all migrations
    println!("=== Applying migrations ===");
    runner.migrate(&mut conn)?;

    // Rollback last migration (for demonstration)
    println!("\n=== Rolling back last migration ===");
    runner.rollback(&mut conn, 1)?;

    // Apply again
    println!("\n=== Applying migrations again ===");
    runner.migrate(&mut conn)?;

    println!("\nMigrations completed successfully!");

    Ok(())
}
```

## Safe Migration Patterns

### 1. Adding a Column with Default Value

```rust
use rusqlite::{Connection, Result};

/// Safely adding a new column
fn add_stop_loss_column(conn: &Connection) -> Result<()> {
    // First add column with NULL
    conn.execute(
        "ALTER TABLE orders ADD COLUMN stop_loss REAL",
        [],
    )?;

    // Then update existing records
    conn.execute(
        "UPDATE orders SET stop_loss = price * 0.95 WHERE stop_loss IS NULL",
        [],
    )?;

    println!("Column stop_loss added and populated");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Create table
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            price REAL NOT NULL
        )",
        [],
    )?;

    // Add test data
    conn.execute("INSERT INTO orders (symbol, price) VALUES ('BTC', 42000.0)", [])?;
    conn.execute("INSERT INTO orders (symbol, price) VALUES ('ETH', 2800.0)", [])?;

    // Apply migration
    add_stop_loss_column(&conn)?;

    // Verify result
    let mut stmt = conn.prepare("SELECT symbol, price, stop_loss FROM orders")?;
    let orders = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;

    for order in orders {
        let (symbol, price, stop_loss) = order?;
        println!("{}: price={}, stop_loss={}", symbol, price, stop_loss);
    }

    Ok(())
}
```

### 2. Renaming Column (via Copy)

```rust
use rusqlite::{Connection, Result};

/// SQLite doesn't support RENAME COLUMN directly (before version 3.25)
/// We use table recreation
fn rename_column_safe(conn: &Connection) -> Result<()> {
    // Start transaction
    conn.execute("BEGIN TRANSACTION", [])?;

    // 1. Create new table with correct column names
    conn.execute(
        "CREATE TABLE trades_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            entry_price REAL NOT NULL,  -- was 'price'
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 2. Copy data
    conn.execute(
        "INSERT INTO trades_new (id, symbol, side, entry_price, quantity, created_at)
         SELECT id, symbol, side, price, quantity, created_at FROM trades",
        [],
    )?;

    // 3. Drop old table
    conn.execute("DROP TABLE trades", [])?;

    // 4. Rename new table
    conn.execute("ALTER TABLE trades_new RENAME TO trades", [])?;

    // 5. Recreate indexes
    conn.execute(
        "CREATE INDEX idx_trades_symbol ON trades(symbol)",
        [],
    )?;

    conn.execute("COMMIT", [])?;

    println!("Column price renamed to entry_price");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity) VALUES ('BTC', 'BUY', 42000.0, 0.5)",
        [],
    )?;

    rename_column_safe(&conn)?;

    // Verify new structure
    let mut stmt = conn.prepare("SELECT symbol, entry_price FROM trades")?;
    let trades = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;

    for trade in trades {
        let (symbol, entry_price) = trade?;
        println!("Trade: {} @ {}", symbol, entry_price);
    }

    Ok(())
}
```

### 3. Table Splitting (Normalization)

```rust
use rusqlite::{Connection, Result};

/// Split trades into trades + instruments
fn normalize_trades_table(conn: &Connection) -> Result<()> {
    // Create instruments table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS instruments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT UNIQUE NOT NULL,
            base_asset TEXT NOT NULL,
            quote_asset TEXT NOT NULL,
            tick_size REAL DEFAULT 0.01,
            lot_size REAL DEFAULT 0.001
        )",
        [],
    )?;

    // Populate instruments from existing trades
    conn.execute(
        "INSERT OR IGNORE INTO instruments (symbol, base_asset, quote_asset)
         SELECT DISTINCT symbol,
                SUBSTR(symbol, 1, INSTR(symbol, '/') - 1),
                SUBSTR(symbol, INSTR(symbol, '/') + 1)
         FROM trades
         WHERE symbol LIKE '%/%'",
        [],
    )?;

    // Add foreign key to trades
    conn.execute(
        "ALTER TABLE trades ADD COLUMN instrument_id INTEGER
         REFERENCES instruments(id)",
        [],
    )?;

    // Populate instrument_id
    conn.execute(
        "UPDATE trades SET instrument_id = (
            SELECT id FROM instruments WHERE instruments.symbol = trades.symbol
        )",
        [],
    )?;

    println!("Table trades normalized");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL
        )",
        [],
    )?;

    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('BTC/USDT', 42000.0, 0.5)", [])?;
    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('ETH/USDT', 2800.0, 2.0)", [])?;
    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('BTC/USDT', 42100.0, 0.3)", [])?;

    normalize_trades_table(&conn)?;

    // Verify result
    let mut stmt = conn.prepare(
        "SELECT t.id, i.symbol, i.base_asset, t.price, t.quantity
         FROM trades t
         JOIN instruments i ON t.instrument_id = i.id"
    )?;

    let results = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    })?;

    println!("\nData after normalization:");
    for result in results {
        let (id, symbol, base, price, qty) = result?;
        println!("  Trade #{}: {} ({}) @ {} qty={}", id, symbol, base, price, qty);
    }

    Ok(())
}
```

## Practical Example: Migration System for Trading Bot

```rust
use rusqlite::{Connection, Result};
use std::collections::HashMap;

/// Full-featured migration system with metadata
struct TradingMigrations {
    conn: Connection,
}

impl TradingMigrations {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let migrations = TradingMigrations { conn };
        migrations.init()?;
        Ok(migrations)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version TEXT UNIQUE NOT NULL,
                description TEXT NOT NULL,
                applied_at TEXT DEFAULT CURRENT_TIMESTAMP,
                checksum TEXT,
                execution_time_ms INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    fn is_applied(&self, version: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
            [version],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn apply(&self, version: &str, description: &str, sql: &str) -> Result<()> {
        if self.is_applied(version)? {
            println!("Migration {} already applied, skipping", version);
            return Ok(());
        }

        println!("Applying migration {}: {}", version, description);

        let start = std::time::Instant::now();

        // Execute migration
        self.conn.execute_batch(sql)?;

        let elapsed = start.elapsed().as_millis() as i64;

        // Record migration info
        self.conn.execute(
            "INSERT INTO _migrations (version, description, execution_time_ms)
             VALUES (?1, ?2, ?3)",
            (version, description, elapsed),
        )?;

        println!("  Completed in {} ms", elapsed);
        Ok(())
    }

    fn run_all(&self) -> Result<()> {
        // V001: Basic tables
        self.apply(
            "V001",
            "Create accounts table",
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                exchange TEXT NOT NULL,
                api_key_hash TEXT,
                is_active INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )"
        )?;

        // V002: Balances table
        self.apply(
            "V002",
            "Create balances table",
            "CREATE TABLE IF NOT EXISTS balances (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                asset TEXT NOT NULL,
                free REAL NOT NULL DEFAULT 0,
                locked REAL NOT NULL DEFAULT 0,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(account_id, asset)
            );
            CREATE INDEX idx_balances_account ON balances(account_id)"
        )?;

        // V003: Orders table
        self.apply(
            "V003",
            "Create orders table",
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                client_order_id TEXT UNIQUE,
                exchange_order_id TEXT,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
                order_type TEXT NOT NULL,
                price REAL,
                quantity REAL NOT NULL,
                filled_quantity REAL DEFAULT 0,
                status TEXT DEFAULT 'NEW',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_orders_account ON orders(account_id);
            CREATE INDEX idx_orders_symbol ON orders(symbol);
            CREATE INDEX idx_orders_status ON orders(status)"
        )?;

        // V004: Executions table
        self.apply(
            "V004",
            "Create executions table",
            "CREATE TABLE IF NOT EXISTS executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL REFERENCES orders(id),
                exchange_trade_id TEXT,
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                commission REAL DEFAULT 0,
                commission_asset TEXT,
                executed_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_executions_order ON executions(order_id)"
        )?;

        // V005: Positions table
        self.apply(
            "V005",
            "Create positions table",
            "CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                symbol TEXT NOT NULL,
                side TEXT NOT NULL CHECK(side IN ('LONG', 'SHORT')),
                quantity REAL NOT NULL,
                entry_price REAL NOT NULL,
                current_price REAL,
                unrealized_pnl REAL,
                realized_pnl REAL DEFAULT 0,
                opened_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(account_id, symbol)
            );
            CREATE INDEX idx_positions_account ON positions(account_id)"
        )?;

        // V006: Add risk management
        self.apply(
            "V006",
            "Add risk management fields to orders",
            "ALTER TABLE orders ADD COLUMN stop_loss REAL;
             ALTER TABLE orders ADD COLUMN take_profit REAL;
             ALTER TABLE orders ADD COLUMN trailing_stop_percent REAL"
        )?;

        // V007: Strategies table
        self.apply(
            "V007",
            "Create strategies table",
            "CREATE TABLE IF NOT EXISTS strategies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                parameters TEXT,  -- JSON
                is_active INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            ALTER TABLE orders ADD COLUMN strategy_id INTEGER REFERENCES strategies(id)"
        )?;

        // V008: Add audit
        self.apply(
            "V008",
            "Create audit log table",
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id)"
        )?;

        Ok(())
    }

    fn status(&self) -> Result<()> {
        println!("\n=== Migration Status ===");

        let mut stmt = self.conn.prepare(
            "SELECT version, description, applied_at, execution_time_ms
             FROM _migrations ORDER BY version"
        )?;

        let migrations = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        for migration in migrations {
            let (version, desc, applied_at, time_ms) = migration?;
            println!("{}: {} (applied: {}, {}ms)",
                     version, desc, applied_at, time_ms);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let migrations = TradingMigrations::new("trading_bot.db")?;

    // Apply all migrations
    migrations.run_all()?;

    // Show status
    migrations.status()?;

    // Verify structure
    println!("\n=== Structure Verification ===");

    let tables: Vec<String> = migrations.conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    println!("Created tables: {:?}", tables);

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Migration | Script that modifies database schema |
| Versioning | Each migration has a unique identifier |
| Transactions | Migrations are executed atomically |
| Rollback | Ability to undo a migration |
| Metadata | Storing information about applied migrations |
| Safe Patterns | Adding with defaults, renaming via copy |

## Homework

1. **Versioning System**: Implement a migration system that supports semantic versioning (V1.0.0, V1.1.0, V2.0.0). Add compatibility checking when applying.

2. **Data Migrations**: Create a migration that not only changes the schema but also transforms data. For example, splits a `full_name` field into `first_name` and `last_name`.

3. **Integrity Check**: Add SQL code checksums to the migration system. On re-run, verify that the migration code hasn't changed since first application.

4. **Dry-run Mode**: Implement a mode that shows which migrations would be applied without actually executing them. Useful for production verification.

## Navigation

[← Previous day](../223-indexes-fast-search/en.md) | [Next day →](../225-postgresql-production/en.md)
