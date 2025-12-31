# День 287: Метрики: Максимальная просадка

## Аналогия из трейдинга

Представьте, что вы запустили торговую стратегию. Вы начали со $100,000, и за несколько месяцев ваш баланс вырос до $150,000 — дела идут отлично! Но затем рынок разворачивается против вас, и ваш баланс падает до $120,000. Это падение на 20% от вашего пика в $150,000 до низшей точки в $120,000 называется **Максимальной просадкой (Maximum Drawdown, MDD)**.

Максимальная просадка — одна из важнейших метрик риска в трейдинге, потому что она показывает:
- **Сколько боли можно ожидать?** — худший возможный убыток от пика
- **Можете ли вы психологически это выдержать?** — просадка в 50% может заставить вас запаниковать и закрыть позиции
- **Сколько капитала вам нужно?** — вам нужен достаточный запас, чтобы пережить просадку

В реальной торговле вы хотите знать:
- Какова максимальная потеря от любого пика до любой последующей низшей точки?
- Сколько времени потребовалось для восстановления до нового пика?
- Остается ли ваша стратегия жизнеспособной после крупной просадки?

## Что такое максимальная просадка?

**Максимальная просадка (MDD)** — это наибольшее процентное снижение стоимости портфеля от исторического пика до последующей низшей точки до достижения нового пика.

### Формула

```
Просадка = (Значение в низшей точке - Значение на пике) / Значение на пике × 100%
MDD = Максимум из всех просадок
```

### Ключевые концепции

1. **Пик (Peak)** — наивысшая точка стоимости портфеля перед снижением
2. **Низшая точка (Trough)** — самая низкая точка после пика
3. **Восстановление (Recovery)** — когда стоимость портфеля достигает нового пика
4. **Период под водой (Underwater Period)** — время, проведенное ниже предыдущего пика

## Простой калькулятор максимальной просадки

```rust
fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        // Обновляем пик, если достигли новой высоты
        if value > peak {
            peak = value;
        }

        // Вычисляем текущую просадку от пика
        let drawdown = (peak - value) / peak * 100.0;

        // Обновляем максимальную просадку, если текущая больше
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

fn main() {
    // Пример кривой доходности: старт с 10k, рост до 15k, падение до 12k
    let equity = vec![
        10000.0, 10500.0, 11000.0, 12000.0, 13000.0,
        15000.0, 14000.0, 13000.0, 12000.0, 13500.0,
        14000.0, 14500.0,
    ];

    let mdd = calculate_max_drawdown(&equity);
    println!("Максимальная просадка: {:.2}%", mdd);

    // Ожидается: 20% (с 15000 до 12000)
    // Расчёт: (15000 - 12000) / 15000 = 0.20 = 20%
}
```

**Вывод:**
```
Максимальная просадка: 20.00%
```

## Детальный анализ просадки

Создадим более полную структуру, которая отслеживает не только MDD, но и когда она произошла:

```rust
#[derive(Debug, Clone)]
struct DrawdownInfo {
    max_drawdown: f64,        // Процент максимальной просадки
    peak_value: f64,          // Значение на пике
    trough_value: f64,        // Значение в низшей точке
    peak_index: usize,        // Когда произошёл пик
    trough_index: usize,      // Когда произошла низшая точка
    current_drawdown: f64,    // Текущая просадка от последнего пика
}

fn analyze_drawdown(equity_curve: &[f64]) -> DrawdownInfo {
    if equity_curve.is_empty() {
        return DrawdownInfo {
            max_drawdown: 0.0,
            peak_value: 0.0,
            trough_value: 0.0,
            peak_index: 0,
            trough_index: 0,
            current_drawdown: 0.0,
        };
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];
    let mut peak_index = 0;
    let mut mdd_peak_index = 0;
    let mut mdd_trough_index = 0;
    let mut mdd_peak_value = equity_curve[0];
    let mut mdd_trough_value = equity_curve[0];

    for (i, &value) in equity_curve.iter().enumerate() {
        // Обновляем пик, если достигли новой высоты
        if value > peak {
            peak = value;
            peak_index = i;
        }

        // Вычисляем текущую просадку от пика
        let drawdown = (peak - value) / peak * 100.0;

        // Обновляем максимальную просадку, если текущая больше
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
            mdd_peak_index = peak_index;
            mdd_trough_index = i;
            mdd_peak_value = peak;
            mdd_trough_value = value;
        }
    }

    // Вычисляем текущую просадку (от самого последнего пика)
    let current_drawdown = (peak - equity_curve[equity_curve.len() - 1]) / peak * 100.0;

    DrawdownInfo {
        max_drawdown,
        peak_value: mdd_peak_value,
        trough_value: mdd_trough_value,
        peak_index: mdd_peak_index,
        trough_index: mdd_trough_index,
        current_drawdown,
    }
}

fn main() {
    let equity = vec![
        10000.0, 11000.0, 12000.0, 15000.0, 14000.0,
        13000.0, 12000.0, 13500.0, 14000.0, 16000.0,
        15000.0, 14500.0,
    ];

    let info = analyze_drawdown(&equity);

    println!("Анализ просадки:");
    println!("  Максимальная просадка: {:.2}%", info.max_drawdown);
    println!("  Значение на пике: ${:.2} (индекс {})", info.peak_value, info.peak_index);
    println!("  Значение в низшей точке: ${:.2} (индекс {})", info.trough_value, info.trough_index);
    println!("  Текущая просадка: {:.2}%", info.current_drawdown);

    // Вычисляем необходимое восстановление
    let recovery_needed = (info.peak_value / info.trough_value - 1.0) * 100.0;
    println!("  Необходимое восстановление от низшей точки: {:.2}%", recovery_needed);
}
```

