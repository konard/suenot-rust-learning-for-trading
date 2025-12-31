# День 197: HTTP Basics: reqwest GET

## Аналогия из трейдинга

Представь, что ты трейдер, который хочет узнать текущую цену биткоина. Ты звонишь на биржу (делаешь запрос), называешь тикер ("BTC/USD"), и биржа отвечает тебе текущей ценой. Это и есть HTTP GET запрос — ты **запрашиваешь** данные, не изменяя их на сервере.

В реальном алготрейдинге:
- **GET /api/v1/ticker** — получить текущую цену
- **GET /api/v1/orderbook** — получить стакан заявок
- **GET /api/v1/trades** — получить историю сделок
- **GET /api/v1/account/balance** — проверить баланс аккаунта

Все эти операции — это GET запросы: ты читаешь данные с сервера, не изменяя их.

## Что такое reqwest?

`reqwest` — это популярная HTTP-библиотека для Rust. Она:
- Поддерживает async/await
- Работает с JSON "из коробки"
- Имеет простой и понятный API
- Поддерживает TLS (HTTPS)
- Позволяет настраивать таймауты, заголовки и многое другое

## Установка

Добавь в `Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Простой GET запрос

```rust
use reqwest;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Простой GET запрос
    let response = reqwest::get("https://api.coingecko.com/api/v3/ping")
        .await?;

    println!("Статус: {}", response.status());
    println!("Заголовки: {:#?}", response.headers());

    // Получаем тело ответа как текст
    let body = response.text().await?;
    println!("Ответ: {}", body);

    Ok(())
}
```

## Получение цены криптовалюты

```rust
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct PriceResponse {
    bitcoin: HashMap<String, f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd,eur,rub";

    let response = reqwest::get(url).await?;

    // Проверяем статус ответа
    if response.status().is_success() {
        let prices: PriceResponse = response.json().await?;

        if let Some(btc_prices) = prices.bitcoin.get("usd") {
            println!("BTC/USD: ${:.2}", btc_prices);
        }
        if let Some(btc_eur) = prices.bitcoin.get("eur") {
            println!("BTC/EUR: €{:.2}", btc_eur);
        }
        if let Some(btc_rub) = prices.bitcoin.get("rub") {
            println!("BTC/RUB: ₽{:.2}", btc_rub);
        }
    } else {
        println!("Ошибка: {}", response.status());
    }

    Ok(())
}
```

## Получение нескольких криптовалют

```rust
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct MultiPriceResponse {
    #[serde(flatten)]
    prices: HashMap<String, HashMap<String, f64>>,
}

async fn get_crypto_prices(symbols: &[&str]) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    let ids = symbols.join(",");
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
        ids
    );

    let response: MultiPriceResponse = reqwest::get(&url)
        .await?
        .json()
        .await?;

    let mut result = HashMap::new();
    for (symbol, prices) in response.prices {
        if let Some(&usd_price) = prices.get("usd") {
            result.insert(symbol, usd_price);
        }
    }

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let symbols = ["bitcoin", "ethereum", "solana", "cardano"];

    let prices = get_crypto_prices(&symbols).await?;

    println!("=== Текущие цены ===");
    for (symbol, price) in &prices {
        println!("{}: ${:.2}", symbol.to_uppercase(), price);
    }

    Ok(())
}
```

## Использование Client для переиспользования соединений

В торговых системах важно минимизировать задержку. Создание нового соединения для каждого запроса — медленно. Используй `Client` для переиспользования соединений:

```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct TickerResponse {
    symbol: String,
    price: String,
}

struct PriceMonitor {
    client: Client,
    base_url: String,
}

impl PriceMonitor {
    fn new() -> Self {
        // Создаём клиент с настройками
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        PriceMonitor {
            client,
            base_url: "https://api.binance.com".to_string(),
        }
    }

    async fn get_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v3/ticker/price?symbol={}", self.base_url, symbol);

        let ticker: TickerResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        let price: f64 = ticker.price.parse()?;
        Ok(price)
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> Vec<(String, Result<f64, String>)> {
        let mut results = Vec::new();

        for symbol in symbols {
            let result = match self.get_price(symbol).await {
                Ok(price) => (symbol.to_string(), Ok(price)),
                Err(e) => (symbol.to_string(), Err(e.to_string())),
            };
            results.push(result);
        }

        results
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = PriceMonitor::new();

    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT"];

    println!("=== Мониторинг цен ===");
    let prices = monitor.get_multiple_prices(&symbols).await;

    for (symbol, result) in prices {
        match result {
            Ok(price) => println!("{}: ${:.4}", symbol, price),
            Err(e) => println!("{}: Ошибка - {}", symbol, e),
        }
    }

    Ok(())
}
```

## Обработка заголовков запроса

Многие биржевые API требуют аутентификации через заголовки:

```rust
use reqwest::{Client, header};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AccountBalance {
    asset: String,
    free: String,
    locked: String,
}

struct ExchangeClient {
    client: Client,
    api_key: String,
}

impl ExchangeClient {
    fn new(api_key: &str) -> Self {
        let client = Client::new();
        ExchangeClient {
            client,
            api_key: api_key.to_string(),
        }
    }

