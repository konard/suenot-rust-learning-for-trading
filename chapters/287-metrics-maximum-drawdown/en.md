# Day 287: Metrics: Maximum Drawdown

## Trading Analogy

Imagine you're running a trading strategy. You start with $100,000, and over several months, your balance grows to $150,000 — things are looking great! But then the market turns against you, and your balance drops to $120,000. This 20% drop from your peak of $150,000 to the lowest point of $120,000 is called **Maximum Drawdown (MDD)**.

Maximum Drawdown is one of the most important risk metrics in trading because it tells you:
- **How much pain can you expect?** — the worst-case loss from a peak
- **Can you psychologically handle it?** — a 50% drawdown might make you panic and close your positions
- **How much capital do you need?** — you need enough buffer to survive the drawdown

In real trading, you want to know:
- What's the maximum loss from any peak to any subsequent trough?
- How long did it take to recover to a new peak?
- Is your strategy still viable after a major drawdown?

## What is Maximum Drawdown?

**Maximum Drawdown (MDD)** is the largest percentage decline in portfolio value from a historical peak to a subsequent trough before a new peak is reached.

### Formula

```
Drawdown = (Trough Value - Peak Value) / Peak Value × 100%
MDD = Maximum of all drawdowns
```

### Key Concepts

1. **Peak** — the highest point of portfolio value before a decline
2. **Trough** — the lowest point after a peak
3. **Recovery** — when portfolio value reaches a new peak
4. **Underwater Period** — time spent below the previous peak

## Simple Maximum Drawdown Calculator

```rust
fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        // Update peak if we've reached a new high
        if value > peak {
            peak = value;
        }

        // Calculate current drawdown from peak
        let drawdown = (peak - value) / peak * 100.0;

        // Update maximum drawdown if current is larger
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

fn main() {
    // Example equity curve: start at 10k, go to 15k, drop to 12k
    let equity = vec![
        10000.0, 10500.0, 11000.0, 12000.0, 13000.0,
        15000.0, 14000.0, 13000.0, 12000.0, 13500.0,
        14000.0, 14500.0,
    ];

    let mdd = calculate_max_drawdown(&equity);
    println!("Maximum Drawdown: {:.2}%", mdd);

    // Expected: 20% (from 15000 to 12000)
    // Calculation: (15000 - 12000) / 15000 = 0.20 = 20%
}
```

**Output:**
```
Maximum Drawdown: 20.00%
```

## Detailed Drawdown Analysis

Let's create a more comprehensive structure that tracks not just the MDD, but also when it occurred:

```rust
#[derive(Debug, Clone)]
struct DrawdownInfo {
    max_drawdown: f64,        // Maximum drawdown percentage
    peak_value: f64,          // Value at peak
    trough_value: f64,        // Value at trough
    peak_index: usize,        // When peak occurred
    trough_index: usize,      // When trough occurred
    current_drawdown: f64,    // Current drawdown from last peak
}

fn analyze_drawdown(equity_curve: &[f64]) -> DrawdownInfo {
    if equity_curve.is_empty() {
        return DrawdownInfo {
            max_drawdown: 0.0,
            peak_value: 0.0,
            trough_value: 0.0,
            peak_index: 0,
            trough_index: 0,
            current_drawdown: 0.0,
        };
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];
    let mut peak_index = 0;
    let mut mdd_peak_index = 0;
    let mut mdd_trough_index = 0;
    let mut mdd_peak_value = equity_curve[0];
    let mut mdd_trough_value = equity_curve[0];

    for (i, &value) in equity_curve.iter().enumerate() {
        // Update peak if we've reached a new high
        if value > peak {
            peak = value;
            peak_index = i;
        }

        // Calculate current drawdown from peak
        let drawdown = (peak - value) / peak * 100.0;

        // Update maximum drawdown if current is larger
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
            mdd_peak_index = peak_index;
            mdd_trough_index = i;
            mdd_peak_value = peak;
            mdd_trough_value = value;
        }
    }

    // Calculate current drawdown (from most recent peak)
    let current_drawdown = (peak - equity_curve[equity_curve.len() - 1]) / peak * 100.0;

    DrawdownInfo {
        max_drawdown,
        peak_value: mdd_peak_value,
        trough_value: mdd_trough_value,
        peak_index: mdd_peak_index,
        trough_index: mdd_trough_index,
        current_drawdown,
    }
}

fn main() {
    let equity = vec![
        10000.0, 11000.0, 12000.0, 15000.0, 14000.0,
        13000.0, 12000.0, 13500.0, 14000.0, 16000.0,
        15000.0, 14500.0,
    ];

    let info = analyze_drawdown(&equity);

    println!("Drawdown Analysis:");
    println!("  Maximum Drawdown: {:.2}%", info.max_drawdown);
    println!("  Peak Value: ${:.2} (at index {})", info.peak_value, info.peak_index);
    println!("  Trough Value: ${:.2} (at index {})", info.trough_value, info.trough_index);
    println!("  Current Drawdown: {:.2}%", info.current_drawdown);

    // Calculate recovery needed
    let recovery_needed = (info.peak_value / info.trough_value - 1.0) * 100.0;
    println!("  Recovery needed from trough: {:.2}%", recovery_needed);
}
```

