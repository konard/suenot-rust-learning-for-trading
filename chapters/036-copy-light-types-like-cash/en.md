# Day 36: Copy — Light Types Like Cash

## Trading Analogy

Imagine the difference between **cash** and a **bank transfer**:

**Cash (Copy types):**
- When you give someone $100 in cash, you just pull a bill from your pocket
- The operation is instant and cheap
- Easy to "copy" — just withdraw more from the ATM
- BTC price = 42000.0 — it's just a number, you can copy it endlessly

**Bank transfer (Move types):**
- Transferring money requires confirmation, time, sometimes fees
- After the transfer, money leaves your account — you "moved" it
- Transaction history, exchange name string — these are complex data

In Rust, types with the `Copy` trait work like cash — they are copied automatically and cheaply.

## What is Copy?

`Copy` is a special marker trait in Rust that tells the compiler:
> "This type can be safely copied bit-by-bit, just by duplicating bytes in memory"

```rust
fn main() {
    // Copy types — copied automatically
    let price = 42000.0_f64;
    let price_copy = price;  // Copying!

    println!("Original: {}", price);      // Works!
    println!("Copy: {}", price_copy);     // Also works!

    // Non-Copy types — moved
    let ticker = String::from("BTC");
    let ticker_moved = ticker;  // Moving!

    // println!("{}", ticker);  // ERROR! ticker was moved
    println!("Moved: {}", ticker_moved);
}
```

## Which Types Have Copy?

### All Primitive Numeric Types

```rust
fn main() {
    // Integers — Copy
    let shares: i32 = 100;
    let shares_copy = shares;
    println!("Shares: {} and {}", shares, shares_copy);

    // Floating point — Copy
    let btc_price: f64 = 42000.0;
    let eth_price: f64 = btc_price;  // Copy of value
    println!("BTC: {}, ETH tracking: {}", btc_price, eth_price);

    // Unsigned — also Copy
    let volume: u64 = 1_000_000;
    process_volume(volume);
    println!("Original volume: {}", volume);  // Still available!
}

fn process_volume(v: u64) {
    println!("Processing volume: {}", v);
}
```

### Bool and Char

```rust
fn main() {
    // bool — Copy
    let is_bull_market = true;
    let market_status = is_bull_market;
    println!("Bull? {} / {}", is_bull_market, market_status);

    // char — Copy
    let trend: char = '↑';
    let saved_trend = trend;
    println!("Trend: {} and saved: {}", trend, saved_trend);
}
```

### Tuples of Copy Types

```rust
fn main() {
    // Tuple of Copy types — also Copy!
    let bid_ask: (f64, f64) = (41999.0, 42001.0);
    let spread_data = bid_ask;  // Copying the tuple

    println!("Original: bid={}, ask={}", bid_ask.0, bid_ask.1);
    println!("Copy: bid={}, ask={}", spread_data.0, spread_data.1);

    // OHLC data
    let candle: (f64, f64, f64, f64) = (42000.0, 42500.0, 41800.0, 42300.0);
    analyze_candle(candle);
    println!("Candle still available: {:?}", candle);
}

fn analyze_candle(ohlc: (f64, f64, f64, f64)) {
    let (open, high, low, close) = ohlc;
    let range = high - low;
    let body = (close - open).abs();
    println!("Range: {:.2}, Body: {:.2}", range, body);
}
```

### Arrays of Copy Types

```rust
fn main() {
    // Array of Copy types — Copy (if size is known)
    let prices: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let backup = prices;  // Copying entire array

    println!("Original: {:?}", prices);
    println!("Backup: {:?}", backup);

    // Passing to function — also a copy
    let avg = calculate_average(prices);
    println!("Average: {:.2}, original: {:?}", avg, prices);
}

fn calculate_average(data: [f64; 5]) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

## Copy vs Clone

It's important to understand the difference:

```rust
fn main() {
    // Copy — implicit, automatic, bitwise
    let price = 42000.0;
    let price2 = price;  // Implicit copy

    // Clone — explicit, can be expensive
    let ticker = String::from("BTC/USDT");
    let ticker2 = ticker.clone();  // Explicit cloning

    println!("Price: {} / {}", price, price2);
    println!("Ticker: {} / {}", ticker, ticker2);
}
```

| Characteristic | Copy | Clone |
|---------------|------|-------|
| Invocation | Automatic | Explicit (.clone()) |
| Cost | Always cheap | Can be expensive |
| Depth | Bitwise copy | Deep copy |
| Requirements | Only simple types | Any type |

## Creating Your Own Copy Type

```rust
// For Copy you also need Clone
#[derive(Debug, Copy, Clone)]
struct Price {
    value: f64,
    timestamp: u64,
}

