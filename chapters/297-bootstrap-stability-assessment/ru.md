# День 297: Bootstrap: оценка стабильности

## Аналогия из трейдинга

Представь, что ты разработал прибыльную торговую стратегию, которая показала 25% доходности за прошлый год. Но вот вопрос: **Это настоящий результат или просто везение?**

Это как покерист, выигравший турнир. Он выиграл благодаря мастерству или просто получил удачные карты? Чтобы узнать, нужно заставить его сыграть тот же турнир много раз с разными комбинациями карт.

**Bootstrap** в бэктестинге работает аналогично: мы «перемешиваем» исторические сделки, чтобы создать тысячи альтернативных сценариев и проверить, остаётся ли стратегия прибыльной. Если она стабильно выигрывает во всех вариациях — она надёжна. Если прибыль исчезает при перестановке сделок — это было везение.

## Что такое bootstrap?

Bootstrap — это статистический метод повторной выборки (resampling), который:

| Аспект | Описание |
|--------|----------|
| **Цель** | Оценка стабильности и статистической значимости результатов |
| **Подход** | Создание множества вариаций путём повторной выборки с возвращением |
| **Применение** | Валидация стратегий, оценка рисков, доверительные интервалы |
| **Размер выборки** | Обычно 1000-10000 итераций bootstrap |
| **Преимущество** | Не требует предположений о распределении данных |
| **Результат** | Доверительные интервалы и распределения вероятностей метрик |

## Простой пример: bootstrap сделок

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

// Одна сделка
#[derive(Debug, Clone)]
struct Trade {
    pnl: f64,
    duration_hours: u32,
}

// Результат бэктеста стратегии
#[derive(Debug)]
struct BacktestResult {
    trades: Vec<Trade>,
    total_pnl: f64,
    win_rate: f64,
    avg_pnl_per_trade: f64,
}

impl BacktestResult {
    fn from_trades(trades: &[Trade]) -> Self {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (wins as f64 / trades.len() as f64) * 100.0;
        let avg_pnl = total_pnl / trades.len() as f64;

        Self {
            trades: trades.to_vec(),
            total_pnl,
            win_rate,
            avg_pnl_per_trade: avg_pnl,
        }
    }
}

// Bootstrap: повторная выборка сделок с возвращением
fn bootstrap_trades(original_trades: &[Trade], n_samples: usize) -> Vec<Trade> {
    let mut rng = thread_rng();
    (0..n_samples)
        .map(|_| original_trades.choose(&mut rng).unwrap().clone())
        .collect()
}

