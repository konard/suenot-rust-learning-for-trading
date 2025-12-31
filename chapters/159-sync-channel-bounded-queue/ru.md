# День 159: sync_channel — ограниченная очередь

## Аналогия из трейдинга

Представь биржевой стакан с ограниченной глубиной. Маркет-мейкер может выставить только определённое количество ордеров. Когда стакан заполнен, новые ордера не принимаются, пока не исполнятся предыдущие. Это и есть **sync_channel** — канал с ограниченной ёмкостью, где отправитель блокируется, если буфер заполнен.

В отличие от обычного `channel()`, который создаёт неограниченную очередь, `sync_channel(n)` создаёт очередь с фиксированным размером `n`. Это критично для торговых систем, где нужен **back-pressure** — механизм замедления быстрых источников данных.

## Базовое использование sync_channel

```rust
use std::sync::mpsc::sync_channel;
use std::thread;

fn main() {
    // Создаём канал с буфером на 3 сообщения
    let (sender, receiver) = sync_channel::<f64>(3);

    // Поток-производитель: отправляет цены
    let producer = thread::spawn(move || {
        let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

        for price in prices {
            println!("[Producer] Отправляю цену: {}", price);
            sender.send(price).unwrap();
            println!("[Producer] Цена {} отправлена", price);
        }
    });

    // Поток-потребитель: обрабатывает цены
    let consumer = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(100));

        while let Ok(price) = receiver.recv() {
            println!("[Consumer] Получена цена: {}", price);
            // Имитация обработки
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

**Важно:** Когда буфер заполнен (3 сообщения), `send()` блокируется до освобождения места.

## Разница между channel и sync_channel

```rust
use std::sync::mpsc::{channel, sync_channel};
use std::thread;
use std::time::Instant;

