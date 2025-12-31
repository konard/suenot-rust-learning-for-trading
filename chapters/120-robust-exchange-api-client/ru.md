# День 120: Проект — Робастный API клиент биржи

## Аналогия из трейдинга

Представь, что ты строишь торгового бота, который работает с биржей 24/7. Биржа может временно недоступна, API может вернуть ошибку, сеть может оборваться в середине сделки. Профессиональный трейдер не паникует — у него есть план на каждый случай. Наш API клиент должен быть таким же: **надёжным, предсказуемым и восстанавливающимся после ошибок**.

В этом проекте мы объединим все знания месяца об обработке ошибок:
- Собственные типы ошибок (`thiserror`)
- Контекст ошибок (`anyhow`)
- Retry логика с exponential backoff
- Circuit breaker для защиты от каскадных сбоев
- Graceful degradation
- Валидация входных данных

## Архитектура проекта

```
exchange_client/
├── src/
│   ├── main.rs           # Точка входа
│   ├── client.rs         # API клиент
│   ├── error.rs          # Типы ошибок
│   ├── retry.rs          # Retry логика
│   ├── circuit_breaker.rs # Circuit breaker
│   ├── models.rs         # Модели данных
│   └── validation.rs     # Валидация
└── Cargo.toml
```

## Шаг 1: Определяем типы ошибок

```rust
// error.rs
use thiserror::Error;

/// Ошибки API клиента биржи
#[derive(Error, Debug)]
pub enum ExchangeError {
    /// Ошибка сети — можно повторить запрос
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        is_retryable: bool,
    },

    /// Ошибка авторизации — нужно обновить ключи
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Ошибка валидации — неправильные входные данные
    #[error("Validation error: {0}")]
    Validation(String),

    /// Ошибка лимитов — слишком много запросов
    #[error("Rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimit { retry_after_secs: u64 },

    /// Ошибка биржи — проблема на стороне сервера
    #[error("Exchange error ({code}): {message}")]
    Exchange { code: i32, message: String },

    /// Недостаточно средств
    #[error("Insufficient funds: need {required}, have {available}")]
    InsufficientFunds { required: f64, available: f64 },

    /// Ордер не найден
    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    /// Circuit breaker открыт
    #[error("Circuit breaker is open, service temporarily unavailable")]
    CircuitBreakerOpen,

    /// Таймаут операции
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Неизвестная ошибка
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ExchangeError {
    /// Можно ли повторить запрос после этой ошибки?
    pub fn is_retryable(&self) -> bool {
        match self {
            ExchangeError::Network { is_retryable, .. } => *is_retryable,
            ExchangeError::RateLimit { .. } => true,
            ExchangeError::Timeout { .. } => true,
            ExchangeError::Exchange { code, .. } => {
                // 5xx ошибки сервера обычно временные
                *code >= 500 && *code < 600
            }
            _ => false,
        }
    }

    /// Получить время ожидания перед повтором (если применимо)
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            ExchangeError::RateLimit { retry_after_secs } => {
                Some(std::time::Duration::from_secs(*retry_after_secs))
            }
            _ => None,
        }
    }
}

/// Результат операции с биржей
pub type ExchangeResult<T> = Result<T, ExchangeError>;
```

## Шаг 2: Модели данных с валидацией

