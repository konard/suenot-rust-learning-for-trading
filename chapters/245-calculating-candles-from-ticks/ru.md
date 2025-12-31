# День 245: Расчёт свечей из тиков

## Аналогия из трейдинга

Представь, что ты стоишь на бирже и наблюдаешь за табло, где каждую секунду меняются цены. Каждая сделка — это **тик**: цена, объём, время. Таких тиков могут быть тысячи в минуту. Смотреть на поток сырых данных невозможно — глаза разбегаются.

Поэтому трейдеры придумали **свечи** (candlesticks). Свеча группирует все тики за определённый период (минуту, час, день) и показывает четыре ключевых значения:
- **Open** (O) — цена первой сделки в периоде
- **High** (H) — максимальная цена за период
- **Low** (L) — минимальная цена за период
- **Close** (C) — цена последней сделки в периоде

Дополнительно свеча может содержать **Volume** — суммарный объём всех сделок за период.

Это как сжать тысячу страниц книги в краткое содержание — теряем детали, но получаем понятную картину движения цены.

## Теория: Агрегация данных в Rust

Расчёт свечей из тиков — классический пример **агрегации данных**. Мы:
1. Группируем данные по временным интервалам
2. Применяем агрегирующие функции (min, max, first, last, sum)
3. Формируем новую структуру данных

В Rust для этого мы используем:
- **Структуры** для представления тиков и свечей
- **Итераторы** для обработки потока данных
- **HashMap** или группировку для агрегации по времени
- **Chrono** для работы с датами и временем

## Базовые структуры данных

```rust
use std::time::{SystemTime, UNIX_EPOCH};

/// Тик — одна сделка на бирже
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,      // Торговый символ (BTC/USDT)
    price: f64,          // Цена сделки
    quantity: f64,       // Объём сделки
    timestamp: u64,      // Unix timestamp в миллисекундах
}

/// OHLCV свеча
#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,           // Цена открытия
    high: f64,           // Максимум
    low: f64,            // Минимум
    close: f64,          // Цена закрытия
    volume: f64,         // Суммарный объём
    open_time: u64,      // Начало периода
    close_time: u64,     // Конец периода
    trade_count: u32,    // Количество сделок
}

impl Candle {
    /// Создаёт новую свечу из первого тика
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    /// Обновляет свечу новым тиком
    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    /// Проверяет, относится ли тик к этой свече
    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}

fn main() {
    // Создаём тестовые тики
    let base_time: u64 = 1700000000000; // Базовое время в миллисекундах

    let ticks = vec![
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 0.5, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42050.0, quantity: 0.3, timestamp: base_time + 1000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41980.0, quantity: 1.2, timestamp: base_time + 2000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42100.0, quantity: 0.8, timestamp: base_time + 3000 },
    ];

    // Создаём минутную свечу (60000 мс = 1 минута)
    let interval_ms: u64 = 60000;
    let mut candle = Candle::new(&ticks[0], interval_ms);

    for tick in ticks.iter().skip(1) {
        if candle.contains(tick) {
            candle.update(tick);
        }
    }

    println!("Свеча: {:?}", candle);
    println!("\nOHLCV: O={} H={} L={} C={} V={}",
        candle.open, candle.high, candle.low, candle.close, candle.volume);
}
```

## Агрегатор свечей в реальном времени

