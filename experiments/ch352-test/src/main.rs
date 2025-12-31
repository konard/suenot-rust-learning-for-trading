// Test code from Chapter 352: Publishing to crates.io

use std::collections::HashMap;

// ============================================
// Part 1: Trading Indicator Trait and SMA/EMA
// ============================================

/// Trait for all trading indicators
pub trait TradingIndicator {
    /// Calculates indicator values from prices
    fn calculate(&self, prices: &[f64]) -> Vec<f64>;

    /// Returns the indicator name
    fn name(&self) -> &str;

    /// Minimum number of data points required
    fn min_periods(&self) -> usize;
}

/// Simple Moving Average (SMA)
#[derive(Debug, Clone)]
pub struct SMA {
    period: usize,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        SMA { period }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl TradingIndicator for SMA {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period {
            return vec![];
        }

        prices
            .windows(self.period)
            .map(|window| window.iter().sum::<f64>() / self.period as f64)
            .collect()
    }

    fn name(&self) -> &str {
        "SMA"
    }

    fn min_periods(&self) -> usize {
        self.period
    }
}

/// Exponential Moving Average (EMA)
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA { period, multiplier }
    }

    pub fn period(&self) -> usize {
        self.period
    }
}

impl TradingIndicator for EMA {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period {
            return vec![];
        }

        let mut result = Vec::with_capacity(prices.len() - self.period + 1);

        let first_ema: f64 = prices[..self.period].iter().sum::<f64>()
            / self.period as f64;
        result.push(first_ema);

        let mut prev_ema = first_ema;
        for price in &prices[self.period..] {
            let ema = (price - prev_ema) * self.multiplier + prev_ema;
            result.push(ema);
            prev_ema = ema;
        }

        result
    }

    fn name(&self) -> &str {
        "EMA"
    }

    fn min_periods(&self) -> usize {
        self.period
    }
}

/// RSI indicator
#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        RSI { period }
    }
}

impl TradingIndicator for RSI {
    fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        if prices.len() < self.period + 1 {
            return vec![];
        }

        let changes: Vec<f64> = prices
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        let mut gains: Vec<f64> = Vec::new();
        let mut losses: Vec<f64> = Vec::new();

        for change in &changes {
            if *change > 0.0 {
                gains.push(*change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let mut result = Vec::new();

        let mut avg_gain: f64 = gains[..self.period].iter().sum::<f64>()
            / self.period as f64;
        let mut avg_loss: f64 = losses[..self.period].iter().sum::<f64>()
            / self.period as f64;

        for i in self.period..gains.len() {
            avg_gain = (avg_gain * (self.period - 1) as f64 + gains[i])
                / self.period as f64;
            avg_loss = (avg_loss * (self.period - 1) as f64 + losses[i])
                / self.period as f64;

            let rs = if avg_loss != 0.0 {
                avg_gain / avg_loss
            } else {
                100.0
            };

            result.push(100.0 - (100.0 / (1.0 + rs)));
        }

        result
    }

    fn name(&self) -> &str {
        "RSI"
    }

    fn min_periods(&self) -> usize {
        self.period + 1
    }
}

// ============================================
// Part 2: Trading Strategy
// ============================================

/// Trading signal
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy { price: f64, quantity: f64 },
    Sell { price: f64, quantity: f64 },
    Hold,
}

/// Trait for trading strategies
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn generate_signal(&self, data: &MarketData) -> Signal;
    fn parameters(&self) -> HashMap<String, f64>;
}

/// Market data
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub prices: Vec<f64>,
    pub volumes: Vec<f64>,
    pub timestamps: Vec<i64>,
}

impl MarketData {
    pub fn new(symbol: &str) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            prices: Vec::new(),
            volumes: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    pub fn add_candle(&mut self, price: f64, volume: f64, timestamp: i64) {
        self.prices.push(price);
        self.volumes.push(volume);
        self.timestamps.push(timestamp);
    }

    pub fn last_price(&self) -> Option<f64> {
        self.prices.last().copied()
    }
}

/// MA Crossover Strategy
#[derive(Debug, Clone)]
pub struct CrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
}

impl CrossoverStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        CrossoverStrategy { fast_period, slow_period }
    }

    fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }
}

impl Strategy for CrossoverStrategy {
    fn name(&self) -> &str {
        "MA Crossover"
    }

    fn generate_signal(&self, data: &MarketData) -> Signal {
        let fast_ma = match self.calculate_sma(&data.prices, self.fast_period) {
            Some(ma) => ma,
            None => return Signal::Hold,
        };

        let slow_ma = match self.calculate_sma(&data.prices, self.slow_period) {
            Some(ma) => ma,
            None => return Signal::Hold,
        };

        let current_price = match data.last_price() {
            Some(p) => p,
            None => return Signal::Hold,
        };

        if fast_ma > slow_ma {
            Signal::Buy {
                price: current_price,
                quantity: 1.0,
            }
        } else if fast_ma < slow_ma {
            Signal::Sell {
                price: current_price,
                quantity: 1.0,
            }
        } else {
            Signal::Hold
        }
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("fast_period".to_string(), self.fast_period as f64);
        params.insert("slow_period".to_string(), self.slow_period as f64);
        params
    }
}

/// Strategy manager
pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn new() -> Self {
        StrategyManager {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub fn generate_signals(&self, data: &MarketData) -> Vec<(&str, Signal)> {
        self.strategies
            .iter()
            .map(|s| (s.name(), s.generate_signal(data)))
            .collect()
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Main function to test everything
// ============================================

fn main() {
    println!("=== Testing Chapter 352 Code Examples ===\n");

    // Test indicators
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0];

    let sma = SMA::new(3);
    let sma_values = sma.calculate(&prices);
    println!("SMA(3): {:?}", sma_values);
    assert!(!sma_values.is_empty(), "SMA should produce values");

    let ema = EMA::new(3);
    let ema_values = ema.calculate(&prices);
    println!("EMA(3): {:?}", ema_values);
    assert!(!ema_values.is_empty(), "EMA should produce values");

    // Test RSI with more data
    let rsi_prices: Vec<f64> = (0..30).map(|i| 50000.0 + (i as f64 * 100.0)).collect();
    let rsi = RSI::new(14);
    let rsi_values = rsi.calculate(&rsi_prices);
    println!("RSI(14) values count: {}", rsi_values.len());

    // Test strategy
    let mut data = MarketData::new("BTCUSDT");
    for i in 0..50 {
        let price = 50000.0 + (i as f64 * 50.0);
        data.add_candle(price, 1000.0, i);
    }

    let strategy = CrossoverStrategy::new(5, 20);
    println!("\nStrategy: {}", strategy.name());
    println!("Parameters: {:?}", strategy.parameters());

    let signal = strategy.generate_signal(&data);
    println!("Signal: {:?}", signal);

    // Test strategy manager
    let mut manager = StrategyManager::new();
    manager.add_strategy(Box::new(CrossoverStrategy::new(5, 20)));
    manager.add_strategy(Box::new(CrossoverStrategy::new(10, 30)));

    println!("\n=== Signals from All Strategies ===");
    for (name, signal) in manager.generate_signals(&data) {
        println!("{}: {:?}", name, signal);
    }

    println!("\n=== All tests passed! ===");
}
