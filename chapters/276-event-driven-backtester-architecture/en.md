# Day 276: Event-Driven Backtester: Architecture Based on Events

## Trading Analogy

Imagine a trading floor: every action is an event. A new quote arrives — that's an event. A trader places an order — event. An order gets filled — event. Balance changes — event. Everything that happens can be described as a sequence of events.

**Event-Driven Architecture (EDA)** is an architectural pattern where the system reacts to events rather than executing code sequentially. This is an ideal approach for backtesting trading strategies:

- **Realism**: Real trading works on events — quotes, orders, executions
- **Modularity**: Each component handles its own logic
- **Testability**: Components can be easily swapped for testing
- **Portability**: The same code works in both backtest and live trading

## What is an Event-Driven Backtester?

An Event-Driven Backtester consists of several key components:

```
┌─────────────────────────────────────────────────────────────────┐
│                        EVENT QUEUE                               │
│  [MarketEvent] → [SignalEvent] → [OrderEvent] → [FillEvent]     │
└─────────────────────────────────────────────────────────────────┘
       ↓                ↓                ↓              ↓
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ DataHandler  │ │   Strategy   │ │  Portfolio   │ │  Execution   │
│              │ │              │ │              │ │   Handler    │
│ Generates    │ │ Analyzes     │ │ Manages      │ │ Executes     │
│ MarketEvent  │ │ and creates  │ │ positions    │ │ orders       │
│              │ │ SignalEvent  │ │              │ │              │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

## Defining Event Types

```rust
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trade direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Long,   // Buy
    Short,  // Sell (short)
}

/// Event types in the system
#[derive(Debug, Clone)]
pub enum Event {
    /// New market data (candle, tick)
    Market(MarketEvent),
    /// Signal from strategy
    Signal(SignalEvent),
    /// Order for execution
    Order(OrderEvent),
    /// Executed trade
    Fill(FillEvent),
}

/// Market data (OHLCV candle)
#[derive(Debug, Clone)]
pub struct MarketEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Signal from trading strategy
#[derive(Debug, Clone)]
pub struct SignalEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub strength: f64,  // Signal strength from 0.0 to 1.0
}

/// Buy/sell order
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub quantity: f64,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Market,
    Limit(f64),  // Limit price
}

/// Executed trade
#[derive(Debug, Clone)]
pub struct FillEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub quantity: f64,
    pub fill_price: f64,
    pub commission: f64,
}

impl MarketEvent {
    pub fn new(symbol: &str, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        MarketEvent {
            timestamp,
            symbol: symbol.to_string(),
            open,
            high,
            low,
            close,
            volume,
        }
    }
}
```

## Event Queue

The central element of the architecture — the event queue:

```rust
/// Event queue — the heart of the system
pub struct EventQueue {
    events: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            events: VecDeque::new(),
        }
    }

    /// Add event to queue
    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    /// Get next event
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Check if there are events
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Number of events in queue
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}
```

## Data Handler: Market Data Source

```rust
/// Historical data source for backtesting
pub struct DataHandler {
    symbol: String,
    data: Vec<MarketEvent>,
    current_index: usize,
}

impl DataHandler {
    pub fn new(symbol: &str, data: Vec<MarketEvent>) -> Self {
        DataHandler {
            symbol: symbol.to_string(),
            data,
            current_index: 0,
        }
    }

    /// Create sample data
    pub fn with_sample_data(symbol: &str) -> Self {
        let mut data = Vec::new();
        let base_price = 100.0;

        // Generate 100 candles with a trend
        for i in 0..100 {
            let trend = (i as f64 * 0.1).sin() * 10.0;
            let noise = (i as f64 * 0.5).cos() * 2.0;
            let price = base_price + trend + noise;

            let event = MarketEvent {
                timestamp: 1000000 + i as u64 * 3600,
                symbol: symbol.to_string(),
                open: price - 0.5,
                high: price + 1.0,
                low: price - 1.0,
                close: price + 0.3,
                volume: 1000.0 + (i as f64 * 10.0),
            };
            data.push(event);
        }

        DataHandler::new(symbol, data)
    }

