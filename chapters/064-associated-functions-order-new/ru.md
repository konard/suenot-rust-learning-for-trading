# День 64: Ассоциированные функции — Order::new()

## Аналогия из трейдинга

Представь биржу. На бирже есть специальный отдел, который создаёт новые ордера. Ты не создаёшь ордер сам — ты обращаешься в отдел биржи и говоришь: "Создай мне ордер на покупку BTC". Этот отдел — часть системы "Ордер", но он работает не с конкретным ордером, а создаёт новые.

В Rust это **ассоциированные функции** — функции, которые связаны со структурой, но не работают с конкретным экземпляром. Самая известная — `new()`, конструктор.

## Методы vs Ассоциированные функции

```rust
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl Order {
    // Ассоциированная функция — НЕТ параметра self
    // Вызывается: Order::new(...)
    fn new(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            symbol: symbol.to_string(),
            quantity,
            price,
        }
    }

    // Метод — ЕСТЬ параметр self
    // Вызывается: order.total_value()
    fn total_value(&self) -> f64 {
        self.quantity * self.price
    }
}

fn main() {
    // Ассоциированная функция вызывается через ::
    let order = Order::new("BTC", 0.5, 42000.0);

    // Метод вызывается через .
    println!("Total value: ${:.2}", order.total_value());
}
```

**Ключевое отличие:**
- `Order::new()` — двоеточие, нет экземпляра до точки
- `order.total_value()` — точка, есть экземпляр `order`

## Конструктор new()

Паттерн `new()` — стандартный способ создания экземпляров в Rust:

```rust
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: Option<f64>,
}

impl Trade {
    fn new(symbol: &str, side: &str, quantity: f64, entry_price: f64) -> Trade {
        Trade {
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            entry_price,
            exit_price: None, // Сделка ещё открыта
        }
    }
}

fn main() {
    let trade = Trade::new("ETH", "buy", 2.0, 2500.0);
    println!("Opened {} {} @ ${}", trade.side, trade.symbol, trade.entry_price);
}
```

## Несколько конструкторов

Можно создать несколько ассоциированных функций для разных сценариев:

```rust
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Position {
    // Основной конструктор
    fn new(symbol: &str, quantity: f64, avg_price: f64) -> Position {
        Position {
            symbol: symbol.to_string(),
            quantity,
            avg_price,
        }
    }

    // Создать пустую позицию
    fn empty(symbol: &str) -> Position {
        Position {
            symbol: symbol.to_string(),
            quantity: 0.0,
            avg_price: 0.0,
        }
    }

    // Создать из сделки
    fn from_trade(symbol: &str, quantity: f64, price: f64) -> Position {
        Position::new(symbol, quantity, price)
    }

    fn value(&self) -> f64 {
        self.quantity * self.avg_price
    }
}

fn main() {
    let pos1 = Position::new("BTC", 0.5, 42000.0);
    let pos2 = Position::empty("ETH");
    let pos3 = Position::from_trade("SOL", 100.0, 25.0);

    println!("BTC position value: ${:.2}", pos1.value());
    println!("ETH position value: ${:.2}", pos2.value());
    println!("SOL position value: ${:.2}", pos3.value());
}
```

## Валидация в конструкторе

Ассоциированные функции могут включать логику валидации:

```rust
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl Order {
    fn new(symbol: &str, quantity: f64, price: f64) -> Option<Order> {
        // Валидация: количество и цена должны быть положительными
        if quantity <= 0.0 || price <= 0.0 {
            return None;
        }

        // Валидация: символ не должен быть пустым
        if symbol.is_empty() {
            return None;
        }

        Some(Order {
            symbol: symbol.to_string(),
            quantity,
            price,
        })
    }
}

fn main() {
    match Order::new("BTC", 0.5, 42000.0) {
        Some(order) => println!("Order created: {} @ ${}", order.symbol, order.price),
        None => println!("Invalid order parameters"),
    }

    match Order::new("", -1.0, 0.0) {
        Some(_) => println!("Order created"),
        None => println!("Invalid order parameters — rejected"),
    }
}
```

## Конструктор с Result

Для более детальной информации об ошибках используем Result:

```rust
#[derive(Debug)]
enum OrderError {
    InvalidQuantity,
    InvalidPrice,
    EmptySymbol,
}

struct LimitOrder {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl LimitOrder {
    fn new(symbol: &str, quantity: f64, price: f64) -> Result<LimitOrder, OrderError> {
        if symbol.is_empty() {
            return Err(OrderError::EmptySymbol);
        }
        if quantity <= 0.0 {
            return Err(OrderError::InvalidQuantity);
        }
        if price <= 0.0 {
            return Err(OrderError::InvalidPrice);
        }

        Ok(LimitOrder {
            symbol: symbol.to_string(),
            quantity,
            price,
        })
    }
}

fn main() {
    match LimitOrder::new("BTC", 0.5, 42000.0) {
        Ok(order) => println!("Order: {} {} @ ${}", order.symbol, order.quantity, order.price),
        Err(e) => println!("Error: {:?}", e),
    }

    match LimitOrder::new("BTC", -1.0, 42000.0) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Фабричные функции

Ассоциированные функции могут создавать экземпляры с предустановленными параметрами:

```rust
struct RiskSettings {
    max_position_size: f64,
    max_loss_percent: f64,
    max_daily_trades: u32,
}

impl RiskSettings {
    fn new(max_position: f64, max_loss: f64, max_trades: u32) -> RiskSettings {
        RiskSettings {
            max_position_size: max_position,
            max_loss_percent: max_loss,
            max_daily_trades: max_trades,
        }
    }

