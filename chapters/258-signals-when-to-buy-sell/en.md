# Day 258: Signals: When to Buy/Sell

## Trading Analogy

Imagine you're standing in front of a screen showing live market quotes. Prices are constantly changing — up, down, sideways. How do you know when exactly to act? Experienced traders use **signals** — specific conditions or combinations of factors that indicate a favorable moment to buy or sell.

A trading signal is like a traffic light at an intersection: green light means "buy", red means "sell", yellow means "wait and observe". In algorithmic trading, we program these "traffic lights" to automatically analyze the market and generate signals.

In this chapter, we'll learn to:
- Model trading signals in Rust
- Create signal generators based on indicators
- Manage orders based on signals
- Account for risk in decision-making

## What is a Trading Signal?

A trading signal is the result of market data analysis, indicating a potential trading opportunity. In Rust, we can model signals using enums:

```rust
/// Trading signal type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Strong buy signal
    StrongBuy,
    /// Weak buy signal
    Buy,
    /// Neutral position, wait
    Hold,
    /// Weak sell signal
    Sell,
    /// Strong sell signal
    StrongSell,
}

impl Signal {
    /// Returns signal strength from -2 to 2
    pub fn strength(&self) -> i32 {
        match self {
            Signal::StrongBuy => 2,
            Signal::Buy => 1,
            Signal::Hold => 0,
            Signal::Sell => -1,
            Signal::StrongSell => -2,
        }
    }

    /// Checks if signal is bullish (buy)
    pub fn is_bullish(&self) -> bool {
        matches!(self, Signal::StrongBuy | Signal::Buy)
    }

    /// Checks if signal is bearish (sell)
    pub fn is_bearish(&self) -> bool {
        matches!(self, Signal::Sell | Signal::StrongSell)
    }
}
```

## Trading Signal Structure

A simple enum isn't enough for real trading. We need a more complete structure:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

/// Complete trading signal information
#[derive(Debug, Clone)]
pub struct TradingSignal {
    /// Instrument ticker
    pub symbol: String,
    /// Signal type
    pub signal: Signal,
    /// Signal strength from 0.0 to 1.0
    pub confidence: f64,
    /// Recommended entry price
    pub entry_price: f64,
    /// Stop-loss level
    pub stop_loss: Option<f64>,
    /// Take-profit level
    pub take_profit: Option<f64>,
    /// Signal generation timestamp
    pub timestamp: u64,
    /// Signal source (which indicator/strategy)
    pub source: String,
}

impl TradingSignal {
    pub fn new(
        symbol: &str,
        signal: Signal,
        confidence: f64,
        entry_price: f64,
        source: &str,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        TradingSignal {
            symbol: symbol.to_string(),
            signal,
            confidence: confidence.clamp(0.0, 1.0),
            entry_price,
            stop_loss: None,
            take_profit: None,
            timestamp,
            source: source.to_string(),
        }
    }

    /// Adds risk management levels
    pub fn with_risk_levels(mut self, stop_loss: f64, take_profit: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self.take_profit = Some(take_profit);
        self
    }

