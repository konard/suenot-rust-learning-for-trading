# Day 120: Project — Robust Exchange API Client

## Trading Analogy

Imagine you're building a trading bot that works with an exchange 24/7. The exchange might be temporarily unavailable, the API might return errors, the network might drop in the middle of a trade. A professional trader doesn't panic — they have a plan for every scenario. Our API client should be the same: **reliable, predictable, and recoverable from errors**.

In this project, we combine all the month's knowledge about error handling:
- Custom error types (`thiserror`)
- Error context (`anyhow`)
- Retry logic with exponential backoff
- Circuit breaker for cascade failure protection
- Graceful degradation
- Input validation

## Project Architecture

```
exchange_client/
├── src/
│   ├── main.rs           # Entry point
│   ├── client.rs         # API client
│   ├── error.rs          # Error types
│   ├── retry.rs          # Retry logic
│   ├── circuit_breaker.rs # Circuit breaker
│   ├── models.rs         # Data models
│   └── validation.rs     # Validation
└── Cargo.toml
```

## Step 1: Defining Error Types

```rust
// error.rs
use thiserror::Error;

/// Exchange API client errors
#[derive(Error, Debug)]
pub enum ExchangeError {
    /// Network error — request can be retried
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        is_retryable: bool,
    },

    /// Authentication error — need to refresh keys
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Validation error — incorrect input data
    #[error("Validation error: {0}")]
    Validation(String),

    /// Rate limit error — too many requests
    #[error("Rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimit { retry_after_secs: u64 },

    /// Exchange error — server-side problem
    #[error("Exchange error ({code}): {message}")]
    Exchange { code: i32, message: String },

    /// Insufficient funds
    #[error("Insufficient funds: need {required}, have {available}")]
    InsufficientFunds { required: f64, available: f64 },

    /// Order not found
    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    /// Circuit breaker is open
    #[error("Circuit breaker is open, service temporarily unavailable")]
    CircuitBreakerOpen,

    /// Operation timeout
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ExchangeError {
    /// Can the request be retried after this error?
    pub fn is_retryable(&self) -> bool {
        match self {
            ExchangeError::Network { is_retryable, .. } => *is_retryable,
            ExchangeError::RateLimit { .. } => true,
            ExchangeError::Timeout { .. } => true,
            ExchangeError::Exchange { code, .. } => {
                // 5xx server errors are usually temporary
                *code >= 500 && *code < 600
            }
            _ => false,
        }
    }

    /// Get wait time before retry (if applicable)
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            ExchangeError::RateLimit { retry_after_secs } => {
                Some(std::time::Duration::from_secs(*retry_after_secs))
            }
            _ => None,
        }
    }
}

/// Result of exchange operation
pub type ExchangeResult<T> = Result<T, ExchangeError>;
```

## Step 2: Data Models with Validation

```rust
// models.rs
use crate::error::{ExchangeError, ExchangeResult};

/// Order side
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Validated price (always positive)
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

/// Validated quantity (always positive)
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

/// Trading symbol (pair, e.g., BTC/USDT)
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

        // Check format: must be letters and /
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

/// Order creation request
#[derive(Debug)]
pub struct OrderRequest {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,  // None for market orders
}

impl OrderRequest {
    /// Builder for creating orders with validation
    pub fn builder() -> OrderRequestBuilder {
        OrderRequestBuilder::default()
    }
}

/// Builder for OrderRequest
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

        // Limit orders require a price
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

/// Order response information
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

/// Balance information
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

/// Current ticker price
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

## Step 3: Retry Logic with Exponential Backoff

```rust
// retry.rs
use std::time::Duration;
use crate::error::{ExchangeError, ExchangeResult};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts
    pub max_attempts: u32,
    /// Initial delay
    pub initial_delay: Duration,
    /// Maximum delay
    pub max_delay: Duration,
    /// Delay multiplier (usually 2.0 for exponential backoff)
    pub multiplier: f64,
    /// Add random jitter to avoid thundering herd
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
    /// Create aggressive retry configuration
    pub fn aggressive() -> Self {
        RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            add_jitter: true,
        }
    }

    /// Create conservative retry configuration
    pub fn conservative() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 3.0,
            add_jitter: true,
        }
    }

    /// Calculate delay for attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let delay_ms = self.initial_delay.as_millis() as f64
            * self.multiplier.powi(attempt as i32 - 1);

        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);

        let delay_ms = if self.add_jitter {
            // Add up to 25% random jitter
            let jitter = delay_ms * 0.25 * rand_simple();
            delay_ms + jitter
        } else {
            delay_ms
        };

        Duration::from_millis(delay_ms as u64)
    }
}

