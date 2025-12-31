# День 283: Метрики: общая прибыль

## Аналогия из трейдинга

Представь, что ты управляешь торговым роботом, который за месяц совершил 100 сделок. Какой первый вопрос ты задашь? Конечно же: **"Сколько заработали?"** Общая прибыль (Total Profit) — это самая базовая и интуитивно понятная метрика в бэктестинге. Она показывает абсолютный финансовый результат твоей стратегии: сумму всех прибылей минус сумму всех убытков.

В реальной торговле это эквивалент вопроса: "Если бы я начал с $10,000, сколько бы у меня было в конце?" Total Profit отвечает именно на этот вопрос.

## Что такое Total Profit?

**Total Profit (Общая прибыль)** — это сумма результатов всех сделок за период тестирования:

```
Total Profit = Σ (PnL каждой сделки)
```

Где PnL (Profit and Loss) каждой сделки рассчитывается как:

```
PnL = (Цена выхода - Цена входа) × Количество - Комиссии
```

## Базовый расчёт Total Profit

```rust
fn main() {
    // Результаты сделок (PnL в долларах)
    let trades = vec![
        150.0,   // Прибыльная сделка
        -80.0,   // Убыточная сделка
        200.0,   // Прибыльная сделка
        -50.0,   // Убыточная сделка
        120.0,   // Прибыльная сделка
        -30.0,   // Убыточная сделка
        180.0,   // Прибыльная сделка
        -100.0,  // Убыточная сделка
    ];

    let total_profit = calculate_total_profit(&trades);

    println!("Всего сделок: {}", trades.len());
    println!("Общая прибыль: ${:.2}", total_profit);
}

fn calculate_total_profit(trades: &[f64]) -> f64 {
    trades.iter().sum()
}
```

## Детальный расчёт с разделением на прибыли и убытки

```rust
fn main() {
    let trades = vec![
        150.0, -80.0, 200.0, -50.0, 120.0, -30.0, 180.0, -100.0
    ];

    let analysis = analyze_profits(&trades);
    print_profit_analysis(&analysis);
}

struct ProfitAnalysis {
    total_profit: f64,
    gross_profit: f64,      // Сумма всех прибылей
    gross_loss: f64,        // Сумма всех убытков (отрицательное число)
    profit_factor: f64,     // Gross Profit / |Gross Loss|
    winning_trades: usize,
    losing_trades: usize,
}

fn analyze_profits(trades: &[f64]) -> ProfitAnalysis {
    let mut gross_profit = 0.0;
    let mut gross_loss = 0.0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;

    for &pnl in trades {
        if pnl > 0.0 {
            gross_profit += pnl;
            winning_trades += 1;
        } else if pnl < 0.0 {
            gross_loss += pnl;
            losing_trades += 1;
        }
    }

    let total_profit = gross_profit + gross_loss;

    // Profit Factor = Gross Profit / |Gross Loss|
    let profit_factor = if gross_loss != 0.0 {
        gross_profit / gross_loss.abs()
    } else {
        f64::INFINITY
    };

    ProfitAnalysis {
        total_profit,
        gross_profit,
        gross_loss,
        profit_factor,
        winning_trades,
        losing_trades,
    }
}

fn print_profit_analysis(analysis: &ProfitAnalysis) {
    println!("╔═══════════════════════════════════════╗");
    println!("║         АНАЛИЗ ПРИБЫЛЬНОСТИ           ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Общая прибыль:     ${:>16.2} ║", analysis.total_profit);
    println!("║ Валовая прибыль:   ${:>16.2} ║", analysis.gross_profit);
    println!("║ Валовой убыток:    ${:>16.2} ║", analysis.gross_loss);
    println!("║ Profit Factor:      {:>16.2} ║", analysis.profit_factor);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Прибыльных сделок: {:>17} ║", analysis.winning_trades);
    println!("║ Убыточных сделок:  {:>17} ║", analysis.losing_trades);
    println!("╚═══════════════════════════════════════╝");
}
```

## Структура сделки и расчёт PnL

