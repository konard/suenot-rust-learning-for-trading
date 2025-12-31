# День 325: jemalloc и mimalloc

## Аналогия из трейдинга

Представь, что ты управляешь складом для торговых товаров. У тебя есть два варианта организации склада:

**Стандартный склад (системный аллокатор)**:
- Универсальный, подходит для всего
- Иногда работники долго ищут место для нового товара
- При высокой нагрузке образуются очереди
- Хорошо работает для небольших объёмов

**Специализированный склад (jemalloc/mimalloc)**:
- Оптимизирован под конкретные типы товаров
- Каждый работник имеет свою зону (thread-local cache)
- Минимум ожидания даже при высокой нагрузке
- Идеален для масштабных операций

В торговой системе это критично:
- **Высокочастотный трейдинг** — тысячи ордеров в секунду, каждая микросекунда на счету
- **Market data processing** — потоки данных требуют быстрого выделения памяти
- **Многопоточность** — каждый поток обрабатывает свой символ или биржу

## Что такое альтернативные аллокаторы?

Стандартный аллокатор памяти в Rust использует системный (glibc malloc на Linux, Windows Heap на Windows). Альтернативные аллокаторы предоставляют улучшенные характеристики:

| Аллокатор | Особенности | Когда использовать |
|-----------|-------------|-------------------|
| **Системный** | Универсальный, без дополнительных зависимостей | Небольшие приложения |
| **jemalloc** | Отличная многопоточность, низкая фрагментация | Серверы, долгоживущие процессы |
| **mimalloc** | Максимальная скорость, маленький размер | HFT, микросервисы |
| **tcmalloc** | Thread-caching, профилирование | Google-style инфраструктура |

### jemalloc

Разработан Facebook для серверных приложений:
- **Thread-local caches** — каждый поток имеет свой кэш для быстрых аллокаций
- **Arena-based allocation** — память разделена на арены для минимизации конфликтов
- **Low fragmentation** — эффективное переиспользование освобождённой памяти
- **Introspection** — встроенные инструменты для анализа использования памяти

### mimalloc

Разработан Microsoft Research:
- **Segment-based** — память организована в сегменты по размеру объектов
- **Free list sharding** — распределённые списки свободных блоков
- **Очень маленький overhead** — минимальные метаданные на аллокацию
- **Отличная масштабируемость** — почти линейная производительность с числом потоков

## Подключение jemalloc

### Cargo.toml

```toml
[dependencies]
tikv-jemallocator = "0.5"

[features]
default = []
jemalloc = ["tikv-jemallocator"]
```

### Использование в коде

```rust
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::collections::HashMap;
use std::time::Instant;

/// Стакан ордеров для торговой системы
struct OrderBook {
    bids: HashMap<u64, Order>,
    asks: HashMap<u64, Order>,
}

#[derive(Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    symbol: String,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    fn add_bid(&mut self, order: Order) {
        self.bids.insert(order.id, order);
    }

    fn add_ask(&mut self, order: Order) {
        self.asks.insert(order.id, order);
    }

    fn remove_order(&mut self, id: u64) -> bool {
        self.bids.remove(&id).is_some() || self.asks.remove(&id).is_some()
    }
}

fn benchmark_order_operations(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut book = OrderBook::new();

    for i in 0..iterations {
        // Добавляем ордера
        book.add_bid(Order {
            id: i as u64,
            price: 50000.0 + (i as f64 * 0.01),
            quantity: 0.1,
            symbol: "BTCUSDT".to_string(),
        });

        book.add_ask(Order {
            id: (i + iterations) as u64,
            price: 50001.0 + (i as f64 * 0.01),
            quantity: 0.1,
            symbol: "BTCUSDT".to_string(),
        });

        // Удаляем часть ордеров
        if i > 100 {
            book.remove_order((i - 100) as u64);
        }
    }

    start.elapsed()
}

fn main() {
    println!("=== Тест производительности аллокатора ===\n");

    #[cfg(feature = "jemalloc")]
    println!("Используется: jemalloc");

    #[cfg(not(feature = "jemalloc"))]
    println!("Используется: системный аллокатор");

    let iterations = 100_000;

    // Прогрев
    let _ = benchmark_order_operations(1000);

    // Замер
    let duration = benchmark_order_operations(iterations);

    println!("\nРезультаты:");
    println!("  Итераций: {}", iterations);
    println!("  Время: {:?}", duration);
    println!("  Операций/сек: {:.0}", iterations as f64 / duration.as_secs_f64());
}
```

