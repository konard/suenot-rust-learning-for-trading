# Day 200: HTTP Client: Connection Pooling

## Trading Analogy

Imagine you're trading on multiple exchanges simultaneously: Binance, Kraken, Coinbase. Every time you need to check a price or send an order, you call the broker, wait for them to pick up, introduce yourself, provide your account number... and only then get the information. This is very slow!

**Connection pooling** is like having dedicated lines to each exchange. The lines are already connected, authentication is complete, and when you need data — you get an immediate response without lengthy greetings. After use, the line isn't disconnected but returns to the pool for the next request.

In the HTTP world, this works similarly:
- Creating a TCP connection takes time (TCP handshake)
- TLS handshake for HTTPS is even more expensive
- Connection pooling allows reusing connections

## What is a Connection Pool?

A connection pool is a set of pre-established connections to a server that can be reused for multiple requests.

```
Without pool (each request):
┌────────┐    connect    ┌────────┐
│ Client │──────────────>│ Server │
└────────┘   request     └────────┘
    │        response        │
    │<───────────────────────│
    │        close           │
    X────────────────────────X

With pool (multiple requests):
┌────────┐    connect    ┌────────┐
│ Client │══════════════>│ Server │  <- Connection stays open
└────────┘   request 1   └────────┘
    │        response 1      │
    │<═══════════════════════│
    │        request 2       │
    │════════════════════════>
    │        response 2      │
    │<═══════════════════════│
    │        request N       │
    │════════════════════════>
    │        response N      │
    │<═══════════════════════│
```

## Basic Example with reqwest

reqwest automatically uses connection pooling:

```rust
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client contains a connection pool inside
    // IMPORTANT: create once and reuse
    let client = Client::new();

    // Get prices from exchange 10 times
    // All requests use the same connections from the pool
    for i in 1..=10 {
        let response = client
            .get("https://api.binance.com/api/v3/ticker/price")
            .query(&[("symbol", "BTCUSDT")])
            .send()
            .await?;

        let price: serde_json::Value = response.json().await?;
        println!("Request {}: BTC = {}", i, price["price"]);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
```

## Anti-pattern: Creating Client in a Loop

```rust
// BAD: creating client for each request
async fn bad_example() {
    for _ in 0..10 {
        // New connection pool created each time
        let client = Client::new();
        let _ = client.get("https://api.exchange.com/price").send().await;
        // client dropped, connection closed
    }
}

// GOOD: one client for all requests
async fn good_example() {
    let client = Client::new();
    for _ in 0..10 {
        // Reusing connections from the pool
        let _ = client.get("https://api.exchange.com/price").send().await;
    }
}
```

## Configuring the Connection Pool

```rust
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

fn create_trading_client() -> Client {
    ClientBuilder::new()
        // Maximum connections per host
        .pool_max_idle_per_host(10)
        // Idle connection timeout
        .pool_idle_timeout(Duration::from_secs(90))
        // Connection timeout
        .connect_timeout(Duration::from_secs(5))
        // Total request timeout
        .timeout(Duration::from_secs(30))
        // Enable HTTP/2 for multiplexing
        .http2_prior_knowledge()
        .build()
        .expect("Failed to create HTTP client")
}
```

## Example: Multi-Exchange Client

```rust
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct ExchangeClient {
    client: Client,
    // Price cache
    price_cache: Arc<RwLock<HashMap<String, f64>>>,
}

impl ExchangeClient {
    fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .pool_idle_timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();

        ExchangeClient {
            client,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_binance_price(&self, symbol: &str) -> Result<f64, reqwest::Error> {
        let resp = self.client
            .get("https://api.binance.com/api/v3/ticker/price")
            .query(&[("symbol", symbol)])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let price: f64 = resp["price"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        // Update cache
        let mut cache = self.price_cache.write().await;
        cache.insert(symbol.to_string(), price);

        Ok(price)
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> HashMap<String, f64> {
        let mut results = HashMap::new();

        // Parallel requests through one connection pool
        let futures: Vec<_> = symbols.iter().map(|symbol| {
            let client = self.clone();
            let symbol = symbol.to_string();
            tokio::spawn(async move {
                (symbol.clone(), client.get_binance_price(&symbol).await)
            })
        }).collect();

        for future in futures {
            if let Ok((symbol, Ok(price))) = future.await {
                results.insert(symbol, price);
            }
        }

        results
    }
}

#[tokio::main]
async fn main() {
    let client = ExchangeClient::new();

    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "XRPUSDT"];

    let prices = client.get_multiple_prices(&symbols).await;

    println!("Current prices:");
    for (symbol, price) in &prices {
        println!("  {}: ${:.2}", symbol, price);
    }
}
```

## Monitoring Pool Usage

```rust
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct MonitoredClient {
    client: Client,
    request_count: AtomicU64,
    error_count: AtomicU64,
}

impl MonitoredClient {
    fn new() -> Arc<Self> {
        Arc::new(MonitoredClient {
            client: Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .unwrap(),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        })
    }

    async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        self.request_count.fetch_add(1, Ordering::SeqCst);

        match self.client.get(url).send().await {
            Ok(resp) => Ok(resp.text().await?),
            Err(e) => {
                self.error_count.fetch_add(1, Ordering::SeqCst);
                Err(e)
            }
        }
    }

    fn stats(&self) -> (u64, u64) {
        (
            self.request_count.load(Ordering::SeqCst),
            self.error_count.load(Ordering::SeqCst)
        )
    }
}

#[tokio::main]
async fn main() {
    let client = MonitoredClient::new();

    // Simulate load
    let mut handles = vec![];

    for _ in 0..100 {
        let c = Arc::clone(&client);
        handles.push(tokio::spawn(async move {
            let _ = c.get("https://httpbin.org/get").await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    let (total, errors) = client.stats();
    println!("Total requests: {}, Errors: {}", total, errors);
    println!("Success rate: {}%", (total - errors) * 100 / total);
}
```