fn main() {
    // Исходный бэктест: 100 сделок
    let original_trades = vec![
        Trade { pnl: 150.0, duration_hours: 4 },
        Trade { pnl: -80.0, duration_hours: 3 },
        Trade { pnl: 200.0, duration_hours: 6 },
        Trade { pnl: -50.0, duration_hours: 2 },
        Trade { pnl: 180.0, duration_hours: 5 },
        // ... ещё 95 сделок
        // Для демо сгенерируем их
    ];

    // Симулируем реалистичные сделки
    let mut all_trades = original_trades;
    for i in 0..95 {
        let pnl = if i % 3 == 0 {
            -50.0 - (i as f64 * 2.0) // Убытки
        } else {
            100.0 + (i as f64 * 3.0) // Прибыли
        };
        all_trades.push(Trade {
            pnl,
            duration_hours: 2 + (i % 6) as u32,
        });
    }

    let original_result = BacktestResult::from_trades(&all_trades);

    println!("=== Исходный бэктест ===");
    println!("Общий PnL: ${:.2}", original_result.total_pnl);
    println!("Win Rate: {:.2}%", original_result.win_rate);
    println!("Средний PnL/сделка: ${:.2}", original_result.avg_pnl_per_trade);

    // Bootstrap: 1000 итераций
    let n_bootstrap = 1000;
    let mut bootstrap_pnls = Vec::new();
    let mut bootstrap_win_rates = Vec::new();

    for _ in 0..n_bootstrap {
        let resampled_trades = bootstrap_trades(&all_trades, all_trades.len());
        let result = BacktestResult::from_trades(&resampled_trades);
        bootstrap_pnls.push(result.total_pnl);
        bootstrap_win_rates.push(result.win_rate);
    }

    // Вычисляем статистику
    bootstrap_pnls.sort_by(|a, b| a.partial_cmp(b).unwrap());
    bootstrap_win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let pnl_mean: f64 = bootstrap_pnls.iter().sum::<f64>() / bootstrap_pnls.len() as f64;
    let pnl_5th = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.05) as usize];
    let pnl_95th = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.95) as usize];

    let wr_mean: f64 = bootstrap_win_rates.iter().sum::<f64>() / bootstrap_win_rates.len() as f64;
    let wr_5th = bootstrap_win_rates[(bootstrap_win_rates.len() as f64 * 0.05) as usize];
    let wr_95th = bootstrap_win_rates[(bootstrap_win_rates.len() as f64 * 0.95) as usize];

    println!("\n=== Bootstrap анализ ({} итераций) ===", n_bootstrap);
    println!("90% доверительный интервал PnL: ${:.2} до ${:.2}", pnl_5th, pnl_95th);
    println!("Среднее PnL: ${:.2}", pnl_mean);
    println!("90% ДИ Win Rate: {:.2}% до {:.2}%", wr_5th, wr_95th);
    println!("Среднее Win Rate: {:.2}%", wr_mean);

    // Проверяем, насколько стратегия надёжна
    let profitable_bootstrap = bootstrap_pnls.iter()
        .filter(|&&pnl| pnl > 0.0)
        .count();
    let probability_profitable = (profitable_bootstrap as f64 / n_bootstrap as f64) * 100.0;

    println!("\nВероятность прибыльности: {:.2}%", probability_profitable);
    if probability_profitable > 95.0 {
        println!("✓ Стратегия НАДЁЖНАЯ - стабильно прибыльная");
    } else if probability_profitable > 70.0 {
        println!("⚠ Стратегия УМЕРЕННО СТАБИЛЬНАЯ");
    } else {
        println!("✗ Стратегия НЕСТАБИЛЬНАЯ - результаты могут быть случайными");
    }
}
```

## Блочный bootstrap: сохранение зависимостей

Реальные рыночные данные имеют зависимости (тренды, кластеризация волатильности). Блочный bootstrap сохраняет их:

```rust
// Блочный bootstrap: повторная выборка блоками для сохранения порядка
fn block_bootstrap(trades: &[Trade], block_size: usize) -> Vec<Trade> {
    let mut rng = thread_rng();
    let n_blocks = (trades.len() + block_size - 1) / block_size;

    let mut result = Vec::new();

    for _ in 0..n_blocks {
        // Случайная начальная позиция
        let start = rng.gen_range(0..trades.len().saturating_sub(block_size));
        let end = (start + block_size).min(trades.len());

        result.extend_from_slice(&trades[start..end]);

        if result.len() >= trades.len() {
            break;
        }
    }

    result.truncate(trades.len());
    result
}

fn main() {
    let trades = generate_trades(200); // Генерация примера сделок

    println!("=== Блочный Bootstrap (сохраняет последовательности) ===\n");

    let block_sizes = vec![5, 10, 20];

    for &block_size in &block_sizes {
        let mut bootstrap_pnls = Vec::new();

        for _ in 0..1000 {
            let resampled = block_bootstrap(&trades, block_size);
            let result = BacktestResult::from_trades(&resampled);
            bootstrap_pnls.push(result.total_pnl);
        }

        bootstrap_pnls.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = bootstrap_pnls[bootstrap_pnls.len() / 2];
        let ci_low = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.05) as usize];
        let ci_high = bootstrap_pnls[(bootstrap_pnls.len() as f64 * 0.95) as usize];

        println!("Размер блока: {}", block_size);
        println!("  Медиана PnL: ${:.2}", median);
        println!("  90% ДИ: ${:.2} до ${:.2}", ci_low, ci_high);
    }
}

