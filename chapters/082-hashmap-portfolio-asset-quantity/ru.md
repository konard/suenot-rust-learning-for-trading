# День 82: HashMap — Портфель: Актив → Количество

## Аналогия из трейдинга

Представь свой торговый портфель. У тебя есть разные активы:
- **BTC**: 0.5 единиц
- **ETH**: 10.0 единиц
- **USDT**: 50000.0 единиц

Это классический пример структуры "ключ-значение":
- **Ключ** (Key): название актива (тикер)
- **Значение** (Value): количество актива

В Rust для таких задач используется `HashMap` — хеш-таблица, которая позволяет быстро находить значение по ключу.

## Что такое HashMap?

`HashMap<K, V>` — это коллекция пар ключ-значение:
- `K` — тип ключа (должен реализовывать `Eq` и `Hash`)
- `V` — тип значения

**Преимущества:**
- Поиск по ключу за O(1) в среднем случае
- Быстрая вставка и удаление
- Гибкий размер

```rust
use std::collections::HashMap;

fn main() {
    // Создаём портфель: актив -> количество
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Добавляем активы
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("USDT"), 50000.0);

    println!("Портфель: {:?}", portfolio);
}
```

## Создание HashMap

### Пустой HashMap

```rust
use std::collections::HashMap;

fn main() {
    // Явное указание типов
    let portfolio: HashMap<String, f64> = HashMap::new();

    // Rust выведет типы при первой вставке
    let mut balances = HashMap::new();
    balances.insert("BTC", 1.5);  // HashMap<&str, f64>

    println!("Portfolio: {:?}", portfolio);
    println!("Balances: {:?}", balances);
}
```

### HashMap с начальной ёмкостью

```rust
use std::collections::HashMap;

fn main() {
    // Предвыделяем место под 100 активов
    let mut large_portfolio: HashMap<String, f64> = HashMap::with_capacity(100);

    large_portfolio.insert(String::from("BTC"), 1.0);

    println!("Capacity: {}", large_portfolio.capacity());
}
```

### Создание из итератора

```rust
use std::collections::HashMap;

fn main() {
    // Из массива кортежей
    let assets = [
        ("BTC", 0.5),
        ("ETH", 10.0),
        ("SOL", 100.0),
    ];

    let portfolio: HashMap<&str, f64> = assets.into_iter().collect();

    println!("Portfolio: {:?}", portfolio);

    // Из двух векторов с помощью zip
    let tickers = vec!["DOGE", "ADA", "DOT"];
    let amounts = vec![10000.0, 500.0, 50.0];

    let portfolio2: HashMap<&str, f64> = tickers
        .into_iter()
        .zip(amounts.into_iter())
        .collect();

    println!("Portfolio 2: {:?}", portfolio2);
}
```

## Доступ к элементам

### Получение значения по ключу

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);

    // get() возвращает Option<&V>
    let btc_amount = portfolio.get("BTC");
    match btc_amount {
        Some(amount) => println!("BTC balance: {}", amount),
        None => println!("BTC not found in portfolio"),
    }

    // Использование if let
    if let Some(amount) = portfolio.get("ETH") {
        println!("ETH balance: {}", amount);
    }

    // unwrap_or для значения по умолчанию
    let xrp_amount = portfolio.get("XRP").unwrap_or(&0.0);
    println!("XRP balance: {}", xrp_amount);

    // copied() для получения копии значения
    let eth_balance: f64 = portfolio.get("ETH").copied().unwrap_or(0.0);
    println!("ETH balance (copied): {}", eth_balance);
}
```

### Проверка наличия ключа

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);

    // contains_key проверяет наличие ключа
    if portfolio.contains_key("BTC") {
        println!("У вас есть Bitcoin!");
    }

    if !portfolio.contains_key("DOGE") {
        println!("Dogecoin отсутствует в портфеле");
    }
}
```

