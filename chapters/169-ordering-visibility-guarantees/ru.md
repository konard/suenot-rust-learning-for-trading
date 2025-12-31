# День 169: Ordering: гарантии видимости

## Аналогия из трейдинга

Представь биржу, где работают два трейдера. Трейдер A обновляет цену актива на экране, а трейдер B принимает решения на основе этой цены. Критически важно, чтобы B видел актуальную цену, а не устаревшие данные из кэша.

В мире процессоров аналогичная проблема: каждое ядро имеет свой кэш, и изменения переменной в одном потоке могут быть не сразу видны в другом. **Memory Ordering** (упорядочивание памяти) определяет, какие гарантии видимости мы получаем при работе с атомарными операциями.

В трейдинге это может проявиться так:
- Поток A записывает новую цену BTC
- Поток B проверяет флаг "цена обновлена" и читает цену
- Без правильного ordering поток B может увидеть флаг = true, но прочитать старую цену!

## Что такое Memory Ordering?

Memory Ordering — это набор гарантий о порядке выполнения операций с памятью между разными потоками. В Rust для атомарных типов используется перечисление `std::sync::atomic::Ordering`:

```rust
use std::sync::atomic::Ordering;

// Доступные варианты:
// Ordering::Relaxed   — минимальные гарантии
// Ordering::Acquire   — гарантии при чтении
// Ordering::Release   — гарантии при записи
// Ordering::AcqRel    — комбинация Acquire + Release
// Ordering::SeqCst    — последовательная согласованность (максимальные гарантии)
```

## Ordering::Relaxed — минимальные гарантии