    /// Calculates risk/reward ratio
    pub fn risk_reward_ratio(&self) -> Option<f64> {
        match (self.stop_loss, self.take_profit) {
            (Some(sl), Some(tp)) => {
                let risk = (self.entry_price - sl).abs();
                let reward = (tp - self.entry_price).abs();
                if risk > 0.0 {
                    Some(reward / risk)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
```

## Indicator-Based Signal Generator

Now let's create a signal generator that analyzes prices and provides trading recommendations:

```rust
/// Candle data (OHLCV)
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Signal generator based on moving average crossover
pub struct MovingAverageCrossover {
    fast_period: usize,
    slow_period: usize,
}

impl MovingAverageCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        MovingAverageCrossover { fast_period, slow_period }
    }

    /// Calculates simple moving average
    fn sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    /// Generates signal based on moving average crossover
    pub fn generate_signal(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal> {
        if candles.len() < self.slow_period + 1 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let current_close = *closes.last()?;

        // Current MA values
        let fast_ma = Self::sma(&closes, self.fast_period)?;
        let slow_ma = Self::sma(&closes, self.slow_period)?;

        // Previous MA values
        let prev_closes: Vec<f64> = closes[..closes.len() - 1].to_vec();
        let prev_fast_ma = Self::sma(&prev_closes, self.fast_period)?;
        let prev_slow_ma = Self::sma(&prev_closes, self.slow_period)?;

        // Determine crossover
        let signal = if prev_fast_ma <= prev_slow_ma && fast_ma > slow_ma {
            // Golden cross — buy signal
            Signal::Buy
        } else if prev_fast_ma >= prev_slow_ma && fast_ma < slow_ma {
            // Death cross — sell signal
            Signal::Sell
        } else {
            Signal::Hold
        };

        if signal == Signal::Hold {
            return None;
        }

        // Calculate confidence based on MA divergence
        let divergence = ((fast_ma - slow_ma) / slow_ma).abs();
        let confidence = (divergence * 100.0).min(1.0);

        let mut trading_signal = TradingSignal::new(
            symbol,
            signal,
            confidence,
            current_close,
            "MA_Crossover",
        );

        // Add stop-loss and take-profit levels
        let atr = self.calculate_atr(candles, 14)?;
        if signal.is_bullish() {
            trading_signal = trading_signal.with_risk_levels(
                current_close - atr * 2.0,  // Stop-loss
                current_close + atr * 3.0,  // Take-profit
            );
        } else {
            trading_signal = trading_signal.with_risk_levels(
                current_close + atr * 2.0,  // Stop-loss
                current_close - atr * 3.0,  // Take-profit
            );
        }

        Some(trading_signal)
    }

    /// Calculates Average True Range
    fn calculate_atr(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period + 1 {
            return None;
        }

        let mut tr_values = Vec::new();
        for i in 1..candles.len() {
            let high = candles[i].high;
            let low = candles[i].low;
            let prev_close = candles[i - 1].close;

            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            tr_values.push(tr);
        }

        let recent_tr: Vec<f64> = tr_values.iter().rev().take(period).copied().collect();
        Some(recent_tr.iter().sum::<f64>() / period as f64)
    }
}
```

## Signal Aggregator

In real trading, multiple indicators are often used simultaneously. Let's create an aggregator that combines signals:

```rust
use std::collections::HashMap;

/// Trait for signal generators
pub trait SignalGenerator {
    fn generate(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal>;
    fn name(&self) -> &str;
}

/// Aggregator for signals from multiple sources
pub struct SignalAggregator {
    generators: Vec<Box<dyn SignalGenerator>>,
    weights: HashMap<String, f64>,
}

impl SignalAggregator {
    pub fn new() -> Self {
        SignalAggregator {
            generators: Vec::new(),
            weights: HashMap::new(),
        }
    }

    /// Adds a signal generator with weight
    pub fn add_generator(&mut self, generator: Box<dyn SignalGenerator>, weight: f64) {
        let name = generator.name().to_string();
        self.generators.push(generator);
        self.weights.insert(name, weight);
    }

    /// Generates an aggregated signal
    pub fn generate_signal(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        let mut signals = Vec::new();

        for generator in &self.generators {
            if let Some(signal) = generator.generate(candles, symbol) {
                let weight = self.weights.get(generator.name()).copied().unwrap_or(1.0);
                let score = signal.signal.strength() as f64 * signal.confidence * weight;
                total_score += score;
                total_weight += weight;
                signals.push(signal);
            }
        }

        if signals.is_empty() || total_weight == 0.0 {
            return None;
        }

        let avg_score = total_score / total_weight;

        // Determine final signal based on average score
        let final_signal = if avg_score > 1.5 {
            Signal::StrongBuy
        } else if avg_score > 0.5 {
            Signal::Buy
        } else if avg_score < -1.5 {
            Signal::StrongSell
        } else if avg_score < -0.5 {
            Signal::Sell
        } else {
            Signal::Hold
        };

        if final_signal == Signal::Hold {
            return None;
        }

        // Average signal parameters
        let avg_entry = signals.iter().map(|s| s.entry_price).sum::<f64>() / signals.len() as f64;
        let confidence = (avg_score.abs() / 2.0).min(1.0);

        Some(TradingSignal::new(
            symbol,
            final_signal,
            confidence,
            avg_entry,
            "Aggregated",
        ))
    }
}
```

## Signal-Based Order Management

Now let's create a system that converts signals into actual orders:

```rust
/// Order type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

/// Order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: OrderStatus,
}

/// Portfolio position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub unrealized_pnl: f64,
}

/// Signal-based order manager
pub struct SignalOrderManager {
    next_order_id: u64,
    max_position_size: f64,
    max_risk_per_trade: f64,  // Percentage of portfolio
    portfolio_value: f64,
    positions: HashMap<String, Position>,
    pending_orders: Vec<Order>,
}

impl SignalOrderManager {
    pub fn new(portfolio_value: f64, max_risk_per_trade: f64) -> Self {
        SignalOrderManager {
            next_order_id: 1,
            max_position_size: portfolio_value * 0.1,  // Max 10% per position
            max_risk_per_trade,
            portfolio_value,
            positions: HashMap::new(),
            pending_orders: Vec::new(),
        }
    }

    /// Processes a trading signal
    pub fn process_signal(&mut self, signal: &TradingSignal) -> Option<Order> {
        // Check minimum confidence
        if signal.confidence < 0.3 {
            println!("Signal rejected: low confidence ({:.2})", signal.confidence);
            return None;
        }

        // Check risk/reward ratio
        if let Some(rr) = signal.risk_reward_ratio() {
            if rr < 1.5 {
                println!("Signal rejected: low R/R ({:.2})", rr);
                return None;
            }
        }

        // Determine order side
        let side = if signal.signal.is_bullish() {
            OrderSide::Buy
        } else if signal.signal.is_bearish() {
            OrderSide::Sell
        } else {
            return None;
        };

        // Calculate position size based on risk
        let quantity = self.calculate_position_size(signal, side);
        if quantity <= 0.0 {
            println!("Signal rejected: position size is zero");
            return None;
        }

        let order = Order {
            id: self.next_order_id,
            symbol: signal.symbol.clone(),
            side,
            quantity,
            price: signal.entry_price,
            stop_loss: signal.stop_loss,
            take_profit: signal.take_profit,
            status: OrderStatus::Pending,
        };

        self.next_order_id += 1;
        self.pending_orders.push(order.clone());

        println!(
            "Order #{} created: {:?} {} {} @ {:.2}",
            order.id, order.side, order.quantity, order.symbol, order.price
        );

        Some(order)
    }

    /// Calculates position size based on risk
    fn calculate_position_size(&self, signal: &TradingSignal, side: OrderSide) -> f64 {
        let stop_loss = match signal.stop_loss {
            Some(sl) => sl,
            None => return 0.0,  // Don't trade without stop-loss
        };

        // Risk amount in money
        let risk_amount = self.portfolio_value * (self.max_risk_per_trade / 100.0);

        // Risk per unit of asset
        let risk_per_unit = match side {
            OrderSide::Buy => (signal.entry_price - stop_loss).abs(),
            OrderSide::Sell => (stop_loss - signal.entry_price).abs(),
        };

        if risk_per_unit <= 0.0 {
            return 0.0;
        }

        // Position size
        let position_size = risk_amount / risk_per_unit;

        // Limit to maximum position size
        let max_quantity = self.max_position_size / signal.entry_price;
        position_size.min(max_quantity)
    }

    /// Updates unrealized profit/loss
    pub fn update_pnl(&mut self, symbol: &str, current_price: f64) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.unrealized_pnl =
                (current_price - position.avg_entry_price) * position.quantity;
        }
    }

    /// Gets total portfolio P&L
    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }
}
```

## RSI Signal Generator

Let's add another popular indicator — RSI (Relative Strength Index):

```rust
/// RSI-based signal generator
pub struct RSISignalGenerator {
    period: usize,
    overbought: f64,
    oversold: f64,
}

