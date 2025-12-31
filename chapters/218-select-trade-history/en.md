# Day 218: SELECT: Reading Trade History

## Trading Analogy

Imagine you're an experienced trader who wants to analyze your trading history. You have a journal of all trades for the year: thousands of records with dates, prices, volumes, and results. To find the information you need, you require a powerful search and filtering tool.

**SQL SELECT** is like an advanced search engine for your trading journal:
- **SELECT \*** — show all data (entire trade history)
- **SELECT symbol, price** — show only specific columns (just tickers and prices)
- **WHERE** — filter records (only BTC trades)
- **ORDER BY** — sort results (by date, by profit)
- **LIMIT** — limit the number of records (last 10 trades)

In real trading, SELECT is used for:
- Analyzing order execution history
- Calculating portfolio statistics
- Finding trading anomalies
- Generating tax reports

## What is SELECT?

SELECT is the primary SQL command for reading data from a database. It allows you to:

1. **Choose columns** — which fields you need
2. **Filter rows** — which records to select
3. **Sort results** — in what order to return them
4. **Group data** — aggregate by categories
5. **Join tables** — combine data from different sources

## Basic SELECT Syntax

```sql
SELECT column1, column2, ...
FROM table_name
WHERE condition
ORDER BY column
LIMIT count;
```

## Project Setup with rusqlite

```rust
// Cargo.toml
// [dependencies]
// rusqlite = { version = "0.31", features = ["bundled"] }

use rusqlite::{Connection, Result, params};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Create trades table
    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            total REAL NOT NULL,
            executed_at TEXT NOT NULL,
            pnl REAL
        )",
        [],
    )?;

    // Populate with test data
    let trades = vec![
        ("BTC", "buy", 42000.0, 0.5, 21000.0, "2024-01-15 10:30:00", None),
        ("BTC", "sell", 43500.0, 0.3, 13050.0, "2024-01-15 14:45:00", Some(450.0)),
        ("ETH", "buy", 2200.0, 5.0, 11000.0, "2024-01-16 09:00:00", None),
        ("BTC", "buy", 41000.0, 0.2, 8200.0, "2024-01-17 11:20:00", None),
        ("ETH", "sell", 2350.0, 3.0, 7050.0, "2024-01-18 16:30:00", Some(450.0)),
        ("SOL", "buy", 95.0, 50.0, 4750.0, "2024-01-19 08:15:00", None),
        ("BTC", "sell", 44000.0, 0.4, 17600.0, "2024-01-20 12:00:00", Some(1200.0)),
        ("SOL", "sell", 105.0, 30.0, 3150.0, "2024-01-21 15:45:00", Some(300.0)),
        ("ETH", "buy", 2100.0, 2.0, 4200.0, "2024-01-22 10:00:00", None),
        ("BTC", "buy", 43000.0, 0.1, 4300.0, "2024-01-23 09:30:00", None),
    ];

    for trade in trades {
        conn.execute(
            "INSERT INTO trades (symbol, side, price, quantity, total, executed_at, pnl)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![trade.0, trade.1, trade.2, trade.3, trade.4, trade.5, trade.6],
        )?;
    }

    println!("Database ready!");
    Ok(())
}
```

## SELECT * — Select All Columns

```rust
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    total: f64,
    executed_at: String,
    pnl: Option<f64>,
}

fn get_all_trades(conn: &Connection) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare("SELECT * FROM trades")?;

    let trades = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    let all_trades = get_all_trades(&conn)?;

    println!("All trades ({} records):", all_trades.len());
    for trade in &all_trades {
        println!("{:?}", trade);
    }

    Ok(())
}
```

## SELECT with Specific Columns

```rust
use rusqlite::{Connection, Result};

fn get_trade_summary(conn: &Connection) -> Result<()> {
    // Select only needed columns
    let mut stmt = conn.prepare(
        "SELECT symbol, price, quantity, executed_at FROM trades"
    )?;

    let trades = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    println!("Trade Summary:");
    println!("{:<8} {:>12} {:>10} {}", "Symbol", "Price", "Quantity", "Date");
    println!("{}", "-".repeat(50));

    for trade in trades {
        let (symbol, price, quantity, date) = trade?;
        println!("{:<8} {:>12.2} {:>10.4} {}", symbol, price, quantity, date);
    }

    Ok(())
}
```

