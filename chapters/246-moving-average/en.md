# Day 246: Moving Average

## Trading Analogy

Imagine you're watching the Bitcoin price. Every second it changes: 42000, 42050, 41980, 42100... How do you understand the overall trend among all this noise? This is where the **Moving Average** (MA) comes to help.

A moving average is like looking at the market with "blurred eyes": instead of sharp spikes, you see a smooth line showing the direction of price movement. It's one of the most important indicators in technical analysis.

In real trading, moving averages are used to:
- Identify trends (price above MA = uptrend)
- Find support and resistance levels
- Generate trading signals (fast and slow MA crossover)
- Filter market noise

## What is a Moving Average?

A moving average is the average price value over a specific period, recalculated with each new value. It's called "moving" because the calculation window "moves" along the data.

For example, SMA-5 (simple moving average over 5 periods):
```
Prices: [100, 102, 101, 103, 105, 104, 106]
SMA-5 at position 4: (100 + 102 + 101 + 103 + 105) / 5 = 102.2
SMA-5 at position 5: (102 + 101 + 103 + 105 + 104) / 5 = 103.0
SMA-5 at position 6: (101 + 103 + 105 + 104 + 106) / 5 = 103.8
```

## Simple Moving Average (SMA)

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;
    let sma_values = calculate_sma(&prices, period);

    println!("=== SMA-{} for BTC ===", period);
    for (i, sma) in sma_values.iter().enumerate() {
        let price_index = i + period - 1;
        println!(
            "Period {}: Price = ${:.2}, SMA = ${:.2}",
            price_index + 1,
            prices[price_index],
            sma
        );
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut sma_values = Vec::new();

    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        let sma = sum / period as f64;
        sma_values.push(sma);
    }

    sma_values
}
```

## Exponential Moving Average (EMA)

EMA gives more weight to recent prices, so it reacts faster to changes:

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;
    let ema_values = calculate_ema(&prices, period);

    println!("=== EMA-{} for BTC ===", period);
    for (i, ema) in ema_values.iter().enumerate() {
        println!("Period {}: EMA = ${:.2}", i + period, ema);
    }
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut ema_values = Vec::new();

    // First EMA value = SMA
    let initial_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    ema_values.push(initial_sma);

    // Smoothing multiplier: 2 / (period + 1)
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate EMA for remaining values
    for i in period..prices.len() {
        let current_price = prices[i];
        let previous_ema = ema_values[ema_values.len() - 1];
        let current_ema = (current_price - previous_ema) * multiplier + previous_ema;
        ema_values.push(current_ema);
    }

    ema_values
}
```

## Comparing SMA and EMA

```rust
fn main() {
    // Sharp price jump to demonstrate the difference
    let prices = vec![
        100.0, 100.0, 100.0, 100.0, 100.0,  // Stable price
        110.0, 115.0, 120.0,                 // Sharp rise
    ];

    let period = 5;
    let sma_values = calculate_sma(&prices, period);
    let ema_values = calculate_ema(&prices, period);

    println!("=== Comparing SMA and EMA (period = {}) ===", period);
    println!("{:<10} {:<10} {:<10} {:<10}", "Index", "Price", "SMA", "EMA");
    println!("{}", "-".repeat(40));

    for i in 0..sma_values.len() {
        let price_idx = i + period - 1;
        println!(
            "{:<10} {:<10.2} {:<10.2} {:<10.2}",
            price_idx + 1,
            prices[price_idx],
            sma_values[i],
            ema_values[i]
        );
    }

    println!("\nConclusion: EMA reacts faster to price changes!");
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

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    let initial_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);
    let multiplier = 2.0 / (period as f64 + 1.0);
    for i in period..prices.len() {
        let prev = result[result.len() - 1];
        result.push((prices[i] - prev) * multiplier + prev);
    }
    result
}
```

## Structure for Trading Analysis

