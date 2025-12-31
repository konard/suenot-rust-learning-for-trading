# День 42: Висячие ссылки (Dangling References)

## Аналогия из трейдинга

Представь ситуацию: ты сохранил ID ордера, чтобы позже проверить его статус. Но пока ты занимался другими делами, система отменила и **удалила** этот ордер. Теперь твой ID указывает на несуществующий ордер — это **висячая ссылка**.

В C/C++ такая ситуация приводит к непредсказуемому поведению: чтение мусора из памяти или крашу программы. Rust **не позволит** создать висячую ссылку — компилятор остановит тебя ещё до запуска программы.

## Что такое Dangling Reference?

Dangling reference (висячая ссылка) — это ссылка, которая указывает на память, которая уже была освобождена. В Rust это невозможно благодаря системе владения (ownership).

```rust
// ЭТО НЕ СКОМПИЛИРУЕТСЯ!
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String {  // Ошибка! Возвращаем ссылку на локальную переменную
    let order_id = String::from("ORD-12345");
    &order_id  // order_id будет удалён после выхода из функции!
}
```

Компилятор выдаст ошибку:
```
error[E0106]: missing lifetime specifier
  --> src/main.rs:6:16
   |
6  | fn dangle() -> &String {
   |                ^ expected named lifetime parameter
```

## Почему это важно в трейдинге?

В торговых системах данные постоянно появляются и исчезают:
- Ордера исполняются и удаляются
- Тикеры делистятся с бирж
- Исторические данные очищаются
- Соединения с биржами закрываются

Если твой код держит ссылку на удалённые данные — катастрофа неизбежна.

## Примеры опасных ситуаций

### Пример 1: Возврат ссылки на локальную переменную

```rust
// ЭТО НЕ СКОМПИЛИРУЕТСЯ!
fn get_best_ticker() -> &str {
    let ticker = String::from("BTC/USDT");
    &ticker  // ticker умрёт здесь, ссылка станет висячей
}
```

### Решение: Вернуть владение

```rust
fn get_best_ticker() -> String {
    let ticker = String::from("BTC/USDT");
    ticker  // Передаём владение вызывающему коду
}

fn main() {
    let ticker = get_best_ticker();
    println!("Best ticker: {}", ticker);
}
```

### Пример 2: Ссылка на удалённый ордер

```rust
// ЭТО НЕ СКОМПИЛИРУЕТСЯ!
fn main() {
    let order_ref;

    {
        let order = String::from("ORD-BTC-001");
        order_ref = &order;  // order_ref ссылается на order
    }  // order удаляется здесь!

    println!("Order: {}", order_ref);  // ОШИБКА: order уже не существует
}
```

Компилятор сообщит:
```
error[E0597]: `order` does not live long enough
```

### Решение: Правильные времена жизни

```rust
fn main() {
    let order = String::from("ORD-BTC-001");
    let order_ref = &order;  // Ссылка живёт, пока живёт order

    println!("Order: {}", order_ref);  // OK!
}
```

## Безопасные паттерны для трейдинга

### Паттерн 1: Возврат владения вместо ссылки

```rust
struct Order {
    id: String,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn create_market_order(symbol: &str, quantity: f64, current_price: f64) -> Order {
    Order {
        id: format!("ORD-{}", chrono_placeholder()),
        symbol: symbol.to_string(),
        price: current_price,
        quantity,
    }
}

fn chrono_placeholder() -> u64 {
    1234567890  // Заглушка для примера
}

fn main() {
    let order = create_market_order("BTC/USDT", 0.5, 42000.0);
    println!("Created order: {} for {} {}", order.id, order.quantity, order.symbol);
}
```

### Паттерн 2: Клонирование при необходимости

