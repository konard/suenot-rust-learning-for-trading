# Day 174: par_iter: Parallel Trade Processing

## Trading Analogy

Imagine you're managing a hedge fund and need to analyze 10,000 stocks to find the best candidates. If you analyze each stock sequentially, it will take a very long time. But if you have a team of 8 analysts, each can take a portion of the stocks and work in parallel — the work will be done 8 times faster!

In Rust, the **Rayon** library provides `par_iter()` — a parallel iterator that automatically distributes work across all CPU cores. It's like having a team of analysts where Rayon decides who analyzes which stock.

In real algorithmic trading, `par_iter` is used for:
- Parallel calculation of indicators across multiple instruments
- Simultaneous processing of orders from different exchanges
- Mass backtesting strategies on historical data
- Calculating risk metrics across the entire portfolio

## What is Rayon and par_iter?

**Rayon** is a data parallelism library for Rust. It allows you to easily turn sequential code into parallel code by simply replacing `.iter()` with `.par_iter()`.

### Main advantages:
- **Simplicity** — minimal code changes
- **Safety** — compiler guarantees no data races
- **Efficiency** — automatic load balancing (work-stealing)
- **Scalability** — automatically uses all available cores

## Installing Rayon

Add to `Cargo.toml`:

```toml
[dependencies]
rayon = "1.10"
```

## Simple Example: Calculating Trade Profits

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

impl Trade {
    fn profit(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.quantity
    }
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), entry_price: 40000.0, exit_price: 42000.0, quantity: 0.5 },
        Trade { symbol: "ETH".to_string(), entry_price: 2500.0, exit_price: 2700.0, quantity: 10.0 },
        Trade { symbol: "SOL".to_string(), entry_price: 100.0, exit_price: 95.0, quantity: 50.0 },
        Trade { symbol: "BNB".to_string(), entry_price: 300.0, exit_price: 320.0, quantity: 5.0 },
    ];

    // Sequential calculation
    let sequential_profit: f64 = trades.iter()
        .map(|t| t.profit())
        .sum();

    // Parallel calculation — just replace iter() with par_iter()!
    let parallel_profit: f64 = trades.par_iter()
        .map(|t| t.profit())
        .sum();

    println!("Sequential: ${:.2}", sequential_profit);
    println!("Parallel: ${:.2}", parallel_profit);
    // Results are identical: $3100.00
}
```

## Performance Comparison

Let's compare performance on a large dataset:

```rust
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone)]
struct OHLCV {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// Volatility calculation — relatively heavy operation
fn calculate_volatility(candles: &[OHLCV]) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }

    let returns: Vec<f64> = candles.windows(2)
        .map(|w| (w[1].close / w[0].close).ln())
        .collect();

    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;

    variance.sqrt() * (252.0_f64).sqrt() // Annualized volatility
}

