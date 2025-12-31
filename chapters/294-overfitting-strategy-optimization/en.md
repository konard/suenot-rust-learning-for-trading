# Day 294: Overfitting: Strategy Over-Optimization

## Trading Analogy

Imagine a trader who analyzes the Bitcoin chart for the last month and finds a "perfect" strategy: buy at every local minimum and sell at every maximum. They backtest on historical data — 500% profit! But when applied to the real market — the strategy fails miserably.

This is a classic example of **overfitting**:
- The strategy is "tuned" to a specific historical period
- It memorized all random market fluctuations
- Instead of general patterns, it learned noise
- On new data, it's useless

Like a student who crams specific problems before an exam instead of understanding principles — they excel at familiar problems but get lost with any slight change in conditions.

## Why is overfitting dangerous in algorithmic trading?

In machine learning and strategy backtesting, overfitting occurs when a model or algorithm fits the training sample too precisely:

| Problem | Cause | Consequence |
|---------|-------|-------------|
| Too many parameters | 50+ indicators, each tuned to hundredths | Strategy "memorizes" noise |
| Small data sample | Testing on 2 weeks | Doesn't account for different market conditions |
| Look-ahead bias | Using future data | Impossible to reproduce in reality |
| Data snooping | Testing thousands of variations | Finding random correlations |
| Survivorship bias | Testing only on surviving assets | Ignoring bankruptcies |

## Signs of Overfitting

```rust
#[derive(Debug)]
struct BacktestResult {
    train_sharpe: f64,      // Sharpe ratio on training set
    test_sharpe: f64,       // Sharpe ratio on test set
    train_profit: f64,      // Profit on training
    test_profit: f64,       // Profit on test
    num_parameters: usize,  // Number of strategy parameters
    num_trades: usize,      // Number of trades
}

impl BacktestResult {
    /// Check for overfitting
    fn is_overfitted(&self) -> bool {
        // Sign 1: Sharpe ratio on test much worse than on training
        let sharpe_degradation = (self.train_sharpe - self.test_sharpe) / self.train_sharpe;

        // Sign 2: Too many parameters relative to number of trades
        let parameter_ratio = self.num_parameters as f64 / self.num_trades as f64;

        // Sign 3: Profit reversal - positive on training, negative on test
        let profit_reversal = self.train_profit > 0.0 && self.test_profit < 0.0;

        sharpe_degradation > 0.3 || parameter_ratio > 0.1 || profit_reversal
    }

    fn print_diagnosis(&self) {
        println!("=== Backtest Diagnosis ===");
        println!("Training set:");
        println!("  Sharpe ratio: {:.2}", self.train_sharpe);
        println!("  Profit: {:.2}%", self.train_profit * 100.0);
        println!("\nTest set:");
        println!("  Sharpe ratio: {:.2}", self.test_sharpe);
        println!("  Profit: {:.2}%", self.test_profit * 100.0);
        println!("\nParameters:");
        println!("  Number of parameters: {}", self.num_parameters);
        println!("  Number of trades: {}", self.num_trades);
        println!("  Ratio: {:.3}", self.num_parameters as f64 / self.num_trades as f64);

        if self.is_overfitted() {
            println!("\n⚠️  WARNING: Signs of overfitting detected!");
        } else {
            println!("\n✅ Strategy looks robust");
        }
    }
}

fn main() {
    // Good strategy
    let good_strategy = BacktestResult {
        train_sharpe: 1.8,
        test_sharpe: 1.6,
        train_profit: 0.35,
        test_profit: 0.28,
        num_parameters: 5,
        num_trades: 150,
    };

    good_strategy.print_diagnosis();
    println!();

    // Overfitted strategy
    let overfitted_strategy = BacktestResult {
        train_sharpe: 3.5,
        test_sharpe: 0.8,
        train_profit: 0.85,
        test_profit: -0.12,
        num_parameters: 25,
        num_trades: 80,
    };

    overfitted_strategy.print_diagnosis();
}
```

## Example: Optimization with Overfitting Protection

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct StrategyParams {
    ma_short: usize,      // Short moving average period
    ma_long: usize,       // Long moving average period
    stop_loss: f64,       // Stop-loss in percentage
    take_profit: f64,     // Take-profit in percentage
}

