# День 86: BTreeMap — отсортированные цены

## Аналогия из трейдинга

Представьте **стакан заявок** (order book) на бирже. Заявки на покупку отсортированы по убыванию цены (лучшая цена сверху), а заявки на продажу — по возрастанию. Биржа должна **мгновенно находить** лучшие цены и поддерживать их в отсортированном порядке.

В Rust для таких задач идеально подходит **BTreeMap** — структура данных, которая хранит ключи в отсортированном порядке и позволяет эффективно выполнять операции с диапазонами цен.

## Что такое BTreeMap?

`BTreeMap` — это ассоциативный массив (словарь), который:
- Хранит пары ключ-значение
- **Автоматически сортирует** ключи
- Позволяет искать по диапазону
- Быстро находит минимум и максимум

```rust
use std::collections::BTreeMap;

fn main() {
    // Стакан заявок: цена -> объём
    let mut order_book: BTreeMap<u64, f64> = BTreeMap::new();

    // Добавляем заявки (цена в центах для точности)
    order_book.insert(4200000, 1.5);  // $42,000.00 -> 1.5 BTC
    order_book.insert(4201000, 2.3);  // $42,010.00 -> 2.3 BTC
    order_book.insert(4199500, 0.8);  // $41,995.00 -> 0.8 BTC

    // Итерация идёт в отсортированном порядке!
    for (price, volume) in &order_book {
        println!("${:.2}: {} BTC", *price as f64 / 100.0, volume);
    }
}
```

## BTreeMap vs HashMap

| Характеристика | BTreeMap | HashMap |
|---------------|----------|---------|
| Порядок ключей | Отсортированный | Произвольный |
| Поиск по ключу | O(log n) | O(1) |
| Вставка | O(log n) | O(1) |
| Диапазонные запросы | Да | Нет |
| Min/Max | O(log n) | O(n) |

**Используйте BTreeMap**, когда важен порядок или нужны диапазонные запросы.

## Основные операции

### Создание и вставка

```rust
use std::collections::BTreeMap;

fn main() {
    // Пустая карта
    let mut prices: BTreeMap<String, f64> = BTreeMap::new();

    // Вставка
    prices.insert("BTC".to_string(), 42000.0);
    prices.insert("ETH".to_string(), 2800.0);
    prices.insert("SOL".to_string(), 98.5);

    println!("{:?}", prices);
    // Выводит в алфавитном порядке: {"BTC": 42000.0, "ETH": 2800.0, "SOL": 98.5}
}
```

### Доступ к элементам

```rust
use std::collections::BTreeMap;

fn main() {
    let mut prices: BTreeMap<&str, f64> = BTreeMap::new();
    prices.insert("BTC", 42000.0);
    prices.insert("ETH", 2800.0);

    // get() возвращает Option<&V>
    if let Some(btc_price) = prices.get("BTC") {
        println!("BTC: ${}", btc_price);
    }

    // get_mut() для изменения
    if let Some(eth_price) = prices.get_mut("ETH") {
        *eth_price = 2850.0;
    }

    // entry() API для условной вставки
    prices.entry("XRP").or_insert(0.55);

    println!("{:?}", prices);
}
```

### Первый и последний элемент

```rust
use std::collections::BTreeMap;

fn main() {
    let mut bid_book: BTreeMap<u64, f64> = BTreeMap::new();

    // Заявки на покупку (bid)
    bid_book.insert(4195000, 2.0);
    bid_book.insert(4198000, 1.5);
    bid_book.insert(4200000, 3.0);  // Лучший bid
    bid_book.insert(4190000, 5.0);

    // Лучшая цена покупки — максимальная
    if let Some((&best_price, &volume)) = bid_book.last_key_value() {
        println!("Best Bid: ${:.2} x {}", best_price as f64 / 100.0, volume);
    }

    // Худшая цена покупки — минимальная
    if let Some((&worst_price, &volume)) = bid_book.first_key_value() {
        println!("Worst Bid: ${:.2} x {}", worst_price as f64 / 100.0, volume);
    }
}
```

## Диапазонные запросы

Главное преимущество `BTreeMap` — работа с диапазонами:

```rust
use std::collections::BTreeMap;

fn main() {
    let mut price_history: BTreeMap<u64, f64> = BTreeMap::new();

    // Временные метки (unix timestamp) -> цена
    price_history.insert(1700000000, 42000.0);
    price_history.insert(1700000060, 42050.0);
    price_history.insert(1700000120, 42100.0);
    price_history.insert(1700000180, 42080.0);
    price_history.insert(1700000240, 42150.0);
    price_history.insert(1700000300, 42200.0);

    // Получить цены за последние 2 минуты
    let start = 1700000180;
    let end = 1700000300;

    println!("Prices from {} to {}:", start, end);
    for (ts, price) in price_history.range(start..=end) {
        println!("  {}: ${}", ts, price);
    }
}
```

### Типы диапазонов

```rust
use std::collections::BTreeMap;
use std::ops::Bound;

fn main() {
    let mut levels: BTreeMap<i32, &str> = BTreeMap::new();
    for i in 1..=10 {
        levels.insert(i, "level");
    }

    // range(start..end) — от start до end (не включая end)
    println!("1..5: {:?}", levels.range(1..5).collect::<Vec<_>>());

    // range(start..=end) — от start до end (включительно)
    println!("1..=5: {:?}", levels.range(1..=5).collect::<Vec<_>>());

    // range(start..) — от start до конца
    println!("8..: {:?}", levels.range(8..).collect::<Vec<_>>());

    // range(..end) — от начала до end
    println!("..3: {:?}", levels.range(..3).collect::<Vec<_>>());

    // С использованием Bound для сложных запросов
    use std::ops::Bound::{Excluded, Included};
    println!("(3, 7]: {:?}",
        levels.range((Excluded(3), Included(7))).collect::<Vec<_>>());
}
```

## Практический пример: стакан заявок

```rust
use std::collections::BTreeMap;

struct OrderBook {
    // Для bids используем отрицательные цены для обратной сортировки
    bids: BTreeMap<i64, f64>,  // -price -> volume (для сортировки по убыванию)
    asks: BTreeMap<i64, f64>,  // price -> volume
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_bid(&mut self, price: i64, volume: f64) {
        *self.bids.entry(-price).or_insert(0.0) += volume;
    }

    fn add_ask(&mut self, price: i64, volume: f64) {
        *self.asks.entry(price).or_insert(0.0) += volume;
    }

    fn best_bid(&self) -> Option<(i64, f64)> {
        self.bids.first_key_value().map(|(&p, &v)| (-p, v))
    }

    fn best_ask(&self) -> Option<(i64, f64)> {
        self.asks.first_key_value().map(|(&p, &v)| (p, v))
    }

    fn spread(&self) -> Option<i64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }

    fn display_top(&self, depth: usize) {
        println!("╔══════════════════════════════════════╗");
        println!("║           ORDER BOOK                 ║");
        println!("╠══════════════════════════════════════╣");

        // Top asks (в обратном порядке)
        let asks: Vec<_> = self.asks.iter().take(depth).collect();
        for (&price, &volume) in asks.iter().rev() {
            println!("║  ASK: ${:>10.2} | {:>8.4} BTC     ║",
                price as f64 / 100.0, volume);
        }

        println!("╠══════════════════════════════════════╣");

        if let Some(spread) = self.spread() {
            println!("║  SPREAD: ${:.2}                      ║",
                spread as f64 / 100.0);
        }

        println!("╠══════════════════════════════════════╣");

        // Top bids
        for (&neg_price, &volume) in self.bids.iter().take(depth) {
            println!("║  BID: ${:>10.2} | {:>8.4} BTC     ║",
                -neg_price as f64 / 100.0, volume);
        }

        println!("╚══════════════════════════════════════╝");
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Добавляем заявки
    book.add_bid(4200000, 2.5);   // $42,000.00
    book.add_bid(4199500, 1.8);   // $41,995.00
    book.add_bid(4199000, 3.2);   // $41,990.00
    book.add_bid(4198000, 5.0);   // $41,980.00

    book.add_ask(4200500, 1.2);   // $42,005.00
    book.add_ask(4201000, 2.0);   // $42,010.00
    book.add_ask(4201500, 0.8);   // $42,015.00
    book.add_ask(4202000, 4.5);   // $42,020.00

    book.display_top(4);

    if let (Some((bid, _)), Some((ask, _))) = (book.best_bid(), book.best_ask()) {
        let mid = (bid + ask) as f64 / 2.0 / 100.0;
        println!("\nMid Price: ${:.2}", mid);
    }
}
```

## Практический пример: ценовые уровни

