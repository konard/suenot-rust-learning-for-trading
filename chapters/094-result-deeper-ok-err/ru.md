# День 94: Result глубже — Ok и Err

## Аналогия из трейдинга

Представь, что ты отправляешь ордер на биржу. Возможны два исхода: **успех** (ордер принят, ты получаешь ID) или **ошибка** (недостаточно средств, биржа недоступна, неверные параметры). Тип `Result<T, E>` в Rust моделирует именно этот паттерн: операция либо возвращает успешное значение типа `T`, либо ошибку типа `E`.

## Анатомия Result

```rust
enum Result<T, E> {
    Ok(T),   // Успех: содержит значение типа T
    Err(E),  // Ошибка: содержит ошибку типа E
}
```

`Result` — это обобщённый enum с двумя вариантами:
- `Ok(T)` — операция успешна, результат внутри
- `Err(E)` — операция провалилась, причина ошибки внутри

## Создание Result значений

```rust
fn main() {
    // Успешные результаты
    let order_id: Result<u64, String> = Ok(12345);
    let balance: Result<f64, &str> = Ok(10000.50);

    // Ошибки
    let failed_order: Result<u64, String> = Err(String::from("Insufficient funds"));
    let api_error: Result<f64, &str> = Err("Connection timeout");

    println!("Order: {:?}", order_id);
    println!("Balance: {:?}", balance);
    println!("Failed: {:?}", failed_order);
    println!("API Error: {:?}", api_error);
}
```

## Обработка Result с match

Базовый и наиболее явный способ обработки:

```rust
fn main() {
    let order_result = place_order("BTCUSDT", 0.1, 42000.0);

    match order_result {
        Ok(order_id) => {
            println!("Order placed successfully! ID: {}", order_id);
        }
        Err(error) => {
            println!("Failed to place order: {}", error);
        }
    }
}

fn place_order(symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    if symbol.is_empty() {
        return Err(String::from("Symbol cannot be empty"));
    }

    // Симуляция успешного размещения ордера
    Ok(1234567890)
}
```

## Методы Result

### is_ok() и is_err()

```rust
fn main() {
    let success: Result<i32, &str> = Ok(42);
    let failure: Result<i32, &str> = Err("error");

    println!("Success is_ok: {}", success.is_ok());   // true
    println!("Success is_err: {}", success.is_err()); // false
    println!("Failure is_ok: {}", failure.is_ok());   // false
    println!("Failure is_err: {}", failure.is_err()); // true

    // Применение в трейдинге
    let order = execute_trade("ETHUSDT", 1.0);
    if order.is_ok() {
        println!("Trade executed, proceeding with portfolio update");
    }
}

fn execute_trade(symbol: &str, qty: f64) -> Result<u64, String> {
    if qty > 0.0 {
        Ok(999)
    } else {
        Err(String::from("Invalid quantity"))
    }
}
```

### unwrap() и expect()

```rust
fn main() {
    // unwrap — паникует при Err, возвращает значение при Ok
    let price: Result<f64, &str> = Ok(42500.0);
    let value = price.unwrap();
    println!("Price: {}", value);

    // expect — то же самое, но с кастомным сообщением
    let balance: Result<f64, &str> = Ok(10000.0);
    let bal = balance.expect("Failed to get balance");
    println!("Balance: {}", bal);

    // ОПАСНО! Это вызовет панику:
    // let error: Result<f64, &str> = Err("API down");
    // error.unwrap(); // panic!
}
```

**Важно:** Используй `unwrap()` только когда на 100% уверен в успехе или в тестах!

### unwrap_or() и unwrap_or_else()

```rust
fn main() {
    let price: Result<f64, &str> = Err("Price unavailable");

    // Значение по умолчанию
    let safe_price = price.unwrap_or(0.0);
    println!("Safe price: {}", safe_price); // 0.0

    // Ленивое вычисление значения по умолчанию
    let calculated: Result<f64, &str> = Err("No data");
    let default = calculated.unwrap_or_else(|err| {
        println!("Error occurred: {}, using fallback", err);
        get_fallback_price()
    });
    println!("Default price: {}", default);
}

fn get_fallback_price() -> f64 {
    40000.0 // Резервная цена
}
```

### unwrap_or_default()

