# День 296: Monte Carlo симуляция

## Аналогия из трейдинга

Представь, что ты разработал торговую стратегию, которая показала хорошую прибыль на исторических данных. Но ты задаёшься вопросами:
- Что если бы я вошёл в рынок на день раньше или позже?
- Насколько сильно изменится результат при других последовательностях сделок?
- Каков реальный диапазон возможных результатов, а не только один исторический путь?

**Monte Carlo симуляция** — это как проигрывание тысяч альтернативных версий твоей торговой истории. Представь, что ты берёшь колоду своих прошлых сделок (прибыльных и убыточных) и тасуешь её тысячи раз, каждый раз получая новую последовательность результатов.

Это как симулятор "что если", который помогает понять:
- Какова вероятность просадки более 20%?
- Какой максимальный убыток может произойти при текущей стратегии?
- Насколько стабильны мои результаты при разных рыночных условиях?

## Что такое Monte Carlo симуляция?

Monte Carlo — это метод численного моделирования, использующий случайность для оценки вероятностных характеристик системы:

| Аспект | Описание |
|--------|----------|
| **Принцип** | Многократное случайное семплирование для получения распределения результатов |
| **Применение в трейдинге** | Оценка риска, тестирование устойчивости стратегий, прогнозирование |
| **Количество итераций** | Обычно 1,000-10,000 симуляций для надёжных результатов |
| **Преимущество** | Показывает не один результат, а диапазон возможных исходов |
| **Недостаток** | Предполагает, что будущее похоже на прошлое (может быть неверно) |

## Простой пример: перемешивание сделок

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct Trade {
    profit: f64,
    date: String,
}

#[derive(Debug)]
struct SimulationResult {
    final_equity: f64,
    max_drawdown: f64,
    total_return: f64,
}

