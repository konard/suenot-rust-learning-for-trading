# День 295: Cross-validation для стратегий

## Аналогия из трейдинга

Представь, что ты оптимизировал торговую стратегию на данных 2023 года, и она показывает фантастические результаты: +60% доходности! Но когда запускаешь её на данных 2024 года — стратегия теряет деньги. Что пошло не так?

Проблема в том, что ты проверил стратегию только на одном периоде. Возможно, она просто "угадала" особенности рынка 2023 года, но не работает в других условиях.

**Cross-validation (кросс-валидация)** решает эту проблему: вместо одного разбиения данных на train/test, мы делаем несколько независимых проверок на разных периодах времени. Если стратегия стабильно работает на всех периодах — значит, она действительно надёжна, а не просто "подогнана" под конкретные данные.

Это как оценка работника не по одному проекту, а по десяти разным задачам: если он везде показывает хороший результат — он действительно компетентен.

## Что такое cross-validation?

Cross-validation — это метод валидации, который:

| Аспект | Описание |
|--------|----------|
| **Подход** | Многократное разбиение данных на обучение/тест |
| **Цель** | Оценить стабильность модели/стратегии |
| **Применение** | Машинное обучение, валидация торговых стратегий |
| **Преимущество** | Более надёжная оценка качества |
| **Для временных рядов** | Важно сохранять хронологический порядок |
| **Метрика** | Среднее качество по всем фолдам (складкам) |

## Простой пример: K-Fold Cross-Validation для временных рядов

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    close: f64,
}

impl Candle {
    fn new(timestamp: &str, close: f64) -> Self {
        Self {
            timestamp: timestamp.to_string(),
            close,
        }
    }
}

#[derive(Debug)]
struct BacktestResult {
    fold_number: usize,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
}

struct TimeSeriesFold<'a, T> {
    train: &'a [T],
    test: &'a [T],
}

// K-Fold Cross-Validation для временных рядов
fn time_series_k_fold<T>(data: &[T], k: usize) -> Vec<TimeSeriesFold<T>> {
    let mut folds = Vec::new();
    let fold_size = data.len() / (k + 1); // +1 потому что нужен запас для обучения

    for i in 0..k {
        let test_start = (i + 1) * fold_size;
        let test_end = test_start + fold_size;

        if test_end > data.len() {
            break;
        }

        folds.push(TimeSeriesFold {
            train: &data[0..test_start],
            test: &data[test_start..test_end],
        });
    }

    folds
}

