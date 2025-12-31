# День 161: Arc: общий доступ между потоками

## Аналогия из трейдинга

Представьте торговую платформу с несколькими модулями:
- Модуль анализа цен
- Модуль управления рисками
- Модуль исполнения ордеров
- Модуль мониторинга

Все эти модули должны иметь доступ к **одним и тем же рыночным данным** — текущей цене, истории котировок, состоянию книги ордеров. При этом каждый модуль работает в своём потоке для максимальной производительности.

В Rust обычные ссылки (`&T`) не могут пережить поток, который их создал. Для безопасного совместного владения данными между потоками нам нужен **Arc** — Atomic Reference Counted (атомарный подсчёт ссылок).

## Проблема: данные между потоками

```rust
use std::thread;

fn main() {
    let market_data = vec![42000.0, 42100.0, 42050.0];

    // ОШИБКА! market_data не может быть использован в другом потоке
    // let handle = thread::spawn(|| {
    //     println!("Price: {}", market_data[0]);
    // });

    // Это не скомпилируется: `market_data` не реализует `Send`
}
```

## Решение с Arc

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // Оборачиваем данные в Arc
    let market_data = Arc::new(vec![42000.0, 42100.0, 42050.0]);

    // Создаём клон Arc (не данных!) для каждого потока
    let data_for_analyzer = Arc::clone(&market_data);
    let data_for_risk = Arc::clone(&market_data);

    let analyzer = thread::spawn(move || {
        let avg: f64 = data_for_analyzer.iter().sum::<f64>()
            / data_for_analyzer.len() as f64;
        println!("Analyzer: Average price = ${:.2}", avg);
    });

    let risk_manager = thread::spawn(move || {
        let max = data_for_risk.iter().cloned().fold(0.0_f64, f64::max);
        let min = data_for_risk.iter().cloned().fold(f64::MAX, f64::min);
        println!("Risk: Range = ${:.2} - ${:.2}", min, max);
    });

    // Ждём завершения потоков
    analyzer.join().unwrap();
    risk_manager.join().unwrap();

    // Оригинальный Arc всё ещё доступен
    println!("Main: {} prices available", market_data.len());
}
```

Вывод:
```
Analyzer: Average price = $42050.00
Risk: Range = $42000.00 - $42100.00
Main: 3 prices available
```

## Как работает Arc

```rust
use std::sync::Arc;

fn main() {
    let price = Arc::new(42000.0_f64);
    println!("Reference count: {}", Arc::strong_count(&price)); // 1

    let price2 = Arc::clone(&price);
    println!("Reference count: {}", Arc::strong_count(&price)); // 2

    {
        let price3 = Arc::clone(&price);
        println!("Reference count: {}", Arc::strong_count(&price)); // 3
    } // price3 выходит из области видимости

    println!("Reference count: {}", Arc::strong_count(&price)); // 2

    // Данные освобождаются только когда счётчик = 0
}
```

## Arc vs Rc

| Характеристика | `Rc<T>` | `Arc<T>` |
|----------------|---------|----------|
| Потокобезопасность | Нет | Да |
| Производительность | Быстрее | Медленнее (атомарные операции) |
| Использование | Один поток | Несколько потоков |
| Trait | `!Send`, `!Sync` | `Send + Sync` (если T: Send + Sync) |

## Практический пример: мониторинг нескольких бирж

```rust
use std::sync::Arc;
use std::thread;
use std::collections::HashMap;

#[derive(Debug)]
struct ExchangeData {
    name: String,
    btc_price: f64,
    eth_price: f64,
    volume_24h: f64,
}

