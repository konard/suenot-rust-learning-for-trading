# День 302: Сравнение стратегий

## Аналогия из трейдинга

Представь, что у тебя есть три торговых робота:
- Робот A: агрессивный скальпер, делает 100 сделок в день с маленькой прибылью
- Робот B: консервативный долгосрочник, держит позиции неделями
- Робот C: средневолновик, торгует на дневных графиках

Каждый показывает прибыль, но какой лучше? Нельзя просто сравнить абсолютную прибыль — нужно учесть риск, просадки, стабильность, затраты на комиссии. Это как сравнивать автомобили: нельзя выбирать только по максимальной скорости, нужно учитывать расход топлива, надёжность, комфорт.

**Сравнение стратегий** — это систематический подход к оценке торговых алгоритмов по множеству критериев, чтобы выбрать наиболее подходящую стратегию для конкретных условий и целей.

## Зачем сравнивать стратегии?

В алготрейдинге важно объективно оценивать и сравнивать стратегии по следующим причинам:

| Причина | Описание |
|---------|----------|
| **Выбор лучшей** | Определить, какая стратегия наиболее эффективна |
| **Диверсификация** | Подобрать некоррелированные стратегии для снижения риска |
| **Оптимизация портфеля** | Распределить капитал между несколькими стратегиями |
| **Понимание компромиссов** | Осознать баланс между риском и доходностью |
| **Адаптация к рынку** | Выбирать стратегии под текущие рыночные условия |

## Метрики для сравнения

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    entry_time: u64,
    exit_time: u64,
    entry_price: f64,
    exit_price: f64,
    size: f64,
    pnl: f64,
    commission: f64,
}

#[derive(Debug)]
struct StrategyMetrics {
    name: String,
    // Доходность
    total_return: f64,
    annual_return: f64,
    // Риск
    volatility: f64,
    max_drawdown: f64,
    // Эффективность
    sharpe_ratio: f64,
    sortino_ratio: f64,
    calmar_ratio: f64,
    // Торговая активность
    total_trades: usize,
    win_rate: f64,
    profit_factor: f64,
    avg_trade: f64,
    avg_win: f64,
    avg_loss: f64,
}

impl StrategyMetrics {
    fn new(name: &str, trades: &[Trade], initial_capital: f64, days: usize) -> Self {
        let total_trades = trades.len();

        // Расчёт доходности
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_return = total_pnl / initial_capital;
        let annual_return = total_return * (365.0 / days as f64);

        // Расчёт доходности по сделкам
        let returns: Vec<f64> = trades.iter().map(|t| t.pnl / initial_capital).collect();
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Волатильность (стандартное отклонение доходности)
        let variance = returns.iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt() * (252.0_f64).sqrt(); // Annualized

        // Максимальная просадка
        let mut equity = initial_capital;
        let mut peak = initial_capital;
        let mut max_dd = 0.0;

        for trade in trades {
            equity += trade.pnl;
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        // Sharpe Ratio
        let risk_free_rate = 0.02; // 2% годовых
        let sharpe_ratio = if volatility > 0.0 {
            (annual_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        // Sortino Ratio (использует только downside volatility)
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .copied()
            .collect();
        let downside_variance = if !downside_returns.is_empty() {
            downside_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / downside_returns.len() as f64
        } else {
            0.0
        };
        let downside_deviation = downside_variance.sqrt() * (252.0_f64).sqrt();
        let sortino_ratio = if downside_deviation > 0.0 {
            (annual_return - risk_free_rate) / downside_deviation
        } else {
            0.0
        };

        // Calmar Ratio (доходность / максимальная просадка)
        let calmar_ratio = if max_dd > 0.0 {
            annual_return / max_dd
        } else {
            0.0
        };

        // Торговые метрики
        let winning_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl < 0.0).collect();

        let win_rate = winning_trades.len() as f64 / total_trades as f64;
        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|t| t.pnl).sum::<f64>() / winning_trades.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|t| t.pnl).sum::<f64>() / losing_trades.len() as f64
        } else {
            0.0
        };
        let avg_trade = total_pnl / total_trades as f64;

