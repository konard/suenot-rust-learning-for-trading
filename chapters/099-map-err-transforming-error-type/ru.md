# День 99: map_err — преобразуем тип ошибки

## Аналогия из трейдинга

Представь, что у тебя есть несколько источников рыночных данных: биржа, брокер и новостной сервис. Каждый из них сообщает об ошибках по-своему:
- Биржа: "CONNECTION_TIMEOUT_ERR_5001"
- Брокер: "Сессия истекла, код 401"
- Новостной сервис: "Rate limit exceeded"

Когда ты строишь единую торговую систему, тебе нужно **преобразовать** все эти разнородные ошибки в единый формат, понятный твоей системе. Метод `map_err` в Rust делает именно это — он трансформирует один тип ошибки в другой.

## Сигнатура map_err

```rust
impl<T, E> Result<T, E> {
    fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F
}
```

**Важно:** `map_err` преобразует только ошибку (`Err`), оставляя успешное значение (`Ok`) без изменений.

## Базовое использование

```rust
fn main() {
    // Парсинг цены может вернуть ParseFloatError
    let price_str = "42500.50";
    let result: Result<f64, String> = price_str
        .parse::<f64>()
        .map_err(|e| format!("Ошибка парсинга цены: {}", e));

    println!("{:?}", result); // Ok(42500.5)

    // Неверный формат
    let bad_price = "not_a_price";
    let result: Result<f64, String> = bad_price
        .parse::<f64>()
        .map_err(|e| format!("Ошибка парсинга цены: {}", e));

    println!("{:?}", result); // Err("Ошибка парсинга цены: invalid float literal")
}
```

## Преобразование ошибок API биржи

```rust
use std::fmt;

// Ошибки от разных источников
#[derive(Debug)]
enum ExchangeError {
    ConnectionFailed(String),
    RateLimited(u32),
    InvalidSymbol(String),
}

#[derive(Debug)]
enum BrokerError {
    SessionExpired,
    InsufficientFunds(f64),
    OrderRejected(String),
}

// Наш единый тип ошибки для торговой системы
#[derive(Debug)]
enum TradingError {
    DataSource(String),
    Execution(String),
    Validation(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::DataSource(msg) => write!(f, "Ошибка источника данных: {}", msg),
            TradingError::Execution(msg) => write!(f, "Ошибка исполнения: {}", msg),
            TradingError::Validation(msg) => write!(f, "Ошибка валидации: {}", msg),
        }
    }
}

// Преобразование ошибок биржи
fn exchange_error_to_trading(e: ExchangeError) -> TradingError {
    match e {
        ExchangeError::ConnectionFailed(msg) =>
            TradingError::DataSource(format!("Биржа недоступна: {}", msg)),
        ExchangeError::RateLimited(seconds) =>
            TradingError::DataSource(format!("Превышен лимит запросов, подождите {} сек", seconds)),
        ExchangeError::InvalidSymbol(sym) =>
            TradingError::Validation(format!("Неизвестный тикер: {}", sym)),
    }
}

// Преобразование ошибок брокера
fn broker_error_to_trading(e: BrokerError) -> TradingError {
    match e {
        BrokerError::SessionExpired =>
            TradingError::DataSource("Сессия истекла, требуется повторная авторизация".to_string()),
        BrokerError::InsufficientFunds(required) =>
            TradingError::Execution(format!("Недостаточно средств, требуется: ${:.2}", required)),
        BrokerError::OrderRejected(reason) =>
            TradingError::Execution(format!("Ордер отклонён: {}", reason)),
    }
}

fn get_price_from_exchange(symbol: &str) -> Result<f64, ExchangeError> {
    if symbol == "INVALID" {
        Err(ExchangeError::InvalidSymbol(symbol.to_string()))
    } else {
        Ok(42500.0)
    }
}

fn place_order_at_broker(symbol: &str, qty: f64, balance: f64) -> Result<String, BrokerError> {
    let required = 42500.0 * qty;
    if balance < required {
        Err(BrokerError::InsufficientFunds(required))
    } else {
        Ok(format!("ORDER-{}-{}", symbol, qty))
    }
}

fn main() {
    // Используем map_err для преобразования
    let price_result: Result<f64, TradingError> = get_price_from_exchange("BTCUSD")
        .map_err(exchange_error_to_trading);
    println!("Цена: {:?}", price_result);

    let invalid_result: Result<f64, TradingError> = get_price_from_exchange("INVALID")
        .map_err(exchange_error_to_trading);
    println!("Ошибка: {:?}", invalid_result);

    let order_result: Result<String, TradingError> = place_order_at_broker("BTCUSD", 1.0, 1000.0)
        .map_err(broker_error_to_trading);
    println!("Ордер: {:?}", order_result);
}
```

## Цепочки с map_err

