# День 154: join: ждём завершения потока

## Аналогия из трейдинга

Представь, что ты отправил несколько аналитиков собирать данные с разных бирж. Один анализирует Binance, другой — Coinbase, третий — Kraken. Ты не можешь принять торговое решение, пока все аналитики не вернутся с результатами. Метод `join()` — это как ожидание возвращения каждого аналитика: ты блокируешь свою работу до тех пор, пока конкретный поток не завершит свою задачу.

**Без join:** Ты принимаешь решение на основе неполных данных — опасно!
**С join:** Ты ждёшь всех аналитиков и принимаешь взвешенное решение.

## Что такое join?

Когда мы создаём поток с помощью `thread::spawn()`, он возвращает `JoinHandle<T>`. Это "ручка" для управления потоком:

```rust
use std::thread;

fn main() {
    // spawn возвращает JoinHandle
    let handle = thread::spawn(|| {
        println!("Работаю в потоке!");
        42  // Возвращаемое значение
    });

    // join() блокирует до завершения потока и возвращает Result
    let result = handle.join();

    match result {
        Ok(value) => println!("Поток вернул: {}", value),
        Err(_) => println!("Поток завершился с паникой!"),
    }
}
```

## Базовый пример: получение цен с бирж

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Сбор цен с бирж ===\n");

    // Запускаем потоки для получения цен
    let binance_handle = thread::spawn(|| {
        println!("[Binance] Подключаюсь...");
        thread::sleep(Duration::from_millis(100));
        let price = 42150.50;
        println!("[Binance] Цена BTC: ${:.2}", price);
        price
    });

    let coinbase_handle = thread::spawn(|| {
        println!("[Coinbase] Подключаюсь...");
        thread::sleep(Duration::from_millis(150));
        let price = 42148.75;
        println!("[Coinbase] Цена BTC: ${:.2}", price);
        price
    });

    let kraken_handle = thread::spawn(|| {
        println!("[Kraken] Подключаюсь...");
        thread::sleep(Duration::from_millis(80));
        let price = 42152.00;
        println!("[Kraken] Цена BTC: ${:.2}", price);
        price
    });

    // Ждём завершения всех потоков
    let binance_price = binance_handle.join().expect("Binance поток упал");
    let coinbase_price = coinbase_handle.join().expect("Coinbase поток упал");
    let kraken_price = kraken_handle.join().expect("Kraken поток упал");

    // Теперь у нас есть все данные
    let avg_price = (binance_price + coinbase_price + kraken_price) / 3.0;
    println!("\n=== Результаты ===");
    println!("Средняя цена BTC: ${:.2}", avg_price);
}
```

## Почему join важен?

### Без join — программа может завершиться раньше потоков

```rust
use std::thread;
use std::time::Duration;

fn main() {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        println!("Это может НЕ напечататься!");
    });

    println!("Main завершается...");
    // Программа завершается, поток убивается!
}
```

### С join — гарантированное ожидание

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        println!("Это ТОЧНО напечатается!");
    });

    println!("Main ждёт...");
    handle.join().unwrap();
    println!("Поток завершился!");
}
```

## Обработка ошибок: Result от join

`join()` возвращает `Result<T, Box<dyn Any + Send>>`:
- `Ok(value)` — поток завершился успешно и вернул значение
- `Err(panic_info)` — поток запаниковал

```rust
use std::thread;

fn main() {
    // Поток с успешным завершением
    let success_handle = thread::spawn(|| {
        "Успех!"
    });

    // Поток с паникой
    let panic_handle = thread::spawn(|| {
        panic!("Что-то пошло не так!");
    });

    // Обработка результатов
    match success_handle.join() {
        Ok(msg) => println!("Успешный поток: {}", msg),
        Err(_) => println!("Поток упал!"),
    }

    match panic_handle.join() {
        Ok(_) => println!("Это не напечатается"),
        Err(_) => println!("Поток с паникой обработан безопасно"),
    }

    println!("Программа продолжает работать!");
}
```

## Практический пример: параллельный анализ портфеля