        let total_wins: f64 = winning_trades.iter().map(|t| t.pnl).sum();
        let total_losses: f64 = losing_trades.iter().map(|t| t.pnl.abs()).sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        StrategyMetrics {
            name: name.to_string(),
            total_return,
            annual_return,
            volatility,
            max_drawdown: max_dd,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            total_trades,
            win_rate,
            profit_factor,
            avg_trade,
            avg_win,
            avg_loss,
        }
    }

    fn print(&self) {
        println!("\n=== {} ===", self.name);
        println!("Доходность:");
        println!("  Общая доходность: {:.2}%", self.total_return * 100.0);
        println!("  Годовая доходность: {:.2}%", self.annual_return * 100.0);
        println!("\nРиск:");
        println!("  Волатильность: {:.2}%", self.volatility * 100.0);
        println!("  Макс. просадка: {:.2}%", self.max_drawdown * 100.0);
        println!("\nЭффективность:");
        println!("  Sharpe Ratio: {:.2}", self.sharpe_ratio);
        println!("  Sortino Ratio: {:.2}", self.sortino_ratio);
        println!("  Calmar Ratio: {:.2}", self.calmar_ratio);
        println!("\nТорговля:");
        println!("  Количество сделок: {}", self.total_trades);
        println!("  Win Rate: {:.2}%", self.win_rate * 100.0);
        println!("  Profit Factor: {:.2}", self.profit_factor);
        println!("  Средняя сделка: ${:.2}", self.avg_trade);
        println!("  Средний выигрыш: ${:.2}", self.avg_win);
        println!("  Средний проигрыш: ${:.2}", self.avg_loss);
    }
}

// Генерация тестовых данных
fn generate_trades(
    strategy_type: &str,
    num_trades: usize,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    volatility_factor: f64,
) -> Vec<Trade> {
    let mut trades = Vec::new();
    let mut time = 1000000u64;

    for i in 0..num_trades {
        let is_win = (i as f64 / num_trades as f64) < win_rate;
        let base_pnl = if is_win { avg_win } else { avg_loss };

        // Добавляем вариативность
        let noise = ((i * 7 + 13) % 100) as f64 / 100.0 - 0.5;
        let pnl = base_pnl * (1.0 + noise * volatility_factor);

        trades.push(Trade {
            entry_time: time,
            exit_time: time + 3600,
            entry_price: 42000.0,
            exit_price: 42000.0 + pnl,
            size: 1.0,
            pnl,
            commission: 5.0,
        });

        time += 7200;
    }

    trades
}

fn main() {
    let initial_capital = 10000.0;
    let days = 365;

    // Стратегия A: Агрессивный скальпер
    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let metrics_a = StrategyMetrics::new("Strategy A: Aggressive Scalper", &trades_a, initial_capital, days);

    // Стратегия B: Консервативный долгосрочник
    let trades_b = generate_trades("Position", 50, 0.65, 300.0, -150.0, 0.2);
    let metrics_b = StrategyMetrics::new("Strategy B: Conservative Position", &trades_b, initial_capital, days);

    // Стратегия C: Средневолновик
    let trades_c = generate_trades("Swing", 150, 0.58, 80.0, -60.0, 0.25);
    let metrics_c = StrategyMetrics::new("Strategy C: Swing Trader", &trades_c, initial_capital, days);

    metrics_a.print();
    metrics_b.print();
    metrics_c.print();
}
```

## Сравнительная таблица стратегий

```rust
struct StrategyComparison {
    strategies: Vec<StrategyMetrics>,
}

impl StrategyComparison {
    fn new(strategies: Vec<StrategyMetrics>) -> Self {
        StrategyComparison { strategies }
    }

    fn print_comparison_table(&self) {
        println!("\n{:=<120}", "");
        println!("СРАВНИТЕЛЬНАЯ ТАБЛИЦА СТРАТЕГИЙ");
        println!("{:=<120}", "");

        // Заголовок
        print!("{:<30}", "Метрика");
        for strategy in &self.strategies {
            print!("{:>28}", strategy.name.split(':').next().unwrap_or(&strategy.name));
        }
        println!();
        println!("{:-<120}", "");

        // Доходность
        self.print_row("Годовая доходность (%)", |s| s.annual_return * 100.0);
        self.print_row("Общая доходность (%)", |s| s.total_return * 100.0);

        println!("{:-<120}", "");

        // Риск
        self.print_row("Волатильность (%)", |s| s.volatility * 100.0);
        self.print_row("Макс. просадка (%)", |s| s.max_drawdown * 100.0);

        println!("{:-<120}", "");

        // Эффективность
        self.print_row("Sharpe Ratio", |s| s.sharpe_ratio);
        self.print_row("Sortino Ratio", |s| s.sortino_ratio);
        self.print_row("Calmar Ratio", |s| s.calmar_ratio);

        println!("{:-<120}", "");

        // Торговля
        self.print_row("Количество сделок", |s| s.total_trades as f64);
        self.print_row("Win Rate (%)", |s| s.win_rate * 100.0);
        self.print_row("Profit Factor", |s| s.profit_factor);

        println!("{:=<120}", "");
    }

