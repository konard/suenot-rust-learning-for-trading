# День 298: Мульти-таймфрейм тестирование

## Аналогия из трейдинга

Представь, что у тебя есть торговая стратегия, которая отлично работает на 1-часовых свечах. Ты бэктестируешь её и видишь стабильную прибыль! Но когда запускаешь её в боевых условиях, стратегия не работает. Почему?

Проблема в том, что ты игнорировал **общую картину рынка**. Твоя стратегия могла открывать длинные позиции на 1-часовом графике, в то время как дневной график показывал нисходящий тренд. Это как смотреть на дерево и не видеть леса.

**Мульти-таймфрейм анализ** — это как смотреть на карту города с разных высот:
- **Месячный график** (высота 10км) — глобальный тренд рынка
- **Недельный график** (высота 1км) — среднесрочный тренд
- **Дневной график** (высота 100м) — краткосрочный тренд
- **Часовой график** (высота 10м) — тактический вход/выход

Лучшие трейдеры анализируют все уровни одновременно: определяют направление на старшем таймфрейме, а точки входа ищут на младшем.

## Что такое мульти-таймфрейм тестирование?

Мульти-таймфрейм тестирование — это бэктестинг торговой стратегии с учётом нескольких временных периодов одновременно. Это позволяет:

1. **Подтвердить направление тренда** — старшие таймфреймы показывают основное направление
2. **Улучшить точность входа** — младшие таймфреймы дают более точные сигналы
3. **Снизить ложные сигналы** — фильтрация сигналов через несколько таймфреймов
4. **Управлять рисками** — лучшее понимание волатильности на разных уровнях

### Концепция таймфреймов

```
Месяц    [====================] Долгосрочный тренд
Неделя     [====][====][====]  Среднесрочные волны
День         [][][][][][][][]  Краткосрочные движения
Час          ................ Микро-флуктуации
```

## Базовая структура для мульти-таймфрейм данных

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeFrame {
    M1,   // 1 минута
    M5,   // 5 минут
    M15,  // 15 минут
    H1,   // 1 час
    H4,   // 4 часа
    D1,   // 1 день
    W1,   // 1 неделя
}

impl TimeFrame {
    fn to_seconds(&self) -> i64 {
        match self {
            TimeFrame::M1 => 60,
            TimeFrame::M5 => 300,
            TimeFrame::M15 => 900,
            TimeFrame::H1 => 3600,
            TimeFrame::H4 => 14400,
            TimeFrame::D1 => 86400,
            TimeFrame::W1 => 604800,
        }
    }

    fn name(&self) -> &str {
        match self {
            TimeFrame::M1 => "1m",
            TimeFrame::M5 => "5m",
            TimeFrame::M15 => "15m",
            TimeFrame::H1 => "1h",
            TimeFrame::H4 => "4h",
            TimeFrame::D1 => "1d",
            TimeFrame::W1 => "1w",
        }
    }
}

struct MultiTimeFrameData {
    symbol: String,
    data: HashMap<TimeFrame, Vec<Candle>>,
}

impl MultiTimeFrameData {
    fn new(symbol: &str) -> Self {
        MultiTimeFrameData {
            symbol: symbol.to_string(),
            data: HashMap::new(),
        }
    }

    fn add_candles(&mut self, timeframe: TimeFrame, candles: Vec<Candle>) {
        self.data.insert(timeframe, candles);
    }

    fn get_candles(&self, timeframe: TimeFrame) -> Option<&Vec<Candle>> {
        self.data.get(&timeframe)
    }

    // Агрегация младшего таймфрейма в старший
    fn aggregate_timeframe(
        &self,
        from: TimeFrame,
        to: TimeFrame,
    ) -> Option<Vec<Candle>> {
        let base_candles = self.data.get(&from)?;
        let ratio = to.to_seconds() / from.to_seconds();

        if ratio <= 1 {
            return None; // Невозможно агрегировать в меньший таймфрейм
        }

        let mut aggregated = Vec::new();
        let mut i = 0;

        while i < base_candles.len() {
            let chunk_size = ratio.min((base_candles.len() - i) as i64) as usize;
            let chunk = &base_candles[i..i + chunk_size];

            if chunk.is_empty() {
                break;
            }

            let open = chunk[0].open;
            let close = chunk[chunk.len() - 1].close;
            let high = chunk.iter().map(|c| c.high).fold(f64::MIN, f64::max);
            let low = chunk.iter().map(|c| c.low).fold(f64::MAX, f64::min);
            let volume = chunk.iter().map(|c| c.volume).sum();

            aggregated.push(Candle {
                timestamp: chunk[0].timestamp,
                open,
                high,
                low,
                close,
                volume,
            });

            i += chunk_size;
        }

        Some(aggregated)
    }
}
```

## Индикаторы для мульти-таймфрейм анализа

```rust
fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in 0..prices.len() {
        if i < period - 1 {
            sma.push(0.0);
        } else {
            let sum: f64 = prices[i - period + 1..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }

    sma
}

fn exponential_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return Vec::new();
    }

    let mut ema = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Первое значение EMA = SMA
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut current_ema = initial_sum / period as f64;

    for _ in 0..period - 1 {
        ema.push(0.0);
    }
    ema.push(current_ema);

    for i in period..prices.len() {
        current_ema = (prices[i] - current_ema) * multiplier + current_ema;
        ema.push(current_ema);
    }

    ema
}

