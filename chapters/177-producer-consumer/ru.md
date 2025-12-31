# День 177: Паттерн: Producer-Consumer

## Аналогия из трейдинга

Представь работу реальной биржи: один отдел непрерывно получает рыночные данные — котировки, сделки, изменения в стакане заявок (это **Producer**, производитель). Другой отдел занимается анализом этих данных и принятием торговых решений (это **Consumer**, потребитель). Между ними находится очередь — буфер, куда данные складываются и откуда они забираются для обработки.

Почему это важно в трейдинге:
- **Рыночные данные приходят быстрее**, чем мы можем их обработать — нужен буфер
- **Разная скорость**: получение данных — микросекунды, анализ — миллисекунды
- **Развязка компонентов**: источник данных не зависит от скорости обработки

Паттерн Producer-Consumer решает классическую проблему многопоточности: как безопасно передавать данные между потоками с разной скоростью работы.

## Что такое Producer-Consumer?

**Producer-Consumer** — это паттерн параллельного программирования, где:
- **Producer** (производитель) — создаёт данные и помещает их в общий буфер
- **Consumer** (потребитель) — забирает данные из буфера и обрабатывает
- **Buffer** (буфер/очередь) — синхронизированная структура между ними

```
┌──────────┐     ┌─────────────┐     ┌──────────┐
│ Producer │ --> │   Buffer    │ --> │ Consumer │
│ (данные) │     │  (очередь)  │     │(обработка)│
└──────────┘     └─────────────┘     └──────────┘
```

### Основные характеристики

| Характеристика | Описание |
|----------------|----------|
| Развязка | Producer и Consumer не зависят друг от друга напрямую |
| Буферизация | Сглаживает пики нагрузки |
| Масштабируемость | Можно добавить несколько producers и consumers |
| Потокобезопасность | Буфер синхронизирован для безопасного доступа |

## Реализация на каналах (mpsc)

