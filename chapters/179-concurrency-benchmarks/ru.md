# День 179: Бенчмарки многопоточности

## Аналогия из трейдинга

Представь, что ты разрабатываешь высокочастотную торговую систему (HFT). Тебе нужно выбрать между разными подходами к обработке ордеров: однопоточная очередь, многопоточный пул, lock-free структуры или акторная модель. Как понять, какой подход будет самым быстрым для твоей нагрузки?

Это похоже на тестирование разных торговых стратегий на исторических данных — ты прогоняешь каждую стратегию на одинаковых условиях и сравниваешь результаты. В программировании этот процесс называется **бенчмаркинг** — измерение производительности кода для выбора оптимального решения.

В реальном трейдинге бенчмарки помогают ответить на вопросы:
- Сколько ордеров в секунду может обработать система?
- Какая задержка при сопоставлении заявок?
- Как производительность изменяется с увеличением числа потоков?
- Какой overhead даёт синхронизация?

## Что такое бенчмарки многопоточности?

Бенчмаркинг многопоточности — это измерение производительности параллельного кода. Основные метрики:

| Метрика | Описание |
|---------|----------|
| Throughput | Количество операций в секунду |
| Latency | Время выполнения одной операции |
| Scalability | Как производительность растёт с числом потоков |
| Contention | Накладные расходы на синхронизацию |

## Инструменты для бенчмаркинга в Rust

### 1. Criterion — стандартный инструмент

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "trading_benchmarks"
harness = false
```

### 2. std::time для простых измерений

```rust
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // Код для измерения
    process_orders();

    let duration = start.elapsed();
    println!("Обработка заняла: {:?}", duration);
}

fn process_orders() {
    // Имитация обработки
    std::thread::sleep(std::time::Duration::from_millis(10));
}
```

## Бенчмарк: однопоточная vs многопоточная обработка ордеров

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn process_order(order: &Order) -> f64 {
    // Имитация обработки: проверка лимитов, расчёт комиссии
    let commission = order.price * order.quantity * 0.001;
    // Имитация задержки вычислений
    std::hint::black_box(commission.sqrt().sin().cos());
    commission
}

fn generate_orders(count: usize) -> Vec<Order> {
    (0..count)
        .map(|i| Order {
            id: i as u64,
            symbol: format!("BTC{}", i % 10),
            price: 42000.0 + (i as f64) * 0.1,
            quantity: 0.1 + (i as f64) * 0.001,
        })
        .collect()
}

// Однопоточная обработка
fn single_threaded_processing(orders: &[Order]) -> f64 {
    orders.iter().map(|o| process_order(o)).sum()
}

// Многопоточная обработка
fn multi_threaded_processing(orders: Vec<Order>, num_threads: usize) -> f64 {
    let chunk_size = (orders.len() + num_threads - 1) / num_threads;
    let orders = Arc::new(orders);
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    for i in 0..num_threads {
        let orders = Arc::clone(&orders);
        let results = Arc::clone(&results);
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(orders.len());

        let handle = thread::spawn(move || {
            let partial_sum: f64 = orders[start..end]
                .iter()
                .map(|o| process_order(o))
                .sum();
            results.lock().unwrap().push(partial_sum);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    results.lock().unwrap().iter().sum()
}

fn benchmark_order_processing() {
    let order_counts = [1_000, 10_000, 100_000];
    let thread_counts = [1, 2, 4, 8];

    println!("=== Бенчмарк обработки ордеров ===\n");
    println!("{:>10} | {:>8} | {:>12} | {:>10}",
             "Ордеров", "Потоков", "Время (мс)", "Орд/сек");
    println!("{}", "-".repeat(50));

    for &count in &order_counts {
        let orders = generate_orders(count);

        // Однопоточный тест
        let start = Instant::now();
        let _result = single_threaded_processing(&orders);
        let single_duration = start.elapsed();
        let single_ops_per_sec = count as f64 / single_duration.as_secs_f64();

        println!("{:>10} | {:>8} | {:>12.2} | {:>10.0}",
                 count, 1, single_duration.as_millis(), single_ops_per_sec);

        // Многопоточные тесты
        for &threads in &thread_counts[1..] {
            let orders_clone = orders.clone();
            let start = Instant::now();
            let _result = multi_threaded_processing(orders_clone, threads);
            let duration = start.elapsed();
            let ops_per_sec = count as f64 / duration.as_secs_f64();
            let speedup = single_duration.as_secs_f64() / duration.as_secs_f64();

            println!("{:>10} | {:>8} | {:>12.2} | {:>10.0} (x{:.2})",
                     count, threads, duration.as_millis(), ops_per_sec, speedup);
        }
        println!();
    }
}

fn main() {
    benchmark_order_processing();
}
```

