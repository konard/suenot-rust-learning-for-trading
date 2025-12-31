# День 303: Документирование результатов

## Аналогия из трейдинга

Представь трейдера, который протестировал десятки стратегий, получил сотни результатов, но не записал ни одного. Через месяц он не помнит:
- Какие параметры давали лучший результат?
- На каких инструментах стратегия работала?
- Какие были максимальные просадки?
- Почему одна стратегия была отклонена?

Это как вести дневник сделок, но забывать в него писать. Результаты тестов без документации — потерянное время и знания.

**Документирование результатов** в алготрейдинге — это:
- Систематическая запись всех бэктестов
- Сохранение параметров и метрик
- Анализ и сравнение стратегий
- База знаний для будущих решений

Как научный эксперимент: без записей невозможно воспроизвести результат или понять, что пошло не так.

## Зачем документировать результаты бэктестинга?

В профессиональном алготрейдинге документирование — это не опция, а обязательная практика:

| Причина | Зачем это нужно | Пример |
|---------|-----------------|--------|
| **Воспроизводимость** | Повторить успешный тест через месяц | Записали seed RNG и параметры |
| **Сравнение** | Выбрать лучшую из 10 стратегий | Таблица с Sharpe ratio всех тестов |
| **Аудит** | Объяснить регулятору или инвестору | Полный отчёт по каждой сделке |
| **Обучение** | Понять, почему стратегия провалилась | Логи показали look-ahead bias |
| **Версионирование** | Отследить изменения в коде стратегии | Git commit hash в каждом отчёте |

## Что документировать?

### 1. Метаданные теста

```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct BacktestMetadata {
    /// Уникальный ID теста
    test_id: String,
    /// Название стратегии
    strategy_name: String,
    /// Версия кода стратегии (git commit)
    code_version: String,
    /// Время запуска теста
    timestamp: DateTime<Utc>,
    /// Кто запустил тест
    author: String,
    /// Описание цели теста
    description: String,
}

impl BacktestMetadata {
    fn new(strategy_name: &str, description: &str) -> Self {
        Self {
            test_id: uuid::Uuid::new_v4().to_string(),
            strategy_name: strategy_name.to_string(),
            code_version: "abc123def".to_string(), // В реальности: git rev-parse HEAD
            timestamp: Utc::now(),
            author: "trading-bot".to_string(),
            description: description.to_string(),
        }
    }

    fn print(&self) {
        println!("=== Метаданные бэктеста ===");
        println!("ID: {}", self.test_id);
        println!("Стратегия: {}", self.strategy_name);
        println!("Версия кода: {}", self.code_version);
        println!("Время: {}", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Автор: {}", self.author);
        println!("Описание: {}", self.description);
    }
}

fn main() {
    let metadata = BacktestMetadata::new(
        "MA Crossover v2.1",
        "Тест с новыми параметрами стоп-лосса"
    );
    metadata.print();
}
```

### 2. Параметры стратегии

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
struct StrategyParameters {
    /// Период короткой скользящей средней
    ma_short_period: usize,
    /// Период длинной скользящей средней
    ma_long_period: usize,
    /// Стоп-лосс в процентах
    stop_loss_pct: f64,
    /// Тейк-профит в процентах
    take_profit_pct: f64,
    /// Максимальный размер позиции (% от капитала)
    max_position_size: f64,
}

impl StrategyParameters {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn print(&self) {
        println!("\n=== Параметры стратегии ===");
        println!("MA Short: {}", self.ma_short_period);
        println!("MA Long: {}", self.ma_long_period);
        println!("Stop Loss: {:.2}%", self.stop_loss_pct * 100.0);
        println!("Take Profit: {:.2}%", self.take_profit_pct * 100.0);
        println!("Max Position Size: {:.2}%", self.max_position_size * 100.0);
    }
}

