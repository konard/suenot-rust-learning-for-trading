# День 153: std::thread::spawn — запускаем поток

## Аналогия из трейдинга

Представь, что ты один трейдер, который следит за несколькими биржами одновременно: Binance, Coinbase, Kraken. Если ты будешь проверять их по очереди — пока смотришь Binance, можешь пропустить выгодную цену на Kraken.

Решение? Нанять **несколько аналитиков** (потоков), каждый из которых будет следить за своей биржей **параллельно**. Функция `std::thread::spawn` — это как нанять нового аналитика для выполнения конкретной задачи.

## Базовый запуск потока

```rust
use std::thread;

fn main() {
    // Запускаем новый поток для мониторинга цен
    thread::spawn(|| {
        println!("Поток: Мониторю цену BTC...");
        println!("Поток: Текущая цена: $42,500");
    });

    println!("Главный поток: Продолжаю работу");
}
```

**Важно:** Главный поток может завершиться раньше, чем дочерний! Пока мы не изучили `join`, вывод может быть неполным.

## Мониторинг нескольких бирж

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // Поток для мониторинга Binance
    thread::spawn(|| {
        for i in 1..=3 {
            println!("[Binance] Проверка #{}: BTC = $42,{:03}", i, 500 + i * 10);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Поток для мониторинга Coinbase
    thread::spawn(|| {
        for i in 1..=3 {
            println!("[Coinbase] Проверка #{}: BTC = $42,{:03}", i, 520 + i * 5);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Главный поток ждёт немного, чтобы увидеть вывод
    thread::sleep(Duration::from_millis(500));
    println!("[Main] Мониторинг завершён");
}
```

## Передача данных в поток

### Клонирование данных

```rust
use std::thread;

fn main() {
    let symbol = String::from("BTC/USDT");
    let initial_price = 42000.0;

    // Клонируем данные для потока
    let symbol_clone = symbol.clone();

    thread::spawn(move || {
        println!("Анализирую пару: {}", symbol_clone);
        println!("Начальная цена: ${:.2}", initial_price);

        // Симуляция анализа
        let target = initial_price * 1.02;  // +2%
        println!("Цель: ${:.2}", target);
    });

    println!("Главный поток: Работаю с {}", symbol);
    thread::sleep(std::time::Duration::from_millis(100));
}
```

### move замыкание

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // move перемещает владение prices в поток
    let handle = thread::spawn(move || {
        let sum: f64 = prices.iter().sum();
        let avg = sum / prices.len() as f64;
        println!("Средняя цена: ${:.2}", avg);
        avg  // Возвращаем значение
    });

    // prices больше недоступен здесь!
    // println!("{:?}", prices);  // Ошибка компиляции

    // Ждём завершения и получаем результат
    let average = handle.join().unwrap();
    println!("Получено из потока: ${:.2}", average);
}
```

## Торговые сигналы из разных источников

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // Поток для технического анализа
    let ta_thread = thread::spawn(|| {
        println!("[TA] Анализирую RSI и MACD...");
        thread::sleep(Duration::from_millis(150));
        let signal = "BUY";  // Результат анализа
        println!("[TA] Сигнал: {}", signal);
        signal
    });

    // Поток для анализа объёмов
    let volume_thread = thread::spawn(|| {
        println!("[Volume] Анализирую объёмы...");
        thread::sleep(Duration::from_millis(100));
        let signal = "NEUTRAL";
        println!("[Volume] Сигнал: {}", signal);
        signal
    });

    // Поток для новостного анализа
    let news_thread = thread::spawn(|| {
        println!("[News] Проверяю новости...");
        thread::sleep(Duration::from_millis(200));
        let signal = "BUY";
        println!("[News] Сигнал: {}", signal);
        signal
    });

    // Собираем результаты
    let ta_signal = ta_thread.join().unwrap();
    let volume_signal = volume_thread.join().unwrap();
    let news_signal = news_thread.join().unwrap();

    println!("\n=== Итоговый анализ ===");
    println!("TA: {}, Volume: {}, News: {}", ta_signal, volume_signal, news_signal);
}
```

## Параллельный расчёт индикаторов

```rust
use std::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    Some(slice.iter().sum::<f64>() / period as f64)
}

fn calculate_rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 {
        return None;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in (prices.len() - period)..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses += change.abs();
        }
    }

    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let prices_for_sma = prices.clone();
    let prices_for_rsi = prices.clone();

    // Параллельный расчёт индикаторов
    let sma_handle = thread::spawn(move || {
        println!("[SMA] Расчёт SMA-10...");
        calculate_sma(&prices_for_sma, 10)
    });

    let rsi_handle = thread::spawn(move || {
        println!("[RSI] Расчёт RSI-14...");
        calculate_rsi(&prices_for_rsi, 14)
    });

    // Получаем результаты
    let sma = sma_handle.join().unwrap();
    let rsi = rsi_handle.join().unwrap();

    println!("\n=== Индикаторы ===");
    match sma {
        Some(v) => println!("SMA-10: ${:.2}", v),
        None => println!("SMA-10: Недостаточно данных"),
    }
    match rsi {
        Some(v) => println!("RSI-14: {:.2}", v),
        None => println!("RSI-14: Недостаточно данных"),
    }
}
```

## Параллельная обработка ордеров

```rust
use std::thread;
use std::time::Duration;

struct Order {
    id: u32,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn process_order(order: Order) -> Result<String, String> {
    println!("[Order #{}] Обработка {} {} @ ${:.2}...",
             order.id, order.side, order.symbol, order.price);

    // Симуляция задержки обработки
    thread::sleep(Duration::from_millis(100));

    // Симуляция успешного выполнения
    Ok(format!("Order #{} executed: {} {} {} @ ${:.2}",
               order.id, order.side, order.quantity, order.symbol, order.price))
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC/USDT".into(), side: "BUY".into(), quantity: 0.5, price: 42000.0 },
        Order { id: 2, symbol: "ETH/USDT".into(), side: "SELL".into(), quantity: 2.0, price: 2800.0 },
        Order { id: 3, symbol: "SOL/USDT".into(), side: "BUY".into(), quantity: 10.0, price: 95.0 },
    ];

    let mut handles = vec![];

    // Запускаем обработку каждого ордера в отдельном потоке
    for order in orders {
        let handle = thread::spawn(move || {
            process_order(order)
        });
        handles.push(handle);
    }

    // Собираем результаты
    println!("\n=== Результаты ===");
    for handle in handles {
        match handle.join().unwrap() {
            Ok(msg) => println!("✓ {}", msg),
            Err(e) => println!("✗ Ошибка: {}", e),
        }
    }
}
```

## Именование потоков (для отладки)

```rust
use std::thread;

fn main() {
    let builder = thread::Builder::new()
        .name("price-monitor".into())
        .stack_size(32 * 1024);  // 32 KB стек

    let handle = builder.spawn(|| {
        let thread = thread::current();
        println!("Поток '{}' запущен", thread.name().unwrap_or("unnamed"));
        println!("Мониторю цены...");
    }).unwrap();

    handle.join().unwrap();
}
```

## Практический пример: мульти-биржевой арбитраж

```rust
use std::thread;
use std::time::Duration;

struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

fn fetch_price(exchange: &str, symbol: &str) -> ExchangePrice {
    // Симуляция получения цены с биржи
    thread::sleep(Duration::from_millis(50 + (exchange.len() * 10) as u64));

    let base_price = 42000.0;
    let spread = match exchange {
        "Binance" => 10.0,
        "Coinbase" => 25.0,
        "Kraken" => 15.0,
        _ => 20.0,
    };

    let offset = match exchange {
        "Binance" => 0.0,
        "Coinbase" => 50.0,
        "Kraken" => -30.0,
        _ => 0.0,
    };

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: base_price + offset - spread / 2.0,
        ask: base_price + offset + spread / 2.0,
    }
}