#[derive(Debug, Copy, Clone)]
struct OrderLevel {
    price: f64,
    quantity: f64,
    side: bool,  // true = buy, false = sell
}

fn main() {
    let btc_price = Price {
        value: 42000.0,
        timestamp: 1703980800,
    };

    // Now Price is Copy!
    let saved_price = btc_price;
    println!("Current: {:?}", btc_price);
    println!("Saved: {:?}", saved_price);

    // OrderLevel is also Copy
    let bid = OrderLevel {
        price: 41999.0,
        quantity: 0.5,
        side: true,
    };

    process_order(bid);
    println!("Bid still available: {:?}", bid);
}

fn process_order(order: OrderLevel) {
    println!("Processing: {} @ {}",
        if order.side { "BUY" } else { "SELL" },
        order.price
    );
}
```

## When Copy is Not Possible

```rust
// CANNOT be Copy — contains String
struct Trade {
    symbol: String,  // String is not Copy!
    price: f64,
    quantity: f64,
}

// CANNOT be Copy — contains Vec
struct Portfolio {
    positions: Vec<f64>,  // Vec is not Copy!
    total_value: f64,
}

// CAN be Copy — only primitives
#[derive(Copy, Clone)]
struct SimplePosition {
    entry_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let pos = SimplePosition {
        entry_price: 42000.0,
        quantity: 0.5,
        is_long: true,
    };

    let backup = pos;  // Copy works
    println!("Entry: {} / {}", pos.entry_price, backup.entry_price);
}
```

## Practical Example: Price Tracking System

```rust
#[derive(Debug, Copy, Clone)]
struct PricePoint {
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug, Copy, Clone)]
struct PriceAlert {
    target_price: f64,
    is_above: bool,  // true = alert when price goes above
    triggered: bool,
}

fn main() {
    // Current price — Copy, can be passed everywhere
    let current = PricePoint {
        price: 42500.0,
        volume: 100.0,
        timestamp: 1703980800,
    };

    // Create alerts
    let mut alerts = [
        PriceAlert { target_price: 43000.0, is_above: true, triggered: false },
        PriceAlert { target_price: 42000.0, is_above: false, triggered: false },
        PriceAlert { target_price: 45000.0, is_above: true, triggered: false },
    ];

    // Check alerts — current is copied on each call
    for alert in &mut alerts {
        check_alert(alert, current);
    }

    // current is still available!
    println!("\nCurrent price: ${:.2}", current.price);
    println!("\nAlert status:");
    for (i, alert) in alerts.iter().enumerate() {
        println!("  Alert {}: ${:.2} {} - {}",
            i + 1,
            alert.target_price,
            if alert.is_above { "above" } else { "below" },
            if alert.triggered { "TRIGGERED!" } else { "waiting" }
        );
    }
}

fn check_alert(alert: &mut PriceAlert, price: PricePoint) {
    if alert.triggered {
        return;
    }

    let should_trigger = if alert.is_above {
        price.price >= alert.target_price
    } else {
        price.price <= alert.target_price
    };

    if should_trigger {
        alert.triggered = true;
        println!("ALERT: Price ${:.2} {} ${:.2}!",
            price.price,
            if alert.is_above { "crossed above" } else { "dropped below" },
            alert.target_price
        );
    }
}
```

## Copy in Functions: When It Matters

```rust
#[derive(Copy, Clone)]
struct RiskMetrics {
    max_position_size: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
    max_daily_loss: f64,
}

