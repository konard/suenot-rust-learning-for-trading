# День 75: Result — операция может провалиться

## Аналогия из трейдинга

Представь, что ты отправляешь ордер на биржу. Что может пойти не так?

- **Недостаточно средств** на балансе
- **Рынок закрыт** в данный момент
- **Цена ушла** слишком далеко от запрошенной
- **Превышен лимит** позиций
- **Сетевая ошибка** при отправке

В реальном трейдинге каждая операция может **успешно выполниться** или **провалиться с ошибкой**. Тип `Result<T, E>` в Rust идеально моделирует эту ситуацию: операция возвращает либо успешный результат типа `T`, либо ошибку типа `E`.

## Теория

### Определение Result

```rust
enum Result<T, E> {
    Ok(T),    // Успех, содержит значение типа T
    Err(E),   // Ошибка, содержит значение типа E
}
```

В отличие от `Option`, который говорит "значение есть или нет", `Result` говорит "операция успешна или провалилась с конкретной причиной".

### Базовое использование

```rust
fn main() {
    // Попытка исполнить ордер
    match execute_order("BTC/USDT", 0.5, 42000.0, 1000.0) {
        Ok(order_id) => println!("Ордер исполнен! ID: {}", order_id),
        Err(error) => println!("Ошибка: {}", error),
    }
}

fn execute_order(
    pair: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    let required = quantity * price;

    if required > balance {
        return Err(format!(
            "Недостаточно средств: нужно {:.2}, доступно {:.2}",
            required, balance
        ));
    }

    // Успешное исполнение
    Ok(format!("ORD-{}-{}", pair.replace("/", ""), 12345))
}
```

## Создание Result

### Возврат Ok и Err

```rust
fn validate_order_price(price: f64, min: f64, max: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    if price < min {
        return Err(format!("Цена {:.2} ниже минимума {:.2}", price, min));
    }
    if price > max {
        return Err(format!("Цена {:.2} выше максимума {:.2}", price, max));
    }
    Ok(price)
}

fn main() {
    println!("{:?}", validate_order_price(42000.0, 40000.0, 45000.0)); // Ok(42000.0)
    println!("{:?}", validate_order_price(-100.0, 40000.0, 45000.0));  // Err(...)
    println!("{:?}", validate_order_price(50000.0, 40000.0, 45000.0)); // Err(...)
}
```

### Типизированные ошибки

```rust
#[derive(Debug)]
enum OrderError {
    InsufficientFunds { required: f64, available: f64 },
    InvalidPrice { price: f64, reason: String },
    MarketClosed,
    RateLimitExceeded,
    NetworkError(String),
}

fn place_market_order(
    symbol: &str,
    quantity: f64,
    balance: f64,
    market_open: bool,
) -> Result<u64, OrderError> {
    if !market_open {
        return Err(OrderError::MarketClosed);
    }

    // Предположим текущая цена 42000
    let price = 42000.0;
    let required = quantity * price;

    if required > balance {
        return Err(OrderError::InsufficientFunds {
            required,
            available: balance,
        });
    }

    // Возвращаем ID ордера
    Ok(123456789)
}

fn main() {
    match place_market_order("BTCUSDT", 1.0, 50000.0, true) {
        Ok(id) => println!("Ордер размещён: {}", id),
        Err(OrderError::InsufficientFunds { required, available }) => {
            println!("Не хватает средств: нужно {}, есть {}", required, available);
        }
        Err(OrderError::MarketClosed) => {
            println!("Рынок закрыт, попробуйте позже");
        }
        Err(e) => println!("Другая ошибка: {:?}", e),
    }
}
```

## Обработка Result

### Метод match

```rust
fn calculate_position_value(
    quantity: f64,
    price: f64,
) -> Result<f64, String> {
    if quantity < 0.0 {
        return Err(String::from("Количество не может быть отрицательным"));
    }
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    Ok(quantity * price)
}

fn main() {
    let result = calculate_position_value(0.5, 42000.0);

    match result {
        Ok(value) => println!("Стоимость позиции: ${:.2}", value),
        Err(msg) => println!("Ошибка расчёта: {}", msg),
    }
}
```

### Методы unwrap и expect

```rust
fn main() {
    // unwrap — паникует при ошибке (используй осторожно!)
    let value = calculate_position_value(0.5, 42000.0).unwrap();
    println!("Value: {}", value);

    // expect — паникует с пользовательским сообщением
    let value = calculate_position_value(0.5, 42000.0)
        .expect("Не удалось рассчитать стоимость позиции");
    println!("Value: {}", value);
}

fn calculate_position_value(qty: f64, price: f64) -> Result<f64, String> {
    if qty <= 0.0 || price <= 0.0 {
        return Err(String::from("Invalid input"));
    }
    Ok(qty * price)
}
```

### Методы unwrap_or и unwrap_or_else

