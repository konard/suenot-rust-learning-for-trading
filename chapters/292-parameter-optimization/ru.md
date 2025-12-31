# День 292: Оптимизация параметров

## Аналогия из трейдинга

Представь, что ты разработал торговую стратегию на основе скользящих средних:
- Покупай, когда быстрая MA пересекает медленную MA снизу вверх
- Продавай, когда быстрая MA пересекает медленную MA сверху вниз

Но какие периоды использовать? 10 и 20? 5 и 15? 20 и 50? 50 и 200? Каждая комбинация даст разные результаты. **Оптимизация параметров** — это систематический процесс тестирования различных комбинаций параметров стратегии, чтобы найти те, которые показывают лучшие результаты на исторических данных.

Это как настройка музыкального инструмента — ты методично подстраиваешь каждую струну (параметр), проверяя, как это влияет на звучание (производительность стратегии), чтобы найти оптимальную конфигурацию.

## Что такое оптимизация параметров?

**Оптимизация параметров** (Parameter Optimization) — это процесс поиска наилучших значений входных параметров торговой стратегии путём систематического тестирования различных комбинаций на исторических данных.

### Основные концепции

| Концепция | Описание | Пример в трейдинге |
|-----------|----------|-------------------|
| Параметр | Настраиваемое значение стратегии | Период MA, уровень стоп-лосса, размер позиции |
| Пространство поиска | Диапазон возможных значений параметров | MA период: от 5 до 200 дней |
| Целевая функция | Метрика для оптимизации | Sharpe Ratio, общая прибыль, максимальная просадка |
| Локальный оптимум | Лучшее значение в ближайшей области | MA(10,20) хорош, но MA(50,200) может быть лучше |
| Глобальный оптимум | Наилучшее значение во всём пространстве | Абсолютно лучшая комбинация параметров |

## Параметры торговой стратегии в Rust

Начнём с определения структуры стратегии с параметрами:

```rust
#[derive(Debug, Clone)]
struct MovingAverageCrossStrategy {
    fast_period: usize,   // Быстрая скользящая средняя
    slow_period: usize,   // Медленная скользящая средняя
    stop_loss_pct: f64,   // Стоп-лосс в процентах
    take_profit_pct: f64, // Тейк-профит в процентах
}

impl MovingAverageCrossStrategy {
    fn new(fast_period: usize, slow_period: usize, stop_loss_pct: f64, take_profit_pct: f64) -> Self {
        Self {
            fast_period,
            slow_period,
            stop_loss_pct,
            take_profit_pct,
        }
    }

    // Расчёт простой скользящей средней
    fn calculate_sma(&self, prices: &[f64], period: usize) -> Vec<f64> {
        let mut sma = Vec::new();

        if period == 0 {
            return vec![0.0; prices.len()];
        }

        for i in 0..prices.len() {
            if i + 1 < period {
                sma.push(0.0); // Недостаточно данных
            } else {
                let sum: f64 = prices[i + 1 - period..=i].iter().sum();
                sma.push(sum / period as f64);
            }
        }

        sma
    }

    // Генерация торговых сигналов
    fn generate_signals(&self, prices: &[f64]) -> Vec<i8> {
        let fast_ma = self.calculate_sma(prices, self.fast_period);
        let slow_ma = self.calculate_sma(prices, self.slow_period);

        let mut signals = Vec::new();

        for i in 0..prices.len() {
            if i == 0 {
                signals.push(0); // Нет сигнала на первой свече
                continue;
            }

            // Пересечение вверх — покупка
            if fast_ma[i - 1] <= slow_ma[i - 1] && fast_ma[i] > slow_ma[i] {
                signals.push(1); // Buy
            }
            // Пересечение вниз — продажа
            else if fast_ma[i - 1] >= slow_ma[i - 1] && fast_ma[i] < slow_ma[i] {
                signals.push(-1); // Sell
            }
            // Нет пересечения
            else {
                signals.push(0); // Hold
            }
        }

        signals
    }
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0,
        107.0, 109.0, 111.0, 110.0, 112.0, 114.0, 113.0, 115.0,
    ];

    let strategy = MovingAverageCrossStrategy::new(3, 7, 2.0, 5.0);
    let signals = strategy.generate_signals(&prices);

    println!("Параметры стратегии:");
    println!("  Быстрая MA: {}", strategy.fast_period);
    println!("  Медленная MA: {}", strategy.slow_period);
    println!("  Стоп-лосс: {}%", strategy.stop_loss_pct);
    println!("  Тейк-профит: {}%", strategy.take_profit_pct);
    println!("\nТорговые сигналы: {:?}", signals);
}
```

