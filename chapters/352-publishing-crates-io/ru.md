# День 352: Публикация на crates.io

## Аналогия из трейдинга

Представь, что ты разработал мощный алгоритм для расчёта торговых индикаторов. Теперь у тебя есть два варианта:

**Приватный инструмент:**
Ты держишь код у себя и используешь только сам. Каждый раз, когда тебе нужна эта функциональность в новом проекте, ты копируешь код вручную.

**Публикация на бирже кода (crates.io):**
Ты "листингуешь" свой алгоритм на публичной площадке. Теперь любой разработчик (включая тебя) может подключить его одной строкой в `Cargo.toml`. Это как IPO для твоего кода — он становится доступен всему сообществу.

| Аспект | Приватный код | Публикация на crates.io |
|--------|---------------|------------------------|
| **Доступность** | Только ты | Весь мир |
| **Обновления** | Ручное копирование | `cargo update` |
| **Версионность** | Нет | Семантическое версионирование |
| **Документация** | Локальная | Автоматически на docs.rs |
| **Зависимости** | Ручное управление | Автоматическое разрешение |

## Подготовка крейта к публикации

### Структура Cargo.toml

```toml
[package]
name = "trading-indicators"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "High-performance trading indicators for algorithmic trading"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/trading-indicators"
homepage = "https://github.com/yourusername/trading-indicators"
documentation = "https://docs.rs/trading-indicators"
readme = "README.md"
keywords = ["trading", "indicators", "finance", "algorithms", "rust"]
categories = ["finance", "algorithms", "mathematics"]
exclude = ["tests/data/*", ".github/*"]

[dependencies]
# Зависимости для расчётов
```

### Обязательные поля

```rust
/// Пример структуры библиотеки для торговых индикаторов
///
/// # Пример использования
///
/// ```rust
/// use trading_indicators::{SMA, TradingIndicator};
///
/// let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];
/// let sma = SMA::new(3);
/// let result = sma.calculate(&prices);
/// println!("SMA: {:?}", result);
/// ```

/// Трейт для всех торговых индикаторов
pub trait TradingIndicator {
    /// Рассчитывает значения индикатора по ценам
    fn calculate(&self, prices: &[f64]) -> Vec<f64>;

    /// Возвращает название индикатора
    fn name(&self) -> &str;

    /// Минимальное количество точек для расчёта
    fn min_periods(&self) -> usize;
}

/// Простая скользящая средняя (SMA)
#[derive(Debug, Clone)]
pub struct SMA {
    period: usize,
}

impl SMA {
    /// Создаёт новый SMA с заданным периодом
    ///
    /// # Аргументы
    ///
    /// * `period` - Период усреднения (должен быть > 0)
    ///
    /// # Паника
    ///
    /// Паникует если period равен 0
    ///
    /// # Пример
    ///
    /// ```rust
    /// use trading_indicators::SMA;
    ///
    /// let sma = SMA::new(20);
    /// assert_eq!(sma.period(), 20);
    /// ```
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        SMA { period }
    }

    /// Возвращает период SMA
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

/// Экспоненциальная скользящая средняя (EMA)
#[derive(Debug, Clone)]
pub struct EMA {
    period: usize,
    multiplier: f64,
}

impl EMA {
    /// Создаёт новый EMA с заданным периодом
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

        // Первое значение — это SMA
        let first_ema: f64 = prices[..self.period].iter().sum::<f64>()
            / self.period as f64;
        result.push(first_ema);

