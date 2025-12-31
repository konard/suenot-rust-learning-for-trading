# Day 248: EMA: Exponential Moving Average

## Trading Analogy

In the previous lesson, we learned about SMA (Simple Moving Average), where all prices have equal weight. But imagine this situation: you're analyzing Bitcoin's price, and yesterday's price is more important to you than the price from a month ago. The market changes, and recent data should have more influence on your decisions.

**EMA (Exponential Moving Average)** is a moving average that gives **more weight to recent prices**. It's like conducting a survey among traders where the opinions of those who traded yesterday count more than those who traded a month ago.

**Why EMA is more important than SMA for trading:**
- Reacts faster to price changes
- Better at capturing trends
- Less lag during sharp market movements
- Used in popular indicators (MACD, Bollinger Bands)

## EMA Formula

EMA is calculated using a recursive formula:

```
EMA_today = Price_today × k + EMA_yesterday × (1 - k)
```

where **k** is the smoothing coefficient (multiplier):

```
k = 2 / (period + 1)
```

For example, for EMA-10:
- k = 2 / (10 + 1) = 2 / 11 ≈ 0.1818

This means the latest price gets ~18% weight, while all previous EMA values get ~82%.

## Simple EMA Implementation

```rust
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut ema_values = Vec::with_capacity(prices.len() - period + 1);

    // Smoothing coefficient
    let k = 2.0 / (period as f64 + 1.0);

    // First EMA value = SMA for the first period
    let first_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    ema_values.push(first_sma);

    // Calculate remaining EMA values
    for i in period..prices.len() {
        let prev_ema = ema_values.last().unwrap();
        let current_ema = prices[i] * k + prev_ema * (1.0 - k);
        ema_values.push(current_ema);
    }

    ema_values
}

fn main() {
    // BTC closing prices for 15 days
    let prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43000.0,
        42700.0, 42900.0, 43200.0, 43500.0, 43300.0,
        43600.0, 43400.0, 43800.0, 44000.0, 44200.0,
    ];

    let ema_10 = calculate_ema(&prices, 10);

    println!("=== EMA-10 for BTC ===");
    for (i, ema) in ema_10.iter().enumerate() {
        println!("Day {}: EMA = ${:.2}", i + 10, ema);
    }
}
```

## Comparing EMA and SMA

```rust
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    prices
        .windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect()
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let k = 2.0 / (period as f64 + 1.0);
    let first_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;

    let mut ema_values = vec![first_sma];

    for i in period..prices.len() {
        let prev_ema = ema_values.last().unwrap();
        let current_ema = prices[i] * k + prev_ema * (1.0 - k);
        ema_values.push(current_ema);
    }

    ema_values
}

fn main() {
    // Simulate a sharp price increase
    let prices = vec![
        100.0, 100.0, 100.0, 100.0, 100.0,  // Stable price
        100.0, 100.0, 100.0, 100.0, 100.0,
        120.0, 125.0, 130.0, 128.0, 135.0,  // Sharp increase
    ];

    let sma_5 = calculate_sma(&prices, 5);
    let ema_5 = calculate_ema(&prices, 5);

    println!("=== Comparing SMA-5 and EMA-5 During Sharp Rise ===\n");
    println!("{:<10} {:>10} {:>10} {:>10}", "Day", "Price", "SMA-5", "EMA-5");
    println!("{}", "-".repeat(45));

    for i in 0..sma_5.len() {
        let day = i + 5;
        let price = prices[day - 1];
        println!(
            "{:<10} {:>10.2} {:>10.2} {:>10.2}",
            day, price, sma_5[i], ema_5[i]
        );
    }

    println!("\nNotice: EMA reacts faster to the price increase!");
}
```

## EMA Indicator Structure

