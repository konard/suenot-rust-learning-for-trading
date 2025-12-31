# День 340: Metrics: Prometheus метрики

## Аналогия из трейдинга

Представь, что ты управляешь профессиональным торговым залом. У тебя работают десятки трейдеров, каждый за своим терминалом. Как понять, насколько эффективно работает вся система?

**Без мониторинга:**
Ты ходишь между столами и спрашиваешь каждого трейдера: "Сколько сделок за час?", "Какой средний profit?", "Есть ли проблемы с подключением?". Это медленно, неточно и отвлекает от работы.

**С Prometheus метриками:**
У каждого терминала есть панель с ключевыми показателями. Центральный экран показывает агрегированные данные по всему залу в реальном времени. Ты видишь:
- Количество ордеров в секунду по каждой бирже
- Среднюю задержку исполнения
- Текущий P&L по всем стратегиям
- Алерты при отклонениях от нормы

| Концепция | В трейдинге | В Prometheus |
|-----------|-------------|--------------|
| **Counter** | Общее количество сделок | Счётчик, который только растёт |
| **Gauge** | Текущая цена актива | Значение, которое может расти и падать |
| **Histogram** | Распределение задержек исполнения | Гистограмма с бакетами |
| **Summary** | Перцентили P&L | Статистика с квантилями |

## Что такое Prometheus?

Prometheus — это система мониторинга с открытым исходным кодом:
- **Pull-based** — Prometheus сам забирает метрики с ваших сервисов
- **Time-series DB** — хранит данные с временными метками
- **PromQL** — мощный язык запросов для анализа
- **Alerting** — встроенная система алертов

### Основные типы метрик

```rust
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec,
    Histogram, HistogramVec, HistogramOpts,
    Registry, Opts, labels,
};
use std::time::Instant;

/// Метрики торговой системы
struct TradingMetrics {
    /// Счётчик: общее количество ордеров
    orders_total: CounterVec,

    /// Счётчик: общий объём торгов в USD
    volume_usd: CounterVec,

    /// Gauge: текущая цена
    current_price: GaugeVec,

    /// Gauge: текущая позиция
    position_size: GaugeVec,

    /// Histogram: время исполнения ордера
    order_latency: HistogramVec,

    /// Histogram: размер ордеров
    order_size: HistogramVec,
}

impl TradingMetrics {
    fn new(registry: &Registry) -> Self {
        let orders_total = CounterVec::new(
            Opts::new("orders_total", "Total number of orders placed"),
            &["exchange", "symbol", "side"],
        ).unwrap();

        let volume_usd = CounterVec::new(
            Opts::new("volume_usd_total", "Total trading volume in USD"),
            &["exchange", "symbol"],
        ).unwrap();

        let current_price = GaugeVec::new(
            Opts::new("current_price", "Current price of the asset"),
            &["exchange", "symbol"],
        ).unwrap();

        let position_size = GaugeVec::new(
            Opts::new("position_size", "Current position size"),
            &["symbol", "strategy"],
        ).unwrap();

        // Histogram с бакетами для задержки (в миллисекундах)
        let latency_buckets = vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
        let order_latency = HistogramVec::new(
            HistogramOpts::new("order_latency_ms", "Order execution latency in milliseconds")
                .buckets(latency_buckets),
            &["exchange", "order_type"],
        ).unwrap();

        // Histogram для размеров ордеров
        let size_buckets = vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0];
        let order_size = HistogramVec::new(
            HistogramOpts::new("order_size", "Order size distribution")
                .buckets(size_buckets),
            &["symbol"],
        ).unwrap();

        // Регистрируем метрики
        registry.register(Box::new(orders_total.clone())).unwrap();
        registry.register(Box::new(volume_usd.clone())).unwrap();
        registry.register(Box::new(current_price.clone())).unwrap();
        registry.register(Box::new(position_size.clone())).unwrap();
        registry.register(Box::new(order_latency.clone())).unwrap();
        registry.register(Box::new(order_size.clone())).unwrap();

        TradingMetrics {
            orders_total,
            volume_usd,
            current_price,
            position_size,
            order_latency,
            order_size,
        }
    }

    /// Записать размещение ордера
    fn record_order(&self, exchange: &str, symbol: &str, side: &str, size: f64, price: f64) {
        self.orders_total
            .with_label_values(&[exchange, symbol, side])
            .inc();

        self.volume_usd
            .with_label_values(&[exchange, symbol])
            .inc_by(size * price);

        self.order_size
            .with_label_values(&[symbol])
            .observe(size);
    }

    /// Записать задержку исполнения
    fn record_latency(&self, exchange: &str, order_type: &str, latency_ms: f64) {
        self.order_latency
            .with_label_values(&[exchange, order_type])
            .observe(latency_ms);
    }

    /// Обновить текущую цену
    fn update_price(&self, exchange: &str, symbol: &str, price: f64) {
        self.current_price
            .with_label_values(&[exchange, symbol])
            .set(price);
    }

    /// Обновить размер позиции
    fn update_position(&self, symbol: &str, strategy: &str, size: f64) {
        self.position_size
            .with_label_values(&[symbol, strategy])
            .set(size);
    }
}

fn main() {
    println!("=== Prometheus метрики для трейдинга ===\n");

    let registry = Registry::new();
    let metrics = TradingMetrics::new(&registry);

    // Симуляция торговой активности
    let exchanges = ["binance", "kraken", "coinbase"];
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];

    // Записываем ордера
    for (i, exchange) in exchanges.iter().enumerate() {
        for (j, symbol) in symbols.iter().enumerate() {
            let price = 50000.0 + (i as f64 * 1000.0) - (j as f64 * 5000.0);
            let size = 0.1 + (j as f64 * 0.05);

            metrics.update_price(exchange, symbol, price);

            // Несколько ордеров
            for _ in 0..5 {
                metrics.record_order(exchange, symbol, "buy", size, price);
                metrics.record_latency(exchange, "limit", 15.0 + (i as f64 * 5.0));
            }

            metrics.update_position(symbol, "momentum", size * 3.0);
        }
    }

    println!("Метрики записаны. Пример вывода:\n");

    // Вывод метрик в формате Prometheus
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();

    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    // Показываем первые 50 строк
    for (i, line) in output.lines().enumerate() {
        if i < 50 {
            println!("{}", line);
        }
    }
    println!("...(и другие метрики)");
}
```

