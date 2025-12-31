# День 299: Мульти-инструмент тестирование

## Аналогия из трейдинга

Представь, что ты разработал торговую стратегию и протестировал её на Bitcoin. Бэктест показал отличные результаты: +150% за год! Ты в восторге и запускаешь её на реальном рынке, но теперь на Ethereum — и внезапно теряешь деньги. Что пошло не так?

Стратегия была **переоптимизирована** под специфику Bitcoin. Она не учитывает, что разные инструменты имеют разные характеристики:
- Bitcoin может иметь волатильность 3-5% в день
- Ethereum может быть более волатильным (4-7%)
- Акции обычно менее волатильны (1-2%)
- Пары форекс имеют свои уникальные паттерны

**Мульти-инструмент тестирование** — это проверка стратегии на различных торговых инструментах, чтобы убедиться, что она работает не из-за везения на одном активе, а благодаря надёжным торговым принципам. Это как тестирование автомобиля на разных дорогах (асфальт, грунт, снег), а не только на трассе в идеальную погоду.

## Что такое мульти-инструмент тестирование?

Мульти-инструмент тестирование (multi-instrument testing) — это метод валидации торговых стратегий, при котором одна и та же стратегия тестируется на разных финансовых инструментах для проверки её устойчивости и универсальности.

### Зачем это нужно?

| Причина | Описание |
|---------|----------|
| **Обнаружение переоптимизации** | Стратегия может быть подогнана под один инструмент |
| **Проверка универсальности** | Хорошие стратегии работают на разных рынках |
| **Оценка риска** | Разные инструменты показывают, как стратегия ведёт себя в разных условиях |
| **Диверсификация** | Понимание, на каких инструментах стратегия работает лучше всего |
| **Устойчивость** | Стратегия должна быть устойчива к различным рыночным режимам |

## Базовая реализация: тестирование на одном инструменте

Сначала создадим структуру для тестирования одной стратегии:

```rust
#[derive(Debug, Clone)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit_pct: f64,
    holding_bars: usize,
}

#[derive(Debug)]
struct BacktestResult {
    instrument: String,
    total_trades: usize,
    winning_trades: usize,
    total_return: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
    win_rate: f64,
}

impl BacktestResult {
    fn new(instrument: String) -> Self {
        BacktestResult {
            instrument,
            total_trades: 0,
            winning_trades: 0,
            total_return: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
        }
    }

    fn calculate_metrics(&mut self, trades: &[Trade]) {
        self.total_trades = trades.len();
        self.winning_trades = trades.iter().filter(|t| t.profit_pct > 0.0).count();
        self.win_rate = if self.total_trades > 0 {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        } else {
            0.0
        };

        // Общая доходность
        self.total_return = trades.iter().map(|t| t.profit_pct).sum();

        // Максимальная просадка (упрощённо)
        let mut peak = 0.0;
        let mut current = 0.0;
        let mut max_dd = 0.0;

        for trade in trades {
            current += trade.profit_pct;
            if current > peak {
                peak = current;
            }
            let dd = peak - current;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        self.max_drawdown = max_dd;

        // Sharpe Ratio (упрощённо)
        if !trades.is_empty() {
            let mean = self.total_return / trades.len() as f64;
            let variance: f64 = trades
                .iter()
                .map(|t| (t.profit_pct - mean).powi(2))
                .sum::<f64>()
                / trades.len() as f64;
            let std_dev = variance.sqrt();
            self.sharpe_ratio = if std_dev > 0.0 {
                mean / std_dev
            } else {
                0.0
            };
        }
    }
}

fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    if period == 0 || prices.is_empty() {
        return vec![0.0; prices.len()];
    }
    for i in 0..prices.len() {
        if i + 1 < period {
            sma.push(0.0);
        } else {
            let start = i + 1 - period;
            let sum: f64 = prices[start..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }
    sma
}

fn backtest_sma_crossover(data: &[OHLCV], fast_period: usize, slow_period: usize) -> Vec<Trade> {
    let closes: Vec<f64> = data.iter().map(|bar| bar.close).collect();
    let fast_sma = simple_moving_average(&closes, fast_period);
    let slow_sma = simple_moving_average(&closes, slow_period);

    let mut trades = Vec::new();
    let mut position: Option<(f64, usize)> = None; // (entry_price, entry_index)

    for i in slow_period..data.len() {
        if fast_sma[i] == 0.0 || slow_sma[i] == 0.0 {
            continue;
        }

        // Сигнал на покупку: быстрая MA пересекает медленную снизу вверх
        if position.is_none()
            && fast_sma[i] > slow_sma[i]
            && fast_sma[i - 1] <= slow_sma[i - 1]
        {
            position = Some((data[i].close, i));
        }
        // Сигнал на продажу: быстрая MA пересекает медленную сверху вниз
        else if let Some((entry_price, entry_idx)) = position {
            if fast_sma[i] < slow_sma[i] && fast_sma[i - 1] >= slow_sma[i - 1] {
                let exit_price = data[i].close;
                let profit_pct = ((exit_price - entry_price) / entry_price) * 100.0;
                trades.push(Trade {
                    entry_price,
                    exit_price,
                    profit_pct,
                    holding_bars: i - entry_idx,
                });
                position = None;
            }
        }
    }

    // Закрываем открытую позицию в конце
    if let Some((entry_price, entry_idx)) = position {
        let exit_price = data[data.len() - 1].close;
        let profit_pct = ((exit_price - entry_price) / entry_price) * 100.0;
        trades.push(Trade {
            entry_price,
            exit_price,
            profit_pct,
            holding_bars: data.len() - 1 - entry_idx,
        });
    }

    trades
}

fn main() {
    // Симулированные данные для Bitcoin
    let btc_data: Vec<OHLCV> = (0..200)
        .map(|i| {
            let trend = 40000.0 + i as f64 * 50.0;
            let cycle = 2000.0 * (i as f64 * 0.05).sin();
            let noise = 100.0 * (i as f64 * 7.0).sin();
            let price = trend + cycle + noise;
            OHLCV {
                timestamp: i as u64,
                open: price - 50.0,
                high: price + 100.0,
                low: price - 100.0,
                close: price,
                volume: 1000000.0,
            }
        })
        .collect();

    println!("=== Тестирование стратегии SMA Crossover на Bitcoin ===\n");

    // Запускаем бэктест
    let trades = backtest_sma_crossover(&btc_data, 10, 30);
    let mut result = BacktestResult::new("BTC/USD".to_string());
    result.calculate_metrics(&trades);

    println!("Инструмент: {}", result.instrument);
    println!("Всего сделок: {}", result.total_trades);
    println!("Прибыльных сделок: {}", result.winning_trades);
    println!("Win Rate: {:.2}%", result.win_rate);
    println!("Общая доходность: {:.2}%", result.total_return);
    println!("Максимальная просадка: {:.2}%", result.max_drawdown);
    println!("Sharpe Ratio: {:.2}", result.sharpe_ratio);
}
```

## Мульти-инструмент тестирование

Теперь расширим наш код для тестирования на нескольких инструментах:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Instrument {
    symbol: String,
    data: Vec<OHLCV>,
}

impl Instrument {
    fn new(symbol: &str, data: Vec<OHLCV>) -> Self {
        Instrument {
            symbol: symbol.to_string(),
            data,
        }
    }
}

struct MultiInstrumentTester {
    instruments: Vec<Instrument>,
    fast_period: usize,
    slow_period: usize,
}

