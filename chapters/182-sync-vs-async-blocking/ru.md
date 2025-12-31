# День 182: Sync vs Async: блокирующий vs неблокирующий код

## Аналогия из трейдинга

Представь два типа трейдеров на бирже:

**Синхронный (блокирующий) трейдер**: Отправляет ордер на биржу и замирает, глядя на экран, пока не получит подтверждение исполнения. Не может ничего делать, пока ждёт ответа — не анализирует графики, не проверяет другие позиции, просто ждёт.

**Асинхронный (неблокирующий) трейдер**: Отправляет ордер и сразу переключается на анализ следующего инструмента. Когда приходит подтверждение — обрабатывает его и продолжает работу. Может одновременно отслеживать десятки ордеров, не простаивая ни секунды.

В реальной торговле второй подход критически важен:
- Биржевые API могут отвечать с задержкой 50-500мс
- Нужно отслеживать котировки по многим инструментам
- Нельзя пропускать торговые сигналы, пока ждёшь ответа от биржи

## Что такое синхронный код?

Синхронный (blocking) код выполняется последовательно. Каждая операция ждёт завершения предыдущей:

```rust
use std::thread;
use std::time::Duration;

fn fetch_price_sync(symbol: &str) -> f64 {
    println!("[{}] Запрашиваю цену {}...",
             chrono::Local::now().format("%H:%M:%S%.3f"), symbol);

    // Имитируем сетевой запрос — поток БЛОКИРОВАН
    thread::sleep(Duration::from_millis(500));

    // Возвращаем "цену"
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

fn main() {
    let start = std::time::Instant::now();

    // Последовательные запросы — каждый ждёт предыдущего
    let btc = fetch_price_sync("BTC");
    let eth = fetch_price_sync("ETH");
    let sol = fetch_price_sync("SOL");

    println!("\nЦены: BTC=${}, ETH=${}, SOL=${}", btc, eth, sol);
    println!("Общее время: {:?}", start.elapsed());
    // Результат: ~1500мс (3 запроса × 500мс)
}
```

**Проблема**: Если биржа отвечает за 500мс, то получение цен по 10 инструментам займёт 5 секунд. За это время цена может уйти!

## Что такое асинхронный код?

Асинхронный (non-blocking) код позволяет начать операцию и заниматься другими делами, пока ждёшь результата:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price_async(symbol: &str) -> f64 {
    println!("[{}] Запрашиваю цену {}...",
             chrono::Local::now().format("%H:%M:%S%.3f"), symbol);

    // Имитируем сетевой запрос — поток НЕ блокирован!
    sleep(Duration::from_millis(500)).await;

    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    // Параллельные запросы — все выполняются одновременно
    let (btc, eth, sol) = tokio::join!(
        fetch_price_async("BTC"),
        fetch_price_async("ETH"),
        fetch_price_async("SOL"),
    );

    println!("\nЦены: BTC=${}, ETH=${}, SOL=${}", btc, eth, sol);
    println!("Общее время: {:?}", start.elapsed());
    // Результат: ~500мс (все запросы параллельно)
}
```

**Преимущество**: 3 запроса за время 1 запроса. При 10 инструментах это 500мс вместо 5 секунд!

## Как работает async/await в Rust?

### Future — обещание результата

`Future` в Rust — это вычисление, которое может быть ещё не завершено:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Упрощённая реализация Future для понимания
struct PriceFuture {
    symbol: String,
    completed: bool,
}

impl Future for PriceFuture {
    type Output = f64;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            // Результат готов
            Poll::Ready(42000.0)
        } else {
            // Ещё не готово, разбуди меня позже
            self.completed = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
```

### async fn создаёт Future

```rust
// Эта функция:
async fn get_balance() -> f64 {
    100.0
}

// Эквивалентна:
fn get_balance() -> impl Future<Output = f64> {
    async { 100.0 }
}
```

### .await приостанавливает выполнение

```rust
async fn trading_workflow() {
    // .await говорит: "Подожди результат, но не блокируй поток"
    let price = fetch_price_async("BTC").await;

    // Продолжаем только когда price готов
    if price > 41000.0 {
        place_order("BTC", "BUY", 0.1).await;
    }
}
```

## Сравнение подходов

```rust
use tokio::time::{sleep, Duration};
use std::thread;

// Синхронный подход — блокирует поток
fn sync_market_scan() {
    let symbols = vec!["BTC", "ETH", "SOL", "AVAX", "MATIC"];
    let start = std::time::Instant::now();

    for symbol in &symbols {
        // Каждый запрос блокирует на 200мс
        thread::sleep(Duration::from_millis(200));
        println!("{}: проверен", symbol);
    }

    println!("Синхронное сканирование: {:?}", start.elapsed());
    // ~1000мс
}

// Асинхронный подход — не блокирует
async fn async_market_scan() {
    let symbols = vec!["BTC", "ETH", "SOL", "AVAX", "MATIC"];
    let start = std::time::Instant::now();

    let futures: Vec<_> = symbols.iter().map(|symbol| {
        async move {
            sleep(Duration::from_millis(200)).await;
            println!("{}: проверен", symbol);
            symbol
        }
    }).collect();

    // Все запросы выполняются параллельно
    futures::future::join_all(futures).await;

    println!("Асинхронное сканирование: {:?}", start.elapsed());
    // ~200мс
}
```

