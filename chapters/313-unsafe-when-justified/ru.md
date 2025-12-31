# День 313: unsafe: когда оправдано

## Аналогия из трейдинга

Представь автоматическую торговую систему с двумя режимами работы:

**Безопасный режим (safe):**
- Все сделки проходят через систему проверок
- Проверяются лимиты позиций
- Валидируются ордера перед отправкой
- Контролируется риск на каждый шаг
- Медленнее, но гарантированно безопасно

**Режим прямого доступа (unsafe):**
- Прямое взаимодействие с биржевым API без промежуточных проверок
- Ордера отправляются напрямую в очередь исполнения
- Нет дополнительных валидаций — максимальная скорость
- Используется в высокочастотном трейдинге (HFT)
- Одна ошибка = потеря денег

В Rust `unsafe` — это именно такой режим прямого доступа. Компилятор доверяет программисту, что он знает, что делает, и снимает некоторые проверки безопасности. Это как снятие страховочного троса у скалолаза — даёт свободу движений, но требует абсолютной уверенности в каждом шаге.

## Что такое unsafe в Rust?

Rust гарантирует безопасность памяти на этапе компиляции через систему владения, проверку времён жизни и правила заимствования. Но иногда нужно сделать то, что компилятор не может проверить:

### Пять суперспособностей unsafe:

1. **Разыменование сырых указателей** (`*const T`, `*mut T`)
2. **Вызов unsafe функций и методов**
3. **Доступ к изменяемым статическим переменным**
4. **Реализация unsafe трейтов**
5. **Доступ к полям объединений (union)**

```rust
fn main() {
    let x = 42;
    let raw_ptr = &x as *const i32;

    // Безопасно: создание сырого указателя
    println!("Создали указатель: {:p}", raw_ptr);

    // НЕБЕЗОПАСНО: разыменование сырого указателя
    unsafe {
        println!("Значение по указателю: {}", *raw_ptr);
    }
}
```

## Когда unsafe оправдан?

### 1. Взаимодействие с FFI (Foreign Function Interface)

При работе с внешними библиотеками на C/C++ для высокопроизводительных вычислений:

```rust
// Подключение библиотеки для технического анализа на C
#[link(name = "ta_lib")]
extern "C" {
    // Расчёт RSI (Relative Strength Index)
    fn TA_RSI(
        start_idx: i32,
        end_idx: i32,
        in_real: *const f64,    // Входные цены
        opt_period: i32,         // Период RSI
        out_begin: *mut i32,     // Индекс начала выходных данных
        out_nb_element: *mut i32, // Количество выходных элементов
        out_real: *mut f64,      // Выходные значения RSI
    ) -> i32;
}

fn calculate_rsi_safe(prices: &[f64], period: usize) -> Vec<f64> {
    let mut out_begin: i32 = 0;
    let mut out_nb_element: i32 = 0;
    let mut output = vec![0.0; prices.len()];

    unsafe {
        let result = TA_RSI(
            0,
            (prices.len() - 1) as i32,
            prices.as_ptr(),
            period as i32,
            &mut out_begin,
            &mut out_nb_element,
            output.as_mut_ptr(),
        );

        if result != 0 {
            panic!("Ошибка расчёта RSI: код {}", result);
        }
    }

    // Обрезаем вектор до реального размера выходных данных
    output.truncate(out_nb_element as usize);
    output
}

fn main() {
    let prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33,
        44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28,
    ];

    let rsi = calculate_rsi_safe(&prices, 14);
    println!("RSI значения: {:?}", rsi);
}
```

### 2. Оптимизация критических участков кода

Высокочастотный трейдинг требует микросекундной точности. Иногда `unsafe` позволяет избежать проверок границ массива:

```rust
#[derive(Debug, Clone)]
struct OrderBookLevel {
    price: f64,
    volume: f64,
}

struct FastOrderBook {
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
}

impl FastOrderBook {
    /// БЕЗОПАСНАЯ версия: с проверкой границ
    fn get_best_bid_safe(&self) -> Option<&OrderBookLevel> {
        self.bids.first()
    }

    /// UNSAFE версия: без проверки границ для максимальной скорости
    /// ТРЕБОВАНИЕ: гарантируем, что bids не пуст через инвариант структуры
    unsafe fn get_best_bid_unchecked(&self) -> &OrderBookLevel {
        // Избегаем проверки границ массива
        self.bids.get_unchecked(0)
    }

    /// Расчёт спреда с использованием unsafe для скорости
    fn calculate_spread_fast(&self) -> f64 {
        // Инвариант: OrderBook всегда содержит минимум 1 bid и 1 ask
        unsafe {
            let best_ask = self.asks.get_unchecked(0).price;
            let best_bid = self.bids.get_unchecked(0).price;
            best_ask - best_bid
        }
    }
}

fn main() {
    let order_book = FastOrderBook {
        bids: vec![
            OrderBookLevel { price: 42150.50, volume: 1.5 },
            OrderBookLevel { price: 42150.00, volume: 2.3 },
        ],
        asks: vec![
            OrderBookLevel { price: 42151.00, volume: 0.8 },
            OrderBookLevel { price: 42151.50, volume: 1.2 },
        ],
    };

    println!("Спред: {:.2}", order_book.calculate_spread_fast());
}
```

⚠️ **ВАЖНО:** В реальном коде нужно гарантировать инвариант (непустые bids/asks) через конструктор и другие методы!

### 3. Реализация низкоуровневых структур данных

Lock-free структуры данных для многопоточной обработки рыночных данных:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;

#[derive(Debug)]
struct Trade {
    price: f64,
    volume: f64,
    timestamp: u64,
}

/// Lock-free стек для хранения последних сделок
/// Используется в high-frequency системах для минимизации задержек
struct LockFreeTradeStack {
    head: AtomicPtr<Node>,
    len: AtomicUsize,
}

struct Node {
    trade: Trade,
    next: *mut Node,
}

