# Day 139: Time Formatting

## Trading Analogy

In trading, time is money, and time formatting is like different ways of recording trade timestamps. Imagine: you have a moment when a position was opened. You can write it as `2024-03-15 14:30:00`, as `March 15, 2024 at 2:30 PM`, or as `1710510600` (Unix timestamp). Different situations require different formats: one format for the exchange API, another for client reports, and a third for the database.

## Basic Formatting with chrono

```rust
use chrono::{DateTime, Utc, Local, NaiveDateTime};

fn main() {
    // Current UTC time
    let now_utc: DateTime<Utc> = Utc::now();

    // Standard RFC 3339 format (ISO 8601)
    println!("RFC 3339: {}", now_utc.to_rfc3339());
    // Output: 2024-03-15T14:30:00+00:00

    // RFC 2822 (email format)
    println!("RFC 2822: {}", now_utc.to_rfc2822());
    // Output: Fri, 15 Mar 2024 14:30:00 +0000

    // Simple output (Display trait)
    println!("Display: {}", now_utc);
    // Output: 2024-03-15 14:30:00 UTC
}
```

## Custom Formats with format()

```rust
use chrono::{DateTime, Utc, Local};

fn main() {
    let trade_time: DateTime<Utc> = Utc::now();

    // Trade date for report
    println!("Trade date: {}", trade_time.format("%Y-%m-%d"));
    // Output: 2024-03-15

    // Trade time
    println!("Trade time: {}", trade_time.format("%H:%M:%S"));
    // Output: 14:30:00

    // Full format for logging
    println!("Log: {}", trade_time.format("%Y-%m-%d %H:%M:%S%.3f"));
    // Output: 2024-03-15 14:30:00.123

    // Format for client report
    println!("Report: {}", trade_time.format("%B %d, %Y at %H:%M"));
    // Output: March 15, 2024 at 14:30
}
```

## Main Format Specifiers

| Specifier | Description | Example |
|-----------|-------------|---------|
| `%Y` | Year (4 digits) | 2024 |
| `%y` | Year (2 digits) | 24 |
| `%m` | Month (01-12) | 03 |
| `%d` | Day (01-31) | 15 |
| `%H` | Hour 24-hour (00-23) | 14 |
| `%I` | Hour 12-hour (01-12) | 02 |
| `%M` | Minutes (00-59) | 30 |
| `%S` | Seconds (00-59) | 45 |
| `%p` | AM/PM | PM |
| `%A` | Weekday full | Friday |
| `%a` | Weekday short | Fri |
| `%B` | Month full | March |
| `%b` | Month short | Mar |
| `%.3f` | Milliseconds | .123 |
| `%.6f` | Microseconds | .123456 |
| `%z` | Timezone offset +HHMM | +0000 |
| `%Z` | Timezone name | UTC |

## Formatting for Different Trading Contexts

```rust
use chrono::{DateTime, Utc, TimeZone};

fn main() {
    let trade_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

    // Format for exchange API (ISO 8601)
    let api_format = trade_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    println!("API: {}", api_format);
    // Output: 2024-03-15T14:30:00Z

    // Format for backup filename
    let filename = format!("trades_{}.csv", trade_time.format("%Y%m%d_%H%M%S"));
    println!("File: {}", filename);
    // Output: trades_20240315_143000.csv

    // Format for client report
    let report = trade_time.format("%d %B %Y, %H:%M:%S").to_string();
    println!("Report: {}", report);
    // Output: 15 March 2024, 14:30:00

    // Short format for UI
    let ui = trade_time.format("%d.%m %H:%M").to_string();
    println!("UI: {}", ui);
    // Output: 15.03 14:30
}
```

## Formatting Trade Logs

```rust
use chrono::{DateTime, Utc, TimeZone};

struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    executed_at: DateTime<Utc>,
}

impl Trade {
    fn format_for_log(&self) -> String {
        format!(
            "[{}] {} {} {} @ {:.2} x {}",
            self.executed_at.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.symbol,
            self.side,
            self.price,
            self.price,
            self.quantity
        )
    }

    fn format_for_report(&self) -> String {
        format!(
            "Trade: {} {:.4} {} at ${:.2}\nTime: {}",
            self.side,
            self.quantity,
            self.symbol,
            self.price,
            self.executed_at.format("%B %d, %Y %H:%M:%S")
        )
    }

    fn format_short(&self) -> String {
        format!(
            "{} {} {} @ ${:.2}",
            self.executed_at.format("%H:%M:%S"),
            self.side,
            self.symbol,
            self.price
        )
    }
}

fn main() {
    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42500.50,
        quantity: 0.5,
        executed_at: Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
    };

    println!("Log:\n{}\n", trade.format_for_log());
    println!("Report:\n{}\n", trade.format_for_report());
    println!("Short:\n{}", trade.format_short());
}
```

## Parsing Time from String

