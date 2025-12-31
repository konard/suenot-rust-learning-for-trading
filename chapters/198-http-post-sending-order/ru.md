# День 198: HTTP POST: отправляем ордер

## Аналогия из трейдинга

Представь, что GET-запрос — это когда ты смотришь котировки на экране терминала: ты получаешь информацию, но ничего не меняешь. А POST-запрос — это когда ты нажимаешь кнопку "Купить" или "Продать": ты отправляешь данные на сервер, и он выполняет действие — создаёт ордер, изменяет твой баланс, влияет на рынок.

В реальном трейдинге POST-запросы используются для:
- Размещения ордеров (лимитных, рыночных, стоп-ордеров)
- Отмены существующих ордеров
- Перевода средств между счетами
- Изменения настроек позиции (стоп-лосс, тейк-профит)

## Что такое HTTP POST?

HTTP POST — это метод, который используется для отправки данных на сервер. В отличие от GET:

| GET | POST |
|-----|------|
| Получает данные | Отправляет данные |
| Данные в URL | Данные в теле запроса |
| Кэшируется | Не кэшируется |
| Безопасен (не меняет состояние) | Небезопасен (меняет состояние) |
| Идемпотентен | Неидемпотентен |

## Библиотека reqwest для POST-запросов

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
    side: String,      // "buy" или "sell"
    order_type: String, // "market" или "limit"
    quantity: f64,
    price: Option<f64>, // Только для лимитных ордеров
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

    println!("Статус ответа: {}", response.status());

    let order_response: OrderResponse = response.json().await?;
    println!("Ордер создан: {:?}", order_response);

    Ok(())
}
```

## Форматы данных в POST-запросах

### 1. JSON (самый распространённый)

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
    println!("Ответ сервера: {}", result);

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

## Работа с заголовками аутентификации

Большинство торговых API требуют аутентификации:

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

    // Создаём подпись (упрощённый пример)
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
        Err(format!("Ошибка API: {}", error_text).into())
    }
}

fn create_signature(secret: &str, order: &LimitOrder, timestamp: i64) -> String {
    // В реальности здесь используется HMAC-SHA256
    format!("signature_placeholder_{}_{}", secret, timestamp)
}
```

## Обработка ошибок POST-запросов

```rust
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
enum OrderError {
    #[error("Недостаточно средств: требуется {required}, доступно {available}")]
    InsufficientFunds { required: f64, available: f64 },

    #[error("Неверный символ: {0}")]
    InvalidSymbol(String),

    #[error("Ордер отклонён: {0}")]
    OrderRejected(String),

    #[error("Ошибка сети: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Ошибка сервера: {0}")]
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
                    available: 0.0, // Нужно получить из ответа
                }),
                -1121 => Err(OrderError::InvalidSymbol(order.symbol.clone())),
                _ => Err(OrderError::OrderRejected(error.message)),
            }
        }
        StatusCode::UNAUTHORIZED => {
            Err(OrderError::OrderRejected("Неверный API ключ".to_string()))
        }
        StatusCode::TOO_MANY_REQUESTS => {
            Err(OrderError::OrderRejected("Превышен лимит запросов".to_string()))
        }
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(OrderError::ServerError(format!("{}: {}", status, body)))
        }
    }
}
```

