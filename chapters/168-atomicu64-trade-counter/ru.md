# День 168: AtomicU64: счётчик сделок

## Аналогия из трейдинга

Представь торговую платформу, которая обрабатывает тысячи сделок в секунду. Каждая сделка должна получить уникальный идентификатор — номер транзакции. Если использовать обычный счётчик с `Mutex`, то каждый поток будет ждать своей очереди, чтобы увеличить счётчик. Это создаёт "узкое горлышко" — потоки тратят время на ожидание вместо обработки сделок.

**AtomicU64** — это как электронный табло на бирже, которое мгновенно обновляется. Каждый трейдер может "атомарно" (неделимо) увеличить счётчик за одну операцию без блокировки других трейдеров. Это позволяет генерировать миллионы уникальных ID в секунду без конфликтов.

В реальном трейдинге `AtomicU64` используется для:
- Генерации уникальных ID для ордеров и транзакций
- Подсчёта количества исполненных сделок
- Отслеживания объёма торгов в реальном времени
- Сбора статистики: количество запросов к API, latency метрики

## Что такое AtomicU64?

`AtomicU64` — это атомарный 64-битный беззнаковый целочисленный тип из модуля `std::sync::atomic`. "Атомарный" означает, что операции над ним выполняются как единое, неделимое действие — никакой другой поток не может увидеть промежуточное состояние.

### Основные операции

| Метод | Описание |
|-------|----------|
| `new(value)` | Создание с начальным значением |
| `load(ordering)` | Чтение текущего значения |
| `store(value, ordering)` | Запись нового значения |
| `fetch_add(value, ordering)` | Добавить и вернуть старое значение |
| `fetch_sub(value, ordering)` | Вычесть и вернуть старое значение |
| `swap(value, ordering)` | Заменить и вернуть старое значение |
| `compare_exchange(current, new, success, failure)` | Условная замена |

### Memory Ordering

Параметр `Ordering` определяет гарантии синхронизации памяти:

| Ordering | Описание | Применение |
|----------|----------|------------|
| `Relaxed` | Минимальные гарантии, только атомарность | Счётчики, статистика |
| `Acquire` | Все последующие операции видят изменения | Чтение при синхронизации |
| `Release` | Все предыдущие операции видны другим потокам | Запись при синхронизации |
| `AcqRel` | Комбинация Acquire и Release | Read-modify-write |
| `SeqCst` | Строгий порядок для всех потоков | Когда важен глобальный порядок |

Для простых счётчиков обычно достаточно `Relaxed`, для синхронизации между потоками используют более строгие гарантии.