fn main() {
    // Данные с нескольких бирж
    let exchanges = Arc::new(vec![
        ExchangeData {
            name: "Binance".to_string(),
            btc_price: 42000.0,
            eth_price: 2500.0,
            volume_24h: 1_000_000.0,
        },
        ExchangeData {
            name: "Coinbase".to_string(),
            btc_price: 42050.0,
            eth_price: 2510.0,
            volume_24h: 500_000.0,
        },
        ExchangeData {
            name: "Kraken".to_string(),
            btc_price: 41980.0,
            eth_price: 2495.0,
            volume_24h: 300_000.0,
        },
    ]);

    // Модуль 1: Поиск лучшей цены BTC
    let data_for_btc = Arc::clone(&exchanges);
    let btc_analyzer = thread::spawn(move || {
        let best = data_for_btc
            .iter()
            .min_by(|a, b| a.btc_price.partial_cmp(&b.btc_price).unwrap())
            .unwrap();
        println!("Best BTC price: {} @ ${:.2}", best.name, best.btc_price);
        (best.name.clone(), best.btc_price)
    });

    // Модуль 2: Поиск лучшей цены ETH
    let data_for_eth = Arc::clone(&exchanges);
    let eth_analyzer = thread::spawn(move || {
        let best = data_for_eth
            .iter()
            .min_by(|a, b| a.eth_price.partial_cmp(&b.eth_price).unwrap())
            .unwrap();
        println!("Best ETH price: {} @ ${:.2}", best.name, best.eth_price);
        (best.name.clone(), best.eth_price)
    });

    // Модуль 3: Анализ ликвидности
    let data_for_volume = Arc::clone(&exchanges);
    let volume_analyzer = thread::spawn(move || {
        let total: f64 = data_for_volume.iter().map(|e| e.volume_24h).sum();
        let by_exchange: HashMap<String, f64> = data_for_volume
            .iter()
            .map(|e| (e.name.clone(), e.volume_24h / total * 100.0))
            .collect();
        println!("Total 24h volume: ${:.0}", total);
        for (name, share) in &by_exchange {
            println!("  {}: {:.1}%", name, share);
        }
        by_exchange
    });

    // Собираем результаты
    let best_btc = btc_analyzer.join().unwrap();
    let best_eth = eth_analyzer.join().unwrap();
    let volume_shares = volume_analyzer.join().unwrap();

    println!("\n=== SUMMARY ===");
    println!("Buy BTC on {} @ ${:.2}", best_btc.0, best_btc.1);
    println!("Buy ETH on {} @ ${:.2}", best_eth.0, best_eth.1);
}
```

## Arc со структурами

```rust
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct Portfolio {
    assets: Vec<(String, f64, f64)>, // (symbol, quantity, price)
}

impl Portfolio {
    fn total_value(&self) -> f64 {
        self.assets.iter().map(|(_, qty, price)| qty * price).sum()
    }

    fn position_values(&self) -> Vec<(String, f64)> {
        self.assets
            .iter()
            .map(|(sym, qty, price)| (sym.clone(), qty * price))
            .collect()
    }
}

fn main() {
    let portfolio = Arc::new(Portfolio {
        assets: vec![
            ("BTC".to_string(), 0.5, 42000.0),
            ("ETH".to_string(), 5.0, 2500.0),
            ("SOL".to_string(), 100.0, 95.0),
        ],
    });

    // Поток 1: Расчёт общей стоимости
    let p1 = Arc::clone(&portfolio);
    let total_handle = thread::spawn(move || {
        let total = p1.total_value();
        println!("Portfolio total: ${:.2}", total);
        total
    });

    // Поток 2: Анализ позиций
    let p2 = Arc::clone(&portfolio);
    let positions_handle = thread::spawn(move || {
        let positions = p2.position_values();
        for (sym, value) in &positions {
            println!("{}: ${:.2}", sym, value);
        }
        positions
    });

    let total = total_handle.join().unwrap();
    let positions = positions_handle.join().unwrap();

    // Расчёт долей
    println!("\n--- Allocation ---");
    for (sym, value) in positions {
        println!("{}: {:.1}%", sym, value / total * 100.0);
    }
}
```

## Arc с большими данными

Arc особенно полезен для больших структур данных, которые дорого клонировать:

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // Большой объём исторических данных
    let historical_prices: Arc<Vec<f64>> = Arc::new(
        (0..1_000_000)
            .map(|i| 40000.0 + (i as f64 * 0.001).sin() * 2000.0)
            .collect()
    );

    println!("Historical data size: {} prices", historical_prices.len());

    let mut handles = vec![];

    // Запускаем 4 потока анализа
    for thread_id in 0..4 {
        let data = Arc::clone(&historical_prices);
        let chunk_size = data.len() / 4;
        let start = thread_id * chunk_size;
        let end = if thread_id == 3 { data.len() } else { start + chunk_size };

        let handle = thread::spawn(move || {
            let chunk = &data[start..end];
            let avg: f64 = chunk.iter().sum::<f64>() / chunk.len() as f64;
            let max = chunk.iter().cloned().fold(0.0_f64, f64::max);
            let min = chunk.iter().cloned().fold(f64::MAX, f64::min);

            println!(
                "Thread {}: chunk [{}-{}], avg=${:.2}, range=${:.2}-${:.2}",
                thread_id, start, end, avg, min, max
            );

            (avg, min, max)
        });

        handles.push(handle);
    }

    // Собираем результаты
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let global_avg = results.iter().map(|(avg, _, _)| avg).sum::<f64>() / 4.0;
    let global_min = results.iter().map(|(_, min, _)| *min).fold(f64::MAX, f64::min);
    let global_max = results.iter().map(|(_, _, max)| *max).fold(0.0_f64, f64::max);

    println!("\n=== GLOBAL STATS ===");
    println!("Average: ${:.2}", global_avg);
    println!("Range: ${:.2} - ${:.2}", global_min, global_max);
}
```

