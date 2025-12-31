# День 235: redis-rs: Подключение к Redis

## Аналогия из трейдинга

Представь, что ты трейдер, который приходит на биржу каждое утро. Прежде чем начать торговать, тебе нужно:
1. Подключиться к торговому терминалу
2. Авторизоваться
3. Проверить, что связь стабильна
4. Только потом отправлять ордера

Точно так же работает подключение к Redis. **Redis** — это сверхбыстрая in-memory база данных, которая в трейдинге используется для:
- Кеширования текущих цен (мгновенный доступ)
- Хранения сессий пользователей
- Очередей ордеров (pub/sub)
- Временного хранения расчётов (риск-метрики, P&L)

Библиотека **redis-rs** — это официальный Rust-клиент для работы с Redis. Сегодня мы научимся устанавливать соединение — первый и важнейший шаг.

## Добавление зависимости

Для начала добавим `redis` в `Cargo.toml`:

```toml
[dependencies]
redis = "0.25"
```

Если нужна асинхронная поддержка (рекомендуется для высоконагруженных торговых систем):

```toml
[dependencies]
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1", features = ["full"] }
```

## Базовое синхронное подключение

```rust
use redis::{Client, Commands, Connection};

fn main() -> redis::RedisResult<()> {
    // Создаём клиента (аналог — настройка торгового терминала)
    let client = Client::open("redis://127.0.0.1:6379/")?;

    // Устанавливаем соединение (аналог — вход на биржу)
    let mut con: Connection = client.get_connection()?;

    println!("Успешное подключение к Redis!");

    // Тестовая операция — записываем текущую цену BTC
    con.set("btc:price", 42500.50)?;

    // Читаем цену обратно
    let price: f64 = con.get("btc:price")?;
    println!("Текущая цена BTC: ${:.2}", price);

    Ok(())
}
```

## Формат строки подключения

Redis поддерживает несколько форматов URL:

```rust
use redis::Client;

fn main() -> redis::RedisResult<()> {
    // Базовое подключение (localhost)
    let _client1 = Client::open("redis://127.0.0.1/")?;

    // С указанием порта
    let _client2 = Client::open("redis://127.0.0.1:6379/")?;

    // С паролем (для защищённых серверов)
    let _client3 = Client::open("redis://:mypassword@127.0.0.1:6379/")?;

    // С выбором базы данных (0-15)
    let _client4 = Client::open("redis://127.0.0.1:6379/2")?;

    // С пользователем и паролем (Redis 6+)
    let _client5 = Client::open("redis://trading_user:secure_pass@127.0.0.1:6379/")?;

    // Unix socket (для локальных высокопроизводительных систем)
    let _client6 = Client::open("unix:///var/run/redis/redis.sock")?;

    // TLS/SSL подключение
    let _client7 = Client::open("rediss://127.0.0.1:6379/")?;

    println!("Все форматы подключения корректны!");

    Ok(())
}
```

## Обработка ошибок подключения

В торговых системах критически важно правильно обрабатывать ошибки:

```rust
use redis::{Client, Commands, RedisError};
use std::time::Duration;
use std::thread;

#[derive(Debug)]
struct TradingDataClient {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
}

impl TradingDataClient {
    fn new(url: &str) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(TradingDataClient {
            client,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        })
    }

    fn connect_with_retry(&self) -> Result<redis::Connection, RedisError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.client.get_connection() {
                Ok(con) => {
                    println!("Подключение установлено (попытка {})", attempt);
                    return Ok(con);
                }
                Err(e) => {
                    println!(
                        "Ошибка подключения (попытка {}/{}): {}",
                        attempt, self.max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        thread::sleep(self.retry_delay);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn store_price(&self, symbol: &str, price: f64) -> Result<(), RedisError> {
        let mut con = self.connect_with_retry()?;
        let key = format!("price:{}", symbol);
        con.set(&key, price)?;
        println!("Сохранена цена {}: ${:.2}", symbol, price);
        Ok(())
    }

    fn get_price(&self, symbol: &str) -> Result<f64, RedisError> {
        let mut con = self.connect_with_retry()?;
        let key = format!("price:{}", symbol);
        let price: f64 = con.get(&key)?;
        Ok(price)
    }
}

fn main() {
    match TradingDataClient::new("redis://127.0.0.1:6379/") {
        Ok(client) => {
            // Сохраняем цены
            if let Err(e) = client.store_price("BTC", 42500.0) {
                eprintln!("Ошибка сохранения BTC: {}", e);
            }

            if let Err(e) = client.store_price("ETH", 2250.0) {
                eprintln!("Ошибка сохранения ETH: {}", e);
            }

            // Читаем цены
            match client.get_price("BTC") {
                Ok(price) => println!("Цена BTC: ${:.2}", price),
                Err(e) => eprintln!("Ошибка чтения BTC: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Не удалось создать клиента: {}", e);
        }
    }
}
```