## Метрики производительности

Чтобы оптимизировать параметры, нам нужна целевая функция для оценки качества:

```rust
#[derive(Debug)]
struct BacktestResults {
    total_return: f64,
    num_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    max_drawdown: f64,
    sharpe_ratio: f64,
}

impl BacktestResults {
    fn new() -> Self {
        Self {
            total_return: 0.0,
            num_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
        }
    }

    fn calculate_win_rate(&self) -> f64 {
        if self.num_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.num_trades as f64) * 100.0
    }

    fn calculate_profit_factor(&self) -> f64 {
        if self.losing_trades == 0 {
            return f64::INFINITY;
        }
        self.winning_trades as f64 / self.losing_trades as f64
    }

    // Комплексная оценка качества стратегии
    fn fitness_score(&self) -> f64 {
        // Комбинация различных метрик
        let return_score = self.total_return;
        let sharpe_score = self.sharpe_ratio * 10.0;
        let drawdown_penalty = self.max_drawdown.abs();
        let trade_count_bonus = (self.num_trades as f64).sqrt();

        return_score + sharpe_score - drawdown_penalty + trade_count_bonus
    }
}

// Простой бэктест для демонстрации
fn backtest_strategy(strategy: &MovingAverageCrossStrategy, prices: &[f64]) -> BacktestResults {
    let mut results = BacktestResults::new();
    let signals = strategy.generate_signals(prices);

    let mut position = 0.0; // 0 = нет позиции
    let mut entry_price = 0.0;
    let mut equity = 10000.0; // Начальный капитал
    let mut peak_equity = equity;
    let mut returns = Vec::new();

    for i in 1..prices.len() {
        // Открываем позицию по сигналу
        if position == 0.0 && signals[i] == 1 {
            position = equity / prices[i]; // Покупаем на весь капитал
            entry_price = prices[i];
            results.num_trades += 1;
        }
        // Закрываем позицию по обратному сигналу
        else if position > 0.0 && signals[i] == -1 {
            let exit_value = position * prices[i];
            let trade_return = (exit_value - equity) / equity;
            returns.push(trade_return);

            if exit_value > equity {
                results.winning_trades += 1;
            } else {
                results.losing_trades += 1;
            }

            equity = exit_value;
            position = 0.0;
        }

        // Обновляем equity
        let current_equity = if position > 0.0 {
            position * prices[i]
        } else {
            equity
        };

        // Отслеживаем просадку
        if current_equity > peak_equity {
            peak_equity = current_equity;
        }
        let drawdown = (current_equity - peak_equity) / peak_equity * 100.0;
        if drawdown < results.max_drawdown {
            results.max_drawdown = drawdown;
        }
    }

    results.total_return = ((equity - 10000.0) / 10000.0) * 100.0;

    // Упрощённый расчёт Sharpe Ratio
    if !returns.is_empty() {
        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            results.sharpe_ratio = mean_return / std_dev;
        }
    }

    results
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0,
        107.0, 109.0, 111.0, 110.0, 112.0, 114.0, 113.0, 115.0,
        116.0, 115.0, 117.0, 119.0, 118.0, 120.0, 122.0, 121.0,
    ];

    let strategy = MovingAverageCrossStrategy::new(3, 7, 2.0, 5.0);
    let results = backtest_strategy(&strategy, &prices);

    println!("Результаты бэктеста:");
    println!("  Общая доходность: {:.2}%", results.total_return);
    println!("  Количество сделок: {}", results.num_trades);
    println!("  Прибыльных сделок: {}", results.winning_trades);
    println!("  Убыточных сделок: {}", results.losing_trades);
    println!("  Win Rate: {:.2}%", results.calculate_win_rate());
    println!("  Макс. просадка: {:.2}%", results.max_drawdown);
    println!("  Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("  Fitness Score: {:.2}", results.fitness_score());
}
```

