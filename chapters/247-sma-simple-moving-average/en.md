# Day 247: SMA: Simple Moving Average

## Trading Analogy

Imagine you're tracking Bitcoin's price. Every minute the price jumps: $42000, $42150, $41980, $42200... How do you determine whether the market is moving up or down? Looking at each individual price is useless — there's too much noise.

**Simple Moving Average (SMA)** is like "smoothing glasses" for a trader. It takes the last N prices and calculates their average value. This helps you see the overall trend by filtering out random fluctuations.

For example:
- **SMA(5)** — average of the last 5 periods (fast, sensitive to changes)
- **SMA(20)** — average of the last 20 periods (slow, smoother)
- **SMA(200)** — average of the last 200 periods (very slow, shows long-term trend)

When the fast SMA crosses above the slow SMA — it's a buy signal ("golden cross"). When it crosses below — it's a sell signal ("death cross").

## What is SMA?

**SMA (Simple Moving Average)** is a technical indicator calculated as the simple arithmetic mean of prices over a specific period:

```
SMA = (P1 + P2 + P3 + ... + Pn) / n
```

Where:
- `P1, P2, ..., Pn` — prices for the last n periods
- `n` — the period (window) of the moving average

### Why "Moving"?

Because with each new price value, the window "moves" forward:

```
Prices:   [100, 102, 101, 103, 105, 104, 106]
SMA(3):         [101, 102, 103, 104, 105]
                  ↑    ↑    ↑    ↑    ↑
               100+   102+  101+  103+  105+
               102+   101+  103+  105+  104+
               101    103   105   104   106
               ───    ───   ───   ───   ───
                3      3     3     3     3
```

## Basic SMA Implementation

```rust
/// Calculates the simple moving average for a vector of prices
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    for i in 0..=(prices.len() - period) {
        let window = &prices[i..i + period];
        let sum: f64 = window.iter().sum();
        let average = sum / period as f64;
        sma_values.push(average);
    }

    sma_values
}

fn main() {
    // Historical BTC prices (in dollars)
    let btc_prices = vec![
        42000.0, 42150.0, 41980.0, 42200.0, 42350.0,
        42100.0, 42400.0, 42550.0, 42300.0, 42600.0,
    ];

    println!("BTC Prices: {:?}", btc_prices);
    println!();

    // Calculate SMA with different periods
    let sma_3 = calculate_sma(&btc_prices, 3);
    let sma_5 = calculate_sma(&btc_prices, 5);

    println!("SMA(3): {:?}", sma_3);
    println!("SMA(5): {:?}", sma_5);
}
```

## Optimized Implementation

The basic implementation recalculates the sum for each window. This is inefficient! We can use a "rolling sum":

```rust
/// Optimized SMA using rolling sum
fn calculate_sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    // Calculate initial sum of the first window
    let mut window_sum: f64 = prices[..period].iter().sum();
    sma_values.push(window_sum / period as f64);

    // Slide through the array, updating the sum
    for i in period..prices.len() {
        // Add new price, remove old price
        window_sum += prices[i] - prices[i - period];
        sma_values.push(window_sum / period as f64);
    }

    sma_values
}

fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];

    let sma_naive = calculate_sma(&prices, 3);
    let sma_optimized = calculate_sma_optimized(&prices, 3);

    println!("Naive algorithm:     {:?}", sma_naive);
    println!("Optimized:           {:?}", sma_optimized);

    // Verify that results match
    assert_eq!(sma_naive.len(), sma_optimized.len());
    for (a, b) in sma_naive.iter().zip(sma_optimized.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
    println!("Results are identical!");
}

/// Basic implementation for comparison
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    for i in 0..=(prices.len() - period) {
        let window = &prices[i..i + period];
        let sum: f64 = window.iter().sum();
        sma_values.push(sum / period as f64);
    }

    sma_values
}
```

## SMA Calculator Structure

For real-world use, it's convenient to create a structure:

```rust
/// Simple Moving Average calculator
#[derive(Debug)]
struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    current_sum: f64,
}

impl SmaCalculator {
    /// Creates a new SMA calculator with the specified period
    fn new(period: usize) -> Self {
        SmaCalculator {
            period,
            prices: Vec::with_capacity(period),
            current_sum: 0.0,
        }
    }

    /// Adds a new price and returns the current SMA value (if available)
    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            // Remove the oldest price
            let old_price = self.prices.remove(0);
            self.current_sum -= old_price;
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    /// Returns the current SMA value without adding a new price
    fn current_sma(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    /// Returns the number of prices needed for SMA calculation
    fn prices_needed(&self) -> usize {
        if self.prices.len() >= self.period {
            0
        } else {
            self.period - self.prices.len()
        }
    }
}

fn main() {
    let mut sma = SmaCalculator::new(3);

    let incoming_prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];

    println!("Receiving real-time prices:");
    for price in incoming_prices {
        let sma_value = sma.add_price(price);
        match sma_value {
            Some(avg) => println!("Price: {:.2}, SMA(3): {:.2}", price, avg),
            None => println!(
                "Price: {:.2}, SMA(3): waiting for {} more values",
                price,
                sma.prices_needed()
            ),
        }
    }
}
```

