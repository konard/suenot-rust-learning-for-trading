# День 194: Async Channels: tokio::sync::mpsc

## Аналогия из трейдинга

Представь себе торговый зал биржи. Множество трейдеров (producers) одновременно выкрикивают свои ордера, а один клерк (consumer) записывает их в книгу заявок. Трейдеры не ждут, пока клерк обработает каждый ордер — они продолжают торговать. Клерк обрабатывает ордера по мере их поступления.

Это и есть **mpsc** (multi-producer, single-consumer) канал — множество отправителей могут асинхронно посылать сообщения одному получателю. В контексте `tokio::sync::mpsc` это происходит в асинхронной среде, где ни отправители, ни получатель не блокируют выполнение других задач.

В реальной торговой системе это может быть:
- Множество WebSocket соединений отправляют обновления цен в один обработчик
- Несколько стратегий генерируют сигналы для одного исполнителя ордеров
- Разные источники данных пересылают события в единый логгер

## Что такое tokio::sync::mpsc?

`tokio::sync::mpsc` — это асинхронный канал для передачи сообщений между задачами в tokio. В отличие от `std::sync::mpsc`:

| Характеристика | std::sync::mpsc | tokio::sync::mpsc |
|----------------|-----------------|-------------------|
| Среда | Синхронная | Асинхронная |
| Блокировка | Блокирует поток | Приостанавливает задачу |
| Использование | Потоки OS | async/await задачи |
| Буфер | Неограниченный | Ограниченный (bounded) или неограниченный |

### Создание канала

```rust
use tokio::sync::mpsc;

// Ограниченный канал с буфером на 100 сообщений
let (tx, rx) = mpsc::channel::<TradeSignal>(100);

// Неограниченный канал (используйте с осторожностью!)
let (tx, rx) = mpsc::unbounded_channel::<TradeSignal>();
```

## Простой пример: Поток торговых сигналов

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct TradeSignal {
    symbol: String,
    action: String,    // "BUY" или "SELL"
    price: f64,
    quantity: f64,
}

#[tokio::main]
async fn main() {
    // Создаём канал с буфером на 32 сигнала
    let (tx, mut rx) = mpsc::channel::<TradeSignal>(32);

    // Генератор сигналов (producer)
    let signal_generator = tokio::spawn(async move {
        let signals = vec![
            TradeSignal {
                symbol: "BTC/USDT".to_string(),
                action: "BUY".to_string(),
                price: 42000.0,
                quantity: 0.5,
            },
            TradeSignal {
                symbol: "ETH/USDT".to_string(),
                action: "SELL".to_string(),
                price: 2800.0,
                quantity: 2.0,
            },
            TradeSignal {
                symbol: "BTC/USDT".to_string(),
                action: "SELL".to_string(),
                price: 42500.0,
                quantity: 0.5,
            },
        ];

        for signal in signals {
            println!("Генерирую сигнал: {:?}", signal);
            tx.send(signal).await.expect("Получатель закрыт");
            sleep(Duration::from_millis(500)).await;
        }

        println!("Все сигналы отправлены");
    });

    // Исполнитель ордеров (consumer)
    let order_executor = tokio::spawn(async move {
        while let Some(signal) = rx.recv().await {
            println!("Исполняю: {} {} {} @ {}",
                signal.action, signal.quantity, signal.symbol, signal.price);

            // Имитируем исполнение ордера
            sleep(Duration::from_millis(100)).await;
        }

        println!("Канал закрыт, исполнитель завершён");
    });

    // Ждём завершения обеих задач
    let _ = tokio::join!(signal_generator, order_executor);
}
```

## Множество отправителей: Мониторинг нескольких бирж

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn monitor_exchange(
    exchange: &str,
    symbol: &str,
    base_price: f64,
    tx: mpsc::Sender<PriceUpdate>,
) {
    for i in 0..5 {
        let spread = 0.001; // 0.1% спред
        let price_change = (i as f64 * 10.0) - 20.0;
        let mid_price = base_price + price_change;

        let update = PriceUpdate {
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            bid: mid_price * (1.0 - spread / 2.0),
            ask: mid_price * (1.0 + spread / 2.0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        if tx.send(update).await.is_err() {
            println!("{}: Получатель закрыт, выходим", exchange);
            break;
        }

        // Разные биржи обновляются с разной частотой
        sleep(Duration::from_millis(100 + (exchange.len() * 50) as u64)).await;
    }

    println!("{}: Мониторинг завершён", exchange);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<PriceUpdate>(100);

    // Запускаем мониторинг нескольких бирж параллельно
    let binance_tx = tx.clone();
    let binance = tokio::spawn(async move {
        monitor_exchange("Binance", "BTC/USDT", 42000.0, binance_tx).await;
    });

    let coinbase_tx = tx.clone();
    let coinbase = tokio::spawn(async move {
        monitor_exchange("Coinbase", "BTC/USD", 42050.0, coinbase_tx).await;
    });

    let kraken_tx = tx.clone();
    let kraken = tokio::spawn(async move {
        monitor_exchange("Kraken", "XBT/USD", 41980.0, kraken_tx).await;
    });

    // Важно: закрываем оригинальный sender, чтобы канал закрылся
    // когда все клоны будут уничтожены
    drop(tx);

    // Агрегатор цен
    let aggregator = tokio::spawn(async move {
        let mut best_bid: Option<(String, f64)> = None;
        let mut best_ask: Option<(String, f64)> = None;

        while let Some(update) = rx.recv().await {
            println!("[{}] {} Bid: {:.2} Ask: {:.2}",
                update.exchange, update.symbol, update.bid, update.ask);

            // Обновляем лучшие цены
            if best_bid.is_none() || update.bid > best_bid.as_ref().unwrap().1 {
                best_bid = Some((update.exchange.clone(), update.bid));
            }
            if best_ask.is_none() || update.ask < best_ask.as_ref().unwrap().1 {
                best_ask = Some((update.exchange.clone(), update.ask));
            }
        }

        if let (Some((bid_ex, bid)), Some((ask_ex, ask))) = (&best_bid, &best_ask) {
            println!("\n=== Лучшие цены ===");
            println!("Лучший BID: {:.2} на {}", bid, bid_ex);
            println!("Лучший ASK: {:.2} на {}", ask, ask_ex);
            println!("Арбитражный спред: {:.2}%", (ask - bid) / bid * 100.0);
        }
    });

    let _ = tokio::join!(binance, coinbase, kraken, aggregator);
}
```

