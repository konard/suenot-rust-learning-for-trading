# День 354: Логи в продакшене

## Аналогия из трейдинга

Представь, что ты управляешь торговым залом крупной биржи. У тебя сотни трейдеров, тысячи сделок в секунду, и ты должен знать всё, что происходит — но не можешь лично следить за каждым экраном.

**Логирование — это как система записи всех действий на торговом полу:**

| Торговый зал | Логирование в продакшене |
|--------------|-------------------------|
| **Запись на камеры** | DEBUG логи — детальная запись всего |
| **Голосовые объявления** | INFO логи — важные события для всех |
| **Сигналы тревоги** | WARN логи — потенциальные проблемы |
| **Пожарная сигнализация** | ERROR логи — критические сбои |
| **Журнал сделок** | Структурированные логи для аудита |

Хороший трейдер всегда ведёт журнал сделок. Хорошая торговая система всегда ведёт логи.

## Уровни логирования

В Rust для логирования используется крейт `tracing` — он гораздо мощнее стандартного `log`:

```rust
use tracing::{trace, debug, info, warn, error, Level};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

/// Уровни логирования от наименее до наиболее критичного:
/// TRACE < DEBUG < INFO < WARN < ERROR

fn demonstrate_log_levels() {
    // TRACE — самый детальный уровень, для отладки внутренней логики
    trace!(
        "Итерация {}: проверяем условие входа, price={}, threshold={}",
        42, 50000.0, 49500.0
    );

    // DEBUG — отладочная информация для разработчиков
    debug!(
        price = 50000.0,
        volume = 1.5,
        "Получены данные о рынке"
    );

    // INFO — важные события в нормальной работе системы
    info!(
        order_id = "ORD-12345",
        symbol = "BTCUSDT",
        side = "BUY",
        "Ордер исполнен"
    );

    // WARN — потенциальные проблемы, которые не остановили работу
    warn!(
        latency_ms = 250,
        threshold_ms = 100,
        "Высокая задержка API"
    );

    // ERROR — ошибки, которые повлияли на работу системы
    error!(
        error = "Connection refused",
        exchange = "binance",
        "Не удалось подключиться к бирже"
    );
}

fn main() {
    // Инициализация подписчика с фильтром уровней
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_target(true)
                .with_thread_ids(true)
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(Level::DEBUG))
        .init();

    demonstrate_log_levels();
}
```

## Структурированное логирование для трейдинга

Структурированные логи — это логи с метаданными, которые легко парсить и анализировать:

```rust
use tracing::{info, warn, error, instrument, span, Level};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trade {
    id: String,
    order_id: String,
    symbol: String,
    price: f64,
    quantity: f64,
    commission: f64,
    timestamp: u64,
}

/// Используем #[instrument] для автоматического создания span с параметрами
#[instrument(
    level = "info",
    name = "place_order",
    skip(order),
    fields(
        order_id = %order.id,
        symbol = %order.symbol,
        side = %order.side,
    )
)]
async fn place_order(order: Order) -> Result<Trade, String> {
    info!(
        price = order.price,
        quantity = order.quantity,
        "Отправка ордера на биржу"
    );

    // Имитация отправки ордера
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Проверка риск-менеджмента
    if order.quantity * order.price > 100000.0 {
        warn!(
            order_value = order.quantity * order.price,
            max_allowed = 100000.0,
            "Размер ордера превышает лимит"
        );
        return Err("Превышен максимальный размер ордера".to_string());
    }

    let trade = Trade {
        id: format!("TRD-{}", uuid::Uuid::new_v4()),
        order_id: order.id.clone(),
        symbol: order.symbol.clone(),
        price: order.price,
        quantity: order.quantity,
        commission: order.price * order.quantity * 0.001,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    info!(
        trade_id = %trade.id,
        commission = trade.commission,
        "Ордер исполнен"
    );

    Ok(trade)
}

/// Логирование с контекстом сессии
#[instrument(
    level = "info",
    name = "trading_session",
    fields(session_id = %session_id, user = %user)
)]
async fn run_trading_session(session_id: &str, user: &str) {
    info!("Начало торговой сессии");

    let orders = vec![
        Order {
            id: "ORD-001".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 50000.0,
            quantity: 0.1,
            status: "NEW".to_string(),
        },
        Order {
            id: "ORD-002".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            price: 3000.0,
            quantity: 1.0,
            status: "NEW".to_string(),
        },
    ];

    for order in orders {
        match place_order(order).await {
            Ok(trade) => {
                info!(
                    trade_id = %trade.id,
                    pnl = (trade.price - 49500.0) * trade.quantity - trade.commission,
                    "Сделка записана"
                );
            }
            Err(e) => {
                error!(error = %e, "Ошибка исполнения ордера");
            }
        }
    }

    info!("Торговая сессия завершена");
}

#[tokio::main]
async fn main() {
    // Инициализация JSON-логирования для продакшена
    tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_current_span(true)
        .with_span_list(true)
        .init();

    run_trading_session("SESSION-2024-001", "trader_bot_1").await;
}
```