fn generate_trades(n: usize) -> Vec<Trade> {
    use rand::Rng;
    let mut rng = thread_rng();

    (0..n).map(|i| {
        let pnl = if i % 3 == 0 {
            -rng.gen_range(20.0..100.0)
        } else {
            rng.gen_range(50.0..200.0)
        };

        Trade {
            pnl,
            duration_hours: rng.gen_range(1..24),
        }
    }).collect()
}
```

## Продвинутый пример: мультиметрический bootstrap

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct DetailedTrade {
    pnl: f64,
    return_pct: f64,
    duration_hours: u32,
    max_adverse_excursion: f64, // Наибольший убыток во время сделки
}

#[derive(Debug)]
struct DetailedMetrics {
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    profit_factor: f64,
}

impl DetailedMetrics {
    fn calculate(trades: &[DetailedTrade]) -> Self {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losses: Vec<_> = trades.iter().filter(|t| t.pnl < 0.0).collect();

        let win_rate = (wins.len() as f64 / trades.len() as f64) * 100.0;

        let gross_profit: f64 = wins.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losses.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            f64::INFINITY
        };

        // Упрощённый Sharpe (предполагая дневные доходности)
        let returns: Vec<f64> = trades.iter().map(|t| t.return_pct).collect();
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        let sharpe_ratio = if std_dev > 0.0 {
            mean_return / std_dev * (252.0_f64).sqrt() // Годовой
        } else {
            0.0
        };

        // Расчёт максимальной просадки
        let mut peak = 0.0;
        let mut max_dd = 0.0;
        let mut cumulative = 0.0;

        for trade in trades {
            cumulative += trade.pnl;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = peak - cumulative;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        Self {
            total_return: total_pnl,
            sharpe_ratio,
            max_drawdown: max_dd,
            win_rate,
            profit_factor,
        }
    }
}

struct BootstrapAnalyzer {
    n_iterations: usize,
}

impl BootstrapAnalyzer {
    fn new(n_iterations: usize) -> Self {
        Self { n_iterations }
    }

    fn analyze(&self, trades: &[DetailedTrade]) -> BootstrapReport {
        let mut all_metrics = Vec::new();

        for _ in 0..self.n_iterations {
            let resampled = self.resample(trades);
            let metrics = DetailedMetrics::calculate(&resampled);
            all_metrics.push(metrics);
        }

        BootstrapReport::from_metrics(all_metrics)
    }

    fn resample(&self, trades: &[DetailedTrade]) -> Vec<DetailedTrade> {
        let mut rng = thread_rng();
        (0..trades.len())
            .map(|_| trades.choose(&mut rng).unwrap().clone())
            .collect()
    }
}

#[derive(Debug)]
struct BootstrapReport {
    sharpe_mean: f64,
    sharpe_ci: (f64, f64),
    max_dd_mean: f64,
    max_dd_ci: (f64, f64),
    win_rate_mean: f64,
    win_rate_ci: (f64, f64),
    profit_factor_mean: f64,
    profit_factor_ci: (f64, f64),
}

impl BootstrapReport {
    fn from_metrics(mut metrics: Vec<DetailedMetrics>) -> Self {
        let n = metrics.len();

        // Извлекаем и сортируем каждую метрику
        let mut sharpes: Vec<f64> = metrics.iter().map(|m| m.sharpe_ratio).collect();
        let mut max_dds: Vec<f64> = metrics.iter().map(|m| m.max_drawdown).collect();
        let mut win_rates: Vec<f64> = metrics.iter().map(|m| m.win_rate).collect();
        let mut profit_factors: Vec<f64> = metrics.iter()
            .map(|m| if m.profit_factor.is_finite() { m.profit_factor } else { 10.0 })
            .collect();

        sharpes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        max_dds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
        profit_factors.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile = |sorted: &[f64], p: f64| -> f64 {
            sorted[(sorted.len() as f64 * p) as usize]
        };

        Self {
            sharpe_mean: sharpes.iter().sum::<f64>() / n as f64,
            sharpe_ci: (percentile(&sharpes, 0.05), percentile(&sharpes, 0.95)),
            max_dd_mean: max_dds.iter().sum::<f64>() / n as f64,
            max_dd_ci: (percentile(&max_dds, 0.05), percentile(&max_dds, 0.95)),
            win_rate_mean: win_rates.iter().sum::<f64>() / n as f64,
            win_rate_ci: (percentile(&win_rates, 0.05), percentile(&win_rates, 0.95)),
            profit_factor_mean: profit_factors.iter().sum::<f64>() / n as f64,
            profit_factor_ci: (percentile(&profit_factors, 0.05), percentile(&profit_factors, 0.95)),
        }
    }

    fn print(&self) {
        println!("\n=== Отчёт Bootstrap анализа ===\n");

        println!("Sharpe Ratio:");
        println!("  Среднее: {:.3}", self.sharpe_mean);
        println!("  90% ДИ: [{:.3}, {:.3}]", self.sharpe_ci.0, self.sharpe_ci.1);

        println!("\nМакс. просадка:");
        println!("  Среднее: ${:.2}", self.max_dd_mean);
        println!("  90% ДИ: [${:.2}, ${:.2}]", self.max_dd_ci.0, self.max_dd_ci.1);

        println!("\nWin Rate:");
        println!("  Среднее: {:.2}%", self.win_rate_mean);
        println!("  90% ДИ: [{:.2}%, {:.2}%]", self.win_rate_ci.0, self.win_rate_ci.1);

        println!("\nProfit Factor:");
        println!("  Среднее: {:.2}", self.profit_factor_mean);
        println!("  90% ДИ: [{:.2}, {:.2}]", self.profit_factor_ci.0, self.profit_factor_ci.1);

        // Оценка
        println!("\n=== Оценка стратегии ===");

        if self.sharpe_ci.0 > 1.0 {
            println!("✓ Высокая уверенность: Sharpe Ratio стабильно > 1.0");
        } else if self.sharpe_ci.0 > 0.5 {
            println!("⚠ Умеренная уверенность: Sharpe Ratio переменчив, но положителен");
        } else {
            println!("✗ Низкая уверенность: Sharpe Ratio ненадёжен");
        }

        if self.profit_factor_ci.0 > 1.5 {
            println!("✓ Надёжная прибыльность: Profit Factor стабильно > 1.5");
        } else if self.profit_factor_ci.0 > 1.0 {
            println!("⚠ Маргинальная прибыльность: Profit Factor едва > 1.0");
        } else {
            println!("✗ Убыточна в некоторых сценариях");
        }
    }
}

fn main() {
    use rand::Rng;
    let mut rng = thread_rng();

    // Генерируем реалистичную историю торговли
    let trades: Vec<DetailedTrade> = (0..150).map(|i| {
        let is_win = i % 3 != 0; // 66% win rate
        let pnl = if is_win {
            rng.gen_range(100.0..300.0)
        } else {
            -rng.gen_range(50.0..150.0)
        };

        DetailedTrade {
            pnl,
            return_pct: pnl / 10000.0, // Предполагая позицию $10k
            duration_hours: rng.gen_range(1..48),
            max_adverse_excursion: if is_win {
                rng.gen_range(0.0..50.0)
            } else {
                pnl.abs()
            },
        }
    }).collect();

    println!("Анализ {} сделок с помощью bootstrap...", trades.len());

    let analyzer = BootstrapAnalyzer::new(5000);
    let report = analyzer.analyze(&trades);
    report.print();
}
```

