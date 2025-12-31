# Day 254: Volume Analysis

## Trading Analogy

Imagine you're a trader watching the order flow on an exchange. Price is moving up, but is it a strong move or a weak one? The answer lies in **volume** — the total number of shares, contracts, or coins traded during a given period. If the price rises on high volume, buyers are in control and the trend is strong. If the price rises on low volume, the move might be a "false breakout" — a weak rally that could reverse.

Volume is like the crowd at an auction: a rising price with loud bidding (high volume) means real demand, while a rising price in a quiet room (low volume) suggests weak conviction.

In algorithmic trading, analyzing volume helps you:
- Confirm price trends and breakouts
- Detect potential reversals (divergence)
- Identify accumulation and distribution phases
- Measure market liquidity and impact costs

## What is Volume Analysis?

Volume analysis is the study of trading volume alongside price movements to make better trading decisions. Key concepts include:

1. **Volume Bars** — the amount traded in each time period
2. **Volume Moving Average** — smoothed volume to identify above/below average activity
3. **Volume-Price Relationship** — confirming trends when price and volume move together
4. **Volume Profile** — distribution of volume across price levels
5. **On-Balance Volume (OBV)** — cumulative indicator linking volume to price direction

## Basic Volume Data Structure

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct VolumeBar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u32,
}

impl VolumeBar {
    pub fn new(timestamp: u64, price: f64, volume: f64) -> Self {
        VolumeBar {
            timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            trade_count: 1,
        }
    }

    pub fn update(&mut self, price: f64, volume: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
        self.trade_count += 1;
    }

    /// Calculate the typical price (average of high, low, close)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Volume-Weighted Average Price for this bar
    pub fn vwap(&self) -> f64 {
        self.typical_price() // Simplified; real VWAP needs tick data
    }

