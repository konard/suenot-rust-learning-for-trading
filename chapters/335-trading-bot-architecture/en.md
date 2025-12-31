# Day 335: Trading Bot Architecture

## Trading Analogy

Imagine you're building a trading firm — not just a simple office, but an entire complex with different departments:

- **Data Department** — receives quotes from exchanges, filters and normalizes them
- **Analytics Department** — calculates indicators and patterns
- **Strategy Department** — makes trading decisions
- **Execution Department** — sends orders to the exchange
- **Risk Management** — monitors positions and limits
- **Logistics** — stores history and system state

Each department works independently, but they coordinate through a **central messaging system**. If analytics is overloaded, data is buffered. If risk management says "stop", execution is blocked.

**Trading bot architecture** in Rust follows this structure:
- **Modules** = departments with their own responsibilities
- **Channels** = internal mail between departments
- **Actors** = workers processing messages
- **Shared state** = common resources with access control

## Main Trading Bot Components

| Component | Responsibility | Pattern |
|-----------|----------------|---------|
| **Market Data** | Receiving and normalizing data | Publisher-Subscriber |
| **Strategy Engine** | Generating signals | State Machine |
| **Order Manager** | Managing orders | Command Pattern |
| **Risk Manager** | Risk control | Interceptor |
| **Position Tracker** | Position accounting | Observer |
| **Logger/Metrics** | Monitoring | Decorator |

## Basic Project Structure

```
trading-bot/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Public API
│   ├── config.rs            # Configuration
│   ├── market_data/
│   │   ├── mod.rs           # Data module
│   │   ├── provider.rs      # Data sources
│   │   └── normalizer.rs    # Normalization
│   ├── strategy/
│   │   ├── mod.rs           # Strategy module
│   │   ├── signal.rs        # Signals
│   │   └── indicators.rs    # Indicators
│   ├── execution/
│   │   ├── mod.rs           # Execution module
│   │   ├── order.rs         # Orders
│   │   └── exchange.rs      # Exchange interface
│   ├── risk/
│   │   ├── mod.rs           # Risk module
│   │   └── limits.rs        # Limits
│   └── core/
│       ├── mod.rs           # System core
│       ├── types.rs         # Common types
│       └── events.rs        # Events
└── tests/
    └── integration_test.rs
```

## Defining Base Types

```rust
use std::time::{SystemTime, Duration};
use std::collections::HashMap;

/// Trading symbol (pair)
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: &str) -> Self {
        Symbol(s.to_uppercase())
    }
}

/// Trade side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Market data (tick)
#[derive(Debug, Clone)]
pub struct Tick {
    pub symbol: Symbol,
    pub bid: f64,
    pub ask: f64,
    pub last_price: f64,
    pub volume: f64,
    pub timestamp: SystemTime,
}

impl Tick {
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    pub fn spread_pct(&self) -> f64 {
        self.spread() / self.bid * 100.0
    }

    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

/// OHLCV candle
#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: Symbol,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: SystemTime,
    pub timeframe: Duration,
}

impl Candle {
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

/// Order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub average_fill_price: Option<f64>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Order {
    pub fn new_market(symbol: Symbol, side: Side, quantity: f64) -> Self {
        let now = SystemTime::now();
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol,
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            status: OrderStatus::Pending,
            filled_quantity: 0.0,
            average_fill_price: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_limit(symbol: Symbol, side: Side, quantity: f64, price: f64) -> Self {
        let now = SystemTime::now();
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol,
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            status: OrderStatus::Pending,
            filled_quantity: 0.0,
            average_fill_price: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected)
    }
}

/// Position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: Symbol,
    pub quantity: f64,
    pub average_entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

impl Position {
    pub fn new(symbol: Symbol) -> Self {
        Position {
            symbol,
            quantity: 0.0,
            average_entry_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    pub fn side(&self) -> Option<Side> {
        if self.quantity > 0.0 {
            Some(Side::Buy)
        } else if self.quantity < 0.0 {
            Some(Side::Sell)
        } else {
            None
        }
    }

    pub fn is_flat(&self) -> bool {
        self.quantity.abs() < 1e-10
    }

    pub fn update_pnl(&mut self, current_price: f64) {
        self.unrealized_pnl = (current_price - self.average_entry_price) * self.quantity;
    }
}
```

