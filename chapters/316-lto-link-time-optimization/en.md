# Day 316: LTO: Link Time Optimization

## Trading Analogy

Imagine you're building a high-frequency trading (HFT) system. You have several modules:
- Market data reception module
- Signal analysis module
- Order execution module
- Risk management module

With regular compilation, each module is compiled separately, and the compiler cannot optimize the interaction between them. It's like having each trader on the team working in isolation, unaware of others' actions.

**LTO (Link Time Optimization)** is like a team-wide briefing before the trading session. The linker sees all the code together and can:
- Remove duplicate functions (one analyst instead of several)
- Inline functions from other modules (direct communication without intermediaries)
- Optimize data transfer between modules (unified communication system)

The result — your trading system runs faster, because every microsecond counts in HFT.

## What is LTO?

**LTO (Link Time Optimization)** is an optimization technique where the compiler defers some optimizations until the linking stage, when all program code is visible.

### Regular Compilation vs LTO

```
Regular compilation:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  module_a.rs│───►│ module_a.o  │    │             │
└─────────────┘    └─────────────┘    │             │
                                      │             │
┌─────────────┐    ┌─────────────┐    │   Linker    │───► binary
│  module_b.rs│───►│ module_b.o  │    │  (simple    │
└─────────────┘    └─────────────┘    │   linking)  │
                                      │             │
┌─────────────┐    ┌─────────────┐    │             │
│  module_c.rs│───►│ module_c.o  │    │             │
└─────────────┘    └─────────────┘    └─────────────┘

LTO compilation:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  module_a.rs│───►│ module_a.bc │    │             │
└─────────────┘    └─────────────┘    │   Linker    │
                                      │     +       │
┌─────────────┐    ┌─────────────┐    │   LLVM      │───► binary
│  module_b.rs│───►│ module_b.bc │    │optimizations│    (faster!)
└─────────────┘    └─────────────┘    │             │
                                      │             │
┌─────────────┐    ┌─────────────┐    │             │
│  module_c.rs│───►│ module_c.bc │    │             │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Why is this important in trading?

| Scenario | LTO Benefit |
|----------|-------------|
| **HFT systems** | Every microsecond matters, LTO can save critical cycles |
| **Big data analysis** | Loop optimization for processing millions of candles |
| **Backtesting** | Accelerating tests on historical data |
| **Risk management** | Fast real-time VaR calculations |
| **Market making** | Minimal latency for quoting |

## Enabling LTO in Rust

### Basic Configuration in Cargo.toml

```toml
[package]
name = "trading_engine"
version = "0.1.0"
edition = "2021"

[profile.release]
lto = true  # Enable full LTO
```

### LTO Options

```toml
[profile.release]
# Option 1: Full LTO (maximum optimization, slow compilation)
lto = true

# Option 2: "Fat" LTO (same as true)
lto = "fat"

# Option 3: "Thin" LTO (balance between compilation speed and optimization)
lto = "thin"

# Option 4: Disable LTO
lto = false

# Option 5: Only within crate (default)
lto = "off"
```

### Comparing LTO Modes

| Mode | Compile Time | Binary Size | Performance |
|------|--------------|-------------|-------------|
| `off` | Fast | Large | Baseline |
| `thin` | Medium | Medium | Good |
| `fat`/`true` | Slow | Small | Maximum |

## Example: Trading Engine with LTO

### Project Structure

```
trading_engine/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── market_data.rs    # Market data module
│   ├── signals.rs        # Signals module
│   ├── execution.rs      # Execution module
│   └── risk.rs           # Risk management module
```

### Cargo.toml with Optimal Settings

```toml
[package]
name = "trading_engine"
version = "0.1.0"
edition = "2021"

[dependencies]

[profile.release]
lto = "fat"           # Maximum optimization
codegen-units = 1     # Single unit for better optimization
panic = "abort"       # Less panic handling code
strip = true          # Strip debug symbols

