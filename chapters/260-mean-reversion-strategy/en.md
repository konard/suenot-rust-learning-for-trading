# Day 260: Strategy: Mean Reversion

## Trading Analogy

Imagine a rubber band stretched between two points. No matter how far you pull it, it always tries to snap back to its natural resting position. This is exactly how mean reversion works in financial markets. When a stock price moves significantly away from its historical average, there's a tendency for it to "snap back" toward that average over time.

Consider a trader watching Bitcoin's price. If BTC typically trades around $50,000 but suddenly drops to $40,000 due to short-term panic, a mean reversion trader sees an opportunity. They believe the price will eventually return to its "normal" level, so they buy at $40,000 expecting to profit when it reverts to the mean.

In algorithmic trading, mean reversion strategies are popular because:
- They provide clear entry and exit signals based on statistical measures
- They work well in range-bound (non-trending) markets
- They can be backtested easily against historical data
- They combine well with risk management rules

## What is Mean Reversion?

Mean reversion is based on the statistical concept that prices and returns tend to move back toward their average over time. The key components are:

1. **Mean (Average)** — The central value around which prices fluctuate
2. **Standard Deviation** — Measures how far prices typically deviate from the mean
3. **Z-Score** — Shows how many standard deviations the current price is from the mean
4. **Bollinger Bands** — Visual representation of mean and deviation boundaries

## Basic Mean Calculation in Rust

```rust
/// Calculates the simple moving average (SMA) of prices
fn calculate_mean(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let sum: f64 = prices.iter().sum();
    Some(sum / prices.len() as f64)
}

/// Calculates the standard deviation of prices
fn calculate_std_dev(prices: &[f64], mean: f64) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let variance: f64 = prices
        .iter()
        .map(|price| {
            let diff = price - mean;
            diff * diff
        })
        .sum::<f64>() / prices.len() as f64;

    Some(variance.sqrt())
}

/// Calculates the Z-score for a given price
fn calculate_z_score(price: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 {
        return 0.0;
    }
    (price - mean) / std_dev
}

fn main() {
    let btc_prices = vec![
        50000.0, 51000.0, 49500.0, 50500.0, 48000.0,
        47500.0, 49000.0, 50000.0, 51500.0, 52000.0,
        48500.0, 47000.0, 46000.0, 45000.0, 44000.0,
    ];

    let mean = calculate_mean(&btc_prices).unwrap();
    let std_dev = calculate_std_dev(&btc_prices, mean).unwrap();
    let current_price = 44000.0;
    let z_score = calculate_z_score(current_price, mean, std_dev);

    println!("Price Analysis:");
    println!("  Mean: ${:.2}", mean);
    println!("  Std Dev: ${:.2}", std_dev);
    println!("  Current Price: ${:.2}", current_price);
    println!("  Z-Score: {:.2}", z_score);

    if z_score < -2.0 {
        println!("  Signal: STRONG BUY (price significantly below mean)");
    } else if z_score < -1.0 {
        println!("  Signal: BUY (price below mean)");
    } else if z_score > 2.0 {
        println!("  Signal: STRONG SELL (price significantly above mean)");
    } else if z_score > 1.0 {
        println!("  Signal: SELL (price above mean)");
    } else {
        println!("  Signal: HOLD (price near mean)");
    }
}
```

## Complete Mean Reversion Trading System

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    symbol: &'static str,
    quantity: f64,
    entry_price: f64,
    entry_z_score: f64,
}

#[derive(Debug)]
struct MeanReversionStrategy {
    symbol: &'static str,
    lookback_period: usize,
    entry_z_threshold: f64,   // Z-score to enter a trade
    exit_z_threshold: f64,    // Z-score to exit (revert to mean)
    price_history: VecDeque<f64>,
    position: Option<Position>,
    cash: f64,
    total_trades: u32,
    winning_trades: u32,
}