## HTTP эндпоинт для Prometheus

Prometheus забирает метрики по HTTP. Создадим сервер:

```rust
use prometheus::{Registry, Counter, Gauge, Histogram, HistogramOpts, Encoder, TextEncoder};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Состояние торговой системы с метриками
struct TradingSystem {
    registry: Registry,
    orders_processed: Counter,
    active_connections: Gauge,
    order_latency: Histogram,
    last_prices: Arc<RwLock<std::collections::HashMap<String, f64>>>,
}

impl TradingSystem {
    fn new() -> Self {
        let registry = Registry::new();

        let orders_processed = Counter::new(
            "trading_orders_processed_total",
            "Total number of processed orders"
        ).unwrap();

        let active_connections = Gauge::new(
            "trading_active_connections",
            "Number of active exchange connections"
        ).unwrap();

        let latency_buckets = vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0];
        let order_latency = Histogram::with_opts(
            HistogramOpts::new("trading_order_latency_ms", "Order processing latency")
                .buckets(latency_buckets)
        ).unwrap();

        registry.register(Box::new(orders_processed.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(order_latency.clone())).unwrap();

        TradingSystem {
            registry,
            orders_processed,
            active_connections,
            order_latency,
            last_prices: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn process_order(&self, symbol: &str, price: f64, quantity: f64) {
        let start = Instant::now();

        // Имитация обработки
        tokio::time::sleep(Duration::from_millis(10 + (price as u64 % 50))).await;

        // Обновляем цену
        {
            let mut prices = self.last_prices.write().await;
            prices.insert(symbol.to_string(), price);
        }

        // Записываем метрики
        let latency = start.elapsed().as_millis() as f64;
        self.order_latency.observe(latency);
        self.orders_processed.inc();

        println!("Обработан ордер: {} @ {:.2}, задержка: {:.1}ms", symbol, price, latency);
    }

    fn connect_exchange(&self, exchange: &str) {
        self.active_connections.inc();
        println!("Подключение к {}", exchange);
    }

    fn disconnect_exchange(&self, exchange: &str) {
        self.active_connections.dec();
        println!("Отключение от {}", exchange);
    }

    fn get_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        String::from_utf8(buffer).unwrap()
    }
}

// HTTP сервер с использованием warp
// use warp::Filter;
//
// #[tokio::main]
// async fn main() {
//     let system = Arc::new(TradingSystem::new());
//
//     // Эндпоинт для метрик
//     let metrics_route = warp::path("metrics")
//         .map({
//             let system = Arc::clone(&system);
//             move || system.get_metrics()
//         });
//
//     // Эндпоинт для здоровья
//     let health_route = warp::path("health")
//         .map(|| "OK");
//
//     let routes = metrics_route.or(health_route);
//
//     println!("Сервер метрик запущен на http://localhost:9090");
//     warp::serve(routes).run(([127, 0, 0, 1], 9090)).await;
// }

#[tokio::main]
async fn main() {
    println!("=== HTTP эндпоинт для Prometheus ===\n");

    let system = TradingSystem::new();

    // Симуляция подключений
    system.connect_exchange("binance");
    system.connect_exchange("kraken");

    // Симуляция обработки ордеров
    for i in 0..10 {
        let symbol = if i % 2 == 0 { "BTCUSDT" } else { "ETHUSDT" };
        let price = 50000.0 + (i as f64 * 100.0);
        system.process_order(symbol, price, 0.1).await;
    }

    // Отключаем одну биржу
    system.disconnect_exchange("kraken");

    println!("\n=== Текущие метрики ===\n");
    println!("{}", system.get_metrics());
}
```

