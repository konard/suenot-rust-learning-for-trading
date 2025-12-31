// Test compilation of code examples from Chapter 213

use std::fs;
use std::collections::HashMap;

/// Trading journal with file persistence
/// (In a real project, use a database!)
#[derive(Debug)]
struct TradingJournal {
    trades: Vec<Trade>,
    portfolio: Portfolio,
    filename: String,
}

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: TradeSide,
    price: f64,
    quantity: f64,
    timestamp: i64,
}

#[derive(Debug, Clone)]
enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Portfolio {
    cash: f64,
    positions: HashMap<String, f64>,
}

impl TradingJournal {
    /// Creates a journal or loads an existing one
    fn new(filename: &str) -> Self {
        // Try to load existing data
        if let Ok(data) = fs::read_to_string(filename) {
            if let Some(journal) = Self::deserialize(&data) {
                println!("Loaded journal: {} trades, balance ${:.2}",
                    journal.trades.len(), journal.portfolio.cash);
                return journal;
            }
        }

        // Create new journal
        println!("Created new journal");
        TradingJournal {
            trades: Vec::new(),
            portfolio: Portfolio {
                cash: 100_000.0,  // Starting capital
                positions: HashMap::new(),
            },
            filename: filename.to_string(),
        }
    }

    /// Adds a trade and saves
    fn add_trade(&mut self, symbol: &str, side: TradeSide,
                 price: f64, quantity: f64) -> Result<u64, String> {
        let cost = price * quantity;

        // Check if trade is possible
        match &side {
            TradeSide::Buy => {
                if self.portfolio.cash < cost {
                    return Err(format!("Insufficient funds: need ${:.2}, have ${:.2}",
                        cost, self.portfolio.cash));
                }
            }
            TradeSide::Sell => {
                let position = self.portfolio.positions.get(symbol).unwrap_or(&0.0);
                if *position < quantity {
                    return Err(format!("Insufficient {}: need {}, have {}",
                        symbol, quantity, position));
                }
            }
        }

        // Create trade
        let trade = Trade {
            id: self.trades.len() as u64 + 1,
            symbol: symbol.to_string(),
            side: side.clone(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        // Update portfolio
        match side {
            TradeSide::Buy => {
                self.portfolio.cash -= cost;
                *self.portfolio.positions
                    .entry(symbol.to_string())
                    .or_insert(0.0) += quantity;
            }
            TradeSide::Sell => {
                self.portfolio.cash += cost;
                if let Some(pos) = self.portfolio.positions.get_mut(symbol) {
                    *pos -= quantity;
                    if *pos <= 0.0 {
                        self.portfolio.positions.remove(symbol);
                    }
                }
            }
        }

        let trade_id = trade.id;
        self.trades.push(trade);

        // IMPORTANT: save after each operation
        self.save()?;

        Ok(trade_id)
    }

    /// Saves journal to file
    fn save(&self) -> Result<(), String> {
        let data = self.serialize();
        fs::write(&self.filename, data)
            .map_err(|e| format!("Save error: {}", e))
    }

    /// Serialization to simple text format
    fn serialize(&self) -> String {
        let mut lines = Vec::new();

        // Portfolio header
        lines.push(format!("CASH:{}", self.portfolio.cash));
        for (symbol, qty) in &self.portfolio.positions {
            lines.push(format!("POS:{}:{}", symbol, qty));
        }

        // Trades
        lines.push("TRADES".to_string());
        for trade in &self.trades {
            let side_str = match trade.side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            };
            lines.push(format!("{}:{}:{}:{}:{}:{}",
                trade.id, trade.symbol, side_str,
                trade.price, trade.quantity, trade.timestamp));
        }

        lines.join("\n")
    }

    /// Deserialization from text
    fn deserialize(data: &str) -> Option<Self> {
        let mut cash = 100_000.0;
        let mut positions = HashMap::new();
        let mut trades = Vec::new();
        let mut reading_trades = false;

        for line in data.lines() {
            if line.starts_with("CASH:") {
                cash = line[5..].parse().ok()?;
            } else if line.starts_with("POS:") {
                let parts: Vec<&str> = line[4..].split(':').collect();
                if parts.len() == 2 {
                    positions.insert(
                        parts[0].to_string(),
                        parts[1].parse().ok()?
                    );
                }
            } else if line == "TRADES" {
                reading_trades = true;
            } else if reading_trades {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 6 {
                    let side = match parts[2] {
                        "BUY" => TradeSide::Buy,
                        "SELL" => TradeSide::Sell,
                        _ => continue,
                    };
                    trades.push(Trade {
                        id: parts[0].parse().ok()?,
                        symbol: parts[1].to_string(),
                        side,
                        price: parts[3].parse().ok()?,
                        quantity: parts[4].parse().ok()?,
                        timestamp: parts[5].parse().ok()?,
                    });
                }
            }
        }

        Some(TradingJournal {
            trades,
            portfolio: Portfolio { cash, positions },
            filename: String::new(),
        })
    }

    /// Shows status
    fn status(&self) {
        println!("\n=== Portfolio Status ===");
        println!("Cash: ${:.2}", self.portfolio.cash);
        println!("Positions:");
        for (symbol, qty) in &self.portfolio.positions {
            println!("  {}: {:.4}", symbol, qty);
        }
        println!("Total trades: {}", self.trades.len());

        if let Some(last) = self.trades.last() {
            let side = match last.side {
                TradeSide::Buy => "Buy",
                TradeSide::Sell => "Sell",
            };
            println!("Last trade: {} {} {} @ ${:.2}",
                side, last.quantity, last.symbol, last.price);
        }
    }
}

fn main() {
    // Journal is saved to file and loaded on restart
    let mut journal = TradingJournal::new("trading_journal.dat");

    journal.status();

    // Simulate trading
    println!("\n=== New Trades ===");

    match journal.add_trade("BTC", TradeSide::Buy, 42000.0, 0.5) {
        Ok(id) => println!("Trade #{}: Bought 0.5 BTC @ $42000", id),
        Err(e) => println!("Error: {}", e),
    }

    match journal.add_trade("ETH", TradeSide::Buy, 2800.0, 5.0) {
        Ok(id) => println!("Trade #{}: Bought 5 ETH @ $2800", id),
        Err(e) => println!("Error: {}", e),
    }

    // Try to sell what we don't have
    match journal.add_trade("SOL", TradeSide::Sell, 100.0, 10.0) {
        Ok(id) => println!("Trade #{}: Sold 10 SOL", id),
        Err(e) => println!("Error: {}", e),
    }

    journal.status();

    println!("\n=== Data will be saved when the program runs again! ===");

    // Clean up test file
    let _ = std::fs::remove_file("trading_journal.dat");
}
