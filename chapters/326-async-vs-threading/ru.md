# День 326: Async vs Threading: выбор модели

## Аналогия из трейдинга

Представь, что ты управляешь торговым залом с несколькими рабочими станциями. У тебя есть два способа обрабатывать входящие ордера:

**Модель Threading (Многопоточность):**
Ты нанимаешь несколько трейдеров. Каждый трейдер сидит за своим рабочим местом и обрабатывает свои ордера независимо. Когда один трейдер ждёт подтверждения сделки от биржи, он просто сидит и ждёт — его рабочее место занято.

**Модель Async (Асинхронность):**
У тебя один трейдер, но он умеет работать очень эффективно. Когда он отправляет ордер на биржу и ждёт ответа, он не сидит без дела — переключается на другой ордер. Когда приходит ответ по первому ордеру, он возвращается к нему.

| Критерий | Threading | Async |
|----------|-----------|-------|
| **Аналогия** | Много трейдеров | Один мульти-задачный трейдер |
| **Ожидание I/O** | Блокирует поток | Переключается на другую задачу |
| **Память** | ~2-8 MB на поток | ~1-4 KB на задачу |
| **Переключение контекста** | Дорого (OS) | Дёшево (runtime) |
| **Лучше для** | CPU-интенсивные задачи | I/O-интенсивные задачи |

## Когда выбирать Threading?

Threading подходит для **CPU-bound** задач, где нужна реальная параллельная работа процессора:

```rust
use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex};

/// Расчёт скользящей средней — CPU-интенсивная операция
fn calculate_sma(prices: &[f64], window: usize) -> Vec<f64> {
    prices
        .windows(window)
        .map(|w| w.iter().sum::<f64>() / window as f64)
        .collect()
}

/// Расчёт RSI — тоже требует CPU
fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    let mut rsi_values = Vec::new();
    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if avg_loss != 0.0 {
            avg_gain / avg_loss
        } else {
            100.0
        };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    rsi_values
}

fn main() {
    // Генерируем данные для 100 инструментов
    let instruments: Vec<Vec<f64>> = (0..100)
        .map(|i| {
            (0..10000)
                .map(|j| 100.0 + (i as f64 * 0.01) + (j as f64 * 0.001).sin())
                .collect()
        })
        .collect();

    println!("=== Сравнение Threading vs Sequential ===\n");

    // Последовательный расчёт
    let start = Instant::now();
    let mut sequential_results = Vec::new();
    for prices in &instruments {
        let sma = calculate_sma(prices, 20);
        let rsi = calculate_rsi(prices, 14);
        sequential_results.push((sma, rsi));
    }
    let sequential_time = start.elapsed();
    println!("Последовательно: {:?}", sequential_time);

    // Многопоточный расчёт
    let start = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Разбиваем на чанки для потоков
    let chunk_size = instruments.len() / 4;
    for chunk in instruments.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let mut local_results = Vec::new();
            for prices in &chunk {
                let sma = calculate_sma(prices, 20);
                let rsi = calculate_rsi(prices, 14);
                local_results.push((sma, rsi));
            }
            results.lock().unwrap().extend(local_results);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let threaded_time = start.elapsed();
    println!("Многопоточно (4 потока): {:?}", threaded_time);

    println!(
        "\nУскорение: {:.2}x",
        sequential_time.as_secs_f64() / threaded_time.as_secs_f64()
    );
}
```

## Когда выбирать Async?

Async подходит для **I/O-bound** задач, где большую часть времени программа ждёт внешних событий:

```rust
use std::time::Duration;

// Эмуляция асинхронного запроса к бирже
async fn fetch_price(exchange: &str, symbol: &str) -> Result<f64, String> {
    // Имитация сетевой задержки
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Возвращаем "цену" на основе хэша для демонстрации
    let hash = exchange.len() + symbol.len();
    Ok(50000.0 + (hash as f64 * 100.0))
}

async fn fetch_order_book(exchange: &str, symbol: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Имитация стакана заявок
    let bids = vec![49900.0, 49850.0, 49800.0];
    let asks = vec![50100.0, 50150.0, 50200.0];
    Ok((bids, asks))
}

async fn submit_order(exchange: &str, symbol: &str, side: &str, price: f64, qty: f64)
    -> Result<String, String>
{
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(format!("ORDER-{}-{}-{}", exchange, symbol, price as u64))
}

#[tokio::main]
async fn main() {
    use std::time::Instant;

    println!("=== Async: Параллельные запросы к биржам ===\n");

    let exchanges = ["binance", "kraken", "coinbase", "bybit"];
    let symbol = "BTCUSDT";

    // Последовательные запросы
    let start = Instant::now();
    for exchange in &exchanges {
        let price = fetch_price(exchange, symbol).await.unwrap();
        println!("{}: ${:.2}", exchange, price);
    }
    println!("Последовательно: {:?}\n", start.elapsed());

    // Параллельные запросы с async
    let start = Instant::now();
    let futures: Vec<_> = exchanges
        .iter()
        .map(|exchange| async move {
            let price = fetch_price(exchange, symbol).await?;
            Ok::<_, String>((exchange.to_string(), price))
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    for result in results {
        if let Ok((exchange, price)) = result {
            println!("{}: ${:.2}", exchange, price);
        }
    }
    println!("Параллельно (async): {:?}", start.elapsed());
}
```

