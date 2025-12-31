# День 176: Отмена операций: Graceful Shutdown

## Аналогия из трейдинга

Представь, что ты запустил торгового бота, который обрабатывает рыночные данные, выставляет ордера и следит за позициями. Внезапно ты получаешь критическое уведомление от биржи — системное обслуживание через 5 минут. Что делать?

**Плохой вариант:** Просто убить процесс. Результат:
- Открытые ордера зависнут на бирже
- Позиции останутся незакрытыми
- Данные могут быть потеряны
- Баланс не сохранится

**Хороший вариант (Graceful Shutdown):**
1. Получить сигнал на остановку
2. Прекратить приём новых ордеров
3. Отменить все активные ордера на бирже
4. Дождаться закрытия всех позиций (или закрыть их по рынку)
5. Сохранить состояние в базу данных
6. Корректно завершить все соединения

Это и есть **graceful shutdown** — контролируемое завершение работы, при котором все операции корректно завершаются, ресурсы освобождаются, и система остаётся в согласованном состоянии.

## Зачем нужен Graceful Shutdown?

В реальных торговых системах graceful shutdown критически важен:

| Сценарий | Без graceful shutdown | С graceful shutdown |
|----------|----------------------|---------------------|
| Деплой новой версии | Потеря активных ордеров | Все ордера безопасно отменены |
| Серверное обслуживание | Рассогласование позиций | Позиции сохранены в БД |
| Критическая ошибка | Зависшие WebSocket соединения | Все соединения закрыты |
| Ручная остановка | Утечка ресурсов | Память и файлы освобождены |

## Сигналы в Unix/Linux

Основные сигналы для graceful shutdown:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Флаг для сигнала остановки
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown_requested);

    // Обработчик Ctrl+C (SIGINT)
    ctrlc::set_handler(move || {
        println!("\nПолучен сигнал остановки (Ctrl+C)...");
        shutdown_clone.store(true, Ordering::SeqCst);
    }).expect("Ошибка установки обработчика Ctrl+C");

    println!("Торговый бот запущен. Нажмите Ctrl+C для остановки.");

    // Основной цикл
    while !shutdown_requested.load(Ordering::SeqCst) {
        println!("Обрабатываю рыночные данные...");
        thread::sleep(Duration::from_secs(1));
    }

    println!("Выполняю graceful shutdown...");
    // Здесь логика корректного завершения
    println!("Бот остановлен.");
}
```

**Примечание:** Для компиляции добавьте в `Cargo.toml`:
```toml
[dependencies]
ctrlc = "3.4"
```

## Паттерн: Cancellation Token

В многопоточных приложениях удобно использовать токен отмены:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Токен для координации отмены операций
#[derive(Clone)]
struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Проверить, была ли запрошена отмена
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Запросить отмену
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// Сборщик рыночных данных
fn market_data_collector(token: CancellationToken, symbol: &str) {
    println!("[{}] Сборщик данных запущен", symbol);

    while !token.is_cancelled() {
        // Имитация получения данных
        println!("[{}] Получена котировка: ${:.2}", symbol, 42000.0 + rand_price());
        thread::sleep(Duration::from_millis(500));
    }

    println!("[{}] Сборщик данных остановлен", symbol);
}

/// Исполнитель ордеров
fn order_executor(token: CancellationToken) {
    println!("[OrderExecutor] Запущен");

    let mut pending_orders = 5; // Имитация активных ордеров

    while !token.is_cancelled() || pending_orders > 0 {
        if token.is_cancelled() {
            // Отменяем оставшиеся ордера
            println!("[OrderExecutor] Отменяю ордер... (осталось: {})", pending_orders);
            pending_orders -= 1;
            thread::sleep(Duration::from_millis(200));
        } else {
            // Обычная работа
            println!("[OrderExecutor] Ожидаю новые ордера...");
            thread::sleep(Duration::from_secs(1));
        }
    }

    println!("[OrderExecutor] Все ордера отменены, остановка");
}

fn rand_price() -> f64 {
    // Простая "случайность" для примера
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64
}

fn main() {
    let token = CancellationToken::new();

    // Запускаем сборщики данных
    let btc_token = token.clone();
    let btc_collector = thread::spawn(move || {
        market_data_collector(btc_token, "BTC");
    });

    let eth_token = token.clone();
    let eth_collector = thread::spawn(move || {
        market_data_collector(eth_token, "ETH");
    });

    // Запускаем исполнитель ордеров
    let executor_token = token.clone();
    let executor = thread::spawn(move || {
        order_executor(executor_token);
    });

    // Имитируем работу
    thread::sleep(Duration::from_secs(3));

    // Инициируем graceful shutdown
    println!("\n=== Инициирую graceful shutdown ===\n");
    token.cancel();

    // Ждём завершения всех потоков
    btc_collector.join().unwrap();
    eth_collector.join().unwrap();
    executor.join().unwrap();

    println!("\n=== Все компоненты остановлены ===");
}
```

