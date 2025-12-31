# День 246: Moving Average: скользящая средняя

## Аналогия из трейдинга

Представь, что ты наблюдаешь за ценой Bitcoin. Каждую секунду цена меняется: 42000, 42050, 41980, 42100... Как понять общий тренд среди всего этого шума? Здесь на помощь приходит **скользящая средняя** (Moving Average, MA).

Скользящая средняя — это как взгляд на рынок "размытыми глазами": вместо резких скачков ты видишь плавную линию, показывающую направление движения цены. Это один из самых важных индикаторов в техническом анализе.

В реальном трейдинге скользящие средние используют для:
- Определения тренда (цена выше MA = восходящий тренд)
- Поиска уровней поддержки и сопротивления
- Генерации торговых сигналов (пересечение быстрой и медленной MA)
- Фильтрации рыночного шума

## Что такое скользящая средняя?

Скользящая средняя — это среднее значение цены за определённый период, которое пересчитывается с каждым новым значением. "Скользящая" потому, что окно расчёта как бы "скользит" по данным.

Например, SMA-5 (простая скользящая средняя за 5 периодов):
```
Цены: [100, 102, 101, 103, 105, 104, 106]
SMA-5 для позиции 4: (100 + 102 + 101 + 103 + 105) / 5 = 102.2
SMA-5 для позиции 5: (102 + 101 + 103 + 105 + 104) / 5 = 103.0
SMA-5 для позиции 6: (101 + 103 + 105 + 104 + 106) / 5 = 103.8
```

## Простая скользящая средняя (SMA)

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;
    let sma_values = calculate_sma(&prices, period);

    println!("=== SMA-{} для BTC ===", period);
    for (i, sma) in sma_values.iter().enumerate() {
        let price_index = i + period - 1;
        println!(
            "Период {}: Цена = ${:.2}, SMA = ${:.2}",
            price_index + 1,
            prices[price_index],
            sma
        );
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut sma_values = Vec::new();

    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        let sma = sum / period as f64;
        sma_values.push(sma);
    }

    sma_values
}
```

## Экспоненциальная скользящая средняя (EMA)

EMA придаёт больший вес последним ценам, поэтому быстрее реагирует на изменения:

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;
    let ema_values = calculate_ema(&prices, period);

    println!("=== EMA-{} для BTC ===", period);
    for (i, ema) in ema_values.iter().enumerate() {
        println!("Период {}: EMA = ${:.2}", i + period, ema);
    }
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut ema_values = Vec::new();

    // Первое значение EMA = SMA
    let initial_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    ema_values.push(initial_sma);

    // Множитель сглаживания: 2 / (period + 1)
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Рассчитываем EMA для остальных значений
    for i in period..prices.len() {
        let current_price = prices[i];
        let previous_ema = ema_values[ema_values.len() - 1];
        let current_ema = (current_price - previous_ema) * multiplier + previous_ema;
        ema_values.push(current_ema);
    }

    ema_values
}
```

## Сравнение SMA и EMA

```rust
fn main() {
    // Резкий скачок цены для демонстрации разницы
    let prices = vec![
        100.0, 100.0, 100.0, 100.0, 100.0,  // Стабильная цена
        110.0, 115.0, 120.0,                 // Резкий рост
    ];

    let period = 5;
    let sma_values = calculate_sma(&prices, period);
    let ema_values = calculate_ema(&prices, period);

    println!("=== Сравнение SMA и EMA (период = {}) ===", period);
    println!("{:<10} {:<10} {:<10} {:<10}", "Индекс", "Цена", "SMA", "EMA");
    println!("{}", "-".repeat(40));

    for i in 0..sma_values.len() {
        let price_idx = i + period - 1;
        println!(
            "{:<10} {:<10.2} {:<10.2} {:<10.2}",
            price_idx + 1,
            prices[price_idx],
            sma_values[i],
            ema_values[i]
        );
    }

    println!("\nВывод: EMA быстрее реагирует на изменения цены!");
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    let initial_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);
    let multiplier = 2.0 / (period as f64 + 1.0);
    for i in period..prices.len() {
        let prev = result[result.len() - 1];
        result.push((prices[i] - prev) * multiplier + prev);
    }
    result
}
```

## Структура для торгового анализа

