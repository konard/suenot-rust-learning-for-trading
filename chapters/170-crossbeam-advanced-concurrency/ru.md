# День 170: crossbeam: продвинутая многопоточность

## Аналогия из трейдинга

Представь, что ты строишь высокочастотную торговую платформу, которая должна:
- Собирать ценовые фиды с нескольких бирж одновременно
- Обрабатывать тысячи ордеров в секунду
- Координировать несколько торговых стратегий, работающих параллельно
- Делиться рыночными данными между потоками анализа без копирования

Инструменты многопоточности стандартной библиотеки (`std::sync`, каналы `mpsc`) работают хорошо, но для высокопроизводительных торговых систем нужно что-то быстрее и гибче. Здесь на помощь приходит **crossbeam** — набор инструментов для конкурентного программирования, которые быстрее, удобнее и мощнее аналогов из стандартной библиотеки.

Это как апгрейд с обычного торгового терминала до профессиональной низколатентной торговой системы — те же концепции, но оптимизированные для производительности.

## Что такое crossbeam?

`crossbeam` — это крейт (на самом деле семейство крейтов), который предоставляет:

| Компонент | Описание | Применение в трейдинге |
|-----------|----------|------------------------|
| `crossbeam-channel` | Быстрые многопроизводитель-многопотребитель каналы | Очереди ордеров, ценовые фиды |
| `crossbeam-utils` | Scoped потоки, утилиты кеширования | Параллельный анализ данных |
| `crossbeam-epoch` | Epoch-based управление памятью | Lock-free стаканы заявок |
| `crossbeam-deque` | Work-stealing очереди | Балансировка нагрузки стратегий |
| `crossbeam-queue` | Lock-free очереди | Высокопроизводительный обмен сообщениями |

## Настройка crossbeam

Добавь в `Cargo.toml`:

```toml
[dependencies]
crossbeam = "0.8"
# Или отдельные крейты:
# crossbeam-channel = "0.5"
# crossbeam-utils = "0.8"
```

## crossbeam против стандартной библиотеки

### Каналы стандартной библиотеки

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // mpsc = multi-producer, single-consumer (много производителей, один потребитель)
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        tx1.send("Цена с Binance").unwrap();
    });

    thread::spawn(move || {
        tx.send("Цена с Coinbase").unwrap();
    });

    // Возможен только ОДИН получатель
    println!("Получено: {}", rx.recv().unwrap());
    println!("Получено: {}", rx.recv().unwrap());
}
```

### Каналы crossbeam: многопроизводитель, многопотребитель

```rust
use crossbeam::channel;
use std::thread;

fn main() {
    // crossbeam = многопроизводитель, многопотребитель!
    let (tx, rx) = channel::unbounded();

    // Несколько отправителей
    let tx1 = tx.clone();
    let tx2 = tx.clone();

    // Несколько получателей!
    let rx1 = rx.clone();
    let rx2 = rx.clone();

    thread::spawn(move || {
        tx1.send("Цена с Binance: 42000").unwrap();
    });

    thread::spawn(move || {
        tx2.send("Цена с Coinbase: 42005").unwrap();
    });

    // Разные потребители могут получать из одного канала
    let consumer1 = thread::spawn(move || {
        if let Ok(msg) = rx1.recv() {
            println!("Потребитель 1 получил: {}", msg);
        }
    });

    let consumer2 = thread::spawn(move || {
        if let Ok(msg) = rx2.recv() {
            println!("Потребитель 2 получил: {}", msg);
        }
    });

    consumer1.join().unwrap();
    consumer2.join().unwrap();
}
```

## Ограниченные и неограниченные каналы

```rust
use crossbeam::channel;
use std::thread;
use std::time::Duration;

