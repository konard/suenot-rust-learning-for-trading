# День 116: Документируем возможные ошибки

## Аналогия из трейдинга

Представь, что ты создаёшь торгового бота и передаёшь его другому трейдеру. Ему нужно знать:
- Какие ошибки может вернуть функция отправки ордера?
- При каких условиях программа может "упасть"?
- Какие данные считаются невалидными?

Это как инструкция к торговому терминалу: "При отключении интернета ордер не будет отправлен", "При недостатке средств вернётся ошибка InsufficientBalance".

В Rust мы используем специальные секции в документационных комментариях (`///`) для описания возможных ошибок: `# Errors`, `# Panics` и `# Safety`.

## Секция # Errors

Используется для функций, возвращающих `Result`. Описывает, при каких условиях функция вернёт `Err`.

```rust
use std::fmt;

/// Ошибки при отправке ордера
#[derive(Debug)]
enum OrderError {
    InsufficientBalance,
    InvalidPrice,
    MarketClosed,
    ConnectionFailed,
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::InsufficientBalance => write!(f, "Недостаточно средств"),
            OrderError::InvalidPrice => write!(f, "Некорректная цена"),
            OrderError::MarketClosed => write!(f, "Рынок закрыт"),
            OrderError::ConnectionFailed => write!(f, "Ошибка соединения"),
        }
    }
}

/// Отправляет рыночный ордер на биржу.
///
/// # Arguments
///
/// * `symbol` - Торговый символ (например, "BTCUSDT")
/// * `quantity` - Количество для покупки
/// * `balance` - Доступный баланс
/// * `market_open` - Открыт ли рынок
///
/// # Returns
///
/// ID успешно созданного ордера
///
/// # Errors
///
/// Возвращает ошибку в следующих случаях:
///
/// * [`OrderError::InsufficientBalance`] - если `balance` меньше стоимости ордера
/// * [`OrderError::InvalidPrice`] - если текущая цена равна нулю или отрицательна
/// * [`OrderError::MarketClosed`] - если `market_open` равен `false`
/// * [`OrderError::ConnectionFailed`] - если не удалось соединиться с биржей
fn send_market_order(
    symbol: &str,
    quantity: f64,
    balance: f64,
    market_open: bool,
) -> Result<u64, OrderError> {
    if !market_open {
        return Err(OrderError::MarketClosed);
    }

    // Симуляция получения цены
    let current_price = get_current_price(symbol);

    if current_price <= 0.0 {
        return Err(OrderError::InvalidPrice);
    }

    let order_value = current_price * quantity;

    if balance < order_value {
        return Err(OrderError::InsufficientBalance);
    }

    // Симуляция успешного ордера
    Ok(12345)
}

fn get_current_price(_symbol: &str) -> f64 {
    42000.0  // Симуляция
}

fn main() {
    match send_market_order("BTCUSDT", 0.1, 5000.0, true) {
        Ok(order_id) => println!("Ордер создан: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Секция # Panics

Описывает условия, при которых функция вызовет `panic!`. Это важно, потому что паника завершает программу (или поток).

```rust
/// Рассчитывает прибыль в процентах.
///
/// # Arguments
///
/// * `entry_price` - Цена входа в позицию
/// * `exit_price` - Цена выхода из позиции
///
/// # Returns
///
/// Процент прибыли или убытка
///
/// # Panics
///
/// Паникует если `entry_price` равна нулю, так как это приведёт
/// к делению на ноль. Всегда проверяйте цену входа перед вызовом.
///
/// # Examples
///
/// ```
/// let profit = calculate_profit_percent(100.0, 110.0);
/// assert_eq!(profit, 10.0);
/// ```
fn calculate_profit_percent(entry_price: f64, exit_price: f64) -> f64 {
    if entry_price == 0.0 {
        panic!("Цена входа не может быть нулём!");
    }

    ((exit_price - entry_price) / entry_price) * 100.0
}

