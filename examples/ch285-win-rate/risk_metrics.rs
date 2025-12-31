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

#[derive(Debug)]
pub struct RiskMetrics {
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub expectancy: f64,
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
