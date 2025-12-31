# День 79: Vec — динамический список сделок

## Аналогия из трейдинга

В реальном трейдинге количество данных постоянно меняется:
- Новые сделки добавляются в журнал
- Ордера отменяются и создаются
- Портфель пополняется новыми активами
- История цен растёт с каждой свечой

Массивы с фиксированным размером здесь не подходят — нужен **динамический список**, который может расти и уменьшаться. В Rust это `Vec<T>` — вектор.

## Что такое Vec?

`Vec<T>` (вектор) — это:
- Динамический массив, размер которого может меняться
- Хранит элементы одного типа `T`
- Выделяет память в куче (heap)
- Основная коллекция для списков в Rust

## Создание вектора

```rust
fn main() {
    // Пустой вектор с указанием типа
    let trades: Vec<f64> = Vec::new();
    println!("Empty trades: {:?}", trades);

    // Макрос vec! — удобный способ создания
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];
    println!("Prices: {:?}", prices);

    // Вектор с начальной ёмкостью (оптимизация)
    let mut orders: Vec<String> = Vec::with_capacity(100);
    println!("Orders capacity: {}", orders.capacity());

    // Вектор из повторяющихся значений
    let zeros = vec![0.0; 10];
    println!("Zeros: {:?}", zeros);
}
```

## Добавление элементов

```rust
fn main() {
    let mut trades: Vec<f64> = Vec::new();

    // push — добавить в конец
    trades.push(42000.0);
    trades.push(42150.0);
    trades.push(41900.0);

    println!("Trades: {:?}", trades);
    println!("Count: {}", trades.len());

    // Добавляем ордера на покупку
    let mut buy_orders = vec![100.0, 200.0];
    buy_orders.push(150.0);
    buy_orders.push(175.0);

    println!("Buy orders: {:?}", buy_orders);
}
```

## Доступ к элементам

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // По индексу (может вызвать panic!)
    println!("First: {}", prices[0]);
    println!("Last: {}", prices[prices.len() - 1]);

    // Безопасный доступ с get()
    match prices.get(2) {
        Some(price) => println!("Price at index 2: {}", price),
        None => println!("Index out of bounds"),
    }

    // Безопасно для несуществующего индекса
    if let Some(price) = prices.get(100) {
        println!("Price: {}", price);
    } else {
        println!("No price at index 100");
    }

    // first() и last()
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());
}
```

## Удаление элементов

```rust
fn main() {
    let mut orders = vec!["BUY BTC", "SELL ETH", "BUY SOL", "SELL BTC"];
    println!("Orders: {:?}", orders);

    // pop — удалить последний элемент
    let last = orders.pop();
    println!("Removed: {:?}", last);
    println!("Orders: {:?}", orders);

    // remove — удалить по индексу
    let removed = orders.remove(1);  // Удаляем "SELL ETH"
    println!("Removed: {}", removed);
    println!("Orders: {:?}", orders);

    // clear — очистить вектор
    orders.clear();
    println!("After clear: {:?}", orders);
    println!("Is empty: {}", orders.is_empty());
}
```

## Изменение элементов

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0];
    println!("Before: {:?}", prices);

    // Изменение по индексу
    prices[0] = 42050.0;
    prices[2] = 42000.0;
    println!("After: {:?}", prices);

    // Изменение через итератор
    for price in &mut prices {
        *price *= 1.01;  // Увеличиваем на 1%
    }
    println!("After 1% increase: {:?}", prices);
}
```

## Итерация по вектору

```rust
fn main() {
    let trades = vec![
        ("BTC", 42000.0, 0.5),
        ("ETH", 2200.0, 2.0),
        ("SOL", 100.0, 10.0),
    ];

    // Простой for (перемещает владение!)
    // for trade in trades { ... }

    // По ссылке — читаем
    println!("=== Portfolio ===");
    for (symbol, price, amount) in &trades {
        let value = price * amount;
        println!("{}: {} @ ${} = ${}", symbol, amount, price, value);
    }

    // С индексом
    println!("\n=== With index ===");
    for (i, trade) in trades.iter().enumerate() {
        println!("[{}] {:?}", i, trade);
    }
}
```

## Практический пример: журнал сделок

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,      // "BUY" или "SELL"
    price: f64,
    quantity: f64,
    timestamp: u64,
}

impl Trade {
    fn new(symbol: &str, side: &str, price: f64, quantity: f64, timestamp: u64) -> Self {
        Trade {
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            timestamp,
        }
    }

    fn value(&self) -> f64 {
        self.price * self.quantity
    }
}