impl MeanReversionStrategy {
    fn new(symbol: &'static str, lookback_period: usize, initial_cash: f64) -> Self {
        MeanReversionStrategy {
            symbol,
            lookback_period,
            entry_z_threshold: 2.0,  // Enter when 2 std devs away from mean
            exit_z_threshold: 0.5,   // Exit when close to mean
            price_history: VecDeque::with_capacity(lookback_period),
            position: None,
            cash: initial_cash,
            total_trades: 0,
            winning_trades: 0,
        }
    }

    fn add_price(&mut self, price: f64) {
        if self.price_history.len() >= self.lookback_period {
            self.price_history.pop_front();
        }
        self.price_history.push_back(price);
    }

    fn calculate_statistics(&self) -> Option<(f64, f64)> {
        if self.price_history.len() < self.lookback_period {
            return None;
        }

        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

        let variance: f64 = prices
            .iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let std_dev = variance.sqrt();

        Some((mean, std_dev))
    }

    fn get_z_score(&self, price: f64) -> Option<f64> {
        self.calculate_statistics().map(|(mean, std_dev)| {
            if std_dev == 0.0 {
                0.0
            } else {
                (price - mean) / std_dev
            }
        })
    }

    fn generate_signal(&self, price: f64) -> Signal {
        match self.get_z_score(price) {
            Some(z) if z <= -self.entry_z_threshold => Signal::StrongBuy,
            Some(z) if z <= -1.0 => Signal::Buy,
            Some(z) if z >= self.entry_z_threshold => Signal::StrongSell,
            Some(z) if z >= 1.0 => Signal::Sell,
            _ => Signal::Hold,
        }
    }

    fn should_exit_position(&self, price: f64) -> bool {
        if let Some(ref position) = self.position {
            if let Some(z_score) = self.get_z_score(price) {
                // Exit when price reverts toward mean
                if position.entry_z_score < 0.0 {
                    // Long position: exit when z-score becomes positive
                    return z_score >= self.exit_z_threshold;
                } else {
                    // Short position: exit when z-score becomes negative
                    return z_score <= -self.exit_z_threshold;
                }
            }
        }
        false
    }

    fn execute_trade(&mut self, price: f64, signal: Signal) -> Option<String> {
        // Check if we should exit existing position
        if self.position.is_some() && self.should_exit_position(price) {
            return self.close_position(price);
        }

        // Open new position if we don't have one
        if self.position.is_none() {
            match signal {
                Signal::StrongBuy => {
                    return self.open_long(price);
                }
                Signal::StrongSell => {
                    return self.open_short(price);
                }
                _ => {}
            }
        }

        None
    }

    fn open_long(&mut self, price: f64) -> Option<String> {
        let quantity = (self.cash * 0.95) / price; // Use 95% of cash
        let z_score = self.get_z_score(price)?;

        self.position = Some(Position {
            symbol: self.symbol,
            quantity,
            entry_price: price,
            entry_z_score: z_score,
        });
        self.cash -= quantity * price;

        Some(format!(
            "LONG {} {:.4} @ ${:.2} (Z-Score: {:.2})",
            self.symbol, quantity, price, z_score
        ))
    }

    fn open_short(&mut self, price: f64) -> Option<String> {
        let quantity = (self.cash * 0.95) / price;
        let z_score = self.get_z_score(price)?;

        // For simplicity, we simulate short by tracking negative quantity
        self.position = Some(Position {
            symbol: self.symbol,
            quantity: -quantity,
            entry_price: price,
            entry_z_score: z_score,
        });
        self.cash += quantity * price; // Receive cash from short sale

        Some(format!(
            "SHORT {} {:.4} @ ${:.2} (Z-Score: {:.2})",
            self.symbol, quantity, price, z_score
        ))
    }

