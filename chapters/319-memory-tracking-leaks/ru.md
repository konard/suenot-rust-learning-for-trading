# День 319: Память: отслеживание утечек

## Аналогия из трейдинга

Представь, что ты управляешь инвестиционным портфелем. Каждый день ты покупаешь новые активы, но иногда забываешь продавать те, которые больше не нужны. Со временем портфель раздувается: деньги заморожены в забытых позициях, ликвидность падает, а расходы на обслуживание растут.

**Утечка памяти** — это та же проблема, но с оперативной памятью:
- **Покупка актива** = выделение памяти под объект
- **Продажа актива** = освобождение памяти
- **Забытая позиция** = утечка памяти

В торговой системе это критично:
- Бот работает 24/7, накапливая утечки
- Память заканчивается → система падает в разгар торговли
- Потеря денег из-за пропущенных сделок

Отслеживание утечек — это как регулярный аудит портфеля: находим "мёртвые" позиции и закрываем их.

## Что такое утечка памяти в Rust?

Хотя Rust гарантирует безопасность памяти, утечки всё ещё возможны:

| Тип утечки | Описание | Пример в трейдинге |
|------------|----------|-------------------|
| **Циклические ссылки** | `Rc`/`Arc` образуют цикл | Стратегия ссылается на менеджер, менеджер на стратегию |
| **Забытые каналы** | Sender/Receiver не закрыты | Канал обновлений цен без подписчиков |
| **Бесконечно растущие коллекции** | HashMap/Vec без очистки | История всех сделок за годы в памяти |
| **mem::forget** | Явный пропуск Drop | Намеренный leak ресурсов |

### Почему Rust не защищает от утечек?

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct TradingStrategy {
    name: String,
    // Циклическая ссылка на менеджер
    manager: RefCell<Option<Rc<StrategyManager>>>,
}

struct StrategyManager {
    strategies: Vec<Rc<TradingStrategy>>,
}

fn create_leak() {
    let strategy = Rc::new(TradingStrategy {
        name: "Scalper".to_string(),
        manager: RefCell::new(None),
    });

    let manager = Rc::new(StrategyManager {
        strategies: vec![Rc::clone(&strategy)],
    });

    // Создаём цикл: strategy -> manager -> strategy
    *strategy.manager.borrow_mut() = Some(Rc::clone(&manager));

    // При выходе: Rc count > 0, память не освобождается!
    println!("Стратегия: {}", strategy.name);
}

fn main() {
    create_leak();
    println!("Функция завершилась, но память не освобождена!");
}
```

## Инструменты для отслеживания утечек

### 1. Valgrind (Linux)

Классический инструмент для обнаружения утечек памяти:

```rust
// trading_bot.rs
use std::collections::HashMap;

struct OrderBook {
    orders: HashMap<u64, Order>,
}

struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: HashMap::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.insert(order.id, order);
    }

    // Утечка: забыли реализовать удаление старых ордеров!
    fn process_fills(&mut self, filled_ids: &[u64]) {
        for id in filled_ids {
            // Должно быть: self.orders.remove(id);
            // Но мы просто логируем и забываем удалить
            if let Some(order) = self.orders.get(id) {
                println!("Ордер {} исполнен: {} @ {}", id, order.symbol, order.price);
            }
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Симуляция: добавляем ордера каждую секунду
    for i in 0..10000 {
        book.add_order(Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.01,
        });

        // Ордера "исполняются", но не удаляются
        if i > 0 && i % 100 == 0 {
            let filled: Vec<u64> = ((i - 100)..i).collect();
            book.process_fills(&filled);
        }
    }

    println!("Активных ордеров: {}", book.orders.len());
    // Ожидаем 100, но имеем 10000!
}
```

Запуск с Valgrind:
```bash
cargo build --release
valgrind --leak-check=full ./target/release/trading_bot
```

### 2. Heaptrack (Linux)

Визуализация использования памяти во времени:

```bash
heaptrack ./target/release/trading_bot
heaptrack_gui heaptrack.trading_bot.*.gz
```

### 3. Instruments (macOS)

```bash
cargo build --release
instruments -t "Leaks" ./target/release/trading_bot
```

### 4. Встроенный аллокатор с трассировкой

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Аллокатор с подсчётом памяти
struct CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn memory_stats() -> (usize, usize, usize) {
    let alloc = ALLOCATED.load(Ordering::SeqCst);
    let dealloc = DEALLOCATED.load(Ordering::SeqCst);
    (alloc, dealloc, alloc.saturating_sub(dealloc))
}

fn print_memory_stats(label: &str) {
    let (alloc, dealloc, in_use) = memory_stats();
    println!(
        "[{}] Выделено: {} KB, Освобождено: {} KB, Используется: {} KB",
        label,
        alloc / 1024,
        dealloc / 1024,
        in_use / 1024
    );
}

fn main() {
    print_memory_stats("Старт");

    // Симуляция торговой активности
    let mut price_history: Vec<f64> = Vec::new();

    for i in 0..100_000 {
        price_history.push(50000.0 + (i as f64 * 0.01));

        if i % 10_000 == 0 {
            print_memory_stats(&format!("После {} цен", i));
        }
    }

    // Освобождаем историю
    drop(price_history);
    print_memory_stats("После очистки");
}
```