Запуск:
```bash
# С системным аллокатором
cargo run --release

# С jemalloc
cargo run --release --features jemalloc
```

## Подключение mimalloc

### Cargo.toml

```toml
[dependencies]
mimalloc = "0.1"

[features]
default = []
mimalloc = ["mimalloc"]
```

### Использование в коде

```rust
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Ценовые данные для market data feed
#[derive(Clone)]
struct PriceTick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// Обработчик потока рыночных данных
struct MarketDataProcessor {
    buffer: Vec<PriceTick>,
    capacity: usize,
}

impl MarketDataProcessor {
    fn new(capacity: usize) -> Self {
        MarketDataProcessor {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn process_tick(&mut self, tick: PriceTick) {
        if self.buffer.len() >= self.capacity {
            // Очищаем старые данные
            self.buffer.drain(..self.capacity / 2);
        }
        self.buffer.push(tick);
    }

    fn get_latest_price(&self, symbol: &str) -> Option<(f64, f64)> {
        self.buffer
            .iter()
            .rev()
            .find(|t| t.symbol == symbol)
            .map(|t| (t.bid, t.ask))
    }
}

fn benchmark_multithreaded(threads: usize, ticks_per_thread: usize) -> std::time::Duration {
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|thread_id| {
            thread::spawn(move || {
                let mut processor = MarketDataProcessor::new(10000);

                for i in 0..ticks_per_thread {
                    let tick = PriceTick {
                        symbol: format!("SYM{}", i % 100),
                        bid: 100.0 + (i as f64 * 0.001),
                        ask: 100.01 + (i as f64 * 0.001),
                        timestamp: i as u64,
                    };
                    processor.process_tick(tick);
                }

                processor.buffer.len()
            })
        })
        .collect();

    let total_processed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let duration = start.elapsed();

    println!(
        "  Потоков: {}, Обработано тиков: {}",
        threads, total_processed
    );

    duration
}

fn main() {
    println!("=== Многопоточный тест mimalloc ===\n");

    #[cfg(feature = "mimalloc")]
    println!("Используется: mimalloc");

    #[cfg(not(feature = "mimalloc"))]
    println!("Используется: системный аллокатор");

    let ticks_per_thread = 100_000;

    for threads in [1, 2, 4, 8] {
        println!("\nТест с {} потоками:", threads);
        let duration = benchmark_multithreaded(threads, ticks_per_thread);
        let total_ticks = threads * ticks_per_thread;
        println!(
            "  Время: {:?}, Тиков/сек: {:.0}",
            duration,
            total_ticks as f64 / duration.as_secs_f64()
        );
    }
}
```

## Сравнение производительности

### Бенчмарк для торговой системы

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::collections::HashMap;

// Счётчик аллокаций для анализа
struct CountingAllocator<A: GlobalAlloc> {
    inner: A,
}

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES_ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        self.inner.dealloc(ptr, layout)
    }
}

fn reset_counters() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    DEALLOC_COUNT.store(0, Ordering::Relaxed);
    BYTES_ALLOCATED.store(0, Ordering::Relaxed);
}

fn get_stats() -> (usize, usize, usize) {
    (
        ALLOC_COUNT.load(Ordering::Relaxed),
        DEALLOC_COUNT.load(Ordering::Relaxed),
        BYTES_ALLOCATED.load(Ordering::Relaxed),
    )
}

/// Симуляция торгового движка
struct TradingEngine {
    positions: HashMap<String, f64>,
    order_history: Vec<TradeOrder>,
    price_cache: HashMap<String, Vec<f64>>,
}

