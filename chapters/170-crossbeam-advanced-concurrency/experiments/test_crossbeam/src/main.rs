use crossbeam::thread;

fn main() {
    println!("=== Test 1: Basic scope with borrowing ===");
    test_basic_scope();

    println!("\n=== Test 2: Portfolio analysis ===");
    test_portfolio_analysis();

    println!("\n=== Test 3: Nested spawning ===");
    test_nested_spawning();

    println!("\n=== Test 4: Technical indicators ===");
    test_technical_indicators();

    println!("\n=== All tests passed! ===");
}

fn test_basic_scope() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0];
    let volumes = vec![100, 250, 150, 300];

    thread::scope(|s| {
        s.spawn(|_| {
            let avg_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
            println!("Average price: ${:.2}", avg_price);
        });

        s.spawn(|_| {
            let total_volume: i32 = volumes.iter().sum();
            println!("Total volume: {} lots", total_volume);
        });

        s.spawn(|_| {
            let weighted_price: f64 = prices.iter()
                .zip(volumes.iter())
                .map(|(p, v)| p * *v as f64)
                .sum::<f64>() / volumes.iter().sum::<i32>() as f64;
            println!("Volume-weighted price: ${:.2}", weighted_price);
        });

    }).unwrap();

    println!("Total prices analyzed: {}", prices.len());
}

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.avg_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price / self.avg_price) - 1.0) * 100.0
    }
}

fn test_portfolio_analysis() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, avg_price: 40000.0, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 15.0, avg_price: 2800.0, current_price: 2650.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, avg_price: 95.0, current_price: 110.0 },
        Position { symbol: "DOGE".to_string(), quantity: 50000.0, avg_price: 0.08, current_price: 0.09 },
    ];

    let results = thread::scope(|s| {
        let pnl_handle = s.spawn(|_| {
            portfolio.iter().map(|p| p.pnl()).sum::<f64>()
        });

        let best_handle = s.spawn(|_| {
            portfolio.iter()
                .max_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        let worst_handle = s.spawn(|_| {
            portfolio.iter()
                .min_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        let value_handle = s.spawn(|_| {
            portfolio.iter().map(|p| p.current_price * p.quantity).sum::<f64>()
        });

        (
            pnl_handle.join().unwrap(),
            best_handle.join().unwrap(),
            worst_handle.join().unwrap(),
            value_handle.join().unwrap(),
        )
    }).unwrap();

    println!("Total PnL: ${:.2}", results.0);
    if let Some((symbol, pct)) = results.1 {
        println!("Best position: {} ({:+.2}%)", symbol, pct);
    }
    if let Some((symbol, pct)) = results.2 {
        println!("Worst position: {} ({:+.2}%)", symbol, pct);
    }
    println!("Portfolio value: ${:.2}", results.3);
}

fn test_nested_spawning() {
    let exchanges = vec!["Binance", "Kraken", "Coinbase"];
    let symbols = vec!["BTC", "ETH", "SOL"];

    // Demonstrate borrowing in nested scopes
    // Outer scope borrows exchanges, inner scope borrows symbols
    thread::scope(|s| {
        // First level: spawn thread for each exchange
        s.spawn(|_| {
            println!("Analyzer thread started");

            // Nested scope: can still borrow from outer scope
            thread::scope(|inner_s| {
                for exchange in &exchanges {
                    for symbol in &symbols {
                        inner_s.spawn(move |_| {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            println!("[{}] {} = $42000.00", exchange, symbol);
                        });
                    }
                }
            }).unwrap();

            println!("All exchange data collected");
        });

        // Another parallel task
        s.spawn(|_| {
            println!("Monitoring {} exchanges with {} symbols",
                     exchanges.len(), symbols.len());
        });
    }).unwrap();

    println!("All data collected!");
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    prices.windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = vec![prices[0]];

    for price in &prices[1..] {
        let new_ema = (price - ema.last().unwrap()) * multiplier + ema.last().unwrap();
        ema.push(new_ema);
    }

    ema
}

fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let changes: Vec<f64> = prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let mut rsi = Vec::new();

    for i in period..changes.len() {
        let window = &changes[i - period..i];
        let gains: f64 = window.iter().filter(|&&x| x > 0.0).sum();
        let losses: f64 = window.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum();

        let rs = if losses == 0.0 { 100.0 } else { gains / losses };
        let rsi_value = 100.0 - (100.0 / (1.0 + rs));
        rsi.push(rsi_value);
    }

    rsi
}

fn test_technical_indicators() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let indicators = thread::scope(|s| {
        let sma_handle = s.spawn(|_| {
            ("SMA(5)", calculate_sma(&prices, 5))
        });

        let ema_handle = s.spawn(|_| {
            ("EMA(5)", calculate_ema(&prices, 5))
        });

        let rsi_handle = s.spawn(|_| {
            ("RSI(14)", calculate_rsi(&prices, 14))
        });

        vec![
            sma_handle.join().unwrap(),
            ema_handle.join().unwrap(),
            rsi_handle.join().unwrap(),
        ]
    }).unwrap();

    for (name, values) in indicators {
        if let Some(last) = values.last() {
            println!("{}: {:.2}", name, last);
        } else {
            println!("{}: insufficient data", name);
        }
    }
}
