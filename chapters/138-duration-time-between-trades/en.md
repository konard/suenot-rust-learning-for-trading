# Day 138: Duration — Time Between Trades

## Trading Analogy

Imagine you're analyzing your trades. You want to know: how much time passed between entry and exit? Did you hold BTC for 5 minutes (scalping) or 3 days (swing trading)? `Duration` in Rust is exactly this kind of "time span". It's not a specific date, but an **interval** — like the holding time of a position.

## What is Duration

`Duration` is a type from Rust's standard library (`std::time::Duration`) representing a span of time. It stores time in seconds and nanoseconds, ensuring high precision.

```rust
use std::time::Duration;

fn main() {
    // Creating Duration in different ways
    let five_seconds = Duration::from_secs(5);
    let half_second = Duration::from_millis(500);
    let microseconds = Duration::from_micros(1000);
    let nanoseconds = Duration::from_nanos(1_000_000);

    println!("5 seconds: {:?}", five_seconds);
    println!("500 ms: {:?}", half_second);
    println!("1000 μs: {:?}", microseconds);
    println!("1_000_000 ns: {:?}", nanoseconds);
}
```

## Duration in Trading

### Position Holding Time

```rust
use std::time::Duration;

fn main() {
    // Holding time for different trade types
    let scalp_trade = Duration::from_secs(45);           // 45 seconds — scalp
    let day_trade = Duration::from_secs(4 * 60 * 60);    // 4 hours — day trade
    let swing_trade = Duration::from_secs(3 * 24 * 60 * 60); // 3 days — swing

    println!("Scalp: {} sec", scalp_trade.as_secs());
    println!("Day trade: {} hours", day_trade.as_secs() / 3600);
    println!("Swing: {} days", swing_trade.as_secs() / 86400);

    // Classify trade by holding time
    classify_trade(scalp_trade);
    classify_trade(day_trade);
    classify_trade(swing_trade);
}

fn classify_trade(holding_time: Duration) {
    let minutes = holding_time.as_secs() / 60;

    let trade_type = if minutes < 5 {
        "Scalp"
    } else if minutes < 60 {
        "Short-term"
    } else if minutes < 24 * 60 {
        "Day trade"
    } else {
        "Swing/Position"
    };

    println!("Trade duration {} min = {}", minutes, trade_type);
}
```

### Time Between Trades

```rust
use std::time::Duration;

fn main() {
    // Time between consecutive trades
    let trade_intervals = vec![
        Duration::from_secs(120),   // 2 minutes after first trade
        Duration::from_secs(45),    // 45 seconds after second
        Duration::from_secs(300),   // 5 minutes after third
        Duration::from_secs(60),    // 1 minute after fourth
    ];

    // Trading frequency analysis
    let total_time: Duration = trade_intervals.iter().sum();
    let avg_interval = total_time / trade_intervals.len() as u32;

    println!("Total trading session time: {} sec", total_time.as_secs());
    println!("Average time between trades: {} sec", avg_interval.as_secs());
    println!("Number of trades: {}", trade_intervals.len() + 1);

    // Trades per hour (if trading at this pace)
    let trades_per_hour = 3600.0 / avg_interval.as_secs_f64();
    println!("Pace: {:.1} trades per hour", trades_per_hour);
}
```

## Operations with Duration

### Time Arithmetic

```rust
use std::time::Duration;

fn main() {
    let entry_to_sl = Duration::from_secs(30);    // Time to stop-loss
    let sl_to_exit = Duration::from_secs(15);     // After stop-loss to close

    // Addition
    let total_time = entry_to_sl + sl_to_exit;
    println!("Total time in trade: {} sec", total_time.as_secs());

    // Multiplication
    let three_trades = total_time * 3;
    println!("Time for 3 such trades: {} sec", three_trades.as_secs());

    // Division
    let half_time = total_time / 2;
    println!("Half time: {} sec", half_time.as_secs());

    // Subtraction (with check)
    if let Some(diff) = entry_to_sl.checked_sub(sl_to_exit) {
        println!("Difference: {} sec", diff.as_secs());
    }

    // saturating_sub — safe subtraction (doesn't go negative)
    let safe_diff = sl_to_exit.saturating_sub(entry_to_sl);
    println!("Safe difference: {} sec", safe_diff.as_secs());
}
```

