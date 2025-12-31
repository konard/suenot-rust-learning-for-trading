# Day 290: Walk-Forward Analysis

## Trading Analogy

Imagine you've developed a profitable trading strategy based on historical data from 2020. You backtest it, and it shows amazing results—profits every month! Excited, you deploy it in 2021, but it starts losing money. What happened?

The strategy was **overfitted** to 2020 data. It learned to trade 2020's specific market conditions, not universal trading principles. It's like studying last year's exam questions and expecting the same questions this year.

**Walk-forward analysis** is like taking practice exams throughout the year instead of just studying old exams. You:
1. Train your strategy on January-March data
2. Test it on April (out-of-sample)
3. Re-train on February-April data
4. Test it on May
5. Keep walking forward through time

This simulates real trading where you continuously adapt your strategy to new market data while validating it on unseen periods.

## What is Walk-Forward Analysis?

Walk-forward analysis is a backtesting method that divides historical data into multiple in-sample (training) and out-of-sample (testing) periods that move forward through time. It helps:

1. **Detect Overfitting** — strategies that only work on historical data fail on out-of-sample periods
2. **Simulate Reality** — real trading involves continuous re-optimization
3. **Validate Robustness** — robust strategies perform consistently across all test windows
4. **Estimate Real Performance** — out-of-sample results better predict live trading

### Walk-Forward Process

```
Time ────────────────────────────────────────────────────►

Window 1:  [====Train====][Test]
Window 2:      [====Train====][Test]
Window 3:          [====Train====][Test]
Window 4:              [====Train====][Test]
                                        ...
```

Each window consists of:
- **In-Sample Period** — optimize strategy parameters
- **Out-of-Sample Period** — test optimized parameters without changes

## Basic Walk-Forward Implementation

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit: f64,
}

#[derive(Debug, Clone)]
struct StrategyParams {
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
}

#[derive(Debug)]
struct WalkForwardResult {
    window: usize,
    in_sample_profit: f64,
    out_sample_profit: f64,
    optimized_params: StrategyParams,
}

fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in 0..prices.len() {
        if i < period - 1 {
            sma.push(0.0);
        } else {
            let sum: f64 = prices[i - period + 1..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }

    sma
}

fn backtest_strategy(
    prices: &[f64],
    params: &StrategyParams,
) -> f64 {
    let fast_sma = simple_moving_average(prices, params.fast_period);
    let slow_sma = simple_moving_average(prices, params.slow_period);

    let mut equity = 10_000.0;
    let mut position: Option<f64> = None;

    for i in params.slow_period..prices.len() {
        if fast_sma[i] == 0.0 || slow_sma[i] == 0.0 {
            continue;
        }

        // Buy signal: fast crosses above slow
        if position.is_none()
            && fast_sma[i] > slow_sma[i]
            && fast_sma[i - 1] <= slow_sma[i - 1]
        {
            position = Some(prices[i]);
        }
        // Sell signal: fast crosses below slow OR stop loss
        else if let Some(entry_price) = position {
            let should_sell = fast_sma[i] < slow_sma[i] && fast_sma[i - 1] >= slow_sma[i - 1];
            let hit_stop_loss = (prices[i] - entry_price) / entry_price < -params.stop_loss;

            if should_sell || hit_stop_loss {
                let profit = (prices[i] - entry_price) / entry_price;
                equity *= 1.0 + profit;
                position = None;
            }
        }
    }

    // Close any open position
    if let Some(entry_price) = position {
        let profit = (prices[prices.len() - 1] - entry_price) / entry_price;
        equity *= 1.0 + profit;
    }

    (equity - 10_000.0) / 10_000.0 * 100.0 // Return as percentage
}

fn optimize_parameters(prices: &[f64]) -> (StrategyParams, f64) {
    let mut best_params = StrategyParams {
        fast_period: 10,
        slow_period: 20,
        stop_loss: 0.05,
    };
    let mut best_profit = f64::MIN;

    // Grid search over parameter space
    for fast in [5, 10, 15, 20].iter() {
        for slow in [20, 30, 50, 100].iter() {
            for stop in [0.02, 0.05, 0.10].iter() {
                if fast >= slow {
                    continue;
                }

                let params = StrategyParams {
                    fast_period: *fast,
                    slow_period: *slow,
                    stop_loss: *stop,
                };

                let profit = backtest_strategy(prices, &params);

                if profit > best_profit {
                    best_profit = profit;
                    best_params = params;
                }
            }
        }
    }

    (best_params, best_profit)
}

fn walk_forward_analysis(
    prices: &[f64],
    in_sample_size: usize,
    out_sample_size: usize,
) -> Vec<WalkForwardResult> {
    let mut results = Vec::new();
    let window_size = in_sample_size + out_sample_size;
    let num_windows = (prices.len() - in_sample_size) / out_sample_size;

    for window in 0..num_windows {
        let start_idx = window * out_sample_size;
        let in_sample_end = start_idx + in_sample_size;
        let out_sample_end = in_sample_end + out_sample_size;

        if out_sample_end > prices.len() {
            break;
        }

        // In-sample optimization
        let in_sample_data = &prices[start_idx..in_sample_end];
        let (optimized_params, in_sample_profit) = optimize_parameters(in_sample_data);

        // Out-of-sample testing
        let out_sample_data = &prices[in_sample_end..out_sample_end];
        let out_sample_profit = backtest_strategy(out_sample_data, &optimized_params);

        results.push(WalkForwardResult {
            window: window + 1,
            in_sample_profit,
            out_sample_profit,
            optimized_params: optimized_params.clone(),
        });

        println!("Window {}: In-sample: {:.2}%, Out-sample: {:.2}%",
            window + 1, in_sample_profit, out_sample_profit);
    }

    results
}

fn main() {
    // Simulated price data (sine wave with trend and noise)
    let prices: Vec<f64> = (0..500)
        .map(|i| {
            let trend = 100.0 + i as f64 * 0.1;
            let cycle = 5.0 * (i as f64 * 0.1).sin();
            let noise = (i as f64 * 7.0).sin() * 2.0;
            trend + cycle + noise
        })
        .collect();

    println!("Running Walk-Forward Analysis...\n");

    let results = walk_forward_analysis(
        &prices,
        200, // in-sample: 200 periods
        50,  // out-of-sample: 50 periods
    );

    // Calculate statistics
    let total_windows = results.len() as f64;
    let avg_in_sample: f64 = results.iter()
        .map(|r| r.in_sample_profit)
        .sum::<f64>() / total_windows;
    let avg_out_sample: f64 = results.iter()
        .map(|r| r.out_sample_profit)
        .sum::<f64>() / total_windows;

    let profitable_windows = results.iter()
        .filter(|r| r.out_sample_profit > 0.0)
        .count();

    println!("\n=== Walk-Forward Analysis Summary ===");
    println!("Total windows: {}", results.len());
    println!("Average in-sample profit: {:.2}%", avg_in_sample);
    println!("Average out-of-sample profit: {:.2}%", avg_out_sample);
    println!("Profitable windows: {}/{} ({:.1}%)",
        profitable_windows,
        results.len(),
        profitable_windows as f64 / total_windows * 100.0
    );

    // Efficiency ratio: out-of-sample / in-sample
    let efficiency = (avg_out_sample / avg_in_sample) * 100.0;
    println!("Efficiency ratio: {:.1}%", efficiency);

    if efficiency > 50.0 {
        println!("\n✓ Strategy appears robust (efficiency > 50%)");
    } else {
        println!("\n✗ Strategy may be overfitted (efficiency < 50%)");
    }
}
```

## Advanced: Anchored Walk-Forward

In **anchored walk-forward**, the in-sample period grows over time instead of sliding:

```
Anchored:   [====Train====][Test]
            [======Train======][Test]
            [=========Train=========][Test]

vs.

Sliding:    [====Train====][Test]
                [====Train====][Test]
                    [====Train====][Test]
```

```rust
fn anchored_walk_forward(
    prices: &[f64],
    initial_in_sample: usize,
    out_sample_size: usize,
) -> Vec<WalkForwardResult> {
    let mut results = Vec::new();
    let num_windows = (prices.len() - initial_in_sample) / out_sample_size;

    for window in 0..num_windows {
        let in_sample_end = initial_in_sample + window * out_sample_size;
        let out_sample_end = in_sample_end + out_sample_size;

        if out_sample_end > prices.len() {
            break;
        }

        // In-sample grows from beginning
        let in_sample_data = &prices[0..in_sample_end];
        let (optimized_params, in_sample_profit) = optimize_parameters(in_sample_data);

        // Out-of-sample testing
        let out_sample_data = &prices[in_sample_end..out_sample_end];
        let out_sample_profit = backtest_strategy(out_sample_data, &optimized_params);

        results.push(WalkForwardResult {
            window: window + 1,
            in_sample_profit,
            out_sample_profit,
            optimized_params,
        });
    }

    results
}
```

## Realistic Trading Engine with Walk-Forward

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    entry_time: usize,
}

#[derive(Debug, Clone)]
struct MarketData {
    timestamp: usize,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Debug)]
struct TradingEngine {
    cash: f64,
    position: Option<Position>,
    trades: Vec<Trade>,
}

impl TradingEngine {
    fn new(initial_cash: f64) -> Self {
        TradingEngine {
            cash: initial_cash,
            position: None,
            trades: Vec::new(),
        }
    }

    fn can_buy(&self, price: f64, quantity: f64) -> bool {
        self.position.is_none() && self.cash >= price * quantity
    }

    fn buy(&mut self, symbol: &str, price: f64, quantity: f64, time: usize) -> Result<(), String> {
        if !self.can_buy(price, quantity) {
            return Err("Cannot buy: insufficient funds or position exists".to_string());
        }

        let cost = price * quantity;
        self.cash -= cost;
        self.position = Some(Position {
            symbol: symbol.to_string(),
            entry_price: price,
            quantity,
            entry_time: time,
        });

        Ok(())
    }

    fn sell(&mut self, price: f64, time: usize) -> Result<f64, String> {
        let position = self.position.take()
            .ok_or("No position to sell".to_string())?;

        let revenue = price * position.quantity;
        self.cash += revenue;

        let profit = revenue - (position.entry_price * position.quantity);
        let profit_pct = profit / (position.entry_price * position.quantity) * 100.0;

        self.trades.push(Trade {
            entry_price: position.entry_price,
            exit_price: price,
            profit: profit_pct,
        });

        Ok(profit_pct)
    }

    fn get_equity(&self, current_price: f64) -> f64 {
        let mut equity = self.cash;
        if let Some(pos) = &self.position {
            equity += pos.quantity * current_price;
        }
        equity
    }

    fn get_performance(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.trades.is_empty() {
            return stats;
        }

        let total_profit: f64 = self.trades.iter().map(|t| t.profit).sum();
        let avg_profit = total_profit / self.trades.len() as f64;
        let winning_trades = self.trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = winning_trades as f64 / self.trades.len() as f64 * 100.0;

        stats.insert("total_trades".to_string(), self.trades.len() as f64);
        stats.insert("avg_profit".to_string(), avg_profit);
        stats.insert("win_rate".to_string(), win_rate);
        stats.insert("total_profit".to_string(), total_profit);

        stats
    }
}

fn main() {
    println!("=== Walk-Forward Analysis with Trading Engine ===\n");

    // Generate market data
    let market_data: Vec<MarketData> = (0..500)
        .map(|i| {
            let base = 100.0 + i as f64 * 0.1;
            let volatility = 2.0;
            MarketData {
                timestamp: i,
                open: base + (i as f64 * 3.0).sin() * volatility,
                high: base + (i as f64 * 3.0).sin() * volatility + 0.5,
                low: base + (i as f64 * 3.0).sin() * volatility - 0.5,
                close: base + (i as f64 * 3.0 + 0.5).sin() * volatility,
            }
        })
        .collect();

    let closes: Vec<f64> = market_data.iter().map(|m| m.close).collect();

    // Run walk-forward
    let results = walk_forward_analysis(&closes, 200, 50);

    println!("\n=== Detailed Results ===");
    for result in &results {
        println!("Window {}: Params(fast={}, slow={}, stop={:.2})",
            result.window,
            result.optimized_params.fast_period,
            result.optimized_params.slow_period,
            result.optimized_params.stop_loss
        );
        println!("  In-sample: {:.2}%, Out-sample: {:.2}%",
            result.in_sample_profit,
            result.out_sample_profit
        );
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Walk-Forward Analysis | Rolling optimization and testing through time |
| Overfitting Detection | Poor out-of-sample performance reveals overfitting |
| In-Sample Period | Training data for parameter optimization |
| Out-of-Sample Period | Testing data to validate strategy robustness |
| Efficiency Ratio | Out-of-sample / in-sample performance ratio |
| Anchored WFA | Growing in-sample window vs sliding window |

## Homework

1. **Implement Rolling Walk-Forward**: Create a function that performs walk-forward analysis with a fixed-size rolling window (sliding window approach). Compare results with the anchored approach.

2. **Add More Metrics**: Extend the `WalkForwardResult` to track:
   - Maximum drawdown in each window
   - Sharpe ratio
   - Number of trades
   - Average trade duration

3. **Multi-Strategy Testing**: Implement walk-forward analysis for multiple strategies (e.g., momentum, mean-reversion, breakout) and compare their efficiency ratios.

4. **Optimization Stability**: Track how often parameters change between windows. Calculate a "stability score" that penalizes strategies requiring frequent re-optimization.

## Navigation

[← Previous day](../289-backtesting-framework/en.md) | [Next day →](../291-monte-carlo-simulation/en.md)
