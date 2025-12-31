# День 338: Graceful Shutdown: корректное завершение

## Аналогия из трейдинга

Представь, что ты управляешь торговой фирмой и наступает конец рабочего дня. У тебя есть два способа закрыть офис:

**Аварийное завершение (Hard Shutdown):**
Ты просто выключаешь свет и уходишь. Трейдеры бросают незавершённые ордера, открытые позиции остаются без присмотра, данные не сохранены. На следующий день ты придёшь к хаосу: потерянные сделки, несинхронизированные позиции, возможные убытки.

**Корректное завершение (Graceful Shutdown):**
Ты объявляешь, что офис закрывается через 15 минут. Трейдеры:
1. Прекращают принимать новые ордера
2. Завершают обработку текущих ордеров
3. Закрывают или хеджируют открытые позиции
4. Сохраняют все данные о сделках
5. Отключаются от бирж
6. Выключают системы в правильном порядке

| Критерий | Hard Shutdown | Graceful Shutdown |
|----------|---------------|-------------------|
| **Аналогия** | Выдернуть вилку | Объявить закрытие |
| **Данные** | Могут потеряться | Сохранены |
| **Ордера** | Брошены | Завершены/отменены |
| **Соединения** | Оборваны | Закрыты корректно |
| **Состояние** | Неизвестно | Консистентно |

## Почему это важно для торговых систем?

Торговые системы особенно чувствительны к некорректному завершению:

1. **Открытые позиции** — незакрытая позиция может привести к убыткам
2. **Незавершённые ордера** — могут исполниться или зависнуть
3. **Потеря данных** — история сделок критически важна
4. **Состояние соединений** — биржи могут заблокировать при резком отключении
5. **Целостность базы данных** — незавершённые транзакции

## Базовый паттерн Graceful Shutdown

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::thread;

/// Флаг для сигнала о завершении
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Торговый воркер
struct TradingWorker {
    name: String,
    processing: AtomicBool,
}

impl TradingWorker {
    fn new(name: &str) -> Self {
        TradingWorker {
            name: name.to_string(),
            processing: AtomicBool::new(false),
        }
    }

    /// Обработка ордеров с проверкой флага завершения
    fn process_orders(&self) {
        println!("[{}] Начинаю обработку ордеров", self.name);

        for i in 1..=10 {
            // Проверяем флаг завершения перед каждой операцией
            if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                println!("[{}] Получен сигнал завершения, прекращаю приём новых ордеров", self.name);
                break;
            }

            self.processing.store(true, Ordering::SeqCst);
            println!("[{}] Обрабатываю ордер #{}", self.name, i);

            // Имитация обработки ордера
            thread::sleep(Duration::from_millis(100));

            self.processing.store(false, Ordering::SeqCst);
        }

        println!("[{}] Обработка завершена", self.name);
    }

    /// Проверка, есть ли активная работа
    fn is_processing(&self) -> bool {
        self.processing.load(Ordering::SeqCst)
    }
}

