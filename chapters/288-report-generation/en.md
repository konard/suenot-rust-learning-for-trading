# Day 288: Report Generation

## Trading Analogy

Imagine you've been trading for a month and want to evaluate your performance. You need a comprehensive report showing:
- Total profit/loss
- Win rate (percentage of profitable trades)
- Maximum drawdown (largest loss from peak)
- Sharpe ratio (risk-adjusted returns)
- Trade distribution by symbol
- Daily P&L breakdown

Without a proper report, you're flying blind — you have the data but can't make sense of it. **Report generation** transforms raw trading data into meaningful insights that help you understand what worked, what didn't, and how to improve your strategy.

In backtesting, report generation is the final step where you analyze historical trade data to evaluate strategy performance before risking real money.

## What is Report Generation?

Report generation is the process of:
1. **Collecting** raw data (trades, prices, positions)
2. **Calculating** metrics (returns, drawdown, ratios)
3. **Formatting** results for human consumption
4. **Exporting** to various formats (text, JSON, CSV, HTML)

In Rust, we use structs to organize data, traits for formatting, and various libraries for export formats.

## Basic Report Structure

```rust
use std::fmt;

#[derive(Debug)]
struct TradeReport {
    symbol: String,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    total_pnl: f64,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
}

impl TradeReport {
    fn new(symbol: String, trades: &[Trade]) -> Self {
        let total_trades = trades.len() as u32;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl = 0.0;
        let mut total_wins = 0.0;
        let mut total_losses = 0.0;

        for trade in trades {
            total_pnl += trade.pnl;
            if trade.pnl > 0.0 {
                winning_trades += 1;
                total_wins += trade.pnl;
            } else {
                losing_trades += 1;
                total_losses += trade.pnl.abs();
            }
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if winning_trades > 0 {
            total_wins / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            total_losses / losing_trades as f64
        } else {
            0.0
        };

        TradeReport {
            symbol,
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            win_rate,
            avg_win,
            avg_loss,
        }
    }
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

impl fmt::Display for TradeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "=== Trade Report: {} ===\n\
             Total Trades: {}\n\
             Winning Trades: {}\n\
             Losing Trades: {}\n\
             Total P&L: ${:.2}\n\
             Win Rate: {:.2}%\n\
             Average Win: ${:.2}\n\
             Average Loss: ${:.2}\n\
             Profit Factor: {:.2}",
            self.symbol,
            self.total_trades,
            self.winning_trades,
            self.losing_trades,
            self.total_pnl,
            self.win_rate,
            self.avg_win,
            self.avg_loss,
            if self.avg_loss > 0.0 {
                self.avg_win * self.winning_trades as f64 / (self.avg_loss * self.losing_trades as f64)
            } else {
                0.0
            }
        )
    }
}

fn main() {
    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 40000.0,
            exit_price: 42000.0,
            quantity: 1.0,
            pnl: 2000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 42000.0,
            exit_price: 41000.0,
            quantity: 1.0,
            pnl: -1000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 41000.0,
            exit_price: 43000.0,
            quantity: 1.0,
            pnl: 2000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 43000.0,
            exit_price: 44500.0,
            quantity: 1.0,
            pnl: 1500.0,
        },
    ];

    let report = TradeReport::new("BTC".to_string(), &trades);
    println!("{}", report);
}
```

Output:
```
=== Trade Report: BTC ===
Total Trades: 4
Winning Trades: 3
Losing Trades: 1
Total P&L: $4500.00
Win Rate: 75.00%
Average Win: $1833.33
Average Loss: $1000.00
Profit Factor: 5.50
```

## Advanced Metrics: Drawdown Analysis

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct DailyPnL {
    date: String,
    pnl: f64,
    cumulative_pnl: f64,
}

#[derive(Debug)]
struct DrawdownReport {
    max_drawdown: f64,
    max_drawdown_percent: f64,
    drawdown_periods: Vec<DrawdownPeriod>,
    current_drawdown: f64,
}

#[derive(Debug)]
struct DrawdownPeriod {
    start_date: String,
    end_date: String,
    peak_value: f64,
    trough_value: f64,
    drawdown: f64,
    drawdown_percent: f64,
}

