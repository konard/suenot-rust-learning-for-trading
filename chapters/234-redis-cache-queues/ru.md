# День 234: Redis: кэш и очереди

## Аналогия из трейдинга

Представь, что ты работаешь на бирже и тебе нужно мгновенно получать текущие цены на активы. Каждый раз запрашивать цену из основной базы данных — слишком медленно. Вместо этого ты используешь **кэш** — быструю память, где хранятся последние цены. Это как держать табличку с текущими котировками прямо перед глазами, а не бегать каждый раз в архив.

А теперь представь поток ордеров: тысячи заявок на покупку и продажу поступают каждую секунду. Их нельзя обрабатывать хаотично — нужна **очередь**, которая гарантирует, что каждый ордер будет обработан в правильном порядке. Redis — это как суперскоростной блокнот, который умеет и кэшировать данные, и организовывать очереди.

## Что такое Redis?

Redis (Remote Dictionary Server) — это:
- **In-memory хранилище** — данные хранятся в оперативной памяти
- **Key-value база данных** — простое хранение по ключам
- **Брокер сообщений** — поддерживает pub/sub и очереди
- **Сверхбыстрый** — задержки измеряются в микросекундах

### Почему Redis важен для трейдинга?

| Применение | Польза |
|------------|--------|
| Кэширование цен | Мгновенный доступ к котировкам |
| Очередь ордеров | Гарантированный порядок обработки |
| Сессии пользователей | Быстрая аутентификация |
| Rate limiting | Защита от перегрузки API |
| Pub/Sub | Рассылка обновлений в реальном времени |

## Подключение к Redis из Rust

Добавьте зависимости в `Cargo.toml`:

```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Базовое подключение

```rust
use redis::AsyncCommands;
use redis::Client;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Подключаемся к Redis
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    // Простые операции с ключами
    con.set("btc_price", 42000.50_f64).await?;
    let price: f64 = con.get("btc_price").await?;
    println!("Цена BTC: ${}", price);

    // Установка с временем жизни (TTL)
    con.set_ex("eth_price", 2800.25_f64, 60).await?; // Истекает через 60 секунд
    println!("Цена ETH установлена с TTL 60 секунд");

    Ok(())
}
```

## Кэширование рыночных данных

### Структура для хранения котировок

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume_24h: f64,
    timestamp: u64,
}

impl MarketTick {
    fn new(symbol: &str, bid: f64, ask: f64, last_price: f64, volume: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        MarketTick {
            symbol: symbol.to_string(),
            bid,
            ask,
            last_price,
            volume_24h: volume,
            timestamp,
        }
    }

    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn spread_percent(&self) -> f64 {
        (self.spread() / self.last_price) * 100.0
    }
}

struct PriceCache {
    client: redis::Client,
}

impl PriceCache {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(PriceCache { client })
    }

    async fn set_tick(&self, tick: &MarketTick, ttl_seconds: u64) -> redis::RedisResult<()> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("tick:{}", tick.symbol);
        let json = serde_json::to_string(tick).unwrap();

        con.set_ex(&key, json, ttl_seconds).await?;

        // Также сохраняем отдельные поля для быстрого доступа
        let price_key = format!("price:{}", tick.symbol);
        con.set_ex(&price_key, tick.last_price, ttl_seconds).await?;

        Ok(())
    }

    async fn get_tick(&self, symbol: &str) -> redis::RedisResult<Option<MarketTick>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("tick:{}", symbol);

        let json: Option<String> = con.get(&key).await?;

        Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
    }

    async fn get_price(&self, symbol: &str) -> redis::RedisResult<Option<f64>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("price:{}", symbol);

        con.get(&key).await
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> redis::RedisResult<Vec<Option<f64>>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let keys: Vec<String> = symbols.iter().map(|s| format!("price:{}", s)).collect();

        redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut con)
            .await
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let cache = PriceCache::new("redis://127.0.0.1:6379/")?;

    // Кэшируем котировки
    let btc_tick = MarketTick::new("BTCUSDT", 41990.0, 42010.0, 42000.0, 50000.0);
    let eth_tick = MarketTick::new("ETHUSDT", 2795.0, 2805.0, 2800.0, 100000.0);

    cache.set_tick(&btc_tick, 5).await?; // TTL 5 секунд для волатильных данных
    cache.set_tick(&eth_tick, 5).await?;

    println!("BTC спред: {:.4}%", btc_tick.spread_percent());
    println!("ETH спред: {:.4}%", eth_tick.spread_percent());

    // Получаем из кэша
    if let Some(tick) = cache.get_tick("BTCUSDT").await? {
        println!("Из кэша: {} @ ${}", tick.symbol, tick.last_price);
    }

    // Массовое получение цен
    let prices = cache.get_multiple_prices(&["BTCUSDT", "ETHUSDT", "SOLUSDT"]).await?;
    println!("Цены: {:?}", prices);

    Ok(())
}
```

