# День 101: Множественные типы ошибок в функции

## Аналогия из трейдинга

Представь, что ты проверяешь торговый ордер перед отправкой на биржу. Ордер может быть отклонён по **разным причинам**:
- Недостаточный баланс (ошибка баланса)
- Неверный тикер (ошибка валидации)
- Биржа недоступна (сетевая ошибка)
- Неверный формат цены (ошибка парсинга)

Каждая причина — это **разный тип ошибки**, но функция должна уметь обрабатывать их все. В Rust для этого есть несколько подходов.

## Проблема: разные типы ошибок

```rust
use std::fs::File;
use std::io::{self, Read};

fn load_portfolio(path: &str) -> Result<f64, ???> {  // Какой тип ошибки?
    let mut file = File::open(path)?;  // io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;  // io::Error
    let balance: f64 = contents.trim().parse()?;  // ParseFloatError
    Ok(balance)
}
```

Здесь `File::open` возвращает `io::Error`, а `parse()` — `ParseFloatError`. Как объединить их в одном `Result`?

## Решение 1: Box<dyn Error>

Универсальный контейнер для любой ошибки:

```rust
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn load_portfolio(path: &str) -> Result<f64, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let balance: f64 = contents.trim().parse()?;
    Ok(balance)
}

fn main() {
    match load_portfolio("portfolio.txt") {
        Ok(balance) => println!("Portfolio balance: ${:.2}", balance),
        Err(e) => println!("Failed to load portfolio: {}", e),
    }
}
```

**Плюсы:** Просто, работает с любыми ошибками.
**Минусы:** Теряем информацию о конкретном типе ошибки.

## Решение 2: Собственный enum ошибок

Создаём свой тип, объединяющий все возможные ошибки:

```rust
use std::fmt;
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradeError {
    IoError(io::Error),
    ParseError(ParseFloatError),
    ValidationError(String),
    InsufficientBalance { required: f64, available: f64 },
    InvalidTicker(String),
}

impl fmt::Display for TradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeError::IoError(e) => write!(f, "IO error: {}", e),
            TradeError::ParseError(e) => write!(f, "Parse error: {}", e),
            TradeError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TradeError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need ${:.2}, have ${:.2}", required, available)
            }
            TradeError::InvalidTicker(ticker) => write!(f, "Invalid ticker: {}", ticker),
        }
    }
}

impl std::error::Error for TradeError {}
```

## Реализация From для автоматической конверсии

Чтобы оператор `?` работал автоматически:

```rust
use std::io;
use std::num::ParseFloatError;

impl From<io::Error> for TradeError {
    fn from(error: io::Error) -> Self {
        TradeError::IoError(error)
    }
}

impl From<ParseFloatError> for TradeError {
    fn from(error: ParseFloatError) -> Self {
        TradeError::ParseError(error)
    }
}
```

Теперь можно использовать `?` без явного преобразования:

```rust
use std::fs::File;
use std::io::Read;

fn load_portfolio(path: &str) -> Result<f64, TradeError> {
    let mut file = File::open(path)?;  // io::Error -> TradeError автоматически
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let balance: f64 = contents.trim().parse()?;  // ParseFloatError -> TradeError
    Ok(balance)
}
```

## Практический пример: валидация торгового ордера

```rust
use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
enum OrderError {
    InvalidTicker(String),
    InvalidQuantity(String),
    InvalidPrice(String),
    InsufficientBalance { required: f64, available: f64 },
    MarketClosed(String),
    ParseError(String),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::InvalidTicker(t) => write!(f, "Invalid ticker: {}", t),
            OrderError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {}", msg),
            OrderError::InvalidPrice(msg) => write!(f, "Invalid price: {}", msg),
            OrderError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need ${:.2}, have ${:.2}", required, available)
            }
            OrderError::MarketClosed(market) => write!(f, "Market {} is closed", market),
            OrderError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for OrderError {}

struct Order {
    ticker: String,
    quantity: f64,
    price: f64,
    side: String,
}

fn validate_order(
    ticker: &str,
    quantity: f64,
    price: f64,
    balance: f64,
    valid_tickers: &HashMap<String, bool>,
    market_open: bool,
) -> Result<Order, OrderError> {
    // Проверка тикера
    if !valid_tickers.contains_key(ticker) {
        return Err(OrderError::InvalidTicker(ticker.to_string()));
    }

    // Проверка количества
    if quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(
            "Quantity must be positive".to_string()
        ));
    }

    // Проверка цены
    if price <= 0.0 {
        return Err(OrderError::InvalidPrice(
            "Price must be positive".to_string()
        ));
    }

    // Проверка баланса
    let required = quantity * price;
    if required > balance {
        return Err(OrderError::InsufficientBalance { required, available: balance });
    }

    // Проверка рынка
    if !market_open {
        return Err(OrderError::MarketClosed("NASDAQ".to_string()));
    }

    Ok(Order {
        ticker: ticker.to_string(),
        quantity,
        price,
        side: "BUY".to_string(),
    })
}

fn main() {
    let mut valid_tickers = HashMap::new();
    valid_tickers.insert("AAPL".to_string(), true);
    valid_tickers.insert("GOOGL".to_string(), true);
    valid_tickers.insert("BTC".to_string(), true);

    let balance = 10000.0;
    let market_open = true;

    // Тест 1: Валидный ордер
    match validate_order("AAPL", 10.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(order) => println!("Order created: {} {} @ ${:.2}",
            order.quantity, order.ticker, order.price),
        Err(e) => println!("Order failed: {}", e),
    }

    // Тест 2: Неверный тикер
    match validate_order("INVALID", 10.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Order failed: {}", e),
    }

    // Тест 3: Недостаточный баланс
    match validate_order("AAPL", 1000.0, 150.0, balance, &valid_tickers, market_open) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Order failed: {}", e),
    }
}
```