impl DrawdownReport {
    fn calculate(daily_pnls: &[DailyPnL]) -> Self {
        let mut max_drawdown = 0.0;
        let mut max_drawdown_percent = 0.0;
        let mut peak = 0.0;
        let mut drawdown_periods = Vec::new();
        let mut in_drawdown = false;
        let mut drawdown_start = String::new();
        let mut peak_date = String::new();

        for pnl in daily_pnls {
            if pnl.cumulative_pnl > peak {
                // New peak — end previous drawdown if any
                if in_drawdown {
                    in_drawdown = false;
                }
                peak = pnl.cumulative_pnl;
                peak_date = pnl.date.clone();
            }

            let current_drawdown = peak - pnl.cumulative_pnl;
            let current_drawdown_percent = if peak != 0.0 {
                (current_drawdown / peak) * 100.0
            } else {
                0.0
            };

            if current_drawdown > 0.0 && !in_drawdown {
                in_drawdown = true;
                drawdown_start = peak_date.clone();
            }

            if current_drawdown > max_drawdown {
                max_drawdown = current_drawdown;
                max_drawdown_percent = current_drawdown_percent;
            }

            // Record drawdown period when recovering
            if in_drawdown && current_drawdown == 0.0 {
                drawdown_periods.push(DrawdownPeriod {
                    start_date: drawdown_start.clone(),
                    end_date: pnl.date.clone(),
                    peak_value: peak,
                    trough_value: peak - max_drawdown,
                    drawdown: max_drawdown,
                    drawdown_percent: max_drawdown_percent,
                });
                in_drawdown = false;
            }
        }

        let current_drawdown = if let Some(last) = daily_pnls.last() {
            peak - last.cumulative_pnl
        } else {
            0.0
        };

        DrawdownReport {
            max_drawdown,
            max_drawdown_percent,
            drawdown_periods,
            current_drawdown,
        }
    }

    fn display(&self) {
        println!("=== Drawdown Analysis ===");
        println!("Maximum Drawdown: ${:.2}", self.max_drawdown);
        println!("Maximum Drawdown %: {:.2}%", self.max_drawdown_percent);
        println!("Current Drawdown: ${:.2}", self.current_drawdown);
        println!("\nDrawdown Periods: {}", self.drawdown_periods.len());

        for (i, period) in self.drawdown_periods.iter().enumerate() {
            println!("\nPeriod {}", i + 1);
            println!("  Start: {}", period.start_date);
            println!("  End: {}", period.end_date);
            println!("  Peak: ${:.2}", period.peak_value);
            println!("  Trough: ${:.2}", period.trough_value);
            println!("  Drawdown: ${:.2} ({:.2}%)", period.drawdown, period.drawdown_percent);
        }
    }
}

fn main() {
    let daily_pnls = vec![
        DailyPnL { date: "2024-01-01".to_string(), pnl: 1000.0, cumulative_pnl: 1000.0 },
        DailyPnL { date: "2024-01-02".to_string(), pnl: 500.0, cumulative_pnl: 1500.0 },
        DailyPnL { date: "2024-01-03".to_string(), pnl: -800.0, cumulative_pnl: 700.0 },
        DailyPnL { date: "2024-01-04".to_string(), pnl: -300.0, cumulative_pnl: 400.0 },
        DailyPnL { date: "2024-01-05".to_string(), pnl: 1200.0, cumulative_pnl: 1600.0 },
        DailyPnL { date: "2024-01-06".to_string(), pnl: 300.0, cumulative_pnl: 1900.0 },
        DailyPnL { date: "2024-01-07".to_string(), pnl: -500.0, cumulative_pnl: 1400.0 },
    ];

    let drawdown = DrawdownReport::calculate(&daily_pnls);
    drawdown.display();
}
```

## Exporting to JSON

```rust
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct BacktestReport {
    strategy_name: String,
    start_date: String,
    end_date: String,
    initial_capital: f64,
    final_capital: f64,
    total_return: f64,
    total_return_percent: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
}

