# Day 296: Monte Carlo Simulation

## Trading Analogy

Imagine you've developed a trading strategy that showed good profits on historical data. But you're wondering:
- What if I had entered the market one day earlier or later?
- How much would the results change with different trade sequences?
- What's the real range of possible outcomes, not just one historical path?

**Monte Carlo simulation** is like replaying thousands of alternative versions of your trading history. Imagine taking a deck of your past trades (profitable and losing) and shuffling it thousands of times, each time getting a new sequence of results.

It's like a "what if" simulator that helps you understand:
- What's the probability of a drawdown exceeding 20%?
- What's the maximum loss that could occur with the current strategy?
- How stable are my results under different market conditions?

## What is Monte Carlo Simulation?

Monte Carlo is a numerical modeling method that uses randomness to evaluate probabilistic characteristics of a system:

| Aspect | Description |
|--------|-------------|
| **Principle** | Repeated random sampling to obtain a distribution of outcomes |
| **Application in Trading** | Risk assessment, strategy robustness testing, forecasting |
| **Number of Iterations** | Usually 1,000-10,000 simulations for reliable results |
| **Advantage** | Shows not one result but a range of possible outcomes |
| **Disadvantage** | Assumes the future resembles the past (may be incorrect) |

## Simple Example: Trade Shuffling

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct Trade {
    profit: f64,
    date: String,
}

#[derive(Debug)]
struct SimulationResult {
    final_equity: f64,
    max_drawdown: f64,
    total_return: f64,
}