    fn print_row<F>(&self, label: &str, extract: F)
    where
        F: Fn(&StrategyMetrics) -> f64,
    {
        print!("{:<30}", label);
        for strategy in &self.strategies {
            let value = extract(strategy);
            print!("{:>28.2}", value);
        }
        println!();
    }

    fn rank_strategies(&self) {
        println!("\n=== РЕЙТИНГ СТРАТЕГИЙ ПО КРИТЕРИЯМ ===\n");

        // Рейтинг по Sharpe Ratio
        let mut by_sharpe: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_sharpe.sort_by(|a, b| b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap());

        println!("По Sharpe Ratio:");
        for (i, strategy) in by_sharpe.iter().enumerate() {
            println!("  {}. {} - {:.2}", i + 1, strategy.name, strategy.sharpe_ratio);
        }

        // Рейтинг по доходности
        let mut by_return: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_return.sort_by(|a, b| b.annual_return.partial_cmp(&a.annual_return).unwrap());

        println!("\nПо годовой доходности:");
        for (i, strategy) in by_return.iter().enumerate() {
            println!("  {}. {} - {:.2}%", i + 1, strategy.name, strategy.annual_return * 100.0);
        }

        // Рейтинг по просадке (меньше = лучше)
        let mut by_dd: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_dd.sort_by(|a, b| a.max_drawdown.partial_cmp(&b.max_drawdown).unwrap());

        println!("\nПо минимальной просадке:");
        for (i, strategy) in by_dd.iter().enumerate() {
            println!("  {}. {} - {:.2}%", i + 1, strategy.name, strategy.max_drawdown * 100.0);
        }

        // Рейтинг по Profit Factor
        let mut by_pf: Vec<&StrategyMetrics> = self.strategies.iter().collect();
        by_pf.sort_by(|a, b| b.profit_factor.partial_cmp(&a.profit_factor).unwrap());

        println!("\nПо Profit Factor:");
        for (i, strategy) in by_pf.iter().enumerate() {
            println!("  {}. {} - {:.2}", i + 1, strategy.name, strategy.profit_factor);
        }
    }

    fn find_best(&self, risk_tolerance: f64) -> Option<&StrategyMetrics> {
        // Выбор лучшей стратегии с учётом риск-толерантности
        // risk_tolerance: 0.0 (консервативный) - 1.0 (агрессивный)

        self.strategies.iter()
            .filter(|s| s.max_drawdown <= 0.1 + risk_tolerance * 0.2) // Ограничение по просадке
            .max_by(|a, b| {
                // Взвешенная оценка
                let score_a = a.sharpe_ratio * (1.0 - risk_tolerance) + a.annual_return * risk_tolerance;
                let score_b = b.sharpe_ratio * (1.0 - risk_tolerance) + b.annual_return * risk_tolerance;
                score_a.partial_cmp(&score_b).unwrap()
            })
    }
}

fn main() {
    let initial_capital = 10000.0;
    let days = 365;

    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 50, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 150, 0.58, 80.0, -60.0, 0.25);

    let metrics_a = StrategyMetrics::new("Strategy A: Scalper", &trades_a, initial_capital, days);
    let metrics_b = StrategyMetrics::new("Strategy B: Position", &trades_b, initial_capital, days);
    let metrics_c = StrategyMetrics::new("Strategy C: Swing", &trades_c, initial_capital, days);

    let comparison = StrategyComparison::new(vec![metrics_a, metrics_b, metrics_c]);

    comparison.print_comparison_table();
    comparison.rank_strategies();

    // Поиск лучшей стратегии для консервативного инвестора
    println!("\n=== РЕКОМЕНДАЦИИ ===\n");
    if let Some(best_conservative) = comparison.find_best(0.2) {
        println!("Для консервативного инвестора: {}", best_conservative.name);
    }

    // Поиск лучшей стратегии для агрессивного инвестора
    if let Some(best_aggressive) = comparison.find_best(0.8) {
        println!("Для агрессивного инвестора: {}", best_aggressive.name);
    }
}
```

## Корреляционный анализ

```rust
#[derive(Debug)]
struct StrategyCorrelation {
    strategy1: String,
    strategy2: String,
    correlation: f64,
}

