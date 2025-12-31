# Day 135: Date Parsing: chrono Crate

## Trading Analogy

In trading, time is everything. Every candle on the chart has a timestamp. When you load historical data, dates come as strings: `"2024-01-15 09:30:00"`, `"15.01.2024"`, `"Jan 15, 2024"`. To analyze this data — compare trade times, filter by trading sessions, calculate position duration — you need to convert strings into actual date and time objects.

The `chrono` crate is Rust's standard tool for working with dates. It's like a universal time converter in a trading terminal: it understands any format and lets you do anything with dates.

## Adding chrono

Add to `Cargo.toml`:

```toml
[dependencies]
chrono = "0.4"
```

## Core chrono Types

```rust
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime, Utc, Local};

fn main() {
    // NaiveDate — date without timezone
    let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    println!("Trade date: {}", trade_date);

    // NaiveTime — time without timezone
    let trade_time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    println!("Trade time: {}", trade_time);

    // NaiveDateTime — date and time without timezone
    let trade_datetime = NaiveDateTime::new(trade_date, trade_time);
    println!("Date and time: {}", trade_datetime);

    // DateTime<Utc> — date and time with UTC timezone
    let now_utc: DateTime<Utc> = Utc::now();
    println!("Now UTC: {}", now_utc);

    // DateTime<Local> — local time
    let now_local: DateTime<Local> = Local::now();
    println!("Now local: {}", now_local);
}
```

## Parsing Dates from Strings

### Standard Formats

```rust
use chrono::NaiveDate;

fn main() {
    // ISO format (YYYY-MM-DD) — most common in trading data
    let date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
    println!("Date: {}", date);

    // European format
    let date_eu = NaiveDate::parse_from_str("15.01.2024", "%d.%m.%Y").unwrap();
    println!("European format: {}", date_eu);

    // American format
    let date_us = NaiveDate::parse_from_str("01/15/2024", "%m/%d/%Y").unwrap();
    println!("American format: {}", date_us);
}
```

### Parsing Date and Time

```rust
use chrono::NaiveDateTime;

fn main() {
    // Exchange data format
    let dt = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00",
        "%Y-%m-%d %H:%M:%S"
    ).unwrap();
    println!("Trade time: {}", dt);

    // ISO 8601 with T separator
    let dt_iso = NaiveDateTime::parse_from_str(
        "2024-01-15T09:30:00",
        "%Y-%m-%dT%H:%M:%S"
    ).unwrap();
    println!("ISO format: {}", dt_iso);

    // With milliseconds (common in tick data)
    let dt_ms = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00.123",
        "%Y-%m-%d %H:%M:%S%.3f"
    ).unwrap();
    println!("With milliseconds: {}", dt_ms);
}
```

### Format Specifiers

| Specifier | Meaning | Example |
|-----------|---------|---------|
| `%Y` | Year (4 digits) | 2024 |
| `%m` | Month (01-12) | 01 |
| `%d` | Day (01-31) | 15 |
| `%H` | Hour (00-23) | 09 |
| `%M` | Minutes (00-59) | 30 |
| `%S` | Seconds (00-59) | 00 |
| `%f` | Microseconds | 123456 |
| `%.3f` | Milliseconds | .123 |
| `%Y-%m-%d` | ISO date | 2024-01-15 |

## Safe Parsing for Trading Data

```rust
use chrono::NaiveDateTime;

fn parse_trade_time(time_str: &str) -> Option<NaiveDateTime> {
    // Try different formats that may come from exchanges
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%d.%m.%Y %H:%M:%S",
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, format) {
            return Some(dt);
        }
    }
    None
}

fn main() {
    let test_times = [
        "2024-01-15 09:30:00",
        "2024-01-15T09:30:00",
        "2024-01-15 09:30:00.123",
        "15.01.2024 09:30:00",
        "invalid-date",
    ];

    for time_str in test_times {
        match parse_trade_time(time_str) {
            Some(dt) => println!("'{}' -> {}", time_str, dt),
            None => println!("'{}' -> Parse error", time_str),
        }
    }
}
```

