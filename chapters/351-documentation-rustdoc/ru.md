# День 351: Документация: rustdoc

## Аналогия из трейдинга

Представь, что ты создал сложную торговую систему. Она работает отлично, приносит прибыль, но через полгода ты возвращаешься к коду и не можешь понять:
- Что делает функция `calculate_signal()`?
- Какие параметры принимает `execute_order()`?
- Почему в `risk_manager` такая сложная логика?

Это как торговая стратегия без журнала сделок — невозможно понять, почему принимались те или иные решения.

**Rustdoc** — это встроенный инструмент Rust для создания документации:
- Автоматически генерирует HTML-документацию из кода
- Поддерживает примеры кода, которые проверяются при компиляции
- Создаёт навигацию по модулям, структурам и функциям

Как API-документация брокера: без неё невозможно понять, какие запросы отправлять и какие ответы ожидать.

## Зачем документировать код?

В профессиональном алготрейдинге документация — это:

| Причина | Описание | Пример |
|---------|----------|--------|
| **Для себя** | Вспомнить логику через месяц | Почему стоп-лосс 2.5%? |
| **Для команды** | Другие разработчики поймут API | Новый разработчик быстро включится |
| **Для аудита** | Регулятор требует понимания системы | Объяснить логику принятия решений |
| **Для тестов** | Примеры в документации — это тесты | Автоматическая проверка корректности |

## Основы rustdoc

### Документационные комментарии

В Rust есть три типа комментариев:

```rust
// Обычный комментарий — не попадает в документацию

/// Документационный комментарий для следующего элемента
/// Используется для функций, структур, модулей

//! Документационный комментарий для текущего модуля
//! Обычно в начале файла lib.rs или mod.rs
```

### Документирование торговой структуры

```rust
/// Торговый ордер для отправки на биржу.
///
/// Представляет заявку на покупку или продажу актива с заданными параметрами.
/// Используется для взаимодействия с биржевым API.
///
/// # Примеры
///
/// ```
/// use trading_lib::Order;
///
/// let order = Order::new("BTCUSDT", Side::Buy, 0.1, 50000.0);
/// assert_eq!(order.symbol(), "BTCUSDT");
/// ```
///
/// # Поля
///
/// * `symbol` - Торговая пара (например, "BTCUSDT")
/// * `side` - Направление сделки (покупка или продажа)
/// * `quantity` - Количество актива
/// * `price` - Цена исполнения
#[derive(Debug, Clone)]
pub struct Order {
    /// Торговая пара (например, "BTCUSDT", "ETHUSDT")
    pub symbol: String,
    /// Направление сделки
    pub side: Side,
    /// Количество актива для торговли
    pub quantity: f64,
    /// Лимитная цена исполнения (None для рыночных ордеров)
    pub price: Option<f64>,
    /// Время создания ордера
    pub created_at: u64,
}

/// Направление торговой операции.
///
/// Определяет, покупаем мы актив или продаём.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    /// Покупка актива (открытие длинной позиции)
    Buy,
    /// Продажа актива (открытие короткой позиции или закрытие длинной)
    Sell,
}

impl Order {
    /// Создаёт новый лимитный ордер.
    ///
    /// # Аргументы
    ///
    /// * `symbol` - Торговая пара (например, "BTCUSDT")
    /// * `side` - Направление сделки
    /// * `quantity` - Количество актива
    /// * `price` - Лимитная цена исполнения
    ///
    /// # Примеры
    ///
    /// ```
    /// let buy_order = Order::new("BTCUSDT", Side::Buy, 0.5, 45000.0);
    /// let sell_order = Order::new("ETHUSDT", Side::Sell, 2.0, 3000.0);
    /// ```
    ///
    /// # Паника
    ///
    /// Функция не паникует, но возвращает ордер с нулевым timestamp.
    pub fn new(symbol: &str, side: Side, quantity: f64, price: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            side,
            quantity,
            price: Some(price),
            created_at: 0, // В реальности: текущий timestamp
        }
    }

    /// Создаёт рыночный ордер (исполняется по текущей цене).
    ///
    /// Рыночные ордера исполняются немедленно по лучшей доступной цене.
    /// Используются когда важна скорость исполнения, а не точная цена.
    ///
    /// # Примеры
    ///
    /// ```
    /// let market_order = Order::market("BTCUSDT", Side::Buy, 1.0);
    /// assert!(market_order.price.is_none());
    /// ```
    ///
    /// # Предупреждение
    ///
    /// На волатильном рынке цена исполнения может значительно
    /// отличаться от ожидаемой (проскальзывание).
    pub fn market(symbol: &str, side: Side, quantity: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            side,
            quantity,
            price: None,
            created_at: 0,
        }
    }

    /// Возвращает торговую пару ордера.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

