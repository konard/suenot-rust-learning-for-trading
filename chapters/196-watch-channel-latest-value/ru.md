# День 196: Watch Channel: последнее значение

## Аналогия из трейдинга

Представь информационное табло на бирже, которое показывает текущую цену Bitcoin. Когда цена меняется, табло обновляется — но показывает только **последнюю** цену. Тебе не важна история изменений за последнюю секунду, важно только актуальное значение прямо сейчас.

Это именно то, что делает **watch channel** в Tokio:
- Один отправитель обновляет значение (биржа обновляет цену)
- Множество получателей видят последнее значение (трейдеры смотрят на табло)
- Промежуточные значения могут быть пропущены (если цена изменилась дважды за секунду, ты увидишь только последнюю)

В реальном трейдинге watch channel идеально подходит для:
- **Текущая цена актива** — всем нужна только последняя цена
- **Статус подключения к бирже** — connected/disconnected
- **Параметры риск-менеджмента** — текущий лимит позиции
- **Режим торговли** — активный/приостановлен/остановлен

## Что такое Watch Channel?

Watch channel — это канал связи между задачами с особыми свойствами:

| Свойство | Описание |
|----------|----------|
| Multi-producer | Много отправителей могут подписаться через `subscribe()` |
| Multi-consumer | Много получателей могут читать значение |
| Только последнее | Хранится только самое свежее значение |
| Уведомления | Получатели оповещаются об изменениях |
| Нет буфера | Промежуточные значения теряются |

## Создание Watch Channel

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    // Создаём канал с начальным значением цены BTC
    let (tx, rx) = watch::channel(42000.0_f64);

    // tx — Sender для отправки новых значений
    // rx — Receiver для чтения последнего значения

    println!("Начальная цена BTC: ${}", *rx.borrow());
}
```

## Базовый пример: Мониторинг цены

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Канал для текущей цены Bitcoin
    let (price_tx, mut price_rx) = watch::channel(42000.0_f64);

    // Задача-продюсер: обновляет цену
    let producer = tokio::spawn(async move {
        let prices = [42100.0, 42050.0, 42200.0, 42150.0, 42300.0];

        for price in prices {
            sleep(Duration::from_millis(500)).await;
            println!("[Биржа] Новая цена BTC: ${}", price);

            if price_tx.send(price).is_err() {
                println!("[Биржа] Все получатели отключились");
                break;
            }
        }
    });

    // Задача-консьюмер: следит за изменениями
    let consumer = tokio::spawn(async move {
        loop {
            // changed() ждёт нового значения
            if price_rx.changed().await.is_err() {
                println!("[Трейдер] Канал закрыт");
                break;
            }

            // borrow_and_update() получает значение и отмечает как прочитанное
            let price = *price_rx.borrow_and_update();
            println!("[Трейдер] Вижу цену: ${}", price);
        }
    });

    producer.await.unwrap();
    consumer.await.unwrap();
}
```

## Множество получателей: Стратегии и мониторинг

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_trade: f64,
}

