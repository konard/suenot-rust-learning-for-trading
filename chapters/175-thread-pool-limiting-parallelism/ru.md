# День 175: Пул потоков: ограничиваем параллелизм

## Аналогия из трейдинга

Представь себе торговый зал биржи. Если каждый трейдер будет нанимать нового помощника для каждой сделки, зал быстро переполнится, люди будут толкаться и мешать друг другу. Гораздо эффективнее иметь фиксированную команду из, скажем, 8 трейдеров, которые берут заявки из очереди и исполняют их по мере освобождения.

Это и есть **пул потоков (thread pool)** — заранее созданный набор потоков, которые ожидают задачи и выполняют их. Вместо создания нового потока для каждой задачи (что дорого), мы переиспользуем существующие потоки.

В алготрейдинге пул потоков критически важен:
- Одновременный анализ 1000 акций не должен создавать 1000 потоков
- Обработка рыночных данных должна быть ограничена возможностями CPU
- Исполнение ордеров требует контролируемого параллелизма

## Зачем ограничивать параллелизм?

```
Без пула потоков:                    С пулом потоков:
┌─────────────────────┐              ┌─────────────────────┐
│ 1000 задач          │              │ 1000 задач          │
│        ↓            │              │        ↓            │
│ 1000 потоков!       │              │ Очередь задач       │
│ (переключение       │              │        ↓            │
│  контекста,         │              │ 8 потоков (= CPU)   │
│  голодание,         │              │ (эффективное        │
│  исчерпание         │              │  использование      │
│  ресурсов)          │              │  ресурсов)          │
└─────────────────────┘              └─────────────────────┘
```

## Простой пул потоков вручную

