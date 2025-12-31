# Day 282: Equity Curve

## Trading Analogy

Imagine you're managing a trading account and want to understand how well your strategy performs. You can't just look at the last trade — you need to see the entire history of capital changes. The **Equity Curve** is a graph showing how the value of your portfolio changes over time.

It's like a medical history for a patient: a doctor doesn't just look at the latest test results, but studies the entire trend of indicators. For a trader, the equity curve is the "medical record" of their strategy, showing:
- Is capital growing steadily?
- What were the maximum drawdowns?
- How quickly did the strategy recover after losses?

## What is an Equity Curve?

An Equity Curve is a time series of capital values, built from the results of each trade or at the end of each trading period. It's a key tool for:

1. **Visualizing performance** — a quick glance at the big picture
2. **Calculating risk metrics** — maximum drawdown, return volatility
3. **Comparing strategies** — which strategy is better?
4. **Identifying market regimes** — when does the strategy work and when doesn't it?

## Basic Equity Curve Structure

```rust
use std::collections::VecDeque;

/// A point on the equity curve
#[derive(Debug, Clone)]
struct EquityPoint {
    timestamp: u64,       // Unix timestamp
    equity: f64,          // Current capital
    cash: f64,            // Available cash
    positions_value: f64, // Value of open positions
}

/// Equity curve with metric calculations
#[derive(Debug)]
struct EquityCurve {
    points: Vec<EquityPoint>,
    initial_capital: f64,
    peak_equity: f64,        // Maximum capital value
    peak_timestamp: u64,     // Time peak was reached
}

impl EquityCurve {
    fn new(initial_capital: f64) -> Self {
        EquityCurve {
            points: Vec::new(),
            initial_capital,
            peak_equity: initial_capital,
            peak_timestamp: 0,
        }
    }

    /// Add a new point to the curve
    fn add_point(&mut self, timestamp: u64, cash: f64, positions_value: f64) {
        let equity = cash + positions_value;

        // Update peak if new maximum reached
        if equity > self.peak_equity {
            self.peak_equity = equity;
            self.peak_timestamp = timestamp;
        }

        self.points.push(EquityPoint {
            timestamp,
            equity,
            cash,
            positions_value,
        });
    }

    /// Get current equity
    fn current_equity(&self) -> f64 {
        self.points.last()
            .map(|p| p.equity)
            .unwrap_or(self.initial_capital)
    }

    /// Calculate total return percentage
    fn total_return(&self) -> f64 {
        let current = self.current_equity();
        ((current - self.initial_capital) / self.initial_capital) * 100.0
    }
}

fn main() {
    let mut curve = EquityCurve::new(100_000.0);

    // Simulate capital changes
    curve.add_point(1, 100_000.0, 0.0);        // Start
    curve.add_point(2, 95_000.0, 7_000.0);     // Bought BTC
    curve.add_point(3, 95_000.0, 8_500.0);     // BTC went up
    curve.add_point(4, 95_000.0, 6_000.0);     // BTC went down
    curve.add_point(5, 101_500.0, 0.0);        // Sold with profit

    println!("Initial capital: ${:.2}", curve.initial_capital);
    println!("Current capital: ${:.2}", curve.current_equity());
    println!("Total return: {:.2}%", curve.total_return());
    println!("Peak capital: ${:.2}", curve.peak_equity);
}
```

## Calculating Drawdown

Drawdown is the decline from a peak value to a trough. It's a critical metric for assessing strategy risk.