## Практический пример: Трекер портфеля

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Начальные позиции
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("USDT"), 10000.0);

    // Цены активов (в USDT)
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2500.0);
    prices.insert(String::from("USDT"), 1.0);

    // Считаем общую стоимость портфеля
    let mut total_value = 0.0;

    println!("=== Состав портфеля ===");
    for (asset, amount) in &portfolio {
        let price = prices.get(asset).unwrap_or(&0.0);
        let value = amount * price;
        total_value += value;

        println!("{}: {} × ${:.2} = ${:.2}", asset, amount, price, value);
    }

    println!("========================");
    println!("Общая стоимость: ${:.2}", total_value);
}
```

## Изменение HashMap

### Обновление значения

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    println!("До: {:?}", portfolio);

    // insert перезаписывает существующее значение
    let old_value = portfolio.insert(String::from("BTC"), 1.0);
    println!("Старое значение: {:?}", old_value);  // Some(0.5)

    println!("После: {:?}", portfolio);
}
```

### Модификация через get_mut

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    // Получаем мутабельную ссылку и увеличиваем
    if let Some(amount) = portfolio.get_mut("BTC") {
        *amount += 0.25;  // Докупили 0.25 BTC
    }

    println!("BTC after purchase: {:?}", portfolio.get("BTC"));
}
```

### Entry API — условная вставка

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);

    // or_insert: вставить, только если ключа нет
    portfolio.entry(String::from("BTC")).or_insert(10.0);  // Не изменится
    portfolio.entry(String::from("ETH")).or_insert(5.0);   // Вставится

    println!("Portfolio: {:?}", portfolio);

    // or_insert_with: ленивое вычисление
    portfolio
        .entry(String::from("SOL"))
        .or_insert_with(|| {
            println!("Вычисляем начальное значение SOL...");
            100.0
        });

    println!("Portfolio: {:?}", portfolio);
}
```

### Entry API — обновление на основе старого значения

```rust
use std::collections::HashMap;

fn main() {
    let trades = vec![
        ("BTC", 0.1),
        ("ETH", 5.0),
        ("BTC", 0.2),
        ("BTC", -0.05),  // продажа
        ("ETH", 2.5),
    ];

    let mut portfolio: HashMap<&str, f64> = HashMap::new();

    // Агрегируем все сделки
    for (asset, amount) in trades {
        portfolio
            .entry(asset)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
    }

    println!("Финальный портфель: {:?}", portfolio);
    // BTC: 0.1 + 0.2 - 0.05 = 0.25
    // ETH: 5.0 + 2.5 = 7.5
}
```

## Итерация по HashMap

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);
    portfolio.insert("SOL", 100.0);

    // Итерация по парам (ключ, значение)
    println!("=== Все активы ===");
    for (asset, amount) in &portfolio {
        println!("{}: {}", asset, amount);
    }

    // Только ключи
    println!("\n=== Тикеры ===");
    for asset in portfolio.keys() {
        println!("{}", asset);
    }

    // Только значения
    println!("\n=== Количества ===");
    for amount in portfolio.values() {
        println!("{}", amount);
    }

    // Мутабельная итерация по значениям
    println!("\n=== Удвоение позиций ===");
    for amount in portfolio.values_mut() {
        *amount *= 2.0;
    }
    println!("После удвоения: {:?}", portfolio);
}
```

## Практический пример: Обработка ордеров

```rust
use std::collections::HashMap;

fn main() {
    // Симуляция книги ордеров: цена -> объём
    let mut order_book: HashMap<String, f64> = HashMap::new();

    // Добавляем ордера на покупку
    let buy_orders = vec![
        ("41000.00", 0.5),
        ("40500.00", 1.2),
        ("40000.00", 2.0),
        ("41000.00", 0.3),  // Добавляем к существующему уровню
    ];

    for (price, volume) in buy_orders {
        order_book
            .entry(String::from(price))
            .and_modify(|v| *v += volume)
            .or_insert(volume);
    }

    println!("=== Книга ордеров (BID) ===");
    for (price, volume) in &order_book {
        println!("${}: {} BTC", price, volume);
    }

    // Общий объём на покупку
    let total_bid_volume: f64 = order_book.values().sum();
    println!("\nОбщий объём BID: {} BTC", total_bid_volume);
}
```

## Практический пример: Подсчёт сделок по активам

```rust
use std::collections::HashMap;