```rust
use std::fmt;

#[derive(Debug, Clone)]
enum TradeDirection {
    Long,   // Покупка
    Short,  // Продажа
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    direction: TradeDirection,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    entry_fee: f64,
    exit_fee: f64,
}

impl Trade {
    fn new(
        symbol: &str,
        direction: TradeDirection,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        fee_percent: f64,
    ) -> Self {
        let entry_value = entry_price * quantity;
        let exit_value = exit_price * quantity;
        let entry_fee = entry_value * (fee_percent / 100.0);
        let exit_fee = exit_value * (fee_percent / 100.0);

        Trade {
            symbol: symbol.to_string(),
            direction,
            entry_price,
            exit_price,
            quantity,
            entry_fee,
            exit_fee,
        }
    }

    fn calculate_pnl(&self) -> f64 {
        let price_diff = match self.direction {
            TradeDirection::Long => self.exit_price - self.entry_price,
            TradeDirection::Short => self.entry_price - self.exit_price,
        };

        let gross_pnl = price_diff * self.quantity;
        let total_fees = self.entry_fee + self.exit_fee;

        gross_pnl - total_fees
    }

    fn calculate_gross_pnl(&self) -> f64 {
        let price_diff = match self.direction {
            TradeDirection::Long => self.exit_price - self.entry_price,
            TradeDirection::Short => self.entry_price - self.exit_price,
        };
        price_diff * self.quantity
    }

    fn total_fees(&self) -> f64 {
        self.entry_fee + self.exit_fee
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = match self.direction {
            TradeDirection::Long => "LONG",
            TradeDirection::Short => "SHORT",
        };
        write!(
            f,
            "{} {} {:.4} @ {:.2} -> {:.2} | PnL: ${:.2}",
            self.symbol,
            direction,
            self.quantity,
            self.entry_price,
            self.exit_price,
            self.calculate_pnl()
        )
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", TradeDirection::Long, 42000.0, 43500.0, 0.5, 0.1),
        Trade::new("ETH", TradeDirection::Long, 2500.0, 2400.0, 2.0, 0.1),
        Trade::new("BTC", TradeDirection::Short, 43000.0, 42000.0, 0.3, 0.1),
        Trade::new("SOL", TradeDirection::Long, 95.0, 105.0, 10.0, 0.1),
        Trade::new("ETH", TradeDirection::Short, 2450.0, 2550.0, 1.5, 0.1),
    ];

    println!("=== Список сделок ===\n");
    for (i, trade) in trades.iter().enumerate() {
        println!("{}. {}", i + 1, trade);
    }

    let total_profit: f64 = trades.iter().map(|t| t.calculate_pnl()).sum();
    let total_gross: f64 = trades.iter().map(|t| t.calculate_gross_pnl()).sum();
    let total_fees: f64 = trades.iter().map(|t| t.total_fees()).sum();

    println!("\n=== Итоги ===");
    println!("Валовая прибыль: ${:.2}", total_gross);
    println!("Всего комиссий:  ${:.2}", total_fees);
    println!("Чистая прибыль:  ${:.2}", total_profit);
}
```

## Расчёт прибыли с учётом времени (Time-Weighted)

```rust
use std::time::Duration;

#[derive(Debug)]
struct TimedTrade {
    pnl: f64,
    duration_hours: f64,  // Длительность сделки в часах
}

fn main() {
    let trades = vec![
        TimedTrade { pnl: 150.0, duration_hours: 2.0 },
        TimedTrade { pnl: -80.0, duration_hours: 5.0 },
        TimedTrade { pnl: 200.0, duration_hours: 1.5 },
        TimedTrade { pnl: -50.0, duration_hours: 8.0 },
        TimedTrade { pnl: 300.0, duration_hours: 24.0 },
    ];

    let total_profit: f64 = trades.iter().map(|t| t.pnl).sum();
    let total_hours: f64 = trades.iter().map(|t| t.duration_hours).sum();
    let profit_per_hour = total_profit / total_hours;

    println!("Общая прибыль: ${:.2}", total_profit);
    println!("Общее время в сделках: {:.1} часов", total_hours);
    println!("Прибыль в час: ${:.2}/час", profit_per_hour);

    // Прибыль за день (24 часа)
    let projected_daily = profit_per_hour * 24.0;
    println!("Проекция на день: ${:.2}/день", projected_daily);
}
```

