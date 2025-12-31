# День 282: Equity Curve: кривая капитала

## Аналогия из трейдинга

Представь, что ты управляешь торговым счётом и хочешь понять, насколько хорошо работает твоя стратегия. Ты не можешь просто смотреть на последнюю сделку — нужно видеть всю историю изменения капитала. **Equity Curve** (кривая капитала) — это график, показывающий, как меняется стоимость твоего портфеля во времени.

Это как история болезни для пациента: врач не смотрит только на последний анализ, а изучает всю динамику показателей. Для трейдера equity curve — это "медицинская карта" стратегии, которая показывает:
- Стабильно ли растёт капитал?
- Какие были максимальные просадки?
- Как быстро восстанавливалась стратегия после убытков?

## Что такое Equity Curve?

Equity Curve — это временной ряд значений капитала, построенный на основе результатов каждой сделки или на конец каждого торгового периода. Она является ключевым инструментом для:

1. **Визуализации эффективности** — быстрый взгляд на общую картину
2. **Расчёта метрик риска** — максимальная просадка, волатильность доходности
3. **Сравнения стратегий** — какая стратегия лучше?
4. **Определения режимов рынка** — когда стратегия работает, а когда нет

## Базовая структура для Equity Curve

```rust
use std::collections::VecDeque;

/// Точка на кривой капитала
#[derive(Debug, Clone)]
struct EquityPoint {
    timestamp: u64,      // Unix timestamp
    equity: f64,         // Текущий капитал
    cash: f64,           // Свободные средства
    positions_value: f64, // Стоимость открытых позиций
}

/// Кривая капитала с расчётом метрик
#[derive(Debug)]
struct EquityCurve {
    points: Vec<EquityPoint>,
    initial_capital: f64,
    peak_equity: f64,        // Максимальное значение капитала
    peak_timestamp: u64,     // Время достижения пика
}

impl EquityCurve {
    fn new(initial_capital: f64) -> Self {
        EquityCurve {
            points: Vec::new(),
            initial_capital,
            peak_equity: initial_capital,
            peak_timestamp: 0,
        }
    }

    /// Добавить новую точку на кривую
    fn add_point(&mut self, timestamp: u64, cash: f64, positions_value: f64) {
        let equity = cash + positions_value;

        // Обновляем пик, если достигнут новый максимум
        if equity > self.peak_equity {
            self.peak_equity = equity;
            self.peak_timestamp = timestamp;
        }

        self.points.push(EquityPoint {
            timestamp,
            equity,
            cash,
            positions_value,
        });
    }

    /// Получить текущий капитал
    fn current_equity(&self) -> f64 {
        self.points.last()
            .map(|p| p.equity)
            .unwrap_or(self.initial_capital)
    }

    /// Рассчитать общую доходность в процентах
    fn total_return(&self) -> f64 {
        let current = self.current_equity();
        ((current - self.initial_capital) / self.initial_capital) * 100.0
    }
}

fn main() {
    let mut curve = EquityCurve::new(100_000.0);

    // Симулируем изменение капитала
    curve.add_point(1, 100_000.0, 0.0);        // Начало
    curve.add_point(2, 95_000.0, 7_000.0);     // Покупка BTC
    curve.add_point(3, 95_000.0, 8_500.0);     // BTC вырос
    curve.add_point(4, 95_000.0, 6_000.0);     // BTC упал
    curve.add_point(5, 101_500.0, 0.0);        // Продали с прибылью

    println!("Начальный капитал: ${:.2}", curve.initial_capital);
    println!("Текущий капитал: ${:.2}", curve.current_equity());
    println!("Общая доходность: {:.2}%", curve.total_return());
    println!("Пик капитала: ${:.2}", curve.peak_equity);
}
```

## Расчёт просадки (Drawdown)

Просадка — это снижение капитала от пикового значения. Это критически важная метрика для оценки риска стратегии.