impl BacktestReport {
    fn new(
        strategy_name: String,
        start_date: String,
        end_date: String,
        initial_capital: f64,
        final_capital: f64,
        sharpe_ratio: f64,
        max_drawdown: f64,
        win_rate: f64,
        total_trades: u32,
        winning_trades: u32,
        losing_trades: u32,
    ) -> Self {
        let total_return = final_capital - initial_capital;
        let total_return_percent = (total_return / initial_capital) * 100.0;

        BacktestReport {
            strategy_name,
            start_date,
            end_date,
            initial_capital,
            final_capital,
            total_return,
            total_return_percent,
            sharpe_ratio,
            max_drawdown,
            win_rate,
            total_trades,
            winning_trades,
            losing_trades,
        }
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = self.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;
        std::fs::write(filename, json)
    }
}

fn main() -> std::io::Result<()> {
    let report = BacktestReport::new(
        "Moving Average Crossover".to_string(),
        "2024-01-01".to_string(),
        "2024-12-31".to_string(),
        100_000.0,
        145_000.0,
        1.8,
        -12_000.0,
        65.5,
        150,
        98,
        52,
    );

    println!("{}", report.to_json().unwrap());

    // Save to file
    report.save_to_file("backtest_report.json")?;
    println!("\nReport saved to backtest_report.json");

    Ok(())
}
```

## CSV Export for Trade Log

```rust
use std::fs::File;
use std::io::{Write, Result};

#[derive(Debug)]
struct TradeLog {
    timestamp: String,
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
    commission: f64,
}

struct CsvReportWriter {
    filename: String,
}

impl CsvReportWriter {
    fn new(filename: String) -> Self {
        CsvReportWriter { filename }
    }

