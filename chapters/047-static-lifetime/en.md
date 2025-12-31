# Day 47: 'static — Data Lives Forever Like Market History

## Trading Analogy

Imagine **Bitcoin price history since its creation** — this data exists "forever" in the context of your program. It was there before launch, will be available throughout runtime, and doesn't depend on specific functions or scopes. This is `'static` — a lifetime for data that exists for the entire duration of the program.

Another example: **exchange name "Binance"** or **ticker "BTC/USDT"** — these are constants that never change and are always available. They're embedded directly in the program's binary.

## What is 'static?

`'static` is a special lifetime in Rust, meaning that data:
- Lives for the entire duration of the program
- Is stored in the program's binary (for string literals)
- Or is created and never freed (for `Box::leak` and similar)

```rust
fn main() {
    // String literals have 'static lifetime
    let exchange: &'static str = "Binance";
    let ticker: &'static str = "BTC/USDT";

    println!("Trading {} on {}", ticker, exchange);
}
```

## String Literals — The Most Common Case

```rust
fn main() {
    // All string literals have type &'static str
    let buy_signal: &'static str = "BUY";
    let sell_signal: &'static str = "SELL";
    let hold_signal: &'static str = "HOLD";

    let signal = get_trading_signal(42000.0, 41000.0);
    println!("Signal: {}", signal);
}

fn get_trading_signal(current_price: f64, sma: f64) -> &'static str {
    if current_price > sma * 1.02 {
        "BUY"  // This is &'static str
    } else if current_price < sma * 0.98 {
        "SELL"
    } else {
        "HOLD"
    }
}
```

## Static Constants for Configuration

```rust
// Static constants — classic example of 'static
static EXCHANGE_NAME: &str = "Binance";
static DEFAULT_FEE_PERCENT: f64 = 0.1;
static MAX_POSITION_SIZE: f64 = 100000.0;
static SUPPORTED_PAIRS: [&str; 4] = ["BTC/USDT", "ETH/USDT", "SOL/USDT", "XRP/USDT"];

fn main() {
    println!("Trading on: {}", EXCHANGE_NAME);
    println!("Default fee: {}%", DEFAULT_FEE_PERCENT);
    println!("Max position: ${}", MAX_POSITION_SIZE);

    println!("Supported pairs:");
    for pair in SUPPORTED_PAIRS.iter() {
        println!("  - {}", pair);
    }
}
```

## const vs static

```rust
// const — value is substituted at each use site
const RISK_PERCENT: f64 = 2.0;
const TRADING_HOURS_START: u32 = 9;
const TRADING_HOURS_END: u32 = 17;

// static — one memory location for the entire program
static BROKER_NAME: &str = "Interactive Brokers";
static mut TOTAL_TRADES: u32 = 0;  // Mutable static (dangerous!)

fn main() {
    // const is simply substituted
    let risk = RISK_PERCENT;

    // static has a fixed address
    println!("Broker: {}", BROKER_NAME);

    // Mutable static requires unsafe
    unsafe {
        TOTAL_TRADES += 1;
        println!("Total trades: {}", TOTAL_TRADES);
    }
}
```

## 'static in Types — Requirement for Data

```rust
use std::thread;

fn main() {
    let ticker = String::from("BTC/USDT");

    // Error: ticker is not 'static, it belongs to main
    // thread::spawn(|| {
    //     println!("Trading {}", ticker);
    // });

    // Solution 1: move — transfer ownership to the thread
    let ticker_for_thread = ticker.clone();
    thread::spawn(move || {
        println!("Trading {}", ticker_for_thread);
    });

    // Solution 2: use 'static data
    thread::spawn(|| {
        let static_ticker: &'static str = "ETH/USDT";
        println!("Also trading {}", static_ticker);
    });

    // Give threads time to complete
    thread::sleep(std::time::Duration::from_millis(100));
}
```

## Practical Example: Trading System Configuration

```rust
// Configuration constants with 'static lifetime
static CONFIG: TradingConfig = TradingConfig {
    exchange: "Binance",
    base_currency: "USDT",
    risk_per_trade: 2.0,
    max_daily_trades: 10,
    allowed_pairs: &["BTC/USDT", "ETH/USDT", "SOL/USDT"],
};

struct TradingConfig {
    exchange: &'static str,
    base_currency: &'static str,
    risk_per_trade: f64,
    max_daily_trades: u32,
    allowed_pairs: &'static [&'static str],
}

impl TradingConfig {
    fn is_pair_allowed(&self, pair: &str) -> bool {
        self.allowed_pairs.contains(&pair)
    }

    fn calculate_position_size(&self, balance: f64) -> f64 {
        balance * (self.risk_per_trade / 100.0)
    }
}

fn main() {
    println!("Exchange: {}", CONFIG.exchange);
    println!("Risk per trade: {}%", CONFIG.risk_per_trade);

    let pair = "BTC/USDT";
    if CONFIG.is_pair_allowed(pair) {
        let position = CONFIG.calculate_position_size(10000.0);
        println!("Position size for {}: ${:.2}", pair, position);
    }

    // Check disallowed pair
    if !CONFIG.is_pair_allowed("DOGE/USDT") {
        println!("DOGE/USDT is not in allowed pairs");
    }
}
```

## Creating 'static Data Dynamically (Box::leak)

