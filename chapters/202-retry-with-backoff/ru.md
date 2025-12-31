# День 202: Retry с backoff: повторяем запросы

## Аналогия из трейдинга

Представь ситуацию: твой торговый бот отправляет ордер на биржу, но сервер временно недоступен. Что делать? Сразу же повторить запрос? А если сервер всё ещё недоступен? Если повторять слишком часто, ты рискуешь:
- Перегрузить сервер ещё больше
- Получить бан за превышение лимита запросов
- Упустить момент, когда сервер снова заработает

**Retry с backoff** — это стратегия повторных попыток, при которой между попытками мы ждём всё дольше и дольше. Как опытный трейдер, который не спамит ордерами, а делает паузу и ждёт подходящего момента.

В реальном трейдинге это критически важно:
- Биржевые API часто возвращают временные ошибки (503, 429)
- Сетевые сбои случаются постоянно
- Rate limiting на биржах — обычное дело

## Что такое Exponential Backoff?

**Exponential Backoff** — это алгоритм, при котором время ожидания между попытками увеличивается экспоненциально:

```
Попытка 1: ждём 1 секунду
Попытка 2: ждём 2 секунды
Попытка 3: ждём 4 секунды
Попытка 4: ждём 8 секунд
...
```

Формула: `delay = base_delay * (2 ^ attempt)`

## Простой пример Retry

```rust
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
enum ExchangeError {
    TemporaryError(String),
    PermanentError(String),
    RateLimited,
}

// Симуляция отправки ордера на биржу
async fn send_order_to_exchange(symbol: &str, quantity: f64, price: f64) -> Result<u64, ExchangeError> {
    // В реальном коде здесь был бы HTTP запрос
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random: f64 = rng.gen();

    if random < 0.7 {
        // 70% вероятность временной ошибки
        Err(ExchangeError::TemporaryError("Server overloaded".to_string()))
    } else {
        Ok(12345) // ID ордера
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
                println!("Ордер {} успешно отправлен с попытки {}", order_id, attempt + 1);
                return Ok(order_id);
            }
            Err(ExchangeError::PermanentError(msg)) => {
                // Постоянная ошибка — повторять бессмысленно
                println!("Постоянная ошибка: {}", msg);
                return Err(ExchangeError::PermanentError(msg));
            }
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    println!("Превышено максимальное число попыток: {}", max_retries);
                    return Err(e);
                }

                // Экспоненциальный backoff
                let delay = base_delay * 2_u32.pow(attempt - 1);
                println!(
                    "Попытка {} не удалась: {:?}. Ждём {:?}...",
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
        Ok(order_id) => println!("Ордер создан: {}", order_id),
        Err(e) => println!("Не удалось создать ордер: {:?}", e),
    }
}
```

## Добавляем Jitter — случайный разброс

Если много клиентов начнут делать retry одновременно, они снова перегрузят сервер в одно и то же время. **Jitter** добавляет случайный разброс к задержке:

```rust
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

fn calculate_backoff_with_jitter(attempt: u32, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
    let mut rng = rand::thread_rng();

    // Экспоненциальный рост
    let exponential_delay = base_delay_ms * 2_u64.pow(attempt);

    // Ограничиваем максимальной задержкой
    let capped_delay = exponential_delay.min(max_delay_ms);

    // Добавляем случайный jitter (0% - 100% от задержки)
    let jitter = rng.gen_range(0..=capped_delay);

    Duration::from_millis(capped_delay + jitter)
}

async fn fetch_price_with_jitter(
    symbol: &str,
    max_retries: u32,
) -> Result<f64, String> {
    let base_delay_ms = 100;
    let max_delay_ms = 10000; // Максимум 10 секунд

    for attempt in 0..max_retries {
        // Симуляция запроса цены
        let result: Result<f64, &str> = if attempt < 2 {
            Err("Service temporarily unavailable")
        } else {
            Ok(42150.50)
        };

        match result {
            Ok(price) => return Ok(price),
            Err(e) => {
                if attempt + 1 >= max_retries {
                    return Err(format!("Превышен лимит попыток: {}", e));
                }

                let delay = calculate_backoff_with_jitter(attempt, base_delay_ms, max_delay_ms);
                println!(
                    "[{}] Попытка {} не удалась. Ждём {:?}...",
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
        Ok(price) => println!("Цена ETH: ${:.2}", price),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Структура для управления Retry

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

        // Экспоненциальный рост
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

// Трейт для определения, стоит ли повторять попытку
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
            // Эти ошибки повторять бессмысленно
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
                    println!("[{}] Успех с попытки {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(e) => {
                if !e.is_retryable() {
                    println!("[{}] Неповторяемая ошибка: {:?}", operation_name, e);
                    return Err(e);
                }

                attempt += 1;
                if attempt >= config.max_retries {
                    println!(
                        "[{}] Превышено число попыток ({}): {:?}",
                        operation_name, config.max_retries, e
                    );
                    return Err(e);
                }

                // Специальная обработка Rate Limiting
                let delay = if let TradingError::RateLimited { retry_after: Some(ra) } = &e {
                    // Это работает только если E = TradingError
                    // В общем случае используем стандартный backoff
                    *ra
                } else {
                    config.calculate_delay(attempt - 1)
                };

                println!(
                    "[{}] Попытка {} не удалась: {:?}. Ждём {:?}",
                    operation_name, attempt, e, delay
                );
                sleep(delay).await;
            }
        }
    }
}
```

## Практический пример: API клиент биржи

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
        // Симуляция сетевого запроса
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
        // Если сервер указал время ожидания, используем его
        if let ApiError::RateLimited { retry_after_ms } = error {
            return Duration::from_millis(*retry_after_ms);
        }

        // Иначе — экспоненциальный backoff с jitter
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
                    // Парсим цену из ответа
                    let price = 42150.50 + (attempt as f64 * 10.0);
                    println!("[Цена] {} = ${:.2} (попытка {})", symbol, price, attempt + 1);
                    return Ok(price);
                }
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }

                    attempt += 1;
                    if attempt >= self.retry_config.max_retries {
                        println!(
                            "[Цена] Превышен лимит попыток для {}: {:?}",
                            symbol, e
                        );
                        return Err(e);
                    }

                    let delay = self.calculate_delay(attempt - 1, &e);
                    println!(
                        "[Цена] Попытка {} для {} не удалась: {:?}. Ждём {:?}",
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
                        "[Ордер] {:?} {} {} @ {} создан (ID: {}, попытка {})",
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
                        println!("[Ордер] Превышен лимит попыток: {:?}", e);
                        return Err(e);
                    }

                    let delay = self.calculate_delay(attempt - 1, &e);
                    println!(
                        "[Ордер] Попытка {} не удалась: {:?}. Ждём {:?}",
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
                        "[Отмена] Ордер {} отменён (попытка {})",
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
                        "[Отмена] Попытка {} для ордера {} не удалась. Ждём {:?}",
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

    // Получаем цену
    match client.get_price("BTC/USDT").await {
        Ok(price) => println!("Текущая цена BTC: ${:.2}", price),
        Err(e) => println!("Не удалось получить цену: {:?}", e),
    }

    // Размещаем ордер
    let order = Order {
        id: None,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 0.1,
        price: 42000.0,
    };

    match client.place_order(&order).await {
        Ok(order_id) => {
            println!("Ордер создан с ID: {}", order_id);

            // Отменяем ордер
            match client.cancel_order(order_id).await {
                Ok(_) => println!("Ордер {} успешно отменён", order_id),
                Err(e) => println!("Не удалось отменить ордер: {:?}", e),
            }
        }
        Err(e) => println!("Не удалось создать ордер: {:?}", e),
    }
}
```

## Retry для множественных запросов

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
                return Err(format!("Не удалось получить цену для {}", symbol));
            }

            let delay = Duration::from_millis(self.base_delay_ms * 2_u64.pow(attempt - 1));
            println!("[{}] Попытка {} не удалась, ждём {:?}", symbol, attempt, delay);
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
    println!("Получаем цены для: {:?}\n", symbols);

    let results = fetcher.fetch_multiple_prices(symbols).await;

    println!("\nРезультаты:");
    for result in results {
        match result {
            Ok((symbol, price)) => println!("  {} = ${:.2}", symbol, price),
            Err(e) => println!("  Ошибка: {}", e),
        }
    }
}
```

