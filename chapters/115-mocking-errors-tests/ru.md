# День 115: Моки ошибок в тестах

## Аналогия из трейдинга

Представь, что ты тестируешь свою торговую систему. Как проверить, что система правильно реагирует на отказ биржи? Не будешь же ты ждать реального сбоя! Вместо этого ты **имитируешь** (мокаешь) ошибку — создаёшь "подставного" провайдера данных, который специально возвращает ошибки.

Это как пожарная тренировка: не нужно поджигать здание, чтобы проверить, знают ли люди, где выход.

## Зачем мокать ошибки?

1. **Надёжность** — убедиться, что система корректно обрабатывает сбои
2. **Изоляция** — тестировать код без зависимости от внешних сервисов
3. **Воспроизводимость** — гарантировать одинаковое поведение теста
4. **Скорость** — не ждать таймаутов реальных сервисов

## Базовый мок с Result

```rust
// Трейт для провайдера рыночных данных
trait MarketDataProvider {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
}

// Реальная реализация (для продакшена)
struct RealExchange;

impl MarketDataProvider for RealExchange {
    fn get_price(&self, symbol: &str) -> Result<f64, String> {
        // В реальности здесь был бы HTTP-запрос к API биржи
        Ok(42000.0)
    }
}

// Мок, который всегда возвращает ошибку
struct FailingExchange {
    error_message: String,
}

impl MarketDataProvider for FailingExchange {
    fn get_price(&self, _symbol: &str) -> Result<f64, String> {
        Err(self.error_message.clone())
    }
}

// Функция, которую мы тестируем
fn calculate_portfolio_value<P: MarketDataProvider>(
    provider: &P,
    holdings: &[(&str, f64)],
) -> Result<f64, String> {
    let mut total = 0.0;
    for (symbol, quantity) in holdings {
        let price = provider.get_price(symbol)?;
        total += price * quantity;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_with_failing_exchange() {
        let mock = FailingExchange {
            error_message: String::from("Connection timeout"),
        };
        let holdings = vec![("BTC", 0.5), ("ETH", 10.0)];

        let result = calculate_portfolio_value(&mock, &holdings);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Connection timeout");
    }
}

fn main() {
    // Демонстрация с реальным провайдером
    let exchange = RealExchange;
    let holdings = vec![("BTC", 0.5)];
    println!("Portfolio: {:?}", calculate_portfolio_value(&exchange, &holdings));

    // Демонстрация с моком ошибки
    let failing = FailingExchange {
        error_message: String::from("API rate limit exceeded"),
    };
    println!("With error: {:?}", calculate_portfolio_value(&failing, &holdings));
}
```

## Конфигурируемые моки

```rust
// Мок с настраиваемым поведением
struct ConfigurableMock {
    prices: std::collections::HashMap<String, Result<f64, String>>,
}

impl ConfigurableMock {
    fn new() -> Self {
        ConfigurableMock {
            prices: std::collections::HashMap::new(),
        }
    }

    fn set_price(&mut self, symbol: &str, price: f64) {
        self.prices.insert(symbol.to_string(), Ok(price));
    }

    fn set_error(&mut self, symbol: &str, error: &str) {
        self.prices.insert(symbol.to_string(), Err(error.to_string()));
    }
}

impl MarketDataProvider for ConfigurableMock {
    fn get_price(&self, symbol: &str) -> Result<f64, String> {
        self.prices
            .get(symbol)
            .cloned()
            .unwrap_or(Err(format!("Unknown symbol: {}", symbol)))
    }
}

trait MarketDataProvider {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_failure() {
        let mut mock = ConfigurableMock::new();
        mock.set_price("BTC", 42000.0);
        mock.set_error("ETH", "Symbol delisted");

        // BTC работает
        assert!(mock.get_price("BTC").is_ok());

        // ETH возвращает ошибку
        let eth_result = mock.get_price("ETH");
        assert!(eth_result.is_err());
        assert!(eth_result.unwrap_err().contains("delisted"));

        // Неизвестный символ
        let unknown = mock.get_price("XYZ");
        assert!(unknown.is_err());
        assert!(unknown.unwrap_err().contains("Unknown symbol"));
    }
}

fn main() {
    let mut mock = ConfigurableMock::new();
    mock.set_price("BTC", 42000.0);
    mock.set_error("ETH", "Symbol suspended");

    println!("BTC: {:?}", mock.get_price("BTC"));
    println!("ETH: {:?}", mock.get_price("ETH"));
    println!("XYZ: {:?}", mock.get_price("XYZ"));
}
```