impl MultiInstrumentTester {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        MultiInstrumentTester {
            instruments: Vec::new(),
            fast_period,
            slow_period,
        }
    }

    fn add_instrument(&mut self, instrument: Instrument) {
        self.instruments.push(instrument);
    }

    fn run_tests(&self) -> Vec<BacktestResult> {
        let mut results = Vec::new();

        for instrument in &self.instruments {
            let trades = backtest_sma_crossover(&instrument.data, self.fast_period, self.slow_period);
            let mut result = BacktestResult::new(instrument.symbol.clone());
            result.calculate_metrics(&trades);
            results.push(result);
        }

        results
    }

    fn print_summary(&self, results: &[BacktestResult]) {
        println!("\n=== Сводка по всем инструментам ===\n");
        println!("{:<12} {:<12} {:<12} {:<15} {:<15} {:<12}",
            "Инструмент", "Сделок", "Win Rate", "Доходность", "Просадка", "Sharpe");
        println!("{}", "-".repeat(85));

        for result in results {
            println!("{:<12} {:<12} {:<11.2}% {:<14.2}% {:<14.2}% {:<12.2}",
                result.instrument,
                result.total_trades,
                result.win_rate,
                result.total_return,
                result.max_drawdown,
                result.sharpe_ratio
            );
        }

        // Средние показатели
        let avg_win_rate = results.iter().map(|r| r.win_rate).sum::<f64>() / results.len() as f64;
        let avg_return = results.iter().map(|r| r.total_return).sum::<f64>() / results.len() as f64;
        let avg_sharpe = results.iter().map(|r| r.sharpe_ratio).sum::<f64>() / results.len() as f64;

        println!("{}", "-".repeat(85));
        println!("{:<12} {:<12} {:<11.2}% {:<14.2}% {:<14} {:<12.2}",
            "СРЕДНЕЕ", "-", avg_win_rate, avg_return, "-", avg_sharpe);
    }
}

fn generate_synthetic_data(symbol: &str, base_price: f64, volatility: f64, trend: f64) -> Vec<OHLCV> {
    (0..200)
        .map(|i| {
            let trend_component = base_price + i as f64 * trend;
            let cycle = volatility * (i as f64 * 0.05).sin();
            let noise = volatility * 0.2 * (i as f64 * 7.0).sin();
            let price = trend_component + cycle + noise;
            OHLCV {
                timestamp: i as u64,
                open: price - volatility * 0.1,
                high: price + volatility * 0.15,
                low: price - volatility * 0.15,
                close: price,
                volume: 1000000.0 * (1.0 + (i as f64 * 0.01).sin() * 0.3),
            }
        })
        .collect()
}

fn main() {
    println!("=== Мульти-инструмент тестирование ===\n");

    // Создаём тестер
    let mut tester = MultiInstrumentTester::new(10, 30);

    // Добавляем различные инструменты с разными характеристиками
    tester.add_instrument(Instrument::new(
        "BTC/USD",
        generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0),
    ));

    tester.add_instrument(Instrument::new(
        "ETH/USD",
        generate_synthetic_data("ETH", 2500.0, 150.0, 3.0),
    ));

    tester.add_instrument(Instrument::new(
        "AAPL",
        generate_synthetic_data("AAPL", 150.0, 5.0, 0.2),
    ));

    tester.add_instrument(Instrument::new(
        "EUR/USD",
        generate_synthetic_data("EUR", 1.1, 0.02, 0.0001),
    ));

    tester.add_instrument(Instrument::new(
        "GOLD",
        generate_synthetic_data("GOLD", 1800.0, 30.0, 0.5),
    ));

    // Запускаем тесты
    let results = tester.run_tests();

    // Выводим результаты
    tester.print_summary(&results);

    // Анализ устойчивости
    println!("\n=== Анализ устойчивости стратегии ===\n");

    let profitable_instruments = results.iter().filter(|r| r.total_return > 0.0).count();
    let consistency_score = (profitable_instruments as f64 / results.len() as f64) * 100.0;

    println!("Прибыльных инструментов: {}/{}", profitable_instruments, results.len());
    println!("Оценка устойчивости: {:.1}%", consistency_score);

    if consistency_score >= 70.0 {
        println!("\n✓ Стратегия показывает хорошую устойчивость (>70%)");
    } else if consistency_score >= 50.0 {
        println!("\n⚠ Стратегия показывает умеренную устойчивость (50-70%)");
    } else {
        println!("\n✗ Стратегия не устойчива (<50%)");
    }
}
```

## Продвинутые метрики: корреляционный анализ

Важно понимать, насколько результаты на разных инструментах коррелируют друг с другом:

```rust
#[derive(Debug)]
struct CorrelationMatrix {
    instruments: Vec<String>,
    matrix: Vec<Vec<f64>>,
}

