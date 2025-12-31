# Day 249: RSI: Relative Strength Index

## Trading Analogy

Imagine you're watching a football match. One team attacks again and again — they're in an "overbought" state, tired, and about to lose momentum. The other team only defends — they're in an "oversold" state, but soon might launch a counterattack.

**RSI (Relative Strength Index)** is an indicator that shows how "tired" the current trend is. If an asset's price has been rising for too long, RSI will show a high value (above 70) — a signal of overbought conditions. If the price has been falling for too long, RSI will show a low value (below 30) — a signal of oversold conditions.

In real trading, RSI helps to:
- Identify trend reversal moments
- Find entry and exit points
- Assess the strength of current price movement

## What is RSI?

RSI (Relative Strength Index) is a momentum oscillator that measures the speed and magnitude of price changes. It was developed by J. Welles Wilder in 1978.

### RSI Formula

```
RSI = 100 - (100 / (1 + RS))

where RS = Average Gain / Average Loss
```

The RSI value is always between 0 and 100:
- **RSI > 70** — asset is overbought (possible reversal down)
- **RSI < 30** — asset is oversold (possible reversal up)
- **RSI ≈ 50** — neutral zone

## Simple RSI Calculation

Let's see how to calculate RSI for a sequence of closing prices:

```rust
/// Calculates price changes between consecutive periods
fn calculate_price_changes(prices: &[f64]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

/// Separates changes into gains and losses
fn separate_gains_losses(changes: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let gains: Vec<f64> = changes.iter()
        .map(|&c| if c > 0.0 { c } else { 0.0 })
        .collect();

    let losses: Vec<f64> = changes.iter()
        .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
        .collect();

    (gains, losses)
}

/// Calculates simple average
fn simple_average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculates RSI for a given period
fn calculate_rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 {
        return None; // Not enough data
    }

    let changes = calculate_price_changes(prices);
    let (gains, losses) = separate_gains_losses(&changes);

    // Take the last `period` values
    let recent_gains = &gains[gains.len().saturating_sub(period)..];
    let recent_losses = &losses[losses.len().saturating_sub(period)..];

    let avg_gain = simple_average(recent_gains);
    let avg_loss = simple_average(recent_losses);

    if avg_loss == 0.0 {
        return Some(100.0); // No losses — RSI = 100
    }

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    Some(rsi)
}

fn main() {
    // BTC closing prices for 15 days
    let btc_prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43200.0,
        43100.0, 43500.0, 44000.0, 43800.0, 44200.0,
        44500.0, 44300.0, 44100.0, 44600.0, 45000.0,
    ];

    println!("BTC closing prices:");
    for (i, price) in btc_prices.iter().enumerate() {
        println!("  Day {}: ${:.2}", i + 1, price);
    }

    if let Some(rsi) = calculate_rsi(&btc_prices, 14) {
        println!("\nRSI (14 periods): {:.2}", rsi);

        if rsi > 70.0 {
            println!("Signal: Overbought — possible reversal down");
        } else if rsi < 30.0 {
            println!("Signal: Oversold — possible reversal up");
        } else {
            println!("Signal: Neutral zone");
        }
    }
}
```

## RSI Calculator Structure

Let's create a structure for tracking RSI in real-time:

```rust
/// RSI calculator with streaming update support
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    /// Creates a new RSI calculator
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    /// Adds a new price and returns current RSI
    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None; // Not enough data
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            // First calculation — simple average
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            // Exponential smoothing (Wilder's method)
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        self.calculate_rsi()
    }

    /// Calculates current RSI
    fn calculate_rsi(&self) -> Option<f64> {
        if !self.initialized {
            return None;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Returns interpretation of current RSI
    fn interpret(&self) -> &'static str {
        match self.calculate_rsi() {
            Some(rsi) if rsi > 70.0 => "Overbought",
            Some(rsi) if rsi < 30.0 => "Oversold",
            Some(_) => "Neutral zone",
            None => "Not enough data",
        }
    }
}

fn main() {
    let mut rsi_calc = RsiCalculator::new(14);

    // Simulate real-time price feed
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42350.0,
        42300.0, 42450.0, 42600.0, 42550.0, 42700.0,
        42850.0, 43000.0, 43150.0, 43300.0, 43250.0,
        43400.0, 43550.0, 43700.0, 43650.0, 43800.0,
    ];

    println!("Streaming RSI calculation for BTC:\n");

    for (i, price) in prices.iter().enumerate() {
        if let Some(rsi) = rsi_calc.update(*price) {
            println!(
                "Day {:2}: Price ${:.2} | RSI: {:5.2} | {}",
                i + 1,
                price,
                rsi,
                rsi_calc.interpret()
            );
        } else {
            println!(
                "Day {:2}: Price ${:.2} | RSI: accumulating data...",
                i + 1,
                price
            );
        }
    }
}
```

