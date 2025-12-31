# Day 303: Documenting Results

## Trading Analogy

Imagine a trader who tested dozens of strategies, got hundreds of results, but didn't write down a single one. A month later, they can't remember:
- Which parameters gave the best results?
- Which instruments did the strategy work on?
- What were the maximum drawdowns?
- Why was one strategy rejected?

It's like keeping a trading journal but forgetting to write in it. Test results without documentation are wasted time and knowledge.

**Documenting results** in algorithmic trading is:
- Systematic recording of all backtests
- Saving parameters and metrics
- Analysis and comparison of strategies
- Knowledge base for future decisions

Like a scientific experiment: without records, it's impossible to reproduce the result or understand what went wrong.

## Why document backtesting results?

In professional algorithmic trading, documentation is not optional but a mandatory practice:

| Reason | Why it's needed | Example |
|--------|----------------|---------|
| **Reproducibility** | Repeat a successful test a month later | Recorded RNG seed and parameters |
| **Comparison** | Choose the best of 10 strategies | Table with Sharpe ratios of all tests |
| **Audit** | Explain to regulator or investor | Full report for each trade |
| **Learning** | Understand why a strategy failed | Logs showed look-ahead bias |
| **Versioning** | Track changes in strategy code | Git commit hash in every report |

## What to document?

### 1. Test Metadata

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct BacktestMetadata {
    /// Unique test ID
    test_id: String,
    /// Strategy name
    strategy_name: String,
    /// Strategy code version (git commit)
    code_version: String,
    /// Test start time
    timestamp: DateTime<Utc>,
    /// Who ran the test
    author: String,
    /// Test purpose description
    description: String,
}

impl BacktestMetadata {
    fn new(strategy_name: &str, description: &str) -> Self {
        Self {
            test_id: uuid::Uuid::new_v4().to_string(),
            strategy_name: strategy_name.to_string(),
            code_version: "abc123def".to_string(), // In reality: git rev-parse HEAD
            timestamp: Utc::now(),
            author: "trading-bot".to_string(),
            description: description.to_string(),
        }
    }

    fn print(&self) {
        println!("=== Backtest Metadata ===");
        println!("ID: {}", self.test_id);
        println!("Strategy: {}", self.strategy_name);
        println!("Code Version: {}", self.code_version);
        println!("Time: {}", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Author: {}", self.author);
        println!("Description: {}", self.description);
    }
}

fn main() {
    let metadata = BacktestMetadata::new(
        "MA Crossover v2.1",
        "Test with new stop-loss parameters"
    );
    metadata.print();
}
```

### 2. Strategy Parameters

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
struct StrategyParameters {
    /// Short moving average period
    ma_short_period: usize,
    /// Long moving average period
    ma_long_period: usize,
    /// Stop-loss percentage
    stop_loss_pct: f64,
    /// Take-profit percentage
    take_profit_pct: f64,
    /// Maximum position size (% of capital)
    max_position_size: f64,
}

impl StrategyParameters {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn print(&self) {
        println!("\n=== Strategy Parameters ===");
        println!("MA Short: {}", self.ma_short_period);
        println!("MA Long: {}", self.ma_long_period);
        println!("Stop Loss: {:.2}%", self.stop_loss_pct * 100.0);
        println!("Take Profit: {:.2}%", self.take_profit_pct * 100.0);
        println!("Max Position Size: {:.2}%", self.max_position_size * 100.0);
    }
}

fn main() {
    let params = StrategyParameters {
        ma_short_period: 10,
        ma_long_period: 50,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.05,
        max_position_size: 0.10,
    };

    params.print();
    println!("\nJSON representation:");
    println!("{}", params.to_json());
}
```

### 3. Performance Metrics

```rust
#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    /// Total return
    total_return: f64,
    /// Annualized return
    annual_return: f64,
    /// Sharpe ratio
    sharpe_ratio: f64,
    /// Maximum drawdown
    max_drawdown: f64,
    /// Win rate (percentage of profitable trades)
    win_rate: f64,
    /// Profit factor
    profit_factor: f64,
    /// Total number of trades
    total_trades: usize,
    /// Number of winning trades
    winning_trades: usize,
    /// Number of losing trades
    losing_trades: usize,
    /// Average profit per trade
    avg_profit_per_trade: f64,
    /// Maximum profit
    max_profit: f64,
    /// Maximum loss
    max_loss: f64,
}

impl PerformanceMetrics {
    fn print(&self) {
        println!("\n=== Performance Metrics ===");
        println!("Total Return: {:.2}%", self.total_return * 100.0);
        println!("Annual Return: {:.2}%", self.annual_return * 100.0);
        println!("Sharpe Ratio: {:.2}", self.sharpe_ratio);
        println!("Max Drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("Win Rate: {:.2}%", self.win_rate * 100.0);
        println!("Profit Factor: {:.2}", self.profit_factor);
        println!("\n=== Trade Statistics ===");
        println!("Total Trades: {}", self.total_trades);
        println!("Winning: {}", self.winning_trades);
        println!("Losing: {}", self.losing_trades);
        println!("Avg Profit: {:.2}%", self.avg_profit_per_trade * 100.0);
        println!("Max Profit: {:.2}%", self.max_profit * 100.0);
        println!("Max Loss: {:.2}%", self.max_loss * 100.0);
    }

    fn grade(&self) -> &str {
        if self.sharpe_ratio > 2.0 && self.max_drawdown.abs() < 0.15 {
            "Excellent ✅"
        } else if self.sharpe_ratio > 1.0 && self.max_drawdown.abs() < 0.25 {
            "Good 👍"
        } else if self.sharpe_ratio > 0.5 {
            "Acceptable ⚠️"
        } else {
            "Unsatisfactory ❌"
        }
    }
}

fn main() {
    let metrics = PerformanceMetrics {
        total_return: 0.45,
        annual_return: 0.18,
        sharpe_ratio: 1.6,
        max_drawdown: -0.12,
        win_rate: 0.58,
        profit_factor: 1.8,
        total_trades: 150,
        winning_trades: 87,
        losing_trades: 63,
        avg_profit_per_trade: 0.003,
        max_profit: 0.08,
        max_loss: -0.05,
    };

    metrics.print();
    println!("\nGrade: {}", metrics.grade());
}
```

## Complete Backtest Report

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    entry_time: String,
    exit_time: String,
    symbol: String,
    side: String,  // "LONG" or "SHORT"
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    pnl_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BacktestReport {
    metadata: BacktestMetadata,
    parameters: StrategyParameters,
    metrics: PerformanceMetrics,
    trades: Vec<Trade>,
    equity_curve: Vec<f64>,  // Daily equity
}

impl BacktestReport {
    fn new(
        metadata: BacktestMetadata,
        parameters: StrategyParameters,
        metrics: PerformanceMetrics,
        trades: Vec<Trade>,
        equity_curve: Vec<f64>,
    ) -> Self {
        Self {
            metadata,
            parameters,
            metrics,
            trades,
            equity_curve,
        }
    }