## Стратегии Backoff

```rust
use std::time::Duration;

#[derive(Clone)]
pub enum BackoffStrategy {
    /// Постоянная задержка
    Constant { delay: Duration },

    /// Линейное увеличение: delay = base * attempt
    Linear { base: Duration, max: Duration },

    /// Экспоненциальное увеличение: delay = base * 2^attempt
    Exponential { base: Duration, max: Duration },

    /// Экспоненциальное с jitter
    ExponentialWithJitter { base: Duration, max: Duration },

    /// Декоррелированный jitter (AWS рекомендация)
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
                // Алгоритм AWS: sleep = min(max, random(base, prev_sleep * 3))
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

    println!("Сравнение стратегий backoff:\n");
    println!("{:<25} {:>10} {:>10} {:>10} {:>10} {:>10}",
             "Стратегия", "Попытка 1", "Попытка 2", "Попытка 3", "Попытка 4", "Попытка 5");
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

## Использование библиотеки tokio-retry

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

    // 60% вероятность ошибки
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
    // Создаём стратегию: начальная задержка 100мс, максимум 5 попыток
    let retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(10))
        .map(jitter) // Добавляем случайный разброс
        .take(5);    // Максимум 5 попыток

    let symbol = "BTC/USDT";

    let result = Retry::spawn(retry_strategy, || async {
        println!("Пытаемся получить цену для {}...", symbol);
        fetch_price_unreliable(symbol).await
    }).await;

    match result {
        Ok(data) => println!("Получены данные: {:?}", data),
        Err(e) => println!("Все попытки неудачны: {}", e),
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Retry | Повторная попытка выполнить операцию при ошибке |
| Backoff | Увеличение времени ожидания между попытками |
| Exponential Backoff | Экспоненциальный рост задержки (2^n) |
| Jitter | Случайный разброс для предотвращения одновременных попыток |
| Retryable errors | Различение временных и постоянных ошибок |
| Rate limiting | Обработка ограничений API на частоту запросов |

## Домашнее задание

1. **Базовый retry**: Реализуй функцию `retry_sync`, которая:
   - Принимает замыкание `F: FnMut() -> Result<T, E>`
   - Повторяет выполнение до `max_retries` раз
   - Использует линейный backoff (задержка увеличивается на фиксированное значение)
   - Логирует каждую попытку

2. **Умный retry для API**: Создай структуру `SmartRetryClient`, которая:
   - Различает типы ошибок (сеть, rate limit, авторизация, невалидные данные)
   - Использует специальную обработку для rate limit (ждёт указанное сервером время)
   - Не повторяет постоянные ошибки
   - Ведёт статистику успешных/неуспешных попыток

3. **Параллельный fetch с retry**: Напиши функцию `fetch_portfolio_prices`, которая:
   - Получает цены для списка активов параллельно
   - Каждый запрос имеет свой retry с backoff
   - Возвращает `HashMap<String, Result<f64, Error>>`
   - Не прерывается при ошибке одного актива

4. **Circuit Breaker + Retry**: Комбинируй паттерны:
   - Реализуй Circuit Breaker из предыдущих глав
   - Добавь retry только когда circuit не разомкнут
   - При слишком частых ошибках circuit размыкается и retry прекращается

## Навигация

[← Предыдущий день](../201-rate-limiting/ru.md) | [Следующий день →](../203-websocket-streaming/ru.md)