## Очереди ордеров с Redis

### Простая очередь с LPUSH/RPOP

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum OrderType {
    Market,
    Limit,
}

struct OrderQueue {
    client: redis::Client,
    queue_name: String,
}

impl OrderQueue {
    fn new(redis_url: &str, queue_name: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(OrderQueue {
            client,
            queue_name: queue_name.to_string(),
        })
    }

    // Добавить ордер в конец очереди
    async fn push(&self, order: &Order) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        con.rpush(&self.queue_name, json).await
    }

    // Добавить приоритетный ордер в начало очереди
    async fn push_priority(&self, order: &Order) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        con.lpush(&self.queue_name, json).await
    }

    // Получить ордер из начала очереди (неблокирующий)
    async fn pop(&self) -> redis::RedisResult<Option<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json: Option<String> = con.lpop(&self.queue_name, None).await?;

        Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
    }

    // Блокирующее получение (ждёт до таймаута)
    async fn pop_blocking(&self, timeout_seconds: f64) -> redis::RedisResult<Option<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;

        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(&self.queue_name)
            .arg(timeout_seconds)
            .query_async(&mut con)
            .await?;

        Ok(result.and_then(|(_, json)| serde_json::from_str(&json).ok()))
    }

    // Получить длину очереди
    async fn len(&self) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.llen(&self.queue_name).await
    }

    // Просмотреть ордера без удаления
    async fn peek(&self, count: isize) -> redis::RedisResult<Vec<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let items: Vec<String> = con.lrange(&self.queue_name, 0, count - 1).await?;

        Ok(items
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let queue = OrderQueue::new("redis://127.0.0.1:6379/", "orders:pending")?;

    // Создаём тестовые ордера
    let orders = vec![
        Order {
            id: "ord_001".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.5,
            price: Some(41000.0),
            timestamp: 1700000001,
        },
        Order {
            id: "ord_002".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity: 10.0,
            price: None,
            timestamp: 1700000002,
        },
    ];

    // Добавляем в очередь
    for order in &orders {
        let queue_len = queue.push(order).await?;
        println!("Ордер {} добавлен, длина очереди: {}", order.id, queue_len);
    }

    // Приоритетный ордер
    let priority_order = Order {
        id: "ord_priority".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 1.0,
        price: None,
        timestamp: 1700000000,
    };
    queue.push_priority(&priority_order).await?;
    println!("Приоритетный ордер добавлен в начало очереди");

    // Просматриваем очередь
    let pending = queue.peek(10).await?;
    println!("\nОрдера в очереди:");
    for order in &pending {
        println!("  {} - {} {} {}", order.id, order.symbol,
            match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            order.quantity);
    }

    // Обрабатываем очередь
    println!("\nОбработка ордеров:");
    while let Some(order) = queue.pop().await? {
        println!("Обработан: {} - {} @ {:?}", order.id, order.symbol, order.price);
    }

    Ok(())
}
```

## Pub/Sub для рыночных данных

### Издатель котировок

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    change_percent: f64,
    timestamp: u64,
}

struct MarketDataPublisher {
    client: redis::Client,
}

impl MarketDataPublisher {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(MarketDataPublisher { client })
    }

    async fn publish(&self, channel: &str, update: &PriceUpdate) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(update).unwrap();

        con.publish(channel, json).await
    }

    async fn publish_to_symbol(&self, update: &PriceUpdate) -> redis::RedisResult<i64> {
        let channel = format!("prices:{}", update.symbol);
        self.publish(&channel, update).await
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let publisher = MarketDataPublisher::new("redis://127.0.0.1:6379/")?;

    let mut interval = interval(Duration::from_secs(1));
    let mut price = 42000.0_f64;

    println!("Начинаю публикацию цен BTC...");

    for i in 0..10 {
        interval.tick().await;

        // Симулируем изменение цены
        let change = (rand::random::<f64>() - 0.5) * 100.0;
        price += change;
        let change_percent = (change / price) * 100.0;

        let update = PriceUpdate {
            symbol: "BTCUSDT".to_string(),
            price,
            change_percent,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let subscribers = publisher.publish_to_symbol(&update).await?;
        println!("[{}] BTC: ${:.2} ({:+.2}%) - {} подписчиков",
            i + 1, update.price, update.change_percent, subscribers);
    }

    Ok(())
}
```

### Подписчик на котировки