[profile.release-with-debug]
inherits = "release"
debug = true          # For profiling
lto = "thin"          # Faster compilation with debug
```

### market_data.rs — Market Data Module

```rust
//! Market data reception and processing module

/// Tick structure (minimum unit of market data)
#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub timestamp: u64,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
}

impl Tick {
    /// Create a new tick
    #[inline]
    pub fn new(timestamp: u64, bid: f64, ask: f64, bid_size: f64, ask_size: f64) -> Self {
        Self {
            timestamp,
            bid,
            ask,
            bid_size,
            ask_size,
        }
    }

    /// Calculate mid price
    #[inline]
    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    /// Calculate spread
    #[inline]
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Calculate spread in basis points
    #[inline]
    pub fn spread_bps(&self) -> f64 {
        (self.spread() / self.mid_price()) * 10_000.0
    }
}

/// Tick to candle aggregator
pub struct CandleAggregator {
    period_seconds: u64,
    current_open: f64,
    current_high: f64,
    current_low: f64,
    current_close: f64,
    current_volume: f64,
    period_start: u64,
}

impl CandleAggregator {
    pub fn new(period_seconds: u64) -> Self {
        Self {
            period_seconds,
            current_open: 0.0,
            current_high: f64::MIN,
            current_low: f64::MAX,
            current_close: 0.0,
            current_volume: 0.0,
            period_start: 0,
        }
    }

    /// Process tick and return candle if period is complete
    #[inline]
    pub fn process_tick(&mut self, tick: &Tick) -> Option<Candle> {
        let price = tick.mid_price();
        let tick_period = tick.timestamp / self.period_seconds;

        if self.period_start == 0 {
            // First tick
            self.period_start = tick_period;
            self.current_open = price;
            self.current_high = price;
            self.current_low = price;
            self.current_close = price;
            self.current_volume = tick.bid_size + tick.ask_size;
            return None;
        }

        if tick_period > self.period_start {
            // New period — create candle
            let candle = Candle {
                timestamp: self.period_start * self.period_seconds,
                open: self.current_open,
                high: self.current_high,
                low: self.current_low,
                close: self.current_close,
                volume: self.current_volume,
            };

            // Reset for new period
            self.period_start = tick_period;
            self.current_open = price;
            self.current_high = price;
            self.current_low = price;
            self.current_close = price;
            self.current_volume = tick.bid_size + tick.ask_size;

            Some(candle)
        } else {
            // Update current period
            self.current_high = self.current_high.max(price);
            self.current_low = self.current_low.min(price);
            self.current_close = price;
            self.current_volume += tick.bid_size + tick.ask_size;
            None
        }
    }
}

/// Candle structure (OHLCV)
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
```

### signals.rs — Trading Signals Module

```rust
//! Trading signal generation module

use crate::market_data::Candle;

/// Trading signal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// SMA (Simple Moving Average) calculator
pub struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    sum: f64,
}

impl SmaCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prices: Vec::with_capacity(period),
            sum: 0.0,
        }
    }

    /// Add price and get current SMA value
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        if self.prices.len() < self.period {
            self.prices.push(price);
            self.sum += price;

            if self.prices.len() == self.period {
                Some(self.sum / self.period as f64)
            } else {
                None
            }
        } else {
            // Remove old price, add new
            let old_price = self.prices[0];
            self.sum = self.sum - old_price + price;

            // Shift array
            self.prices.rotate_left(1);
            self.prices[self.period - 1] = price;

            Some(self.sum / self.period as f64)
        }
    }

    /// Current SMA value (if available)
    #[inline]
    pub fn value(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

/// EMA (Exponential Moving Average) calculator
pub struct EmaCalculator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
    initial_sum: f64,
}

