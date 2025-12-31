# Day 26: Constants — Fixed Exchange Fee

## Trading Analogy

Every exchange has a **trading fee** — a fixed percentage that doesn't change during bot operation. Binance charges 0.1%, Bybit — 0.1%, Kraken — 0.16%. These values are known in advance and remain constant.

In Rust, there's a special keyword for such values — `const`. Constants **never change** and are computed at compile time.

## Declaring Constants

```rust
const BINANCE_FEE: f64 = 0.001;        // 0.1%
const BYBIT_FEE: f64 = 0.001;          // 0.1%
const MAX_LEVERAGE: u32 = 125;          // Maximum leverage
const MIN_ORDER_USDT: f64 = 10.0;       // Minimum order

fn main() {
    println!("Binance fee: {}%", BINANCE_FEE * 100.0);
    println!("Maximum leverage: {}x", MAX_LEVERAGE);
    println!("Minimum order: {} USDT", MIN_ORDER_USDT);
}
```

## Naming Conventions

Constants are written in **UPPER_SNAKE_CASE**:

```rust
const TRADING_FEE: f64 = 0.001;          // Correct
const MAX_POSITION_SIZE: f64 = 100000.0; // Correct
const minOrderSize: f64 = 10.0;          // Wrong! (but will compile)
```

## Constants vs. let

| Feature | `const` | `let` (without mut) |
|---------|---------|---------------------|
| Mutability | Never | No, but shadowing allowed |
| Type | Required | Can be inferred |
| Evaluation | Compile time | Runtime |
| Scope | Can be global | Local only |
| Naming | UPPER_SNAKE_CASE | snake_case |

```rust
const MAKER_FEE: f64 = 0.0002;  // Known at compile time

fn main() {
    let taker_fee = 0.0004;     // Computed at runtime

    // Cannot change a constant
    // MAKER_FEE = 0.0003;      // ERROR!

    // let without mut can't be changed either, but shadowing works
    let taker_fee = 0.0005;     // Creates a new variable
}
```

## Constants for Strategy Parameters

```rust
// Risk management parameters
const MAX_RISK_PER_TRADE: f64 = 0.02;    // 2% of deposit
const MAX_DAILY_LOSS: f64 = 0.06;        // 6% max daily loss
const MAX_OPEN_POSITIONS: u32 = 5;       // Max open positions

// Trading parameters
const DEFAULT_LEVERAGE: u32 = 10;
const STOP_LOSS_PERCENT: f64 = 0.02;     // 2% stop loss
const TAKE_PROFIT_PERCENT: f64 = 0.04;   // 4% take profit

fn main() {
    let deposit = 10000.0;

    let max_loss_per_trade = deposit * MAX_RISK_PER_TRADE;
    let max_daily_loss_amount = deposit * MAX_DAILY_LOSS;

    println!("=== Risk Management ===");
    println!("Deposit: {} USDT", deposit);
    println!("Max loss per trade: {} USDT ({}%)",
        max_loss_per_trade, MAX_RISK_PER_TRADE * 100.0);
    println!("Max daily loss: {} USDT ({}%)",
        max_daily_loss_amount, MAX_DAILY_LOSS * 100.0);
    println!("Max open positions: {}", MAX_OPEN_POSITIONS);
}
```

## Global Constants

Constants can be declared **outside functions**:

```rust
// Global constants are accessible everywhere
const EXCHANGE_NAME: &str = "Binance";
const API_RATE_LIMIT: u32 = 1200;        // Requests per minute
const WEBSOCKET_PING_INTERVAL: u32 = 30; // Seconds

fn print_exchange_info() {
    println!("Exchange: {}", EXCHANGE_NAME);
    println!("API limit: {} req/min", API_RATE_LIMIT);
}

fn print_websocket_info() {
    println!("Ping every {} seconds", WEBSOCKET_PING_INTERVAL);
}

fn main() {
    print_exchange_info();
    print_websocket_info();
}
```

## Computed Constants

Constants can use **constant expressions**:

```rust
const MINUTES_PER_HOUR: u32 = 60;
const HOURS_PER_DAY: u32 = 24;
const MINUTES_PER_DAY: u32 = MINUTES_PER_HOUR * HOURS_PER_DAY;

const TRADING_FEE_PERCENT: f64 = 0.1;
const TRADING_FEE: f64 = TRADING_FEE_PERCENT / 100.0;

// Use in position calculations
const DEFAULT_POSITION_USDT: f64 = 1000.0;
const FEE_FOR_ROUND_TRIP: f64 = DEFAULT_POSITION_USDT * TRADING_FEE * 2.0;

fn main() {
    println!("Minutes per day: {}", MINUTES_PER_DAY);
    println!("Fee: {}%", TRADING_FEE_PERCENT);
    println!("Fee for opening+closing {} USDT: {} USDT",
        DEFAULT_POSITION_USDT, FEE_FOR_ROUND_TRIP);
}
```

## Data Types in Constants

Type annotation is **required**:

```rust
const LEVERAGE: u32 = 10;              // u32
const FEE: f64 = 0.001;                // f64
const IS_TESTNET: bool = true;         // bool
const TICKER: &str = "BTCUSDT";        // &str (string slice)
const DECIMALS: usize = 8;             // usize

fn main() {
    println!("Ticker: {}", TICKER);
    println!("Leverage: {}x", LEVERAGE);
    println!("Testnet: {}", IS_TESTNET);
}
```

## Practical Example: Fee Calculator

```rust
// Different exchange fees
const BINANCE_SPOT_FEE: f64 = 0.001;
const BINANCE_FUTURES_FEE: f64 = 0.0004;
const BYBIT_FEE: f64 = 0.001;
const OKX_FEE: f64 = 0.0008;

// VIP discounts
const VIP1_DISCOUNT: f64 = 0.1;   // 10% discount
const VIP2_DISCOUNT: f64 = 0.2;   // 20% discount
const VIP3_DISCOUNT: f64 = 0.3;   // 30% discount

fn calculate_fee(volume: f64, base_fee: f64, discount: f64) -> f64 {
    let effective_fee = base_fee * (1.0 - discount);
    volume * effective_fee
}

fn main() {
    let trade_volume = 50000.0;  // 50,000 USDT

    println!("=== Fee Calculator ===");
    println!("Trade volume: {} USDT\n", trade_volume);

    // Without discount
    let binance_fee = calculate_fee(trade_volume, BINANCE_SPOT_FEE, 0.0);
    let bybit_fee = calculate_fee(trade_volume, BYBIT_FEE, 0.0);

    println!("Binance Spot: {} USDT", binance_fee);
    println!("Bybit: {} USDT", bybit_fee);

    // With VIP2 discount
    let binance_vip2 = calculate_fee(trade_volume, BINANCE_SPOT_FEE, VIP2_DISCOUNT);
    println!("\nBinance with VIP2 discount: {} USDT", binance_vip2);
    println!("Savings: {} USDT", binance_fee - binance_vip2);
}
```

## Practical Example: Trading Bot with Constants

