# Day 178: Pattern: Fan-out Fan-in

## Trading Analogy

Imagine you're managing a trading system that needs to analyze data from 10 exchanges simultaneously. You could do this sequentially — first check Binance, then Kraken, then Coinbase... But that would take a lot of time!

Instead, you use the **Fan-out Fan-in** pattern:
- **Fan-out (spreading)**: distribute tasks to multiple workers in parallel — each worker queries its own exchange
- **Fan-in (converging)**: collect results from all workers into a single stream for analysis

It's like sending 10 analysts to check prices on different exchanges, and then they all report back to you, and you pick the best price.

```
                    +-> Worker 1 (Binance)  --+
                    |                          |
Input data ---------+-> Worker 2 (Kraken)   --+---> Result aggregation
     (Fan-out)      |                          |         (Fan-in)
                    +-> Worker 3 (Coinbase) --+
```

## What is Fan-out Fan-in?

**Fan-out Fan-in** is a parallel processing pattern where:

1. **Fan-out**: a single task is distributed among multiple parallel processors
2. **Fan-in**: results from all processors are collected in one place

This pattern is ideal for:
- Aggregating data from multiple sources
- Parallel processing of a large number of elements
- Distributed computations

## Simple Example: Price Analysis from Exchanges

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceData {
    exchange: String,
    symbol: String,
    price: f64,
    volume: f64,
}

fn main() {
    // Channel for collecting results (Fan-in)
    let (tx, rx) = mpsc::channel();

    let exchanges = vec![
        ("Binance", 42150.0, 1000.0),
        ("Kraken", 42145.0, 800.0),
        ("Coinbase", 42160.0, 1200.0),
        ("Bitstamp", 42140.0, 500.0),
        ("Gemini", 42155.0, 600.0),
    ];

    // Fan-out: spawn a worker for each exchange
    for (exchange, price, volume) in exchanges {
        let tx = tx.clone();
        thread::spawn(move || {
            // Simulate network delay
            thread::sleep(Duration::from_millis(100 + (price as u64 % 50)));

            let data = PriceData {
                exchange: exchange.to_string(),
                symbol: "BTC/USD".to_string(),
                price,
                volume,
            };

            println!("[{}] Price received: ${:.2}", exchange, price);
            tx.send(data).unwrap();
        });
    }

    // Important: drop the original sender
    drop(tx);

    // Fan-in: collect all results
    let mut prices: Vec<PriceData> = Vec::new();
    for data in rx {
        prices.push(data);
    }

    // Analyze collected data
    println!("\n=== Market Analysis ===");

    let best_bid = prices.iter()
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        .unwrap();

    let best_ask = prices.iter()
        .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        .unwrap();

    let avg_price: f64 = prices.iter().map(|p| p.price).sum::<f64>() / prices.len() as f64;
    let total_volume: f64 = prices.iter().map(|p| p.volume).sum();

    println!("Best buy price: {} @ ${:.2}", best_bid.exchange, best_bid.price);
    println!("Best sell price: {} @ ${:.2}", best_ask.exchange, best_ask.price);
    println!("Average price: ${:.2}", avg_price);
    println!("Total volume: {:.0} BTC", total_volume);
    println!("Spread: ${:.2}", best_bid.price - best_ask.price);
}
```

## Fan-out with Worker Pool

In real systems, we limit the number of parallel workers:

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct TradeSignal {
    symbol: String,
    signal_type: String,
    strength: f64,
}

#[derive(Debug)]
struct AnalysisResult {
    symbol: String,
    recommendation: String,
    confidence: f64,
}

fn main() {
    // Task queue (symbols to analyze)
    let symbols = vec![
        "BTC/USD", "ETH/USD", "SOL/USD", "ADA/USD",
        "DOT/USD", "LINK/USD", "AVAX/USD", "MATIC/USD",
    ];

    let tasks: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(
        symbols.iter().map(|s| s.to_string()).collect()
    ));

    let (tx, rx) = mpsc::channel();

    // Create a pool of 3 workers (Fan-out with limit)
    let num_workers = 3;
    let mut handles = vec![];

    for worker_id in 0..num_workers {
        let tasks = Arc::clone(&tasks);
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            loop {
                // Get a task from the queue
                let symbol = {
                    let mut queue = tasks.lock().unwrap();
                    queue.pop_front()
                };

                match symbol {
                    Some(symbol) => {
                        println!("Worker {} analyzing {}", worker_id, symbol);

                        // Simulate analysis
                        thread::sleep(std::time::Duration::from_millis(150));

                        // Generate analysis result
                        let strength = (worker_id as f64 * 0.1) + 0.7;
                        let result = AnalysisResult {
                            symbol: symbol.clone(),
                            recommendation: if strength > 0.75 {
                                "BUY".to_string()
                            } else {
                                "HOLD".to_string()
                            },
                            confidence: strength,
                        };

                        tx.send(result).unwrap();
                    }
                    None => break, // Queue is empty
                }
            }
            println!("Worker {} finished", worker_id);
        });

        handles.push(handle);
    }

    drop(tx); // Close the channel after spawning all workers

    // Fan-in: collect results
    let mut results: Vec<AnalysisResult> = Vec::new();
    for result in rx {
        println!("  -> {} received: {} ({:.0}%)",
            result.symbol,
            result.recommendation,
            result.confidence * 100.0
        );
        results.push(result);
    }

    // Wait for all workers to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Final analysis
    println!("\n=== Final Recommendations ===");
    let buy_signals: Vec<_> = results.iter()
        .filter(|r| r.recommendation == "BUY")
        .collect();

    println!("Buy signals: {} out of {}", buy_signals.len(), results.len());
    for signal in buy_signals {
        println!("  {} (confidence: {:.0}%)", signal.symbol, signal.confidence * 100.0);
    }
}
```

