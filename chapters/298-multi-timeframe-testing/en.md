# Day 298: Multi-Timeframe Testing

## Trading Analogy

Imagine you have a trading strategy that works great on 1-hour candles. You backtest it and see consistent profits! But when you deploy it live, the strategy doesn't work. Why?

The problem is you ignored the **bigger market picture**. Your strategy might have been opening long positions on the 1-hour chart while the daily chart showed a downtrend. It's like looking at a tree and missing the forest.

**Multi-timeframe analysis** is like looking at a city map from different heights:
- **Monthly chart** (10km altitude) — global market trend
- **Weekly chart** (1km altitude) — medium-term trend
- **Daily chart** (100m altitude) — short-term trend
- **Hourly chart** (10m altitude) — tactical entry/exit

The best traders analyze all levels simultaneously: determine direction on higher timeframes and find entry points on lower timeframes.

## What is Multi-Timeframe Testing?

Multi-timeframe testing is backtesting a trading strategy considering multiple time periods simultaneously. This allows you to:

1. **Confirm trend direction** — higher timeframes show the main direction
2. **Improve entry precision** — lower timeframes give more accurate signals
3. **Reduce false signals** — filtering signals through multiple timeframes
4. **Manage risk better** — better understanding of volatility at different levels

### Timeframe Concept

```
Month    [====================] Long-term trend
Week       [====][====][====]  Medium-term waves
Day          [][][][][][][][]  Short-term movements
Hour         ................ Micro-fluctuations
```

## Basic Structure for Multi-Timeframe Data

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeFrame {
    M1,   // 1 minute
    M5,   // 5 minutes
    M15,  // 15 minutes
    H1,   // 1 hour
    H4,   // 4 hours
    D1,   // 1 day
    W1,   // 1 week
}

impl TimeFrame {
    fn to_seconds(&self) -> i64 {
        match self {
            TimeFrame::M1 => 60,
            TimeFrame::M5 => 300,
            TimeFrame::M15 => 900,
            TimeFrame::H1 => 3600,
            TimeFrame::H4 => 14400,
            TimeFrame::D1 => 86400,
            TimeFrame::W1 => 604800,
        }
    }

    fn name(&self) -> &str {
        match self {
            TimeFrame::M1 => "1m",
            TimeFrame::M5 => "5m",
            TimeFrame::M15 => "15m",
            TimeFrame::H1 => "1h",
            TimeFrame::H4 => "4h",
            TimeFrame::D1 => "1d",
            TimeFrame::W1 => "1w",
        }
    }
}

struct MultiTimeFrameData {
    symbol: String,
    data: HashMap<TimeFrame, Vec<Candle>>,
}

impl MultiTimeFrameData {
    fn new(symbol: &str) -> Self {
        MultiTimeFrameData {
            symbol: symbol.to_string(),
            data: HashMap::new(),
        }
    }

    fn add_candles(&mut self, timeframe: TimeFrame, candles: Vec<Candle>) {
        self.data.insert(timeframe, candles);
    }

    fn get_candles(&self, timeframe: TimeFrame) -> Option<&Vec<Candle>> {
        self.data.get(&timeframe)
    }

    // Aggregate lower timeframe to higher timeframe
    fn aggregate_timeframe(
        &self,
        from: TimeFrame,
        to: TimeFrame,
    ) -> Option<Vec<Candle>> {
        let base_candles = self.data.get(&from)?;
        let ratio = to.to_seconds() / from.to_seconds();

        if ratio <= 1 {
            return None; // Cannot aggregate to smaller timeframe
        }

        let mut aggregated = Vec::new();
        let mut i = 0;

        while i < base_candles.len() {
            let chunk_size = ratio.min((base_candles.len() - i) as i64) as usize;
            let chunk = &base_candles[i..i + chunk_size];

            if chunk.is_empty() {
                break;
            }

            let open = chunk[0].open;
            let close = chunk[chunk.len() - 1].close;
            let high = chunk.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let low = chunk.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            let volume = chunk.iter().map(|c| c.volume).sum();

            aggregated.push(Candle {
                timestamp: chunk[0].timestamp,
                open,
                high,
                low,
                close,
                volume,
            });

            i += chunk_size;
        }

        Some(aggregated)
    }
}
```

## Indicators for Multi-Timeframe Analysis

```rust
fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in 0..prices.len() {
        if i < period - 1 {
            sma.push(0.0);
        } else {
            let sum: f64 = prices[i - period + 1..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }

    sma
}

fn exponential_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return Vec::new();
    }

    let mut ema = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // First EMA value = SMA
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut current_ema = initial_sum / period as f64;

    for _ in 0..period - 1 {
        ema.push(0.0);
    }
    ema.push(current_ema);

    for i in period..prices.len() {
        current_ema = (prices[i] - current_ema) * multiplier + current_ema;
        ema.push(current_ema);
    }

    ema
}