fn main() {
    let mut trade_log: Vec<Trade> = Vec::new();

    // Добавляем сделки
    trade_log.push(Trade::new("BTC", "BUY", 42000.0, 0.5, 1000));
    trade_log.push(Trade::new("ETH", "BUY", 2200.0, 2.0, 1001));
    trade_log.push(Trade::new("BTC", "SELL", 43000.0, 0.3, 1002));
    trade_log.push(Trade::new("SOL", "BUY", 100.0, 10.0, 1003));

    println!("=== Trade Log ({} trades) ===", trade_log.len());
    for trade in &trade_log {
        println!("{} {} {} @ ${:.2} = ${:.2}",
            trade.side, trade.quantity, trade.symbol,
            trade.price, trade.value());
    }

    // Общий объём сделок
    let total_volume: f64 = trade_log.iter()
        .map(|t| t.value())
        .sum();
    println!("\nTotal volume: ${:.2}", total_volume);
}
```

## Практический пример: управление ордерами

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    status: String,
}

fn main() {
    let mut order_book: Vec<Order> = Vec::new();

    // Создаём ордера
    order_book.push(Order {
        id: 1, symbol: "BTC".to_string(), side: "BUY".to_string(),
        price: 41000.0, quantity: 0.5, status: "OPEN".to_string()
    });
    order_book.push(Order {
        id: 2, symbol: "ETH".to_string(), side: "SELL".to_string(),
        price: 2300.0, quantity: 2.0, status: "OPEN".to_string()
    });
    order_book.push(Order {
        id: 3, symbol: "BTC".to_string(), side: "BUY".to_string(),
        price: 40500.0, quantity: 1.0, status: "OPEN".to_string()
    });

    println!("=== Open Orders ===");
    for order in &order_book {
        println!("#{}: {} {} {} @ ${}",
            order.id, order.side, order.quantity, order.symbol, order.price);
    }

    // Отменяем ордер #2
    if let Some(pos) = order_book.iter().position(|o| o.id == 2) {
        let cancelled = order_book.remove(pos);
        println!("\nCancelled order #{}", cancelled.id);
    }

    // Находим все BTC ордера
    let btc_orders: Vec<&Order> = order_book.iter()
        .filter(|o| o.symbol == "BTC")
        .collect();

    println!("\n=== BTC Orders ===");
    for order in btc_orders {
        println!("#{}: {} @ ${}", order.id, order.side, order.price);
    }
}
```

## Практический пример: анализ цен

```rust
fn main() {
    // История цен BTC
    let mut prices: Vec<f64> = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    // Добавляем новые цены
    prices.push(42600.0);
    prices.push(42550.0);
    prices.push(42700.0);

    println!("Price history ({} candles):", prices.len());

    // Расчёт SMA
    let sma5 = calculate_sma(&prices, 5);
    let sma10 = calculate_sma(&prices, 10);

    println!("SMA-5: ${:.2}", sma5);
    println!("SMA-10: ${:.2}", sma10);

    // Текущая цена vs SMA
    let current = *prices.last().unwrap();
    if current > sma5 {
        println!("Price above SMA-5 - bullish signal");
    } else {
        println!("Price below SMA-5 - bearish signal");
    }

    // Волатильность
    let volatility = calculate_volatility(&prices);
    println!("Volatility: {:.2}%", volatility);
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return 0.0;
    }
    let sum: f64 = prices.iter().rev().take(period).sum();
    sum / period as f64
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance: f64 = prices.iter()
        .map(|p| (p - mean).powi(2))
        .sum::<f64>() / prices.len() as f64;

    (variance.sqrt() / mean) * 100.0
}
```

## Практический пример: управление портфелем

```rust
#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Position {
    fn value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    fn pnl(&self, current_price: f64) -> f64 {
        self.quantity * (current_price - self.avg_price)
    }

    fn pnl_percent(&self, current_price: f64) -> f64 {
        (current_price - self.avg_price) / self.avg_price * 100.0
    }
}

fn main() {
    let mut portfolio: Vec<Position> = Vec::new();

    // Добавляем позиции
    portfolio.push(Position {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        avg_price: 40000.0,
    });
    portfolio.push(Position {
        symbol: "ETH".to_string(),
        quantity: 5.0,
        avg_price: 2000.0,
    });
    portfolio.push(Position {
        symbol: "SOL".to_string(),
        quantity: 50.0,
        avg_price: 80.0,
    });

    // Текущие цены
    let prices = [("BTC", 42000.0), ("ETH", 2200.0), ("SOL", 100.0)];

    println!("=== Portfolio ===");
    let mut total_value = 0.0;
    let mut total_pnl = 0.0;

    for pos in &portfolio {
        // Находим текущую цену
        let current = prices.iter()
            .find(|(s, _)| *s == pos.symbol)
            .map(|(_, p)| *p)
            .unwrap_or(0.0);

        let value = pos.value(current);
        let pnl = pos.pnl(current);
        let pnl_pct = pos.pnl_percent(current);

        println!("{}: {} @ ${:.2} -> ${:.2} | P&L: ${:.2} ({:+.2}%)",
            pos.symbol, pos.quantity, pos.avg_price,
            value, pnl, pnl_pct);

        total_value += value;
        total_pnl += pnl;
    }

    println!("\nTotal Value: ${:.2}", total_value);
    println!("Total P&L: ${:.2}", total_pnl);
}
```

