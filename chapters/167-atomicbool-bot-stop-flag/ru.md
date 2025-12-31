# День 167: AtomicBool: флаг остановки бота

## Аналогия из трейдинга

Представь, что у тебя работает торговый бот, который каждую секунду проверяет рынок и совершает сделки. Ты хочешь иметь возможность безопасно остановить его в любой момент — например, при резком падении рынка, достижении лимита убытков или просто в конце торговой сессии.

Как это реализовать? Нужен **флаг остановки** — переменная, которую один поток (главный) может установить в `true`, а другой поток (бот) постоянно проверяет. Если флаг стал `true`, бот завершает работу.

Проблема в том, что обычная переменная `bool` небезопасна для многопоточного доступа:
- Один поток может прочитать старое значение, пока другой его меняет
- Компилятор может закэшировать значение в регистре
- CPU может переупорядочить операции

**AtomicBool** решает эти проблемы — это атомарный (неделимый) булев тип, безопасный для использования из нескольких потоков без мьютексов.

## Что такое AtomicBool?

`AtomicBool` — это примитив синхронизации из модуля `std::sync::atomic`. Он гарантирует:

1. **Атомарность** — операции чтения и записи неделимы
2. **Видимость** — изменения сразу видны всем потокам
3. **Отсутствие гонок данных** — безопасен без блокировок

```rust
use std::sync::atomic::{AtomicBool, Ordering};

// Создание AtomicBool
let flag = AtomicBool::new(false);

// Чтение значения
let value = flag.load(Ordering::Relaxed);

// Запись значения
flag.store(true, Ordering::Relaxed);
```

## Ordering: порядок операций с памятью

При работе с атомарными типами нужно указывать `Ordering` — это говорит компилятору и CPU, какие гарантии упорядочивания нужны:

| Ordering | Описание | Когда использовать |
|----------|----------|-------------------|
| `Relaxed` | Нет гарантий упорядочивания | Простые счётчики, флаги без зависимостей |
| `Acquire` | Все записи до Release видны | Чтение данных, защищённых флагом |
| `Release` | Записи видны после Acquire | Запись данных, защищённых флагом |
| `SeqCst` | Полный последовательный порядок | Когда нужна максимальная строгость |

Для простого флага остановки обычно достаточно `Relaxed`.

## Простой пример: флаг остановки

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Флаг остановки, разделяемый между потоками
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Клон для рабочего потока
    let worker_flag = Arc::clone(&stop_flag);

    // Рабочий поток (торговый бот)
    let worker = thread::spawn(move || {
        let mut iteration = 0;

        // Работаем пока флаг не установлен
        while !worker_flag.load(Ordering::Relaxed) {
            iteration += 1;
            println!("Бот: итерация {}, проверяю рынок...", iteration);

            // Имитация работы
            thread::sleep(Duration::from_millis(200));
        }

        println!("Бот: получен сигнал остановки, завершаю работу");
        iteration
    });

    // Главный поток ждёт немного, потом останавливает бота
    thread::sleep(Duration::from_secs(1));
    println!("Главный: отправляю сигнал остановки");
    stop_flag.store(true, Ordering::Relaxed);

    // Ждём завершения бота
    let iterations = worker.join().unwrap();
    println!("Бот выполнил {} итераций", iterations);
}
```

## Торговый бот с флагом остановки

Рассмотрим более реалистичный пример:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct TradingBot {
    stop_flag: Arc<AtomicBool>,
    symbol: String,
    max_trades: u32,
    trades_executed: u32,
    total_pnl: f64,
}

impl TradingBot {
    fn new(stop_flag: Arc<AtomicBool>, symbol: &str, max_trades: u32) -> Self {
        TradingBot {
            stop_flag,
            symbol: symbol.to_string(),
            max_trades,
            trades_executed: 0,
            total_pnl: 0.0,
        }
    }

    fn run(&mut self) {
        println!("[{}] Бот запущен", self.symbol);

        while !self.should_stop() {
            // Проверяем рыночные условия (имитация)
            let price = self.get_market_price();

            // Принимаем решение о сделке
            if self.should_trade(price) {
                self.execute_trade(price);
            }

            // Пауза между итерациями
            thread::sleep(Duration::from_millis(100));
        }

        println!("[{}] Бот остановлен. Сделок: {}, PnL: ${:.2}",
            self.symbol, self.trades_executed, self.total_pnl);
    }

    fn should_stop(&self) -> bool {
        // Останавливаемся если:
        // 1. Получен сигнал остановки
        // 2. Достигнут лимит сделок
        self.stop_flag.load(Ordering::Relaxed) ||
        self.trades_executed >= self.max_trades
    }

    fn get_market_price(&self) -> f64 {
        // Имитация получения цены
        42000.0 + (rand_simple() * 1000.0 - 500.0)
    }

    fn should_trade(&self, price: f64) -> bool {
        // Простая стратегия: торгуем с вероятностью 30%
        rand_simple() < 0.3
    }

    fn execute_trade(&mut self, price: f64) {
        self.trades_executed += 1;

        // Имитация результата сделки
        let pnl = (rand_simple() - 0.5) * 100.0;
        self.total_pnl += pnl;

        println!("[{}] Сделка #{} по цене ${:.2}, PnL: ${:.2}",
            self.symbol, self.trades_executed, price, pnl);
    }
}

// Простой генератор случайных чисел
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Запускаем несколько ботов для разных пар
    let mut handles = vec![];

    for symbol in ["BTC/USDT", "ETH/USDT", "SOL/USDT"] {
        let flag = Arc::clone(&stop_flag);
        let sym = symbol.to_string();

        let handle = thread::spawn(move || {
            let mut bot = TradingBot::new(flag, &sym, 10);
            bot.run();
        });

        handles.push(handle);
    }

    // Ждём 2 секунды и останавливаем всех ботов
    thread::sleep(Duration::from_secs(2));
    println!("\n=== Отправляю сигнал остановки всем ботам ===\n");
    stop_flag.store(true, Ordering::Relaxed);

    // Ждём завершения всех ботов
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nВсе боты остановлены");
}
```