## Trading Strategy: SMA Crossover

One of the classic strategies is trading on the crossover of two SMAs:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug)]
struct SmaCrossoverStrategy {
    fast_sma: SmaCalculator,
    slow_sma: SmaCalculator,
    previous_fast: Option<f64>,
    previous_slow: Option<f64>,
}

impl SmaCrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be smaller than slow period");
        SmaCrossoverStrategy {
            fast_sma: SmaCalculator::new(fast_period),
            slow_sma: SmaCalculator::new(slow_period),
            previous_fast: None,
            previous_slow: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        let current_fast = self.fast_sma.add_price(price);
        let current_slow = self.slow_sma.add_price(price);

        let signal = match (self.previous_fast, self.previous_slow, current_fast, current_slow) {
            (Some(prev_f), Some(prev_s), Some(curr_f), Some(curr_s)) => {
                // Golden cross: fast SMA crosses above slow SMA
                if prev_f <= prev_s && curr_f > curr_s {
                    Signal::Buy
                }
                // Death cross: fast SMA crosses below slow SMA
                else if prev_f >= prev_s && curr_f < curr_s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.previous_fast = current_fast;
        self.previous_slow = current_slow;

        signal
    }
}

/// Simple Moving Average calculator
#[derive(Debug)]
struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    current_sum: f64,
}

