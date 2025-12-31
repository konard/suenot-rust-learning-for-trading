# Day 136: Timezones: UTC and Local Time

## Trading Analogy

Imagine you're trading on multiple exchanges simultaneously: NYSE opens at 9:30 AM New York time, Moscow Exchange at 10:00 AM Moscow time, and crypto exchanges run 24/7 in UTC. To avoid confusion about when each exchange opens, all professional traders use **a single time standard — UTC** (Coordinated Universal Time). It's like a "time language" that everyone understands.

UTC is the "zero meridian" of time. Moscow = UTC+3, New York = UTC-5. When you see a trade timestamp `1704067200`, it's always UTC — universal time that doesn't depend on timezone.

## Why This Matters in Trading

```rust
// Problem without unified time:
// New York trade: 2024-01-01 09:30:00 EST
// Moscow trade:   2024-01-01 17:30:00 MSK
// Which was first? Need to calculate in your head!

// With UTC everything is clear:
// New York trade: 2024-01-01 14:30:00 UTC
// Moscow trade:   2024-01-01 14:30:00 UTC
// They're simultaneous!
```

## Adding chrono

```toml
# Cargo.toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

## UTC Time — Exchange Standard

```rust
use chrono::{Utc, DateTime};

fn main() {
    // Current time in UTC — how all exchanges work
    let now: DateTime<Utc> = Utc::now();
    println!("Current UTC time: {}", now);

    // Example output: 2024-01-15 14:30:45.123456789 UTC

    // Precision matters for trading systems
    println!("Date: {}", now.format("%Y-%m-%d"));
    println!("Time: {}", now.format("%H:%M:%S"));
    println!("Milliseconds: {}", now.format("%H:%M:%S%.3f"));
}
```

## Local Time — For the Trader

```rust
use chrono::{Local, DateTime, Utc};

fn main() {
    // UTC — for storage and APIs
    let utc_now: DateTime<Utc> = Utc::now();

    // Local — for displaying to the trader
    let local_now: DateTime<Local> = Local::now();

    println!("Exchange time (UTC): {}", utc_now.format("%H:%M:%S"));
    println!("Your time:           {}", local_now.format("%H:%M:%S"));

    // Converting UTC -> Local for display
    let trade_time_utc = Utc::now();
    let trade_time_local: DateTime<Local> = trade_time_utc.with_timezone(&Local);

    println!("Trade in UTC:   {}", trade_time_utc);
    println!("Trade locally:  {}", trade_time_local);
}
```

## Working with Specific Timezones

```rust
use chrono::{Utc, DateTime, FixedOffset, TimeZone};

fn main() {
    // Exchange timezones as UTC offsets
    let moscow = FixedOffset::east_opt(3 * 3600).unwrap();      // UTC+3
    let new_york = FixedOffset::west_opt(5 * 3600).unwrap();    // UTC-5
    let tokyo = FixedOffset::east_opt(9 * 3600).unwrap();       // UTC+9

    let utc_now = Utc::now();

    println!("UTC:      {}", utc_now.format("%H:%M:%S"));
    println!("Moscow:   {}", utc_now.with_timezone(&moscow).format("%H:%M:%S"));
    println!("New York: {}", utc_now.with_timezone(&new_york).format("%H:%M:%S"));
    println!("Tokyo:    {}", utc_now.with_timezone(&tokyo).format("%H:%M:%S"));
}
```

## Practical Example: Exchange Trading Hours

```rust
use chrono::{Utc, DateTime, Weekday, Datelike, Timelike, FixedOffset, TimeZone};

struct Exchange {
    name: String,
    timezone: FixedOffset,
    open_hour: u32,
    open_minute: u32,
    close_hour: u32,
    close_minute: u32,
}

