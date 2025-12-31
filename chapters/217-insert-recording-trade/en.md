# Day 217: INSERT: Recording a Trade

## Trading Analogy

Every trade on an exchange is a story. When you buy or sell an asset, this information must be recorded in a database: price, volume, time, direction. Without recording trades, it's impossible to track profit and loss, analyze trading history, or conduct audits.

Imagine you have a trading terminal, and you just executed an order to buy 0.5 BTC at $42,000. This trade must be immediately recorded in the database:

```
Trade #12345
├── Symbol: BTC/USD
├── Direction: BUY
├── Quantity: 0.5
├── Price: 42000.00
├── Fee: 21.00
└── Time: 2024-01-15 10:30:45
```

The SQL `INSERT` statement is your tool for recording every trade in the database.

## What is INSERT?

`INSERT` is a SQL command for adding new records (rows) to a database table. In trading context, we use INSERT for:

- Recording executed trades
- Saving order history
- Logging portfolio changes
- Capturing price quotes

### Basic SQL Syntax

```sql
-- Insert a single record
INSERT INTO trades (symbol, side, quantity, price, timestamp)
VALUES ('BTC/USD', 'BUY', 0.5, 42000.00, '2024-01-15 10:30:45');

-- Insert multiple records
INSERT INTO trades (symbol, side, quantity, price, timestamp)
VALUES
    ('BTC/USD', 'BUY', 0.5, 42000.00, '2024-01-15 10:30:45'),
    ('ETH/USD', 'SELL', 2.0, 2500.00, '2024-01-15 10:31:12'),
    ('BTC/USD', 'SELL', 0.25, 42100.00, '2024-01-15 10:32:00');
```

## Working with Databases in Rust

Rust has several popular libraries for database operations:

| Library | Type | Description |
|---------|------|-------------|
| `rusqlite` | SQLite | Lightweight embedded DB |
| `sqlx` | Async | Asynchronous, compile-time verification |
| `diesel` | ORM | Type-safe ORM |
| `tokio-postgres` | PostgreSQL | Async PostgreSQL |

We'll start with `rusqlite` — a simple and straightforward library for SQLite.

## Project Setup

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Creating a Trades Table

```rust
use rusqlite::{Connection, Result};

fn create_trades_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            quantity REAL NOT NULL CHECK(quantity > 0),
            price REAL NOT NULL CHECK(price > 0),
            fee REAL DEFAULT 0,
            timestamp TEXT NOT NULL,
            strategy TEXT,
            notes TEXT
        )",
        [],
    )?;

    println!("Trades table created successfully");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;
    Ok(())
}
```

## Simple Trade Insertion

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    fee: f64,
    timestamp: String,
    strategy: Option<String>,
}

fn insert_trade(conn: &Connection, trade: &Trade) -> Result<i64> {
    conn.execute(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
    )?;

    // Return the ID of the inserted record
    Ok(conn.last_insert_rowid())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;

    let trade = Trade {
        symbol: "BTC/USD".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 42000.0,
        fee: 21.0,
        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        strategy: Some("momentum".to_string()),
    };

    let trade_id = insert_trade(&conn, &trade)?;
    println!("Trade recorded with ID: {}", trade_id);

    Ok(())
}
```

## Batch Insert of Multiple Trades

During active trading, you often need to record many trades simultaneously. Batch insertion is significantly more efficient than inserting one record at a time.

```rust
use rusqlite::{Connection, Result, params};