```rust
/// Extended equity curve with drawdown calculations
#[derive(Debug)]
struct AdvancedEquityCurve {
    points: Vec<EquityPoint>,
    initial_capital: f64,
    peak_equity: f64,
    max_drawdown: f64,           // Maximum drawdown in %
    max_drawdown_duration: u64,  // Duration of maximum drawdown
    current_drawdown: f64,       // Current drawdown in %
}

#[derive(Debug, Clone)]
struct EquityPoint {
    timestamp: u64,
    equity: f64,
    cash: f64,
    positions_value: f64,
    drawdown: f64,  // Drawdown from peak in %
}

impl AdvancedEquityCurve {
    fn new(initial_capital: f64) -> Self {
        AdvancedEquityCurve {
            points: Vec::new(),
            initial_capital,
            peak_equity: initial_capital,
            max_drawdown: 0.0,
            max_drawdown_duration: 0,
            current_drawdown: 0.0,
        }
    }

    fn add_point(&mut self, timestamp: u64, cash: f64, positions_value: f64) {
        let equity = cash + positions_value;

        // Update peak
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        // Calculate drawdown
        let drawdown = if self.peak_equity > 0.0 {
            ((self.peak_equity - equity) / self.peak_equity) * 100.0
        } else {
            0.0
        };

        self.current_drawdown = drawdown;

        // Update maximum drawdown
        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }

        self.points.push(EquityPoint {
            timestamp,
            equity,
            cash,
            positions_value,
            drawdown,
        });
    }

    /// Get all drawdown periods
    fn get_drawdown_periods(&self) -> Vec<DrawdownPeriod> {
        let mut periods = Vec::new();
        let mut in_drawdown = false;
        let mut start_idx = 0;
        let mut peak_before_dd = 0.0;

        for (i, point) in self.points.iter().enumerate() {
            if !in_drawdown && point.drawdown > 0.0 {
                // Drawdown starts
                in_drawdown = true;
                start_idx = i;
                peak_before_dd = if i > 0 {
                    self.points[i - 1].equity
                } else {
                    self.initial_capital
                };
            } else if in_drawdown && point.drawdown == 0.0 {
                // End of drawdown (recovery)
                in_drawdown = false;
                let max_dd = self.points[start_idx..i]
                    .iter()
                    .map(|p| p.drawdown)
                    .fold(0.0, f64::max);

                periods.push(DrawdownPeriod {
                    start_timestamp: self.points[start_idx].timestamp,
                    end_timestamp: point.timestamp,
                    max_drawdown: max_dd,
                    recovery_time: point.timestamp - self.points[start_idx].timestamp,
                });
            }
        }

        // If still in drawdown
        if in_drawdown {
            let last = self.points.last().unwrap();
            let max_dd = self.points[start_idx..]
                .iter()
                .map(|p| p.drawdown)
                .fold(0.0, f64::max);

            periods.push(DrawdownPeriod {
                start_timestamp: self.points[start_idx].timestamp,
                end_timestamp: last.timestamp,
                max_drawdown: max_dd,
                recovery_time: 0, // Not recovered yet
            });
        }

        periods
    }
}

#[derive(Debug)]
struct DrawdownPeriod {
    start_timestamp: u64,
    end_timestamp: u64,
    max_drawdown: f64,
    recovery_time: u64,
}

fn main() {
    let mut curve = AdvancedEquityCurve::new(100_000.0);

    // Simulate trading with drawdowns
    let equity_history = vec![
        (1, 100_000.0),
        (2, 102_000.0),  // +2%
        (3, 105_000.0),  // New peak
        (4, 98_000.0),   // Drawdown -6.67%
        (5, 95_000.0),   // Drawdown -9.52%
        (6, 100_000.0),  // Recovery
        (7, 108_000.0),  // New peak
        (8, 103_000.0),  // Small drawdown
        (9, 110_000.0),  // New peak
    ];

    for (ts, equity) in equity_history {
        curve.add_point(ts, equity, 0.0);
    }

    println!("=== Equity Curve Analysis ===\n");
    println!("Initial capital: ${:.2}", curve.initial_capital);
    println!("Peak capital: ${:.2}", curve.peak_equity);
    println!("Maximum drawdown: {:.2}%", curve.max_drawdown);

    println!("\n--- Drawdown Periods ---");
    for (i, period) in curve.get_drawdown_periods().iter().enumerate() {
        println!(
            "Drawdown #{}: {:.2}% (from {} to {}, recovery: {} periods)",
            i + 1,
            period.max_drawdown,
            period.start_timestamp,
            period.end_timestamp,
            period.recovery_time
        );
    }
}
```

## Calculating Key Metrics