    /// Is this a bullish bar (close > open)?
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

fn main() {
    let mut bar = VolumeBar::new(1704067200, 42000.0, 0.5);
    bar.update(42100.0, 0.3);
    bar.update(41950.0, 0.2);
    bar.update(42200.0, 0.8);

    println!("Volume Bar: {:?}", bar);
    println!("Total Volume: {:.2}", bar.volume);
    println!("Typical Price: {:.2}", bar.typical_price());
    println!("Is Bullish: {}", bar.is_bullish());
}
```

## Volume Moving Average

```rust
use std::collections::VecDeque;

pub struct VolumeMovingAverage {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
}

impl VolumeMovingAverage {
    pub fn new(period: usize) -> Self {
        VolumeMovingAverage {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    pub fn update(&mut self, volume: f64) -> Option<f64> {
        self.volumes.push_back(volume);
        self.sum += volume;

        if self.volumes.len() > self.period {
            if let Some(old) = self.volumes.pop_front() {
                self.sum -= old;
            }
        }

        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<f64> {
        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    /// Check if current volume is above average (signal of strength)
    pub fn is_above_average(&self, current_volume: f64) -> Option<bool> {
        self.current().map(|avg| current_volume > avg)
    }

    /// Calculate volume ratio (current / average)
    pub fn volume_ratio(&self, current_volume: f64) -> Option<f64> {
        self.current().map(|avg| {
            if avg > 0.0 {
                current_volume / avg
            } else {
                1.0
            }
        })
    }
}

fn main() {
    let mut vma = VolumeMovingAverage::new(5);

    let volumes = vec![100.0, 150.0, 120.0, 180.0, 200.0, 300.0, 250.0];

    for (i, &vol) in volumes.iter().enumerate() {
        let avg = vma.update(vol);
        let ratio = vma.volume_ratio(vol);

        println!(
            "Bar {}: Volume={:.0}, Avg={:?}, Ratio={:.2}x",
            i + 1,
            vol,
            avg.map(|a| format!("{:.0}", a)),
            ratio.unwrap_or(1.0)
        );

        if let Some(true) = vma.is_above_average(vol) {
            println!("  -> Above average volume! Potential signal.");
        }
    }
}
```

## On-Balance Volume (OBV)

On-Balance Volume is a cumulative indicator that adds volume on up days and subtracts on down days:

```rust
#[derive(Debug)]
pub struct OnBalanceVolume {
    obv: f64,
    previous_close: Option<f64>,
    history: Vec<f64>,
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        OnBalanceVolume {
            obv: 0.0,
            previous_close: None,
            history: Vec::new(),
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) -> f64 {
        if let Some(prev_close) = self.previous_close {
            if close > prev_close {
                // Price up: add volume (buyers in control)
                self.obv += volume;
            } else if close < prev_close {
                // Price down: subtract volume (sellers in control)
                self.obv -= volume;
            }
            // If close == prev_close, OBV stays the same
        }

        self.previous_close = Some(close);
        self.history.push(self.obv);
        self.obv
    }

    pub fn current(&self) -> f64 {
        self.obv
    }

    /// Detect OBV divergence with price
    /// Returns Some(true) for bullish divergence, Some(false) for bearish
    pub fn check_divergence(&self, price_making_new_low: bool, price_making_new_high: bool) -> Option<bool> {
        if self.history.len() < 2 {
            return None;
        }

        let recent_obv = self.obv;
        let prev_obv = self.history[self.history.len() - 2];

        // Bullish divergence: price makes new low, but OBV doesn't
        if price_making_new_low && recent_obv > prev_obv {
            return Some(true);
        }

        // Bearish divergence: price makes new high, but OBV doesn't
        if price_making_new_high && recent_obv < prev_obv {
            return Some(false);
        }

        None
    }
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut obv = OnBalanceVolume::new();

    // Simulated price and volume data (uptrend with volume confirmation)
    let data = vec![
        (100.0, 1000.0),
        (102.0, 1500.0), // Up, add volume
        (101.0, 800.0),  // Down, subtract volume
        (104.0, 2000.0), // Up, add volume
        (106.0, 2500.0), // Up, add volume
        (105.0, 1200.0), // Down, subtract volume
    ];

    println!("On-Balance Volume Analysis:");
    println!("{:-<50}", "");

    for (i, (close, volume)) in data.iter().enumerate() {
        let obv_value = obv.update(*close, *volume);
        let direction = if i > 0 && *close > data[i - 1].0 {
            "UP"
        } else if i > 0 && *close < data[i - 1].0 {
            "DOWN"
        } else {
            "START"
        };

        println!(
            "Bar {}: Close={:.2}, Vol={:.0}, Direction={}, OBV={:.0}",
            i + 1,
            close,
            volume,
            direction,
            obv_value
        );
    }

    println!("{:-<50}", "");
    println!("Final OBV: {:.0}", obv.current());
    println!("Positive OBV indicates buying pressure dominated.");
}
```

## Volume Profile

Volume Profile shows how much volume traded at each price level:

```rust
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct VolumeProfile {
    /// Price level -> Volume traded at that level
    profile: BTreeMap<i64, f64>,
    tick_size: f64,
    total_volume: f64,
}

impl VolumeProfile {
    pub fn new(tick_size: f64) -> Self {
        VolumeProfile {
            profile: BTreeMap::new(),
            tick_size,
            total_volume: 0.0,
        }
    }

    /// Convert price to price level (bucket)
    fn price_to_level(&self, price: f64) -> i64 {
        (price / self.tick_size).round() as i64
    }

    /// Add volume at a specific price
    pub fn add_volume(&mut self, price: f64, volume: f64) {
        let level = self.price_to_level(price);
        *self.profile.entry(level).or_insert(0.0) += volume;
        self.total_volume += volume;
    }

    /// Get the Point of Control (price with highest volume)
    pub fn point_of_control(&self) -> Option<(f64, f64)> {
        self.profile
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&level, &volume)| (level as f64 * self.tick_size, volume))
    }

    /// Get the Value Area (price range containing 70% of volume)
    pub fn value_area(&self) -> Option<(f64, f64)> {
        if self.profile.is_empty() {
            return None;
        }

        let target_volume = self.total_volume * 0.70;
        let poc_level = self.profile
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&l, _)| l)?;

        let mut included_volume = *self.profile.get(&poc_level).unwrap_or(&0.0);
        let mut low_level = poc_level;
        let mut high_level = poc_level;

        // Expand from POC until we have 70% of volume
        while included_volume < target_volume {
            let below = self.profile.get(&(low_level - 1)).unwrap_or(&0.0);
            let above = self.profile.get(&(high_level + 1)).unwrap_or(&0.0);

            if below >= above && *below > 0.0 {
                low_level -= 1;
                included_volume += below;
            } else if *above > 0.0 {
                high_level += 1;
                included_volume += above;
            } else {
                break;
            }
        }

        Some((
            low_level as f64 * self.tick_size,
            high_level as f64 * self.tick_size,
        ))
    }

