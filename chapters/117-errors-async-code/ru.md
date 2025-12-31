# День 117: Ошибки в асинхронном коде (превью)

## Аналогия из трейдинга

Представь, что у тебя запущено несколько торговых ботов одновременно: один следит за ценой BTC, другой за ETH, третий отправляет ордера на биржу. Каждый бот работает **асинхронно** — не блокируя остальных. Но что происходит, когда один из ботов сталкивается с ошибкой?

Если бот, следящий за BTC, потерял соединение с биржей — остальные боты должны продолжать работать. Но при этом мы должны **правильно обработать** ошибку: залогировать её, попробовать переподключиться, или уведомить трейдера.

В синхронном коде ошибка останавливает выполнение. В асинхронном — ошибка в одной задаче не должна "уронить" все остальные. Это и есть главная сложность обработки ошибок в async.

## Почему ошибки в async сложнее?

### 1. Отложенное выполнение

В async коде функция не выполняется сразу — она возвращает `Future`:

```rust
use std::future::Future;

// Синхронный код — выполняется сразу
fn sync_fetch_price() -> Result<f64, String> {
    // Работает прямо сейчас
    Ok(42000.0)
}

// Async код — возвращает "обещание" результата
async fn async_fetch_price() -> Result<f64, String> {
    // Выполнится только когда мы сделаем .await
    Ok(42000.0)
}

#[tokio::main]
async fn main() {
    // Без .await ничего не произойдёт!
    let future = async_fetch_price(); // Просто создали Future

    // Теперь выполняем
    let result = future.await;
    println!("Price: {:?}", result);
}
```

### 2. Ошибки могут произойти в любой момент

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price_with_timeout() -> Result<f64, String> {
    // Запрос может упасть в любой момент ожидания
    sleep(Duration::from_secs(1)).await;

    // Или здесь
    let price = simulate_api_call().await?;

    // Или здесь
    Ok(price)
}

async fn simulate_api_call() -> Result<f64, String> {
    // Симулируем случайную ошибку
    if rand::random::<bool>() {
        Err("Connection lost".to_string())
    } else {
        Ok(42000.0)
    }
}
```

### 3. Параллельные задачи

Когда выполняется несколько задач одновременно, каждая может завершиться с ошибкой:

```rust
use tokio::join;

async fn fetch_btc_price() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_price() -> Result<f64, String> {
    Err("ETH API unavailable".to_string())
}

async fn fetch_sol_price() -> Result<f64, String> {
    Ok(95.0)
}

#[tokio::main]
async fn main() {
    // Все три запроса выполняются параллельно
    let (btc, eth, sol) = join!(
        fetch_btc_price(),
        fetch_eth_price(),
        fetch_sol_price()
    );

    // Каждый результат нужно обработать отдельно
    println!("BTC: {:?}", btc);  // Ok(42000.0)
    println!("ETH: {:?}", eth);  // Err("ETH API unavailable")
    println!("SOL: {:?}", sol);  // Ok(95.0)
}
```

## Оператор ? в async функциях

Оператор `?` работает так же, как в обычных функциях:

```rust
use tokio::fs;

async fn load_portfolio() -> Result<Portfolio, Box<dyn std::error::Error>> {
    // ? пробрасывает ошибку вверх
    let content = fs::read_to_string("portfolio.json").await?;
    let portfolio: Portfolio = serde_json::from_str(&content)?;
    Ok(portfolio)
}

#[derive(Debug, serde::Deserialize)]
struct Portfolio {
    btc: f64,
    eth: f64,
}
```

## Обработка ошибок с tokio::spawn

Когда задача запускается через `tokio::spawn`, ошибки оборачиваются в `JoinError`:

```rust
use tokio::task::JoinError;

async fn risky_trade() -> Result<f64, String> {
    Err("Trade failed: insufficient balance".to_string())
}

#[tokio::main]
async fn main() {
    // spawn возвращает JoinHandle
    let handle = tokio::spawn(async {
        risky_trade().await
    });

    // Результат — Result<Result<f64, String>, JoinError>
    match handle.await {
        Ok(inner_result) => {
            // Задача завершилась, проверяем внутренний Result
            match inner_result {
                Ok(profit) => println!("Profit: ${:.2}", profit),
                Err(e) => println!("Trade error: {}", e),
            }
        }
        Err(join_error) => {
            // Задача паниковала или была отменена
            println!("Task failed: {:?}", join_error);
        }
    }
}
```

### Упрощаем с flatten

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        fetch_price().await
    });

    // Используем flatten для упрощения
    match handle.await {
        Ok(Ok(price)) => println!("Price: ${}", price),
        Ok(Err(e)) => println!("API error: {}", e),
        Err(e) => println!("Task panic: {:?}", e),
    }
}

async fn fetch_price() -> Result<f64, String> {
    Ok(42000.0)
}
```

