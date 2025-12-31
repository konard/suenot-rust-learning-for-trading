# Day 299: Multi-Instrument Testing

## Trading Analogy

Imagine you've developed a trading strategy and tested it on Bitcoin. The backtest shows excellent results: +150% in one year! You're excited and deploy it to the real market, but now on Ethereum—and suddenly you're losing money. What went wrong?

The strategy was **overfitted** to Bitcoin's specifics. It doesn't account for the fact that different instruments have different characteristics:
- Bitcoin might have 3-5% daily volatility
- Ethereum can be more volatile (4-7%)
- Stocks are usually less volatile (1-2%)
- Forex pairs have their own unique patterns

**Multi-instrument testing** is about validating your strategy across different trading instruments to ensure it works not due to luck on one asset, but because of solid trading principles. It's like testing a car on different road types (asphalt, dirt, snow) instead of only on a highway in perfect weather.

## What is Multi-Instrument Testing?

Multi-instrument testing is a method of validating trading strategies where the same strategy is tested across different financial instruments to verify its robustness and universality.

### Why Do We Need It?

| Reason | Description |
|--------|-------------|
| **Detect Overfitting** | A strategy might be curve-fitted to one instrument |
| **Verify Universality** | Good strategies work across different markets |
| **Risk Assessment** | Different instruments show how the strategy behaves in various conditions |
| **Diversification** | Understanding which instruments work best with your strategy |
| **Robustness** | Strategy should be robust to different market regimes |

## Basic Implementation: Testing on One Instrument

First, let's create a structure for testing a single strategy:

```rust
#[derive(Debug, Clone)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit_pct: f64,
    holding_bars: usize,
}

#[derive(Debug)]
struct BacktestResult {
    instrument: String,
    total_trades: usize,
    winning_trades: usize,
    total_return: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    win_rate: f64,
}

impl BacktestResult {
    fn new(instrument: String) -> Self {
        BacktestResult {
            instrument,
            total_trades: 0,
            winning_trades: 0,
            total_return: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
        }
    }

    fn calculate_metrics(&mut self, trades: &[Trade]) {
        self.total_trades = trades.len();
        self.winning_trades = trades.iter().filter(|t| t.profit_pct > 0.0).count();
        self.win_rate = if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        };

        // Total return
        self.total_return = trades.iter().map(|t| t.profit_pct).sum();

        // Maximum drawdown (simplified)
        let mut peak = 0.0;
        let mut current = 0.0;
        let mut max_dd = 0.0;

        for trade in trades {
            current += trade.profit_pct;
            if current > peak {
                peak = current;
            }
            let dd = peak - current;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        self.max_drawdown = max_dd;

        // Sharpe Ratio (simplified)
        if !trades.is_empty() {
            let mean = self.total_return / trades.len() as f64;
            let variance: f64 = trades
                .iter()
                .map(|t| (t.profit_pct - mean).powi(2))
                .sum::<f64>()
                / trades.len() as f64;
            let std_dev = variance.sqrt();
            self.sharpe_ratio = if std_dev > 0.0 {
                mean / std_dev
            } else {
                0.0
            };
        }
    }
}

fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    if period == 0 || prices.is_empty() {
        return vec![0.0; prices.len()];
    }
    for i in 0..prices.len() {
        if i + 1 < period {
            sma.push(0.0);
        } else {
            let start = i + 1 - period;
            let sum: f64 = prices[start..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }
    sma
}

fn backtest_sma_crossover(data: &[OHLCV], fast_period: usize, slow_period: usize) -> Vec<Trade> {
    let closes: Vec<f64> = data.iter().map(|bar| bar.close).collect();
    let fast_sma = simple_moving_average(&closes, fast_period);
    let slow_sma = simple_moving_average(&closes, slow_period);

    let mut trades = Vec::new();
    let mut position: Option<(f64, usize)> = None; // (entry_price, entry_index)

    for i in slow_period..data.len() {
        if fast_sma[i] == 0.0 || slow_sma[i] == 0.0 {
            continue;
        }

        // Buy signal: fast MA crosses above slow MA
        if position.is_none()
            && fast_sma[i] > slow_sma[i]
            && fast_sma[i - 1] <= slow_sma[i - 1]
        {
            position = Some((data[i].close, i));
        }
        // Sell signal: fast MA crosses below slow MA
        else if let Some((entry_price, entry_idx)) = position {
            if fast_sma[i] < slow_sma[i] && fast_sma[i - 1] >= slow_sma[i - 1] {
                let exit_price = data[i].close;
                let profit_pct = ((exit_price - entry_price) / entry_price) * 100.0;
                trades.push(Trade {
                    entry_price,
                    exit_price,
                    profit_pct,
                    holding_bars: i - entry_idx,
                });
                position = None;
            }
        }
    }

    // Close any open position at the end
    if let Some((entry_price, entry_idx)) = position {
        let exit_price = data[data.len() - 1].close;
        let profit_pct = ((exit_price - entry_price) / entry_price) * 100.0;
        trades.push(Trade {
            entry_price,
            exit_price,
            profit_pct,
            holding_bars: data.len() - 1 - entry_idx,
        });
    }

    trades
}

fn main() {
    // Simulated Bitcoin data
    let btc_data: Vec<OHLCV> = (0..200)
        .map(|i| {
            let trend = 40000.0 + i as f64 * 50.0;
            let cycle = 2000.0 * (i as f64 * 0.05).sin();
            let noise = 100.0 * (i as f64 * 7.0).sin();
            let price = trend + cycle + noise;
            OHLCV {
                timestamp: i as u64,
                open: price - 50.0,
                high: price + 100.0,
                low: price - 100.0,
                close: price,
                volume: 1000000.0,
            }
        })
        .collect();

    println!("=== Testing SMA Crossover Strategy on Bitcoin ===\n");

    // Run backtest
    let trades = backtest_sma_crossover(&btc_data, 10, 30);
    let mut result = BacktestResult::new("BTC/USD".to_string());
    result.calculate_metrics(&trades);

    println!("Instrument: {}", result.instrument);
    println!("Total trades: {}", result.total_trades);
    println!("Winning trades: {}", result.winning_trades);
    println!("Win Rate: {:.2}%", result.win_rate);
    println!("Total return: {:.2}%", result.total_return);
    println!("Maximum drawdown: {:.2}%", result.max_drawdown);
    println!("Sharpe Ratio: {:.2}", result.sharpe_ratio);
}
```

