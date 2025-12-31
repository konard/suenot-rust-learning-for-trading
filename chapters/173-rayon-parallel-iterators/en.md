# Day 173: rayon: Parallel Iterators

## Trading Analogy

Imagine you're a trader who needs to analyze closing prices of 1000 stocks to identify which ones had the highest volatility today. Doing this sequentially would be like checking each stock one by one — it works, but it's slow. What if you could hire 8 analysts, give each of them ~125 stocks, and have them work in parallel? That's exactly what **rayon** does for your code!

In real trading scenarios, parallelization is crucial for:
- Analyzing thousands of price points across multiple assets
- Calculating risk metrics for an entire portfolio
- Backtesting strategies across multiple time periods simultaneously
- Processing real-time market data from multiple exchanges

## What is Rayon?

Rayon is a Rust library that enables **data parallelism** with minimal code changes. Instead of manually spawning threads, managing work distribution, and collecting results, you simply change `.iter()` to `.par_iter()` and rayon handles everything automatically.

### Key Features

1. **Drop-in replacement** — minimal code changes required
2. **Work stealing** — efficient load balancing across CPU cores
3. **Automatic thread management** — no manual thread spawning
4. **Safe by design** — leverages Rust's ownership system

## Getting Started with Rayon

Add to your `Cargo.toml`:

```toml
[dependencies]
rayon = "1.8"
```

### Basic Example: Calculating Portfolio Value

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    current_price: f64,
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 10.0, current_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, current_price: 95.0 },
        Position { symbol: "AAPL".to_string(), quantity: 50.0, current_price: 185.0 },
        Position { symbol: "GOOGL".to_string(), quantity: 20.0, current_price: 140.0 },
    ];

    // Sequential calculation
    let sequential_total: f64 = portfolio
        .iter()
        .map(|pos| pos.quantity * pos.current_price)
        .sum();

    // Parallel calculation — just change iter() to par_iter()!
    let parallel_total: f64 = portfolio
        .par_iter()
        .map(|pos| pos.quantity * pos.current_price)
        .sum();

    println!("Sequential total: ${:.2}", sequential_total);
    println!("Parallel total: ${:.2}", parallel_total);
}
```

## Parallel Iterator Methods

Rayon provides parallel versions of most standard iterator methods:

| Sequential | Parallel | Description |
|-----------|----------|-------------|
| `.iter()` | `.par_iter()` | Immutable parallel iteration |
| `.iter_mut()` | `.par_iter_mut()` | Mutable parallel iteration |
| `.into_iter()` | `.into_par_iter()` | Consuming parallel iteration |
| `.map()` | `.map()` | Transform elements |
| `.filter()` | `.filter()` | Filter elements |
| `.reduce()` | `.reduce()` | Combine elements |
| `.sum()` | `.sum()` | Sum numeric values |
| `.for_each()` | `.for_each()` | Execute side effects |

## Trading Example: Volatility Analysis

Let's analyze the volatility of multiple assets in parallel:

```rust
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceHistory {
    symbol: String,
    prices: Vec<f64>,
}

impl PriceHistory {
    fn calculate_volatility(&self) -> f64 {
        if self.prices.len() < 2 {
            return 0.0;
        }

        // Calculate daily returns
        let returns: Vec<f64> = self.prices
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        // Calculate mean return
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        // Calculate variance
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        // Return standard deviation (volatility)
        variance.sqrt()
    }
}