## Parsing OHLCV Data with Dates

```rust
use chrono::NaiveDateTime;

struct Candle {
    timestamp: NaiveDateTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn parse_candle(line: &str) -> Option<Candle> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 6 {
        return None;
    }

    let timestamp = NaiveDateTime::parse_from_str(parts[0], "%Y-%m-%d %H:%M:%S").ok()?;
    let open = parts[1].parse().ok()?;
    let high = parts[2].parse().ok()?;
    let low = parts[3].parse().ok()?;
    let close = parts[4].parse().ok()?;
    let volume = parts[5].parse().ok()?;

    Some(Candle {
        timestamp,
        open,
        high,
        low,
        close,
        volume,
    })
}

fn main() {
    let csv_lines = [
        "2024-01-15 09:30:00,42000.0,42150.0,41950.0,42100.0,1500.5",
        "2024-01-15 09:35:00,42100.0,42200.0,42050.0,42180.0,1200.3",
        "2024-01-15 09:40:00,42180.0,42250.0,42100.0,42150.0,980.7",
    ];

    println!("Loaded candles:");
    println!("{:<20} {:>10} {:>10} {:>10} {:>10}",
             "Time", "Open", "High", "Low", "Close");
    println!("{}", "-".repeat(65));

    for line in csv_lines {
        if let Some(candle) = parse_candle(line) {
            println!("{:<20} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
                     candle.timestamp.format("%Y-%m-%d %H:%M"),
                     candle.open, candle.high, candle.low, candle.close);
        }
    }
}
```

## Date Operations

### Comparing Dates

```rust
use chrono::NaiveDateTime;

fn main() {
    let entry_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    let exit_time = NaiveDateTime::parse_from_str(
        "2024-01-15 14:45:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    if exit_time > entry_time {
        println!("Exit after entry — valid trade");
    }

    // Position duration
    let duration = exit_time - entry_time;
    println!("Position held for: {} hours {} minutes",
             duration.num_hours(),
             duration.num_minutes() % 60);
}
```

### Date Arithmetic

```rust
use chrono::{NaiveDateTime, Duration};

fn main() {
    let trade_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Adding time
    let plus_5min = trade_time + Duration::minutes(5);
    let plus_1hour = trade_time + Duration::hours(1);
    let plus_1day = trade_time + Duration::days(1);

    println!("Trade time:  {}", trade_time);
    println!("+ 5 minutes: {}", plus_5min);
    println!("+ 1 hour:    {}", plus_1hour);
    println!("+ 1 day:     {}", plus_1day);

    // Subtracting time
    let yesterday = trade_time - Duration::days(1);
    println!("Yesterday:   {}", yesterday);
}
```

### Extracting Components

```rust
use chrono::{NaiveDateTime, Datelike, Timelike, Weekday};

fn main() {
    let trade_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Date components
    println!("Year: {}", trade_time.year());
    println!("Month: {}", trade_time.month());
    println!("Day: {}", trade_time.day());
    println!("Weekday: {:?}", trade_time.weekday());

    // Time components
    println!("Hour: {}", trade_time.hour());
    println!("Minutes: {}", trade_time.minute());
    println!("Seconds: {}", trade_time.second());

    // Check for trading day (not weekend)
    let weekday = trade_time.weekday();
    let is_trading_day = weekday != Weekday::Sat && weekday != Weekday::Sun;
    println!("Trading day: {}", is_trading_day);
}
```

## Filtering Trades by Time