## Каналы для координации shutdown

Для более гибкой координации используем каналы:

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Команды для управления воркером
enum WorkerCommand {
    ProcessOrder { id: u64, symbol: String, quantity: f64 },
    Shutdown,
}

/// Результаты работы воркера
enum WorkerResult {
    OrderProcessed { id: u64, success: bool },
    ShutdownComplete { pending_orders: usize },
}

fn order_worker(
    commands: Receiver<WorkerCommand>,
    results: Sender<WorkerResult>,
) {
    let mut pending = Vec::new();

    loop {
        match commands.recv() {
            Ok(WorkerCommand::ProcessOrder { id, symbol, quantity }) => {
                println!("Обрабатываю ордер #{}: {} x {}", id, symbol, quantity);
                pending.push(id);

                // Имитация обработки
                thread::sleep(Duration::from_millis(100));

                pending.retain(|&x| x != id);
                let _ = results.send(WorkerResult::OrderProcessed { id, success: true });
            }

            Ok(WorkerCommand::Shutdown) => {
                println!("Получена команда shutdown, отменяю {} ордеров...", pending.len());

                // Отменяем все активные ордера
                for order_id in &pending {
                    println!("  Отменяю ордер #{}", order_id);
                    thread::sleep(Duration::from_millis(50));
                }

                let _ = results.send(WorkerResult::ShutdownComplete {
                    pending_orders: pending.len(),
                });
                break;
            }

            Err(_) => {
                // Канал закрыт
                println!("Канал команд закрыт, завершаюсь");
                break;
            }
        }
    }
}