    /// Get next bar (generates MarketEvent)
    pub fn get_next_bar(&mut self, queue: &mut EventQueue) -> bool {
        if self.current_index >= self.data.len() {
            return false; // Data exhausted
        }

        let bar = self.data[self.current_index].clone();
        queue.push(Event::Market(bar));
        self.current_index += 1;
        true
    }

    /// Get current closing price
    pub fn get_latest_price(&self) -> Option<f64> {
        if self.current_index > 0 {
            Some(self.data[self.current_index - 1].close)
        } else {
            None
        }
    }

    /// Reset position for new backtest
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}
```

## Trading Strategy

```rust
/// Trait for trading strategies
pub trait Strategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue);
}

/// Simple moving average crossover strategy
pub struct MovingAverageCrossStrategy {
    symbol: String,
    short_window: usize,
    long_window: usize,
    prices: Vec<f64>,
    in_position: bool,
}

impl MovingAverageCrossStrategy {
    pub fn new(symbol: &str, short_window: usize, long_window: usize) -> Self {
        MovingAverageCrossStrategy {
            symbol: symbol.to_string(),
            short_window,
            long_window,
            prices: Vec::new(),
            in_position: false,
        }
    }

    fn calculate_sma(&self, window: usize) -> Option<f64> {
        if self.prices.len() < window {
            return None;
        }

        let sum: f64 = self.prices.iter().rev().take(window).sum();
        Some(sum / window as f64)
    }
}

impl Strategy for MovingAverageCrossStrategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue) {
        // Add new price
        self.prices.push(event.close);

        // Wait for enough data
        if self.prices.len() < self.long_window {
            return;
        }

        // Calculate moving averages
        let short_sma = self.calculate_sma(self.short_window).unwrap();
        let long_sma = self.calculate_sma(self.long_window).unwrap();

        // Generate signals
        if short_sma > long_sma && !self.in_position {
            // Short MA crossed above long MA — buy
            let signal = SignalEvent {
                timestamp: event.timestamp,
                symbol: self.symbol.clone(),
                direction: Direction::Long,
                strength: (short_sma - long_sma) / long_sma, // Relative strength
            };
            queue.push(Event::Signal(signal));
            self.in_position = true;
            println!("Buy signal: SMA{}={:.2} > SMA{}={:.2}",
                     self.short_window, short_sma,
                     self.long_window, long_sma);
        } else if short_sma < long_sma && self.in_position {
            // Short MA crossed below long MA — sell
            let signal = SignalEvent {
                timestamp: event.timestamp,
                symbol: self.symbol.clone(),
                direction: Direction::Short,
                strength: (long_sma - short_sma) / long_sma,
            };
            queue.push(Event::Signal(signal));
            self.in_position = false;
            println!("Sell signal: SMA{}={:.2} < SMA{}={:.2}",
                     self.short_window, short_sma,
                     self.long_window, long_sma);
        }
    }
}
```

## Portfolio: Position and Risk Management

```rust
use std::collections::HashMap;

/// Position for an instrument
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

impl Position {
    pub fn new(symbol: &str) -> Self {
        Position {
            symbol: symbol.to_string(),
            quantity: 0.0,
            avg_price: 0.0,
            current_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
        if self.quantity != 0.0 {
            self.unrealized_pnl = (price - self.avg_price) * self.quantity;
        }
    }
}

/// Trader's portfolio
pub struct Portfolio {
    initial_capital: f64,
    cash: f64,
    positions: HashMap<String, Position>,
    total_commission: f64,
    trade_count: u32,
    equity_curve: Vec<f64>,
}

impl Portfolio {
    pub fn new(initial_capital: f64) -> Self {
        Portfolio {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            total_commission: 0.0,
            trade_count: 0,
            equity_curve: vec![initial_capital],
        }
    }

    /// Handle signal — create order
    pub fn handle_signal(&mut self, signal: &SignalEvent, queue: &mut EventQueue, current_price: f64) {
        // Simple risk management: invest 10% of capital
        let position_size = self.cash * 0.1;
        let quantity = position_size / current_price;

        let order = OrderEvent {
            timestamp: signal.timestamp,
            symbol: signal.symbol.clone(),
            direction: signal.direction,
            quantity,
            order_type: OrderType::Market,
        };

        queue.push(Event::Order(order));
    }