fn main() {
    // Неограниченный: безлимитная ёмкость (осторожно с памятью!)
    let (tx_unbounded, rx_unbounded) = channel::unbounded::<f64>();

    // Ограниченный: лимитированная ёмкость (обратное давление)
    let (tx_bounded, rx_bounded) = channel::bounded::<f64>(100);

    // Производитель: обработчик ордеров
    let tx = tx_bounded.clone();
    thread::spawn(move || {
        for i in 0..1000 {
            let price = 42000.0 + i as f64;
            // Заблокируется, если канал полон — естественное обратное давление!
            tx.send(price).unwrap();
            println!("Отправлена цена: {}", price);
        }
    });

    // Медленный потребитель
    thread::spawn(move || {
        while let Ok(price) = rx_bounded.recv() {
            println!("Обработка цены: {}", price);
            thread::sleep(Duration::from_millis(10)); // Медленная обработка
        }
    });

    thread::sleep(Duration::from_secs(2));
}
```

## Макрос select!: ожидание нескольких каналов

Одна из самых мощных функций crossbeam — `select!` — позволяет ожидать несколько каналов одновременно:

```rust
use crossbeam::channel::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    price: f64,
}

#[derive(Debug)]
struct OrderFill {
    order_id: u64,
    filled_price: f64,
    quantity: f64,
}

fn main() {
    let (price_tx, price_rx): (Sender<PriceUpdate>, Receiver<PriceUpdate>) = channel::unbounded();
    let (order_tx, order_rx): (Sender<OrderFill>, Receiver<OrderFill>) = channel::unbounded();
    let (shutdown_tx, shutdown_rx) = channel::bounded::<()>(1);

    // Симулятор ценового фида
    let price_sender = price_tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            thread::sleep(Duration::from_millis(100));
            price_sender.send(PriceUpdate {
                exchange: "Binance".to_string(),
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            }).unwrap();
        }
    });

    // Симулятор исполнения ордеров
    let order_sender = order_tx.clone();
    thread::spawn(move || {
        for i in 0..3 {
            thread::sleep(Duration::from_millis(150));
            order_sender.send(OrderFill {
                order_id: i + 1,
                filled_price: 41995.0 + i as f64 * 5.0,
                quantity: 0.1,
            }).unwrap();
        }
    });

    // Завершение через 1 секунду
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        let _ = shutdown_tx.send(());
    });

    // Главный цикл событий с использованием select!
    loop {
        crossbeam::channel::select! {
            recv(price_rx) -> msg => {
                match msg {
                    Ok(update) => println!("Цена: {} {} = ${}",
                        update.exchange, update.symbol, update.price),
                    Err(_) => println!("Канал цен закрыт"),
                }
            }
            recv(order_rx) -> msg => {
                match msg {
                    Ok(fill) => println!("Ордер #{} исполнен: {} @ ${}",
                        fill.order_id, fill.quantity, fill.filled_price),
                    Err(_) => println!("Канал ордеров закрыт"),
                }
            }
            recv(shutdown_rx) -> _ => {
                println!("Получен сигнал завершения!");
                break;
            }
        }
    }

    println!("Торговая система остановлена.");
}
```

## Scoped потоки: заимствование данных без Arc

Обычные потоки требуют время жизни `'static` — нужно использовать `move` или `Arc`. Scoped потоки crossbeam позволяют заимствовать данные напрямую:

### Обычные потоки (требуют Arc)

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let prices = Arc::new(vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0]);

    let prices1 = Arc::clone(&prices);
    let prices2 = Arc::clone(&prices);

    let h1 = thread::spawn(move || {
        let sum: f64 = prices1.iter().sum();
        sum / prices1.len() as f64
    });

    let h2 = thread::spawn(move || {
        prices2.iter().cloned().fold(f64::MIN, f64::max)
    });

    println!("Среднее: {}", h1.join().unwrap());
    println!("Максимум: {}", h2.join().unwrap());
}
```

### Scoped потоки (прямое заимствование)

```rust
use crossbeam::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0];
    let volumes = vec![10.5, 20.0, 15.3, 8.7, 25.1];

    // Scoped потоки могут заимствовать локальные данные!
    thread::scope(|s| {
        // Поток 1: Расчёт средней цены
        let avg_handle = s.spawn(|_| {
            let sum: f64 = prices.iter().sum();
            sum / prices.len() as f64
        });

        // Поток 2: Расчёт общего объёма
        let vol_handle = s.spawn(|_| {
            volumes.iter().sum::<f64>()
        });

        // Поток 3: Расчёт VWAP (нужны оба массива!)
        let vwap_handle = s.spawn(|_| {
            let total_value: f64 = prices.iter()
                .zip(volumes.iter())
                .map(|(p, v)| p * v)
                .sum();
            let total_volume: f64 = volumes.iter().sum();
            total_value / total_volume
        });

        // Scoped потоки автоматически объединяются в конце scope
        println!("Средняя цена: ${:.2}", avg_handle.join().unwrap());
        println!("Общий объём: {:.2}", vol_handle.join().unwrap());
        println!("VWAP: ${:.2}", vwap_handle.join().unwrap());
    }).unwrap();

    // prices и volumes всё ещё доступны здесь!
    println!("Исходные данные всё ещё доступны: {} цен", prices.len());
}
```

## Lock-Free очередь: ArrayQueue

Для высокопроизводительных сценариев, где нельзя позволить накладные расходы мьютексов:

```rust
use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    // Lock-free очередь фиксированного размера
    let tick_queue = Arc::new(ArrayQueue::new(1000));

    // Производитель: фид рыночных данных
    let producer_queue = Arc::clone(&tick_queue);
    let producer = thread::spawn(move || {
        for i in 0..100 {
            let tick = MarketTick {
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + (i as f64 * 0.5),
                volume: 0.1 + (i as f64 * 0.01),
                timestamp: 1000000 + i,
            };

            match producer_queue.push(tick) {
                Ok(_) => {}
                Err(tick) => println!("Очередь полна, пропущено: {:?}", tick),
            }
        }
        println!("Производитель завершён: отправлено 100 тиков");
    });

    // Потребитель: обработчик стратегии
    let consumer_queue = Arc::clone(&tick_queue);
    let consumer = thread::spawn(move || {
        let mut count = 0;
        let mut total_volume = 0.0;

        loop {
            match consumer_queue.pop() {
                Some(tick) => {
                    count += 1;
                    total_volume += tick.volume;
                }
                None => {
                    if count >= 100 {
                        break;
                    }
                    thread::yield_now(); // Дать производителю поработать
                }
            }
        }

        println!("Потребитель завершён: обработано {} тиков, общий объём: {:.2}",
            count, total_volume);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Work-Stealing Deque: балансировка нагрузки

Для параллельной обработки, где некоторые задачи занимают больше времени:

```rust
use crossbeam::deque::{Injector, Stealer, Worker};
use std::thread;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct AnalysisTask {
    symbol: String,
    data_points: usize,
}

fn main() {
    // Глобальная очередь для новых задач
    let injector: Arc<Injector<AnalysisTask>> = Arc::new(Injector::new());

    // Создаём воркеров
    let worker1 = Worker::new_fifo();
    let worker2 = Worker::new_fifo();

    // Stealers позволяют воркерам красть задачи друг у друга
    let stealer1 = worker1.stealer();
    let stealer2 = worker2.stealer();

    // Добавляем задачи в глобальную очередь
    for i in 0..20 {
        injector.push(AnalysisTask {
            symbol: format!("ASSET_{}", i),
            data_points: 100 + i * 50, // Разная сложность
        });
    }

    let inj1 = Arc::clone(&injector);
    let inj2 = Arc::clone(&injector);

    // Воркер 1
    let handle1 = thread::spawn(move || {
        let mut processed = 0;
        loop {
            // Сначала пробуем локальную очередь
            let task = worker1.pop()
                // Потом глобальную очередь
                .or_else(|| inj1.steal().success())
                // Потом пробуем украсть у воркера 2
                .or_else(|| stealer2.steal().success());

            match task {
                Some(task) => {
                    // Симулируем анализ
                    thread::sleep(std::time::Duration::from_micros(task.data_points as u64));
                    println!("Воркер 1 проанализировал {} ({} точек)", task.symbol, task.data_points);
                    processed += 1;
                }
                None => {
                    if processed >= 10 {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }
        println!("Воркер 1 выполнил {} задач", processed);
    });

    // Воркер 2
    let handle2 = thread::spawn(move || {
        let mut processed = 0;
        loop {
            let task = worker2.pop()
                .or_else(|| inj2.steal().success())
                .or_else(|| stealer1.steal().success());

            match task {
                Some(task) => {
                    thread::sleep(std::time::Duration::from_micros(task.data_points as u64));
                    println!("Воркер 2 проанализировал {} ({} точек)", task.symbol, task.data_points);
                    processed += 1;
                }
                None => {
                    if processed >= 10 {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }
        println!("Воркер 2 выполнил {} задач", processed);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Практический пример: агрегатор цен с нескольких бирж

```rust
use crossbeam::channel::{self, Sender, Receiver};
use crossbeam::thread;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: Instant,
}

#[derive(Debug)]
struct AggregatedPrice {
    symbol: String,
    best_bid: f64,
    best_bid_exchange: String,
    best_ask: f64,
    best_ask_exchange: String,
    spread: f64,
}

fn simulate_exchange(name: &str, tx: Sender<PriceQuote>, base_price: f64) {
    for i in 0..10 {
        let spread = 5.0 + (i as f64 * 0.5);
        let quote = PriceQuote {
            exchange: name.to_string(),
            symbol: "BTC/USD".to_string(),
            bid: base_price - spread + (i as f64 * 2.0),
            ask: base_price + spread + (i as f64 * 2.0),
            timestamp: Instant::now(),
        };
        tx.send(quote).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn aggregate_prices(quotes: &[PriceQuote]) -> Option<AggregatedPrice> {
    if quotes.is_empty() {
        return None;
    }

    let mut best_bid = f64::MIN;
    let mut best_bid_exchange = String::new();
    let mut best_ask = f64::MAX;
    let mut best_ask_exchange = String::new();

    for quote in quotes {
        if quote.bid > best_bid {
            best_bid = quote.bid;
            best_bid_exchange = quote.exchange.clone();
        }
        if quote.ask < best_ask {
            best_ask = quote.ask;
            best_ask_exchange = quote.exchange.clone();
        }
    }

    Some(AggregatedPrice {
        symbol: quotes[0].symbol.clone(),
        best_bid,
        best_bid_exchange,
        best_ask,
        best_ask_exchange,
        spread: best_ask - best_bid,
    })
}

fn main() {
    let (quote_tx, quote_rx): (Sender<PriceQuote>, Receiver<PriceQuote>) =
        channel::unbounded();
    let (result_tx, result_rx): (Sender<AggregatedPrice>, Receiver<AggregatedPrice>) =
        channel::unbounded();

    thread::scope(|s| {
        // Симуляторы бирж
        let tx1 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Binance", tx1, 42000.0);
        });

        let tx2 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Coinbase", tx2, 42010.0);
        });

        let tx3 = quote_tx.clone();
        s.spawn(move |_| {
            simulate_exchange("Kraken", tx3, 41995.0);
        });

        drop(quote_tx); // Закрываем отправитель, чтобы получатель знал о завершении

        // Поток агрегатора
        let agg_rx = quote_rx.clone();
        let agg_tx = result_tx.clone();
        s.spawn(move |_| {
            let mut quotes_by_exchange: HashMap<String, PriceQuote> = HashMap::new();

            while let Ok(quote) = agg_rx.recv() {
                quotes_by_exchange.insert(quote.exchange.clone(), quote);

                // Агрегируем, когда есть котировки от всех бирж
                if quotes_by_exchange.len() >= 3 {
                    let quotes: Vec<_> = quotes_by_exchange.values().cloned().collect();
                    if let Some(aggregated) = aggregate_prices(&quotes) {
                        agg_tx.send(aggregated).unwrap();
                    }
                }
            }
        });

        drop(result_tx);

        // Обработчик результатов
        s.spawn(move |_| {
            println!("\n=== Агрегированный ценовой фид ===\n");
            while let Ok(agg) = result_rx.recv() {
                println!("{}: Лучший Bid ${:.2} ({}), Лучший Ask ${:.2} ({}), Спред: ${:.2}",
                    agg.symbol,
                    agg.best_bid, agg.best_bid_exchange,
                    agg.best_ask, agg.best_ask_exchange,
                    agg.spread
                );

                // Возможность арбитража?
                if agg.spread < 0.0 {
                    println!("  >>> ВОЗМОЖНОСТЬ АРБИТРАЖА! <<<");
                }
            }
            println!("\n=== Фид закрыт ===");
        });
    }).unwrap();
}
```

## Сравнение производительности

| Функция | std::sync::mpsc | crossbeam-channel |
|---------|-----------------|-------------------|
| Производители | Несколько | Несколько |
| Потребители | Один | Несколько |
| select! | Нет (использовать recv_timeout) | Да |
| Производительность | Хорошая | Лучше (в 2-10 раз быстрее) |
| Ограниченные каналы | Да (sync_channel) | Да |
| Нулевая ёмкость | Нет | Да (rendezvous) |

## Когда использовать crossbeam

| Сценарий | Использовать crossbeam |
|----------|------------------------|
| Нужно несколько потребителей | Да (MPMC каналы) |
| Высокопроизводительный обмен сообщениями | Да (быстрые каналы) |
| Заимствование данных в потоках | Да (scoped потоки) |
| Lock-free структуры данных | Да (очереди, деки) |
| Work-stealing параллелизм | Да (Deque) |
| Простая очередь с одним потребителем | Возможно (std::mpsc достаточно) |

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| crossbeam | Высокопроизводительный набор инструментов многопоточности |
| MPMC каналы | Несколько производителей И несколько потребителей |
| Ограниченные каналы | Обратное давление с лимитом ёмкости |
| Макрос select! | Ожидание нескольких каналов одновременно |
| Scoped потоки | Заимствование данных без Arc/move |
| ArrayQueue | Lock-free очередь фиксированного размера |
| Work-stealing | Балансировка нагрузки с деками |

## Домашнее задание

1. **Объединитель ценовых фидов**: Создай систему с 5 симуляторами бирж, отправляющими цены центральному агрегатору. Используй `select!` для обработки всех фидов и расчёта лучших bid/ask по всем биржам.

2. **Маршрутизатор ордеров**: Реализуй систему маршрутизации ордеров, где:
   - Ордера приходят через ограниченный канал
   - Несколько рабочих потоков обрабатывают ордера
   - Каждый воркер может красть работу у других, когда простаивает
   - Отслеживай общее количество обработанных ордеров каждым воркером

3. **Scoped анализ**: Используя scoped потоки, распараллель расчёт:
   - Simple Moving Average (SMA)
   - Exponential Moving Average (EMA)
   - Relative Strength Index (RSI)
   Все работают с одними и теми же ценовыми данными без использования Arc.

4. **Обработчик таймаутов**: Создай торговую систему, использующую `select!` с таймаутом:
   - Если нет обновления цены 500мс, логировать предупреждение
   - Если нет обновления 2 секунды, инициировать переподключение
   - Корректное завершение по сигналу CTRL+C

## Навигация

[← Предыдущий день](../164-deadlock-threads-block/ru.md) | [Следующий день →](../171-crossbeam-channels/ru.md)
