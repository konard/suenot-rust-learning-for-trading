# Day 292: Parameter Optimization

## Trading Analogy

Imagine you've developed a trading strategy based on moving averages:
- Buy when the fast MA crosses above the slow MA
- Sell when the fast MA crosses below the slow MA

But which periods should you use? 10 and 20? 5 and 15? 20 and 50? 50 and 200? Each combination will produce different results. **Parameter Optimization** is the systematic process of testing different combinations of strategy parameters to find those that perform best on historical data.

It's like tuning a musical instrument — you methodically adjust each string (parameter), checking how it affects the sound (strategy performance), to find the optimal configuration.

## What is Parameter Optimization?

**Parameter Optimization** is the process of finding the best values for a trading strategy's input parameters by systematically testing different combinations on historical data.

### Core Concepts

| Concept | Description | Trading Example |
|---------|-------------|-----------------|
| Parameter | Adjustable strategy value | MA period, stop-loss level, position size |
| Search Space | Range of possible parameter values | MA period: from 5 to 200 days |
| Objective Function | Metric to optimize | Sharpe Ratio, total return, max drawdown |
| Local Optimum | Best value in nearby area | MA(10,20) is good, but MA(50,200) might be better |
| Global Optimum | Best value across entire space | Absolute best parameter combination |

## Trading Strategy Parameters in Rust

Let's start by defining a strategy structure with parameters:

```rust
#[derive(Debug, Clone)]
struct MovingAverageCrossStrategy {
    fast_period: usize,   // Fast moving average
    slow_period: usize,   // Slow moving average
    stop_loss_pct: f64,   // Stop loss percentage
    take_profit_pct: f64, // Take profit percentage
}

impl MovingAverageCrossStrategy {
    fn new(fast_period: usize, slow_period: usize, stop_loss_pct: f64, take_profit_pct: f64) -> Self {
        Self {
            fast_period,
            slow_period,
            stop_loss_pct,
            take_profit_pct,
        }
    }

    // Calculate simple moving average
    fn calculate_sma(&self, prices: &[f64], period: usize) -> Vec<f64> {
        let mut sma = Vec::new();

        if period == 0 {
            return vec![0.0; prices.len()];
        }

        for i in 0..prices.len() {
            if i + 1 < period {
                sma.push(0.0); // Insufficient data
            } else {
                let sum: f64 = prices[i + 1 - period..=i].iter().sum();
                sma.push(sum / period as f64);
            }
        }

        sma
    }

    // Generate trading signals
    fn generate_signals(&self, prices: &[f64]) -> Vec<i8> {
        let fast_ma = self.calculate_sma(prices, self.fast_period);
        let slow_ma = self.calculate_sma(prices, self.slow_period);

        let mut signals = Vec::new();

        for i in 0..prices.len() {
            if i == 0 {
                signals.push(0); // No signal on first bar
                continue;
            }

            // Crossover up — buy
            if fast_ma[i - 1] <= slow_ma[i - 1] && fast_ma[i] > slow_ma[i] {
                signals.push(1); // Buy
            }
            // Crossover down — sell
            else if fast_ma[i - 1] >= slow_ma[i - 1] && fast_ma[i] < slow_ma[i] {
                signals.push(-1); // Sell
            }
            // No crossover
            else {
                signals.push(0); // Hold
            }
        }

        signals
    }
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0,
        107.0, 109.0, 111.0, 110.0, 112.0, 114.0, 113.0, 115.0,
    ];

    let strategy = MovingAverageCrossStrategy::new(3, 7, 2.0, 5.0);
    let signals = strategy.generate_signals(&prices);

    println!("Strategy Parameters:");
    println!("  Fast MA: {}", strategy.fast_period);
    println!("  Slow MA: {}", strategy.slow_period);
    println!("  Stop Loss: {}%", strategy.stop_loss_pct);
    println!("  Take Profit: {}%", strategy.take_profit_pct);
    println!("\nTrading Signals: {:?}", signals);
}
```

## Performance Metrics

To optimize parameters, we need an objective function to evaluate quality:

```rust
#[derive(Debug)]
struct BacktestResults {
    total_return: f64,
    num_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    max_drawdown: f64,
    sharpe_ratio: f64,
}

impl BacktestResults {
    fn new() -> Self {
        Self {
            total_return: 0.0,
            num_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
        }
    }

    fn calculate_win_rate(&self) -> f64 {
        if self.num_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.num_trades as f64) * 100.0
    }

    fn calculate_profit_factor(&self) -> f64 {
        if self.losing_trades == 0 {
            return f64::INFINITY;
        }
        self.winning_trades as f64 / self.losing_trades as f64
    }

    // Comprehensive strategy quality score
    fn fitness_score(&self) -> f64 {
        // Combination of different metrics
        let return_score = self.total_return;
        let sharpe_score = self.sharpe_ratio * 10.0;
        let drawdown_penalty = self.max_drawdown.abs();
        let trade_count_bonus = (self.num_trades as f64).sqrt();

        return_score + sharpe_score - drawdown_penalty + trade_count_bonus
    }
}

// Simple backtest for demonstration
fn backtest_strategy(strategy: &MovingAverageCrossStrategy, prices: &[f64]) -> BacktestResults {
    let mut results = BacktestResults::new();
    let signals = strategy.generate_signals(prices);

    let mut position = 0.0; // 0 = no position
    let mut entry_price = 0.0;
    let mut equity = 10000.0; // Initial capital
    let mut peak_equity = equity;
    let mut returns = Vec::new();

    for i in 1..prices.len() {
        // Open position on signal
        if position == 0.0 && signals[i] == 1 {
            position = equity / prices[i]; // Buy with full capital
            entry_price = prices[i];
            results.num_trades += 1;
        }
        // Close position on reverse signal
        else if position > 0.0 && signals[i] == -1 {
            let exit_value = position * prices[i];
            let trade_return = (exit_value - equity) / equity;
            returns.push(trade_return);

            if exit_value > equity {
                results.winning_trades += 1;
            } else {
                results.losing_trades += 1;
            }

            equity = exit_value;
            position = 0.0;
        }

        // Update equity
        let current_equity = if position > 0.0 {
            position * prices[i]
        } else {
            equity
        };

        // Track drawdown
        if current_equity > peak_equity {
            peak_equity = current_equity;
        }
        let drawdown = (current_equity - peak_equity) / peak_equity * 100.0;
        if drawdown < results.max_drawdown {
            results.max_drawdown = drawdown;
        }
    }

    results.total_return = ((equity - 10000.0) / 10000.0) * 100.0;

    // Simplified Sharpe Ratio calculation
    if !returns.is_empty() {
        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            results.sharpe_ratio = mean_return / std_dev;
        }
    }

    results
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0,
        107.0, 109.0, 111.0, 110.0, 112.0, 114.0, 113.0, 115.0,
        116.0, 115.0, 117.0, 119.0, 118.0, 120.0, 122.0, 121.0,
    ];

    let strategy = MovingAverageCrossStrategy::new(3, 7, 2.0, 5.0);
    let results = backtest_strategy(&strategy, &prices);

    println!("Backtest Results:");
    println!("  Total Return: {:.2}%", results.total_return);
    println!("  Number of Trades: {}", results.num_trades);
    println!("  Winning Trades: {}", results.winning_trades);
    println!("  Losing Trades: {}", results.losing_trades);
    println!("  Win Rate: {:.2}%", results.calculate_win_rate());
    println!("  Max Drawdown: {:.2}%", results.max_drawdown);
    println!("  Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("  Fitness Score: {:.2}", results.fitness_score());
}
```

## Simple Optimization: Parameter Sweep

Now let's optimize parameters by testing different combinations:

```rust
#[derive(Debug, Clone)]
struct OptimizationResult {
    fast_period: usize,
    slow_period: usize,
    fitness_score: f64,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
}

fn optimize_ma_periods(prices: &[f64]) -> Vec<OptimizationResult> {
    let mut results = Vec::new();

    // Parameter ranges for optimization
    let fast_periods = vec![3, 5, 7, 10, 12, 15];
    let slow_periods = vec![10, 15, 20, 25, 30, 40, 50];

    println!("Starting parameter optimization...\n");

    for &fast in &fast_periods {
        for &slow in &slow_periods {
            // Fast MA must be smaller than slow MA
            if fast >= slow {
                continue;
            }

            let strategy = MovingAverageCrossStrategy::new(fast, slow, 2.0, 5.0);
            let backtest_result = backtest_strategy(&strategy, prices);

            let opt_result = OptimizationResult {
                fast_period: fast,
                slow_period: slow,
                fitness_score: backtest_result.fitness_score(),
                total_return: backtest_result.total_return,
                sharpe_ratio: backtest_result.sharpe_ratio,
                max_drawdown: backtest_result.max_drawdown,
            };

            println!(
                "MA({}, {}) -> Return: {:.2}%, Sharpe: {:.2}, Drawdown: {:.2}%, Fitness: {:.2}",
                fast, slow,
                opt_result.total_return,
                opt_result.sharpe_ratio,
                opt_result.max_drawdown,
                opt_result.fitness_score
            );

            results.push(opt_result);
        }
    }

    results
}

fn find_best_parameters(results: &[OptimizationResult]) -> Option<&OptimizationResult> {
    results.iter()
        .max_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap())
}

fn main() {
    // Generate more realistic data
    let prices: Vec<f64> = (0..100)
        .map(|i| {
            100.0 + (i as f64 * 0.5) + ((i as f64 * 0.1).sin() * 5.0)
        })
        .collect();

    println!("Historical data: {} bars\n", prices.len());

    let results = optimize_ma_periods(&prices);

    println!("\n=== Optimization Results ===\n");

    if let Some(best) = find_best_parameters(&results) {
        println!("Best Parameters:");
        println!("  Fast MA: {}", best.fast_period);
        println!("  Slow MA: {}", best.slow_period);
        println!("  Total Return: {:.2}%", best.total_return);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
        println!("  Max Drawdown: {:.2}%", best.max_drawdown);
        println!("  Fitness Score: {:.2}", best.fitness_score);
    }

    // Top 5 best combinations
    println!("\nTop 5 Best Combinations:");
    let mut sorted_results = results.clone();
    sorted_results.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());

    for (i, result) in sorted_results.iter().take(5).enumerate() {
        println!(
            "{}. MA({}, {}) - Fitness: {:.2}, Return: {:.2}%",
            i + 1,
            result.fast_period,
            result.slow_period,
            result.fitness_score,
            result.total_return
        );
    }
}
```

