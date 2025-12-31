# День 320: Valgrind и Heaptrack

## Аналогия из трейдинга

Представь, что ты управляешь хедж-фондом с миллиардными активами. Твоя торговая система обрабатывает тысячи ордеров в секунду. Однажды ты замечаешь, что система замедляется — сначала незаметно, потом критично. Потребление памяти растёт, пока система не падает в самый разгар торгов.

Это похоже на склад с тысячами торговых позиций:
- **Утечка памяти** — коробки (аллокации) приходят, но никогда не уходят. Склад заполняется до отказа
- **Избыточные аллокации** — рабочие постоянно перекладывают коробки туда-сюда, тратя время на логистику вместо торговли
- **Фрагментация памяти** — хранение мелких предметов в больших коробках, разбросанных случайно, затрудняет поиск места для новых

**Valgrind** и **Heaptrack** — это аудиторы склада:
- **Valgrind** — тщательный инспектор, отслеживающий каждое перемещение коробки, находящий утечки и обнаруживающий некорректный доступ
- **Heaptrack** — быстрый аналитик, предоставляющий статистику аллокаций и определяющий узкие места

В продакшн торговых системах проблемы с памятью могут вызвать:
- **Проскальзывание**: Медленные операции с памятью означают задержку исполнения ордеров
- **Падения системы**: Нехватка памяти во время волатильного рынка = упущенные сделки
- **Непредсказуемые задержки**: Паузы типа GC от фрагментации влияют на тайминг

## Что такое Valgrind?

**Valgrind** — это мощный фреймворк инструментирования для отладки памяти. Его основной инструмент, **Memcheck**, обнаруживает:

| Проблема | Описание | Влияние на трейдинг |
|----------|----------|---------------------|
| **Утечки памяти** | Выделенная память не освобождается | Падения системы при длительной работе |
| **Некорректное чтение/запись** | Доступ к освобождённой или неинициализированной памяти | Повреждение данных, неправильные цены |
| **Использование после освобождения** | Использование памяти после деаллокации | Непредсказуемое поведение |
| **Двойное освобождение** | Освобождение одной и той же памяти дважды | Падения |
| **Некорректный доступ к памяти** | Переполнение буфера, разыменование null-указателя | Уязвимости безопасности |

### Установка Valgrind

```bash
# Ubuntu/Debian
sudo apt-get install valgrind

# Fedora
sudo dnf install valgrind

# macOS (только Intel, не поддерживается на Apple Silicon)
brew install valgrind
```

## Что такое Heaptrack?

**Heaptrack** — это современный профилировщик кучи. В отличие от Valgrind, он фокусируется на:

| Возможность | Описание |
|-------------|----------|
| **Скорость** | Намного быстрее Valgrind (в 10-100 раз) |
| **Отслеживание аллокаций** | Количество и размеры всех аллокаций |
| **Стеки вызовов** | Какие функции выделяют больше всего |
| **Flame-графики** | Визуальное представление узких мест |
| **Обнаружение утечек** | Определение памяти, которая никогда не освобождается |

### Установка Heaptrack

```bash
# Ubuntu/Debian
sudo apt-get install heaptrack heaptrack-gui

# Fedora
sudo dnf install heaptrack

# Сборка из исходников (если нет в репозиториях)
git clone https://github.com/KDE/heaptrack.git
cd heaptrack && mkdir build && cd build
cmake .. && make && sudo make install
```

## Базовое использование с Rust

### Создание тестового проекта

Сначала создадим торговую систему с намеренными проблемами памяти:

```rust
// src/main.rs
use std::collections::HashMap;
use std::time::Instant;

/// Торговый ордер с накладными расходами на аллокацию
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,      // Аллокация в куче
    price: f64,
    quantity: f64,
    timestamp: u64,
    metadata: HashMap<String, String>,  // Дополнительные аллокации
}

impl Order {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "api".to_string());
        metadata.insert("status".to_string(), "pending".to_string());

        Order {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        }
    }
}

/// Книга ордеров с потенциальными проблемами памяти
struct OrderBook {
    orders: Vec<Order>,
    history: Vec<Order>,  // Потенциальная утечка, если не управлять
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: Vec::new(),
            history: Vec::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.push(order.clone());
        // БАГ: История растёт бесконтрольно — паттерн утечки памяти
        self.history.push(order);
    }

    fn execute_order(&mut self, id: u64) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            let order = self.orders.remove(pos);
            // История продолжает расти даже после исполнения
            self.history.push(order.clone());
            Some(order)
        } else {
            None
        }
    }

    fn memory_stats(&self) {
        println!("Активных ордеров: {}", self.orders.len());
        println!("Размер истории: {}", self.history.len());
        println!("Примерный объём истории: {} КБ",
            self.history.len() * std::mem::size_of::<Order>() / 1024);
    }
}

/// Симулирует высокочастотную торговлю с аллокациями
fn simulate_trading(iterations: usize) {
    let mut order_book = OrderBook::new();
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT"];

    println!("Запуск симуляции торговли с {} итерациями", iterations);
    let start = Instant::now();

    for i in 0..iterations {
        // Создаём новый ордер (выделяем память)
        let symbol = symbols[i % symbols.len()];
        let order = Order::new(
            i as u64,
            symbol,
            50000.0 + (i as f64 * 0.01),
            0.1 + (i as f64 * 0.001),
        );

        order_book.add_order(order);

        // Исполняем некоторые ордера (но история продолжает расти)
        if i > 0 && i % 10 == 0 {
            order_book.execute_order((i - 5) as u64);
        }

        // Периодически выводим статистику
        if i > 0 && i % 10000 == 0 {
            order_book.memory_stats();
        }
    }

    let duration = start.elapsed();
    println!("\nСимуляция завершена за {:?}", duration);
    order_book.memory_stats();
}

fn main() {
    // Меньше итераций для Valgrind (он медленный)
    // Увеличить для реалистичного профилирования с Heaptrack
    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);

    simulate_trading(iterations);
}
```

### Компиляция для профилирования

```bash
# Сборка с отладочными символами для лучших стек-трейсов
# Используем release для реалистичной производительности, но сохраняем отладочную информацию
cargo build --release

# Или с максимальной отладочной информацией
RUSTFLAGS="-g" cargo build --release
```

## Использование Valgrind с Rust

### Запуск Memcheck

```bash
# Базовая проверка памяти
valgrind --leak-check=full ./target/release/trading_profiler

# Детальный анализ утечек
valgrind --leak-check=full --show-leak-kinds=all \
         --track-origins=yes ./target/release/trading_profiler

# С уменьшенным количеством итераций (Valgrind в ~20 раз медленнее)
ITERATIONS=5000 valgrind --leak-check=full ./target/release/trading_profiler
```

### Понимание вывода Valgrind

```
==12345== HEAP SUMMARY:
==12345==     in use at exit: 1,234,567 bytes in 8,901 blocks
==12345==   total heap usage: 123,456 allocs, 114,555 frees, 12,345,678 bytes allocated
==12345==
==12345== 567,890 bytes in 1,234 blocks are definitely lost in loss record 42 of 50
==12345==    at 0x4C2FB0F: malloc (vg_replace_malloc.c:299)
==12345==    by 0x55AABCD: alloc::alloc::alloc (alloc.rs:81)
==12345==    by 0x55AACDE: <alloc::vec::Vec<T>>::push (vec.rs:1234)
==12345==    by 0x123456: trading_profiler::OrderBook::add_order (main.rs:45)
==12345==    by 0x234567: trading_profiler::simulate_trading (main.rs:78)
```

Ключевые метрики:
- **definitely lost**: Утечка памяти (нет указателей на неё)
- **indirectly lost**: Память, достижимая только через утечку
- **possibly lost**: Память с внутренними указателями (обычно ложные срабатывания в Rust)
- **still reachable**: Память, достижимая при выходе (часто намеренно)