## Параллельный bootstrap для скорости

```rust
use rayon::prelude::*;

fn parallel_bootstrap_analysis(
    trades: &[DetailedTrade],
    n_iterations: usize,
) -> Vec<DetailedMetrics> {
    (0..n_iterations)
        .into_par_iter()
        .map(|_| {
            let resampled = resample_trades(trades);
            DetailedMetrics::calculate(&resampled)
        })
        .collect()
}

fn resample_trades(trades: &[DetailedTrade]) -> Vec<DetailedTrade> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    (0..trades.len())
        .map(|_| trades.choose(&mut rng).unwrap().clone())
        .collect()
}

fn main() {
    let trades = generate_trades(300);

    println!("Запуск параллельного bootstrap с 10,000 итераций...\n");

    let start = std::time::Instant::now();
    let metrics = parallel_bootstrap_analysis(&trades, 10000);
    let duration = start.elapsed();

    println!("Завершено за {:?}", duration);
    println!("Обработано {} bootstrap выборок", metrics.len());

    let report = BootstrapReport::from_metrics(metrics);
    report.print();
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Bootstrap | Метод повторной выборки для оценки стабильности результатов |
| Доверительные интервалы | Диапазон, в котором вероятно находится истинное значение (напр., 90% ДИ) |
| Повторная выборка сделок | Создание альтернативных историй путём перемешивания сделок |
| Блочный bootstrap | Сохранение временных зависимостей при повторной выборке |
| Мультиметрический анализ | Одновременное тестирование нескольких метрик производительности |
| Параллельная обработка | Использование Rayon для ускорения тысяч итераций |

## Практические задания

1. **Базовый Bootstrap**: Реализуй bootstrap анализ для простой стратегии с 50 сделками. Вычисли 95% доверительные интервалы для общего PnL и win rate.

2. **Блочный vs простой**: Сравни простой bootstrap и блочный bootstrap (размер блока = 10) для стратегии в трендовом рынке. Какой даёт более реалистичные результаты?

3. **Минимальный размер выборки**: Экспериментируй с разным количеством сделок (20, 50, 100, 200). При каком количестве доверительные интервалы становятся приемлемо узкими?

4. **Множественные стратегии**: Проведи bootstrap для 3 разных стратегий одновременно и определи, у какой наиболее стабильный Sharpe Ratio.

## Домашнее задание

1. **Percentile Bootstrap**: Реализуй функцию, вычисляющую любой перцентиль (например, 1-й, 5-й, 25-й, 50-й, 75-й, 95-й, 99-й) для любой метрики, и визуализируй полное распределение.

2. **Сравнение стратегий**: Используй bootstrap для определения, статистически ли Стратегия A лучше Стратегии B. Вычисли вероятность того, что A превосходит B, на основе 10,000 bootstrap выборок.

3. **Временной блочный bootstrap**: Реализуй moving block bootstrap, который учитывает временной порядок при повторной выборке. Сравни результаты с простым bootstrap на стратегии mean-reversion.

4. **Метрики риска**: Расширь bootstrap для вычисления доверительных интервалов для:
   - Максимального количества последовательных убытков
   - Value at Risk (VaR) на уровне 95% и 99%
   - Conditional Value at Risk (CVaR)
   - Времени восстановления после просадок

## Навигация

[← Предыдущий день](../296-monte-carlo-simulations/ru.md) | [Следующий день →](../298-confidence-intervals/ru.md)