impl EmaCalculator {
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_ema: None,
            count: 0,
            initial_sum: 0.0,
        }
    }

    /// Update EMA with new price
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.count += 1;

        match self.current_ema {
            Some(ema) => {
                let new_ema = (price * self.multiplier) + (ema * (1.0 - self.multiplier));
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
            None => {
                self.initial_sum += price;
                if self.count >= self.period {
                    let sma = self.initial_sum / self.period as f64;
                    self.current_ema = Some(sma);
                    Some(sma)
                } else {
                    None
                }
            }
        }
    }

    /// Current EMA value
    #[inline]
    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

/// Moving average crossover signal generator
pub struct CrossoverSignalGenerator {
    fast_ema: EmaCalculator,
    slow_ema: EmaCalculator,
    previous_fast: Option<f64>,
    previous_slow: Option<f64>,
}

impl CrossoverSignalGenerator {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_ema: EmaCalculator::new(fast_period),
            slow_ema: EmaCalculator::new(slow_period),
            previous_fast: None,
            previous_slow: None,
        }
    }

    /// Process candle and get signal
    #[inline]
    pub fn process_candle(&mut self, candle: &Candle) -> Signal {
        let price = candle.close;

        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        let signal = match (fast, slow, self.previous_fast, self.previous_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Crossover from below — buy signal
                if pf <= ps && f > s {
                    Signal::Buy
                }
                // Crossover from above — sell signal
                else if pf >= ps && f < s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.previous_fast = fast;
        self.previous_slow = slow;

        signal
    }
}

/// RSI (Relative Strength Index) calculator
pub struct RsiCalculator {
    period: usize,
    avg_gain: f64,
    avg_loss: f64,
    previous_price: Option<f64>,
    count: usize,
    initial_gains: Vec<f64>,
    initial_losses: Vec<f64>,
}

impl RsiCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            avg_gain: 0.0,
            avg_loss: 0.0,
            previous_price: None,
            count: 0,
            initial_gains: Vec::with_capacity(period),
            initial_losses: Vec::with_capacity(period),
        }
    }

    /// Update RSI with new price
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        if let Some(prev) = self.previous_price {
            let change = price - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            self.count += 1;

            if self.count < self.period {
                self.initial_gains.push(gain);
                self.initial_losses.push(loss);
                self.previous_price = Some(price);
                return None;
            } else if self.count == self.period {
                self.initial_gains.push(gain);
                self.initial_losses.push(loss);
                self.avg_gain = self.initial_gains.iter().sum::<f64>() / self.period as f64;
                self.avg_loss = self.initial_losses.iter().sum::<f64>() / self.period as f64;
            } else {
                // Wilder's smoothing
                self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
            }

            self.previous_price = Some(price);

            if self.avg_loss == 0.0 {
                Some(100.0)
            } else {
                let rs = self.avg_gain / self.avg_loss;
                Some(100.0 - (100.0 / (1.0 + rs)))
            }
        } else {
            self.previous_price = Some(price);
            None
        }
    }
}
```

### execution.rs — Order Execution Module

```rust
//! Order execution module

use crate::signals::Signal;

/// Order type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub timestamp: u64,
}

/// Executed trade
#[derive(Debug, Clone)]
pub struct Trade {
    pub order_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub commission: f64,
    pub timestamp: u64,
}

/// Order execution manager
pub struct ExecutionManager {
    next_order_id: u64,
    commission_rate: f64, // In percent
    pending_orders: Vec<Order>,
}

impl ExecutionManager {
    pub fn new(commission_rate: f64) -> Self {
        Self {
            next_order_id: 1,
            commission_rate,
            pending_orders: Vec::new(),
        }
    }

    /// Create market order from signal
    #[inline]
    pub fn signal_to_order(
        &mut self,
        signal: Signal,
        symbol: &str,
        quantity: f64,
        timestamp: u64,
    ) -> Option<Order> {
        let side = match signal {
            Signal::Buy => OrderSide::Buy,
            Signal::Sell => OrderSide::Sell,
            Signal::Hold => return None,
        };

        let order = Order {
            id: self.next_order_id,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            timestamp,
        };

        self.next_order_id += 1;
        Some(order)
    }