fn insert_trades_batch(conn: &Connection, trades: &[Trade]) -> Result<Vec<i64>> {
    let mut ids = Vec::new();

    // Use a transaction for atomicity and performance
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        for trade in trades {
            stmt.execute(params![
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.fee,
                trade.timestamp,
                trade.strategy,
            ])?;
            ids.push(tx.last_insert_rowid());
        }
    }

    tx.commit()?;

    println!("Recorded {} trades", ids.len());
    Ok(ids)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;

    let trades = vec![
        Trade {
            symbol: "BTC/USD".to_string(),
            side: "BUY".to_string(),
            quantity: 0.1,
            price: 42000.0,
            fee: 4.2,
            timestamp: "2024-01-15 10:30:00".to_string(),
            strategy: Some("scalping".to_string()),
        },
        Trade {
            symbol: "ETH/USD".to_string(),
            side: "SELL".to_string(),
            quantity: 2.0,
            price: 2500.0,
            fee: 5.0,
            timestamp: "2024-01-15 10:30:05".to_string(),
            strategy: Some("scalping".to_string()),
        },
        Trade {
            symbol: "BTC/USD".to_string(),
            side: "SELL".to_string(),
            quantity: 0.1,
            price: 42050.0,
            fee: 4.21,
            timestamp: "2024-01-15 10:30:10".to_string(),
            strategy: Some("scalping".to_string()),
        },
    ];

    let ids = insert_trades_batch(&conn, &trades)?;
    println!("Trade IDs: {:?}", ids);

    Ok(())
}
```

## Trading Engine with Trade Recording

Let's create a more realistic example — a trading engine that executes orders and automatically records trades:

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    fn as_str(&self) -> &str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

struct TradingEngine {
    conn: Connection,
    balances: HashMap<String, f64>,
    positions: HashMap<String, f64>,
    fee_rate: f64,
}

impl TradingEngine {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                fee REAL NOT NULL,
                total REAL NOT NULL,
                timestamp TEXT NOT NULL,
                pnl REAL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS balance_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                currency TEXT NOT NULL,
                amount REAL NOT NULL,
                change REAL NOT NULL,
                reason TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 100_000.0);

        Ok(TradingEngine {
            conn,
            balances,
            positions: HashMap::new(),
            fee_rate: 0.001, // 0.1% commission
        })
    }

    fn execute_order(&mut self, order: &Order) -> Result<i64> {
        let fee = order.quantity * order.price * self.fee_rate;
        let total = order.quantity * order.price;

        // Check balance
        match order.side {
            OrderSide::Buy => {
                let usd_balance = self.balances.get("USD").unwrap_or(&0.0);
                if *usd_balance < total + fee {
                    return Err(rusqlite::Error::InvalidQuery);
                }
            }
            OrderSide::Sell => {
                let position = self.positions.get(&order.symbol).unwrap_or(&0.0);
                if *position < order.quantity {
                    return Err(rusqlite::Error::InvalidQuery);
                }
            }
        }

        // Calculate PnL for sell orders
        let pnl: Option<f64> = match order.side {
            OrderSide::Sell => {
                // Simplified PnL calculation
                Some(0.0) // In reality, need to track average entry price
            }
            OrderSide::Buy => None,
        };

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        // Record trade in DB
        self.conn.execute(
            "INSERT INTO trades (symbol, side, quantity, price, fee, total, timestamp, pnl)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                order.symbol,
                order.side.as_str(),
                order.quantity,
                order.price,
                fee,
                total,
                timestamp,
                pnl,
            ],
        )?;

        let trade_id = self.conn.last_insert_rowid();

        // Update balances and positions
        match order.side {
            OrderSide::Buy => {
                *self.balances.get_mut("USD").unwrap() -= total + fee;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0) += order.quantity;
            }
            OrderSide::Sell => {
                *self.balances.get_mut("USD").unwrap() += total - fee;
                *self.positions.get_mut(&order.symbol).unwrap() -= order.quantity;
            }
        }

        // Log balance change
        let usd_balance = *self.balances.get("USD").unwrap();
        let change = match order.side {
            OrderSide::Buy => -(total + fee),
            OrderSide::Sell => total - fee,
        };

        self.conn.execute(
            "INSERT INTO balance_log (currency, amount, change, reason, timestamp)
             VALUES ('USD', ?1, ?2, ?3, ?4)",
            params![
                usd_balance,
                change,
                format!("Trade #{}", trade_id),
                timestamp,
            ],
        )?;

        println!(
            "Trade #{}: {} {} {} @ ${:.2} (fee: ${:.2})",
            trade_id,
            order.side.as_str(),
            order.quantity,
            order.symbol,
            order.price,
            fee
        );

        Ok(trade_id)
    }

    fn get_balance(&self, currency: &str) -> f64 {
        *self.balances.get(currency).unwrap_or(&0.0)
    }

    fn get_position(&self, symbol: &str) -> f64 {
        *self.positions.get(symbol).unwrap_or(&0.0)
    }
}

fn main() -> Result<()> {
    let mut engine = TradingEngine::new("trading_engine.db")?;

    println!("Initial balance: ${:.2}", engine.get_balance("USD"));

    // Series of trades
    let orders = vec![
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 0.5,
            price: 42000.0,
        },
        Order {
            symbol: "ETH/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 10.0,
            price: 2500.0,
        },
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 0.3,
            price: 41800.0,
        },
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Sell,
            quantity: 0.2,
            price: 42500.0,
        },
    ];

    for order in &orders {
        match engine.execute_order(order) {
            Ok(_) => {}
            Err(e) => println!("Order execution error: {:?}", e),
        }
    }

    println!("\nFinal USD balance: ${:.2}", engine.get_balance("USD"));
    println!("BTC position: {:.4}", engine.get_position("BTC/USD"));
    println!("ETH position: {:.4}", engine.get_position("ETH/USD"));

    Ok(())
}
```