impl RSISignalGenerator {
    pub fn new(period: usize, overbought: f64, oversold: f64) -> Self {
        RSISignalGenerator {
            period,
            overbought,
            oversold,
        }
    }

    /// Calculates RSI
    fn calculate_rsi(&self, candles: &[Candle]) -> Option<f64> {
        if candles.len() < self.period + 1 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..candles.len() {
            let change = candles[i].close - candles[i - 1].close;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }

        let recent_gains: Vec<f64> = gains.iter().rev().take(self.period).copied().collect();
        let recent_losses: Vec<f64> = losses.iter().rev().take(self.period).copied().collect();

        let avg_gain: f64 = recent_gains.iter().sum::<f64>() / self.period as f64;
        let avg_loss: f64 = recent_losses.iter().sum::<f64>() / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }
}

impl SignalGenerator for RSISignalGenerator {
    fn generate(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal> {
        let rsi = self.calculate_rsi(candles)?;
        let current_price = candles.last()?.close;

        let signal = if rsi < self.oversold {
            // Oversold — buy signal
            Signal::Buy
        } else if rsi > self.overbought {
            // Overbought — sell signal
            Signal::Sell
        } else {
            return None;
        };

        // Confidence depends on deviation strength
        let confidence = if signal == Signal::Buy {
            (self.oversold - rsi) / self.oversold
        } else {
            (rsi - self.overbought) / (100.0 - self.overbought)
        };

        Some(TradingSignal::new(
            symbol,
            signal,
            confidence.abs().min(1.0),
            current_price,
            "RSI",
        ))
    }

    fn name(&self) -> &str {
        "RSI"
    }
}
```

## Practical Example: Complete Signal System

```rust
use std::collections::VecDeque;

/// Complete signal generation and execution system
pub struct TradingSystem {
    candle_buffer: HashMap<String, VecDeque<Candle>>,
    buffer_size: usize,
    aggregator: SignalAggregator,
    order_manager: SignalOrderManager,
    signal_history: Vec<TradingSignal>,
}

impl TradingSystem {
    pub fn new(portfolio_value: f64) -> Self {
        let mut aggregator = SignalAggregator::new();

        // Add signal generators
        aggregator.add_generator(
            Box::new(MovingAverageCrossoverWrapper::new(10, 30)),
            1.0,
        );
        aggregator.add_generator(
            Box::new(RSISignalGenerator::new(14, 70.0, 30.0)),
            0.8,
        );

        TradingSystem {
            candle_buffer: HashMap::new(),
            buffer_size: 100,
            aggregator,
            order_manager: SignalOrderManager::new(portfolio_value, 2.0),
            signal_history: Vec::new(),
        }
    }

