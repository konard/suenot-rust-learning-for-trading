# День 146: Структурированные логи: tracing

## Аналогия из трейдинга

Представь, что ты профессиональный трейдер, который ведёт подробный **журнал торговли**. Каждая запись содержит не просто текст "купил биткоин", а структурированную информацию:
- Время операции
- Тикер инструмента
- Цена входа
- Объём позиции
- Стратегия, которая сгенерировала сигнал
- Уровень важности (информация, предупреждение, ошибка)

Библиотека `tracing` в Rust работает точно так же — это не просто логирование, а **структурированная система диагностики**, где каждое событие несёт контекст и может быть проанализировано автоматически.

## Почему tracing, а не log?

| Характеристика | `log` | `tracing` |
|---------------|-------|-----------|
| Структурированные данные | Ограниченно | Полная поддержка |
| Асинхронный код | Проблемы с контекстом | Нативная поддержка |
| Спаны (временные интервалы) | Нет | Да |
| Производительность | Хорошая | Отличная (zero-cost) |
| Фильтрация | По уровню | По уровню, цели, полям |

## Установка

Добавь в `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Базовое использование

```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber;

fn main() {
    // Инициализация подписчика (subscriber)
    tracing_subscriber::fmt::init();

    info!("Торговый бот запущен");
    debug!("Загрузка конфигурации...");

    let btc_price = 42000.0;
    let position_size = 0.5;

    // Структурированные поля
    info!(
        ticker = "BTC/USDT",
        price = btc_price,
        size = position_size,
        "Открыта позиция"
    );

    warn!(
        ticker = "BTC/USDT",
        current_pnl = -50.0,
        threshold = -100.0,
        "PnL приближается к порогу стоп-лосса"
    );

    error!(
        ticker = "BTC/USDT",
        error_code = "INSUFFICIENT_BALANCE",
        required = 5000.0,
        available = 3000.0,
        "Недостаточно средств для открытия позиции"
    );
}
```

## Уровни логирования

```rust
use tracing::{trace, debug, info, warn, error, Level};

fn log_levels_demo() {
    // От наименее до наиболее важного:
    trace!("Детальная отладка — каждый тик цены");
    debug!("Отладочная информация — расчёт индикаторов");
    info!("Информационные события — открытие/закрытие позиций");
    warn!("Предупреждения — высокая волатильность, низкий баланс");
    error!("Ошибки — неудачные ордера, потеря соединения");
}
```

## Структурированные поля

```rust
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    let order_id = "ORD-12345";
    let ticker = "ETH/USDT";
    let side = "BUY";
    let price = 2500.0_f64;
    let quantity = 1.5_f64;

    // Именованные поля
    info!(
        order_id = order_id,
        ticker = ticker,
        side = side,
        price = price,
        quantity = quantity,
        total_value = price * quantity,
        "Ордер создан"
    );

    // Сокращённая форма (имя переменной = имя поля)
    info!(%order_id, %ticker, %side, price, quantity, "Ордер создан");

    // Форматы полей:
    // ? — Debug форматирование
    // % — Display форматирование
    let prices = vec![42000.0, 42100.0, 42050.0];
    info!(prices = ?prices, "Последние цены");
}
```

## Спаны (Spans) — временные контексты

Спан — это именованный временной интервал. В трейдинге это идеально подходит для отслеживания жизненного цикла ордеров или сделок.

```rust
use tracing::{info, info_span, warn, Instrument};

fn main() {
    tracing_subscriber::fmt::init();

    // Создание спана для отслеживания сделки
    let trade_span = info_span!(
        "trade",
        trade_id = "TRD-001",
        ticker = "BTC/USDT",
        strategy = "momentum"
    );

    // Вход в спан
    let _guard = trade_span.enter();

    info!(price = 42000.0, side = "BUY", "Открытие позиции");

    // Симуляция работы
    analyze_market();
    check_stop_loss(42000.0, 41500.0);

    info!(price = 43000.0, pnl = 500.0, "Закрытие позиции");
    // Спан автоматически закрывается при выходе из области видимости
}

