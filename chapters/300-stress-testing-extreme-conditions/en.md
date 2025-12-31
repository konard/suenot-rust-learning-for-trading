# Day 300: Stress Testing: Extreme Conditions

## Trading Analogy

Imagine a trader who developed an excellent strategy for normal market conditions — it consistently generates 2-3% per month. But then a "black swan" arrives: a sudden market crash of 40% in one day, like in March 2020.

What happens to the strategy?
- Stop-losses don't trigger due to gaps
- Liquidity evaporates — impossible to close positions
- Leverage leads to margin call
- All capital lost in one day

This is a classic problem: **the strategy wasn't tested under extreme conditions**. It worked in "sunny weather" but wasn't prepared for a storm.

**Stress testing** is checking a trading system's resilience to extreme market events:
- Crashes and sharp price spikes
- Periods of zero liquidity
- Exchange technical failures
- Flash crashes
- Series of losing trades

## Why do we need stress testing?

Backtesting typically tests "normal" conditions — average volatility, typical trading volumes. But real markets periodically go crazy:

| Event | Date | What Happened |
|-------|------|---------------|
| Black Monday | 10/19/1987 | Dow Jones fell 22% in one day |
| Flash Crash | 05/06/2010 | Indices crashed 9% in 5 minutes |
| Swiss Franc Shock | 01/15/2015 | CHF rose 30% in minutes |
| COVID Crash | 03/12/2020 | S&P 500 fell 12% in one day |
| GameStop Short Squeeze | 01/28/2021 | GME rose 1900% in 2 weeks |

Stress testing answers questions:
- Will the strategy survive a 30% market crash?
- What if volatility increases 10x?
- How many consecutive losing trades can occur?
- What's the maximum possible loss?

## Types of Stress Tests

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct MarketConditions {
    volatility_multiplier: f64,  // Volatility multiplier (1.0 = normal, 5.0 = stress)
    liquidity_reduction: f64,    // Liquidity reduction (0.0 = normal, 0.9 = 90% drop)
    gap_probability: f64,        // Probability of gaps (0.0 - 1.0)
    max_drawdown: f64,           // Maximum market drawdown
}

impl MarketConditions {
    fn normal() -> Self {
        Self {
            volatility_multiplier: 1.0,
            liquidity_reduction: 0.0,
            gap_probability: 0.01,
            max_drawdown: 0.20,
        }
    }

    fn stress_crash() -> Self {
        Self {
            volatility_multiplier: 10.0,
            liquidity_reduction: 0.8,
            gap_probability: 0.3,
            max_drawdown: 0.50,
        }
    }

    fn stress_low_volatility() -> Self {
        Self {
            volatility_multiplier: 0.1,
            liquidity_reduction: 0.5,
            gap_probability: 0.0,
            max_drawdown: 0.05,
        }
    }

    fn stress_flash_crash() -> Self {
        Self {
            volatility_multiplier: 20.0,
            liquidity_reduction: 0.95,
            gap_probability: 0.5,
            max_drawdown: 0.30,
        }
    }
}

#[derive(Debug)]
struct StressTestResult {
    scenario: String,
    max_loss: f64,
    max_drawdown: f64,
    trades_executed: usize,
    trades_slipped: usize,
    final_balance: f64,
    survived: bool,
}

impl StressTestResult {
    fn print(&self) {
        println!("=== Scenario: {} ===", self.scenario);
        println!("Maximum loss: {:.2}%", self.max_loss * 100.0);
        println!("Maximum drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("Trades executed: {}", self.trades_executed);
        println!("Trades with slippage: {}", self.trades_slipped);
        println!("Final balance: ${:.2}", self.final_balance);

        if self.survived {
            println!("✅ Strategy survived");
        } else {
            println!("❌ Margin call / Total capital loss");
        }
    }
}

fn main() {
    println!("=== Trading Strategy Stress Testing ===\n");

    let scenarios = vec![
        ("Normal conditions", MarketConditions::normal()),
        ("Market crash", MarketConditions::stress_crash()),
        ("Low volatility", MarketConditions::stress_low_volatility()),
        ("Flash crash", MarketConditions::stress_flash_crash()),
    ];

    for (name, conditions) in scenarios {
        println!("Conditions: {:?}", conditions);
        println!();
    }
}
```

## Example 1: Simulating Extreme Prices

```rust
#[derive(Debug, Clone)]
struct PriceBar {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Price generator with extreme conditions
struct StressDataGenerator {
    base_price: f64,
    base_volatility: f64,
}

impl StressDataGenerator {
    fn new(base_price: f64) -> Self {
        Self {
            base_price,
            base_volatility: 0.02, // 2% normal volatility
        }
    }