```rust
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct AssetAnalysis {
    symbol: String,
    price: f64,
    change_24h: f64,
    signal: String,
}

fn analyze_asset(symbol: &str, price: f64) -> AssetAnalysis {
    // Имитация анализа
    thread::sleep(Duration::from_millis(50));

    let change_24h = match symbol {
        "BTC" => 2.5,
        "ETH" => -1.2,
        "SOL" => 5.8,
        _ => 0.0,
    };

    let signal = if change_24h > 3.0 {
        "STRONG BUY"
    } else if change_24h > 0.0 {
        "BUY"
    } else if change_24h > -3.0 {
        "HOLD"
    } else {
        "SELL"
    };

    AssetAnalysis {
        symbol: symbol.to_string(),
        price,
        change_24h,
        signal: signal.to_string(),
    }
}

fn main() {
    println!("=== Параллельный анализ портфеля ===\n");

    let start = std::time::Instant::now();

    // Запускаем анализ каждого актива в отдельном потоке
    let btc_handle = thread::spawn(|| {
        analyze_asset("BTC", 42150.0)
    });

    let eth_handle = thread::spawn(|| {
        analyze_asset("ETH", 2250.0)
    });

    let sol_handle = thread::spawn(|| {
        analyze_asset("SOL", 98.50)
    });

    // Собираем результаты
    let analyses = vec![
        btc_handle.join().expect("BTC анализ упал"),
        eth_handle.join().expect("ETH анализ упал"),
        sol_handle.join().expect("SOL анализ упал"),
    ];

    let elapsed = start.elapsed();

    // Выводим результаты
    println!("╔═══════════════════════════════════════════════╗");
    println!("║           АНАЛИЗ ПОРТФЕЛЯ                     ║");
    println!("╠═══════════════════════════════════════════════╣");

    for analysis in &analyses {
        println!("║ {:>6} | ${:>10.2} | {:>+6.1}% | {:>10} ║",
            analysis.symbol,
            analysis.price,
            analysis.change_24h,
            analysis.signal
        );
    }

    println!("╠═══════════════════════════════════════════════╣");
    println!("║ Время анализа: {:>6.2?}                       ║", elapsed);
    println!("╚═══════════════════════════════════════════════╝");
}
```

## Ожидание нескольких потоков

### Способ 1: Последовательный join

```rust
use std::thread;

fn main() {
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                println!("Поток {} работает", i);
                i * 10
            })
        })
        .collect();

    // Ждём каждый поток по очереди
    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    println!("Результаты: {:?}", results);
}
```

### Способ 2: Сбор результатов в цикле

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Параллельный расчёт индикаторов ===\n");

    let indicators = vec!["SMA", "EMA", "RSI", "MACD", "BB"];

    let handles: Vec<_> = indicators
        .iter()
        .map(|&name| {
            thread::spawn(move || {
                // Имитация расчёта
                thread::sleep(Duration::from_millis(50));
                let value = match name {
                    "SMA" => 42100.0,
                    "EMA" => 42050.0,
                    "RSI" => 65.5,
                    "MACD" => 125.0,
                    "BB" => 42200.0,
                    _ => 0.0,
                };
                (name, value)
            })
        })
        .collect();

    println!("Все индикаторы запущены, ожидаем результаты...\n");

    for handle in handles {
        match handle.join() {
            Ok((name, value)) => println!("{}: {:.2}", name, value),
            Err(_) => println!("Ошибка расчёта!"),
        }
    }
}
```

## Возвращаемые значения из потоков

Потоки могут возвращать любой тип, реализующий `Send`:

```rust
use std::thread;
use std::collections::HashMap;

fn main() {
    // Возврат простого значения
    let num_handle = thread::spawn(|| 42);

    // Возврат String
    let str_handle = thread::spawn(|| String::from("Результат"));

    // Возврат Vec
    let vec_handle = thread::spawn(|| vec![1, 2, 3, 4, 5]);

    // Возврат HashMap
    let map_handle = thread::spawn(|| {
        let mut prices = HashMap::new();
        prices.insert("BTC", 42000.0);
        prices.insert("ETH", 2200.0);
        prices
    });

    println!("Число: {}", num_handle.join().unwrap());
    println!("Строка: {}", str_handle.join().unwrap());
    println!("Вектор: {:?}", vec_handle.join().unwrap());
    println!("HashMap: {:?}", map_handle.join().unwrap());
}
```

## Практический пример: мониторинг нескольких торговых пар

```rust
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    bid: f64,
    ask: f64,
    spread: f64,
}

fn monitor_pair(symbol: &str, base_price: f64) -> PriceUpdate {
    thread::sleep(Duration::from_millis(100));

    let spread_pct = 0.0005; // 0.05%
    let bid = base_price * (1.0 - spread_pct);
    let ask = base_price * (1.0 + spread_pct);

    PriceUpdate {
        symbol: symbol.to_string(),
        bid,
        ask,
        spread: ask - bid,
    }
}

