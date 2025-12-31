# Day 225: PostgreSQL: Production Database

## Trading Analogy

Think of SQLite as a trading journal in Excel sitting on your computer. It's great for personal use and strategy testing. But when your trading bot becomes a serious business and you need reliability, scalability, and the ability to work from multiple servers — you need **PostgreSQL**.

PostgreSQL is like a professional exchange accounting system:
- **Reliability**: even during server failure, data is preserved (ACID transactions)
- **Scalability**: millions of trades per day — no problem
- **Concurrent access**: many bots and analysts can work simultaneously
- **Replication**: real-time backups on other servers

## Why PostgreSQL for Production?

### SQLite vs PostgreSQL

| Characteristic | SQLite | PostgreSQL |
|----------------|--------|------------|
| Architecture | Embedded (file) | Client-server |
| Concurrent writes | Limited | Full support |
| Scaling | Vertical | Horizontal |
| Replication | No | Yes |
| JSON support | Basic | Advanced |
| Full-text search | Limited | Full |
| Data size | Up to 140 TB | Unlimited |

### When to Switch to PostgreSQL

```
Your bot is growing:
┌─────────────────────────────────────────────┐
│ Prototype     │ SQLite — fast and simple    │
├───────────────┼─────────────────────────────┤
│ Testing       │ SQLite — convenient locally │
├───────────────┼─────────────────────────────┤
│ Production    │ PostgreSQL — reliable       │
├───────────────┼─────────────────────────────┤
│ Scale         │ PostgreSQL + replication    │
└─────────────────────────────────────────────┘
```

## Connecting to PostgreSQL from Rust

### Adding Dependencies to Cargo.toml

```toml
[dependencies]
postgres = "0.19"           # Synchronous client
tokio-postgres = "0.7"      # Asynchronous client (for the next chapter)
```

### Basic Connection

```rust
use postgres::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    // PostgreSQL connection string
    let connection_string = "host=localhost user=trader password=secret dbname=trading_db";

    // Create client
    let mut client = Client::connect(connection_string, NoTls)?;

    println!("Successfully connected to PostgreSQL!");

    // Check version
    let row = client.query_one("SELECT version()", &[])?;
    let version: &str = row.get(0);
    println!("PostgreSQL version: {}", version);

    Ok(())
}
```

## Creating Schema for Trading System

### Trades Table

```rust
use postgres::{Client, NoTls, Error};

fn create_trading_schema(client: &mut Client) -> Result<(), Error> {
    // Create trade side type
    client.batch_execute("
        DO $$ BEGIN
            CREATE TYPE trade_side AS ENUM ('buy', 'sell');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
    ")?;

    // Create order status type
    client.batch_execute("
        DO $$ BEGIN
            CREATE TYPE order_status AS ENUM ('pending', 'filled', 'cancelled', 'rejected');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
    ")?;

    // Instruments table
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS instruments (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL UNIQUE,
            name VARCHAR(100),
            exchange VARCHAR(50),
            tick_size DECIMAL(18, 8),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
    ")?;

    // Orders table
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS orders (
            id SERIAL PRIMARY KEY,
            external_id VARCHAR(100) UNIQUE,
            instrument_id INTEGER REFERENCES instruments(id),
            side trade_side NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            quantity DECIMAL(18, 8) NOT NULL,
            filled_quantity DECIMAL(18, 8) DEFAULT 0,
            status order_status DEFAULT 'pending',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
    ")?;

    // Trades table
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS trades (
            id SERIAL PRIMARY KEY,
            order_id INTEGER REFERENCES orders(id),
            instrument_id INTEGER REFERENCES instruments(id),
            side trade_side NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            quantity DECIMAL(18, 8) NOT NULL,
            commission DECIMAL(18, 8) DEFAULT 0,
            executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
    ")?;

    // Indexes for fast lookups
    client.batch_execute("
        CREATE INDEX IF NOT EXISTS idx_orders_instrument ON orders(instrument_id);
        CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
        CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);
        CREATE INDEX IF NOT EXISTS idx_trades_instrument ON trades(instrument_id);
        CREATE INDEX IF NOT EXISTS idx_trades_executed ON trades(executed_at);
    ")?;

    println!("Trading database schema created!");
    Ok(())
}

fn main() -> Result<(), Error> {
    let mut client = Client::connect(
        "host=localhost user=trader password=secret dbname=trading_db",
        NoTls
    )?;

    create_trading_schema(&mut client)?;
    Ok(())
}
```

## CRUD Operations for Trading System

### Data Models