fn main() {
    println!("=== Базовый Graceful Shutdown ===\n");

    let worker = Arc::new(TradingWorker::new("OrderProcessor"));
    let worker_clone = Arc::clone(&worker);

    // Запускаем воркер в отдельном потоке
    let handle = thread::spawn(move || {
        worker_clone.process_orders();
    });

    // Имитация работы системы
    thread::sleep(Duration::from_millis(350));

    // Инициируем graceful shutdown
    println!("\n>>> Инициирую graceful shutdown...\n");
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

    // Ждём завершения воркера
    handle.join().unwrap();

    println!("\n>>> Система корректно завершена");
}
```

## Обработка сигналов Unix (SIGTERM, SIGINT)

В продакшене важно обрабатывать системные сигналы:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

/// Менеджер graceful shutdown
struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
    is_shutting_down: AtomicBool,
}

impl ShutdownManager {
    fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        ShutdownManager {
            shutdown_tx,
            is_shutting_down: AtomicBool::new(false),
        }
    }

    /// Получить подписку на событие shutdown
    fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Инициировать shutdown
    fn initiate(&self) {
        if !self.is_shutting_down.swap(true, Ordering::SeqCst) {
            println!("[ShutdownManager] Инициирую graceful shutdown...");
            let _ = self.shutdown_tx.send(());
        }
    }

    fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::SeqCst)
    }
}

/// Торговый сервис
struct TradingService {
    name: String,
    shutdown_rx: broadcast::Receiver<()>,
}

impl TradingService {
    fn new(name: &str, shutdown_manager: &ShutdownManager) -> Self {
        TradingService {
            name: name.to_string(),
            shutdown_rx: shutdown_manager.subscribe(),
        }
    }

    async fn run(&mut self) {
        println!("[{}] Сервис запущен", self.name);

        loop {
            tokio::select! {
                // Основная работа
                _ = self.do_work() => {
                    // Работа выполнена, продолжаем
                }
                // Получен сигнал shutdown
                _ = self.shutdown_rx.recv() => {
                    println!("[{}] Получен сигнал shutdown", self.name);
                    break;
                }
            }
        }

        // Выполняем cleanup
        self.cleanup().await;
        println!("[{}] Сервис завершён", self.name);
    }

    async fn do_work(&self) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Имитация обработки рыночных данных
    }

    async fn cleanup(&self) {
        println!("[{}] Выполняю cleanup...", self.name);
        // Закрываем соединения, сохраняем данные и т.д.
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() {
    println!("=== Обработка сигналов Unix ===\n");

    let shutdown_manager = Arc::new(ShutdownManager::new());

    // Запускаем обработчик сигналов
    let shutdown_for_signals = Arc::clone(&shutdown_manager);
    tokio::spawn(async move {
        // Ждём SIGTERM или SIGINT (Ctrl+C)
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\n>>> Получен SIGINT (Ctrl+C)");
            }
            // Для Unix систем можно добавить SIGTERM:
            // _ = async {
            //     let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
            //     sigterm.recv().await
            // } => {
            //     println!("\n>>> Получен SIGTERM");
            // }
        }
        shutdown_for_signals.initiate();
    });

    // Создаём торговые сервисы
    let mut market_data = TradingService::new("MarketData", &shutdown_manager);
    let mut order_executor = TradingService::new("OrderExecutor", &shutdown_manager);

    // Запускаем сервисы
    let md_handle = tokio::spawn(async move {
        market_data.run().await;
    });

    let oe_handle = tokio::spawn(async move {
        order_executor.run().await;
    });

    // Симуляция работы системы (в реальности здесь был бы долгий процесс)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Инициируем shutdown программно (для демонстрации)
    shutdown_manager.initiate();

    // Ждём завершения всех сервисов с таймаутом
    let shutdown_timeout = Duration::from_secs(10);
    match timeout(shutdown_timeout, async {
        let _ = md_handle.await;
        let _ = oe_handle.await;
    }).await {
        Ok(_) => println!("\n>>> Все сервисы корректно завершены"),
        Err(_) => println!("\n>>> Таймаут! Принудительное завершение"),
    }
}
```

## Многоуровневый Shutdown для торговой системы