```rust
/// Расширенная кривая капитала с расчётом просадок
#[derive(Debug)]
struct AdvancedEquityCurve {
    points: Vec<EquityPoint>,
    initial_capital: f64,
    peak_equity: f64,
    max_drawdown: f64,           // Максимальная просадка в %
    max_drawdown_duration: u64,  // Длительность максимальной просадки
    current_drawdown: f64,       // Текущая просадка в %
}

#[derive(Debug, Clone)]
struct EquityPoint {
    timestamp: u64,
    equity: f64,
    cash: f64,
    positions_value: f64,
    drawdown: f64,  // Просадка от пика в %
}

impl AdvancedEquityCurve {
    fn new(initial_capital: f64) -> Self {
        AdvancedEquityCurve {
            points: Vec::new(),
            initial_capital,
            peak_equity: initial_capital,
            max_drawdown: 0.0,
            max_drawdown_duration: 0,
            current_drawdown: 0.0,
        }
    }

    fn add_point(&mut self, timestamp: u64, cash: f64, positions_value: f64) {
        let equity = cash + positions_value;

        // Обновляем пик
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        // Рассчитываем просадку
        let drawdown = if self.peak_equity > 0.0 {
            ((self.peak_equity - equity) / self.peak_equity) * 100.0
        } else {
            0.0
        };

        self.current_drawdown = drawdown;

        // Обновляем максимальную просадку
        if drawdown > self.max_drawdown {
            self.max_drawdown = drawdown;
        }

        self.points.push(EquityPoint {
            timestamp,
            equity,
            cash,
            positions_value,
            drawdown,
        });
    }

    /// Получить все периоды просадок
    fn get_drawdown_periods(&self) -> Vec<DrawdownPeriod> {
        let mut periods = Vec::new();
        let mut in_drawdown = false;
        let mut start_idx = 0;
        let mut peak_before_dd = 0.0;

        for (i, point) in self.points.iter().enumerate() {
            if !in_drawdown && point.drawdown > 0.0 {
                // Начало просадки
                in_drawdown = true;
                start_idx = i;
                peak_before_dd = if i > 0 {
                    self.points[i - 1].equity
                } else {
                    self.initial_capital
                };
            } else if in_drawdown && point.drawdown == 0.0 {
                // Конец просадки (восстановление)
                in_drawdown = false;
                let max_dd = self.points[start_idx..i]
                    .iter()
                    .map(|p| p.drawdown)
                    .fold(0.0, f64::max);

                periods.push(DrawdownPeriod {
                    start_timestamp: self.points[start_idx].timestamp,
                    end_timestamp: point.timestamp,
                    max_drawdown: max_dd,
                    recovery_time: point.timestamp - self.points[start_idx].timestamp,
                });
            }
        }

        // Если всё ещё в просадке
        if in_drawdown {
            let last = self.points.last().unwrap();
            let max_dd = self.points[start_idx..]
                .iter()
                .map(|p| p.drawdown)
                .fold(0.0, f64::max);

            periods.push(DrawdownPeriod {
                start_timestamp: self.points[start_idx].timestamp,
                end_timestamp: last.timestamp,
                max_drawdown: max_dd,
                recovery_time: 0, // Ещё не восстановились
            });
        }

        periods
    }
}

#[derive(Debug)]
struct DrawdownPeriod {
    start_timestamp: u64,
    end_timestamp: u64,
    max_drawdown: f64,
    recovery_time: u64,
}

fn main() {
    let mut curve = AdvancedEquityCurve::new(100_000.0);

    // Симулируем торговлю с просадками
    let equity_history = vec![
        (1, 100_000.0),
        (2, 102_000.0),  // +2%
        (3, 105_000.0),  // Новый пик
        (4, 98_000.0),   // Просадка -6.67%
        (5, 95_000.0),   // Просадка -9.52%
        (6, 100_000.0),  // Восстановление
        (7, 108_000.0),  // Новый пик
        (8, 103_000.0),  // Небольшая просадка
        (9, 110_000.0),  // Новый пик
    ];

    for (ts, equity) in equity_history {
        curve.add_point(ts, equity, 0.0);
    }

    println!("=== Анализ Equity Curve ===\n");
    println!("Начальный капитал: ${:.2}", curve.initial_capital);
    println!("Пик капитала: ${:.2}", curve.peak_equity);
    println!("Максимальная просадка: {:.2}%", curve.max_drawdown);

    println!("\n--- Периоды просадок ---");
    for (i, period) in curve.get_drawdown_periods().iter().enumerate() {
        println!(
            "Просадка #{}: {:.2}% (с {} по {}, восстановление: {} периодов)",
            i + 1,
            period.max_drawdown,
            period.start_timestamp,
            period.end_timestamp,
            period.recovery_time
        );
    }
}
```

