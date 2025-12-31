# День 173: rayon: параллельные итераторы

## Аналогия из трейдинга

Представь, что ты трейдер, которому нужно проанализировать цены закрытия 1000 акций, чтобы найти самые волатильные за сегодня. Делать это последовательно — всё равно что проверять каждую акцию по очереди: работает, но медленно. А что если нанять 8 аналитиков, дать каждому ~125 акций и пусть работают параллельно? Именно это и делает **rayon** для твоего кода!

В реальном трейдинге параллелизация критически важна для:
- Анализа тысяч ценовых точек по множеству активов
- Расчёта метрик риска для всего портфеля
- Бэктестинга стратегий по нескольким временным периодам одновременно
- Обработки рыночных данных в реальном времени с нескольких бирж

## Что такое Rayon?

Rayon — это библиотека Rust для **параллелизма данных** с минимальными изменениями кода. Вместо ручного создания потоков, распределения работы и сбора результатов, ты просто меняешь `.iter()` на `.par_iter()`, и rayon делает всё автоматически.

### Ключевые особенности

1. **Прямая замена** — минимальные изменения в коде
2. **Work stealing** — эффективное распределение нагрузки между ядрами CPU
3. **Автоматическое управление потоками** — никакого ручного создания потоков
4. **Безопасность на уровне дизайна** — использует систему владения Rust

## Начало работы с Rayon

Добавь в `Cargo.toml`:

```toml
[dependencies]
rayon = "1.8"
```

### Базовый пример: Расчёт стоимости портфеля

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    current_price: f64,
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 10.0, current_price: 2500.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, current_price: 95.0 },
        Position { symbol: "AAPL".to_string(), quantity: 50.0, current_price: 185.0 },
        Position { symbol: "GOOGL".to_string(), quantity: 20.0, current_price: 140.0 },
    ];

    // Последовательный расчёт
    let sequential_total: f64 = portfolio
        .iter()
        .map(|pos| pos.quantity * pos.current_price)
        .sum();

    // Параллельный расчёт — просто меняем iter() на par_iter()!
    let parallel_total: f64 = portfolio
        .par_iter()
        .map(|pos| pos.quantity * pos.current_price)
        .sum();

    println!("Последовательно: ${:.2}", sequential_total);
    println!("Параллельно: ${:.2}", parallel_total);
}
```

## Методы параллельных итераторов

Rayon предоставляет параллельные версии большинства стандартных методов итераторов:

| Последовательный | Параллельный | Описание |
|-----------------|--------------|----------|
| `.iter()` | `.par_iter()` | Неизменяемая параллельная итерация |
| `.iter_mut()` | `.par_iter_mut()` | Изменяемая параллельная итерация |
| `.into_iter()` | `.into_par_iter()` | Потребляющая параллельная итерация |
| `.map()` | `.map()` | Преобразование элементов |
| `.filter()` | `.filter()` | Фильтрация элементов |
| `.reduce()` | `.reduce()` | Объединение элементов |
| `.sum()` | `.sum()` | Сумма числовых значений |
| `.for_each()` | `.for_each()` | Выполнение побочных эффектов |

## Пример из трейдинга: Анализ волатильности

Проанализируем волатильность нескольких активов параллельно:

```rust
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceHistory {
    symbol: String,
    prices: Vec<f64>,
}

impl PriceHistory {
    fn calculate_volatility(&self) -> f64 {
        if self.prices.len() < 2 {
            return 0.0;
        }

        // Рассчитываем дневные доходности
        let returns: Vec<f64> = self.prices
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        // Рассчитываем среднюю доходность
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        // Рассчитываем дисперсию
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        // Возвращаем стандартное отклонение (волатильность)
        variance.sqrt()
    }
}

