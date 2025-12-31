# День 188: tokio::select!: первый кто ответит

## Аналогия из трейдинга

Представь, что ты трейдер, который одновременно отправил запросы на получение цен с нескольких бирж: Binance, Bybit и OKX. Тебе не нужны все три ответа — достаточно получить первую актуальную цену, чтобы принять торговое решение. Как только одна биржа ответила — ты действуешь, не дожидаясь остальных.

Именно так работает `tokio::select!` — он запускает несколько асинхронных операций параллельно и возвращает результат **первой завершившейся**. Остальные операции отменяются.

Это критически важно в алготрейдинге:
- Получить первую цену из нескольких источников
- Исполнить ордер с таймаутом — или отменить
- Реагировать на первое из нескольких событий рынка
- Управлять несколькими WebSocket-соединениями одновременно

## Что такое tokio::select!?

`tokio::select!` — это макрос, который позволяет ожидать несколько async-операций одновременно и реагировать на первую завершившуюся.

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = sleep(Duration::from_secs(1)) => {
            println!("Прошла 1 секунда");
        }
        _ = sleep(Duration::from_secs(2)) => {
            println!("Прошли 2 секунды");
        }
    }
    // Выведет: "Прошла 1 секунда"
    // Второй sleep будет отменён
}
```

## Базовый синтаксис

```rust
tokio::select! {
    результат1 = async_операция1 => {
        // Обработка результата1
    }
    результат2 = async_операция2 => {
        // Обработка результата2
    }
    // Можно добавить больше веток
}
```

## Получение цен с нескольких бирж

Рассмотрим практический пример — запрос цен с разных бирж:

```rust
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    price: f64,
    latency_ms: u128,
}

// Симуляция запроса к бирже
async fn fetch_price(exchange: &str, symbol: &str, delay_ms: u64) -> PriceQuote {
    let start = Instant::now();

    // Симулируем сетевую задержку
    sleep(Duration::from_millis(delay_ms)).await;

    // Симулируем разные цены на разных биржах
    let price = match exchange {
        "Binance" => 42150.50,
        "Bybit" => 42148.00,
        "OKX" => 42152.25,
        _ => 42150.00,
    };

    PriceQuote {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        price,
        latency_ms: start.elapsed().as_millis(),
    }
}

#[tokio::main]
async fn main() {
    println!("Запрашиваем цену BTC с нескольких бирж...\n");

    let start = Instant::now();

    // select! вернёт результат первой биржи, которая ответит
    let fastest_quote = tokio::select! {
        quote = fetch_price("Binance", "BTC/USDT", 150) => quote,
        quote = fetch_price("Bybit", "BTC/USDT", 100) => quote,   // Самая быстрая
        quote = fetch_price("OKX", "BTC/USDT", 200) => quote,
    };

    println!("Первый ответ от: {}", fastest_quote.exchange);
    println!("Цена: ${:.2}", fastest_quote.price);
    println!("Задержка: {}ms", fastest_quote.latency_ms);
    println!("Общее время: {}ms", start.elapsed().as_millis());
}
```

## Исполнение ордера с таймаутом

Один из самых важных паттернов — таймауты:

```rust
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderResult {
    order_id: u64,
    status: String,
    filled_price: f64,
    filled_quantity: f64,
}

// Симуляция отправки ордера на биржу
async fn submit_order(order: &Order) -> OrderResult {
    // Симулируем задержку исполнения (может быть долгой при низкой ликвидности)
    sleep(Duration::from_millis(500)).await;

    OrderResult {
        order_id: order.id,
        status: "FILLED".to_string(),
        filled_price: order.price,
        filled_quantity: order.quantity,
    }
}