## Multi-Parameter Optimization

Optimizing multiple parameters simultaneously:

```rust
#[derive(Debug, Clone)]
struct FullOptimizationResult {
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
    take_profit: f64,
    fitness_score: f64,
    total_return: f64,
}

fn optimize_all_parameters(prices: &[f64]) -> Vec<FullOptimizationResult> {
    let mut results = Vec::new();
    let mut tested = 0;

    let fast_periods = vec![5, 10, 15];
    let slow_periods = vec![20, 30, 40];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 7.0];

    let total_combinations = fast_periods.len()
        * slow_periods.len()
        * stop_losses.len()
        * take_profits.len();

    println!("Testing {} parameter combinations...\n", total_combinations);

    for &fast in &fast_periods {
        for &slow in &slow_periods {
            if fast >= slow {
                continue;
            }

            for &sl in &stop_losses {
                for &tp in &take_profits {
                    tested += 1;

                    let strategy = MovingAverageCrossStrategy::new(fast, slow, sl, tp);
                    let backtest = backtest_strategy(&strategy, prices);

                    let result = FullOptimizationResult {
                        fast_period: fast,
                        slow_period: slow,
                        stop_loss: sl,
                        take_profit: tp,
                        fitness_score: backtest.fitness_score(),
                        total_return: backtest.total_return,
                    };

                    if tested % 10 == 0 {
                        println!(
                            "Progress: {}/{} ({:.1}%)",
                            tested, total_combinations,
                            (tested as f64 / total_combinations as f64) * 100.0
                        );
                    }

                    results.push(result);
                }
            }
        }
    }

    println!("\nOptimization completed!\n");
    results
}

fn main() {
    let prices: Vec<f64> = (0..200)
        .map(|i| {
            100.0 + (i as f64 * 0.3) + ((i as f64 * 0.1).sin() * 8.0)
        })
        .collect();

    let results = optimize_all_parameters(&prices);

    // Find best result
    let best = results.iter()
        .max_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap())
        .unwrap();

    println!("=== Optimal Parameters ===");
    println!("Fast MA: {}", best.fast_period);
    println!("Slow MA: {}", best.slow_period);
    println!("Stop Loss: {}%", best.stop_loss);
    println!("Take Profit: {}%", best.take_profit);
    println!("Total Return: {:.2}%", best.total_return);
    println!("Fitness Score: {:.2}", best.fitness_score);
}
```

## Important Optimization Warnings

```rust
// DANGER: Overfitting

fn demonstrate_overfitting_risk() {
    println!("⚠️  PARAMETER OPTIMIZATION DANGERS ⚠️\n");

    println!("1. OVERFITTING:");
    println!("   - Parameters work perfectly on historical data");
    println!("   - But perform poorly on new data");
    println!("   - Solution: out-of-sample testing\n");

    println!("2. DATA SNOOPING:");
    println!("   - Repeated optimization on same data");
    println!("   - Parameters fitted to random fluctuations");
    println!("   - Solution: validation holdout set\n");

    println!("3. OPTIMIZATION BIAS:");
    println!("   - Selecting only best results");
    println!("   - Ignoring poor results from other parameters");
    println!("   - Solution: walk-forward analysis\n");

    println!("4. CURVE FITTING:");
    println!("   - Too many parameters to optimize");
    println!("   - Strategy too complex");
    println!("   - Solution: simplicity and robustness\n");
}

fn main() {
    demonstrate_overfitting_risk();

    println!("=== SAFE OPTIMIZATION RULES ===\n");

    println!("✓ Use in-sample and out-of-sample periods");
    println!("✓ Check parameter stability (sensitivity analysis)");
    println!("✓ Prefer simple strategies to complex ones");
    println!("✓ Use cross-validation");
    println!("✓ Test across different market conditions");
    println!("✓ Track parameter robustness");
}
```

