# День 61: Поля структур — цена, объём, направление

## Аналогия из трейдинга

Представь торговый ордер как анкету, которую заполняет трейдер:
- **Тикер** — какой актив торгуем
- **Цена** — по какой цене хотим купить/продать
- **Объём** — сколько единиц
- **Направление** — покупка или продажа

Каждый пункт анкеты — это **поле структуры**. Поля имеют имена и типы, что делает код понятным и надёжным.

## Что такое поля структур?

Поля — это именованные компоненты структуры. Каждое поле имеет:
- **Имя** — как называется
- **Тип** — какие данные хранит

```rust
struct Order {
    symbol: String,      // Поле symbol типа String
    price: f64,          // Поле price типа f64
    quantity: f64,       // Поле quantity типа f64
    is_buy: bool,        // Поле is_buy типа bool
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        is_buy: true,
    };

    println!("Order: {} {} {} @ {}",
        if order.is_buy { "BUY" } else { "SELL" },
        order.quantity,
        order.symbol,
        order.price
    );
}
```

## Доступ к полям

Для доступа к полям используется точечная нотация:

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let trade = Trade {
        symbol: String::from("ETH/USDT"),
        entry_price: 2500.0,
        exit_price: 2650.0,
        quantity: 2.0,
        is_long: true,
    };

    // Доступ к отдельным полям
    println!("Symbol: {}", trade.symbol);
    println!("Entry: ${}", trade.entry_price);
    println!("Exit: ${}", trade.exit_price);
    println!("Size: {}", trade.quantity);

    // Расчёт PnL на основе полей
    let price_diff = if trade.is_long {
        trade.exit_price - trade.entry_price
    } else {
        trade.entry_price - trade.exit_price
    };

    let pnl = price_diff * trade.quantity;
    println!("PnL: ${:.2}", pnl);
}
```

## Изменяемые поля

Чтобы изменять поля, структура должна быть объявлена как `mut`:

```rust
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
    unrealized_pnl: f64,
}

fn main() {
    let mut position = Position {
        symbol: String::from("BTC/USDT"),
        size: 1.0,
        entry_price: 42000.0,
        unrealized_pnl: 0.0,
    };

    println!("Initial position: {} BTC @ ${}", position.size, position.entry_price);

    // Цена изменилась — обновляем PnL
    let current_price = 43500.0;
    position.unrealized_pnl = (current_price - position.entry_price) * position.size;

    println!("Current price: ${}", current_price);
    println!("Unrealized PnL: ${:.2}", position.unrealized_pnl);

    // Добавляем к позиции (усреднение)
    let add_size = 0.5;
    let add_price = 43000.0;

    let total_cost = position.entry_price * position.size + add_price * add_size;
    position.size += add_size;
    position.entry_price = total_cost / position.size;

    println!("\nAfter averaging:");
    println!("New size: {} BTC", position.size);
    println!("Average entry: ${:.2}", position.entry_price);
}
```

## Типы полей

Поля могут быть любого типа, включая другие структуры:

```rust
struct Price {
    bid: f64,
    ask: f64,
}

struct Volume {
    bid_size: f64,
    ask_size: f64,
}

struct OrderBookLevel {
    price: Price,       // Вложенная структура
    volume: Volume,     // Вложенная структура
    timestamp: u64,
}

fn main() {
    let top_of_book = OrderBookLevel {
        price: Price {
            bid: 42000.0,
            ask: 42010.0,
        },
        volume: Volume {
            bid_size: 2.5,
            ask_size: 1.8,
        },
        timestamp: 1703980800,
    };

    // Доступ к вложенным полям
    let spread = top_of_book.price.ask - top_of_book.price.bid;
    let total_liquidity = top_of_book.volume.bid_size + top_of_book.volume.ask_size;

    println!("Best Bid: ${} x {:.2}",
        top_of_book.price.bid,
        top_of_book.volume.bid_size
    );
    println!("Best Ask: ${} x {:.2}",
        top_of_book.price.ask,
        top_of_book.volume.ask_size
    );
    println!("Spread: ${:.2}", spread);
    println!("Total liquidity: {:.2} BTC", total_liquidity);
}
```

## Практический пример: анализ свечи

```rust
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    let candle = Candle {
        open: 42000.0,
        high: 42800.0,
        low: 41500.0,
        close: 42600.0,
        volume: 1250.5,
        timestamp: 1703980800,
    };

    // Анализ свечи через поля
    let is_bullish = candle.close > candle.open;
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;
    let upper_shadow = candle.high - candle.close.max(candle.open);
    let lower_shadow = candle.close.min(candle.open) - candle.low;

    println!("╔═══════════════════════════════════╗");
    println!("║       CANDLE ANALYSIS             ║");
    println!("╠═══════════════════════════════════╣");
    println!("║ Open:   ${:.2}               ║", candle.open);
    println!("║ High:   ${:.2}               ║", candle.high);
    println!("║ Low:    ${:.2}               ║", candle.low);
    println!("║ Close:  ${:.2}               ║", candle.close);
    println!("║ Volume: {:.2} BTC             ║", candle.volume);
    println!("╠═══════════════════════════════════╣");
    println!("║ Type: {}                   ║",
        if is_bullish { "BULLISH" } else { "BEARISH" });
    println!("║ Body:  ${:.2}                 ║", body);
    println!("║ Range: ${:.2}                ║", range);
    println!("║ Upper Shadow: ${:.2}          ║", upper_shadow);
    println!("║ Lower Shadow: ${:.2}          ║", lower_shadow);
    println!("╚═══════════════════════════════════╝");
}
```

## Практический пример: управление ордерами

```rust
struct LimitOrder {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    filled: f64,
    status: String,
}

