# День 183: async/await синтаксис

## Аналогия из трейдинга

Представь, что ты управляешь торговым ботом, который должен одновременно:
- Получать котировки с 5 разных бирж
- Отслеживать свои открытые позиции
- Проверять лимиты риска
- Отправлять уведомления

**Синхронный подход:** Бот последовательно опрашивает каждую биржу, ждёт ответа, потом следующую... Пока он ждёт ответа от Binance, цена на Kraken уже изменилась!

**Асинхронный подход с async/await:** Бот отправляет все запросы одновременно и обрабатывает ответы по мере их поступления. Пока ждём ответа от одной биржи, работаем с данными от другой.

Это как опытный трейдер, который следит за несколькими мониторами одновременно, а не смотрит на них по очереди.

## Что такое async/await?

`async` и `await` — это синтаксический сахар для работы с асинхронным кодом в Rust:

- **async** — объявляет функцию или блок как асинхронный
- **await** — приостанавливает выполнение до получения результата Future

```rust
// Синхронная функция — блокирует поток
fn fetch_price_sync() -> f64 {
    // Ждём ответа, ничего не делаем
    42000.0
}

// Асинхронная функция — возвращает Future
async fn fetch_price_async() -> f64 {
    // Позволяет другим задачам выполняться пока ждём
    42000.0
}
```

## Базовый пример: получение цены актива

```rust
use tokio::time::{sleep, Duration};

// Имитируем запрос к API биржи
async fn fetch_btc_price(exchange: &str) -> f64 {
    println!("[{}] Запрашиваю цену BTC...", exchange);

    // Имитируем задержку сети (1 секунда)
    sleep(Duration::from_millis(1000)).await;

    // Возвращаем "цену" в зависимости от биржи
    match exchange {
        "Binance" => 42150.0,
        "Kraken" => 42145.0,
        "Coinbase" => 42155.0,
        _ => 42000.0,
    }
}

#[tokio::main]
async fn main() {
    println!("Получаю цену BTC с Binance...");

    let price = fetch_btc_price("Binance").await;

    println!("Цена BTC: ${:.2}", price);
}
```

## Параллельное выполнение с tokio::join!

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(symbol: &str, exchange: &str) -> f64 {
    println!("[{}] Запрашиваю {}...", exchange, symbol);
    sleep(Duration::from_millis(500)).await;

    match (symbol, exchange) {
        ("BTC", "Binance") => 42150.0,
        ("BTC", "Kraken") => 42145.0,
        ("ETH", "Binance") => 2250.0,
        ("ETH", "Kraken") => 2248.0,
        _ => 0.0,
    }
}

#[tokio::main]
async fn main() {
    println!("=== Последовательное выполнение ===");
    let start = std::time::Instant::now();

    let btc_binance = fetch_price("BTC", "Binance").await;
    let btc_kraken = fetch_price("BTC", "Kraken").await;
    let eth_binance = fetch_price("ETH", "Binance").await;
    let eth_kraken = fetch_price("ETH", "Kraken").await;

    println!("Последовательно заняло: {:?}", start.elapsed());

    println!("\n=== Параллельное выполнение ===");
    let start = std::time::Instant::now();

    // tokio::join! выполняет все futures параллельно
    let (btc_b, btc_k, eth_b, eth_k) = tokio::join!(
        fetch_price("BTC", "Binance"),
        fetch_price("BTC", "Kraken"),
        fetch_price("ETH", "Binance"),
        fetch_price("ETH", "Kraken")
    );

    println!("Параллельно заняло: {:?}", start.elapsed());

    println!("\nЦены:");
    println!("BTC Binance: ${:.2}", btc_b);
    println!("BTC Kraken: ${:.2}", btc_k);
    println!("ETH Binance: ${:.2}", eth_b);
    println!("ETH Kraken: ${:.2}", eth_k);
}
```

## Обработка ошибок в async функциях

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct ExchangeError {
    exchange: String,
    message: String,
}

async fn fetch_price_safe(symbol: &str, exchange: &str) -> Result<f64, ExchangeError> {
    println!("[{}] Запрашиваю {}...", exchange, symbol);
    sleep(Duration::from_millis(300)).await;

    // Имитируем случайные ошибки
    if exchange == "OfflineExchange" {
        return Err(ExchangeError {
            exchange: exchange.to_string(),
            message: "Биржа недоступна".to_string(),
        });
    }

    match (symbol, exchange) {
        ("BTC", "Binance") => Ok(42150.0),
        ("BTC", "Kraken") => Ok(42145.0),
        ("ETH", "Binance") => Ok(2250.0),
        _ => Err(ExchangeError {
            exchange: exchange.to_string(),
            message: format!("Символ {} не найден", symbol),
        }),
    }
}

#[tokio::main]
async fn main() {
    // Обработка Result с ? оператором
    match get_best_btc_price().await {
        Ok(price) => println!("Лучшая цена BTC: ${:.2}", price),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}

async fn get_best_btc_price() -> Result<f64, ExchangeError> {
    let binance_price = fetch_price_safe("BTC", "Binance").await?;
    let kraken_price = fetch_price_safe("BTC", "Kraken").await?;

    Ok(binance_price.min(kraken_price))
}
```

