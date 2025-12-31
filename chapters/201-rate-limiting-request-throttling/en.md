# Day 201: Rate Limiting: Request Throttling

## Trading Analogy

Imagine you're trading on Binance and your trading bot sends 1000 requests per second to get current prices for all trading pairs. The exchange will immediately block your IP address or API key! Why? Because the exchange uses **rate limiting** — a mechanism to protect servers from overload.

It's like a queue at a bank: a teller can only serve a certain number of clients per hour. If everyone rushes in at once — the system will crash. That's why limits are established: for example, 10 requests per second or 1200 requests per minute.

In trading, rate limiting is critically important:
- **Binance**: 1200 requests per minute (weight-based system)
- **Bybit**: 120 requests per minute per endpoint
- **Kraken**: 15-20 calls per minute (depends on tier)

If you exceed the limit — you'll get a 429 error (Too Many Requests) or a temporary ban.

## What is Rate Limiting?

**Rate limiting** is a technique for controlling the number of requests a client can send within a specific time period. Client-side implementation allows us to:

1. **Stay within API limits** — avoid getting blocked
2. **Fairly distribute resources** — don't interfere with other users
3. **Protect the server** — prevent accidental DDoS from our bot

## Main Rate Limiting Algorithms

### 1. Fixed Window

The simplest approach: count requests in fixed time intervals.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct FixedWindowLimiter {
    max_requests: u64,
    window_duration: Duration,
    request_count: AtomicU64,
    window_start: Mutex<Instant>,
}

impl FixedWindowLimiter {
    fn new(max_requests: u64, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            request_count: AtomicU64::new(0),
            window_start: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self) -> bool {
        let mut window_start = self.window_start.lock().await;
        let now = Instant::now();

        // If window has expired — reset the counter
        if now.duration_since(*window_start) >= self.window_duration {
            *window_start = now;
            self.request_count.store(0, Ordering::SeqCst);
        }

        // Check if we can make a request
        let current = self.request_count.fetch_add(1, Ordering::SeqCst);
        if current < self.max_requests {
            true
        } else {
            self.request_count.fetch_sub(1, Ordering::SeqCst);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    // Limit: 10 requests per second (like on some exchanges)
    let limiter = Arc::new(FixedWindowLimiter::new(10, Duration::from_secs(1)));

    println!("Simulating API requests to exchange (limit: 10 req/sec):");

    for i in 0..15 {
        if limiter.acquire().await {
            println!("  Request {} to /api/v1/ticker/price - OK", i + 1);
        } else {
            println!("  Request {} to /api/v1/ticker/price - REJECTED (limit exceeded)", i + 1);
        }
    }
}
```

### 2. Token Bucket

A more flexible algorithm: tokens are added at a constant rate, each request consumes a token.

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct TokenBucket {
    capacity: f64,           // Maximum tokens in the bucket
    tokens: Mutex<f64>,      // Current number of tokens
    refill_rate: f64,        // Tokens per second
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Mutex::new(capacity), // Start with a full bucket
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    async fn try_acquire(&self, tokens_needed: f64) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // Add tokens for elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        *tokens = (*tokens + new_tokens).min(self.capacity);
        *last_refill = now;

        // Try to take tokens
        if *tokens >= tokens_needed {
            *tokens -= tokens_needed;
            true
        } else {
            false
        }
    }

    async fn acquire(&self, tokens_needed: f64) {
        loop {
            if self.try_acquire(tokens_needed).await {
                return;
            }
            // Wait a bit and try again
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn available_tokens(&self) -> f64 {
        *self.tokens.lock().await
    }
}

#[tokio::main]
async fn main() {
    // Binance-like limiter: 20 requests per second
    let bucket = Arc::new(TokenBucket::new(20.0, 20.0));

    println!("Token Bucket Rate Limiter for trading bot:");
    println!("  Capacity: 20 tokens");
    println!("  Refill rate: 20 tokens/sec\n");

    // Simulate different request types with different weights
    let requests = vec![
        ("GET /api/v1/ticker/price", 1.0),      // Light request
        ("GET /api/v1/depth", 5.0),             // Heavy request (order book)
        ("POST /api/v1/order", 1.0),            // Place order
        ("GET /api/v1/klines", 5.0),            // Candlestick history
    ];

    for (endpoint, weight) in requests {
        let available = bucket.available_tokens().await;
        println!("Available tokens: {:.1}", available);

        bucket.acquire(weight).await;
        println!("  {} (weight: {}) - executed", endpoint, weight);
    }
}
```

### 3. Sliding Window Log

Accurate but requires more memory: we store the timestamp of each request.

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct SlidingWindowLog {
    window_duration: Duration,
    max_requests: usize,
    timestamps: Mutex<VecDeque<Instant>>,
}

impl SlidingWindowLog {
    fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            window_duration,
            max_requests,
            timestamps: Mutex::new(VecDeque::new()),
        }
    }