## Использование Heaptrack с Rust

### Запуск Heaptrack

```bash
# Профилируем приложение
heaptrack ./target/release/trading_profiler

# С большим количеством итераций (Heaptrack намного быстрее)
ITERATIONS=100000 heaptrack ./target/release/trading_profiler

# Вывод: heaptrack.trading_profiler.12345.gz
```

### Анализ результатов

```bash
# Текстовый анализ
heaptrack_print heaptrack.trading_profiler.12345.gz

# GUI анализ (если доступен)
heaptrack_gui heaptrack.trading_profiler.12345.gz
```

### Понимание вывода Heaptrack

```
SUMMARY
=======
Total runtime: 2.5s
Total memory allocated: 156.78 MB
Peak heap memory: 45.23 MB
Peak RSS: 67.89 MB
Total allocations: 234,567
Calls to malloc: 234,567

MOST MEMORY ALLOCATED
=====================
  1. 89.45 MB from 123,456 allocations
     alloc::vec::Vec<T>::push
       at /rustc/xxx/library/alloc/src/vec/mod.rs:1234
     trading_profiler::OrderBook::add_order
       at src/main.rs:45

  2. 34.56 MB from 67,890 allocations
     alloc::string::String::from
       at /rustc/xxx/library/alloc/src/string.rs:567
     trading_profiler::Order::new
       at src/main.rs:23

LEAKED MEMORY
=============
  Total: 23.45 MB

  1. 18.90 MB leaked from:
     trading_profiler::OrderBook::history
       at src/main.rs:38
```

## Практический пример: Оптимизация обработчика ценового потока

Создадим более реалистичный пример с оптимизацией памяти:

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

/// Тик цены из рыночного потока
#[derive(Debug, Clone)]
struct PriceTick {
    symbol: Arc<str>,  // Интернированный символ — общий для всех тиков
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// НЕЭФФЕКТИВНО: Аллокация String на каждый тик
#[derive(Debug, Clone)]
struct PriceTickBad {
    symbol: String,  // Аллоцируется каждый раз
    bid: f64,
    ask: f64,
    timestamp: u64,
}

/// Агрегатор цен с ограниченным потреблением памяти
struct PriceAggregator {
    // Кольцевой буфер — ограниченное использование памяти
    prices: HashMap<Arc<str>, VecDeque<f64>>,
    max_history: usize,
    symbol_cache: HashMap<String, Arc<str>>,
}

impl PriceAggregator {
    fn new(max_history: usize) -> Self {
        PriceAggregator {
            prices: HashMap::new(),
            max_history,
            symbol_cache: HashMap::new(),
        }
    }

    /// Интернирование строки символа (аллокация один раз, повторное использование везде)
    fn intern_symbol(&mut self, symbol: &str) -> Arc<str> {
        if let Some(cached) = self.symbol_cache.get(symbol) {
            return Arc::clone(cached);
        }
        let interned: Arc<str> = Arc::from(symbol);
        self.symbol_cache.insert(symbol.to_string(), Arc::clone(&interned));
        interned
    }

    /// Добавление цены с ограниченной памятью
    fn add_price(&mut self, symbol: Arc<str>, price: f64) {
        let history = self.prices
            .entry(symbol)
            .or_insert_with(|| VecDeque::with_capacity(self.max_history));

        // Ограниченный буфер — старые цены удаляются
        if history.len() >= self.max_history {
            history.pop_front();  // Удаляем самую старую
        }
        history.push_back(price);
    }

    /// Расчёт скользящего среднего
    fn get_sma(&self, symbol: &Arc<str>, period: usize) -> Option<f64> {
        self.prices.get(symbol).map(|history| {
            let count = history.len().min(period);
            if count == 0 {
                return 0.0;
            }
            history.iter().rev().take(count).sum::<f64>() / count as f64
        })
    }

