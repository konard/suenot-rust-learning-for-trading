# День 244: OHLCV-свечи: базовая структура

## Аналогия из трейдинга

Представь, что ты наблюдаешь за оживлённым торговым залом. Каждую секунду цены колеблются — то вверх, то вниз. Но как зафиксировать весь этот хаос осмысленным образом? Ты не можешь отслеживать каждый тик, поэтому трейдеры используют гениальную абстракцию: **OHLCV-свечу**.

Представь OHLCV-свечу как "сводный отчёт" за период торговой активности:
- **Open (Открытие)**: С чего мы начали? (Первая сделка периода)
- **High (Максимум)**: Как высоко мы поднялись? (Максимальная цена)
- **Low (Минимум)**: Как низко мы опустились? (Минимальная цена)
- **Close (Закрытие)**: Чем мы закончили? (Последняя сделка периода)
- **Volume (Объём)**: Сколько было активности? (Общий объём торгов)

Это как прогноз погоды для цен: вместо того чтобы говорить "температура была 20°C в 9:00, потом 21°C в 9:01, потом 20.5°C в 9:02...", ты просто говоришь "Температура колебалась от 18°C до 25°C, начиная с 20°C и заканчивая 22°C."

В алгоритмическом трейдинге OHLCV-свечи — это фундаментальные строительные блоки для:
- Технического анализа и индикаторов (RSI, MACD, полосы Боллинджера)
- Распознавания паттернов (поглощение, доджи, молот)
- Бэктестинга торговых стратегий
- Построения графиков и визуализаций

## Что такое OHLCV-свеча?

OHLCV-свеча представляет агрегированные ценовые данные за определённый временной интервал. Распространённые интервалы включают:
- **1 минута (1m)**: Для скальпинга и высокочастотных стратегий
- **5 минут (5m)**: Краткосрочная торговля
- **1 час (1h)**: Внутридневной анализ
- **1 день (1D)**: Свинг-трейдинг и ежедневный анализ
- **1 неделя (1W)**: Долгосрочные тренды

```
        │
        │ ← High (максимальная цена за период)
    ┌───┴───┐
    │       │
    │       │ ← Тело (показывает Open vs Close)
    │       │
    └───┬───┘
        │
        │ ← Low (минимальная цена за период)
        │

    Зелёная свеча: Close > Open (цена выросла)
    Красная свеча: Close < Open (цена упала)
```

## Базовая структура OHLCV в Rust

Определим фундаментальную структуру:

```rust
/// Представляет одну OHLCV-свечу
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    /// Цена открытия периода
    pub open: f64,
    /// Максимальная цена за период
    pub high: f64,
    /// Минимальная цена за период
    pub low: f64,
    /// Цена закрытия периода
    pub close: f64,
    /// Объём торгов за период
    pub volume: f64,
    /// Временная метка (Unix timestamp в миллисекундах)
    pub timestamp: u64,
}

impl Candle {
    /// Создаёт новую свечу с валидацией
    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        timestamp: u64,
    ) -> Result<Self, String> {
        // Валидация OHLCV-данных
        if high < low {
            return Err("Максимум не может быть меньше минимума".to_string());
        }
        if high < open || high < close {
            return Err("Максимум должен быть >= открытию и закрытию".to_string());
        }
        if low > open || low > close {
            return Err("Минимум должен быть <= открытию и закрытию".to_string());
        }
        if volume < 0.0 {
            return Err("Объём не может быть отрицательным".to_string());
        }

        Ok(Candle {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        })
    }

    /// Возвращает true, если это бычья (зелёная) свеча
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Возвращает true, если это медвежья (красная) свеча
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Возвращает размер тела (разница между открытием и закрытием)
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Возвращает полный диапазон (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Возвращает размер верхней тени
    pub fn upper_wick(&self) -> f64 {
        self.high - self.close.max(self.open)
    }

    /// Возвращает размер нижней тени
    pub fn lower_wick(&self) -> f64 {
        self.close.min(self.open) - self.low
    }
}

fn main() {
    // Создаём бычью свечу BTC/USDT
    let btc_candle = Candle::new(
        42000.0,  // Open
        42500.0,  // High
        41800.0,  // Low
        42300.0,  // Close
        150.5,    // Объём в BTC
        1703980800000, // Timestamp: 31 декабря 2024 00:00:00 UTC
    ).expect("Некорректные данные свечи");

    println!("Свеча BTC/USDT: {:?}", btc_candle);
    println!("Бычья: {}", btc_candle.is_bullish());
    println!("Размер тела: ${:.2}", btc_candle.body_size());
    println!("Диапазон: ${:.2}", btc_candle.range());
    println!("Верхняя тень: ${:.2}", btc_candle.upper_wick());
    println!("Нижняя тень: ${:.2}", btc_candle.lower_wick());
}
```