```rust
// === BOT CONFIGURATION ===
const BOT_NAME: &str = "TrendFollower v1.0";
const EXCHANGE: &str = "Binance Futures";

// Trading parameters
const TRADING_PAIR: &str = "BTCUSDT";
const LEVERAGE: u32 = 10;
const POSITION_SIZE_USDT: f64 = 1000.0;

// Risk management
const STOP_LOSS_PERCENT: f64 = 0.015;     // 1.5%
const TAKE_PROFIT_PERCENT: f64 = 0.03;    // 3%
const MAX_TRADES_PER_DAY: u32 = 10;

// Fees
const TAKER_FEE: f64 = 0.0004;
const MAKER_FEE: f64 = 0.0002;

fn main() {
    println!("=== {} ===", BOT_NAME);
    println!("Exchange: {}", EXCHANGE);
    println!("Pair: {}", TRADING_PAIR);
    println!();

    // Calculate trade parameters
    let entry_price = 42000.0;
    let position_btc = POSITION_SIZE_USDT / entry_price;

    let stop_loss_price = entry_price * (1.0 - STOP_LOSS_PERCENT);
    let take_profit_price = entry_price * (1.0 + TAKE_PROFIT_PERCENT);

    println!("=== Trade Parameters ===");
    println!("Position size: {} USDT ({:.6} BTC)", POSITION_SIZE_USDT, position_btc);
    println!("Leverage: {}x", LEVERAGE);
    println!("Entry price: {} USDT", entry_price);
    println!("Stop loss: {} USDT ({:.1}%)", stop_loss_price, STOP_LOSS_PERCENT * 100.0);
    println!("Take profit: {} USDT ({:.1}%)", take_profit_price, TAKE_PROFIT_PERCENT * 100.0);

    // Calculate potential P&L
    let max_loss = POSITION_SIZE_USDT * STOP_LOSS_PERCENT * LEVERAGE as f64;
    let max_profit = POSITION_SIZE_USDT * TAKE_PROFIT_PERCENT * LEVERAGE as f64;

    // Account for fees (entry + exit)
    let total_fee = POSITION_SIZE_USDT * TAKER_FEE * 2.0;

    println!();
    println!("=== Potential P&L ===");
    println!("Max loss: -{:.2} USDT", max_loss);
    println!("Max profit: +{:.2} USDT", max_profit);
    println!("Fee (round trip): {:.2} USDT", total_fee);
    println!("Net max profit: +{:.2} USDT", max_profit - total_fee);
}
```

## static vs const

There's also `static`, but use `const` for simple values:

```rust
const TRADING_FEE: f64 = 0.001;     // Recommended for simple values
static EXCHANGE: &str = "Binance"; // Has a fixed memory address

fn main() {
    println!("Fee: {}", TRADING_FEE);
    println!("Exchange: {}", EXCHANGE);
}
```

**When to use `static`:**
- When you need a fixed memory address
- For mutable global data (`static mut`, unsafe)
- For large data arrays

**For most trading use cases, use `const`.**

## What We Learned

| Concept | Description |
|---------|-------------|
| `const` | Constant known at compile time |
| UPPER_SNAKE_CASE | Naming convention for constants |
| Global constants | Accessible in all functions |
| Required type | Constants require type annotation |
| Constant expressions | Can compute based on other constants |

## Exercises

### Exercise 1: Exchange Configuration

Create a set of constants for different exchanges:

```rust
// Create constants for:
// - Exchange names
// - Maker/taker fees
// - API limits
// - Minimum order sizes

fn main() {
    // Print configuration for each exchange
}
```

### Exercise 2: Position Calculator

Using constants, create a position size calculator:

```rust
const MAX_RISK_PERCENT: f64 = 0.02;  // 2% risk
const LEVERAGE: u32 = 10;

fn main() {
    let deposit = 5000.0;
    let entry_price = 42000.0;
    let stop_loss_price = 41500.0;

    // Calculate:
    // 1. Maximum risk in USDT
    // 2. Position size for given stop loss
    // 3. Amount of BTC in position
}
```

### Exercise 3: Fee Table

```rust
// Create constants for different VIP levels
// Print a fee table for volumes:
// 10,000 / 50,000 / 100,000 / 500,000 USDT
```

## Homework

1. **Create a configuration file for your trading bot:**
   - Risk management parameters (max loss, position size)
   - Trading parameters (leverage, stop loss, take profit)
   - Exchange parameters (fees, limits)

2. **Calculate break-even:**
   - At what profit does a trade cover the fee?
   - Create a minimum price movement calculator

3. **Compare exchanges:**
   - Create constants for 3-4 exchanges
   - Calculate fees for 100 trades of 1000 USDT each
   - Determine the most cost-effective exchange

## Navigation

[← Previous day](../025-statements-expressions/en.md) | [Next day →](../027-shadowing/en.md)
