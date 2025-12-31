# Day 215: rusqlite: Connecting to SQLite

## Trading Analogy

Imagine you're a trader keeping a journal of all your trades in a notebook. Every time you need to find a specific trade or calculate statistics, you have to flip through all the pages. And if you lose the notebook — all your data is gone forever.

**SQLite** is like a smart electronic journal that:
- Stores all your trades in a single file
- Instantly finds information by any criteria
- Automatically creates backups
- Works without a separate server (embedded database)

**rusqlite** is the bridge between Rust and SQLite. It allows your trading bot to:
- Save trade history for analysis
- Store strategy settings
- Cache market data
- Maintain order logs

## What is rusqlite?

`rusqlite` is a Rust wrapper for the SQLite library. It provides:

| Feature | Description |
|---------|-------------|
| Type safety | Rust types when working with SQL |
| Memory safety | Automatic resource management |
| Parameterized queries | Protection against SQL injection |
| Transactions | Atomic data operations |

## Adding rusqlite to Your Project

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
```

The `bundled` flag includes SQLite directly in your binary — no need to install SQLite separately.

## Creating a Connection

### Basic Connection to a File Database

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Create or open a database
    let conn = Connection::open("trading_bot.db")?;

    println!("Database opened successfully!");

    // Connection is automatically closed when it goes out of scope
    Ok(())
}
```

### In-Memory Database (for Testing)

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // In-memory database — data disappears when program ends
    let conn = Connection::open_in_memory()?;

    println!("In-memory database created for testing");

    Ok(())
}
```

## Practical Example: Trading Bot Connection

```rust
use rusqlite::{Connection, Result};
use std::path::Path;

/// Database manager for trading bot
struct TradingDatabase {
    conn: Connection,
}

impl TradingDatabase {
    /// Creates a new database connection
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Enable foreign key checks
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        Ok(TradingDatabase { conn })
    }

    /// Creates an in-memory database for testing
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(TradingDatabase { conn })
    }

    /// Checks if the connection is active
    fn is_connected(&self) -> bool {
        self.conn.execute_batch("SELECT 1").is_ok()
    }
}

fn main() -> Result<()> {
    // Create database for trading bot
    let db = TradingDatabase::new("my_trading_bot.db")?;

    if db.is_connected() {
        println!("Trading bot successfully connected to database!");
    }

    // Test database in memory
    let test_db = TradingDatabase::new_in_memory()?;
    println!("Test database created: {}", test_db.is_connected());

    Ok(())
}
```

## Database Open Modes

```rust
use rusqlite::{Connection, OpenFlags, Result};

