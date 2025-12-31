# День 171: crossbeam channels: быстрее mpsc

## Аналогия из трейдинга

Представь высокочастотную торговую систему. Стандартные каналы `std::sync::mpsc` — это как почтовое отделение: надёжно, но медленно. А `crossbeam-channel` — это как прямой оптоволоконный канал между биржей и твоим торговым сервером: данные летят с минимальной задержкой.

В реальном трейдинге каждая микросекунда на счету:
- Market data feed генерирует тысячи обновлений цен в секунду
- Торговый движок должен мгновенно получать эти данные
- Стратегия анализирует и отправляет ордера без задержек

Стандартный `mpsc` может стать узким местом, а `crossbeam-channel` решает эту проблему благодаря lock-free алгоритмам.

## Почему crossbeam-channel быстрее?

| Характеристика | std::sync::mpsc | crossbeam-channel |
|----------------|-----------------|-------------------|
| Алгоритм | Блокирующий | Lock-free |
| Производители | Много (mpsc) | Много (mpmc) |
| Потребители | Один | Много |
| Bounded/Unbounded | Только unbounded | Оба варианта |
| Select | Нет | Есть |
| Zero-capacity | Нет | Есть |

## Установка crossbeam-channel

Добавь в `Cargo.toml`:

```toml
[dependencies]
crossbeam-channel = "0.5"
```

## Базовое использование

### Создание каналов

```rust
use crossbeam_channel::{unbounded, bounded};

fn main() {
    // Неограниченный канал (как std::sync::mpsc::channel)
    let (tx, rx) = unbounded::<f64>();

    // Ограниченный канал с буфером на 100 сообщений
    let (tx_bounded, rx_bounded) = bounded::<f64>(100);

    // Zero-capacity канал (рандеву) — отправитель ждёт получателя
    let (tx_zero, rx_zero) = bounded::<f64>(0);

    // Отправляем цену BTC
    tx.send(42500.0).unwrap();

    // Получаем цену
    let price = rx.recv().unwrap();
    println!("Получена цена BTC: ${}", price);
}
```

## Пример: Market Data Feed

```rust
use crossbeam_channel::{bounded, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

fn market_data_producer(tx: Sender<MarketTick>, symbol: &str) {
    let mut timestamp = 0u64;
    let base_price = match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    for i in 0..1000 {
        let spread = 0.01 * base_price; // 1% спред
        let variation = (i as f64 * 0.1).sin() * base_price * 0.001;

        let tick = MarketTick {
            symbol: symbol.to_string(),
            bid: base_price + variation,
            ask: base_price + variation + spread,
            timestamp,
        };

        if tx.send(tick).is_err() {
            break; // Канал закрыт
        }
        timestamp += 1;
    }
}

fn trading_strategy(rx: Receiver<MarketTick>) -> Vec<String> {
    let mut signals = Vec::new();
    let mut last_prices: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    while let Ok(tick) = rx.recv() {
        let mid_price = (tick.bid + tick.ask) / 2.0;

        if let Some(&prev_price) = last_prices.get(&tick.symbol) {
            let change_pct = (mid_price - prev_price) / prev_price * 100.0;

            if change_pct > 0.05 {
                signals.push(format!("BUY {} @ {:.2}", tick.symbol, tick.ask));
            } else if change_pct < -0.05 {
                signals.push(format!("SELL {} @ {:.2}", tick.symbol, tick.bid));
            }
        }

        last_prices.insert(tick.symbol.clone(), mid_price);
    }

    signals
}

fn main() {
    // Bounded канал — не позволяем буферу расти бесконтрольно
    let (tx, rx) = bounded::<MarketTick>(1000);

    let start = Instant::now();

    // Запускаем продьюсеров для разных символов
    let tx_btc = tx.clone();
    let btc_producer = thread::spawn(move || {
        market_data_producer(tx_btc, "BTC");
    });

    let tx_eth = tx.clone();
    let eth_producer = thread::spawn(move || {
        market_data_producer(tx_eth, "ETH");
    });

    // Закрываем оригинальный sender — иначе канал никогда не закроется
    drop(tx);

    // Запускаем стратегию
    let strategy = thread::spawn(move || {
        trading_strategy(rx)
    });

    btc_producer.join().unwrap();
    eth_producer.join().unwrap();

    let signals = strategy.join().unwrap();

    let elapsed = start.elapsed();

    println!("Обработано за {:?}", elapsed);
    println!("Всего сигналов: {}", signals.len());
    println!("Примеры сигналов:");
    for signal in signals.iter().take(5) {
        println!("  {}", signal);
    }
}
```

