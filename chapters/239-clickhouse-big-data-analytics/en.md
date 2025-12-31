# Day 239: ClickHouse: Big Data Analytics

## Trading Analogy

Imagine you're running a quantitative trading desk that processes millions of market events per day: trades, quotes, order book updates, and market indicators. At the end of each day, your analysts need to answer questions like:

- "What was the average spread for BTC/USDT between 2 PM and 4 PM across all exchanges?"
- "Which trading pairs had the highest volume spike in the last 30 days?"
- "Calculate the VWAP (Volume Weighted Average Price) for every minute of the last year"

A traditional row-based database would crawl through these queries, reading entire rows when you only need a few columns. **ClickHouse** is like having a specialized analytics department that organizes all your trading data by columns — when you ask about prices, it only reads prices, not timestamps, volumes, or other fields. This columnar approach, combined with extreme compression and parallel processing, makes ClickHouse capable of scanning billions of rows in seconds.

In real trading operations, ClickHouse is used for:
- Historical tick data analysis
- Real-time market surveillance
- Performance attribution and P&L analysis
- Risk factor calculations across large portfolios
- Backtesting strategy performance over years of data

## What is ClickHouse?

ClickHouse is an open-source columnar database management system designed for Online Analytical Processing (OLAP). Key characteristics:

| Feature | Description |
|---------|-------------|
| **Columnar Storage** | Data stored by columns, not rows — reads only needed columns |
| **Compression** | Achieves 10-20x compression ratios on market data |
| **Vectorized Execution** | Processes data in batches using SIMD instructions |
| **Real-time Ingestion** | Handles millions of inserts per second |
| **SQL Support** | Full SQL with extensions for time-series analysis |
| **Distributed** | Scales horizontally across multiple nodes |

## Setting Up ClickHouse with Rust

Add the ClickHouse client to your `Cargo.toml`:

```toml
[dependencies]
clickhouse = { version = "0.11", features = ["time", "uuid"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
time = { version = "0.3", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## Basic Connection and Table Creation

```rust
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct Trade {
    #[serde(with = "clickhouse::serde::time::datetime64::micros")]
    timestamp: OffsetDateTime,
    symbol: String,
    exchange: String,
    price: f64,
    quantity: f64,
    side: String,  // "buy" or "sell"
    trade_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to ClickHouse
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("trading");

    // Create a table optimized for time-series trading data
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                timestamp DateTime64(6),
                symbol LowCardinality(String),
                exchange LowCardinality(String),
                price Float64,
                quantity Float64,
                side LowCardinality(String),
                trade_id String
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, timestamp)
            "#,
        )
        .execute()
        .await?;

    println!("Trades table created successfully!");
    Ok(())
}
```

### Understanding the Table Design

```
Column Types for Trading Data:
┌─────────────────────────────────────────────────────────────┐
│ DateTime64(6)     → Microsecond precision timestamps        │
│ LowCardinality    → Optimized for columns with few values   │
│                     (symbols, exchanges, sides)             │
│ Float64           → Standard precision for prices/quantities│
│ String            → Variable-length text (trade IDs)        │
├─────────────────────────────────────────────────────────────┤
│ ENGINE = MergeTree()                                        │
│ ├── PARTITION BY toYYYYMM(timestamp)                        │
│ │   └── Monthly partitions for efficient data management    │
│ └── ORDER BY (symbol, timestamp)                            │
│     └── Optimizes queries filtering by symbol + time range  │
└─────────────────────────────────────────────────────────────┘
```

## Inserting Trading Data

### Single Insert

```rust
use clickhouse::Client;
use time::OffsetDateTime;
use uuid::Uuid;

async fn insert_single_trade(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let trade = Trade {
        timestamp: OffsetDateTime::now_utc(),
        symbol: "BTC/USDT".to_string(),
        exchange: "Binance".to_string(),
        price: 42150.50,
        quantity: 0.5,
        side: "buy".to_string(),
        trade_id: Uuid::new_v4().to_string(),
    };

    let mut insert = client.insert("trades")?;
    insert.write(&trade).await?;
    insert.end().await?;

    println!("Trade inserted: {:?}", trade);
    Ok(())
}
```

### Batch Insert for High-Throughput

```rust
use clickhouse::Client;
use time::OffsetDateTime;
use uuid::Uuid;
use std::time::Duration;

async fn insert_trades_batch(
    client: &Client,
    trades: Vec<Trade>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();

    let mut insert = client.insert("trades")?;

    for trade in &trades {
        insert.write(trade).await?;
    }

    insert.end().await?;

    let elapsed = start.elapsed();
    let rate = trades.len() as f64 / elapsed.as_secs_f64();

    println!(
        "Inserted {} trades in {:?} ({:.0} trades/sec)",
        trades.len(),
        elapsed,
        rate
    );

    Ok(())
}