impl CorrelationMatrix {
    fn calculate(results: &[BacktestResult]) -> Self {
        let n = results.len();
        let instruments: Vec<String> = results.iter().map(|r| r.instrument.clone()).collect();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 1.0;
                } else {
                    // Упрощённая корреляция на основе соотношения метрик
                    let corr = calculate_simple_correlation(
                        results[i].total_return,
                        results[i].sharpe_ratio,
                        results[j].total_return,
                        results[j].sharpe_ratio,
                    );
                    matrix[i][j] = corr;
                }
            }
        }

        CorrelationMatrix {
            instruments,
            matrix,
        }
    }

    fn print(&self) {
        println!("\n=== Матрица корреляции результатов ===\n");
        print!("{:<12}", "");
        for symbol in &self.instruments {
            print!("{:<12}", symbol);
        }
        println!();

        for (i, symbol) in self.instruments.iter().enumerate() {
            print!("{:<12}", symbol);
            for j in 0..self.instruments.len() {
                print!("{:<12.2}", self.matrix[i][j]);
            }
            println!();
        }
    }
}

fn calculate_simple_correlation(
    return1: f64,
    sharpe1: f64,
    return2: f64,
    sharpe2: f64,
) -> f64 {
    // Упрощённая формула: среднее нормализованных метрик
    let norm_return = 1.0 - ((return1 - return2).abs() / (return1.abs() + return2.abs() + 0.01));
    let norm_sharpe = 1.0 - ((sharpe1 - sharpe2).abs() / (sharpe1.abs() + sharpe2.abs() + 0.01));
    (norm_return + norm_sharpe) / 2.0
}

