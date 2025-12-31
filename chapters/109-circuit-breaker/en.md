# Day 109: Circuit Breaker — Cascade Failure Protection

## Trading Analogy

Imagine you're trading through an exchange API. Suddenly, the exchange starts responding with delays or returning errors. If you keep sending requests, several bad things happen:

1. **System overload** — your retry attempts only make the problem worse
2. **Cascade failure** — other parts of your system (risk management, logging) start failing too
3. **Money loss** — orders get stuck, positions don't close

**Circuit Breaker** works like an automatic switch in your electrical panel: when overloaded, it "breaks the circuit", giving the system time to recover.

## Theoretical Foundations

### Three States of Circuit Breaker

```
     ┌─────────────────────────────────────────────────────────┐
     │                                                         │
     ▼                                                         │
┌─────────┐   failure_threshold   ┌────────┐   timeout   ┌─────────────┐
│ CLOSED  │ ─────────────────────►│  OPEN  │────────────►│ HALF-OPEN   │
│(working)│                       │(waiting)│            │(testing)    │
└─────────┘                       └────────┘             └─────────────┘
     ▲                                 ▲                       │
     │         success                 │      failure          │
     └─────────────────────────────────┴───────────────────────┘
```

- **Closed**: Normal operation, requests pass through
- **Open**: Failure detected, requests are blocked
- **Half-Open**: Testing one request to check if service recovered

### Key Parameters

| Parameter | Description | Typical value for trading |
|-----------|-------------|--------------------------|
| `failure_threshold` | Failures before opening | 3-5 |
| `success_threshold` | Successes to recover | 2-3 |
| `timeout` | Wait time in open state | 30-60 sec |

## Basic Implementation

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
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

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0; // Reset failure counter
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    println!("Circuit CLOSED: Service recovered");
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                    println!("Circuit OPEN: Too many failures!");
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
                println!("Circuit OPEN: Test failed");
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.state
    }
}

fn main() {
    let mut cb = CircuitBreaker::new(
        3,                      // 3 failures to open
        2,                      // 2 successes to recover
        Duration::from_secs(5), // 5 seconds timeout
    );

    // Simulation
    println!("State: {:?}", cb.state());

    // Simulate failures
    for i in 1..=4 {
        if cb.can_execute() {
            println!("Attempt {}: executing request...", i);
            cb.record_failure();
        } else {
            println!("Attempt {}: Circuit open, skipping", i);
        }
        println!("State: {:?}\n", cb.state());
    }
}
```

## Exchange API Application

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
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

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }
}

#[derive(Debug)]
enum ExchangeError {
    Timeout,
    RateLimit,
    ServerError,
    CircuitOpen,
}

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
struct OrderResult {
    order_id: String,
    status: String,
}

struct ExchangeClient {
    circuit_breaker: CircuitBreaker,
    request_count: u32,
}

impl ExchangeClient {
    fn new() -> Self {
        ExchangeClient {
            circuit_breaker: CircuitBreaker::new(
                3,                       // 3 failures — open
                2,                       // 2 successes — recover
                Duration::from_secs(30), // 30 seconds wait
            ),
            request_count: 0,
        }
    }

    fn place_order(&mut self, order: &Order) -> Result<OrderResult, ExchangeError> {
        // Check circuit breaker state
        if !self.circuit_breaker.can_execute() {
            return Err(ExchangeError::CircuitOpen);
        }

        // Execute request (simulation)
        let result = self.execute_request(order);

        // Update circuit breaker state
        match &result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(_) => self.circuit_breaker.record_failure(),
        }

        result
    }

    fn execute_request(&mut self, order: &Order) -> Result<OrderResult, ExchangeError> {
        self.request_count += 1;

        // Simulation: every 3rd request fails
        if self.request_count % 3 == 0 {
            Err(ExchangeError::Timeout)
        } else {
            Ok(OrderResult {
                order_id: format!("ORD-{}", self.request_count),
                status: "FILLED".to_string(),
            })
        }
    }

    fn get_circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state
    }
}

fn main() {
    let mut client = ExchangeClient::new();

    let order = Order {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.1,
        price: 42000.0,
    };

    // Attempt to place orders
    for i in 1..=10 {
        println!("--- Attempt {} ---", i);
        println!("Circuit state: {:?}", client.get_circuit_state());

        match client.place_order(&order) {
            Ok(result) => println!("Success: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }
        println!();
    }
}
```

