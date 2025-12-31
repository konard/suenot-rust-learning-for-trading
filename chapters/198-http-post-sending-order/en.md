# Day 198: HTTP POST: Sending Order

## Trading Analogy

Think of a GET request as looking at quotes on your terminal screen: you receive information but don't change anything. A POST request is like pressing the "Buy" or "Sell" button: you send data to the server, and it performs an action — creates an order, modifies your balance, affects the market.

In real trading, POST requests are used for:
- Placing orders (limit, market, stop orders)
- Canceling existing orders
- Transferring funds between accounts
- Modifying position settings (stop-loss, take-profit)

## What is HTTP POST?

HTTP POST is a method used to send data to a server. Unlike GET:

| GET | POST |
|-----|------|
| Retrieves data | Sends data |
| Data in URL | Data in request body |
| Cacheable | Not cacheable |
| Safe (doesn't change state) | Unsafe (changes state) |
| Idempotent | Non-idempotent |

## The reqwest Library for POST Requests

```rust
// Cargo.toml
// [dependencies]
// reqwest = { version = "0.11", features = ["json"] }
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OrderRequest {
    symbol: String,
    side: String,       // "buy" or "sell"
    order_type: String, // "market" or "limit"
    quantity: f64,
    price: Option<f64>, // Only for limit orders
}

#[derive(Debug, Deserialize)]
struct OrderResponse {
    order_id: String,
    status: String,
    filled_quantity: f64,
    average_price: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let order = OrderRequest {
        symbol: "BTC/USDT".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        quantity: 0.001,
        price: Some(42000.0),
    };

    let response = client
        .post("https://api.example.com/orders")
        .json(&order)
        .send()
        .await?;

    println!("Response status: {}", response.status());

    let order_response: OrderResponse = response.json().await?;
    println!("Order created: {:?}", order_response);

    Ok(())
}
```

## Data Formats in POST Requests

### 1. JSON (most common)

```rust
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct MarketOrder {
    symbol: String,
    side: String,
    quantity: f64,
}

async fn place_market_order(client: &Client, order: &MarketOrder) -> Result<String, reqwest::Error> {
    let response = client
        .post("https://api.exchange.com/v1/order")
        .header("Content-Type", "application/json")
        .json(order)
        .send()
        .await?;

    response.text().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let order = MarketOrder {
        symbol: "ETH/USDT".to_string(),
        side: "sell".to_string(),
        quantity: 0.5,
    };

    let result = place_market_order(&client, &order).await?;
    println!("Server response: {}", result);

    Ok(())
}
```

### 2. Form URL-encoded

```rust
use reqwest::Client;
use std::collections::HashMap;

async fn place_order_form(client: &Client) -> Result<String, reqwest::Error> {
    let mut params = HashMap::new();
    params.insert("symbol", "BTC/USDT");
    params.insert("side", "buy");
    params.insert("quantity", "0.001");

    let response = client
        .post("https://api.exchange.com/v1/order")
        .form(&params)
        .send()
        .await?;

    response.text().await
}
```

## Working with Authentication Headers

Most trading APIs require authentication:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LimitOrder {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    time_in_force: String,
}

#[derive(Debug, Deserialize)]
struct OrderResult {
    order_id: u64,
    client_order_id: String,
    status: String,
}

async fn place_authenticated_order(
    client: &Client,
    api_key: &str,
    api_secret: &str,
    order: &LimitOrder,
) -> Result<OrderResult, Box<dyn std::error::Error>> {
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Create signature (simplified example)
    let signature = create_signature(api_secret, order, timestamp);

    let response = client
        .post("https://api.exchange.com/v1/order")
        .header("X-API-KEY", api_key)
        .header("X-SIGNATURE", signature)
        .header("X-TIMESTAMP", timestamp.to_string())
        .json(order)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        let error_text = response.text().await?;
        Err(format!("API error: {}", error_text).into())
    }
}

fn create_signature(secret: &str, order: &LimitOrder, timestamp: i64) -> String {
    // In reality, HMAC-SHA256 is used here
    format!("signature_placeholder_{}_{}", secret, timestamp)
}
```

## Error Handling for POST Requests

```rust
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
enum OrderError {
    #[error("Insufficient funds: need {required}, have {available}")]
    InsufficientFunds { required: f64, available: f64 },

    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Server error: {0}")]
    ServerError(String),
}

#[derive(Serialize)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Deserialize)]
struct ApiError {
    code: i32,
    message: String,
}