```rust
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};

fn main() {
    // Parsing RFC 3339
    let time_str = "2024-03-15T14:30:00Z";
    let parsed: DateTime<Utc> = DateTime::parse_from_rfc3339(time_str)
        .expect("Invalid RFC 3339 format")
        .with_timezone(&Utc);
    println!("Parsed RFC 3339: {}", parsed);

    // Parsing custom format
    let custom_str = "03/15/2024 14:30:00";
    let naive = NaiveDateTime::parse_from_str(custom_str, "%m/%d/%Y %H:%M:%S")
        .expect("Invalid date format");
    let datetime = Utc.from_utc_datetime(&naive);
    println!("Parsed custom: {}", datetime);

    // Parsing Binance format
    let binance_time = "2024-03-15 14:30:00";
    let binance_parsed = NaiveDateTime::parse_from_str(binance_time, "%Y-%m-%d %H:%M:%S")
        .expect("Invalid Binance format");
    println!("Binance: {}", Utc.from_utc_datetime(&binance_parsed));
}
```

## Formatting Relative Time

```rust
use chrono::{DateTime, Utc, Duration, TimeZone};

fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(time);

    if diff < Duration::seconds(60) {
        format!("{} sec ago", diff.num_seconds())
    } else if diff < Duration::minutes(60) {
        format!("{} min ago", diff.num_minutes())
    } else if diff < Duration::hours(24) {
        format!("{} hours ago", diff.num_hours())
    } else if diff < Duration::days(7) {
        format!("{} days ago", diff.num_days())
    } else {
        time.format("%Y-%m-%d").to_string()
    }
}

fn main() {
    let now = Utc::now();

    // Trade 30 seconds ago
    let recent = now - Duration::seconds(30);
    println!("Recent trade: {}", format_relative_time(recent));

    // Trade 2 hours ago
    let earlier = now - Duration::hours(2);
    println!("Earlier: {}", format_relative_time(earlier));

    // Trade from last week
    let old = now - Duration::days(10);
    println!("Old: {}", format_relative_time(old));
}
```

## Formatting OHLCV Candles

```rust
use chrono::{DateTime, Utc, TimeZone, Duration};

struct Candle {
    open_time: DateTime<Utc>,
    close_time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn timeframe_label(&self) -> String {
        let duration = self.close_time.signed_duration_since(self.open_time);

        if duration <= Duration::minutes(1) {
            "1m".to_string()
        } else if duration <= Duration::minutes(5) {
            "5m".to_string()
        } else if duration <= Duration::minutes(15) {
            "15m".to_string()
        } else if duration <= Duration::hours(1) {
            "1h".to_string()
        } else if duration <= Duration::hours(4) {
            "4h".to_string()
        } else if duration <= Duration::days(1) {
            "1d".to_string()
        } else {
            "1w".to_string()
        }
    }

    fn format_for_chart(&self) -> String {
        format!(
            "[{}] {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2}",
            self.timeframe_label(),
            self.open_time.format("%Y-%m-%d %H:%M"),
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume
        )
    }
}

fn main() {
    let candle = Candle {
        open_time: Utc.with_ymd_and_hms(2024, 3, 15, 14, 0, 0).unwrap(),
        close_time: Utc.with_ymd_and_hms(2024, 3, 15, 15, 0, 0).unwrap(),
        open: 42000.0,
        high: 42500.0,
        low: 41800.0,
        close: 42300.0,
        volume: 1500.5,
    };

    println!("{}", candle.format_for_chart());
}
```

## Formatting Trading Sessions

```rust
use chrono::{DateTime, Utc, Weekday, Timelike, Datelike, TimeZone};

struct TradingSession {
    name: String,
    open_hour: u32,
    close_hour: u32,
}

fn format_session_time(time: DateTime<Utc>, session: &TradingSession) -> String {
    let hour = time.hour();
    let is_open = hour >= session.open_hour && hour < session.close_hour;

    let status = if is_open { "OPEN" } else { "CLOSED" };

    format!(
        "{}: {} ({}:00 - {}:00 UTC) | Current time: {}",
        session.name,
        status,
        session.open_hour,
        session.close_hour,
        time.format("%H:%M:%S")
    )
}

fn main() {
    let now = Utc::now();

    let sessions = vec![
        TradingSession {
            name: "Tokyo".to_string(),
            open_hour: 0,  // 00:00 UTC = 09:00 JST
            close_hour: 9,
        },
        TradingSession {
            name: "London".to_string(),
            open_hour: 8,
            close_hour: 17,
        },
        TradingSession {
            name: "New York".to_string(),
            open_hour: 13,
            close_hour: 22,
        },
    ];

    println!("Trading session status:\n");
    for session in &sessions {
        println!("{}", format_session_time(now, session));
    }
}
```

## Locale-Aware Formatting