```rust
#[derive(Debug, Clone)]
struct PriceData {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct MovingAverageAnalyzer {
    prices: Vec<PriceData>,
    sma_period: usize,
    ema_period: usize,
}

impl MovingAverageAnalyzer {
    fn new(sma_period: usize, ema_period: usize) -> Self {
        MovingAverageAnalyzer {
            prices: Vec::new(),
            sma_period,
            ema_period,
        }
    }

    fn add_price(&mut self, data: PriceData) {
        self.prices.push(data);
    }

    fn get_closes(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.close).collect()
    }

    fn calculate_sma(&self) -> Vec<f64> {
        let closes = self.get_closes();
        if closes.len() < self.sma_period {
            return vec![];
        }

        let mut result = Vec::new();
        for i in 0..=(closes.len() - self.sma_period) {
            let sum: f64 = closes[i..i + self.sma_period].iter().sum();
            result.push(sum / self.sma_period as f64);
        }
        result
    }

    fn calculate_ema(&self) -> Vec<f64> {
        let closes = self.get_closes();
        if closes.len() < self.ema_period {
            return vec![];
        }

        let mut result = Vec::new();
        let initial: f64 = closes[0..self.ema_period].iter().sum::<f64>()
            / self.ema_period as f64;
        result.push(initial);

        let mult = 2.0 / (self.ema_period as f64 + 1.0);
        for i in self.ema_period..closes.len() {
            let prev = result[result.len() - 1];
            result.push((closes[i] - prev) * mult + prev);
        }
        result
    }

    fn get_trend(&self) -> Option<&'static str> {
        let closes = self.get_closes();
        let sma = self.calculate_sma();

        if closes.is_empty() || sma.is_empty() {
            return None;
        }

        let last_price = closes[closes.len() - 1];
        let last_sma = sma[sma.len() - 1];

        if last_price > last_sma * 1.01 {
            Some("Uptrend (Bullish)")
        } else if last_price < last_sma * 0.99 {
            Some("Downtrend (Bearish)")
        } else {
            Some("Sideways")
        }
    }
}

fn main() {
    let mut analyzer = MovingAverageAnalyzer::new(5, 5);

    // Simulate incoming data
    let test_data = vec![
        PriceData { timestamp: 1, open: 42000.0, high: 42100.0, low: 41900.0, close: 42050.0, volume: 100.0 },
        PriceData { timestamp: 2, open: 42050.0, high: 42200.0, low: 42000.0, close: 42150.0, volume: 120.0 },
        PriceData { timestamp: 3, open: 42150.0, high: 42300.0, low: 42100.0, close: 42250.0, volume: 110.0 },
        PriceData { timestamp: 4, open: 42250.0, high: 42400.0, low: 42200.0, close: 42350.0, volume: 130.0 },
        PriceData { timestamp: 5, open: 42350.0, high: 42500.0, low: 42300.0, close: 42450.0, volume: 140.0 },
        PriceData { timestamp: 6, open: 42450.0, high: 42600.0, low: 42400.0, close: 42550.0, volume: 150.0 },
        PriceData { timestamp: 7, open: 42550.0, high: 42700.0, low: 42500.0, close: 42650.0, volume: 160.0 },
    ];

    for data in test_data {
        analyzer.add_price(data);
    }

    println!("=== Moving Average Analysis ===");
    println!("SMA-5: {:?}", analyzer.calculate_sma());
    println!("EMA-5: {:?}", analyzer.calculate_ema());
    println!("Trend: {:?}", analyzer.get_trend());
}
```

## Moving Average Crossover (Golden Cross / Death Cross)