```rust
use std::collections::HashMap;

/// Агрегатор для построения свечей из потока тиков
struct CandleAggregator {
    interval_ms: u64,                        // Интервал свечи в миллисекундах
    current_candles: HashMap<String, Candle>, // Текущие незакрытые свечи
    completed_candles: Vec<Candle>,          // Завершённые свечи
}

impl CandleAggregator {
    fn new(interval_ms: u64) -> Self {
        CandleAggregator {
            interval_ms,
            current_candles: HashMap::new(),
            completed_candles: Vec::new(),
        }
    }

    /// Обрабатывает новый тик
    fn process_tick(&mut self, tick: &Tick) {
        let candle_key = tick.symbol.clone();

        match self.current_candles.get_mut(&candle_key) {
            Some(candle) => {
                if candle.contains(tick) {
                    // Тик относится к текущей свече
                    candle.update(tick);
                } else {
                    // Свеча закрылась, сохраняем её
                    let completed = candle.clone();
                    self.completed_candles.push(completed);

                    // Создаём новую свечу
                    *candle = Candle::new(tick, self.interval_ms);
                }
            }
            None => {
                // Первый тик для этого символа
                let candle = Candle::new(tick, self.interval_ms);
                self.current_candles.insert(candle_key, candle);
            }
        }
    }

    /// Возвращает текущую незакрытую свечу для символа
    fn get_current_candle(&self, symbol: &str) -> Option<&Candle> {
        self.current_candles.get(symbol)
    }

    /// Возвращает все завершённые свечи
    fn get_completed_candles(&self) -> &[Candle] {
        &self.completed_candles
    }
}

fn main() {
    let base_time: u64 = 1700000000000;

    // Симулируем поток тиков за несколько минут
    let ticks = vec![
        // Первая минута
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 0.5, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42100.0, quantity: 0.3, timestamp: base_time + 15000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41900.0, quantity: 1.0, timestamp: base_time + 30000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42050.0, quantity: 0.7, timestamp: base_time + 45000 },
        // Вторая минута
        Tick { symbol: "BTC/USDT".to_string(), price: 42080.0, quantity: 0.4, timestamp: base_time + 60000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42200.0, quantity: 0.6, timestamp: base_time + 75000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42150.0, quantity: 0.9, timestamp: base_time + 90000 },
        // Третья минута
        Tick { symbol: "BTC/USDT".to_string(), price: 42300.0, quantity: 0.5, timestamp: base_time + 120000 },
    ];

    let mut aggregator = CandleAggregator::new(60000); // 1 минута

    for tick in &ticks {
        aggregator.process_tick(tick);
        println!("Обработан тик: цена={}, время={}",
            tick.price, tick.timestamp - base_time);
    }

    println!("\n--- Завершённые свечи ---");
    for (i, candle) in aggregator.get_completed_candles().iter().enumerate() {
        println!("Свеча {}: O={} H={} L={} C={} V={:.1} Сделок={}",
            i + 1, candle.open, candle.high, candle.low,
            candle.close, candle.volume, candle.trade_count);
    }

    if let Some(current) = aggregator.get_current_candle("BTC/USDT") {
        println!("\n--- Текущая свеча (незакрытая) ---");
        println!("O={} H={} L={} C={} V={:.1} Сделок={}",
            current.open, current.high, current.low,
            current.close, current.volume, current.trade_count);
    }
}

#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}

impl Candle {
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}
```

## Множественные таймфреймы

В реальном трейдинге часто нужны свечи разных таймфреймов одновременно:

```rust
use std::collections::HashMap;

/// Мультитаймфреймовый агрегатор
struct MultiTimeframeAggregator {
    aggregators: HashMap<String, CandleAggregator>, // Ключ: "symbol:interval"
}

impl MultiTimeframeAggregator {
    fn new() -> Self {
        MultiTimeframeAggregator {
            aggregators: HashMap::new(),
        }
    }

    /// Добавляет таймфрейм для отслеживания
    fn add_timeframe(&mut self, symbol: &str, interval_ms: u64) {
        let key = format!("{}:{}", symbol, interval_ms);
        self.aggregators.insert(key, CandleAggregator::new(interval_ms));
    }

    /// Обрабатывает тик для всех таймфреймов
    fn process_tick(&mut self, tick: &Tick) {
        for (key, aggregator) in self.aggregators.iter_mut() {
            if key.starts_with(&tick.symbol) {
                aggregator.process_tick(tick);
            }
        }
    }

    /// Получает текущую свечу для конкретного таймфрейма
    fn get_candle(&self, symbol: &str, interval_ms: u64) -> Option<&Candle> {
        let key = format!("{}:{}", symbol, interval_ms);
        self.aggregators.get(&key)?.get_current_candle(symbol)
    }
}

fn main() {
    let base_time: u64 = 1700000000000;

    let mut mtf = MultiTimeframeAggregator::new();

    // Добавляем разные таймфреймы для BTC
    mtf.add_timeframe("BTC/USDT", 60000);    // 1 минута
    mtf.add_timeframe("BTC/USDT", 300000);   // 5 минут
    mtf.add_timeframe("BTC/USDT", 3600000);  // 1 час

    // Генерируем тики
    let ticks = vec![
        Tick { symbol: "BTC/USDT".to_string(), price: 42000.0, quantity: 1.0, timestamp: base_time },
        Tick { symbol: "BTC/USDT".to_string(), price: 42150.0, quantity: 0.5, timestamp: base_time + 30000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 41950.0, quantity: 0.8, timestamp: base_time + 60000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42200.0, quantity: 1.2, timestamp: base_time + 120000 },
        Tick { symbol: "BTC/USDT".to_string(), price: 42300.0, quantity: 0.6, timestamp: base_time + 300000 },
    ];

    for tick in &ticks {
        mtf.process_tick(tick);
    }

    println!("=== Мультитаймфреймовый анализ BTC/USDT ===\n");

    if let Some(candle) = mtf.get_candle("BTC/USDT", 60000) {
        println!("1 минута:  O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }

    if let Some(candle) = mtf.get_candle("BTC/USDT", 300000) {
        println!("5 минут:   O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }

    if let Some(candle) = mtf.get_candle("BTC/USDT", 3600000) {
        println!("1 час:     O={} H={} L={} C={}",
            candle.open, candle.high, candle.low, candle.close);
    }
}

// Структуры повторяются для компиляции
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}

impl Candle {
    fn new(tick: &Tick, interval_ms: u64) -> Self {
        let open_time = (tick.timestamp / interval_ms) * interval_ms;
        Candle {
            symbol: tick.symbol.clone(),
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.quantity,
            open_time,
            close_time: open_time + interval_ms - 1,
            trade_count: 1,
        }
    }

    fn update(&mut self, tick: &Tick) {
        self.high = self.high.max(tick.price);
        self.low = self.low.min(tick.price);
        self.close = tick.price;
        self.volume += tick.quantity;
        self.trade_count += 1;
    }

    fn contains(&self, tick: &Tick) -> bool {
        tick.timestamp >= self.open_time && tick.timestamp <= self.close_time
    }
}

struct CandleAggregator {
    interval_ms: u64,
    current_candles: HashMap<String, Candle>,
    completed_candles: Vec<Candle>,
}

impl CandleAggregator {
    fn new(interval_ms: u64) -> Self {
        CandleAggregator {
            interval_ms,
            current_candles: HashMap::new(),
            completed_candles: Vec::new(),
        }
    }

    fn process_tick(&mut self, tick: &Tick) {
        let candle_key = tick.symbol.clone();
        match self.current_candles.get_mut(&candle_key) {
            Some(candle) => {
                if candle.contains(tick) {
                    candle.update(tick);
                } else {
                    let completed = candle.clone();
                    self.completed_candles.push(completed);
                    *candle = Candle::new(tick, self.interval_ms);
                }
            }
            None => {
                let candle = Candle::new(tick, self.interval_ms);
                self.current_candles.insert(candle_key, candle);
            }
        }
    }

    fn get_current_candle(&self, symbol: &str) -> Option<&Candle> {
        self.current_candles.get(symbol)
    }
}
```

## Расчёт индикаторов на свечах

После формирования свечей можно рассчитывать технические индикаторы:

```rust
/// Простая скользящая средняя (SMA)
fn calculate_sma(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    let sum: f64 = candles.iter()
        .rev()
        .take(period)
        .map(|c| c.close)
        .sum();

    Some(sum / period as f64)
}

/// Экспоненциальная скользящая средняя (EMA)
fn calculate_ema(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = candles[candles.len() - period].close;

    for candle in candles.iter().rev().take(period - 1) {
        ema = (candle.close - ema) * multiplier + ema;
    }

    Some(ema)
}

/// Определение тренда по свечам
fn detect_trend(candles: &[Candle], lookback: usize) -> String {
    if candles.len() < lookback {
        return "Недостаточно данных".to_string();
    }

    let recent: Vec<&Candle> = candles.iter().rev().take(lookback).collect();

    let bullish_count = recent.iter().filter(|c| c.close > c.open).count();
    let bearish_count = recent.iter().filter(|c| c.close < c.open).count();

    let avg_body_size: f64 = recent.iter()
        .map(|c| (c.close - c.open).abs())
        .sum::<f64>() / lookback as f64;

    if bullish_count > bearish_count * 2 {
        format!("Сильный восходящий тренд ({}% бычьих свечей)",
            bullish_count * 100 / lookback)
    } else if bearish_count > bullish_count * 2 {
        format!("Сильный нисходящий тренд ({}% медвежьих свечей)",
            bearish_count * 100 / lookback)
    } else {
        format!("Боковое движение (средний размер тела: {:.2})", avg_body_size)
    }
}

fn main() {
    // Создаём исторические свечи
    let candles = vec![
        Candle { symbol: "BTC/USDT".to_string(), open: 40000.0, high: 40500.0, low: 39800.0, close: 40300.0, volume: 100.0, open_time: 0, close_time: 0, trade_count: 50 },
        Candle { symbol: "BTC/USDT".to_string(), open: 40300.0, high: 41000.0, low: 40200.0, close: 40800.0, volume: 120.0, open_time: 0, close_time: 0, trade_count: 60 },
        Candle { symbol: "BTC/USDT".to_string(), open: 40800.0, high: 41500.0, low: 40700.0, close: 41200.0, volume: 150.0, open_time: 0, close_time: 0, trade_count: 70 },
        Candle { symbol: "BTC/USDT".to_string(), open: 41200.0, high: 42000.0, low: 41100.0, close: 41800.0, volume: 180.0, open_time: 0, close_time: 0, trade_count: 80 },
        Candle { symbol: "BTC/USDT".to_string(), open: 41800.0, high: 42500.0, low: 41600.0, close: 42200.0, volume: 200.0, open_time: 0, close_time: 0, trade_count: 90 },
    ];

    println!("=== Анализ свечей BTC/USDT ===\n");

    if let Some(sma) = calculate_sma(&candles, 3) {
        println!("SMA(3): {:.2}", sma);
    }

    if let Some(ema) = calculate_ema(&candles, 3) {
        println!("EMA(3): {:.2}", ema);
    }

    let trend = detect_trend(&candles, 5);
    println!("\nТренд: {}", trend);

    println!("\n--- Последние свечи ---");
    for (i, candle) in candles.iter().enumerate() {
        let direction = if candle.close > candle.open { "+" } else { "-" };
        println!("Свеча {}: {} O={} C={} ({})",
            i + 1, direction, candle.open, candle.close,
            if candle.close > candle.open { "бычья" } else { "медвежья" });
    }
}

#[derive(Debug, Clone)]
struct Candle {
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    open_time: u64,
    close_time: u64,
    trade_count: u32,
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Тик (Tick) | Одна сделка на бирже с ценой, объёмом и временем |
| OHLCV свеча | Агрегация тиков: Open, High, Low, Close, Volume |
| Интервал свечи | Временной период группировки (1m, 5m, 1h, 1d) |
| Агрегация данных | Группировка и применение функций (min, max, sum) |
| Мультитаймфрейм | Одновременное построение свечей разных интервалов |
| Технические индикаторы | SMA, EMA и другие расчёты на основе свечей |

## Практические задания

1. **Простой агрегатор**: Реализуй функцию `ticks_to_candle(ticks: &[Tick]) -> Candle`, которая преобразует вектор тиков в одну свечу. Проверь на тестовых данных.

2. **Валидация тиков**: Добавь в структуру `Tick` метод `is_valid()`, который проверяет:
   - Цена > 0
   - Объём >= 0
   - Временная метка в разумном диапазоне (не в будущем)

3. **Свечи по объёму**: Создай агрегатор, который формирует свечи не по времени, а по накопленному объёму (например, каждые 100 BTC торговли — новая свеча).

## Домашнее задание

1. **Детектор аномалий**: Напиши функцию, которая анализирует поток тиков и определяет аномальные всплески:
   - Резкое изменение цены (>1% за одну сделку)
   - Аномальный объём (>10x от среднего)
   - Пропуски во времени (>10 секунд без сделок)

2. **VWAP калькулятор**: Реализуй расчёт Volume Weighted Average Price (VWAP) из тиков:
   ```
   VWAP = Сумма(Цена × Объём) / Сумма(Объём)
   ```
   Отслеживай VWAP в реальном времени при поступлении новых тиков.

3. **Свечной паттерн**: Реализуй распознавание паттерна "Поглощение" (Engulfing):
   - Бычье поглощение: маленькая красная свеча, затем большая зелёная
   - Медвежье поглощение: маленькая зелёная свеча, затем большая красная

4. **Экспорт данных**: Создай функцию для экспорта свечей в CSV формат:
   ```
   timestamp,open,high,low,close,volume
   1700000000000,42000.0,42100.0,41900.0,42050.0,2.5
   ```

## Навигация

[← Предыдущий день](../244-tick-data-processing/ru.md) | [Следующий день →](../246-order-book-aggregation/ru.md)
