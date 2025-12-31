# День 185: tokio runtime: движок асинхронности

## Аналогия из трейдинга

Представь торговый зал биржи. Традиционный подход (синхронный) — это когда один трейдер обрабатывает одну заявку за раз: принял заявку → дождался подтверждения от контрагента → записал в журнал → только потом берёт следующую. Если контрагент думает 5 минут — трейдер просто сидит и ждёт.

**tokio runtime** — это как современный электронный торговый зал с умным распределителем задач. Вместо того чтобы ждать, он говорит: "Пока ждём ответа по BTC, давай обработаем заявку на ETH. А пока ETH думает — проверим статус SOL". Один "трейдер" (поток) может жонглировать сотнями заявок, потому что большую часть времени он не работает, а ждёт ответа от сети.

В реальном трейдинге это критически важно:
- Подключение к 10 биржам одновременно
- Получение котировок по 100 торговым парам
- Отправка ордеров без блокировки получения данных
- Мониторинг позиций пока выполняются другие операции

## Что такое tokio runtime?

**tokio** — это асинхронный runtime для Rust. Runtime — это "движок", который:

1. **Планирует задачи** — решает, какую async-функцию выполнять сейчас
2. **Управляет потоками** — создаёт пул потоков для выполнения задач
3. **Обрабатывает I/O** — эффективно ждёт данных от сети, файлов
4. **Предоставляет таймеры** — для задержек и таймаутов

```rust
// Future — это обещание результата в будущем
// tokio runtime — это тот, кто выполняет эти обещания

async fn fetch_price(symbol: &str) -> f64 {
    // Это не блокирует поток — tokio переключится на другую задачу
    tokio::time::sleep(Duration::from_millis(100)).await;
    42000.0
}
```

## Архитектура tokio

```
┌─────────────────────────────────────────────────────────────┐
│                      tokio Runtime                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Reactor   │  │  Scheduler  │  │ Timer Wheel │         │
│  │  (I/O poll) │  │ (task queue)│  │  (delays)   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  Thread Pool                                                 │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐               │
│  │Thread 1│ │Thread 2│ │Thread 3│ │Thread N│               │
│  │ Tasks  │ │ Tasks  │ │ Tasks  │ │ Tasks  │               │
│  └────────┘ └────────┘ └────────┘ └────────┘               │
└─────────────────────────────────────────────────────────────┘
```

## Установка tokio

Добавьте в `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

Доступные features:
- `rt` — базовый runtime
- `rt-multi-thread` — многопоточный runtime
- `time` — таймеры и задержки
- `net` — сетевые операции
- `io-util` — утилиты для I/O
- `sync` — примитивы синхронизации
- `macros` — макросы `#[tokio::main]` и `#[tokio::test]`
- `full` — все features (удобно для разработки)

## Создание runtime вручную

```rust
use tokio::runtime::Runtime;
use std::time::Duration;

fn main() {
    // Создаём runtime вручную
    let rt = Runtime::new().unwrap();

    // Выполняем асинхронный код
    rt.block_on(async {
        println!("Запускаем торгового бота...");

        // Имитируем получение цены
        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Цена BTC: $42,000");
    });

    println!("Runtime завершён");
}
```

## Типы runtime

### Single-threaded runtime

Для простых приложений или когда нужен полный контроль:

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Однопоточный runtime
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Однопоточный режим");
        println!("Идеально для простого бота с одной биржей");

        let price = fetch_price("BTC").await;
        println!("Цена: ${:.2}", price);
    });
}

async fn fetch_price(_symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(100)).await;
    42000.0
}
```

### Multi-threaded runtime

Для высокопроизводительных приложений:

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Многопоточный runtime
    let rt = Builder::new_multi_thread()
        .worker_threads(4)  // 4 рабочих потока
        .thread_name("trading-worker")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Многопоточный режим — 4 потока");
        println!("Идеально для мультибиржевого бота");

        // Параллельное получение цен
        let (btc, eth, sol) = tokio::join!(
            fetch_price("BTC"),
            fetch_price("ETH"),
            fetch_price("SOL")
        );

        println!("BTC: ${:.2}", btc);
        println!("ETH: ${:.2}", eth);
        println!("SOL: ${:.2}", sol);
    });
}

async fn fetch_price(symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(100)).await;
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        "SOL" => 100.0,
        _ => 0.0,
    }
}
```

## Практический пример: Монитор цен

