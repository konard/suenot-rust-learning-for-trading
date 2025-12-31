# День 112: Newtype Pattern для валидации

## Аналогия из трейдинга

Представь, что у тебя есть торговый терминал, где цена и количество — это просто числа типа `f64`. Что произойдёт, если ты случайно передашь количество вместо цены? Компилятор не заметит ошибку, и ты можешь выставить ордер на покупку 42000 биткоинов по цене 0.5!

**Newtype pattern** — это как специальные конверты для разных типов данных. Цена лежит в "конверте Price", количество — в "конверте Quantity". Теперь их невозможно перепутать.

## Что такое Newtype Pattern?

Newtype — это структура-обёртка с одним полем, которая создаёт новый тип на основе существующего:

```rust
struct Price(f64);      // Цена — это f64, но в "обёртке"
struct Quantity(f64);   // Количество — тоже f64, но другой тип!
```

## Базовый пример: защита от перепутывания

```rust
fn main() {
    let price = Price::new(42000.0).unwrap();
    let quantity = Quantity::new(0.5).unwrap();

    // Это скомпилируется:
    let order = create_order(price, quantity);

    // А это — НЕТ! Компилятор защитит нас:
    // let bad_order = create_order(quantity, price); // Ошибка компиляции!
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug)]
struct Order {
    price: Price,
    quantity: Quantity,
}

fn create_order(price: Price, quantity: Quantity) -> Order {
    Order { price, quantity }
}

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 {
            Some(Price(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 {
            Some(Quantity(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}
```

## Валидация при создании

Главная сила newtype — валидация прямо в конструкторе:

```rust
fn main() {
    // Валидные значения
    let valid_price = Price::new(42000.0);
    println!("Valid price: {:?}", valid_price);

    // Невалидные значения отклоняются
    let invalid_price = Price::new(-100.0);
    println!("Invalid price: {:?}", invalid_price);

    let zero_price = Price::new(0.0);
    println!("Zero price: {:?}", zero_price);
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

impl Price {
    fn new(value: f64) -> Result<Price, PriceError> {
        if value.is_nan() {
            return Err(PriceError::NotANumber);
        }
        if value.is_infinite() {
            return Err(PriceError::Infinite);
        }
        if value <= 0.0 {
            return Err(PriceError::NonPositive(value));
        }
        if value > 1_000_000_000.0 {
            return Err(PriceError::TooLarge(value));
        }
        Ok(Price(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Debug)]
enum PriceError {
    NonPositive(f64),
    TooLarge(f64),
    NotANumber,
    Infinite,
}
```

## Newtype для процентов риска

```rust
fn main() {
    let risk = RiskPercent::new(2.0).unwrap();
    println!("Risk: {}%", risk.value());

    let balance = 10000.0;
    let risk_amount = risk.calculate_amount(balance);
    println!("Risk amount for ${}: ${:.2}", balance, risk_amount);

    // Попытка создать невалидный риск
    match RiskPercent::new(150.0) {
        Ok(_) => println!("Created"),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[derive(Debug, Clone, Copy)]
struct RiskPercent(f64);

#[derive(Debug)]
enum RiskError {
    Negative(f64),
    TooHigh(f64),
    Zero,
}

impl RiskPercent {
    /// Создаёт процент риска (допустимо от 0.01% до 100%)
    fn new(value: f64) -> Result<RiskPercent, RiskError> {
        if value <= 0.0 {
            return Err(RiskError::Zero);
        }
        if value < 0.0 {
            return Err(RiskError::Negative(value));
        }
        if value > 100.0 {
            return Err(RiskError::TooHigh(value));
        }
        Ok(RiskPercent(value))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn calculate_amount(&self, balance: f64) -> f64 {
        balance * (self.0 / 100.0)
    }
}
```

## Newtype для тикера (символа актива)

```rust
fn main() {
    match Ticker::new("BTCUSDT") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }

    match Ticker::new("") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }

    match Ticker::new("BTC-USDT-PERP-FUTURE") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[derive(Debug, Clone)]
struct Ticker(String);

#[derive(Debug)]
enum TickerError {
    Empty,
    TooLong(usize),
    InvalidCharacter(char),
}

impl Ticker {
    fn new(value: &str) -> Result<Ticker, TickerError> {
        if value.is_empty() {
            return Err(TickerError::Empty);
        }

        if value.len() > 20 {
            return Err(TickerError::TooLong(value.len()));
        }

        // Только буквы, цифры и дефис
        for ch in value.chars() {
            if !ch.is_alphanumeric() && ch != '-' {
                return Err(TickerError::InvalidCharacter(ch));
            }
        }

        Ok(Ticker(value.to_uppercase()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}
```

