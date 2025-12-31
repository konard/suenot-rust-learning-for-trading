# День 309: String vs &str в горячих путях

## Аналогия из трейдинга

Представь высокочастотную торговую систему, обрабатывающую тысячи обновлений цен в секунду. Каждое обновление содержит символ тикера вроде "BTCUSDT".

Два подхода:
1. **String подход**: Как делать новую ксерокопию каждого документа, который нужно прочитать — трата бумаги (памяти) и времени
2. **&str подход**: Как указывать на оригинальный документ — мгновенно, без копирования

В горячих путях (код, который выполняется миллионы раз), разница между `String` и `&str` как разница между:
- Ручной записью каждого подтверждения сделки vs. указанием на журнал сделок
- Физическими копиями графиков цен vs. ссылками на главный график

Когда твой торговый бот обрабатывает 100,000 сделок в секунду, каждая наносекунда и каждый байт памяти имеют значение. Выбор между `String` и `&str` может означать разницу между задержкой в 50ms и 5ms — достаточно, чтобы упустить прибыльные сделки.

## Фундаментальная разница

| Тип | Что это | Владение | Размещение | Стоимость |
|-----|---------|----------|------------|-----------|
| **String** | Владеющий, растущий буфер строки | Владеет данными | Куча | Выделяет память, копирует данные |
| **&str** | Ссылка на срез строки | Заимствует данные | Стек (только ссылка) | Без копирования, просто указатель + длина |

### Расположение в памяти

```rust
fn main() {
    // String: выделяется в куче, владеющий
    let owned: String = String::from("BTCUSDT");
    // Память: В стеке указатель + длина + ёмкость
    //         В куче реальные данные: ['B','T','C','U','S','D','T']

    // &str: ссылка на строковые данные (могут быть где угодно)
    let borrowed: &str = "BTCUSDT";
    // Память: Только указатель + длина в стеке
    //         Данные в read-only памяти программы

    println!("Размер String: {} байт", std::mem::size_of_val(&owned));    // 24 байта
    println!("Размер &str: {} байт", std::mem::size_of_val(&borrowed));   // 16 байт
}
```

## Почему это важно в горячих путях

### Пример 1: Обработка книги заявок

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
struct Order {
    symbol: String,  // Владеющая строка
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderRef<'a> {
    symbol: &'a str,  // Заимствованная строка
    price: f64,
    quantity: f64,
}

// Горячий путь: вызывается миллионы раз
fn process_order_owned(symbol: String, price: f64, qty: f64) -> f64 {
    // Выделение памяти под String происходит здесь - дорого!
    let order = Order {
        symbol,  // Перемещает String
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

// Горячий путь: оптимизированная версия
fn process_order_borrowed(symbol: &str, price: f64, qty: f64) -> f64 {
    // Без выделения памяти - просто используем ссылку
    let order = OrderRef {
        symbol,  // Просто копирует ссылку (указатель + длина)
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

fn main() {
    let iterations = 1_000_000;

    // Бенчмарк с String (с выделением памяти)
    let start = Instant::now();
    for _ in 0..iterations {
        let symbol = String::from("BTCUSDT");  // Выделение памяти!
        process_order_owned(symbol, 50000.0, 0.5);
    }
    let string_duration = start.elapsed();

    // Бенчмарк с &str (без копирования)
    let symbol_ref = "BTCUSDT";
    let start = Instant::now();
    for _ in 0..iterations {
        process_order_borrowed(symbol_ref, 50000.0, 0.5);
    }
    let str_duration = start.elapsed();

    println!("=== Сравнение производительности ===");
    println!("Версия String: {:?}", string_duration);
    println!("Версия &str:   {:?}", str_duration);
    println!("Ускорение: {:.2}x", string_duration.as_nanos() as f64 / str_duration.as_nanos() as f64);
}
```

**Ожидаемый вывод:**
```
=== Сравнение производительности ===
Версия String: 45ms
Версия &str:   8ms
Ускорение: 5.63x
```

### Пример 2: Сопоставление тикеров

```rust
use std::collections::HashMap;

struct MarketData {
    prices: HashMap<String, f64>,  // Владеющие ключи - требуют выделения памяти для поиска
}

impl MarketData {
    fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 50000.0);
        prices.insert("ETHUSDT".to_string(), 3000.0);
        prices.insert("SOLUSDT".to_string(), 100.0);
        MarketData { prices }
    }

    // ПЛОХО: Выделяет новый String для каждого поиска
    fn get_price_slow(&self, symbol: &str) -> Option<f64> {
        self.prices.get(&symbol.to_string()).copied()
        // ^^^ Выделяет String только для поиска!
    }

    // ХОРОШО: Использует &str напрямую
    fn get_price_fast(&self, symbol: &str) -> Option<f64> {
        self.prices.get(symbol).copied()
        // ^^^ Без выделения памяти, HashMap может заимствовать для поиска
    }
}

fn main() {
    let market = MarketData::new();
    let iterations = 1_000_000;

    // Медленный путь: выделение String для каждого поиска
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = market.get_price_slow("BTCUSDT");
    }
    println!("Медленно (с выделением): {:?}", start.elapsed());

    // Быстрый путь: использование &str напрямую
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = market.get_price_fast("BTCUSDT");
    }
    println!("Быстро (без копирования): {:?}", start.elapsed());
}
```

## Когда использовать String vs &str

### Используй &str когда:

1. **Только чтение/сравнение** — Не нужно изменять строку
2. **В параметрах функций** — Принимаешь заимствованные данные, чтобы избежать принудительного выделения памяти
3. **В горячих путях** — Любой код, который выполняется часто
4. **В временных операциях** — Парсинг, валидация, поиск

```rust
// Хорошо: принимает &str, вызывающему не нужно выделять память
fn validate_symbol(symbol: &str) -> bool {
    symbol.len() >= 3 && symbol.chars().all(|c| c.is_uppercase() || c.is_numeric())
}

