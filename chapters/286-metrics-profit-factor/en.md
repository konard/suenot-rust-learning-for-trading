# Day 286: Metrics: Profit Factor

## Trading Analogy

Imagine you're analyzing your trading journal for the year. You won on 60 trades, earning a total of $15,000. You lost on 40 trades, losing a total of $5,000. How do you evaluate your performance? One simple metric traders use is **Profit Factor** — it tells you how many dollars you earn for every dollar you lose.

In this example:
- Total profits: $15,000
- Total losses: $5,000
- Profit Factor = $15,000 / $5,000 = 3.0

This means for every dollar you lost, you earned three dollars! A Profit Factor above 1.0 means you're profitable. The higher the number, the better your strategy performs — but beware of unrealistically high values (above 4.0) which might indicate overfitting in backtesting.

## What is Profit Factor?

**Profit Factor** is one of the most important metrics in backtesting trading strategies. It's a simple ratio that answers the question: "How much profit do I make relative to my losses?"

### Formula

```
Profit Factor = Total Gross Profit / Total Gross Loss
```

Where:
- **Total Gross Profit** = Sum of all winning trades
- **Total Gross Loss** = Sum of all losing trades (absolute value)

### Interpretation

| Profit Factor | Meaning |
|---------------|---------|
| < 1.0 | Losing strategy (losses exceed profits) |
| = 1.0 | Break-even (profits equal losses) |
| 1.0 - 1.5 | Marginally profitable |
| 1.5 - 2.5 | Good performance |
| 2.5 - 4.0 | Excellent performance |
| > 4.0 | Suspiciously high (possible overfitting) |

## Simple Example: Calculating Profit Factor

```rust
fn main() {
    // Trading results for the month
    let winning_trades = vec![150.0, 200.0, 300.0, 120.0, 180.0];
    let losing_trades = vec![-80.0, -100.0, -60.0, -90.0];

    // Calculate total gross profit
    let total_profit: f64 = winning_trades.iter().sum();

    // Calculate total gross loss (absolute value)
    let total_loss: f64 = losing_trades.iter()
        .map(|&x: &f64| x.abs())
        .sum();

    // Calculate Profit Factor
    let profit_factor = total_profit / total_loss;

    println!("Total Profit: ${:.2}", total_profit);
    println!("Total Loss: ${:.2}", total_loss);
    println!("Profit Factor: {:.2}", profit_factor);

    // Interpretation
    if profit_factor < 1.0 {
        println!("Strategy is losing money!");
    } else if profit_factor >= 1.5 {
        println!("Strategy is performing well!");
    } else {
        println!("Strategy is marginally profitable.");
    }
}
```

Output:
```
Total Profit: $950.00
Total Loss: $330.00
Profit Factor: 2.88
Strategy is performing well!
```

## Creating a Trade Structure

Let's build a more realistic example with individual trades:

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    trade_type: TradeType,
}

#[derive(Debug, Clone, Copy)]
enum TradeType {
    Long,
    Short,
}

impl Trade {
    fn new(symbol: &str, entry_price: f64, exit_price: f64, quantity: f64, trade_type: TradeType) -> Self {
        Trade {
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            trade_type,
        }
    }