## Комбинирование ошибок из разных источников

```rust
use std::fmt;
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum PortfolioError {
    Io(io::Error),
    Parse(ParseFloatError),
    Validation(String),
    Calculation(String),
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortfolioError::Io(e) => write!(f, "IO error: {}", e),
            PortfolioError::Parse(e) => write!(f, "Parse error: {}", e),
            PortfolioError::Validation(msg) => write!(f, "Validation: {}", msg),
            PortfolioError::Calculation(msg) => write!(f, "Calculation: {}", msg),
        }
    }
}

impl std::error::Error for PortfolioError {}

impl From<io::Error> for PortfolioError {
    fn from(e: io::Error) -> Self {
        PortfolioError::Io(e)
    }
}

impl From<ParseFloatError> for PortfolioError {
    fn from(e: ParseFloatError) -> Self {
        PortfolioError::Parse(e)
    }
}

fn calculate_portfolio_risk(positions: &[(String, f64, f64)]) -> Result<f64, PortfolioError> {
    if positions.is_empty() {
        return Err(PortfolioError::Validation(
            "Portfolio cannot be empty".to_string()
        ));
    }

    let total_value: f64 = positions
        .iter()
        .map(|(_, qty, price)| qty * price)
        .sum();

    if total_value <= 0.0 {
        return Err(PortfolioError::Calculation(
            "Total portfolio value must be positive".to_string()
        ));
    }

    // Простой расчёт риска: сумма квадратов весов
    let risk: f64 = positions
        .iter()
        .map(|(_, qty, price)| {
            let weight = (qty * price) / total_value;
            weight * weight
        })
        .sum();

    Ok(risk.sqrt())
}

fn main() {
    let positions = vec![
        ("AAPL".to_string(), 10.0, 150.0),
        ("GOOGL".to_string(), 5.0, 2800.0),
        ("BTC".to_string(), 0.5, 42000.0),
    ];

    match calculate_portfolio_risk(&positions) {
        Ok(risk) => println!("Portfolio risk score: {:.4}", risk),
        Err(e) => println!("Error: {}", e),
    }

    // Тест с пустым портфелем
    match calculate_portfolio_risk(&[]) {
        Ok(risk) => println!("Risk: {:.4}", risk),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Обработка ошибок в цепочке операций

```rust
use std::fmt;