    /// Save report to JSON file
    fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(filename, json)?;
        println!("✅ Report saved to: {}", filename);
        Ok(())
    }

    /// Load report from JSON file
    fn load_from_file(filename: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let report: BacktestReport = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(report)
    }

    /// Generate text report
    fn generate_text_report(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════\n");
        report.push_str("           BACKTEST STRATEGY REPORT\n");
        report.push_str("═══════════════════════════════════════════════════\n\n");

        // Metadata
        report.push_str(&format!("Test ID: {}\n", self.metadata.test_id));
        report.push_str(&format!("Strategy: {}\n", self.metadata.strategy_name));
        report.push_str(&format!("Code Version: {}\n", self.metadata.code_version));
        report.push_str(&format!("Date: {}\n", self.metadata.timestamp.format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("Description: {}\n\n", self.metadata.description));

        // Parameters
        report.push_str("─── Strategy Parameters ───\n");
        report.push_str(&format!("MA Short: {}\n", self.parameters.ma_short_period));
        report.push_str(&format!("MA Long: {}\n", self.parameters.ma_long_period));
        report.push_str(&format!("Stop Loss: {:.2}%\n", self.parameters.stop_loss_pct * 100.0));
        report.push_str(&format!("Take Profit: {:.2}%\n\n", self.parameters.take_profit_pct * 100.0));

        // Metrics
        report.push_str("─── Results ───\n");
        report.push_str(&format!("Total Return: {:.2}%\n", self.metrics.total_return * 100.0));
        report.push_str(&format!("Sharpe Ratio: {:.2}\n", self.metrics.sharpe_ratio));
        report.push_str(&format!("Max Drawdown: {:.2}%\n", self.metrics.max_drawdown * 100.0));
        report.push_str(&format!("Win Rate: {:.2}%\n", self.metrics.win_rate * 100.0));
        report.push_str(&format!("Total Trades: {}\n", self.metrics.total_trades));
        report.push_str(&format!("Grade: {}\n\n", self.metrics.grade()));

        report.push_str("═══════════════════════════════════════════════════\n");

        report
    }

    fn print_summary(&self) {
        println!("{}", self.generate_text_report());
    }
}

fn main() {
    // Create sample report
    let metadata = BacktestMetadata::new(
        "MA Crossover v2.1",
        "Test on BTC/USDT with optimized parameters"
    );

    let parameters = StrategyParameters {
        ma_short_period: 10,
        ma_long_period: 50,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.05,
        max_position_size: 0.10,
    };

    let metrics = PerformanceMetrics {
        total_return: 0.45,
        annual_return: 0.18,
        sharpe_ratio: 1.6,
        max_drawdown: -0.12,
        win_rate: 0.58,
        profit_factor: 1.8,
        total_trades: 150,
        winning_trades: 87,
        losing_trades: 63,
        avg_profit_per_trade: 0.003,
        max_profit: 0.08,
        max_loss: -0.05,
    };

    let trades = vec![
        Trade {
            entry_time: "2024-01-15 10:30:00".to_string(),
            exit_time: "2024-01-15 14:20:00".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: "LONG".to_string(),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.1,
            pnl: 150.0,
            pnl_pct: 0.0357,
        },
        // ... more trades
    ];

    let equity_curve = vec![10000.0, 10150.0, 10300.0, 10250.0, 10500.0];

    let report = BacktestReport::new(metadata, parameters, metrics, trades, equity_curve);

    // Print summary
    report.print_summary();

    // Save to file
    let filename = format!("backtest_report_{}.json", report.metadata.test_id);
    report.save_to_file(&filename).unwrap();
}
```

## Results Database

For storing multiple tests, you can use a simple file system or database:

```rust
use std::fs;
use std::path::Path;

struct BacktestDatabase {
    storage_path: String,
}

impl BacktestDatabase {
    fn new(storage_path: &str) -> Self {
        // Create storage folder if it doesn't exist
        fs::create_dir_all(storage_path).ok();
        Self {
            storage_path: storage_path.to_string(),
        }
    }

