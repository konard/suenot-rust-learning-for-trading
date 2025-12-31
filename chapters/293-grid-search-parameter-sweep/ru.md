# День 293: Grid search: перебор параметров

## Аналогия из трейдинга

Представь, что ты разработал торговую стратегию на основе скользящих средних (Moving Average). Теперь нужно найти оптимальные параметры:
- Короткое окно MA: 5, 10, 15 или 20 периодов?
- Длинное окно MA: 50, 100, 150 или 200 периодов?
- Stop-loss: 1%, 2%, 3% или 5%?
- Take-profit: 2%, 4%, 6% или 10%?

Вместо того чтобы тестировать каждую комбинацию вручную, тебе нужен **grid search (перебор по сетке)** — систематический метод перебора всех возможных комбинаций параметров для поиска лучшей конфигурации.

Это как тестирование всех настроек радиоприёмника для поиска самого чистого сигнала: ты методично проверяешь каждую комбинацию частоты и громкости, чтобы найти идеальные параметры.

## Что такое grid search?

Grid search — это метод оптимизации параметров, который:

| Аспект | Описание |
|--------|----------|
| **Подход** | Исчерпывающий перебор всех комбинаций |
| **Параметры** | Дискретные значения из заранее определённого набора |
| **Применение** | Машинное обучение, бэктестинг стратегий |
| **Сложность** | O(n^m), где n — значений на параметр, m — количество параметров |
| **Преимущество** | Гарантия нахождения лучшей комбинации в заданном пространстве |
| **Недостаток** | Может быть очень медленным при большом количестве параметров |

## Простой пример: перебор параметров

```rust
// Определяем пространство поиска
struct GridSearchSpace {
    short_ma_periods: Vec<usize>,
    long_ma_periods: Vec<usize>,
    stop_loss_pct: Vec<f64>,
    take_profit_pct: Vec<f64>,
}

// Конфигурация стратегии
#[derive(Debug, Clone)]
struct StrategyConfig {
    short_ma: usize,
    long_ma: usize,
    stop_loss: f64,
    take_profit: f64,
}

// Результат бэктеста
#[derive(Debug)]
struct BacktestResult {
    config: StrategyConfig,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn main() {
    // Определяем пространство параметров
    let search_space = GridSearchSpace {
        short_ma_periods: vec![5, 10, 15, 20],
        long_ma_periods: vec![50, 100, 150, 200],
        stop_loss_pct: vec![1.0, 2.0, 3.0, 5.0],
        take_profit_pct: vec![2.0, 4.0, 6.0, 10.0],
    };

    let mut all_results = Vec::new();
    let mut total_combinations = 0;

    // Grid search: перебор всех комбинаций
    for &short_ma in &search_space.short_ma_periods {
        for &long_ma in &search_space.long_ma_periods {
            // Пропускаем некорректные комбинации
            if short_ma >= long_ma {
                continue;
            }

            for &stop_loss in &search_space.stop_loss_pct {
                for &take_profit in &search_space.take_profit_pct {
                    total_combinations += 1;

                    let config = StrategyConfig {
                        short_ma,
                        long_ma,
                        stop_loss,
                        take_profit,
                    };

                    // Симулируем бэктест
                    let result = run_backtest(&config);
                    all_results.push(result);
                }
            }
        }
    }

    println!("Протестировано {} комбинаций параметров", total_combinations);

    // Находим лучшую конфигурацию по Sharpe Ratio
    if let Some(best) = all_results.iter().max_by(|a, b|
        a.sharpe_ratio.partial_cmp(&b.sharpe_ratio).unwrap()
    ) {
        println!("\nЛучшая конфигурация:");
        println!("  Короткая MA: {}", best.config.short_ma);
        println!("  Длинная MA: {}", best.config.long_ma);
        println!("  Stop-loss: {:.1}%", best.config.stop_loss);
        println!("  Take-profit: {:.1}%", best.config.take_profit);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
        println!("  Доходность: {:.2}%", best.total_return);
        println!("  Макс. просадка: {:.2}%", best.max_drawdown);
        println!("  Win Rate: {:.2}%", best.win_rate);
    }
}

// Симуляция бэктеста (заглушка)
fn run_backtest(config: &StrategyConfig) -> BacktestResult {
    // В реальности здесь был бы полноценный бэктестинг
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    config.short_ma.hash(&mut hasher);
    config.long_ma.hash(&mut hasher);
    let seed = hasher.finish();

    // Псевдослучайные метрики для демонстрации
    let random = (seed % 10000) as f64 / 10000.0;

    BacktestResult {
        config: config.clone(),
        total_return: 5.0 + random * 30.0,
        sharpe_ratio: 0.5 + random * 2.0,
        max_drawdown: 5.0 + random * 15.0,
        win_rate: 45.0 + random * 30.0,
    }
}
```

