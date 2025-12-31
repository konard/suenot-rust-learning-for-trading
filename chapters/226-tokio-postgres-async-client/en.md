# Day 226: tokio-postgres: Async Client

## Trading Analogy

Imagine a modern high-frequency trading system that must simultaneously:
- Record every trade in the database
- Query historical data for analysis
- Update current portfolio positions
- Track order statuses

If you use a synchronous database client, each operation blocks the thread — like a trader who can only do one thing at a time: either look at charts, or place an order, or check balance. With **tokio-postgres** (an async PostgreSQL client), your system can execute multiple database operations in parallel without blocking the main thread — like a team of traders where everyone is busy with their task, but all working simultaneously.

## What is tokio-postgres?

`tokio-postgres` is an asynchronous PostgreSQL client for Rust, built on the Tokio runtime. It allows you to:

- Execute queries without blocking the thread
- Handle multiple connections in parallel
- Work efficiently with connection pools
- Use prepared statements for optimization

### Adding the Dependency

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
```

## Basic Connection

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // PostgreSQL connection string
    let connection_string = "host=localhost user=trader password=secret dbname=trading";

    // Establish connection
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

    // The connection must be handled in a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Now we can use the client for queries
    let rows = client
        .query("SELECT symbol, price FROM prices LIMIT 5", &[])
        .await?;

    for row in rows {
        let symbol: &str = row.get(0);
        let price: f64 = row.get(1);
        println!("{}: ${:.2}", symbol, price);
    }

    Ok(())
}
```

