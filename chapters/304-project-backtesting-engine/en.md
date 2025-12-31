# Day 304: Project: Backtesting Engine

## Trading Analogy

Imagine you're an experienced trader who wants to develop a professional trading strategy. You can't just throw money at random ideas in the live market—that's too risky and expensive. Instead, you need a **backtesting engine**: a system that tests your strategy on historical data to see how it would have performed.

A good backtesting engine is like a flight simulator for pilots:
- Safe environment for testing risky ideas
- Realistic simulation with all costs and slippage
- Detailed performance metrics
- Validation that strategy works on different time periods
- Protection against overfitting

This month we learned all the components of professional backtesting. Now we're putting it all together into a complete, production-ready backtesting engine.

## Project Overview

We're building a comprehensive backtesting engine that includes:

1. **Core Engine**: Strategy execution, order simulation, position tracking
2. **Commission & Slippage**: Realistic cost modeling
3. **Metrics Calculation**: All key performance indicators
4. **Risk Management**: Drawdown tracking, position sizing
5. **Validation**: Walk-forward analysis, out-of-sample testing
6. **Reporting**: Detailed reports with visualizations
7. **Optimization**: Parameter tuning with overfitting protection

## Project Architecture

```
backtesting_engine/
├── src/
│   ├── main.rs              # Entry point & CLI
│   ├── lib.rs               # Public API
│   ├── engine/
│   │   ├── mod.rs           # Core backtesting engine
│   │   ├── broker.rs        # Simulated broker
│   │   └── executor.rs      # Trade execution logic
│   ├── data/
│   │   ├── mod.rs           # Market data handling
│   │   ├── candle.rs        # OHLCV candles
│   │   └── loader.rs        # Data loading from CSV/JSON
│   ├── strategy/
│   │   ├── mod.rs           # Strategy trait
│   │   ├── moving_average.rs # Example MA crossover strategy
│   │   └── mean_reversion.rs # Example mean reversion strategy
│   ├── metrics/
│   │   ├── mod.rs           # Performance metrics
│   │   ├── returns.rs       # Return calculations
│   │   ├── risk.rs          # Risk metrics (Sharpe, drawdown)
│   │   └── trade_stats.rs   # Trade statistics
│   ├── validation/
│   │   ├── mod.rs           # Validation methods
│   │   ├── walk_forward.rs  # Walk-forward analysis
│   │   ├── cross_validation.rs # K-fold cross-validation
│   │   └── monte_carlo.rs   # Monte Carlo simulation
│   ├── optimization/
│   │   ├── mod.rs           # Parameter optimization
│   │   ├── grid_search.rs   # Grid search
│   │   └── genetic.rs       # Genetic algorithm (bonus)
│   ├── report/
│   │   ├── mod.rs           # Report generation
│   │   ├── equity_curve.rs  # Equity curve visualization
│   │   └── html.rs          # HTML report generation
│   └── utils/
│       ├── mod.rs           # Utility functions
│       └── commissions.rs   # Commission models
└── Cargo.toml
```

## Step 1: Data Structures

```rust
// src/data/candle.rs
use serde::{Deserialize, Serialize};

/// OHLCV candle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

// src/engine/mod.rs
use std::collections::HashMap;

/// Trading position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub size: f64,           // Positive = long, negative = short
    pub entry_price: f64,
    pub entry_time: u64,
    pub realized_pnl: f64,   // Closed P&L
    pub unrealized_pnl: f64, // Open P&L
}

impl Position {
    pub fn update_unrealized_pnl(&mut self, current_price: f64) {
        self.unrealized_pnl = (current_price - self.entry_price) * self.size;
    }

    pub fn market_value(&self, current_price: f64) -> f64 {
        self.size * current_price
    }
}

/// Executed trade record
#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub commission: f64,
    pub pnl: f64,            // For closing trades
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

/// Portfolio state at a point in time
#[derive(Debug, Clone)]
pub struct PortfolioSnapshot {
    pub timestamp: u64,
    pub cash: f64,
    pub positions_value: f64,
    pub total_equity: f64,
    pub drawdown: f64,
}
```