## Метрики для торговых стратегий

```rust
use prometheus::{
    Registry, CounterVec, GaugeVec, HistogramVec, HistogramOpts, Opts,
};
use std::collections::HashMap;
use std::time::Instant;

/// Метрики эффективности стратегии
struct StrategyMetrics {
    /// Количество сигналов
    signals_generated: CounterVec,

    /// Количество исполненных сделок
    trades_executed: CounterVec,

    /// Текущий P&L
    unrealized_pnl: GaugeVec,

    /// Реализованный P&L
    realized_pnl: CounterVec,

    /// Распределение прибыльности сделок
    trade_pnl_distribution: HistogramVec,

    /// Win rate (как Gauge для текущего значения)
    win_rate: GaugeVec,

    /// Sharpe ratio
    sharpe_ratio: GaugeVec,

    /// Максимальная просадка
    max_drawdown: GaugeVec,
}

impl StrategyMetrics {
    fn new(registry: &Registry) -> Self {
        let signals_generated = CounterVec::new(
            Opts::new("strategy_signals_total", "Total signals generated"),
            &["strategy", "signal_type"],
        ).unwrap();

        let trades_executed = CounterVec::new(
            Opts::new("strategy_trades_total", "Total trades executed"),
            &["strategy", "symbol", "side"],
        ).unwrap();

        let unrealized_pnl = GaugeVec::new(
            Opts::new("strategy_unrealized_pnl", "Current unrealized P&L"),
            &["strategy", "symbol"],
        ).unwrap();

        let realized_pnl = CounterVec::new(
            Opts::new("strategy_realized_pnl_total", "Total realized P&L"),
            &["strategy"],
        ).unwrap();

        // Бакеты для P&L: от -1000 до +1000
        let pnl_buckets = vec![
            -1000.0, -500.0, -100.0, -50.0, -10.0, 0.0,
            10.0, 50.0, 100.0, 500.0, 1000.0
        ];
        let trade_pnl_distribution = HistogramVec::new(
            HistogramOpts::new("strategy_trade_pnl", "P&L distribution per trade")
                .buckets(pnl_buckets),
            &["strategy"],
        ).unwrap();

        let win_rate = GaugeVec::new(
            Opts::new("strategy_win_rate", "Current win rate (0-1)"),
            &["strategy"],
        ).unwrap();

        let sharpe_ratio = GaugeVec::new(
            Opts::new("strategy_sharpe_ratio", "Current Sharpe ratio"),
            &["strategy"],
        ).unwrap();

        let max_drawdown = GaugeVec::new(
            Opts::new("strategy_max_drawdown", "Maximum drawdown percentage"),
            &["strategy"],
        ).unwrap();

        // Регистрация
        registry.register(Box::new(signals_generated.clone())).unwrap();
        registry.register(Box::new(trades_executed.clone())).unwrap();
        registry.register(Box::new(unrealized_pnl.clone())).unwrap();
        registry.register(Box::new(realized_pnl.clone())).unwrap();
        registry.register(Box::new(trade_pnl_distribution.clone())).unwrap();
        registry.register(Box::new(win_rate.clone())).unwrap();
        registry.register(Box::new(sharpe_ratio.clone())).unwrap();
        registry.register(Box::new(max_drawdown.clone())).unwrap();

        StrategyMetrics {
            signals_generated,
            trades_executed,
            unrealized_pnl,
            realized_pnl,
            trade_pnl_distribution,
            win_rate,
            sharpe_ratio,
            max_drawdown,
        }
    }
}

/// Трекер стратегии с метриками
struct StrategyTracker {
    name: String,
    metrics: StrategyMetrics,
    trades: Vec<Trade>,
    wins: u64,
    losses: u64,
    peak_equity: f64,
    current_equity: f64,
}

struct Trade {
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: Option<f64>,
    size: f64,
    pnl: Option<f64>,
}

impl StrategyTracker {
    fn new(name: &str, metrics: StrategyMetrics) -> Self {
        StrategyTracker {
            name: name.to_string(),
            metrics,
            trades: Vec::new(),
            wins: 0,
            losses: 0,
            peak_equity: 10000.0,  // Начальный капитал
            current_equity: 10000.0,
        }
    }

    fn generate_signal(&self, signal_type: &str) {
        self.metrics.signals_generated
            .with_label_values(&[&self.name, signal_type])
            .inc();

        println!("[{}] Сигнал: {}", self.name, signal_type);
    }

    fn open_trade(&mut self, symbol: &str, side: &str, price: f64, size: f64) {
        let trade = Trade {
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price: price,
            exit_price: None,
            size,
            pnl: None,
        };

        self.trades.push(trade);

        self.metrics.trades_executed
            .with_label_values(&[&self.name, symbol, side])
            .inc();

        println!("[{}] Открыта позиция: {} {} {} @ {:.2}",
                 self.name, side, size, symbol, price);
    }

    fn close_trade(&mut self, symbol: &str, exit_price: f64) {
        if let Some(trade) = self.trades.iter_mut()
            .find(|t| t.symbol == symbol && t.exit_price.is_none())
        {
            trade.exit_price = Some(exit_price);

            // Расчёт P&L
            let pnl = if trade.side == "buy" {
                (exit_price - trade.entry_price) * trade.size
            } else {
                (trade.entry_price - exit_price) * trade.size
            };

            trade.pnl = Some(pnl);

            // Обновляем equity
            self.current_equity += pnl;
            if self.current_equity > self.peak_equity {
                self.peak_equity = self.current_equity;
            }

            // Обновляем win/loss
            if pnl > 0.0 {
                self.wins += 1;
            } else {
                self.losses += 1;
            }

            // Записываем метрики
            self.metrics.trade_pnl_distribution
                .with_label_values(&[&self.name])
                .observe(pnl);

            if pnl > 0.0 {
                self.metrics.realized_pnl
                    .with_label_values(&[&self.name])
                    .inc_by(pnl);
            }

            // Обновляем win rate
            let total_trades = self.wins + self.losses;
            if total_trades > 0 {
                let win_rate = self.wins as f64 / total_trades as f64;
                self.metrics.win_rate
                    .with_label_values(&[&self.name])
                    .set(win_rate);
            }

            // Обновляем максимальную просадку
            let drawdown = (self.peak_equity - self.current_equity) / self.peak_equity * 100.0;
            self.metrics.max_drawdown
                .with_label_values(&[&self.name])
                .set(drawdown);

            println!("[{}] Закрыта позиция: {} @ {:.2}, P&L: {:.2}, Win Rate: {:.1}%",
                     self.name, symbol, exit_price, pnl,
                     self.wins as f64 / (self.wins + self.losses) as f64 * 100.0);
        }
    }

    fn update_unrealized_pnl(&self, symbol: &str, current_price: f64) {
        if let Some(trade) = self.trades.iter()
            .find(|t| t.symbol == symbol && t.exit_price.is_none())
        {
            let unrealized = if trade.side == "buy" {
                (current_price - trade.entry_price) * trade.size
            } else {
                (trade.entry_price - current_price) * trade.size
            };

            self.metrics.unrealized_pnl
                .with_label_values(&[&self.name, symbol])
                .set(unrealized);
        }
    }
}

fn main() {
    println!("=== Метрики торговых стратегий ===\n");

    let registry = Registry::new();
    let metrics = StrategyMetrics::new(&registry);

    let mut tracker = StrategyTracker::new("momentum_btc", metrics);

    // Симуляция торговли
    tracker.generate_signal("long");
    tracker.open_trade("BTCUSDT", "buy", 50000.0, 0.1);

    // Обновление unrealized P&L
    tracker.update_unrealized_pnl("BTCUSDT", 51000.0);

    tracker.close_trade("BTCUSDT", 51500.0);

    tracker.generate_signal("short");
    tracker.open_trade("BTCUSDT", "sell", 51500.0, 0.15);
    tracker.close_trade("BTCUSDT", 50500.0);

    // Ещё несколько сделок
    for i in 0..5 {
        tracker.generate_signal("long");
        let entry = 50000.0 + (i as f64 * 100.0);
        tracker.open_trade("BTCUSDT", "buy", entry, 0.1);

        // Некоторые сделки убыточные
        let exit = if i % 3 == 0 {
            entry - 200.0  // Убыток
        } else {
            entry + 300.0  // Прибыль
        };
        tracker.close_trade("BTCUSDT", exit);
    }

    println!("\n=== Итоговые метрики ===\n");

    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();

    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    for line in output.lines() {
        if line.starts_with("strategy_") {
            println!("{}", line);
        }
    }
}
```