#[derive(Clone)]
struct TradeOrder {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine {
            positions: HashMap::new(),
            order_history: Vec::new(),
            price_cache: HashMap::new(),
        }
    }

    fn process_order(&mut self, order: TradeOrder) {
        // Обновляем позицию
        let position = self.positions.entry(order.symbol.clone()).or_insert(0.0);
        match order.side {
            OrderSide::Buy => *position += order.quantity,
            OrderSide::Sell => *position -= order.quantity,
        }

        // Кэшируем цену
        self.price_cache
            .entry(order.symbol.clone())
            .or_insert_with(Vec::new)
            .push(order.price);

        // Сохраняем в историю
        self.order_history.push(order);

        // Очищаем старый кэш цен (ограничиваем размер)
        for prices in self.price_cache.values_mut() {
            if prices.len() > 1000 {
                prices.drain(..500);
            }
        }
    }

    fn get_position(&self, symbol: &str) -> f64 {
        *self.positions.get(symbol).unwrap_or(&0.0)
    }
}

fn run_trading_simulation(orders: usize) -> (std::time::Duration, (usize, usize, usize)) {
    reset_counters();

    let start = Instant::now();
    let mut engine = TradingEngine::new();

    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT"];

    for i in 0..orders {
        let order = TradeOrder {
            id: i as u64,
            symbol: symbols[i % symbols.len()].to_string(),
            side: if i % 2 == 0 {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            },
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01 + (i as f64 * 0.001) % 1.0,
        };
        engine.process_order(order);
    }

    let duration = start.elapsed();
    let stats = get_stats();

    (duration, stats)
}

fn main() {
    println!("=== Сравнение производительности аллокаторов ===\n");

    let test_sizes = [10_000, 50_000, 100_000];

    for &size in &test_sizes {
        println!("Тест с {} ордерами:", size);

        let (duration, (allocs, deallocs, bytes)) = run_trading_simulation(size);

        println!("  Время: {:?}", duration);
        println!("  Аллокаций: {}", allocs);
        println!("  Деаллокаций: {}", deallocs);
        println!("  Байт выделено: {} KB", bytes / 1024);
        println!(
            "  Ордеров/сек: {:.0}",
            size as f64 / duration.as_secs_f64()
        );
        println!();
    }
}
```

## Продвинутые настройки

### Настройка jemalloc через переменные окружения

```bash
# Включить профилирование
export MALLOC_CONF="prof:true,prof_prefix:jeprof.out"

# Настроить размер арен
export MALLOC_CONF="narenas:4"

# Включить статистику
export MALLOC_CONF="stats_print:true"
```

### Программная настройка jemalloc

```rust
#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[cfg(feature = "jemalloc")]
use tikv_jemalloc_ctl::{epoch, stats};

fn print_jemalloc_stats() {
    #[cfg(feature = "jemalloc")]
    {
        // Обновляем статистику
        epoch::advance().unwrap();

        let allocated = stats::allocated::read().unwrap();
        let resident = stats::resident::read().unwrap();

        println!("=== Статистика jemalloc ===");
        println!("  Выделено: {} KB", allocated / 1024);
        println!("  Резидентная память: {} KB", resident / 1024);
        println!("  Фрагментация: {:.1}%",
                 (resident - allocated) as f64 / resident as f64 * 100.0);
    }

    #[cfg(not(feature = "jemalloc"))]
    println!("jemalloc не включён");
}

