# Day 238: TimescaleDB: Time Series — Storing Market History

## Trading Analogy

Every trader knows that markets have memory. Price movements, volume spikes, order flow — all of this data has a timestamp attached to it. A candlestick chart isn't just a visualization; it's a time series of OHLCV (Open, High, Low, Close, Volume) data points. To analyze the market, backtest strategies, or detect patterns, you need to store and query millions of these time-stamped records efficiently.

Traditional databases struggle with time series data at scale. Imagine trying to query "What was the average volume for BTC/USDT in 5-minute intervals over the last year?" from a table with 100 million rows. This is where **TimescaleDB** shines — it's PostgreSQL optimized for time series, automatically partitioning your data by time and providing lightning-fast aggregations.

## What is TimescaleDB?

TimescaleDB is an open-source time series database built on PostgreSQL. It provides:

- **Hypertables**: Automatic time-based partitioning
- **Continuous Aggregates**: Pre-computed rollups that update automatically
- **Compression**: Up to 95% storage reduction for historical data
- **Full SQL**: All PostgreSQL features, including joins and indexes
- **Time Bucket Functions**: Easy aggregation by time intervals

For trading applications, this means you can:
- Store tick-by-tick data efficiently
- Query historical OHLCV candles in milliseconds
- Calculate moving averages, VWAP, and other indicators on the fly
- Join price data with order history and portfolio snapshots

## Setting Up TimescaleDB with Rust

First, add the necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = "1.33"
serde = { version = "1.0", features = ["derive"] }
```

## Connecting to TimescaleDB

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to TimescaleDB (same as PostgreSQL connection)
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Verify TimescaleDB extension is enabled
    let row = client
        .query_one("SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'", &[])
        .await?;

    let version: &str = row.get(0);
    println!("TimescaleDB version: {}", version);

    Ok(())
}
```

## Creating a Hypertable for OHLCV Data

A hypertable is TimescaleDB's core concept — it looks like a regular table but automatically partitions data by time:

```rust
use tokio_postgres::{NoTls, Error};

async fn create_ohlcv_table(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Create the base table
    client.execute(
        "CREATE TABLE IF NOT EXISTS ohlcv (
            time        TIMESTAMPTZ NOT NULL,
            symbol      VARCHAR(20) NOT NULL,
            open        DECIMAL(18, 8) NOT NULL,
            high        DECIMAL(18, 8) NOT NULL,
            low         DECIMAL(18, 8) NOT NULL,
            close       DECIMAL(18, 8) NOT NULL,
            volume      DECIMAL(24, 8) NOT NULL,
            trade_count INTEGER DEFAULT 0
        )",
        &[],
    ).await?;

    // Convert to hypertable with 1-day chunks
    // If table already exists as hypertable, this will error (handle appropriately)
    match client.execute(
        "SELECT create_hypertable('ohlcv', 'time',
            chunk_time_interval => INTERVAL '1 day',
            if_not_exists => TRUE
        )",
        &[],
    ).await {
        Ok(_) => println!("Hypertable created successfully"),
        Err(e) => println!("Hypertable may already exist: {}", e),
    }

    // Create indexes for common query patterns
    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_ohlcv_symbol_time
         ON ohlcv (symbol, time DESC)",
        &[],
    ).await?;

    println!("OHLCV table ready for time series data!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    create_ohlcv_table(&client).await?;
    Ok(())
}
```

## Inserting OHLCV Candles

```rust
use chrono::{DateTime, Utc};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
struct Candle {
    time: DateTime<Utc>,
    symbol: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    trade_count: i32,
}

async fn insert_candle(client: &tokio_postgres::Client, candle: &Candle) -> Result<(), Error> {
    client.execute(
        "INSERT INTO ohlcv (time, symbol, open, high, low, close, volume, trade_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &candle.time,
            &candle.symbol,
            &candle.open,
            &candle.high,
            &candle.low,
            &candle.close,
            &candle.volume,
            &candle.trade_count,
        ],
    ).await?;

    Ok(())
}

async fn insert_candles_batch(
    client: &tokio_postgres::Client,
    candles: &[Candle]
) -> Result<u64, Error> {
    // For bulk inserts, use COPY or batch INSERT
    let mut count = 0u64;

    for candle in candles {
        client.execute(
            "INSERT INTO ohlcv (time, symbol, open, high, low, close, volume, trade_count)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT DO NOTHING",
            &[
                &candle.time,
                &candle.symbol,
                &candle.open,
                &candle.high,
                &candle.low,
                &candle.close,
                &candle.volume,
                &candle.trade_count,
            ],
        ).await?;
        count += 1;
    }

    println!("Inserted {} candles", count);
    Ok(count)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Example: Insert a single candle
    let candle = Candle {
        time: Utc::now(),
        symbol: "BTC/USDT".to_string(),
        open: Decimal::new(4250000, 2),    // 42500.00
        high: Decimal::new(4275000, 2),    // 42750.00
        low: Decimal::new(4240000, 2),     // 42400.00
        close: Decimal::new(4268000, 2),   // 42680.00
        volume: Decimal::new(15234567, 4), // 1523.4567
        trade_count: 4521,
    };

    insert_candle(&client, &candle).await?;
    println!("Candle inserted: {:?}", candle);

    Ok(())
}
```