## Step 2: Commission Models

```rust
// src/utils/commissions.rs

/// Commission model for realistic cost simulation
#[derive(Debug, Clone)]
pub struct CommissionModel {
    pub taker_fee_percent: f64,
    pub maker_fee_percent: f64,
    pub min_fee: f64,
}

impl CommissionModel {
    pub fn binance_spot() -> Self {
        CommissionModel {
            taker_fee_percent: 0.1,  // 0.1%
            maker_fee_percent: 0.1,
            min_fee: 0.0,
        }
    }

    pub fn zero() -> Self {
        CommissionModel {
            taker_fee_percent: 0.0,
            maker_fee_percent: 0.0,
            min_fee: 0.0,
        }
    }

    pub fn calculate(&self, value: f64, is_maker: bool) -> f64 {
        let fee_percent = if is_maker {
            self.maker_fee_percent
        } else {
            self.taker_fee_percent
        };

        let fee = value * fee_percent / 100.0;
        fee.max(self.min_fee)
    }
}

/// Slippage model
#[derive(Debug, Clone)]
pub struct SlippageModel {
    pub fixed_percent: f64,  // Fixed slippage percentage
    pub volume_impact: f64,  // Impact based on order size vs volume
}

impl SlippageModel {
    pub fn simple(percent: f64) -> Self {
        SlippageModel {
            fixed_percent: percent,
            volume_impact: 0.0,
        }
    }

    pub fn calculate(&self, price: f64, size: f64, volume: f64) -> f64 {
        let fixed = price * self.fixed_percent / 100.0;
        let impact = if volume > 0.0 {
            price * (size / volume) * self.volume_impact
        } else {
            0.0
        };
        fixed + impact
    }
}
```

## Step 3: Strategy Trait

```rust
// src/strategy/mod.rs
use crate::data::Candle;
use crate::engine::Side;

/// Signal from strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Strategy trait that all trading strategies must implement
pub trait Strategy: Send + Sync {
    /// Initialize strategy with parameters
    fn initialize(&mut self, params: HashMap<String, f64>);

    /// Generate signal based on current market data
    /// - candles: Historical price data (oldest first)
    /// - index: Current candle index
    fn generate_signal(&mut self, candles: &[Candle], index: usize) -> Signal;

    /// Get strategy name
    fn name(&self) -> &str;

    /// Get current parameters
    fn parameters(&self) -> HashMap<String, f64>;
}
```

## Step 4: Example Strategy - Moving Average Crossover

```rust
// src/strategy/moving_average.rs
use super::{Signal, Strategy};
use crate::data::Candle;
use std::collections::HashMap;

pub struct MovingAverageCrossover {
    short_period: usize,
    long_period: usize,
    last_signal: Signal,
}

impl MovingAverageCrossover {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        MovingAverageCrossover {
            short_period,
            long_period,
            last_signal: Signal::Hold,
        }
    }

    fn calculate_sma(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period {
            return None;
        }

        let sum: f64 = candles.iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }
}

impl Strategy for MovingAverageCrossover {
    fn initialize(&mut self, params: HashMap<String, f64>) {
        if let Some(&short) = params.get("short_period") {
            self.short_period = short as usize;
        }
        if let Some(&long) = params.get("long_period") {
            self.long_period = long as usize;
        }
    }

    fn generate_signal(&mut self, candles: &[Candle], index: usize) -> Signal {
        let data = &candles[..=index];

        if data.len() < self.long_period {
            return Signal::Hold;
        }

        let short_ma = self.calculate_sma(data, self.short_period);
        let long_ma = self.calculate_sma(data, self.long_period);

        match (short_ma, long_ma) {
            (Some(short), Some(long)) => {
                let signal = if short > long && self.last_signal != Signal::Buy {
                    Signal::Buy
                } else if short < long && self.last_signal != Signal::Sell {
                    Signal::Sell
                } else {
                    Signal::Hold
                };

                if signal != Signal::Hold {
                    self.last_signal = signal;
                }
                signal
            }
            _ => Signal::Hold,
        }
    }

    fn name(&self) -> &str {
        "MA Crossover"
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("short_period".to_string(), self.short_period as f64);
        params.insert("long_period".to_string(), self.long_period as f64);
        params
    }
}
```