fn main() {
    let order = Order::new("BTCUSDT", Side::Buy, 0.5, 50000.0);
    println!("Создан ордер: {:?}", order);
}
```

## Секции документации

Rustdoc поддерживает специальные секции:

```rust
/// Рассчитывает индикатор RSI (Relative Strength Index).
///
/// RSI измеряет скорость и изменение ценовых движений.
/// Значения выше 70 указывают на перекупленность,
/// ниже 30 — на перепроданность.
///
/// # Аргументы
///
/// * `prices` - Слайс цен закрытия
/// * `period` - Период расчёта (обычно 14)
///
/// # Возвращает
///
/// Вектор значений RSI. Первые `period` значений будут None,
/// так как для расчёта нужна история.
///
/// # Примеры
///
/// ```
/// let prices = vec![44.0, 44.5, 44.2, 44.8, 45.1, 45.3, 45.0];
/// let rsi = calculate_rsi(&prices, 14);
/// ```
///
/// # Ошибки
///
/// Возвращает пустой вектор, если:
/// * `prices` содержит меньше `period + 1` элементов
/// * `period` равен нулю
///
/// # Паника
///
/// Функция не паникует.
///
/// # Безопасность
///
/// Функция безопасна для использования в многопоточном контексте.
///
/// # Производительность
///
/// Сложность: O(n), где n — количество цен.
/// Использует один проход по данным.
///
/// # См. также
///
/// * [`calculate_macd`] - Другой популярный индикатор
/// * [`calculate_bollinger_bands`] - Полосы Боллинджера
pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    if prices.len() < period + 1 || period == 0 {
        return vec![];
    }

    let mut result = vec![None; period];

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    if gains.len() < period {
        return result;
    }

    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };
        result.push(Some(rsi));
    }

    result
}

fn main() {
    let prices = vec![44.0, 44.5, 44.2, 44.8, 45.1, 45.3, 45.0, 44.8, 44.5, 44.9,
                      45.2, 45.5, 45.8, 46.0, 45.7, 45.5, 45.3, 45.1, 45.4, 45.6];
    let rsi = calculate_rsi(&prices, 14);

    println!("RSI values:");
    for (i, value) in rsi.iter().enumerate() {
        if let Some(v) = value {
            println!("  Day {}: {:.2}", i + 1, v);
        }
    }
}
```

## Документация модулей

Для модулей используется `//!`:

```rust
//! # Торговая библиотека
//!
//! Эта библиотека предоставляет инструменты для алгоритмической торговли.
//!
//! ## Основные возможности
//!
//! * Технические индикаторы (RSI, MACD, Bollinger Bands)
//! * Управление ордерами
//! * Риск-менеджмент
//! * Бэктестинг стратегий
//!
//! ## Быстрый старт
//!
//! ```rust
//! use trading_lib::{Order, Side, RiskManager};
//!
//! // Создаём ордер
//! let order = Order::new("BTCUSDT", Side::Buy, 0.1, 50000.0);
//!
//! // Проверяем риски
//! let risk_manager = RiskManager::new(10000.0, 0.02);
//! if risk_manager.approve(&order) {
//!     println!("Ордер одобрен!");
//! }
//! ```
//!
//! ## Модули
//!
//! * [`indicators`] - Технические индикаторы
//! * [`orders`] - Управление ордерами
//! * [`risk`] - Управление рисками
//!
//! ## Зависимости
//!
//! Библиотека использует минимум зависимостей для высокой производительности.

/// Модуль технических индикаторов.
///
/// Содержит реализации популярных индикаторов для анализа рынка:
/// * RSI — индекс относительной силы
/// * MACD — схождение/расхождение скользящих средних
/// * Bollinger Bands — полосы волатильности
pub mod indicators {
    /// Рассчитывает простую скользящую среднюю (SMA).
    ///
    /// # Примеры
    ///
    /// ```
    /// let prices = vec![10.0, 11.0, 12.0, 11.5, 12.5];
    /// let sma = calculate_sma(&prices, 3);
    /// ```
    pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.len() < period || period == 0 {
            return vec![];
        }