```rust
#[derive(Debug, PartialEq)]
enum CrossoverSignal {
    GoldenCross,  // Fast MA crosses slow MA from below (buy signal)
    DeathCross,   // Fast MA crosses slow MA from above (sell signal)
    NoSignal,
}

fn detect_crossover(fast_ma: &[f64], slow_ma: &[f64]) -> CrossoverSignal {
    if fast_ma.len() < 2 || slow_ma.len() < 2 {
        return CrossoverSignal::NoSignal;
    }

    let len = fast_ma.len().min(slow_ma.len());
    let fast_prev = fast_ma[len - 2];
    let fast_curr = fast_ma[len - 1];
    let slow_prev = slow_ma[len - 2];
    let slow_curr = slow_ma[len - 1];

    // Golden Cross: fast was below, now above
    if fast_prev <= slow_prev && fast_curr > slow_curr {
        return CrossoverSignal::GoldenCross;
    }

    // Death Cross: fast was above, now below
    if fast_prev >= slow_prev && fast_curr < slow_curr {
        return CrossoverSignal::DeathCross;
    }

    CrossoverSignal::NoSignal
}

fn main() {
    // Data simulation with crossover
    let prices_bullish = vec![
        100.0, 101.0, 99.0, 100.0, 102.0,   // Start
        104.0, 106.0, 108.0, 110.0, 112.0,  // Rise (Golden Cross)
    ];

    let prices_bearish = vec![
        110.0, 112.0, 111.0, 109.0, 108.0,  // Start
        106.0, 104.0, 102.0, 100.0, 98.0,   // Fall (Death Cross)
    ];

    let fast_period = 3;
    let slow_period = 5;

    // Bullish scenario analysis
    let fast_ma_bull = calculate_sma(&prices_bullish, fast_period);
    let slow_ma_bull = calculate_sma(&prices_bullish, slow_period);
    let signal_bull = detect_crossover(&fast_ma_bull, &slow_ma_bull);

    println!("=== Bullish Scenario ===");
    println!("Fast MA (SMA-{}): {:?}", fast_period, fast_ma_bull);
    println!("Slow MA (SMA-{}): {:?}", slow_period, slow_ma_bull);
    println!("Signal: {:?}", signal_bull);

    println!();

    // Bearish scenario analysis
    let fast_ma_bear = calculate_sma(&prices_bearish, fast_period);
    let slow_ma_bear = calculate_sma(&prices_bearish, slow_period);
    let signal_bear = detect_crossover(&fast_ma_bear, &slow_ma_bear);

    println!("=== Bearish Scenario ===");
    println!("Fast MA (SMA-{}): {:?}", fast_period, fast_ma_bear);
    println!("Slow MA (SMA-{}): {:?}", slow_period, slow_ma_bear);
    println!("Signal: {:?}", signal_bear);
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
```

## Weighted Moving Average (WMA)

```rust
fn calculate_wma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::new();

    // Sum of weights: 1 + 2 + 3 + ... + period = period * (period + 1) / 2
    let weight_sum: f64 = (period * (period + 1) / 2) as f64;

    for i in 0..=(prices.len() - period) {
        let mut weighted_sum = 0.0;
        for j in 0..period {
            let weight = (j + 1) as f64;
            weighted_sum += prices[i + j] * weight;
        }
        result.push(weighted_sum / weight_sum);
    }

    result
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;

    let sma = calculate_sma(&prices, period);
    let wma = calculate_wma(&prices, period);
    let ema = calculate_ema(&prices, period);

    println!("=== Comparing Moving Average Types (period = {}) ===", period);
    println!("{:<10} {:<12} {:<12} {:<12}", "Index", "SMA", "WMA", "EMA");
    println!("{}", "-".repeat(46));

    for i in 0..sma.len() {
        println!(
            "{:<10} {:<12.2} {:<12.2} {:<12.2}",
            i + period,
            sma[i],
            wma[i],
            ema[i]
        );
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

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    let initial: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(initial);
    let mult = 2.0 / (period as f64 + 1.0);
    for i in period..prices.len() {
        let prev = result[result.len() - 1];
        result.push((prices[i] - prev) * mult + prev);
    }
    result
}
```

## Practical Example: Trading Strategy with MA