impl LockFreeTradeStack {
    fn new() -> Self {
        LockFreeTradeStack {
            head: AtomicPtr::new(ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    /// Добавление сделки в стек (lock-free)
    fn push(&self, trade: Trade) {
        unsafe {
            // Создаём новый узел в куче
            let new_node = Box::into_raw(Box::new(Node {
                trade,
                next: ptr::null_mut(),
            }));

            loop {
                // Читаем текущую голову
                let old_head = self.head.load(Ordering::Acquire);

                // Устанавливаем next нового узла на старую голову
                (*new_node).next = old_head;

                // Пытаемся атомарно заменить голову
                if self.head.compare_exchange(
                    old_head,
                    new_node,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    self.len.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                // Если не получилось — повторяем (кто-то другой изменил голову)
            }
        }
    }

    /// Извлечение последней сделки (lock-free)
    fn pop(&self) -> Option<Trade> {
        unsafe {
            loop {
                let old_head = self.head.load(Ordering::Acquire);

                if old_head.is_null() {
                    return None;
                }

                let next = (*old_head).next;

                // Пытаемся атомарно заменить голову на следующий элемент
                if self.head.compare_exchange(
                    old_head,
                    next,
                    Ordering::Release,
                    Ordering::Acquire,
                ).is_ok() {
                    self.len.fetch_sub(1, Ordering::Relaxed);

                    // Извлекаем trade и освобождаем память узла
                    let boxed_node = Box::from_raw(old_head);
                    return Some(boxed_node.trade);
                }
                // Если не получилось — повторяем
            }
        }
    }

    fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

impl Drop for LockFreeTradeStack {
    fn drop(&mut self) {
        // Очищаем все узлы при удалении стека
        while self.pop().is_some() {}
    }
}

fn main() {
    let stack = LockFreeTradeStack::new();

    // Добавляем сделки
    stack.push(Trade { price: 42100.0, volume: 0.5, timestamp: 1000 });
    stack.push(Trade { price: 42105.0, volume: 1.2, timestamp: 1001 });
    stack.push(Trade { price: 42103.0, volume: 0.8, timestamp: 1002 });

    println!("Количество сделок в стеке: {}", stack.len());

    // Извлекаем сделки (в обратном порядке — стек LIFO)
    while let Some(trade) = stack.pop() {
        println!("Trade: ${:.2}, Vol: {:.2}, Time: {}",
            trade.price, trade.volume, trade.timestamp);
    }
}
```

### 4. Работа с изменяемыми глобальными переменными

Кэширование конфигурации для быстрого доступа из разных потоков:

```rust
use std::sync::Mutex;

/// Глобальная конфигурация торговой системы
struct TradingConfig {
    max_position_size: f64,
    max_order_value: f64,
    risk_limit_percent: f64,
}

// Статическая изменяемая переменная (требует unsafe для доступа)
static mut TRADING_CONFIG: Option<TradingConfig> = None;
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

/// Инициализация конфигурации (вызывается один раз при старте)
fn init_config(config: TradingConfig) {
    let _lock = CONFIG_LOCK.lock().unwrap();
    unsafe {
        TRADING_CONFIG = Some(config);
    }
}

/// Безопасное чтение конфигурации
fn get_max_position() -> f64 {
    unsafe {
        TRADING_CONFIG
            .as_ref()
            .map(|c| c.max_position_size)
            .unwrap_or(0.0)
    }
}

fn main() {
    // Инициализируем конфигурацию при старте
    init_config(TradingConfig {
        max_position_size: 100_000.0,
        max_order_value: 50_000.0,
        risk_limit_percent: 2.0,
    });

    // Используем конфигурацию
    println!("Максимальный размер позиции: ${:.2}", get_max_position());
}
```

**Лучшая альтернатива без unsafe:**

```rust
use std::sync::OnceLock;

static TRADING_CONFIG: OnceLock<TradingConfig> = OnceLock::new();

fn init_config(config: TradingConfig) {
    TRADING_CONFIG.set(config).expect("Конфигурация уже инициализирована");
}

fn get_max_position() -> f64 {
    TRADING_CONFIG
        .get()
        .map(|c| c.max_position_size)
        .unwrap_or(0.0)
}
```

## Правила безопасного использования unsafe

### 1. Минимизируй unsafe блоки

```rust
// ❌ ПЛОХО: большой unsafe блок
unsafe {
    let ptr = data.as_ptr();
    let value = *ptr;
    process_value(value);
    another_safe_operation();
    yet_another_safe_operation();
}

// ✅ ХОРОШО: только критическая операция в unsafe
let value = unsafe { *data.as_ptr() };
process_value(value);
another_safe_operation();
yet_another_safe_operation();
```

### 2. Оборачивай unsafe в безопасный API

```rust
/// Небезопасная функция для прямого доступа к памяти
unsafe fn raw_memory_access(ptr: *const f64, len: usize) -> f64 {
    let mut sum = 0.0;
    for i in 0..len {
        sum += *ptr.add(i);
    }
    sum
}

/// Безопасная обёртка
fn calculate_sum(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    unsafe {
        // Гарантируем корректность через проверку среза
        raw_memory_access(prices.as_ptr(), prices.len())
    }
}

fn main() {
    let prices = vec![100.0, 101.5, 99.8, 102.3];
    println!("Сумма цен: {:.2}", calculate_sum(&prices));
}
```

### 3. Документируй инварианты

```rust
/// Быстрый доступ к элементу массива без проверки границ
///
/// # Safety
///
/// Вызывающий код ДОЛЖЕН гарантировать:
/// - `index < data.len()`
/// - `data` не пуст
unsafe fn get_price_unchecked(data: &[f64], index: usize) -> f64 {
    *data.get_unchecked(index)
}

fn calculate_price_change(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    // Безопасно: проверили длину, индексы корректны
    unsafe {
        let last = get_price_unchecked(prices, prices.len() - 1);
        let first = get_price_unchecked(prices, 0);
        ((last - first) / first) * 100.0
    }
}

fn main() {
    let btc_prices = vec![40000.0, 41000.0, 39500.0, 42000.0];
    println!("Изменение цены: {:.2}%", calculate_price_change(&btc_prices));
}
```

## Пример: оптимизированный расчёт скользящей средней

Сравним безопасную и unsafe версии:

```rust
/// Безопасная версия SMA
fn sma_safe(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in period - 1..prices.len() {
        let sum: f64 = prices[i - period + 1..=i].iter().sum();
        result.push(sum / period as f64);
    }

    result
}

/// Unsafe версия SMA с оптимизацией
fn sma_unsafe_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);

    // Считаем первое окно
    let mut sum = 0.0;
    for i in 0..period {
        sum += prices[i];
    }
    result.push(sum / period as f64);

    // Скользящее окно: убираем первый элемент, добавляем следующий
    for i in period..prices.len() {
        unsafe {
            // БЕЗОПАСНО:
            // - i >= period (по условию цикла)
            // - i < prices.len() (по условию цикла)
            // - i - period < prices.len() (т.к. i < prices.len())
            sum -= prices.get_unchecked(i - period);
            sum += prices.get_unchecked(i);
        }
        result.push(sum / period as f64);
    }

    result
}

fn main() {
    let prices: Vec<f64> = (0..1000).map(|i| 40000.0 + (i as f64 * 0.1).sin() * 100.0).collect();
    let period = 20;

    // Бенчмарк
    use std::time::Instant;

    let start = Instant::now();
    let _sma_safe_result = sma_safe(&prices, period);
    let safe_duration = start.elapsed();

    let start = Instant::now();
    let _sma_unsafe_result = sma_unsafe_optimized(&prices, period);
    let unsafe_duration = start.elapsed();

    println!("Безопасная SMA: {:?}", safe_duration);
    println!("Unsafe SMA: {:?}", unsafe_duration);
    println!("Ускорение: {:.2}x", safe_duration.as_nanos() as f64 / unsafe_duration.as_nanos() as f64);
}
```

## Практические задания

### Задание 1: Безопасная обёртка для FFI

Создай безопасную обёртку для C-функции расчёта VWAP (Volume Weighted Average Price):

```rust
extern "C" {
    fn calculate_vwap_c(
        prices: *const f64,
        volumes: *const f64,
        len: usize,
        output: *mut f64,
    ) -> i32;
}

// Твоя задача: реализовать безопасную функцию
fn calculate_vwap_safe(prices: &[f64], volumes: &[f64]) -> Result<f64, String> {
    // TODO: реализуй проверки и вызов unsafe функции
    todo!()
}
```

### Задание 2: Lock-free счётчик сделок

Реализуй потокобезопасный счётчик общего количества сделок с использованием атомарных операций:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct TradeCounter {
    total_trades: AtomicU64,
    total_volume: AtomicU64, // Умножаем на 1000 для хранения 3 знаков после запятой
}

impl TradeCounter {
    fn new() -> Self {
        todo!()
    }

    fn add_trade(&self, volume: f64) {
        // TODO: атомарно увеличь счётчики
        todo!()
    }

    fn get_stats(&self) -> (u64, f64) {
        // TODO: верни (количество сделок, общий объём)
        todo!()
    }
}
```

### Задание 3: Оптимизация через get_unchecked

Оптимизируй функцию расчёта максимальной просадки (maximum drawdown):

```rust
/// Безопасная версия
fn max_drawdown_safe(equity_curve: &[f64]) -> f64 {
    let mut max_dd = 0.0;
    let mut peak = equity_curve[0];

    for &value in equity_curve {
        if value > peak {
            peak = value;
        }
        let dd = (peak - value) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

/// Твоя задача: создай unsafe оптимизированную версию
fn max_drawdown_unsafe(equity_curve: &[f64]) -> f64 {
    // TODO: используй get_unchecked для ускорения
    todo!()
}
```

### Задание 4: Безопасная статическая конфигурация

Реализуй систему конфигурации без использования `static mut`:

```rust
// Используй OnceLock или lazy_static
use std::sync::OnceLock;

struct ExchangeConfig {
    api_url: String,
    max_reconnects: u32,
    timeout_ms: u64,
}

// TODO: создай глобальную конфигурацию и функции для работы с ней
```

## Домашнее задание

1. **Анализ производительности unsafe:**
   - Реализуй функцию расчёта EMA (Exponential Moving Average) в двух версиях: safe и unsafe
   - Создай бенчмарк на массиве из 1_000_000 элементов
   - Сравни производительность
   - Оцени, стоит ли оптимизация дополнительного риска

2. **FFI для TA-Lib:**
   - Установи библиотеку TA-Lib (Technical Analysis Library)
   - Создай безопасные обёртки для 5 индикаторов: SMA, EMA, RSI, MACD, Bollinger Bands
   - Напиши тесты для проверки корректности
   - Добавь обработку всех возможных ошибок

3. **Lock-free очередь ордеров:**
   - Реализуй lock-free очередь для хранения ордеров (FIFO)
   - Поддержка операций: enqueue, dequeue, len
   - Тестирование в многопоточной среде (10 потоков)
   - Сравнение производительности с `std::sync::mpsc`

4. **Детектор unsafe кода:**
   - Напиши утилиту, которая анализирует Rust-проект
   - Находит все `unsafe` блоки и функции
   - Проверяет наличие документации с `# Safety` секцией
   - Генерирует отчёт о потенциальных рисках

5. **Безопасные абстракции:**
   - Возьми любую unsafe функцию из стандартной библиотеки
   - Создай максимально безопасную обёртку
   - Докажи через типы, что неправильное использование невозможно
   - Напиши документацию с примерами

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **unsafe** | Режим, позволяющий обойти некоторые проверки компилятора |
| **Сырые указатели** | `*const T` и `*mut T` — указатели без гарантий владения |
| **FFI** | Взаимодействие с кодом на других языках (C/C++) |
| **get_unchecked** | Доступ к элементам без проверки границ массива |
| **Атомарные операции** | Lock-free синхронизация через `std::sync::atomic` |
| **static mut** | Изменяемые глобальные переменные (лучше избегать) |
| **OnceLock** | Безопасная альтернатива для глобальных переменных |
| **Инварианты** | Условия, которые должны выполняться для корректности unsafe кода |

## Важные принципы

1. **unsafe ≠ опасно** — это инструмент для экспертов, а не признак плохого кода
2. **Минимизируй область** — делай unsafe блоки как можно меньше
3. **Документируй** — всегда пиши `# Safety` с условиями корректности
4. **Оборачивай** — создавай безопасные API поверх unsafe кода
5. **Проверяй** — используй инструменты типа Miri для поиска undefined behavior
6. **Оправдывай** — unsafe должен давать реальную пользу (производительность, FFI)

## Когда НЕ использовать unsafe

❌ **Не используй unsafe для:**
- Обхода системы типов "потому что удобно"
- Изменяемых глобальных переменных без крайней необходимости
- Преждевременной оптимизации ("вдруг будет быстрее")
- Кода, который можно написать безопасно с минимальными потерями

✅ **Используй unsafe когда:**
- Взаимодействуешь с внешними библиотеками через FFI
- Оптимизируешь критические участки после профилирования
- Реализуешь низкоуровневые структуры данных
- Пишешь абстракции для безопасного интерфейса

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