fn analyze_market() {
    let span = info_span!("market_analysis");
    let _guard = span.enter();

    info!("Расчёт RSI...");
    info!("Расчёт MACD...");
    info!(rsi = 65.0, macd = 0.5, "Индикаторы рассчитаны");
}

fn check_stop_loss(entry: f64, stop: f64) {
    let span = info_span!("stop_loss_check", entry = entry, stop = stop);
    let _guard = span.enter();

    let risk = ((entry - stop) / entry) * 100.0;
    if risk > 2.0 {
        warn!(risk_percent = risk, "Риск превышает 2%");
    } else {
        info!(risk_percent = risk, "Риск в пределах нормы");
    }
}
```

## Атрибут #[instrument]

Автоматически создаёт спан для функции:

```rust
use tracing::{info, instrument, warn};

fn main() {
    tracing_subscriber::fmt::init();

    let result = execute_trade("BTC/USDT", 42000.0, 0.5, "BUY");
    info!(result = ?result, "Результат сделки");
}

#[instrument(
    name = "execute_trade",
    skip(ticker),  // Не включать в спан
    fields(
        ticker = %ticker,
        trade_type = "market"
    )
)]
fn execute_trade(ticker: &str, price: f64, quantity: f64, side: &str) -> Result<String, String> {
    info!("Проверка баланса...");

    if quantity <= 0.0 {
        warn!("Некорректный объём");
        return Err("Invalid quantity".to_string());
    }

    info!(
        value = price * quantity,
        "Исполнение ордера"
    );

    Ok(format!("ORD-{}", rand_id()))
}

fn rand_id() -> u32 {
    42 // Упрощённо для примера
}
```

## Асинхронный код с tracing

`tracing` отлично работает с async/await:

```rust
use tracing::{info, instrument, Instrument};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Способ 1: через атрибут
    fetch_price("BTC/USDT").await;

    // Способ 2: через .instrument()
    let span = tracing::info_span!("price_monitor");
    monitor_prices().instrument(span).await;
}

#[instrument]
async fn fetch_price(ticker: &str) -> f64 {
    info!("Запрос цены...");

    // Симуляция API-вызова
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let price = 42000.0;
    info!(price = price, "Цена получена");
    price
}

async fn monitor_prices() {
    info!("Мониторинг начат");

    for i in 0..3 {
        let price = fetch_price("BTC/USDT").await;
        info!(iteration = i, price = price, "Обновление цены");
    }

    info!("Мониторинг завершён");
}
```

## Фильтрация логов

```rust
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() {
    // Фильтрация через переменную окружения RUST_LOG
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Или программно:
    // tracing_subscriber::fmt()
    //     .with_max_level(Level::DEBUG)
    //     .init();

    // Запуск: RUST_LOG=debug cargo run
    // Или: RUST_LOG=trading_bot=debug,hyper=warn cargo run
}
```

## Практический пример: торговый бот с логированием

```rust
use tracing::{info, warn, error, instrument, info_span};
use tracing_subscriber::fmt;
use std::collections::HashMap;

fn main() {
    // Инициализация с красивым форматом
    fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("=== Торговый бот v1.0 ===");

    let mut bot = TradingBot::new(10000.0);

    bot.process_signal("BTC/USDT", Signal::Buy, 42000.0, 0.1);
    bot.process_signal("ETH/USDT", Signal::Buy, 2500.0, 1.0);
    bot.process_signal("BTC/USDT", Signal::Sell, 43000.0, 0.1);

    bot.print_summary();
}

#[derive(Debug, Clone)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

struct TradingBot {
    balance: f64,
    positions: HashMap<String, Position>,
    trade_count: u32,
}