```rust
use redis::Client;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut pubsub = client.get_async_pubsub().await?;

    // Подписываемся на несколько каналов
    pubsub.subscribe("prices:BTCUSDT").await?;
    pubsub.subscribe("prices:ETHUSDT").await?;

    // Или подписка по шаблону
    pubsub.psubscribe("prices:*").await?;

    println!("Подписан на обновления цен...");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel_name().to_string();
        let payload: String = msg.get_payload()?;

        println!("Канал: {} | Данные: {}", channel, payload);

        // Парсим и обрабатываем
        if let Ok(update) = serde_json::from_str::<serde_json::Value>(&payload) {
            if let Some(price) = update.get("price").and_then(|p| p.as_f64()) {
                println!("  -> Цена: ${:.2}", price);
            }
        }
    }

    Ok(())
}
```

## Rate Limiting для API

```rust
use redis::AsyncCommands;
use std::time::Duration;

struct RateLimiter {
    client: redis::Client,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    fn new(redis_url: &str, max_requests: u32, window_seconds: u64) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(RateLimiter {
            client,
            max_requests,
            window_seconds,
        })
    }

    async fn check_rate_limit(&self, client_id: &str) -> redis::RedisResult<RateLimitResult> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("ratelimit:{}", client_id);

        // Используем MULTI/EXEC для атомарности
        let current: i64 = con.incr(&key, 1).await?;

        if current == 1 {
            // Первый запрос - устанавливаем TTL
            con.expire(&key, self.window_seconds as i64).await?;
        }

        let remaining = self.max_requests as i64 - current;
        let ttl: i64 = con.ttl(&key).await?;

        if current > self.max_requests as i64 {
            Ok(RateLimitResult::Exceeded {
                retry_after: Duration::from_secs(ttl.max(0) as u64),
            })
        } else {
            Ok(RateLimitResult::Allowed {
                remaining: remaining.max(0) as u32,
                reset_in: Duration::from_secs(ttl.max(0) as u64),
            })
        }
    }
}

#[derive(Debug)]
enum RateLimitResult {
    Allowed { remaining: u32, reset_in: Duration },
    Exceeded { retry_after: Duration },
}

async fn handle_api_request(limiter: &RateLimiter, client_id: &str) -> Result<(), String> {
    match limiter.check_rate_limit(client_id).await {
        Ok(RateLimitResult::Allowed { remaining, reset_in }) => {
            println!("Запрос разрешён. Осталось: {}, сброс через: {:?}",
                remaining, reset_in);
            Ok(())
        }
        Ok(RateLimitResult::Exceeded { retry_after }) => {
            println!("Лимит превышен! Повторите через: {:?}", retry_after);
            Err(format!("Rate limit exceeded. Retry after {:?}", retry_after))
        }
        Err(e) => {
            println!("Ошибка Redis: {}", e);
            // В случае ошибки Redis — пропускаем (fail open)
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let limiter = RateLimiter::new("redis://127.0.0.1:6379/", 5, 10)?; // 5 запросов за 10 секунд

    let client_id = "trader_123";

    println!("Тестирование rate limiting:");
    for i in 1..=8 {
        println!("\nЗапрос #{}", i);
        let _ = handle_api_request(&limiter, client_id).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
```

## Практический пример: Торговая система с Redis

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trade {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    unrealized_pnl: f64,
}

struct TradingSystem {
    redis_client: redis::Client,
    positions: Arc<RwLock<HashMap<String, Position>>>,
}

impl TradingSystem {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(TradingSystem {
            redis_client: client,
            positions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // Кэширование позиций
    async fn sync_positions_to_cache(&self) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let positions = self.positions.read().await;

        for (symbol, position) in positions.iter() {
            let key = format!("position:{}", symbol);
            let json = serde_json::to_string(position).unwrap();
            con.set_ex(&key, json, 300).await?; // 5 минут TTL
        }

        Ok(())
    }

    // Запись сделки в историю
    async fn record_trade(&self, trade: &Trade) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        // В список последних сделок
        let trades_key = format!("trades:{}", trade.symbol);
        let json = serde_json::to_string(trade).unwrap();
        con.lpush(&trades_key, &json).await?;
        con.ltrim(&trades_key, 0, 999).await?; // Храним последние 1000 сделок

        // В sorted set для быстрого поиска по времени
        let history_key = format!("trade_history:{}", trade.symbol);
        con.zadd(&history_key, &json, trade.timestamp as f64).await?;

        // Публикуем событие
        con.publish("trades:new", &json).await?;

        Ok(())
    }

    // Получение последних сделок
    async fn get_recent_trades(&self, symbol: &str, count: isize) -> redis::RedisResult<Vec<Trade>> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("trades:{}", symbol);

        let items: Vec<String> = con.lrange(&key, 0, count - 1).await?;

        Ok(items
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }

    // Очередь ордеров с приоритетами
    async fn submit_order(&self, order: &Order, priority: i64) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        // Используем sorted set для приоритетной очереди
        // Меньший score = выше приоритет
        con.zadd("orders:priority_queue", &json, priority as f64).await?;