        prices
            .windows(period)
            .map(|window| window.iter().sum::<f64>() / period as f64)
            .collect()
    }

    /// Рассчитывает экспоненциальную скользящую среднюю (EMA).
    ///
    /// EMA даёт больший вес недавним ценам по сравнению с SMA.
    ///
    /// # Формула
    ///
    /// `EMA = Price * k + EMA_prev * (1 - k)`
    /// где `k = 2 / (period + 1)`
    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.is_empty() || period == 0 {
            return vec![];
        }

        let k = 2.0 / (period as f64 + 1.0);
        let mut ema = vec![prices[0]];

        for price in prices.iter().skip(1) {
            let prev_ema = *ema.last().unwrap();
            ema.push(price * k + prev_ema * (1.0 - k));
        }

        ema
    }
}

/// Модуль управления рисками.
pub mod risk {
    /// Менеджер рисков для торговых операций.
    ///
    /// Контролирует размер позиций и максимальные потери.
    pub struct RiskManager {
        /// Общий капитал
        pub capital: f64,
        /// Максимальный риск на сделку (доля от капитала)
        pub max_risk_per_trade: f64,
    }

    impl RiskManager {
        /// Создаёт нового менеджера рисков.
        pub fn new(capital: f64, max_risk_per_trade: f64) -> Self {
            RiskManager {
                capital,
                max_risk_per_trade,
            }
        }

        /// Рассчитывает максимальный размер позиции.
        ///
        /// # Формула
        ///
        /// `position_size = (capital * max_risk) / (entry_price * stop_loss_pct)`
        ///
        /// # Примеры
        ///
        /// ```
        /// let rm = RiskManager::new(10000.0, 0.02);
        /// let size = rm.calculate_position_size(50000.0, 0.02);
        /// assert!(size > 0.0);
        /// ```
        pub fn calculate_position_size(&self, entry_price: f64, stop_loss_pct: f64) -> f64 {
            if entry_price <= 0.0 || stop_loss_pct <= 0.0 {
                return 0.0;
            }
            (self.capital * self.max_risk_per_trade) / (entry_price * stop_loss_pct)
        }
    }
}

fn main() {
    use indicators::{calculate_sma, calculate_ema};
    use risk::RiskManager;

    let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5];

    let sma = calculate_sma(&prices, 3);
    println!("SMA(3): {:?}", sma);

    let ema = calculate_ema(&prices, 3);
    println!("EMA(3): {:?}", ema);

    let rm = RiskManager::new(10000.0, 0.02);
    let position_size = rm.calculate_position_size(50000.0, 0.02);
    println!("Max position size: {:.4} BTC", position_size);
}
```

## Примеры кода в документации

Примеры в документации компилируются и тестируются:

```rust
/// Валидирует торговый сигнал.
///
/// # Примеры
///
/// Базовое использование:
///
/// ```
/// let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.75);
/// assert!(signal.is_valid());
/// ```
///
/// Сигнал с низкой уверенностью:
///
/// ```
/// let weak_signal = TradingSignal::new("ETHUSDT", Action::Sell, 0.3);
/// assert!(!weak_signal.is_strong());
/// ```
///
/// Пример с паникой (используется should_panic):
///
/// ```should_panic
/// let invalid = TradingSignal::new("", Action::Buy, 1.5);
/// invalid.validate().unwrap();
/// ```
///
/// Пример, который не компилируется (для демонстрации):
///
/// ```compile_fail
/// let signal = TradingSignal::new("BTC", Action::Buy, 0.5);
/// signal.private_method(); // Приватный метод недоступен
/// ```
///
/// Пример, который не запускается:
///
/// ```no_run
/// let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.9);
/// signal.execute().await; // Требует async runtime
/// ```
///
/// Скрытые строки (не показываются в документации):
///
/// ```
/// # use std::collections::HashMap;
/// # let mut cache = HashMap::new();
/// # cache.insert("BTCUSDT", 50000.0);
/// let price = cache.get("BTCUSDT").unwrap();
/// assert_eq!(*price, 50000.0);
/// ```
#[derive(Debug)]
pub struct TradingSignal {
    symbol: String,
    action: Action,
    confidence: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Buy,
    Sell,
    Hold,
}