## Error Handling for INSERT

```rust
use rusqlite::{Connection, Result, Error};

fn insert_trade_safe(conn: &Connection, trade: &Trade) -> Result<i64> {
    // Validate data before insertion
    if trade.quantity <= 0.0 {
        return Err(Error::InvalidParameterName(
            "Quantity must be positive".to_string()
        ));
    }

    if trade.price <= 0.0 {
        return Err(Error::InvalidParameterName(
            "Price must be positive".to_string()
        ));
    }

    if trade.side != "BUY" && trade.side != "SELL" {
        return Err(Error::InvalidParameterName(
            "Side must be BUY or SELL".to_string()
        ));
    }

    // Try to insert
    match conn.execute(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
    ) {
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(Error::SqliteFailure(err, msg)) => {
            eprintln!("SQLite error: {:?} - {:?}", err, msg);
            Err(Error::SqliteFailure(err, msg))
        }
        Err(e) => {
            eprintln!("Unknown error: {:?}", e);
            Err(e)
        }
    }
}
```

## INSERT with RETURNING Clause

SQLite 3.35+ supports `RETURNING` to get inserted data:

```rust
use rusqlite::{Connection, Result, params};

fn insert_trade_returning(conn: &Connection, trade: &Trade) -> Result<(i64, String)> {
    let mut stmt = conn.prepare(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING id, timestamp"
    )?;

    let (id, ts): (i64, String) = stmt.query_row(
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    println!("Inserted trade ID={} at {}", id, ts);
    Ok((id, ts))
}
```

## INSERT Performance

| Method | Speed | Use Case |
|--------|-------|----------|
| Single INSERT | Slow | Rare operations |
| Batch in transaction | Fast | Multiple records |
| Prepared Statement | Very fast | Repeated operations |
| Bulk INSERT (VALUES) | Maximum | Data import |

### Optimized Bulk Insert Example

```rust
use rusqlite::{Connection, Result};

fn bulk_insert_trades(conn: &Connection, trades: &[Trade]) -> Result<()> {
    // Disable sync for maximum speed (be careful!)
    conn.execute("PRAGMA synchronous = OFF", [])?;
    conn.execute("PRAGMA journal_mode = MEMORY", [])?;

    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        for trade in trades {
            stmt.execute(params![
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.fee,
                trade.timestamp,
                trade.strategy,
            ])?;
        }
    }

    tx.commit()?;

    // Restore safe settings
    conn.execute("PRAGMA synchronous = FULL", [])?;
    conn.execute("PRAGMA journal_mode = DELETE", [])?;

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| INSERT | SQL command for adding records |
| rusqlite | Rust library for SQLite |
| params![] | Macro for parameterized queries |
| Transaction | Grouping operations for atomicity |
| Prepared Statement | Precompiled query for speed |
| RETURNING | Return data after insertion |
| last_insert_rowid() | Get the ID of inserted record |

## Practical Exercises

1. **Quote Journal**: Create a `quotes` table and a function for recording quotes (bid, ask, spread, timestamp). Implement batch insertion for recording 1000 quotes per second.

2. **Order History**: Create an order recording system with fields: id, symbol, side, type (LIMIT/MARKET), quantity, price, status, created_at, filled_at. Implement status UPDATE on execution.

3. **Change Audit**: Create an audit table that automatically records all changes to the trades table using SQLite triggers.

## Homework

1. **Trading Journal**: Create a full-featured trading journal with:
   - Trade recording with fee calculation
   - Deposit and withdrawal recording
   - Total PnL calculation
   - CSV export

2. **Performance Optimization**: Measure insertion time for 10,000 trades using different methods:
   - Single INSERTs
   - Batch in transaction
   - With different PRAGMA settings

   Compare results and draw conclusions.

3. **Multi-threaded Recording**: Create a system with multiple threads simultaneously recording trades. Use connection pooling or proper synchronization.

4. **DB Migrations**: Implement a migration system for evolving the trades table schema:
   - Version 1: basic fields
   - Version 2: add exchange field
   - Version 3: add order_id field

## Navigation

[← Previous day](../216-select-query-trades/en.md) | [Next day →](../218-update-modify-order/en.md)