#[derive(Debug, Clone, Copy)]
enum Trend {
    Bullish,   // Восходящий
    Bearish,   // Нисходящий
    Sideways,  // Боковой
}

fn detect_trend(candles: &[Candle], sma_period: usize) -> Trend {
    if candles.len() < sma_period {
        return Trend::Sideways;
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let sma = simple_moving_average(&closes, sma_period);

    let last_idx = candles.len() - 1;
    let last_close = candles[last_idx].close;
    let last_sma = sma[last_idx];

    if last_close > last_sma * 1.02 {
        Trend::Bullish
    } else if last_close < last_sma * 0.98 {
        Trend::Bearish
    } else {
        Trend::Sideways
    }
}
```

## Мульти-таймфрейм стратегия

```rust
#[derive(Debug)]
struct MultiTimeFrameStrategy {
    symbol: String,
    higher_tf: TimeFrame,  // Старший таймфрейм для тренда
    lower_tf: TimeFrame,   // Младший таймфрейм для входа
    trend_period: usize,
    signal_period: usize,
}

impl MultiTimeFrameStrategy {
    fn new(
        symbol: &str,
        higher_tf: TimeFrame,
        lower_tf: TimeFrame,
        trend_period: usize,
        signal_period: usize,
    ) -> Self {
        MultiTimeFrameStrategy {
            symbol: symbol.to_string(),
            higher_tf,
            lower_tf,
            trend_period,
            signal_period,
        }
    }

    fn analyze(&self, data: &MultiTimeFrameData) -> Option<Signal> {
        // Получаем данные обоих таймфреймов
        let higher_candles = data.get_candles(self.higher_tf)?;
        let lower_candles = data.get_candles(self.lower_tf)?;

        // Определяем тренд на старшем таймфрейме
        let higher_trend = detect_trend(higher_candles, self.trend_period);

        // Ищем сигнал на младшем таймфрейме
        let lower_signal = self.find_entry_signal(lower_candles);

        // Комбинируем сигналы
        match (higher_trend, lower_signal) {
            (Trend::Bullish, Some(Signal::Buy)) => Some(Signal::Buy),
            (Trend::Bearish, Some(Signal::Sell)) => Some(Signal::Sell),
            _ => None, // Игнорируем сигналы против тренда
        }
    }

    fn find_entry_signal(&self, candles: &[Candle]) -> Option<Signal> {
        if candles.len() < self.signal_period + 2 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let fast_ema = exponential_moving_average(&closes, self.signal_period);
        let slow_ema = exponential_moving_average(&closes, self.signal_period * 2);

        let len = candles.len();

        // Пересечение EMA вверх = сигнал покупки
        if fast_ema[len - 1] > slow_ema[len - 1]
            && fast_ema[len - 2] <= slow_ema[len - 2]
        {
            return Some(Signal::Buy);
        }

        // Пересечение EMA вниз = сигнал продажи
        if fast_ema[len - 1] < slow_ema[len - 1]
            && fast_ema[len - 2] >= slow_ema[len - 2]
        {
            return Some(Signal::Sell);
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
enum Signal {
    Buy,
    Sell,
}
```

## Бэктестинг с мульти-таймфрейм анализом

```rust
#[derive(Debug)]
struct BacktestResult {
    total_trades: usize,
    winning_trades: usize,
    total_profit: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn backtest_multi_timeframe(
    data: &MultiTimeFrameData,
    strategy: &MultiTimeFrameStrategy,
) -> BacktestResult {
    let mut equity = 10_000.0;
    let mut peak_equity = equity;
    let mut max_drawdown = 0.0;
    let mut trades = Vec::new();
    let mut position: Option<f64> = None;

    let lower_candles = data
        .get_candles(strategy.lower_tf)
        .expect("Нет данных для младшего таймфрейма");

    // Нужно минимум данных для обоих таймфреймов
    let start_idx = strategy.trend_period.max(strategy.signal_period * 2) + 10;

    for i in start_idx..lower_candles.len() {
        // Создаём срез данных до текущего момента
        let current_lower = &lower_candles[..=i];

        // Для старшего таймфрейма берём соответствующий срез
        let higher_candles = data
            .get_candles(strategy.higher_tf)
            .expect("Нет данных для старшего таймфрейма");

        // Формируем временные данные для анализа
        let mut current_data = MultiTimeFrameData::new(&data.symbol);
        current_data.add_candles(strategy.lower_tf, current_lower.to_vec());
        current_data.add_candles(strategy.higher_tf, higher_candles.to_vec());

        // Получаем сигнал
        let signal = strategy.analyze(&current_data);
        let current_price = lower_candles[i].close;

        match (signal, position) {
            (Some(Signal::Buy), None) => {
                // Открываем длинную позицию
                position = Some(current_price);
            }
            (Some(Signal::Sell), Some(entry_price)) => {
                // Закрываем позицию
                let profit_pct = (current_price - entry_price) / entry_price;
                equity *= 1.0 + profit_pct;

                trades.push(profit_pct > 0.0);
                position = None;

                // Обновляем максимальную просадку
                if equity > peak_equity {
                    peak_equity = equity;
                }
                let drawdown = (peak_equity - equity) / peak_equity;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
            _ => {}
        }
    }

    // Закрываем открытую позицию в конце
    if let Some(entry_price) = position {
        let last_price = lower_candles.last().unwrap().close;
        let profit_pct = (last_price - entry_price) / entry_price;
        equity *= 1.0 + profit_pct;
        trades.push(profit_pct > 0.0);
    }

    let winning_trades = trades.iter().filter(|&&win| win).count();
    let total_trades = trades.len();
    let total_profit = (equity - 10_000.0) / 10_000.0 * 100.0;
    let win_rate = if total_trades > 0 {
        winning_trades as f64 / total_trades as f64 * 100.0
    } else {
        0.0
    };

    BacktestResult {
        total_trades,
        winning_trades,
        total_profit,
        max_drawdown: max_drawdown * 100.0,
        win_rate,
    }
}

fn main() {
    println!("=== Мульти-таймфрейм бэктестинг ===\n");

    // Генерация данных для 1-часового таймфрейма
    let h1_candles: Vec<Candle> = (0..1000)
        .map(|i| {
            let base = 50000.0 + (i as f64 * 0.05).sin() * 2000.0;
            let trend = i as f64 * 5.0;
            let noise = (i as f64 * 13.0).sin() * 100.0;
            let price = base + trend + noise;

            Candle {
                timestamp: i * 3600,
                open: price - 50.0,
                high: price + 100.0,
                low: price - 100.0,
                close: price,
                volume: 1000.0 + (i as f64 * 7.0).sin().abs() * 500.0,
            }
        })
        .collect();

    // Создаём мульти-таймфрейм данные
    let mut mtf_data = MultiTimeFrameData::new("BTC/USD");
    mtf_data.add_candles(TimeFrame::H1, h1_candles.clone());

    // Агрегируем в 4-часовой таймфрейм
    if let Some(h4_candles) = mtf_data.aggregate_timeframe(TimeFrame::H1, TimeFrame::H4) {
        mtf_data.add_candles(TimeFrame::H4, h4_candles);
    }

    // Создаём стратегию: тренд на H4, вход на H1
    let strategy = MultiTimeFrameStrategy::new(
        "BTC/USD",
        TimeFrame::H4, // Старший таймфрейм
        TimeFrame::H1, // Младший таймфрейм
        20,            // Период для определения тренда
        10,            // Период для сигналов входа
    );

    // Запускаем бэктест
    let result = backtest_multi_timeframe(&mtf_data, &strategy);

    println!("Результаты бэктеста:");
    println!("  Всего сделок: {}", result.total_trades);
    println!("  Прибыльных сделок: {}", result.winning_trades);
    println!("  Win Rate: {:.2}%", result.win_rate);
    println!("  Общая прибыль: {:.2}%", result.total_profit);
    println!("  Максимальная просадка: {:.2}%", result.max_drawdown);

    if result.win_rate > 50.0 && result.total_profit > 10.0 {
        println!("\n✓ Стратегия показала хорошие результаты!");
    } else {
        println!("\n✗ Стратегия требует доработки");
    }
}
```

## Продвинутые техники

### 1. Тройной таймфрейм анализ

```rust
struct TripleTimeFrameStrategy {
    long_term: TimeFrame,   // Долгосрочный тренд (D1/W1)
    mid_term: TimeFrame,    // Среднесрочный тренд (H4)
    short_term: TimeFrame,  // Краткосрочный вход (H1)
}

impl TripleTimeFrameStrategy {
    fn analyze(&self, data: &MultiTimeFrameData) -> Option<Signal> {
        // Все три таймфрейма должны совпадать для сильного сигнала
        let long_trend = self.get_trend(data, self.long_term)?;
        let mid_trend = self.get_trend(data, self.mid_term)?;
        let short_signal = self.get_signal(data, self.short_term)?;

        match (long_trend, mid_trend, short_signal) {
            (Trend::Bullish, Trend::Bullish, Signal::Buy) => Some(Signal::Buy),
            (Trend::Bearish, Trend::Bearish, Signal::Sell) => Some(Signal::Sell),
            _ => None,
        }
    }

    fn get_trend(&self, data: &MultiTimeFrameData, tf: TimeFrame) -> Option<Trend> {
        let candles = data.get_candles(tf)?;
        Some(detect_trend(candles, 20))
    }

    fn get_signal(&self, data: &MultiTimeFrameData, tf: TimeFrame) -> Option<Signal> {
        let candles = data.get_candles(tf)?;
        if candles.len() < 20 {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let ema = exponential_moving_average(&closes, 10);

        let len = closes.len();
        if closes[len - 1] > ema[len - 1] {
            Some(Signal::Buy)
        } else {
            Some(Signal::Sell)
        }
    }
}
```

### 2. Адаптивные стоп-лоссы по таймфреймам

```rust
fn calculate_adaptive_stop_loss(
    candles: &[Candle],
    timeframe: TimeFrame,
) -> f64 {
    // Стоп-лосс зависит от волатильности таймфрейма
    let atr = calculate_atr(candles, 14);

    match timeframe {
        TimeFrame::H1 => atr * 1.5,
        TimeFrame::H4 => atr * 2.0,
        TimeFrame::D1 => atr * 2.5,
        _ => atr * 2.0,
    }
}

fn calculate_atr(candles: &[Candle], period: usize) -> f64 {
    if candles.len() < period + 1 {
        return 0.0;
    }

    let mut tr_sum = 0.0;

    for i in 1..=period {
        let high_low = candles[i].high - candles[i].low;
        let high_close = (candles[i].high - candles[i - 1].close).abs();
        let low_close = (candles[i].low - candles[i - 1].close).abs();

        let true_range = high_low.max(high_close).max(low_close);
        tr_sum += true_range;
    }

    tr_sum / period as f64
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Мульти-таймфрейм анализ | Анализ рынка на нескольких временных периодах |
| Старший таймфрейм | Определяет общее направление тренда |
| Младший таймфрейм | Даёт точные точки входа и выхода |
| Агрегация данных | Преобразование данных из младшего в старший таймфрейм |
| Фильтрация сигналов | Игнорирование сигналов против старшего тренда |
| Адаптивный риск-менеджмент | Стоп-лоссы зависят от таймфрейма и волатильности |

## Практические задания

1. **Сравнение таймфреймов**: Протестируй одну и ту же стратегию на разных комбинациях таймфреймов (H1-M15, H4-H1, D1-H4). Какая комбинация даёт лучшие результаты?

2. **Реализуй RSI для мульти-таймфрейма**: Добавь индикатор RSI и создай стратегию, где RSI на старшем таймфрейме определяет зоны перекупленности/перепроданности, а младший даёт точку входа.

3. **Волатильность по таймфреймам**: Создай функцию, которая рассчитывает ATR для всех таймфреймов и использует эту информацию для динамического размера позиции.

4. **Визуализация сигналов**: Расширь код, чтобы логировать моменты, когда на старшем таймфрейме один тренд, а на младшем — другой. Сколько таких конфликтов происходит?

## Домашнее задание

1. **Четырёхуровневая стратегия**: Реализуй стратегию с четырьмя таймфреймами (W1, D1, H4, H1), где каждый уровень даёт подтверждение для следующего.

2. **Strength Index**: Создай индекс силы тренда (0-100%), который учитывает согласованность всех таймфреймов. Торгуй только когда индекс > 70%.

3. **Backtesting Comparison**: Сравни результаты:
   - Стратегия только на H1
   - Стратегия с подтверждением H4
   - Стратегия с тройным таймфреймом (D1-H4-H1)

   Построй таблицу метрик для каждого варианта.

4. **Real-time Simulation**: Создай симуляцию реального времени, где данные поступают свеча за свечой на младшем таймфрейме, а старшие таймфреймы обновляются соответственно.

## Навигация

[← Предыдущий день](../293-grid-search-parameter-sweep/ru.md) | [Следующий день →](../299-advanced-testing-techniques/ru.md)