fn calculate_correlation(trades1: &[Trade], trades2: &[Trade]) -> f64 {
    // Синхронизируем сделки по времени
    // Упрощённая версия: берём PnL в процентах для каждой сделки

    let returns1: Vec<f64> = trades1.iter().map(|t| t.pnl).collect();
    let returns2: Vec<f64> = trades2.iter().map(|t| t.pnl).collect();

    let n = returns1.len().min(returns2.len());
    if n == 0 {
        return 0.0;
    }

    let mean1 = returns1[..n].iter().sum::<f64>() / n as f64;
    let mean2 = returns2[..n].iter().sum::<f64>() / n as f64;

    let mut covariance = 0.0;
    let mut var1 = 0.0;
    let mut var2 = 0.0;

    for i in 0..n {
        let diff1 = returns1[i] - mean1;
        let diff2 = returns2[i] - mean2;
        covariance += diff1 * diff2;
        var1 += diff1 * diff1;
        var2 += diff2 * diff2;
    }

    let std1 = (var1 / n as f64).sqrt();
    let std2 = (var2 / n as f64).sqrt();

    if std1 == 0.0 || std2 == 0.0 {
        return 0.0;
    }

    covariance / n as f64 / (std1 * std2)
}

fn analyze_portfolio_diversification(strategies: &[(&str, Vec<Trade>)]) {
    println!("\n=== КОРРЕЛЯЦИОННАЯ МАТРИЦА ===\n");

    let n = strategies.len();
    let mut correlations = Vec::new();

    // Заголовок
    print!("{:<20}", "");
    for (name, _) in strategies {
        print!("{:>15}", name);
    }
    println!();
    println!("{:-<80}", "");

    // Матрица корреляций
    for i in 0..n {
        print!("{:<20}", strategies[i].0);
        for j in 0..n {
            let corr = if i == j {
                1.0
            } else {
                calculate_correlation(&strategies[i].1, &strategies[j].1)
            };

            print!("{:>15.2}", corr);

            if i < j {
                correlations.push(StrategyCorrelation {
                    strategy1: strategies[i].0.to_string(),
                    strategy2: strategies[j].0.to_string(),
                    correlation: corr,
                });
            }
        }
        println!();
    }

    println!("\n=== АНАЛИЗ ДИВЕРСИФИКАЦИИ ===\n");

    // Находим некоррелированные пары
    let mut uncorrelated: Vec<&StrategyCorrelation> = correlations.iter()
        .filter(|c| c.correlation.abs() < 0.3)
        .collect();
    uncorrelated.sort_by(|a, b| a.correlation.abs().partial_cmp(&b.correlation.abs()).unwrap());

    if !uncorrelated.is_empty() {
        println!("Наименее коррелированные пары (хорошо для диверсификации):");
        for corr in uncorrelated.iter().take(3) {
            println!("  {} <-> {}: корреляция = {:.2}",
                corr.strategy1, corr.strategy2, corr.correlation);
        }
    }

    // Находим сильно коррелированные пары
    let mut highly_correlated: Vec<&StrategyCorrelation> = correlations.iter()
        .filter(|c| c.correlation > 0.7)
        .collect();
    highly_correlated.sort_by(|a, b| b.correlation.partial_cmp(&a.correlation).unwrap());

    if !highly_correlated.is_empty() {
        println!("\nСильно коррелированные пары (избыточны в портфеле):");
        for corr in highly_correlated.iter().take(3) {
            println!("  {} <-> {}: корреляция = {:.2}",
                corr.strategy1, corr.strategy2, corr.correlation);
        }
    }
}

