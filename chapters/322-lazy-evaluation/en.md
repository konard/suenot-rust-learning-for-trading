# Day 322: Lazy Evaluation

## Trading Analogy

Imagine you're running a trading bot that monitors 1000 cryptocurrencies. Every second you receive price updates and need to recalculate indicators (SMA, EMA, RSI, MACD) for each pair.

**Eager approach** — is like recalculating ALL indicators for ALL pairs every second:
- 1000 pairs × 10 indicators × 60 seconds = 600,000 calculations per minute
- 99% of them are unnecessary if the price hasn't changed
- Resources are wasted

**Lazy approach** — like an experienced trader:
- Recalculate an indicator only when its value is ACTUALLY needed
- If the strategy only checks BTC/USDT — don't calculate the other 999 pairs
- If the price hasn't changed — use the cached value

In trading, lazy evaluation is critical for:
- Saving CPU in HFT systems
- Reducing decision-making latency
- Efficiently processing large data volumes
- Optimizing backtesting on historical data

## What is Lazy Evaluation?

**Lazy Evaluation** — is an expression evaluation strategy where computation is deferred until the result is actually needed.

### Comparison of Approaches

```
┌─────────────────────────────────────────────────────────────────┐
│             Eager vs Lazy Evaluation                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EAGER:                                                         │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │ Input 1 │───▶│ Compute │───▶│ Result 1│    │ Used?   │      │
│  └─────────┘    └─────────┘    └─────────┘    │ Maybe   │      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    └─────────┘      │
│  │ Input 2 │───▶│ Compute │───▶│ Result 2│    │ Maybe   │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│                                                                 │
│  LAZY:                                                          │
│  ┌─────────┐              ┌─────────┐    ┌─────────┐           │
│  │ Input 1 │─ ─ ─ ─ ─ ─ ▶│ Compute │───▶│ Result 1│           │
│  └─────────┘   (delayed)  └─────────┘    └─────────┘           │
│       │                        ▲                                │
│       │                        │                                │
│       └────────────────────────┘                                │
│          Only when result is needed                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Benefits of Lazy Evaluation

| Benefit | Description | Trading Example |
|---------|-------------|-----------------|
| **CPU savings** | Compute only on demand | Calculate indicators only for active strategies |
| **Memory savings** | Don't store unnecessary results | Process 1M candles without loading all into memory |
| **Infinite structures** | Work with potentially infinite data | Infinite price stream from exchange |
| **Short-circuiting** | Stop computation on early result | Stop-loss triggered — don't calculate profit target |

## Lazy Evaluation in Rust

### Iterators — the Main Tool

```rust
/// Demonstration of lazy iterators in trading context
fn main() {
    // Simulating price stream
    let prices: Vec<f64> = vec![
        50000.0, 50100.0, 49900.0, 50200.0, 50150.0,
        50300.0, 50250.0, 50400.0, 50350.0, 50500.0,
    ];

    // EAGER: Compute ALL percentage changes at once
    let all_changes_eager: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect(); // <- collect() forces computation of ALL elements

    println!("Eager: computed {} changes", all_changes_eager.len());

    // LAZY: Compute only needed changes
    let significant_changes: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0) // Lazy operation
        .filter(|&change| change.abs() > 0.1)   // Lazy operation
        .take(3)                                  // Lazy operation
        .collect();                              // Computation starts here

    println!("Lazy: found {} significant changes: {:?}",
             significant_changes.len(), significant_changes);
}
```

### Lazy Iterator Adapters

```rust
/// Chain of lazy operations for trading data analysis
struct Trade {
    timestamp: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn analyze_trades_lazy(trades: &[Trade]) {
    // All these operations are LAZY — nothing is computed until collect/for_each
    let btc_buys = trades.iter()
        .filter(|t| t.symbol == "BTCUSDT")      // Lazy
        .filter(|t| t.side == "BUY")            // Lazy
        .map(|t| t.price * t.quantity)          // Lazy
        .take_while(|&volume| volume < 100000.0); // Lazy

    // Processing starts only now
    let total_volume: f64 = btc_buys.sum();
    println!("BTC buy volume (before first large trade): ${:.2}", total_volume);
}

fn main() {
    let trades = vec![
        Trade { timestamp: 1704067200, symbol: "BTCUSDT".to_string(),
                price: 50000.0, quantity: 0.5, side: "BUY".to_string() },
        Trade { timestamp: 1704067201, symbol: "ETHUSDT".to_string(),
                price: 2500.0, quantity: 10.0, side: "SELL".to_string() },
        Trade { timestamp: 1704067202, symbol: "BTCUSDT".to_string(),
                price: 50100.0, quantity: 1.0, side: "BUY".to_string() },
        Trade { timestamp: 1704067203, symbol: "BTCUSDT".to_string(),
                price: 50200.0, quantity: 3.0, side: "BUY".to_string() },
    ];

    analyze_trades_lazy(&trades);
}
```

## Practical Example: Lazy Indicator Calculation

### Structure with Deferred Computation

```rust
use std::cell::RefCell;

/// Lazy SMA calculator — computes only when accessed
struct LazySMA {
    prices: Vec<f64>,
    period: usize,
    // Cache to avoid repeated computations
    cached_value: RefCell<Option<f64>>,
}

impl LazySMA {
    fn new(period: usize) -> Self {
        LazySMA {
            prices: Vec::new(),
            period,
            cached_value: RefCell::new(None),
        }
    }