fn calculate_equity_curve(trades: &[Trade], initial_capital: f64) -> (Vec<f64>, f64) {
    let mut equity_curve = vec![initial_capital];
    let mut max_equity = initial_capital;
    let mut max_drawdown = 0.0;

    for trade in trades {
        let new_equity = equity_curve.last().unwrap() + trade.profit;
        equity_curve.push(new_equity);

        if new_equity > max_equity {
            max_equity = new_equity;
        }

        let drawdown = (max_equity - new_equity) / max_equity * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    (equity_curve, max_drawdown)
}

fn run_single_simulation(trades: &[Trade], initial_capital: f64) -> SimulationResult {
    let mut rng = thread_rng();
    let mut shuffled = trades.to_vec();
    shuffled.shuffle(&mut rng);

    let (equity_curve, max_drawdown) = calculate_equity_curve(&shuffled, initial_capital);
    let final_equity = *equity_curve.last().unwrap();
    let total_return = (final_equity - initial_capital) / initial_capital * 100.0;

    SimulationResult {
        final_equity,
        max_drawdown,
        total_return,
    }
}

fn monte_carlo_analysis(
    trades: &[Trade],
    initial_capital: f64,
    simulations: usize,
) -> Vec<SimulationResult> {
    (0..simulations)
        .map(|_| run_single_simulation(trades, initial_capital))
        .collect()
}

fn main() {
    // Исторические сделки стратегии
    let historical_trades = vec![
        Trade { profit: 150.0, date: "2024-01-15".to_string() },
        Trade { profit: -80.0, date: "2024-01-16".to_string() },
        Trade { profit: 200.0, date: "2024-01-17".to_string() },
        Trade { profit: -120.0, date: "2024-01-18".to_string() },
        Trade { profit: 300.0, date: "2024-01-19".to_string() },
        Trade { profit: 100.0, date: "2024-01-22".to_string() },
        Trade { profit: -90.0, date: "2024-01-23".to_string() },
        Trade { profit: 250.0, date: "2024-01-24".to_string() },
        Trade { profit: -150.0, date: "2024-01-25".to_string() },
        Trade { profit: 180.0, date: "2024-01-26".to_string() },
    ];

    let initial_capital = 10_000.0;
    let num_simulations = 1_000;

    println!("Запуск {} Monte Carlo симуляций...\n", num_simulations);

    let results = monte_carlo_analysis(&historical_trades, initial_capital, num_simulations);

    // Анализ результатов
    let total_returns: Vec<f64> = results.iter().map(|r| r.total_return).collect();
    let max_drawdowns: Vec<f64> = results.iter().map(|r| r.max_drawdown).collect();

    let avg_return = total_returns.iter().sum::<f64>() / total_returns.len() as f64;
    let min_return = total_returns.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_return = total_returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let avg_drawdown = max_drawdowns.iter().sum::<f64>() / max_drawdowns.len() as f64;
    let worst_drawdown = max_drawdowns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("=== Результаты Monte Carlo симуляции ===\n");
    println!("Доходность:");
    println!("  Средняя: {:.2}%", avg_return);
    println!("  Минимальная: {:.2}%", min_return);
    println!("  Максимальная: {:.2}%", max_return);
    println!("\nПросадка:");
    println!("  Средняя: {:.2}%", avg_drawdown);
    println!("  Максимальная (worst case): {:.2}%", worst_drawdown);

    // Процентили
    let mut sorted_returns = total_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p5 = sorted_returns[(sorted_returns.len() as f64 * 0.05) as usize];
    let p95 = sorted_returns[(sorted_returns.len() as f64 * 0.95) as usize];

    println!("\nДоверительный интервал 90%:");
    println!("  5-й процентиль: {:.2}%", p5);
    println!("  95-й процентиль: {:.2}%", p95);
    println!("\nВероятность убытка: {:.1}%",
        (sorted_returns.iter().filter(|&&r| r < 0.0).count() as f64
         / sorted_returns.len() as f64 * 100.0));
}
```

## Продвинутая симуляция: с учётом позиционного размера

```rust
use rand::Rng;

#[derive(Debug, Clone)]
struct DetailedTrade {
    profit_pct: f64,  // Прибыль в процентах от капитала
    position_size: f64,  // Размер позиции (0.0-1.0)
}

fn calculate_compounded_equity(
    trades: &[DetailedTrade],
    initial_capital: f64,
) -> (Vec<f64>, f64, f64) {
    let mut equity = initial_capital;
    let mut equity_curve = vec![equity];
    let mut max_equity = equity;
    let mut max_drawdown = 0.0;

    for trade in trades {
        // Прибыль зависит от размера позиции
        let position_value = equity * trade.position_size;
        let profit = position_value * (trade.profit_pct / 100.0);

        equity += profit;
        equity_curve.push(equity);

        if equity > max_equity {
            max_equity = equity;
        }

        let drawdown = (max_equity - equity) / max_equity * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    let total_return = (equity - initial_capital) / initial_capital * 100.0;
    (equity_curve, max_drawdown, total_return)
}

fn monte_carlo_with_position_sizing(
    base_trades: &[DetailedTrade],
    initial_capital: f64,
    simulations: usize,
) {
    let mut rng = thread_rng();
    let mut all_final_equities = Vec::new();
    let mut all_max_drawdowns = Vec::new();

    for _ in 0..simulations {
        // Перемешиваем порядок сделок
        let mut shuffled = base_trades.to_vec();
        shuffled.shuffle(&mut rng);

        let (_, max_dd, _) = calculate_compounded_equity(&shuffled, initial_capital);
        let final_equity = calculate_compounded_equity(&shuffled, initial_capital).0.last().unwrap().clone();

        all_final_equities.push(final_equity);
        all_max_drawdowns.push(max_dd);
    }

    // Статистика
    all_final_equities.sort_by(|a, b| a.partial_cmp(b).unwrap());
    all_max_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_equity = all_final_equities[simulations / 2];
    let p5_equity = all_final_equities[(simulations as f64 * 0.05) as usize];
    let p95_equity = all_final_equities[(simulations as f64 * 0.95) as usize];

    let median_dd = all_max_drawdowns[simulations / 2];
    let p95_dd = all_max_drawdowns[(simulations as f64 * 0.95) as usize];

    println!("\n=== Monte Carlo с позиционным размером ===\n");
    println!("Конечный капитал:");
    println!("  Медиана: ${:.2}", median_equity);
    println!("  5-й процентиль (худший): ${:.2}", p5_equity);
    println!("  95-й процентиль (лучший): ${:.2}", p95_equity);
    println!("\nМаксимальная просадка:");
    println!("  Медиана: {:.2}%", median_dd);
    println!("  95-й процентиль (worst case): {:.2}%", p95_dd);

    // Вероятность разорения (капитал < 50% от начального)
    let ruin_threshold = initial_capital * 0.5;
    let ruin_count = all_final_equities.iter().filter(|&&eq| eq < ruin_threshold).count();
    println!("\nВероятность потери >50% капитала: {:.2}%",
        ruin_count as f64 / simulations as f64 * 100.0);
}

fn main() {
    let trades = vec![
        DetailedTrade { profit_pct: 3.5, position_size: 0.25 },
        DetailedTrade { profit_pct: -2.0, position_size: 0.25 },
        DetailedTrade { profit_pct: 5.0, position_size: 0.3 },
        DetailedTrade { profit_pct: -3.0, position_size: 0.25 },
        DetailedTrade { profit_pct: 4.2, position_size: 0.25 },
        DetailedTrade { profit_pct: 2.8, position_size: 0.2 },
        DetailedTrade { profit_pct: -1.5, position_size: 0.25 },
        DetailedTrade { profit_pct: 6.0, position_size: 0.3 },
        DetailedTrade { profit_pct: -2.5, position_size: 0.25 },
        DetailedTrade { profit_pct: 3.0, position_size: 0.25 },
    ];

    monte_carlo_with_position_sizing(&trades, 10_000.0, 5_000);
}
```

## Параллельная Monte Carlo с Rayon

```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct MonteCarloStats {
    returns: Vec<f64>,
    drawdowns: Vec<f64>,
    sharpe_ratios: Vec<f64>,
}

impl MonteCarloStats {
    fn new() -> Self {
        Self {
            returns: Vec::new(),
            drawdowns: Vec::new(),
            sharpe_ratios: Vec::new(),
        }
    }

    fn add_result(&mut self, ret: f64, dd: f64, sharpe: f64) {
        self.returns.push(ret);
        self.drawdowns.push(dd);
        self.sharpe_ratios.push(sharpe);
    }

    fn analyze(&mut self) {
        self.returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.sharpe_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    fn percentile(&self, data: &[f64], p: f64) -> f64 {
        let idx = (data.len() as f64 * p) as usize;
        data[idx.min(data.len() - 1)]
    }

    fn print_analysis(&self) {
        println!("\n=== Анализ {} симуляций ===\n", self.returns.len());

        println!("Доходность:");
        println!("  Медиана: {:.2}%", self.percentile(&self.returns, 0.5));
        println!("  5%: {:.2}%", self.percentile(&self.returns, 0.05));
        println!("  95%: {:.2}%", self.percentile(&self.returns, 0.95));

        println!("\nПросадка:");
        println!("  Медиана: {:.2}%", self.percentile(&self.drawdowns, 0.5));
        println!("  95% (worst case): {:.2}%", self.percentile(&self.drawdowns, 0.95));

        println!("\nSharpe Ratio:");
        println!("  Медиана: {:.2}", self.percentile(&self.sharpe_ratios, 0.5));
        println!("  5%: {:.2}", self.percentile(&self.sharpe_ratios, 0.05));
        println!("  95%: {:.2}", self.percentile(&self.sharpe_ratios, 0.95));
    }
}

fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        0.0
    } else {
        (mean_return - risk_free_rate) / std_dev
    }
}

fn parallel_monte_carlo(
    trades: &[Trade],
    initial_capital: f64,
    simulations: usize,
) -> MonteCarloStats {
    let stats = Arc::new(Mutex::new(MonteCarloStats::new()));

    (0..simulations).into_par_iter().for_each(|_| {
        let result = run_single_simulation(trades, initial_capital);

        // Рассчитываем дневные доходности (упрощённо)
        let daily_returns = vec![result.total_return / 252.0; 252];
        let sharpe = calculate_sharpe_ratio(&daily_returns, 0.02);

        let mut stats_lock = stats.lock().unwrap();
        stats_lock.add_result(result.total_return, result.max_drawdown, sharpe);
    });

    let mut final_stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();
    final_stats.analyze();
    final_stats
}

fn main() {
    let trades = vec![
        Trade { profit: 150.0, date: "2024-01-15".to_string() },
        Trade { profit: -80.0, date: "2024-01-16".to_string() },
        Trade { profit: 200.0, date: "2024-01-17".to_string() },
        Trade { profit: -120.0, date: "2024-01-18".to_string() },
        Trade { profit: 300.0, date: "2024-01-19".to_string() },
        Trade { profit: 100.0, date: "2024-01-22".to_string() },
        Trade { profit: -90.0, date: "2024-01-23".to_string() },
        Trade { profit: 250.0, date: "2024-01-24".to_string() },
        Trade { profit: -150.0, date: "2024-01-25".to_string() },
        Trade { profit: 180.0, date: "2024-01-26".to_string() },
    ];

    use std::time::Instant;
    let start = Instant::now();

    let stats = parallel_monte_carlo(&trades, 10_000.0, 10_000);

    let duration = start.elapsed();
    stats.print_analysis();

    println!("\nВремя выполнения: {:?}", duration);
}
```

## Вариация: генерация синтетических цен

```rust
use rand::distributions::{Distribution, Normal};

fn generate_synthetic_prices(
    initial_price: f64,
    num_periods: usize,
    mean_return: f64,      // Средняя дневная доходность
    volatility: f64,       // Дневная волатильность
) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(mean_return, volatility);

    let mut prices = vec![initial_price];

    for _ in 0..num_periods {
        let last_price = *prices.last().unwrap();
        let return_pct = normal.sample(&mut rng);
        let new_price = last_price * (1.0 + return_pct / 100.0);
        prices.push(new_price);
    }

    prices
}