## tokio::try_join! — прерываем при первой ошибке

Если нужно выполнить несколько async операций и прервать всё при первой ошибке:

```rust
use tokio::try_join;

async fn fetch_btc_price() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_price() -> Result<f64, String> {
    Err("ETH API down".to_string())
}

async fn fetch_sol_price() -> Result<f64, String> {
    Ok(95.0)
}

#[tokio::main]
async fn main() {
    // try_join! прерывается при первой ошибке
    let result = try_join!(
        fetch_btc_price(),
        fetch_eth_price(),
        fetch_sol_price()
    );

    match result {
        Ok((btc, eth, sol)) => {
            println!("All prices: BTC={}, ETH={}, SOL={}", btc, eth, sol);
        }
        Err(e) => {
            // Получили ошибку — остальные задачи отменяются
            println!("Failed to fetch prices: {}", e);
        }
    }
}
```

## tokio::select! — гонка задач

`select!` позволяет ждать первую завершившуюся задачу:

```rust
use tokio::{select, time::{sleep, Duration}};

async fn fetch_from_binance() -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;
    Ok(42000.0)
}

async fn fetch_from_kraken() -> Result<f64, String> {
    sleep(Duration::from_millis(150)).await;
    Ok(42050.0)
}

async fn timeout_task() {
    sleep(Duration::from_secs(5)).await;
}

#[tokio::main]
async fn main() {
    select! {
        result = fetch_from_binance() => {
            match result {
                Ok(price) => println!("Binance: ${}", price),
                Err(e) => println!("Binance error: {}", e),
            }
        }
        result = fetch_from_kraken() => {
            match result {
                Ok(price) => println!("Kraken: ${}", price),
                Err(e) => println!("Kraken error: {}", e),
            }
        }
        _ = timeout_task() => {
            println!("All requests timed out");
        }
    }
}
```

## Паттерн: обработка ошибок в мониторинге цен

```rust
use tokio::time::{interval, Duration};

struct PriceMonitor {
    symbol: String,
    retry_count: u32,
    max_retries: u32,
}

impl PriceMonitor {
    fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            retry_count: 0,
            max_retries: 3,
        }
    }

    async fn fetch_price(&self) -> Result<f64, String> {
        // Симуляция API запроса
        if rand::random::<f64>() < 0.3 {
            Err(format!("{} API error", self.symbol))
        } else {
            Ok(42000.0 * rand::random::<f64>())
        }
    }

    async fn run(&mut self) {
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            ticker.tick().await;

            match self.fetch_price().await {
                Ok(price) => {
                    println!("[{}] Price: ${:.2}", self.symbol, price);
                    self.retry_count = 0; // Сбрасываем счётчик при успехе
                }
                Err(e) => {
                    self.retry_count += 1;
                    println!("[{}] Error (attempt {}): {}",
                        self.symbol, self.retry_count, e);

                    if self.retry_count >= self.max_retries {
                        println!("[{}] Max retries reached, stopping", self.symbol);
                        break;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut monitor = PriceMonitor::new("BTC");
    monitor.run().await;
}
```

## Паттерн: отмена задачи при ошибке

```rust
use tokio::sync::watch;

async fn price_fetcher(symbol: &str, mut shutdown: watch::Receiver<bool>) {
    loop {
        tokio::select! {
            _ = async {
                // Симулируем получение цены
                tokio::time::sleep(Duration::from_secs(1)).await;
                match fetch_price(symbol).await {
                    Ok(price) => println!("[{}] ${:.2}", symbol, price),
                    Err(e) => println!("[{}] Error: {}", symbol, e),
                }
            } => {}

            _ = shutdown.changed() => {
                println!("[{}] Shutdown signal received", symbol);
                break;
            }
        }
    }
}

async fn fetch_price(symbol: &str) -> Result<f64, String> {
    Ok(42000.0)
}

use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Запускаем несколько мониторов
    let btc_handle = tokio::spawn(price_fetcher("BTC", shutdown_rx.clone()));
    let eth_handle = tokio::spawn(price_fetcher("ETH", shutdown_rx));

    // Ждём 3 секунды и отправляем сигнал остановки
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = shutdown_tx.send(true);

    // Ждём завершения задач
    let _ = tokio::join!(btc_handle, eth_handle);
    println!("All monitors stopped");
}
```

