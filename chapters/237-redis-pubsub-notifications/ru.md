# День 237: Redis Pub/Sub — Уведомления в реальном времени

## Аналогия из трейдинга

Представь торговый зал биржи: когда происходит важное событие — резкое изменение цены, крупная сделка или срабатывание стоп-лосса — об этом должны узнать все заинтересованные участники мгновенно. Трейдеры подписываются на определённые инструменты и получают уведомления только о тех событиях, которые их интересуют.

Redis Pub/Sub работает по такому же принципу:
- **Publisher (издатель)** — система, которая публикует события (например, изменение цены BTC)
- **Subscriber (подписчик)** — клиент, который подписан на определённые каналы и получает уведомления
- **Channel (канал)** — именованный поток сообщений (например, `prices:BTC`, `orders:filled`, `alerts:risk`)

В реальной торговле это используется для:
- Мгновенных уведомлений об изменении цен
- Оповещения о выполнении ордеров
- Сигналов от торговых стратегий
- Алертов риск-менеджмента

## Что такое Redis Pub/Sub?

Redis Pub/Sub — это механизм обмена сообщениями, работающий по принципу «издатель-подписчик»:

1. **Подписчики** регистрируются на один или несколько каналов
2. **Издатели** отправляют сообщения в каналы
3. Redis **доставляет** сообщения всем активным подписчикам канала
4. Сообщения **не сохраняются** — если подписчика нет онлайн, он не получит сообщение

```
┌─────────────┐     ┌───────────┐     ┌──────────────┐
│  Publisher  │────▶│   Redis   │────▶│  Subscriber  │
│  (Prices)   │     │  Channel  │     │  (Trader 1)  │
└─────────────┘     │ prices:BTC│     └──────────────┘
                    │           │     ┌──────────────┐
                    │           │────▶│  Subscriber  │
                    │           │     │  (Trader 2)  │
                    └───────────┘     └──────────────┘
```

## Настройка проекта

```toml
# Cargo.toml
[package]
name = "trading-notifications"
version = "0.1.0"
edition = "2021"

[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "aio"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## Базовый пример: Подписка на канал

```rust
use redis::{Client, Commands, PubSubCommands};
use std::thread;
use std::time::Duration;