## Мониторинг инфраструктуры

```rust
use prometheus::{Registry, GaugeVec, CounterVec, HistogramVec, HistogramOpts, Opts};
use std::time::{Duration, Instant};

/// Метрики инфраструктуры торговой системы
struct InfraMetrics {
    /// Состояние подключений к биржам
    exchange_connection_status: GaugeVec,

    /// Время отклика бирж
    exchange_response_time: HistogramVec,

    /// Количество ошибок по типам
    errors_total: CounterVec,

    /// Использование rate limits
    rate_limit_usage: GaugeVec,

    /// Размер очередей
    queue_size: GaugeVec,

    /// Время обработки в очереди
    queue_latency: HistogramVec,

    /// WebSocket сообщения в секунду
    websocket_messages: CounterVec,

    /// Состояние памяти
    memory_usage_bytes: GaugeVec,
}

impl InfraMetrics {
    fn new(registry: &Registry) -> Self {
        let exchange_connection_status = GaugeVec::new(
            Opts::new("exchange_connection_status", "Exchange connection status (1=connected, 0=disconnected)"),
            &["exchange"],
        ).unwrap();

        let response_buckets = vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 5000.0];
        let exchange_response_time = HistogramVec::new(
            HistogramOpts::new("exchange_response_time_ms", "Exchange API response time")
                .buckets(response_buckets),
            &["exchange", "endpoint"],
        ).unwrap();

        let errors_total = CounterVec::new(
            Opts::new("errors_total", "Total number of errors"),
            &["exchange", "error_type"],
        ).unwrap();

        let rate_limit_usage = GaugeVec::new(
            Opts::new("rate_limit_usage", "Rate limit usage (0-1)"),
            &["exchange"],
        ).unwrap();

        let queue_size = GaugeVec::new(
            Opts::new("queue_size", "Current queue size"),
            &["queue_name"],
        ).unwrap();

        let queue_buckets = vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0];
        let queue_latency = HistogramVec::new(
            HistogramOpts::new("queue_latency_ms", "Time spent in queue")
                .buckets(queue_buckets),
            &["queue_name"],
        ).unwrap();

        let websocket_messages = CounterVec::new(
            Opts::new("websocket_messages_total", "Total WebSocket messages received"),
            &["exchange", "channel"],
        ).unwrap();

        let memory_usage_bytes = GaugeVec::new(
            Opts::new("memory_usage_bytes", "Memory usage in bytes"),
            &["component"],
        ).unwrap();

        // Регистрация
        registry.register(Box::new(exchange_connection_status.clone())).unwrap();
        registry.register(Box::new(exchange_response_time.clone())).unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();
        registry.register(Box::new(rate_limit_usage.clone())).unwrap();
        registry.register(Box::new(queue_size.clone())).unwrap();
        registry.register(Box::new(queue_latency.clone())).unwrap();
        registry.register(Box::new(websocket_messages.clone())).unwrap();
        registry.register(Box::new(memory_usage_bytes.clone())).unwrap();

        InfraMetrics {
            exchange_connection_status,
            exchange_response_time,
            errors_total,
            rate_limit_usage,
            queue_size,
            queue_latency,
            websocket_messages,
            memory_usage_bytes,
        }
    }
}

/// Симулятор подключения к бирже
struct ExchangeConnection {
    name: String,
    metrics: InfraMetrics,
    is_connected: bool,
}

impl ExchangeConnection {
    fn new(name: &str, metrics: InfraMetrics) -> Self {
        ExchangeConnection {
            name: name.to_string(),
            metrics,
            is_connected: false,
        }
    }

    fn connect(&mut self) {
        self.is_connected = true;
        self.metrics.exchange_connection_status
            .with_label_values(&[&self.name])
            .set(1.0);
        println!("[{}] Подключено", self.name);
    }

    fn disconnect(&mut self) {
        self.is_connected = false;
        self.metrics.exchange_connection_status
            .with_label_values(&[&self.name])
            .set(0.0);
        println!("[{}] Отключено", self.name);
    }

    fn api_call(&self, endpoint: &str) -> Result<(), &'static str> {
        let start = Instant::now();

        // Имитация API вызова
        std::thread::sleep(Duration::from_millis(10 + (self.name.len() as u64 * 5)));

        let latency = start.elapsed().as_millis() as f64;
        self.metrics.exchange_response_time
            .with_label_values(&[&self.name, endpoint])
            .observe(latency);

        // Иногда возвращаем ошибку
        if latency > 50.0 {
            self.metrics.errors_total
                .with_label_values(&[&self.name, "timeout"])
                .inc();
            return Err("Timeout");
        }

        println!("[{}] API {}: {:.1}ms", self.name, endpoint, latency);
        Ok(())
    }

    fn receive_websocket_message(&self, channel: &str) {
        self.metrics.websocket_messages
            .with_label_values(&[&self.name, channel])
            .inc();
    }

    fn update_rate_limit(&self, usage: f64) {
        self.metrics.rate_limit_usage
            .with_label_values(&[&self.name])
            .set(usage);

        if usage > 0.8 {
            println!("[{}] ВНИМАНИЕ: Rate limit {:.0}%!", self.name, usage * 100.0);
        }
    }
}

/// Очередь с метриками
struct MeteredQueue {
    name: String,
    items: Vec<(Instant, String)>,
    metrics: InfraMetrics,
}

impl MeteredQueue {
    fn new(name: &str, metrics: InfraMetrics) -> Self {
        MeteredQueue {
            name: name.to_string(),
            items: Vec::new(),
            metrics,
        }
    }

    fn push(&mut self, item: String) {
        self.items.push((Instant::now(), item));
        self.metrics.queue_size
            .with_label_values(&[&self.name])
            .set(self.items.len() as f64);
    }

    fn pop(&mut self) -> Option<String> {
        if let Some((enqueue_time, item)) = self.items.pop() {
            let latency = enqueue_time.elapsed().as_secs_f64() * 1000.0;
            self.metrics.queue_latency
                .with_label_values(&[&self.name])
                .observe(latency);

            self.metrics.queue_size
                .with_label_values(&[&self.name])
                .set(self.items.len() as f64);

            Some(item)
        } else {
            None
        }
    }
}

fn main() {
    println!("=== Мониторинг инфраструктуры ===\n");

    let registry = Registry::new();
    let metrics = InfraMetrics::new(&registry);

    // Создаём подключения к биржам
    let mut binance = ExchangeConnection::new("binance", InfraMetrics::new(&Registry::new()));
    let mut kraken = ExchangeConnection::new("kraken", InfraMetrics::new(&Registry::new()));

    binance.connect();
    kraken.connect();

    // Симуляция API вызовов
    for _ in 0..5 {
        let _ = binance.api_call("/api/v3/ticker");
        let _ = kraken.api_call("/0/public/Ticker");
    }

    // WebSocket сообщения
    for _ in 0..100 {
        binance.receive_websocket_message("trades");
        kraken.receive_websocket_message("book");
    }

    // Rate limits
    binance.update_rate_limit(0.45);
    kraken.update_rate_limit(0.85);

    // Очередь ордеров
    let mut order_queue = MeteredQueue::new("orders", metrics);

    for i in 0..10 {
        order_queue.push(format!("order_{}", i));
    }

    std::thread::sleep(Duration::from_millis(50));

    while order_queue.pop().is_some() {}

    // Отключаем одну биржу
    kraken.disconnect();

    println!("\n=== Инфраструктурные метрики готовы ===");
}
```