fn main() {
    println!("=== Мониторинг торговых пар ===\n");

    let pairs = vec![
        ("BTC/USDT", 42000.0),
        ("ETH/USDT", 2200.0),
        ("SOL/USDT", 98.0),
        ("XRP/USDT", 0.55),
    ];

    // Запускаем мониторинг каждой пары в отдельном потоке
    let handles: Vec<_> = pairs
        .into_iter()
        .map(|(symbol, price)| {
            thread::spawn(move || monitor_pair(symbol, price))
        })
        .collect();

    // Собираем обновления
    let updates: Vec<PriceUpdate> = handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .collect();

    // Выводим результаты
    println!("╔════════════════════════════════════════════════════╗");
    println!("║  Пара      │    Bid     │    Ask     │   Spread   ║");
    println!("╠════════════════════════════════════════════════════╣");

    for update in &updates {
        println!("║ {:>9} │ {:>10.4} │ {:>10.4} │ {:>10.4} ║",
            update.symbol,
            update.bid,
            update.ask,
            update.spread
        );
    }

    println!("╚════════════════════════════════════════════════════╝");

    // Находим пару с минимальным спредом
    if let Some(best) = updates.iter().min_by(|a, b| {
        a.spread.partial_cmp(&b.spread).unwrap()
    }) {
        println!("\nЛучший спред: {} (${:.4})", best.symbol, best.spread);
    }
}
```

## Практические упражнения

### Упражнение 1: Параллельный расчёт скользящих средних

Напиши программу, которая параллельно рассчитывает SMA для разных периодов:

```rust
use std::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let sum: f64 = prices[prices.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let periods = vec![3, 5, 7, 10];

    // TODO: Запусти расчёт каждого SMA в отдельном потоке
    // и выведи результаты
}
```

### Упражнение 2: Арбитражный сканер

Создай программу, которая параллельно получает цены с разных "бирж" и находит арбитражные возможности:

```rust
use std::thread;
use std::time::Duration;

fn get_price(exchange: &str, symbol: &str) -> f64 {
    thread::sleep(Duration::from_millis(50));
    // Имитация разных цен
    match (exchange, symbol) {
        ("Binance", "BTC") => 42000.0,
        ("Coinbase", "BTC") => 42050.0,
        ("Kraken", "BTC") => 41980.0,
        _ => 0.0,
    }
}

fn main() {
    // TODO: Получи цены BTC со всех бирж параллельно
    // Найди возможность арбитража (разница > 0.1%)
}
```

### Упражнение 3: Обработка ошибок в потоках

Напиши программу, которая безопасно обрабатывает ситуации, когда некоторые потоки падают:

```rust
use std::thread;

fn fetch_price(exchange: &str) -> f64 {
    if exchange == "BadExchange" {
        panic!("Биржа недоступна!");
    }
    42000.0
}

fn main() {
    let exchanges = vec!["Binance", "BadExchange", "Coinbase", "Kraken"];

    // TODO: Запусти потоки для каждой биржи
    // Обработай падения безопасно
    // Посчитай среднюю цену только из успешных результатов
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `JoinHandle<T>` | Возвращается `spawn()`, позволяет ждать завершения потока |
| `.join()` | Блокирует текущий поток до завершения целевого |
| `Result<T, E>` | join возвращает Result — можно обработать панику |
| `.unwrap()` / `.expect()` | Извлечение значения с паникой при ошибке |
| Параллельный сбор | Запуск нескольких потоков и сбор результатов |

## Домашнее задание

1. **Параллельный бэктест**: Создай программу, которая запускает тестирование торговой стратегии на разных таймфреймах параллельно (1m, 5m, 15m, 1h). Каждый поток должен возвращать результат бэктеста (прибыль, количество сделок, win rate).

2. **Мультибиржевой ордербук**: Напиши систему, которая параллельно получает ордербуки с 5 бирж и агрегирует их в единый "виртуальный" ордербук. Используй `join()` для синхронизации.

3. **Отказоустойчивый загрузчик**: Создай загрузчик данных, который пытается получить цену актива с 3 бирж параллельно. Если одна биржа падает (panic), остальные должны работать. Верни первую успешную цену или ошибку, если все упали.

4. **Параллельный расчёт риска**: Напиши программу, которая параллельно рассчитывает метрики риска для портфеля: VaR, Sharpe Ratio, Max Drawdown, Beta. Объедини результаты в итоговый отчёт о риске.

## Навигация

[← Предыдущий день](../153-thread-spawn/ru.md) | [Следующий день →](../155-move-closures/ru.md)