## Parallel Portfolio Analysis

Using Fan-out Fan-in to calculate portfolio metrics:

```rust
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

#[derive(Debug)]
struct PositionMetrics {
    symbol: String,
    pnl: f64,
    pnl_percent: f64,
    value: f64,
    risk_score: f64,
}

fn calculate_position_metrics(position: Position) -> PositionMetrics {
    // Simulate complex calculation
    thread::sleep(std::time::Duration::from_millis(50));

    let pnl = (position.current_price - position.entry_price) * position.quantity;
    let pnl_percent = ((position.current_price / position.entry_price) - 1.0) * 100.0;
    let value = position.current_price * position.quantity;

    // Simple risk calculation based on volatility
    let risk_score = (pnl_percent.abs() / 10.0).min(1.0);

    PositionMetrics {
        symbol: position.symbol,
        pnl,
        pnl_percent,
        value,
        risk_score,
    }
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, entry_price: 40000.0, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 30.0, entry_price: 2500.0, current_price: 2650.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, entry_price: 100.0, current_price: 95.0 },
        Position { symbol: "ADA".to_string(), quantity: 5000.0, entry_price: 0.50, current_price: 0.55 },
        Position { symbol: "DOT".to_string(), quantity: 200.0, entry_price: 7.0, current_price: 7.5 },
        Position { symbol: "LINK".to_string(), quantity: 150.0, entry_price: 15.0, current_price: 14.0 },
    ];

    let (tx, rx) = mpsc::channel();

    println!("Starting parallel analysis of {} positions...\n", portfolio.len());
    let start = std::time::Instant::now();

    // Fan-out: each position is analyzed in a separate thread
    for position in portfolio {
        let tx = tx.clone();
        thread::spawn(move || {
            let metrics = calculate_position_metrics(position);
            tx.send(metrics).unwrap();
        });
    }

    drop(tx);

    // Fan-in: collect all metrics
    let mut all_metrics: Vec<PositionMetrics> = Vec::new();
    for metrics in rx {
        all_metrics.push(metrics);
    }

    let elapsed = start.elapsed();

    // Aggregated portfolio metrics
    println!("=== Portfolio Analysis (in {:?}) ===\n", elapsed);

    let total_value: f64 = all_metrics.iter().map(|m| m.value).sum();
    let total_pnl: f64 = all_metrics.iter().map(|m| m.pnl).sum();
    let avg_risk: f64 = all_metrics.iter().map(|m| m.risk_score).sum::<f64>()
        / all_metrics.len() as f64;

    println!("+--------------------------------------------------------+");
    println!("|                 PORTFOLIO METRICS                      |");
    println!("+--------------------------------------------------------+");

    for m in &all_metrics {
        let pnl_indicator = if m.pnl >= 0.0 { "+" } else { "" };
        println!("| {:6} | PnL: {}${:>10.2} ({:>+6.2}%) | Risk: {:.2} |",
            m.symbol,
            pnl_indicator,
            m.pnl.abs(),
            m.pnl_percent,
            m.risk_score
        );
    }

    println!("+--------------------------------------------------------+");
    println!("| Total value:        ${:>12.2}                  |", total_value);
    println!("| Total PnL:          ${:>12.2}                  |", total_pnl);
    println!("| Average risk:       {:>13.2}                  |", avg_risk);
    println!("+--------------------------------------------------------+");

    // Identify high-risk positions
    let risky_positions: Vec<_> = all_metrics.iter()
        .filter(|m| m.risk_score > 0.5)
        .collect();

    if !risky_positions.is_empty() {
        println!("\n  High-risk positions:");
        for pos in risky_positions {
            println!("   - {} (risk: {:.2})", pos.symbol, pos.risk_score);
        }
    }
}
```