    fn close_position(&mut self, price: f64) -> Option<String> {
        let position = self.position.take()?;
        let z_score = self.get_z_score(price).unwrap_or(0.0);

        let pnl = if position.quantity > 0.0 {
            // Closing long position
            let revenue = position.quantity * price;
            self.cash += revenue;
            revenue - (position.quantity * position.entry_price)
        } else {
            // Closing short position
            let cost = (-position.quantity) * price;
            self.cash -= cost;
            (position.entry_price - price) * (-position.quantity)
        };

        self.total_trades += 1;
        if pnl > 0.0 {
            self.winning_trades += 1;
        }

        Some(format!(
            "CLOSE {} @ ${:.2} | PnL: ${:.2} | Z-Score: {:.2}",
            self.symbol, price, pnl, z_score
        ))
    }

    fn get_portfolio_value(&self, current_price: f64) -> f64 {
        let position_value = match &self.position {
            Some(pos) if pos.quantity > 0.0 => pos.quantity * current_price,
            Some(pos) => {
                // Short position: profit when price falls
                let short_qty = -pos.quantity;
                let initial_value = short_qty * pos.entry_price;
                let current_value = short_qty * current_price;
                initial_value - current_value + self.cash
            }
            None => 0.0,
        };

        if self.position.as_ref().map_or(true, |p| p.quantity > 0.0) {
            self.cash + position_value
        } else {
            position_value
        }
    }

    fn get_win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }
}

fn main() {
    // Simulated BTC price data with mean-reverting characteristics
    let prices = vec![
        50000.0, 50500.0, 51000.0, 51500.0, 52000.0,  // Trending up
        52500.0, 53000.0, 54000.0, 55000.0, 56000.0,  // Overextended
        55000.0, 54000.0, 52000.0, 50000.0, 49000.0,  // Reverting
        48000.0, 47000.0, 46000.0, 45000.0, 44000.0,  // Oversold
        45000.0, 46000.0, 48000.0, 49000.0, 50000.0,  // Reverting back
        50500.0, 51000.0, 50500.0, 50000.0, 49500.0,  // Stabilizing
    ];

    let mut strategy = MeanReversionStrategy::new("BTC", 10, 100_000.0);

    println!("=== Mean Reversion Strategy Backtest ===\n");
    println!("Initial Capital: $100,000.00");
    println!("Lookback Period: 10 periods");
    println!("Entry Z-Score Threshold: +/- 2.0");
    println!("Exit Z-Score Threshold: +/- 0.5\n");
    println!("{:-<60}", "");

    for (day, &price) in prices.iter().enumerate() {
        strategy.add_price(price);

        if let Some(z_score) = strategy.get_z_score(price) {
            let signal = strategy.generate_signal(price);

            print!("Day {:2}: ${:.2} | Z: {:+.2} | ", day + 1, price, z_score);

            if let Some(action) = strategy.execute_trade(price, signal) {
                println!("{}", action);
            } else {
                println!("Signal: {:?}", signal);
            }
        } else {
            println!("Day {:2}: ${:.2} | Collecting data...", day + 1, price);
        }
    }

    let final_price = *prices.last().unwrap();
    println!("{:-<60}", "");
    println!("\n=== Strategy Performance ===");
    println!("Final Portfolio Value: ${:.2}", strategy.get_portfolio_value(final_price));
    println!("Total Trades: {}", strategy.total_trades);
    println!("Win Rate: {:.1}%", strategy.get_win_rate());
    println!("Remaining Cash: ${:.2}", strategy.cash);
}
```

## Bollinger Bands Implementation

```rust
#[derive(Debug, Clone)]
struct BollingerBands {
    period: usize,
    num_std_dev: f64,
    prices: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct BandValues {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
    percent_b: f64,
}

impl BollingerBands {
    fn new(period: usize, num_std_dev: f64) -> Self {
        BollingerBands {
            period,
            num_std_dev,
            prices: Vec::new(),
        }
    }

    fn add_price(&mut self, price: f64) {
        self.prices.push(price);
    }