// Отмена ордера
async fn cancel_order(order_id: u64) -> bool {
    println!("Отменяем ордер #{}", order_id);
    sleep(Duration::from_millis(50)).await;
    true
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: 12345,
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.1,
    };

    println!("Отправляем ордер: {:?}\n", order);

    // Вариант 1: использование select! с sleep
    let result = tokio::select! {
        result = submit_order(&order) => {
            println!("Ордер исполнен!");
            Some(result)
        }
        _ = sleep(Duration::from_millis(300)) => {
            println!("Таймаут! Ордер не исполнен за 300ms");
            cancel_order(order.id).await;
            None
        }
    };

    match result {
        Some(r) => println!("Результат: {:?}", r),
        None => println!("Ордер был отменён"),
    }

    // Вариант 2: использование tokio::time::timeout (более идиоматично)
    println!("\n--- Альтернатива с timeout ---\n");

    let order2 = Order {
        id: 12346,
        symbol: "ETH/USDT".to_string(),
        side: "SELL".to_string(),
        price: 2500.0,
        quantity: 1.0,
    };

    match timeout(Duration::from_millis(300), submit_order(&order2)).await {
        Ok(result) => println!("Успех: {:?}", result),
        Err(_) => {
            println!("Таймаут!");
            cancel_order(order2.id).await;
        }
    }
}
```

## Обработка нескольких источников данных

В реальном трейдинге нужно одновременно слушать:
- Обновления цен
- Исполнения ордеров
- Сигналы стратегии
- Команды управления

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
enum MarketEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    Signal { action: String, symbol: String },
}

#[derive(Debug)]
enum Command {
    Pause,
    Resume,
    Shutdown,
}

async fn price_feed(tx: mpsc::Sender<MarketEvent>) {
    let mut price = 42000.0;
    loop {
        sleep(Duration::from_millis(100)).await;
        price += (rand_simple() - 0.5) * 10.0;

        let event = MarketEvent::PriceUpdate {
            symbol: "BTC/USDT".to_string(),
            price,
        };

        if tx.send(event).await.is_err() {
            break;
        }
    }
}

async fn order_updates(tx: mpsc::Sender<MarketEvent>) {
    let mut order_id = 1000;
    loop {
        sleep(Duration::from_millis(500)).await;
        order_id += 1;

        let event = MarketEvent::OrderFilled {
            order_id,
            price: 42000.0 + rand_simple() * 100.0,
        };

        if tx.send(event).await.is_err() {
            break;
        }
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

#[tokio::main]
async fn main() {
    let (market_tx, mut market_rx) = mpsc::channel::<MarketEvent>(100);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(10);

    // Запускаем источники данных
    let market_tx_clone = market_tx.clone();
    tokio::spawn(async move {
        price_feed(market_tx_clone).await;
    });

    tokio::spawn(async move {
        order_updates(market_tx).await;
    });

    // Симулируем команду shutdown через 1 секунду
    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        let _ = cmd_tx.send(Command::Shutdown).await;
    });

    println!("Запуск торгового движка...\n");

    let mut running = true;
    let mut event_count = 0;

    while running {
        tokio::select! {
            // Обработка рыночных событий
            Some(event) = market_rx.recv() => {
                event_count += 1;
                match event {
                    MarketEvent::PriceUpdate { symbol, price } => {
                        println!("[ЦЕНА] {}: ${:.2}", symbol, price);
                    }
                    MarketEvent::OrderFilled { order_id, price } => {
                        println!("[ОРДЕР] #{} исполнен по ${:.2}", order_id, price);
                    }
                    MarketEvent::Signal { action, symbol } => {
                        println!("[СИГНАЛ] {} {}", action, symbol);
                    }
                }
            }

            // Обработка команд управления
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    Command::Pause => println!("\n⏸ Пауза"),
                    Command::Resume => println!("\n▶ Возобновление"),
                    Command::Shutdown => {
                        println!("\n🛑 Завершение работы...");
                        running = false;
                    }
                }
            }

            // Если все каналы закрыты
            else => {
                println!("Все источники данных закрыты");
                running = false;
            }
        }
    }

    println!("\nОбработано событий: {}", event_count);
}
```

## Гонка между стратегиями

Запуск нескольких стратегий, где побеждает первая с сигналом:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct TradeSignal {
    strategy: String,
    action: String,  // "BUY" или "SELL"
    symbol: String,
    confidence: f64,
}

async fn momentum_strategy(symbol: &str) -> Option<TradeSignal> {
    // Симуляция анализа моментума
    sleep(Duration::from_millis(150)).await;

    // Предположим, стратегия нашла сигнал
    Some(TradeSignal {
        strategy: "Momentum".to_string(),
        action: "BUY".to_string(),
        symbol: symbol.to_string(),
        confidence: 0.75,
    })
}

