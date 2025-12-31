# День 332: Tail latency: p99 оптимизация

## Аналогия из трейдинга

Представь, что ты управляешь высокочастотной торговой фирмой. Среднее время исполнения ордера — 1мс. Впечатляет! Но есть проблема: каждый сотый ордер (p99) исполняется за 50мс. Во время этих "хвостовых" событий рынок уже сдвинулся, и ты получаешь невыгодное исполнение или упущенные возможности.

**Tail latency** (хвостовая задержка) — это как самый медленный трейдер в твоей команде. Даже если 99 трейдеров молниеносно быстры, тот один медленный может испортить всю дневную прибыль, когда ему попадётся критический ордер.

| Метрика | Описание | Влияние на трейдинг |
|---------|----------|---------------------|
| **p50 (медиана)** | 50% запросов быстрее | Средний опыт |
| **p90** | 90% запросов быстрее | Большинство ордеров в порядке |
| **p95** | 95% запросов быстрее | Иногда замедления |
| **p99** | 99% запросов быстрее | Критично для HFT |
| **p99.9** | 99.9% запросов быстрее | Ультра-низколатентные системы |

В трейдинге p99 важен потому что:
- Один медленный ордер может вызвать значительное проскальзывание
- Арбитражные возможности исчезают за миллисекунды
- Решения по риск-менеджменту должны быть мгновенными
- Маркет-мейкеры требуют стабильного времени отклика

## Понимание хвостовой задержки

### Что вызывает tail latency?

```rust
use std::time::{Duration, Instant};
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// Исполнение ордера с различными источниками задержки
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: Instant,
}

/// Измерение задержки с отслеживанием перцентилей
struct LatencyTracker {
    latencies: Vec<Duration>,
}

impl LatencyTracker {
    fn new() -> Self {
        LatencyTracker {
            latencies: Vec::with_capacity(10000),
        }
    }

    fn record(&mut self, latency: Duration) {
        self.latencies.push(latency);
    }

    fn percentile(&mut self, p: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }

        self.latencies.sort();
        let index = ((self.latencies.len() as f64 * p / 100.0) as usize)
            .min(self.latencies.len() - 1);
        self.latencies[index]
    }

    fn report(&mut self) {
        println!("Отчёт о задержках:");
        println!("  p50:   {:?}", self.percentile(50.0));
        println!("  p90:   {:?}", self.percentile(90.0));
        println!("  p95:   {:?}", self.percentile(95.0));
        println!("  p99:   {:?}", self.percentile(99.0));
        println!("  p99.9: {:?}", self.percentile(99.9));
        println!("  max:   {:?}", self.latencies.iter().max().unwrap_or(&Duration::ZERO));
    }
}

/// Симуляция различных причин хвостовой задержки
fn process_order_with_latency_sources(order: &Order, iteration: u64) -> Duration {
    let start = Instant::now();

    // Симуляция обычной обработки
    std::hint::black_box(order.price * order.quantity);

    // Причина 1: Периодические паузы типа GC (выделение памяти)
    if iteration % 100 == 0 {
        let _allocations: Vec<Vec<u8>> = (0..1000)
            .map(|_| vec![0u8; 1024])
            .collect();
    }

    // Причина 2: Симуляция конкуренции за блокировки
    if iteration % 50 == 0 {
        std::thread::sleep(Duration::from_micros(100));
    }

    // Причина 3: Симуляция промаха кэша
    if iteration % 200 == 0 {
        let data: Vec<u64> = (0..10000).collect();
        let sum: u64 = data.iter().step_by(64).sum();
        std::hint::black_box(sum);
    }

    start.elapsed()
}

fn main() {
    println!("=== Понимание источников хвостовой задержки ===\n");

    let mut tracker = LatencyTracker::new();

    for i in 0..10000 {
        let order = Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            quantity: 0.1,
            timestamp: Instant::now(),
        };

        let latency = process_order_with_latency_sources(&order, i);
        tracker.record(latency);
    }

    tracker.report();

    println!("\nКлючевой вывод: p99 может быть в 10-100 раз выше p50!");
}
```

## Стратегия 1: Предварительное выделение и пулинг объектов

Один из главных источников хвостовой задержки — выделение памяти. Предварительное выделение буферов устраняет эту проблему:

```rust
use std::time::{Duration, Instant};

/// Предварительно выделенный буфер для обработки ордеров
struct OrderBuffer {
    data: Vec<u8>,
    position: usize,
}

impl OrderBuffer {
    fn with_capacity(size: usize) -> Self {
        OrderBuffer {
            data: vec![0u8; size],
            position: 0,
        }
    }

    fn reset(&mut self) {
        self.position = 0;
    }

    fn write(&mut self, bytes: &[u8]) -> bool {
        if self.position + bytes.len() > self.data.len() {
            return false;
        }
        self.data[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        true
    }
}

/// Пул объектов для переиспользования структур ордеров
struct OrderPool {
    available: Vec<Box<OrderData>>,
    in_use: usize,
}

#[derive(Default)]
struct OrderData {
    id: u64,
    symbol: [u8; 16],
    symbol_len: usize,
    price: f64,
    quantity: f64,
    side: u8, // 0 = покупка, 1 = продажа
}

impl OrderPool {
    fn new(initial_size: usize) -> Self {
        let available = (0..initial_size)
            .map(|_| Box::new(OrderData::default()))
            .collect();

        OrderPool {
            available,
            in_use: 0,
        }
    }

    fn acquire(&mut self) -> Option<Box<OrderData>> {
        self.available.pop().map(|order| {
            self.in_use += 1;
            order
        })
    }

    fn release(&mut self, mut order: Box<OrderData>) {
        // Сбрасываем данные ордера
        order.id = 0;
        order.symbol_len = 0;
        order.price = 0.0;
        order.quantity = 0.0;
        order.side = 0;

        self.available.push(order);
        self.in_use -= 1;
    }

    fn stats(&self) -> (usize, usize) {
        (self.available.len(), self.in_use)
    }
}

fn benchmark_allocation_strategies() {
    println!("=== Предварительное выделение vs Динамическое ===\n");

    const ITERATIONS: usize = 100_000;

    // Динамическое выделение (вызывает хвостовую задержку)
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        let buffer = vec![0u8; 1024];
        std::hint::black_box(&buffer);
        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("Динамическое выделение:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);

    // Предварительно выделенный буфер (стабильная задержка)
    let mut buffer = OrderBuffer::with_capacity(1024);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        buffer.reset();
        buffer.write(&[0u8; 100]);
        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("\nПредварительно выделенный буфер:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);

    // Пулинг объектов
    let mut pool = OrderPool::new(1000);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        if let Some(mut order) = pool.acquire() {
            order.id = i as u64;
            order.price = 50000.0;
            std::hint::black_box(&order);
            pool.release(order);
        }

        let latency = iter_start.elapsed();
        max_latency = max_latency.max(latency);
    }

    println!("\nПулинг объектов:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);
}

fn main() {
    benchmark_allocation_strategies();
}
```

## Стратегия 2: Lock-free структуры данных

Блокировки — главный источник хвостовой задержки из-за конкуренции. Lock-free альтернативы обеспечивают более стабильную производительность:

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Lock-free счётчик ордеров с использованием атомиков
struct AtomicOrderCounter {
    buy_count: AtomicU64,
    sell_count: AtomicU64,
    total_volume: AtomicU64, // Хранится как fixed-point (умножено на 1000)
}

impl AtomicOrderCounter {
    fn new() -> Self {
        AtomicOrderCounter {
            buy_count: AtomicU64::new(0),
            sell_count: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
        }
    }

    fn record_buy(&self, volume: f64) {
        self.buy_count.fetch_add(1, Ordering::Relaxed);
        let volume_fixed = (volume * 1000.0) as u64;
        self.total_volume.fetch_add(volume_fixed, Ordering::Relaxed);
    }

    fn record_sell(&self, volume: f64) {
        self.sell_count.fetch_add(1, Ordering::Relaxed);
        let volume_fixed = (volume * 1000.0) as u64;
        self.total_volume.fetch_add(volume_fixed, Ordering::Relaxed);
    }

    fn stats(&self) -> (u64, u64, f64) {
        let buys = self.buy_count.load(Ordering::Relaxed);
        let sells = self.sell_count.load(Ordering::Relaxed);
        let volume = self.total_volume.load(Ordering::Relaxed) as f64 / 1000.0;
        (buys, sells, volume)
    }
}

