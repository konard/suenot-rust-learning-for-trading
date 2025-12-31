# Day 291: Out-of-Sample Testing

## Trading Analogy

Imagine you developed a trading strategy based on Bitcoin data from January to June 2024. It shows excellent results: +45% for the period! You launch the bot with real money and... lose 20% in a month. What happened?

The problem is called **overfitting**. Your strategy perfectly adapted to historical data but doesn't work on new data. It's like a student who memorized answers to specific questions but didn't understand the material.

**Out-of-sample testing** solves this problem:
- **In-sample** (training set): January-June 2024 — develop and optimize the strategy
- **Out-of-sample** (test set): July-September 2024 — verify if the strategy works on data it "hasn't seen"

If the strategy performs well on both sets — it's a sign of reliability. If only on in-sample — likely overfitting.

## Basic Data Splitting

Let's start with simple data splitting into training and test sets:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct DataSplit<T> {
    in_sample: Vec<T>,
    out_of_sample: Vec<T>,
}

fn split_data<T: Clone>(data: &[T], split_ratio: f64) -> DataSplit<T> {
    let split_point = (data.len() as f64 * split_ratio) as usize;

    DataSplit {
        in_sample: data[..split_point].to_vec(),
        out_of_sample: data[split_point..].to_vec(),
    }
}

fn main() {
    // Bitcoin historical data
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43800.0),
        Candle::new("2024-01-08", 44500.0),
        Candle::new("2024-01-09", 45000.0),
        Candle::new("2024-01-10", 44800.0),
    ];

    // Split 70% for training, 30% for testing
    let split = split_data(&candles, 0.7);

    println!("=== Data Split ===");
    println!("Total candles: {}", candles.len());
    println!("In-sample (training): {}", split.in_sample.len());
    println!("Out-of-sample (testing): {}", split.out_of_sample.len());

    println!("\n=== In-sample data ===");
    for candle in &split.in_sample {
        println!("{}: ${:.2}", candle.timestamp, candle.close);
    }

    println!("\n=== Out-of-sample data ===");
    for candle in &split.out_of_sample {
        println!("{}: ${:.2}", candle.timestamp, candle.close);
    }
}
```

## Simple Trading Strategy

Let's create a simple moving average (SMA) based strategy:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn calculate_sma(&self, candles: &[Candle]) -> Vec<f64> {
        let mut sma_values = Vec::new();

        if candles.len() < self.period {
            return sma_values;
        }

        for i in (self.period - 1)..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            sma_values.push(sum / self.period as f64);
        }

        sma_values
    }

    // Signal: BUY if price above SMA, SELL if below
    fn generate_signals(&self, candles: &[Candle]) -> Vec<String> {
        let sma_values = self.calculate_sma(candles);
        let mut signals = Vec::new();

        // Skip first period-1 candles
        for _ in 0..(self.period - 1) {
            signals.push("HOLD".to_string());
        }

        for (i, sma) in sma_values.iter().enumerate() {
            let candle_idx = i + self.period - 1;
            if candles[candle_idx].close > *sma {
                signals.push("BUY".to_string());
            } else {
                signals.push("SELL".to_string());
            }
        }

        signals
    }
}

fn main() {
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
    ];

    let strategy = SMAStrategy::new(3);
    let signals = strategy.generate_signals(&candles);

    println!("=== Trading Signals (SMA-3) ===");
    for (i, candle) in candles.iter().enumerate() {
        println!("{}: ${:.2} -> {}", candle.timestamp, candle.close, signals[i]);
    }
}
```

## Backtesting the Strategy

Now let's implement a backtesting system:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct BacktestResult {
    total_trades: usize,
    profitable_trades: usize,
    total_return: f64,
    max_drawdown: f64,
}

impl BacktestResult {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.profitable_trades as f64 / self.total_trades as f64) * 100.0
    }

    fn print_summary(&self, label: &str) {
        println!("\n=== {} ===", label);
        println!("Total trades: {}", self.total_trades);
        println!("Profitable: {}", self.profitable_trades);
        println!("Win rate: {:.2}%", self.win_rate());
        println!("Total return: {:.2}%", self.total_return);
        println!("Max drawdown: {:.2}%", self.max_drawdown);
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None; // Entry price
        let mut total_trades = 0;
        let mut profitable_trades = 0;
        let mut total_return = 0.0;
        let mut equity = 100.0; // Initial capital 100%
        let mut peak_equity = 100.0;
        let mut max_drawdown = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                profitable_trades: 0,
                total_return: 0.0,
                max_drawdown: 0.0,
            };
        }

        for i in self.period..candles.len() {
            // Calculate SMA
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;

            let price = candles[i].close;

            // Trading logic
            if position.is_none() && price > sma {
                // Open BUY position
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    // Close position
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    equity += profit;

                    total_trades += 1;
                    if profit > 0.0 {
                        profitable_trades += 1;
                    }

                    // Update max drawdown
                    if equity > peak_equity {
                        peak_equity = equity;
                    }
                    let drawdown = ((peak_equity - equity) / peak_equity) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }

                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            profitable_trades,
            total_return,
            max_drawdown,
        }
    }
}