```rust
#[derive(Debug, Clone)]
struct PriceData {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct MovingAverageAnalyzer {
    prices: Vec<PriceData>,
    sma_period: usize,
    ema_period: usize,
}

impl MovingAverageAnalyzer {
    fn new(sma_period: usize, ema_period: usize) -> Self {
        MovingAverageAnalyzer {
            prices: Vec::new(),
            sma_period,
            ema_period,
        }
    }

    fn add_price(&mut self, data: PriceData) {
        self.prices.push(data);
    }

    fn get_closes(&self) -> Vec<f64> {
        self.prices.iter().map(|p| p.close).collect()
    }

    fn calculate_sma(&self) -> Vec<f64> {
        let closes = self.get_closes();
        if closes.len() < self.sma_period {
            return vec![];
        }

        let mut result = Vec::new();
        for i in 0..=(closes.len() - self.sma_period) {
            let sum: f64 = closes[i..i + self.sma_period].iter().sum();
            result.push(sum / self.sma_period as f64);
        }
        result
    }

    fn calculate_ema(&self) -> Vec<f64> {
        let closes = self.get_closes();
        if closes.len() < self.ema_period {
            return vec![];
        }

        let mut result = Vec::new();
        let initial: f64 = closes[0..self.ema_period].iter().sum::<f64>()
            / self.ema_period as f64;
        result.push(initial);

        let mult = 2.0 / (self.ema_period as f64 + 1.0);
        for i in self.ema_period..closes.len() {
            let prev = result[result.len() - 1];
            result.push((closes[i] - prev) * mult + prev);
        }
        result
    }

    fn get_trend(&self) -> Option<&'static str> {
        let closes = self.get_closes();
        let sma = self.calculate_sma();

        if closes.is_empty() || sma.is_empty() {
            return None;
        }

        let last_price = closes[closes.len() - 1];
        let last_sma = sma[sma.len() - 1];

        if last_price > last_sma * 1.01 {
            Some("Восходящий тренд (Bullish)")
        } else if last_price < last_sma * 0.99 {
            Some("Нисходящий тренд (Bearish)")
        } else {
            Some("Боковой тренд (Sideways)")
        }
    }
}

fn main() {
    let mut analyzer = MovingAverageAnalyzer::new(5, 5);

    // Симулируем поступление данных
    let test_data = vec![
        PriceData { timestamp: 1, open: 42000.0, high: 42100.0, low: 41900.0, close: 42050.0, volume: 100.0 },
        PriceData { timestamp: 2, open: 42050.0, high: 42200.0, low: 42000.0, close: 42150.0, volume: 120.0 },
        PriceData { timestamp: 3, open: 42150.0, high: 42300.0, low: 42100.0, close: 42250.0, volume: 110.0 },
        PriceData { timestamp: 4, open: 42250.0, high: 42400.0, low: 42200.0, close: 42350.0, volume: 130.0 },
        PriceData { timestamp: 5, open: 42350.0, high: 42500.0, low: 42300.0, close: 42450.0, volume: 140.0 },
        PriceData { timestamp: 6, open: 42450.0, high: 42600.0, low: 42400.0, close: 42550.0, volume: 150.0 },
        PriceData { timestamp: 7, open: 42550.0, high: 42700.0, low: 42500.0, close: 42650.0, volume: 160.0 },
    ];

    for data in test_data {
        analyzer.add_price(data);
    }

    println!("=== Анализ скользящих средних ===");
    println!("SMA-5: {:?}", analyzer.calculate_sma());
    println!("EMA-5: {:?}", analyzer.calculate_ema());
    println!("Тренд: {:?}", analyzer.get_trend());
}
```

## Пересечение скользящих средних (Golden Cross / Death Cross)