## Практический пример: Торговый клиент

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
            .expect("Не удалось создать HTTP клиент");

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
            Err(format!("Ошибка {}: {}", status, body).into())
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
            Err(format!("Ошибка отмены {}: {}", status, body).into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TradingClient::new(
        "https://api.exchange.com",
        "your_api_key_here",
    );

    // Пример 1: Рыночный ордер на покупку
    println!("Размещаем рыночный ордер на покупку BTC...");
    match client.place_market_order("BTC/USDT", OrderSide::Buy, 0.001).await {
        Ok(order) => println!("Ордер создан: {:?}", order),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример 2: Лимитный ордер на продажу
    println!("\nРазмещаем лимитный ордер на продажу ETH...");
    match client.place_limit_order("ETH/USDT", OrderSide::Sell, 0.1, 2500.0).await {
        Ok(order) => {
            println!("Лимитный ордер создан: {:?}", order);

            // Пример 3: Отмена ордера
            println!("\nОтменяем ордер...");
            match client.cancel_order(&order.order_id, "ETH/USDT").await {
                Ok(result) => println!("Ордер отменён: {:?}", result),
                Err(e) => println!("Ошибка отмены: {}", e),
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пример 4: Стоп-лосс
    println!("\nУстанавливаем стоп-лосс...");
    match client.place_stop_loss("BTC/USDT", OrderSide::Sell, 0.001, 40000.0).await {
        Ok(order) => println!("Стоп-лосс установлен: {:?}", order),
        Err(e) => println!("Ошибка: {}", e),
    }

    Ok(())
}
```

## Повторные попытки при ошибках

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
        println!("Попытка {} из {}", attempt, max_retries);

        match client.post(url).json(body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return response.text().await.map_err(|e| e.to_string());
                }

                let status = response.status();

                // Для некоторых ошибок нет смысла повторять
                if status.as_u16() == 400 || status.as_u16() == 401 {
                    return Err(format!("Неисправимая ошибка: {}", status));
                }

                last_error = format!("HTTP ошибка: {}", status);
            }
            Err(e) => {
                last_error = format!("Ошибка сети: {}", e);
            }
        }

        if attempt < max_retries {
            // Экспоненциальная задержка
            let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
            println!("Ожидание {:?} перед повторной попыткой...", delay);
            sleep(delay).await;
        }
    }

    Err(format!(
        "Все {} попыток неудачны. Последняя ошибка: {}",
        max_retries, last_error
    ))
}
```

## Пакетная отправка ордеров

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
                    Err(format!("Ошибка: {}", response.status()))
                }
            }
        })
        .collect();

    join_all(futures).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Создаём сетку лимитных ордеров
    let orders: Vec<BatchOrder> = (0..5)
        .map(|i| BatchOrder {
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            quantity: 0.001,
            price: 41000.0 - (i as f64 * 100.0),
        })
        .collect();

    println!("Отправляем {} ордеров...", orders.len());

    let results = place_batch_orders(&client, orders).await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(order) => println!("Ордер {}: {} ({})", i + 1, order.order_id, order.status),
            Err(e) => println!("Ордер {}: Ошибка - {}", i + 1, e),
        }
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| HTTP POST | Метод для отправки данных на сервер |
| JSON body | Сериализация данных в JSON для отправки |
| Form data | Альтернативный формат данных |
| Заголовки | Content-Type, Authorization и другие |
| Обработка ошибок | Проверка статуса и парсинг ошибок API |
| Повторные попытки | Экспоненциальная задержка при ошибках |
| Пакетные запросы | Параллельная отправка нескольких ордеров |

## Домашнее задание

1. **Простой торговый бот**: Напиши программу, которая:
   - Получает текущую цену BTC (GET-запрос)
   - Если цена ниже заданного порога — создаёт ордер на покупку (POST-запрос)
   - Логирует все действия в консоль

2. **Обработка ошибок**: Расширь торгового клиента из примера, добавив:
   - Проверку баланса перед размещением ордера
   - Обработку rate limits (HTTP 429)
   - Автоматический повтор при временных ошибках

3. **Сетка ордеров**: Реализуй функцию `create_order_grid`, которая:
   - Принимает центральную цену, шаг и количество уровней
   - Создаёт лимитные ордера на покупку ниже и на продажу выше центральной цены
   - Возвращает список созданных ордеров

4. **WebSocket уведомления**: Добавь к торговому клиенту:
   - Отслеживание статуса ордера после размещения
   - Уведомление при исполнении ордера
   - Механизм отмены ордера по таймауту

## Навигация

[← Предыдущий день](../197-http-basics-reqwest-get/ru.md) | [Следующий день →](../199-http-headers-api-authorization/ru.md)
