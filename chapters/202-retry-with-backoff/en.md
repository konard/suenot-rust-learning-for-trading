# Day 202: Retry with Backoff: Repeating Requests

## Trading Analogy

Imagine this situation: your trading bot sends an order to the exchange, but the server is temporarily unavailable. What should you do? Immediately retry the request? What if the server is still unavailable? If you retry too frequently, you risk:
- Overloading the server even more
- Getting banned for exceeding rate limits
- Missing the moment when the server comes back online

**Retry with backoff** is a retry strategy where we wait longer and longer between attempts. Like an experienced trader who doesn't spam orders but pauses and waits for the right moment.

In real trading, this is critically important:
- Exchange APIs often return temporary errors (503, 429)
- Network failures happen constantly
- Rate limiting on exchanges is common

## What is Exponential Backoff?

**Exponential Backoff** is an algorithm where the wait time between attempts increases exponentially:

```
Attempt 1: wait 1 second
Attempt 2: wait 2 seconds
Attempt 3: wait 4 seconds
Attempt 4: wait 8 seconds
...
```

Formula: `delay = base_delay * (2 ^ attempt)`

## Simple Retry Example

```rust
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
enum ExchangeError {
    TemporaryError(String),
    PermanentError(String),
    RateLimited,
}

// Simulating order submission to exchange
async fn send_order_to_exchange(symbol: &str, quantity: f64, price: f64) -> Result<u64, ExchangeError> {
    // In real code, this would be an HTTP request
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: f64 = rng.gen();

    if random < 0.7 {
        // 70% chance of temporary error
        Err(ExchangeError::TemporaryError("Server overloaded".to_string()))
    } else {
        Ok(12345) // Order ID
    }
}

async fn send_order_with_retry(
    symbol: &str,
    quantity: f64,
    price: f64,
    max_retries: u32,
) -> Result<u64, ExchangeError> {
    let mut attempt = 0;
    let base_delay = Duration::from_millis(100);

    loop {
        match send_order_to_exchange(symbol, quantity, price).await {
            Ok(order_id) => {
                println!("Order {} successfully sent on attempt {}", order_id, attempt + 1);
                return Ok(order_id);
            }
            Err(ExchangeError::PermanentError(msg)) => {
                // Permanent error — retrying is pointless
                println!("Permanent error: {}", msg);
                return Err(ExchangeError::PermanentError(msg));
            }
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    println!("Maximum retry attempts exceeded: {}", max_retries);
                    return Err(e);
                }

                // Exponential backoff
                let delay = base_delay * 2_u32.pow(attempt - 1);
                println!(
                    "Attempt {} failed: {:?}. Waiting {:?}...",
                    attempt, e, delay
                );
                sleep(delay).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    match send_order_with_retry("BTC/USDT", 0.1, 42000.0, 5).await {
        Ok(order_id) => println!("Order created: {}", order_id),
        Err(e) => println!("Failed to create order: {:?}", e),
    }
}
```

## Adding Jitter — Random Variation

If many clients start retrying simultaneously, they will overload the server again at the same time. **Jitter** adds random variation to the delay:

```rust
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

fn calculate_backoff_with_jitter(attempt: u32, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
    let mut rng = rand::thread_rng();

    // Exponential growth
    let exponential_delay = base_delay_ms * 2_u64.pow(attempt);

    // Cap at maximum delay
    let capped_delay = exponential_delay.min(max_delay_ms);

    // Add random jitter (0% - 100% of delay)
    let jitter = rng.gen_range(0..=capped_delay);

    Duration::from_millis(capped_delay + jitter)
}

async fn fetch_price_with_jitter(
    symbol: &str,
    max_retries: u32,
) -> Result<f64, String> {
    let base_delay_ms = 100;
    let max_delay_ms = 10000; // Maximum 10 seconds

    for attempt in 0..max_retries {
        // Simulating price request
        let result: Result<f64, &str> = if attempt < 2 {
            Err("Service temporarily unavailable")
        } else {
            Ok(42150.50)
        };

        match result {
            Ok(price) => return Ok(price),
            Err(e) => {
                if attempt + 1 >= max_retries {
                    return Err(format!("Retry limit exceeded: {}", e));
                }

                let delay = calculate_backoff_with_jitter(attempt, base_delay_ms, max_delay_ms);
                println!(
                    "[{}] Attempt {} failed. Waiting {:?}...",
                    symbol, attempt + 1, delay
                );
                sleep(delay).await;
            }
        }
    }

    Err("Unexpected end of retry loop".to_string())
}

#[tokio::main]
async fn main() {
    match fetch_price_with_jitter("ETH/USDT", 5).await {
        Ok(price) => println!("ETH price: ${:.2}", price),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Retry Configuration Structure

```rust
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new(max_retries: u32) -> Self {
        RetryConfig {
            max_retries,
            ..Default::default()
        }
    }

    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_jitter(mut self, use_jitter: bool) -> Self {
        self.use_jitter = use_jitter;
        self
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_ms = self.base_delay.as_millis() as u64;
        let max_ms = self.max_delay.as_millis() as u64;

        // Exponential growth
        let exponential = base_ms.saturating_mul(2_u64.pow(attempt));
        let capped = exponential.min(max_ms);

        if self.use_jitter {
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0..=capped / 2);
            Duration::from_millis(capped + jitter)
        } else {
            Duration::from_millis(capped)
        }
    }
}

// Trait to determine if retry is worthwhile
pub trait Retryable {
    fn is_retryable(&self) -> bool;
}

#[derive(Debug)]
pub enum TradingError {
    NetworkError(String),
    RateLimited { retry_after: Option<Duration> },
    ServerError(String),
    InvalidOrder(String),
    InsufficientFunds,
}

impl Retryable for TradingError {
    fn is_retryable(&self) -> bool {
        match self {
            TradingError::NetworkError(_) => true,
            TradingError::RateLimited { .. } => true,
            TradingError::ServerError(_) => true,
            // These errors are not worth retrying
            TradingError::InvalidOrder(_) => false,
            TradingError::InsufficientFunds => false,
        }
    }
}

pub async fn retry_async<T, E, F, Fut>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T, E>
where
    E: Retryable + std::fmt::Debug,
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    println!("[{}] Success on attempt {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(e) => {
                if !e.is_retryable() {
                    println!("[{}] Non-retryable error: {:?}", operation_name, e);
                    return Err(e);
                }

                attempt += 1;
                if attempt >= config.max_retries {
                    println!(
                        "[{}] Retry limit exceeded ({}): {:?}",
                        operation_name, config.max_retries, e
                    );
                    return Err(e);
                }

                // Special handling for Rate Limiting
                let delay = if let TradingError::RateLimited { retry_after: Some(ra) } = &e {
                    // This works only if E = TradingError
                    // In general case, use standard backoff
                    *ra
                } else {
                    config.calculate_delay(attempt - 1)
                };

                println!(
                    "[{}] Attempt {} failed: {:?}. Waiting {:?}",
                    operation_name, attempt, e, delay
                );
                sleep(delay).await;
            }
        }
    }
}
```

## Practical Example: Exchange API Client

```rust
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<u64>,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum ApiError {
    NetworkError(String),
    RateLimited { retry_after_ms: u64 },
    ServerError { status: u16, message: String },
    InvalidRequest(String),
    AuthenticationError,
}

impl ApiError {
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            ApiError::NetworkError(_) | ApiError::RateLimited { .. } | ApiError::ServerError { .. }
        )
    }
}

pub struct ExchangeClient {
    base_url: String,
    api_key: String,
    retry_config: RetryConfig,
}

#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 30000,
        }
    }
}

impl ExchangeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        ExchangeClient {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    async fn send_request(&self, endpoint: &str) -> Result<String, ApiError> {
        // Simulating network request
        let mut rng = rand::thread_rng();
        let random: f64 = rng.gen();

        sleep(Duration::from_millis(50)).await;

