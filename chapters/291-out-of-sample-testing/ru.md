# День 291: Out-of-sample тестирование

## Аналогия из трейдинга

Представь, что ты разработал торговую стратегию на основе Bitcoin данных с января по июнь 2024. Она показывает отличные результаты: +45% за период! Ты запускаешь бота с реальными деньгами и... теряешь 20% за месяц. Что случилось?

Проблема называется **overfitting** (переобучение). Твоя стратегия идеально подстроилась под исторические данные, но не работает на новых. Это как студент, который выучил ответы на конкретные вопросы, но не понял материал.

**Out-of-sample testing** (тестирование на новых данных) решает эту проблему:
- **In-sample** (обучающая выборка): Январь-Июнь 2024 — разрабатываем и оптимизируем стратегию
- **Out-of-sample** (тестовая выборка): Июль-Сентябрь 2024 — проверяем, работает ли стратегия на данных, которые она "не видела"

Если стратегия хорошо работает на обеих выборках — это признак надёжности. Если только на in-sample — скорее всего, переобучение.

## Базовое разделение данных

Начнём с простого разделения данных на обучающую и тестовую выборки:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct DataSplit<T> {
    in_sample: Vec<T>,
    out_of_sample: Vec<T>,
}

fn split_data<T: Clone>(data: &[T], split_ratio: f64) -> DataSplit<T> {
    let split_point = (data.len() as f64 * split_ratio) as usize;

    DataSplit {
        in_sample: data[..split_point].to_vec(),
        out_of_sample: data[split_point..].to_vec(),
    }
}

fn main() {
    // Исторические данные Bitcoin
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43800.0),
        Candle::new("2024-01-08", 44500.0),
        Candle::new("2024-01-09", 45000.0),
        Candle::new("2024-01-10", 44800.0),
    ];

    // Разделяем 70% на обучение, 30% на тестирование
    let split = split_data(&candles, 0.7);

    println!("=== Разделение данных ===");
    println!("Всего свечей: {}", candles.len());
    println!("In-sample (обучение): {}", split.in_sample.len());
    println!("Out-of-sample (тестирование): {}", split.out_of_sample.len());

    println!("\n=== In-sample данные ===");
    for candle in &split.in_sample {
        println!("{}: ${:.2}", candle.timestamp, candle.close);
    }

    println!("\n=== Out-of-sample данные ===");
    for candle in &split.out_of_sample {
        println!("{}: ${:.2}", candle.timestamp, candle.close);
    }
}
```

## Простая торговая стратегия

Создадим простую стратегию на основе скользящего среднего (SMA):

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn calculate_sma(&self, candles: &[Candle]) -> Vec<f64> {
        let mut sma_values = Vec::new();

        if candles.len() < self.period {
            return sma_values;
        }

        for i in (self.period - 1)..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            sma_values.push(sum / self.period as f64);
        }

        sma_values
    }

    // Сигнал: BUY если цена выше SMA, SELL если ниже
    fn generate_signals(&self, candles: &[Candle]) -> Vec<String> {
        let sma_values = self.calculate_sma(candles);
        let mut signals = Vec::new();

        // Первые period-1 свечей пропускаем
        for _ in 0..(self.period - 1) {
            signals.push("HOLD".to_string());
        }

        for (i, sma) in sma_values.iter().enumerate() {
            let candle_idx = i + self.period - 1;
            if candles[candle_idx].close > *sma {
                signals.push("BUY".to_string());
            } else {
                signals.push("SELL".to_string());
            }
        }

        signals
    }
}

fn main() {
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
    ];

    let strategy = SMAStrategy::new(3);
    let signals = strategy.generate_signals(&candles);

    println!("=== Торговые сигналы (SMA-3) ===");
    for (i, candle) in candles.iter().enumerate() {
        println!("{}: ${:.2} -> {}", candle.timestamp, candle.close, signals[i]);
    }
}
```

## Бэктестинг стратегии

