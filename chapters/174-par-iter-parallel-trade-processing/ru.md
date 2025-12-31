# День 174: par_iter: параллельная обработка сделок

## Аналогия из трейдинга

Представь, что ты управляешь хедж-фондом и тебе нужно проанализировать 10 000 акций для выбора лучших кандидатов. Если анализировать каждую акцию последовательно, это займёт очень много времени. Но если у тебя есть команда из 8 аналитиков, каждый может взять часть акций и работать параллельно — работа завершится в 8 раз быстрее!

В Rust библиотека **Rayon** предоставляет `par_iter()` — параллельный итератор, который автоматически распределяет работу между всеми ядрами процессора. Это как иметь команду аналитиков, где Rayon сам решает, кто какую акцию анализирует.

В реальном алготрейдинге `par_iter` используется для:
- Параллельного расчёта индикаторов по множеству инструментов
- Одновременной обработки ордеров из разных бирж
- Массового бэктестинга стратегий на исторических данных
- Расчёта риск-метрик по всему портфелю

## Что такое Rayon и par_iter?

**Rayon** — это библиотека для параллелизма данных в Rust. Она позволяет легко превратить последовательный код в параллельный, просто заменив `.iter()` на `.par_iter()`.

### Основные преимущества:
- **Простота** — минимальные изменения в коде
- **Безопасность** — компилятор гарантирует отсутствие гонок данных
- **Эффективность** — автоматическое распределение нагрузки (work-stealing)
- **Масштабируемость** — автоматически использует все доступные ядра

## Установка Rayon

Добавь в `Cargo.toml`:

```toml
[dependencies]
rayon = "1.10"
```

## Простой пример: расчёт прибыли по сделкам

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

impl Trade {
    fn profit(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.quantity
    }
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), entry_price: 40000.0, exit_price: 42000.0, quantity: 0.5 },
        Trade { symbol: "ETH".to_string(), entry_price: 2500.0, exit_price: 2700.0, quantity: 10.0 },
        Trade { symbol: "SOL".to_string(), entry_price: 100.0, exit_price: 95.0, quantity: 50.0 },
        Trade { symbol: "BNB".to_string(), entry_price: 300.0, exit_price: 320.0, quantity: 5.0 },
    ];

    // Последовательный расчёт
    let sequential_profit: f64 = trades.iter()
        .map(|t| t.profit())
        .sum();

    // Параллельный расчёт — просто замени iter() на par_iter()!
    let parallel_profit: f64 = trades.par_iter()
        .map(|t| t.profit())
        .sum();

    println!("Последовательно: ${:.2}", sequential_profit);
    println!("Параллельно: ${:.2}", parallel_profit);
    // Результаты идентичны: $3100.00
}
```

## Сравнение производительности

Давайте сравним производительность на большом наборе данных:

```rust
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone)]
struct OHLCV {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// Расчёт волатильности — относительно тяжёлая операция
fn calculate_volatility(candles: &[OHLCV]) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }

    let returns: Vec<f64> = candles.windows(2)
        .map(|w| (w[1].close / w[0].close).ln())
        .collect();

    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;

    variance.sqrt() * (252.0_f64).sqrt() // Годовая волатильность
}

