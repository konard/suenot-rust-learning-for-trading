// Test code from Chapter 299: Multi-Instrument Testing

use std::collections::HashMap;

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

fn generate_synthetic_data(_symbol: &str, base_price: f64, volatility: f64, trend: f64) -> Vec<OHLCV> {
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