## Мок с последовательностью ответов

```rust
use std::cell::RefCell;

// Мок, который возвращает разные результаты при каждом вызове
struct SequenceMock {
    responses: RefCell<Vec<Result<f64, String>>>,
    call_count: RefCell<usize>,
}

impl SequenceMock {
    fn new(responses: Vec<Result<f64, String>>) -> Self {
        SequenceMock {
            responses: RefCell::new(responses),
            call_count: RefCell::new(0),
        }
    }

    fn get_call_count(&self) -> usize {
        *self.call_count.borrow()
    }
}

trait MarketDataProvider {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
}

impl MarketDataProvider for SequenceMock {
    fn get_price(&self, _symbol: &str) -> Result<f64, String> {
        let mut count = self.call_count.borrow_mut();
        let responses = self.responses.borrow();

        let result = if *count < responses.len() {
            responses[*count].clone()
        } else {
            Err(String::from("No more responses configured"))
        };

        *count += 1;
        result
    }
}

// Функция с логикой повторных попыток
fn get_price_with_retry<P: MarketDataProvider>(
    provider: &P,
    symbol: &str,
    max_retries: usize,
) -> Result<f64, String> {
    let mut last_error = String::from("No attempts made");

    for attempt in 0..=max_retries {
        match provider.get_price(symbol) {
            Ok(price) => return Ok(price),
            Err(e) => {
                last_error = format!("Attempt {}: {}", attempt + 1, e);
                // В реальности здесь была бы задержка
            }
        }
    }

    Err(format!("All {} retries failed. Last: {}", max_retries + 1, last_error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_succeeds_on_third_attempt() {
        let mock = SequenceMock::new(vec![
            Err(String::from("Timeout")),
            Err(String::from("Connection reset")),
            Ok(42000.0),
        ]);

        let result = get_price_with_retry(&mock, "BTC", 3);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42000.0);
        assert_eq!(mock.get_call_count(), 3);
    }

    #[test]
    fn test_retry_exhausted() {
        let mock = SequenceMock::new(vec![
            Err(String::from("Error 1")),
            Err(String::from("Error 2")),
            Err(String::from("Error 3")),
        ]);

        let result = get_price_with_retry(&mock, "BTC", 2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("All 3 retries failed"));
    }
}

fn main() {
    // Симуляция временного сбоя
    let mock = SequenceMock::new(vec![
        Err(String::from("Connection timeout")),
        Err(String::from("Server busy")),
        Ok(42000.0),
    ]);

    println!("Trying to get price with retries...");
    let result = get_price_with_retry(&mock, "BTC", 3);
    println!("Result: {:?}", result);
    println!("Total calls: {}", mock.get_call_count());
}
```

## Тестирование обработки сетевых ошибок