    fn write_trades(&self, trades: &[TradeLog]) -> Result<()> {
        let mut file = File::create(&self.filename)?;

        // Write header
        writeln!(
            file,
            "Timestamp,Symbol,Side,Quantity,Entry Price,Exit Price,P&L,Commission"
        )?;

        // Write data
        for trade in trades {
            writeln!(
                file,
                "{},{},{},{},{},{},{:.2},{:.2}",
                trade.timestamp,
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.entry_price,
                trade.exit_price,
                trade.pnl,
                trade.commission
            )?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let trades = vec![
        TradeLog {
            timestamp: "2024-01-01 09:30:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 1.0,
            entry_price: 40000.0,
            exit_price: 42000.0,
            pnl: 2000.0,
            commission: 40.0,
        },
        TradeLog {
            timestamp: "2024-01-02 14:15:00".to_string(),
            symbol: "ETH".to_string(),
            side: "SHORT".to_string(),
            quantity: 10.0,
            entry_price: 2200.0,
            exit_price: 2100.0,
            pnl: 1000.0,
            commission: 22.0,
        },
        TradeLog {
            timestamp: "2024-01-03 11:45:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 0.5,
            entry_price: 41000.0,
            exit_price: 40000.0,
            pnl: -500.0,
            commission: 20.5,
        },
    ];

    let writer = CsvReportWriter::new("trade_log.csv".to_string());
    writer.write_trades(&trades)?;

    println!("Trade log exported to trade_log.csv");
    Ok(())
}
```

## HTML Report Generation

```rust
use std::fs::File;
use std::io::{Write, Result};

struct HtmlReportGenerator {
    title: String,
}

impl HtmlReportGenerator {
    fn new(title: String) -> Self {
        HtmlReportGenerator { title }
    }

    fn generate(&self, report: &BacktestSummary, filename: &str) -> Result<()> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #4CAF50;
            padding-bottom: 10px;
        }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 10px;
            border-bottom: 1px solid #eee;
        }}
        .metric-label {{
            font-weight: bold;
            color: #666;
        }}
        .metric-value {{
            color: #333;
        }}
        .positive {{
            color: #4CAF50;
        }}
        .negative {{
            color: #f44336;
        }}
        .section {{
            margin-top: 30px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>

        <div class="section">
            <h2>Performance Summary</h2>
            <div class="metric">
                <span class="metric-label">Initial Capital:</span>
                <span class="metric-value">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Final Capital:</span>
                <span class="metric-value">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Total Return:</span>
                <span class="metric-value {}">${:.2} ({:.2}%)</span>
            </div>
        </div>

        <div class="section">
            <h2>Risk Metrics</h2>
            <div class="metric">
                <span class="metric-label">Maximum Drawdown:</span>
                <span class="metric-value negative">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Sharpe Ratio:</span>
                <span class="metric-value">{:.2}</span>
            </div>
        </div>

        <div class="section">
            <h2>Trade Statistics</h2>
            <div class="metric">
                <span class="metric-label">Total Trades:</span>
                <span class="metric-value">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Winning Trades:</span>
                <span class="metric-value positive">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Losing Trades:</span>
                <span class="metric-value negative">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Win Rate:</span>
                <span class="metric-value">{:.2}%</span>
            </div>
        </div>
    </div>
</body>
</html>"#,
            self.title,
            report.strategy_name,
            report.initial_capital,
            report.final_capital,
            if report.total_return > 0.0 { "positive" } else { "negative" },
            report.total_return,
            report.total_return_percent,
            report.max_drawdown,
            report.sharpe_ratio,
            report.total_trades,
            report.winning_trades,
            report.losing_trades,
            report.win_rate,
        );

        let mut file = File::create(filename)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
struct BacktestSummary {
    strategy_name: String,
    initial_capital: f64,
    final_capital: f64,
    total_return: f64,
    total_return_percent: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    win_rate: f64,
}

fn main() -> Result<()> {
    let summary = BacktestSummary {
        strategy_name: "RSI Mean Reversion".to_string(),
        initial_capital: 100_000.0,
        final_capital: 135_500.0,
        total_return: 35_500.0,
        total_return_percent: 35.5,
        max_drawdown: -8_200.0,
        sharpe_ratio: 2.1,
        total_trades: 200,
        winning_trades: 135,
        losing_trades: 65,
        win_rate: 67.5,
    };

    let generator = HtmlReportGenerator::new("Backtest Report".to_string());
    generator.generate(&summary, "backtest_report.html")?;

    println!("HTML report generated: backtest_report.html");
    Ok(())
}
```

## Comprehensive Reporting System

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Result};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    timestamp: String,
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
    commission: f64,
}

#[derive(Debug)]
struct ComprehensiveReport {
    trades: Vec<Trade>,
    initial_capital: f64,
}

impl ComprehensiveReport {
    fn new(trades: Vec<Trade>, initial_capital: f64) -> Self {
        ComprehensiveReport {
            trades,
            initial_capital,
        }
    }

    fn calculate_metrics(&self) -> ReportMetrics {
        let total_trades = self.trades.len() as u32;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl = 0.0;
        let mut total_commission = 0.0;

        let mut symbol_performance: HashMap<String, f64> = HashMap::new();

        for trade in &self.trades {
            total_pnl += trade.pnl;
            total_commission += trade.commission;

            if trade.pnl > 0.0 {
                winning_trades += 1;
            } else if trade.pnl < 0.0 {
                losing_trades += 1;
            }

            *symbol_performance.entry(trade.symbol.clone()).or_insert(0.0) += trade.pnl;
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let final_capital = self.initial_capital + total_pnl - total_commission;
        let total_return_percent = ((final_capital - self.initial_capital) / self.initial_capital) * 100.0;

        ReportMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            total_commission,
            win_rate,
            final_capital,
            total_return_percent,
            symbol_performance,
        }
    }

    fn generate_text_report(&self, filename: &str) -> Result<()> {
        let metrics = self.calculate_metrics();
        let mut file = File::create(filename)?;

        writeln!(file, "=")?;
        writeln!(file, "=== BACKTEST REPORT ===")?;
        writeln!(file, "=======================")?;
        writeln!(file)?;

        writeln!(file, "CAPITAL OVERVIEW")?;
        writeln!(file, "Initial Capital: ${:.2}", self.initial_capital)?;
        writeln!(file, "Final Capital: ${:.2}", metrics.final_capital)?;
        writeln!(file, "Total Return: {:.2}%", metrics.total_return_percent)?;
        writeln!(file)?;

        writeln!(file, "TRADE STATISTICS")?;
        writeln!(file, "Total Trades: {}", metrics.total_trades)?;
        writeln!(file, "Winning Trades: {}", metrics.winning_trades)?;
        writeln!(file, "Losing Trades: {}", metrics.losing_trades)?;
        writeln!(file, "Win Rate: {:.2}%", metrics.win_rate)?;
        writeln!(file)?;

        writeln!(file, "P&L BREAKDOWN")?;
        writeln!(file, "Gross P&L: ${:.2}", metrics.total_pnl)?;
        writeln!(file, "Total Commission: ${:.2}", metrics.total_commission)?;
        writeln!(file, "Net P&L: ${:.2}", metrics.total_pnl - metrics.total_commission)?;
        writeln!(file)?;

        writeln!(file, "PERFORMANCE BY SYMBOL")?;
        for (symbol, pnl) in &metrics.symbol_performance {
            writeln!(file, "{}: ${:.2}", symbol, pnl)?;
        }

        Ok(())
    }

    fn print_summary(&self) {
        let metrics = self.calculate_metrics();
        println!("\n=== BACKTEST SUMMARY ===");
        println!("Trades: {}", metrics.total_trades);
        println!("Win Rate: {:.2}%", metrics.win_rate);
        println!("Total Return: {:.2}%", metrics.total_return_percent);
        println!("Final Capital: ${:.2}", metrics.final_capital);
    }
}

#[derive(Debug)]
struct ReportMetrics {
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    total_pnl: f64,
    total_commission: f64,
    win_rate: f64,
    final_capital: f64,
    total_return_percent: f64,
    symbol_performance: HashMap<String, f64>,
}

fn main() -> Result<()> {
    let trades = vec![
        Trade {
            id: 1,
            timestamp: "2024-01-01 09:30:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 1.0,
            entry_price: 40000.0,
            exit_price: 42000.0,
            pnl: 2000.0,
            commission: 40.0,
        },
        Trade {
            id: 2,
            timestamp: "2024-01-02 14:15:00".to_string(),
            symbol: "ETH".to_string(),
            side: "LONG".to_string(),
            quantity: 10.0,
            entry_price: 2200.0,
            exit_price: 2300.0,
            pnl: 1000.0,
            commission: 22.0,
        },
        Trade {
            id: 3,
            timestamp: "2024-01-03 11:45:00".to_string(),
            symbol: "BTC".to_string(),
            side: "SHORT".to_string(),
            quantity: 0.5,
            entry_price: 41000.0,
            exit_price: 42000.0,
            pnl: -500.0,
            commission: 20.5,
        },
    ];

    let report = ComprehensiveReport::new(trades, 100_000.0);

    report.print_summary();
    report.generate_text_report("backtest_summary.txt")?;

    println!("\nDetailed report saved to backtest_summary.txt");

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Report Structure | Organizing data with structs for clear representation |
| Display Trait | Implementing `fmt::Display` for custom formatting |
| Metrics Calculation | Computing win rate, drawdown, returns from raw data |
| JSON Export | Using `serde` for structured data serialization |
| CSV Export | Writing tabular data for spreadsheet analysis |
| HTML Generation | Creating visual reports with embedded styling |
| File I/O | Saving reports to various file formats |

## Homework

1. **Monthly Performance Report**: Create a report that breaks down P&L by month. Calculate:
   - Monthly returns
   - Best month
   - Worst month
   - Average monthly return
   - Volatility (standard deviation of monthly returns)

2. **Per-Symbol Analysis**: Generate a report showing performance broken down by trading symbol:
   - Total trades per symbol
   - Win rate per symbol
   - Average P&L per symbol
   - Best performing symbol
   - Worst performing symbol

3. **Trade Duration Analysis**: Add trade duration tracking and create a report showing:
   - Average trade duration
   - Shortest trade
   - Longest trade
   - Relationship between duration and profitability

4. **Multi-Format Reporter**: Create a `ReportGenerator` trait that can export to multiple formats:
   ```rust
   trait ReportGenerator {
       fn to_json(&self) -> String;
       fn to_csv(&self) -> String;
       fn to_html(&self) -> String;
       fn to_text(&self) -> String;
   }
   ```
   Implement it for a `BacktestReport` struct that includes all major metrics.

## Navigation

[← Previous day](../287-performance-metrics/en.md) | [Next day →](../289-visualization/en.md)