#[tokio::main]
async fn main() {
    let initial_data = MarketData {
        symbol: "BTC/USDT".to_string(),
        bid: 41950.0,
        ask: 42050.0,
        last_trade: 42000.0,
    };

    let (data_tx, data_rx) = watch::channel(initial_data);

    // Клонируем receiver для каждой задачи
    let mut strategy_rx = data_rx.clone();
    let mut risk_rx = data_rx.clone();
    let mut logger_rx = data_rx;

    // Торговая стратегия следит за спредом
    let strategy = tokio::spawn(async move {
        loop {
            if strategy_rx.changed().await.is_err() {
                break;
            }
            let data = strategy_rx.borrow_and_update().clone();
            let spread = data.ask - data.bid;

            if spread < 50.0 {
                println!("[Стратегия] Узкий спред ${:.2} — хорошо для маркет-ордера!", spread);
            } else {
                println!("[Стратегия] Широкий спред ${:.2} — используем лимитный ордер", spread);
            }
        }
    });

    // Риск-менеджмент следит за ценой
    let risk_manager = tokio::spawn(async move {
        let mut last_price = 0.0;

        loop {
            if risk_rx.changed().await.is_err() {
                break;
            }
            let data = risk_rx.borrow_and_update().clone();

            if last_price > 0.0 {
                let change_pct = (data.last_trade - last_price) / last_price * 100.0;
                if change_pct.abs() > 0.5 {
                    println!("[Риск] Резкое движение: {:.2}%!", change_pct);
                }
            }
            last_price = data.last_trade;
        }
    });

    // Логгер записывает все изменения
    let logger = tokio::spawn(async move {
        loop {
            if logger_rx.changed().await.is_err() {
                break;
            }
            let data = logger_rx.borrow_and_update().clone();
            println!("[Лог] {} bid={:.2} ask={:.2} last={:.2}",
                data.symbol, data.bid, data.ask, data.last_trade);
        }
    });

    // Продюсер обновляет рыночные данные
    let updates = vec![
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42000.0, ask: 42020.0, last_trade: 42010.0 },
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42100.0, ask: 42200.0, last_trade: 42150.0 },
        MarketData { symbol: "BTC/USDT".to_string(), bid: 42400.0, ask: 42450.0, last_trade: 42420.0 },
    ];

    for data in updates {
        sleep(Duration::from_millis(300)).await;
        let _ = data_tx.send(data);
    }

    // Даём время получателям обработать
    sleep(Duration::from_millis(100)).await;
    drop(data_tx); // Закрываем канал

    let _ = tokio::join!(strategy, risk_manager, logger);
}
```

## Проверка изменений без ожидания

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel("active");

    // has_changed() проверяет без блокировки
    println!("Изменилось? {}", rx.has_changed().unwrap());

    // Отправляем новое значение
    tx.send("paused").unwrap();

    println!("Изменилось? {}", rx.has_changed().unwrap());

    // borrow() читает без отметки о прочтении
    println!("Статус: {}", *rx.borrow());
    println!("Изменилось? {}", rx.has_changed().unwrap()); // Всё ещё true!

    // borrow_and_update() читает И отмечает прочитанным
    {
        let status = rx.borrow_and_update();
        println!("Статус: {}", *status);
    }
    println!("Изменилось? {}", rx.has_changed().unwrap()); // Теперь false
}
```

## Практический пример: Торговый режим

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug, PartialEq)]
enum TradingMode {
    Active,      // Торговля разрешена
    Paused,      // Временная пауза
    CloseOnly,   // Только закрытие позиций
    Stopped,     // Полная остановка
}

struct TradingEngine {
    mode_rx: watch::Receiver<TradingMode>,
}

impl TradingEngine {
    fn new(mode_rx: watch::Receiver<TradingMode>) -> Self {
        TradingEngine { mode_rx }
    }

    async fn can_open_position(&self) -> bool {
        *self.mode_rx.borrow() == TradingMode::Active
    }

    async fn can_close_position(&self) -> bool {
        let mode = self.mode_rx.borrow().clone();
        mode == TradingMode::Active || mode == TradingMode::CloseOnly
    }