fn main() {
    // Создаём стратегии с разными характеристиками
    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 500, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 500, 0.58, 80.0, -60.0, 0.25);

    let strategies = vec![
        ("Scalper", trades_a),
        ("Position", trades_b),
        ("Swing", trades_c),
    ];

    analyze_portfolio_diversification(&strategies);
}
```

## Визуализация сравнения (текстовая)

```rust
fn plot_equity_curves(strategies: &[(&str, Vec<Trade>)], initial_capital: f64) {
    println!("\n=== КРИВЫЕ КАПИТАЛА ===\n");

    let max_trades = strategies.iter().map(|(_, t)| t.len()).max().unwrap_or(0);
    let step = (max_trades / 50).max(1); // 50 точек на графике

    for point in (0..max_trades).step_by(step) {
        print!("{:>4}: ", point);

        for (i, (name, trades)) in strategies.iter().enumerate() {
            let equity: f64 = initial_capital + trades[..point.min(trades.len())]
                .iter()
                .map(|t| t.pnl)
                .sum::<f64>();

            let normalized = ((equity - initial_capital) / initial_capital * 100.0) as i32;
            let bar_length = ((normalized + 50) / 5).max(0).min(30) as usize;

            let markers = ['█', '▓', '▒'];
            print!("{}", markers[i % markers.len()].to_string().repeat(bar_length));
            print!(" ");
        }
        println!();
    }

    println!("\nЛегенда:");
    for (i, (name, _)) in strategies.iter().enumerate() {
        let markers = ['█', '▓', '▒'];
        println!("  {} - {}", markers[i % markers.len()], name);
    }
}

fn main() {
    let initial_capital = 10000.0;

    let trades_a = generate_trades("Scalper", 500, 0.52, 25.0, -20.0, 0.3);
    let trades_b = generate_trades("Position", 500, 0.65, 300.0, -150.0, 0.2);
    let trades_c = generate_trades("Swing", 500, 0.58, 80.0, -60.0, 0.25);

    let strategies = vec![
        ("Scalper", trades_a),
        ("Position", trades_b),
        ("Swing", trades_c),
    ];

    plot_equity_curves(&strategies, initial_capital);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Сравнение стратегий** | Систематическая оценка по множеству метрик |
| **Sharpe Ratio** | Доходность с учётом риска |
| **Sortino Ratio** | Sharpe, но учитывает только отрицательную волатильность |
| **Calmar Ratio** | Доходность относительно максимальной просадки |
| **Корреляция** | Мера зависимости между стратегиями |
| **Диверсификация** | Снижение риска через некоррелированные стратегии |
| **Win Rate** | Процент прибыльных сделок |
| **Profit Factor** | Отношение прибылей к убыткам |

## Практические задания

1. **Базовое сравнение**: Реализуй функцию сравнения двух стратегий:
   - Вычисли все основные метрики (Sharpe, Sortino, Calmar, Profit Factor)
   - Построй таблицу сравнения
   - Выведи рекомендацию, какая стратегия лучше и почему

2. **Корреляционный анализ**: Создай анализатор:
   - Рассчитай корреляции между 5 разными стратегиями
   - Построй корреляционную матрицу
   - Найди пары с корреляцией < 0.2 для диверсификации
   - Предложи оптимальную комбинацию из 3 стратегий

3. **Ранжирование**: Напиши систему скоринга:
   - Присвой веса разным метрикам (доходность 30%, Sharpe 25%, просадка 25%, Profit Factor 20%)
   - Рассчитай итоговый балл для каждой стратегии
   - Сортируй стратегии по баллам
   - Добавь возможность изменять веса в зависимости от риск-профиля

4. **Визуализация**: Построй текстовые графики для сравнения:
   - Equity curves всех стратегий на одном графике
   - Bar chart для сравнения метрик
   - Drawdown curves
   - Monthly returns comparison

## Домашнее задание

1. **Мультистратегический портфель**: Создай систему управления портфелем:
   - Несколько стратегий работают параллельно
   - Распределяй капитал пропорционально Sharpe Ratio
   - Отключай стратегии при просадке > 15%
   - Перебалансируй каждые N сделок

2. **Адаптивное сравнение**: Реализуй динамическое сравнение:
   - Сравнивай стратегии на скользящих окнах (30, 90, 180 дней)
   - Отслеживай, как меняются метрики со временем
   - Определяй, какая стратегия работает лучше в текущих условиях
   - Автоматически переключай между стратегиями

3. **Статистическая значимость**: Добавь проверку:
   - Используй t-test для сравнения средних доходностей
   - Рассчитай p-value для разницы в Sharpe Ratio
   - Определи, является ли различие статистически значимым
   - Построй доверительные интервалы для метрик

4. **Машинное обучение для выбора стратегии**: Создай ML-модель:
   - Признаки: волатильность рынка, тренд, объём
   - Цель: предсказать, какая стратегия покажет лучший результат
   - Обучи на исторических данных
   - Тестируй на out-of-sample данных

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