```rust
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{Duration, timeout};
use std::collections::HashMap;

/// Стадии shutdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownPhase {
    Running,           // Система работает
    StopNewOrders,     // Прекращаем приём новых ордеров
    CancelPending,     // Отменяем pending ордера
    ClosePositions,    // Закрываем открытые позиции
    SaveState,         // Сохраняем состояние
    Disconnect,        // Отключаемся от бирж
    Complete,          // Завершение
}

/// Ордер в системе
#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

/// Позиция
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

/// Торговый движок с graceful shutdown
struct TradingEngine {
    phase: Arc<RwLock<ShutdownPhase>>,
    pending_orders: Arc<RwLock<Vec<Order>>>,
    open_positions: Arc<RwLock<HashMap<String, Position>>>,
    shutdown_tx: broadcast::Sender<ShutdownPhase>,
}

impl TradingEngine {
    fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        // Создаём тестовые данные
        let mut positions = HashMap::new();
        positions.insert("BTCUSDT".to_string(), Position {
            symbol: "BTCUSDT".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
        });
        positions.insert("ETHUSDT".to_string(), Position {
            symbol: "ETHUSDT".to_string(),
            quantity: 5.0,
            entry_price: 3000.0,
        });

        let pending_orders = vec![
            Order {
                id: "ORD-001".to_string(),
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                quantity: 0.1,
                price: 49000.0,
                status: "PENDING".to_string(),
            },
            Order {
                id: "ORD-002".to_string(),
                symbol: "ETHUSDT".to_string(),
                side: "SELL".to_string(),
                quantity: 1.0,
                price: 3100.0,
                status: "PENDING".to_string(),
            },
        ];

        TradingEngine {
            phase: Arc::new(RwLock::new(ShutdownPhase::Running)),
            pending_orders: Arc::new(RwLock::new(pending_orders)),
            open_positions: Arc::new(RwLock::new(positions)),
            shutdown_tx,
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<ShutdownPhase> {
        self.shutdown_tx.subscribe()
    }

    async fn set_phase(&self, phase: ShutdownPhase) {
        *self.phase.write().await = phase;
        let _ = self.shutdown_tx.send(phase);
        println!("[TradingEngine] Переход в фазу: {:?}", phase);
    }

    async fn current_phase(&self) -> ShutdownPhase {
        *self.phase.read().await
    }

    /// Попытка разместить ордер
    async fn place_order(&self, order: Order) -> Result<(), String> {
        let phase = self.current_phase().await;

        if phase != ShutdownPhase::Running {
            return Err(format!(
                "Новые ордера не принимаются, система в фазе {:?}",
                phase
            ));
        }

        self.pending_orders.write().await.push(order.clone());
        println!("[TradingEngine] Ордер принят: {}", order.id);
        Ok(())
    }

    /// Отмена всех pending ордеров
    async fn cancel_all_pending(&self) -> Vec<String> {
        let mut orders = self.pending_orders.write().await;
        let cancelled: Vec<String> = orders.iter().map(|o| o.id.clone()).collect();

        for order in orders.iter_mut() {
            order.status = "CANCELLED".to_string();
            println!("[TradingEngine] Ордер отменён: {}", order.id);
        }

        orders.clear();
        cancelled
    }

    /// Закрытие всех позиций
    async fn close_all_positions(&self) -> Vec<(String, f64)> {
        let mut positions = self.open_positions.write().await;
        let mut closed = Vec::new();

        for (symbol, position) in positions.iter() {
            // Имитация закрытия по рыночной цене
            let market_price = position.entry_price * 1.01; // Примерная цена
            let pnl = (market_price - position.entry_price) * position.quantity;

            println!(
                "[TradingEngine] Закрываю позицию {}: {} @ ${:.2}, P&L: ${:.2}",
                symbol, position.quantity, market_price, pnl
            );

            closed.push((symbol.clone(), pnl));
        }

        positions.clear();
        closed
    }

    /// Сохранение состояния
    async fn save_state(&self) {
        println!("[TradingEngine] Сохраняю состояние...");

        // Имитация сохранения в файл/БД
        tokio::time::sleep(Duration::from_millis(100)).await;

        let orders = self.pending_orders.read().await;
        let positions = self.open_positions.read().await;

        println!(
            "[TradingEngine] Состояние сохранено: {} ордеров, {} позиций",
            orders.len(),
            positions.len()
        );
    }

    /// Полный graceful shutdown
    async fn graceful_shutdown(&self, shutdown_timeout: Duration) -> Result<(), String> {
        println!("\n=== Начинаю Graceful Shutdown ===\n");

        // Фаза 1: Прекращаем приём новых ордеров
        self.set_phase(ShutdownPhase::StopNewOrders).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Фаза 2: Отменяем pending ордера
        self.set_phase(ShutdownPhase::CancelPending).await;
        let cancelled = self.cancel_all_pending().await;
        println!("Отменено ордеров: {}", cancelled.len());

        // Фаза 3: Закрываем позиции
        self.set_phase(ShutdownPhase::ClosePositions).await;
        let closed = self.close_all_positions().await;
        let total_pnl: f64 = closed.iter().map(|(_, pnl)| pnl).sum();
        println!("Закрыто позиций: {}, Общий P&L: ${:.2}", closed.len(), total_pnl);

        // Фаза 4: Сохраняем состояние
        self.set_phase(ShutdownPhase::SaveState).await;
        self.save_state().await;

        // Фаза 5: Отключение
        self.set_phase(ShutdownPhase::Disconnect).await;
        println!("[TradingEngine] Отключаюсь от бирж...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Завершение
        self.set_phase(ShutdownPhase::Complete).await;

        println!("\n=== Graceful Shutdown завершён ===");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("=== Многоуровневый Shutdown торговой системы ===\n");

    let engine = Arc::new(TradingEngine::new());
    let engine_clone = Arc::clone(&engine);

    // Подписчик на изменения фазы
    let mut phase_rx = engine.subscribe();
    tokio::spawn(async move {
        while let Ok(phase) = phase_rx.recv().await {
            println!("[Monitor] Текущая фаза: {:?}", phase);
        }
    });

    // Попытка размещения ордеров
    println!("--- Попытка размещения ордеров ---");

    let order = Order {
        id: "ORD-003".to_string(),
        symbol: "SOLUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 10.0,
        price: 100.0,
        status: "NEW".to_string(),
    };

    match engine.place_order(order).await {
        Ok(_) => println!("Ордер успешно размещён"),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Инициируем shutdown
    tokio::time::sleep(Duration::from_millis(200)).await;

    let shutdown_result = engine_clone
        .graceful_shutdown(Duration::from_secs(30))
        .await;

    match shutdown_result {
        Ok(_) => println!("\nСистема корректно завершена"),
        Err(e) => println!("\nОшибка shutdown: {}", e),
    }

    // Попытка размещения после shutdown
    println!("\n--- Попытка размещения после shutdown ---");
    let order = Order {
        id: "ORD-004".to_string(),
        symbol: "SOLUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 10.0,
        price: 100.0,
        status: "NEW".to_string(),
    };

    match engine.place_order(order).await {
        Ok(_) => println!("Ордер успешно размещён"),
        Err(e) => println!("Ожидаемая ошибка: {}", e),
    }
}
```