impl TradingSignal {
    pub fn new(symbol: &str, action: Action, confidence: f64) -> Self {
        TradingSignal {
            symbol: symbol.to_string(),
            action,
            confidence,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.symbol.is_empty() && self.confidence >= 0.0 && self.confidence <= 1.0
    }

    pub fn is_strong(&self) -> bool {
        self.confidence >= 0.7
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err("Confidence must be between 0 and 1".to_string());
        }
        Ok(())
    }
}

fn main() {
    let signal = TradingSignal::new("BTCUSDT", Action::Buy, 0.85);
    println!("Signal: {:?}", signal);
    println!("Is valid: {}", signal.is_valid());
    println!("Is strong: {}", signal.is_strong());
}
```

## Генерация документации

### Команды cargo doc

```bash
# Сгенерировать документацию
cargo doc

# Сгенерировать и открыть в браузере
cargo doc --open

# Включить документацию зависимостей
cargo doc --document-private-items

# Для конкретного пакета
cargo doc --package my_trading_lib

# Проверить примеры в документации
cargo test --doc
```

### Настройка в Cargo.toml

```toml
[package]
name = "trading_lib"
version = "0.1.0"
edition = "2021"
authors = ["Trading Team <team@example.com>"]
description = "Библиотека для алгоритмической торговли"
documentation = "https://docs.rs/trading_lib"
repository = "https://github.com/example/trading_lib"
license = "MIT"
keywords = ["trading", "finance", "algorithms"]
categories = ["finance", "algorithms"]

# Настройки документации
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[features]
default = []
# Включить расширенную документацию
full-docs = []
```

## Ссылки и навигация

```rust
/// Обработчик торговых событий.
///
/// Связанные типы:
/// * [`Order`] — торговый ордер
/// * [`Trade`](crate::trades::Trade) — исполненная сделка
/// * [`Position`](super::Position) — открытая позиция
///
/// Внешние ссылки:
/// * [Binance API](https://binance-docs.github.io/apidocs/)
/// * [Trading View](https://www.tradingview.com/)
///
/// Методы:
/// * [`Self::process`] — обработать событие
/// * [`Self::new`] — создать обработчик
///
/// Модули:
/// * [`crate::indicators`] — технические индикаторы
/// * [`crate::risk`] — управление рисками
pub struct EventHandler {
    orders: Vec<Order>,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
}

impl EventHandler {
    /// Создаёт новый обработчик событий.
    pub fn new() -> Self {
        EventHandler { orders: Vec::new() }
    }

    /// Обрабатывает торговое событие.
    ///
    /// См. также: [`Order`], [`Self::new`]
    pub fn process(&mut self, order: Order) {
        self.orders.push(order);
    }
}

fn main() {
    let mut handler = EventHandler::new();
    handler.process(Order { id: "ORD-001".to_string() });
    println!("Orders processed: {}", handler.orders.len());
}
```

## Практический пример: Документирование торговой библиотеки

```rust
//! # Trading Strategy Library
//!
//! Библиотека для разработки и тестирования торговых стратегий.
//!
//! ## Архитектура
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │  Market     │────▶│  Strategy   │────▶│  Orders     │
//! │  Data       │     │  Engine     │     │  Manager    │
//! └─────────────┘     └─────────────┘     └─────────────┘
//!                           │
//!                           ▼
//!                     ┌─────────────┐
//!                     │    Risk     │
//!                     │  Manager    │
//!                     └─────────────┘
//! ```
//!
//! ## Пример использования
//!
//! ```rust,no_run
//! use trading_lib::{Strategy, MarketData, RiskManager};
//!
//! fn main() {
//!     // Инициализация
//!     let mut strategy = MACrossoverStrategy::new(10, 50);
//!     let risk_manager = RiskManager::new(10000.0, 0.02);
//!
//!     // Получение данных
//!     let data = MarketData::fetch("BTCUSDT").await?;
//!
//!     // Генерация сигнала
//!     if let Some(signal) = strategy.analyze(&data) {
//!         if risk_manager.approve(&signal) {
//!             strategy.execute(signal).await?;
//!         }
//!     }
//! }
//! ```

use std::collections::HashMap;