## Асинхронное подключение с Tokio

Для высокопроизводительных торговых систем рекомендуется асинхронный подход:

```rust
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Создаём клиента
    let client = Client::open("redis://127.0.0.1:6379/")?;

    // Асинхронное подключение с мультиплексированием
    let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    println!("Асинхронное подключение установлено!");

    // Асинхронные операции
    con.set("btc:price", 42500.50).await?;
    con.set("eth:price", 2250.75).await?;

    let btc: f64 = con.get("btc:price").await?;
    let eth: f64 = con.get("eth:price").await?;

    println!("BTC: ${:.2}, ETH: ${:.2}", btc, eth);

    Ok(())
}
```

## Connection Manager для торговых приложений

`ConnectionManager` автоматически переподключается при обрывах связи — критично для 24/7 торговых систем:

```rust
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use std::time::Duration;

struct MarketDataCache {
    manager: ConnectionManager,
}

impl MarketDataCache {
    async fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(MarketDataCache { manager })
    }

    async fn update_price(&mut self, symbol: &str, price: f64) -> redis::RedisResult<()> {
        let key = format!("market:{}:price", symbol);
        let timestamp_key = format!("market:{}:updated", symbol);

        // Сохраняем цену и время обновления
        self.manager.set(&key, price).await?;
        self.manager.set(&timestamp_key, chrono::Utc::now().timestamp()).await?;

        // Устанавливаем TTL (время жизни) — 60 секунд
        self.manager.expire::<_, ()>(&key, 60).await?;
        self.manager.expire::<_, ()>(&timestamp_key, 60).await?;

        Ok(())
    }

    async fn get_price(&mut self, symbol: &str) -> redis::RedisResult<Option<f64>> {
        let key = format!("market:{}:price", symbol);
        self.manager.get(&key).await
    }

    async fn update_order_book(
        &mut self,
        symbol: &str,
        bids: &[(f64, f64)],  // (price, quantity)
        asks: &[(f64, f64)],
    ) -> redis::RedisResult<()> {
        let bids_key = format!("orderbook:{}:bids", symbol);
        let asks_key = format!("orderbook:{}:asks", symbol);

        // Очищаем старые данные
        self.manager.del::<_, ()>(&bids_key).await?;
        self.manager.del::<_, ()>(&asks_key).await?;

        // Добавляем bids (sorted set: score = price)
        for (price, qty) in bids {
            let member = format!("{}:{}", price, qty);
            self.manager.zadd::<_, _, _, ()>(&bids_key, member, *price).await?;
        }

        // Добавляем asks
        for (price, qty) in asks {
            let member = format!("{}:{}", price, qty);
            self.manager.zadd::<_, _, _, ()>(&asks_key, member, *price).await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let mut cache = MarketDataCache::new("redis://127.0.0.1:6379/").await?;

    // Обновляем рыночные данные
    cache.update_price("BTC", 42500.0).await?;
    cache.update_price("ETH", 2250.0).await?;

    // Обновляем стакан
    cache.update_order_book(
        "BTC",
        &[(42490.0, 1.5), (42480.0, 2.3), (42470.0, 5.0)],
        &[(42510.0, 1.2), (42520.0, 3.1), (42530.0, 4.5)],
    ).await?;

    // Читаем цену
    if let Some(price) = cache.get_price("BTC").await? {
        println!("Кешированная цена BTC: ${:.2}", price);
    }

    Ok(())
}
```

## Проверка состояния подключения

```rust
use redis::{Client, Commands, Connection};

fn check_redis_connection(con: &mut Connection) -> bool {
    match redis::cmd("PING").query::<String>(con) {
        Ok(response) => response == "PONG",
        Err(_) => false,
    }
}

fn get_redis_info(con: &mut Connection) -> redis::RedisResult<()> {
    // Получаем информацию о сервере
    let info: String = redis::cmd("INFO").arg("server").query(con)?;

    println!("=== Информация о Redis сервере ===");
    for line in info.lines() {
        if line.starts_with("redis_version") ||
           line.starts_with("connected_clients") ||
           line.starts_with("used_memory_human") {
            println!("{}", line);
        }
    }

    Ok(())
}

fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;

    if check_redis_connection(&mut con) {
        println!("Redis доступен!");
        get_redis_info(&mut con)?;
    } else {
        eprintln!("Redis недоступен!");
    }

    Ok(())
}
```

## Пул соединений для многопоточных систем

Для торговых систем с множеством потоков используем пул соединений:

```rust
use redis::{Client, Commands};
use std::sync::Arc;
use std::thread;

fn main() -> redis::RedisResult<()> {
    let client = Arc::new(Client::open("redis://127.0.0.1:6379/")?);

    let mut handles = vec![];

    // Симулируем несколько торговых потоков
    for i in 0..4 {
        let client_clone = Arc::clone(&client);

        let handle = thread::spawn(move || {
            // Каждый поток получает своё соединение
            let mut con = client_clone.get_connection().expect("Ошибка подключения");

            let symbol = match i {
                0 => "BTC",
                1 => "ETH",
                2 => "SOL",
                _ => "DOGE",
            };

            // Симулируем обновление цены
            let price = 1000.0 * (i as f64 + 1.0);
            let key = format!("price:{}", symbol);

            let _: () = con.set(&key, price).expect("Ошибка записи");
            println!("Поток {}: {} = ${:.2}", i, symbol, price);

            // Читаем обратно
            let stored: f64 = con.get(&key).expect("Ошибка чтения");
            println!("Поток {}: Прочитано {} = ${:.2}", i, symbol, stored);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Ошибка потока");
    }

    println!("Все потоки завершены успешно!");

    Ok(())
}
```

## Конфигурация через переменные окружения

```rust
use redis::Client;
use std::env;

#[derive(Debug)]
struct RedisConfig {
    host: String,
    port: u16,
    password: Option<String>,
    database: u8,
}

impl RedisConfig {
    fn from_env() -> Self {
        RedisConfig {
            host: env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .unwrap_or(6379),
            password: env::var("REDIS_PASSWORD").ok(),
            database: env::var("REDIS_DATABASE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }

    fn to_url(&self) -> String {
        match &self.password {
            Some(pass) => format!(
                "redis://:{}@{}:{}/{}",
                pass, self.host, self.port, self.database
            ),
            None => format!(
                "redis://{}:{}/{}",
                self.host, self.port, self.database
            ),
        }
    }
}

fn main() -> redis::RedisResult<()> {
    let config = RedisConfig::from_env();
    println!("Конфигурация Redis: {:?}", config);

    let client = Client::open(config.to_url())?;
    let _con = client.get_connection()?;

    println!("Подключение успешно!");

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Client::open()` | Создание клиента Redis с URL |
| `get_connection()` | Синхронное подключение |
| `get_multiplexed_async_connection()` | Асинхронное подключение |
| `ConnectionManager` | Автоматическое переподключение |
| Формат URL | `redis://[user:pass@]host:port/db` |
| Обработка ошибок | Повторные попытки, таймауты |
| Пул соединений | Для многопоточных приложений |

## Упражнения

1. **Базовое подключение**: Создай программу, которая подключается к Redis, записывает текущую цену трёх криптовалют (BTC, ETH, SOL) и читает их обратно.

2. **Обработка ошибок**: Модифицируй программу из упражнения 1, чтобы она корректно обрабатывала ситуацию, когда Redis недоступен (выводила понятное сообщение, а не паниковала).

3. **Конфигурация**: Реализуй подключение через переменные окружения (`REDIS_URL` или отдельные `REDIS_HOST`, `REDIS_PORT`, `REDIS_PASSWORD`).

4. **Многопоточность**: Создай 4 потока, каждый из которых записывает и читает цены разных активов. Убедись, что все операции выполняются корректно.

## Домашнее задание

1. **Мониторинг подключения**: Создай структуру `RedisHealthChecker`, которая:
   - Периодически проверяет доступность Redis (PING)
   - Логирует время отклика
   - Уведомляет (через вывод в консоль) при проблемах с подключением

2. **Кеш котировок**: Реализуй структуру `QuoteCache` с методами:
   - `new(redis_url: &str)` — создание с подключением к Redis
   - `update_quote(symbol: &str, bid: f64, ask: f64)` — обновление котировки
   - `get_quote(symbol: &str) -> Option<(f64, f64)>` — получение котировки
   - `get_spread(symbol: &str) -> Option<f64>` — расчёт спреда

3. **Graceful shutdown**: Реализуй торговое приложение, которое:
   - Подключается к Redis при старте
   - Обрабатывает сигнал завершения (Ctrl+C)
   - Корректно закрывает соединение перед выходом

4. **Бенчмарк**: Измерь производительность Redis:
   - Время одиночной записи/чтения
   - Пропускную способность (операций в секунду)
   - Сравни синхронный и асинхронный подходы

## Навигация

[← День 234: Redis: Кеши и очереди](../234-redis-cache-queues/ru.md) | [День 236: Redis: Кеширование последних цен →](../236-redis-caching-latest-prices/ru.md)