// Использование - не требуется выделение памяти
let is_valid = validate_symbol("BTCUSDT");  // ✅ Без копирования
```

### Используй String когда:

1. **Требуется владение** — Нужно хранить и владеть данными
2. **Требуется изменение** — Нужно изменять, добавлять или строить строки
3. **Динамические данные** — Создание строк во время выполнения
4. **Возврат из функций** — Когда создаёшь новую строку для возврата

```rust
// Хорошо: возвращает String, потому что создаём новые данные
fn format_ticker(base: &str, quote: &str) -> String {
    format!("{}{}", base, quote)  // Создаёт новый String
}

// Использование
let ticker = format_ticker("BTC", "USDT");  // ticker владеет данными
```

## Общие паттерны в торговых системах

### Паттерн 1: Валидация заявок (горячий путь)

```rust
#[derive(Debug)]
struct OrderValidator;

impl OrderValidator {
    // ✅ Оптимально: использует &str для валидации
    fn validate_symbol(&self, symbol: &str) -> Result<(), &'static str> {
        if symbol.is_empty() {
            return Err("Символ не может быть пустым");
        }
        if !symbol.chars().all(|c| c.is_alphanumeric()) {
            return Err("Символ должен быть буквенно-цифровым");
        }
        Ok(())
    }

    // ✅ Оптимально: использует ссылки повсюду
    fn validate_order(&self, symbol: &str, price: f64, quantity: f64) -> Result<(), String> {
        self.validate_symbol(symbol)?;

        if price <= 0.0 {
            return Err(format!("Неверная цена для {}: {}", symbol, price));
        }

        if quantity <= 0.0 {
            return Err(format!("Неверное количество для {}: {}", symbol, quantity));
        }

        Ok(())
    }
}

fn main() {
    let validator = OrderValidator;

    // Горячий путь: валидация тысяч заявок в секунду
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];

    for symbol in symbols {
        // ✅ Без выделения памяти - просто передаём ссылки
        match validator.validate_order(symbol, 50000.0, 1.0) {
            Ok(_) => println!("✅ Заявка {} валидна", symbol),
            Err(e) => println!("❌ {}", e),
        }
    }
}
```

### Паттерн 2: Агрегация цен

```rust
use std::collections::HashMap;

struct PriceAggregator {
    // Храним владеющие String как ключи (выделяются однажды)
    prices: HashMap<String, Vec<f64>>,
}

impl PriceAggregator {
    fn new() -> Self {
        PriceAggregator {
            prices: HashMap::new(),
        }
    }