fn main() {
    // Generate test data
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
    ];

    let strategy = SMAStrategy::new(3);
    let result = strategy.backtest(&candles);

    result.print_summary("Backtest Results");
}
```

## In-sample vs Out-of-sample

Complete example with data splitting and result comparison:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct DataSplit<T> {
    in_sample: Vec<T>,
    out_of_sample: Vec<T>,
}

fn split_data<T: Clone>(data: &[T], split_ratio: f64) -> DataSplit<T> {
    let split_point = (data.len() as f64 * split_ratio) as usize;

    DataSplit {
        in_sample: data[..split_point].to_vec(),
        out_of_sample: data[split_point..].to_vec(),
    }
}

#[derive(Debug)]
struct BacktestResult {
    total_trades: usize,
    profitable_trades: usize,
    total_return: f64,
    max_drawdown: f64,
}

impl BacktestResult {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.profitable_trades as f64 / self.total_trades as f64) * 100.0
    }

    fn print_summary(&self, label: &str) {
        println!("\n=== {} ===", label);
        println!("Total trades: {}", self.total_trades);
        println!("Profitable: {}", self.profitable_trades);
        println!("Win rate: {:.2}%", self.win_rate());
        println!("Total return: {:.2}%", self.total_return);
        println!("Max drawdown: {:.2}%", self.max_drawdown);
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None;
        let mut total_trades = 0;
        let mut profitable_trades = 0;
        let mut total_return = 0.0;
        let mut equity = 100.0;
        let mut peak_equity = 100.0;
        let mut max_drawdown = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                profitable_trades: 0,
                total_return: 0.0,
                max_drawdown: 0.0,
            };
        }

        for i in self.period..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;
            let price = candles[i].close;

            if position.is_none() && price > sma {
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    equity += profit;

                    total_trades += 1;
                    if profit > 0.0 {
                        profitable_trades += 1;
                    }

                    if equity > peak_equity {
                        peak_equity = equity;
                    }
                    let drawdown = ((peak_equity - equity) / peak_equity) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }

                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            profitable_trades,
            total_return,
            max_drawdown,
        }
    }
}

fn main() {
    // Large dataset
    let all_candles = vec![
        // In-sample data
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
        Candle::new("2024-01-11", 43500.0),
        Candle::new("2024-01-12", 44000.0),
        Candle::new("2024-01-13", 44500.0),
        Candle::new("2024-01-14", 44200.0),
        // Out-of-sample data
        Candle::new("2024-01-15", 44800.0),
        Candle::new("2024-01-16", 45000.0),
        Candle::new("2024-01-17", 44500.0),
        Candle::new("2024-01-18", 44000.0),
        Candle::new("2024-01-19", 44500.0),
        Candle::new("2024-01-20", 45000.0),
    ];

    // Split data 70/30
    let split = split_data(&all_candles, 0.7);

    println!("=== Data Split ===");
    println!("Total: {} candles", all_candles.len());
    println!("In-sample: {} candles", split.in_sample.len());
    println!("Out-of-sample: {} candles", split.out_of_sample.len());

    // Test strategy
    let strategy = SMAStrategy::new(3);

    let in_sample_result = strategy.backtest(&split.in_sample);
    in_sample_result.print_summary("In-sample Results");

    let out_of_sample_result = strategy.backtest(&split.out_of_sample);
    out_of_sample_result.print_summary("Out-of-sample Results");

    // Overfitting analysis
    println!("\n=== Overfitting Analysis ===");
    let return_diff = (in_sample_result.total_return - out_of_sample_result.total_return).abs();
    let win_rate_diff = (in_sample_result.win_rate() - out_of_sample_result.win_rate()).abs();

    println!("Return difference: {:.2}%", return_diff);
    println!("Win rate difference: {:.2}%", win_rate_diff);

    if return_diff < 10.0 && win_rate_diff < 15.0 {
        println!("✅ Strategy is stable (small divergence)");
    } else {
        println!("⚠️  Possible overfitting (large divergence)");
    }
}
```