## WHERE — Data Filtering

### Filter by Symbol

```rust
use rusqlite::{Connection, Result, params};

fn get_trades_by_symbol(conn: &Connection, symbol: &str) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades WHERE symbol = ?1"
    )?;

    let trades = stmt.query_map(params![symbol], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // All BTC trades
    let btc_trades = get_trades_by_symbol(&conn, "BTC")?;
    println!("BTC trades: {} records", btc_trades.len());

    // All ETH trades
    let eth_trades = get_trades_by_symbol(&conn, "ETH")?;
    println!("ETH trades: {} records", eth_trades.len());

    Ok(())
}
```

### Multiple Conditions

```rust
use rusqlite::{Connection, Result, params};

fn get_profitable_btc_sells(conn: &Connection) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades
         WHERE symbol = ?1
           AND side = ?2
           AND pnl > 0"
    )?;

    let trades = stmt.query_map(params!["BTC", "sell"], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn get_large_trades(conn: &Connection, min_total: f64) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades WHERE total >= ?1"
    )?;

    let trades = stmt.query_map(params![min_total], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Profitable BTC sells
    let profitable = get_profitable_btc_sells(&conn)?;
    println!("Profitable BTC sells:");
    for trade in &profitable {
        println!("  {} @ {} = +${:.2}",
            trade.quantity, trade.price, trade.pnl.unwrap_or(0.0));
    }

    // Large trades (over $10,000)
    let large = get_large_trades(&conn, 10000.0)?;
    println!("\nLarge trades (> $10,000): {} records", large.len());

    Ok(())
}
```

### Comparison Operators and Ranges

```rust
use rusqlite::{Connection, Result, params};

fn get_trades_in_price_range(
    conn: &Connection,
    symbol: &str,
    min_price: f64,
    max_price: f64,
) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades
         WHERE symbol = ?1
           AND price BETWEEN ?2 AND ?3"
    )?;

    let trades = stmt.query_map(params![symbol, min_price, max_price], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn get_trades_by_date_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades
         WHERE executed_at >= ?1 AND executed_at <= ?2"
    )?;

    let trades = stmt.query_map(params![start_date, end_date], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // BTC in range $42,000 - $44,000
    let btc_range = get_trades_in_price_range(&conn, "BTC", 42000.0, 44000.0)?;
    println!("BTC in range $42k-$44k: {} trades", btc_range.len());

    // Trades in specific period
    let period = get_trades_by_date_range(
        &conn,
        "2024-01-15 00:00:00",
        "2024-01-18 23:59:59"
    )?;
    println!("Trades from Jan 15-18: {} records", period.len());

    Ok(())
}
```

## ORDER BY — Sorting Results

```rust
use rusqlite::{Connection, Result};

fn get_trades_sorted_by_total(conn: &Connection, ascending: bool) -> Result<Vec<Trade>> {
    let order = if ascending { "ASC" } else { "DESC" };
    let sql = format!("SELECT * FROM trades ORDER BY total {}", order);

    let mut stmt = conn.prepare(&sql)?;

    let trades = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn get_trades_sorted_by_date(conn: &Connection) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades ORDER BY executed_at DESC"
    )?;

    let trades = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn get_trades_multi_sort(conn: &Connection) -> Result<Vec<Trade>> {
    // Sort by symbol, then by date
    let mut stmt = conn.prepare(
        "SELECT * FROM trades ORDER BY symbol ASC, executed_at DESC"
    )?;

    let trades = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Largest trades
    println!("Top 5 largest trades:");
    let largest = get_trades_sorted_by_total(&conn, false)?;
    for trade in largest.iter().take(5) {
        println!("  {} {} {} @ {} = ${:.2}",
            trade.symbol, trade.side, trade.quantity, trade.price, trade.total);
    }

    // Recent trades
    println!("\nLast 3 trades:");
    let recent = get_trades_sorted_by_date(&conn)?;
    for trade in recent.iter().take(3) {
        println!("  {} - {} {}", trade.executed_at, trade.symbol, trade.side);
    }

    Ok(())
}
```

## LIMIT and OFFSET — Pagination