## Multi-Instrument Testing

Now let's extend our code to test across multiple instruments:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Instrument {
    symbol: String,
    data: Vec<OHLCV>,
}

impl Instrument {
    fn new(symbol: &str, data: Vec<OHLCV>) -> Self {
        Instrument {
            symbol: symbol.to_string(),
            data,
        }
    }
}

struct MultiInstrumentTester {
    instruments: Vec<Instrument>,
    fast_period: usize,
    slow_period: usize,
}

impl MultiInstrumentTester {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        MultiInstrumentTester {
            instruments: Vec::new(),
            fast_period,
            slow_period,
        }
    }

    fn add_instrument(&mut self, instrument: Instrument) {
        self.instruments.push(instrument);
    }

    fn run_tests(&self) -> Vec<BacktestResult> {
        let mut results = Vec::new();

        for instrument in &self.instruments {
            let trades = backtest_sma_crossover(&instrument.data, self.fast_period, self.slow_period);
            let mut result = BacktestResult::new(instrument.symbol.clone());
            result.calculate_metrics(&trades);
            results.push(result);
        }

        results
    }

    fn print_summary(&self, results: &[BacktestResult]) {
        println!("\n=== Summary Across All Instruments ===\n");
        println!("{:<12} {:<12} {:<12} {:<15} {:<15} {:<12}",
            "Instrument", "Trades", "Win Rate", "Return", "Drawdown", "Sharpe");
        println!("{}", "-".repeat(85));

        for result in results {
            println!("{:<12} {:<12} {:<11.2}% {:<14.2}% {:<14.2}% {:<12.2}",
                result.instrument,
                result.total_trades,
                result.win_rate,
                result.total_return,
                result.max_drawdown,
                result.sharpe_ratio
            );
        }

        // Average metrics
        let avg_win_rate = results.iter().map(|r| r.win_rate).sum::<f64>() / results.len() as f64;
        let avg_return = results.iter().map(|r| r.total_return).sum::<f64>() / results.len() as f64;
        let avg_sharpe = results.iter().map(|r| r.sharpe_ratio).sum::<f64>() / results.len() as f64;

        println!("{}", "-".repeat(85));
        println!("{:<12} {:<12} {:<11.2}% {:<14.2}% {:<14} {:<12.2}",
            "AVERAGE", "-", avg_win_rate, avg_return, "-", avg_sharpe);
    }
}

