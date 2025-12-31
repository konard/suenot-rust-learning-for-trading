# День 192: tokio::interval: периодические задачи

## Аналогия из трейдинга

Представь себе торгового робота, который должен регулярно выполнять определённые действия:
- Каждую секунду проверять текущую цену актива
- Каждые 5 секунд пересчитывать скользящие средние
- Каждую минуту проверять баланс портфеля
- Каждый час сохранять статистику торговли

Это как будильники, которые срабатывают через равные промежутки времени. В tokio для таких задач используется `tokio::time::interval` — механизм, который "тикает" с заданной периодичностью.

## Что такое tokio::interval?

`tokio::interval` создаёт асинхронный интервал, который срабатывает через заданные промежутки времени. В отличие от `sleep`, который просто ждёт один раз, `interval` продолжает "тикать" бесконечно.

```rust
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    // Создаём интервал с периодом 1 секунда
    let mut ticker = interval(Duration::from_secs(1));

    // Первый tick срабатывает сразу!
    ticker.tick().await;
    println!("Тик 1");

    ticker.tick().await;
    println!("Тик 2");

    ticker.tick().await;
    println!("Тик 3");
}
```

**Важно:** Первый вызов `tick()` срабатывает мгновенно! Это удобно для случаев, когда нужно сразу выполнить действие, а потом повторять его периодически.

## Мониторинг цены актива

Рассмотрим практический пример — мониторинг цены Bitcoin каждую секунду:

```rust
use tokio::time::{interval, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

struct PriceMonitor {
    current_price: Arc<RwLock<Option<PriceData>>>,
    price_history: Arc<RwLock<Vec<PriceData>>>,
}

impl PriceMonitor {
    fn new() -> Self {
        PriceMonitor {
            current_price: Arc::new(RwLock::new(None)),
            price_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn start_monitoring(&self) {
        let current_price = Arc::clone(&self.current_price);
        let price_history = Arc::clone(&self.price_history);

        let mut ticker = interval(Duration::from_secs(1));

        loop {
            ticker.tick().await;

            // Симулируем получение цены (в реальности — запрос к API)
            let price = simulate_price_fetch().await;

            let data = PriceData {
                symbol: "BTC/USDT".to_string(),
                price,
                timestamp: get_timestamp(),
            };

            println!("Цена BTC: ${:.2}", data.price);

            // Обновляем текущую цену
            {
                let mut current = current_price.write().await;
                *current = Some(data.clone());
            }

            // Сохраняем в историю
            {
                let mut history = price_history.write().await;
                history.push(data);

                // Храним только последние 100 записей
                if history.len() > 100 {
                    history.remove(0);
                }
            }
        }
    }
}

async fn simulate_price_fetch() -> f64 {
    // Симуляция случайной цены около $42000
    42000.0 + (rand::random::<f64>() - 0.5) * 1000.0
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn rand_random() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(get_timestamp());
    (hasher.finish() % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let monitor = PriceMonitor::new();

    // Запускаем мониторинг (работает бесконечно)
    // В реальном приложении нужен механизм остановки
    monitor.start_monitoring().await;
}
```

## Несколько интервалов одновременно

В торговле часто нужно выполнять разные задачи с разной периодичностью. Используем `tokio::select!` для этого:

```rust
use tokio::time::{interval, Duration};

struct TradingBot {
    symbol: String,
}

impl TradingBot {
    fn new(symbol: &str) -> Self {
        TradingBot {
            symbol: symbol.to_string(),
        }
    }

    async fn check_price(&self) {
        println!("[{}] Проверка цены...", self.symbol);
    }

    async fn calculate_indicators(&self) {
        println!("[{}] Расчёт индикаторов (SMA, RSI, MACD)...", self.symbol);
    }

    async fn check_signals(&self) {
        println!("[{}] Проверка торговых сигналов...", self.symbol);
    }

    async fn save_statistics(&self) {
        println!("[{}] Сохранение статистики...", self.symbol);
    }

    async fn run(&self) {
        // Разные интервалы для разных задач
        let mut price_ticker = interval(Duration::from_secs(1));
        let mut indicator_ticker = interval(Duration::from_secs(5));
        let mut signal_ticker = interval(Duration::from_secs(10));
        let mut stats_ticker = interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = price_ticker.tick() => {
                    self.check_price().await;
                }
                _ = indicator_ticker.tick() => {
                    self.calculate_indicators().await;
                }
                _ = signal_ticker.tick() => {
                    self.check_signals().await;
                }
                _ = stats_ticker.tick() => {
                    self.save_statistics().await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let bot = TradingBot::new("BTC/USDT");
    bot.run().await;
}
```

## Поведение при пропуске тиков (MissedTickBehavior)

Что происходит, если обработка тика занимает больше времени, чем интервал? Tokio предлагает три стратегии:

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};

async fn demonstrate_missed_tick_behavior() {
    // Стратегия 1: Burst (по умолчанию)
    // Пропущенные тики выполняются сразу подряд
    let mut burst_ticker = interval(Duration::from_millis(100));
    burst_ticker.set_missed_tick_behavior(MissedTickBehavior::Burst);

    // Стратегия 2: Delay
    // Следующий тик отсчитывается от момента завершения предыдущего
    let mut delay_ticker = interval(Duration::from_millis(100));
    delay_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // Стратегия 3: Skip
    // Пропущенные тики игнорируются, следующий — по расписанию
    let mut skip_ticker = interval(Duration::from_millis(100));
    skip_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
}
```

### Пример для трейдинга

```rust
use tokio::time::{interval, Duration, MissedTickBehavior, sleep};

struct OrderBookMonitor {
    symbol: String,
}

impl OrderBookMonitor {
    async fn heavy_analysis(&self) {
        // Симуляция тяжёлой обработки (200ms при интервале 100ms)
        println!("[{}] Начало анализа стакана заявок...", self.symbol);
        sleep(Duration::from_millis(200)).await;
        println!("[{}] Анализ завершён", self.symbol);
    }

    async fn run_with_skip(&self) {
        // Skip — лучший выбор для мониторинга в реальном времени
        // Нам важны свежие данные, а не накопившиеся старые
        let mut ticker = interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        for i in 0..5 {
            ticker.tick().await;
            println!("Тик {}", i + 1);
            self.heavy_analysis().await;
        }
    }

    async fn run_with_delay(&self) {
        // Delay — гарантирует минимальный интервал между запусками
        // Хорошо для задач, где важна полнота обработки
        let mut ticker = interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        for i in 0..5 {
            ticker.tick().await;
            println!("Тик {}", i + 1);
            self.heavy_analysis().await;
        }
    }
}

#[tokio::main]
async fn main() {
    let monitor = OrderBookMonitor {
        symbol: "ETH/USDT".to_string(),
    };

    println!("=== Режим Skip ===");
    monitor.run_with_skip().await;

    println!("\n=== Режим Delay ===");
    monitor.run_with_delay().await;
}
```

## Остановка интервала

В реальных приложениях нужен механизм остановки:

```rust
use tokio::time::{interval, Duration};
use tokio::sync::watch;

struct PriceAlert {
    symbol: String,
    target_price: f64,
}

