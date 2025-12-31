# День 288: Генерация отчётов

## Аналогия из трейдинга

Представьте, что вы торговали месяц и хотите оценить свою работу. Вам нужен полный отчёт с информацией:
- Общая прибыль/убыток
- Процент прибыльных сделок (win rate)
- Максимальная просадка (maximum drawdown)
- Коэффициент Шарпа (risk-adjusted returns)
- Распределение сделок по инструментам
- Разбивка P&L по дням

Без правильного отчёта вы летите вслепую — данные есть, но разобраться в них невозможно. **Генерация отчётов** преобразует сырые торговые данные в понятные показатели, которые помогают понять, что сработало, что нет, и как улучшить стратегию.

В бэктестинге генерация отчёта — это финальный шаг, где вы анализируете исторические данные о сделках, чтобы оценить производительность стратегии перед тем, как рисковать реальными деньгами.

## Что такое генерация отчётов?

Генерация отчётов — это процесс:
1. **Сбора** сырых данных (сделки, цены, позиции)
2. **Расчёта** метрик (доходность, просадка, коэффициенты)
3. **Форматирования** результатов для удобного чтения
4. **Экспорта** в различные форматы (текст, JSON, CSV, HTML)

В Rust мы используем структуры для организации данных, трейты для форматирования и различные библиотеки для экспорта в разные форматы.

## Базовая структура отчёта

```rust
use std::fmt;

#[derive(Debug)]
struct TradeReport {
    symbol: String,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    total_pnl: f64,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
}

impl TradeReport {
    fn new(symbol: String, trades: &[Trade]) -> Self {
        let total_trades = trades.len() as u32;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl = 0.0;
        let mut total_wins = 0.0;
        let mut total_losses = 0.0;

        for trade in trades {
            total_pnl += trade.pnl;
            if trade.pnl > 0.0 {
                winning_trades += 1;
                total_wins += trade.pnl;
            } else {
                losing_trades += 1;
                total_losses += trade.pnl.abs();
            }
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if winning_trades > 0 {
            total_wins / winning_trades as f64
        } else {
            0.0
        };

        let avg_loss = if losing_trades > 0 {
            total_losses / losing_trades as f64
        } else {
            0.0
        };

        TradeReport {
            symbol,
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            win_rate,
            avg_win,
            avg_loss,
        }
    }
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

impl fmt::Display for TradeReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "=== Отчёт по сделкам: {} ===\n\
             Всего сделок: {}\n\
             Прибыльных сделок: {}\n\
             Убыточных сделок: {}\n\
             Общий P&L: ${:.2}\n\
             Win Rate: {:.2}%\n\
             Средняя прибыль: ${:.2}\n\
             Средний убыток: ${:.2}\n\
             Profit Factor: {:.2}",
            self.symbol,
            self.total_trades,
            self.winning_trades,
            self.losing_trades,
            self.total_pnl,
            self.win_rate,
            self.avg_win,
            self.avg_loss,
            if self.avg_loss > 0.0 {
                self.avg_win * self.winning_trades as f64 / (self.avg_loss * self.losing_trades as f64)
            } else {
                0.0
            }
        )
    }
}

fn main() {
    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 40000.0,
            exit_price: 42000.0,
            quantity: 1.0,
            pnl: 2000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 42000.0,
            exit_price: 41000.0,
            quantity: 1.0,
            pnl: -1000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 41000.0,
            exit_price: 43000.0,
            quantity: 1.0,
            pnl: 2000.0,
        },
        Trade {
            symbol: "BTC".to_string(),
            entry_price: 43000.0,
            exit_price: 44500.0,
            quantity: 1.0,
            pnl: 1500.0,
        },
    ];

    let report = TradeReport::new("BTC".to_string(), &trades);
    println!("{}", report);
}
```

Вывод:
```
=== Отчёт по сделкам: BTC ===
Всего сделок: 4
Прибыльных сделок: 3
Убыточных сделок: 1
Общий P&L: $4500.00
Win Rate: 75.00%
Средняя прибыль: $1833.33
Средний убыток: $1000.00
Profit Factor: 5.50
```

## Продвинутые метрики: анализ просадок

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct DailyPnL {
    date: String,
    pnl: f64,
    cumulative_pnl: f64,
}

#[derive(Debug)]
struct DrawdownReport {
    max_drawdown: f64,
    max_drawdown_percent: f64,
    drawdown_periods: Vec<DrawdownPeriod>,
    current_drawdown: f64,
}