fn main() {
    let risk = RiskMetrics {
        max_position_size: 10000.0,
        stop_loss_percent: 2.0,
        take_profit_percent: 6.0,
        max_daily_loss: 500.0,
    };

    // Thanks to Copy, risk can be used multiple times
    let position_ok = validate_position_size(risk, 5000.0);
    let stop_price = calculate_stop_loss(risk, 42000.0);
    let take_price = calculate_take_profit(risk, 42000.0);
    let daily_ok = check_daily_risk(risk, 300.0);

    println!("Position valid: {}", position_ok);
    println!("Stop loss: ${:.2}", stop_price);
    println!("Take profit: ${:.2}", take_price);
    println!("Daily risk OK: {}", daily_ok);

    // risk is still available after all calls!
    println!("\nMax position: ${:.2}", risk.max_position_size);
}

fn validate_position_size(config: RiskMetrics, size: f64) -> bool {
    size <= config.max_position_size
}

fn calculate_stop_loss(config: RiskMetrics, entry: f64) -> f64 {
    entry * (1.0 - config.stop_loss_percent / 100.0)
}

fn calculate_take_profit(config: RiskMetrics, entry: f64) -> f64 {
    entry * (1.0 + config.take_profit_percent / 100.0)
}

fn check_daily_risk(config: RiskMetrics, current_loss: f64) -> bool {
    current_loss < config.max_daily_loss
}
```

## Exercises

### Exercise 1: Identify Copy Types
```rust
// Which of these types are Copy?
// 1. i32
// 2. String
// 3. (f64, f64)
// 4. Vec<f64>
// 5. [f64; 3]
// 6. &str
// 7. bool
// 8. (String, i32)

fn main() {
    // Verify your assumptions!
    let a: i32 = 42;
    let _b = a;
    println!("i32 is Copy: {}", a);  // Does it compile?

    // Add checks for other types...
}
```

### Exercise 2: Create a Copy Struct

```rust
// Create a struct Tick with fields:
// - price: f64
// - bid: f64
// - ask: f64
// - volume: f64
// Make it Copy and write a calculate_spread function

fn main() {
    // Your code here
}
```

### Exercise 3: Copy vs Move

```rust
// Fix the code to make it compile,
// using your knowledge of Copy

fn main() {
    let price = 42000.0_f64;
    let ticker = String::from("BTC");

    print_price(price);
    print_price(price);  // Should work

    print_ticker(ticker);
    // print_ticker(ticker);  // How to make this work?
}

fn print_price(p: f64) {
    println!("Price: {}", p);
}

fn print_ticker(t: String) {
    println!("Ticker: {}", t);
}
```

### Exercise 4: Risk Calculator

```rust
// Create a Copy struct TradeSetup and functions for:
// 1. Calculating position size
// 2. Calculating Risk/Reward ratio
// 3. Calculating maximum loss

#[derive(Copy, Clone)]
struct TradeSetup {
    // Fill in the fields
}

fn main() {
    // Your code here
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Copy trait | Marker for types with cheap bitwise copying |
| Copy types | i32, f64, bool, char, tuples and arrays of Copy types |
| Non-Copy | String, Vec, Box — require explicit clone() |
| #[derive(Copy, Clone)] | Automatic implementation for custom structs |
| Limitation | All struct fields must be Copy |

## Homework

1. **Quote struct**: Create a Copy struct for exchange quote with fields bid, ask, bid_size, ask_size, timestamp. Write functions to calculate spread and mid-price.

2. **Signal system**: Create a Copy struct for trading signal (entry price, stop, take, direction). Write a function that takes a signal and current price and returns an action.

3. **Multi-timeframe**: Create a function that takes the same price (Copy) and analyzes it against different levels (array of levels). Make sure the price remains available after all checks.

4. **Copy vs Clone benchmark**: Create two versions of a struct — one Copy (primitives only), another with String. Measure time for 1000 copies and clones. What's the difference?

## Navigation

[← Previous day](../035-clone-duplicating-orders/en.md) | [Next day →](../037-drop-cleaning-up-positions/en.md)
