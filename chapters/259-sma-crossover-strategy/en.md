# Day 259: Strategy: SMA Crossover

## Trading Analogy

Imagine you're watching a busy market. Prices constantly fluctuate — up, down, sideways. How do you know when to buy or sell? One classic approach is to look at the "average mood" of the market over different time periods.

Think of it like reading weather patterns. The short-term average (like a 10-day moving average) shows you recent conditions — it's like checking today's weather. The long-term average (like a 50-day moving average) shows the overall trend — it's like knowing the season. When today's weather is warmer than the seasonal average, spring might be coming. When the short-term average crosses above the long-term average, a bullish trend might be forming.

This is the essence of the **SMA Crossover Strategy**:
- **Golden Cross**: Short-term SMA crosses ABOVE long-term SMA → Buy signal (bullish)
- **Death Cross**: Short-term SMA crosses BELOW long-term SMA → Sell signal (bearish)

In real trading, this strategy is used by:
- Swing traders looking for trend changes
- Portfolio managers for asset allocation decisions
- Algorithmic trading systems as a baseline strategy
- Risk managers to confirm trend direction

## What is a Simple Moving Average (SMA)?

The Simple Moving Average is the arithmetic mean of prices over a specified period:

```
SMA = (P1 + P2 + P3 + ... + Pn) / n
```

Where:
- `P1, P2, ... Pn` are the prices (usually closing prices)
- `n` is the period (number of data points)

### Key Characteristics

| Property | Description |
|----------|-------------|
| Lagging indicator | Reacts to past price movements |
| Smoothing effect | Filters out noise from price data |
| Period sensitivity | Shorter periods = more sensitive, more signals |
| Universal | Works on any timeframe (minutes, hours, days) |

## Implementing SMA in Rust

Let's build our SMA crossover strategy step by step.

### Step 1: Basic SMA Calculation

```rust
/// Calculate Simple Moving Average from a slice of prices
fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let sum: f64 = prices.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

fn main() {
    let prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43000.0,
        43200.0, 43100.0, 43500.0, 43800.0, 44000.0,
    ];

    if let Some(sma_5) = calculate_sma(&prices, 5) {
        println!("5-period SMA: ${:.2}", sma_5);
    }

    if let Some(sma_10) = calculate_sma(&prices, 10) {
        println!("10-period SMA: ${:.2}", sma_10);
    }
}
```

### Step 2: SMA Series Calculator

For crossover detection, we need a series of SMA values:

```rust
/// Calculate a series of SMA values
fn calculate_sma_series(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    (0..prices.len())
        .map(|i| {
            if i + 1 >= period {
                let slice = &prices[i + 1 - period..=i];
                Some(slice.iter().sum::<f64>() / period as f64)
            } else {
                None // Not enough data points yet
            }
        })
        .collect()
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,
        104.0, 106.0, 108.0, 107.0, 110.0,
    ];

    let sma_3 = calculate_sma_series(&prices, 3);

    println!("Price\t\tSMA(3)");
    println!("-----\t\t------");
    for (i, price) in prices.iter().enumerate() {
        match sma_3[i] {
            Some(sma) => println!("{:.2}\t\t{:.2}", price, sma),
            None => println!("{:.2}\t\t-", price),
        }
    }
}
```

### Step 3: Crossover Signal Detection