    /// Generate normal prices
    fn generate_normal(&self, bars: usize) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            let change = self.deterministic_noise(i) * self.base_volatility;
            price *= 1.0 + change;

            let bar = PriceBar {
                timestamp: i as i64,
                open: price,
                high: price * 1.005,
                low: price * 0.995,
                close: price,
                volume: 1000000.0,
            };

            data.push(bar);
        }

        data
    }

    /// Generate prices with crash (sharp drop)
    fn generate_crash(&self, bars: usize, crash_at: usize, crash_magnitude: f64) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            // At crash moment — sharp drop
            if i == crash_at {
                price *= 1.0 - crash_magnitude; // Drop by crash_magnitude %
            } else {
                let change = self.deterministic_noise(i) * self.base_volatility;
                price *= 1.0 + change;
            }

            // At crash moment — large gap between open and close
            let (open, close) = if i == crash_at {
                let prev_close = if i > 0 { data[i-1].close } else { price };
                (prev_close, price) // Gap down
            } else {
                (price, price)
            };

            let bar = PriceBar {
                timestamp: i as i64,
                open,
                high: open.max(close) * 1.002,
                low: open.min(close) * 0.998,
                close,
                volume: if i == crash_at { 10000000.0 } else { 1000000.0 },
            };

            data.push(bar);
        }

        data
    }

    /// Generate prices with flash crash (drop and quick recovery)
    fn generate_flash_crash(&self, bars: usize, crash_at: usize) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            if i == crash_at {
                // Sharp 20% drop
                price *= 0.80;
            } else if i == crash_at + 1 {
                // Quick 15% recovery
                price *= 1.15;
            } else {
                let change = self.deterministic_noise(i) * self.base_volatility;
                price *= 1.0 + change;
            }

            let bar = PriceBar {
                timestamp: i as i64,
                open: price,
                high: price * 1.005,
                low: price * 0.995,
                close: price,
                volume: if i == crash_at || i == crash_at + 1 { 50000000.0 } else { 1000000.0 },
            };

            data.push(bar);
        }

        data
    }

    /// Deterministic noise (for reproducibility)
    fn deterministic_noise(&self, seed: usize) -> f64 {
        let x = ((seed * 1103515245 + 12345) % 2147483648) as f64;
        (x / 2147483648.0) * 2.0 - 1.0 // Range [-1, 1]
    }
}

fn main() {
    let generator = StressDataGenerator::new(50000.0);

    println!("=== Generating Extreme Market Data ===\n");

    // Scenario 1: Normal conditions
    let normal_data = generator.generate_normal(100);
    println!("1. Normal conditions:");
    println!("   Starting price: ${:.2}", normal_data.first().unwrap().close);
    println!("   Ending price: ${:.2}", normal_data.last().unwrap().close);
    println!();

    // Scenario 2: 40% crash
    let crash_data = generator.generate_crash(100, 50, 0.40);
    println!("2. 40% crash (bar 50):");
    println!("   Price before crash: ${:.2}", crash_data[49].close);
    println!("   Price after crash: ${:.2}", crash_data[50].close);
    println!("   Drop: {:.2}%", (crash_data[49].close - crash_data[50].close) / crash_data[49].close * 100.0);
    println!();

    // Scenario 3: Flash crash
    let flash_data = generator.generate_flash_crash(100, 50);
    println!("3. Flash crash (bar 50):");
    println!("   Price before: ${:.2}", flash_data[49].close);
    println!("   Price at crash: ${:.2}", flash_data[50].close);
    println!("   Price after recovery: ${:.2}", flash_data[51].close);
    println!();
}
```

## Example 2: Testing Stop-Losses on Gaps

```rust
#[derive(Debug, Clone)]
struct Position {
    entry_price: f64,
    quantity: f64,
    stop_loss: f64,
    take_profit: f64,
}

impl Position {
    fn new(entry_price: f64, quantity: f64, stop_loss_pct: f64, take_profit_pct: f64) -> Self {
        Self {
            entry_price,
            quantity,
            stop_loss: entry_price * (1.0 - stop_loss_pct),
            take_profit: entry_price * (1.0 + take_profit_pct),
        }
    }

