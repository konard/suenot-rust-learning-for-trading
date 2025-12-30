# Day 16: Comments — Documenting Trading Logic

## Trading Analogy

Imagine a trader's journal. Every trade is recorded with notes: **why** you opened the position, **what** signal you saw, **what** went wrong. Without these notes, you won't remember your decision logic a month later. Comments in code are like a journal for programmers: they explain **why** the code was written, not **what** it does.

## Single-Line Comments

Start with `//` and continue to the end of the line:

```rust
fn main() {
    // Current Bitcoin price
    let btc_price = 42000.0;

    let position_size = 0.5;  // Position size in BTC

    // Calculate position value
    let position_value = btc_price * position_size;

    println!("Position value: ${}", position_value);
}
```

## Multi-Line Comments

Start with `/*` and end with `*/`:

```rust
fn main() {
    /*
     * Scalping strategy:
     * 1. Enter on level breakout
     * 2. Stop-loss 0.5% from entry
     * 3. Take-profit 1% from entry
     */

    let entry_price = 42000.0;
    let stop_loss = entry_price * 0.995;    // -0.5%
    let take_profit = entry_price * 1.01;   // +1%

    println!("Entry: {}, SL: {}, TP: {}", entry_price, stop_loss, take_profit);
}
```

## Documentation Comments

### For functions and structs: `///`

Generate documentation via `cargo doc`:

```rust
/// Calculates the profit/loss of a trade.
///
/// # Arguments
///
/// * `entry_price` - Price at position entry
/// * `exit_price` - Price at position exit
/// * `quantity` - Amount of asset
///
/// # Returns
///
/// Profit (positive number) or loss (negative number)
///
/// # Example
///
/// ```
/// let pnl = calculate_pnl(42000.0, 43500.0, 0.5);
/// assert!(pnl > 0.0);
/// ```
fn calculate_pnl(entry_price: f64, exit_price: f64, quantity: f64) -> f64 {
    (exit_price - entry_price) * quantity
}

fn main() {
    let pnl = calculate_pnl(42000.0, 43500.0, 0.5);
    println!("PnL: ${:.2}", pnl);
}
```

### For modules: `//!`

```rust
//! # Risk Management Module
//!
//! This module contains functions for:
//! - Position size calculation
//! - Stop-loss determination
//! - Maximum risk computation

/// Calculates position size based on risk.
fn calculate_position_size(balance: f64, risk_percent: f64, stop_distance: f64) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    risk_amount / stop_distance
}

fn main() {
    let size = calculate_position_size(10000.0, 2.0, 500.0);
    println!("Position size: {:.4}", size);
}
```

## Comments in Trading Code

