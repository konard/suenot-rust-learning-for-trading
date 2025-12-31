# Day 302: Comparing Strategies

## Trading Analogy

Imagine you have three trading robots:
- Robot A: aggressive scalper, makes 100 trades per day with small profits
- Robot B: conservative long-term holder, holds positions for weeks
- Robot C: swing trader, trades on daily charts

Each shows profit, but which is better? You can't just compare absolute profit — you need to consider risk, drawdowns, stability, commission costs. It's like comparing cars: you can't choose based solely on maximum speed, you need to consider fuel consumption, reliability, comfort.

**Strategy comparison** is a systematic approach to evaluating trading algorithms across multiple criteria to choose the most suitable strategy for specific conditions and goals.

## Why Compare Strategies?

In algorithmic trading, it's important to objectively evaluate and compare strategies for the following reasons:

| Reason | Description |
|--------|-------------|
| **Choose the best** | Determine which strategy is most effective |
| **Diversification** | Select uncorrelated strategies to reduce risk |
| **Portfolio optimization** | Distribute capital among multiple strategies |
| **Understand trade-offs** | Recognize the balance between risk and return |
| **Market adaptation** | Select strategies for current market conditions |

## Metrics for Comparison

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    entry_time: u64,
    exit_time: u64,
    entry_price: f64,
    exit_price: f64,
    size: f64,
    pnl: f64,
    commission: f64,
}

#[derive(Debug)]
struct StrategyMetrics {
    name: String,
    // Returns
    total_return: f64,
    annual_return: f64,
    // Risk
    volatility: f64,
    max_drawdown: f64,
    // Efficiency
    sharpe_ratio: f64,
    sortino_ratio: f64,
    calmar_ratio: f64,
    // Trading activity
    total_trades: usize,
    win_rate: f64,
    profit_factor: f64,
    avg_trade: f64,
    avg_win: f64,
    avg_loss: f64,
}

impl StrategyMetrics {
    fn new(name: &str, trades: &[Trade], initial_capital: f64, days: usize) -> Self {
        let total_trades = trades.len();

        // Calculate returns
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_return = total_pnl / initial_capital;
        let annual_return = total_return * (365.0 / days as f64);

        // Calculate returns per trade
        let returns: Vec<f64> = trades.iter().map(|t| t.pnl / initial_capital).collect();
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Volatility (standard deviation of returns)
        let variance = returns.iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt() * (252.0_f64).sqrt(); // Annualized

        // Maximum drawdown
        let mut equity = initial_capital;
        let mut peak = initial_capital;
        let mut max_dd = 0.0;

        for trade in trades {
            equity += trade.pnl;
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        // Sharpe Ratio
        let risk_free_rate = 0.02; // 2% annual
        let sharpe_ratio = if volatility > 0.0 {
            (annual_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        // Sortino Ratio (uses only downside volatility)
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();
        let downside_variance = if !downside_returns.is_empty() {
            downside_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / downside_returns.len() as f64
        } else {
            0.0
        };
        let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt();
        let sortino_ratio = if downside_deviation > 0.0 {
            (annual_return - risk_free_rate) / downside_deviation
        } else {
            0.0
        };

        // Calmar Ratio (return / maximum drawdown)
        let calmar_ratio = if max_dd > 0.0 {
            annual_return / max_dd
        } else {
            0.0
        };

        // Trading metrics
        let winning_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl < 0.0).collect();

        let win_rate = winning_trades.len() as f64 / total_trades as f64;
        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|t| t.pnl).sum::<f64>() / winning_trades.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|t| t.pnl).sum::<f64>() / losing_trades.len() as f64
        } else {
            0.0
        };
        let avg_trade = total_pnl / total_trades as f64;