## Итератор для grid search

Лучшая практика — создать итератор для ленивого перебора параметров:

```rust
struct GridSearchIter {
    short_ma_values: Vec<usize>,
    long_ma_values: Vec<usize>,
    stop_loss_values: Vec<f64>,
    take_profit_values: Vec<f64>,
    current_indices: [usize; 4],
    finished: bool,
}

impl GridSearchIter {
    fn new(
        short_ma: Vec<usize>,
        long_ma: Vec<usize>,
        stop_loss: Vec<f64>,
        take_profit: Vec<f64>,
    ) -> Self {
        Self {
            short_ma_values: short_ma,
            long_ma_values: long_ma,
            stop_loss_values: stop_loss,
            take_profit_values: take_profit,
            current_indices: [0, 0, 0, 0],
            finished: false,
        }
    }

    fn increment_indices(&mut self) -> bool {
        // Инкрементируем индексы как многомерный счётчик
        let mut carry = true;

        // Take profit (последний индекс)
        if carry {
            self.current_indices[3] += 1;
            if self.current_indices[3] >= self.take_profit_values.len() {
                self.current_indices[3] = 0;
            } else {
                carry = false;
            }
        }

        // Stop loss
        if carry {
            self.current_indices[2] += 1;
            if self.current_indices[2] >= self.stop_loss_values.len() {
                self.current_indices[2] = 0;
            } else {
                carry = false;
            }
        }

        // Long MA
        if carry {
            self.current_indices[1] += 1;
            if self.current_indices[1] >= self.long_ma_values.len() {
                self.current_indices[1] = 0;
            } else {
                carry = false;
            }
        }

        // Short MA
        if carry {
            self.current_indices[0] += 1;
            if self.current_indices[0] >= self.short_ma_values.len() {
                return false; // Дошли до конца
            }
        }

        true
    }
}

impl Iterator for GridSearchIter {
    type Item = StrategyConfig;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            let config = StrategyConfig {
                short_ma: self.short_ma_values[self.current_indices[0]],
                long_ma: self.long_ma_values[self.current_indices[1]],
                stop_loss: self.stop_loss_values[self.current_indices[2]],
                take_profit: self.take_profit_values[self.current_indices[3]],
            };

            // Подготавливаем следующую итерацию
            if !self.increment_indices() {
                self.finished = true;
            }

            // Пропускаем некорректные комбинации
            if config.short_ma < config.long_ma {
                return Some(config);
            }

            if self.finished {
                return None;
            }
        }
    }
}

fn main() {
    let grid = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    );

    let mut count = 0;
    let mut best_result: Option<BacktestResult> = None;

    for config in grid {
        count += 1;
        let result = run_backtest(&config);

        // Обновляем лучший результат
        best_result = match best_result {
            None => Some(result),
            Some(current_best) => {
                if result.sharpe_ratio > current_best.sharpe_ratio {
                    Some(result)
                } else {
                    Some(current_best)
                }
            }
        };

        if count % 10 == 0 {
            println!("Протестировано {} конфигураций...", count);
        }
    }

    println!("\nВсего протестировано: {} конфигураций", count);

    if let Some(best) = best_result {
        println!("\nЛучший результат:");
        println!("  MA: {}/{}", best.config.short_ma, best.config.long_ma);
        println!("  Stop/Take: {:.1}%/{:.1}%",
            best.config.stop_loss, best.config.take_profit);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
    }
}
```

