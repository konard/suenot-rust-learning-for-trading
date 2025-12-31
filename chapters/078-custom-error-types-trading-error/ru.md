# День 78: Собственные типы ошибок: TradingError

## Аналогия из трейдинга

Представь, что ты работаешь с разными биржами. Каждая биржа сообщает об ошибках по-своему: одна говорит "Недостаточно средств", другая "Balance too low", третья возвращает код ошибки 403. Как трейдеру разобраться во всём этом хаосе?

Решение — создать **единый стандарт ошибок** для твоей торговой системы. Вместо того чтобы обрабатывать десятки разных сообщений, ты определяешь свой тип `TradingError`, который унифицирует все возможные проблемы.

## Зачем нужны собственные типы ошибок?

```rust
// Плохо: возвращаем String — непонятно, какие ошибки возможны
fn execute_order(order: &Order) -> Result<Trade, String> {
    Err(String::from("Something went wrong"))
}

// Хорошо: чёткий перечень всех возможных ошибок
fn execute_order(order: &Order) -> Result<Trade, TradingError> {
    Err(TradingError::InsufficientBalance {
        required: 1000.0,
        available: 500.0
    })
}
```

Собственные типы ошибок дают:
- **Полный список** возможных ошибок (видно в коде)
- **Типизированную информацию** об ошибке (не просто текст)
- **Возможность match** — разная реакция на разные ошибки
- **Совместимость с `?`** — легко пробрасывать ошибки вверх

## Создаём TradingError

```rust
#[derive(Debug)]
enum TradingError {
    // Ошибки баланса
    InsufficientBalance { required: f64, available: f64 },

    // Ошибки ордера
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    InvalidPrice { price: f64, reason: String },
    OrderNotFound { order_id: u64 },

    // Ошибки рынка
    MarketClosed,
    SymbolNotFound { symbol: String },

    // Ошибки соединения
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },

    // Обёртка для внешних ошибок
    ApiError { code: i32, message: String },
}
```

## Реализуем Display для красивого вывода

```rust
use std::fmt;

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно ${:.2}, доступно ${:.2}",
                       required, available)
            }
            TradingError::InvalidOrderSize { size, min, max } => {
                write!(f, "Неверный размер ордера {}: допустимо от {} до {}",
                       size, min, max)
            }
            TradingError::InvalidPrice { price, reason } => {
                write!(f, "Неверная цена {}: {}", price, reason)
            }
            TradingError::OrderNotFound { order_id } => {
                write!(f, "Ордер #{} не найден", order_id)
            }
            TradingError::MarketClosed => {
                write!(f, "Рынок закрыт")
            }
            TradingError::SymbolNotFound { symbol } => {
                write!(f, "Инструмент '{}' не найден", symbol)
            }
            TradingError::ConnectionLost => {
                write!(f, "Соединение потеряно")
            }
            TradingError::Timeout { operation, seconds } => {
                write!(f, "Таймаут операции '{}' после {} сек", operation, seconds)
            }
            TradingError::ApiError { code, message } => {
                write!(f, "Ошибка API [{}]: {}", code, message)
            }
        }
    }
}
```

## Реализуем std::error::Error

```rust
use std::error::Error;

impl Error for TradingError {}

fn main() {
    let err = TradingError::InsufficientBalance {
        required: 10000.0,
        available: 5000.0
    };

    // Display — для пользователя
    println!("Ошибка: {}", err);

    // Debug — для разработчика
    println!("Debug: {:?}", err);
}
```

## Практический пример: торговая система