```rust
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration, interval};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct PriceMonitor {
    prices: Arc<RwLock<Vec<PriceData>>>,
}

impl PriceMonitor {
    fn new() -> Self {
        PriceMonitor {
            prices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn fetch_price(&self, symbol: &str) -> f64 {
        // Имитация сетевого запроса
        sleep(Duration::from_millis(50)).await;

        // Возвращаем "случайную" цену
        let base = match symbol {
            "BTC" => 42000.0,
            "ETH" => 2500.0,
            "SOL" => 100.0,
            _ => 50.0,
        };

        // Добавляем небольшую вариацию
        let variation = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() % 100) as f64;

        base + variation
    }

    async fn update_price(&self, symbol: &str) {
        let price = self.fetch_price(symbol).await;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data = PriceData {
            symbol: symbol.to_string(),
            price,
            timestamp,
        };

        let mut prices = self.prices.write().await;

        // Обновляем или добавляем
        if let Some(existing) = prices.iter_mut().find(|p| p.symbol == symbol) {
            *existing = data;
        } else {
            prices.push(data);
        }
    }

    async fn get_all_prices(&self) -> Vec<PriceData> {
        self.prices.read().await.clone()
    }
}

fn main() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let monitor = Arc::new(PriceMonitor::new());

        println!("=== Монитор цен криптовалют ===\n");

        // Запускаем обновление цен для нескольких символов
        let symbols = vec!["BTC", "ETH", "SOL"];

        // Первоначальное получение всех цен параллельно
        let mut handles = vec![];
        for symbol in &symbols {
            let monitor = Arc::clone(&monitor);
            let sym = symbol.to_string();
            handles.push(tokio::spawn(async move {
                monitor.update_price(&sym).await;
            }));
        }

        // Ждём завершения всех задач
        for handle in handles {
            handle.await.unwrap();
        }

        // Показываем цены
        let prices = monitor.get_all_prices().await;
        for price in &prices {
            println!("{}: ${:.2}", price.symbol, price.price);
        }

        println!("\n=== Периодическое обновление (3 цикла) ===\n");

        // Периодическое обновление
        let mut interval = interval(Duration::from_secs(1));

        for cycle in 1..=3 {
            interval.tick().await;

            println!("Цикл {}:", cycle);

            // Обновляем все цены параллельно
            let mut handles = vec![];
            for symbol in &symbols {
                let monitor = Arc::clone(&monitor);
                let sym = symbol.to_string();
                handles.push(tokio::spawn(async move {
                    monitor.update_price(&sym).await;
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }

            // Показываем обновлённые цены
            let prices = monitor.get_all_prices().await;
            for price in &prices {
                println!("  {}: ${:.2}", price.symbol, price.price);
            }
        }

        println!("\nМониторинг завершён");
    });
}
```

## Пример: Торговый движок с runtime

```rust
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
enum OrderResult {
    Filled { order_id: u64, fill_price: f64 },
    Rejected { order_id: u64, reason: String },
}

async fn process_order(order: Order) -> OrderResult {
    // Имитация обработки ордера
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Простая логика: покупка всегда успешна, продажа — 80%
    let success = match order.side {
        OrderSide::Buy => true,
        OrderSide::Sell => order.id % 5 != 0, // каждый 5-й отклоняется
    };

    if success {
        OrderResult::Filled {
            order_id: order.id,
            fill_price: order.price,
        }
    } else {
        OrderResult::Rejected {
            order_id: order.id,
            reason: "Недостаточно ликвидности".to_string(),
        }
    }
}

fn main() {
    // Создаём runtime с настройками для трейдинга
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("order-processor")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("=== Торговый движок запущен ===\n");

        // Канал для отправки ордеров
        let (tx, mut rx) = mpsc::channel::<Order>(100);

        // Задача для обработки ордеров
        let processor = tokio::spawn(async move {
            let mut results = Vec::new();

            while let Some(order) = rx.recv().await {
                println!("Обрабатываю ордер #{}: {:?} {} {:.4} @ ${:.2}",
                    order.id, order.side, order.symbol,
                    order.quantity, order.price);

                let result = process_order(order).await;

                match &result {
                    OrderResult::Filled { order_id, fill_price } => {
                        println!("  ✓ Ордер #{} исполнен по ${:.2}",
                            order_id, fill_price);
                    }
                    OrderResult::Rejected { order_id, reason } => {
                        println!("  ✗ Ордер #{} отклонён: {}",
                            order_id, reason);
                    }
                }

                results.push(result);
            }

            results
        });

        // Отправляем ордера
        let orders = vec![
            Order { id: 1, symbol: "BTC".into(), side: OrderSide::Buy,
                    price: 42000.0, quantity: 0.1 },
            Order { id: 2, symbol: "ETH".into(), side: OrderSide::Buy,
                    price: 2500.0, quantity: 1.0 },
            Order { id: 3, symbol: "BTC".into(), side: OrderSide::Sell,
                    price: 42100.0, quantity: 0.05 },
            Order { id: 4, symbol: "SOL".into(), side: OrderSide::Buy,
                    price: 100.0, quantity: 10.0 },
            Order { id: 5, symbol: "ETH".into(), side: OrderSide::Sell,
                    price: 2550.0, quantity: 0.5 },
        ];

        println!("Отправляю {} ордеров...\n", orders.len());

        for order in orders {
            tx.send(order).await.unwrap();
        }

        // Закрываем канал, чтобы processor завершился
        drop(tx);

        // Ждём завершения обработки
        let results = processor.await.unwrap();

        println!("\n=== Итоги ===");
        let filled = results.iter()
            .filter(|r| matches!(r, OrderResult::Filled { .. }))
            .count();
        let rejected = results.len() - filled;

        println!("Исполнено: {}", filled);
        println!("Отклонено: {}", rejected);
    });

    println!("\nТорговый движок остановлен");
}
```