fn main() {
    let params = StrategyParameters {
        ma_short_period: 10,
        ma_long_period: 50,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.05,
        max_position_size: 0.10,
    };

    params.print();
    println!("\nJSON representation:");
    println!("{}", params.to_json());
}
```

### 3. Метрики производительности

```rust
#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    /// Общая доходность
    total_return: f64,
    /// Annualized return
    annual_return: f64,
    /// Sharpe ratio
    sharpe_ratio: f64,
    /// Maximum drawdown
    max_drawdown: f64,
    /// Win rate (процент прибыльных сделок)
    win_rate: f64,
    /// Profit factor
    profit_factor: f64,
    /// Общее количество сделок
    total_trades: usize,
    /// Количество прибыльных сделок
    winning_trades: usize,
    /// Количество убыточных сделок
    losing_trades: usize,
    /// Средняя прибыль на сделку
    avg_profit_per_trade: f64,
    /// Максимальная прибыль
    max_profit: f64,
    /// Максимальный убыток
    max_loss: f64,
}

impl PerformanceMetrics {
    fn print(&self) {
        println!("\n=== Метрики производительности ===");
        println!("Общая доходность: {:.2}%", self.total_return * 100.0);
        println!("Годовая доходность: {:.2}%", self.annual_return * 100.0);
        println!("Sharpe Ratio: {:.2}", self.sharpe_ratio);
        println!("Max Drawdown: {:.2}%", self.max_drawdown * 100.0);
        println!("Win Rate: {:.2}%", self.win_rate * 100.0);
        println!("Profit Factor: {:.2}", self.profit_factor);
        println!("\n=== Статистика сделок ===");
        println!("Всего сделок: {}", self.total_trades);
        println!("Прибыльных: {}", self.winning_trades);
        println!("Убыточных: {}", self.losing_trades);
        println!("Средняя прибыль: {:.2}%", self.avg_profit_per_trade * 100.0);
        println!("Макс. прибыль: {:.2}%", self.max_profit * 100.0);
        println!("Макс. убыток: {:.2}%", self.max_loss * 100.0);
    }

    fn grade(&self) -> &str {
        if self.sharpe_ratio > 2.0 && self.max_drawdown.abs() < 0.15 {
            "Отлично ✅"
        } else if self.sharpe_ratio > 1.0 && self.max_drawdown.abs() < 0.25 {
            "Хорошо 👍"
        } else if self.sharpe_ratio > 0.5 {
            "Приемлемо ⚠️"
        } else {
            "Неудовлетворительно ❌"
        }
    }
}