fn main() {
    // Generate data for 1000 stocks, 1000 candles each
    let stocks: Vec<Vec<OHLCV>> = (0..1000)
        .map(|_| {
            (0..1000)
                .map(|i| OHLCV {
                    open: 100.0 + (i as f64 * 0.1).sin() * 10.0,
                    high: 105.0 + (i as f64 * 0.1).sin() * 10.0,
                    low: 95.0 + (i as f64 * 0.1).sin() * 10.0,
                    close: 100.0 + (i as f64 * 0.1).cos() * 10.0,
                    volume: 1000000.0,
                })
                .collect()
        })
        .collect();

    // Sequential calculation
    let start = Instant::now();
    let _sequential: Vec<f64> = stocks.iter()
        .map(|candles| calculate_volatility(candles))
        .collect();
    let sequential_time = start.elapsed();

    // Parallel calculation
    let start = Instant::now();
    let _parallel: Vec<f64> = stocks.par_iter()
        .map(|candles| calculate_volatility(candles))
        .collect();
    let parallel_time = start.elapsed();

    println!("Sequential: {:?}", sequential_time);
    println!("Parallel: {:?}", parallel_time);
    println!("Speedup: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
}
```

On an 8-core processor, you can expect 4-7x speedup!

## Parallel Signal Filtering

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    signal_strength: f64,  // from -1.0 (sell) to 1.0 (buy)
    volume_ratio: f64,     // volume to average ratio
    rsi: f64,              // RSI indicator (0-100)
}

impl TradingSignal {
    fn is_strong_buy(&self) -> bool {
        self.signal_strength > 0.7
            && self.volume_ratio > 1.5
            && self.rsi < 30.0
    }

    fn is_strong_sell(&self) -> bool {
        self.signal_strength < -0.7
            && self.volume_ratio > 1.5
            && self.rsi > 70.0
    }
}

fn main() {
    // Generate signals for 10000 instruments
    let signals: Vec<TradingSignal> = (0..10000)
        .map(|i| TradingSignal {
            symbol: format!("STOCK{}", i),
            signal_strength: ((i as f64 * 0.001).sin()),
            volume_ratio: 1.0 + (i as f64 * 0.002).cos().abs(),
            rsi: 50.0 + ((i as f64 * 0.003).sin() * 40.0),
        })
        .collect();

    // Find strong buy signals in parallel
    let buy_signals: Vec<&TradingSignal> = signals.par_iter()
        .filter(|s| s.is_strong_buy())
        .collect();

    // Find strong sell signals in parallel
    let sell_signals: Vec<&TradingSignal> = signals.par_iter()
        .filter(|s| s.is_strong_sell())
        .collect();

    println!("Buy signals found: {}", buy_signals.len());
    println!("Sell signals found: {}", sell_signals.len());

    // Display first 5 buy signals
    for signal in buy_signals.iter().take(5) {
        println!("  {} - strength: {:.2}, RSI: {:.1}",
            signal.symbol, signal.signal_strength, signal.rsi);
    }
}
```

## Parallel Portfolio Processing

```rust
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    current_price: f64,
    daily_change: f64,
}

#[derive(Debug)]
struct PositionAnalysis {
    symbol: String,
    pnl: f64,
    pnl_percent: f64,
    market_value: f64,
}

fn analyze_position(position: &Position, market_data: &HashMap<String, MarketData>) -> PositionAnalysis {
    let market = market_data.get(&position.symbol)
        .unwrap_or(&MarketData {
            symbol: position.symbol.clone(),
            current_price: position.avg_price,
            daily_change: 0.0,
        });

    let market_value = position.quantity * market.current_price;
    let cost_basis = position.quantity * position.avg_price;
    let pnl = market_value - cost_basis;
    let pnl_percent = (pnl / cost_basis) * 100.0;

    PositionAnalysis {
        symbol: position.symbol.clone(),
        pnl,
        pnl_percent,
        market_value,
    }
}

fn main() {
    // Portfolio with multiple positions
    let positions: Vec<Position> = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.0, avg_price: 40000.0 },
        Position { symbol: "ETH".to_string(), quantity: 50.0, avg_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 500.0, avg_price: 100.0 },
        Position { symbol: "BNB".to_string(), quantity: 100.0, avg_price: 300.0 },
        Position { symbol: "XRP".to_string(), quantity: 10000.0, avg_price: 0.5 },
        Position { symbol: "ADA".to_string(), quantity: 20000.0, avg_price: 0.4 },
        Position { symbol: "DOT".to_string(), quantity: 1000.0, avg_price: 7.0 },
        Position { symbol: "AVAX".to_string(), quantity: 200.0, avg_price: 35.0 },
    ];

    // Current market data
    let market_data: HashMap<String, MarketData> = [
        ("BTC", 42000.0, 2.5),
        ("ETH", 2700.0, 3.0),
        ("SOL", 110.0, 5.0),
        ("BNB", 320.0, 1.5),
        ("XRP", 0.55, -2.0),
        ("ADA", 0.45, 4.0),
        ("DOT", 7.5, 2.0),
        ("AVAX", 38.0, 3.5),
    ].iter()
    .map(|(symbol, price, change)| {
        (symbol.to_string(), MarketData {
            symbol: symbol.to_string(),
            current_price: *price,
            daily_change: *change,
        })
    })
    .collect();

    // Parallel analysis of all positions
    let analyses: Vec<PositionAnalysis> = positions.par_iter()
        .map(|pos| analyze_position(pos, &market_data))
        .collect();

    // Parallel calculation of total metrics
    let total_pnl: f64 = analyses.par_iter().map(|a| a.pnl).sum();
    let total_value: f64 = analyses.par_iter().map(|a| a.market_value).sum();

    println!("=== Portfolio Analysis ===\n");
    for analysis in &analyses {
        println!("{}: P&L ${:.2} ({:+.2}%), Value: ${:.2}",
            analysis.symbol, analysis.pnl, analysis.pnl_percent, analysis.market_value);
    }
    println!("\n=== Summary ===");
    println!("Total P&L: ${:.2}", total_pnl);
    println!("Total Value: ${:.2}", total_value);
}
```

## Parallel Strategy Backtesting

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone)]
struct BacktestResult {
    strategy_name: String,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    trades_count: u32,
}

