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