```rust
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,        // Golden Cross
    Sell,       // Death Cross
    Hold,       // No crossover
}

#[derive(Debug, Clone)]
struct CrossoverSignal {
    index: usize,
    signal: Signal,
    short_sma: f64,
    long_sma: f64,
    price: f64,
}

/// Detect SMA crossover signals
fn detect_crossovers(
    prices: &[f64],
    short_period: usize,
    long_period: usize,
) -> Vec<CrossoverSignal> {
    let short_sma = calculate_sma_series(prices, short_period);
    let long_sma = calculate_sma_series(prices, long_period);

    let mut signals = Vec::new();

    for i in 1..prices.len() {
        // Need both SMAs for current and previous period
        if let (Some(short_curr), Some(long_curr), Some(short_prev), Some(long_prev)) = (
            short_sma[i],
            long_sma[i],
            short_sma[i - 1],
            long_sma[i - 1],
        ) {
            let signal = if short_prev <= long_prev && short_curr > long_curr {
                Signal::Buy // Golden Cross
            } else if short_prev >= long_prev && short_curr < long_curr {
                Signal::Sell // Death Cross
            } else {
                Signal::Hold
            };

            if signal != Signal::Hold {
                signals.push(CrossoverSignal {
                    index: i,
                    signal,
                    short_sma: short_curr,
                    long_sma: long_curr,
                    price: prices[i],
                });
            }
        }
    }

    signals
}

fn main() {
    // Simulated BTC prices with a trend change
    let prices = vec![
        40000.0, 40500.0, 41000.0, 40800.0, 41500.0,
        42000.0, 42500.0, 43000.0, 43500.0, 44000.0,
        44500.0, 44200.0, 43800.0, 43500.0, 43000.0,
        42500.0, 42000.0, 41500.0, 41000.0, 40500.0,
    ];

    let signals = detect_crossovers(&prices, 3, 7);

    println!("=== SMA Crossover Signals ===\n");
    for signal in &signals {
        let signal_type = match signal.signal {
            Signal::Buy => "BUY (Golden Cross)",
            Signal::Sell => "SELL (Death Cross)",
            Signal::Hold => "HOLD",
        };

        println!(
            "Day {}: {} at ${:.2}",
            signal.index, signal_type, signal.price
        );
        println!(
            "  Short SMA: ${:.2}, Long SMA: ${:.2}\n",
            signal.short_sma, signal.long_sma
        );
    }
}
```

## Complete Trading Strategy Implementation

Now let's build a complete backtesting system for the SMA crossover strategy:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    entry_index: usize,
    exit_index: Option<usize>,
    position_type: PositionType,
}

#[derive(Debug, Clone, PartialEq)]
enum PositionType {
    Long,
    Short,
}

#[derive(Debug)]
struct BacktestResult {
    trades: Vec<Trade>,
    total_return: f64,
    win_rate: f64,
    max_drawdown: f64,
    total_trades: usize,
}

struct SmaCrossoverStrategy {
    short_period: usize,
    long_period: usize,
    prices: Vec<f64>,
}

impl SmaCrossoverStrategy {
    fn new(short_period: usize, long_period: usize) -> Self {
        SmaCrossoverStrategy {
            short_period,
            long_period,
            prices: Vec::new(),
        }
    }

    fn load_prices(&mut self, prices: Vec<f64>) {
        self.prices = prices;
    }

    fn calculate_sma_at(&self, end_index: usize, period: usize) -> Option<f64> {
        if end_index + 1 < period {
            return None;
        }

        let start = end_index + 1 - period;
        let slice = &self.prices[start..=end_index];
        Some(slice.iter().sum::<f64>() / period as f64)
    }

