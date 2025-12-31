# Day 137: Timestamp — Unix Time for Exchanges

## Trading Analogy

Every trade on an exchange has an **exact timestamp**. When you look at order history, you see: "2024-01-15 14:32:05.123". But for computers and exchanges, it's more convenient to store time as **a single number** — the number of seconds (or milliseconds) since January 1, 1970.

Imagine that instead of the date "January 15, 2024, 14:32:05", the exchange records **1705329125**. This is the **Unix timestamp** — a universal way of recording time that:
- Works the same across all time zones
- Is easy to compare (larger = later)
- Is convenient to store in databases
- Is used by all major exchanges (Binance, Bybit, Kraken)

**Timestamp is like a unique receipt number on an exchange**, except instead of a serial number, time is used.

## What is Unix Timestamp?

```rust
fn main() {
    // Unix timestamp — seconds since January 1, 1970 (UTC)
    let timestamp: i64 = 1705329125;

    println!("Timestamp: {}", timestamp);
    println!("This is approximately: January 15, 2024, 14:32:05 UTC");

    // Exchanges often use milliseconds
    let timestamp_ms: i64 = 1705329125123;
    println!("In milliseconds: {}", timestamp_ms);
}
```

## Getting Current Time

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Current Unix timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let timestamp_secs = now.as_secs();
    let timestamp_millis = now.as_millis();

    println!("Seconds since 1970: {}", timestamp_secs);
    println!("Milliseconds since 1970: {}", timestamp_millis);
}
```

## Timestamp in Trading Operations

### Recording a Trade

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct Trade {
    symbol: String,
    side: String,       // "BUY" or "SELL"
    price: f64,
    quantity: f64,
    timestamp: i64,     // Unix timestamp in milliseconds
}

fn create_trade(symbol: &str, side: &str, price: f64, quantity: f64) -> Trade {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    Trade {
        symbol: symbol.to_string(),
        side: side.to_string(),
        price,
        quantity,
        timestamp,
    }
}

fn main() {
    let trade = create_trade("BTC/USDT", "BUY", 42500.0, 0.5);

    println!("Trade:");
    println!("  Symbol: {}", trade.symbol);
    println!("  Side: {}", trade.side);
    println!("  Price: ${:.2}", trade.price);
    println!("  Quantity: {}", trade.quantity);
    println!("  Timestamp: {}", trade.timestamp);
}
```