## MPMC: Множество потребителей

Главное преимущество `crossbeam-channel` — поддержка множества потребителей (Multi-Producer Multi-Consumer):

```rust
use crossbeam_channel::bounded;
use std::thread;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn order_processor(id: u32, rx: crossbeam_channel::Receiver<Order>) {
    while let Ok(order) = rx.recv() {
        println!(
            "Процессор {}: Обработка ордера #{} - {} {} {} @ {:.2}",
            id, order.id, order.side, order.quantity, order.symbol, order.price
        );
        // Имитация обработки
        thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("Процессор {}: Завершение работы", id);
}

fn main() {
    let (tx, rx) = bounded::<Order>(100);

    // Запускаем 4 обработчика ордеров
    let mut processors = vec![];
    for i in 0..4 {
        let rx_clone = rx.clone();
        processors.push(thread::spawn(move || {
            order_processor(i, rx_clone);
        }));
    }

    // Отправляем ордера
    for id in 0..20 {
        let order = Order {
            id,
            symbol: if id % 2 == 0 { "BTC".to_string() } else { "ETH".to_string() },
            side: if id % 3 == 0 { "BUY".to_string() } else { "SELL".to_string() },
            price: 42000.0 + id as f64 * 100.0,
            quantity: 0.1 + (id as f64 * 0.01),
        };
        tx.send(order).unwrap();
    }

    // Закрываем канал
    drop(tx);

    // Ждём завершения всех процессоров
    for p in processors {
        p.join().unwrap();
    }

    println!("Все ордера обработаны!");
}
```

## Select: Ожидание нескольких каналов

`crossbeam_channel::select!` позволяет ожидать сразу несколько каналов:

```rust
use crossbeam_channel::{bounded, select, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    RiskAlert { message: String },
}

fn price_feed(tx: Sender<TradingEvent>) {
    for i in 0..10 {
        thread::sleep(Duration::from_millis(100));
        tx.send(TradingEvent::PriceUpdate {
            symbol: "BTC".to_string(),
            price: 42000.0 + i as f64 * 10.0,
        }).ok();
    }
}

fn order_executor(tx: Sender<TradingEvent>) {
    for id in 1..=3 {
        thread::sleep(Duration::from_millis(300));
        tx.send(TradingEvent::OrderFilled {
            order_id: id,
            price: 42000.0 + id as f64 * 50.0,
        }).ok();
    }
}

fn risk_monitor(tx: Sender<TradingEvent>) {
    thread::sleep(Duration::from_millis(500));
    tx.send(TradingEvent::RiskAlert {
        message: "Высокая волатильность!".to_string(),
    }).ok();
}

fn main() {
    let (price_tx, price_rx) = bounded::<TradingEvent>(10);
    let (order_tx, order_rx) = bounded::<TradingEvent>(10);
    let (risk_tx, risk_rx) = bounded::<TradingEvent>(10);

    // Запускаем источники событий
    let h1 = thread::spawn(move || price_feed(price_tx));
    let h2 = thread::spawn(move || order_executor(order_tx));
    let h3 = thread::spawn(move || risk_monitor(risk_tx));

    // Главный цикл обработки событий
    let mut running = true;
    let mut event_count = 0;

    while running && event_count < 20 {
        select! {
            recv(price_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("📈 Цена: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            recv(order_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("✅ Ордер: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            recv(risk_rx) -> msg => {
                match msg {
                    Ok(event) => {
                        println!("⚠️ Риск: {:?}", event);
                        event_count += 1;
                    }
                    Err(_) => {}
                }
            }
            default(Duration::from_millis(1000)) => {
                println!("Таймаут — нет событий");
                running = false;
            }
        }
    }

    h1.join().ok();
    h2.join().ok();
    h3.join().ok();

    println!("\nОбработано событий: {}", event_count);
}
```