fn monte_carlo_price_simulation(
    initial_price: f64,
    num_periods: usize,
    simulations: usize,
    mean_return: f64,
    volatility: f64,
) {
    let mut final_prices = Vec::new();

    for _ in 0..simulations {
        let prices = generate_synthetic_prices(
            initial_price,
            num_periods,
            mean_return,
            volatility,
        );
        final_prices.push(*prices.last().unwrap());
    }

    final_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("\n=== Симуляция цены актива ===\n");
    println!("Начальная цена: ${:.2}", initial_price);
    println!("Периодов: {}", num_periods);
    println!("Симуляций: {}", simulations);
    println!("\nКонечная цена:");
    println!("  5-й процентиль: ${:.2}",
        final_prices[(simulations as f64 * 0.05) as usize]);
    println!("  Медиана: ${:.2}", final_prices[simulations / 2]);
    println!("  95-й процентиль: ${:.2}",
        final_prices[(simulations as f64 * 0.95) as usize]);

    let prices_below_initial = final_prices.iter()
        .filter(|&&p| p < initial_price)
        .count();
    println!("\nВероятность снижения цены: {:.1}%",
        prices_below_initial as f64 / simulations as f64 * 100.0);
}

fn main() {
    monte_carlo_price_simulation(
        100.0,    // Начальная цена $100
        252,      // 1 год торговых дней
        10_000,   // 10k симуляций
        0.05,     // 0.05% средняя дневная доходность
        1.5,      // 1.5% дневная волатильность
    );
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Monte Carlo симуляция | Метод оценки вероятностных характеристик через случайное семплирование |
| Перемешивание сделок | Тестирование устойчивости стратегии к порядку сделок |
| Позиционный размер | Учёт размера позиции при расчёте компаундинга |
| Параллелизация | Использование Rayon для ускорения симуляций |
| Синтетические цены | Генерация случайных ценовых траекторий |
| Статистический анализ | Процентили, доверительные интервалы, вероятности |

## Практические задания

1. **Базовая симуляция**: Реализуй Monte Carlo для списка из 20 сделок, запусти 1000 симуляций и найди:
   - Среднюю доходность
   - Медианную максимальную просадку
   - Вероятность убытка

2. **Компаундинг**: Модифицируй симуляцию, чтобы каждая сделка учитывала размер позиции как процент от текущего капитала.

3. **Визуализация**: Сохрани результаты 100 equity curves в CSV файл для построения графика веера возможных траекторий.

4. **Value at Risk (VaR)**: Рассчитай VaR на уровне 95% — максимальный убыток, который не будет превышён в 95% случаев.

## Домашнее задание

1. **Симуляция с реинвестированием**: Создай Monte Carlo симуляцию, где прибыль от каждой сделки реинвестируется (размер следующей позиции увеличивается пропорционально капиталу).

2. **Стресс-тестинг**: Запусти симуляции с разными уровнями волатильности (нормальная, высокая, экстремальная) и сравни результаты.

3. **Оптимизация размера позиции**: Используй Monte Carlo для поиска оптимального размера позиции, который максимизирует Sharpe Ratio при ограничении на максимальную просадку.

4. **Корреляция сделок**: Добавь модель, где последовательные сделки имеют корреляцию (например, после убыточной сделки вероятность следующей убыточной выше).

## Навигация

[← Предыдущий день](../293-grid-search-parameter-sweep/ru.md) | [Следующий день →](../297-bootstrap-resampling/ru.md)