// Simplified backtest simulation (in reality, this involves complex calculations)
fn backtest_strategy(strategy: &Strategy, prices: &[f64]) -> BacktestResult {
    // Simplified simulation for demonstration
    let volatility = strategy.fast_period as f64 / strategy.slow_period as f64;
    let base_return = (prices.last().unwrap_or(&100.0) / prices.first().unwrap_or(&100.0) - 1.0) * 100.0;

    BacktestResult {
        strategy_name: strategy.name.clone(),
        total_return: base_return * volatility,
        sharpe_ratio: (base_return * volatility) / 15.0,
        max_drawdown: 10.0 + volatility * 5.0,
        win_rate: 0.45 + volatility * 0.1,
        trades_count: (250.0 / (strategy.fast_period as f64 + strategy.slow_period as f64) * 10.0) as u32,
    }
}

fn main() {
    // Generate multiple strategies for optimization
    let strategies: Vec<Strategy> = (5..50)
        .flat_map(|fast| {
            (20..200).step_by(10).map(move |slow| {
                Strategy {
                    name: format!("MA_{}_{}", fast, slow),
                    fast_period: fast,
                    slow_period: slow,
                    stop_loss: 0.02,
                    take_profit: 0.05,
                }
            })
        })
        .filter(|s| s.fast_period < s.slow_period)
        .collect();

    println!("Testing {} strategies...", strategies.len());

    // Historical prices (simulation)
    let prices: Vec<f64> = (0..1000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 20.0)
        .collect();

    // Parallel backtest of all strategies
    let mut results: Vec<BacktestResult> = strategies.par_iter()
        .map(|strategy| backtest_strategy(strategy, &prices))
        .collect();

    // Sort by Sharpe Ratio
    results.sort_by(|a, b| b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap());

    println!("\n=== Top 5 Strategies by Sharpe Ratio ===\n");
    for result in results.iter().take(5) {
        println!("{}: Return {:.2}%, Sharpe {:.2}, MaxDD {:.2}%, WinRate {:.2}%",
            result.strategy_name,
            result.total_return,
            result.sharpe_ratio,
            result.max_drawdown,
            result.win_rate * 100.0
        );
    }
}
```

## Parallel Aggregation with reduce

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct DailyStats {
    date: String,
    trades: u32,
    volume: f64,
    pnl: f64,
    fees: f64,
}

#[derive(Debug, Clone, Default)]
struct AggregatedStats {
    total_trades: u32,
    total_volume: f64,
    total_pnl: f64,
    total_fees: f64,
    best_day_pnl: f64,
    worst_day_pnl: f64,
}

fn main() {
    // Daily statistics for a year
    let daily_stats: Vec<DailyStats> = (0..365)
        .map(|day| {
            let pnl = ((day as f64 * 0.1).sin() * 1000.0) + 100.0;
            DailyStats {
                date: format!("2024-{:02}-{:02}", (day / 30) + 1, (day % 30) + 1),
                trades: 50 + (day % 30),
                volume: 1000000.0 + (day as f64 * 1000.0),
                pnl,
                fees: 50.0 + (day as f64 * 0.5),
            }
        })
        .collect();

    // Parallel aggregation with reduce
    let aggregated = daily_stats.par_iter()
        .map(|day| AggregatedStats {
            total_trades: day.trades,
            total_volume: day.volume,
            total_pnl: day.pnl,
            total_fees: day.fees,
            best_day_pnl: day.pnl,
            worst_day_pnl: day.pnl,
        })
        .reduce(
            || AggregatedStats {
                best_day_pnl: f64::MIN,
                worst_day_pnl: f64::MAX,
                ..Default::default()
            },
            |mut acc, stats| {
                acc.total_trades += stats.total_trades;
                acc.total_volume += stats.total_volume;
                acc.total_pnl += stats.total_pnl;
                acc.total_fees += stats.total_fees;
                acc.best_day_pnl = acc.best_day_pnl.max(stats.best_day_pnl);
                acc.worst_day_pnl = acc.worst_day_pnl.min(stats.worst_day_pnl);
                acc
            }
        );

    println!("=== Yearly Statistics ===\n");
    println!("Total trades: {}", aggregated.total_trades);
    println!("Total volume: ${:.2}", aggregated.total_volume);
    println!("Total P&L: ${:.2}", aggregated.total_pnl);
    println!("Total fees: ${:.2}", aggregated.total_fees);
    println!("Best day: ${:.2}", aggregated.best_day_pnl);
    println!("Worst day: ${:.2}", aggregated.worst_day_pnl);
    println!("Net profit: ${:.2}", aggregated.total_pnl - aggregated.total_fees);
}
```

## Parallel Order Processing with par_iter_mut

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
enum OrderStatus {
    New,
    Validated,
    Rejected(String),
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    status: OrderStatus,
}