fn main() {
    // Имитация истории цен для нескольких активов
    let assets = vec![
        PriceHistory {
            symbol: "BTC".to_string(),
            prices: vec![40000.0, 41000.0, 39500.0, 42000.0, 43500.0, 41000.0],
        },
        PriceHistory {
            symbol: "ETH".to_string(),
            prices: vec![2400.0, 2500.0, 2350.0, 2600.0, 2550.0, 2700.0],
        },
        PriceHistory {
            symbol: "SOL".to_string(),
            prices: vec![90.0, 95.0, 88.0, 102.0, 98.0, 105.0],
        },
        PriceHistory {
            symbol: "AAPL".to_string(),
            prices: vec![180.0, 182.0, 179.0, 185.0, 183.0, 186.0],
        },
    ];

    // Рассчитываем волатильность для всех активов параллельно
    let volatilities: HashMap<String, f64> = assets
        .par_iter()
        .map(|asset| {
            let vol = asset.calculate_volatility();
            println!("Рассчитана волатильность для {}: {:.4}", asset.symbol, vol);
            (asset.symbol.clone(), vol)
        })
        .collect();

    // Находим самый волатильный актив
    let most_volatile = volatilities
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap());

    if let Some((symbol, vol)) = most_volatile {
        println!("\nСамый волатильный актив: {} с волатильностью {:.4}", symbol, vol);
    }
}
```

## Параллельный Reduce: Расчёт риска портфеля

При объединении параллельных результатов используй `reduce()` с нейтральным элементом:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    pnl: f64,         // Прибыль/Убыток
    risk_score: f64,  // Риск от 0 до 1
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".to_string(), pnl: 1500.0, risk_score: 0.8 },
        Trade { symbol: "ETH".to_string(), pnl: -200.0, risk_score: 0.6 },
        Trade { symbol: "SOL".to_string(), pnl: 800.0, risk_score: 0.9 },
        Trade { symbol: "AAPL".to_string(), pnl: 300.0, risk_score: 0.3 },
        Trade { symbol: "GOOGL".to_string(), pnl: -100.0, risk_score: 0.4 },
    ];

    // Рассчитываем общий P&L и средний риск параллельно
    let (total_pnl, total_risk, count) = trades
        .par_iter()
        .map(|trade| (trade.pnl, trade.risk_score, 1))
        .reduce(
            || (0.0, 0.0, 0),  // Нейтральный элемент
            |acc, item| (acc.0 + item.0, acc.1 + item.1, acc.2 + item.2),
        );

    let avg_risk = if count > 0 { total_risk / count as f64 } else { 0.0 };

    println!("Общий P&L: ${:.2}", total_pnl);
    println!("Средний Risk Score: {:.2}", avg_risk);
    println!("Количество сделок: {}", count);
}
```

## Фильтрация и сбор: Поиск возможностей

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct MarketSignal {
    symbol: String,
    signal_strength: f64,  // -1.0 (сильная продажа) до 1.0 (сильная покупка)
    volume: u64,
    price: f64,
}

fn main() {
    let signals = vec![
        MarketSignal { symbol: "BTC".to_string(), signal_strength: 0.85, volume: 1_000_000, price: 42000.0 },
        MarketSignal { symbol: "ETH".to_string(), signal_strength: 0.25, volume: 500_000, price: 2500.0 },
        MarketSignal { symbol: "SOL".to_string(), signal_strength: 0.92, volume: 200_000, price: 95.0 },
        MarketSignal { symbol: "DOGE".to_string(), signal_strength: -0.3, volume: 10_000, price: 0.08 },
        MarketSignal { symbol: "AAPL".to_string(), signal_strength: 0.45, volume: 2_000_000, price: 185.0 },
        MarketSignal { symbol: "GME".to_string(), signal_strength: 0.15, volume: 50_000, price: 25.0 },
    ];

    // Ищем сильные сигналы на покупку с высоким объёмом параллельно
    let opportunities: Vec<_> = signals
        .par_iter()
        .filter(|s| s.signal_strength > 0.5 && s.volume > 100_000)
        .map(|s| (s.symbol.clone(), s.signal_strength, s.price))
        .collect();

    println!("Сильные сигналы на покупку:");
    for (symbol, strength, price) in &opportunities {
        println!("  {} - Сигнал: {:.2}, Цена: ${:.2}", symbol, strength, price);
    }
}
```

## Параллельная сортировка

Rayon предоставляет эффективную параллельную сортировку:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

fn main() {
    let mut orders = vec![
        Order { id: 1, price: 42100.0, quantity: 0.5, timestamp: 1000 },
        Order { id: 2, price: 42050.0, quantity: 1.0, timestamp: 1001 },
        Order { id: 3, price: 42200.0, quantity: 0.3, timestamp: 1002 },
        Order { id: 4, price: 41900.0, quantity: 2.0, timestamp: 1003 },
        Order { id: 5, price: 42000.0, quantity: 0.8, timestamp: 1004 },
    ];

    // Сортируем ордера по цене (по возрастанию) параллельно
    orders.par_sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

    println!("Ордера отсортированы по цене:");
    for order in &orders {
        println!("  ID: {}, Цена: ${:.2}, Кол-во: {}", order.id, order.price, order.quantity);
    }

    // Сортируем по количеству (по убыванию) параллельно
    orders.par_sort_by(|a, b| b.quantity.partial_cmp(&a.quantity).unwrap());

    println!("\nОрдера отсортированы по количеству (убыв.):");
    for order in &orders {
        println!("  ID: {}, Цена: ${:.2}, Кол-во: {}", order.id, order.price, order.quantity);
    }
}
```

