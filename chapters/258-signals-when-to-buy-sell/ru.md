# День 258: Сигналы: когда покупать/продавать

## Аналогия из трейдинга

Представь, что ты стоишь у экрана с биржевыми котировками. Цены постоянно меняются — вверх, вниз, в боковике. Как понять, когда именно нужно действовать? Опытные трейдеры используют **сигналы** — определённые условия или комбинации факторов, которые указывают на благоприятный момент для покупки или продажи.

Торговый сигнал — это как светофор на перекрёстке: зелёный свет говорит «покупай», красный — «продавай», жёлтый — «жди и наблюдай». В алгоритмическом трейдинге мы программируем эти «светофоры», чтобы они автоматически анализировали рынок и подавали сигналы.

В этой главе мы научимся:
- Моделировать торговые сигналы в Rust
- Создавать генераторы сигналов на основе индикаторов
- Управлять ордерами по сигналам
- Учитывать риски при принятии решений

## Что такое торговый сигнал?

Торговый сигнал — это результат анализа рыночных данных, указывающий на потенциальную торговую возможность. В Rust мы можем моделировать сигналы с помощью перечислений (enum):

```rust
/// Тип торгового сигнала
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Сильный сигнал на покупку
    StrongBuy,
    /// Слабый сигнал на покупку
    Buy,
    /// Нейтральная позиция, ждём
    Hold,
    /// Слабый сигнал на продажу
    Sell,
    /// Сильный сигнал на продажу
    StrongSell,
}

impl Signal {
    /// Возвращает силу сигнала от -2 до 2
    pub fn strength(&self) -> i32 {
        match self {
            Signal::StrongBuy => 2,
            Signal::Buy => 1,
            Signal::Hold => 0,
            Signal::Sell => -1,
            Signal::StrongSell => -2,
        }
    }

    /// Проверяет, является ли сигнал бычьим (на покупку)
    pub fn is_bullish(&self) -> bool {
        matches!(self, Signal::StrongBuy | Signal::Buy)
    }

    /// Проверяет, является ли сигнал медвежьим (на продажу)
    pub fn is_bearish(&self) -> bool {
        matches!(self, Signal::Sell | Signal::StrongSell)
    }
}
```

## Структура торгового сигнала

Простой enum недостаточен для реальной торговли. Нам нужна более полная структура:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

/// Полная информация о торговом сигнале
#[derive(Debug, Clone)]
pub struct TradingSignal {
    /// Тикер инструмента
    pub symbol: String,
    /// Тип сигнала
    pub signal: Signal,
    /// Сила сигнала от 0.0 до 1.0
    pub confidence: f64,
    /// Рекомендуемая цена входа
    pub entry_price: f64,
    /// Уровень стоп-лосса
    pub stop_loss: Option<f64>,
    /// Уровень тейк-профита
    pub take_profit: Option<f64>,
    /// Временная метка генерации сигнала
    pub timestamp: u64,
    /// Источник сигнала (какой индикатор/стратегия)
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

    /// Добавляет уровни риск-менеджмента
    pub fn with_risk_levels(mut self, stop_loss: f64, take_profit: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self.take_profit = Some(take_profit);
        self
    }

    /// Рассчитывает соотношение риск/прибыль
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

## Генератор сигналов на основе индикаторов

Теперь создадим генератор сигналов, который анализирует цены и выдаёт торговые рекомендации:

```rust
/// Данные свечи (OHLCV)
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Генератор сигналов на основе скользящих средних
pub struct MovingAverageCrossover {
    fast_period: usize,
    slow_period: usize,
}

impl MovingAverageCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        MovingAverageCrossover { fast_period, slow_period }
    }

    /// Рассчитывает простую скользящую среднюю
    fn sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    /// Генерирует сигнал на основе пересечения скользящих средних
    pub fn generate_signal(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal> {
        if candles.len() < self.slow_period + 1 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let current_close = *closes.last()?;

        // Текущие значения MA
        let fast_ma = Self::sma(&closes, self.fast_period)?;
        let slow_ma = Self::sma(&closes, self.slow_period)?;

        // Предыдущие значения MA
        let prev_closes: Vec<f64> = closes[..closes.len() - 1].to_vec();
        let prev_fast_ma = Self::sma(&prev_closes, self.fast_period)?;
        let prev_slow_ma = Self::sma(&prev_closes, self.slow_period)?;

        // Определяем пересечение
        let signal = if prev_fast_ma <= prev_slow_ma && fast_ma > slow_ma {
            // Золотой крест — сигнал на покупку
            Signal::Buy
        } else if prev_fast_ma >= prev_slow_ma && fast_ma < slow_ma {
            // Мёртвый крест — сигнал на продажу
            Signal::Sell
        } else {
            Signal::Hold
        };

        if signal == Signal::Hold {
            return None;
        }

        // Рассчитываем уверенность на основе расхождения MA
        let divergence = ((fast_ma - slow_ma) / slow_ma).abs();
        let confidence = (divergence * 100.0).min(1.0);

        let mut trading_signal = TradingSignal::new(
            symbol,
            signal,
            confidence,
            current_close,
            "MA_Crossover",
        );

        // Добавляем уровни стоп-лосса и тейк-профита
        let atr = self.calculate_atr(candles, 14)?;
        if signal.is_bullish() {
            trading_signal = trading_signal.with_risk_levels(
                current_close - atr * 2.0,  // Стоп-лосс
                current_close + atr * 3.0,  // Тейк-профит
            );
        } else {
            trading_signal = trading_signal.with_risk_levels(
                current_close + atr * 2.0,  // Стоп-лосс
                current_close - atr * 3.0,  // Тейк-профит
            );
        }

        Some(trading_signal)
    }

    /// Рассчитывает Average True Range
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

## Агрегатор сигналов

В реальной торговле часто используют несколько индикаторов одновременно. Создадим агрегатор, который объединяет сигналы:

```rust
use std::collections::HashMap;

/// Трейт для генераторов сигналов
pub trait SignalGenerator {
    fn generate(&self, candles: &[Candle], symbol: &str) -> Option<TradingSignal>;
    fn name(&self) -> &str;
}

/// Агрегатор сигналов от нескольких источников
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