## Market Scanner with Fan-out Fan-in

A realistic scanner example that looks for trading opportunities:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume_24h: f64,
    change_24h: f64,
    rsi: f64,
}

#[derive(Debug)]
struct TradingOpportunity {
    symbol: String,
    signal: String,
    entry_price: f64,
    target_price: f64,
    stop_loss: f64,
    score: f64,
}

fn scan_symbol(data: MarketData) -> Option<TradingOpportunity> {
    // Simulate analysis
    thread::sleep(Duration::from_millis(100));

    // Conditions for trading opportunity
    let is_oversold = data.rsi < 30.0;
    let is_overbought = data.rsi > 70.0;
    let high_volume = data.volume_24h > 1_000_000.0;
    let significant_drop = data.change_24h < -5.0;
    let significant_rise = data.change_24h > 5.0;

    if is_oversold && high_volume && significant_drop {
        // Buy signal
        Some(TradingOpportunity {
            symbol: data.symbol,
            signal: "BUY".to_string(),
            entry_price: data.price,
            target_price: data.price * 1.10, // +10%
            stop_loss: data.price * 0.95,    // -5%
            score: (30.0 - data.rsi) / 30.0 + (data.change_24h.abs() / 10.0),
        })
    } else if is_overbought && high_volume && significant_rise {
        // Sell signal
        Some(TradingOpportunity {
            symbol: data.symbol,
            signal: "SELL".to_string(),
            entry_price: data.price,
            target_price: data.price * 0.90, // -10%
            stop_loss: data.price * 1.05,    // +5%
            score: (data.rsi - 70.0) / 30.0 + (data.change_24h / 10.0),
        })
    } else {
        None
    }
}

fn main() {
    // Simulated market data
    let market_data = vec![
        MarketData { symbol: "BTC".to_string(), price: 42000.0, volume_24h: 5_000_000.0, change_24h: -7.5, rsi: 25.0 },
        MarketData { symbol: "ETH".to_string(), price: 2600.0, volume_24h: 2_000_000.0, change_24h: 2.0, rsi: 55.0 },
        MarketData { symbol: "SOL".to_string(), price: 95.0, volume_24h: 1_500_000.0, change_24h: 8.0, rsi: 78.0 },
        MarketData { symbol: "ADA".to_string(), price: 0.55, volume_24h: 800_000.0, change_24h: -3.0, rsi: 42.0 },
        MarketData { symbol: "DOT".to_string(), price: 7.5, volume_24h: 1_200_000.0, change_24h: -6.0, rsi: 28.0 },
        MarketData { symbol: "AVAX".to_string(), price: 35.0, volume_24h: 900_000.0, change_24h: 1.5, rsi: 50.0 },
        MarketData { symbol: "LINK".to_string(), price: 14.0, volume_24h: 1_100_000.0, change_24h: 6.5, rsi: 72.0 },
        MarketData { symbol: "MATIC".to_string(), price: 0.85, volume_24h: 600_000.0, change_24h: -2.0, rsi: 38.0 },
    ];

    let (tx, rx) = mpsc::channel();

    println!("Scanning market ({} symbols)...\n", market_data.len());
    let start = std::time::Instant::now();

    // Fan-out: parallel scanning
    for data in market_data {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = scan_symbol(data);
            tx.send(result).unwrap();
        });
    }

    drop(tx);

    // Fan-in: collect results
    let mut opportunities: Vec<TradingOpportunity> = Vec::new();
    for result in rx {
        if let Some(opp) = result {
            opportunities.push(opp);
        }
    }

    let elapsed = start.elapsed();

    // Sort by signal strength
    opportunities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    println!("=== Scan Results (in {:?}) ===\n", elapsed);

    if opportunities.is_empty() {
        println!("No trading opportunities found.");
    } else {
        println!("Found {} trading opportunities:\n", opportunities.len());

        for (i, opp) in opportunities.iter().enumerate() {
            let emoji = if opp.signal == "BUY" { "[BUY]" } else { "[SELL]" };
            println!("{}. {} {} {}", i + 1, emoji, opp.signal, opp.symbol);
            println!("   Entry: ${:.4}", opp.entry_price);
            println!("   Target: ${:.4}", opp.target_price);
            println!("   Stop: ${:.4}", opp.stop_loss);
            println!("   Signal strength: {:.2}\n", opp.score);
        }
    }
}
```

## Fan-out Fan-in with Error Handling

In real systems, error handling is important:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum FetchError {
    Timeout,
    NetworkError(String),
    ParseError,
}

#[derive(Debug)]
struct ExchangePrice {
    exchange: String,
    price: f64,
}

type FetchResult = Result<ExchangePrice, (String, FetchError)>;

fn fetch_price(exchange: &str, should_fail: bool) -> FetchResult {
    // Simulate network request
    thread::sleep(Duration::from_millis(100));

    if should_fail {
        return Err((exchange.to_string(), FetchError::NetworkError("Connection refused".to_string())));
    }

    // Simulate successful response
    let base_price = 42000.0;
    let variation = (exchange.len() as f64 * 10.0) - 30.0;

    Ok(ExchangePrice {
        exchange: exchange.to_string(),
        price: base_price + variation,
    })
}

fn main() {
    let exchanges = vec![
        ("Binance", false),
        ("Kraken", true),      // Simulate failure
        ("Coinbase", false),
        ("Bitstamp", false),
        ("Gemini", true),      // Simulate failure
    ];

    let (tx, rx) = mpsc::channel();

    println!("Fetching prices from {} exchanges...\n", exchanges.len());

    // Fan-out with error handling
    for (exchange, should_fail) in exchanges {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = fetch_price(exchange, should_fail);
            tx.send(result).unwrap();
        });
    }

    drop(tx);

    // Fan-in with separating successful and failed results
    let mut successful: Vec<ExchangePrice> = Vec::new();
    let mut failed: Vec<(String, FetchError)> = Vec::new();

    for result in rx {
        match result {
            Ok(price) => {
                println!("[OK] {}: ${:.2}", price.exchange, price.price);
                successful.push(price);
            }
            Err((exchange, error)) => {
                println!("[ERROR] {}: {:?}", exchange, error);
                failed.push((exchange, error));
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Successful: {}", successful.len());
    println!("Errors: {}", failed.len());

    if !successful.is_empty() {
        let avg_price: f64 = successful.iter().map(|p| p.price).sum::<f64>()
            / successful.len() as f64;
        println!("Average price (from successful): ${:.2}", avg_price);
    }

    // For critical systems, you might require a minimum number of successful responses
    let min_required = 3;
    if successful.len() < min_required {
        println!("\n  Warning: received fewer than {} responses, data may be inaccurate", min_required);
    }
}
```