**Output:**
```
Drawdown Analysis:
  Maximum Drawdown: 20.00%
  Peak Value: $15000.00 (at index 3)
  Trough Value: $12000.00 (at index 6)
  Current Drawdown: 9.38%
  Recovery needed from trough: 25.00%
```

**Important Note:** To recover from a 20% loss, you need a 25% gain! This is because you're starting from a smaller base.

## Real Trading Strategy Example

Let's simulate a simple moving average crossover strategy and calculate its drawdown:

```rust
#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    position_size: f64,
}

impl Trade {
    fn profit(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.position_size
    }
}

fn simulate_strategy_equity_curve(
    prices: &[f64],
    initial_capital: f64,
) -> Vec<f64> {
    let mut equity_curve = vec![initial_capital];
    let mut capital = initial_capital;
    let position_size = 1.0; // Trade 1 unit per signal

    // Simple strategy: buy when price goes up, sell when it goes down
    let mut in_position = false;
    let mut entry_price = 0.0;

    for i in 1..prices.len() {
        let prev_price = prices[i - 1];
        let current_price = prices[i];

        if !in_position && current_price > prev_price {
            // Enter long position
            entry_price = current_price;
            in_position = true;
        } else if in_position && current_price < prev_price {
            // Exit position
            let trade = Trade {
                entry_price,
                exit_price: current_price,
                position_size,
            };
            capital += trade.profit();
            in_position = false;
        }

        equity_curve.push(capital);
    }

    equity_curve
}

fn main() {
    // Simulated BTC prices over time
    let btc_prices = vec![
        40000.0, 42000.0, 45000.0, 43000.0, 41000.0,
        44000.0, 48000.0, 50000.0, 47000.0, 45000.0,
        43000.0, 46000.0, 49000.0, 52000.0, 51000.0,
    ];

    let initial_capital = 100000.0;
    let equity_curve = simulate_strategy_equity_curve(&btc_prices, initial_capital);

    println!("Equity Curve:");
    for (i, &equity) in equity_curve.iter().enumerate() {
        println!("  Day {}: ${:.2}", i, equity);
    }

    let drawdown_info = analyze_drawdown(&equity_curve);
    println!("\nStrategy Performance:");
    println!("  Initial Capital: ${:.2}", initial_capital);
    println!("  Final Capital: ${:.2}", equity_curve.last().unwrap());
    println!("  Total Return: {:.2}%",
        (equity_curve.last().unwrap() / initial_capital - 1.0) * 100.0);
    println!("  Maximum Drawdown: {:.2}%", drawdown_info.max_drawdown);
    println!("  Current Drawdown: {:.2}%", drawdown_info.current_drawdown);
}
```

## Multiple Drawdown Periods

A strategy can have multiple drawdown periods. Let's track all of them:

```rust
#[derive(Debug, Clone)]
struct DrawdownPeriod {
    peak_value: f64,
    trough_value: f64,
    peak_index: usize,
    trough_index: usize,
    drawdown_pct: f64,
    recovery_index: Option<usize>, // When (if) it recovered
}

fn find_all_drawdown_periods(equity_curve: &[f64]) -> Vec<DrawdownPeriod> {
    if equity_curve.is_empty() {
        return vec![];
    }

    let mut periods = Vec::new();
    let mut peak = equity_curve[0];
    let mut peak_index = 0;
    let mut in_drawdown = false;
    let mut trough = equity_curve[0];
    let mut trough_index = 0;

    for (i, &value) in equity_curve.iter().enumerate() {
        if value >= peak {
            // New peak or recovery
            if in_drawdown {
                // End of drawdown period - we recovered
                periods.last_mut().unwrap().recovery_index = Some(i);
                in_drawdown = false;
            }
            peak = value;
            peak_index = i;
            trough = value;
            trough_index = i;
        } else {
            // We're below the peak
            if !in_drawdown {
                // Start of new drawdown
                in_drawdown = true;
                trough = value;
                trough_index = i;

                periods.push(DrawdownPeriod {
                    peak_value: peak,
                    trough_value: value,
                    peak_index,
                    trough_index: i,
                    drawdown_pct: (peak - value) / peak * 100.0,
                    recovery_index: None,
                });
            } else if value < trough {
                // New low in current drawdown
                trough = value;
                trough_index = i;

                // Update the current period
                let last = periods.last_mut().unwrap();
                last.trough_value = value;
                last.trough_index = i;
                last.drawdown_pct = (peak - value) / peak * 100.0;
            }
        }
    }

    periods
}

fn main() {
    let equity = vec![
        10000.0, 12000.0, 15000.0, 13000.0, 12000.0, // First drawdown
        14000.0, 16000.0, 18000.0, 17000.0, 16000.0, // Second drawdown
        15000.0, 17000.0, 19000.0, 20000.0,          // Recovery and growth
    ];

    let periods = find_all_drawdown_periods(&equity);

    println!("Found {} drawdown periods:\n", periods.len());

    for (i, period) in periods.iter().enumerate() {
        println!("Drawdown #{}", i + 1);
        println!("  Peak: ${:.2} (index {})", period.peak_value, period.peak_index);
        println!("  Trough: ${:.2} (index {})", period.trough_value, period.trough_index);
        println!("  Drawdown: {:.2}%", period.drawdown_pct);

        if let Some(recovery_idx) = period.recovery_index {
            let duration = recovery_idx - period.peak_index;
            println!("  Recovered at index {} (duration: {} periods)", recovery_idx, duration);
        } else {
            println!("  Not yet recovered!");
        }
        println!();
    }

    // Calculate average drawdown
    let avg_drawdown: f64 = periods.iter()
        .map(|p| p.drawdown_pct)
        .sum::<f64>() / periods.len() as f64;

    println!("Average Drawdown: {:.2}%", avg_drawdown);

    let max_drawdown = periods.iter()
        .map(|p| p.drawdown_pct)
        .fold(0.0_f64, f64::max);

    println!("Maximum Drawdown: {:.2}%", max_drawdown);
}
```

## Complete Backtesting Module with Drawdown

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BacktestResults {
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub entry_time: usize,
    pub exit_time: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub position_size: f64,
    pub profit: f64,
    pub profit_pct: f64,
}

pub struct Backtester {
    initial_capital: f64,
    position_size_pct: f64, // Percentage of capital per trade
}

impl Backtester {
    pub fn new(initial_capital: f64, position_size_pct: f64) -> Self {
        Self {
            initial_capital,
            position_size_pct,
        }
    }

    pub fn run(&self, prices: &[f64], signals: &[i32]) -> BacktestResults {
        // signals: 1 = buy, -1 = sell, 0 = hold
        let mut capital = self.initial_capital;
        let mut equity_curve = vec![capital];
        let mut trades = Vec::new();

        let mut in_position = false;
        let mut entry_price = 0.0;
        let mut entry_time = 0;
        let mut position_size = 0.0;

        for i in 0..prices.len() {
            if signals[i] == 1 && !in_position {
                // Enter long position
                entry_price = prices[i];
                entry_time = i;
                position_size = (capital * self.position_size_pct) / prices[i];
                in_position = true;
            } else if signals[i] == -1 && in_position {
                // Exit position
                let exit_price = prices[i];
                let profit = (exit_price - entry_price) * position_size;
                let profit_pct = (exit_price / entry_price - 1.0) * 100.0;

                capital += profit;

                trades.push(Trade {
                    entry_time,
                    exit_time: i,
                    entry_price,
                    exit_price,
                    position_size,
                    profit,
                    profit_pct,
                });

                in_position = false;
            }

            equity_curve.push(capital);
        }

        // Calculate metrics
        let total_return = (capital / self.initial_capital - 1.0) * 100.0;
        let max_drawdown = calculate_max_drawdown(&equity_curve);

        let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = if trades.is_empty() {
            0.0
        } else {
            winning_trades as f64 / trades.len() as f64 * 100.0
        };

        let gross_profit: f64 = trades.iter()
            .filter(|t| t.profit > 0.0)
            .map(|t| t.profit)
            .sum();

        let gross_loss: f64 = trades.iter()
            .filter(|t| t.profit < 0.0)
            .map(|t| t.profit.abs())
            .sum();

        let profit_factor = if gross_loss == 0.0 {
            f64::INFINITY
        } else {
            gross_profit / gross_loss
        };

        // Simple Sharpe ratio approximation (assuming daily returns)
        let returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| (w[1] / w[0] - 1.0))
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        let sharpe_ratio = if std_dev == 0.0 {
            0.0
        } else {
            mean_return / std_dev * (252.0_f64).sqrt() // Annualized
        };