```rust
use chrono::{NaiveDateTime, NaiveTime, Timelike};

struct Trade {
    time: NaiveDateTime,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn is_during_trading_hours(time: &NaiveDateTime) -> bool {
    let market_open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    let market_close = NaiveTime::from_hms_opt(16, 0, 0).unwrap();

    let trade_time = time.time();
    trade_time >= market_open && trade_time < market_close
}

fn is_first_hour(time: &NaiveDateTime) -> bool {
    let hour = time.hour();
    hour == 9 || (hour == 10 && time.minute() < 30)
}

fn is_last_hour(time: &NaiveDateTime) -> bool {
    let hour = time.hour();
    hour == 15 || (hour == 14 && time.minute() >= 30)
}

fn main() {
    let trades = vec![
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 09:35:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42000.0,
            quantity: 0.5,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42150.0,
            quantity: 0.3,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 15:45:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42200.0,
            quantity: 0.4,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 17:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42100.0,
            quantity: 0.2,
        },
    ];

    println!("Trade time analysis:");
    println!("{}", "-".repeat(60));

    for trade in &trades {
        let during_hours = is_during_trading_hours(&trade.time);
        let first_hour = is_first_hour(&trade.time);
        let last_hour = is_last_hour(&trade.time);

        println!("{} - {} @ ${:.2}",
                 trade.time.format("%H:%M:%S"),
                 trade.symbol,
                 trade.price);
        println!("  During hours: {}, First hour: {}, Last hour: {}",
                 during_hours, first_hour, last_hour);
    }

    // Filter only trades during trading hours
    let valid_trades: Vec<_> = trades
        .iter()
        .filter(|t| is_during_trading_hours(&t.time))
        .collect();

    println!("\nTrades during trading hours: {}", valid_trades.len());
}
```

## Grouping Candles by Period

```rust
use chrono::{NaiveDateTime, Duration, Timelike};

struct Candle {
    timestamp: NaiveDateTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn aggregate_to_hourly(candles: &[Candle]) -> Vec<Candle> {
    if candles.is_empty() {
        return vec![];
    }

    let mut hourly: Vec<Candle> = vec![];
    let mut current_hour = candles[0].timestamp.date().and_hms_opt(
        candles[0].timestamp.hour(), 0, 0
    ).unwrap();

    let mut open = candles[0].open;
    let mut high = candles[0].high;
    let mut low = candles[0].low;
    let mut close = candles[0].close;
    let mut volume = 0.0;

    for candle in candles {
        let candle_hour = candle.timestamp.date().and_hms_opt(
            candle.timestamp.hour(), 0, 0
        ).unwrap();

        if candle_hour != current_hour {
            // Save hourly candle
            hourly.push(Candle {
                timestamp: current_hour,
                open,
                high,
                low,
                close,
                volume,
            });

            // Start new one
            current_hour = candle_hour;
            open = candle.open;
            high = candle.high;
            low = candle.low;
            volume = 0.0;
        }

        high = high.max(candle.high);
        low = low.min(candle.low);
        close = candle.close;
        volume += candle.volume;
    }

    // Last candle
    hourly.push(Candle {
        timestamp: current_hour,
        open,
        high,
        low,
        close,
        volume,
    });

    hourly
}

fn main() {
    let candles_5m = vec![
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42000.0, high: 42050.0, low: 41980.0, close: 42030.0, volume: 100.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:05:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42030.0, high: 42100.0, low: 42020.0, close: 42080.0, volume: 150.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:10:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42080.0, high: 42120.0, low: 42050.0, close: 42100.0, volume: 120.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42100.0, high: 42200.0, low: 42090.0, close: 42180.0, volume: 200.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:05:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42180.0, high: 42250.0, low: 42150.0, close: 42220.0, volume: 180.0,
        },
    ];

    let hourly = aggregate_to_hourly(&candles_5m);

    println!("Hourly candles:");
    for candle in hourly {
        println!("{}: O={:.0} H={:.0} L={:.0} C={:.0} V={:.0}",
                 candle.timestamp.format("%Y-%m-%d %H:00"),
                 candle.open, candle.high, candle.low, candle.close, candle.volume);
    }
}
```

## Formatting Dates for Output