### Comparing Duration

```rust
use std::time::Duration;

fn main() {
    let max_holding_time = Duration::from_secs(300); // 5 minutes max
    let current_holding = Duration::from_secs(180);  // 3 minutes held

    if current_holding < max_holding_time {
        let remaining = max_holding_time - current_holding;
        println!("Can hold for {} more sec", remaining.as_secs());
    } else {
        println!("Time to close the position!");
    }

    // Check for limit exceeded
    let trades = vec![
        ("BTCUSD", Duration::from_secs(240)),
        ("ETHUSD", Duration::from_secs(600)),
        ("SOLUSD", Duration::from_secs(180)),
    ];

    for (symbol, holding) in &trades {
        if *holding > max_holding_time {
            println!("{}: time limit EXCEEDED!", symbol);
        } else {
            println!("{}: within limit", symbol);
        }
    }
}
```

## Duration Methods

### Extracting Components

```rust
use std::time::Duration;

fn main() {
    let trade_duration = Duration::new(3723, 500_000_000); // 1 hour 2 min 3.5 sec

    // Total time in different units
    println!("In seconds: {}", trade_duration.as_secs());
    println!("In milliseconds: {}", trade_duration.as_millis());
    println!("In microseconds: {}", trade_duration.as_micros());
    println!("In nanoseconds: {}", trade_duration.as_nanos());

    // As floating point
    println!("In seconds (f64): {:.3}", trade_duration.as_secs_f64());
    println!("In seconds (f32): {:.3}", trade_duration.as_secs_f32());

    // Individual components
    println!("Seconds (integer part): {}", trade_duration.as_secs());
    println!("Nanoseconds (fractional part): {}", trade_duration.subsec_nanos());
    println!("Milliseconds (fractional part): {}", trade_duration.subsec_millis());
}
```

### Formatting for Trading

```rust
use std::time::Duration;

fn main() {
    let holding_times = vec![
        Duration::from_secs(45),
        Duration::from_secs(3661),
        Duration::from_secs(86400 + 7200 + 180),
    ];

    for duration in holding_times {
        println!("{}", format_trading_duration(duration));
    }
}

fn format_trading_duration(d: Duration) -> String {
    let total_secs = d.as_secs();

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
```

## Duration + chrono

The `chrono` crate extends time handling capabilities:

```rust
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::time::Duration as StdDuration;

fn main() {
    // Trade entry and exit times
    let entry_time: DateTime<Utc> = "2024-01-15T10:30:00Z".parse().unwrap();
    let exit_time: DateTime<Utc> = "2024-01-15T14:45:30Z".parse().unwrap();

    // Difference is chrono::Duration
    let holding_time: ChronoDuration = exit_time - entry_time;

    println!("Entry: {}", entry_time);
    println!("Exit: {}", exit_time);
    println!("Time in position: {} hours {} minutes {} seconds",
        holding_time.num_hours(),
        holding_time.num_minutes() % 60,
        holding_time.num_seconds() % 60
    );

    // Convert to std::time::Duration
    if let Ok(std_duration) = holding_time.to_std() {
        println!("std::time::Duration: {:?}", std_duration);
    }

    // Create chrono::Duration
    let max_hold = ChronoDuration::hours(8);
    if holding_time < max_hold {
        println!("Trade within daily limit");
    }
}
```

## Practical Example: Trade Analysis