    fn add_price(&mut self, price: f64) {
        self.prices.push(price);
        // Invalidate cache when new price is added
        *self.cached_value.borrow_mut() = None;
    }

    /// Lazy value retrieval — compute only if needed
    fn get(&self) -> Option<f64> {
        // Check cache
        if let Some(cached) = *self.cached_value.borrow() {
            return Some(cached);
        }

        // Compute only if there's enough data
        if self.prices.len() < self.period {
            return None;
        }

        // Calculate SMA
        let sum: f64 = self.prices[self.prices.len() - self.period..].iter().sum();
        let sma = sum / self.period as f64;

        // Cache the result
        *self.cached_value.borrow_mut() = Some(sma);

        Some(sma)
    }
}

fn main() {
    let mut sma = LazySMA::new(3);

    // Add prices — SMA not computed yet
    sma.add_price(100.0);
    sma.add_price(102.0);
    sma.add_price(101.0);
    println!("Prices added, SMA not yet computed");

    // Computation happens only now
    println!("SMA(3) = {:?}", sma.get());

    // Repeated call uses cache
    println!("SMA(3) from cache = {:?}", sma.get());

    // Add new price — cache invalidated
    sma.add_price(103.0);
    println!("SMA(3) after new price = {:?}", sma.get());
}
```

### Lazy Indicator Using Closures

```rust
use std::cell::RefCell;

/// Universal lazy indicator
struct LazyIndicator<F>
where
    F: Fn(&[f64]) -> Option<f64>,
{
    prices: Vec<f64>,
    compute: F,
    cached: RefCell<Option<f64>>,
    dirty: RefCell<bool>,
}

impl<F> LazyIndicator<F>
where
    F: Fn(&[f64]) -> Option<f64>,
{
    fn new(compute: F) -> Self {
        LazyIndicator {
            prices: Vec::new(),
            compute,
            cached: RefCell::new(None),
            dirty: RefCell::new(true),
        }
    }

    fn push(&mut self, price: f64) {
        self.prices.push(price);
        *self.dirty.borrow_mut() = true;
    }

    fn value(&self) -> Option<f64> {
        if !*self.dirty.borrow() {
            return *self.cached.borrow();
        }

        let result = (self.compute)(&self.prices);
        *self.cached.borrow_mut() = result;
        *self.dirty.borrow_mut() = false;
        result
    }
}

fn main() {
    // Lazy RSI
    let rsi_compute = |prices: &[f64]| -> Option<f64> {
        if prices.len() < 15 {
            return None;
        }

        let period = 14;
        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (prices.len() - period)..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    };

    let mut rsi = LazyIndicator::new(rsi_compute);

    // Add 20 prices
    for i in 0..20 {
        rsi.push(50000.0 + (i as f64 * 100.0 * ((i as f64 * 0.5).sin())));
    }

    // RSI is computed only now
    println!("RSI = {:?}", rsi.value());
    // Repeated call — from cache
    println!("RSI (cached) = {:?}", rsi.value());
}
```

## Lazy Iterators for Stream Processing

### Processing Millions of Candles Without Loading into Memory

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

/// OHLCV candle
#[derive(Debug)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            return None;
        }

        Some(Candle {
            timestamp: parts[0].parse().ok()?,
            open: parts[1].parse().ok()?,
            high: parts[2].parse().ok()?,
            low: parts[3].parse().ok()?,
            close: parts[4].parse().ok()?,
            volume: parts[5].parse().ok()?,
        })
    }

    fn body_percent(&self) -> f64 {
        ((self.close - self.open) / self.open).abs() * 100.0
    }
}

/// Lazy iterator over candles from file
struct CandleIterator<R: BufRead> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> CandleIterator<R> {
    fn new(reader: R) -> Self {
        CandleIterator {
            reader,
            buffer: String::new(),
        }
    }
}

impl<R: BufRead> Iterator for CandleIterator<R> {
    type Item = Candle;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        loop {
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    if let Some(candle) = Candle::from_csv_line(self.buffer.trim()) {
                        return Some(candle);
                    }
                    self.buffer.clear();
                    // Skip invalid lines
                }
                Err(_) => return None,
            }
        }
    }
}

fn analyze_large_file_lazy(filename: &str) {
    // Example analysis of a large file
    // In real code this would be File::open(filename)

    // Data simulation for demonstration
    let data = "1704067200,50000,50100,49900,50050,1000
1704070800,50050,50200,49950,50150,1500
1704074400,50150,50500,50100,50450,2000
1704078000,50450,50600,50300,50350,1800
1704081600,50350,50400,49800,49850,2500";

    let reader = BufReader::new(data.as_bytes());
    let candles = CandleIterator::new(reader);

    // Lazy chain of operations
    let strong_candles: Vec<_> = candles
        .filter(|c| c.volume > 1000.0)        // Only high-volume candles
        .filter(|c| c.body_percent() > 0.5)   // Only with large body
        .take(2)                               // Take first 2
        .collect();

    println!("Found {} strong candles", strong_candles.len());
    for candle in &strong_candles {
        println!("  {:?}", candle);
    }
}

fn main() {
    analyze_large_file_lazy("candles.csv");
}
```

### Infinite Price Stream

```rust
use std::time::{Duration, Instant};

/// Generator of infinite price stream
struct PriceStream {
    current_price: f64,
    volatility: f64,
    tick: u64,
}

impl PriceStream {
    fn new(initial_price: f64, volatility: f64) -> Self {
        PriceStream {
            current_price: initial_price,
            volatility,
            tick: 0,
        }
    }
}

impl Iterator for PriceStream {
    type Item = (u64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        // Infinite iterator — always returns Some
        self.tick += 1;

        // Simulate random price movement
        let change = ((self.tick as f64 * 0.1).sin() * self.volatility)
                   + ((self.tick as f64 * 0.03).cos() * self.volatility * 0.5);
        self.current_price *= 1.0 + change / 100.0;

        Some((self.tick, self.current_price))
    }
}

fn main() {
    println!("=== Infinite price stream (taking only 10) ===\n");

    let stream = PriceStream::new(50000.0, 0.1);

    // Take only first 10 prices from infinite stream
    // Without take() the program would run forever!
    let prices: Vec<_> = stream
        .take(10)
        .collect();

    for (tick, price) in &prices {
        println!("Tick {}: ${:.2}", tick, price);
    }

    println!("\n=== Finding first significant move ===\n");

    let stream = PriceStream::new(50000.0, 0.5);

    // Lazy search — stops on first found
    let big_move = stream
        .take(1000)
        .enumerate()
        .find(|(i, (_, price))| {
            if *i > 0 {
                let prev_price = 50000.0 * (1.0 + (*i as f64 * 0.001).sin() * 0.005);
                (price - prev_price).abs() / prev_price > 0.01
            } else {
                false
            }
        });

    match big_move {
        Some((idx, (tick, price))) => {
            println!("Significant move at tick {} (index {}): ${:.2}", tick, idx, price);
        }
        None => println!("No significant moves found"),
    }
}
```

## Lazy Initialization with OnceCell and LazyCell

### Deferred Configuration Loading

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

/// Global configuration with lazy initialization
static CONFIG: OnceLock<TradingConfig> = OnceLock::new();

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    api_secret: String,
    symbols: Vec<String>,
    risk_limit: f64,
}

