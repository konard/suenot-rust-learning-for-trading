# Day 5: Immutability — The Deal Price is Locked

## Trading Analogy

When you execute a trade on an exchange, the price is **locked** at the moment of execution. You bought BTC at $42,000 — this entry price will never change. It's recorded in history.

In Rust, variables work the same way — by default they are **immutable**. This protects against accidental data changes.

## Immutable by Default

```rust
fn main() {
    let btc_price = 42000;
    btc_price = 43000;  // ERROR! Cannot change
}
```

The compiler will say:
```
error[E0384]: cannot assign twice to immutable variable `btc_price`
```

**Analogy:** This is like trying to change the price of an already executed trade in history. Not allowed!

## Why Immutability?

In trading, this is critically important:

```rust
fn main() {
    let entry_price = 42000.0;
    let stop_loss = entry_price - 1000.0;  // 41000

    // ... lots of code ...

    // Imagine someone accidentally wrote:
    // entry_price = 50000.0;

    // Now stop_loss is calculated wrong!
    // In a real bot, this could cost money!
}
```

Rust **protects** you from such errors at compile time.

## Mutable Variables with `mut`

When change is **actually needed**, use `mut`:

```rust
fn main() {
    let mut balance = 10000.0;
    println!("Initial balance: {} USDT", balance);

    // Made a profitable trade
    balance = balance + 500.0;
    println!("After trade: {} USDT", balance);

    // Another trade
    balance = balance - 200.0;
    println!("After second trade: {} USDT", balance);
}
```

Output:
```
Initial balance: 10000 USDT
After trade: 10500 USDT
After second trade: 10300 USDT
```

## When to Use mut?

### Use `mut` when:
- Balance changes after each trade
- Current price updates in real-time
- Trade counter increases
- Position opens/closes

### DON'T use `mut` when:
- Entry price for a trade (fixed)
- Exchange commission (usually constant)
- Initial deposit (for reporting)
- Strategy parameters (for backtesting)

## Practical Examples

### Example 1: Price Updates

```rust
fn main() {
    let mut current_price = 42000.0;

    println!("Price: {}", current_price);

    // Price changed
    current_price = 42150.0;
    println!("New price: {}", current_price);

    current_price = 42300.0;
    println!("Even newer: {}", current_price);
}
```

### Example 2: Trade Counting

```rust
fn main() {
    let mut trade_count = 0;

    // Made a trade
    trade_count = trade_count + 1;
    println!("Trades: {}", trade_count);

    // Another trade
    trade_count = trade_count + 1;
    println!("Trades: {}", trade_count);

    // And another
    trade_count = trade_count + 1;
    println!("Total trades today: {}", trade_count);
}
```

### Example 3: Trade Simulation

```rust
fn main() {
    // Fixed parameters (immutable)
    let entry_price = 42000.0;
    let take_profit = 43000.0;
    let stop_loss = 41500.0;
    let position_size = 0.5;

    // Mutable state
    let mut current_price = entry_price;
    let mut pnl = 0.0;

    println!("=== Trade Simulation ===");
    println!("Entry: {} USDT", entry_price);
    println!("Take profit: {} USDT", take_profit);
    println!("Stop loss: {} USDT", stop_loss);

    // Price moves up
    current_price = 42500.0;
    pnl = (current_price - entry_price) * position_size;
    println!("\nPrice: {} | PnL: {} USDT", current_price, pnl);

    current_price = 43000.0;
    pnl = (current_price - entry_price) * position_size;
    println!("Price: {} | PnL: {} USDT", current_price, pnl);
    println!("Take profit reached!");
}
```

## Compound Assignment Operators

Instead of `x = x + 1` you can write `x += 1`:

```rust
fn main() {
    let mut balance = 10000.0;

    balance += 500.0;   // balance = balance + 500.0
    println!("After profit: {}", balance);

    balance -= 200.0;   // balance = balance - 200.0
    println!("After loss: {}", balance);

    balance *= 1.1;     // balance = balance * 1.1 (increased by 10%)
    println!("After growth: {}", balance);

    balance /= 2.0;     // balance = balance / 2.0
    println!("Half: {}", balance);
}
```

## What You Can't Change

You cannot change a variable's type:

```rust
fn main() {
    let mut price = 42000;    // integer
    price = 42000.5;          // ERROR! Can't assign float
}
```

For this, there's **shadowing** (next topic).

## Practical Example: Simple Trading Bot

```rust
fn main() {
    // Strategy constants
    let initial_balance = 10000.0;
    let risk_per_trade = 0.02;  // 2%

    // Mutable state
    let mut balance = initial_balance;
    let mut total_trades = 0;
    let mut winning_trades = 0;

    println!("=== Trading Bot ===");
    println!("Initial balance: {} USDT\n", balance);

    // Trade 1: profitable
    let trade_result = 150.0;
    balance += trade_result;
    total_trades += 1;
    winning_trades += 1;
    println!("Trade {}: +{} USDT | Balance: {}", total_trades, trade_result, balance);

    // Trade 2: loss
    let trade_result = -80.0;
    balance += trade_result;
    total_trades += 1;
    println!("Trade {}: {} USDT | Balance: {}", total_trades, trade_result, balance);

    // Trade 3: profitable
    let trade_result = 200.0;
    balance += trade_result;
    total_trades += 1;
    winning_trades += 1;
    println!("Trade {}: +{} USDT | Balance: {}", total_trades, trade_result, balance);

    // Summary
    println!("\n=== Summary ===");
    println!("Total trades: {}", total_trades);
    println!("Winning trades: {}", winning_trades);
    println!("Initial balance: {} USDT", initial_balance);
    println!("Final balance: {} USDT", balance);
    println!("Profit: {} USDT", balance - initial_balance);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Immutable | By default, variables cannot be changed |
| `mut` | Makes a variable mutable |
| `+=`, `-=` | Compound operators |
| Safety | Protection against accidental changes |

## Homework

1. Create a simulation of 5 trades:
   - Initial balance: 5000 USDT (immutable)
   - Current balance: changes after each trade
   - Trade counter: increases

2. Try to change an immutable variable — read the error message

3. Add tracking of:
   - Total winning trades
   - Total losing trades
   - Maximum profit from a single trade

## Navigation

[← Previous day](../004-variables-asset-prices/en.md) | [Next day →](../006-data-types/en.md)
