# День 308: Микрооптимизации: аллокации

## Аналогия из трейдинга

Представь трейдера, который постоянно переводит деньги между разными биржами для каждой сделки. Даже если стратегия прибыльная, накладные расходы на переводы съедают всю прибыль. Комиссия 0.1% за перевод кажется мелочью, но при 10,000 операций в день это уже серьёзные потери.

В программировании **аллокация памяти** — это как перевод средств между счетами. Каждый раз, когда мы создаём `Vec`, `String`, `Box` или `HashMap`, Rust обращается к системному аллокатору, что занимает время. Для большинства программ это незаметно, но в высокочастотном трейдинге (HFT), где счёт идёт на микросекунды, лишние аллокации могут стоить денег.

## Почему аллокации важны в алготрейдинге?

В торговых системах критична **latency** (задержка) — время между получением рыночных данных и отправкой ордера. Даже задержка в 100 микросекунд может означать проигрыш конкурентам.

| Операция | Типичное время | Влияние |
|----------|---------------|---------|
| Аллокация heap (malloc) | ~100-500 нс | Замедляет обработку |
| Копирование данных | ~10-50 нс/KB | Растёт с размером |
| Stack операции | ~1-5 нс | Почти незаметно |
| Системный вызов | ~1-10 мкс | Очень дорого |

**Проблемы чрезмерных аллокаций:**
1. **Latency спайки** — непредсказуемые задержки
2. **CPU cache misses** — данные разбросаны по памяти
3. **Фрагментация памяти** — неэффективное использование
4. **Garbage collection паузы** (не в Rust, но концепция важна)

## Где происходят аллокации в Rust?

```rust
// ❌ Аллокации heap
let prices = Vec::new();           // Аллокация при push
let symbol = String::from("BTCUSDT"); // Аллокация строки
let data = Box::new(MarketData {});   // Аллокация на heap
let map = HashMap::new();           // Аллокация + internal buffers

// ✅ Данные на stack (быстро)
let price = 42000.0;
let array = [0.0; 100];  // Фиксированный размер
let tuple = (price, quantity);
```

## Пример: Измерение аллокаций

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Аллокатор-обёртка для подсчёта аллокаций
struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn get_allocation_stats() -> (usize, usize, usize) {
    let allocated = ALLOCATED.load(Ordering::SeqCst);
    let deallocated = DEALLOCATED.load(Ordering::SeqCst);
    let count = ALLOCATION_COUNT.load(Ordering::SeqCst);
    (allocated, deallocated, count)
}

fn reset_allocation_stats() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
    ALLOCATION_COUNT.store(0, Ordering::SeqCst);
}

// Структура для хранения цены и объёма
#[derive(Debug, Clone)]
struct PriceLevel {
    price: f64,
    quantity: f64,
}

// ❌ Неоптимальная версия: много аллокаций
fn process_orderbook_slow(levels: &[(f64, f64)]) -> Vec<PriceLevel> {
    let mut result = Vec::new(); // Аллокация 1

    for &(price, qty) in levels {
        let level = PriceLevel { // Каждая итерация — потенциальная аллокация
            price,
            quantity: qty,
        };
        result.push(level); // Может вызвать реаллокацию
    }

    result
}

// ✅ Оптимизированная версия: одна аллокация
fn process_orderbook_fast(levels: &[(f64, f64)]) -> Vec<PriceLevel> {
    let mut result = Vec::with_capacity(levels.len()); // Точный размер

    for &(price, qty) in levels {
        result.push(PriceLevel { price, quantity: qty });
    }

    result
}

