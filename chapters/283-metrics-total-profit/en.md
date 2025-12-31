# Day 283: Metrics: Total Profit

## Trading Analogy

Imagine you're managing a trading bot that executed 100 trades over the past month. What's the first question you'd ask? Of course: **"How much did we make?"** Total Profit is the most basic and intuitive metric in backtesting. It shows the absolute financial result of your strategy: the sum of all profits minus the sum of all losses.

In real trading, this is equivalent to asking: "If I started with $10,000, how much would I have at the end?" Total Profit answers exactly this question.

## What is Total Profit?

**Total Profit** is the sum of all trade results over the testing period:

```
Total Profit = Σ (PnL of each trade)
```

Where PnL (Profit and Loss) of each trade is calculated as:

```
PnL = (Exit Price - Entry Price) × Quantity - Fees
```

## Basic Total Profit Calculation

```rust
fn main() {
    // Trade results (PnL in dollars)
    let trades = vec![
        150.0,   // Winning trade
        -80.0,   // Losing trade
        200.0,   // Winning trade
        -50.0,   // Losing trade
        120.0,   // Winning trade
        -30.0,   // Losing trade
        180.0,   // Winning trade
        -100.0,  // Losing trade
    ];

    let total_profit = calculate_total_profit(&trades);

    println!("Total trades: {}", trades.len());
    println!("Total profit: ${:.2}", total_profit);
}

fn calculate_total_profit(trades: &[f64]) -> f64 {
    trades.iter().sum()
}
```

## Detailed Calculation with Profit and Loss Breakdown

```rust
fn main() {
    let trades = vec![
        150.0, -80.0, 200.0, -50.0, 120.0, -30.0, 180.0, -100.0
    ];

    let analysis = analyze_profits(&trades);
    print_profit_analysis(&analysis);
}

struct ProfitAnalysis {
    total_profit: f64,
    gross_profit: f64,      // Sum of all profits
    gross_loss: f64,        // Sum of all losses (negative number)
    profit_factor: f64,     // Gross Profit / |Gross Loss|
    winning_trades: usize,
    losing_trades: usize,
}

fn analyze_profits(trades: &[f64]) -> ProfitAnalysis {
    let mut gross_profit = 0.0;
    let mut gross_loss = 0.0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;

    for &pnl in trades {
        if pnl > 0.0 {
            gross_profit += pnl;
            winning_trades += 1;
        } else if pnl < 0.0 {
            gross_loss += pnl;
            losing_trades += 1;
        }
    }

    let total_profit = gross_profit + gross_loss;

    // Profit Factor = Gross Profit / |Gross Loss|
    let profit_factor = if gross_loss != 0.0 {
        gross_profit / gross_loss.abs()
    } else {
        f64::INFINITY
    };

    ProfitAnalysis {
        total_profit,
        gross_profit,
        gross_loss,
        profit_factor,
        winning_trades,
        losing_trades,
    }
}

fn print_profit_analysis(analysis: &ProfitAnalysis) {
    println!("╔═══════════════════════════════════════╗");
    println!("║         PROFIT ANALYSIS               ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Total Profit:        ${:>14.2} ║", analysis.total_profit);
    println!("║ Gross Profit:        ${:>14.2} ║", analysis.gross_profit);
    println!("║ Gross Loss:          ${:>14.2} ║", analysis.gross_loss);
    println!("║ Profit Factor:        {:>14.2} ║", analysis.profit_factor);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Winning Trades:       {:>14} ║", analysis.winning_trades);
    println!("║ Losing Trades:        {:>14} ║", analysis.losing_trades);
    println!("╚═══════════════════════════════════════╝");
}
```

## Trade Structure and PnL Calculation

