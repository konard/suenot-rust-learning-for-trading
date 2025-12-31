use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug)]
struct BacktestResult {
    train_sharpe: f64,
    test_sharpe: f64,
    train_profit: f64,
    test_profit: f64,
    num_parameters: usize,
    num_trades: usize,
}

impl BacktestResult {
    fn is_overfitted(&self) -> bool {
        let sharpe_degradation = (self.train_sharpe - self.test_sharpe) / self.train_sharpe;
        let parameter_ratio = self.num_parameters as f64 / self.num_trades as f64;
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

#[derive(Debug, Clone)]
struct StrategyParams {
    ma_short: usize,
    ma_long: usize,
    stop_loss: f64,
    take_profit: f64,
}

impl StrategyParams {
    fn count_params(&self) -> usize {
        4
    }
}

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
}

fn backtest_strategy(prices: &[f64], params: &StrategyParams) -> Vec<Trade> {
    let mut trades = Vec::new();

    if prices.len() < params.ma_long {
        return trades;
    }

    let mut position_open = false;
    let mut entry_price = 0.0;

    for i in params.ma_long + 1..prices.len() {
        let short_ma: f64 = prices[i - params.ma_short..i].iter().sum::<f64>()
            / params.ma_short as f64;
        let long_ma: f64 = prices[i - params.ma_long..i].iter().sum::<f64>()
            / params.ma_long as f64;

        let prev_short_ma: f64 = prices[i - params.ma_short - 1..i - 1].iter().sum::<f64>()
            / params.ma_short as f64;
        let prev_long_ma: f64 = prices[i - params.ma_long - 1..i - 1].iter().sum::<f64>()
            / params.ma_long as f64;

        if !position_open && prev_short_ma <= prev_long_ma && short_ma > long_ma {
            position_open = true;
            entry_price = prices[i];
        }

        if position_open {
            let current_pnl = (prices[i] - entry_price) / entry_price;

            if current_pnl <= -params.stop_loss || current_pnl >= params.take_profit {
                trades.push(Trade {
                    entry_price,
                    exit_price: prices[i],
                    pnl: current_pnl,
                });
                position_open = false;
            } else if prev_short_ma >= prev_long_ma && short_ma < long_ma {
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

    mean_return / std_dev * (252.0_f64).sqrt()
}

fn walk_forward_analysis(prices: &[f64], params: &StrategyParams) -> BacktestResult {
    let split_point = (prices.len() as f64 * 0.7) as usize;

    let train_prices = &prices[..split_point];
    let test_prices = &prices[split_point..];

    let train_trades = backtest_strategy(train_prices, params);
    let train_sharpe = calculate_sharpe(&train_trades);
    let train_profit: f64 = train_trades.iter().map(|t| t.pnl).sum();

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

fn generate_price_data(days: usize, start_price: f64) -> Vec<f64> {
    let mut prices = Vec::with_capacity(days);
    let mut price = start_price;

    for i in 0..days {
        // More pronounced trends with larger movements
        let trend = (i as f64 * 0.05).sin() * 500.0;
        let noise = ((i * 13) % 37) as f64 - 18.0;
        price += trend + noise;
        prices.push(price);
    }

    prices
}

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
    println!("=== Chapter 294: Testing Overfitting Detection ===\n");

    // Test 1: Basic overfitting detection
    println!("Test 1: Good vs Overfitted Strategy\n");

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

    let overfitted_strategy = BacktestResult {
        train_sharpe: 3.5,
        test_sharpe: 0.8,
        train_profit: 0.85,
        test_profit: -0.12,
        num_parameters: 25,
        num_trades: 80,
    };

    overfitted_strategy.print_diagnosis();
    println!("\n{}\n", "=".repeat(60));

    // Test 2: Walk-forward analysis
    println!("Test 2: Walk-Forward Analysis\n");

    let prices = generate_price_data(500, 42000.0);

    let simple_params = StrategyParams {
        ma_short: 5,
        ma_long: 20,
        stop_loss: 0.02,
        take_profit: 0.05,
    };

    println!("Simple strategy (4 parameters):");
    let simple_result = walk_forward_analysis(&prices, &simple_params);
    simple_result.print_diagnosis();
    println!("\n{}\n", "=".repeat(60));

    // Test 3: Monte Carlo simulation
    println!("Test 3: Monte Carlo Simulation\n");

    let trades = backtest_strategy(&prices, &simple_params);

    if !trades.is_empty() {
        let original_pnl: f64 = trades.iter().map(|t| t.pnl).sum();

        println!("Original profit: {:.2}%", original_pnl * 100.0);
        println!("Number of trades: {}", trades.len());

        let mc_results = monte_carlo_simulation(&trades, 1000);

        let p5 = mc_results[50];
        let p95 = mc_results[950];

        println!("90% confidence interval: [{:.2}%, {:.2}%]", p5 * 100.0, p95 * 100.0);

        if original_pnl < p5 || original_pnl > p95 {
            println!("⚠️  Result is outside the confidence interval!");
        } else {
            println!("✅ Result within normal range");
        }
    } else {
        println!("No trades generated - parameters may need adjustment");
    }

    println!("\n✅ All tests completed successfully!");
}