impl SmaCalculator {
    fn new(period: usize) -> Self {
        SmaCalculator {
            period,
            prices: Vec::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            let old_price = self.prices.remove(0);
            self.current_sum -= old_price;
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut strategy = SmaCrossoverStrategy::new(3, 5);

    // Simulate price stream
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,  // Data accumulation
        108.0, 110.0, 109.0,                 // Rising
        106.0, 103.0, 100.0, 98.0,           // Falling
        101.0, 104.0, 107.0,                 // Recovery
    ];

    println!("SMA(3)/SMA(5) Crossover Strategy:");
    println!("{:>8} {:>8}", "Price", "Signal");
    println!("{}", "-".repeat(20));

    for price in prices {
        let signal = strategy.update(price);
        let signal_str = match signal {
            Signal::Buy => "BUY",
            Signal::Sell => "SELL",
            Signal::Hold => "-",
        };
        println!("{:>8.2} {:>8}", price, signal_str);
    }
}
```

## Using VecDeque for Efficiency

`Vec::remove(0)` is an O(n) operation. For better performance, we use `VecDeque`:

```rust
use std::collections::VecDeque;

/// Efficient SMA calculator using VecDeque
#[derive(Debug)]
struct EfficientSma {
    period: usize,
    prices: VecDeque<f64>,
    current_sum: f64,
}

impl EfficientSma {
    fn new(period: usize) -> Self {
        EfficientSma {
            period,
            prices: VecDeque::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            // O(1) operation thanks to VecDeque
            if let Some(old_price) = self.prices.pop_front() {
                self.current_sum -= old_price;
            }
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    fn current_sma(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut sma = EfficientSma::new(5);

    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0,
    ];

    println!("Efficient SMA(5) calculation for BTC:");
    for price in prices {
        match sma.add_price(price) {
            Some(avg) => println!("Price: ${:.2} -> SMA: ${:.2}", price, avg),
            None => println!("Price: ${:.2} -> SMA: accumulating data...", price),
        }
    }
}
```

## Practical Example: Portfolio Analysis

```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
struct Asset {
    symbol: String,
    sma_short: EfficientSma,
    sma_long: EfficientSma,
    last_price: f64,
    position: f64, // quantity of asset
}

#[derive(Debug)]
struct Portfolio {
    assets: HashMap<String, Asset>,
    cash: f64,
}

#[derive(Debug)]
struct EfficientSma {
    period: usize,
    prices: VecDeque<f64>,
    current_sum: f64,
}

impl EfficientSma {
    fn new(period: usize) -> Self {
        EfficientSma {
            period,
            prices: VecDeque::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            if let Some(old_price) = self.prices.pop_front() {
                self.current_sum -= old_price;
            }
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio {
            assets: HashMap::new(),
            cash: initial_cash,
        }
    }

    fn add_asset(&mut self, symbol: &str, short_period: usize, long_period: usize) {
        self.assets.insert(symbol.to_string(), Asset {
            symbol: symbol.to_string(),
            sma_short: EfficientSma::new(short_period),
            sma_long: EfficientSma::new(long_period),
            last_price: 0.0,
            position: 0.0,
        });
    }

    fn update_price(&mut self, symbol: &str, price: f64) -> Option<String> {
        let asset = self.assets.get_mut(symbol)?;
        asset.last_price = price;

        let short_sma = asset.sma_short.add_price(price);
        let long_sma = asset.sma_long.add_price(price);

        match (short_sma, long_sma) {
            (Some(short), Some(long)) => {
                let trend = if short > long {
                    "UPTREND"
                } else if short < long {
                    "DOWNTREND"
                } else {
                    "SIDEWAYS"
                };

                Some(format!(
                    "{}: Price=${:.2}, SMA(short)=${:.2}, SMA(long)=${:.2}, Trend: {}",
                    symbol, price, short, long, trend
                ))
            }
            _ => Some(format!("{}: Price=${:.2}, accumulating data...", symbol, price)),
        }
    }

    fn get_portfolio_value(&self) -> f64 {
        let positions_value: f64 = self.assets.values()
            .map(|a| a.position * a.last_price)
            .sum();
        self.cash + positions_value
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100_000.0);

    // Add assets with different SMA periods
    portfolio.add_asset("BTC", 5, 20);
    portfolio.add_asset("ETH", 5, 20);

    // Simulate receiving prices
    let btc_prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42400.0, 42350.0, 42500.0, 42450.0,
        42600.0, 42700.0, 42650.0, 42800.0, 42750.0,
        42900.0, 43000.0, 42950.0, 43100.0, 43050.0,
        43200.0, 43300.0, 43250.0, 43400.0, 43350.0,
    ];

    let eth_prices = vec![
        2200.0, 2210.0, 2205.0, 2220.0, 2215.0,
        2230.0, 2240.0, 2235.0, 2250.0, 2245.0,
        2260.0, 2270.0, 2265.0, 2280.0, 2275.0,
        2290.0, 2300.0, 2295.0, 2310.0, 2305.0,
        2320.0, 2330.0, 2325.0, 2340.0, 2335.0,
    ];

    println!("=== Portfolio Analysis with SMA ===\n");

    for i in 0..btc_prices.len() {
        println!("--- Tick {} ---", i + 1);

        if let Some(analysis) = portfolio.update_price("BTC", btc_prices[i]) {
            println!("{}", analysis);
        }

        if let Some(analysis) = portfolio.update_price("ETH", eth_prices[i]) {
            println!("{}", analysis);
        }

        println!();
    }

    println!("Total portfolio value: ${:.2}", portfolio.get_portfolio_value());
}
```

## Comparing SMA with Other Indicators

| Indicator | Formula | Characteristics |
|-----------|---------|-----------------|
| SMA | (P1 + P2 + ... + Pn) / n | Simple, lagging |
| EMA | α * P + (1-α) * EMA_prev | More weight to recent data |
| WMA | Σ(Pi * Wi) / Σ(Wi) | Linear weights |

```rust
/// Comparing SMA and EMA
fn main() {
    let prices = vec![
        100.0, 102.0, 104.0, 103.0, 105.0,
        107.0, 106.0, 108.0, 110.0, 109.0,
    ];

    let period = 5;
    let alpha = 2.0 / (period as f64 + 1.0); // Smoothing factor for EMA

    // Calculate SMA
    let sma_values = calculate_sma(&prices, period);

    // Calculate EMA
    let ema_values = calculate_ema(&prices, alpha);

    println!("{:>8} {:>10} {:>10}", "Price", "SMA(5)", "EMA(5)");
    println!("{}", "-".repeat(30));

    for i in 0..prices.len() {
        let sma = if i >= period - 1 {
            format!("{:.2}", sma_values[i - period + 1])
        } else {
            "-".to_string()
        };

        let ema = format!("{:.2}", ema_values[i]);

        println!("{:>8.2} {:>10} {:>10}", prices[i], sma, ema);
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

fn calculate_ema(prices: &[f64], alpha: f64) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let mut ema_values = vec![prices[0]]; // First EMA value = first price

    for i in 1..prices.len() {
        let ema = alpha * prices[i] + (1.0 - alpha) * ema_values[i - 1];
        ema_values.push(ema);
    }

    ema_values
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SMA | Simple Moving Average — arithmetic mean of prices over a period |
| Period | Number of values for calculation (sliding window size) |
| Rolling Sum | Optimization: update sum instead of full recalculation |
| VecDeque | Efficient data structure for implementing sliding window |
| SMA Crossover | Trading strategy based on two SMAs with different periods |
| Golden Cross | Buy signal: fast SMA crosses above slow SMA |
| Death Cross | Sell signal: fast SMA crosses below slow SMA |

## Homework

1. **Multiple SMAs**: Create a `MultiSma` structure that simultaneously tracks several SMAs with different periods (e.g., 5, 10, 20, 50, 200). Implement a method that returns the current price position relative to all SMAs.

2. **Trend Detector**: Write a function that analyzes price history and determines trend strength based on the distance between SMA(20) and SMA(50). The greater the distance, the stronger the trend.

3. **Strategy Backtesting**: Implement a simple backtester for the SMA crossover strategy. Calculate:
   - Number of trades
   - Total profit/loss
   - Win rate percentage
   - Maximum drawdown

4. **Adaptive SMA**: Create a calculator that automatically adjusts the SMA period based on market volatility: use a shorter period during high volatility and a longer period during low volatility.

## Navigation

[← Previous day](../246-trading-algorithms-intro/en.md) | [Next day →](../248-ema-exponential-moving-average/en.md)
