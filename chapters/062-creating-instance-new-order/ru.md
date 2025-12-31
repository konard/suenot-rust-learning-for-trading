# День 62: Создание экземпляра — новый ордер

## Аналогия из трейдинга

Когда ты хочешь купить или продать актив на бирже, ты заполняешь **бланк ордера**: указываешь тикер, направление (покупка/продажа), цену, количество. Это и есть создание экземпляра — ты берёшь шаблон (структуру) и заполняешь конкретными данными.

## Базовое создание экземпляра

```rust
// Определяем структуру ордера
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    // Создаём экземпляр — заполняем все поля
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 42000.0,
        quantity: 0.5,
    };

    println!("Ордер: {} {} {} по цене {}",
        order.side, order.quantity, order.symbol, order.price);
}
```

**Важно:** Нужно указать значения для **всех** полей структуры!

## Порядок полей не важен

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

fn main() {
    // Поля можно указывать в любом порядке
    let trade = Trade {
        quantity: 1.0,           // Сначала количество
        symbol: String::from("ETH/USDT"),  // Потом символ
        exit_price: 2200.0,      // Цена выхода
        entry_price: 2000.0,     // Цена входа
    };

    let pnl = (trade.exit_price - trade.entry_price) * trade.quantity;
    println!("Сделка по {}: PnL = ${:.2}", trade.symbol, pnl);
}
```

## Сокращённая инициализация (Field Init Shorthand)

Когда имя переменной совпадает с именем поля, можно использовать сокращённый синтаксис:

```rust
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

fn main() {
    let symbol = String::from("SOL/USDT");
    let quantity = 10.0;
    let entry_price = 100.0;

    // Длинная форма
    let position1 = Position {
        symbol: symbol.clone(),
        quantity: quantity,
        entry_price: entry_price,
    };

    // Сокращённая форма — если имена совпадают
    let symbol = String::from("AVAX/USDT");
    let quantity = 5.0;
    let entry_price = 35.0;

    let position2 = Position {
        symbol,       // вместо symbol: symbol
        quantity,     // вместо quantity: quantity
        entry_price,  // вместо entry_price: entry_price
    };

    println!("Позиция 1: {} x {}", position1.symbol, position1.quantity);
    println!("Позиция 2: {} x {}", position2.symbol, position2.quantity);
}
```

## Синтаксис обновления структуры (Struct Update Syntax)

Позволяет создать новый экземпляр на основе существующего, изменив только часть полей:

```rust
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    time_in_force: String,
}

fn main() {
    // Базовый ордер
    let base_order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 42000.0,
        quantity: 1.0,
        time_in_force: String::from("GTC"), // Good Till Cancelled
    };

    // Новый ордер с другой ценой, остальное берём из base_order
    let limit_order = Order {
        price: 41500.0,  // Другая цена
        ..base_order     // Остальные поля из base_order
    };

    // Осторожно! base_order.symbol перемещён в limit_order
    // base_order больше нельзя использовать целиком

    println!("Лимитный ордер по цене: {}", limit_order.price);
    println!("Символ: {}", limit_order.symbol);
}
```

**Важно:** `..base_order` должен стоять в конце!

## Создание с Clone для повторного использования

```rust
#[derive(Clone)]
struct OrderTemplate {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let template = OrderTemplate {
        symbol: String::from("ETH/USDT"),
        side: String::from("Buy"),
        price: 2000.0,
        quantity: 1.0,
    };

    // Создаём несколько ордеров на основе шаблона
    let order1 = OrderTemplate {
        price: 1950.0,
        ..template.clone()  // Клонируем, чтобы template остался доступен
    };

    let order2 = OrderTemplate {
        price: 1900.0,
        ..template.clone()
    };

    let order3 = OrderTemplate {
        price: 1850.0,
        ..template  // Последнее использование — можно без clone
    };

    println!("Лесенка ордеров: {}, {}, {}",
        order1.price, order2.price, order3.price);
}
```

## Практический пример: создание ордеров для торговли

```rust
struct MarketOrder {
    symbol: String,
    side: String,
    quantity: f64,
    timestamp: u64,
}

struct LimitOrder {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    time_in_force: String,
}

struct StopOrder {
    symbol: String,
    side: String,
    stop_price: f64,
    quantity: f64,
}

fn main() {
    // Рыночный ордер — исполняется немедленно по текущей цене
    let market_buy = MarketOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        quantity: 0.1,
        timestamp: 1704067200,
    };

    // Лимитный ордер — ждёт нужной цены
    let limit_buy = LimitOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 40000.0,
        quantity: 0.5,
        time_in_force: String::from("GTC"),
    };

    // Стоп-ордер — активируется при достижении цены
    let stop_loss = StopOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Sell"),
        stop_price: 38000.0,
        quantity: 0.5,
    };

    println!("Рыночный: {} {} x {}",
        market_buy.side, market_buy.symbol, market_buy.quantity);
    println!("Лимитный: {} {} x {} @ {}",
        limit_buy.side, limit_buy.symbol, limit_buy.quantity, limit_buy.price);
    println!("Стоп: {} {} x {} @ {}",
        stop_loss.side, stop_loss.symbol, stop_loss.quantity, stop_loss.stop_price);
}
```

## Создание экземпляров в функциях

```rust
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
}