## Использование Decimal для финансовой точности

Для реальных торговых приложений проблемы точности с плавающей точкой могут быть критичными. Рассмотрим использование крейта `rust_decimal`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq)]
pub struct PreciseCandle {
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub timestamp: u64,
}

impl PreciseCandle {
    pub fn new(
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        timestamp: u64,
    ) -> Result<Self, String> {
        if high < low {
            return Err("Максимум не может быть меньше минимума".to_string());
        }
        if volume < Decimal::ZERO {
            return Err("Объём не может быть отрицательным".to_string());
        }

        Ok(PreciseCandle {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        })
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn body_size(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Вычисляет процентное изменение от открытия к закрытию
    pub fn percent_change(&self) -> Decimal {
        if self.open == Decimal::ZERO {
            return Decimal::ZERO;
        }
        ((self.close - self.open) / self.open) * dec!(100)
    }
}

fn main() {
    let eth_candle = PreciseCandle::new(
        dec!(2250.50),
        dec!(2280.75),
        dec!(2240.25),
        dec!(2275.00),
        dec!(5000.123456),
        1703980800000,
    ).expect("Некорректная свеча");

    println!("Свеча ETH: {:?}", eth_candle);
    println!("Изменение: {:.4}%", eth_candle.percent_change());
}
```

## Построение свечей из тиковых данных

На практике вы часто получаете отдельные сделки (тики) и должны агрегировать их в свечи:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct CandleBuilder {
    interval_ms: u64,        // Интервал свечи в миллисекундах
    current_open: Option<f64>,
    current_high: f64,
    current_low: f64,
    current_close: f64,
    current_volume: f64,
    period_start: u64,
    completed_candles: VecDeque<Candle>,
}

impl CandleBuilder {
    /// Создаёт новый построитель свечей для указанного интервала
    pub fn new(interval_ms: u64) -> Self {
        CandleBuilder {
            interval_ms,
            current_open: None,
            current_high: f64::MIN,
            current_low: f64::MAX,
            current_close: 0.0,
            current_volume: 0.0,
            period_start: 0,
            completed_candles: VecDeque::new(),
        }
    }

    /// Добавляет сделку и потенциально завершает свечу
    pub fn add_trade(&mut self, trade: &Trade) -> Option<Candle> {
        let trade_period = (trade.timestamp / self.interval_ms) * self.interval_ms;

        // Проверяем, нужно ли закрыть текущую свечу и начать новую
        if self.current_open.is_some() && trade_period > self.period_start {
            let completed = self.finalize_candle();
            self.start_new_candle(trade, trade_period);
            return completed;
        }

        // Начинаем новую свечу, если её нет
        if self.current_open.is_none() {
            self.start_new_candle(trade, trade_period);
            return None;
        }

        // Обновляем текущую свечу
        self.update_candle(trade);
        None
    }

    fn start_new_candle(&mut self, trade: &Trade, period_start: u64) {
        self.current_open = Some(trade.price);
        self.current_high = trade.price;
        self.current_low = trade.price;
        self.current_close = trade.price;
        self.current_volume = trade.quantity;
        self.period_start = period_start;
    }

    fn update_candle(&mut self, trade: &Trade) {
        self.current_high = self.current_high.max(trade.price);
        self.current_low = self.current_low.min(trade.price);
        self.current_close = trade.price;
        self.current_volume += trade.quantity;
    }

    fn finalize_candle(&mut self) -> Option<Candle> {
        if let Some(open) = self.current_open {
            let candle = Candle {
                open,
                high: self.current_high,
                low: self.current_low,
                close: self.current_close,
                volume: self.current_volume,
                timestamp: self.period_start,
            };
            self.current_open = None;
            return Some(candle);
        }
        None
    }
}

fn main() {
    // Создаём построитель минутных свечей
    let mut builder = CandleBuilder::new(60_000); // 60 секунд в мс

    // Симулируем входящие сделки
    let trades = vec![
        Trade { price: 42000.0, quantity: 0.5, timestamp: 1703980800000 },
        Trade { price: 42050.0, quantity: 0.3, timestamp: 1703980815000 },
        Trade { price: 42100.0, quantity: 0.8, timestamp: 1703980830000 },
        Trade { price: 41950.0, quantity: 0.2, timestamp: 1703980845000 },
        Trade { price: 42020.0, quantity: 0.6, timestamp: 1703980858000 },
        // Начинается новая минута
        Trade { price: 42030.0, quantity: 0.4, timestamp: 1703980860000 },
    ];

    println!("Обработка сделок...\n");

    for trade in &trades {
        println!("Сделка: ${:.2} x {:.2} @ {}", trade.price, trade.quantity, trade.timestamp);

        if let Some(candle) = builder.add_trade(trade) {
            println!("\n=== Завершена минутная свеча ===");
            println!("  Open:   ${:.2}", candle.open);
            println!("  High:   ${:.2}", candle.high);
            println!("  Low:    ${:.2}", candle.low);
            println!("  Close:  ${:.2}", candle.close);
            println!("  Volume: {:.2}", candle.volume);
            println!("  Бычья: {}\n", candle.is_bullish());
        }
    }
}
```

## Практический пример: простой анализ свечей

Проанализируем серию свечей для выявления паттернов:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: u64,
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

    /// Проверяет, является ли свеча доджи (маленькое тело относительно диапазона)
    pub fn is_doji(&self, threshold: f64) -> bool {
        if self.range() == 0.0 {
            return true;
        }
        self.body_size() / self.range() < threshold
    }

    /// Проверяет, является ли свеча молотом (длинная нижняя тень, короткая верхняя)
    pub fn is_hammer(&self, body_ratio: f64) -> bool {
        let body = self.body_size();
        let lower_wick = self.close.min(self.open) - self.low;
        let upper_wick = self.high - self.close.max(self.open);

        lower_wick > body * 2.0 && upper_wick < body * body_ratio
    }
}

/// Анализирует серию свечей
pub struct CandleAnalyzer {
    candles: Vec<Candle>,
}

impl CandleAnalyzer {
    pub fn new(candles: Vec<Candle>) -> Self {
        CandleAnalyzer { candles }
    }

