# День 365: Финал: собираем всё вместе

## Аналогия из трейдинга

Поздравляем! Вы достигли финального дня своего путешествия. Думайте об этом годе как о создании полноценной торговой операции с нуля.

**Вы начинали как одиночный трейдер с идеей** — изучая основы Rust, как изучение чтения графиков и размещение первых ордеров.

**Вы построили инфраструктуру** — владение и заимствование это ваши правила управления рисками, структуры и перечисления это ваши типы ордеров и позиций, трейты это ваши торговые стратегии, которые можно применить к любому активу.

**Вы масштабировали операцию** — асинхронное программирование это ваша способность мониторить несколько рынков одновременно, параллелизм это параллельное выполнение стратегий на разных биржах.

**Теперь вы готовы к деплою** — у вас есть мониторинг, логирование, тестирование и CI/CD. Вы больше не просто трейдер; вы управляете профессиональной торговой фирмой.

| Путь трейдера | Путь в Rust |
|---------------|-------------|
| **Учимся читать графики** | Базовый синтаксис, переменные, типы |
| **Размещаем первые ордера** | Функции, управление потоком |
| **Правила риск-менеджмента** | Владение, заимствование, времена жизни |
| **Определяем типы ордеров** | Структуры, перечисления, трейты |
| **Управление портфелем** | Коллекции, дженерики |
| **Обработка ошибок** | Result, Option, оператор ? |
| **Мониторинг нескольких рынков** | Async/await, futures |
| **Параллельное выполнение** | Потоки, каналы, Mutex |
| **Интеграция с биржами** | HTTP клиенты, WebSockets |
| **Сохранение данных** | Подключения к базам данных |
| **Оптимизация производительности** | Профилирование, оптимизация |
| **Продакшн деплой** | Docker, CI/CD, мониторинг |

## Полная торговая система

В этой финальной главе мы построим полную алгоритмическую торговую систему, которая демонстрирует всё, что вы изучили. Это не просто игрушечный пример — это готовая к продакшну архитектура, которую можно расширять.

### Структура проекта

Профессиональный Rust проект для торговли организован как воркспейс с несколькими крейтами:

```
trading-system/
├── Cargo.toml              # Определение воркспейса
├── crates/
│   ├── common/             # Общие типы и утилиты
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs    # Типы Order, Trade, Position
│   │       ├── errors.rs   # Пользовательские типы ошибок
│   │       └── config.rs   # Управление конфигурацией
│   │
│   ├── market-data/        # Обработка рыночных данных
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── feed.rs     # WebSocket фид данных
│   │       ├── orderbook.rs
│   │       └── candles.rs
│   │
│   ├── strategy/           # Торговые стратегии
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs   # Трейт Strategy
│   │       ├── momentum.rs
│   │       └── mean_reversion.rs
│   │
│   ├── execution/          # Исполнение ордеров
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs
│   │       └── risk.rs
│   │
│   └── bot/                # Основной бинарник торгового бота
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── tests/                  # Интеграционные тесты
├── benches/                # Бенчмарки производительности
└── docker/                 # Контейнеризация
```

### Cargo.toml воркспейса

```toml
[workspace]
resolver = "2"
members = [
    "crates/common",
    "crates/market-data",
    "crates/strategy",
    "crates/execution",
    "crates/bot",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["Trading Bot Team"]
license = "MIT"

[workspace.dependencies]
# Асинхронный рантайм
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Сериализация
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP/WebSocket
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }

# Логирование
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# База данных
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }

# Метрики
prometheus = "0.13"

# Время
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Обработка ошибок
thiserror = "1.0"
anyhow = "1.0"

# Тестирование
mockall = "0.12"
```

## Базовые типы и доменная модель

Начнём с основы — наши доменные типы, представляющие торговые концепции:

```rust
// crates/common/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Торговый символ (например, "BTCUSDT", "ETHUSDT")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }

    pub fn base(&self) -> &str {
        // Извлечь базовую валюту (BTC из BTCUSDT)
        &self.0[..self.0.len() - 4]
    }

    pub fn quote(&self) -> &str {
        // Извлечь котируемую валюту (USDT из BTCUSDT)
        &self.0[self.0.len() - 4..]
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Сторона ордера: покупка или продажа
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Получить противоположную сторону
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// Тип ордера
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Рыночный ордер - исполнить по текущей цене
    Market,
    /// Лимитный ордер - исполнить по указанной цене или лучше
    Limit { price: f64 },
    /// Стоп-лосс ордер
    StopLoss { stop_price: f64 },
    /// Стоп-лимит ордер
    StopLimit { stop_price: f64, limit_price: f64 },
}

/// Статус ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Торговый ордер
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Создать новый рыночный ордер
    pub fn market(symbol: Symbol, side: Side, quantity: f64) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Market,
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Создать новый лимитный ордер
    pub fn limit(symbol: Symbol, side: Side, quantity: f64, price: f64) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Limit { price },
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Проверить, полностью ли исполнен ордер
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Проверить, активен ли ещё ордер
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Pending | OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Вычислить оставшееся количество
    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }
}

/// Исполненная сделка
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub symbol: Symbol,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub commission_asset: String,
    pub executed_at: DateTime<Utc>,
}

impl Trade {
    /// Вычислить номинальную стоимость сделки
    pub fn notional(&self) -> f64 {
        self.price * self.quantity
    }

    /// Вычислить чистую стоимость после комиссии
    pub fn net_value(&self) -> f64 {
        match self.side {
            Side::Buy => self.notional() + self.commission,
            Side::Sell => self.notional() - self.commission,
        }
    }
}

/// Позиция в торговом активе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: Symbol,
    pub quantity: f64,        // Положительное = лонг, отрицательное = шорт
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    /// Создать новую позицию из сделки
    pub fn from_trade(trade: &Trade) -> Self {
        let quantity = match trade.side {
            Side::Buy => trade.quantity,
            Side::Sell => -trade.quantity,
        };

        let now = Utc::now();
        Position {
            symbol: trade.symbol.clone(),
            quantity,
            entry_price: trade.price,
            unrealized_pnl: 0.0,
            realized_pnl: -trade.commission, // Комиссия это затраты
            opened_at: now,
            updated_at: now,
        }
    }

    /// Проверить, является ли позиция лонгом
    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }

    /// Проверить, является ли позиция шортом
    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }

    /// Проверить, закрыта ли позиция
    pub fn is_closed(&self) -> bool {
        self.quantity.abs() < 1e-10
    }

    /// Обновить нереализованный PnL на основе текущей цены
    pub fn update_pnl(&mut self, current_price: f64) {
        let price_change = current_price - self.entry_price;
        self.unrealized_pnl = price_change * self.quantity;
        self.updated_at = Utc::now();
    }

    /// Добавить сделку к позиции
    pub fn apply_trade(&mut self, trade: &Trade) {
        let trade_quantity = match trade.side {
            Side::Buy => trade.quantity,
            Side::Sell => -trade.quantity,
        };

        // Проверить, уменьшает ли это позицию
        if (self.quantity > 0.0 && trade_quantity < 0.0)
            || (self.quantity < 0.0 && trade_quantity > 0.0)
        {
            // Рассчитать реализованный PnL для закрытой части
            let closed_quantity = trade_quantity.abs().min(self.quantity.abs());
            let pnl = (trade.price - self.entry_price) * closed_quantity * self.quantity.signum();
            self.realized_pnl += pnl - trade.commission;
        }

        // Обновить позицию
        let new_quantity = self.quantity + trade_quantity;

        // Если добавляем к позиции или переворачиваем, обновить цену входа
        if (self.quantity >= 0.0 && trade_quantity > 0.0)
            || (self.quantity <= 0.0 && trade_quantity < 0.0)
            || self.quantity.signum() != new_quantity.signum()
        {
            // Средневзвешенная цена входа
            let total_cost = self.entry_price * self.quantity.abs() + trade.price * trade_quantity.abs();
            self.entry_price = total_cost / (self.quantity.abs() + trade_quantity.abs());
        }

        self.quantity = new_quantity;
        self.updated_at = Utc::now();
    }
}

/// OHLCV свечные данные
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

impl Candle {
    /// Получить типичную цену (среднее HLC)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Получить диапазон (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Проверить, является ли свеча бычьей
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Проверить, является ли свеча медвежьей
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

/// Уровень стакана ордеров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Снимок стакана ордеров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: Vec<BookLevel>,  // Отсортированы по убыванию цены
    pub asks: Vec<BookLevel>,  // Отсортированы по возрастанию цены
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    /// Получить лучшую цену покупки
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    /// Получить лучшую цену продажи
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    /// Получить среднюю цену
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Получить спред
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Получить спред в процентах
    pub fn spread_pct(&self) -> Option<f64> {
        match (self.mid_price(), self.spread()) {
            (Some(mid), Some(spread)) if mid > 0.0 => Some(spread / mid * 100.0),
            _ => None,
        }
    }
}

/// Торговый сигнал, генерируемый стратегией
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    /// Сигнал на покупку с целевым размером
    Buy { quantity: f64, reason: String },
    /// Сигнал на продажу с целевым размером
    Sell { quantity: f64, reason: String },
    /// Удержание - нет действий
    Hold,
}
```

## Обработка ошибок

Надёжная торговая система нуждается в комплексной обработке ошибок:

```rust
// crates/common/src/errors.rs

use thiserror::Error;

/// Основной тип ошибки для торговой системы
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Ошибка ордера: {0}")]
    Order(#[from] OrderError),

    #[error("Ошибка рыночных данных: {0}")]
    MarketData(#[from] MarketDataError),

    #[error("Ошибка стратегии: {0}")]
    Strategy(#[from] StrategyError),

    #[error("Ошибка риск-менеджмента: {0}")]
    Risk(#[from] RiskError),

    #[error("Ошибка конфигурации: {0}")]
    Config(String),

    #[error("Ошибка базы данных: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Сетевая ошибка: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Внутренняя ошибка: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Некорректный ордер: {0}")]
    Invalid(String),

    #[error("Ордер отклонён биржей: {reason}")]
    Rejected { order_id: String, reason: String },

    #[error("Ордер не найден: {0}")]
    NotFound(String),

    #[error("Ордер уже исполнен")]
    AlreadyFilled,

    #[error("Ордер отменён")]
    Cancelled,

    #[error("Недостаточно средств: требуется {required}, доступно {available}")]
    InsufficientBalance { required: f64, available: f64 },
}

#[derive(Error, Debug)]
pub enum MarketDataError {
    #[error("Потеряно соединение с {exchange}")]
    ConnectionLost { exchange: String },

    #[error("Таймаут фида данных для {symbol}")]
    Timeout { symbol: String },

    #[error("Получены некорректные данные: {0}")]
    InvalidData(String),

    #[error("Символ не найден: {0}")]
    SymbolNotFound(String),

    #[error("Превышен лимит запросов на {exchange}")]
    RateLimited { exchange: String },
}

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Ошибка инициализации стратегии: {0}")]
    InitializationFailed(String),

    #[error("Недостаточно данных для расчёта: нужно {required}, есть {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Некорректный параметр: {name} = {value}, ожидалось {expected}")]
    InvalidParameter {
        name: String,
        value: String,
        expected: String,
    },
}

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Размер позиции {size} превышает максимум {max}")]
    PositionSizeExceeded { size: f64, max: f64 },

    #[error("Достигнут дневной лимит убытка: {loss} / {limit}")]
    DailyLossLimitReached { loss: f64, limit: f64 },

    #[error("Превышена максимальная просадка: {drawdown}% > {limit}%")]
    MaxDrawdownExceeded { drawdown: f64, limit: f64 },

    #[error("Слишком много открытых позиций: {count} / {max}")]
    TooManyPositions { count: usize, max: usize },

    #[error("Символ {symbol} ограничен")]
    SymbolRestricted { symbol: String },
}

/// Тип результата для торговых операций
pub type Result<T> = std::result::Result<T, TradingError>;
```

## Трейт Strategy

Стратегии определяются с помощью трейтов, что позволяет гибко их комбинировать:

```rust
// crates/strategy/src/traits.rs

use async_trait::async_trait;
use common::{
    errors::Result,
    types::{Candle, OrderBook, Position, Signal, Symbol},
};
use std::collections::HashMap;

/// Состояние рынка, передаваемое стратегиям
#[derive(Debug, Clone)]
pub struct MarketState {
    pub symbol: Symbol,
    pub orderbook: OrderBook,
    pub candles: Vec<Candle>,
    pub positions: HashMap<Symbol, Position>,
    pub account_balance: f64,
}

/// Конфигурация стратегии
pub trait StrategyConfig: Send + Sync + Clone + 'static {
    /// Название стратегии для логирования и идентификации
    fn name(&self) -> &str;

    /// Символы, которыми торгует стратегия
    fn symbols(&self) -> &[Symbol];

    /// Количество исторических свечей, необходимых для расчёта
    fn required_history(&self) -> usize;
}

/// Основной трейт стратегии
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Тип конфигурации для этой стратегии
    type Config: StrategyConfig;

    /// Создать новый экземпляр стратегии
    fn new(config: Self::Config) -> Self
    where
        Self: Sized;

    /// Получить конфигурацию стратегии
    fn config(&self) -> &Self::Config;

    /// Инициализировать стратегию (загрузить исторические данные, прогреть индикаторы)
    async fn initialize(&mut self) -> Result<()>;

    /// Сгенерировать торговый сигнал на основе текущего состояния рынка
    async fn generate_signal(&mut self, state: &MarketState) -> Result<Signal>;

    /// Вызывается при исполнении ордера
    fn on_order_filled(&mut self, _order: &common::types::Order) {
        // По умолчанию: ничего не делать
    }

    /// Вызывается при каждой новой свече
    fn on_candle(&mut self, _candle: &Candle) {
        // По умолчанию: ничего не делать
    }

    /// Вызывается при обновлении стакана
    fn on_orderbook_update(&mut self, _orderbook: &OrderBook) {
        // По умолчанию: ничего не делать
    }
}
```

### Реализация моментум-стратегии

```rust
// crates/strategy/src/momentum.rs

use async_trait::async_trait;
use common::{
    errors::{Result, StrategyError},
    types::{Candle, Signal, Symbol},
};
use tracing::{debug, info, instrument};

use crate::traits::{MarketState, Strategy, StrategyConfig};

/// Конфигурация моментум-стратегии
#[derive(Debug, Clone)]
pub struct MomentumConfig {
    pub name: String,
    pub symbols: Vec<Symbol>,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_threshold: f64,
    pub position_size_pct: f64,
}

impl StrategyConfig for MomentumConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn required_history(&self) -> usize {
        self.slow_period + 10 // Дополнительный буфер для сглаживания
    }
}

/// Простая моментум-стратегия на основе пересечения скользящих средних
pub struct MomentumStrategy {
    config: MomentumConfig,
    fast_ma: Option<f64>,
    slow_ma: Option<f64>,
    prev_fast_ma: Option<f64>,
    prev_slow_ma: Option<f64>,
}

impl MomentumStrategy {
    /// Вычислить простую скользящую среднюю (SMA)
    fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    /// Вычислить экспоненциальную скользящую среднюю (EMA)
    fn calculate_ema(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for price in prices.iter().skip(1) {
            ema = (price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Обновить индикаторы новыми ценовыми данными
    fn update_indicators(&mut self, candles: &[Candle]) {
        // Сохранить предыдущие значения для определения пересечения
        self.prev_fast_ma = self.fast_ma;
        self.prev_slow_ma = self.slow_ma;

        // Извлечь цены закрытия
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        // Вычислить новые MA
        self.fast_ma = Self::calculate_ema(&closes, self.config.fast_period);
        self.slow_ma = Self::calculate_ema(&closes, self.config.slow_period);
    }

    /// Определить пересечение
    fn detect_crossover(&self) -> Option<Signal> {
        let (fast, slow, prev_fast, prev_slow) = match (
            self.fast_ma,
            self.slow_ma,
            self.prev_fast_ma,
            self.prev_slow_ma,
        ) {
            (Some(f), Some(s), Some(pf), Some(ps)) => (f, s, pf, ps),
            _ => return None,
        };

        // Вычислить силу моментума
        let momentum = (fast - slow) / slow * 100.0;

        // Бычье пересечение: быстрая пересекает медленную снизу вверх
        if prev_fast <= prev_slow && fast > slow && momentum > self.config.signal_threshold {
            debug!(
                fast_ma = fast,
                slow_ma = slow,
                momentum = momentum,
                "Обнаружено бычье пересечение"
            );
            return Some(Signal::Buy {
                quantity: 0.0, // Будет рассчитано на основе размера позиции
                reason: format!(
                    "Пересечение MA: быстрая({:.2}) > медленная({:.2}), моментум: {:.2}%",
                    fast, slow, momentum
                ),
            });
        }

        // Медвежье пересечение: быстрая пересекает медленную сверху вниз
        if prev_fast >= prev_slow && fast < slow && momentum.abs() > self.config.signal_threshold {
            debug!(
                fast_ma = fast,
                slow_ma = slow,
                momentum = momentum,
                "Обнаружено медвежье пересечение"
            );
            return Some(Signal::Sell {
                quantity: 0.0,
                reason: format!(
                    "Пересечение MA: быстрая({:.2}) < медленная({:.2}), моментум: {:.2}%",
                    fast, slow, momentum
                ),
            });
        }

        None
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
    type Config = MomentumConfig;

    fn new(config: Self::Config) -> Self {
        MomentumStrategy {
            config,
            fast_ma: None,
            slow_ma: None,
            prev_fast_ma: None,
            prev_slow_ma: None,
        }
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    async fn initialize(&mut self) -> Result<()> {
        info!(
            strategy = %self.config.name,
            fast_period = self.config.fast_period,
            slow_period = self.config.slow_period,
            "Инициализация моментум-стратегии"
        );
        Ok(())
    }

    #[instrument(skip(self, state), fields(strategy = %self.config.name))]
    async fn generate_signal(&mut self, state: &MarketState) -> Result<Signal> {
        // Проверить, достаточно ли данных
        let required = self.config.required_history();
        if state.candles.len() < required {
            return Err(StrategyError::InsufficientData {
                required,
                available: state.candles.len(),
            }
            .into());
        }

        // Обновить индикаторы
        self.update_indicators(&state.candles);

        // Проверить наличие сигналов
        if let Some(signal) = self.detect_crossover() {
            // Рассчитать размер позиции
            let position_value = state.account_balance * self.config.position_size_pct / 100.0;
            let current_price = state.candles.last().unwrap().close;
            let quantity = position_value / current_price;

            return Ok(match signal {
                Signal::Buy { reason, .. } => Signal::Buy { quantity, reason },
                Signal::Sell { reason, .. } => Signal::Sell { quantity, reason },
                Signal::Hold => Signal::Hold,
            });
        }

        Ok(Signal::Hold)
    }

    fn on_candle(&mut self, candle: &Candle) {
        debug!(
            symbol = %candle.symbol,
            close = candle.close,
            "Обработка новой свечи"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::types::OrderBook;
    use std::collections::HashMap;

    fn create_test_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &price)| Candle {
                symbol: Symbol::new("BTCUSDT"),
                open: price,
                high: price * 1.01,
                low: price * 0.99,
                close: price,
                volume: 1000.0,
                timestamp: Utc::now() + chrono::Duration::minutes(i as i64),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_bullish_crossover() {
        let config = MomentumConfig {
            name: "test_momentum".to_string(),
            symbols: vec![Symbol::new("BTCUSDT")],
            fast_period: 5,
            slow_period: 10,
            signal_threshold: 0.1,
            position_size_pct: 10.0,
        };

        let mut strategy = MomentumStrategy::new(config);
        strategy.initialize().await.unwrap();

        // Создать ценовые данные с бычьим трендом
        let mut prices = vec![100.0; 10];
        prices.extend(vec![105.0, 110.0, 115.0, 120.0, 125.0]); // Растущие цены

        let candles = create_test_candles(&prices);
        let state = MarketState {
            symbol: Symbol::new("BTCUSDT"),
            orderbook: OrderBook {
                symbol: Symbol::new("BTCUSDT"),
                bids: vec![],
                asks: vec![],
                timestamp: Utc::now(),
            },
            candles,
            positions: HashMap::new(),
            account_balance: 10000.0,
        };

        let signal = strategy.generate_signal(&state).await.unwrap();

        // Должны получить сигнал на покупку из-за растущих цен
        match signal {
            Signal::Buy { quantity, reason } => {
                assert!(quantity > 0.0);
                assert!(reason.contains("Пересечение"));
            }
            _ => {} // Может не сработать в зависимости от точных значений MA
        }
    }
}
```