fn create_trade(id: u64, symbol: &str, side: &str, price: f64, quantity: f64) -> Trade {
    Trade {
        id,
        symbol: String::from(symbol),
        side: String::from(side),
        price,
        quantity,
        pnl: 0.0,  // PnL рассчитаем позже
    }
}

fn close_trade(open_trade: &Trade, exit_price: f64) -> Trade {
    let pnl = if open_trade.side == "Buy" {
        (exit_price - open_trade.price) * open_trade.quantity
    } else {
        (open_trade.price - exit_price) * open_trade.quantity
    };

    Trade {
        id: open_trade.id,
        symbol: open_trade.symbol.clone(),
        side: if open_trade.side == "Buy" {
            String::from("Sell")
        } else {
            String::from("Buy")
        },
        price: exit_price,
        quantity: open_trade.quantity,
        pnl,
    }
}

fn main() {
    let entry = create_trade(1, "BTC/USDT", "Buy", 42000.0, 0.5);
    println!("Открыли: {} {} @ {}", entry.side, entry.symbol, entry.price);

    let exit = close_trade(&entry, 43500.0);
    println!("Закрыли: {} {} @ {}", exit.side, exit.symbol, exit.price);
    println!("PnL: ${:.2}", exit.pnl);
}
```

## Вложенные структуры

```rust
struct Price {
    value: f64,
    currency: String,
}

struct OrderDetails {
    symbol: String,
    side: String,
}

struct CompleteOrder {
    details: OrderDetails,
    price: Price,
    quantity: f64,
}

fn main() {
    // Создаём вложенные структуры
    let order = CompleteOrder {
        details: OrderDetails {
            symbol: String::from("ETH/USDT"),
            side: String::from("Buy"),
        },
        price: Price {
            value: 2000.0,
            currency: String::from("USDT"),
        },
        quantity: 2.5,
    };

    println!("Ордер: {} {} x {} @ {} {}",
        order.details.side,
        order.details.symbol,
        order.quantity,
        order.price.value,
        order.price.currency
    );
}
```

## Массив экземпляров структуры

```rust
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

fn main() {
    // Массив свечей
    let candles = [
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0 },
        Candle { open: 42600.0, high: 43000.0, low: 42400.0, close: 42900.0 },
    ];

    println!("История цен:");
    for (i, candle) in candles.iter().enumerate() {
        let change = candle.close - candle.open;
        let emoji = if change >= 0.0 { "📈" } else { "📉" };
        println!("  Свеча {}: O={} H={} L={} C={} {}",
            i + 1, candle.open, candle.high, candle.low, candle.close, emoji);
    }
}
```

## Паттерны создания экземпляров

```rust
struct RiskParams {
    max_position_size: f64,
    max_loss_per_trade: f64,
    daily_loss_limit: f64,
}

// Паттерн 1: Создание с значениями по умолчанию через функцию
fn default_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 1000.0,
        max_loss_per_trade: 50.0,
        daily_loss_limit: 200.0,
    }
}

// Паттерн 2: Создание агрессивных параметров
fn aggressive_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 5000.0,
        max_loss_per_trade: 200.0,
        daily_loss_limit: 1000.0,
    }
}

// Паттерн 3: Создание консервативных параметров
fn conservative_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 500.0,
        max_loss_per_trade: 25.0,
        daily_loss_limit: 100.0,
    }
}

fn main() {
    let default_risk = default_risk_params();
    let aggressive_risk = aggressive_risk_params();
    let conservative_risk = conservative_risk_params();

    println!("Стандартный риск: max позиция = ${}", default_risk.max_position_size);
    println!("Агрессивный риск: max позиция = ${}", aggressive_risk.max_position_size);
    println!("Консервативный риск: max позиция = ${}", conservative_risk.max_position_size);
}
```

## Что мы узнали

| Концепт | Синтаксис | Описание |
|---------|-----------|----------|
| Базовое создание | `Struct { field: value }` | Указываем все поля |
| Сокращённая форма | `Struct { field }` | Когда имя переменной = имя поля |
| Update syntax | `Struct { field: val, ..other }` | Берём остальные поля из другого экземпляра |
| Вложенные структуры | `Struct { inner: Inner { } }` | Структура внутри структуры |
| Массив структур | `[Struct { }, Struct { }]` | Коллекция экземпляров |

## Домашнее задание

1. Создай структуру `Portfolio` с полями: `name`, `balance`, `positions_count`. Создай три разных портфеля

2. Реализуй структуру `TradeSignal` с полями: `symbol`, `action` (Buy/Sell), `confidence` (0.0-1.0), `timestamp`. Создай массив из 5 сигналов

3. Создай структуру `ExchangeConfig` и используй struct update syntax для создания конфигураций разных бирж с общими базовыми настройками

4. Напиши функцию `create_bracket_orders(symbol, entry_price, stop_loss, take_profit, quantity)`, которая возвращает кортеж из трёх ордеров: entry, stop-loss и take-profit

## Навигация

[← Предыдущий день](../061-struct-fields-price-volume-direction/ru.md) | [Следующий день →](../063-methods-order-execute/ru.md)