## Расчёт ключевых метрик

```rust
use std::f64::consts::E;

/// Полноценный анализатор equity curve
struct EquityAnalyzer {
    returns: Vec<f64>,          // Дневные доходности
    equity_values: Vec<f64>,    // Значения капитала
    risk_free_rate: f64,        // Безрисковая ставка (годовая)
}

impl EquityAnalyzer {
    fn new(equity_values: Vec<f64>, risk_free_rate: f64) -> Self {
        // Рассчитываем доходности
        let returns: Vec<f64> = equity_values.windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        EquityAnalyzer {
            returns,
            equity_values,
            risk_free_rate,
        }
    }

    /// Средняя доходность
    fn mean_return(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }
        self.returns.iter().sum::<f64>() / self.returns.len() as f64
    }

    /// Стандартное отклонение доходности (волатильность)
    fn volatility(&self) -> f64 {
        if self.returns.len() < 2 {
            return 0.0;
        }

        let mean = self.mean_return();
        let variance: f64 = self.returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (self.returns.len() - 1) as f64;

        variance.sqrt()
    }

    /// Коэффициент Шарпа (годовой)
    fn sharpe_ratio(&self, periods_per_year: f64) -> f64 {
        let vol = self.volatility();
        if vol == 0.0 {
            return 0.0;
        }

        let mean_return = self.mean_return();
        let excess_return = mean_return - (self.risk_free_rate / periods_per_year);

        (excess_return / vol) * periods_per_year.sqrt()
    }

    /// Коэффициент Сортино (учитывает только отрицательную волатильность)
    fn sortino_ratio(&self, periods_per_year: f64) -> f64 {
        let mean = self.mean_return();

        // Считаем только отрицательные отклонения
        let downside_returns: Vec<f64> = self.returns.iter()
            .filter(|&&r| r < 0.0)
            .cloned()
            .collect();

        if downside_returns.is_empty() {
            return f64::INFINITY; // Нет отрицательных доходностей
        }

        let downside_variance: f64 = downside_returns.iter()
            .map(|r| r.powi(2))
            .sum::<f64>() / downside_returns.len() as f64;

        let downside_deviation = downside_variance.sqrt();

        if downside_deviation == 0.0 {
            return f64::INFINITY;
        }

        let excess_return = mean - (self.risk_free_rate / periods_per_year);
        (excess_return / downside_deviation) * periods_per_year.sqrt()
    }

    /// Максимальная просадка
    fn max_drawdown(&self) -> f64 {
        let mut max_dd = 0.0;
        let mut peak = self.equity_values[0];

        for &equity in &self.equity_values {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd * 100.0
    }

    /// Коэффициент Калмара (годовая доходность / макс просадка)
    fn calmar_ratio(&self, periods_per_year: f64) -> f64 {
        let max_dd = self.max_drawdown();
        if max_dd == 0.0 {
            return f64::INFINITY;
        }

        let total_return = (self.equity_values.last().unwrap()
            / self.equity_values.first().unwrap() - 1.0) * 100.0;

        let num_periods = self.equity_values.len() as f64;
        let years = num_periods / periods_per_year;
        let annual_return = total_return / years;

        annual_return / max_dd
    }

    /// Процент прибыльных периодов
    fn win_rate(&self) -> f64 {
        if self.returns.is_empty() {
            return 0.0;
        }

        let wins = self.returns.iter().filter(|&&r| r > 0.0).count();
        (wins as f64 / self.returns.len() as f64) * 100.0
    }

    /// Profit Factor (сумма прибылей / сумма убытков)
    fn profit_factor(&self) -> f64 {
        let gains: f64 = self.returns.iter()
            .filter(|&&r| r > 0.0)
            .sum();

        let losses: f64 = self.returns.iter()
            .filter(|&&r| r < 0.0)
            .map(|r| r.abs())
            .sum();

        if losses == 0.0 {
            return f64::INFINITY;
        }

        gains / losses
    }

    /// Вывод всех метрик
    fn print_report(&self, periods_per_year: f64) {
        println!("╔════════════════════════════════════════╗");
        println!("║     ОТЧЁТ ПО EQUITY CURVE              ║");
        println!("╠════════════════════════════════════════╣");
        println!("║ Общая доходность:     {:>10.2}%      ║",
            (self.equity_values.last().unwrap() / self.equity_values.first().unwrap() - 1.0) * 100.0);
        println!("║ Средняя доходность:   {:>10.4}%      ║", self.mean_return() * 100.0);
        println!("║ Волатильность:        {:>10.4}%      ║", self.volatility() * 100.0);
        println!("║ Макс. просадка:       {:>10.2}%      ║", self.max_drawdown());
        println!("╠════════════════════════════════════════╣");
        println!("║ Коэф. Шарпа:          {:>10.2}       ║", self.sharpe_ratio(periods_per_year));
        println!("║ Коэф. Сортино:        {:>10.2}       ║", self.sortino_ratio(periods_per_year));
        println!("║ Коэф. Калмара:        {:>10.2}       ║", self.calmar_ratio(periods_per_year));
        println!("╠════════════════════════════════════════╣");
        println!("║ Win Rate:             {:>10.2}%      ║", self.win_rate());
        println!("║ Profit Factor:        {:>10.2}       ║", self.profit_factor());
        println!("╚════════════════════════════════════════╝");
    }
}

fn main() {
    // Симулируем дневные значения капитала за год
    let equity_values: Vec<f64> = vec![
        100_000.0, 100_500.0, 101_200.0, 100_800.0, 101_500.0,
        102_300.0, 101_900.0, 103_100.0, 104_200.0, 103_800.0,
        105_100.0, 106_000.0, 105_200.0, 106_800.0, 107_500.0,
        106_900.0, 108_200.0, 109_100.0, 108_500.0, 110_000.0,
    ];

    let analyzer = EquityAnalyzer::new(equity_values, 0.05); // 5% безрисковая ставка

    // 252 торговых дня в году
    analyzer.print_report(252.0);
}
```