## HTTP/2 and Multiplexing

HTTP/2 allows sending multiple requests through a single connection:

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client with HTTP/2
    let client = Client::builder()
        .http2_prior_knowledge()
        .build()?;

    // All requests go through one HTTP/2 connection
    // with multiplexing
    let futures = (0..10).map(|i| {
        let client = client.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let _ = client
                .get("https://nghttp2.org/httpbin/get")
                .send()
                .await;
            println!("Request {}: {:?}", i, start.elapsed());
        })
    });

    futures::future::join_all(futures).await;

    Ok(())
}
```

## Practical Example: Arbitrage Scanner

```rust
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PriceData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: Instant,
}

struct ArbitrageScanner {
    client: Client,
    prices: Arc<Mutex<HashMap<String, Vec<PriceData>>>>,
}

impl ArbitrageScanner {
    fn new() -> Self {
        ArbitrageScanner {
            client: Client::builder()
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(120))
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            prices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn fetch_binance_price(&self, symbol: &str) -> Option<PriceData> {
        let url = format!(
            "https://api.binance.com/api/v3/ticker/bookTicker?symbol={}",
            symbol
        );

        let resp = self.client.get(&url).send().await.ok()?;
        let data: serde_json::Value = resp.json().await.ok()?;

        Some(PriceData {
            exchange: "Binance".to_string(),
            symbol: symbol.to_string(),
            bid: data["bidPrice"].as_str()?.parse().ok()?,
            ask: data["askPrice"].as_str()?.parse().ok()?,
            timestamp: Instant::now(),
        })
    }

    async fn scan_symbol(&self, symbol: &str) {
        // Single client (single pool) for all exchanges
        if let Some(price) = self.fetch_binance_price(symbol).await {
            let mut prices = self.prices.lock().await;
            prices.entry(symbol.to_string())
                .or_insert_with(Vec::new)
                .push(price);
        }
    }

    async fn find_arbitrage(&self) -> Vec<String> {
        let prices = self.prices.lock().await;
        let mut opportunities = Vec::new();

        for (symbol, price_list) in prices.iter() {
            if price_list.len() < 2 {
                continue;
            }

            for i in 0..price_list.len() {
                for j in (i + 1)..price_list.len() {
                    let spread = (price_list[i].ask - price_list[j].bid)
                        / price_list[j].bid * 100.0;

                    if spread.abs() > 0.5 {
                        opportunities.push(format!(
                            "{}: {} bid={:.2}, {} ask={:.2}, spread={:.2}%",
                            symbol,
                            price_list[i].exchange, price_list[i].bid,
                            price_list[j].exchange, price_list[j].ask,
                            spread
                        ));
                    }
                }
            }
        }

        opportunities
    }
}

#[tokio::main]
async fn main() {
    let scanner = ArbitrageScanner::new();

    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"];

    // Scan all symbols in parallel
    let mut handles = vec![];
    for symbol in &symbols {
        let s = scanner.client.clone();
        let symbol = symbol.to_string();
        handles.push(tokio::spawn({
            let scanner_ref = &scanner;
            async move {
                // Using the same connection pool here
            }
        }));
    }

    // Simple demonstration
    for symbol in &symbols {
        scanner.scan_symbol(symbol).await;
        println!("Scanned: {}", symbol);
    }

    println!("\nSearching for arbitrage opportunities...");
    let opportunities = scanner.find_arbitrage().await;

    if opportunities.is_empty() {
        println!("No arbitrage opportunities found");
    } else {
        for opp in opportunities {
            println!("  {}", opp);
        }
    }
}
```

## Testing with Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_connection_pooling() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/price"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"price": "42000.00"}))
            )
            .expect(100) // Expect 100 requests
            .mount(&mock_server)
            .await;

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .build()
            .unwrap();

        let url = format!("{}/api/price", mock_server.uri());

        // 100 requests through 5 connections in the pool
        let mut handles = vec![];
        for _ in 0..100 {
            let client = client.clone();
            let url = url.clone();
            handles.push(tokio::spawn(async move {
                client.get(&url).send().await.unwrap()
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Connection Pool | Set of reusable connections |
| `Client::new()` | Creates client with default pool |
| `pool_max_idle_per_host` | Maximum idle connections per host |
| `pool_idle_timeout` | Idle connection lifetime |
| HTTP/2 | Request multiplexing in single connection |
| Client reuse | Key to efficient connection pooling |

## Homework

1. **Pool Benchmark**: Write a program that compares performance of:
   - Creating a new client for each request
   - Using one client with a pool

   Measure time for 100 requests in each case.

2. **Multi-Exchange Client**: Create a `MultiExchangeClient` struct that:
   - Uses one `Client` for all exchanges
   - Has methods `get_binance_price()`, `get_kraken_price()` (can use mocks)
   - Queries all exchanges in parallel

3. **Pool Monitoring**: Implement a wrapper around `Client` that:
   - Counts active requests
   - Logs time for each request
   - Shows statistics per host

4. **Graceful Degradation**: Create a client that:
   - Has timeouts at each level (connect, read, total)
   - Returns last cached value on error
   - Tracks success/failure statistics

## Navigation

[← Previous day](../199-http-headers-api-auth/en.md) | [Next day →](../201-rate-limiting-throttling/en.md)