## Гибридный подход: Async + Threading

В реальных торговых системах часто используют комбинацию обоих подходов:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Данные рынка — обновляются асинхронно
#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
}

/// Торговый сигнал — результат CPU-интенсивных расчётов
#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    action: String,  // "BUY", "SELL", "HOLD"
    confidence: f64,
    price_target: f64,
}

/// Кэш рыночных данных с thread-safe доступом
struct MarketDataCache {
    data: Arc<RwLock<HashMap<String, MarketData>>>,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn update(&self, market_data: MarketData) {
        let mut data = self.data.write().await;
        data.insert(market_data.symbol.clone(), market_data);
    }

    async fn get(&self, symbol: &str) -> Option<MarketData> {
        let data = self.data.read().await;
        data.get(symbol).cloned()
    }

    async fn get_all(&self) -> Vec<MarketData> {
        let data = self.data.read().await;
        data.values().cloned().collect()
    }
}

/// CPU-интенсивный анализ — запускается в отдельном потоке
fn analyze_market_data(data: Vec<MarketData>) -> Vec<TradingSignal> {
    // Имитация сложных вычислений
    data.iter()
        .map(|md| {
            // Простая стратегия на основе спреда
            let spread_pct = (md.ask - md.bid) / md.bid * 100.0;

            let (action, confidence) = if spread_pct < 0.1 {
                ("BUY", 0.8)
            } else if spread_pct > 0.5 {
                ("SELL", 0.7)
            } else {
                ("HOLD", 0.5)
            };

            TradingSignal {
                symbol: md.symbol.clone(),
                action: action.to_string(),
                confidence,
                price_target: md.last_price * if action == "BUY" { 1.02 } else { 0.98 },
            }
        })
        .collect()
}

#[tokio::main]
async fn main() {
    println!("=== Гибридный подход: Async + Threading ===\n");

    let cache = Arc::new(MarketDataCache::new());

    // Асинхронное обновление данных (имитация WebSocket)
    let cache_clone = Arc::clone(&cache);
    let update_task = tokio::spawn(async move {
        let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT"];

        for (i, symbol) in symbols.iter().enumerate() {
            let md = MarketData {
                symbol: symbol.to_string(),
                bid: 50000.0 + (i as f64 * 1000.0),
                ask: 50010.0 + (i as f64 * 1000.0),
                last_price: 50005.0 + (i as f64 * 1000.0),
                volume: 1000000.0,
            };
            cache_clone.update(md).await;
            println!("Обновлены данные: {}", symbol);
        }
    });

    // Ждём загрузки данных
    update_task.await.unwrap();

    // Получаем все данные для анализа
    let all_data = cache.get_all().await;
    println!("\nЗагружено {} инструментов", all_data.len());

    // CPU-интенсивный анализ в отдельном потоке
    let signals = tokio::task::spawn_blocking(move || {
        println!("Запуск анализа в отдельном потоке...");
        analyze_market_data(all_data)
    })
    .await
    .unwrap();

    println!("\n=== Торговые сигналы ===");
    for signal in signals {
        println!(
            "{}: {} (уверенность: {:.0}%, цель: ${:.2})",
            signal.symbol, signal.action, signal.confidence * 100.0, signal.price_target
        );
    }
}
```

## Сравнение моделей

### Накладные расходы на память

```rust
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};

static TASK_COUNT: AtomicUsize = AtomicUsize::new(0);

