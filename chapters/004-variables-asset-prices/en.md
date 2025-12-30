# Day 4: Variables — Storing Asset Prices

## Trading Analogy

When you trade, you need to remember information:
- Entry price for the trade
- Number of coins purchased
- Current balance

In programming, we use **variables** for this — they're like named memory cells where we store data.

**Variable = labeled envelope with money inside**

## Declaring Variables

In Rust, variables are created with `let`:

```rust
fn main() {
    let btc_price = 42000;
    println!("BTC price: {} USD", btc_price);
}
```

Let's break it down:
- `let` — keyword for "create a variable"
- `btc_price` — variable name (our "envelope")
- `=` — assign a value
- `42000` — the value inside

## Naming Rules

```rust
fn main() {
    // Good names (snake_case)
    let bitcoin_price = 42000;
    let entry_price = 41500;
    let position_size = 0.5;

    // Bad names
    // let 1price = 100;  // can't start with a digit
    // let my-price = 100; // can't use hyphens
}
```

In Rust, we use `snake_case`: words_separated_by_underscores.

**Analogy:** This is like stock tickers — short, clear names. `BTC` is better than `Bitcoin_Cryptocurrency_Token_v2`.

## Unused Variables

If a variable isn't used, Rust will warn you:

```rust
fn main() {
    let unused_variable = 100;  // Warning: unused variable
}
```

To tell Rust "I know I'm not using this", add `_`:

```rust
fn main() {
    let _unused_variable = 100;  // No warning
}
```

## Multiple Variables

```rust
fn main() {
    let symbol = "BTC/USDT";
    let entry_price = 42000.0;
    let current_price = 43500.0;
    let quantity = 0.5;

    println!("Symbol: {}", symbol);
    println!("Entry price: {}", entry_price);
    println!("Current price: {}", current_price);
    println!("Quantity: {}", quantity);
}
```

## Type Inference

Rust is smart — it figures out the variable type:

```rust
fn main() {
    let price = 42000;      // Rust understands: this is an integer
    let amount = 0.5;       // Rust understands: this is a float
    let symbol = "BTC";     // Rust understands: this is a string
    let is_long = true;     // Rust understands: this is a boolean
}
```

## Explicit Type Annotation

Sometimes you need to specify the type explicitly:

```rust
fn main() {
    let price: f64 = 42000.0;    // 64-bit floating point
    let quantity: f32 = 0.5;     // 32-bit floating point
    let shares: i32 = 100;       // 32-bit signed integer
    let volume: u64 = 1000000;   // 64-bit unsigned integer
}
```

**Analogy:**
- `f64` — large safe for precise amounts (BTC prices with 8 decimal places)
- `f32` — smaller safe, takes less space
- `i32` — safe for numbers including negatives (profit/loss)
- `u64` — huge safe for positive numbers only (trading volume)

## Variables in Expressions

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    let profit = (exit_price - entry_price) * quantity;

    println!("Profit: {} USDT", profit);
}
```

## Using Variables Multiple Times

```rust
fn main() {
    let btc_price = 42000.0;

    println!("BTC price: {}", btc_price);
    println!("Price of 2 BTC: {}", btc_price * 2.0);
    println!("Price of 0.5 BTC: {}", btc_price * 0.5);

    // btc_price is still 42000.0 — we didn't change it!
}
```

## Practical Example: Position Calculator

```rust
fn main() {
    // Input data
    let balance = 10000.0;          // Balance in USDT
    let risk_percent = 2.0;         // Risk per trade (%)
    let entry_price = 42000.0;      // Entry price
    let stop_loss = 41000.0;        // Stop loss

    // Calculations
    let risk_amount = balance * (risk_percent / 100.0);
    let price_difference = entry_price - stop_loss;
    let position_size = risk_amount / price_difference;

    // Output
    println!("=== Position Calculator ===");
    println!("Balance: {} USDT", balance);
    println!("Risk: {}% = {} USDT", risk_percent, risk_amount);
    println!("Position size: {} BTC", position_size);
    println!("Position value: {} USDT", position_size * entry_price);
}
```

Output:
```
=== Position Calculator ===
Balance: 10000 USDT
Risk: 2% = 200 USDT
Position size: 0.2 BTC
Position value: 8400 USDT
```

## What We Learned

| Concept | Example | Description |
|---------|---------|-------------|
| let | `let price = 42000;` | Create a variable |
| snake_case | `entry_price` | Naming style |
| Type inference | `let x = 5;` | Rust figures out the type |
| Explicit type | `let x: f64 = 5.0;` | We specify the type |
| `_` | `let _unused = 0;` | Suppress warning |

## Homework

1. Create variables to store:
   - Cryptocurrency name (string)
   - Purchase price (floating point)
   - Number of coins (floating point)
   - Purchase date (string)

2. Calculate and print:
   - Total purchase value
   - Value if price increases by 10%
   - Value if price decreases by 10%

3. Experiment with types: try using `f32` instead of `f64`

## Navigation

[← Previous day](../003-cargo-project-manager/en.md) | [Next day →](../005-immutability-locked-price/en.md)
