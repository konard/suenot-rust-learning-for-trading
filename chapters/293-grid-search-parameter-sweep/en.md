# Day 293: Grid Search: Parameter Sweep

## Trading Analogy

Imagine you've developed a trading strategy based on Moving Averages. Now you need to find the optimal parameters:
- Short MA window: 5, 10, 15, or 20 periods?
- Long MA window: 50, 100, 150, or 200 periods?
- Stop-loss: 1%, 2%, 3%, or 5%?
- Take-profit: 2%, 4%, 6%, or 10%?

Instead of testing each combination manually, you need **grid search** — a systematic method of testing all possible parameter combinations to find the best configuration.

It's like testing all radio settings to find the clearest signal: you methodically check every combination of frequency and volume to find the perfect parameters.

## What is Grid Search?

Grid search is a parameter optimization method that:

| Aspect | Description |
|--------|-------------|
| **Approach** | Exhaustive search of all combinations |
| **Parameters** | Discrete values from a predefined set |
| **Application** | Machine learning, strategy backtesting |
| **Complexity** | O(n^m), where n = values per parameter, m = number of parameters |
| **Advantage** | Guaranteed to find best combination in the search space |
| **Disadvantage** | Can be very slow with many parameters |

## Simple Example: Parameter Sweep

```rust
// Define search space
struct GridSearchSpace {
    short_ma_periods: Vec<usize>,
    long_ma_periods: Vec<usize>,
    stop_loss_pct: Vec<f64>,
    take_profit_pct: Vec<f64>,
}

// Strategy configuration
#[derive(Debug, Clone)]
struct StrategyConfig {
    short_ma: usize,
    long_ma: usize,
    stop_loss: f64,
    take_profit: f64,
}

// Backtest result
#[derive(Debug)]
struct BacktestResult {
    config: StrategyConfig,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn main() {
    // Define parameter space
    let search_space = GridSearchSpace {
        short_ma_periods: vec![5, 10, 15, 20],
        long_ma_periods: vec![50, 100, 150, 200],
        stop_loss_pct: vec![1.0, 2.0, 3.0, 5.0],
        take_profit_pct: vec![2.0, 4.0, 6.0, 10.0],
    };

    let mut all_results = Vec::new();
    let mut total_combinations = 0;

    // Grid search: iterate all combinations
    for &short_ma in &search_space.short_ma_periods {
        for &long_ma in &search_space.long_ma_periods {
            // Skip invalid combinations
            if short_ma >= long_ma {
                continue;
            }

            for &stop_loss in &search_space.stop_loss_pct {
                for &take_profit in &search_space.take_profit_pct {
                    total_combinations += 1;

                    let config = StrategyConfig {
                        short_ma,
                        long_ma,
                        stop_loss,
                        take_profit,
                    };

                    // Simulate backtest
                    let result = run_backtest(&config);
                    all_results.push(result);
                }
            }
        }
    }

    println!("Tested {} parameter combinations", total_combinations);

    // Find best configuration by Sharpe Ratio
    if let Some(best) = all_results.iter().max_by(|a, b|
        a.sharpe_ratio.partial_cmp(&b.sharpe_ratio).unwrap()
    ) {
        println!("\nBest configuration:");
        println!("  Short MA: {}", best.config.short_ma);
        println!("  Long MA: {}", best.config.long_ma);
        println!("  Stop-loss: {:.1}%", best.config.stop_loss);
        println!("  Take-profit: {:.1}%", best.config.take_profit);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
        println!("  Total Return: {:.2}%", best.total_return);
        println!("  Max Drawdown: {:.2}%", best.max_drawdown);
        println!("  Win Rate: {:.2}%", best.win_rate);
    }
}

// Backtest simulation (stub)
fn run_backtest(config: &StrategyConfig) -> BacktestResult {
    // In reality, this would be a full backtest
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    config.short_ma.hash(&mut hasher);
    config.long_ma.hash(&mut hasher);
    let seed = hasher.finish();

    // Pseudo-random metrics for demonstration
    let random = (seed % 10000) as f64 / 10000.0;

    BacktestResult {
        config: config.clone(),
        total_return: 5.0 + random * 30.0,
        sharpe_ratio: 0.5 + random * 2.0,
        max_drawdown: 5.0 + random * 15.0,
        win_rate: 45.0 + random * 30.0,
    }
}
```

## Iterator for Grid Search

Best practice — create an iterator for lazy parameter generation:

```rust
struct GridSearchIter {
    short_ma_values: Vec<usize>,
    long_ma_values: Vec<usize>,
    stop_loss_values: Vec<f64>,
    take_profit_values: Vec<f64>,
    current_indices: [usize; 4],
    finished: bool,
}

impl GridSearchIter {
    fn new(
        short_ma: Vec<usize>,
        long_ma: Vec<usize>,
        stop_loss: Vec<f64>,
        take_profit: Vec<f64>,
    ) -> Self {
        Self {
            short_ma_values: short_ma,
            long_ma_values: long_ma,
            stop_loss_values: stop_loss,
            take_profit_values: take_profit,
            current_indices: [0, 0, 0, 0],
            finished: false,
        }
    }

    fn increment_indices(&mut self) -> bool {
        // Increment indices like a multi-dimensional counter
        let mut carry = true;

        // Take profit (last index)
        if carry {
            self.current_indices[3] += 1;
            if self.current_indices[3] >= self.take_profit_values.len() {
                self.current_indices[3] = 0;
            } else {
                carry = false;
            }
        }

        // Stop loss
        if carry {
            self.current_indices[2] += 1;
            if self.current_indices[2] >= self.stop_loss_values.len() {
                self.current_indices[2] = 0;
            } else {
                carry = false;
            }
        }

        // Long MA
        if carry {
            self.current_indices[1] += 1;
            if self.current_indices[1] >= self.long_ma_values.len() {
                self.current_indices[1] = 0;
            } else {
                carry = false;
            }
        }

        // Short MA
        if carry {
            self.current_indices[0] += 1;
            if self.current_indices[0] >= self.short_ma_values.len() {
                return false; // Reached the end
            }
        }

        true
    }
}

impl Iterator for GridSearchIter {
    type Item = StrategyConfig;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            let config = StrategyConfig {
                short_ma: self.short_ma_values[self.current_indices[0]],
                long_ma: self.long_ma_values[self.current_indices[1]],
                stop_loss: self.stop_loss_values[self.current_indices[2]],
                take_profit: self.take_profit_values[self.current_indices[3]],
            };

            // Prepare next iteration
            if !self.increment_indices() {
                self.finished = true;
            }

            // Skip invalid combinations
            if config.short_ma < config.long_ma {
                return Some(config);
            }

            if self.finished {
                return None;
            }
        }
    }
}

fn main() {
    let grid = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    );

    let mut count = 0;
    let mut best_result: Option<BacktestResult> = None;

    for config in grid {
        count += 1;
        let result = run_backtest(&config);

        // Update best result
        best_result = match best_result {
            None => Some(result),
            Some(current_best) => {
                if result.sharpe_ratio > current_best.sharpe_ratio {
                    Some(result)
                } else {
                    Some(current_best)
                }
            }
        };

        if count % 10 == 0 {
            println!("Tested {} configurations...", count);
        }
    }

    println!("\nTotal tested: {} configurations", count);

    if let Some(best) = best_result {
        println!("\nBest result:");
        println!("  MA: {}/{}", best.config.short_ma, best.config.long_ma);
        println!("  Stop/Take: {:.1}%/{:.1}%",
            best.config.stop_loss, best.config.take_profit);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
    }
}
```

## Parallel Grid Search

To speed up the process, use multithreading with Rayon:

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

fn parallel_grid_search() {
    let grid_configs: Vec<StrategyConfig> = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    ).collect();

    println!("Running parallel testing of {} configurations...",
        grid_configs.len());

    let best_result = Arc::new(Mutex::new(None::<BacktestResult>));

    // Parallel sweep with Rayon
    grid_configs.par_iter().for_each(|config| {
        let result = run_backtest(config);

        // Update best result thread-safely
        let mut best = best_result.lock().unwrap();
        match &*best {
            None => *best = Some(result),
            Some(current_best) => {
                if result.sharpe_ratio > current_best.sharpe_ratio {
                    *best = Some(result);
                }
            }
        }
    });

    let best = best_result.lock().unwrap();
    if let Some(ref result) = *best {
        println!("\nBest configuration (parallel search):");
        println!("  MA: {}/{}", result.config.short_ma, result.config.long_ma);
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
    }
}

fn main() {
    use std::time::Instant;

    let start = Instant::now();
    parallel_grid_search();
    let duration = start.elapsed();

    println!("\nExecution time: {:?}", duration);
}
```

## Advanced Example: Multi-Level Grid Search

```rust
#[derive(Debug, Clone)]
struct AdvancedStrategyConfig {
    // Indicators
    ma_short: usize,
    ma_long: usize,
    rsi_period: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,