## Методы AtomicBool

AtomicBool предоставляет несколько полезных методов:

### Базовые операции

```rust
use std::sync::atomic::{AtomicBool, Ordering};

let flag = AtomicBool::new(false);

// load — чтение значения
let value = flag.load(Ordering::Relaxed);

// store — запись значения
flag.store(true, Ordering::Relaxed);

// swap — записать новое значение и вернуть старое
let old = flag.swap(false, Ordering::Relaxed);
println!("Было: {}, стало: false", old);
```

### Условные операции

```rust
use std::sync::atomic::{AtomicBool, Ordering};

let flag = AtomicBool::new(false);

// compare_exchange — атомарное сравнение и обмен
// Если flag == false, записать true и вернуть Ok(false)
// Иначе вернуть Err(текущее значение)
match flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed) {
    Ok(old) => println!("Успешно изменено с {} на true", old),
    Err(current) => println!("Не изменено, текущее значение: {}", current),
}

// fetch_and — атомарный AND
let result = flag.fetch_and(true, Ordering::Relaxed);

// fetch_or — атомарный OR
let result = flag.fetch_or(false, Ordering::Relaxed);

// fetch_xor — атомарный XOR
let result = flag.fetch_xor(true, Ordering::Relaxed);

// fetch_nand — атомарный NAND
let result = flag.fetch_nand(true, Ordering::Relaxed);
```

## Практический пример: graceful shutdown

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct GracefulBot {
    running: Arc<AtomicBool>,
    shutdown_requested: Arc<AtomicBool>,
}