## Querying Time Series Data

### Basic Queries

```rust
use chrono::{DateTime, Utc, Duration};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

async fn get_latest_candles(
    client: &tokio_postgres::Client,
    symbol: &str,
    limit: i64,
) -> Result<Vec<(DateTime<Utc>, Decimal, Decimal, Decimal, Decimal)>, Error> {
    let rows = client.query(
        "SELECT time, open, high, low, close
         FROM ohlcv
         WHERE symbol = $1
         ORDER BY time DESC
         LIMIT $2",
        &[&symbol, &limit],
    ).await?;

    let candles: Vec<_> = rows.iter().map(|row| {
        (
            row.get::<_, DateTime<Utc>>(0),
            row.get::<_, Decimal>(1),
            row.get::<_, Decimal>(2),
            row.get::<_, Decimal>(3),
            row.get::<_, Decimal>(4),
        )
    }).collect();

    Ok(candles)
}

async fn get_price_range(
    client: &tokio_postgres::Client,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(Decimal, Decimal), Error> {
    let row = client.query_one(
        "SELECT MIN(low) as period_low, MAX(high) as period_high
         FROM ohlcv
         WHERE symbol = $1 AND time >= $2 AND time < $3",
        &[&symbol, &start, &end],
    ).await?;

    Ok((row.get(0), row.get(1)))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Get latest 10 candles for BTC/USDT
    let candles = get_latest_candles(&client, "BTC/USDT", 10).await?;
    println!("Latest candles:");
    for (time, open, high, low, close) in candles {
        println!("  {} | O:{} H:{} L:{} C:{}",
            time.format("%Y-%m-%d %H:%M"),
            open, high, low, close
        );
    }

    Ok(())
}
```

## Time Bucket Aggregations

TimescaleDB's `time_bucket` function is perfect for creating custom timeframe candles:

```rust
use chrono::{DateTime, Utc};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug)]
struct AggregatedCandle {
    bucket: DateTime<Utc>,
    symbol: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    trade_count: i64,
}

async fn get_candles_by_timeframe(
    client: &tokio_postgres::Client,
    symbol: &str,
    interval: &str,  // e.g., "5 minutes", "1 hour", "1 day"
    limit: i64,
) -> Result<Vec<AggregatedCandle>, Error> {
    // Using TimescaleDB's time_bucket and first/last functions
    let query = format!(
        "SELECT
            time_bucket('{}', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count
         FROM ohlcv
         WHERE symbol = $1
         GROUP BY bucket, symbol
         ORDER BY bucket DESC
         LIMIT $2",
        interval
    );

    let rows = client.query(&query, &[&symbol, &limit]).await?;

    let candles: Vec<AggregatedCandle> = rows.iter().map(|row| {
        AggregatedCandle {
            bucket: row.get(0),
            symbol: row.get(1),
            open: row.get(2),
            high: row.get(3),
            low: row.get(4),
            close: row.get(5),
            volume: row.get(6),
            trade_count: row.get(7),
        }
    }).collect();

    Ok(candles)
}

async fn calculate_vwap(
    client: &tokio_postgres::Client,
    symbol: &str,
    interval: &str,
) -> Result<Vec<(DateTime<Utc>, Decimal)>, Error> {
    // Volume Weighted Average Price calculation
    let query = format!(
        "SELECT
            time_bucket('{}', time) AS bucket,
            SUM((high + low + close) / 3 * volume) / SUM(volume) AS vwap
         FROM ohlcv
         WHERE symbol = $1
         GROUP BY bucket
         ORDER BY bucket DESC
         LIMIT 100",
        interval
    );

    let rows = client.query(&query, &[&symbol]).await?;

    let vwap_data: Vec<(DateTime<Utc>, Decimal)> = rows.iter().map(|row| {
        (row.get(0), row.get(1))
    }).collect();

    Ok(vwap_data)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Get 1-hour candles
    println!("=== 1-Hour Candles ===");
    let hourly = get_candles_by_timeframe(&client, "BTC/USDT", "1 hour", 24).await?;
    for candle in hourly.iter().take(5) {
        println!("{}: O={} H={} L={} C={} V={}",
            candle.bucket.format("%Y-%m-%d %H:%M"),
            candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }

    // Calculate VWAP
    println!("\n=== Hourly VWAP ===");
    let vwap = calculate_vwap(&client, "BTC/USDT", "1 hour").await?;
    for (time, price) in vwap.iter().take(5) {
        println!("{}: VWAP = {}", time.format("%Y-%m-%d %H:%M"), price);
    }

    Ok(())
}
```