#[derive(Debug)]
struct DrawdownPeriod {
    start_date: String,
    end_date: String,
    peak_value: f64,
    trough_value: f64,
    drawdown: f64,
    drawdown_percent: f64,
}

impl DrawdownReport {
    fn calculate(daily_pnls: &[DailyPnL]) -> Self {
        let mut max_drawdown = 0.0;
        let mut max_drawdown_percent = 0.0;
        let mut peak = 0.0;
        let mut drawdown_periods = Vec::new();
        let mut in_drawdown = false;
        let mut drawdown_start = String::new();
        let mut peak_date = String::new();

        for pnl in daily_pnls {
            if pnl.cumulative_pnl > peak {
                // Новый пик — завершаем предыдущую просадку, если была
                if in_drawdown {
                    in_drawdown = false;
                }
                peak = pnl.cumulative_pnl;
                peak_date = pnl.date.clone();
            }

            let current_drawdown = peak - pnl.cumulative_pnl;
            let current_drawdown_percent = if peak != 0.0 {
                (current_drawdown / peak) * 100.0
            } else {
                0.0
            };

            if current_drawdown > 0.0 && !in_drawdown {
                in_drawdown = true;
                drawdown_start = peak_date.clone();
            }

            if current_drawdown > max_drawdown {
                max_drawdown = current_drawdown;
                max_drawdown_percent = current_drawdown_percent;
            }

            // Фиксируем период просадки при восстановлении
            if in_drawdown && current_drawdown == 0.0 {
                drawdown_periods.push(DrawdownPeriod {
                    start_date: drawdown_start.clone(),
                    end_date: pnl.date.clone(),
                    peak_value: peak,
                    trough_value: peak - max_drawdown,
                    drawdown: max_drawdown,
                    drawdown_percent: max_drawdown_percent,
                });
                in_drawdown = false;
            }
        }

        let current_drawdown = if let Some(last) = daily_pnls.last() {
            peak - last.cumulative_pnl
        } else {
            0.0
        };

        DrawdownReport {
            max_drawdown,
            max_drawdown_percent,
            drawdown_periods,
            current_drawdown,
        }
    }

    fn display(&self) {
        println!("=== Анализ просадок ===");
        println!("Максимальная просадка: ${:.2}", self.max_drawdown);
        println!("Максимальная просадка %: {:.2}%", self.max_drawdown_percent);
        println!("Текущая просадка: ${:.2}", self.current_drawdown);
        println!("\nПериодов просадок: {}", self.drawdown_periods.len());

        for (i, period) in self.drawdown_periods.iter().enumerate() {
            println!("\nПериод {}", i + 1);
            println!("  Начало: {}", period.start_date);
            println!("  Конец: {}", period.end_date);
            println!("  Пик: ${:.2}", period.peak_value);
            println!("  Дно: ${:.2}", period.trough_value);
            println!("  Просадка: ${:.2} ({:.2}%)", period.drawdown, period.drawdown_percent);
        }
    }
}

fn main() {
    let daily_pnls = vec![
        DailyPnL { date: "2024-01-01".to_string(), pnl: 1000.0, cumulative_pnl: 1000.0 },
        DailyPnL { date: "2024-01-02".to_string(), pnl: 500.0, cumulative_pnl: 1500.0 },
        DailyPnL { date: "2024-01-03".to_string(), pnl: -800.0, cumulative_pnl: 700.0 },
        DailyPnL { date: "2024-01-04".to_string(), pnl: -300.0, cumulative_pnl: 400.0 },
        DailyPnL { date: "2024-01-05".to_string(), pnl: 1200.0, cumulative_pnl: 1600.0 },
        DailyPnL { date: "2024-01-06".to_string(), pnl: 300.0, cumulative_pnl: 1900.0 },
        DailyPnL { date: "2024-01-07".to_string(), pnl: -500.0, cumulative_pnl: 1400.0 },
    ];

    let drawdown = DrawdownReport::calculate(&daily_pnls);
    drawdown.display();
}
```

## Экспорт в JSON

```rust
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct BacktestReport {
    strategy_name: String,
    start_date: String,
    end_date: String,
    initial_capital: f64,
    final_capital: f64,
    total_return: f64,
    total_return_percent: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
}