/// Simple random number generator (0.0 - 1.0)
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Attempt execution information
#[derive(Debug)]
pub struct AttemptInfo {
    pub attempt: u32,
    pub max_attempts: u32,
    pub last_error: Option<ExchangeError>,
}

/// Execute operation with retry
pub fn with_retry<T, F>(config: &RetryConfig, mut operation: F) -> ExchangeResult<T>
where
    F: FnMut() -> ExchangeResult<T>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        // Wait before attempt (except first)
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
                // If error provides specific wait time, use it
                if let Some(retry_after) = e.retry_after() {
                    println!("  Rate limited, waiting {:?}", retry_after);
                    std::thread::sleep(retry_after);
                }

                // Check if retryable
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

/// Execute operation with retry and callback for each attempt
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

## Step 4: Circuit Breaker

```rust
// circuit_breaker.rs
use std::time::{Duration, Instant};
use crate::error::{ExchangeError, ExchangeResult};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Closed — requests pass normally
    Closed,
    /// Open — all requests are rejected
    Open,
    /// Half-open — allowing probe requests
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Time before transitioning to HalfOpen
    pub reset_timeout: Duration,
    /// Success threshold to close circuit
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

/// Circuit Breaker for cascade failure protection
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

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Check if request can be executed
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if enough time has passed
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

    /// Record successful request
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure counter
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
                // Should not happen
            }
        }
    }

    /// Record failed request
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

    /// Execute operation with circuit breaker protection
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
                // Only retryable errors affect circuit breaker
                if e.is_retryable() {
                    self.record_failure();
                }
                Err(e)
            }
        }
    }

    /// Get statistics
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

/// Circuit breaker statistics
#[derive(Debug)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub time_until_reset: Option<Duration>,
}
```

## Step 5: Main API Client

