# День 184: Future: обещание результата

## Аналогия из трейдинга

Представь, что ты размещаешь лимитный ордер на покупку Bitcoin по цене $40,000. Ордер отправлен на биржу, но исполнится он только когда цена достигнет твоей целевой отметки. Ты получил **обещание** результата — "квитанцию", которая говорит: "Когда условия будут выполнены, ты получишь свои Bitcoin".

В Rust это называется **Future** — обещание того, что когда-нибудь в будущем мы получим результат. Future — это как лимитный ордер: он представляет работу, которая ещё не завершена, но завершится в будущем.

В реальном трейдинге примеры Future:
- **Лимитный ордер** — ждёт достижения целевой цены
- **Запрос к API биржи** — ждёт ответа от сервера
- **Загрузка исторических данных** — ждёт передачи данных по сети
- **Подключение к WebSocket** — ждёт установки соединения

## Что такое Future?

Future в Rust — это trait (типаж), который представляет асинхронное вычисление. Вместо того чтобы блокировать поток в ожидании результата, Future позволяет программе продолжать другую работу.

```rust
// Упрощённое определение trait Future
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

// Poll — это enum с двумя вариантами
pub enum Poll<T> {
    Ready(T),    // Результат готов
    Pending,     // Ещё работаем, позвони позже
}
```

Это как звонить на биржу и спрашивать "Мой ордер исполнен?":
- `Ready(результат)` — "Да, вот твои Bitcoin!"
- `Pending` — "Ещё нет, перезвони позже"

## Простой пример Future

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Future, который ждёт достижения целевой цены
struct PriceTarget {
    target_price: f64,
    current_price: f64,
    started: Instant,
    timeout: Duration,
}

impl PriceTarget {
    fn new(target_price: f64, current_price: f64) -> Self {
        PriceTarget {
            target_price,
            current_price,
            started: Instant::now(),
            timeout: Duration::from_secs(10),
        }
    }

    fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
    }
}

impl Future for PriceTarget {
    type Output = Result<f64, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Проверяем таймаут
        if self.started.elapsed() > self.timeout {
            return Poll::Ready(Err("Таймаут: цена не достигнута".to_string()));
        }

