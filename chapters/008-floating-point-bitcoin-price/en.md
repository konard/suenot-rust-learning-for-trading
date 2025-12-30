# Day 8: Floating Point — Precise Bitcoin Price

## Trading Analogy

Bitcoin trades with precision up to 8 decimal places:
- BTC: 42156.**78901234**
- ETH: 2250.**123456**
- USDT: 1.**0001**

To work with such numbers, we need **floating point numbers** (floats).

## Types in Rust

| Type | Size | Precision | When to use |
|------|------|-----------|-------------|
| `f32` | 32 bits | ~7 digits | Charts, fast calculations |
| `f64` | 64 bits | ~15 digits | Prices, money |

```rust
fn main() {
    let btc_price: f64 = 42156.78901234;
    let eth_price: f64 = 2250.50;

    println!("BTC: ${}", btc_price);
    println!("ETH: ${}", eth_price);
}
```

**Rule:** For money, ALWAYS use `f64`!

## Why f64, not f32?

```rust
fn main() {
    // Problem with f32
    let balance_f32: f32 = 1_000_000.0;
    let small_trade_f32: f32 = 0.01;
    let result_f32 = balance_f32 + small_trade_f32;

    println!("f32: {} + {} = {}", balance_f32, small_trade_f32, result_f32);
    // Might print: 1000000.0 (lost a penny!)

    // Solution with f64
    let balance_f64: f64 = 1_000_000.0;
    let small_trade_f64: f64 = 0.01;
    let result_f64 = balance_f64 + small_trade_f64;

    println!("f64: {} + {} = {}", balance_f64, small_trade_f64, result_f64);
    // Prints: 1000000.01 (exact!)
}
```