fn main() -> redis::RedisResult<()> {
    // Создаём двух клиентов: один для публикации, другой для подписки
    let publisher_client = Client::open("redis://127.0.0.1/")?;
    let subscriber_client = Client::open("redis://127.0.0.1/")?;

    // Поток подписчика
    let subscriber_handle = thread::spawn(move || {
        let mut con = subscriber_client.get_connection().unwrap();

        // Подписываемся на канал цен BTC
        con.subscribe(&["prices:BTC"], |msg| {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            println!("[Подписчик] Канал: {}, Сообщение: {}", channel, payload);

            // Возвращаем ControlFlow для управления подпиской
            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Даём подписчику время подключиться
    thread::sleep(Duration::from_millis(100));

    // Поток издателя
    let publisher_handle = thread::spawn(move || {
        let mut con = publisher_client.get_connection().unwrap();

        // Публикуем несколько обновлений цены
        for i in 0..5 {
            let price = 42000.0 + (i as f64 * 100.0);
            let message = format!("BTC: ${:.2}", price);

            let subscribers: i32 = con.publish("prices:BTC", &message).unwrap();
            println!("[Издатель] Отправлено: {} ({} подписчиков)", message, subscribers);

            thread::sleep(Duration::from_millis(500));
        }
    });

    publisher_handle.join().unwrap();
    // Примечание: subscriber_handle будет работать бесконечно

    Ok(())
}
```

## Уведомления о ценах в торговой системе

```rust
use redis::{Client, Commands, PubSubCommands};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: i64,
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeAlert {
    alert_type: String,
    symbol: String,
    message: String,
    severity: String,
    timestamp: i64,
}

fn main() -> redis::RedisResult<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // Подписчик на обновления цен
    let price_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        println!("[Подписчик цен] Подключаюсь к каналам...");

        con.subscribe(&["prices:BTC", "prices:ETH", "prices:SOL"], |msg| {
            let payload: String = msg.get_payload().unwrap();

            if let Ok(update) = serde_json::from_str::<PriceUpdate>(&payload) {
                println!(
                    "[Цена] {} = ${:.2} (объём: {:.4}, источник: {})",
                    update.symbol, update.price, update.volume, update.source
                );

                // Проверяем резкие изменения цены
                if update.price > 50000.0 {
                    println!("  ⚠️  Цена {} выше $50,000!", update.symbol);
                }
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Подписчик на торговые алерты
    let alert_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        println!("[Подписчик алертов] Подключаюсь к каналу alerts...");

        con.subscribe(&["alerts:trading"], |msg| {
            let payload: String = msg.get_payload().unwrap();

            if let Ok(alert) = serde_json::from_str::<TradeAlert>(&payload) {
                let icon = match alert.severity.as_str() {
                    "critical" => "🔴",
                    "warning" => "🟡",
                    "info" => "🔵",
                    _ => "⚪",
                };

                println!(
                    "[Алерт] {} {} [{}]: {}",
                    icon, alert.alert_type, alert.symbol, alert.message
                );
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    // Издатель цен (симуляция маркет-дата фида)
    let publisher = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        let symbols = vec![
            ("BTC", 42000.0, "prices:BTC"),
            ("ETH", 2800.0, "prices:ETH"),
            ("SOL", 98.0, "prices:SOL"),
        ];

        for i in 0..10 {
            for (symbol, base_price, channel) in &symbols {
                let price_change = (i as f64 * 50.0) * if i % 2 == 0 { 1.0 } else { -1.0 };
                let price = base_price + price_change;

                let update = PriceUpdate {
                    symbol: symbol.to_string(),
                    price,
                    volume: 1.5 + (i as f64 * 0.1),
                    timestamp: chrono::Utc::now().timestamp(),
                    source: "Binance".to_string(),
                };

                let json = serde_json::to_string(&update).unwrap();
                let _: i32 = con.publish(*channel, &json).unwrap();

                // Генерируем алерт при определённых условиях
                if price > 42500.0 && *symbol == "BTC" {
                    let alert = TradeAlert {
                        alert_type: "PRICE_SPIKE".to_string(),
                        symbol: symbol.to_string(),
                        message: format!("Цена превысила $42,500 (текущая: ${:.2})", price),
                        severity: "warning".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                    };

                    let alert_json = serde_json::to_string(&alert).unwrap();
                    let _: i32 = con.publish("alerts:trading", &alert_json).unwrap();
                }
            }

            thread::sleep(Duration::from_millis(1000));
        }

        running_clone.store(false, Ordering::SeqCst);
    });

    publisher.join().unwrap();

    Ok(())
}
```

## Асинхронный Pub/Sub с Tokio

```rust
use redis::aio::PubSub;
use redis::{AsyncCommands, Client};
use tokio::sync::mpsc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OrderNotification {
    order_id: u64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
    filled_qty: f64,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RiskAlert {
    portfolio_id: String,
    metric: String,
    current_value: f64,
    threshold: f64,
    message: String,
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1/")?;

    // Канал для передачи уведомлений в основной обработчик
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Задача подписчика на ордера
    let order_subscriber = {
        let client = client.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("orders:filled").await.unwrap();
            pubsub.subscribe("orders:cancelled").await.unwrap();
            pubsub.subscribe("orders:rejected").await.unwrap();

            println!("[Ордера] Подписан на каналы ордеров");

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let channel: String = msg.get_channel_name().to_string();
                let payload: String = msg.get_payload().unwrap();

                if let Ok(order) = serde_json::from_str::<OrderNotification>(&payload) {
                    let notification = format!(
                        "[{}] Ордер #{}: {} {} {} @ ${:.2} (исполнено: {:.4})",
                        order.status.to_uppercase(),
                        order.order_id,
                        order.side.to_uppercase(),
                        order.quantity,
                        order.symbol,
                        order.price,
                        order.filled_qty
                    );
                    tx.send(notification).await.unwrap();
                }
            }
        })
    };

    // Задача подписчика на риск-алерты
    let risk_subscriber = {
        let client = client.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("risk:alerts").await.unwrap();

            println!("[Риск] Подписан на канал риск-алертов");

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap();

                if let Ok(alert) = serde_json::from_str::<RiskAlert>(&payload) {
                    let notification = format!(
                        "[РИСК] Портфель {}: {} = {:.2}% (порог: {:.2}%) - {}",
                        alert.portfolio_id,
                        alert.metric,
                        alert.current_value,
                        alert.threshold,
                        alert.message
                    );
                    tx.send(notification).await.unwrap();
                }
            }
        })
    };

    // Издатель событий (симуляция торгового движка)
    let publisher = {
        let client = client.clone();

        tokio::spawn(async move {
            let mut con = client.get_multiplexed_async_connection().await.unwrap();

            // Симулируем исполнение ордеров
            for i in 1..=5 {
                let order = OrderNotification {
                    order_id: 1000 + i,
                    symbol: "BTC/USDT".to_string(),
                    side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                    quantity: 0.1 * i as f64,
                    price: 42000.0 + (i as f64 * 100.0),
                    status: "filled".to_string(),
                    filled_qty: 0.1 * i as f64,
                    timestamp: chrono::Utc::now().timestamp(),
                };

                let json = serde_json::to_string(&order).unwrap();
                let _: i32 = con.publish("orders:filled", &json).await.unwrap();

                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // Симулируем риск-алерт
            let risk_alert = RiskAlert {
                portfolio_id: "PORT-001".to_string(),
                metric: "drawdown".to_string(),
                current_value: 12.5,
                threshold: 10.0,
                message: "Просадка превысила допустимый уровень!".to_string(),
            };

            let json = serde_json::to_string(&risk_alert).unwrap();
            let _: i32 = con.publish("risk:alerts", &json).await.unwrap();
        })
    };

    // Обработчик уведомлений
    let handler = tokio::spawn(async move {
        while let Some(notification) = rx.recv().await {
            println!("{}", notification);
        }
    });

    // Ждём завершения издателя
    publisher.await.unwrap();

    // Даём время на получение всех сообщений
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
```

## Паттерн подписки с маской (Pattern Subscribe)

```rust
use redis::{Client, PubSubCommands};
use std::thread;
use std::time::Duration;

fn main() -> redis::RedisResult<()> {
    let subscriber_client = Client::open("redis://127.0.0.1/")?;
    let publisher_client = Client::open("redis://127.0.0.1/")?;

    // Подписчик с паттерном — получает все сообщения о ценах
    let pattern_subscriber = thread::spawn(move || {
        let mut con = subscriber_client.get_connection().unwrap();

        println!("[Паттерн] Подписываюсь на prices:* ...");

        // psubscribe позволяет использовать маски
        con.psubscribe(&["prices:*", "orders:*"], |msg| {
            let pattern: String = msg.get_pattern().unwrap_or_default().to_string();
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            println!(
                "[Паттерн: {}] Канал: {} -> {}",
                pattern, channel, payload
            );

            redis::ControlFlow::Continue
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(100));

    // Издатель отправляет в разные каналы
    let publisher = thread::spawn(move || {
        let mut con = publisher_client.get_connection().unwrap();

        let channels = vec![
            "prices:BTC",
            "prices:ETH",
            "prices:SOL",
            "orders:filled",
            "orders:cancelled",
        ];

        for (i, channel) in channels.iter().enumerate() {
            let message = format!("Сообщение #{} для {}", i + 1, channel);
            let _: i32 = con.publish(*channel, &message).unwrap();
            println!("[Издатель] {} -> {}", channel, message);
            thread::sleep(Duration::from_millis(200));
        }
    });

    publisher.join().unwrap();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
```

## Система уведомлений торговых стратегий

```rust
use redis::{Client, Commands, PubSubCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StrategySignal {
    strategy_id: String,
    signal_type: String, // "entry", "exit", "adjust"
    symbol: String,
    direction: String,   // "long", "short"
    confidence: f64,
    price_target: Option<f64>,
    stop_loss: Option<f64>,
    timestamp: i64,
}

#[derive(Debug, Clone)]
struct SignalAggregator {
    signals: Arc<Mutex<HashMap<String, Vec<StrategySignal>>>>,
}

impl SignalAggregator {
    fn new() -> Self {
        SignalAggregator {
            signals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_signal(&self, signal: StrategySignal) {
        let mut signals = self.signals.lock().unwrap();
        signals
            .entry(signal.symbol.clone())
            .or_insert_with(Vec::new)
            .push(signal);
    }

    fn get_consensus(&self, symbol: &str) -> Option<String> {
        let signals = self.signals.lock().unwrap();

        if let Some(symbol_signals) = signals.get(symbol) {
            if symbol_signals.is_empty() {
                return None;
            }

            let long_confidence: f64 = symbol_signals
                .iter()
                .filter(|s| s.direction == "long")
                .map(|s| s.confidence)
                .sum();

            let short_confidence: f64 = symbol_signals
                .iter()
                .filter(|s| s.direction == "short")
                .map(|s| s.confidence)
                .sum();

            if long_confidence > short_confidence && long_confidence > 0.5 {
                Some(format!("LONG (уверенность: {:.1}%)", long_confidence * 100.0))
            } else if short_confidence > long_confidence && short_confidence > 0.5 {
                Some(format!("SHORT (уверенность: {:.1}%)", short_confidence * 100.0))
            } else {
                Some("НЕЙТРАЛЬНО".to_string())
            }
        } else {
            None
        }
    }
}

fn main() -> redis::RedisResult<()> {
    let aggregator = SignalAggregator::new();
    let aggregator_clone = aggregator.clone();

    // Подписчик на сигналы стратегий
    let signal_subscriber = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        con.psubscribe(&["strategy:*:signals"], |msg| {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload().unwrap();

            if let Ok(signal) = serde_json::from_str::<StrategySignal>(&payload) {
                println!(
                    "[Сигнал] {} от {}: {} {} (уверенность: {:.0}%)",
                    signal.signal_type.to_uppercase(),
                    signal.strategy_id,
                    signal.direction.to_uppercase(),
                    signal.symbol,
                    signal.confidence * 100.0
                );

                if let Some(target) = signal.price_target {
                    println!("  Цель: ${:.2}", target);
                }
                if let Some(sl) = signal.stop_loss {
                    println!("  Стоп-лосс: ${:.2}", sl);
                }

                aggregator_clone.add_signal(signal);
            }

            redis::ControlFlow::Continue
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(100));

    // Публикация сигналов от разных стратегий
    let publisher = thread::spawn(move || {
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let mut con = client.get_connection().unwrap();

        let strategies = vec![
            ("momentum", "long", 0.75),
            ("mean_reversion", "short", 0.60),
            ("breakout", "long", 0.85),
            ("ml_predictor", "long", 0.70),
        ];

        for (strategy, direction, confidence) in strategies {
            let signal = StrategySignal {
                strategy_id: strategy.to_string(),
                signal_type: "entry".to_string(),
                symbol: "BTC/USDT".to_string(),
                direction: direction.to_string(),
                confidence,
                price_target: Some(45000.0),
                stop_loss: Some(40000.0),
                timestamp: chrono::Utc::now().timestamp(),
            };

            let channel = format!("strategy:{}:signals", strategy);
            let json = serde_json::to_string(&signal).unwrap();
            let _: i32 = con.publish(&channel, &json).unwrap();

            thread::sleep(Duration::from_millis(300));
        }
    });

    publisher.join().unwrap();
    thread::sleep(Duration::from_secs(1));

    // Показываем консенсус
    if let Some(consensus) = aggregator.get_consensus("BTC/USDT") {
        println!("\n[Консенсус] BTC/USDT: {}", consensus);
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Pub/Sub | Паттерн обмена сообщениями «издатель-подписчик» |
| Publisher | Клиент, публикующий сообщения в каналы |
| Subscriber | Клиент, получающий сообщения из каналов |
| Channel | Именованный поток сообщений |
| `subscribe` | Подписка на конкретные каналы |
| `psubscribe` | Подписка по маске (паттерну) |
| Fire-and-forget | Сообщения не сохраняются — только доставка онлайн-подписчикам |

## Упражнения

1. **Монитор цен**: Создай систему, которая подписывается на обновления цен нескольких криптовалют и выводит уведомление, когда цена изменяется более чем на 1% за последние 5 минут.

2. **Оповещение о ордерах**: Реализуй систему уведомлений, которая:
   - Подписывается на каналы `orders:pending`, `orders:filled`, `orders:cancelled`
   - Ведёт статистику по каждому типу события
   - Отправляет алерт, если количество отменённых ордеров превышает 10% от общего числа

3. **Маршрутизатор сигналов**: Напиши программу, которая:
   - Получает сигналы от нескольких торговых стратегий через Pub/Sub
   - Агрегирует сигналы по инструментам
   - Публикует консолидированные сигналы в отдельный канал

4. **Мультиплексор каналов**: Создай async-систему с использованием Tokio, которая:
   - Подписывается на несколько каналов одновременно
   - Обрабатывает сообщения из разных каналов в разных задачах
   - Реализует graceful shutdown при получении сигнала завершения

## Домашнее задание

Реализуй полноценную систему уведомлений для торгового бота:

1. **Издатель маркет-даты**: Публикует обновления цен каждые 100мс
2. **Издатель ордеров**: Публикует статусы ордеров при их изменении
3. **Подписчик-агрегатор**: Собирает данные и вычисляет метрики
4. **Подписчик-алертер**: Отправляет уведомления при выполнении условий:
   - Резкое изменение цены (> 2% за минуту)
   - Исполнение крупного ордера (> 1 BTC)
   - Просадка портфеля (> 5%)

Добавь обработку ошибок соединения с автоматическим переподключением.

## Навигация

[← Предыдущий день](../236-redis-sorted-sets-leaderboard/ru.md) | [Следующий день →](../238-redis-streams-event-sourcing/ru.md)