```rust
use std::f64::consts::E;

/// Full equity curve analyzer
struct EquityAnalyzer {
    returns: Vec<f64>,          // Daily returns
    equity_values: Vec<f64>,    // Equity values
    risk_free_rate: f64,        // Annual risk-free rate
}

impl EquityAnalyzer {
    fn new(equity_values: Vec<f64>, risk_free_rate: f64) -> Self {
        // Calculate returns
        let returns: Vec<f64> = equity_values.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        EquityAnalyzer {
            returns,
            equity_values,
            risk_free_rate,
        }
    }

    /// Mean return
    fn mean_return(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        self.returns.iter().sum::<f64>() / self.returns.len() as f64
    }

    /// Standard deviation of returns (volatility)
    fn volatility(&self) -> f64 {
        if self.returns.len() < 2 {
            return 0.0;
        }

        let mean = self.mean_return();
        let variance: f64 = self.returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (self.returns.len() - 1) as f64;

        variance.sqrt()
    }

    /// Sharpe Ratio (annualized)
    fn sharpe_ratio(&self, periods_per_year: f64) -> f64 {
        let vol = self.volatility();
        if vol == 0.0 {
            return 0.0;
        }

        let mean_return = self.mean_return();
        let excess_return = mean_return - (self.risk_free_rate / periods_per_year);

        (excess_return / vol) * periods_per_year.sqrt()
    }

    /// Sortino Ratio (considers only downside volatility)
    fn sortino_ratio(&self, periods_per_year: f64) -> f64 {
        let mean = self.mean_return();

        // Count only negative deviations
        let downside_returns: Vec<f64> = self.returns.iter()
            .filter(|&&r| r < 0.0)
            .cloned()
            .collect();

        if downside_returns.is_empty() {
            return f64::INFINITY; // No negative returns
        }

        let downside_variance: f64 = downside_returns.iter()
            .map(|r| r.powi(2))
            .sum::<f64>() / downside_returns.len() as f64;

        let downside_deviation = downside_variance.sqrt();

        if downside_deviation == 0.0 {
            return f64::INFINITY;
        }

        let excess_return = mean - (self.risk_free_rate / periods_per_year);
        (excess_return / downside_deviation) * periods_per_year.sqrt()
    }

    /// Maximum drawdown
    fn max_drawdown(&self) -> f64 {
        let mut max_dd = 0.0;
        let mut peak = self.equity_values[0];

        for &equity in &self.equity_values {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd * 100.0
    }

    /// Calmar Ratio (annual return / max drawdown)
    fn calmar_ratio(&self, periods_per_year: f64) -> f64 {
        let max_dd = self.max_drawdown();
        if max_dd == 0.0 {
            return f64::INFINITY;
        }

        let total_return = (self.equity_values.last().unwrap()
            / self.equity_values.first().unwrap() - 1.0) * 100.0;

        let num_periods = self.equity_values.len() as f64;
        let years = num_periods / periods_per_year;
        let annual_return = total_return / years;

        annual_return / max_dd
    }

    /// Win rate percentage
    fn win_rate(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }

        let wins = self.returns.iter().filter(|&&r| r > 0.0).count();
        (wins as f64 / self.returns.len() as f64) * 100.0
    }

    /// Profit Factor (sum of profits / sum of losses)
    fn profit_factor(&self) -> f64 {
        let gains: f64 = self.returns.iter()
            .filter(|&&r| r > 0.0)
            .sum();

        let losses: f64 = self.returns.iter()
            .filter(|&&r| r < 0.0)
            .map(|r| r.abs())
            .sum();

        if losses == 0.0 {
            return f64::INFINITY;
        }

        gains / losses
    }

    /// Print all metrics
    fn print_report(&self, periods_per_year: f64) {
        println!("╔════════════════════════════════════════╗");
        println!("║       EQUITY CURVE REPORT              ║");
        println!("╠════════════════════════════════════════╣");
        println!("║ Total Return:         {:>10.2}%      ║",
            (self.equity_values.last().unwrap() / self.equity_values.first().unwrap() - 1.0) * 100.0);
        println!("║ Mean Return:          {:>10.4}%      ║", self.mean_return() * 100.0);
        println!("║ Volatility:           {:>10.4}%      ║", self.volatility() * 100.0);
        println!("║ Max Drawdown:         {:>10.2}%      ║", self.max_drawdown());
        println!("╠════════════════════════════════════════╣");
        println!("║ Sharpe Ratio:         {:>10.2}       ║", self.sharpe_ratio(periods_per_year));
        println!("║ Sortino Ratio:        {:>10.2}       ║", self.sortino_ratio(periods_per_year));
        println!("║ Calmar Ratio:         {:>10.2}       ║", self.calmar_ratio(periods_per_year));
        println!("╠════════════════════════════════════════╣");
        println!("║ Win Rate:             {:>10.2}%      ║", self.win_rate());
        println!("║ Profit Factor:        {:>10.2}       ║", self.profit_factor());
        println!("╚════════════════════════════════════════╝");
    }
}

fn main() {
    // Simulate daily equity values for a year
    let equity_values: Vec<f64> = vec![
        100_000.0, 100_500.0, 101_200.0, 100_800.0, 101_500.0,
        102_300.0, 101_900.0, 103_100.0, 104_200.0, 103_800.0,
        105_100.0, 106_000.0, 105_200.0, 106_800.0, 107_500.0,
        106_900.0, 108_200.0, 109_100.0, 108_500.0, 110_000.0,
    ];

    let analyzer = EquityAnalyzer::new(equity_values, 0.05); // 5% risk-free rate

    // 252 trading days per year
    analyzer.print_report(252.0);
}
```

## ASCII Visualization of Equity Curve

