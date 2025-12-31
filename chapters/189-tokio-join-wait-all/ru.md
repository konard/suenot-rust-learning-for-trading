# День 189: tokio::join!: ждём всех

## Аналогия из трейдинга

Представь, что ты — алготрейдер, и перед открытием торгов тебе нужно собрать данные из нескольких источников одновременно:
- Текущие цены с биржи Binance
- Текущие цены с биржи Kraken
- Новости с финансовых порталов
- Данные твоего портфеля

Если запрашивать их последовательно — каждый запрос занимает ~500ms, и в сумме получится 2 секунды. Но если запустить все запросы **одновременно** и подождать, пока **все** завершатся — получится те же ~500ms! Это именно то, что делает `tokio::join!` — запускает несколько асинхронных операций параллельно и ждёт завершения **всех**.

В реальном трейдинге это критически важно:
- При арбитраже нужно получить цены со всех бирж одновременно
- При мониторинге портфеля нужно обновить все позиции сразу
- При анализе рынка нужно собрать данные по всем активам параллельно

## Что такое tokio::join!?

`tokio::join!` — это макрос, который:
1. Запускает несколько асинхронных выражений **одновременно**
2. Ждёт, пока **все** они завершатся
3. Возвращает кортеж с результатами всех операций

```
                 join!
                   |
      +------------+------------+
      |            |            |
      v            v            v
  Future 1     Future 2     Future 3
      |            |            |
      v            v            v
  Result 1     Result 2     Result 3
      |            |            |
      +------------+------------+
                   |
                   v
          (Result1, Result2, Result3)
```

## Почему не последовательный await?

Рассмотрим проблему на примере:

```rust
use tokio::time::{sleep, Duration};

async fn get_binance_price() -> f64 {
    sleep(Duration::from_millis(500)).await; // Имитация запроса
    42000.0
}

async fn get_kraken_price() -> f64 {
    sleep(Duration::from_millis(500)).await;
    42050.0
}

// МЕДЛЕННО: последовательное выполнение (~1000ms)
async fn fetch_prices_sequential() -> (f64, f64) {
    let binance = get_binance_price().await;  // 500ms
    let kraken = get_kraken_price().await;    // ещё 500ms
    (binance, kraken)
}

// БЫСТРО: параллельное выполнение (~500ms)
async fn fetch_prices_parallel() -> (f64, f64) {
    tokio::join!(
        get_binance_price(),
        get_kraken_price()
    )
}
```

## Базовый синтаксис

```rust
use tokio;

#[tokio::main]
async fn main() {
    // Базовое использование
    let (result1, result2, result3) = tokio::join!(
        async_operation_1(),
        async_operation_2(),
        async_operation_3()
    );

    // С режимом biased (фиксированный порядок опроса)
    let (result1, result2) = tokio::join!(
        biased;
        priority_operation(),
        secondary_operation()
    );
}
```

## Практический пример: Сбор данных с нескольких бирж

```rust
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug, Clone)]
struct PriceData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn fetch_binance_price(symbol: &str) -> PriceData {
    // Имитация API запроса к Binance
    sleep(Duration::from_millis(150)).await;

    PriceData {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42000.0,
        ask: 42010.0,
        timestamp: 1234567890,
    }
}

async fn fetch_kraken_price(symbol: &str) -> PriceData {
    // Имитация API запроса к Kraken
    sleep(Duration::from_millis(200)).await;

    PriceData {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 42005.0,
        ask: 42015.0,
        timestamp: 1234567890,
    }
}

async fn fetch_coinbase_price(symbol: &str) -> PriceData {
    // Имитация API запроса к Coinbase
    sleep(Duration::from_millis(180)).await;

    PriceData {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 41995.0,
        ask: 42008.0,
        timestamp: 1234567890,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USD";
    let start = Instant::now();

    // Запрашиваем цены со всех бирж одновременно
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_binance_price(symbol),
        fetch_kraken_price(symbol),
        fetch_coinbase_price(symbol)
    );

    let elapsed = start.elapsed();

    println!("Получены цены за {:?}:", elapsed);
    println!("  Binance: bid={}, ask={}", binance.bid, binance.ask);
    println!("  Kraken:  bid={}, ask={}", kraken.bid, kraken.ask);
    println!("  Coinbase: bid={}, ask={}", coinbase.bid, coinbase.ask);

    // Находим лучшие цены для арбитража
    let best_bid = [binance.bid, kraken.bid, coinbase.bid]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let best_ask = [binance.ask, kraken.ask, coinbase.ask]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);

    println!("\nЛучший bid: {}, лучший ask: {}", best_bid, best_ask);
    println!("Спред: {:.2}", best_ask - best_bid);
}
```