## Tokio CancellationToken

Tokio предоставляет удобный примитив для graceful shutdown:

```rust
use tokio_util::sync::CancellationToken;
use tokio::time::{Duration, interval};
use std::sync::Arc;

/// Сервис получения рыночных данных
async fn market_data_service(token: CancellationToken, symbol: String) {
    println!("[MarketData:{}] Запущен", symbol);

    let mut price = 50000.0;
    let mut tick_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[MarketData:{}] Получен сигнал отмены, завершаюсь...", symbol);
                break;
            }
            _ = tick_interval.tick() => {
                // Имитация обновления цены
                price += (rand_simple() - 0.5) * 100.0;
                // Здесь была бы обработка обновления
            }
        }
    }

    // Cleanup
    println!("[MarketData:{}] Cleanup выполнен", symbol);
}

/// Сервис исполнения ордеров
async fn order_executor_service(token: CancellationToken) {
    println!("[OrderExecutor] Запущен");

    let mut order_count = 0;

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[OrderExecutor] Получен сигнал отмены");
                println!("[OrderExecutor] Завершаю обработку {} ордеров...", order_count);
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                order_count += 1;
                // Имитация обработки ордера
            }
        }
    }

    // Финализация
    println!("[OrderExecutor] Все ордера обработаны");
}

/// Псевдо-рандом для демонстрации
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    println!("=== Tokio CancellationToken ===\n");

    // Создаём родительский токен
    let parent_token = CancellationToken::new();

    // Создаём дочерние токены для разных сервисов
    let md_btc_token = parent_token.child_token();
    let md_eth_token = parent_token.child_token();
    let executor_token = parent_token.child_token();

    // Запускаем сервисы
    let md_btc = tokio::spawn(market_data_service(md_btc_token, "BTCUSDT".to_string()));
    let md_eth = tokio::spawn(market_data_service(md_eth_token, "ETHUSDT".to_string()));
    let executor = tokio::spawn(order_executor_service(executor_token));

    // Работаем некоторое время
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Инициируем shutdown
    println!("\n>>> Инициирую shutdown через CancellationToken\n");
    parent_token.cancel();

    // Ждём завершения всех сервисов
    let _ = tokio::join!(md_btc, md_eth, executor);

    println!("\n>>> Все сервисы завершены");
}
```

## Shutdown с таймаутами и принудительным завершением

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, timeout};

