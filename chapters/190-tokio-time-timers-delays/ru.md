# День 190: tokio::time: таймеры и задержки

## Аналогия из трейдинга

Представь, что ты работаешь трейдером на бирже. Время — это твой главный инструмент:

- **Задержка (sleep)** — как пауза между проверками цен. Ты не хочешь спамить биржу запросами каждую миллисекунду, поэтому ждёшь секунду перед следующим запросом.
- **Таймаут (timeout)** — как стоп-лосс по времени. Если биржа не ответила за 5 секунд — считаем, что что-то пошло не так.
- **Интервал (interval)** — как регулярный мониторинг портфеля. Каждые 10 секунд ты проверяешь текущие позиции.
- **Instant** — как точная метка времени для измерения скорости исполнения ордера.

В асинхронном программировании работа со временем отличается от синхронного кода: вместо блокировки потока мы "усыпляем" задачу, позволяя другим задачам работать.

## Основы tokio::time

`tokio::time` предоставляет набор инструментов для работы со временем в асинхронном коде:

| Инструмент | Назначение | Аналогия в трейдинге |
|------------|------------|----------------------|
| `sleep` | Пауза на заданное время | Ждём перед следующим запросом к API |
| `timeout` | Ограничение времени операции | Отменяем ордер если не исполнен за N секунд |
| `interval` | Периодическое выполнение | Проверка портфеля каждые 10 секунд |
| `Instant` | Точное измерение времени | Замер скорости исполнения |

## sleep: асинхронная пауза

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Запрашиваем цену BTC...");
    let price = get_btc_price().await;
    println!("Цена BTC: ${}", price);

    // Ждём 1 секунду перед следующим запросом (rate limiting)
    println!("Ожидание перед следующим запросом...");
    sleep(Duration::from_secs(1)).await;

    println!("Запрашиваем цену снова...");
    let new_price = get_btc_price().await;
    println!("Новая цена BTC: ${}", new_price);
}

async fn get_btc_price() -> f64 {
    // Имитация запроса к API
    sleep(Duration::from_millis(100)).await;
    42_000.50
}
```

### Важное отличие от std::thread::sleep

```rust
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start = Instant::now();

    // Эти задачи выполняются ПАРАЛЛЕЛЬНО!
    let (price1, price2, price3) = tokio::join!(
        fetch_price_with_delay("BTC", 1000),
        fetch_price_with_delay("ETH", 1500),
        fetch_price_with_delay("SOL", 800),
    );

    println!("BTC: ${}, ETH: ${}, SOL: ${}", price1, price2, price3);
    println!("Общее время: {:?}", start.elapsed());
    // Выведет ~1.5 секунды, а не 3.3 секунды!
}

async fn fetch_price_with_delay(symbol: &str, delay_ms: u64) -> f64 {
    // Имитация задержки сети
    sleep(Duration::from_millis(delay_ms)).await;

    match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}
```

**Ключевое различие:**
- `std::thread::sleep` блокирует весь поток — никто другой не может работать
- `tokio::time::sleep` отдаёт управление рантайму — другие задачи продолжают работу

## timeout: ограничение времени ожидания

Таймаут критически важен в трейдинге — если биржа не отвечает, нужно быстро принять решение.

```rust
use tokio::time::{timeout, Duration, sleep};

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42_000.0,
        quantity: 0.1,
    };

    println!("Отправляем ордер #{} на биржу...", order.id);

    // Даём 5 секунд на исполнение ордера
    match timeout(Duration::from_secs(5), execute_order(&order)).await {
        Ok(Ok(trade_id)) => {
            println!("Ордер исполнен! Trade ID: {}", trade_id);
        }
        Ok(Err(e)) => {
            println!("Ошибка исполнения: {}", e);
        }
        Err(_) => {
            println!("ТАЙМАУТ! Ордер не исполнен за 5 секунд.");
            println!("Отменяем ордер #{}", order.id);
            cancel_order(order.id).await;
        }
    }
}

async fn execute_order(order: &Order) -> Result<u64, String> {
    // Имитация медленной биржи (6 секунд — дольше таймаута)
    sleep(Duration::from_secs(6)).await;
    Ok(12345)
}

async fn cancel_order(order_id: u64) {
    println!("Ордер {} отменён", order_id);
}
```

### Таймаут с retry логикой

```rust
use tokio::time::{timeout, Duration, sleep};

async fn fetch_price_with_retry(symbol: &str, max_retries: u32) -> Result<f64, String> {
    for attempt in 1..=max_retries {
        println!("Попытка {} получить цену {}...", attempt, symbol);

        match timeout(Duration::from_secs(2), fetch_price(symbol)).await {
            Ok(Ok(price)) => {
                return Ok(price);
            }
            Ok(Err(e)) => {
                println!("Ошибка API: {}", e);
            }
            Err(_) => {
                println!("Таймаут при запросе цены");
            }
        }

        if attempt < max_retries {
            // Экспоненциальная задержка между попытками
            let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
            println!("Ждём {:?} перед следующей попыткой...", delay);
            sleep(delay).await;
        }
    }

    Err(format!("Не удалось получить цену {} за {} попыток", symbol, max_retries))
}

async fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Имитация нестабильного API
    sleep(Duration::from_millis(500)).await;

    // 30% шанс ошибки
    if rand::random::<f32>() < 0.3 {
        return Err("API временно недоступен".to_string());
    }

    Ok(42_000.0)
}
```

## interval: периодические задачи

Интервалы идеальны для регулярного мониторинга в трейдинге.

```rust
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
struct Portfolio {
    btc_balance: f64,
    eth_balance: f64,
    usdt_balance: f64,
}

#[tokio::main]
async fn main() {
    // Создаём интервал в 5 секунд
    let mut interval = interval(Duration::from_secs(5));

    let portfolio = Portfolio {
        btc_balance: 1.5,
        eth_balance: 10.0,
        usdt_balance: 50_000.0,
    };

    println!("Запускаем мониторинг портфеля...");

    // Мониторим 30 секунд (6 тиков)
    for tick_number in 1..=6 {
        // tick() ждёт до следующего интервала
        interval.tick().await;

        let btc_price = get_price("BTC").await;
        let eth_price = get_price("ETH").await;

        let total_value = portfolio.btc_balance * btc_price
            + portfolio.eth_balance * eth_price
            + portfolio.usdt_balance;

        println!(
            "[Тик {}] Стоимость портфеля: ${:.2}",
            tick_number, total_value
        );
    }

    println!("Мониторинг завершён");
}

async fn get_price(symbol: &str) -> f64 {
    // Имитация получения цены с небольшой вариацией
    let base_price = match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_200.0,
        _ => 1.0,
    };

    // Добавляем случайную вариацию ±1%
    let variation = (rand::random::<f64>() - 0.5) * 0.02;
    base_price * (1.0 + variation)
}
```

### MissedTickBehavior: что делать с пропущенными тиками

```rust
use tokio::time::{interval, Duration, MissedTickBehavior, sleep};

#[tokio::main]
async fn main() {
    // Интервал каждые 100мс
    let mut price_check = interval(Duration::from_millis(100));

    // Что делать если обработка заняла дольше интервала?
    // Burst — выполнить пропущенные тики как можно быстрее
    // Delay — пропустить и продолжить с нового интервала (по умолчанию)
    // Skip — пропустить пропущенные тики
    price_check.set_missed_tick_behavior(MissedTickBehavior::Skip);

    for i in 1..=10 {
        price_check.tick().await;
        println!("Проверка цены #{}", i);

        // Иногда обработка занимает больше времени
        if i == 3 {
            println!("Долгая обработка...");
            sleep(Duration::from_millis(350)).await; // Пропустим ~3 тика
        }
    }
}
```

## Instant: точное измерение времени

В трейдинге миллисекунды важны — нужно точно измерять задержки.

```rust
use tokio::time::{Instant, sleep, Duration};

#[derive(Debug)]
struct OrderExecutionMetrics {
    order_id: u64,
    submission_time: Duration,
    confirmation_time: Duration,
    total_latency: Duration,
}

#[tokio::main]
async fn main() {
    let metrics = measure_order_latency().await;

    println!("=== Метрики исполнения ордера #{} ===", metrics.order_id);
    println!("Время отправки: {:?}", metrics.submission_time);
    println!("Время подтверждения: {:?}", metrics.confirmation_time);
    println!("Общая задержка: {:?}", metrics.total_latency);

    // Алерт если задержка слишком высокая
    if metrics.total_latency > Duration::from_millis(500) {
        println!("ВНИМАНИЕ: Высокая задержка! Проверьте соединение.");
    }
}

async fn measure_order_latency() -> OrderExecutionMetrics {
    let start = Instant::now();

    // Фаза 1: Отправка ордера
    submit_order().await;
    let submission_time = start.elapsed();

    // Фаза 2: Ожидание подтверждения
    let confirmation_start = Instant::now();
    wait_for_confirmation().await;
    let confirmation_time = confirmation_start.elapsed();

    OrderExecutionMetrics {
        order_id: 1,
        submission_time,
        confirmation_time,
        total_latency: start.elapsed(),
    }
}

async fn submit_order() {
    sleep(Duration::from_millis(50)).await;
}

async fn wait_for_confirmation() {
    sleep(Duration::from_millis(150)).await;
}
```

## Практический пример: Rate Limiter для API биржи

Биржи ограничивают количество запросов. Вот как правильно реализовать rate limiting:

```rust
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;

struct RateLimiter {
    requests_per_second: u32,
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        RateLimiter {
            requests_per_second,
            last_request: Mutex::new(Instant::now()),
            min_interval: Duration::from_secs(1) / requests_per_second,
        }
    }

    async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }

        *last = Instant::now();
    }
}

struct ExchangeClient {
    rate_limiter: Arc<RateLimiter>,
}

impl ExchangeClient {
    fn new(max_requests_per_second: u32) -> Self {
        ExchangeClient {
            rate_limiter: Arc::new(RateLimiter::new(max_requests_per_second)),
        }
    }