```rust
fn main() {
    let result: Result<String, &str> = Err("error");
    let value = result.unwrap_or_default(); // Пустая строка
    println!("Value: '{}'", value);

    let num_result: Result<i32, &str> = Err("error");
    let num = num_result.unwrap_or_default(); // 0
    println!("Number: {}", num);

    // В трейдинге: количество по умолчанию
    let qty: Result<f64, String> = fetch_position_size("BTCUSDT");
    let position = qty.unwrap_or_default(); // 0.0 если ошибка
    println!("Position size: {}", position);
}

fn fetch_position_size(symbol: &str) -> Result<f64, String> {
    Err(format!("No position for {}", symbol))
}
```

## Трансформация Result

### map() — преобразование Ok

```rust
fn main() {
    let price_result: Result<f64, String> = Ok(42000.0);

    // Конвертация в рубли
    let rub_price = price_result.map(|usd| usd * 90.0);
    println!("Price in RUB: {:?}", rub_price); // Ok(3780000.0)

    // При ошибке map не выполняется
    let error: Result<f64, String> = Err(String::from("No price"));
    let mapped = error.map(|p| p * 2.0);
    println!("Mapped error: {:?}", mapped); // Err("No price")
}
```

### map_err() — преобразование Err

```rust
fn main() {
    let error: Result<f64, &str> = Err("timeout");

    // Преобразование типа ошибки
    let detailed: Result<f64, String> = error.map_err(|e| {
        format!("API Error: {} - please retry", e)
    });
    println!("{:?}", detailed);

    // Добавление контекста к ошибке
    let result = fetch_price("BTCUSDT")
        .map_err(|e| format!("[BTCUSDT] {}", e));
    println!("{:?}", result);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    Err(String::from("Connection refused"))
}
```

### and_then() — цепочка операций

```rust
fn main() {
    let result = validate_symbol("BTCUSDT")
        .and_then(|symbol| fetch_balance(&symbol))
        .and_then(|balance| calculate_position_size(balance, 2.0));

    match result {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error in chain: {}", e),
    }
}

fn validate_symbol(symbol: &str) -> Result<String, String> {
    if symbol.len() >= 6 {
        Ok(symbol.to_string())
    } else {
        Err(String::from("Invalid symbol format"))
    }
}

fn fetch_balance(symbol: &str) -> Result<f64, String> {
    // Симуляция получения баланса
    Ok(10000.0)
}

fn calculate_position_size(balance: f64, risk_pct: f64) -> Result<f64, String> {
    if risk_pct > 0.0 && risk_pct <= 100.0 {
        Ok(balance * (risk_pct / 100.0))
    } else {
        Err(String::from("Invalid risk percentage"))
    }
}
```

### or_else() — восстановление после ошибки

```rust
fn main() {
    let price = fetch_from_primary()
        .or_else(|_| fetch_from_secondary())
        .or_else(|_| fetch_from_cache());

    println!("Price: {:?}", price);
}

fn fetch_from_primary() -> Result<f64, String> {
    Err(String::from("Primary API down"))
}

fn fetch_from_secondary() -> Result<f64, String> {
    Err(String::from("Secondary API down"))
}

fn fetch_from_cache() -> Result<f64, String> {
    Ok(41500.0) // Кэшированная цена
}
```

## Оператор ? — распространение ошибок

```rust
fn main() {
    match process_trade("BTCUSDT", 0.5, 42000.0) {
        Ok(result) => println!("Trade result: {}", result),
        Err(e) => println!("Trade failed: {}", e),
    }
}

fn process_trade(symbol: &str, qty: f64, price: f64) -> Result<String, String> {
    // Оператор ? автоматически возвращает Err если результат — ошибка
    let validated_symbol = validate_trading_pair(symbol)?;
    let order_id = place_limit_order(&validated_symbol, qty, price)?;
    let confirmation = confirm_order(order_id)?;

    Ok(format!("Trade confirmed: {}", confirmation))
}

fn validate_trading_pair(symbol: &str) -> Result<String, String> {
    if symbol.ends_with("USDT") || symbol.ends_with("BTC") {
        Ok(symbol.to_uppercase())
    } else {
        Err(format!("Unsupported trading pair: {}", symbol))
    }
}

fn place_limit_order(symbol: &str, qty: f64, price: f64) -> Result<u64, String> {
    if qty > 0.0 && price > 0.0 {
        Ok(1234567890)
    } else {
        Err(String::from("Invalid order parameters"))
    }
}

fn confirm_order(order_id: u64) -> Result<String, String> {
    Ok(format!("ORD-{}", order_id))
}
```