impl TradingConfig {
    fn load() -> Self {
        println!("[CONFIG] Loading configuration...");
        // Simulate loading from file/env
        TradingConfig {
            api_key: "your_api_key".to_string(),
            api_secret: "your_api_secret".to_string(),
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            risk_limit: 0.02,
        }
    }
}

fn get_config() -> &'static TradingConfig {
    CONFIG.get_or_init(|| TradingConfig::load())
}

fn main() {
    println!("Program started");
    println!("CONFIG not yet initialized");

    // First call — initialization happens
    let config = get_config();
    println!("\nConfiguration loaded:");
    println!("  Symbols: {:?}", config.symbols);
    println!("  Risk limit: {:.1}%", config.risk_limit * 100.0);

    // Repeated call — uses cached value
    println!("\nRepeated call:");
    let config2 = get_config();
    println!("  Symbols: {:?}", config2.symbols);
}
```

### Lazy Computations for Strategy

```rust
use std::cell::OnceCell;

/// Trading strategy with lazy indicators
struct TradingStrategy {
    prices: Vec<f64>,
    // Lazy indicators — computed only when accessed
    sma_20: OnceCell<f64>,
    sma_50: OnceCell<f64>,
    rsi_14: OnceCell<f64>,
    volatility: OnceCell<f64>,
}

