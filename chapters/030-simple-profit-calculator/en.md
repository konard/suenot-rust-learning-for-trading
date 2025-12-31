# Day 30: Simple Profit Calculator

## Trading Analogy

Every trader constantly calculates profits and losses. How much did I make on this trade? What percentage of my account? Did I factor in fees? Today we'll build a tool that does this automatically — a **profit calculator**. This is the first step toward building real trading software.

## Basic Profit Calculation

Trade Profit = (Exit Price - Entry Price) × Quantity

```rust
fn main() {
    let entry_price = 42000.0;  // Bought BTC at $42,000
    let exit_price = 43500.0;   // Sold at $43,500
    let quantity = 0.5;         // 0.5 BTC

    let profit = (exit_price - entry_price) * quantity;
    println!("Profit: ${:.2}", profit);  // $750.00
}
```

## Profit Calculation Function

Let's extract the logic into a function:

```rust
fn main() {
    let profit1 = calculate_profit(42000.0, 43500.0, 0.5);
    let profit2 = calculate_profit(2500.0, 2400.0, 2.0);
    let profit3 = calculate_profit(95.0, 110.0, 10.0);

    println!("BTC trade: ${:.2}", profit1);   // $750.00
    println!("ETH trade: ${:.2}", profit2);   // -$200.00
    println!("SOL trade: ${:.2}", profit3);   // $150.00
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

## Profit Percentage Calculation

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;

    let profit_pct = calculate_profit_percent(entry, exit);
    println!("Profit: {:.2}%", profit_pct);  // 3.57%
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    if entry == 0.0 {
        return 0.0;  // Protection against division by zero
    }
    ((exit - entry) / entry) * 100.0
}
```

## Accounting for Fees

Exchanges charge a fee for each trade. Usually it's a percentage of the volume:

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;  // 0.1% fee (typical for exchanges)

    let gross_profit = calculate_profit(entry, exit, quantity);
    let total_fees = calculate_total_fees(entry, exit, quantity, fee_percent);
    let net_profit = gross_profit - total_fees;

    println!("Gross profit: ${:.2}", gross_profit);
    println!("Fees: ${:.2}", total_fees);
    println!("Net profit: ${:.2}", net_profit);
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_total_fees(entry: f64, exit: f64, quantity: f64, fee_pct: f64) -> f64 {
    let entry_value = entry * quantity;
    let exit_value = exit * quantity;
    (entry_value + exit_value) * (fee_pct / 100.0)
}
```

## ROI — Return on Investment

ROI shows what percentage of profit was earned relative to the invested funds:

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_pct = 0.1;

    let roi = calculate_roi(entry, exit, quantity, fee_pct);
    println!("ROI: {:.2}%", roi);
}

fn calculate_roi(entry: f64, exit: f64, quantity: f64, fee_pct: f64) -> f64 {
    let investment = entry * quantity;
    if investment == 0.0 {
        return 0.0;
    }

    let gross = (exit - entry) * quantity;
    let fees = (entry + exit) * quantity * (fee_pct / 100.0);
    let net = gross - fees;

    (net / investment) * 100.0
}
```

## Determining Trade Status

```rust
fn main() {
    analyze_trade(42000.0, 43500.0, 0.5);
    analyze_trade(42000.0, 41000.0, 0.5);
    analyze_trade(42000.0, 42000.0, 0.5);
}

fn analyze_trade(entry: f64, exit: f64, quantity: f64) {
    let profit = calculate_profit(entry, exit, quantity);
    let status = get_trade_status(profit);
    let emoji = get_status_emoji(profit);

    println!("{} {} ${:.2}", emoji, status, profit.abs());
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn get_trade_status(profit: f64) -> &'static str {
    if profit > 0.0 {
        "PROFIT"
    } else if profit < 0.0 {
        "LOSS"
    } else {
        "BREAKEVEN"
    }
}

fn get_status_emoji(profit: f64) -> &'static str {
    if profit > 0.0 {
        "[+]"
    } else if profit < 0.0 {
        "[-]"
    } else {
        "[=]"
    }
}
```

## Complete Profit Calculator