fn main() {
    let levels = vec![
        (42000.0, 1.5),
        (42001.0, 2.3),
        (42002.0, 0.8),
        (42003.0, 1.2),
        (42004.0, 3.1),
    ];

    println!("=== Сравнение аллокаций ===\n");

    // Тест неоптимальной версии
    reset_allocation_stats();
    let _ = process_orderbook_slow(&levels);
    let (alloc1, dealloc1, count1) = get_allocation_stats();
    println!("❌ Медленная версия (без with_capacity):");
    println!("   Аллоцировано: {} байт", alloc1);
    println!("   Количество аллокаций: {}", count1);

    // Тест оптимизированной версии
    reset_allocation_stats();
    let _ = process_orderbook_fast(&levels);
    let (alloc2, dealloc2, count2) = get_allocation_stats();
    println!("\n✅ Быстрая версия (с with_capacity):");
    println!("   Аллоцировано: {} байт", alloc2);
    println!("   Количество аллокаций: {}", count2);

    println!("\n📊 Разница:");
    println!("   Экономия аллокаций: {} раз", count1 as f64 / count2 as f64);
}
```

## Оптимизация 1: Предварительное выделение памяти

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}

// ❌ Плохо: растущие аллокации
fn collect_trades_bad(count: usize) -> Vec<Trade> {
    let mut trades = Vec::new(); // Capacity = 0

    for i in 0..count {
        trades.push(Trade {  // Реаллокация при 1, 2, 4, 8, 16...
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }

    trades
}

// ✅ Хорошо: одна аллокация
fn collect_trades_good(count: usize) -> Vec<Trade> {
    let mut trades = Vec::with_capacity(count); // Точный размер

    for i in 0..count {
        trades.push(Trade {
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }

    trades
}

// ✅ Ещё лучше: переиспользование буфера
fn collect_trades_reuse(count: usize, buffer: &mut Vec<Trade>) {
    buffer.clear(); // Не деаллоцирует память
    buffer.reserve(count); // Расширяет capacity если нужно

    for i in 0..count {
        buffer.push(Trade {
            symbol: format!("BTC-{}", i),
            price: 42000.0 + i as f64,
            quantity: 0.1,
        });
    }
}

fn main() {
    println!("=== Стратегии предварительного выделения ===\n");

    let count = 1000;

    // Версия с реаллокациями
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let trades1 = collect_trades_bad(count);
    let time1 = start.elapsed();
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ Без capacity:");
    println!("   Время: {:?}", time1);
    println!("   Аллокаций: {}", count1);

    // Версия с with_capacity
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let trades2 = collect_trades_good(count);
    let time2 = start.elapsed();
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ С with_capacity:");
    println!("   Время: {:?}", time2);
    println!("   Аллокаций: {}", count2);

    // Версия с переиспользованием
    let mut buffer = Vec::new();
    reset_allocation_stats();
    let start = std::time::Instant::now();
    collect_trades_reuse(count, &mut buffer);
    let time3 = start.elapsed();
    let (alloc3, _, count3) = get_allocation_stats();
    println!("\n✅ С переиспользованием буфера:");
    println!("   Время: {:?}", time3);
    println!("   Аллокаций: {}", count3);

    println!("\n📊 Ускорение: {:.2}x", time1.as_nanos() as f64 / time3.as_nanos() as f64);
}
```

## Оптимизация 2: Избегаем клонирования строк

```rust
use std::borrow::Cow;

#[derive(Debug)]
struct Order<'a> {
    symbol: Cow<'a, str>,  // Copy-on-write: заимствует или владеет
    price: f64,
    quantity: f64,
}

// ❌ Плохо: клонирует строку каждый раз
fn create_order_bad(symbol: &str, price: f64, qty: f64) -> Order<'static> {
    Order {
        symbol: Cow::Owned(symbol.to_string()), // Аллокация
        price,
        quantity: qty,
    }
}

// ✅ Хорошо: заимствует строку
fn create_order_good(symbol: &str, price: f64, qty: f64) -> Order {
    Order {
        symbol: Cow::Borrowed(symbol), // Без аллокации
        price,
        quantity: qty,
    }
}

// ✅ Умное решение: клонирует только при изменении
fn normalize_symbol(symbol: &str) -> Cow<str> {
    if symbol.contains('-') {
        // Нужно изменить — клонируем
        Cow::Owned(symbol.replace('-', ""))
    } else {
        // Без изменений — заимствуем
        Cow::Borrowed(symbol)
    }
}

fn main() {
    println!("=== Оптимизация строк с Cow ===\n");

    let symbol = "BTCUSDT";

    // Плохой вариант
    reset_allocation_stats();
    let order1 = create_order_bad(symbol, 42000.0, 1.0);
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ С клонированием: {} аллокаций, {} байт", count1, alloc1);

    // Хороший вариант
    reset_allocation_stats();
    let order2 = create_order_good(symbol, 42000.0, 1.0);
    let (alloc2, _, count2) = get_allocation_stats();
    println!("✅ С заимствованием: {} аллокаций, {} байт", count2, alloc2);

    println!("\n=== Умное клонирование ===");

    reset_allocation_stats();
    let normalized1 = normalize_symbol("BTCUSDT");
    let (a1, _, c1) = get_allocation_stats();
    println!("Без '-': {} аллокаций (заимствование)", c1);

    reset_allocation_stats();
    let normalized2 = normalize_symbol("BTC-USDT");
    let (a2, _, c2) = get_allocation_stats();
    println!("С '-': {} аллокаций (клонирование для изменения)", c2);
}
```