## Важно: Arc даёт только иммутабельный доступ

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let price = Arc::new(42000.0_f64);
    let price_clone = Arc::clone(&price);

    let handle = thread::spawn(move || {
        // Можем читать
        println!("Price: ${}", *price_clone);

        // НЕ можем изменять!
        // *price_clone = 43000.0; // ОШИБКА!
    });

    handle.join().unwrap();

    // Для изменения нужен Arc<Mutex<T>> - об этом в следующей главе
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Arc<T>` | Atomic Reference Counted — потокобезопасный умный указатель |
| `Arc::new(data)` | Создание Arc с данными |
| `Arc::clone(&arc)` | Создание нового указателя (не клона данных!) |
| `Arc::strong_count()` | Текущее количество ссылок |
| `*arc` | Доступ к данным (только чтение) |
| `Arc` vs `Rc` | Arc для многопоточности, Rc для однопоточности |

## Практические задания

1. **Мульти-анализатор цен**: Создайте программу, которая запускает 3 потока для параллельного расчёта SMA (простое скользящее среднее), EMA (экспоненциальное) и RSI по одним и тем же историческим данным.

2. **Портфельный трекер**: Реализуйте систему, где несколько потоков параллельно рассчитывают метрики портфеля: общую стоимость, нереализованный P&L, exposure по секторам.

3. **Агрегатор ордербуков**: Напишите программу, которая получает данные книги ордеров с нескольких бирж и в параллельных потоках находит лучший bid/ask и рассчитывает спред.

## Домашнее задание

1. Реализуйте многопоточный калькулятор волатильности:
   - Загрузите исторические данные в `Arc<Vec<f64>>`
   - В одном потоке рассчитайте дневную волатильность
   - В другом потоке рассчитайте недельную волатильность
   - В третьем потоке найдите максимальную просадку

2. Создайте систему мониторинга портфеля:
   - Храните данные портфеля в `Arc<Portfolio>`
   - Запустите поток расчёта VaR (Value at Risk)
   - Запустите поток расчёта Sharpe Ratio
   - Запустите поток проверки лимитов

3. Напишите арбитражный сканер:
   - Цены с бирж храните в `Arc<HashMap<String, ExchangeData>>`
   - Параллельно ищите арбитражные возможности между парами бирж
   - Выводите найденные спреды и потенциальную прибыль

4. Реализуйте бэктестер с параллельным анализом:
   - Исторические данные в `Arc<Vec<Candle>>`
   - Запустите несколько потоков с разными параметрами стратегии
   - Соберите результаты и найдите оптимальные параметры

## Навигация

[← Предыдущий день](../160-mutex-one-trader/ru.md) | [Следующий день →](../162-arc-mutex-shared-mutable/ru.md)