## Event System

```rust
use tokio::sync::mpsc;
use std::time::SystemTime;

/// All events in the system
#[derive(Debug, Clone)]
pub enum Event {
    // Market data
    TickReceived(Tick),
    CandleReceived(Candle),

    // Strategy signals
    SignalGenerated(TradingSignal),

    // Orders
    OrderCreated(Order),
    OrderUpdated(Order),
    OrderFilled(Order),
    OrderCancelled(String),

    // Positions
    PositionOpened(Position),
    PositionUpdated(Position),
    PositionClosed(Position),

    // Risks
    RiskLimitExceeded(RiskAlert),

    // System
    Heartbeat,
    Shutdown,
    Error(String),
}

/// Trading signal
#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub symbol: Symbol,
    pub side: Side,
    pub strength: f64,      // 0.0 - 1.0
    pub reason: String,
    pub suggested_quantity: f64,
    pub suggested_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub timestamp: SystemTime,
}

/// Risk alert
#[derive(Debug, Clone)]
pub struct RiskAlert {
    pub alert_type: RiskAlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum RiskAlertType {
    MaxPositionExceeded,
    MaxDrawdownExceeded,
    DailyLossLimitHit,
    MaxOrdersExceeded,
    SpreadTooWide,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Event bus
pub struct EventBus {
    sender: mpsc::Sender<Event>,
    subscribers: Vec<mpsc::Sender<Event>>,
}

impl EventBus {
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<Event>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (
            EventBus {
                sender,
                subscribers: Vec::new(),
            },
            receiver,
        )
    }

    pub fn subscribe(&mut self) -> mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.push(tx);
        rx
    }

    pub async fn publish(&self, event: Event) {
        // Send to main channel
        let _ = self.sender.send(event.clone()).await;

        // Send to all subscribers
        for subscriber in &self.subscribers {
            let _ = subscriber.send(event.clone()).await;
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }
}
```

## Market Data Module

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Market data provider trait
#[async_trait::async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn subscribe(&mut self, symbols: &[Symbol]) -> Result<(), String>;
    async fn unsubscribe(&mut self, symbols: &[Symbol]) -> Result<(), String>;
    async fn disconnect(&mut self) -> Result<(), String>;
}

/// Market data aggregator
pub struct MarketDataAggregator {
    providers: Vec<Box<dyn MarketDataProvider>>,
    event_sender: mpsc::Sender<Event>,
    price_cache: Arc<RwLock<HashMap<Symbol, Tick>>>,
    candle_cache: Arc<RwLock<HashMap<(Symbol, Duration), Vec<Candle>>>>,
}

impl MarketDataAggregator {
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        MarketDataAggregator {
            providers: Vec::new(),
            event_sender,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            candle_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn MarketDataProvider>) {
        self.providers.push(provider);
    }

    pub async fn get_latest_tick(&self, symbol: &Symbol) -> Option<Tick> {
        let cache = self.price_cache.read().await;
        cache.get(symbol).cloned()
    }

    pub async fn update_tick(&self, tick: Tick) {
        // Update cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(tick.symbol.clone(), tick.clone());
        }