/// Lock-free SPSC (Single Producer Single Consumer) кольцевой буфер
/// Идеален для очередей ордеров между потоками
struct SpscQueue<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: AtomicUsize, // Потребитель читает отсюда
    tail: AtomicUsize, // Производитель пишет сюда
}

impl<T> SpscQueue<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        SpscQueue {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    fn push(&mut self, item: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;

        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Очередь полна
        }

        self.buffer[tail] = Some(item);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Очередь пуста
        }

        let item = self.buffer[head].take();
        self.head.store((head + 1) % self.capacity, Ordering::Release);
        item
    }
}

/// Бенчмарк lock-free vs mutex-based счётчика
fn benchmark_lock_free() {
    println!("=== Lock-Free vs Mutex производительность ===\n");

    use std::sync::Mutex;

    const ITERATIONS: u64 = 1_000_000;
    const THREADS: usize = 4;

    // Счётчик на мьютексе
    let mutex_counter = Arc::new(Mutex::new((0u64, 0u64, 0.0f64)));
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let counter = Arc::clone(&mutex_counter);
            thread::spawn(move || {
                let mut max_latency = Duration::ZERO;
                for i in 0..ITERATIONS / THREADS as u64 {
                    let iter_start = Instant::now();
                    let mut guard = counter.lock().unwrap();
                    if i % 2 == 0 {
                        guard.0 += 1;
                    } else {
                        guard.1 += 1;
                    }
                    guard.2 += 0.1;
                    drop(guard);
                    max_latency = max_latency.max(iter_start.elapsed());
                }
                max_latency
            })
        })
        .collect();

    let max_mutex_latency = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .max()
        .unwrap();

    println!("Счётчик на мьютексе:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_mutex_latency);

    // Lock-free счётчик
    let atomic_counter = Arc::new(AtomicOrderCounter::new());
    let start = Instant::now();

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let counter = Arc::clone(&atomic_counter);
            thread::spawn(move || {
                let mut max_latency = Duration::ZERO;
                for i in 0..ITERATIONS / THREADS as u64 {
                    let iter_start = Instant::now();
                    if i % 2 == 0 {
                        counter.record_buy(0.1);
                    } else {
                        counter.record_sell(0.1);
                    }
                    max_latency = max_latency.max(iter_start.elapsed());
                }
                max_latency
            })
        })
        .collect();

    let max_atomic_latency = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .max()
        .unwrap();

    println!("\nLock-free счётчик:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_atomic_latency);

    let (buys, sells, volume) = atomic_counter.stats();
    println!("  Итоговая статистика: {} покупок, {} продаж, {:.2} объём", buys, sells, volume);
}

fn main() {
    benchmark_lock_free();
}
```

## Стратегия 3: Избегание системных вызовов на горячем пути

Системные вызовы могут вызывать непредсказуемые всплески задержки. Минимизируй их на критических путях:

```rust
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

/// Кэшированная метка времени для избежания системных вызовов
struct CachedTimestamp {
    cached_nanos: AtomicU64,
    last_update: AtomicU64,
}

impl CachedTimestamp {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        CachedTimestamp {
            cached_nanos: AtomicU64::new(now),
            last_update: AtomicU64::new(now),
        }
    }

    /// Быстрый путь: возвращает кэшированную метку времени
    fn now_fast(&self) -> u64 {
        self.cached_nanos.load(Ordering::Relaxed)
    }

    /// Обновление кэша (вызывай периодически из фонового потока)
    fn update(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        self.cached_nanos.store(now, Ordering::Relaxed);
        self.last_update.store(now, Ordering::Relaxed);
    }
}

/// Лог сделок с батчированием записей для избежания I/O системных вызовов
struct BatchedTradeLog {
    buffer: Vec<TradeRecord>,
    capacity: usize,
}

#[derive(Clone)]
struct TradeRecord {
    timestamp: u64,
    symbol_id: u32,
    price: f64,
    quantity: f64,
    side: u8,
}

impl BatchedTradeLog {
    fn new(batch_size: usize) -> Self {
        BatchedTradeLog {
            buffer: Vec::with_capacity(batch_size),
            capacity: batch_size,
        }
    }

    /// Быстрый путь: добавляем в буфер, без системных вызовов
    fn log(&mut self, record: TradeRecord) -> bool {
        if self.buffer.len() >= self.capacity {
            return false; // Буфер полон, нужен сброс
        }
        self.buffer.push(record);
        true
    }

