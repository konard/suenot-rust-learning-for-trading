# День 335: Архитектура торгового бота

## Аналогия из трейдинга

Представь, что ты строишь торговый дом — не простой офис, а целый комплекс с разными подразделениями:

- **Отдел данных** — получает котировки с бирж, фильтрует и нормализует
- **Аналитический отдел** — рассчитывает индикаторы и паттерны
- **Стратегический отдел** — принимает решения о сделках
- **Отдел исполнения** — отправляет ордера на биржу
- **Риск-менеджмент** — контролирует позиции и лимиты
- **Логистика** — хранит историю и состояние системы

Каждый отдел работает независимо, но они координируются через **центральную систему сообщений**. Если аналитики перегружены, данные буферизуются. Если риск-менеджмент говорит "стоп", исполнение блокируется.

**Архитектура торгового бота** в Rust повторяет эту структуру:
- **Модули** = отделы со своей ответственностью
- **Каналы (channels)** = внутренняя почта между отделами
- **Акторы** = сотрудники, обрабатывающие сообщения
- **Shared state** = общие ресурсы с контролем доступа

## Основные компоненты торгового бота

| Компонент | Ответственность | Паттерн |
|-----------|-----------------|---------|
| **Market Data** | Получение и нормализация данных | Publisher-Subscriber |
| **Strategy Engine** | Генерация сигналов | State Machine |
| **Order Manager** | Управление ордерами | Command Pattern |
| **Risk Manager** | Контроль рисков | Interceptor |
| **Position Tracker** | Учёт позиций | Observer |
| **Logger/Metrics** | Мониторинг | Decorator |

## Базовая структура проекта

```
trading-bot/
├── Cargo.toml
├── src/
│   ├── main.rs              # Точка входа
│   ├── lib.rs               # Публичное API
│   ├── config.rs            # Конфигурация
│   ├── market_data/
│   │   ├── mod.rs           # Модуль данных
│   │   ├── provider.rs      # Источники данных
│   │   └── normalizer.rs    # Нормализация
│   ├── strategy/
│   │   ├── mod.rs           # Модуль стратегий
│   │   ├── signal.rs        # Сигналы
│   │   └── indicators.rs    # Индикаторы
│   ├── execution/
│   │   ├── mod.rs           # Модуль исполнения
│   │   ├── order.rs         # Ордера
│   │   └── exchange.rs      # Интерфейс биржи
│   ├── risk/
│   │   ├── mod.rs           # Модуль рисков
│   │   └── limits.rs        # Лимиты
│   └── core/
│       ├── mod.rs           # Ядро системы
│       ├── types.rs         # Общие типы
│       └── events.rs        # События
└── tests/
    └── integration_test.rs
```

## Определение базовых типов

```rust
use std::time::{SystemTime, Duration};
use std::collections::HashMap;

/// Торговый символ (пара)
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: &str) -> Self {
        Symbol(s.to_uppercase())
    }
}

/// Сторона сделки
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

/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Статус ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Рыночные данные (тик)
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

/// OHLCV свеча
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

/// Ордер
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

/// Позиция
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

## Система событий

```rust
use tokio::sync::mpsc;
use std::time::SystemTime;

/// Все события в системе
#[derive(Debug, Clone)]
pub enum Event {
    // Рыночные данные
    TickReceived(Tick),
    CandleReceived(Candle),

    // Сигналы стратегии
    SignalGenerated(TradingSignal),

    // Ордера
    OrderCreated(Order),
    OrderUpdated(Order),
    OrderFilled(Order),
    OrderCancelled(String),

    // Позиции
    PositionOpened(Position),
    PositionUpdated(Position),
    PositionClosed(Position),

    // Риски
    RiskLimitExceeded(RiskAlert),

    // Система
    Heartbeat,
    Shutdown,
    Error(String),
}

/// Торговый сигнал
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

/// Алерт по рискам
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