```rust
// models.rs
use crate::error::{ExchangeError, ExchangeResult};

/// Сторона ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Статус ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Валидированная цена (всегда положительная)
#[derive(Debug, Clone, Copy)]
pub struct Price(f64);

impl Price {
    pub fn new(value: f64) -> ExchangeResult<Self> {
        if value <= 0.0 {
            return Err(ExchangeError::Validation(
                format!("Price must be positive, got {}", value)
            ));
        }
        if !value.is_finite() {
            return Err(ExchangeError::Validation(
                "Price must be a finite number".to_string()
            ));
        }
        Ok(Price(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Валидированное количество (всегда положительное)
#[derive(Debug, Clone, Copy)]
pub struct Quantity(f64);

impl Quantity {
    pub fn new(value: f64) -> ExchangeResult<Self> {
        if value <= 0.0 {
            return Err(ExchangeError::Validation(
                format!("Quantity must be positive, got {}", value)
            ));
        }
        if !value.is_finite() {
            return Err(ExchangeError::Validation(
                "Quantity must be a finite number".to_string()
            ));
        }
        Ok(Quantity(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Торговый символ (пара, например BTC/USDT)
#[derive(Debug, Clone)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(value: &str) -> ExchangeResult<Self> {
        let value = value.trim().to_uppercase();

        if value.is_empty() {
            return Err(ExchangeError::Validation(
                "Symbol cannot be empty".to_string()
            ));
        }

        if value.len() > 20 {
            return Err(ExchangeError::Validation(
                "Symbol too long (max 20 characters)".to_string()
            ));
        }

        // Проверяем формат: должны быть буквы и /
        if !value.chars().all(|c| c.is_ascii_alphabetic() || c == '/') {
            return Err(ExchangeError::Validation(
                format!("Invalid symbol format: {}", value)
            ));
        }

        Ok(Symbol(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Запрос на создание ордера
#[derive(Debug)]
pub struct OrderRequest {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,  // None для рыночных ордеров
}

impl OrderRequest {
    /// Builder для создания ордера с валидацией
    pub fn builder() -> OrderRequestBuilder {
        OrderRequestBuilder::default()
    }
}

/// Builder для OrderRequest
#[derive(Default)]
pub struct OrderRequestBuilder {
    symbol: Option<Symbol>,
    side: Option<OrderSide>,
    order_type: Option<OrderType>,
    quantity: Option<Quantity>,
    price: Option<Price>,
}

impl OrderRequestBuilder {
    pub fn symbol(mut self, symbol: &str) -> ExchangeResult<Self> {
        self.symbol = Some(Symbol::new(symbol)?);
        Ok(self)
    }

    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    pub fn quantity(mut self, quantity: f64) -> ExchangeResult<Self> {
        self.quantity = Some(Quantity::new(quantity)?);
        Ok(self)
    }

    pub fn price(mut self, price: f64) -> ExchangeResult<Self> {
        self.price = Some(Price::new(price)?);
        Ok(self)
    }

    pub fn build(self) -> ExchangeResult<OrderRequest> {
        let symbol = self.symbol.ok_or_else(|| {
            ExchangeError::Validation("Symbol is required".to_string())
        })?;

        let side = self.side.ok_or_else(|| {
            ExchangeError::Validation("Side is required".to_string())
        })?;

        let order_type = self.order_type.ok_or_else(|| {
            ExchangeError::Validation("Order type is required".to_string())
        })?;

        let quantity = self.quantity.ok_or_else(|| {
            ExchangeError::Validation("Quantity is required".to_string())
        })?;

        // Лимитный ордер требует цены
        if order_type == OrderType::Limit && self.price.is_none() {
            return Err(ExchangeError::Validation(
                "Limit order requires a price".to_string()
            ));
        }

        Ok(OrderRequest {
            symbol,
            side,
            order_type,
            quantity,
            price: self.price,
        })
    }
}

/// Ответ с информацией об ордере
#[derive(Debug, Clone)]
pub struct OrderResponse {
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub price: Option<f64>,
    pub average_price: Option<f64>,
    pub status: OrderStatus,
    pub created_at: u64,  // Unix timestamp
}

/// Информация о балансе
#[derive(Debug, Clone)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

impl Balance {
    pub fn total(&self) -> f64 {
        self.free + self.locked
    }
}

/// Текущая цена тикера
#[derive(Debug, Clone)]
pub struct Ticker {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume_24h: f64,
    pub timestamp: u64,
}

impl Ticker {
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    pub fn spread_percent(&self) -> f64 {
        if self.bid > 0.0 {
            (self.spread() / self.bid) * 100.0
        } else {
            0.0
        }
    }
}
```

## Шаг 3: Retry логика с exponential backoff