fn main() {
    // Исторические данные за несколько периодов
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
    ];

    let k = 3; // Количество фолдов
    let folds = time_series_k_fold(&candles, k);

    println!("=== Time Series K-Fold Cross-Validation (K={}) ===\n", k);

    for (i, fold) in folds.iter().enumerate() {
        println!("Фолд {}:", i + 1);
        println!("  Train: {} свечей (до {})",
            fold.train.len(),
            fold.train.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!("  Test: {} свечей ({} - {})",
            fold.test.len(),
            fold.test.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.test.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!();
    }
}
```

## Бэктест с cross-validation

Теперь применим cross-validation для оценки торговой стратегии:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
struct StrategyConfig {
    ma_period: usize,
    stop_loss_pct: f64,
    take_profit_pct: f64,
}

// Симуляция бэктеста
fn run_backtest(data: &[Candle], config: &StrategyConfig) -> (f64, f64, f64, f64) {
    // Используем хеш для псевдослучайных, но детерминированных результатов
    let mut hasher = DefaultHasher::new();
    config.ma_period.hash(&mut hasher);
    data.len().hash(&mut hasher);
    let seed = (hasher.finish() % 10000) as f64 / 10000.0;

    let total_return = 5.0 + seed * 30.0;
    let sharpe_ratio = 0.8 + seed * 1.5;
    let max_drawdown = 5.0 + seed * 10.0;
    let win_rate = 50.0 + seed * 25.0;

    (total_return, sharpe_ratio, max_drawdown, win_rate)
}

fn cross_validate_strategy(
    data: &[Candle],
    config: &StrategyConfig,
    k: usize,
) -> Vec<BacktestResult> {
    let folds = time_series_k_fold(data, k);
    let mut results = Vec::new();

    for (i, fold) in folds.iter().enumerate() {
        let (total_return, sharpe_ratio, max_drawdown, win_rate) =
            run_backtest(fold.test, config);

        results.push(BacktestResult {
            fold_number: i + 1,
            total_return,
            sharpe_ratio,
            max_drawdown,
            win_rate,
        });
    }

    results
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    let config = StrategyConfig {
        ma_period: 10,
        stop_loss_pct: 2.0,
        take_profit_pct: 5.0,
    };

    let k = 3;
    let results = cross_validate_strategy(&candles, &config, k);

    println!("=== Cross-Validation результаты ===\n");

    let mut total_return_sum = 0.0;
    let mut sharpe_sum = 0.0;
    let mut drawdown_sum = 0.0;
    let mut win_rate_sum = 0.0;

    for result in &results {
        println!("Фолд {}:", result.fold_number);
        println!("  Доходность: {:.2}%", result.total_return);
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
        println!("  Max Drawdown: {:.2}%", result.max_drawdown);
        println!("  Win Rate: {:.2}%", result.win_rate);
        println!();

        total_return_sum += result.total_return;
        sharpe_sum += result.sharpe_ratio;
        drawdown_sum += result.max_drawdown;
        win_rate_sum += result.win_rate;
    }

    let n = results.len() as f64;

    println!("=== Средние метрики по всем фолдам ===");
    println!("Средняя доходность: {:.2}%", total_return_sum / n);
    println!("Средний Sharpe Ratio: {:.2}", sharpe_sum / n);
    println!("Средняя Max Drawdown: {:.2}%", drawdown_sum / n);
    println!("Средний Win Rate: {:.2}%", win_rate_sum / n);
}
```

## Walk-Forward Cross-Validation

Более продвинутый метод для временных рядов — walk-forward validation:

```rust
struct WalkForwardFold<'a, T> {
    train: &'a [T],
    test: &'a [T],
}

fn walk_forward_split<T>(
    data: &[T],
    initial_train_size: usize,
    test_size: usize,
    step_size: usize,
) -> Vec<WalkForwardFold<T>> {
    let mut folds = Vec::new();
    let mut train_end = initial_train_size;

    while train_end + test_size <= data.len() {
        let test_end = train_end + test_size;

        folds.push(WalkForwardFold {
            train: &data[0..train_end],
            test: &data[train_end..test_end],
        });

        train_end += step_size;
    }

    folds
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    // Начальный размер обучения: 6 месяцев
    // Размер теста: 2 месяца
    // Шаг: 1 месяц
    let folds = walk_forward_split(&candles, 6, 2, 1);

    println!("=== Walk-Forward Cross-Validation ===\n");

    for (i, fold) in folds.iter().enumerate() {
        println!("Период {}:", i + 1);
        println!("  Train: {} месяцев ({} - {})",
            fold.train.len(),
            fold.train.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.train.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!("  Test: {} месяца ({} - {})",
            fold.test.len(),
            fold.test.first().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")),
            fold.test.last().map(|c| &c.timestamp).unwrap_or(&String::from("N/A")));
        println!();
    }
}
```

## Продвинутый пример: Cross-Validation с оптимизацией параметров

```rust
#[derive(Debug, Clone)]
struct CrossValidationResult {
    config: StrategyConfig,
    fold_results: Vec<BacktestResult>,
    avg_sharpe: f64,
    std_sharpe: f64,
    avg_return: f64,
    stability_score: f64,
}

fn calculate_std_dev(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

fn evaluate_config_with_cv(
    data: &[Candle],
    config: &StrategyConfig,
    k: usize,
) -> CrossValidationResult {
    let fold_results = cross_validate_strategy(data, config, k);

    let sharpe_values: Vec<f64> = fold_results.iter()
        .map(|r| r.sharpe_ratio)
        .collect();

    let return_values: Vec<f64> = fold_results.iter()
        .map(|r| r.total_return)
        .collect();

    let avg_sharpe = sharpe_values.iter().sum::<f64>() / sharpe_values.len() as f64;
    let avg_return = return_values.iter().sum::<f64>() / return_values.len() as f64;
    let std_sharpe = calculate_std_dev(&sharpe_values, avg_sharpe);

    // Стабильность: высокий средний Sharpe и низкая вариация
    let stability_score = avg_sharpe / (1.0 + std_sharpe);

    CrossValidationResult {
        config: config.clone(),
        fold_results,
        avg_sharpe,
        std_sharpe,
        avg_return,
        stability_score,
    }
}

fn grid_search_with_cv(data: &[Candle], k: usize) -> Vec<CrossValidationResult> {
    let mut results = Vec::new();

    let ma_periods = vec![5, 10, 20];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 8.0];

    for &ma_period in &ma_periods {
        for &stop_loss in &stop_losses {
            for &take_profit in &take_profits {
                if take_profit <= stop_loss {
                    continue;
                }

                let config = StrategyConfig {
                    ma_period,
                    stop_loss_pct: stop_loss,
                    take_profit_pct: take_profit,
                };

                let cv_result = evaluate_config_with_cv(data, &config, k);
                results.push(cv_result);
            }
        }
    }

    results
}

fn main() {
    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    println!("Запуск Grid Search с Cross-Validation...\n");

    let k = 3;
    let cv_results = grid_search_with_cv(&candles, k);

    // Сортируем по stability score
    let mut sorted_results = cv_results;
    sorted_results.sort_by(|a, b|
        b.stability_score.partial_cmp(&a.stability_score).unwrap()
    );

    println!("=== Топ-5 конфигураций по стабильности ===\n");

    for (i, result) in sorted_results.iter().take(5).enumerate() {
        println!("{}. MA={}, SL={:.1}%, TP={:.1}%",
            i + 1,
            result.config.ma_period,
            result.config.stop_loss_pct,
            result.config.take_profit_pct);
        println!("   Средний Sharpe: {:.2} (±{:.2})",
            result.avg_sharpe, result.std_sharpe);
        println!("   Средняя доходность: {:.2}%", result.avg_return);
        println!("   Стабильность: {:.2}", result.stability_score);
        println!();
    }
}
```

## Параллельный Cross-Validation с Rayon

```rust
use rayon::prelude::*;

fn parallel_grid_search_with_cv(data: &[Candle], k: usize) -> Vec<CrossValidationResult> {
    let ma_periods = vec![5, 10, 20];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 8.0];

    // Генерируем все конфигурации
    let mut configs = Vec::new();
    for &ma_period in &ma_periods {
        for &stop_loss in &stop_losses {
            for &take_profit in &take_profits {
                if take_profit > stop_loss {
                    configs.push(StrategyConfig {
                        ma_period,
                        stop_loss_pct: stop_loss,
                        take_profit_pct: take_profit,
                    });
                }
            }
        }
    }

    println!("Параллельное тестирование {} конфигураций...\n", configs.len());

    // Параллельная оценка с Rayon
    configs.par_iter()
        .map(|config| evaluate_config_with_cv(data, config, k))
        .collect()
}

fn main() {
    use std::time::Instant;

    let candles = vec![
        Candle::new("2024-01", 42000.0),
        Candle::new("2024-02", 43000.0),
        Candle::new("2024-03", 44000.0),
        Candle::new("2024-04", 43500.0),
        Candle::new("2024-05", 45000.0),
        Candle::new("2024-06", 46000.0),
        Candle::new("2024-07", 45500.0),
        Candle::new("2024-08", 47000.0),
        Candle::new("2024-09", 48000.0),
        Candle::new("2024-10", 47500.0),
        Candle::new("2024-11", 49000.0),
        Candle::new("2024-12", 50000.0),
    ];

    let start = Instant::now();
    let k = 3;
    let results = parallel_grid_search_with_cv(&candles, k);
    let duration = start.elapsed();

    println!("Время выполнения: {:?}", duration);
    println!("Протестировано {} конфигураций", results.len());

    // Находим лучшую по стабильности
    if let Some(best) = results.iter()
        .max_by(|a, b| a.stability_score.partial_cmp(&b.stability_score).unwrap())
    {
        println!("\nЛучшая конфигурация:");
        println!("  MA period: {}", best.config.ma_period);
        println!("  Stop-loss: {:.1}%", best.config.stop_loss_pct);
        println!("  Take-profit: {:.1}%", best.config.take_profit_pct);
        println!("  Stability Score: {:.2}", best.stability_score);
        println!("  Average Sharpe: {:.2} (±{:.2})",
            best.avg_sharpe, best.std_sharpe);
    }
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Cross-Validation | Многократная проверка на разных периодах |
| K-Fold CV | Разбиение на K фолдов для валидации |
| Walk-Forward CV | Последовательное расширение обучающей выборки |
| Stability Score | Метрика стабильности результатов |
| Grid Search + CV | Комбинация перебора параметров и валидации |
| Параллелизация | Ускорение с помощью Rayon |

## Практические задания

1. **Базовый K-Fold**: Реализуй 5-fold cross-validation для простой стратегии на основе RSI. Выведи результаты для каждого фолда.

2. **Walk-Forward**: Создай walk-forward validation с расширяющимся окном обучения (anchored walk-forward), где тренировочные данные всегда начинаются с начала истории.

3. **Stability Analysis**: Добавь функцию, которая вычисляет не только среднее и стандартное отклонение Sharpe Ratio, но и коэффициент вариации (CV = std/mean).

4. **Визуализация**: Сохрани результаты каждого фолда в CSV файл с полями: fold_number, config, sharpe_ratio, total_return, max_drawdown.

## Домашнее задание

1. **Time Series Split**: Реализуй альтернативный метод разбиения TimeSeriesSplit, который:
   - Создаёт фолды с фиксированным размером обучающей выборки (sliding window)
   - Поддерживает gap между train и test для предотвращения утечки данных

2. **Nested Cross-Validation**: Реализуй вложенную кросс-валидацию:
   - Внешний цикл: оценка качества модели
   - Внутренний цикл: подбор гиперпараметров
   - Это позволяет избежать overfitting при выборе параметров

3. **Purged K-Fold**: Для стратегий с удержанием позиций реализуй purged K-fold:
   - Исключи из обучающей выборки данные, которые "перекрываются" с тестовым периодом
   - Это важно для стратегий, где позиции держатся несколько дней

4. **Monte Carlo CV**: Вместо фиксированных фолдов используй случайное разбиение:
   - Выполни N итераций (например, 100)
   - Каждый раз случайно выбирай непрерывный период для теста
   - Усредни результаты

## Навигация

[← Предыдущий день](../293-grid-search-parameter-sweep/ru.md) | [Следующий день →](../296-monte-carlo-simulations/ru.md)