fn main() {
    // Генерируем данные для 1000 акций, по 1000 свечей каждая
    let stocks: Vec<Vec<OHLCV>> = (0..1000)
        .map(|_| {
            (0..1000)
                .map(|i| OHLCV {
                    open: 100.0 + (i as f64 * 0.1).sin() * 10.0,
                    high: 105.0 + (i as f64 * 0.1).sin() * 10.0,
                    low: 95.0 + (i as f64 * 0.1).sin() * 10.0,
                    close: 100.0 + (i as f64 * 0.1).cos() * 10.0,
                    volume: 1000000.0,
                })
                .collect()
        })
        .collect();

    // Последовательный расчёт
    let start = Instant::now();
    let _sequential: Vec<f64> = stocks.iter()
        .map(|candles| calculate_volatility(candles))
        .collect();
    let sequential_time = start.elapsed();

    // Параллельный расчёт
    let start = Instant::now();
    let _parallel: Vec<f64> = stocks.par_iter()
        .map(|candles| calculate_volatility(candles))
        .collect();
    let parallel_time = start.elapsed();

    println!("Последовательно: {:?}", sequential_time);
    println!("Параллельно: {:?}", parallel_time);
    println!("Ускорение: {:.2}x", sequential_time.as_secs_f64() / parallel_time.as_secs_f64());
}
```

На 8-ядерном процессоре можно ожидать ускорение в 4-7 раз!

## Параллельная фильтрация сигналов

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    signal_strength: f64,  // от -1.0 (продавать) до 1.0 (покупать)
    volume_ratio: f64,     // отношение объёма к среднему
    rsi: f64,              // RSI индикатор (0-100)
}

impl TradingSignal {
    fn is_strong_buy(&self) -> bool {
        self.signal_strength > 0.7
            && self.volume_ratio > 1.5
            && self.rsi < 30.0
    }

    fn is_strong_sell(&self) -> bool {
        self.signal_strength < -0.7
            && self.volume_ratio > 1.5
            && self.rsi > 70.0
    }
}

fn main() {
    // Генерируем сигналы для 10000 инструментов
    let signals: Vec<TradingSignal> = (0..10000)
        .map(|i| TradingSignal {
            symbol: format!("STOCK{}", i),
            signal_strength: ((i as f64 * 0.001).sin()),
            volume_ratio: 1.0 + (i as f64 * 0.002).cos().abs(),
            rsi: 50.0 + ((i as f64 * 0.003).sin() * 40.0),
        })
        .collect();

    // Параллельно находим сильные сигналы на покупку
    let buy_signals: Vec<&TradingSignal> = signals.par_iter()
        .filter(|s| s.is_strong_buy())
        .collect();

    // Параллельно находим сильные сигналы на продажу
    let sell_signals: Vec<&TradingSignal> = signals.par_iter()
        .filter(|s| s.is_strong_sell())
        .collect();

    println!("Найдено сигналов на покупку: {}", buy_signals.len());
    println!("Найдено сигналов на продажу: {}", sell_signals.len());

    // Вывод первых 5 сигналов на покупку
    for signal in buy_signals.iter().take(5) {
        println!("  {} - сила: {:.2}, RSI: {:.1}",
            signal.symbol, signal.signal_strength, signal.rsi);
    }
}
```

## Параллельная обработка портфеля