## Параллельная обработка: Бэктестинг нескольких стратегий

```rust
use rayon::prelude::*;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    buy_threshold: f64,
    sell_threshold: f64,
}

#[derive(Debug, Clone)]
struct BacktestResult {
    strategy_name: String,
    total_return: f64,
    max_drawdown: f64,
    win_rate: f64,
}

fn backtest_strategy(strategy: &Strategy, prices: &[f64]) -> BacktestResult {
    let mut position = false;
    let mut entry_price = 0.0;
    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut total_trades = 0;
    let mut peak_equity = 0.0;
    let mut max_drawdown = 0.0;
    let mut equity = 10000.0;

    for i in 1..prices.len() {
        let change = (prices[i] - prices[i - 1]) / prices[i - 1];

        if !position && change > strategy.buy_threshold {
            position = true;
            entry_price = prices[i];
        } else if position && change < -strategy.sell_threshold {
            position = false;
            let pnl = (prices[i] - entry_price) / entry_price;
            total_pnl += pnl;
            equity *= 1.0 + pnl;
            total_trades += 1;
            if pnl > 0.0 {
                wins += 1;
            }
        }

        peak_equity = peak_equity.max(equity);
        let drawdown = (peak_equity - equity) / peak_equity;
        max_drawdown = max_drawdown.max(drawdown);
    }

    BacktestResult {
        strategy_name: strategy.name.clone(),
        total_return: total_pnl * 100.0,
        max_drawdown: max_drawdown * 100.0,
        win_rate: if total_trades > 0 {
            (wins as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        },
    }
}

fn main() {
    // Генерируем тестовые ценовые данные
    let prices: Vec<f64> = (0..10000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0 + (i as f64 * 0.001) * 5.0)
        .collect();

    // Определяем множество стратегий для тестирования
    let strategies: Vec<Strategy> = (1..=50)
        .flat_map(|i| {
            (1..=10).map(move |j| Strategy {
                name: format!("Strategy_{}_{}", i, j),
                buy_threshold: 0.001 * i as f64,
                sell_threshold: 0.001 * j as f64,
            })
        })
        .collect();

    println!("Тестируем {} стратегий...", strategies.len());

    // Последовательный бэктестинг
    let start = Instant::now();
    let _sequential_results: Vec<BacktestResult> = strategies
        .iter()
        .map(|s| backtest_strategy(s, &prices))
        .collect();
    let sequential_time = start.elapsed();

    // Параллельный бэктестинг
    let start = Instant::now();
    let parallel_results: Vec<BacktestResult> = strategies
        .par_iter()
        .map(|s| backtest_strategy(s, &prices))
        .collect();
    let parallel_time = start.elapsed();

    println!("Последовательное время: {:?}", sequential_time);
    println!("Параллельное время: {:?}", parallel_time);
    println!(
        "Ускорение: {:.2}x",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    // Находим лучшую стратегию
    let best = parallel_results
        .iter()
        .max_by(|a, b| a.total_return.partial_cmp(&b.total_return).unwrap());

    if let Some(result) = best {
        println!("\nЛучшая стратегия: {}", result.strategy_name);
        println!("  Общая доходность: {:.2}%", result.total_return);
        println!("  Макс. просадка: {:.2}%", result.max_drawdown);
        println!("  Процент выигрышных: {:.2}%", result.win_rate);
    }
}
```

## Настройка пула потоков

