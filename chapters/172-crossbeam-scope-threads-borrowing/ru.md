# День 172: crossbeam scope: потоки с заимствованием

## Аналогия из трейдинга

Представь, что ты управляющий торгового зала, и тебе нужно срочно разослать команду аналитикам: "Проанализируйте этот список активов". С обычными потоками (`std::thread::spawn`) ты должен был бы **скопировать** список для каждого аналитика — а это дорого при большом портфеле.

С `crossbeam::scope` ты можешь просто **показать** аналитикам список на экране — они все посмотрят на один и тот же экран, сделают свою работу, и только когда ВСЕ закончат, ты переключишь экран на другую задачу. Никакого копирования, полная безопасность — никто не уйдёт с совещания, пока все не закончат.

В мире потоков это решает главную проблему: как позволить потокам **заимствовать** локальные переменные со стека, а не требовать `'static` время жизни.

## Проблема стандартных потоков

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0];

    // ЭТО НЕ СКОМПИЛИРУЕТСЯ!
    let handle = thread::spawn(|| {
        // Ошибка: `prices` не живёт достаточно долго
        println!("Первая цена: {}", prices[0]);
    });

    handle.join().unwrap();
}
```

Компилятор справедливо ругается: он не может гарантировать, что `prices` будет существовать, пока поток работает. Вдруг поток переживёт main()?

Классические решения:
- `move` — переместить владение в поток (но тогда нельзя использовать в других потоках)
- `Arc::clone()` — клонировать умный указатель (накладные расходы)

## crossbeam::scope — элегантное решение

```rust
use crossbeam::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0];
    let volumes = vec![100, 250, 150, 300];

    // Область видимости (scope) гарантирует:
    // все потоки завершатся ДО выхода из блока
    thread::scope(|s| {
        // Поток 1: анализирует цены
        s.spawn(|_| {
            let avg_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
            println!("Средняя цена: ${:.2}", avg_price);
        });

        // Поток 2: анализирует объёмы
        s.spawn(|_| {
            let total_volume: i32 = volumes.iter().sum();
            println!("Общий объём: {} лотов", total_volume);
        });

        // Поток 3: использует ОБА массива
        s.spawn(|_| {
            let weighted_price: f64 = prices.iter()
                .zip(volumes.iter())
                .map(|(p, v)| p * *v as f64)
                .sum::<f64>() / volumes.iter().sum::<i32>() as f64;
            println!("Средневзвешенная цена: ${:.2}", weighted_price);
        });

    }).unwrap(); // Все потоки завершились

    // prices и volumes всё ещё доступны здесь!
    println!("Всего цен в анализе: {}", prices.len());
}
```

## Как это работает

```
main()                          scope
  |                               |
  v                               v
prices: [...]  ─────────────────> s.spawn() видит prices
volumes: [...]  ────────────────> s.spawn() видит volumes
  |                               |
  |   +-- Thread 1 ──────────────>|
  |   +-- Thread 2 ──────────────>|
  |   +-- Thread 3 ──────────────>|
  |                               |
  |<────── ВСЕ потоки join ───────+
  |
  v