```rust
fn main() {
    let tickers = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    // Получаем копию, а не ссылку
    let best_ticker = find_best_ticker(&tickers);

    // tickers может быть изменён, best_ticker остаётся валидным
    println!("Best: {}", best_ticker);
}

fn find_best_ticker(tickers: &[&str]) -> String {
    // Возвращаем String, а не &str
    tickers.first().unwrap_or(&"UNKNOWN").to_string()
}
```

### Паттерн 3: Option для отсутствующих данных

```rust
struct OrderBook {
    orders: Vec<Order>,
}

struct Order {
    id: String,
    price: f64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook { orders: Vec::new() }
    }

    fn add_order(&mut self, id: String, price: f64) {
        self.orders.push(Order { id, price });
    }

    // Возвращаем Option, а не голую ссылку
    fn find_order(&self, id: &str) -> Option<&Order> {
        self.orders.iter().find(|o| o.id == id)
    }

    // Безопасное удаление с проверкой
    fn cancel_order(&mut self, id: &str) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            Some(self.orders.remove(pos))
        } else {
            None
        }
    }
}

fn main() {
    let mut book = OrderBook::new();
    book.add_order("ORD-001".to_string(), 42000.0);
    book.add_order("ORD-002".to_string(), 42100.0);

    // Безопасный поиск
    match book.find_order("ORD-001") {
        Some(order) => println!("Found: {} at {}", order.id, order.price),
        None => println!("Order not found"),
    }

    // Безопасное удаление
    if let Some(cancelled) = book.cancel_order("ORD-001") {
        println!("Cancelled: {}", cancelled.id);
    }

    // Повторный поиск удалённого ордера
    match book.find_order("ORD-001") {
        Some(_) => println!("Still exists"),
        None => println!("Order no longer exists"),  // Это выведется
    }
}
```

## Распространённые ошибки и как их избежать

### Ошибка 1: Попытка вернуть ссылку на вычисленное значение

```rust
// НЕ СКОМПИЛИРУЕТСЯ!
fn calculate_spread(bid: f64, ask: f64) -> &f64 {
    let spread = ask - bid;
    &spread  // spread будет удалён!
}
```

**Решение:** Возвращай значение, а не ссылку:

```rust
fn calculate_spread(bid: f64, ask: f64) -> f64 {
    ask - bid
}

fn main() {
    let spread = calculate_spread(42000.0, 42010.0);
    println!("Spread: ${:.2}", spread);
}
```

### Ошибка 2: Сохранение ссылки при изменении коллекции

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // Это безопасно:
    let first = prices[0];  // Копия значения
    prices.push(42200.0);   // Изменение вектора
    println!("First: {}", first);  // OK!

    // А это было бы опасно (Rust не позволит):
    // let first_ref = &prices[0];
    // prices.push(42200.0);  // ОШИБКА: нельзя изменять при активной ссылке
    // println!("First: {}", first_ref);
}
```

### Ошибка 3: Возврат ссылки на временное значение

```rust
// НЕ СКОМПИЛИРУЕТСЯ!
fn get_formatted_price(price: f64) -> &str {
    let formatted = format!("${:.2}", price);
    &formatted  // Временная строка будет удалена!
}
```

**Решение:**

```rust
fn get_formatted_price(price: f64) -> String {
    format!("${:.2}", price)
}