```rust
// retry.rs
use std::time::Duration;
use crate::error::{ExchangeError, ExchangeResult};

/// Конфигурация retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Максимальное количество попыток
    pub max_attempts: u32,
    /// Начальная задержка
    pub initial_delay: Duration,
    /// Максимальная задержка
    pub max_delay: Duration,
    /// Множитель задержки (обычно 2.0 для exponential backoff)
    pub multiplier: f64,
    /// Добавлять случайный jitter для избежания thundering herd
    pub add_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            add_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Создать конфигурацию для агрессивного retry
    pub fn aggressive() -> Self {
        RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            add_jitter: true,
        }
    }

    /// Создать конфигурацию для консервативного retry
    pub fn conservative() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 3.0,
            add_jitter: true,
        }
    }

    /// Вычислить задержку для попытки
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let delay_ms = self.initial_delay.as_millis() as f64
            * self.multiplier.powi(attempt as i32 - 1);

        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);

        let delay_ms = if self.add_jitter {
            // Добавляем до 25% случайного jitter
            let jitter = delay_ms * 0.25 * rand_simple();
            delay_ms + jitter
        } else {
            delay_ms
        };

        Duration::from_millis(delay_ms as u64)
    }
}

/// Простой генератор случайных чисел (0.0 - 1.0)
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Информация о попытке выполнения
#[derive(Debug)]
pub struct AttemptInfo {
    pub attempt: u32,
    pub max_attempts: u32,
    pub last_error: Option<ExchangeError>,
}

/// Выполнить операцию с retry
pub fn with_retry<T, F>(config: &RetryConfig, mut operation: F) -> ExchangeResult<T>
where
    F: FnMut() -> ExchangeResult<T>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        // Ждём перед попыткой (кроме первой)
        if attempt > 0 {
            let delay = config.delay_for_attempt(attempt);
            println!(
                "  Retry attempt {}/{} after {:?}",
                attempt + 1,
                config.max_attempts,
                delay
            );
            std::thread::sleep(delay);
        }

        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Если ошибка предоставляет конкретное время ожидания, используем его
                if let Some(retry_after) = e.retry_after() {
                    println!("  Rate limited, waiting {:?}", retry_after);
                    std::thread::sleep(retry_after);
                }

                // Проверяем, можно ли повторить
                if !e.is_retryable() {
                    return Err(e);
                }

                println!("  Attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ExchangeError::Unknown("Retry exhausted without error".to_string())
    }))
}

/// Выполнить операцию с retry и callback для каждой попытки
pub fn with_retry_callback<T, F, C>(
    config: &RetryConfig,
    mut operation: F,
    mut on_attempt: C,
) -> ExchangeResult<T>
where
    F: FnMut() -> ExchangeResult<T>,
    C: FnMut(&AttemptInfo),
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        let info = AttemptInfo {
            attempt: attempt + 1,
            max_attempts: config.max_attempts,
            last_error: last_error.take(),
        };

        on_attempt(&info);

        if attempt > 0 {
            let delay = config.delay_for_attempt(attempt);
            std::thread::sleep(delay);
        }

        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if let Some(retry_after) = e.retry_after() {
                    std::thread::sleep(retry_after);
                }

                if !e.is_retryable() {
                    return Err(e);
                }

                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ExchangeError::Unknown("Retry exhausted".to_string())
    }))
}
```

## Шаг 4: Circuit Breaker

```rust
// circuit_breaker.rs
use std::time::{Duration, Instant};
use crate::error::{ExchangeError, ExchangeResult};

/// Состояние circuit breaker
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Закрыт — запросы проходят нормально
    Closed,
    /// Открыт — все запросы отклоняются
    Open,
    /// Полуоткрыт — пропускаем пробные запросы
    HalfOpen,
}

/// Конфигурация circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Порог неудачных запросов для открытия
    pub failure_threshold: u32,
    /// Время, через которое перейдём в HalfOpen
    pub reset_timeout: Duration,
    /// Количество успешных запросов для закрытия
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        CircuitBreakerConfig {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            success_threshold: 3,
        }
    }
}

/// Circuit Breaker для защиты от каскадных сбоев
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
        }
    }

    /// Получить текущее состояние
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Проверить, можно ли выполнить запрос
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Проверяем, прошло ли достаточно времени
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        println!("Circuit breaker: Open -> HalfOpen");
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Сообщить об успешном запросе
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Сбрасываем счётчик неудач
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    println!("Circuit breaker: HalfOpen -> Closed");
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Не должно происходить
            }
        }
    }

    /// Сообщить о неудачном запросе
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    println!(
                        "Circuit breaker: Closed -> Open (failures: {})",
                        self.failure_count
                    );
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                println!("Circuit breaker: HalfOpen -> Open");
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {
                self.last_failure_time = Some(Instant::now());
            }
        }
    }

    /// Выполнить операцию с protection circuit breaker
    pub fn execute<T, F>(&mut self, operation: F) -> ExchangeResult<T>
    where
        F: FnOnce() -> ExchangeResult<T>,
    {
        if !self.can_execute() {
            return Err(ExchangeError::CircuitBreakerOpen);
        }

        match operation() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                // Только retryable ошибки влияют на circuit breaker
                if e.is_retryable() {
                    self.record_failure();
                }
                Err(e)
            }
        }
    }

    /// Получить статистику
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state,
            failure_count: self.failure_count,
            success_count: self.success_count,
            time_until_reset: self.time_until_reset(),
        }
    }

    fn time_until_reset(&self) -> Option<Duration> {
        if self.state != CircuitState::Open {
            return None;
        }

        self.last_failure_time.map(|t| {
            let elapsed = t.elapsed();
            if elapsed >= self.config.reset_timeout {
                Duration::ZERO
            } else {
                self.config.reset_timeout - elapsed
            }
        })
    }
}

/// Статистика circuit breaker
#[derive(Debug)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub time_until_reset: Option<Duration>,
}
```