```rust
use postgres::{Client, NoTls, Error, Row};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl fmt::Display for TradeSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeSide::Buy => write!(f, "buy"),
            TradeSide::Sell => write!(f, "sell"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug)]
pub struct Instrument {
    pub id: i32,
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
}

#[derive(Debug)]
pub struct Order {
    pub id: i32,
    pub external_id: Option<String>,
    pub instrument_id: i32,
    pub side: TradeSide,
    pub price: f64,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: OrderStatus,
}

#[derive(Debug)]
pub struct Trade {
    pub id: i32,
    pub order_id: i32,
    pub instrument_id: i32,
    pub side: TradeSide,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
}
```

### Adding an Instrument

```rust
fn add_instrument(
    client: &mut Client,
    symbol: &str,
    name: Option<&str>,
    exchange: Option<&str>,
    tick_size: f64,
) -> Result<i32, Error> {
    let row = client.query_one(
        "INSERT INTO instruments (symbol, name, exchange, tick_size)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (symbol) DO UPDATE SET name = EXCLUDED.name
         RETURNING id",
        &[&symbol, &name, &exchange, &tick_size],
    )?;

    let id: i32 = row.get(0);
    println!("Instrument {} added with id={}", symbol, id);
    Ok(id)
}

fn main() -> Result<(), Error> {
    let mut client = Client::connect(
        "host=localhost user=trader password=secret dbname=trading_db",
        NoTls
    )?;

    // Add instruments
    add_instrument(&mut client, "BTC/USDT", Some("Bitcoin"), Some("Binance"), 0.01)?;
    add_instrument(&mut client, "ETH/USDT", Some("Ethereum"), Some("Binance"), 0.01)?;
    add_instrument(&mut client, "AAPL", Some("Apple Inc"), Some("NASDAQ"), 0.01)?;

    Ok(())
}
```

### Creating and Updating an Order

```rust
fn create_order(
    client: &mut Client,
    external_id: Option<&str>,
    instrument_id: i32,
    side: TradeSide,
    price: f64,
    quantity: f64,
) -> Result<i32, Error> {
    let side_str = side.to_string();

    let row = client.query_one(
        "INSERT INTO orders (external_id, instrument_id, side, price, quantity)
         VALUES ($1, $2, $3::trade_side, $4, $5)
         RETURNING id",
        &[&external_id, &instrument_id, &side_str, &price, &quantity],
    )?;

    let id: i32 = row.get(0);
    println!("Order created: id={}, {:?} {} @ {}", id, side, quantity, price);
    Ok(id)
}

fn update_order_status(
    client: &mut Client,
    order_id: i32,
    status: OrderStatus,
    filled_quantity: Option<f64>,
) -> Result<(), Error> {
    let status_str = status.to_string();

    let query = match filled_quantity {
        Some(qty) => {
            client.execute(
                "UPDATE orders
                 SET status = $1::order_status, filled_quantity = $2, updated_at = CURRENT_TIMESTAMP
                 WHERE id = $3",
                &[&status_str, &qty, &order_id],
            )?
        }
        None => {
            client.execute(
                "UPDATE orders
                 SET status = $1::order_status, updated_at = CURRENT_TIMESTAMP
                 WHERE id = $2",
                &[&status_str, &order_id],
            )?
        }
    };

    println!("Order {} updated: status={:?}", order_id, status);
    Ok(())
}
```

### Recording a Trade

```rust
fn record_trade(
    client: &mut Client,
    order_id: i32,
    instrument_id: i32,
    side: TradeSide,
    price: f64,
    quantity: f64,
    commission: f64,
) -> Result<i32, Error> {
    let side_str = side.to_string();

    let row = client.query_one(
        "INSERT INTO trades (order_id, instrument_id, side, price, quantity, commission)
         VALUES ($1, $2, $3::trade_side, $4, $5, $6)
         RETURNING id",
        &[&order_id, &instrument_id, &side_str, &price, &quantity, &commission],
    )?;

    let id: i32 = row.get(0);
    println!("Trade recorded: id={}, {} {} @ {}", id, side_str, quantity, price);
    Ok(id)
}
```

## Transactions: Atomic Operations