// Generate sample trading data
fn generate_sample_trades(count: usize) -> Vec<Trade> {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT", "XRP/USDT"];
    let exchanges = vec!["Binance", "Coinbase", "Kraken", "OKX"];
    let base_prices = vec![42000.0, 2200.0, 95.0, 0.62];

    (0..count)
        .map(|i| {
            let symbol_idx = i % symbols.len();
            let exchange_idx = (i / 2) % exchanges.len();
            let price_variance = (i as f64 * 0.001).sin() * 100.0;

            Trade {
                timestamp: OffsetDateTime::now_utc(),
                symbol: symbols[symbol_idx].to_string(),
                exchange: exchanges[exchange_idx].to_string(),
                price: base_prices[symbol_idx] + price_variance,
                quantity: 0.1 + (i as f64 % 10.0) * 0.1,
                side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                trade_id: Uuid::new_v4().to_string(),
            }
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("trading");

    // Insert 100,000 sample trades
    let trades = generate_sample_trades(100_000);
    insert_trades_batch(&client, trades).await?;

    Ok(())
}
```

## Analytical Queries for Trading

### VWAP Calculation

```rust
use clickhouse::{Client, Row};
use serde::Deserialize;

#[derive(Debug, Row, Deserialize)]
struct VwapResult {
    symbol: String,
    vwap: f64,
    total_volume: f64,
    trade_count: u64,
}

async fn calculate_vwap(
    client: &Client,
    symbol: &str,
    hours: u32,
) -> Result<VwapResult, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            symbol,
            sum(price * quantity) / sum(quantity) AS vwap,
            sum(quantity) AS total_volume,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL ? HOUR
        GROUP BY symbol
    "#;

    let result = client
        .query(query)
        .bind(symbol)
        .bind(hours)
        .fetch_one::<VwapResult>()
        .await?;

    println!(
        "{} VWAP ({}h): ${:.2} | Volume: {:.4} | Trades: {}",
        result.symbol, hours, result.vwap, result.total_volume, result.trade_count
    );

    Ok(result)
}
```

### Time-Bucketed OHLCV (Candlestick) Data

```rust
#[derive(Debug, Row, Deserialize)]
struct Candle {
    bucket: String,
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_count: u64,
}

async fn get_ohlcv_candles(
    client: &Client,
    symbol: &str,
    interval_minutes: u32,
    limit: u32,
) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            toStartOfInterval(timestamp, INTERVAL ? MINUTE) AS bucket,
            symbol,
            argMin(price, timestamp) AS open,
            max(price) AS high,
            min(price) AS low,
            argMax(price, timestamp) AS close,
            sum(quantity) AS volume,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
        GROUP BY bucket, symbol
        ORDER BY bucket DESC
        LIMIT ?
    "#;

    let candles = client
        .query(query)
        .bind(interval_minutes)
        .bind(symbol)
        .bind(limit)
        .fetch_all::<Candle>()
        .await?;

    for candle in &candles {
        println!(
            "{} | O:{:.2} H:{:.2} L:{:.2} C:{:.2} | Vol:{:.4}",
            candle.bucket, candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }

    Ok(candles)
}
```

### Exchange Volume Comparison

```rust
#[derive(Debug, Row, Deserialize)]
struct ExchangeVolume {
    exchange: String,
    symbol: String,
    total_volume: f64,
    buy_volume: f64,
    sell_volume: f64,
    avg_trade_size: f64,
    trade_count: u64,
}

async fn compare_exchange_volumes(
    client: &Client,
    symbol: &str,
) -> Result<Vec<ExchangeVolume>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            exchange,
            symbol,
            sum(quantity) AS total_volume,
            sumIf(quantity, side = 'buy') AS buy_volume,
            sumIf(quantity, side = 'sell') AS sell_volume,
            avg(quantity) AS avg_trade_size,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL 24 HOUR
        GROUP BY exchange, symbol
        ORDER BY total_volume DESC
    "#;

    let volumes = client
        .query(query)
        .bind(symbol)
        .fetch_all::<ExchangeVolume>()
        .await?;

    println!("\n=== {} Volume by Exchange (24h) ===", symbol);
    for vol in &volumes {
        let buy_ratio = vol.buy_volume / vol.total_volume * 100.0;
        println!(
            "{}: {:.4} total | Buy: {:.1}% | Avg size: {:.6} | {} trades",
            vol.exchange, vol.total_volume, buy_ratio, vol.avg_trade_size, vol.trade_count
        );
    }

    Ok(volumes)
}
```

## Advanced Analytics: Spread and Slippage Analysis

```rust
#[derive(Debug, Row, Deserialize)]
struct SpreadAnalysis {
    time_bucket: String,
    symbol: String,
    avg_spread_bps: f64,
    max_spread_bps: f64,
    min_spread_bps: f64,
    quote_count: u64,
}

