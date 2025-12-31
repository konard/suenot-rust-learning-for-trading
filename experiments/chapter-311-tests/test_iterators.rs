// Test basic iterator examples
use std::time::Instant;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: i32,
    profit: f64,
}

#[derive(Debug)]
struct Order {
    id: u32,
    symbol: String,
    price: f64,
    quantity: i32,
}

fn average_price_loop(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let mut sum = 0.0;
    for price in prices {
        sum += price;
    }

    Some(sum / prices.len() as f64)
}

fn average_price_iter(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    Some(prices.iter().sum::<f64>() / prices.len() as f64)
}

fn get_large_btc_orders_loop(orders: &[Order], min_quantity: i32) -> Vec<u32> {
    let mut result = Vec::new();

    for order in orders {
        if order.symbol == "BTC" && order.quantity >= min_quantity {
            result.push(order.id);
        }
    }

    result
}

fn get_large_btc_orders_iter(orders: &[Order], min_quantity: i32) -> Vec<u32> {
    orders.iter()
        .filter(|o| o.symbol == "BTC")
        .filter(|o| o.quantity >= min_quantity)
        .map(|o| o.id)
        .collect()
}

#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn new(timestamp: u64, close: f64) -> Self {
        Self {
            timestamp,
            open: close,
            high: close * 1.01,
            low: close * 0.99,
            close,
            volume: 1000.0,
        }
    }
}

fn calculate_returns_loop(candles: &[Candle]) -> Vec<f64> {
    let mut returns = Vec::with_capacity(candles.len() - 1);

    for i in 1..candles.len() {
        let return_pct = (candles[i].close - candles[i - 1].close) / candles[i - 1].close;
        returns.push(return_pct);
    }

    returns
}

fn calculate_returns_iter(candles: &[Candle]) -> Vec<f64> {
    candles.windows(2)
        .map(|pair| (pair[1].close - pair[0].close) / pair[0].close)
        .collect()
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn profit(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn profit_pct(&self) -> f64 {
        (self.current_price - self.entry_price) / self.entry_price
    }
}

fn analyze_portfolio(positions: &[Position]) {
    // 1. Total profit/loss (iterator)
    let total_pnl: f64 = positions.iter()
        .map(|p| p.profit())
        .sum();

    // 2. Number of profitable positions
    let profitable_count = positions.iter()
        .filter(|p| p.profit() > 0.0)
        .count();

    // 3. Average profit percentage (profitable only)
    let avg_profit_pct = positions.iter()
        .filter(|p| p.profit() > 0.0)
        .map(|p| p.profit_pct())
        .sum::<f64>() / profitable_count.max(1) as f64;

    // 4. Worst position
    let worst_position = positions.iter()
        .min_by(|a, b| a.profit().partial_cmp(&b.profit()).unwrap());

    // 5. Symbols with loss > 10%
    let heavy_losses: Vec<&str> = positions.iter()
        .filter(|p| p.profit_pct() < -0.10)
        .map(|p| p.symbol.as_str())
        .collect();

    println!("=== Portfolio Analysis ===");
    println!("Total P&L: ${:.2}", total_pnl);
    println!("Profitable positions: {} out of {}", profitable_count, positions.len());
    println!("Average profit: {:.2}%", avg_profit_pct * 100.0);

    if let Some(worst) = worst_position {
        println!("Worst position: {} (${:.2})", worst.symbol, worst.profit());
    }

    if !heavy_losses.is_empty() {
        println!("Losses > 10%: {:?}", heavy_losses);
    }
}

fn main() {
    println!("=== Testing Iterator Examples ===\n");

    // Test 1: Average price
    let btc_prices = vec![42000.0, 43500.0, 41800.0, 44200.0, 43000.0];

    match average_price_loop(&btc_prices) {
        Some(avg) => println!("Average price (loop): ${:.2}", avg),
        None => println!("No data"),
    }

    match average_price_iter(&btc_prices) {
        Some(avg) => println!("Average price (iterator): ${:.2}", avg),
        None => println!("No data"),
    }
    println!();

    // Test 2: Large BTC orders
    let orders = vec![
        Order { id: 1, symbol: "BTC".to_string(), price: 42000.0, quantity: 5 },
        Order { id: 2, symbol: "ETH".to_string(), price: 2500.0, quantity: 50 },
        Order { id: 3, symbol: "BTC".to_string(), price: 43000.0, quantity: 2 },
        Order { id: 4, symbol: "BTC".to_string(), price: 41500.0, quantity: 10 },
    ];

    let large_btc_loop = get_large_btc_orders_loop(&orders, 5);
    println!("Large BTC orders (loop): {:?}", large_btc_loop);

    let large_btc_iter = get_large_btc_orders_iter(&orders, 5);
    println!("Large BTC orders (iterator): {:?}", large_btc_iter);
    println!();

    // Test 3: Benchmark
    let candles: Vec<Candle> = (0..10_000)
        .map(|i| Candle::new(i, 42000.0 + (i as f64 * 0.5)))
        .collect();

    let start = Instant::now();
    let returns_loop = calculate_returns_loop(&candles);
    let duration_loop = start.elapsed();

    let start = Instant::now();
    let returns_iter = calculate_returns_iter(&candles);
    let duration_iter = start.elapsed();

    println!("=== Benchmark: returns calculation ===");
    println!("Number of candles: {}", candles.len());
    println!("for loop:  {:?} ({} results)", duration_loop, returns_loop.len());
    println!("Iterator:  {:?} ({} results)", duration_iter, returns_iter.len());

    if duration_iter < duration_loop {
        let speedup = duration_loop.as_nanos() as f64 / duration_iter.as_nanos() as f64;
        println!("✅ Iterator is {:.2}x faster", speedup);
    } else {
        println!("⚖️  Performance is roughly the same");
    }
    println!();

    // Test 4: Portfolio analysis
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.0, entry_price: 40000.0, current_price: 43000.0 },
        Position { symbol: "ETH".to_string(), quantity: 50.0, entry_price: 2800.0, current_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, entry_price: 120.0, current_price: 105.0 },
        Position { symbol: "AAPL".to_string(), quantity: 10.0, entry_price: 180.0, current_price: 185.0 },
    ];

    analyze_portfolio(&portfolio);
}
