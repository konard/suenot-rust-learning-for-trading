# День 195: Broadcast Channel: всем подписчикам

## Аналогия из трейдинга

Представь биржевой терминал, который транслирует котировки в реальном времени. Когда цена Bitcoin меняется, эту информацию должны получить **все** подключённые клиенты одновременно: торговые боты, риск-менеджеры, аналитические системы, мобильные приложения трейдеров. Это классический паттерн **broadcast** — один отправитель, много получателей.

В Tokio для этого существует `broadcast` канал — специальный канал для трансляции сообщений всем подписчикам. В отличие от обычного `mpsc` канала, где каждое сообщение получает только один получатель, в broadcast канале **каждый** подписчик получает **каждое** сообщение.

Реальные примеры использования в трейдинге:
- Рассылка обновлений цен всем торговым стратегиям
- Уведомление о срабатывании стоп-лоссов
- Трансляция новостей и событий рынка
- Синхронизация состояния между компонентами системы

## Что такое Broadcast Channel?

`tokio::sync::broadcast` — это многопроизводительный, многопотребительский канал, где **каждое** отправленное сообщение доставляется **всем** активным получателям.

Ключевые особенности:
1. **Клонирование сообщений** — каждый подписчик получает свою копию сообщения
2. **Ограниченная ёмкость** — канал имеет фиксированный размер буфера
3. **Отставание (lag)** — медленные получатели могут пропускать сообщения
4. **Динамическая подписка** — новые подписчики могут присоединяться в любой момент

```rust
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    // Создаём broadcast канал с буфером на 16 сообщений
    let (tx, mut rx1) = broadcast::channel::<f64>(16);

    // Создаём второго подписчика
    let mut rx2 = tx.subscribe();

    // Отправляем цену — её получат ОБА подписчика
    tx.send(42000.0).unwrap();

    // Каждый получатель получает своё сообщение
    println!("Подписчик 1: {}", rx1.recv().await.unwrap());
    println!("Подписчик 2: {}", rx2.recv().await.unwrap());
}
```

## Простой пример: Трансляция котировок

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    // Канал для рассылки обновлений цен
    let (tx, _) = broadcast::channel::<PriceUpdate>(100);

    // Создаём подписчиков для разных компонентов
    let mut trading_bot_rx = tx.subscribe();
    let mut risk_manager_rx = tx.subscribe();
    let mut logger_rx = tx.subscribe();

    // Торговый бот
    let trading_bot = tokio::spawn(async move {
        while let Ok(update) = trading_bot_rx.recv().await {
            println!("[Бот] Получена цена {}: ${:.2}",
                update.symbol, update.price);
            // Здесь логика принятия торговых решений
        }
    });

    // Риск-менеджер
    let risk_manager = tokio::spawn(async move {
        while let Ok(update) = risk_manager_rx.recv().await {
            println!("[Риск] Проверка позиции по {}: ${:.2}",
                update.symbol, update.price);
            // Здесь проверка лимитов и рисков
        }
    });

    // Логгер
    let logger = tokio::spawn(async move {
        while let Ok(update) = logger_rx.recv().await {
            println!("[Лог] {} @ {} = ${:.2}",
                update.symbol, update.timestamp, update.price);
        }
    });

    // Имитация потока котировок
    for i in 0..5 {
        let update = PriceUpdate {
            symbol: "BTC".to_string(),
            price: 42000.0 + (i as f64 * 100.0),
            timestamp: i,
        };

        tx.send(update).unwrap();
        sleep(Duration::from_millis(100)).await;
    }

    // Закрываем канал (drop отправителя)
    drop(tx);

    // Ждём завершения всех задач
    let _ = tokio::join!(trading_bot, risk_manager, logger);
}
```

## Обработка отставания (Lagged)

Если подписчик обрабатывает сообщения медленнее, чем они поступают, он может пропустить часть сообщений. Это важно учитывать в торговых системах:

```rust
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
}