```rust
fn main() {
    let result = parse_and_validate_order("BTCUSD", "0.5", "42000.50");
    println!("{:?}", result);

    let bad_result = parse_and_validate_order("BTCUSD", "abc", "42000.50");
    println!("{:?}", bad_result);
}

#[derive(Debug)]
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidQuantity(String),
    InvalidPrice(String),
    InvalidSymbol(String),
}

fn parse_and_validate_order(
    symbol: &str,
    qty_str: &str,
    price_str: &str,
) -> Result<Order, OrderError> {
    // Каждый парсинг преобразует свою ошибку в нужный вариант
    let quantity: f64 = qty_str
        .parse()
        .map_err(|e| OrderError::InvalidQuantity(format!("{}: {}", qty_str, e)))?;

    let price: f64 = price_str
        .parse()
        .map_err(|e| OrderError::InvalidPrice(format!("{}: {}", price_str, e)))?;

    if symbol.is_empty() {
        return Err(OrderError::InvalidSymbol("Пустой тикер".to_string()));
    }

    Ok(Order {
        symbol: symbol.to_string(),
        quantity,
        price,
    })
}
```

## Преобразование в строку с контекстом

```rust
fn main() {
    let prices = vec!["42500.0", "invalid", "43000.0"];

    for (i, price_str) in prices.iter().enumerate() {
        let result = parse_price_with_context(price_str, i);
        match result {
            Ok(price) => println!("Цена {}: ${:.2}", i, price),
            Err(e) => println!("Ошибка: {}", e),
        }
    }
}

fn parse_price_with_context(s: &str, index: usize) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|e| format!("Не удалось распарсить цену #{} '{}': {}", index, s, e))
}
```

## map_err с замыканиями и захватом контекста

```rust
fn main() {
    let symbol = "ETHUSD";
    let source = "Binance";

    let result = fetch_price_with_context(symbol, source);
    println!("{:?}", result);
}

fn fetch_price(symbol: &str) -> Result<f64, std::io::Error> {
    // Имитация ошибки ввода-вывода
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Сервер недоступен"
    ))
}

fn fetch_price_with_context(symbol: &str, source: &str) -> Result<f64, String> {
    // Замыкание захватывает symbol и source для формирования информативной ошибки
    fetch_price(symbol)
        .map_err(|e| format!(
            "[{}] Ошибка получения цены {}: {} ({})",
            source, symbol, e, e.kind()
        ))
}
```

## Практический пример: загрузка портфеля

```rust
use std::collections::HashMap;

fn main() {
    let portfolio_data = r#"
        BTCUSD:0.5
        ETHUSD:2.0
        SOLUSD:10.0
    "#;

    match load_portfolio(portfolio_data) {
        Ok(portfolio) => {
            println!("Портфель загружен:");
            for (symbol, qty) in &portfolio {
                println!("  {}: {} единиц", symbol, qty);
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример с ошибкой
    let bad_data = "BTCUSD:not_a_number";
    match load_portfolio(bad_data) {
        Ok(_) => println!("Успех"),
        Err(e) => println!("Ошибка: {}", e),
    }
}

#[derive(Debug)]
enum PortfolioError {
    ParseError { line: usize, details: String },
    EmptyPortfolio,
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioError::ParseError { line, details } =>
                write!(f, "Ошибка парсинга на строке {}: {}", line, details),
            PortfolioError::EmptyPortfolio =>
                write!(f, "Портфель пуст"),
        }
    }
}

fn load_portfolio(data: &str) -> Result<HashMap<String, f64>, PortfolioError> {
    let mut portfolio = HashMap::new();

    for (line_num, line) in data.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (symbol, qty) = parse_portfolio_line(line, line_num + 1)?;
        portfolio.insert(symbol, qty);
    }

    if portfolio.is_empty() {
        return Err(PortfolioError::EmptyPortfolio);
    }

    Ok(portfolio)
}

fn parse_portfolio_line(line: &str, line_num: usize) -> Result<(String, f64), PortfolioError> {
    let parts: Vec<&str> = line.split(':').collect();

    if parts.len() != 2 {
        return Err(PortfolioError::ParseError {
            line: line_num,
            details: format!("Ожидался формат 'SYMBOL:QTY', получено '{}'", line),
        });
    }

    let symbol = parts[0].to_string();
    let qty = parts[1]
        .parse::<f64>()
        // Преобразуем ParseFloatError в PortfolioError с контекстом
        .map_err(|e| PortfolioError::ParseError {
            line: line_num,
            details: format!("Неверное количество '{}': {}", parts[1], e),
        })?;

    Ok((symbol, qty))
}
```

## map_err vs and_then для ошибок