```rust
#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Trade {
    order_id: u64,
    executed_price: f64,
    executed_quantity: f64,
}

#[derive(Debug)]
struct TradingAccount {
    balance: f64,
    positions: Vec<(String, f64)>, // (symbol, quantity)
}

impl TradingAccount {
    fn validate_order(&self, order: &Order) -> Result<(), TradingError> {
        // Проверяем минимальный размер ордера
        let min_size = 0.001;
        let max_size = 1000.0;

        if order.quantity < min_size || order.quantity > max_size {
            return Err(TradingError::InvalidOrderSize {
                size: order.quantity,
                min: min_size,
                max: max_size,
            });
        }

        // Проверяем цену
        if order.price <= 0.0 {
            return Err(TradingError::InvalidPrice {
                price: order.price,
                reason: String::from("Цена должна быть положительной"),
            });
        }

        // Проверяем баланс для покупки
        if matches!(order.side, OrderSide::Buy) {
            let required = order.price * order.quantity;
            if required > self.balance {
                return Err(TradingError::InsufficientBalance {
                    required,
                    available: self.balance,
                });
            }
        }

        Ok(())
    }

    fn execute_order(&mut self, order: &Order) -> Result<Trade, TradingError> {
        // Сначала валидируем
        self.validate_order(order)?;

        // Симулируем исполнение
        match order.side {
            OrderSide::Buy => {
                let cost = order.price * order.quantity;
                self.balance -= cost;
            }
            OrderSide::Sell => {
                // Проверяем наличие позиции для продажи
                let position = self.positions.iter_mut()
                    .find(|(s, _)| s == &order.symbol);

                match position {
                    Some((_, qty)) if *qty >= order.quantity => {
                        *qty -= order.quantity;
                        self.balance += order.price * order.quantity;
                    }
                    _ => {
                        return Err(TradingError::InsufficientBalance {
                            required: order.quantity,
                            available: 0.0,
                        });
                    }
                }
            }
        }

        Ok(Trade {
            order_id: order.id,
            executed_price: order.price,
            executed_quantity: order.quantity,
        })
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 1000.0,
        positions: vec![("BTC".to_string(), 0.5)],
    };

    // Попытка купить слишком много
    let big_order = Order {
        id: 1,
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        price: 50000.0,
        quantity: 1.0,
    };

    match account.execute_order(&big_order) {
        Ok(trade) => println!("Сделка исполнена: {:?}", trade),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Успешная покупка
    let small_order = Order {
        id: 2,
        symbol: "ETH".to_string(),
        side: OrderSide::Buy,
        price: 100.0,
        quantity: 5.0,
    };

    match account.execute_order(&small_order) {
        Ok(trade) => println!("Сделка исполнена: {:?}", trade),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("Баланс после сделок: ${:.2}", account.balance);
}
```

## Конвертация ошибок: From trait

Часто нужно преобразовать ошибки из внешних библиотек в свой тип:

```rust
use std::io;
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidPrice { price: f64, reason: String },
    IoError(io::Error),
    ParseError(String),
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно {}, доступно {}", required, available)
            }
            TradingError::InvalidPrice { price, reason } => {
                write!(f, "Неверная цена {}: {}", price, reason)
            }
            TradingError::IoError(e) => write!(f, "Ошибка ввода/вывода: {}", e),
            TradingError::ParseError(s) => write!(f, "Ошибка парсинга: {}", s),
        }
    }
}

impl std::error::Error for TradingError {}

// Автоматическое преобразование io::Error в TradingError
impl From<io::Error> for TradingError {
    fn from(err: io::Error) -> Self {
        TradingError::IoError(err)
    }
}

// Автоматическое преобразование ParseFloatError в TradingError
impl From<ParseFloatError> for TradingError {
    fn from(err: ParseFloatError) -> Self {
        TradingError::ParseError(err.to_string())
    }
}

// Теперь можно использовать ? с разными типами ошибок
fn load_price_from_file(path: &str) -> Result<f64, TradingError> {
    let content = std::fs::read_to_string(path)?; // io::Error -> TradingError
    let price: f64 = content.trim().parse()?;     // ParseFloatError -> TradingError

    if price <= 0.0 {
        return Err(TradingError::InvalidPrice {
            price,
            reason: String::from("Цена из файла должна быть положительной"),
        });
    }

    Ok(price)
}

fn main() {
    match load_price_from_file("price.txt") {
        Ok(price) => println!("Цена: ${:.2}", price),
        Err(e) => println!("Ошибка загрузки: {}", e),
    }
}
```

## Обработка разных ошибок по-разному

```rust
fn handle_trading_error(error: &TradingError) {
    match error {
        TradingError::InsufficientBalance { required, available } => {
            let deficit = required - available;
            println!("Пополните баланс минимум на ${:.2}", deficit);
        }
        TradingError::InvalidOrderSize { size, min, max } => {
            if *size < *min {
                println!("Увеличьте размер ордера минимум до {}", min);
            } else {
                println!("Уменьшите размер ордера максимум до {}", max);
            }
        }
        TradingError::MarketClosed => {
            println!("Дождитесь открытия рынка");
        }
        TradingError::ConnectionLost => {
            println!("Проверьте интернет-соединение");
        }
        TradingError::Timeout { operation, .. } => {
            println!("Попробуйте повторить операцию '{}'", operation);
        }
        _ => {
            println!("Обратитесь в поддержку: {}", error);
        }
    }
}

#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    InvalidPrice { price: f64, reason: String },
    OrderNotFound { order_id: u64 },
    MarketClosed,
    SymbolNotFound { symbol: String },
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },
    ApiError { code: i32, message: String },
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let errors = vec![
        TradingError::InsufficientBalance { required: 5000.0, available: 1000.0 },
        TradingError::InvalidOrderSize { size: 0.0001, min: 0.001, max: 1000.0 },
        TradingError::MarketClosed,
        TradingError::ConnectionLost,
    ];

    for err in &errors {
        println!("\n--- Ошибка: {} ---", err);
        handle_trading_error(err);
    }
}
```

