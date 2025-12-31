# Day 228: sqlx Migrations

## Trading Analogy

Imagine you're developing a trading system. Initially, you only need to store asset prices. Then you need to track trading volumes. Later — order history, then user portfolios, risk management, and trading strategies. Each of these changes requires modifying the database structure.

**Database migrations** are like the evolution of a trading system: a controlled, versioned way to change your data schema. Just as a trader adapts their strategies to changing market conditions, migrations allow your database to "adapt" to new business requirements.

In real trading, this is critical:
- Adding new order types without losing historical data
- Extending price analytics while maintaining compatibility
- Evolving the risk management system

## What are Migrations?

A migration is a file containing SQL commands that describe a database schema change. Each migration has:

1. **Version** — a unique identifier (usually a timestamp)
2. **Up script** — commands to apply changes
3. **Down script** — commands to revert changes

```
migrations/
├── 20240101000000_create_assets.sql
├── 20240102000000_create_trades.sql
├── 20240103000000_add_volume_column.sql
└── 20240104000000_create_portfolio.sql
```

## Installing sqlx-cli

To work with migrations, you need the command-line tool:

```bash
cargo install sqlx-cli --features postgres
```

For multiple database support:

```bash
cargo install sqlx-cli --features postgres,mysql,sqlite
```

## Project Setup

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros"] }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
chrono = { version = "0.4", features = ["serde"] }
```

Create a `.env` file:

```bash
DATABASE_URL=postgres://trader:password@localhost:5432/trading_db
```

## Creating Your First Migration

```bash
# Initialize migrations directory
sqlx migrate add create_assets
```

This creates a file `migrations/<timestamp>_create_assets.sql`:

```sql
-- migrations/20240101120000_create_assets.sql

