# Day 199: HTTP Headers: API Authorization

## Trading Analogy

Imagine you're arriving at an exchange to trade. Before you're allowed to make any trades, security will check your pass — a document confirming your identity and access rights. Without a pass, you won't get past the reception area.

HTTP headers work exactly the same way. When your trading bot sends a request to an exchange API:
- **API key** — is your "pass", verifying your identity
- **Request signature** — is the "seal", confirming the request is actually from you
- **Timestamp** — is the "date on the pass", protecting against replay attacks with old requests

Without the correct headers, the exchange will simply reject your request — just like security won't let someone in without proper documents.

## What are HTTP Headers?

HTTP headers are metadata sent along with a request or response. They contain additional information:

```
GET /api/v1/account/balance HTTP/1.1
Host: api.exchange.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
X-API-KEY: your-api-key-here
X-TIMESTAMP: 1704067200000
Content-Type: application/json
```

### Key Headers for Trading

| Header | Purpose |
|--------|---------|
| `Authorization` | Authorization token (Bearer, Basic) |
| `X-API-KEY` | Public API key |
| `X-API-SECRET` | Request signature (HMAC) |
| `X-TIMESTAMP` | Timestamp in milliseconds |
| `Content-Type` | Data format (application/json) |

## Types of API Authorization

### 1. API Key in Header

The simplest approach — send the API key in a header:

```rust
use reqwest::header::{HeaderMap, HeaderValue};

fn create_api_key_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-API-KEY",
        HeaderValue::from_str(api_key).unwrap()
    );
    headers
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = "your-exchange-api-key";

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.exchange.com/api/v1/ticker/price")
        .headers(create_api_key_headers(api_key))
        .send()
        .await?;

    println!("Status: {}", response.status());
    println!("Response: {}", response.text().await?);

    Ok(())
}
```

### 2. Bearer Token (OAuth 2.0)

Many modern APIs use Bearer tokens:

```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

fn create_bearer_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).unwrap()
    );
    headers
}

async fn get_portfolio(token: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.broker.com/v1/portfolio")
        .headers(create_bearer_headers(token))
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}
```

### 3. HMAC Signature (Binance-style)

Crypto exchanges often require request signing using HMAC:

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

struct ExchangeClient {
    api_key: String,
    secret_key: String,
    client: reqwest::Client,
}

impl ExchangeClient {
    fn new(api_key: String, secret_key: String) -> Self {
        ExchangeClient {
            api_key,
            secret_key,
            client: reqwest::Client::new(),
        }
    }

    fn get_timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    fn sign(&self, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            HeaderValue::from_str(&self.api_key).unwrap()
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded")
        );
        headers
    }

    async fn get_account_balance(&self) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = Self::get_timestamp();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign(&query);

        let url = format!(
            "https://api.binance.com/api/v3/account?{}&signature={}",
            query, signature
        );

        let response = self.client
            .get(&url)
            .headers(self.create_headers())
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    async fn place_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let timestamp = Self::get_timestamp();

        let params = format!(
            "symbol={}&side={}&type=LIMIT&timeInForce=GTC&quantity={}&price={}&timestamp={}",
            symbol, side, quantity, price, timestamp
        );

        let signature = self.sign(&params);
        let body = format!("{}&signature={}", params, signature);

        let response = self.client
            .post("https://api.binance.com/api/v3/order")
            .headers(self.create_headers())
            .body(body)
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ExchangeClient::new(
        "your-api-key".to_string(),
        "your-secret-key".to_string(),
    );

    // Get balance
    match client.get_account_balance().await {
        Ok(balance) => println!("Balance: {}", balance),
        Err(e) => println!("Error getting balance: {}", e),
    }

    // Place order (example only!)
    // match client.place_order("BTCUSDT", "BUY", 0.001, 40000.0).await {
    //     Ok(order) => println!("Order: {}", order),
    //     Err(e) => println!("Error placing order: {}", e),
    // }

    Ok(())
}
```

## Secure Key Storage

**Never store keys in code!** Use environment variables:

```rust
use std::env;

struct ApiCredentials {
    api_key: String,
    secret_key: String,
}

impl ApiCredentials {
    fn from_env() -> Result<Self, String> {
        let api_key = env::var("EXCHANGE_API_KEY")
            .map_err(|_| "EXCHANGE_API_KEY not set")?;
        let secret_key = env::var("EXCHANGE_SECRET_KEY")
            .map_err(|_| "EXCHANGE_SECRET_KEY not set")?;

        Ok(ApiCredentials { api_key, secret_key })
    }
}

#[tokio::main]
async fn main() {
    // Load .env file (using dotenv)
    dotenv::dotenv().ok();

    match ApiCredentials::from_env() {
        Ok(creds) => {
            println!("Keys loaded successfully");
            println!("API Key: {}...", &creds.api_key[..8]);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Create a .env file with EXCHANGE_API_KEY and EXCHANGE_SECRET_KEY");
        }
    }
}
```

`.env` file:
```
EXCHANGE_API_KEY=your-api-key-here
EXCHANGE_SECRET_KEY=your-secret-key-here
```

## Practical Example: Trading Client

```rust
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
struct TickerPrice {
    symbol: String,
    price: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountInfo {
    balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    asset: String,
    free: String,
    locked: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderResponse {
    symbol: String,
    #[serde(rename = "orderId")]
    order_id: u64,
    status: String,
    #[serde(rename = "executedQty")]
    executed_qty: String,
}

struct TradingClient {
    api_key: String,
    secret_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl TradingClient {
    fn new(api_key: String, secret_key: String) -> Self {
        TradingClient {
            api_key,
            secret_key,
            client: reqwest::Client::new(),
            base_url: "https://api.binance.com".to_string(),
        }
    }

    fn timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    fn sign(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            HeaderValue::from_str(&self.api_key).unwrap()
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded")
        );
        headers
    }

    // Public endpoint (no auth required)
    async fn get_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/api/v3/ticker/price?symbol={}",
            self.base_url, symbol
        );

        let ticker: TickerPrice = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(ticker.price.parse()?)
    }

