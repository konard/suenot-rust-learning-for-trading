# День 60: Структуры — создание типа Order

## Аналогия из трейдинга

Когда ты размещаешь ордер на бирже, ты указываешь множество параметров: символ, направление (покупка/продажа), количество, цену, тип ордера. Все эти данные связаны между собой — это **один ордер**. Структуры в Rust позволяют объединить связанные данные в единый тип.

Представь торговый терминал: каждый ордер — это не просто набор разрозненных чисел и строк, а **цельный объект** со всеми его характеристиками.

## Что такое структура?

Структура (`struct`) — это пользовательский тип данных, который группирует связанные значения под одним именем.

```rust
fn main() {
    // Без структуры — разрозненные переменные
    let symbol = "BTC/USDT";
    let side = "buy";
    let quantity = 0.5;
    let price = 42000.0;

    // Со структурой — всё в одном месте
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
    };

    println!("Order: {} {} {} @ {}", order.side, order.quantity, order.symbol, order.price);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

## Определение структуры

```rust
struct Order {
    symbol: String,      // Торговая пара
    side: String,        // "buy" или "sell"
    quantity: f64,       // Количество
    price: f64,          // Цена
    order_type: String,  // "limit", "market", "stop"
    timestamp: u64,      // Время создания (Unix timestamp)
}
```

**Важно:**
- Имя структуры пишется в `PascalCase` (каждое слово с большой буквы)
- Поля перечисляются через запятую
- После последнего поля запятая опциональна, но рекомендуется

## Создание экземпляра структуры

```rust
fn main() {
    let order = Order {
        symbol: String::from("ETH/USDT"),
        side: String::from("buy"),
        quantity: 2.5,
        price: 2500.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    println!("Created order for {}", order.symbol);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

## Доступ к полям

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.1,
        price: 42000.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    // Доступ через точку
    println!("Symbol: {}", order.symbol);
    println!("Side: {}", order.side);
    println!("Quantity: {}", order.quantity);
    println!("Price: ${}", order.price);

    // Расчёт стоимости ордера
    let order_value = order.quantity * order.price;
    println!("Order value: ${:.2}", order_value);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

## Изменяемые структуры

```rust
fn main() {
    // Для изменения полей структура должна быть mut
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.1,
        price: 42000.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    println!("Original price: ${}", order.price);

    // Изменяем цену
    order.price = 41500.0;
    println!("Updated price: ${}", order.price);

    // Изменяем количество
    order.quantity = 0.2;
    println!("Updated quantity: {}", order.quantity);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

**Важно:** В Rust нельзя сделать только отдельные поля изменяемыми — вся структура либо изменяемая, либо нет.

## Сокращённая инициализация полей

Если переменная имеет то же имя, что и поле структуры:

```rust
fn main() {
    let symbol = String::from("SOL/USDT");
    let side = String::from("sell");
    let quantity = 10.0;
    let price = 100.0;

    // Полная форма
    let order1 = Order {
        symbol: symbol.clone(),
        side: side.clone(),
        quantity: quantity,
        price: price,
    };

    // Сокращённая форма (field init shorthand)
    let order2 = Order {
        symbol,  // Эквивалентно symbol: symbol
        side,    // Эквивалентно side: side
        quantity,
        price,
    };

    println!("Order 1: {} {}", order1.symbol, order1.quantity);
    println!("Order 2: {} {}", order2.symbol, order2.quantity);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

## Синтаксис обновления структуры

Создание новой структуры на основе существующей:

```rust
fn main() {
    let order1 = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
        order_type: String::from("limit"),
    };

    // Создаём похожий ордер, изменив только количество
    let order2 = Order {
        quantity: 1.0,          // Новое значение
        ..order1                // Остальное из order1
    };

    // Внимание: order1 больше недоступен, так как String был перемещён!
    println!("Order 2: {} @ {}", order2.quantity, order2.price);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
}
```

## Кортежные структуры

Структуры без именованных полей:

```rust
fn main() {
    // Кортежная структура для bid/ask
    let spread = BidAsk(42000.0, 42050.0);

    println!("Bid: {}, Ask: {}", spread.0, spread.1);
    println!("Spread: ${:.2}", spread.1 - spread.0);

    // Кортежная структура для OHLCV
    let candle = OHLCV(42000.0, 42500.0, 41800.0, 42300.0, 1500.0);

    println!("Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
             candle.0, candle.1, candle.2, candle.3, candle.4);
}

struct BidAsk(f64, f64);
struct OHLCV(f64, f64, f64, f64, f64);
```

## Unit-подобные структуры

Структуры без полей (маркерные типы):

```rust
fn main() {
    let _connected = Connected;
    let _disconnected = Disconnected;

    println!("Connection states defined");
}

struct Connected;
struct Disconnected;
```

## Практический пример: торговая система

```rust
fn main() {
    // Создаём несколько ордеров
    let orders = vec![
        Order {
            id: 1,
            symbol: String::from("BTC/USDT"),
            side: String::from("buy"),
            quantity: 0.5,
            price: 42000.0,
            status: String::from("filled"),
        },
        Order {
            id: 2,
            symbol: String::from("ETH/USDT"),
            side: String::from("sell"),
            quantity: 5.0,
            price: 2500.0,
            status: String::from("pending"),
        },
        Order {
            id: 3,
            symbol: String::from("BTC/USDT"),
            side: String::from("sell"),
            quantity: 0.3,
            price: 43000.0,
            status: String::from("filled"),
        },
    ];

    print_order_book(&orders);

    let stats = calculate_portfolio_stats(&orders);
    print_portfolio_stats(&stats);
}

struct Order {
    id: u64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

struct PortfolioStats {
    total_orders: usize,
    filled_orders: usize,
    pending_orders: usize,
    total_buy_value: f64,
    total_sell_value: f64,
}

fn print_order_book(orders: &[Order]) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                      ORDER BOOK                           ║");
    println!("╠═════╦════════════╦══════╦══════════╦════════════╦═════════╣");
    println!("║ ID  ║   Symbol   ║ Side ║ Quantity ║    Price   ║ Status  ║");
    println!("╠═════╬════════════╬══════╬══════════╬════════════╬═════════╣");

    for order in orders {
        println!("║ {:>3} ║ {:^10} ║ {:^4} ║ {:>8.4} ║ ${:>9.2} ║ {:^7} ║",
                 order.id,
                 order.symbol,
                 order.side,
                 order.quantity,
                 order.price,
                 order.status);
    }

    println!("╚═════╩════════════╩══════╩══════════╩════════════╩═════════╝");
}

fn calculate_portfolio_stats(orders: &[Order]) -> PortfolioStats {
    let total_orders = orders.len();
    let filled_orders = orders.iter().filter(|o| o.status == "filled").count();
    let pending_orders = orders.iter().filter(|o| o.status == "pending").count();

    let total_buy_value: f64 = orders
        .iter()
        .filter(|o| o.side == "buy" && o.status == "filled")
        .map(|o| o.quantity * o.price)
        .sum();

    let total_sell_value: f64 = orders
        .iter()
        .filter(|o| o.side == "sell" && o.status == "filled")
        .map(|o| o.quantity * o.price)
        .sum();

    PortfolioStats {
        total_orders,
        filled_orders,
        pending_orders,
        total_buy_value,
        total_sell_value,
    }
}

fn print_portfolio_stats(stats: &PortfolioStats) {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║          PORTFOLIO STATISTICS         ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Total Orders:      {:>18} ║", stats.total_orders);
    println!("║ Filled Orders:     {:>18} ║", stats.filled_orders);
    println!("║ Pending Orders:    {:>18} ║", stats.pending_orders);
    println!("║ Total Buy Value:   ${:>16.2} ║", stats.total_buy_value);
    println!("║ Total Sell Value:  ${:>16.2} ║", stats.total_sell_value);
    println!("║ Net Position:      ${:>16.2} ║", stats.total_sell_value - stats.total_buy_value);
    println!("╚═══════════════════════════════════════╝");
}
```

## Вложенные структуры

```rust
fn main() {
    let trade = Trade {
        id: 12345,
        order: Order {
            symbol: String::from("BTC/USDT"),
            side: String::from("buy"),
            quantity: 0.5,
            price: 42000.0,
        },
        execution: Execution {
            filled_quantity: 0.5,
            average_price: 41950.0,
            fee: 10.49,
            timestamp: 1703980800,
        },
    };

    println!("Trade #{}", trade.id);
    println!("Symbol: {}", trade.order.symbol);
    println!("Requested: {} @ ${}", trade.order.quantity, trade.order.price);
    println!("Executed: {} @ ${:.2}", trade.execution.filled_quantity, trade.execution.average_price);
    println!("Fee: ${:.2}", trade.execution.fee);

    // Расчёт PnL
    let cost = trade.order.quantity * trade.order.price;
    let value = trade.execution.filled_quantity * trade.execution.average_price;
    let pnl = value - cost - trade.execution.fee;
    println!("Slippage PnL: ${:.2}", pnl);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

struct Execution {
    filled_quantity: f64,
    average_price: f64,
    fee: f64,
    timestamp: u64,
}

struct Trade {
    id: u64,
    order: Order,
    execution: Execution,
}
```

## Отладка структур с #[derive(Debug)]

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
    };

    // Компактный вывод
    println!("Order: {:?}", order);

    // Красивый вывод
    println!("Order: {:#?}", order);
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

Вывод:
```
Order: Order { symbol: "BTC/USDT", side: "buy", quantity: 0.5, price: 42000.0 }
Order: Order {
    symbol: "BTC/USDT",
    side: "buy",
    quantity: 0.5,
    price: 42000.0,
}
```

## Что мы узнали

| Концепция | Описание | Пример |
|-----------|----------|--------|
| Определение | Группировка связанных данных | `struct Order { ... }` |
| Создание | Инициализация экземпляра | `Order { symbol: ... }` |
| Доступ | Чтение полей через точку | `order.price` |
| Изменение | Требует `mut` | `order.price = 42000.0` |
| Shorthand | Сокращённая инициализация | `Order { symbol, price }` |
| Update syntax | Копирование полей | `Order { price, ..order1 }` |
| Кортежные | Без имён полей | `struct Point(f64, f64)` |
| Debug | Вывод для отладки | `#[derive(Debug)]` |

## Домашнее задание

1. Создай структуру `Candle` с полями `open`, `high`, `low`, `close`, `volume`, `timestamp`. Напиши функцию, которая определяет тип свечи (бычья/медвежья/доджи).

2. Создай структуру `Position` с полями `symbol`, `side`, `entry_price`, `quantity`, `current_price`. Напиши функцию для расчёта нереализованного PnL.

3. Создай структуру `TradingBot` с вложенными структурами `Config` и `State`. Напиши функцию, которая обновляет состояние бота.

4. Используя структуру `Order`, создай вектор ордеров и напиши функции:
   - Фильтрация по символу
   - Расчёт общего объёма покупок и продаж
   - Поиск ордера с максимальной стоимостью

## Навигация

[← Предыдущий день](../059-shadowing-strategy-params/ru.md) | [Следующий день →](../061-struct-methods-order-actions/ru.md)