```rust
use rusqlite::{Connection, Result, params};

fn get_trades_paginated(
    conn: &Connection,
    page: usize,
    per_page: usize,
) -> Result<Vec<Trade>> {
    let offset = (page - 1) * per_page;

    let mut stmt = conn.prepare(
        "SELECT * FROM trades
         ORDER BY executed_at DESC
         LIMIT ?1 OFFSET ?2"
    )?;

    let trades = stmt.query_map(params![per_page as i64, offset as i64], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn get_last_n_trades(conn: &Connection, n: usize) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM trades ORDER BY executed_at DESC LIMIT ?1"
    )?;

    let trades = stmt.query_map(params![n as i64], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            total: row.get(5)?,
            executed_at: row.get(6)?,
            pnl: row.get(7)?,
        })
    })?;

    trades.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Pagination: 3 records per page
    println!("Page 1:");
    for trade in get_trades_paginated(&conn, 1, 3)? {
        println!("  {} {} {}", trade.executed_at, trade.symbol, trade.side);
    }

    println!("\nPage 2:");
    for trade in get_trades_paginated(&conn, 2, 3)? {
        println!("  {} {} {}", trade.executed_at, trade.symbol, trade.side);
    }

    // Last 5 trades
    println!("\nLast 5 trades:");
    for trade in get_last_n_trades(&conn, 5)? {
        println!("  {} - {} {} {}",
            trade.executed_at, trade.symbol, trade.side, trade.quantity);
    }

    Ok(())
}
```

## Aggregate Functions

```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug)]
struct TradingStats {
    total_trades: i64,
    total_volume: f64,
    avg_trade_size: f64,
    max_trade: f64,
    min_trade: f64,
}

fn get_trading_stats(conn: &Connection) -> Result<TradingStats> {
    let mut stmt = conn.prepare(
        "SELECT
            COUNT(*) as total_trades,
            SUM(total) as total_volume,
            AVG(total) as avg_trade_size,
            MAX(total) as max_trade,
            MIN(total) as min_trade
         FROM trades"
    )?;

    stmt.query_row([], |row| {
        Ok(TradingStats {
            total_trades: row.get(0)?,
            total_volume: row.get(1)?,
            avg_trade_size: row.get(2)?,
            max_trade: row.get(3)?,
            min_trade: row.get(4)?,
        })
    })
}

fn get_total_pnl(conn: &Connection) -> Result<f64> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(SUM(pnl), 0) FROM trades WHERE pnl IS NOT NULL"
    )?;

    stmt.query_row([], |row| row.get(0))
}

fn get_symbol_stats(conn: &Connection, symbol: &str) -> Result<TradingStats> {
    let mut stmt = conn.prepare(
        "SELECT
            COUNT(*),
            COALESCE(SUM(total), 0),
            COALESCE(AVG(total), 0),
            COALESCE(MAX(total), 0),
            COALESCE(MIN(total), 0)
         FROM trades
         WHERE symbol = ?1"
    )?;

    stmt.query_row(params![symbol], |row| {
        Ok(TradingStats {
            total_trades: row.get(0)?,
            total_volume: row.get(1)?,
            avg_trade_size: row.get(2)?,
            max_trade: row.get(3)?,
            min_trade: row.get(4)?,
        })
    })
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Overall statistics
    let stats = get_trading_stats(&conn)?;
    println!("=== Overall Statistics ===");
    println!("Total trades: {}", stats.total_trades);
    println!("Total volume: ${:.2}", stats.total_volume);
    println!("Average trade size: ${:.2}", stats.avg_trade_size);
    println!("Max trade: ${:.2}", stats.max_trade);
    println!("Min trade: ${:.2}", stats.min_trade);

    // PnL
    let total_pnl = get_total_pnl(&conn)?;
    println!("\nTotal PnL: ${:.2}", total_pnl);

    // BTC statistics
    let btc_stats = get_symbol_stats(&conn, "BTC")?;
    println!("\n=== BTC Statistics ===");
    println!("Trades: {}", btc_stats.total_trades);
    println!("Volume: ${:.2}", btc_stats.total_volume);

    Ok(())
}
```

## GROUP BY — Grouping Data