async fn place_order_with_error_handling(
    client: &Client,
    order: &Order,
) -> Result<String, OrderError> {
    let response = client
        .post("https://api.exchange.com/v1/order")
        .json(order)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK | StatusCode::CREATED => {
            Ok(response.text().await?)
        }
        StatusCode::BAD_REQUEST => {
            let error: ApiError = response.json().await?;
            match error.code {
                -1013 => Err(OrderError::InsufficientFunds {
                    required: order.quantity * order.price,
                    available: 0.0, // Should get from response
                }),
                -1121 => Err(OrderError::InvalidSymbol(order.symbol.clone())),
                _ => Err(OrderError::OrderRejected(error.message)),
            }
        }
        StatusCode::UNAUTHORIZED => {
            Err(OrderError::OrderRejected("Invalid API key".to_string()))
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Err(OrderError::OrderRejected("Rate limit exceeded".to_string()))
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(OrderError::ServerError(format!("{}: {}", status, body)))
        }
    }
}
```

## Practical Example: Trading Client

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
struct TradingClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Serialize)]
struct CreateOrderRequest {
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct CreateOrderResponse {
    order_id: String,
    symbol: String,
    status: String,
    side: String,
    price: f64,
    quantity: f64,
    filled_quantity: f64,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct CancelOrderRequest {
    order_id: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct CancelOrderResponse {
    order_id: String,
    status: String,
}

impl TradingClient {
    fn new(base_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        TradingClient {
            client,
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
    ) -> Result<CreateOrderResponse, Box<dyn std::error::Error>> {
        let request = CreateOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
        };

        self.send_order(&request).await
    }

    async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
    ) -> Result<CreateOrderResponse, Box<dyn std::error::Error>> {
        let request = CreateOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
        };

        self.send_order(&request).await
    }

    async fn place_stop_loss(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        stop_price: f64,
    ) -> Result<CreateOrderResponse, Box<dyn std::error::Error>> {
        let request = CreateOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::StopLoss,
            quantity,
            price: None,
            stop_price: Some(stop_price),
        };

        self.send_order(&request).await
    }

    async fn send_order(
        &self,
        request: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/orders", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status();
            let body = response.text().await?;
            Err(format!("Error {}: {}", status, body).into())
        }
    }

    async fn cancel_order(
        &self,
        order_id: &str,
        symbol: &str,
    ) -> Result<CancelOrderResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/orders/cancel", self.base_url);

        let request = CancelOrderRequest {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status();
            let body = response.text().await?;
            Err(format!("Cancel error {}: {}", status, body).into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TradingClient::new(
        "https://api.exchange.com",
        "your_api_key_here",
    );

    // Example 1: Market order to buy
    println!("Placing market buy order for BTC...");
    match client.place_market_order("BTC/USDT", OrderSide::Buy, 0.001).await {
        Ok(order) => println!("Order created: {:?}", order),
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: Limit order to sell
    println!("\nPlacing limit sell order for ETH...");
    match client.place_limit_order("ETH/USDT", OrderSide::Sell, 0.1, 2500.0).await {
        Ok(order) => {
            println!("Limit order created: {:?}", order);

            // Example 3: Cancel order
            println!("\nCanceling order...");
            match client.cancel_order(&order.order_id, "ETH/USDT").await {
                Ok(result) => println!("Order canceled: {:?}", result),
                Err(e) => println!("Cancel error: {}", e),
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 4: Stop-loss
    println!("\nSetting stop-loss...");
    match client.place_stop_loss("BTC/USDT", OrderSide::Sell, 0.001, 40000.0).await {
        Ok(order) => println!("Stop-loss set: {:?}", order),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
```

## Retry Logic on Errors

```rust
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

async fn post_with_retry<T: Serialize>(
    client: &Client,
    url: &str,
    body: &T,
    max_retries: u32,
) -> Result<String, String> {
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        println!("Attempt {} of {}", attempt, max_retries);

        match client.post(url).json(body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return response.text().await.map_err(|e| e.to_string());
                }

                let status = response.status();

                // No point retrying for certain errors
                if status.as_u16() == 400 || status.as_u16() == 401 {
                    return Err(format!("Unrecoverable error: {}", status));
                }

                last_error = format!("HTTP error: {}", status);
            }
            Err(e) => {
                last_error = format!("Network error: {}", e);
            }
        }

        if attempt < max_retries {
            // Exponential backoff
            let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
            println!("Waiting {:?} before retry...", delay);
            sleep(delay).await;
        }
    }

    Err(format!(
        "All {} attempts failed. Last error: {}",
        max_retries, last_error
    ))
}
```

## Batch Order Submission

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::future::join_all;

#[derive(Debug, Clone, Serialize)]
struct BatchOrder {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug, Deserialize)]
struct BatchOrderResult {
    order_id: String,
    status: String,
}

async fn place_batch_orders(
    client: &Client,
    orders: Vec<BatchOrder>,
) -> Vec<Result<BatchOrderResult, String>> {
    let futures: Vec<_> = orders
        .into_iter()
        .map(|order| {
            let client = client.clone();
            async move {
                let response = client
                    .post("https://api.exchange.com/v1/order")
                    .json(&order)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                if response.status().is_success() {
                    response
                        .json::<BatchOrderResult>()
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    Err(format!("Error: {}", response.status()))
                }
            }
        })
        .collect();

    join_all(futures).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Create a grid of limit orders
    let orders: Vec<BatchOrder> = (0..5)
        .map(|i| BatchOrder {
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            quantity: 0.001,
            price: 41000.0 - (i as f64 * 100.0),
        })
        .collect();

    println!("Sending {} orders...", orders.len());

    let results = place_batch_orders(&client, orders).await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(order) => println!("Order {}: {} ({})", i + 1, order.order_id, order.status),
            Err(e) => println!("Order {}: Error - {}", i + 1, e),
        }
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| HTTP POST | Method for sending data to a server |
| JSON body | Serializing data to JSON for transmission |
| Form data | Alternative data format |
| Headers | Content-Type, Authorization, and others |
| Error handling | Checking status and parsing API errors |
| Retry logic | Exponential backoff on errors |
| Batch requests | Sending multiple orders in parallel |

## Homework

1. **Simple Trading Bot**: Write a program that:
   - Gets the current BTC price (GET request)
   - If the price is below a threshold — creates a buy order (POST request)
   - Logs all actions to the console

2. **Error Handling**: Extend the trading client from the example by adding:
   - Balance check before placing an order
   - Rate limit handling (HTTP 429)
   - Automatic retry on temporary errors

3. **Order Grid**: Implement a `create_order_grid` function that:
   - Takes a center price, step size, and number of levels
   - Creates limit buy orders below and sell orders above the center price
   - Returns a list of created orders

4. **WebSocket Notifications**: Add to the trading client:
   - Order status tracking after placement
   - Notification when an order is filled
   - Order cancellation by timeout mechanism

## Navigation

[← Previous day](../197-http-basics-reqwest-get/en.md) | [Next day →](../199-http-headers-api-authorization/en.md)
