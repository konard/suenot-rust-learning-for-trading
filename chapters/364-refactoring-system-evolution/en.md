# Day 364: Refactoring: System Evolution

## Trading Analogy

Imagine you've been trading manually for several years and recording all your trades in Excel. At first, it was a simple spreadsheet: date, ticker, buy price, sell price, profit. But over time you added new columns: commissions, slippage, holding time, chart screenshots, news links...

**Refactoring is reorganizing your trading journal without changing the recorded data:**

| Trading Journal | Code Refactoring |
|-----------------|------------------|
| **Split huge table into linked sheets** | Break large module into submodules |
| **Create templates for typical trades** | Extract repeating code into functions |
| **Rename columns for clarity** | Give clear names to variables and functions |
| **Add formulas instead of manual calculations** | Automate routine operations |
| **Group trades by strategies** | Organize code by domain areas |

**The main rule of refactoring:** trading results must not change. If you had $10,000 profit before reorganizing the journal — after reorganization it should be exactly the same.

## Refactoring Principles

### When to Refactor?

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Signs that code needs refactoring ("code smells")
///
/// In trading it's similar: when a trading system becomes too complex
/// to understand and maintain, it's time to simplify.

// SMELL 1: Code duplication
// Bad: same commission calculation logic repeats

mod before_refactoring {
    pub fn calculate_spot_fee(volume: f64, price: f64) -> f64 {
        let notional = volume * price;
        let fee_rate = 0.001; // 0.1%
        notional * fee_rate
    }

    pub fn calculate_futures_fee(volume: f64, price: f64) -> f64 {
        let notional = volume * price;
        let fee_rate = 0.0005; // 0.05%
        notional * fee_rate
    }

    pub fn calculate_margin_fee(volume: f64, price: f64) -> f64 {
        let notional = volume * price;
        let fee_rate = 0.001; // 0.1%
        notional * fee_rate
    }
}

// Good: extract common logic

mod after_refactoring {
    #[derive(Debug, Clone, Copy)]
    pub enum MarketType {
        Spot,
        Futures,
        Margin,
    }

    impl MarketType {
        pub fn fee_rate(&self) -> f64 {
            match self {
                MarketType::Spot => 0.001,
                MarketType::Futures => 0.0005,
                MarketType::Margin => 0.001,
            }
        }
    }

    pub fn calculate_fee(market: MarketType, volume: f64, price: f64) -> f64 {
        let notional = volume * price;
        notional * market.fee_rate()
    }
}

// SMELL 2: Long functions
// Bad: function does too much

mod long_function_before {
    use super::*;

    pub fn process_trade_signal(
        signal: &str,
        price: f64,
        balance: f64,
    ) -> Result<String, String> {
        // Signal validation
        if signal.is_empty() {
            return Err("Empty signal".to_string());
        }
        let parts: Vec<&str> = signal.split(':').collect();
        if parts.len() != 3 {
            return Err("Invalid signal format".to_string());
        }
        let symbol = parts[0];
        let side = parts[1];
        let strength = parts[2].parse::<f64>()
            .map_err(|_| "Invalid strength")?;

        // Position size calculation
        let risk_percent = 0.02;
        let position_size = balance * risk_percent * strength;
        if position_size > balance * 0.5 {
            return Err("Position too large".to_string());
        }

        // Commission calculation
        let fee = position_size * 0.001;
        let net_position = position_size - fee;

        // Order formation
        let order = format!(
            "{}:{}:{}:{}",
            symbol,
            side,
            net_position / price,
            price
        );

        Ok(order)
    }
}

// Good: break into small functions with single responsibility

mod long_function_after {
    #[derive(Debug)]
    pub struct TradeSignal {
        pub symbol: String,
        pub side: String,
        pub strength: f64,
    }

    impl TradeSignal {
        pub fn parse(signal: &str) -> Result<Self, String> {
            if signal.is_empty() {
                return Err("Empty signal".to_string());
            }

            let parts: Vec<&str> = signal.split(':').collect();
            if parts.len() != 3 {
                return Err("Invalid signal format".to_string());
            }

            let strength = parts[2].parse::<f64>()
                .map_err(|_| "Invalid strength")?;

            Ok(TradeSignal {
                symbol: parts[0].to_string(),
                side: parts[1].to_string(),
                strength,
            })
        }
    }

    pub fn calculate_position_size(
        balance: f64,
        risk_percent: f64,
        strength: f64,
    ) -> Result<f64, String> {
        let position = balance * risk_percent * strength;

        if position > balance * 0.5 {
            return Err("Position too large".to_string());
        }

        Ok(position)
    }