// Table for order book snapshots
async fn create_orderbook_table(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS orderbook_snapshots (
                timestamp DateTime64(6),
                symbol LowCardinality(String),
                exchange LowCardinality(String),
                best_bid Float64,
                best_ask Float64,
                bid_size Float64,
                ask_size Float64,
                mid_price Float64,
                spread_bps Float64
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, exchange, timestamp)
            "#,
        )
        .execute()
        .await?;

    Ok(())
}

async fn analyze_spreads(
    client: &Client,
    symbol: &str,
    interval_minutes: u32,
) -> Result<Vec<SpreadAnalysis>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            toStartOfInterval(timestamp, INTERVAL ? MINUTE) AS time_bucket,
            symbol,
            avg(spread_bps) AS avg_spread_bps,
            max(spread_bps) AS max_spread_bps,
            min(spread_bps) AS min_spread_bps,
            count() AS quote_count
        FROM orderbook_snapshots
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL 24 HOUR
        GROUP BY time_bucket, symbol
        ORDER BY time_bucket
    "#;

    let analysis = client
        .query(query)
        .bind(interval_minutes)
        .bind(symbol)
        .fetch_all::<SpreadAnalysis>()
        .await?;

    println!("\n=== {} Spread Analysis ({}min buckets) ===", symbol, interval_minutes);
    for row in &analysis {
        println!(
            "{} | Avg: {:.2}bps | Range: {:.2}-{:.2}bps | {} quotes",
            row.time_bucket, row.avg_spread_bps, row.min_spread_bps, row.max_spread_bps, row.quote_count
        );
    }

    Ok(analysis)
}
```

## Real-Time Aggregation with Materialized Views

```rust
async fn create_materialized_views(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // Create a target table for minute-level aggregations
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS trades_1m (
                timestamp DateTime,
                symbol LowCardinality(String),
                open Float64,
                high Float64,
                low Float64,
                close Float64,
                volume Float64,
                buy_volume Float64,
                sell_volume Float64,
                trade_count UInt64,
                vwap Float64
            )
            ENGINE = SummingMergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, timestamp)
            "#,
        )
        .execute()
        .await?;

    // Create materialized view that auto-aggregates incoming trades
    client
        .query(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS trades_1m_mv
            TO trades_1m
            AS SELECT
                toStartOfMinute(timestamp) AS timestamp,
                symbol,
                argMin(price, timestamp) AS open,
                max(price) AS high,
                min(price) AS low,
                argMax(price, timestamp) AS close,
                sum(quantity) AS volume,
                sumIf(quantity, side = 'buy') AS buy_volume,
                sumIf(quantity, side = 'sell') AS sell_volume,
                count() AS trade_count,
                sum(price * quantity) / sum(quantity) AS vwap
            FROM trades
            GROUP BY timestamp, symbol
            "#,
        )
        .execute()
        .await?;

    println!("Materialized view created - trades will auto-aggregate to 1-minute candles!");
    Ok(())
}
```

## Portfolio Analytics

```rust
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct PortfolioTrade {
    #[serde(with = "clickhouse::serde::time::datetime64::micros")]
    timestamp: OffsetDateTime,
    portfolio_id: String,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    fees: f64,
    pnl: f64,
}

#[derive(Debug, Row, Deserialize)]
struct PortfolioPerformance {
    portfolio_id: String,
    total_trades: u64,
    winning_trades: u64,
    win_rate: f64,
    total_pnl: f64,
    avg_pnl_per_trade: f64,
    max_drawdown: f64,
    sharpe_approx: f64,
}

async fn analyze_portfolio_performance(
    client: &Client,
    portfolio_id: &str,
) -> Result<PortfolioPerformance, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            portfolio_id,
            count() AS total_trades,
            countIf(pnl > 0) AS winning_trades,
            countIf(pnl > 0) / count() * 100 AS win_rate,
            sum(pnl) AS total_pnl,
            avg(pnl) AS avg_pnl_per_trade,
            min(runningAccumulate(sumState(pnl))) AS max_drawdown,
            avg(pnl) / stddevPop(pnl) AS sharpe_approx
        FROM portfolio_trades
        WHERE portfolio_id = ?
        GROUP BY portfolio_id
    "#;

    let perf = client
        .query(query)
        .bind(portfolio_id)
        .fetch_one::<PortfolioPerformance>()
        .await?;

    println!("\n=== Portfolio {} Performance ===", portfolio_id);
    println!("Total Trades: {}", perf.total_trades);
    println!("Win Rate: {:.1}%", perf.win_rate);
    println!("Total P&L: ${:.2}", perf.total_pnl);
    println!("Avg P&L/Trade: ${:.2}", perf.avg_pnl_per_trade);
    println!("Sharpe (approx): {:.2}", perf.sharpe_approx);

    Ok(perf)
}
```

## Streaming Inserts with Async Channels

```rust
use tokio::sync::mpsc;
use std::time::Duration;