fn calculate_equity_curve(trades: &[Trade], initial_capital: f64) -> (Vec<f64>, f64) {
    let mut equity_curve = vec![initial_capital];
    let mut max_equity = initial_capital;
    let mut max_drawdown = 0.0;

    for trade in trades {
        let new_equity = equity_curve.last().unwrap() + trade.profit;
        equity_curve.push(new_equity);

        if new_equity > max_equity {
            max_equity = new_equity;
        }

        let drawdown = (max_equity - new_equity) / max_equity * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    (equity_curve, max_drawdown)
}

fn run_single_simulation(trades: &[Trade], initial_capital: f64) -> SimulationResult {
    let mut rng = thread_rng();
    let mut shuffled = trades.to_vec();
    shuffled.shuffle(&mut rng);

    let (equity_curve, max_drawdown) = calculate_equity_curve(&shuffled, initial_capital);
    let final_equity = *equity_curve.last().unwrap();
    let total_return = (final_equity - initial_capital) / initial_capital * 100.0;

    SimulationResult {
        final_equity,
        max_drawdown,
        total_return,
    }
}

fn monte_carlo_analysis(
    trades: &[Trade],
    initial_capital: f64,
    simulations: usize,
) -> Vec<SimulationResult> {
    (0..simulations)
        .map(|_| run_single_simulation(trades, initial_capital))
        .collect()
}

fn main() {
    // Historical strategy trades
    let historical_trades = vec![
        Trade { profit: 150.0, date: "2024-01-15".to_string() },
        Trade { profit: -80.0, date: "2024-01-16".to_string() },
        Trade { profit: 200.0, date: "2024-01-17".to_string() },
        Trade { profit: -120.0, date: "2024-01-18".to_string() },
        Trade { profit: 300.0, date: "2024-01-19".to_string() },
        Trade { profit: 100.0, date: "2024-01-22".to_string() },
        Trade { profit: -90.0, date: "2024-01-23".to_string() },
        Trade { profit: 250.0, date: "2024-01-24".to_string() },
        Trade { profit: -150.0, date: "2024-01-25".to_string() },
        Trade { profit: 180.0, date: "2024-01-26".to_string() },
    ];

    let initial_capital = 10_000.0;
    let num_simulations = 1_000;

    println!("Running {} Monte Carlo simulations...\n", num_simulations);

    let results = monte_carlo_analysis(&historical_trades, initial_capital, num_simulations);

    // Analyze results
    let total_returns: Vec<f64> = results.iter().map(|r| r.total_return).collect();
    let max_drawdowns: Vec<f64> = results.iter().map(|r| r.max_drawdown).collect();

    let avg_return = total_returns.iter().sum::<f64>() / total_returns.len() as f64;
    let min_return = total_returns.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_return = total_returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let avg_drawdown = max_drawdowns.iter().sum::<f64>() / max_drawdowns.len() as f64;
    let worst_drawdown = max_drawdowns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("=== Monte Carlo Simulation Results ===\n");
    println!("Returns:");
    println!("  Average: {:.2}%", avg_return);
    println!("  Minimum: {:.2}%", min_return);
    println!("  Maximum: {:.2}%", max_return);
    println!("\nDrawdown:");
    println!("  Average: {:.2}%", avg_drawdown);
    println!("  Maximum (worst case): {:.2}%", worst_drawdown);

    // Percentiles
    let mut sorted_returns = total_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p5 = sorted_returns[(sorted_returns.len() as f64 * 0.05) as usize];
    let p95 = sorted_returns[(sorted_returns.len() as f64 * 0.95) as usize];

    println!("\n90% Confidence Interval:");
    println!("  5th percentile: {:.2}%", p5);
    println!("  95th percentile: {:.2}%", p95);
    println!("\nProbability of loss: {:.1}%",
        (sorted_returns.iter().filter(|&&r| r < 0.0).count() as f64
         / sorted_returns.len() as f64 * 100.0));
}
```

## Advanced Simulation: With Position Sizing

```rust
use rand::Rng;

#[derive(Debug, Clone)]
struct DetailedTrade {
    profit_pct: f64,  // Profit as percentage of capital
    position_size: f64,  // Position size (0.0-1.0)
}

fn calculate_compounded_equity(
    trades: &[DetailedTrade],
    initial_capital: f64,
) -> (Vec<f64>, f64, f64) {
    let mut equity = initial_capital;
    let mut equity_curve = vec![equity];
    let mut max_equity = equity;
    let mut max_drawdown = 0.0;

    for trade in trades {
        // Profit depends on position size
        let position_value = equity * trade.position_size;
        let profit = position_value * (trade.profit_pct / 100.0);

        equity += profit;
        equity_curve.push(equity);

        if equity > max_equity {
            max_equity = equity;
        }

        let drawdown = (max_equity - equity) / max_equity * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    let total_return = (equity - initial_capital) / initial_capital * 100.0;
    (equity_curve, max_drawdown, total_return)
}

fn monte_carlo_with_position_sizing(
    base_trades: &[DetailedTrade],
    initial_capital: f64,
    simulations: usize,
) {
    let mut rng = thread_rng();
    let mut all_final_equities = Vec::new();
    let mut all_max_drawdowns = Vec::new();

    for _ in 0..simulations {
        // Shuffle trade order
        let mut shuffled = base_trades.to_vec();
        shuffled.shuffle(&mut rng);

        let (_, max_dd, _) = calculate_compounded_equity(&shuffled, initial_capital);
        let final_equity = calculate_compounded_equity(&shuffled, initial_capital).0.last().unwrap().clone();

        all_final_equities.push(final_equity);
        all_max_drawdowns.push(max_dd);
    }

    // Statistics
    all_final_equities.sort_by(|a, b| a.partial_cmp(b).unwrap());
    all_max_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_equity = all_final_equities[simulations / 2];
    let p5_equity = all_final_equities[(simulations as f64 * 0.05) as usize];
    let p95_equity = all_final_equities[(simulations as f64 * 0.95) as usize];

    let median_dd = all_max_drawdowns[simulations / 2];
    let p95_dd = all_max_drawdowns[(simulations as f64 * 0.95) as usize];

    println!("\n=== Monte Carlo with Position Sizing ===\n");
    println!("Final Equity:");
    println!("  Median: ${:.2}", median_equity);
    println!("  5th percentile (worst): ${:.2}", p5_equity);
    println!("  95th percentile (best): ${:.2}", p95_equity);
    println!("\nMaximum Drawdown:");
    println!("  Median: {:.2}%", median_dd);
    println!("  95th percentile (worst case): {:.2}%", p95_dd);

    // Probability of ruin (equity < 50% of initial)
    let ruin_threshold = initial_capital * 0.5;
    let ruin_count = all_final_equities.iter().filter(|&&eq| eq < ruin_threshold).count();
    println!("\nProbability of losing >50% capital: {:.2}%",
        ruin_count as f64 / simulations as f64 * 100.0);
}

fn main() {
    let trades = vec![
        DetailedTrade { profit_pct: 3.5, position_size: 0.25 },
        DetailedTrade { profit_pct: -2.0, position_size: 0.25 },
        DetailedTrade { profit_pct: 5.0, position_size: 0.3 },
        DetailedTrade { profit_pct: -3.0, position_size: 0.25 },
        DetailedTrade { profit_pct: 4.2, position_size: 0.25 },
        DetailedTrade { profit_pct: 2.8, position_size: 0.2 },
        DetailedTrade { profit_pct: -1.5, position_size: 0.25 },
        DetailedTrade { profit_pct: 6.0, position_size: 0.3 },
        DetailedTrade { profit_pct: -2.5, position_size: 0.25 },
        DetailedTrade { profit_pct: 3.0, position_size: 0.25 },
    ];

    monte_carlo_with_position_sizing(&trades, 10_000.0, 5_000);
}
```

## Parallel Monte Carlo with Rayon

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct MonteCarloStats {
    returns: Vec<f64>,
    drawdowns: Vec<f64>,
    sharpe_ratios: Vec<f64>,
}

impl MonteCarloStats {
    fn new() -> Self {
        Self {
            returns: Vec::new(),
            drawdowns: Vec::new(),
            sharpe_ratios: Vec::new(),
        }
    }

    fn add_result(&mut self, ret: f64, dd: f64, sharpe: f64) {
        self.returns.push(ret);
        self.drawdowns.push(dd);
        self.sharpe_ratios.push(sharpe);
    }

    fn analyze(&mut self) {
        self.returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.sharpe_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    fn percentile(&self, data: &[f64], p: f64) -> f64 {
        let idx = (data.len() as f64 * p) as usize;
        data[idx.min(data.len() - 1)]
    }

    fn print_analysis(&self) {
        println!("\n=== Analysis of {} Simulations ===\n", self.returns.len());

        println!("Returns:");
        println!("  Median: {:.2}%", self.percentile(&self.returns, 0.5));
        println!("  5%: {:.2}%", self.percentile(&self.returns, 0.05));
        println!("  95%: {:.2}%", self.percentile(&self.returns, 0.95));

        println!("\nDrawdown:");
        println!("  Median: {:.2}%", self.percentile(&self.drawdowns, 0.5));
        println!("  95% (worst case): {:.2}%", self.percentile(&self.drawdowns, 0.95));

        println!("\nSharpe Ratio:");
        println!("  Median: {:.2}", self.percentile(&self.sharpe_ratios, 0.5));
        println!("  5%: {:.2}", self.percentile(&self.sharpe_ratios, 0.05));
        println!("  95%: {:.2}", self.percentile(&self.sharpe_ratios, 0.95));
    }
}

fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        0.0
    } else {
        (mean_return - risk_free_rate) / std_dev
    }
}

fn parallel_monte_carlo(
    trades: &[Trade],
    initial_capital: f64,
    simulations: usize,
) -> MonteCarloStats {
    let stats = Arc::new(Mutex::new(MonteCarloStats::new()));

    (0..simulations).into_par_iter().for_each(|_| {
        let result = run_single_simulation(trades, initial_capital);

        // Calculate daily returns (simplified)
        let daily_returns = vec![result.total_return / 252.0; 252];
        let sharpe = calculate_sharpe_ratio(&daily_returns, 0.02);

        let mut stats_lock = stats.lock().unwrap();
        stats_lock.add_result(result.total_return, result.max_drawdown, sharpe);
    });

    let mut final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();
    final_stats.analyze();
    final_stats
}

fn main() {
    let trades = vec![
        Trade { profit: 150.0, date: "2024-01-15".to_string() },
        Trade { profit: -80.0, date: "2024-01-16".to_string() },
        Trade { profit: 200.0, date: "2024-01-17".to_string() },
        Trade { profit: -120.0, date: "2024-01-18".to_string() },
        Trade { profit: 300.0, date: "2024-01-19".to_string() },
        Trade { profit: 100.0, date: "2024-01-22".to_string() },
        Trade { profit: -90.0, date: "2024-01-23".to_string() },
        Trade { profit: 250.0, date: "2024-01-24".to_string() },
        Trade { profit: -150.0, date: "2024-01-25".to_string() },
        Trade { profit: 180.0, date: "2024-01-26".to_string() },
    ];

    use std::time::Instant;
    let start = Instant::now();

    let stats = parallel_monte_carlo(&trades, 10_000.0, 10_000);

    let duration = start.elapsed();
    stats.print_analysis();

    println!("\nExecution time: {:?}", duration);
}
```

## Variation: Synthetic Price Generation

```rust
use rand::distributions::{Distribution, Normal};

fn generate_synthetic_prices(
    initial_price: f64,
    num_periods: usize,
    mean_return: f64,      // Mean daily return
    volatility: f64,       // Daily volatility
) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(mean_return, volatility);

    let mut prices = vec![initial_price];

    for _ in 0..num_periods {
        let last_price = *prices.last().unwrap();
        let return_pct = normal.sample(&mut rng);
        let new_price = last_price * (1.0 + return_pct / 100.0);
        prices.push(new_price);
    }

    prices
}

fn monte_carlo_price_simulation(
    initial_price: f64,
    num_periods: usize,
    simulations: usize,
    mean_return: f64,
    volatility: f64,
) {
    let mut final_prices = Vec::new();

    for _ in 0..simulations {
        let prices = generate_synthetic_prices(
            initial_price,
            num_periods,
            mean_return,
            volatility,
        );
        final_prices.push(*prices.last().unwrap());
    }

    final_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("\n=== Asset Price Simulation ===\n");
    println!("Initial price: ${:.2}", initial_price);
    println!("Periods: {}", num_periods);
    println!("Simulations: {}", simulations);
    println!("\nFinal price:");
    println!("  5th percentile: ${:.2}",
        final_prices[(simulations as f64 * 0.05) as usize]);
    println!("  Median: ${:.2}", final_prices[simulations / 2]);
    println!("  95th percentile: ${:.2}",
        final_prices[(simulations as f64 * 0.95) as usize]);

    let prices_below_initial = final_prices.iter()
        .filter(|&&p| p < initial_price)
        .count();
    println!("\nProbability of price decline: {:.1}%",
        prices_below_initial as f64 / simulations as f64 * 100.0);
}

fn main() {
    monte_carlo_price_simulation(
        100.0,    // Initial price $100
        252,      // 1 year of trading days
        10_000,   // 10k simulations
        0.05,     // 0.05% mean daily return
        1.5,      // 1.5% daily volatility
    );
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Monte Carlo Simulation | Method for evaluating probabilistic characteristics through random sampling |
| Trade Shuffling | Testing strategy robustness to trade order |
| Position Sizing | Accounting for position size in compounding calculations |
| Parallelization | Using Rayon to accelerate simulations |
| Synthetic Prices | Generating random price trajectories |
| Statistical Analysis | Percentiles, confidence intervals, probabilities |

## Practical Exercises

1. **Basic Simulation**: Implement Monte Carlo for a list of 20 trades, run 1000 simulations and find:
   - Average return
   - Median maximum drawdown
   - Probability of loss

2. **Compounding**: Modify the simulation so each trade accounts for position size as a percentage of current capital.

3. **Visualization**: Save results of 100 equity curves to a CSV file for plotting a fan chart of possible trajectories.

4. **Value at Risk (VaR)**: Calculate 95% VaR — the maximum loss that won't be exceeded in 95% of cases.

## Homework

1. **Simulation with Reinvestment**: Create a Monte Carlo simulation where profit from each trade is reinvested (next position size increases proportionally to capital).

2. **Stress Testing**: Run simulations with different volatility levels (normal, high, extreme) and compare results.

3. **Position Size Optimization**: Use Monte Carlo to find the optimal position size that maximizes Sharpe Ratio while constraining maximum drawdown.

4. **Trade Correlation**: Add a model where consecutive trades have correlation (e.g., after a losing trade, the probability of the next losing trade is higher).

## Navigation

[← Previous day](../293-grid-search-parameter-sweep/en.md) | [Next day →](../297-bootstrap-resampling/en.md)