fn main() -> Result<()> {
    // Read-only — safe for data analysis
    let read_only = Connection::open_with_flags(
        "trading_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    println!("Opened database in read-only mode");

    // Read and write (default)
    let read_write = Connection::open_with_flags(
        "trading_data.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    println!("Opened database for reading and writing");

    // Create if doesn't exist
    let create_new = Connection::open_with_flags(
        "new_trading.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    println!("Database created or opened");

    Ok(())
}
```

## Handling Connection Errors

```rust
use rusqlite::{Connection, Error, Result};
use std::path::Path;

/// Database check result
enum DatabaseStatus {
    Ready,
    Created,
    Error(String),
}

fn check_database(path: &str) -> DatabaseStatus {
    // Check if file exists
    let exists = Path::new(path).exists();

    match Connection::open(path) {
        Ok(conn) => {
            // Check database integrity
            match conn.execute_batch("PRAGMA integrity_check;") {
                Ok(_) => {
                    if exists {
                        DatabaseStatus::Ready
                    } else {
                        DatabaseStatus::Created
                    }
                }
                Err(e) => DatabaseStatus::Error(format!("Database corrupted: {}", e)),
            }
        }
        Err(e) => DatabaseStatus::Error(format!("Failed to open: {}", e)),
    }
}

fn main() {
    let paths = vec![
        "trading_bot.db",
        "/invalid/path/database.db",
        ":memory:",
    ];

    for path in paths {
        print!("Checking '{}': ", path);
        match check_database(path) {
            DatabaseStatus::Ready => println!("Ready to use"),
            DatabaseStatus::Created => println!("New database created"),
            DatabaseStatus::Error(e) => println!("Error: {}", e),
        }
    }
}
```

## Configuration for Trading Bot

```rust
use rusqlite::{Connection, Result};

/// Database settings for high-performance trading bot
fn configure_for_trading(conn: &Connection) -> Result<()> {
    // WAL mode — allows reading while writing
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    // Synchronization — balance between speed and reliability
    // NORMAL — good compromise for trading
    conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

    // In-memory cache — speeds up frequent queries
    conn.execute_batch("PRAGMA cache_size = -64000;")?; // 64MB

    // Page size — optimal for SSD
    conn.execute_batch("PRAGMA page_size = 4096;")?;

    // Temporary tables in memory
    conn.execute_batch("PRAGMA temp_store = MEMORY;")?;

    println!("Database optimized for trading");

    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("fast_trading.db")?;
    configure_for_trading(&conn)?;

    Ok(())
}
```

## Connection Pool (for Multi-threaded Bot)

For multi-threaded applications, use `r2d2` with `rusqlite`:

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
```

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::thread;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection manager
    let manager = SqliteConnectionManager::file("trading_pool.db");

    // Create pool with 4 connections
    let pool = Arc::new(Pool::builder()
        .max_size(4)
        .build(manager)?);

    println!("Connection pool created");

    // Simulate multi-threaded work
    let mut handles = vec![];

    for i in 0..4 {
        let pool = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            let conn = pool.get().expect("Failed to get connection");
            println!("Thread {} got connection", i);

            // Simulate database work
            conn.execute_batch("SELECT 1").expect("Query error");

            println!("Thread {} finished work", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads completed");

    Ok(())
}
```

## Trading Application Structure with Database

```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

/// Trading bot with database connection
struct TradingBot {
    name: String,
    db: Arc<Mutex<Connection>>,
}

impl TradingBot {
    fn new(name: &str, db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Basic configuration
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        Ok(TradingBot {
            name: name.to_string(),
            db: Arc::new(Mutex::new(conn)),
        })
    }

    fn log_message(&self, message: &str) {
        println!("[{}] {}", self.name, message);
    }

    fn check_db_connection(&self) -> bool {
        match self.db.lock() {
            Ok(conn) => conn.execute_batch("SELECT 1").is_ok(),
            Err(_) => false,
        }
    }
}

fn main() -> Result<()> {
    let bot = TradingBot::new("CryptoTrader", "crypto_trades.db")?;

    bot.log_message("Bot started");

    if bot.check_db_connection() {
        bot.log_message("Database connection is active");
    } else {
        bot.log_message("ERROR: Database unavailable!");
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `rusqlite` | Rust wrapper for working with SQLite |
| `Connection::open()` | Opening a file database |
| `Connection::open_in_memory()` | In-memory database for testing |
| `OpenFlags` | Open mode flags |
| PRAGMA | SQLite performance settings |
| WAL mode | Improved journaling mode |
| Connection pool | r2d2 for multi-threaded applications |

## Practical Exercises

1. **Simple Connection**: Create a program that:
   - Opens a database `my_trades.db`
   - Verifies the connection works
   - Prints the SQLite version (`SELECT sqlite_version()`)

2. **Error Handling**: Write a function that:
   - Tries to open a database at the specified path
   - Returns a clear error message
   - Handles cases: file doesn't exist, no access permissions, corrupted database

3. **Database Configurator**: Create a `DatabaseConfig` struct with methods:
   - `with_wal_mode()` — enables WAL
   - `with_cache_size(mb: u32)` — sets cache size
   - `apply(conn: &Connection)` — applies settings

4. **Multi-threaded Test**: Using a connection pool:
   - Create 4 threads
   - Each thread executes 100 `SELECT 1` queries
   - Measure total execution time

## Homework

1. **Database Manager**: Implement a `DatabaseManager` struct that:
   - Stores multiple connections to different databases (trades, analytics, logs)
   - Provides a `get_connection(db_name: &str)` method to get the needed connection
   - Automatically creates databases if they don't exist

2. **Connection Monitor**: Create a component that:
   - Periodically checks connection status (every 5 seconds)
   - Automatically reconnects on connection loss
   - Logs all events (connection, disconnection, reconnection)

3. **Connection Benchmark**: Write a performance test that:
   - Compares performance with and without WAL
   - Measures the impact of cache size on performance
   - Outputs a table with results

4. **Data Migration**: Implement a function that:
   - Opens two databases (old and new)
   - Copies all data from one to the other
   - Verifies integrity after copying

## Navigation

[← Previous day](../214-sqlite-embedded-database/en.md) | [Next day →](../216-create-table-trades/en.md)