```rust
use std::time::Duration;

fn main() {
    let trades = vec![
        Trade::new("BTCUSD", 42000.0, 42500.0, Duration::from_secs(180)),
        Trade::new("ETHUSD", 2200.0, 2150.0, Duration::from_secs(3600)),
        Trade::new("BTCUSD", 42100.0, 42800.0, Duration::from_secs(45)),
        Trade::new("SOLUSD", 95.0, 98.5, Duration::from_secs(7200)),
    ];

    let analysis = analyze_trades(&trades);
    print_analysis(&analysis);
}

struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    holding_time: Duration,
}

impl Trade {
    fn new(symbol: &str, entry: f64, exit: f64, holding: Duration) -> Self {
        Trade {
            symbol: symbol.to_string(),
            entry_price: entry,
            exit_price: exit,
            holding_time: holding,
        }
    }

    fn pnl_percent(&self) -> f64 {
        ((self.exit_price - self.entry_price) / self.entry_price) * 100.0
    }

    fn is_profitable(&self) -> bool {
        self.exit_price > self.entry_price
    }
}

struct TradeAnalysis {
    total_trades: usize,
    profitable_trades: usize,
    total_holding_time: Duration,
    avg_holding_time: Duration,
    fastest_trade: Duration,
    slowest_trade: Duration,
    avg_pnl_percent: f64,
}

fn analyze_trades(trades: &[Trade]) -> TradeAnalysis {
    let total_trades = trades.len();
    let profitable_trades = trades.iter().filter(|t| t.is_profitable()).count();

    let total_holding_time: Duration = trades.iter()
        .map(|t| t.holding_time)
        .sum();

    let avg_holding_time = total_holding_time / total_trades as u32;

    let fastest_trade = trades.iter()
        .map(|t| t.holding_time)
        .min()
        .unwrap_or(Duration::ZERO);

    let slowest_trade = trades.iter()
        .map(|t| t.holding_time)
        .max()
        .unwrap_or(Duration::ZERO);

    let avg_pnl_percent = trades.iter()
        .map(|t| t.pnl_percent())
        .sum::<f64>() / total_trades as f64;

    TradeAnalysis {
        total_trades,
        profitable_trades,
        total_holding_time,
        avg_holding_time,
        fastest_trade,
        slowest_trade,
        avg_pnl_percent,
    }
}

fn print_analysis(a: &TradeAnalysis) {
    println!("╔══════════════════════════════════════╗");
    println!("║          TRADE ANALYSIS              ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Total trades:        {:>14} ║", a.total_trades);
    println!("║ Profitable:          {:>14} ║", a.profitable_trades);
    println!("║ Win rate:            {:>13.1}% ║",
        (a.profitable_trades as f64 / a.total_trades as f64) * 100.0);
    println!("║ Average PnL:         {:>13.2}% ║", a.avg_pnl_percent);
    println!("╠══════════════════════════════════════╣");
    println!("║ Total time:          {:>11} sec ║", a.total_holding_time.as_secs());
    println!("║ Average time:        {:>11} sec ║", a.avg_holding_time.as_secs());
    println!("║ Fastest trade:       {:>11} sec ║", a.fastest_trade.as_secs());
    println!("║ Slowest trade:       {:>11} sec ║", a.slowest_trade.as_secs());
    println!("╚══════════════════════════════════════╝");
}
```

## Measuring Execution Time

```rust
use std::time::{Duration, Instant};

fn main() {
    // Measure indicator calculation time
    let prices: Vec<f64> = (0..10000).map(|i| 42000.0 + (i as f64 * 0.1)).collect();

    let start = Instant::now();
    let sma = calculate_sma(&prices, 20);
    let elapsed: Duration = start.elapsed();

    println!("SMA-20 calculated in {:?}", elapsed);
    println!("Last value: {:.2}", sma.last().unwrap_or(&0.0));

    // Compare different implementations
    benchmark_implementations(&prices);
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();
    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

fn benchmark_implementations(prices: &[f64]) {
    // Simple implementation
    let start = Instant::now();
    for _ in 0..100 {
        let _: Vec<f64> = prices.iter()
            .enumerate()
            .filter(|(i, _)| *i >= 19)
            .map(|(i, _)| prices[i-19..=i].iter().sum::<f64>() / 20.0)
            .collect();
    }
    let simple_time = start.elapsed();

    // Optimized implementation
    let start = Instant::now();
    for _ in 0..100 {
        let _ = calculate_sma(prices, 20);
    }
    let optimized_time = start.elapsed();

    println!("\nComparison (100 iterations):");
    println!("Simple: {:?}", simple_time);
    println!("Optimized: {:?}", optimized_time);
    println!("Speedup: {:.1}x",
        simple_time.as_secs_f64() / optimized_time.as_secs_f64());
}
```