```rust
// Типы сетевых ошибок
#[derive(Debug, Clone)]
enum NetworkError {
    Timeout,
    ConnectionRefused,
    DnsError(String),
    TlsError,
    RateLimited { retry_after: u64 },
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::Timeout => write!(f, "Connection timeout"),
            NetworkError::ConnectionRefused => write!(f, "Connection refused"),
            NetworkError::DnsError(host) => write!(f, "DNS lookup failed for {}", host),
            NetworkError::TlsError => write!(f, "TLS handshake failed"),
            NetworkError::RateLimited { retry_after } => {
                write!(f, "Rate limited, retry after {} seconds", retry_after)
            }
        }
    }
}

trait ExchangeApi {
    fn fetch_orderbook(&self, symbol: &str) -> Result<Orderbook, NetworkError>;
}

#[derive(Debug, Clone)]
struct Orderbook {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

struct NetworkErrorMock {
    error: NetworkError,
}

impl ExchangeApi for NetworkErrorMock {
    fn fetch_orderbook(&self, _symbol: &str) -> Result<Orderbook, NetworkError> {
        Err(self.error.clone())
    }
}

// Обработчик с разной логикой для разных ошибок
fn handle_orderbook_request<A: ExchangeApi>(
    api: &A,
    symbol: &str,
) -> String {
    match api.fetch_orderbook(symbol) {
        Ok(book) => format!("Got {} bids, {} asks", book.bids.len(), book.asks.len()),
        Err(NetworkError::Timeout) => String::from("Request timed out, will retry"),
        Err(NetworkError::RateLimited { retry_after }) => {
            format!("Rate limited, waiting {} seconds", retry_after)
        }
        Err(NetworkError::ConnectionRefused) => String::from("Exchange is down"),
        Err(e) => format!("Unexpected error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_handling() {
        let mock = NetworkErrorMock {
            error: NetworkError::Timeout,
        };
        let result = handle_orderbook_request(&mock, "BTC/USDT");
        assert!(result.contains("retry"));
    }

    #[test]
    fn test_rate_limit_handling() {
        let mock = NetworkErrorMock {
            error: NetworkError::RateLimited { retry_after: 30 },
        };
        let result = handle_orderbook_request(&mock, "BTC/USDT");
        assert!(result.contains("30 seconds"));
    }

    #[test]
    fn test_connection_refused_handling() {
        let mock = NetworkErrorMock {
            error: NetworkError::ConnectionRefused,
        };
        let result = handle_orderbook_request(&mock, "BTC/USDT");
        assert!(result.contains("down"));
    }
}

fn main() {
    // Тестируем разные типы ошибок
    let errors = vec![
        NetworkError::Timeout,
        NetworkError::RateLimited { retry_after: 60 },
        NetworkError::ConnectionRefused,
        NetworkError::DnsError(String::from("api.exchange.com")),
    ];

    for error in errors {
        let mock = NetworkErrorMock { error };
        let result = handle_orderbook_request(&mock, "BTC/USDT");
        println!("{}", result);
    }
}
```

## Мок для тестирования торговых операций

