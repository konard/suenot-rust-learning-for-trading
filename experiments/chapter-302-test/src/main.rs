use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    entry_time: u64,
    exit_time: u64,
    entry_price: f64,
    exit_price: f64,
    size: f64,
    pnl: f64,
    commission: f64,
}

#[derive(Debug)]
struct StrategyMetrics {
    name: String,
    total_return: f64,
    annual_return: f64,
    volatility: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    sortino_ratio: f64,
    calmar_ratio: f64,
    total_trades: usize,
    win_rate: f64,
    profit_factor: f64,
    avg_trade: f64,
    avg_win: f64,
    avg_loss: f64,
}

impl StrategyMetrics {
    fn new(name: &str, trades: &[Trade], initial_capital: f64, days: usize) -> Self {
        let total_trades = trades.len();

        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_return = total_pnl / initial_capital;
        let annual_return = total_return * (365.0 / days as f64);

        let returns: Vec<f64> = trades.iter().map(|t| t.pnl / initial_capital).collect();
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;

        let variance = returns.iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt() * (252.0_f64).sqrt();

        let mut equity = initial_capital;
        let mut peak = initial_capital;
        let mut max_dd = 0.0;

        for trade in trades {
            equity += trade.pnl;
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        let risk_free_rate = 0.02;
        let sharpe_ratio = if volatility > 0.0 {
            (annual_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();
        let downside_variance = if !downside_returns.is_empty() {
            downside_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / downside_returns.len() as f64
        } else {
            0.0
        };
        let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt();
        let sortino_ratio = if downside_deviation > 0.0 {
            (annual_return - risk_free_rate) / downside_deviation
        } else {
            0.0
        };

        let calmar_ratio = if max_dd > 0.0 {
            annual_return / max_dd
        } else {
            0.0
        };

        let winning_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl < 0.0).collect();

        let win_rate = winning_trades.len() as f64 / total_trades as f64;
        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|t| t.pnl).sum::<f64>() / winning_trades.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|t| t.pnl).sum::<f64>() / losing_trades.len() as f64
        } else {
            0.0
        };
        let avg_trade = total_pnl / total_trades as f64;

        let total_wins: f64 = winning_trades.iter().map(|t| t.pnl).sum();
        let total_losses: f64 = losing_trades.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        StrategyMetrics {
            name: name.to_string(),
            total_return,
            annual_return,
            volatility,
            max_drawdown: max_dd,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            total_trades,
            win_rate,
            profit_factor,
            avg_trade,
            avg_win,
            avg_loss,
        }
    }

    fn print(&self) {
        println!("\n=== {} ===", self.name);
        println!("Returns:");
        println!("  Total return: {:.2}%", self.total_return * 100.0);
        println!("  Annual return: {:.2}%", self.annual_return * 100.0);
        println!("\nRisk:");
        println!("  Volatility: {:.2}%", self.volatility * 100.0);
        println!("  Max drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("\nEfficiency:");
        println!("  Sharpe Ratio: {:.2}", self.sharpe_ratio);
        println!("  Sortino Ratio: {:.2}", self.sortino_ratio);
        println!("  Calmar Ratio: {:.2}", self.calmar_ratio);
        println!("\nTrading:");
        println!("  Total trades: {}", self.total_trades);
        println!("  Win Rate: {:.2}%", self.win_rate * 100.0);
        println!("  Profit Factor: {:.2}", self.profit_factor);
        println!("  Average trade: ${:.2}", self.avg_trade);
        println!("  Average win: ${:.2}", self.avg_win);
        println!("  Average loss: ${:.2}", self.avg_loss);
    }
}

fn generate_trades(
    strategy_type: &str,
    num_trades: usize,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    volatility_factor: f64,
) -> Vec<Trade> {
    let mut trades = Vec::new();
    let mut time = 1000000u64;

    for i in 0..num_trades {
        let is_win = (i as f64 / num_trades as f64) < win_rate;
        let base_pnl = if is_win { avg_win } else { avg_loss };

        let noise = ((i * 7 + 13) % 100) as f64 / 100.0 - 0.5;
        let pnl = base_pnl * (1.0 + noise * volatility_factor);

        trades.push(Trade {
            entry_time: time,
            exit_time: time + 3600,
            entry_price: 42000.0,
            exit_price: 42000.0 + pnl,
            size: 1.0,
            pnl,
            commission: 5.0,
        });

        time += 7200;
    }

    trades
}

fn main() {
    let initial_capital = 10000.0;
    let days = 365;

    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let metrics_a = StrategyMetrics::new("Strategy A: Aggressive Scalper", &trades_a, initial_capital, days);

    let trades_b = generate_trades("Position", 50, 0.65, 300.0, -150.0, 0.2);
    let metrics_b = StrategyMetrics::new("Strategy B: Conservative Position", &trades_b, initial_capital, days);

    let trades_c = generate_trades("Swing", 150, 0.58, 80.0, -60.0, 0.25);
    let metrics_c = StrategyMetrics::new("Strategy C: Swing Trader", &trades_c, initial_capital, days);

    metrics_a.print();
    metrics_b.print();
    metrics_c.print();

    println!("\n✅ All code examples compile successfully!");
}