fn main() {
    println!("Прибыль: {:.2}%", calculate_profit_percent(42000.0, 44100.0));
}
```

## Лучше Result вместо panic

В трейдинге критически важна надёжность. Предпочитайте `Result` вместо `panic!`:

```rust
use std::fmt;

#[derive(Debug)]
enum CalculationError {
    DivisionByZero,
    NegativePrice,
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationError::DivisionByZero => write!(f, "Деление на ноль"),
            CalculationError::NegativePrice => write!(f, "Отрицательная цена"),
        }
    }
}

/// Безопасно рассчитывает прибыль в процентах.
///
/// # Arguments
///
/// * `entry_price` - Цена входа в позицию
/// * `exit_price` - Цена выхода из позиции
///
/// # Returns
///
/// Процент прибыли или убытка, обёрнутый в `Result`
///
/// # Errors
///
/// * [`CalculationError::DivisionByZero`] - если `entry_price` равна нулю
/// * [`CalculationError::NegativePrice`] - если любая из цен отрицательна
///
/// # Examples
///
/// ```
/// let profit = calculate_profit_percent_safe(100.0, 110.0);
/// assert_eq!(profit.unwrap(), 10.0);
/// ```
fn calculate_profit_percent_safe(
    entry_price: f64,
    exit_price: f64,
) -> Result<f64, CalculationError> {
    if entry_price < 0.0 || exit_price < 0.0 {
        return Err(CalculationError::NegativePrice);
    }

    if entry_price == 0.0 {
        return Err(CalculationError::DivisionByZero);
    }

    Ok(((exit_price - entry_price) / entry_price) * 100.0)
}

fn main() {
    match calculate_profit_percent_safe(42000.0, 44100.0) {
        Ok(profit) => println!("Прибыль: {:.2}%", profit),
        Err(e) => println!("Ошибка расчёта: {}", e),
    }
}
```

## Документация для Option

Для функций, возвращающих `Option`, используйте `# Returns` с объяснением когда возвращается `None`:

```rust
/// Ищет ордер по ID в списке.
///
/// # Arguments
///
/// * `orders` - Список кортежей (order_id, price, quantity)
/// * `order_id` - ID искомого ордера
///
/// # Returns
///
/// Возвращает `Some((price, quantity))` если ордер найден,
/// или `None` если ордер с данным ID отсутствует в списке.
fn find_order(orders: &[(u64, f64, f64)], order_id: u64) -> Option<(f64, f64)> {
    orders
        .iter()
        .find(|(id, _, _)| *id == order_id)
        .map(|(_, price, quantity)| (*price, *quantity))
}

fn main() {
    let orders = vec![
        (1, 42000.0, 0.5),
        (2, 42100.0, 0.3),
        (3, 41900.0, 0.7),
    ];

    match find_order(&orders, 2) {
        Some((price, qty)) => println!("Ордер найден: {} @ {}", qty, price),
        None => println!("Ордер не найден"),
    }
}
```

## Комплексный пример: API клиент биржи