async fn mean_reversion_strategy(symbol: &str) -> Option<TradeSignal> {
    // Симуляция анализа mean reversion
    sleep(Duration::from_millis(200)).await;

    Some(TradeSignal {
        strategy: "Mean Reversion".to_string(),
        action: "SELL".to_string(),
        symbol: symbol.to_string(),
        confidence: 0.65,
    })
}

async fn breakout_strategy(symbol: &str) -> Option<TradeSignal> {
    // Симуляция анализа пробоя
    sleep(Duration::from_millis(100)).await;

    // Эта стратегия не нашла сигнал
    None
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";

    println!("Запуск стратегий для {}...\n", symbol);

    // Ждём первый валидный сигнал
    let signal = tokio::select! {
        result = momentum_strategy(symbol) => {
            println!("Momentum завершился первым");
            result
        }
        result = mean_reversion_strategy(symbol) => {
            println!("Mean Reversion завершился первым");
            result
        }
        result = breakout_strategy(symbol) => {
            println!("Breakout завершился первым");
            result
        }
    };

    match signal {
        Some(s) => {
            println!("\nПолучен сигнал:");
            println!("  Стратегия: {}", s.strategy);
            println!("  Действие: {}", s.action);
            println!("  Уверенность: {:.0}%", s.confidence * 100.0);
        }
        None => {
            println!("\nСигнал не найден");
        }
    }
}
```

## Biased select — приоритет веток

По умолчанию `select!` выбирает ветки случайно, если несколько готовы одновременно. Для приоритизации используется `biased`:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Priority {
    High,   // Риск-менеджмент, стоп-лоссы
    Medium, // Исполнение ордеров
    Low,    // Логирование, аналитика
}

#[derive(Debug)]
struct Task {
    priority: Priority,
    description: String,
}

#[tokio::main]
async fn main() {
    let (high_tx, mut high_rx) = mpsc::channel::<Task>(10);
    let (medium_tx, mut medium_rx) = mpsc::channel::<Task>(10);
    let (low_tx, mut low_rx) = mpsc::channel::<Task>(10);

    // Отправляем задачи
    tokio::spawn(async move {
        high_tx.send(Task {
            priority: Priority::High,
            description: "СТОП-ЛОСС сработал!".to_string(),
        }).await.unwrap();
    });

    tokio::spawn(async move {
        medium_tx.send(Task {
            priority: Priority::Medium,
            description: "Исполнить ордер #123".to_string(),
        }).await.unwrap();
    });

    tokio::spawn(async move {
        low_tx.send(Task {
            priority: Priority::Low,
            description: "Записать статистику".to_string(),
        }).await.unwrap();
    });

    // Даём время на отправку
    sleep(Duration::from_millis(10)).await;

    // biased гарантирует проверку веток в порядке объявления
    for _ in 0..3 {
        tokio::select! {
            biased;  // Проверяем ветки по порядку!

            Some(task) = high_rx.recv() => {
                println!("🔴 ВЫСОКИЙ: {}", task.description);
            }
            Some(task) = medium_rx.recv() => {
                println!("🟡 СРЕДНИЙ: {}", task.description);
            }
            Some(task) = low_rx.recv() => {
                println!("🟢 НИЗКИЙ: {}", task.description);
            }
            else => break,
        }
    }
}
```

## Обработка ошибок в select!

```rust
use tokio::time::{sleep, Duration};
use std::io;

async fn fetch_from_primary() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(100)).await;
    // Симулируем ошибку первичного источника
    Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Primary down"))
}

async fn fetch_from_backup() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(150)).await;
    Ok(42150.50)
}

async fn fetch_from_cache() -> Result<f64, io::Error> {
    sleep(Duration::from_millis(10)).await;
    Ok(42000.0)  // Устаревшая, но доступная цена
}

#[tokio::main]
async fn main() {
    println!("Попытка получить цену BTC...\n");

    // Получаем первый успешный результат
    let price = tokio::select! {
        result = fetch_from_primary() => {
            match result {
                Ok(p) => {
                    println!("✓ Получено с primary");
                    Some(p)
                }
                Err(e) => {
                    println!("✗ Primary error: {}", e);
                    None
                }
            }
        }
        result = fetch_from_backup() => {
            match result {
                Ok(p) => {
                    println!("✓ Получено с backup");
                    Some(p)
                }
                Err(e) => {
                    println!("✗ Backup error: {}", e);
                    None
                }
            }
        }
    };

    // Если оба основных источника не сработали — используем кэш
    let final_price = match price {
        Some(p) => p,
        None => {
            println!("\nИспользуем кэш...");
            fetch_from_cache().await.unwrap_or(0.0)
        }
    };

    println!("\nИтоговая цена: ${:.2}", final_price);
}
```