## Алерты на основе метрик

```rust
use prometheus::{Registry, Gauge, Counter};
use std::time::{Duration, Instant};

/// Правило алертинга
struct AlertRule {
    name: String,
    condition: Box<dyn Fn(f64) -> bool>,
    threshold: f64,
    duration: Duration,
    triggered_at: Option<Instant>,
    is_firing: bool,
}

impl AlertRule {
    fn new<F>(name: &str, threshold: f64, duration: Duration, condition: F) -> Self
    where
        F: Fn(f64) -> bool + 'static,
    {
        AlertRule {
            name: name.to_string(),
            condition: Box::new(condition),
            threshold,
            duration,
            triggered_at: None,
            is_firing: false,
        }
    }

    fn check(&mut self, value: f64) -> Option<String> {
        let is_triggered = (self.condition)(value);

        if is_triggered {
            match self.triggered_at {
                None => {
                    self.triggered_at = Some(Instant::now());
                }
                Some(start) => {
                    if start.elapsed() >= self.duration && !self.is_firing {
                        self.is_firing = true;
                        return Some(format!(
                            "ALERT [{}]: значение {:.2} превысило порог {:.2}",
                            self.name, value, self.threshold
                        ));
                    }
                }
            }
        } else {
            if self.is_firing {
                self.is_firing = false;
                self.triggered_at = None;
                return Some(format!("RESOLVED [{}]: значение вернулось в норму", self.name));
            }
            self.triggered_at = None;
        }

        None
    }
}

/// Менеджер алертов для торговой системы
struct TradingAlertManager {
    rules: Vec<AlertRule>,
}

impl TradingAlertManager {
    fn new() -> Self {
        let mut rules = Vec::new();

        // Алерт на высокую задержку
        rules.push(AlertRule::new(
            "HighOrderLatency",
            100.0,
            Duration::from_secs(30),
            |v| v > 100.0,
        ));

        // Алерт на низкий win rate
        rules.push(AlertRule::new(
            "LowWinRate",
            0.4,
            Duration::from_secs(300),
            |v| v < 0.4,
        ));

        // Алерт на большую просадку
        rules.push(AlertRule::new(
            "HighDrawdown",
            10.0,
            Duration::from_secs(60),
            |v| v > 10.0,
        ));

        // Алерт на высокое использование rate limit
        rules.push(AlertRule::new(
            "RateLimitWarning",
            0.8,
            Duration::from_secs(10),
            |v| v > 0.8,
        ));

        // Алерт на отсутствие активности
        rules.push(AlertRule::new(
            "NoTradingActivity",
            0.0,
            Duration::from_secs(600),
            |v| v == 0.0,
        ));

        TradingAlertManager { rules }
    }

    fn check_latency(&mut self, value: f64) {
        if let Some(alert) = self.rules[0].check(value) {
            println!("{}", alert);
        }
    }

    fn check_win_rate(&mut self, value: f64) {
        if let Some(alert) = self.rules[1].check(value) {
            println!("{}", alert);
        }
    }

    fn check_drawdown(&mut self, value: f64) {
        if let Some(alert) = self.rules[2].check(value) {
            println!("{}", alert);
        }
    }

    fn check_rate_limit(&mut self, value: f64) {
        if let Some(alert) = self.rules[3].check(value) {
            println!("{}", alert);
        }
    }
}

fn main() {
    println!("=== Система алертинга ===\n");

    let mut alerts = TradingAlertManager::new();

    // Симуляция значений метрик
    println!("Проверка задержки...");
    for latency in [50.0, 80.0, 120.0, 150.0, 90.0, 60.0] {
        alerts.check_latency(latency);
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nПроверка win rate...");
    for win_rate in [0.6, 0.5, 0.35, 0.30, 0.45, 0.55] {
        alerts.check_win_rate(win_rate);
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nПроверка просадки...");
    for drawdown in [2.0, 5.0, 8.0, 12.0, 15.0, 7.0, 3.0] {
        alerts.check_drawdown(drawdown);
        std::thread::sleep(Duration::from_millis(100));
    }

    println!("\nПроверка rate limit...");
    for usage in [0.3, 0.5, 0.7, 0.85, 0.9, 0.6, 0.4] {
        alerts.check_rate_limit(usage);
        std::thread::sleep(Duration::from_millis(100));
    }
}
```