fn main() {
    let mut order = LimitOrder {
        id: 12345,
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 1.0,
        filled: 0.0,
        status: String::from("NEW"),
    };

    println!("=== Order #{} ===", order.id);
    println!("{} {} {} @ ${}",
        order.side, order.quantity, order.symbol, order.price);
    println!("Status: {}", order.status);

    // Частичное исполнение
    let fill_qty = 0.3;
    order.filled += fill_qty;
    order.status = String::from("PARTIALLY_FILLED");

    println!("\n--- Partial fill: {} ---", fill_qty);
    println!("Filled: {}/{}", order.filled, order.quantity);
    println!("Remaining: {}", order.quantity - order.filled);
    println!("Status: {}", order.status);

    // Полное исполнение
    order.filled = order.quantity;
    order.status = String::from("FILLED");

    println!("\n--- Order complete ---");
    println!("Status: {}", order.status);
}
```

## Практический пример: трекинг портфеля

```rust
struct Asset {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

fn main() {
    let assets = [
        Asset {
            symbol: String::from("BTC"),
            quantity: 1.5,
            avg_price: 40000.0,
            current_price: 43000.0,
        },
        Asset {
            symbol: String::from("ETH"),
            quantity: 10.0,
            avg_price: 2200.0,
            current_price: 2500.0,
        },
        Asset {
            symbol: String::from("SOL"),
            quantity: 50.0,
            avg_price: 80.0,
            current_price: 95.0,
        },
    ];

    println!("{:<8} {:>10} {:>12} {:>12} {:>12}",
        "Asset", "Quantity", "Avg Price", "Current", "PnL");
    println!("{}", "-".repeat(58));

    let mut total_value = 0.0;
    let mut total_cost = 0.0;

    for asset in &assets {
        let cost = asset.quantity * asset.avg_price;
        let value = asset.quantity * asset.current_price;
        let pnl = value - cost;

        total_cost += cost;
        total_value += value;

        println!("{:<8} {:>10.2} {:>12.2} {:>12.2} {:>+12.2}",
            asset.symbol,
            asset.quantity,
            asset.avg_price,
            asset.current_price,
            pnl
        );
    }

    println!("{}", "-".repeat(58));
    println!("{:<8} {:>10} {:>12.2} {:>12.2} {:>+12.2}",
        "TOTAL", "", total_cost, total_value, total_value - total_cost);
}
```

## Практический пример: риск-менеджмент

```rust
struct RiskParams {
    max_position_size: f64,
    max_loss_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}

struct TradeSetup {
    symbol: String,
    entry_price: f64,
    position_size: f64,
    direction: String,
}

fn main() {
    let risk = RiskParams {
        max_position_size: 10000.0,   // Максимальный размер позиции в USD
        max_loss_percent: 2.0,         // Максимальный убыток 2% от капитала
        stop_loss_percent: 1.5,        // Стоп-лосс 1.5% от входа
        take_profit_percent: 3.0,      // Тейк-профит 3% от входа
    };

    let setup = TradeSetup {
        symbol: String::from("BTC/USDT"),
        entry_price: 42000.0,
        position_size: 5000.0,
        direction: String::from("LONG"),
    };

    // Расчёт уровней на основе полей
    let stop_loss = if setup.direction == "LONG" {
        setup.entry_price * (1.0 - risk.stop_loss_percent / 100.0)
    } else {
        setup.entry_price * (1.0 + risk.stop_loss_percent / 100.0)
    };

    let take_profit = if setup.direction == "LONG" {
        setup.entry_price * (1.0 + risk.take_profit_percent / 100.0)
    } else {
        setup.entry_price * (1.0 - risk.take_profit_percent / 100.0)
    };

    let potential_loss = (setup.entry_price - stop_loss).abs()
        * (setup.position_size / setup.entry_price);
    let potential_profit = (take_profit - setup.entry_price).abs()
        * (setup.position_size / setup.entry_price);
    let risk_reward = potential_profit / potential_loss;

    println!("=== Trade Setup: {} ===", setup.symbol);
    println!("Direction: {}", setup.direction);
    println!("Entry: ${:.2}", setup.entry_price);
    println!("Position: ${:.2}", setup.position_size);
    println!();
    println!("Stop Loss: ${:.2} ({:.1}%)", stop_loss, risk.stop_loss_percent);
    println!("Take Profit: ${:.2} ({:.1}%)", take_profit, risk.take_profit_percent);
    println!();
    println!("Potential Loss: ${:.2}", potential_loss);
    println!("Potential Profit: ${:.2}", potential_profit);
    println!("Risk/Reward: 1:{:.2}", risk_reward);

    // Проверка рисков
    let size_ok = setup.position_size <= risk.max_position_size;
    println!("\nRisk Check:");
    println!("  Position size OK: {} (${:.0} <= ${:.0})",
        size_ok, setup.position_size, risk.max_position_size);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `struct.field` | Доступ к полю структуры |
| `mut struct` | Изменяемая структура |
| Вложенные поля | Структуры внутри структур |
| Именованные поля | Понятный и читаемый код |

## Домашнее задание

1. Создай структуру `Ticker` с полями: symbol, last_price, change_24h, volume_24h. Напиши код для вывода информации о тикере.

2. Создай структуру `Wallet` с полями: currency, balance, available, locked. Реализуй логику для блокировки части баланса под ордер.

3. Создай структуру `TradeStats` с полями: total_trades, wins, losses, gross_profit, gross_loss. Вычисли win rate и profit factor.

4. Создай вложенные структуры для представления торговой пары: `TradingPair` содержит `BaseAsset` и `QuoteAsset`, каждый из которых имеет symbol и precision.

## Навигация

[← Предыдущий день](../060-structs-creating-order-type/ru.md) | [Следующий день →](../062-creating-instance-new-order/ru.md)