    /// Handle executed trade
    pub fn handle_fill(&mut self, fill: &FillEvent) {
        let position = self.positions
            .entry(fill.symbol.clone())
            .or_insert_with(|| Position::new(&fill.symbol));

        match fill.direction {
            Direction::Long => {
                // Buy
                let total_cost = fill.fill_price * fill.quantity + fill.commission;

                if position.quantity > 0.0 {
                    // Average into position
                    let total_qty = position.quantity + fill.quantity;
                    position.avg_price =
                        (position.avg_price * position.quantity + fill.fill_price * fill.quantity)
                        / total_qty;
                    position.quantity = total_qty;
                } else if position.quantity < 0.0 {
                    // Close short
                    let pnl = (position.avg_price - fill.fill_price) * fill.quantity.min(-position.quantity);
                    position.realized_pnl += pnl;
                    position.quantity += fill.quantity;
                    if position.quantity > 0.0 {
                        position.avg_price = fill.fill_price;
                    }
                } else {
                    position.avg_price = fill.fill_price;
                    position.quantity = fill.quantity;
                }

                self.cash -= total_cost;
            }
            Direction::Short => {
                // Sell
                let revenue = fill.fill_price * fill.quantity - fill.commission;

                if position.quantity > 0.0 {
                    // Close long
                    let pnl = (fill.fill_price - position.avg_price) * fill.quantity.min(position.quantity);
                    position.realized_pnl += pnl;
                    position.quantity -= fill.quantity;
                } else {
                    // Open or increase short
                    position.avg_price = fill.fill_price;
                    position.quantity -= fill.quantity;
                }

                self.cash += revenue;
            }
        }

        self.total_commission += fill.commission;
        self.trade_count += 1;

        // Update equity curve
        self.update_equity(fill.fill_price);
    }

    /// Update equity on price change
    pub fn update_market_value(&mut self, symbol: &str, price: f64) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.update_price(price);
        }
        self.update_equity(price);
    }

    fn update_equity(&mut self, _current_price: f64) {
        let positions_value: f64 = self.positions.values()
            .map(|p| p.quantity * p.current_price)
            .sum();
        let total_equity = self.cash + positions_value;
        self.equity_curve.push(total_equity);
    }

    /// Get final statistics
    pub fn get_stats(&self) -> PortfolioStats {
        let final_equity = self.equity_curve.last().copied().unwrap_or(self.initial_capital);
        let total_return = (final_equity - self.initial_capital) / self.initial_capital * 100.0;

        // Calculate maximum drawdown
        let mut max_equity = self.initial_capital;
        let mut max_drawdown = 0.0;
        for &equity in &self.equity_curve {
            if equity > max_equity {
                max_equity = equity;
            }
            let drawdown = (max_equity - equity) / max_equity * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let realized_pnl: f64 = self.positions.values()
            .map(|p| p.realized_pnl)
            .sum();

        PortfolioStats {
            initial_capital: self.initial_capital,
            final_equity,
            total_return,
            max_drawdown,
            total_trades: self.trade_count,
            total_commission: self.total_commission,
            realized_pnl,
        }
    }
}

#[derive(Debug)]
pub struct PortfolioStats {
    pub initial_capital: f64,
    pub final_equity: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub total_trades: u32,
    pub total_commission: f64,
    pub realized_pnl: f64,
}

impl std::fmt::Display for PortfolioStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\n+======================================+\n\
             |       BACKTEST RESULTS               |\n\
             +======================================+\n\
             | Initial Capital:    {:>15.2} |\n\
             | Final Equity:       {:>15.2} |\n\
             | Total Return:       {:>14.2}% |\n\
             | Max Drawdown:       {:>14.2}% |\n\
             | Total Trades:       {:>15} |\n\
             | Total Commission:   {:>15.2} |\n\
             | Realized P&L:       {:>15.2} |\n\
             +======================================+",
            self.initial_capital,
            self.final_equity,
            self.total_return,
            self.max_drawdown,
            self.total_trades,
            self.total_commission,
            self.realized_pnl
        )
    }
}
```

## Execution Handler: Order Execution

```rust
/// Order execution simulator
pub struct ExecutionHandler {
    commission_rate: f64,  // Commission percentage
    slippage: f64,         // Slippage
}