        BacktestResults {
            trades,
            equity_curve,
            total_return,
            max_drawdown,
            sharpe_ratio,
            win_rate,
            profit_factor,
        }
    }
}

fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        if value > peak {
            peak = value;
        }
        let drawdown = (peak - value) / peak * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

fn main() {
    // Example: Simple moving average crossover strategy
    let prices = vec![
        100.0, 102.0, 105.0, 103.0, 101.0, 104.0, 108.0,
        110.0, 107.0, 105.0, 103.0, 106.0, 109.0, 112.0,
    ];

    // Generate signals (simplified)
    let signals = vec![
        0, 1, 0, 0, -1, 1, 0,
        0, 0, -1, 0, 1, 0, 0,
    ];

    let backtester = Backtester::new(10000.0, 0.95);
    let results = backtester.run(&prices, &signals);

    println!("=== Backtest Results ===");
    println!("Total Trades: {}", results.trades.len());
    println!("Win Rate: {:.2}%", results.win_rate);
    println!("Total Return: {:.2}%", results.total_return);
    println!("Max Drawdown: {:.2}%", results.max_drawdown);
    println!("Profit Factor: {:.2}", results.profit_factor);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);

    println!("\n=== Individual Trades ===");
    for (i, trade) in results.trades.iter().enumerate() {
        println!("Trade #{}: Profit: ${:.2} ({:.2}%)",
            i + 1, trade.profit, trade.profit_pct);
    }

    println!("\n=== Equity Curve ===");
    for (i, &equity) in results.equity_curve.iter().enumerate() {
        println!("Period {}: ${:.2}", i, equity);
    }
}
```

## Why Maximum Drawdown Matters

| Metric | What it tells you |
|--------|-------------------|
| **Max Drawdown** | Worst-case loss you'd experience |
| **Recovery Factor** | Total Return / Max Drawdown (higher is better) |
| **Calmar Ratio** | Annual Return / Max Drawdown (risk-adjusted return) |
| **Win Rate** | Percentage of profitable trades |
| **Profit Factor** | Gross Profit / Gross Loss |

**Rule of Thumb:**
- MDD < 10%: Conservative, low risk
- MDD 10-20%: Moderate risk
- MDD 20-30%: High risk
- MDD > 30%: Very high risk, requires strong psychology

## What We Learned

| Concept | Description |
|---------|-------------|
| Maximum Drawdown | Largest peak-to-trough decline |
| Peak | Highest portfolio value before decline |
| Trough | Lowest point after peak |
| Recovery | Return to previous peak level |
| Drawdown Period | Time from peak to recovery |
| Underwater Period | Time spent below previous peak |

## Homework

1. **Calculate MDD**: Write a function that takes an equity curve and returns:
   - Maximum drawdown percentage
   - Peak value and index
   - Trough value and index
   - Whether the portfolio has recovered

2. **Drawdown Duration**: Extend the MDD calculator to track:
   - How many periods the maximum drawdown lasted
   - Average drawdown duration across all drawdown periods
   - Longest underwater period (time below previous peak)

3. **Risk Metrics Suite**: Create a `RiskMetrics` struct with methods to calculate:
   - Maximum Drawdown
   - Calmar Ratio (Annual Return / MDD)
   - Recovery Factor (Total Return / MDD)
   - Ulcer Index (RMS of drawdowns)

4. **Visual Drawdown**: Write a function that prints a simple text-based chart showing:
   - Equity curve
   - Peak levels
   - Drawdown periods highlighted
   - Current drawdown status

   Example output:
   ```
   Equity: $15000 ████████ PEAK
   Equity: $14000 ███████░ -6.7%
   Equity: $12000 ██████░░ -20.0% ← MAX DRAWDOWN
   Equity: $14500 ███████░ Recovering
   ```

## Navigation

[← Previous day](../286-metrics-profit-factor/en.md) | [Next day →](../288-report-generation/en.md)