    /// Check position exit (accounting for gaps)
    fn check_exit(&self, bar: &PriceBar) -> Option<(f64, &str)> {
        // If opening gaps below stop-loss — exit at open, not at stop_loss
        if bar.open < self.stop_loss {
            return Some((bar.open, "Gap Stop-Loss"));
        }

        // If low reached stop-loss — exit at stop-loss
        if bar.low <= self.stop_loss {
            return Some((self.stop_loss, "Stop-Loss"));
        }

        // If opening gaps above take-profit
        if bar.open > self.take_profit {
            return Some((bar.open, "Gap Take-Profit"));
        }

        // If high reached take-profit
        if bar.high >= self.take_profit {
            return Some((self.take_profit, "Take-Profit"));
        }

        None
    }

    fn calculate_pnl(&self, exit_price: f64) -> f64 {
        (exit_price - self.entry_price) * self.quantity
    }
}

fn test_stop_loss_on_gaps() {
    let generator = StressDataGenerator::new(50000.0);

    println!("=== Testing Stop-Losses on Gaps ===\n");

    // Test 1: Normal conditions
    println!("1. Normal conditions (no gaps):");
    let normal_data = generator.generate_normal(100);
    let position = Position::new(50000.0, 1.0, 0.05, 0.10);

    for (i, bar) in normal_data.iter().enumerate() {
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            println!("   Close on bar {}: {} at price ${:.2}", i, reason, exit_price);
            println!("   P&L: ${:.2}", pnl);
            break;
        }
    }
    println!();

    // Test 2: Crash with gap
    println!("2. Crash with gap (40% drop):");
    let crash_data = generator.generate_crash(100, 10, 0.40);
    let position = Position::new(crash_data[5].close, 1.0, 0.05, 0.10);

    println!("   Entry price: ${:.2}", position.entry_price);
    println!("   Stop-loss set at: ${:.2}", position.stop_loss);

    for (i, bar) in crash_data.iter().skip(6).enumerate() {
        let actual_i = i + 6;
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            let expected_loss = position.entry_price * 0.05;
            let actual_loss = -pnl;

            println!("   Close on bar {}: {}", actual_i, reason);
            println!("   Exit price: ${:.2}", exit_price);
            println!("   Expected loss: ${:.2} (5%)", expected_loss);
            println!("   Actual loss: ${:.2} ({:.1}%)", actual_loss, actual_loss / position.entry_price * 100.0);

            if actual_loss > expected_loss * 1.5 {
                println!("   ⚠️ WARNING: Loss is {:.1}x larger than expected due to gap!",
                         actual_loss / expected_loss);
            }
            break;
        }
    }
    println!();

    // Test 3: Flash crash
    println!("3. Flash crash:");
    let flash_data = generator.generate_flash_crash(100, 10);
    let position = Position::new(flash_data[5].close, 1.0, 0.05, 0.10);

    println!("   Entry price: ${:.2}", position.entry_price);

    for (i, bar) in flash_data.iter().skip(6).enumerate() {
        let actual_i = i + 6;
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            println!("   Close on bar {}: {} at price ${:.2}", actual_i, reason, exit_price);
            println!("   P&L: ${:.2}", pnl);
            break;
        }
    }
}

fn main() {
    test_stop_loss_on_gaps();
}
```

## Example 3: Maximum Drawdown and Losing Streaks

```rust
#[derive(Debug)]
struct TradingAccount {
    initial_balance: f64,
    current_balance: f64,
    peak_balance: f64,
    max_drawdown: f64,
    consecutive_losses: usize,
    max_consecutive_losses: usize,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            current_balance: initial_balance,
            peak_balance: initial_balance,
            max_drawdown: 0.0,
            consecutive_losses: 0,
            max_consecutive_losses: 0,
        }
    }

    fn execute_trade(&mut self, pnl: f64) {
        self.current_balance += pnl;

        // Update peak
        if self.current_balance > self.peak_balance {
            self.peak_balance = self.current_balance;
            self.consecutive_losses = 0; // Reset losing streak
        }

        // Calculate current drawdown
        let current_drawdown = (self.peak_balance - self.current_balance) / self.peak_balance;

        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
        }

        // Track losing streaks
        if pnl < 0.0 {
            self.consecutive_losses += 1;
            if self.consecutive_losses > self.max_consecutive_losses {
                self.max_consecutive_losses = self.consecutive_losses;
            }
        } else {
            self.consecutive_losses = 0;
        }
    }

    fn is_margin_call(&self, margin_call_level: f64) -> bool {
        let drawdown = (self.initial_balance - self.current_balance) / self.initial_balance;
        drawdown >= margin_call_level
    }

    fn print_stats(&self) {
        println!("=== Account Statistics ===");
        println!("Initial balance: ${:.2}", self.initial_balance);
        println!("Current balance: ${:.2}", self.current_balance);
        println!("Peak balance: ${:.2}", self.peak_balance);
        println!("Maximum drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("Maximum consecutive losses: {}", self.max_consecutive_losses);
        println!("P&L: ${:.2} ({:.2}%)",
                 self.current_balance - self.initial_balance,
                 (self.current_balance - self.initial_balance) / self.initial_balance * 100.0);
    }
}

