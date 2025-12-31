# День 191: tokio::timeout: ограничиваем ожидание

## Аналогия из трейдинга

Представь, что ты отправляешь рыночный ордер на биржу. В идеальном мире биржа мгновенно отвечает: "Ордер исполнен!" Но что, если биржа перегружена? Или сеть медленная? Или сервер биржи завис?

В реальном трейдинге ты никогда не хочешь ждать бесконечно:
- **Запрос цены**: Если биржа не ответила за 500мс — цена уже устарела, используй другой источник
- **Размещение ордера**: Если нет ответа за 2 секунды — считай, что ордер не размещён, и прими решение
- **Отмена ордера**: Критически важно знать результат быстро — рынок не ждёт

`tokio::timeout` — это твой "таймер терпения". Он говорит: "Жди результата, но не дольше X времени. Если время вышло — возвращай ошибку, и я решу, что делать дальше."

## Что такое tokio::timeout?

`tokio::timeout` — это функция из библиотеки Tokio, которая оборачивает любую асинхронную операцию (Future) и ограничивает время её выполнения. Если операция не завершается за указанное время, она прерывается, и возвращается ошибка `Elapsed`.

```rust
use tokio::time::{timeout, Duration};

async fn example() {
    // Ждём результат максимум 5 секунд
    let result = timeout(Duration::from_secs(5), some_async_operation()).await;

    match result {
        Ok(value) => println!("Операция завершилась: {:?}", value),
        Err(_) => println!("Таймаут! Операция не успела завершиться"),
    }
}
```

## Базовый синтаксис

```rust
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    // timeout возвращает Result<T, Elapsed>
    // где T — результат внутренней Future
    let result = timeout(
        Duration::from_secs(2),  // Максимальное время ожидания
        async {
            // Ваша асинхронная операция
            tokio::time::sleep(Duration::from_secs(1)).await;
            "Готово!"
        }
    ).await;

    match result {
        Ok(value) => println!("Успех: {}", value),
        Err(elapsed) => println!("Таймаут: {:?}", elapsed),
    }
}
```

## Пример: Запрос цены с таймаутом

```rust
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

// Имитация запроса цены с биржи
async fn fetch_price(exchange: &str, symbol: &str) -> f64 {
    // Имитируем разное время ответа разных бирж
    let delay = match exchange {
        "fast_exchange" => 100,
        "normal_exchange" => 500,
        "slow_exchange" => 2000,
        _ => 1000,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Возвращаем "цену"
    match (exchange, symbol) {
        ("fast_exchange", "BTC/USDT") => 42150.50,
        ("normal_exchange", "BTC/USDT") => 42148.25,
        ("slow_exchange", "BTC/USDT") => 42145.00,
        _ => 42000.0,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";
    let max_wait = Duration::from_millis(300); // Максимум 300мс на ответ

    let exchanges = vec!["fast_exchange", "normal_exchange", "slow_exchange"];
    let mut prices: HashMap<&str, Option<f64>> = HashMap::new();

    for exchange in exchanges {
        println!("Запрашиваем цену {} у {}...", symbol, exchange);

        let result = timeout(max_wait, fetch_price(exchange, symbol)).await;

        match result {
            Ok(price) => {
                println!("  {} ответила: ${:.2}", exchange, price);
                prices.insert(exchange, Some(price));
            }
            Err(_) => {
                println!("  {} — ТАЙМАУТ! Биржа не ответила за {:?}", exchange, max_wait);
                prices.insert(exchange, None);
            }
        }
    }

    // Используем цены от бирж, которые успели ответить
    let valid_prices: Vec<f64> = prices
        .values()
        .filter_map(|p| *p)
        .collect();

    if !valid_prices.is_empty() {
        let avg_price: f64 = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("\nСредняя цена (из {} источников): ${:.2}", valid_prices.len(), avg_price);
    } else {
        println!("\nНи одна биржа не ответила вовремя!");
    }
}
```

## Пример: Размещение ордера с таймаутом

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct Order {
    id: Option<u64>,
    symbol: String,
    side: String,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug)]