#[derive(Debug, Clone, Copy)]
enum Trend {
    Bullish,   // Uptrend
    Bearish,   // Downtrend
    Sideways,  // Ranging
}

fn detect_trend(candles: &[Candle], sma_period: usize) -> Trend {
    if candles.len() < sma_period {
        return Trend::Sideways;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let sma = simple_moving_average(&closes, sma_period);

    let last_idx = candles.len() - 1;
    let last_close = candles[last_idx].close;
    let last_sma = sma[last_idx];

    if last_close > last_sma * 1.02 {
        Trend::Bullish
    } else if last_close < last_sma * 0.98 {
        Trend::Bearish
    } else {
        Trend::Sideways
    }
}
```

## Multi-Timeframe Strategy

```rust
#[derive(Debug)]
struct MultiTimeFrameStrategy {
    symbol: String,
    higher_tf: TimeFrame,  // Higher timeframe for trend
    lower_tf: TimeFrame,   // Lower timeframe for entry
    trend_period: usize,
    signal_period: usize,
}

impl MultiTimeFrameStrategy {
    fn new(
        symbol: &str,
        higher_tf: TimeFrame,
        lower_tf: TimeFrame,
        trend_period: usize,
        signal_period: usize,
    ) -> Self {
        MultiTimeFrameStrategy {
            symbol: symbol.to_string(),
            higher_tf,
            lower_tf,
            trend_period,
            signal_period,
        }
    }

    fn analyze(&self, data: &MultiTimeFrameData) -> Option<Signal> {
        // Get data from both timeframes
        let higher_candles = data.get_candles(self.higher_tf)?;
        let lower_candles = data.get_candles(self.lower_tf)?;

        // Determine trend on higher timeframe
        let higher_trend = detect_trend(higher_candles, self.trend_period);

        // Find entry signal on lower timeframe
        let lower_signal = self.find_entry_signal(lower_candles);

        // Combine signals
        match (higher_trend, lower_signal) {
            (Trend::Bullish, Some(Signal::Buy)) => Some(Signal::Buy),
            (Trend::Bearish, Some(Signal::Sell)) => Some(Signal::Sell),
            _ => None, // Ignore signals against the trend
        }
    }

    fn find_entry_signal(&self, candles: &[Candle]) -> Option<Signal> {
        if candles.len() < self.signal_period + 2 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let fast_ema = exponential_moving_average(&closes, self.signal_period);
        let slow_ema = exponential_moving_average(&closes, self.signal_period * 2);

        let len = candles.len();

        // EMA crossover up = buy signal
        if fast_ema[len - 1] > slow_ema[len - 1]
            && fast_ema[len - 2] <= slow_ema[len - 2]
        {
            return Some(Signal::Buy);
        }

        // EMA crossover down = sell signal
        if fast_ema[len - 1] < slow_ema[len - 1]
            && fast_ema[len - 2] >= slow_ema[len - 2]
        {
            return Some(Signal::Sell);
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
enum Signal {
    Buy,
    Sell,
}
```

## Backtesting with Multi-Timeframe Analysis

```rust
#[derive(Debug)]
struct BacktestResult {
    total_trades: usize,
    winning_trades: usize,
    total_profit: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn backtest_multi_timeframe(
    data: &MultiTimeFrameData,
    strategy: &MultiTimeFrameStrategy,
) -> BacktestResult {
    let mut equity = 10_000.0;
    let mut peak_equity = equity;
    let mut max_drawdown = 0.0;
    let mut trades = Vec::new();
    let mut position: Option<f64> = None;

    let lower_candles = data
        .get_candles(strategy.lower_tf)
        .expect("No data for lower timeframe");

    // Need minimum data for both timeframes
    let start_idx = strategy.trend_period.max(strategy.signal_period * 2) + 10;

    for i in start_idx..lower_candles.len() {
        // Create data slice up to current moment
        let current_lower = &lower_candles[..=i];

        // For higher timeframe, take corresponding slice
        let higher_candles = data
            .get_candles(strategy.higher_tf)
            .expect("No data for higher timeframe");

        // Form temporary data for analysis
        let mut current_data = MultiTimeFrameData::new(&data.symbol);
        current_data.add_candles(strategy.lower_tf, current_lower.to_vec());
        current_data.add_candles(strategy.higher_tf, higher_candles.to_vec());

        // Get signal
        let signal = strategy.analyze(&current_data);
        let current_price = lower_candles[i].close;

        match (signal, position) {
            (Some(Signal::Buy), None) => {
                // Open long position
                position = Some(current_price);
            }
            (Some(Signal::Sell), Some(entry_price)) => {
                // Close position
                let profit_pct = (current_price - entry_price) / entry_price;
                equity *= 1.0 + profit_pct;

                trades.push(profit_pct > 0.0);
                position = None;

                // Update maximum drawdown
                if equity > peak_equity {
                    peak_equity = equity;
                }
                let drawdown = (peak_equity - equity) / peak_equity;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
            _ => {}
        }
    }

    // Close any open position at the end
    if let Some(entry_price) = position {
        let last_price = lower_candles.last().unwrap().close;
        let profit_pct = (last_price - entry_price) / entry_price;
        equity *= 1.0 + profit_pct;
        trades.push(profit_pct > 0.0);
    }

    let winning_trades = trades.iter().filter(|&&win| win).count();
    let total_trades = trades.len();
    let total_profit = (equity - 10_000.0) / 10_000.0 * 100.0;
    let win_rate = if total_trades > 0 {
        winning_trades as f64 / total_trades as f64 * 100.0
    } else {
        0.0
    };

    BacktestResult {
        total_trades,
        winning_trades,
        total_profit,
        max_drawdown: max_drawdown * 100.0,
        win_rate,
    }
}

fn main() {
    println!("=== Multi-Timeframe Backtesting ===\n");

    // Generate data for 1-hour timeframe
    let h1_candles: Vec<Candle> = (0..1000)
        .map(|i| {
            let base = 50000.0 + (i as f64 * 0.05).sin() * 2000.0;
            let trend = i as f64 * 5.0;
            let noise = (i as f64 * 13.0).sin() * 100.0;
            let price = base + trend + noise;

            Candle {
                timestamp: i * 3600,
                open: price - 50.0,
                high: price + 100.0,
                low: price - 100.0,
                close: price,
                volume: 1000.0 + (i as f64 * 7.0).sin().abs() * 500.0,
            }
        })
        .collect();

    // Create multi-timeframe data
    let mut mtf_data = MultiTimeFrameData::new("BTC/USD");
    mtf_data.add_candles(TimeFrame::H1, h1_candles.clone());

    // Aggregate to 4-hour timeframe
    if let Some(h4_candles) = mtf_data.aggregate_timeframe(TimeFrame::H1, TimeFrame::H4) {
        mtf_data.add_candles(TimeFrame::H4, h4_candles);
    }

    // Create strategy: trend on H4, entry on H1
    let strategy = MultiTimeFrameStrategy::new(
        "BTC/USD",
        TimeFrame::H4, // Higher timeframe
        TimeFrame::H1, // Lower timeframe
        20,            // Period for trend detection
        10,            // Period for entry signals
    );

    // Run backtest
    let result = backtest_multi_timeframe(&mtf_data, &strategy);

    println!("Backtest Results:");
    println!("  Total trades: {}", result.total_trades);
    println!("  Winning trades: {}", result.winning_trades);
    println!("  Win Rate: {:.2}%", result.win_rate);
    println!("  Total profit: {:.2}%", result.total_profit);
    println!("  Maximum drawdown: {:.2}%", result.max_drawdown);

    if result.win_rate > 50.0 && result.total_profit > 10.0 {
        println!("\n✓ Strategy showed good results!");
    } else {
        println!("\n✗ Strategy needs improvement");
    }
}
```

## Advanced Techniques

### 1. Triple Timeframe Analysis

```rust
struct TripleTimeFrameStrategy {
    long_term: TimeFrame,   // Long-term trend (D1/W1)
    mid_term: TimeFrame,    // Medium-term trend (H4)
    short_term: TimeFrame,  // Short-term entry (H1)
}

impl TripleTimeFrameStrategy {
    fn analyze(&self, data: &MultiTimeFrameData) -> Option<Signal> {
        // All three timeframes must align for strong signal
        let long_trend = self.get_trend(data, self.long_term)?;
        let mid_trend = self.get_trend(data, self.mid_term)?;
        let short_signal = self.get_signal(data, self.short_term)?;

        match (long_trend, mid_trend, short_signal) {
            (Trend::Bullish, Trend::Bullish, Signal::Buy) => Some(Signal::Buy),
            (Trend::Bearish, Trend::Bearish, Signal::Sell) => Some(Signal::Sell),
            _ => None,
        }
    }

    fn get_trend(&self, data: &MultiTimeFrameData, tf: TimeFrame) -> Option<Trend> {
        let candles = data.get_candles(tf)?;
        Some(detect_trend(candles, 20))
    }

    fn get_signal(&self, data: &MultiTimeFrameData, tf: TimeFrame) -> Option<Signal> {
        let candles = data.get_candles(tf)?;
        if candles.len() < 20 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let ema = exponential_moving_average(&closes, 10);

        let len = closes.len();
        if closes[len - 1] > ema[len - 1] {
            Some(Signal::Buy)
        } else {
            Some(Signal::Sell)
        }
    }
}
```

### 2. Adaptive Stop-Loss by Timeframe

```rust
fn calculate_adaptive_stop_loss(
    candles: &[Candle],
    timeframe: TimeFrame,
) -> f64 {
    // Stop-loss depends on timeframe volatility
    let atr = calculate_atr(candles, 14);

    match timeframe {
        TimeFrame::H1 => atr * 1.5,
        TimeFrame::H4 => atr * 2.0,
        TimeFrame::D1 => atr * 2.5,
        _ => atr * 2.0,
    }
}

fn calculate_atr(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }

    let mut tr_sum = 0.0;

    for i in 1..=period {
        let high_low = candles[i].high - candles[i].low;
        let high_close = (candles[i].high - candles[i - 1].close).abs();
        let low_close = (candles[i].low - candles[i - 1].close).abs();

        let true_range = high_low.max(high_close).max(low_close);
        tr_sum += true_range;
    }

    tr_sum / period as f64
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Multi-timeframe analysis | Analyzing the market across multiple time periods |
| Higher timeframe | Determines overall trend direction |
| Lower timeframe | Provides precise entry and exit points |
| Data aggregation | Converting data from lower to higher timeframe |
| Signal filtering | Ignoring signals against higher timeframe trend |
| Adaptive risk management | Stop-losses depend on timeframe and volatility |

## Practical Exercises

1. **Compare Timeframes**: Test the same strategy on different timeframe combinations (H1-M15, H4-H1, D1-H4). Which combination gives the best results?

2. **Implement RSI for Multi-Timeframe**: Add RSI indicator and create a strategy where RSI on higher timeframe determines overbought/oversold zones, and lower timeframe gives entry point.

3. **Volatility Across Timeframes**: Create a function that calculates ATR for all timeframes and uses this information for dynamic position sizing.

4. **Signal Visualization**: Extend the code to log moments when higher timeframe shows one trend but lower timeframe shows another. How many such conflicts occur?

## Homework

1. **Four-Level Strategy**: Implement a strategy with four timeframes (W1, D1, H4, H1), where each level gives confirmation for the next.

2. **Strength Index**: Create a trend strength index (0-100%) that considers alignment of all timeframes. Trade only when index > 70%.

3. **Backtesting Comparison**: Compare results of:
   - Strategy only on H1
   - Strategy with H4 confirmation
   - Strategy with triple timeframe (D1-H4-H1)

   Build a metrics table for each variant.

4. **Real-time Simulation**: Create a real-time simulation where data comes candle by candle on lower timeframe, and higher timeframes update accordingly.

## Navigation

[← Previous day](../293-grid-search-parameter-sweep/en.md) | [Next day →](../299-advanced-testing-techniques/en.md)
