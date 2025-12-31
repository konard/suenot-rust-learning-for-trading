# Day 297: Bootstrap: Stability Assessment

## Trading Analogy

Imagine you've developed a profitable trading strategy that showed a 25% return over the past year. But here's the question: **Was this result genuine or just luck?**

It's like a poker player who won a tournament. Did they win because of skill, or did they just get lucky with the cards? To find out, you'd make them play the same tournament many times with different card combinations.

**Bootstrap** in backtesting works similarly: we "shuffle" historical trades to create thousands of alternative scenarios and see if the strategy remains profitable. If it consistently wins across all variations, it's robust. If profits disappear when trades are rearranged, it was likely luck.

## What is Bootstrap?

Bootstrap is a statistical resampling method that:

| Aspect | Description |
|--------|-------------|
| **Purpose** | Assess stability and statistical significance of results |
| **Approach** | Create multiple variations by resampling data with replacement |
| **Application** | Strategy validation, risk assessment, confidence intervals |
| **Sample Size** | Typically 1000-10000 bootstrap iterations |
| **Advantage** | No assumptions about data distribution required |
| **Result** | Confidence intervals and probability distributions of metrics |

## Simple Example: Trade Bootstrap

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

// Single trade
#[derive(Debug, Clone)]
struct Trade {
    pnl: f64,
    duration_hours: u32,
}

// Strategy backtest result
#[derive(Debug)]
struct BacktestResult {
    trades: Vec<Trade>,
    total_pnl: f64,
    win_rate: f64,
    avg_pnl_per_trade: f64,
}

impl BacktestResult {
    fn from_trades(trades: &[Trade]) -> Self {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (wins as f64 / trades.len() as f64) * 100.0;
        let avg_pnl = total_pnl / trades.len() as f64;

        Self {
            trades: trades.to_vec(),
            total_pnl,
            win_rate,
            avg_pnl_per_trade: avg_pnl,
        }
    }
}

// Bootstrap: resample trades with replacement
fn bootstrap_trades(original_trades: &[Trade], n_samples: usize) -> Vec<Trade> {
    let mut rng = thread_rng();
    (0..n_samples)
        .map(|_| original_trades.choose(&mut rng).unwrap().clone())
        .collect()
}

fn main() {
    // Original backtest: 100 trades
    let original_trades = vec![
        Trade { pnl: 150.0, duration_hours: 4 },
        Trade { pnl: -80.0, duration_hours: 3 },
        Trade { pnl: 200.0, duration_hours: 6 },
        Trade { pnl: -50.0, duration_hours: 2 },
        Trade { pnl: 180.0, duration_hours: 5 },
        // ... 95 more trades
        // For demo, we'll simulate them
    ];

    // Simulate realistic trades
    let mut all_trades = original_trades;
    for i in 0..95 {
        let pnl = if i % 3 == 0 {
            -50.0 - (i as f64 * 2.0) // Losses
        } else {
            100.0 + (i as f64 * 3.0) // Wins
        };
        all_trades.push(Trade {
            pnl,
            duration_hours: 2 + (i % 6) as u32,
        });
    }

    let original_result = BacktestResult::from_trades(&all_trades);

    println!("=== Original Backtest ===");
    println!("Total PnL: ${:.2}", original_result.total_pnl);
    println!("Win Rate: {:.2}%", original_result.win_rate);
    println!("Avg PnL/Trade: ${:.2}", original_result.avg_pnl_per_trade);

    // Bootstrap: 1000 iterations
    let n_bootstrap = 1000;
    let mut bootstrap_pnls = Vec::new();
    let mut bootstrap_win_rates = Vec::new();

    for _ in 0..n_bootstrap {
        let resampled_trades = bootstrap_trades(&all_trades, all_trades.len());
        let result = BacktestResult::from_trades(&resampled_trades);
        bootstrap_pnls.push(result.total_pnl);
        bootstrap_win_rates.push(result.win_rate);
    }

    // Calculate statistics
    bootstrap_pnls.sort_by(|a, b| a.partial_cmp(b).unwrap());
    bootstrap_win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let pnl_mean: f64 = bootstrap_pnls.iter().sum::<f64>() / bootstrap_pnls.len() as f64;
    let pnl_5th = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.05) as usize];
    let pnl_95th = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.95) as usize];

    let wr_mean: f64 = bootstrap_win_rates.iter().sum::<f64>() / bootstrap_win_rates.len() as f64;
    let wr_5th = bootstrap_win_rates[(bootstrap_win_rates.len() as f64 * 0.05) as usize];
    let wr_95th = bootstrap_win_rates[(bootstrap_win_rates.len() as f64 * 0.95) as usize];

    println!("\n=== Bootstrap Analysis ({} iterations) ===", n_bootstrap);
    println!("PnL 90% Confidence Interval: ${:.2} to ${:.2}", pnl_5th, pnl_95th);
    println!("PnL Mean: ${:.2}", pnl_mean);
    println!("Win Rate 90% CI: {:.2}% to {:.2}%", wr_5th, wr_95th);
    println!("Win Rate Mean: {:.2}%", wr_mean);

    // Check if strategy is robust
    let profitable_bootstrap = bootstrap_pnls.iter()
        .filter(|&&pnl| pnl > 0.0)
        .count();
    let probability_profitable = (profitable_bootstrap as f64 / n_bootstrap as f64) * 100.0;

    println!("\nProbability of profitability: {:.2}%", probability_profitable);
    if probability_profitable > 95.0 {
        println!("✓ Strategy is ROBUST - consistently profitable");
    } else if probability_profitable > 70.0 {
        println!("⚠ Strategy is MODERATELY STABLE");
    } else {
        println!("✗ Strategy is UNSTABLE - results may be due to luck");
    }
}
```

## Block Bootstrap: Preserving Dependencies

Real market data has dependencies (trends, volatility clustering). Block bootstrap preserves these:

```rust
// Block bootstrap: resample in chunks to preserve order
fn block_bootstrap(trades: &[Trade], block_size: usize) -> Vec<Trade> {
    let mut rng = thread_rng();
    let n_blocks = (trades.len() + block_size - 1) / block_size;

    let mut result = Vec::new();

    for _ in 0..n_blocks {
        // Random starting position
        let start = rng.gen_range(0..trades.len().saturating_sub(block_size));
        let end = (start + block_size).min(trades.len());

        result.extend_from_slice(&trades[start..end]);

        if result.len() >= trades.len() {
            break;
        }
    }

    result.truncate(trades.len());
    result
}