fn thread_memory_demo() {
    println!("=== Память: Threads vs Async ===\n");

    // Потоки: каждый занимает ~2-8 MB стека
    println!("Создание 10 потоков...");
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            // Каждый поток имеет свой стек
            let local_data: [u8; 1024] = [0; 1024];
            thread::sleep(std::time::Duration::from_millis(100));
            local_data[0] + i as u8
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
    println!("Потоки завершены (каждый использовал ~2MB стека)\n");

    // Async: задачи занимают минимум памяти
    println!("Создание 10000 async задач...");
    // В реальном коде это было бы:
    // let runtime = tokio::runtime::Runtime::new().unwrap();
    // runtime.block_on(async {
    //     let futures: Vec<_> = (0..10000)
    //         .map(|_| async { tokio::time::sleep(Duration::from_millis(100)).await })
    //         .collect();
    //     futures::future::join_all(futures).await;
    // });
    println!("Async задачи используют ~KB памяти каждая");
}

fn main() {
    thread_memory_demo();
}
```

### Таблица выбора модели

| Сценарий | Рекомендуемая модель | Причина |
|----------|---------------------|---------|
| WebSocket подключения к биржам | Async | I/O-bound, много ожидания |
| REST API запросы | Async | Сетевые задержки |
| Расчёт индикаторов | Threading | CPU-bound |
| Бэктестинг стратегий | Threading | Интенсивные вычисления |
| Обработка ордеров | Async | I/O операции с биржей |
| Анализ больших данных | Threading | CPU-bound |
| Event-driven торговля | Async | Реакция на события |
| Monte-Carlo симуляции | Threading | Вычислительно интенсивно |

## Паттерн: Actor Model для торговых систем

Actor Model хорошо подходит для организации торговых систем:

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;

/// Сообщения для актора рыночных данных
#[derive(Debug, Clone)]
enum MarketMessage {
    PriceUpdate { symbol: String, price: f64 },
    GetPrice { symbol: String, response: mpsc::Sender<Option<f64>> },
    Subscribe { symbol: String },
}

/// Сообщения для актора ордеров
#[derive(Debug)]
enum OrderMessage {
    PlaceOrder { symbol: String, side: String, price: f64, qty: f64 },
    CancelOrder { order_id: String },
    GetOpenOrders { response: mpsc::Sender<Vec<String>> },
}

/// Актор рыночных данных
async fn market_data_actor(mut rx: mpsc::Receiver<MarketMessage>) {
    let mut prices: HashMap<String, f64> = HashMap::new();
    let mut subscriptions: Vec<String> = Vec::new();

    println!("[MarketData Actor] Запущен");

    while let Some(msg) = rx.recv().await {
        match msg {
            MarketMessage::PriceUpdate { symbol, price } => {
                prices.insert(symbol.clone(), price);
                println!("[MarketData] Обновление: {} = ${:.2}", symbol, price);
            }
            MarketMessage::GetPrice { symbol, response } => {
                let price = prices.get(&symbol).copied();
                let _ = response.send(price).await;
            }
            MarketMessage::Subscribe { symbol } => {
                if !subscriptions.contains(&symbol) {
                    subscriptions.push(symbol.clone());
                    println!("[MarketData] Подписка на: {}", symbol);
                }
            }
        }
    }
}

/// Актор управления ордерами
async fn order_manager_actor(
    mut rx: mpsc::Receiver<OrderMessage>,
    market_tx: mpsc::Sender<MarketMessage>,
) {
    let mut orders: Vec<String> = Vec::new();
    let mut order_counter = 0u64;

    println!("[OrderManager Actor] Запущен");

    while let Some(msg) = rx.recv().await {
        match msg {
            OrderMessage::PlaceOrder { symbol, side, price, qty } => {
                order_counter += 1;
                let order_id = format!("ORD-{:06}", order_counter);
                orders.push(order_id.clone());

                println!(
                    "[OrderManager] Новый ордер: {} {} {} {} @ ${:.2}",
                    order_id, side, qty, symbol, price
                );

                // Запрашиваем текущую цену через актор рыночных данных
                let (resp_tx, mut resp_rx) = mpsc::channel(1);
                let _ = market_tx.send(MarketMessage::GetPrice {
                    symbol: symbol.clone(),
                    response: resp_tx,
                }).await;

                if let Some(Some(current_price)) = resp_rx.recv().await {
                    let diff = (price - current_price).abs() / current_price * 100.0;
                    println!(
                        "[OrderManager] Текущая цена: ${:.2}, отклонение: {:.2}%",
                        current_price, diff
                    );
                }
            }
            OrderMessage::CancelOrder { order_id } => {
                orders.retain(|id| id != &order_id);
                println!("[OrderManager] Ордер отменён: {}", order_id);
            }
            OrderMessage::GetOpenOrders { response } => {
                let _ = response.send(orders.clone()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Actor Model для торговой системы ===\n");

    // Создаём каналы для акторов
    let (market_tx, market_rx) = mpsc::channel::<MarketMessage>(100);
    let (order_tx, order_rx) = mpsc::channel::<OrderMessage>(100);

    // Запускаем акторы
    let market_handle = tokio::spawn(market_data_actor(market_rx));
    let order_handle = tokio::spawn(order_manager_actor(order_rx, market_tx.clone()));

    // Симуляция работы системы
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Подписка на инструменты
    market_tx.send(MarketMessage::Subscribe {
        symbol: "BTCUSDT".to_string()
    }).await.unwrap();

    // Обновление цен
    market_tx.send(MarketMessage::PriceUpdate {
        symbol: "BTCUSDT".to_string(),
        price: 50000.0
    }).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Размещение ордеров
    order_tx.send(OrderMessage::PlaceOrder {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 49900.0,
        qty: 0.1,
    }).await.unwrap();

    order_tx.send(OrderMessage::PlaceOrder {
        symbol: "BTCUSDT".to_string(),
        side: "SELL".to_string(),
        price: 50100.0,
        qty: 0.1,
    }).await.unwrap();

    // Даём время на обработку
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Получаем список открытых ордеров
    let (resp_tx, mut resp_rx) = mpsc::channel(1);
    order_tx.send(OrderMessage::GetOpenOrders { response: resp_tx }).await.unwrap();

    if let Some(orders) = resp_rx.recv().await {
        println!("\nОткрытые ордера: {:?}", orders);
    }

    // Закрываем каналы для завершения акторов
    drop(market_tx);
    drop(order_tx);

    let _ = market_handle.await;
    let _ = order_handle.await;

    println!("\nАкторы завершены");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Threading** | Параллельное выполнение на нескольких ядрах CPU |
| **Async** | Кооперативная многозадачность для I/O операций |
| **CPU-bound** | Задачи, ограниченные вычислительной мощностью |
| **I/O-bound** | Задачи, ограниченные скоростью ввода-вывода |
| **spawn_blocking** | Запуск CPU-задач из async контекста |
| **Actor Model** | Паттерн изоляции состояния через сообщения |
| **Гибридный подход** | Комбинация async для I/O и threading для CPU |

## Практические задания

1. **Параллельный загрузчик данных**: Создай систему, которая:
   - Асинхронно загружает исторические данные с нескольких бирж
   - Использует потоки для параллельной обработки данных
   - Объединяет результаты и сохраняет в единый формат
   - Показывает прогресс загрузки

2. **Распределённый расчёт индикаторов**: Реализуй систему:
   - Использует thread pool для расчёта тяжёлых индикаторов
   - Асинхронно получает рыночные данные
   - Кеширует результаты расчётов
   - Обновляет индикаторы при новых данных

3. **Мониторинг производительности**: Создай инструмент:
   - Сравнивает время выполнения async vs threading
   - Измеряет использование памяти
   - Визуализирует результаты
   - Даёт рекомендации по выбору модели

4. **Event-driven торговый бот**: Реализуй бота:
   - Использует async для обработки событий
   - Применяет threading для сложных расчётов
   - Управляет состоянием через акторы
   - Логирует все операции

## Домашнее задание

1. **Бенчмарк async vs threading**: Напиши тест, который:
   - Создаёт 1000 задач с разным соотношением I/O и CPU
   - Измеряет время выполнения для обеих моделей
   - Находит точку перегиба, где одна модель лучше другой
   - Генерирует отчёт с графиками
   - Даёт рекомендации на основе результатов

2. **Торговая система с горячим переключением**: Реализуй систему:
   - Позволяет переключаться между async и threading в runtime
   - Сохраняет состояние при переключении
   - Измеряет производительность обоих режимов
   - Автоматически выбирает оптимальный режим
   - Логирует причины переключения

3. **Пул воркеров для анализа**: Создай пул:
   - Динамически масштабируется под нагрузку
   - Распределяет CPU-задачи между потоками
   - Использует async для координации
   - Собирает метрики производительности
   - Имеет механизм graceful shutdown

4. **Симулятор биржи**: Разработай симулятор:
   - Async обработка подключений клиентов
   - Threading для matching engine
   - Actor model для изоляции компонентов
   - Тестирование под высокой нагрузкой
   - Мониторинг латентности операций

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../327-*/ru.md)