## Шаг 5: Главный API клиент

```rust
// client.rs
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::error::{ExchangeError, ExchangeResult};
use crate::models::*;
use crate::retry::{with_retry, RetryConfig};

/// Конфигурация API клиента
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
    pub timeout_ms: u64,
    pub retry_config: RetryConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            api_key: String::new(),
            api_secret: String::new(),
            base_url: "https://api.exchange.example".to_string(),
            timeout_ms: 5000,
            retry_config: RetryConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }
}

/// Робастный API клиент биржи
pub struct ExchangeClient {
    config: ClientConfig,
    circuit_breaker: CircuitBreaker,
    // Симуляция состояния для демонстрации
    simulated_balance: f64,
    request_count: u32,
}

impl ExchangeClient {
    /// Создать новый клиент
    pub fn new(config: ClientConfig) -> ExchangeResult<Self> {
        // Валидируем конфигурацию
        if config.api_key.is_empty() {
            return Err(ExchangeError::Validation(
                "API key is required".to_string()
            ));
        }
        if config.api_secret.is_empty() {
            return Err(ExchangeError::Validation(
                "API secret is required".to_string()
            ));
        }

        Ok(ExchangeClient {
            circuit_breaker: CircuitBreaker::new(config.circuit_breaker_config.clone()),
            config,
            simulated_balance: 10000.0,
            request_count: 0,
        })
    }

    /// Получить текущую цену
    pub fn get_ticker(&mut self, symbol: &str) -> ExchangeResult<Ticker> {
        // Валидируем символ
        let symbol = Symbol::new(symbol)?;

        let retry_config = self.config.retry_config.clone();

        // Выполняем с retry и circuit breaker
        self.execute_with_protection(&retry_config, || {
            self.simulate_ticker_request(symbol.as_str())
        })
    }

    /// Получить баланс
    pub fn get_balance(&mut self, asset: &str) -> ExchangeResult<Balance> {
        let retry_config = self.config.retry_config.clone();
        let asset = asset.to_uppercase();

        self.execute_with_protection(&retry_config, || {
            self.simulate_balance_request(&asset)
        })
    }

    /// Создать ордер
    pub fn create_order(&mut self, request: &OrderRequest) -> ExchangeResult<OrderResponse> {
        // Сначала проверяем баланс
        let balance = self.get_balance("USDT")?;

        let required = match request.order_type {
            OrderType::Market => request.quantity.value() * 45000.0, // Примерная цена BTC
            OrderType::Limit => {
                request.quantity.value() * request.price
                    .map(|p| p.value())
                    .unwrap_or(45000.0)
            }
            _ => request.quantity.value() * 45000.0,
        };

        if balance.free < required && request.side == OrderSide::Buy {
            return Err(ExchangeError::InsufficientFunds {
                required,
                available: balance.free,
            });
        }

        let retry_config = self.config.retry_config.clone();

        self.execute_with_protection(&retry_config, || {
            self.simulate_order_request(request)
        })
    }

    /// Отменить ордер
    pub fn cancel_order(&mut self, symbol: &str, order_id: &str) -> ExchangeResult<OrderResponse> {
        let symbol = Symbol::new(symbol)?;

        if order_id.is_empty() {
            return Err(ExchangeError::Validation(
                "Order ID is required".to_string()
            ));
        }

        let retry_config = self.config.retry_config.clone();
        let order_id = order_id.to_string();
        let symbol_str = symbol.as_str().to_string();

        self.execute_with_protection(&retry_config, || {
            self.simulate_cancel_request(&symbol_str, &order_id)
        })
    }

    /// Получить статус ордера
    pub fn get_order(&mut self, symbol: &str, order_id: &str) -> ExchangeResult<OrderResponse> {
        let symbol = Symbol::new(symbol)?;

        if order_id.is_empty() {
            return Err(ExchangeError::Validation(
                "Order ID is required".to_string()
            ));
        }

        let retry_config = self.config.retry_config.clone();
        let order_id = order_id.to_string();
        let symbol_str = symbol.as_str().to_string();

        self.execute_with_protection(&retry_config, || {
            self.simulate_get_order_request(&symbol_str, &order_id)
        })
    }

    /// Получить статистику circuit breaker
    pub fn circuit_breaker_stats(&self) -> crate::circuit_breaker::CircuitBreakerStats {
        // Создаём копию для получения статистики
        CircuitBreaker::new(self.config.circuit_breaker_config.clone()).stats()
    }

    // === Вспомогательные методы ===

    fn execute_with_protection<T, F>(
        &mut self,
        retry_config: &RetryConfig,
        operation: F,
    ) -> ExchangeResult<T>
    where
        F: Fn() -> ExchangeResult<T>,
    {
        // Сначала проверяем circuit breaker
        self.circuit_breaker.execute(|| {
            // Затем применяем retry
            with_retry(retry_config, || operation())
        })
    }

    // === Симуляция запросов для демонстрации ===

    fn simulate_ticker_request(&mut self, symbol: &str) -> ExchangeResult<Ticker> {
        self.request_count += 1;

        // Симулируем случайные ошибки для демонстрации
        if self.request_count % 7 == 0 {
            return Err(ExchangeError::Network {
                message: "Connection timeout".to_string(),
                source: None,
                is_retryable: true,
            });
        }

        if self.request_count % 13 == 0 {
            return Err(ExchangeError::RateLimit {
                retry_after_secs: 1,
            });
        }

        Ok(Ticker {
            symbol: symbol.to_string(),
            bid: 44950.0,
            ask: 45050.0,
            last: 45000.0,
            volume_24h: 1234567.89,
            timestamp: 1699000000,
        })
    }

    fn simulate_balance_request(&self, asset: &str) -> ExchangeResult<Balance> {
        Ok(Balance {
            asset: asset.to_string(),
            free: self.simulated_balance,
            locked: 1000.0,
        })
    }

    fn simulate_order_request(&mut self, request: &OrderRequest) -> ExchangeResult<OrderResponse> {
        self.request_count += 1;

        // Симулируем сетевую ошибку
        if self.request_count % 5 == 0 {
            return Err(ExchangeError::Network {
                message: "Request failed".to_string(),
                source: None,
                is_retryable: true,
            });
        }

        // Обновляем симулированный баланс
        let order_value = request.quantity.value()
            * request.price.map(|p| p.value()).unwrap_or(45000.0);

        if request.side == OrderSide::Buy {
            self.simulated_balance -= order_value;
        } else {
            self.simulated_balance += order_value;
        }

        Ok(OrderResponse {
            order_id: format!("ORD-{}", self.request_count),
            symbol: request.symbol.as_str().to_string(),
            side: request.side,
            order_type: request.order_type,
            quantity: request.quantity.value(),
            filled_quantity: request.quantity.value(),
            price: request.price.map(|p| p.value()),
            average_price: Some(45000.0),
            status: OrderStatus::Filled,
            created_at: 1699000000,
        })
    }

    fn simulate_cancel_request(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> ExchangeResult<OrderResponse> {
        // 50% шанс, что ордер не найден
        if order_id.contains("999") {
            return Err(ExchangeError::OrderNotFound {
                order_id: order_id.to_string(),
            });
        }

        Ok(OrderResponse {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.1,
            filled_quantity: 0.0,
            price: Some(44000.0),
            average_price: None,
            status: OrderStatus::Cancelled,
            created_at: 1699000000,
        })
    }

    fn simulate_get_order_request(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> ExchangeResult<OrderResponse> {
        if order_id.contains("999") {
            return Err(ExchangeError::OrderNotFound {
                order_id: order_id.to_string(),
            });
        }

        Ok(OrderResponse {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.1,
            filled_quantity: 0.1,
            price: Some(45000.0),
            average_price: Some(44950.0),
            status: OrderStatus::Filled,
            created_at: 1699000000,
        })
    }
}
```