    /// Save report to database
    fn save_report(&self, report: &BacktestReport) -> std::io::Result<String> {
        let filename = format!(
            "{}/{}_{}_{}.json",
            self.storage_path,
            report.metadata.timestamp.format("%Y%m%d_%H%M%S"),
            report.metadata.strategy_name.replace(" ", "_"),
            &report.metadata.test_id[..8]
        );

        report.save_to_file(&filename)?;
        Ok(filename)
    }

    /// Load all reports
    fn load_all_reports(&self) -> Vec<BacktestReport> {
        let mut reports = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Ok(report) = BacktestReport::load_from_file(
                            entry.path().to_str().unwrap()
                        ) {
                            reports.push(report);
                        }
                    }
                }
            }
        }

        reports
    }

    /// Find best strategies by Sharpe ratio
    fn find_best_strategies(&self, top_n: usize) -> Vec<BacktestReport> {
        let mut reports = self.load_all_reports();
        reports.sort_by(|a, b| {
            b.metrics.sharpe_ratio
                .partial_cmp(&a.metrics.sharpe_ratio)
                .unwrap()
        });
        reports.truncate(top_n);
        reports
    }

    /// Statistics for all tests
    fn print_statistics(&self) {
        let reports = self.load_all_reports();

        println!("=== Backtest Database Statistics ===");
        println!("Total tests: {}", reports.len());

        if reports.is_empty() {
            return;
        }

        let avg_sharpe: f64 = reports.iter()
            .map(|r| r.metrics.sharpe_ratio)
            .sum::<f64>() / reports.len() as f64;

        let avg_return: f64 = reports.iter()
            .map(|r| r.metrics.total_return)
            .sum::<f64>() / reports.len() as f64;

        println!("Average Sharpe: {:.2}", avg_sharpe);
        println!("Average Return: {:.2}%", avg_return * 100.0);

        println!("\nTop-3 strategies by Sharpe:");
        for (i, report) in self.find_best_strategies(3).iter().enumerate() {
            println!("  {}. {} - Sharpe: {:.2}",
                i + 1,
                report.metadata.strategy_name,
                report.metrics.sharpe_ratio
            );
        }
    }
}