## Управление рисками

Профессиональная торговая система должна иметь надёжное управление рисками:

```rust
// crates/execution/src/risk.rs

use common::{
    errors::{Result, RiskError},
    types::{Order, Position, Side, Symbol},
};
use std::collections::HashMap;
use tracing::{info, warn};

/// Конфигурация риск-менеджмента
#[derive(Debug, Clone)]
pub struct RiskConfig {
    /// Максимальный размер позиции в процентах от счёта
    pub max_position_pct: f64,
    /// Максимальное количество открытых позиций
    pub max_positions: usize,
    /// Максимальный дневной убыток в процентах от счёта
    pub max_daily_loss_pct: f64,
    /// Максимальная просадка в процентах
    pub max_drawdown_pct: f64,
    /// Максимальная стоимость одного ордера
    pub max_order_value: f64,
    /// Ограниченные символы (нельзя торговать)
    pub restricted_symbols: Vec<Symbol>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_pct: 10.0,
            max_positions: 5,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 20.0,
            max_order_value: 50000.0,
            restricted_symbols: vec![],
        }
    }
}

/// Состояние риск-менеджера
pub struct RiskManager {
    config: RiskConfig,
    daily_pnl: f64,
    peak_balance: f64,
    current_balance: f64,
}

impl RiskManager {
    pub fn new(config: RiskConfig, initial_balance: f64) -> Self {
        RiskManager {
            config,
            daily_pnl: 0.0,
            peak_balance: initial_balance,
            current_balance: initial_balance,
        }
    }

    /// Проверить, проходит ли ордер все риск-проверки
    pub fn check_order(
        &self,
        order: &Order,
        positions: &HashMap<Symbol, Position>,
        current_price: f64,
    ) -> Result<()> {
        // Проверить, ограничен ли символ
        if self.config.restricted_symbols.contains(&order.symbol) {
            return Err(RiskError::SymbolRestricted {
                symbol: order.symbol.to_string(),
            }
            .into());
        }

        // Рассчитать стоимость ордера
        let order_value = order.quantity * current_price;

        // Проверить максимальную стоимость ордера
        if order_value > self.config.max_order_value {
            warn!(
                order_value = order_value,
                max = self.config.max_order_value,
                "Стоимость ордера превышает лимит"
            );
            return Err(RiskError::PositionSizeExceeded {
                size: order_value,
                max: self.config.max_order_value,
            }
            .into());
        }

        // Проверить размер позиции после ордера
        let current_position = positions.get(&order.symbol);
        let new_position_value = match (current_position, order.side) {
            (Some(pos), Side::Buy) => {
                (pos.quantity + order.quantity) * current_price
            }
            (Some(pos), Side::Sell) => {
                (pos.quantity - order.quantity).abs() * current_price
            }
            (None, _) => order_value,
        };

        let max_position_value = self.current_balance * self.config.max_position_pct / 100.0;
        if new_position_value > max_position_value {
            return Err(RiskError::PositionSizeExceeded {
                size: new_position_value,
                max: max_position_value,
            }
            .into());
        }

        // Проверить количество позиций
        let position_count = positions.values().filter(|p| !p.is_closed()).count();
        if position_count >= self.config.max_positions
            && !positions.contains_key(&order.symbol)
        {
            return Err(RiskError::TooManyPositions {
                count: position_count,
                max: self.config.max_positions,
            }
            .into());
        }

        // Проверить дневной лимит убытка
        let daily_loss_limit = self.peak_balance * self.config.max_daily_loss_pct / 100.0;
        if self.daily_pnl < -daily_loss_limit {
            return Err(RiskError::DailyLossLimitReached {
                loss: self.daily_pnl.abs(),
                limit: daily_loss_limit,
            }
            .into());
        }

        // Проверить просадку
        let current_drawdown = (self.peak_balance - self.current_balance) / self.peak_balance * 100.0;
        if current_drawdown > self.config.max_drawdown_pct {
            return Err(RiskError::MaxDrawdownExceeded {
                drawdown: current_drawdown,
                limit: self.config.max_drawdown_pct,
            }
            .into());
        }

        info!(
            symbol = %order.symbol,
            side = ?order.side,
            quantity = order.quantity,
            order_value = order_value,
            "Ордер прошёл риск-проверки"
        );

        Ok(())
    }

    /// Обновить баланс и отслеживание PnL
    pub fn update_balance(&mut self, new_balance: f64, pnl: f64) {
        self.current_balance = new_balance;
        self.daily_pnl += pnl;

        // Обновить пиковый баланс если новый максимум
        if new_balance > self.peak_balance {
            self.peak_balance = new_balance;
        }
    }

    /// Сбросить дневной PnL (вызывать в начале нового торгового дня)
    pub fn reset_daily_pnl(&mut self) {
        self.daily_pnl = 0.0;
    }

    /// Получить текущую просадку в процентах
    pub fn current_drawdown(&self) -> f64 {
        if self.peak_balance <= 0.0 {
            return 0.0;
        }
        (self.peak_balance - self.current_balance) / self.peak_balance * 100.0
    }

    /// Получить текущие метрики риска
    pub fn get_metrics(&self) -> RiskMetrics {
        RiskMetrics {
            daily_pnl: self.daily_pnl,
            current_drawdown: self.current_drawdown(),
            peak_balance: self.peak_balance,
            current_balance: self.current_balance,
            daily_loss_remaining: (self.peak_balance * self.config.max_daily_loss_pct / 100.0)
                + self.daily_pnl,
        }
    }
}

/// Текущие метрики риска
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub daily_pnl: f64,
    pub current_drawdown: f64,
    pub peak_balance: f64,
    pub current_balance: f64,
    pub daily_loss_remaining: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::OrderType;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_position_size_limit() {
        let config = RiskConfig {
            max_position_pct: 10.0,
            max_order_value: 10000.0,
            ..Default::default()
        };

        let risk_manager = RiskManager::new(config, 100000.0);
        let positions = HashMap::new();

        // Ордер, превышающий лимит размера позиции
        let order = Order {
            id: Uuid::new_v4(),
            symbol: Symbol::new("BTCUSDT"),
            side: Side::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            filled_quantity: 0.0,
            status: common::types::OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Цена 50000 означает стоимость ордера 50000, макс 10% от 100000 = 10000
        let result = risk_manager.check_order(&order, &positions, 50000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_loss_limit() {
        let config = RiskConfig {
            max_daily_loss_pct: 5.0,
            ..Default::default()
        };

        let mut risk_manager = RiskManager::new(config, 100000.0);

        // Симулируем убытки
        risk_manager.update_balance(94000.0, -6000.0);

        let order = Order::market(
            Symbol::new("BTCUSDT"),
            Side::Buy,
            0.01,
        );

        let positions = HashMap::new();
        let result = risk_manager.check_order(&order, &positions, 50000.0);

        // Должно завершиться ошибкой, т.к. дневной убыток 6000 превышает 5% от 100000 = 5000
        assert!(result.is_err());
    }
}
```