```rust
use std::fmt;

#[derive(Debug, Clone)]
enum TradeDirection {
    Long,   // Buy
    Short,  // Sell
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    direction: TradeDirection,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_fee: f64,
    exit_fee: f64,
}

impl Trade {
    fn new(
        symbol: &str,
        direction: TradeDirection,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        fee_percent: f64,
    ) -> Self {
        let entry_value = entry_price * quantity;
        let exit_value = exit_price * quantity;
        let entry_fee = entry_value * (fee_percent / 100.0);
        let exit_fee = exit_value * (fee_percent / 100.0);

        Trade {
            symbol: symbol.to_string(),
            direction,
            entry_price,
            exit_price,
            quantity,
            entry_fee,
            exit_fee,
        }
    }

    fn calculate_pnl(&self) -> f64 {
        let price_diff = match self.direction {
            TradeDirection::Long => self.exit_price - self.entry_price,
            TradeDirection::Short => self.entry_price - self.exit_price,
        };

        let gross_pnl = price_diff * self.quantity;
        let total_fees = self.entry_fee + self.exit_fee;

        gross_pnl - total_fees
    }

    fn calculate_gross_pnl(&self) -> f64 {
        let price_diff = match self.direction {
            TradeDirection::Long => self.exit_price - self.entry_price,
            TradeDirection::Short => self.entry_price - self.exit_price,
        };
        price_diff * self.quantity
    }

    fn total_fees(&self) -> f64 {
        self.entry_fee + self.exit_fee
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = match self.direction {
            TradeDirection::Long => "LONG",
            TradeDirection::Short => "SHORT",
        };
        write!(
            f,
            "{} {} {:.4} @ {:.2} -> {:.2} | PnL: ${:.2}",
            self.symbol,
            direction,
            self.quantity,
            self.entry_price,
            self.exit_price,
            self.calculate_pnl()
        )
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", TradeDirection::Long, 42000.0, 43500.0, 0.5, 0.1),
        Trade::new("ETH", TradeDirection::Long, 2500.0, 2400.0, 2.0, 0.1),
        Trade::new("BTC", TradeDirection::Short, 43000.0, 42000.0, 0.3, 0.1),
        Trade::new("SOL", TradeDirection::Long, 95.0, 105.0, 10.0, 0.1),
        Trade::new("ETH", TradeDirection::Short, 2450.0, 2550.0, 1.5, 0.1),
    ];

    println!("=== Trade List ===\n");
    for (i, trade) in trades.iter().enumerate() {
        println!("{}. {}", i + 1, trade);
    }

    let total_profit: f64 = trades.iter().map(|t| t.calculate_pnl()).sum();
    let total_gross: f64 = trades.iter().map(|t| t.calculate_gross_pnl()).sum();
    let total_fees: f64 = trades.iter().map(|t| t.total_fees()).sum();

    println!("\n=== Summary ===");
    println!("Gross Profit:  ${:.2}", total_gross);
    println!("Total Fees:    ${:.2}", total_fees);
    println!("Net Profit:    ${:.2}", total_profit);
}
```

## Time-Weighted Profit Calculation

```rust
use std::time::Duration;

#[derive(Debug)]
struct TimedTrade {
    pnl: f64,
    duration_hours: f64,  // Trade duration in hours
}

fn main() {
    let trades = vec![
        TimedTrade { pnl: 150.0, duration_hours: 2.0 },
        TimedTrade { pnl: -80.0, duration_hours: 5.0 },
        TimedTrade { pnl: 200.0, duration_hours: 1.5 },
        TimedTrade { pnl: -50.0, duration_hours: 8.0 },
        TimedTrade { pnl: 300.0, duration_hours: 24.0 },
    ];

    let total_profit: f64 = trades.iter().map(|t| t.pnl).sum();
    let total_hours: f64 = trades.iter().map(|t| t.duration_hours).sum();
    let profit_per_hour = total_profit / total_hours;

    println!("Total Profit: ${:.2}", total_profit);
    println!("Total Time in Trades: {:.1} hours", total_hours);
    println!("Profit per Hour: ${:.2}/hour", profit_per_hour);

    // Projected daily profit (24 hours)
    let projected_daily = profit_per_hour * 24.0;
    println!("Projected Daily: ${:.2}/day", projected_daily);
}
```