fn generate_synthetic_data(symbol: &str, base_price: f64, volatility: f64, trend: f64) -> Vec<OHLCV> {
    (0..200)
        .map(|i| {
            let trend_component = base_price + i as f64 * trend;
            let cycle = volatility * (i as f64 * 0.05).sin();
            let noise = volatility * 0.2 * (i as f64 * 7.0).sin();
            let price = trend_component + cycle + noise;
            OHLCV {
                timestamp: i as u64,
                open: price - volatility * 0.1,
                high: price + volatility * 0.15,
                low: price - volatility * 0.15,
                close: price,
                volume: 1000000.0 * (1.0 + (i as f64 * 0.01).sin() * 0.3),
            }
        })
        .collect()
}

fn main() {
    println!("=== Multi-Instrument Testing ===\n");

    // Create tester
    let mut tester = MultiInstrumentTester::new(10, 30);

    // Add different instruments with different characteristics
    tester.add_instrument(Instrument::new(
        "BTC/USD",
        generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0),
    ));

    tester.add_instrument(Instrument::new(
        "ETH/USD",
        generate_synthetic_data("ETH", 2500.0, 150.0, 3.0),
    ));

    tester.add_instrument(Instrument::new(
        "AAPL",
        generate_synthetic_data("AAPL", 150.0, 5.0, 0.2),
    ));

    tester.add_instrument(Instrument::new(
        "EUR/USD",
        generate_synthetic_data("EUR", 1.1, 0.02, 0.0001),
    ));

    tester.add_instrument(Instrument::new(
        "GOLD",
        generate_synthetic_data("GOLD", 1800.0, 30.0, 0.5),
    ));

    // Run tests
    let results = tester.run_tests();

    // Print results
    tester.print_summary(&results);

    // Robustness analysis
    println!("\n=== Strategy Robustness Analysis ===\n");

    let profitable_instruments = results.iter().filter(|r| r.total_return > 0.0).count();
    let consistency_score = (profitable_instruments as f64 / results.len() as f64) * 100.0;

    println!("Profitable instruments: {}/{}", profitable_instruments, results.len());
    println!("Consistency score: {:.1}%", consistency_score);

    if consistency_score >= 70.0 {
        println!("\n✓ Strategy shows good robustness (>70%)");
    } else if consistency_score >= 50.0 {
        println!("\n⚠ Strategy shows moderate robustness (50-70%)");
    } else {
        println!("\n✗ Strategy is not robust (<50%)");
    }
}
```

## Advanced Metrics: Correlation Analysis

It's important to understand how results across different instruments correlate with each other:

```rust
#[derive(Debug)]
struct CorrelationMatrix {
    instruments: Vec<String>,
    matrix: Vec<Vec<f64>>,
}

impl CorrelationMatrix {
    fn calculate(results: &[BacktestResult]) -> Self {
        let n = results.len();
        let instruments: Vec<String> = results.iter().map(|r| r.instrument.clone()).collect();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 1.0;
                } else {
                    // Simplified correlation based on metric ratios
                    let corr = calculate_simple_correlation(
                        results[i].total_return,
                        results[i].sharpe_ratio,
                        results[j].total_return,
                        results[j].sharpe_ratio,
                    );
                    matrix[i][j] = corr;
                }
            }
        }

        CorrelationMatrix {
            instruments,
            matrix,
        }
    }

    fn print(&self) {
        println!("\n=== Results Correlation Matrix ===\n");
        print!("{:<12}", "");
        for symbol in &self.instruments {
            print!("{:<12}", symbol);
        }
        println!();

        for (i, symbol) in self.instruments.iter().enumerate() {
            print!("{:<12}", symbol);
            for j in 0..self.instruments.len() {
                print!("{:<12.2}", self.matrix[i][j]);
            }
            println!();
        }
    }
}

fn calculate_simple_correlation(
    return1: f64,
    sharpe1: f64,
    return2: f64,
    sharpe2: f64,
) -> f64 {
    // Simplified formula: average of normalized metrics
    let norm_return = 1.0 - ((return1 - return2).abs() / (return1.abs() + return2.abs() + 0.01));
    let norm_sharpe = 1.0 - ((sharpe1 - sharpe2).abs() / (sharpe1.abs() + sharpe2.abs() + 0.01));
    (norm_return + norm_sharpe) / 2.0
}