/// Шина событий
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
        // Отправляем в основной канал
        let _ = self.sender.send(event.clone()).await;

        // Отправляем всем подписчикам
        for subscriber in &self.subscribers {
            let _ = subscriber.send(event.clone()).await;
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }
}
```

## Модуль рыночных данных

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Провайдер рыночных данных
#[async_trait::async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn connect(&mut self) -> Result<(), String>;
    async fn subscribe(&mut self, symbols: &[Symbol]) -> Result<(), String>;
    async fn unsubscribe(&mut self, symbols: &[Symbol]) -> Result<(), String>;
    async fn disconnect(&mut self) -> Result<(), String>;
}

/// Агрегатор рыночных данных
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
        // Обновляем кэш
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(tick.symbol.clone(), tick.clone());
        }

        // Публикуем событие
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

/// Симулятор рыночных данных (для тестирования)
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

                // Случайное изменение цены
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
        println!("[SimulatedData] Подключено");
        Ok(())
    }

    async fn subscribe(&mut self, symbols: &[Symbol]) -> Result<(), String> {
        self.symbols.extend(symbols.iter().cloned());
        println!("[SimulatedData] Подписка на: {:?}", symbols);
        Ok(())
    }

    async fn unsubscribe(&mut self, symbols: &[Symbol]) -> Result<(), String> {
        self.symbols.retain(|s| !symbols.contains(s));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.running = false;
        println!("[SimulatedData] Отключено");
        Ok(())
    }
}
```

## Модуль стратегии