## select! в цикле — Event Loop

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[tokio::main]
async fn main() {
    let (price_tx, mut price_rx) = mpsc::channel::<f64>(100);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // Симуляция потока цен
    tokio::spawn(async move {
        let mut price = 42000.0;
        let mut interval = interval(Duration::from_millis(100));

        loop {
            interval.tick().await;
            price += (rand_simple() - 0.5) * 20.0;
            if price_tx.send(price).await.is_err() {
                break;
            }
        }
    });

    // Shutdown через 2 секунды
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let _ = shutdown_tx.send(()).await;
    });

    let mut position = Position {
        symbol: "BTC/USDT".to_string(),
        quantity: 0.5,
        entry_price: 42000.0,
        current_price: 42000.0,
    };

    // Интервал для периодических проверок
    let mut check_interval = interval(Duration::from_millis(500));
    let start = Instant::now();

    println!("Мониторинг позиции: {} @ ${:.2}\n",
             position.symbol, position.entry_price);

    loop {
        tokio::select! {
            // Обновление цены
            Some(price) = price_rx.recv() => {
                position.current_price = price;
            }

            // Периодическая проверка позиции
            _ = check_interval.tick() => {
                let elapsed = start.elapsed().as_secs_f64();
                println!("[{:.1}s] Цена: ${:.2} | PnL: ${:.2} ({:+.2}%)",
                    elapsed,
                    position.current_price,
                    position.pnl(),
                    position.pnl_percent()
                );

                // Проверка стоп-лосса
                if position.pnl_percent() < -2.0 {
                    println!("⚠️  СТОП-ЛОСС! Закрываем позицию");
                    break;
                }

                // Проверка тейк-профита
                if position.pnl_percent() > 2.0 {
                    println!("🎯 ТЕЙК-ПРОФИТ! Закрываем позицию");
                    break;
                }
            }

            // Команда на завершение
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Получен сигнал завершения");
                break;
            }
        }
    }

    println!("\nИтог:");
    println!("  Финальная цена: ${:.2}", position.current_price);
    println!("  PnL: ${:.2} ({:+.2}%)", position.pnl(), position.pnl_percent());
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::select!` | Ожидание первого из нескольких async-событий |
| Cancellation | Остальные ветки отменяются после завершения одной |
| `biased` | Принудительный порядок проверки веток |
| Timeout pattern | Комбинация select! с sleep для таймаутов |
| Event loop | select! в цикле для постоянной обработки событий |
| Multiple channels | Одновременное прослушивание нескольких каналов |

## Домашнее задание

1. **Агрегатор цен**: Реализуй функцию `get_best_price()`, которая запрашивает цену с 5 бирж параллельно и возвращает первую полученную цену. Добавь таймаут в 500ms — если никто не ответил, верни ошибку.

2. **Приоритетный обработчик ордеров**: Создай систему с тремя очередями ордеров (market, limit, stop). Используй `biased` select, чтобы market-ордера обрабатывались в первую очередь.

3. **Торговый бот с graceful shutdown**: Напиши бота, который:
   - Слушает поток цен
   - Обрабатывает сигналы стратегии
   - Корректно завершает работу по Ctrl+C (используй `tokio::signal::ctrl_c()`)
   - Сохраняет состояние перед выходом

4. **Мультибиржевой арбитраж**: Реализуй функцию, которая одновременно:
   - Ждёт цену с биржи A
   - Ждёт цену с биржи B
   - Сравнивает цены и логирует возможность арбитража
   - Используй `tokio::join!` для получения обеих цен, или `select!` для первой

## Навигация

[← Предыдущий день](../187-async-await-basic/ru.md) | [Следующий день →](../189-tokio-spawn-concurrent-tasks/ru.md)