fn main() {
    // Simulated price histories for multiple assets
    let assets = vec![
        PriceHistory {
            symbol: "BTC".to_string(),
            prices: vec![40000.0, 41000.0, 39500.0, 42000.0, 43500.0, 41000.0],
        },
        PriceHistory {
            symbol: "ETH".to_string(),
            prices: vec![2400.0, 2500.0, 2350.0, 2600.0, 2550.0, 2700.0],
        },
        PriceHistory {
            symbol: "SOL".to_string(),
            prices: vec![90.0, 95.0, 88.0, 102.0, 98.0, 105.0],
        },
        PriceHistory {
            symbol: "AAPL".to_string(),
            prices: vec![180.0, 182.0, 179.0, 185.0, 183.0, 186.0],
        },
    ];

    // Calculate volatility for all assets in parallel
    let volatilities: HashMap<String, f64> = assets
        .par_iter()
        .map(|asset| {
            let vol = asset.calculate_volatility();
            println!("Calculated volatility for {}: {:.4}", asset.symbol, vol);
            (asset.symbol.clone(), vol)
        })
        .collect();

    // Find the most volatile asset
    let most_volatile = volatilities
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    if let Some((symbol, vol)) = most_volatile {
        println!("\nMost volatile asset: {} with volatility {:.4}", symbol, vol);
    }
}
```

## Parallel Reduce: Portfolio Risk Calculation

When combining parallel results, use `reduce()` with an identity element:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    pnl: f64,         // Profit/Loss
    risk_score: f64,  // Risk from 0 to 1
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), pnl: 1500.0, risk_score: 0.8 },
        Trade { symbol: "ETH".to_string(), pnl: -200.0, risk_score: 0.6 },
        Trade { symbol: "SOL".to_string(), pnl: 800.0, risk_score: 0.9 },
        Trade { symbol: "AAPL".to_string(), pnl: 300.0, risk_score: 0.3 },
        Trade { symbol: "GOOGL".to_string(), pnl: -100.0, risk_score: 0.4 },
    ];

    // Calculate total P&L and average risk in parallel
    let (total_pnl, total_risk, count) = trades
        .par_iter()
        .map(|trade| (trade.pnl, trade.risk_score, 1))
        .reduce(
            || (0.0, 0.0, 0),  // Identity element
            |acc, item| (acc.0 + item.0, acc.1 + item.1, acc.2 + item.2),
        );

    let avg_risk = if count > 0 { total_risk / count as f64 } else { 0.0 };

    println!("Total P&L: ${:.2}", total_pnl);
    println!("Average Risk Score: {:.2}", avg_risk);
    println!("Number of trades: {}", count);
}
```

## Filtering and Collecting: Finding Opportunities

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct MarketSignal {
    symbol: String,
    signal_strength: f64,  // -1.0 (strong sell) to 1.0 (strong buy)
    volume: u64,
    price: f64,
}

fn main() {
    let signals = vec![
        MarketSignal { symbol: "BTC".to_string(), signal_strength: 0.85, volume: 1_000_000, price: 42000.0 },
        MarketSignal { symbol: "ETH".to_string(), signal_strength: 0.25, volume: 500_000, price: 2500.0 },
        MarketSignal { symbol: "SOL".to_string(), signal_strength: 0.92, volume: 200_000, price: 95.0 },
        MarketSignal { symbol: "DOGE".to_string(), signal_strength: -0.3, volume: 10_000, price: 0.08 },
        MarketSignal { symbol: "AAPL".to_string(), signal_strength: 0.45, volume: 2_000_000, price: 185.0 },
        MarketSignal { symbol: "GME".to_string(), signal_strength: 0.15, volume: 50_000, price: 25.0 },
    ];

    // Find strong buy signals with high volume in parallel
    let opportunities: Vec<_> = signals
        .par_iter()
        .filter(|s| s.signal_strength > 0.5 && s.volume > 100_000)
        .map(|s| (s.symbol.clone(), s.signal_strength, s.price))
        .collect();

    println!("Strong buy opportunities:");
    for (symbol, strength, price) in &opportunities {
        println!("  {} - Signal: {:.2}, Price: ${:.2}", symbol, strength, price);
    }
}
```

## Parallel Sorting

Rayon provides efficient parallel sorting:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

fn main() {
    let mut orders = vec![
        Order { id: 1, price: 42100.0, quantity: 0.5, timestamp: 1000 },
        Order { id: 2, price: 42050.0, quantity: 1.0, timestamp: 1001 },
        Order { id: 3, price: 42200.0, quantity: 0.3, timestamp: 1002 },
        Order { id: 4, price: 41900.0, quantity: 2.0, timestamp: 1003 },
        Order { id: 5, price: 42000.0, quantity: 0.8, timestamp: 1004 },
    ];

    // Sort orders by price (ascending) in parallel
    orders.par_sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    println!("Orders sorted by price:");
    for order in &orders {
        println!("  ID: {}, Price: ${:.2}, Qty: {}", order.id, order.price, order.quantity);
    }

    // Sort by quantity (descending) in parallel
    orders.par_sort_by(|a, b| b.quantity.partial_cmp(&a.quantity).unwrap());

    println!("\nOrders sorted by quantity (desc):");
    for order in &orders {
        println!("  ID: {}, Price: ${:.2}, Qty: {}", order.id, order.price, order.quantity);
    }
}
```