## Методы для TradingError

```rust
#[derive(Debug)]
enum TradingError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidOrderSize { size: f64, min: f64, max: f64 },
    ConnectionLost,
    Timeout { operation: String, seconds: u64 },
    ApiError { code: i32, message: String },
}

impl TradingError {
    /// Можно ли повторить операцию?
    fn is_retryable(&self) -> bool {
        matches!(self,
            TradingError::ConnectionLost |
            TradingError::Timeout { .. } |
            TradingError::ApiError { code, .. } if *code >= 500
        )
    }

    /// Это критическая ошибка?
    fn is_critical(&self) -> bool {
        matches!(self, TradingError::InsufficientBalance { .. })
    }

    /// Код ошибки для логирования
    fn error_code(&self) -> &'static str {
        match self {
            TradingError::InsufficientBalance { .. } => "E001",
            TradingError::InvalidOrderSize { .. } => "E002",
            TradingError::ConnectionLost => "E003",
            TradingError::Timeout { .. } => "E004",
            TradingError::ApiError { .. } => "E005",
        }
    }
}

fn process_with_retry(operation: impl Fn() -> Result<(), TradingError>) {
    let max_retries = 3;
    let mut attempts = 0;

    loop {
        match operation() {
            Ok(()) => {
                println!("Операция успешна!");
                break;
            }
            Err(e) if e.is_retryable() && attempts < max_retries => {
                attempts += 1;
                println!("[{}] Повтор {}/{}: {}", e.error_code(), attempts, max_retries, e);
            }
            Err(e) => {
                println!("[{}] Фатальная ошибка: {}", e.error_code(), e);
                if e.is_critical() {
                    println!("Требуется немедленное вмешательство!");
                }
                break;
            }
        }
    }
}

impl std::fmt::Display for TradingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let mut counter = 0;

    process_with_retry(|| {
        counter += 1;
        if counter < 3 {
            Err(TradingError::ConnectionLost)
        } else {
            Ok(())
        }
    });
}
```

## Что мы узнали

| Концепт | Описание |
|---------|----------|
| `enum` для ошибок | Перечисление всех возможных ошибок |
| `Display` trait | Красивый вывод для пользователя |
| `Error` trait | Совместимость со стандартной библиотекой |
| `From` trait | Автоматическое преобразование ошибок |
| `match` на ошибках | Разная логика для разных ошибок |
| Методы на enum | Полезные функции типа `is_retryable()` |

## Упражнения

1. **Расширь TradingError**: добавь варианты `RateLimitExceeded { retry_after: u64 }` и `AuthenticationFailed { reason: String }`

2. **Иерархия ошибок**: создай отдельные enum `OrderError`, `ConnectionError`, `AccountError` и объедини их в `TradingError`

3. **Логирование**: добавь метод `log_level(&self) -> &str`, который возвращает "ERROR", "WARN" или "INFO" в зависимости от типа ошибки

4. **Контекст**: реализуй метод `with_context(self, ctx: &str) -> Self`, который добавляет контекст к ошибке

## Домашнее задание

Создай полноценную систему ошибок для торгового бота:

1. Определи `TradingError` с минимум 10 вариантами ошибок
2. Реализуй `Display`, `Debug`, `Error`
3. Добавь `From` для `std::io::Error` и `serde_json::Error`
4. Реализуй методы `is_retryable()`, `is_critical()`, `suggested_action() -> String`
5. Напиши функцию `execute_with_error_handling()`, которая обрабатывает все типы ошибок по-разному

## Навигация

[← День 77: Оператор ?](../077-question-mark-operator/ru.md) | [День 79: Vec — динамический список сделок →](../079-vec-dynamic-trades/ru.md)