    async fn try_acquire(&self) -> bool {
        let mut timestamps = self.timestamps.lock().await;
        let now = Instant::now();

        // Remove old timestamps outside the window
        while let Some(ts) = timestamps.front() {
            if now.duration_since(*ts) >= self.window_duration {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        // Check the limit
        if timestamps.len() < self.max_requests {
            timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    async fn time_until_available(&self) -> Duration {
        let timestamps = self.timestamps.lock().await;
        if timestamps.len() < self.max_requests {
            return Duration::ZERO;
        }

        if let Some(oldest) = timestamps.front() {
            let elapsed = Instant::now().duration_since(*oldest);
            if elapsed < self.window_duration {
                return self.window_duration - elapsed;
            }
        }

        Duration::ZERO
    }
}

#[tokio::main]
async fn main() {
    // Kraken-like limit: 15 requests per minute
    let limiter = SlidingWindowLog::new(15, Duration::from_secs(60));

    println!("Sliding Window Rate Limiter (Kraken-style):");
    println!("  Limit: 15 requests per minute\n");

    for i in 0..20 {
        if limiter.try_acquire().await {
            println!("  [{}] Request accepted", i + 1);
        } else {
            let wait_time = limiter.time_until_available().await;
            println!(
                "  [{}] Request rejected, wait {:.1} sec",
                i + 1,
                wait_time.as_secs_f64()
            );
        }
    }
}
```

## Practical Example: Exchange Client with Rate Limiting

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Realistic structure for request weights (like on Binance)
struct RequestWeight {
    endpoint: &'static str,
    weight: u32,
}

const ENDPOINTS: &[RequestWeight] = &[
    RequestWeight { endpoint: "/api/v3/ticker/price", weight: 1 },
    RequestWeight { endpoint: "/api/v3/depth", weight: 5 },
    RequestWeight { endpoint: "/api/v3/klines", weight: 5 },
    RequestWeight { endpoint: "/api/v3/order", weight: 1 },
    RequestWeight { endpoint: "/api/v3/account", weight: 10 },
];

struct ExchangeRateLimiter {
    max_weight_per_minute: u32,
    current_weight: Mutex<u32>,
    window_start: Mutex<Instant>,
}

impl ExchangeRateLimiter {
    fn new(max_weight_per_minute: u32) -> Self {
        Self {
            max_weight_per_minute,
            current_weight: Mutex::new(0),
            window_start: Mutex::new(Instant::now()),
        }
    }

    async fn check_and_wait(&self, weight: u32) {
        loop {
            let mut current = self.current_weight.lock().await;
            let mut start = self.window_start.lock().await;
            let now = Instant::now();

            // Reset window if a minute has passed
            if now.duration_since(*start) >= Duration::from_secs(60) {
                *start = now;
                *current = 0;
            }

            // Check if we can execute the request
            if *current + weight <= self.max_weight_per_minute {
                *current += weight;
                return;
            }

            // Calculate wait time
            let elapsed = now.duration_since(*start);
            let wait_time = Duration::from_secs(60) - elapsed;

            drop(current);
            drop(start);

            println!(
                "    [Rate Limit] Waiting {:.1} sec until limit reset...",
                wait_time.as_secs_f64()
            );
            tokio::time::sleep(wait_time).await;
        }
    }

    async fn get_current_usage(&self) -> (u32, u32) {
        let current = *self.current_weight.lock().await;
        (current, self.max_weight_per_minute)
    }
}

struct TradingApiClient {
    rate_limiter: Arc<ExchangeRateLimiter>,
    base_url: String,
}

impl TradingApiClient {
    fn new(max_weight_per_minute: u32) -> Self {
        Self {
            rate_limiter: Arc::new(ExchangeRateLimiter::new(max_weight_per_minute)),
            base_url: "https://api.exchange.com".to_string(),
        }
    }

    async fn get_price(&self, symbol: &str) -> Result<f64, String> {
        self.rate_limiter.check_and_wait(1).await;

        // Simulate HTTP request
        println!("  GET {}/api/v3/ticker/price?symbol={}", self.base_url, symbol);

        // In reality this would be reqwest::get(...)
        Ok(42000.0) // Simulated BTC price
    }

    async fn get_orderbook(&self, symbol: &str, limit: u32) -> Result<OrderBook, String> {
        // Order book is a heavy request with weight 5
        self.rate_limiter.check_and_wait(5).await;

        println!(
            "  GET {}/api/v3/depth?symbol={}&limit={}",
            self.base_url, symbol, limit
        );

        Ok(OrderBook {
            bids: vec![(41990.0, 1.5), (41980.0, 2.3)],
            asks: vec![(42010.0, 0.8), (42020.0, 1.2)],
        })
    }

    async fn place_order(&self, symbol: &str, side: &str, qty: f64) -> Result<u64, String> {
        self.rate_limiter.check_and_wait(1).await;

        println!(
            "  POST {}/api/v3/order {{ symbol: {}, side: {}, qty: {} }}",
            self.base_url, symbol, side, qty
        );

        Ok(12345678) // Simulated order_id
    }

    async fn get_account(&self) -> Result<AccountInfo, String> {
        // Account info is a very heavy request (weight 10)
        self.rate_limiter.check_and_wait(10).await;

        println!("  GET {}/api/v3/account", self.base_url);

        Ok(AccountInfo {
            balances: vec![
                ("BTC".to_string(), 1.5),
                ("USDT".to_string(), 50000.0),
            ],
        })
    }

    async fn print_usage(&self) {
        let (current, max) = self.rate_limiter.get_current_usage().await;
        println!("  [Used: {}/{} weight]", current, max);
    }
}

#[derive(Debug)]
struct OrderBook {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

#[derive(Debug)]
struct AccountInfo {
    balances: Vec<(String, f64)>,
}

#[tokio::main]
async fn main() {
    println!("=== Trading Client with Rate Limiting ===\n");

    // Binance-like: 1200 weight per minute
    let client = TradingApiClient::new(1200);

    println!("1. Getting BTC price:");
    let price = client.get_price("BTCUSDT").await.unwrap();
    println!("    BTC Price: ${}\n", price);
    client.print_usage().await;

    println!("\n2. Getting order book:");
    let orderbook = client.get_orderbook("BTCUSDT", 10).await.unwrap();
    println!("    Best bid: ${}", orderbook.bids[0].0);
    println!("    Best ask: ${}\n", orderbook.asks[0].0);
    client.print_usage().await;

    println!("\n3. Placing order:");
    let order_id = client.place_order("BTCUSDT", "BUY", 0.1).await.unwrap();
    println!("    Order ID: {}\n", order_id);
    client.print_usage().await;

    println!("\n4. Getting account info:");
    let account = client.get_account().await.unwrap();
    for (asset, balance) in &account.balances {
        println!("    {}: {}", asset, balance);
    }
    client.print_usage().await;

    println!("\n=== Simulating bulk requests ===");

    // Simulate many price requests
    for i in 0..10 {
        let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"];
        let symbol = symbols[i % symbols.len()];
        let _ = client.get_price(symbol).await;
    }

    println!("\nFinal usage:");
    client.print_usage().await;
}
```

## Adaptive Rate Limiter

A smart limiter that adapts to server responses:

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum RateLimitState {
    Normal,       // Operating normally
    Cautious,     // Received warning, slowing down
    Throttled,    // Received 429, heavily slowing down
}

struct AdaptiveRateLimiter {
    base_delay: Duration,
    current_delay: Mutex<Duration>,
    state: Mutex<RateLimitState>,
    last_request: Mutex<Instant>,
    consecutive_errors: Mutex<u32>,
}

impl AdaptiveRateLimiter {
    fn new(requests_per_second: f64) -> Self {
        let base_delay = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            base_delay,
            current_delay: Mutex::new(base_delay),
            state: Mutex::new(RateLimitState::Normal),
            last_request: Mutex::new(Instant::now()),
            consecutive_errors: Mutex::new(0),
        }
    }

    async fn wait_for_slot(&self) {
        let delay = *self.current_delay.lock().await;
        let mut last = self.last_request.lock().await;

        let elapsed = last.elapsed();
        if elapsed < delay {
            tokio::time::sleep(delay - elapsed).await;
        }

        *last = Instant::now();
    }

    async fn report_success(&self) {
        let mut errors = self.consecutive_errors.lock().await;
        *errors = 0;

        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        // Gradually return to normal mode
        match *state {
            RateLimitState::Throttled => {
                *state = RateLimitState::Cautious;
                *delay = self.base_delay * 2;
                println!("    [Limiter] Transitioning to Cautious mode");
            }
            RateLimitState::Cautious => {
                *state = RateLimitState::Normal;
                *delay = self.base_delay;
                println!("    [Limiter] Returning to Normal mode");
            }
            RateLimitState::Normal => {}
        }
    }

    async fn report_rate_limit_warning(&self) {
        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        *state = RateLimitState::Cautious;
        *delay = self.base_delay * 2;
        println!("    [Limiter] Warning! Slowing down x2");
    }

    async fn report_rate_limit_error(&self) {
        let mut errors = self.consecutive_errors.lock().await;
        *errors += 1;

        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        *state = RateLimitState::Throttled;

        // Exponential backoff
        let multiplier = 2u32.pow((*errors).min(5));
        *delay = self.base_delay * multiplier;

        println!(
            "    [Limiter] Error 429! Slowing down x{} (consecutive errors: {})",
            multiplier, *errors
        );
    }

    async fn get_state(&self) -> RateLimitState {
        *self.state.lock().await
    }
}

// Server response simulator
struct MockServer {
    request_count: Mutex<u32>,
    limit: u32,
}

impl MockServer {
    fn new(limit: u32) -> Self {
        Self {
            request_count: Mutex::new(0),
            limit,
        }
    }

    async fn handle_request(&self) -> Result<u32, u32> {
        let mut count = self.request_count.lock().await;
        *count += 1;

        // Reset counter every 10 requests (simulating time window)
        if *count > 10 {
            *count = 1;
        }

        if *count > self.limit {
            Err(429) // Too Many Requests
        } else if *count == self.limit {
            Ok(200)  // OK but on the edge
        } else {
            Ok(200)
        }
    }

    async fn reset(&self) {
        *self.request_count.lock().await = 0;
    }
}

#[tokio::main]
async fn main() {
    println!("=== Adaptive Rate Limiter ===\n");

    let limiter = Arc::new(AdaptiveRateLimiter::new(5.0)); // 5 req/sec base
    let server = Arc::new(MockServer::new(3)); // Server accepts only 3 req at a time

    println!("Simulating trading requests:\n");

    for i in 0..15 {
        limiter.wait_for_slot().await;

        let result = server.handle_request().await;

        match result {
            Ok(200) => {
                println!("[{}] Request successful (state: {:?})", i + 1, limiter.get_state().await);
                limiter.report_success().await;
            }
            Err(429) => {
                println!("[{}] Error 429! (state: {:?})", i + 1, limiter.get_state().await);
                limiter.report_rate_limit_error().await;
            }
            _ => {}
        }

        // Periodically reset the server (simulating new time window)
        if i == 7 {
            println!("\n--- New time window ---\n");
            server.reset().await;
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Rate Limiting | Controlling the number of requests over a time period |
| Fixed Window | Simple counter in a fixed time window |
| Token Bucket | Bucket of tokens with constant refill |
| Sliding Window | Sliding window with precise counting |
| Weight System | Different requests have different costs |
| Adaptive Limiter | Automatically adjusts based on server responses |

## Practice Exercises

1. **Multi-level Limiter**: Implement a limiter that accounts for multiple restrictions simultaneously:
   - 10 requests per second
   - 100 requests per minute
   - 1000 requests per hour

2. **Priority Limiter**: Create a system where orders have priority over price queries. If the limit is almost exhausted, only allow important requests through.

3. **Distributed Limiter**: Implement a rate limiter that works with multiple bot instances, using a shared counter (hint: use tokio::sync::broadcast or Redis).

## Homework

1. **Multi-Exchange Rate Limiter**: Create a `MultiExchangeLimiter` struct that manages limits for multiple exchanges simultaneously. Each exchange has its own limits:
   ```rust
   struct ExchangeLimits {
       name: String,
       requests_per_minute: u32,
       weight_per_minute: u32,
   }
   ```

2. **Rate Limit with Redis**: Implement a `RedisRateLimiter` that stores state in Redis, allowing multiple processes to share common limits.

3. **Smart Exchange Client**: Create a client that:
   - Automatically chooses the optimal time for requests
   - Groups multiple requests into batches when possible
   - Caches frequently requested data
   - Uses WebSocket instead of REST for streaming data

4. **Limit Visualization**: Add a method that outputs a nice ASCII graph of limit usage over the last 60 seconds.

## Navigation

[← Previous day](../200-http-client-connection-pooling/en.md) | [Next day →](../202-retry-backoff/en.md)