    async fn run(&mut self) {
        println!("[Engine] Запуск торгового движка");

        loop {
            // Ждём изменения режима
            if self.mode_rx.changed().await.is_err() {
                println!("[Engine] Канал управления закрыт, остановка");
                break;
            }

            let mode = self.mode_rx.borrow_and_update().clone();

            match mode {
                TradingMode::Active => {
                    println!("[Engine] Режим ACTIVE: полная торговля разрешена");
                }
                TradingMode::Paused => {
                    println!("[Engine] Режим PAUSED: ждём возобновления");
                }
                TradingMode::CloseOnly => {
                    println!("[Engine] Режим CLOSE_ONLY: только закрытие позиций");
                }
                TradingMode::Stopped => {
                    println!("[Engine] Режим STOPPED: полная остановка");
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (mode_tx, mode_rx) = watch::channel(TradingMode::Active);

    let mut engine = TradingEngine::new(mode_rx);

    let engine_task = tokio::spawn(async move {
        engine.run().await;
    });

    // Симуляция изменения режимов
    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Рынок волатилен -> PAUSED");
    mode_tx.send(TradingMode::Paused).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Закрываем позиции -> CLOSE_ONLY");
    mode_tx.send(TradingMode::CloseOnly).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Рынок стабилизировался -> ACTIVE");
    mode_tx.send(TradingMode::Active).unwrap();

    sleep(Duration::from_millis(100)).await;
    println!("\n[Control] Конец торговой сессии -> STOPPED");
    mode_tx.send(TradingMode::Stopped).unwrap();

    engine_task.await.unwrap();
}
```

## Паттерн: Конфигурация в реальном времени

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    max_drawdown_pct: f64,
    stop_loss_pct: f64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_size: 10.0,
            max_daily_loss: 5000.0,
            max_drawdown_pct: 5.0,
            stop_loss_pct: 2.0,
        }
    }
}

struct RiskManager {
    config_rx: watch::Receiver<RiskConfig>,
}

impl RiskManager {
    async fn check_order(&self, symbol: &str, size: f64, price: f64) -> Result<(), String> {
        let config = self.config_rx.borrow().clone();

        if size > config.max_position_size {
            return Err(format!(
                "Размер {} превышает лимит {}",
                size, config.max_position_size
            ));
        }

        let order_value = size * price;
        if order_value > config.max_daily_loss {
            return Err(format!(
                "Стоимость ордера ${} превышает дневной лимит ${}",
                order_value, config.max_daily_loss
            ));
        }

        println!("[Risk] Ордер {} x {} @ ${} одобрен", symbol, size, price);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (config_tx, config_rx) = watch::channel(RiskConfig::default());

    let risk_manager = RiskManager { config_rx };

    // Проверяем ордер с начальными настройками
    println!("=== Начальная конфигурация ===");
    let _ = risk_manager.check_order("BTC", 5.0, 42000.0).await;
    let _ = risk_manager.check_order("BTC", 15.0, 42000.0).await; // Отклонён

    // Обновляем конфигурацию
    println!("\n=== Обновляем лимиты ===");
    let new_config = RiskConfig {
        max_position_size: 20.0,
        max_daily_loss: 10000.0,
        max_drawdown_pct: 3.0,
        stop_loss_pct: 1.5,
    };
    config_tx.send(new_config).unwrap();

    // Теперь ордер пройдёт
    let _ = risk_manager.check_order("BTC", 15.0, 42000.0).await;
}
```

## Сравнение с broadcast channel

| Характеристика | watch | broadcast |
|----------------|-------|-----------|
| Буфер | 1 (только последнее) | Настраиваемый |
| Пропуск сообщений | Да | Только при переполнении |
| Начальное значение | Обязательно | Нет |
| Подписка на лету | `subscribe()` | `subscribe()` |
| Использование | Состояние | События |

## Когда использовать watch channel

**Используй watch когда:**
- Важно только последнее значение (текущая цена, статус)
- Получатели могут пропустить промежуточные значения
- Нужно распространять конфигурацию
- Требуется сигнал состояния (включён/выключен)

**Не используй watch когда:**
- Каждое сообщение должно быть обработано (ордера!)
- Нужна история изменений
- Важен порядок обработки всех событий

## Практический пример: Мониторинг портфеля

```rust
use tokio::sync::watch;
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct PortfolioSnapshot {
    timestamp: u64,
    total_value: f64,
    positions: HashMap<String, f64>,
    unrealized_pnl: f64,
    daily_pnl: f64,
}

#[tokio::main]
async fn main() {
    let initial_snapshot = PortfolioSnapshot {
        timestamp: 0,
        total_value: 100000.0,
        positions: HashMap::new(),
        unrealized_pnl: 0.0,
        daily_pnl: 0.0,
    };

    let (snapshot_tx, snapshot_rx) = watch::channel(initial_snapshot);

    // UI-поток обновляет дашборд
    let mut ui_rx = snapshot_rx.clone();
    let ui_task = tokio::spawn(async move {
        loop {
            if ui_rx.changed().await.is_err() {
                break;
            }
            let snap = ui_rx.borrow_and_update().clone();
            println!("[UI] Портфель: ${:.2} | PnL: ${:.2} | Позиций: {}",
                snap.total_value, snap.daily_pnl, snap.positions.len());
        }
    });

    // Риск-мониторинг проверяет лимиты
    let mut risk_rx = snapshot_rx.clone();
    let risk_task = tokio::spawn(async move {
        loop {
            if risk_rx.changed().await.is_err() {
                break;
            }
            let snap = risk_rx.borrow_and_update().clone();

            let drawdown_pct = -snap.unrealized_pnl / 100000.0 * 100.0;
            if drawdown_pct > 2.0 {
                println!("[RISK ALERT] Просадка {:.2}% превышает лимит!", drawdown_pct);
            }
        }
    });

    // Симуляция обновлений портфеля
    let mut time = 1u64;
    for i in 0..5 {
        sleep(Duration::from_millis(200)).await;

        let mut positions = HashMap::new();
        positions.insert("BTC".to_string(), 2.0 + i as f64 * 0.5);
        positions.insert("ETH".to_string(), 10.0);

        let pnl = (i as f64 - 2.0) * 1000.0; // Варьируется от -2000 до +2000

        let snapshot = PortfolioSnapshot {
            timestamp: time,
            total_value: 100000.0 + pnl,
            positions,
            unrealized_pnl: pnl,
            daily_pnl: pnl,
        };

        let _ = snapshot_tx.send(snapshot);
        time += 1;
    }

    sleep(Duration::from_millis(100)).await;
    drop(snapshot_tx);

    let _ = tokio::join!(ui_task, risk_task);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `watch::channel(T)` | Создаёт канал с начальным значением типа T |
| `tx.send(value)` | Обновляет значение, оповещает получателей |
| `rx.borrow()` | Читает значение без отметки о прочтении |
| `rx.borrow_and_update()` | Читает и отмечает прочитанным |
| `rx.changed().await` | Ждёт нового значения |
| `rx.has_changed()` | Проверяет наличие изменений |
| `tx.subscribe()` | Создаёт нового получателя |

## Домашнее задание

1. **Цена в реальном времени**: Создай систему, где один продюсер обновляет цену BTC каждые 100мс, а три консьюмера:
   - Выводят каждое изменение
   - Считают скользящее среднее за последние 5 значений
   - Генерируют сигнал при изменении цены более чем на 1%

2. **Конфигурация торговли**: Реализуй структуру `TradingConfig` с параметрами (leverage, max_orders, allowed_symbols) и систему, которая:
   - Позволяет обновлять конфигурацию на лету
   - Имеет несколько воркеров, которые проверяют конфиг перед каждой операцией

3. **Статус подключения**: Создай монитор подключения к бирже с состояниями (Connected, Reconnecting, Disconnected) и:
   - Торговый движок, который приостанавливает работу при Disconnected
   - Логгер, который записывает все изменения статуса
   - Alerter, который отправляет уведомление после 3 попыток реконнекта

4. **Портфельный дашборд**: Реализуй систему мониторинга портфеля, где:
   - Продюсер обновляет позиции каждые 500мс
   - UI-консьюмер показывает топ-3 позиции
   - Risk-консьюмер проверяет общий exposure
   - Performance-консьюмер считает дневной PnL

## Навигация

[← Предыдущий день](../195-broadcast-channel-all-subscribers/ru.md) | [Следующий день →](../197-http-basics-reqwest-get/ru.md)