    /// Print the volume profile as a histogram
    pub fn print_histogram(&self, max_width: usize) {
        if self.profile.is_empty() {
            println!("No volume data");
            return;
        }

        let max_volume = self.profile.values().cloned().fold(0.0_f64, f64::max);
        let poc = self.point_of_control();

        println!("\nVolume Profile:");
        println!("{:-<60}", "");

        for (&level, &volume) in self.profile.iter().rev() {
            let price = level as f64 * self.tick_size;
            let bar_len = ((volume / max_volume) * max_width as f64) as usize;
            let bar: String = "#".repeat(bar_len);
            let is_poc = poc.map(|(p, _)| (p - price).abs() < self.tick_size / 2.0).unwrap_or(false);

            println!(
                "${:>8.2} | {:width$} {:.0}{}",
                price,
                bar,
                volume,
                if is_poc { " <- POC" } else { "" },
                width = max_width
            );
        }
    }
}

fn main() {
    let mut profile = VolumeProfile::new(100.0); // $100 price buckets

    // Simulate trades at various price levels
    let trades = vec![
        (41900.0, 50.0),
        (42000.0, 150.0),
        (42000.0, 200.0),
        (42100.0, 180.0),
        (42100.0, 120.0),
        (42100.0, 100.0),
        (42200.0, 80.0),
        (42200.0, 90.0),
        (42300.0, 40.0),
        (42000.0, 100.0),
    ];

    for (price, volume) in trades {
        profile.add_volume(price, volume);
    }

    profile.print_histogram(30);

    if let Some((poc_price, poc_volume)) = profile.point_of_control() {
        println!("\nPoint of Control: ${:.2} ({:.0} volume)", poc_price, poc_volume);
    }

    if let Some((va_low, va_high)) = profile.value_area() {
        println!("Value Area: ${:.2} - ${:.2}", va_low, va_high);
    }
}
```

## Volume-Based Trading Signals

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum VolumeSignal {
    StrongBuy,      // High volume + price up
    WeakBuy,        // Low volume + price up (potential reversal)
    StrongSell,     // High volume + price down
    WeakSell,       // Low volume + price down (potential reversal)
    Accumulation,   // High volume + price stable (smart money buying)
    Distribution,   // High volume + price stable (smart money selling)
    Neutral,
}

pub struct VolumeAnalyzer {
    volume_ma: VolumeMovingAverage,
    obv: OnBalanceVolume,
    volume_threshold: f64, // Multiplier for "high volume" (e.g., 1.5x average)
}

impl VolumeAnalyzer {
    pub fn new(ma_period: usize, volume_threshold: f64) -> Self {
        VolumeAnalyzer {
            volume_ma: VolumeMovingAverage::new(ma_period),
            obv: OnBalanceVolume::new(),
            volume_threshold,
        }
    }

    pub fn analyze(&mut self, bar: &VolumeBar) -> VolumeSignal {
        // Update indicators
        self.volume_ma.update(bar.volume);
        self.obv.update(bar.close, bar.volume);

        // Calculate volume ratio
        let volume_ratio = self.volume_ma.volume_ratio(bar.volume).unwrap_or(1.0);
        let is_high_volume = volume_ratio >= self.volume_threshold;

        // Calculate price change
        let price_change_pct = if bar.open > 0.0 {
            (bar.close - bar.open) / bar.open * 100.0
        } else {
            0.0
        };

        // Determine signal based on volume and price action
        match (is_high_volume, price_change_pct) {
            (true, pct) if pct > 0.5 => VolumeSignal::StrongBuy,
            (true, pct) if pct < -0.5 => VolumeSignal::StrongSell,
            (true, pct) if pct.abs() <= 0.5 => {
                // High volume but stable price - accumulation or distribution
                if self.obv.current() > 0.0 {
                    VolumeSignal::Accumulation
                } else {
                    VolumeSignal::Distribution
                }
            }
            (false, pct) if pct > 0.5 => VolumeSignal::WeakBuy,
            (false, pct) if pct < -0.5 => VolumeSignal::WeakSell,
            _ => VolumeSignal::Neutral,
        }
    }
}

// Include necessary structs from earlier
use std::collections::VecDeque;

pub struct VolumeMovingAverage {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
}

impl VolumeMovingAverage {
    pub fn new(period: usize) -> Self {
        VolumeMovingAverage {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    pub fn update(&mut self, volume: f64) -> Option<f64> {
        self.volumes.push_back(volume);
        self.sum += volume;
        if self.volumes.len() > self.period {
            if let Some(old) = self.volumes.pop_front() {
                self.sum -= old;
            }
        }
        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    pub fn volume_ratio(&self, current_volume: f64) -> Option<f64> {
        if self.volumes.len() == self.period && self.sum > 0.0 {
            Some(current_volume / (self.sum / self.period as f64))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct OnBalanceVolume {
    obv: f64,
    previous_close: Option<f64>,
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        OnBalanceVolume {
            obv: 0.0,
            previous_close: None,
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) -> f64 {
        if let Some(prev) = self.previous_close {
            if close > prev {
                self.obv += volume;
            } else if close < prev {
                self.obv -= volume;
            }
        }
        self.previous_close = Some(close);
        self.obv
    }

    pub fn current(&self) -> f64 {
        self.obv
    }
}

#[derive(Debug, Clone)]
pub struct VolumeBar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u32,
}

fn main() {
    let mut analyzer = VolumeAnalyzer::new(5, 1.5);

    // Simulate market data
    let bars = vec![
        VolumeBar { timestamp: 1, open: 100.0, high: 101.0, low: 99.5, close: 100.5, volume: 1000.0, trade_count: 50 },
        VolumeBar { timestamp: 2, open: 100.5, high: 102.0, low: 100.0, close: 101.8, volume: 1200.0, trade_count: 60 },
        VolumeBar { timestamp: 3, open: 101.8, high: 103.0, low: 101.5, close: 102.5, volume: 1100.0, trade_count: 55 },
        VolumeBar { timestamp: 4, open: 102.5, high: 103.5, low: 102.0, close: 103.2, volume: 1500.0, trade_count: 75 },
        VolumeBar { timestamp: 5, open: 103.2, high: 105.0, low: 103.0, close: 104.8, volume: 2500.0, trade_count: 120 }, // High volume breakout
        VolumeBar { timestamp: 6, open: 104.8, high: 106.0, low: 104.5, close: 105.5, volume: 800.0, trade_count: 40 },  // Low volume continuation
        VolumeBar { timestamp: 7, open: 105.5, high: 106.0, low: 103.0, close: 103.5, volume: 3000.0, trade_count: 150 }, // High volume reversal
    ];

    println!("Volume Analysis Results:");
    println!("{:-<60}", "");

    for bar in &bars {
        let signal = analyzer.analyze(bar);
        let change = (bar.close - bar.open) / bar.open * 100.0;

        println!(
            "Bar {}: Close={:.2} ({:+.2}%), Vol={:.0} -> {:?}",
            bar.timestamp, bar.close, change, bar.volume, signal
        );
    }
}
```

## Practical Example: Volume-Weighted Trading Strategy

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub side: TradeSide,
    pub price: f64,
    pub quantity: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct VolumeStrategy {
    // Configuration
    volume_ma_period: usize,
    volume_spike_threshold: f64,
    position_size: f64,

    // State
    volume_history: VecDeque<f64>,
    price_history: VecDeque<f64>,
    position: f64,
    trades: Vec<Trade>,
}

impl VolumeStrategy {
    pub fn new(volume_ma_period: usize, spike_threshold: f64, position_size: f64) -> Self {
        VolumeStrategy {
            volume_ma_period,
            volume_spike_threshold: spike_threshold,
            position_size,
            volume_history: VecDeque::with_capacity(volume_ma_period),
            price_history: VecDeque::with_capacity(volume_ma_period),
            position: 0.0,
            trades: Vec::new(),
        }
    }