```rust
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct SymbolSummary {
    symbol: String,
    trade_count: i64,
    total_volume: f64,
    avg_price: f64,
}

fn get_volume_by_symbol(conn: &Connection) -> Result<Vec<SymbolSummary>> {
    let mut stmt = conn.prepare(
        "SELECT
            symbol,
            COUNT(*) as trade_count,
            SUM(total) as total_volume,
            AVG(price) as avg_price
         FROM trades
         GROUP BY symbol
         ORDER BY total_volume DESC"
    )?;

    let summaries = stmt.query_map([], |row| {
        Ok(SymbolSummary {
            symbol: row.get(0)?,
            trade_count: row.get(1)?,
            total_volume: row.get(2)?,
            avg_price: row.get(3)?,
        })
    })?;

    summaries.collect()
}

#[derive(Debug)]
struct SideSummary {
    side: String,
    trade_count: i64,
    total_volume: f64,
}

fn get_volume_by_side(conn: &Connection) -> Result<Vec<SideSummary>> {
    let mut stmt = conn.prepare(
        "SELECT
            side,
            COUNT(*) as trade_count,
            SUM(total) as total_volume
         FROM trades
         GROUP BY side"
    )?;

    let summaries = stmt.query_map([], |row| {
        Ok(SideSummary {
            side: row.get(0)?,
            trade_count: row.get(1)?,
            total_volume: row.get(2)?,
        })
    })?;

    summaries.collect()
}

#[derive(Debug)]
struct DailySummary {
    date: String,
    trade_count: i64,
    total_volume: f64,
    total_pnl: f64,
}

fn get_daily_summary(conn: &Connection) -> Result<Vec<DailySummary>> {
    let mut stmt = conn.prepare(
        "SELECT
            DATE(executed_at) as trade_date,
            COUNT(*) as trade_count,
            SUM(total) as total_volume,
            COALESCE(SUM(pnl), 0) as total_pnl
         FROM trades
         GROUP BY DATE(executed_at)
         ORDER BY trade_date"
    )?;

    let summaries = stmt.query_map([], |row| {
        Ok(DailySummary {
            date: row.get(0)?,
            trade_count: row.get(1)?,
            total_volume: row.get(2)?,
            total_pnl: row.get(3)?,
        })
    })?;

    summaries.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Volume by symbol
    println!("=== Volume by Symbol ===");
    println!("{:<8} {:>6} {:>15} {:>12}", "Symbol", "Trades", "Volume", "Avg Price");
    println!("{}", "-".repeat(45));

    for summary in get_volume_by_symbol(&conn)? {
        println!("{:<8} {:>6} {:>15.2} {:>12.2}",
            summary.symbol,
            summary.trade_count,
            summary.total_volume,
            summary.avg_price);
    }

    // Volume by side
    println!("\n=== Buy vs Sell ===");
    for summary in get_volume_by_side(&conn)? {
        println!("{}: {} trades for ${:.2}",
            summary.side, summary.trade_count, summary.total_volume);
    }

    // Daily statistics
    println!("\n=== Daily Statistics ===");
    for day in get_daily_summary(&conn)? {
        println!("{}: {} trades, ${:.2}, PnL: ${:.2}",
            day.date, day.trade_count, day.total_volume, day.total_pnl);
    }

    Ok(())
}
```

## HAVING — Filtering Groups

```rust
use rusqlite::{Connection, Result, params};

fn get_active_symbols(conn: &Connection, min_trades: i64) -> Result<Vec<SymbolSummary>> {
    let mut stmt = conn.prepare(
        "SELECT
            symbol,
            COUNT(*) as trade_count,
            SUM(total) as total_volume,
            AVG(price) as avg_price
         FROM trades
         GROUP BY symbol
         HAVING COUNT(*) >= ?1
         ORDER BY trade_count DESC"
    )?;

    let summaries = stmt.query_map(params![min_trades], |row| {
        Ok(SymbolSummary {
            symbol: row.get(0)?,
            trade_count: row.get(1)?,
            total_volume: row.get(2)?,
            avg_price: row.get(3)?,
        })
    })?;

    summaries.collect()
}

fn get_high_volume_days(conn: &Connection, min_volume: f64) -> Result<Vec<DailySummary>> {
    let mut stmt = conn.prepare(
        "SELECT
            DATE(executed_at) as trade_date,
            COUNT(*) as trade_count,
            SUM(total) as total_volume,
            COALESCE(SUM(pnl), 0) as total_pnl
         FROM trades
         GROUP BY DATE(executed_at)
         HAVING SUM(total) >= ?1
         ORDER BY total_volume DESC"
    )?;

    let summaries = stmt.query_map(params![min_volume], |row| {
        Ok(DailySummary {
            date: row.get(0)?,
            trade_count: row.get(1)?,
            total_volume: row.get(2)?,
            total_pnl: row.get(3)?,
        })
    })?;

    summaries.collect()
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Active symbols (>= 3 trades)
    println!("Symbols with 3+ trades:");
    for summary in get_active_symbols(&conn, 3)? {
        println!("  {}: {} trades", summary.symbol, summary.trade_count);
    }

    // High volume days
    println!("\nDays with volume > $15,000:");
    for day in get_high_volume_days(&conn, 15000.0)? {
        println!("  {}: ${:.2}", day.date, day.total_volume);
    }

    Ok(())
}
```

