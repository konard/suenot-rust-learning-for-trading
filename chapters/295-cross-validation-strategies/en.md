# Day 295: Cross-Validation for Strategies

## Trading Analogy

Imagine you optimized a trading strategy on 2023 data, and it shows fantastic results: +60% returns! But when you run it on 2024 data — the strategy loses money. What went wrong?

The problem is that you only tested the strategy on one period. Perhaps it just "guessed" the peculiarities of the 2023 market, but doesn't work in other conditions.

**Cross-validation** solves this problem: instead of one train/test split, we perform several independent checks across different time periods. If a strategy consistently performs well across all periods — it's truly reliable, not just "fitted" to specific data.

It's like evaluating an employee not on one project, but on ten different tasks: if they show good results everywhere — they're truly competent.

## What is Cross-Validation?

Cross-validation is a validation method that:

| Aspect | Description |
|--------|-------------|
| **Approach** | Multiple train/test data splits |
| **Goal** | Assess model/strategy stability |
| **Application** | Machine learning, trading strategy validation |
| **Advantage** | More reliable quality assessment |
| **For time series** | Important to preserve chronological order |
| **Metric** | Average quality across all folds |

## Simple Example: K-Fold Cross-Validation for Time Series

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Self {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

#[derive(Debug)]
struct BacktestResult {
    fold_number: usize,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
}

struct TimeSeriesFold<'a, T> {
    train: &'a [T],
    test: &'a [T],
}

// K-Fold Cross-Validation for time series
fn time_series_k_fold<T>(data: &[T], k: usize) -> Vec<TimeSeriesFold<T>> {
    let mut folds = Vec::new();
    let fold_size = data.len() / (k + 1); // +1 because we need reserve for training

    for i in 0..k {
        let test_start = (i + 1) * fold_size;
        let test_end = test_start + fold_size;

        if test_end > data.len() {
            break;
        }

        folds.push(TimeSeriesFold {
            train: &data[0..test_start],
            test: &data[test_start..test_end],
        });
    }

    folds
}

