# День 364: Рефакторинг: эволюция системы

## Аналогия из трейдинга

Представь, что ты несколько лет торгуешь вручную и записываешь все сделки в Excel. Сначала это была простая таблица: дата, тикер, цена покупки, цена продажи, прибыль. Но со временем ты добавлял новые колонки: комиссии, проскальзывание, время удержания позиции, скриншоты графиков, ссылки на новости...

**Рефакторинг — это реорганизация твоего торгового журнала без изменения записанных данных:**

| Торговый журнал | Рефакторинг кода |
|-----------------|------------------|
| **Разделяем огромную таблицу на связанные листы** | Разбиваем большой модуль на подмодули |
| **Создаём шаблоны для типовых сделок** | Выделяем повторяющийся код в функции |
| **Переименовываем колонки для ясности** | Даём понятные имена переменным и функциям |
| **Добавляем формулы вместо ручных расчётов** | Автоматизируем рутинные операции |
| **Группируем сделки по стратегиям** | Организуем код по доменным областям |

**Главное правило рефакторинга:** результат торговли не должен измениться. Если до рефакторинга журнала у тебя был профит $10,000 — после реорганизации должно быть ровно столько же.

## Принципы рефакторинга

### Когда рефакторить?

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Признаки того, что код нуждается в рефакторинге ("запахи кода")
///
/// В трейдинге аналогично: когда торговая система становится слишком сложной
/// для понимания и поддержки, пора её упрощать.

// ЗАПАХ 1: Дублирование кода
// Плохо: одна и та же логика расчёта комиссии повторяется

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

// Хорошо: выделяем общую логику

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

// ЗАПАХ 2: Длинные функции
// Плохо: функция делает слишком много

mod long_function_before {
    use super::*;