    pub fn apply_fee(amount: f64, fee_rate: f64) -> f64 {
        amount * (1.0 - fee_rate)
    }

    #[derive(Debug)]
    pub struct Order {
        pub symbol: String,
        pub side: String,
        pub quantity: f64,
        pub price: f64,
    }

    impl Order {
        pub fn from_signal(
            signal: &TradeSignal,
            net_position: f64,
            price: f64,
        ) -> Self {
            Order {
                symbol: signal.symbol.clone(),
                side: signal.side.clone(),
                quantity: net_position / price,
                price,
            }
        }
    }

    pub fn process_trade_signal(
        signal: &str,
        price: f64,
        balance: f64,
    ) -> Result<Order, String> {
        let signal = TradeSignal::parse(signal)?;
        let position = calculate_position_size(balance, 0.02, signal.strength)?;
        let net_position = apply_fee(position, 0.001);

        Ok(Order::from_signal(&signal, net_position, price))
    }
}

fn main() {
    // Demonstrate commission calculation refactoring
    println!("=== Commission Calculation Refactoring ===\n");

    let volume = 1.5;
    let price = 50000.0;

    // Before refactoring
    let spot_fee_old = before_refactoring::calculate_spot_fee(volume, price);
    let futures_fee_old = before_refactoring::calculate_futures_fee(volume, price);

    // After refactoring
    let spot_fee_new = after_refactoring::calculate_fee(
        after_refactoring::MarketType::Spot,
        volume,
        price,
    );
    let futures_fee_new = after_refactoring::calculate_fee(
        after_refactoring::MarketType::Futures,
        volume,
        price,
    );

    println!("Spot fee: ${:.2} (old) = ${:.2} (new)", spot_fee_old, spot_fee_new);
    println!("Futures fee: ${:.2} (old) = ${:.2} (new)", futures_fee_old, futures_fee_new);

    // Demonstrate signal processing refactoring
    println!("\n=== Signal Processing Refactoring ===\n");

    let signal = "BTCUSDT:BUY:0.8";
    let balance = 100000.0;

    match long_function_after::process_trade_signal(signal, price, balance) {
        Ok(order) => println!("Order: {:?}", order),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Extract Method Pattern

```rust
use std::collections::HashMap;

/// Extract Method is a fundamental refactoring technique.
/// Like extracting a separate trading rule from an overall strategy.

// Before refactoring: monolithic market analyzer
mod before {
    pub fn analyze_market(prices: &[f64], volumes: &[f64]) -> String {
        // Calculate moving averages
        let mut sma_20 = 0.0;
        if prices.len() >= 20 {
            let sum: f64 = prices[prices.len()-20..].iter().sum();
            sma_20 = sum / 20.0;
        }

        let mut sma_50 = 0.0;
        if prices.len() >= 50 {
            let sum: f64 = prices[prices.len()-50..].iter().sum();
            sma_50 = sum / 50.0;
        }

        // Calculate volume
        let avg_volume = if !volumes.is_empty() {
            volumes.iter().sum::<f64>() / volumes.len() as f64
        } else {
            0.0
        };

        let current_volume = *volumes.last().unwrap_or(&0.0);
        let volume_spike = current_volume > avg_volume * 1.5;

        // Determine trend
        let current_price = *prices.last().unwrap_or(&0.0);
        let trend = if sma_20 > sma_50 && current_price > sma_20 {
            "BULLISH"
        } else if sma_20 < sma_50 && current_price < sma_20 {
            "BEARISH"
        } else {
            "NEUTRAL"
        };

        // Generate signal
        if trend == "BULLISH" && volume_spike {
            "STRONG_BUY".to_string()
        } else if trend == "BEARISH" && volume_spike {
            "STRONG_SELL".to_string()
        } else if trend == "BULLISH" {
            "BUY".to_string()
        } else if trend == "BEARISH" {
            "SELL".to_string()
        } else {
            "HOLD".to_string()
        }
    }
}

// After refactoring: separated responsibilities
mod after {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Trend {
        Bullish,
        Bearish,
        Neutral,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Signal {
        StrongBuy,
        Buy,
        Hold,
        Sell,
        StrongSell,
    }

    /// Moving average calculator
    pub struct MovingAverageCalculator;

    impl MovingAverageCalculator {
        pub fn sma(prices: &[f64], period: usize) -> Option<f64> {
            if prices.len() < period {
                return None;
            }

            let sum: f64 = prices[prices.len() - period..].iter().sum();
            Some(sum / period as f64)
        }
    }

    /// Volume analyzer
    pub struct VolumeAnalyzer;

    impl VolumeAnalyzer {
        pub fn average(volumes: &[f64]) -> f64 {
            if volumes.is_empty() {
                return 0.0;
            }
            volumes.iter().sum::<f64>() / volumes.len() as f64
        }

        pub fn is_spike(volumes: &[f64], threshold: f64) -> bool {
            let current = *volumes.last().unwrap_or(&0.0);
            let average = Self::average(volumes);
            current > average * threshold
        }
    }

    /// Trend detector
    pub struct TrendDetector;

    impl TrendDetector {
        pub fn detect(prices: &[f64]) -> Trend {
            let sma_20 = MovingAverageCalculator::sma(prices, 20);
            let sma_50 = MovingAverageCalculator::sma(prices, 50);
            let current = *prices.last().unwrap_or(&0.0);

            match (sma_20, sma_50) {
                (Some(short), Some(long)) => {
                    if short > long && current > short {
                        Trend::Bullish
                    } else if short < long && current < short {
                        Trend::Bearish
                    } else {
                        Trend::Neutral
                    }
                }
                _ => Trend::Neutral,
            }
        }
    }

    /// Signal generator — combines all components
    pub struct SignalGenerator;

    impl SignalGenerator {
        pub fn generate(prices: &[f64], volumes: &[f64]) -> Signal {
            let trend = TrendDetector::detect(prices);
            let volume_spike = VolumeAnalyzer::is_spike(volumes, 1.5);

            match (trend, volume_spike) {
                (Trend::Bullish, true) => Signal::StrongBuy,
                (Trend::Bearish, true) => Signal::StrongSell,
                (Trend::Bullish, false) => Signal::Buy,
                (Trend::Bearish, false) => Signal::Sell,
                (Trend::Neutral, _) => Signal::Hold,
            }
        }
    }
}

fn main() {
    // Generate test data
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 10.0) + (i as f64).sin() * 100.0)
        .collect();

    let volumes: Vec<f64> = (0..100)
        .map(|i| 1000.0 + (i as f64 * 5.0) + if i == 99 { 3000.0 } else { 0.0 })
        .collect();

    println!("=== Before and After Refactoring Comparison ===\n");

    let signal_old = before::analyze_market(&prices, &volumes);
    let signal_new = after::SignalGenerator::generate(&prices, &volumes);

    println!("Signal (before refactoring): {}", signal_old);
    println!("Signal (after refactoring): {:?}", signal_new);

    // Benefits of new approach
    println!("\n=== Refactoring Benefits ===\n");

    // Can use components separately
    let trend = after::TrendDetector::detect(&prices);
    println!("Trend: {:?}", trend);

    let sma_20 = after::MovingAverageCalculator::sma(&prices, 20);
    let sma_50 = after::MovingAverageCalculator::sma(&prices, 50);
    println!("SMA(20): {:.2}", sma_20.unwrap_or(0.0));
    println!("SMA(50): {:.2}", sma_50.unwrap_or(0.0));

    let volume_spike = after::VolumeAnalyzer::is_spike(&volumes, 1.5);
    println!("Volume spike: {}", volume_spike);
}
```

## Refactoring Through Types

```rust
use std::marker::PhantomData;

/// In Rust, types are a powerful refactoring tool.
/// They make invalid states impossible at compile time.

// Before refactoring: order can be in invalid state
mod before {
    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: String,
        pub symbol: String,
        pub side: String,      // "BUY" or "SELL" — but what if typo?
        pub price: f64,        // Can be negative
        pub quantity: f64,     // Can be zero
        pub status: String,    // Anything
        pub filled_qty: f64,   // Can be greater than quantity
    }

    impl Order {
        pub fn execute(&mut self) -> Result<(), String> {
            // Many runtime checks
            if self.side != "BUY" && self.side != "SELL" {
                return Err("Invalid side".to_string());
            }
            if self.price <= 0.0 {
                return Err("Invalid price".to_string());
            }
            if self.quantity <= 0.0 {
                return Err("Invalid quantity".to_string());
            }
            if self.status != "PENDING" {
                return Err("Order not pending".to_string());
            }

            self.status = "FILLED".to_string();
            self.filled_qty = self.quantity;
            Ok(())
        }
    }
}

// After refactoring: invalid states are impossible

mod after {
    use std::marker::PhantomData;

    // Types for order states (Type State Pattern)
    pub struct Pending;
    pub struct PartiallyFilled;
    pub struct Filled;
    pub struct Cancelled;

    // Type-safe enums
    #[derive(Debug, Clone, Copy)]
    pub enum Side {
        Buy,
        Sell,
    }

    // Newtype pattern for validation
    #[derive(Debug, Clone, Copy)]
    pub struct Price(f64);

    impl Price {
        pub fn new(value: f64) -> Result<Self, &'static str> {
            if value <= 0.0 {
                return Err("Price must be positive");
            }
            Ok(Price(value))
        }

        pub fn value(&self) -> f64 {
            self.0
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Quantity(f64);

    impl Quantity {
        pub fn new(value: f64) -> Result<Self, &'static str> {
            if value <= 0.0 {
                return Err("Quantity must be positive");
            }
            Ok(Quantity(value))
        }

        pub fn value(&self) -> f64 {
            self.0
        }
    }

    /// Order with typed state
    #[derive(Debug)]
    pub struct Order<State> {
        id: String,
        symbol: String,
        side: Side,
        price: Price,
        quantity: Quantity,
        filled_qty: f64,
        _state: PhantomData<State>,
    }

    impl Order<Pending> {
        pub fn new(
            id: String,
            symbol: String,
            side: Side,
            price: Price,
            quantity: Quantity,
        ) -> Self {
            Order {
                id,
                symbol,
                side,
                price,
                quantity,
                filled_qty: 0.0,
                _state: PhantomData,
            }
        }

        /// Pending -> PartiallyFilled
        pub fn partial_fill(self, qty: f64) -> Order<PartiallyFilled> {
            Order {
                id: self.id,
                symbol: self.symbol,
                side: self.side,
                price: self.price,
                quantity: self.quantity,
                filled_qty: qty,
                _state: PhantomData,
            }
        }

        /// Pending -> Filled
        pub fn fill(self) -> Order<Filled> {
            Order {
                id: self.id,
                symbol: self.symbol,
                side: self.side,
                price: self.price,
                quantity: self.quantity,
                filled_qty: self.quantity.value(),
                _state: PhantomData,
            }
        }

        /// Pending -> Cancelled
        pub fn cancel(self) -> Order<Cancelled> {
            Order {
                id: self.id,
                symbol: self.symbol,
                side: self.side,
                price: self.price,
                quantity: self.quantity,
                filled_qty: 0.0,
                _state: PhantomData,
            }
        }
    }

    impl Order<PartiallyFilled> {
        /// PartiallyFilled -> Filled
        pub fn complete_fill(self) -> Order<Filled> {
            Order {
                id: self.id,
                symbol: self.symbol,
                side: self.side,
                price: self.price,
                quantity: self.quantity,
                filled_qty: self.quantity.value(),
                _state: PhantomData,
            }
        }

        /// PartiallyFilled -> Cancelled (with partial fill)
        pub fn cancel_remaining(self) -> Order<Cancelled> {
            Order {
                id: self.id,
                symbol: self.symbol,
                side: self.side,
                price: self.price,
                quantity: self.quantity,
                filled_qty: self.filled_qty,
                _state: PhantomData,
            }
        }

        pub fn filled_quantity(&self) -> f64 {
            self.filled_qty
        }
    }

    impl Order<Filled> {
        pub fn calculate_notional(&self) -> f64 {
            self.price.value() * self.quantity.value()
        }
    }

    // Common methods for all states
    impl<State> Order<State> {
        pub fn id(&self) -> &str {
            &self.id
        }

        pub fn symbol(&self) -> &str {
            &self.symbol
        }

        pub fn side(&self) -> Side {
            self.side
        }
    }
}

fn main() {
    println!("=== Refactoring Through Types ===\n");

    // Old way — errors discovered only at runtime
    println!("Before refactoring:");
    let mut old_order = before::Order {
        id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUUY".to_string(), // Typo — compiler won't catch
        price: 50000.0,
        quantity: 1.0,
        status: "PENDING".to_string(),
        filled_qty: 0.0,
    };

    match old_order.execute() {
        Ok(_) => println!("  Order executed"),
        Err(e) => println!("  Runtime error: {}", e),
    }

    // New way — errors are impossible
    println!("\nAfter refactoring:");

    let price = after::Price::new(50000.0).expect("Invalid price");
    let quantity = after::Quantity::new(1.0).expect("Invalid quantity");

    let pending_order = after::Order::new(
        "ORD-002".to_string(),
        "BTCUSDT".to_string(),
        after::Side::Buy,  // Only Buy or Sell — typo impossible
        price,
        quantity,
    );

    println!("  Created: {:?}", pending_order.id());

    // State transitions guaranteed by types
    let partial_order = pending_order.partial_fill(0.5);
    println!("  Partial fill: {} units", partial_order.filled_quantity());

    let filled_order = partial_order.complete_fill();
    println!("  Notional: ${:.2}", filled_order.calculate_notional());

    // This won't compile:
    // filled_order.cancel(); // Error: no method `cancel` for Order<Filled>
    // pending_order.calculate_notional(); // Error: moved value
}
```

## Trading System Refactoring

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Comprehensive example of trading system refactoring.
/// Shows evolution from simple script to modular architecture.

// Version 1: Everything in one function (typical for prototype)
mod v1_prototype {
    pub fn trading_bot(prices: &[f64], balance: f64) -> f64 {
        let mut cash = balance;
        let mut position = 0.0;

        for i in 1..prices.len() {
            let prev = prices[i - 1];
            let curr = prices[i];

            // Simple strategy: buy on dip, sell on rise
            if curr < prev * 0.99 && cash > 0.0 {
                // Buy
                let qty = cash / curr;
                position += qty;
                cash = 0.0;
                println!("V1: BUY {:.4} @ {:.2}", qty, curr);
            } else if curr > prev * 1.01 && position > 0.0 {
                // Sell
                cash = position * curr;
                println!("V1: SELL {:.4} @ {:.2}", position, curr);
                position = 0.0;
            }
        }

        // Final balance
        cash + position * prices.last().unwrap_or(&0.0)
    }
}

// Version 2: Extract components
mod v2_components {
    #[derive(Debug, Clone, Copy)]
    pub enum Signal {
        Buy,
        Sell,
        Hold,
    }

    /// Strategy separated from execution
    pub trait Strategy {
        fn generate_signal(&self, prev_price: f64, curr_price: f64) -> Signal;
    }

    pub struct MomentumStrategy {
        pub buy_threshold: f64,
        pub sell_threshold: f64,
    }

    impl Strategy for MomentumStrategy {
        fn generate_signal(&self, prev_price: f64, curr_price: f64) -> Signal {
            let change = (curr_price - prev_price) / prev_price;

            if change < -self.buy_threshold {
                Signal::Buy
            } else if change > self.sell_threshold {
                Signal::Sell
            } else {
                Signal::Hold
            }
        }
    }

    /// Position separated from trading logic
    pub struct Position {
        pub cash: f64,
        pub quantity: f64,
    }

    impl Position {
        pub fn new(cash: f64) -> Self {
            Position { cash, quantity: 0.0 }
        }

        pub fn buy(&mut self, price: f64) {
            if self.cash > 0.0 {
                let qty = self.cash / price;
                self.quantity += qty;
                self.cash = 0.0;
            }
        }

        pub fn sell(&mut self, price: f64) {
            if self.quantity > 0.0 {
                self.cash = self.quantity * price;
                self.quantity = 0.0;
            }
        }

        pub fn value(&self, price: f64) -> f64 {
            self.cash + self.quantity * price
        }
    }

    /// Bot uses composition
    pub fn trading_bot<S: Strategy>(
        strategy: &S,
        prices: &[f64],
        initial_balance: f64,
    ) -> f64 {
        let mut position = Position::new(initial_balance);

        for i in 1..prices.len() {
            let signal = strategy.generate_signal(prices[i - 1], prices[i]);

            match signal {
                Signal::Buy => {
                    println!("V2: BUY @ {:.2}", prices[i]);
                    position.buy(prices[i]);
                }
                Signal::Sell => {
                    println!("V2: SELL @ {:.2}", prices[i]);
                    position.sell(prices[i]);
                }
                Signal::Hold => {}
            }
        }

        position.value(*prices.last().unwrap_or(&0.0))
    }
}

// Version 3: Full modularity with traits
mod v3_modular {
    use std::collections::VecDeque;

    /// Data source abstraction
    pub trait DataSource {
        fn get_price(&self, index: usize) -> Option<f64>;
        fn len(&self) -> usize;
    }

    pub struct VecDataSource {
        prices: Vec<f64>,
    }

    impl VecDataSource {
        pub fn new(prices: Vec<f64>) -> Self {
            VecDataSource { prices }
        }
    }

    impl DataSource for VecDataSource {
        fn get_price(&self, index: usize) -> Option<f64> {
            self.prices.get(index).copied()
        }

        fn len(&self) -> usize {
            self.prices.len()
        }
    }

    /// Strategy abstraction with state
    pub trait Strategy {
        fn on_price(&mut self, price: f64) -> Signal;
        fn reset(&mut self);
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Signal {
        Buy(f64),  // with position size
        Sell(f64),
        Hold,
    }

    /// SMA Crossover strategy
    pub struct SmaCrossover {
        short_period: usize,
        long_period: usize,
        prices: VecDeque<f64>,
        prev_short_above: Option<bool>,
    }

    impl SmaCrossover {
        pub fn new(short_period: usize, long_period: usize) -> Self {
            SmaCrossover {
                short_period,
                long_period,
                prices: VecDeque::new(),
                prev_short_above: None,
            }
        }

        fn sma(&self, period: usize) -> Option<f64> {
            if self.prices.len() < period {
                return None;
            }

            let sum: f64 = self.prices.iter().rev().take(period).sum();
            Some(sum / period as f64)
        }
    }

    impl Strategy for SmaCrossover {
        fn on_price(&mut self, price: f64) -> Signal {
            self.prices.push_back(price);
            if self.prices.len() > self.long_period + 10 {
                self.prices.pop_front();
            }

            let short_sma = self.sma(self.short_period);
            let long_sma = self.sma(self.long_period);

            match (short_sma, long_sma) {
                (Some(short), Some(long)) => {
                    let short_above = short > long;

                    let signal = match self.prev_short_above {
                        Some(prev) if prev != short_above => {
                            if short_above {
                                Signal::Buy(1.0)
                            } else {
                                Signal::Sell(1.0)
                            }
                        }
                        _ => Signal::Hold,
                    };

                    self.prev_short_above = Some(short_above);
                    signal
                }
                _ => Signal::Hold,
            }
        }

        fn reset(&mut self) {
            self.prices.clear();
            self.prev_short_above = None;
        }
    }

    /// Risk management abstraction
    pub trait RiskManager {
        fn adjust_signal(&self, signal: Signal, portfolio_value: f64) -> Signal;
    }

    pub struct MaxPositionRisk {
        max_position_pct: f64,
    }

    impl MaxPositionRisk {
        pub fn new(max_position_pct: f64) -> Self {
            MaxPositionRisk { max_position_pct }
        }
    }

    impl RiskManager for MaxPositionRisk {
        fn adjust_signal(&self, signal: Signal, portfolio_value: f64) -> Signal {
            match signal {
                Signal::Buy(size) => {
                    let max_size = portfolio_value * self.max_position_pct;
                    Signal::Buy(size.min(max_size))
                }
                other => other,
            }
        }
    }

    /// Execution abstraction
    pub trait Executor {
        fn execute(&mut self, signal: Signal, price: f64) -> Option<Trade>;
    }

    #[derive(Debug, Clone)]
    pub struct Trade {
        pub side: String,
        pub price: f64,
        pub quantity: f64,
    }

    pub struct SimulatedExecutor {
        cash: f64,
        position: f64,
        fee_rate: f64,
    }

    impl SimulatedExecutor {
        pub fn new(initial_cash: f64, fee_rate: f64) -> Self {
            SimulatedExecutor {
                cash: initial_cash,
                position: 0.0,
                fee_rate,
            }
        }

        pub fn portfolio_value(&self, price: f64) -> f64 {
            self.cash + self.position * price
        }
    }

    impl Executor for SimulatedExecutor {
        fn execute(&mut self, signal: Signal, price: f64) -> Option<Trade> {
            match signal {
                Signal::Buy(size) if self.cash > 0.0 => {
                    let qty = (self.cash * size / price).min(self.cash / price);
                    let cost = qty * price * (1.0 + self.fee_rate);

                    if cost <= self.cash {
                        self.cash -= cost;
                        self.position += qty;
                        return Some(Trade {
                            side: "BUY".to_string(),
                            price,
                            quantity: qty,
                        });
                    }
                    None
                }
                Signal::Sell(size) if self.position > 0.0 => {
                    let qty = self.position * size;
                    let proceeds = qty * price * (1.0 - self.fee_rate);

                    self.cash += proceeds;
                    self.position -= qty;
                    Some(Trade {
                        side: "SELL".to_string(),
                        price,
                        quantity: qty,
                    })
                }
                _ => None,
            }
        }
    }

    /// Trading engine — brings everything together
    pub struct TradingEngine<S, R, E>
    where
        S: Strategy,
        R: RiskManager,
        E: Executor,
    {
        strategy: S,
        risk_manager: R,
        executor: E,
        trades: Vec<Trade>,
    }

    impl<S, R, E> TradingEngine<S, R, E>
    where
        S: Strategy,
        R: RiskManager,
        E: Executor,
    {
        pub fn new(strategy: S, risk_manager: R, executor: E) -> Self {
            TradingEngine {
                strategy,
                risk_manager,
                executor,
                trades: Vec::new(),
            }
        }

        pub fn process_price(&mut self, price: f64, portfolio_value: f64) {
            let raw_signal = self.strategy.on_price(price);
            let adjusted_signal = self.risk_manager.adjust_signal(raw_signal, portfolio_value);

            if let Some(trade) = self.executor.execute(adjusted_signal, price) {
                println!("V3: {} {:.4} @ {:.2}", trade.side, trade.quantity, trade.price);
                self.trades.push(trade);
            }
        }

        pub fn trades(&self) -> &[Trade] {
            &self.trades
        }
    }

    /// Convenient backtest function
    pub fn backtest<D, S, R>(
        data: &D,
        mut strategy: S,
        risk_manager: R,
        initial_balance: f64,
        fee_rate: f64,
    ) -> f64
    where
        D: DataSource,
        S: Strategy,
        R: RiskManager,
    {
        let mut executor = SimulatedExecutor::new(initial_balance, fee_rate);
        let mut engine = TradingEngine::new(strategy, risk_manager, executor);

        for i in 0..data.len() {
            if let Some(price) = data.get_price(i) {
                let portfolio_value = engine.executor.portfolio_value(price);
                engine.process_price(price, portfolio_value);
            }
        }

        let last_price = data.get_price(data.len() - 1).unwrap_or(0.0);
        engine.executor.portfolio_value(last_price)
    }
}

fn main() {
    // Generate test data: trend with noise
    let prices: Vec<f64> = (0..100)
        .map(|i| {
            let trend = 50000.0 + (i as f64) * 50.0;
            let noise = ((i as f64) * 0.5).sin() * 500.0;
            trend + noise
        })
        .collect();

    let initial_balance = 10000.0;

    println!("=== Trading System Evolution ===\n");
    println!("Initial balance: ${:.2}\n", initial_balance);

    // Version 1: Prototype
    println!("--- Version 1: Prototype ---");
    let final_v1 = v1_prototype::trading_bot(&prices, initial_balance);
    println!("Final balance: ${:.2}\n", final_v1);

    // Version 2: Components
    println!("--- Version 2: Components ---");
    let strategy = v2_components::MomentumStrategy {
        buy_threshold: 0.01,
        sell_threshold: 0.01,
    };
    let final_v2 = v2_components::trading_bot(&strategy, &prices, initial_balance);
    println!("Final balance: ${:.2}\n", final_v2);

    // Version 3: Modular architecture
    println!("--- Version 3: Modular ---");
    let data = v3_modular::VecDataSource::new(prices.clone());
    let strategy = v3_modular::SmaCrossover::new(10, 30);
    let risk = v3_modular::MaxPositionRisk::new(0.5);
    let final_v3 = v3_modular::backtest(&data, strategy, risk, initial_balance, 0.001);
    println!("Final balance: ${:.2}\n", final_v3);

    println!("=== Benefits of Modular Version ===");
    println!("1. Strategy can be changed without modifying engine");
    println!("2. Risk management is independent from strategy");
    println!("3. Easy to add real execution instead of simulation");
    println!("4. Each component can be tested separately");
    println!("5. Code is self-documenting through types");
}
```

## Refactoring Quality Metrics

```rust
use std::collections::HashMap;

/// Metrics that help evaluate refactoring quality.
/// Like analyzing trading strategy effectiveness.

/// Code complexity metrics
#[derive(Debug, Default)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub functions_count: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub cyclomatic_complexity: usize,
    pub dependencies_count: usize,
}

impl CodeMetrics {
    pub fn quality_score(&self) -> f64 {
        let mut score = 100.0;

        // Penalty for long functions
        if self.avg_function_length > 20.0 {
            score -= (self.avg_function_length - 20.0) * 2.0;
        }

        // Penalty for high complexity
        if self.cyclomatic_complexity > 10 {
            score -= (self.cyclomatic_complexity - 10) as f64 * 3.0;
        }

        // Penalty for many dependencies
        if self.dependencies_count > 5 {
            score -= (self.dependencies_count - 5) as f64 * 2.0;
        }

        score.max(0.0)
    }
}

/// Comparison of metrics before and after refactoring
pub struct RefactoringReport {
    pub before: CodeMetrics,
    pub after: CodeMetrics,
    pub tests_passed_before: usize,
    pub tests_passed_after: usize,
    pub total_tests: usize,
}

impl RefactoringReport {
    pub fn improvement(&self) -> f64 {
        let before_score = self.before.quality_score();
        let after_score = self.after.quality_score();

        if before_score == 0.0 {
            return 0.0;
        }

        ((after_score - before_score) / before_score) * 100.0
    }

    pub fn is_safe(&self) -> bool {
        // Refactoring is safe if all tests pass
        self.tests_passed_after >= self.tests_passed_before
    }

    pub fn summary(&self) -> String {
        format!(
            "Refactoring: {:.1}% improvement, tests: {}/{} -> {}/{}, safe: {}",
            self.improvement(),
            self.tests_passed_before,
            self.total_tests,
            self.tests_passed_after,
            self.total_tests,
            if self.is_safe() { "Yes" } else { "NO!" }
        )
    }
}

/// Simulate trading system analysis
fn analyze_trading_system(version: &str) -> CodeMetrics {
    match version {
        "v1_prototype" => CodeMetrics {
            lines_of_code: 50,
            functions_count: 1,
            avg_function_length: 50.0,
            max_function_length: 50,
            cyclomatic_complexity: 12,
            dependencies_count: 0,
        },
        "v2_components" => CodeMetrics {
            lines_of_code: 80,
            functions_count: 5,
            avg_function_length: 16.0,
            max_function_length: 25,
            cyclomatic_complexity: 6,
            dependencies_count: 2,
        },
        "v3_modular" => CodeMetrics {
            lines_of_code: 150,
            functions_count: 15,
            avg_function_length: 10.0,
            max_function_length: 20,
            cyclomatic_complexity: 4,
            dependencies_count: 5,
        },
        _ => CodeMetrics::default(),
    }
}

fn main() {
    println!("=== Refactoring Quality Metrics ===\n");

    // Analyze each version
    let versions = vec!["v1_prototype", "v2_components", "v3_modular"];

    for version in &versions {
        let metrics = analyze_trading_system(version);
        println!("{}:", version);
        println!("  Lines of code: {}", metrics.lines_of_code);
        println!("  Functions: {}", metrics.functions_count);
        println!("  Avg function length: {:.1}", metrics.avg_function_length);
        println!("  Cyclomatic complexity: {}", metrics.cyclomatic_complexity);
        println!("  Quality score: {:.1}/100\n", metrics.quality_score());
    }

    // Refactoring report v1 -> v3
    println!("=== Refactoring Report v1 -> v3 ===\n");

    let report = RefactoringReport {
        before: analyze_trading_system("v1_prototype"),
        after: analyze_trading_system("v3_modular"),
        tests_passed_before: 5,
        tests_passed_after: 15,
        total_tests: 15,
    };

    println!("{}", report.summary());
    println!("\nDetails:");
    println!("  Quality before: {:.1}", report.before.quality_score());
    println!("  Quality after: {:.1}", report.after.quality_score());
    println!("  Improvement: {:.1}%", report.improvement());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Refactoring** | Improving code structure without changing behavior |
| **Extract Method** | Extracting part of code into a separate function |
| **DRY** | Don't Repeat Yourself — eliminate duplication |
| **Single Responsibility** | Each module is responsible for one thing |
| **Type State Pattern** | Using types to control states |
| **Newtype Pattern** | Wrappers for compile-time validation |
| **Composition** | Building complex from simple components |

## Practical Exercises

1. **Order Parser Refactoring**: Take an order parsing function from string and transform it:
   - Extract validation into separate functions
   - Create type-safe structures for Side, Price, Quantity
   - Add Builder pattern for order creation
   - Write tests proving behavior equivalence

2. **Modular Fee Calculator**: Create a fee calculation system:
   - Define `FeeCalculator` trait
   - Implement different strategies: fixed, percentage, tiered
   - Add calculator composition (maker/taker + volume discounts)
   - Ensure results match original logic

3. **Risk Manager Refactoring**: Transform monolithic risk manager:
   - Extract position size check
   - Extract max loss check
   - Extract concentration check
   - Use Chain of Responsibility pattern

4. **Backtester Evolution**: Take a backtesting system through three versions:
   - V1: simple loop in one function
   - V2: extracted components (DataSource, Strategy, Executor)
   - V3: full modularity with traits and generics

## Homework

1. **Complete Trading Bot Refactoring**: Take any trading code and:
   - Analyze "code smells"
   - Create refactoring plan
   - Execute refactoring step by step with commits
   - Run tests after each step
   - Measure metrics before and after
   - Document improvements

2. **Zero-Downtime Migration**: Implement the pattern:
   - New code runs parallel to old code
   - Results are compared (shadow mode)
   - Gradual traffic switching
   - Rollback on result divergence
   - Complete removal of old code

3. **Refactoring Automation**: Create tools:
   - Duplicate code detector
   - Function complexity analyzer
   - Quality report generator
   - Improvement suggestions

4. **Performance-Preserving Refactoring**: Prove refactoring didn't slow down the system:
   - Write benchmarks before refactoring
   - Save baseline results
   - Perform refactoring
   - Compare performance
   - Optimize if there's regression

## Navigation

[← Previous Day](../354-production-logging/en.md) | [Next Day →](../365-*/en.md)
