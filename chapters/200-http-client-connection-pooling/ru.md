# День 200: HTTP Client: Connection Pooling

## Аналогия из трейдинга

Представь, что ты торгуешь на нескольких биржах одновременно: Binance, Kraken, Coinbase. Каждый раз, когда нужно проверить цену или отправить ордер, ты звонишь брокеру, ждёшь пока он возьмёт трубку, представляешься, называешь свой номер счёта... и только потом получаешь информацию. Это очень медленно!

**Connection pooling** — это как иметь несколько выделенных линий с каждой биржей. Линии уже подключены, аутентификация пройдена, и когда нужны данные — ты сразу получаешь ответ без долгих приветствий. После использования линия не разрывается, а возвращается в пул для следующего запроса.

В мире HTTP это работает аналогично:
- Создание TCP-соединения занимает время (TCP handshake)
- TLS-рукопожатие для HTTPS ещё дороже
- Connection pooling позволяет переиспользовать соединения

## Что такое Connection Pool?

Connection pool — это набор предустановленных соединений с сервером, которые можно переиспользовать для множества запросов.

```
Без пула (каждый запрос):
┌────────┐    connect    ┌────────┐
│ Client │──────────────>│ Server │
└────────┘   request     └────────┘
    │        response        │
    │<───────────────────────│
    │        close           │
    X────────────────────────X

С пулом (множество запросов):
┌────────┐    connect    ┌────────┐
│ Client │══════════════>│ Server │  <- Соединение остаётся
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

## Базовый пример с reqwest

reqwest автоматически использует connection pooling:

```rust
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client содержит connection pool внутри
    // ВАЖНО: создаём один раз и переиспользуем
    let client = Client::new();

    // Получаем цены с биржи 10 раз
    // Все запросы используют одни и те же соединения из пула
    for i in 1..=10 {
        let response = client
            .get("https://api.binance.com/api/v3/ticker/price")
            .query(&[("symbol", "BTCUSDT")])
            .send()
            .await?;

        let price: serde_json::Value = response.json().await?;
        println!("Запрос {}: BTC = {}", i, price["price"]);

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
```

## Антипаттерн: создание клиента в цикле

```rust
// ПЛОХО: создание клиента на каждый запрос
async fn bad_example() {
    for _ in 0..10 {
        // Каждый раз создаётся новый пул соединений
        let client = Client::new();
        let _ = client.get("https://api.exchange.com/price").send().await;
        // client удаляется, соединение закрывается
    }
}

// ХОРОШО: один клиент для всех запросов
async fn good_example() {
    let client = Client::new();
    for _ in 0..10 {
        // Переиспользуем соединения из пула
        let _ = client.get("https://api.exchange.com/price").send().await;
    }
}
```

## Настройка Connection Pool

```rust
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

fn create_trading_client() -> Client {
    ClientBuilder::new()
        // Максимальное количество соединений на один хост
        .pool_max_idle_per_host(10)
        // Время жизни неактивного соединения
        .pool_idle_timeout(Duration::from_secs(90))
        // Таймаут соединения
        .connect_timeout(Duration::from_secs(5))
        // Общий таймаут запроса
        .timeout(Duration::from_secs(30))
        // Включаем HTTP/2 для мультиплексирования
        .http2_prior_knowledge()
        .build()
        .expect("Не удалось создать HTTP клиент")
}
```

## Пример: Мультибиржевой клиент

```rust
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct ExchangeClient {
    client: Client,
    // Кеш последних цен
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

        // Обновляем кеш
        let mut cache = self.price_cache.write().await;
        cache.insert(symbol.to_string(), price);

        Ok(price)
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> HashMap<String, f64> {
        let mut results = HashMap::new();

        // Параллельные запросы через один пул соединений
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

    println!("Текущие цены:");
    for (symbol, price) in &prices {
        println!("  {}: ${:.2}", symbol, price);
    }
}
```

## Мониторинг использования пула

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

    // Симулируем нагрузку
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
    println!("Всего запросов: {}, Ошибок: {}", total, errors);
    println!("Успешных: {}%", (total - errors) * 100 / total);
}
```

## HTTP/2 и мультиплексирование

HTTP/2 позволяет отправлять множество запросов через одно соединение:

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Клиент с HTTP/2
    let client = Client::builder()
        .http2_prior_knowledge()
        .build()?;

    // Все запросы идут через одно HTTP/2 соединение
    // с мультиплексированием
    let futures = (0..10).map(|i| {
        let client = client.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let _ = client
                .get("https://nghttp2.org/httpbin/get")
                .send()
                .await;
            println!("Запрос {}: {:?}", i, start.elapsed());
        })
    });

    futures::future::join_all(futures).await;

    Ok(())
}
```

## Практический пример: Арбитражный сканер

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
        // Один клиент (один пул) для всех бирж
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

    // Сканируем все символы параллельно
    let mut handles = vec![];
    for symbol in &symbols {
        let s = scanner.client.clone();
        let symbol = symbol.to_string();
        handles.push(tokio::spawn({
            let scanner_ref = &scanner;
            async move {
                // Здесь используем тот же пул соединений
            }
        }));
    }

    // Простая демонстрация
    for symbol in &symbols {
        scanner.scan_symbol(symbol).await;
        println!("Отсканировано: {}", symbol);
    }

    println!("\nПоиск арбитражных возможностей...");
    let opportunities = scanner.find_arbitrage().await;

    if opportunities.is_empty() {
        println!("Арбитражные возможности не найдены");
    } else {
        for opp in opportunities {
            println!("  {}", opp);
        }
    }
}
```

## Тестирование с моками

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
            .expect(100) // Ожидаем 100 запросов
            .mount(&mock_server)
            .await;

        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .build()
            .unwrap();

        let url = format!("{}/api/price", mock_server.uri());

        // 100 запросов через 5 соединений в пуле
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

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Connection Pool | Набор переиспользуемых соединений |
| `Client::new()` | Создаёт клиент с пулом по умолчанию |
| `pool_max_idle_per_host` | Максимум неактивных соединений на хост |
| `pool_idle_timeout` | Время жизни неактивного соединения |
| HTTP/2 | Мультиплексирование запросов в одном соединении |
| Переиспользование клиента | Ключ к эффективному connection pooling |

## Домашнее задание

1. **Бенчмарк пула**: Напиши программу, которая сравнивает производительность:
   - Создание нового клиента на каждый запрос
   - Использование одного клиента с пулом

   Измерь время для 100 запросов в каждом случае.

2. **Мульти-биржевой клиент**: Создай структуру `MultiExchangeClient`, которая:
   - Использует один `Client` для всех бирж
   - Имеет методы `get_binance_price()`, `get_kraken_price()` (можно использовать моки)
   - Параллельно опрашивает все биржи

3. **Мониторинг пула**: Реализуй обёртку над `Client`, которая:
   - Считает количество активных запросов
   - Логирует время каждого запроса
   - Показывает статистику по хостам

4. **Graceful degradation**: Создай клиент, который:
   - Имеет таймауты на каждом уровне (connect, read, total)
   - При ошибке возвращает последнее закешированное значение
   - Ведёт статистику успешных/неуспешных запросов

## Навигация

[← Предыдущий день](../199-http-headers-api-auth/ru.md) | [Следующий день →](../201-rate-limiting-throttling/ru.md)