## Creating Tables for a Trading System

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_trading_database(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Table for storing price data
    client.execute(
        "CREATE TABLE IF NOT EXISTS price_history (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            volume DECIMAL(18, 8) NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Orders table
    client.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            side VARCHAR(4) NOT NULL,
            order_type VARCHAR(10) NOT NULL,
            price DECIMAL(18, 8),
            quantity DECIMAL(18, 8) NOT NULL,
            status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Portfolio table
    client.execute(
        "CREATE TABLE IF NOT EXISTS portfolio (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL UNIQUE,
            quantity DECIMAL(18, 8) NOT NULL,
            avg_price DECIMAL(18, 8) NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Trades table
    client.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id SERIAL PRIMARY KEY,
            order_id INTEGER REFERENCES orders(id),
            symbol VARCHAR(20) NOT NULL,
            side VARCHAR(4) NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            quantity DECIMAL(18, 8) NOT NULL,
            fee DECIMAL(18, 8) DEFAULT 0,
            executed_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    println!("Trading system tables created successfully!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    setup_trading_database(&client).await?;

    Ok(())
}
```

## Price Analysis: Writing and Reading Data

```rust
use tokio_postgres::{NoTls, Error, Row};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: Decimal,
    volume: Decimal,
}

impl PriceData {
    fn from_row(row: &Row) -> Self {
        PriceData {
            symbol: row.get("symbol"),
            price: row.get("price"),
            volume: row.get("volume"),
        }
    }
}

struct PriceAnalyzer {
    client: tokio_postgres::Client,
}

impl PriceAnalyzer {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(PriceAnalyzer { client })
    }

    // Record a new price
    async fn record_price(&self, symbol: &str, price: Decimal, volume: Decimal) -> Result<(), Error> {
        self.client.execute(
            "INSERT INTO price_history (symbol, price, volume) VALUES ($1, $2, $3)",
            &[&symbol, &price, &volume],
        ).await?;

        println!("Recorded price {} for {}", price, symbol);
        Ok(())
    }

    // Get the latest price
    async fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceData>, Error> {
        let row = self.client.query_opt(
            "SELECT symbol, price, volume FROM price_history
             WHERE symbol = $1
             ORDER BY timestamp DESC
             LIMIT 1",
            &[&symbol],
        ).await?;

        Ok(row.map(|r| PriceData::from_row(&r)))
    }

    // Get average price over a period
    async fn get_average_price(&self, symbol: &str, hours: i32) -> Result<Option<Decimal>, Error> {
        let row = self.client.query_opt(
            "SELECT AVG(price) as avg_price FROM price_history
             WHERE symbol = $1
             AND timestamp > NOW() - INTERVAL '1 hour' * $2",
            &[&symbol, &hours],
        ).await?;

        Ok(row.and_then(|r| r.get("avg_price")))
    }

    // Get price range (min and max)
    async fn get_price_range(&self, symbol: &str, hours: i32) -> Result<(Option<Decimal>, Option<Decimal>), Error> {
        let row = self.client.query_one(
            "SELECT MIN(price) as min_price, MAX(price) as max_price
             FROM price_history
             WHERE symbol = $1
             AND timestamp > NOW() - INTERVAL '1 hour' * $2",
            &[&symbol, &hours],
        ).await?;

        let min_price: Option<Decimal> = row.get("min_price");
        let max_price: Option<Decimal> = row.get("max_price");

        Ok((min_price, max_price))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let analyzer = PriceAnalyzer::new("host=localhost user=trader dbname=trading").await?;

    // Record some prices
    let btc_price = Decimal::from_str("42500.50").unwrap();
    let eth_price = Decimal::from_str("2250.75").unwrap();

    analyzer.record_price("BTC", btc_price, Decimal::from_str("1.5").unwrap()).await?;
    analyzer.record_price("ETH", eth_price, Decimal::from_str("10.0").unwrap()).await?;

    // Get the latest price
    if let Some(price) = analyzer.get_latest_price("BTC").await? {
        println!("Latest BTC price: {}", price.price);
    }

    // Get average price over 24 hours
    if let Some(avg) = analyzer.get_average_price("BTC", 24).await? {
        println!("BTC 24h average price: {}", avg);
    }

    Ok(())
}
```

## Order Management

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
struct Order {
    id: Option<i32>,
    symbol: String,
    side: String,
    order_type: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
}

struct OrderManager {
    client: tokio_postgres::Client,
}

impl OrderManager {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(OrderManager { client })
    }

    // Create a new order
    async fn create_order(&self, order: &Order) -> Result<i32, Error> {
        let row = self.client.query_one(
            "INSERT INTO orders (symbol, side, order_type, price, quantity, status)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
            &[&order.symbol, &order.side, &order.order_type,
              &order.price, &order.quantity, &order.status],
        ).await?;

        let id: i32 = row.get(0);
        println!("Created order #{}: {} {} {} @ {:?}",
                 id, order.side, order.quantity, order.symbol, order.price);
        Ok(id)
    }

    // Update order status
    async fn update_order_status(&self, order_id: i32, status: &str) -> Result<bool, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
            &[&status, &order_id],
        ).await?;

        if rows_affected > 0 {
            println!("Order #{} updated: status = {}", order_id, status);
        }

        Ok(rows_affected > 0)
    }

    // Get active orders
    async fn get_active_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>, Error> {
        let rows = match symbol {
            Some(s) => {
                self.client.query(
                    "SELECT id, symbol, side, order_type, price, quantity, status
                     FROM orders
                     WHERE status IN ('pending', 'partial')
                     AND symbol = $1
                     ORDER BY created_at DESC",
                    &[&s],
                ).await?
            }
            None => {
                self.client.query(
                    "SELECT id, symbol, side, order_type, price, quantity, status
                     FROM orders
                     WHERE status IN ('pending', 'partial')
                     ORDER BY created_at DESC",
                    &[],
                ).await?
            }
        };

        let orders: Vec<Order> = rows.iter().map(|row| {
            Order {
                id: Some(row.get("id")),
                symbol: row.get("symbol"),
                side: row.get("side"),
                order_type: row.get("order_type"),
                price: row.get("price"),
                quantity: row.get("quantity"),
                status: row.get("status"),
            }
        }).collect();

        Ok(orders)
    }

    // Cancel an order
    async fn cancel_order(&self, order_id: i32) -> Result<bool, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = 'cancelled', updated_at = NOW()
             WHERE id = $1 AND status IN ('pending', 'partial')",
            &[&order_id],
        ).await?;

        if rows_affected > 0 {
            println!("Order #{} cancelled", order_id);
        }

        Ok(rows_affected > 0)
    }

    // Cancel all orders for a symbol
    async fn cancel_all_orders(&self, symbol: &str) -> Result<u64, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = 'cancelled', updated_at = NOW()
             WHERE symbol = $1 AND status IN ('pending', 'partial')",
            &[&symbol],
        ).await?;

        println!("Cancelled {} orders for {}", rows_affected, symbol);
        Ok(rows_affected)
    }
}
```

## Portfolio Tracking

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
}

impl Position {
    fn market_value(&self, current_price: Decimal) -> Decimal {
        self.quantity * current_price
    }

    fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        self.quantity * (current_price - self.avg_price)
    }
}