### Candlestick (OHLCV)

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct Candle {
    open_time: i64,     // Candle open time
    close_time: i64,    // Candle close time
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn create_1h_candle(open_time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Candle {
    // Hourly candle lasts 3600 seconds (1 hour)
    let close_time = open_time + 3600 * 1000 - 1; // in milliseconds, -1 to avoid overlap

    Candle {
        open_time,
        close_time,
        open,
        high,
        low,
        close,
        volume,
    }
}

fn main() {
    // Start of hour: January 15, 2024, 14:00:00 UTC
    let open_time: i64 = 1705327200000; // milliseconds

    let candle = create_1h_candle(
        open_time,
        42000.0,  // open
        42500.0,  // high
        41800.0,  // low
        42300.0,  // close
        1250.5    // volume
    );

    println!("1H Candle:");
    println!("  Open: {} ({})", candle.open_time, format_timestamp(candle.open_time));
    println!("  Close: {} ({})", candle.close_time, format_timestamp(candle.close_time));
    println!("  OHLC: {}/{}/{}/{}", candle.open, candle.high, candle.low, candle.close);
    println!("  Volume: {:.2}", candle.volume);
}

fn format_timestamp(ts: i64) -> String {
    // Simple formatting for demonstration
    format!("{}ms from UNIX epoch", ts)
}
```

## Comparison and Sorting by Time

```rust
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    timestamp: i64,
}

fn main() {
    let mut orders = vec![
        Order { id: 1, price: 42000.0, quantity: 0.5, timestamp: 1705329125000 },
        Order { id: 2, price: 42100.0, quantity: 0.3, timestamp: 1705329120000 },
        Order { id: 3, price: 41900.0, quantity: 0.7, timestamp: 1705329130000 },
    ];

    // Sort by time (oldest to newest)
    orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    println!("Orders by creation time:");
    for order in &orders {
        println!("  ID: {}, Price: {}, Time: {}", order.id, order.price, order.timestamp);
    }

    // Find the newest order
    if let Some(newest) = orders.iter().max_by_key(|o| o.timestamp) {
        println!("\nNewest order: ID {}", newest.id);
    }

    // Find orders from the last 5 seconds
    let five_seconds_ago = 1705329130000 - 5000;
    let recent: Vec<&Order> = orders
        .iter()
        .filter(|o| o.timestamp >= five_seconds_ago)
        .collect();

    println!("\nOrders from last 5 seconds: {}", recent.len());
}
```

## Calculating Order Execution Time

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct OrderExecution {
    order_id: u64,
    created_at: i64,
    executed_at: i64,
}

impl OrderExecution {
    fn execution_time_ms(&self) -> i64 {
        self.executed_at - self.created_at
    }

    fn execution_time_secs(&self) -> f64 {
        self.execution_time_ms() as f64 / 1000.0
    }
}

fn main() {
    let execution = OrderExecution {
        order_id: 12345,
        created_at: 1705329125000,
        executed_at: 1705329125150, // executed after 150ms
    };

    println!("Order #{}", execution.order_id);
    println!("  Execution time: {}ms", execution.execution_time_ms());
    println!("  Execution time: {:.3}s", execution.execution_time_secs());

    // Check for slow execution
    if execution.execution_time_ms() > 100 {
        println!("  WARNING: Slow execution!");
    } else {
        println!("  Fast execution");
    }
}
```

## Candle Intervals

```rust
const MINUTE: i64 = 60 * 1000;        // 1 minute in milliseconds
const HOUR: i64 = 60 * MINUTE;        // 1 hour
const DAY: i64 = 24 * HOUR;           // 1 day
const WEEK: i64 = 7 * DAY;            // 1 week

fn get_candle_start(timestamp: i64, interval: i64) -> i64 {
    // Round down to interval start
    (timestamp / interval) * interval
}

fn get_candle_end(timestamp: i64, interval: i64) -> i64 {
    get_candle_start(timestamp, interval) + interval - 1
}

fn main() {
    let now: i64 = 1705329125500; // Example current time

    println!("Current timestamp: {}", now);
    println!();

    // 1-minute candle
    let m1_start = get_candle_start(now, MINUTE);
    let m1_end = get_candle_end(now, MINUTE);
    println!("1m candle: {} - {}", m1_start, m1_end);

    // 1-hour candle
    let h1_start = get_candle_start(now, HOUR);
    let h1_end = get_candle_end(now, HOUR);
    println!("1h candle: {} - {}", h1_start, h1_end);

    // Daily candle
    let d1_start = get_candle_start(now, DAY);
    let d1_end = get_candle_end(now, DAY);
    println!("1d candle: {} - {}", d1_start, d1_end);
}
```

## Working with Exchange APIs

```rust
// Binance uses milliseconds
fn binance_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// Some exchanges use seconds
fn kraken_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// Conversion between formats
fn millis_to_secs(millis: i64) -> i64 {
    millis / 1000
}

fn secs_to_millis(secs: i64) -> i64 {
    secs * 1000
}

fn main() {
    let binance_ts = 1705329125123_i64; // milliseconds
    let kraken_ts = 1705329125_i64;     // seconds

    // Normalize to common format (milliseconds)
    let normalized_binance = binance_ts;
    let normalized_kraken = secs_to_millis(kraken_ts);

    println!("Binance: {} ms", normalized_binance);
    println!("Kraken:  {} ms", normalized_kraken);

    // Time difference between exchanges
    let diff = (normalized_binance - normalized_kraken).abs();
    println!("Difference: {} ms", diff);
}
```

## Checking for Stale Data

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct MarketData {
    symbol: String,
    price: f64,
    timestamp: i64,
}

fn is_stale(data: &MarketData, max_age_ms: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    now - data.timestamp > max_age_ms
}

fn main() {
    let data = MarketData {
        symbol: "BTC/USDT".to_string(),
        price: 42500.0,
        timestamp: 1705329125000,
    };

    // Check: is data older than 1 second?
    let max_age = 1000; // 1 second

    // For demonstration, compare with fixed time
    let now = 1705329127000_i64; // 2 seconds later
    let age = now - data.timestamp;

    println!("Data age: {}ms", age);

    if age > max_age {
        println!("WARNING: Data is stale! Do not use for trading.");
    } else {
        println!("Data is fresh.");
    }
}
```

## Timestamp in Logs and Debugging

```rust
use std::time::{SystemTime, UNIX_EPOCH};

enum LogLevel {
    Info,
    Warning,
    Error,
}

struct LogEntry {
    timestamp: i64,
    level: LogLevel,
    message: String,
}

fn log(level: LogLevel, message: &str) -> LogEntry {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    LogEntry {
        timestamp,
        level,
        message: message.to_string(),
    }
}

fn print_log(entry: &LogEntry) {
    let level_str = match entry.level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
    };

    println!("[{}] [{}] {}", entry.timestamp, level_str, entry.message);
}

