# День 66: Tuple Structs — Price(f64)

## Аналогия из трейдинга

В торговле мы работаем с множеством числовых значений: цена, количество, объём, комиссия. Все они могут быть `f64`, но **смешивать их — опасно**:

```rust
// Опасно! Можно случайно перепутать
fn execute_order(price: f64, quantity: f64, fee: f64) { ... }

// Какой аргумент куда? Легко ошибиться!
execute_order(0.5, 42000.0, 0.001);  // Упс! Перепутали price и quantity
```

**Tuple struct** позволяет создавать **типы-обёртки**, которые компилятор различает:

```rust
struct Price(f64);
struct Quantity(f64);

// Теперь компилятор не даст перепутать!
fn execute_order(price: Price, quantity: Quantity) { ... }
```

Это как если бы у каждого значения был свой **ярлык** — "это цена", "это количество".

## Что такое Tuple Struct?

Tuple struct — это именованный кортеж. Он похож на обычную структуру, но поля не имеют имён, а доступ к ним осуществляется по индексу:

```rust
fn main() {
    // Обычный кортеж
    let price_tuple: (f64,) = (42000.0,);

    // Tuple struct — именованный кортеж
    struct Price(f64);
    let price = Price(42000.0);

    // Доступ по индексу
    println!("Price: {}", price.0);
}
```

## Создание Tuple Structs

```rust
// Один элемент — newtype pattern
struct Price(f64);
struct Quantity(f64);
struct Volume(f64);
struct Percentage(f64);

// Несколько элементов
struct BidAsk(f64, f64);
struct OHLC(f64, f64, f64, f64);

fn main() {
    let btc_price = Price(42000.0);
    let trade_qty = Quantity(0.5);
    let spread = BidAsk(42000.0, 42010.0);
    let candle = OHLC(42000.0, 42500.0, 41800.0, 42200.0);

    println!("BTC Price: ${}", btc_price.0);
    println!("Trade Quantity: {}", trade_qty.0);
    println!("Bid: {}, Ask: {}", spread.0, spread.1);
    println!("O: {}, H: {}, L: {}, C: {}", candle.0, candle.1, candle.2, candle.3);
}
```

## Newtype Pattern — защита от ошибок

Главное преимущество tuple struct — **типобезопасность**:

```rust
struct Price(f64);
struct Quantity(f64);
struct Fee(f64);

fn calculate_cost(price: Price, quantity: Quantity, fee: Fee) -> f64 {
    let subtotal = price.0 * quantity.0;
    let fee_amount = subtotal * fee.0;
    subtotal + fee_amount
}

fn main() {
    let price = Price(42000.0);
    let qty = Quantity(0.5);
    let fee = Fee(0.001);

    let cost = calculate_cost(price, qty, fee);
    println!("Total cost: ${:.2}", cost);

    // Это НЕ скомпилируется — типы разные!
    // let wrong = calculate_cost(qty, price, fee);
    // error: expected `Price`, found `Quantity`
}
```

## Деструктуризация

```rust
struct BidAsk(f64, f64);
struct OHLCV(f64, f64, f64, f64, f64);

fn main() {
    let spread = BidAsk(42000.0, 42010.0);
    let candle = OHLCV(42000.0, 42500.0, 41800.0, 42200.0, 150.5);

    // Деструктуризация
    let BidAsk(bid, ask) = spread;
    println!("Spread: {}", ask - bid);

    // Частичная деструктуризация
    let OHLCV(open, _, _, close, _) = candle;
    println!("Change: {}", close - open);
}
```

## Реализация методов

```rust
#[derive(Debug, Clone, Copy)]
struct Price(f64);

impl Price {
    fn new(value: f64) -> Self {
        Price(value)
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn with_slippage(&self, percent: f64) -> Price {
        Price(self.0 * (1.0 + percent / 100.0))
    }

    fn spread_to(&self, other: &Price) -> f64 {
        (other.0 - self.0).abs()
    }
}

fn main() {
    let bid = Price::new(42000.0);
    let ask = Price::new(42015.0);

    println!("Bid: ${}", bid.value());
    println!("Ask: ${}", ask.value());
    println!("Spread: ${}", bid.spread_to(&ask));

    let with_slippage = bid.with_slippage(0.1);
    println!("Bid + 0.1% slippage: ${:.2}", with_slippage.value());
}
```

## Derive-макросы

