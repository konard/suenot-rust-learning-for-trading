# День 187: tokio::spawn: асинхронные задачи

## Аналогия из трейдинга

Представь себе торгового робота, который одновременно:
- Следит за ценами BTC на Binance
- Следит за ценами BTC на Kraken
- Проверяет балансы на обеих биржах
- Ищет арбитражные возможности

Если выполнять эти задачи последовательно, мы потеряем драгоценное время — арбитражная возможность исчезнет, пока мы ждём ответа от первой биржи. Нам нужно запускать все эти задачи **параллельно**.

`tokio::spawn` — это как найм нескольких ассистентов-трейдеров: каждый занимается своей задачей, а ты (главный поток) координируешь их работу и принимаешь решения на основе полученных данных.

## Что такое tokio::spawn?

`tokio::spawn` создаёт **асинхронную задачу** (task), которая выполняется параллельно с другими задачами в рантайме Tokio. Это не OS-поток — это легковесная задача, которых можно создать тысячи без существенных накладных расходов.

```rust
use tokio::spawn;

#[tokio::main]
async fn main() {
    // Запускаем задачу в фоне
    let handle = spawn(async {
        // Асинхронный код выполняется параллельно
        println!("Задача выполняется!");
        42
    });

    // Ждём завершения и получаем результат
    let result = handle.await.unwrap();
    println!("Результат: {}", result);
}
```

## Простой пример: мониторинг нескольких активов

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn fetch_price(symbol: &str) -> PriceUpdate {
    // Имитируем запрос к API биржи
    sleep(Duration::from_millis(100)).await;

    let price = match symbol {
        "BTC" => 42000.0 + rand_price_offset(),
        "ETH" => 2500.0 + rand_price_offset(),
        "SOL" => 100.0 + rand_price_offset(),
        _ => 0.0,
    };

    PriceUpdate {
        symbol: symbol.to_string(),
        price,
        timestamp: current_timestamp(),
    }
}

fn rand_price_offset() -> f64 {
    // Простая имитация колебания цены
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 1000) as f64 / 10.0
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    let symbols = vec!["BTC", "ETH", "SOL"];

    // Запускаем параллельные задачи для каждого актива
    let mut handles = vec![];

    for symbol in symbols {
        let handle = tokio::spawn(async move {
            fetch_price(symbol).await
        });
        handles.push(handle);
    }

    // Собираем результаты
    println!("=== Текущие цены ===");
    for handle in handles {
        match handle.await {
            Ok(update) => {
                println!("{}: ${:.2}", update.symbol, update.price);
            }
            Err(e) => {
                println!("Ошибка получения цены: {:?}", e);
            }
        }
    }
}
```

## JoinHandle: управление задачами

`tokio::spawn` возвращает `JoinHandle<T>` — дескриптор задачи, через который можно:
- Дождаться завершения (`.await`)
- Отменить задачу (`.abort()`)
- Проверить статус завершения

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Запускаем долгую задачу
    let handle = tokio::spawn(async {
        println!("Начинаю мониторинг рынка...");

        for i in 1..=10 {
            sleep(Duration::from_millis(100)).await;
            println!("Цикл мониторинга #{}", i);
        }

        "Мониторинг завершён"
    });

    // Ждём немного и отменяем
    sleep(Duration::from_millis(350)).await;
    println!("Отмена мониторинга!");
    handle.abort();

    // Проверяем результат
    match handle.await {
        Ok(result) => println!("Результат: {}", result),
        Err(e) if e.is_cancelled() => println!("Задача отменена"),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}
```

## Пример: арбитражный сканер