**Вывод:**
```
Анализ просадки:
  Максимальная просадка: 20.00%
  Значение на пике: $15000.00 (индекс 3)
  Значение в низшей точке: $12000.00 (индекс 6)
  Текущая просадка: 9.38%
  Необходимое восстановление от низшей точки: 25.00%
```

**Важная заметка:** Чтобы восстановиться после потери 20%, нужен рост на 25%! Это потому что вы начинаете с меньшей базы.

## Пример реальной торговой стратегии

Давайте смоделируем простую стратегию пересечения скользящих средних и вычислим её просадку:

```rust
#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    position_size: f64,
}

impl Trade {
    fn profit(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.position_size
    }
}

fn simulate_strategy_equity_curve(
    prices: &[f64],
    initial_capital: f64,
) -> Vec<f64> {
    let mut equity_curve = vec![initial_capital];
    let mut capital = initial_capital;
    let position_size = 1.0; // Торгуем 1 единицей на сигнал

    // Простая стратегия: покупаем при росте цены, продаём при падении
    let mut in_position = false;
    let mut entry_price = 0.0;

    for i in 1..prices.len() {
        let prev_price = prices[i - 1];
        let current_price = prices[i];

        if !in_position && current_price > prev_price {
            // Входим в длинную позицию
            entry_price = current_price;
            in_position = true;
        } else if in_position && current_price < prev_price {
            // Выходим из позиции
            let trade = Trade {
                entry_price,
                exit_price: current_price,
                position_size,
            };
            capital += trade.profit();
            in_position = false;
        }

        equity_curve.push(capital);
    }

    equity_curve
}

fn main() {
    // Симулированные цены BTC во времени
    let btc_prices = vec![
        40000.0, 42000.0, 45000.0, 43000.0, 41000.0,
        44000.0, 48000.0, 50000.0, 47000.0, 45000.0,
        43000.0, 46000.0, 49000.0, 52000.0, 51000.0,
    ];

    let initial_capital = 100000.0;
    let equity_curve = simulate_strategy_equity_curve(&btc_prices, initial_capital);

    println!("Кривая доходности:");
    for (i, &equity) in equity_curve.iter().enumerate() {
        println!("  День {}: ${:.2}", i, equity);
    }

    let drawdown_info = analyze_drawdown(&equity_curve);
    println!("\nПоказатели стратегии:");
    println!("  Начальный капитал: ${:.2}", initial_capital);
    println!("  Конечный капитал: ${:.2}", equity_curve.last().unwrap());
    println!("  Общая доходность: {:.2}%",
        (equity_curve.last().unwrap() / initial_capital - 1.0) * 100.0);
    println!("  Максимальная просадка: {:.2}%", drawdown_info.max_drawdown);
    println!("  Текущая просадка: {:.2}%", drawdown_info.current_drawdown);
}
```

## Несколько периодов просадки

Стратегия может иметь несколько периодов просадки. Давайте отследим их все:

```rust
#[derive(Debug, Clone)]
struct DrawdownPeriod {
    peak_value: f64,
    trough_value: f64,
    peak_index: usize,
    trough_index: usize,
    drawdown_pct: f64,
    recovery_index: Option<usize>, // Когда (если) восстановилась
}

fn find_all_drawdown_periods(equity_curve: &[f64]) -> Vec<DrawdownPeriod> {
    if equity_curve.is_empty() {
        return vec![];
    }

    let mut periods = Vec::new();
    let mut peak = equity_curve[0];
    let mut peak_index = 0;
    let mut in_drawdown = false;
    let mut trough = equity_curve[0];
    let mut trough_index = 0;

    for (i, &value) in equity_curve.iter().enumerate() {
        if value >= peak {
            // Новый пик или восстановление
            if in_drawdown {
                // Конец периода просадки - мы восстановились
                periods.last_mut().unwrap().recovery_index = Some(i);
                in_drawdown = false;
            }
            peak = value;
            peak_index = i;
            trough = value;
            trough_index = i;
        } else {
            // Мы ниже пика
            if !in_drawdown {
                // Начало новой просадки
                in_drawdown = true;
                trough = value;
                trough_index = i;

                periods.push(DrawdownPeriod {
                    peak_value: peak,
                    trough_value: value,
                    peak_index,
                    trough_index: i,
                    drawdown_pct: (peak - value) / peak * 100.0,
                    recovery_index: None,
                });
            } else if value < trough {
                // Новый минимум в текущей просадке
                trough = value;
                trough_index = i;

                // Обновляем текущий период
                let last = periods.last_mut().unwrap();
                last.trough_value = value;
                last.trough_index = i;
                last.drawdown_pct = (peak - value) / peak * 100.0;
            }
        }
    }

    periods
}

fn main() {
    let equity = vec![
        10000.0, 12000.0, 15000.0, 13000.0, 12000.0, // Первая просадка
        14000.0, 16000.0, 18000.0, 17000.0, 16000.0, // Вторая просадка
        15000.0, 17000.0, 19000.0, 20000.0,          // Восстановление и рост
    ];

    let periods = find_all_drawdown_periods(&equity);

    println!("Найдено {} периодов просадки:\n", periods.len());

    for (i, period) in periods.iter().enumerate() {
        println!("Просадка #{}", i + 1);
        println!("  Пик: ${:.2} (индекс {})", period.peak_value, period.peak_index);
        println!("  Низшая точка: ${:.2} (индекс {})", period.trough_value, period.trough_index);
        println!("  Просадка: {:.2}%", period.drawdown_pct);

        if let Some(recovery_idx) = period.recovery_index {
            let duration = recovery_idx - period.peak_index;
            println!("  Восстановилась на индексе {} (длительность: {} периодов)", recovery_idx, duration);
        } else {
            println!("  Ещё не восстановилась!");
        }
        println!();
    }

    // Вычисляем среднюю просадку
    let avg_drawdown: f64 = periods.iter()
        .map(|p| p.drawdown_pct)
        .sum::<f64>() / periods.len() as f64;

    println!("Средняя просадка: {:.2}%", avg_drawdown);

    let max_drawdown = periods.iter()
        .map(|p| p.drawdown_pct)
        .fold(0.0_f64, f64::max);

    println!("Максимальная просадка: {:.2}%", max_drawdown);
}
```

## Полный модуль бэктестинга с просадкой

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BacktestResults {
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub entry_time: usize,
    pub exit_time: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub position_size: f64,
    pub profit: f64,
    pub profit_pct: f64,
}

pub struct Backtester {
    initial_capital: f64,
    position_size_pct: f64, // Процент капитала на сделку
}

impl Backtester {
    pub fn new(initial_capital: f64, position_size_pct: f64) -> Self {
        Self {
            initial_capital,
            position_size_pct,
        }
    }

    pub fn run(&self, prices: &[f64], signals: &[i32]) -> BacktestResults {
        // signals: 1 = покупка, -1 = продажа, 0 = удержание
        let mut capital = self.initial_capital;
        let mut equity_curve = vec![capital];
        let mut trades = Vec::new();

        let mut in_position = false;
        let mut entry_price = 0.0;
        let mut entry_time = 0;
        let mut position_size = 0.0;

        for i in 0..prices.len() {
            if signals[i] == 1 && !in_position {
                // Входим в длинную позицию
                entry_price = prices[i];
                entry_time = i;
                position_size = (capital * self.position_size_pct) / prices[i];
                in_position = true;
            } else if signals[i] == -1 && in_position {
                // Выходим из позиции
                let exit_price = prices[i];
                let profit = (exit_price - entry_price) * position_size;
                let profit_pct = (exit_price / entry_price - 1.0) * 100.0;

                capital += profit;

                trades.push(Trade {
                    entry_time,
                    exit_time: i,
                    entry_price,
                    exit_price,
                    position_size,
                    profit,
                    profit_pct,
                });

                in_position = false;
            }

            equity_curve.push(capital);
        }

        // Вычисляем метрики
        let total_return = (capital / self.initial_capital - 1.0) * 100.0;
        let max_drawdown = calculate_max_drawdown(&equity_curve);

        let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = if trades.is_empty() {
            0.0
        } else {
            winning_trades as f64 / trades.len() as f64 * 100.0
        };

        let gross_profit: f64 = trades.iter()
            .filter(|t| t.profit > 0.0)
            .map(|t| t.profit)
            .sum();

        let gross_loss: f64 = trades.iter()
            .filter(|t| t.profit < 0.0)
            .map(|t| t.profit.abs())
            .sum();

        let profit_factor = if gross_loss == 0.0 {
            f64::INFINITY
        } else {
            gross_profit / gross_loss
        };

        // Простая аппроксимация коэффициента Шарпа (предполагая дневную доходность)
        let returns: Vec<f64> = equity_curve.windows(2)
            .map(|w| (w[1] / w[0] - 1.0))
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        let sharpe_ratio = if std_dev == 0.0 {
            0.0
        } else {
            mean_return / std_dev * (252.0_f64).sqrt() // Аннуализированный
        };

        BacktestResults {
            trades,
            equity_curve,
            total_return,
            max_drawdown,
            sharpe_ratio,
            win_rate,
            profit_factor,
        }
    }
}

fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
    if equity_curve.is_empty() {
        return 0.0;
    }

    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        if value > peak {
            peak = value;
        }
        let drawdown = (peak - value) / peak * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    max_drawdown
}

fn main() {
    // Пример: простая стратегия пересечения скользящих средних
    let prices = vec![
        100.0, 102.0, 105.0, 103.0, 101.0, 104.0, 108.0,
        110.0, 107.0, 105.0, 103.0, 106.0, 109.0, 112.0,
    ];

    // Генерируем сигналы (упрощённо)
    let signals = vec![
        0, 1, 0, 0, -1, 1, 0,
        0, 0, -1, 0, 1, 0, 0,
    ];

    let backtester = Backtester::new(10000.0, 0.95);
    let results = backtester.run(&prices, &signals);

    println!("=== Результаты бэктеста ===");
    println!("Всего сделок: {}", results.trades.len());
    println!("Процент выигрышей: {:.2}%", results.win_rate);
    println!("Общая доходность: {:.2}%", results.total_return);
    println!("Максимальная просадка: {:.2}%", results.max_drawdown);
    println!("Профит-фактор: {:.2}", results.profit_factor);
    println!("Коэффициент Шарпа: {:.2}", results.sharpe_ratio);

    println!("\n=== Отдельные сделки ===");
    for (i, trade) in results.trades.iter().enumerate() {
        println!("Сделка #{}: Прибыль: ${:.2} ({:.2}%)",
            i + 1, trade.profit, trade.profit_pct);
    }

    println!("\n=== Кривая доходности ===");
    for (i, &equity) in results.equity_curve.iter().enumerate() {
        println!("Период {}: ${:.2}", i, equity);
    }
}
```

## Почему важна максимальная просадка

| Метрика | Что она показывает |
|---------|-------------------|
| **Макс. просадка** | Худший возможный убыток, который вы испытаете |
| **Фактор восстановления** | Общая доходность / Макс. просадка (чем выше, тем лучше) |
| **Коэффициент Кальмара** | Годовая доходность / Макс. просадка (доходность с учётом риска) |
| **Процент выигрышей** | Процент прибыльных сделок |
| **Профит-фактор** | Валовая прибыль / Валовый убыток |

**Практическое правило:**
- MDD < 10%: Консервативный, низкий риск
- MDD 10-20%: Умеренный риск
- MDD 20-30%: Высокий риск
- MDD > 30%: Очень высокий риск, требует сильной психологии

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Максимальная просадка | Наибольшее падение от пика до низшей точки |
| Пик | Наивысшая стоимость портфеля до падения |
| Низшая точка | Самая низкая точка после пика |
| Восстановление | Возврат к предыдущему уровню пика |
| Период просадки | Время от пика до восстановления |
| Период под водой | Время, проведённое ниже предыдущего пика |

## Домашнее задание

1. **Вычисление MDD**: Напишите функцию, которая принимает кривую доходности и возвращает:
   - Процент максимальной просадки
   - Значение и индекс пика
   - Значение и индекс низшей точки
   - Восстановился ли портфель

2. **Длительность просадки**: Расширьте калькулятор MDD для отслеживания:
   - Сколько периодов длилась максимальная просадка
   - Средняя длительность просадки по всем периодам просадки
   - Самый длинный период под водой (время ниже предыдущего пика)

3. **Набор метрик риска**: Создайте структуру `RiskMetrics` с методами для вычисления:
   - Максимальной просадки
   - Коэффициента Кальмара (Годовая доходность / MDD)
   - Фактора восстановления (Общая доходность / MDD)
   - Индекса Ulcer (среднеквадратичное значение просадок)

4. **Визуализация просадки**: Напишите функцию, которая выводит простой текстовый график, показывающий:
   - Кривую доходности
   - Уровни пиков
   - Выделенные периоды просадки
   - Статус текущей просадки

   Пример вывода:
   ```
   Капитал: $15000 ████████ ПИК
   Капитал: $14000 ███████░ -6.7%
   Капитал: $12000 ██████░░ -20.0% ← МАКС ПРОСАДКА
   Капитал: $14500 ███████░ Восстановление
   ```

## Навигация

[← Предыдущий день](../286-metrics-profit-factor/ru.md) | [Следующий день →](../288-report-generation/ru.md)