```rust
fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       PROFIT CALCULATOR               ║");
    println!("╠═══════════════════════════════════════╣");

    // Trade data
    let symbol = "BTC/USDT";
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;

    // Calculations
    let entry_value = entry_price * quantity;
    let exit_value = exit_price * quantity;

    let gross_profit = calculate_profit(entry_price, exit_price, quantity);
    let profit_percent = calculate_profit_percent(entry_price, exit_price);

    let entry_fee = calculate_fee(entry_value, fee_percent);
    let exit_fee = calculate_fee(exit_value, fee_percent);
    let total_fees = entry_fee + exit_fee;

    let net_profit = gross_profit - total_fees;
    let roi = calculate_roi_from_values(net_profit, entry_value);

    let status = get_trade_status(net_profit);

    // Output results
    println!("║ Pair:           {:>20} ║", symbol);
    println!("║ Entry Price:    ${:>18.2} ║", entry_price);
    println!("║ Exit Price:     ${:>18.2} ║", exit_price);
    println!("║ Quantity:       {:>19.4} ║", quantity);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Entry Value:    ${:>18.2} ║", entry_value);
    println!("║ Exit Value:     ${:>18.2} ║", exit_value);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Gross Profit:   ${:>18.2} ║", gross_profit);
    println!("║ Price Change:   {:>18.2}% ║", profit_percent);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Entry Fee:      ${:>18.2} ║", entry_fee);
    println!("║ Exit Fee:       ${:>18.2} ║", exit_fee);
    println!("║ Total Fees:     ${:>18.2} ║", total_fees);
    println!("╠═══════════════════════════════════════╣");
    println!("║ NET PROFIT:     ${:>18.2} ║", net_profit);
    println!("║ ROI:            {:>18.2}% ║", roi);
    println!("║ Status:         {:>20} ║", status);
    println!("╚═══════════════════════════════════════╝");
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    if entry == 0.0 { return 0.0; }
    ((exit - entry) / entry) * 100.0
}

fn calculate_fee(value: f64, fee_percent: f64) -> f64 {
    value * (fee_percent / 100.0)
}

fn calculate_roi_from_values(net_profit: f64, investment: f64) -> f64 {
    if investment == 0.0 { return 0.0; }
    (net_profit / investment) * 100.0
}

fn get_trade_status(profit: f64) -> &'static str {
    if profit > 0.0 {
        "PROFIT"
    } else if profit < 0.0 {
        "LOSS"
    } else {
        "BREAKEVEN"
    }
}
```

## Analyzing Multiple Trades

```rust
fn main() {
    // Array of trades: (entry, exit, quantity)
    let trades = [
        (42000.0, 43500.0, 0.5),   // BTC: profit
        (2500.0, 2400.0, 2.0),     // ETH: loss
        (95.0, 110.0, 10.0),      // SOL: profit
        (0.65, 0.58, 1000.0),     // XRP: loss
        (28000.0, 29500.0, 0.3),  // BTC: profit
    ];

    let fee_pct = 0.1;
    let mut total_profit = 0.0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;

    println!("╔═════════════════════════════════════════╗");
    println!("║         PORTFOLIO ANALYSIS              ║");
    println!("╠═════════════════════════════════════════╣");

    for (i, &(entry, exit, qty)) in trades.iter().enumerate() {
        let gross = (exit - entry) * qty;
        let fees = (entry + exit) * qty * (fee_pct / 100.0);
        let net = gross - fees;

        total_profit += net;

        if net > 0.0 {
            winning_trades += 1;
        } else if net < 0.0 {
            losing_trades += 1;
        }

        let status = if net > 0.0 { "+" } else { "-" };
        println!("║ Trade #{}: {} ${:>10.2}              ║", i + 1, status, net.abs());
    }

    let total_trades = trades.len();
    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;

    println!("╠═════════════════════════════════════════╣");
    println!("║ Total trades:    {:>5}                  ║", total_trades);
    println!("║ Winners:         {:>5}                  ║", winning_trades);
    println!("║ Losers:          {:>5}                  ║", losing_trades);
    println!("║ Win Rate:        {:>5.1}%                 ║", win_rate);
    println!("╠═════════════════════════════════════════╣");
    let status = if total_profit > 0.0 { "PROFIT" } else { "LOSS" };
    println!("║ TOTAL:           ${:>10.2} ({})   ║", total_profit, status);
    println!("╚═════════════════════════════════════════╝");
}
```

## Practical Exercises

### Exercise 1: Breakeven Point Calculation

Find the exit price at which the trade covers fees:

```rust
fn main() {
    let entry = 42000.0;
    let quantity = 0.5;
    let fee_pct = 0.1;

    let breakeven = calculate_breakeven(entry, quantity, fee_pct);
    println!("Breakeven point: ${:.2}", breakeven);
}

fn calculate_breakeven(entry: f64, quantity: f64, fee_pct: f64) -> f64 {
    // entry_fee = entry * qty * fee_pct / 100
    // exit_fee = exit * qty * fee_pct / 100
    // profit = (exit - entry) * qty
    // net = profit - entry_fee - exit_fee = 0
    // Solve equation for exit

    let fee_rate = fee_pct / 100.0;
    let numerator = entry * (1.0 + fee_rate);
    let denominator = 1.0 - fee_rate;

    numerator / denominator
}
```

### Exercise 2: Comparing Long and Short Positions

```rust
fn main() {
    let price1 = 42000.0;
    let price2 = 43500.0;
    let quantity = 0.5;

    let long_profit = calculate_long_profit(price1, price2, quantity);
    let short_profit = calculate_short_profit(price1, price2, quantity);

    println!("Long (buy at {}, sell at {}): ${:.2}", price1, price2, long_profit);
    println!("Short (sell at {}, buy at {}): ${:.2}", price1, price2, short_profit);
}

fn calculate_long_profit(entry: f64, exit: f64, qty: f64) -> f64 {
    (exit - entry) * qty
}

fn calculate_short_profit(entry: f64, exit: f64, qty: f64) -> f64 {
    (entry - exit) * qty  // In short, profit when price drops
}
```

### Exercise 3: Calculator with Slippage

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let slippage_pct = 0.05;  // 0.05% slippage

    let (actual_entry, actual_exit, profit) =
        calculate_with_slippage(entry, exit, quantity, slippage_pct);

    println!("Expected entry: ${:.2}", entry);
    println!("Actual entry: ${:.2}", actual_entry);
    println!("Expected exit: ${:.2}", exit);
    println!("Actual exit: ${:.2}", actual_exit);
    println!("Profit with slippage: ${:.2}", profit);
}

fn calculate_with_slippage(
    entry: f64,
    exit: f64,
    quantity: f64,
    slippage_pct: f64
) -> (f64, f64, f64) {
    let slippage_rate = slippage_pct / 100.0;

    // When buying, price is higher (slippage up)
    let actual_entry = entry * (1.0 + slippage_rate);

    // When selling, price is lower (slippage down)
    let actual_exit = exit * (1.0 - slippage_rate);

    let profit = (actual_exit - actual_entry) * quantity;

    (actual_entry, actual_exit, profit)
}
```

### Exercise 4: Profit as Portfolio Percentage

```rust
fn main() {
    let portfolio_value = 10000.0;  // Total portfolio size
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.1;  // Position size

    let profit = (exit - entry) * quantity;
    let position_value = entry * quantity;
    let portfolio_impact = (profit / portfolio_value) * 100.0;
    let position_size_pct = (position_value / portfolio_value) * 100.0;

    println!("Portfolio size: ${:.2}", portfolio_value);
    println!("Position size: ${:.2} ({:.1}% of portfolio)", position_value, position_size_pct);
    println!("Profit: ${:.2}", profit);
    println!("Portfolio impact: {:.2}%", portfolio_impact);
}
```

## What We Learned

| Concept | Formula | Example |
|---------|---------|---------|
| Profit | (exit - entry) × qty | (43500 - 42000) × 0.5 = $750 |
| Profit % | ((exit - entry) / entry) × 100 | 3.57% |
| Fee | value × fee% / 100 | 21000 × 0.1% = $21 |
| ROI | (net_profit / investment) × 100 | 3.47% |
| Win Rate | wins / total × 100 | 60% |

## Homework

1. **Leveraged Calculator**: Create a function `calculate_leveraged_pnl(entry, exit, quantity, leverage) -> f64` that accounts for leverage.

2. **Risk/Reward Calculator**: Write a function `calculate_risk_reward(entry, stop_loss, take_profit) -> f64` that returns the risk-to-reward ratio.

3. **Average Entry Price**: Implement a function `calculate_average_entry(entries: &[(f64, f64)]) -> f64` that calculates the average entry price for multiple purchases (price, quantity).

4. **Trade Series Simulator**: Create a program that simulates 10 trades with random results and outputs overall statistics (total PnL, win rate, max drawdown).

## Navigation

[← Previous day](../029-input-validation/en.md) | [Next day →](../031-position-sizing/en.md)