### Explaining Formulas

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // SMA (Simple Moving Average) = sum of prices / number of periods
    let sma: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

    // RSI = 100 - (100 / (1 + RS))
    // where RS = average gain / average loss
    // Simplified calculation for demonstration
    let avg_gain = 75.0;
    let avg_loss = 25.0;
    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    println!("SMA: {:.2}, RSI: {:.2}", sma, rsi);
}
```

### Explaining Trading Logic

```rust
fn main() {
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let rsi = 65.0;

    // Conditions for long:
    // 1. Price above SMA-20 (uptrend)
    // 2. RSI between 30 and 70 (not overbought/oversold)
    // 3. RSI rising (momentum)

    let above_sma = current_price > sma_20;
    let rsi_neutral = rsi > 30.0 && rsi < 70.0;

    if above_sma && rsi_neutral {
        println!("LONG signal");
    } else {
        println!("No signal");
    }
}
```

### TODO and FIXME Markers

```rust
fn main() {
    let balance = 10000.0;
    let risk_percent = 2.0;

    // TODO: add maximum position size check
    // TODO: account for exchange commission

    let position_size = balance * (risk_percent / 100.0);

    // FIXME: division by zero when stop-loss is zero
    let stop_distance = 500.0;
    let lots = position_size / stop_distance;

    // HACK: temporary solution until we have exchange API
    let simulated_fill_price = 42000.0 * 1.001;  // slippage 0.1%

    println!("Lots: {:.4}, Fill: {:.2}", lots, simulated_fill_price);
}
```

## When to Comment

### Good Comments

```rust
fn main() {
    // Sharpe ratio > 1 is considered good,
    // > 2 is excellent, > 3 is exceptional
    let sharpe_ratio = 1.85;

    // Maximum drawdown should not exceed 20%
    // according to our risk parameters
    let max_drawdown = 0.15;

    // Using 252 trading days per year
    // (standard for US markets)
    let trading_days = 252;

    let annual_return = 0.25;
    let annualized_volatility = annual_return / sharpe_ratio;

    println!("Volatility: {:.2}%", annualized_volatility * 100.0);
}
```

### Bad Comments (Obvious)

```rust
fn main() {
    // Declare variable price
    let price = 42000.0;  // NOT NEEDED!

    // Increase quantity by 1
    let mut quantity = 5;
    quantity += 1;  // NOT NEEDED!

    // Call println function
    println!("Price: {}, Qty: {}", price, quantity);  // NOT NEEDED!
}
```

### Better — Clear Names Instead of Comments

```rust
fn main() {
    // Bad:
    let p = 42000.0;  // price
    let q = 0.5;      // quantity
    let f = 0.001;    // fee

    // Good — names speak for themselves:
    let bitcoin_price = 42000.0;
    let position_quantity = 0.5;
    let exchange_fee_rate = 0.001;

    let total_with_fee = bitcoin_price * position_quantity * (1.0 + exchange_fee_rate);
    println!("Total: ${:.2}", total_with_fee);
}
```

## Documenting Trading Strategies

```rust
/// Moving Average Crossover Strategy.
///
/// # Strategy Logic
///
/// - **Buy signal**: fast MA crosses slow MA from below
/// - **Sell signal**: fast MA crosses slow MA from above
///
/// # Parameters
///
/// * `fast_ma` - Fast moving average value (e.g., SMA-10)
/// * `slow_ma` - Slow moving average value (e.g., SMA-50)
/// * `prev_fast_ma` - Previous fast MA value
/// * `prev_slow_ma` - Previous slow MA value
///
/// # Returns
///
/// * `1` - buy signal (BUY)
/// * `-1` - sell signal (SELL)
/// * `0` - no signal (HOLD)
///
/// # Example
///
/// ```
/// let signal = ma_crossover_signal(42100.0, 42000.0, 41900.0, 42000.0);
/// assert_eq!(signal, 1); // Buy: fast MA crossed slow MA from below
/// ```
fn ma_crossover_signal(fast_ma: f64, slow_ma: f64, prev_fast_ma: f64, prev_slow_ma: f64) -> i32 {
    let currently_above = fast_ma > slow_ma;
    let was_below = prev_fast_ma <= prev_slow_ma;

    let currently_below = fast_ma < slow_ma;
    let was_above = prev_fast_ma >= prev_slow_ma;

    if currently_above && was_below {
        1  // Bullish crossover — buy
    } else if currently_below && was_above {
        -1  // Bearish crossover — sell
    } else {
        0  // No crossover — hold
    }
}

fn main() {
    let signal = ma_crossover_signal(42100.0, 42000.0, 41900.0, 42000.0);

    match signal {
        1 => println!("Signal: BUY"),
        -1 => println!("Signal: SELL"),
        _ => println!("Signal: HOLD"),
    }
}
```

## Comments for Disabling Code

```rust
fn main() {
    let price = 42000.0;
    let quantity = 0.5;

    // Temporarily disabled for debugging
    // let with_leverage = apply_leverage(price, 10);

    /*
    Old fee calculation logic:
    let fee = price * quantity * 0.001;
    let total = price * quantity + fee;
    */

    // New logic with VIP level consideration
    let vip_fee_rate = 0.0005;
    let fee = price * quantity * vip_fee_rate;
    let total = price * quantity + fee;

    println!("Total with VIP fee: ${:.2}", total);
}
```

## Structuring Code with Sections

```rust
fn main() {
    // ============================================
    // CONFIGURATION
    // ============================================

    let initial_balance = 10000.0;
    let risk_per_trade = 0.02;
    let max_positions = 5;

    // ============================================
    // MARKET DATA
    // ============================================

    let btc_price = 42000.0;
    let eth_price = 2200.0;

    // ============================================
    // CALCULATIONS
    // ============================================

    let btc_position_value = initial_balance * risk_per_trade;
    let btc_quantity = btc_position_value / btc_price;

    // ============================================
    // OUTPUT RESULTS
    // ============================================

    println!("BTC position: {:.6} BTC (${:.2})", btc_quantity, btc_position_value);
}
```

## Practical Example: Fully Documented Position Calculator

```rust
//! # Position Size Calculator
//!
//! Module for calculating optimal position size
//! based on risk management.