impl StrategyParams {
    fn count_params(&self) -> usize {
        4 // Number of tunable parameters
    }
}

#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
}

/// Simple MA crossover strategy
fn backtest_strategy(prices: &[f64], params: &StrategyParams) -> Vec<Trade> {
    let mut trades = Vec::new();

    if prices.len() < params.ma_long {
        return trades;
    }

    let mut position_open = false;
    let mut entry_price = 0.0;

    for i in params.ma_long..prices.len() {
        // Calculate moving averages
        let short_ma: f64 = prices[i - params.ma_short..i].iter().sum::<f64>()
            / params.ma_short as f64;
        let long_ma: f64 = prices[i - params.ma_long..i].iter().sum::<f64>()
            / params.ma_long as f64;

        let prev_short_ma: f64 = prices[i - params.ma_short - 1..i - 1].iter().sum::<f64>()
            / params.ma_short as f64;
        let prev_long_ma: f64 = prices[i - params.ma_long - 1..i - 1].iter().sum::<f64>()
            / params.ma_long as f64;

        // Buy signal: short MA crosses long MA from below
        if !position_open && prev_short_ma <= prev_long_ma && short_ma > long_ma {
            position_open = true;
            entry_price = prices[i];
        }

        // Check for position close
        if position_open {
            let current_pnl = (prices[i] - entry_price) / entry_price;

            // Stop-loss or take-profit
            if current_pnl <= -params.stop_loss || current_pnl >= params.take_profit {
                trades.push(Trade {
                    entry_price,
                    exit_price: prices[i],
                    pnl: current_pnl,
                });
                position_open = false;
            }
            // Sell signal: short MA crosses long MA from above
            else if prev_short_ma >= prev_long_ma && short_ma < long_ma {
                trades.push(Trade {
                    entry_price,
                    exit_price: prices[i],
                    pnl: current_pnl,
                });
                position_open = false;
            }
        }
    }

    trades
}

/// Calculate Sharpe ratio
fn calculate_sharpe(trades: &[Trade]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let returns: Vec<f64> = trades.iter().map(|t| t.pnl).collect();
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    let variance = returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;

    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return 0.0;
    }

    // Annualized Sharpe (assuming 252 trading days)
    mean_return / std_dev * (252.0_f64).sqrt()
}

/// Walk-forward analysis for overfitting protection
fn walk_forward_analysis(prices: &[f64], params: &StrategyParams) -> BacktestResult {
    // Split data: 70% training, 30% test
    let split_point = (prices.len() as f64 * 0.7) as usize;

    let train_prices = &prices[..split_point];
    let test_prices = &prices[split_point..];

    // Backtest on training set
    let train_trades = backtest_strategy(train_prices, params);
    let train_sharpe = calculate_sharpe(&train_trades);
    let train_profit: f64 = train_trades.iter().map(|t| t.pnl).sum();

    // Backtest on test set
    let test_trades = backtest_strategy(test_prices, params);
    let test_sharpe = calculate_sharpe(&test_trades);
    let test_profit: f64 = test_trades.iter().map(|t| t.pnl).sum();

    BacktestResult {
        train_sharpe,
        test_sharpe,
        train_profit,
        test_profit,
        num_parameters: params.count_params(),
        num_trades: train_trades.len() + test_trades.len(),
    }
}

/// Generate test data (price simulation)
fn generate_price_data(days: usize, start_price: f64) -> Vec<f64> {
    let mut prices = Vec::with_capacity(days);
    let mut price = start_price;

    for i in 0..days {
        // Simple trend with noise
        let trend = (i as f64 * 0.01).sin() * 10.0;
        let noise = ((i * 7) % 17) as f64 - 8.0; // Deterministic "noise"
        price += trend + noise;
        prices.push(price);
    }

    prices
}