    fn average_volume(&self) -> Option<f64> {
        if self.volume_history.len() == self.volume_ma_period {
            Some(self.volume_history.iter().sum::<f64>() / self.volume_ma_period as f64)
        } else {
            None
        }
    }

    fn price_trend(&self) -> Option<f64> {
        if self.price_history.len() < 2 {
            return None;
        }
        let first = self.price_history.front()?;
        let last = self.price_history.back()?;
        Some((last - first) / first * 100.0)
    }

    pub fn on_bar(&mut self, timestamp: u64, close: f64, volume: f64) -> Option<Trade> {
        // Update history
        self.volume_history.push_back(volume);
        self.price_history.push_back(close);

        if self.volume_history.len() > self.volume_ma_period {
            self.volume_history.pop_front();
            self.price_history.pop_front();
        }

        // Need enough data
        let avg_volume = self.average_volume()?;
        let trend = self.price_trend()?;

        let volume_ratio = volume / avg_volume;
        let is_volume_spike = volume_ratio >= self.volume_spike_threshold;

        // Strategy logic
        let trade = if is_volume_spike && trend > 1.0 && self.position <= 0.0 {
            // Volume spike with uptrend - buy signal
            Some(Trade {
                timestamp,
                side: TradeSide::Buy,
                price: close,
                quantity: self.position_size,
                reason: format!(
                    "Volume spike ({:.1}x avg) with uptrend ({:+.2}%)",
                    volume_ratio, trend
                ),
            })
        } else if is_volume_spike && trend < -1.0 && self.position >= 0.0 {
            // Volume spike with downtrend - sell signal
            Some(Trade {
                timestamp,
                side: TradeSide::Sell,
                price: close,
                quantity: self.position_size,
                reason: format!(
                    "Volume spike ({:.1}x avg) with downtrend ({:+.2}%)",
                    volume_ratio, trend
                ),
            })
        } else if !is_volume_spike && self.position != 0.0 && trend.abs() < 0.5 {
            // Low volume and sideways - exit position
            let side = if self.position > 0.0 { TradeSide::Sell } else { TradeSide::Buy };
            Some(Trade {
                timestamp,
                side,
                price: close,
                quantity: self.position.abs(),
                reason: "Low volume exit - momentum fading".to_string(),
            })
        } else {
            None
        };

        // Update position
        if let Some(ref t) = trade {
            match t.side {
                TradeSide::Buy => self.position += t.quantity,
                TradeSide::Sell => self.position -= t.quantity,
            }
            self.trades.push(t.clone());
        }

        trade
    }