## Walk-Forward Analysis

A more advanced method — walk-forward testing:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

#[derive(Debug)]
struct BacktestResult {
    total_return: f64,
    total_trades: usize,
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None;
        let mut total_trades = 0;
        let mut total_return = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                total_return: 0.0,
            };
        }

        for i in self.period..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;
            let price = candles[i].close;

            if position.is_none() && price > sma {
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    total_trades += 1;
                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            total_return,
        }
    }
}

fn walk_forward_test(candles: &[Candle], train_size: usize, test_size: usize) {
    let mut current_pos = 0;
    let mut iteration = 1;

    println!("=== Walk-Forward Testing ===\n");

    while current_pos + train_size + test_size <= candles.len() {
        let train_end = current_pos + train_size;
        let test_end = train_end + test_size;

        let train_data = &candles[current_pos..train_end];
        let test_data = &candles[train_end..test_end];

        println!("--- Iteration {} ---", iteration);
        println!("Training: {} - {}",
            train_data.first().unwrap().timestamp,
            train_data.last().unwrap().timestamp);
        println!("Testing: {} - {}",
            test_data.first().unwrap().timestamp,
            test_data.last().unwrap().timestamp);

        // Test different periods on training set
        let mut best_period = 3;
        let mut best_return = f64::MIN;

        for period in 3..=7 {
            let strategy = SMAStrategy::new(period);
            let result = strategy.backtest(train_data);

            if result.total_return > best_return {
                best_return = result.total_return;
                best_period = period;
            }
        }

        println!("Best period on training: {} (return: {:.2}%)",
            best_period, best_return);

        // Test best parameter on out-of-sample
        let strategy = SMAStrategy::new(best_period);
        let test_result = strategy.backtest(test_data);

        println!("Result on test data: {:.2}%", test_result.total_return);
        println!();

        current_pos += test_size;
        iteration += 1;
    }
}

fn main() {
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
        Candle::new("2024-01-11", 43500.0),
        Candle::new("2024-01-12", 44000.0),
        Candle::new("2024-01-13", 44500.0),
        Candle::new("2024-01-14", 44200.0),
        Candle::new("2024-01-15", 44800.0),
        Candle::new("2024-01-16", 45000.0),
        Candle::new("2024-01-17", 44500.0),
        Candle::new("2024-01-18", 44000.0),
        Candle::new("2024-01-19", 44500.0),
        Candle::new("2024-01-20", 45000.0),
    ];

    // Walk-forward: 10 candles training, 5 candles testing
    walk_forward_test(&candles, 10, 5);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `In-sample` | Data for training and optimizing the strategy |
| `Out-of-sample` | Data for verifying strategy reliability |
| `Overfitting` | Strategy works only on historical data |
| `Walk-forward` | Sequential testing with sliding window |
| `Data split` | Splitting data into training and test sets |

## Practical Exercises

1. **Data Splitting**: Write a function that splits data into three parts: train (60%), validation (20%), test (20%).

2. **Strategy Comparison**: Create two strategies (SMA and EMA) and compare their results on in-sample and out-of-sample data.

3. **Overfitting Metrics**: Implement a function that calculates stability ratio: `stability_ratio = out_of_sample_return / in_sample_return`. If the ratio is close to 1.0 — the strategy is stable.

## Homework

1. Create a strategy testing system:
   - Load historical data for a year
   - Split into in-sample (first 9 months) and out-of-sample (last 3 months)
   - Test the strategy on both sets
   - Compare results and identify signs of overfitting

2. Implement walk-forward optimization:
   - Use a sliding window (e.g., 30 days training, 10 days test)
   - On each iteration, find optimal strategy parameters
   - Test found parameters on the next period
   - Plot cumulative returns

3. Create a Monte Carlo validation system:
   - Take historical data
   - Create 1000 random permutations of the data
   - Test the strategy on each permutation
   - Calculate average return and standard deviation
   - Determine if the strategy is robust to data order

4. Implement time series cross-validation:
   - Split data into 5 sequential blocks
   - Use blocks 1-3 for training, block 4 for validation, block 5 for testing
   - Repeat the process with shifts
   - Calculate average prediction accuracy

## Navigation

[← Previous day](../290-walk-forward-analysis/en.md) | [Next day →](../292-parameter-optimization/en.md)