struct TradingDataPipeline {
    client: Client,
    buffer_size: usize,
}

impl TradingDataPipeline {
    fn new(client: Client, buffer_size: usize) -> Self {
        Self { client, buffer_size }
    }

    async fn start_ingestion(
        &self,
        mut receiver: mpsc::Receiver<Trade>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer: Vec<Trade> = Vec::with_capacity(self.buffer_size);
        let mut last_flush = std::time::Instant::now();
        let flush_interval = Duration::from_secs(1);

        loop {
            tokio::select! {
                Some(trade) = receiver.recv() => {
                    buffer.push(trade);

                    // Flush when buffer is full
                    if buffer.len() >= self.buffer_size {
                        self.flush_buffer(&mut buffer).await?;
                        last_flush = std::time::Instant::now();
                    }
                }
                _ = tokio::time::sleep(flush_interval) => {
                    // Periodic flush even if buffer isn't full
                    if !buffer.is_empty() && last_flush.elapsed() >= flush_interval {
                        self.flush_buffer(&mut buffer).await?;
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        }
    }

    async fn flush_buffer(
        &self,
        buffer: &mut Vec<Trade>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut insert = self.client.insert("trades")?;
        for trade in buffer.iter() {
            insert.write(trade).await?;
        }
        insert.end().await?;

        println!("Flushed {} trades to ClickHouse", buffer.len());
        buffer.clear();

        Ok(())
    }
}

// Example usage with simulated market data
async fn simulate_market_feed(sender: mpsc::Sender<Trade>) {
    let symbols = vec!["BTC/USDT", "ETH/USDT"];
    let mut counter = 0u64;

    loop {
        for symbol in &symbols {
            let trade = Trade {
                timestamp: OffsetDateTime::now_utc(),
                symbol: symbol.to_string(),
                exchange: "Binance".to_string(),
                price: 42000.0 + (counter as f64 * 0.1).sin() * 100.0,
                quantity: 0.1 + (counter % 10) as f64 * 0.05,
                side: if counter % 2 == 0 { "buy" } else { "sell" }.to_string(),
                trade_id: format!("T{}", counter),
            };

            if sender.send(trade).await.is_err() {
                return;
            }
            counter += 1;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Columnar Storage | ClickHouse stores data by columns for fast analytical queries |
| MergeTree Engine | Primary table engine with sorting, partitioning, and merging |
| LowCardinality | Optimization for columns with few distinct values |
| DateTime64 | Microsecond-precision timestamps for tick data |
| Materialized Views | Auto-aggregate data on insert for real-time analytics |
| Batch Inserts | Buffer and batch writes for high throughput |
| VWAP/OHLCV | Standard trading analytics computed efficiently |

## Exercises

1. **Multi-Exchange Arbitrage Detector**: Write a query that finds price discrepancies greater than 0.1% between exchanges for the same symbol within 1-second windows.

2. **Volume Profile Analysis**: Create a query that calculates volume distribution across price levels (Price Volume Profile) for a given symbol over the last 24 hours.

3. **Trade Flow Imbalance**: Implement a streaming calculation that tracks buy/sell volume imbalance in real-time and stores alerts when imbalance exceeds a threshold.

4. **Performance Attribution**: Design tables and queries to track P&L attribution by symbol, strategy, and time period for a multi-strategy trading system.

## Homework

1. **Tick Data Warehouse**: Design a complete schema for storing:
   - Trade ticks
   - Order book snapshots (top 10 levels)
   - Funding rates
   - Liquidation events

   Include appropriate partitioning, sorting keys, and materialized views for common queries.

2. **Backtesting Data Pipeline**: Build a Rust service that:
   - Reads historical trade data from CSV files
   - Transforms and validates the data
   - Bulk loads into ClickHouse with progress reporting
   - Handles duplicate detection and data quality checks

3. **Real-time Dashboard Backend**: Create an async Rust service that:
   - Exposes REST endpoints for common trading analytics
   - Implements caching for frequently-accessed data
   - Supports WebSocket streaming of live aggregations
   - Handles concurrent queries efficiently

4. **Cross-Exchange Analysis Tool**: Implement a CLI tool that:
   - Compares execution quality across exchanges
   - Calculates effective spreads and market impact
   - Generates reports on best execution venues
   - Supports multiple time granularities

## Navigation

[← Previous day](../238-timescaledb-time-series/en.md) | [Next day →](../240-questdb-market-data/en.md)