## tokio::select! — первый готовый результат

В трейдинге часто нужен самый быстрый ответ:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_from_binance() -> f64 {
    sleep(Duration::from_millis(150)).await;
    42150.0
}

async fn fetch_from_kraken() -> f64 {
    sleep(Duration::from_millis(100)).await;
    42145.0
}

async fn fetch_from_coinbase() -> f64 {
    sleep(Duration::from_millis(200)).await;
    42155.0
}

#[tokio::main]
async fn main() {
    println!("Жду первый ответ от любой биржи...");

    let price = tokio::select! {
        price = fetch_from_binance() => {
            println!("Binance ответил первым!");
            price
        }
        price = fetch_from_kraken() => {
            println!("Kraken ответил первым!");
            price
        }
        price = fetch_from_coinbase() => {
            println!("Coinbase ответил первым!");
            price
        }
    };

    println!("Получена цена: ${:.2}", price);
}
```

## Практический пример: Мониторинг портфеля

```rust
use tokio::time::{sleep, Duration, interval};

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

#[derive(Debug)]
struct PositionStatus {
    symbol: String,
    current_price: f64,
    pnl: f64,
    pnl_percent: f64,
}

async fn fetch_current_price(symbol: &str) -> f64 {
    // Имитируем API запрос
    sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => 42500.0 + (rand_simple() * 1000.0 - 500.0),
        "ETH" => 2300.0 + (rand_simple() * 100.0 - 50.0),
        "SOL" => 95.0 + (rand_simple() * 10.0 - 5.0),
        _ => 100.0,
    }
}

// Простой генератор псевдослучайных чисел
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

async fn check_position(position: &Position) -> PositionStatus {
    let current_price = fetch_current_price(&position.symbol).await;
    let pnl = (current_price - position.entry_price) * position.quantity;
    let pnl_percent = ((current_price / position.entry_price) - 1.0) * 100.0;

    PositionStatus {
        symbol: position.symbol.clone(),
        current_price,
        pnl,
        pnl_percent,
    }
}

async fn monitor_portfolio(positions: Vec<Position>) {
    println!("=== Мониторинг портфеля ===\n");

    // Проверяем все позиции параллельно
    let mut futures = Vec::new();
    for position in &positions {
        futures.push(check_position(position));
    }

    let results = futures::future::join_all(futures).await;

    let mut total_pnl = 0.0;

    for status in results {
        let emoji = if status.pnl >= 0.0 { "+" } else { "" };
        println!(
            "{}: ${:.2} | PnL: {}${:.2} ({}{:.2}%)",
            status.symbol,
            status.current_price,
            emoji,
            status.pnl,
            emoji,
            status.pnl_percent
        );
        total_pnl += status.pnl;
    }

    println!("\nОбщий PnL: ${:.2}", total_pnl);
}

#[tokio::main]
async fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 5.0, entry_price: 2200.0 },
        Position { symbol: "SOL".to_string(), quantity: 50.0, entry_price: 90.0 },
    ];

    monitor_portfolio(portfolio).await;
}
```

## Таймауты для торговых операций

```rust
use tokio::time::{timeout, Duration};

async fn place_order(symbol: &str, side: &str, quantity: f64) -> Result<String, String> {
    // Имитируем размещение ордера
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(format!("ORDER-{}-{}-{}", symbol, side, quantity))
}

async fn place_order_with_timeout(
    symbol: &str,
    side: &str,
    quantity: f64,
    timeout_ms: u64
) -> Result<String, String> {
    let order_future = place_order(symbol, side, quantity);

    match timeout(Duration::from_millis(timeout_ms), order_future).await {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Таймаут при размещении ордера {} {} {}",
            side, quantity, symbol
        )),
    }
}