impl ExecutionHandler {
    pub fn new(commission_rate: f64, slippage: f64) -> Self {
        ExecutionHandler {
            commission_rate,
            slippage,
        }
    }

    /// Simulate order execution
    pub fn execute_order(&self, order: &OrderEvent, current_price: f64, queue: &mut EventQueue) {
        // Add slippage
        let fill_price = match order.direction {
            Direction::Long => current_price * (1.0 + self.slippage),
            Direction::Short => current_price * (1.0 - self.slippage),
        };

        // Calculate commission
        let commission = fill_price * order.quantity * self.commission_rate;

        let fill = FillEvent {
            timestamp: order.timestamp,
            symbol: order.symbol.clone(),
            direction: order.direction,
            quantity: order.quantity,
            fill_price,
            commission,
        };

        println!("Order filled: {:?} {} @ {:.2} (commission: {:.2})",
                 order.direction, order.symbol, fill_price, commission);

        queue.push(Event::Fill(fill));
    }
}
```

## Main Backtesting Loop

```rust
/// Backtesting engine
pub struct Backtester {
    event_queue: EventQueue,
    data_handler: DataHandler,
    strategy: Box<dyn Strategy>,
    portfolio: Portfolio,
    execution_handler: ExecutionHandler,
}

impl Backtester {
    pub fn new(
        data_handler: DataHandler,
        strategy: Box<dyn Strategy>,
        initial_capital: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Self {
        Backtester {
            event_queue: EventQueue::new(),
            data_handler,
            strategy,
            portfolio: Portfolio::new(initial_capital),
            execution_handler: ExecutionHandler::new(commission_rate, slippage),
        }
    }

    /// Run backtest
    pub fn run(&mut self) {
        println!("Starting backtest...\n");

        // Main backtesting loop
        loop {
            // 1. Get new market data
            if !self.data_handler.get_next_bar(&mut self.event_queue) {
                // Data exhausted
                break;
            }

            // 2. Process all events in queue
            while let Some(event) = self.event_queue.pop() {
                match event {
                    Event::Market(ref market_event) => {
                        // Update portfolio market value
                        self.portfolio.update_market_value(
                            &market_event.symbol,
                            market_event.close
                        );

                        // Strategy analyzes data
                        self.strategy.calculate_signals(
                            market_event,
                            &mut self.event_queue
                        );
                    }
                    Event::Signal(ref signal_event) => {
                        // Portfolio handles signal
                        if let Some(price) = self.data_handler.get_latest_price() {
                            self.portfolio.handle_signal(
                                signal_event,
                                &mut self.event_queue,
                                price
                            );
                        }
                    }
                    Event::Order(ref order_event) => {
                        // Execute order
                        if let Some(price) = self.data_handler.get_latest_price() {
                            self.execution_handler.execute_order(
                                order_event,
                                price,
                                &mut self.event_queue
                            );
                        }
                    }
                    Event::Fill(ref fill_event) => {
                        // Update portfolio
                        self.portfolio.handle_fill(fill_event);
                    }
                }
            }
        }

        println!("\nBacktest completed!");
    }

    /// Get results
    pub fn get_results(&self) -> PortfolioStats {
        self.portfolio.get_stats()
    }
}

fn main() {
    // Create components
    let data_handler = DataHandler::with_sample_data("BTC/USDT");
    let strategy = Box::new(MovingAverageCrossStrategy::new("BTC/USDT", 5, 20));

    // Create backtester
    let mut backtester = Backtester::new(
        data_handler,
        strategy,
        100_000.0,  // Initial capital
        0.001,      // Commission 0.1%
        0.0005,     // Slippage 0.05%
    );

    // Run backtest
    backtester.run();

    // Print results
    let stats = backtester.get_results();
    println!("{}", stats);
}
```

## Extended Example: RSI Strategy

```rust
/// RSI-based strategy (Relative Strength Index)
pub struct RSIStrategy {
    symbol: String,
    period: usize,
    overbought: f64,
    oversold: f64,
    prices: Vec<f64>,
    in_position: bool,
}

impl RSIStrategy {
    pub fn new(symbol: &str, period: usize, overbought: f64, oversold: f64) -> Self {
        RSIStrategy {
            symbol: symbol.to_string(),
            period,
            overbought,
            oversold,
            prices: Vec::new(),
            in_position: false,
        }
    }