```rust
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    current_price: f64,
    daily_change: f64,
}

#[derive(Debug)]
struct PositionAnalysis {
    symbol: String,
    pnl: f64,
    pnl_percent: f64,
    market_value: f64,
}

fn analyze_position(position: &Position, market_data: &HashMap<String, MarketData>) -> PositionAnalysis {
    let market = market_data.get(&position.symbol)
        .unwrap_or(&MarketData {
            symbol: position.symbol.clone(),
            current_price: position.avg_price,
            daily_change: 0.0,
        });

    let market_value = position.quantity * market.current_price;
    let cost_basis = position.quantity * position.avg_price;
    let pnl = market_value - cost_basis;
    let pnl_percent = (pnl / cost_basis) * 100.0;

    PositionAnalysis {
        symbol: position.symbol.clone(),
        pnl,
        pnl_percent,
        market_value,
    }
}

fn main() {
    // Портфель из множества позиций
    let positions: Vec<Position> = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.0, avg_price: 40000.0 },
        Position { symbol: "ETH".to_string(), quantity: 50.0, avg_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 500.0, avg_price: 100.0 },
        Position { symbol: "BNB".to_string(), quantity: 100.0, avg_price: 300.0 },
        Position { symbol: "XRP".to_string(), quantity: 10000.0, avg_price: 0.5 },
        Position { symbol: "ADA".to_string(), quantity: 20000.0, avg_price: 0.4 },
        Position { symbol: "DOT".to_string(), quantity: 1000.0, avg_price: 7.0 },
        Position { symbol: "AVAX".to_string(), quantity: 200.0, avg_price: 35.0 },
    ];

    // Текущие рыночные данные
    let market_data: HashMap<String, MarketData> = [
        ("BTC", 42000.0, 2.5),
        ("ETH", 2700.0, 3.0),
        ("SOL", 110.0, 5.0),
        ("BNB", 320.0, 1.5),
        ("XRP", 0.55, -2.0),
        ("ADA", 0.45, 4.0),
        ("DOT", 7.5, 2.0),
        ("AVAX", 38.0, 3.5),
    ].iter()
    .map(|(symbol, price, change)| {
        (symbol.to_string(), MarketData {
            symbol: symbol.to_string(),
            current_price: *price,
            daily_change: *change,
        })
    })
    .collect();

    // Параллельный анализ всех позиций
    let analyses: Vec<PositionAnalysis> = positions.par_iter()
        .map(|pos| analyze_position(pos, &market_data))
        .collect();

    // Параллельный расчёт общих метрик
    let total_pnl: f64 = analyses.par_iter().map(|a| a.pnl).sum();
    let total_value: f64 = analyses.par_iter().map(|a| a.market_value).sum();

    println!("=== Анализ портфеля ===\n");
    for analysis in &analyses {
        println!("{}: P&L ${:.2} ({:+.2}%), Стоимость: ${:.2}",
            analysis.symbol, analysis.pnl, analysis.pnl_percent, analysis.market_value);
    }
    println!("\n=== Итого ===");
    println!("Общий P&L: ${:.2}", total_pnl);
    println!("Общая стоимость: ${:.2}", total_value);
}
```

## Параллельный бэктестинг стратегий

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone)]
struct BacktestResult {
    strategy_name: String,
    total_return: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    win_rate: f64,
    trades_count: u32,
}

// Симуляция бэктеста (в реальности это сложные вычисления)
fn backtest_strategy(strategy: &Strategy, prices: &[f64]) -> BacktestResult {
    // Упрощённая симуляция для демонстрации
    let volatility = strategy.fast_period as f64 / strategy.slow_period as f64;
    let base_return = (prices.last().unwrap_or(&100.0) / prices.first().unwrap_or(&100.0) - 1.0) * 100.0;

    BacktestResult {
        strategy_name: strategy.name.clone(),
        total_return: base_return * volatility,
        sharpe_ratio: (base_return * volatility) / 15.0,
        max_drawdown: 10.0 + volatility * 5.0,
        win_rate: 0.45 + volatility * 0.1,
        trades_count: (250.0 / (strategy.fast_period as f64 + strategy.slow_period as f64) * 10.0) as u32,
    }
}

