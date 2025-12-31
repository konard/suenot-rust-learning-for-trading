# День 70: Enum — OrderSide: покупка или продажа

## Аналогия из трейдинга

В торговле каждый ордер имеет **направление**:
- **Buy** (покупка) — открываем длинную позицию или закрываем короткую
- **Sell** (продажа) — открываем короткую позицию или закрываем длинную

Это **взаимоисключающие** состояния — ордер не может быть одновременно на покупку и продажу. В Rust для моделирования таких ситуаций используются **перечисления (enums)**.

## Что такое Enum?

Enum (перечисление) — это тип данных, который может принимать одно из нескольких заранее определённых значений:

```rust
// Определяем enum для направления ордера
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let my_order = OrderSide::Buy;
    let another_order = OrderSide::Sell;

    println!("Ордер создан!");
}
```

## Создание Enum

```rust
enum OrderSide {
    Buy,   // Вариант 1: покупка
    Sell,  // Вариант 2: продажа
}

fn main() {
    // Создаём значения enum
    let side1 = OrderSide::Buy;
    let side2 = OrderSide::Sell;

    // Используем в структуре (забегая вперёд)
    println!("Ордера созданы");
}
```

## Сопоставление с образцом (Pattern Matching)

Для работы с enum используется конструкция `match`:

```rust
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Buy;

    // match проверяет все варианты
    match side {
        OrderSide::Buy => println!("Покупаем актив"),
        OrderSide::Sell => println!("Продаём актив"),
    }
}
```

### Возврат значений из match

```rust
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Sell;

    let action = match side {
        OrderSide::Buy => "LONG",
        OrderSide::Sell => "SHORT",
    };

    println!("Направление позиции: {}", action);
}
```

## Enum с данными

Варианты enum могут содержать данные:

```rust
enum OrderType {
    Market,                      // Рыночный ордер (без данных)
    Limit(f64),                  // Лимитный ордер (с ценой)
    StopLoss { price: f64, trigger: f64 },  // Стоп-лосс (именованные поля)
}

fn main() {
    let order1 = OrderType::Market;
    let order2 = OrderType::Limit(42000.0);
    let order3 = OrderType::StopLoss {
        price: 41000.0,
        trigger: 41500.0
    };

    match order1 {
        OrderType::Market => println!("Рыночный ордер — исполнится сразу"),
        OrderType::Limit(price) => println!("Лимитный ордер по цене {}", price),
        OrderType::StopLoss { price, trigger } => {
            println!("Стоп-лосс: сработает при {} , исполнится по {}", trigger, price);
        }
    }
}
```

## Практический пример: OrderSide

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Buy;
    let price = 42000.0;
    let quantity = 0.5;

    // Расчёт стоимости позиции
    let position_value = price * quantity;

    // Определяем знак позиции
    let signed_quantity = match side {
        OrderSide::Buy => quantity,
        OrderSide::Sell => -quantity,
    };

    println!("╔════════════════════════════════╗");
    println!("║         ORDER DETAILS          ║");
    println!("╠════════════════════════════════╣");
    println!("║ Side:     {:?}                 ║", side);
    println!("║ Price:    ${:.2}           ║", price);
    println!("║ Quantity: {:.4} BTC           ║", quantity);
    println!("║ Value:    ${:.2}           ║", position_value);
    println!("║ Signed:   {:+.4}              ║", signed_quantity);
    println!("╚════════════════════════════════╝");
}
```

## Практический пример: расчёт PnL

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    // Открываем позицию
    let entry_side = OrderSide::Buy;
    let entry_price = 42000.0;
    let quantity = 0.5;

    // Текущая цена
    let current_price = 43500.0;

    // Расчёт PnL зависит от направления
    let pnl = calculate_pnl(entry_side, entry_price, current_price, quantity);

    println!("Entry: {:?} @ ${:.2}", entry_side, entry_price);
    println!("Current: ${:.2}", current_price);
    println!("PnL: ${:+.2}", pnl);
}

fn calculate_pnl(side: OrderSide, entry: f64, current: f64, qty: f64) -> f64 {
    match side {
        OrderSide::Buy => (current - entry) * qty,  // Лонг: растёт = профит
        OrderSide::Sell => (entry - current) * qty, // Шорт: падает = профит
    }
}
```