```rust
use tokio::time::{sleep, Duration, timeout};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    symbol: String,
    buy_exchange: String,
    sell_exchange: String,
    buy_price: f64,
    sell_price: f64,
    profit_percent: f64,
}

async fn fetch_exchange_price(exchange: &str, symbol: &str) -> ExchangePrice {
    // Имитируем разную латентность бирж
    let delay = match exchange {
        "Binance" => 50,
        "Kraken" => 80,
        "Coinbase" => 100,
        _ => 150,
    };
    sleep(Duration::from_millis(delay)).await;

    // Имитируем разные цены на биржах
    let base_price = match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    let spread = match exchange {
        "Binance" => 0.001,   // 0.1% спред
        "Kraken" => 0.002,    // 0.2% спред
        "Coinbase" => 0.003,  // 0.3% спред
        _ => 0.005,
    };

    // Добавляем случайное отклонение для каждой биржи
    let exchange_offset = match exchange {
        "Binance" => 0.0,
        "Kraken" => base_price * 0.002,  // Kraken чуть дороже
        "Coinbase" => -base_price * 0.001, // Coinbase чуть дешевле
        _ => 0.0,
    };

    let mid_price = base_price + exchange_offset;

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: mid_price * (1.0 - spread / 2.0),
        ask: mid_price * (1.0 + spread / 2.0),
    }
}

fn find_arbitrage(prices: &[ExchangePrice]) -> Vec<ArbitrageOpportunity> {
    let mut opportunities = vec![];

    // Сравниваем все пары бирж
    for buy_price in prices {
        for sell_price in prices {
            if buy_price.exchange == sell_price.exchange {
                continue;
            }

            // Покупаем по ask на одной бирже, продаём по bid на другой
            let profit = sell_price.bid - buy_price.ask;
            let profit_percent = (profit / buy_price.ask) * 100.0;

            if profit_percent > 0.0 {
                opportunities.push(ArbitrageOpportunity {
                    symbol: buy_price.symbol.clone(),
                    buy_exchange: buy_price.exchange.clone(),
                    sell_exchange: sell_price.exchange.clone(),
                    buy_price: buy_price.ask,
                    sell_price: sell_price.bid,
                    profit_percent,
                });
            }
        }
    }

    opportunities.sort_by(|a, b| b.profit_percent.partial_cmp(&a.profit_percent).unwrap());
    opportunities
}

#[tokio::main]
async fn main() {
    let exchanges = vec!["Binance", "Kraken", "Coinbase"];
    let symbol = "BTC";

    println!("=== Арбитражный сканер ===\n");
    println!("Запрашиваем цены на {} со всех бирж...\n", symbol);

    // Параллельно запрашиваем цены со всех бирж
    let mut handles = vec![];

    for exchange in &exchanges {
        let exchange = exchange.to_string();
        let symbol = symbol.to_string();

        let handle = tokio::spawn(async move {
            // Таймаут на запрос
            timeout(
                Duration::from_secs(1),
                fetch_exchange_price(&exchange, &symbol)
            ).await
        });

        handles.push(handle);
    }

    // Собираем результаты
    let mut prices = vec![];

    for handle in handles {
        match handle.await {
            Ok(Ok(price)) => {
                println!("{}: bid=${:.2}, ask=${:.2}",
                    price.exchange, price.bid, price.ask);
                prices.push(price);
            }
            Ok(Err(_)) => println!("Таймаут запроса"),
            Err(e) => println!("Ошибка задачи: {:?}", e),
        }
    }

    // Анализируем арбитражные возможности
    println!("\n=== Арбитражные возможности ===\n");

    let opportunities = find_arbitrage(&prices);

    if opportunities.is_empty() {
        println!("Арбитражных возможностей не найдено");
    } else {
        for opp in opportunities.iter().take(3) {
            println!("Купить на {} по ${:.2}", opp.buy_exchange, opp.buy_price);
            println!("Продать на {} по ${:.2}", opp.sell_exchange, opp.sell_price);
            println!("Прибыль: {:.4}%\n", opp.profit_percent);
        }
    }
}
```

## Обработка паники в задачах

Если задача паникует, паника не распространяется на другие задачи — она изолирована. Но `JoinHandle::await` вернёт ошибку:

```rust
use tokio::time::{sleep, Duration};

async fn risky_price_fetch(symbol: &str) -> f64 {
    if symbol == "INVALID" {
        panic!("Неизвестный символ!");
    }
    42000.0
}

#[tokio::main]
async fn main() {
    let symbols = vec!["BTC", "INVALID", "ETH"];

    let mut handles = vec![];

    for symbol in symbols {
        let handle = tokio::spawn(async move {
            risky_price_fetch(symbol).await
        });
        handles.push((symbol, handle));
    }

    for (symbol, handle) in handles {
        match handle.await {
            Ok(price) => println!("{}: ${:.2}", symbol, price),
            Err(e) if e.is_panic() => {
                println!("{}: ПАНИКА в задаче!", symbol);
            }
            Err(e) => println!("{}: ошибка {:?}", symbol, e),
        }
    }

    println!("\nПрограмма продолжает работу!");
}
```

## Пример: многопоточный торговый бот

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};

#[derive(Debug, Clone)]
enum TradingSignal {
    Buy { symbol: String, price: f64 },
    Sell { symbol: String, price: f64 },
    Hold { symbol: String },
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

// Задача: получение рыночных данных
async fn market_data_task(
    symbol: String,
    tx: mpsc::Sender<MarketData>,
) {
    let mut ticker = interval(Duration::from_millis(500));
    let mut price = match symbol.as_str() {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    for i in 0..10 {
        ticker.tick().await;

        // Симуляция движения цены
        let change = (i as f64 % 3.0 - 1.0) * 10.0;
        price += change;

        let data = MarketData {
            symbol: symbol.clone(),
            price,
            volume: 1000.0 + (i as f64 * 100.0),
            timestamp: current_timestamp(),
        };

        if tx.send(data.clone()).await.is_err() {
            println!("[{}] Канал закрыт", symbol);
            break;
        }

        println!("[{}] Цена: ${:.2}", symbol, data.price);
    }
}

// Задача: анализ и генерация сигналов
async fn signal_generator_task(
    mut rx: mpsc::Receiver<MarketData>,
    signal_tx: mpsc::Sender<TradingSignal>,
) {
    let mut last_prices: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Some(data) = rx.recv().await {
        let signal = if let Some(&last_price) = last_prices.get(&data.symbol) {
            let change_percent = ((data.price - last_price) / last_price) * 100.0;

            if change_percent > 0.02 {
                TradingSignal::Sell {
                    symbol: data.symbol.clone(),
                    price: data.price,
                }
            } else if change_percent < -0.02 {
                TradingSignal::Buy {
                    symbol: data.symbol.clone(),
                    price: data.price,
                }
            } else {
                TradingSignal::Hold {
                    symbol: data.symbol.clone(),
                }
            }
        } else {
            TradingSignal::Hold {
                symbol: data.symbol.clone(),
            }
        };

        last_prices.insert(data.symbol, data.price);

        if signal_tx.send(signal).await.is_err() {
            break;
        }
    }
}

// Задача: исполнение торговых сигналов
async fn execution_task(
    mut signal_rx: mpsc::Receiver<TradingSignal>,
) {
    let mut position: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Some(signal) = signal_rx.recv().await {
        match signal {
            TradingSignal::Buy { symbol, price } => {
                let qty = 0.1;
                *position.entry(symbol.clone()).or_insert(0.0) += qty;
                println!(">>> ПОКУПКА {} по ${:.2} (позиция: {:.2})",
                    symbol, price, position.get(&symbol).unwrap_or(&0.0));
            }
            TradingSignal::Sell { symbol, price } => {
                if let Some(pos) = position.get_mut(&symbol) {
                    if *pos > 0.0 {
                        let qty = (*pos).min(0.1);
                        *pos -= qty;
                        println!(">>> ПРОДАЖА {} по ${:.2} (позиция: {:.2})",
                            symbol, price, pos);
                    }
                }
            }
            TradingSignal::Hold { symbol } => {
                // Ничего не делаем
            }
        }
    }

    println!("\n=== Итоговые позиции ===");
    for (symbol, qty) in &position {
        println!("{}: {:.4}", symbol, qty);
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    println!("=== Торговый бот запущен ===\n");

    // Каналы для связи между задачами
    let (data_tx, data_rx) = mpsc::channel::<MarketData>(100);
    let (signal_tx, signal_rx) = mpsc::channel::<TradingSignal>(100);

    // Запускаем задачи сбора данных
    let btc_tx = data_tx.clone();
    let btc_task = tokio::spawn(async move {
        market_data_task("BTC".to_string(), btc_tx).await;
    });

    let eth_tx = data_tx.clone();
    let eth_task = tokio::spawn(async move {
        market_data_task("ETH".to_string(), eth_tx).await;
    });

    // Закрываем оригинальный отправитель
    drop(data_tx);

    // Запускаем анализатор сигналов
    let signal_task = tokio::spawn(async move {
        signal_generator_task(data_rx, signal_tx).await;
    });

    // Запускаем исполнитель
    let execution_task = tokio::spawn(async move {
        execution_task(signal_rx).await;
    });

    // Ждём завершения всех задач
    let _ = tokio::join!(btc_task, eth_task, signal_task, execution_task);

    println!("\n=== Торговый бот остановлен ===");
}
```

## tokio::spawn vs async block

| Характеристика | `tokio::spawn` | async block |
|----------------|----------------|-------------|
| Выполнение | Параллельно | Последовательно (по умолчанию) |
| Требования к данным | `'static + Send` | Любые ссылки |
| Отмена | Можно через `.abort()` | Только через drop |
| Паника | Изолирована | Распространяется |

```rust
#[tokio::main]
async fn main() {
    let data = vec![1, 2, 3];

    // async block — можно использовать ссылку на data
    let result1 = async {
        data.iter().sum::<i32>()
    }.await;

    // spawn — данные должны быть 'static
    let data_owned = data.clone();
    let handle = tokio::spawn(async move {
        data_owned.iter().sum::<i32>()
    });
    let result2 = handle.await.unwrap();

    println!("Результаты: {} и {}", result1, result2);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::spawn` | Создание параллельной асинхронной задачи |
| `JoinHandle` | Дескриптор для управления задачей |
| `.await` на JoinHandle | Ожидание завершения задачи |
| `.abort()` | Отмена выполняющейся задачи |
| Изоляция паники | Паника в задаче не влияет на другие |
| `'static + Send` | Требования к данным в spawned задаче |

## Домашнее задание

1. **Параллельный монитор цен**: Создай программу, которая параллельно мониторит цены 5 разных криптовалют. Каждая задача должна обновлять цену каждые 500мс. Главный поток должен собирать все обновления и выводить сводную таблицу каждые 2 секунды.

2. **Арбитражный бот с таймаутами**: Расширь арбитражный сканер из примера:
   - Добавь 5 бирж
   - Установи разные таймауты для каждой биржи
   - Если биржа не отвечает — пропускай её и продолжай работу
   - Логируй все таймауты и ошибки

3. **Система риск-менеджмента**: Реализуй систему с тремя параллельными задачами:
   - **Price Monitor**: следит за ценой и отправляет обновления
   - **Risk Calculator**: получает обновления и считает текущий риск позиции
   - **Alert System**: получает уровень риска и отправляет алерты, если риск превышает порог

4. **Graceful Shutdown**: Модифицируй торгового бота:
   - Добавь обработку сигнала отмены (используй `tokio::select!` и `tokio_util::sync::CancellationToken`)
   - При получении сигнала все задачи должны корректно завершить работу
   - Сохрани итоговое состояние перед выходом

## Навигация

[← Предыдущий день](../186-tokio-main-entry-point/ru.md) | [Следующий день →](../188-join-select-await/ru.md)