    // Консервативные настройки
    fn conservative() -> RiskSettings {
        RiskSettings {
            max_position_size: 1000.0,
            max_loss_percent: 1.0,
            max_daily_trades: 3,
        }
    }

    // Умеренные настройки
    fn moderate() -> RiskSettings {
        RiskSettings {
            max_position_size: 5000.0,
            max_loss_percent: 2.0,
            max_daily_trades: 10,
        }
    }

    // Агрессивные настройки
    fn aggressive() -> RiskSettings {
        RiskSettings {
            max_position_size: 20000.0,
            max_loss_percent: 5.0,
            max_daily_trades: 50,
        }
    }
}

fn main() {
    let settings = RiskSettings::conservative();
    println!("Max position: ${}", settings.max_position_size);
    println!("Max loss: {}%", settings.max_loss_percent);
    println!("Max trades: {}", settings.max_daily_trades);

    let aggressive = RiskSettings::aggressive();
    println!("\nAggressive - Max position: ${}", aggressive.max_position_size);
}
```

## Практический пример: Order Builder

```rust
#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Order {
    // Счётчик для генерации ID
    fn next_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    // Рыночный ордер на покупку
    fn market_buy(symbol: &str, quantity: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "buy".to_string(),
            order_type: "market".to_string(),
            quantity,
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Рыночный ордер на продажу
    fn market_sell(symbol: &str, quantity: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "sell".to_string(),
            order_type: "market".to_string(),
            quantity,
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Лимитный ордер на покупку
    fn limit_buy(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(price),
            stop_loss: None,
            take_profit: None,
        }
    }

    // Лимитный ордер на продажу
    fn limit_sell(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "sell".to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(price),
            stop_loss: None,
            take_profit: None,
        }
    }

    // Ордер с защитными уровнями
    fn with_bracket(symbol: &str, quantity: f64, entry: f64, sl: f64, tp: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: if entry > sl { "buy" } else { "sell" }.to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(entry),
            stop_loss: Some(sl),
            take_profit: Some(tp),
        }
    }

    fn describe(&self) {
        println!("Order #{}: {} {} {} @ {:?}",
            self.id, self.side, self.quantity, self.symbol,
            self.price.map(|p| format!("${:.2}", p)).unwrap_or("MARKET".to_string())
        );
        if let Some(sl) = self.stop_loss {
            println!("  Stop Loss: ${:.2}", sl);
        }
        if let Some(tp) = self.take_profit {
            println!("  Take Profit: ${:.2}", tp);
        }
    }
}

fn main() {
    println!("=== Order Factory ===\n");

    let order1 = Order::market_buy("BTC", 0.1);
    order1.describe();

    let order2 = Order::limit_sell("ETH", 5.0, 2800.0);
    order2.describe();

    let order3 = Order::with_bracket("BTC", 0.5, 42000.0, 40000.0, 46000.0);
    order3.describe();
}
```

## Self как возвращаемый тип

Вместо имени структуры можно использовать `Self`:

```rust
struct Wallet {
    balance: f64,
    currency: String,
}

impl Wallet {
    fn new(balance: f64, currency: &str) -> Self {
        Self {
            balance,
            currency: currency.to_string(),
        }
    }

    fn usd(balance: f64) -> Self {
        Self::new(balance, "USD")
    }

    fn btc(balance: f64) -> Self {
        Self::new(balance, "BTC")
    }

    fn empty(currency: &str) -> Self {
        Self::new(0.0, currency)
    }
}

fn main() {
    let wallet1 = Wallet::usd(10000.0);
    let wallet2 = Wallet::btc(0.5);
    let wallet3 = Wallet::empty("ETH");

    println!("{}: {:.2}", wallet1.currency, wallet1.balance);
    println!("{}: {:.8}", wallet2.currency, wallet2.balance);
    println!("{}: {:.4}", wallet3.currency, wallet3.balance);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Struct::func()` | Вызов ассоциированной функции |
| Нет `self` | Не требует экземпляра |
| `new()` | Стандартный конструктор |
| `Self` | Псевдоним текущего типа |
| Фабричные функции | Создание с предустановками |
| Валидация | Option/Result в конструкторе |

## Упражнения

1. Создай структуру `Candle` (OHLCV) с ассоциированными функциями:
   - `new(open, high, low, close, volume)` — обычный конструктор
   - `from_price(price)` — свеча где O=H=L=C=price, volume=0
   - `bullish(open, close, volume)` — зелёная свеча (close > open)
   - `bearish(open, close, volume)` — красная свеча (close < open)

2. Создай структуру `StopOrder` с валидацией:
   - `new()` возвращает `Result<StopOrder, String>`
   - Проверяй что stop_price > 0
   - Проверяй что quantity > 0

3. Создай структуру `TradingSession` с фабричными функциями:
   - `new_york()` — NYSE часы
   - `london()` — LSE часы
   - `tokyo()` — TSE часы
   - `crypto()` — 24/7

## Домашнее задание

1. Создай структуру `Portfolio` с:
   - `new()` — пустой портфель
   - `with_cash(amount)` — портфель с начальным капиталом
   - `demo()` — демо-портфель с виртуальными $100,000

2. Создай структуру `Strategy` с фабриками:
   - `sma_crossover(fast, slow)` — стратегия пересечения SMA
   - `rsi_oversold(period, level)` — стратегия перепроданности RSI
   - `breakout(period)` — стратегия пробоя

3. Создай структуру `ExchangeAPI` с:
   - `binance()` — API Binance
   - `bybit()` — API Bybit
   - `demo()` — Демо режим без реальных запросов

## Навигация

[← День 63: Методы](../063-methods-order-execute/ru.md) | [День 65: Несколько блоков impl →](../065-multiple-impl-blocks/ru.md)
