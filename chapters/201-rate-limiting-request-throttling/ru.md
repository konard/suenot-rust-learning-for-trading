# День 201: Rate Limiting: ограничение запросов

## Аналогия из трейдинга

Представь, что ты торгуешь на бирже Binance и твой торговый бот отправляет по 1000 запросов в секунду, чтобы получить актуальные цены всех торговых пар. Биржа тут же заблокирует твой IP-адрес или API-ключ! Почему? Потому что биржа использует **rate limiting** (ограничение частоты запросов) — механизм защиты от перегрузки серверов.

Это как очередь в банке: кассир может обслужить только определённое количество клиентов в час. Если все ринутся одновременно — система рухнет. Поэтому устанавливается лимит: например, 10 запросов в секунду или 1200 запросов в минуту.

В трейдинге rate limiting критически важен:
- **Binance**: 1200 запросов в минуту (весовая система)
- **Bybit**: 120 запросов в минуту на эндпоинт
- **Kraken**: 15-20 вызовов в минуту (зависит от тарифа)

Если превысишь лимит — получишь ошибку 429 (Too Many Requests) или временную блокировку.

## Что такое Rate Limiting?

**Rate limiting** — это техника контроля количества запросов, которые клиент может отправить за определённый период времени. Реализация на стороне клиента позволяет:

1. **Не превышать лимиты API** — избегаем блокировок
2. **Справедливо распределять ресурсы** — не мешаем другим пользователям
3. **Защищать сервер** — предотвращаем случайные DDoS от нашего бота

## Основные алгоритмы Rate Limiting

### 1. Fixed Window (Фиксированное окно)

Простейший подход: считаем запросы в фиксированных временных интервалах.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct FixedWindowLimiter {
    max_requests: u64,
    window_duration: Duration,
    request_count: AtomicU64,
    window_start: Mutex<Instant>,
}

impl FixedWindowLimiter {
    fn new(max_requests: u64, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            request_count: AtomicU64::new(0),
            window_start: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self) -> bool {
        let mut window_start = self.window_start.lock().await;
        let now = Instant::now();

        // Если окно истекло — сбрасываем счётчик
        if now.duration_since(*window_start) >= self.window_duration {
            *window_start = now;
            self.request_count.store(0, Ordering::SeqCst);
        }

        // Проверяем, можем ли сделать запрос
        let current = self.request_count.fetch_add(1, Ordering::SeqCst);
        if current < self.max_requests {
            true
        } else {
            self.request_count.fetch_sub(1, Ordering::SeqCst);
            false
        }
    }
}

#[tokio::main]
async fn main() {
    // Лимит: 10 запросов в секунду (как на некоторых биржах)
    let limiter = Arc::new(FixedWindowLimiter::new(10, Duration::from_secs(1)));

    println!("Симуляция запросов к API биржи (лимит: 10 req/sec):");

    for i in 0..15 {
        if limiter.acquire().await {
            println!("  Запрос {} к /api/v1/ticker/price - OK", i + 1);
        } else {
            println!("  Запрос {} к /api/v1/ticker/price - ОТКЛОНЁН (превышен лимит)", i + 1);
        }
    }
}
```

### 2. Token Bucket (Корзина токенов)

Более гибкий алгоритм: токены добавляются с постоянной скоростью, каждый запрос потребляет токен.

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct TokenBucket {
    capacity: f64,           // Максимум токенов в корзине
    tokens: Mutex<f64>,      // Текущее количество токенов
    refill_rate: f64,        // Токенов в секунду
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Mutex::new(capacity), // Начинаем с полной корзины
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    async fn try_acquire(&self, tokens_needed: f64) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // Добавляем токены за прошедшее время
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        *tokens = (*tokens + new_tokens).min(self.capacity);
        *last_refill = now;

        // Пробуем взять токены
        if *tokens >= tokens_needed {
            *tokens -= tokens_needed;
            true
        } else {
            false
        }
    }

    async fn acquire(&self, tokens_needed: f64) {
        loop {
            if self.try_acquire(tokens_needed).await {
                return;
            }
            // Ждём немного и пробуем снова
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn available_tokens(&self) -> f64 {
        *self.tokens.lock().await
    }
}

#[tokio::main]
async fn main() {
    // Binance-подобный лимитер: 20 запросов в секунду
    let bucket = Arc::new(TokenBucket::new(20.0, 20.0));

    println!("Token Bucket Rate Limiter для торгового бота:");
    println!("  Ёмкость: 20 токенов");
    println!("  Пополнение: 20 токенов/сек\n");

    // Симуляция разных типов запросов с разным весом
    let requests = vec![
        ("GET /api/v1/ticker/price", 1.0),      // Лёгкий запрос
        ("GET /api/v1/depth", 5.0),             // Тяжёлый запрос (стакан)
        ("POST /api/v1/order", 1.0),            // Создание ордера
        ("GET /api/v1/klines", 5.0),            // История свечей
    ];

    for (endpoint, weight) in requests {
        let available = bucket.available_tokens().await;
        println!("Доступно токенов: {:.1}", available);

        bucket.acquire(weight).await;
        println!("  {} (вес: {}) - выполнен", endpoint, weight);
    }
}
```

