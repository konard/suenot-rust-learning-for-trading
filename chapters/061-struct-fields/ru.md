# День 61: Поля структур — Цена, Объём, Направление

## Аналогия из трейдинга

Каждая сделка на бирже содержит несколько ключевых параметров:
- **Price** (цена) — по какой цене совершена сделка
- **Volume** (объём) — какое количество актива было куплено/продано
- **Direction** (направление) — покупка (Buy) или продажа (Sell)

Эти данные всегда идут вместе и описывают одну сущность — сделку. В Rust для группировки связанных данных с именованными полями используются **структуры** (structs).

## Что такое структура?

Структура — это пользовательский тип данных, который группирует связанные значения под именованными полями:

```rust
struct Trade {
    price: f64,
    volume: f64,
    is_buy: bool,
}

fn main() {
    let trade = Trade {
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };

    println!("Price: {}", trade.price);
    println!("Volume: {}", trade.volume);
    println!("Direction: {}", if trade.is_buy { "Buy" } else { "Sell" });
}
```

## Преимущества структур над кортежами

Сравним кортеж и структуру:

```rust
fn main() {
    // Кортеж — непонятно, что есть что
    let trade_tuple: (f64, f64, bool) = (42000.0, 0.5, true);
    println!("Price: {}", trade_tuple.0);  // Что такое .0?

    // Структура — всё понятно по именам
    let trade_struct = Trade {
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };
    println!("Price: {}", trade_struct.price);  // Очевидно!
}

struct Trade {
    price: f64,
    volume: f64,
    is_buy: bool,
}
```

## Определение структур

### Базовая структура

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    order_type: String,
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        side: String::from("buy"),
        order_type: String::from("limit"),
    };

    println!("Order: {} {} {} @ {}",
        order.side, order.quantity, order.symbol, order.price);
}
```

### Структура с различными типами полей

```rust
struct Candle {
    symbol: String,
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() {
    let candle = Candle {
        symbol: String::from("ETH/USDT"),
        timestamp: 1703980800,
        open: 2200.0,
        high: 2250.0,
        low: 2180.0,
        close: 2230.0,
        volume: 1500.5,
    };

    let is_bullish = candle.close > candle.open;
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;

    println!("Symbol: {}", candle.symbol);
    println!("Type: {}", if is_bullish { "Bullish" } else { "Bearish" });
    println!("Body: {:.2}, Range: {:.2}", body, range);
}
```

## Доступ к полям

### Чтение полей

```rust
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let position = Position {
        symbol: String::from("BTC/USDT"),
        entry_price: 42000.0,
        quantity: 0.5,
        is_long: true,
    };

    // Доступ к отдельным полям
    println!("Symbol: {}", position.symbol);
    println!("Entry: ${}", position.entry_price);
    println!("Size: {}", position.quantity);
    println!("Direction: {}", if position.is_long { "Long" } else { "Short" });

    // Использование полей в вычислениях
    let notional_value = position.entry_price * position.quantity;
    println!("Notional Value: ${:.2}", notional_value);
}
```

### Изменение полей (mut)

```rust
struct Portfolio {
    balance: f64,
    total_pnl: f64,
    trade_count: u32,
}

fn main() {
    let mut portfolio = Portfolio {
        balance: 10000.0,
        total_pnl: 0.0,
        trade_count: 0,
    };

    println!("Initial balance: ${}", portfolio.balance);

    // Симулируем прибыльную сделку
    let trade_pnl = 150.0;
    portfolio.balance += trade_pnl;
    portfolio.total_pnl += trade_pnl;
    portfolio.trade_count += 1;

    println!("After trade: ${}", portfolio.balance);
    println!("Total PnL: ${}", portfolio.total_pnl);
    println!("Trades: {}", portfolio.trade_count);

    // Ещё одна сделка (убыточная)
    let trade_pnl = -50.0;
    portfolio.balance += trade_pnl;
    portfolio.total_pnl += trade_pnl;
    portfolio.trade_count += 1;

    println!("\nFinal state:");
    println!("Balance: ${}", portfolio.balance);
    println!("Total PnL: ${}", portfolio.total_pnl);
    println!("Trades: {}", portfolio.trade_count);
}
```

## Сокращённая инициализация полей

Если переменная имеет то же имя, что и поле:

```rust
struct Trade {
    symbol: String,
    price: f64,
    volume: f64,
}