## Continuous Aggregates for Real-Time Dashboards

Continuous aggregates are materialized views that automatically update:

```rust
use tokio_postgres::{NoTls, Error};

async fn create_continuous_aggregates(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Create a continuous aggregate for hourly data
    client.execute(
        "CREATE MATERIALIZED VIEW IF NOT EXISTS ohlcv_hourly
         WITH (timescaledb.continuous) AS
         SELECT
            time_bucket('1 hour', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count,
            count(*) AS candle_count
         FROM ohlcv
         GROUP BY bucket, symbol
         WITH NO DATA",
        &[],
    ).await?;

    // Create daily aggregate
    client.execute(
        "CREATE MATERIALIZED VIEW IF NOT EXISTS ohlcv_daily
         WITH (timescaledb.continuous) AS
         SELECT
            time_bucket('1 day', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count
         FROM ohlcv
         GROUP BY bucket, symbol
         WITH NO DATA",
        &[],
    ).await?;

    // Set up automatic refresh policy (every hour for hourly data)
    client.execute(
        "SELECT add_continuous_aggregate_policy('ohlcv_hourly',
            start_offset => INTERVAL '3 hours',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour'
        )",
        &[],
    ).await?;

    println!("Continuous aggregates created!");
    Ok(())
}

async fn query_hourly_aggregate(
    client: &tokio_postgres::Client,
    symbol: &str,
    limit: i64,
) -> Result<(), Error> {
    // Query the pre-computed aggregate — much faster!
    let rows = client.query(
        "SELECT bucket, open, high, low, close, volume
         FROM ohlcv_hourly
         WHERE symbol = $1
         ORDER BY bucket DESC
         LIMIT $2",
        &[&symbol, &limit],
    ).await?;

    println!("Hourly data from continuous aggregate:");
    for row in rows {
        let bucket: chrono::DateTime<chrono::Utc> = row.get(0);
        let close: rust_decimal::Decimal = row.get(4);
        let volume: rust_decimal::Decimal = row.get(5);
        println!("  {} | Close: {} | Volume: {}",
            bucket.format("%Y-%m-%d %H:%M"), close, volume
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    create_continuous_aggregates(&client).await?;
    query_hourly_aggregate(&client, "BTC/USDT", 24).await?;

    Ok(())
}
```

## Practical Example: Trading Analytics Dashboard