Сначала рассмотрим, как работает пул потоков изнутри:

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Воркер {} получил задачу", id);
                    job();
                }
                Err(_) => {
                    println!("Воркер {} завершает работу", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn main() {
    let pool = ThreadPool::new(4);

    // Имитация анализа 10 акций
    for i in 0..10 {
        pool.execute(move || {
            let symbol = format!("STOCK_{}", i);
            println!("Анализирую {}", symbol);
            thread::sleep(std::time::Duration::from_millis(100));
            println!("{}: анализ завершён", symbol);
        });
    }

    // Pool автоматически дождётся завершения при выходе из scope
}
```

## Использование rayon — промышленный стандарт

На практике используют библиотеку `rayon`, которая предоставляет мощный и оптимизированный пул потоков:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct StockAnalysis {
    symbol: String,
    price: f64,
    sma_20: f64,
    sma_50: f64,
    signal: Signal,
}

#[derive(Debug, Clone)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

fn analyze_stock(symbol: &str, prices: &[f64]) -> StockAnalysis {
    // Рассчитываем скользящие средние
    let sma_20 = if prices.len() >= 20 {
        prices.iter().rev().take(20).sum::<f64>() / 20.0
    } else {
        prices.iter().sum::<f64>() / prices.len() as f64
    };

    let sma_50 = if prices.len() >= 50 {
        prices.iter().rev().take(50).sum::<f64>() / 50.0
    } else {
        prices.iter().sum::<f64>() / prices.len() as f64
    };

    let current_price = *prices.last().unwrap();

    let signal = if sma_20 > sma_50 && current_price > sma_20 {
        Signal::Buy
    } else if sma_20 < sma_50 && current_price < sma_20 {
        Signal::Sell
    } else {
        Signal::Hold
    };

    StockAnalysis {
        symbol: symbol.to_string(),
        price: current_price,
        sma_20,
        sma_50,
        signal,
    }
}

fn main() {
    // Имитация данных для 100 акций
    let stocks: Vec<(String, Vec<f64>)> = (0..100)
        .map(|i| {
            let symbol = format!("STOCK_{:03}", i);
            let prices: Vec<f64> = (0..100)
                .map(|j| 100.0 + (i as f64 * 0.1) + (j as f64 * 0.01))
                .collect();
            (symbol, prices)
        })
        .collect();

    // Параллельный анализ с ограниченным числом потоков (по умолчанию = CPU cores)
    let results: Vec<StockAnalysis> = stocks
        .par_iter()  // Параллельный итератор!
        .map(|(symbol, prices)| analyze_stock(symbol, prices))
        .collect();

    // Выводим сигналы на покупку
    let buy_signals: Vec<_> = results
        .iter()
        .filter(|a| matches!(a.signal, Signal::Buy))
        .collect();

    println!("Найдено {} сигналов на покупку:", buy_signals.len());
    for analysis in buy_signals.iter().take(5) {
        println!(
            "  {} @ ${:.2} (SMA20: {:.2}, SMA50: {:.2})",
            analysis.symbol, analysis.price, analysis.sma_20, analysis.sma_50
        );
    }
}
```

## Настройка размера пула в rayon

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Создаём глобальный пул с 4 потоками
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let symbols: Vec<String> = (0..20)
        .map(|i| format!("CRYPTO_{}", i))
        .collect();

    // Теперь параллельные операции используют только 4 потока
    symbols.par_iter().for_each(|symbol| {
        println!(
            "Поток {:?} обрабатывает {}",
            std::thread::current().id(),
            symbol
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    });
}
```

## Локальный пул потоков

Иногда нужно несколько независимых пулов:

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::Arc;

struct TradingSystem {
    data_pool: rayon::ThreadPool,     // Для обработки рыночных данных
    analysis_pool: rayon::ThreadPool,  // Для анализа
    order_pool: rayon::ThreadPool,     // Для работы с ордерами
}

impl TradingSystem {
    fn new() -> Self {
        TradingSystem {
            // Много данных — много потоков
            data_pool: ThreadPoolBuilder::new()
                .num_threads(8)
                .thread_name(|i| format!("data-worker-{}", i))
                .build()
                .unwrap(),

            // Анализ CPU-intensive — по количеству ядер
            analysis_pool: ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .thread_name(|i| format!("analysis-worker-{}", i))
                .build()
                .unwrap(),

            // Ордера критичны — меньше потоков, выше приоритет
            order_pool: ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("order-worker-{}", i))
                .build()
                .unwrap(),
        }
    }

    fn process_market_data(&self, data: Vec<f64>) -> Vec<f64> {
        self.data_pool.install(|| {
            data.par_iter()
                .map(|&price| price * 1.0001) // Имитация обработки
                .collect()
        })
    }

    fn analyze_positions(&self, positions: Vec<(String, f64)>) -> Vec<String> {
        self.analysis_pool.install(|| {
            positions
                .par_iter()
                .filter(|(_, value)| *value > 10000.0)
                .map(|(symbol, value)| {
                    format!("{}: ${:.2} - требует внимания", symbol, value)
                })
                .collect()
        })
    }
}

fn main() {
    let system = TradingSystem::new();

    // Имитация рыночных данных
    let prices: Vec<f64> = (0..1000).map(|i| 100.0 + i as f64 * 0.1).collect();

    // Обработка в data_pool
    let processed = system.process_market_data(prices);
    println!("Обработано {} цен", processed.len());

    // Анализ позиций в analysis_pool
    let positions: Vec<(String, f64)> = vec![
        ("BTC".to_string(), 50000.0),
        ("ETH".to_string(), 3000.0),
        ("SOL".to_string(), 15000.0),
    ];

    let alerts = system.analyze_positions(positions);
    for alert in alerts {
        println!("{}", alert);
    }
}
```

## Пул потоков для обработки ордеров

```rust
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::collections::VecDeque;
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    order_id: u64,
    status: ExecutionStatus,
    filled_price: Option<f64>,
}

#[derive(Debug, Clone)]
enum ExecutionStatus {
    Filled,
    PartiallyFilled,
    Rejected(String),
}

struct OrderExecutor {
    order_counter: AtomicU64,
    execution_log: Arc<Mutex<Vec<ExecutionResult>>>,
}

impl OrderExecutor {
    fn new() -> Self {
        OrderExecutor {
            order_counter: AtomicU64::new(0),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn execute_order(&self, order: &Order) -> ExecutionResult {
        // Имитация проверки и исполнения ордера
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = if order.quantity > 1000.0 {
            ExecutionResult {
                order_id: order.id,
                status: ExecutionStatus::Rejected(
                    "Слишком большой объём".to_string()
                ),
                filled_price: None,
            }
        } else {
            // Имитация slippage
            let slippage = match order.side {
                OrderSide::Buy => 1.001,
                OrderSide::Sell => 0.999,
            };

            ExecutionResult {
                order_id: order.id,
                status: ExecutionStatus::Filled,
                filled_price: Some(order.price * slippage),
            }
        };

        self.execution_log.lock().unwrap().push(result.clone());
        result
    }

    fn process_batch(&self, orders: Vec<Order>) -> Vec<ExecutionResult> {
        // Параллельная обработка с ограниченным параллелизмом
        orders
            .par_iter()
            .map(|order| self.execute_order(order))
            .collect()
    }

    fn get_stats(&self) -> (usize, usize, usize) {
        let log = self.execution_log.lock().unwrap();
        let filled = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Filled))
            .count();
        let partial = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::PartiallyFilled))
            .count();
        let rejected = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Rejected(_)))
            .count();
        (filled, partial, rejected)
    }
}

fn main() {
    // Ограничиваем до 4 потоков для обработки ордеров
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let executor = OrderExecutor::new();

    // Создаём пачку ордеров
    let orders: Vec<Order> = (0..50)
        .map(|i| Order {
            id: i,
            symbol: if i % 2 == 0 { "BTC" } else { "ETH" }.to_string(),
            side: if i % 3 == 0 { OrderSide::Sell } else { OrderSide::Buy },
            quantity: 10.0 + (i as f64 * 50.0),
            price: 42000.0 + (i as f64 * 10.0),
        })
        .collect();

    println!("Обрабатываем {} ордеров...", orders.len());

    let start = std::time::Instant::now();
    let results = executor.process_batch(orders);
    let elapsed = start.elapsed();

    println!("Обработано за {:?}", elapsed);

    let (filled, partial, rejected) = executor.get_stats();
    println!("\nРезультаты:");
    println!("  Исполнено: {}", filled);
    println!("  Частично: {}", partial);
    println!("  Отклонено: {}", rejected);

    // Выводим первые 5 результатов
    println!("\nПримеры исполнения:");
    for result in results.iter().take(5) {
        match &result.status {
            ExecutionStatus::Filled => {
                println!(
                    "  Ордер {}: исполнен @ ${:.2}",
                    result.order_id,
                    result.filled_price.unwrap()
                );
            }
            ExecutionStatus::Rejected(reason) => {
                println!("  Ордер {}: отклонён - {}", result.order_id, reason);
            }
            ExecutionStatus::PartiallyFilled => {
                println!("  Ордер {}: частично исполнен", result.order_id);
            }
        }
    }
}
```

## Контроль нагрузки с помощью семафора

Иногда нужен ещё более тонкий контроль:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct RateLimitedClient {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl RateLimitedClient {
    fn new(max_concurrent: usize) -> Self {
        RateLimitedClient {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<f64, String> {
        // Ждём разрешения от семафора
        let _permit = self.semaphore.acquire().await.unwrap();

        println!(
            "Запрашиваем цену {} (активных запросов: {})",
            symbol,
            self.max_concurrent - self.semaphore.available_permits()
        );

        // Имитация API запроса
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Permit автоматически освобождается здесь
        Ok(42000.0 + rand::random::<f64>() * 1000.0)
    }
}

#[tokio::main]
async fn main() {
    // Максимум 3 одновременных запроса к API
    let client = RateLimitedClient::new(3);

    let symbols = vec!["BTC", "ETH", "SOL", "ADA", "DOT", "LINK", "AVAX", "MATIC"];

    let mut handles = vec![];

    for symbol in symbols {
        let client = client.clone();
        let symbol = symbol.to_string();

        handles.push(tokio::spawn(async move {
            match client.fetch_price(&symbol).await {
                Ok(price) => println!("{}: ${:.2}", symbol, price),
                Err(e) => println!("{}: ошибка - {}", symbol, e),
            }
        }));
    }

    // Ждём завершения всех запросов
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Сравнение подходов

| Подход | Когда использовать | Преимущества |
|--------|-------------------|--------------|
| Ручной ThreadPool | Обучение, простые случаи | Полный контроль |
| rayon | CPU-bound задачи | Автоматический параллелизм |
| tokio + Semaphore | Async I/O | Контроль конкурентности |
| Несколько пулов | Разные приоритеты | Изоляция задач |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Thread Pool | Набор переиспользуемых потоков для выполнения задач |
| Work Stealing | Потоки крадут задачи друг у друга (rayon) |
| Ограничение параллелизма | Контроль над количеством одновременных операций |
| rayon | Библиотека для параллельных итераторов |
| Semaphore | Ограничение конкурентности в async коде |

## Практические задания

1. **Простой пул потоков**: Реализуй свой ThreadPool с методом `execute()` и корректным завершением работы через Drop.

2. **Параллельный скринер акций**: Используя rayon, создай функцию, которая:
   - Принимает список из 1000 символов акций
   - Для каждой рассчитывает технические индикаторы
   - Возвращает топ-10 по какому-либо критерию

3. **Rate-limited API клиент**: Реализуй клиент с ограничением:
   - Максимум 10 запросов в секунду
   - Максимум 3 одновременных запроса
   - Очередь ожидающих запросов

4. **Приоритетные пулы**: Создай систему с тремя пулами:
   - High Priority (2 потока) — для критичных операций
   - Normal (4 потока) — для обычных задач
   - Background (2 потока) — для фоновых вычислений

## Домашнее задание

1. **Бэктестинг с пулом потоков**: Напиши программу, которая параллельно тестирует торговую стратегию на 100 различных инструментах. Используй rayon для ограничения параллелизма количеством ядер CPU.

2. **Мониторинг позиций**: Создай систему, которая каждую секунду проверяет PnL по 50 открытым позициям. Ограничь параллельные вычисления до 8 потоков и собирай статистику времени обработки.

3. **Конкурентная биржа**: Реализуй структуру `ExchangeSimulator`, которая:
   - Имеет пул для обработки входящих ордеров
   - Имеет пул для мэтчинга ордеров
   - Имеет пул для рассылки уведомлений
   - Каждый пул ограничен разным количеством потоков

4. **Адаптивный пул**: Создай пул, который:
   - Начинает с 2 потоков
   - Увеличивается до 8, если очередь задач растёт
   - Уменьшается обратно при низкой нагрузке
   - Логирует изменения размера

## Навигация

[← Предыдущий день](../174-scoped-threads/ru.md) | [Следующий день →](../176-work-stealing/ru.md)
