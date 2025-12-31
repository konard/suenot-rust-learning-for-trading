# Day 255: VWAP: Volume Weighted Average Price

## Trading Analogy

Imagine you're a large institutional trader who needs to buy 100,000 shares of a stock throughout the day. You don't want to move the market by placing one giant order, so you spread your purchases across the trading session. At the end of the day, how do you know if you got a good price?

This is where **VWAP (Volume Weighted Average Price)** comes in. VWAP is like a "fair price" benchmark that tells you what the average price was, weighted by how much trading volume occurred at each price level. If you bought below VWAP, you got a better deal than average. If you bought above VWAP, you paid a premium.

In algorithmic trading, VWAP is crucial for:
- **Execution benchmarking** — Did your algorithm buy/sell at fair prices?
- **Order execution strategies** — VWAP algorithms aim to match or beat this benchmark
- **Trend identification** — Price above VWAP suggests bullish pressure, below suggests bearish
- **Risk management** — Detecting if you're paying too much for large orders

## What is VWAP?

VWAP is calculated using this formula:

```
VWAP = Σ(Price × Volume) / Σ(Volume)
```

Or in more detail:
```
VWAP = (P₁×V₁ + P₂×V₂ + ... + Pₙ×Vₙ) / (V₁ + V₂ + ... + Vₙ)
```

Where:
- `P` = Price (typically the typical price: (High + Low + Close) / 3)
- `V` = Volume at that price level

## Basic VWAP Implementation

```rust
/// Represents a single trade or candle
#[derive(Debug, Clone)]
struct PriceData {
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl PriceData {
    fn new(high: f64, low: f64, close: f64, volume: f64) -> Self {
        PriceData { high, low, close, volume }
    }

    /// Calculate typical price (average of high, low, close)
    fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// VWAP Calculator that accumulates data throughout the session
#[derive(Debug)]
struct VwapCalculator {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
}

impl VwapCalculator {
    fn new() -> Self {
        VwapCalculator {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
        }
    }

    /// Add new price data and return updated VWAP
    fn add(&mut self, data: &PriceData) -> f64 {
        let typical_price = data.typical_price();

        self.cumulative_volume += data.volume;
        self.cumulative_price_volume += typical_price * data.volume;

        self.calculate()
    }

    /// Calculate current VWAP
    fn calculate(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    /// Reset calculator for new trading session
    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_price_volume = 0.0;
    }
}

fn main() {
    let mut vwap_calc = VwapCalculator::new();

    // Simulated intraday trading data
    let trading_data = vec![
        PriceData::new(100.50, 99.80, 100.20, 10000.0),
        PriceData::new(100.30, 99.90, 100.10, 15000.0),
        PriceData::new(100.80, 100.00, 100.60, 20000.0),
        PriceData::new(101.20, 100.40, 101.00, 25000.0),
        PriceData::new(101.50, 100.80, 101.20, 18000.0),
    ];

    println!("=== VWAP Calculation Throughout the Day ===\n");

    for (i, data) in trading_data.iter().enumerate() {
        let vwap = vwap_calc.add(data);
        let typical = data.typical_price();

        let position = if data.close > vwap {
            "above"
        } else if data.close < vwap {
            "below"
        } else {
            "at"
        };

        println!(
            "Period {}: Close=${:.2}, Typical=${:.2}, Volume={:.0}, VWAP=${:.4} (Price is {} VWAP)",
            i + 1, data.close, typical, data.volume, vwap, position
        );
    }

    println!("\nFinal VWAP: ${:.4}", vwap_calc.calculate());
}
```

## VWAP with Standard Deviation Bands