## Визуализация Equity Curve в терминале

```rust
/// Простая ASCII-визуализация equity curve
struct AsciiChart {
    width: usize,
    height: usize,
}

impl AsciiChart {
    fn new(width: usize, height: usize) -> Self {
        AsciiChart { width, height }
    }

    fn plot(&self, values: &[f64], title: &str) {
        if values.is_empty() {
            println!("Нет данных для отображения");
            return;
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        println!("\n{}", title);
        println!("{}", "═".repeat(self.width + 10));

        // Создаём сетку
        let mut grid = vec![vec![' '; self.width]; self.height];

        // Заполняем точки
        let step = values.len() as f64 / self.width as f64;
        for x in 0..self.width {
            let idx = (x as f64 * step) as usize;
            if idx < values.len() {
                let normalized = if range > 0.0 {
                    (values[idx] - min_val) / range
                } else {
                    0.5
                };
                let y = ((1.0 - normalized) * (self.height - 1) as f64) as usize;
                let y = y.min(self.height - 1);
                grid[y][x] = '█';
            }
        }

        // Выводим график
        for (i, row) in grid.iter().enumerate() {
            let label = if i == 0 {
                format!("{:>8.0} │", max_val)
            } else if i == self.height - 1 {
                format!("{:>8.0} │", min_val)
            } else {
                "         │".to_string()
            };

            let line: String = row.iter().collect();
            println!("{}{}", label, line);
        }

        println!("         └{}", "─".repeat(self.width));
        println!("          0{:>width$}", values.len() - 1, width = self.width - 1);
    }
}

fn main() {
    // Генерируем equity curve с трендом и шумом
    let mut equity = vec![100_000.0];
    let mut current = 100_000.0;

    for i in 1..50 {
        // Тренд вверх с шумом
        let change = (i as f64 * 50.0) + ((i as f64).sin() * 2000.0);
        current = 100_000.0 + change;
        equity.push(current);
    }

    let chart = AsciiChart::new(50, 15);
    chart.plot(&equity, "Equity Curve - Торговая стратегия");

    println!("\nСтатистика:");
    println!("  Начало: ${:.2}", equity.first().unwrap());
    println!("  Конец:  ${:.2}", equity.last().unwrap());
    println!("  Мин:    ${:.2}", equity.iter().cloned().fold(f64::INFINITY, f64::min));
    println!("  Макс:   ${:.2}", equity.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
}
```