```rust
fn main() {
    // map_err — только преобразует ошибку
    let result1: Result<i32, String> = "42".parse::<i32>()
        .map_err(|e| format!("Ошибка: {}", e));

    // and_then — позволяет вернуть новую ошибку на основе успешного значения
    let result2: Result<i32, String> = "42".parse::<i32>()
        .map_err(|e| format!("Ошибка: {}", e))
        .and_then(|n| {
            if n > 0 {
                Ok(n)
            } else {
                Err("Число должно быть положительным".to_string())
            }
        });

    println!("result1: {:?}", result1);
    println!("result2: {:?}", result2);
}
```

## Пример: риск-менеджмент с разными источниками ошибок

```rust
fn main() {
    match execute_trade("BTCUSD", 0.5, 50000.0) {
        Ok(order_id) => println!("Сделка выполнена: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Симуляция ошибки валидации
    match execute_trade("BTCUSD", -0.5, 50000.0) {
        Ok(order_id) => println!("Сделка выполнена: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }
}

#[derive(Debug)]
enum TradeError {
    Validation(String),
    RiskLimit(String),
    Execution(String),
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeError::Validation(msg) => write!(f, "Валидация: {}", msg),
            TradeError::RiskLimit(msg) => write!(f, "Риск-лимит: {}", msg),
            TradeError::Execution(msg) => write!(f, "Исполнение: {}", msg),
        }
    }
}

fn validate_quantity(qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        Err(format!("Количество должно быть положительным, получено: {}", qty))
    } else if qty > 100.0 {
        Err(format!("Превышен максимальный объём: {}", qty))
    } else {
        Ok(qty)
    }
}

fn check_risk_limits(symbol: &str, qty: f64, price: f64) -> Result<(), String> {
    let max_position_value = 100_000.0;
    let position_value = qty * price;

    if position_value > max_position_value {
        Err(format!(
            "Позиция ${:.2} превышает лимит ${:.2}",
            position_value, max_position_value
        ))
    } else {
        Ok(())
    }
}

fn send_order(symbol: &str, qty: f64, price: f64) -> Result<String, std::io::Error> {
    // Успешное исполнение
    Ok(format!("ORD-{}-{}-{}", symbol, qty, price))
}

fn execute_trade(symbol: &str, qty: f64, price: f64) -> Result<String, TradeError> {
    // Валидация — преобразуем String в TradeError::Validation
    let validated_qty = validate_quantity(qty)
        .map_err(TradeError::Validation)?;

    // Проверка рисков — преобразуем String в TradeError::RiskLimit
    check_risk_limits(symbol, validated_qty, price)
        .map_err(TradeError::RiskLimit)?;

    // Отправка ордера — преобразуем io::Error в TradeError::Execution
    send_order(symbol, validated_qty, price)
        .map_err(|e| TradeError::Execution(e.to_string()))
}
```

## Что мы узнали

| Аспект | Описание |
|--------|----------|
| `map_err` | Преобразует тип ошибки в `Result` |
| Сигнатура | `Result<T, E> -> Result<T, F>` |
| Когда использовать | При объединении разных источников ошибок |
| Замыкания | Можно захватывать контекст для информативных сообщений |
| Цепочки | Легко комбинируется с `?` оператором |

## Практические задания

1. **Парсер рыночных данных**: Напиши функцию, которая парсит строку формата `"BTCUSD,42500.00,0.5"` и преобразует все ошибки парсинга в единый тип `MarketDataError` с информацией о том, какое поле не удалось распарсить.

2. **Унификация ошибок**: Создай три функции, имитирующие вызовы к разным API (биржа, брокер, аналитика). Каждая возвращает свой тип ошибки. Напиши функцию-обёртку, которая с помощью `map_err` приводит все ошибки к единому типу.

3. **Загрузчик конфигурации**: Реализуй загрузку торговой конфигурации из файла. Используй `map_err` для добавления контекста (имя файла, номер строки) к ошибкам парсинга.

4. **Многошаговая валидация**: Создай функцию валидации ордера, которая проверяет символ, количество, цену и баланс. Каждая проверка может вернуть ошибку — преобразуй их все в `OrderValidationError` с конкретным описанием проблемы.

## Домашнее задание

1. Реализуй систему загрузки исторических данных из нескольких источников (файл, API, кэш). Каждый источник имеет свой тип ошибки. Используй `map_err` для унификации и добавления информации об источнике.

2. Создай калькулятор размера позиции с полной валидацией всех входных параметров. Все ошибки должны быть преобразованы в `PositionSizeError` с детальным описанием проблемы.

3. Напиши функцию, которая читает и парсит файл с историей сделок. Используй `map_err` для:
   - Преобразования ошибок чтения файла
   - Добавления номера строки к ошибкам парсинга
   - Создания понятных сообщений об ошибках

4. Реализуй цепочку обработки торгового сигнала: получение данных → валидация → проверка рисков → исполнение. Каждый шаг может вернуть ошибку разного типа — унифицируй их с помощью `map_err`.

## Навигация

[← Предыдущий день](../098-result-method-chaining/ru.md) | [Следующий день →](../100-ok-or-converting-option-to-result/ru.md)