        // Publish event
        let _ = self.event_sender.send(Event::TickReceived(tick)).await;
    }

    pub async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Duration,
        limit: usize,
    ) -> Vec<Candle> {
        let cache = self.candle_cache.read().await;
        cache
            .get(&(symbol.clone(), timeframe))
            .map(|candles| {
                candles
                    .iter()
                    .rev()
                    .take(limit)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Simulated data provider (for testing)
pub struct SimulatedDataProvider {
    symbols: Vec<Symbol>,
    running: bool,
    event_sender: mpsc::Sender<Event>,
}

impl SimulatedDataProvider {
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        SimulatedDataProvider {
            symbols: Vec::new(),
            running: false,
            event_sender,
        }
    }

    pub async fn run(&mut self) {
        use rand::Rng;

        self.running = true;
        let mut rng = rand::thread_rng();
        let mut prices: HashMap<Symbol, f64> = self.symbols
            .iter()
            .map(|s| (s.clone(), 50000.0))
            .collect();

        while self.running {
            for symbol in &self.symbols {
                let current_price = prices.get_mut(symbol).unwrap();

                // Random price change
                let change = rng.gen_range(-0.001..0.001);
                *current_price *= 1.0 + change;

                let spread = *current_price * 0.0001; // 0.01% spread

                let tick = Tick {
                    symbol: symbol.clone(),
                    bid: *current_price - spread / 2.0,
                    ask: *current_price + spread / 2.0,
                    last_price: *current_price,
                    volume: rng.gen_range(0.1..10.0),
                    timestamp: SystemTime::now(),
                };

                let _ = self.event_sender.send(Event::TickReceived(tick)).await;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[async_trait::async_trait]
impl MarketDataProvider for SimulatedDataProvider {
    async fn connect(&mut self) -> Result<(), String> {
        println!("[SimulatedData] Connected");
        Ok(())
    }

    async fn subscribe(&mut self, symbols: &[Symbol]) -> Result<(), String> {
        self.symbols.extend(symbols.iter().cloned());
        println!("[SimulatedData] Subscribed to: {:?}", symbols);
        Ok(())
    }

    async fn unsubscribe(&mut self, symbols: &[Symbol]) -> Result<(), String> {
        self.symbols.retain(|s| !symbols.contains(s));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.running = false;
        println!("[SimulatedData] Disconnected");
        Ok(())
    }
}
```

## Strategy Module

```rust
use std::collections::VecDeque;

/// Strategy trait
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn on_tick(&mut self, tick: &Tick) -> Option<TradingSignal>;
    fn on_candle(&mut self, candle: &Candle) -> Option<TradingSignal>;
    fn reset(&mut self);
}

/// Simple SMA crossover strategy
pub struct SmaCrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
    fast_sma: VecDeque<f64>,
    slow_sma: VecDeque<f64>,
    prices: VecDeque<f64>,
    last_signal: Option<Side>,
}

impl SmaCrossoverStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        SmaCrossoverStrategy {
            fast_period,
            slow_period,
            fast_sma: VecDeque::new(),
            slow_sma: VecDeque::new(),
            prices: VecDeque::with_capacity(slow_period + 1),
            last_signal: None,
        }
    }

    fn calculate_sma(prices: &VecDeque<f64>, period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }
}

impl Strategy for SmaCrossoverStrategy {
    fn name(&self) -> &str {
        "SMA Crossover"
    }

    fn on_tick(&mut self, tick: &Tick) -> Option<TradingSignal> {
        // Add price
        self.prices.push_back(tick.last_price);
        if self.prices.len() > self.slow_period + 10 {
            self.prices.pop_front();
        }

        // Calculate SMA
        let fast = Self::calculate_sma(&self.prices, self.fast_period)?;
        let slow = Self::calculate_sma(&self.prices, self.slow_period)?;

        // Store for analysis
        self.fast_sma.push_back(fast);
        self.slow_sma.push_back(slow);
        if self.fast_sma.len() > 2 {
            self.fast_sma.pop_front();
            self.slow_sma.pop_front();
        }

        // Check for crossover
        if self.fast_sma.len() < 2 {
            return None;
        }

        let prev_fast = self.fast_sma[0];
        let prev_slow = self.slow_sma[0];
        let curr_fast = self.fast_sma[1];
        let curr_slow = self.slow_sma[1];

        // Golden cross (fast crosses slow from below)
        if prev_fast <= prev_slow && curr_fast > curr_slow {
            if self.last_signal != Some(Side::Buy) {
                self.last_signal = Some(Side::Buy);
                return Some(TradingSignal {
                    symbol: tick.symbol.clone(),
                    side: Side::Buy,
                    strength: (curr_fast - curr_slow) / curr_slow,
                    reason: format!(
                        "Golden cross: fast SMA ({:.2}) > slow SMA ({:.2})",
                        curr_fast, curr_slow
                    ),
                    suggested_quantity: 0.1,
                    suggested_price: Some(tick.ask),
                    stop_loss: Some(tick.last_price * 0.98),
                    take_profit: Some(tick.last_price * 1.04),
                    timestamp: SystemTime::now(),
                });
            }
        }

        // Death cross (fast crosses slow from above)
        if prev_fast >= prev_slow && curr_fast < curr_slow {
            if self.last_signal != Some(Side::Sell) {
                self.last_signal = Some(Side::Sell);
                return Some(TradingSignal {
                    symbol: tick.symbol.clone(),
                    side: Side::Sell,
                    strength: (curr_slow - curr_fast) / curr_slow,
                    reason: format!(
                        "Death cross: fast SMA ({:.2}) < slow SMA ({:.2})",
                        curr_fast, curr_slow
                    ),
                    suggested_quantity: 0.1,
                    suggested_price: Some(tick.bid),
                    stop_loss: Some(tick.last_price * 1.02),
                    take_profit: Some(tick.last_price * 0.96),
                    timestamp: SystemTime::now(),
                });
            }
        }

        None
    }

    fn on_candle(&mut self, _candle: &Candle) -> Option<TradingSignal> {
        // This strategy works on ticks
        None
    }

    fn reset(&mut self) {
        self.fast_sma.clear();
        self.slow_sma.clear();
        self.prices.clear();
        self.last_signal = None;
    }
}

/// Strategy manager
pub struct StrategyEngine {
    strategies: Vec<Box<dyn Strategy>>,
    event_sender: mpsc::Sender<Event>,
}

impl StrategyEngine {
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        StrategyEngine {
            strategies: Vec::new(),
            event_sender,
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        println!("[StrategyEngine] Added strategy: {}", strategy.name());
        self.strategies.push(strategy);
    }

    pub async fn process_tick(&mut self, tick: &Tick) {
        for strategy in &mut self.strategies {
            if let Some(signal) = strategy.on_tick(tick) {
                println!(
                    "[StrategyEngine] Signal from {}: {:?} {} (strength: {:.2}%)",
                    strategy.name(),
                    signal.side,
                    signal.symbol.0,
                    signal.strength * 100.0
                );
                let _ = self.event_sender.send(Event::SignalGenerated(signal)).await;
            }
        }
    }

    pub async fn process_candle(&mut self, candle: &Candle) {
        for strategy in &mut self.strategies {
            if let Some(signal) = strategy.on_candle(candle) {
                let _ = self.event_sender.send(Event::SignalGenerated(signal)).await;
            }
        }
    }
}
```

## Order Management Module

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Exchange interface
#[async_trait::async_trait]
pub trait Exchange: Send + Sync {
    async fn place_order(&self, order: &Order) -> Result<Order, String>;
    async fn cancel_order(&self, order_id: &str) -> Result<(), String>;
    async fn get_order(&self, order_id: &str) -> Result<Order, String>;
    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>, String>;
    async fn get_balance(&self, asset: &str) -> Result<f64, String>;
}

/// Order manager
pub struct OrderManager {
    exchange: Arc<dyn Exchange>,
    orders: Arc<RwLock<HashMap<String, Order>>>,
    event_sender: mpsc::Sender<Event>,
}

impl OrderManager {
    pub fn new(exchange: Arc<dyn Exchange>, event_sender: mpsc::Sender<Event>) -> Self {
        OrderManager {
            exchange,
            orders: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub async fn submit_order(&self, mut order: Order) -> Result<Order, String> {
        println!(
            "[OrderManager] Submitting order: {:?} {} {} @ {:?}",
            order.side,
            order.quantity,
            order.symbol.0,
            order.price
        );

        // Send to exchange
        let result = self.exchange.place_order(&order).await?;
        order.id = result.id.clone();
        order.status = result.status;

        // Save to cache
        {
            let mut orders = self.orders.write().await;
            orders.insert(order.id.clone(), order.clone());
        }

        // Publish event
        let _ = self.event_sender.send(Event::OrderCreated(order.clone())).await;

        Ok(order)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<(), String> {
        println!("[OrderManager] Cancelling order: {}", order_id);

        self.exchange.cancel_order(order_id).await?;

        // Update cache
        {
            let mut orders = self.orders.write().await;
            if let Some(order) = orders.get_mut(order_id) {
                order.status = OrderStatus::Cancelled;
                order.updated_at = SystemTime::now();
            }
        }

        let _ = self.event_sender.send(Event::OrderCancelled(order_id.to_string())).await;

        Ok(())
    }

    pub async fn get_open_orders(&self) -> Vec<Order> {
        let orders = self.orders.read().await;
        orders
            .values()
            .filter(|o| !o.is_complete())
            .cloned()
            .collect()
    }

    pub async fn sync_orders(&self) -> Result<(), String> {
        let exchange_orders = self.exchange.get_open_orders(None).await?;

        let mut orders = self.orders.write().await;
        for order in exchange_orders {
            orders.insert(order.id.clone(), order);
        }

        Ok(())
    }
}

/// Simulated exchange (for testing)
pub struct SimulatedExchange {
    orders: Arc<RwLock<HashMap<String, Order>>>,
    balances: Arc<RwLock<HashMap<String, f64>>>,
}

impl SimulatedExchange {
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 100000.0);
        balances.insert("BTC".to_string(), 1.0);

        SimulatedExchange {
            orders: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(balances)),
        }
    }
}

#[async_trait::async_trait]
impl Exchange for SimulatedExchange {
    async fn place_order(&self, order: &Order) -> Result<Order, String> {
        let mut new_order = order.clone();

        // For market order — instant execution
        if order.order_type == OrderType::Market {
            new_order.status = OrderStatus::Filled;
            new_order.filled_quantity = order.quantity;
            new_order.average_fill_price = order.price;
        } else {
            new_order.status = OrderStatus::Open;
        }

        new_order.updated_at = SystemTime::now();

        let mut orders = self.orders.write().await;
        orders.insert(new_order.id.clone(), new_order.clone());

        Ok(new_order)
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), String> {
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(order_id) {
            order.status = OrderStatus::Cancelled;
            Ok(())
        } else {
            Err("Order not found".to_string())
        }
    }

    async fn get_order(&self, order_id: &str) -> Result<Order, String> {
        let orders = self.orders.read().await;
        orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| "Order not found".to_string())
    }

    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>, String> {
        let orders = self.orders.read().await;
        Ok(orders
            .values()
            .filter(|o| {
                !o.is_complete() && symbol.map(|s| &o.symbol == s).unwrap_or(true)
            })
            .cloned()
            .collect())
    }

    async fn get_balance(&self, asset: &str) -> Result<f64, String> {
        let balances = self.balances.read().await;
        Ok(*balances.get(asset).unwrap_or(&0.0))
    }
}
```

## Risk Management Module

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Risk management configuration
#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_size: f64,          // Max position size
    pub max_daily_loss: f64,             // Max daily loss
    pub max_drawdown_pct: f64,           // Max drawdown in %
    pub max_orders_per_minute: u32,      // Max orders per minute
    pub min_order_interval_ms: u64,      // Min interval between orders
    pub max_spread_pct: f64,             // Max spread for entry
    pub stop_loss_pct: f64,              // Required stop-loss
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_size: 1.0,
            max_daily_loss: 1000.0,
            max_drawdown_pct: 10.0,
            max_orders_per_minute: 10,
            min_order_interval_ms: 1000,
            max_spread_pct: 0.1,
            stop_loss_pct: 2.0,
        }
    }
}