## Практический пример: Бэктестер со встроенной Equity Curve

```rust
use std::collections::HashMap;

/// Структура сделки
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_time: u64,
    exit_time: u64,
    side: TradeSide,
}

#[derive(Debug, Clone, PartialEq)]
enum TradeSide {
    Long,
    Short,
}

impl Trade {
    fn pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn return_pct(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price / self.entry_price - 1.0) * 100.0,
            TradeSide::Short => (self.entry_price / self.exit_price - 1.0) * 100.0,
        }
    }
}

/// Результаты бэктеста
struct BacktestResult {
    trades: Vec<Trade>,
    equity_curve: Vec<(u64, f64)>,  // (timestamp, equity)
    initial_capital: f64,
}

impl BacktestResult {
    fn new(initial_capital: f64) -> Self {
        BacktestResult {
            trades: Vec::new(),
            equity_curve: vec![(0, initial_capital)],
            initial_capital,
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        let last_equity = self.equity_curve.last().unwrap().1;
        let new_equity = last_equity + trade.pnl();
        self.equity_curve.push((trade.exit_time, new_equity));
        self.trades.push(trade);
    }

    fn final_equity(&self) -> f64 {
        self.equity_curve.last().unwrap().1
    }

    fn total_return(&self) -> f64 {
        (self.final_equity() / self.initial_capital - 1.0) * 100.0
    }

    fn max_drawdown(&self) -> f64 {
        let mut max_dd = 0.0;
        let mut peak = self.initial_capital;

        for &(_, equity) in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }

    fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        let wins = self.trades.iter().filter(|t| t.pnl() > 0.0).count();
        (wins as f64 / self.trades.len() as f64) * 100.0
    }

    fn avg_win(&self) -> f64 {
        let wins: Vec<f64> = self.trades.iter()
            .filter(|t| t.pnl() > 0.0)
            .map(|t| t.pnl())
            .collect();

        if wins.is_empty() {
            return 0.0;
        }
        wins.iter().sum::<f64>() / wins.len() as f64
    }

    fn avg_loss(&self) -> f64 {
        let losses: Vec<f64> = self.trades.iter()
            .filter(|t| t.pnl() < 0.0)
            .map(|t| t.pnl().abs())
            .collect();

        if losses.is_empty() {
            return 0.0;
        }
        losses.iter().sum::<f64>() / losses.len() as f64
    }

    fn profit_factor(&self) -> f64 {
        let gross_profit: f64 = self.trades.iter()
            .filter(|t| t.pnl() > 0.0)
            .map(|t| t.pnl())
            .sum();

        let gross_loss: f64 = self.trades.iter()
            .filter(|t| t.pnl() < 0.0)
            .map(|t| t.pnl().abs())
            .sum();

        if gross_loss == 0.0 {
            return f64::INFINITY;
        }
        gross_profit / gross_loss
    }

    fn print_report(&self) {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║         РЕЗУЛЬТАТЫ БЭКТЕСТА              ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Начальный капитал:  ${:>15.2}    ║", self.initial_capital);
        println!("║ Финальный капитал:  ${:>15.2}    ║", self.final_equity());
        println!("║ Общая доходность:   {:>15.2}%   ║", self.total_return());
        println!("╠══════════════════════════════════════════╣");
        println!("║ Всего сделок:       {:>15}     ║", self.trades.len());
        println!("║ Win Rate:           {:>15.2}%   ║", self.win_rate());
        println!("║ Средняя прибыль:    ${:>15.2}    ║", self.avg_win());
        println!("║ Средний убыток:     ${:>15.2}    ║", self.avg_loss());
        println!("╠══════════════════════════════════════════╣");
        println!("║ Profit Factor:      {:>15.2}     ║", self.profit_factor());
        println!("║ Макс. просадка:     {:>15.2}%   ║", self.max_drawdown());
        println!("╚══════════════════════════════════════════╝");
    }
}

fn main() {
    let mut result = BacktestResult::new(100_000.0);

    // Симулируем серию сделок
    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 40000.0,
            exit_price: 42000.0,
            quantity: 1.0,
            entry_time: 1,
            exit_time: 5,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "ETH".to_string(),
            entry_price: 2500.0,
            exit_price: 2400.0,
            quantity: 10.0,
            entry_time: 6,
            exit_time: 10,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 43000.0,
            exit_price: 41000.0,
            quantity: 1.0,
            entry_time: 11,
            exit_time: 15,
            side: TradeSide::Short,
        },
        Trade {
            symbol: "ETH".to_string(),
            entry_price: 2300.0,
            exit_price: 2600.0,
            quantity: 15.0,
            entry_time: 16,
            exit_time: 20,
            side: TradeSide::Long,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 44000.0,
            exit_price: 46000.0,
            quantity: 1.5,
            entry_time: 21,
            exit_time: 25,
            side: TradeSide::Long,
        },
    ];

    for trade in trades {
        println!(
            "Сделка: {} {} @ {:.2} -> {:.2}, P/L: ${:.2}",
            trade.symbol,
            if trade.side == TradeSide::Long { "LONG" } else { "SHORT" },
            trade.entry_price,
            trade.exit_price,
            trade.pnl()
        );
        result.add_trade(trade);
    }

    result.print_report();

    // Выводим equity curve
    println!("\n--- Equity Curve ---");
    for (ts, equity) in &result.equity_curve {
        println!("T={}: ${:.2}", ts, equity);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Equity Curve | Временной ряд значений капитала |
| Drawdown | Просадка от пикового значения |
| Sharpe Ratio | Доходность с учётом волатильности |
| Sortino Ratio | Доходность с учётом только негативной волатильности |
| Calmar Ratio | Годовая доходность / максимальная просадка |
| Profit Factor | Отношение суммы прибылей к сумме убытков |
| Win Rate | Процент прибыльных сделок |

## Практические задания

1. **Простая Equity Curve**: Создай структуру для отслеживания капитала с возможностью добавления сделок. Реализуй методы для расчёта текущего капитала и общей доходности.

2. **Анализ просадок**: Расширь equity curve функцией поиска всех периодов просадок. Для каждой просадки сохраняй: глубину, время начала, время восстановления.

3. **Сравнение стратегий**: Создай программу, которая сравнивает две equity curve и определяет, какая стратегия лучше по:
   - Sharpe Ratio
   - Максимальной просадке
   - Profit Factor

## Домашнее задание

1. **Rolling Metrics**: Реализуй расчёт метрик за скользящее окно (например, Sharpe Ratio за последние 30 дней). Это поможет увидеть, как меняется качество стратегии со временем.

2. **Monte Carlo симуляция**: Используя историю сделок, сгенерируй 1000 случайных перестановок и построй распределение возможных equity curves. Это покажет, насколько результаты зависят от удачи.

3. **Система алертов**: Реализуй систему оповещений, которая:
   - Предупреждает при достижении просадки > 10%
   - Уведомляет при новом максимуме капитала
   - Отслеживает падение Sharpe Ratio ниже заданного порога

4. **Экспорт в CSV**: Добавь возможность экспорта equity curve в CSV-файл для дальнейшего анализа в Excel или Python.

## Навигация

[← Предыдущий день](../281-backtest-report/ru.md) | [Следующий день →](../283-sharpe-ratio/ru.md)