fn main() {
    println!("=== Корреляционный анализ ===\n");

    let mut tester = MultiInstrumentTester::new(10, 30);

    tester.add_instrument(Instrument::new("BTC/USD", generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0)));
    tester.add_instrument(Instrument::new("ETH/USD", generate_synthetic_data("ETH", 2500.0, 150.0, 3.0)));
    tester.add_instrument(Instrument::new("AAPL", generate_synthetic_data("AAPL", 150.0, 5.0, 0.2)));

    let results = tester.run_tests();
    tester.print_summary(&results);

    let correlation = CorrelationMatrix::calculate(&results);
    correlation.print();
}
```

## Адаптивные параметры для разных инструментов

Часто стратегия требует разных параметров для разных инструментов:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct InstrumentConfig {
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    stop_loss_pct: f64,
}

struct AdaptiveMultiInstrumentTester {
    configs: HashMap<String, InstrumentConfig>,
    instruments: Vec<Instrument>,
}

impl AdaptiveMultiInstrumentTester {
    fn new() -> Self {
        AdaptiveMultiInstrumentTester {
            configs: HashMap::new(),
            instruments: Vec::new(),
        }
    }

    fn add_instrument_with_config(&mut self, instrument: Instrument, config: InstrumentConfig) {
        self.configs.insert(instrument.symbol.clone(), config);
        self.instruments.push(instrument);
    }

    fn run_adaptive_tests(&self) -> Vec<BacktestResult> {
        let mut results = Vec::new();

        for instrument in &self.instruments {
            if let Some(config) = self.configs.get(&instrument.symbol) {
                let trades = backtest_sma_crossover(
                    &instrument.data,
                    config.fast_period,
                    config.slow_period,
                );
                let mut result = BacktestResult::new(instrument.symbol.clone());
                result.calculate_metrics(&trades);
                results.push(result);
            }
        }

        results
    }
}

fn main() {
    println!("=== Адаптивное мульти-инструмент тестирование ===\n");

    let mut tester = AdaptiveMultiInstrumentTester::new();

    // BTC: более медленные параметры из-за высокой волатильности
    tester.add_instrument_with_config(
        Instrument::new("BTC/USD", generate_synthetic_data("BTC", 40000.0, 2000.0, 50.0)),
        InstrumentConfig {
            symbol: "BTC/USD".to_string(),
            fast_period: 15,
            slow_period: 40,
            stop_loss_pct: 5.0,
        },
    );

    // ETH: средние параметры
    tester.add_instrument_with_config(
        Instrument::new("ETH/USD", generate_synthetic_data("ETH", 2500.0, 150.0, 3.0)),
        InstrumentConfig {
            symbol: "ETH/USD".to_string(),
            fast_period: 12,
            slow_period: 35,
            stop_loss_pct: 4.0,
        },
    );

    // AAPL: быстрые параметры для акций
    tester.add_instrument_with_config(
        Instrument::new("AAPL", generate_synthetic_data("AAPL", 150.0, 5.0, 0.2)),
        InstrumentConfig {
            symbol: "AAPL".to_string(),
            fast_period: 8,
            slow_period: 20,
            stop_loss_pct: 2.0,
        },
    );

    let results = tester.run_adaptive_tests();

    println!("{:<12} {:<12} {:<12} {:<12} {:<15}",
        "Инструмент", "Fast MA", "Slow MA", "Сделок", "Доходность");
    println!("{}", "-".repeat(70));

    for result in &results {
        if let Some(config) = tester.configs.get(&result.instrument) {
            println!("{:<12} {:<12} {:<12} {:<12} {:<14.2}%",
                result.instrument,
                config.fast_period,
                config.slow_period,
                result.total_trades,
                result.total_return
            );
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Мульти-инструмент тестирование | Проверка стратегии на различных торговых инструментах |
| Устойчивость стратегии | Хорошая стратегия работает на разных рынках |
| Корреляционный анализ | Понимание, как результаты на разных инструментах связаны |
| Адаптивные параметры | Разные инструменты могут требовать разных настроек |
| Оценка консистентности | Процент инструментов, на которых стратегия прибыльна |
| Диверсификация | Тестирование помогает понять, где стратегия работает лучше |

## Практические задания

1. **Расширенные метрики**: Добавь расчёт дополнительных метрик для каждого инструмента:
   - Средняя длительность сделки
   - Максимальная серия убыточных сделок
   - Profit Factor (сумма прибылей / сумма убытков)
   - Восстановление после просадки

2. **Нормализация по волатильности**: Создай функцию, которая нормализует результаты с учётом волатильности инструмента, чтобы справедливо сравнивать BTC и акции.

3. **Автоматическая оптимизация**: Реализуй систему, которая автоматически подбирает оптимальные параметры для каждого инструмента с использованием grid search.

4. **Группировка инструментов**: Реализуй анализ, который группирует инструменты по характеристикам (криптовалюты, акции, форекс) и показывает, на каких группах стратегия работает лучше.

## Домашнее задание

1. **Реализовать портфельное тестирование**: Создай систему, которая запускает стратегию одновременно на всех инструментах как портфель, учитывая распределение капитала между инструментами.

2. **Добавить реалистичные данные**: Используй реальные исторические данные (через API или CSV файлы) вместо синтетических для более точного тестирования.

3. **Создать визуализацию**: Реализуй вывод результатов в виде таблицы или графика, показывающего доходность по каждому инструменту.

4. **Тестирование на разных таймфреймах**: Расширь тестер, чтобы он мог проверять стратегию не только на разных инструментах, но и на разных таймфреймах (1m, 5m, 1h, 1d).

## Навигация

[← Предыдущий день](../293-grid-search-parameter-sweep/ru.md) | [Следующий день →](../300-advanced-backtesting/ru.md)