```rust
#[derive(Debug, PartialEq)]
enum CrossoverSignal {
    GoldenCross,  // Быстрая MA пересекает медленную снизу вверх (сигнал покупки)
    DeathCross,   // Быстрая MA пересекает медленную сверху вниз (сигнал продажи)
    NoSignal,
}

fn detect_crossover(fast_ma: &[f64], slow_ma: &[f64]) -> CrossoverSignal {
    if fast_ma.len() < 2 || slow_ma.len() < 2 {
        return CrossoverSignal::NoSignal;
    }

    let len = fast_ma.len().min(slow_ma.len());
    let fast_prev = fast_ma[len - 2];
    let fast_curr = fast_ma[len - 1];
    let slow_prev = slow_ma[len - 2];
    let slow_curr = slow_ma[len - 1];

    // Golden Cross: быстрая была ниже, стала выше
    if fast_prev <= slow_prev && fast_curr > slow_curr {
        return CrossoverSignal::GoldenCross;
    }

    // Death Cross: быстрая была выше, стала ниже
    if fast_prev >= slow_prev && fast_curr < slow_curr {
        return CrossoverSignal::DeathCross;
    }

    CrossoverSignal::NoSignal
}

fn main() {
    // Симуляция данных с пересечением
    let prices_bullish = vec![
        100.0, 101.0, 99.0, 100.0, 102.0,   // Начало
        104.0, 106.0, 108.0, 110.0, 112.0,  // Рост (Golden Cross)
    ];

    let prices_bearish = vec![
        110.0, 112.0, 111.0, 109.0, 108.0,  // Начало
        106.0, 104.0, 102.0, 100.0, 98.0,   // Падение (Death Cross)
    ];

    let fast_period = 3;
    let slow_period = 5;

    // Анализ бычьего сценария
    let fast_ma_bull = calculate_sma(&prices_bullish, fast_period);
    let slow_ma_bull = calculate_sma(&prices_bullish, slow_period);
    let signal_bull = detect_crossover(&fast_ma_bull, &slow_ma_bull);

    println!("=== Бычий сценарий ===");
    println!("Быстрая MA (SMA-{}): {:?}", fast_period, fast_ma_bull);
    println!("Медленная MA (SMA-{}): {:?}", slow_period, slow_ma_bull);
    println!("Сигнал: {:?}", signal_bull);

    println!();

    // Анализ медвежьего сценария
    let fast_ma_bear = calculate_sma(&prices_bearish, fast_period);
    let slow_ma_bear = calculate_sma(&prices_bearish, slow_period);
    let signal_bear = detect_crossover(&fast_ma_bear, &slow_ma_bear);

    println!("=== Медвежий сценарий ===");
    println!("Быстрая MA (SMA-{}): {:?}", fast_period, fast_ma_bear);
    println!("Медленная MA (SMA-{}): {:?}", slow_period, slow_ma_bear);
    println!("Сигнал: {:?}", signal_bear);
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}
```

## Взвешенная скользящая средняя (WMA)

```rust
fn calculate_wma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::new();

    // Сумма весов: 1 + 2 + 3 + ... + period = period * (period + 1) / 2
    let weight_sum: f64 = (period * (period + 1) / 2) as f64;

    for i in 0..=(prices.len() - period) {
        let mut weighted_sum = 0.0;
        for j in 0..period {
            let weight = (j + 1) as f64;
            weighted_sum += prices[i + j] * weight;
        }
        result.push(weighted_sum / weight_sum);
    }

    result
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let period = 5;

    let sma = calculate_sma(&prices, period);
    let wma = calculate_wma(&prices, period);
    let ema = calculate_ema(&prices, period);

    println!("=== Сравнение типов скользящих средних (период = {}) ===", period);
    println!("{:<10} {:<12} {:<12} {:<12}", "Индекс", "SMA", "WMA", "EMA");
    println!("{}", "-".repeat(46));

    for i in 0..sma.len() {
        println!(
            "{:<10} {:<12.2} {:<12.2} {:<12.2}",
            i + period,
            sma[i],
            wma[i],
            ema[i]
        );
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    let initial: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(initial);
    let mult = 2.0 / (period as f64 + 1.0);
    for i in period..prices.len() {
        let prev = result[result.len() - 1];
        result.push((prices[i] - prev) * mult + prev);
    }
    result
}
```

## Практический пример: торговая стратегия с MA