fn main() {
    let db = BacktestDatabase::new("./backtest_results");

    // Save several reports (examples)
    println!("Saving test reports...\n");

    // ... creating and saving reports

    // Statistics
    db.print_statistics();
}
```

## Automatic Markdown Report Generation

```rust
impl BacktestReport {
    /// Generate Markdown report for documentation
    fn generate_markdown_report(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Backtest Report: {}\n\n", self.metadata.strategy_name));

        md.push_str("## Metadata\n\n");
        md.push_str(&format!("- **Test ID**: `{}`\n", self.metadata.test_id));
        md.push_str(&format!("- **Strategy**: {}\n", self.metadata.strategy_name));
        md.push_str(&format!("- **Code Version**: `{}`\n", self.metadata.code_version));
        md.push_str(&format!("- **Date**: {}\n", self.metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        md.push_str(&format!("- **Author**: {}\n", self.metadata.author));
        md.push_str(&format!("- **Description**: {}\n\n", self.metadata.description));

        md.push_str("## Strategy Parameters\n\n");
        md.push_str("```json\n");
        md.push_str(&self.parameters.to_json());
        md.push_str("\n```\n\n");

        md.push_str("## Performance Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total Return | {:.2}% |\n", self.metrics.total_return * 100.0));
        md.push_str(&format!("| Annual Return | {:.2}% |\n", self.metrics.annual_return * 100.0));
        md.push_str(&format!("| Sharpe Ratio | {:.2} |\n", self.metrics.sharpe_ratio));
        md.push_str(&format!("| Max Drawdown | {:.2}% |\n", self.metrics.max_drawdown * 100.0));
        md.push_str(&format!("| Win Rate | {:.2}% |\n", self.metrics.win_rate * 100.0));
        md.push_str(&format!("| Profit Factor | {:.2} |\n", self.metrics.profit_factor));
        md.push_str(&format!("| Total Trades | {} |\n", self.metrics.total_trades));
        md.push_str(&format!("| Winning Trades | {} |\n", self.metrics.winning_trades));
        md.push_str(&format!("| Losing Trades | {} |\n\n", self.metrics.losing_trades));

        md.push_str(&format!("**Grade**: {}\n\n", self.metrics.grade()));

        md.push_str("## Trade Summary\n\n");
        md.push_str(&format!("Total trades: {}\n\n", self.trades.len()));

        if !self.trades.is_empty() {
            md.push_str("### First 5 Trades\n\n");
            md.push_str("| Entry Time | Symbol | Side | Entry Price | Exit Price | P&L % |\n");
            md.push_str("|------------|--------|------|-------------|------------|-------|\n");

            for trade in self.trades.iter().take(5) {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.2} | {:.2} | {:.2}% |\n",
                    trade.entry_time,
                    trade.symbol,
                    trade.side,
                    trade.entry_price,
                    trade.exit_price,
                    trade.pnl_pct * 100.0
                ));
            }
            md.push_str("\n");
        }

        md
    }

    fn save_markdown_report(&self, filename: &str) -> std::io::Result<()> {
        let markdown = self.generate_markdown_report();
        std::fs::write(filename, markdown)?;
        println!("✅ Markdown report saved: {}", filename);
        Ok(())
    }
}

fn main() {
    // ... creating report

    let report = BacktestReport::new(
        BacktestMetadata::new("MA Crossover", "Test description"),
        StrategyParameters {
            ma_short_period: 10,
            ma_long_period: 50,
            stop_loss_pct: 0.02,
            take_profit_pct: 0.05,
            max_position_size: 0.10,
        },
        PerformanceMetrics {
            total_return: 0.45,
            annual_return: 0.18,
            sharpe_ratio: 1.6,
            max_drawdown: -0.12,
            win_rate: 0.58,
            profit_factor: 1.8,
            total_trades: 150,
            winning_trades: 87,
            losing_trades: 63,
            avg_profit_per_trade: 0.003,
            max_profit: 0.08,
            max_loss: -0.05,
        },
        vec![],
        vec![],
    );

    // Save Markdown report
    report.save_markdown_report("backtest_report.md").unwrap();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Metadata** | Test ID, code version, time, author, description |
| **Parameters** | All strategy settings in structured form |
| **Metrics** | Sharpe, return, drawdown, win rate, and others |
| **Trades** | Complete journal of all trades with details |
| **Equity Curve** | Chart of capital changes over time |
| **JSON** | Standard format for data serialization |
| **Database** | Storage of all backtests for comparison |
| **Markdown** | Human-readable reports for documentation |

## Homework

1. **Extended Report**: Add to `BacktestReport`:
   - List of all used indicators with their parameters
   - Information about traded instruments (symbols, timeframes)
   - Monthly statistics (monthly returns)
   - Backtest execution time
   - System information (OS, Rust version)

2. **Comparative Report**: Create a `compare_reports` function:
   - Takes 2-5 backtest reports
   - Creates a comparison table of all metrics
   - Highlights best values in each category
   - Generates recommendation on which strategy to choose
   - Saves to Markdown with nice formatting

3. **HTML Dashboard**: Generate an HTML page with:
   - Equity curve visualization (can use plotly or Chart.js)
   - Metrics tables
   - List of all trades
   - Filter by profitable/losing trades
   - Interactive drawdown charts

4. **Automatic Report Sending**: Implement a notification system:
   - After each backtest, send brief report to Telegram/Slack
   - When detecting strategy with Sharpe > 2.0 — urgent notification
   - Weekly summary of all tests for the week
   - Automatic GitHub issue creation when test fails

## Navigation

[← Previous Day](../294-overfitting-strategy-optimization/en.md) | [Next Day →](../304-project-backtesting-engine/en.md)