    /// Processes a new candle
    pub fn on_candle(&mut self, symbol: &str, candle: Candle) -> Option<Order> {
        // Add candle to buffer
        let buffer = self.candle_buffer
            .entry(symbol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.buffer_size));

        if buffer.len() >= self.buffer_size {
            buffer.pop_front();
        }
        buffer.push_back(candle);

        // Generate signal
        let candles: Vec<Candle> = buffer.iter().cloned().collect();
        let signal = self.aggregator.generate_signal(&candles, symbol)?;

        println!(
            "[{}] Signal: {:?} (confidence: {:.2}%, R/R: {:?})",
            symbol,
            signal.signal,
            signal.confidence * 100.0,
            signal.risk_reward_ratio()
        );

        self.signal_history.push(signal.clone());

        // Convert signal to order
        self.order_manager.process_signal(&signal)
    }

    /// Returns signal statistics
    pub fn signal_statistics(&self) -> SignalStats {
        let total = self.signal_history.len();
        let buy_signals = self.signal_history
            .iter()
            .filter(|s| s.signal.is_bullish())
            .count();
        let sell_signals = self.signal_history
            .iter()
            .filter(|s| s.signal.is_bearish())
            .count();
        let avg_confidence = if total > 0 {
            self.signal_history.iter().map(|s| s.confidence).sum::<f64>() / total as f64
        } else {
            0.0
        };

        SignalStats {
            total_signals: total,
            buy_signals,
            sell_signals,
            average_confidence: avg_confidence,
        }
    }
}