impl Exchange {
    fn new(name: &str, utc_offset_hours: i32, open: (u32, u32), close: (u32, u32)) -> Self {
        let offset = if utc_offset_hours >= 0 {
            FixedOffset::east_opt(utc_offset_hours * 3600).unwrap()
        } else {
            FixedOffset::west_opt(-utc_offset_hours * 3600).unwrap()
        };

        Exchange {
            name: name.to_string(),
            timezone: offset,
            open_hour: open.0,
            open_minute: open.1,
            close_hour: close.0,
            close_minute: close.1,
        }
    }

    fn is_open(&self, utc_time: DateTime<Utc>) -> bool {
        let local_time = utc_time.with_timezone(&self.timezone);

        // Check weekends
        match local_time.weekday() {
            Weekday::Sat | Weekday::Sun => return false,
            _ => {}
        }

        let hour = local_time.hour();
        let minute = local_time.minute();
        let current_minutes = hour * 60 + minute;
        let open_minutes = self.open_hour * 60 + self.open_minute;
        let close_minutes = self.close_hour * 60 + self.close_minute;

        current_minutes >= open_minutes && current_minutes < close_minutes
    }

    fn time_until_open(&self, utc_time: DateTime<Utc>) -> Option<String> {
        if self.is_open(utc_time) {
            return None;
        }

        let local_time = utc_time.with_timezone(&self.timezone);
        let current_minutes = local_time.hour() * 60 + local_time.minute();
        let open_minutes = self.open_hour * 60 + self.open_minute;

        if current_minutes < open_minutes {
            let diff = open_minutes - current_minutes;
            Some(format!("{}h {}min", diff / 60, diff % 60))
        } else {
            Some("Tomorrow".to_string())
        }
    }
}

fn main() {
    let exchanges = vec![
        Exchange::new("NYSE", -5, (9, 30), (16, 0)),
        Exchange::new("MOEX", 3, (10, 0), (18, 50)),
        Exchange::new("TSE", 9, (9, 0), (15, 0)),
    ];

    let now = Utc::now();
    println!("Current UTC time: {}\n", now.format("%Y-%m-%d %H:%M:%S"));

    for exchange in &exchanges {
        let status = if exchange.is_open(now) {
            "OPEN".to_string()
        } else {
            match exchange.time_until_open(now) {
                Some(time) => format!("Closed (opens in {})", time),
                None => "Closed".to_string(),
            }
        };
        println!("{}: {}", exchange.name, status);
    }
}
```

## Parsing Time with Timezone from API

```rust
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime, TimeZone};

fn main() {
    // Different time formats from exchanges

    // 1. ISO 8601 with timezone (common format)
    let iso_time = "2024-01-15T14:30:00+03:00";
    let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(iso_time).unwrap();
    let utc_time: DateTime<Utc> = parsed.with_timezone(&Utc);
    println!("ISO -> UTC: {}", utc_time);

    // 2. Time without timezone (need to know which zone it's in)
    let naive_time = "2024-01-15 14:30:00";
    let naive: NaiveDateTime = NaiveDateTime::parse_from_str(naive_time, "%Y-%m-%d %H:%M:%S").unwrap();

    // Assume this is Moscow Exchange time (UTC+3)
    let moscow = FixedOffset::east_opt(3 * 3600).unwrap();
    let moscow_time = moscow.from_local_datetime(&naive).unwrap();
    let utc_from_moscow: DateTime<Utc> = moscow_time.with_timezone(&Utc);
    println!("Moscow -> UTC: {}", utc_from_moscow);

    // 3. Time with Z suffix (already UTC)
    let z_time = "2024-01-15T14:30:00Z";
    let parsed_z: DateTime<Utc> = z_time.parse().unwrap();
    println!("Z-format: {}", parsed_z);
}
```

## Storing Trades in UTC

```rust
use chrono::{DateTime, Utc, Local};

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,  // Always store in UTC!
}

impl Trade {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        Trade {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }

    fn display_local_time(&self) -> String {
        let local: DateTime<Local> = self.timestamp.with_timezone(&Local);
        local.format("%Y-%m-%d %H:%M:%S %Z").to_string()
    }