```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Price(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Quantity(f64);

fn main() {
    let price1 = Price(42000.0);
    let price2 = Price(42000.0);
    let price3 = Price(42100.0);

    // Debug
    println!("{:?}", price1);  // Price(42000.0)

    // Clone, Copy
    let price_copy = price1;
    println!("Original: {:?}, Copy: {:?}", price1, price_copy);

    // PartialEq
    println!("price1 == price2: {}", price1 == price2);  // true
    println!("price1 == price3: {}", price1 == price3);  // false

    // PartialOrd
    println!("price1 < price3: {}", price1 < price3);    // true
}
```

## Практический пример: типобезопасный ордер

```rust
#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug, Clone, Copy)]
struct Fee(f64);

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    price: Price,
    quantity: Quantity,
}

impl Order {
    fn new(symbol: &str, side: &str, price: Price, quantity: Quantity) -> Self {
        Order {
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
        }
    }

    fn notional(&self) -> f64 {
        self.price.0 * self.quantity.0
    }

    fn with_fee(&self, fee: Fee) -> f64 {
        self.notional() * (1.0 + fee.0)
    }
}

fn main() {
    let order = Order::new(
        "BTC/USDT",
        "BUY",
        Price(42000.0),
        Quantity(0.5),
    );

    let fee = Fee(0.001);

    println!("{:?}", order);
    println!("Notional: ${:.2}", order.notional());
    println!("With fee: ${:.2}", order.with_fee(fee));
}
```

## Практический пример: PnL калькулятор

```rust
#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug, Clone, Copy)]
struct PnL(f64);

struct Position {
    symbol: String,
    entry_price: Price,
    quantity: Quantity,
    is_long: bool,
}

impl Position {
    fn new(symbol: &str, entry: Price, qty: Quantity, is_long: bool) -> Self {
        Position {
            symbol: symbol.to_string(),
            entry_price: entry,
            quantity: qty,
            is_long,
        }
    }

    fn calculate_pnl(&self, current_price: Price) -> PnL {
        let diff = current_price.0 - self.entry_price.0;
        let pnl = if self.is_long {
            diff * self.quantity.0
        } else {
            -diff * self.quantity.0
        };
        PnL(pnl)
    }

    fn calculate_pnl_percent(&self, current_price: Price) -> f64 {
        let pnl = self.calculate_pnl(current_price);
        let entry_value = self.entry_price.0 * self.quantity.0;
        (pnl.0 / entry_value) * 100.0
    }
}

fn main() {
    let position = Position::new(
        "BTC/USDT",
        Price(42000.0),
        Quantity(0.5),
        true,  // Long
    );

    let prices = [
        Price(42500.0),
        Price(41500.0),
        Price(43000.0),
        Price(40000.0),
    ];

    println!("=== PnL Calculator ===");
    println!("Entry: ${}", position.entry_price.0);
    println!("Quantity: {}", position.quantity.0);
    println!("Side: {}", if position.is_long { "LONG" } else { "SHORT" });
    println!();

    for price in prices {
        let pnl = position.calculate_pnl(price);
        let pnl_pct = position.calculate_pnl_percent(price);

        println!(
            "Price: ${:.2} | PnL: {:+.2} ({:+.2}%)",
            price.0, pnl.0, pnl_pct
        );
    }
}
```

## Практический пример: риск-менеджмент

```rust
#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug, Clone, Copy)]
struct RiskPercent(f64);

#[derive(Debug, Clone, Copy)]
struct Balance(f64);

struct RiskCalculator {
    balance: Balance,
    risk_per_trade: RiskPercent,
}

impl RiskCalculator {
    fn new(balance: Balance, risk: RiskPercent) -> Self {
        RiskCalculator {
            balance,
            risk_per_trade: risk,
        }
    }

    fn max_loss(&self) -> f64 {
        self.balance.0 * (self.risk_per_trade.0 / 100.0)
    }

    fn position_size(&self, entry: Price, stop_loss: Price) -> Quantity {
        let risk_amount = self.max_loss();
        let price_diff = (entry.0 - stop_loss.0).abs();
        Quantity(risk_amount / price_diff)
    }

    fn stop_loss_price(&self, entry: Price, quantity: Quantity, is_long: bool) -> Price {
        let risk_amount = self.max_loss();
        let price_diff = risk_amount / quantity.0;

        if is_long {
            Price(entry.0 - price_diff)
        } else {
            Price(entry.0 + price_diff)
        }
    }
}

fn main() {
    let calculator = RiskCalculator::new(
        Balance(10000.0),
        RiskPercent(2.0),  // 2% risk per trade
    );

    let entry = Price(42000.0);
    let stop = Price(41000.0);

    println!("=== Risk Management ===");
    println!("Balance: ${}", calculator.balance.0);
    println!("Risk per trade: {}%", calculator.risk_per_trade.0);
    println!("Max loss: ${}", calculator.max_loss());
    println!();

    let size = calculator.position_size(entry, stop);
    println!("Entry: ${}, Stop: ${}", entry.0, stop.0);
    println!("Recommended position size: {:.4} BTC", size.0);
    println!();

    // Обратная задача
    let fixed_qty = Quantity(0.1);
    let calc_stop = calculator.stop_loss_price(entry, fixed_qty, true);
    println!("With qty {}: Stop loss at ${:.2}", fixed_qty.0, calc_stop.0);
}
```