```rust
use std::collections::BTreeMap;

fn main() {
    // Хранение объёмов на ценовых уровнях
    let mut volume_profile: BTreeMap<u64, f64> = BTreeMap::new();

    // Добавляем сделки
    let trades = vec![
        (4200000u64, 1.5f64),
        (4200500, 2.0),
        (4200000, 0.8),
        (4199500, 1.2),
        (4200500, 3.0),
        (4201000, 0.5),
    ];

    for (price, volume) in trades {
        *volume_profile.entry(price).or_insert(0.0) += volume;
    }

    // Найти Point of Control (уровень с максимальным объёмом)
    let poc = volume_profile
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(&price, &vol)| (price, vol));

    println!("=== Volume Profile ===");
    for (price, volume) in &volume_profile {
        let bar = "█".repeat((volume * 10.0) as usize);
        println!("${:.2}: {:>5.2} {}",
            *price as f64 / 100.0, volume, bar);
    }

    if let Some((price, vol)) = poc {
        println!("\nPoint of Control: ${:.2} ({:.2} BTC)",
            price as f64 / 100.0, vol);
    }

    // Найти Value Area (70% объёма вокруг POC)
    let total_volume: f64 = volume_profile.values().sum();
    println!("Total Volume: {:.2} BTC", total_volume);
}
```

## Практический пример: история цен с временными метками

```rust
use std::collections::BTreeMap;

struct PriceHistory {
    data: BTreeMap<u64, f64>,  // timestamp -> price
}

impl PriceHistory {
    fn new() -> Self {
        PriceHistory {
            data: BTreeMap::new(),
        }
    }

    fn add(&mut self, timestamp: u64, price: f64) {
        self.data.insert(timestamp, price);
    }

    fn get_range(&self, from: u64, to: u64) -> Vec<(u64, f64)> {
        self.data
            .range(from..=to)
            .map(|(&ts, &price)| (ts, price))
            .collect()
    }

    fn get_latest(&self, n: usize) -> Vec<(u64, f64)> {
        self.data
            .iter()
            .rev()
            .take(n)
            .map(|(&ts, &price)| (ts, price))
            .collect()
    }

    fn high_low(&self, from: u64, to: u64) -> Option<(f64, f64)> {
        let prices: Vec<f64> = self.data
            .range(from..=to)
            .map(|(_, &p)| p)
            .collect();

        if prices.is_empty() {
            return None;
        }

        let high = prices.iter().cloned().fold(f64::MIN, f64::max);
        let low = prices.iter().cloned().fold(f64::MAX, f64::min);

        Some((high, low))
    }

    fn price_at_or_before(&self, timestamp: u64) -> Option<f64> {
        self.data
            .range(..=timestamp)
            .last()
            .map(|(_, &price)| price)
    }
}

fn main() {
    let mut history = PriceHistory::new();

    // Добавляем минутные данные
    let base_ts = 1700000000u64;
    let prices = [42000.0, 42050.0, 42100.0, 42080.0, 42150.0,
                  42200.0, 42180.0, 42250.0, 42300.0, 42280.0];

    for (i, price) in prices.iter().enumerate() {
        history.add(base_ts + (i as u64 * 60), *price);
    }

    // Последние 5 цен
    println!("=== Last 5 prices ===");
    for (ts, price) in history.get_latest(5) {
        println!("  {}: ${:.2}", ts, price);
    }

    // High/Low за период
    let from = base_ts + 120;
    let to = base_ts + 420;
    if let Some((high, low)) = history.high_low(from, to) {
        println!("\nHigh/Low from {} to {}:", from, to);
        println!("  High: ${:.2}", high);
        println!("  Low: ${:.2}", low);
        println!("  Range: ${:.2}", high - low);
    }

    // Цена на определённый момент
    let query_ts = base_ts + 150;  // Между точками
    if let Some(price) = history.price_at_or_before(query_ts) {
        println!("\nPrice at or before {}: ${:.2}", query_ts, price);
    }
}
```

## Практический пример: риск-менеджмент по уровням

```rust
use std::collections::BTreeMap;

fn main() {
    // Уровни стоп-лосса для разных размеров позиции
    let mut risk_levels: BTreeMap<f64, f64> = BTreeMap::new();

    // Цена -> процент от капитала для выхода
    risk_levels.insert(41000.0, 50.0);  // Если упадёт до 41K - выходим 50%
    risk_levels.insert(40000.0, 30.0);  // Если до 40K - ещё 30%
    risk_levels.insert(39000.0, 20.0);  // Последние 20%

    let current_price = 40500.0;

    println!("Current price: ${:.2}", current_price);
    println!("\nActive stop-loss levels:");

    // Найти все уровни ниже текущей цены
    for (level, pct) in risk_levels.range(..current_price) {
        println!("  ${:.2}: exit {:.0}% of position", level, pct);
    }

    // Ближайший уровень стоп-лосса
    if let Some((level, pct)) = risk_levels.range(..current_price).last() {
        let distance = current_price - level;
        let distance_pct = (distance / current_price) * 100.0;
        println!("\nNearest stop: ${:.2} ({:.2}% away)", level, distance_pct);
        println!("Will exit: {:.0}% of position", pct);
    }
}
```