## Practical Example: Trade History Analyzer

```rust
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    total: f64,
    executed_at: String,
    pnl: Option<f64>,
}

struct TradeAnalyzer {
    conn: Connection,
}

impl TradeAnalyzer {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(TradeAnalyzer { conn })
    }

    /// Get all trades with filters
    fn get_trades(&self, filters: &TradeFilters) -> Result<Vec<Trade>> {
        let mut sql = String::from("SELECT * FROM trades WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_index = 1;

        if let Some(symbol) = &filters.symbol {
            sql.push_str(&format!(" AND symbol = ?{}", param_index));
            params_vec.push(Box::new(symbol.clone()));
            param_index += 1;
        }

        if let Some(side) = &filters.side {
            sql.push_str(&format!(" AND side = ?{}", param_index));
            params_vec.push(Box::new(side.clone()));
            param_index += 1;
        }

        if let Some(min_total) = filters.min_total {
            sql.push_str(&format!(" AND total >= ?{}", param_index));
            params_vec.push(Box::new(min_total));
            param_index += 1;
        }

        if let Some(start_date) = &filters.start_date {
            sql.push_str(&format!(" AND executed_at >= ?{}", param_index));
            params_vec.push(Box::new(start_date.clone()));
            param_index += 1;
        }

        if let Some(end_date) = &filters.end_date {
            sql.push_str(&format!(" AND executed_at <= ?{}", param_index));
            params_vec.push(Box::new(end_date.clone()));
        }

        sql.push_str(" ORDER BY executed_at DESC");

        if let Some(limit) = filters.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = self.conn.prepare(&sql)?;

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let trades = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Trade {
                id: row.get(0)?,
                symbol: row.get(1)?,
                side: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
                total: row.get(5)?,
                executed_at: row.get(6)?,
                pnl: row.get(7)?,
            })
        })?;

        trades.collect()
    }

    /// Calculate total PnL
    fn calculate_total_pnl(&self) -> Result<f64> {
        self.conn.query_row(
            "SELECT COALESCE(SUM(pnl), 0) FROM trades WHERE pnl IS NOT NULL",
            [],
            |row| row.get(0),
        )
    }

    /// Calculate PnL by symbol
    fn calculate_pnl_by_symbol(&self) -> Result<HashMap<String, f64>> {
        let mut stmt = self.conn.prepare(
            "SELECT symbol, COALESCE(SUM(pnl), 0)
             FROM trades
             WHERE pnl IS NOT NULL
             GROUP BY symbol"
        )?;

        let mut pnl_map = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;

        for row in rows {
            let (symbol, pnl) = row?;
            pnl_map.insert(symbol, pnl);
        }

        Ok(pnl_map)
    }

    /// Find best and worst trades
    fn get_best_and_worst_trades(&self) -> Result<(Option<Trade>, Option<Trade>)> {
        let best = self.conn.query_row(
            "SELECT * FROM trades WHERE pnl IS NOT NULL ORDER BY pnl DESC LIMIT 1",
            [],
            |row| {
                Ok(Trade {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    side: row.get(2)?,
                    price: row.get(3)?,
                    quantity: row.get(4)?,
                    total: row.get(5)?,
                    executed_at: row.get(6)?,
                    pnl: row.get(7)?,
                })
            },
        ).ok();

        let worst = self.conn.query_row(
            "SELECT * FROM trades WHERE pnl IS NOT NULL ORDER BY pnl ASC LIMIT 1",
            [],
            |row| {
                Ok(Trade {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    side: row.get(2)?,
                    price: row.get(3)?,
                    quantity: row.get(4)?,
                    total: row.get(5)?,
                    executed_at: row.get(6)?,
                    pnl: row.get(7)?,
                })
            },
        ).ok();

        Ok((best, worst))
    }

    /// Generate report
    fn generate_report(&self) -> Result<String> {
        let mut report = String::new();

        // Overall statistics
        let total_trades: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM trades", [], |r| r.get(0)
        )?;

        let total_volume: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(total), 0) FROM trades", [], |r| r.get(0)
        )?;

        let total_pnl = self.calculate_total_pnl()?;

        report.push_str("╔════════════════════════════════════════╗\n");
        report.push_str("║        TRADE HISTORY REPORT            ║\n");
        report.push_str("╠════════════════════════════════════════╣\n");
        report.push_str(&format!("║ Total trades:        {:>16} ║\n", total_trades));
        report.push_str(&format!("║ Total volume:     ${:>16.2} ║\n", total_volume));
        report.push_str(&format!("║ Total PnL:        ${:>16.2} ║\n", total_pnl));
        report.push_str("╠════════════════════════════════════════╣\n");

        // PnL by symbol
        report.push_str("║           PnL BY SYMBOL                ║\n");
        report.push_str("╠════════════════════════════════════════╣\n");

        for (symbol, pnl) in self.calculate_pnl_by_symbol()? {
            let sign = if pnl >= 0.0 { "+" } else { "" };
            report.push_str(&format!("║ {:>8}:          ${}{:>13.2} ║\n",
                symbol, sign, pnl));
        }

        report.push_str("╠════════════════════════════════════════╣\n");

        // Best and worst trades
        let (best, worst) = self.get_best_and_worst_trades()?;

        if let Some(trade) = best {
            report.push_str(&format!("║ Best trade: {} +${:.2}            ║\n",
                trade.symbol, trade.pnl.unwrap_or(0.0)));
        }

        if let Some(trade) = worst {
            report.push_str(&format!("║ Worst trade: {} ${:.2}             ║\n",
                trade.symbol, trade.pnl.unwrap_or(0.0)));
        }

        report.push_str("╚════════════════════════════════════════╝\n");

        Ok(report)
    }
}

#[derive(Default)]
struct TradeFilters {
    symbol: Option<String>,
    side: Option<String>,
    min_total: Option<f64>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<usize>,
}

fn main() -> Result<()> {
    let analyzer = TradeAnalyzer::new("trading.db")?;

    // Generate report
    println!("{}", analyzer.generate_report()?);

    // Get trades with filters
    let filters = TradeFilters {
        symbol: Some("BTC".to_string()),
        min_total: Some(10000.0),
        ..Default::default()
    };

    println!("\nLarge BTC trades:");
    for trade in analyzer.get_trades(&filters)? {
        println!("  {} {} {} @ ${:.2} = ${:.2}",
            trade.executed_at, trade.side, trade.quantity, trade.price, trade.total);
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `SELECT *` | Select all columns |
| `SELECT col1, col2` | Select specific columns |
| `WHERE` | Filter rows by condition |
| `ORDER BY` | Sort results |
| `LIMIT`, `OFFSET` | Paginate results |
| `COUNT`, `SUM`, `AVG` | Aggregate functions |
| `GROUP BY` | Group by column |
| `HAVING` | Filter groups |

## Homework

1. **Trade Search**: Implement a `search_trades` function that accepts a struct with optional filters (symbol, side, date_from, date_to, min_price, max_price) and returns matching trades.

2. **Period Statistics**: Create a function that calculates statistics (trade count, volume, PnL) for weekly and monthly periods.

3. **Anomaly Detector**: Write a function that finds trades significantly different from average (e.g., volume more than 3x the average).

4. **CSV Export**: Implement a function to export SELECT results to a CSV file with column headers.

## Navigation

[← Previous day](../217-insert-recording-trade/en.md) | [Next day →](../219-update-order-status/en.md)