    fn display_utc_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

fn main() {
    let trade = Trade::new(1, "BTC/USDT", 42500.0, 0.5);

    println!("Trade #{}", trade.id);
    println!("  Symbol: {}", trade.symbol);
    println!("  Price: ${:.2}", trade.price);
    println!("  Volume: {}", trade.quantity);
    println!("  Time (UTC):   {}", trade.display_utc_time());
    println!("  Time (local): {}", trade.display_local_time());
}
```

## Comparing Trade Times

```rust
use chrono::{DateTime, Utc, Duration};

fn main() {
    let trade1_time: DateTime<Utc> = "2024-01-15T14:30:00Z".parse().unwrap();
    let trade2_time: DateTime<Utc> = "2024-01-15T14:30:05Z".parse().unwrap();

    // Comparison
    if trade1_time < trade2_time {
        println!("Trade 1 was before trade 2");
    }

    // Time difference
    let diff: Duration = trade2_time - trade1_time;
    println!("Time between trades: {} seconds", diff.num_seconds());

    // Check: was the trade recent?
    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);

    if trade1_time > one_hour_ago {
        println!("Trade was within the last hour");
    } else {
        println!("Trade was more than an hour ago");
    }
}
```

## Filtering Trades by Time

```rust
use chrono::{DateTime, Utc, Duration, Timelike};

struct Trade {
    symbol: String,
    price: f64,
    timestamp: DateTime<Utc>,
}

fn filter_by_hour(trades: &[Trade], start_hour: u32, end_hour: u32) -> Vec<&Trade> {
    trades
        .iter()
        .filter(|t| {
            let hour = t.timestamp.hour();
            hour >= start_hour && hour < end_hour
        })
        .collect()
}

fn filter_last_n_minutes(trades: &[Trade], minutes: i64) -> Vec<&Trade> {
    let cutoff = Utc::now() - Duration::minutes(minutes);
    trades
        .iter()
        .filter(|t| t.timestamp > cutoff)
        .collect()
}

fn main() {
    let now = Utc::now();

    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            price: 42000.0,
            timestamp: now - Duration::minutes(5)
        },
        Trade {
            symbol: "ETH".to_string(),
            price: 2500.0,
            timestamp: now - Duration::minutes(30)
        },
        Trade {
            symbol: "BTC".to_string(),
            price: 42100.0,
            timestamp: now - Duration::minutes(120)
        },
    ];

    // Trades in the last 15 minutes
    let recent = filter_last_n_minutes(&trades, 15);
    println!("Trades in the last 15 minutes: {}", recent.len());

    for trade in recent {
        println!("  {} @ ${:.2}", trade.symbol, trade.price);
    }
}
```

## Working with Trading Sessions

```rust
use chrono::{DateTime, Utc, FixedOffset, TimeZone, Timelike, Datelike, Weekday};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TradingSession {
    Asian,      // Tokyo: 00:00-09:00 UTC
    European,   // London: 08:00-17:00 UTC
    American,   // New York: 13:00-22:00 UTC
    Overlap,    // Session overlap
    Weekend,    // Weekends
}

fn get_current_session(utc_time: DateTime<Utc>) -> TradingSession {
    match utc_time.weekday() {
        Weekday::Sat | Weekday::Sun => return TradingSession::Weekend,
        _ => {}
    }

    let hour = utc_time.hour();

    match hour {
        // Asia-Europe overlap
        8 => TradingSession::Overlap,
        // Europe-America overlap
        13..=16 => TradingSession::Overlap,
        // Asia only
        0..=7 => TradingSession::Asian,
        // Europe only
        9..=12 => TradingSession::European,
        // America only
        17..=21 => TradingSession::American,
        // Night / transition
        _ => TradingSession::Asian,
    }
}

fn get_session_volatility_multiplier(session: TradingSession) -> f64 {
    match session {
        TradingSession::Overlap => 1.5,   // High volatility
        TradingSession::American => 1.2,  // Elevated
        TradingSession::European => 1.1,  // Moderate
        TradingSession::Asian => 0.8,     // Low
        TradingSession::Weekend => 0.5,   // Minimal (crypto)
    }
}

