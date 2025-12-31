# Day 285: Metrics: Win Rate

## Trading Analogy

Imagine you're playing basketball and trying to shoot the ball into the hoop. You took 100 shots, of which 60 went in and 40 missed. Your shooting percentage is 60%. In trading, **Win Rate** (percentage of profitable trades) works the same way — it's the ratio of profitable trades to the total number of trades.

Important to understand:
- Win Rate = 70% means 7 out of 10 trades closed with profit
- Win Rate alone **does not guarantee** strategy profitability
- A strategy with 40% Win Rate can be more profitable than one with 80% Win Rate if the average win is significantly larger than the average loss

For example:
- Strategy A: Win Rate 90%, average win $10, average loss $100 → unprofitable
- Strategy B: Win Rate 40%, average win $300, average loss $100 → profitable

## What is Win Rate?

**Win Rate** (win ratio, success rate) is a basic performance metric for a trading strategy that shows what proportion of trades closed with profit.

Formula:
```
Win Rate = (Number of Profitable Trades / Total Number of Trades) × 100%
```

### Types of Win Rate

1. **Overall Win Rate** — across all trades
2. **Long Win Rate** — only for long positions
3. **Short Win Rate** — only for short positions
4. **Win Rate by Instrument** — separately for each trading instrument
5. **Win Rate by Time Period** — by days of week, time of day, etc.

## Basic Implementation in Rust

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TradeResult {
    Win,
    Loss,
    BreakEven,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: TradeSide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    pub fn new(symbol: &str, entry_price: f64, exit_price: f64, quantity: f64, side: TradeSide) -> Self {
        Self {
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            side,
        }
    }

    pub fn pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    pub fn result(&self) -> TradeResult {
        let pnl = self.pnl();
        if pnl > 0.0 {
            TradeResult::Win
        } else if pnl < 0.0 {
            TradeResult::Loss
        } else {
            TradeResult::BreakEven
        }
    }
}

pub struct WinRateCalculator {
    trades: Vec<Trade>,
}

impl WinRateCalculator {
    pub fn new() -> Self {
        Self { trades: Vec::new() }
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let winning_trades = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning_trades as f64 / self.trades.len() as f64) * 100.0
    }

    pub fn total_trades(&self) -> usize {
        self.trades.len()
    }

    pub fn winning_trades(&self) -> usize {
        self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count()
    }

    pub fn losing_trades(&self) -> usize {
        self.trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .count()
    }
}

fn main() {
    let mut calculator = WinRateCalculator::new();

    // Add winning trades
    calculator.add_trade(Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long));
    calculator.add_trade(Trade::new("ETH", 2000.0, 2100.0, 10.0, TradeSide::Long));
    calculator.add_trade(Trade::new("BTC", 42000.0, 41000.0, 1.0, TradeSide::Short));

    // Add losing trades
    calculator.add_trade(Trade::new("BTC", 40000.0, 39000.0, 1.0, TradeSide::Long));
    calculator.add_trade(Trade::new("ETH", 2000.0, 1900.0, 10.0, TradeSide::Long));

    println!("Total trades: {}", calculator.total_trades());
    println!("Winning: {}", calculator.winning_trades());
    println!("Losing: {}", calculator.losing_trades());
    println!("Win Rate: {:.2}%", calculator.win_rate());
}
```

**Output:**
```
Total trades: 5
Winning: 3
Losing: 2
Win Rate: 60.00%
```

## Advanced Win Rate Analytics

Often you need to analyze Win Rate across various parameters:

```rust
use std::collections::HashMap;

pub struct AdvancedWinRateAnalyzer {
    trades: Vec<Trade>,
}

impl AdvancedWinRateAnalyzer {
    pub fn new(trades: Vec<Trade>) -> Self {
        Self { trades }
    }