## Шаг 6: Главная программа

```rust
// main.rs
mod client;
mod circuit_breaker;
mod error;
mod models;
mod retry;

use client::{ClientConfig, ExchangeClient};
use error::ExchangeResult;
use models::{OrderRequest, OrderSide, OrderType};
use retry::RetryConfig;

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║   Робастный API Клиент Биржи                 ║");
    println!("║   Robust Exchange API Client                 ║");
    println!("╚══════════════════════════════════════════════╝\n");

    if let Err(e) = run_demo() {
        eprintln!("\n❌ Fatal error: {}", e);

        // Печатаем цепочку ошибок
        let mut source = std::error::Error::source(&e);
        while let Some(s) = source {
            eprintln!("   Caused by: {}", s);
            source = std::error::Error::source(s);
        }
    }
}

fn run_demo() -> ExchangeResult<()> {
    // Создаём конфигурацию
    let config = ClientConfig {
        api_key: "demo_key_12345".to_string(),
        api_secret: "demo_secret_67890".to_string(),
        base_url: "https://api.demo-exchange.com".to_string(),
        timeout_ms: 5000,
        retry_config: RetryConfig {
            max_attempts: 3,
            ..RetryConfig::default()
        },
        ..ClientConfig::default()
    };

    // Создаём клиент
    let mut client = ExchangeClient::new(config)?;
    println!("✓ Client initialized\n");

    // === Демонстрация 1: Получение цены ===
    println!("📊 Demo 1: Fetching ticker price");
    println!("─────────────────────────────────");

    match client.get_ticker("BTC/USDT") {
        Ok(ticker) => {
            println!("  Symbol: {}", ticker.symbol);
            println!("  Bid: ${:.2}", ticker.bid);
            println!("  Ask: ${:.2}", ticker.ask);
            println!("  Last: ${:.2}", ticker.last);
            println!("  Spread: {:.4}%", ticker.spread_percent());
        }
        Err(e) => {
            println!("  ⚠ Error: {}", e);
        }
    }
    println!();

    // === Демонстрация 2: Получение баланса ===
    println!("💰 Demo 2: Checking balance");
    println!("─────────────────────────────────");

    match client.get_balance("USDT") {
        Ok(balance) => {
            println!("  Asset: {}", balance.asset);
            println!("  Free: ${:.2}", balance.free);
            println!("  Locked: ${:.2}", balance.locked);
            println!("  Total: ${:.2}", balance.total());
        }
        Err(e) => {
            println!("  ⚠ Error: {}", e);
        }
    }
    println!();

    // === Демонстрация 3: Создание ордера с валидацией ===
    println!("📝 Demo 3: Creating order with validation");
    println!("─────────────────────────────────────────");

    // Пробуем создать ордер с невалидными данными
    println!("  Attempting invalid order (negative quantity)...");
    match OrderRequest::builder()
        .symbol("BTC/USDT")?
        .side(OrderSide::Buy)
        .order_type(OrderType::Limit)
        .quantity(-0.1)  // Невалидное количество
    {
        Ok(_) => println!("  Unexpected: order created"),
        Err(e) => println!("  ✓ Validation caught error: {}", e),
    }

    // Создаём валидный ордер
    println!("\n  Creating valid order...");
    let order = OrderRequest::builder()
        .symbol("BTC/USDT")?
        .side(OrderSide::Buy)
        .order_type(OrderType::Limit)
        .quantity(0.01)?
        .price(44000.0)?
        .build()?;

    match client.create_order(&order) {
        Ok(response) => {
            println!("  ✓ Order created!");
            println!("    Order ID: {}", response.order_id);
            println!("    Status: {:?}", response.status);
            println!("    Filled: {} / {}", response.filled_quantity, response.quantity);
        }
        Err(e) => {
            println!("  ⚠ Error: {}", e);
        }
    }
    println!();

    // === Демонстрация 4: Обработка ошибки недостаточного баланса ===
    println!("💸 Demo 4: Handling insufficient funds");
    println!("─────────────────────────────────────────");

    let big_order = OrderRequest::builder()
        .symbol("BTC/USDT")?
        .side(OrderSide::Buy)
        .order_type(OrderType::Limit)
        .quantity(1000.0)?  // Слишком большой ордер
        .price(45000.0)?
        .build()?;

    match client.create_order(&big_order) {
        Ok(_) => println!("  Unexpected: order created"),
        Err(e) => {
            println!("  ✓ Error handled: {}", e);
            println!("    Is retryable: {}", e.is_retryable());
        }
    }
    println!();

    // === Демонстрация 5: Множественные запросы (retry и circuit breaker) ===
    println!("🔄 Demo 5: Multiple requests (testing retry & circuit breaker)");
    println!("─────────────────────────────────────────────────────────────");

    for i in 1..=10 {
        print!("  Request {}: ", i);
        match client.get_ticker("ETH/USDT") {
            Ok(ticker) => println!("✓ ${:.2}", ticker.last),
            Err(e) => println!("✗ {}", e),
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!();

    // === Демонстрация 6: Отмена ордера ===
    println!("❌ Demo 6: Cancelling orders");
    println!("─────────────────────────────────");

    println!("  Cancelling existing order...");
    match client.cancel_order("BTC/USDT", "ORD-123") {
        Ok(response) => {
            println!("  ✓ Order cancelled");
            println!("    Status: {:?}", response.status);
        }
        Err(e) => {
            println!("  ⚠ Error: {}", e);
        }
    }

    println!("\n  Cancelling non-existent order...");
    match client.cancel_order("BTC/USDT", "ORD-999") {
        Ok(_) => println!("  Unexpected: order cancelled"),
        Err(e) => {
            println!("  ✓ Error handled: {}", e);
        }
    }
    println!();

    // === Финальная статистика ===
    println!("📈 Final Statistics");
    println!("───────────────────");

    if let Ok(balance) = client.get_balance("USDT") {
        println!("  Final balance: ${:.2}", balance.free);
    }

    println!("\n✓ Demo completed successfully!");

    Ok(())
}
```