## Step 5: Core Backtesting Engine

```rust
// src/engine/mod.rs
use crate::data::Candle;
use crate::strategy::{Signal, Strategy};
use crate::utils::commissions::{CommissionModel, SlippageModel};

pub struct BacktestEngine {
    initial_capital: f64,
    cash: f64,
    positions: HashMap<String, Position>,
    trades: Vec<Trade>,
    equity_curve: Vec<PortfolioSnapshot>,
    commission_model: CommissionModel,
    slippage_model: SlippageModel,
    peak_equity: f64,
    max_drawdown: f64,
}

impl BacktestEngine {
    pub fn new(initial_capital: f64) -> Self {
        BacktestEngine {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
            commission_model: CommissionModel::binance_spot(),
            slippage_model: SlippageModel::simple(0.05), // 0.05% slippage
            peak_equity: initial_capital,
            max_drawdown: 0.0,
        }
    }

    pub fn set_commission_model(&mut self, model: CommissionModel) {
        self.commission_model = model;
    }

    pub fn set_slippage_model(&mut self, model: SlippageModel) {
        self.slippage_model = model;
    }

    pub fn run(
        &mut self,
        symbol: String,
        candles: &[Candle],
        strategy: &mut dyn Strategy,
    ) -> BacktestResult {
        // Reset state
        self.cash = self.initial_capital;
        self.positions.clear();
        self.trades.clear();
        self.equity_curve.clear();
        self.peak_equity = self.initial_capital;
        self.max_drawdown = 0.0;

        // Run through historical data
        for (index, candle) in candles.iter().enumerate() {
            let signal = strategy.generate_signal(candles, index);

            match signal {
                Signal::Buy => self.execute_buy(&symbol, candle),
                Signal::Sell => self.execute_sell(&symbol, candle),
                Signal::Hold => {}
            }

            // Update portfolio snapshot
            self.update_snapshot(candle);
        }

        // Close any remaining positions at the end
        if let Some(last_candle) = candles.last() {
            if self.positions.contains_key(&symbol) {
                self.execute_sell(&symbol, last_candle);
            }
        }

        self.generate_result()
    }

    fn execute_buy(&mut self, symbol: &str, candle: &Candle) {
        // Don't buy if we already have a position
        if self.positions.contains_key(symbol) {
            return;
        }

        let price = candle.close;
        let slippage = self.slippage_model.calculate(price, 1.0, candle.volume);
        let execution_price = price + slippage;

        // Use 95% of available cash (leave some for fees)
        let available = self.cash * 0.95;
        let size = available / execution_price;
        let value = size * execution_price;

        let commission = self.commission_model.calculate(value, false);
        let total_cost = value + commission;

        if total_cost <= self.cash {
            self.cash -= total_cost;

            self.positions.insert(
                symbol.to_string(),
                Position {
                    symbol: symbol.to_string(),
                    size,
                    entry_price: execution_price,
                    entry_time: candle.timestamp,
                    realized_pnl: 0.0,
                    unrealized_pnl: 0.0,
                },
            );

            self.trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: symbol.to_string(),
                side: Side::Buy,
                price: execution_price,
                size,
                commission,
                pnl: 0.0,
            });
        }
    }

    fn execute_sell(&mut self, symbol: &str, candle: &Candle) {
        if let Some(position) = self.positions.remove(symbol) {
            let price = candle.close;
            let slippage = self.slippage_model.calculate(price, position.size, candle.volume);
            let execution_price = price - slippage;

            let value = position.size * execution_price;
            let commission = self.commission_model.calculate(value, false);

            let pnl = (execution_price - position.entry_price) * position.size - commission;

            self.cash += value - commission;

            self.trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: symbol.to_string(),
                side: Side::Sell,
                price: execution_price,
                size: position.size,
                commission,
                pnl,
            });
        }
    }

    fn update_snapshot(&mut self, candle: &Candle) {
        let mut positions_value = 0.0;

        for position in self.positions.values_mut() {
            position.update_unrealized_pnl(candle.close);
            positions_value += position.market_value(candle.close);
        }

        let total_equity = self.cash + positions_value;

        // Update peak and drawdown
        if total_equity > self.peak_equity {
            self.peak_equity = total_equity;
        }

        let current_drawdown = (self.peak_equity - total_equity) / self.peak_equity;
        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
        }

        self.equity_curve.push(PortfolioSnapshot {
            timestamp: candle.timestamp,
            cash: self.cash,
            positions_value,
            total_equity,
            drawdown: current_drawdown,
        });
    }

    fn generate_result(&self) -> BacktestResult {
        let final_equity = self.equity_curve.last()
            .map(|s| s.total_equity)
            .unwrap_or(self.initial_capital);

        let total_return = (final_equity - self.initial_capital) / self.initial_capital;

        BacktestResult {
            initial_capital: self.initial_capital,
            final_equity,
            total_return,
            total_trades: self.trades.len(),
            winning_trades: self.trades.iter().filter(|t| t.pnl > 0.0).count(),
            losing_trades: self.trades.iter().filter(|t| t.pnl < 0.0).count(),
            max_drawdown: self.max_drawdown,
            equity_curve: self.equity_curve.clone(),
            trades: self.trades.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub initial_capital: f64,
    pub final_equity: f64,
    pub total_return: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub max_drawdown: f64,
    pub equity_curve: Vec<PortfolioSnapshot>,
    pub trades: Vec<Trade>,
}
```