## Complete Trading Journal

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TradeRecord {
    id: u64,
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    fees: f64,
}

struct TradingJournal {
    trades: Vec<TradeRecord>,
    next_id: u64,
}

impl TradingJournal {
    fn new() -> Self {
        TradingJournal {
            trades: Vec::new(),
            next_id: 1,
        }
    }

    fn add_trade(
        &mut self,
        symbol: &str,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        fee_percent: f64,
    ) {
        let gross_pnl = (exit_price - entry_price) * quantity;
        let fees = (entry_price + exit_price) * quantity * (fee_percent / 100.0);
        let net_pnl = gross_pnl - fees;

        let record = TradeRecord {
            id: self.next_id,
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            pnl: net_pnl,
            fees,
        };

        self.trades.push(record);
        self.next_id += 1;
    }

    fn total_profit(&self) -> f64 {
        self.trades.iter().map(|t| t.pnl).sum()
    }

    fn total_fees(&self) -> f64 {
        self.trades.iter().map(|t| t.fees).sum()
    }

    fn gross_profit(&self) -> f64 {
        self.trades
            .iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .sum()
    }

    fn gross_loss(&self) -> f64 {
        self.trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl)
            .sum()
    }

    fn profit_by_symbol(&self) -> HashMap<String, f64> {
        let mut profits: HashMap<String, f64> = HashMap::new();

        for trade in &self.trades {
            *profits.entry(trade.symbol.clone()).or_insert(0.0) += trade.pnl;
        }

        profits
    }

    fn best_trade(&self) -> Option<&TradeRecord> {
        self.trades.iter().max_by(|a, b| {
            a.pnl.partial_cmp(&b.pnl).unwrap()
        })
    }

    fn worst_trade(&self) -> Option<&TradeRecord> {
        self.trades.iter().min_by(|a, b| {
            a.pnl.partial_cmp(&b.pnl).unwrap()
        })
    }

    fn average_profit(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        self.total_profit() / self.trades.len() as f64
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║           TRADING JOURNAL                 ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Total Trades:        {:>20} ║", self.trades.len());
        println!("║ Total Profit:        ${:>18.2} ║", self.total_profit());
        println!("║ Gross Profit:        ${:>18.2} ║", self.gross_profit());
        println!("║ Gross Loss:          ${:>18.2} ║", self.gross_loss());
        println!("║ Total Fees:          ${:>18.2} ║", self.total_fees());
        println!("║ Average Profit:      ${:>18.2} ║", self.average_profit());
        println!("╠═══════════════════════════════════════════╣");

        if let Some(best) = self.best_trade() {
            println!("║ Best Trade:          ${:>18.2} ║", best.pnl);
        }
        if let Some(worst) = self.worst_trade() {
            println!("║ Worst Trade:         ${:>18.2} ║", worst.pnl);
        }

        println!("╠═══════════════════════════════════════════╣");
        println!("║ Profit by Symbol:                         ║");

        for (symbol, profit) in self.profit_by_symbol() {
            println!("║   {:6}:            ${:>18.2} ║", symbol, profit);
        }

        println!("╚═══════════════════════════════════════════╝");
    }
}

fn main() {
    let mut journal = TradingJournal::new();

    // Add trades
    journal.add_trade("BTC", 42000.0, 43500.0, 0.5, 0.1);
    journal.add_trade("ETH", 2500.0, 2400.0, 2.0, 0.1);
    journal.add_trade("BTC", 43000.0, 44000.0, 0.3, 0.1);
    journal.add_trade("SOL", 95.0, 105.0, 10.0, 0.1);
    journal.add_trade("ETH", 2450.0, 2300.0, 1.5, 0.1);
    journal.add_trade("BTC", 44500.0, 43800.0, 0.2, 0.1);
    journal.add_trade("SOL", 102.0, 98.0, 15.0, 0.1);
    journal.add_trade("BTC", 43500.0, 45000.0, 0.4, 0.1);

    journal.print_summary();
}
```

## Profit Calculation Relative to Initial Capital

```rust
struct BacktestResult {
    initial_capital: f64,
    final_capital: f64,
    trades: Vec<f64>,
}