Теперь реализуем систему бэктестинга:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct BacktestResult {
    total_trades: usize,
    profitable_trades: usize,
    total_return: f64,
    max_drawdown: f64,
}

impl BacktestResult {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.profitable_trades as f64 / self.total_trades as f64) * 100.0
    }

    fn print_summary(&self, label: &str) {
        println!("\n=== {} ===", label);
        println!("Всего сделок: {}", self.total_trades);
        println!("Прибыльных: {}", self.profitable_trades);
        println!("Win rate: {:.2}%", self.win_rate());
        println!("Общая доходность: {:.2}%", self.total_return);
        println!("Максимальная просадка: {:.2}%", self.max_drawdown);
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None; // Цена входа
        let mut total_trades = 0;
        let mut profitable_trades = 0;
        let mut total_return = 0.0;
        let mut equity = 100.0; // Начальный капитал 100%
        let mut peak_equity = 100.0;
        let mut max_drawdown = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                profitable_trades: 0,
                total_return: 0.0,
                max_drawdown: 0.0,
            };
        }

        for i in self.period..candles.len() {
            // Рассчитываем SMA
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;

            let price = candles[i].close;

            // Торговая логика
            if position.is_none() && price > sma {
                // Открываем позицию BUY
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    // Закрываем позицию
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    equity += profit;

                    total_trades += 1;
                    if profit > 0.0 {
                        profitable_trades += 1;
                    }

                    // Обновляем максимальную просадку
                    if equity > peak_equity {
                        peak_equity = equity;
                    }
                    let drawdown = ((peak_equity - equity) / peak_equity) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }

                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            profitable_trades,
            total_return,
            max_drawdown,
        }
    }
}

fn main() {
    // Генерируем тестовые данные
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
    ];

    let strategy = SMAStrategy::new(3);
    let result = strategy.backtest(&candles);

    result.print_summary("Результаты бэктеста");
}
```

## In-sample vs Out-of-sample

Полный пример с разделением и сравнением результатов:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

struct DataSplit<T> {
    in_sample: Vec<T>,
    out_of_sample: Vec<T>,
}

fn split_data<T: Clone>(data: &[T], split_ratio: f64) -> DataSplit<T> {
    let split_point = (data.len() as f64 * split_ratio) as usize;

    DataSplit {
        in_sample: data[..split_point].to_vec(),
        out_of_sample: data[split_point..].to_vec(),
    }
}

#[derive(Debug)]
struct BacktestResult {
    total_trades: usize,
    profitable_trades: usize,
    total_return: f64,
    max_drawdown: f64,
}

impl BacktestResult {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.profitable_trades as f64 / self.total_trades as f64) * 100.0
    }

    fn print_summary(&self, label: &str) {
        println!("\n=== {} ===", label);
        println!("Всего сделок: {}", self.total_trades);
        println!("Прибыльных: {}", self.profitable_trades);
        println!("Win rate: {:.2}%", self.win_rate());
        println!("Общая доходность: {:.2}%", self.total_return);
        println!("Максимальная просадка: {:.2}%", self.max_drawdown);
    }
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None;
        let mut total_trades = 0;
        let mut profitable_trades = 0;
        let mut total_return = 0.0;
        let mut equity = 100.0;
        let mut peak_equity = 100.0;
        let mut max_drawdown = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                profitable_trades: 0,
                total_return: 0.0,
                max_drawdown: 0.0,
            };
        }

        for i in self.period..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;
            let price = candles[i].close;

            if position.is_none() && price > sma {
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    equity += profit;

                    total_trades += 1;
                    if profit > 0.0 {
                        profitable_trades += 1;
                    }

                    if equity > peak_equity {
                        peak_equity = equity;
                    }
                    let drawdown = ((peak_equity - equity) / peak_equity) * 100.0;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }

                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            profitable_trades,
            total_return,
            max_drawdown,
        }
    }
}

fn main() {
    // Большой набор данных
    let all_candles = vec![
        // In-sample данные
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
        Candle::new("2024-01-11", 43500.0),
        Candle::new("2024-01-12", 44000.0),
        Candle::new("2024-01-13", 44500.0),
        Candle::new("2024-01-14", 44200.0),
        // Out-of-sample данные
        Candle::new("2024-01-15", 44800.0),
        Candle::new("2024-01-16", 45000.0),
        Candle::new("2024-01-17", 44500.0),
        Candle::new("2024-01-18", 44000.0),
        Candle::new("2024-01-19", 44500.0),
        Candle::new("2024-01-20", 45000.0),
    ];

    // Разделяем данные 70/30
    let split = split_data(&all_candles, 0.7);

    println!("=== Разделение данных ===");
    println!("Всего: {} свечей", all_candles.len());
    println!("In-sample: {} свечей", split.in_sample.len());
    println!("Out-of-sample: {} свечей", split.out_of_sample.len());

    // Тестируем стратегию
    let strategy = SMAStrategy::new(3);

    let in_sample_result = strategy.backtest(&split.in_sample);
    in_sample_result.print_summary("In-sample результаты");

    let out_of_sample_result = strategy.backtest(&split.out_of_sample);
    out_of_sample_result.print_summary("Out-of-sample результаты");

    // Анализ переобучения
    println!("\n=== Анализ переобучения ===");
    let return_diff = (in_sample_result.total_return - out_of_sample_result.total_return).abs();
    let win_rate_diff = (in_sample_result.win_rate() - out_of_sample_result.win_rate()).abs();

    println!("Разница в доходности: {:.2}%", return_diff);
    println!("Разница в win rate: {:.2}%", win_rate_diff);

    if return_diff < 10.0 && win_rate_diff < 15.0 {
        println!("✅ Стратегия стабильна (малое расхождение)");
    } else {
        println!("⚠️  Возможное переобучение (большое расхождение)");
    }
}
```