struct PortfolioTracker {
    client: tokio_postgres::Client,
}

impl PortfolioTracker {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(PortfolioTracker { client })
    }

    // Update position after a trade
    async fn update_position(&self, symbol: &str, quantity: Decimal, price: Decimal) -> Result<(), Error> {
        // Use UPSERT (INSERT ... ON CONFLICT)
        self.client.execute(
            "INSERT INTO portfolio (symbol, quantity, avg_price)
             VALUES ($1, $2, $3)
             ON CONFLICT (symbol) DO UPDATE SET
                quantity = portfolio.quantity + EXCLUDED.quantity,
                avg_price = CASE
                    WHEN EXCLUDED.quantity > 0 THEN
                        (portfolio.avg_price * portfolio.quantity + EXCLUDED.avg_price * EXCLUDED.quantity)
                        / (portfolio.quantity + EXCLUDED.quantity)
                    ELSE portfolio.avg_price
                END,
                updated_at = NOW()",
            &[&symbol, &quantity, &price],
        ).await?;

        println!("Position {} updated: +{} @ {}", symbol, quantity, price);
        Ok(())
    }

    // Get a position
    async fn get_position(&self, symbol: &str) -> Result<Option<Position>, Error> {
        let row = self.client.query_opt(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE symbol = $1",
            &[&symbol],
        ).await?;

        Ok(row.map(|r| Position {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }))
    }

    // Get all positions
    async fn get_all_positions(&self) -> Result<Vec<Position>, Error> {
        let rows = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        let positions: Vec<Position> = rows.iter().map(|r| Position {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }).collect();

        Ok(positions)
    }

    // Calculate total portfolio value
    async fn calculate_portfolio_value(
        &self,
        prices: &HashMap<String, Decimal>
    ) -> Result<Decimal, Error> {
        let positions = self.get_all_positions().await?;

        let total: Decimal = positions.iter()
            .filter_map(|pos| prices.get(&pos.symbol).map(|&p| pos.market_value(p)))
            .sum();

        Ok(total)
    }

    // Calculate unrealized PnL
    async fn calculate_unrealized_pnl(
        &self,
        prices: &HashMap<String, Decimal>
    ) -> Result<Decimal, Error> {
        let positions = self.get_all_positions().await?;

        let total_pnl: Decimal = positions.iter()
            .filter_map(|pos| prices.get(&pos.symbol).map(|&p| pos.unrealized_pnl(p)))
            .sum();

        Ok(total_pnl)
    }
}
```

## Risk Management

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::str::FromStr;

struct RiskManager {
    client: tokio_postgres::Client,
    max_position_size: Decimal,
    max_daily_loss: Decimal,
}

impl RiskManager {
    async fn new(
        connection_string: &str,
        max_position_size: Decimal,
        max_daily_loss: Decimal,
    ) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(RiskManager {
            client,
            max_position_size,
            max_daily_loss,
        })
    }

    // Check position limit
    async fn check_position_limit(&self, symbol: &str, additional_quantity: Decimal) -> Result<bool, Error> {
        let row = self.client.query_opt(
            "SELECT quantity FROM portfolio WHERE symbol = $1",
            &[&symbol],
        ).await?;

        let current_quantity: Decimal = row
            .map(|r| r.get("quantity"))
            .unwrap_or(Decimal::ZERO);

        let new_quantity = current_quantity + additional_quantity;

        if new_quantity.abs() > self.max_position_size {
            println!("RISK: Position limit exceeded for {}! Current: {}, Requested: +{}",
                     symbol, current_quantity, additional_quantity);
            return Ok(false);
        }

        Ok(true)
    }

    // Get daily PnL
    async fn get_daily_pnl(&self) -> Result<Decimal, Error> {
        let row = self.client.query_one(
            "SELECT COALESCE(SUM(
                CASE WHEN side = 'sell' THEN price * quantity - fee
                     ELSE -(price * quantity + fee)
                END
            ), 0) as daily_pnl
             FROM trades
             WHERE executed_at >= CURRENT_DATE",
            &[],
        ).await?;

        let daily_pnl: Decimal = row.get("daily_pnl");
        Ok(daily_pnl)
    }

    // Check daily loss limit
    async fn check_daily_loss_limit(&self) -> Result<bool, Error> {
        let daily_pnl = self.get_daily_pnl().await?;

        if daily_pnl < -self.max_daily_loss {
            println!("RISK: Daily loss limit exceeded! PnL: {}", daily_pnl);
            return Ok(false);
        }

        Ok(true)
    }

    // Check if an order can be placed
    async fn can_place_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: Decimal,
    ) -> Result<bool, Error> {
        // Check daily limit
        if !self.check_daily_loss_limit().await? {
            return Ok(false);
        }

        // For buys, check position increase
        let quantity_change = if side == "buy" { quantity } else { -quantity };

        if !self.check_position_limit(symbol, quantity_change).await? {
            return Ok(false);
        }

        println!("Risk check passed for {} {} {}", side, quantity, symbol);
        Ok(true)
    }

    // Get risk report
    async fn get_risk_report(&self) -> Result<String, Error> {
        let daily_pnl = self.get_daily_pnl().await?;

        let positions = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        let mut report = String::from("=== RISK REPORT ===\n");
        report.push_str(&format!("Daily PnL: {}\n", daily_pnl));
        report.push_str(&format!("Daily loss limit: {}\n", self.max_daily_loss));
        report.push_str(&format!("Utilized: {:.2}%\n\n",
            (daily_pnl.abs() / self.max_daily_loss * Decimal::from(100))));

        report.push_str("Positions:\n");
        for row in positions {
            let symbol: String = row.get("symbol");
            let quantity: Decimal = row.get("quantity");
            let utilization = (quantity.abs() / self.max_position_size * Decimal::from(100));
            report.push_str(&format!("  {}: {} ({:.2}% of limit)\n", symbol, quantity, utilization));
        }

        Ok(report)
    }
}
```