## Что мы узнали

| Паттерн | Применение | Польза |
|---------|------------|--------|
| Custom Error Types | `ExchangeError` enum | Типизированные ошибки с контекстом |
| Newtype Validation | `Price`, `Quantity`, `Symbol` | Невалидные данные не попадут в систему |
| Builder Pattern | `OrderRequestBuilder` | Пошаговое создание сложных объектов |
| Retry with Backoff | `with_retry()` | Автоматическое восстановление |
| Circuit Breaker | `CircuitBreaker` | Защита от каскадных сбоев |
| Error Classification | `is_retryable()` | Умное решение о повторе |

## Паттерны из месяца Error Handling

1. **thiserror** — создаём типизированные ошибки
2. **Result везде** — ошибка как значение, не исключение
3. **Оператор ?** — пробрасываем ошибки вверх
4. **Валидация на границе** — проверяем данные при входе
5. **Retry с ограничением** — не повторяем бесконечно
6. **Circuit Breaker** — защита от каскадных сбоев
7. **Graceful Degradation** — работаем с частичными данными

## Домашнее задание

1. **Добавь WebSocket поддержку** — создай отдельный модуль для стриминга цен с reconnect логикой

2. **Реализуй Rate Limiter** — ограничь количество запросов в секунду с очередью ожидания

3. **Добавь кеширование** — кешируй последние цены для уменьшения запросов

4. **Расширь Circuit Breaker** — добавь метрики (количество успехов/неудач, среднее время ответа)

5. **Напиши тесты** — покрой каждый модуль unit-тестами, используя моки для сетевых ошибок

## Навигация

[← Предыдущий день](../119-error-as-value/ru.md) | [Следующий день →](../121-reading-file-price-history/ru.md)