В Rust самый естественный способ реализации Producer-Consumer — использование каналов `std::sync::mpsc`:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    // Создаём канал (буфер неограниченного размера)
    let (tx, rx) = mpsc::channel();

    // Producer: генерирует рыночные данные
    let producer = thread::spawn(move || {
        let ticks = vec![
            MarketTick { symbol: "BTC".to_string(), price: 42000.0, volume: 1.5, timestamp: 1 },
            MarketTick { symbol: "ETH".to_string(), price: 2200.0, volume: 10.0, timestamp: 2 },
            MarketTick { symbol: "BTC".to_string(), price: 42050.0, volume: 0.8, timestamp: 3 },
            MarketTick { symbol: "ETH".to_string(), price: 2205.0, volume: 5.0, timestamp: 4 },
            MarketTick { symbol: "BTC".to_string(), price: 42100.0, volume: 2.0, timestamp: 5 },
        ];

        for tick in ticks {
            println!("[Producer] Отправляю: {} @ {}", tick.symbol, tick.price);
            tx.send(tick).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        println!("[Producer] Завершил работу");
    });

    // Consumer: обрабатывает рыночные данные
    let consumer = thread::spawn(move || {
        let mut total_btc_volume = 0.0;
        let mut total_eth_volume = 0.0;

        // recv() блокируется до получения данных или закрытия канала
        while let Ok(tick) = rx.recv() {
            println!("[Consumer] Получил: {} @ {} (объём: {})",
                tick.symbol, tick.price, tick.volume);

            match tick.symbol.as_str() {
                "BTC" => total_btc_volume += tick.volume,
                "ETH" => total_eth_volume += tick.volume,
                _ => {}
            }

            // Имитируем обработку
            thread::sleep(Duration::from_millis(150));
        }

        println!("[Consumer] Итого BTC: {}, ETH: {}", total_btc_volume, total_eth_volume);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Ограниченный буфер с sync_channel

Для предотвращения переполнения памяти используйте `sync_channel` с ограниченным буфером:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct TradeSignal {
    symbol: String,
    action: String,    // "BUY" или "SELL"
    price: f64,
    quantity: f64,
}

fn main() {
    // Буфер на 3 элемента — producer будет блокироваться, если буфер полон
    let (tx, rx) = mpsc::sync_channel::<TradeSignal>(3);

    let producer = thread::spawn(move || {
        let signals = vec![
            TradeSignal { symbol: "BTC".into(), action: "BUY".into(), price: 42000.0, quantity: 0.5 },
            TradeSignal { symbol: "ETH".into(), action: "SELL".into(), price: 2200.0, quantity: 5.0 },
            TradeSignal { symbol: "BTC".into(), action: "SELL".into(), price: 42100.0, quantity: 0.3 },
            TradeSignal { symbol: "SOL".into(), action: "BUY".into(), price: 100.0, quantity: 10.0 },
            TradeSignal { symbol: "ETH".into(), action: "BUY".into(), price: 2180.0, quantity: 8.0 },
            TradeSignal { symbol: "BTC".into(), action: "BUY".into(), price: 41900.0, quantity: 1.0 },
        ];

        for signal in signals {
            let start = Instant::now();
            println!("[Producer] Отправляю сигнал: {} {} @ {}",
                signal.action, signal.symbol, signal.price);

            // send() блокируется, если буфер полон
            tx.send(signal).unwrap();

            let elapsed = start.elapsed();
            if elapsed > Duration::from_millis(10) {
                println!("[Producer] Ждал {} мс (буфер был полон)", elapsed.as_millis());
            }
        }
        println!("[Producer] Все сигналы отправлены");
    });

    let consumer = thread::spawn(move || {
        let mut executed = 0;

        while let Ok(signal) = rx.recv() {
            println!("[Consumer] Исполняю: {} {} {} @ {}",
                signal.action, signal.quantity, signal.symbol, signal.price);

            // Медленная обработка — имитирует отправку ордера на биржу
            thread::sleep(Duration::from_millis(500));
            executed += 1;
        }

        println!("[Consumer] Исполнено ордеров: {}", executed);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Множество Producers, один Consumer

Паттерн `mpsc` (multiple producer, single consumer) идеально подходит для сбора данных из нескольких источников:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Producer 1: Binance
    let tx1 = tx.clone();
    let binance = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Binance".to_string(),
                symbol: "BTC/USDT".to_string(),
                bid: 42000.0 + i as f64 * 10.0,
                ask: 42001.0 + i as f64 * 10.0,
            };
            tx1.send(update).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Producer 2: Coinbase
    let tx2 = tx.clone();
    let coinbase = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Coinbase".to_string(),
                symbol: "BTC/USD".to_string(),
                bid: 42005.0 + i as f64 * 8.0,
                ask: 42008.0 + i as f64 * 8.0,
            };
            tx2.send(update).unwrap();
            thread::sleep(Duration::from_millis(120));
        }
    });

    // Producer 3: Kraken
    let tx3 = tx;
    let kraken = thread::spawn(move || {
        for i in 0..3 {
            let update = PriceUpdate {
                exchange: "Kraken".to_string(),
                symbol: "XBTUSD".to_string(),
                bid: 41998.0 + i as f64 * 12.0,
                ask: 42002.0 + i as f64 * 12.0,
            };
            tx3.send(update).unwrap();
            thread::sleep(Duration::from_millis(80));
        }
    });

    // Consumer: агрегирует цены и ищет арбитраж
    let aggregator = thread::spawn(move || {
        let mut best_bid = 0.0;
        let mut best_ask = f64::MAX;
        let mut bid_exchange = String::new();
        let mut ask_exchange = String::new();

        while let Ok(update) = rx.recv() {
            println!("[{}] {} bid: {:.2}, ask: {:.2}",
                update.exchange, update.symbol, update.bid, update.ask);

            if update.bid > best_bid {
                best_bid = update.bid;
                bid_exchange = update.exchange.clone();
            }
            if update.ask < best_ask {
                best_ask = update.ask;
                ask_exchange = update.exchange.clone();
            }

            // Проверяем возможность арбитража
            if best_bid > best_ask {
                println!(">>> АРБИТРАЖ! Купить на {} @ {:.2}, продать на {} @ {:.2}",
                    ask_exchange, best_ask, bid_exchange, best_bid);
            }
        }

        println!("\nЛучший bid: {} на {}", best_bid, bid_exchange);
        println!("Лучший ask: {} на {}", best_ask, ask_exchange);
    });

    binance.join().unwrap();
    coinbase.join().unwrap();
    kraken.join().unwrap();
    aggregator.join().unwrap();
}
```

## Практический пример: Торговый конвейер

Реализуем полноценный конвейер обработки данных:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

// Этап 1: Сырые рыночные данные
#[derive(Debug, Clone)]
struct RawTick {
    symbol: String,
    price: f64,
    volume: f64,
}

// Этап 2: Обогащённые данные с индикаторами
#[derive(Debug)]
struct EnrichedTick {
    symbol: String,
    price: f64,
    volume: f64,
    sma_5: f64,      // Скользящая средняя за 5 тиков
    price_change: f64,
}

// Этап 3: Торговый сигнал
#[derive(Debug)]
struct Signal {
    symbol: String,
    action: String,
    confidence: f64,
    reason: String,
}

fn main() {
    // Канал 1: сырые данные -> обогащение
    let (raw_tx, raw_rx) = mpsc::channel::<RawTick>();

    // Канал 2: обогащённые данные -> генерация сигналов
    let (enriched_tx, enriched_rx) = mpsc::channel::<EnrichedTick>();

    // Канал 3: сигналы -> исполнение
    let (signal_tx, signal_rx) = mpsc::channel::<Signal>();

    // Producer: источник рыночных данных
    let data_source = thread::spawn(move || {
        let ticks = vec![
            RawTick { symbol: "BTC".into(), price: 42000.0, volume: 1.0 },
            RawTick { symbol: "BTC".into(), price: 42050.0, volume: 1.5 },
            RawTick { symbol: "BTC".into(), price: 42100.0, volume: 2.0 },
            RawTick { symbol: "BTC".into(), price: 42080.0, volume: 0.8 },
            RawTick { symbol: "BTC".into(), price: 42150.0, volume: 3.0 },
            RawTick { symbol: "BTC".into(), price: 42200.0, volume: 2.5 },
            RawTick { symbol: "BTC".into(), price: 42180.0, volume: 1.2 },
        ];

        for tick in ticks {
            println!("[DataSource] Новый тик: {} @ {}", tick.symbol, tick.price);
            raw_tx.send(tick).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Consumer/Producer: обогащение данных
    let enricher = thread::spawn(move || {
        let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();
        let mut last_price: HashMap<String, f64> = HashMap::new();

        while let Ok(tick) = raw_rx.recv() {
            let history = price_history.entry(tick.symbol.clone()).or_insert_with(Vec::new);
            history.push(tick.price);

            // Вычисляем SMA за последние 5 тиков
            let sma_5 = if history.len() >= 5 {
                history.iter().rev().take(5).sum::<f64>() / 5.0
            } else {
                history.iter().sum::<f64>() / history.len() as f64
            };

            // Вычисляем изменение цены
            let prev = last_price.get(&tick.symbol).copied().unwrap_or(tick.price);
            let price_change = (tick.price - prev) / prev * 100.0;
            last_price.insert(tick.symbol.clone(), tick.price);

            let enriched = EnrichedTick {
                symbol: tick.symbol,
                price: tick.price,
                volume: tick.volume,
                sma_5,
                price_change,
            };

            println!("[Enricher] SMA: {:.2}, Изменение: {:.3}%",
                enriched.sma_5, enriched.price_change);
            enriched_tx.send(enriched).unwrap();
        }
    });

    // Consumer/Producer: генерация сигналов
    let signal_generator = thread::spawn(move || {
        while let Ok(tick) = enriched_rx.recv() {
            // Простая стратегия: цена выше SMA и растёт = покупаем
            let signal = if tick.price > tick.sma_5 && tick.price_change > 0.1 {
                Some(Signal {
                    symbol: tick.symbol,
                    action: "BUY".into(),
                    confidence: tick.price_change.abs().min(1.0),
                    reason: format!("Цена выше SMA, рост {:.2}%", tick.price_change),
                })
            } else if tick.price < tick.sma_5 && tick.price_change < -0.1 {
                Some(Signal {
                    symbol: tick.symbol,
                    action: "SELL".into(),
                    confidence: tick.price_change.abs().min(1.0),
                    reason: format!("Цена ниже SMA, падение {:.2}%", tick.price_change),
                })
            } else {
                None
            };

            if let Some(sig) = signal {
                println!("[SignalGen] Сигнал: {} {} (уверенность: {:.2})",
                    sig.action, sig.symbol, sig.confidence);
                signal_tx.send(sig).unwrap();
            }
        }
    });

    // Consumer: исполнение ордеров
    let executor = thread::spawn(move || {
        let mut orders = 0;

        while let Ok(signal) = signal_rx.recv() {
            if signal.confidence > 0.15 {
                println!("[Executor] === ИСПОЛНЯЮ: {} {} ===", signal.action, signal.symbol);
                println!("           Причина: {}", signal.reason);
                orders += 1;
            } else {
                println!("[Executor] Пропускаю сигнал (низкая уверенность)");
            }
        }

        println!("\n[Executor] Всего исполнено ордеров: {}", orders);
    });

    data_source.join().unwrap();
    enricher.join().unwrap();
    signal_generator.join().unwrap();
    executor.join().unwrap();
}
```