#[tokio::main]
async fn main() {
    // Маленький буфер для демонстрации отставания
    let (tx, mut rx) = broadcast::channel::<MarketData>(4);

    // Медленный получатель
    let slow_consumer = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(data) => {
                    println!("Получено: {} bid={:.2} ask={:.2}",
                        data.symbol, data.bid, data.ask);
                    // Медленная обработка
                    sleep(Duration::from_millis(200)).await;
                }
                Err(RecvError::Lagged(skipped)) => {
                    // Пропустили сообщения — важно залогировать!
                    println!("ВНИМАНИЕ: Пропущено {} сообщений!", skipped);
                    // В реальной системе здесь нужна синхронизация
                }
                Err(RecvError::Closed) => {
                    println!("Канал закрыт");
                    break;
                }
            }
        }
    });

    // Быстрый отправитель
    for i in 0..10 {
        let data = MarketData {
            symbol: "ETH".to_string(),
            bid: 2500.0 + i as f64,
            ask: 2501.0 + i as f64,
        };

        match tx.send(data) {
            Ok(receivers) => println!("Отправлено {} получателям", receivers),
            Err(_) => println!("Нет активных получателей"),
        }

        sleep(Duration::from_millis(50)).await;
    }

    drop(tx);
    let _ = slow_consumer.await;
}
```

## Практический пример: Система торговых сигналов

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum TradingSignal {
    Buy { symbol: String, price: f64, reason: String },
    Sell { symbol: String, price: f64, reason: String },
    StopLoss { symbol: String, trigger_price: f64 },
    TakeProfit { symbol: String, trigger_price: f64 },
    MarketAlert { message: String },
}

struct SignalBroadcaster {
    sender: broadcast::Sender<TradingSignal>,
}

impl SignalBroadcaster {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        SignalBroadcaster { sender }
    }

    fn subscribe(&self) -> broadcast::Receiver<TradingSignal> {
        self.sender.subscribe()
    }

    fn broadcast(&self, signal: TradingSignal) -> Result<usize, String> {
        self.sender.send(signal)
            .map_err(|_| "Нет активных подписчиков".to_string())
    }
}

// Торговый исполнитель
async fn order_executor(
    mut rx: broadcast::Receiver<TradingSignal>,
    name: String,
) {
    println!("[{}] Запущен", name);

    while let Ok(signal) = rx.recv().await {
        match signal {
            TradingSignal::Buy { symbol, price, reason } => {
                println!("[{}] ПОКУПКА {} @ ${:.2} ({})",
                    name, symbol, price, reason);
            }
            TradingSignal::Sell { symbol, price, reason } => {
                println!("[{}] ПРОДАЖА {} @ ${:.2} ({})",
                    name, symbol, price, reason);
            }
            TradingSignal::StopLoss { symbol, trigger_price } => {
                println!("[{}] Установлен СТОП-ЛОСС {} @ ${:.2}",
                    name, symbol, trigger_price);
            }
            TradingSignal::TakeProfit { symbol, trigger_price } => {
                println!("[{}] Установлен ТЕЙК-ПРОФИТ {} @ ${:.2}",
                    name, symbol, trigger_price);
            }
            TradingSignal::MarketAlert { message } => {
                println!("[{}] АЛЕРТ: {}", name, message);
            }
        }
    }

    println!("[{}] Остановлен", name);
}

#[tokio::main]
async fn main() {
    let broadcaster = SignalBroadcaster::new(100);

    // Создаём несколько исполнителей для разных стратегий
    let scalper = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Скальпер".to_string(),
    ));

    let swing_trader = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Свинг-трейдер".to_string(),
    ));

    let risk_monitor = tokio::spawn(order_executor(
        broadcaster.subscribe(),
        "Риск-монитор".to_string(),
    ));

    // Небольшая задержка для запуска получателей
    sleep(Duration::from_millis(50)).await;

    // Отправляем серию сигналов
    let signals = vec![
        TradingSignal::MarketAlert {
            message: "Высокая волатильность на рынке!".to_string()
        },
        TradingSignal::Buy {
            symbol: "BTC".to_string(),
            price: 42000.0,
            reason: "Пробой сопротивления".to_string()
        },
        TradingSignal::StopLoss {
            symbol: "BTC".to_string(),
            trigger_price: 41000.0
        },
        TradingSignal::TakeProfit {
            symbol: "BTC".to_string(),
            trigger_price: 45000.0
        },
        TradingSignal::Sell {
            symbol: "ETH".to_string(),
            price: 2500.0,
            reason: "Достигнут тейк-профит".to_string()
        },
    ];

    for signal in signals {
        match broadcaster.broadcast(signal) {
            Ok(count) => println!("--- Сигнал отправлен {} подписчикам ---", count),
            Err(e) => println!("Ошибка: {}", e),
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Закрываем канал
    drop(broadcaster);

    // Ждём завершения всех задач
    let _ = tokio::join!(scalper, swing_trader, risk_monitor);
}
```