fn main() {
    let prices = generate_price_data(500, 42000.0);

    println!("=== Testing Strategies for Overfitting ===\n");

    // Simple strategy with few parameters
    let simple_params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    println!("1. Simple strategy (4 parameters):");
    let simple_result = walk_forward_analysis(&prices, &simple_params);
    simple_result.print_diagnosis();
    println!();

    // Complex strategy (simulating overfitting through fine-tuning)
    let complex_params = StrategyParams {
        ma_short: 7,
        ma_long: 43,
        stop_loss: 0.0234,
        take_profit: 0.1567,
    };

    println!("2. Over-optimized strategy (4 parameters, but too precise tuning):");
    println!("   (parameters too precisely tuned: 7, 43, 0.0234, 0.1567)");
    let complex_result = walk_forward_analysis(&prices, &complex_params);
    complex_result.print_diagnosis();
}
```

## Methods to Combat Overfitting

### 1. Cross-Validation

```rust
/// K-fold cross-validation for backtesting
fn k_fold_validation(prices: &[f64], params: &StrategyParams, k: usize) -> Vec<f64> {
    let fold_size = prices.len() / k;
    let mut sharpe_ratios = Vec::new();

    for i in 0..k {
        // Use i-th fold as test, rest as training
        let test_start = i * fold_size;
        let test_end = (i + 1) * fold_size;

        let test_fold = &prices[test_start..test_end];
        let trades = backtest_strategy(test_fold, params);

        sharpe_ratios.push(calculate_sharpe(&trades));
    }

    sharpe_ratios
}