fn main() {
    // Генерируем множество стратегий для оптимизации
    let strategies: Vec<Strategy> = (5..50)
        .flat_map(|fast| {
            (20..200).step_by(10).map(move |slow| {
                Strategy {
                    name: format!("MA_{}_{}", fast, slow),
                    fast_period: fast,
                    slow_period: slow,
                    stop_loss: 0.02,
                    take_profit: 0.05,
                }
            })
        })
        .filter(|s| s.fast_period < s.slow_period)
        .collect();

    println!("Тестируем {} стратегий...", strategies.len());

    // Исторические цены (симуляция)
    let prices: Vec<f64> = (0..1000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 20.0)
        .collect();

    // Параллельный бэктест всех стратегий
    let mut results: Vec<BacktestResult> = strategies.par_iter()
        .map(|strategy| backtest_strategy(strategy, &prices))
        .collect();

    // Сортируем по Sharpe Ratio
    results.sort_by(|a, b| b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap());

    println!("\n=== Топ-5 стратегий по Sharpe Ratio ===\n");
    for result in results.iter().take(5) {
        println!("{}: Return {:.2}%, Sharpe {:.2}, MaxDD {:.2}%, WinRate {:.2}%",
            result.strategy_name,
            result.total_return,
            result.sharpe_ratio,
            result.max_drawdown,
            result.win_rate * 100.0
        );
    }
}
```

## Параллельная агрегация с reduce

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct DailyStats {
    date: String,
    trades: u32,
    volume: f64,
    pnl: f64,
    fees: f64,
}

#[derive(Debug, Clone, Default)]
struct AggregatedStats {
    total_trades: u32,
    total_volume: f64,
    total_pnl: f64,
    total_fees: f64,
    best_day_pnl: f64,
    worst_day_pnl: f64,
}

fn main() {
    // Статистика по дням за год
    let daily_stats: Vec<DailyStats> = (0..365)
        .map(|day| {
            let pnl = ((day as f64 * 0.1).sin() * 1000.0) + 100.0;
            DailyStats {
                date: format!("2024-{:02}-{:02}", (day / 30) + 1, (day % 30) + 1),
                trades: 50 + (day % 30),
                volume: 1000000.0 + (day as f64 * 1000.0),
                pnl,
                fees: 50.0 + (day as f64 * 0.5),
            }
        })
        .collect();

    // Параллельная агрегация с reduce
    let aggregated = daily_stats.par_iter()
        .map(|day| AggregatedStats {
            total_trades: day.trades,
            total_volume: day.volume,
            total_pnl: day.pnl,
            total_fees: day.fees,
            best_day_pnl: day.pnl,
            worst_day_pnl: day.pnl,
        })
        .reduce(
            || AggregatedStats {
                best_day_pnl: f64::MIN,
                worst_day_pnl: f64::MAX,
                ..Default::default()
            },
            |mut acc, stats| {
                acc.total_trades += stats.total_trades;
                acc.total_volume += stats.total_volume;
                acc.total_pnl += stats.total_pnl;
                acc.total_fees += stats.total_fees;
                acc.best_day_pnl = acc.best_day_pnl.max(stats.best_day_pnl);
                acc.worst_day_pnl = acc.worst_day_pnl.min(stats.worst_day_pnl);
                acc
            }
        );

    println!("=== Годовая статистика ===\n");
    println!("Всего сделок: {}", aggregated.total_trades);
    println!("Общий объём: ${:.2}", aggregated.total_volume);
    println!("Общий P&L: ${:.2}", aggregated.total_pnl);
    println!("Общие комиссии: ${:.2}", aggregated.total_fees);
    println!("Лучший день: ${:.2}", aggregated.best_day_pnl);
    println!("Худший день: ${:.2}", aggregated.worst_day_pnl);
    println!("Чистая прибыль: ${:.2}", aggregated.total_pnl - aggregated.total_fees);
}
```

## Параллельная обработка ордеров с par_iter_mut

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
enum OrderStatus {
    New,
    Validated,
    Rejected(String),
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    status: OrderStatus,
}

fn validate_order(order: &mut Order, available_balance: f64, max_position: f64) {
    let order_value = order.quantity * order.price;

    if order.quantity <= 0.0 {
        order.status = OrderStatus::Rejected("Количество должно быть положительным".to_string());
    } else if order.price <= 0.0 {
        order.status = OrderStatus::Rejected("Цена должна быть положительной".to_string());
    } else if order_value > available_balance {
        order.status = OrderStatus::Rejected(format!(
            "Недостаточно средств: нужно {:.2}, есть {:.2}",
            order_value, available_balance
        ));
    } else if order.quantity > max_position {
        order.status = OrderStatus::Rejected(format!(
            "Превышен лимит позиции: {} > {}",
            order.quantity, max_position
        ));
    } else {
        order.status = OrderStatus::Validated;
    }
}