#[tokio::main]
async fn main() {
    // Успешный ордер
    match place_order_with_timeout("BTC", "BUY", 0.1, 1000).await {
        Ok(order_id) => println!("Ордер размещён: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Ордер с таймаутом
    match place_order_with_timeout("ETH", "SELL", 1.0, 100).await {
        Ok(order_id) => println!("Ордер размещён: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Периодические задачи: обновление цен

```rust
use tokio::time::{interval, Duration};

#[derive(Debug)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn fetch_and_log_price(symbol: &str) -> PriceUpdate {
    let price = match symbol {
        "BTC" => 42000.0 + (rand_simple() * 100.0),
        "ETH" => 2200.0 + (rand_simple() * 10.0),
        _ => 100.0,
    };

    PriceUpdate {
        symbol: symbol.to_string(),
        price,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let mut price_interval = interval(Duration::from_secs(1));
    let mut update_count = 0;

    println!("Начинаю мониторинг цен (5 обновлений)...\n");

    loop {
        price_interval.tick().await;

        let (btc, eth) = tokio::join!(
            fetch_and_log_price("BTC"),
            fetch_and_log_price("ETH")
        );

        println!("[{}] BTC: ${:.2} | ETH: ${:.2}",
            btc.timestamp, btc.price, eth.price);

        update_count += 1;
        if update_count >= 5 {
            break;
        }
    }

    println!("\nМониторинг завершён.");
}
```

## Spawn: выполнение в фоне

```rust
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

#[derive(Debug)]
enum TradeSignal {
    Buy { symbol: String, price: f64 },
    Sell { symbol: String, price: f64 },
}

async fn price_monitor(symbol: String, tx: mpsc::Sender<TradeSignal>) {
    let mut last_price = 42000.0;

    for _ in 0..5 {
        sleep(Duration::from_millis(500)).await;

        // Имитируем изменение цены
        let new_price = last_price + (rand_simple() * 200.0 - 100.0);

        // Генерируем сигнал при значительном изменении
        if new_price > last_price * 1.001 {
            let _ = tx.send(TradeSignal::Buy {
                symbol: symbol.clone(),
                price: new_price,
            }).await;
        } else if new_price < last_price * 0.999 {
            let _ = tx.send(TradeSignal::Sell {
                symbol: symbol.clone(),
                price: new_price,
            }).await;
        }

        last_price = new_price;
    }
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<TradeSignal>(100);

    // Запускаем мониторы в фоне
    let btc_tx = tx.clone();
    let eth_tx = tx.clone();

    tokio::spawn(async move {
        price_monitor("BTC".to_string(), btc_tx).await;
    });

    tokio::spawn(async move {
        price_monitor("ETH".to_string(), eth_tx).await;
    });

    // Закрываем оригинальный sender
    drop(tx);

    println!("Жду торговые сигналы...\n");

    while let Some(signal) = rx.recv().await {
        match signal {
            TradeSignal::Buy { symbol, price } => {
                println!("[BUY]  {} @ ${:.2}", symbol, price);
            }
            TradeSignal::Sell { symbol, price } => {
                println!("[SELL] {} @ ${:.2}", symbol, price);
            }
        }
    }

    println!("\nВсе мониторы завершили работу.");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `async fn` | Объявляет асинхронную функцию, возвращающую Future |
| `.await` | Приостанавливает выполнение до готовности Future |
| `tokio::join!` | Выполняет несколько futures параллельно |
| `tokio::select!` | Возвращает первый готовый результат |
| `tokio::spawn` | Запускает задачу в фоне |
| `timeout` | Ограничивает время ожидания |
| `interval` | Создаёт периодический таймер |

## Домашнее задание

1. **Арбитражный сканер**: Создай async функцию, которая получает цены BTC с 5 бирж параллельно и находит возможности для арбитража (разница > 0.5%).

2. **Ордер с повторами**: Реализуй функцию `place_order_with_retry`, которая:
   - Пытается разместить ордер
   - При ошибке повторяет до 3 раз с экспоненциальной задержкой
   - Использует timeout для каждой попытки

3. **Потоковый монитор позиций**: Напиши программу, которая:
   - Каждую секунду получает цены для портфеля из 10 активов
   - Рассчитывает общий PnL
   - Отправляет уведомление при изменении PnL > 5%

4. **Конкурентный движок исполнения**: Создай структуру `OrderExecutor` с методом:
   ```rust
   async fn execute_batch(&self, orders: Vec<Order>) -> Vec<Result<OrderResult, Error>>
   ```
   Который исполняет до 5 ордеров параллельно (используй Semaphore).

## Навигация

[← Предыдущий день](../182-tokio-runtime/ru.md) | [Следующий день →](../184-futures-and-streams/ru.md)