## Типичные паттерны утечек в торговых системах

### Паттерн 1: Бесконечно растущий кэш

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct PriceCache {
    cache: HashMap<String, Vec<(Instant, f64)>>,
    // Нет ограничения на размер!
}

impl PriceCache {
    fn new() -> Self {
        PriceCache {
            cache: HashMap::new(),
        }
    }

    // ПЛОХО: данные накапливаются бесконечно
    fn add_price_bad(&mut self, symbol: &str, price: f64) {
        self.cache
            .entry(symbol.to_string())
            .or_insert_with(Vec::new)
            .push((Instant::now(), price));
    }
}

// ИСПРАВЛЕНИЕ: ограниченный кэш

struct BoundedPriceCache {
    cache: HashMap<String, Vec<(Instant, f64)>>,
    max_age: Duration,
    max_entries_per_symbol: usize,
}

impl BoundedPriceCache {
    fn new(max_age: Duration, max_entries: usize) -> Self {
        BoundedPriceCache {
            cache: HashMap::new(),
            max_age,
            max_entries_per_symbol: max_entries,
        }
    }

    fn add_price(&mut self, symbol: &str, price: f64) {
        let now = Instant::now();
        let prices = self.cache
            .entry(symbol.to_string())
            .or_insert_with(Vec::new);

        // Добавляем новую цену
        prices.push((now, price));

        // Удаляем устаревшие
        prices.retain(|(time, _)| now.duration_since(*time) < self.max_age);

        // Ограничиваем размер
        if prices.len() > self.max_entries_per_symbol {
            let drain_count = prices.len() - self.max_entries_per_symbol;
            prices.drain(..drain_count);
        }
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        for prices in self.cache.values_mut() {
            prices.retain(|(time, _)| now.duration_since(*time) < self.max_age);
        }
        // Удаляем пустые записи
        self.cache.retain(|_, prices| !prices.is_empty());
    }
}

fn main() {
    let mut cache = BoundedPriceCache::new(
        Duration::from_secs(3600),  // Хранить цены за 1 час
        1000,                        // Максимум 1000 записей на символ
    );

    // Симуляция потока данных
    for i in 0..10000 {
        cache.add_price("BTCUSDT", 50000.0 + i as f64);

        if i % 1000 == 0 {
            cache.cleanup();
            println!("Размер кэша после очистки: {} записей",
                     cache.cache.values().map(|v| v.len()).sum::<usize>());
        }
    }
}
```

### Паттерн 2: Циклические ссылки через Weak

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Exchange {
    name: String,
    strategies: RefCell<Vec<Rc<Strategy>>>,
}

struct Strategy {
    name: String,
    // Используем Weak вместо Rc для предотвращения цикла
    exchange: Weak<Exchange>,
}

impl Strategy {
    fn execute(&self) {
        // Пытаемся получить сильную ссылку
        if let Some(exchange) = self.exchange.upgrade() {
            println!("Стратегия '{}' работает на бирже '{}'",
                     self.name, exchange.name);
        } else {
            println!("Биржа больше не доступна");
        }
    }
}

impl Drop for Strategy {
    fn drop(&mut self) {
        println!("Стратегия '{}' удалена", self.name);
    }
}

impl Drop for Exchange {
    fn drop(&mut self) {
        println!("Биржа '{}' удалена", self.name);
    }
}

fn main() {
    let exchange = Rc::new(Exchange {
        name: "Binance".to_string(),
        strategies: RefCell::new(Vec::new()),
    });

    let scalper = Rc::new(Strategy {
        name: "Scalper".to_string(),
        exchange: Rc::downgrade(&exchange),  // Weak ссылка
    });

    let arbitrage = Rc::new(Strategy {
        name: "Arbitrage".to_string(),
        exchange: Rc::downgrade(&exchange),  // Weak ссылка
    });

    exchange.strategies.borrow_mut().push(Rc::clone(&scalper));
    exchange.strategies.borrow_mut().push(Rc::clone(&arbitrage));

    scalper.execute();
    arbitrage.execute();

    println!("\nСбрасываем ссылки на стратегии...");
    drop(scalper);
    drop(arbitrage);

    println!("Exchange Rc count: {}", Rc::strong_count(&exchange));
    println!("\nСбрасываем exchange...");
    drop(exchange);
    // Всё корректно освобождается!
}
```