```rust
#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    /// Adds a new price and returns the current EMA value (if available)
    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                // Accumulate data for the first SMA
                self.initial_sum += price;

                if self.prices_count >= self.period {
                    // First EMA value = SMA
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                // Calculate new EMA value
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    /// Returns current EMA value without adding a new price
    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }

    /// Checks if the indicator is ready (enough data accumulated)
    pub fn is_ready(&self) -> bool {
        self.current_ema.is_some()
    }

    /// Resets the indicator
    pub fn reset(&mut self) {
        self.current_ema = None;
        self.prices_count = 0;
        self.initial_sum = 0.0;
    }
}

fn main() {
    let mut ema = EmaIndicator::new(5);

    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    println!("=== Streaming EMA-5 Calculation ===\n");

    for (i, &price) in prices.iter().enumerate() {
        let ema_value = ema.update(price);

        match ema_value {
            Some(value) => {
                println!("Day {}: Price = ${:.2}, EMA-5 = ${:.2}", i + 1, price, value);
            }
            None => {
                println!("Day {}: Price = ${:.2}, EMA-5 = (accumulating data...)", i + 1, price);
            }
        }
    }
}
```

## EMA Crossover: Trading Strategy

One of the most popular strategies is the crossover of fast and slow EMAs:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug)]
pub struct EmaCrossover {
    fast_ema: EmaIndicator,
    slow_ema: EmaIndicator,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

impl EmaCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");

        EmaCrossover {
            fast_ema: EmaIndicator::new(fast_period),
            slow_ema: EmaIndicator::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }

    pub fn update(&mut self, price: f64) -> Signal {
        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        let signal = match (fast, slow, self.prev_fast, self.prev_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Fast crosses slow from below = buy signal
                if pf <= ps && f > s {
                    Signal::Buy
                }
                // Fast crosses slow from above = sell signal
                else if pf >= ps && f < s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.prev_fast = fast;
        self.prev_slow = slow;

        signal
    }

    pub fn fast_ema(&self) -> Option<f64> {
        self.fast_ema.value()
    }

    pub fn slow_ema(&self) -> Option<f64> {
        self.slow_ema.value()
    }
}

fn main() {
    let mut strategy = EmaCrossover::new(5, 10);

    // Simulate price movement: first rise, then fall
    let prices = vec![
        // Initial period
        100.0, 101.0, 102.0, 101.5, 103.0,
        104.0, 105.0, 106.0, 107.0, 108.0,
        // Rise (should trigger Buy signal)
        110.0, 112.0, 115.0, 118.0, 120.0,
        // Reversal and fall (should trigger Sell signal)
        118.0, 115.0, 112.0, 108.0, 105.0,
        102.0, 100.0, 98.0, 95.0, 92.0,
    ];

    println!("=== EMA Crossover Strategy (5/10) ===\n");
    println!("{:<6} {:>10} {:>12} {:>12} {:>10}", "Day", "Price", "EMA-5", "EMA-10", "Signal");
    println!("{}", "-".repeat(55));

    for (i, &price) in prices.iter().enumerate() {
        let signal = strategy.update(price);

        let fast_str = strategy.fast_ema()
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "---".to_string());

        let slow_str = strategy.slow_ema()
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "---".to_string());

        let signal_str = match signal {
            Signal::Buy => ">>> BUY <<<",
            Signal::Sell => "<<< SELL >>>",
            Signal::Hold => "",
        };