fn main() {
    let symbol = String::from("BTC/USDT");
    let price = 42000.0;
    let volume = 0.5;

    // Полная форма
    let trade1 = Trade {
        symbol: symbol.clone(),
        price: price,
        volume: volume,
    };

    // Сокращённая форма (field init shorthand)
    let symbol = String::from("ETH/USDT");
    let price = 2200.0;
    let volume = 1.0;

    let trade2 = Trade {
        symbol,  // Сокращённо: symbol: symbol
        price,   // Сокращённо: price: price
        volume,  // Сокращённо: volume: volume
    };

    println!("Trade 1: {} @ {}", trade1.symbol, trade1.price);
    println!("Trade 2: {} @ {}", trade2.symbol, trade2.price);
}
```

## Синтаксис обновления структур

Создание новой структуры на основе существующей:

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn main() {
    let original_order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        side: String::from("buy"),
    };

    // Изменяем только цену, остальное копируем
    let modified_order = Order {
        price: 41500.0,
        ..original_order
    };

    // Внимание: original_order.symbol был перемещён!
    println!("Modified order: {} @ {}", modified_order.symbol, modified_order.price);
}
```

## Практический пример: Торговая система

```rust
struct TradeSignal {
    symbol: String,
    direction: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
}

fn main() {
    let signal = TradeSignal {
        symbol: String::from("BTC/USDT"),
        direction: String::from("long"),
        entry_price: 42000.0,
        stop_loss: 41000.0,
        take_profit: 44000.0,
        position_size: 0.5,
    };

    // Расчёт риск-менеджмента
    let risk_amount = (signal.entry_price - signal.stop_loss).abs() * signal.position_size;
    let reward_amount = (signal.take_profit - signal.entry_price).abs() * signal.position_size;
    let risk_reward_ratio = reward_amount / risk_amount;

    println!("╔══════════════════════════════════════╗");
    println!("║         TRADE SIGNAL                 ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Symbol:      {}               ║", signal.symbol);
    println!("║ Direction:   {}                    ║", signal.direction);
    println!("║ Entry:       ${:.2}             ║", signal.entry_price);
    println!("║ Stop Loss:   ${:.2}             ║", signal.stop_loss);
    println!("║ Take Profit: ${:.2}             ║", signal.take_profit);
    println!("║ Size:        {} BTC                 ║", signal.position_size);
    println!("╠══════════════════════════════════════╣");
    println!("║ Risk:        ${:.2}               ║", risk_amount);
    println!("║ Reward:      ${:.2}              ║", reward_amount);
    println!("║ R:R Ratio:   {:.2}                   ║", risk_reward_ratio);
    println!("╚══════════════════════════════════════╝");
}
```

## Практический пример: Анализ ордербука

```rust
struct OrderBookLevel {
    price: f64,
    volume: f64,
    order_count: u32,
}

fn main() {
    // Лучшие уровни ордербука
    let best_bid = OrderBookLevel {
        price: 42000.0,
        volume: 5.5,
        order_count: 12,
    };

    let best_ask = OrderBookLevel {
        price: 42010.0,
        volume: 3.2,
        order_count: 8,
    };

    // Анализ
    let spread = best_ask.price - best_bid.price;
    let spread_percent = (spread / best_bid.price) * 100.0;
    let mid_price = (best_bid.price + best_ask.price) / 2.0;
    let imbalance = best_bid.volume / (best_bid.volume + best_ask.volume);

    println!("=== Order Book Analysis ===\n");

    println!("Best Bid: ${:.2} x {:.4} ({} orders)",
        best_bid.price, best_bid.volume, best_bid.order_count);
    println!("Best Ask: ${:.2} x {:.4} ({} orders)",
        best_ask.price, best_ask.volume, best_ask.order_count);

    println!("\nMetrics:");
    println!("  Spread:     ${:.2} ({:.4}%)", spread, spread_percent);
    println!("  Mid Price:  ${:.2}", mid_price);
    println!("  Imbalance:  {:.1}% bid-heavy", imbalance * 100.0);
}
```