## Ограниченный vs Неограниченный канал

### Ограниченный канал (Bounded Channel)

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
}

#[tokio::main]
async fn main() {
    // Буфер на 3 ордера — защита от перегрузки
    let (tx, mut rx) = mpsc::channel::<Order>(3);

    let producer = tokio::spawn(async move {
        for i in 1..=10 {
            let order = Order {
                id: i,
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            };

            println!("Пытаюсь отправить ордер #{}", i);

            // send() будет ждать, если буфер заполнен
            match tx.send(order).await {
                Ok(_) => println!("Ордер #{} отправлен", i),
                Err(e) => {
                    println!("Ошибка отправки: {}", e);
                    break;
                }
            }
        }
    });

    let consumer = tokio::spawn(async move {
        // Имитируем медленную обработку
        while let Some(order) = rx.recv().await {
            println!("Обрабатываю ордер #{}: {} @ {}",
                order.id, order.symbol, order.price);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(producer, consumer);
}
```

### Неограниченный канал (Unbounded Channel)

```rust
use tokio::sync::mpsc;

#[derive(Debug)]
struct MarketEvent {
    event_type: String,
    data: String,
}

#[tokio::main]
async fn main() {
    // Неограниченный канал — не блокирует отправителя
    // Используйте с осторожностью: может привести к исчерпанию памяти!
    let (tx, mut rx) = mpsc::unbounded_channel::<MarketEvent>();

    let producer = tokio::spawn(async move {
        for i in 1..=1000 {
            let event = MarketEvent {
                event_type: "TRADE".to_string(),
                data: format!("Trade #{}", i),
            };

            // unbounded_send не требует await
            if tx.send(event).is_err() {
                println!("Получатель закрыт");
                break;
            }
        }
        println!("Отправлено 1000 событий");
    });

    let consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(event) = rx.recv().await {
            count += 1;
            if count % 100 == 0 {
                println!("Обработано {} событий", count);
            }
        }
        println!("Всего обработано: {}", count);
    });

    let _ = tokio::join!(producer, consumer);
}
```

## Практический пример: Система исполнения ордеров

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
}

#[derive(Debug)]
enum ExecutionResult {
    Filled { order_id: u64, fill_price: f64 },
    Rejected { order_id: u64, reason: String },
    PartialFill { order_id: u64, filled_qty: f64, remaining_qty: f64 },
}

struct OrderExecutor {
    current_prices: HashMap<String, f64>,
}

impl OrderExecutor {
    fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert("BTC/USDT".to_string(), 42000.0);
        prices.insert("ETH/USDT".to_string(), 2800.0);
        OrderExecutor { current_prices: prices }
    }

    async fn execute(&self, order: Order) -> ExecutionResult {
        // Имитируем задержку исполнения
        sleep(Duration::from_millis(50)).await;

        let current_price = match self.current_prices.get(&order.symbol) {
            Some(price) => *price,
            None => return ExecutionResult::Rejected {
                order_id: order.id,
                reason: format!("Неизвестный символ: {}", order.symbol),
            },
        };

        match order.order_type {
            OrderType::Market => {
                ExecutionResult::Filled {
                    order_id: order.id,
                    fill_price: current_price,
                }
            }
            OrderType::Limit { price } => {
                let can_fill = match order.side {
                    OrderSide::Buy => current_price <= price,
                    OrderSide::Sell => current_price >= price,
                };

                if can_fill {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        fill_price: price,
                    }
                } else {
                    ExecutionResult::Rejected {
                        order_id: order.id,
                        reason: format!(
                            "Цена {} не достигла лимита {}",
                            current_price, price
                        ),
                    }
                }
            }
            OrderType::StopLoss { trigger_price } => {
                let triggered = match order.side {
                    OrderSide::Sell => current_price <= trigger_price,
                    OrderSide::Buy => current_price >= trigger_price,
                };

                if triggered {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        fill_price: current_price,
                    }
                } else {
                    ExecutionResult::Rejected {
                        order_id: order.id,
                        reason: "Стоп не сработал".to_string(),
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (order_tx, mut order_rx) = mpsc::channel::<Order>(100);
    let (result_tx, mut result_rx) = mpsc::channel::<ExecutionResult>(100);

    // Стратегия 1: Агрессивный трейдер
    let tx1 = order_tx.clone();
    let strategy1 = tokio::spawn(async move {
        for i in 1..=3 {
            let order = Order {
                id: i,
                symbol: "BTC/USDT".to_string(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                quantity: 0.1,
            };
            println!("Стратегия 1: Отправляю рыночный ордер #{}", i);
            tx1.send(order).await.ok();
            sleep(Duration::from_millis(100)).await;
        }
    });

    // Стратегия 2: Лимитный трейдер
    let tx2 = order_tx.clone();
    let strategy2 = tokio::spawn(async move {
        for i in 4..=6 {
            let order = Order {
                id: i,
                symbol: "ETH/USDT".to_string(),
                side: OrderSide::Sell,
                order_type: OrderType::Limit { price: 2850.0 },
                quantity: 1.0,
            };
            println!("Стратегия 2: Отправляю лимитный ордер #{}", i);
            tx2.send(order).await.ok();
            sleep(Duration::from_millis(150)).await;
        }
    });

    drop(order_tx);

    // Исполнитель ордеров
    let result_tx_clone = result_tx.clone();
    let executor = tokio::spawn(async move {
        let executor = OrderExecutor::new();

        while let Some(order) = order_rx.recv().await {
            println!("Исполнитель: Обрабатываю ордер #{}", order.id);
            let result = executor.execute(order).await;
            result_tx_clone.send(result).await.ok();
        }
        println!("Исполнитель: Все ордера обработаны");
    });

    drop(result_tx);

    // Обработчик результатов
    let result_handler = tokio::spawn(async move {
        let mut filled = 0;
        let mut rejected = 0;

        while let Some(result) = result_rx.recv().await {
            match result {
                ExecutionResult::Filled { order_id, fill_price } => {
                    println!("Результат: Ордер #{} исполнен по цене {}",
                        order_id, fill_price);
                    filled += 1;
                }
                ExecutionResult::Rejected { order_id, reason } => {
                    println!("Результат: Ордер #{} отклонён: {}",
                        order_id, reason);
                    rejected += 1;
                }
                ExecutionResult::PartialFill { order_id, filled_qty, remaining_qty } => {
                    println!("Результат: Ордер #{} частично исполнен: {} / {}",
                        order_id, filled_qty, filled_qty + remaining_qty);
                }
            }
        }

        println!("\n=== Итоги ===");
        println!("Исполнено: {}", filled);
        println!("Отклонено: {}", rejected);
    });

    let _ = tokio::join!(strategy1, strategy2, executor, result_handler);
}
```

## try_send и try_recv: Неблокирующие операции

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct QuickSignal {
    symbol: String,
    urgency: u8,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<QuickSignal>(3);

    // Быстрый отправитель — не хочет ждать
    let sender = tokio::spawn(async move {
        for i in 1..=10 {
            let signal = QuickSignal {
                symbol: format!("SYM{}", i),
                urgency: (i % 3) as u8,
            };

            // try_send не ждёт, возвращает ошибку если канал полон
            match tx.try_send(signal) {
                Ok(_) => println!("Сигнал {} отправлен мгновенно", i),
                Err(mpsc::error::TrySendError::Full(s)) => {
                    println!("Канал полон! Сигнал {} ({}) потерян", i, s.symbol);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    println!("Канал закрыт");
                    break;
                }
            }

            sleep(Duration::from_millis(50)).await;
        }
    });

    // Медленный получатель
    let receiver = tokio::spawn(async move {
        loop {
            // try_recv не ждёт, возвращает ошибку если канал пуст
            match rx.try_recv() {
                Ok(signal) => {
                    println!("Получен сигнал: {} (срочность: {})",
                        signal.symbol, signal.urgency);
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    println!("Канал пуст, делаем другую работу...");
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    println!("Все отправители закрыты");
                    break;
                }
            }

            sleep(Duration::from_millis(200)).await;
        }
    });

    let _ = tokio::join!(sender, receiver);
}
```

## Graceful Shutdown с каналами

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Command {
    Process(String),
    Shutdown,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(10);

    // Рабочий процесс
    let worker = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(Command::Process(data)) => {
                    println!("Обрабатываю: {}", data);
                    sleep(Duration::from_millis(100)).await;
                }
                Some(Command::Shutdown) => {
                    println!("Получена команда на завершение");

                    // Дообрабатываем оставшиеся сообщения
                    while let Ok(cmd) = rx.try_recv() {
                        if let Command::Process(data) = cmd {
                            println!("Дообрабатываю: {}", data);
                        }
                    }

                    println!("Рабочий процесс завершён корректно");
                    break;
                }
                None => {
                    println!("Канал закрыт");
                    break;
                }
            }
        }
    });

    // Главный процесс
    for i in 1..=5 {
        tx.send(Command::Process(format!("Задача {}", i))).await.ok();
    }

    println!("Отправляю команду на завершение...");
    tx.send(Command::Shutdown).await.ok();

    worker.await.ok();
    println!("Программа завершена");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| mpsc channel | Множество отправителей, один получатель |
| Bounded channel | Ограниченный буфер, отправитель ждёт при заполнении |
| Unbounded channel | Неограниченный буфер, может исчерпать память |
| tx.clone() | Создание дополнительного отправителя |
| send().await | Асинхронная отправка с ожиданием |
| try_send() | Неблокирующая отправка |
| recv().await | Асинхронное получение с ожиданием |
| try_recv() | Неблокирующее получение |
| drop(tx) | Закрытие канала (сигнал завершения) |

## Домашнее задание

1. **Агрегатор цен**: Создай систему, где несколько "бирж" (async задач) отправляют обновления цен в один агрегатор. Агрегатор должен выводить лучшую цену bid/ask каждую секунду.

2. **Очередь ордеров с приоритетами**: Модифицируй систему исполнения ордеров так, чтобы рыночные ордера обрабатывались перед лимитными. Используй два канала или поле приоритета.

3. **Rate Limiter**: Создай компонент, который принимает запросы через канал и пропускает не более N запросов в секунду. Остальные либо ставит в очередь, либо отклоняет.

4. **Мониторинг канала**: Добавь метрики к каналу:
   - Количество отправленных сообщений
   - Количество полученных сообщений
   - Текущий размер очереди (можно использовать `capacity()` и `len()` для некоторых каналов)
   - Количество потерянных сообщений (при использовании `try_send`)

## Навигация

[← Предыдущий день](../193-async-mutex-tokio/ru.md) | [Следующий день →](../195-broadcast-channel/ru.md)