## Реализация арифметических операций

Newtype можно наделить арифметикой:

```rust
use std::ops::{Add, Sub, Mul};

fn main() {
    let price1 = Price::new(42000.0).unwrap();
    let price2 = Price::new(1500.0).unwrap();

    let sum = price1 + price2;
    println!("Sum: ${:.2}", sum.value());

    let diff = price1 - price2;
    println!("Diff: ${:.2}", diff.value());

    let qty = Quantity::new(0.5).unwrap();
    let value = price1 * qty;
    println!("Position value: ${:.2}", value);
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 { Some(Price(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 { Some(Quantity(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

// Price + Price = Price
impl Add for Price {
    type Output = Price;
    fn add(self, other: Price) -> Price {
        Price(self.0 + other.0)
    }
}

// Price - Price = Price (может быть отрицательной разницей!)
impl Sub for Price {
    type Output = f64;  // Возвращаем f64, т.к. разница может быть отрицательной
    fn sub(self, other: Price) -> f64 {
        self.0 - other.0
    }
}

// Price * Quantity = f64 (стоимость позиции)
impl Mul<Quantity> for Price {
    type Output = f64;
    fn mul(self, qty: Quantity) -> f64 {
        self.0 * qty.0
    }
}
```

## Полный пример: система управления ордерами

```rust
fn main() {
    // Создаём валидированные значения
    let ticker = Ticker::new("BTCUSDT").expect("Invalid ticker");
    let price = Price::new(42000.0).expect("Invalid price");
    let quantity = Quantity::new(0.5).expect("Invalid quantity");
    let stop_loss = Price::new(40000.0).expect("Invalid stop loss");

    // Создаём ордер с полной валидацией
    match Order::new(ticker, price, quantity, Some(stop_loss)) {
        Ok(order) => {
            println!("{}", order.summary());
        }
        Err(e) => {
            println!("Order creation failed: {:?}", e);
        }
    }
}

#[derive(Debug, Clone)]
struct Ticker(String);

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug)]
struct Order {
    ticker: Ticker,
    price: Price,
    quantity: Quantity,
    stop_loss: Option<Price>,
}

#[derive(Debug)]
enum OrderError {
    StopLossTooHigh,
    PositionTooSmall,
}

impl Ticker {
    fn new(value: &str) -> Option<Ticker> {
        if !value.is_empty() && value.len() <= 20 {
            Some(Ticker(value.to_uppercase()))
        } else {
            None
        }
    }
    fn as_str(&self) -> &str { &self.0 }
}

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 && value.is_finite() { Some(Price(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 && value.is_finite() { Some(Quantity(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Order {
    fn new(
        ticker: Ticker,
        price: Price,
        quantity: Quantity,
        stop_loss: Option<Price>,
    ) -> Result<Order, OrderError> {
        // Дополнительная валидация на уровне ордера
        if let Some(sl) = stop_loss {
            if sl.value() >= price.value() {
                return Err(OrderError::StopLossTooHigh);
            }
        }

        let position_value = price.value() * quantity.value();
        if position_value < 10.0 {
            return Err(OrderError::PositionTooSmall);
        }

        Ok(Order {
            ticker,
            price,
            quantity,
            stop_loss,
        })
    }

    fn position_value(&self) -> f64 {
        self.price.value() * self.quantity.value()
    }

    fn summary(&self) -> String {
        let sl_info = match &self.stop_loss {
            Some(sl) => format!(" | SL: ${:.2}", sl.value()),
            None => String::new(),
        };

        format!(
            "Order: {} | {:.4} @ ${:.2} | Value: ${:.2}{}",
            self.ticker.as_str(),
            self.quantity.value(),
            self.price.value(),
            self.position_value(),
            sl_info
        )
    }
}
```

## Newtype для идентификаторов