fn main() {
    let trades = vec![
        "BTC", "ETH", "BTC", "SOL", "BTC", "ETH",
        "DOGE", "BTC", "SOL", "ETH", "BTC", "BTC",
    ];

    let mut trade_counts: HashMap<&str, u32> = HashMap::new();

    // Подсчитываем количество сделок по каждому активу
    for trade in &trades {
        let count = trade_counts.entry(trade).or_insert(0);
        *count += 1;
    }

    println!("=== Статистика сделок ===");
    for (asset, count) in &trade_counts {
        println!("{}: {} сделок", asset, count);
    }

    // Находим самый торгуемый актив
    if let Some((top_asset, top_count)) = trade_counts.iter().max_by_key(|&(_, count)| count) {
        println!("\nСамый активный: {} ({} сделок)", top_asset, top_count);
    }
}
```

## Практический пример: Конвертация валют

```rust
use std::collections::HashMap;

fn main() {
    // Курсы обмена к USD
    let mut exchange_rates: HashMap<&str, f64> = HashMap::new();
    exchange_rates.insert("BTC", 42000.0);
    exchange_rates.insert("ETH", 2500.0);
    exchange_rates.insert("SOL", 100.0);
    exchange_rates.insert("USDT", 1.0);
    exchange_rates.insert("EUR", 1.08);

    // Портфель в разных валютах
    let mut portfolio: HashMap<&str, f64> = HashMap::new();
    portfolio.insert("BTC", 0.5);
    portfolio.insert("ETH", 10.0);
    portfolio.insert("EUR", 5000.0);

    // Конвертируем всё в USD
    let mut total_usd = 0.0;

    println!("=== Конвертация в USD ===");
    for (currency, amount) in &portfolio {
        if let Some(rate) = exchange_rates.get(currency) {
            let usd_value = amount * rate;
            total_usd += usd_value;
            println!("{} {} × ${:.2} = ${:.2}", amount, currency, rate, usd_value);
        } else {
            println!("{} {}: курс не найден!", amount, currency);
        }
    }

    println!("========================");
    println!("Итого в USD: ${:.2}", total_usd);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `HashMap<K, V>` | Коллекция пар ключ-значение |
| `insert()` | Вставка или обновление элемента |
| `get()` | Получение значения (возвращает Option) |
| `get_mut()` | Получение мутабельной ссылки |
| `contains_key()` | Проверка наличия ключа |
| `entry().or_insert()` | Условная вставка |
| `keys()`, `values()` | Итераторы по ключам/значениям |

## Упражнения

1. **Трекер P&L**: Создай HashMap для отслеживания прибыли/убытка по каждому активу. Реализуй функции добавления сделки и вывода общего P&L.

2. **Балансы по биржам**: Создай вложенный HashMap `HashMap<String, HashMap<String, f64>>` для хранения балансов по биржам (биржа -> актив -> количество).

3. **Топ-3 позиции**: Напиши функцию, которая принимает портфель и цены, возвращает 3 самые крупные позиции по стоимости.

4. **История цен**: Создай HashMap для хранения последних 10 цен каждого актива. Реализуй функцию расчёта средней цены.

## Домашнее задание

1. Реализуй простой order book с уровнями цен и объёмами:
   ```rust
   // Структура: HashMap<цена, объём>
   // Функции: add_order, remove_order, get_best_bid, get_best_ask
   ```

2. Создай систему отслеживания позиций:
   - Добавление/удаление позиций
   - Расчёт средней цены входа
   - Расчёт нереализованного P&L

3. Напиши конвертер валют с поддержкой:
   - Прямой конвертации (BTC → USD)
   - Обратной конвертации (USD → BTC)
   - Кросс-курсов (ETH → BTC через USD)

## Навигация

[← Предыдущий день](../081-iterating-vec-orders/ru.md) | [Следующий день →](../083-hashmap-methods/ru.md)