    /// Добавляет генератор сигналов с весом
    pub fn add_generator(&mut self, generator: Box<dyn SignalGenerator>, weight: f64) {
        let name = generator.name().to_string();
        self.generators.push(generator);
        self.weights.insert(name, weight);
    }

    /// Генерирует агрегированный сигнал
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

        // Определяем итоговый сигнал на основе среднего балла
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

        // Усредняем параметры сигналов
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

## Управление ордерами по сигналам

Теперь создадим систему, которая преобразует сигналы в реальные ордера:

```rust
/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Статус ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

/// Ордер
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

/// Позиция в портфеле
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub unrealized_pnl: f64,
}

/// Менеджер ордеров на основе сигналов
pub struct SignalOrderManager {
    next_order_id: u64,
    max_position_size: f64,
    max_risk_per_trade: f64,  // В процентах от портфеля
    portfolio_value: f64,
    positions: HashMap<String, Position>,
    pending_orders: Vec<Order>,
}

impl SignalOrderManager {
    pub fn new(portfolio_value: f64, max_risk_per_trade: f64) -> Self {
        SignalOrderManager {
            next_order_id: 1,
            max_position_size: portfolio_value * 0.1,  // Макс 10% на позицию
            max_risk_per_trade,
            portfolio_value,
            positions: HashMap::new(),
            pending_orders: Vec::new(),
        }
    }

    /// Обрабатывает торговый сигнал
    pub fn process_signal(&mut self, signal: &TradingSignal) -> Option<Order> {
        // Проверяем минимальную уверенность
        if signal.confidence < 0.3 {
            println!("Сигнал отклонён: низкая уверенность ({:.2})", signal.confidence);
            return None;
        }

        // Проверяем соотношение риск/прибыль
        if let Some(rr) = signal.risk_reward_ratio() {
            if rr < 1.5 {
                println!("Сигнал отклонён: низкое R/R ({:.2})", rr);
                return None;
            }
        }

        // Определяем сторону ордера
        let side = if signal.signal.is_bullish() {
            OrderSide::Buy
        } else if signal.signal.is_bearish() {
            OrderSide::Sell
        } else {
            return None;
        };

        // Рассчитываем размер позиции на основе риска
        let quantity = self.calculate_position_size(signal, side);
        if quantity <= 0.0 {
            println!("Сигнал отклонён: размер позиции равен нулю");
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
            "Создан ордер #{}: {:?} {} {} @ {:.2}",
            order.id, order.side, order.quantity, order.symbol, order.price
        );

        Some(order)
    }

    /// Рассчитывает размер позиции на основе риска
    fn calculate_position_size(&self, signal: &TradingSignal, side: OrderSide) -> f64 {
        let stop_loss = match signal.stop_loss {
            Some(sl) => sl,
            None => return 0.0,  // Без стоп-лосса не торгуем
        };

        // Риск на сделку в деньгах
        let risk_amount = self.portfolio_value * (self.max_risk_per_trade / 100.0);

        // Риск на единицу актива
        let risk_per_unit = match side {
            OrderSide::Buy => (signal.entry_price - stop_loss).abs(),
            OrderSide::Sell => (stop_loss - signal.entry_price).abs(),
        };

        if risk_per_unit <= 0.0 {
            return 0.0;
        }

        // Размер позиции
        let position_size = risk_amount / risk_per_unit;

        // Ограничиваем максимальным размером позиции
        let max_quantity = self.max_position_size / signal.entry_price;
        position_size.min(max_quantity)
    }

    /// Обновляет нереализованную прибыль/убыток
    pub fn update_pnl(&mut self, symbol: &str, current_price: f64) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.unrealized_pnl =
                (current_price - position.avg_entry_price) * position.quantity;
        }
    }

    /// Получает общий P&L портфеля
    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }
}
```

## RSI-генератор сигналов

Добавим ещё один популярный индикатор — RSI (Relative Strength Index):

```rust
/// Генератор сигналов на основе RSI
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

    /// Рассчитывает RSI
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
            // Перепроданность — сигнал на покупку
            Signal::Buy
        } else if rsi > self.overbought {
            // Перекупленность — сигнал на продажу
            Signal::Sell
        } else {
            return None;
        };

        // Уверенность зависит от силы отклонения
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

## Практический пример: Полная система сигналов

```rust
use std::collections::VecDeque;

/// Полная система генерации и исполнения сигналов
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