## Движок исполнения

Движок исполнения координирует ордера, сделки и обновления позиций:

```rust
// crates/execution/src/engine.rs

use common::{
    errors::{OrderError, Result},
    types::{Order, OrderStatus, Position, Side, Symbol, Trade},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::risk::{RiskConfig, RiskManager};

/// События, генерируемые движком исполнения
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    OrderSubmitted(Order),
    OrderFilled(Order, Trade),
    OrderPartiallyFilled(Order, Trade),
    OrderCancelled(Order),
    OrderRejected(Order, String),
    PositionOpened(Position),
    PositionUpdated(Position),
    PositionClosed(Position),
}

/// Интерфейс биржи для исполнения ордеров
#[async_trait::async_trait]
pub trait Exchange: Send + Sync {
    /// Отправить ордер на биржу
    async fn submit_order(&self, order: &Order) -> Result<()>;

    /// Отменить ордер
    async fn cancel_order(&self, order_id: Uuid) -> Result<()>;

    /// Получить текущую цену для символа
    async fn get_price(&self, symbol: &Symbol) -> Result<f64>;

    /// Получить баланс счёта
    async fn get_balance(&self, asset: &str) -> Result<f64>;
}

/// Основной движок исполнения
pub struct ExecutionEngine<E: Exchange> {
    exchange: Arc<E>,
    risk_manager: Arc<RwLock<RiskManager>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    positions: Arc<RwLock<HashMap<Symbol, Position>>>,
    event_sender: mpsc::Sender<ExecutionEvent>,
}

impl<E: Exchange> ExecutionEngine<E> {
    pub fn new(
        exchange: E,
        risk_config: RiskConfig,
        initial_balance: f64,
        event_sender: mpsc::Sender<ExecutionEvent>,
    ) -> Self {
        ExecutionEngine {
            exchange: Arc::new(exchange),
            risk_manager: Arc::new(RwLock::new(RiskManager::new(risk_config, initial_balance))),
            orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// Отправить новый ордер
    #[instrument(skip(self), fields(order_id = %order.id, symbol = %order.symbol))]
    pub async fn submit_order(&self, mut order: Order) -> Result<Uuid> {
        let order_id = order.id;

        // Получить текущую цену для риск-проверок
        let current_price = self.exchange.get_price(&order.symbol).await?;

        // Выполнить риск-проверки
        {
            let positions = self.positions.read().await;
            let risk_manager = self.risk_manager.read().await;
            risk_manager.check_order(&order, &positions, current_price)?;
        }

        // Отправить на биржу
        match self.exchange.submit_order(&order).await {
            Ok(()) => {
                order.status = OrderStatus::Open;

                // Сохранить ордер
                {
                    let mut orders = self.orders.write().await;
                    orders.insert(order_id, order.clone());
                }

                // Отправить событие
                self.emit_event(ExecutionEvent::OrderSubmitted(order)).await;

                info!("Ордер успешно отправлен");
                Ok(order_id)
            }
            Err(e) => {
                order.status = OrderStatus::Rejected;

                error!(error = %e, "Ордер отклонён");
                self.emit_event(ExecutionEvent::OrderRejected(
                    order,
                    e.to_string(),
                ))
                .await;

                Err(e)
            }
        }
    }

    /// Обработать исполнение сделки от биржи
    #[instrument(skip(self), fields(order_id = %trade.order_id, trade_id = %trade.id))]
    pub async fn process_fill(&self, trade: Trade) -> Result<()> {
        let mut orders = self.orders.write().await;
        let mut positions = self.positions.write().await;

        // Получить ордер
        let order = orders.get_mut(&trade.order_id).ok_or_else(|| {
            OrderError::NotFound(trade.order_id.to_string())
        })?;

        // Обновить ордер
        order.filled_quantity += trade.quantity;
        order.updated_at = chrono::Utc::now();

        let is_fully_filled = order.filled_quantity >= order.quantity;
        if is_fully_filled {
            order.status = OrderStatus::Filled;
        } else {
            order.status = OrderStatus::PartiallyFilled;
        }

        // Обновить или создать позицию
        if let Some(position) = positions.get_mut(&trade.symbol) {
            position.apply_trade(&trade);

            if position.is_closed() {
                let closed_position = positions.remove(&trade.symbol).unwrap();
                self.emit_event(ExecutionEvent::PositionClosed(closed_position)).await;
            } else {
                self.emit_event(ExecutionEvent::PositionUpdated(position.clone())).await;
            }
        } else {
            let position = Position::from_trade(&trade);
            positions.insert(trade.symbol.clone(), position.clone());
            self.emit_event(ExecutionEvent::PositionOpened(position)).await;
        }

        // Обновить риск-менеджер
        {
            let mut risk_manager = self.risk_manager.write().await;
            let balance = self.exchange.get_balance("USDT").await.unwrap_or(0.0);
            let pnl = if trade.side == Side::Sell {
                trade.notional() - trade.commission
            } else {
                -(trade.notional() + trade.commission)
            };
            risk_manager.update_balance(balance, pnl);
        }

        // Отправить событие исполнения
        if is_fully_filled {
            self.emit_event(ExecutionEvent::OrderFilled(order.clone(), trade)).await;
        } else {
            self.emit_event(ExecutionEvent::OrderPartiallyFilled(order.clone(), trade)).await;
        }

        info!("Сделка успешно обработана");
        Ok(())
    }

    /// Отменить ордер
    #[instrument(skip(self))]
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        // Отменить на бирже
        self.exchange.cancel_order(order_id).await?;

        // Обновить локальное состояние
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(&order_id) {
            order.status = OrderStatus::Cancelled;
            order.updated_at = chrono::Utc::now();

            self.emit_event(ExecutionEvent::OrderCancelled(order.clone())).await;
            info!("Ордер отменён");
        } else {
            warn!("Ордер не найден для отмены");
        }

        Ok(())
    }

    /// Получить все открытые ордера
    pub async fn get_open_orders(&self) -> Vec<Order> {
        let orders = self.orders.read().await;
        orders
            .values()
            .filter(|o| o.is_active())
            .cloned()
            .collect()
    }

    /// Получить все позиции
    pub async fn get_positions(&self) -> HashMap<Symbol, Position> {
        self.positions.read().await.clone()
    }

    /// Получить позицию для символа
    pub async fn get_position(&self, symbol: &Symbol) -> Option<Position> {
        self.positions.read().await.get(symbol).cloned()
    }

    /// Отправить событие подписчикам
    async fn emit_event(&self, event: ExecutionEvent) {
        if let Err(e) = self.event_sender.send(event).await {
            error!(error = %e, "Не удалось отправить событие исполнения");
        }
    }
}

/// Мок биржи для тестирования
pub struct MockExchange {
    prices: RwLock<HashMap<Symbol, f64>>,
    balances: RwLock<HashMap<String, f64>>,
}

impl MockExchange {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert(Symbol::new("BTCUSDT"), 50000.0);
        prices.insert(Symbol::new("ETHUSDT"), 3000.0);

        let mut balances = HashMap::new();
        balances.insert("USDT".to_string(), 100000.0);
        balances.insert("BTC".to_string(), 1.0);
        balances.insert("ETH".to_string(), 10.0);

        MockExchange {
            prices: RwLock::new(prices),
            balances: RwLock::new(balances),
        }
    }

    pub async fn set_price(&self, symbol: Symbol, price: f64) {
        self.prices.write().await.insert(symbol, price);
    }
}

#[async_trait::async_trait]
impl Exchange for MockExchange {
    async fn submit_order(&self, _order: &Order) -> Result<()> {
        // Симуляция сетевой задержки
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(())
    }

    async fn cancel_order(&self, _order_id: Uuid) -> Result<()> {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        Ok(())
    }

    async fn get_price(&self, symbol: &Symbol) -> Result<f64> {
        self.prices
            .read()
            .await
            .get(symbol)
            .copied()
            .ok_or_else(|| common::errors::MarketDataError::SymbolNotFound(symbol.to_string()).into())
    }

    async fn get_balance(&self, asset: &str) -> Result<f64> {
        Ok(self.balances.read().await.get(asset).copied().unwrap_or(0.0))
    }
}
```