```rust
fn get_current_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTCUSDT" => Ok(42000.0),
        "ETHUSDT" => Ok(2500.0),
        _ => Err(format!("Неизвестный символ: {}", symbol)),
    }
}

fn main() {
    // unwrap_or — значение по умолчанию
    let btc_price = get_current_price("BTCUSDT").unwrap_or(0.0);
    let unknown_price = get_current_price("UNKNOWN").unwrap_or(0.0);

    println!("BTC: {}, Unknown: {}", btc_price, unknown_price);

    // unwrap_or_else — ленивое вычисление значения по умолчанию
    let price = get_current_price("UNKNOWN").unwrap_or_else(|err| {
        println!("Предупреждение: {}", err);
        0.0 // Возвращаем значение по умолчанию
    });
}
```

### Метод map

```rust
fn parse_price(input: &str) -> Result<f64, String> {
    input.parse::<f64>()
        .map_err(|_| format!("Невозможно распарсить '{}' как цену", input))
}

fn main() {
    // map — преобразование успешного значения
    let doubled = parse_price("42000.0")
        .map(|price| price * 2.0);

    println!("{:?}", doubled); // Ok(84000.0)

    // Цепочка map
    let formatted = parse_price("42000.0")
        .map(|price| price * 1.1)  // Добавляем 10%
        .map(|price| format!("${:.2}", price));

    println!("{:?}", formatted); // Ok("$46200.00")
}
```

### Метод and_then (flatMap)

```rust
fn get_balance(account: &str) -> Result<f64, String> {
    match account {
        "main" => Ok(10000.0),
        "trading" => Ok(5000.0),
        _ => Err(format!("Аккаунт '{}' не найден", account)),
    }
}

fn calculate_max_position(balance: f64, risk_percent: f64) -> Result<f64, String> {
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Процент риска должен быть от 0 до 100"));
    }
    Ok(balance * (risk_percent / 100.0))
}

fn main() {
    // and_then — цепочка операций, каждая из которых может провалиться
    let max_position = get_balance("trading")
        .and_then(|balance| calculate_max_position(balance, 5.0));

    println!("{:?}", max_position); // Ok(250.0)

    // Если первая операция провалится
    let max_position = get_balance("unknown")
        .and_then(|balance| calculate_max_position(balance, 5.0));

    println!("{:?}", max_position); // Err("Аккаунт 'unknown' не найден")
}
```

## Оператор ? — распространение ошибок

```rust
#[derive(Debug)]
struct TradeResult {
    order_id: u64,
    executed_price: f64,
    quantity: f64,
    fees: f64,
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    if symbol.is_empty() {
        return Err(String::from("Символ не может быть пустым"));
    }
    if !symbol.chars().all(|c| c.is_alphanumeric()) {
        return Err(String::from("Символ содержит недопустимые символы"));
    }
    Ok(())
}

fn validate_quantity(quantity: f64) -> Result<(), String> {
    if quantity <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    if quantity > 1000.0 {
        return Err(String::from("Превышен максимальный размер ордера"));
    }
    Ok(())
}

fn get_market_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTCUSDT" => Ok(42000.0),
        "ETHUSDT" => Ok(2500.0),
        _ => Err(format!("Нет данных о цене для {}", symbol)),
    }
}

fn execute_trade(
    symbol: &str,
    quantity: f64,
    balance: f64,
) -> Result<TradeResult, String> {
    // Оператор ? автоматически возвращает Err, если операция провалилась
    validate_symbol(symbol)?;
    validate_quantity(quantity)?;

    let price = get_market_price(symbol)?;
    let total_cost = price * quantity;

    if total_cost > balance {
        return Err(format!(
            "Недостаточно средств: нужно {:.2}, доступно {:.2}",
            total_cost, balance
        ));
    }

    let fees = total_cost * 0.001; // 0.1% комиссия

    Ok(TradeResult {
        order_id: 123456,
        executed_price: price,
        quantity,
        fees,
    })
}

fn main() {
    match execute_trade("BTCUSDT", 0.5, 50000.0) {
        Ok(result) => {
            println!("Сделка исполнена!");
            println!("Order ID: {}", result.order_id);
            println!("Цена: ${:.2}", result.executed_price);
            println!("Количество: {}", result.quantity);
            println!("Комиссия: ${:.2}", result.fees);
        }
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Комбинирование Result с Option

```rust
fn find_best_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }
    prices.iter().cloned().reduce(f64::min)
}

fn calculate_profit(
    entry: f64,
    exit: f64,
    quantity: f64,
) -> Result<f64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    Ok((exit - entry) * quantity)
}

fn analyze_trade_opportunity(
    prices: &[f64],
    current_price: f64,
    quantity: f64,
) -> Result<f64, String> {
    // ok_or преобразует Option в Result
    let best_entry = find_best_price(prices)
        .ok_or_else(|| String::from("Нет исторических цен для анализа"))?;

    // Рассчитываем потенциальную прибыль
    calculate_profit(best_entry, current_price, quantity)
}