```rust
use chrono::{DateTime, Utc, Duration};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug)]
struct MarketStats {
    symbol: String,
    current_price: Decimal,
    price_change_24h: Decimal,
    high_24h: Decimal,
    low_24h: Decimal,
    volume_24h: Decimal,
    vwap_24h: Decimal,
    trade_count_24h: i64,
}

#[derive(Debug)]
struct MovingAverages {
    symbol: String,
    sma_20: Decimal,
    sma_50: Decimal,
    ema_12: Decimal,
    ema_26: Decimal,
}

async fn get_market_stats(
    client: &tokio_postgres::Client,
    symbol: &str,
) -> Result<MarketStats, Error> {
    let now = Utc::now();
    let yesterday = now - Duration::hours(24);

    let row = client.query_one(
        "WITH latest AS (
            SELECT close as current_price
            FROM ohlcv
            WHERE symbol = $1
            ORDER BY time DESC
            LIMIT 1
        ),
        day_ago AS (
            SELECT close as price_24h_ago
            FROM ohlcv
            WHERE symbol = $1 AND time <= $2
            ORDER BY time DESC
            LIMIT 1
        ),
        stats AS (
            SELECT
                MAX(high) as high_24h,
                MIN(low) as low_24h,
                SUM(volume) as volume_24h,
                SUM((high + low + close) / 3 * volume) / NULLIF(SUM(volume), 0) as vwap,
                SUM(trade_count) as trade_count
            FROM ohlcv
            WHERE symbol = $1 AND time >= $2
        )
        SELECT
            l.current_price,
            l.current_price - COALESCE(d.price_24h_ago, l.current_price) as change,
            s.high_24h,
            s.low_24h,
            s.volume_24h,
            COALESCE(s.vwap, l.current_price) as vwap,
            COALESCE(s.trade_count, 0) as trade_count
        FROM latest l
        CROSS JOIN stats s
        LEFT JOIN day_ago d ON true",
        &[&symbol, &yesterday],
    ).await?;

    Ok(MarketStats {
        symbol: symbol.to_string(),
        current_price: row.get(0),
        price_change_24h: row.get(1),
        high_24h: row.get(2),
        low_24h: row.get(3),
        volume_24h: row.get(4),
        vwap_24h: row.get(5),
        trade_count_24h: row.get(6),
    })
}

async fn calculate_moving_averages(
    client: &tokio_postgres::Client,
    symbol: &str,
    timeframe: &str,
) -> Result<MovingAverages, Error> {
    // Calculate SMAs using time_bucket
    let row = client.query_one(
        &format!(
            "WITH candles AS (
                SELECT
                    time_bucket('{}', time) as bucket,
                    last(close, time) as close
                FROM ohlcv
                WHERE symbol = $1
                GROUP BY bucket
                ORDER BY bucket DESC
                LIMIT 50
            ),
            numbered AS (
                SELECT close, ROW_NUMBER() OVER (ORDER BY bucket DESC) as rn
                FROM candles
            )
            SELECT
                (SELECT AVG(close) FROM numbered WHERE rn <= 20) as sma_20,
                (SELECT AVG(close) FROM numbered WHERE rn <= 50) as sma_50,
                (SELECT AVG(close) FROM numbered WHERE rn <= 12) as ema_12_approx,
                (SELECT AVG(close) FROM numbered WHERE rn <= 26) as ema_26_approx
            ", timeframe
        ),
        &[&symbol],
    ).await?;

    Ok(MovingAverages {
        symbol: symbol.to_string(),
        sma_20: row.get(0),
        sma_50: row.get(1),
        ema_12: row.get(2),
        ema_26: row.get(3),
    })
}

fn display_dashboard(stats: &MarketStats, ma: &MovingAverages) {
    let change_pct = if stats.current_price != Decimal::ZERO {
        (stats.price_change_24h / stats.current_price) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let trend = if stats.price_change_24h > Decimal::ZERO { "+" } else { "" };

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    TRADING DASHBOARD                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Symbol: {:<54} ║", stats.symbol);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Current Price:  ${:<46} ║", stats.current_price);
    println!("║ 24h Change:     {}{} ({}{:.2}%)                              ║",
        trend, stats.price_change_24h, trend, change_pct);
    println!("║ 24h High:       ${:<46} ║", stats.high_24h);
    println!("║ 24h Low:        ${:<46} ║", stats.low_24h);
    println!("║ 24h Volume:     {:<47} ║", stats.volume_24h);
    println!("║ 24h VWAP:       ${:<46} ║", stats.vwap_24h);
    println!("║ 24h Trades:     {:<47} ║", stats.trade_count_24h);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                   MOVING AVERAGES                            ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ SMA(20):        ${:<46} ║", ma.sma_20);
    println!("║ SMA(50):        ${:<46} ║", ma.sma_50);
    println!("║ EMA(12):        ${:<46} ║", ma.ema_12);
    println!("║ EMA(26):        ${:<46} ║", ma.ema_26);
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Simple trend analysis
    let trend_signal = if stats.current_price > ma.sma_20 && ma.sma_20 > ma.sma_50 {
        "BULLISH - Price above SMAs, short-term uptrend"
    } else if stats.current_price < ma.sma_20 && ma.sma_20 < ma.sma_50 {
        "BEARISH - Price below SMAs, short-term downtrend"
    } else {
        "NEUTRAL - Mixed signals, consolidation phase"
    };

    println!("║ Trend:          {:<46} ║", trend_signal);
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let symbol = "BTC/USDT";

    // Fetch market stats and moving averages
    let stats = get_market_stats(&client, symbol).await?;
    let ma = calculate_moving_averages(&client, symbol, "1 hour").await?;

    // Display the dashboard
    display_dashboard(&stats, &ma);

    Ok(())
}
```