#[derive(Debug)]
enum TradingError {
    DataFetch(String),
    Analysis(String),
    Execution(String),
    RiskLimit(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::DataFetch(msg) => write!(f, "Data fetch failed: {}", msg),
            TradingError::Analysis(msg) => write!(f, "Analysis failed: {}", msg),
            TradingError::Execution(msg) => write!(f, "Execution failed: {}", msg),
            TradingError::RiskLimit(msg) => write!(f, "Risk limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for TradingError {}

struct MarketData {
    price: f64,
    volume: f64,
}

struct Signal {
    action: String,
    confidence: f64,
}

struct TradeResult {
    executed_price: f64,
    quantity: f64,
}

fn fetch_market_data(ticker: &str) -> Result<MarketData, TradingError> {
    // Симуляция получения данных
    if ticker == "INVALID" {
        return Err(TradingError::DataFetch(
            format!("Unknown ticker: {}", ticker)
        ));
    }
    Ok(MarketData { price: 42000.0, volume: 1000000.0 })
}

fn analyze_signal(data: &MarketData) -> Result<Signal, TradingError> {
    if data.volume < 1000.0 {
        return Err(TradingError::Analysis(
            "Insufficient volume for analysis".to_string()
        ));
    }
    Ok(Signal {
        action: "BUY".to_string(),
        confidence: 0.75,
    })
}

fn check_risk_limits(signal: &Signal, max_risk: f64) -> Result<(), TradingError> {
    if signal.confidence < max_risk {
        return Err(TradingError::RiskLimit(
            format!("Confidence {:.2} below threshold {:.2}",
                signal.confidence, max_risk)
        ));
    }
    Ok(())
}

fn execute_trade(signal: &Signal, quantity: f64) -> Result<TradeResult, TradingError> {
    if quantity <= 0.0 {
        return Err(TradingError::Execution(
            "Invalid quantity".to_string()
        ));
    }
    Ok(TradeResult {
        executed_price: 42000.0,
        quantity,
    })
}

fn trading_pipeline(ticker: &str, quantity: f64, max_risk: f64) -> Result<TradeResult, TradingError> {
    let data = fetch_market_data(ticker)?;
    let signal = analyze_signal(&data)?;
    check_risk_limits(&signal, max_risk)?;
    let result = execute_trade(&signal, quantity)?;
    Ok(result)
}

fn main() {
    // Успешная сделка
    match trading_pipeline("BTC", 0.5, 0.7) {
        Ok(result) => println!(
            "Trade executed: {} @ ${:.2}",
            result.quantity, result.executed_price
        ),
        Err(e) => println!("Trading failed: {}", e),
    }

    // Ошибка: неверный тикер
    match trading_pipeline("INVALID", 0.5, 0.7) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }

    // Ошибка: превышен лимит риска
    match trading_pipeline("BTC", 0.5, 0.9) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Паттерн: восстановление после ошибки

```rust
fn get_price_with_fallback(primary: &str, fallback: &str) -> Result<f64, String> {
    // Пробуем основной источник
    match fetch_price(primary) {
        Ok(price) => return Ok(price),
        Err(e) => println!("Primary source failed: {}, trying fallback...", e),
    }

    // Пробуем резервный источник
    fetch_price(fallback)
}

fn fetch_price(source: &str) -> Result<f64, String> {
    match source {
        "binance" => Ok(42000.0),
        "coinbase" => Ok(42100.0),
        _ => Err(format!("Unknown source: {}", source)),
    }
}

fn main() {
    match get_price_with_fallback("unknown", "binance") {
        Ok(price) => println!("Price: ${:.2}", price),
        Err(e) => println!("All sources failed: {}", e),
    }
}
```

## Упражнения

### Упражнение 1: Валидатор транзакций

Создайте enum `TransactionError` с вариантами:
- `InvalidAmount` — некорректная сумма
- `InvalidAddress` — неверный адрес
- `NetworkError` — сетевая ошибка
- `InsufficientFunds` — недостаточно средств

Реализуйте функцию `validate_transaction()`.

### Упражнение 2: Парсер торговых данных

Напишите функцию, которая парсит строку формата `"TICKER:PRICE:VOLUME"` и возвращает структуру или один из типов ошибок.

### Упражнение 3: Многоэтапная валидация

Создайте функцию `process_trade_request()`, которая:
1. Парсит входные данные
2. Валидирует тикер
3. Проверяет лимиты
4. Рассчитывает комиссию

Каждый этап может вернуть свой тип ошибки.

## Домашнее задание

1. **Кастомный тип ошибки для биржи**

   Создайте `ExchangeError` с вариантами для разных ошибок биржевого API: ошибка аутентификации, лимит запросов, недоступность сервиса, ошибка ордера.

2. **Цепочка обработки ордера**

   Реализуйте полный пайплайн обработки ордера с 5 этапами, каждый из которых может вернуть свой тип ошибки.

3. **Агрегатор данных**

   Напишите функцию, которая запрашивает цену из трёх источников и возвращает результат или агрегированную ошибку.

4. **Retry с разными ошибками**

   Реализуйте функцию `retry_operation()`, которая повторяет операцию только для определённых типов ошибок (например, сетевых), но не для других (например, ошибок валидации).

## Что мы узнали

| Подход | Когда использовать |
|--------|-------------------|
| `Box<dyn Error>` | Быстрое прототипирование, когда тип ошибки неважен |
| `enum` с ошибками | Когда нужно обрабатывать ошибки по-разному |
| `From` trait | Для автоматической конверсии с оператором `?` |
| Вложенные ошибки | Когда важно сохранить исходную ошибку |

## Навигация

[← Предыдущий день](../100-result-type-trade-execution/ru.md) | [Следующий день →](../102-question-mark-operator/ru.md)