## Bounded vs Unbounded: Контроль памяти

```rust
use crossbeam_channel::{bounded, unbounded, TrySendError};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceQuote {
    symbol: String,
    price: f64,
}

fn main() {
    // Bounded канал — защита от переполнения памяти
    let (tx, rx) = bounded::<PriceQuote>(5);

    // Быстрый продьюсер
    let producer = thread::spawn(move || {
        for i in 0..20 {
            let quote = PriceQuote {
                symbol: "BTC".to_string(),
                price: 42000.0 + i as f64,
            };

            // try_send не блокирует, если буфер полон
            match tx.try_send(quote) {
                Ok(_) => println!("Отправлено: цена #{}", i),
                Err(TrySendError::Full(q)) => {
                    println!("Буфер полон! Пропускаем цену: {:.2}", q.price);
                }
                Err(TrySendError::Disconnected(_)) => {
                    println!("Канал закрыт!");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    });

    // Медленный потребитель
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(quote) = rx.recv() {
            println!("Получено: {} @ {:.2}", quote.symbol, quote.price);
            count += 1;
            // Медленная обработка
            thread::sleep(Duration::from_millis(200));
        }
        println!("Всего обработано: {}", count);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Практический пример: Торговый движок с приоритетами

```rust
use crossbeam_channel::{bounded, select, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum Priority {
    High,   // Стоп-лоссы, рисковые алерты
    Normal, // Обычные ордера
    Low,    // Отчёты, логирование
}

#[derive(Debug)]
struct Task {
    id: u64,
    priority: Priority,
    description: String,
}

struct PriorityTaskQueue {
    high_rx: Receiver<Task>,
    normal_rx: Receiver<Task>,
    low_rx: Receiver<Task>,
}

impl PriorityTaskQueue {
    fn recv(&self) -> Option<Task> {
        // Приоритетный select — сначала проверяем high, потом normal, потом low
        select! {
            recv(self.high_rx) -> task => task.ok(),
            recv(self.normal_rx) -> task => task.ok(),
            recv(self.low_rx) -> task => task.ok(),
            default(Duration::from_millis(100)) => None,
        }
    }
}

fn main() {
    let (high_tx, high_rx) = bounded::<Task>(10);
    let (normal_tx, normal_rx) = bounded::<Task>(100);
    let (low_tx, low_rx) = bounded::<Task>(100);

    let queue = PriorityTaskQueue {
        high_rx,
        normal_rx,
        low_rx,
    };

    // Генератор задач
    let task_generator = {
        let high_tx = high_tx.clone();
        let normal_tx = normal_tx.clone();
        let low_tx = low_tx.clone();

        thread::spawn(move || {
            for id in 0..15 {
                let (tx, priority, desc) = match id % 5 {
                    0 => (&high_tx, Priority::High, "СТОП-ЛОСС ТРИГГЕР!"),
                    1 | 2 => (&normal_tx, Priority::Normal, "Обычный ордер"),
                    _ => (&low_tx, Priority::Low, "Запись в лог"),
                };

                tx.send(Task {
                    id,
                    priority: priority.clone(),
                    description: desc.to_string(),
                }).ok();

                thread::sleep(Duration::from_millis(50));
            }

            drop(high_tx);
            drop(normal_tx);
            drop(low_tx);
        })
    };

    // Закрываем оригинальные отправители
    drop(high_tx);
    drop(normal_tx);
    drop(low_tx);

    // Обработчик задач
    let processor = thread::spawn(move || {
        let mut processed = 0;
        let start = Instant::now();

        loop {
            match queue.recv() {
                Some(task) => {
                    let priority_str = match task.priority {
                        Priority::High => "🔴 HIGH",
                        Priority::Normal => "🟡 NORMAL",
                        Priority::Low => "🟢 LOW",
                    };
                    println!("[{}] Задача #{}: {}", priority_str, task.id, task.description);
                    processed += 1;
                    thread::sleep(Duration::from_millis(30));
                }
                None => {
                    if start.elapsed() > Duration::from_secs(2) {
                        break;
                    }
                }
            }
        }

        processed
    });

    task_generator.join().unwrap();
    let total = processor.join().unwrap();

    println!("\nВсего обработано задач: {}", total);
}
```

## Сравнение производительности

```rust
use crossbeam_channel::bounded;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const NUM_MESSAGES: usize = 1_000_000;
const NUM_PRODUCERS: usize = 4;

fn bench_std_mpsc() -> Duration {
    let (tx, rx) = mpsc::channel::<u64>();
    let start = Instant::now();

    let producers: Vec<_> = (0..NUM_PRODUCERS)
        .map(|_| {
            let tx = tx.clone();
            thread::spawn(move || {
                for i in 0..(NUM_MESSAGES / NUM_PRODUCERS) as u64 {
                    tx.send(i).unwrap();
                }
            })
        })
        .collect();

    drop(tx);

    let consumer = thread::spawn(move || {
        let mut count = 0u64;
        while rx.recv().is_ok() {
            count += 1;
        }
        count
    });

    for p in producers {
        p.join().unwrap();
    }
    consumer.join().unwrap();

    start.elapsed()
}

fn bench_crossbeam() -> Duration {
    let (tx, rx) = bounded::<u64>(10000);
    let start = Instant::now();

    let producers: Vec<_> = (0..NUM_PRODUCERS)
        .map(|_| {
            let tx = tx.clone();
            thread::spawn(move || {
                for i in 0..(NUM_MESSAGES / NUM_PRODUCERS) as u64 {
                    tx.send(i).unwrap();
                }
            })
        })
        .collect();

    drop(tx);

    let consumer = thread::spawn(move || {
        let mut count = 0u64;
        while rx.recv().is_ok() {
            count += 1;
        }
        count
    });

    for p in producers {
        p.join().unwrap();
    }
    consumer.join().unwrap();

    start.elapsed()
}

use std::time::Duration;

fn main() {
    println!("Бенчмарк: {} сообщений, {} продьюсеров", NUM_MESSAGES, NUM_PRODUCERS);
    println!();

    let std_time = bench_std_mpsc();
    println!("std::sync::mpsc:     {:?}", std_time);

    let crossbeam_time = bench_crossbeam();
    println!("crossbeam-channel:   {:?}", crossbeam_time);

    let speedup = std_time.as_nanos() as f64 / crossbeam_time.as_nanos() as f64;
    println!();
    println!("crossbeam быстрее в {:.2}x раз", speedup);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `crossbeam-channel` | Быстрая lock-free альтернатива std::sync::mpsc |
| MPMC | Поддержка множества продьюсеров И множества потребителей |
| `bounded(n)` | Канал с ограниченным буфером |
| `unbounded()` | Канал без ограничения буфера |
| `select!` | Макрос для ожидания нескольких каналов |
| `try_send` | Неблокирующая отправка |
| Zero-capacity | Рандеву-канал для синхронизации |

## Домашнее задание

1. **Market Data Aggregator**: Реализуй систему, которая:
   - Получает данные из 5 разных источников (потоков)
   - Агрегирует цены по каждому символу
   - Отправляет усреднённую цену в стратегию
   - Использует bounded каналы для защиты от перегрузки

2. **Order Router**: Создай маршрутизатор ордеров с:
   - Каналом для входящих ордеров
   - 3 каналами для разных бирж (по типу инструмента)
   - Логикой выбора лучшей цены
   - Метриками производительности (ордеров/сек)

3. **Сравнение производительности**: Напиши бенчмарк, сравнивающий:
   - `std::sync::mpsc` vs `crossbeam-channel`
   - Bounded vs Unbounded каналы
   - Разные размеры буфера (10, 100, 1000, 10000)

4. **Event Sourcing**: Реализуй систему событий для трейдинга:
   - Все изменения состояния — через события в канале
   - Несколько обработчиков подписаны на события
   - Персистентность событий в файл
   - Восстановление состояния при старте

## Навигация

[← Предыдущий день](../170-crossbeam-advanced-concurrency/ru.md) | [Следующий день →](../172-crossbeam-scope/ru.md)