```rust
/// Simple ASCII visualization of equity curve
struct AsciiChart {
    width: usize,
    height: usize,
}

impl AsciiChart {
    fn new(width: usize, height: usize) -> Self {
        AsciiChart { width, height }
    }

    fn plot(&self, values: &[f64], title: &str) {
        if values.is_empty() {
            println!("No data to display");
            return;
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        println!("\n{}", title);
        println!("{}", "═".repeat(self.width + 10));

        // Create grid
        let mut grid = vec![vec![' '; self.width]; self.height];

        // Fill points
        let step = values.len() as f64 / self.width as f64;
        for x in 0..self.width {
            let idx = (x as f64 * step) as usize;
            if idx < values.len() {
                let normalized = if range > 0.0 {
                    (values[idx] - min_val) / range
                } else {
                    0.5
                };
                let y = ((1.0 - normalized) * (self.height - 1) as f64) as usize;
                let y = y.min(self.height - 1);
                grid[y][x] = '█';
            }
        }

        // Output chart
        for (i, row) in grid.iter().enumerate() {
            let label = if i == 0 {
                format!("{:>8.0} │", max_val)
            } else if i == self.height - 1 {
                format!("{:>8.0} │", min_val)
            } else {
                "         │".to_string()
            };

            let line: String = row.iter().collect();
            println!("{}{}", label, line);
        }

        println!("         └{}", "─".repeat(self.width));
        println!("          0{:>width$}", values.len() - 1, width = self.width - 1);
    }
}

fn main() {
    // Generate equity curve with trend and noise
    let mut equity = vec![100_000.0];
    let mut current = 100_000.0;

    for i in 1..50 {
        // Upward trend with noise
        let change = (i as f64 * 50.0) + ((i as f64).sin() * 2000.0);
        current = 100_000.0 + change;
        equity.push(current);
    }

    let chart = AsciiChart::new(50, 15);
    chart.plot(&equity, "Equity Curve - Trading Strategy");

    println!("\nStatistics:");
    println!("  Start: ${:.2}", equity.first().unwrap());
    println!("  End:   ${:.2}", equity.last().unwrap());
    println!("  Min:   ${:.2}", equity.iter().cloned().fold(f64::INFINITY, f64::min));
    println!("  Max:   ${:.2}", equity.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
}
```

## Practical Example: Backtester with Built-in Equity Curve