## Параллельный grid search

Для ускорения процесса используем многопоточность с Rayon:

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

fn parallel_grid_search() {
    let grid_configs: Vec<StrategyConfig> = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    ).collect();

    println!("Запуск параллельного тестирования {} конфигураций...",
        grid_configs.len());

    let best_result = Arc::new(Mutex::new(None::<BacktestResult>));

    // Параллельный перебор с Rayon
    grid_configs.par_iter().for_each(|config| {
        let result = run_backtest(config);

        // Обновляем лучший результат потокобезопасно
        let mut best = best_result.lock().unwrap();
        match &*best {
            None => *best = Some(result),
            Some(current_best) => {
                if result.sharpe_ratio > current_best.sharpe_ratio {
                    *best = Some(result);
                }
            }
        }
    });

    let best = best_result.lock().unwrap();
    if let Some(ref result) = *best {
        println!("\nЛучшая конфигурация (параллельный поиск):");
        println!("  MA: {}/{}", result.config.short_ma, result.config.long_ma);
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
    }
}

fn main() {
    use std::time::Instant;

    let start = Instant::now();
    parallel_grid_search();
    let duration = start.elapsed();

    println!("\nВремя выполнения: {:?}", duration);
}
```

## Продвинутый пример: многоуровневый grid search

```rust
#[derive(Debug, Clone)]
struct AdvancedStrategyConfig {
    // Индикаторы
    ma_short: usize,
    ma_long: usize,
    rsi_period: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,

    // Управление рисками
    stop_loss_pct: f64,
    take_profit_pct: f64,
    position_size_pct: f64,

    // Фильтры
    min_volume: f64,
    max_spread_pct: f64,
}

#[derive(Debug)]
struct AdvancedBacktestResult {
    config: AdvancedStrategyConfig,
    total_return: f64,
    sharpe_ratio: f64,
    sortino_ratio: f64,
    max_drawdown: f64,
    calmar_ratio: f64,
    win_rate: f64,
    profit_factor: f64,
    total_trades: usize,
}

struct AdvancedGridSearch {
    configs: Vec<AdvancedStrategyConfig>,
}