/// Рыночные данные для анализа.
///
/// Содержит OHLCV данные (Open, High, Low, Close, Volume)
/// для одного торгового инструмента.
///
/// # Структура данных
///
/// | Поле | Описание | Пример |
/// |------|----------|--------|
/// | symbol | Торговая пара | "BTCUSDT" |
/// | prices | Цены закрытия | [50000.0, 50100.0, ...] |
/// | volumes | Объёмы | [100.5, 150.2, ...] |
///
/// # Производительность
///
/// Хранит данные в векторах для быстрого последовательного доступа.
/// Для случайного доступа используйте индексы.
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Торговая пара
    pub symbol: String,
    /// Цены открытия
    pub open: Vec<f64>,
    /// Максимальные цены
    pub high: Vec<f64>,
    /// Минимальные цены
    pub low: Vec<f64>,
    /// Цены закрытия
    pub close: Vec<f64>,
    /// Объёмы торгов
    pub volume: Vec<f64>,
}

impl MarketData {
    /// Создаёт пустой контейнер для рыночных данных.
    ///
    /// # Примеры
    ///
    /// ```
    /// let data = MarketData::new("BTCUSDT");
    /// assert_eq!(data.len(), 0);
    /// ```
    pub fn new(symbol: &str) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            open: Vec::new(),
            high: Vec::new(),
            low: Vec::new(),
            close: Vec::new(),
            volume: Vec::new(),
        }
    }

    /// Возвращает количество свечей.
    pub fn len(&self) -> usize {
        self.close.len()
    }

    /// Проверяет, пусты ли данные.
    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }

    /// Добавляет новую свечу.
    ///
    /// # Примеры
    ///
    /// ```
    /// let mut data = MarketData::new("BTCUSDT");
    /// data.add_candle(50000.0, 50500.0, 49800.0, 50200.0, 1000.0);
    /// assert_eq!(data.len(), 1);
    /// ```
    pub fn add_candle(&mut self, open: f64, high: f64, low: f64, close: f64, volume: f64) {
        self.open.push(open);
        self.high.push(high);
        self.low.push(low);
        self.close.push(close);
        self.volume.push(volume);
    }

    /// Возвращает последнюю цену закрытия.
    ///
    /// # Возвращает
    ///
    /// `Some(price)` если есть данные, иначе `None`.
    pub fn last_close(&self) -> Option<f64> {
        self.close.last().copied()
    }
}

/// Стратегия на пересечении скользящих средних.
///
/// Генерирует сигнал на покупку, когда короткая MA
/// пересекает длинную MA снизу вверх, и наоборот для продажи.
///
/// # Параметры
///
/// * `short_period` — период короткой MA (обычно 10-20)
/// * `long_period` — период длинной MA (обычно 50-200)
///
/// # Пример
///
/// ```
/// let strategy = MACrossoverStrategy::new(10, 50);
/// let signal = strategy.generate_signal(&market_data);
/// ```
///
/// # Логика сигналов
///
/// | Условие | Сигнал |
/// |---------|--------|
/// | Short MA > Long MA (пересечение) | Buy |
/// | Short MA < Long MA (пересечение) | Sell |
/// | Нет пересечения | Hold |
pub struct MACrossoverStrategy {
    short_period: usize,
    long_period: usize,
    prev_short_above: Option<bool>,
}

/// Торговый сигнал.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Сигнал на покупку
    Buy,
    /// Сигнал на продажу
    Sell,
    /// Удерживать позицию
    Hold,
}

impl MACrossoverStrategy {
    /// Создаёт новую стратегию с заданными периодами.
    ///
    /// # Паника
    ///
    /// Паникует если `short_period >= long_period`.
    ///
    /// # Примеры
    ///
    /// ```
    /// let strategy = MACrossoverStrategy::new(10, 50);
    /// ```
    ///
    /// ```should_panic
    /// let invalid = MACrossoverStrategy::new(50, 10);
    /// ```
    pub fn new(short_period: usize, long_period: usize) -> Self {
        assert!(short_period < long_period, "Short period must be less than long period");
        MACrossoverStrategy {
            short_period,
            long_period,
            prev_short_above: None,
        }
    }