```rust
use chrono::NaiveDateTime;

fn main() {
    let dt = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:45", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Different output formats
    println!("ISO:           {}", dt.format("%Y-%m-%d %H:%M:%S"));
    println!("Compact:       {}", dt.format("%Y%m%d_%H%M%S"));
    println!("Readable:      {}", dt.format("%d %B %Y, %H:%M"));
    println!("Date only:     {}", dt.format("%Y-%m-%d"));
    println!("Time only:     {}", dt.format("%H:%M:%S"));
    println!("For filename:  {}", dt.format("%Y-%m-%d_%H-%M-%S"));
}
```

## Practical Example: Trading Session Analysis

```rust
use chrono::{NaiveDateTime, Duration, Timelike, Weekday, Datelike};

struct Trade {
    timestamp: NaiveDateTime,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
}

struct SessionStats {
    date: String,
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    total_pnl: f64,
    first_trade: String,
    last_trade: String,
    session_duration_mins: i64,
}

fn analyze_session(trades: &[Trade]) -> Option<SessionStats> {
    if trades.is_empty() {
        return None;
    }

    let first = trades.first().unwrap();
    let last = trades.last().unwrap();

    let winning = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing = trades.iter().filter(|t| t.pnl < 0.0).count();
    let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();

    let duration = last.timestamp - first.timestamp;

    Some(SessionStats {
        date: first.timestamp.format("%Y-%m-%d").to_string(),
        total_trades: trades.len(),
        winning_trades: winning,
        losing_trades: losing,
        total_pnl,
        first_trade: first.timestamp.format("%H:%M:%S").to_string(),
        last_trade: last.timestamp.format("%H:%M:%S").to_string(),
        session_duration_mins: duration.num_minutes(),
    })
}

fn main() {
    let trades = vec![
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:32:15", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            pnl: 150.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:45:30", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "ETHUSD".to_string(),
            side: "SELL".to_string(),
            price: 2500.0,
            quantity: 2.0,
            pnl: -50.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 14:20:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "BUY".to_string(),
            price: 42150.0,
            quantity: 0.3,
            pnl: 200.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 15:55:45", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "SELL".to_string(),
            price: 42300.0,
            quantity: 0.4,
            pnl: 75.0,
        },
    ];

    if let Some(stats) = analyze_session(&trades) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         SESSION STATISTICS                ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Date:              {:>20} ║", stats.date);
        println!("║ Total trades:      {:>20} ║", stats.total_trades);
        println!("║ Winning trades:    {:>20} ║", stats.winning_trades);
        println!("║ Losing trades:     {:>20} ║", stats.losing_trades);
        println!("║ Total PnL:         ${:>19.2} ║", stats.total_pnl);
        println!("║ First trade:       {:>20} ║", stats.first_trade);
        println!("║ Last trade:        {:>20} ║", stats.last_trade);
        println!("║ Duration:          {:>17} min ║", stats.session_duration_mins);
        println!("╚═══════════════════════════════════════════╝");
    }
}
```

## What We Learned

| Type | Description | Example |
|------|-------------|---------|
| `NaiveDate` | Date without timezone | `2024-01-15` |
| `NaiveTime` | Time without timezone | `09:30:00` |
| `NaiveDateTime` | Date and time | `2024-01-15 09:30:00` |
| `DateTime<Utc>` | Date/time in UTC | With timezone |
| `Duration` | Time interval | Hours, minutes, seconds |

## Homework

1. Write a function `parse_exchange_timestamp(s: &str) -> Result<NaiveDateTime, String>` that supports at least 5 different date formats from different exchanges

2. Create a function `get_trading_session(time: &NaiveDateTime) -> &str` that returns "pre-market", "regular", "after-hours", or "closed" depending on the time

3. Implement a function `calculate_holding_period(entry: &str, exit: &str) -> Result<Duration, String>` that calculates position holding time

4. Write a program that groups a list of trades by day and outputs statistics for each day

## Navigation

[← Previous day](../134-csv-reading-ohlcv/en.md) | [Next day →](../136-timezones-utc-local/en.md)