impl AdvancedGridSearch {
    fn new() -> Self {
        let mut configs = Vec::new();

        // Параметры индикаторов
        let ma_short_values = vec![5, 10, 20];
        let ma_long_values = vec![50, 100, 200];
        let rsi_periods = vec![14, 21];
        let rsi_oversold_values = vec![20.0, 30.0];
        let rsi_overbought_values = vec![70.0, 80.0];

        // Параметры риска
        let stop_loss_values = vec![1.0, 2.0, 3.0];
        let take_profit_values = vec![3.0, 6.0, 9.0];
        let position_sizes = vec![25.0, 50.0, 100.0];

        // Фильтры
        let min_volumes = vec![100000.0, 500000.0];
        let max_spreads = vec![0.1, 0.2];

        // Генерация всех комбинаций
        for &ma_short in &ma_short_values {
            for &ma_long in &ma_long_values {
                if ma_short >= ma_long { continue; }

                for &rsi_period in &rsi_periods {
                    for &rsi_oversold in &rsi_oversold_values {
                        for &rsi_overbought in &rsi_overbought_values {
                            if rsi_oversold >= rsi_overbought { continue; }

                            for &stop_loss in &stop_loss_values {
                                for &take_profit in &take_profit_values {
                                    if take_profit <= stop_loss { continue; }

                                    for &position_size in &position_sizes {
                                        for &min_volume in &min_volumes {
                                            for &max_spread in &max_spreads {
                                                configs.push(AdvancedStrategyConfig {
                                                    ma_short,
                                                    ma_long,
                                                    rsi_period,
                                                    rsi_oversold,
                                                    rsi_overbought,
                                                    stop_loss_pct: stop_loss,
                                                    take_profit_pct: take_profit,
                                                    position_size_pct: position_size,
                                                    min_volume,
                                                    max_spread_pct: max_spread,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { configs }
    }

    fn run_parallel(&self) -> Vec<AdvancedBacktestResult> {
        use rayon::prelude::*;

        println!("Тестирование {} конфигураций...", self.configs.len());

        self.configs.par_iter()
            .map(|config| run_advanced_backtest(config))
            .collect()
    }

    fn find_best_by_metric<F>(&self, results: &[AdvancedBacktestResult],
                               metric_fn: F) -> Option<&AdvancedBacktestResult>
    where
        F: Fn(&AdvancedBacktestResult) -> f64,
    {
        results.iter()
            .max_by(|a, b| metric_fn(a).partial_cmp(&metric_fn(b)).unwrap())
    }
}

fn run_advanced_backtest(config: &AdvancedStrategyConfig) -> AdvancedBacktestResult {
    // Симуляция бэктеста
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    config.ma_short.hash(&mut hasher);
    config.ma_long.hash(&mut hasher);
    config.rsi_period.hash(&mut hasher);
    let seed = (hasher.finish() % 10000) as f64 / 10000.0;

    AdvancedBacktestResult {
        config: config.clone(),
        total_return: 10.0 + seed * 50.0,
        sharpe_ratio: 0.5 + seed * 2.5,
        sortino_ratio: 0.6 + seed * 2.8,
        max_drawdown: 8.0 + seed * 20.0,
        calmar_ratio: 0.3 + seed * 1.5,
        win_rate: 40.0 + seed * 35.0,
        profit_factor: 1.0 + seed * 2.0,
        total_trades: 50 + (seed * 200.0) as usize,
    }
}

fn main() {
    let grid_search = AdvancedGridSearch::new();
    let results = grid_search.run_parallel();

    // Находим лучшие по разным метрикам
    println!("\n=== Результаты Grid Search ===\n");

    if let Some(best_sharpe) = grid_search.find_best_by_metric(&results,
        |r| r.sharpe_ratio) {
        println!("Лучший Sharpe Ratio: {:.2}", best_sharpe.sharpe_ratio);
        println!("  MA: {}/{}", best_sharpe.config.ma_short,
            best_sharpe.config.ma_long);
        println!("  RSI: {} ({:.0}/{:.0})", best_sharpe.config.rsi_period,
            best_sharpe.config.rsi_oversold, best_sharpe.config.rsi_overbought);
    }

    if let Some(best_return) = grid_search.find_best_by_metric(&results,
        |r| r.total_return) {
        println!("\nЛучшая доходность: {:.2}%", best_return.total_return);
        println!("  Размер позиции: {:.0}%", best_return.config.position_size_pct);
        println!("  Stop/Take: {:.1}%/{:.1}%",
            best_return.config.stop_loss_pct, best_return.config.take_profit_pct);
    }

    if let Some(best_calmar) = grid_search.find_best_by_metric(&results,
        |r| r.calmar_ratio) {
        println!("\nЛучший Calmar Ratio: {:.2}", best_calmar.calmar_ratio);
        println!("  Макс. просадка: {:.2}%", best_calmar.max_drawdown);
        println!("  Win Rate: {:.2}%", best_calmar.win_rate);
    }

    // Статистика
    let avg_sharpe: f64 = results.iter()
        .map(|r| r.sharpe_ratio)
        .sum::<f64>() / results.len() as f64;

    println!("\n=== Общая статистика ===");
    println!("Протестировано конфигураций: {}", results.len());
    println!("Средний Sharpe Ratio: {:.2}", avg_sharpe);
}
```

## Оптимизация: раннее прерывание

```rust
fn grid_search_with_early_stopping(
    min_sharpe_threshold: f64,
    max_iterations: usize,
) -> Option<BacktestResult> {
    let grid = GridSearchIter::new(
        vec![5, 10, 15, 20],
        vec![50, 100, 150, 200],
        vec![1.0, 2.0, 3.0, 5.0],
        vec![2.0, 4.0, 6.0, 10.0],
    );

    let mut best_result: Option<BacktestResult> = None;
    let mut iterations = 0;

    for config in grid {
        iterations += 1;

        let result = run_backtest(&config);

        // Обновляем лучший результат
        let is_new_best = match &best_result {
            None => true,
            Some(current_best) => result.sharpe_ratio > current_best.sharpe_ratio,
        };

        if is_new_best {
            println!("Новый лучший результат на итерации {}: Sharpe={:.2}",
                iterations, result.sharpe_ratio);

            // Проверка порога
            if result.sharpe_ratio >= min_sharpe_threshold {
                println!("Достигнут порог {:.2}! Останавливаем поиск.",
                    min_sharpe_threshold);
                return Some(result);
            }

            best_result = Some(result);
        }

        // Проверка лимита итераций
        if iterations >= max_iterations {
            println!("Достигнут лимит {} итераций.", max_iterations);
            break;
        }
    }

    best_result
}

fn main() {
    println!("Grid search с ранним прерыванием:\n");

    if let Some(result) = grid_search_with_early_stopping(1.8, 100) {
        println!("\nНайдена конфигурация:");
        println!("  Sharpe Ratio: {:.2}", result.sharpe_ratio);
        println!("  MA: {}/{}", result.config.short_ma, result.config.long_ma);
    }
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Grid Search | Исчерпывающий перебор всех комбинаций параметров |
| Итератор параметров | Ленивая генерация конфигураций |
| Параллельный перебор | Использование Rayon для ускорения |
| Многомерный поиск | Оптимизация по нескольким параметрам одновременно |
| Раннее прерывание | Остановка при достижении цели |
| Метрики качества | Sharpe Ratio, Sortino, Calmar, Win Rate |

## Практические задания

1. **Базовый Grid Search**: Реализуй перебор для стратегии Mean Reversion с параметрами:
   - Z-score threshold: 1.5, 2.0, 2.5, 3.0
   - Lookback period: 20, 50, 100
   - Position size: 10%, 25%, 50%

2. **Кастомный итератор**: Создай `GridSearchIter`, который пропускает комбинации, где `take_profit < 2 * stop_loss`.

3. **Параллельный бэктест**: Используй Rayon для параллельного тестирования 1000+ конфигураций стратегии на основе RSI.

4. **Валидация результатов**: Раздели данные на train/test (70%/30%) и проверь, что лучшие параметры на train-данных показывают хорошие результаты на test-данных.

## Домашнее задание

1. **Multi-Metric Optimization**: Создай систему grid search, которая находит конфигурации, оптимальные по нескольким метрикам одновременно (Sharpe > 1.5 И Max Drawdown < 15% И Win Rate > 55%).

2. **Adaptive Grid Search**: Реализуй двухэтапный поиск:
   - Этап 1: Грубый поиск с большим шагом параметров
   - Этап 2: Уточнённый поиск вокруг лучших результатов с малым шагом

3. **Cross-Validation Grid Search**: Для каждой конфигурации выполняй K-fold cross-validation (k=5) и усредняй результаты для более надёжной оценки.

4. **Logging и визуализация**: Сохраняй все результаты в CSV файл и создай скрипт для построения heatmap зависимости Sharpe Ratio от пары параметров (например, MA short vs MA long).

## Навигация

[← Предыдущий день](../170-crossbeam-advanced-concurrency/ru.md) | [Следующий день →](../294-genetic-algorithms-optimization/ru.md)