    /// Генерирует торговый сигнал на основе рыночных данных.
    ///
    /// # Аргументы
    ///
    /// * `data` — рыночные данные с ценами закрытия
    ///
    /// # Возвращает
    ///
    /// Торговый сигнал (`Buy`, `Sell`, или `Hold`).
    ///
    /// # Примеры
    ///
    /// ```
    /// let mut strategy = MACrossoverStrategy::new(5, 10);
    /// let mut data = MarketData::new("BTCUSDT");
    ///
    /// // Добавляем данные...
    /// for i in 0..20 {
    ///     data.add_candle(100.0, 101.0, 99.0, 100.0 + i as f64, 1000.0);
    /// }
    ///
    /// let signal = strategy.generate_signal(&data);
    /// println!("Signal: {:?}", signal);
    /// ```
    pub fn generate_signal(&mut self, data: &MarketData) -> Signal {
        if data.close.len() < self.long_period {
            return Signal::Hold;
        }

        let short_ma = self.calculate_sma(&data.close, self.short_period);
        let long_ma = self.calculate_sma(&data.close, self.long_period);

        let short_above = short_ma > long_ma;

        let signal = match self.prev_short_above {
            Some(prev) if prev != short_above => {
                if short_above { Signal::Buy } else { Signal::Sell }
            }
            _ => Signal::Hold,
        };

        self.prev_short_above = Some(short_above);
        signal
    }

    fn calculate_sma(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return 0.0;
        }
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        sum / period as f64
    }
}

fn main() {
    // Демонстрация использования
    let mut data = MarketData::new("BTCUSDT");

    // Добавляем тестовые данные
    let prices = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 105.0,
                  104.5, 106.0, 107.0, 108.0, 107.5, 109.0, 110.0,
                  111.0, 112.0, 113.0, 114.0, 115.0, 116.0];

    for price in prices.iter() {
        data.add_candle(*price, price + 1.0, price - 1.0, *price, 1000.0);
    }

    let mut strategy = MACrossoverStrategy::new(5, 10);
    let signal = strategy.generate_signal(&data);

    println!("Market Data: {} candles", data.len());
    println!("Strategy Signal: {:?}", signal);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **`///`** | Документационный комментарий для элемента |
| **`//!`** | Документационный комментарий для модуля |
| **Секции** | `# Examples`, `# Arguments`, `# Returns`, `# Panics`, `# Errors` |
| **Примеры кода** | Компилируются и тестируются автоматически |
| **Атрибуты** | `should_panic`, `no_run`, `compile_fail`, `ignore` |
| **Ссылки** | `[`Name`]`, `[`crate::path`]`, `[`Self::method`]` |
| **cargo doc** | Генерация HTML документации |
| **cargo test --doc** | Тестирование примеров в документации |

## Практические задания

1. **Документация индикатора**: Напиши полную документацию для функции расчёта MACD:
   - Описание алгоритма
   - Все параметры с примерами значений
   - Примеры использования
   - Обработка граничных случаев

2. **Документация модуля**: Создай модуль `position_manager` с полной документацией:
   - Описание модуля с диаграммой
   - Документация всех публичных типов
   - Примеры использования
   - Ссылки между типами

3. **Тестируемые примеры**: Напиши документацию с примерами для:
   - Успешного сценария
   - Обработки ошибок
   - Пограничных случаев
   - Примера с паникой

4. **Интеграция с CI**: Настрой автоматическую:
   - Генерацию документации при push
   - Проверку примеров в документации
   - Публикацию на GitHub Pages

## Домашнее задание

1. **Полная документация библиотеки**: Создай документированную торговую библиотеку:
   - Минимум 5 публичных структур с полной документацией
   - Каждая функция с примерами кода
   - Модульная документация с обзором
   - Диаграмма архитектуры в README
   - Все примеры должны компилироваться

2. **Changelog и версионирование**: Реализуй систему документирования изменений:
   - Структура для хранения изменений по версиям
   - Автоматическая генерация CHANGELOG.md
   - Документация breaking changes
   - Примеры миграции между версиями

3. **Интерактивная документация**: Создай расширенную документацию:
   - Примеры для каждого публичного API
   - Tutorials в виде doc-тестов
   - FAQ в документации модуля
   - Ссылки на внешние ресурсы

4. **Документация с бенчмарками**: Добавь в документацию:
   - Информацию о производительности
   - Сравнение разных подходов
   - Рекомендации по оптимизации
   - Big-O нотацию для алгоритмов

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../352-*/ru.md)