        // Последующие значения рассчитываются по формуле EMA
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
```

## Документация для публикации

### Написание README.md

```markdown
# Trading Indicators

[![Crates.io](https://img.shields.io/crates/v/trading-indicators.svg)](https://crates.io/crates/trading-indicators)
[![Documentation](https://docs.rs/trading-indicators/badge.svg)](https://docs.rs/trading-indicators)
[![License](https://img.shields.io/crates/l/trading-indicators.svg)](LICENSE)

High-performance trading indicators for algorithmic trading in Rust.

## Features

- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- Relative Strength Index (RSI)
- MACD
- Bollinger Bands

## Installation

Add to your `Cargo.toml`:

\```toml
[dependencies]
trading-indicators = "0.1"
\```

## Quick Start

\```rust
use trading_indicators::{SMA, EMA, TradingIndicator};

fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];

    // Simple Moving Average
    let sma = SMA::new(3);
    let sma_values = sma.calculate(&prices);
    println!("SMA(3): {:?}", sma_values);

    // Exponential Moving Average
    let ema = EMA::new(3);
    let ema_values = ema.calculate(&prices);
    println!("EMA(3): {:?}", ema_values);
}
\```

## License

Licensed under either of Apache License, Version 2.0 or MIT license.
```

### Документация кода (doc comments)

```rust
//! # Trading Indicators Library
//!
//! Библиотека для расчёта торговых индикаторов.
//!
//! ## Обзор
//!
//! Эта библиотека предоставляет высокопроизводительные реализации
//! популярных торговых индикаторов:
//!
//! - [`SMA`] - Простая скользящая средняя
//! - [`EMA`] - Экспоненциальная скользящая средняя
//! - [`RSI`] - Индекс относительной силы
//! - [`MACD`] - Схождение/расхождение скользящих средних
//!
//! ## Пример использования
//!
//! ```rust
//! use trading_indicators::{SMA, TradingIndicator};
//!
//! let btc_prices = vec![
//!     50000.0, 50500.0, 51000.0, 50800.0, 51200.0,
//!     51500.0, 51300.0, 52000.0, 52500.0, 52300.0,
//! ];
//!
//! let sma = SMA::new(5);
//! let sma_values = sma.calculate(&btc_prices);
//!
//! // Определение тренда
//! if let (Some(&last_price), Some(&last_sma)) =
//!     (btc_prices.last(), sma_values.last())
//! {
//!     if last_price > last_sma {
//!         println!("Восходящий тренд: цена выше SMA");
//!     } else {
//!         println!("Нисходящий тренд: цена ниже SMA");
//!     }
//! }
//! ```
//!
//! ## Производительность
//!
//! Все индикаторы оптимизированы для работы с большими массивами данных
//! и используют итераторы Rust для минимизации аллокаций.

/// Индекс относительной силы (RSI)
///
/// RSI измеряет скорость и величину ценовых изменений
/// для определения перекупленности или перепроданности актива.
///
/// # Формула
///
/// RSI = 100 - (100 / (1 + RS))
/// где RS = средний рост / средний убыток
///
/// # Интерпретация
///
/// - RSI > 70: Актив перекуплен (возможна коррекция вниз)
/// - RSI < 30: Актив перепродан (возможен отскок вверх)
///
/// # Пример
///
/// ```rust
/// use trading_indicators::{RSI, TradingIndicator};
///
/// let prices = vec![
///     44.0, 44.25, 44.50, 43.75, 44.50,
///     44.25, 44.50, 44.00, 43.50, 43.25,
///     43.50, 44.25, 44.75, 45.00, 45.50,
/// ];
///
/// let rsi = RSI::new(14);
/// if let Some(&current_rsi) = rsi.calculate(&prices).last() {
///     match current_rsi {
///         r if r > 70.0 => println!("Перекуплен: {:.1}", r),
///         r if r < 30.0 => println!("Перепродан: {:.1}", r),
///         r => println!("Нейтрально: {:.1}", r),
///     }
/// }
/// ```
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

        // Рассчитываем изменения цен
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

        // Первый RSI
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
```

## Процесс публикации

### Шаг 1: Регистрация на crates.io

```bash
# Создай аккаунт на https://crates.io через GitHub

# Получи API токен в настройках профиля
# https://crates.io/settings/tokens

# Авторизуйся в Cargo
cargo login <your-api-token>
```

### Шаг 2: Проверка перед публикацией

```bash
# Проверка на ошибки
cargo check

# Запуск тестов
cargo test

# Проверка документации
cargo doc --no-deps --open

# Сухой запуск публикации (без реальной публикации)
cargo publish --dry-run

# Проверка, что крейт упаковывается корректно
cargo package --list
```

### Шаг 3: Публикация

```rust
// Демонстрация процесса публикации через код
fn demonstrate_publish_process() {
    println!("=== Процесс публикации на crates.io ===\n");

    // Шаг 1: Проверка версии
    println!("1. Проверяем версию в Cargo.toml...");
    println!("   version = \"0.1.0\"");

    // Шаг 2: Проверка метаданных
    println!("\n2. Обязательные поля:");
    println!("   - name: имя крейта (уникальное)");
    println!("   - version: версия по SemVer");
    println!("   - license: MIT OR Apache-2.0");
    println!("   - description: краткое описание");

    // Шаг 3: Публикация
    println!("\n3. Команда публикации:");
    println!("   cargo publish");

    // После публикации
    println!("\n4. После успешной публикации:");
    println!("   - Крейт доступен на crates.io");
    println!("   - Документация на docs.rs");
    println!("   - Нельзя удалить или перезаписать версию!");
}

fn main() {
    demonstrate_publish_process();
}
```

## Версионирование (SemVer)

```rust
/// Демонстрация семантического версионирования для торговой библиотеки
///
/// MAJOR.MINOR.PATCH
///
/// - MAJOR: Несовместимые изменения API
/// - MINOR: Новая функциональность с обратной совместимостью
/// - PATCH: Исправление багов

// Версия 0.1.0 - Начальная версия
mod v0_1_0 {
    pub struct SMA {
        pub period: usize,
    }

    impl SMA {
        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect()
        }
    }
}

// Версия 0.1.1 - PATCH: Исправлен баг с пустым массивом
mod v0_1_1 {
    pub struct SMA {
        pub period: usize,
    }

    impl SMA {
        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            // Добавлена проверка на пустой массив (PATCH)
            if prices.len() < self.period {
                return vec![];
            }

            prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect()
        }
    }
}

// Версия 0.2.0 - MINOR: Добавлен новый индикатор
mod v0_2_0 {
    pub struct SMA {
        period: usize,
    }

    // Новая функциональность (MINOR)
    pub struct EMA {
        period: usize,
    }

    impl EMA {
        pub fn new(period: usize) -> Self {
            EMA { period }
        }

        pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
            // Реализация EMA
            vec![]
        }
    }
}

// Версия 1.0.0 - MAJOR: Изменён API
mod v1_0_0 {
    /// Теперь используется Result вместо паники
    pub struct SMA {
        period: usize,
    }

    #[derive(Debug)]
    pub enum IndicatorError {
        InvalidPeriod,
        InsufficientData,
    }

    impl SMA {
        // BREAKING CHANGE: возвращает Result
        pub fn new(period: usize) -> Result<Self, IndicatorError> {
            if period == 0 {
                return Err(IndicatorError::InvalidPeriod);
            }
            Ok(SMA { period })
        }

        // BREAKING CHANGE: возвращает Result
        pub fn calculate(&self, prices: &[f64]) -> Result<Vec<f64>, IndicatorError> {
            if prices.len() < self.period {
                return Err(IndicatorError::InsufficientData);
            }

            Ok(prices
                .windows(self.period)
                .map(|w| w.iter().sum::<f64>() / self.period as f64)
                .collect())
        }
    }
}

fn main() {
    println!("=== Семантическое версионирование ===\n");

    println!("0.1.0 -> 0.1.1: PATCH");
    println!("  Исправлен баг, API не изменился\n");

    println!("0.1.1 -> 0.2.0: MINOR");
    println!("  Добавлен EMA, старый код работает\n");

    println!("0.2.0 -> 1.0.0: MAJOR");
    println!("  Изменён API (Result вместо panic)");
    println!("  Требуется обновление кода пользователей");
}
```

## Управление зависимостями

```toml
# Cargo.toml с правильным управлением зависимостями

[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# Точная версия (не рекомендуется для большинства случаев)
serde = "=1.0.193"

# Совместимая версия (рекомендуется)
tokio = "1.35"          # Любая 1.x.y где x >= 35

# С указанием features
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }

# Из git репозитория (для разработки)
# trading-indicators = { git = "https://github.com/user/trading-indicators" }

# Локальный путь (для разработки)
# trading-indicators = { path = "../trading-indicators" }

[dev-dependencies]
# Зависимости только для тестов
criterion = "0.5"

[build-dependencies]
# Зависимости для build.rs
# cc = "1.0"

# Опциональные features
[features]
default = ["sma", "ema"]
sma = []
ema = []
rsi = []
macd = []
all-indicators = ["sma", "ema", "rsi", "macd"]
```

### Пример использования features

```rust
//! Библиотека с опциональными индикаторами

#[cfg(feature = "sma")]
mod sma;

#[cfg(feature = "ema")]
mod ema;

#[cfg(feature = "rsi")]
mod rsi;

#[cfg(feature = "macd")]
mod macd;

// Реэкспорт включённых модулей
#[cfg(feature = "sma")]
pub use sma::SMA;

#[cfg(feature = "ema")]
pub use ema::EMA;

#[cfg(feature = "rsi")]
pub use rsi::RSI;

#[cfg(feature = "macd")]
pub use macd::MACD;

/// Проверка включённых features
pub fn available_indicators() -> Vec<&'static str> {
    let mut indicators = Vec::new();

    #[cfg(feature = "sma")]
    indicators.push("SMA");

    #[cfg(feature = "ema")]
    indicators.push("EMA");

    #[cfg(feature = "rsi")]
    indicators.push("RSI");

    #[cfg(feature = "macd")]
    indicators.push("MACD");

    indicators
}

fn main() {
    println!("Доступные индикаторы: {:?}", available_indicators());
}
```

## Практический пример: Публикация торговой библиотеки

```rust
//! # Trading Strategy Library
//!
//! Полнофункциональная библиотека для алготрейдинга.

use std::collections::HashMap;

/// Торговый сигнал
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy { price: f64, quantity: f64 },
    Sell { price: f64, quantity: f64 },
    Hold,
}

/// Трейт для торговых стратегий
pub trait Strategy: Send + Sync {
    /// Название стратегии
    fn name(&self) -> &str;

    /// Генерация сигнала на основе данных
    fn generate_signal(&self, data: &MarketData) -> Signal;

    /// Параметры стратегии
    fn parameters(&self) -> HashMap<String, f64>;
}

/// Рыночные данные
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

/// Стратегия пересечения скользящих средних
#[derive(Debug, Clone)]
pub struct CrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
}

impl CrossoverStrategy {
    /// Создаёт новую стратегию пересечения MA
    ///
    /// # Аргументы
    ///
    /// * `fast_period` - Период быстрой MA
    /// * `slow_period` - Период медленной MA
    ///
    /// # Пример
    ///
    /// ```rust
    /// use trading_strategy::{CrossoverStrategy, Strategy, MarketData};
    ///
    /// let strategy = CrossoverStrategy::new(10, 20);
    ///
    /// let mut data = MarketData::new("BTCUSDT");
    /// for i in 0..30 {
    ///     data.add_candle(50000.0 + (i as f64 * 100.0), 1000.0, i);
    /// }
    ///
    /// let signal = strategy.generate_signal(&data);
    /// println!("{:?}", signal);
    /// ```
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