```rust
use std::fmt;

/// Типы ордеров
#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
}

/// Ошибки торгового API
#[derive(Debug)]
enum TradingApiError {
    /// Недостаточный баланс для совершения сделки
    InsufficientBalance { required: f64, available: f64 },
    /// Рынок закрыт в данный момент
    MarketClosed,
    /// Некорректные параметры ордера
    InvalidOrder(String),
    /// Ошибка сети
    NetworkError(String),
    /// Ордер отклонён биржей
    OrderRejected { reason: String },
}

impl fmt::Display for TradingApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingApiError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно {}, доступно {}", required, available)
            }
            TradingApiError::MarketClosed => write!(f, "Рынок закрыт"),
            TradingApiError::InvalidOrder(msg) => write!(f, "Некорректный ордер: {}", msg),
            TradingApiError::NetworkError(msg) => write!(f, "Ошибка сети: {}", msg),
            TradingApiError::OrderRejected { reason } => {
                write!(f, "Ордер отклонён: {}", reason)
            }
        }
    }
}

struct TradingClient {
    balance: f64,
    connected: bool,
    market_open: bool,
}

impl TradingClient {
    /// Создаёт нового клиента торгового API.
    ///
    /// # Arguments
    ///
    /// * `initial_balance` - Начальный баланс счёта
    ///
    /// # Panics
    ///
    /// Паникует если `initial_balance` отрицателен.
    /// Используйте [`TradingClient::try_new`] для безопасного создания.
    fn new(initial_balance: f64) -> Self {
        if initial_balance < 0.0 {
            panic!("Начальный баланс не может быть отрицательным");
        }

        TradingClient {
            balance: initial_balance,
            connected: true,
            market_open: true,
        }
    }

    /// Безопасно создаёт нового клиента торгового API.
    ///
    /// # Arguments
    ///
    /// * `initial_balance` - Начальный баланс счёта
    ///
    /// # Returns
    ///
    /// `Some(TradingClient)` если баланс валиден, `None` если отрицателен.
    fn try_new(initial_balance: f64) -> Option<Self> {
        if initial_balance < 0.0 {
            return None;
        }

        Some(TradingClient {
            balance: initial_balance,
            connected: true,
            market_open: true,
        })
    }

    /// Размещает новый ордер на бирже.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Торговый символ (например, "BTCUSDT", "ETHUSDT")
    /// * `quantity` - Количество актива для покупки/продажи
    /// * `order_type` - Тип ордера: Market, Limit или StopLoss
    ///
    /// # Returns
    ///
    /// ID успешно созданного ордера
    ///
    /// # Errors
    ///
    /// Возвращает ошибку в следующих случаях:
    ///
    /// * [`TradingApiError::NetworkError`] - если клиент не подключен к бирже.
    ///   Вызовите `reconnect()` перед повторной попыткой.
    ///
    /// * [`TradingApiError::MarketClosed`] - если рынок закрыт.
    ///   Рыночные ордера невозможны в нерабочее время.
    ///
    /// * [`TradingApiError::InvalidOrder`] - если:
    ///   - `symbol` пустой или содержит недопустимые символы
    ///   - `quantity` меньше или равно нулю
    ///   - Для Limit ордера: цена меньше или равна нулю
    ///   - Для StopLoss: trigger_price меньше или равен нулю
    ///
    /// * [`TradingApiError::InsufficientBalance`] - если баланс меньше
    ///   стоимости ордера. Содержит требуемую и доступную сумму.
    ///
    /// * [`TradingApiError::OrderRejected`] - если биржа отклонила ордер
    ///   по внутренним причинам (например, превышен лимит позиции).
    ///
    /// # Examples
    ///
    /// ```
    /// let client = TradingClient::new(10000.0);
    ///
    /// // Рыночный ордер
    /// match client.place_order("BTCUSDT", 0.1, OrderType::Market) {
    ///     Ok(id) => println!("Ордер создан: {}", id),
    ///     Err(e) => eprintln!("Ошибка: {}", e),
    /// }
    ///
    /// // Лимитный ордер
    /// let limit = OrderType::Limit { price: 42000.0 };
    /// client.place_order("BTCUSDT", 0.5, limit)?;
    /// ```
    fn place_order(
        &self,
        symbol: &str,
        quantity: f64,
        order_type: OrderType,
    ) -> Result<u64, TradingApiError> {
        // Проверка соединения
        if !self.connected {
            return Err(TradingApiError::NetworkError(
                "Нет соединения с биржей".to_string()
            ));
        }

        // Проверка рынка
        if !self.market_open {
            return Err(TradingApiError::MarketClosed);
        }

        // Валидация символа
        if symbol.is_empty() {
            return Err(TradingApiError::InvalidOrder(
                "Символ не может быть пустым".to_string()
            ));
        }

        // Валидация количества
        if quantity <= 0.0 {
            return Err(TradingApiError::InvalidOrder(
                format!("Количество должно быть положительным, получено: {}", quantity)
            ));
        }

        // Валидация типа ордера
        match &order_type {
            OrderType::Limit { price } if *price <= 0.0 => {
                return Err(TradingApiError::InvalidOrder(
                    format!("Цена лимитного ордера должна быть положительной: {}", price)
                ));
            }
            OrderType::StopLoss { trigger_price } if *trigger_price <= 0.0 => {
                return Err(TradingApiError::InvalidOrder(
                    format!("Триггер-цена должна быть положительной: {}", trigger_price)
                ));
            }
            _ => {}
        }

        // Расчёт стоимости ордера (симуляция)
        let price = match &order_type {
            OrderType::Market => 42000.0,  // Текущая рыночная цена
            OrderType::Limit { price } => *price,
            OrderType::StopLoss { trigger_price } => *trigger_price,
        };

        let order_value = price * quantity;

        // Проверка баланса
        if self.balance < order_value {
            return Err(TradingApiError::InsufficientBalance {
                required: order_value,
                available: self.balance,
            });
        }

        // Успешное создание ордера
        Ok(12345)
    }

    /// Получает текущую цену актива.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Торговый символ
    ///
    /// # Returns
    ///
    /// Текущая цена актива или `None`, если:
    /// - Символ не найден на бирже
    /// - Нет доступных котировок
    /// - Клиент не подключен к бирже
    fn get_price(&self, symbol: &str) -> Option<f64> {
        if !self.connected || symbol.is_empty() {
            return None;
        }

        // Симуляция получения цены
        match symbol {
            "BTCUSDT" => Some(42000.0),
            "ETHUSDT" => Some(2500.0),
            _ => None,
        }
    }
}