## Circuit Breaker with Metrics

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
struct CircuitMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    rejected_requests: u64,  // Rejected due to open circuit
    state_changes: Vec<(Instant, CircuitState)>,
    response_times: VecDeque<Duration>,  // Sliding window
}

impl CircuitMetrics {
    fn new() -> Self {
        CircuitMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rejected_requests: 0,
            state_changes: Vec::new(),
            response_times: VecDeque::with_capacity(100),
        }
    }

    fn record_request(&mut self, success: bool, response_time: Duration) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // Keep last 100 response times
        if self.response_times.len() >= 100 {
            self.response_times.pop_front();
        }
        self.response_times.push_back(response_time);
    }

    fn record_rejection(&mut self) {
        self.rejected_requests += 1;
    }

    fn record_state_change(&mut self, new_state: CircuitState) {
        self.state_changes.push((Instant::now(), new_state));
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    fn avg_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.response_times.iter().sum();
        total / self.response_times.len() as u32
    }
}

struct MonitoredCircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
    metrics: CircuitMetrics,
}

impl MonitoredCircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        MonitoredCircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
            metrics: CircuitMetrics::new(),
        }
    }

    fn execute<F, T, E>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Check if we can execute
        if !self.can_execute() {
            self.metrics.record_rejection();
            return Err(CircuitBreakerError::CircuitOpen);
        }

        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        match &result {
            Ok(_) => {
                self.metrics.record_request(true, elapsed);
                self.record_success();
                result.map_err(CircuitBreakerError::ServiceError)
            }
            Err(_) => {
                self.metrics.record_request(false, elapsed);
                self.record_failure();
                result.map_err(CircuitBreakerError::ServiceError)
            }
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.transition_to(CircuitState::HalfOpen);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn transition_to(&mut self, new_state: CircuitState) {
        if self.state != new_state {
            self.state = new_state;
            self.metrics.record_state_change(new_state);
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.transition_to(CircuitState::Closed);
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.transition_to(CircuitState::Open);
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.transition_to(CircuitState::Open);
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn get_metrics(&self) -> &CircuitMetrics {
        &self.metrics
    }

    fn print_status(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║       CIRCUIT BREAKER STATUS          ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ State:          {:>20?} ║", self.state);
        println!("║ Total requests: {:>20} ║", self.metrics.total_requests);
        println!("║ Successful:     {:>20} ║", self.metrics.successful_requests);
        println!("║ Failed:         {:>20} ║", self.metrics.failed_requests);
        println!("║ Rejected:       {:>20} ║", self.metrics.rejected_requests);
        println!("║ Success rate:   {:>19.1}% ║", self.metrics.success_rate());
        println!("║ Avg response:   {:>17.2?} ║", self.metrics.avg_response_time());
        println!("╚═══════════════════════════════════════╝");
    }
}

#[derive(Debug)]
enum CircuitBreakerError<E> {
    CircuitOpen,
    ServiceError(E),
}

fn main() {
    let mut cb = MonitoredCircuitBreaker::new(
        3,
        2,
        Duration::from_secs(5),
    );

    // Simulate requests
    let mut fail_next = false;
    for i in 1..=15 {
        let result: Result<String, &str> = cb.execute(|| {
            if i % 4 == 0 {
                Err("Timeout")
            } else {
                Ok(format!("Response {}", i))
            }
        });

        match result {
            Ok(v) => println!("#{}: Success - {}", i, v),
            Err(CircuitBreakerError::CircuitOpen) => println!("#{}: CIRCUIT OPEN", i),
            Err(CircuitBreakerError::ServiceError(e)) => println!("#{}: Error - {}", i, e),
        }
    }

    println!();
    cb.print_status();
}
```

## Circuit Breaker for Multiple Services

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
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

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.state
    }
}

/// Circuit Breaker manager for multiple services
struct CircuitBreakerRegistry {
    breakers: HashMap<String, CircuitBreaker>,
    default_failure_threshold: u32,
    default_success_threshold: u32,
    default_timeout: Duration,
}

impl CircuitBreakerRegistry {
    fn new() -> Self {
        CircuitBreakerRegistry {
            breakers: HashMap::new(),
            default_failure_threshold: 5,
            default_success_threshold: 2,
            default_timeout: Duration::from_secs(30),
        }
    }

    fn register(&mut self, name: &str, failure_threshold: u32, success_threshold: u32, timeout: Duration) {
        self.breakers.insert(
            name.to_string(),
            CircuitBreaker::new(failure_threshold, success_threshold, timeout),
        );
    }

    fn get_or_create(&mut self, name: &str) -> &mut CircuitBreaker {
        if !self.breakers.contains_key(name) {
            self.breakers.insert(
                name.to_string(),
                CircuitBreaker::new(
                    self.default_failure_threshold,
                    self.default_success_threshold,
                    self.default_timeout,
                ),
            );
        }
        self.breakers.get_mut(name).unwrap()
    }

    fn print_all_states(&self) {
        println!("╔════════════════════════════════════════════╗");
        println!("║         CIRCUIT BREAKER REGISTRY           ║");
        println!("╠════════════════════╦═══════════════════════╣");
        println!("║ Service            ║ State                 ║");
        println!("╠════════════════════╬═══════════════════════╣");
        for (name, cb) in &self.breakers {
            println!("║ {:18} ║ {:21?} ║", name, cb.state());
        }
        println!("╚════════════════════╩═══════════════════════╝");
    }
}

/// Trading system with multiple exchanges
struct TradingSystem {
    registry: CircuitBreakerRegistry,
}

impl TradingSystem {
    fn new() -> Self {
        let mut registry = CircuitBreakerRegistry::new();

        // Different settings for different exchanges
        registry.register("binance", 3, 2, Duration::from_secs(30));
        registry.register("bybit", 5, 3, Duration::from_secs(60));
        registry.register("kraken", 4, 2, Duration::from_secs(45));

        TradingSystem { registry }
    }

    fn execute_on_exchange<F, T>(&mut self, exchange: &str, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let cb = self.registry.get_or_create(exchange);

        if !cb.can_execute() {
            return Err(format!("{} circuit is OPEN", exchange));
        }

        let result = operation();

        match &result {
            Ok(_) => cb.record_success(),
            Err(_) => cb.record_failure(),
        }

        result
    }

    fn get_price(&mut self, exchange: &str, symbol: &str) -> Result<f64, String> {
        self.execute_on_exchange(exchange, || {
            // Simulate API call
            match (exchange, symbol) {
                ("binance", "BTC/USDT") => Ok(42000.0),
                ("bybit", "BTC/USDT") => Ok(42010.0),
                _ => Err("Symbol not found".to_string()),
            }
        })
    }

    fn print_status(&self) {
        self.registry.print_all_states();
    }
}

fn main() {
    let mut system = TradingSystem::new();

    // Get prices from different exchanges
    let exchanges = ["binance", "bybit", "kraken"];
    let symbol = "BTC/USDT";

    for exchange in &exchanges {
        match system.get_price(exchange, symbol) {
            Ok(price) => println!("{}: {} = ${:.2}", exchange, symbol, price),
            Err(e) => println!("{}: Error - {}", exchange, e),
        }
    }

    println!();
    system.print_status();
}
```

## Practical Exercises

### Exercise 1: Circuit Breaker with Exponential Backoff

Add logic where timeout increases with each transition to Open state:
- 1st time: 30 sec
- 2nd time: 60 sec
- 3rd time: 120 sec
- Maximum: 5 minutes

### Exercise 2: Sliding Window Circuit Breaker

Instead of a simple error counter, implement a Circuit Breaker based on a sliding window:
- Store results of the last N requests (e.g., 10)
- If error percentage exceeds threshold (e.g., 50%), open the circuit

### Exercise 3: Circuit Breaker for WebSocket

Adapt Circuit Breaker for WebSocket connections:
- Track connection loss
- Auto-reconnect with backoff
- Switch to backup endpoint on prolonged failure

### Exercise 4: Health Check Endpoint

Add a special health check to Circuit Breaker:
- In HalfOpen state, send health check request first, not a real one
- Only if health check succeeds, allow real requests

## Homework

1. **Implement Bulkhead pattern**: Limit concurrent requests to each service (e.g., max 10 concurrent requests)

2. **Add alerting**: Send notification when transitioning to Open state (simulate with print)

3. **Create Fallback mechanism**: If primary exchange is unavailable, automatically switch to backup

4. **Implement Retry with Circuit Breaker**: Before opening circuit, make 3 retry attempts with exponential backoff

## What We Learned

| Concept | Description |
|---------|-------------|
| Circuit Breaker | Pattern for cascade failure protection |
| Three states | Closed → Open → HalfOpen → Closed |
| failure_threshold | Number of failures to open |
| success_threshold | Number of successes to recover |
| timeout | Wait time before testing |
| Metrics | Monitoring state and performance |
| Registry | Managing multiple Circuit Breakers |

## Navigation

[← Previous day](../108-retry-patterns/en.md) | [Next day →](../110-fallback-strategies/en.md)