    fn memory_stats(&self) {
        let total_prices: usize = self.prices.values().map(|v| v.len()).sum();
        let symbols = self.prices.len();
        println!("Отслеживаемых символов: {}", symbols);
        println!("Всего цен в памяти: {}", total_prices);
        println!("Размер кеша символов: {}", self.symbol_cache.len());
    }
}

/// НЕЭФФЕКТИВНАЯ версия для сравнения
struct PriceAggregatorBad {
    prices: HashMap<String, Vec<f64>>,  // Неограниченный + String ключи
}

impl PriceAggregatorBad {
    fn new() -> Self {
        PriceAggregatorBad {
            prices: HashMap::new(),
        }
    }

    fn add_price(&mut self, symbol: &str, price: f64) {
        // Аллоцирует String при каждом поиске!
        self.prices
            .entry(symbol.to_string())  // Аллокация!
            .or_insert_with(Vec::new)
            .push(price);  // Неограниченный рост
    }
}

fn benchmark_optimized(iterations: usize) -> std::time::Duration {
    let mut aggregator = PriceAggregator::new(1000);  // Ограничено 1000 ценами
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];

    let start = Instant::now();

    for i in 0..iterations {
        let symbol_str = symbols[i % symbols.len()];
        let symbol = aggregator.intern_symbol(symbol_str);
        let price = 50000.0 + (i as f64 * 0.001);

        aggregator.add_price(symbol.clone(), price);

        // Периодически вычисляем SMA
        if i % 100 == 0 {
            let _ = aggregator.get_sma(&symbol, 20);
        }
    }

    let duration = start.elapsed();
    println!("\nОптимизированная версия:");
    aggregator.memory_stats();
    duration
}

fn benchmark_inefficient(iterations: usize) -> std::time::Duration {
    let mut aggregator = PriceAggregatorBad::new();
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT"];

    let start = Instant::now();

    for i in 0..iterations {
        let symbol = symbols[i % symbols.len()];
        let price = 50000.0 + (i as f64 * 0.001);

        aggregator.add_price(symbol, price);
    }

    let duration = start.elapsed();
    let total_prices: usize = aggregator.prices.values().map(|v| v.len()).sum();
    println!("\nНеэффективная версия:");
    println!("Всего цен в памяти: {}", total_prices);
    duration
}

fn main() {
    let iterations = std::env::var("ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    println!("=== Сравнение профилирования памяти ===");
    println!("Итераций: {}\n", iterations);

    let optimized_time = benchmark_optimized(iterations);
    let inefficient_time = benchmark_inefficient(iterations);

    println!("\n=== Результаты производительности ===");
    println!("Оптимизированная:   {:?}", optimized_time);
    println!("Неэффективная:      {:?}", inefficient_time);
    println!("Ускорение:          {:.2}x",
        inefficient_time.as_nanos() as f64 / optimized_time.as_nanos() as f64);
}
```

### Запуск сравнения с Heaptrack

```bash
# Сборка
cargo build --release

# Профилируем оптимизированную версию
ITERATIONS=500000 heaptrack ./target/release/trading_profiler

# Сравниваем аллокации
heaptrack_print heaptrack.trading_profiler.*.gz | head -50
```

## Обнаружение утечек памяти в торговых системах

### Типичные паттерны утечек

```rust
use std::collections::HashMap;

/// Паттерн 1: Неограниченный кеш
struct LeakyCache {
    cache: HashMap<String, Vec<f64>>,  // Никогда не вытесняет старые записи
}

impl LeakyCache {
    fn add(&mut self, key: &str, value: f64) {
        self.cache
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(value);
        // БАГ: Кеш растёт бесконечно
    }
}

/// Паттерн 2: История событий без очистки
struct LeakyEventLog {
    events: Vec<String>,  // Никогда не очищается
}

impl LeakyEventLog {
    fn log(&mut self, event: &str) {
        self.events.push(format!("[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            event
        ));
        // БАГ: События накапливаются бесконечно
    }
}

/// Паттерн 3: Циклические ссылки (редко в Rust, но возможно с Rc)
use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    value: i32,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,  // Создаёт цикл!
}