```rust
fn main() {
    // Create data that will live forever
    let market_data: &'static MarketSnapshot = create_static_snapshot();

    println!("Static snapshot:");
    println!("  Price: ${}", market_data.price);
    println!("  Volume: {}", market_data.volume);

    // This data is now accessible everywhere and always
    process_in_another_function(market_data);
}

struct MarketSnapshot {
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn create_static_snapshot() -> &'static MarketSnapshot {
    let snapshot = Box::new(MarketSnapshot {
        price: 42000.0,
        volume: 1500.0,
        timestamp: 1234567890,
    });

    // Box::leak turns data into 'static
    // WARNING: memory will never be freed!
    Box::leak(snapshot)
}

fn process_in_another_function(data: &'static MarketSnapshot) {
    println!("Processing snapshot from timestamp: {}", data.timestamp);
}
```

## 'static bound vs 'static lifetime

```rust
use std::fmt::Display;

// T: 'static means T contains NO non-'static references
// This does NOT mean T itself must be &'static
fn log_trade_info<T: Display + 'static>(info: T) {
    println!("[TRADE LOG] {}", info);
}

fn main() {
    // String owns its data, contains no references — OK
    let trade = String::from("BUY 0.5 BTC @ $42,000");
    log_trade_info(trade);

    // i64 — primitive, no references — OK
    log_trade_info(42000_i64);

    // &'static str — static reference — OK
    log_trade_info("Executed");

    // Won't work with temporary reference:
    // let local = String::from("local");
    // log_trade_info(&local);  // Error: &String is not 'static
}
```

## Reference Table for Market Data

```rust
// Static table of order types
static ORDER_TYPES: &[OrderTypeInfo] = &[
    OrderTypeInfo { name: "MARKET", requires_price: false, description: "Execute immediately at market price" },
    OrderTypeInfo { name: "LIMIT", requires_price: true, description: "Execute at specified price or better" },
    OrderTypeInfo { name: "STOP", requires_price: true, description: "Trigger market order at stop price" },
    OrderTypeInfo { name: "STOP_LIMIT", requires_price: true, description: "Trigger limit order at stop price" },
];

struct OrderTypeInfo {
    name: &'static str,
    requires_price: bool,
    description: &'static str,
}

fn get_order_type_info(name: &str) -> Option<&'static OrderTypeInfo> {
    ORDER_TYPES.iter().find(|info| info.name == name)
}

fn main() {
    if let Some(info) = get_order_type_info("LIMIT") {
        println!("Order type: {}", info.name);
        println!("Requires price: {}", info.requires_price);
        println!("Description: {}", info.description);
    }

    println!("\nAll order types:");
    for order_type in ORDER_TYPES.iter() {
        println!("  {} - {}", order_type.name, order_type.description);
    }
}
```

## Lazy Static — Initialize on First Use

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

// OnceLock allows one-time initialization of static data
static EXCHANGE_FEES: OnceLock<HashMap<&'static str, f64>> = OnceLock::new();

fn get_exchange_fees() -> &'static HashMap<&'static str, f64> {
    EXCHANGE_FEES.get_or_init(|| {
        let mut fees = HashMap::new();
        fees.insert("Binance", 0.1);
        fees.insert("Coinbase", 0.5);
        fees.insert("Kraken", 0.26);
        fees.insert("FTX", 0.07);
        fees
    })
}

fn main() {
    let fees = get_exchange_fees();

    println!("Exchange fees:");
    for (exchange, fee) in fees.iter() {
        println!("  {}: {}%", exchange, fee);
    }

    // Second call returns the same data (already initialized)
    let fee = get_exchange_fees().get("Binance").unwrap();
    println!("\nBinance fee: {}%", fee);
}
```

## When to Use 'static

| Scenario | Example | Solution |
|----------|---------|----------|
| Constant strings | Exchange names, tickers | `&'static str` literals |
| Global configuration | System settings | `static CONFIG: ...` |
| Lookup tables | Order types, fees | `static TABLE: &[...]` |
| Data for threads | Thread-safe access | `Arc<T>` or `'static` |
| Lifetime cache | Historical data | `Box::leak` or `lazy_static` |

## Cautions

```rust
// ⚠️ Box::leak creates memory leaks — use carefully!
fn bad_example() {
    for _ in 0..1000000 {
        let leaked: &'static str = Box::leak(String::from("data").into_boxed_str());
        // Each iteration leaks memory!
    }
}

// ✅ Better to use Arc for shared data
use std::sync::Arc;

fn good_example() {
    let shared_data = Arc::new(String::from("shared market data"));

    let data_clone = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        println!("Thread 1: {}", data_clone);
    });

    let data_clone2 = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        println!("Thread 2: {}", data_clone2);
    });
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `'static` lifetime | Data lives for entire program |
| `&'static str` | String literals in code |
| `static` variables | Global data with fixed address |
| `const` | Value substituted at compile time |
| `T: 'static` bound | Type contains no non-static references |
| `Box::leak` | Creates 'static data dynamically |
| `OnceLock` | Lazy initialization of static data |

## Homework

1. **Exchange Configuration**: Create a static table with exchange information (name, fees, minimum lot) and functions to query this data.

2. **Signal System**: Write a function `get_signal_description(signal: &str) -> Option<&'static SignalInfo>` that returns trading signal descriptions from a static table.

3. **Historical Data Cache**: Using `OnceLock`, create a cache with historical prices that initializes on first access.

4. **Thread-safe Logger**: Create a simple trading operations logger that can be safely used from multiple threads using `'static` data.

## Navigation

[← Previous day](../046-lifetime-annotations/en.md) | [Next day →](../048-lifetime-elision/en.md)