        // Добавляем генераторы сигналов
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

    /// Обрабатывает новую свечу
    pub fn on_candle(&mut self, symbol: &str, candle: Candle) -> Option<Order> {
        // Добавляем свечу в буфер
        let buffer = self.candle_buffer
            .entry(symbol.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.buffer_size));

        if buffer.len() >= self.buffer_size {
            buffer.pop_front();
        }
        buffer.push_back(candle);

        // Генерируем сигнал
        let candles: Vec<Candle> = buffer.iter().cloned().collect();
        let signal = self.aggregator.generate_signal(&candles, symbol)?;

        println!(
            "[{}] Сигнал: {:?} (уверенность: {:.2}%, R/R: {:?})",
            symbol,
            signal.signal,
            signal.confidence * 100.0,
            signal.risk_reward_ratio()
        );

        self.signal_history.push(signal.clone());

        // Преобразуем сигнал в ордер
        self.order_manager.process_signal(&signal)
    }

    /// Возвращает статистику сигналов
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

// Обёртка для MovingAverageCrossover для реализации трейта
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

    // Симулируем поток свечей
    let test_candles = vec![
        Candle { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0 },
        Candle { open: 101.0, high: 103.0, low: 100.0, close: 102.5, volume: 1200.0 },
        Candle { open: 102.5, high: 105.0, low: 102.0, close: 104.0, volume: 1500.0 },
        Candle { open: 104.0, high: 106.0, low: 103.0, close: 105.5, volume: 1800.0 },
        Candle { open: 105.5, high: 107.0, low: 104.5, close: 106.0, volume: 2000.0 },
        // ... добавьте больше свечей для реального тестирования
    ];

    for (i, candle) in test_candles.into_iter().enumerate() {
        println!("\n--- Свеча #{} ---", i + 1);
        if let Some(order) = system.on_candle("BTC/USDT", candle) {
            println!("Ордер создан: {:?}", order);
        }
    }

    println!("\n--- Статистика сигналов ---");
    let stats = system.signal_statistics();
    println!("{:?}", stats);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Signal` enum | Перечисление типов торговых сигналов (Buy, Sell, Hold) |
| `TradingSignal` | Структура с полной информацией о сигнале |
| Генератор сигналов | Компонент, анализирующий данные и создающий сигналы |
| Агрегатор | Объединяет сигналы от нескольких источников |
| Риск-менеджмент | Расчёт размера позиции на основе допустимого риска |
| R/R ratio | Соотношение потенциальной прибыли к риску |
| Трейт `SignalGenerator` | Абстракция для различных стратегий генерации сигналов |

## Домашнее задание

1. **MACD-генератор**: Реализуй генератор сигналов на основе индикатора MACD (Moving Average Convergence Divergence). Сигнал на покупку — когда линия MACD пересекает сигнальную линию снизу вверх, на продажу — сверху вниз.

2. **Фильтр волатильности**: Добавь в систему фильтр, который отклоняет сигналы при слишком высокой или слишком низкой волатильности. Используй ATR для измерения волатильности.

3. **Многотаймфреймовый анализ**: Модифицируй `SignalAggregator`, чтобы он мог учитывать сигналы с разных таймфреймов (например, 1 час и 4 часа). Сигнал считается сильным, если он подтверждён на обоих таймфреймах.

4. **Система оповещений**: Создай структуру `AlertSystem`, которая:
   - Подписывается на сигналы от `TradingSystem`
   - Фильтрует сигналы по заданным критериям
   - Формирует текстовые уведомления о важных сигналах
   - Ведёт лог всех оповещений

## Навигация

[← Предыдущий день](../257-strategy-pattern-trading/ru.md) | [Следующий день →](../259-backtesting-signals/ru.md)