## Parallel Queries

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::try_join;

struct TradingDashboard {
    client: tokio_postgres::Client,
}

impl TradingDashboard {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(TradingDashboard { client })
    }

    // Get full dashboard — all queries execute in parallel!
    async fn get_dashboard_data(&self) -> Result<DashboardData, Error> {
        // Run all queries in parallel
        let (positions, active_orders, recent_trades, daily_pnl) = try_join!(
            self.get_positions(),
            self.get_active_orders(),
            self.get_recent_trades(10),
            self.get_daily_pnl()
        )?;

        Ok(DashboardData {
            positions,
            active_orders,
            recent_trades,
            daily_pnl,
        })
    }

    async fn get_positions(&self) -> Result<Vec<PositionInfo>, Error> {
        let rows = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        Ok(rows.iter().map(|r| PositionInfo {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }).collect())
    }

    async fn get_active_orders(&self) -> Result<Vec<OrderInfo>, Error> {
        let rows = self.client.query(
            "SELECT id, symbol, side, price, quantity, status
             FROM orders
             WHERE status IN ('pending', 'partial')
             ORDER BY created_at DESC",
            &[],
        ).await?;

        Ok(rows.iter().map(|r| OrderInfo {
            id: r.get("id"),
            symbol: r.get("symbol"),
            side: r.get("side"),
            price: r.get("price"),
            quantity: r.get("quantity"),
            status: r.get("status"),
        }).collect())
    }

    async fn get_recent_trades(&self, limit: i64) -> Result<Vec<TradeInfo>, Error> {
        let rows = self.client.query(
            "SELECT id, symbol, side, price, quantity, executed_at
             FROM trades
             ORDER BY executed_at DESC
             LIMIT $1",
            &[&limit],
        ).await?;

        Ok(rows.iter().map(|r| TradeInfo {
            id: r.get("id"),
            symbol: r.get("symbol"),
            side: r.get("side"),
            price: r.get("price"),
            quantity: r.get("quantity"),
        }).collect())
    }

    async fn get_daily_pnl(&self) -> Result<Decimal, Error> {
        let row = self.client.query_one(
            "SELECT COALESCE(SUM(
                CASE WHEN side = 'sell' THEN price * quantity - fee
                     ELSE -(price * quantity + fee)
                END
            ), 0) as pnl FROM trades WHERE executed_at >= CURRENT_DATE",
            &[],
        ).await?;

        Ok(row.get("pnl"))
    }
}