### Паттерн 3: Отслеживание памяти с помощью RAII

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

static ACTIVE_ORDERS: AtomicUsize = AtomicUsize::new(0);
static TOTAL_ORDERS_CREATED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct TrackedOrder {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl TrackedOrder {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        ACTIVE_ORDERS.fetch_add(1, Ordering::SeqCst);
        TOTAL_ORDERS_CREATED.fetch_add(1, Ordering::SeqCst);

        TrackedOrder {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
        }
    }
}

impl Drop for TrackedOrder {
    fn drop(&mut self) {
        ACTIVE_ORDERS.fetch_sub(1, Ordering::SeqCst);
    }
}

fn get_order_stats() -> (usize, usize) {
    (
        ACTIVE_ORDERS.load(Ordering::SeqCst),
        TOTAL_ORDERS_CREATED.load(Ordering::SeqCst),
    )
}

fn main() {
    println!("=== Тест отслеживания ордеров ===\n");

    {
        let mut orders = Vec::new();

        for i in 0..1000 {
            orders.push(TrackedOrder::new(
                i,
                "BTCUSDT",
                50000.0 + i as f64,
                0.01,
            ));
        }

        let (active, total) = get_order_stats();
        println!("После создания 1000 ордеров:");
        println!("  Активных: {}", active);
        println!("  Всего создано: {}", total);

        // Удаляем половину ордеров
        orders.drain(..500);

        let (active, total) = get_order_stats();
        println!("\nПосле удаления 500 ордеров:");
        println!("  Активных: {}", active);
        println!("  Всего создано: {}", total);
    }

    let (active, total) = get_order_stats();
    println!("\nПосле выхода из блока:");
    println!("  Активных: {}", active);
    println!("  Всего создано: {}", total);

    if active != 0 {
        println!("\n!!! УТЕЧКА ПАМЯТИ: {} ордеров не освобождены!", active);
    } else {
        println!("\nВсе ордера корректно освобождены!");
    }
}
```

## Продвинутые техники отслеживания

### Профилирование аллокаций с dhat

```toml
# Cargo.toml
[dependencies]
dhat = "0.3"