fn main() {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    // Запускаем воркер
    let worker = thread::spawn(move || {
        order_worker(cmd_rx, result_tx);
    });

    // Отправляем несколько ордеров
    for i in 1..=5 {
        cmd_tx.send(WorkerCommand::ProcessOrder {
            id: i,
            symbol: "BTC".to_string(),
            quantity: 0.1 * i as f64,
        }).unwrap();
    }

    // Получаем результаты
    for _ in 0..5 {
        if let Ok(WorkerResult::OrderProcessed { id, success }) = result_rx.recv() {
            println!("Ордер #{} обработан: {}", id, if success { "успех" } else { "ошибка" });
        }
    }

    // Инициируем graceful shutdown
    println!("\n=== Отправляю команду shutdown ===\n");
    cmd_tx.send(WorkerCommand::Shutdown).unwrap();

    // Ждём подтверждения завершения
    if let Ok(WorkerResult::ShutdownComplete { pending_orders }) = result_rx.recv() {
        println!("Shutdown завершён, отменено {} ордеров", pending_orders);
    }

    worker.join().unwrap();
}
```

## Практический пример: Торговый бот с Graceful Shutdown

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Состояние ордера
#[derive(Debug, Clone)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

/// Ордер
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    status: OrderStatus,
}

/// Позиция
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

/// Состояние торгового бота
struct TradingBot {
    running: AtomicBool,
    shutdown_requested: AtomicBool,
    next_order_id: AtomicU64,
    orders: Mutex<HashMap<u64, Order>>,
    positions: Mutex<HashMap<String, Position>>,
    cash_balance: Mutex<f64>,
}

impl TradingBot {
    fn new(initial_cash: f64) -> Arc<Self> {
        Arc::new(TradingBot {
            running: AtomicBool::new(true),
            shutdown_requested: AtomicBool::new(false),
            next_order_id: AtomicU64::new(1),
            orders: Mutex::new(HashMap::new()),
            positions: Mutex::new(HashMap::new()),
            cash_balance: Mutex::new(initial_cash),
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    fn request_shutdown(&self) {
        println!("[Bot] Запрошен graceful shutdown");
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Создать новый ордер
    fn place_order(&self, symbol: &str, quantity: f64, price: f64) -> Option<u64> {
        if self.is_shutdown_requested() {
            println!("[Bot] Отклоняю новый ордер — идёт shutdown");
            return None;
        }

        let id = self.next_order_id.fetch_add(1, Ordering::SeqCst);
        let order = Order {
            id,
            symbol: symbol.to_string(),
            quantity,
            price,
            status: OrderStatus::Pending,
        };

        let mut orders = self.orders.lock().unwrap();
        orders.insert(id, order);
        println!("[Bot] Ордер #{} создан: {} x {} @ ${:.2}", id, symbol, quantity, price);

        Some(id)
    }

    /// Отменить все активные ордера
    fn cancel_all_orders(&self) -> Vec<u64> {
        let mut orders = self.orders.lock().unwrap();
        let mut cancelled = Vec::new();

        for (id, order) in orders.iter_mut() {
            if matches!(order.status, OrderStatus::Pending) {
                order.status = OrderStatus::Cancelled;
                cancelled.push(*id);
                println!("[Bot] Ордер #{} отменён", id);
            }
        }

        cancelled
    }

    /// Закрыть все позиции по рынку
    fn close_all_positions(&self) {
        let mut positions = self.positions.lock().unwrap();
        let mut cash = self.cash_balance.lock().unwrap();

        for (symbol, position) in positions.drain() {
            // Имитация закрытия позиции
            let market_price = position.avg_price * 1.001; // Небольшой slippage
            let revenue = position.quantity * market_price;
            *cash += revenue;
            println!(
                "[Bot] Позиция {} закрыта: {} x ${:.2} = ${:.2}",
                symbol, position.quantity, market_price, revenue
            );
        }
    }

    /// Получить количество активных ордеров
    fn pending_orders_count(&self) -> usize {
        let orders = self.orders.lock().unwrap();
        orders
            .values()
            .filter(|o| matches!(o.status, OrderStatus::Pending))
            .count()
    }

    /// Сохранить состояние (имитация)
    fn save_state(&self) {
        let orders = self.orders.lock().unwrap();
        let positions = self.positions.lock().unwrap();
        let cash = self.cash_balance.lock().unwrap();

        println!("[Bot] Сохраняю состояние:");
        println!("  - Ордеров: {}", orders.len());
        println!("  - Позиций: {}", positions.len());
        println!("  - Баланс: ${:.2}", *cash);
        // В реальности здесь запись в БД
    }

    /// Выполнить graceful shutdown
    fn graceful_shutdown(&self) {
        println!("\n=== Начинаю graceful shutdown ===\n");

        // Шаг 1: Прекращаем приём новых ордеров (уже сделано через shutdown_requested)
        println!("[Shutdown] Шаг 1: Приём новых ордеров прекращён");

        // Шаг 2: Отменяем все активные ордера
        println!("[Shutdown] Шаг 2: Отменяю активные ордера...");
        let cancelled = self.cancel_all_orders();
        println!("[Shutdown] Отменено {} ордеров", cancelled.len());

        // Имитация ожидания подтверждения отмены от биржи
        thread::sleep(Duration::from_millis(500));

        // Шаг 3: Закрываем все позиции
        println!("[Shutdown] Шаг 3: Закрываю позиции...");
        self.close_all_positions();

        // Шаг 4: Сохраняем состояние
        println!("[Shutdown] Шаг 4: Сохраняю состояние...");
        self.save_state();

        // Шаг 5: Останавливаем бота
        println!("[Shutdown] Шаг 5: Останавливаю все компоненты...");
        self.stop();

        println!("\n=== Graceful shutdown завершён ===\n");
    }
}

/// Имитация торговой стратегии
fn trading_strategy(bot: Arc<TradingBot>) {
    println!("[Strategy] Стратегия запущена");

    let mut tick = 0;
    while bot.is_running() {
        if !bot.is_shutdown_requested() {
            tick += 1;
            // Имитация торговых решений
            if tick % 3 == 0 {
                bot.place_order("BTC", 0.1, 42000.0 + tick as f64 * 10.0);
            }
        }
        thread::sleep(Duration::from_millis(300));
    }

    println!("[Strategy] Стратегия остановлена");
}

/// Имитация сборщика рыночных данных
fn market_data_handler(bot: Arc<TradingBot>) {
    println!("[MarketData] Обработчик данных запущен");

    while bot.is_running() {
        if !bot.is_shutdown_requested() {
            // Имитация получения котировок
            println!("[MarketData] BTC: ${:.2}", 42000.0);
        }
        thread::sleep(Duration::from_millis(500));
    }

    println!("[MarketData] Обработчик данных остановлен");
}

fn main() {
    let bot = TradingBot::new(100_000.0);

    // Добавляем начальную позицию для демонстрации
    {
        let mut positions = bot.positions.lock().unwrap();
        positions.insert("ETH".to_string(), Position {
            symbol: "ETH".to_string(),
            quantity: 10.0,
            avg_price: 2500.0,
        });
        let mut cash = bot.cash_balance.lock().unwrap();
        *cash -= 25_000.0; // Уменьшаем на стоимость позиции
    }

    // Запускаем компоненты
    let bot_clone1 = Arc::clone(&bot);
    let strategy = thread::spawn(move || trading_strategy(bot_clone1));

    let bot_clone2 = Arc::clone(&bot);
    let market_data = thread::spawn(move || market_data_handler(bot_clone2));

    // Имитируем работу
    thread::sleep(Duration::from_secs(2));

    // Инициируем graceful shutdown
    bot.request_shutdown();
    bot.graceful_shutdown();

    // Ждём завершения всех потоков
    strategy.join().unwrap();
    market_data.join().unwrap();

    println!("Все компоненты бота успешно остановлены!");
}
```