#[derive(Debug)]
struct DashboardData {
    positions: Vec<PositionInfo>,
    active_orders: Vec<OrderInfo>,
    recent_trades: Vec<TradeInfo>,
    daily_pnl: Decimal,
}

#[derive(Debug)]
struct PositionInfo {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
}

#[derive(Debug)]
struct OrderInfo {
    id: i32,
    symbol: String,
    side: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
}

#[derive(Debug)]
struct TradeInfo {
    id: i32,
    symbol: String,
    side: String,
    price: Decimal,
    quantity: Decimal,
}
```

## Transactions

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

struct TradingEngine {
    client: tokio_postgres::Client,
}

impl TradingEngine {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(TradingEngine { client })
    }

    // Execute order atomically using a transaction
    async fn execute_order(
        &self,
        order_id: i32,
        executed_price: Decimal,
        executed_quantity: Decimal,
    ) -> Result<i32, Error> {
        // Start transaction
        let transaction = self.client.transaction().await?;

        // 1. Get order information
        let order_row = transaction.query_one(
            "SELECT symbol, side, quantity, status FROM orders WHERE id = $1 FOR UPDATE",
            &[&order_id],
        ).await?;

        let symbol: String = order_row.get("symbol");
        let side: String = order_row.get("side");
        let status: String = order_row.get("status");

        if status != "pending" && status != "partial" {
            // Rollback if order is not active
            transaction.rollback().await?;
            return Err(Error::__private_api_protocol_error("Order is not active"));
        }

        // 2. Create trade record
        let trade_row = transaction.query_one(
            "INSERT INTO trades (order_id, symbol, side, price, quantity)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&order_id, &symbol, &side, &executed_price, &executed_quantity],
        ).await?;

        let trade_id: i32 = trade_row.get(0);

        // 3. Update order status
        transaction.execute(
            "UPDATE orders SET status = 'filled', updated_at = NOW() WHERE id = $1",
            &[&order_id],
        ).await?;

        // 4. Update portfolio
        let quantity_change = if side == "buy" {
            executed_quantity
        } else {
            -executed_quantity
        };

        transaction.execute(
            "INSERT INTO portfolio (symbol, quantity, avg_price)
             VALUES ($1, $2, $3)
             ON CONFLICT (symbol) DO UPDATE SET
                quantity = portfolio.quantity + EXCLUDED.quantity,
                avg_price = CASE
                    WHEN EXCLUDED.quantity > 0 THEN
                        (portfolio.avg_price * portfolio.quantity + EXCLUDED.avg_price * EXCLUDED.quantity)
                        / NULLIF(portfolio.quantity + EXCLUDED.quantity, 0)
                    ELSE portfolio.avg_price
                END,
                updated_at = NOW()",
            &[&symbol, &quantity_change, &executed_price],
        ).await?;

        // 5. Commit transaction
        transaction.commit().await?;

        println!("Order #{} executed: {} {} {} @ {}",
                 order_id, side, executed_quantity, symbol, executed_price);

        Ok(trade_id)
    }
}
```