fn main() {
    // Historical data over several periods
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
    ];

    let k = 3; // Number of folds
    let folds = time_series_k_fold(&candles, k);

    println!("=== Time Series K-Fold Cross-Validation (K={}) ===\n", k);

    for (i, fold) in folds.iter().enumerate() {
        println!("Fold {}:", i + 1);
        println!("  Train: {} candles (up to {})",
            fold.train.len(),
            fold.train.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!("  Test: {} candles ({} - {})",
            fold.test.len(),
            fold.test.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.test.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!();
    }
}
```

## Backtest with Cross-Validation

Now let's apply cross-validation to evaluate a trading strategy:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
struct StrategyConfig {
    ma_period: usize,
    stop_loss_pct: f64,
    take_profit_pct: f64,
}

// Backtest simulation
fn run_backtest(data: &[Candle], config: &StrategyConfig) -> (f64, f64, f64, f64) {
    // Use hash for pseudo-random but deterministic results
    let mut hasher = DefaultHasher::new();
    config.ma_period.hash(&mut hasher);
    data.len().hash(&mut hasher);
    let seed = (hasher.finish() % 10000) as f64 / 10000.0;

    let total_return = 5.0 + seed * 30.0;
    let sharpe_ratio = 0.8 + seed * 1.5;
    let max_drawdown = 5.0 + seed * 10.0;
    let win_rate = 50.0 + seed * 25.0;

    (total_return, sharpe_ratio, max_drawdown, win_rate)
}

fn cross_validate_strategy(
    data: &[Candle],
    config: &StrategyConfig,
    k: usize,
) -> Vec<BacktestResult> {
    let folds = time_series_k_fold(data, k);
    let mut results = Vec::new();

    for (i, fold) in folds.iter().enumerate() {
        let (total_return, sharpe_ratio, max_drawdown, win_rate) =
            run_backtest(fold.test, config);

        results.push(BacktestResult {
            fold_number: i + 1,
            total_return,
            sharpe_ratio,
            max_drawdown,
            win_rate,
        });
    }

    results
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    let config = StrategyConfig {
        ma_period: 10,
        stop_loss_pct: 2.0,
        take_profit_pct: 5.0,
    };

    let k = 3;
    let results = cross_validate_strategy(&candles, &config, k);

    println!("=== Cross-Validation Results ===\n");

    let mut total_return_sum = 0.0;
    let mut sharpe_sum = 0.0;
    let mut drawdown_sum = 0.0;
    let mut win_rate_sum = 0.0;

    for result in &results {
        println!("Fold {}:", result.fold_number);
        println!("  Total Return: {:.2}%", result.total_return);
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
        println!("  Max Drawdown: {:.2}%", result.max_drawdown);
        println!("  Win Rate: {:.2}%", result.win_rate);
        println!();

        total_return_sum += result.total_return;
        sharpe_sum += result.sharpe_ratio;
        drawdown_sum += result.max_drawdown;
        win_rate_sum += result.win_rate;
    }

    let n = results.len() as f64;

    println!("=== Average Metrics Across All Folds ===");
    println!("Average Total Return: {:.2}%", total_return_sum / n);
    println!("Average Sharpe Ratio: {:.2}", sharpe_sum / n);
    println!("Average Max Drawdown: {:.2}%", drawdown_sum / n);
    println!("Average Win Rate: {:.2}%", win_rate_sum / n);
}
```

## Walk-Forward Cross-Validation

A more advanced method for time series — walk-forward validation:

```rust
struct WalkForwardFold<'a, T> {
    train: &'a [T],
    test: &'a [T],
}

fn walk_forward_split<T>(
    data: &[T],
    initial_train_size: usize,
    test_size: usize,
    step_size: usize,
) -> Vec<WalkForwardFold<T>> {
    let mut folds = Vec::new();
    let mut train_end = initial_train_size;

    while train_end + test_size <= data.len() {
        let test_end = train_end + test_size;

        folds.push(WalkForwardFold {
            train: &data[0..train_end],
            test: &data[train_end..test_end],
        });

        train_end += step_size;
    }

    folds
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    // Initial train size: 6 months
    // Test size: 2 months
    // Step: 1 month
    let folds = walk_forward_split(&candles, 6, 2, 1);

    println!("=== Walk-Forward Cross-Validation ===\n");

    for (i, fold) in folds.iter().enumerate() {
        println!("Period {}:", i + 1);
        println!("  Train: {} months ({} - {})",
            fold.train.len(),
            fold.train.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.train.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!("  Test: {} months ({} - {})",
            fold.test.len(),
            fold.test.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.test.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!();
    }
}
```

## Advanced Example: Cross-Validation with Parameter Optimization

```rust
#[derive(Debug, Clone)]
struct CrossValidationResult {
    config: StrategyConfig,
    fold_results: Vec<BacktestResult>,
    avg_sharpe: f64,
    std_sharpe: f64,
    avg_return: f64,
    stability_score: f64,
}

fn calculate_std_dev(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn evaluate_config_with_cv(
    data: &[Candle],
    config: &StrategyConfig,
    k: usize,
) -> CrossValidationResult {
    let fold_results = cross_validate_strategy(data, config, k);

    let sharpe_values: Vec<f64> = fold_results.iter()
        .map(|r| r.sharpe_ratio)
        .collect();

    let return_values: Vec<f64> = fold_results.iter()
        .map(|r| r.total_return)
        .collect();

    let avg_sharpe = sharpe_values.iter().sum::<f64>() / sharpe_values.len() as f64;
    let avg_return = return_values.iter().sum::<f64>() / return_values.len() as f64;
    let std_sharpe = calculate_std_dev(&sharpe_values, avg_sharpe);

    // Stability: high average Sharpe and low variation
    let stability_score = avg_sharpe / (1.0 + std_sharpe);

    CrossValidationResult {
        config: config.clone(),
        fold_results,
        avg_sharpe,
        std_sharpe,
        avg_return,
        stability_score,
    }
}

fn grid_search_with_cv(data: &[Candle], k: usize) -> Vec<CrossValidationResult> {
    let mut results = Vec::new();

    let ma_periods = vec![5, 10, 20];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 8.0];

    for &ma_period in &ma_periods {
        for &stop_loss in &stop_losses {
            for &take_profit in &take_profits {
                if take_profit <= stop_loss {
                    continue;
                }

                let config = StrategyConfig {
                    ma_period,
                    stop_loss_pct: stop_loss,
                    take_profit_pct: take_profit,
                };

                let cv_result = evaluate_config_with_cv(data, &config, k);
                results.push(cv_result);
            }
        }
    }

    results
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    println!("Running Grid Search with Cross-Validation...\n");

    let k = 3;
    let cv_results = grid_search_with_cv(&candles, k);

    // Sort by stability score
    let mut sorted_results = cv_results;
    sorted_results.sort_by(|a, b|
        b.stability_score.partial_cmp(&a.stability_score).unwrap()
    );

    println!("=== Top-5 Configurations by Stability ===\n");

    for (i, result) in sorted_results.iter().take(5).enumerate() {
        println!("{}. MA={}, SL={:.1}%, TP={:.1}%",
            i + 1,
            result.config.ma_period,
            result.config.stop_loss_pct,
            result.config.take_profit_pct);
        println!("   Average Sharpe: {:.2} (±{:.2})",
            result.avg_sharpe, result.std_sharpe);
        println!("   Average Return: {:.2}%", result.avg_return);
        println!("   Stability: {:.2}", result.stability_score);
        println!();
    }
}
```

## Parallel Cross-Validation with Rayon

```rust
use rayon::prelude::*;

fn parallel_grid_search_with_cv(data: &[Candle], k: usize) -> Vec<CrossValidationResult> {
    let ma_periods = vec![5, 10, 20];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 8.0];

    // Generate all configurations
    let mut configs = Vec::new();
    for &ma_period in &ma_periods {
        for &stop_loss in &stop_losses {
            for &take_profit in &take_profits {
                if take_profit > stop_loss {
                    configs.push(StrategyConfig {
                        ma_period,
                        stop_loss_pct: stop_loss,
                        take_profit_pct: take_profit,
                    });
                }
            }
        }
    }

    println!("Parallel testing of {} configurations...\n", configs.len());

    // Parallel evaluation with Rayon
    configs.par_iter()
        .map(|config| evaluate_config_with_cv(data, config, k))
        .collect()
}

fn main() {
    use std::time::Instant;

    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    let start = Instant::now();
    let k = 3;
    let results = parallel_grid_search_with_cv(&candles, k);
    let duration = start.elapsed();

    println!("Execution time: {:?}", duration);
    println!("Tested {} configurations", results.len());

    // Find best by stability
    if let Some(best) = results.iter()
        .max_by(|a, b| a.stability_score.partial_cmp(&b.stability_score).unwrap())
    {
        println!("\nBest configuration:");
        println!("  MA period: {}", best.config.ma_period);
        println!("  Stop-loss: {:.1}%", best.config.stop_loss_pct);
        println!("  Take-profit: {:.1}%", best.config.take_profit_pct);
        println!("  Stability Score: {:.2}", best.stability_score);
        println!("  Average Sharpe: {:.2} (±{:.2})",
            best.avg_sharpe, best.std_sharpe);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Cross-Validation | Multiple checks across different periods |
| K-Fold CV | Splitting into K folds for validation |
| Walk-Forward CV | Sequential expansion of training set |
| Stability Score | Metric for result stability |
| Grid Search + CV | Combination of parameter sweep and validation |
| Parallelization | Acceleration using Rayon |

## Practical Exercises

1. **Basic K-Fold**: Implement 5-fold cross-validation for a simple RSI-based strategy. Output results for each fold.

2. **Walk-Forward**: Create walk-forward validation with expanding training window (anchored walk-forward), where training data always starts from the beginning of history.

3. **Stability Analysis**: Add a function that calculates not only mean and standard deviation of Sharpe Ratio, but also coefficient of variation (CV = std/mean).

4. **Visualization**: Save results of each fold to a CSV file with fields: fold_number, config, sharpe_ratio, total_return, max_drawdown.

## Homework

1. **Time Series Split**: Implement an alternative TimeSeriesSplit method that:
   - Creates folds with fixed training set size (sliding window)
   - Supports gap between train and test to prevent data leakage

2. **Nested Cross-Validation**: Implement nested cross-validation:
   - Outer loop: model quality assessment
   - Inner loop: hyperparameter tuning
   - This avoids overfitting when selecting parameters

3. **Purged K-Fold**: For strategies holding positions, implement purged K-fold:
   - Exclude from training set data that "overlaps" with the test period
   - This is important for strategies where positions are held for several days

4. **Monte Carlo CV**: Instead of fixed folds, use random splitting:
   - Perform N iterations (e.g., 100)
   - Each time randomly select a continuous period for testing
   - Average the results

## Navigation

[← Previous day](../293-grid-search-parameter-sweep/en.md) | [Next day →](../296-monte-carlo-simulations/en.md)