## Практический пример: Трекинг портфеля

```rust
struct Asset {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

fn main() {
    let mut assets = [
        Asset {
            symbol: String::from("BTC"),
            quantity: 0.5,
            avg_price: 40000.0,
            current_price: 42000.0,
        },
        Asset {
            symbol: String::from("ETH"),
            quantity: 5.0,
            avg_price: 2000.0,
            current_price: 2200.0,
        },
        Asset {
            symbol: String::from("SOL"),
            quantity: 50.0,
            avg_price: 80.0,
            current_price: 95.0,
        },
    ];

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                    PORTFOLIO TRACKER                    ║");
    println!("╠════════╦══════════╦══════════╦══════════╦══════════════╣");
    println!("║ Asset  ║ Quantity ║ Avg Cost ║ Current  ║ PnL          ║");
    println!("╠════════╬══════════╬══════════╬══════════╬══════════════╣");

    let mut total_cost = 0.0;
    let mut total_value = 0.0;

    for asset in &assets {
        let cost = asset.quantity * asset.avg_price;
        let value = asset.quantity * asset.current_price;
        let pnl = value - cost;
        let pnl_percent = (pnl / cost) * 100.0;

        total_cost += cost;
        total_value += value;

        println!("║ {:6} ║ {:8.4} ║ {:>8.2} ║ {:>8.2} ║ {:+8.2} ({:+.1}%) ║",
            asset.symbol, asset.quantity, asset.avg_price,
            asset.current_price, pnl, pnl_percent);
    }

    let total_pnl = total_value - total_cost;
    let total_pnl_percent = (total_pnl / total_cost) * 100.0;

    println!("╠════════╩══════════╩══════════╩══════════╩══════════════╣");
    println!("║ Total Cost:  ${:>10.2}                              ║", total_cost);
    println!("║ Total Value: ${:>10.2}                              ║", total_value);
    println!("║ Total PnL:   ${:>+10.2} ({:+.2}%)                    ║", total_pnl, total_pnl_percent);
    println!("╚════════════════════════════════════════════════════════╝");
}
```

## Debug-вывод структур

Для отладки используйте атрибут `#[derive(Debug)]`:

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    volume: f64,
    is_buy: bool,
}

fn main() {
    let trade = Trade {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };

    // Обычный debug вывод
    println!("{:?}", trade);

    // Красивый debug вывод
    println!("{:#?}", trade);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `struct Name { field: Type }` | Определение структуры |
| `instance.field` | Доступ к полю |
| `mut` | Изменяемая структура |
| `field,` | Сокращённая инициализация |
| `..other` | Синтаксис обновления |
| `#[derive(Debug)]` | Debug-вывод структуры |

## Домашнее задание

1. Создай структуру `MarketData` с полями: symbol, bid, ask, last_price, volume_24h. Напиши код для расчёта спреда и mid-price.

2. Создай структуру `RiskMetrics` с полями: position_size, entry_price, stop_loss, take_profit, account_balance. Рассчитай риск в процентах от депозита.

3. Создай массив из нескольких структур `Trade` и посчитай общий объём и средневзвешенную цену.

4. Реализуй структуру `TradingStrategy` с полями для настроек стратегии (take_profit_percent, stop_loss_percent, max_position_size). Используй её для генерации торгового сигнала.

## Навигация

[← Предыдущий день](../060-structs-intro/ru.md) | [Следующий день →](../062-struct-methods/ru.md)