## Broadcast vs MPSC: когда что использовать

| Характеристика | broadcast | mpsc |
|----------------|-----------|------|
| Получатели | Много (каждый получает всё) | Много (каждый получает часть) |
| Копирование | Сообщение клонируется | Сообщение передаётся |
| Пропуск сообщений | Возможен (Lagged) | Невозможен |
| Применение | Котировки, события, уведомления | Очередь задач, конвейер |

## Продвинутый пример: Мультивалютный терминал

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, interval};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Quote {
    symbol: String,
    bid: f64,
    ask: f64,
    volume: f64,
    timestamp: u64,
}

struct QuoteFeed {
    sender: broadcast::Sender<Quote>,
}

impl QuoteFeed {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        QuoteFeed { sender }
    }

    fn subscribe(&self) -> broadcast::Receiver<Quote> {
        self.sender.subscribe()
    }

    fn publish(&self, quote: Quote) {
        // Игнорируем ошибку, если нет подписчиков
        let _ = self.sender.send(quote);
    }
}

// Генератор котировок (имитация биржи)
async fn quote_generator(feed: Arc<QuoteFeed>) {
    let symbols = vec![
        ("BTC", 42000.0),
        ("ETH", 2500.0),
        ("SOL", 100.0),
    ];

    let mut tick = 0u64;
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        for (symbol, base_price) in &symbols {
            // Имитация случайного движения цены
            let variation = (tick as f64 * 0.1).sin() * base_price * 0.01;
            let bid = base_price + variation;
            let ask = bid + base_price * 0.001; // спред 0.1%

            let quote = Quote {
                symbol: symbol.to_string(),
                bid,
                ask,
                volume: 100.0 + (tick as f64 % 50.0),
                timestamp: tick,
            };

            feed.publish(quote);
        }

        tick += 1;

        if tick >= 20 {
            break;
        }
    }
}

// Монитор спреда
async fn spread_monitor(mut rx: broadcast::Receiver<Quote>) {
    let mut max_spreads: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Ok(quote) = rx.recv().await {
        let spread_pct = (quote.ask - quote.bid) / quote.bid * 100.0;

        let max_spread = max_spreads
            .entry(quote.symbol.clone())
            .or_insert(0.0);

        if spread_pct > *max_spread {
            *max_spread = spread_pct;
            println!("[Спред] {} макс. спред: {:.4}%",
                quote.symbol, spread_pct);
        }
    }

    println!("\n=== Итоговые максимальные спреды ===");
    for (symbol, spread) in max_spreads {
        println!("{}: {:.4}%", symbol, spread);
    }
}

// Детектор объёма
async fn volume_detector(mut rx: broadcast::Receiver<Quote>) {
    while let Ok(quote) = rx.recv().await {
        if quote.volume > 120.0 {
            println!("[Объём] {} высокий объём: {:.0}",
                quote.symbol, quote.volume);
        }
    }
}

// Логгер в файл (имитация)
async fn file_logger(mut rx: broadcast::Receiver<Quote>) {
    let mut count = 0;

    while let Ok(quote) = rx.recv().await {
        count += 1;
        // В реальности здесь запись в файл
        if count % 10 == 0 {
            println!("[Логгер] Записано {} котировок", count);
        }
    }

    println!("[Логгер] Всего записано: {} котировок", count);
}