Professional traders often use VWAP with standard deviation bands to identify overbought/oversold conditions:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct PriceData {
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl PriceData {
    fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// VWAP with standard deviation bands
#[derive(Debug)]
struct VwapWithBands {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
    price_data: VecDeque<(f64, f64)>, // (typical_price, volume)
    band_multiplier: f64,
}

impl VwapWithBands {
    fn new(band_multiplier: f64) -> Self {
        VwapWithBands {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
            price_data: VecDeque::new(),
            band_multiplier,
        }
    }

    fn add(&mut self, data: &PriceData) {
        let typical_price = data.typical_price();

        self.cumulative_volume += data.volume;
        self.cumulative_price_volume += typical_price * data.volume;
        self.price_data.push_back((typical_price, data.volume));
    }

    fn vwap(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    /// Calculate volume-weighted standard deviation
    fn std_deviation(&self) -> f64 {
        if self.cumulative_volume == 0.0 || self.price_data.is_empty() {
            return 0.0;
        }

        let vwap = self.vwap();
        let mut weighted_variance_sum = 0.0;

        for (price, volume) in &self.price_data {
            let deviation = price - vwap;
            weighted_variance_sum += deviation * deviation * volume;
        }

        (weighted_variance_sum / self.cumulative_volume).sqrt()
    }

    fn upper_band(&self) -> f64 {
        self.vwap() + self.band_multiplier * self.std_deviation()
    }

    fn lower_band(&self) -> f64 {
        self.vwap() - self.band_multiplier * self.std_deviation()
    }

    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_price_volume = 0.0;
        self.price_data.clear();
    }
}

fn main() {
    // Create VWAP with 2 standard deviation bands
    let mut vwap = VwapWithBands::new(2.0);

    let trading_data = vec![
        PriceData { high: 100.50, low: 99.80, close: 100.20, volume: 10000.0 },
        PriceData { high: 100.30, low: 99.90, close: 100.10, volume: 15000.0 },
        PriceData { high: 100.80, low: 100.00, close: 100.60, volume: 20000.0 },
        PriceData { high: 101.20, low: 100.40, close: 101.00, volume: 25000.0 },
        PriceData { high: 101.50, low: 100.80, close: 101.20, volume: 18000.0 },
        PriceData { high: 101.80, low: 101.00, close: 101.50, volume: 22000.0 },
        PriceData { high: 102.20, low: 101.20, close: 101.80, volume: 30000.0 },
    ];

    println!("=== VWAP with Standard Deviation Bands ===\n");

    for (i, data) in trading_data.iter().enumerate() {
        vwap.add(data);

        let current_vwap = vwap.vwap();
        let upper = vwap.upper_band();
        let lower = vwap.lower_band();

        let signal = if data.close > upper {
            "OVERBOUGHT - Consider selling"
        } else if data.close < lower {
            "OVERSOLD - Consider buying"
        } else {
            "NEUTRAL"
        };

        println!("Period {}:", i + 1);
        println!("  Close: ${:.2}", data.close);
        println!("  VWAP:  ${:.4}", current_vwap);
        println!("  Upper Band (+2σ): ${:.4}", upper);
        println!("  Lower Band (-2σ): ${:.4}", lower);
        println!("  Signal: {}\n", signal);
    }
}
```

## VWAP Execution Algorithm

One of the most common uses of VWAP in algorithmic trading is as an execution benchmark. Here's a simple VWAP execution algorithm:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    executed_quantity: f64,
    average_price: f64,
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    typical_volume_profile: Vec<f64>, // Expected volume for each period
}

/// VWAP Execution Algorithm
/// Aims to execute orders in line with expected volume distribution
#[derive(Debug)]
struct VwapExecutor {
    target_quantity: f64,
    executed_quantity: f64,
    total_cost: f64,
    current_period: usize,
    volume_profile: Vec<f64>,
    order_id_counter: u64,
    orders: Vec<Order>,
}

impl VwapExecutor {
    fn new(target_quantity: f64, volume_profile: Vec<f64>) -> Self {
        VwapExecutor {
            target_quantity,
            executed_quantity: 0.0,
            total_cost: 0.0,
            current_period: 0,
            volume_profile,
            order_id_counter: 0,
            orders: Vec::new(),
        }
    }

    /// Calculate how much to trade in current period based on volume profile
    fn calculate_period_quantity(&self) -> f64 {
        if self.current_period >= self.volume_profile.len() {
            return 0.0;
        }

        let remaining_quantity = self.target_quantity - self.executed_quantity;
        let total_profile: f64 = self.volume_profile.iter().sum();
        let remaining_profile: f64 = self.volume_profile[self.current_period..].iter().sum();

        if remaining_profile == 0.0 {
            return remaining_quantity;
        }

        // Proportional allocation based on volume profile
        let period_weight = self.volume_profile[self.current_period] / remaining_profile;
        remaining_quantity * period_weight
    }

    /// Execute for current period
    fn execute_period(&mut self, market_price: f64) -> Option<Order> {
        let quantity = self.calculate_period_quantity();

        if quantity <= 0.0 {
            self.current_period += 1;
            return None;
        }

        self.order_id_counter += 1;
        let order = Order {
            id: self.order_id_counter,
            symbol: "BTC".to_string(),
            quantity,
            executed_quantity: quantity,
            average_price: market_price,
        };

        self.executed_quantity += quantity;
        self.total_cost += quantity * market_price;
        self.orders.push(order.clone());
        self.current_period += 1;

        Some(order)
    }

    /// Get average execution price
    fn average_execution_price(&self) -> f64 {
        if self.executed_quantity == 0.0 {
            return 0.0;
        }
        self.total_cost / self.executed_quantity
    }

    /// Check execution quality vs VWAP
    fn execution_quality(&self, market_vwap: f64) -> f64 {
        // Positive means we did better than VWAP (for buys, lower is better)
        market_vwap - self.average_execution_price()
    }

    fn progress(&self) -> f64 {
        (self.executed_quantity / self.target_quantity) * 100.0
    }
}

fn main() {
    // Volume profile: expected volume distribution throughout the day
    // Higher values = more volume expected in that period
    let volume_profile = vec![
        0.15, // Morning open - high volume
        0.10, // Mid-morning
        0.08, // Late morning
        0.05, // Lunch - low volume
        0.05, // Early afternoon
        0.07, // Mid-afternoon
        0.10, // Late afternoon
        0.15, // Approaching close
        0.25, // Closing auction - highest volume
    ];

    let mut executor = VwapExecutor::new(1000.0, volume_profile);

    // Simulated market prices for each period
    let market_prices = vec![
        42000.0, 42150.0, 42100.0, 42050.0, 42200.0,
        42300.0, 42250.0, 42400.0, 42350.0,
    ];

    // Calculate market VWAP (what we're trying to match)
    let market_volumes = vec![
        15000.0, 10000.0, 8000.0, 5000.0, 5000.0,
        7000.0, 10000.0, 15000.0, 25000.0,
    ];

    let total_volume: f64 = market_volumes.iter().sum();
    let market_vwap: f64 = market_prices.iter()
        .zip(market_volumes.iter())
        .map(|(p, v)| p * v)
        .sum::<f64>() / total_volume;

    println!("=== VWAP Execution Algorithm ===\n");
    println!("Target: Buy 1000 BTC throughout the day");
    println!("Market VWAP: ${:.2}\n", market_vwap);

    for (i, price) in market_prices.iter().enumerate() {
        if let Some(order) = executor.execute_period(*price) {
            println!(
                "Period {}: Executed {:.2} BTC @ ${:.2} (Progress: {:.1}%)",
                i + 1, order.quantity, order.average_price, executor.progress()
            );
        }
    }

    println!("\n=== Execution Summary ===");
    println!("Total Executed: {:.2} BTC", executor.executed_quantity);
    println!("Average Price: ${:.2}", executor.average_execution_price());
    println!("Market VWAP: ${:.2}", market_vwap);

    let quality = executor.execution_quality(market_vwap);
    if quality > 0.0 {
        println!("Result: OUTPERFORMED VWAP by ${:.2} per unit!", quality);
    } else {
        println!("Result: Underperformed VWAP by ${:.2} per unit", -quality);
    }
}
```

## Real-Time VWAP Trading Signal Generator

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TradingSignal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

impl TradingSignal {
    fn as_str(&self) -> &'static str {
        match self {
            TradingSignal::StrongBuy => "STRONG BUY",
            TradingSignal::Buy => "BUY",
            TradingSignal::Neutral => "NEUTRAL",
            TradingSignal::Sell => "SELL",
            TradingSignal::StrongSell => "STRONG SELL",
        }
    }
}

#[derive(Debug)]
struct VwapSignalGenerator {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
    squared_deviation_sum: f64,
    data_points: Vec<f64>,
    signal_threshold: f64,
}

impl VwapSignalGenerator {
    fn new(signal_threshold: f64) -> Self {
        VwapSignalGenerator {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
            squared_deviation_sum: 0.0,
            data_points: Vec::new(),
            signal_threshold,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.cumulative_volume += volume;
        self.cumulative_price_volume += price * volume;
        self.data_points.push(price);

        // Update squared deviation for std calculation
        let vwap = self.vwap();
        self.squared_deviation_sum += (price - vwap).powi(2) * volume;
    }

    fn vwap(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    fn std_deviation(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        (self.squared_deviation_sum / self.cumulative_volume).sqrt()
    }

    fn generate_signal(&self, current_price: f64) -> TradingSignal {
        let vwap = self.vwap();
        let std = self.std_deviation();

        if std == 0.0 || vwap == 0.0 {
            return TradingSignal::Neutral;
        }

        let deviation = (current_price - vwap) / std;

        if deviation < -2.0 * self.signal_threshold {
            TradingSignal::StrongBuy
        } else if deviation < -self.signal_threshold {
            TradingSignal::Buy
        } else if deviation > 2.0 * self.signal_threshold {
            TradingSignal::StrongSell
        } else if deviation > self.signal_threshold {
            TradingSignal::Sell
        } else {
            TradingSignal::Neutral
        }
    }

    fn distance_from_vwap(&self, current_price: f64) -> f64 {
        let vwap = self.vwap();
        if vwap == 0.0 {
            return 0.0;
        }
        ((current_price - vwap) / vwap) * 100.0
    }
}

fn main() {
    let mut signal_gen = VwapSignalGenerator::new(1.0);

    // Simulated real-time price feed
    let price_feed = vec![
        (100.0, 1000.0),  // (price, volume)
        (100.5, 1500.0),
        (101.0, 2000.0),
        (100.8, 1800.0),
        (101.5, 2200.0),
        (102.0, 2500.0),
        (103.5, 3000.0),  // Big move up
        (104.0, 3500.0),
        (103.0, 2000.0),
        (102.5, 1500.0),
        (99.0, 4000.0),   // Sharp drop
        (98.5, 5000.0),
        (99.5, 2500.0),
    ];

    println!("=== Real-Time VWAP Signal Generator ===\n");

    for (i, (price, volume)) in price_feed.iter().enumerate() {
        signal_gen.update(*price, *volume);

        let signal = signal_gen.generate_signal(*price);
        let vwap = signal_gen.vwap();
        let distance = signal_gen.distance_from_vwap(*price);

        println!("Tick {}: Price=${:.2}, Volume={:.0}", i + 1, price, volume);
        println!("  VWAP: ${:.4}, Distance: {:+.2}%", vwap, distance);
        println!("  Signal: {}\n", signal.as_str());
    }
}
```

## Portfolio VWAP Analysis

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    quantity: f64,
    price: f64,
    volume: f64,
}

#[derive(Debug)]
struct PortfolioVwapAnalyzer {
    trades: HashMap<String, Vec<Trade>>,
    market_vwap: HashMap<String, f64>,
}

impl PortfolioVwapAnalyzer {
    fn new() -> Self {
        PortfolioVwapAnalyzer {
            trades: HashMap::new(),
            market_vwap: HashMap::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades
            .entry(trade.symbol.clone())
            .or_insert_with(Vec::new)
            .push(trade);
    }

    fn set_market_vwap(&mut self, symbol: &str, vwap: f64) {
        self.market_vwap.insert(symbol.to_string(), vwap);
    }

    /// Calculate execution VWAP for a symbol
    fn execution_vwap(&self, symbol: &str) -> Option<f64> {
        let trades = self.trades.get(symbol)?;

        if trades.is_empty() {
            return None;
        }

        let total_cost: f64 = trades.iter()
            .map(|t| t.price * t.quantity)
            .sum();
        let total_quantity: f64 = trades.iter()
            .map(|t| t.quantity)
            .sum();

        Some(total_cost / total_quantity)
    }

    /// Calculate execution quality (slippage vs VWAP)
    fn slippage(&self, symbol: &str) -> Option<f64> {
        let exec_vwap = self.execution_vwap(symbol)?;
        let market_vwap = self.market_vwap.get(symbol)?;

        // Positive = worse execution (paid more for buys)
        Some(exec_vwap - market_vwap)
    }

    /// Calculate slippage in basis points
    fn slippage_bps(&self, symbol: &str) -> Option<f64> {
        let slippage = self.slippage(symbol)?;
        let market_vwap = self.market_vwap.get(symbol)?;

        Some((slippage / market_vwap) * 10000.0)
    }

    fn analyze_all(&self) {
        println!("=== Portfolio VWAP Analysis ===\n");

        for symbol in self.trades.keys() {
            let trades = &self.trades[symbol];
            let total_qty: f64 = trades.iter().map(|t| t.quantity).sum();
            let exec_vwap = self.execution_vwap(symbol).unwrap_or(0.0);
            let market_vwap = self.market_vwap.get(symbol).copied().unwrap_or(0.0);
            let slippage_bps = self.slippage_bps(symbol).unwrap_or(0.0);

            println!("Symbol: {}", symbol);
            println!("  Total Trades: {}", trades.len());
            println!("  Total Quantity: {:.4}", total_qty);
            println!("  Execution VWAP: ${:.4}", exec_vwap);
            println!("  Market VWAP: ${:.4}", market_vwap);
            println!("  Slippage: {:.2} bps", slippage_bps);

            if slippage_bps > 0.0 {
                println!("  Assessment: UNDERPERFORMED (paid premium)\n");
            } else if slippage_bps < 0.0 {
                println!("  Assessment: OUTPERFORMED (got discount)\n");
            } else {
                println!("  Assessment: MATCHED VWAP exactly\n");
            }
        }
    }
}

fn main() {
    let mut analyzer = PortfolioVwapAnalyzer::new();

    // Add trades
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        price: 42100.0,
        volume: 1000.0,
    });
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.3,
        price: 42050.0,
        volume: 800.0,
    });
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.2,
        price: 42200.0,
        volume: 500.0,
    });

    analyzer.add_trade(Trade {
        symbol: "ETH".to_string(),
        quantity: 5.0,
        price: 2250.0,
        volume: 10000.0,
    });
    analyzer.add_trade(Trade {
        symbol: "ETH".to_string(),
        quantity: 3.0,
        price: 2230.0,
        volume: 6000.0,
    });

    // Set market VWAP for comparison
    analyzer.set_market_vwap("BTC", 42150.0);
    analyzer.set_market_vwap("ETH", 2240.0);

    analyzer.analyze_all();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| VWAP | Volume Weighted Average Price — benchmark for fair price |
| Typical Price | (High + Low + Close) / 3 — used in VWAP calculation |
| VWAP Bands | Standard deviation bands around VWAP for overbought/oversold signals |
| VWAP Execution | Algorithm that executes orders in line with expected volume distribution |
| Slippage | Difference between execution price and benchmark (VWAP) |
| Basis Points (bps) | 1 bps = 0.01% — used to measure execution quality |

## Homework

1. **Anchored VWAP**: Implement an anchored VWAP calculator that allows you to start the VWAP calculation from any point (e.g., from a significant price level or event). Add a method to reset the anchor point.

2. **Multi-Timeframe VWAP**: Create a system that calculates VWAP across multiple timeframes (1-minute, 5-minute, 1-hour) simultaneously. Compare how signals differ across timeframes.

3. **VWAP Crossover Strategy**: Implement a trading strategy that:
   - Generates BUY signals when price crosses above VWAP with increasing volume
   - Generates SELL signals when price crosses below VWAP with increasing volume
   - Tracks trade performance and calculates Sharpe ratio

4. **VWAP Participation Algorithm**: Build an advanced VWAP execution algorithm that:
   - Adjusts participation rate based on market volatility
   - Implements "catch-up" logic if falling behind target
   - Includes slippage limits and circuit breakers
   - Reports real-time execution quality metrics

## Navigation

[← Previous day](../254-twap-time-weighted-average-price/en.md) | [Next day →](../256-ema-exponential-moving-average/en.md)