## Parallel Processing: Backtesting Multiple Strategies

```rust
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    buy_threshold: f64,
    sell_threshold: f64,
}

#[derive(Debug, Clone)]
struct BacktestResult {
    strategy_name: String,
    total_return: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn backtest_strategy(strategy: &Strategy, prices: &[f64]) -> BacktestResult {
    let mut position = false;
    let mut entry_price = 0.0;
    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut total_trades = 0;
    let mut peak_equity = 0.0;
    let mut max_drawdown = 0.0;
    let mut equity = 10000.0;

    for i in 1..prices.len() {
        let change = (prices[i] - prices[i - 1]) / prices[i - 1];

        if !position && change > strategy.buy_threshold {
            position = true;
            entry_price = prices[i];
        } else if position && change < -strategy.sell_threshold {
            position = false;
            let pnl = (prices[i] - entry_price) / entry_price;
            total_pnl += pnl;
            equity *= 1.0 + pnl;
            total_trades += 1;
            if pnl > 0.0 {
                wins += 1;
            }
        }

        peak_equity = peak_equity.max(equity);
        let drawdown = (peak_equity - equity) / peak_equity;
        max_drawdown = max_drawdown.max(drawdown);
    }

    BacktestResult {
        strategy_name: strategy.name.clone(),
        total_return: total_pnl * 100.0,
        max_drawdown: max_drawdown * 100.0,
        win_rate: if total_trades > 0 {
            (wins as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        },
    }
}

fn main() {
    // Generate sample price data
    let prices: Vec<f64> = (0..10000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0 + (i as f64 * 0.001) * 5.0)
        .collect();

    // Define multiple strategies to test
    let strategies: Vec<Strategy> = (1..=50)
        .flat_map(|i| {
            (1..=10).map(move |j| Strategy {
                name: format!("Strategy_{}_{}", i, j),
                buy_threshold: 0.001 * i as f64,
                sell_threshold: 0.001 * j as f64,
            })
        })
        .collect();

    println!("Backtesting {} strategies...", strategies.len());

    // Sequential backtesting
    let start = Instant::now();
    let _sequential_results: Vec<BacktestResult> = strategies
        .iter()
        .map(|s| backtest_strategy(s, &prices))
        .collect();
    let sequential_time = start.elapsed();

    // Parallel backtesting
    let start = Instant::now();
    let parallel_results: Vec<BacktestResult> = strategies
        .par_iter()
        .map(|s| backtest_strategy(s, &prices))
        .collect();
    let parallel_time = start.elapsed();

    println!("Sequential time: {:?}", sequential_time);
    println!("Parallel time: {:?}", parallel_time);
    println!(
        "Speedup: {:.2}x",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    // Find best strategy
    let best = parallel_results
        .iter()
        .max_by(|a, b| a.total_return.partial_cmp(&b.total_return).unwrap());

    if let Some(result) = best {
        println!("\nBest Strategy: {}", result.strategy_name);
        println!("  Total Return: {:.2}%", result.total_return);
        println!("  Max Drawdown: {:.2}%", result.max_drawdown);
        println!("  Win Rate: {:.2}%", result.win_rate);
    }
}
```

## Configuring Thread Pool

