# Day 284: Metrics: Number of Trades

## Trading Analogy

Imagine you're analyzing the results of a trading strategy over a month. The first question any trader asks is: "How many trades were executed?" This is a fundamental metric that shows strategy activity and directly impacts commissions, risk, and psychological load.

Too many trades — high commissions "eat" your profits (overtrading). Too few — the strategy might miss opportunities. The number of trades is the starting point for analyzing any trading system.

## What is the "Number of Trades" Metric?

Number of Trades is a basic backtesting metric that shows:
- Total number of executed trades over a period
- Activity level of the trading strategy
- Frequency of position entries and exits

## Basic Trade Counting

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_time: u64,  // Unix timestamp
    exit_time: u64,
}

impl Trade {
    fn pnl(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.quantity
    }

    fn is_profitable(&self) -> bool {
        self.pnl() > 0.0
    }
}

fn count_trades(trades: &[Trade]) -> usize {
    trades.len()
}

fn main() {
    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.5,
            entry_time: 1700000000,
            exit_time: 1700003600,
        },
        Trade {
            symbol: "ETH".to_string(),
            entry_price: 2200.0,
            exit_price: 2150.0,
            quantity: 2.0,
            entry_time: 1700010000,
            exit_time: 1700020000,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 43000.0,
            exit_price: 44200.0,
            quantity: 0.3,
            entry_time: 1700030000,
            exit_time: 1700040000,
        },
    ];

    println!("Total trades: {}", count_trades(&trades));
}
```

## Extended Trade Count Statistics

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    side: TradeSide,
}

#[derive(Debug, Clone, PartialEq)]
enum TradeSide {
    Long,
    Short,
}

impl Trade {
    fn pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn is_profitable(&self) -> bool {
        self.pnl() > 0.0
    }
}

#[derive(Debug)]
struct TradeCountMetrics {
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    breakeven_trades: usize,
    long_trades: usize,
    short_trades: usize,
    trades_by_symbol: HashMap<String, usize>,
}

impl TradeCountMetrics {
    fn calculate(trades: &[Trade]) -> Self {
        let total_trades = trades.len();

        let winning_trades = trades.iter()
            .filter(|t| t.pnl() > 0.0)
            .count();

        let losing_trades = trades.iter()
            .filter(|t| t.pnl() < 0.0)
            .count();

        let breakeven_trades = trades.iter()
            .filter(|t| t.pnl() == 0.0)
            .count();

        let long_trades = trades.iter()
            .filter(|t| t.side == TradeSide::Long)
            .count();

        let short_trades = trades.iter()
            .filter(|t| t.side == TradeSide::Short)
            .count();

        let mut trades_by_symbol: HashMap<String, usize> = HashMap::new();
        for trade in trades {
            *trades_by_symbol.entry(trade.symbol.clone()).or_insert(0) += 1;
        }

        TradeCountMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            breakeven_trades,
            long_trades,
            short_trades,
            trades_by_symbol,
        }
    }

    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.total_trades as f64) * 100.0
    }

    fn print_report(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║       TRADE COUNT STATISTICS          ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Total trades:      {:>18} ║", self.total_trades);
        println!("║ Winning trades:    {:>18} ║", self.winning_trades);
        println!("║ Losing trades:     {:>18} ║", self.losing_trades);
        println!("║ Breakeven trades:  {:>18} ║", self.breakeven_trades);
        println!("╠═══════════════════════════════════════╣");
        println!("║ Win Rate:          {:>17.2}% ║", self.win_rate());
        println!("╠═══════════════════════════════════════╣");
        println!("║ Long positions:    {:>18} ║", self.long_trades);
        println!("║ Short positions:   {:>18} ║", self.short_trades);
        println!("╠═══════════════════════════════════════╣");
        println!("║ By instrument:                        ║");
        for (symbol, count) in &self.trades_by_symbol {
            println!("║   {:6}:          {:>18} ║", symbol, count);
        }
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), entry_price: 42000.0, exit_price: 43500.0, quantity: 0.5, side: TradeSide::Long },
        Trade { symbol: "BTC".to_string(), entry_price: 43500.0, exit_price: 43000.0, quantity: 0.5, side: TradeSide::Short },
        Trade { symbol: "ETH".to_string(), entry_price: 2200.0, exit_price: 2150.0, quantity: 2.0, side: TradeSide::Long },
        Trade { symbol: "ETH".to_string(), entry_price: 2150.0, exit_price: 2300.0, quantity: 1.5, side: TradeSide::Long },
        Trade { symbol: "BTC".to_string(), entry_price: 44000.0, exit_price: 44000.0, quantity: 0.2, side: TradeSide::Long },
    ];

    let metrics = TradeCountMetrics::calculate(&trades);
    metrics.print_report();
}
```