## Таймаут для Graceful Shutdown

Важно ограничить время на graceful shutdown, чтобы система не зависла:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct ShutdownCoordinator {
    shutdown_requested: Arc<AtomicBool>,
    timeout: Duration,
}

impl ShutdownCoordinator {
    fn new(timeout: Duration) -> Self {
        ShutdownCoordinator {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            timeout,
        }
    }

    fn token(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown_requested)
    }

    fn initiate_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    fn wait_for_workers(&self, handles: Vec<thread::JoinHandle<()>>) -> bool {
        let start = Instant::now();

        for handle in handles {
            let remaining = self.timeout.saturating_sub(start.elapsed());

            if remaining.is_zero() {
                println!("ВНИМАНИЕ: Таймаут graceful shutdown истёк!");
                println!("Некоторые потоки будут принудительно завершены.");
                return false;
            }

            // В реальности нужен join с таймаутом, который std не поддерживает напрямую
            // Используем простой подход: поток должен сам проверять токен и завершаться
            match handle.join() {
                Ok(_) => println!("Поток успешно завершён"),
                Err(_) => println!("Ошибка при завершении потока"),
            }
        }

        true
    }
}

fn worker(id: usize, shutdown: Arc<AtomicBool>) {
    println!("Worker {} запущен", id);

    while !shutdown.load(Ordering::SeqCst) {
        // Имитация работы
        thread::sleep(Duration::from_millis(100));
    }

    // Имитация cleanup (может занять время)
    println!("Worker {}: выполняю cleanup...", id);
    thread::sleep(Duration::from_millis(300));

    println!("Worker {} остановлен", id);
}

fn main() {
    let coordinator = ShutdownCoordinator::new(Duration::from_secs(5));

    // Запускаем воркеров
    let mut handles = Vec::new();
    for i in 0..3 {
        let token = coordinator.token();
        handles.push(thread::spawn(move || worker(i, token)));
    }

    // Работаем
    thread::sleep(Duration::from_secs(1));

    // Начинаем shutdown
    println!("\n=== Начинаю graceful shutdown с таймаутом 5 секунд ===\n");
    coordinator.initiate_shutdown();

    if coordinator.wait_for_workers(handles) {
        println!("\nВсе воркеры завершились в рамках таймаута");
    } else {
        println!("\nНекоторые воркеры не успели завершиться");
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Graceful Shutdown | Контролируемое завершение с корректным освобождением ресурсов |
| Cancellation Token | Атомарный флаг для координации отмены между потоками |
| SIGINT/SIGTERM | Unix-сигналы для запроса завершения программы |
| Каналы для shutdown | Использование `mpsc` для координации завершения |
| Таймаут shutdown | Ограничение времени на graceful shutdown |
| Фазы shutdown | Прекращение приёма → отмена операций → сохранение → остановка |

## Домашнее задание

1. **Простой shutdown**: Создай программу с тремя потоками, которая:
   - Корректно обрабатывает Ctrl+C
   - Отправляет всем потокам сигнал на остановку
   - Ждёт завершения каждого потока
   - Выводит статистику работы каждого потока

2. **Биржевой шлюз**: Реализуй структуру `ExchangeGateway` с методами:
   - `connect()` — подключение к бирже
   - `submit_order(order)` — отправка ордера
   - `shutdown()` — graceful shutdown, который:
     - Отменяет все активные ордера
     - Ждёт подтверждения отмены (с таймаутом)
     - Закрывает соединение

3. **Координатор shutdown**: Создай `ShutdownCoordinator`, который:
   - Регистрирует несколько компонентов
   - При shutdown вызывает их в определённом порядке (обратном порядке регистрации)
   - Логирует время завершения каждого компонента
   - Имеет общий таймаут на весь процесс

4. **Checkpoint система**: Реализуй систему, которая:
   - Периодически сохраняет состояние торгового бота
   - При shutdown сохраняет финальный checkpoint
   - При запуске восстанавливает состояние из последнего checkpoint

## Навигация

[← Предыдущий день](../175-thread-pool-limiting-parallelism/ru.md) | [Следующий день →](../177-pattern-producer-consumer/ru.md)