/// Risk manager
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: Arc<RwLock<f64>>,
    peak_balance: Arc<RwLock<f64>>,
    orders_this_minute: Arc<RwLock<u32>>,
    last_order_time: Arc<RwLock<SystemTime>>,
    event_sender: mpsc::Sender<Event>,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_balance: f64, event_sender: mpsc::Sender<Event>) -> Self {
        RiskManager {
            config,
            daily_pnl: Arc::new(RwLock::new(0.0)),
            peak_balance: Arc::new(RwLock::new(initial_balance)),
            orders_this_minute: Arc::new(RwLock::new(0)),
            last_order_time: Arc::new(RwLock::new(SystemTime::UNIX_EPOCH)),
            event_sender,
        }
    }

    /// Checks if position can be opened
    pub async fn check_order(&self, signal: &TradingSignal, tick: &Tick) -> Result<(), RiskAlert> {
        // Check spread
        if tick.spread_pct() > self.config.max_spread_pct {
            return Err(RiskAlert {
                alert_type: RiskAlertType::SpreadTooWide,
                message: format!(
                    "Spread {:.3}% exceeds limit {:.3}%",
                    tick.spread_pct(),
                    self.config.max_spread_pct
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Check position size
        if signal.suggested_quantity > self.config.max_position_size {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxPositionExceeded,
                message: format!(
                    "Position size {} exceeds limit {}",
                    signal.suggested_quantity, self.config.max_position_size
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Check daily loss
        let daily_pnl = *self.daily_pnl.read().await;
        if daily_pnl < -self.config.max_daily_loss {
            return Err(RiskAlert {
                alert_type: RiskAlertType::DailyLossLimitHit,
                message: format!(
                    "Daily loss ${:.2} exceeds limit ${:.2}",
                    -daily_pnl, self.config.max_daily_loss
                ),
                severity: AlertSeverity::Critical,
                timestamp: SystemTime::now(),
            });
        }

        // Check order frequency
        let orders = *self.orders_this_minute.read().await;
        if orders >= self.config.max_orders_per_minute {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxOrdersExceeded,
                message: format!(
                    "Orders per minute limit exceeded: {} >= {}",
                    orders, self.config.max_orders_per_minute
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Check interval between orders
        let last_order = *self.last_order_time.read().await;
        if let Ok(elapsed) = last_order.elapsed() {
            if elapsed.as_millis() < self.config.min_order_interval_ms as u128 {
                return Err(RiskAlert {
                    alert_type: RiskAlertType::MaxOrdersExceeded,
                    message: format!(
                        "Too fast: {}ms elapsed, minimum {}ms",
                        elapsed.as_millis(),
                        self.config.min_order_interval_ms
                    ),
                    severity: AlertSeverity::Warning,
                    timestamp: SystemTime::now(),
                });
            }
        }

        // Check for stop-loss
        if signal.stop_loss.is_none() {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxPositionExceeded,
                message: "Order without stop-loss is prohibited".to_string(),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        Ok(())
    }

    /// Records order execution
    pub async fn record_order(&self) {
        *self.orders_this_minute.write().await += 1;
        *self.last_order_time.write().await = SystemTime::now();
    }

    /// Updates P&L
    pub async fn update_pnl(&self, pnl_change: f64) {
        *self.daily_pnl.write().await += pnl_change;
    }

    /// Checks drawdown
    pub async fn check_drawdown(&self, current_balance: f64) -> Option<RiskAlert> {
        let peak = *self.peak_balance.read().await;

        if current_balance > peak {
            *self.peak_balance.write().await = current_balance;
            return None;
        }

        let drawdown_pct = (peak - current_balance) / peak * 100.0;

        if drawdown_pct > self.config.max_drawdown_pct {
            return Some(RiskAlert {
                alert_type: RiskAlertType::MaxDrawdownExceeded,
                message: format!(
                    "Drawdown {:.2}% exceeds limit {:.2}%",
                    drawdown_pct, self.config.max_drawdown_pct
                ),
                severity: AlertSeverity::Critical,
                timestamp: SystemTime::now(),
            });
        }

        None
    }

    /// Resets daily statistics
    pub async fn reset_daily(&self) {
        *self.daily_pnl.write().await = 0.0;
        *self.orders_this_minute.write().await = 0;
    }
}
```

## Main Coordinator — TradingBot

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

/// Trading bot configuration
#[derive(Debug, Clone)]
pub struct BotConfig {
    pub symbols: Vec<Symbol>,
    pub risk_config: RiskConfig,
    pub dry_run: bool,             // Simulation mode
    pub auto_trade: bool,          // Automatic trading
}

/// Main trading bot class
pub struct TradingBot {
    config: BotConfig,
    market_data: Arc<MarketDataAggregator>,
    strategy_engine: Arc<tokio::sync::Mutex<StrategyEngine>>,
    order_manager: Arc<OrderManager>,
    risk_manager: Arc<RiskManager>,
    event_sender: mpsc::Sender<Event>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl TradingBot {
    pub fn new(
        config: BotConfig,
        exchange: Arc<dyn Exchange>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        let market_data = Arc::new(MarketDataAggregator::new(event_sender.clone()));
        let strategy_engine = Arc::new(tokio::sync::Mutex::new(
            StrategyEngine::new(event_sender.clone())
        ));
        let order_manager = Arc::new(OrderManager::new(exchange, event_sender.clone()));
        let risk_manager = Arc::new(RiskManager::new(
            config.risk_config.clone(),
            100000.0, // Initial balance
            event_sender.clone(),
        ));

        TradingBot {
            config,
            market_data,
            strategy_engine,
            order_manager,
            risk_manager,
            event_sender,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn add_strategy(&self, strategy: Box<dyn Strategy>) {
        // Adding strategy requires lock
        let engine = self.strategy_engine.clone();
        tokio::spawn(async move {
            engine.lock().await.add_strategy(strategy);
        });
    }

    /// Main event loop
    pub async fn run(&self, mut event_rx: mpsc::Receiver<Event>) {
        use std::sync::atomic::Ordering;

        self.running.store(true, Ordering::SeqCst);
        println!("[TradingBot] Starting main loop");

        while self.running.load(Ordering::SeqCst) {
            match event_rx.recv().await {
                Some(event) => {
                    self.handle_event(event).await;
                }
                None => {
                    println!("[TradingBot] Event channel closed");
                    break;
                }
            }
        }

        println!("[TradingBot] Stopped");
    }

    async fn handle_event(&self, event: Event) {
        match event {
            Event::TickReceived(tick) => {
                self.on_tick(tick).await;
            }
            Event::SignalGenerated(signal) => {
                self.on_signal(signal).await;
            }
            Event::OrderFilled(order) => {
                self.on_order_filled(order).await;
            }
            Event::RiskLimitExceeded(alert) => {
                self.on_risk_alert(alert).await;
            }
            Event::Shutdown => {
                self.running.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            Event::Heartbeat => {
                // System health check
            }
            _ => {}
        }
    }

    async fn on_tick(&self, tick: Tick) {
        // Update cache
        self.market_data.update_tick(tick.clone()).await;

        // Process strategies
        let mut engine = self.strategy_engine.lock().await;
        engine.process_tick(&tick).await;
    }

    async fn on_signal(&self, signal: TradingSignal) {
        println!(
            "[TradingBot] Signal received: {:?} {} (strength: {:.2}%)",
            signal.side,
            signal.symbol.0,
            signal.strength * 100.0
        );

        // Get current tick for validation
        let tick = match self.market_data.get_latest_tick(&signal.symbol).await {
            Some(t) => t,
            None => {
                println!("[TradingBot] No data for {}", signal.symbol.0);
                return;
            }
        };

        // Check risks
        if let Err(alert) = self.risk_manager.check_order(&signal, &tick).await {
            println!("[TradingBot] Risk control: {}", alert.message);
            let _ = self.event_sender.send(Event::RiskLimitExceeded(alert)).await;
            return;
        }

        // Create and submit order
        if self.config.auto_trade {
            let order = match signal.suggested_price {
                Some(price) => Order::new_limit(
                    signal.symbol.clone(),
                    signal.side,
                    signal.suggested_quantity,
                    price,
                ),
                None => Order::new_market(
                    signal.symbol.clone(),
                    signal.side,
                    signal.suggested_quantity,
                ),
            };

            match self.order_manager.submit_order(order).await {
                Ok(order) => {
                    self.risk_manager.record_order().await;
                    println!("[TradingBot] Order created: {}", order.id);
                }
                Err(e) => {
                    println!("[TradingBot] Error creating order: {}", e);
                }
            }
        } else {
            println!("[TradingBot] Auto-trading disabled, signal ignored");
        }
    }

    async fn on_order_filled(&self, order: Order) {
        println!(
            "[TradingBot] Order filled: {} {:?} {} @ ${:.2}",
            order.id,
            order.side,
            order.quantity,
            order.average_fill_price.unwrap_or(0.0)
        );
    }

    async fn on_risk_alert(&self, alert: RiskAlert) {
        match alert.severity {
            AlertSeverity::Critical => {
                println!("[TradingBot] CRITICAL ALERT: {}", alert.message);
                // Can stop trading here
            }
            AlertSeverity::Warning => {
                println!("[TradingBot] WARNING: {}", alert.message);
            }
            AlertSeverity::Info => {
                println!("[TradingBot] INFO: {}", alert.message);
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
```

## Usage Example

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("=== Rust Trading Bot ===\n");

    // Create event channels
    let (event_tx, event_rx) = mpsc::channel::<Event>(1000);

    // Create exchange simulator
    let exchange = Arc::new(SimulatedExchange::new());

    // Bot configuration
    let config = BotConfig {
        symbols: vec![
            Symbol::new("BTCUSDT"),
            Symbol::new("ETHUSDT"),
        ],
        risk_config: RiskConfig {
            max_position_size: 0.5,
            max_daily_loss: 500.0,
            max_drawdown_pct: 5.0,
            max_orders_per_minute: 5,
            min_order_interval_ms: 2000,
            max_spread_pct: 0.05,
            stop_loss_pct: 2.0,
        },
        dry_run: true,
        auto_trade: true,
    };

    // Create bot
    let bot = Arc::new(TradingBot::new(config.clone(), exchange, event_tx.clone()));

    // Add strategy
    {
        let strategy = SmaCrossoverStrategy::new(5, 20);
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            // Small delay for initialization
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            bot_clone.add_strategy(Box::new(strategy));
        });
    }

    // Start data simulator
    let data_tx = event_tx.clone();
    let symbols = config.symbols.clone();
    let data_handle = tokio::spawn(async move {
        let mut provider = SimulatedDataProvider::new(data_tx);
        provider.subscribe(&symbols).await.unwrap();
        provider.run().await;
    });

    // Start main bot loop
    let bot_clone = bot.clone();
    let bot_handle = tokio::spawn(async move {
        bot_clone.run(event_rx).await;
    });

    // Let system run
    println!("Bot started, running for 10 seconds...\n");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Stop
    println!("\nStopping bot...");
    bot.stop();
    let _ = event_tx.send(Event::Shutdown).await;

    // Wait for completion
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        bot_handle
    ).await;

    println!("\n=== Bot stopped ===");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Modular architecture** | Separating system into independent components |
| **Event-driven** | Communication through events instead of direct calls |
| **Event bus** | Centralized message passing system |
| **Traits for abstractions** | `Strategy`, `Exchange`, `MarketDataProvider` |
| **Async/await** | Asynchronous I/O operation handling |
| **Arc + RwLock** | Thread-safe shared state |
| **Risk Management** | Integration of risk control into every operation |

## Practical Exercises

1. **Add RSI Strategy**: Implement an RSI-based strategy:
   - Buy when RSI < 30 (oversold)
   - Sell when RSI > 70 (overbought)
   - Implement the `Strategy` trait

2. **Add WebSocket Provider**: Create a real data provider:
   - Connect to exchange WebSocket API
   - Parse JSON into `Tick` structures
   - Handle reconnection

3. **Add File Logging**: Implement a logging component:
   - Write all events to file
   - Log rotation by size
   - Formatting with timestamps

4. **Add Metrics Monitoring**: Create a metrics system:
   - Number of trades per period
   - Average P&L
   - Win rate
   - Time between trades

## Homework

1. **Multi-Exchange Bot**: Extend the architecture for multiple exchanges:
   - Arbitrage opportunities between exchanges
   - Liquidity aggregation
   - Position synchronization

2. **Backtesting Module**: Add ability to test on historical data:
   - Load CSV/JSON data
   - Emulate order execution
   - Results report

3. **REST API for Management**: Create an HTTP API for the bot:
   - Start/stop trading
   - Get status
   - Change parameters
   - View positions

4. **Alerts and Notifications**: Implement an alert system:
   - Telegram/Discord integration
   - Email notifications
   - Configurable alert conditions

5. **Persistence Layer**: Add state persistence:
   - Database for trade history
   - Recovery after restart
   - Operation audit

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../336-*/en.md)