    /// Медленный путь: сброс буфера (вызывай из фонового потока)
    fn flush(&mut self) -> Vec<TradeRecord> {
        std::mem::take(&mut self.buffer)
    }

    fn pending_count(&self) -> usize {
        self.buffer.len()
    }
}

fn benchmark_syscall_avoidance() {
    println!("=== Избегание системных вызовов ===\n");

    const ITERATIONS: usize = 1_000_000;

    // Прямые системные вызовы (медленно, непредсказуемо)
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut timestamps = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let iter_start = Instant::now();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        timestamps.push(ts);
        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("Прямые вызовы SystemTime:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);
    std::hint::black_box(timestamps);

    // Кэшированная метка времени (быстро, стабильно)
    let cached_ts = CachedTimestamp::new();
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut timestamps = Vec::with_capacity(ITERATIONS);

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        // Обновляем кэш изредка (симуляция фонового потока)
        if i % 10000 == 0 {
            cached_ts.update();
        }

        let ts = cached_ts.now_fast();
        timestamps.push(ts);
        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("\nКэшированная метка времени:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);
    std::hint::black_box(timestamps);

    // Батчированное логирование
    let mut log = BatchedTradeLog::new(1000);
    let start = Instant::now();
    let mut max_latency = Duration::ZERO;
    let mut flushes = 0;

    for i in 0..ITERATIONS {
        let iter_start = Instant::now();

        let record = TradeRecord {
            timestamp: i as u64,
            symbol_id: 1,
            price: 50000.0,
            quantity: 0.1,
            side: (i % 2) as u8,
        };

        if !log.log(record.clone()) {
            // Сброс произойдёт здесь (в реальном коде — отправка в фоновый поток)
            let _records = log.flush();
            flushes += 1;
            log.log(record);
        }

        max_latency = max_latency.max(iter_start.elapsed());
    }

    println!("\nБатчированное логирование:");
    println!("  Общее время: {:?}", start.elapsed());
    println!("  Макс. задержка: {:?}", max_latency);
    println!("  Сбросов: {}", flushes);
}

fn main() {
    benchmark_syscall_avoidance();
}
```

## Стратегия 4: Привязка к CPU и NUMA-осведомлённость

Для ультра-низкой задержки привязывай потоки к конкретным ядрам CPU:

```rust
use std::thread;
use std::time::{Duration, Instant};

/// Торговый движок с учётом привязки к CPU
struct TradingEngine {
    core_id: usize,
    name: String,
}

impl TradingEngine {
    fn new(name: &str, core_id: usize) -> Self {
        TradingEngine {
            core_id,
            name: name.to_string(),
        }
    }

    /// Симуляция установки привязки к CPU
    /// В реальном коде используй core_affinity crate или libc напрямую
    fn describe_affinity(&self) {
        println!(
            "Движок '{}' должен быть привязан к ядру {}",
            self.name, self.core_id
        );

        // Пример с использованием core_affinity crate (псевдокод):
        // let core_ids = core_affinity::get_core_ids().unwrap();
        // if let Some(core_id) = core_ids.get(self.core_id) {
        //     core_affinity::set_for_current(*core_id);
        // }
    }

    fn process_orders(&self, count: usize) -> Duration {
        let start = Instant::now();

        for i in 0..count {
            // Симуляция обработки ордера
            let price = 50000.0 + (i as f64 * 0.01);
            let quantity = 0.1;
            let _value = price * quantity;
            std::hint::black_box(_value);
        }

        start.elapsed()
    }
}

/// Стратегия NUMA-осведомлённого выделения памяти
struct NumaAwareBuffer {
    // В реальном коде используй numa crate для NUMA-локального выделения
    data: Vec<u8>,
    numa_node: usize,
}

impl NumaAwareBuffer {
    fn new(size: usize, numa_node: usize) -> Self {
        println!("Выделение {} байт на NUMA-узле {}", size, numa_node);

        // В реальном коде:
        // let ptr = numa::alloc_onnode(size, numa_node);
        // let data = unsafe { Vec::from_raw_parts(ptr, size, size) };

        NumaAwareBuffer {
            data: vec![0u8; size],
            numa_node,
        }
    }

    fn write(&mut self, offset: usize, value: u8) {
        if offset < self.data.len() {
            self.data[offset] = value;
        }
    }

    fn read(&self, offset: usize) -> u8 {
        self.data.get(offset).copied().unwrap_or(0)
    }
}

/// Демонстрация кэш-дружественной разметки данных
#[repr(C)]
struct CacheFriendlyOrder {
    // Горячие данные (часто используются) — помещаются в одну кэш-линию (64 байта)
    price: f64,         // 8 байт
    quantity: f64,      // 8 байт
    timestamp: u64,     // 8 байт
    order_id: u64,      // 8 байт
    symbol_id: u32,     // 4 байта
    side: u8,           // 1 байт
    order_type: u8,     // 1 байт
    _padding: [u8; 26], // Выравнивание до 64 байт
}

#[repr(C)]
struct CacheUnfriendlyOrder {
    // Холодные данные перемешаны с горячими — плохая утилизация кэша
    order_id: u64,
    notes: [u8; 256],         // Редко используется
    price: f64,
    customer_data: [u8; 128], // Редко используется
    quantity: f64,
    audit_log: [u8; 512],     // Редко используется
    timestamp: u64,
}

fn benchmark_cache_layout() {
    println!("=== Кэш-дружественная разметка данных ===\n");

    const ITERATIONS: usize = 1_000_000;

    // Кэш-дружественная разметка
    let orders: Vec<CacheFriendlyOrder> = (0..1000)
        .map(|i| CacheFriendlyOrder {
            price: 50000.0 + i as f64,
            quantity: 0.1,
            timestamp: i as u64,
            order_id: i as u64,
            symbol_id: 1,
            side: (i % 2) as u8,
            order_type: 0,
            _padding: [0; 26],
        })
        .collect();

    let start = Instant::now();
    let mut sum = 0.0f64;

    for _ in 0..ITERATIONS {
        for order in &orders {
            sum += order.price * order.quantity;
        }
    }

    println!("Кэш-дружественная разметка:");
    println!("  Время: {:?}", start.elapsed());
    println!("  Размер структуры: {} байт", std::mem::size_of::<CacheFriendlyOrder>());
    std::hint::black_box(sum);

    // Примечание: Бенчмарк кэш-недружественной разметки опущен для краткости
    // Недружественная разметка была бы значительно медленнее из-за промахов кэша

    println!("\nКлючевой вывод: Держи горячие данные вместе в кэш-линиях!");
}

fn main() {
    println!("=== Привязка к CPU и NUMA-осведомлённость ===\n");

    // Создаём движки для разных ядер
    let market_data_engine = TradingEngine::new("MarketData", 0);
    let order_engine = TradingEngine::new("OrderProcessing", 1);
    let risk_engine = TradingEngine::new("RiskManagement", 2);

    // Описываем настройки привязки
    market_data_engine.describe_affinity();
    order_engine.describe_affinity();
    risk_engine.describe_affinity();

    println!("\n--- Запуск бенчмарков ---\n");

    // Бенчмарк обработки ордеров
    let duration = order_engine.process_orders(100_000);
    println!("Обработка ордеров (100k ордеров): {:?}", duration);

    // Демо NUMA-осведомлённого буфера
    let mut buffer = NumaAwareBuffer::new(1024 * 1024, 0);
    buffer.write(0, 42);
    println!("Чтение из NUMA буфера: {}", buffer.read(0));

    println!();
    benchmark_cache_layout();
}
```

## Стратегия 5: Измерение и мониторинг p99

Нельзя оптимизировать то, что не измеряешь. Вот комплексная система мониторинга задержек:

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Высокоточная гистограмма задержек по принципам HDR Histogram
struct LatencyHistogram {
    // Бакеты для разных диапазонов задержек (в наносекундах)
    // 0-1мкс, 1-10мкс, 10-100мкс, 100мкс-1мс, 1-10мс, 10-100мс, 100мс+
    buckets: [AtomicU64; 7],
    total_count: AtomicU64,
    min_ns: AtomicU64,
    max_ns: AtomicU64,
    sum_ns: AtomicU64,
}

impl LatencyHistogram {
    fn new() -> Self {
        LatencyHistogram {
            buckets: [
                AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                AtomicU64::new(0),
            ],
            total_count: AtomicU64::new(0),
            min_ns: AtomicU64::new(u64::MAX),
            max_ns: AtomicU64::new(0),
            sum_ns: AtomicU64::new(0),
        }
    }

    fn record(&self, latency: Duration) {
        let nanos = latency.as_nanos() as u64;

        // Обновляем бакет
        let bucket_idx = match nanos {
            0..=1_000 => 0,             // 0-1мкс
            1_001..=10_000 => 1,        // 1-10мкс
            10_001..=100_000 => 2,      // 10-100мкс
            100_001..=1_000_000 => 3,   // 100мкс-1мс
            1_000_001..=10_000_000 => 4, // 1-10мс
            10_000_001..=100_000_000 => 5, // 10-100мс
            _ => 6,                      // 100мс+
        };

        self.buckets[bucket_idx].fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
        self.sum_ns.fetch_add(nanos, Ordering::Relaxed);

        // Обновляем min/max (используя compare-and-swap для потокобезопасности)
        let mut current_min = self.min_ns.load(Ordering::Relaxed);
        while nanos < current_min {
            match self.min_ns.compare_exchange_weak(
                current_min, nanos, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        let mut current_max = self.max_ns.load(Ordering::Relaxed);
        while nanos > current_max {
            match self.max_ns.compare_exchange_weak(
                current_max, nanos, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    fn report(&self) {
        let total = self.total_count.load(Ordering::Relaxed);
        if total == 0 {
            println!("Нет записанных данных");
            return;
        }

        let bucket_names = [
            "0-1мкс", "1-10мкс", "10-100мкс", "100мкс-1мс",
            "1-10мс", "10-100мс", "100мс+"
        ];

        println!("Распределение задержек:");
        println!("{}", "-".repeat(50));

        let mut cumulative = 0u64;
        for (i, name) in bucket_names.iter().enumerate() {
            let count = self.buckets[i].load(Ordering::Relaxed);
            cumulative += count;
            let pct = count as f64 / total as f64 * 100.0;
            let cumulative_pct = cumulative as f64 / total as f64 * 100.0;

            let bar_len = (pct / 2.0) as usize;
            let bar: String = "#".repeat(bar_len);

            println!(
                "{:>12}: {:>8} ({:>5.1}%) [p{:>5.1}] {}",
                name, count, pct, cumulative_pct, bar
            );
        }

        println!("{}", "-".repeat(50));

        let min_us = self.min_ns.load(Ordering::Relaxed) as f64 / 1000.0;
        let max_us = self.max_ns.load(Ordering::Relaxed) as f64 / 1000.0;
        let avg_us = (self.sum_ns.load(Ordering::Relaxed) as f64 / total as f64) / 1000.0;

        println!("Статистика:");
        println!("  Кол-во: {}", total);
        println!("  Мин:    {:.2}мкс", min_us);
        println!("  Макс:   {:.2}мкс", max_us);
        println!("  Сред:   {:.2}мкс", avg_us);
    }
}

/// Монитор задержек для различных операций
struct OperationMonitor {
    histograms: HashMap<String, Arc<LatencyHistogram>>,
}

impl OperationMonitor {
    fn new() -> Self {
        OperationMonitor {
            histograms: HashMap::new(),
        }
    }

    fn get_or_create(&mut self, operation: &str) -> Arc<LatencyHistogram> {
        self.histograms
            .entry(operation.to_string())
            .or_insert_with(|| Arc::new(LatencyHistogram::new()))
            .clone()
    }

    fn report_all(&self) {
        for (name, histogram) in &self.histograms {
            println!("\n=== {} ===", name);
            histogram.report();
        }
    }
}

/// Процессор ордеров с учётом задержки
struct OrderProcessor {
    histogram: Arc<LatencyHistogram>,
}

impl OrderProcessor {
    fn new(histogram: Arc<LatencyHistogram>) -> Self {
        OrderProcessor { histogram }
    }

    fn process(&self, order_id: u64) {
        let start = Instant::now();

        // Симуляция обработки ордера
        let price = 50000.0 + (order_id % 1000) as f64;
        let quantity = 0.1 * (order_id % 10) as f64;
        let _value = price * quantity;
        std::hint::black_box(_value);

        // Периодически медленная операция
        if order_id % 100 == 0 {
            std::thread::sleep(Duration::from_micros(50));
        }

        self.histogram.record(start.elapsed());
    }
}

fn main() {
    println!("=== Система мониторинга задержек ===\n");

    let mut monitor = OperationMonitor::new();

    // Создаём процессоры для различных операций
    let order_histogram = monitor.get_or_create("OrderProcessing");
    let market_histogram = monitor.get_or_create("MarketDataUpdate");

    let order_processor = OrderProcessor::new(order_histogram);

    // Обрабатываем ордера
    println!("Обработка 10,000 ордеров...\n");
    for i in 0..10_000 {
        order_processor.process(i);
    }

    // Симулируем обновления рыночных данных
    let market_hist = market_histogram;
    for i in 0..5_000 {
        let start = Instant::now();

        // Симуляция обработки рыночных данных
        let _price = 50000.0 + (i as f64 * 0.001).sin() * 100.0;
        std::hint::black_box(_price);

        if i % 200 == 0 {
            std::thread::sleep(Duration::from_micros(100));
        }

        market_hist.record(start.elapsed());
    }

    // Выводим отчёты
    monitor.report_all();

    println!("\n=== Ключевые выводы ===");
    println!("1. Мониторь p99 постоянно в продакшене");
    println!("2. Настрой алерты на превышение порога p99");
    println!("3. Исследуй любой p99 > 10x p50");
    println!("4. Отслеживай тренды перцентилей во времени");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Tail Latency** | Задержка на высоких перцентилях (p99, p99.9) |
| **p99** | 99% запросов быстрее этого значения |
| **Предварительное выделение** | Выделяй память заранее, чтобы избежать всплесков |
| **Пулинг объектов** | Переиспользуй объекты вместо создания новых |
| **Lock-free** | Структуры данных без конкуренции за мьютексы |
| **Привязка к CPU** | Привязка потоков к конкретным ядрам CPU |
| **NUMA** | Осведомлённость о неоднородном доступе к памяти |
| **Кэш-линия** | 64-байтный блок кэша CPU |
| **Разделение горячего/холодного** | Разделяй часто и редко используемые данные |

## Практические задания

1. **Трекер задержек**: Создай систему, которая:
   - Записывает задержку для каждого типа ордера отдельно
   - Рассчитывает скользящие перцентили (p50, p90, p99)
   - Обнаруживает аномалии задержки в реальном времени
   - Экспортирует метрики для визуализации

2. **Реализация пула объектов**: Создай универсальный пул объектов:
   - Потокобезопасные acquire и release
   - Автоматическое расширение пула при исчерпании
   - Статистика использования пула
   - Эффективное хранение в памяти

3. **Lock-free очередь ордеров**: Реализуй MPSC очередь:
   - Множество производителей (потоки рыночных данных)
   - Один потребитель (процессор ордеров)
   - Ограниченная ёмкость с обратным давлением
   - Измерение задержки каждой операции

4. **Кэш-оптимизированная книга ордеров**: Спроектируй книгу ордеров:
   - Кэш-дружественная разметка данных
   - Минимум аллокаций при обновлениях
   - Предварительно выделенные ценовые уровни
   - Бенчмарк против наивной реализации

## Домашнее задание

1. **Полная система оптимизации p99**: Создай торговую систему, которая:
   - Измеряет задержку на каждом этапе обработки ордера
   - Реализует все 5 стратегий оптимизации из этой главы
   - Сравнивает p99 до и после каждой оптимизации
   - Генерирует детальный отчёт с графиками
   - Достигает p99 < 2x p50 для обработки ордеров

2. **Аллокатор бюджета задержки**: Создай инструмент, который:
   - Определяет бюджеты задержки для каждого компонента
   - Мониторит фактическую задержку относительно бюджетов
   - Алертит при превышении бюджета компонентами
   - Предлагает оптимизации на основе паттернов
   - Отслеживает использование бюджета во времени

3. **Адаптивный батч-процессор**: Реализуй процессор, который:
   - Динамически подстраивает размер батча на основе задержки
   - Балансирует пропускную способность и задержку
   - Поддерживает p99 в пределах целевого порога
   - Изящно обрабатывает пики нагрузки
   - Отчитывается о метриках эффективности

4. **Memory-mapped журнал ордеров**: Создай журнал, который:
   - Использует memory-mapped файлы для персистентности
   - Достигает стабильной задержки записи
   - Поддерживает восстановление после краша
   - Измеряет p99 задержки записи
   - Сравнивается с традиционным файловым I/O

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../333-*/ru.md)