## Оптимизация 3: Пулы объектов

```rust
use std::collections::VecDeque;

/// Пул для переиспользования буферов
struct BufferPool {
    pool: VecDeque<Vec<f64>>,
    capacity: usize,
}

impl BufferPool {
    fn new(capacity: usize) -> Self {
        BufferPool {
            pool: VecDeque::new(),
            capacity,
        }
    }

    /// Берём буфер из пула или создаём новый
    fn acquire(&mut self) -> Vec<f64> {
        self.pool.pop_front().unwrap_or_else(|| Vec::with_capacity(self.capacity))
    }

    /// Возвращаем буфер в пул
    fn release(&mut self, mut buffer: Vec<f64>) {
        buffer.clear(); // Очищаем данные, но сохраняем capacity
        if self.pool.len() < 10 { // Ограничиваем размер пула
            self.pool.push_back(buffer);
        }
        // Иначе буфер будет дропнут
    }
}

/// Расчёт скользящей средней
fn calculate_sma(prices: &[f64], period: usize, buffer: &mut Vec<f64>) -> f64 {
    buffer.clear();
    buffer.extend_from_slice(&prices[prices.len() - period..]);
    buffer.iter().sum::<f64>() / period as f64
}

fn main() {
    let prices: Vec<f64> = (0..1000).map(|i| 42000.0 + i as f64 * 0.5).collect();
    let iterations = 10000;

    println!("=== Сравнение с пулом и без пула ===\n");

    // ❌ Без пула: каждый раз новая аллокация
    reset_allocation_stats();
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut buffer = Vec::new();
        let _sma = calculate_sma(&prices, 20, &mut buffer);
    }
    let time1 = start.elapsed();
    let (alloc1, _, count1) = get_allocation_stats();
    println!("❌ Без пула:");
    println!("   Время: {:?}", time1);
    println!("   Аллокаций: {}", count1);

    // ✅ С пулом: переиспользуем буферы
    reset_allocation_stats();
    let start = std::time::Instant::now();
    let mut pool = BufferPool::new(100);
    for _ in 0..iterations {
        let mut buffer = pool.acquire();
        let _sma = calculate_sma(&prices, 20, &mut buffer);
        pool.release(buffer);
    }
    let time2 = start.elapsed();
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ С пулом:");
    println!("   Время: {:?}", time2);
    println!("   Аллокаций: {}", count2);

    println!("\n📊 Ускорение: {:.2}x", time1.as_nanos() as f64 / time2.as_nanos() as f64);
    println!("📊 Сокращение аллокаций: {:.1}%", (1.0 - count2 as f64 / count1 as f64) * 100.0);
}
```

## Оптимизация 4: SmallVec — гибридное хранение