Rayon uses a global thread pool by default, but you can customize it:

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Configure custom thread pool
    ThreadPoolBuilder::new()
        .num_threads(4)  // Use 4 threads
        .build_global()
        .expect("Failed to build thread pool");

    let numbers: Vec<i32> = (1..=100).collect();

    // This will use the configured 4-thread pool
    let sum: i32 = numbers.par_iter().sum();
    println!("Sum: {}", sum);
}
```

### Using Scoped Thread Pool

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();

    let prices = vec![100.0, 105.0, 103.0, 110.0, 108.0];

    // Execute within specific pool
    let result = pool.install(|| {
        prices
            .par_iter()
            .map(|p| p * 1.1)  // 10% markup
            .sum::<f64>()
    });

    println!("Total with markup: ${:.2}", result);
}
```

## When NOT to Use Rayon

Parallel processing has overhead. Avoid rayon when:

1. **Small datasets** — thread spawning overhead exceeds benefits
2. **Simple operations** — parallelism overhead may be greater than the work
3. **I/O-bound tasks** — use async instead
4. **Sequential dependencies** — operations that must happen in order

```rust
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let small_data: Vec<i32> = (1..100).collect();
    let large_data: Vec<i32> = (1..1_000_000).collect();

    // Small dataset — sequential might be faster
    let start = Instant::now();
    let _: i32 = small_data.iter().map(|x| x * 2).sum();
    let seq_small = start.elapsed();

    let start = Instant::now();
    let _: i32 = small_data.par_iter().map(|x| x * 2).sum();
    let par_small = start.elapsed();

    println!("Small dataset (100 items):");
    println!("  Sequential: {:?}", seq_small);
    println!("  Parallel: {:?}", par_small);

    // Large dataset — parallel is faster
    let start = Instant::now();
    let _: i32 = large_data.iter().map(|x| x * 2).sum();
    let seq_large = start.elapsed();

    let start = Instant::now();
    let _: i32 = large_data.par_iter().map(|x| x * 2).sum();
    let par_large = start.elapsed();

    println!("\nLarge dataset (1M items):");
    println!("  Sequential: {:?}", seq_large);
    println!("  Parallel: {:?}", par_large);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `rayon` | Library for data parallelism in Rust |
| `par_iter()` | Parallel immutable iterator |
| `par_iter_mut()` | Parallel mutable iterator |
| `into_par_iter()` | Parallel consuming iterator |
| Work stealing | Automatic load balancing between threads |
| `reduce()` | Combine parallel results with identity element |
| `par_sort()` | Parallel sorting |
| `ThreadPoolBuilder` | Custom thread pool configuration |

## Exercises

1. **Price Analysis**: Create a function that takes a vector of 10,000 price points and uses rayon to calculate the moving average with a window of 20. Compare performance with the sequential version.

2. **Multi-Asset Filter**: Given a list of 1000 assets with their daily returns, use parallel iterators to filter those with returns > 5% and volume > 1 million.

3. **Portfolio Optimization**: Implement parallel calculation of Sharpe ratio for multiple asset combinations in a portfolio.

4. **Order Book Aggregation**: Given order books from 10 different exchanges, use rayon to aggregate bid/ask prices and find the best execution price in parallel.

## Homework

1. **Parallel Technical Indicators**: Implement a system that calculates multiple technical indicators (RSI, MACD, Bollinger Bands) for a list of 100 assets in parallel. Measure the speedup compared to sequential processing.

2. **Strategy Grid Search**: Create a parallel grid search that tests 1000 different parameter combinations for a simple trading strategy. Use rayon's `into_par_iter()` to process all combinations and find the optimal parameters.

3. **Risk Calculator**: Build a Value at Risk (VaR) calculator that runs 10,000 Monte Carlo simulations in parallel to estimate portfolio risk. Compare the execution time with a sequential implementation.

4. **Multi-Exchange Arbitrage Scanner**: Implement a scanner that checks for arbitrage opportunities across 5 exchanges for 100 trading pairs simultaneously. Use rayon to parallelize both the exchange loop and the trading pair loop.

## Navigation

[← Previous day](../172-crossbeam-scope-threads-borrowing/en.md) | [Next day →](../174-par-iter-parallel-trade-processing/en.md)