## Обработка ошибок и graceful shutdown

В реальных системах важно корректно обрабатывать завершение работы:

```rust
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct OrderUpdate {
    order_id: u64,
    status: String,
    filled_qty: f64,
}

fn main() {
    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();

    let shutdown_producer = Arc::clone(&shutdown);
    let producer = thread::spawn(move || {
        let mut order_id = 1;

        while !shutdown_producer.load(Ordering::Relaxed) {
            let update = OrderUpdate {
                order_id,
                status: if order_id % 3 == 0 { "FILLED".into() } else { "PARTIAL".into() },
                filled_qty: (order_id as f64) * 0.1,
            };

            match tx.send(update) {
                Ok(_) => println!("[Producer] Отправил обновление ордера #{}", order_id),
                Err(e) => {
                    println!("[Producer] Канал закрыт: {}", e);
                    break;
                }
            }

            order_id += 1;
            thread::sleep(Duration::from_millis(200));
        }

        println!("[Producer] Получил сигнал завершения");
    });

    let shutdown_consumer = Arc::clone(&shutdown);
    let consumer = thread::spawn(move || {
        let mut processed = 0;

        loop {
            // Используем recv_timeout для проверки shutdown
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(update) => {
                    println!("[Consumer] Ордер #{}: {} (исполнено: {:.2})",
                        update.order_id, update.status, update.filled_qty);
                    processed += 1;
                }
                Err(RecvTimeoutError::Timeout) => {
                    // Проверяем флаг завершения
                    if shutdown_consumer.load(Ordering::Relaxed) {
                        println!("[Consumer] Завершение по флагу");
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    println!("[Consumer] Канал закрыт");
                    break;
                }
            }
        }

        println!("[Consumer] Обработано обновлений: {}", processed);
    });

    // Даём поработать 2 секунды
    thread::sleep(Duration::from_secs(2));

    // Сигнал завершения
    println!("\n[Main] Отправляем сигнал завершения...\n");
    shutdown.store(true, Ordering::Relaxed);

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("Программа завершена корректно");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Producer-Consumer | Паттерн разделения производства и потребления данных |
| `mpsc::channel` | Неограниченный канал, множество producers -> один consumer |
| `mpsc::sync_channel` | Ограниченный буфер, блокирует producer при переполнении |
| Конвейер | Цепочка producer-consumer для поэтапной обработки |
| Graceful shutdown | Корректное завершение с использованием флагов и таймаутов |

## Домашнее задание

1. **Агрегатор котировок**: Реализуй систему с 3 producers (биржи) и 1 consumer. Consumer должен:
   - Хранить последние цены по каждой бирже
   - Выводить лучший bid/ask в реальном времени
   - Детектировать арбитражные возможности (bid на одной бирже > ask на другой)

2. **Риск-менеджер**: Создай конвейер из трёх этапов:
   - Producer: генерирует торговые сигналы
   - Processor 1: фильтрует сигналы по риску (отклоняет слишком большие позиции)
   - Processor 2: вычисляет размер позиции по Kelly criterion
   - Consumer: исполняет ордера и ведёт учёт P&L

3. **Балансировщик нагрузки**: Реализуй систему с 1 producer и 3 consumers:
   - Producer отправляет ордера в очередь
   - Каждый consumer обрабатывает ордера для своей группы символов
   - Используй отдельные каналы для распределения по символам

4. **Мониторинг очереди**: Добавь к любому примеру мониторинг:
   - Счётчик элементов в очереди
   - Среднее время обработки
   - Предупреждение при накоплении очереди > N элементов

## Навигация

[← Предыдущий день](../176-operation-cancellation/ru.md) | [Следующий день →](../178-fan-out-fan-in/ru.md)