    /// Execute market order at current price
    #[inline]
    pub fn execute_market_order(&self, order: &Order, current_price: f64) -> Trade {
        let execution_price = match order.side {
            OrderSide::Buy => current_price * 1.0001,  // Slippage on buy
            OrderSide::Sell => current_price * 0.9999, // Slippage on sell
        };

        let commission = order.quantity * execution_price * (self.commission_rate / 100.0);

        Trade {
            order_id: order.id,
            symbol: order.symbol.clone(),
            side: order.side,
            quantity: order.quantity,
            price: execution_price,
            commission,
            timestamp: order.timestamp,
        }
    }

    /// Add limit order to queue
    pub fn add_limit_order(&mut self, mut order: Order, limit_price: f64) {
        order.order_type = OrderType::Limit;
        order.price = Some(limit_price);
        self.pending_orders.push(order);
    }

    /// Check and execute limit orders
    #[inline]
    pub fn check_pending_orders(&mut self, bid: f64, ask: f64) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut executed_indices = Vec::new();

        for (i, order) in self.pending_orders.iter().enumerate() {
            if let Some(limit_price) = order.price {
                let should_execute = match order.side {
                    OrderSide::Buy => ask <= limit_price,   // Buy if ask below limit
                    OrderSide::Sell => bid >= limit_price, // Sell if bid above limit
                };

                if should_execute {
                    let execution_price = match order.side {
                        OrderSide::Buy => ask,
                        OrderSide::Sell => bid,
                    };

                    let commission =
                        order.quantity * execution_price * (self.commission_rate / 100.0);

                    trades.push(Trade {
                        order_id: order.id,
                        symbol: order.symbol.clone(),
                        side: order.side,
                        quantity: order.quantity,
                        price: execution_price,
                        commission,
                        timestamp: order.timestamp,
                    });

                    executed_indices.push(i);
                }
            }
        }

        // Remove executed orders (in reverse order for correct indices)
        for i in executed_indices.into_iter().rev() {
            self.pending_orders.remove(i);
        }

        trades
    }
}
```

### risk.rs — Risk Management Module

```rust
//! Risk management module

use crate::execution::{OrderSide, Trade};

/// Position in an instrument
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,       // Positive — long, negative — short
    pub average_price: f64,  // Average entry price
    pub unrealized_pnl: f64, // Unrealized profit/loss
    pub realized_pnl: f64,   // Realized profit/loss
}

impl Position {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity: 0.0,
            average_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    /// Update position after trade
    #[inline]
    pub fn update_from_trade(&mut self, trade: &Trade) {
        let trade_quantity = match trade.side {
            OrderSide::Buy => trade.quantity,
            OrderSide::Sell => -trade.quantity,
        };

        if self.quantity == 0.0 {
            // Opening new position
            self.quantity = trade_quantity;
            self.average_price = trade.price;
        } else if (self.quantity > 0.0) == (trade_quantity > 0.0) {
            // Increasing position — recalculate average price
            let total_cost = self.quantity * self.average_price + trade_quantity * trade.price;
            self.quantity += trade_quantity;
            self.average_price = total_cost / self.quantity;
        } else {
            // Reducing or reversing position
            let closing_quantity = trade_quantity.abs().min(self.quantity.abs());
            let pnl = closing_quantity * (trade.price - self.average_price) * self.quantity.signum();
            self.realized_pnl += pnl - trade.commission;

            self.quantity += trade_quantity;

            if self.quantity.abs() < 1e-10 {
                self.quantity = 0.0;
                self.average_price = 0.0;
            } else if self.quantity.signum() != (self.quantity - trade_quantity).signum() {
                // Position reversal
                self.average_price = trade.price;
            }
        }
    }

    /// Update unrealized PnL at current price
    #[inline]
    pub fn update_unrealized_pnl(&mut self, current_price: f64) {
        if self.quantity != 0.0 {
            self.unrealized_pnl = self.quantity * (current_price - self.average_price);
        } else {
            self.unrealized_pnl = 0.0;
        }
    }