        Ok(())
    }

    // Получение следующего ордера по приоритету
    async fn get_next_order(&self) -> redis::RedisResult<Option<Order>> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        // ZPOPMIN возвращает элемент с минимальным score
        let result: Vec<(String, f64)> = redis::cmd("ZPOPMIN")
            .arg("orders:priority_queue")
            .arg(1)
            .query_async(&mut con)
            .await?;

        Ok(result.first().and_then(|(json, _)| serde_json::from_str(json).ok()))
    }

    // Статистика за период
    async fn get_trading_stats(&self, symbol: &str) -> redis::RedisResult<TradingStats> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        let trades_key = format!("trades:{}", symbol);
        let count: i64 = con.llen(&trades_key).await?;

        // Получаем последние сделки для расчёта объёма
        let recent: Vec<String> = con.lrange(&trades_key, 0, 99).await?;
        let trades: Vec<Trade> = recent
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect();

        let total_volume: f64 = trades.iter().map(|t| t.quantity * t.price).sum();
        let avg_price = if !trades.is_empty() {
            trades.iter().map(|t| t.price).sum::<f64>() / trades.len() as f64
        } else {
            0.0
        };

        Ok(TradingStats {
            symbol: symbol.to_string(),
            trade_count: count,
            total_volume,
            avg_price,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct TradingStats {
    symbol: String,
    trade_count: i64,
    total_volume: f64,
    avg_price: f64,
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let system = TradingSystem::new("redis://127.0.0.1:6379/")?;

    // Записываем несколько сделок
    let trades = vec![
        Trade {
            id: "t1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            timestamp: 1700000001,
        },
        Trade {
            id: "t2".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            price: 42100.0,
            quantity: 0.3,
            timestamp: 1700000002,
        },
    ];

    for trade in &trades {
        system.record_trade(trade).await?;
        println!("Записана сделка: {} {} {} @ ${}",
            trade.id, trade.side, trade.quantity, trade.price);
    }

    // Отправляем ордера с приоритетами
    let orders = vec![
        (Order {
            id: "o1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 41000.0,
            quantity: 1.0,
        }, 10), // Низкий приоритет
        (Order {
            id: "o2".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            price: 43000.0,
            quantity: 0.5,
        }, 1),  // Высокий приоритет
    ];

    for (order, priority) in &orders {
        system.submit_order(order, *priority).await?;
        println!("Ордер {} отправлен с приоритетом {}", order.id, priority);
    }

    // Получаем ордера по приоритету
    println!("\nОбработка ордеров по приоритету:");
    while let Some(order) = system.get_next_order().await? {
        println!("  Обработка: {} - {} {} @ ${}",
            order.id, order.side, order.quantity, order.price);
    }

    // Статистика
    let stats = system.get_trading_stats("BTCUSDT").await?;
    println!("\nСтатистика по BTCUSDT:");
    println!("  Сделок: {}", stats.trade_count);
    println!("  Объём: ${:.2}", stats.total_volume);
    println!("  Средняя цена: ${:.2}", stats.avg_price);

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Redis | In-memory хранилище для кэша и очередей |
| SET/GET | Базовые операции с ключами |
| TTL | Автоматическое истечение данных |
| LPUSH/RPOP | Очереди (FIFO) |
| Sorted Sets | Приоритетные очереди |
| Pub/Sub | Рассылка событий в реальном времени |
| Rate Limiting | Ограничение частоты запросов |

## Домашнее задание

1. **Кэш котировок**: Реализуйте систему кэширования, которая:
   - Хранит котировки с разным TTL для разных таймфреймов (1 сек для тиков, 1 мин для свечей)
   - Автоматически обновляет кэш при получении новых данных
   - Поддерживает batch-получение цен для нескольких символов

2. **Очередь ордеров с DLQ**: Создайте систему обработки ордеров с:
   - Основной очередью ордеров
   - Dead Letter Queue (DLQ) для неудачных ордеров
   - Автоматическим retry с экспоненциальной задержкой
   - Метриками успешности обработки

3. **Real-time алерты**: Реализуйте систему алертов на основе Pub/Sub:
   - Подписка на изменения цен
   - Триггеры по условиям (цена выше/ниже порога)
   - Уведомления через отдельный канал
   - Хранение истории алертов

4. **Распределённый rate limiter**: Создайте систему ограничения запросов для торгового API:
   - Разные лимиты для разных эндпоинтов
   - Поддержка burst-режима
   - Метрики использования лимитов
   - Graceful degradation при недоступности Redis

## Навигация

[← Предыдущий день](../233-postgresql-sqlx/ru.md) | [Следующий день →](../235-mongodb-document-storage/ru.md)