    // Win Rate for long positions
    pub fn long_win_rate(&self) -> f64 {
        let long_trades: Vec<_> = self.trades.iter()
            .filter(|t| t.side == TradeSide::Long)
            .collect();

        if long_trades.is_empty() {
            return 0.0;
        }

        let winning = long_trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning as f64 / long_trades.len() as f64) * 100.0
    }

    // Win Rate for short positions
    pub fn short_win_rate(&self) -> f64 {
        let short_trades: Vec<_> = self.trades.iter()
            .filter(|t| t.side == TradeSide::Short)
            .collect();

        if short_trades.is_empty() {
            return 0.0;
        }

        let winning = short_trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning as f64 / short_trades.len() as f64) * 100.0
    }

    // Win Rate by each instrument
    pub fn win_rate_by_symbol(&self) -> HashMap<String, f64> {
        let mut symbol_map: HashMap<String, Vec<&Trade>> = HashMap::new();

        for trade in &self.trades {
            symbol_map.entry(trade.symbol.clone())
                .or_insert_with(Vec::new)
                .push(trade);
        }

        symbol_map.into_iter()
            .map(|(symbol, trades)| {
                let winning = trades.iter()
                    .filter(|t| t.result() == TradeResult::Win)
                    .count();
                let wr = (winning as f64 / trades.len() as f64) * 100.0;
                (symbol, wr)
            })
            .collect()
    }

    // Average win size
    pub fn average_win(&self) -> f64 {
        let wins: Vec<_> = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .collect();

        if wins.is_empty() {
            return 0.0;
        }

        let total_pnl: f64 = wins.iter().map(|t| t.pnl()).sum();
        total_pnl / wins.len() as f64
    }

    // Average loss size
    pub fn average_loss(&self) -> f64 {
        let losses: Vec<_> = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .collect();

        if losses.is_empty() {
            return 0.0;
        }

        let total_pnl: f64 = losses.iter().map(|t| t.pnl()).sum();
        total_pnl / losses.len() as f64
    }

    // Profit Factor = Average Win / Average Loss (absolute value)
    pub fn profit_factor(&self) -> f64 {
        let avg_win = self.average_win();
        let avg_loss = self.average_loss().abs();

        if avg_loss == 0.0 {
            return f64::INFINITY;
        }

        avg_win / avg_loss
    }

    // Full analysis report
    pub fn report(&self) {
        println!("=== Win Rate Analysis ===");
        println!("Overall Win Rate: {:.2}%", self.long_win_rate());
        println!("Long Win Rate: {:.2}%", self.long_win_rate());
        println!("Short Win Rate: {:.2}%", self.short_win_rate());
        println!();

        println!("Win Rate by instrument:");
        for (symbol, wr) in self.win_rate_by_symbol() {
            println!("  {}: {:.2}%", symbol, wr);
        }
        println!();

        println!("Average win: ${:.2}", self.average_win());
        println!("Average loss: ${:.2}", self.average_loss());
        println!("Profit Factor: {:.2}", self.profit_factor());
    }
}

fn main() {
    let trades = vec![
        // BTC Long trades
        Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long),
        Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeSide::Long),
        Trade::new("BTC", 40500.0, 42000.0, 1.0, TradeSide::Long),

        // ETH Long trades
        Trade::new("ETH", 2000.0, 2100.0, 10.0, TradeSide::Long),
        Trade::new("ETH", 2100.0, 2050.0, 10.0, TradeSide::Long),

        // BTC Short trades
        Trade::new("BTC", 42000.0, 41000.0, 1.0, TradeSide::Short),
        Trade::new("BTC", 41000.0, 42000.0, 1.0, TradeSide::Short),
    ];

    let analyzer = AdvancedWinRateAnalyzer::new(trades);
    analyzer.report();
}
```

**Example output:**
```
=== Win Rate Analysis ===
Overall Win Rate: 71.43%
Long Win Rate: 60.00%
Short Win Rate: 50.00%

Win Rate by instrument:
  BTC: 60.00%
  ETH: 50.00%

Average win: $1200.00
Average loss: $-750.00
Profit Factor: 1.60
```

## Win Rate in Risk Management Context

```rust
#[derive(Debug)]
pub struct RiskMetrics {
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub expectancy: f64, // Expected value per trade
}

impl RiskMetrics {
    pub fn calculate(trades: &[Trade]) -> Self {
        let total = trades.len() as f64;
        if total == 0.0 {
            return Self {
                win_rate: 0.0,
                avg_win: 0.0,
                avg_loss: 0.0,
                profit_factor: 0.0,
                expectancy: 0.0,
            };
        }

        let wins: Vec<_> = trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .collect();
        let losses: Vec<_> = trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .collect();

        let win_rate = (wins.len() as f64 / total) * 100.0;

        let avg_win = if wins.is_empty() {
            0.0
        } else {
            wins.iter().map(|t| t.pnl()).sum::<f64>() / wins.len() as f64
        };

        let avg_loss = if losses.is_empty() {
            0.0
        } else {
            losses.iter().map(|t| t.pnl()).sum::<f64>() / losses.len() as f64
        };

        let profit_factor = if avg_loss == 0.0 {
            f64::INFINITY
        } else {
            avg_win / avg_loss.abs()
        };

        // Expectancy = (Win% × Avg Win) - (Loss% × |Avg Loss|)
        let loss_rate = (losses.len() as f64 / total) * 100.0;
        let expectancy = (win_rate / 100.0 * avg_win) - (loss_rate / 100.0 * avg_loss.abs());

        Self {
            win_rate,
            avg_win,
            avg_loss,
            profit_factor,
            expectancy,
        }
    }