## Полезные методы Vec

```rust
fn main() {
    let mut prices = vec![42000.0, 41500.0, 42500.0, 41000.0, 43000.0];

    // Информация о векторе
    println!("Length: {}", prices.len());
    println!("Capacity: {}", prices.capacity());
    println!("Is empty: {}", prices.is_empty());

    // Поиск
    println!("Contains 42000: {}", prices.contains(&42000.0));

    // Сортировка
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("Sorted: {:?}", prices);

    // Обратный порядок
    prices.reverse();
    println!("Reversed: {:?}", prices);

    // Дедупликация (после сортировки)
    let mut data = vec![1, 2, 2, 3, 3, 3, 4];
    data.dedup();
    println!("Deduped: {:?}", data);

    // Retain — оставить элементы по условию
    let mut trades = vec![100.0, -50.0, 200.0, -100.0, 150.0];
    trades.retain(|&x| x > 0.0);  // Только прибыльные
    println!("Profitable: {:?}", trades);
}
```

## Объединение векторов

```rust
fn main() {
    let mut btc_trades = vec![42000.0, 42100.0, 42200.0];
    let eth_trades = vec![2200.0, 2250.0, 2300.0];

    // extend — добавить все элементы
    // btc_trades.extend(eth_trades);

    // append — переместить все элементы (eth_trades станет пустым)
    let mut all_trades = btc_trades.clone();
    let mut eth_copy = eth_trades.clone();
    all_trades.append(&mut eth_copy);

    println!("All trades: {:?}", all_trades);
    println!("ETH copy after append: {:?}", eth_copy);

    // concat через итераторы
    let combined: Vec<f64> = btc_trades.iter()
        .chain(eth_trades.iter())
        .cloned()
        .collect();
    println!("Combined: {:?}", combined);
}
```

## Преобразование в слайс

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Vec автоматически приводится к слайсу
    print_prices(&prices);

    // Явное преобразование
    let slice: &[f64] = &prices[..];
    println!("Slice: {:?}", slice);

    // Часть вектора
    let last_three = &prices[2..];
    println!("Last 3: {:?}", last_three);
}

fn print_prices(prices: &[f64]) {
    println!("Prices ({}):", prices.len());
    for price in prices {
        println!("  ${:.2}", price);
    }
}
```

## Vec vs Array — когда что использовать

| Характеристика | Array `[T; N]` | Vec `Vec<T>` |
|---------------|----------------|--------------|
| Размер | Фиксированный | Динамический |
| Память | Стек | Куча |
| Производительность | Немного быстрее | Гибче |
| Использование | Известный размер | Неизвестный размер |

**Используй Array когда:**
- Размер известен на этапе компиляции
- Нужна максимальная производительность
- Пример: OHLC свеча `[f64; 4]`

**Используй Vec когда:**
- Размер неизвестен или меняется
- Данные приходят динамически
- Пример: история сделок, ордера

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Vec::new()` | Создать пустой вектор |
| `vec![...]` | Макрос для создания вектора |
| `push()` | Добавить элемент в конец |
| `pop()` | Удалить и вернуть последний |
| `get(i)` | Безопасный доступ по индексу |
| `len()` | Количество элементов |
| `iter()` | Итератор по элементам |

## Домашнее задание

1. **Журнал сделок**: Создай структуру `TradeLog` с методами:
   - `add_trade()` — добавить сделку
   - `total_volume()` — общий объём
   - `profitable_trades()` — список прибыльных сделок
   - `by_symbol()` — сделки по тикеру

2. **Книга ордеров**: Реализуй простую книгу ордеров:
   - Добавление лимитных ордеров
   - Отмена по ID
   - Поиск лучшей цены bid/ask
   - Matching (исполнение) при пересечении

3. **Расчёт индикаторов**: Напиши функции для Vec<f64>:
   - `sma(period)` — скользящая средняя
   - `ema(period)` — экспоненциальная средняя
   - `rsi(period)` — индекс относительной силы

4. **Управление рисками**: Создай трекер позиций с:
   - Расчётом общего риска портфеля
   - Нахождением позиций с убытком > 5%
   - Ребалансировкой по весам

## Навигация

[← День 78](../078-hashmap-asset-lookup/ru.md) | [День 80 →](../080-hashset-unique-tickers/ru.md)
