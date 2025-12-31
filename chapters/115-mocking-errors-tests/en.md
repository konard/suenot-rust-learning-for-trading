# Day 115: Mocking Errors in Tests

## Trading Analogy

Imagine you're testing your trading system. How do you verify that your system correctly handles exchange failures? You can't wait for a real outage! Instead, you **mock** the error — you create a "fake" data provider that deliberately returns errors.

It's like a fire drill: you don't need to set the building on fire to check if people know where the exit is.

## Why Mock Errors?

1. **Reliability** — ensure the system correctly handles failures
2. **Isolation** — test code without depending on external services
3. **Reproducibility** — guarantee consistent test behavior
4. **Speed** — don't wait for real service timeouts

## Basic Mock with Result

```rust
// Trait for market data provider
trait MarketDataProvider {
    fn get_price(&self, symbol: &str) -> Result<f64, String>;
}

// Real implementation (for production)
struct RealExchange;

impl MarketDataProvider for RealExchange {
    fn get_price(&self, symbol: &str) -> Result<f64, String> {
        // In reality, this would be an HTTP request to exchange API
        Ok(42000.0)
    }
}

// Mock that always returns an error
struct FailingExchange {
    error_message: String,
}

impl MarketDataProvider for FailingExchange {
    fn get_price(&self, _symbol: &str) -> Result<f64, String> {
        Err(self.error_message.clone())
    }
}

// Function we're testing
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
    // Demo with real provider
    let exchange = RealExchange;
    let holdings = vec![("BTC", 0.5)];
    println!("Portfolio: {:?}", calculate_portfolio_value(&exchange, &holdings));

    // Demo with error mock
    let failing = FailingExchange {
        error_message: String::from("API rate limit exceeded"),
    };
    println!("With error: {:?}", calculate_portfolio_value(&failing, &holdings));
}
```

## Configurable Mocks

```rust
// Mock with configurable behavior
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

        // BTC works
        assert!(mock.get_price("BTC").is_ok());

        // ETH returns error
        let eth_result = mock.get_price("ETH");
        assert!(eth_result.is_err());
        assert!(eth_result.unwrap_err().contains("delisted"));

        // Unknown symbol
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

## Mock with Response Sequence

```rust
use std::cell::RefCell;

// Mock that returns different results on each call
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

// Function with retry logic
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
                // In reality, there would be a delay here
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
    // Simulate temporary failure
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

## Testing Network Error Handling

```rust
// Network error types
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

// Handler with different logic for different errors
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
    // Test different error types
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

## Mock for Testing Trading Operations

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

// Mock with call recording for verification
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
        // Record the call
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

// Trading strategy
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

        // Verify order was sent with correct parameters
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
    // Demo successful order
    let success_mock = OrderExecutorMock::new_success();
    println!("Success: {:?}", execute_strategy(&success_mock, "BUY", "BTC/USDT", 42000.0));

    // Demo balance error
    let fail_mock = OrderExecutorMock::new_failing(OrderError::InsufficientBalance);
    println!("Failure: {:?}", execute_strategy(&fail_mock, "BUY", "BTC/USDT", 42000.0));

    // Check recorded calls
    println!("Recorded calls: {:?}", fail_mock.get_calls());
}
```

## Pattern: Stateful Mock

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

// Mock with mutable state
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
            // Update state on successful check
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

        // First trade - OK
        assert!(mock.check_trade("ETH", 3000.0).is_ok());
        assert_eq!(mock.get_exposure("ETH"), 3000.0);

        // Second trade - OK
        assert!(mock.check_trade("ETH", 3000.0).is_ok());
        assert_eq!(mock.get_exposure("ETH"), 6000.0);

        // Third trade - limit exceeded
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

## What We Learned

| Pattern | Description | When to Use |
|---------|-------------|-------------|
| Simple mock | Always returns error | Basic error handling test |
| Configurable | Different responses for different inputs | Partial failures |
| Sequential | Different response on each call | Retry logic testing |
| Recording | Saves all calls | Parameter verification |
| Stateful | Changes between calls | Complex scenarios |

## Practice Exercises

1. Create an exchange API mock that returns `RateLimited` on every 5th request

2. Implement a mock with timeout behavior: first N requests "hang" (return `Timeout`), then work normally

3. Write a balance checking mock that tracks history of all requests and allows verifying their sequence

4. Create a mock that simulates partial order execution (order for 10 BTC executes only 7 BTC and returns `PartialFill` error)

## Homework

1. Implement a complete mock system for testing a trading bot:
   - Mock for market data (prices, orderbooks)
   - Mock for order execution (with different error types)
   - Mock for risk management

2. Write tests for the scenario "exchange goes down during trading session":
   - Open positions should be protected
   - New orders should be rejected
   - System should log the problem

3. Create a mock that simulates "split-brain" (different exchange nodes return different prices) and write a test verifying the system detects this discrepancy

## Navigation

[← Previous day](../114-testing-errors-handling/en.md) | [Next day →](../116-documenting-errors/en.md)