### 3. Sliding Window Log (Скользящее окно с логом)

Точный, но требует больше памяти: храним timestamp каждого запроса.

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct SlidingWindowLog {
    window_duration: Duration,
    max_requests: usize,
    timestamps: Mutex<VecDeque<Instant>>,
}

impl SlidingWindowLog {
    fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            window_duration,
            max_requests,
            timestamps: Mutex::new(VecDeque::new()),
        }
    }

    async fn try_acquire(&self) -> bool {
        let mut timestamps = self.timestamps.lock().await;
        let now = Instant::now();

        // Удаляем старые timestamp'ы за пределами окна
        while let Some(ts) = timestamps.front() {
            if now.duration_since(*ts) >= self.window_duration {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        // Проверяем лимит
        if timestamps.len() < self.max_requests {
            timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    async fn time_until_available(&self) -> Duration {
        let timestamps = self.timestamps.lock().await;
        if timestamps.len() < self.max_requests {
            return Duration::ZERO;
        }

        if let Some(oldest) = timestamps.front() {
            let elapsed = Instant::now().duration_since(*oldest);
            if elapsed < self.window_duration {
                return self.window_duration - elapsed;
            }
        }

        Duration::ZERO
    }
}

#[tokio::main]
async fn main() {
    // Kraken-подобный лимит: 15 запросов в минуту
    let limiter = SlidingWindowLog::new(15, Duration::from_secs(60));

    println!("Sliding Window Rate Limiter (Kraken-style):");
    println!("  Лимит: 15 запросов в минуту\n");

    for i in 0..20 {
        if limiter.try_acquire().await {
            println!("  [{}] Запрос принят", i + 1);
        } else {
            let wait_time = limiter.time_until_available().await;
            println!(
                "  [{}] Запрос отклонён, подождите {:.1} сек",
                i + 1,
                wait_time.as_secs_f64()
            );
        }
    }
}
```

## Практический пример: Клиент биржи с Rate Limiting

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Реалистичная структура для весов запросов (как на Binance)
struct RequestWeight {
    endpoint: &'static str,
    weight: u32,
}

const ENDPOINTS: &[RequestWeight] = &[
    RequestWeight { endpoint: "/api/v3/ticker/price", weight: 1 },
    RequestWeight { endpoint: "/api/v3/depth", weight: 5 },
    RequestWeight { endpoint: "/api/v3/klines", weight: 5 },
    RequestWeight { endpoint: "/api/v3/order", weight: 1 },
    RequestWeight { endpoint: "/api/v3/account", weight: 10 },
];

struct ExchangeRateLimiter {
    max_weight_per_minute: u32,
    current_weight: Mutex<u32>,
    window_start: Mutex<Instant>,
}

impl ExchangeRateLimiter {
    fn new(max_weight_per_minute: u32) -> Self {
        Self {
            max_weight_per_minute,
            current_weight: Mutex::new(0),
            window_start: Mutex::new(Instant::now()),
        }
    }

    async fn check_and_wait(&self, weight: u32) {
        loop {
            let mut current = self.current_weight.lock().await;
            let mut start = self.window_start.lock().await;
            let now = Instant::now();

            // Сбрасываем окно если прошла минута
            if now.duration_since(*start) >= Duration::from_secs(60) {
                *start = now;
                *current = 0;
            }

            // Проверяем, можем ли выполнить запрос
            if *current + weight <= self.max_weight_per_minute {
                *current += weight;
                return;
            }

            // Вычисляем время ожидания
            let elapsed = now.duration_since(*start);
            let wait_time = Duration::from_secs(60) - elapsed;

            drop(current);
            drop(start);

            println!(
                "    [Rate Limit] Ожидание {:.1} сек до сброса лимита...",
                wait_time.as_secs_f64()
            );
            tokio::time::sleep(wait_time).await;
        }
    }

    async fn get_current_usage(&self) -> (u32, u32) {
        let current = *self.current_weight.lock().await;
        (current, self.max_weight_per_minute)
    }
}

struct TradingApiClient {
    rate_limiter: Arc<ExchangeRateLimiter>,
    base_url: String,
}

impl TradingApiClient {
    fn new(max_weight_per_minute: u32) -> Self {
        Self {
            rate_limiter: Arc::new(ExchangeRateLimiter::new(max_weight_per_minute)),
            base_url: "https://api.exchange.com".to_string(),
        }
    }

    async fn get_price(&self, symbol: &str) -> Result<f64, String> {
        self.rate_limiter.check_and_wait(1).await;

        // Симуляция HTTP запроса
        println!("  GET {}/api/v3/ticker/price?symbol={}", self.base_url, symbol);

        // В реальности здесь был бы reqwest::get(...)
        Ok(42000.0) // Симулированная цена BTC
    }

    async fn get_orderbook(&self, symbol: &str, limit: u32) -> Result<OrderBook, String> {
        // Стакан — тяжёлый запрос с весом 5
        self.rate_limiter.check_and_wait(5).await;

        println!(
            "  GET {}/api/v3/depth?symbol={}&limit={}",
            self.base_url, symbol, limit
        );

        Ok(OrderBook {
            bids: vec![(41990.0, 1.5), (41980.0, 2.3)],
            asks: vec![(42010.0, 0.8), (42020.0, 1.2)],
        })
    }

    async fn place_order(&self, symbol: &str, side: &str, qty: f64) -> Result<u64, String> {
        self.rate_limiter.check_and_wait(1).await;

        println!(
            "  POST {}/api/v3/order {{ symbol: {}, side: {}, qty: {} }}",
            self.base_url, symbol, side, qty
        );

        Ok(12345678) // Симулированный order_id
    }

    async fn get_account(&self) -> Result<AccountInfo, String> {
        // Информация об аккаунте — очень тяжёлый запрос (вес 10)
        self.rate_limiter.check_and_wait(10).await;

        println!("  GET {}/api/v3/account", self.base_url);

        Ok(AccountInfo {
            balances: vec![
                ("BTC".to_string(), 1.5),
                ("USDT".to_string(), 50000.0),
            ],
        })
    }

    async fn print_usage(&self) {
        let (current, max) = self.rate_limiter.get_current_usage().await;
        println!("  [Использовано: {}/{} веса]", current, max);
    }
}

#[derive(Debug)]
struct OrderBook {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

#[derive(Debug)]
struct AccountInfo {
    balances: Vec<(String, f64)>,
}

#[tokio::main]
async fn main() {
    println!("=== Торговый клиент с Rate Limiting ===\n");

    // Binance-like: 1200 веса в минуту
    let client = TradingApiClient::new(1200);

    println!("1. Получаем цену BTC:");
    let price = client.get_price("BTCUSDT").await.unwrap();
    println!("    Цена BTC: ${}\n", price);
    client.print_usage().await;

    println!("\n2. Получаем стакан заявок:");
    let orderbook = client.get_orderbook("BTCUSDT", 10).await.unwrap();
    println!("    Лучший bid: ${}", orderbook.bids[0].0);
    println!("    Лучший ask: ${}\n", orderbook.asks[0].0);
    client.print_usage().await;

    println!("\n3. Размещаем ордер:");
    let order_id = client.place_order("BTCUSDT", "BUY", 0.1).await.unwrap();
    println!("    Order ID: {}\n", order_id);
    client.print_usage().await;

    println!("\n4. Получаем информацию об аккаунте:");
    let account = client.get_account().await.unwrap();
    for (asset, balance) in &account.balances {
        println!("    {}: {}", asset, balance);
    }
    client.print_usage().await;

    println!("\n=== Симуляция массовых запросов ===");

    // Симулируем много запросов цен
    for i in 0..10 {
        let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"];
        let symbol = symbols[i % symbols.len()];
        let _ = client.get_price(symbol).await;
    }

    println!("\nИтоговое использование:");
    client.print_usage().await;
}
```

## Адаптивный Rate Limiter

Умный лимитер, который адаптируется к ответам сервера:

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum RateLimitState {
    Normal,       // Работаем в штатном режиме
    Cautious,     // Получили предупреждение, замедляемся
    Throttled,    // Получили 429, сильно замедляемся
}

struct AdaptiveRateLimiter {
    base_delay: Duration,
    current_delay: Mutex<Duration>,
    state: Mutex<RateLimitState>,
    last_request: Mutex<Instant>,
    consecutive_errors: Mutex<u32>,
}

impl AdaptiveRateLimiter {
    fn new(requests_per_second: f64) -> Self {
        let base_delay = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            base_delay,
            current_delay: Mutex::new(base_delay),
            state: Mutex::new(RateLimitState::Normal),
            last_request: Mutex::new(Instant::now()),
            consecutive_errors: Mutex::new(0),
        }
    }

    async fn wait_for_slot(&self) {
        let delay = *self.current_delay.lock().await;
        let mut last = self.last_request.lock().await;

        let elapsed = last.elapsed();
        if elapsed < delay {
            tokio::time::sleep(delay - elapsed).await;
        }

        *last = Instant::now();
    }

    async fn report_success(&self) {
        let mut errors = self.consecutive_errors.lock().await;
        *errors = 0;

        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        // Постепенно возвращаемся к нормальному режиму
        match *state {
            RateLimitState::Throttled => {
                *state = RateLimitState::Cautious;
                *delay = self.base_delay * 2;
                println!("    [Limiter] Переход в режим Cautious");
            }
            RateLimitState::Cautious => {
                *state = RateLimitState::Normal;
                *delay = self.base_delay;
                println!("    [Limiter] Возврат в нормальный режим");
            }
            RateLimitState::Normal => {}
        }
    }

    async fn report_rate_limit_warning(&self) {
        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        *state = RateLimitState::Cautious;
        *delay = self.base_delay * 2;
        println!("    [Limiter] Предупреждение! Замедляемся x2");
    }

    async fn report_rate_limit_error(&self) {
        let mut errors = self.consecutive_errors.lock().await;
        *errors += 1;

        let mut state = self.state.lock().await;
        let mut delay = self.current_delay.lock().await;

        *state = RateLimitState::Throttled;

        // Экспоненциальный backoff
        let multiplier = 2u32.pow((*errors).min(5));
        *delay = self.base_delay * multiplier;

        println!(
            "    [Limiter] Ошибка 429! Замедляемся x{} (ошибок подряд: {})",
            multiplier, *errors
        );
    }

    async fn get_state(&self) -> RateLimitState {
        *self.state.lock().await
    }
}

// Симулятор ответов сервера
struct MockServer {
    request_count: Mutex<u32>,
    limit: u32,
}

impl MockServer {
    fn new(limit: u32) -> Self {
        Self {
            request_count: Mutex::new(0),
            limit,
        }
    }

    async fn handle_request(&self) -> Result<u32, u32> {
        let mut count = self.request_count.lock().await;
        *count += 1;

        // Сбрасываем счётчик каждые 10 запросов (симуляция временного окна)
        if *count > 10 {
            *count = 1;
        }

        if *count > self.limit {
            Err(429) // Too Many Requests
        } else if *count == self.limit {
            Ok(200)  // OK но на грани
        } else {
            Ok(200)
        }
    }

    async fn reset(&self) {
        *self.request_count.lock().await = 0;
    }
}

#[tokio::main]
async fn main() {
    println!("=== Адаптивный Rate Limiter ===\n");

    let limiter = Arc::new(AdaptiveRateLimiter::new(5.0)); // 5 req/sec базово
    let server = Arc::new(MockServer::new(3)); // Сервер принимает только 3 req за раз

    println!("Симуляция торговых запросов:\n");

    for i in 0..15 {
        limiter.wait_for_slot().await;

        let result = server.handle_request().await;

        match result {
            Ok(200) => {
                println!("[{}] Запрос успешен (state: {:?})", i + 1, limiter.get_state().await);
                limiter.report_success().await;
            }
            Err(429) => {
                println!("[{}] Ошибка 429! (state: {:?})", i + 1, limiter.get_state().await);
                limiter.report_rate_limit_error().await;
            }
            _ => {}
        }

        // Периодически сбрасываем сервер (симуляция нового временного окна)
        if i == 7 {
            println!("\n--- Новое временное окно ---\n");
            server.reset().await;
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Rate Limiting | Ограничение количества запросов за период времени |
| Fixed Window | Простой счётчик в фиксированном временном окне |
| Token Bucket | Корзина токенов с постоянным пополнением |
| Sliding Window | Скользящее окно с точным подсчётом |
| Весовая система | Разные запросы имеют разную стоимость |
| Адаптивный лимитер | Автоматическая подстройка под ответы сервера |

## Практические задания

1. **Многоуровневый лимитер**: Реализуй лимитер, который учитывает несколько ограничений одновременно:
   - 10 запросов в секунду
   - 100 запросов в минуту
   - 1000 запросов в час

2. **Лимитер с приоритетами**: Создай систему, где ордера имеют приоритет над запросами цен. Если лимит почти исчерпан, пропускаем только важные запросы.

3. **Распределённый лимитер**: Реализуй rate limiter, который работает с несколькими инстансами бота, используя общий счётчик (подсказка: используй tokio::sync::broadcast или Redis).

## Домашнее задание

1. **Multi-Exchange Rate Limiter**: Создай структуру `MultiExchangeLimiter`, которая управляет лимитами для нескольких бирж одновременно. Каждая биржа имеет свои лимиты:
   ```rust
   struct ExchangeLimits {
       name: String,
       requests_per_minute: u32,
       weight_per_minute: u32,
   }
   ```

2. **Rate Limit с Redis**: Реализуй `RedisRateLimiter`, который хранит состояние в Redis, позволяя нескольким процессам использовать общие лимиты.

3. **Умный клиент биржи**: Создай клиент, который:
   - Автоматически выбирает оптимальное время для запросов
   - Группирует несколько запросов в batch когда возможно
   - Кеширует часто запрашиваемые данные
   - Использует WebSocket вместо REST для потоковых данных

4. **Визуализация лимитов**: Добавь метод, который выводит красивый ASCII-график использования лимитов за последние 60 секунд.

## Навигация

[← Предыдущий день](../200-http-client-connection-pooling/ru.md) | [Следующий день →](../202-retry-backoff/ru.md)