## Pattern Visualization

```
+-------------------------------------------------------------+
|                     FAN-OUT FAN-IN                          |
+-------------------------------------------------------------+
|                                                             |
|   Input         Fan-out           Fan-in        Result      |
|   data                                                      |
|                                                             |
|                 +---------+                                 |
|              +--|Worker 1 |--+                              |
|              |  +---------+  |                              |
|   +------+   |  +---------+  |   +----------+   +--------+  |
|   | Tasks|---+--|Worker 2 |--+---|Aggregator|---| Result |  |
|   +------+   |  +---------+  |   +----------+   +--------+  |
|              |  +---------+  |                              |
|              +--|Worker N |--+                              |
|                 +---------+                                 |
|                                                             |
|   * Load          * Parallel       * Combining              |
|     distribution    processing       results                |
|                                                             |
+-------------------------------------------------------------+
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Fan-out | Distributing a task among multiple parallel processors |
| Fan-in | Collecting results from all processors in one place |
| Worker pool | Limited number of workers for resource control |
| `mpsc::channel` | The main tool for Fan-in in Rust |
| Error handling | Important to track successful and failed results |
| Aggregation | Computing summary metrics from collected data |

## Advantages of the Pattern

1. **Parallelism**: process many tasks simultaneously
2. **Scalability**: easy to add more workers
3. **Fault tolerance**: failure of one worker doesn't block the entire process
4. **Efficiency**: utilize all available CPU resources

## Homework

1. **Multi-exchange Arbitrage**: Implement a system that fetches prices from 5 exchanges in parallel and finds arbitrage opportunities (price difference > 0.5%).

2. **Correlation Analysis**: Create a system that calculates correlation between 10 trading pairs in parallel and outputs a correlation matrix.

3. **Portfolio Stress Tester**: Implement a Fan-out Fan-in system that runs 100 Monte Carlo scenarios in parallel to assess portfolio risk.

4. **Pattern Scanner**: Create a scanner that searches for technical patterns (head-and-shoulders, double bottom, etc.) across 20 trading pairs in parallel and returns found patterns with confidence scores.

## Navigation

[Previous day](../177-pipeline-parallel-processing/en.md) | [Next day](../179-select-waiting-multiple-channels/en.md)