## RSI with Trading Signals

Let's implement a trading signal system based on RSI:

```rust
/// Trading signal type
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    StrongBuy,    // RSI < 20
    Buy,          // RSI < 30
    Hold,         // 30 <= RSI <= 70
    Sell,         // RSI > 70
    StrongSell,   // RSI > 80
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Signal::StrongBuy => write!(f, "STRONG BUY"),
            Signal::Buy => write!(f, "BUY"),
            Signal::Hold => write!(f, "HOLD"),
            Signal::Sell => write!(f, "SELL"),
            Signal::StrongSell => write!(f, "STRONG SELL"),
        }
    }
}

/// RSI-based trading strategy
struct RsiStrategy {
    rsi_calculator: RsiCalculator,
    oversold_level: f64,
    overbought_level: f64,
    strong_oversold: f64,
    strong_overbought: f64,
}

impl RsiStrategy {
    fn new(period: usize) -> Self {
        RsiStrategy {
            rsi_calculator: RsiCalculator::new(period),
            oversold_level: 30.0,
            overbought_level: 70.0,
            strong_oversold: 20.0,
            strong_overbought: 80.0,
        }
    }

    /// Updates the strategy and returns a signal
    fn update(&mut self, price: f64) -> Option<(f64, Signal)> {
        let rsi = self.rsi_calculator.update(price)?;

        let signal = if rsi < self.strong_oversold {
            Signal::StrongBuy
        } else if rsi < self.oversold_level {
            Signal::Buy
        } else if rsi > self.strong_overbought {
            Signal::StrongSell
        } else if rsi > self.overbought_level {
            Signal::Sell
        } else {
            Signal::Hold
        };

        Some((rsi, signal))
    }
}

/// RSI Calculator (reusing previous implementation)
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None;
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

fn main() {
    let mut strategy = RsiStrategy::new(14);

    // Simulate a volatile market
    let prices = vec![
        50000.0, 50500.0, 51000.0, 51500.0, 52000.0,
        52500.0, 53000.0, 53500.0, 54000.0, 54500.0,
        55000.0, 55500.0, 56000.0, 56500.0, 57000.0,
        // Sharp rise — RSI should show overbought
        58000.0, 59000.0, 60000.0, 61000.0, 62000.0,
        // Correction
        61000.0, 60000.0, 59000.0, 58000.0, 57000.0,
        56000.0, 55000.0, 54000.0, 53000.0, 52000.0,
        // Strong drop — RSI should show oversold
        51000.0, 50000.0, 49000.0, 48000.0, 47000.0,
    ];

    println!("RSI Strategy for BTC:\n");
    println!("{:>4} | {:>10} | {:>6} | {:>12}", "Day", "Price", "RSI", "Signal");
    println!("{}", "-".repeat(42));

    for (i, price) in prices.iter().enumerate() {
        if let Some((rsi, signal)) = strategy.update(*price) {
            println!(
                "{:>4} | ${:>9.0} | {:>5.1} | {}",
                i + 1,
                price,
                rsi,
                signal
            );
        }
    }
}
```

## RSI Divergence

Divergence is a discrepancy between price and RSI that can signal a trend reversal:

```rust
/// Divergence type
#[derive(Debug, PartialEq)]
enum Divergence {
    Bullish,   // Price makes new low, RSI doesn't (bullish)
    Bearish,   // Price makes new high, RSI doesn't (bearish)
    None,
}

/// Divergence detector
struct DivergenceDetector {
    price_highs: Vec<f64>,
    price_lows: Vec<f64>,
    rsi_at_highs: Vec<f64>,
    rsi_at_lows: Vec<f64>,
    lookback: usize,
}

impl DivergenceDetector {
    fn new(lookback: usize) -> Self {
        DivergenceDetector {
            price_highs: Vec::new(),
            price_lows: Vec::new(),
            rsi_at_highs: Vec::new(),
            rsi_at_lows: Vec::new(),
            lookback,
        }
    }

    /// Records a local high
    fn record_high(&mut self, price: f64, rsi: f64) {
        self.price_highs.push(price);
        self.rsi_at_highs.push(rsi);

        // Keep only the most recent values
        if self.price_highs.len() > self.lookback {
            self.price_highs.remove(0);
            self.rsi_at_highs.remove(0);
        }
    }

    /// Records a local low
    fn record_low(&mut self, price: f64, rsi: f64) {
        self.price_lows.push(price);
        self.rsi_at_lows.push(rsi);

        if self.price_lows.len() > self.lookback {
            self.price_lows.remove(0);
            self.rsi_at_lows.remove(0);
        }
    }

    /// Checks for bearish divergence
    fn check_bearish_divergence(&self) -> bool {
        if self.price_highs.len() < 2 {
            return false;
        }

        let len = self.price_highs.len();
        let prev_price = self.price_highs[len - 2];
        let curr_price = self.price_highs[len - 1];
        let prev_rsi = self.rsi_at_highs[len - 2];
        let curr_rsi = self.rsi_at_highs[len - 1];

        // Price higher but RSI lower — bearish divergence
        curr_price > prev_price && curr_rsi < prev_rsi
    }

    /// Checks for bullish divergence
    fn check_bullish_divergence(&self) -> bool {
        if self.price_lows.len() < 2 {
            return false;
        }

        let len = self.price_lows.len();
        let prev_price = self.price_lows[len - 2];
        let curr_price = self.price_lows[len - 1];
        let prev_rsi = self.rsi_at_lows[len - 2];
        let curr_rsi = self.rsi_at_lows[len - 1];

        // Price lower but RSI higher — bullish divergence
        curr_price < prev_price && curr_rsi > prev_rsi
    }

    /// Detects current divergence
    fn detect(&self) -> Divergence {
        if self.check_bullish_divergence() {
            Divergence::Bullish
        } else if self.check_bearish_divergence() {
            Divergence::Bearish
        } else {
            Divergence::None
        }
    }
}

fn main() {
    let mut detector = DivergenceDetector::new(5);

    // Simulate bearish divergence:
    // Price makes new highs, but RSI falls
    println!("Demonstrating bearish divergence:\n");

    detector.record_high(50000.0, 75.0);
    println!("High 1: Price $50,000, RSI: 75.0");

    detector.record_high(52000.0, 72.0);
    println!("High 2: Price $52,000, RSI: 72.0");

    detector.record_high(54000.0, 68.0);
    println!("High 3: Price $54,000, RSI: 68.0");

    println!("\nAnalysis: Price rising ($50K -> $52K -> $54K)");
    println!("          RSI falling (75 -> 72 -> 68)");
    println!("Result: {:?}", detector.detect());
    println!("Interpretation: Upward momentum weakening, possible reversal down");

    // Simulate bullish divergence
    let mut detector2 = DivergenceDetector::new(5);

    println!("\n{}\n", "=".repeat(50));
    println!("Demonstrating bullish divergence:\n");

    detector2.record_low(50000.0, 25.0);
    println!("Low 1: Price $50,000, RSI: 25.0");

    detector2.record_low(48000.0, 28.0);
    println!("Low 2: Price $48,000, RSI: 28.0");

    detector2.record_low(46000.0, 32.0);
    println!("Low 3: Price $46,000, RSI: 32.0");

    println!("\nAnalysis: Price falling ($50K -> $48K -> $46K)");
    println!("          RSI rising (25 -> 28 -> 32)");
    println!("Result: {:?}", detector2.detect());
    println!("Interpretation: Downward momentum weakening, possible reversal up");
}
```

## Multi-Timeframe RSI

Analyzing RSI across multiple timeframes for more reliable signals:

```rust
use std::collections::HashMap;

/// Timeframe
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Timeframe {
    Minutes15,
    Hour1,
    Hours4,
    Daily,
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Timeframe::Minutes15 => write!(f, "15M"),
            Timeframe::Hour1 => write!(f, "1H"),
            Timeframe::Hours4 => write!(f, "4H"),
            Timeframe::Daily => write!(f, "1D"),
        }
    }
}

/// Multi-timeframe RSI analyzer
struct MultiTimeframeRsi {
    analyzers: HashMap<Timeframe, RsiCalculator>,
}

impl MultiTimeframeRsi {
    fn new(period: usize) -> Self {
        let mut analyzers = HashMap::new();
        analyzers.insert(Timeframe::Minutes15, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Hour1, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Hours4, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Daily, RsiCalculator::new(period));

        MultiTimeframeRsi { analyzers }
    }

    /// Updates RSI for the specified timeframe
    fn update(&mut self, timeframe: Timeframe, price: f64) -> Option<f64> {
        self.analyzers.get_mut(&timeframe)?.update(price)
    }

    /// Gets consensus across all timeframes
    fn get_consensus(&self) -> String {
        let mut bullish = 0;
        let mut bearish = 0;
        let mut neutral = 0;

        for (tf, calc) in &self.analyzers {
            if let Some(rsi) = calc.calculate_rsi() {
                if rsi < 30.0 {
                    bullish += 1;
                } else if rsi > 70.0 {
                    bearish += 1;
                } else {
                    neutral += 1;
                }
            }
        }

        if bullish > bearish && bullish > neutral {
            "Bullish consensus (buy)".to_string()
        } else if bearish > bullish && bearish > neutral {
            "Bearish consensus (sell)".to_string()
        } else {
            "Mixed signals (wait)".to_string()
        }
    }

    /// Prints current RSI values across all timeframes
    fn print_status(&self) {
        println!("\nMulti-Timeframe RSI:");
        println!("{}", "-".repeat(30));

        for tf in [Timeframe::Minutes15, Timeframe::Hour1,
                   Timeframe::Hours4, Timeframe::Daily] {
            if let Some(calc) = self.analyzers.get(&tf) {
                if let Some(rsi) = calc.calculate_rsi() {
                    let status = if rsi < 30.0 {
                        "Oversold"
                    } else if rsi > 70.0 {
                        "Overbought"
                    } else {
                        "Neutral"
                    };
                    println!("{:>4}: RSI {:5.1} ({})", tf, rsi, status);
                }
            }
        }

        println!("\nConsensus: {}", self.get_consensus());
    }
}

/// RSI Calculator (using previous implementation)
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None;
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        self.calculate_rsi()
    }

    fn calculate_rsi(&self) -> Option<f64> {
        if !self.initialized {
            return None;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

fn main() {
    let mut mtf_rsi = MultiTimeframeRsi::new(14);

    // Simulate data for different timeframes
    // 15-minute data (more volatile)
    let m15_prices = vec![
        100.0, 101.0, 100.5, 102.0, 103.0, 102.5, 104.0, 105.0,
        104.5, 106.0, 107.0, 106.5, 108.0, 109.0, 108.5, 110.0,
    ];

    // Hourly data (medium volatility)
    let h1_prices = vec![
        100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0,
        110.0, 109.0, 111.0, 113.0, 112.0, 114.0, 116.0, 115.0,
    ];

    // 4-hour data (smoothed)
    let h4_prices = vec![
        100.0, 105.0, 110.0, 108.0, 112.0, 115.0, 113.0, 118.0,
        120.0, 118.0, 122.0, 125.0, 123.0, 128.0, 130.0, 128.0,
    ];

    // Daily data (most smoothed)
    let daily_prices = vec![
        100.0, 110.0, 105.0, 115.0, 120.0, 118.0, 125.0, 130.0,
        128.0, 135.0, 140.0, 138.0, 145.0, 150.0, 148.0, 155.0,
    ];

    // Update RSI for each timeframe
    for price in m15_prices {
        mtf_rsi.update(Timeframe::Minutes15, price);
    }

    for price in h1_prices {
        mtf_rsi.update(Timeframe::Hour1, price);
    }

    for price in h4_prices {
        mtf_rsi.update(Timeframe::Hours4, price);
    }

    for price in daily_prices {
        mtf_rsi.update(Timeframe::Daily, price);
    }

    mtf_rsi.print_status();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| RSI | Relative Strength Index — oscillator from 0 to 100 |
| Overbought | RSI > 70 — possible reversal down |
| Oversold | RSI < 30 — possible reversal up |
| RS | Relative Strength = Average Gain / Average Loss |
| Wilder's Smoothing | Exponential smoothing for calculating averages |
| Divergence | Discrepancy between price and RSI — reversal signal |
| Multi-Timeframe | RSI analysis across multiple time intervals |

## Homework

1. **RSI Calculator**: Implement an `RsiIndicator` structure with methods:
   - `new(period: usize)` — create with specified period
   - `add_price(price: f64)` — add a new price
   - `get_rsi()` — get current RSI value
   - `get_signal()` — get trading signal (Buy/Sell/Hold)

2. **RSI with Configurable Levels**: Extend the calculator by adding:
   - Configurable overbought/oversold levels
   - RSI value history for the last N periods
   - Method to determine RSI trend (rising/falling/sideways)

3. **Divergence Detector**: Create a system that:
   - Automatically identifies local price highs and lows
   - Compares them with corresponding RSI values
   - Issues a warning when divergence is detected

4. **RSI Strategy Backtesting**: Write a program that:
   - Loads historical data (can use a price array)
   - Applies an RSI strategy (buy when RSI < 30, sell when RSI > 70)
   - Calculates total profit/loss
   - Compares with "buy and hold" strategy

## Navigation

[← Previous day](../248-ema-exponential-moving-average/en.md) | [Next day →](../250-macd-moving-average-convergence-divergence/en.md)