```rust
#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Position {
    Long,
    Short,
    None,
}

struct MACrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
    current_position: Position,
    trades: Vec<Trade>,
}

impl MACrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        MACrossoverStrategy {
            fast_period,
            slow_period,
            current_position: Position::None,
            trades: Vec::new(),
        }
    }

    fn backtest(&mut self, prices: &[f64]) {
        let fast_ma = calculate_sma(prices, self.fast_period);
        let slow_ma = calculate_sma(prices, self.slow_period);

        if fast_ma.len() < 2 || slow_ma.len() < 2 {
            println!("Insufficient data for backtest");
            return;
        }

        // Synchronize indices
        let offset = self.slow_period - 1;

        for i in 1..slow_ma.len() {
            let fast_idx = i + (self.slow_period - self.fast_period);
            if fast_idx >= fast_ma.len() {
                break;
            }

            let fast_prev = fast_ma[fast_idx - 1];
            let fast_curr = fast_ma[fast_idx];
            let slow_prev = slow_ma[i - 1];
            let slow_curr = slow_ma[i];

            let price = prices[offset + i];

            // Golden Cross - open Long
            if fast_prev <= slow_prev && fast_curr > slow_curr {
                if self.current_position == Position::Short {
                    self.close_position(price);
                }
                if self.current_position == Position::None {
                    self.open_position(price, Position::Long);
                }
            }

            // Death Cross - open Short
            if fast_prev >= slow_prev && fast_curr < slow_curr {
                if self.current_position == Position::Long {
                    self.close_position(price);
                }
                if self.current_position == Position::None {
                    self.open_position(price, Position::Short);
                }
            }
        }

        // Close open position at the last price
        if self.current_position != Position::None {
            let last_price = prices[prices.len() - 1];
            self.close_position(last_price);
        }
    }

    fn open_position(&mut self, price: f64, position: Position) {
        self.current_position = position;
        self.trades.push(Trade {
            entry_price: price,
            exit_price: None,
            position,
        });
        println!("Opened {:?} position at {:.2}", position, price);
    }

    fn close_position(&mut self, price: f64) {
        if let Some(trade) = self.trades.last_mut() {
            trade.exit_price = Some(price);
            let pnl = match trade.position {
                Position::Long => price - trade.entry_price,
                Position::Short => trade.entry_price - price,
                Position::None => 0.0,
            };
            println!(
                "Closed {:?} position at {:.2}, P&L: {:.2}",
                trade.position, price, pnl
            );
        }
        self.current_position = Position::None;
    }

    fn calculate_total_pnl(&self) -> f64 {
        self.trades
            .iter()
            .filter_map(|t| {
                t.exit_price.map(|exit| match t.position {
                    Position::Long => exit - t.entry_price,
                    Position::Short => t.entry_price - exit,
                    Position::None => 0.0,
                })
            })
            .sum()
    }
}

fn main() {
    // Price data simulation with trends
    let prices = vec![
        100.0, 101.0, 102.0, 101.5, 103.0,  // Start
        105.0, 107.0, 109.0, 111.0, 113.0,  // Uptrend
        112.0, 110.0, 108.0, 106.0, 104.0,  // Downtrend
        105.0, 107.0, 108.0, 110.0, 112.0,  // New uptrend
    ];

    let mut strategy = MACrossoverStrategy::new(3, 5);

    println!("=== MA Crossover Strategy Backtest ===");
    println!("Fast MA: SMA-3, Slow MA: SMA-5\n");

    strategy.backtest(&prices);

    println!("\n=== Summary ===");
    println!("Total trades: {}", strategy.trades.len());
    println!("Total P&L: {:.2}", strategy.calculate_total_pnl());
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
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SMA | Simple Moving Average — arithmetic mean over a period |
| EMA | Exponential Moving Average — more weight to recent prices |
| WMA | Weighted Moving Average — linearly increasing weights |
| Golden Cross | Fast MA crosses slow MA from below — buy signal |
| Death Cross | Fast MA crosses slow MA from above — sell signal |
| MA Period | Number of values for calculation — affects sensitivity |

## Homework

1. **SMA Optimization**: Implement an efficient SMA calculation that doesn't recalculate the sum at each step, but uses a sliding window (adds new value and subtracts old one).

2. **Multiple MAs**: Create a `MultiMAAnalyzer` structure that simultaneously calculates SMA-10, SMA-20, SMA-50, SMA-200 and determines the overall trend by their relative positioning.

3. **Adaptive MA**: Implement KAMA (Kaufman's Adaptive Moving Average), which automatically adjusts its sensitivity based on market volatility.

4. **Backtest with Fees**: Modify the trading strategy from the example by adding:
   - Trading commission (0.1%)
   - Stop-loss (2% from entry price)
   - Take-profit (5% from entry price)
   - Win rate calculation (percentage of profitable trades)

## Navigation

[← Previous day](../245-calculating-candles-from-ticks/en.md) | [Next day →](../247-sma-simple-moving-average/en.md)
