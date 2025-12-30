# Day 13: Functions — Calculating Trade Profit

## Trading Analogy

In trading, there are many repetitive calculations:
- Trade profit calculation
- Position size calculation
- Indicator computation
- Entry condition checks

Instead of writing the same code over and over, we extract it into **functions**. A function is a named block of code that can be called many times.

## Simplest Function

```rust
fn main() {
    say_hello();
    say_hello();
    say_hello();
}

fn say_hello() {
    println!("Hello, Trader!");
}
```

- `fn` — keyword for declaring a function
- `say_hello` — function name (snake_case)
- `()` — parameters (empty for now)
- `{}` — function body

## Function with Parameters

```rust
fn main() {
    print_price("BTC", 42000.0);
    print_price("ETH", 2500.0);
    print_price("SOL", 95.0);
}

fn print_price(symbol: &str, price: f64) {
    println!("{}: ${:.2}", symbol, price);
}
```

Parameters are specified with types: `name: type`

## Function with Return Value

```rust
fn main() {
    let profit = calculate_profit(42000.0, 43500.0, 0.5);
    println!("Profit: ${:.2}", profit);
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

- `-> f64` — return type
- The last expression WITHOUT semicolon is the return value

## return for Early Exit

```rust
fn main() {
    println!("Profit%: {:.2}%", calculate_profit_percent(42000.0, 43500.0));
    println!("Profit%: {:.2}%", calculate_profit_percent(42000.0, 0.0));
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    // Protection against division by zero
    if entry == 0.0 {
        return 0.0;
    }

    ((exit - entry) / entry) * 100.0
}
```

## Multiple Functions

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;

    let gross_profit = calculate_gross_profit(entry, exit, quantity);
    let fees = calculate_fees(entry, exit, quantity, fee_percent);
    let net_profit = calculate_net_profit(gross_profit, fees);

    println!("Gross Profit: ${:.2}", gross_profit);
    println!("Fees: ${:.2}", fees);
    println!("Net Profit: ${:.2}", net_profit);
}

fn calculate_gross_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_fees(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    let entry_value = entry * quantity;
    let exit_value = exit * quantity;
    (entry_value + exit_value) * (fee_percent / 100.0)
}

fn calculate_net_profit(gross: f64, fees: f64) -> f64 {
    gross - fees
}
```

## Functions Calling Other Functions

```rust
fn main() {
    let result = full_trade_analysis(42000.0, 43500.0, 0.5, 0.1);
    println!("Net profit: ${:.2}", result);
}

fn full_trade_analysis(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    let gross = calculate_gross_profit(entry, exit, quantity);
    let fees = calculate_fees(entry, exit, quantity, fee_percent);
    calculate_net_profit(gross, fees)
}

fn calculate_gross_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_fees(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    (entry * quantity + exit * quantity) * (fee_percent / 100.0)
}

fn calculate_net_profit(gross: f64, fees: f64) -> f64 {
    gross - fees
}
```

## Documenting Functions

```rust
/// Calculates position size based on risk management.
///
/// # Arguments
///
/// * `balance` - Available balance
/// * `risk_percent` - Risk percentage per trade (e.g., 2.0 for 2%)
/// * `entry_price` - Entry price
/// * `stop_loss` - Stop-loss price
///
/// # Returns
///
/// Number of asset units to buy
///
/// # Example
///
/// ```
/// let size = calculate_position_size(10000.0, 2.0, 42000.0, 41000.0);
/// assert_eq!(size, 0.2);
/// ```
fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64
) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = (entry_price - stop_loss).abs();

    if price_risk == 0.0 {
        return 0.0;
    }

    risk_amount / price_risk
}