fn main() {
    println!("=== Мониторинг памяти торговой системы ===\n");

    // Симулируем работу
    let mut data: Vec<Vec<f64>> = Vec::new();

    for i in 0..100 {
        data.push((0..10000).map(|x| x as f64 * 0.001).collect());

        if i % 10 == 0 {
            print_jemalloc_stats();
            // Освобождаем часть памяти
            if data.len() > 50 {
                data.drain(..25);
            }
        }
    }

    println!("\nФинальная статистика:");
    print_jemalloc_stats();
}
```

### Настройка mimalloc

```rust
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    // mimalloc настраивается через переменные окружения:
    // MIMALLOC_VERBOSE=1 - включить логи
    // MIMALLOC_SHOW_STATS=1 - показать статистику при завершении
    // MIMALLOC_ARENA_EAGER_COMMIT=1 - жадное выделение арен
    // MIMALLOC_LARGE_OS_PAGES=1 - использовать huge pages

    println!("=== Тест mimalloc ===\n");

    let mut buffers: Vec<Vec<u8>> = Vec::new();

    // Создаём нагрузку с разными размерами аллокаций
    for size in [64, 256, 1024, 4096, 16384, 65536] {
        println!("Аллокации размером {} байт:", size);

        for _ in 0..1000 {
            buffers.push(vec![0u8; size]);
        }

        println!("  Создано {} буферов", buffers.len());

        // Освобождаем половину
        buffers.truncate(buffers.len() / 2);
    }

    println!("\nФинальное количество буферов: {}", buffers.len());
}
```

## Выбор аллокатора для торговой системы

### Рекомендации по выбору

```rust
/// Выбор аллокатора в зависимости от use case
///
/// | Сценарий                    | Рекомендуемый аллокатор |
/// |-----------------------------|-------------------------|
/// | HFT (< 1ms latency)         | mimalloc                |
/// | Market data processing      | jemalloc                |
/// | Backtesting (много памяти)  | jemalloc                |
/// | Микросервисы                | mimalloc                |
/// | Долгоживущие процессы       | jemalloc                |
/// | Embedded/Edge               | mimalloc (меньше размер)|

// Пример условной компиляции для разных окружений
#[cfg(all(feature = "mimalloc", target_os = "linux"))]
use mimalloc::MiMalloc;

#[cfg(all(feature = "jemalloc", target_os = "linux"))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(feature = "mimalloc", target_os = "linux"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(all(feature = "jemalloc", target_os = "linux", not(feature = "mimalloc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn get_allocator_name() -> &'static str {
    #[cfg(feature = "mimalloc")]
    return "mimalloc";

    #[cfg(all(feature = "jemalloc", not(feature = "mimalloc")))]
    return "jemalloc";

    #[cfg(not(any(feature = "mimalloc", feature = "jemalloc")))]
    return "system";
}

fn main() {
    println!("Активный аллокатор: {}", get_allocator_name());
}
```

### Бенчмарк для выбора оптимального аллокатора

```rust
use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Бенчмарк паттернов аллокации в торговых системах
struct AllocationBenchmark {
    small_allocs: u64,    // < 256 байт (ордера, тики)
    medium_allocs: u64,   // 256 - 4KB (буферы сообщений)
    large_allocs: u64,    // > 4KB (исторические данные)
}

impl AllocationBenchmark {
    fn new() -> Self {
        AllocationBenchmark {
            small_allocs: 0,
            medium_allocs: 0,
            large_allocs: 0,
        }
    }

    fn run_small_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut items: Vec<Box<[u8; 64]>> = Vec::new();
        for _ in 0..count {
            items.push(Box::new([0u8; 64]));
        }

        // Случайное освобождение
        for i in (0..items.len()).step_by(3) {
            if i < items.len() {
                items.swap_remove(i.min(items.len() - 1));
            }
        }

        self.small_allocs = start.elapsed().as_nanos() as u64;
    }

    fn run_medium_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut buffers: Vec<Vec<u8>> = Vec::new();
        for i in 0..count {
            buffers.push(vec![0u8; 256 + (i % 3840)]);
        }

        // FIFO освобождение (типично для буферов сообщений)
        while buffers.len() > count / 2 {
            buffers.remove(0);
        }

        self.medium_allocs = start.elapsed().as_nanos() as u64;
    }

    fn run_large_allocations(&mut self, count: usize) {
        let start = Instant::now();

        let mut data: Vec<Vec<f64>> = Vec::new();
        for _ in 0..count {
            // Исторические данные: 1000 свечей
            data.push(vec![0.0; 1000]);
        }

        // Очистка старых данных
        data.retain(|v| v.len() > 500);

        self.large_allocs = start.elapsed().as_nanos() as u64;
    }

    fn report(&self) {
        println!("  Мелкие аллокации: {} мкс", self.small_allocs / 1000);
        println!("  Средние аллокации: {} мкс", self.medium_allocs / 1000);
        println!("  Крупные аллокации: {} мкс", self.large_allocs / 1000);
        println!("  Всего: {} мкс",
                 (self.small_allocs + self.medium_allocs + self.large_allocs) / 1000);
    }
}