## Walk-Forward анализ

Более продвинутый метод — walk-forward тестирование:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Candle {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

#[derive(Debug)]
struct BacktestResult {
    total_return: f64,
    total_trades: usize,
}

struct SMAStrategy {
    period: usize,
}

impl SMAStrategy {
    fn new(period: usize) -> Self {
        SMAStrategy { period }
    }

    fn backtest(&self, candles: &[Candle]) -> BacktestResult {
        let mut position: Option<f64> = None;
        let mut total_trades = 0;
        let mut total_return = 0.0;

        if candles.len() < self.period {
            return BacktestResult {
                total_trades: 0,
                total_return: 0.0,
            };
        }

        for i in self.period..candles.len() {
            let sum: f64 = candles[(i + 1 - self.period)..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            let sma = sum / self.period as f64;
            let price = candles[i].close;

            if position.is_none() && price > sma {
                position = Some(price);
            } else if let Some(entry_price) = position {
                if price < sma {
                    let profit = ((price - entry_price) / entry_price) * 100.0;
                    total_return += profit;
                    total_trades += 1;
                    position = None;
                }
            }
        }

        BacktestResult {
            total_trades,
            total_return,
        }
    }
}

fn walk_forward_test(candles: &[Candle], train_size: usize, test_size: usize) {
    let mut current_pos = 0;
    let mut iteration = 1;

    println!("=== Walk-Forward тестирование ===\n");

    while current_pos + train_size + test_size <= candles.len() {
        let train_end = current_pos + train_size;
        let test_end = train_end + test_size;

        let train_data = &candles[current_pos..train_end];
        let test_data = &candles[train_end..test_end];

        println!("--- Итерация {} ---", iteration);
        println!("Обучение: {} - {}",
            train_data.first().unwrap().timestamp,
            train_data.last().unwrap().timestamp);
        println!("Тестирование: {} - {}",
            test_data.first().unwrap().timestamp,
            test_data.last().unwrap().timestamp);

        // Тестируем разные периоды на обучающей выборке
        let mut best_period = 3;
        let mut best_return = f64::MIN;

        for period in 3..=7 {
            let strategy = SMAStrategy::new(period);
            let result = strategy.backtest(train_data);

            if result.total_return > best_return {
                best_return = result.total_return;
                best_period = period;
            }
        }

        println!("Лучший период на обучении: {} (доходность: {:.2}%)",
            best_period, best_return);

        // Тестируем лучший параметр на out-of-sample
        let strategy = SMAStrategy::new(best_period);
        let test_result = strategy.backtest(test_data);

        println!("Результат на тестовых данных: {:.2}%", test_result.total_return);
        println!();

        current_pos += test_size;
        iteration += 1;
    }
}

fn main() {
    let candles = vec![
        Candle::new("2024-01-01", 42000.0),
        Candle::new("2024-01-02", 42500.0),
        Candle::new("2024-01-03", 43000.0),
        Candle::new("2024-01-04", 42800.0),
        Candle::new("2024-01-05", 43500.0),
        Candle::new("2024-01-06", 44000.0),
        Candle::new("2024-01-07", 43500.0),
        Candle::new("2024-01-08", 43000.0),
        Candle::new("2024-01-09", 42500.0),
        Candle::new("2024-01-10", 43000.0),
        Candle::new("2024-01-11", 43500.0),
        Candle::new("2024-01-12", 44000.0),
        Candle::new("2024-01-13", 44500.0),
        Candle::new("2024-01-14", 44200.0),
        Candle::new("2024-01-15", 44800.0),
        Candle::new("2024-01-16", 45000.0),
        Candle::new("2024-01-17", 44500.0),
        Candle::new("2024-01-18", 44000.0),
        Candle::new("2024-01-19", 44500.0),
        Candle::new("2024-01-20", 45000.0),
    ];

    // Walk-forward: 10 свечей обучение, 5 свечей тестирование
    walk_forward_test(&candles, 10, 5);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `In-sample` | Данные для обучения и оптимизации стратегии |
| `Out-of-sample` | Данные для проверки надёжности стратегии |
| `Overfitting` | Переобучение — стратегия работает только на исторических данных |
| `Walk-forward` | Последовательное тестирование с перемещением окна |
| `Data split` | Разделение данных на обучающую и тестовую выборки |

## Практические задания

1. **Разделение данных**: Напиши функцию, которая разделяет данные на три части: train (60%), validation (20%), test (20%).

2. **Сравнение стратегий**: Создай две стратегии (SMA и EMA) и сравни их результаты на in-sample и out-of-sample данных.

3. **Метрики переобучения**: Реализуй функцию, которая вычисляет коэффициент стабильности: `stability_ratio = out_of_sample_return / in_sample_return`. Если коэффициент близок к 1.0 — стратегия стабильна.

## Домашнее задание

1. Создай систему тестирования стратегии:
   - Загрузи исторические данные за год
   - Раздели на in-sample (первые 9 месяцев) и out-of-sample (последние 3 месяца)
   - Протестируй стратегию на обеих выборках
   - Сравни результаты и выяви признаки переобучения

2. Реализуй walk-forward оптимизацию:
   - Используй скользящее окно (например, 30 дней обучение, 10 дней тест)
   - На каждой итерации находи оптимальные параметры стратегии
   - Тестируй найденные параметры на следующем периоде
   - Построй график совокупной доходности

3. Создай систему Monte Carlo валидации:
   - Возьми исторические данные
   - Создай 1000 случайных перестановок данных
   - Протестируй стратегию на каждой перестановке
   - Вычисли среднюю доходность и стандартное отклонение
   - Определи, является ли стратегия устойчивой к порядку данных

4. Реализуй cross-validation для временных рядов:
   - Раздели данные на 5 последовательных блоков
   - Используй блоки 1-3 для обучения, блок 4 для валидации, блок 5 для теста
   - Повтори процесс со сдвигом
   - Вычисли среднюю точность предсказаний

## Навигация

[← Предыдущий день](../290-walk-forward-analysis/ru.md) | [Следующий день →](../292-parameter-optimization/ru.md)