fn main() {
    // Обычный канал — неограниченный буфер
    println!("=== Обычный channel ===");
    let (tx, rx) = channel::<i32>();
    let start = Instant::now();

    for i in 0..1000 {
        tx.send(i).unwrap();  // Никогда не блокируется
    }
    println!("1000 сообщений отправлено за {:?}", start.elapsed());
    drop(tx);
    drop(rx);

    // Синхронный канал — ограниченный буфер
    println!("\n=== sync_channel(10) ===");
    let (tx, rx) = sync_channel::<i32>(10);

    let sender = thread::spawn(move || {
        let start = Instant::now();
        for i in 0..100 {
            tx.send(i).unwrap();  // Блокируется, когда буфер полон
        }
        println!("100 сообщений отправлено за {:?}", start.elapsed());
    });

    let receiver = thread::spawn(move || {
        while let Ok(_) = rx.recv() {
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();
}
```

## sync_channel(0) — рандеву-канал

```rust
use std::sync::mpsc::sync_channel;
use std::thread;

fn main() {
    // Канал с нулевым буфером — синхронная передача
    let (sender, receiver) = sync_channel::<(String, f64)>(0);

    let order_executor = thread::spawn(move || {
        while let Ok((symbol, price)) = receiver.recv() {
            println!("[Executor] Исполняю ордер: {} @ {:.2}", symbol, price);
            thread::sleep(std::time::Duration::from_millis(100));
            println!("[Executor] Ордер {} исполнен", symbol);
        }
    });

    let order_sender = thread::spawn(move || {
        let orders = [
            ("BTC".to_string(), 42000.0),
            ("ETH".to_string(), 2200.0),
            ("SOL".to_string(), 95.0),
        ];

        for (symbol, price) in orders {
            println!("[Sender] Отправляю ордер: {} @ {:.2}", symbol, price);
            // Блокируется, пока receiver не получит сообщение
            sender.send((symbol.clone(), price)).unwrap();
            println!("[Sender] Ордер {} принят исполнителем", symbol);
        }
    });

    order_sender.join().unwrap();
    order_executor.join().unwrap();
}
```

**Рандеву-канал** гарантирует, что отправитель и получатель синхронизированы — отправка завершается только когда получатель забрал сообщение.

## Практический пример: Rate Limiter для ордеров

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    // Ограничиваем количество ордеров в обработке до 5
    let (order_tx, order_rx) = sync_channel::<Order>(5);
    let (result_tx, result_rx) = sync_channel::<(u64, String)>(5);

    // Обработчик ордеров
    let processor = thread::spawn(move || {
        while let Ok(order) = order_rx.recv() {
            println!("[Processor] Обрабатываю ордер #{}", order.id);

            // Имитация обработки ордера
            thread::sleep(Duration::from_millis(200));

            let result = format!(
                "{} {} {} @ {:.2} - FILLED",
                order.side, order.quantity, order.symbol, order.price
            );
            result_tx.send((order.id, result)).unwrap();
        }
    });

    // Поток сбора результатов
    let collector = thread::spawn(move || {
        while let Ok((id, result)) = result_rx.recv() {
            println!("[Result] Ордер #{}: {}", id, result);
        }
    });

    // Генератор ордеров — будет замедляться из-за back-pressure
    let generator = thread::spawn(move || {
        let start = Instant::now();

        for i in 0..15 {
            let order = Order {
                id: i,
                symbol: "BTCUSDT".to_string(),
                side: if i % 2 == 0 { "BUY".to_string() } else { "SELL".to_string() },
                price: 42000.0 + (i as f64 * 10.0),
                quantity: 0.1,
            };

            let send_start = Instant::now();
            println!("[Generator] Отправляю ордер #{}...", i);
            order_tx.send(order).unwrap();
            println!(
                "[Generator] Ордер #{} принят за {:?} (всего: {:?})",
                i, send_start.elapsed(), start.elapsed()
            );
        }

        println!("\n[Generator] Все 15 ордеров отправлены за {:?}", start.elapsed());
    });

    generator.join().unwrap();
    drop(order_tx);  // Закрываем канал, чтобы processor завершился
    processor.join().unwrap();
    drop(result_tx);  // Закрываем канал результатов
    collector.join().unwrap();
}
```

## Пример: Order Book с ограниченной глубиной

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderBookUpdate {
    Bid { price: u64, quantity: f64 },
    Ask { price: u64, quantity: f64 },
    Clear,
}

struct OrderBook {
    bids: BTreeMap<u64, f64>,  // price -> quantity
    asks: BTreeMap<u64, f64>,
    max_depth: usize,
}

impl OrderBook {
    fn new(max_depth: usize) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            max_depth,
        }
    }

    fn update(&mut self, update: OrderBookUpdate) {
        match update {
            OrderBookUpdate::Bid { price, quantity } => {
                if quantity > 0.0 {
                    self.bids.insert(price, quantity);
                } else {
                    self.bids.remove(&price);
                }
                // Ограничиваем глубину
                while self.bids.len() > self.max_depth {
                    if let Some((&lowest, _)) = self.bids.iter().next() {
                        self.bids.remove(&lowest);
                    }
                }
            }
            OrderBookUpdate::Ask { price, quantity } => {
                if quantity > 0.0 {
                    self.asks.insert(price, quantity);
                } else {
                    self.asks.remove(&price);
                }
                while self.asks.len() > self.max_depth {
                    if let Some((&highest, _)) = self.asks.iter().next_back() {
                        self.asks.remove(&highest);
                    }
                }
            }
            OrderBookUpdate::Clear => {
                self.bids.clear();
                self.asks.clear();
            }
        }
    }

    fn best_bid(&self) -> Option<(u64, f64)> {
        self.bids.iter().next_back().map(|(&p, &q)| (p, q))
    }

    fn best_ask(&self) -> Option<(u64, f64)> {
        self.asks.iter().next().map(|(&p, &q)| (p, q))
    }

    fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }
}