## Step 6: Performance Metrics

```rust
// src/metrics/mod.rs
use crate::engine::{BacktestResult, Trade};

pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub avg_trade_duration_hours: f64,
}

impl PerformanceMetrics {
    pub fn calculate(result: &BacktestResult, trading_days: usize) -> Self {
        let total_return = result.total_return;
        let years = trading_days as f64 / 365.0;
        let annualized_return = if years > 0.0 {
            ((1.0 + total_return).powf(1.0 / years)) - 1.0
        } else {
            0.0
        };

        let sharpe_ratio = Self::calculate_sharpe_ratio(&result.equity_curve);
        let (win_rate, profit_factor, avg_win, avg_loss, largest_win, largest_loss) =
            Self::calculate_trade_stats(&result.trades);

        let avg_duration = Self::calculate_avg_duration(&result.trades);

        PerformanceMetrics {
            total_return,
            annualized_return,
            sharpe_ratio,
            max_drawdown: result.max_drawdown,
            win_rate,
            profit_factor,
            average_win: avg_win,
            average_loss: avg_loss,
            largest_win,
            largest_loss,
            avg_trade_duration_hours: avg_duration,
        }
    }

    fn calculate_sharpe_ratio(equity_curve: &[crate::engine::PortfolioSnapshot]) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = equity_curve
            .windows(2)
            .map(|w| (w[1].total_equity - w[0].total_equity) / w[0].total_equity)
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            // Annualized Sharpe (assuming daily data)
            mean / std_dev * (365.0_f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_trade_stats(trades: &[Trade]) -> (f64, f64, f64, f64, f64, f64) {
        let closed_trades: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.side == crate::engine::Side::Sell)
            .collect();

        if closed_trades.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let winning: Vec<f64> = closed_trades
            .iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .collect();

        let losing: Vec<f64> = closed_trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .collect();

        let win_rate = winning.len() as f64 / closed_trades.len() as f64;

        let total_wins: f64 = winning.iter().sum();
        let total_losses: f64 = losing.iter().sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        let avg_win = if !winning.is_empty() {
            winning.iter().sum::<f64>() / winning.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losing.is_empty() {
            losing.iter().sum::<f64>() / losing.len() as f64
        } else {
            0.0
        };

        let largest_win = winning.iter().cloned().fold(0.0, f64::max);
        let largest_loss = losing.iter().cloned().fold(0.0, f64::max);

        (win_rate, profit_factor, avg_win, avg_loss, largest_win, largest_loss)
    }

    fn calculate_avg_duration(trades: &[Trade]) -> f64 {
        if trades.len() < 2 {
            return 0.0;
        }

        let mut total_duration = 0u64;
        let mut count = 0;

        for window in trades.windows(2) {
            if window[0].side == crate::engine::Side::Buy
                && window[1].side == crate::engine::Side::Sell {
                total_duration += window[1].timestamp - window[0].timestamp;
                count += 1;
            }
        }

        if count > 0 {
            (total_duration as f64 / count as f64) / 3600.0 // Convert to hours
        } else {
            0.0
        }
    }

    pub fn print(&self) {
        println!("=== Performance Metrics ===");
        println!("Total Return:        {:.2}%", self.total_return * 100.0);
        println!("Annualized Return:   {:.2}%", self.annualized_return * 100.0);
        println!("Sharpe Ratio:        {:.2}", self.sharpe_ratio);
        println!("Max Drawdown:        {:.2}%", self.max_drawdown * 100.0);
        println!("Win Rate:            {:.2}%", self.win_rate * 100.0);
        println!("Profit Factor:       {:.2}", self.profit_factor);
        println!("Average Win:         ${:.2}", self.average_win);
        println!("Average Loss:        ${:.2}", self.average_loss);
        println!("Largest Win:         ${:.2}", self.largest_win);
        println!("Largest Loss:        ${:.2}", self.largest_loss);
        println!("Avg Trade Duration:  {:.1} hours", self.avg_trade_duration_hours);
    }
}
```