## Простая оптимизация: перебор параметров

Теперь оптимизируем параметры путём перебора различных комбинаций:

```rust
#[derive(Debug, Clone)]
struct OptimizationResult {
    fast_period: usize,
    slow_period: usize,
    fitness_score: f64,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
}

fn optimize_ma_periods(prices: &[f64]) -> Vec<OptimizationResult> {
    let mut results = Vec::new();

    // Диапазоны параметров для оптимизации
    let fast_periods = vec![3, 5, 7, 10, 12, 15];
    let slow_periods = vec![10, 15, 20, 25, 30, 40, 50];

    println!("Начинаем оптимизацию параметров...\n");

    for &fast in &fast_periods {
        for &slow in &slow_periods {
            // Быстрая MA должна быть меньше медленной
            if fast >= slow {
                continue;
            }

            let strategy = MovingAverageCrossStrategy::new(fast, slow, 2.0, 5.0);
            let backtest_result = backtest_strategy(&strategy, prices);

            let opt_result = OptimizationResult {
                fast_period: fast,
                slow_period: slow,
                fitness_score: backtest_result.fitness_score(),
                total_return: backtest_result.total_return,
                sharpe_ratio: backtest_result.sharpe_ratio,
                max_drawdown: backtest_result.max_drawdown,
            };

            println!(
                "MA({}, {}) -> Return: {:.2}%, Sharpe: {:.2}, Drawdown: {:.2}%, Fitness: {:.2}",
                fast, slow,
                opt_result.total_return,
                opt_result.sharpe_ratio,
                opt_result.max_drawdown,
                opt_result.fitness_score
            );

            results.push(opt_result);
        }
    }

    results
}

fn find_best_parameters(results: &[OptimizationResult]) -> Option<&OptimizationResult> {
    results.iter()
        .max_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap())
}

fn main() {
    // Генерируем более реалистичные данные
    let prices: Vec<f64> = (0..100)
        .map(|i| {
            100.0 + (i as f64 * 0.5) + ((i as f64 * 0.1).sin() * 5.0)
        })
        .collect();

    println!("Исторические данные: {} свечей\n", prices.len());

    let results = optimize_ma_periods(&prices);

    println!("\n=== Результаты оптимизации ===\n");

    if let Some(best) = find_best_parameters(&results) {
        println!("Лучшие параметры:");
        println!("  Быстрая MA: {}", best.fast_period);
        println!("  Медленная MA: {}", best.slow_period);
        println!("  Общая доходность: {:.2}%", best.total_return);
        println!("  Sharpe Ratio: {:.2}", best.sharpe_ratio);
        println!("  Макс. просадка: {:.2}%", best.max_drawdown);
        println!("  Fitness Score: {:.2}", best.fitness_score);
    }

    // Топ-5 лучших комбинаций
    println!("\nТоп-5 лучших комбинаций:");
    let mut sorted_results = results.clone();
    sorted_results.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());

    for (i, result) in sorted_results.iter().take(5).enumerate() {
        println!(
            "{}. MA({}, {}) - Fitness: {:.2}, Return: {:.2}%",
            i + 1,
            result.fast_period,
            result.slow_period,
            result.fitness_score,
            result.total_return
        );
    }
}
```

## Многопараметрическая оптимизация

Оптимизация нескольких параметров одновременно:

```rust
#[derive(Debug, Clone)]
struct FullOptimizationResult {
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
    take_profit: f64,
    fitness_score: f64,
    total_return: f64,
}

fn optimize_all_parameters(prices: &[f64]) -> Vec<FullOptimizationResult> {
    let mut results = Vec::new();
    let mut tested = 0;

    let fast_periods = vec![5, 10, 15];
    let slow_periods = vec![20, 30, 40];
    let stop_losses = vec![1.0, 2.0, 3.0];
    let take_profits = vec![3.0, 5.0, 7.0];

    let total_combinations = fast_periods.len()
        * slow_periods.len()
        * stop_losses.len()
        * take_profits.len();

    println!("Тестируем {} комбинаций параметров...\n", total_combinations);

    for &fast in &fast_periods {
        for &slow in &slow_periods {
            if fast >= slow {
                continue;
            }

            for &sl in &stop_losses {
                for &tp in &take_profits {
                    tested += 1;

                    let strategy = MovingAverageCrossStrategy::new(fast, slow, sl, tp);
                    let backtest = backtest_strategy(&strategy, prices);

                    let result = FullOptimizationResult {
                        fast_period: fast,
                        slow_period: slow,
                        stop_loss: sl,
                        take_profit: tp,
                        fitness_score: backtest.fitness_score(),
                        total_return: backtest.total_return,
                    };

                    if tested % 10 == 0 {
                        println!(
                            "Прогресс: {}/{} ({:.1}%)",
                            tested, total_combinations,
                            (tested as f64 / total_combinations as f64) * 100.0
                        );
                    }

                    results.push(result);
                }
            }
        }
    }

    println!("\nОптимизация завершена!\n");
    results
}

fn main() {
    let prices: Vec<f64> = (0..200)
        .map(|i| {
            100.0 + (i as f64 * 0.3) + ((i as f64 * 0.1).sin() * 8.0)
        })
        .collect();

    let results = optimize_all_parameters(&prices);

    // Находим лучший результат
    let best = results.iter()
        .max_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap())
        .unwrap();

    println!("=== Оптимальные параметры ===");
    println!("Быстрая MA: {}", best.fast_period);
    println!("Медленная MA: {}", best.slow_period);
    println!("Стоп-лосс: {}%", best.stop_loss);
    println!("Тейк-профит: {}%", best.take_profit);
    println!("Общая доходность: {:.2}%", best.total_return);
    println!("Fitness Score: {:.2}", best.fitness_score);
}
```

## Важные предупреждения об оптимизации

```rust
// ОПАСНОСТЬ: Переоптимизация (Overfitting)

fn demonstrate_overfitting_risk() {
    println!("⚠️  ОПАСНОСТИ ОПТИМИЗАЦИИ ПАРАМЕТРОВ ⚠️\n");

    println!("1. ПЕРЕОБУЧЕНИЕ (Overfitting):");
    println!("   - Параметры идеально работают на исторических данных");
    println!("   - Но плохо работают на новых данных");
    println!("   - Решение: out-of-sample тестирование\n");

    println!("2. DATA SNOOPING (Подглядывание в данные):");
    println!("   - Многократная оптимизация на одних и тех же данных");
    println!("   - Параметры подстроены под случайные флуктуации");
    println!("   - Решение: отложенная выборка для валидации\n");

    println!("3. OPTIMIZATION BIAS (Смещение оптимизации):");
    println!("   - Выбираем только лучшие результаты");
    println!("   - Игнорируем плохие результаты других параметров");
    println!("   - Решение: walk-forward анализ\n");

    println!("4. CURVE FITTING (Подгонка кривой):");
    println!("   - Слишком много параметров для оптимизации");
    println!("   - Стратегия слишком сложная");
    println!("   - Решение: простота и robustness\n");
}

fn main() {
    demonstrate_overfitting_risk();

    println!("=== ПРАВИЛА БЕЗОПАСНОЙ ОПТИМИЗАЦИИ ===\n");

    println!("✓ Используй in-sample и out-of-sample периоды");
    println!("✓ Проверяй стабильность параметров (sensitivity analysis)");
    println!("✓ Предпочитай простые стратегии сложным");
    println!("✓ Используй cross-validation");
    println!("✓ Тестируй на разных рыночных условиях");
    println!("✓ Отслеживай robustness (устойчивость) параметров");
}
```

## Визуализация пространства параметров