fn main() {
    let price_str = get_formatted_price(42000.0);
    println!("Price: {}", price_str);
}
```

## Практический пример: Безопасный менеджер портфолио

```rust
#[derive(Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct Portfolio {
    positions: Vec<Position>,
    balance: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            positions: Vec::new(),
            balance: initial_balance,
        }
    }

    // Возвращает копию позиции (безопасно)
    fn get_position(&self, symbol: &str) -> Option<Position> {
        self.positions
            .iter()
            .find(|p| p.symbol == symbol)
            .cloned()  // Клонируем, чтобы не зависеть от времени жизни
    }

    // Открытие позиции
    fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;
        if cost > self.balance {
            return Err("Insufficient balance".to_string());
        }

        self.balance -= cost;
        self.positions.push(Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
        });

        Ok(())
    }

    // Закрытие позиции — возвращает PnL
    fn close_position(&mut self, symbol: &str, exit_price: f64) -> Result<f64, String> {
        let pos_index = self.positions
            .iter()
            .position(|p| p.symbol == symbol)
            .ok_or_else(|| format!("Position {} not found", symbol))?;

        let position = self.positions.remove(pos_index);
        let pnl = (exit_price - position.entry_price) * position.quantity;
        let proceeds = position.quantity * exit_price;
        self.balance += proceeds;

        Ok(pnl)
    }

    // Общий PnL (возвращает значение, не ссылку)
    fn calculate_total_pnl(&self, current_prices: &[(String, f64)]) -> f64 {
        self.positions.iter().map(|pos| {
            let current = current_prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            (current - pos.entry_price) * pos.quantity
        }).sum()
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100000.0);

    // Открываем позиции
    portfolio.open_position("BTC", 0.5, 42000.0).unwrap();
    portfolio.open_position("ETH", 5.0, 2500.0).unwrap();

    // Получаем копию позиции (безопасно)
    if let Some(btc_pos) = portfolio.get_position("BTC") {
        println!("BTC position: {} @ ${}", btc_pos.quantity, btc_pos.entry_price);
    }

    // Расчёт PnL с текущими ценами
    let current_prices = vec![
        ("BTC".to_string(), 43000.0),
        ("ETH".to_string(), 2600.0),
    ];

    let total_pnl = portfolio.calculate_total_pnl(&current_prices);
    println!("Total unrealized PnL: ${:.2}", total_pnl);

    // Закрываем позицию
    match portfolio.close_position("BTC", 43000.0) {
        Ok(pnl) => println!("BTC closed with PnL: ${:.2}", pnl),
        Err(e) => println!("Error: {}", e),
    }

    // Попытка получить закрытую позицию
    match portfolio.get_position("BTC") {
        Some(_) => println!("BTC still open"),
        None => println!("BTC position closed"),  // Это выведется
    }

    println!("Final balance: ${:.2}", portfolio.balance);
}
```

## Сравнение подходов

| Подход | Безопасность | Производительность | Когда использовать |
|--------|--------------|--------------------|--------------------|
| Возврат владения | Высокая | Средняя | Данные нужны надолго |
| Клонирование | Высокая | Ниже | Небольшие данные |
| Ссылки с lifetimes | Высокая | Высокая | Данные живут достаточно долго |
| Option/Result | Высокая | Высокая | Когда данных может не быть |

## Что мы узнали

1. **Dangling reference** — ссылка на освобождённую память
2. Rust **предотвращает** создание висячих ссылок на этапе компиляции
3. Возвращай **владение** вместо ссылки, когда данные создаются внутри функции
4. Используй **Option** для обработки отсутствующих данных
5. **Клонируй** данные, если нужна независимая копия

## Домашнее задание

1. Напиши функцию `create_trade_report(trades: &[Trade]) -> String`, которая создаёт строку отчёта и безопасно возвращает её

2. Реализуй структуру `OrderManager` с методами:
   - `submit_order(&mut self, order: Order) -> String` — возвращает ID ордера
   - `get_order(&self, id: &str) -> Option<Order>` — возвращает копию ордера
   - `cancel_order(&mut self, id: &str) -> Result<Order, String>` — отменяет и возвращает ордер

3. Создай функцию `find_best_price`, которая принимает `&[f64]` и возвращает лучшую цену безопасным способом (подумай, что вернуть: `f64`, `Option<f64>`, или `Result<f64, String>`)

4. Исправь следующий код (он не компилируется):
```rust
fn get_ticker_info(symbol: &str) -> &str {
    let info = format!("{} - Active", symbol);
    &info
}
```

## Навигация

[← Предыдущий день](../041-no-mixing-references/ru.md) | [Следующий день →](../043-lifetimes-order-duration/ru.md)