## Step 7: Walk-Forward Validation

```rust
// src/validation/walk_forward.rs
use crate::data::Candle;
use crate::engine::BacktestEngine;
use crate::strategy::Strategy;

pub struct WalkForwardConfig {
    pub train_size: usize,   // Number of candles for training
    pub test_size: usize,    // Number of candles for testing
    pub step_size: usize,    // Step between windows
}

pub struct WalkForwardResult {
    pub windows: Vec<WindowResult>,
    pub average_train_return: f64,
    pub average_test_return: f64,
    pub consistency_score: f64,
}

pub struct WindowResult {
    pub window_id: usize,
    pub train_return: f64,
    pub test_return: f64,
    pub train_sharpe: f64,
    pub test_sharpe: f64,
}

pub fn walk_forward_analysis(
    candles: &[Candle],
    strategy_factory: &dyn Fn() -> Box<dyn Strategy>,
    config: WalkForwardConfig,
    initial_capital: f64,
) -> WalkForwardResult {
    let mut windows = Vec::new();
    let mut position = 0;
    let mut window_id = 0;

    while position + config.train_size + config.test_size <= candles.len() {
        let train_data = &candles[position..position + config.train_size];
        let test_data = &candles[position + config.train_size..
                                 position + config.train_size + config.test_size];

        // Train on training window
        let mut strategy = strategy_factory();
        let mut engine = BacktestEngine::new(initial_capital);
        let train_result = engine.run("BTC".to_string(), train_data, strategy.as_mut());

        // Test on next window
        let mut engine = BacktestEngine::new(initial_capital);
        let test_result = engine.run("BTC".to_string(), test_data, strategy.as_mut());

        let train_metrics = crate::metrics::PerformanceMetrics::calculate(
            &train_result,
            config.train_size
        );
        let test_metrics = crate::metrics::PerformanceMetrics::calculate(
            &test_result,
            config.test_size
        );

        windows.push(WindowResult {
            window_id,
            train_return: train_result.total_return,
            test_return: test_result.total_return,
            train_sharpe: train_metrics.sharpe_ratio,
            test_sharpe: test_metrics.sharpe_ratio,
        });

        position += config.step_size;
        window_id += 1;
    }

    let avg_train = windows.iter().map(|w| w.train_return).sum::<f64>() / windows.len() as f64;
    let avg_test = windows.iter().map(|w| w.test_return).sum::<f64>() / windows.len() as f64;

    // Consistency score: how many windows had positive test returns
    let positive_windows = windows.iter().filter(|w| w.test_return > 0.0).count();
    let consistency = positive_windows as f64 / windows.len() as f64;

    WalkForwardResult {
        windows,
        average_train_return: avg_train,
        average_test_return: avg_test,
        consistency_score: consistency,
    }
}
```