        println!(
            "{:<6} {:>10.2} {:>12} {:>12} {:>10}",
            i + 1, price, fast_str, slow_str, signal_str
        );
    }
}
```

## Multi-Timeframe EMA Analysis

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

#[derive(Debug)]
pub struct MultiEmaAnalyzer {
    emas: HashMap<usize, EmaIndicator>,
}

impl MultiEmaAnalyzer {
    pub fn new(periods: &[usize]) -> Self {
        let mut emas = HashMap::new();
        for &period in periods {
            emas.insert(period, EmaIndicator::new(period));
        }
        MultiEmaAnalyzer { emas }
    }

    pub fn update(&mut self, price: f64) {
        for ema in self.emas.values_mut() {
            ema.update(price);
        }
    }

    pub fn get_ema(&self, period: usize) -> Option<f64> {
        self.emas.get(&period).and_then(|ema| ema.value())
    }

    /// Determines trend based on EMA alignment
    pub fn analyze_trend(&self) -> TrendAnalysis {
        let mut sorted_periods: Vec<_> = self.emas.keys().copied().collect();
        sorted_periods.sort();

        let mut values: Vec<(usize, f64)> = vec![];
        for &period in &sorted_periods {
            if let Some(value) = self.get_ema(period) {
                values.push((period, value));
            }
        }

        if values.len() < 2 {
            return TrendAnalysis::Unknown;
        }

        // Check if EMAs are sorted ascending (bullish trend)
        // or descending (bearish trend)
        let is_bullish = values.windows(2).all(|w| w[0].1 > w[1].1);
        let is_bearish = values.windows(2).all(|w| w[0].1 < w[1].1);

        if is_bullish {
            TrendAnalysis::Bullish
        } else if is_bearish {
            TrendAnalysis::Bearish
        } else {
            TrendAnalysis::Sideways
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendAnalysis {
    Bullish,   // Fast EMAs above slow EMAs
    Bearish,   // Fast EMAs below slow EMAs
    Sideways,  // EMAs intertwined
    Unknown,   // Not enough data
}

fn main() {
    let mut analyzer = MultiEmaAnalyzer::new(&[9, 21, 55, 200]);

    // Simulate steady growth
    let prices: Vec<f64> = (0..250)
        .map(|i| 100.0 + (i as f64) * 0.5 + (i as f64 * 0.1).sin() * 5.0)
        .collect();

    println!("=== Multi-EMA Analysis ===\n");

    for (i, &price) in prices.iter().enumerate() {
        analyzer.update(price);

        // Output every 50 days
        if (i + 1) % 50 == 0 {
            println!("Day {}:", i + 1);
            println!("  Price: ${:.2}", price);

            for &period in &[9, 21, 55, 200] {
                if let Some(ema) = analyzer.get_ema(period) {
                    println!("  EMA-{}: ${:.2}", period, ema);
                }
            }

            println!("  Trend: {:?}", analyzer.analyze_trend());
            println!();
        }
    }
}
```

## Practical Example: Trading Bot with EMA

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    position_size: f64,
    side: TradeSide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    pub fn pnl(&self) -> Option<f64> {
        self.exit_price.map(|exit| {
            match self.side {
                TradeSide::Long => (exit - self.entry_price) * self.position_size,
                TradeSide::Short => (self.entry_price - exit) * self.position_size,
            }
        })
    }
}

#[derive(Debug)]
pub struct EmaBot {
    fast_ema: EmaIndicator,
    slow_ema: EmaIndicator,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    current_position: Option<Trade>,
    closed_trades: Vec<Trade>,
    initial_capital: f64,
    current_capital: f64,
    risk_per_trade: f64,  // Percentage of capital per trade
}