```rust
use std::collections::HashMap;

/// Trade structure
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_time: u64,
    exit_time: u64,
    side: TradeSide,
}

#[derive(Debug, Clone, PartialEq)]
enum TradeSide {
    Long,
    Short,
}

impl Trade {
    fn pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn return_pct(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price / self.entry_price - 1.0) * 100.0,
            TradeSide::Short => (self.entry_price / self.exit_price - 1.0) * 100.0,
        }
    }
}

/// Backtest results
struct BacktestResult {
    trades: Vec<Trade>,
    equity_curve: Vec<(u64, f64)>,  // (timestamp, equity)
    initial_capital: f64,
}

impl BacktestResult {
    fn new(initial_capital: f64) -> Self {
        BacktestResult {
            trades: Vec::new(),
            equity_curve: vec![(0, initial_capital)],
            initial_capital,
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        let last_equity = self.equity_curve.last().unwrap().1;
        let new_equity = last_equity + trade.pnl();
        self.equity_curve.push((trade.exit_time, new_equity));
        self.trades.push(trade);
    }

    fn final_equity(&self) -> f64 {
        self.equity_curve.last().unwrap().1
    }

    fn total_return(&self) -> f64 {
        (self.final_equity() / self.initial_capital - 1.0) * 100.0
    }

    fn max_drawdown(&self) -> f64 {
        let mut max_dd = 0.0;
        let mut peak = self.initial_capital;

        for &(_, equity) in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }

    fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        let wins = self.trades.iter().filter(|t| t.pnl() > 0.0).count();
        (wins as f64 / self.trades.len() as f64) * 100.0
    }

    fn avg_win(&self) -> f64 {
        let wins: Vec<f64> = self.trades.iter()
            .filter(|t| t.pnl() > 0.0)
            .map(|t| t.pnl())
            .collect();

        if wins.is_empty() {
            return 0.0;
        }
        wins.iter().sum::<f64>() / wins.len() as f64
    }

    fn avg_loss(&self) -> f64 {
        let losses: Vec<f64> = self.trades.iter()
            .filter(|t| t.pnl() < 0.0)
            .map(|t| t.pnl().abs())
            .collect();

        if losses.is_empty() {
            return 0.0;
        }
        losses.iter().sum::<f64>() / losses.len() as f64
    }

    fn profit_factor(&self) -> f64 {
        let gross_profit: f64 = self.trades.iter()
            .filter(|t| t.pnl() > 0.0)
            .map(|t| t.pnl())
            .sum();

        let gross_loss: f64 = self.trades.iter()
            .filter(|t| t.pnl() < 0.0)
            .map(|t| t.pnl().abs())
            .sum();

        if gross_loss == 0.0 {
            return f64::INFINITY;
        }
        gross_profit / gross_loss
    }

    fn print_report(&self) {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║           BACKTEST RESULTS               ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Initial Capital:    ${:>15.2}    ║", self.initial_capital);
        println!("║ Final Capital:      ${:>15.2}    ║", self.final_equity());
        println!("║ Total Return:       {:>15.2}%   ║", self.total_return());
        println!("╠══════════════════════════════════════════╣");
        println!("║ Total Trades:       {:>15}     ║", self.trades.len());
        println!("║ Win Rate:           {:>15.2}%   ║", self.win_rate());
        println!("║ Average Win:        ${:>15.2}    ║", self.avg_win());
        println!("║ Average Loss:       ${:>15.2}    ║", self.avg_loss());
        println!("╠══════════════════════════════════════════╣");
        println!("║ Profit Factor:      {:>15.2}     ║", self.profit_factor());
        println!("║ Max Drawdown:       {:>15.2}%   ║", self.max_drawdown());
        println!("╚══════════════════════════════════════════╝");
    }
}

fn main() {
    let mut result = BacktestResult::new(100_000.0);

    // Simulate a series of trades
    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 40000.0,
            exit_price: 42000.0,
            quantity: 1.0,
            entry_time: 1,
            exit_time: 5,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "ETH".to_string(),
            entry_price: 2500.0,
            exit_price: 2400.0,
            quantity: 10.0,
            entry_time: 6,
            exit_time: 10,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 43000.0,
            exit_price: 41000.0,
            quantity: 1.0,
            entry_time: 11,
            exit_time: 15,
            side: TradeSide::Short,
        },
        Trade {
            symbol: "ETH".to_string(),
            entry_price: 2300.0,
            exit_price: 2600.0,
            quantity: 15.0,
            entry_time: 16,
            exit_time: 20,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 44000.0,
            exit_price: 46000.0,
            quantity: 1.5,
            entry_time: 21,
            exit_time: 25,
            side: TradeSide::Long,
        },
    ];

    for trade in trades {
        println!(
            "Trade: {} {} @ {:.2} -> {:.2}, P/L: ${:.2}",
            trade.symbol,
            if trade.side == TradeSide::Long { "LONG" } else { "SHORT" },
            trade.entry_price,
            trade.exit_price,
            trade.pnl()
        );
        result.add_trade(trade);
    }

    result.print_report();

    // Output equity curve
    println!("\n--- Equity Curve ---");
    for (ts, equity) in &result.equity_curve {
        println!("T={}: ${:.2}", ts, equity);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Equity Curve | Time series of capital values |
| Drawdown | Decline from peak value |
| Sharpe Ratio | Return adjusted for volatility |
| Sortino Ratio | Return adjusted for downside volatility only |
| Calmar Ratio | Annual return / maximum drawdown |
| Profit Factor | Ratio of gross profits to gross losses |
| Win Rate | Percentage of profitable trades |

## Practical Exercises

1. **Simple Equity Curve**: Create a structure for tracking capital with the ability to add trades. Implement methods to calculate current capital and total return.

2. **Drawdown Analysis**: Extend the equity curve with a function to find all drawdown periods. For each drawdown, save: depth, start time, recovery time.

3. **Strategy Comparison**: Create a program that compares two equity curves and determines which strategy is better by:
   - Sharpe Ratio
   - Maximum Drawdown
   - Profit Factor

## Homework

1. **Rolling Metrics**: Implement calculation of metrics over a rolling window (e.g., Sharpe Ratio for the last 30 days). This will help you see how strategy quality changes over time.

2. **Monte Carlo Simulation**: Using trade history, generate 1000 random permutations and build a distribution of possible equity curves. This will show how much the results depend on luck.

3. **Alert System**: Implement an alert system that:
   - Warns when drawdown exceeds 10%
   - Notifies when a new equity high is reached
   - Tracks when Sharpe Ratio falls below a set threshold

4. **CSV Export**: Add the ability to export the equity curve to a CSV file for further analysis in Excel or Python.

## Navigation

[← Previous day](../281-backtest-report/en.md) | [Next day →](../283-sharpe-ratio/en.md)