## Time-Based Trade Analysis

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    timestamp: u64,  // Unix timestamp
}

#[derive(Debug)]
struct TimeBasedTradeAnalysis {
    trades_per_day: HashMap<String, usize>,
    trades_per_hour: HashMap<u32, usize>,
    average_trades_per_day: f64,
}

impl TimeBasedTradeAnalysis {
    fn analyze(trades: &[Trade]) -> Self {
        let mut trades_per_day: HashMap<String, usize> = HashMap::new();
        let mut trades_per_hour: HashMap<u32, usize> = HashMap::new();

        for trade in trades {
            // Simple timestamp to date conversion (for example)
            let day = trade.timestamp / 86400;
            let day_str = format!("Day {}", day);
            *trades_per_day.entry(day_str).or_insert(0) += 1;

            // Hour of day (0-23)
            let hour = ((trade.timestamp % 86400) / 3600) as u32;
            *trades_per_hour.entry(hour).or_insert(0) += 1;
        }

        let total_days = trades_per_day.len().max(1);
        let average_trades_per_day = trades.len() as f64 / total_days as f64;

        TimeBasedTradeAnalysis {
            trades_per_day,
            trades_per_hour,
            average_trades_per_day,
        }
    }

    fn most_active_hour(&self) -> Option<(u32, usize)> {
        self.trades_per_hour
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, count)| (*hour, *count))
    }
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), entry_price: 42000.0, exit_price: 43500.0, quantity: 0.5, timestamp: 1700000000 },
        Trade { symbol: "BTC".to_string(), entry_price: 43500.0, exit_price: 44000.0, quantity: 0.3, timestamp: 1700003600 },
        Trade { symbol: "ETH".to_string(), entry_price: 2200.0, exit_price: 2250.0, quantity: 2.0, timestamp: 1700007200 },
        Trade { symbol: "BTC".to_string(), entry_price: 44000.0, exit_price: 43800.0, quantity: 0.4, timestamp: 1700086400 },
        Trade { symbol: "ETH".to_string(), entry_price: 2250.0, exit_price: 2300.0, quantity: 1.5, timestamp: 1700090000 },
    ];

    let analysis = TimeBasedTradeAnalysis::analyze(&trades);

    println!("Average trades per day: {:.2}", analysis.average_trades_per_day);

    if let Some((hour, count)) = analysis.most_active_hour() {
        println!("Most active hour: {}:00 ({} trades)", hour, count);
    }
}
```

## Trade Frequency Analysis

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_time: u64,
    exit_time: u64,
}

#[derive(Debug)]
struct TradeFrequencyMetrics {
    total_trades: usize,
    total_period_seconds: u64,
    average_trade_duration: f64,
    average_time_between_trades: f64,
    trades_per_hour: f64,
    trades_per_day: f64,
}

impl TradeFrequencyMetrics {
    fn calculate(trades: &[Trade]) -> Option<Self> {
        if trades.is_empty() {
            return None;
        }

        let total_trades = trades.len();

        // Find first entry and last exit
        let first_entry = trades.iter().map(|t| t.entry_time).min()?;
        let last_exit = trades.iter().map(|t| t.exit_time).max()?;
        let total_period_seconds = last_exit - first_entry;

        // Average trade duration
        let total_duration: u64 = trades.iter()
            .map(|t| t.exit_time - t.entry_time)
            .sum();
        let average_trade_duration = total_duration as f64 / total_trades as f64;

        // Average time between trades
        let average_time_between_trades = if total_trades > 1 {
            total_period_seconds as f64 / (total_trades - 1) as f64
        } else {
            0.0
        };

        // Trades per hour and per day
        let hours = (total_period_seconds as f64 / 3600.0).max(1.0);
        let days = (total_period_seconds as f64 / 86400.0).max(1.0);

        Some(TradeFrequencyMetrics {
            total_trades,
            total_period_seconds,
            average_trade_duration,
            average_time_between_trades,
            trades_per_hour: total_trades as f64 / hours,
            trades_per_day: total_trades as f64 / days,
        })
    }

    fn format_duration(seconds: f64) -> String {
        if seconds < 60.0 {
            format!("{:.0} sec", seconds)
        } else if seconds < 3600.0 {
            format!("{:.1} min", seconds / 60.0)
        } else if seconds < 86400.0 {
            format!("{:.1} hr", seconds / 3600.0)
        } else {
            format!("{:.1} days", seconds / 86400.0)
        }
    }

    fn print_report(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║     TRADE FREQUENCY ANALYSIS          ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Total trades:      {:>18} ║", self.total_trades);
        println!("║ Analysis period:   {:>18} ║", Self::format_duration(self.total_period_seconds as f64));
        println!("╠═══════════════════════════════════════╣");
        println!("║ Avg trade length:  {:>18} ║", Self::format_duration(self.average_trade_duration));
        println!("║ Between trades:    {:>18} ║", Self::format_duration(self.average_time_between_trades));
        println!("╠═══════════════════════════════════════╣");
        println!("║ Trades per hour:   {:>18.2} ║", self.trades_per_hour);
        println!("║ Trades per day:    {:>18.2} ║", self.trades_per_day);
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), entry_time: 1700000000, exit_time: 1700001800 },
        Trade { symbol: "ETH".to_string(), entry_time: 1700003600, exit_time: 1700005400 },
        Trade { symbol: "BTC".to_string(), entry_time: 1700010000, exit_time: 1700013600 },
        Trade { symbol: "BTC".to_string(), entry_time: 1700020000, exit_time: 1700021200 },
        Trade { symbol: "ETH".to_string(), entry_time: 1700030000, exit_time: 1700032800 },
    ];

    if let Some(metrics) = TradeFrequencyMetrics::calculate(&trades) {
        metrics.print_report();
    }
}
```