        let total_wins: f64 = winning_trades.iter().map(|t| t.pnl).sum();
        let total_losses: f64 = losing_trades.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        StrategyMetrics {
            name: name.to_string(),
            total_return,
            annual_return,
            volatility,
            max_drawdown: max_dd,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            total_trades,
            win_rate,
            profit_factor,
            avg_trade,
            avg_win,
            avg_loss,
        }
    }

    fn print(&self) {
        println!("\n=== {} ===", self.name);
        println!("Returns:");
        println!("  Total return: {:.2}%", self.total_return * 100.0);
        println!("  Annual return: {:.2}%", self.annual_return * 100.0);
        println!("\nRisk:");
        println!("  Volatility: {:.2}%", self.volatility * 100.0);
        println!("  Max drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("\nEfficiency:");
        println!("  Sharpe Ratio: {:.2}", self.sharpe_ratio);
        println!("  Sortino Ratio: {:.2}", self.sortino_ratio);
        println!("  Calmar Ratio: {:.2}", self.calmar_ratio);
        println!("\nTrading:");
        println!("  Total trades: {}", self.total_trades);
        println!("  Win Rate: {:.2}%", self.win_rate * 100.0);
        println!("  Profit Factor: {:.2}", self.profit_factor);
        println!("  Average trade: ${:.2}", self.avg_trade);
        println!("  Average win: ${:.2}", self.avg_win);
        println!("  Average loss: ${:.2}", self.avg_loss);
    }
}

// Generate test data
fn generate_trades(
    strategy_type: &str,
    num_trades: usize,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    volatility_factor: f64,
) -> Vec<Trade> {
    let mut trades = Vec::new();
    let mut time = 1000000u64;

    for i in 0..num_trades {
        let is_win = (i as f64 / num_trades as f64) < win_rate;
        let base_pnl = if is_win { avg_win } else { avg_loss };

        // Add variability
        let noise = ((i * 7 + 13) % 100) as f64 / 100.0 - 0.5;
        let pnl = base_pnl * (1.0 + noise * volatility_factor);

        trades.push(Trade {
            entry_time: time,
            exit_time: time + 3600,
            entry_price: 42000.0,
            exit_price: 42000.0 + pnl,
            size: 1.0,
            pnl,
            commission: 5.0,
        });

        time += 7200;
    }

    trades
}

fn main() {
    let initial_capital = 10000.0;
    let days = 365;

    // Strategy A: Aggressive scalper
    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let metrics_a = StrategyMetrics::new("Strategy A: Aggressive Scalper", &trades_a, initial_capital, days);

    // Strategy B: Conservative long-term
    let trades_b = generate_trades("Position", 50, 0.65, 300.0, -150.0, 0.2);
    let metrics_b = StrategyMetrics::new("Strategy B: Conservative Position", &trades_b, initial_capital, days);

    // Strategy C: Swing trader
    let trades_c = generate_trades("Swing", 150, 0.58, 80.0, -60.0, 0.25);
    let metrics_c = StrategyMetrics::new("Strategy C: Swing Trader", &trades_c, initial_capital, days);

    metrics_a.print();
    metrics_b.print();
    metrics_c.print();
}
```

## Strategy Comparison Table

```rust
struct StrategyComparison {
    strategies: Vec<StrategyMetrics>,
}

impl StrategyComparison {
    fn new(strategies: Vec<StrategyMetrics>) -> Self {
        StrategyComparison { strategies }
    }

    fn print_comparison_table(&self) {
        println!("\n{:=<120}", "");
        println!("STRATEGY COMPARISON TABLE");
        println!("{:=<120}", "");

        // Header
        print!("{:<30}", "Metric");
        for strategy in &self.strategies {
            print!("{:>28}", strategy.name.split(':').next().unwrap_or(&strategy.name));
        }
        println!();
        println!("{:-<120}", "");

        // Returns
        self.print_row("Annual Return (%)", |s| s.annual_return * 100.0);
        self.print_row("Total Return (%)", |s| s.total_return * 100.0);

        println!("{:-<120}", "");

        // Risk
        self.print_row("Volatility (%)", |s| s.volatility * 100.0);
        self.print_row("Max Drawdown (%)", |s| s.max_drawdown * 100.0);

        println!("{:-<120}", "");

        // Efficiency
        self.print_row("Sharpe Ratio", |s| s.sharpe_ratio);
        self.print_row("Sortino Ratio", |s| s.sortino_ratio);
        self.print_row("Calmar Ratio", |s| s.calmar_ratio);

        println!("{:-<120}", "");

        // Trading
        self.print_row("Total Trades", |s| s.total_trades as f64);
        self.print_row("Win Rate (%)", |s| s.win_rate * 100.0);
        self.print_row("Profit Factor", |s| s.profit_factor);

        println!("{:=<120}", "");
    }