```rust
use std::collections::HashMap;

fn create_heatmap(results: &[OptimizationResult]) {
    let mut heatmap: HashMap<(usize, usize), f64> = HashMap::new();

    for result in results {
        heatmap.insert(
            (result.fast_period, result.slow_period),
            result.fitness_score
        );
    }

    println!("\n=== ТЕПЛОВАЯ КАРТА FITNESS SCORE ===\n");
    println!("     Slow MA Period");
    print!("Fast  ");

    let slow_periods: Vec<usize> = vec![10, 15, 20, 25, 30, 40, 50];
    let fast_periods: Vec<usize> = vec![3, 5, 7, 10, 12, 15];

    for slow in &slow_periods {
        print!("{:>6} ", slow);
    }
    println!();

    for fast in &fast_periods {
        print!("{:>4}  ", fast);
        for slow in &slow_periods {
            if let Some(&score) = heatmap.get(&(*fast, *slow)) {
                let symbol = if score > 50.0 {
                    "██"
                } else if score > 30.0 {
                    "▓▓"
                } else if score > 10.0 {
                    "▒▒"
                } else if score > 0.0 {
                    "░░"
                } else {
                    "  "
                };
                print!("{} ", symbol);
            } else {
                print!("   ");
            }
        }
        println!();
    }

    println!("\nЛегенда: ██ > 50  ▓▓ > 30  ▒▒ > 10  ░░ > 0");
}

fn main() {
    let prices: Vec<f64> = (0..100)
        .map(|i| 100.0 + (i as f64 * 0.5) + ((i as f64 * 0.1).sin() * 5.0))
        .collect();

    let results = optimize_ma_periods(&prices);
    create_heatmap(&results);
}
```

## Практические задания

### Упражнение 1: Однопараметрическая оптимизация
Оптимизируй только период RSI (Relative Strength Index) от 5 до 30. Найди оптимальный период для определения перекупленности/перепроданности.

### Упражнение 2: Двухпараметрическая оптимизация
Оптимизируй стратегию Bollinger Bands:
- Период скользящей средней (10-30)
- Количество стандартных отклонений (1.5-3.0)

### Упражнение 3: Сравнение целевых функций
Реализуй оптимизацию с разными целевыми функциями:
- Максимизация общей прибыли
- Максимизация Sharpe Ratio
- Минимизация максимальной просадки

Сравни найденные оптимальные параметры.

### Упражнение 4: Robustness Testing
Найди оптимальные параметры и протестируй их стабильность:
- Измени параметры на ±10%
- Измерь, насколько меняется производительность
- Найди параметры с наибольшей robustness (устойчивостью)

## Домашнее задание

1. **Оптимизатор с прогресс-баром**: Создай оптимизатор, который показывает детальный прогресс:
   - Текущая комбинация параметров
   - Процент выполнения
   - Оценочное время завершения
   - Лучший найденный результат на данный момент

2. **Параллельная оптимизация**: Используя потоки или `rayon`, распараллель процесс оптимизации. Протестируй различные комбинации параметров одновременно и сравни скорость с последовательным выполнением.

3. **In-Sample / Out-of-Sample**: Раздели данные на две части:
   - In-sample (60%): для оптимизации
   - Out-of-sample (40%): для валидации

   Найди параметры на in-sample, затем протестируй их на out-of-sample. Сравни результаты.

4. **Sensitivity Analysis**: Создай анализ чувствительности параметров:
   - Найди оптимальные параметры
   - Систематически меняй каждый параметр в диапазоне ±30%
   - Построй график, показывающий влияние каждого параметра
   - Определи, какие параметры наиболее критичны

5. **Multi-Objective Optimization**: Реализуй оптимизацию с несколькими целями одновременно:
   - Максимизируй доходность
   - Минимизируй просадку
   - Минимизируй количество сделок (комиссии)

   Используй weighted score или Pareto frontier для балансировки целей.

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Оптимизация параметров | Систематический поиск лучших значений параметров стратегии |
| Целевая функция | Метрика для оценки качества параметров (Sharpe, прибыль, просадка) |
| Пространство поиска | Диапазон возможных значений для каждого параметра |
| Fitness Score | Комплексная оценка качества стратегии |
| Переобучение | Риск оптимизации под исторические данные |
| In-sample / Out-of-sample | Разделение данных для обучения и валидации |
| Robustness | Устойчивость параметров к изменениям |
| Sensitivity Analysis | Анализ чувствительности стратегии к параметрам |

## Навигация

[← Предыдущий день](../291-out-of-sample-testing/ru.md) | [Следующий день →](../293-grid-search-parameter-sweep/ru.md)
