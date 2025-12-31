# День 166: Atomics: атомарные операции

## Аналогия из трейдинга

Представь терминал на бирже, где тысячи трейдеров одновременно видят текущую цену актива. Каждую миллисекунду цена обновляется, и все должны видеть одно и то же значение. Если один трейдер видит цену 42000, а другой в тот же момент — 42100, это приведёт к хаосу и арбитражным дырам.

**Атомарные операции** — это как электронное табло цены на бирже. Когда цена меняется, она меняется мгновенно и полностью — невозможно увидеть "половину" нового значения. Либо старая цена, либо новая, никаких промежуточных состояний.

В реальном алготрейдинге атомарные операции используются для:
- Счётчиков исполненных ордеров
- Флагов остановки торговли
- Текущей рыночной цены
- Счётчиков подключённых клиентов к API

## Что такое Atomics?

Атомарные типы (Atomics) — это примитивы, которые гарантируют, что операции чтения и записи выполняются целиком и неделимо. В Rust они находятся в модуле `std::sync::atomic`.

### Преимущества перед Mutex:

| Характеристика | Mutex | Atomic |
|---------------|-------|--------|
| Блокировка | Да, ждём освобождения | Нет блокировки |
| Производительность | Медленнее | Очень быстро |
| Защита данных | Любые типы | Только примитивы |
| Deadlock | Возможен | Невозможен |
| Сложность | Простой API | Требует понимания Ordering |

### Основные атомарные типы:

```rust
use std::sync::atomic::{
    AtomicBool,    // Атомарный булев
    AtomicI32,     // Атомарное i32
    AtomicI64,     // Атомарное i64
    AtomicU32,     // Атомарное u32
    AtomicU64,     // Атомарное u64
    AtomicUsize,   // Атомарное usize
    AtomicPtr,     // Атомарный указатель
    Ordering,      // Порядок памяти
};
```