    pub fn process_trade_signal(
        signal: &str,
        price: f64,
        balance: f64,
    ) -> Result<String, String> {
        // Валидация сигнала
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

        // Расчёт размера позиции
        let risk_percent = 0.02;
        let position_size = balance * risk_percent * strength;
        if position_size > balance * 0.5 {
            return Err("Position too large".to_string());
        }

        // Расчёт комиссии
        let fee = position_size * 0.001;
        let net_position = position_size - fee;

        // Формирование ордера
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

// Хорошо: разбиваем на маленькие функции с одной ответственностью

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
    // Демонстрация рефакторинга расчёта комиссий
    println!("=== Рефакторинг расчёта комиссий ===\n");

    let volume = 1.5;
    let price = 50000.0;

    // До рефакторинга
    let spot_fee_old = before_refactoring::calculate_spot_fee(volume, price);
    let futures_fee_old = before_refactoring::calculate_futures_fee(volume, price);

    // После рефакторинга
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

    // Демонстрация рефакторинга обработки сигналов
    println!("\n=== Рефакторинг обработки сигналов ===\n");

    let signal = "BTCUSDT:BUY:0.8";
    let balance = 100000.0;

    match long_function_after::process_trade_signal(signal, price, balance) {
        Ok(order) => println!("Order: {:?}", order),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Паттерн "Извлечение метода"

```rust
use std::collections::HashMap;

/// Извлечение метода — базовый приём рефакторинга.
/// Как выделение отдельного торгового правила из общей стратегии.

// До рефакторинга: монолитный анализатор рынка
mod before {
    pub fn analyze_market(prices: &[f64], volumes: &[f64]) -> String {
        // Расчёт скользящих средних
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

        // Расчёт объёма
        let avg_volume = if !volumes.is_empty() {
            volumes.iter().sum::<f64>() / volumes.len() as f64
        } else {
            0.0
        };

        let current_volume = *volumes.last().unwrap_or(&0.0);
        let volume_spike = current_volume > avg_volume * 1.5;

        // Определение тренда
        let current_price = *prices.last().unwrap_or(&0.0);
        let trend = if sma_20 > sma_50 && current_price > sma_20 {
            "BULLISH"
        } else if sma_20 < sma_50 && current_price < sma_20 {
            "BEARISH"
        } else {
            "NEUTRAL"
        };

        // Генерация сигнала
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

// После рефакторинга: разделённые ответственности
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

    /// Калькулятор скользящих средних
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

    /// Анализатор объёмов
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

    /// Определитель тренда
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

    /// Генератор сигналов — объединяет все компоненты
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
    // Генерируем тестовые данные
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 10.0) + (i as f64).sin() * 100.0)
        .collect();

    let volumes: Vec<f64> = (0..100)
        .map(|i| 1000.0 + (i as f64 * 5.0) + if i == 99 { 3000.0 } else { 0.0 })
        .collect();

    println!("=== Сравнение до и после рефакторинга ===\n");

    let signal_old = before::analyze_market(&prices, &volumes);
    let signal_new = after::SignalGenerator::generate(&prices, &volumes);

    println!("Сигнал (до рефакторинга): {}", signal_old);
    println!("Сигнал (после рефакторинга): {:?}", signal_new);

    // Преимущества нового подхода
    println!("\n=== Преимущества рефакторинга ===\n");

    // Можем использовать компоненты отдельно
    let trend = after::TrendDetector::detect(&prices);
    println!("Тренд: {:?}", trend);

    let sma_20 = after::MovingAverageCalculator::sma(&prices, 20);
    let sma_50 = after::MovingAverageCalculator::sma(&prices, 50);
    println!("SMA(20): {:.2}", sma_20.unwrap_or(0.0));
    println!("SMA(50): {:.2}", sma_50.unwrap_or(0.0));

    let volume_spike = after::VolumeAnalyzer::is_spike(&volumes, 1.5);
    println!("Всплеск объёма: {}", volume_spike);
}
```

## Рефакторинг через типы

```rust
use std::marker::PhantomData;

/// В Rust типы — мощный инструмент рефакторинга.
/// Они делают невалидные состояния невозможными на этапе компиляции.

// До рефакторинга: ордер может быть в невалидном состоянии
mod before {
    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: String,
        pub symbol: String,
        pub side: String,      // "BUY" или "SELL" — но что если опечатка?
        pub price: f64,        // Может быть отрицательной
        pub quantity: f64,     // Может быть нулевой
        pub status: String,    // Что угодно
        pub filled_qty: f64,   // Может быть больше quantity
    }

    impl Order {
        pub fn execute(&mut self) -> Result<(), String> {
            // Много рантайм проверок
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

// После рефакторинга: невалидные состояния невозможны

mod after {
    use std::marker::PhantomData;

    // Типы для состояний ордера (Type State Pattern)
    pub struct Pending;
    pub struct PartiallyFilled;
    pub struct Filled;
    pub struct Cancelled;

    // Типобезопасные enums
    #[derive(Debug, Clone, Copy)]
    pub enum Side {
        Buy,
        Sell,
    }

    // Newtype паттерн для валидации
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

    /// Ордер с типизированным состоянием
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

        /// PartiallyFilled -> Cancelled (с частичным исполнением)
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

    // Общие методы для всех состояний
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
    println!("=== Рефакторинг через типы ===\n");

    // Старый способ — ошибки обнаруживаются только в рантайме
    println!("До рефакторинга:");
    let mut old_order = before::Order {
        id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUUY".to_string(), // Опечатка — компилятор не поймает
        price: 50000.0,
        quantity: 1.0,
        status: "PENDING".to_string(),
        filled_qty: 0.0,
    };

    match old_order.execute() {
        Ok(_) => println!("  Order executed"),
        Err(e) => println!("  Runtime error: {}", e),
    }

    // Новый способ — ошибки невозможны
    println!("\nПосле рефакторинга:");

    let price = after::Price::new(50000.0).expect("Invalid price");
    let quantity = after::Quantity::new(1.0).expect("Invalid quantity");

    let pending_order = after::Order::new(
        "ORD-002".to_string(),
        "BTCUSDT".to_string(),
        after::Side::Buy,  // Только Buy или Sell — опечатка невозможна
        price,
        quantity,
    );

    println!("  Created: {:?}", pending_order.id());

    // Переход состояний гарантирован типами
    let partial_order = pending_order.partial_fill(0.5);
    println!("  Partial fill: {} units", partial_order.filled_quantity());

    let filled_order = partial_order.complete_fill();
    println!("  Notional: ${:.2}", filled_order.calculate_notional());

    // Это не скомпилируется:
    // filled_order.cancel(); // Error: no method `cancel` for Order<Filled>
    // pending_order.calculate_notional(); // Error: moved value
}
```

## Рефакторинг торговой системы

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Комплексный пример рефакторинга торговой системы.
/// Показываем эволюцию от простого скрипта до модульной архитектуры.

// Версия 1: Всё в одной функции (типично для прототипа)
mod v1_prototype {
    pub fn trading_bot(prices: &[f64], balance: f64) -> f64 {
        let mut cash = balance;
        let mut position = 0.0;

        for i in 1..prices.len() {
            let prev = prices[i - 1];
            let curr = prices[i];

            // Простая стратегия: покупаем на падении, продаём на росте
            if curr < prev * 0.99 && cash > 0.0 {
                // Покупаем
                let qty = cash / curr;
                position += qty;
                cash = 0.0;
                println!("V1: BUY {:.4} @ {:.2}", qty, curr);
            } else if curr > prev * 1.01 && position > 0.0 {
                // Продаём
                cash = position * curr;
                println!("V1: SELL {:.4} @ {:.2}", position, curr);
                position = 0.0;
            }
        }

        // Финальный баланс
        cash + position * prices.last().unwrap_or(&0.0)
    }
}

// Версия 2: Выделяем компоненты
mod v2_components {
    #[derive(Debug, Clone, Copy)]
    pub enum Signal {
        Buy,
        Sell,
        Hold,
    }

    /// Стратегия отделена от исполнения
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

    /// Позиция отделена от торговой логики
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

    /// Бот использует композицию
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

// Версия 3: Полная модульность с трейтами
mod v3_modular {
    use std::collections::VecDeque;

    /// Абстракция источника данных
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

    /// Абстракция стратегии с состоянием
    pub trait Strategy {
        fn on_price(&mut self, price: f64) -> Signal;
        fn reset(&mut self);
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Signal {
        Buy(f64),  // с размером позиции
        Sell(f64),
        Hold,
    }

    /// SMA Crossover стратегия
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

    /// Абстракция риск-менеджмента
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

    /// Абстракция исполнения
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

    /// Торговый движок — собирает всё вместе
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

    /// Удобная функция для бэктеста
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
    // Генерируем тестовые данные: тренд с шумом
    let prices: Vec<f64> = (0..100)
        .map(|i| {
            let trend = 50000.0 + (i as f64) * 50.0;
            let noise = ((i as f64) * 0.5).sin() * 500.0;
            trend + noise
        })
        .collect();

    let initial_balance = 10000.0;

    println!("=== Эволюция торговой системы ===\n");
    println!("Начальный баланс: ${:.2}\n", initial_balance);

    // Версия 1: Прототип
    println!("--- Версия 1: Прототип ---");
    let final_v1 = v1_prototype::trading_bot(&prices, initial_balance);
    println!("Финальный баланс: ${:.2}\n", final_v1);

    // Версия 2: Компоненты
    println!("--- Версия 2: Компоненты ---");
    let strategy = v2_components::MomentumStrategy {
        buy_threshold: 0.01,
        sell_threshold: 0.01,
    };
    let final_v2 = v2_components::trading_bot(&strategy, &prices, initial_balance);
    println!("Финальный баланс: ${:.2}\n", final_v2);

    // Версия 3: Модульная архитектура
    println!("--- Версия 3: Модульная ---");
    let data = v3_modular::VecDataSource::new(prices.clone());
    let strategy = v3_modular::SmaCrossover::new(10, 30);
    let risk = v3_modular::MaxPositionRisk::new(0.5);
    let final_v3 = v3_modular::backtest(&data, strategy, risk, initial_balance, 0.001);
    println!("Финальный баланс: ${:.2}\n", final_v3);

    println!("=== Преимущества модульной версии ===");
    println!("1. Стратегию можно менять без изменения движка");
    println!("2. Риск-менеджмент независим от стратегии");
    println!("3. Легко добавить реальное исполнение вместо симуляции");
    println!("4. Каждый компонент можно тестировать отдельно");
    println!("5. Код самодокументируется через типы");
}
```

## Метрики качества рефакторинга

```rust
use std::collections::HashMap;

/// Метрики, которые помогают оценить качество рефакторинга.
/// Как анализ эффективности торговой стратегии.

/// Метрики сложности кода
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

        // Штраф за длинные функции
        if self.avg_function_length > 20.0 {
            score -= (self.avg_function_length - 20.0) * 2.0;
        }

        // Штраф за высокую сложность
        if self.cyclomatic_complexity > 10 {
            score -= (self.cyclomatic_complexity - 10) as f64 * 3.0;
        }

        // Штраф за много зависимостей
        if self.dependencies_count > 5 {
            score -= (self.dependencies_count - 5) as f64 * 2.0;
        }

        score.max(0.0)
    }
}

/// Сравнение метрик до и после рефакторинга
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
        // Рефакторинг безопасен, если все тесты проходят
        self.tests_passed_after >= self.tests_passed_before
    }

    pub fn summary(&self) -> String {
        format!(
            "Рефакторинг: улучшение {:.1}%, тесты: {}/{} -> {}/{}, безопасен: {}",
            self.improvement(),
            self.tests_passed_before,
            self.total_tests,
            self.tests_passed_after,
            self.total_tests,
            if self.is_safe() { "Да" } else { "НЕТ!" }
        )
    }
}

/// Симуляция анализа торговой системы
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
    println!("=== Метрики качества рефакторинга ===\n");