    fn calculate(&self) -> Option<BandValues> {
        if self.prices.len() < self.period {
            return None;
        }

        let recent_prices: Vec<f64> = self.prices
            .iter()
            .rev()
            .take(self.period)
            .copied()
            .collect();

        let middle = recent_prices.iter().sum::<f64>() / self.period as f64;

        let variance: f64 = recent_prices
            .iter()
            .map(|p| (p - middle).powi(2))
            .sum::<f64>() / self.period as f64;

        let std_dev = variance.sqrt();
        let band_width = self.num_std_dev * std_dev;

        let upper = middle + band_width;
        let lower = middle - band_width;

        let current_price = *self.prices.last().unwrap();
        let bandwidth = (upper - lower) / middle * 100.0;
        let percent_b = (current_price - lower) / (upper - lower);

        Some(BandValues {
            upper,
            middle,
            lower,
            bandwidth,
            percent_b,
        })
    }

    fn get_signal(&self) -> Option<&'static str> {
        let bands = self.calculate()?;
        let current_price = *self.prices.last()?;

        if current_price <= bands.lower {
            Some("BUY - Price at lower band")
        } else if current_price >= bands.upper {
            Some("SELL - Price at upper band")
        } else if bands.percent_b < 0.2 {
            Some("Consider BUY - Near lower band")
        } else if bands.percent_b > 0.8 {
            Some("Consider SELL - Near upper band")
        } else {
            Some("HOLD - Price within bands")
        }
    }
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,
        104.0, 106.0, 108.0, 107.0, 105.0,
        103.0, 101.0, 99.0, 97.0, 95.0,
        96.0, 98.0, 100.0, 102.0, 104.0,
    ];

    let mut bb = BollingerBands::new(10, 2.0);

    println!("=== Bollinger Bands Analysis ===\n");

    for (day, &price) in prices.iter().enumerate() {
        bb.add_price(price);

        if let Some(bands) = bb.calculate() {
            println!("Day {}: Price ${:.2}", day + 1, price);
            println!("  Upper Band:  ${:.2}", bands.upper);
            println!("  Middle Band: ${:.2}", bands.middle);
            println!("  Lower Band:  ${:.2}", bands.lower);
            println!("  Bandwidth:   {:.2}%", bands.bandwidth);
            println!("  %B:          {:.2}", bands.percent_b);
            if let Some(signal) = bb.get_signal() {
                println!("  Signal:      {}", signal);
            }
            println!();
        }
    }
}
```

## Risk Management for Mean Reversion

```rust
use std::collections::VecDeque;

#[derive(Debug)]
struct RiskManagedMeanReversion {
    symbol: String,
    lookback_period: usize,
    max_position_size: f64,      // Maximum position as % of portfolio
    stop_loss_pct: f64,          // Stop loss percentage
    take_profit_pct: f64,        // Take profit percentage
    max_drawdown_pct: f64,       // Maximum allowed drawdown
    price_history: VecDeque<f64>,
    entry_price: Option<f64>,
    peak_portfolio_value: f64,
    current_portfolio_value: f64,
    is_trading_halted: bool,
}

impl RiskManagedMeanReversion {
    fn new(symbol: &str, initial_capital: f64) -> Self {
        RiskManagedMeanReversion {
            symbol: symbol.to_string(),
            lookback_period: 20,
            max_position_size: 0.25,      // 25% max position
            stop_loss_pct: 0.05,          // 5% stop loss
            take_profit_pct: 0.10,        // 10% take profit
            max_drawdown_pct: 0.15,       // 15% max drawdown
            price_history: VecDeque::with_capacity(20),
            entry_price: None,
            peak_portfolio_value: initial_capital,
            current_portfolio_value: initial_capital,
            is_trading_halted: false,
        }
    }

    fn update_portfolio_value(&mut self, new_value: f64) {
        self.current_portfolio_value = new_value;
        if new_value > self.peak_portfolio_value {
            self.peak_portfolio_value = new_value;
        }

        // Check drawdown
        let drawdown = (self.peak_portfolio_value - self.current_portfolio_value)
            / self.peak_portfolio_value;

        if drawdown >= self.max_drawdown_pct {
            self.is_trading_halted = true;
            println!("WARNING: Trading halted due to {:.1}% drawdown!", drawdown * 100.0);
        }
    }