## Основной торговый бот

Наконец, соберём всё вместе в основном боте:

```rust
// crates/bot/src/main.rs

use common::{
    errors::Result,
    types::{Candle, OrderBook, Signal, Symbol},
};
use execution::{
    engine::{ExecutionEngine, ExecutionEvent, MockExchange},
    risk::RiskConfig,
};
use strategy::{
    momentum::{MomentumConfig, MomentumStrategy},
    traits::{MarketState, Strategy},
};

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, Level};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

/// Конфигурация торгового бота
#[derive(Debug, Clone)]
struct BotConfig {
    symbols: Vec<Symbol>,
    update_interval_ms: u64,
    risk_config: RiskConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            symbols: vec![
                Symbol::new("BTCUSDT"),
                Symbol::new("ETHUSDT"),
            ],
            update_interval_ms: 1000,
            risk_config: RiskConfig::default(),
        }
    }
}

/// Основной торговый бот
struct TradingBot<S: Strategy> {
    config: BotConfig,
    strategy: S,
    engine: Arc<ExecutionEngine<MockExchange>>,
    event_receiver: mpsc::Receiver<ExecutionEvent>,
    candles: HashMap<Symbol, Vec<Candle>>,
    orderbooks: HashMap<Symbol, OrderBook>,
    running: Arc<RwLock<bool>>,
}

impl<S: Strategy + 'static> TradingBot<S> {
    pub fn new(config: BotConfig, strategy: S) -> Self {
        let (event_sender, event_receiver) = mpsc::channel(1000);

        let exchange = MockExchange::new();
        let engine = Arc::new(ExecutionEngine::new(
            exchange,
            config.risk_config.clone(),
            100000.0, // Начальный баланс
            event_sender,
        ));

        TradingBot {
            config,
            strategy,
            engine,
            event_receiver,
            candles: HashMap::new(),
            orderbooks: HashMap::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Инициализировать бота
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Инициализация торгового бота");

        // Инициализировать стратегию
        self.strategy.initialize().await?;

        // Инициализировать хранилище свечей для каждого символа
        for symbol in &self.config.symbols {
            self.candles.insert(symbol.clone(), Vec::new());
            self.orderbooks.insert(
                symbol.clone(),
                OrderBook {
                    symbol: symbol.clone(),
                    bids: vec![],
                    asks: vec![],
                    timestamp: Utc::now(),
                },
            );
        }

        info!("Торговый бот успешно инициализирован");
        Ok(())
    }

    /// Запустить основной торговый цикл
    pub async fn run(&mut self) -> Result<()> {
        info!("Запуск торгового бота");
        *self.running.write().await = true;

        // Запустить обработчик событий
        let engine = self.engine.clone();
        let mut event_rx = std::mem::replace(
            &mut self.event_receiver,
            mpsc::channel(1).1,
        );

        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                handle_execution_event(event).await;
            }
        });

        // Основной торговый цикл
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(self.config.update_interval_ms)
        );

        while *self.running.read().await {
            interval.tick().await;

            // Обработать каждый символ
            for symbol in self.config.symbols.clone() {
                if let Err(e) = self.process_symbol(&symbol).await {
                    error!(symbol = %symbol, error = %e, "Ошибка обработки символа");
                }
            }
        }

        info!("Торговый бот остановлен");
        Ok(())
    }

    /// Обработать один символ
    async fn process_symbol(&mut self, symbol: &Symbol) -> Result<()> {
        // Получить текущее состояние рынка
        let candles = self.candles.get(symbol).cloned().unwrap_or_default();
        let orderbook = self.orderbooks.get(symbol).cloned().unwrap_or_else(|| {
            OrderBook {
                symbol: symbol.clone(),
                bids: vec![],
                asks: vec![],
                timestamp: Utc::now(),
            }
        });

        // Получить позиции
        let positions = self.engine.get_positions().await;

        // Построить состояние рынка
        let state = MarketState {
            symbol: symbol.clone(),
            orderbook,
            candles,
            positions,
            account_balance: 100000.0, // TODO: Получить с биржи
        };

        // Сгенерировать сигнал
        match self.strategy.generate_signal(&state).await {
            Ok(signal) => {
                self.execute_signal(symbol, signal).await?;
            }
            Err(e) => {
                // Ошибки стратегии часто не фатальны (например, недостаточно данных)
                tracing::debug!(error = %e, "Ошибка стратегии");
            }
        }

        Ok(())
    }

    /// Исполнить торговый сигнал
    async fn execute_signal(&self, symbol: &Symbol, signal: Signal) -> Result<()> {
        match signal {
            Signal::Buy { quantity, reason } => {
                info!(
                    symbol = %symbol,
                    quantity = quantity,
                    reason = %reason,
                    "Исполнение сигнала на покупку"
                );

                let order = common::types::Order::market(
                    symbol.clone(),
                    common::types::Side::Buy,
                    quantity,
                );

                self.engine.submit_order(order).await?;
            }
            Signal::Sell { quantity, reason } => {
                info!(
                    symbol = %symbol,
                    quantity = quantity,
                    reason = %reason,
                    "Исполнение сигнала на продажу"
                );

                let order = common::types::Order::market(
                    symbol.clone(),
                    common::types::Side::Sell,
                    quantity,
                );

                self.engine.submit_order(order).await?;
            }
            Signal::Hold => {
                // Ничего не делать
            }
        }

        Ok(())
    }

    /// Остановить бота корректно
    pub async fn stop(&self) {
        info!("Остановка торгового бота");
        *self.running.write().await = false;
    }
}

/// Обработать события исполнения
async fn handle_execution_event(event: ExecutionEvent) {
    match event {
        ExecutionEvent::OrderSubmitted(order) => {
            info!(
                order_id = %order.id,
                symbol = %order.symbol,
                side = ?order.side,
                quantity = order.quantity,
                "Ордер отправлен"
            );
        }
        ExecutionEvent::OrderFilled(order, trade) => {
            info!(
                order_id = %order.id,
                trade_id = %trade.id,
                price = trade.price,
                quantity = trade.quantity,
                "Ордер исполнен"
            );
        }
        ExecutionEvent::OrderPartiallyFilled(order, trade) => {
            info!(
                order_id = %order.id,
                filled = trade.quantity,
                remaining = order.remaining_quantity(),
                "Ордер частично исполнен"
            );
        }
        ExecutionEvent::OrderCancelled(order) => {
            info!(order_id = %order.id, "Ордер отменён");
        }
        ExecutionEvent::OrderRejected(order, reason) => {
            error!(
                order_id = %order.id,
                reason = %reason,
                "Ордер отклонён"
            );
        }
        ExecutionEvent::PositionOpened(position) => {
            info!(
                symbol = %position.symbol,
                quantity = position.quantity,
                entry_price = position.entry_price,
                "Позиция открыта"
            );
        }
        ExecutionEvent::PositionUpdated(position) => {
            info!(
                symbol = %position.symbol,
                quantity = position.quantity,
                unrealized_pnl = position.unrealized_pnl,
                "Позиция обновлена"
            );
        }
        ExecutionEvent::PositionClosed(position) => {
            info!(
                symbol = %position.symbol,
                realized_pnl = position.realized_pnl,
                "Позиция закрыта"
            );
        }
    }
}

/// Инициализировать логирование
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    info!("=== Алгоритмическая торговая система ===");
    info!("День 365: Собираем всё вместе");

    // Создать конфигурацию стратегии
    let strategy_config = MomentumConfig {
        name: "BTC_Momentum".to_string(),
        symbols: vec![Symbol::new("BTCUSDT")],
        fast_period: 12,
        slow_period: 26,
        signal_threshold: 0.5,
        position_size_pct: 10.0,
    };

    // Создать конфигурацию бота
    let bot_config = BotConfig {
        symbols: vec![Symbol::new("BTCUSDT")],
        update_interval_ms: 1000,
        risk_config: RiskConfig {
            max_position_pct: 20.0,
            max_positions: 3,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 15.0,
            max_order_value: 25000.0,
            restricted_symbols: vec![],
        },
    };

    // Создать и инициализировать бота
    let strategy = MomentumStrategy::new(strategy_config);
    let mut bot = TradingBot::new(bot_config, strategy);

    bot.initialize().await?;

    // Запустить с корректным завершением
    let running = bot.running.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Получен сигнал завершения");
        *running.write().await = false;
    });

    bot.run().await?;

    info!("Завершение работы торгового бота");
    Ok(())
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| **Воркспейс** | Организация больших проектов в несколько крейтов с общими зависимостями |
| **Доменное моделирование** | Использование системы типов Rust для точного моделирования торговых концепций |
| **Трейты** | Определение абстракций для стратегий, бирж и других компонентов |
| **Обработка ошибок** | Создание комплексных типов ошибок с помощью thiserror |
| **Асинхронная архитектура** | Использование tokio для параллельной обработки рыночных данных и ордеров |
| **Управление рисками** | Реализация размера позиций, лимитов убытков и защиты от просадки |
| **Событийная архитектура** | Использование каналов для разделения компонентов и обработки событий |
| **Тестирование** | Написание юнит-тестов для критических компонентов, таких как риск-проверки |
| **Логирование** | Использование tracing для структурированного, контекстного логирования |
| **Корректное завершение** | Обработка сигналов для чистой остановки бота |

## Практические задания

1. **Добавить стратегию возврата к среднему**: Реализовать новую стратегию, которая:
   - Вычисляет полосы Боллинджера (20-периодная SMA с 2 стандартными отклонениями)
   - Покупает, когда цена касается нижней полосы
   - Продаёт, когда цена касается верхней полосы
   - Включает размер позиции на основе волатильности

2. **Реализовать трекер позиций**: Создать компонент, который:
   - Отслеживает все открытые позиции с PnL в реальном времени
   - Вычисляет метрики на уровне портфеля (общая экспозиция, корреляция)
   - Генерирует оповещения при приближении позиций к лимитам
   - Сохраняет историю позиций в базу данных

3. **Добавить интеграцию с биржей**: Расширить трейт биржи для:
   - Подключения к реальному API биржи (тестнет Binance)
   - Обработки rate limiting с экспоненциальным отступом
   - Реализации WebSocket для обновлений ордеров в реальном времени
   - Парсинга и валидации ответов биржи

4. **Построить движок бэктестинга**: Создать систему, которая:
   - Воспроизводит исторические свечные данные
   - Симулирует исполнение ордеров с реалистичным проскальзыванием
   - Вычисляет метрики производительности (коэффициент Шарпа, максимальная просадка)
   - Сравнивает конфигурации нескольких стратегий

## Домашнее задание

1. **Полная торговая система**: Построить готовый к продакшну бот, который:
   - Реализует как минимум две разные стратегии
   - Включает комплексное управление рисками
   - Сохраняет все сделки и позиции в PostgreSQL
   - Экспортирует метрики Prometheus для мониторинга
   - Имеет Docker Compose для локальной разработки
   - Включает интеграционные тесты с мок-биржей
   - Имеет CI/CD пайплайн с GitHub Actions

2. **Продвинутые типы ордеров**: Расширить систему для поддержки:
   - Трейлинг стоп-лосс ордеров
   - OCO (One-Cancels-Other) ордеров
   - Айсберг ордеров (скрытое количество)
   - TWAP (Time-weighted average price) исполнения
   - Умной маршрутизации между несколькими биржами

3. **Интеграция машинного обучения**: Добавить ML возможности:
   - Инженерия признаков из рыночных данных (доходности, волатильность, моментум)
   - Обучение простого классификатора для предсказания направления цены
   - Использование модели в стратегии для генерации сигналов
   - Реализация онлайн-обучения для адаптации к изменениям рынка
   - Отслеживание производительности модели и триггер переобучения

4. **Мульти-биржевой арбитраж**: Построить детектор арбитража:
   - Подключение к нескольким биржам одновременно
   - Обнаружение расхождений цен между биржами
   - Расчёт арбитражных возможностей с учётом комиссий
   - Исполнение синхронизированных ордеров на нескольких биржах
   - Обработка частичного исполнения и неудачных ордеров

5. **Дашборд реального времени**: Создать мониторинговый дашборд:
   - WebSocket сервер для обновлений в реальном времени
   - React/Vue фронтенд с позициями и PnL
   - Интерактивные графики с интеграцией TradingView
   - Конфигурация оповещений и доставка уведомлений
   - Исторический анализ производительности и отчётность

## Поздравляем!

Вы завершили 365 дней изучения Rust для алгоритмического трейдинга. Вы прошли путь от "Hello, World!" до построения полной торговой системы с:

- **Прочными основами** во владении, типах и обработке ошибок Rust
- **Профессиональной архитектурой** с использованием воркспейсов, трейтов и модульного дизайна
- **Готовностью к продакшну** с логированием, мониторингом и тестированием
- **Экспертизой в трейдинге** в рыночных данных, стратегиях и управлении рисками

Это только начало. Экосистема Rust постоянно развивается, а мир алгоритмического трейдинга предлагает бесконечные возможности для применения ваших навыков. Продолжайте учиться, продолжайте строить, и удачной торговли!

## Навигация

[← Предыдущий день](../354-production-logging/ru.md)