```rust
use postgres::{Client, NoTls, Error, Transaction};

fn execute_order_with_trade(
    client: &mut Client,
    instrument_id: i32,
    side: TradeSide,
    price: f64,
    quantity: f64,
    commission_rate: f64,
) -> Result<(i32, i32), Error> {
    // Start transaction
    let mut transaction = client.transaction()?;

    let side_str = side.to_string();

    // Create order
    let order_row = transaction.query_one(
        "INSERT INTO orders (instrument_id, side, price, quantity, filled_quantity, status)
         VALUES ($1, $2::trade_side, $3, $4, $4, 'filled'::order_status)
         RETURNING id",
        &[&instrument_id, &side_str, &price, &quantity],
    )?;
    let order_id: i32 = order_row.get(0);

    // Calculate commission
    let commission = price * quantity * commission_rate;

    // Record trade
    let trade_row = transaction.query_one(
        "INSERT INTO trades (order_id, instrument_id, side, price, quantity, commission)
         VALUES ($1, $2, $3::trade_side, $4, $5, $6)
         RETURNING id",
        &[&order_id, &instrument_id, &side_str, &price, &quantity, &commission],
    )?;
    let trade_id: i32 = trade_row.get(0);

    // Commit transaction
    transaction.commit()?;

    println!("Transaction completed: order_id={}, trade_id={}", order_id, trade_id);
    Ok((order_id, trade_id))
}

fn main() -> Result<(), Error> {
    let mut client = Client::connect(
        "host=localhost user=trader password=secret dbname=trading_db",
        NoTls
    )?;

    // Execute order with trade atomically
    let (order_id, trade_id) = execute_order_with_trade(
        &mut client,
        1,              // instrument_id for BTC
        TradeSide::Buy,
        42500.0,        // price
        0.1,            // quantity
        0.001,          // commission 0.1%
    )?;

    println!("Created order {} and trade {}", order_id, trade_id);
    Ok(())
}
```

## Aggregating Trading Data

```rust
#[derive(Debug)]
struct TradingStats {
    instrument: String,
    total_trades: i64,
    total_volume: f64,
    total_commission: f64,
    avg_price: f64,
    first_trade: Option<String>,
    last_trade: Option<String>,
}

fn get_trading_statistics(
    client: &mut Client,
    days: i32,
) -> Result<Vec<TradingStats>, Error> {
    let rows = client.query(
        "SELECT
            i.symbol,
            COUNT(t.id) as total_trades,
            COALESCE(SUM(t.quantity), 0) as total_volume,
            COALESCE(SUM(t.commission), 0) as total_commission,
            COALESCE(AVG(t.price), 0) as avg_price,
            MIN(t.executed_at)::TEXT as first_trade,
            MAX(t.executed_at)::TEXT as last_trade
         FROM instruments i
         LEFT JOIN trades t ON i.id = t.instrument_id
            AND t.executed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
         GROUP BY i.id, i.symbol
         ORDER BY total_volume DESC",
        &[&days.to_string()],
    )?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(TradingStats {
            instrument: row.get(0),
            total_trades: row.get(1),
            total_volume: row.get::<_, rust_decimal::Decimal>(2).to_string().parse().unwrap_or(0.0),
            total_commission: row.get::<_, rust_decimal::Decimal>(3).to_string().parse().unwrap_or(0.0),
            avg_price: row.get::<_, rust_decimal::Decimal>(4).to_string().parse().unwrap_or(0.0),
            first_trade: row.get(5),
            last_trade: row.get(6),
        });
    }

    Ok(stats)
}

fn get_pnl_by_instrument(client: &mut Client) -> Result<(), Error> {
    let rows = client.query(
        "SELECT
            i.symbol,
            SUM(CASE WHEN t.side = 'buy' THEN -t.price * t.quantity ELSE t.price * t.quantity END) as realized_pnl,
            SUM(t.commission) as total_commission,
            SUM(CASE WHEN t.side = 'buy' THEN -t.price * t.quantity ELSE t.price * t.quantity END) - SUM(t.commission) as net_pnl
         FROM trades t
         JOIN instruments i ON t.instrument_id = i.id
         GROUP BY i.id, i.symbol
         HAVING COUNT(*) > 0
         ORDER BY net_pnl DESC",
        &[],
    )?;

    println!("\n=== PnL by Instrument ===");
    println!("{:<12} {:>15} {:>15} {:>15}", "Instrument", "Realized PnL", "Commission", "Net PnL");
    println!("{}", "-".repeat(60));

    for row in rows {
        let symbol: String = row.get(0);
        let realized_pnl: rust_decimal::Decimal = row.get(1);
        let commission: rust_decimal::Decimal = row.get(2);
        let net_pnl: rust_decimal::Decimal = row.get(3);

        println!("{:<12} {:>15.2} {:>15.2} {:>15.2}",
            symbol, realized_pnl, commission, net_pnl);
    }

    Ok(())
}
```

## Practical Example: Trading Repository