## Обработка ошибок с anyhow в async

```rust
use anyhow::{Context, Result, bail};

async fn place_order(symbol: &str, quantity: f64, price: f64) -> Result<String> {
    if quantity <= 0.0 {
        bail!("Quantity must be positive");
    }

    let order_id = submit_to_exchange(symbol, quantity, price)
        .await
        .context("Failed to submit order to exchange")?;

    confirm_order(&order_id)
        .await
        .context(format!("Failed to confirm order {}", order_id))?;

    Ok(order_id)
}

async fn submit_to_exchange(symbol: &str, quantity: f64, price: f64) -> Result<String> {
    // Симуляция отправки на биржу
    Ok(format!("ORD-{}-{}", symbol, rand::random::<u32>()))
}

async fn confirm_order(order_id: &str) -> Result<()> {
    // Симуляция подтверждения
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    match place_order("BTC", 0.5, 42000.0).await {
        Ok(id) => println!("Order placed: {}", id),
        Err(e) => {
            // anyhow даёт полную цепочку ошибок
            println!("Order failed: {:?}", e);
        }
    }
    Ok(())
}
```

## Практический пример: робастный клиент биржи

```rust
use std::time::Duration;
use tokio::time::sleep;

struct ExchangeClient {
    name: String,
    max_retries: u32,
    retry_delay: Duration,
}

#[derive(Debug)]
enum ExchangeError {
    ConnectionFailed(String),
    RateLimited,
    InvalidResponse(String),
    Timeout,
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::Timeout => write!(f, "Request timed out"),
        }
    }
}

impl std::error::Error for ExchangeError {}

impl ExchangeClient {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<f64, ExchangeError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.try_fetch_price(symbol).await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    println!("[{}] Attempt {}/{} failed: {}",
                        self.name, attempt, self.max_retries, e);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        // Exponential backoff
                        let delay = self.retry_delay * (2_u32.pow(attempt - 1));
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn try_fetch_price(&self, symbol: &str) -> Result<f64, ExchangeError> {
        // Симуляция различных ошибок
        let random: f64 = rand::random();

        if random < 0.2 {
            Err(ExchangeError::ConnectionFailed("Network error".to_string()))
        } else if random < 0.3 {
            Err(ExchangeError::RateLimited)
        } else if random < 0.35 {
            Err(ExchangeError::Timeout)
        } else {
            Ok(42000.0 + (random * 1000.0))
        }
    }
}

#[tokio::main]
async fn main() {
    let client = ExchangeClient::new("Binance");

    match client.fetch_price("BTCUSDT").await {
        Ok(price) => println!("BTC price: ${:.2}", price),
        Err(e) => println!("Failed after all retries: {}", e),
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `async fn` с `Result` | Async функции возвращают `Future<Output = Result<T, E>>` |
| `?` в async | Работает так же, как в sync коде |
| `tokio::spawn` + ошибки | Возвращает `Result<T, JoinError>` — двойной Result |
| `try_join!` | Прерывает выполнение при первой ошибке |
| `join!` | Ждёт все задачи, каждая может иметь свою ошибку |
| `select!` | Ждёт первую завершившуюся задачу |
| Retry с backoff | Повторяем запросы с увеличивающейся задержкой |
| Graceful shutdown | Используем каналы для корректной остановки |

## Домашнее задание

1. Напиши async функцию `fetch_prices_concurrent(symbols: Vec<&str>) -> HashMap<String, Result<f64, String>>`, которая получает цены для всех символов параллельно и возвращает результат для каждого.

2. Реализуй `PriceAggregator`, который:
   - Запрашивает цену с нескольких бирж параллельно
   - Возвращает среднюю цену, если хотя бы 2 биржи ответили
   - Возвращает ошибку, если меньше 2 бирж ответили

3. Создай async функцию `execute_with_timeout<T, E>(future: impl Future<Output = Result<T, E>>, timeout: Duration) -> Result<T, String>`, которая добавляет таймаут к любой async операции.

4. Напиши `OrderExecutor` с методами:
   - `place_order()` — размещает ордер с retry логикой
   - `cancel_order()` — отменяет ордер
   - `wait_for_fill()` — ждёт исполнения с таймаутом

## Навигация

[← Предыдущий день](../116-documenting-errors/ru.md) | [Следующий день →](../118-fail-fast/ru.md)