-- Assets table for trading
CREATE TABLE assets (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    asset_type VARCHAR(20) NOT NULL CHECK (asset_type IN ('crypto', 'stock', 'forex', 'commodity')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for fast symbol lookup
CREATE INDEX idx_assets_symbol ON assets(symbol);

-- Insert base assets
INSERT INTO assets (symbol, name, asset_type) VALUES
    ('BTC', 'Bitcoin', 'crypto'),
    ('ETH', 'Ethereum', 'crypto'),
    ('AAPL', 'Apple Inc.', 'stock'),
    ('EUR/USD', 'Euro/US Dollar', 'forex');
```

## Creating Reversible Migrations

For rollback capability, use reversible migrations:

```bash
sqlx migrate add -r create_trades
```

This creates two files:

```sql
-- migrations/20240102120000_create_trades.up.sql

CREATE TABLE trades (
    id BIGSERIAL PRIMARY KEY,
    asset_id INTEGER REFERENCES assets(id),
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    total_value DECIMAL(20, 8) GENERATED ALWAYS AS (price * quantity) STORED,
    fee DECIMAL(20, 8) DEFAULT 0,
    executed_at TIMESTAMPTZ DEFAULT NOW(),
    strategy VARCHAR(50),
    notes TEXT
);

-- Indexes for trade analysis
CREATE INDEX idx_trades_asset ON trades(asset_id);
CREATE INDEX idx_trades_executed ON trades(executed_at);
CREATE INDEX idx_trades_strategy ON trades(strategy);
```

```sql
-- migrations/20240102120000_create_trades.down.sql

DROP INDEX IF EXISTS idx_trades_strategy;
DROP INDEX IF EXISTS idx_trades_executed;
DROP INDEX IF EXISTS idx_trades_asset;
DROP TABLE IF EXISTS trades;
```

## Running Migrations

```bash
# Apply all pending migrations
sqlx migrate run

# Check migration status
sqlx migrate info
```

## Portfolio and Risk Management Migration

```sql
-- migrations/20240103120000_create_portfolio.up.sql

-- Trader portfolio
CREATE TABLE portfolio (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    asset_id INTEGER REFERENCES assets(id),
    quantity DECIMAL(20, 8) NOT NULL DEFAULT 0,
    avg_buy_price DECIMAL(20, 8),
    total_invested DECIMAL(20, 8) DEFAULT 0,
    unrealized_pnl DECIMAL(20, 8) DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, asset_id)
);

-- Risk management limits
CREATE TABLE risk_limits (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    max_position_size DECIMAL(20, 8) NOT NULL,
    max_daily_loss DECIMAL(20, 8) NOT NULL,
    max_drawdown_percent DECIMAL(5, 2) NOT NULL,
    stop_loss_percent DECIMAL(5, 2),
    take_profit_percent DECIMAL(5, 2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Risk events log
CREATE TABLE risk_events (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) CHECK (severity IN ('info', 'warning', 'critical')),
    message TEXT NOT NULL,
    metadata JSONB,
    occurred_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_risk_events_user ON risk_events(user_id);
CREATE INDEX idx_risk_events_occurred ON risk_events(occurred_at);
```

```sql
-- migrations/20240103120000_create_portfolio.down.sql

DROP TABLE IF EXISTS risk_events;
DROP TABLE IF EXISTS risk_limits;
DROP TABLE IF EXISTS portfolio;
```

## Programmatic Migration Application

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Apply migrations from directory
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;

    println!("Migrations applied successfully!");

    Ok(())
}
```

## Embedded Migrations

For production builds, you can embed migrations into the binary:

```rust
use sqlx::postgres::PgPoolOptions;

// Embed migrations at compile time
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Apply embedded migrations
    MIGRATOR.run(&pool).await?;

    println!("Embedded migrations applied!");

    Ok(())
}
```

## Data Migration: Price History

```sql
-- migrations/20240104120000_create_price_history.up.sql

-- Store price history for analysis
CREATE TABLE price_history (
    id BIGSERIAL PRIMARY KEY,
    asset_id INTEGER REFERENCES assets(id),
    open_price DECIMAL(20, 8) NOT NULL,
    high_price DECIMAL(20, 8) NOT NULL,
    low_price DECIMAL(20, 8) NOT NULL,
    close_price DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    timeframe VARCHAR(10) NOT NULL CHECK (timeframe IN ('1m', '5m', '15m', '1h', '4h', '1d', '1w')),
    candle_time TIMESTAMPTZ NOT NULL,
    UNIQUE(asset_id, timeframe, candle_time)
);

-- Partitioning for large data volumes
CREATE INDEX idx_price_history_asset_time ON price_history(asset_id, candle_time);
CREATE INDEX idx_price_history_timeframe ON price_history(timeframe, candle_time);

-- Materialized view for daily statistics
CREATE MATERIALIZED VIEW daily_stats AS
SELECT
    asset_id,
    DATE(candle_time) as trade_date,
    MIN(low_price) as day_low,
    MAX(high_price) as day_high,
    SUM(volume) as total_volume,
    (ARRAY_AGG(open_price ORDER BY candle_time))[1] as day_open,
    (ARRAY_AGG(close_price ORDER BY candle_time DESC))[1] as day_close
FROM price_history
WHERE timeframe = '1h'
GROUP BY asset_id, DATE(candle_time);

CREATE UNIQUE INDEX idx_daily_stats ON daily_stats(asset_id, trade_date);
```

## Adding Columns to Existing Tables

```sql
-- migrations/20240105120000_add_strategy_params.up.sql

-- Add trading strategy parameters
ALTER TABLE trades ADD COLUMN IF NOT EXISTS strategy_params JSONB;
ALTER TABLE trades ADD COLUMN IF NOT EXISTS signal_strength DECIMAL(5, 2);
ALTER TABLE trades ADD COLUMN IF NOT EXISTS market_condition VARCHAR(20);

-- Documentation comments
COMMENT ON COLUMN trades.strategy_params IS 'JSON strategy parameters: stop_loss, take_profit, indicators';
COMMENT ON COLUMN trades.signal_strength IS 'Signal strength from 0.00 to 1.00';
COMMENT ON COLUMN trades.market_condition IS 'Market condition: trending, ranging, volatile';
```

```sql
-- migrations/20240105120000_add_strategy_params.down.sql

ALTER TABLE trades DROP COLUMN IF EXISTS market_condition;
ALTER TABLE trades DROP COLUMN IF EXISTS signal_strength;
ALTER TABLE trades DROP COLUMN IF EXISTS strategy_params;
```

## Practical Example: Trading System with Migrations

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow)]
struct Asset {
    id: i32,
    symbol: String,
    name: String,
    asset_type: String,
}

#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    asset_id: i32,
    side: String,
    price: rust_decimal::Decimal,
    quantity: rust_decimal::Decimal,
    total_value: rust_decimal::Decimal,
    executed_at: DateTime<Utc>,
    strategy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StrategyParams {
    stop_loss: f64,
    take_profit: f64,
    indicators: Vec<String>,
}

struct TradingSystem {
    pool: PgPool,
}

impl TradingSystem {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(database_url).await?;

        // Apply migrations on startup
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    async fn get_assets(&self) -> Result<Vec<Asset>, sqlx::Error> {
        sqlx::query_as!(
            Asset,
            "SELECT id, symbol, name, asset_type FROM assets ORDER BY symbol"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn execute_trade(
        &self,
        asset_id: i32,
        side: &str,
        price: f64,
        quantity: f64,
        strategy: Option<&str>,
        params: Option<StrategyParams>,
    ) -> Result<i64, sqlx::Error> {
        let params_json = params.map(|p| serde_json::to_value(p).unwrap());

        let result = sqlx::query!(
            r#"
            INSERT INTO trades (asset_id, side, price, quantity, strategy, strategy_params)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            asset_id,
            side,
            rust_decimal::Decimal::from_f64_retain(price),
            rust_decimal::Decimal::from_f64_retain(quantity),
            strategy,
            params_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    async fn get_portfolio_value(&self, user_id: i32) -> Result<f64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(p.quantity * ph.close_price), 0) as "total_value!"
            FROM portfolio p
            JOIN LATERAL (
                SELECT close_price
                FROM price_history
                WHERE asset_id = p.asset_id
                ORDER BY candle_time DESC
                LIMIT 1
            ) ph ON true
            WHERE p.user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.total_value.to_string().parse().unwrap_or(0.0))
    }

    async fn check_risk_limits(&self, user_id: i32) -> Result<bool, sqlx::Error> {
        let limits = sqlx::query!(
            "SELECT max_daily_loss, max_drawdown_percent FROM risk_limits WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(limits) = limits {
            // Check daily losses
            let daily_pnl = sqlx::query!(
                r#"
                SELECT COALESCE(SUM(
                    CASE WHEN side = 'sell' THEN total_value ELSE -total_value END
                ), 0) as "pnl!"
                FROM trades
                WHERE executed_at > NOW() - INTERVAL '1 day'
                "#
            )
            .fetch_one(&self.pool)
            .await?;

            let pnl: f64 = daily_pnl.pnl.to_string().parse().unwrap_or(0.0);
            let max_loss: f64 = limits.max_daily_loss.to_string().parse().unwrap_or(0.0);

            if pnl < -max_loss {
                // Log risk event
                sqlx::query!(
                    r#"
                    INSERT INTO risk_events (user_id, event_type, severity, message)
                    VALUES ($1, 'daily_loss_exceeded', 'critical', $2)
                    "#,
                    user_id,
                    format!("Daily loss {} exceeds limit {}", pnl, max_loss)
                )
                .execute(&self.pool)
                .await?;

                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let trading = TradingSystem::new(&database_url).await?;

    // Get list of assets
    let assets = trading.get_assets().await?;
    println!("Available assets:");
    for asset in &assets {
        println!("  {} - {} ({})", asset.symbol, asset.name, asset.asset_type);
    }

    // Execute trade with strategy parameters
    let params = StrategyParams {
        stop_loss: 0.02,
        take_profit: 0.05,
        indicators: vec!["RSI".to_string(), "MACD".to_string()],
    };

    let trade_id = trading.execute_trade(
        1,
        "buy",
        42000.0,
        0.5,
        Some("momentum"),
        Some(params),
    ).await?;

    println!("Executed trade #{}", trade_id);

    // Check risk limits
    let can_trade = trading.check_risk_limits(1).await?;
    println!("Can continue trading: {}", can_trade);

    Ok(())
}
```

## Reverting Migrations

```bash
# Revert the last migration
sqlx migrate revert

# Revert all migrations
sqlx migrate revert --target-version 0
```

## Best Practices

| Practice | Description |
|----------|-------------|
| Atomicity | One migration = one logical change |
| Reversibility | Always write down scripts |
| Idempotency | Use IF NOT EXISTS / IF EXISTS |
| Testing | Test migrations on data copies |
| Versioning | Store migrations in git |
| Documentation | Comment complex changes |

## What We Learned

| Concept | Description |
|---------|-------------|
| Migrations | Versioned database schema changes |
| sqlx-cli | Command-line tool for migrations |
| Up/Down | Apply and revert scripts |
| Embedded | Embedding migrations in binary |
| sqlx::migrate! | Macro for compile-time verification |

## Homework

1. **Order System**: Create migrations for a complete order system with types (market, limit, stop-loss, take-profit), statuses, and change history.

2. **Analytics**: Add a migration with materialized views for:
   - Daily statistics per asset
   - Top 10 profitable trades per month
   - Average returns by strategy

3. **Audit System**: Create an audit system using triggers:
   - audit_log table to record all changes
   - Triggers on INSERT/UPDATE/DELETE for trades and portfolio

4. **Data Migration**: Write a migration that:
   - Creates a new table structure
   - Migrates data from the old structure
   - Drops the old table
   - Renames the new table

## Navigation

[← Previous day](../227-sqlx-compile-time-queries/en.md) | [Next day →](../229-connection-pool/en.md)