```rust
use postgres::{Client, NoTls, Error};
use std::collections::HashMap;

pub struct TradingRepository {
    client: Client,
}

impl TradingRepository {
    pub fn new(connection_string: &str) -> Result<Self, Error> {
        let client = Client::connect(connection_string, NoTls)?;
        Ok(TradingRepository { client })
    }

    pub fn get_instrument_by_symbol(&mut self, symbol: &str) -> Result<Option<Instrument>, Error> {
        let row = self.client.query_opt(
            "SELECT id, symbol, name, exchange FROM instruments WHERE symbol = $1",
            &[&symbol],
        )?;

        Ok(row.map(|r| Instrument {
            id: r.get(0),
            symbol: r.get(1),
            name: r.get(2),
            exchange: r.get(3),
        }))
    }

    pub fn get_open_orders(&mut self) -> Result<Vec<Order>, Error> {
        let rows = self.client.query(
            "SELECT id, external_id, instrument_id, side::TEXT, price, quantity, filled_quantity, status::TEXT
             FROM orders
             WHERE status = 'pending'
             ORDER BY created_at",
            &[],
        )?;

        let mut orders = Vec::new();
        for row in rows {
            let side_str: String = row.get(3);
            let status_str: String = row.get(7);

            orders.push(Order {
                id: row.get(0),
                external_id: row.get(1),
                instrument_id: row.get(2),
                side: match side_str.as_str() {
                    "buy" => TradeSide::Buy,
                    _ => TradeSide::Sell,
                },
                price: row.get::<_, rust_decimal::Decimal>(4).to_string().parse().unwrap_or(0.0),
                quantity: row.get::<_, rust_decimal::Decimal>(5).to_string().parse().unwrap_or(0.0),
                filled_quantity: row.get::<_, rust_decimal::Decimal>(6).to_string().parse().unwrap_or(0.0),
                status: match status_str.as_str() {
                    "pending" => OrderStatus::Pending,
                    "filled" => OrderStatus::Filled,
                    "cancelled" => OrderStatus::Cancelled,
                    _ => OrderStatus::Rejected,
                },
            });
        }

        Ok(orders)
    }

    pub fn get_position(&mut self, instrument_id: i32) -> Result<f64, Error> {
        let row = self.client.query_one(
            "SELECT
                COALESCE(SUM(CASE WHEN side = 'buy' THEN quantity ELSE -quantity END), 0) as position
             FROM trades
             WHERE instrument_id = $1",
            &[&instrument_id],
        )?;

        let position: rust_decimal::Decimal = row.get(0);
        Ok(position.to_string().parse().unwrap_or(0.0))
    }

    pub fn get_all_positions(&mut self) -> Result<HashMap<String, f64>, Error> {
        let rows = self.client.query(
            "SELECT
                i.symbol,
                COALESCE(SUM(CASE WHEN t.side = 'buy' THEN t.quantity ELSE -t.quantity END), 0) as position
             FROM instruments i
             LEFT JOIN trades t ON i.id = t.instrument_id
             GROUP BY i.id, i.symbol
             HAVING SUM(CASE WHEN t.side = 'buy' THEN t.quantity ELSE -t.quantity END) != 0",
            &[],
        )?;

        let mut positions = HashMap::new();
        for row in rows {
            let symbol: String = row.get(0);
            let position: rust_decimal::Decimal = row.get(1);
            positions.insert(symbol, position.to_string().parse().unwrap_or(0.0));
        }

        Ok(positions)
    }
}

fn main() -> Result<(), Error> {
    let mut repo = TradingRepository::new(
        "host=localhost user=trader password=secret dbname=trading_db"
    )?;

    // Get open orders
    let open_orders = repo.get_open_orders()?;
    println!("Open orders: {}", open_orders.len());

    // Get all positions
    let positions = repo.get_all_positions()?;
    println!("\nPositions:");
    for (symbol, qty) in &positions {
        println!("  {}: {}", symbol, qty);
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| PostgreSQL | Powerful production-ready DBMS |
| `postgres` crate | Synchronous PostgreSQL client |
| ENUM types | Type-safe enumerations in database |
| Transactions | Atomic data operations |
| Prepared statements | SQL injection protection |
| Indexes | Faster field lookups |
| Aggregation | Statistics and data analytics |

## Homework

1. **Balance table**: Create a `balances` table to store balance for each asset. Add a trigger that automatically updates the balance when a new trade is recorded.

2. **Order change history**: Create an `order_history` table for auditing all order changes (status, filled_quantity). Use a trigger for automatic recording.

3. **PnL calculation function**: Write a SQL function `calculate_pnl(instrument_id, start_date, end_date)` that returns realized PnL for the period.

4. **Trade search**: Implement a trade search function with filtering:
   - By instrument
   - By date range
   - By direction (buy/sell)
   - By minimum volume

   With pagination (LIMIT/OFFSET).

## Navigation

[← Previous day](../224-migrations-schema-evolution/en.md) | [Next day →](../226-tokio-postgres-async-client/en.md)
