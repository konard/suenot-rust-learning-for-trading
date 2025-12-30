# Day 14: Function Parameters — Passing Entry and Exit Price

## Trading Analogy

When you calculate trade profit, you need **input data**:
- Entry price
- Exit price
- Volume
- Commission

This data is passed to a function as **parameters**. The function uses them for calculations and returns a result.

## Parameter Basics

```rust
fn main() {
    // Pass parameters to function
    let result = calculate_pnl(42000.0, 43500.0, 0.5);
    println!("PnL: ${:.2}", result);
}

// Declare parameters with types
fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

Each parameter **must** have a type!

## Pass by Value

Simple types (numbers, bool) are passed **by value** — a copy is created:

```rust
fn main() {
    let price = 42000.0;
    double_price(price);
    println!("Original price: {}", price);  // Still 42000.0!
}

fn double_price(mut p: f64) {
    p = p * 2.0;
    println!("Doubled: {}", p);  // 84000.0
}
```

Changes inside the function do NOT affect the original.

## Pass by Reference

For larger data, we use references:

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Pass reference to array
    let avg = calculate_average(&prices);
    println!("Average: {:.2}", avg);
}

// &[f64] — reference to slice (any size)
fn calculate_average(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    let sum: f64 = prices.iter().sum();
    sum / prices.len() as f64
}
```

## Mutable References

When you need to modify data inside a function:

```rust
fn main() {
    let mut balance = 10000.0;
    println!("Before: {}", balance);

    add_profit(&mut balance, 500.0);
    println!("After profit: {}", balance);

    add_profit(&mut balance, -200.0);  // Loss as negative profit
    println!("After loss: {}", balance);
}

// &mut f64 — mutable reference
fn add_profit(balance: &mut f64, profit: f64) {
    *balance += profit;  // * to dereference
}
```

## Different Ways to Pass

```rust
fn main() {
    // 1. By value (for simple types)
    let x = 42;
    takes_value(x);
    println!("x still exists: {}", x);

    // 2. By reference (for reading)
    let prices = vec![1.0, 2.0, 3.0];
    takes_reference(&prices);
    println!("prices still exists: {:?}", prices);

    // 3. By mutable reference (for modification)
    let mut balance = 100.0;
    takes_mut_reference(&mut balance);
    println!("balance changed: {}", balance);

    // 4. Move (take ownership)
    let data = String::from("BTC");
    takes_ownership(data);
    // println!("{}", data);  // ERROR! data has been moved
}

fn takes_value(n: i32) {
    println!("Got value: {}", n);
}

fn takes_reference(v: &Vec<f64>) {
    println!("Got reference, len: {}", v.len());
}

fn takes_mut_reference(b: &mut f64) {
    *b += 50.0;
}

fn takes_ownership(s: String) {
    println!("Got ownership of: {}", s);
}
```

## Returning Multiple Values via Tuple

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42150.0];

    let (min, max, avg) = analyze_prices(&prices);

    println!("Min: {:.2}", min);
    println!("Max: {:.2}", max);
    println!("Avg: {:.2}", avg);
    println!("Range: {:.2}", max - min);
}

fn analyze_prices(prices: &[f64]) -> (f64, f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let mut sum = 0.0;

    for &price in prices {
        if price < min { min = price; }
        if price > max { max = price; }
        sum += price;
    }

    let avg = if prices.is_empty() { 0.0 } else { sum / prices.len() as f64 };

    (min, max, avg)
}
```

## Default Parameter Values (via Wrapper)

Rust doesn't have default parameter values, but you can use a pattern:

```rust
fn main() {
    // Option 1: full parameter set
    let pnl1 = calculate_net_pnl(42000.0, 43500.0, 0.5, 0.1);

    // Option 2: simplified version (with default fee)
    let pnl2 = calculate_net_pnl_default(42000.0, 43500.0, 0.5);

    println!("With custom fee: {:.2}", pnl1);
    println!("With default fee: {:.2}", pnl2);
}

fn calculate_net_pnl(entry: f64, exit: f64, qty: f64, fee_percent: f64) -> f64 {
    let gross = (exit - entry) * qty;
    let fee = (entry * qty + exit * qty) * (fee_percent / 100.0);
    gross - fee
}