## Практический пример: Система исполнения ордеров

```rust
fn main() {
    let orders = vec![
        ("BTCUSDT", 0.1, 42000.0),
        ("ETHUSDT", 2.0, 2500.0),
        ("INVALID", 1.0, 100.0),  // Неверный символ
        ("SOLUSDT", -1.0, 50.0),  // Отрицательное количество
    ];

    for (symbol, qty, price) in orders {
        match execute_order(symbol, qty, price) {
            Ok(order) => println!("{}", order),
            Err(e) => println!("REJECTED: {} - {}", symbol, e),
        }
    }
}

#[derive(Debug)]
struct OrderConfirmation {
    order_id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    total: f64,
    status: String,
}

impl std::fmt::Display for OrderConfirmation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FILLED: {} | {} @ ${:.2} | Total: ${:.2} | ID: {}",
            self.symbol, self.quantity, self.price, self.total, self.order_id
        )
    }
}

fn execute_order(symbol: &str, qty: f64, price: f64) -> Result<OrderConfirmation, String> {
    // Валидация символа
    if !symbol.ends_with("USDT") && !symbol.ends_with("BTC") {
        return Err(format!("Invalid trading pair: {}", symbol));
    }

    // Валидация количества
    if qty <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }

    // Валидация цены
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }

    // Проверка минимального объёма
    let total = qty * price;
    if total < 10.0 {
        return Err(format!("Order too small: ${:.2} (min: $10)", total));
    }

    // Успешное исполнение
    Ok(OrderConfirmation {
        order_id: generate_order_id(),
        symbol: symbol.to_string(),
        quantity: qty,
        price,
        total,
        status: String::from("FILLED"),
    })
}

fn generate_order_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
```

## Кастомные типы ошибок

```rust
use std::fmt;

#[derive(Debug)]
enum TradingError {
    InsufficientFunds { required: f64, available: f64 },
    InvalidQuantity(f64),
    SymbolNotFound(String),
    RiskLimitExceeded { current: f64, max: f64 },
    NetworkError(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::InsufficientFunds { required, available } => {
                write!(f, "Insufficient funds: need ${:.2}, have ${:.2}", required, available)
            }
            TradingError::InvalidQuantity(qty) => {
                write!(f, "Invalid quantity: {}", qty)
            }
            TradingError::SymbolNotFound(symbol) => {
                write!(f, "Symbol not found: {}", symbol)
            }
            TradingError::RiskLimitExceeded { current, max } => {
                write!(f, "Risk limit exceeded: {:.2}% > {:.2}%", current, max)
            }
            TradingError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
        }
    }
}

fn main() {
    let result = open_position("BTCUSDT", 1.0, 50000.0, 5000.0);

    match result {
        Ok(position_id) => println!("Position opened: {}", position_id),
        Err(e) => println!("Error: {}", e),
    }
}

fn open_position(
    symbol: &str,
    qty: f64,
    price: f64,
    balance: f64,
) -> Result<u64, TradingError> {
    // Проверка символа
    let valid_symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    if !valid_symbols.contains(&symbol) {
        return Err(TradingError::SymbolNotFound(symbol.to_string()));
    }

    // Проверка количества
    if qty <= 0.0 {
        return Err(TradingError::InvalidQuantity(qty));
    }

    // Проверка баланса
    let required = qty * price;
    if required > balance {
        return Err(TradingError::InsufficientFunds {
            required,
            available: balance,
        });
    }

    // Проверка риска
    let risk_percent = (required / balance) * 100.0;
    if risk_percent > 10.0 {
        return Err(TradingError::RiskLimitExceeded {
            current: risk_percent,
            max: 10.0,
        });
    }

    Ok(12345)
}
```

## Комбинирование Result и Option

