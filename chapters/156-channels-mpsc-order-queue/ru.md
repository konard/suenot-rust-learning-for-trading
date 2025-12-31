# День 156: Каналы mpsc — очередь ордеров

## Аналогия из трейдинга

Представь торговый зал биржи: множество трейдеров (отправители) выкрикивают свои ордера, а один маркет-мейкер (получатель) принимает и обрабатывает их по очереди. Это и есть **mpsc** — multiple producers, single consumer (много производителей, один потребитель).

В алготрейдинге:
- **Производители** — модули, генерирующие торговые сигналы (анализ RSI, MACD, уровней поддержки)
- **Потребитель** — order execution engine, который обрабатывает ордера последовательно
- **Канал** — очередь ордеров между ними

## Что такое mpsc

`mpsc` (multi-producer, single-consumer) — это канал для передачи данных между потоками:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // Создаём канал: tx — передатчик, rx — приёмник
    let (tx, rx) = mpsc::channel();

    // Поток-производитель отправляет ордер
    thread::spawn(move || {
        let order = "BUY BTC 0.5";
        tx.send(order).unwrap();
        println!("Ордер отправлен: {}", order);
    });

    // Главный поток получает ордер
    let received = rx.recv().unwrap();
    println!("Ордер получен: {}", received);
}
```

## Типы каналов

### Неограниченный канал (unbounded)

Очередь без лимита — как торговая книга без ограничений:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel(); // Неограниченный канал

    thread::spawn(move || {
        for i in 1..=5 {
            let order = format!("ORDER_{}", i);
            tx.send(order).unwrap();
        }
    });

    // Получаем все ордера
    for order in rx {
        println!("Обрабатываю: {}", order);
    }
}
```

### Ограниченный канал (bounded)

Канал с фиксированной ёмкостью — как очередь с лимитом на количество ордеров:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // sync_channel с ёмкостью 2 — буфер на 2 ордера
    let (tx, rx) = mpsc::sync_channel(2);

    thread::spawn(move || {
        for i in 1..=5 {
            println!("Отправляю ордер {}", i);
            tx.send(i).unwrap(); // Блокируется, если буфер полон
            println!("Ордер {} отправлен", i);
        }
    });

    thread::sleep(std::time::Duration::from_millis(100));

    for order in rx {
        println!("Получен ордер: {}", order);
        thread::sleep(std::time::Duration::from_millis(50));
    }
}
```

## Несколько производителей

Несколько источников сигналов отправляют ордера одному исполнителю:

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    source: String,
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Клонируем передатчик для каждого источника сигналов
    let tx_rsi = tx.clone();
    let tx_macd = tx.clone();
    let tx_support = tx;

    // RSI анализатор
    thread::spawn(move || {
        let order = Order {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            source: "RSI".to_string(),
        };
        tx_rsi.send(order).unwrap();
    });

    // MACD анализатор
    thread::spawn(move || {
        let order = Order {
            symbol: "ETH/USDT".to_string(),
            side: "SELL".to_string(),
            quantity: 2.0,
            source: "MACD".to_string(),
        };
        tx_macd.send(order).unwrap();
    });

    // Анализатор уровней поддержки
    thread::spawn(move || {
        let order = Order {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.3,
            source: "Support Level".to_string(),
        };
        tx_support.send(order).unwrap();
    });

    // Исполнитель обрабатывает все ордера
    for _ in 0..3 {
        match rx.recv() {
            Ok(order) => {
                println!("Исполняю ордер от {}: {} {} {}",
                    order.source, order.side, order.quantity, order.symbol);
            }
            Err(_) => break,
        }
    }
}
```

## Обработка ошибок

```rust
use std::sync::mpsc::{self, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        tx.send("Delayed order".to_string()).unwrap();
    });

    // try_recv — неблокирующий приём
    match rx.try_recv() {
        Ok(order) => println!("Получен: {}", order),
        Err(TryRecvError::Empty) => println!("Очередь пуста"),
        Err(TryRecvError::Disconnected) => println!("Канал закрыт"),
    }

    // recv_timeout — ожидание с таймаутом
    match rx.recv_timeout(Duration::from_millis(100)) {
        Ok(order) => println!("Получен: {}", order),
        Err(RecvTimeoutError::Timeout) => println!("Таймаут ожидания"),
        Err(RecvTimeoutError::Disconnected) => println!("Канал закрыт"),
    }

    // Ждём достаточно долго
    match rx.recv_timeout(Duration::from_millis(200)) {
        Ok(order) => println!("Получен: {}", order),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}
```

## Практический пример: Система исполнения ордеров

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit(f64),
    StopLoss(f64),
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: Side,
    order_type: OrderType,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