#[derive(Debug)]
pub struct SignalStats {
    pub total_signals: usize,
    pub buy_signals: usize,
    pub sell_signals: usize,
    pub average_confidence: f64,
}

// Wrapper for MovingAverageCrossover to implement trait
struct MovingAverageCrossoverWrapper {
    inner: MovingAverageCrossover,
}

impl MovingAverageCrossoverWrapper {
    fn new(fast: usize, slow: usize) -> Self {
        MovingAverageCrossoverWrapper {
            inner: MovingAverageCrossover::new(fast, slow),
        }
    }
}

impl SignalGenerator for MovingAverageCrossoverWrapper {
    fn generate(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal> {
        self.inner.generate_signal(candles, symbol)
    }

    fn name(&self) -> &str {
        "MA_Crossover"
    }
}

fn main() {
    let mut system = TradingSystem::new(100_000.0);

    // Simulate candle stream
    let test_candles = vec![
        Candle { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0 },
        Candle { open: 101.0, high: 103.0, low: 100.0, close: 102.5, volume: 1200.0 },
        Candle { open: 102.5, high: 105.0, low: 102.0, close: 104.0, volume: 1500.0 },
        Candle { open: 104.0, high: 106.0, low: 103.0, close: 105.5, volume: 1800.0 },
        Candle { open: 105.5, high: 107.0, low: 104.5, close: 106.0, volume: 2000.0 },
        // ... add more candles for real testing
    ];

    for (i, candle) in test_candles.into_iter().enumerate() {
        println!("\n--- Candle #{} ---", i + 1);
        if let Some(order) = system.on_candle("BTC/USDT", candle) {
            println!("Order created: {:?}", order);
        }
    }

    println!("\n--- Signal Statistics ---");
    let stats = system.signal_statistics();
    println!("{:?}", stats);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Signal` enum | Enumeration of trading signal types (Buy, Sell, Hold) |
| `TradingSignal` | Structure with complete signal information |
| Signal Generator | Component that analyzes data and creates signals |
| Aggregator | Combines signals from multiple sources |
| Risk Management | Position sizing based on allowed risk |
| R/R ratio | Ratio of potential profit to risk |
| `SignalGenerator` trait | Abstraction for various signal generation strategies |

## Homework

1. **MACD Generator**: Implement a signal generator based on the MACD (Moving Average Convergence Divergence) indicator. Buy signal — when MACD line crosses signal line from below, sell signal — from above.

2. **Volatility Filter**: Add a filter to the system that rejects signals during too high or too low volatility. Use ATR to measure volatility.

3. **Multi-Timeframe Analysis**: Modify `SignalAggregator` to account for signals from different timeframes (e.g., 1 hour and 4 hours). A signal is considered strong if confirmed on both timeframes.

4. **Alert System**: Create an `AlertSystem` structure that:
   - Subscribes to signals from `TradingSystem`
   - Filters signals by specified criteria
   - Generates text notifications for important signals
   - Maintains a log of all alerts

## Navigation

[← Previous day](../257-strategy-pattern-trading/en.md) | [Next day →](../259-backtesting-signals/en.md)