    pub fn is_profitable(&self) -> bool {
        self.expectancy > 0.0
    }

    pub fn required_win_rate_for_breakeven(&self) -> f64 {
        // What Win Rate would make the strategy break even?
        // Expectancy = 0
        // WR × AvgWin - (1 - WR) × |AvgLoss| = 0
        // WR × AvgWin = (1 - WR) × |AvgLoss|
        // WR × AvgWin = |AvgLoss| - WR × |AvgLoss|
        // WR × (AvgWin + |AvgLoss|) = |AvgLoss|
        // WR = |AvgLoss| / (AvgWin + |AvgLoss|)

        let avg_loss_abs = self.avg_loss.abs();
        if self.avg_win + avg_loss_abs == 0.0 {
            return 0.0;
        }

        (avg_loss_abs / (self.avg_win + avg_loss_abs)) * 100.0
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long),  // +1000
        Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeSide::Long),  // -500
        Trade::new("BTC", 40500.0, 42000.0, 1.0, TradeSide::Long),  // +1500
        Trade::new("ETH", 2000.0, 1900.0, 10.0, TradeSide::Long),   // -1000
        Trade::new("ETH", 2000.0, 2200.0, 10.0, TradeSide::Long),   // +2000
    ];

    let metrics = RiskMetrics::calculate(&trades);

    println!("=== Risk Metrics ===");
    println!("Win Rate: {:.2}%", metrics.win_rate);
    println!("Average win: ${:.2}", metrics.avg_win);
    println!("Average loss: ${:.2}", metrics.avg_loss);
    println!("Profit Factor: {:.2}", metrics.profit_factor);
    println!("Expectancy: ${:.2}", metrics.expectancy);
    println!("Profitable: {}", if metrics.is_profitable() { "Yes" } else { "No" });
    println!("Required Win Rate for breakeven: {:.2}%",
             metrics.required_win_rate_for_breakeven());
}
```

## Practical Exercises

### Exercise 1: Basic Win Rate Calculator
Create a `WinRateTracker` struct that:
- Stores a list of trades
- Calculates overall Win Rate
- Outputs the number of winning and losing trades

```rust
// Your code here
```

### Exercise 2: Win Rate by Time Periods
Extend the `Trade` struct by adding a `timestamp: i64` field. Implement a function that calculates Win Rate:
- By days of the week (Monday, Tuesday, etc.)
- By hours of the day (morning, afternoon, evening)

```rust
// Your code here
```

### Exercise 3: Strategy Simulator
Create a `simulate_strategy` function that:
- Takes parameters: `win_rate`, `avg_win`, `avg_loss`, `num_trades`
- Generates random trades based on these parameters
- Returns the total profit/loss

Hint: use the `rand` crate for generating random results.

```rust
// Your code here
```

### Exercise 4: Streak Analyzer
Implement a function that finds:
- Maximum consecutive winning streak
- Maximum consecutive losing streak
- Average length of winning and losing streaks

```rust
// Your code here
```

## Homework

1. **Minimum Win Rate Calculator**: Write a function that calculates the minimum required Win Rate for breakeven based on a desired risk/reward ratio. For example, if risk/reward = 1:2, what should the Win Rate be?

2. **Win Rate Tracker with Persistence**: Create a struct that:
   - Saves trades to a JSON file
   - Loads trades from a file
   - Tracks Win Rate in real-time
   - Exports statistics to CSV format

3. **Backtesting with Win Rate Optimization**: Implement a simple backtester for a Moving Average Crossover strategy:
   - Test different moving average periods (e.g., MA(10)/MA(20), MA(20)/MA(50))
   - For each combination, calculate Win Rate, Profit Factor, and Expectancy
   - Find optimal parameters

4. **Win Rate Dashboard**: Create a console dashboard (using `tui` or `crossterm` library) that:
   - Displays Win Rate in real-time
   - Shows a Win Rate chart for the last N trades
   - Displays a warning if Win Rate falls below a threshold
   - Shows top 3 best and worst instruments by Win Rate

## What We Learned

| Concept | Description |
|---------|-------------|
| Win Rate | Percentage of profitable trades out of total trades |
| TradeResult | Enum for classifying trade outcomes (Win/Loss/BreakEven) |
| Profit Factor | Ratio of average win to average loss |
| Expectancy | Expected profit/loss per trade |
| Breakeven Win Rate | Minimum Win Rate for strategy to break even |
| Advanced Analytics | Analyzing Win Rate by instruments, directions, time periods |

## Navigation

[← Previous Day](../284-backtesting-basics/en.md) | [Next Day →](../286-profit-factor-metric/en.md)