prices и volumes всё ещё живы!
```

Ключевой момент: `scope` гарантирует, что все порождённые потоки завершатся **до** возврата из scope. Это позволяет компилятору быть уверенным, что заимствованные данные живут достаточно долго.

## Практический пример: Параллельный анализ портфеля

```rust
use crossbeam::thread;
use std::collections::HashMap;

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.avg_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price / self.avg_price) - 1.0) * 100.0
    }
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, avg_price: 40000.0, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 15.0, avg_price: 2800.0, current_price: 2650.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, avg_price: 95.0, current_price: 110.0 },
        Position { symbol: "DOGE".to_string(), quantity: 50000.0, avg_price: 0.08, current_price: 0.09 },
    ];

    // Используем scope для параллельного анализа
    let results = thread::scope(|s| {
        // Поток 1: Считаем общий PnL
        let pnl_handle = s.spawn(|_| {
            let total_pnl: f64 = portfolio.iter()
                .map(|p| p.pnl())
                .sum();
            total_pnl
        });

        // Поток 2: Находим лучшую позицию
        let best_handle = s.spawn(|_| {
            portfolio.iter()
                .max_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        // Поток 3: Находим худшую позицию
        let worst_handle = s.spawn(|_| {
            portfolio.iter()
                .min_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        // Поток 4: Считаем общую стоимость портфеля
        let value_handle = s.spawn(|_| {
            portfolio.iter()
                .map(|p| p.current_price * p.quantity)
                .sum::<f64>()
        });

        // Собираем результаты (все join происходят внутри scope)
        (
            pnl_handle.join().unwrap(),
            best_handle.join().unwrap(),
            worst_handle.join().unwrap(),
            value_handle.join().unwrap(),
        )
    }).unwrap();

    println!("=== Анализ портфеля ===");
    println!("Общий PnL: ${:.2}", results.0);
    if let Some((symbol, pct)) = results.1 {
        println!("Лучшая позиция: {} ({:+.2}%)", symbol, pct);
    }
    if let Some((symbol, pct)) = results.2 {
        println!("Худшая позиция: {} ({:+.2}%)", symbol, pct);
    }
    println!("Стоимость портфеля: ${:.2}", results.3);
}
```

## Вложенные потоки (Nested Spawning)

Иногда потоку нужно создать дополнительные потоки. Для этого можно использовать вложенные scope — внутренний scope будет иметь доступ к данным из внешнего контекста:

```rust
use crossbeam::thread;

fn main() {
    let exchanges = vec!["Binance", "Kraken", "Coinbase"];
    let symbols = vec!["BTC", "ETH", "SOL"];

    // Внешний scope для основных задач
    thread::scope(|s| {
        // Первый поток: анализатор
        s.spawn(|_| {
            println!("Анализатор запущен");

            // Вложенный scope для параллельных запросов к биржам
            thread::scope(|inner_s| {
                for exchange in &exchanges {
                    for symbol in &symbols {
                        inner_s.spawn(move |_| {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            println!("[{}] {} = $42000.00", exchange, symbol);
                        });
                    }
                }
            }).unwrap();

            println!("Все данные от бирж получены");
        });

        // Второй поток: мониторинг (работает параллельно с первым)
        s.spawn(|_| {
            println!("Мониторим {} бирж с {} символами",
                     exchanges.len(), symbols.len());
        });
    }).unwrap();

    println!("Все данные собраны!");
}
```

## Сравнение с std::thread::scope

Начиная с Rust 1.63, в стандартной библиотеке появился `std::thread::scope`. Он похож на crossbeam, но есть различия:

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0];

    // std::thread::scope (Rust 1.63+)
    thread::scope(|s| {
        s.spawn(|| {
            println!("Цена: {}", prices[0]);
        });
    });

    println!("После scope: {:?}", prices);
}
```

| Особенность | crossbeam::scope | std::thread::scope |
|-------------|------------------|-------------------|
| Доступность | Любая версия Rust | Rust 1.63+ |
| Вложенные потоки | Удобнее (параметр scope) | Возможно, но менее удобно |
| Возврат значений | Через join() | Через join() |
| Panic handling | unwrap() на результате | Паника пробрасывается |
| Производительность | Оптимизирован | Стандартная |

## Обработка ошибок в scope

```rust
use crossbeam::thread;

fn main() {
    let orders = vec![
        ("BTC", 1.0, 42000.0),
        ("ETH", 10.0, 2800.0),
        ("INVALID", -1.0, 0.0), // Ошибочный ордер
    ];

    let result = thread::scope(|s| {
        let handles: Vec<_> = orders.iter()
            .map(|(symbol, qty, price)| {
                s.spawn(move |_| {
                    if *qty <= 0.0 {
                        Err(format!("Неверное количество для {}", symbol))
                    } else {
                        Ok(format!("{}: {} x ${}", symbol, qty, price))
                    }
                })
            })
            .collect();

        // Собираем результаты
        handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    match result {
        Ok(results) => {
            for r in results {
                match r {
                    Ok(msg) => println!("Успех: {}", msg),
                    Err(e) => println!("Ошибка: {}", e),
                }
            }
        }
        Err(e) => println!("Panic в потоке: {:?}", e),
    }
}
```

## Пример: Параллельный расчёт технических индикаторов

```rust
use crossbeam::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    prices.windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = vec![prices[0]];

    for price in &prices[1..] {
        let new_ema = (price - ema.last().unwrap()) * multiplier + ema.last().unwrap();
        ema.push(new_ema);
    }

    ema
}

fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let changes: Vec<f64> = prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let mut rsi = Vec::new();

    for i in period..changes.len() {
        let window = &changes[i - period..i];
        let gains: f64 = window.iter().filter(|&&x| x > 0.0).sum();
        let losses: f64 = window.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum();

        let rs = if losses == 0.0 { 100.0 } else { gains / losses };
        let rsi_value = 100.0 - (100.0 / (1.0 + rs));
        rsi.push(rsi_value);
    }

    rsi
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let indicators = thread::scope(|s| {
        let sma_handle = s.spawn(|_| {
            ("SMA(5)", calculate_sma(&prices, 5))
        });

        let ema_handle = s.spawn(|_| {
            ("EMA(5)", calculate_ema(&prices, 5))
        });

        let rsi_handle = s.spawn(|_| {
            ("RSI(14)", calculate_rsi(&prices, 14))
        });

        vec![
            sma_handle.join().unwrap(),
            ema_handle.join().unwrap(),
            rsi_handle.join().unwrap(),
        ]
    }).unwrap();

    println!("=== Технические индикаторы ===");
    for (name, values) in indicators {
        if let Some(last) = values.last() {
            println!("{}: {:.2}", name, last);
        } else {
            println!("{}: недостаточно данных", name);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `crossbeam::scope` | Создаёт область, где потоки могут заимствовать локальные данные |
| Гарантия завершения | Все потоки гарантированно завершатся до выхода из scope |
| Заимствование | Потоки могут заимствовать `&T` и `&mut T` со стека |
| Вложенные потоки | Параметр scope передаётся для создания дочерних потоков |
| Возврат значений | `s.spawn().join()` возвращает результат потока |
| std::thread::scope | Аналог в стандартной библиотеке (Rust 1.63+) |

## Домашнее задание

1. **Параллельный анализ свечей**: Напиши программу, которая:
   - Имеет вектор OHLCV данных (Open, High, Low, Close, Volume)
   - Запускает 4 потока, каждый считает: среднее Open, максимальный High, минимальный Low, сумму Volume
   - Использует `crossbeam::scope` для заимствования данных

2. **Мониторинг нескольких портфелей**: Создай структуру `Portfolio` и вектор портфелей. Используя scope:
   - Запусти отдельный поток для каждого портфеля
   - Каждый поток вычисляет: общую стоимость, PnL, количество позиций
   - Соберите результаты и выведите сводную таблицу

3. **Параллельный бэктест**: Имея исторические данные:
   - Раздели их на N частей
   - Запусти N потоков, каждый тестирует стратегию на своей части
   - Объедини результаты в общую статистику

4. **Сравнение производительности**: Напиши бенчмарк:
   - Один и тот же расчёт последовательно
   - С использованием `crossbeam::scope`
   - С использованием `std::thread::scope`
   - Сравни время выполнения для разного количества данных

## Навигация

[← Предыдущий день](../171-crossbeam-channels-faster-mpsc/ru.md) | [Следующий день →](../173-rayon-parallel-iterators/ru.md)