## Простой пример: Счётчик ордеров

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // Атомарный счётчик ордеров
    let order_counter = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];

    // 10 потоков, каждый создаёт 100 ордеров
    for thread_id in 0..10 {
        let counter = Arc::clone(&order_counter);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Атомарный инкремент — без блокировки!
                let order_id = counter.fetch_add(1, Ordering::SeqCst);

                // order_id — уникальный ID для каждого ордера
                if order_id % 100 == 0 {
                    println!("Поток {}: создан ордер #{}", thread_id, order_id);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Гарантированно получим 1000!
    let total = order_counter.load(Ordering::SeqCst);
    println!("Всего создано ордеров: {}", total);
}
```

## Ordering (Порядок памяти)

Самая сложная часть атомарных операций — понимание `Ordering`. Это указывает компилятору и процессору, как синхронизировать память между потоками.

### Виды Ordering:

```rust
use std::sync::atomic::Ordering;

// Relaxed — минимальные гарантии, максимальная производительность
// Используй для счётчиков, где порядок не важен
Ordering::Relaxed

// Acquire — все чтения после этой операции видят записи до Release
// Используй при чтении флага или указателя
Ordering::Acquire

// Release — все записи до этой операции видны после Acquire
// Используй при записи флага или указателя
Ordering::Release

// AcqRel — комбинация Acquire и Release
// Используй для read-modify-write операций
Ordering::AcqRel

// SeqCst — самые строгие гарантии, все потоки видят одинаковый порядок
// Используй когда не уверен — это самый безопасный вариант
Ordering::SeqCst
```

### Практическое правило:

```rust
// Если не уверен — используй SeqCst
let value = atomic.load(Ordering::SeqCst);
atomic.store(new_value, Ordering::SeqCst);

// Для простых счётчиков можно Relaxed
counter.fetch_add(1, Ordering::Relaxed);

// Для флагов остановки: Release при записи, Acquire при чтении
stop_flag.store(true, Ordering::Release);
if stop_flag.load(Ordering::Acquire) { /* ... */ }
```

## Флаг остановки торгового бота

Классический паттерн — использование `AtomicBool` для безопасной остановки потоков:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingBot {
    should_stop: AtomicBool,
}

impl TradingBot {
    fn new() -> Self {
        TradingBot {
            should_stop: AtomicBool::new(false),
        }
    }

    fn stop(&self) {
        println!("Получен сигнал остановки!");
        self.should_stop.store(true, Ordering::Release);
    }

    fn is_running(&self) -> bool {
        !self.should_stop.load(Ordering::Acquire)
    }

    fn run_strategy(&self, name: &str) {
        println!("Стратегия '{}' запущена", name);
        let mut tick = 0;

        while self.is_running() {
            tick += 1;

            // Имитация торговой логики
            if tick % 10 == 0 {
                println!("[{}] Тик #{}: анализ рынка...", name, tick);
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("Стратегия '{}' остановлена на тике #{}", name, tick);
    }
}

fn main() {
    let bot = Arc::new(TradingBot::new());

    let bot1 = Arc::clone(&bot);
    let bot2 = Arc::clone(&bot);

    // Запускаем две стратегии
    let strategy1 = thread::spawn(move || {
        bot1.run_strategy("Momentum");
    });

    let strategy2 = thread::spawn(move || {
        bot2.run_strategy("MeanReversion");
    });

    // Ждём 2 секунды и останавливаем
    thread::sleep(Duration::from_secs(2));
    bot.stop();

    strategy1.join().unwrap();
    strategy2.join().unwrap();

    println!("Все стратегии остановлены");
}
```

## Атомарная цена актива

Пример обновления и чтения цены несколькими потоками:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Храним цену как u64 (центы) для атомарных операций
// $42000.50 = 4200050 центов
struct AtomicPrice {
    // Цена в центах (для точности)
    price_cents: AtomicU64,
}

impl AtomicPrice {
    fn new(price: f64) -> Self {
        AtomicPrice {
            price_cents: AtomicU64::new((price * 100.0) as u64),
        }
    }

    fn get(&self) -> f64 {
        self.price_cents.load(Ordering::Acquire) as f64 / 100.0
    }

    fn set(&self, price: f64) {
        self.price_cents.store((price * 100.0) as u64, Ordering::Release);
    }

    // Атомарное обновление только если цена изменилась значительно
    fn update_if_significant(&self, new_price: f64, threshold_percent: f64) -> bool {
        let new_cents = (new_price * 100.0) as u64;

        loop {
            let current = self.price_cents.load(Ordering::Acquire);
            let current_price = current as f64 / 100.0;

            let change_percent = ((new_price - current_price) / current_price * 100.0).abs();

            if change_percent < threshold_percent {
                return false; // Изменение незначительное
            }

            // CAS (Compare-And-Swap) — атомарная проверка и замена
            match self.price_cents.compare_exchange(
                current,
                new_cents,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,  // Успешно обновили
                Err(_) => continue,    // Кто-то изменил, пробуем снова
            }
        }
    }
}

fn main() {
    let btc_price = Arc::new(AtomicPrice::new(42000.0));

    let price_reader = Arc::clone(&btc_price);
    let price_writer = Arc::clone(&btc_price);

    // Поток обновления цены (имитация биржи)
    let writer = thread::spawn(move || {
        let prices = [42100.0, 41950.0, 42300.0, 41800.0, 42500.0];

        for price in prices {
            price_writer.set(price);
            println!("[Биржа] Новая цена BTC: ${:.2}", price);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // Поток чтения цены (трейдер)
    let reader = thread::spawn(move || {
        for i in 0..10 {
            let price = price_reader.get();
            println!("[Трейдер] Тик #{}: BTC = ${:.2}", i + 1, price);
            thread::sleep(Duration::from_millis(300));
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();
}
```

## Compare-And-Swap (CAS)

CAS — фундаментальная атомарная операция. Она проверяет текущее значение и заменяет его на новое, только если текущее совпадает с ожидаемым.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

// Атомарное резервирование ликвидности для ордера
struct LiquidityPool {
    available: AtomicU64, // Доступная ликвидность в центах
}

impl LiquidityPool {
    fn new(amount: f64) -> Self {
        LiquidityPool {
            available: AtomicU64::new((amount * 100.0) as u64),
        }
    }

    fn get_available(&self) -> f64 {
        self.available.load(Ordering::Acquire) as f64 / 100.0
    }

    // Атомарное резервирование средств
    fn reserve(&self, amount: f64) -> Result<(), String> {
        let amount_cents = (amount * 100.0) as u64;

        loop {
            let current = self.available.load(Ordering::Acquire);

            if current < amount_cents {
                return Err(format!(
                    "Недостаточно ликвидности: нужно ${:.2}, доступно ${:.2}",
                    amount, current as f64 / 100.0
                ));
            }

            let new_value = current - amount_cents;

            // CAS: если значение не изменилось — резервируем
            match self.available.compare_exchange(
                current,
                new_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    println!("Зарезервировано ${:.2}, осталось ${:.2}",
                        amount, new_value as f64 / 100.0);
                    return Ok(());
                }
                Err(_) => {
                    // Кто-то изменил значение, пробуем снова
                    continue;
                }
            }
        }
    }

    // Возврат средств
    fn release(&self, amount: f64) {
        let amount_cents = (amount * 100.0) as u64;
        self.available.fetch_add(amount_cents, Ordering::AcqRel);
        println!("Возвращено ${:.2}", amount);
    }
}

fn main() {
    let pool = Arc::new(LiquidityPool::new(10000.0)); // $10,000

    let mut handles = vec![];

    // 5 трейдеров пытаются резервировать средства
    for trader_id in 0..5 {
        let pool_clone = Arc::clone(&pool);

        let handle = thread::spawn(move || {
            let amounts = [1500.0, 2000.0, 1000.0];

            for amount in amounts {
                match pool_clone.reserve(amount) {
                    Ok(_) => println!("Трейдер {}: успешно зарезервировал ${}", trader_id, amount),
                    Err(e) => println!("Трейдер {}: {}", trader_id, e),
                }
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nИтоговая доступная ликвидность: ${:.2}", pool.get_available());
}
```

## Статистика торговли с атомиками

```rust
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct TradingStats {
    total_trades: AtomicU64,
    winning_trades: AtomicU64,
    losing_trades: AtomicU64,
    total_pnl_cents: AtomicI64, // Может быть отрицательным!
}

impl TradingStats {
    fn new() -> Self {
        TradingStats {
            total_trades: AtomicU64::new(0),
            winning_trades: AtomicU64::new(0),
            losing_trades: AtomicU64::new(0),
            total_pnl_cents: AtomicI64::new(0),
        }
    }

    fn record_trade(&self, pnl: f64) {
        self.total_trades.fetch_add(1, Ordering::Relaxed);

        let pnl_cents = (pnl * 100.0) as i64;
        self.total_pnl_cents.fetch_add(pnl_cents, Ordering::Relaxed);

        if pnl >= 0.0 {
            self.winning_trades.fetch_add(1, Ordering::Relaxed);
        } else {
            self.losing_trades.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn get_summary(&self) -> TradingSummary {
        TradingSummary {
            total: self.total_trades.load(Ordering::Acquire),
            wins: self.winning_trades.load(Ordering::Acquire),
            losses: self.losing_trades.load(Ordering::Acquire),
            pnl: self.total_pnl_cents.load(Ordering::Acquire) as f64 / 100.0,
        }
    }
}

struct TradingSummary {
    total: u64,
    wins: u64,
    losses: u64,
    pnl: f64,
}

impl TradingSummary {
    fn win_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.wins as f64 / self.total as f64 * 100.0
        }
    }
}

fn main() {
    let stats = Arc::new(TradingStats::new());

    let stats1 = Arc::clone(&stats);
    let stats2 = Arc::clone(&stats);
    let stats3 = Arc::clone(&stats);

    // Три стратегии торгуют параллельно
    let strategy1 = thread::spawn(move || {
        let results = [150.0, -50.0, 200.0, -30.0, 80.0];
        for pnl in results {
            stats1.record_trade(pnl);
            thread::sleep(Duration::from_millis(50));
        }
    });

    let strategy2 = thread::spawn(move || {
        let results = [-100.0, 300.0, -20.0, 150.0, -80.0];
        for pnl in results {
            stats2.record_trade(pnl);
            thread::sleep(Duration::from_millis(40));
        }
    });

    let strategy3 = thread::spawn(move || {
        let results = [50.0, 100.0, -200.0, 75.0, 25.0];
        for pnl in results {
            stats3.record_trade(pnl);
            thread::sleep(Duration::from_millis(60));
        }
    });

    // Мониторинг в реальном времени
    let stats_monitor = Arc::clone(&stats);
    let monitor = thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            let summary = stats_monitor.get_summary();
            println!(
                "Статистика: сделок={}, винрейт={:.1}%, PnL=${:.2}",
                summary.total, summary.win_rate(), summary.pnl
            );
        }
    });

    strategy1.join().unwrap();
    strategy2.join().unwrap();
    strategy3.join().unwrap();
    monitor.join().unwrap();

    let final_summary = stats.get_summary();
    println!("\n=== Итоговая статистика ===");
    println!("Всего сделок: {}", final_summary.total);
    println!("Выигрышных: {}", final_summary.wins);
    println!("Убыточных: {}", final_summary.losses);
    println!("Win Rate: {:.1}%", final_summary.win_rate());
    println!("Общий PnL: ${:.2}", final_summary.pnl);
}
```

## Практический пример: Rate Limiter для API

```rust
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct RateLimiter {
    requests_this_second: AtomicU32,
    max_requests_per_second: u32,
    window_start_ms: AtomicU64,
}

impl RateLimiter {
    fn new(max_rps: u32) -> Self {
        RateLimiter {
            requests_this_second: AtomicU32::new(0),
            max_requests_per_second: max_rps,
            window_start_ms: AtomicU64::new(0),
        }
    }

    fn try_acquire(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let current_window = now_ms / 1000 * 1000; // Начало текущей секунды

        // Проверяем, не началось ли новое окно
        let last_window = self.window_start_ms.load(Ordering::Acquire);

        if current_window > last_window {
            // Пытаемся обновить окно
            if self.window_start_ms
                .compare_exchange(last_window, current_window, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Успешно обновили окно, сбрасываем счётчик
                self.requests_this_second.store(0, Ordering::Release);
            }
        }

        // Пытаемся увеличить счётчик
        loop {
            let current = self.requests_this_second.load(Ordering::Acquire);

            if current >= self.max_requests_per_second {
                return false; // Лимит достигнут
            }

            if self.requests_this_second
                .compare_exchange(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return true; // Успешно получили слот
            }
            // Иначе — кто-то другой изменил, пробуем снова
        }
    }
}

fn main() {
    let limiter = Arc::new(RateLimiter::new(10)); // 10 запросов в секунду

    let mut handles = vec![];

    // 5 потоков пытаются делать запросы
    for client_id in 0..5 {
        let limiter_clone = Arc::clone(&limiter);

        let handle = thread::spawn(move || {
            let mut success = 0;
            let mut rejected = 0;

            for _ in 0..10 {
                if limiter_clone.try_acquire() {
                    success += 1;
                } else {
                    rejected += 1;
                }
                thread::sleep(Duration::from_millis(50));
            }

            println!("Клиент {}: {} успешных, {} отклонённых",
                client_id, success, rejected);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Atomics | Неблокирующие потокобезопасные примитивы |
| `AtomicBool/U64/I64` | Основные атомарные типы |
| `load()` / `store()` | Чтение и запись атомарного значения |
| `fetch_add()` | Атомарное сложение |
| `compare_exchange()` | CAS — условная замена |
| `Ordering` | Порядок синхронизации памяти |
| `SeqCst` | Самый строгий и безопасный Ordering |
| `Relaxed` | Минимальный Ordering для счётчиков |

## Практические задания

1. **Счётчик подключений**: Создай структуру `ConnectionPool` с атомарным счётчиком активных подключений. Реализуй методы `connect()` (увеличивает счётчик) и `disconnect()` (уменьшает). Добавь лимит максимальных подключений.

2. **Атомарный best bid/ask**: Реализуй структуру `BestQuotes` с атомарными полями для лучшей цены покупки и продажи. Несколько потоков должны обновлять цены, а другие — читать спред.

3. **Lock-free очередь ордеров**: Используя `AtomicUsize` для индексов, реализуй простую кольцевую очередь фиксированного размера для ордеров.

4. **Мониторинг латентности**: Создай структуру для отслеживания минимального, максимального и среднего времени исполнения ордеров, используя только атомарные операции.

## Домашнее задание

1. **Атомарный Order ID генератор**: Реализуй глобальный генератор уникальных ID для ордеров, который безопасно работает из любого потока. ID должен включать временную метку и порядковый номер.

2. **Trading Circuit Breaker**: Создай "предохранитель", который автоматически останавливает торговлю, если:
   - Убыток за минуту превысил порог
   - Количество ошибок API превысило лимит
   Используй атомарные счётчики для всех метрик.

3. **Lock-free Price Cache**: Реализуй кэш цен для нескольких активов, где обновления происходят атомарно. Поддержи операцию "получить все цены не старше N миллисекунд".

4. **Сравнение производительности**: Напиши бенчмарк, сравнивающий производительность `AtomicU64` и `Mutex<u64>` для счётчика при разном количестве потоков (1, 2, 4, 8, 16). Построй график зависимости.

## Навигация

[← Предыдущий день](../165-avoiding-deadlock-lock-ordering/ru.md) | [Следующий день →](../167-arc-atomic-reference-counting/ru.md)