/// ИСПРАВЛЕННЫЕ версии

/// Исправленный паттерн 1: LRU кеш с вытеснением
use std::collections::BTreeMap;

struct BoundedCache {
    cache: HashMap<String, f64>,
    order: BTreeMap<u64, String>,  // Для LRU вытеснения
    max_size: usize,
    counter: u64,
}

impl BoundedCache {
    fn new(max_size: usize) -> Self {
        BoundedCache {
            cache: HashMap::with_capacity(max_size),
            order: BTreeMap::new(),
            max_size,
            counter: 0,
        }
    }

    fn add(&mut self, key: &str, value: f64) {
        // Вытесняем самый старый при достижении лимита
        while self.cache.len() >= self.max_size {
            if let Some((&oldest_time, _)) = self.order.iter().next() {
                if let Some(oldest_key) = self.order.remove(&oldest_time) {
                    self.cache.remove(&oldest_key);
                }
            }
        }

        self.counter += 1;
        self.cache.insert(key.to_string(), value);
        self.order.insert(self.counter, key.to_string());
    }
}

/// Исправленный паттерн 2: Скользящий лог событий
struct RollingEventLog {
    events: std::collections::VecDeque<String>,
    max_events: usize,
}

impl RollingEventLog {
    fn new(max_events: usize) -> Self {
        RollingEventLog {
            events: std::collections::VecDeque::with_capacity(max_events),
            max_events,
        }
    }

    fn log(&mut self, event: &str) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();  // Удаляем самое старое
        }
        self.events.push_back(format!("[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            event
        ));
    }
}

/// Исправленный паттерн 3: Использование Weak для обратных ссылок
use std::rc::Weak;

struct SafeNode {
    value: i32,
    next: Option<Rc<RefCell<SafeNode>>>,
    prev: Weak<RefCell<SafeNode>>,  // Weak разрывает цикл
}
```

## Файлы подавления Valgrind

Иногда Valgrind сообщает о ложных срабатываниях. Создайте файлы подавления для известных безопасных паттернов:

```
# rust.supp - файл подавления Valgrind для Rust
{
   rust_thread_local
   Memcheck:Leak
   match-leak-kinds: possible
   fun:malloc
   ...
   fun:*thread_local*
}

{
   rust_backtrace
   Memcheck:Leak
   match-leak-kinds: reachable
   ...
   fun:*backtrace*
}
```

Использование:
```bash
valgrind --suppressions=rust.supp --leak-check=full ./target/release/app
```

## Интеграция профилирования памяти в CI/CD

### Пример GitHub Actions

```yaml
name: Memory Profiling

on:
  push:
    branches: [main]
  pull_request:

jobs:
  memory-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Valgrind
        run: sudo apt-get install -y valgrind

      - name: Build
        run: cargo build --release

      - name: Run Valgrind
        run: |
          ITERATIONS=5000 valgrind --leak-check=full \
            --error-exitcode=1 \
            ./target/release/trading_profiler 2>&1 | tee valgrind.log

      - name: Check for leaks
        run: |
          if grep -q "definitely lost:" valgrind.log; then
            echo "Обнаружены утечки памяти!"
            grep -A5 "definitely lost:" valgrind.log
            exit 1
          fi

      - name: Upload report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: valgrind-report
          path: valgrind.log
```

## Скрипт сравнения производительности

```rust
use std::process::Command;
use std::fs;

fn run_with_heaptrack(binary: &str, iterations: usize) -> String {
    let output = Command::new("heaptrack")
        .arg(binary)
        .env("ITERATIONS", iterations.to_string())
        .output()
        .expect("Не удалось запустить heaptrack");

    String::from_utf8_lossy(&output.stderr).to_string()
}