fn main() {
    let trades = generate_trades(200); // Generate sample trades

    println!("=== Block Bootstrap (preserves trade sequences) ===\n");

    let block_sizes = vec![5, 10, 20];

    for &block_size in &block_sizes {
        let mut bootstrap_pnls = Vec::new();

        for _ in 0..1000 {
            let resampled = block_bootstrap(&trades, block_size);
            let result = BacktestResult::from_trades(&resampled);
            bootstrap_pnls.push(result.total_pnl);
        }

        bootstrap_pnls.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = bootstrap_pnls[bootstrap_pnls.len() / 2];
        let ci_low = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.05) as usize];
        let ci_high = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.95) as usize];

        println!("Block size: {}", block_size);
        println!("  Median PnL: ${:.2}", median);
        println!("  90% CI: ${:.2} to ${:.2}", ci_low, ci_high);
    }
}

fn generate_trades(n: usize) -> Vec<Trade> {
    use rand::Rng;
    let mut rng = thread_rng();

    (0..n).map(|i| {
        let pnl = if i % 3 == 0 {
            -rng.gen_range(20.0..100.0)
        } else {
            rng.gen_range(50.0..200.0)
        };

        Trade {
            pnl,
            duration_hours: rng.gen_range(1..24),
        }
    }).collect()
}
```

## Advanced Example: Multi-Metric Bootstrap

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct DetailedTrade {
    pnl: f64,
    return_pct: f64,
    duration_hours: u32,
    max_adverse_excursion: f64, // Largest loss during trade
}

#[derive(Debug)]
struct DetailedMetrics {
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    profit_factor: f64,
}

impl DetailedMetrics {
    fn calculate(trades: &[DetailedTrade]) -> Self {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losses: Vec<_> = trades.iter().filter(|t| t.pnl < 0.0).collect();

        let win_rate = (wins.len() as f64 / trades.len() as f64) * 100.0;

        let gross_profit: f64 = wins.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losses.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            f64::INFINITY
        };

        // Simplified Sharpe (assuming daily returns)
        let returns: Vec<f64> = trades.iter().map(|t| t.return_pct).collect();
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        let sharpe_ratio = if std_dev > 0.0 {
            mean_return / std_dev * (252.0_f64).sqrt() // Annualized
        } else {
            0.0
        };

        // Max drawdown calculation
        let mut peak = 0.0;
        let mut max_dd = 0.0;
        let mut cumulative = 0.0;

        for trade in trades {
            cumulative += trade.pnl;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = peak - cumulative;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        Self {
            total_return: total_pnl,
            sharpe_ratio,
            max_drawdown: max_dd,
            win_rate,
            profit_factor,
        }
    }
}

struct BootstrapAnalyzer {
    n_iterations: usize,
}

impl BootstrapAnalyzer {
    fn new(n_iterations: usize) -> Self {
        Self { n_iterations }
    }

    fn analyze(&self, trades: &[DetailedTrade]) -> BootstrapReport {
        let mut all_metrics = Vec::new();

        for _ in 0..self.n_iterations {
            let resampled = self.resample(trades);
            let metrics = DetailedMetrics::calculate(&resampled);
            all_metrics.push(metrics);
        }

        BootstrapReport::from_metrics(all_metrics)
    }

    fn resample(&self, trades: &[DetailedTrade]) -> Vec<DetailedTrade> {
        let mut rng = thread_rng();
        (0..trades.len())
            .map(|_| trades.choose(&mut rng).unwrap().clone())
            .collect()
    }
}

#[derive(Debug)]
struct BootstrapReport {
    sharpe_mean: f64,
    sharpe_ci: (f64, f64),
    max_dd_mean: f64,
    max_dd_ci: (f64, f64),
    win_rate_mean: f64,
    win_rate_ci: (f64, f64),
    profit_factor_mean: f64,
    profit_factor_ci: (f64, f64),
}

impl BootstrapReport {
    fn from_metrics(mut metrics: Vec<DetailedMetrics>) -> Self {
        let n = metrics.len();

        // Extract and sort each metric
        let mut sharpes: Vec<f64> = metrics.iter().map(|m| m.sharpe_ratio).collect();
        let mut max_dds: Vec<f64> = metrics.iter().map(|m| m.max_drawdown).collect();
        let mut win_rates: Vec<f64> = metrics.iter().map(|m| m.win_rate).collect();
        let mut profit_factors: Vec<f64> = metrics.iter()
            .map(|m| if m.profit_factor.is_finite() { m.profit_factor } else { 10.0 })
            .collect();

        sharpes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        max_dds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
        profit_factors.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile = |sorted: &[f64], p: f64| -> f64 {
            sorted[(sorted.len() as f64 * p) as usize]
        };

        Self {
            sharpe_mean: sharpes.iter().sum::<f64>() / n as f64,
            sharpe_ci: (percentile(&sharpes, 0.05), percentile(&sharpes, 0.95)),
            max_dd_mean: max_dds.iter().sum::<f64>() / n as f64,
            max_dd_ci: (percentile(&max_dds, 0.05), percentile(&max_dds, 0.95)),
            win_rate_mean: win_rates.iter().sum::<f64>() / n as f64,
            win_rate_ci: (percentile(&win_rates, 0.05), percentile(&win_rates, 0.95)),
            profit_factor_mean: profit_factors.iter().sum::<f64>() / n as f64,
            profit_factor_ci: (percentile(&profit_factors, 0.05), percentile(&profit_factors, 0.95)),
        }
    }

    fn print(&self) {
        println!("\n=== Bootstrap Analysis Report ===\n");

        println!("Sharpe Ratio:");
        println!("  Mean: {:.3}", self.sharpe_mean);
        println!("  90% CI: [{:.3}, {:.3}]", self.sharpe_ci.0, self.sharpe_ci.1);

        println!("\nMax Drawdown:");
        println!("  Mean: ${:.2}", self.max_dd_mean);
        println!("  90% CI: [${:.2}, ${:.2}]", self.max_dd_ci.0, self.max_dd_ci.1);

        println!("\nWin Rate:");
        println!("  Mean: {:.2}%", self.win_rate_mean);
        println!("  90% CI: [{:.2}%, {:.2}%]", self.win_rate_ci.0, self.win_rate_ci.1);

        println!("\nProfit Factor:");
        println!("  Mean: {:.2}", self.profit_factor_mean);
        println!("  90% CI: [{:.2}, {:.2}]", self.profit_factor_ci.0, self.profit_factor_ci.1);

        // Assessment
        println!("\n=== Strategy Assessment ===");

        if self.sharpe_ci.0 > 1.0 {
            println!("✓ Strong confidence: Sharpe Ratio consistently > 1.0");
        } else if self.sharpe_ci.0 > 0.5 {
            println!("⚠ Moderate confidence: Sharpe Ratio variable but positive");
        } else {
            println!("✗ Low confidence: Sharpe Ratio unreliable");
        }

        if self.profit_factor_ci.0 > 1.5 {
            println!("✓ Robust profitability: Profit Factor consistently > 1.5");
        } else if self.profit_factor_ci.0 > 1.0 {
            println!("⚠ Marginal profitability: Profit Factor barely > 1.0");
        } else {
            println!("✗ Unprofitable in some scenarios");
        }
    }
}

fn main() {
    use rand::Rng;
    let mut rng = thread_rng();

    // Generate realistic trading history
    let trades: Vec<DetailedTrade> = (0..150).map(|i| {
        let is_win = i % 3 != 0; // 66% win rate
        let pnl = if is_win {
            rng.gen_range(100.0..300.0)
        } else {
            -rng.gen_range(50.0..150.0)
        };

        DetailedTrade {
            pnl,
            return_pct: pnl / 10000.0, // Assuming $10k position
            duration_hours: rng.gen_range(1..48),
            max_adverse_excursion: if is_win {
                rng.gen_range(0.0..50.0)
            } else {
                pnl.abs()
            },
        }
    }).collect();

    println!("Analyzing {} trades with bootstrap...", trades.len());

    let analyzer = BootstrapAnalyzer::new(5000);
    let report = analyzer.analyze(&trades);
    report.print();
}
```