    fn calculate_rsi(&self) -> Option<f64> {
        if self.prices.len() < self.period + 1 {
            return None;
        }

        let changes: Vec<f64> = self.prices
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        let recent_changes: Vec<f64> = changes
            .iter()
            .rev()
            .take(self.period)
            .copied()
            .collect();

        let gains: f64 = recent_changes.iter()
            .filter(|&&x| x > 0.0)
            .sum();
        let losses: f64 = recent_changes.iter()
            .filter(|&&x| x < 0.0)
            .map(|x| x.abs())
            .sum();

        let avg_gain = gains / self.period as f64;
        let avg_loss = losses / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }
}

impl Strategy for RSIStrategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue) {
        self.prices.push(event.close);

        if let Some(rsi) = self.calculate_rsi() {
            // RSI below oversold — buy signal
            if rsi < self.oversold && !self.in_position {
                let signal = SignalEvent {
                    timestamp: event.timestamp,
                    symbol: self.symbol.clone(),
                    direction: Direction::Long,
                    strength: (self.oversold - rsi) / self.oversold,
                };
                queue.push(Event::Signal(signal));
                self.in_position = true;
                println!("RSI buy signal: RSI = {:.2} (oversold)", rsi);
            }
            // RSI above overbought — sell signal
            else if rsi > self.overbought && self.in_position {
                let signal = SignalEvent {
                    timestamp: event.timestamp,
                    symbol: self.symbol.clone(),
                    direction: Direction::Short,
                    strength: (rsi - self.overbought) / (100.0 - self.overbought),
                };
                queue.push(Event::Signal(signal));
                self.in_position = false;
                println!("RSI sell signal: RSI = {:.2} (overbought)", rsi);
            }
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Event-Driven Architecture | System reacts to events rather than executing code sequentially |
| EventQueue | Central queue for passing events between components |
| MarketEvent | Event with market data (OHLCV) |
| SignalEvent | Signal from trading strategy |
| OrderEvent | Order for execution |
| FillEvent | Executed trade |
| DataHandler | Historical data source |
| Strategy trait | Interface for trading strategies |
| Portfolio | Position, balance, and risk management |
| ExecutionHandler | Order execution simulation |

## Homework

### Exercise 1: Bollinger Bands Strategy

Implement a Bollinger Bands strategy:
- Buy when price touches the lower band
- Sell when price touches the upper band

```rust
pub struct BollingerBandsStrategy {
    symbol: String,
    period: usize,
    num_std: f64,
    prices: Vec<f64>,
    in_position: bool,
}

// Implement methods:
// - calculate_bands(&self) -> Option<(f64, f64, f64)>  // (upper, middle, lower)
// - Strategy trait
```

### Exercise 2: Stop-Loss and Take-Profit

Add stop-loss and take-profit support to `Portfolio`:
- Set SL/TP levels when opening a position
- Automatically close position when levels are reached

### Exercise 3: Multiple Instruments

Extend `DataHandler` to work with multiple trading pairs simultaneously. Implement a pairs trading strategy.

### Exercise 4: Parameter Optimization

Create a function for strategy parameter optimization:

```rust
fn optimize_strategy(
    data: &[MarketEvent],
    param_ranges: &[(f64, f64, f64)], // (min, max, step)
) -> (Vec<f64>, PortfolioStats) {
    // Iterate through all parameter combinations
    // Return best parameters and results
}
```

## Navigation

[← Previous day](../275-backtesting-fundamentals/en.md) | [Next day →](../277-backtester-data-handling/en.md)