fn main() {
    let exchanges = vec!["Binance", "Coinbase", "Kraken"];
    let symbol = "BTC/USDT";

    let mut handles = vec![];

    println!("Запрашиваем цены с {} бирж параллельно...\n", exchanges.len());

    for exchange in exchanges {
        let sym = symbol.to_string();
        let handle = thread::spawn(move || {
            fetch_price(exchange, &sym)
        });
        handles.push(handle);
    }

    let mut prices: Vec<ExchangePrice> = vec![];
    for handle in handles {
        prices.push(handle.join().unwrap());
    }

    // Выводим цены
    println!("{:<12} {:>12} {:>12} {:>12}", "Exchange", "Bid", "Ask", "Spread");
    println!("{}", "-".repeat(50));

    for p in &prices {
        println!("{:<12} ${:>10.2} ${:>10.2} ${:>10.2}",
                 p.exchange, p.bid, p.ask, p.ask - p.bid);
    }

    // Ищем арбитражные возможности
    println!("\n=== Арбитражные возможности ===");

    for i in 0..prices.len() {
        for j in 0..prices.len() {
            if i != j {
                let profit = prices[j].bid - prices[i].ask;
                if profit > 0.0 {
                    println!("Купить на {} @ ${:.2}, продать на {} @ ${:.2} = ${:.2} прибыли",
                             prices[i].exchange, prices[i].ask,
                             prices[j].exchange, prices[j].bid,
                             profit);
                }
            }
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `thread::spawn` | Создаёт новый поток выполнения |
| `move` замыкание | Перемещает владение данными в поток |
| `JoinHandle` | Хендл для ожидания завершения потока |
| `thread::sleep` | Приостановка потока на время |
| `thread::current()` | Получение информации о текущем потоке |
| `Builder` | Расширенная настройка потока |

## Важные моменты

1. **Данные должны быть `Send`** — тип должен быть безопасен для передачи между потоками
2. **Владение передаётся** — после `move` данные недоступны в исходном потоке
3. **Клонируйте, если нужно** — `clone()` позволяет использовать данные в нескольких потоках
4. **Главный поток может завершиться раньше** — используйте `join()` для ожидания

## Домашнее задание

1. **Мульти-биржевой мониторинг:** Создай программу, которая параллельно получает цены с 5 разных "бирж" и находит лучшую цену для покупки и продажи

2. **Параллельный бэктест:** Напиши функцию, которая запускает бэктест одной стратегии на нескольких временных периодах параллельно и собирает результаты

3. **Система оповещений:** Создай систему, где один поток генерирует случайные цены, а другие потоки проверяют различные условия (пересечение уровней, волатильность, объём)

4. **Обработка истории сделок:** Напиши программу, которая параллельно обрабатывает историю сделок, разбитую на "чанки" (части), и суммирует результаты

## Навигация

[← День 152: Потоки: следим за биржами параллельно](../152-threads-watching-exchanges/ru.md) | [День 154: join: ждём завершения потока →](../154-thread-join/ru.md)