```rust
fn main() {
    // Option -> Result
    let maybe_price: Option<f64> = Some(42000.0);
    let result: Result<f64, &str> = maybe_price.ok_or("Price not found");
    println!("{:?}", result);

    // Result -> Option
    let result: Result<f64, &str> = Ok(42000.0);
    let maybe: Option<f64> = result.ok();
    println!("{:?}", maybe);

    // Практический пример
    match find_and_execute("BTCUSDT") {
        Ok(price) => println!("Executed at: ${:.2}", price),
        Err(e) => println!("Failed: {}", e),
    }
}

fn find_and_execute(symbol: &str) -> Result<f64, String> {
    let price = get_best_price(symbol)
        .ok_or_else(|| format!("No price for {}", symbol))?;

    execute_at_price(symbol, price)
}

fn get_best_price(symbol: &str) -> Option<f64> {
    match symbol {
        "BTCUSDT" => Some(42000.0),
        "ETHUSDT" => Some(2500.0),
        _ => None,
    }
}

fn execute_at_price(symbol: &str, price: f64) -> Result<f64, String> {
    if price > 0.0 {
        Ok(price)
    } else {
        Err(String::from("Invalid price for execution"))
    }
}
```

## Упражнения

### Упражнение 1: Валидатор ордера
Создай функцию `validate_order`, которая проверяет все параметры ордера и возвращает `Result<ValidatedOrder, OrderError>`.

```rust
#[derive(Debug)]
struct ValidatedOrder {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidSymbol(String),
    InvalidSide(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance,
}

fn validate_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<ValidatedOrder, OrderError> {
    // Твоя реализация
    todo!()
}
```

### Упражнение 2: Парсер торгового сигнала
Напиши функцию, которая парсит строку торгового сигнала в структуру:

```rust
#[derive(Debug)]
struct TradeSignal {
    action: String,
    symbol: String,
    price: f64,
}

fn parse_signal(input: &str) -> Result<TradeSignal, String> {
    // Формат: "BUY BTCUSDT 42000.50"
    // Твоя реализация
    todo!()
}
```

### Упражнение 3: Цепочка проверок риск-менеджмента
Реализуй систему проверок перед открытием позиции:

```rust
fn pre_trade_checks(
    symbol: &str,
    quantity: f64,
    price: f64,
    balance: f64,
    max_risk: f64,
    max_position: f64,
) -> Result<(), String> {
    // Проверки:
    // 1. Символ валиден
    // 2. Достаточно средств
    // 3. Риск в пределах нормы
    // 4. Размер позиции не превышает максимум
    todo!()
}
```

## Что мы узнали

| Метод | Описание | Пример использования |
|-------|----------|---------------------|
| `Ok(v)` | Создать успешный результат | `Ok(order_id)` |
| `Err(e)` | Создать ошибку | `Err("Invalid")` |
| `is_ok()` | Проверить на успех | Условная логика |
| `is_err()` | Проверить на ошибку | Логирование ошибок |
| `unwrap()` | Извлечь или паника | Только в тестах! |
| `expect()` | Извлечь с сообщением | Отладка |
| `unwrap_or()` | Значение по умолчанию | Безопасное извлечение |
| `map()` | Преобразовать Ok | Конвертация валют |
| `map_err()` | Преобразовать Err | Добавление контекста |
| `and_then()` | Цепочка операций | Workflow ордера |
| `?` | Распространить ошибку | Чистый код |

## Домашнее задание

1. **Система исполнения ордеров**: Создай полную систему с типами ордеров (market, limit, stop), валидацией и исполнением. Каждый этап должен возвращать `Result`.

2. **Парсер биржевого API**: Напиши функции для парсинга JSON-ответов от биржи с обработкой всех возможных ошибок (неверный формат, отсутствующие поля, невалидные значения).

3. **Калькулятор риска**: Реализуй функцию расчёта размера позиции с проверками:
   - Валидность входных данных
   - Достаточность баланса
   - Соблюдение лимитов риска
   - Минимальный размер ордера

4. **Обработчик ошибок с retry**: Создай обёртку, которая повторяет операцию N раз при определённых типах ошибок (сетевые), но сразу возвращает ошибку при других (валидация).

## Навигация

[← Предыдущий день](../093-result-introduction/ru.md) | [Следующий день →](../095-question-mark-operator/ru.md)