/// Information about the calculated position.
struct PositionInfo {
    /// Position size in asset units
    size: f64,
    /// Position value in account currency
    value: f64,
    /// Risk in account currency
    risk_amount: f64,
    /// Potential profit
    potential_profit: f64,
    /// Risk/reward ratio
    risk_reward_ratio: f64,
}

/// Calculates position size using fixed percentage risk method.
///
/// # Formula
///
/// `position_size = (balance * risk_percent) / |entry - stop_loss|`
///
/// # Arguments
///
/// * `balance` - Account balance
/// * `risk_percent` - Risk percentage per trade (e.g., 2.0 for 2%)
/// * `entry_price` - Entry price
/// * `stop_loss` - Stop-loss level
/// * `take_profit` - Take-profit level
///
/// # Example
///
/// ```
/// let position = calculate_position(10000.0, 2.0, 42000.0, 41500.0, 43000.0);
/// println!("Size: {:.6}", position.size);
/// ```
fn calculate_position(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
) -> PositionInfo {
    // Risk in account currency = balance * risk_percent / 100
    let risk_amount = balance * (risk_percent / 100.0);

    // Distance to stop-loss in price units
    let stop_distance = (entry_price - stop_loss).abs();

    // Position size = risk / stop_distance
    let size = risk_amount / stop_distance;

    // Position value
    let value = size * entry_price;

    // Distance to take-profit
    let profit_distance = (take_profit - entry_price).abs();

    // Potential profit
    let potential_profit = size * profit_distance;

    // Risk/reward ratio (R:R)
    // R:R > 1 means potential profit exceeds risk
    let risk_reward_ratio = profit_distance / stop_distance;

    PositionInfo {
        size,
        value,
        risk_amount,
        potential_profit,
        risk_reward_ratio,
    }
}

/// Prints position information in a readable format.
fn print_position_info(info: &PositionInfo) {
    println!("╔════════════════════════════════════╗");
    println!("║      POSITION CALCULATOR           ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Size:           {:>16.6} ║", info.size);
    println!("║ Value:          ${:>15.2} ║", info.value);
    println!("║ Risk:           ${:>15.2} ║", info.risk_amount);
    println!("║ Potential:      ${:>15.2} ║", info.potential_profit);
    println!("║ Risk/Reward:    {:>16.2} ║", info.risk_reward_ratio);
    println!("╚════════════════════════════════════╝");
}

fn main() {
    // Trade parameters
    let balance = 10000.0;       // Balance $10,000
    let risk = 2.0;              // Risk 2% per trade
    let entry = 42000.0;         // Entry at $42,000
    let stop = 41500.0;          // Stop at $41,500 (-1.2%)
    let target = 43500.0;        // Target $43,500 (+3.6%)

    let position = calculate_position(balance, risk, entry, stop, target);
    print_position_info(&position);
}
```

## What We Learned

| Comment Type | Syntax | Purpose |
|--------------|--------|---------|
| Single-line | `// text` | Brief explanation |
| Multi-line | `/* text */` | Block of explanations |
| Documentation | `/// text` | Function/struct documentation |
| Module documentation | `//! text` | Module documentation |
| TODO | `// TODO:` | Planned improvements |
| FIXME | `// FIXME:` | Known issues |

## Rules for Good Comments

1. **Explain "why", not "what"** — code shows what happens, comments explain why
2. **Document formulas** — especially in trading where there's lots of math
3. **Keep comments updated** — outdated comment is worse than no comment
4. **Use TODO/FIXME** — helps track technical debt
5. **Write doc comments** — for public functions and APIs

## Homework

1. Add documentation comments to a Simple Moving Average (SMA) calculation function with usage examples

2. Write a function `calculate_atr()` (Average True Range) with detailed comments explaining the formula

3. Create a documented `TradeJournal` struct for keeping trade records with doc comments for each field

4. Comment an RSI divergence strategy: when price makes a new high but RSI doesn't (bearish divergence)

## Navigation

[← Previous day](../015-return-values-pnl/en.md) | [Next day →](../017-control-flow-if/en.md)