    /// Вычисляет простую скользящую среднюю цен закрытия
    pub fn sma(&self, period: usize) -> Vec<f64> {
        if self.candles.len() < period {
            return vec![];
        }

        let mut result = Vec::new();
        for i in (period - 1)..self.candles.len() {
            let sum: f64 = self.candles[(i + 1 - period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            result.push(sum / period as f64);
        }
        result
    }

    /// Находит паттерны бычьего поглощения
    pub fn find_bullish_engulfing(&self) -> Vec<usize> {
        let mut patterns = Vec::new();

        for i in 1..self.candles.len() {
            let prev = &self.candles[i - 1];
            let curr = &self.candles[i];

            // Бычье поглощение: предыдущая медвежья, текущая бычья,
            // тело текущей полностью покрывает тело предыдущей
            if prev.is_bearish() && curr.is_bullish()
                && curr.open < prev.close && curr.close > prev.open
            {
                patterns.push(i);
            }
        }
        patterns
    }

    /// Вычисляет средний объём
    pub fn average_volume(&self) -> f64 {
        if self.candles.is_empty() {
            return 0.0;
        }
        self.candles.iter().map(|c| c.volume).sum::<f64>() / self.candles.len() as f64
    }

    /// Находит свечи с высоким объёмом (выше среднего * множитель)
    pub fn high_volume_candles(&self, multiplier: f64) -> Vec<usize> {
        let avg = self.average_volume();
        self.candles
            .iter()
            .enumerate()
            .filter(|(_, c)| c.volume > avg * multiplier)
            .map(|(i, _)| i)
            .collect()
    }
}

fn main() {
    // Примерные данные свечей (симуляция часовых свечей BTC/USDT)
    let candles = vec![
        Candle { open: 42000.0, high: 42200.0, low: 41800.0, close: 41900.0, volume: 100.0, timestamp: 0 },
        Candle { open: 41900.0, high: 42100.0, low: 41700.0, close: 41750.0, volume: 120.0, timestamp: 1 },
        Candle { open: 41750.0, high: 42300.0, low: 41600.0, close: 42250.0, volume: 200.0, timestamp: 2 },
        Candle { open: 42250.0, high: 42400.0, low: 42100.0, close: 42350.0, volume: 150.0, timestamp: 3 },
        Candle { open: 42350.0, high: 42600.0, low: 42300.0, close: 42550.0, volume: 180.0, timestamp: 4 },
    ];

    let analyzer = CandleAnalyzer::new(candles.clone());

    println!("=== Анализ свечей ===\n");

    // Анализ отдельных свечей
    for (i, candle) in candles.iter().enumerate() {
        let trend = if candle.is_bullish() { "Бычья" } else { "Медвежья" };
        let doji = if candle.is_doji(0.1) { " (Доджи)" } else { "" };
        println!(
            "Свеча {}: {} ${:.0} -> ${:.0} | Диапазон: ${:.0} | Объём: {:.0}{}",
            i, trend, candle.open, candle.close, candle.range(), candle.volume, doji
        );
    }

    // Расчёт SMA
    println!("\nSMA за 3 периода: {:?}", analyzer.sma(3));

    // Поиск паттернов
    let engulfing = analyzer.find_bullish_engulfing();
    println!("Паттерны бычьего поглощения на индексах: {:?}", engulfing);

    // Анализ объёма
    println!("Средний объём: {:.2}", analyzer.average_volume());
    println!("Свечи с высоким объёмом (>1.5x от среднего): {:?}", analyzer.high_volume_candles(1.5));
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| OHLCV | Open, High, Low, Close, Volume — пять ключевых точек данных свечи |
| Свеча (Candlestick) | Визуальное представление ценового действия за период времени |
| Бычья/Медвежья | Указывает, выросла (бычья) или упала (медвежья) цена |
| Тело | Разница между ценами открытия и закрытия |
| Тени (Wicks) | Линии выше и ниже тела, показывающие ценовые экстремумы |
| Агрегация тиков | Построение свечей из отдельных сделок |
| Технические индикаторы | Расчёты на основе данных свечей (SMA, паттерны) |

## Упражнения

1. **Валидация свечей**: Расширь функцию `Candle::new()`, чтобы также проверять, что цены положительные и что timestamp не равен нулю.

2. **Сериализация свечей**: Реализуй `serde::Serialize` и `serde::Deserialize` для структуры `Candle`, чтобы можно было сохранять/загружать данные свечей в/из JSON-файлов.

3. **Конвертация таймфреймов**: Напиши функцию, которая принимает минутные свечи и агрегирует их в 5-минутные. Функция должна правильно рассчитывать новые значения OHLCV.

4. **Поиск паттернов**: Реализуй обнаружение паттерна "Три белых солдата" (три последовательные бычьи свечи с более высокими закрытиями).

## Домашнее задание

1. **Полноценный построитель свечей**: Расширь `CandleBuilder` для обработки пропусков в данных (отсутствующие сделки) и генерации пустых свечей для непрерывных графиков.

2. **Профиль объёма**: Создай функцию, которая группирует свечи по ценовым уровням и вычисляет общий объём, проторгованный на каждом уровне. Это полезно для определения зон поддержки/сопротивления.

3. **Статистика свечей**: Реализуй структуру `CandleStats`, которая вычисляет:
   - Средний размер тела
   - Средний диапазон
   - Соотношение бычьих/медвежьих свечей
   - Наиболее частый ценовой диапазон

4. **Агрегация в реальном времени**: Модифицируй `CandleBuilder` для работы с данными WebSocket в реальном времени. Симулируй поток сделок и отображай формирующиеся свечи в реальном времени.

## Навигация

[← Предыдущий день](../243-order-book-matching/ru.md) | [Следующий день →](../245-ohlcv-timeframe-conversion/ru.md)