fn parse_heaptrack_summary(output_file: &str) -> HeaptrackStats {
    let output = Command::new("heaptrack_print")
        .arg(output_file)
        .output()
        .expect("Не удалось обработать вывод heaptrack");

    let text = String::from_utf8_lossy(&output.stdout);

    // Парсим ключевые метрики
    HeaptrackStats {
        total_allocated: parse_bytes(&text, "Total memory allocated"),
        peak_memory: parse_bytes(&text, "Peak heap memory"),
        total_allocations: parse_count(&text, "Total allocations"),
    }
}

#[derive(Debug)]
struct HeaptrackStats {
    total_allocated: u64,
    peak_memory: u64,
    total_allocations: u64,
}

fn parse_bytes(text: &str, pattern: &str) -> u64 {
    // Реализация парсинга байтовых значений
    0
}

fn parse_count(text: &str, pattern: &str) -> u64 {
    // Реализация парсинга количества
    0
}

fn main() {
    println!("=== Автоматическое профилирование памяти ===\n");

    // Запускаем профилирование
    let stats = run_with_heaptrack("./target/release/trading_app", 100_000);

    println!("Профилирование завершено!");
    println!("{}", stats);
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| **Valgrind** | Инструмент отладки памяти, обнаруживающий утечки, некорректный доступ и use-after-free |
| **Heaptrack** | Быстрый профилировщик кучи, показывающий паттерны аллокаций и узкие места |
| **Утечка памяти** | Память, которая выделена, но никогда не освобождается, вызывая постепенное исчерпание ресурсов |
| **Ограниченные буферы** | Использование VecDeque с максимальной ёмкостью для предотвращения неограниченного роста |
| **Интернирование строк** | Разделение строковых данных через Arc<str> для уменьшения аллокаций |
| **Файлы подавления** | Фильтрация известных ложных срабатываний Valgrind |
| **Интеграция в CI** | Автоматическая проверка памяти в конвейерах сборки |

## Практические упражнения

1. **Найди утечку**: Создай менеджер торговых ордеров, который намеренно пропускает память. Используй Valgrind для определения утечки, затем исправь её с помощью ограниченных структур данных.

2. **Аудит аллокаций**: Профилируй существующее торговое приложение с помощью Heaptrack. Определи 3 основных узких места по аллокациям и предложи оптимизации.

3. **Бюджет памяти**: Реализуй агрегатор цен, работающий в рамках фиксированного бюджета памяти (например, 100 МБ). Используй Heaptrack для проверки соблюдения лимита под нагрузкой.

4. **CI конвейер**: Настрой GitHub Actions workflow, запускающий Valgrind на каждый PR и завершающийся с ошибкой при обнаружении утечек памяти.

## Домашнее задание

1. **Менеджер истории сделок**: Построй систему, которая:
   - Записывает все сделки с метками времени
   - Использует скользящее окно (хранит последние N сделок)
   - Профилируй с помощью Valgrind и Heaptrack
   - Сравни использование памяти с неограниченной версией
   - Сгенерируй отчёт, показывающий экономию памяти

2. **Оптимизатор памяти книги ордеров**: Создай книгу ордеров, которая:
   - Обрабатывает 1 миллион ордеров за запуск
   - Использует интернирование строк для символов
   - Реализует ограниченную историю ордеров
   - Профилируй паттерны памяти с Heaptrack
   - Достигни пикового использования памяти < 50 МБ

3. **Набор для обнаружения утечек**: Напиши тестовую обвязку, которая:
   - Запускает множество тестовых сценариев
   - Использует Valgrind для проверки каждого на утечки
   - Генерирует HTML-отчёт с результатами
   - Интегрируется с cargo test
   - Поддерживает интеграцию в CI/CD

4. **Регрессионные тесты памяти**: Реализуй систему, которая:
   - Записывает базовое использование памяти
   - Сравнивает новые коммиты с базовой линией
   - Предупреждает, если использование памяти увеличивается более чем на 10%
   - Использует Heaptrack для измерений
   - Запускается автоматически при PR

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../321-*/ru.md)