        // Сигнал на покупку: быстрая MA пересекает медленную снизу вверх
        if fast_ma > slow_ma {
            Signal::Buy {
                price: current_price,
                quantity: 1.0,
            }
        }
        // Сигнал на продажу: быстрая MA пересекает медленную сверху вниз
        else if fast_ma < slow_ma {
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

/// Менеджер стратегий
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

fn main() {
    println!("=== Демонстрация торговой библиотеки ===\n");

    // Создаём рыночные данные
    let mut data = MarketData::new("BTCUSDT");

    // Симулируем восходящий тренд
    let base_price = 50000.0;
    for i in 0..50 {
        let price = base_price + (i as f64 * 50.0) + (i as f64).sin() * 100.0;
        data.add_candle(price, 1000.0, i);
    }

    // Создаём стратегию
    let strategy = CrossoverStrategy::new(5, 20);

    println!("Стратегия: {}", strategy.name());
    println!("Параметры: {:?}", strategy.parameters());
    println!("Последняя цена: ${:.2}", data.last_price().unwrap());

    // Генерируем сигнал
    let signal = strategy.generate_signal(&data);
    println!("\nСигнал: {:?}", signal);

    // Используем менеджер стратегий
    let mut manager = StrategyManager::new();
    manager.add_strategy(Box::new(CrossoverStrategy::new(5, 20)));
    manager.add_strategy(Box::new(CrossoverStrategy::new(10, 30)));

    println!("\n=== Сигналы от всех стратегий ===");
    for (name, signal) in manager.generate_signals(&data) {
        println!("{}: {:?}", name, signal);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **crates.io** | Официальный реестр пакетов Rust |
| **Cargo.toml** | Конфигурация крейта с метаданными |
| **SemVer** | Семантическое версионирование (MAJOR.MINOR.PATCH) |
| **cargo publish** | Команда публикации крейта |
| **docs.rs** | Автоматическая документация для опубликованных крейтов |
| **Features** | Опциональные возможности крейта |
| **API Token** | Токен для авторизации публикации |

## Практические задания

1. **Создание библиотеки индикаторов**: Создай крейт с:
   - Реализацией SMA, EMA, RSI
   - Полной документацией для каждой функции
   - Примерами использования в doc-тестах
   - README.md с инструкциями
   - Тестами для всех публичных функций

2. **Версионирование библиотеки**: Разработай:
   - Версию 0.1.0 с базовой функциональностью
   - Версию 0.2.0 с новыми индикаторами
   - Версию 1.0.0 с улучшенным API
   - CHANGELOG.md с описанием изменений

3. **Features для крейта**: Добавь:
   - Опциональные тяжёлые зависимости
   - Конфигурируемые компоненты
   - Feature для асинхронного API
   - Документацию по использованию features

4. **CI/CD для публикации**: Настрой:
   - Автоматическое тестирование при PR
   - Проверку форматирования (rustfmt)
   - Проверку стиля (clippy)
   - Автоматическую публикацию при создании тега

## Домашнее задание

1. **Полноценная торговая библиотека**: Создай крейт:
   - С минимум 5 индикаторами (SMA, EMA, RSI, MACD, Bollinger Bands)
   - С документацией на русском и английском
   - С примерами в директории examples/
   - С бенчмарками производительности
   - Опубликуй на crates.io (или подготовь к публикации)

2. **Workspace с несколькими крейтами**: Создай:
   - trading-core: базовые типы и трейты
   - trading-indicators: реализации индикаторов
   - trading-strategies: торговые стратегии
   - trading-bot: CLI приложение
   - Настрой версионирование между крейтами

3. **Миграция между версиями**: Напиши:
   - Руководство по миграции с 0.x на 1.0
   - Deprecation warnings для устаревшего API
   - Инструменты для автоматической миграции
   - Тесты обратной совместимости

4. **Документация как в лучших крейтах**: Изучи:
   - Документацию serde, tokio, reqwest
   - Создай такую же структуру для своего крейта
   - Добавь cookbook с рецептами
   - Создай интерактивные примеры

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../353-*/ru.md)