impl TradingStrategy {
    fn new(prices: Vec<f64>) -> Self {
        TradingStrategy {
            prices,
            sma_20: OnceCell::new(),
            sma_50: OnceCell::new(),
            rsi_14: OnceCell::new(),
            volatility: OnceCell::new(),
        }
    }

    fn get_sma(&self, period: usize) -> Option<f64> {
        if self.prices.len() < period {
            return None;
        }
        let sum: f64 = self.prices[self.prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    fn sma_20(&self) -> Option<f64> {
        self.sma_20.get_or_init(|| {
            println!("[COMPUTE] Computing SMA(20)...");
            self.get_sma(20).unwrap_or(0.0)
        });
        self.sma_20.get().copied()
    }

    fn sma_50(&self) -> Option<f64> {
        self.sma_50.get_or_init(|| {
            println!("[COMPUTE] Computing SMA(50)...");
            self.get_sma(50).unwrap_or(0.0)
        });
        self.sma_50.get().copied()
    }

    fn rsi_14(&self) -> Option<f64> {
        self.rsi_14.get_or_init(|| {
            println!("[COMPUTE] Computing RSI(14)...");
            // Simplified RSI calculation
            if self.prices.len() < 15 {
                return 50.0;
            }
            let mut gains = 0.0;
            let mut losses = 0.0;
            for i in (self.prices.len() - 14)..self.prices.len() {
                let change = self.prices[i] - self.prices[i - 1];
                if change > 0.0 { gains += change; }
                else { losses += change.abs(); }
            }
            if losses == 0.0 { 100.0 }
            else { 100.0 - (100.0 / (1.0 + gains / losses)) }
        });
        self.rsi_14.get().copied()
    }

    fn volatility(&self) -> Option<f64> {
        self.volatility.get_or_init(|| {
            println!("[COMPUTE] Computing volatility...");
            if self.prices.len() < 20 {
                return 0.0;
            }
            let slice = &self.prices[self.prices.len() - 20..];
            let mean: f64 = slice.iter().sum::<f64>() / 20.0;
            let variance: f64 = slice.iter()
                .map(|p| (p - mean).powi(2))
                .sum::<f64>() / 20.0;
            variance.sqrt()
        });
        self.volatility.get().copied()
    }

    /// Signal generation — uses only needed indicators
    fn generate_signal(&self) -> &'static str {
        // First check RSI — if overbought/oversold, rest doesn't matter
        if let Some(rsi) = self.rsi_14() {
            if rsi > 70.0 {
                return "SELL (RSI overbought)";
            }
            if rsi < 30.0 {
                return "BUY (RSI oversold)";
            }
        }

        // Check SMA crossover only if RSI is neutral
        let sma_20 = self.sma_20();
        let sma_50 = self.sma_50();

        match (sma_20, sma_50) {
            (Some(fast), Some(slow)) if fast > slow => "BUY (SMA crossover)",
            (Some(fast), Some(slow)) if fast < slow => "SELL (SMA crossunder)",
            _ => "HOLD",
        }
    }
}

fn main() {
    println!("=== Lazy indicators in trading strategy ===\n");

    // Generate 100 prices
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 0.1).sin() * 1000.0)
        .collect();

    let strategy = TradingStrategy::new(prices);

    println!("Strategy created, indicators not yet computed\n");

    // Generate signal — only needed indicators are computed
    println!("Generating signal:");
    let signal = strategy.generate_signal();
    println!("Signal: {}\n", signal);

    // Repeated call — all from cache
    println!("Repeated signal generation:");
    let signal2 = strategy.generate_signal();
    println!("Signal: {} (no computations)", signal2);
}
```

## Lazy Evaluation for Backtesting Optimization

### Lazy Backtester

```rust
use std::collections::HashMap;
use std::cell::OnceCell;