## Timeouts and Delays

```rust
use std::time::Duration;
use std::thread;

fn main() {
    println!("Waiting for order confirmation...");

    // Simulate waiting with timeout
    let timeout = Duration::from_secs(5);
    let check_interval = Duration::from_millis(500);
    let mut elapsed = Duration::ZERO;
    let mut confirmed = false;

    while elapsed < timeout {
        // Simulate status check
        if check_order_status() {
            confirmed = true;
            break;
        }

        thread::sleep(check_interval);
        elapsed += check_interval;
        println!("  Elapsed: {:?}", elapsed);
    }

    if confirmed {
        println!("Order confirmed!");
    } else {
        println!("Timeout: order not confirmed within {:?}", timeout);
    }
}

fn check_order_status() -> bool {
    // Simulation: confirms on 3rd check
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER >= 3
    }
}
```

## Duration Constants

```rust
use std::time::Duration;

fn main() {
    // Useful constants
    println!("ZERO: {:?}", Duration::ZERO);
    println!("MAX: {:?}", Duration::MAX);

    // Check for zero
    let no_delay = Duration::ZERO;
    if no_delay.is_zero() {
        println!("No delay");
    }

    // Safe operations
    let result = Duration::MAX.checked_add(Duration::from_secs(1));
    match result {
        Some(d) => println!("Result: {:?}", d),
        None => println!("Overflow!"),
    }
}
```

## What We Learned

| Method | Description | Example |
|--------|-------------|---------|
| `Duration::from_secs(n)` | From seconds | `Duration::from_secs(60)` |
| `Duration::from_millis(n)` | From milliseconds | `Duration::from_millis(500)` |
| `Duration::from_secs_f64(f)` | From fractional seconds | `Duration::from_secs_f64(1.5)` |
| `.as_secs()` | To seconds | `d.as_secs()` |
| `.as_millis()` | To milliseconds | `d.as_millis()` |
| `.as_secs_f64()` | To fractional seconds | `d.as_secs_f64()` |
| `+`, `-`, `*`, `/` | Arithmetic | `d1 + d2` |
| `.checked_add()` | Safe addition | `d.checked_add(d2)` |
| `.saturating_sub()` | Subtraction without negative | `d.saturating_sub(d2)` |
| `Duration::ZERO` | Zero duration | Comparison |
| `.is_zero()` | Check for zero | `d.is_zero()` |

## Homework

1. Write a function `average_trade_duration(trades: &[Trade]) -> Option<Duration>` that returns the average position holding time

2. Create a function `classify_by_duration(trades: &[Trade]) -> HashMap<String, Vec<&Trade>>` that groups trades by type: "scalp" (< 5 min), "intraday" (5 min - 24 h), "swing" (> 24 h)

3. Implement a function `calculate_time_weighted_pnl(trades: &[Trade]) -> f64` that calculates PnL weighted by holding time (longer trades have more weight)

4. Write a `TradingSession` struct with methods:
   - `start()` — start session
   - `record_trade(trade: Trade)` — record a trade
   - `finish()` — end session
   - `report()` — report with session time, trade count, average interval between trades

## Navigation

[← Previous day](../137-timestamp-unix-time/en.md) | [Next day →](../139-time-formatting/en.md)