    fn calculate_position_size(&self, entry_price: f64) -> f64 {
        if self.is_trading_halted {
            return 0.0;
        }

        let max_investment = self.current_portfolio_value * self.max_position_size;
        let position_size = max_investment / entry_price;

        position_size
    }

    fn check_stop_loss(&self, current_price: f64) -> bool {
        if let Some(entry) = self.entry_price {
            let loss_pct = (entry - current_price) / entry;
            return loss_pct >= self.stop_loss_pct;
        }
        false
    }

    fn check_take_profit(&self, current_price: f64) -> bool {
        if let Some(entry) = self.entry_price {
            let profit_pct = (current_price - entry) / entry;
            return profit_pct >= self.take_profit_pct;
        }
        false
    }

    fn get_risk_metrics(&self) -> String {
        let drawdown = (self.peak_portfolio_value - self.current_portfolio_value)
            / self.peak_portfolio_value * 100.0;

        format!(
            "Portfolio: ${:.2} | Peak: ${:.2} | Drawdown: {:.2}% | Status: {}",
            self.current_portfolio_value,
            self.peak_portfolio_value,
            drawdown,
            if self.is_trading_halted { "HALTED" } else { "ACTIVE" }
        )
    }
}

fn main() {
    let mut strategy = RiskManagedMeanReversion::new("ETH", 50_000.0);

    println!("=== Risk-Managed Mean Reversion ===\n");
    println!("Max Position Size: {:.0}%", strategy.max_position_size * 100.0);
    println!("Stop Loss: {:.0}%", strategy.stop_loss_pct * 100.0);
    println!("Take Profit: {:.0}%", strategy.take_profit_pct * 100.0);
    println!("Max Drawdown: {:.0}%\n", strategy.max_drawdown_pct * 100.0);

    // Simulate trading scenario
    strategy.entry_price = Some(2000.0);
    let position_size = strategy.calculate_position_size(2000.0);
    println!("Entry at $2000.00");
    println!("Position size: {:.4} ETH (${:.2})\n",
        position_size, position_size * 2000.0);

    // Simulate price movements
    let prices = vec![2000.0, 1980.0, 1950.0, 1900.0, 1850.0, 2100.0, 2200.0];

    for price in prices {
        let position_value = position_size * price;
        strategy.update_portfolio_value(50_000.0 - (position_size * 2000.0) + position_value);

        println!("Price: ${:.2}", price);
        println!("  {}", strategy.get_risk_metrics());

        if strategy.check_stop_loss(price) {
            println!("  ACTION: Stop loss triggered!");
        }
        if strategy.check_take_profit(price) {
            println!("  ACTION: Take profit triggered!");
        }
        println!();
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Mean Reversion | Strategy based on prices returning to historical average |
| Z-Score | Measures distance from mean in standard deviations |
| Bollinger Bands | Visual bands showing mean and volatility boundaries |
| Standard Deviation | Statistical measure of price dispersion |
| Entry/Exit Signals | Rules for opening and closing positions |
| Risk Management | Stop-loss, take-profit, and position sizing rules |

## Homework

1. **Enhanced Z-Score Strategy**: Modify the mean reversion strategy to use an exponential moving average (EMA) instead of simple moving average. Compare the performance of both approaches with historical data.

2. **Multi-Asset Mean Reversion**: Implement a strategy that trades pairs of correlated assets (e.g., BTC and ETH). When the spread between them deviates from the historical mean, open positions expecting convergence.

3. **Dynamic Thresholds**: Create a system that automatically adjusts entry and exit Z-score thresholds based on recent market volatility. Higher volatility should require larger deviations before triggering trades.

4. **Backtest Framework**: Build a complete backtesting framework that:
   - Reads historical price data from a file
   - Runs the mean reversion strategy
   - Calculates performance metrics (Sharpe ratio, max drawdown, win rate)
   - Generates a report with trade-by-trade analysis

## Navigation

[← Previous day](../259-momentum-strategy/en.md) | [Next day →](../261-arbitrage-strategy/en.md)