## Удаление элементов

```rust
use std::collections::BTreeMap;

fn main() {
    let mut orders: BTreeMap<u64, f64> = BTreeMap::new();
    orders.insert(100, 1.0);
    orders.insert(200, 2.0);
    orders.insert(300, 3.0);
    orders.insert(400, 4.0);
    orders.insert(500, 5.0);

    // Удалить конкретный ордер
    if let Some(volume) = orders.remove(&300) {
        println!("Removed order at 300 with volume {}", volume);
    }

    // Удалить первый элемент
    if let Some((price, volume)) = orders.pop_first() {
        println!("Popped first: {} -> {}", price, volume);
    }

    // Удалить последний элемент
    if let Some((price, volume)) = orders.pop_last() {
        println!("Popped last: {} -> {}", price, volume);
    }

    println!("Remaining: {:?}", orders);
}
```

## Слияние BTreeMap

```rust
use std::collections::BTreeMap;

fn main() {
    // Два источника данных о ценах
    let mut exchange_a: BTreeMap<&str, f64> = BTreeMap::new();
    exchange_a.insert("BTC", 42000.0);
    exchange_a.insert("ETH", 2800.0);

    let mut exchange_b: BTreeMap<&str, f64> = BTreeMap::new();
    exchange_b.insert("ETH", 2810.0);  // Другая цена!
    exchange_b.insert("SOL", 98.0);

    // Слияние (exchange_b перезаписывает exchange_a)
    for (symbol, price) in exchange_b {
        exchange_a.insert(symbol, price);
    }

    println!("Merged prices: {:?}", exchange_a);

    // Или с усреднением цен
    let mut prices_a: BTreeMap<&str, f64> = BTreeMap::new();
    prices_a.insert("BTC", 42000.0);
    prices_a.insert("ETH", 2800.0);

    let prices_b: BTreeMap<&str, f64> = BTreeMap::from([
        ("ETH", 2810.0),
        ("SOL", 98.0),
    ]);

    for (symbol, price_b) in &prices_b {
        prices_a
            .entry(symbol)
            .and_modify(|p| *p = (*p + price_b) / 2.0)
            .or_insert(*price_b);
    }

    println!("Averaged prices: {:?}", prices_a);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `BTreeMap::new()` | Создание пустой карты |
| `insert(k, v)` | Вставка пары ключ-значение |
| `get(&k)` | Получение значения по ключу |
| `first_key_value()` | Минимальный ключ и значение |
| `last_key_value()` | Максимальный ключ и значение |
| `range(start..end)` | Итерация по диапазону |
| `pop_first()` / `pop_last()` | Удаление первого/последнего |
| `entry().or_insert()` | Условная вставка |

## Упражнения

1. Создайте `BTreeMap` для хранения цен закрытия по датам. Реализуйте функцию поиска максимальной цены за заданный период.

2. Реализуйте простой стакан заявок с функциями `add_order`, `remove_order`, `get_best_bid`, `get_best_ask`.

3. Создайте систему ценовых уровней поддержки/сопротивления. Найдите ближайший уровень выше и ниже текущей цены.

4. Реализуйте кэш исторических данных с автоматическим удалением старых записей (хранить только последние N минут).

## Домашнее задание

1. **Order Matching Engine**: Реализуйте простой матчинг ордеров. Когда bid >= ask, должна происходить сделка.

2. **VWAP Calculator**: Используя BTreeMap<timestamp, (price, volume)>, рассчитайте Volume Weighted Average Price за период.

3. **Support/Resistance Finder**: На основе истории цен найдите уровни, где цена часто разворачивалась (уровни с большим количеством касаний).

4. **Multi-Exchange Aggregator**: Объедините стаканы заявок с нескольких бирж в один агрегированный стакан.

## Навигация

[← Предыдущий день](../085-hashset-unique-tickers/ru.md) | [Следующий день →](../087-vecdeque-order-queue/ru.md)