    fn backtest(&self) -> BacktestResult {
        let mut trades: Vec<Trade> = Vec::new();
        let mut current_position: Option<Trade> = None;
        let mut equity_curve: Vec<f64> = Vec::new();
        let initial_capital = 10000.0;
        let mut capital = initial_capital;

        for i in self.long_period..self.prices.len() {
            let short_curr = self.calculate_sma_at(i, self.short_period).unwrap();
            let long_curr = self.calculate_sma_at(i, self.long_period).unwrap();
            let short_prev = self.calculate_sma_at(i - 1, self.short_period).unwrap();
            let long_prev = self.calculate_sma_at(i - 1, self.long_period).unwrap();

            // Golden Cross - Buy Signal
            if short_prev <= long_prev && short_curr > long_curr {
                // Close any existing short position
                if let Some(mut pos) = current_position.take() {
                    if pos.position_type == PositionType::Short {
                        pos.exit_price = Some(self.prices[i]);
                        pos.exit_index = Some(i);
                        let pnl = pos.entry_price - self.prices[i];
                        capital += pnl / pos.entry_price * capital;
                        trades.push(pos);
                    }
                }

                // Open long position
                current_position = Some(Trade {
                    entry_price: self.prices[i],
                    exit_price: None,
                    entry_index: i,
                    exit_index: None,
                    position_type: PositionType::Long,
                });
            }

            // Death Cross - Sell Signal
            if short_prev >= long_prev && short_curr < long_curr {
                // Close any existing long position
                if let Some(mut pos) = current_position.take() {
                    if pos.position_type == PositionType::Long {
                        pos.exit_price = Some(self.prices[i]);
                        pos.exit_index = Some(i);
                        let pnl = self.prices[i] - pos.entry_price;
                        capital += pnl / pos.entry_price * capital;
                        trades.push(pos);
                    }
                }

                // Open short position
                current_position = Some(Trade {
                    entry_price: self.prices[i],
                    exit_price: None,
                    entry_index: i,
                    exit_index: None,
                    position_type: PositionType::Short,
                });
            }

            equity_curve.push(capital);
        }

        // Close any remaining position at the end
        if let Some(mut pos) = current_position.take() {
            let final_price = *self.prices.last().unwrap();
            pos.exit_price = Some(final_price);
            pos.exit_index = Some(self.prices.len() - 1);

            let pnl = match pos.position_type {
                PositionType::Long => final_price - pos.entry_price,
                PositionType::Short => pos.entry_price - final_price,
            };
            capital += pnl / pos.entry_price * capital;
            trades.push(pos);
        }

        // Calculate statistics
        let winning_trades = trades.iter().filter(|t| {
            if let Some(exit) = t.exit_price {
                match t.position_type {
                    PositionType::Long => exit > t.entry_price,
                    PositionType::Short => exit < t.entry_price,
                }
            } else {
                false
            }
        }).count();

        let total_trades = trades.len();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        // Calculate max drawdown
        let mut peak = initial_capital;
        let mut max_drawdown = 0.0;
        for equity in &equity_curve {
            if *equity > peak {
                peak = *equity;
            }
            let drawdown = (peak - equity) / peak * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let total_return = (capital - initial_capital) / initial_capital * 100.0;

        BacktestResult {
            trades,
            total_return,
            win_rate,
            max_drawdown,
            total_trades,
        }
    }
}

fn main() {
    // Generate sample price data with trends
    let mut prices: Vec<f64> = Vec::new();
    let mut price = 100.0;

    // Uptrend
    for _ in 0..30 {
        price += (rand_simple() - 0.3) * 2.0;
        prices.push(price);
    }

    // Downtrend
    for _ in 0..30 {
        price += (rand_simple() - 0.7) * 2.0;
        prices.push(price);
    }

    // Another uptrend
    for _ in 0..30 {
        price += (rand_simple() - 0.3) * 2.0;
        prices.push(price);
    }

    let mut strategy = SmaCrossoverStrategy::new(5, 20);
    strategy.load_prices(prices);

    let result = strategy.backtest();

    println!("=== SMA Crossover Backtest Results ===\n");
    println!("Short SMA Period: 5");
    println!("Long SMA Period: 20");
    println!("-----------------------------------");
    println!("Total Trades: {}", result.total_trades);
    println!("Win Rate: {:.2}%", result.win_rate);
    println!("Total Return: {:.2}%", result.total_return);
    println!("Max Drawdown: {:.2}%", result.max_drawdown);

    println!("\n=== Trade History ===\n");
    for (i, trade) in result.trades.iter().enumerate() {
        let direction = match trade.position_type {
            PositionType::Long => "LONG",
            PositionType::Short => "SHORT",
        };

        if let Some(exit_price) = trade.exit_price {
            let pnl_pct = match trade.position_type {
                PositionType::Long => (exit_price - trade.entry_price) / trade.entry_price * 100.0,
                PositionType::Short => (trade.entry_price - exit_price) / trade.entry_price * 100.0,
            };

            println!(
                "Trade {}: {} | Entry: ${:.2} | Exit: ${:.2} | PnL: {:.2}%",
                i + 1, direction, trade.entry_price, exit_price, pnl_pct
            );
        }
    }
}

// Simple pseudo-random number generator for demonstration
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    static mut SEED: u64 = 0;
    unsafe {
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((SEED >> 16) & 0x7fff) as f64 / 32768.0
    }
}
```

## Real-Time SMA Tracker

For live trading, you need an efficient rolling SMA calculator:

```rust
use std::collections::VecDeque;