enum OrderResult {
    Filled { order_id: u64, fill_price: f64 },
    PartialFill { order_id: u64, filled_qty: f64, remaining: f64 },
    Pending { order_id: u64 },
    Rejected { reason: String },
}

// Имитация API биржи
async fn place_order_api(order: Order) -> OrderResult {
    // Имитируем задержку сети и обработки
    tokio::time::sleep(Duration::from_millis(800)).await;

    // Имитируем разные результаты
    OrderResult::Filled {
        order_id: 12345,
        fill_price: 42100.0,
    }
}

async fn place_order_with_timeout(order: Order, max_wait: Duration) -> Result<OrderResult, String> {
    println!("Размещаем ордер: {:?}", order);

    match timeout(max_wait, place_order_api(order.clone())).await {
        Ok(result) => {
            println!("Ордер обработан: {:?}", result);
            Ok(result)
        }
        Err(_) => {
            // Таймаут! Ордер мог быть размещён, но ответ не получен
            Err(format!(
                "Таймаут при размещении ордера {} {}. \
                ВНИМАНИЕ: Ордер мог быть размещён! Проверьте открытые ордера.",
                order.side, order.symbol
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: None,
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: Some(42000.0),
    };

    // Сценарий 1: Достаточный таймаут
    println!("=== Сценарий 1: Таймаут 2 секунды ===");
    match place_order_with_timeout(order.clone(), Duration::from_secs(2)).await {
        Ok(result) => println!("Успех: {:?}", result),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!();

    // Сценарий 2: Слишком короткий таймаут
    println!("=== Сценарий 2: Таймаут 100мс ===");
    match place_order_with_timeout(order.clone(), Duration::from_millis(100)).await {
        Ok(result) => println!("Успех: {:?}", result),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Пример: Получение данных с нескольких бирж с таймаутом

```rust
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct MarketData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume_24h: f64,
}

async fn fetch_market_data(exchange: &str, symbol: &str) -> MarketData {
    // Имитируем разные задержки для разных бирж
    let delay = match exchange {
        "binance" => 150,
        "kraken" => 300,
        "coinbase" => 450,
        "huobi" => 1500, // Медленная биржа
        _ => 500,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Генерируем разные цены для разных бирж
    let base_price = 42000.0;
    let spread = match exchange {
        "binance" => 10.0,
        "kraken" => 15.0,
        "coinbase" => 20.0,
        "huobi" => 25.0,
        _ => 12.0,
    };

    MarketData {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: base_price - spread / 2.0,
        ask: base_price + spread / 2.0,
        last_price: base_price,
        volume_24h: 1_000_000.0,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";
    let exchanges = vec!["binance", "kraken", "coinbase", "huobi"];
    let data_timeout = Duration::from_millis(500);

    println!("Собираем рыночные данные {} с {} бирж...", symbol, exchanges.len());
    println!("Таймаут: {:?}\n", data_timeout);

    let mut results: HashMap<String, Option<MarketData>> = HashMap::new();

    // Запрашиваем данные параллельно
    let mut handles = vec![];
    for exchange in &exchanges {
        let exchange = exchange.to_string();
        let symbol = symbol.to_string();

        handles.push(tokio::spawn(async move {
            let result = timeout(
                data_timeout,
                fetch_market_data(&exchange, &symbol)
            ).await;

            (exchange, result)
        }));
    }

    // Собираем результаты
    for handle in handles {
        let (exchange, result) = handle.await.unwrap();

        match result {
            Ok(data) => {
                println!("✓ {}: bid={:.2}, ask={:.2}, spread={:.2}",
                    data.exchange, data.bid, data.ask, data.ask - data.bid);
                results.insert(exchange, Some(data));
            }
            Err(_) => {
                println!("✗ {}: ТАЙМАУТ", exchange);
                results.insert(exchange, None);
            }
        }
    }

    // Анализируем полученные данные
    let valid_data: Vec<&MarketData> = results
        .values()
        .filter_map(|d| d.as_ref())
        .collect();

    if !valid_data.is_empty() {
        let best_bid = valid_data.iter().map(|d| d.bid).fold(f64::MIN, f64::max);
        let best_ask = valid_data.iter().map(|d| d.ask).fold(f64::MAX, f64::min);

        println!("\n=== Агрегированные данные ===");
        println!("Лучший bid: ${:.2}", best_bid);
        println!("Лучший ask: ${:.2}", best_ask);
        println!("Кросс-биржевой спред: ${:.2}", best_ask - best_bid);
    }
}
```

## timeout vs timeout_at

Tokio предоставляет две версии timeout:

```rust
use tokio::time::{timeout, timeout_at, Duration, Instant};

#[tokio::main]
async fn main() {
    // timeout: относительное время от текущего момента
    let result1 = timeout(
        Duration::from_secs(5),
        some_operation()
    ).await;

    // timeout_at: абсолютный момент времени (deadline)
    let deadline = Instant::now() + Duration::from_secs(5);
    let result2 = timeout_at(
        deadline,
        some_operation()
    ).await;
}

async fn some_operation() -> &'static str {
    tokio::time::sleep(Duration::from_secs(1)).await;
    "done"
}
```

## Пример: Торговый бот с таймаутами

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct TradingConfig {
    price_fetch_timeout: Duration,
    order_timeout: Duration,
    cancel_timeout: Duration,
}

impl Default for TradingConfig {
    fn default() -> Self {
        TradingConfig {
            price_fetch_timeout: Duration::from_millis(500),
            order_timeout: Duration::from_secs(2),
            cancel_timeout: Duration::from_secs(1),
        }
    }
}

struct TradingBot {
    config: TradingConfig,
    position: f64,
    balance: f64,
}

impl TradingBot {
    fn new(config: TradingConfig) -> Self {
        TradingBot {
            config,
            position: 0.0,
            balance: 10000.0,
        }
    }

    async fn get_price(&self, symbol: &str) -> Result<f64, String> {
        // Имитация запроса цены
        async fn fetch(symbol: &str) -> f64 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            42000.0 + (symbol.len() as f64 * 10.0)
        }

        timeout(self.config.price_fetch_timeout, fetch(symbol))
            .await
            .map_err(|_| format!("Таймаут при получении цены {}", symbol))
    }

    async fn place_order(&mut self, side: &str, quantity: f64, price: f64) -> Result<u64, String> {
        // Имитация размещения ордера
        async fn submit(_: &str, _: f64, _: f64) -> u64 {
            tokio::time::sleep(Duration::from_millis(800)).await;
            rand::random::<u64>() % 1000000
        }

        let order_id = timeout(
            self.config.order_timeout,
            submit(side, quantity, price)
        )
        .await
        .map_err(|_| "Таймаут при размещении ордера".to_string())?;

        // Обновляем позицию
        match side {
            "BUY" => {
                self.position += quantity;
                self.balance -= quantity * price;
            }
            "SELL" => {
                self.position -= quantity;
                self.balance += quantity * price;
            }
            _ => {}
        }

        Ok(order_id)
    }

    async fn cancel_order(&self, order_id: u64) -> Result<bool, String> {
        // Имитация отмены ордера
        async fn cancel(_: u64) -> bool {
            tokio::time::sleep(Duration::from_millis(300)).await;
            true
        }

        timeout(self.config.cancel_timeout, cancel(order_id))
            .await
            .map_err(|_| format!("Таймаут при отмене ордера {}", order_id))
    }

    fn status(&self) -> String {
        format!("Позиция: {:.4} BTC, Баланс: ${:.2}", self.position, self.balance)
    }
}

// Добавляем простой генератор случайных чисел
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T>() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

#[tokio::main]
async fn main() {
    let config = TradingConfig::default();
    let mut bot = TradingBot::new(config);

    println!("=== Торговый бот с таймаутами ===\n");
    println!("Начальное состояние: {}\n", bot.status());

    // Получаем цену
    match bot.get_price("BTC/USDT").await {
        Ok(price) => {
            println!("Текущая цена: ${:.2}", price);

            // Размещаем ордер
            match bot.place_order("BUY", 0.1, price).await {
                Ok(order_id) => {
                    println!("Ордер размещён: #{}", order_id);
                    println!("Новое состояние: {}", bot.status());
                }
                Err(e) => println!("Ошибка ордера: {}", e),
            }
        }
        Err(e) => println!("Ошибка цены: {}", e),
    }
}
```

## Обработка ошибок таймаута

```rust
use tokio::time::{timeout, Duration, error::Elapsed};

#[derive(Debug)]
enum TradingError {
    Timeout(String),
    NetworkError(String),
    ApiError(String),
}

impl From<Elapsed> for TradingError {
    fn from(_: Elapsed) -> Self {
        TradingError::Timeout("Операция превысила лимит времени".to_string())
    }
}

async fn robust_api_call<T, F, Fut>(
    operation_name: &str,
    max_duration: Duration,
    operation: F,
) -> Result<T, TradingError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, TradingError>>,
{
    match timeout(max_duration, operation()).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(elapsed) => {
            println!("⚠ Таймаут в '{}': {:?}", operation_name, elapsed);
            Err(TradingError::Timeout(format!(
                "{} не завершилась за {:?}",
                operation_name, max_duration
            )))
        }
    }
}

#[tokio::main]
async fn main() {
    let result = robust_api_call(
        "Получение баланса",
        Duration::from_secs(2),
        || async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, TradingError>(1000.0_f64)
        }
    ).await;

    match result {
        Ok(balance) => println!("Баланс: ${:.2}", balance),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}
```

## Практические задания

### Задание 1: Запрос котировок с таймаутом
Напиши функцию `get_best_quote`, которая:
- Запрашивает цены с трёх бирж параллельно
- Устанавливает таймаут 200мс на каждый запрос
- Возвращает лучшую (самую низкую) цену покупки из полученных ответов
- Обрабатывает случай, когда ни одна биржа не ответила вовремя

### Задание 2: Ордер с повторными попытками
Реализуй функцию `place_order_with_retry`, которая:
- Пытается разместить ордер с таймаутом 1 секунда
- При таймауте делает до 3 повторных попыток
- Увеличивает таймаут на каждой попытке (1с, 2с, 3с)
- Логирует каждую попытку

### Задание 3: Отмена ордера с гарантией
Создай функцию `safe_cancel_order`, которая:
- Пытается отменить ордер с таймаутом
- При таймауте проверяет статус ордера (исполнен/активен/отменён)
- Возвращает чёткий результат о судьбе ордера

## Домашнее задание

1. **Агрегатор цен с таймаутами**: Создай структуру `PriceAggregator`, которая:
   - Хранит список бирж для опроса
   - Имеет настраиваемый таймаут для каждой биржи
   - Возвращает VWAP (Volume Weighted Average Price) из всех ответивших бирж
   - Логирует, какие биржи не ответили вовремя

2. **Торговый движок с SLA**: Реализуй торговый движок, который:
   - Гарантирует ответ на любую операцию за 5 секунд
   - Для критических операций (отмена, стоп-лосс) использует короткие таймауты
   - При превышении SLA отправляет алерт (имитируй через println)

3. **Параллельные запросы с общим дедлайном**: Напиши функцию, которая:
   - Отправляет 5 параллельных запросов
   - Все запросы должны завершиться за 1 секунду (общий дедлайн)
   - Использует `timeout_at` для единого момента окончания
   - Возвращает все успешные результаты

4. **Graceful degradation**: Реализуй систему, которая:
   - При таймауте основного источника данных переключается на резервный
   - Ведёт статистику успешных/неуспешных запросов
   - Автоматически увеличивает таймауты при частых ошибках

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::timeout` | Ограничивает время выполнения асинхронной операции |
| `Duration` | Структура для представления промежутка времени |
| `Elapsed` | Ошибка, возвращаемая при истечении таймаута |
| `timeout_at` | Версия с абсолютным временем (дедлайном) |
| Graceful degradation | Паттерн плавной деградации при ошибках |

## Навигация

[← Предыдущий день](../190-tokio-time-timers-delays/ru.md) | [Следующий день →](../192-tokio-interval-periodic-tasks/ru.md)
