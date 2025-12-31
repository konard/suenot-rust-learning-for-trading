#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Disable some pedantic checks for readability
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

use std::collections::HashMap;

/// Trading position
#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    pub fn new(symbol: &str, quantity: f64, entry_price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            entry_price,
            current_price: entry_price,
        }
    }

    /// Calculates unrealized profit/loss
    pub fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    /// Updates current price
    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
    }

    /// Returns market value of position
    pub fn market_value(&self) -> f64 {
        self.current_price * self.quantity.abs()
    }
}

/// Trader's portfolio
pub struct Portfolio {
    positions: HashMap<String, Position>,
    cash: f64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_cash,
        }
    }

    /// Adds a new position or increases existing one
    pub fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        // Clippy approves: using entry API
        self.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                // Weighted average entry price
                let total_quantity = pos.quantity + quantity;
                if total_quantity.abs() > f64::EPSILON {
                    pos.entry_price = (pos.entry_price * pos.quantity + price * quantity)
                        / total_quantity;
                }
                pos.quantity = total_quantity;
            })
            .or_insert_with(|| Position::new(symbol, quantity, price));

        self.cash -= quantity * price;
    }

    /// Closes position completely
    pub fn close_position(&mut self, symbol: &str) -> Option<f64> {
        // Clippy approves: using remove instead of get + remove
        self.positions.remove(symbol).map(|pos| {
            let pnl = pos.unrealized_pnl();
            self.cash += pos.market_value() + pnl;
            pnl
        })
    }

    /// Updates prices for all positions
    pub fn update_prices(&mut self, prices: &HashMap<String, f64>) {
        // Clippy approves: values_mut for in-place modification
        for position in self.positions.values_mut() {
            if let Some(&price) = prices.get(&position.symbol) {
                position.update_price(price);
            }
        }
    }

    /// Calculates total unrealized `PnL`
    pub fn total_unrealized_pnl(&self) -> f64 {
        // Clippy approves: using sum()
        self.positions.values().map(Position::unrealized_pnl).sum()
    }

    /// Returns total portfolio value
    pub fn total_value(&self) -> f64 {
        self.cash + self.positions.values().map(Position::market_value).sum::<f64>()
    }

    /// Returns positions sorted by `PnL`
    pub fn positions_by_pnl(&self) -> Vec<&Position> {
        // Clippy may suggest sorted_by instead of sort_by on clone
        let mut positions: Vec<_> = self.positions.values().collect();
        positions.sort_by(|a, b| {
            b.unrealized_pnl()
                .partial_cmp(&a.unrealized_pnl())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        positions
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100_000.0);

    // Open positions
    portfolio.add_position("BTCUSDT", 0.5, 50_000.0);
    portfolio.add_position("ETHUSDT", 5.0, 3_000.0);
    portfolio.add_position("SOLUSDT", 100.0, 100.0);

    println!("=== Initial Portfolio ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Cash: ${:.2}", portfolio.cash);

    // Update prices
    let mut new_prices = HashMap::new();
    new_prices.insert("BTCUSDT".to_string(), 52_000.0);
    new_prices.insert("ETHUSDT".to_string(), 3_200.0);
    new_prices.insert("SOLUSDT".to_string(), 95.0);

    portfolio.update_prices(&new_prices);

    println!("\n=== After Price Update ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Unrealized PnL: ${:.2}", portfolio.total_unrealized_pnl());

    println!("\n=== Positions by PnL ===");
    for pos in portfolio.positions_by_pnl() {
        println!(
            "{}: quantity={:.2}, PnL=${:.2}",
            pos.symbol,
            pos.quantity,
            pos.unrealized_pnl()
        );
    }

    // Close profitable position
    if let Some(pnl) = portfolio.close_position("ETHUSDT") {
        println!("\nClosed ETHUSDT position with PnL: ${pnl:.2}");
    }

    println!("\n=== Final Portfolio ===");
    println!("Total value: ${:.2}", portfolio.total_value());
    println!("Cash: ${:.2}", portfolio.cash);
}