## Compression for Historical Data

TimescaleDB can compress old data to save storage:

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_compression(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Enable compression on the hypertable
    client.execute(
        "ALTER TABLE ohlcv SET (
            timescaledb.compress,
            timescaledb.compress_segmentby = 'symbol',
            timescaledb.compress_orderby = 'time DESC'
        )",
        &[],
    ).await?;

    // Add compression policy: compress chunks older than 7 days
    client.execute(
        "SELECT add_compression_policy('ohlcv', INTERVAL '7 days')",
        &[],
    ).await?;

    println!("Compression policy enabled!");

    // Check compression stats
    let rows = client.query(
        "SELECT
            hypertable_name,
            total_chunks,
            compressed_chunks,
            pg_size_pretty(before_compression_total_bytes) as before,
            pg_size_pretty(after_compression_total_bytes) as after
         FROM timescaledb_information.compression_stats",
        &[],
    ).await?;

    for row in rows {
        let table: &str = row.get(0);
        let total: i64 = row.get(1);
        let compressed: i64 = row.get(2);
        let before: &str = row.get(3);
        let after: &str = row.get(4);

        println!("Table: {} | Chunks: {}/{} compressed | Size: {} -> {}",
            table, compressed, total, before, after
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    setup_compression(&client).await?;
    Ok(())
}
```

## Data Retention Policies

Automatically drop old data to manage storage:

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_retention_policy(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Keep only last 365 days of minute data
    client.execute(
        "SELECT add_retention_policy('ohlcv', INTERVAL '365 days')",
        &[],
    ).await?;

    println!("Retention policy: keeping last 365 days of data");

    // For the continuous aggregates, keep them longer
    // Hourly data: 2 years
    client.execute(
        "SELECT add_retention_policy('ohlcv_hourly', INTERVAL '730 days')",
        &[],
    ).await?;

    // Daily data: 5 years
    client.execute(
        "SELECT add_retention_policy('ohlcv_daily', INTERVAL '1825 days')",
        &[],
    ).await?;

    println!("Aggregates retention: hourly=2 years, daily=5 years");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    setup_retention_policy(&client).await?;
    Ok(())
}
```

## What We Learned

| Concept | Description | Trading Use Case |
|---------|-------------|------------------|
| Hypertable | Auto-partitioned time series table | Store millions of OHLCV candles efficiently |
| `time_bucket()` | Aggregate data by time intervals | Create custom timeframe candles (5m, 1h, 1d) |
| `first()` / `last()` | Get first/last value in a time bucket | Calculate open/close prices in aggregations |
| Continuous Aggregates | Auto-updating materialized views | Real-time dashboards, pre-computed indicators |
| Compression | Reduce storage for historical data | Store years of tick data cost-effectively |
| Retention Policy | Automatic data cleanup | Keep granular data for X days, aggregates longer |

## Homework

1. **Tick Data Storage**: Create a hypertable for storing individual trades (tick data) with fields: `time`, `symbol`, `price`, `quantity`, `side` (buy/sell). Write functions to:
   - Insert ticks in batches of 1000
   - Aggregate ticks into 1-minute OHLCV candles using `time_bucket`
   - Calculate the bid-ask spread from tick data

2. **Multi-Symbol Dashboard**: Extend the trading dashboard example to:
   - Track multiple symbols simultaneously (BTC, ETH, SOL)
   - Show correlation between symbols over the last 24 hours
   - Identify which symbol had the highest volatility (using `stddev()` on returns)

3. **Backtesting Data Pipeline**: Build a data pipeline that:
   - Stores OHLCV data for backtesting
   - Provides a function to fetch data for a date range with a specific timeframe
   - Implements a simple moving average crossover signal generator
   - Stores signals in a separate hypertable with timestamps

4. **Performance Comparison**: Create a benchmark that:
   - Inserts 1 million candles into both a regular PostgreSQL table and a TimescaleDB hypertable
   - Compares query performance for: latest N candles, time range queries, aggregations
   - Measures compression ratio and storage savings
   - Documents the results with timing statistics

## Navigation

[← Previous day](../237-redis-pub-sub-notifications/en.md) | [Next day →](../239-clickhouse-big-data-analytics/en.md)