    async fn get_ticker(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);

        // Добавляем заголовки к запросу
        let response = self.client
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header(header::ACCEPT, "application/json")
            .header(header::USER_AGENT, "TradingBot/1.0")
            .send()
            .await?;

        // Проверяем статус
        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct Ticker {
            price: String,
        }

        let ticker: Ticker = response.json().await?;
        Ok(ticker.price.parse()?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ExchangeClient::new("YOUR_API_KEY_HERE");

    match client.get_ticker("BTCUSDT").await {
        Ok(price) => println!("BTC/USDT: ${:.2}", price),
        Err(e) => println!("Ошибка: {}", e),
    }

    Ok(())
}
```

## Обработка ошибок и повторные попытки

В реальных торговых системах важно правильно обрабатывать ошибки сети:

```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct TickerResponse {
    symbol: String,
    price: String,
}

struct RobustPriceFetcher {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
}

impl RobustPriceFetcher {
    fn new(max_retries: u32, retry_delay_ms: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        RobustPriceFetcher {
            client,
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<f64, String> {
        let url = format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            symbol
        );

        let mut last_error = String::new();

        for attempt in 1..=self.max_retries {
            match self.try_fetch(&url).await {
                Ok(price) => {
                    if attempt > 1 {
                        println!("Успех с попытки {}", attempt);
                    }
                    return Ok(price);
                }
                Err(e) => {
                    last_error = e.to_string();
                    println!(
                        "Попытка {}/{} не удалась: {}",
                        attempt, self.max_retries, last_error
                    );

                    if attempt < self.max_retries {
                        // Экспоненциальная задержка
                        let delay = self.retry_delay * attempt;
                        println!("Ожидание {:?} перед повторной попыткой...", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(format!(
            "Все {} попыток не удались. Последняя ошибка: {}",
            self.max_retries, last_error
        ))
    }

    async fn try_fetch(&self, url: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let ticker: TickerResponse = response.json().await?;
        let price: f64 = ticker.price.parse()?;

        Ok(price)
    }
}

#[tokio::main]
async fn main() {
    let fetcher = RobustPriceFetcher::new(3, 1000);

    match fetcher.fetch_price("BTCUSDT").await {
        Ok(price) => println!("\nИтоговая цена BTC/USDT: ${:.2}", price),
        Err(e) => println!("\nНе удалось получить цену: {}", e),
    }
}
```

## Параллельные запросы

Для эффективного получения данных по нескольким торговым парам используй параллельные запросы:

```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use futures::future::join_all;

#[derive(Debug, Deserialize)]
struct TickerResponse {
    symbol: String,
    price: String,
}

#[derive(Debug)]
struct PriceData {
    symbol: String,
    price: f64,
}

async fn fetch_price(client: &Client, symbol: &str) -> Result<PriceData, String> {
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{}: {}", symbol, e))?;

    if !response.status().is_success() {
        return Err(format!("{}: HTTP {}", symbol, response.status()));
    }

    let ticker: TickerResponse = response
        .json()
        .await
        .map_err(|e| format!("{}: {}", symbol, e))?;

    let price: f64 = ticker
        .price
        .parse()
        .map_err(|e| format!("{}: parse error: {}", symbol, e))?;

    Ok(PriceData {
        symbol: symbol.to_string(),
        price,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let symbols = vec![
        "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT",
        "ADAUSDT", "XRPUSDT", "DOTUSDT", "DOGEUSDT",
    ];

    println!("Запрашиваем цены для {} валют параллельно...\n", symbols.len());

    let start = std::time::Instant::now();

    // Создаём фьючерсы для всех запросов
    let futures: Vec<_> = symbols
        .iter()
        .map(|symbol| fetch_price(&client, symbol))
        .collect();

    // Выполняем все запросы параллельно
    let results = join_all(futures).await;

    let elapsed = start.elapsed();

    println!("=== Результаты ===");
    let mut success_count = 0;
    let mut total_value = 0.0;

    for result in results {
        match result {
            Ok(data) => {
                println!("{}: ${:.4}", data.symbol, data.price);
                success_count += 1;
                total_value += data.price;
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }

    println!("\n=== Статистика ===");
    println!("Успешных запросов: {}/{}", success_count, symbols.len());
    println!("Общее время: {:?}", elapsed);
    println!(
        "Среднее время на запрос: {:?}",
        elapsed / symbols.len() as u32
    );

    Ok(())
}
```

## Практический пример: Монитор портфеля

```rust
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use futures::future::join_all;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_buy_price: f64,
}

#[derive(Debug)]
struct PortfolioValue {
    symbol: String,
    quantity: f64,
    current_price: f64,
    cost_basis: f64,
    current_value: f64,
    pnl: f64,
    pnl_percent: f64,
}

struct PortfolioMonitor {
    client: Client,
    positions: Vec<Position>,
}

impl PortfolioMonitor {
    fn new(positions: Vec<Position>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        PortfolioMonitor { client, positions }
    }

    async fn fetch_current_price(&self, symbol: &str) -> Result<f64, String> {
        #[derive(Deserialize)]
        struct Ticker {
            price: String,
        }

        let url = format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}USDT",
            symbol
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let ticker: Ticker = response.json().await.map_err(|e| e.to_string())?;
        ticker.price.parse().map_err(|e: std::num::ParseFloatError| e.to_string())
    }

    async fn calculate_portfolio(&self) -> Vec<PortfolioValue> {
        let futures: Vec<_> = self.positions
            .iter()
            .map(|pos| async {
                let price_result = self.fetch_current_price(&pos.symbol).await;
                (pos.clone(), price_result)
            })
            .collect();

        let results = join_all(futures).await;

        results
            .into_iter()
            .filter_map(|(pos, price_result)| {
                price_result.ok().map(|current_price| {
                    let cost_basis = pos.quantity * pos.avg_buy_price;
                    let current_value = pos.quantity * current_price;
                    let pnl = current_value - cost_basis;
                    let pnl_percent = (pnl / cost_basis) * 100.0;

                    PortfolioValue {
                        symbol: pos.symbol,
                        quantity: pos.quantity,
                        current_price,
                        cost_basis,
                        current_value,
                        pnl,
                        pnl_percent,
                    }
                })
            })
            .collect()
    }

    fn print_portfolio(&self, values: &[PortfolioValue]) {
        println!("\n{:=<70}", "");
        println!("{:^70}", "ПОРТФЕЛЬ");
        println!("{:=<70}", "");
        println!(
            "{:<8} {:>10} {:>12} {:>12} {:>12} {:>10}",
            "Символ", "Кол-во", "Цена", "Стоимость", "P&L", "P&L %"
        );
        println!("{:-<70}", "");

        let mut total_cost = 0.0;
        let mut total_value = 0.0;
        let mut total_pnl = 0.0;

        for pv in values {
            let pnl_sign = if pv.pnl >= 0.0 { "+" } else { "" };
            println!(
                "{:<8} {:>10.4} {:>12.2} {:>12.2} {:>11}{:.2} {:>9}{:.2}%",
                pv.symbol,
                pv.quantity,
                pv.current_price,
                pv.current_value,
                pnl_sign,
                pv.pnl,
                pnl_sign,
                pv.pnl_percent
            );

            total_cost += pv.cost_basis;
            total_value += pv.current_value;
            total_pnl += pv.pnl;
        }

        let total_pnl_percent = (total_pnl / total_cost) * 100.0;
        let pnl_sign = if total_pnl >= 0.0 { "+" } else { "" };

        println!("{:-<70}", "");
        println!(
            "{:<8} {:>10} {:>12} {:>12.2} {:>11}{:.2} {:>9}{:.2}%",
            "ИТОГО", "", "", total_value, pnl_sign, total_pnl, pnl_sign, total_pnl_percent
        );
        println!("{:=<70}", "");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Пример портфеля
    let positions = vec![
        Position {
            symbol: "BTC".to_string(),
            quantity: 0.5,
            avg_buy_price: 35000.0,
        },
        Position {
            symbol: "ETH".to_string(),
            quantity: 5.0,
            avg_buy_price: 2000.0,
        },
        Position {
            symbol: "SOL".to_string(),
            quantity: 50.0,
            avg_buy_price: 25.0,
        },
        Position {
            symbol: "ADA".to_string(),
            quantity: 1000.0,
            avg_buy_price: 0.45,
        },
    ];

    let monitor = PortfolioMonitor::new(positions);

    println!("Загрузка текущих цен...");
    let portfolio_values = monitor.calculate_portfolio().await;

    monitor.print_portfolio(&portfolio_values);

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `reqwest::get()` | Простой GET запрос |
| `Client` | Переиспользуемый HTTP клиент |
| `.json()` | Десериализация JSON ответа |
| `.headers()` | Добавление заголовков к запросу |
| `timeout` | Ограничение времени ожидания |
| `join_all` | Параллельное выполнение запросов |

## Домашнее задание

1. **Мониторинг цены**: Напиши программу, которая каждые 5 секунд получает цену BTC/USDT и выводит изменение в процентах от предыдущего значения.

2. **Топ-10 криптовалют**: Используя API CoinGecko, получи список топ-10 криптовалют по рыночной капитализации и выведи их названия, цены и изменение за 24 часа.

3. **Сравнение бирж**: Напиши программу, которая параллельно запрашивает цену BTC на нескольких биржах (Binance, Coinbase, Kraken) и показывает разницу цен (арбитражные возможности).

4. **Кэширование запросов**: Создай структуру `CachedPriceFetcher`, которая:
   - Кэширует результаты запросов на заданное время (например, 5 секунд)
   - Возвращает кэшированное значение, если оно ещё актуально
   - Делает новый запрос только при истечении кэша

## Навигация

[← Предыдущий день](../196-watch-channel-latest-value/ru.md) | [Следующий день →](../198-http-post-sending-order/ru.md)