## Практический пример: торговый бот

```rust
use tokio::time::{sleep, Duration, interval};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

// Асинхронное получение рыночных данных
async fn fetch_market_data(symbol: &str) -> MarketData {
    // Имитируем API запрос
    sleep(Duration::from_millis(100)).await;

    MarketData {
        symbol: symbol.to_string(),
        price: 42000.0 + (rand::random::<f64>() * 1000.0),
        volume: 1000.0 + (rand::random::<f64>() * 500.0),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}

// Асинхронная отправка ордера
async fn submit_order(order: Order) -> Result<String, String> {
    // Имитируем отправку на биржу
    sleep(Duration::from_millis(50)).await;

    Ok(format!("ORDER-{}", rand::random::<u32>()))
}

// Асинхронная стратегия
async fn momentum_strategy(
    data: MarketData,
    prev_price: Option<f64>,
) -> Option<Order> {
    let Some(prev) = prev_price else {
        return None;
    };

    let change_pct = (data.price - prev) / prev * 100.0;

    // Покупаем при росте > 0.5%
    if change_pct > 0.5 {
        Some(Order {
            symbol: data.symbol,
            side: "BUY".to_string(),
            quantity: 0.01,
            price: data.price,
        })
    }
    // Продаём при падении > 0.5%
    else if change_pct < -0.5 {
        Some(Order {
            symbol: data.symbol,
            side: "SELL".to_string(),
            quantity: 0.01,
            price: data.price,
        })
    } else {
        None
    }
}

// Главный цикл торгового бота
async fn trading_bot() {
    let symbols = vec!["BTC", "ETH", "SOL"];
    let mut prev_prices: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    let mut interval = interval(Duration::from_secs(1));

    for _ in 0..10 { // 10 итераций для демонстрации
        interval.tick().await;

        // Получаем данные по всем инструментам ПАРАЛЛЕЛЬНО
        let data_futures: Vec<_> = symbols.iter()
            .map(|s| fetch_market_data(s))
            .collect();

        let market_data = futures::future::join_all(data_futures).await;

        // Анализируем каждый инструмент
        for data in market_data {
            let prev = prev_prices.get(&data.symbol).copied();

            if let Some(order) = momentum_strategy(data.clone(), prev).await {
                // Отправляем ордер асинхронно
                match submit_order(order).await {
                    Ok(id) => println!("Ордер размещён: {}", id),
                    Err(e) => println!("Ошибка: {}", e),
                }
            }

            prev_prices.insert(data.symbol.clone(), data.price);
        }
    }
}
```

## Когда использовать sync, а когда async?

### Используй синхронный код когда:

```rust
// 1. Вычисления без I/O
fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    prices.iter()
        .rev()
        .take(period)
        .sum::<f64>() / period as f64
}

// 2. Простые скрипты без параллельности
fn main() {
    let data = vec![100.0, 101.0, 102.0, 101.5, 103.0];
    let sma = calculate_sma(&data, 3);
    println!("SMA(3) = {}", sma);
}

// 3. CPU-интенсивные задачи
fn backtest_strategy(historical_data: &[f64]) -> f64 {
    // Тяжёлые вычисления лучше в обычных потоках
    historical_data.iter()
        .map(|p| complex_calculation(*p))
        .sum()
}
```

### Используй асинхронный код когда:

```rust
// 1. Работа с сетью
async fn fetch_orderbook(symbol: &str) -> OrderBook {
    let response = reqwest::get(format!(
        "https://api.exchange.com/orderbook/{}", symbol
    )).await.unwrap();

    response.json().await.unwrap()
}

// 2. Множество параллельных I/O операций
async fn monitor_portfolio(symbols: Vec<String>) {
    let mut handles = vec![];

    for symbol in symbols {
        handles.push(tokio::spawn(async move {
            loop {
                let price = fetch_price(&symbol).await;
                update_portfolio(&symbol, price).await;
                sleep(Duration::from_secs(1)).await;
            }
        }));
    }

    futures::future::join_all(handles).await;
}

// 3. Серверы и обработчики соединений
async fn websocket_handler(ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        // Обрабатываем сообщения от биржи
        process_market_update(msg).await;
    }
}
```

## Распространённые ошибки

### 1. Блокирующий код в async контексте

```rust
// ПЛОХО: thread::sleep блокирует весь runtime
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1)); // Блокирует!
}

// ХОРОШО: tokio::time::sleep не блокирует
async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### 2. CPU-интенсивные задачи в async

```rust
// ПЛОХО: тяжёлые вычисления блокируют async runtime
async fn bad_backtest() {
    let result = heavy_computation(); // Блокирует на секунды!
    result
}