/// Задача, которая может зависнуть
async fn potentially_slow_task(id: u32, slow: bool) -> Result<u32, String> {
    println!("[Task-{}] Начинаю выполнение", id);

    if slow {
        // Имитация зависшей задачи
        tokio::time::sleep(Duration::from_secs(60)).await;
    } else {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("[Task-{}] Выполнена", id);
    Ok(id)
}

/// Менеджер задач с graceful shutdown
struct TaskManager {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl TaskManager {
    fn new(max_concurrent: usize) -> Self {
        TaskManager {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    async fn shutdown_with_timeout(
        &self,
        tasks: Vec<tokio::task::JoinHandle<Result<u32, String>>>,
        graceful_timeout: Duration,
        force_timeout: Duration,
    ) {
        println!("\n=== Начинаю shutdown ===");
        println!("Graceful timeout: {:?}", graceful_timeout);
        println!("Force timeout: {:?}", force_timeout);

        // Фаза 1: Graceful shutdown
        println!("\n[Фаза 1] Ожидаю завершения задач...");

        match timeout(graceful_timeout, async {
            for (i, task) in tasks.into_iter().enumerate() {
                match task.await {
                    Ok(Ok(id)) => println!("  Задача {} завершена успешно", id),
                    Ok(Err(e)) => println!("  Задача {} завершилась с ошибкой: {}", i, e),
                    Err(_) => println!("  Задача {} была отменена", i),
                }
            }
        }).await {
            Ok(_) => {
                println!("\n[Результат] Все задачи завершены корректно");
                return;
            }
            Err(_) => {
                println!("\n[Фаза 1] Таймаут! Переход к принудительному завершению");
            }
        }

        // Фаза 2: Force shutdown
        println!("\n[Фаза 2] Принудительное завершение...");

        // В реальности здесь были бы дополнительные действия:
        // - Отмена задач
        // - Сохранение состояния
        // - Логирование незавершённых операций

        tokio::time::sleep(force_timeout).await;
        println!("[Фаза 2] Принудительное завершение выполнено");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Shutdown с таймаутами ===\n");

    let manager = TaskManager::new(5);

    // Создаём задачи (одна будет "зависать")
    let mut tasks = Vec::new();

    for i in 0..5 {
        let slow = i == 2; // Задача 2 будет медленной
        let handle = tokio::spawn(potentially_slow_task(i, slow));
        tasks.push(handle);
    }

    // Даём время начать выполнение
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Запускаем shutdown
    manager.shutdown_with_timeout(
        tasks,
        Duration::from_millis(500),  // Graceful timeout
        Duration::from_millis(100),  // Force timeout
    ).await;

    println!("\n>>> Shutdown завершён");
}
```

## Паттерн: Shutdown Hook для торговой системы

```rust
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::VecDeque;

/// Тип функции-хука
type ShutdownHook = Box<dyn Fn() + Send + Sync>;

/// Приоритет хука
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HookPriority {
    Critical = 0,   // Первыми выполняются (сохранение данных)
    High = 1,       // Закрытие позиций
    Normal = 2,     // Отмена ордеров
    Low = 3,        // Отключение от бирж
    Cleanup = 4,    // Очистка ресурсов
}

/// Зарегистрированный хук
struct RegisteredHook {
    name: String,
    priority: HookPriority,
    hook: ShutdownHook,
}

/// Менеджер shutdown хуков
struct ShutdownHookManager {
    hooks: RwLock<Vec<RegisteredHook>>,
}

impl ShutdownHookManager {
    fn new() -> Self {
        ShutdownHookManager {
            hooks: RwLock::new(Vec::new()),
        }
    }

    /// Регистрация хука
    async fn register<F>(&self, name: &str, priority: HookPriority, hook: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let registered = RegisteredHook {
            name: name.to_string(),
            priority,
            hook: Box::new(hook),
        };

        let mut hooks = self.hooks.write().await;
        hooks.push(registered);

        // Сортируем по приоритету
        hooks.sort_by(|a, b| a.priority.cmp(&b.priority));

        println!("[HookManager] Зарегистрирован хук '{}' с приоритетом {:?}", name, priority);
    }

    /// Выполнение всех хуков
    async fn run_all(&self) {
        let hooks = self.hooks.read().await;

        println!("\n[HookManager] Выполняю {} хуков...\n", hooks.len());

        for hook in hooks.iter() {
            println!("[HookManager] Выполняю '{}' ({:?})...", hook.name, hook.priority);
            (hook.hook)();
        }

        println!("\n[HookManager] Все хуки выполнены");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Shutdown Hooks для торговой системы ===\n");

    let hook_manager = Arc::new(ShutdownHookManager::new());

    // Регистрируем хуки в разном порядке
    hook_manager.register(
        "disconnect_exchanges",
        HookPriority::Low,
        || println!("  -> Отключаюсь от бирж")
    ).await;

    hook_manager.register(
        "save_state",
        HookPriority::Critical,
        || println!("  -> Сохраняю критическое состояние")
    ).await;

    hook_manager.register(
        "close_positions",
        HookPriority::High,
        || println!("  -> Закрываю открытые позиции")
    ).await;

    hook_manager.register(
        "cancel_orders",
        HookPriority::Normal,
        || println!("  -> Отменяю pending ордера")
    ).await;

    hook_manager.register(
        "cleanup_temp_files",
        HookPriority::Cleanup,
        || println!("  -> Очищаю временные файлы")
    ).await;

    // Симуляция работы
    println!("Система работает...");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Выполняем shutdown
    println!("\n>>> Инициирую shutdown");
    hook_manager.run_all().await;

    println!("\n>>> Система завершена");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Graceful Shutdown** | Корректное завершение с сохранением состояния |
| **Hard Shutdown** | Немедленное завершение без очистки |
| **Shutdown фазы** | Последовательные этапы завершения |
| **CancellationToken** | Примитив Tokio для отмены задач |
| **Shutdown hooks** | Функции, вызываемые при завершении |
| **Shutdown timeout** | Ограничение времени на graceful shutdown |
| **Signal handling** | Обработка SIGTERM, SIGINT |

## Практические задания

1. **Graceful Shutdown для WebSocket клиента**: Создай систему, которая:
   - Подключается к нескольким биржам по WebSocket
   - При shutdown корректно закрывает все соединения
   - Сохраняет последние полученные данные
   - Логирует все этапы отключения

2. **Order Manager с shutdown**: Реализуй менеджер ордеров:
   - Отслеживает все активные ордера
   - При shutdown отменяет pending ордера
   - Ждёт подтверждения отмены с таймаутом
   - Логирует неотменённые ордера

3. **Position Closer**: Создай сервис:
   - Отслеживает открытые позиции
   - При shutdown закрывает позиции по рынку
   - Рассчитывает финальный P&L
   - Генерирует отчёт о закрытии

4. **State Persister**: Реализуй систему сохранения:
   - Периодически сохраняет состояние
   - При shutdown делает финальное сохранение
   - Проверяет целостность данных
   - Поддерживает восстановление после аварии

## Домашнее задание

1. **Полная торговая система с graceful shutdown**: Разработай систему:
   - Market data сервис с подписками
   - Order execution с очередью ордеров
   - Position tracking с расчётом P&L
   - Risk manager с лимитами
   - Многоуровневый shutdown со всеми фазами
   - Обработка всех системных сигналов
   - Таймауты на каждой фазе
   - Логирование всех операций

2. **Fault-tolerant shutdown**: Реализуй shutdown, который:
   - Продолжает работу при ошибках в отдельных компонентах
   - Имеет fallback для критических операций
   - Сохраняет информацию об ошибках
   - Позволяет частичное восстановление
   - Отправляет оповещения при проблемах

3. **Distributed shutdown coordinator**: Создай координатор:
   - Управляет shutdown нескольких сервисов
   - Учитывает зависимости между сервисами
   - Координирует порядок завершения
   - Обрабатывает недоступные сервисы
   - Генерирует общий отчёт

4. **Shutdown testing framework**: Разработай фреймворк:
   - Симулирует различные сценарии shutdown
   - Тестирует все пути выполнения
   - Проверяет сохранность данных
   - Измеряет время каждой фазы
   - Генерирует отчёты о покрытии

## Навигация

[← Предыдущий день](../337-*/ru.md) | [Следующий день →](../339-*/ru.md)