        if random < 0.3 {
            Err(ApiError::NetworkError("Connection timeout".to_string()))
        } else if random < 0.4 {
            Err(ApiError::RateLimited { retry_after_ms: 1000 })
        } else if random < 0.5 {
            Err(ApiError::ServerError {
                status: 503,
                message: "Service temporarily unavailable".to_string(),
            })
        } else {
            Ok(format!("Response from {}{}", self.base_url, endpoint))
        }
    }

    fn calculate_delay(&self, attempt: u32, error: &ApiError) -> Duration {
        // If server specified wait time, use it
        if let ApiError::RateLimited { retry_after_ms } = error {
            return Duration::from_millis(*retry_after_ms);
        }

        // Otherwise — exponential backoff with jitter
        let base = self.retry_config.base_delay_ms;
        let max = self.retry_config.max_delay_ms;

        let exponential = base * 2_u64.pow(attempt);
        let capped = exponential.min(max);

        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=capped / 2);

        Duration::from_millis(capped + jitter)
    }

    pub async fn get_price(&self, symbol: &str) -> Result<f64, ApiError> {
        let endpoint = format!("/api/v1/ticker/{}", symbol);
        let mut attempt = 0;

        loop {
            match self.send_request(&endpoint).await {
                Ok(_response) => {
                    // Parse price from response
                    let price = 42150.50 + (attempt as f64 * 10.0);
                    println!("[Price] {} = ${:.2} (attempt {})", symbol, price, attempt + 1);
                    return Ok(price);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    attempt += 1;
                    if attempt >= self.retry_config.max_retries {
                        println!(
                            "[Price] Retry limit exceeded for {}: {:?}",
                            symbol, e
                        );
                        return Err(e);
                    }

                    let delay = self.calculate_delay(attempt - 1, &e);
                    println!(
                        "[Price] Attempt {} for {} failed: {:?}. Waiting {:?}",
                        attempt, symbol, e, delay
                    );
                    sleep(delay).await;
                }
            }
        }
    }

    pub async fn place_order(&self, order: &Order) -> Result<u64, ApiError> {
        let endpoint = "/api/v1/orders";
        let mut attempt = 0;

        loop {
            match self.send_request(endpoint).await {
                Ok(_response) => {
                    let order_id = 1000 + attempt as u64;
                    println!(
                        "[Order] {:?} {} {} @ {} created (ID: {}, attempt {})",
                        order.side, order.quantity, order.symbol, order.price,
                        order_id, attempt + 1
                    );
                    return Ok(order_id);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    attempt += 1;
                    if attempt >= self.retry_config.max_retries {
                        println!("[Order] Retry limit exceeded: {:?}", e);
                        return Err(e);
                    }

                    let delay = self.calculate_delay(attempt - 1, &e);
                    println!(
                        "[Order] Attempt {} failed: {:?}. Waiting {:?}",
                        attempt, e, delay
                    );
                    sleep(delay).await;
                }
            }
        }
    }

    pub async fn cancel_order(&self, order_id: u64) -> Result<bool, ApiError> {
        let endpoint = format!("/api/v1/orders/{}", order_id);
        let mut attempt = 0;

        loop {
            match self.send_request(&endpoint).await {
                Ok(_) => {
                    println!(
                        "[Cancel] Order {} cancelled (attempt {})",
                        order_id, attempt + 1
                    );
                    return Ok(true);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    attempt += 1;
                    if attempt >= self.retry_config.max_retries {
                        return Err(e);
                    }

                    let delay = self.calculate_delay(attempt - 1, &e);
                    println!(
                        "[Cancel] Attempt {} for order {} failed. Waiting {:?}",
                        attempt, order_id, delay
                    );
                    sleep(delay).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let client = ExchangeClient::new("https://api.exchange.com", "my-api-key")
        .with_retry_config(RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 10000,
        });

    // Get price
    match client.get_price("BTC/USDT").await {
        Ok(price) => println!("Current BTC price: ${:.2}", price),
        Err(e) => println!("Failed to get price: {:?}", e),
    }

    // Place order
    let order = Order {
        id: None,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 0.1,
        price: 42000.0,
    };

    match client.place_order(&order).await {
        Ok(order_id) => {
            println!("Order created with ID: {}", order_id);

            // Cancel order
            match client.cancel_order(order_id).await {
                Ok(_) => println!("Order {} successfully cancelled", order_id),
                Err(e) => println!("Failed to cancel order: {:?}", e),
            }
        }
        Err(e) => println!("Failed to create order: {:?}", e),
    }
}
```

## Retry for Multiple Requests

```rust
use std::time::Duration;
use tokio::time::sleep;
use futures::future::join_all;

#[derive(Clone)]
struct PriceFetcher {
    max_retries: u32,
    base_delay_ms: u64,
}

impl PriceFetcher {
    fn new() -> Self {
        PriceFetcher {
            max_retries: 3,
            base_delay_ms: 100,
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<(String, f64), String> {
        use rand::Rng;
        let mut attempt = 0;

        loop {
            let mut rng = rand::thread_rng();
            let success: bool = rng.gen_bool(0.6);

            if success {
                let price: f64 = match symbol {
                    "BTC/USDT" => 42000.0 + rng.gen_range(-100.0..100.0),
                    "ETH/USDT" => 2500.0 + rng.gen_range(-50.0..50.0),
                    "SOL/USDT" => 100.0 + rng.gen_range(-5.0..5.0),
                    _ => 1.0,
                };
                return Ok((symbol.to_string(), price));
            }

            attempt += 1;
            if attempt >= self.max_retries {
                return Err(format!("Failed to get price for {}", symbol));
            }

            let delay = Duration::from_millis(self.base_delay_ms * 2_u64.pow(attempt - 1));
            println!("[{}] Attempt {} failed, waiting {:?}", symbol, attempt, delay);
            sleep(delay).await;
        }
    }

    async fn fetch_multiple_prices(&self, symbols: Vec<&str>) -> Vec<Result<(String, f64), String>> {
        let futures: Vec<_> = symbols
            .into_iter()
            .map(|symbol| self.fetch_price(symbol))
            .collect();

        join_all(futures).await
    }
}

#[tokio::main]
async fn main() {
    let fetcher = PriceFetcher::new();

    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];
    println!("Fetching prices for: {:?}\n", symbols);

    let results = fetcher.fetch_multiple_prices(symbols).await;

    println!("\nResults:");
    for result in results {
        match result {
            Ok((symbol, price)) => println!("  {} = ${:.2}", symbol, price),
            Err(e) => println!("  Error: {}", e),
        }
    }
}
```

## Backoff Strategies

```rust
use std::time::Duration;

#[derive(Clone)]
pub enum BackoffStrategy {
    /// Constant delay
    Constant { delay: Duration },

    /// Linear increase: delay = base * attempt
    Linear { base: Duration, max: Duration },

    /// Exponential increase: delay = base * 2^attempt
    Exponential { base: Duration, max: Duration },

    /// Exponential with jitter
    ExponentialWithJitter { base: Duration, max: Duration },

    /// Decorrelated jitter (AWS recommendation)
    DecorrelatedJitter { base: Duration, max: Duration },
}

impl BackoffStrategy {
    pub fn calculate(&self, attempt: u32, previous_delay: Option<Duration>) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match self {
            BackoffStrategy::Constant { delay } => *delay,

            BackoffStrategy::Linear { base, max } => {
                let delay = *base * (attempt + 1);
                delay.min(*max)
            }

            BackoffStrategy::Exponential { base, max } => {
                let multiplier = 2_u32.pow(attempt);
                let delay = *base * multiplier;
                delay.min(*max)
            }

            BackoffStrategy::ExponentialWithJitter { base, max } => {
                let multiplier = 2_u32.pow(attempt);
                let base_delay = (*base * multiplier).min(*max);
                let jitter_range = base_delay.as_millis() as u64;
                let jitter = rng.gen_range(0..=jitter_range);
                Duration::from_millis(base_delay.as_millis() as u64 + jitter)
            }

            BackoffStrategy::DecorrelatedJitter { base, max } => {
                // AWS algorithm: sleep = min(max, random(base, prev_sleep * 3))
                let prev = previous_delay.unwrap_or(*base);
                let upper = (prev.as_millis() as u64 * 3).min(max.as_millis() as u64);
                let lower = base.as_millis() as u64;
                let delay = rng.gen_range(lower..=upper.max(lower));
                Duration::from_millis(delay)
            }
        }
    }
}

fn demonstrate_strategies() {
    let strategies = vec![
        ("Constant", BackoffStrategy::Constant {
            delay: Duration::from_millis(100)
        }),
        ("Linear", BackoffStrategy::Linear {
            base: Duration::from_millis(100),
            max: Duration::from_secs(10)
        }),
        ("Exponential", BackoffStrategy::Exponential {
            base: Duration::from_millis(100),
            max: Duration::from_secs(30)
        }),
        ("Exponential + Jitter", BackoffStrategy::ExponentialWithJitter {
            base: Duration::from_millis(100),
            max: Duration::from_secs(30)
        }),
    ];

    println!("Comparing backoff strategies:\n");
    println!("{:<25} {:>10} {:>10} {:>10} {:>10} {:>10}",
             "Strategy", "Attempt 1", "Attempt 2", "Attempt 3", "Attempt 4", "Attempt 5");
    println!("{}", "-".repeat(80));

    for (name, strategy) in strategies {
        print!("{:<25}", name);
        let mut prev_delay = None;
        for attempt in 0..5 {
            let delay = strategy.calculate(attempt, prev_delay);
            print!(" {:>9}ms", delay.as_millis());
            prev_delay = Some(delay);
        }
        println!();
    }
}

fn main() {
    demonstrate_strategies();
}
```

## Using tokio-retry Library

```rust
use std::time::Duration;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tokio_retry::Retry;

#[derive(Debug)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn fetch_price_unreliable(symbol: &str) -> Result<PriceData, &'static str> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // 60% chance of failure
    if rng.gen_bool(0.6) {
        Err("Connection failed")
    } else {
        Ok(PriceData {
            symbol: symbol.to_string(),
            price: 42150.50,
            timestamp: 1699999999,
        })
    }
}

#[tokio::main]
async fn main() {
    // Create strategy: initial delay 100ms, maximum 5 attempts
    let retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(10))
        .map(jitter) // Add random variation
        .take(5);    // Maximum 5 attempts

    let symbol = "BTC/USDT";

    let result = Retry::spawn(retry_strategy, || async {
        println!("Trying to get price for {}...", symbol);
        fetch_price_unreliable(symbol).await
    }).await;

    match result {
        Ok(data) => println!("Data received: {:?}", data),
        Err(e) => println!("All attempts failed: {}", e),
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Retry | Repeating an operation after failure |
| Backoff | Increasing wait time between attempts |
| Exponential Backoff | Exponential delay growth (2^n) |
| Jitter | Random variation to prevent synchronized retries |
| Retryable errors | Distinguishing between temporary and permanent errors |
| Rate limiting | Handling API request frequency limits |

## Homework

1. **Basic retry**: Implement a `retry_sync` function that:
   - Takes a closure `F: FnMut() -> Result<T, E>`
   - Retries up to `max_retries` times
   - Uses linear backoff (delay increases by a fixed amount)
   - Logs each attempt

2. **Smart API retry**: Create a `SmartRetryClient` structure that:
   - Distinguishes error types (network, rate limit, authentication, invalid data)
   - Uses special handling for rate limit (waits server-specified time)
   - Does not retry permanent errors
   - Keeps statistics of successful/failed attempts

3. **Parallel fetch with retry**: Write a `fetch_portfolio_prices` function that:
   - Gets prices for a list of assets in parallel
   - Each request has its own retry with backoff
   - Returns `HashMap<String, Result<f64, Error>>`
   - Doesn't abort when one asset fails

4. **Circuit Breaker + Retry**: Combine patterns:
   - Implement Circuit Breaker from previous chapters
   - Add retry only when circuit is not open
   - When errors are too frequent, circuit opens and retry stops

## Navigation

[← Previous day](../201-rate-limiting/en.md) | [Next day →](../203-websocket-streaming/en.md)