```rust
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum OrderError {
    InsufficientBalance,
    InvalidPrice,
    InvalidQuantity,
    MarketClosed,
    SymbolNotFound,
}

#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

trait OrderExecutor {
    fn place_order(&self, symbol: &str, side: &str, price: f64, quantity: f64)
        -> Result<Order, OrderError>;
}

// Мок с записью вызовов для проверки
struct OrderExecutorMock {
    should_fail: bool,
    error: OrderError,
    recorded_calls: RefCell<Vec<(String, String, f64, f64)>>,
}

impl OrderExecutorMock {
    fn new_success() -> Self {
        OrderExecutorMock {
            should_fail: false,
            error: OrderError::InsufficientBalance,
            recorded_calls: RefCell::new(Vec::new()),
        }
    }

    fn new_failing(error: OrderError) -> Self {
        OrderExecutorMock {
            should_fail: true,
            error,
            recorded_calls: RefCell::new(Vec::new()),
        }
    }

    fn get_calls(&self) -> Vec<(String, String, f64, f64)> {
        self.recorded_calls.borrow().clone()
    }
}

impl OrderExecutor for OrderExecutorMock {
    fn place_order(&self, symbol: &str, side: &str, price: f64, quantity: f64)
        -> Result<Order, OrderError>
    {
        // Записываем вызов
        self.recorded_calls.borrow_mut().push((
            symbol.to_string(),
            side.to_string(),
            price,
            quantity,
        ));

        if self.should_fail {
            Err(self.error.clone())
        } else {
            Ok(Order {
                id: format!("ORD-{}", self.recorded_calls.borrow().len()),
                symbol: symbol.to_string(),
                side: side.to_string(),
                price,
                quantity,
            })
        }
    }
}

// Торговая стратегия
fn execute_strategy<E: OrderExecutor>(
    executor: &E,
    signal: &str,
    symbol: &str,
    price: f64,
) -> Result<String, String> {
    let (side, qty) = match signal {
        "BUY" => ("BUY", 0.1),
        "SELL" => ("SELL", 0.1),
        _ => return Err(String::from("Unknown signal")),
    };

    match executor.place_order(symbol, side, price, qty) {
        Ok(order) => Ok(format!("Order placed: {:?}", order.id)),
        Err(OrderError::InsufficientBalance) => {
            Err(String::from("Not enough balance, skipping trade"))
        }
        Err(OrderError::MarketClosed) => {
            Err(String::from("Market closed, queuing order"))
        }
        Err(e) => Err(format!("Order failed: {:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_with_insufficient_balance() {
        let mock = OrderExecutorMock::new_failing(OrderError::InsufficientBalance);

        let result = execute_strategy(&mock, "BUY", "BTC/USDT", 42000.0);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not enough balance"));

        // Проверяем, что ордер был отправлен с правильными параметрами
        let calls = mock.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "BTC/USDT");
        assert_eq!(calls[0].1, "BUY");
    }

    #[test]
    fn test_strategy_with_market_closed() {
        let mock = OrderExecutorMock::new_failing(OrderError::MarketClosed);

        let result = execute_strategy(&mock, "SELL", "ETH/USDT", 2500.0);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Market closed"));
    }

    #[test]
    fn test_successful_order() {
        let mock = OrderExecutorMock::new_success();

        let result = execute_strategy(&mock, "BUY", "BTC/USDT", 42000.0);

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Order placed"));
    }
}

fn main() {
    // Демонстрация успешного ордера
    let success_mock = OrderExecutorMock::new_success();
    println!("Success: {:?}", execute_strategy(&success_mock, "BUY", "BTC/USDT", 42000.0));

    // Демонстрация ошибки баланса
    let fail_mock = OrderExecutorMock::new_failing(OrderError::InsufficientBalance);
    println!("Failure: {:?}", execute_strategy(&fail_mock, "BUY", "BTC/USDT", 42000.0));

    // Проверка записанных вызовов
    println!("Recorded calls: {:?}", fail_mock.get_calls());
}
```

## Паттерн: мок с состоянием

