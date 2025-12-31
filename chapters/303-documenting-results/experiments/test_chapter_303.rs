// Test script for Chapter 303 code examples
// This combines all the code examples to verify they compile

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct BacktestMetadata {
    test_id: String,
    strategy_name: String,
    code_version: String,
    timestamp: DateTime<Utc>,
    author: String,
    description: String,
}

impl BacktestMetadata {
    fn new(strategy_name: &str, description: &str) -> Self {
        Self {
            test_id: "test-uuid-12345".to_string(), // Simplified for testing
            strategy_name: strategy_name.to_string(),
            code_version: "abc123def".to_string(),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StrategyParameters {
    ma_short_period: usize,
    ma_long_period: usize,
    stop_loss_pct: f64,
    take_profit_pct: f64,
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

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    total_return: f64,
    annual_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    profit_factor: f64,
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    avg_profit_per_trade: f64,
    max_profit: f64,
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

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    entry_time: String,
    exit_time: String,
    symbol: String,
    side: String,
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
    equity_curve: Vec<f64>,
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

    fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(filename, json)?;
        println!("✅ Report saved to: {}", filename);
        Ok(())
    }

    fn generate_text_report(&self) -> String {
        let mut report = String::new();
        report.push_str("═══════════════════════════════════════════════════\n");
        report.push_str("           BACKTEST STRATEGY REPORT\n");
        report.push_str("═══════════════════════════════════════════════════\n\n");
        report.push_str(&format!("Test ID: {}\n", self.metadata.test_id));
        report.push_str(&format!("Strategy: {}\n", self.metadata.strategy_name));
        report
    }

    fn print_summary(&self) {
        println!("{}", self.generate_text_report());
    }

    fn generate_markdown_report(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Backtest Report: {}\n\n", self.metadata.strategy_name));
        md.push_str("## Metadata\n\n");
        md.push_str(&format!("- **Test ID**: `{}`\n", self.metadata.test_id));
        md
    }
}

fn main() {
    println!("Testing Chapter 303 code examples...\n");

    // Test 1: Metadata
    let metadata = BacktestMetadata::new(
        "MA Crossover v2.1",
        "Test with new stop-loss parameters"
    );
    metadata.print();

    // Test 2: Parameters
    let params = StrategyParameters {
        ma_short_period: 10,
        ma_long_period: 50,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.05,
        max_position_size: 0.10,
    };
    params.print();

    // Test 3: Metrics
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

    // Test 4: Full report
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
    ];

    let equity_curve = vec![10000.0, 10150.0, 10300.0, 10250.0, 10500.0];

    let report = BacktestReport::new(
        metadata,
        params.clone(),
        metrics,
        trades,
        equity_curve
    );

    report.print_summary();

    println!("\n✅ All code examples compiled and ran successfully!");
}