/// Trade result
#[derive(Debug, Clone)]
struct TradeResult {
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

/// Strategy metrics — computed lazily
struct BacktestMetrics {
    trades: Vec<TradeResult>,
    // Caches for lazy metrics
    total_pnl: OnceCell<f64>,
    win_rate: OnceCell<f64>,
    max_drawdown: OnceCell<f64>,
    sharpe_ratio: OnceCell<f64>,
    profit_factor: OnceCell<f64>,
}

impl BacktestMetrics {
    fn new(trades: Vec<TradeResult>) -> Self {
        BacktestMetrics {
            trades,
            total_pnl: OnceCell::new(),
            win_rate: OnceCell::new(),
            max_drawdown: OnceCell::new(),
            sharpe_ratio: OnceCell::new(),
            profit_factor: OnceCell::new(),
        }
    }

    fn total_pnl(&self) -> f64 {
        *self.total_pnl.get_or_init(|| {
            println!("[METRICS] Computing Total PnL...");
            self.trades.iter().map(|t| t.pnl).sum()
        })
    }

    fn win_rate(&self) -> f64 {
        *self.win_rate.get_or_init(|| {
            println!("[METRICS] Computing Win Rate...");
            if self.trades.is_empty() {
                return 0.0;
            }
            let wins = self.trades.iter().filter(|t| t.pnl > 0.0).count();
            wins as f64 / self.trades.len() as f64 * 100.0
        })
    }

    fn max_drawdown(&self) -> f64 {
        *self.max_drawdown.get_or_init(|| {
            println!("[METRICS] Computing Max Drawdown...");
            let mut peak = 0.0;
            let mut max_dd = 0.0;
            let mut equity = 0.0;

            for trade in &self.trades {
                equity += trade.pnl;
                if equity > peak {
                    peak = equity;
                }
                let dd = (peak - equity) / peak.max(1.0);
                if dd > max_dd {
                    max_dd = dd;
                }
            }
            max_dd * 100.0
        })
    }

    fn sharpe_ratio(&self) -> f64 {
        *self.sharpe_ratio.get_or_init(|| {
            println!("[METRICS] Computing Sharpe Ratio...");
            if self.trades.len() < 2 {
                return 0.0;
            }

            let returns: Vec<f64> = self.trades.iter()
                .map(|t| t.pnl / t.entry_price / t.quantity)
                .collect();

            let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns.iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>() / returns.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev == 0.0 {
                return 0.0;
            }

            // Annualized (assuming daily returns)
            mean / std_dev * (252.0_f64).sqrt()
        })
    }

    fn profit_factor(&self) -> f64 {
        *self.profit_factor.get_or_init(|| {
            println!("[METRICS] Computing Profit Factor...");
            let gross_profit: f64 = self.trades.iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl)
                .sum();
            let gross_loss: f64 = self.trades.iter()
                .filter(|t| t.pnl < 0.0)
                .map(|t| t.pnl.abs())
                .sum();

            if gross_loss == 0.0 {
                return f64::INFINITY;
            }

            gross_profit / gross_loss
        })
    }

    /// Quick report — only basic metrics
    fn quick_report(&self) {
        println!("\n=== Quick Report ===");
        println!("Total PnL: ${:.2}", self.total_pnl());
        println!("Win Rate: {:.1}%", self.win_rate());
    }

    /// Full report — all metrics
    fn full_report(&self) {
        println!("\n=== Full Report ===");
        println!("Total PnL: ${:.2}", self.total_pnl());
        println!("Win Rate: {:.1}%", self.win_rate());
        println!("Max Drawdown: {:.1}%", self.max_drawdown());
        println!("Sharpe Ratio: {:.2}", self.sharpe_ratio());
        println!("Profit Factor: {:.2}", self.profit_factor());
    }
}