## Prepared Statements for Optimization

```rust
use tokio_postgres::{NoTls, Error, Statement};
use rust_decimal::Decimal;

struct OptimizedPriceRecorder {
    client: tokio_postgres::Client,
    insert_statement: Statement,
    select_latest_statement: Statement,
}

impl OptimizedPriceRecorder {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // Prepare statements in advance
        let insert_statement = client.prepare(
            "INSERT INTO price_history (symbol, price, volume) VALUES ($1, $2, $3)"
        ).await?;

        let select_latest_statement = client.prepare(
            "SELECT price FROM price_history WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
        ).await?;

        Ok(OptimizedPriceRecorder {
            client,
            insert_statement,
            select_latest_statement,
        })
    }

    // Fast price recording using prepared statement
    async fn record_price(&self, symbol: &str, price: Decimal, volume: Decimal) -> Result<(), Error> {
        self.client.execute(&self.insert_statement, &[&symbol, &price, &volume]).await?;
        Ok(())
    }

    // Fast retrieval of latest price
    async fn get_latest_price(&self, symbol: &str) -> Result<Option<Decimal>, Error> {
        let row = self.client.query_opt(&self.select_latest_statement, &[&symbol]).await?;
        Ok(row.map(|r| r.get(0)))
    }

    // Batch price recording
    async fn record_prices_batch(&self, prices: &[(String, Decimal, Decimal)]) -> Result<(), Error> {
        for (symbol, price, volume) in prices {
            self.record_price(symbol, *price, *volume).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let recorder = OptimizedPriceRecorder::new(
        "host=localhost user=trader dbname=trading"
    ).await?;

    // Record many prices — prepared statements are faster
    for i in 0..1000 {
        let price = Decimal::from(42000 + i);
        recorder.record_price("BTC", price, Decimal::from(1)).await?;
    }

    println!("Recorded 1000 price points");

    if let Some(price) = recorder.get_latest_price("BTC").await? {
        println!("Latest BTC price: {}", price);
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio_postgres::connect` | Async connection to PostgreSQL |
| `client.query()` | Execute query returning rows |
| `client.execute()` | Execute query without returning data |
| `client.query_one()` | Query exactly one row |
| `client.query_opt()` | Query optionally one row |
| `client.transaction()` | Start a transaction |
| `client.prepare()` | Create a prepared statement |
| `try_join!` | Execute multiple async operations in parallel |

## Homework

1. **Price Monitoring System**: Create a service that:
   - Records asset prices every second
   - Calculates moving average over the last N records
   - Sends notification if price deviates more than X%

2. **Order Book in Database**: Implement a full order book:
   - Adding limit orders
   - Order matching
   - Partial fills
   - Getting best bid/ask

3. **Trade Audit System**: Create an audit system:
   - Logging all portfolio operations
   - Position change history
   - Report generation by time periods

4. **Connection Pool**: Using `deadpool-postgres`, create:
   - Pool of multiple connections
   - Parallel processing of many requests
   - Graceful shutdown with closing all connections

## Navigation

[← Previous day](../225-diesel-orm-rust/en.md) | [Next day →](../227-sqlx-compile-time-sql/en.md)
