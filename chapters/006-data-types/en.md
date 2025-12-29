# Day 6: Data Types — Price, Quantity, Ticker Symbol

## Trading Analogy

On an exchange, we work with different kinds of data:
- **Prices** — numbers with decimals (42156.78 USDT)
- **Quantities** — also decimals (0.00123 BTC)
- **Trading volumes** — huge whole numbers (1,000,000,000)
- **Tickers** — text ("BTC", "ETH")
- **Statuses** — yes/no (is market open?)

In Rust, each kind of data has its own **type**.

## Scalar Types

Scalar types represent single values.

### 1. Integers

| Type | Size | Range | Trading Example |
|------|------|-------|-----------------|
| `i8` | 8 bits | -128 to 127 | — |
| `i16` | 16 bits | -32,768 to 32,767 | — |
| `i32` | 32 bits | ±2 billion | Daily trade count |
| `i64` | 64 bits | ±9 quintillion | Timestamp in milliseconds |
| `u8` | 8 bits | 0 to 255 | Percentage (0-100) |
| `u16` | 16 bits | 0 to 65,535 | Server port |
| `u32` | 32 bits | 0 to 4 billion | Order ID |
| `u64` | 64 bits | 0 to 18 quintillion | Trading volume in satoshi |

```rust
fn main() {
    let trade_count: i32 = 150;           // Daily trades
    let order_id: u64 = 123456789012;     // Order ID
    let timestamp: i64 = 1703865600000;   // Unix time in ms
    let risk_percent: u8 = 2;             // 2% risk

    println!("Trades: {}", trade_count);
    println!("Order ID: {}", order_id);
}
```

**Analogy:**
- `i` = signed — can be negative (loss)
- `u` = unsigned — positive only (ID, volume)

### 2. Floating Point Numbers (Floats)

| Type | Size | Precision | Example |
|------|------|-----------|---------|
| `f32` | 32 bits | ~7 digits | Fast calculations |
| `f64` | 64 bits | ~15 digits | Precise prices |

```rust
fn main() {
    let btc_price: f64 = 42156.78901234;  // Precise price
    let quick_calc: f32 = 42156.78;       // Less precise

    println!("BTC: {}", btc_price);
    println!("Quick: {}", quick_calc);
}
```

**Important for trading:** Always use `f64` for money! `f32` can lose precision.

```rust
fn main() {
    // f32 problem
    let balance_f32: f32 = 1000000.0 + 0.01;
    println!("f32: {}", balance_f32);  // Might be 1000000.0!

    // f64 solution
    let balance_f64: f64 = 1000000.0 + 0.01;
    println!("f64: {}", balance_f64);  // Exactly 1000000.01
}
```

### 3. Booleans

```rust
fn main() {
    let market_open: bool = true;
    let position_active: bool = false;
    let is_profitable: bool = true;

    println!("Market open: {}", market_open);
    println!("Has position: {}", position_active);
}
```

**Analogy:** Indicator lights on a panel — on/off.

### 4. Characters

```rust
fn main() {
    let direction: char = '↑';  // Trend direction
    let status: char = '✓';     // Trade status
    let currency: char = '₿';   // Bitcoin symbol

    println!("Trend: {}", direction);
    println!("Status: {}", status);
}
```

## Compound Types

### 1. Tuples

Group different types together:

```rust
fn main() {
    // (ticker, price, volume)
    let trade: (&str, f64, f64) = ("BTC/USDT", 42000.0, 0.5);

    // Access by index
    println!("Ticker: {}", trade.0);
    println!("Price: {}", trade.1);
    println!("Volume: {}", trade.2);

    // Destructuring
    let (symbol, price, volume) = trade;
    println!("{}: {} x {}", symbol, price, volume);
}
```

**Analogy:** A row in a trades table — multiple fields together.

### 2. Arrays

Fixed list of same types:

```rust
fn main() {
    // Last 5 closing prices
    let closes: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    println!("First price: {}", closes[0]);
    println!("Last price: {}", closes[4]);

    // Array of same values
    let zeros: [f64; 10] = [0.0; 10];  // 10 zeros

    println!("Length: {}", closes.len());
}
```

**Analogy:** Candles on a chart for the last N periods.

## Strings

Rust has two string types:

### `&str` — string slice (borrowed text)

```rust
fn main() {
    let ticker: &str = "BTC/USDT";
    let exchange: &str = "Binance";

    println!("Trading {} on {}", ticker, exchange);
}
```

### `String` — owned string (can be modified)

```rust
fn main() {
    let mut message = String::from("Price: ");
    message.push_str("42000");
    message.push_str(" USDT");

    println!("{}", message);
}
```

**Analogy:**
- `&str` — a note with ticker on the monitor (read-only)
- `String` — a notebook where we write notes (can modify)

## Practical Example: Trade Structure

```rust
fn main() {
    // Trade data
    let symbol: &str = "ETH/USDT";
    let side: char = 'B';           // B = Buy, S = Sell
    let entry_price: f64 = 2250.50;
    let quantity: f64 = 1.5;
    let stop_loss: f64 = 2200.0;
    let take_profit: f64 = 2350.0;
    let is_active: bool = true;
    let order_id: u64 = 9876543210;

    // Calculations
    let position_value: f64 = entry_price * quantity;
    let potential_loss: f64 = (entry_price - stop_loss) * quantity;
    let potential_profit: f64 = (take_profit - entry_price) * quantity;

    // Output
    println!("╔══════════════════════════════════╗");
    println!("║         TRADE INFO               ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Symbol: {:>20}    ║", symbol);
    println!("║ Side: {:>22}    ║", if side == 'B' { "BUY" } else { "SELL" });
    println!("║ Order ID: {:>18}    ║", order_id);
    println!("║ Entry: {:>21.2}    ║", entry_price);
    println!("║ Quantity: {:>18.4}    ║", quantity);
    println!("║ Value: {:>21.2}    ║", position_value);
    println!("║ Stop Loss: {:>17.2}    ║", stop_loss);
    println!("║ Take Profit: {:>15.2}    ║", take_profit);
    println!("║ Pot. Loss: {:>17.2}    ║", potential_loss);
    println!("║ Pot. Profit: {:>15.2}    ║", potential_profit);
    println!("║ Active: {:>20}    ║", is_active);
    println!("╚══════════════════════════════════╝");
}
```

## Type Conversion

```rust
fn main() {
    let price_int: i32 = 42000;
    let price_float: f64 = price_int as f64;  // i32 -> f64

    let big_number: u64 = 1000000;
    let small_number: u32 = big_number as u32;  // Be careful!

    println!("Float: {}", price_float);
    println!("Small: {}", small_number);
}
```

## What We Learned

| Type | Trading Use Case |
|------|-----------------|
| `i32`, `i64` | Counters, timestamp |
| `u32`, `u64` | IDs, volumes |
| `f64` | Prices, quantities |
| `bool` | Status flags |
| `&str` | Tickers, names |
| `(T, T, T)` | Data grouping |
| `[T; N]` | Historical data |

## Homework

1. Create variables describing an order:
   - Order ID (u64)
   - Ticker (&str)
   - Side (char: 'B' or 'S')
   - Price (f64)
   - Quantity (f64)
   - Is filled (bool)

2. Create an array of 7 closing prices for a week

3. Create a tuple with bid and ask price, calculate spread

4. Output all information nicely formatted

## Navigation

[← Previous day](../005-immutability-locked-price/en.md) | [Next day →](../007-integers-counting-shares/en.md)