fn main() {
    let logs = vec![
        LogEntry {
            timestamp: 1705329125000,
            level: LogLevel::Info,
            message: "Connected to exchange".to_string()
        },
        LogEntry {
            timestamp: 1705329125050,
            level: LogLevel::Info,
            message: "Received BTC/USDT data".to_string()
        },
        LogEntry {
            timestamp: 1705329125100,
            level: LogLevel::Warning,
            message: "High latency: 85ms".to_string()
        },
        LogEntry {
            timestamp: 1705329125500,
            level: LogLevel::Error,
            message: "Order rejected: insufficient funds".to_string()
        },
    ];

    println!("=== Trading System Log ===");
    for entry in &logs {
        print_log(entry);
    }

    // Analysis: how much time between first and last event?
    if logs.len() >= 2 {
        let duration = logs.last().unwrap().timestamp - logs.first().unwrap().timestamp;
        println!("\nTotal runtime: {}ms", duration);
    }
}
```

## Practical Example: Latency Analyzer

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct LatencyStats {
    min: i64,
    max: i64,
    total: i64,
    count: usize,
}

impl LatencyStats {
    fn new() -> Self {
        LatencyStats {
            min: i64::MAX,
            max: i64::MIN,
            total: 0,
            count: 0,
        }
    }

    fn add(&mut self, latency: i64) {
        if latency < self.min { self.min = latency; }
        if latency > self.max { self.max = latency; }
        self.total += latency;
        self.count += 1;
    }

    fn average(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.total as f64 / self.count as f64 }
    }

    fn print(&self) {
        println!("Latency Statistics:");
        println!("  Requests: {}", self.count);
        println!("  Min: {}ms", self.min);
        println!("  Max: {}ms", self.max);
        println!("  Average: {:.2}ms", self.average());
    }
}

fn main() {
    // Simulated exchange request latency data
    let request_times = [
        (1705329125000_i64, 1705329125015_i64), // 15ms
        (1705329125100_i64, 1705329125108_i64), // 8ms
        (1705329125200_i64, 1705329125225_i64), // 25ms
        (1705329125300_i64, 1705329125312_i64), // 12ms
        (1705329125400_i64, 1705329125450_i64), // 50ms - slow
    ];

    let mut stats = LatencyStats::new();

    for (sent, received) in request_times.iter() {
        let latency = received - sent;
        stats.add(latency);

        if latency > 20 {
            println!("SLOW REQUEST: {}ms at timestamp {}", latency, sent);
        }
    }

    println!();
    stats.print();
}
```

## What We Learned

| Concept | Description | Usage Example |
|---------|-------------|---------------|
| Unix timestamp | Seconds/milliseconds since 1.1.1970 | Trade time |
| SystemTime | System time in Rust | Getting current time |
| Milliseconds | 1/1000 of a second | Binance API |
| Timestamp comparison | Simple number comparison | Sorting orders |
| Intervals | Calculating candle start/end | OHLCV data |

## Practice Exercises

1. **Time Converter**: Write a function `timestamp_to_date_parts(ts: i64) -> (i32, u32, u32, u32, u32, u32)` that extracts year, month, day, hours, minutes, seconds from a timestamp (without external libraries).

2. **Gap Detector**: Write a function `find_data_gaps(timestamps: &[i64], expected_interval: i64) -> Vec<(i64, i64)>` that finds gaps in candlestick data.

3. **Rate Limiter**: Implement a `RateLimiter` struct that tracks the number of requests in the last second and blocks when the limit is exceeded.

4. **Clock Synchronization**: Write a function that calculates the difference between local time and exchange server time based on the timestamp in the API response.

## Homework

1. Create a `TradeHistory` struct that stores trades and allows:
   - Adding trades with automatic timestamp
   - Getting trades from the last N minutes
   - Calculating average price for a period

2. Write a tick-to-candle aggregation function: `aggregate_to_candles(ticks: &[Tick], interval: i64) -> Vec<Candle>`

3. Implement a "heartbeat" system that checks if data from the exchange is arriving regularly and issues a warning if delayed by more than 5 seconds

4. Create a utility for synchronizing data between exchanges with different timestamp formats (seconds vs milliseconds)

## Navigation

[← Previous day](../136-timezones-utc-local/en.md) | [Next day →](../138-duration-time-between-trades/en.md)