```rust
use chrono::{DateTime, Utc, TimeZone};

fn format_localized(time: DateTime<Utc>, locale: &str) -> String {
    match locale {
        "en" => time.format("%B %d, %Y at %I:%M:%S %p").to_string(),
        "de" => {
            let months = [
                "Januar", "Februar", "März", "April", "Mai", "Juni",
                "Juli", "August", "September", "Oktober", "November", "Dezember"
            ];
            let month_idx = time.month0() as usize;
            format!(
                "{}. {} {} um {}",
                time.day(),
                months[month_idx],
                time.year(),
                time.format("%H:%M:%S")
            )
        }
        _ => time.to_rfc3339(),
    }
}

fn main() {
    let trade_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

    println!("English: {}", format_localized(trade_time, "en"));
    println!("German: {}", format_localized(trade_time, "de"));
    println!("Default: {}", format_localized(trade_time, "other"));
}
```

## Practical Example: Trade Report Generator

```rust
use chrono::{DateTime, Utc, TimeZone, Duration};

struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
    executed_at: DateTime<Utc>,
}

struct TradeReport {
    trades: Vec<Trade>,
    generated_at: DateTime<Utc>,
}

impl TradeReport {
    fn new(trades: Vec<Trade>) -> Self {
        TradeReport {
            trades,
            generated_at: Utc::now(),
        }
    }

    fn generate_header(&self) -> String {
        format!(
            "╔══════════════════════════════════════════════════════════════╗\n\
             ║                       TRADE REPORT                           ║\n\
             ║  Generated: {:^47} ║\n\
             ╠══════════════════════════════════════════════════════════════╣",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    fn format_trade(&self, trade: &Trade) -> String {
        let pnl_sign = if trade.pnl >= 0.0 { "+" } else { "" };
        format!(
            "║ #{:04} | {} | {:>8} {:>10} | {:.4} @ ${:>10.2} | {}${:>8.2} ║",
            trade.id,
            trade.executed_at.format("%m/%d %H:%M"),
            trade.side,
            trade.symbol,
            trade.quantity,
            trade.price,
            pnl_sign,
            trade.pnl
        )
    }

    fn generate_summary(&self) -> String {
        let total_pnl: f64 = self.trades.iter().map(|t| t.pnl).sum();
        let winning_trades = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (winning_trades as f64 / self.trades.len() as f64) * 100.0;

        format!(
            "╠══════════════════════════════════════════════════════════════╣\n\
             ║ Total trades: {:>3}    Win Rate: {:>5.1}%    PnL: ${:>12.2} ║\n\
             ╚══════════════════════════════════════════════════════════════╝",
            self.trades.len(),
            win_rate,
            total_pnl
        )
    }

    fn generate(&self) -> String {
        let mut report = self.generate_header();
        report.push('\n');

        for trade in &self.trades {
            report.push_str(&self.format_trade(trade));
            report.push('\n');
        }

        report.push_str(&self.generate_summary());
        report
    }
}

fn main() {
    let base_time = Utc.with_ymd_and_hms(2024, 3, 15, 10, 0, 0).unwrap();

    let trades = vec![
        Trade {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            pnl: 250.0,
            executed_at: base_time,
        },
        Trade {
            id: 2,
            symbol: "ETH/USDT".to_string(),
            side: "SELL".to_string(),
            price: 2800.0,
            quantity: 2.0,
            pnl: -50.0,
            executed_at: base_time + Duration::hours(2),
        },
        Trade {
            id: 3,
            symbol: "BTC/USDT".to_string(),
            side: "SELL".to_string(),
            price: 42500.0,
            quantity: 0.5,
            pnl: 250.0,
            executed_at: base_time + Duration::hours(4),
        },
    ];

    let report = TradeReport::new(trades);
    println!("{}", report.generate());
}
```

## What We Learned

| Method | Description | Example Output |
|--------|-------------|----------------|
| `to_rfc3339()` | ISO 8601 standard | 2024-03-15T14:30:00+00:00 |
| `to_rfc2822()` | Email format | Fri, 15 Mar 2024 14:30:00 +0000 |
| `format("%...")` | Custom format | Any |
| `parse_from_rfc3339()` | Parse ISO 8601 | DateTime |
| `parse_from_str()` | Parse custom format | NaiveDateTime |

## Homework

1. Write a function `format_trade_time(time: DateTime<Utc>, format_type: &str) -> String` that supports formats: "api", "log", "report", "short"

2. Create an `OrderExecution` struct with timestamps for order creation, submission, and execution. Add a method to format all stages

3. Implement a function `parse_exchange_time(time_str: &str, exchange: &str) -> Result<DateTime<Utc>, String>` that parses time in different exchange formats (Binance, Coinbase, Kraken)

4. Write a daily report generator that groups trades by hour and formats output for terminal display

## Navigation

[← Previous day](../138-duration-time-between-trades/en.md) | [Next day →](../140-toml-trading-bot-config/en.md)