`Relaxed` гарантирует только атомарность операции — никаких гарантий порядка относительно других операций. Подходит для счётчиков, где важно только финальное значение.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Счётчик исполненных ордеров — нам важно только итоговое число
    let orders_executed = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    for trader_id in 0..4 {
        let counter = Arc::clone(&orders_executed);

        let handle = thread::spawn(move || {
            for order in 0..100 {
                // Relaxed достаточно — нам важна только атомарность инкремента
                counter.fetch_add(1, Ordering::Relaxed);

                if order % 25 == 0 {
                    println!("Трейдер {}: исполнил ордер #{}", trader_id, order);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Финальное значение всегда корректно
    println!("Всего исполнено ордеров: {}",
        orders_executed.load(Ordering::Relaxed));
}
```

### Когда использовать Relaxed

- Простые счётчики (количество сделок, запросов)
- Статистика, где важен только итог
- Случаи, когда порядок операций не важен

## Ordering::Acquire и Ordering::Release — синхронизация данных

`Release` и `Acquire` работают в паре и обеспечивают гарантию "happens-before":
- **Release** при записи: все предыдущие записи завершены
- **Acquire** при чтении: все последующие чтения увидят данные после Release

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Структура для публикации цены
struct PricePublisher {
    price: AtomicU64,        // Цена в копейках (чтобы избежать f64)
    ready: AtomicBool,       // Флаг готовности
}

fn main() {
    let publisher = Arc::new(PricePublisher {
        price: AtomicU64::new(0),
        ready: AtomicBool::new(false),
    });

    let pub_clone = Arc::clone(&publisher);
    let sub_clone = Arc::clone(&publisher);

    // Поток-издатель: обновляет цену
    let publisher_handle = thread::spawn(move || {
        // Симулируем получение новой цены с биржи
        thread::sleep(Duration::from_millis(50));

        let new_price = 42_500_00_u64; // $42,500.00 в копейках

        // Сначала записываем цену (обычная атомарная запись)
        pub_clone.price.store(new_price, Ordering::Relaxed);

        // Release гарантирует: ВСЕ предыдущие записи (включая price)
        // будут видны после того, как другой поток прочитает ready = true с Acquire
        pub_clone.ready.store(true, Ordering::Release);

        println!("Издатель: Цена обновлена до ${:.2}", new_price as f64 / 100.0);
    });

    // Поток-подписчик: читает цену
    let subscriber_handle = thread::spawn(move || {
        // Ждём, пока цена будет готова
        loop {
            // Acquire гарантирует: если мы видим ready = true,
            // то мы также увидим ВСЕ записи, сделанные ДО Release
            if sub_clone.ready.load(Ordering::Acquire) {
                // Теперь безопасно читать цену — она точно актуальна
                let price = sub_clone.price.load(Ordering::Relaxed);
                println!("Подписчик: Получена цена ${:.2}", price as f64 / 100.0);
                break;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    publisher_handle.join().unwrap();
    subscriber_handle.join().unwrap();
}
```

### Визуализация Acquire-Release

```
Поток A (Publisher)          Поток B (Subscriber)
       |                            |
  price = 42500                     |
       |                            |
  [RELEASE]                         |
  ready = true  --------→    ready == true?
       |                     [ACQUIRE]
       |                            |
       |                     price = 42500 ✓
       |                            |
```

## Пример: Торговый сигнал с данными

В HFT-системах часто нужно передать не только сигнал, но и связанные данные:

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradeSignal {
    // Данные сигнала
    symbol_id: AtomicU64,
    price: AtomicU64,         // В копейках
    quantity: AtomicI64,      // Положительное = покупка, отрицательное = продажа

    // Флаг готовности сигнала
    signal_ready: AtomicBool,
}

impl TradeSignal {
    fn new() -> Self {
        TradeSignal {
            symbol_id: AtomicU64::new(0),
            price: AtomicU64::new(0),
            quantity: AtomicI64::new(0),
            signal_ready: AtomicBool::new(false),
        }
    }

    // Публикация сигнала (вызывается из потока анализа)
    fn publish(&self, symbol: u64, price: u64, qty: i64) {
        // Записываем данные (порядок не важен между собой)
        self.symbol_id.store(symbol, Ordering::Relaxed);
        self.price.store(price, Ordering::Relaxed);
        self.quantity.store(qty, Ordering::Relaxed);

        // Release — все записи выше становятся видимыми
        self.signal_ready.store(true, Ordering::Release);
    }

    // Получение сигнала (вызывается из потока исполнения)
    fn consume(&self) -> Option<(u64, u64, i64)> {
        // Acquire — если видим true, все данные актуальны
        if self.signal_ready.load(Ordering::Acquire) {
            let symbol = self.symbol_id.load(Ordering::Relaxed);
            let price = self.price.load(Ordering::Relaxed);
            let qty = self.quantity.load(Ordering::Relaxed);

            // Сбрасываем флаг (можно Relaxed, т.к. мы единственный читатель)
            self.signal_ready.store(false, Ordering::Relaxed);

            Some((symbol, price, qty))
        } else {
            None
        }
    }
}

fn main() {
    let signal = Arc::new(TradeSignal::new());

    let signal_producer = Arc::clone(&signal);
    let signal_consumer = Arc::clone(&signal);

    // Поток анализа рынка
    let analyst = thread::spawn(move || {
        // Симуляция анализа
        thread::sleep(Duration::from_millis(100));

        // BTC (id=1), цена $42,500, покупка 10 единиц
        signal_producer.publish(1, 42_500_00, 10);
        println!("Аналитик: Сигнал на покупку опубликован");

        thread::sleep(Duration::from_millis(100));

        // ETH (id=2), цена $2,200, продажа 50 единиц
        signal_producer.publish(2, 2_200_00, -50);
        println!("Аналитик: Сигнал на продажу опубликован");
    });

    // Поток исполнения ордеров
    let executor = thread::spawn(move || {
        let mut executed = 0;

        while executed < 2 {
            if let Some((symbol, price, qty)) = signal_consumer.consume() {
                let action = if qty > 0 { "ПОКУПКА" } else { "ПРОДАЖА" };
                let symbol_name = match symbol {
                    1 => "BTC",
                    2 => "ETH",
                    _ => "UNKNOWN",
                };

                println!(
                    "Исполнитель: {} {} {} по ${:.2}",
                    action,
                    qty.abs(),
                    symbol_name,
                    price as f64 / 100.0
                );

                executed += 1;
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    analyst.join().unwrap();
    executor.join().unwrap();
}
```

## Ordering::AcqRel — для read-modify-write операций

`AcqRel` объединяет `Acquire` и `Release` для операций вроде `fetch_add`, `compare_exchange`:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

struct OrderBook {
    // Лучшая цена покупки (bid) и продажи (ask)
    best_bid: AtomicU64,
    best_ask: AtomicU64,
}

impl OrderBook {
    fn new(bid: u64, ask: u64) -> Self {
        OrderBook {
            best_bid: AtomicU64::new(bid),
            best_ask: AtomicU64::new(ask),
        }
    }

    // Попытка улучшить bid (конкурентное обновление)
    fn try_improve_bid(&self, new_bid: u64) -> bool {
        let mut current = self.best_bid.load(Ordering::Relaxed);

        loop {
            // Новый bid должен быть выше текущего
            if new_bid <= current {
                return false;
            }

            // AcqRel: читаем текущее значение (Acquire) и записываем новое (Release)
            match self.best_bid.compare_exchange_weak(
                current,
                new_bid,
                Ordering::AcqRel,  // Успех: и Acquire, и Release
                Ordering::Relaxed  // Неудача: просто читаем заново
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    // Попытка улучшить ask (должен быть ниже)
    fn try_improve_ask(&self, new_ask: u64) -> bool {
        let mut current = self.best_ask.load(Ordering::Relaxed);

        loop {
            if new_ask >= current {
                return false;
            }

            match self.best_ask.compare_exchange_weak(
                current,
                new_ask,
                Ordering::AcqRel,
                Ordering::Relaxed
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    fn get_spread(&self) -> i64 {
        let ask = self.best_ask.load(Ordering::Acquire) as i64;
        let bid = self.best_bid.load(Ordering::Acquire) as i64;
        ask - bid
    }
}

fn main() {
    let book = Arc::new(OrderBook::new(42_000_00, 42_010_00));

    let mut handles = vec![];

    // Несколько маркет-мейкеров конкурируют за лучшие цены
    for mm_id in 0..4 {
        let book_clone = Arc::clone(&book);

        let handle = thread::spawn(move || {
            for i in 0..10 {
                // Каждый MM пытается улучшить bid и ask
                let new_bid = 42_000_00 + mm_id * 100 + i * 10;
                let new_ask = 42_010_00 - mm_id * 100 - i * 10;

                if book_clone.try_improve_bid(new_bid) {
                    println!(
                        "MM{}: Улучшен bid до ${:.2}",
                        mm_id,
                        new_bid as f64 / 100.0
                    );
                }

                if book_clone.try_improve_ask(new_ask) {
                    println!(
                        "MM{}: Улучшен ask до ${:.2}",
                        mm_id,
                        new_ask as f64 / 100.0
                    );
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "\nИтоговый спред: ${:.2}",
        book.get_spread() as f64 / 100.0
    );
}
```

## Ordering::SeqCst — последовательная согласованность

`SeqCst` (Sequentially Consistent) — самый строгий режим. Гарантирует единый глобальный порядок всех операций для всех потоков. Используйте когда порядок между разными атомарными переменными критически важен.

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingHalt {
    halt_flag: AtomicBool,
    last_price: AtomicU64,
    halt_reason: AtomicU64, // 1 = волатильность, 2 = новости, 3 = техничесий сбой
}

impl TradingHalt {
    fn new() -> Self {
        TradingHalt {
            halt_flag: AtomicBool::new(false),
            last_price: AtomicU64::new(0),
            halt_reason: AtomicU64::new(0),
        }
    }

    // Остановка торгов — все потоки должны видеть согласованное состояние
    fn halt_trading(&self, price: u64, reason: u64) {
        // SeqCst гарантирует, что все потоки увидят эти операции в одном порядке
        self.last_price.store(price, Ordering::SeqCst);
        self.halt_reason.store(reason, Ordering::SeqCst);
        self.halt_flag.store(true, Ordering::SeqCst);
    }

    fn is_halted(&self) -> bool {
        self.halt_flag.load(Ordering::SeqCst)
    }

    fn get_halt_info(&self) -> (u64, u64) {
        // SeqCst гарантирует согласованное чтение
        let price = self.last_price.load(Ordering::SeqCst);
        let reason = self.halt_reason.load(Ordering::SeqCst);
        (price, reason)
    }

    fn resume_trading(&self) {
        self.halt_flag.store(false, Ordering::SeqCst);
    }
}

fn main() {
    let halt_system = Arc::new(TradingHalt::new());

    // Поток мониторинга — следит за волатильностью
    let monitor = {
        let system = Arc::clone(&halt_system);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));

            // Обнаружена высокая волатильность!
            println!("Монитор: Высокая волатильность, останавливаем торги!");
            system.halt_trading(41_500_00, 1);
        })
    };

    // Несколько торговых потоков
    let mut traders = vec![];
    for id in 0..3 {
        let system = Arc::clone(&halt_system);
        traders.push(thread::spawn(move || {
            for i in 0..10 {
                if system.is_halted() {
                    let (price, reason) = system.get_halt_info();
                    let reason_str = match reason {
                        1 => "волатильность",
                        2 => "новости",
                        3 => "тех. сбой",
                        _ => "неизвестно",
                    };
                    println!(
                        "Трейдер {}: Торги остановлены! Причина: {}, цена: ${:.2}",
                        id,
                        reason_str,
                        price as f64 / 100.0
                    );
                    break;
                }

                println!("Трейдер {}: Работаю, итерация {}", id, i);
                thread::sleep(Duration::from_millis(30));
            }
        }));
    }

    monitor.join().unwrap();
    for t in traders {
        t.join().unwrap();
    }
}
```

## Сравнение режимов Ordering

| Ordering | Гарантии | Производительность | Применение |
|----------|----------|-------------------|------------|
| Relaxed | Только атомарность | Максимальная | Счётчики, статистика |
| Acquire | Видимость записей до Release | Высокая | Чтение флагов, указателей |
| Release | Предыдущие записи видны после Acquire | Высокая | Публикация данных |
| AcqRel | Acquire + Release | Средняя | compare_exchange, fetch_* |
| SeqCst | Глобальный порядок | Низкая | Критические секции, алгоритмы синхронизации |

## Практический пример: Lock-free очередь цен

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const QUEUE_SIZE: usize = 16;

struct PriceQueue {
    buffer: [AtomicU64; QUEUE_SIZE],
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

impl PriceQueue {
    fn new() -> Self {
        PriceQueue {
            buffer: std::array::from_fn(|_| AtomicU64::new(0)),
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
        }
    }

    // Производитель записывает цену
    fn push(&self, price: u64) -> bool {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Acquire);

        // Проверяем, есть ли место
        if write.wrapping_sub(read) >= QUEUE_SIZE {
            return false; // Очередь полна
        }

        let index = write % QUEUE_SIZE;

        // Записываем цену
        self.buffer[index].store(price, Ordering::Relaxed);

        // Release: запись цены видна после инкремента позиции
        self.write_pos.store(write.wrapping_add(1), Ordering::Release);

        true
    }

    // Потребитель читает цену
    fn pop(&self) -> Option<u64> {
        let read = self.read_pos.load(Ordering::Relaxed);

        // Acquire: видим все записи до Release в write_pos
        let write = self.write_pos.load(Ordering::Acquire);

        if read == write {
            return None; // Очередь пуста
        }

        let index = read % QUEUE_SIZE;
        let price = self.buffer[index].load(Ordering::Relaxed);

        // Release: следующий потребитель увидит обновлённую позицию
        self.read_pos.store(read.wrapping_add(1), Ordering::Release);

        Some(price)
    }
}

fn main() {
    let queue = Arc::new(PriceQueue::new());

    // Производитель: подаёт цены с биржи
    let producer = {
        let q = Arc::clone(&queue);
        thread::spawn(move || {
            for i in 0..20 {
                let price = 42_000_00 + i * 100; // Цена растёт

                while !q.push(price) {
                    // Очередь полна, ждём
                    thread::sleep(Duration::from_micros(100));
                }

                println!("Производитель: записал цену ${:.2}", price as f64 / 100.0);
                thread::sleep(Duration::from_millis(10));
            }
        })
    };

    // Потребитель: обрабатывает цены
    let consumer = {
        let q = Arc::clone(&queue);
        thread::spawn(move || {
            let mut count = 0;
            while count < 20 {
                if let Some(price) = q.pop() {
                    println!("Потребитель: обработал цену ${:.2}", price as f64 / 100.0);
                    count += 1;
                } else {
                    thread::sleep(Duration::from_micros(100));
                }
            }
        })
    };

    producer.join().unwrap();
    consumer.join().unwrap();

    println!("Все цены обработаны!");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Memory Ordering | Гарантии видимости изменений между потоками |
| Relaxed | Только атомарность, без гарантий порядка |
| Acquire | Гарантирует видимость всех записей до Release |
| Release | Делает предыдущие записи видимыми для Acquire |
| AcqRel | Комбинация для read-modify-write операций |
| SeqCst | Глобальный порядок, максимальные гарантии |

## Домашнее задание

1. **Счётчик с гарантиями**: Создай структуру `TradeCounter` с атомарными счётчиками для покупок и продаж. Реализуй метод `get_net_position()`, который возвращает разницу. Какой Ordering нужен для каждой операции?

2. **Публикация OHLCV**: Реализуй структуру `OHLCVPublisher` с полями Open, High, Low, Close, Volume (все AtomicU64). Один поток обновляет данные, другой читает. Используй Acquire-Release для гарантии согласованности.

3. **Флаг экстренной остановки**: Создай систему с тремя потоками:
   - Поток A устанавливает флаг остановки
   - Поток B проверяет флаг и читает причину остановки
   - Поток C проверяет флаг и читает время остановки
   Используй SeqCst чтобы все потоки видели согласованное состояние.

4. **Lock-free стек**: Модифицируй пример с очередью для реализации LIFO-стека цен. Используй compare_exchange_weak с правильным Ordering.

## Навигация

[← Предыдущий день](../168-atomic-compare-exchange/ru.md) | [Следующий день →](../170-memory-barriers/ru.md)