```rust
fn main() {
    let order_id = OrderId::generate();
    let trade_id = TradeId::generate();

    println!("Order ID: {}", order_id.as_str());
    println!("Trade ID: {}", trade_id.as_str());

    // Это не скомпилируется — разные типы!
    // process_order(trade_id); // Ошибка!
    process_order(order_id);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OrderId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TradeId(String);

impl OrderId {
    fn generate() -> OrderId {
        OrderId(format!("ORD-{}", random_suffix()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl TradeId {
    fn generate() -> TradeId {
        TradeId(format!("TRD-{}", random_suffix()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

fn random_suffix() -> u32 {
    // Упрощённая генерация для примера
    12345
}

fn process_order(order_id: OrderId) {
    println!("Processing order: {}", order_id.as_str());
}
```

## Паттерн Parse, don't validate

Вместо валидации данных в каждой функции, валидируем один раз при создании:

```rust
fn main() {
    // ❌ Плохо: валидация в каждой функции
    // fn calculate_risk(balance: f64, risk_pct: f64) -> f64 {
    //     assert!(balance > 0.0);
    //     assert!(risk_pct > 0.0 && risk_pct <= 100.0);
    //     balance * risk_pct / 100.0
    // }

    // ✅ Хорошо: валидация при создании типа
    let balance = Balance::new(10000.0).expect("Invalid balance");
    let risk = RiskPercent::new(2.0).expect("Invalid risk");

    // Функция принимает уже валидированные типы
    let risk_amount = calculate_risk(balance, risk);
    println!("Risk amount: ${:.2}", risk_amount);
}

#[derive(Debug, Clone, Copy)]
struct Balance(f64);

#[derive(Debug, Clone, Copy)]
struct RiskPercent(f64);

impl Balance {
    fn new(value: f64) -> Option<Balance> {
        if value >= 0.0 && value.is_finite() {
            Some(Balance(value))
        } else {
            None
        }
    }
    fn value(&self) -> f64 { self.0 }
}

impl RiskPercent {
    fn new(value: f64) -> Option<RiskPercent> {
        if value > 0.0 && value <= 100.0 {
            Some(RiskPercent(value))
        } else {
            None
        }
    }
    fn value(&self) -> f64 { self.0 }
}

// Функция не может получить невалидные данные!
fn calculate_risk(balance: Balance, risk: RiskPercent) -> f64 {
    balance.value() * risk.value() / 100.0
}
```

## Что мы узнали

| Концепция | Описание | Пример |
|-----------|----------|--------|
| Newtype | Структура-обёртка с одним полем | `struct Price(f64)` |
| Типобезопасность | Компилятор не даст перепутать типы | `Price` ≠ `Quantity` |
| Валидация | Проверка при создании | `Price::new()` |
| Инкапсуляция | Доступ только через методы | `price.value()` |
| Parse, don't validate | Валидируй один раз | Типы гарантируют корректность |

## Упражнения

1. **Создай newtype `Leverage`** для кредитного плеча (от 1x до 125x):
   ```rust
   // Leverage::new(10) -> Ok(Leverage)
   // Leverage::new(200) -> Err(LeverageError::TooHigh)
   ```

2. **Реализуй newtype `Email`** для email-адресов с базовой валидацией (должен содержать @)

3. **Добавь арифметику** к типу `Balance`:
   - `Balance + Balance = Balance`
   - `Balance - Balance = Result<Balance, BalanceError>` (нельзя уйти в минус)

4. **Создай систему типов** для торговли:
   - `BaseAsset` и `QuoteAsset` (нельзя перепутать BTC и USDT)
   - `TradingPair` который объединяет их

## Домашнее задание

1. Создай полную систему типов для ордера с newtype для всех полей:
   - `OrderId`, `Price`, `Quantity`, `Leverage`, `Ticker`
   - Все типы должны валидироваться при создании

2. Реализуй калькулятор позиции с типобезопасными вычислениями:
   - `PositionSize = Balance / Price`
   - `PositionValue = Price * Quantity`
   - `PnL = (ExitPrice - EntryPrice) * Quantity`

3. Создай newtype `Timestamp` для временных меток с валидацией (не в будущем, не раньше 2009 года — год создания биткоина)

4. Реализуй конвертер валют с newtype для каждой валюты:
   ```rust
   let btc = Btc::new(1.0)?;
   let rate = ExchangeRate::new(42000.0)?;
   let usdt: Usdt = btc.convert(rate);
   ```

## Навигация

[← Предыдущий день](../111-input-validation/ru.md) | [Следующий день →](../113-builder-pattern/ru.md)