fn main() {
    let now = Utc::now();
    let session = get_current_session(now);
    let volatility = get_session_volatility_multiplier(session);

    println!("Current UTC time: {}", now.format("%H:%M"));
    println!("Trading session: {:?}", session);
    println!("Volatility multiplier: {:.1}x", volatility);

    // Demo for different hours
    println!("\nSessions throughout the day:");
    for hour in [3, 8, 10, 14, 18, 23] {
        let test_time = Utc::now()
            .with_hour(hour).unwrap()
            .with_minute(0).unwrap();
        let s = get_current_session(test_time);
        println!("  {:02}:00 UTC -> {:?}", hour, s);
    }
}
```

## Converting Time for Reports

```rust
use chrono::{DateTime, Utc, Local, FixedOffset, TimeZone};

struct TradeReport {
    trades: Vec<(DateTime<Utc>, f64)>, // (time, pnl)
}

impl TradeReport {
    fn new() -> Self {
        TradeReport { trades: Vec::new() }
    }

    fn add_trade(&mut self, time: DateTime<Utc>, pnl: f64) {
        self.trades.push((time, pnl));
    }

    fn print_for_timezone(&self, name: &str, offset_hours: i32) {
        let tz = if offset_hours >= 0 {
            FixedOffset::east_opt(offset_hours * 3600).unwrap()
        } else {
            FixedOffset::west_opt(-offset_hours * 3600).unwrap()
        };

        println!("\n=== Report for {} (UTC{:+}) ===", name, offset_hours);

        let mut total = 0.0;
        for (utc_time, pnl) in &self.trades {
            let local_time = utc_time.with_timezone(&tz);
            total += pnl;
            let sign = if *pnl >= 0.0 { "+" } else { "" };
            println!(
                "{} | PnL: {}${:.2}",
                local_time.format("%Y-%m-%d %H:%M:%S"),
                sign,
                pnl
            );
        }
        println!("Total: ${:.2}", total);
    }
}

fn main() {
    let mut report = TradeReport::new();

    // Add trades (time in UTC)
    let base_time: DateTime<Utc> = "2024-01-15T14:30:00Z".parse().unwrap();
    report.add_trade(base_time, 150.0);
    report.add_trade(base_time + chrono::Duration::hours(1), -50.0);
    report.add_trade(base_time + chrono::Duration::hours(3), 200.0);

    // Print for different timezones
    report.print_for_timezone("Moscow", 3);
    report.print_for_timezone("New York", -5);
    report.print_for_timezone("London", 0);
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|-------------------|
| `Utc` | Universal time | Storage, APIs, comparison |
| `Local` | Local time | Display to user |
| `FixedOffset` | Specific timezone | Exchanges in different countries |
| `with_timezone()` | Conversion | Reports, UI |
| Time comparison | `<`, `>`, `-` | Sorting trades |

## Rules for Trading Systems

1. **Store in UTC** — all timestamps in the database should be UTC only
2. **Receive in UTC** — request time in UTC from APIs
3. **Convert for display** — show the user in their timezone
4. **Compare in UTC** — all time calculations only in UTC
5. **Log in UTC** — logs always with UTC timestamp

## Homework

1. Write a function `is_market_open(exchange: &str, utc_time: DateTime<Utc>) -> bool` that checks if a specific exchange is open (support NYSE, NASDAQ, MOEX, LSE)

2. Create a struct `OrderWithTimezone` that stores creation time in UTC but can display it in any timezone

3. Implement a function `group_trades_by_session(trades: &[Trade]) -> HashMap<TradingSession, Vec<&Trade>>` that groups trades by trading session

4. Write a function to calculate statistics: how many trades were in each trading session over the last month

## Navigation

[← Previous day](../135-date-parsing-chrono/en.md) | [Next day →](../137-timestamp-unix-time/en.md)