fn main() {
    // Ограничиваем очередь обновлений до 10
    let (update_tx, update_rx) = sync_channel::<OrderBookUpdate>(10);

    // Поток обновления стакана
    let book_handler = thread::spawn(move || {
        let mut book = OrderBook::new(5);  // Глубина 5 уровней

        while let Ok(update) = update_rx.recv() {
            book.update(update);

            if let (Some((bid, bid_qty)), Some((ask, ask_qty))) = (book.best_bid(), book.best_ask()) {
                println!(
                    "📊 BBO: {} x {:.4} | {} x {:.4} | Spread: {}",
                    bid, bid_qty, ask, ask_qty,
                    book.spread().unwrap_or(0)
                );
            }
        }
    });

    // Источник обновлений (имитация WebSocket)
    let feed = thread::spawn(move || {
        let updates = [
            OrderBookUpdate::Bid { price: 42000, quantity: 1.5 },
            OrderBookUpdate::Ask { price: 42010, quantity: 2.0 },
            OrderBookUpdate::Bid { price: 41990, quantity: 0.8 },
            OrderBookUpdate::Ask { price: 42020, quantity: 1.2 },
            OrderBookUpdate::Bid { price: 42000, quantity: 2.0 },  // Обновление
            OrderBookUpdate::Ask { price: 42005, quantity: 0.5 },  // Новый лучший ask
            OrderBookUpdate::Bid { price: 42003, quantity: 3.0 },  // Новый лучший bid
        ];

        for update in updates {
            update_tx.send(update).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    feed.join().unwrap();
    drop(update_tx);
    book_handler.join().unwrap();
}
```

## try_send — неблокирующая отправка

```rust
use std::sync::mpsc::{sync_channel, TrySendError};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct MarketTick {
    symbol: String,
    price: f64,
    timestamp: u64,
}

fn main() {
    // Буфер на 3 тика — старые данные отбрасываются
    let (tx, rx) = sync_channel::<MarketTick>(3);

    // Быстрый источник данных
    let producer = thread::spawn(move || {
        let mut timestamp = 0u64;

        for i in 0..20 {
            let tick = MarketTick {
                symbol: "BTCUSDT".to_string(),
                price: 42000.0 + (i as f64 * 5.0),
                timestamp,
            };
            timestamp += 1;

            match tx.try_send(tick) {
                Ok(()) => println!("[Feed] Тик {} отправлен", i),
                Err(TrySendError::Full(tick)) => {
                    println!("[Feed] Буфер полон, тик {} пропущен (цена: {})", i, tick.price);
                }
                Err(TrySendError::Disconnected(_)) => {
                    println!("[Feed] Канал закрыт");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    // Медленный потребитель
    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(tick) = rx.recv() {
            println!(
                "[Strategy] Обрабатываю тик: {} @ {:.2} (ts: {})",
                tick.symbol, tick.price, tick.timestamp
            );
            count += 1;
            thread::sleep(Duration::from_millis(50));  // Медленная обработка
        }
        println!("[Strategy] Обработано {} тиков", count);
    });

    producer.join().unwrap();
    drop(tx);
    consumer.join().unwrap();
}
```

## Пример: Pipeline обработки торговых данных

```rust
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct RawTick {
    symbol: String,
    price: f64,
    volume: f64,
}

#[derive(Debug)]
struct NormalizedTick {
    symbol: String,
    price_usd: f64,
    volume_usd: f64,
}

#[derive(Debug)]
struct Signal {
    symbol: String,
    action: String,
    strength: f64,
}

fn main() {
    // Pipeline с ограниченными буферами
    let (raw_tx, raw_rx) = sync_channel::<RawTick>(5);
    let (normalized_tx, normalized_rx) = sync_channel::<NormalizedTick>(3);
    let (signal_tx, signal_rx) = sync_channel::<Signal>(2);

    // Этап 1: Нормализация данных
    let normalizer = thread::spawn(move || {
        let btc_usd = 42000.0;  // Курс для конвертации

        while let Ok(tick) = raw_rx.recv() {
            let normalized = NormalizedTick {
                symbol: tick.symbol,
                price_usd: tick.price * btc_usd,
                volume_usd: tick.volume * tick.price * btc_usd,
            };
            println!("[Normalizer] {} -> {:.2} USD", normalized.symbol, normalized.price_usd);
            normalized_tx.send(normalized).unwrap();
        }
    });

    // Этап 2: Генерация сигналов
    let signal_generator = thread::spawn(move || {
        let mut last_price = 0.0;

        while let Ok(tick) = normalized_rx.recv() {
            let strength = if last_price > 0.0 {
                (tick.price_usd - last_price) / last_price
            } else {
                0.0
            };

            let action = if strength > 0.001 {
                "BUY"
            } else if strength < -0.001 {
                "SELL"
            } else {
                "HOLD"
            };

            let signal = Signal {
                symbol: tick.symbol,
                action: action.to_string(),
                strength: strength.abs(),
            };

            println!("[SignalGen] {} -> {} ({:.4})", signal.symbol, signal.action, signal.strength);
            signal_tx.send(signal).unwrap();
            last_price = tick.price_usd;
        }
    });

    // Этап 3: Исполнение сигналов
    let executor = thread::spawn(move || {
        while let Ok(signal) = signal_rx.recv() {
            if signal.action != "HOLD" && signal.strength > 0.002 {
                println!(
                    "[Executor] ИСПОЛНЯЮ: {} {} (сила: {:.4})",
                    signal.action, signal.symbol, signal.strength
                );
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Источник данных
    let ticks = vec![
        RawTick { symbol: "ETHBTC".to_string(), price: 0.052, volume: 10.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.053, volume: 15.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.051, volume: 20.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.054, volume: 12.0 },
        RawTick { symbol: "ETHBTC".to_string(), price: 0.055, volume: 8.0 },
    ];

    for tick in ticks {
        raw_tx.send(tick).unwrap();
        thread::sleep(Duration::from_millis(50));
    }

    drop(raw_tx);
    normalizer.join().unwrap();
    drop(normalized_tx);
    signal_generator.join().unwrap();
    drop(signal_tx);
    executor.join().unwrap();
}
```

## Выбор размера буфера

```rust
use std::sync::mpsc::sync_channel;

fn main() {
    // Размер буфера зависит от сценария:

    // 0 — Рандеву: строгая синхронизация отправителя и получателя
    // Используй для критических операций, где важен порядок
    let (_tx, _rx) = sync_channel::<i32>(0);

    // 1-10 — Малый буфер: минимальная задержка, жёсткий back-pressure
    // Используй для торговых сигналов в реальном времени
    let (_tx, _rx) = sync_channel::<i32>(5);

    // 10-100 — Средний буфер: баланс между задержкой и пропускной способностью
    // Используй для обработки рыночных данных
    let (_tx, _rx) = sync_channel::<i32>(50);

    // 100+ — Большой буфер: высокая пропускная способность, возможна задержка
    // Используй для логирования, аналитики
    let (_tx, _rx) = sync_channel::<i32>(1000);

    println!("Размер буфера выбирается исходя из требований к задержке и пропускной способности");
}
```

## Сравнение channel и sync_channel

| Характеристика | `channel()` | `sync_channel(n)` |
|---------------|-------------|-------------------|
| Размер буфера | Неограничен | Фиксирован (n) |
| `send()` блокируется | Никогда | Когда буфер полон |
| Память | Растёт неограниченно | Ограничена |
| Back-pressure | Нет | Есть |
| Использование | Логирование, события | Торговые данные, потоки |

## Что мы узнали

- `sync_channel(n)` создаёт канал с буфером размера `n`
- `sync_channel(0)` создаёт рандеву-канал — отправитель ждёт получателя
- `send()` блокируется, когда буфер заполнен
- `try_send()` возвращает ошибку вместо блокировки
- Back-pressure помогает контролировать поток данных в торговых системах

## Домашнее задание

1. Реализуй систему rate limiting для API запросов к бирже с использованием `sync_channel(10)` — не более 10 запросов в очереди

2. Создай pipeline обработки свечей (OHLCV): получение -> расчёт индикаторов -> генерация сигналов, где каждый этап использует `sync_channel` с разными размерами буфера

3. Реализуй систему маршрутизации ордеров: один источник, несколько обработчиков по разным биржам, с использованием `try_send()` для пропуска перегруженных направлений

4. Создай симуляцию торгового бота с рандеву-каналом (`sync_channel(0)`) для синхронизации отправки и подтверждения ордеров

## Навигация

[← Предыдущий день](../158-channel-producer-consumer/ru.md) | [Следующий день →](../160-select-macro-multiplexing/ru.md)
