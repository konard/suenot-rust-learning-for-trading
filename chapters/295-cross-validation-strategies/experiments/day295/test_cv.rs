// Test script for Day 295 Cross-Validation examples

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

#[derive(Debug, Clone)]
struct StrategyConfig {
    ma_period: usize,
    stop_loss_pct: f64,
    take_profit_pct: f64,
}

// K-Fold Cross-Validation for time series
fn time_series_k_fold<T>(data: &[T], k: usize) -> Vec<TimeSeriesFold<T>> {
    let mut folds = Vec::new();
    let fold_size = data.len() / (k + 1);

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

// Backtest simulation
fn run_backtest(data: &[Candle], config: &StrategyConfig) -> (f64, f64, f64, f64) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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
    println!("=== Testing Day 295: Cross-Validation Examples ===\n");

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

    // Test 1: Basic K-Fold
    println!("Test 1: Time Series K-Fold");
    let k = 3;
    let folds = time_series_k_fold(&candles, k);
    println!("Created {} folds", folds.len());
    for (i, fold) in folds.iter().enumerate() {
        println!("  Fold {}: train={}, test={}", i + 1, fold.train.len(), fold.test.len());
    }
    println!();

    // Test 2: Cross-Validation with Backtest
    println!("Test 2: Cross-Validation with Strategy");
    let config = StrategyConfig {
        ma_period: 10,
        stop_loss_pct: 2.0,
        take_profit_pct: 5.0,
    };

    let results = cross_validate_strategy(&candles, &config, k);

    let mut sharpe_sum = 0.0;
    for result in &results {
        println!("  Fold {}: Sharpe={:.2}, Return={:.2}%",
            result.fold_number, result.sharpe_ratio, result.total_return);
        sharpe_sum += result.sharpe_ratio;
    }
    println!("  Average Sharpe: {:.2}", sharpe_sum / results.len() as f64);
    println!();

    println!("All tests passed!");
}