    // Risk management
    stop_loss_pct: f64,
    take_profit_pct: f64,
    position_size_pct: f64,

    // Filters
    min_volume: f64,
    max_spread_pct: f64,
}

#[derive(Debug)]
struct AdvancedBacktestResult {
    config: AdvancedStrategyConfig,
    total_return: f64,
    sharpe_ratio: f64,
    sortino_ratio: f64,
    max_drawdown: f64,
    calmar_ratio: f64,
    win_rate: f64,
    profit_factor: f64,
    total_trades: usize,
}

struct AdvancedGridSearch {
    configs: Vec<AdvancedStrategyConfig>,
}

impl AdvancedGridSearch {
    fn new() -> Self {
        let mut configs = Vec::new();

        // Indicator parameters
        let ma_short_values = vec![5, 10, 20];
        let ma_long_values = vec![50, 100, 200];
        let rsi_periods = vec![14, 21];
        let rsi_oversold_values = vec![20.0, 30.0];
        let rsi_overbought_values = vec![70.0, 80.0];

        // Risk parameters
        let stop_loss_values = vec![1.0, 2.0, 3.0];
        let take_profit_values = vec![3.0, 6.0, 9.0];
        let position_sizes = vec![25.0, 50.0, 100.0];

        // Filters
        let min_volumes = vec![100000.0, 500000.0];
        let max_spreads = vec![0.1, 0.2];

        // Generate all combinations
        for &ma_short in &ma_short_values {
            for &ma_long in &ma_long_values {
                if ma_short >= ma_long { continue; }

                for &rsi_period in &rsi_periods {
                    for &rsi_oversold in &rsi_oversold_values {
                        for &rsi_overbought in &rsi_overbought_values {
                            if rsi_oversold >= rsi_overbought { continue; }

                            for &stop_loss in &stop_loss_values {
                                for &take_profit in &take_profit_values {
                                    if take_profit <= stop_loss { continue; }

                                    for &position_size in &position_sizes {
                                        for &min_volume in &min_volumes {
                                            for &max_spread in &max_spreads {
                                                configs.push(AdvancedStrategyConfig {
                                                    ma_short,
                                                    ma_long,
                                                    rsi_period,
                                                    rsi_oversold,
                                                    rsi_overbought,
                                                    stop_loss_pct: stop_loss,
                                                    take_profit_pct: take_profit,
                                                    position_size_pct: position_size,
                                                    min_volume,
                                                    max_spread_pct: max_spread,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { configs }
    }

    fn run_parallel(&self) -> Vec<AdvancedBacktestResult> {
        use rayon::prelude::*;

        println!("Testing {} configurations...", self.configs.len());

        self.configs.par_iter()
            .map(|config| run_advanced_backtest(config))
            .collect()
    }

    fn find_best_by_metric<F>(&self, results: &[AdvancedBacktestResult],
                               metric_fn: F) -> Option<&AdvancedBacktestResult>
    where
        F: Fn(&AdvancedBacktestResult) -> f64,
    {
        results.iter()
            .max_by(|a, b| metric_fn(a).partial_cmp(&metric_fn(b)).unwrap())
    }
}

fn run_advanced_backtest(config: &AdvancedStrategyConfig) -> AdvancedBacktestResult {
    // Backtest simulation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    config.ma_short.hash(&mut hasher);
    config.ma_long.hash(&mut hasher);
    config.rsi_period.hash(&mut hasher);
    let seed = (hasher.finish() % 10000) as f64 / 10000.0;

    AdvancedBacktestResult {
        config: config.clone(),
        total_return: 10.0 + seed * 50.0,
        sharpe_ratio: 0.5 + seed * 2.5,
        sortino_ratio: 0.6 + seed * 2.8,
        max_drawdown: 8.0 + seed * 20.0,
        calmar_ratio: 0.3 + seed * 1.5,
        win_rate: 40.0 + seed * 35.0,
        profit_factor: 1.0 + seed * 2.0,
        total_trades: 50 + (seed * 200.0) as usize,
    }
}

fn main() {
    let grid_search = AdvancedGridSearch::new();
    let results = grid_search.run_parallel();

    // Find best by different metrics
    println!("\n=== Grid Search Results ===\n");

    if let Some(best_sharpe) = grid_search.find_best_by_metric(&results,
        |r| r.sharpe_ratio) {
        println!("Best Sharpe Ratio: {:.2}", best_sharpe.sharpe_ratio);
        println!("  MA: {}/{}", best_sharpe.config.ma_short,
            best_sharpe.config.ma_long);
        println!("  RSI: {} ({:.0}/{:.0})", best_sharpe.config.rsi_period,
            best_sharpe.config.rsi_oversold, best_sharpe.config.rsi_overbought);
    }

    if let Some(best_return) = grid_search.find_best_by_metric(&results,
        |r| r.total_return) {
        println!("\nBest Total Return: {:.2}%", best_return.total_return);
        println!("  Position Size: {:.0}%", best_return.config.position_size_pct);
        println!("  Stop/Take: {:.1}%/{:.1}%",
            best_return.config.stop_loss_pct, best_return.config.take_profit_pct);
    }

    if let Some(best_calmar) = grid_search.find_best_by_metric(&results,
        |r| r.calmar_ratio) {
        println!("\nBest Calmar Ratio: {:.2}", best_calmar.calmar_ratio);
        println!("  Max Drawdown: {:.2}%", best_calmar.max_drawdown);
        println!("  Win Rate: {:.2}%", best_calmar.win_rate);
    }

    // Statistics
    let avg_sharpe: f64 = results.iter()
        .map(|r| r.sharpe_ratio)
        .sum::<f64>() / results.len() as f64;

    println!("\n=== Overall Statistics ===");
    println!("Configurations tested: {}", results.len());
    println!("Average Sharpe Ratio: {:.2}", avg_sharpe);
}
```

## Optimization: Early Stopping

```rust
fn grid_search_with_early_stopping(
    min_sharpe_threshold: f64,
    max_iterations: usize,
) -> Option<BacktestResult> {
    let grid = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    );

    let mut best_result: Option<BacktestResult> = None;
    let mut iterations = 0;

    for config in grid {
        iterations += 1;

        let result = run_backtest(&config);

        // Update best result
        let is_new_best = match &best_result {
            None => true,
            Some(current_best) => result.sharpe_ratio > current_best.sharpe_ratio,
        };

        if is_new_best {
            println!("New best result at iteration {}: Sharpe={:.2}",
                iterations, result.sharpe_ratio);

            // Check threshold
            if result.sharpe_ratio >= min_sharpe_threshold {
                println!("Reached threshold {:.2}! Stopping search.",
                    min_sharpe_threshold);
                return Some(result);
            }

            best_result = Some(result);
        }

        // Check iteration limit
        if iterations >= max_iterations {
            println!("Reached iteration limit {}.", max_iterations);
            break;
        }
    }

    best_result
}

fn main() {
    println!("Grid search with early stopping:\n");

    if let Some(result) = grid_search_with_early_stopping(1.8, 100) {
        println!("\nFound configuration:");
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
        println!("  MA: {}/{}", result.config.short_ma, result.config.long_ma);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Grid Search | Exhaustive search of all parameter combinations |
| Parameter Iterator | Lazy generation of configurations |
| Parallel Sweep | Using Rayon for acceleration |
| Multi-Dimensional Search | Optimization across multiple parameters simultaneously |
| Early Stopping | Halting when goal is reached |
| Quality Metrics | Sharpe Ratio, Sortino, Calmar, Win Rate |

## Practical Exercises

1. **Basic Grid Search**: Implement a sweep for a Mean Reversion strategy with parameters:
   - Z-score threshold: 1.5, 2.0, 2.5, 3.0
   - Lookback period: 20, 50, 100
   - Position size: 10%, 25%, 50%

2. **Custom Iterator**: Create a `GridSearchIter` that skips combinations where `take_profit < 2 * stop_loss`.

3. **Parallel Backtest**: Use Rayon to parallel test 1000+ configurations of an RSI-based strategy.

4. **Result Validation**: Split data into train/test (70%/30%) and verify that best parameters on train data show good results on test data.

## Homework

1. **Multi-Metric Optimization**: Create a grid search system that finds configurations optimal across multiple metrics simultaneously (Sharpe > 1.5 AND Max Drawdown < 15% AND Win Rate > 55%).

2. **Adaptive Grid Search**: Implement a two-stage search:
   - Stage 1: Coarse search with large parameter steps
   - Stage 2: Refined search around best results with small steps

3. **Cross-Validation Grid Search**: For each configuration, perform K-fold cross-validation (k=5) and average results for more reliable evaluation.

4. **Logging and Visualization**: Save all results to a CSV file and create a script to plot a heatmap of Sharpe Ratio dependency on a pair of parameters (e.g., MA short vs MA long).

## Navigation

[← Previous day](../170-crossbeam-advanced-concurrency/en.md) | [Next day →](../294-genetic-algorithms-optimization/en.md)
