use std::collections::HashMap;

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

pub struct AdvancedWinRateAnalyzer {
    trades: Vec<Trade>,
}

impl AdvancedWinRateAnalyzer {
    pub fn new(trades: Vec<Trade>) -> Self {
        Self { trades }
    }

    pub fn overall_win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let winning = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning as f64 / self.trades.len() as f64) * 100.0
    }

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

    pub fn profit_factor(&self) -> f64 {
        let avg_win = self.average_win();
        let avg_loss = self.average_loss().abs();

        if avg_loss == 0.0 {
            return f64::INFINITY;
        }

        avg_win / avg_loss
    }

    pub fn report(&self) {
        println!("=== Win Rate Analysis ===");
        println!("Overall Win Rate: {:.2}%", self.overall_win_rate());
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