    // Calculate profit/loss for this trade
    fn pnl(&self) -> f64 {
        match self.trade_type {
            TradeType::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeType::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn is_winner(&self) -> bool {
        self.pnl() > 0.0
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long),   // +2000
        Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long),    // -1000
        Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long),   // +1000
        Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long),       // -100
        Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short),  // +3000
    ];

    let total_profit: f64 = trades.iter()
        .filter(|t| t.is_winner())
        .map(|t| t.pnl())
        .sum();

    let total_loss: f64 = trades.iter()
        .filter(|t| !t.is_winner())
        .map(|t| t.pnl().abs())
        .sum();

    let profit_factor = if total_loss > 0.0 {
        total_profit / total_loss
    } else {
        f64::INFINITY // All trades were winners!
    };

    println!("Total Profit: ${:.2}", total_profit);
    println!("Total Loss: ${:.2}", total_loss);
    println!("Profit Factor: {:.2}", profit_factor);
    println!("\nTrade breakdown:");
    for (i, trade) in trades.iter().enumerate() {
        println!("  Trade {}: {} {:?} - PnL: ${:.2}",
            i + 1,
            trade.symbol,
            trade.trade_type,
            trade.pnl()
        );
    }
}
```

## Building a Backtesting Framework

Let's create a simple backtesting system that calculates Profit Factor:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct BacktestResult {
    trades: Vec<Trade>,
    metrics: HashMap<String, f64>,
}

impl BacktestResult {
    fn new(trades: Vec<Trade>) -> Self {
        let mut result = BacktestResult {
            trades,
            metrics: HashMap::new(),
        };
        result.calculate_metrics();
        result
    }

    fn calculate_metrics(&mut self) {
        let total_profit: f64 = self.trades.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = self.trades.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        let profit_factor = if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        };

        let win_count = self.trades.iter().filter(|t| t.is_winner()).count();
        let total_count = self.trades.len();
        let win_rate = win_count as f64 / total_count as f64;

        let net_profit = total_profit - total_loss;

        self.metrics.insert("total_profit".to_string(), total_profit);
        self.metrics.insert("total_loss".to_string(), total_loss);
        self.metrics.insert("profit_factor".to_string(), profit_factor);
        self.metrics.insert("win_rate".to_string(), win_rate);
        self.metrics.insert("net_profit".to_string(), net_profit);
        self.metrics.insert("total_trades".to_string(), total_count as f64);
    }

    fn print_report(&self) {
        println!("=== Backtest Report ===");
        println!("Total Trades: {:.0}", self.metrics.get("total_trades").unwrap_or(&0.0));
        println!("Win Rate: {:.2}%", self.metrics.get("win_rate").unwrap_or(&0.0) * 100.0);
        println!("Total Profit: ${:.2}", self.metrics.get("total_profit").unwrap_or(&0.0));
        println!("Total Loss: ${:.2}", self.metrics.get("total_loss").unwrap_or(&0.0));
        println!("Net Profit: ${:.2}", self.metrics.get("net_profit").unwrap_or(&0.0));
        println!("Profit Factor: {:.2}", self.metrics.get("profit_factor").unwrap_or(&0.0));

        let pf = self.metrics.get("profit_factor").unwrap_or(&0.0);
        println!("\nInterpretation:");
        if *pf < 1.0 {
            println!("  ⚠️  Strategy is losing money!");
        } else if *pf < 1.5 {
            println!("  ⚡ Strategy is marginally profitable.");
        } else if *pf < 2.5 {
            println!("  ✅ Good performance!");
        } else if *pf <= 4.0 {
            println!("  🎯 Excellent performance!");
        } else {
            println!("  ⚠️  Suspiciously high - check for overfitting!");
        }
    }
}

fn main() {
    // Simulate a trading strategy
    let trades = vec![
        Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long),
        Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long),
        Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long),
        Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long),
        Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short),
        Trade::new("ETH", 2600.0, 2800.0, 5.0, TradeType::Long),
        Trade::new("BTC", 43000.0, 41000.0, 2.0, TradeType::Short),
    ];

    let result = BacktestResult::new(trades);
    result.print_report();
}
```

## Comparing Strategies

Let's compare two different trading strategies using Profit Factor:

```rust
struct Strategy {
    name: String,
    trades: Vec<Trade>,
}

impl Strategy {
    fn new(name: &str) -> Self {
        Strategy {
            name: name.to_string(),
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    fn calculate_profit_factor(&self) -> f64 {
        let total_profit: f64 = self.trades.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = self.trades.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        }
    }

    fn print_summary(&self) {
        println!("\n{}", "=".repeat(50));
        println!("Strategy: {}", self.name);
        println!("{}", "=".repeat(50));

        let result = BacktestResult::new(self.trades.clone());
        result.print_report();
    }
}

fn main() {
    // Strategy 1: Aggressive (high risk, high reward)
    let mut aggressive = Strategy::new("Aggressive Momentum");
    aggressive.add_trade(Trade::new("BTC", 40000.0, 45000.0, 2.0, TradeType::Long));  // +10000
    aggressive.add_trade(Trade::new("BTC", 45000.0, 42000.0, 2.0, TradeType::Long));  // -6000
    aggressive.add_trade(Trade::new("BTC", 42000.0, 48000.0, 1.5, TradeType::Long));  // +9000
    aggressive.add_trade(Trade::new("BTC", 48000.0, 44000.0, 1.5, TradeType::Long));  // -6000

    // Strategy 2: Conservative (lower risk, consistent gains)
    let mut conservative = Strategy::new("Conservative Mean Reversion");
    conservative.add_trade(Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeType::Long));  // +1000
    conservative.add_trade(Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeType::Long));  // -500
    conservative.add_trade(Trade::new("BTC", 40500.0, 41500.0, 1.0, TradeType::Long));  // +1000
    conservative.add_trade(Trade::new("BTC", 41500.0, 41200.0, 1.0, TradeType::Long));  // -300
    conservative.add_trade(Trade::new("BTC", 41200.0, 42000.0, 1.0, TradeType::Long));  // +800
    conservative.add_trade(Trade::new("BTC", 42000.0, 41700.0, 1.0, TradeType::Long));  // -300

    aggressive.print_summary();
    conservative.print_summary();

    // Compare
    println!("\n{}", "=".repeat(50));
    println!("COMPARISON");
    println!("{}", "=".repeat(50));
    println!("Aggressive PF: {:.2}", aggressive.calculate_profit_factor());
    println!("Conservative PF: {:.2}", conservative.calculate_profit_factor());
}
```