## Практический пример: торговый сигнал

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum Signal {
    Enter(OrderSide),
    Exit(OrderSide),
    Hold,
}

fn main() {
    let prices = [42000.0, 42500.0, 42300.0, 43000.0, 42800.0];

    println!("=== Trading Signals ===\n");

    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let curr = prices[i];

        let signal = generate_signal(prev, curr);

        let action = match &signal {
            Signal::Enter(side) => format!("ENTER {:?}", side),
            Signal::Exit(side) => format!("EXIT {:?}", side),
            Signal::Hold => "HOLD".to_string(),
        };

        println!("Price: {:.2} -> {:.2} | Signal: {}", prev, curr, action);
    }
}

fn generate_signal(prev_price: f64, current_price: f64) -> Signal {
    let change_percent = (current_price - prev_price) / prev_price * 100.0;

    if change_percent > 1.0 {
        Signal::Enter(OrderSide::Buy)  // Сильный рост — покупаем
    } else if change_percent < -1.0 {
        Signal::Enter(OrderSide::Sell) // Сильное падение — продаём
    } else {
        Signal::Hold // Боковик — ждём
    }
}
```

## Практический пример: книга ордеров

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

fn main() {
    let orders = vec![
        Order { id: 1, side: OrderSide::Buy, price: 41990.0, quantity: 1.5 },
        Order { id: 2, side: OrderSide::Buy, price: 41980.0, quantity: 2.0 },
        Order { id: 3, side: OrderSide::Sell, price: 42010.0, quantity: 0.8 },
        Order { id: 4, side: OrderSide::Sell, price: 42020.0, quantity: 1.2 },
        Order { id: 5, side: OrderSide::Buy, price: 41970.0, quantity: 3.0 },
    ];

    // Разделяем ордера по сторонам
    let bids: Vec<&Order> = orders.iter()
        .filter(|o| o.side == OrderSide::Buy)
        .collect();

    let asks: Vec<&Order> = orders.iter()
        .filter(|o| o.side == OrderSide::Sell)
        .collect();

    println!("=== ORDER BOOK ===\n");

    println!("BIDS (покупка):");
    for order in &bids {
        println!("  ${:.2} x {:.4}", order.price, order.quantity);
    }

    println!("\nASKS (продажа):");
    for order in &asks {
        println!("  ${:.2} x {:.4}", order.price, order.quantity);
    }

    // Лучшие цены
    let best_bid = bids.iter().map(|o| o.price).fold(f64::MIN, f64::max);
    let best_ask = asks.iter().map(|o| o.price).fold(f64::MAX, f64::min);

    println!("\n--- Spread ---");
    println!("Best Bid: ${:.2}", best_bid);
    println!("Best Ask: ${:.2}", best_ask);
    println!("Spread:   ${:.2}", best_ask - best_bid);
}
```

## Методы для Enum