impl PriceAlert {
    async fn monitor_until_target(
        &self,
        mut shutdown: watch::Receiver<bool>,
    ) -> Option<f64> {
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let current_price = self.fetch_price().await;
                    println!("[{}] Цена: ${:.2}, Цель: ${:.2}",
                        self.symbol, current_price, self.target_price);

                    if current_price >= self.target_price {
                        println!("Целевая цена достигнута!");
                        return Some(current_price);
                    }
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        println!("Мониторинг остановлен");
                        return None;
                    }
                }
            }
        }
    }

    async fn fetch_price(&self) -> f64 {
        // Симуляция цены с ростом
        static COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        42000.0 + count as f64 * 100.0
    }
}

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let alert = PriceAlert {
        symbol: "BTC/USDT".to_string(),
        target_price: 42500.0,
    };

    // Запускаем мониторинг в отдельной задаче
    let handle = tokio::spawn(async move {
        alert.monitor_until_target(shutdown_rx).await
    });

    // Можем остановить вручную через 10 секунд
    // shutdown_tx.send(true).unwrap();

    match handle.await.unwrap() {
        Some(price) => println!("Финальная цена: ${:.2}", price),
        None => println!("Мониторинг был прерван"),
    }
}
```

## interval_at: начало в определённый момент

Иногда нужно начать интервал не сразу, а в определённый момент:

```rust
use tokio::time::{interval_at, Duration, Instant};

async fn schedule_market_tasks() {
    let now = Instant::now();

    // Начинаем через 5 секунд, потом каждую минуту
    let start = now + Duration::from_secs(5);
    let mut ticker = interval_at(start, Duration::from_secs(60));

    println!("Планировщик запущен, первый тик через 5 секунд...");

    loop {
        ticker.tick().await;
        println!("Выполняю запланированную задачу: {:?}", Instant::now());
    }
}