[features]
dhat-heap = []
```

```rust
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Код торговой системы
    let mut prices: Vec<f64> = Vec::new();

    for i in 0..100_000 {
        prices.push(50000.0 + i as f64 * 0.01);
    }

    // При завершении dhat выведет отчёт
}
```

Запуск:
```bash
cargo run --release --features dhat-heap
```

### Интеграция с метриками Prometheus

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MemoryMetrics {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
    high_water_mark: AtomicUsize,
}

impl MemoryMetrics {
    fn new() -> Self {
        MemoryMetrics {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            high_water_mark: AtomicUsize::new(0),
        }
    }

    fn record_alloc(&self, bytes: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        let new_total = self.bytes_allocated.fetch_add(bytes, Ordering::Relaxed) + bytes;
        let current_in_use = new_total - self.bytes_deallocated.load(Ordering::Relaxed);

        // Обновляем high water mark
        let mut current_hwm = self.high_water_mark.load(Ordering::Relaxed);
        while current_in_use > current_hwm {
            match self.high_water_mark.compare_exchange_weak(
                current_hwm,
                current_in_use,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new) => current_hwm = new,
            }
        }
    }

    fn record_dealloc(&self, bytes: usize) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_deallocated.fetch_add(bytes, Ordering::Relaxed);
    }

    fn report(&self) {
        let allocs = self.allocations.load(Ordering::Relaxed);
        let deallocs = self.deallocations.load(Ordering::Relaxed);
        let bytes_alloc = self.bytes_allocated.load(Ordering::Relaxed);
        let bytes_dealloc = self.bytes_deallocated.load(Ordering::Relaxed);
        let hwm = self.high_water_mark.load(Ordering::Relaxed);

        println!("=== Отчёт по памяти ===");
        println!("Всего аллокаций: {}", allocs);
        println!("Всего деаллокаций: {}", deallocs);
        println!("Незавершённые аллокации: {}", allocs - deallocs);
        println!("Байт выделено: {} KB", bytes_alloc / 1024);
        println!("Байт освобождено: {} KB", bytes_dealloc / 1024);
        println!("Текущее использование: {} KB", (bytes_alloc - bytes_dealloc) / 1024);
        println!("Пиковое использование: {} KB", hwm / 1024);
    }
}

fn main() {
    let metrics = Arc::new(MemoryMetrics::new());

    // Симуляция работы торговой системы
    println!("Запуск симуляции...\n");

    // Имитируем аллокации
    for i in 0..1000 {
        let size = 1024 * ((i % 10) + 1);  // От 1KB до 10KB
        metrics.record_alloc(size);

        // Освобождаем 80% аллокаций
        if i % 5 != 0 {
            metrics.record_dealloc(size);
        }
    }

    metrics.report();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Утечка памяти** | Выделенная память, которая никогда не освобождается |
| **Циклические ссылки** | Rc/Arc образующие цикл, предотвращающий освобождение |
| **Weak** | Слабая ссылка, не влияющая на счётчик ссылок |
| **Valgrind** | Внешний инструмент для обнаружения утечек |
| **dhat** | Встроенный профайлер аллокаций для Rust |
| **RAII** | Паттерн автоматического управления ресурсами через Drop |
| **Bounded cache** | Кэш с ограничением размера для предотвращения роста |
| **Memory metrics** | Сбор статистики использования памяти |

## Практические задания

1. **Детектор утечек**: Создай обёртку для `Box<T>`, которая:
   - Отслеживает все аллокации и деаллокации
   - Хранит stack trace места создания
   - При завершении программы выводит список не освобождённых объектов
   - Показывает где они были созданы

2. **Анализатор циклов**: Напиши утилиту, которая:
   - Строит граф связей между `Rc<T>` объектами
   - Обнаруживает циклические зависимости
   - Предлагает замену на `Weak<T>` где необходимо
   - Генерирует визуализацию графа

3. **Мониторинг реального времени**: Реализуй систему мониторинга:
   - Отслеживает использование памяти каждую секунду
   - Строит график роста/падения
   - Предупреждает при аномальном росте
   - Интегрируется с торговыми метриками

4. **Ограниченная книга ордеров**: Создай OrderBook с:
   - Автоматической очисткой старых ордеров
   - Ограничением на количество ордеров на символ
   - Метриками использования памяти
   - Тестами на отсутствие утечек

## Домашнее задание

1. **Memory Profiler для торгового бота**: Разработай профилировщик, который:
   - Встраивается в существующий торговый бот
   - Отслеживает аллокации по категориям (ордера, цены, индикаторы)
   - Генерирует отчёты каждые N секунд
   - Записывает историю использования памяти
   - Предупреждает о потенциальных утечках

2. **Stress-test на утечки**: Напиши тестовый фреймворк:
   - Запускает торгового бота с разными нагрузками
   - Мониторит память в течение длительного времени
   - Определяет есть ли постоянный рост
   - Генерирует отчёт с графиками
   - Автоматически определяет утечки

3. **Ограниченный пул объектов**: Реализуй пул:
   - Переиспользует объекты вместо постоянных аллокаций
   - Имеет ограничение на максимальный размер
   - Автоматически сжимается при низкой нагрузке
   - Собирает метрики эффективности
   - Безопасен для многопоточного использования

4. **CI интеграция для проверки утечек**: Создай pipeline:
   - Запускает тесты с Valgrind/ASAN
   - Проверяет отсутствие утечек
   - Блокирует merge при обнаружении проблем
   - Генерирует отчёты для code review
   - Хранит историю проверок

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../320-*/ru.md)