    fn print_row<F>(&self, label: &str, extract: F)
    where
        F: Fn(&StrategyMetrics) -> f64,
    {
        print!("{:<30}", label);
        for strategy in &self.strategies {
            let value = extract(strategy);
            print!("{:>28.2}", value);
        }
        println!();
    }

    fn rank_strategies(&self) {
        println!("\n=== STRATEGY RANKINGS BY CRITERIA ===\n");

        // Ranking by Sharpe Ratio
        let mut by_sharpe: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_sharpe.sort_by(|a, b| b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap());

        println!("By Sharpe Ratio:");
        for (i, strategy) in by_sharpe.iter().enumerate() {
            println!("  {}. {} - {:.2}", i + 1, strategy.name, strategy.sharpe_ratio);
        }

        // Ranking by return
        let mut by_return: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_return.sort_by(|a, b| b.annual_return.partial_cmp(&a.annual_return).unwrap());

        println!("\nBy Annual Return:");
        for (i, strategy) in by_return.iter().enumerate() {
            println!("  {}. {} - {:.2}%", i + 1, strategy.name, strategy.annual_return * 100.0);
        }

        // Ranking by drawdown (lower = better)
        let mut by_dd: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_dd.sort_by(|a, b| a.max_drawdown.partial_cmp(&b.max_drawdown).unwrap());

        println!("\nBy Minimum Drawdown:");
        for (i, strategy) in by_dd.iter().enumerate() {
            println!("  {}. {} - {:.2}%", i + 1, strategy.name, strategy.max_drawdown * 100.0);
        }

        // Ranking by Profit Factor
        let mut by_pf: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_pf.sort_by(|a, b| b.profit_factor.partial_cmp(&a.profit_factor).unwrap());

        println!("\nBy Profit Factor:");
        for (i, strategy) in by_pf.iter().enumerate() {
            println!("  {}. {} - {:.2}", i + 1, strategy.name, strategy.profit_factor);
        }
    }

    fn find_best(&self, risk_tolerance: f64) -> Option<&StrategyMetrics> {
        // Choose best strategy with risk tolerance
        // risk_tolerance: 0.0 (conservative) - 1.0 (aggressive)

        self.strategies.iter()
            .filter(|s| s.max_drawdown <= 0.1 + risk_tolerance * 0.2) // Drawdown limit
            .max_by(|a, b| {
                // Weighted score
                let score_a = a.sharpe_ratio * (1.0 - risk_tolerance) + a.annual_return * risk_tolerance;
                let score_b = b.sharpe_ratio * (1.0 - risk_tolerance) + b.annual_return * risk_tolerance;
                score_a.partial_cmp(&score_b).unwrap()
            })
    }
}

fn main() {
    let initial_capital = 10000.0;
    let days = 365;

    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 50, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 150, 0.58, 80.0, -60.0, 0.25);

    let metrics_a = StrategyMetrics::new("Strategy A: Scalper", &trades_a, initial_capital, days);
    let metrics_b = StrategyMetrics::new("Strategy B: Position", &trades_b, initial_capital, days);
    let metrics_c = StrategyMetrics::new("Strategy C: Swing", &trades_c, initial_capital, days);

    let comparison = StrategyComparison::new(vec![metrics_a, metrics_b, metrics_c]);

    comparison.print_comparison_table();
    comparison.rank_strategies();

    // Find best strategy for conservative investor
    println!("\n=== RECOMMENDATIONS ===\n");
    if let Some(best_conservative) = comparison.find_best(0.2) {
        println!("For conservative investor: {}", best_conservative.name);
    }

    // Find best strategy for aggressive investor
    if let Some(best_aggressive) = comparison.find_best(0.8) {
        println!("For aggressive investor: {}", best_aggressive.name);
    }
}
```

## Correlation Analysis

```rust
#[derive(Debug)]
struct StrategyCorrelation {
    strategy1: String,
    strategy2: String,
    correlation: f64,
}