    async fn get_price(&self, symbol: &str) -> f64 {
        // Ждём если нужно соблюсти rate limit
        self.rate_limiter.wait().await;

        // Делаем запрос
        println!("Запрос цены {}", symbol);
        sleep(Duration::from_millis(50)).await; // Имитация сетевого запроса

        42_000.0
    }

    async fn get_order_book(&self, symbol: &str) -> Vec<(f64, f64)> {
        self.rate_limiter.wait().await;

        println!("Запрос стакана {}", symbol);
        sleep(Duration::from_millis(100)).await;

        vec![(42_000.0, 1.5), (41_999.0, 2.0), (41_998.0, 0.5)]
    }
}

#[tokio::main]
async fn main() {
    // Биржа разрешает 5 запросов в секунду
    let client = ExchangeClient::new(5);

    let start = Instant::now();

    // Делаем 10 запросов — должно занять ~2 секунды
    for i in 1..=10 {
        client.get_price("BTC").await;
        println!("Запрос {} выполнен, прошло {:?}", i, start.elapsed());
    }

    println!("Все запросы выполнены за {:?}", start.elapsed());
}
```

## Практический пример: Мониторинг с таймаутами

```rust
use tokio::time::{interval, timeout, Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: Instant,
}

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate(PriceUpdate),
    ConnectionLost,
    Reconnected,
}

async fn price_monitor(tx: mpsc::Sender<TradingEvent>) {
    let symbols = vec!["BTC", "ETH", "SOL"];
    let mut check_interval = interval(Duration::from_secs(1));

    loop {
        check_interval.tick().await;

        for symbol in &symbols {
            // Даём 500мс на получение цены
            let result = timeout(
                Duration::from_millis(500),
                fetch_price_from_exchange(symbol)
            ).await;

            match result {
                Ok(Ok(price)) => {
                    let update = PriceUpdate {
                        symbol: symbol.to_string(),
                        price,
                        timestamp: Instant::now(),
                    };
                    tx.send(TradingEvent::PriceUpdate(update)).await.ok();
                }
                Ok(Err(_)) | Err(_) => {
                    println!("Проблема с получением цены {}", symbol);
                    tx.send(TradingEvent::ConnectionLost).await.ok();
                }
            }
        }
    }
}

async fn fetch_price_from_exchange(symbol: &str) -> Result<f64, String> {
    // Имитация сетевого запроса
    tokio::time::sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => Ok(42_000.0 + rand::random::<f64>() * 100.0),
        "ETH" => Ok(2_200.0 + rand::random::<f64>() * 10.0),
        "SOL" => Ok(95.0 + rand::random::<f64>() * 2.0),
        _ => Err("Unknown symbol".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    // Запускаем монитор цен в отдельной задаче
    tokio::spawn(price_monitor(tx));

    // Обрабатываем события 10 секунд
    let deadline = Instant::now() + Duration::from_secs(10);

    while Instant::now() < deadline {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(event)) => {
                match event {
                    TradingEvent::PriceUpdate(update) => {
                        println!(
                            "{}: ${:.2}",
                            update.symbol, update.price
                        );
                    }
                    TradingEvent::ConnectionLost => {
                        println!("Соединение потеряно!");
                    }
                    TradingEvent::Reconnected => {
                        println!("Соединение восстановлено");
                    }
                }
            }
            Ok(None) => {
                println!("Канал закрыт");
                break;
            }
            Err(_) => {
                println!("Нет событий 2 секунды");
            }
        }
    }

    println!("Мониторинг завершён");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `sleep` | Асинхронная пауза без блокировки потока |
| `timeout` | Ограничение времени выполнения операции |
| `interval` | Периодическое выполнение задач |
| `Instant` | Точное измерение прошедшего времени |
| `MissedTickBehavior` | Стратегия обработки пропущенных интервалов |
| Rate Limiting | Контроль частоты запросов к API |

## Домашнее задание

1. **Умный монитор цен**: Создай систему, которая:
   - Проверяет цену BTC каждые 5 секунд
   - Если цена изменилась больше чем на 1% — выводит алерт
   - Если API не отвечает 3 раза подряд — переходит в режим пониженной частоты (каждые 30 секунд)

2. **Rate Limiter с burst**: Модифицируй `RateLimiter` чтобы он:
   - Позволял "накапливать" неиспользованные запросы (до 10)
   - Мог отправить burst из накопленных запросов

3. **Измерение latency**: Напиши систему, которая:
   - Измеряет среднюю, минимальную и максимальную задержку API
   - Сохраняет статистику за последние 100 запросов
   - Выводит отчёт каждые 30 секунд

4. **Таймаут с graceful degradation**: Реализуй функцию получения цены, которая:
   - Сначала пробует быстрый API (таймаут 100мс)
   - При неудаче пробует резервный API (таймаут 500мс)
   - При полной неудаче возвращает последнюю известную цену

## Навигация

[← Предыдущий день](../189-tokio-join/ru.md) | [Следующий день →](../191-tokio-timeout/ru.md)