**Вывод:**
```
Получены цены за ~200ms:    (а не ~530ms при последовательном выполнении!)
  Binance: bid=42000, ask=42010
  Kraken:  bid=42005, ask=42015
  Coinbase: bid=41995, ask=42008

Лучший bid: 42005, лучший ask: 42008
Спред: 3.00
```

## Мониторинг портфеля с tokio::join!

```rust
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug, Clone)]
struct PositionStatus {
    symbol: String,
    quantity: f64,
    current_price: f64,
    pnl: f64,
    pnl_percent: f64,
}

async fn get_current_price(symbol: &str) -> f64 {
    // Имитация получения текущей цены
    sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => 42500.0,
        "ETH" => 2250.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

async fn calculate_position_status(position: Position) -> PositionStatus {
    let current_price = get_current_price(&position.symbol).await;
    let current_value = position.quantity * current_price;
    let cost_basis = position.quantity * position.avg_price;
    let pnl = current_value - cost_basis;
    let pnl_percent = (pnl / cost_basis) * 100.0;

    PositionStatus {
        symbol: position.symbol,
        quantity: position.quantity,
        current_price,
        pnl,
        pnl_percent,
    }
}

#[tokio::main]
async fn main() {
    let positions = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, avg_price: 40000.0 },
        Position { symbol: "ETH".to_string(), quantity: 5.0, avg_price: 2000.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, avg_price: 80.0 },
    ];

    println!("Обновление статуса портфеля...\n");

    // Обновляем все позиции одновременно
    let (btc_status, eth_status, sol_status) = tokio::join!(
        calculate_position_status(positions[0].clone()),
        calculate_position_status(positions[1].clone()),
        calculate_position_status(positions[2].clone())
    );

    let all_statuses = vec![btc_status, eth_status, sol_status];

    println!("{:<6} {:>10} {:>12} {:>12} {:>10}",
        "Актив", "Кол-во", "Цена", "PnL", "PnL %");
    println!("{}", "-".repeat(54));

    let mut total_pnl = 0.0;
    for status in &all_statuses {
        println!("{:<6} {:>10.4} {:>12.2} {:>12.2} {:>9.2}%",
            status.symbol,
            status.quantity,
            status.current_price,
            status.pnl,
            status.pnl_percent
        );
        total_pnl += status.pnl;
    }

    println!("{}", "-".repeat(54));
    println!("Общий PnL: ${:.2}", total_pnl);
}
```

## Обработка ошибок с tokio::join!

Важно понимать: `tokio::join!` ждёт завершения **всех** операций, даже если некоторые завершились с ошибкой. Для раннего прерывания при ошибке используйте `tokio::try_join!`.

### tokio::join! — ждём всех, даже при ошибках

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(exchange: &str) -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;

    match exchange {
        "binance" => Ok(42000.0),
        "kraken" => Err("Connection timeout".to_string()),
        "coinbase" => Ok(42050.0),
        _ => Err("Unknown exchange".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // join! ждёт ВСЕ результаты, даже если есть ошибки
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_price("binance"),
        fetch_price("kraken"),
        fetch_price("coinbase")
    );

    println!("Binance: {:?}", binance);   // Ok(42000.0)
    println!("Kraken: {:?}", kraken);     // Err("Connection timeout")
    println!("Coinbase: {:?}", coinbase); // Ok(42050.0)

    // Обрабатываем результаты индивидуально
    let valid_prices: Vec<f64> = [binance, kraken, coinbase]
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    if !valid_prices.is_empty() {
        let avg = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("Средняя цена (без ошибок): {:.2}", avg);
    }
}
```

### tokio::try_join! — прерываемся при первой ошибке

```rust
use tokio::time::{sleep, Duration};