fn run_benchmark() {
    let mut bench = AllocationBenchmark::new();

    bench.run_small_allocations(50_000);
    bench.run_medium_allocations(10_000);
    bench.run_large_allocations(1_000);

    bench.report();
}

fn run_multithreaded_benchmark(threads: usize) {
    let start = Instant::now();
    let total_ops = Arc::new(AtomicU64::new(0));

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let ops = Arc::clone(&total_ops);
            thread::spawn(move || {
                let mut local_ops = 0u64;

                for _ in 0..10_000 {
                    // Типичный паттерн: создать-обработать-удалить
                    let order = Box::new([0u8; 128]);
                    let _ = order.len();
                    drop(order);
                    local_ops += 1;
                }

                ops.fetch_add(local_ops, Ordering::Relaxed);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let duration = start.elapsed();
    let ops = total_ops.load(Ordering::Relaxed);

    println!(
        "  {} потоков: {} ops, {:?}, {:.0} ops/sec",
        threads,
        ops,
        duration,
        ops as f64 / duration.as_secs_f64()
    );
}

fn main() {
    println!("=== Бенчмарк для выбора аллокатора ===\n");

    println!("Однопоточный тест:");
    run_benchmark();

    println!("\nМногопоточный тест:");
    for threads in [1, 2, 4, 8] {
        run_multithreaded_benchmark(threads);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **jemalloc** | Аллокатор с thread-local caches и аренами для серверных приложений |
| **mimalloc** | Быстрый аллокатор от Microsoft с низким overhead |
| **Thread-local cache** | Локальный кэш для каждого потока, минимизирующий блокировки |
| **Arena** | Изолированная область памяти для группы аллокаций |
| **Фрагментация** | Неэффективное использование памяти из-за разбросанных свободных блоков |
| **Global allocator** | Атрибут Rust для замены стандартного аллокатора |

## Практические задания

1. **Сравнительный бенчмарк**: Создай программу, которая:
   - Замеряет производительность системного, jemalloc и mimalloc аллокаторов
   - Тестирует разные паттерны аллокации (мелкие, средние, крупные)
   - Генерирует отчёт с рекомендацией
   - Учитывает многопоточность

2. **Мониторинг памяти**: Реализуй систему мониторинга:
   - Отслеживает использование памяти в реальном времени
   - Определяет фрагментацию
   - Предупреждает о потенциальных проблемах
   - Интегрируется с торговыми метриками

3. **Адаптивный аллокатор**: Напиши обёртку, которая:
   - Переключается между аллокаторами в зависимости от нагрузки
   - Оптимизирует под конкретные паттерны использования
   - Собирает статистику для анализа
   - Предоставляет API для настройки

4. **Профилирование торговой системы**: Создай инструмент:
   - Профилирует аллокации по компонентам системы
   - Определяет hotspots
   - Предлагает оптимизации
   - Генерирует визуализацию

## Домашнее задание

1. **Оптимизация HFT-бота**: Возьми существующий торговый бот и:
   - Измерь базовую производительность
   - Попробуй jemalloc и mimalloc
   - Найди оптимальную конфигурацию
   - Задокументируй результаты с графиками
   - Добейся улучшения latency минимум на 20%

2. **Memory Pool для ордеров**: Реализуй пул объектов:
   - Предаллоцирует N объектов Order
   - Переиспользует объекты вместо аллокации/деаллокации
   - Измерь разницу в производительности
   - Сравни с использованием альтернативных аллокаторов
   - Определи точку безубыточности (когда пул выгоднее)

3. **CI Pipeline с бенчмарками**: Создай pipeline:
   - Запускает бенчмарки при каждом PR
   - Сравнивает с baseline
   - Предупреждает о регрессиях производительности
   - Хранит историю результатов
   - Генерирует отчёты для code review

4. **Гибридная стратегия аллокации**: Разработай систему:
   - Использует разные аллокаторы для разных типов данных
   - Hot path использует mimalloc
   - Cold data использует jemalloc
   - Автоматически профилирует и адаптируется
   - Документирует trade-offs

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../326-*/ru.md)