```rust
// Требует: cargo add smallvec
use smallvec::{SmallVec, smallvec};

/// Обработка небольших списков ордеров
#[derive(Debug)]
struct OrderUpdate {
    // До 4 элементов на stack, больше — на heap
    price_levels: SmallVec<[(f64, f64); 4]>,
}

impl OrderUpdate {
    fn new() -> Self {
        OrderUpdate {
            price_levels: smallvec![],
        }
    }

    fn add_level(&mut self, price: f64, qty: f64) {
        self.price_levels.push((price, qty));
    }

    fn total_volume(&self) -> f64 {
        self.price_levels.iter().map(|(_, qty)| qty).sum()
    }
}

fn main() {
    println!("=== SmallVec: stack для малых данных ===\n");

    // Малый объём — остаётся на stack
    reset_allocation_stats();
    let mut update1 = OrderUpdate::new();
    update1.add_level(42000.0, 1.0);
    update1.add_level(42001.0, 2.0);
    let (alloc1, _, count1) = get_allocation_stats();
    println!("✅ 2 элемента (stack):");
    println!("   Аллокаций: {}", count1);
    println!("   Объём: {}", update1.total_volume());

    // Больше 4 элементов — переходит на heap
    reset_allocation_stats();
    let mut update2 = OrderUpdate::new();
    for i in 0..10 {
        update2.add_level(42000.0 + i as f64, 1.0);
    }
    let (alloc2, _, count2) = get_allocation_stats();
    println!("\n✅ 10 элементов (heap после 4-го):");
    println!("   Аллокаций: {}", count2);
    println!("   Объём: {}", update2.total_volume());

    println!("\n📝 SmallVec автоматически переключается между stack и heap");
}
```

## Практические рекомендации

### Когда оптимизировать аллокации?

1. **Измерь сначала** — используй профайлеры (perf, valgrind, heaptrack)
2. **Горячие пути** — оптимизируй код, который выполняется тысячи раз в секунду
3. **Latency-critical** — HFT, real-time обработка market data
4. **Не оптимизируй преждевременно** — читаемость важнее микрооптимизаций

### Чек-лист оптимизации аллокаций

```rust
// ✅ Хорошие практики
Vec::with_capacity(n)        // Предварительное выделение
HashMap::with_capacity(n)    // То же для map
String::with_capacity(n)     // И для строк
buffer.clear()               // Переиспользование вместо нового Vec::new()
&str вместо String           // Заимствование вместо владения
Cow<str>                     // Copy-on-write
SmallVec                     // Stack для малых данных
arrayvec                     // Vec фиксированного размера на stack
```

### Инструменты для анализа

```bash
# Профилирование аллокаций
cargo install cargo-flamegraph
cargo flamegraph --bin my_trading_bot

# Анализ heap использования
valgrind --tool=massif ./target/release/my_trading_bot
ms_print massif.out.*

# Подсчёт аллокаций
cargo add dhat
# Добавить #[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Heap allocation** | Аллокация памяти через системный аллокатор (медленно) |
| **Stack allocation** | Быстрое выделение памяти на стеке |
| **with_capacity** | Предварительное выделение для Vec/HashMap/String |
| **Buffer reuse** | Переиспользование буферов через .clear() |
| **Cow** | Copy-on-write: заимствует или клонирует по необходимости |
| **Object pool** | Пул объектов для переиспользования |
| **SmallVec** | Гибрид stack/heap для малых коллекций |
| **Zero-copy** | Обработка данных без копирования |

## Домашнее задание

1. **Профилирование аллокаций**: Напиши программу, которая:
   - Обрабатывает поток market data (1000 обновлений/сек)
   - Считает топ-10 активных инструментов
   - Замеряй аллокации до и после оптимизации
   - Используй custom allocator для подсчёта

2. **Object Pool для ордеров**: Реализуй пул объектов:
   - Пул структур `Order` с capacity 1000
   - Методы acquire/release
   - Автоматическое расширение пула при необходимости
   - Статистика использования (hit rate, miss rate)

3. **Zero-allocation парсер**: Создай парсер JSON market data:
   - Использует `&str` вместо `String` где возможно
   - SmallVec для массивов
   - Переиспользует буферы между вызовами
   - Сравни с наивной версией

4. **Бенчмарк сравнение**: Протестируй производительность:
   - `Vec` vs `SmallVec` vs `ArrayVec`
   - `String` vs `&str` vs `Cow<str>`
   - HashMap с и без `with_capacity`
   - Используй criterion для точных измерений

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