async fn fetch_critical_price(exchange: &str) -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;

    match exchange {
        "binance" => Ok(42000.0),
        "kraken" => Err("API key expired".to_string()),
        "coinbase" => Ok(42050.0),
        _ => Err("Unknown exchange".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // try_join! прерывается при ПЕРВОЙ ошибке
    let result = tokio::try_join!(
        fetch_critical_price("binance"),
        fetch_critical_price("kraken"),
        fetch_critical_price("coinbase")
    );

    match result {
        Ok((binance, kraken, coinbase)) => {
            println!("Все цены получены успешно!");
            println!("Binance: {}, Kraken: {}, Coinbase: {}",
                binance, kraken, coinbase);
        }
        Err(e) => {
            println!("Ошибка при получении цен: {}", e);
            // Можно реализовать fallback логику
        }
    }
}
```

## Продвинутый пример: Торговый агрегатор

```rust
use tokio::time::{sleep, Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct OrderBook {
    exchange: String,
    symbol: String,
    bids: Vec<(f64, f64)>, // (price, quantity)
    asks: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
struct MarketNews {
    source: String,
    headline: String,
    sentiment: f64, // -1.0 to 1.0
}

#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    action: String,
    confidence: f64,
}

async fn fetch_order_book(exchange: &str, symbol: &str) -> OrderBook {
    sleep(Duration::from_millis(150)).await;

    OrderBook {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bids: vec![(42000.0, 1.5), (41990.0, 2.0), (41980.0, 3.0)],
        asks: vec![(42010.0, 1.0), (42020.0, 2.5), (42030.0, 1.8)],
    }
}

async fn fetch_news(symbol: &str) -> Vec<MarketNews> {
    sleep(Duration::from_millis(200)).await;

    vec![
        MarketNews {
            source: "CryptoNews".to_string(),
            headline: format!("{} показывает устойчивый рост", symbol),
            sentiment: 0.7,
        },
        MarketNews {
            source: "TradingView".to_string(),
            headline: format!("Технический анализ {} указывает на продолжение тренда", symbol),
            sentiment: 0.5,
        },
    ]
}

async fn calculate_signals(order_books: &[OrderBook], news: &[MarketNews]) -> Vec<TradingSignal> {
    sleep(Duration::from_millis(50)).await;

    // Простой анализ на основе спреда и новостей
    let avg_sentiment: f64 = news.iter().map(|n| n.sentiment).sum::<f64>()
        / news.len() as f64;

    let mut signals = Vec::new();

    for book in order_books {
        let best_bid = book.bids.first().map(|(p, _)| *p).unwrap_or(0.0);
        let best_ask = book.asks.first().map(|(p, _)| *p).unwrap_or(0.0);
        let spread_percent = (best_ask - best_bid) / best_bid * 100.0;

        let action = if avg_sentiment > 0.5 && spread_percent < 0.05 {
            "BUY"
        } else if avg_sentiment < -0.5 {
            "SELL"
        } else {
            "HOLD"
        };

        signals.push(TradingSignal {
            symbol: book.symbol.clone(),
            action: action.to_string(),
            confidence: avg_sentiment.abs() * (1.0 - spread_percent),
        });
    }

    signals
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USD";
    let start = Instant::now();

    println!("Агрегация рыночных данных для {}...\n", symbol);

    // Параллельно получаем данные из всех источников
    let (binance_book, kraken_book, news) = tokio::join!(
        fetch_order_book("Binance", symbol),
        fetch_order_book("Kraken", symbol),
        fetch_news(symbol)
    );

    let order_books = vec![binance_book.clone(), kraken_book.clone()];

    // Рассчитываем сигналы
    let signals = calculate_signals(&order_books, &news).await;

    let elapsed = start.elapsed();

    println!("=== Рыночные данные (получены за {:?}) ===\n", elapsed);

    for book in &order_books {
        println!("{} - {}", book.exchange, book.symbol);
        println!("  Лучший bid: {:.2}", book.bids[0].0);
        println!("  Лучший ask: {:.2}", book.asks[0].0);
        println!("  Спред: {:.2}\n", book.asks[0].0 - book.bids[0].0);
    }

    println!("=== Новости ===\n");
    for news_item in &news {
        println!("  [{}] {} (sentiment: {:.2})",
            news_item.source,
            news_item.headline,
            news_item.sentiment);
    }

    println!("\n=== Торговые сигналы ===\n");
    for signal in &signals {
        println!("  {} {} (confidence: {:.2}%)",
            signal.symbol,
            signal.action,
            signal.confidence * 100.0);
    }
}
```

## Сравнение: join! vs select! vs spawn

| Характеристика | `join!` | `select!` | `spawn` |
|----------------|---------|-----------|---------|
| Ждёт | Все операции | Первую завершённую | Не ждёт |
| Возвращает | Все результаты | Один результат | JoinHandle |
| Параллелизм | На одном потоке | На одном потоке | На разных потоках |
| Использование | Агрегация данных | Тайм-ауты, отмена | Фоновые задачи |

## Практические упражнения

### Упражнение 1: Мульти-биржевой арбитражный сканер

Создай функцию, которая одновременно получает цены с 5 бирж и находит арбитражные возможности:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    buy_exchange: String,
    sell_exchange: String,
    symbol: String,
    buy_price: f64,
    sell_price: f64,
    profit_percent: f64,
}

// Твоя реализация здесь
async fn find_arbitrage(symbol: &str) -> Vec<ArbitrageOpportunity> {
    todo!()
}
```

### Упражнение 2: Параллельное обновление лимитных ордеров

Реализуй функцию, которая одновременно обновляет несколько лимитных ордеров на разных биржах:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct LimitOrder {
    id: String,
    exchange: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct UpdateResult {
    order_id: String,
    success: bool,
    new_price: Option<f64>,
    error: Option<String>,
}

// Твоя реализация здесь
async fn update_orders_parallel(orders: Vec<LimitOrder>, new_prices: Vec<f64>)
    -> Vec<UpdateResult> {
    todo!()
}
```

### Упражнение 3: Таймаут для группы операций

Реализуй функцию, которая получает данные с нескольких бирж с общим таймаутом:

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct ExchangeData {
    exchange: String,
    price: Option<f64>,
    volume: Option<f64>,
}

// Твоя реализация здесь
async fn fetch_with_timeout(
    exchanges: Vec<&str>,
    symbol: &str,
    max_wait: Duration
) -> Result<Vec<ExchangeData>, String> {
    todo!()
}
```

### Упражнение 4: Постоянный мониторинг с интервалами

Реализуй систему, которая каждые N секунд одновременно обновляет данные со всех источников:

```rust
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct MarketSnapshot {
    timestamp: u64,
    prices: Vec<(String, f64)>,
    volumes: Vec<(String, f64)>,
}

// Твоя реализация здесь
async fn run_market_monitor(
    exchanges: Vec<String>,
    symbol: String,
    update_interval: Duration,
    callback: impl Fn(MarketSnapshot)
) {
    todo!()
}
```

## Домашнее задание

1. **Агрегатор ликвидности**: Создай систему, которая собирает данные стаканов заявок с 3+ бирж одновременно и строит объединённый стакан с лучшими ценами.

2. **Мониторинг рисков**: Реализуй параллельную проверку всех позиций портфеля на превышение лимитов риска. Каждая проверка должна запрашивать текущую цену и рассчитывать потенциальный убыток.

3. **Арбитражный бот**: Напиши бота, который:
   - Каждые 100ms получает цены со всех бирж параллельно
   - Находит арбитражные возможности
   - При обнаружении возможности одновременно отправляет ордера на обе биржи
   - Использует `try_join!` для отмены при ошибке на одной из бирж

4. **Система уведомлений**: Создай систему, которая одновременно отправляет уведомления через разные каналы (Telegram, Email, SMS) при срабатывании алерта. Используй `join!` чтобы дождаться подтверждения от всех каналов.

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::join!` | Ожидание завершения всех асинхронных операций |
| Параллельный сбор данных | Одновременный запрос к нескольким источникам |
| `biased` режим | Фиксированный порядок опроса futures |
| `tokio::try_join!` | Прерывание при первой ошибке |
| Агрегация результатов | Обработка кортежа результатов от join! |
| Конкурентность vs параллелизм | join! даёт конкурентность, spawn — параллелизм |

## Навигация

[← Предыдущий день](../188-tokio-select-first-respond/ru.md) | [Следующий день →](../190-tokio-time-timers-delays/ru.md)