    // Анализируем каждую версию
    let versions = vec!["v1_prototype", "v2_components", "v3_modular"];

    for version in &versions {
        let metrics = analyze_trading_system(version);
        println!("{}:", version);
        println!("  Строк кода: {}", metrics.lines_of_code);
        println!("  Функций: {}", metrics.functions_count);
        println!("  Средняя длина функции: {:.1}", metrics.avg_function_length);
        println!("  Цикломатическая сложность: {}", metrics.cyclomatic_complexity);
        println!("  Оценка качества: {:.1}/100\n", metrics.quality_score());
    }

    // Отчёт о рефакторинге v1 -> v3
    println!("=== Отчёт о рефакторинге v1 -> v3 ===\n");

    let report = RefactoringReport {
        before: analyze_trading_system("v1_prototype"),
        after: analyze_trading_system("v3_modular"),
        tests_passed_before: 5,
        tests_passed_after: 15,
        total_tests: 15,
    };

    println!("{}", report.summary());
    println!("\nДетали:");
    println!("  Качество до: {:.1}", report.before.quality_score());
    println!("  Качество после: {:.1}", report.after.quality_score());
    println!("  Улучшение: {:.1}%", report.improvement());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Рефакторинг** | Улучшение структуры кода без изменения поведения |
| **Извлечение метода** | Выделение части кода в отдельную функцию |
| **DRY** | Don't Repeat Yourself — устранение дублирования |
| **Single Responsibility** | Каждый модуль отвечает за одну вещь |
| **Type State Pattern** | Использование типов для контроля состояний |
| **Newtype Pattern** | Обёртки для валидации на этапе компиляции |
| **Композиция** | Сборка сложного из простых компонентов |

## Практические задания

1. **Рефакторинг парсера ордеров**: Возьми функцию парсинга ордера из строки и преобразуй её:
   - Выдели валидацию в отдельные функции
   - Создай типобезопасные структуры для Side, Price, Quantity
   - Добавь паттерн Builder для создания ордеров
   - Напиши тесты, доказывающие эквивалентность поведения

2. **Модульный калькулятор комиссий**: Создай систему расчёта комиссий:
   - Определи трейт `FeeCalculator`
   - Реализуй разные стратегии: фиксированная, процентная, тарифная
   - Добавь композицию калькуляторов (maker/taker + объёмные скидки)
   - Убедись, что результаты совпадают с оригинальной логикой

3. **Рефакторинг риск-менеджера**: Преобразуй монолитный риск-менеджер:
   - Выдели проверку размера позиции
   - Выдели проверку максимального убытка
   - Выдели проверку концентрации
   - Используй паттерн Chain of Responsibility

4. **Эволюция бэктестера**: Проведи систему бэктестинга через три версии:
   - V1: простой цикл в одной функции
   - V2: выделенные компоненты (DataSource, Strategy, Executor)
   - V3: полная модульность с трейтами и generics

## Домашнее задание

1. **Полный рефакторинг торгового бота**: Возьми любой торговый код и:
   - Проанализируй "запахи кода"
   - Составь план рефакторинга
   - Выполни рефакторинг поэтапно с коммитами
   - После каждого этапа прогоняй тесты
   - Измерь метрики до и после
   - Задокументируй улучшения

2. **Миграция без даунтайма**: Реализуй паттерн:
   - Новый код работает параллельно со старым
   - Результаты сравниваются (shadow mode)
   - Постепенное переключение трафика
   - Откат при расхождении результатов
   - Полное удаление старого кода

3. **Автоматизация рефакторинга**: Создай инструменты:
   - Детектор дублирующегося кода
   - Анализатор сложности функций
   - Генератор отчётов о качестве
   - Предложения по улучшению

4. **Рефакторинг с сохранением производительности**: Докажи, что рефакторинг не замедлил систему:
   - Напиши бенчмарки до рефакторинга
   - Сохрани baseline результаты
   - Выполни рефакторинг
   - Сравни производительность
   - Оптимизируй, если есть регрессия

## Навигация

[← Предыдущий день](../354-production-logging/ru.md) | [Следующий день →](../365-*/ru.md)