```rust
use std::collections::VecDeque;

/// Трейт стратегии
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn on_tick(&mut self, tick: &Tick) -> Option<TradingSignal>;
    fn on_candle(&mut self, candle: &Candle) -> Option<TradingSignal>;
    fn reset(&mut self);
}

/// Простая SMA кроссовер стратегия
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
        // Добавляем цену
        self.prices.push_back(tick.last_price);
        if self.prices.len() > self.slow_period + 10 {
            self.prices.pop_front();
        }

        // Рассчитываем SMA
        let fast = Self::calculate_sma(&self.prices, self.fast_period)?;
        let slow = Self::calculate_sma(&self.prices, self.slow_period)?;

        // Сохраняем для анализа
        self.fast_sma.push_back(fast);
        self.slow_sma.push_back(slow);
        if self.fast_sma.len() > 2 {
            self.fast_sma.pop_front();
            self.slow_sma.pop_front();
        }

        // Проверяем пересечение
        if self.fast_sma.len() < 2 {
            return None;
        }

        let prev_fast = self.fast_sma[0];
        let prev_slow = self.slow_sma[0];
        let curr_fast = self.fast_sma[1];
        let curr_slow = self.slow_sma[1];

        // Золотой крест (fast пересекает slow снизу вверх)
        if prev_fast <= prev_slow && curr_fast > curr_slow {
            if self.last_signal != Some(Side::Buy) {
                self.last_signal = Some(Side::Buy);
                return Some(TradingSignal {
                    symbol: tick.symbol.clone(),
                    side: Side::Buy,
                    strength: (curr_fast - curr_slow) / curr_slow,
                    reason: format!(
                        "Золотой крест: fast SMA ({:.2}) > slow SMA ({:.2})",
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

        // Мёртвый крест (fast пересекает slow сверху вниз)
        if prev_fast >= prev_slow && curr_fast < curr_slow {
            if self.last_signal != Some(Side::Sell) {
                self.last_signal = Some(Side::Sell);
                return Some(TradingSignal {
                    symbol: tick.symbol.clone(),
                    side: Side::Sell,
                    strength: (curr_slow - curr_fast) / curr_slow,
                    reason: format!(
                        "Мёртвый крест: fast SMA ({:.2}) < slow SMA ({:.2})",
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
        // Эта стратегия работает на тиках
        None
    }

    fn reset(&mut self) {
        self.fast_sma.clear();
        self.slow_sma.clear();
        self.prices.clear();
        self.last_signal = None;
    }
}

/// Менеджер стратегий
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
        println!("[StrategyEngine] Добавлена стратегия: {}", strategy.name());
        self.strategies.push(strategy);
    }

    pub async fn process_tick(&mut self, tick: &Tick) {
        for strategy in &mut self.strategies {
            if let Some(signal) = strategy.on_tick(tick) {
                println!(
                    "[StrategyEngine] Сигнал от {}: {:?} {} (сила: {:.2}%)",
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

## Модуль управления ордерами

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Интерфейс биржи
#[async_trait::async_trait]
pub trait Exchange: Send + Sync {
    async fn place_order(&self, order: &Order) -> Result<Order, String>;
    async fn cancel_order(&self, order_id: &str) -> Result<(), String>;
    async fn get_order(&self, order_id: &str) -> Result<Order, String>;
    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>, String>;
    async fn get_balance(&self, asset: &str) -> Result<f64, String>;
}

/// Менеджер ордеров
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
            "[OrderManager] Отправка ордера: {:?} {} {} @ {:?}",
            order.side,
            order.quantity,
            order.symbol.0,
            order.price
        );

        // Отправляем на биржу
        let result = self.exchange.place_order(&order).await?;
        order.id = result.id.clone();
        order.status = result.status;

        // Сохраняем в кэш
        {
            let mut orders = self.orders.write().await;
            orders.insert(order.id.clone(), order.clone());
        }

        // Публикуем событие
        let _ = self.event_sender.send(Event::OrderCreated(order.clone())).await;

        Ok(order)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<(), String> {
        println!("[OrderManager] Отмена ордера: {}", order_id);

        self.exchange.cancel_order(order_id).await?;

        // Обновляем кэш
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

/// Симулятор биржи (для тестирования)
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

        // Для market ордера — мгновенное исполнение
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

## Модуль риск-менеджмента

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Конфигурация риск-менеджмента
#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_size: f64,          // Макс. размер позиции
    pub max_daily_loss: f64,             // Макс. дневной убыток
    pub max_drawdown_pct: f64,           // Макс. просадка в %
    pub max_orders_per_minute: u32,      // Макс. ордеров в минуту
    pub min_order_interval_ms: u64,      // Мин. интервал между ордерами
    pub max_spread_pct: f64,             // Макс. спред для входа
    pub stop_loss_pct: f64,              // Обязательный стоп-лосс
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

/// Риск-менеджер
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

    /// Проверяет, можно ли открыть позицию
    pub async fn check_order(&self, signal: &TradingSignal, tick: &Tick) -> Result<(), RiskAlert> {
        // Проверка спреда
        if tick.spread_pct() > self.config.max_spread_pct {
            return Err(RiskAlert {
                alert_type: RiskAlertType::SpreadTooWide,
                message: format!(
                    "Спред {:.3}% превышает лимит {:.3}%",
                    tick.spread_pct(),
                    self.config.max_spread_pct
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Проверка размера позиции
        if signal.suggested_quantity > self.config.max_position_size {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxPositionExceeded,
                message: format!(
                    "Размер позиции {} превышает лимит {}",
                    signal.suggested_quantity, self.config.max_position_size
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Проверка дневного убытка
        let daily_pnl = *self.daily_pnl.read().await;
        if daily_pnl < -self.config.max_daily_loss {
            return Err(RiskAlert {
                alert_type: RiskAlertType::DailyLossLimitHit,
                message: format!(
                    "Дневной убыток ${:.2} превышает лимит ${:.2}",
                    -daily_pnl, self.config.max_daily_loss
                ),
                severity: AlertSeverity::Critical,
                timestamp: SystemTime::now(),
            });
        }

        // Проверка частоты ордеров
        let orders = *self.orders_this_minute.read().await;
        if orders >= self.config.max_orders_per_minute {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxOrdersExceeded,
                message: format!(
                    "Превышен лимит ордеров в минуту: {} >= {}",
                    orders, self.config.max_orders_per_minute
                ),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        // Проверка интервала между ордерами
        let last_order = *self.last_order_time.read().await;
        if let Ok(elapsed) = last_order.elapsed() {
            if elapsed.as_millis() < self.config.min_order_interval_ms as u128 {
                return Err(RiskAlert {
                    alert_type: RiskAlertType::MaxOrdersExceeded,
                    message: format!(
                        "Слишком быстро: прошло {}ms, минимум {}ms",
                        elapsed.as_millis(),
                        self.config.min_order_interval_ms
                    ),
                    severity: AlertSeverity::Warning,
                    timestamp: SystemTime::now(),
                });
            }
        }

        // Проверка наличия стоп-лосса
        if signal.stop_loss.is_none() {
            return Err(RiskAlert {
                alert_type: RiskAlertType::MaxPositionExceeded,
                message: "Ордер без стоп-лосса запрещён".to_string(),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
            });
        }

        Ok(())
    }

    /// Регистрирует выполнение ордера
    pub async fn record_order(&self) {
        *self.orders_this_minute.write().await += 1;
        *self.last_order_time.write().await = SystemTime::now();
    }

    /// Обновляет P&L
    pub async fn update_pnl(&self, pnl_change: f64) {
        *self.daily_pnl.write().await += pnl_change;
    }

    /// Проверяет просадку
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
                    "Просадка {:.2}% превышает лимит {:.2}%",
                    drawdown_pct, self.config.max_drawdown_pct
                ),
                severity: AlertSeverity::Critical,
                timestamp: SystemTime::now(),
            });
        }

        None
    }

    /// Сброс дневной статистики
    pub async fn reset_daily(&self) {
        *self.daily_pnl.write().await = 0.0;
        *self.orders_this_minute.write().await = 0;
    }
}
```