fn main() {
    let metrics = PerformanceMetrics {
        total_return: 0.45,
        annual_return: 0.18,
        sharpe_ratio: 1.6,
        max_drawdown: -0.12,
        win_rate: 0.58,
        profit_factor: 1.8,
        total_trades: 150,
        winning_trades: 87,
        losing_trades: 63,
        avg_profit_per_trade: 0.003,
        max_profit: 0.08,
        max_loss: -0.05,
    };

    metrics.print();
    println!("\nОценка: {}", metrics.grade());
}
```

## Полный отчёт о бэктесте

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    entry_time: String,
    exit_time: String,
    symbol: String,
    side: String,  // "LONG" или "SHORT"
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    pnl_pct: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct BacktestReport {
    metadata: BacktestMetadata,
    parameters: StrategyParameters,
    metrics: PerformanceMetrics,
    trades: Vec<Trade>,
    equity_curve: Vec<f64>,  // Эквити по дням
}

impl BacktestReport {
    fn new(
        metadata: BacktestMetadata,
        parameters: StrategyParameters,
        metrics: PerformanceMetrics,
        trades: Vec<Trade>,
        equity_curve: Vec<f64>,
    ) -> Self {
        Self {
            metadata,
            parameters,
            metrics,
            trades,
            equity_curve,
        }
    }

    /// Сохранить отчёт в JSON файл
    fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(filename, json)?;
        println!("✅ Отчёт сохранён в: {}", filename);
        Ok(())
    }

    /// Загрузить отчёт из JSON файла
    fn load_from_file(filename: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(filename)?;
        let report: BacktestReport = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(report)
    }

    /// Создать текстовый отчёт
    fn generate_text_report(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════\n");
        report.push_str("           ОТЧЁТ О БЭКТЕСТИНГЕ СТРАТЕГИИ\n");
        report.push_str("═══════════════════════════════════════════════════\n\n");

        // Метаданные
        report.push_str(&format!("ID теста: {}\n", self.metadata.test_id));
        report.push_str(&format!("Стратегия: {}\n", self.metadata.strategy_name));
        report.push_str(&format!("Версия кода: {}\n", self.metadata.code_version));
        report.push_str(&format!("Дата: {}\n", self.metadata.timestamp.format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("Описание: {}\n\n", self.metadata.description));

        // Параметры
        report.push_str("─── Параметры стратегии ───\n");
        report.push_str(&format!("MA Short: {}\n", self.parameters.ma_short_period));
        report.push_str(&format!("MA Long: {}\n", self.parameters.ma_long_period));
        report.push_str(&format!("Stop Loss: {:.2}%\n", self.parameters.stop_loss_pct * 100.0));
        report.push_str(&format!("Take Profit: {:.2}%\n\n", self.parameters.take_profit_pct * 100.0));

        // Метрики
        report.push_str("─── Результаты ───\n");
        report.push_str(&format!("Общая доходность: {:.2}%\n", self.metrics.total_return * 100.0));
        report.push_str(&format!("Sharpe Ratio: {:.2}\n", self.metrics.sharpe_ratio));
        report.push_str(&format!("Max Drawdown: {:.2}%\n", self.metrics.max_drawdown * 100.0));
        report.push_str(&format!("Win Rate: {:.2}%\n", self.metrics.win_rate * 100.0));
        report.push_str(&format!("Всего сделок: {}\n", self.metrics.total_trades));
        report.push_str(&format!("Оценка: {}\n\n", self.metrics.grade()));

        report.push_str("═══════════════════════════════════════════════════\n");

        report
    }

    fn print_summary(&self) {
        println!("{}", self.generate_text_report());
    }
}

fn main() {
    // Создаём примерный отчёт
    let metadata = BacktestMetadata::new(
        "MA Crossover v2.1",
        "Тест на BTC/USDT с оптимизированными параметрами"
    );

    let parameters = StrategyParameters {
        ma_short_period: 10,
        ma_long_period: 50,
        stop_loss_pct: 0.02,
        take_profit_pct: 0.05,
        max_position_size: 0.10,
    };

    let metrics = PerformanceMetrics {
        total_return: 0.45,
        annual_return: 0.18,
        sharpe_ratio: 1.6,
        max_drawdown: -0.12,
        win_rate: 0.58,
        profit_factor: 1.8,
        total_trades: 150,
        winning_trades: 87,
        losing_trades: 63,
        avg_profit_per_trade: 0.003,
        max_profit: 0.08,
        max_loss: -0.05,
    };

    let trades = vec![
        Trade {
            entry_time: "2024-01-15 10:30:00".to_string(),
            exit_time: "2024-01-15 14:20:00".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: "LONG".to_string(),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.1,
            pnl: 150.0,
            pnl_pct: 0.0357,
        },
        // ... больше сделок
    ];

    let equity_curve = vec![10000.0, 10150.0, 10300.0, 10250.0, 10500.0];

    let report = BacktestReport::new(metadata, parameters, metrics, trades, equity_curve);

    // Печатаем краткий отчёт
    report.print_summary();

    // Сохраняем в файл
    let filename = format!("backtest_report_{}.json", report.metadata.test_id);
    report.save_to_file(&filename).unwrap();
}
```

## База данных результатов

Для хранения множества тестов можно использовать простую файловую систему или базу данных:

```rust
use std::fs;
use std::path::Path;

struct BacktestDatabase {
    storage_path: String,
}

impl BacktestDatabase {
    fn new(storage_path: &str) -> Self {
        // Создаём папку для хранения, если не существует
        fs::create_dir_all(storage_path).ok();
        Self {
            storage_path: storage_path.to_string(),
        }
    }

    /// Сохранить отчёт в базу
    fn save_report(&self, report: &BacktestReport) -> std::io::Result<String> {
        let filename = format!(
            "{}/{}_{}_{}.json",
            self.storage_path,
            report.metadata.timestamp.format("%Y%m%d_%H%M%S"),
            report.metadata.strategy_name.replace(" ", "_"),
            &report.metadata.test_id[..8]
        );

        report.save_to_file(&filename)?;
        Ok(filename)
    }

    /// Загрузить все отчёты
    fn load_all_reports(&self) -> Vec<BacktestReport> {
        let mut reports = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.storage_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Ok(report) = BacktestReport::load_from_file(
                            entry.path().to_str().unwrap()
                        ) {
                            reports.push(report);
                        }
                    }
                }
            }
        }

        reports
    }

    /// Найти лучшие стратегии по Sharpe ratio
    fn find_best_strategies(&self, top_n: usize) -> Vec<BacktestReport> {
        let mut reports = self.load_all_reports();
        reports.sort_by(|a, b| {
            b.metrics.sharpe_ratio
                .partial_cmp(&a.metrics.sharpe_ratio)
                .unwrap()
        });
        reports.truncate(top_n);
        reports
    }

    /// Статистика по всем тестам
    fn print_statistics(&self) {
        let reports = self.load_all_reports();

        println!("=== Статистика базы данных бэктестов ===");
        println!("Всего тестов: {}", reports.len());

        if reports.is_empty() {
            return;
        }

        let avg_sharpe: f64 = reports.iter()
            .map(|r| r.metrics.sharpe_ratio)
            .sum::<f64>() / reports.len() as f64;

        let avg_return: f64 = reports.iter()
            .map(|r| r.metrics.total_return)
            .sum::<f64>() / reports.len() as f64;

        println!("Средний Sharpe: {:.2}", avg_sharpe);
        println!("Средняя доходность: {:.2}%", avg_return * 100.0);

        println!("\nТоп-3 стратегии по Sharpe:");
        for (i, report) in self.find_best_strategies(3).iter().enumerate() {
            println!("  {}. {} - Sharpe: {:.2}",
                i + 1,
                report.metadata.strategy_name,
                report.metrics.sharpe_ratio
            );
        }
    }
}

fn main() {
    let db = BacktestDatabase::new("./backtest_results");

    // Сохраняем несколько отчётов (примеры)
    println!("Сохранение тестовых отчётов...\n");

    // ... создание и сохранение отчётов

    // Статистика
    db.print_statistics();
}
```

## Автоматическая генерация Markdown отчётов

```rust
impl BacktestReport {
    /// Генерация Markdown отчёта для документации
    fn generate_markdown_report(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Backtest Report: {}\n\n", self.metadata.strategy_name));

        md.push_str("## Metadata\n\n");
        md.push_str(&format!("- **Test ID**: `{}`\n", self.metadata.test_id));
        md.push_str(&format!("- **Strategy**: {}\n", self.metadata.strategy_name));
        md.push_str(&format!("- **Code Version**: `{}`\n", self.metadata.code_version));
        md.push_str(&format!("- **Date**: {}\n", self.metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        md.push_str(&format!("- **Author**: {}\n", self.metadata.author));
        md.push_str(&format!("- **Description**: {}\n\n", self.metadata.description));

        md.push_str("## Strategy Parameters\n\n");
        md.push_str("```json\n");
        md.push_str(&self.parameters.to_json());
        md.push_str("\n```\n\n");

        md.push_str("## Performance Metrics\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total Return | {:.2}% |\n", self.metrics.total_return * 100.0));
        md.push_str(&format!("| Annual Return | {:.2}% |\n", self.metrics.annual_return * 100.0));
        md.push_str(&format!("| Sharpe Ratio | {:.2} |\n", self.metrics.sharpe_ratio));
        md.push_str(&format!("| Max Drawdown | {:.2}% |\n", self.metrics.max_drawdown * 100.0));
        md.push_str(&format!("| Win Rate | {:.2}% |\n", self.metrics.win_rate * 100.0));
        md.push_str(&format!("| Profit Factor | {:.2} |\n", self.metrics.profit_factor));
        md.push_str(&format!("| Total Trades | {} |\n", self.metrics.total_trades));
        md.push_str(&format!("| Winning Trades | {} |\n", self.metrics.winning_trades));
        md.push_str(&format!("| Losing Trades | {} |\n\n", self.metrics.losing_trades));

        md.push_str(&format!("**Grade**: {}\n\n", self.metrics.grade()));

        md.push_str("## Trade Summary\n\n");
        md.push_str(&format!("Total trades: {}\n\n", self.trades.len()));

        if !self.trades.is_empty() {
            md.push_str("### First 5 Trades\n\n");
            md.push_str("| Entry Time | Symbol | Side | Entry Price | Exit Price | P&L % |\n");
            md.push_str("|------------|--------|------|-------------|------------|-------|\n");

            for trade in self.trades.iter().take(5) {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.2} | {:.2} | {:.2}% |\n",
                    trade.entry_time,
                    trade.symbol,
                    trade.side,
                    trade.entry_price,
                    trade.exit_price,
                    trade.pnl_pct * 100.0
                ));
            }
            md.push_str("\n");
        }

        md
    }

    fn save_markdown_report(&self, filename: &str) -> std::io::Result<()> {
        let markdown = self.generate_markdown_report();
        std::fs::write(filename, markdown)?;
        println!("✅ Markdown отчёт сохранён: {}", filename);
        Ok(())
    }
}