fn main() {
    let client = TradingClient::new(10000.0);

    // Пример успешного ордера
    match client.place_order("BTCUSDT", 0.1, OrderType::Market) {
        Ok(id) => println!("Ордер создан: {}", id),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример ошибки - недостаточно средств
    match client.place_order("BTCUSDT", 1.0, OrderType::Market) {
        Ok(id) => println!("Ордер создан: {}", id),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Генерация документации

Чтобы увидеть свою документацию в красивом виде:

```bash
cargo doc --open
```

Это создаст HTML-документацию со всеми секциями # Errors, # Panics и примерами.

## Что мы узнали

| Секция | Назначение | Когда использовать |
|--------|------------|-------------------|
| `# Errors` | Описание ошибок Result | Функции возвращающие Result |
| `# Panics` | Условия паники | Функции с assert!/panic! |
| `# Returns` + None | Когда Option = None | Функции возвращающие Option |
| `# Safety` | Инварианты unsafe | unsafe функции |

## Практические задания

1. **Документирование торгового валидатора**

   Напиши функцию `validate_order(price, quantity, side)` и задокументируй все возможные ошибки валидации.

2. **Документация калькулятора рисков**

   Создай функцию `calculate_position_size(balance, risk_percent, entry, stop_loss)` с полной документацией ошибок.

3. **API методы с документацией**

   Реализуй методы `get_balance()`, `cancel_order(id)`, `get_open_orders()` для `TradingClient` с документацией всех возможных ошибок.

## Домашнее задание

1. Создай структуру `Portfolio` с методами:
   - `add_position(symbol, quantity, price)` - с документацией ошибок
   - `remove_position(symbol)` - с документацией когда возвращается `None`
   - `total_value()` - с документацией возможных паник

2. Напиши функцию `parse_trade_signal(signal_str)` которая парсит торговые сигналы вида "BUY:BTCUSDT:0.5" и задокументируй все ошибки парсинга.

3. Реализуй `OrderBook` с методом `get_best_bid()` и `get_best_ask()`, документируя когда возвращается `None`.

4. Используй `cargo doc --open` чтобы просмотреть свою документацию и убедиться, что она понятна другим разработчикам.

## Навигация

[← Предыдущий день](../115-mocking-errors-in-tests/ru.md) | [Следующий день →](../117-async-errors-preview/ru.md)