#[tokio::main]
async fn main() {
    schedule_market_tasks().await;
}
```

## Практический пример: Торговый планировщик

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MarketData {
    price: f64,
    volume: f64,
    sma_20: Option<f64>,
    rsi: Option<f64>,
}

#[derive(Debug)]
struct TradingScheduler {
    symbol: String,
    market_data: Arc<RwLock<MarketData>>,
    price_history: Arc<RwLock<Vec<f64>>>,
}

impl TradingScheduler {
    fn new(symbol: &str) -> Self {
        TradingScheduler {
            symbol: symbol.to_string(),
            market_data: Arc::new(RwLock::new(MarketData {
                price: 0.0,
                volume: 0.0,
                sma_20: None,
                rsi: None,
            })),
            price_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn update_price(&self) {
        let price = simulate_price();

        let mut data = self.market_data.write().await;
        data.price = price;

        let mut history = self.price_history.write().await;
        history.push(price);

        // Храним только 100 последних цен
        if history.len() > 100 {
            history.remove(0);
        }

        println!("[Цена] {}: ${:.2}", self.symbol, price);
    }

    async fn calculate_sma(&self) {
        let history = self.price_history.read().await;

        if history.len() >= 20 {
            let sum: f64 = history.iter().rev().take(20).sum();
            let sma = sum / 20.0;

            drop(history); // Освобождаем read lock

            let mut data = self.market_data.write().await;
            data.sma_20 = Some(sma);

            println!("[SMA-20] {}: ${:.2}", self.symbol, sma);
        }
    }

    async fn calculate_rsi(&self) {
        let history = self.price_history.read().await;

        if history.len() >= 14 {
            // Упрощённый расчёт RSI
            let changes: Vec<f64> = history
                .windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let gains: f64 = changes.iter()
                .filter(|&&x| x > 0.0)
                .sum();
            let losses: f64 = changes.iter()
                .filter(|&&x| x < 0.0)
                .map(|x| x.abs())
                .sum();

            let rsi = if losses == 0.0 {
                100.0
            } else {
                let rs = gains / losses;
                100.0 - (100.0 / (1.0 + rs))
            };

            drop(history);

            let mut data = self.market_data.write().await;
            data.rsi = Some(rsi);

            println!("[RSI] {}: {:.1}", self.symbol, rsi);
        }
    }

    async fn check_trading_signals(&self) {
        let data = self.market_data.read().await;

        if let (Some(sma), Some(rsi)) = (data.sma_20, data.rsi) {
            let price = data.price;

            // Простая стратегия:
            // Покупка: цена выше SMA и RSI < 30
            // Продажа: цена ниже SMA и RSI > 70
            if price > sma && rsi < 30.0 {
                println!("[СИГНАЛ] {} - ПОКУПКА! Цена ${:.2} > SMA ${:.2}, RSI {:.1}",
                    self.symbol, price, sma, rsi);
            } else if price < sma && rsi > 70.0 {
                println!("[СИГНАЛ] {} - ПРОДАЖА! Цена ${:.2} < SMA ${:.2}, RSI {:.1}",
                    self.symbol, price, sma, rsi);
            }
        }
    }

    async fn run(&self, max_iterations: usize) {
        let mut price_ticker = interval(Duration::from_millis(500));
        let mut sma_ticker = interval(Duration::from_secs(2));
        let mut rsi_ticker = interval(Duration::from_secs(3));
        let mut signal_ticker = interval(Duration::from_secs(5));

        // Для задач с индикаторами используем Skip
        sma_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        rsi_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                println!("\nЗавершение работы планировщика");
                break;
            }

            tokio::select! {
                _ = price_ticker.tick() => {
                    self.update_price().await;
                    iterations += 1;
                }
                _ = sma_ticker.tick() => {
                    self.calculate_sma().await;
                }
                _ = rsi_ticker.tick() => {
                    self.calculate_rsi().await;
                }
                _ = signal_ticker.tick() => {
                    self.check_trading_signals().await;
                }
            }
        }
    }
}

fn simulate_price() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    42000.0 + (nanos as f64 / 1_000_000.0) - 500.0
}

#[tokio::main]
async fn main() {
    let scheduler = TradingScheduler::new("BTC/USDT");

    println!("Запуск торгового планировщика...\n");
    scheduler.run(30).await;

    println!("\nФинальное состояние:");
    let data = scheduler.market_data.read().await;
    println!("{:?}", *data);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `interval(duration)` | Создаёт периодический таймер |
| `tick().await` | Ожидает следующего тика |
| Первый тик | Срабатывает мгновенно |
| `MissedTickBehavior::Burst` | Пропущенные тики выполняются подряд |
| `MissedTickBehavior::Delay` | Отсчёт от завершения предыдущего |
| `MissedTickBehavior::Skip` | Пропущенные тики игнорируются |
| `interval_at(start, period)` | Начало в определённый момент |
| `tokio::select!` | Обработка нескольких интервалов |

## Практические задания

1. **Монитор нескольких активов**: Создай программу, которая одновременно отслеживает цены BTC, ETH и SOL с разными интервалами (1с, 2с, 3с соответственно).

2. **Детектор волатильности**: Напиши программу, которая каждые 5 секунд вычисляет волатильность (стандартное отклонение) за последние 20 измерений цены и выводит предупреждение, если волатильность превышает порог.

3. **Балансировщик портфеля**: Реализуй программу, которая каждую минуту проверяет распределение активов в портфеле и выводит рекомендации по ребалансировке, если отклонение от целевого распределения превышает 5%.

## Домашнее задание

1. **Система алертов**: Реализуй систему ценовых алертов с разными типами:
   - Алерт при достижении цены
   - Алерт при изменении цены на N%
   - Алерт при пересечении скользящей средней

   Используй разные интервалы для проверки разных типов алертов.

2. **Heartbeat монитор**: Создай систему, которая:
   - Отправляет "heartbeat" каждые 10 секунд
   - Проверяет, что все компоненты системы "живы"
   - Выводит предупреждение, если компонент не отвечает 30 секунд

3. **Умный планировщик**: Реализуй планировщик, который:
   - Увеличивает частоту проверки цены при высокой волатильности
   - Уменьшает частоту в спокойные периоды
   - Использует `interval` с динамическим изменением периода

4. **Симулятор торговой сессии**: Создай программу, которая:
   - Моделирует торговую сессию с открытием и закрытием рынка
   - Использует `interval_at` для запуска в определённое время
   - Выполняет разные действия до, во время и после сессии

## Навигация

[← Предыдущий день](../191-tokio-sleep-delay/ru.md) | [Следующий день →](../193-tokio-timeout/ru.md)