## Float Arithmetic

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    // Basic operations
    let profit = (exit_price - entry_price) * quantity;
    let percent_change = ((exit_price - entry_price) / entry_price) * 100.0;

    println!("Profit: ${:.2}", profit);
    println!("Change: {:.2}%", percent_change);
}
```

## Special Values

```rust
fn main() {
    // Infinity
    let infinity = f64::INFINITY;
    let neg_infinity = f64::NEG_INFINITY;

    // Division by zero
    let result = 1.0 / 0.0;  // Infinity

    // NaN (Not a Number)
    let nan = f64::NAN;
    let also_nan = 0.0 / 0.0;

    println!("Infinity: {}", infinity);
    println!("1/0 = {}", result);
    println!("NaN: {}", nan);

    // Checking for NaN
    println!("Is NaN: {}", nan.is_nan());
    println!("Is infinite: {}", infinity.is_infinite());
    println!("Is finite: {}", 42.0_f64.is_finite());
}
```

**In trading:** NaN can appear when dividing 0/0 (e.g., percentage of zero volume). Always check!

## Rounding

```rust
fn main() {
    let price = 42156.789012;

    // Different rounding methods
    println!("Original: {}", price);
    println!("floor (down): {}", price.floor());     // 42156.0
    println!("ceil (up): {}", price.ceil());         // 42157.0
    println!("round (nearest): {}", price.round());  // 42157.0
    println!("trunc (truncate): {}", price.trunc()); // 42156.0

    // Round to N decimal places
    let rounded = (price * 100.0).round() / 100.0;
    println!("To 2 decimals: {}", rounded);  // 42156.79
}
```

## Output Formatting

```rust
fn main() {
    let btc_price = 42156.789012345;
    let eth_quantity = 1.23456789;

    // Decimal places
    println!("Price: ${:.2}", btc_price);      // $42156.79
    println!("Quantity: {:.8}", eth_quantity); // 1.23456789

    // Field width
    println!("Price: ${:>12.2}", btc_price);   // $    42156.79
    println!("Price: ${:<12.2}", btc_price);   // $42156.79

    // Zero padding
    println!("ID: {:08.2}", 42.5);             // 00042.50
}
```

## Mathematical Functions

```rust
fn main() {
    let price = 42000.0_f64;

    // Square root (for volatility)
    let sqrt = price.sqrt();
    println!("Square root: {}", sqrt);

    // Power
    let squared = price.powi(2);        // Integer power
    let powered = price.powf(1.5);      // Fractional power
    println!("Squared: {}", squared);

    // Logarithm (for log returns)
    let ln = price.ln();                // Natural
    let log10 = price.log10();          // Base 10
    println!("ln(price): {}", ln);

    // Exponential
    let exp = 0.05_f64.exp();           // e^0.05
    println!("e^0.05: {}", exp);

    // Absolute value
    let loss = -500.0_f64;
    println!("Loss: {}, Abs: {}", loss, loss.abs());

    // Min/max
    let a = 42000.0_f64;
    let b = 43000.0_f64;
    println!("Min: {}, Max: {}", a.min(b), a.max(b));
}
```

## Practical Example: Return Calculation

```rust
fn main() {
    let initial_price = 40000.0;
    let final_price = 42000.0;

    // Simple return
    let simple_return = (final_price - initial_price) / initial_price * 100.0;

    // Log return (better for finance)
    let log_return = (final_price / initial_price).ln() * 100.0;

    println!("Initial price: ${:.2}", initial_price);
    println!("Final price: ${:.2}", final_price);
    println!("Simple return: {:.2}%", simple_return);
    println!("Log return: {:.2}%", log_return);
}
```

## Practical Example: Volatility Calculation

```rust
fn main() {
    // Daily returns (in percent)
    let returns = [1.5, -0.8, 2.1, -1.2, 0.5, 1.8, -0.3];

    // Mean
    let sum: f64 = returns.iter().sum();
    let mean = sum / returns.len() as f64;

    // Variance
    let variance: f64 = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;

    // Standard deviation (volatility)
    let volatility = variance.sqrt();

    // Annual volatility (approximately 252 trading days)
    let annual_volatility = volatility * (252.0_f64).sqrt();

    println!("Mean return: {:.2}%", mean);
    println!("Daily volatility: {:.2}%", volatility);
    println!("Annual volatility: {:.2}%", annual_volatility);
}
```

## Float Comparison Problem

```rust
fn main() {
    let a = 0.1 + 0.2;
    let b = 0.3;

    // WRONG!
    if a == b {
        println!("Equal");
    } else {
        println!("Not equal: {} != {}", a, b);  // Surprise!
    }

    // RIGHT: comparison with tolerance
    let epsilon = 1e-10;
    if (a - b).abs() < epsilon {
        println!("Practically equal");
    }
}
```

**Analogy:** This is like comparing prices from different exchanges — they'll never be exactly equal, but can be "close enough".

## Practical Example: Trading Calculator

```rust
fn main() {
    // Input data
    let balance: f64 = 10_000.0;
    let risk_percent: f64 = 2.0;
    let entry_price: f64 = 42_000.0;
    let stop_loss: f64 = 41_000.0;
    let take_profit: f64 = 44_000.0;
    let fee_percent: f64 = 0.1;

    // Calculations
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = entry_price - stop_loss;
    let position_size = risk_amount / price_risk;
    let position_value = position_size * entry_price;

    let potential_loss = price_risk * position_size;
    let potential_profit = (take_profit - entry_price) * position_size;
    let risk_reward = potential_profit / potential_loss;

    let entry_fee = position_value * (fee_percent / 100.0);
    let exit_fee = position_size * take_profit * (fee_percent / 100.0);
    let total_fees = entry_fee + exit_fee;

    let net_profit = potential_profit - total_fees;

    // Output
    println!("╔══════════════════════════════════╗");
    println!("║      TRADING CALCULATOR          ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Balance:        ${:>14.2} ║", balance);
    println!("║ Risk:              {:>11.1}% ║", risk_percent);
    println!("║ Entry:          ${:>14.2} ║", entry_price);
    println!("║ Stop Loss:      ${:>14.2} ║", stop_loss);
    println!("║ Take Profit:    ${:>14.2} ║", take_profit);
    println!("╠══════════════════════════════════╣");
    println!("║ Position Size:   {:>14.8} ║", position_size);
    println!("║ Position Value: ${:>14.2} ║", position_value);
    println!("║ Potential Loss: ${:>14.2} ║", potential_loss);
    println!("║ Potential Profit:${:>13.2} ║", potential_profit);
    println!("║ Risk/Reward:      {:>13.2} ║", risk_reward);
    println!("║ Total Fees:     ${:>14.2} ║", total_fees);
    println!("║ Net Profit:     ${:>14.2} ║", net_profit);
    println!("╚══════════════════════════════════╝");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `f64` | Main type for money |
| `f32` | For fast calculations |
| Rounding | floor, ceil, round, trunc |
| Formatting | {:.2} for 2 decimals |
| Comparison | Use epsilon |

## Homework

1. Create a commission calculator for an exchange:
   - Maker fee: 0.1%
   - Taker fee: 0.2%
   - Calculate commission for any trade size

2. Calculate log return for a series of 10 prices

3. Implement a function to round to a specific number of decimal places

4. Write a check: is a number finite and not NaN

## Navigation

[← Previous day](../007-integers-counting-shares/en.md) | [Next day →](../009-booleans-market-status/en.md)