Rayon использует глобальный пул потоков по умолчанию, но его можно настроить:

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Настраиваем свой пул потоков
    ThreadPoolBuilder::new()
        .num_threads(4)  // Используем 4 потока
        .build_global()
        .expect("Не удалось создать пул потоков");

    let numbers: Vec<i32> = (1..=100).collect();

    // Это будет использовать настроенный 4-поточный пул
    let sum: i32 = numbers.par_iter().sum();
    println!("Сумма: {}", sum);
}
```

### Использование локального пула потоков

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();

    let prices = vec![100.0, 105.0, 103.0, 110.0, 108.0];

    // Выполняем в конкретном пуле
    let result = pool.install(|| {
        prices
            .par_iter()
            .map(|p| p * 1.1)  // 10% наценка
            .sum::<f64>()
    });

    println!("Итого с наценкой: ${:.2}", result);
}
```

## Когда НЕ использовать Rayon

Параллельная обработка имеет накладные расходы. Избегай rayon когда:

1. **Маленькие наборы данных** — накладные расходы на создание потоков превышают пользу
2. **Простые операции** — накладные расходы на параллелизм могут быть больше самой работы
3. **I/O-ограниченные задачи** — используй async вместо этого
4. **Последовательные зависимости** — операции, которые должны выполняться по порядку

```rust
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let small_data: Vec<i32> = (1..100).collect();
    let large_data: Vec<i32> = (1..1_000_000).collect();

    // Маленький набор данных — последовательный может быть быстрее
    let start = Instant::now();
    let _: i32 = small_data.iter().map(|x| x * 2).sum();
    let seq_small = start.elapsed();

    let start = Instant::now();
    let _: i32 = small_data.par_iter().map(|x| x * 2).sum();
    let par_small = start.elapsed();

    println!("Маленький набор данных (100 элементов):");
    println!("  Последовательно: {:?}", seq_small);
    println!("  Параллельно: {:?}", par_small);

    // Большой набор данных — параллельный быстрее
    let start = Instant::now();
    let _: i32 = large_data.iter().map(|x| x * 2).sum();
    let seq_large = start.elapsed();

    let start = Instant::now();
    let _: i32 = large_data.par_iter().map(|x| x * 2).sum();
    let par_large = start.elapsed();

    println!("\nБольшой набор данных (1M элементов):");
    println!("  Последовательно: {:?}", seq_large);
    println!("  Параллельно: {:?}", par_large);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `rayon` | Библиотека для параллелизма данных в Rust |
| `par_iter()` | Параллельный неизменяемый итератор |
| `par_iter_mut()` | Параллельный изменяемый итератор |
| `into_par_iter()` | Параллельный потребляющий итератор |
| Work stealing | Автоматическое распределение нагрузки между потоками |
| `reduce()` | Объединение параллельных результатов с нейтральным элементом |
| `par_sort()` | Параллельная сортировка |
| `ThreadPoolBuilder` | Настройка пула потоков |

## Практические задания

1. **Анализ цен**: Создай функцию, которая принимает вектор из 10 000 ценовых точек и использует rayon для расчёта скользящего среднего с окном 20. Сравни производительность с последовательной версией.

2. **Мультиактивный фильтр**: Имея список из 1000 активов с их дневными доходностями, используй параллельные итераторы для фильтрации тех, у которых доходность > 5% и объём > 1 миллиона.

3. **Оптимизация портфеля**: Реализуй параллельный расчёт коэффициента Шарпа для множества комбинаций активов в портфеле.

4. **Агрегация стакана заявок**: Имея стаканы заявок с 10 разных бирж, используй rayon для параллельной агрегации bid/ask цен и поиска лучшей цены исполнения.

## Домашнее задание

1. **Параллельные технические индикаторы**: Реализуй систему, которая рассчитывает несколько технических индикаторов (RSI, MACD, полосы Боллинджера) для списка из 100 активов параллельно. Измерь ускорение по сравнению с последовательной обработкой.

2. **Grid Search стратегий**: Создай параллельный grid search, который тестирует 1000 различных комбинаций параметров для простой торговой стратегии. Используй `into_par_iter()` для обработки всех комбинаций и поиска оптимальных параметров.

3. **Калькулятор риска**: Построй калькулятор Value at Risk (VaR), который запускает 10 000 симуляций Монте-Карло параллельно для оценки риска портфеля. Сравни время выполнения с последовательной реализацией.

4. **Сканер арбитража между биржами**: Реализуй сканер, который проверяет арбитражные возможности на 5 биржах для 100 торговых пар одновременно. Используй rayon для параллелизации как по биржам, так и по торговым парам.

## Навигация

[← Предыдущий день](../172-crossbeam-scope-threads-borrowing/ru.md) | [Следующий день →](../174-par-iter-parallel-trade-processing/ru.md)
