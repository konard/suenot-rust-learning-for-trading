# День 80: Методы Vec — push, pop, get, len

## Аналогия из трейдинга

В реальной торговле данные постоянно меняются:
- **Новые сделки** добавляются в журнал в реальном времени
- **Ордера** исполняются и удаляются из книги заявок
- **Ценовая история** растёт с каждой новой свечой
- **Портфель** динамически меняется при покупке/продаже активов

В отличие от массивов с фиксированным размером, `Vec` (вектор) — это **динамическая коллекция**, которая может расти и уменьшаться во время выполнения программы.

## Создание вектора

```rust
fn main() {
    // Пустой вектор с указанием типа
    let mut prices: Vec<f64> = Vec::new();

    // Вектор с начальными значениями (макрос vec!)
    let closes = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // Вектор с повторяющимся значением
    let zeros: Vec<f64> = vec![0.0; 10];  // 10 нулей

    // С заданной ёмкостью (оптимизация)
    let mut orders: Vec<f64> = Vec::with_capacity(100);

    println!("Closes: {:?}", closes);
    println!("Zeros: {:?}", zeros);
    println!("Orders capacity: {}", orders.capacity());
}
```

## Метод push() — добавление элементов

`push()` добавляет элемент в конец вектора:

```rust
fn main() {
    let mut trade_log: Vec<f64> = Vec::new();

    // Добавляем сделки по мере их исполнения
    trade_log.push(42150.50);  // Первая сделка
    trade_log.push(42155.00);  // Вторая сделка
    trade_log.push(42148.75);  // Третья сделка

    println!("Trade log: {:?}", trade_log);
    println!("Total trades: {}", trade_log.len());
}
```

### Практический пример: лог цен в реальном времени

```rust
fn main() {
    let mut price_feed: Vec<f64> = Vec::new();

    // Симуляция получения цен с биржи
    let incoming_prices = [42000.0, 42005.0, 41998.0, 42010.0, 42015.0];

    for price in incoming_prices {
        price_feed.push(price);
        println!("Received: ${:.2}, Total ticks: {}", price, price_feed.len());
    }

    println!("\nFull price feed: {:?}", price_feed);
}
```

## Метод pop() — удаление последнего элемента

`pop()` удаляет и возвращает последний элемент. Возвращает `Option<T>`:

```rust
fn main() {
    let mut order_book: Vec<f64> = vec![100.0, 150.0, 200.0, 250.0];

    println!("Order book: {:?}", order_book);

    // Исполняем последний ордер
    match order_book.pop() {
        Some(order) => println!("Executed order: ${}", order),
        None => println!("Order book is empty!"),
    }

    println!("After execution: {:?}", order_book);

    // Безопасно работает с пустым вектором
    let mut empty: Vec<f64> = Vec::new();
    let result = empty.pop();
    println!("Pop from empty: {:?}", result);  // None
}
```

### Практический пример: LIFO-стек ордеров

```rust
fn main() {
    let mut pending_orders: Vec<(String, f64, f64)> = Vec::new();

    // Добавляем ордера (символ, цена, количество)
    pending_orders.push(("BTC".to_string(), 42000.0, 0.5));
    pending_orders.push(("ETH".to_string(), 2200.0, 2.0));
    pending_orders.push(("BTC".to_string(), 42100.0, 0.3));

    println!("Pending orders: {}", pending_orders.len());

    // Отменяем последний ордер
    if let Some((symbol, price, qty)) = pending_orders.pop() {
        println!("Cancelled: {} {} @ ${}", qty, symbol, price);
    }

    println!("Remaining orders: {}", pending_orders.len());
}
```

## Метод len() — длина вектора