// Wrapper with default value
fn calculate_net_pnl_default(entry: f64, exit: f64, qty: f64) -> f64 {
    calculate_net_pnl(entry, exit, qty, 0.1)  // 0.1% by default
}
```

## Practical Example: Complete Trade Calculation

```rust
fn main() {
    // Trade data
    let symbol = "BTC/USDT";
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;
    let is_long = true;

    // Calculations
    let gross_pnl = calculate_gross_pnl(entry_price, exit_price, quantity, is_long);
    let (entry_fee, exit_fee, total_fees) = calculate_fees(
        entry_price, exit_price, quantity, fee_percent
    );
    let net_pnl = gross_pnl - total_fees;
    let roi = calculate_roi(entry_price, quantity, net_pnl);

    // Output
    print_trade_summary(
        symbol,
        entry_price,
        exit_price,
        quantity,
        is_long,
        gross_pnl,
        total_fees,
        net_pnl,
        roi
    );
}

fn calculate_gross_pnl(entry: f64, exit: f64, qty: f64, is_long: bool) -> f64 {
    if is_long {
        (exit - entry) * qty
    } else {
        (entry - exit) * qty  // Short: profit on decline
    }
}

fn calculate_fees(entry: f64, exit: f64, qty: f64, fee_pct: f64) -> (f64, f64, f64) {
    let entry_fee = entry * qty * (fee_pct / 100.0);
    let exit_fee = exit * qty * (fee_pct / 100.0);
    (entry_fee, exit_fee, entry_fee + exit_fee)
}

fn calculate_roi(entry: f64, qty: f64, net_pnl: f64) -> f64 {
    let investment = entry * qty;
    if investment == 0.0 { 0.0 } else { (net_pnl / investment) * 100.0 }
}

fn print_trade_summary(
    symbol: &str,
    entry: f64,
    exit: f64,
    qty: f64,
    is_long: bool,
    gross: f64,
    fees: f64,
    net: f64,
    roi: f64
) {
    println!("╔══════════════════════════════════════╗");
    println!("║         TRADE SUMMARY                ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Symbol:     {:<24} ║", symbol);
    println!("║ Direction:  {:<24} ║", if is_long { "LONG" } else { "SHORT" });
    println!("║ Entry:      ${:<23.2} ║", entry);
    println!("║ Exit:       ${:<23.2} ║", exit);
    println!("║ Quantity:   {:<24.8} ║", qty);
    println!("╠══════════════════════════════════════╣");
    println!("║ Gross PnL:  ${:<23.2} ║", gross);
    println!("║ Fees:       ${:<23.2} ║", fees);
    println!("║ Net PnL:    ${:<23.2} ║", net);
    println!("║ ROI:        {:<23.2}% ║", roi);
    println!("╚══════════════════════════════════════╝");
}
```

## Common Parameter Patterns

```rust
// Pattern 1: Price and quantity
fn process_order(price: f64, quantity: f64) -> f64 {
    price * quantity
}

// Pattern 2: OHLC data
fn analyze_candle(open: f64, high: f64, low: f64, close: f64) -> bool {
    close > open  // Bullish?
}

// Pattern 3: Price array
fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Some(sum / period as f64)
}

// Pattern 4: Balance modification
fn apply_trade_result(balance: &mut f64, pnl: f64) {
    *balance += pnl;
}

fn main() {
    // Using all patterns
    let value = process_order(42000.0, 0.5);
    println!("Order value: {}", value);

    let is_bullish = analyze_candle(42000.0, 42500.0, 41800.0, 42300.0);
    println!("Bullish: {}", is_bullish);

    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    if let Some(sma) = calculate_sma(&prices, 3) {
        println!("SMA-3: {:.2}", sma);
    }

    let mut balance = 10000.0;
    apply_trade_result(&mut balance, 500.0);
    println!("New balance: {}", balance);
}
```

## What We Learned

| Passing | Syntax | When to Use |
|---------|--------|-------------|
| By value | `fn f(x: i32)` | Simple types (Copy) |
| By reference | `fn f(x: &T)` | Read only |
| Mutable reference | `fn f(x: &mut T)` | Need to modify |
| Move | `fn f(x: String)` | Transfer ownership |

## Homework

1. Write a function `update_portfolio(portfolio: &mut Vec<f64>, trade_result: f64)` that adds trade result to portfolio

2. Create a function `validate_trade_params(entry: f64, stop: f64, take: f64) -> bool` that validates parameter logic

3. Implement a function `split_position(total: f64, parts: usize) -> Vec<f64>` that divides a position into parts

4. Write a function `merge_candles(candles: &[(f64, f64, f64, f64)]) -> (f64, f64, f64, f64)` that merges multiple candles into one

## Navigation

[← Previous day](../013-functions-trade-profit/en.md) | [Next day →](../015-return-values-pnl/en.md)