impl BacktestResult {
    fn total_profit(&self) -> f64 {
        self.final_capital - self.initial_capital
    }

    fn total_return_percent(&self) -> f64 {
        (self.total_profit() / self.initial_capital) * 100.0
    }

    fn annualized_return(&self, days: u32) -> f64 {
        let total_return = self.total_return_percent() / 100.0;
        let years = days as f64 / 365.0;

        if years <= 0.0 {
            return 0.0;
        }

        // Annualized Return = (1 + total_return)^(1/years) - 1
        ((1.0 + total_return).powf(1.0 / years) - 1.0) * 100.0
    }

    fn print_results(&self, days: u32) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         BACKTEST RESULTS                  ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Initial Capital:     ${:>18.2} ║", self.initial_capital);
        println!("║ Final Capital:       ${:>18.2} ║", self.final_capital);
        println!("║ Total Profit:        ${:>18.2} ║", self.total_profit());
        println!("║ Total Return:         {:>17.2}% ║", self.total_return_percent());
        println!("║ Annualized Return:    {:>17.2}% ║", self.annualized_return(days));
        println!("║ Test Period:          {:>14} days ║", days);
        println!("╚═══════════════════════════════════════════╝");
    }
}

fn main() {
    // Simulate a 90-day backtest
    let trades = vec![
        500.0, -200.0, 800.0, -150.0, 600.0,
        -300.0, 450.0, -100.0, 700.0, -250.0,
        550.0, -180.0, 900.0, -400.0, 350.0,
    ];

    let initial_capital = 10000.0;
    let total_pnl: f64 = trades.iter().sum();
    let final_capital = initial_capital + total_pnl;

    let result = BacktestResult {
        initial_capital,
        final_capital,
        trades,
    };

    result.print_results(90);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Total Profit | Sum of PnL from all trades |
| Gross Profit | Sum of only winning trades |
| Gross Loss | Sum of only losing trades |
| Net Profit | Profit after deducting fees |
| Profit Factor | Ratio of gross profit to gross loss |
| Total Return % | Percentage return relative to initial capital |
| Annualized Return | Return normalized to a yearly basis |

## Practice Exercises

1. **PnL Calculator**: Write a function that takes a vector of trades (entry, exit, quantity) and returns total profit accounting for fees.

2. **Period Analysis**: Implement a function that groups trades by month and calculates profit for each month.

3. **Equity Curve**: Create a function that takes a list of PnL values and initial capital, returning a vector of capital values after each trade.

4. **Strategy Comparison**: Write a program that compares Total Profit of two different strategies and determines the better one.

## Homework

1. Implement a `StrategyMetrics` struct with fields:
   - `total_profit`
   - `gross_profit`
   - `gross_loss`
   - `profit_factor`
   - `average_win`
   - `average_loss`
   - `largest_win`
   - `largest_loss`

   And a method `from_trades(trades: &[f64]) -> Self`

2. Add a `rolling_profit(window: usize)` method to `TradingJournal` that calculates the rolling sum of profit over the last N trades.

3. Implement a function `compare_strategies(strategy_a: &[f64], strategy_b: &[f64])` that outputs a comparative metrics table.

4. Create a backtest simulator that:
   - Takes initial capital and a list of signals (Buy/Sell)
   - Calculates PnL for each trade
   - Tracks current balance
   - Outputs final statistics

## Navigation

[← Previous day](../282-backtest-walk-forward-analysis/en.md) | [Next day →](../284-metrics-profit-factor/en.md)