## Интеграция с Grafana

```yaml
# prometheus.yml - конфигурация Prometheus
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

rule_files:
  - "trading_rules.yml"

scrape_configs:
  - job_name: 'trading-system'
    static_configs:
      - targets: ['trading-bot:9090']
    metrics_path: '/metrics'

  - job_name: 'market-data-processor'
    static_configs:
      - targets: ['market-data:9091']

  - job_name: 'order-executor'
    static_configs:
      - targets: ['executor:9092']
```

```yaml
# trading_rules.yml - правила алертинга
groups:
  - name: trading_alerts
    rules:
      - alert: HighOrderLatency
        expr: histogram_quantile(0.99, rate(trading_order_latency_ms_bucket[5m])) > 100
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Высокая задержка исполнения ордеров"
          description: "99-й перцентиль задержки превышает 100ms"

      - alert: LowWinRate
        expr: strategy_win_rate < 0.4
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Низкий win rate стратегии"
          description: "Win rate упал ниже 40%"

      - alert: ExchangeDisconnected
        expr: exchange_connection_status == 0
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "Биржа отключена"
          description: "Потеряно соединение с {{ $labels.exchange }}"

      - alert: HighDrawdown
        expr: strategy_max_drawdown > 10
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Высокая просадка"
          description: "Просадка превышает 10%"

      - alert: RateLimitExceeded
        expr: rate_limit_usage > 0.9
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Близко к лимиту запросов"
          description: "Использование rate limit на {{ $labels.exchange }} превышает 90%"
```