`len()` возвращает количество элементов:

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL", "ADA"];

    println!("Assets in portfolio: {}", portfolio.len());

    // Проверка на пустоту
    let empty_portfolio: Vec<&str> = Vec::new();
    println!("Is empty: {}", empty_portfolio.is_empty());
    println!("Has assets: {}", !portfolio.is_empty());

    // Ёмкость vs длина
    let mut prices = Vec::with_capacity(100);
    prices.push(42000.0);
    prices.push(42100.0);

    println!("Length: {}", prices.len());       // 2
    println!("Capacity: {}", prices.capacity()); // 100
}
```

### Практический пример: проверка достаточности данных для индикатора

```rust
fn main() {
    let mut candles: Vec<f64> = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    let sma_period = 5;
    let ema_period = 12;
    let rsi_period = 14;

    println!("Candles available: {}", candles.len());
    println!("Can calculate SMA-{}: {}", sma_period, candles.len() >= sma_period);
    println!("Can calculate EMA-{}: {}", ema_period, candles.len() >= ema_period);
    println!("Can calculate RSI-{}: {}", rsi_period, candles.len() >= rsi_period);

    // Добавляем больше данных
    for i in 0..10 {
        candles.push(42000.0 + (i as f64 * 50.0));
    }

    println!("\nAfter adding data:");
    println!("Candles available: {}", candles.len());
    println!("Can calculate RSI-{}: {}", rsi_period, candles.len() >= rsi_period);
}
```

## Метод get() — безопасный доступ к элементам

`get()` возвращает `Option<&T>`, предотвращая панику при выходе за границы:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Безопасный доступ
    match prices.get(2) {
        Some(price) => println!("Price at index 2: ${}", price),
        None => println!("Index out of bounds"),
    }

    // Доступ к несуществующему индексу
    match prices.get(100) {
        Some(price) => println!("Price: ${}", price),
        None => println!("No price at index 100"),
    }

    // Сокращённая форма с if let
    if let Some(last) = prices.get(prices.len() - 1) {
        println!("Latest price: ${}", last);
    }

    // ОПАСНО: прямой доступ по индексу может вызвать панику!
    // println!("{}", prices[100]);  // panic!
}
```

### Практический пример: безопасное получение скользящей средней

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Получаем цены для расчёта SMA-3
    let sma_window = 3;

    // Безопасно получаем последние N цен
    if prices.len() >= sma_window {
        let start_index = prices.len() - sma_window;
        let mut sum = 0.0;

        for i in start_index..prices.len() {
            if let Some(price) = prices.get(i) {
                sum += price;
            }
        }

        let sma = sum / sma_window as f64;
        println!("SMA-{}: ${:.2}", sma_window, sma);
    } else {
        println!("Not enough data for SMA-{}", sma_window);
    }
}
```

## Комбинирование методов

```rust
fn main() {
    let mut price_history: Vec<f64> = Vec::new();

    // Заполняем историю
    let ticks = [42000.0, 42050.0, 42025.0, 42100.0, 42080.0, 42150.0];

    for tick in ticks {
        price_history.push(tick);

        // Выводим статистику после каждого тика
        let len = price_history.len();
        let last = price_history.get(len - 1).unwrap();

        print!("Tick #{}: ${:.2}", len, last);

        // Если есть достаточно данных, считаем изменение
        if len >= 2 {
            if let Some(prev) = price_history.get(len - 2) {
                let change = (last - prev) / prev * 100.0;
                let sign = if change >= 0.0 { "+" } else { "" };
                print!(" ({}{}%)", sign, format!("{:.2}", change));
            }
        }

        println!();
    }
}
```

## Практический пример: журнал сделок

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let mut trades: Vec<Trade> = Vec::new();

    // Добавляем сделки
    trades.push(Trade {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    });

    trades.push(Trade {
        symbol: "ETH/USDT".to_string(),
        side: "BUY".to_string(),
        price: 2200.0,
        quantity: 2.0,
    });

    trades.push(Trade {
        symbol: "BTC/USDT".to_string(),
        side: "SELL".to_string(),
        price: 42500.0,
        quantity: 0.5,
    });

    // Статистика
    println!("Total trades: {}", trades.len());

    // Последняя сделка
    if let Some(last) = trades.get(trades.len() - 1) {
        println!("Last trade: {} {} {} @ ${}",
            last.side, last.quantity, last.symbol, last.price);
    }

    // Рассчитываем P&L по BTC
    let mut btc_pnl = 0.0;
    for trade in &trades {
        if trade.symbol == "BTC/USDT" {
            if trade.side == "BUY" {
                btc_pnl -= trade.price * trade.quantity;
            } else {
                btc_pnl += trade.price * trade.quantity;
            }
        }
    }

    println!("BTC P&L: ${:.2}", btc_pnl);
}
```

## Практический пример: управление книгой ордеров