impl GracefulBot {
    fn new() -> Self {
        GracefulBot {
            running: Arc::new(AtomicBool::new(false)),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    fn start(&self) -> thread::JoinHandle<()> {
        let running = Arc::clone(&self.running);
        let shutdown = Arc::clone(&self.shutdown_requested);

        thread::spawn(move || {
            // Отмечаем, что бот запущен
            running.store(true, Ordering::Release);
            println!("Бот: запущен и готов к работе");

            let mut pending_orders = vec![];

            loop {
                // Проверяем запрос на остановку
                if shutdown.load(Ordering::Acquire) {
                    println!("Бот: получен запрос на остановку");

                    // Graceful shutdown: завершаем текущие операции
                    if !pending_orders.is_empty() {
                        println!("Бот: завершаю {} отложенных ордеров", pending_orders.len());
                        for order in pending_orders.drain(..) {
                            println!("  - Отменён ордер: {}", order);
                            thread::sleep(Duration::from_millis(50));
                        }
                    }

                    break;
                }

                // Имитация работы
                let order_id = rand_simple() as u64;
                if rand_simple() < 0.3 {
                    pending_orders.push(format!("ORD-{}", order_id));
                    println!("Бот: создан ордер ORD-{}", order_id);
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Отмечаем, что бот остановлен
            running.store(false, Ordering::Release);
            println!("Бот: полностью остановлен");
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    fn request_shutdown(&self) {
        println!("Контроллер: отправляю запрос на остановку");
        self.shutdown_requested.store(true, Ordering::Release);
    }
}

// Простой генератор случайных чисел
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    let bot = GracefulBot::new();

    let handle = bot.start();

    // Ждём пока бот запустится
    while !bot.is_running() {
        thread::sleep(Duration::from_millis(10));
    }

    println!("Контроллер: бот работает, жду 1 секунду...\n");
    thread::sleep(Duration::from_secs(1));

    // Запрашиваем остановку
    bot.request_shutdown();

    // Ждём завершения
    handle.join().unwrap();

    println!("\nКонтроллер: бот успешно остановлен");
}
```

## Пример: несколько флагов состояния

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingState {
    // Флаги состояния системы
    market_open: AtomicBool,
    trading_enabled: AtomicBool,
    risk_limit_hit: AtomicBool,
    maintenance_mode: AtomicBool,
}

impl TradingState {
    fn new() -> Self {
        TradingState {
            market_open: AtomicBool::new(true),
            trading_enabled: AtomicBool::new(true),
            risk_limit_hit: AtomicBool::new(false),
            maintenance_mode: AtomicBool::new(false),
        }
    }

    fn can_trade(&self) -> bool {
        self.market_open.load(Ordering::Relaxed) &&
        self.trading_enabled.load(Ordering::Relaxed) &&
        !self.risk_limit_hit.load(Ordering::Relaxed) &&
        !self.maintenance_mode.load(Ordering::Relaxed)
    }

    fn get_status(&self) -> String {
        format!(
            "Рынок: {}, Торговля: {}, Риск: {}, Обслуживание: {}",
            if self.market_open.load(Ordering::Relaxed) { "открыт" } else { "закрыт" },
            if self.trading_enabled.load(Ordering::Relaxed) { "вкл" } else { "выкл" },
            if self.risk_limit_hit.load(Ordering::Relaxed) { "лимит!" } else { "ок" },
            if self.maintenance_mode.load(Ordering::Relaxed) { "вкл" } else { "выкл" }
        )
    }
}

fn main() {
    let state = Arc::new(TradingState::new());

    // Поток-трейдер
    let trader_state = Arc::clone(&state);
    let trader = thread::spawn(move || {
        for i in 1..=10 {
            if trader_state.can_trade() {
                println!("Трейдер: выполняю сделку #{}", i);
            } else {
                println!("Трейдер: торговля недоступна - {}", trader_state.get_status());
            }
            thread::sleep(Duration::from_millis(200));
        }
    });

    // Поток-контроллер рисков
    let risk_state = Arc::clone(&state);
    let risk_manager = thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        println!("\nРиск-менеджер: достигнут лимит риска!");
        risk_state.risk_limit_hit.store(true, Ordering::Relaxed);

        thread::sleep(Duration::from_millis(800));
        println!("Риск-менеджер: лимит риска сброшен\n");
        risk_state.risk_limit_hit.store(false, Ordering::Relaxed);
    });

    trader.join().unwrap();
    risk_manager.join().unwrap();

    println!("\nИтоговое состояние: {}", state.get_status());
}
```

## AtomicBool vs Mutex<bool>

| Характеристика | AtomicBool | Mutex\<bool\> |
|----------------|------------|---------------|
| Блокировка | Нет (lock-free) | Да |
| Производительность | Очень высокая | Ниже |
| Сложность | Простой | Сложнее |
| Возможности | Только булево значение | Любые операции |
| Deadlock | Невозможен | Возможен |

Используй `AtomicBool` когда:
- Нужен простой флаг (вкл/выкл)
- Важна максимальная производительность
- Не нужны составные операции

Используй `Mutex<bool>` когда:
- Нужно защитить несколько связанных переменных
- Нужны составные операции (проверка + изменение нескольких значений)

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| AtomicBool | Атомарный булев тип для многопоточного доступа |
| Ordering | Гарантии упорядочивания операций с памятью |
| load/store | Базовые операции чтения и записи |
| swap | Атомарный обмен значений |
| compare_exchange | Условное атомарное изменение |
| Lock-free | Работа без блокировок (мьютексов) |

## Домашнее задание

1. **Торговый бот с несколькими режимами**: Реализуй бота с тремя состояниями (AtomicBool каждое):
   - `aggressive_mode` — агрессивная торговля
   - `safe_mode` — консервативная торговля
   - `paused` — приостановлен

   Бот должен проверять все флаги и менять поведение соответственно.

2. **Система мониторинга**: Создай структуру `HealthMonitor` с методами:
   - `set_healthy(service: &str, healthy: bool)` — установить статус сервиса
   - `is_all_healthy()` — проверить, все ли сервисы работают
   - `get_unhealthy_services()` — получить список неработающих сервисов

3. **Атомарный переключатель**: Используя `compare_exchange`, реализуй функцию `toggle_once`, которая:
   - Переключает флаг только если он в исходном состоянии
   - Возвращает `true` если переключение удалось, `false` если уже было переключено
   - Гарантирует, что только один поток успешно переключит флаг

4. **Rate limiter**: Реализуй структуру `RateLimiter` с AtomicBool для флага "слишком много запросов":
   - Поток-монитор считает запросы и устанавливает флаг при превышении лимита
   - Рабочие потоки проверяют флаг перед выполнением операций
   - Автоматический сброс флага через заданный интервал

## Навигация

[← Предыдущий день](../166-atomics-lock-free-counters/ru.md) | [Следующий день →](../168-atomic-ordering-memory-barriers/ru.md)