fn validate_order(order: &mut Order, available_balance: f64, max_position: f64) {
    let order_value = order.quantity * order.price;

    if order.quantity <= 0.0 {
        order.status = OrderStatus::Rejected("Quantity must be positive".to_string());
    } else if order.price <= 0.0 {
        order.status = OrderStatus::Rejected("Price must be positive".to_string());
    } else if order_value > available_balance {
        order.status = OrderStatus::Rejected(format!(
            "Insufficient funds: need {:.2}, have {:.2}",
            order_value, available_balance
        ));
    } else if order.quantity > max_position {
        order.status = OrderStatus::Rejected(format!(
            "Position limit exceeded: {} > {}",
            order.quantity, max_position
        ));
    } else {
        order.status = OrderStatus::Validated;
    }
}

fn main() {
    let available_balance = 100_000.0;
    let max_position = 100.0;

    // Create many orders for validation
    let mut orders: Vec<Order> = (0..10000)
        .map(|i| Order {
            id: i,
            symbol: format!("STOCK{}", i % 100),
            quantity: (i % 150) as f64,
            price: 100.0 + (i % 50) as f64,
            status: OrderStatus::New,
        })
        .collect();

    // Parallel validation of all orders
    orders.par_iter_mut()
        .for_each(|order| validate_order(order, available_balance, max_position));

    // Count results
    let validated: Vec<_> = orders.iter()
        .filter(|o| matches!(o.status, OrderStatus::Validated))
        .collect();

    let rejected: Vec<_> = orders.iter()
        .filter(|o| matches!(o.status, OrderStatus::Rejected(_)))
        .collect();

    println!("Total orders: {}", orders.len());
    println!("Validated: {}", validated.len());
    println!("Rejected: {}", rejected.len());

    // Show examples of rejected orders
    println!("\nExamples of rejected orders:");
    for order in rejected.iter().take(5) {
        if let OrderStatus::Rejected(reason) = &order.status {
            println!("  Order {}: {}", order.id, reason);
        }
    }
}
```

## When to Use par_iter

### When par_iter is effective:

```rust
use rayon::prelude::*;

// Good: many elements, each operation takes time
let results: Vec<_> = large_dataset.par_iter()
    .map(|item| expensive_calculation(item))
    .collect();

// Good: independent calculations
let stats: Vec<_> = symbols.par_iter()
    .map(|symbol| calculate_technical_indicators(symbol))
    .collect();
```

### When to use regular iter instead:

```rust
// Bad: too few elements
let small_data = vec![1, 2, 3, 4, 5];
let sum: i32 = small_data.par_iter().sum(); // Overhead > benefit

// Bad: operations too simple
let doubled: Vec<_> = numbers.par_iter()
    .map(|x| x * 2) // Operation is too fast
    .collect();

// Bad: operations require synchronization
let shared_state = Arc::new(Mutex::new(0));
data.par_iter().for_each(|x| {
    *shared_state.lock().unwrap() += x; // Lock kills parallelism
});
```

## Thread Pool Configuration

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Create a pool with a specific number of threads
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    let data: Vec<i32> = (0..1000).collect();

    // Execute work in our pool
    let result: i32 = pool.install(|| {
        data.par_iter()
            .map(|x| x * x)
            .sum()
    });

    println!("Result: {}", result);

    // You can set a global pool at program start
    // ThreadPoolBuilder::new()
    //     .num_threads(8)
    //     .build_global()
    //     .unwrap();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `par_iter()` | Parallel iterator for immutable data |
| `par_iter_mut()` | Parallel iterator for mutable data |
| `into_par_iter()` | Parallel iterator that takes ownership |
| `reduce()` | Parallel aggregation with result combining |
| Work-stealing | Automatic redistribution of work between threads |
| ThreadPool | Configuration of worker thread count |

## Homework

1. **Parallel Stock Screener**: Create a `Stock` struct with fields (symbol, price, volume, pe_ratio, market_cap) and implement a parallel screening function that filters stocks by multiple criteria (P/E < 15, volume > 1M, price growth > 5%).

2. **Parallel Correlation Calculation**: Write a function that calculates correlations between all pairs of assets in a portfolio in parallel. Use `par_iter` to process all pair combinations.

3. **Strategy Parameter Optimization**: Implement parallel parameter search for a trading strategy (MA period, stop-loss and take-profit levels). Find the combination with the best risk-adjusted return.

4. **Parallel Tick Processing**: Create a system that processes tick data streams from different exchanges in parallel, aggregating OHLCV candles for each instrument.

## Navigation

[← Day 164: Deadlock](../164-deadlock-threads-block/en.md) | [Next day →](../175-threadpool-work-distribution/en.md)