fn main() {
    let available_balance = 100_000.0;
    let max_position = 100.0;

    // Создаём множество ордеров для валидации
    let mut orders: Vec<Order> = (0..10000)
        .map(|i| Order {
            id: i,
            symbol: format!("STOCK{}", i % 100),
            quantity: (i % 150) as f64,
            price: 100.0 + (i % 50) as f64,
            status: OrderStatus::New,
        })
        .collect();

    // Параллельная валидация всех ордеров
    orders.par_iter_mut()
        .for_each(|order| validate_order(order, available_balance, max_position));

    // Подсчёт результатов
    let validated: Vec<_> = orders.iter()
        .filter(|o| matches!(o.status, OrderStatus::Validated))
        .collect();

    let rejected: Vec<_> = orders.iter()
        .filter(|o| matches!(o.status, OrderStatus::Rejected(_)))
        .collect();

    println!("Всего ордеров: {}", orders.len());
    println!("Прошли валидацию: {}", validated.len());
    println!("Отклонено: {}", rejected.len());

    // Примеры отклонённых ордеров
    println!("\nПримеры отклонённых ордеров:");
    for order in rejected.iter().take(5) {
        if let OrderStatus::Rejected(reason) = &order.status {
            println!("  Ордер {}: {}", order.id, reason);
        }
    }
}
```

## Когда использовать par_iter

### Когда par_iter эффективен:

```rust
use rayon::prelude::*;

// Хорошо: много элементов, каждая операция занимает время
let results: Vec<_> = large_dataset.par_iter()
    .map(|item| expensive_calculation(item))
    .collect();

// Хорошо: независимые вычисления
let stats: Vec<_> = symbols.par_iter()
    .map(|symbol| calculate_technical_indicators(symbol))
    .collect();
```

### Когда лучше использовать обычный iter:

```rust
// Плохо: слишком мало элементов
let small_data = vec![1, 2, 3, 4, 5];
let sum: i32 = small_data.par_iter().sum(); // Overhead > выигрыш

// Плохо: слишком простые операции
let doubled: Vec<_> = numbers.par_iter()
    .map(|x| x * 2) // Операция слишком быстрая
    .collect();

// Плохо: операции требуют синхронизации
let shared_state = Arc::new(Mutex::new(0));
data.par_iter().for_each(|x| {
    *shared_state.lock().unwrap() += x; // Блокировка убивает параллелизм
});
```

## Настройка пула потоков

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Создаём пул с определённым количеством потоков
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    let data: Vec<i32> = (0..1000).collect();

    // Выполняем работу в нашем пуле
    let result: i32 = pool.install(|| {
        data.par_iter()
            .map(|x| x * x)
            .sum()
    });

    println!("Результат: {}", result);

    // Можно задать глобальный пул при старте программы
    // ThreadPoolBuilder::new()
    //     .num_threads(8)
    //     .build_global()
    //     .unwrap();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `par_iter()` | Параллельный итератор для иммутабельных данных |
| `par_iter_mut()` | Параллельный итератор для мутабельных данных |
| `into_par_iter()` | Параллельный итератор с владением данными |
| `reduce()` | Параллельная агрегация с объединением результатов |
| Work-stealing | Автоматическое перераспределение работы между потоками |
| ThreadPool | Настройка количества рабочих потоков |

## Домашнее задание

1. **Параллельный скринер акций**: Создай структуру `Stock` с полями (symbol, price, volume, pe_ratio, market_cap) и реализуй функцию параллельного скрининга, которая фильтрует акции по множеству критериев (P/E < 15, объём > 1M, рост цены > 5%).

2. **Параллельный расчёт корреляций**: Напиши функцию, которая параллельно рассчитывает корреляции между всеми парами активов в портфеле. Используй `par_iter` для обработки всех комбинаций пар.

3. **Оптимизация параметров стратегии**: Реализуй параллельный перебор параметров для торговой стратегии (период MA, уровни стоп-лосса и тейк-профита). Найди комбинацию с лучшим соотношением доходность/риск.

4. **Параллельная обработка тиков**: Создай систему, которая параллельно обрабатывает поток тиковых данных с разных бирж, агрегируя OHLCV свечи для каждого инструмента.

## Навигация

[← День 164: Deadlock](../164-deadlock-threads-block/ru.md) | [Следующий день →](../175-threadpool-work-distribution/ru.md)