fn main() {
    let historical_prices = vec![41000.0, 41500.0, 40800.0, 41200.0];
    let current = 42000.0;

    match analyze_trade_opportunity(&historical_prices, current, 0.5) {
        Ok(profit) => println!("Потенциальная прибыль: ${:.2}", profit),
        Err(e) => println!("Ошибка анализа: {}", e),
    }

    // С пустым массивом
    match analyze_trade_opportunity(&[], current, 0.5) {
        Ok(profit) => println!("Прибыль: ${:.2}", profit),
        Err(e) => println!("Ошибка: {}", e), // "Нет исторических цен..."
    }
}
```

## Практический пример: Торговый валидатор

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum ValidationError {
    EmptySymbol,
    InvalidSide(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance { required: f64, available: f64 },
    PositionLimitExceeded { current: f64, max: f64 },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptySymbol => write!(f, "Символ не указан"),
            ValidationError::InvalidSide(s) => write!(f, "Неверная сторона: {}", s),
            ValidationError::InvalidQuantity(q) => write!(f, "Неверное количество: {}", q),
            ValidationError::InvalidPrice(p) => write!(f, "Неверная цена: {}", p),
            ValidationError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно {:.2}, есть {:.2}", required, available)
            }
            ValidationError::PositionLimitExceeded { current, max } => {
                write!(f, "Превышен лимит позиции: {} > {}", current, max)
            }
        }
    }
}

fn validate_order(
    order: &Order,
    balance: f64,
    current_position: f64,
    max_position: f64,
) -> Result<(), ValidationError> {
    // Проверка символа
    if order.symbol.is_empty() {
        return Err(ValidationError::EmptySymbol);
    }

    // Проверка стороны
    if order.side != "BUY" && order.side != "SELL" {
        return Err(ValidationError::InvalidSide(order.side.clone()));
    }

    // Проверка количества
    if order.quantity <= 0.0 {
        return Err(ValidationError::InvalidQuantity(order.quantity));
    }

    // Проверка цены
    if order.price <= 0.0 {
        return Err(ValidationError::InvalidPrice(order.price));
    }

    // Проверка баланса (только для покупки)
    if order.side == "BUY" {
        let required = order.quantity * order.price;
        if required > balance {
            return Err(ValidationError::InsufficientBalance {
                required,
                available: balance,
            });
        }
    }

    // Проверка лимита позиции
    let new_position = if order.side == "BUY" {
        current_position + order.quantity
    } else {
        current_position - order.quantity
    };

    if new_position.abs() > max_position {
        return Err(ValidationError::PositionLimitExceeded {
            current: new_position.abs(),
            max: max_position,
        });
    }

    Ok(())
}

fn main() {
    let order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 0.5,
        price: 42000.0,
    };

    let balance = 50000.0;
    let current_position = 0.3;
    let max_position = 1.0;

    match validate_order(&order, balance, current_position, max_position) {
        Ok(()) => {
            println!("Ордер прошёл валидацию");
            println!("Отправляем на биржу: {:?}", order);
        }
        Err(e) => {
            println!("Валидация не пройдена: {}", e);
        }
    }

    // Тест с ошибкой
    let bad_order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 2.0, // Превысит лимит позиции
        price: 42000.0,
    };

    match validate_order(&bad_order, balance, current_position, max_position) {
        Ok(()) => println!("Ордер OK"),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Result<T, E>` | Операция возвращает успех `T` или ошибку `E` |
| `Ok(value)` | Успешный результат |
| `Err(error)` | Ошибка с описанием причины |
| `?` оператор | Автоматическое распространение ошибок |
| `unwrap()` | Извлечь значение или паниковать |
| `unwrap_or(default)` | Извлечь значение или вернуть умолчание |
| `map()` | Преобразовать успешное значение |
| `and_then()` | Цепочка fallible операций |
| `ok_or()` | Преобразование Option в Result |

## Практические задания

1. **Валидация цены**: Напиши функцию `validate_price(price: f64, min: f64, max: f64) -> Result<f64, PriceError>`, где `PriceError` — enum с вариантами `Negative`, `BelowMin`, `AboveMax`.

2. **Парсер ордера**: Создай функцию `parse_order(input: &str) -> Result<Order, ParseError>`, которая парсит строку формата "BUY BTCUSDT 0.5 42000.0" в структуру ордера.

3. **Цепочка проверок**: Реализуй функцию `process_trade(...)`, которая последовательно проверяет: баланс → лимиты → рыночные условия, используя оператор `?`.

4. **Агрегация ошибок**: Напиши функцию, которая валидирует несколько полей и собирает все ошибки в `Vec<ValidationError>`.

## Домашнее задание

1. Создай enum `TradingError` с вариантами для всех возможных ошибок трейдинговой системы (сетевые, валидация, исполнение, и т.д.) и реализуй для него `Display`.

2. Напиши функцию `execute_strategy(...)`, которая:
   - Получает рыночные данные (может провалиться)
   - Анализирует сигнал (может не найти сигнал)
   - Валидирует ордер (может быть невалидным)
   - Отправляет на исполнение (может отклониться)

   Используй `Result` и `?` для элегантной обработки всех случаев.

3. Реализуй retry-логику: функция `execute_with_retry(order, max_attempts) -> Result<TradeResult, TradingError>`, которая пытается исполнить ордер несколько раз при recoverable ошибках.

## Навигация

[← Предыдущий день](../074-option-value-may-be-absent/ru.md) | [Следующий день →](../076-question-mark-operator/ru.md)