    // Private endpoint (requires auth)
    async fn get_balances(&self) -> Result<Vec<Balance>, Box<dyn std::error::Error>> {
        let timestamp = Self::timestamp();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign(&query);

        let url = format!(
            "{}/api/v3/account?{}&signature={}",
            self.base_url, query, signature
        );

        let account: AccountInfo = self.client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?
            .json()
            .await?;

        // Filter only non-zero balances
        let non_zero: Vec<Balance> = account.balances
            .into_iter()
            .filter(|b| {
                let free: f64 = b.free.parse().unwrap_or(0.0);
                let locked: f64 = b.locked.parse().unwrap_or(0.0);
                free > 0.0 || locked > 0.0
            })
            .collect();

        Ok(non_zero)
    }

    // Place a limit order
    async fn place_limit_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
    ) -> Result<OrderResponse, Box<dyn std::error::Error>> {
        let timestamp = Self::timestamp();

        let params = format!(
            "symbol={}&side={}&type=LIMIT&timeInForce=GTC&quantity={:.8}&price={:.2}&timestamp={}",
            symbol, side, quantity, price, timestamp
        );

        let signature = self.sign(&params);
        let body = format!("{}&signature={}", params, signature);

        let url = format!("{}/api/v3/order", self.base_url);

        let order: OrderResponse = self.client
            .post(&url)
            .headers(self.auth_headers())
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(order)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let api_key = std::env::var("BINANCE_API_KEY")
        .unwrap_or_else(|_| "demo-key".to_string());
    let secret_key = std::env::var("BINANCE_SECRET_KEY")
        .unwrap_or_else(|_| "demo-secret".to_string());

    let client = TradingClient::new(api_key, secret_key);

    // Get BTC price (public endpoint)
    match client.get_price("BTCUSDT").await {
        Ok(price) => println!("BTC Price: ${:.2}", price),
        Err(e) => println!("Error getting price: {}", e),
    }

    // Get balances (private endpoint)
    match client.get_balances().await {
        Ok(balances) => {
            println!("\nYour balances:");
            for balance in balances {
                println!(
                    "  {}: free {}, locked {}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => println!("Error getting balances: {}", e),
    }

    Ok(())
}
```

## Handling Authorization Errors

```rust
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiError {
    code: i32,
    msg: String,
}

#[derive(Debug)]
enum AuthError {
    InvalidApiKey,
    InvalidSignature,
    TimestampOutOfWindow,
    IpNotWhitelisted,
    RateLimited,
    Unknown(String),
}

impl From<ApiError> for AuthError {
    fn from(error: ApiError) -> Self {
        match error.code {
            -2014 => AuthError::InvalidApiKey,
            -1022 => AuthError::InvalidSignature,
            -1021 => AuthError::TimestampOutOfWindow,
            -2015 => AuthError::IpNotWhitelisted,
            -1003 => AuthError::RateLimited,
            _ => AuthError::Unknown(error.msg),
        }
    }
}

async fn handle_auth_response(
    response: reqwest::Response
) -> Result<String, AuthError> {
    match response.status() {
        StatusCode::OK => {
            Ok(response.text().await.unwrap_or_default())
        }
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            let error: ApiError = response.json().await
                .unwrap_or(ApiError { code: 0, msg: "Unknown".to_string() });
            Err(error.into())
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Err(AuthError::RateLimited)
        }
        _ => {
            let text = response.text().await.unwrap_or_default();
            Err(AuthError::Unknown(text))
        }
    }
}

fn handle_auth_error(error: AuthError) {
    match error {
        AuthError::InvalidApiKey => {
            eprintln!("Error: Invalid API key. Check your settings.");
        }
        AuthError::InvalidSignature => {
            eprintln!("Error: Invalid signature. Check your secret key.");
        }
        AuthError::TimestampOutOfWindow => {
            eprintln!("Error: Timestamp out of sync. Synchronize your system clock.");
        }
        AuthError::IpNotWhitelisted => {
            eprintln!("Error: IP not whitelisted. Add your IP in API settings.");
        }
        AuthError::RateLimited => {
            eprintln!("Error: Rate limit exceeded. Wait before retrying.");
        }
        AuthError::Unknown(msg) => {
            eprintln!("Unknown error: {}", msg);
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| HTTP Headers | Request metadata containing authorization information |
| API Key | Client identifier for API access |
| Bearer Token | OAuth 2.0 format authorization token |
| HMAC Signature | Cryptographic signature for authenticity verification |
| Timestamp | Protection against request replay attacks |
| Secure Storage | Using environment variables for keys |

## Homework

1. **Basic Client**: Create a `CryptoExchangeClient` struct that:
   - Stores API key and secret key
   - Has a `get_ticker(symbol: &str)` method for fetching prices
   - Uses proper authorization headers

2. **Request Signing**: Implement a `sign_request` function that:
   - Takes request parameters and secret key
   - Returns HMAC-SHA256 signature in hex format
   - Adds timestamp to parameters

3. **Error Handling**: Extend the client with error handling:
   - Parse API error codes (401, 403, 429)
   - Automatic retry on temporary errors
   - Log all authorization attempts

4. **Key Rotation**: Implement a mechanism for working with multiple API keys:
   - Store multiple key pairs
   - Switch to backup key on error
   - Track status of each key

## Navigation

[← Previous day](../198-http-post-sending-order/en.md) | [Next day →](../200-http-client-connection-pooling/en.md)