## Ротация логов и производительность

В продакшене логи должны ротироваться, чтобы не занимать всё дисковое пространство:

```rust
use tracing::{info, Level};
use tracing_subscriber::{self, prelude::*};
use tracing_appender::{
    rolling::{RollingFileAppender, Rotation},
    non_blocking,
};

fn setup_production_logging() {
    // Ротация логов по размеру или времени
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,  // Новый файл каждый день
        "/var/log/trading",
        "trading-bot.log"
    );

    // Неблокирующее логирование для высокой производительности
    let (non_blocking_appender, _guard) = non_blocking(file_appender);

    // Настройка подписчика с несколькими слоями
    tracing_subscriber::registry()
        // Консольный вывод для разработки
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
        )
        // Файловый вывод в JSON
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking_appender)
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG)
        )
        .init();

    info!("Логирование инициализировано");
}

/// Демонстрация влияния логирования на производительность
fn benchmark_logging() {
    use std::time::Instant;

    let iterations = 100_000;

    // Замер времени с логированием
    let start = Instant::now();
    for i in 0..iterations {
        tracing::trace!(iteration = i, "Высокочастотная операция");
    }
    let with_trace = start.elapsed();

    // Замер с info (меньше логов)
    let start = Instant::now();
    for i in 0..iterations {
        if i % 1000 == 0 {
            tracing::info!(iteration = i, "Периодический отчёт");
        }
    }
    let with_info = start.elapsed();

    println!("С TRACE логами: {:?}", with_trace);
    println!("С INFO логами (каждые 1000): {:?}", with_info);
}

fn main() {
    setup_production_logging();
    benchmark_logging();
}
```

## Контекстное логирование для отслеживания ордеров

Когда ордер проходит через множество компонентов, важно отслеживать его путь:

```rust
use tracing::{info, warn, error, instrument, Span};
use tracing::field::Empty;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Генератор уникальных ID для отслеживания запросов
fn generate_trace_id() -> String {
    format!("trace-{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
}

/// Контекст отслеживания для ордера
#[derive(Clone, Debug)]
struct OrderContext {
    trace_id: String,
    order_id: String,
    symbol: String,
    user_id: String,
}

/// Модуль валидации ордера
#[instrument(
    name = "validate_order",
    skip(ctx),
    fields(
        trace_id = %ctx.trace_id,
        order_id = %ctx.order_id,
        validation_result = Empty
    )
)]
async fn validate_order(ctx: &OrderContext, price: f64, quantity: f64) -> Result<(), String> {
    info!("Начало валидации ордера");

    // Проверка минимального объёма
    if quantity < 0.001 {
        warn!(quantity = quantity, min_quantity = 0.001, "Объём ниже минимума");
        Span::current().record("validation_result", "REJECTED_MIN_QTY");
        return Err("Объём ниже минимума".to_string());
    }

    // Проверка максимальной цены
    if price > 1_000_000.0 {
        warn!(price = price, max_price = 1_000_000.0, "Цена выше максимума");
        Span::current().record("validation_result", "REJECTED_MAX_PRICE");
        return Err("Цена выше максимума".to_string());
    }

    Span::current().record("validation_result", "PASSED");
    info!("Валидация пройдена");
    Ok(())
}

/// Модуль проверки баланса
#[instrument(
    name = "check_balance",
    skip(ctx),
    fields(trace_id = %ctx.trace_id, order_id = %ctx.order_id)
)]
async fn check_balance(ctx: &OrderContext, required: f64) -> Result<(), String> {
    info!(required_balance = required, "Проверка баланса");

    // Имитация запроса к базе данных
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let available_balance = 100_000.0; // Имитация

    if required > available_balance {
        error!(
            required = required,
            available = available_balance,
            "Недостаточно средств"
        );
        return Err("Недостаточно средств".to_string());
    }

    info!(available_balance = available_balance, "Баланс достаточен");
    Ok(())
}

/// Модуль отправки на биржу
#[instrument(
    name = "send_to_exchange",
    skip(ctx),
    fields(
        trace_id = %ctx.trace_id,
        order_id = %ctx.order_id,
        exchange_order_id = Empty
    )
)]
async fn send_to_exchange(
    ctx: &OrderContext,
    price: f64,
    quantity: f64
) -> Result<String, String> {
    info!(exchange = "binance", "Отправка ордера на биржу");

    // Имитация сетевого запроса
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let exchange_order_id = format!("EX-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    Span::current().record("exchange_order_id", &exchange_order_id);

    info!(
        exchange_order_id = %exchange_order_id,
        latency_ms = 50,
        "Ордер принят биржей"
    );

    Ok(exchange_order_id)
}

/// Полный цикл обработки ордера с трейсингом
#[instrument(
    name = "process_order",
    fields(
        trace_id = %generate_trace_id(),
        order_id = %order_id,
        symbol = %symbol,
        total_latency_ms = Empty
    )
)]
async fn process_order(
    order_id: &str,
    symbol: &str,
    user_id: &str,
    price: f64,
    quantity: f64,
) -> Result<String, String> {
    let start = std::time::Instant::now();

    let ctx = OrderContext {
        trace_id: Span::current()
            .field("trace_id")
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        order_id: order_id.to_string(),
        symbol: symbol.to_string(),
        user_id: user_id.to_string(),
    };

    info!(
        user_id = %user_id,
        price = price,
        quantity = quantity,
        "Начало обработки ордера"
    );

    // Этап 1: Валидация
    validate_order(&ctx, price, quantity).await?;

    // Этап 2: Проверка баланса
    let required = price * quantity;
    check_balance(&ctx, required).await?;

    // Этап 3: Отправка на биржу
    let exchange_id = send_to_exchange(&ctx, price, quantity).await?;

    let total_latency = start.elapsed().as_millis();
    Span::current().record("total_latency_ms", total_latency);

    info!(
        exchange_id = %exchange_id,
        total_latency_ms = total_latency,
        "Ордер успешно обработан"
    );

    Ok(exchange_id)
}

#[tokio::main]
async fn main() {
    // Инициализация с форматом для анализа
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    println!("=== Обработка ордера с полным трейсингом ===\n");

    match process_order(
        "ORD-001",
        "BTCUSDT",
        "trader_123",
        50000.0,
        0.1
    ).await {
        Ok(exchange_id) => println!("\nУспех! Exchange ID: {}", exchange_id),
        Err(e) => println!("\nОшибка: {}", e),
    }
}
```

## Логирование метрик для мониторинга

Логи можно использовать для сбора метрик производительности:

```rust
use tracing::{info, warn, Span};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

/// Счётчики для метрик
struct TradingMetrics {
    orders_total: AtomicU64,
    orders_success: AtomicU64,
    orders_failed: AtomicU64,
    total_volume: AtomicU64, // В сотых долях (центы)
    total_latency_ms: AtomicU64,
}

impl TradingMetrics {
    fn new() -> Self {
        TradingMetrics {
            orders_total: AtomicU64::new(0),
            orders_success: AtomicU64::new(0),
            orders_failed: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    fn record_order(&self, success: bool, volume: f64, latency_ms: u64) {
        self.orders_total.fetch_add(1, Ordering::Relaxed);

        if success {
            self.orders_success.fetch_add(1, Ordering::Relaxed);
            // Конвертируем в центы для атомарного хранения
            self.total_volume.fetch_add((volume * 100.0) as u64, Ordering::Relaxed);
        } else {
            self.orders_failed.fetch_add(1, Ordering::Relaxed);
        }

        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    fn log_summary(&self) {
        let total = self.orders_total.load(Ordering::Relaxed);
        let success = self.orders_success.load(Ordering::Relaxed);
        let failed = self.orders_failed.load(Ordering::Relaxed);
        let volume = self.total_volume.load(Ordering::Relaxed) as f64 / 100.0;
        let latency = self.total_latency_ms.load(Ordering::Relaxed);

        let avg_latency = if total > 0 { latency / total } else { 0 };
        let success_rate = if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 };

        info!(
            orders_total = total,
            orders_success = success,
            orders_failed = failed,
            success_rate_pct = format!("{:.2}", success_rate),
            total_volume_usd = format!("{:.2}", volume),
            avg_latency_ms = avg_latency,
            "Сводка торговых метрик"
        );

        // Предупреждение при низком success rate
        if total > 10 && success_rate < 95.0 {
            warn!(
                success_rate = format!("{:.2}%", success_rate),
                threshold = "95%",
                "Низкий процент успешных ордеров"
            );
        }

        // Предупреждение при высокой латентности
        if avg_latency > 200 {
            warn!(
                avg_latency_ms = avg_latency,
                threshold_ms = 200,
                "Высокая средняя латентность"
            );
        }
    }
}

/// Обёртка для измерения времени выполнения
struct TimedOperation {
    name: String,
    start: Instant,
}

impl TimedOperation {
    fn start(name: &str) -> Self {
        TimedOperation {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    fn finish(self) -> u64 {
        let elapsed = self.start.elapsed().as_millis() as u64;
        info!(
            operation = %self.name,
            duration_ms = elapsed,
            "Операция завершена"
        );
        elapsed
    }
}

async fn simulate_trading(metrics: Arc<TradingMetrics>) {
    for i in 0..20 {
        let timer = TimedOperation::start(&format!("order_{}", i));

        // Имитация обработки ордера
        tokio::time::sleep(std::time::Duration::from_millis(20 + (i * 5) as u64)).await;

        let latency = timer.finish();

        // Имитация: 90% успешных ордеров
        let success = i % 10 != 0;
        let volume = 1000.0 + (i as f64 * 100.0);

        metrics.record_order(success, volume, latency);

        // Логируем сводку каждые 5 ордеров
        if (i + 1) % 5 == 0 {
            metrics.log_summary();
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Логирование метрик торговли ===\n");

    let metrics = Arc::new(TradingMetrics::new());

    simulate_trading(metrics.clone()).await;

    println!("\n=== Финальная сводка ===\n");
    metrics.log_summary();
}
```

## Фильтрация логов по компонентам

В большой системе важно уметь фильтровать логи по модулям:

```rust
use tracing::{info, debug, warn, Level};
use tracing_subscriber::{self, filter::EnvFilter, prelude::*};

mod market_data {
    use tracing::{info, debug};

    pub fn process_tick(symbol: &str, price: f64) {
        debug!(symbol = %symbol, price = price, "Получен тик");
        // Обработка...
        info!(symbol = %symbol, "Тик обработан");
    }
}

mod order_manager {
    use tracing::{info, warn};

    pub fn place_order(order_id: &str, price: f64) {
        info!(order_id = %order_id, price = price, "Размещение ордера");

        if price > 100000.0 {
            warn!(order_id = %order_id, "Высокая цена ордера");
        }
    }
}

mod risk_manager {
    use tracing::{info, warn, error};

    pub fn check_risk(position_size: f64) -> bool {
        info!(position_size = position_size, "Проверка риска");

        if position_size > 10.0 {
            warn!(position_size = position_size, max = 10.0, "Размер позиции превышает лимит");
            return false;
        }

        true
    }
}

fn main() {
    // Фильтрация логов через переменную окружения RUST_LOG
    // Примеры:
    // RUST_LOG=debug - все debug логи
    // RUST_LOG=market_data=debug,order_manager=info - разные уровни для модулей
    // RUST_LOG=warn,risk_manager=debug - warn по умолчанию, debug для risk_manager

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Значение по умолчанию если RUST_LOG не установлен
            EnvFilter::new("info")
                .add_directive("market_data=debug".parse().unwrap())
                .add_directive("risk_manager=debug".parse().unwrap())
        });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    println!("=== Фильтрация логов по модулям ===\n");
    println!("Установите RUST_LOG для изменения фильтрации");
    println!("Например: RUST_LOG=market_data=debug,order_manager=warn\n");

    // Демонстрация логирования из разных модулей
    market_data::process_tick("BTCUSDT", 50000.0);
    order_manager::place_order("ORD-001", 50000.0);
    order_manager::place_order("ORD-002", 150000.0); // Высокая цена

    let risk_ok = risk_manager::check_risk(5.0);
    info!(approved = risk_ok, "Результат проверки риска");

    let risk_exceeded = risk_manager::check_risk(15.0);
    info!(approved = risk_exceeded, "Результат проверки риска");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Уровни логов** | TRACE, DEBUG, INFO, WARN, ERROR — от детального до критического |
| **tracing** | Современный фреймворк для структурированного логирования в Rust |
| **Span** | Контекст выполнения, объединяющий связанные логи |
| **#[instrument]** | Макрос для автоматического создания span с параметрами функции |
| **Структурированные логи** | Логи с метаданными для машинного парсинга |
| **Ротация логов** | Автоматическое создание новых файлов по времени или размеру |
| **Неблокирующее логирование** | Запись логов без блокировки основного потока |
| **EnvFilter** | Фильтрация логов через переменную окружения RUST_LOG |

## Практические задания

1. **Система аудита сделок**: Создай систему, которая:
   - Логирует каждую сделку со всеми деталями
   - Использует span для группировки связанных событий
   - Выводит логи в JSON формате для анализа
   - Включает trace_id для отслеживания запросов

2. **Мониторинг производительности**: Реализуй систему:
   - Измеряет время выполнения каждой операции
   - Логирует предупреждения при превышении порогов
   - Собирает статистику в структурированном виде
   - Генерирует периодические отчёты

3. **Многоуровневое логирование**: Создай конфигурацию:
   - Консольный вывод для разработки (цветной, читаемый)
   - Файловый вывод в JSON для продакшена
   - Отдельные файлы для ошибок
   - Динамическое изменение уровня через RUST_LOG

4. **Интеграция с мониторингом**: Реализуй:
   - Экспорт логов в формате для Elasticsearch
   - Добавление кастомных метрик в логи
   - Алерты на критические события
   - Dashboard с агрегированной статистикой

## Домашнее задание

1. **Система логирования для торгового бота**: Создай систему:
   - Полный аудит всех торговых операций
   - Разные уровни для разных модулей (market_data=debug, risk=info)
   - Ротация логов ежедневно с сжатием старых файлов
   - Поиск по логам через trace_id
   - Метрики: количество ордеров, success rate, средняя латентность

2. **Анализатор логов**: Напиши инструмент:
   - Парсит JSON логи торговой системы
   - Находит паттерны ошибок (повторяющиеся проблемы)
   - Строит временную шкалу событий для конкретного ордера
   - Выявляет аномалии в латентности
   - Генерирует отчёт в markdown формате

3. **Real-time мониторинг**: Создай dashboard:
   - Стримит логи в реальном времени через WebSocket
   - Фильтрация по уровню и модулю на клиенте
   - Подсветка ошибок и предупреждений
   - Графики метрик (ордера/сек, латентность, ошибки)
   - Алерты в Telegram/Slack при критических ошибках

4. **Распределённый трейсинг**: Реализуй систему:
   - Передача trace_id между микросервисами
   - Сбор логов со всех сервисов в единое хранилище
   - Визуализация пути запроса через систему
   - Измерение латентности на каждом этапе
   - Интеграция с OpenTelemetry

## Навигация

[← Предыдущий день](../353-production-monitoring/ru.md) | [Следующий день →](../355-secret-rotation/ru.md)