fn main() {
    println!("=== Correlation Analysis ===\n");

    let mut tester = MultiInstrumentTester::new(10, 30);

    tester.add_instrument(Instrument::new("BTC/USD", generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0)));
    tester.add_instrument(Instrument::new("ETH/USD", generate_synthetic_data("ETH", 2500.0, 150.0, 3.0)));
    tester.add_instrument(Instrument::new("AAPL", generate_synthetic_data("AAPL", 150.0, 5.0, 0.2)));

    let results = tester.run_tests();
    tester.print_summary(&results);

    let correlation = CorrelationMatrix::calculate(&results);
    correlation.print();
}
```

## Adaptive Parameters for Different Instruments

Often a strategy requires different parameters for different instruments:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct InstrumentConfig {
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    stop_loss_pct: f64,
}

struct AdaptiveMultiInstrumentTester {
    configs: HashMap<String, InstrumentConfig>,
    instruments: Vec<Instrument>,
}

impl AdaptiveMultiInstrumentTester {
    fn new() -> Self {
        AdaptiveMultiInstrumentTester {
            configs: HashMap::new(),
            instruments: Vec::new(),
        }
    }

    fn add_instrument_with_config(&mut self, instrument: Instrument, config: InstrumentConfig) {
        self.configs.insert(instrument.symbol.clone(), config);
        self.instruments.push(instrument);
    }

    fn run_adaptive_tests(&self) -> Vec<BacktestResult> {
        let mut results = Vec::new();

        for instrument in &self.instruments {
            if let Some(config) = self.configs.get(&instrument.symbol) {
                let trades = backtest_sma_crossover(
                    &instrument.data,
                    config.fast_period,
                    config.slow_period,
                );
                let mut result = BacktestResult::new(instrument.symbol.clone());
                result.calculate_metrics(&trades);
                results.push(result);
            }
        }

        results
    }
}

fn main() {
    println!("=== Adaptive Multi-Instrument Testing ===\n");

    let mut tester = AdaptiveMultiInstrumentTester::new();

    // BTC: slower parameters due to high volatility
    tester.add_instrument_with_config(
        Instrument::new("BTC/USD", generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0)),
        InstrumentConfig {
            symbol: "BTC/USD".to_string(),
            fast_period: 15,
            slow_period: 40,
            stop_loss_pct: 5.0,
        },
    );

    // ETH: medium parameters
    tester.add_instrument_with_config(
        Instrument::new("ETH/USD", generate_synthetic_data("ETH", 2500.0, 150.0, 3.0)),
        InstrumentConfig {
            symbol: "ETH/USD".to_string(),
            fast_period: 12,
            slow_period: 35,
            stop_loss_pct: 4.0,
        },
    );

    // AAPL: faster parameters for stocks
    tester.add_instrument_with_config(
        Instrument::new("AAPL", generate_synthetic_data("AAPL", 150.0, 5.0, 0.2)),
        InstrumentConfig {
            symbol: "AAPL".to_string(),
            fast_period: 8,
            slow_period: 20,
            stop_loss_pct: 2.0,
        },
    );

    let results = tester.run_adaptive_tests();

    println!("{:<12} {:<12} {:<12} {:<12} {:<15}",
        "Instrument", "Fast MA", "Slow MA", "Trades", "Return");
    println!("{}", "-".repeat(70));

    for result in &results {
        if let Some(config) = tester.configs.get(&result.instrument) {
            println!("{:<12} {:<12} {:<12} {:<12} {:<14.2}%",
                result.instrument,
                config.fast_period,
                config.slow_period,
                result.total_trades,
                result.total_return
            );
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Multi-Instrument Testing | Validating strategies across different trading instruments |
| Strategy Robustness | Good strategies work across different markets |
| Correlation Analysis | Understanding how results across instruments relate |
| Adaptive Parameters | Different instruments may require different settings |
| Consistency Assessment | Percentage of instruments where strategy is profitable |
| Diversification | Testing helps understand where strategy performs best |

## Practice Exercises

1. **Extended Metrics**: Add calculation of additional metrics for each instrument:
   - Average trade duration
   - Maximum consecutive losing trades
   - Profit Factor (sum of profits / sum of losses)
   - Recovery from drawdown

2. **Volatility Normalization**: Create a function that normalizes results by instrument volatility to fairly compare BTC and stocks.

3. **Automatic Optimization**: Implement a system that automatically finds optimal parameters for each instrument using grid search.

4. **Instrument Grouping**: Implement analysis that groups instruments by characteristics (crypto, stocks, forex) and shows which groups work best with the strategy.

## Homework

1. **Implement Portfolio Testing**: Create a system that runs the strategy simultaneously on all instruments as a portfolio, considering capital allocation across instruments.

2. **Add Real Data**: Use real historical data (via API or CSV files) instead of synthetic data for more accurate testing.

3. **Create Visualization**: Implement output in the form of tables or charts showing returns for each instrument.

4. **Multi-Timeframe Testing**: Extend the tester to validate the strategy not only across different instruments but also across different timeframes (1m, 5m, 1h, 1d).

## Navigation

[← Previous day](../293-grid-search-parameter-sweep/en.md) | [Next day →](../300-advanced-backtesting/en.md)