struct RollingSma {
    period: usize,
    prices: VecDeque<f64>,
    sum: f64,
}

impl RollingSma {
    fn new(period: usize) -> Self {
        RollingSma {
            period,
            prices: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.sum += price;

        if self.prices.len() > self.period {
            if let Some(old_price) = self.prices.pop_front() {
                self.sum -= old_price;
            }
        }

        if self.prices.len() >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        if self.prices.len() >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

struct CrossoverTracker {
    short_sma: RollingSma,
    long_sma: RollingSma,
    prev_short: Option<f64>,
    prev_long: Option<f64>,
}

impl CrossoverTracker {
    fn new(short_period: usize, long_period: usize) -> Self {
        CrossoverTracker {
            short_sma: RollingSma::new(short_period),
            long_sma: RollingSma::new(long_period),
            prev_short: None,
            prev_long: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        let short_curr = self.short_sma.update(price);
        let long_curr = self.long_sma.update(price);

        let signal = match (short_curr, long_curr, self.prev_short, self.prev_long) {
            (Some(sc), Some(lc), Some(sp), Some(lp)) => {
                if sp <= lp && sc > lc {
                    Signal::Buy
                } else if sp >= lp && sc < lc {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.prev_short = short_curr;
        self.prev_long = long_curr;

        signal
    }
}

#[derive(Debug, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

fn main() {
    let mut tracker = CrossoverTracker::new(3, 7);

    // Simulate incoming price ticks
    let price_stream = vec![
        100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 105.0,
        106.0, 107.0, 108.0, 107.5, 106.0, 105.0, 104.0,
        103.0, 102.0, 101.0, 100.0, 99.0, 98.0,
    ];

    println!("=== Real-Time Crossover Tracking ===\n");
    println!("Price\t\tSignal");
    println!("-----\t\t------");

    for price in price_stream {
        let signal = tracker.update(price);

        let signal_str = match signal {
            Signal::Buy => ">>> BUY <<<",
            Signal::Sell => "<<< SELL >>>",
            Signal::Hold => "-",
        };

        println!("{:.2}\t\t{}", price, signal_str);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SMA | Average price over N periods, smooths price action |
| Golden Cross | Short SMA crosses above long SMA — bullish signal |
| Death Cross | Short SMA crosses below long SMA — bearish signal |
| Lagging indicator | SMA reacts to past data, signals come with delay |
| Period selection | Shorter = more sensitive, longer = more reliable |
| Rolling calculation | Efficient O(1) update using sliding window |

## Exercises

1. **Basic SMA**: Write a function that takes a vector of prices and a period, returning the SMA. Test it with BTC daily prices.

2. **Multiple Timeframes**: Create a function that calculates SMAs for multiple periods (5, 10, 20, 50) simultaneously and displays them in a table format.

3. **Signal Counter**: Build a program that reads price data and counts how many Golden Crosses and Death Crosses occurred in the dataset.

4. **Trend Strength**: Implement a function that measures the "strength" of a crossover by calculating how fast the short SMA is diverging from the long SMA after a signal.

## Homework

1. **Enhanced Strategy**: Modify the SMA crossover strategy to include:
   - A minimum distance between SMAs before generating a signal (filter for false crossovers)
   - Volume confirmation (only take signals on above-average volume days)
   - Stop-loss and take-profit levels

2. **Multi-Asset Scanner**: Create a program that:
   - Takes price data for multiple assets (BTC, ETH, SOL, etc.)
   - Monitors SMA crossovers on all of them
   - Reports which assets are currently showing buy or sell signals

3. **Parameter Optimization**: Build a backtester that:
   - Tests different combinations of short/long SMA periods (e.g., 5/20, 10/30, 15/50)
   - Reports which combination produces the best results
   - Handles edge cases (not enough data, no trades, etc.)

4. **Live Dashboard**: Create a terminal-based dashboard that:
   - Continuously receives price updates
   - Displays current short and long SMA values
   - Shows the current signal status (Long, Short, or Neutral)
   - Tracks the P&L of following the signals

## Navigation

[← Previous day](../258-moving-averages-intro/en.md) | [Next day →](../260-ema-exponential-moving-average/en.md)