## Простой счётчик сделок

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Атомарный счётчик сделок
    let trade_counter = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    // 10 торговых потоков
    for trader_id in 0..10 {
        let counter = Arc::clone(&trade_counter);

        let handle = thread::spawn(move || {
            // Каждый трейдер совершает 100 сделок
            for _ in 0..100 {
                // fetch_add возвращает предыдущее значение
                let trade_id = counter.fetch_add(1, Ordering::Relaxed);

                // Используем trade_id для логирования (выводим только каждую 50-ю)
                if trade_id % 50 == 0 {
                    println!("Трейдер {}: сделка #{}", trader_id, trade_id);
                }
            }
        });

        handles.push(handle);
    }

    // Ждём завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }

    // Читаем финальное значение
    let total = trade_counter.load(Ordering::Relaxed);
    println!("\nВсего совершено сделок: {}", total);
}
```

## Генератор уникальных ID для ордеров

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Генератор уникальных идентификаторов ордеров
struct OrderIdGenerator {
    /// Счётчик в младших 32 битах
    counter: AtomicU64,
    /// Префикс (timestamp при старте) в старших 32 битах
    prefix: u64,
}

impl OrderIdGenerator {
    fn new() -> Self {
        // Используем timestamp как префикс для уникальности между перезапусками
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        OrderIdGenerator {
            counter: AtomicU64::new(0),
            prefix: (timestamp & 0xFFFFFFFF) << 32,
        }
    }

    /// Генерирует уникальный ID для нового ордера
    fn next_order_id(&self) -> u64 {
        let sequence = self.counter.fetch_add(1, Ordering::Relaxed);
        self.prefix | (sequence & 0xFFFFFFFF)
    }

    /// Возвращает количество сгенерированных ID
    fn generated_count(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: &'static str,
    quantity: f64,
    price: f64,
}

fn main() {
    let generator = Arc::new(OrderIdGenerator::new());

    let mut handles = vec![];

    // Симуляция нескольких торговых стратегий
    for strategy in 0..4 {
        let gen = Arc::clone(&generator);

        let handle = thread::spawn(move || {
            let symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT"];
            let mut orders = Vec::new();

            for i in 0..25 {
                let order = Order {
                    id: gen.next_order_id(),
                    symbol: symbols[i % symbols.len()],
                    quantity: (i as f64 + 1.0) * 0.1,
                    price: 40000.0 + (i as f64 * 100.0),
                };
                orders.push(order);
            }

            println!("Стратегия {} создала {} ордеров", strategy, orders.len());
            orders
        });

        handles.push(handle);
    }

    // Собираем все ордера
    let mut all_orders = Vec::new();
    for handle in handles {
        let orders = handle.join().unwrap();
        all_orders.extend(orders);
    }

    println!("\nВсего создано ордеров: {}", all_orders.len());
    println!("Сгенерировано ID: {}", generator.generated_count());

    // Проверяем уникальность ID
    let mut ids: Vec<u64> = all_orders.iter().map(|o| o.id).collect();
    ids.sort();
    ids.dedup();

    if ids.len() == all_orders.len() {
        println!("Все ID уникальны!");
    } else {
        println!("ОШИБКА: обнаружены дубликаты ID!");
    }

    // Показываем несколько ордеров
    println!("\nПримеры ордеров:");
    for order in all_orders.iter().take(5) {
        println!("  {:?}", order);
    }
}
```

## Многопоточная торговая статистика

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Статистика торговой системы
struct TradingStats {
    /// Количество исполненных ордеров на покупку
    buy_orders: AtomicU64,
    /// Количество исполненных ордеров на продажу
    sell_orders: AtomicU64,
    /// Общий объём торгов (в центах для точности)
    total_volume_cents: AtomicU64,
    /// Количество отменённых ордеров
    cancelled_orders: AtomicU64,
    /// Количество ошибок
    errors: AtomicU64,
}