fn calculate_correlation(trades1: &[Trade], trades2: &[Trade]) -> f64 {
    // Synchronize trades by time
    // Simplified version: take PnL percentage for each trade

    let returns1: Vec<f64> = trades1.iter().map(|t| t.pnl).collect();
    let returns2: Vec<f64> = trades2.iter().map(|t| t.pnl).collect();

    let n = returns1.len().min(returns2.len());
    if n == 0 {
        return 0.0;
    }

    let mean1 = returns1[..n].iter().sum::<f64>() / n as f64;
    let mean2 = returns2[..n].iter().sum::<f64>() / n as f64;

    let mut covariance = 0.0;
    let mut var1 = 0.0;
    let mut var2 = 0.0;

    for i in 0..n {
        let diff1 = returns1[i] - mean1;
        let diff2 = returns2[i] - mean2;
        covariance += diff1 * diff2;
        var1 += diff1 * diff1;
        var2 += diff2 * diff2;
    }

    let std1 = (var1 / n as f64).sqrt();
    let std2 = (var2 / n as f64).sqrt();

    if std1 == 0.0 || std2 == 0.0 {
        return 0.0;
    }

    covariance / n as f64 / (std1 * std2)
}

fn analyze_portfolio_diversification(strategies: &[(&str, Vec<Trade>)]) {
    println!("\n=== CORRELATION MATRIX ===\n");

    let n = strategies.len();
    let mut correlations = Vec::new();

    // Header
    print!("{:<20}", "");
    for (name, _) in strategies {
        print!("{:>15}", name);
    }
    println!();
    println!("{:-<80}", "");

    // Correlation matrix
    for i in 0..n {
        print!("{:<20}", strategies[i].0);
        for j in 0..n {
            let corr = if i == j {
                1.0
            } else {
                calculate_correlation(&strategies[i].1, &strategies[j].1)
            };

            print!("{:>15.2}", corr);

            if i < j {
                correlations.push(StrategyCorrelation {
                    strategy1: strategies[i].0.to_string(),
                    strategy2: strategies[j].0.to_string(),
                    correlation: corr,
                });
            }
        }
        println!();
    }

    println!("\n=== DIVERSIFICATION ANALYSIS ===\n");

    // Find uncorrelated pairs
    let mut uncorrelated: Vec<&StrategyCorrelation> = correlations.iter()
        .filter(|c| c.correlation.abs() < 0.3)
        .collect();
    uncorrelated.sort_by(|a, b| a.correlation.abs().partial_cmp(&b.correlation.abs()).unwrap());

    if !uncorrelated.is_empty() {
        println!("Least correlated pairs (good for diversification):");
        for corr in uncorrelated.iter().take(3) {
            println!("  {} <-> {}: correlation = {:.2}",
                corr.strategy1, corr.strategy2, corr.correlation);
        }
    }

    // Find highly correlated pairs
    let mut highly_correlated: Vec<&StrategyCorrelation> = correlations.iter()
        .filter(|c| c.correlation > 0.7)
        .collect();
    highly_correlated.sort_by(|a, b| b.correlation.partial_cmp(&a.correlation).unwrap());

    if !highly_correlated.is_empty() {
        println!("\nHighly correlated pairs (redundant in portfolio):");
        for corr in highly_correlated.iter().take(3) {
            println!("  {} <-> {}: correlation = {:.2}",
                corr.strategy1, corr.strategy2, corr.correlation);
        }
    }
}