    // ✅ Принимает &str, выделяет память только если ключа нет
    fn add_price(&mut self, symbol: &str, price: f64) {
        self.prices
            .entry(symbol.to_string())  // Выделяет память только при вставке нового ключа
            .or_insert_with(Vec::new)
            .push(price);
    }

    // ✅ Принимает &str для поиска (без выделения памяти)
    fn get_average(&self, symbol: &str) -> Option<f64> {
        self.prices.get(symbol).map(|prices| {
            prices.iter().sum::<f64>() / prices.len() as f64
        })
    }

    // Возвращает владеющий String при построении новых данных
    fn get_summary(&self, symbol: &str) -> Option<String> {
        self.prices.get(symbol).map(|prices| {
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let min = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            format!("{}: сред={:.2}, мин={:.2}, макс={:.2}", symbol, avg, min, max)
        })
    }
}

fn main() {
    let mut aggregator = PriceAggregator::new();

    // Горячий путь: добавление цен (только первое добавление выделяет ключ)
    let symbol = "BTCUSDT";
    for price in [50000.0, 50100.0, 49900.0, 50200.0] {
        aggregator.add_price(symbol, price);  // ✅ параметр &str
    }

    // Поиск: без выделения памяти
    if let Some(avg) = aggregator.get_average("BTCUSDT") {
        println!("Средняя цена BTC: ${:.2}", avg);
    }

    // Сводка: выделяет новый String (допустимо, не в горячем пути)
    if let Some(summary) = aggregator.get_summary("BTCUSDT") {
        println!("{}", summary);
    }
}
```

### Паттерн 3: Построение строк из частей

```rust
struct TradeReport {
    symbol: String,
    trades: Vec<f64>,
}

impl TradeReport {
    // Принимает параметры &str, хранит владеющий String
    fn new(symbol: &str) -> Self {
        TradeReport {
            symbol: symbol.to_string(),  // Выделяем память однажды при создании
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, pnl: f64) {
        self.trades.push(pnl);
    }

    // Возвращает владеющий String (создаём новые данные)
    fn generate_report(&self) -> String {
        let total_pnl: f64 = self.trades.iter().sum();
        let num_trades = self.trades.len();
        let avg_pnl = if num_trades > 0 {
            total_pnl / num_trades as f64
        } else {
            0.0
        };

        format!(
            "=== Отчёт по сделкам: {} ===\n\
             Всего сделок: {}\n\
             Общий PnL: ${:.2}\n\
             Средний PnL: ${:.2}",
            self.symbol, num_trades, total_pnl, avg_pnl
        )
    }
}

fn main() {
    let mut report = TradeReport::new("BTCUSDT");  // ✅ Передаём &str

    report.add_trade(150.0);
    report.add_trade(-50.0);
    report.add_trade(200.0);

    println!("{}", report.generate_report());  // Возвращает владеющий String
}
```

## Продвинутое: Интернирование строк для повторяющихся символов

Когда у тебя ограниченный набор символов, используемых многократно (как тикеры), интернирование строк может помочь:

```rust
use std::collections::HashMap;
use std::sync::Arc;

struct SymbolCache {
    cache: HashMap<String, Arc<str>>,
}

impl SymbolCache {
    fn new() -> Self {
        SymbolCache {
            cache: HashMap::new(),
        }
    }