```rust
// Пример дашборда для Grafana (в формате JSON)
// Это упрощённый пример конфигурации

/*
{
  "dashboard": {
    "title": "Trading System Dashboard",
    "panels": [
      {
        "title": "Orders per Second",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(orders_total[1m])",
            "legendFormat": "{{exchange}} - {{symbol}}"
          }
        ]
      },
      {
        "title": "Order Latency (p99)",
        "type": "gauge",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, rate(order_latency_ms_bucket[5m]))"
          }
        ]
      },
      {
        "title": "P&L by Strategy",
        "type": "graph",
        "targets": [
          {
            "expr": "strategy_realized_pnl_total",
            "legendFormat": "{{strategy}}"
          }
        ]
      },
      {
        "title": "Win Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "strategy_win_rate * 100",
            "legendFormat": "{{strategy}}"
          }
        ]
      },
      {
        "title": "Exchange Status",
        "type": "table",
        "targets": [
          {
            "expr": "exchange_connection_status",
            "legendFormat": "{{exchange}}"
          }
        ]
      }
    ]
  }
}
*/

fn main() {
    println!("=== Grafana Dashboard Configuration ===\n");

    println!("Полезные PromQL запросы для торговой системы:\n");

    println!("1. Количество ордеров в секунду:");
    println!("   rate(orders_total[1m])\n");

    println!("2. 99-й перцентиль задержки:");
    println!("   histogram_quantile(0.99, rate(order_latency_ms_bucket[5m]))\n");

    println!("3. Средний P&L за последний час:");
    println!("   increase(strategy_realized_pnl_total[1h])\n");

    println!("4. Win rate по стратегиям:");
    println!("   strategy_win_rate * 100\n");

    println!("5. Объём торгов в USD:");
    println!("   sum(rate(volume_usd_total[1h])) by (exchange)\n");

    println!("6. Ошибки по типам:");
    println!("   sum(rate(errors_total[5m])) by (error_type)\n");

    println!("7. Использование rate limit:");
    println!("   rate_limit_usage * 100\n");

    println!("8. Просадка по стратегиям:");
    println!("   strategy_max_drawdown\n");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Counter** | Метрика, которая только увеличивается (ордера, объём) |
| **Gauge** | Метрика с произвольным значением (цена, позиция) |
| **Histogram** | Распределение значений по бакетам (задержка) |
| **Labels** | Метки для группировки метрик (биржа, символ) |
| **Registry** | Хранилище для всех метрик приложения |
| **Scraping** | Процесс сбора метрик Prometheus |
| **PromQL** | Язык запросов для анализа метрик |
| **Alerting** | Автоматические уведомления по условиям |

## Практические задания

1. **Базовые метрики ордеров**: Создай систему метрик, которая:
   - Считает количество ордеров по биржам и символам
   - Отслеживает объём торгов
   - Измеряет задержку исполнения
   - Экспортирует метрики по HTTP

2. **Мониторинг стратегии**: Реализуй трекер стратегии:
   - Отслеживает win rate в реальном времени
   - Считает Sharpe ratio
   - Мониторит просадку
   - Генерирует алерты при критических значениях

3. **Дашборд производительности**: Создай набор метрик:
   - Время обработки сообщений
   - Размеры очередей
   - Использование памяти
   - Состояние подключений

4. **Система алертинга**: Реализуй менеджер алертов:
   - Настраиваемые правила
   - Различные уровни severity
   - История алертов
   - Интеграция с уведомлениями

## Домашнее задание

1. **Полноценный мониторинг торгового бота**: Создай систему:
   - Все типы метрик (counters, gauges, histograms)
   - Метки по биржам, символам, стратегиям
   - HTTP эндпоинт для Prometheus
   - Grafana дашборд с ключевыми показателями
   - Минимум 5 правил алертинга

2. **Метрики для HFT**: Реализуй мониторинг:
   - Наносекундная точность измерения задержки
   - Отслеживание jitter (вариации задержки)
   - Мониторинг tick-to-trade latency
   - Анализ очередей ордеров
   - Профилирование hot paths

3. **Распределённый мониторинг**: Создай архитектуру:
   - Несколько торговых ботов с метриками
   - Централизованный Prometheus
   - Агрегация метрик по сервисам
   - Корреляция метрик между компонентами
   - Tracing для отслеживания ордеров

4. **ML-enhanced мониторинг**: Добавь интеллектуальный анализ:
   - Автоматическое определение аномалий
   - Предсказание проблем на основе трендов
   - Адаптивные пороги алертов
   - Корреляция между метриками
   - Рекомендации по оптимизации

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../341-*/ru.md)