enum ExecutionResult {
    Filled { order_id: u64, price: f64, quantity: f64 },
    PartialFill { order_id: u64, filled: f64, remaining: f64 },
    Rejected { order_id: u64, reason: String },
}

fn main() {
    let (order_tx, order_rx) = mpsc::channel::<Order>();
    let (result_tx, result_rx) = mpsc::channel::<ExecutionResult>();

    // Поток исполнения ордеров
    let exec_result_tx = result_tx.clone();
    thread::spawn(move || {
        let mut current_price = 42000.0;

        for order in order_rx {
            println!("Обрабатываю ордер #{}: {:?} {} {}",
                order.id, order.side, order.quantity, order.symbol);

            // Симуляция изменения цены
            current_price += (order.id as f64 - 2.0) * 50.0;

            let result = match order.order_type {
                OrderType::Market => {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        price: current_price,
                        quantity: order.quantity,
                    }
                }
                OrderType::Limit(limit_price) => {
                    let can_fill = match order.side {
                        Side::Buy => current_price <= limit_price,
                        Side::Sell => current_price >= limit_price,
                    };

                    if can_fill {
                        ExecutionResult::Filled {
                            order_id: order.id,
                            price: limit_price,
                            quantity: order.quantity,
                        }
                    } else {
                        ExecutionResult::Rejected {
                            order_id: order.id,
                            reason: format!(
                                "Цена {} не соответствует лимиту {}",
                                current_price, limit_price
                            ),
                        }
                    }
                }
                OrderType::StopLoss(stop_price) => {
                    let triggered = match order.side {
                        Side::Buy => current_price >= stop_price,
                        Side::Sell => current_price <= stop_price,
                    };

                    if triggered {
                        ExecutionResult::Filled {
                            order_id: order.id,
                            price: current_price,
                            quantity: order.quantity,
                        }
                    } else {
                        ExecutionResult::Rejected {
                            order_id: order.id,
                            reason: format!(
                                "Stop-loss не активирован: цена {} стоп {}",
                                current_price, stop_price
                            ),
                        }
                    }
                }
            };

            exec_result_tx.send(result).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Поток обработки результатов
    let result_handler = thread::spawn(move || {
        let mut filled_count = 0;
        let mut rejected_count = 0;
        let mut total_volume = 0.0;

        for result in result_rx {
            match result {
                ExecutionResult::Filled { order_id, price, quantity } => {
                    println!("  ✓ Ордер #{} исполнен: {} @ ${:.2}",
                        order_id, quantity, price);
                    filled_count += 1;
                    total_volume += price * quantity;
                }
                ExecutionResult::PartialFill { order_id, filled, remaining } => {
                    println!("  ◐ Ордер #{} частично: {} исполнено, {} осталось",
                        order_id, filled, remaining);
                }
                ExecutionResult::Rejected { order_id, reason } => {
                    println!("  ✗ Ордер #{} отклонён: {}", order_id, reason);
                    rejected_count += 1;
                }
            }
        }

        println!("\n=== Статистика ===");
        println!("Исполнено: {}", filled_count);
        println!("Отклонено: {}", rejected_count);
        println!("Общий объём: ${:.2}", total_volume);
    });

    // Генерируем ордера
    let orders = vec![
        Order {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Market,
            quantity: 0.5,
            timestamp: 1000,
        },
        Order {
            id: 2,
            symbol: "BTC/USDT".to_string(),
            side: Side::Sell,
            order_type: OrderType::Limit(42100.0),
            quantity: 0.3,
            timestamp: 1001,
        },
        Order {
            id: 3,
            symbol: "ETH/USDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit(3000.0),
            quantity: 2.0,
            timestamp: 1002,
        },
        Order {
            id: 4,
            symbol: "BTC/USDT".to_string(),
            side: Side::Sell,
            order_type: OrderType::StopLoss(41000.0),
            quantity: 0.2,
            timestamp: 1003,
        },
    ];

    for order in orders {
        order_tx.send(order).unwrap();
    }

    drop(order_tx); // Закрываем канал ордеров
    drop(result_tx); // Закрываем канал результатов

    result_handler.join().unwrap();
}
```

## Паттерны использования

### 1. Worker Pool — пул обработчиков

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));

    // Создаём пул воркеров
    let mut handles = vec![];

    for id in 0..3 {
        let rx = rx.clone();
        let handle = thread::spawn(move || {
            loop {
                let task = {
                    let rx = rx.lock().unwrap();
                    rx.recv()
                };

                match task {
                    Ok(order) => {
                        println!("Worker {} обрабатывает: {}", id, order);
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }
        });
        handles.push(handle);
    }

    // Отправляем задачи
    for i in 1..=10 {
        tx.send(format!("Order_{}", i)).unwrap();
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### 2. Request-Response — запрос-ответ

```rust
use std::sync::mpsc;
use std::thread;

struct PriceRequest {
    symbol: String,
    response_tx: mpsc::Sender<f64>,
}

fn main() {
    let (request_tx, request_rx) = mpsc::channel::<PriceRequest>();

    // Поток получения цен
    thread::spawn(move || {
        for req in request_rx {
            let price = match req.symbol.as_str() {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2500.0,
                _ => 0.0,
            };
            req.response_tx.send(price).unwrap();
        }
    });

    // Запрашиваем цены
    for symbol in ["BTC/USDT", "ETH/USDT", "XRP/USDT"] {
        let (resp_tx, resp_rx) = mpsc::channel();

        request_tx.send(PriceRequest {
            symbol: symbol.to_string(),
            response_tx: resp_tx,
        }).unwrap();

        let price = resp_rx.recv().unwrap();
        println!("{}: ${:.2}", symbol, price);
    }
}
```

### 3. Event Stream — поток событий

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum MarketEvent {
    PriceUpdate { symbol: String, price: f64 },
    TradeExecuted { symbol: String, quantity: f64, price: f64 },
    OrderBookUpdate { symbol: String, bids: Vec<f64>, asks: Vec<f64> },
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Симулятор рыночных событий
    let tx_clone = tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            tx_clone.send(MarketEvent::PriceUpdate {
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + (i as f64) * 10.0,
            }).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        tx.send(MarketEvent::TradeExecuted {
            symbol: "BTC/USDT".to_string(),
            quantity: 0.5,
            price: 42005.0,
        }).unwrap();
    });

    // Обработчик событий
    for _ in 0..6 {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                match event {
                    MarketEvent::PriceUpdate { symbol, price } => {
                        println!("[PRICE] {}: ${:.2}", symbol, price);
                    }
                    MarketEvent::TradeExecuted { symbol, quantity, price } => {
                        println!("[TRADE] {}: {} @ ${:.2}", symbol, quantity, price);
                    }
                    MarketEvent::OrderBookUpdate { symbol, .. } => {
                        println!("[BOOK] {} обновлён", symbol);
                    }
                }
            }
            Err(_) => break,
        }
    }
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|----------------------|
| `mpsc::channel()` | Неограниченный канал | Очередь ордеров без лимита |
| `mpsc::sync_channel(n)` | Канал с буфером | Контроль нагрузки |
| `tx.clone()` | Множество отправителей | Несколько источников сигналов |
| `rx.recv()` | Блокирующий приём | Ожидание ордера |
| `rx.try_recv()` | Неблокирующий приём | Проверка без ожидания |
| `rx.recv_timeout()` | Приём с таймаутом | Таймаут на исполнение |

## Практические упражнения

### Упражнение 1: Агрегатор сигналов

Создай систему, где 3 стратегии (RSI, MACD, Bollinger) отправляют сигналы одному агрегатору:

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
struct Signal {
    strategy: String,
    symbol: String,
    action: String, // "BUY", "SELL", "HOLD"
    confidence: f64,
}

fn main() {
    // Твой код здесь
    // 1. Создай канал для сигналов
    // 2. Запусти 3 потока-стратегии
    // 3. Агрегируй сигналы и принимай решение
}
```

### Упражнение 2: Rate Limiter

Реализуй ограничитель скорости отправки ордеров (максимум 10 ордеров в секунду):

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Твой код здесь
    // Используй sync_channel для ограничения
}
```

### Упражнение 3: Приоритетная очередь

Модифицируй систему так, чтобы стоп-лосс ордера обрабатывались в первую очередь:

```rust
// Подсказка: используй два канала — один для срочных,
// другой для обычных ордеров
```

## Домашнее задание

1. **Система мониторинга позиций**: Создай систему, где несколько потоков отслеживают разные инструменты и отправляют уведомления в центральный обработчик при достижении stop-loss или take-profit уровней.

2. **Распределённый калькулятор индикаторов**: Реализуй систему, где один поток загружает исторические данные, несколько потоков параллельно рассчитывают разные индикаторы (RSI, MACD, SMA), и результаты собираются в один поток для анализа.

3. **Симулятор биржи**: Создай упрощённый симулятор биржи с:
   - Каналом для входящих ордеров
   - Matching engine, который сводит ордера
   - Каналом для уведомлений об исполнении
   - Несколькими "трейдерами", отправляющими ордера

4. **Graceful shutdown**: Модифицируй систему исполнения ордеров так, чтобы она корректно завершала работу при получении сигнала завершения, обработав все ордера в очереди.

## Навигация

[← Предыдущий день](../155-threads-price-monitor/ru.md) | [Следующий день →](../157-channels-crossbeam/ru.md)