fn main() {
    println!("=== Lazy backtest metrics ===\n");

    // Simulate trading results
    let trades: Vec<TradeResult> = (0..100)
        .map(|i| {
            let entry = 50000.0 + (i as f64 * 100.0);
            let exit = entry * (1.0 + ((i as f64 * 0.3).sin() * 0.02));
            let quantity = 0.1;
            TradeResult {
                entry_price: entry,
                exit_price: exit,
                quantity,
                pnl: (exit - entry) * quantity,
            }
        })
        .collect();

    let metrics = BacktestMetrics::new(trades);

    println!("Metrics created, nothing computed\n");

    // Quick report — compute only 2 metrics
    println!("--- Quick report ---");
    metrics.quick_report();

    println!("\n--- Full report ---");
    // Full report — TotalPnL and WinRate already cached
    metrics.full_report();
}
```

## Performance Comparison: Eager vs Lazy

```rust
use std::time::Instant;

/// Benchmark comparing eager and lazy approaches
fn benchmark_eager_vs_lazy() {
    const NUM_PRICES: usize = 1_000_000;
    const SAMPLE_SIZE: usize = 100;

    // Generate large dataset
    let prices: Vec<f64> = (0..NUM_PRICES)
        .map(|i| 50000.0 + (i as f64 * 0.001).sin() * 5000.0)
        .collect();

    println!("=== Benchmark: Eager vs Lazy ===\n");
    println!("Data: {} prices", NUM_PRICES);
    println!("Looking for: first {} prices > $52000\n", SAMPLE_SIZE);

    // EAGER approach: compute everything, then filter
    let start = Instant::now();
    let filtered_eager: Vec<f64> = prices.clone()
        .into_iter()
        .filter(|&p| p > 52000.0)
        .collect();
    let result_eager: Vec<f64> = filtered_eager.into_iter().take(SAMPLE_SIZE).collect();
    let eager_time = start.elapsed();

    // LAZY approach: compute only what's needed
    let start = Instant::now();
    let result_lazy: Vec<f64> = prices.iter()
        .copied()
        .filter(|&p| p > 52000.0)
        .take(SAMPLE_SIZE)
        .collect();
    let lazy_time = start.elapsed();

    println!("EAGER:");
    println!("  Time: {:?}", eager_time);
    println!("  Result: {} elements", result_eager.len());

    println!("\nLAZY:");
    println!("  Time: {:?}", lazy_time);
    println!("  Result: {} elements", result_lazy.len());

    let speedup = eager_time.as_nanos() as f64 / lazy_time.as_nanos() as f64;
    println!("\nSpeedup: {:.1}x", speedup);
}

fn main() {
    benchmark_eager_vs_lazy();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Lazy Evaluation** | Deferred computation until the result is needed |
| **Iterators** | The main tool for lazy evaluation in Rust |
| **OnceCell/OnceLock** | Lazy initialization with caching |
| **Short-circuit** | Stopping computation when result is achieved |
| **Infinite iterators** | Working with infinite sequences |
| **Memoization** | Caching results to avoid repeated computations |

## Practical Exercises

1. **Lazy OrderBook**: Create an OrderBook structure that:
   - Computes spread only when accessed
   - Caches best bid/ask
   - Invalidates cache when the book changes

2. **Lazy CSV Parser**: Implement an iterator that:
   - Reads CSV file line by line
   - Parses a line only when next() is called
   - Supports filter/map/take without loading entire file

3. **Lazy Strategy**: Write a strategy that:
   - Has 10 different indicators
   - Computes indicators only if they're needed for signal
   - Uses short-circuit evaluation

4. **Benchmark**: Compare eager and lazy approaches on:
   - 10M candles
   - Finding first 10 candles matching a condition
   - Measure CPU and memory difference

## Homework

1. **Lazy Market Scanner**:
   - Scans 1000 cryptocurrencies
   - Computes indicators only for pairs that pass initial filter
   - Uses multi-level filtering (volume → volatility → pattern)
   - Measure CPU savings compared to eager approach

2. **Streaming Backtester**:
   - Processes historical data as a lazy stream
   - Doesn't load all data into memory
   - Supports stopping by condition (loss limit reached)
   - Outputs metrics in real-time

3. **Lazy Config System**:
   - Configuration loads only on first access
   - Supports hot-reload (reload on file change)
   - Validation happens lazily
   - Logs only actually used parameters

4. **Optimize Existing Code**:
   - Take your trading bot or example from previous chapters
   - Find places with eager computations
   - Convert them to lazy
   - Measure performance improvement

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../323-*/en.md)