## Advanced: Profit Factor Over Time

Track how Profit Factor changes as you execute more trades:

```rust
struct PerformanceTracker {
    trades: Vec<Trade>,
}

impl PerformanceTracker {
    fn new() -> Self {
        PerformanceTracker {
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
        self.print_current_pf();
    }

    fn calculate_profit_factor_at(&self, end_index: usize) -> f64 {
        let trades_slice = &self.trades[0..=end_index];

        let total_profit: f64 = trades_slice.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = trades_slice.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        }
    }

    fn print_current_pf(&self) {
        if !self.trades.is_empty() {
            let pf = self.calculate_profit_factor_at(self.trades.len() - 1);
            println!("After {} trades: PF = {:.2}", self.trades.len(), pf);
        }
    }

    fn plot_pf_curve(&self) {
        println!("\nProfit Factor Evolution:");
        println!("{}", "=".repeat(60));

        for i in 0..self.trades.len() {
            let pf = self.calculate_profit_factor_at(i);
            let bar_length = if pf.is_finite() {
                (pf * 10.0).min(50.0) as usize
            } else {
                50
            };
            let bar = "█".repeat(bar_length);
            println!("Trade {:2}: {:.2} | {}", i + 1, pf, bar);
        }
    }
}

fn main() {
    let mut tracker = PerformanceTracker::new();

    println!("Starting trading session...\n");

    tracker.add_trade(Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long));
    tracker.add_trade(Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long));
    tracker.add_trade(Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long));
    tracker.add_trade(Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long));
    tracker.add_trade(Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short));

    tracker.plot_pf_curve();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Profit Factor | Ratio of total profits to total losses |
| Formula | `Total Gross Profit / Total Gross Loss` |
| Interpretation | > 1.0 = profitable, < 1.0 = losing |
| Good Range | 1.5 - 4.0 for realistic strategies |
| Overfitting Warning | PF > 4.0 may indicate curve fitting |
| Backtesting | Essential metric for strategy evaluation |

## Practical Exercises

1. **Win Rate vs Profit Factor**: Create two strategies:
   - Strategy A: 70% win rate, average win $100, average loss $200
   - Strategy B: 40% win rate, average win $400, average loss $100

   Calculate the Profit Factor for each. Which is better?

2. **Trade Journal Analyzer**: Write a program that:
   - Reads trades from a CSV file (symbol, entry, exit, quantity, type)
   - Calculates Profit Factor
   - Shows breakdown by symbol
   - Identifies which assets have the best PF

3. **Monthly Performance**: Track trades across multiple months and calculate:
   - Profit Factor per month
   - Overall Profit Factor
   - Identify best and worst performing months

4. **Risk-Adjusted PF**: Extend the Profit Factor calculation to include:
   - Maximum drawdown
   - Sharpe ratio
   - Create a composite score combining PF with other metrics

## Homework

1. **Complete Strategy Backtester**: Build a backtesting system that:
   - Simulates 100 random trades with realistic price movements
   - Calculates Profit Factor, win rate, and net profit
   - Generates a report showing daily, weekly, and monthly PF
   - Warns if PF is suspiciously high (> 4.0)

2. **Monte Carlo Simulation**:
   - Take a strategy with known PF (e.g., 2.0)
   - Run 1000 simulations with randomized trade order
   - Calculate the distribution of possible PF values
   - Plot a histogram of results

3. **Multi-Asset Portfolio**: Create a portfolio tracker that:
   - Tracks trades across BTC, ETH, and SOL
   - Calculates Profit Factor for each asset separately
   - Calculates overall portfolio Profit Factor
   - Shows which asset contributes most to overall PF

4. **Adaptive Strategy**: Implement a strategy that:
   - Monitors its own Profit Factor in real-time
   - If PF drops below 1.5, reduces position size by 50%
   - If PF goes above 2.5, increases position size by 25%
   - Logs all position size adjustments

## Navigation

[← Previous day](../285-previous-chapter/en.md) | [Next day →](../287-next-chapter/en.md)