fn main() {
    // Create strategies with different characteristics
    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 500, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 500, 0.58, 80.0, -60.0, 0.25);

    let strategies = vec![
        ("Scalper", trades_a),
        ("Position", trades_b),
        ("Swing", trades_c),
    ];

    analyze_portfolio_diversification(&strategies);
}
```

## Visualization (Text-Based)

```rust
fn plot_equity_curves(strategies: &[(&str, Vec<Trade>)], initial_capital: f64) {
    println!("\n=== EQUITY CURVES ===\n");

    let max_trades = strategies.iter().map(|(_, t)| t.len()).max().unwrap_or(0);
    let step = (max_trades / 50).max(1); // 50 points on chart

    for point in (0..max_trades).step_by(step) {
        print!("{:>4}: ", point);

        for (i, (name, trades)) in strategies.iter().enumerate() {
            let equity: f64 = initial_capital + trades[..point.min(trades.len())]
                .iter()
                .map(|t| t.pnl)
                .sum::<f64>();

            let normalized = ((equity - initial_capital) / initial_capital * 100.0) as i32;
            let bar_length = ((normalized + 50) / 5).max(0).min(30) as usize;

            let markers = ['█', '▓', '▒'];
            print!("{}", markers[i % markers.len()].to_string().repeat(bar_length));
            print!(" ");
        }
        println!();
    }

    println!("\nLegend:");
    for (i, (name, _)) in strategies.iter().enumerate() {
        let markers = ['█', '▓', '▒'];
        println!("  {} - {}", markers[i % markers.len()], name);
    }
}

fn main() {
    let initial_capital = 10000.0;

    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 500, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 500, 0.58, 80.0, -60.0, 0.25);

    let strategies = vec![
        ("Scalper", trades_a),
        ("Position", trades_b),
        ("Swing", trades_c),
    ];

    plot_equity_curves(&strategies, initial_capital);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Strategy comparison** | Systematic evaluation across multiple metrics |
| **Sharpe Ratio** | Risk-adjusted return |
| **Sortino Ratio** | Sharpe, but only considers downside volatility |
| **Calmar Ratio** | Return relative to maximum drawdown |
| **Correlation** | Measure of dependency between strategies |
| **Diversification** | Risk reduction through uncorrelated strategies |
| **Win Rate** | Percentage of profitable trades |
| **Profit Factor** | Ratio of profits to losses |

## Practical Exercises

1. **Basic comparison**: Implement a function to compare two strategies:
   - Calculate all main metrics (Sharpe, Sortino, Calmar, Profit Factor)
   - Build comparison table
   - Output recommendation on which strategy is better and why

2. **Correlation analysis**: Create an analyzer:
   - Calculate correlations between 5 different strategies
   - Build correlation matrix
   - Find pairs with correlation < 0.2 for diversification
   - Suggest optimal combination of 3 strategies

3. **Ranking**: Write a scoring system:
   - Assign weights to different metrics (return 30%, Sharpe 25%, drawdown 25%, Profit Factor 20%)
   - Calculate total score for each strategy
   - Sort strategies by score
   - Add ability to change weights based on risk profile

4. **Visualization**: Build text-based charts for comparison:
   - Equity curves of all strategies on one chart
   - Bar chart for metric comparison
   - Drawdown curves
   - Monthly returns comparison

## Homework

1. **Multi-strategy portfolio**: Create portfolio management system:
   - Multiple strategies work in parallel
   - Allocate capital proportionally to Sharpe Ratio
   - Disable strategies with drawdown > 15%
   - Rebalance every N trades

2. **Adaptive comparison**: Implement dynamic comparison:
   - Compare strategies on rolling windows (30, 90, 180 days)
   - Track how metrics change over time
   - Determine which strategy performs best in current conditions
   - Automatically switch between strategies

3. **Statistical significance**: Add validation:
   - Use t-test to compare mean returns
   - Calculate p-value for Sharpe Ratio difference
   - Determine if difference is statistically significant
   - Build confidence intervals for metrics

4. **Machine learning for strategy selection**: Create ML model:
   - Features: market volatility, trend, volume
   - Target: predict which strategy will perform best
   - Train on historical data
   - Test on out-of-sample data

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