## Главный координатор — TradingBot

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

/// Конфигурация торгового бота
#[derive(Debug, Clone)]
pub struct BotConfig {
    pub symbols: Vec<Symbol>,
    pub risk_config: RiskConfig,
    pub dry_run: bool,             // Режим симуляции
    pub auto_trade: bool,          // Автоматическая торговля
}

/// Главный класс торгового бота
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
            100000.0, // Начальный баланс
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
        // Добавление стратегии требует блокировки
        let engine = self.strategy_engine.clone();
        tokio::spawn(async move {
            engine.lock().await.add_strategy(strategy);
        });
    }

    /// Основной цикл обработки событий
    pub async fn run(&self, mut event_rx: mpsc::Receiver<Event>) {
        use std::sync::atomic::Ordering;

        self.running.store(true, Ordering::SeqCst);
        println!("[TradingBot] Запуск основного цикла");

        while self.running.load(Ordering::SeqCst) {
            match event_rx.recv().await {
                Some(event) => {
                    self.handle_event(event).await;
                }
                None => {
                    println!("[TradingBot] Канал событий закрыт");
                    break;
                }
            }
        }

        println!("[TradingBot] Остановлен");
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
                // Проверка здоровья системы
            }
            _ => {}
        }
    }

    async fn on_tick(&self, tick: Tick) {
        // Обновляем кэш
        self.market_data.update_tick(tick.clone()).await;

        // Обрабатываем стратегии
        let mut engine = self.strategy_engine.lock().await;
        engine.process_tick(&tick).await;
    }

    async fn on_signal(&self, signal: TradingSignal) {
        println!(
            "[TradingBot] Получен сигнал: {:?} {} (сила: {:.2}%)",
            signal.side,
            signal.symbol.0,
            signal.strength * 100.0
        );

        // Получаем текущий тик для проверки
        let tick = match self.market_data.get_latest_tick(&signal.symbol).await {
            Some(t) => t,
            None => {
                println!("[TradingBot] Нет данных для {}", signal.symbol.0);
                return;
            }
        };

        // Проверяем риски
        if let Err(alert) = self.risk_manager.check_order(&signal, &tick).await {
            println!("[TradingBot] Риск-контроль: {}", alert.message);
            let _ = self.event_sender.send(Event::RiskLimitExceeded(alert)).await;
            return;
        }

        // Создаём и отправляем ордер
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
                    println!("[TradingBot] Ордер создан: {}", order.id);
                }
                Err(e) => {
                    println!("[TradingBot] Ошибка создания ордера: {}", e);
                }
            }
        } else {
            println!("[TradingBot] Автотрейдинг выключен, сигнал проигнорирован");
        }
    }

    async fn on_order_filled(&self, order: Order) {
        println!(
            "[TradingBot] Ордер исполнен: {} {:?} {} @ ${:.2}",
            order.id,
            order.side,
            order.quantity,
            order.average_fill_price.unwrap_or(0.0)
        );
    }

    async fn on_risk_alert(&self, alert: RiskAlert) {
        match alert.severity {
            AlertSeverity::Critical => {
                println!("[TradingBot] КРИТИЧЕСКИЙ АЛЕРТ: {}", alert.message);
                // Можно остановить торговлю
            }
            AlertSeverity::Warning => {
                println!("[TradingBot] ПРЕДУПРЕЖДЕНИЕ: {}", alert.message);
            }
            AlertSeverity::Info => {
                println!("[TradingBot] ИНФО: {}", alert.message);
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
```

## Пример использования

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    println!("=== Торговый бот на Rust ===\n");

    // Создаём каналы событий
    let (event_tx, event_rx) = mpsc::channel::<Event>(1000);

    // Создаём симулятор биржи
    let exchange = Arc::new(SimulatedExchange::new());

    // Конфигурация бота
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

    // Создаём бота
    let bot = Arc::new(TradingBot::new(config.clone(), exchange, event_tx.clone()));

    // Добавляем стратегию
    {
        let strategy = SmaCrossoverStrategy::new(5, 20);
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            // Небольшая задержка для инициализации
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            bot_clone.add_strategy(Box::new(strategy));
        });
    }

    // Запускаем симулятор данных
    let data_tx = event_tx.clone();
    let symbols = config.symbols.clone();
    let data_handle = tokio::spawn(async move {
        let mut provider = SimulatedDataProvider::new(data_tx);
        provider.subscribe(&symbols).await.unwrap();
        provider.run().await;
    });

    // Запускаем основной цикл бота
    let bot_clone = bot.clone();
    let bot_handle = tokio::spawn(async move {
        bot_clone.run(event_rx).await;
    });

    // Даём системе поработать
    println!("Бот запущен, работаем 10 секунд...\n");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Останавливаем
    println!("\nОстановка бота...");
    bot.stop();
    let _ = event_tx.send(Event::Shutdown).await;

    // Ждём завершения
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        bot_handle
    ).await;

    println!("\n=== Бот остановлен ===");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Модульная архитектура** | Разделение системы на независимые компоненты |
| **Event-driven** | Коммуникация через события вместо прямых вызовов |
| **Шина событий** | Централизованная система передачи сообщений |
| **Трейты для абстракций** | `Strategy`, `Exchange`, `MarketDataProvider` |
| **Async/await** | Асинхронная обработка I/O операций |
| **Arc + RwLock** | Thread-safe разделяемое состояние |
| **Risk Management** | Интеграция контроля рисков в каждую операцию |

## Практические задания

1. **Добавь RSI стратегию**: Реализуй стратегию на базе RSI:
   - Покупка при RSI < 30 (перепроданность)
   - Продажа при RSI > 70 (перекупленность)
   - Реализуй трейт `Strategy`

2. **Добавь WebSocket провайдер**: Создай реальный провайдер данных:
   - Подключение к WebSocket API биржи
   - Парсинг JSON в `Tick` структуры
   - Обработка переподключения

3. **Добавь логирование в файл**: Реализуй компонент логирования:
   - Запись всех событий в файл
   - Ротация логов по размеру
   - Форматирование с временными метками

4. **Добавь мониторинг метрик**: Создай систему метрик:
   - Количество сделок за период
   - Средний P&L
   - Win rate
   - Время между сделками

## Домашнее задание

1. **Многобиржевой бот**: Расширь архитектуру для работы с несколькими биржами:
   - Арбитражные возможности между биржами
   - Агрегация ликвидности
   - Синхронизация позиций

2. **Бэктестинг модуль**: Добавь возможность тестирования на исторических данных:
   - Загрузка CSV/JSON данных
   - Эмуляция исполнения ордеров
   - Отчёт о результатах

3. **REST API для управления**: Создай HTTP API для бота:
   - Старт/стоп торговли
   - Получение статуса
   - Изменение параметров
   - Просмотр позиций

4. **Алерты и уведомления**: Реализуй систему оповещений:
   - Telegram/Discord интеграция
   - Email уведомления
   - Настраиваемые условия алертов

5. **Persistence слой**: Добавь сохранение состояния:
   - База данных для истории сделок
   - Восстановление после перезапуска
   - Аудит операций

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../336-*/ru.md)