```rust
#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Position {
    Long,
    Short,
    None,
}

struct MACrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
    current_position: Position,
    trades: Vec<Trade>,
}

impl MACrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        MACrossoverStrategy {
            fast_period,
            slow_period,
            current_position: Position::None,
            trades: Vec::new(),
        }
    }

    fn backtest(&mut self, prices: &[f64]) {
        let fast_ma = calculate_sma(prices, self.fast_period);
        let slow_ma = calculate_sma(prices, self.slow_period);

        if fast_ma.len() < 2 || slow_ma.len() < 2 {
            println!("Недостаточно данных для бэктеста");
            return;
        }

        // Синхронизируем индексы
        let offset = self.slow_period - 1;

        for i in 1..slow_ma.len() {
            let fast_idx = i + (self.slow_period - self.fast_period);
            if fast_idx >= fast_ma.len() {
                break;
            }

            let fast_prev = fast_ma[fast_idx - 1];
            let fast_curr = fast_ma[fast_idx];
            let slow_prev = slow_ma[i - 1];
            let slow_curr = slow_ma[i];

            let price = prices[offset + i];

            // Golden Cross - открываем Long
            if fast_prev <= slow_prev && fast_curr > slow_curr {
                if self.current_position == Position::Short {
                    self.close_position(price);
                }
                if self.current_position == Position::None {
                    self.open_position(price, Position::Long);
                }
            }

            // Death Cross - открываем Short
            if fast_prev >= slow_prev && fast_curr < slow_curr {
                if self.current_position == Position::Long {
                    self.close_position(price);
                }
                if self.current_position == Position::None {
                    self.open_position(price, Position::Short);
                }
            }
        }

        // Закрываем открытую позицию по последней цене
        if self.current_position != Position::None {
            let last_price = prices[prices.len() - 1];
            self.close_position(last_price);
        }
    }

    fn open_position(&mut self, price: f64, position: Position) {
        self.current_position = position;
        self.trades.push(Trade {
            entry_price: price,
            exit_price: None,
            position,
        });
        println!("Открыта позиция {:?} по цене {:.2}", position, price);
    }

    fn close_position(&mut self, price: f64) {
        if let Some(trade) = self.trades.last_mut() {
            trade.exit_price = Some(price);
            let pnl = match trade.position {
                Position::Long => price - trade.entry_price,
                Position::Short => trade.entry_price - price,
                Position::None => 0.0,
            };
            println!(
                "Закрыта позиция {:?} по цене {:.2}, P&L: {:.2}",
                trade.position, price, pnl
            );
        }
        self.current_position = Position::None;
    }

    fn calculate_total_pnl(&self) -> f64 {
        self.trades
            .iter()
            .filter_map(|t| {
                t.exit_price.map(|exit| match t.position {
                    Position::Long => exit - t.entry_price,
                    Position::Short => t.entry_price - exit,
                    Position::None => 0.0,
                })
            })
            .sum()
    }
}

fn main() {
    // Симуляция ценовых данных с трендами
    let prices = vec![
        100.0, 101.0, 102.0, 101.5, 103.0,  // Начало
        105.0, 107.0, 109.0, 111.0, 113.0,  // Восходящий тренд
        112.0, 110.0, 108.0, 106.0, 104.0,  // Нисходящий тренд
        105.0, 107.0, 108.0, 110.0, 112.0,  // Новый восходящий тренд
    ];

    let mut strategy = MACrossoverStrategy::new(3, 5);

    println!("=== Бэктест стратегии MA Crossover ===");
    println!("Быстрая MA: SMA-3, Медленная MA: SMA-5\n");

    strategy.backtest(&prices);

    println!("\n=== Итоги ===");
    println!("Всего сделок: {}", strategy.trades.len());
    println!("Общий P&L: {:.2}", strategy.calculate_total_pnl());
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }
    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SMA | Простая скользящая средняя — среднее арифметическое за период |
| EMA | Экспоненциальная скользящая средняя — больший вес последним ценам |
| WMA | Взвешенная скользящая средняя — линейно возрастающие веса |
| Golden Cross | Быстрая MA пересекает медленную снизу вверх — сигнал покупки |
| Death Cross | Быстрая MA пересекает медленную сверху вниз — сигнал продажи |
| Период MA | Количество значений для расчёта — влияет на чувствительность |

## Домашнее задание

1. **Оптимизация SMA**: Реализуй эффективный расчёт SMA, который не пересчитывает сумму заново на каждом шаге, а использует скользящее окно (добавляет новое значение и вычитает старое).

2. **Множественные MA**: Создай структуру `MultiMAAnalyzer`, которая одновременно рассчитывает SMA-10, SMA-20, SMA-50, SMA-200 и определяет общий тренд по их взаимному расположению.

3. **Адаптивная MA**: Реализуй KAMA (Kaufman's Adaptive Moving Average), которая автоматически подстраивает свою чувствительность под волатильность рынка.

4. **Бэктест с комиссиями**: Модифицируй торговую стратегию из примера, добавив:
   - Комиссию за сделку (0.1%)
   - Stop-loss (2% от цены входа)
   - Take-profit (5% от цены входа)
   - Подсчёт win rate (процент прибыльных сделок)

## Навигация

[← Предыдущий день](../245-calculating-candles-from-ticks/ru.md) | [Следующий день →](../247-sma-simple-moving-average/ru.md)