## Complete Trading Statistics Analyzer

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum TradeSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: TradeSide,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_time: u64,
    exit_time: u64,
    fees: f64,
}

impl Trade {
    fn gross_pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn net_pnl(&self) -> f64 {
        self.gross_pnl() - self.fees
    }

    fn duration_seconds(&self) -> u64 {
        self.exit_time - self.entry_time
    }
}

#[derive(Debug)]
struct BacktestReport {
    // Count metrics
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,

    // By direction
    long_trades: usize,
    short_trades: usize,
    long_winners: usize,
    short_winners: usize,

    // By instrument
    trades_by_symbol: HashMap<String, usize>,

    // Derived metrics
    win_rate: f64,
    long_win_rate: f64,
    short_win_rate: f64,

    // Financial metrics
    total_gross_pnl: f64,
    total_net_pnl: f64,
    total_fees: f64,

    // Averages
    avg_winner: f64,
    avg_loser: f64,
    profit_factor: f64,
}

impl BacktestReport {
    fn generate(trades: &[Trade]) -> Self {
        let total_trades = trades.len();

        let winners: Vec<&Trade> = trades.iter().filter(|t| t.net_pnl() > 0.0).collect();
        let losers: Vec<&Trade> = trades.iter().filter(|t| t.net_pnl() < 0.0).collect();

        let winning_trades = winners.len();
        let losing_trades = losers.len();

        let long_trades: Vec<&Trade> = trades.iter()
            .filter(|t| t.side == TradeSide::Long)
            .collect();
        let short_trades: Vec<&Trade> = trades.iter()
            .filter(|t| t.side == TradeSide::Short)
            .collect();

        let long_winners = long_trades.iter().filter(|t| t.net_pnl() > 0.0).count();
        let short_winners = short_trades.iter().filter(|t| t.net_pnl() > 0.0).count();

        let mut trades_by_symbol: HashMap<String, usize> = HashMap::new();
        for trade in trades {
            *trades_by_symbol.entry(trade.symbol.clone()).or_insert(0) += 1;
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let long_win_rate = if !long_trades.is_empty() {
            (long_winners as f64 / long_trades.len() as f64) * 100.0
        } else {
            0.0
        };

        let short_win_rate = if !short_trades.is_empty() {
            (short_winners as f64 / short_trades.len() as f64) * 100.0
        } else {
            0.0
        };

        let total_gross_pnl: f64 = trades.iter().map(|t| t.gross_pnl()).sum();
        let total_fees: f64 = trades.iter().map(|t| t.fees).sum();
        let total_net_pnl = total_gross_pnl - total_fees;

        let avg_winner = if !winners.is_empty() {
            winners.iter().map(|t| t.net_pnl()).sum::<f64>() / winners.len() as f64
        } else {
            0.0
        };

        let avg_loser = if !losers.is_empty() {
            losers.iter().map(|t| t.net_pnl()).sum::<f64>() / losers.len() as f64
        } else {
            0.0
        };

        let gross_profit: f64 = winners.iter().map(|t| t.net_pnl()).sum();
        let gross_loss: f64 = losers.iter().map(|t| t.net_pnl().abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        BacktestReport {
            total_trades,
            winning_trades,
            losing_trades,
            long_trades: long_trades.len(),
            short_trades: short_trades.len(),
            long_winners,
            short_winners,
            trades_by_symbol,
            win_rate,
            long_win_rate,
            short_win_rate,
            total_gross_pnl,
            total_net_pnl,
            total_fees,
            avg_winner,
            avg_loser,
            profit_factor,
        }
    }

    fn print(&self) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║              BACKTEST REPORT                          ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║                  NUMBER OF TRADES                     ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║ Total trades:            {:>28} ║", self.total_trades);
        println!("║ Winning trades:          {:>28} ║", self.winning_trades);
        println!("║ Losing trades:           {:>28} ║", self.losing_trades);
        println!("║ Win Rate:                {:>27.2}% ║", self.win_rate);
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║                   BY DIRECTION                        ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║ Long trades:             {:>28} ║", self.long_trades);
        println!("║ Long winners:            {:>28} ║", self.long_winners);
        println!("║ Long Win Rate:           {:>27.2}% ║", self.long_win_rate);
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║ Short trades:            {:>28} ║", self.short_trades);
        println!("║ Short winners:           {:>28} ║", self.short_winners);
        println!("║ Short Win Rate:          {:>27.2}% ║", self.short_win_rate);
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║                  BY INSTRUMENT                        ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        for (symbol, count) in &self.trades_by_symbol {
            println!("║ {:8}:                {:>28} ║", symbol, count);
        }
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║               FINANCIAL RESULTS                       ║");
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║ Gross PnL:               ${:>27.2} ║", self.total_gross_pnl);
        println!("║ Fees:                    ${:>27.2} ║", self.total_fees);
        println!("║ Net PnL:                 ${:>27.2} ║", self.total_net_pnl);
        println!("╠═══════════════════════════════════════════════════════╣");
        println!("║ Average winner:          ${:>27.2} ║", self.avg_winner);
        println!("║ Average loser:           ${:>27.2} ║", self.avg_loser);
        println!("║ Profit Factor:           {:>28.2} ║", self.profit_factor);
        println!("╚═══════════════════════════════════════════════════════╝");
    }
}

fn main() {
    let trades = vec![
        Trade {
            id: 1,
            symbol: "BTC".to_string(),
            side: TradeSide::Long,
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.5,
            entry_time: 1700000000,
            exit_time: 1700003600,
            fees: 21.25,
        },
        Trade {
            id: 2,
            symbol: "ETH".to_string(),
            side: TradeSide::Long,
            entry_price: 2200.0,
            exit_price: 2150.0,
            quantity: 2.0,
            entry_time: 1700010000,
            exit_time: 1700020000,
            fees: 4.35,
        },
        Trade {
            id: 3,
            symbol: "BTC".to_string(),
            side: TradeSide::Short,
            entry_price: 44000.0,
            exit_price: 43500.0,
            quantity: 0.3,
            entry_time: 1700030000,
            exit_time: 1700040000,
            fees: 13.13,
        },
        Trade {
            id: 4,
            symbol: "ETH".to_string(),
            side: TradeSide::Long,
            entry_price: 2150.0,
            exit_price: 2350.0,
            quantity: 1.5,
            entry_time: 1700050000,
            exit_time: 1700060000,
            fees: 3.38,
        },
        Trade {
            id: 5,
            symbol: "BTC".to_string(),
            side: TradeSide::Long,
            entry_price: 43800.0,
            exit_price: 43600.0,
            quantity: 0.4,
            entry_time: 1700070000,
            exit_time: 1700080000,
            fees: 17.48,
        },
    ];

    let report = BacktestReport::generate(&trades);
    report.print();
}
```

## Comparing Strategies by Trade Count

```rust
#[derive(Debug)]
struct StrategyStats {
    name: String,
    total_trades: usize,
    win_rate: f64,
    net_pnl: f64,
    trades_per_day: f64,
}

fn compare_strategies(strategies: &[StrategyStats]) {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║                     STRATEGY COMPARISON                           ║");
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!("║ {:15} │ {:8} │ {:8} │ {:12} │ {:8} ║",
             "Strategy", "Trades", "Win%", "Net PnL", "Per Day");
    println!("╠═══════════════════════════════════════════════════════════════════╣");

    for s in strategies {
        println!("║ {:15} │ {:8} │ {:7.2}% │ ${:11.2} │ {:8.2} ║",
                 s.name, s.total_trades, s.win_rate, s.net_pnl, s.trades_per_day);
    }

    println!("╚═══════════════════════════════════════════════════════════════════╝");

    // Analysis
    if let Some(most_active) = strategies.iter().max_by_key(|s| s.total_trades) {
        println!("\nMost active: {} ({} trades)", most_active.name, most_active.total_trades);
    }

    if let Some(most_profitable) = strategies.iter()
        .max_by(|a, b| a.net_pnl.partial_cmp(&b.net_pnl).unwrap()) {
        println!("Most profitable: {} (${:.2})", most_profitable.name, most_profitable.net_pnl);
    }
}

fn main() {
    let strategies = vec![
        StrategyStats {
            name: "Scalping".to_string(),
            total_trades: 150,
            win_rate: 58.0,
            net_pnl: 1250.0,
            trades_per_day: 15.0,
        },
        StrategyStats {
            name: "Swing".to_string(),
            total_trades: 25,
            win_rate: 48.0,
            net_pnl: 3200.0,
            trades_per_day: 2.5,
        },
        StrategyStats {
            name: "Trend Follow".to_string(),
            total_trades: 12,
            win_rate: 42.0,
            net_pnl: 4500.0,
            trades_per_day: 1.2,
        },
    ];

    compare_strategies(&strategies);
}
```

## What We Learned

| Metric | Description | Application |
|--------|-------------|-------------|
| Total Trades | Total number of trades | Assess strategy activity |
| Win Rate | Percentage of profitable trades | Entry/exit quality |
| Trades by Symbol | Distribution by instrument | Diversification |
| Trades per Day | Trading frequency | Impact on commissions |
| Long/Short Ratio | Direction balance | Strategy balance |

## Practice Exercises

1. **Trade Filtering**: Write a function that filters trades by minimum volume and returns the count of filtered trades.

2. **Streak Analysis**: Implement counting of the maximum consecutive winning and losing trade streaks.

3. **Day of Week Grouping**: Write a function that groups trades by day of week and shows which day has the most trades.

## Homework

1. **Extended Statistics**: Add to `BacktestReport` these metrics:
   - Average duration of winning and losing trades
   - Maximum number of trades in a single day
   - Overtrading coefficient (ratio of trades to profitability)

2. **Overtrading Detector**: Create a function that analyzes trade count and warns if:
   - More than 20 trades per day
   - Win Rate drops as trade count increases
   - Fees exceed 10% of gross PnL

3. **Strategy Simulator**: Write a random trade generator with configurable parameters (win rate, average trade duration) and analyze how trade count affects the final result.

4. **Frequency Optimizer**: Implement a function that finds the optimal number of trades per day where net PnL is maximized considering fees.

## Navigation

[← Previous day](../283-metrics-total-pnl/en.md) | [Next day →](../285-metrics-win-rate/en.md)