```rust
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct RiskLimitError {
    current_exposure: f64,
    max_exposure: f64,
    requested: f64,
}

trait RiskManager {
    fn check_trade(&self, symbol: &str, value: f64) -> Result<(), RiskLimitError>;
    fn get_exposure(&self, symbol: &str) -> f64;
}

// Мок с изменяемым состоянием
struct StatefulRiskMock {
    exposures: RefCell<std::collections::HashMap<String, f64>>,
    max_exposure: f64,
}

impl StatefulRiskMock {
    fn new(max_exposure: f64) -> Self {
        StatefulRiskMock {
            exposures: RefCell::new(std::collections::HashMap::new()),
            max_exposure,
        }
    }

    fn set_exposure(&self, symbol: &str, value: f64) {
        self.exposures.borrow_mut().insert(symbol.to_string(), value);
    }
}

impl RiskManager for StatefulRiskMock {
    fn check_trade(&self, symbol: &str, value: f64) -> Result<(), RiskLimitError> {
        let current = self.get_exposure(symbol);
        let new_exposure = current + value;

        if new_exposure > self.max_exposure {
            Err(RiskLimitError {
                current_exposure: current,
                max_exposure: self.max_exposure,
                requested: value,
            })
        } else {
            // Обновляем состояние при успешной проверке
            self.exposures.borrow_mut().insert(symbol.to_string(), new_exposure);
            Ok(())
        }
    }

    fn get_exposure(&self, symbol: &str) -> f64 {
        *self.exposures.borrow().get(symbol).unwrap_or(&0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_limit_exceeded() {
        let mock = StatefulRiskMock::new(10000.0);
        mock.set_exposure("BTC", 9000.0);

        let result = mock.check_trade("BTC", 2000.0);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.current_exposure, 9000.0);
        assert_eq!(err.max_exposure, 10000.0);
        assert_eq!(err.requested, 2000.0);
    }

    #[test]
    fn test_accumulating_exposure() {
        let mock = StatefulRiskMock::new(10000.0);

        // Первая сделка - OK
        assert!(mock.check_trade("ETH", 3000.0).is_ok());
        assert_eq!(mock.get_exposure("ETH"), 3000.0);

        // Вторая сделка - OK
        assert!(mock.check_trade("ETH", 3000.0).is_ok());
        assert_eq!(mock.get_exposure("ETH"), 6000.0);

        // Третья сделка - превышение лимита
        assert!(mock.check_trade("ETH", 5000.0).is_err());
    }
}

fn main() {
    let risk_mock = StatefulRiskMock::new(10000.0);

    println!("Initial ETH exposure: {}", risk_mock.get_exposure("ETH"));

    let trade1 = risk_mock.check_trade("ETH", 4000.0);
    println!("Trade 1 (4000): {:?}", trade1);
    println!("ETH exposure: {}", risk_mock.get_exposure("ETH"));

    let trade2 = risk_mock.check_trade("ETH", 4000.0);
    println!("Trade 2 (4000): {:?}", trade2);
    println!("ETH exposure: {}", risk_mock.get_exposure("ETH"));

    let trade3 = risk_mock.check_trade("ETH", 4000.0);
    println!("Trade 3 (4000): {:?}", trade3);
}
```

## Что мы узнали

| Паттерн | Описание | Когда использовать |
|---------|----------|-------------------|
| Простой мок | Всегда возвращает ошибку | Тест базовой обработки |
| Конфигурируемый | Разные ответы для разных входов | Частичные сбои |
| Последовательный | Разные ответы на каждый вызов | Тест retry-логики |
| С записью | Сохраняет все вызовы | Проверка параметров |
| С состоянием | Изменяется между вызовами | Сложные сценарии |

## Практические задания

1. Создай мок для API биржи, который возвращает `RateLimited` на каждый 5-й запрос

2. Реализуй мок с таймаутом: первые N запросов "зависают" (возвращают `Timeout`), потом работают нормально

3. Напиши мок для проверки баланса, который отслеживает историю всех запросов и позволяет проверить их последовательность

4. Создай мок, который симулирует частичное исполнение ордера (ордер на 10 BTC исполняется только на 7 BTC и возвращает ошибку `PartialFill`)

## Домашнее задание

1. Реализуй полноценную систему моков для тестирования торгового бота:
   - Мок для маркет-данных (цены, стаканы)
   - Мок для исполнения ордеров (с разными типами ошибок)
   - Мок для риск-менеджмента

2. Напиши тесты для сценария "биржа упала посреди торговой сессии":
   - Открытые позиции должны быть защищены
   - Новые ордера должны отклоняться
   - Система должна логировать проблему

3. Создай мок, который симулирует "сплит-брейн" (разные ноды биржи возвращают разные цены) и напиши тест, проверяющий, что система обнаруживает это расхождение

## Навигация

[← Предыдущий день](../114-testing-errors-handling/ru.md) | [Следующий день →](../116-documenting-errors/ru.md)