    /// Total PnL
    #[inline]
    pub fn total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl
    }
}

/// Risk manager
pub struct RiskManager {
    max_position_size: f64,    // Maximum position size
    max_drawdown_percent: f64, // Maximum drawdown in %
    daily_loss_limit: f64,     // Daily loss limit
    peak_equity: f64,          // Peak equity value
    current_equity: f64,       // Current equity
    daily_pnl: f64,            // Daily PnL
    trading_halted: bool,      // Trading halted flag
}

impl RiskManager {
    pub fn new(
        initial_equity: f64,
        max_position_size: f64,
        max_drawdown_percent: f64,
        daily_loss_limit: f64,
    ) -> Self {
        Self {
            max_position_size,
            max_drawdown_percent,
            daily_loss_limit,
            peak_equity: initial_equity,
            current_equity: initial_equity,
            daily_pnl: 0.0,
            trading_halted: false,
        }
    }

    /// Check if trading is allowed
    #[inline]
    pub fn is_trading_allowed(&self) -> bool {
        !self.trading_halted
    }

    /// Check if position size is acceptable
    #[inline]
    pub fn check_position_size(&self, quantity: f64) -> bool {
        quantity.abs() <= self.max_position_size
    }

    /// Update risk state after trade
    #[inline]
    pub fn update_after_trade(&mut self, trade: &Trade, position: &Position) {
        // Update daily PnL
        self.daily_pnl = position.realized_pnl;

        // Update equity
        self.current_equity = self.peak_equity + position.total_pnl();

        // Update peak
        if self.current_equity > self.peak_equity {
            self.peak_equity = self.current_equity;
        }

        // Check stop conditions
        self.check_risk_limits();
    }

    /// Check risk limits
    #[inline]
    fn check_risk_limits(&mut self) {
        // Check daily limit
        if self.daily_pnl < -self.daily_loss_limit {
            self.trading_halted = true;
            println!("RISK: Daily loss limit reached! PnL: {:.2}", self.daily_pnl);
            return;
        }

        // Check maximum drawdown
        let drawdown_percent =
            (self.peak_equity - self.current_equity) / self.peak_equity * 100.0;

        if drawdown_percent > self.max_drawdown_percent {
            self.trading_halted = true;
            println!(
                "RISK: Maximum drawdown reached! Drawdown: {:.2}%",
                drawdown_percent
            );
        }
    }

    /// Calculate optimal position size (Kelly criterion)
    #[inline]
    pub fn calculate_kelly_size(
        &self,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> f64 {
        if avg_loss == 0.0 {
            return 0.0;
        }

        let win_loss_ratio = avg_win / avg_loss;
        let kelly = win_rate - (1.0 - win_rate) / win_loss_ratio;

        // Use half Kelly for conservativeness
        let half_kelly = kelly * 0.5;

        // Limit to maximum position size
        half_kelly.max(0.0).min(self.max_position_size / self.current_equity)
    }

    /// Current drawdown in percent
    #[inline]
    pub fn current_drawdown_percent(&self) -> f64 {
        if self.peak_equity > 0.0 {
            (self.peak_equity - self.current_equity) / self.peak_equity * 100.0
        } else {
            0.0
        }
    }

    /// Reset daily statistics
    pub fn reset_daily_stats(&mut self) {
        self.daily_pnl = 0.0;
        if !self.trading_halted || self.current_drawdown_percent() < self.max_drawdown_percent {
            self.trading_halted = false;
        }
    }
}

/// Performance metrics calculator
pub struct PerformanceMetrics {
    trades: Vec<f64>,      // PnL of each trade
    equity_curve: Vec<f64>, // Equity curve
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    /// Add trade result
    #[inline]
    pub fn add_trade(&mut self, pnl: f64, equity: f64) {
        self.trades.push(pnl);
        self.equity_curve.push(equity);
    }