## Практический пример: конверсия валют

```rust
#[derive(Debug, Clone, Copy)]
struct USD(f64);

#[derive(Debug, Clone, Copy)]
struct BTC(f64);

#[derive(Debug, Clone, Copy)]
struct ETH(f64);

// Курсы обмена
struct ExchangeRates {
    btc_usd: f64,
    eth_usd: f64,
}

impl ExchangeRates {
    fn usd_to_btc(&self, usd: USD) -> BTC {
        BTC(usd.0 / self.btc_usd)
    }

    fn btc_to_usd(&self, btc: BTC) -> USD {
        USD(btc.0 * self.btc_usd)
    }

    fn usd_to_eth(&self, usd: USD) -> ETH {
        ETH(usd.0 / self.eth_usd)
    }

    fn eth_to_usd(&self, eth: ETH) -> USD {
        USD(eth.0 * self.eth_usd)
    }

    fn btc_to_eth(&self, btc: BTC) -> ETH {
        let usd = self.btc_to_usd(btc);
        self.usd_to_eth(usd)
    }
}

fn main() {
    let rates = ExchangeRates {
        btc_usd: 42000.0,
        eth_usd: 2200.0,
    };

    let investment = USD(10000.0);

    println!("=== Currency Conversion ===");
    println!("Investment: ${:.2}", investment.0);
    println!();

    let in_btc = rates.usd_to_btc(investment);
    let in_eth = rates.usd_to_eth(investment);

    println!("In BTC: {:.6} BTC", in_btc.0);
    println!("In ETH: {:.4} ETH", in_eth.0);
    println!();

    // Обратная конверсия
    let one_btc = BTC(1.0);
    let btc_value = rates.btc_to_usd(one_btc);
    let btc_in_eth = rates.btc_to_eth(one_btc);

    println!("1 BTC = ${:.2}", btc_value.0);
    println!("1 BTC = {:.4} ETH", btc_in_eth.0);
}
```

## Сравнение: кортеж vs tuple struct vs struct

```rust
fn main() {
    // Обычный кортеж — анонимный
    let tuple: (f64, f64) = (42000.0, 42010.0);

    // Tuple struct — именованный кортеж
    struct BidAsk(f64, f64);
    let tuple_struct = BidAsk(42000.0, 42010.0);

    // Обычная структура — именованные поля
    struct Spread {
        bid: f64,
        ask: f64,
    }
    let regular_struct = Spread { bid: 42000.0, ask: 42010.0 };

    // Доступ
    println!("Tuple: {}, {}", tuple.0, tuple.1);
    println!("Tuple struct: {}, {}", tuple_struct.0, tuple_struct.1);
    println!("Struct: {}, {}", regular_struct.bid, regular_struct.ask);
}
```

| Тип | Когда использовать |
|-----|-------------------|
| Кортеж | Временная группировка значений |
| Tuple struct | Newtype pattern, типобезопасность |
| Struct | Много полей, нужны именованные поля |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `struct Name(T)` | Объявление tuple struct |
| `Name(value)` | Создание экземпляра |
| `instance.0` | Доступ к полю |
| `let Name(x) = instance` | Деструктуризация |
| Newtype pattern | Обёртка для типобезопасности |

## Домашнее задание

1. Создай tuple structs для: `OrderId(u64)`, `Symbol(String)`, `Timestamp(u64)`. Реализуй методы `new()` и `value()` для каждого.

2. Напиши функцию расчёта комиссии, которая принимает `Price`, `Quantity`, `FeeRate` и возвращает `Fee`. Компилятор должен не давать перепутать аргументы.

3. Создай `Portfolio` struct с балансами в разных валютах (`USD`, `BTC`, `ETH`). Реализуй метод `total_in_usd()`, который конвертирует всё в доллары.

4. Реализуй калькулятор размера позиции с типами `AccountBalance`, `RiskPercent`, `EntryPrice`, `StopLossPrice`, `PositionSize`. Все входные данные должны быть типобезопасными.

## Навигация

[← Предыдущий день](../065-structs-intro/ru.md) | [Следующий день →](../067-struct-fields/ru.md)