## Полноценный торговый журнал

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TradeRecord {
    id: u64,
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    fees: f64,
}

struct TradingJournal {
    trades: Vec<TradeRecord>,
    next_id: u64,
}

impl TradingJournal {
    fn new() -> Self {
        TradingJournal {
            trades: Vec::new(),
            next_id: 1,
        }
    }

    fn add_trade(
        &mut self,
        symbol: &str,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        fee_percent: f64,
    ) {
        let gross_pnl = (exit_price - entry_price) * quantity;
        let fees = (entry_price + exit_price) * quantity * (fee_percent / 100.0);
        let net_pnl = gross_pnl - fees;

        let record = TradeRecord {
            id: self.next_id,
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            pnl: net_pnl,
            fees,
        };

        self.trades.push(record);
        self.next_id += 1;
    }

    fn total_profit(&self) -> f64 {
        self.trades.iter().map(|t| t.pnl).sum()
    }

    fn total_fees(&self) -> f64 {
        self.trades.iter().map(|t| t.fees).sum()
    }

    fn gross_profit(&self) -> f64 {
        self.trades
            .iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .sum()
    }

    fn gross_loss(&self) -> f64 {
        self.trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl)
            .sum()
    }

    fn profit_by_symbol(&self) -> HashMap<String, f64> {
        let mut profits: HashMap<String, f64> = HashMap::new();

        for trade in &self.trades {
            *profits.entry(trade.symbol.clone()).or_insert(0.0) += trade.pnl;
        }

        profits
    }

    fn best_trade(&self) -> Option<&TradeRecord> {
        self.trades.iter().max_by(|a, b| {
            a.pnl.partial_cmp(&b.pnl).unwrap()
        })
    }

    fn worst_trade(&self) -> Option<&TradeRecord> {
        self.trades.iter().min_by(|a, b| {
            a.pnl.partial_cmp(&b.pnl).unwrap()
        })
    }

    fn average_profit(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        self.total_profit() / self.trades.len() as f64
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║           ТОРГОВЫЙ ЖУРНАЛ                 ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Всего сделок:        {:>20} ║", self.trades.len());
        println!("║ Общая прибыль:       ${:>18.2} ║", self.total_profit());
        println!("║ Валовая прибыль:     ${:>18.2} ║", self.gross_profit());
        println!("║ Валовой убыток:      ${:>18.2} ║", self.gross_loss());
        println!("║ Всего комиссий:      ${:>18.2} ║", self.total_fees());
        println!("║ Средняя прибыль:     ${:>18.2} ║", self.average_profit());
        println!("╠═══════════════════════════════════════════╣");

        if let Some(best) = self.best_trade() {
            println!("║ Лучшая сделка:       ${:>18.2} ║", best.pnl);
        }
        if let Some(worst) = self.worst_trade() {
            println!("║ Худшая сделка:       ${:>18.2} ║", worst.pnl);
        }

        println!("╠═══════════════════════════════════════════╣");
        println!("║ Прибыль по инструментам:                  ║");

        for (symbol, profit) in self.profit_by_symbol() {
            println!("║   {:6}:            ${:>18.2} ║", symbol, profit);
        }

        println!("╚═══════════════════════════════════════════╝");
    }
}

fn main() {
    let mut journal = TradingJournal::new();

    // Добавляем сделки
    journal.add_trade("BTC", 42000.0, 43500.0, 0.5, 0.1);
    journal.add_trade("ETH", 2500.0, 2400.0, 2.0, 0.1);
    journal.add_trade("BTC", 43000.0, 44000.0, 0.3, 0.1);
    journal.add_trade("SOL", 95.0, 105.0, 10.0, 0.1);
    journal.add_trade("ETH", 2450.0, 2300.0, 1.5, 0.1);
    journal.add_trade("BTC", 44500.0, 43800.0, 0.2, 0.1);
    journal.add_trade("SOL", 102.0, 98.0, 15.0, 0.1);
    journal.add_trade("BTC", 43500.0, 45000.0, 0.4, 0.1);

    journal.print_summary();
}
```

## Расчёт прибыли относительно начального капитала

```rust
struct BacktestResult {
    initial_capital: f64,
    final_capital: f64,
    trades: Vec<f64>,
}