    /// Calculate Sharpe ratio (simplified)
    #[inline]
    pub fn sharpe_ratio(&self, risk_free_rate: f64) -> Option<f64> {
        if self.trades.len() < 2 {
            return None;
        }

        let mean: f64 = self.trades.iter().sum::<f64>() / self.trades.len() as f64;
        let variance: f64 = self.trades
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.trades.len() - 1) as f64;

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return None;
        }

        Some((mean - risk_free_rate) / std_dev)
    }

    /// Win rate percentage
    #[inline]
    pub fn win_rate(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }

        let wins = self.trades.iter().filter(|&&x| x > 0.0).count();
        Some(wins as f64 / self.trades.len() as f64 * 100.0)
    }

    /// Profit Factor
    #[inline]
    pub fn profit_factor(&self) -> Option<f64> {
        let gross_profit: f64 = self.trades.iter().filter(|&&x| x > 0.0).sum();
        let gross_loss: f64 = self.trades.iter().filter(|&&x| x < 0.0).sum::<f64>().abs();

        if gross_loss == 0.0 {
            return None;
        }

        Some(gross_profit / gross_loss)
    }

    /// Maximum drawdown
    #[inline]
    pub fn max_drawdown(&self) -> f64 {
        let mut peak = f64::MIN;
        let mut max_dd = 0.0f64;

        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak * 100.0;
            max_dd = max_dd.max(dd);
        }

        max_dd
    }
}
```

### main.rs — Main Module

```rust
//! High-performance trading engine with LTO optimization

mod market_data;
mod signals;
mod execution;
mod risk;

use market_data::{Tick, CandleAggregator};
use signals::{CrossoverSignalGenerator, Signal, RsiCalculator};
use execution::ExecutionManager;
use risk::{Position, RiskManager, PerformanceMetrics};

use std::time::Instant;