## Step 8: Main Example - Putting It All Together

```rust
// src/main.rs
use backtesting_engine::data::Candle;
use backtesting_engine::engine::BacktestEngine;
use backtesting_engine::strategy::moving_average::MovingAverageCrossover;
use backtesting_engine::metrics::PerformanceMetrics;
use backtesting_engine::validation::walk_forward::*;
use backtesting_engine::utils::commissions::CommissionModel;

fn main() {
    println!("=== Backtesting Engine Demo ===\n");

    // Load sample data (in real project, load from CSV/database)
    let candles = generate_sample_data();
    println!("Loaded {} candles\n", candles.len());

    // Test 1: Simple backtest
    println!("TEST 1: Simple Backtest");
    println!("------------------------");
    run_simple_backtest(&candles);
    println!();

    // Test 2: Walk-forward validation
    println!("TEST 2: Walk-Forward Validation");
    println!("--------------------------------");
    run_walk_forward(&candles);
    println!();

    // Test 3: Parameter comparison
    println!("TEST 3: Parameter Comparison");
    println!("----------------------------");
    compare_parameters(&candles);
}

fn run_simple_backtest(candles: &[Candle]) {
    let mut engine = BacktestEngine::new(10000.0);
    engine.set_commission_model(CommissionModel::binance_spot());

    let mut strategy = MovingAverageCrossover::new(10, 30);
    let result = engine.run("BTC".to_string(), candles, &mut strategy);

    println!("Initial Capital: ${:.2}", result.initial_capital);
    println!("Final Equity:    ${:.2}", result.final_equity);
    println!("Total Return:    {:.2}%", result.total_return * 100.0);
    println!("Total Trades:    {}", result.total_trades);
    println!("Winning Trades:  {}", result.winning_trades);
    println!("Losing Trades:   {}", result.losing_trades);

    let metrics = PerformanceMetrics::calculate(&result, candles.len());
    println!("\nDetailed Metrics:");
    metrics.print();
}

fn run_walk_forward(candles: &[Candle]) {
    let config = WalkForwardConfig {
        train_size: 100,
        test_size: 50,
        step_size: 50,
    };

    let strategy_factory = || -> Box<dyn backtesting_engine::strategy::Strategy> {
        Box::new(MovingAverageCrossover::new(10, 30))
    };

    let wf_result = walk_forward_analysis(candles, &strategy_factory, config, 10000.0);

    println!("Number of windows: {}", wf_result.windows.len());
    println!("Average train return: {:.2}%", wf_result.average_train_return * 100.0);
    println!("Average test return:  {:.2}%", wf_result.average_test_return * 100.0);
    println!("Consistency score:    {:.2}%", wf_result.consistency_score * 100.0);

    println!("\nWindow-by-window:");
    for window in &wf_result.windows {
        println!("  Window {}: Train {:.2}%, Test {:.2}%",
            window.window_id,
            window.train_return * 100.0,
            window.test_return * 100.0
        );
    }
}

fn compare_parameters(candles: &[Candle]) {
    let param_sets = [
        (5, 20),
        (10, 30),
        (20, 50),
        (10, 50),
    ];

    println!("Comparing different MA parameters:\n");
    println!("{:<15} {:<15} {:<15} {:<15}", "Parameters", "Return", "Sharpe", "Max DD");
    println!("{}", "-".repeat(60));

    for (short, long) in param_sets.iter() {
        let mut engine = BacktestEngine::new(10000.0);
        let mut strategy = MovingAverageCrossover::new(*short, *long);
        let result = engine.run("BTC".to_string(), candles, &mut strategy);
        let metrics = PerformanceMetrics::calculate(&result, candles.len());

        println!("{:<15} {:<15.2}% {:<15.2} {:<15.2}%",
            format!("MA({},{})", short, long),
            result.total_return * 100.0,
            metrics.sharpe_ratio,
            result.max_drawdown * 100.0
        );
    }
}

// Generate sample price data for demonstration
fn generate_sample_data() -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut price = 40000.0;
    let base_time = 1609459200; // 2021-01-01

    for i in 0..500 {
        // Simple random walk with trend
        let change = (rand::random::<f64>() - 0.48) * 100.0;
        price += change;
        price = price.max(30000.0).min(60000.0);

        let volatility = price * 0.005;
        let high = price + volatility;
        let low = price - volatility;

        candles.push(Candle {
            timestamp: base_time + (i * 3600), // 1-hour candles
            open: price,
            high,
            low,
            close: price,
            volume: 100.0 + rand::random::<f64>() * 50.0,
        });
    }

    candles
}

// Note: Add rand crate to Cargo.toml:
// [dependencies]
// rand = "0.8"
// serde = { version = "1.0", features = ["derive"] }
```

