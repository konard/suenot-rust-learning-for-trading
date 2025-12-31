# Day 108: Retry Logic — Repeating Failed Requests

## Trading Analogy

When working with exchange APIs, temporary failures are common:
- Exchange server overload
- Network timeouts
- Rate limiting
- Temporary service unavailability

An experienced trader doesn't give up after the first failure — they retry with a reasonable delay. This is **retry logic**.

## Basic Concept: Simple Retry

```rust
fn main() {
    match fetch_price_with_retry("BTC", 3) {
        Ok(price) => println!("BTC price: ${:.2}", price),
        Err(e) => println!("Failed after all retries: {}", e),
    }
}

fn fetch_price_with_retry(symbol: &str, max_retries: u32) -> Result<f64, String> {
    let mut attempts = 0;

    loop {
        attempts += 1;
        println!("Attempt {} for {}", attempts, symbol);

        match fetch_price(symbol) {
            Ok(price) => return Ok(price),
            Err(e) => {
                if attempts >= max_retries {
                    return Err(format!("Failed after {} attempts: {}", attempts, e));
                }
                println!("  Retry needed: {}", e);
            }
        }
    }
}

// Simulated API call (70% success rate)
fn fetch_price(symbol: &str) -> Result<f64, String> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    if random < 70 {
        Ok(42500.0 + (random as f64))
    } else {
        Err("Connection timeout".to_string())
    }
}
```

## Exponential Backoff

Best practice is to increase delay between attempts:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    match fetch_with_exponential_backoff("ETH", 5) {
        Ok(price) => println!("ETH price: ${:.2}", price),
        Err(e) => println!("Error: {}", e),
    }
}