## Parameter Space Visualization

```rust
use std::collections::HashMap;

fn create_heatmap(results: &[OptimizationResult]) {
    let mut heatmap: HashMap<(usize, usize), f64> = HashMap::new();

    for result in results {
        heatmap.insert(
            (result.fast_period, result.slow_period),
            result.fitness_score
        );
    }

    println!("\n=== FITNESS SCORE HEATMAP ===\n");
    println!("     Slow MA Period");
    print!("Fast  ");

    let slow_periods: Vec<usize> = vec![10, 15, 20, 25, 30, 40, 50];
    let fast_periods: Vec<usize> = vec![3, 5, 7, 10, 12, 15];

    for slow in &slow_periods {
        print!("{:>6} ", slow);
    }
    println!();

    for fast in &fast_periods {
        print!("{:>4}  ", fast);
        for slow in &slow_periods {
            if let Some(&score) = heatmap.get(&(*fast, *slow)) {
                let symbol = if score > 50.0 {
                    "██"
                } else if score > 30.0 {
                    "▓▓"
                } else if score > 10.0 {
                    "▒▒"
                } else if score > 0.0 {
                    "░░"
                } else {
                    "  "
                };
                print!("{} ", symbol);
            } else {
                print!("   ");
            }
        }
        println!();
    }

    println!("\nLegend: ██ > 50  ▓▓ > 30  ▒▒ > 10  ░░ > 0");
}

fn main() {
    let prices: Vec<f64> = (0..100)
        .map(|i| 100.0 + (i as f64 * 0.5) + ((i as f64 * 0.1).sin() * 5.0))
        .collect();

    let results = optimize_ma_periods(&prices);
    create_heatmap(&results);
}
```

## Practical Exercises

### Exercise 1: Single-Parameter Optimization
Optimize only the RSI (Relative Strength Index) period from 5 to 30. Find the optimal period for identifying overbought/oversold conditions.

### Exercise 2: Two-Parameter Optimization
Optimize a Bollinger Bands strategy:
- Moving average period (10-30)
- Number of standard deviations (1.5-3.0)

### Exercise 3: Objective Function Comparison
Implement optimization with different objective functions:
- Maximize total profit
- Maximize Sharpe Ratio
- Minimize maximum drawdown

Compare the optimal parameters found.

### Exercise 4: Robustness Testing
Find optimal parameters and test their stability:
- Change parameters by ±10%
- Measure performance change
- Find parameters with highest robustness

## Homework

1. **Optimizer with Progress Bar**: Create an optimizer that shows detailed progress:
   - Current parameter combination
   - Completion percentage
   - Estimated time to completion
   - Best result found so far

2. **Parallel Optimization**: Using threads or `rayon`, parallelize the optimization process. Test different parameter combinations simultaneously and compare speed with sequential execution.

3. **In-Sample / Out-of-Sample**: Split data into two parts:
   - In-sample (60%): for optimization
   - Out-of-sample (40%): for validation

   Find parameters on in-sample, then test on out-of-sample. Compare results.

4. **Sensitivity Analysis**: Create parameter sensitivity analysis:
   - Find optimal parameters
   - Systematically vary each parameter in ±30% range
   - Plot graphs showing each parameter's impact
   - Identify which parameters are most critical

5. **Multi-Objective Optimization**: Implement optimization with multiple objectives:
   - Maximize returns
   - Minimize drawdown
   - Minimize number of trades (commissions)

   Use weighted score or Pareto frontier to balance objectives.

## What We Learned

| Concept | Description |
|---------|-------------|
| Parameter Optimization | Systematic search for best strategy parameter values |
| Objective Function | Metric for evaluating parameter quality (Sharpe, profit, drawdown) |
| Search Space | Range of possible values for each parameter |
| Fitness Score | Comprehensive strategy quality assessment |
| Overfitting | Risk of optimizing to historical data |
| In-sample / Out-of-sample | Data splitting for training and validation |
| Robustness | Parameter stability to changes |
| Sensitivity Analysis | Analyzing strategy sensitivity to parameters |

## Navigation

[← Previous Day](../291-out-of-sample-testing/en.md) | [Next Day →](../293-grid-search-parameter-sweep/en.md)