## What We Learned

This comprehensive project brings together all backtesting concepts:

| Component | Purpose |
|-----------|---------|
| **Data Structures** | OHLCV candles, positions, trades, portfolio snapshots |
| **Commission & Slippage** | Realistic cost modeling for accurate results |
| **Strategy Trait** | Flexible interface for any trading strategy |
| **Backtesting Engine** | Core simulation with order execution |
| **Performance Metrics** | Comprehensive statistics (Sharpe, drawdown, win rate) |
| **Walk-Forward Analysis** | Out-of-sample validation |
| **Position Management** | Entry, exit, P&L tracking |
| **Risk Management** | Drawdown monitoring, position sizing |

## Homework

1. **Add Mean Reversion Strategy**: Implement a mean reversion strategy that:
   - Calculates Bollinger Bands
   - Buys when price touches lower band
   - Sells when price touches upper band
   - Compare performance with MA crossover

2. **Implement K-Fold Cross-Validation**: Add cross-validation module:
   - Split data into K folds
   - Train on K-1 folds, test on remaining
   - Calculate average performance across all folds
   - Detect overfitting by comparing train vs test

3. **Add HTML Report Generator**: Create detailed HTML reports with:
   - Equity curve chart
   - Drawdown chart
   - Trade distribution histogram
   - Monthly returns table
   - Risk/return scatter plot

4. **Parameter Optimization**: Implement grid search optimizer:
   - Test multiple parameter combinations
   - Rank by Sharpe ratio or other metric
   - Apply walk-forward to each combination
   - Find robust parameters across different periods

5. **Risk-Adjusted Position Sizing**: Add position sizing based on:
   - Kelly Criterion
   - Fixed fractional method
   - Volatility-based sizing
   - Maximum drawdown constraint

6. **Add More Metrics**: Implement additional metrics:
   - Sortino ratio (downside deviation)
   - Calmar ratio (return/max drawdown)
   - Omega ratio
   - Win/loss streak analysis
   - Recovery time from drawdowns

## Extension Ideas

For the ambitious:

1. **Multi-Asset Support**: Backtest portfolios with multiple instruments
2. **Short Selling**: Add support for short positions
3. **Leverage**: Implement margin and leverage calculations
4. **Stop Loss / Take Profit**: Automated exit rules
5. **Genetic Algorithm Optimization**: Evolve strategy parameters
6. **Transaction Cost Analysis**: Detailed commission breakdown
7. **Benchmark Comparison**: Compare strategy vs buy-and-hold
8. **Monte Carlo Simulation**: Randomize trade order to test robustness

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md) | [Next day →](../305-*/en.md)