fn main() {
    let prices = generate_price_data(500, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    let sharpe_ratios = k_fold_validation(&prices, &params, 5);

    println!("=== 5-Fold Cross-Validation ===");
    for (i, sharpe) in sharpe_ratios.iter().enumerate() {
        println!("Fold {}: Sharpe = {:.2}", i + 1, sharpe);
    }

    let mean_sharpe = sharpe_ratios.iter().sum::<f64>() / sharpe_ratios.len() as f64;
    let std_sharpe = {
        let variance = sharpe_ratios.iter()
            .map(|s| (s - mean_sharpe).powi(2))
            .sum::<f64>() / sharpe_ratios.len() as f64;
        variance.sqrt()
    };

    println!("\nMean Sharpe: {:.2} ± {:.2}", mean_sharpe, std_sharpe);

    if std_sharpe / mean_sharpe.abs() > 0.5 {
        println!("⚠️  High variability — possible overfitting!");
    } else {
        println!("✅ Strategy is stable across different periods");
    }
}
```

### 2. Regularization: Complexity Constraint

```rust
#[derive(Debug)]
struct RegularizedStrategy {
    params: StrategyParams,
    complexity_penalty: f64,
}

impl RegularizedStrategy {
    /// Evaluation with complexity penalty (Akaike Information Criterion)
    fn calculate_aic(&self, trades: &[Trade]) -> f64 {
        let n = trades.len() as f64;
        let k = self.params.count_params() as f64;

        if n == 0.0 {
            return f64::INFINITY;
        }

        // Log-likelihood (simplified through mean profit)
        let mean_pnl = trades.iter().map(|t| t.pnl).sum::<f64>() / n;
        let log_likelihood = -n * mean_pnl.abs().ln();

        // AIC = 2k - 2ln(L)
        // Lower AIC is better (balance between accuracy and simplicity)
        2.0 * k - 2.0 * log_likelihood + self.complexity_penalty * k
    }

    fn evaluate(&self, prices: &[f64]) -> f64 {
        let trades = backtest_strategy(prices, &self.params);
        self.calculate_aic(&trades)
    }
}

fn main() {
    let prices = generate_price_data(500, 42000.0);

    let strategies = vec![
        RegularizedStrategy {
            params: StrategyParams {
                ma_short: 10,
                ma_long: 50,
                stop_loss: 0.05,
                take_profit: 0.10,
            },
            complexity_penalty: 1.0,
        },
        RegularizedStrategy {
            params: StrategyParams {
                ma_short: 7,
                ma_long: 43,
                stop_loss: 0.0234,
                take_profit: 0.1567,
            },
            complexity_penalty: 1.0,
        },
    ];

    println!("=== Comparing Strategies with Complexity (AIC) ===");
    for (i, strategy) in strategies.iter().enumerate() {
        let aic = strategy.evaluate(&prices);
        println!("Strategy {}: AIC = {:.2}", i + 1, aic);
    }
    println!("\nLower AIC value = better balance accuracy/simplicity");
}
```

### 3. Out-of-Sample Testing

```rust
/// Strict separation: training -> validation -> test
fn three_way_split_test(prices: &[f64], params: &StrategyParams) {
    let n = prices.len();
    let train_end = n * 50 / 100;  // 50% training
    let val_end = n * 75 / 100;    // 25% validation
    // 25% test

    let train = &prices[..train_end];
    let validation = &prices[train_end..val_end];
    let test = &prices[val_end..];

    let train_trades = backtest_strategy(train, params);
    let val_trades = backtest_strategy(validation, params);
    let test_trades = backtest_strategy(test, params);

    println!("=== Three-Way Data Split ===");
    println!("Training (50%):   Sharpe = {:.2}", calculate_sharpe(&train_trades));
    println!("Validation (25%): Sharpe = {:.2}", calculate_sharpe(&val_trades));
    println!("Test (25%):       Sharpe = {:.2}", calculate_sharpe(&test_trades));

    let train_sharpe = calculate_sharpe(&train_trades);
    let test_sharpe = calculate_sharpe(&test_trades);

    if (train_sharpe - test_sharpe).abs() / train_sharpe > 0.3 {
        println!("\n⚠️  Performance on test significantly differs!");
    } else {
        println!("\n✅ Strategy generalizes well");
    }
}

fn main() {
    let prices = generate_price_data(1000, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    three_way_split_test(&prices, &params);
}
```

## Practical Recommendations

### The 10:1 Rule
Each parameter should have at least 10 trades. If a strategy has 5 parameters — you need at least 50 trades for reliable testing.

### Monte Carlo Simulation
Testing the strategy on thousands of random trade permutations:

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

fn monte_carlo_simulation(trades: &[Trade], iterations: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut results = Vec::new();

    for _ in 0..iterations {
        let mut shuffled = trades.to_vec();
        shuffled.shuffle(&mut rng);

        let total_pnl: f64 = shuffled.iter().map(|t| t.pnl).sum();
        results.push(total_pnl);
    }

    results.sort_by(|a, b| a.partial_cmp(b).unwrap());
    results
}

fn main() {
    let prices = generate_price_data(500, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    let trades = backtest_strategy(&prices, &params);
    let original_pnl: f64 = trades.iter().map(|t| t.pnl).sum();

    println!("=== Monte Carlo Simulation (1000 iterations) ===");
    println!("Original profit: {:.2}%", original_pnl * 100.0);

    let mc_results = monte_carlo_simulation(&trades, 1000);

    // 5th and 95th percentiles (90% confidence interval)
    let p5 = mc_results[50];
    let p95 = mc_results[950];

    println!("90% confidence interval: [{:.2}%, {:.2}%]", p5 * 100.0, p95 * 100.0);

    if original_pnl < p5 || original_pnl > p95 {
        println!("⚠️  Result is outside the confidence interval!");
        println!("    Trade order may have critical significance (lucky streak)");
    } else {
        println!("✅ Result within normal range");
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Overfitting** | When a strategy is over-tuned to historical data |
| **Train/Test Split** | Separating data into training and test sets |
| **Walk-Forward** | Sequential testing on rolling windows |
| **Cross-Validation** | K-fold validation to assess stability |
| **Sharpe Ratio** | Risk-return metric for strategy evaluation |
| **AIC** | Information criterion with complexity penalty |
| **Monte Carlo** | Simulation to assess result robustness |
| **10:1 Rule** | Minimum 10 trades per parameter |

## Homework

1. **Overfitting Detector**: Write a function that:
   - Takes backtest results
   - Checks for 5+ signs of overfitting
   - Outputs a score from 0 to 100 (overfitting risk)
   - Generates a report with recommendations

2. **Walk-Forward Optimizer**: Implement an optimization system:
   - Rolling window (6 months training, 1 month test)
   - Automatic parameter tuning on training window
   - Testing on the next month
   - Calculate average performance across all windows

3. **Validation Method Comparison**: Test one strategy using:
   - Simple train/test split (70/30)
   - K-fold cross-validation (k=5)
   - Walk-forward analysis
   - Monte Carlo simulation

   Compare results and stability of assessments.

4. **Regularized Optimization**: Create a parameter optimizer with:
   - Penalty for large number of parameters
   - Bonus for large number of trades
   - Penalty for long periods without trades
   - Balance between profit and robustness

## Navigation

[← Previous day](../293-grid-search-parameter-sweep/en.md) | [Next day →](../307-benchmarks-criterion/en.md)