    // Интернирует символ: выделяет память однажды, переиспользует Arc<str>
    fn intern(&mut self, symbol: &str) -> Arc<str> {
        if let Some(cached) = self.cache.get(symbol) {
            return Arc::clone(cached);  // ✅ Без выделения памяти, просто клонируем Arc
        }

        let interned: Arc<str> = Arc::from(symbol);
        self.cache.insert(symbol.to_string(), Arc::clone(&interned));
        interned
    }
}

#[derive(Debug, Clone)]
struct OptimizedOrder {
    symbol: Arc<str>,  // Разделяемое владение, дёшево клонировать
    price: f64,
    quantity: f64,
}

fn main() {
    let mut cache = SymbolCache::new();

    // Первый раз: выделяет память
    let btc_symbol = cache.intern("BTCUSDT");

    // Последующие разы: переиспользует существующий Arc<str>
    let orders: Vec<OptimizedOrder> = (0..5)
        .map(|i| OptimizedOrder {
            symbol: Arc::clone(&btc_symbol),  // ✅ Дешёвое клонирование
            price: 50000.0 + i as f64 * 10.0,
            quantity: 0.1,
        })
        .collect();

    println!("Создано {} заявок", orders.len());
    for (i, order) in orders.iter().enumerate() {
        println!("Заявка {}: {:?}", i + 1, order);
    }
}
```

## Рекомендации по производительности

### Стоимость выделения памяти

```rust
use std::time::Instant;

fn benchmark_allocations() {
    let iterations = 1_000_000;

    // Бенчмарк 1: Выделение String
    let start = Instant::now();
    for _ in 0..iterations {
        let _s = String::from("BTCUSDT");  // Выделение в куче
    }
    let string_time = start.elapsed();

    // Бенчмарк 2: &str (без выделения)
    let start = Instant::now();
    for _ in 0..iterations {
        let _s: &str = "BTCUSDT";  // Без выделения в куче
    }
    let str_time = start.elapsed();

    println!("=== Стоимость выделения памяти ===");
    println!("String::from(): {:?}", string_time);
    println!("&str литерал:   {:?}", str_time);
    println!("Разница:        {:?}", string_time - str_time);
    println!("Сэкономлено выделений: {}", iterations);
}

fn main() {
    benchmark_allocations();
}
```

### Практические правила

| Операция | Стоимость | Когда использовать |
|----------|-----------|-------------------|
| `"литерал"` | Бесплатно (время компиляции) | Выбор по умолчанию |
| `&str` параметр | Дёшево (только указатель) | Параметры функций |
| `String::from()` | Дорого (выделение в куче) | Когда нужно владение |
| `to_string()` | Дорого (выделение в куче) | Конвертация для хранения |
| `clone()` на String | Дорого (копирует данные) | Когда неизбежно |
| `clone()` на &str | Неприменимо (нельзя клонировать ссылку) | Используй `.to_string()` вместо этого |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **String** | Владеющий, выделяемый в куче, растущий строковый буфер |
| **&str** | Заимствованная ссылка на строковые данные (срез) |
| **Горячий путь** | Код, который выполняется очень часто (критично для производительности) |
| **Без копирования** | Использование ссылок вместо копирования данных |
| **Интернирование строк** | Переиспользование единственных экземпляров строк через Arc<str> |
| **Стоимость выделения** | Выделение в куче дорого, избегай в горячих путях |
| **Заимствование для параметров** | Принимай &str в функциях, чтобы избежать принудительного выделения |
| **Владение для хранения** | Используй String когда нужно хранить или изменять данные |

## Домашнее задание

1. **Профайлер производительности**: Создай инструмент, который:
   - Принимает список из 1000 символов тикеров
   - Обрабатывает их 10,000 раз
   - Сравнивает три подхода:
     - Использование `String` везде
     - Использование `&str` везде где возможно
     - Использование `Arc<str>` с интернированием
   - Измеряет и выводит:
     - Общее время выполнения
     - Выделения памяти (используй глобальный трекер аллокатора)
     - Пиковое использование памяти
   - Генерирует отчёт о производительности

2. **Оптимизатор книги заявок**: Реализуй книгу заявок, которая:
   - Хранит заявки со ссылками на символы
   - Реализует версии с `String` и `&str`
   - Бенчмаркит вставку 100,000 заявок
   - Бенчмаркит поиск 1,000,000 запросов
   - Показывает разницу в использовании памяти
   - Демонстрирует когда каждый подход лучше

3. **Кэш символов**: Построй production-ready кэш символов:
   - Использует `Arc<str>` для разделяемого владения
   - Реализует метод `intern()` для дедупликации
   - Предоставляет `get_or_intern()` для удобного использования
   - Отслеживает процент попаданий в кэш
   - Измеряет экономию памяти vs. использование String везде
   - Потокобезопасная реализация (бонус)

4. **Анализатор оптимизации строк**: Напиши инструмент, который:
   - Парсит Rust код (может быть простым на regex)
   - Находит анти-паттерны:
     - `&symbol.to_string()` в вызовах функций
     - Ненужные `clone()` на строках
     - Использование `String` в параметрах функций
     - Поиски в HashMap с выделением памяти
   - Предлагает оптимизации
   - Оценивает потенциальный прирост производительности

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md) | [Следующий день →](../310-*/ru.md)