// ХОРОШО: выносим в отдельный поток
async fn good_backtest() {
    let result = tokio::task::spawn_blocking(|| {
        heavy_computation()
    }).await.unwrap();
    result
}
```

### 3. Забытый .await

```rust
async fn forgotten_await() {
    let future = fetch_price("BTC"); // Возвращает Future, не f64!

    // Ошибка: future никогда не выполнится
    // println!("Price: {}", future); // Не скомпилируется

    // Правильно:
    let price = future.await;
    println!("Price: {}", price);
}
```

## Практический пример: мультибиржевой арбитраж

```rust
use tokio::time::{timeout, Duration};
use futures::future::join_all;

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

async fn fetch_binance_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(80)).await;
    Ok(ExchangePrice {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42000.0,
        ask: 42005.0,
    })
}

async fn fetch_kraken_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(120)).await;
    Ok(ExchangePrice {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 42010.0,
        ask: 42020.0,
    })
}

async fn fetch_coinbase_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(ExchangePrice {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 41995.0,
        ask: 42008.0,
    })
}

async fn find_arbitrage(symbol: &str) -> Option<(String, String, f64)> {
    // Запрашиваем цены со всех бирж ПАРАЛЛЕЛЬНО с таймаутом
    let fetch_timeout = Duration::from_millis(500);

    let results = join_all(vec![
        timeout(fetch_timeout, fetch_binance_price(symbol)),
        timeout(fetch_timeout, fetch_kraken_price(symbol)),
        timeout(fetch_timeout, fetch_coinbase_price(symbol)),
    ]).await;

    // Собираем успешные результаты
    let prices: Vec<ExchangePrice> = results
        .into_iter()
        .filter_map(|r| r.ok().and_then(|inner| inner.ok()))
        .collect();

    if prices.len() < 2 {
        return None;
    }

    // Ищем лучшие цены покупки и продажи
    let best_bid = prices.iter()
        .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap())?;
    let best_ask = prices.iter()
        .min_by(|a, b| a.ask.partial_cmp(&b.ask).unwrap())?;

    // Арбитраж: покупаем дёшево, продаём дорого
    let spread = best_bid.bid - best_ask.ask;
    let spread_pct = spread / best_ask.ask * 100.0;

    if spread > 0.0 {
        Some((
            format!("Купить на {} @ {}", best_ask.exchange, best_ask.ask),
            format!("Продать на {} @ {}", best_bid.exchange, best_bid.bid),
            spread_pct,
        ))
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    println!("Поиск арбитражных возможностей...\n");

    let symbols = vec!["BTC", "ETH", "SOL"];

    // Ищем арбитраж по всем парам параллельно
    let arb_futures: Vec<_> = symbols.iter()
        .map(|s| find_arbitrage(s))
        .collect();

    let results = join_all(arb_futures).await;

    for (symbol, result) in symbols.iter().zip(results) {
        match result {
            Some((buy, sell, pct)) => {
                println!("{}: Арбитраж найден! (+{:.3}%)", symbol, pct);
                println!("  {}", buy);
                println!("  {}", sell);
            }
            None => {
                println!("{}: Арбитраж не найден", symbol);
            }
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Синхронный код | Выполняется последовательно, блокирует поток |
| Асинхронный код | Не блокирует, позволяет параллельное выполнение |
| `async fn` | Создаёт функцию, возвращающую Future |
| `.await` | Приостанавливает выполнение до готовности результата |
| `Future` | Значение, которое будет доступно в будущем |
| `tokio::join!` | Запускает несколько Future параллельно |
| `tokio::spawn` | Создаёт независимую асинхронную задачу |

## Домашнее задание

1. **Параллельный сканер рынка**: Напиши программу, которая:
   - Получает цены по 10 инструментам параллельно
   - Вычисляет изменение за день для каждого
   - Выводит топ-3 растущих и топ-3 падающих
   - Сравни время синхронной и асинхронной версий

2. **Стриминг котировок**: Реализуй асинхронный стрим, который:
   - Генерирует случайные изменения цены каждые 100мс
   - Вычисляет скользящее среднее (SMA) в реальном времени
   - Отправляет сигнал при пересечении ценой SMA

3. **Мультибиржевой коннектор**: Создай структуру `ExchangeConnector`, которая:
   - Подключается к нескольким "биржам" асинхронно
   - Отслеживает состояние подключения
   - Автоматически переподключается при разрыве
   - Агрегирует данные со всех бирж

4. **Асинхронный бэктестер**: Реализуй систему, которая:
   - Загружает исторические данные асинхронно
   - Запускает несколько стратегий параллельно
   - Собирает результаты и сравнивает их производительность

## Навигация

[← Предыдущий день](../181-tokio-runtime/ru.md) | [Следующий день →](../183-async-await-syntax/ru.md)