```rust
// client.rs
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::error::{ExchangeError, ExchangeResult};
use crate::models::*;
use crate::retry::{with_retry, RetryConfig};

/// API client configuration
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

/// Robust exchange API client
pub struct ExchangeClient {
    config: ClientConfig,
    circuit_breaker: CircuitBreaker,
    // Simulation state for demonstration
    simulated_balance: f64,
    request_count: u32,
}

impl ExchangeClient {
    /// Create new client
    pub fn new(config: ClientConfig) -> ExchangeResult<Self> {
        // Validate configuration
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

    /// Get current price
    pub fn get_ticker(&mut self, symbol: &str) -> ExchangeResult<Ticker> {
        // Validate symbol
        let symbol = Symbol::new(symbol)?;

        let retry_config = self.config.retry_config.clone();

        // Execute with retry and circuit breaker
        self.execute_with_protection(&retry_config, || {
            self.simulate_ticker_request(symbol.as_str())
        })
    }

    /// Get balance
    pub fn get_balance(&mut self, asset: &str) -> ExchangeResult<Balance> {
        let retry_config = self.config.retry_config.clone();
        let asset = asset.to_uppercase();

        self.execute_with_protection(&retry_config, || {
            self.simulate_balance_request(&asset)
        })
    }

    /// Create order
    pub fn create_order(&mut self, request: &OrderRequest) -> ExchangeResult<OrderResponse> {
        // First check balance
        let balance = self.get_balance("USDT")?;

        let required = match request.order_type {
            OrderType::Market => request.quantity.value() * 45000.0, // Approximate BTC price
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

    /// Cancel order
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

    /// Get order status
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

    /// Get circuit breaker statistics
    pub fn circuit_breaker_stats(&self) -> crate::circuit_breaker::CircuitBreakerStats {
        // Create copy for statistics
        CircuitBreaker::new(self.config.circuit_breaker_config.clone()).stats()
    }

    // === Helper methods ===

    fn execute_with_protection<T, F>(
        &mut self,
        retry_config: &RetryConfig,
        operation: F,
    ) -> ExchangeResult<T>
    where
        F: Fn() -> ExchangeResult<T>,
    {
        // First check circuit breaker
        self.circuit_breaker.execute(|| {
            // Then apply retry
            with_retry(retry_config, || operation())
        })
    }

    // === Request simulation for demonstration ===

    fn simulate_ticker_request(&mut self, symbol: &str) -> ExchangeResult<Ticker> {
        self.request_count += 1;

        // Simulate random errors for demonstration
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

        // Simulate network error
        if self.request_count % 5 == 0 {
            return Err(ExchangeError::Network {
                message: "Request failed".to_string(),
                source: None,
                is_retryable: true,
            });
        }

        // Update simulated balance
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
        // 50% chance order not found
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

## Step 6: Main Program

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
    println!("║   Robust Exchange API Client                 ║");
    println!("╚══════════════════════════════════════════════╝\n");

    if let Err(e) = run_demo() {
        eprintln!("\n❌ Fatal error: {}", e);

        // Print error chain
        let mut source = std::error::Error::source(&e);
        while let Some(s) = source {
            eprintln!("   Caused by: {}", s);
            source = std::error::Error::source(s);
        }
    }
}

fn run_demo() -> ExchangeResult<()> {
    // Create configuration
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

    // Create client
    let mut client = ExchangeClient::new(config)?;
    println!("✓ Client initialized\n");

    // === Demo 1: Fetching price ===
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

    // === Demo 2: Checking balance ===
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

    // === Demo 3: Creating order with validation ===
    println!("📝 Demo 3: Creating order with validation");
    println!("─────────────────────────────────────────");

    // Try creating order with invalid data
    println!("  Attempting invalid order (negative quantity)...");
    match OrderRequest::builder()
        .symbol("BTC/USDT")?
        .side(OrderSide::Buy)
        .order_type(OrderType::Limit)
        .quantity(-0.1)  // Invalid quantity
    {
        Ok(_) => println!("  Unexpected: order created"),
        Err(e) => println!("  ✓ Validation caught error: {}", e),
    }

    // Create valid order
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

    // === Demo 4: Handling insufficient funds ===
    println!("💸 Demo 4: Handling insufficient funds");
    println!("─────────────────────────────────────────");

    let big_order = OrderRequest::builder()
        .symbol("BTC/USDT")?
        .side(OrderSide::Buy)
        .order_type(OrderType::Limit)
        .quantity(1000.0)?  // Too large order
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

    // === Demo 5: Multiple requests (retry & circuit breaker) ===
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

    // === Demo 6: Cancelling orders ===
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

    // === Final statistics ===
    println!("📈 Final Statistics");
    println!("───────────────────");

    if let Ok(balance) = client.get_balance("USDT") {
        println!("  Final balance: ${:.2}", balance.free);
    }

    println!("\n✓ Demo completed successfully!");

    Ok(())
}
```

## What We Learned

| Pattern | Application | Benefit |
|---------|-------------|---------|
| Custom Error Types | `ExchangeError` enum | Typed errors with context |
| Newtype Validation | `Price`, `Quantity`, `Symbol` | Invalid data can't enter the system |
| Builder Pattern | `OrderRequestBuilder` | Step-by-step complex object creation |
| Retry with Backoff | `with_retry()` | Automatic recovery |
| Circuit Breaker | `CircuitBreaker` | Cascade failure protection |
| Error Classification | `is_retryable()` | Smart retry decisions |

## Error Handling Patterns from This Month

1. **thiserror** — creating typed errors
2. **Result everywhere** — error as value, not exception
3. **? operator** — propagating errors up
4. **Boundary validation** — checking data at entry points
5. **Retry with limits** — not retrying forever
6. **Circuit Breaker** — cascade failure protection
7. **Graceful Degradation** — working with partial data

## Homework

1. **Add WebSocket support** — create a separate module for price streaming with reconnect logic

2. **Implement Rate Limiter** — limit requests per second with a waiting queue

3. **Add caching** — cache recent prices to reduce API calls

4. **Extend Circuit Breaker** — add metrics (success/failure counts, average response time)

5. **Write tests** — cover each module with unit tests, using mocks for network errors

## Navigation

[← Previous day](../119-error-as-value/en.md) | [Next day →](../121-reading-file-price-history/en.md)