impl EmaBot {
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        initial_capital: f64,
        risk_per_trade: f64,
    ) -> Self {
        EmaBot {
            fast_ema: EmaIndicator::new(fast_period),
            slow_ema: EmaIndicator::new(slow_period),
            prev_fast: None,
            prev_slow: None,
            current_position: None,
            closed_trades: vec![],
            initial_capital,
            current_capital: initial_capital,
            risk_per_trade,
        }
    }

    pub fn on_price(&mut self, price: f64) {
        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        if let (Some(f), Some(s), Some(pf), Some(ps)) = (fast, slow, self.prev_fast, self.prev_slow) {
            // Golden Cross: fast EMA crosses slow from below
            if pf <= ps && f > s {
                self.close_position_if_exists(price);
                self.open_long(price);
            }
            // Death Cross: fast EMA crosses slow from above
            else if pf >= ps && f < s {
                self.close_position_if_exists(price);
                self.open_short(price);
            }
        }

        self.prev_fast = fast;
        self.prev_slow = slow;
    }

    fn open_long(&mut self, price: f64) {
        let position_value = self.current_capital * self.risk_per_trade;
        let size = position_value / price;

        self.current_position = Some(Trade {
            entry_price: price,
            exit_price: None,
            position_size: size,
            side: TradeSide::Long,
        });

        println!("  → OPENED LONG position: {} BTC @ ${:.2}", size, price);
    }

    fn open_short(&mut self, price: f64) {
        let position_value = self.current_capital * self.risk_per_trade;
        let size = position_value / price;

        self.current_position = Some(Trade {
            entry_price: price,
            exit_price: None,
            position_size: size,
            side: TradeSide::Short,
        });

        println!("  → OPENED SHORT position: {} BTC @ ${:.2}", size, price);
    }

    fn close_position_if_exists(&mut self, price: f64) {
        if let Some(mut trade) = self.current_position.take() {
            trade.exit_price = Some(price);

            if let Some(pnl) = trade.pnl() {
                self.current_capital += pnl;
                println!(
                    "  ← CLOSED position @ ${:.2}, PnL: ${:.2}",
                    price, pnl
                );
            }

            self.closed_trades.push(trade);
        }
    }

    pub fn close_all(&mut self, price: f64) {
        self.close_position_if_exists(price);
    }

    pub fn report(&self) {
        println!("\n=== REPORT ===");
        println!("Initial capital: ${:.2}", self.initial_capital);
        println!("Current capital: ${:.2}", self.current_capital);

        let total_pnl = self.current_capital - self.initial_capital;
        let return_pct = (total_pnl / self.initial_capital) * 100.0;

        println!("Total PnL: ${:.2} ({:.2}%)", total_pnl, return_pct);
        println!("Total trades: {}", self.closed_trades.len());

        let winning = self.closed_trades.iter()
            .filter(|t| t.pnl().unwrap_or(0.0) > 0.0)
            .count();

        if !self.closed_trades.is_empty() {
            let win_rate = (winning as f64 / self.closed_trades.len() as f64) * 100.0;
            println!("Win Rate: {:.1}%", win_rate);
        }
    }
}

fn main() {
    let mut bot = EmaBot::new(9, 21, 10000.0, 0.5);  // 50% capital per trade

    // Simulate volatile market
    let prices: Vec<f64> = vec![
        // Start
        100.0, 101.0, 99.0, 102.0, 100.0, 103.0, 101.0, 104.0, 102.0, 105.0,
        103.0, 106.0, 104.0, 107.0, 105.0, 108.0, 106.0, 109.0, 107.0, 110.0,
        108.0, 111.0,
        // Steady rise
        112.0, 115.0, 118.0, 120.0, 123.0, 125.0, 128.0, 130.0, 133.0, 135.0,
        // Reversal and fall
        132.0, 128.0, 125.0, 120.0, 115.0, 110.0, 105.0, 100.0, 95.0, 90.0,
        // Recovery
        92.0, 95.0, 98.0, 100.0, 103.0, 106.0, 110.0, 115.0, 120.0, 125.0,
    ];

    println!("=== EMA Trading Bot (9/21) ===\n");

    for (i, &price) in prices.iter().enumerate() {
        println!("Day {}: Price = ${:.2}", i + 1, price);
        bot.on_price(price);
    }

    // Close all positions at the last price
    bot.close_all(*prices.last().unwrap());

    bot.report();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| EMA | Exponential Moving Average — gives more weight to recent prices |
| Multiplier k | k = 2 / (period + 1), determines the weight of the latest price |
| EMA formula | EMA = Price × k + EMA_prev × (1 - k) |
| EMA vs SMA | EMA reacts faster to price changes |
| EMA Crossover | Strategy based on fast and slow EMA crossing |
| Golden Cross | Fast EMA crosses slow from below — buy signal |
| Death Cross | Fast EMA crosses slow from above — sell signal |

## Homework

1. **Period Optimization**: Write a function that tests different EMA period combinations (e.g., 5-10, 9-21, 12-26) on historical data and finds the most profitable combination.

2. **EMA with Trend Filter**: Add a trend filter to the EMA Crossover strategy based on EMA-200. Only open long positions when price is above EMA-200, and only short positions when below.

3. **Adaptive EMA**: Implement KAMA (Kaufman Adaptive Moving Average), where the EMA period automatically adjusts based on market volatility.

4. **Triple EMA**: Create a strategy with three EMAs (fast, medium, slow). A buy signal appears only when all three EMAs are aligned correctly (fast > medium > slow).

## Navigation

[← Previous day](../247-sma-simple-moving-average/en.md) | [Next day →](../249-rsi-relative-strength-index/en.md)