fn fetch_with_exponential_backoff(symbol: &str, max_retries: u32) -> Result<f64, String> {
    let base_delay_ms = 100; // Initial delay

    for attempt in 0..max_retries {
        println!("Attempt {} for {}", attempt + 1, symbol);

        match fetch_price(symbol) {
            Ok(price) => return Ok(price),
            Err(e) => {
                if attempt + 1 >= max_retries {
                    return Err(format!("All {} attempts failed: {}", max_retries, e));
                }

                // Exponential delay: 100ms, 200ms, 400ms, 800ms...
                let delay = base_delay_ms * 2_u64.pow(attempt);
                println!("  Waiting {}ms before retry...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    Err("Unexpected end of retry loop".to_string())
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    if random < 60 {
        Ok(2500.0 + (random as f64))
    } else {
        Err("Server overloaded".to_string())
    }
}
```

## Retry Configuration Structure

```rust
use std::thread;
use std::time::Duration;

struct RetryConfig {
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    exponential: bool,
}

impl RetryConfig {
    fn new() -> Self {
        RetryConfig {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 10000,
            exponential: true,
        }
    }

    fn with_max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n;
        self
    }

    fn with_base_delay(mut self, ms: u64) -> Self {
        self.base_delay_ms = ms;
        self
    }

    fn calculate_delay(&self, attempt: u32) -> u64 {
        if self.exponential {
            let delay = self.base_delay_ms * 2_u64.pow(attempt);
            delay.min(self.max_delay_ms)
        } else {
            self.base_delay_ms
        }
    }
}

fn main() {
    let config = RetryConfig::new()
        .with_max_attempts(5)
        .with_base_delay(200);

    match submit_order_with_retry("BTC", 0.1, 42000.0, &config) {
        Ok(order_id) => println!("Order placed: {}", order_id),
        Err(e) => println!("Order failed: {}", e),
    }
}

fn submit_order_with_retry(
    symbol: &str,
    quantity: f64,
    price: f64,
    config: &RetryConfig,
) -> Result<String, String> {
    for attempt in 0..config.max_attempts {
        println!("Order attempt {} for {} {} @ ${}",
                 attempt + 1, quantity, symbol, price);

        match submit_order(symbol, quantity, price) {
            Ok(id) => return Ok(id),
            Err(e) => {
                if attempt + 1 >= config.max_attempts {
                    return Err(format!("Order failed after {} attempts: {}",
                                      config.max_attempts, e));
                }

                let delay = config.calculate_delay(attempt);
                println!("  Retrying in {}ms: {}", delay, e);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    Err("Retry loop ended unexpectedly".to_string())
}

fn submit_order(_symbol: &str, _quantity: f64, _price: f64) -> Result<String, String> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    if random < 50 {
        Ok(format!("ORD-{}", random))
    } else {
        Err("Rate limit exceeded".to_string())
    }
}
```

## Retry with Error Classification

Not all errors should be retried:

```rust
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum OrderError {
    RateLimited,      // Can retry
    NetworkTimeout,   // Can retry
    InsufficientFunds, // Do NOT retry
    InvalidSymbol,     // Do NOT retry
    ServerError,       // Can retry
}

impl OrderError {
    fn is_retryable(&self) -> bool {
        match self {
            OrderError::RateLimited => true,
            OrderError::NetworkTimeout => true,
            OrderError::ServerError => true,
            OrderError::InsufficientFunds => false,
            OrderError::InvalidSymbol => false,
        }
    }

    fn suggested_delay_ms(&self) -> u64 {
        match self {
            OrderError::RateLimited => 1000,    // Rate limit — wait longer
            OrderError::NetworkTimeout => 500,
            OrderError::ServerError => 2000,
            _ => 0,
        }
    }
}

fn main() {
    match place_order_smart_retry("BTC", 0.5, 42000.0) {
        Ok(id) => println!("Success! Order ID: {}", id),
        Err(e) => println!("Failed: {:?}", e),
    }
}

fn place_order_smart_retry(
    symbol: &str,
    quantity: f64,
    price: f64,
) -> Result<String, OrderError> {
    let max_retries = 5;

    for attempt in 0..max_retries {
        println!("Attempt {}: {} {} @ ${}", attempt + 1, quantity, symbol, price);

        match place_order(symbol, quantity, price) {
            Ok(id) => return Ok(id),
            Err(e) => {
                println!("  Error: {:?}", e);

                // Check if we should retry
                if !e.is_retryable() {
                    println!("  Error is not retryable, giving up");
                    return Err(e);
                }

                if attempt + 1 >= max_retries {
                    return Err(e);
                }

                let delay = e.suggested_delay_ms();
                println!("  Waiting {}ms before retry...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    Err(OrderError::ServerError)
}

fn place_order(_symbol: &str, _quantity: f64, _price: f64) -> Result<String, OrderError> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    match random {
        0..=39 => Ok(format!("ORD-{}", random)),
        40..=59 => Err(OrderError::RateLimited),
        60..=79 => Err(OrderError::NetworkTimeout),
        80..=89 => Err(OrderError::ServerError),
        _ => Err(OrderError::InsufficientFunds),
    }
}
```

## Retry with Jitter

Adding randomness to avoid "thundering herd":

```rust
use std::thread;
use std::time::Duration;

fn main() {
    for i in 1..=3 {
        println!("\n=== Request {} ===", i);
        match fetch_with_jitter("SOL", 3) {
            Ok(price) => println!("SOL: ${:.2}", price),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn calculate_delay_with_jitter(attempt: u32, base_ms: u64) -> u64 {
    let base_delay = base_ms * 2_u64.pow(attempt);

    // Add random jitter ±25%
    let jitter_range = base_delay / 4;
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64;

    let jitter = (random % (jitter_range * 2)) as i64 - jitter_range as i64;

    (base_delay as i64 + jitter).max(0) as u64
}

fn fetch_with_jitter(symbol: &str, max_retries: u32) -> Result<f64, String> {
    for attempt in 0..max_retries {
        match fetch_price_unreliable(symbol) {
            Ok(price) => return Ok(price),
            Err(e) => {
                if attempt + 1 >= max_retries {
                    return Err(e);
                }

                let delay = calculate_delay_with_jitter(attempt, 100);
                println!("  Attempt {} failed, waiting {}ms", attempt + 1, delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    Err("All retries failed".to_string())
}

fn fetch_price_unreliable(_symbol: &str) -> Result<f64, String> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    if random < 40 {
        Ok(95.0 + (random as f64 / 10.0))
    } else {
        Err("Temporary failure".to_string())
    }
}
```

## Practical Example: Trading Client with Retry

```rust
use std::thread;
use std::time::Duration;

struct TradingClient {
    max_retries: u32,
    base_delay_ms: u64,
    timeout_ms: u64,
}

#[derive(Debug)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

#[derive(Debug)]
enum ApiError {
    Timeout,
    RateLimited,
    ServerError(String),
    InvalidRequest(String),
}

impl ApiError {
    fn is_retryable(&self) -> bool {
        matches!(self, ApiError::Timeout | ApiError::RateLimited | ApiError::ServerError(_))
    }
}

impl TradingClient {
    fn new() -> Self {
        TradingClient {
            max_retries: 3,
            base_delay_ms: 200,
            timeout_ms: 5000,
        }
    }

    fn with_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    fn get_price(&self, symbol: &str) -> Result<f64, ApiError> {
        self.with_retry(|| self.fetch_price_internal(symbol))
    }

    fn place_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
    ) -> Result<Order, ApiError> {
        self.with_retry(|| self.place_order_internal(symbol, side, quantity, price))
    }

    fn get_balance(&self, currency: &str) -> Result<f64, ApiError> {
        self.with_retry(|| self.get_balance_internal(currency))
    }

    fn with_retry<T, F>(&self, operation: F) -> Result<T, ApiError>
    where
        F: Fn() -> Result<T, ApiError>,
    {
        let mut last_error = ApiError::Timeout;

        for attempt in 0..self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    last_error = e;

                    if attempt + 1 < self.max_retries {
                        let delay = self.base_delay_ms * 2_u64.pow(attempt);
                        println!("  Attempt {} failed, retrying in {}ms...",
                                attempt + 1, delay);
                        thread::sleep(Duration::from_millis(delay));
                    }
                }
            }
        }

        Err(last_error)
    }

    // Internal methods (API simulation)
    fn fetch_price_internal(&self, symbol: &str) -> Result<f64, ApiError> {
        let random = get_random();
        match random % 100 {
            0..=59 => Ok(if symbol == "BTC" { 42500.0 } else { 2500.0 }),
            60..=79 => Err(ApiError::Timeout),
            80..=94 => Err(ApiError::RateLimited),
            _ => Err(ApiError::ServerError("Internal error".to_string())),
        }
    }

    fn place_order_internal(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
    ) -> Result<Order, ApiError> {
        let random = get_random();
        match random % 100 {
            0..=49 => Ok(Order {
                id: format!("ORD-{}", random),
                symbol: symbol.to_string(),
                side: side.to_string(),
                quantity,
                price,
                status: "FILLED".to_string(),
            }),
            50..=69 => Err(ApiError::Timeout),
            70..=84 => Err(ApiError::RateLimited),
            85..=94 => Err(ApiError::ServerError("Try again later".to_string())),
            _ => Err(ApiError::InvalidRequest("Invalid quantity".to_string())),
        }
    }

    fn get_balance_internal(&self, _currency: &str) -> Result<f64, ApiError> {
        let random = get_random();
        match random % 100 {
            0..=69 => Ok(10000.0 + (random % 1000) as f64),
            70..=89 => Err(ApiError::Timeout),
            _ => Err(ApiError::RateLimited),
        }
    }
}

fn get_random() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   TRADING CLIENT WITH RETRY LOGIC      ║");
    println!("╚════════════════════════════════════════╝\n");

    let client = TradingClient::new().with_retries(4);

    // Get balance
    println!("--- Fetching balance ---");
    match client.get_balance("USDT") {
        Ok(balance) => println!("Balance: ${:.2}\n", balance),
        Err(e) => println!("Balance error: {:?}\n", e),
    }

    // Get price
    println!("--- Fetching BTC price ---");
    match client.get_price("BTC") {
        Ok(price) => println!("BTC: ${:.2}\n", price),
        Err(e) => println!("Price error: {:?}\n", e),
    }

    // Place order
    println!("--- Placing order ---");
    match client.place_order("BTC", "BUY", 0.1, 42500.0) {
        Ok(order) => {
            println!("Order placed successfully!");
            println!("  ID: {}", order.id);
            println!("  {} {} {} @ ${}",
                    order.side, order.quantity, order.symbol, order.price);
            println!("  Status: {}", order.status);
        }
        Err(e) => println!("Order error: {:?}", e),
    }
}
```

## Circuit Breaker Pattern

If a service keeps failing, stop trying:

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,    // Normal operation
    Open,      // Service unavailable, don't try
    HalfOpen,  // Testing recovery
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure: Option<Instant>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout_secs: u64) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            reset_timeout: Duration::from_secs(reset_timeout_secs),
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if it's time to try again
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    fn state_name(&self) -> &str {
        match self.state {
            CircuitState::Closed => "CLOSED",
            CircuitState::Open => "OPEN",
            CircuitState::HalfOpen => "HALF-OPEN",
        }
    }
}

fn main() {
    let mut breaker = CircuitBreaker::new(3, 5);

    println!("Circuit Breaker Demo\n");

    for i in 1..=10 {
        println!("Request #{}: Circuit is {}", i, breaker.state_name());

        if !breaker.can_execute() {
            println!("  Circuit OPEN - request blocked\n");
            std::thread::sleep(Duration::from_millis(1000));
            continue;
        }

        // Simulate request
        match make_request() {
            Ok(result) => {
                println!("  Success: {}", result);
                breaker.record_success();
            }
            Err(e) => {
                println!("  Failed: {}", e);
                breaker.record_failure();
            }
        }

        println!("  Failures: {}\n", breaker.failure_count);
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn make_request() -> Result<String, String> {
    let random = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 100;

    if random < 30 {
        Ok("Data received".to_string())
    } else {
        Err("Service unavailable".to_string())
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Retry Logic | Repeating failed operations |
| Exponential Backoff | Increasing delay between attempts |
| Jitter | Random component in delay |
| Retryable Errors | Error classification for retry |
| Circuit Breaker | Stopping attempts on persistent failures |
| Max Retries | Limiting number of attempts |

## Exercises

1. **Retry with Timeout**: Implement a retry function that stops if total time exceeds a limit.

2. **Retry Logging**: Add logging of all attempts with timestamps to retry logic.

3. **Adaptive Retry**: Implement retry that increases delay on frequent RateLimited errors.

4. **Portfolio Retry**: Write a function that fetches prices for a list of assets with retry for each.

## Homework

1. Implement a `RetryPolicy` structure with configurable parameters:
   - Maximum number of attempts
   - Delay strategy (linear, exponential, fixed)
   - Maximum total duration
   - List of retryable errors

2. Create a `SmartTradingClient` that:
   - Uses Circuit Breaker for overload protection
   - Applies different retry strategies for different request types
   - Logs all retry attempts

3. Write a `batch_fetch_with_retry` function that:
   - Fetches data for multiple symbols
   - Independently retries failed requests
   - Returns partial results if some requests fail

4. Implement "hedged requests" — sending a duplicate request before the first one times out.

## Navigation

[← Previous day](../107-error-propagation-trading/en.md) | [Next day →](../109-custom-error-types/en.md)
