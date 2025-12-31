// Test file to verify code examples from Chapter 349 compile correctly

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

impl Order {
    fn new(symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Order {
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            timestamp,
        }
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }

    fn is_buy(&self) -> bool {
        matches!(self.side, OrderSide::Buy)
    }
}

fn calculate_portfolio_value(orders: &[Order], prices: &HashMap<String, f64>) -> f64 {
    orders
        .iter()
        .filter(|o| o.is_buy())
        .map(|o| {
            let current_price = prices.get(&o.symbol).unwrap_or(&o.price);
            o.quantity * current_price
        })
        .sum()
}

/// Exchange fee table
/// Format: (maker_fee, taker_fee)
#[rustfmt::skip]
const EXCHANGE_FEES: &[(&str, (f64, f64))] = &[
    ("Binance",  (0.001, 0.001)),
    ("Coinbase", (0.004, 0.006)),
    ("Kraken",   (0.002, 0.005)),
    ("Bybit",    (0.001, 0.001)),
    ("OKX",      (0.002, 0.005)),
];

/// Asset correlation matrix
#[rustfmt::skip]
const CORRELATION_MATRIX: [[f64; 4]; 4] = [
    // BTC    ETH    SOL    ADA
    [ 1.00,  0.85,  0.72,  0.68],  // BTC
    [ 0.85,  1.00,  0.78,  0.75],  // ETH
    [ 0.72,  0.78,  1.00,  0.82],  // SOL
    [ 0.68,  0.75,  0.82,  1.00],  // ADA
];

/// Macro for creating an order
macro_rules! order {
    ($symbol:expr, $side:ident, $price:expr, $qty:expr) => {
        Order {
            symbol: $symbol.to_string(),
            side: OrderSide::$side,
            price: $price,
            quantity: $qty,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    };
}

fn main() {
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 50000.0);
    prices.insert("ETHUSDT".to_string(), 3000.0);

    let orders = vec![
        Order::new("BTCUSDT", OrderSide::Buy, 49000.0, 0.5),
        Order::new("ETHUSDT", OrderSide::Buy, 2900.0, 2.0),
        Order::new("BTCUSDT", OrderSide::Sell, 51000.0, 0.2),
    ];

    let value = calculate_portfolio_value(&orders, &prices);
    println!("Portfolio value: ${:.2}", value);

    // Test macro
    let macro_orders = vec![
        order!("BTCUSDT", Buy, 50000.0, 0.1),
        order!("ETHUSDT", Sell, 3000.0, 1.0),
    ];

    for order in &macro_orders {
        println!("{:?}", order);
    }

    // Display fees
    println!("\n=== Exchange Fees ===");
    for (exchange, (maker, taker)) in EXCHANGE_FEES {
        println!("{:12} Maker: {:.2}% Taker: {:.2}%", exchange, maker * 100.0, taker * 100.0);
    }

    // Display correlation matrix
    println!("\n=== Correlation Matrix ===");
    let assets = ["BTC", "ETH", "SOL", "ADA"];
    print!("     ");
    for asset in &assets {
        print!("{:>6}", asset);
    }
    println!();

    for (i, row) in CORRELATION_MATRIX.iter().enumerate() {
        print!("{:>4} ", assets[i]);
        for val in row {
            print!("{:>6.2}", val);
        }
        println!();
    }

    println!("\nAll examples compiled and ran successfully!");
}