/// Simulate trading under different conditions
fn simulate_extreme_losing_streak(num_trades: usize, win_rate: f64, avg_win: f64, avg_loss: f64) -> TradingAccount {
    let mut account = TradingAccount::new(10000.0);

    for i in 0..num_trades {
        // Deterministic "randomness" for reproducibility
        let rand = ((i * 1103515245 + 12345) % 100) as f64 / 100.0;

        let pnl = if rand < win_rate {
            avg_win
        } else {
            -avg_loss
        };

        account.execute_trade(pnl);

        if account.is_margin_call(0.50) {
            println!("⚠️ Margin call on trade {}", i + 1);
            break;
        }
    }

    account
}

fn main() {
    println!("=== Stress Test: Maximum Losing Streaks ===\n");

    // Scenario 1: Normal strategy (60% win rate)
    println!("1. Normal strategy (60% win rate, 1:1.5 risk/reward):");
    let normal = simulate_extreme_losing_streak(1000, 0.60, 150.0, 100.0);
    normal.print_stats();
    println!();

    // Scenario 2: Bad streak (30% win rate for 200 trades)
    println!("2. Extreme bad streak (30% win rate):");
    let bad_streak = simulate_extreme_losing_streak(200, 0.30, 150.0, 100.0);
    bad_streak.print_stats();
    println!();

    // Scenario 3: Large losses (poor risk management)
    println!("3. Large losses (poor risk management, 50% win rate, but 1:0.5 risk/reward):");
    let bad_risk = simulate_extreme_losing_streak(500, 0.50, 100.0, 200.0);
    bad_risk.print_stats();
    println!();
}
```

## Practical Stress Testing Recommendations

### 1. Historical Scenarios

Test your strategy on real historical events:

```rust
fn test_historical_scenarios(strategy: &Strategy) {
    let scenarios = vec![
        ("Black Monday 1987", -0.22, "1987-10-19"),
        ("Flash Crash 2010", -0.09, "2010-05-06"),
        ("COVID Crash 2020", -0.12, "2020-03-12"),
    ];

    for (name, drop, date) in scenarios {
        println!("Testing: {}", name);
        // Load data for this date and run strategy
        // test_strategy_on_date(strategy, date, drop);
    }
}
```

### 2. Synthetic Extremes

Create artificial but plausible extreme conditions:

- Volatility × 10
- Trading volume × 0.1 (low liquidity)
- Series of 20 consecutive losing trades
- 15% gap between bars

### 3. Limits Testing

Check boundary values:

```rust
fn test_position_limits() {
    // What if position size = 0?
    // What if position size = MAX?
    // What if balance = 0?
    // What if price = 0.00000001?
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Stress Testing** | Checking strategy resilience to extreme conditions |
| **Flash Crash** | Sharp price drop with quick recovery |
| **Gap Risk** | Risk of gaps when stop-loss won't trigger at set price |
| **Maximum Drawdown** | Maximum drop from peak to trough |
| **Consecutive Losses** | Series of losing trades in a row |
| **Margin Call** | Forced position closure at critical drawdown |
| **Liquidity Stress** | Testing under low liquidity conditions |

## Homework

1. **Stress Test Generator**: Create a market data generator that can simulate:
   - Normal conditions
   - Crash (sharp N% drop)
   - Flash crash (drop and recovery)
   - Low volatility period
   - High volatility period
   - Gaps (random and at critical moments)

2. **Resilience Analyzer**: Write a system that:
   - Takes a trading strategy
   - Runs it through 10+ stress scenarios
   - Calculates maximum drawdown for each
   - Determines minimum capital for survival
   - Generates report with recommendations

3. **Monte Carlo Stress Test**: Implement simulation:
   - Generate 1000 scenarios with varying volatility
   - Run strategy for each scenario
   - Calculate distribution of maximum drawdowns
   - Find 95th percentile (VaR - Value at Risk)
   - Estimate probability of margin call

4. **Gap Risk Analyzer**: Create gap risk analysis tool:
   - Analyze historical gaps (frequency, size)
   - Simulate stop-losses at different gap sizes
   - Calculate actual expected loss vs theoretical
   - Recommendations for position sizing considering gap risk

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