fn main() {
    println!("=== Trading Engine with LTO Optimization ===\n");

    // Initialize components
    let mut aggregator = CandleAggregator::new(60); // 1-minute candles
    let mut signal_gen = CrossoverSignalGenerator::new(12, 26); // EMA 12/26
    let mut rsi = RsiCalculator::new(14);
    let mut execution = ExecutionManager::new(0.1); // 0.1% commission
    let mut position = Position::new("BTC/USD");
    let mut risk_manager = RiskManager::new(
        100_000.0,  // Initial capital
        10.0,       // Max position size
        20.0,       // Max drawdown 20%
        5_000.0,    // Daily loss limit
    );
    let mut metrics = PerformanceMetrics::new();

    // Generate test data (simulating 100K ticks)
    let ticks = generate_test_ticks(100_000);

    println!("Processing {} ticks...\n", ticks.len());

    let start = Instant::now();

    let mut trade_count = 0;

    for tick in &ticks {
        // Aggregate ticks into candles
        if let Some(candle) = aggregator.process_tick(tick) {
            // Generate signal
            let signal = signal_gen.process_candle(&candle);
            let _rsi_value = rsi.update(candle.close);

            // Check risk
            if !risk_manager.is_trading_allowed() {
                continue;
            }

            // Create and execute order
            if signal != Signal::Hold {
                if let Some(order) = execution.signal_to_order(
                    signal,
                    "BTC/USD",
                    0.1, // Position size
                    candle.timestamp,
                ) {
                    // Check position size
                    if risk_manager.check_position_size(0.1) {
                        let trade = execution.execute_market_order(&order, tick.mid_price());

                        // Update position
                        position.update_from_trade(&trade);
                        position.update_unrealized_pnl(tick.mid_price());

                        // Update risk manager
                        risk_manager.update_after_trade(&trade, &position);

                        // Record metric
                        metrics.add_trade(
                            position.realized_pnl,
                            100_000.0 + position.total_pnl(),
                        );

                        trade_count += 1;
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();

    // Output results
    println!("=== Results ===\n");
    println!("Processing time: {:?}", elapsed);
    println!("Ticks per second: {:.0}", ticks.len() as f64 / elapsed.as_secs_f64());
    println!("Total trades: {}", trade_count);
    println!();
    println!("Position: {:.4} BTC", position.quantity);
    println!("Average price: ${:.2}", position.average_price);
    println!("Realized PnL: ${:.2}", position.realized_pnl);
    println!("Unrealized PnL: ${:.2}", position.unrealized_pnl);
    println!("Total PnL: ${:.2}", position.total_pnl());
    println!();

    if let Some(wr) = metrics.win_rate() {
        println!("Win Rate: {:.1}%", wr);
    }
    if let Some(pf) = metrics.profit_factor() {
        println!("Profit Factor: {:.2}", pf);
    }
    if let Some(sharpe) = metrics.sharpe_ratio(0.0) {
        println!("Sharpe Ratio: {:.2}", sharpe);
    }
    println!("Max Drawdown: {:.2}%", metrics.max_drawdown());
}

/// Generate test ticks
fn generate_test_ticks(count: usize) -> Vec<Tick> {
    let mut ticks = Vec::with_capacity(count);
    let mut price = 42000.0;
    let mut timestamp = 1700000000u64;

    for i in 0..count {
        // Random price change
        let change = ((i * 17 + 13) % 100) as f64 / 100.0 - 0.5;
        price += change * 10.0;
        price = price.max(40000.0).min(45000.0);

        let spread = 0.5 + ((i * 7) % 10) as f64 / 10.0;
        let bid = price - spread / 2.0;
        let ask = price + spread / 2.0;

        ticks.push(Tick::new(
            timestamp,
            bid,
            ask,
            1.0 + (i % 5) as f64,
            1.0 + ((i + 3) % 5) as f64,
        ));

        timestamp += 100; // 100ms between ticks
    }

    ticks
}
```

## Measuring LTO Impact

### Benchmark: Comparing Modes

```rust
//! Benchmark for comparing performance with different LTO settings

use std::time::Instant;

/// SMA calculation — typical trading operation
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();

    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

/// EMA calculation
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    result.push(prices[0]);

    for &price in &prices[1..] {
        let prev_ema = *result.last().unwrap();
        let ema = (price * multiplier) + (prev_ema * (1.0 - multiplier));
        result.push(ema);
    }

    result
}

/// Check crossover
#[inline(always)]
fn check_crossover(fast: &[f64], slow: &[f64]) -> Vec<i32> {
    fast.iter()
        .zip(slow.iter())
        .zip(fast.iter().skip(1).zip(slow.iter().skip(1)))
        .map(|((pf, ps), (cf, cs))| {
            if pf <= ps && cf > cs {
                1  // Bullish crossover
            } else if pf >= ps && cf < cs {
                -1 // Bearish crossover
            } else {
                0  // No crossover
            }
        })
        .collect()
}

fn main() {
    // Generate data
    let prices: Vec<f64> = (0..1_000_000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0)
        .collect();

    println!("=== LTO Benchmark ===\n");
    println!("Data points: {}\n", prices.len());

    // SMA benchmark
    let start = Instant::now();
    for _ in 0..10 {
        let _ = calculate_sma(&prices, 20);
    }
    let sma_time = start.elapsed() / 10;
    println!("SMA(20): {:?} per iteration", sma_time);

    // EMA benchmark
    let start = Instant::now();
    for _ in 0..10 {
        let _ = calculate_ema(&prices, 20);
    }
    let ema_time = start.elapsed() / 10;
    println!("EMA(20): {:?} per iteration", ema_time);

    // Crossover benchmark
    let fast_ema = calculate_ema(&prices, 12);
    let slow_ema = calculate_ema(&prices, 26);

    let start = Instant::now();
    for _ in 0..10 {
        let _ = check_crossover(&fast_ema, &slow_ema);
    }
    let cross_time = start.elapsed() / 10;
    println!("Crossover check: {:?} per iteration", cross_time);

    println!("\nTip: Compile with different LTO settings and compare!");
    println!("  cargo build --release                    # lto = false");
    println!("  cargo build --release --config 'profile.release.lto=\"thin\"'");
    println!("  cargo build --release --config 'profile.release.lto=\"fat\"'");
}
```

### Performance Comparison Script

```bash
#!/bin/bash
# benchmark_lto.sh - Compare performance with different LTO modes

echo "=== LTO Benchmark ==="

# Build without LTO
echo -e "\n[1/3] Building without LTO..."
cargo build --release 2>/dev/null
echo "Binary size (no LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Running benchmark..."
time target/release/trading_engine

# Build with Thin LTO
echo -e "\n[2/3] Building with Thin LTO..."
cargo build --release --config 'profile.release.lto="thin"' 2>/dev/null
echo "Binary size (Thin LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Running benchmark..."
time target/release/trading_engine

# Build with Fat LTO
echo -e "\n[3/3] Building with Fat LTO..."
cargo build --release --config 'profile.release.lto="fat"' --config 'profile.release.codegen-units=1' 2>/dev/null
echo "Binary size (Fat LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Running benchmark..."
time target/release/trading_engine

echo -e "\n=== Comparison complete ==="
```

## When to Use LTO

### Recommendations

| Situation | Recommendation |
|-----------|----------------|
| **Development** | `lto = false` — fast compilation |
| **CI/CD tests** | `lto = "thin"` — balanced speed |
| **Production HFT** | `lto = "fat"` — maximum speed |
| **Backtesting** | `lto = "thin"` — good balance |
| **Debugging** | `lto = false` — preserve symbols |

### Additional Optimizations with LTO

```toml
[profile.release]
lto = "fat"
codegen-units = 1        # Single compilation unit
opt-level = 3            # Maximum optimization
panic = "abort"          # Remove stack unwinding code
strip = true             # Strip debug symbols
debug = false            # No debug info

[profile.release.package."*"]
opt-level = 3            # Optimize dependencies too
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **LTO** | Link Time Optimization — optimization at linking stage |
| **Fat LTO** | Full cross-module optimization (slow compilation) |
| **Thin LTO** | Fast parallel LTO with good results |
| **codegen-units** | Number of parallel compilation units |
| **Inlining** | LTO enables cross-module function inlining |
| **Dead code elimination** | Removing unused code from entire program |

## Practical Exercises

1. **Trading Indicator Benchmark**: Implement MACD calculation and measure performance with different LTO modes. Use `criterion` for accurate measurements.

2. **Profiling**: Use `perf` or `flamegraph` to analyze which functions were inlined with and without LTO. Compare call graphs.

3. **Binary Size**: Create a comparison table of your trading application's binary size with different settings (`lto`, `strip`, `panic`).

4. **Cross-Module Optimization**: Create a trading indicators library and a main application. Measure how much LTO improves performance when calling library functions.

## Homework

1. **Optimal Build Profiles**: Create multiple build profiles for different trading bot use cases:
   - `dev` — fast development
   - `test` — for running tests
   - `bench` — for benchmarks
   - `production` — for production use
   Measure compilation time and performance for each.

2. **C++ Comparison**: Write equivalent SMA/EMA calculation code in C++ with `-flto`. Compare Rust LTO vs C++ LTO performance on identical data.

3. **LTO Impact Analysis**: Using `llvm-mca` or similar tools, analyze the generated machine code for a critical function (e.g., RSI calculation) with and without LTO. Document the differences.

4. **CI/CD Integration**: Set up GitHub Actions or similar CI system for automatic trading application builds with different LTO profiles and artifact publishing. Add automatic benchmarks to track performance regressions.

## Navigation

[← Previous day](../314-ffi-c-library-integration/en.md) | [Next day →](../317-*/en.md)