    pub fn get_trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn calculate_pnl(&self) -> f64 {
        let mut pnl = 0.0;
        let mut position = 0.0;
        let mut avg_entry = 0.0;

        for trade in &self.trades {
            match trade.side {
                TradeSide::Buy => {
                    if position < 0.0 {
                        // Closing short
                        pnl += (avg_entry - trade.price) * trade.quantity.min(-position);
                    }
                    let new_position = position + trade.quantity;
                    if new_position > 0.0 && position >= 0.0 {
                        avg_entry = (avg_entry * position + trade.price * trade.quantity)
                            / new_position;
                    } else if new_position > 0.0 {
                        avg_entry = trade.price;
                    }
                    position = new_position;
                }
                TradeSide::Sell => {
                    if position > 0.0 {
                        // Closing long
                        pnl += (trade.price - avg_entry) * trade.quantity.min(position);
                    }
                    let new_position = position - trade.quantity;
                    if new_position < 0.0 && position <= 0.0 {
                        avg_entry = (avg_entry * (-position) + trade.price * trade.quantity)
                            / (-new_position);
                    } else if new_position < 0.0 {
                        avg_entry = trade.price;
                    }
                    position = new_position;
                }
            }
        }

        pnl
    }
}

fn main() {
    let mut strategy = VolumeStrategy::new(5, 1.8, 1.0);

    // Simulated market data: (timestamp, close, volume)
    let market_data = vec![
        (1, 100.0, 1000.0),
        (2, 100.5, 1100.0),
        (3, 101.0, 1050.0),
        (4, 101.5, 1200.0),
        (5, 102.0, 1150.0),
        (6, 103.0, 2500.0), // Volume spike + uptrend -> BUY
        (7, 103.5, 1300.0),
        (8, 104.0, 1400.0),
        (9, 104.2, 800.0),  // Low volume
        (10, 104.0, 750.0), // Low volume, sideways -> EXIT
        (11, 103.5, 1100.0),
        (12, 102.0, 2800.0), // Volume spike + downtrend -> SELL
        (13, 101.0, 1500.0),
        (14, 100.5, 1200.0),
        (15, 100.8, 600.0),  // Low volume, sideways -> EXIT
    ];

    println!("Volume-Based Trading Strategy");
    println!("{:=<60}", "");

    for (timestamp, close, volume) in market_data {
        if let Some(trade) = strategy.on_bar(timestamp, close, volume) {
            println!(
                "\n[Bar {}] TRADE: {:?} {:.2} @ ${:.2}",
                timestamp, trade.side, trade.quantity, trade.price
            );
            println!("         Reason: {}", trade.reason);
        }
    }

    println!("\n{:=<60}", "");
    println!("Total trades: {}", strategy.get_trades().len());
    println!("P&L: ${:.2}", strategy.calculate_pnl());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Volume Bar | OHLCV data structure for candlestick representation |
| Volume Moving Average | Smoothed volume to identify high/low activity periods |
| On-Balance Volume (OBV) | Cumulative volume indicator showing buying/selling pressure |
| Volume Profile | Distribution of volume across price levels |
| Point of Control (POC) | Price level with the highest traded volume |
| Value Area | Price range containing 70% of volume |
| Volume Spike | Volume significantly above average (signal of interest) |
| Volume-Price Divergence | When price and volume don't confirm each other |

## Homework

1. **VWAP Calculator**: Implement a Volume-Weighted Average Price (VWAP) calculator that:
   - Takes a stream of trades (price, volume, timestamp)
   - Calculates running VWAP throughout the trading day
   - Resets at market open
   - Shows when price is above/below VWAP

2. **Volume Breakout Detector**: Create a system that:
   - Monitors volume in real-time
   - Detects when volume exceeds 2x the 20-period average
   - Classifies breakouts as bullish/bearish based on price action
   - Logs alerts with timestamps and volume ratios

3. **Volume Profile Trading Levels**: Build a `TradingLevelFinder` that:
   - Constructs a volume profile from historical data
   - Identifies High Volume Nodes (HVN) as support/resistance
   - Identifies Low Volume Nodes (LVN) as potential breakout zones
   - Returns a list of key price levels for trading

4. **Multi-Timeframe Volume Analysis**: Implement an analyzer that:
   - Aggregates volume data across multiple timeframes (1m, 5m, 15m, 1h)
   - Detects when volume signals align across timeframes
   - Generates stronger signals when multiple timeframes confirm
   - Uses `Arc<Mutex<>>` for thread-safe shared state across timeframes

## Navigation

[← Previous day](../253-momentum-indicators/en.md) | [Next day →](../255-price-action-patterns/en.md)