#[tokio::main]
async fn main() {
    let feed = Arc::new(QuoteFeed::new(256));

    // Запускаем подписчиков
    let spread_task = tokio::spawn(spread_monitor(feed.subscribe()));
    let volume_task = tokio::spawn(volume_detector(feed.subscribe()));
    let logger_task = tokio::spawn(file_logger(feed.subscribe()));

    // Запускаем генератор
    quote_generator(feed).await;

    // Ждём завершения подписчиков
    let _ = tokio::join!(spread_task, volume_task, logger_task);
}
```

## Обработка ошибок и паттерны

### Проверка количества подписчиков

```rust
use tokio::sync::broadcast;

fn main() {
    let (tx, rx1) = broadcast::channel::<i32>(16);
    let rx2 = tx.subscribe();

    // Количество активных получателей
    println!("Активных получателей: {}", tx.receiver_count());

    // Удаляем одного
    drop(rx1);
    println!("После drop: {}", tx.receiver_count());

    // Отправка возвращает количество получивших
    let sent_to = tx.send(42).unwrap();
    println!("Отправлено {} получателям", sent_to);

    drop(rx2);

    // Ошибка при отправке, если нет подписчиков
    match tx.send(43) {
        Ok(n) => println!("Отправлено: {}", n),
        Err(_) => println!("Нет активных подписчиков!"),
    }
}
```

### Динамическое подключение подписчиков

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct Event {
    id: u64,
    data: String,
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Event>(100);
    let tx_clone = tx.clone();

    // Отправитель событий
    let sender = tokio::spawn(async move {
        for i in 0..10 {
            let event = Event {
                id: i,
                data: format!("Событие {}", i),
            };

            match tx.send(event) {
                Ok(n) => println!("Событие {} отправлено {} подписчикам", i, n),
                Err(_) => println!("Нет подписчиков для события {}", i),
            }

            sleep(Duration::from_millis(100)).await;
        }
    });

    // Подписчик присоединяется позже
    sleep(Duration::from_millis(350)).await;

    let mut late_subscriber = tx_clone.subscribe();
    println!("--- Поздний подписчик присоединился ---");

    let receiver = tokio::spawn(async move {
        while let Ok(event) = late_subscriber.recv().await {
            println!("Поздний подписчик получил: {:?}", event);
        }
    });

    let _ = sender.await;
    drop(tx_clone);
    let _ = receiver.await;
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `broadcast::channel` | Канал для рассылки сообщений всем подписчикам |
| `tx.subscribe()` | Создание нового подписчика |
| `tx.send()` | Отправка сообщения всем |
| `rx.recv().await` | Асинхронное получение сообщения |
| `RecvError::Lagged` | Подписчик пропустил сообщения |
| `receiver_count()` | Количество активных подписчиков |
| Буфер | Фиксированный размер, важен для производительности |

## Домашнее задание

1. **Система котировок**: Создай систему рассылки котировок с:
   - Генератором цен для 5 криптовалют
   - Подписчиком, который считает среднюю цену за последние 10 тиков
   - Подписчиком, который определяет тренд (рост/падение)
   - Подписчиком, который логирует аномальные скачки цен (>1%)

2. **Торговые алерты**: Реализуй систему алертов:
   - При достижении заданных ценовых уровней
   - При превышении объёма
   - При изменении спреда выше порога
   Все алерты должны получать несколько компонентов (Telegram-бот, email, логгер).

3. **Обработка отставания**: Создай стресс-тест:
   - Быстрый отправитель (1000 сообщений/сек)
   - Медленный получатель с обработкой `Lagged`
   - Подсчёт пропущенных сообщений
   - Механизм восстановления (запрос пропущенных данных)

4. **Мультиканальная подписка**: Реализуй систему с несколькими каналами:
   - Канал котировок BTC
   - Канал котировок ETH
   - Канал торговых сигналов
   Создай подписчика, который слушает все каналы одновременно с помощью `tokio::select!`.

## Навигация

[← Предыдущий день](../194-watch-channel-latest-value/ru.md) | [Следующий день →](../196-oneshot-channel-single-response/ru.md)