Можно добавлять методы к enum через `impl`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    // Противоположная сторона
    fn opposite(&self) -> OrderSide {
        match self {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }

    // Знак для расчётов
    fn sign(&self) -> f64 {
        match self {
            OrderSide::Buy => 1.0,
            OrderSide::Sell => -1.0,
        }
    }

    // Строковое представление
    fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

fn main() {
    let side = OrderSide::Buy;

    println!("Side: {}", side.as_str());
    println!("Sign: {}", side.sign());
    println!("Opposite: {}", side.opposite().as_str());

    // Использование в расчётах
    let quantity = 0.5;
    let signed_qty = quantity * side.sign();
    println!("Signed quantity: {:+.4}", signed_qty);
}
```

## Полный пример: система ордеров

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    filled: f64,
    status: OrderStatus,
}

impl Order {
    fn new(id: u64, symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Self {
        Order {
            id,
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            filled: 0.0,
            status: OrderStatus::Pending,
        }
    }

    fn fill(&mut self, amount: f64) {
        self.filled += amount;

        if self.filled >= self.quantity {
            self.filled = self.quantity;
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }

    fn cancel(&mut self) {
        if self.status != OrderStatus::Filled {
            self.status = OrderStatus::Cancelled;
        }
    }

    fn remaining(&self) -> f64 {
        self.quantity - self.filled
    }

    fn fill_percent(&self) -> f64 {
        (self.filled / self.quantity) * 100.0
    }
}

fn main() {
    let mut order = Order::new(1, "BTC/USDT", OrderSide::Buy, 42000.0, 1.0);

    println!("=== Order Lifecycle ===\n");

    println!("Created: {:?}", order.status);
    println!("Remaining: {:.4}\n", order.remaining());

    // Частичное исполнение
    order.fill(0.3);
    println!("After partial fill:");
    println!("  Status: {:?}", order.status);
    println!("  Filled: {:.1}%", order.fill_percent());
    println!("  Remaining: {:.4}\n", order.remaining());

    // Полное исполнение
    order.fill(0.7);
    println!("After full fill:");
    println!("  Status: {:?}", order.status);
    println!("  Filled: {:.1}%", order.fill_percent());

    // Попытка отменить исполненный ордер
    order.cancel();
    println!("\nAfter cancel attempt: {:?}", order.status);
}
```

## Сравнение Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side1 = OrderSide::Buy;
    let side2 = OrderSide::Buy;
    let side3 = OrderSide::Sell;

    println!("Buy == Buy: {}", side1 == side2);   // true
    println!("Buy == Sell: {}", side1 == side3);  // false
    println!("Buy != Sell: {}", side1 != side3);  // true

    // Использование в условиях
    if side1 == OrderSide::Buy {
        println!("Это ордер на покупку");
    }
}
```

## if let — упрощённый match

Когда нужно проверить только один вариант:

```rust
#[derive(Debug)]
enum OrderType {
    Market,
    Limit(f64),
    StopLoss(f64),
}

fn main() {
    let order = OrderType::Limit(42000.0);

    // Вместо полного match
    if let OrderType::Limit(price) = order {
        println!("Лимитный ордер по цене: ${:.2}", price);
    }

    // С else
    let order2 = OrderType::Market;

    if let OrderType::Limit(price) = order2 {
        println!("Лимитный ордер: {}", price);
    } else {
        println!("Не лимитный ордер");
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `enum Name { A, B }` | Определение перечисления |
| `Name::A` | Обращение к варианту |
| `match value { ... }` | Сопоставление с образцом |
| `enum Name { A(T) }` | Вариант с данными |
| `impl Name { }` | Методы для enum |
| `if let Pattern = value` | Упрощённый match |
| `#[derive(PartialEq)]` | Сравнение вариантов |

## Домашнее задание

1. Создай enum `TimeInForce` с вариантами: `GTC` (Good Till Cancel), `IOC` (Immediate Or Cancel), `FOK` (Fill Or Kill). Добавь метод `description()`, возвращающий описание каждого типа.

2. Создай enum `OrderType` с вариантами:
   - `Market` — без данных
   - `Limit(f64)` — с ценой
   - `StopMarket(f64)` — с триггерной ценой
   - `StopLimit { trigger: f64, price: f64 }` — с двумя ценами

   Напиши функцию `describe_order`, которая выводит описание ордера.

3. Реализуй функцию `process_signals`, которая принимает массив сигналов `Signal::Buy` или `Signal::Sell` и подсчитывает количество каждого типа.

4. Создай полноценную структуру `Position` с enum `PositionSide { Long, Short, Flat }`. Добавь методы:
   - `open(side, price, quantity)` — открыть позицию
   - `close(price)` — закрыть позицию и вернуть PnL
   - `is_open()` — проверить, открыта ли позиция

## Навигация

[← Предыдущий день](../069-review-week-9/ru.md) | [Следующий день →](../071-enum-order-type/ru.md)