```rust
fn main() {
    let mut bid_orders: Vec<f64> = Vec::new();
    let mut ask_orders: Vec<f64> = Vec::new();

    // Добавляем ордера на покупку (bid)
    bid_orders.push(41900.0);
    bid_orders.push(41850.0);
    bid_orders.push(41800.0);

    // Добавляем ордера на продажу (ask)
    ask_orders.push(42000.0);
    ask_orders.push(42050.0);
    ask_orders.push(42100.0);

    // Лучшие цены
    let best_bid = bid_orders.get(0);
    let best_ask = ask_orders.get(0);

    match (best_bid, best_ask) {
        (Some(bid), Some(ask)) => {
            println!("Best Bid: ${}", bid);
            println!("Best Ask: ${}", ask);
            println!("Spread: ${:.2} ({:.4}%)",
                ask - bid,
                (ask - bid) / bid * 100.0);
        }
        _ => println!("Order book is incomplete"),
    }

    // Исполняем ордер (удаляем лучший ask)
    if let Some(executed) = ask_orders.pop() {
        println!("\nMarket buy executed at ${}", executed);
    }

    // Глубина рынка
    println!("\nBid depth: {} orders", bid_orders.len());
    println!("Ask depth: {} orders", ask_orders.len());
}
```

## Практический пример: скользящее окно цен

```rust
fn main() {
    let mut price_window: Vec<f64> = Vec::new();
    let window_size = 5;

    let price_stream = [
        42000.0, 42050.0, 42025.0, 42100.0, 42080.0,
        42150.0, 42120.0, 42200.0, 42180.0, 42250.0
    ];

    for price in price_stream {
        // Добавляем новую цену
        price_window.push(price);

        // Удаляем старые данные если окно переполнено
        while price_window.len() > window_size {
            price_window.remove(0);  // Удаляем первый элемент
        }

        // Рассчитываем SMA если окно заполнено
        if price_window.len() == window_size {
            let sum: f64 = price_window.iter().sum();
            let sma = sum / window_size as f64;
            println!("Price: ${:.2} | SMA-{}: ${:.2} | Window: {:?}",
                price, window_size, sma, price_window);
        } else {
            println!("Price: ${:.2} | Collecting data... ({}/{})",
                price, price_window.len(), window_size);
        }
    }
}
```

## Дополнительные полезные методы

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // first() и last()
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // insert() - вставка по индексу
    prices.insert(0, 41800.0);  // В начало
    println!("After insert: {:?}", prices);

    // remove() - удаление по индексу
    let removed = prices.remove(2);
    println!("Removed: {}, Now: {:?}", removed, prices);

    // clear() - очистка
    prices.clear();
    println!("After clear: {:?}, len: {}", prices, prices.len());

    // extend() - добавление нескольких элементов
    prices.extend([42000.0, 42100.0, 42200.0]);
    println!("After extend: {:?}", prices);

    // contains()
    println!("Contains 42100: {}", prices.contains(&42100.0));
}
```

## Что мы узнали

| Метод | Описание | Возвращает |
|-------|----------|------------|
| `push(val)` | Добавляет элемент в конец | `()` |
| `pop()` | Удаляет и возвращает последний элемент | `Option<T>` |
| `len()` | Возвращает количество элементов | `usize` |
| `get(i)` | Безопасный доступ по индексу | `Option<&T>` |
| `is_empty()` | Проверка на пустоту | `bool` |
| `capacity()` | Текущая ёмкость | `usize` |
| `first()` | Первый элемент | `Option<&T>` |
| `last()` | Последний элемент | `Option<&T>` |

## Домашнее задание

1. **Симуляция торгового тикера**: Создай вектор, который накапливает цены из "потока данных". После каждого тика выводи текущую цену, среднюю цену, минимум и максимум.

2. **Стек ордеров**: Реализуй систему ордеров с использованием `push` и `pop`. Добавь функции для добавления ордера, отмены последнего ордера и просмотра последнего ордера без удаления (peek).

3. **Скользящая средняя**: Реализуй функцию, которая поддерживает вектор фиксированного размера (например, 20 элементов) и при добавлении нового элемента удаляет самый старый.

4. **Журнал сделок с фильтрацией**: Создай вектор сделок и реализуй функции:
   - Подсчёт общего количества сделок
   - Получение последних N сделок
   - Фильтрация сделок по символу

## Навигация

[← Предыдущий день](../079-vec-introduction/ru.md) | [Следующий день →](../081-vec-iteration/ru.md)