fn main() {
    let size = calculate_position_size(10000.0, 2.0, 42000.0, 41000.0);
    println!("Position size: {} BTC", size);
}
```

## Practical Example: Trading Calculator

```rust
fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       TRADING CALCULATOR              ║");
    println!("╚═══════════════════════════════════════╝\n");

    let balance = 10000.0;
    let risk_percent = 2.0;
    let entry = 42000.0;
    let stop_loss = 41000.0;
    let take_profit = 44000.0;
    let fee_percent = 0.1;

    // Position size calculation
    let position_size = calculate_position_size(balance, risk_percent, entry, stop_loss);
    let position_value = position_size * entry;

    // Risk and profit calculations
    let max_loss = calculate_pnl(entry, stop_loss, position_size);
    let max_profit = calculate_pnl(entry, take_profit, position_size);
    let risk_reward = calculate_risk_reward(entry, stop_loss, take_profit);

    // Fees
    let entry_fee = calculate_fee(position_value, fee_percent);
    let exit_fee_loss = calculate_fee(position_size * stop_loss, fee_percent);
    let exit_fee_profit = calculate_fee(position_size * take_profit, fee_percent);

    // Net result
    let net_loss = max_loss - entry_fee - exit_fee_loss;
    let net_profit = max_profit - entry_fee - exit_fee_profit;

    // Output
    print_section("INPUT DATA");
    println!("Balance:      ${:.2}", balance);
    println!("Risk:         {:.1}%", risk_percent);
    println!("Entry:        ${:.2}", entry);
    println!("Stop Loss:    ${:.2}", stop_loss);
    println!("Take Profit:  ${:.2}", take_profit);

    print_section("POSITION");
    println!("Size:         {:.8} BTC", position_size);
    println!("Value:        ${:.2}", position_value);

    print_section("RISK/REWARD");
    println!("Max Loss:     ${:.2}", max_loss);
    println!("Max Profit:   ${:.2}", max_profit);
    println!("R:R Ratio:    1:{:.2}", risk_reward);

    print_section("FEES (@ {:.1}%)", fee_percent);
    println!("Entry fee:    ${:.2}", entry_fee);
    println!("Exit (loss):  ${:.2}", exit_fee_loss);
    println!("Exit (profit):${:.2}", exit_fee_profit);

    print_section("NET RESULT");
    println!("Net Loss:     ${:.2}", net_loss);
    println!("Net Profit:   ${:.2}", net_profit);
}

fn calculate_position_size(balance: f64, risk_percent: f64, entry: f64, stop_loss: f64) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = (entry - stop_loss).abs();
    if price_risk == 0.0 { 0.0 } else { risk_amount / price_risk }
}

fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_risk_reward(entry: f64, stop_loss: f64, take_profit: f64) -> f64 {
    let risk = (entry - stop_loss).abs();
    let reward = (take_profit - entry).abs();
    if risk == 0.0 { 0.0 } else { reward / risk }
}

fn calculate_fee(value: f64, fee_percent: f64) -> f64 {
    value * (fee_percent / 100.0)
}

fn print_section(title: &str) {
    println!("\n--- {} ---", title);
}
```

## Expressions vs Statements

```rust
fn main() {
    // Statement - doesn't return a value
    let x = 5;  // This is a statement

    // Expression - returns a value
    let y = {
        let temp = 3;
        temp + 1  // This is an expression (no ;)
    };

    println!("y = {}", y);  // 4

    // Same in functions
    let result = add(2, 3);
    println!("result = {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b  // Expression - gets returned
}

fn add_with_return(a: i32, b: i32) -> i32 {
    return a + b;  // Explicit return
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `fn name()` | Function declaration |
| `fn name(x: T)` | Function with parameter |
| `-> T` | Return type |
| No `;` | Return expression |
| `return` | Early exit |
| `///` | Documentation comment |

## Homework

1. Write functions for complete trade analysis:
   - `calculate_position_value(price, quantity) -> f64`
   - `calculate_pnl(entry, exit, quantity) -> f64`
   - `calculate_pnl_percent(entry, exit) -> f64`
   - `is_profitable(entry, exit) -> bool`

2. Create a Kelly Criterion calculation function:
   `kelly(win_rate, avg_win, avg_loss) -> f64`

3. Write a price formatting function:
   `format_price(price, decimals) -> String`

4. Implement an order validation function:
   `is_valid_order(price, quantity, balance) -> bool`

## Navigation

[← Previous day](../012-arrays-closing-prices/en.md) | [Next day →](../014-function-parameters/en.md)
