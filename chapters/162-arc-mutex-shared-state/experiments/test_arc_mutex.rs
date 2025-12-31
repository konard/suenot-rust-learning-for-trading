// Test file for verifying Arc<Mutex<T>> code examples from Chapter 162

use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::time::Duration;

fn test_basic_example() {
    println!("=== Test: Basic Shared Balance ===");
    let balance = Arc::new(Mutex::new(10000.0_f64));
    let mut handles = vec![];

    for strategy_id in 1..=3 {
        let balance_clone = Arc::clone(&balance);

        let handle = thread::spawn(move || {
            let mut bal = balance_clone.lock().unwrap();
            let profit = strategy_id as f64 * 100.0;
            *bal += profit;
            println!("Strategy {}: added ${:.2}, balance: ${:.2}",
                     strategy_id, profit, *bal);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final balance: ${:.2}", *balance.lock().unwrap());
}

#[derive(Debug, Clone)]
struct Portfolio {
    balance: f64,
    positions: HashMap<String, f64>,
    total_pnl: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: HashMap::new(),
            total_pnl: 0.0,
        }
    }

    fn buy(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;
        if cost > self.balance {
            return Err(format!("Insufficient funds: need ${:.2}, have ${:.2}",
                              cost, self.balance));
        }

        self.balance -= cost;
        *self.positions.entry(ticker.to_string()).or_insert(0.0) += quantity;
        Ok(())
    }

    fn sell(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<f64, String> {
        let position = self.positions.get(ticker).copied().unwrap_or(0.0);
        if quantity > position {
            return Err(format!("Insufficient {}: need {}, have {}",
                              ticker, quantity, position));
        }

        let revenue = quantity * price;
        self.balance += revenue;
        *self.positions.get_mut(ticker).unwrap() -= quantity;

        if self.positions[ticker] == 0.0 {
            self.positions.remove(ticker);
        }

        Ok(revenue)
    }
}

fn test_portfolio() {
    println!("\n=== Test: Portfolio Structure ===");
    let portfolio = Arc::new(Mutex::new(Portfolio::new(100000.0)));
    let mut handles = vec![];

    let p1 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p1.lock().unwrap();
        match port.buy("BTC", 0.5, 42000.0) {
            Ok(_) => println!("Strategy 1: bought 0.5 BTC"),
            Err(e) => println!("Strategy 1: error - {}", e),
        }
    }));

    let p2 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p2.lock().unwrap();
        match port.buy("ETH", 5.0, 2200.0) {
            Ok(_) => println!("Strategy 2: bought 5 ETH"),
            Err(e) => println!("Strategy 2: error - {}", e),
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let final_portfolio = portfolio.lock().unwrap();
    println!("\nFinal portfolio:");
    println!("Balance: ${:.2}", final_portfolio.balance);
    println!("Positions: {:?}", final_portfolio.positions);
}

#[derive(Debug, Default)]
struct TradeStats {
    total_trades: u64,
    winning_trades: u64,
    losing_trades: u64,
    total_pnl: f64,
    max_profit: f64,
    max_loss: f64,
}

impl TradeStats {
    fn record_trade(&mut self, pnl: f64) {
        self.total_trades += 1;
        self.total_pnl += pnl;

        if pnl > 0.0 {
            self.winning_trades += 1;
            if pnl > self.max_profit {
                self.max_profit = pnl;
            }
        } else if pnl < 0.0 {
            self.losing_trades += 1;
            if pnl < self.max_loss {
                self.max_loss = pnl;
            }
        }
    }

    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.total_trades as f64) * 100.0
    }
}

fn test_trade_stats() {
    println!("\n=== Test: Trade Statistics ===");
    let stats = Arc::new(Mutex::new(TradeStats::default()));
    let mut handles = vec![];

    for thread_id in 0..3 {
        let stats_clone = Arc::clone(&stats);

        handles.push(thread::spawn(move || {
            let trades = vec![150.0, -50.0, 200.0, -30.0, 100.0];

            for pnl in trades {
                let adjusted_pnl = pnl * (thread_id as f64 + 1.0);
                let mut s = stats_clone.lock().unwrap();
                s.record_trade(adjusted_pnl);
                println!("Thread {}: trade ${:.2}", thread_id, adjusted_pnl);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_stats = stats.lock().unwrap();
    println!("\n=== Final Statistics ===");
    println!("Total trades: {}", final_stats.total_trades);
    println!("Winning: {}", final_stats.winning_trades);
    println!("Losing: {}", final_stats.losing_trades);
    println!("Win Rate: {:.1}%", final_stats.win_rate());
    println!("Total PnL: ${:.2}", final_stats.total_pnl);
    println!("Max profit: ${:.2}", final_stats.max_profit);
    println!("Max loss: ${:.2}", final_stats.max_loss);
}

fn test_lock_minimization() {
    println!("\n=== Test: Lock Minimization Pattern ===");

    struct OrderBook {
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    }

    let order_book = Arc::new(Mutex::new(OrderBook {
        bids: vec![(42000.0, 1.5), (41990.0, 2.0)],
        asks: vec![(42010.0, 1.0), (42020.0, 3.0)],
    }));

    let ob = Arc::clone(&order_book);
    let analyzer = thread::spawn(move || {
        let (best_bid, best_ask) = {
            let book = ob.lock().unwrap();
            (book.bids[0].0, book.asks[0].0)
        };

        thread::sleep(Duration::from_millis(10));
        let spread = best_ask - best_bid;
        println!("Spread: ${:.2}", spread);
    });

    analyzer.join().unwrap();
}

fn main() {
    test_basic_example();
    test_portfolio();
    test_trade_stats();
    test_lock_minimization();

    println!("\n=== All tests passed! ===");
}