impl BacktestReport {
    fn new(
        strategy_name: String,
        start_date: String,
        end_date: String,
        initial_capital: f64,
        final_capital: f64,
        sharpe_ratio: f64,
        max_drawdown: f64,
        win_rate: f64,
        total_trades: u32,
        winning_trades: u32,
        losing_trades: u32,
    ) -> Self {
        let total_return = final_capital - initial_capital;
        let total_return_percent = (total_return / initial_capital) * 100.0;

        BacktestReport {
            strategy_name,
            start_date,
            end_date,
            initial_capital,
            final_capital,
            total_return,
            total_return_percent,
            sharpe_ratio,
            max_drawdown,
            win_rate,
            total_trades,
            winning_trades,
            losing_trades,
        }
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = self.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;
        std::fs::write(filename, json)
    }
}

fn main() -> std::io::Result<()> {
    let report = BacktestReport::new(
        "Moving Average Crossover".to_string(),
        "2024-01-01".to_string(),
        "2024-12-31".to_string(),
        100_000.0,
        145_000.0,
        1.8,
        -12_000.0,
        65.5,
        150,
        98,
        52,
    );

    println!("{}", report.to_json().unwrap());

    // Сохраняем в файл
    report.save_to_file("backtest_report.json")?;
    println!("\nОтчёт сохранён в backtest_report.json");

    Ok(())
}
```

## Экспорт в CSV для журнала сделок

```rust
use std::fs::File;
use std::io::{Write, Result};

#[derive(Debug)]
struct TradeLog {
    timestamp: String,
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
    commission: f64,
}

struct CsvReportWriter {
    filename: String,
}

impl CsvReportWriter {
    fn new(filename: String) -> Self {
        CsvReportWriter { filename }
    }