fn main() {
    // ... создание отчёта

    let report = BacktestReport::new(
        BacktestMetadata::new("MA Crossover", "Test description"),
        StrategyParameters {
            ma_short_period: 10,
            ma_long_period: 50,
            stop_loss_pct: 0.02,
            take_profit_pct: 0.05,
            max_position_size: 0.10,
        },
        PerformanceMetrics {
            total_return: 0.45,
            annual_return: 0.18,
            sharpe_ratio: 1.6,
            max_drawdown: -0.12,
            win_rate: 0.58,
            profit_factor: 1.8,
            total_trades: 150,
            winning_trades: 87,
            losing_trades: 63,
            avg_profit_per_trade: 0.003,
            max_profit: 0.08,
            max_loss: -0.05,
        },
        vec![],
        vec![],
    );

    // Сохраняем Markdown отчёт
    report.save_markdown_report("backtest_report.md").unwrap();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Метаданные** | ID теста, версия кода, время, автор, описание |
| **Параметры** | Все настройки стратегии в структурированном виде |
| **Метрики** | Sharpe, доходность, просадка, win rate и другие |
| **Сделки** | Полный журнал всех сделок с деталями |
| **Equity Curve** | График изменения капитала во времени |
| **JSON** | Стандартный формат для сериализации данных |
| **База данных** | Хранилище всех бэктестов для сравнения |
| **Markdown** | Человеко-читаемые отчёты для документации |

## Домашнее задание

1. **Расширенный отчёт**: Добавь в `BacktestReport`:
   - Список всех используемых индикаторов с их параметрами
   - Информацию о торгуемых инструментах (символы, таймфреймы)
   - Статистику по месяцам (помесячная доходность)
   - Время выполнения бэктеста
   - Системную информацию (OS, версия Rust)

2. **Сравнительный отчёт**: Создай функцию `compare_reports`:
   - Принимает 2-5 отчётов о бэктестах
   - Создаёт таблицу сравнения всех метрик
   - Выделяет лучшие значения в каждой категории
   - Генерирует рекомендацию, какую стратегию выбрать
   - Сохраняет в Markdown с красивым форматированием

3. **HTML Dashboard**: Сгенерируй HTML страницу с:
   - Визуализацией equity curve (можно использовать plotly или Chart.js)
   - Таблицами метрик
   - Списком всех сделок
   - Фильтрацией по прибыльным/убыточным сделкам
   - Интерактивными графиками просадки

4. **Автоматическая отправка отчётов**: Реализуй систему уведомлений:
   - После каждого бэктеста отправляй краткий отчёт в Telegram/Slack
   - При обнаружении стратегии с Sharpe > 2.0 — срочное уведомление
   - Еженедельная сводка всех тестов за неделю
   - Автоматическое создание GitHub issue при провале теста

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md) | [Следующий день →](../304-project-backtesting-engine/ru.md)