        // Проверяем, достигнута ли целевая цена
        if self.current_price >= self.target_price {
            Poll::Ready(Ok(self.current_price))
        } else {
            // Ещё не готово — просим разбудить нас позже
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// Использование с async/await (требует runtime как tokio)
async fn wait_for_price() {
    println!("Ожидаем целевую цену...");
    // В реальном коде PriceTarget обновлялся бы из внешнего источника
}
```

## async/await — сахар для Future

Ключевые слова `async` и `await` — это синтаксический сахар, который делает работу с Future удобной:

```rust
use tokio::time::{sleep, Duration};

/// Асинхронная функция — это функция, которая возвращает Future
async fn fetch_bitcoin_price() -> f64 {
    // Имитируем запрос к API биржи
    println!("Запрашиваем цену BTC...");
    sleep(Duration::from_millis(100)).await;

    // Возвращаем "полученную" цену
    42_150.75
}

async fn fetch_ethereum_price() -> f64 {
    println!("Запрашиваем цену ETH...");
    sleep(Duration::from_millis(150)).await;

    2_850.30
}

async fn calculate_portfolio_value(btc_amount: f64, eth_amount: f64) -> f64 {
    // await приостанавливает выполнение до готовности Future
    let btc_price = fetch_bitcoin_price().await;
    let eth_price = fetch_ethereum_price().await;

    let btc_value = btc_amount * btc_price;
    let eth_value = eth_amount * eth_price;

    println!("BTC: {} × ${:.2} = ${:.2}", btc_amount, btc_price, btc_value);
    println!("ETH: {} × ${:.2} = ${:.2}", eth_amount, eth_price, eth_value);

    btc_value + eth_value
}

#[tokio::main]
async fn main() {
    let total = calculate_portfolio_value(0.5, 10.0).await;
    println!("Общая стоимость портфеля: ${:.2}", total);
}
```

## Параллельное выполнение Future

Одно из главных преимуществ Future — возможность выполнять несколько операций параллельно:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(symbol: &str) -> (String, f64) {
    // Имитация разного времени ответа для разных бирж
    let delay = match symbol {
        "BTC" => 100,
        "ETH" => 150,
        "SOL" => 80,
        _ => 200,
    };

    sleep(Duration::from_millis(delay)).await;

    let price = match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_800.0,
        "SOL" => 95.0,
        _ => 0.0,
    };

    (symbol.to_string(), price)
}

#[tokio::main]
async fn main() {
    // ❌ Последовательное выполнение — медленно!
    println!("=== Последовательно ===");
    let start = std::time::Instant::now();

    let btc = fetch_price("BTC").await;
    let eth = fetch_price("ETH").await;
    let sol = fetch_price("SOL").await;

    println!("Время: {:?}", start.elapsed());
    println!("BTC: ${}, ETH: ${}, SOL: ${}", btc.1, eth.1, sol.1);

    // ✅ Параллельное выполнение с join! — быстро!
    println!("\n=== Параллельно с join! ===");
    let start = std::time::Instant::now();

    let (btc, eth, sol) = tokio::join!(
        fetch_price("BTC"),
        fetch_price("ETH"),
        fetch_price("SOL")
    );

    println!("Время: {:?}", start.elapsed());
    println!("BTC: ${}, ETH: ${}, SOL: ${}", btc.1, eth.1, sol.1);
}
```

## Future как тип возврата

Функции с `async` возвращают тип, реализующий `Future`:

```rust
use std::future::Future;

/// Явное указание возвращаемого типа
fn fetch_order_status(order_id: u64) -> impl Future<Output = String> {
    async move {
        // Имитация запроса к API
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if order_id % 2 == 0 {
            "filled".to_string()
        } else {
            "pending".to_string()
        }
    }
}

/// Можно использовать и более короткую форму
async fn fetch_order_status_short(order_id: u64) -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    if order_id % 2 == 0 {
        "filled".to_string()
    } else {
        "pending".to_string()
    }
}

#[tokio::main]
async fn main() {
    let status1 = fetch_order_status(42).await;
    let status2 = fetch_order_status_short(43).await;

    println!("Ордер 42: {}", status1);
    println!("Ордер 43: {}", status2);
}
```

## Обработка ошибок в Future

Future часто возвращают `Result` для обработки ошибок:

```rust
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug)]
enum TradingError {
    Timeout,
    NetworkError(String),
    InsufficientFunds,
    OrderRejected(String),
}

async fn place_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<u64, TradingError> {
    println!("Размещаем ордер: {} {} {} @ ${}", side, quantity, symbol, price);

    // Имитируем сетевой запрос
    sleep(Duration::from_millis(100)).await;

    // Проверяем условия (имитация)
    if quantity > 100.0 {
        return Err(TradingError::InsufficientFunds);
    }

    if price < 0.0 {
        return Err(TradingError::OrderRejected("Некорректная цена".to_string()));
    }

    // Успех — возвращаем ID ордера
    Ok(12345)
}

async fn place_order_with_timeout(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<u64, TradingError> {
    // Оборачиваем в timeout
    match timeout(Duration::from_secs(5), place_order(symbol, side, quantity, price)).await {
        Ok(result) => result,
        Err(_) => Err(TradingError::Timeout),
    }
}

#[tokio::main]
async fn main() {
    // Успешный ордер
    match place_order_with_timeout("BTC", "buy", 0.1, 42000.0).await {
        Ok(order_id) => println!("Ордер размещён: #{}", order_id),
        Err(e) => println!("Ошибка: {:?}", e),
    }

    // Ордер с ошибкой
    match place_order_with_timeout("BTC", "buy", 1000.0, 42000.0).await {
        Ok(order_id) => println!("Ордер размещён: #{}", order_id),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}
```

## Практический пример: Мониторинг нескольких бирж

```rust
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn fetch_from_binance(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(50)).await;
    PriceQuote {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42_000.0,
        ask: 42_010.0,
        timestamp: 1234567890,
    }
}

async fn fetch_from_kraken(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(75)).await;
    PriceQuote {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 41_995.0,
        ask: 42_015.0,
        timestamp: 1234567890,
    }
}

async fn fetch_from_coinbase(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(60)).await;
    PriceQuote {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 42_005.0,
        ask: 42_020.0,
        timestamp: 1234567890,
    }
}

async fn find_best_price(symbol: &str) -> (PriceQuote, PriceQuote) {
    // Запрашиваем все биржи параллельно
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_from_binance(symbol),
        fetch_from_kraken(symbol),
        fetch_from_coinbase(symbol)
    );

    let quotes = vec![binance, kraken, coinbase];

    // Лучшая цена для покупки (минимальный ask)
    let best_buy = quotes.iter()
        .min_by(|a, b| a.ask.partial_cmp(&b.ask).unwrap())
        .unwrap()
        .clone();

    // Лучшая цена для продажи (максимальный bid)
    let best_sell = quotes.iter()
        .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap())
        .unwrap()
        .clone();

    (best_buy, best_sell)
}

async fn check_arbitrage_opportunity(symbol: &str) -> Option<f64> {
    let (best_buy, best_sell) = find_best_price(symbol).await;

    println!("Лучшая покупка: {} @ ${:.2}", best_buy.exchange, best_buy.ask);
    println!("Лучшая продажа: {} @ ${:.2}", best_sell.exchange, best_sell.bid);

    let spread = best_sell.bid - best_buy.ask;
    let spread_percent = (spread / best_buy.ask) * 100.0;

    if spread > 0.0 {
        println!("Арбитражная возможность: ${:.2} ({:.4}%)", spread, spread_percent);
        Some(spread)
    } else {
        println!("Арбитраж отсутствует: спред ${:.2}", spread);
        None
    }
}

#[tokio::main]
async fn main() {
    println!("=== Поиск арбитражных возможностей ===\n");

    let start = std::time::Instant::now();

    check_arbitrage_opportunity("BTC").await;

    println!("\nВремя выполнения: {:?}", start.elapsed());
}
```

## Состояния Future

Future может находиться в разных состояниях:

```
┌─────────────────────────────────────────────────────────────┐
│                        Future                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────┐    poll()     ┌─────────┐                     │
│   │ Создан  │ ─────────────>│ Pending │                     │
│   └─────────┘               └────┬────┘                     │
│                                  │                           │
│                                  │ poll() возвращает Pending │
│                                  v                           │
│                            ┌─────────┐                       │
│                            │ Ожидает │ <──────┐              │
│                            └────┬────┘        │              │
│                                 │             │              │
│                                 │ waker.wake()│              │
│                                 v             │              │
│                            ┌─────────┐        │              │
│                            │  Снова  │────────┘              │
│                            │ poll()  │                       │
│                            └────┬────┘                       │
│                                 │                            │
│                                 │ poll() возвращает Ready    │
│                                 v                            │
│                            ┌─────────┐                       │
│                            │  Ready  │                       │
│                            │(Готов)  │                       │
│                            └─────────┘                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Future | Trait, представляющий асинхронное вычисление |
| Poll::Ready | Результат готов |
| Poll::Pending | Результат ещё не готов, нужно подождать |
| async | Ключевое слово для создания асинхронных функций |
| await | Приостанавливает выполнение до готовности Future |
| tokio::join! | Выполняет несколько Future параллельно |
| timeout | Ограничивает время ожидания Future |

## Практические задания

1. **Параллельный запрос цен**: Напиши функцию, которая запрашивает цены 5 криптовалют параллельно и возвращает их в HashMap.

2. **Таймаут ордера**: Создай функцию размещения ордера с таймаутом. Если ордер не подтверждён за 5 секунд, возвращай ошибку.

3. **Первый ответ**: Используя `tokio::select!`, напиши функцию, которая возвращает цену с первой ответившей биржи.

4. **Retry логика**: Создай обёртку, которая повторяет неудавшийся запрос до 3 раз с экспоненциальной задержкой.

## Домашнее задание

1. **Агрегатор цен**: Создай структуру `PriceAggregator`, которая:
   - Подключается к нескольким "биржам" (имитация)
   - Параллельно запрашивает цены
   - Возвращает лучшие bid/ask
   - Логирует время каждого запроса

2. **Торговый бот с Future**: Реализуй асинхронную функцию `execute_strategy`, которая:
   - Получает текущие цены (Future)
   - Анализирует условия входа
   - Размещает ордер (Future)
   - Ждёт подтверждения (Future с таймаутом)
   - Возвращает результат сделки или ошибку

3. **Мониторинг с отменой**: Создай бесконечный цикл мониторинга цен, который можно остановить через `tokio::select!` при получении сигнала отмены.

4. **Custom Future**: Реализуй свой Future `DelayedOrder`, который:
   - Принимает ордер и задержку
   - Возвращает `Poll::Pending` пока не истечёт задержка
   - Возвращает `Poll::Ready(ордер)` когда пора исполнять

## Навигация

[← Предыдущий день](../183-async-await-syntax/ru.md) | [Следующий день →](../185-tokio-runtime-async-engine/ru.md)