impl BacktestResult {
    fn total_profit(&self) -> f64 {
        self.final_capital - self.initial_capital
    }

    fn total_return_percent(&self) -> f64 {
        (self.total_profit() / self.initial_capital) * 100.0
    }

    fn annualized_return(&self, days: u32) -> f64 {
        let total_return = self.total_return_percent() / 100.0;
        let years = days as f64 / 365.0;

        if years <= 0.0 {
            return 0.0;
        }

        // Годовая доходность = (1 + total_return)^(1/years) - 1
        ((1.0 + total_return).powf(1.0 / years) - 1.0) * 100.0
    }

    fn print_results(&self, days: u32) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         РЕЗУЛЬТАТЫ БЭКТЕСТА               ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Начальный капитал:   ${:>18.2} ║", self.initial_capital);
        println!("║ Конечный капитал:    ${:>18.2} ║", self.final_capital);
        println!("║ Общая прибыль:       ${:>18.2} ║", self.total_profit());
        println!("║ Доходность:           {:>17.2}% ║", self.total_return_percent());
        println!("║ Годовая доходность:   {:>17.2}% ║", self.annualized_return(days));
        println!("║ Период теста:         {:>14} дней ║", days);
        println!("╚═══════════════════════════════════════════╝");
    }
}

fn main() {
    // Симуляция бэктеста за 90 дней
    let trades = vec![
        500.0, -200.0, 800.0, -150.0, 600.0,
        -300.0, 450.0, -100.0, 700.0, -250.0,
        550.0, -180.0, 900.0, -400.0, 350.0,
    ];

    let initial_capital = 10000.0;
    let total_pnl: f64 = trades.iter().sum();
    let final_capital = initial_capital + total_pnl;

    let result = BacktestResult {
        initial_capital,
        final_capital,
        trades,
    };

    result.print_results(90);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Total Profit | Сумма PnL всех сделок |
| Gross Profit | Сумма только прибыльных сделок |
| Gross Loss | Сумма только убыточных сделок |
| Net Profit | Прибыль за вычетом комиссий |
| Profit Factor | Отношение валовой прибыли к валовому убытку |
| Total Return % | Процентная доходность относительно начального капитала |
| Annualized Return | Приведённая к году доходность |

## Практические задания

1. **Калькулятор PnL**: Напиши функцию, которая принимает вектор сделок (entry, exit, quantity) и возвращает общую прибыль с учётом комиссий.

2. **Анализ по периодам**: Реализуй функцию, которая разбивает сделки по месяцам и считает прибыль за каждый месяц.

3. **Equity Curve**: Создай функцию, которая принимает список PnL и начальный капитал, а возвращает вектор значений капитала после каждой сделки.

4. **Сравнение стратегий**: Напиши программу, которая сравнивает Total Profit двух разных стратегий и определяет лучшую.

## Домашнее задание

1. Реализуй структуру `StrategyMetrics` с полями:
   - `total_profit`
   - `gross_profit`
   - `gross_loss`
   - `profit_factor`
   - `average_win`
   - `average_loss`
   - `largest_win`
   - `largest_loss`

   И методом `from_trades(trades: &[f64]) -> Self`

2. Добавь в `TradingJournal` метод `rolling_profit(window: usize)`, который считает скользящую сумму прибыли за последние N сделок.

3. Реализуй функцию `compare_strategies(strategy_a: &[f64], strategy_b: &[f64])`, которая выводит сравнительную таблицу метрик.

4. Создай симулятор бэктеста, который:
   - Принимает начальный капитал и список сигналов (Buy/Sell)
   - Рассчитывает PnL каждой сделки
   - Отслеживает текущий баланс
   - Выводит итоговую статистику

## Навигация

[← Предыдущий день](../282-backtest-walk-forward-analysis/ru.md) | [Следующий день →](../284-metrics-profit-factor/ru.md)