    fn write_trades(&self, trades: &[TradeLog]) -> Result<()> {
        let mut file = File::create(&self.filename)?;

        // Пишем заголовок
        writeln!(
            file,
            "Timestamp,Symbol,Side,Quantity,Entry Price,Exit Price,P&L,Commission"
        )?;

        // Пишем данные
        for trade in trades {
            writeln!(
                file,
                "{},{},{},{},{},{},{:.2},{:.2}",
                trade.timestamp,
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.entry_price,
                trade.exit_price,
                trade.pnl,
                trade.commission
            )?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let trades = vec![
        TradeLog {
            timestamp: "2024-01-01 09:30:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 1.0,
            entry_price: 40000.0,
            exit_price: 42000.0,
            pnl: 2000.0,
            commission: 40.0,
        },
        TradeLog {
            timestamp: "2024-01-02 14:15:00".to_string(),
            symbol: "ETH".to_string(),
            side: "SHORT".to_string(),
            quantity: 10.0,
            entry_price: 2200.0,
            exit_price: 2100.0,
            pnl: 1000.0,
            commission: 22.0,
        },
        TradeLog {
            timestamp: "2024-01-03 11:45:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 0.5,
            entry_price: 41000.0,
            exit_price: 40000.0,
            pnl: -500.0,
            commission: 20.5,
        },
    ];

    let writer = CsvReportWriter::new("trade_log.csv".to_string());
    writer.write_trades(&trades)?;

    println!("Журнал сделок экспортирован в trade_log.csv");
    Ok(())
}
```

## Генерация HTML-отчётов

```rust
use std::fs::File;
use std::io::{Write, Result};

struct HtmlReportGenerator {
    title: String,
}

impl HtmlReportGenerator {
    fn new(title: String) -> Self {
        HtmlReportGenerator { title }
    }

    fn generate(&self, report: &BacktestSummary, filename: &str) -> Result<()> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #4CAF50;
            padding-bottom: 10px;
        }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 10px;
            border-bottom: 1px solid #eee;
        }}
        .metric-label {{
            font-weight: bold;
            color: #666;
        }}
        .metric-value {{
            color: #333;
        }}
        .positive {{
            color: #4CAF50;
        }}
        .negative {{
            color: #f44336;
        }}
        .section {{
            margin-top: 30px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>

        <div class="section">
            <h2>Обзор результатов</h2>
            <div class="metric">
                <span class="metric-label">Начальный капитал:</span>
                <span class="metric-value">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Конечный капитал:</span>
                <span class="metric-value">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Общий доход:</span>
                <span class="metric-value {}">${:.2} ({:.2}%)</span>
            </div>
        </div>

        <div class="section">
            <h2>Метрики риска</h2>
            <div class="metric">
                <span class="metric-label">Максимальная просадка:</span>
                <span class="metric-value negative">${:.2}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Коэффициент Шарпа:</span>
                <span class="metric-value">{:.2}</span>
            </div>
        </div>

        <div class="section">
            <h2>Статистика сделок</h2>
            <div class="metric">
                <span class="metric-label">Всего сделок:</span>
                <span class="metric-value">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Прибыльных сделок:</span>
                <span class="metric-value positive">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Убыточных сделок:</span>
                <span class="metric-value negative">{}</span>
            </div>
            <div class="metric">
                <span class="metric-label">Win Rate:</span>
                <span class="metric-value">{:.2}%</span>
            </div>
        </div>
    </div>
</body>
</html>"#,
            self.title,
            report.strategy_name,
            report.initial_capital,
            report.final_capital,
            if report.total_return > 0.0 { "positive" } else { "negative" },
            report.total_return,
            report.total_return_percent,
            report.max_drawdown,
            report.sharpe_ratio,
            report.total_trades,
            report.winning_trades,
            report.losing_trades,
            report.win_rate,
        );

        let mut file = File::create(filename)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
struct BacktestSummary {
    strategy_name: String,
    initial_capital: f64,
    final_capital: f64,
    total_return: f64,
    total_return_percent: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    win_rate: f64,
}

fn main() -> Result<()> {
    let summary = BacktestSummary {
        strategy_name: "RSI Mean Reversion".to_string(),
        initial_capital: 100_000.0,
        final_capital: 135_500.0,
        total_return: 35_500.0,
        total_return_percent: 35.5,
        max_drawdown: -8_200.0,
        sharpe_ratio: 2.1,
        total_trades: 200,
        winning_trades: 135,
        losing_trades: 65,
        win_rate: 67.5,
    };

    let generator = HtmlReportGenerator::new("Отчёт по бэктестингу".to_string());
    generator.generate(&summary, "backtest_report.html")?;

    println!("HTML-отчёт создан: backtest_report.html");
    Ok(())
}
```

## Комплексная система отчётности

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Result};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    timestamp: String,
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
    commission: f64,
}

#[derive(Debug)]
struct ComprehensiveReport {
    trades: Vec<Trade>,
    initial_capital: f64,
}

impl ComprehensiveReport {
    fn new(trades: Vec<Trade>, initial_capital: f64) -> Self {
        ComprehensiveReport {
            trades,
            initial_capital,
        }
    }

    fn calculate_metrics(&self) -> ReportMetrics {
        let total_trades = self.trades.len() as u32;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl = 0.0;
        let mut total_commission = 0.0;

        let mut symbol_performance: HashMap<String, f64> = HashMap::new();

        for trade in &self.trades {
            total_pnl += trade.pnl;
            total_commission += trade.commission;

            if trade.pnl > 0.0 {
                winning_trades += 1;
            } else if trade.pnl < 0.0 {
                losing_trades += 1;
            }

            *symbol_performance.entry(trade.symbol.clone()).or_insert(0.0) += trade.pnl;
        }

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let final_capital = self.initial_capital + total_pnl - total_commission;
        let total_return_percent = ((final_capital - self.initial_capital) / self.initial_capital) * 100.0;

        ReportMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            total_commission,
            win_rate,
            final_capital,
            total_return_percent,
            symbol_performance,
        }
    }

    fn generate_text_report(&self, filename: &str) -> Result<()> {
        let metrics = self.calculate_metrics();
        let mut file = File::create(filename)?;

        writeln!(file, "=======================")?;
        writeln!(file, "=== ОТЧЁТ ПО БЭКТЕСТУ ===")?;
        writeln!(file, "=======================")?;
        writeln!(file)?;

        writeln!(file, "ОБЗОР КАПИТАЛА")?;
        writeln!(file, "Начальный капитал: ${:.2}", self.initial_capital)?;
        writeln!(file, "Конечный капитал: ${:.2}", metrics.final_capital)?;
        writeln!(file, "Общий доход: {:.2}%", metrics.total_return_percent)?;
        writeln!(file)?;

        writeln!(file, "СТАТИСТИКА СДЕЛОК")?;
        writeln!(file, "Всего сделок: {}", metrics.total_trades)?;
        writeln!(file, "Прибыльных сделок: {}", metrics.winning_trades)?;
        writeln!(file, "Убыточных сделок: {}", metrics.losing_trades)?;
        writeln!(file, "Win Rate: {:.2}%", metrics.win_rate)?;
        writeln!(file)?;

        writeln!(file, "РАЗБИВКА P&L")?;
        writeln!(file, "Валовый P&L: ${:.2}", metrics.total_pnl)?;
        writeln!(file, "Всего комиссий: ${:.2}", metrics.total_commission)?;
        writeln!(file, "Чистый P&L: ${:.2}", metrics.total_pnl - metrics.total_commission)?;
        writeln!(file)?;

        writeln!(file, "РЕЗУЛЬТАТЫ ПО ИНСТРУМЕНТАМ")?;
        for (symbol, pnl) in &metrics.symbol_performance {
            writeln!(file, "{}: ${:.2}", symbol, pnl)?;
        }

        Ok(())
    }

    fn print_summary(&self) {
        let metrics = self.calculate_metrics();
        println!("\n=== СВОДКА ПО БЭКТЕСТУ ===");
        println!("Сделок: {}", metrics.total_trades);
        println!("Win Rate: {:.2}%", metrics.win_rate);
        println!("Общий доход: {:.2}%", metrics.total_return_percent);
        println!("Конечный капитал: ${:.2}", metrics.final_capital);
    }
}

#[derive(Debug)]
struct ReportMetrics {
    total_trades: u32,
    winning_trades: u32,
    losing_trades: u32,
    total_pnl: f64,
    total_commission: f64,
    win_rate: f64,
    final_capital: f64,
    total_return_percent: f64,
    symbol_performance: HashMap<String, f64>,
}

fn main() -> Result<()> {
    let trades = vec![
        Trade {
            id: 1,
            timestamp: "2024-01-01 09:30:00".to_string(),
            symbol: "BTC".to_string(),
            side: "LONG".to_string(),
            quantity: 1.0,
            entry_price: 40000.0,
            exit_price: 42000.0,
            pnl: 2000.0,
            commission: 40.0,
        },
        Trade {
            id: 2,
            timestamp: "2024-01-02 14:15:00".to_string(),
            symbol: "ETH".to_string(),
            side: "LONG".to_string(),
            quantity: 10.0,
            entry_price: 2200.0,
            exit_price: 2300.0,
            pnl: 1000.0,
            commission: 22.0,
        },
        Trade {
            id: 3,
            timestamp: "2024-01-03 11:45:00".to_string(),
            symbol: "BTC".to_string(),
            side: "SHORT".to_string(),
            quantity: 0.5,
            entry_price: 41000.0,
            exit_price: 42000.0,
            pnl: -500.0,
            commission: 20.5,
        },
    ];

    let report = ComprehensiveReport::new(trades, 100_000.0);

    report.print_summary();
    report.generate_text_report("backtest_summary.txt")?;

    println!("\nПодробный отчёт сохранён в backtest_summary.txt");

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|---------|-------------|
| Структура отчёта | Организация данных через структуры для чёткого представления |
| Трейт Display | Реализация `fmt::Display` для кастомного форматирования |
| Расчёт метрик | Вычисление win rate, просадки, доходности из сырых данных |
| Экспорт в JSON | Использование `serde` для сериализации структурированных данных |
| Экспорт в CSV | Запись табличных данных для анализа в электронных таблицах |
| Генерация HTML | Создание визуальных отчётов со встроенными стилями |
| Работа с файлами | Сохранение отчётов в различные форматы файлов |

## Домашнее задание

1. **Отчёт по месячной производительности**: Создайте отчёт, разбивающий P&L по месяцам. Рассчитайте:
   - Месячную доходность
   - Лучший месяц
   - Худший месяц
   - Среднюю месячную доходность
   - Волатильность (стандартное отклонение месячных доходностей)

2. **Анализ по инструментам**: Сгенерируйте отчёт, показывающий производительность по торговым инструментам:
   - Общее количество сделок по каждому инструменту
   - Win rate по каждому инструменту
   - Средний P&L по каждому инструменту
   - Самый доходный инструмент
   - Самый убыточный инструмент

3. **Анализ длительности сделок**: Добавьте отслеживание длительности сделок и создайте отчёт, показывающий:
   - Среднюю длительность сделки
   - Самую короткую сделку
   - Самую длинную сделку
   - Связь между длительностью и прибыльностью

4. **Мультиформатный генератор отчётов**: Создайте трейт `ReportGenerator`, который может экспортировать в несколько форматов:
   ```rust
   trait ReportGenerator {
       fn to_json(&self) -> String;
       fn to_csv(&self) -> String;
       fn to_html(&self) -> String;
       fn to_text(&self) -> String;
   }
   ```
   Реализуйте его для структуры `BacktestReport`, включающей все основные метрики.

## Навигация

[← Предыдущий день](../287-performance-metrics/ru.md) | [Следующий день →](../289-visualization/ru.md)