impl TradingStats {
    fn new() -> Self {
        TradingStats {
            buy_orders: AtomicU64::new(0),
            sell_orders: AtomicU64::new(0),
            total_volume_cents: AtomicU64::new(0),
            cancelled_orders: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    fn record_buy(&self, volume: f64) {
        self.buy_orders.fetch_add(1, Ordering::Relaxed);
        let cents = (volume * 100.0) as u64;
        self.total_volume_cents.fetch_add(cents, Ordering::Relaxed);
    }

    fn record_sell(&self, volume: f64) {
        self.sell_orders.fetch_add(1, Ordering::Relaxed);
        let cents = (volume * 100.0) as u64;
        self.total_volume_cents.fetch_add(cents, Ordering::Relaxed);
    }

    fn record_cancel(&self) {
        self.cancelled_orders.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            buy_orders: self.buy_orders.load(Ordering::Relaxed),
            sell_orders: self.sell_orders.load(Ordering::Relaxed),
            total_volume: self.total_volume_cents.load(Ordering::Relaxed) as f64 / 100.0,
            cancelled_orders: self.cancelled_orders.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct StatsSnapshot {
    buy_orders: u64,
    sell_orders: u64,
    total_volume: f64,
    cancelled_orders: u64,
    errors: u64,
}

impl StatsSnapshot {
    fn total_orders(&self) -> u64 {
        self.buy_orders + self.sell_orders
    }

    fn buy_ratio(&self) -> f64 {
        if self.total_orders() == 0 {
            0.0
        } else {
            self.buy_orders as f64 / self.total_orders() as f64 * 100.0
        }
    }
}

fn main() {
    let stats = Arc::new(TradingStats::new());
    let start = Instant::now();

    let mut handles = vec![];

    // Поток мониторинга
    let stats_monitor = Arc::clone(&stats);
    let monitor_handle = thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            let snap = stats_monitor.snapshot();
            println!(
                "[Мониторинг] Ордеров: {} (Buy: {:.1}%), Объём: ${:.2}",
                snap.total_orders(),
                snap.buy_ratio(),
                snap.total_volume
            );
        }
    });

    // Торговые потоки
    for trader_id in 0..4 {
        let stats_clone = Arc::clone(&stats);

        let handle = thread::spawn(move || {
            for i in 0..500 {
                let volume = 100.0 + (i as f64 * 0.5);

                match i % 10 {
                    0..=5 => stats_clone.record_buy(volume),
                    6..=8 => stats_clone.record_sell(volume),
                    9 => {
                        if i % 20 == 9 {
                            stats_clone.record_error();
                        } else {
                            stats_clone.record_cancel();
                        }
                    }
                    _ => {}
                }
            }
            println!("Трейдер {} завершил работу", trader_id);
        });

        handles.push(handle);
    }

    // Ждём завершения
    for handle in handles {
        handle.join().unwrap();
    }
    monitor_handle.join().unwrap();

    let elapsed = start.elapsed();
    let final_stats = stats.snapshot();

    println!("\n===== Итоговая статистика =====");
    println!("Время работы: {:?}", elapsed);
    println!("Покупки: {}", final_stats.buy_orders);
    println!("Продажи: {}", final_stats.sell_orders);
    println!("Всего сделок: {}", final_stats.total_orders());
    println!("Общий объём: ${:.2}", final_stats.total_volume);
    println!("Отменено: {}", final_stats.cancelled_orders);
    println!("Ошибок: {}", final_stats.errors);
    println!("Buy/Sell ratio: {:.1}%", final_stats.buy_ratio());
}
```

## Высокопроизводительный счётчик тиков

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Счётчик рыночных тиков с поддержкой статистики
struct TickCounter {
    /// Количество обработанных тиков
    ticks: AtomicU64,
    /// Последняя цена (умноженная на 100 для точности)
    last_price_cents: AtomicU64,
    /// Максимальная цена за сессию
    max_price_cents: AtomicU64,
    /// Минимальная цена за сессию
    min_price_cents: AtomicU64,
}

impl TickCounter {
    fn new(initial_price: f64) -> Self {
        let price_cents = (initial_price * 100.0) as u64;
        TickCounter {
            ticks: AtomicU64::new(0),
            last_price_cents: AtomicU64::new(price_cents),
            max_price_cents: AtomicU64::new(price_cents),
            min_price_cents: AtomicU64::new(price_cents),
        }
    }

    /// Обрабатывает новый тик с ценой
    fn record_tick(&self, price: f64) {
        self.ticks.fetch_add(1, Ordering::Relaxed);

        let price_cents = (price * 100.0) as u64;
        self.last_price_cents.store(price_cents, Ordering::Relaxed);

        // Атомарно обновляем максимум
        let mut current_max = self.max_price_cents.load(Ordering::Relaxed);
        while price_cents > current_max {
            match self.max_price_cents.compare_exchange_weak(
                current_max,
                price_cents,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Атомарно обновляем минимум
        let mut current_min = self.min_price_cents.load(Ordering::Relaxed);
        while price_cents < current_min {
            match self.min_price_cents.compare_exchange_weak(
                current_min,
                price_cents,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
    }

    fn tick_count(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }

    fn last_price(&self) -> f64 {
        self.last_price_cents.load(Ordering::Relaxed) as f64 / 100.0
    }

    fn price_range(&self) -> (f64, f64) {
        let min = self.min_price_cents.load(Ordering::Relaxed) as f64 / 100.0;
        let max = self.max_price_cents.load(Ordering::Relaxed) as f64 / 100.0;
        (min, max)
    }
}

fn main() {
    let counter = Arc::new(TickCounter::new(42000.0));
    let start = Instant::now();

    let mut handles = vec![];

    // Симуляция нескольких источников данных
    for source_id in 0..4 {
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            let base_price = 42000.0 + (source_id as f64 * 10.0);

            for i in 0..10000 {
                // Симулируем колебания цены
                let variation = ((i as f64 * 0.1).sin() * 500.0) +
                               ((source_id as f64 + i as f64) * 0.01);
                let price = base_price + variation;

                counter_clone.record_tick(price);
            }
        });

        handles.push(handle);
    }

    // Мониторинг в реальном времени
    let counter_monitor = Arc::clone(&counter);
    let monitor = thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(10));
            let ticks = counter_monitor.tick_count();
            let price = counter_monitor.last_price();
            let (min, max) = counter_monitor.price_range();
            println!(
                "Тиков: {:6} | Цена: ${:.2} | Диапазон: ${:.2} - ${:.2}",
                ticks, price, min, max
            );
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    monitor.join().unwrap();

    let elapsed = start.elapsed();
    let total_ticks = counter.tick_count();
    let (min, max) = counter.price_range();

    println!("\n===== Результаты =====");
    println!("Обработано тиков: {}", total_ticks);
    println!("Время: {:?}", elapsed);
    println!("Скорость: {:.0} тиков/сек", total_ticks as f64 / elapsed.as_secs_f64());
    println!("Последняя цена: ${:.2}", counter.last_price());
    println!("Диапазон цен: ${:.2} - ${:.2}", min, max);
    println!("Волатильность: ${:.2}", max - min);
}
```

## Сравнение AtomicU64 и Mutex

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn benchmark_atomic(iterations: u64, threads: usize) -> std::time::Duration {
    let counter = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..iterations {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_mutex(iterations: u64, threads: usize) -> std::time::Duration {
    let counter = Arc::new(Mutex::new(0u64));
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let c = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..iterations {
                    let mut guard = c.lock().unwrap();
                    *guard += 1;
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

fn main() {
    let iterations = 100_000;
    let thread_counts = [1, 2, 4, 8];

    println!("Сравнение производительности: AtomicU64 vs Mutex");
    println!("Итераций на поток: {}\n", iterations);
    println!("{:>8} {:>12} {:>12} {:>10}", "Потоков", "Atomic", "Mutex", "Разница");
    println!("{}", "-".repeat(46));

    for &threads in &thread_counts {
        let atomic_time = benchmark_atomic(iterations, threads);
        let mutex_time = benchmark_mutex(iterations, threads);

        let speedup = mutex_time.as_nanos() as f64 / atomic_time.as_nanos() as f64;

        println!(
            "{:>8} {:>10.2}ms {:>10.2}ms {:>9.1}x",
            threads,
            atomic_time.as_secs_f64() * 1000.0,
            mutex_time.as_secs_f64() * 1000.0,
            speedup
        );
    }

    println!("\nВывод: AtomicU64 значительно быстрее Mutex для простых счётчиков,");
    println!("особенно при высокой конкуренции между потоками.");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `AtomicU64` | Атомарный 64-битный беззнаковый счётчик |
| `fetch_add` | Атомарное увеличение, возвращает старое значение |
| `compare_exchange` | Условная замена для сложных операций |
| `Ordering::Relaxed` | Минимальные гарантии, максимальная скорость |
| Lock-free | Атомарные операции не блокируют потоки |
| Производительность | AtomicU64 быстрее Mutex для простых операций |

## Домашнее задание

1. **Счётчик сделок по типам**: Создай структуру `TradeTypeCounter` с отдельными `AtomicU64` для:
   - Market orders (рыночные ордера)
   - Limit orders (лимитные ордера)
   - Stop orders (стоп-ордера)

   Реализуй методы для подсчёта каждого типа и получения общей статистики.

2. **Генератор ID с проверкой**: Модифицируй `OrderIdGenerator` так, чтобы он:
   - Использовал `compare_exchange` для предотвращения переполнения счётчика
   - Возвращал `Option<u64>` вместо `u64`
   - Логировал предупреждение при приближении к лимиту (например, 90% от `u32::MAX`)

3. **Скользящее среднее**: Реализуй структуру `AtomicMovingAverage` для расчёта среднего значения последних N значений:
   - Используй кольцевой буфер с атомарным индексом
   - Атомарно обновляй сумму при добавлении нового значения
   - Реализуй потокобезопасный метод `average()`

4. **Rate Limiter**: Создай структуру `TradeRateLimiter`, которая:
   - Ограничивает количество сделок в секунду
   - Использует атомарные счётчики для отслеживания
   - Возвращает `true`/`false` при попытке совершить сделку
   - Автоматически сбрасывает счётчик каждую секунду

## Навигация

[← Предыдущий день](../167-atomicbool-bot-stop-flag/ru.md) | [Следующий день →](../169-ordering-visibility-guarantees/ru.md)