#[derive(Debug)]
struct Position {
    quantity: f64,
    entry_price: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        info!(balance = initial_balance, "Инициализация бота");
        Self {
            balance: initial_balance,
            positions: HashMap::new(),
            trade_count: 0,
        }
    }

    #[instrument(skip(self), fields(trade_id = self.trade_count + 1))]
    fn process_signal(&mut self, ticker: &str, signal: Signal, price: f64, quantity: f64) {
        info!(signal = ?signal, "Обработка сигнала");

        match signal {
            Signal::Buy => self.open_position(ticker, price, quantity),
            Signal::Sell => self.close_position(ticker, price, quantity),
            Signal::Hold => info!("Удержание позиции"),
        }
    }

    fn open_position(&mut self, ticker: &str, price: f64, quantity: f64) {
        let span = info_span!("open_position", ticker = ticker);
        let _guard = span.enter();

        let cost = price * quantity;

        if cost > self.balance {
            error!(
                required = cost,
                available = self.balance,
                "Недостаточно средств"
            );
            return;
        }

        self.balance -= cost;
        self.positions.insert(
            ticker.to_string(),
            Position {
                quantity,
                entry_price: price,
            },
        );
        self.trade_count += 1;

        info!(
            cost = cost,
            new_balance = self.balance,
            "Позиция открыта"
        );
    }

    fn close_position(&mut self, ticker: &str, price: f64, quantity: f64) {
        let span = info_span!("close_position", ticker = ticker);
        let _guard = span.enter();

        if let Some(position) = self.positions.remove(ticker) {
            let sell_quantity = quantity.min(position.quantity);
            let revenue = price * sell_quantity;
            let cost = position.entry_price * sell_quantity;
            let pnl = revenue - cost;

            self.balance += revenue;
            self.trade_count += 1;

            if pnl >= 0.0 {
                info!(
                    pnl = pnl,
                    roi_percent = (pnl / cost) * 100.0,
                    "Прибыльная сделка"
                );
            } else {
                warn!(
                    pnl = pnl,
                    roi_percent = (pnl / cost) * 100.0,
                    "Убыточная сделка"
                );
            }
        } else {
            warn!("Позиция не найдена");
        }
    }

    fn print_summary(&self) {
        info!(
            balance = self.balance,
            open_positions = self.positions.len(),
            total_trades = self.trade_count,
            "=== Итоги сессии ==="
        );
    }
}
```

## Форматы вывода

```rust
use tracing_subscriber::fmt;

fn setup_json_logging() {
    // JSON формат для машинной обработки
    fmt()
        .json()
        .init();
}

fn setup_compact_logging() {
    // Компактный формат для консоли
    fmt()
        .compact()
        .init();
}

fn setup_pretty_logging() {
    // Красивый формат для разработки
    fmt()
        .pretty()
        .init();
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|----------------------|
| События (Events) | Одномоментные записи | Исполнение ордера, получение цены |
| Спаны (Spans) | Временные интервалы | Жизненный цикл сделки |
| Поля (Fields) | Структурированные данные | Цена, объём, тикер, PnL |
| Уровни | Важность события | Ошибки > Предупреждения > Информация |
| Подписчики | Обработка событий | Вывод в консоль, файл, систему мониторинга |

## Практические задания

1. **Базовое логирование**: Добавь структурированное логирование в функцию расчёта PnL с полями: entry_price, exit_price, quantity, gross_pnl, fees, net_pnl.

2. **Спаны для сделок**: Создай функцию, которая открывает спан для каждой сделки и логирует все этапы: валидация → исполнение → подтверждение.

3. **Фильтрация по модулям**: Настрой логирование так, чтобы модуль `risk_management` выводил DEBUG, а остальные только INFO.

4. **Асинхронный мониторинг**: Напиши async функцию мониторинга портфеля, которая каждые N секунд логирует текущую стоимость с использованием `#[instrument]`.

## Домашнее задание

1. Создай структуру `TradeLogger`, которая оборачивает tracing и предоставляет методы: `log_order_placed`, `log_order_filled`, `log_order_cancelled`, `log_position_opened`, `log_position_closed`.

2. Реализуй middleware для логирования всех входящих рыночных данных (цена, объём, время) с автоматическим созданием спанов для каждого тикера.

3. Напиши функцию `setup_production_logging()`, которая настраивает:
   - JSON формат для stdout
   - Фильтрацию по переменной окружения
   - Включение thread_id и timestamp

4. Создай систему алертов на основе tracing: если PnL падает ниже порога, должно генерироваться событие уровня ERROR с полной информацией о позиции.

## Навигация

← Предыдущий день | Следующий день →

*Примечание: Ссылки на соседние главы будут добавлены после их создания.*