## Runtime в разных сценариях

### Сценарий 1: Простой бот с одной биржей

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Однопоточный — достаточно для одной биржи
    let rt = Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Простой бот для Binance");

        loop {
            let price = fetch_binance_price("BTCUSDT").await;
            println!("BTC: ${:.2}", price);

            if should_trade(price) {
                execute_trade("BUY", 0.01).await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            break; // Для примера выходим после первой итерации
        }
    });
}

async fn fetch_binance_price(_symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(50)).await;
    42000.0
}

fn should_trade(_price: f64) -> bool {
    false // Логика принятия решений
}

async fn execute_trade(side: &str, amount: f64) {
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Выполнен {} {}", side, amount);
}
```

### Сценарий 2: Мультибиржевой арбитраж

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Многопоточный — для параллельной работы с биржами
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("arbitrage")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Арбитражный бот\n");

        // Параллельно получаем цены со всех бирж
        let (binance, bybit, okx) = tokio::join!(
            fetch_exchange_price("Binance", "BTC"),
            fetch_exchange_price("Bybit", "BTC"),
            fetch_exchange_price("OKX", "BTC")
        );

        println!("Binance: ${:.2}", binance);
        println!("Bybit:   ${:.2}", bybit);
        println!("OKX:     ${:.2}", okx);

        // Ищем арбитраж
        let min_price = binance.min(bybit).min(okx);
        let max_price = binance.max(bybit).max(okx);
        let spread = (max_price - min_price) / min_price * 100.0;

        println!("\nСпред: {:.2}%", spread);

        if spread > 0.1 {
            println!("Найдена арбитражная возможность!");
        } else {
            println!("Арбитраж не выгоден");
        }
    });
}

async fn fetch_exchange_price(exchange: &str, _symbol: &str) -> f64 {
    // Имитируем разное время ответа бирж
    let delay = match exchange {
        "Binance" => 50,
        "Bybit" => 80,
        "OKX" => 60,
        _ => 100,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Разные цены на разных биржах
    match exchange {
        "Binance" => 42000.0,
        "Bybit" => 42010.0,
        "OKX" => 41995.0,
        _ => 42000.0,
    }
}
```

## Конфигурация runtime для production

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn create_production_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        // Количество потоков = количество CPU ядер
        .worker_threads(num_cpus::get())
        // Имя потоков для отладки
        .thread_name("trading-runtime")
        // Размер стека (по умолчанию 2MB)
        .thread_stack_size(3 * 1024 * 1024)
        // Включаем все функции
        .enable_all()
        // Строим runtime
        .build()
        .expect("Не удалось создать tokio runtime")
}

fn main() {
    let rt = create_production_runtime();

    rt.block_on(async {
        println!("Production runtime запущен");
        println!("Потоков: {}", num_cpus::get());

        // Ваш торговый код здесь
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("Работа завершена");
    });
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| tokio runtime | Движок для выполнения асинхронного кода |
| Runtime::new() | Создание runtime с настройками по умолчанию |
| Builder | Тонкая настройка runtime |
| new_current_thread() | Однопоточный runtime |
| new_multi_thread() | Многопоточный runtime |
| block_on() | Выполнение Future в синхронном контексте |
| worker_threads() | Настройка количества потоков |

## Домашнее задание

1. **Базовый runtime**: Создай программу, которая использует ручное создание runtime для получения цен трёх криптовалют параллельно. Измерь время выполнения и сравни с последовательным получением.

2. **Настройка потоков**: Создай два runtime — однопоточный и многопоточный. Запусти в каждом по 10 параллельных задач (имитация получения цен с задержкой 100ms). Сравни время выполнения.

3. **Торговый сервер**: Реализуй структуру `TradingServer` с методами:
   - `new(threads: usize)` — создание с указанием количества потоков
   - `run(&self, handler: F)` — запуск с обработчиком
   - `shutdown(self)` — graceful shutdown

   Сервер должен принимать ордера через канал и обрабатывать их параллельно.

4. **Профилирование**: Добавь в runtime callback'и для мониторинга:
   - Количество активных задач
   - Время работы каждого потока
   - Использование памяти

   Выведи статистику после завершения работы.

## Навигация

[← Предыдущий день](../184-future-promise-result/ru.md) | [Следующий день →](../186-tokio-main-entry-point/ru.md)