## Parallel Bootstrap for Speed

```rust
use rayon::prelude::*;

fn parallel_bootstrap_analysis(
    trades: &[DetailedTrade],
    n_iterations: usize,
) -> Vec<DetailedMetrics> {
    (0..n_iterations)
        .into_par_iter()
        .map(|_| {
            let resampled = resample_trades(trades);
            DetailedMetrics::calculate(&resampled)
        })
        .collect()
}

fn resample_trades(trades: &[DetailedTrade]) -> Vec<DetailedTrade> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    (0..trades.len())
        .map(|_| trades.choose(&mut rng).unwrap().clone())
        .collect()
}

fn main() {
    let trades = generate_trades(300);

    println!("Running parallel bootstrap with 10,000 iterations...\n");

    let start = std::time::Instant::now();
    let metrics = parallel_bootstrap_analysis(&trades, 10000);
    let duration = start.elapsed();

    println!("Completed in {:?}", duration);
    println!("Processed {} bootstrap samples", metrics.len());

    let report = BootstrapReport::from_metrics(metrics);
    report.print();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Bootstrap | Resampling method for assessing result stability |
| Confidence Intervals | Range where true value likely lies (e.g., 90% CI) |
| Trade Resampling | Creating alternative histories by shuffling trades |
| Block Bootstrap | Preserving time dependencies in resampling |
| Multi-Metric Analysis | Simultaneously testing multiple performance metrics |
| Parallel Processing | Using Rayon to speed up thousands of iterations |

## Practical Exercises

1. **Basic Bootstrap**: Implement bootstrap analysis for a simple strategy with 50 trades. Calculate 95% confidence intervals for total PnL and win rate.

2. **Block vs Simple**: Compare simple bootstrap and block bootstrap (block size = 10) for a trending market strategy. Which gives more realistic results?

3. **Minimum Sample Size**: Experiment with different numbers of trades (20, 50, 100, 200). At what point do confidence intervals become acceptably narrow?

4. **Multiple Strategies**: Bootstrap 3 different strategies simultaneously and determine which has the most stable Sharpe Ratio.

## Homework

1. **Percentile Bootstrap**: Implement a function that calculates any percentile (e.g., 1st, 5th, 25th, 50th, 75th, 95th, 99th) for any metric, and visualize the full distribution.

2. **Strategy Comparison**: Use bootstrap to determine if Strategy A is statistically better than Strategy B. Calculate the probability that A outperforms B based on 10,000 bootstrap samples.

3. **Temporal Block Bootstrap**: Implement a moving block bootstrap that respects temporal ordering while resampling. Compare results with simple bootstrap on a mean-reversion strategy.

4. **Risk Metrics**: Extend bootstrap to calculate confidence intervals for:
   - Maximum consecutive losses
   - Value at Risk (VaR) at 95% and 99%
   - Conditional Value at Risk (CVaR)
   - Recovery time from drawdowns

## Navigation

[← Previous day](../296-monte-carlo-simulations/en.md) | [Next day →](../298-confidence-intervals/en.md)
