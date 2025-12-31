# День 199: HTTP Headers: авторизация API

## Аналогия из трейдинга

Представь, что ты приходишь на биржу для торговли. Прежде чем тебе позволят совершать сделки, охрана проверит твой пропуск — документ, подтверждающий твою личность и права доступа. Без пропуска ты не пройдёшь дальше приёмной.

HTTP-заголовки работают точно так же. Когда твой торговый бот отправляет запрос к API биржи:
- **API-ключ** — это твой "пропуск", удостоверяющий личность
- **Подпись запроса** — это "печать", подтверждающая, что запрос действительно от тебя
- **Временная метка** — это "дата на пропуске", защищающая от повторного использования старых запросов

Без правильных заголовков биржа просто отклонит твой запрос — как охрана не пустит человека без документов.

## Что такое HTTP-заголовки?

HTTP-заголовки — это метаданные, которые отправляются вместе с запросом или ответом. Они содержат дополнительную информацию:

```
GET /api/v1/account/balance HTTP/1.1
Host: api.exchange.com
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
X-API-KEY: your-api-key-here
X-TIMESTAMP: 1704067200000
Content-Type: application/json
```

### Основные заголовки для трейдинга

| Заголовок | Назначение |
|-----------|------------|
| `Authorization` | Токен авторизации (Bearer, Basic) |
| `X-API-KEY` | Публичный API-ключ |
| `X-API-SECRET` | Подпись запроса (HMAC) |
| `X-TIMESTAMP` | Временная метка в миллисекундах |
| `Content-Type` | Формат данных (application/json) |

## Типы авторизации API

### 1. API-ключ в заголовке

Самый простой способ — отправить API-ключ в заголовке:

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

    println!("Статус: {}", response.status());
    println!("Ответ: {}", response.text().await?);

    Ok(())
}
```

### 2. Bearer Token (OAuth 2.0)

Многие современные API используют Bearer-токены:

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

### 3. HMAC-подпись (Binance-style)

Криптобиржи часто требуют подпись запроса с использованием HMAC:

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
            .expect("HMAC может принимать ключ любой длины");
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

    // Получаем баланс
    match client.get_account_balance().await {
        Ok(balance) => println!("Баланс: {}", balance),
        Err(e) => println!("Ошибка получения баланса: {}", e),
    }

    // Размещаем ордер (только для примера!)
    // match client.place_order("BTCUSDT", "BUY", 0.001, 40000.0).await {
    //     Ok(order) => println!("Ордер: {}", order),
    //     Err(e) => println!("Ошибка размещения ордера: {}", e),
    // }

    Ok(())
}
```

## Безопасное хранение ключей

**Никогда не храни ключи в коде!** Используй переменные окружения:

```rust
use std::env;

struct ApiCredentials {
    api_key: String,
    secret_key: String,
}

impl ApiCredentials {
    fn from_env() -> Result<Self, String> {
        let api_key = env::var("EXCHANGE_API_KEY")
            .map_err(|_| "EXCHANGE_API_KEY не установлен")?;
        let secret_key = env::var("EXCHANGE_SECRET_KEY")
            .map_err(|_| "EXCHANGE_SECRET_KEY не установлен")?;

        Ok(ApiCredentials { api_key, secret_key })
    }
}

#[tokio::main]
async fn main() {
    // Загружаем .env файл (используя dotenv)
    dotenv::dotenv().ok();

    match ApiCredentials::from_env() {
        Ok(creds) => {
            println!("Ключи загружены успешно");
            println!("API Key: {}...", &creds.api_key[..8]);
        }
        Err(e) => {
            eprintln!("Ошибка: {}", e);
            eprintln!("Создайте .env файл с EXCHANGE_API_KEY и EXCHANGE_SECRET_KEY");
        }
    }
}
```

Файл `.env`:
```
EXCHANGE_API_KEY=your-api-key-here
EXCHANGE_SECRET_KEY=your-secret-key-here
```

## Практический пример: клиент для торговли

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
            .expect("HMAC принимает ключ любой длины");
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

    // Публичный эндпоинт (без авторизации)
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

    // Приватный эндпоинт (требует авторизации)
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

        // Фильтруем только ненулевые балансы
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

    // Размещение лимитного ордера
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

    // Получаем цену BTC (публичный эндпоинт)
    match client.get_price("BTCUSDT").await {
        Ok(price) => println!("Цена BTC: ${:.2}", price),
        Err(e) => println!("Ошибка получения цены: {}", e),
    }

    // Получаем балансы (приватный эндпоинт)
    match client.get_balances().await {
        Ok(balances) => {
            println!("\nВаши балансы:");
            for balance in balances {
                println!(
                    "  {}: свободно {}, заблокировано {}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => println!("Ошибка получения балансов: {}", e),
    }

    Ok(())
}
```

## Обработка ошибок авторизации

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
            eprintln!("Ошибка: Неверный API-ключ. Проверьте настройки.");
        }
        AuthError::InvalidSignature => {
            eprintln!("Ошибка: Неверная подпись. Проверьте секретный ключ.");
        }
        AuthError::TimestampOutOfWindow => {
            eprintln!("Ошибка: Рассинхронизация времени. Синхронизируйте системные часы.");
        }
        AuthError::IpNotWhitelisted => {
            eprintln!("Ошибка: IP не в белом списке. Добавьте IP в настройках API.");
        }
        AuthError::RateLimited => {
            eprintln!("Ошибка: Превышен лимит запросов. Подождите перед повтором.");
        }
        AuthError::Unknown(msg) => {
            eprintln!("Неизвестная ошибка: {}", msg);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| HTTP-заголовки | Метаданные запроса с информацией об авторизации |
| API-ключ | Идентификатор клиента для доступа к API |
| Bearer Token | Токен авторизации в формате OAuth 2.0 |
| HMAC-подпись | Криптографическая подпись для проверки подлинности |
| Временная метка | Защита от повторного использования запросов |
| Безопасное хранение | Использование переменных окружения для ключей |

## Домашнее задание

1. **Базовый клиент**: Создай структуру `CryptoExchangeClient`, которая:
   - Хранит API-ключ и секретный ключ
   - Имеет метод `get_ticker(symbol: &str)` для получения цены
   - Использует правильные заголовки авторизации

2. **Подпись запросов**: Реализуй функцию `sign_request`, которая:
   - Принимает параметры запроса и секретный ключ
   - Возвращает HMAC-SHA256 подпись в hex-формате
   - Добавляет временную метку к параметрам

3. **Обработка ошибок**: Расширь клиент обработкой ошибок:
   - Парсинг кодов ошибок API (401, 403, 429)
   - Автоматический retry при временных ошибках
   - Логирование всех попыток авторизации

4. **Ротация ключей**: Реализуй механизм работы с несколькими API-ключами:
   - Хранение нескольких пар ключей
   - Переключение на резервный ключ при ошибке
   - Отслеживание статуса каждого ключа

## Навигация

[← Предыдущий день](../198-http-post-sending-order/ru.md) | [Следующий день →](../200-http-client-connection-pooling/ru.md)