## Бенчмарк: сравнение примитивов синхронизации

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

const ITERATIONS: u64 = 1_000_000;
const NUM_THREADS: usize = 4;

// Тест с Mutex
fn benchmark_mutex() -> std::time::Duration {
    let counter = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..NUM_THREADS {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..ITERATIONS / NUM_THREADS as u64 {
                let mut guard = counter.lock().unwrap();
                *guard += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Тест с RwLock (только запись)
fn benchmark_rwlock_write() -> std::time::Duration {
    let counter = Arc::new(RwLock::new(0u64));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..NUM_THREADS {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..ITERATIONS / NUM_THREADS as u64 {
                let mut guard = counter.write().unwrap();
                *guard += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Тест с RwLock (чтение/запись 90/10)
fn benchmark_rwlock_mixed() -> std::time::Duration {
    let counter = Arc::new(RwLock::new(0u64));
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..NUM_THREADS {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for j in 0..ITERATIONS / NUM_THREADS as u64 {
                if (i + j as usize) % 10 == 0 {
                    // 10% записей
                    let mut guard = counter.write().unwrap();
                    *guard += 1;
                } else {
                    // 90% чтений
                    let guard = counter.read().unwrap();
                    std::hint::black_box(*guard);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

// Тест с Atomic
fn benchmark_atomic() -> std::time::Duration {
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..NUM_THREADS {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..ITERATIONS / NUM_THREADS as u64 {
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn main() {
    println!("=== Бенчмарк примитивов синхронизации ===");
    println!("Итераций: {}, Потоков: {}\n", ITERATIONS, NUM_THREADS);

    // Прогрев
    let _ = benchmark_mutex();
    let _ = benchmark_atomic();

    // Реальные измерения
    let mutex_time = benchmark_mutex();
    let rwlock_write_time = benchmark_rwlock_write();
    let rwlock_mixed_time = benchmark_rwlock_mixed();
    let atomic_time = benchmark_atomic();

    println!("{:<20} | {:>12} | {:>15}",
             "Примитив", "Время (мс)", "Опер/сек");
    println!("{}", "-".repeat(55));

    let ops = ITERATIONS as f64;
    println!("{:<20} | {:>12.2} | {:>15.0}",
             "Mutex", mutex_time.as_millis(), ops / mutex_time.as_secs_f64());
    println!("{:<20} | {:>12.2} | {:>15.0}",
             "RwLock (запись)", rwlock_write_time.as_millis(), ops / rwlock_write_time.as_secs_f64());
    println!("{:<20} | {:>12.2} | {:>15.0}",
             "RwLock (90/10)", rwlock_mixed_time.as_millis(), ops / rwlock_mixed_time.as_secs_f64());
    println!("{:<20} | {:>12.2} | {:>15.0}",
             "Atomic", atomic_time.as_millis(), ops / atomic_time.as_secs_f64());

    println!("\nВывод: Atomic в {:.1}x быстрее Mutex",
             mutex_time.as_secs_f64() / atomic_time.as_secs_f64());
}
```

## Бенчмарк: масштабируемость с числом потоков

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    volume: f64,
}

fn analyze_market_data(data: &[MarketData]) -> f64 {
    data.iter()
        .map(|d| {
            let spread = (d.ask - d.bid) / d.bid * 100.0;
            let weighted_price = (d.bid + d.ask) / 2.0 * d.volume;
            std::hint::black_box(spread + weighted_price.sqrt())
        })
        .sum()
}

fn generate_market_data(count: usize) -> Vec<MarketData> {
    (0..count)
        .map(|i| MarketData {
            symbol: format!("PAIR{}", i % 100),
            bid: 100.0 + (i as f64) * 0.001,
            ask: 100.1 + (i as f64) * 0.001,
            volume: 1000.0 + (i as f64),
        })
        .collect()
}

fn benchmark_scalability(data: &[MarketData], max_threads: usize) {
    println!("=== Тест масштабируемости ===");
    println!("Записей данных: {}\n", data.len());
    println!("{:>8} | {:>12} | {:>10} | {:>10}",
             "Потоков", "Время (мс)", "Ускорение", "Эффект.");
    println!("{}", "-".repeat(50));

    let mut baseline_time = None;

    for num_threads in 1..=max_threads {
        let chunk_size = (data.len() + num_threads - 1) / num_threads;
        let counter = Arc::new(AtomicU64::new(0));
        let mut handles = vec![];

        let start = Instant::now();

        for i in 0..num_threads {
            let start_idx = i * chunk_size;
            let end_idx = ((i + 1) * chunk_size).min(data.len());

            if start_idx >= data.len() {
                break;
            }

            let chunk: Vec<MarketData> = data[start_idx..end_idx].to_vec();
            let counter = Arc::clone(&counter);

            handles.push(thread::spawn(move || {
                let result = analyze_market_data(&chunk);
                counter.fetch_add(result as u64, Ordering::Relaxed);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();

        if baseline_time.is_none() {
            baseline_time = Some(duration);
        }

        let speedup = baseline_time.unwrap().as_secs_f64() / duration.as_secs_f64();
        let efficiency = speedup / num_threads as f64 * 100.0;

        println!("{:>8} | {:>12.2} | {:>10.2}x | {:>9.1}%",
                 num_threads, duration.as_millis() as f64, speedup, efficiency);
    }
}

fn main() {
    let data = generate_market_data(1_000_000);
    benchmark_scalability(&data, 8);
}
```

## Бенчмарк: конкуренция за общий ресурс (стакан заявок)

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::collections::BTreeMap;
use std::thread;
use std::time::Instant;

type Price = u64; // Цена в копейках для избежания float сравнений
type Quantity = u64;

#[derive(Debug, Clone)]
struct OrderBook {
    bids: BTreeMap<Price, Quantity>, // Заявки на покупку
    asks: BTreeMap<Price, Quantity>, // Заявки на продажу
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_bid(&mut self, price: Price, qty: Quantity) {
        *self.bids.entry(price).or_insert(0) += qty;
    }

    fn add_ask(&mut self, price: Price, qty: Quantity) {
        *self.asks.entry(price).or_insert(0) += qty;
    }

    fn get_spread(&self) -> Option<u64> {
        let best_bid = self.bids.keys().last()?;
        let best_ask = self.asks.keys().next()?;
        Some(best_ask.saturating_sub(*best_bid))
    }
}

fn benchmark_orderbook_mutex(iterations: u64, threads: usize) -> std::time::Duration {
    let book = Arc::new(Mutex::new(OrderBook::new()));
    let mut handles = vec![];

    let start = Instant::now();

    for t in 0..threads {
        let book = Arc::clone(&book);
        handles.push(thread::spawn(move || {
            for i in 0..iterations / threads as u64 {
                let mut guard = book.lock().unwrap();
                if (t + i as usize) % 2 == 0 {
                    guard.add_bid(10000 + (i % 100), 10);
                } else {
                    guard.add_ask(10100 + (i % 100), 10);
                }
                // Иногда читаем спред
                if i % 10 == 0 {
                    let _ = guard.get_spread();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn benchmark_orderbook_rwlock(iterations: u64, threads: usize) -> std::time::Duration {
    let book = Arc::new(RwLock::new(OrderBook::new()));
    let mut handles = vec![];

    let start = Instant::now();

    for t in 0..threads {
        let book = Arc::clone(&book);
        handles.push(thread::spawn(move || {
            for i in 0..iterations / threads as u64 {
                // 80% чтений, 20% записей
                if (t + i as usize) % 5 == 0 {
                    let mut guard = book.write().unwrap();
                    if (t + i as usize) % 2 == 0 {
                        guard.add_bid(10000 + (i % 100), 10);
                    } else {
                        guard.add_ask(10100 + (i % 100), 10);
                    }
                } else {
                    let guard = book.read().unwrap();
                    let _ = guard.get_spread();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    start.elapsed()
}

fn main() {
    let iterations = 100_000u64;
    let thread_counts = [1, 2, 4, 8];

    println!("=== Бенчмарк стакана заявок ===");
    println!("Операций: {}\n", iterations);
    println!("{:>8} | {:>15} | {:>15} | {:>10}",
             "Потоков", "Mutex (мс)", "RwLock (мс)", "Разница");
    println!("{}", "-".repeat(60));

    for &threads in &thread_counts {
        // Прогрев
        let _ = benchmark_orderbook_mutex(1000, threads);
        let _ = benchmark_orderbook_rwlock(1000, threads);

        // Измерение
        let mutex_time = benchmark_orderbook_mutex(iterations, threads);
        let rwlock_time = benchmark_orderbook_rwlock(iterations, threads);

        let diff = if rwlock_time < mutex_time {
            format!("-{:.1}%", (1.0 - rwlock_time.as_secs_f64() / mutex_time.as_secs_f64()) * 100.0)
        } else {
            format!("+{:.1}%", (rwlock_time.as_secs_f64() / mutex_time.as_secs_f64() - 1.0) * 100.0)
        };

        println!("{:>8} | {:>15.2} | {:>15.2} | {:>10}",
                 threads, mutex_time.as_millis(), rwlock_time.as_millis(), diff);
    }

    println!("\nВывод: RwLock эффективнее при преобладании чтений (80%+)");
}
```

## Бенчмарк с использованием Criterion

```rust
// benches/trading_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

fn process_trade(price: f64, quantity: f64) -> f64 {
    let commission = price * quantity * 0.001;
    black_box(commission.sqrt())
}

fn single_thread_trades(n: usize) {
    let mut total = 0.0;
    for i in 0..n {
        total += process_trade(100.0 + i as f64, 10.0);
    }
    black_box(total);
}

fn multi_thread_trades(n: usize, threads: usize) {
    let total = Arc::new(AtomicU64::new(0));
    let chunk = n / threads;

    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let total = Arc::clone(&total);
            thread::spawn(move || {
                let mut local = 0.0f64;
                for i in 0..chunk {
                    local += process_trade(100.0 + (t * chunk + i) as f64, 10.0);
                }
                total.fetch_add(local.to_bits(), Ordering::Relaxed);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

fn benchmark_trading(c: &mut Criterion) {
    let mut group = c.benchmark_group("trade_processing");

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_thread", size),
            size,
            |b, &n| b.iter(|| single_thread_trades(n)),
        );

        for threads in [2, 4, 8].iter() {
            group.bench_with_input(
                BenchmarkId::new(format!("multi_thread_{}", threads), size),
                size,
                |b, &n| b.iter(|| multi_thread_trades(n, *threads)),
            );
        }
    }

    group.finish();
}

criterion_group!(benches, benchmark_trading);
criterion_main!(benches);
```

## Лучшие практики бенчмаркинга

### 1. Избегайте оптимизаций компилятора

```rust
use std::hint::black_box;

fn main() {
    let result = expensive_calculation();

    // Без black_box компилятор может удалить вычисления
    black_box(result);
}

fn expensive_calculation() -> f64 {
    let mut sum = 0.0;
    for i in 0..1_000_000 {
        sum += (i as f64).sqrt();
    }
    sum
}
```

### 2. Прогрев перед измерениями

```rust
use std::time::Instant;

fn benchmark_with_warmup<F, R>(name: &str, warmup_runs: u32, measured_runs: u32, f: F)
where
    F: Fn() -> R,
{
    // Прогрев
    for _ in 0..warmup_runs {
        std::hint::black_box(f());
    }

    // Измерение
    let mut durations = Vec::with_capacity(measured_runs as usize);
    for _ in 0..measured_runs {
        let start = Instant::now();
        std::hint::black_box(f());
        durations.push(start.elapsed());
    }

    // Статистика
    let total: std::time::Duration = durations.iter().sum();
    let avg = total / measured_runs;
    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();

    println!("{}: avg={:?}, min={:?}, max={:?}", name, avg, min, max);
}

fn main() {
    benchmark_with_warmup("calculation", 100, 1000, || {
        (0..10000).map(|x| (x as f64).sqrt()).sum::<f64>()
    });
}
```

### 3. Учитывайте NUMA и кэш

```rust
use std::sync::Arc;
use std::thread;

// Плохо: false sharing из-за близких атомиков
struct BadCounters {
    counter1: std::sync::atomic::AtomicU64,
    counter2: std::sync::atomic::AtomicU64,
}

// Хорошо: padding предотвращает false sharing
#[repr(align(128))] // Размер кэш-линии
struct AlignedCounter(std::sync::atomic::AtomicU64);

struct GoodCounters {
    counter1: AlignedCounter,
    counter2: AlignedCounter,
}

fn main() {
    use std::sync::atomic::Ordering;

    // Демонстрация false sharing
    let bad = Arc::new(BadCounters {
        counter1: std::sync::atomic::AtomicU64::new(0),
        counter2: std::sync::atomic::AtomicU64::new(0),
    });

    let b1 = Arc::clone(&bad);
    let b2 = Arc::clone(&bad);

    let start = std::time::Instant::now();

    let h1 = thread::spawn(move || {
        for _ in 0..10_000_000 {
            b1.counter1.fetch_add(1, Ordering::Relaxed);
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..10_000_000 {
            b2.counter2.fetch_add(1, Ordering::Relaxed);
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    println!("Без padding: {:?}", start.elapsed());
}
```

## Практический пример: Сравнение архитектур торговой системы

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct TradeEvent {
    symbol: String,
    price: f64,
    quantity: f64,
    side: bool, // true = buy, false = sell
}

// Архитектура 1: Общий Mutex
fn architecture_shared_mutex(events: Vec<TradeEvent>) -> std::time::Duration {
    let queue = Arc::new(Mutex::new(VecDeque::from(events)));
    let processed = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..4 {
        let queue = Arc::clone(&queue);
        let processed = Arc::clone(&processed);

        handles.push(thread::spawn(move || {
            loop {
                let event = {
                    let mut q = queue.lock().unwrap();
                    q.pop_front()
                };

                match event {
                    Some(e) => {
                        // Обработка события
                        let _ = std::hint::black_box(e.price * e.quantity);
                        *processed.lock().unwrap() += 1;
                    }
                    None => break,
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

// Архитектура 2: Каналы (MPSC)
fn architecture_channels(events: Vec<TradeEvent>) -> std::time::Duration {
    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));
    let processed = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];

    let start = Instant::now();

    // Отправка событий
    let sender = thread::spawn(move || {
        for event in events {
            tx.send(event).unwrap();
        }
    });

    // Обработчики
    for _ in 0..4 {
        let rx = Arc::clone(&rx);
        let processed = Arc::clone(&processed);

        handles.push(thread::spawn(move || {
            loop {
                let event = {
                    let rx = rx.lock().unwrap();
                    rx.try_recv()
                };

                match event {
                    Ok(e) => {
                        let _ = std::hint::black_box(e.price * e.quantity);
                        *processed.lock().unwrap() += 1;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        thread::yield_now();
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
            }
        }));
    }

    sender.join().unwrap();
    for h in handles {
        h.join().unwrap();
    }

    start.elapsed()
}

// Архитектура 3: Предварительное разделение данных
fn architecture_partitioned(events: Vec<TradeEvent>) -> std::time::Duration {
    let num_threads = 4;
    let chunk_size = (events.len() + num_threads - 1) / num_threads;
    let chunks: Vec<Vec<TradeEvent>> = events
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect();

    let start = Instant::now();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            thread::spawn(move || {
                let mut count = 0u64;
                for e in chunk {
                    let _ = std::hint::black_box(e.price * e.quantity);
                    count += 1;
                }
                count
            })
        })
        .collect();

    let _total: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();

    start.elapsed()
}

fn main() {
    let event_counts = [10_000, 100_000, 1_000_000];

    println!("=== Сравнение архитектур торговой системы ===\n");
    println!("{:>10} | {:>15} | {:>15} | {:>15}",
             "События", "Shared Mutex", "Channels", "Partitioned");
    println!("{}", "-".repeat(65));

    for &count in &event_counts {
        let events: Vec<TradeEvent> = (0..count)
            .map(|i| TradeEvent {
                symbol: format!("BTC{}", i % 10),
                price: 42000.0 + (i as f64) * 0.01,
                quantity: 0.1,
                side: i % 2 == 0,
            })
            .collect();

        // Прогрев
        let _ = architecture_shared_mutex(events[..1000.min(count)].to_vec());
        let _ = architecture_channels(events[..1000.min(count)].to_vec());
        let _ = architecture_partitioned(events[..1000.min(count)].to_vec());

        // Измерения
        let mutex_time = architecture_shared_mutex(events.clone());
        let channel_time = architecture_channels(events.clone());
        let partition_time = architecture_partitioned(events);

        println!("{:>10} | {:>15.2} | {:>15.2} | {:>15.2}",
                 count,
                 mutex_time.as_millis(),
                 channel_time.as_millis(),
                 partition_time.as_millis());
    }

    println!("\nВывод: Предварительное разделение данных — самый быстрый подход");
    println!("       для независимых операций обработки.");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Throughput | Количество операций в секунду |
| Latency | Время отклика одной операции |
| Scalability | Рост производительности с числом потоков |
| Criterion | Библиотека для точных бенчмарков |
| black_box | Защита от оптимизаций компилятора |
| Warmup | Прогрев перед измерениями |
| False sharing | Конфликт кэш-линий между потоками |

## Домашнее задание

1. **Бенчмарк структур данных**: Сравни производительность `Vec`, `VecDeque` и `LinkedList` для очереди ордеров при добавлении и удалении элементов из разных концов. Построй график зависимости от размера очереди.

2. **Профилирование latency**: Напиши бенчмарк, который измеряет не только среднее время, но и перцентили (p50, p95, p99) для обработки ордеров. Это критически важно для HFT-систем.

3. **Сравнение каналов**: Сравни производительность `std::sync::mpsc`, `crossbeam-channel` и `flume` для передачи торговых событий между потоками. Учитывай разные сценарии: один отправитель, много получателей (SPMC) и наоборот.

4. **Оптимизация стакана заявок**: Реализуй стакан заявок с использованием:
   - `BTreeMap` с `Mutex`
   - `BTreeMap` с `RwLock`
   - Lock-free структуры (можно использовать `crossbeam-skiplist`)

   Измерь производительность при разном соотношении чтений/записей (50/50, 90/10, 99/1).

## Навигация

[← Предыдущий день](../178-crossbeam-channels/ru.md) | [Следующий день →](../180-thread-pools-rayon/ru.md)
