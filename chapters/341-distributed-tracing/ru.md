# День 341: Трейсинг распределённых систем

## Аналогия из трейдинга

Представь, что ты управляешь крупной торговой платформой, где ордер проходит через множество сервисов: приём ордера → валидация → риск-менеджмент → маршрутизация → исполнение на бирже → подтверждение.

**Без трейсинга:**
Клиент жалуется: "Мой ордер исполнялся 5 секунд!" Ты смотришь в логи каждого сервиса по отдельности, пытаясь понять, где была задержка. Это как искать иголку в стоге сена.

**С трейсингом:**
Каждому ордеру присваивается уникальный `trace_id`. Когда ордер проходит через сервисы, каждый шаг записывается как `span` (отрезок). Ты видишь полный путь ордера: приём (2ms) → валидация (5ms) → риск-менеджмент (3ms) → маршрутизация (1ms) → исполнение (4890ms) → подтверждение (3ms). Сразу видно — проблема в исполнении на бирже!

| Компонент | Описание | Аналогия в трейдинге |
|-----------|----------|---------------------|
| **Trace** | Полный путь запроса | Жизненный цикл ордера |
| **Span** | Отдельная операция | Этап обработки ордера |
| **Context** | Передаваемые метаданные | ID ордера, ID клиента |
| **Propagation** | Передача контекста между сервисами | Передача ордера между отделами |

## Основы OpenTelemetry

OpenTelemetry — стандарт для сбора телеметрии (traces, metrics, logs). В Rust используется через крейт `tracing`:

```rust
use tracing::{info, instrument, span, Level};
use tracing_subscriber::fmt;
use std::collections::HashMap;

/// Структура ордера
#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: String,      // "BUY" или "SELL"
    price: f64,
    quantity: f64,
    client_id: String,
}

/// Результат исполнения
#[derive(Debug)]
struct ExecutionResult {
    order_id: String,
    status: String,
    executed_price: f64,
    executed_qty: f64,
    latency_ms: u64,
}

/// Приём ордера — первый span в trace
#[instrument(level = "info", fields(order_id = %order.id, symbol = %order.symbol))]
fn receive_order(order: &Order) -> Result<(), String> {
    info!("Получен ордер от клиента {}", order.client_id);

    // Имитация обработки
    std::thread::sleep(std::time::Duration::from_millis(2));

    info!("Ордер принят в обработку");
    Ok(())
}

/// Валидация ордера
#[instrument(level = "info", skip(order), fields(order_id = %order.id))]
fn validate_order(order: &Order) -> Result<(), String> {
    info!("Проверка параметров ордера");

    // Проверяем корректность
    if order.price <= 0.0 {
        return Err("Некорректная цена".to_string());
    }
    if order.quantity <= 0.0 {
        return Err("Некорректное количество".to_string());
    }
    if order.symbol.is_empty() {
        return Err("Символ не указан".to_string());
    }

    std::thread::sleep(std::time::Duration::from_millis(5));
    info!("Валидация пройдена");
    Ok(())
}

/// Проверка рисков
#[instrument(level = "info", fields(order_id = %order.id, exposure = %order.price * order.quantity))]
fn check_risk(order: &Order, max_position: f64) -> Result<(), String> {
    info!("Проверка лимитов риска");

    let order_value = order.price * order.quantity;

    if order_value > max_position {
        return Err(format!(
            "Превышен лимит позиции: {} > {}",
            order_value, max_position
        ));
    }

    std::thread::sleep(std::time::Duration::from_millis(3));
    info!(order_value = order_value, "Риск-проверка пройдена");
    Ok(())
}

/// Маршрутизация к бирже
#[instrument(level = "info", fields(order_id = %order.id))]
fn route_order(order: &Order) -> Result<String, String> {
    info!("Выбор оптимального маршрута");

    // Простая логика маршрутизации
    let exchange = if order.symbol.ends_with("USDT") {
        "binance"
    } else if order.symbol.ends_with("USD") {
        "kraken"
    } else {
        "default"
    };

    std::thread::sleep(std::time::Duration::from_millis(1));
    info!(exchange = exchange, "Маршрут выбран");
    Ok(exchange.to_string())
}

/// Исполнение на бирже
#[instrument(level = "info", fields(order_id = %order.id, exchange = %exchange))]
fn execute_on_exchange(order: &Order, exchange: &str) -> Result<ExecutionResult, String> {
    info!("Отправка ордера на биржу");

    // Имитация задержки биржи
    let latency = match exchange {
        "binance" => 15,
        "kraken" => 25,
        _ => 50,
    };

    std::thread::sleep(std::time::Duration::from_millis(latency));

    // Имитация проскальзывания
    let slippage = if order.side == "BUY" { 1.001 } else { 0.999 };
    let executed_price = order.price * slippage;

    info!(
        executed_price = executed_price,
        latency_ms = latency,
        "Ордер исполнен"
    );

    Ok(ExecutionResult {
        order_id: order.id.clone(),
        status: "FILLED".to_string(),
        executed_price,
        executed_qty: order.quantity,
        latency_ms: latency,
    })
}

/// Полный цикл обработки ордера
#[instrument(level = "info", name = "order_lifecycle", fields(trace_id = %trace_id))]
fn process_order(order: Order, trace_id: &str, max_position: f64) -> Result<ExecutionResult, String> {
    info!("=== Начало обработки ордера ===");

    // Каждый шаг создаёт свой span
    receive_order(&order)?;
    validate_order(&order)?;
    check_risk(&order, max_position)?;

    let exchange = route_order(&order)?;
    let result = execute_on_exchange(&order, &exchange)?;

    info!(
        status = %result.status,
        total_latency_ms = result.latency_ms,
        "=== Ордер обработан ==="
    );

    Ok(result)
}

fn main() {
    // Инициализация трейсинга
    fmt::init();

    println!("=== Трейсинг распределённых систем ===\n");

    let order = Order {
        id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 50000.0,
        quantity: 0.1,
        client_id: "CLIENT-123".to_string(),
    };

    let trace_id = "trace-abc123";
    let max_position = 100000.0;

    match process_order(order, trace_id, max_position) {
        Ok(result) => {
            println!("\nРезультат исполнения:");
            println!("  Статус: {}", result.status);
            println!("  Цена исполнения: ${:.2}", result.executed_price);
            println!("  Количество: {}", result.executed_qty);
            println!("  Латентность: {}ms", result.latency_ms);
        }
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Распределённый трейсинг между сервисами

В реальных системах трейсинг должен пересекать границы сервисов:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Контекст трейсинга, передаваемый между сервисами
#[derive(Debug, Clone)]
struct TraceContext {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    baggage: HashMap<String, String>,
}

impl TraceContext {
    fn new(trace_id: &str) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            span_id: generate_span_id(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }

    fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }

    fn with_baggage(mut self, key: &str, value: &str) -> Self {
        self.baggage.insert(key.to_string(), value.to_string());
        self
    }
}

fn generate_span_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", nanos & 0xFFFFFFFFFFFFFFFF)
}

/// Span — единица работы
#[derive(Debug)]
struct Span {
    name: String,
    context: TraceContext,
    start_time: Instant,
    end_time: Option<Instant>,
    attributes: HashMap<String, String>,
    events: Vec<SpanEvent>,
    status: SpanStatus,
}

#[derive(Debug)]
struct SpanEvent {
    name: String,
    timestamp: Instant,
    attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum SpanStatus {
    Ok,
    Error(String),
}

impl Span {
    fn new(name: &str, context: TraceContext) -> Self {
        Self {
            name: name.to_string(),
            context,
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::Ok,
        }
    }

    fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    fn add_event(&mut self, name: &str) {
        self.events.push(SpanEvent {
            name: name.to_string(),
            timestamp: Instant::now(),
            attributes: HashMap::new(),
        });
    }

    fn set_error(&mut self, error: &str) {
        self.status = SpanStatus::Error(error.to_string());
    }

    fn end(&mut self) {
        self.end_time = Some(Instant::now());
    }

    fn duration(&self) -> Duration {
        self.end_time
            .unwrap_or_else(Instant::now)
            .duration_since(self.start_time)
    }
}

/// Коллектор spans для анализа
struct SpanCollector {
    spans: Vec<Span>,
}

impl SpanCollector {
    fn new() -> Self {
        Self { spans: Vec::new() }
    }

    fn collect(&mut self, span: Span) {
        self.spans.push(span);
    }

    fn print_trace(&self, trace_id: &str) {
        println!("\n=== Trace: {} ===", trace_id);

        let trace_spans: Vec<_> = self.spans
            .iter()
            .filter(|s| s.context.trace_id == trace_id)
            .collect();

        // Строим дерево spans
        for span in &trace_spans {
            let indent = if span.context.parent_span_id.is_some() { "  " } else { "" };
            let status = match &span.status {
                SpanStatus::Ok => "OK",
                SpanStatus::Error(e) => e.as_str(),
            };

            println!(
                "{}{} [{:?}] - {}",
                indent,
                span.name,
                span.duration(),
                status
            );

            for (key, value) in &span.attributes {
                println!("{}  {} = {}", indent, key, value);
            }
        }
    }

    fn get_critical_path(&self, trace_id: &str) -> Duration {
        self.spans
            .iter()
            .filter(|s| s.context.trace_id == trace_id && s.context.parent_span_id.is_none())
            .map(|s| s.duration())
            .sum()
    }
}

/// Симуляция Order Gateway Service
fn order_gateway(ctx: TraceContext, order_data: &str, collector: &mut SpanCollector) -> Result<TraceContext, String> {
    let mut span = Span::new("order_gateway.receive", ctx.clone());
    span.set_attribute("order.data", order_data);
    span.add_event("order_received");

    // Парсинг ордера
    std::thread::sleep(Duration::from_millis(2));
    span.add_event("order_parsed");

    span.end();
    collector.collect(span);

    Ok(ctx.child())
}

/// Симуляция Risk Service
fn risk_service(ctx: TraceContext, order_value: f64, collector: &mut SpanCollector) -> Result<TraceContext, String> {
    let mut span = Span::new("risk_service.check", ctx.clone());
    span.set_attribute("order.value", &order_value.to_string());

    // Проверка лимитов
    std::thread::sleep(Duration::from_millis(5));

    if order_value > 1_000_000.0 {
        span.set_error("position_limit_exceeded");
        span.end();
        collector.collect(span);
        return Err("Position limit exceeded".to_string());
    }

    span.add_event("risk_check_passed");
    span.end();
    collector.collect(span);

    Ok(ctx.child())
}

/// Симуляция Execution Service
fn execution_service(ctx: TraceContext, symbol: &str, collector: &mut SpanCollector) -> Result<(TraceContext, f64), String> {
    let mut span = Span::new("execution_service.execute", ctx.clone());
    span.set_attribute("symbol", symbol);

    // Подключение к бирже
    let child_ctx = ctx.child();
    let mut connect_span = Span::new("exchange.connect", child_ctx.clone());
    std::thread::sleep(Duration::from_millis(3));
    connect_span.end();
    collector.collect(connect_span);

    // Отправка ордера
    let child_ctx2 = child_ctx.child();
    let mut send_span = Span::new("exchange.send_order", child_ctx2.clone());
    std::thread::sleep(Duration::from_millis(10));
    send_span.set_attribute("exchange", "binance");
    send_span.add_event("order_sent");

    // Получение подтверждения
    std::thread::sleep(Duration::from_millis(5));
    send_span.add_event("confirmation_received");
    send_span.end();
    collector.collect(send_span);

    let executed_price = 50001.5;
    span.set_attribute("executed_price", &executed_price.to_string());
    span.end();
    collector.collect(span);

    Ok((child_ctx2.child(), executed_price))
}

/// Полный путь ордера через распределённую систему
fn distributed_order_flow(order_id: &str, symbol: &str, value: f64) {
    let mut collector = SpanCollector::new();
    let trace_id = format!("trace-{}", order_id);

    println!("Запуск распределённого трейсинга для ордера {}", order_id);

    // Создаём корневой контекст
    let ctx = TraceContext::new(&trace_id)
        .with_baggage("order_id", order_id)
        .with_baggage("client_id", "CLIENT-456");

    // Корневой span
    let mut root_span = Span::new("order.process", ctx.clone());
    root_span.set_attribute("order_id", order_id);
    root_span.set_attribute("symbol", symbol);

    // Проходим через все сервисы
    let result = order_gateway(ctx.child(), &format!("{}|{}|{}", order_id, symbol, value), &mut collector)
        .and_then(|ctx| risk_service(ctx, value, &mut collector))
        .and_then(|ctx| execution_service(ctx, symbol, &mut collector));

    match result {
        Ok((_, price)) => {
            root_span.set_attribute("status", "success");
            root_span.set_attribute("executed_price", &price.to_string());
            println!("Ордер исполнен по цене: ${:.2}", price);
        }
        Err(e) => {
            root_span.set_error(&e);
            println!("Ошибка: {}", e);
        }
    }

    root_span.end();
    collector.collect(root_span);

    // Выводим trace
    collector.print_trace(&trace_id);

    println!(
        "\nОбщее время обработки: {:?}",
        collector.get_critical_path(&trace_id)
    );
}

fn main() {
    println!("=== Распределённый трейсинг торговой системы ===\n");

    distributed_order_flow("ORD-001", "BTCUSDT", 50000.0);

    println!("\n--- Второй ордер (с ошибкой риска) ---\n");

    distributed_order_flow("ORD-002", "ETHUSDT", 2_000_000.0);
}
```

## Интеграция с Jaeger

Jaeger — популярная система для хранения и визуализации traces:

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Формат Jaeger Thrift для отправки spans
#[derive(Debug)]
struct JaegerSpan {
    trace_id_low: i64,
    trace_id_high: i64,
    span_id: i64,
    parent_span_id: i64,
    operation_name: String,
    flags: i32,
    start_time: i64,  // microseconds
    duration: i64,    // microseconds
    tags: Vec<JaegerTag>,
    logs: Vec<JaegerLog>,
}

#[derive(Debug, Clone)]
struct JaegerTag {
    key: String,
    value: JaegerTagValue,
}

#[derive(Debug, Clone)]
enum JaegerTagValue {
    String(String),
    Double(f64),
    Bool(bool),
    Long(i64),
}

#[derive(Debug)]
struct JaegerLog {
    timestamp: i64,
    fields: Vec<JaegerTag>,
}

/// Экспортёр spans в формате Jaeger
struct JaegerExporter {
    service_name: String,
    spans: Vec<JaegerSpan>,
}

impl JaegerExporter {
    fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            spans: Vec::new(),
        }
    }

    fn add_span(&mut self, span: JaegerSpan) {
        self.spans.push(span);
    }

    fn export(&self) -> String {
        // В реальности отправляем через HTTP/UDP
        // Здесь формируем JSON для демонстрации
        let mut output = format!("Service: {}\n", self.service_name);
        output.push_str("Spans:\n");

        for span in &self.spans {
            output.push_str(&format!(
                "  - {} ({}μs)\n",
                span.operation_name, span.duration
            ));

            for tag in &span.tags {
                let value = match &tag.value {
                    JaegerTagValue::String(s) => s.clone(),
                    JaegerTagValue::Double(d) => d.to_string(),
                    JaegerTagValue::Bool(b) => b.to_string(),
                    JaegerTagValue::Long(l) => l.to_string(),
                };
                output.push_str(&format!("    {}: {}\n", tag.key, value));
            }
        }

        output
    }
}

/// Построитель spans для торговых операций
struct TradingSpanBuilder {
    trace_id: i64,
    service_name: String,
}

impl TradingSpanBuilder {
    fn new(service_name: &str) -> Self {
        let trace_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        Self {
            trace_id,
            service_name: service_name.to_string(),
        }
    }

    fn create_span(
        &self,
        operation: &str,
        parent_id: i64,
        duration_us: i64,
        tags: Vec<(&str, &str)>,
    ) -> JaegerSpan {
        let span_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        JaegerSpan {
            trace_id_low: self.trace_id,
            trace_id_high: 0,
            span_id,
            parent_span_id: parent_id,
            operation_name: operation.to_string(),
            flags: 1,
            start_time,
            duration: duration_us,
            tags: tags
                .into_iter()
                .map(|(k, v)| JaegerTag {
                    key: k.to_string(),
                    value: JaegerTagValue::String(v.to_string()),
                })
                .collect(),
            logs: Vec::new(),
        }
    }
}

/// Демонстрация интеграции с Jaeger
fn jaeger_integration_demo() {
    println!("=== Интеграция с Jaeger ===\n");

    let mut exporter = JaegerExporter::new("trading-gateway");
    let builder = TradingSpanBuilder::new("trading-gateway");

    // Создаём spans для торговой операции
    let root_span = builder.create_span(
        "process_order",
        0,
        50000,  // 50ms
        vec![
            ("order.id", "ORD-12345"),
            ("order.symbol", "BTCUSDT"),
            ("order.side", "BUY"),
        ],
    );

    let validate_span = builder.create_span(
        "validate_order",
        root_span.span_id,
        5000,  // 5ms
        vec![
            ("validation.type", "full"),
        ],
    );

    let risk_span = builder.create_span(
        "check_risk",
        root_span.span_id,
        3000,  // 3ms
        vec![
            ("risk.limit", "100000"),
            ("risk.status", "passed"),
        ],
    );

    let execute_span = builder.create_span(
        "execute_order",
        root_span.span_id,
        40000,  // 40ms
        vec![
            ("exchange", "binance"),
            ("execution.price", "50001.50"),
            ("execution.status", "filled"),
        ],
    );

    exporter.add_span(root_span);
    exporter.add_span(validate_span);
    exporter.add_span(risk_span);
    exporter.add_span(execute_span);

    println!("{}", exporter.export());
}

fn main() {
    jaeger_integration_demo();
}
```

## Метрики из traces

Трейсы можно использовать для генерации метрик:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Метрики, извлекаемые из traces
#[derive(Debug, Default)]
struct TraceMetrics {
    // Латентность
    latency_sum: Duration,
    latency_count: u64,
    latency_min: Option<Duration>,
    latency_max: Option<Duration>,

    // Ошибки
    error_count: u64,

    // Распределение по операциям
    operation_counts: HashMap<String, u64>,
    operation_latencies: HashMap<String, Vec<Duration>>,
}

impl TraceMetrics {
    fn new() -> Self {
        Self::default()
    }

    fn record_span(&mut self, operation: &str, duration: Duration, is_error: bool) {
        // Общая латентность
        self.latency_sum += duration;
        self.latency_count += 1;

        self.latency_min = Some(
            self.latency_min.map_or(duration, |min| min.min(duration))
        );
        self.latency_max = Some(
            self.latency_max.map_or(duration, |max| max.max(duration))
        );

        // Ошибки
        if is_error {
            self.error_count += 1;
        }

        // По операциям
        *self.operation_counts.entry(operation.to_string()).or_insert(0) += 1;
        self.operation_latencies
            .entry(operation.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    fn avg_latency(&self) -> Duration {
        if self.latency_count == 0 {
            Duration::ZERO
        } else {
            self.latency_sum / self.latency_count as u32
        }
    }

    fn error_rate(&self) -> f64 {
        if self.latency_count == 0 {
            0.0
        } else {
            self.error_count as f64 / self.latency_count as f64 * 100.0
        }
    }

    fn percentile(&self, operation: &str, p: f64) -> Option<Duration> {
        self.operation_latencies.get(operation).and_then(|latencies| {
            if latencies.is_empty() {
                return None;
            }

            let mut sorted = latencies.clone();
            sorted.sort();

            let index = ((p / 100.0) * sorted.len() as f64) as usize;
            let index = index.min(sorted.len() - 1);
            Some(sorted[index])
        })
    }

    fn print_summary(&self) {
        println!("=== Метрики из traces ===\n");

        println!("Общая статистика:");
        println!("  Всего spans: {}", self.latency_count);
        println!("  Средняя латентность: {:?}", self.avg_latency());
        println!("  Мин. латентность: {:?}", self.latency_min.unwrap_or_default());
        println!("  Макс. латентность: {:?}", self.latency_max.unwrap_or_default());
        println!("  Ошибок: {} ({:.2}%)", self.error_count, self.error_rate());

        println!("\nПо операциям:");
        for (op, count) in &self.operation_counts {
            let p50 = self.percentile(op, 50.0);
            let p99 = self.percentile(op, 99.0);

            println!(
                "  {}: {} вызовов, p50={:?}, p99={:?}",
                op,
                count,
                p50.unwrap_or_default(),
                p99.unwrap_or_default()
            );
        }
    }
}

/// Трейсинг торговых операций с метриками
struct InstrumentedTrader {
    metrics: TraceMetrics,
}

impl InstrumentedTrader {
    fn new() -> Self {
        Self {
            metrics: TraceMetrics::new(),
        }
    }

    fn execute_order(&mut self, symbol: &str, side: &str, price: f64, qty: f64) -> Result<f64, String> {
        // Валидация
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(2));
        self.metrics.record_span("validate", start.elapsed(), false);

        // Риск-проверка
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(3));
        let order_value = price * qty;
        let is_risk_error = order_value > 100_000.0;
        self.metrics.record_span("risk_check", start.elapsed(), is_risk_error);

        if is_risk_error {
            return Err("Risk limit exceeded".to_string());
        }

        // Исполнение
        let start = Instant::now();
        // Имитация различной латентности
        let execution_time = if symbol.contains("BTC") { 10 } else { 15 };
        std::thread::sleep(Duration::from_millis(execution_time));
        self.metrics.record_span("execute", start.elapsed(), false);

        // Слегка изменяем цену (проскальзывание)
        let executed_price = if side == "BUY" {
            price * 1.001
        } else {
            price * 0.999
        };

        Ok(executed_price)
    }

    fn get_metrics(&self) -> &TraceMetrics {
        &self.metrics
    }
}

fn main() {
    println!("=== Метрики из трейсов торговой системы ===\n");

    let mut trader = InstrumentedTrader::new();

    // Симулируем серию ордеров
    let orders = vec![
        ("BTCUSDT", "BUY", 50000.0, 0.1),
        ("ETHUSDT", "SELL", 3000.0, 1.0),
        ("BTCUSDT", "BUY", 50100.0, 0.5),
        ("SOLUSDT", "BUY", 100.0, 10.0),
        ("BTCUSDT", "SELL", 50050.0, 0.2),
        ("ETHUSDT", "BUY", 2990.0, 2.0),
        // Ордер с превышением лимита
        ("BTCUSDT", "BUY", 50000.0, 10.0),
    ];

    for (symbol, side, price, qty) in orders {
        match trader.execute_order(symbol, side, price, qty) {
            Ok(executed_price) => {
                println!("{} {} {} @ ${:.2} -> ${:.2}",
                    side, qty, symbol, price, executed_price);
            }
            Err(e) => {
                println!("{} {} {} @ ${:.2} -> ОШИБКА: {}",
                    side, qty, symbol, price, e);
            }
        }
    }

    println!();
    trader.get_metrics().print_summary();
}
```

## Трейсинг WebSocket соединений

Для торговых систем важен трейсинг WebSocket соединений:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Состояние WebSocket соединения
#[derive(Debug, Clone)]
enum WsConnectionState {
    Connecting,
    Connected,
    Authenticated,
    Subscribed,
    Disconnected,
}

/// Span для WebSocket событий
#[derive(Debug)]
struct WsSpan {
    id: String,
    operation: String,
    state: WsConnectionState,
    start_time: Instant,
    end_time: Option<Instant>,
    attributes: HashMap<String, String>,
    events: Vec<(String, Instant)>,
}

impl WsSpan {
    fn new(operation: &str) -> Self {
        Self {
            id: format!("{:016x}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64),
            operation: operation.to_string(),
            state: WsConnectionState::Connecting,
            start_time: Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    fn set_state(&mut self, state: WsConnectionState) {
        self.events.push((format!("state:{:?}", state), Instant::now()));
        self.state = state;
    }

    fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    fn add_event(&mut self, event: &str) {
        self.events.push((event.to_string(), Instant::now()));
    }

    fn end(&mut self) {
        self.end_time = Some(Instant::now());
    }

    fn duration(&self) -> Duration {
        self.end_time.unwrap_or_else(Instant::now).duration_since(self.start_time)
    }
}

/// Трейсер для WebSocket market data
struct WsMarketDataTracer {
    connection_spans: Vec<WsSpan>,
    message_count: u64,
    error_count: u64,
}

impl WsMarketDataTracer {
    fn new() -> Self {
        Self {
            connection_spans: Vec::new(),
            message_count: 0,
            error_count: 0,
        }
    }

    fn trace_connection(&mut self, exchange: &str, endpoint: &str) -> &mut WsSpan {
        let mut span = WsSpan::new("ws.connect");
        span.add_attribute("exchange", exchange);
        span.add_attribute("endpoint", endpoint);

        self.connection_spans.push(span);
        self.connection_spans.last_mut().unwrap()
    }

    fn trace_subscription(&mut self, symbols: &[&str]) {
        if let Some(span) = self.connection_spans.last_mut() {
            span.add_event("subscribe_started");
            span.add_attribute("symbols", &symbols.join(","));

            // Имитация подписки
            std::thread::sleep(Duration::from_millis(5));

            span.set_state(WsConnectionState::Subscribed);
            span.add_event("subscribe_completed");
        }
    }

    fn trace_message(&mut self, msg_type: &str, symbol: &str, latency_us: u64) {
        self.message_count += 1;

        // Логируем каждое 100-е сообщение
        if self.message_count % 100 == 0 {
            println!(
                "  [MSG #{}] {} {} latency={}μs",
                self.message_count, msg_type, symbol, latency_us
            );
        }
    }

    fn trace_error(&mut self, error: &str) {
        self.error_count += 1;
        if let Some(span) = self.connection_spans.last_mut() {
            span.add_event(&format!("error:{}", error));
        }
    }

    fn trace_disconnect(&mut self, reason: &str) {
        if let Some(span) = self.connection_spans.last_mut() {
            span.add_event(&format!("disconnect:{}", reason));
            span.set_state(WsConnectionState::Disconnected);
            span.end();
        }
    }

    fn print_summary(&self) {
        println!("\n=== WebSocket Трейсинг ===\n");

        for span in &self.connection_spans {
            println!("Соединение: {}", span.operation);
            println!("  ID: {}", span.id);
            println!("  Состояние: {:?}", span.state);
            println!("  Длительность: {:?}", span.duration());

            println!("  Атрибуты:");
            for (k, v) in &span.attributes {
                println!("    {}: {}", k, v);
            }

            println!("  События:");
            for (event, time) in &span.events {
                let offset = time.duration_since(span.start_time);
                println!("    +{:?}: {}", offset, event);
            }
        }

        println!("\nСтатистика:");
        println!("  Сообщений обработано: {}", self.message_count);
        println!("  Ошибок: {}", self.error_count);
    }
}

/// Демонстрация трейсинга WebSocket
fn ws_tracing_demo() {
    println!("=== Трейсинг WebSocket Market Data ===\n");

    let mut tracer = WsMarketDataTracer::new();

    // Подключение
    println!("Подключение к Binance...");
    let span = tracer.trace_connection("binance", "wss://stream.binance.com:9443");
    std::thread::sleep(Duration::from_millis(50));
    span.set_state(WsConnectionState::Connected);

    // Аутентификация
    std::thread::sleep(Duration::from_millis(10));
    span.set_state(WsConnectionState::Authenticated);
    span.add_event("auth_success");

    // Подписка
    tracer.trace_subscription(&["BTCUSDT", "ETHUSDT", "SOLUSDT"]);

    // Симуляция получения сообщений
    println!("\nПолучение market data...");
    for i in 0..350 {
        let symbol = match i % 3 {
            0 => "BTCUSDT",
            1 => "ETHUSDT",
            _ => "SOLUSDT",
        };

        // Случайная латентность
        let latency = 50 + (i % 200) as u64;
        tracer.trace_message("trade", symbol, latency);

        // Иногда ошибки
        if i == 150 {
            tracer.trace_error("message_parse_error");
        }
    }

    // Отключение
    tracer.trace_disconnect("normal_closure");

    tracer.print_summary();
}

fn main() {
    ws_tracing_demo();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Trace** | Полный путь запроса через систему |
| **Span** | Отдельная операция в рамках trace |
| **Context Propagation** | Передача контекста между сервисами |
| **Baggage** | Пользовательские данные, передаваемые с контекстом |
| **Jaeger** | Система хранения и визуализации traces |
| **OpenTelemetry** | Стандарт сбора телеметрии |
| **Span Attributes** | Метаданные операции |
| **Span Events** | События внутри операции |

## Практические задания

1. **Трейсинг ордера через систему**: Создай систему, которая:
   - Отслеживает ордер от приёма до исполнения
   - Записывает время каждого этапа
   - Показывает bottleneck'и в обработке
   - Генерирует alert при превышении латентности

2. **Correlation ID для арбитража**: Реализуй:
   - Связывание ордеров на разных биржах
   - Общий trace для арбитражной сделки
   - Расчёт общей латентности арбитража
   - Анализ успешности по trace данным

3. **Мониторинг WebSocket потоков**: Создай:
   - Трейсинг подключений к биржам
   - Отслеживание задержек в данных
   - Детекцию разрывов соединений
   - Метрики качества данных

4. **Визуализация traces**: Реализуй:
   - Экспорт в формат Jaeger
   - Построение timeline операций
   - Группировку spans по сервисам
   - Поиск аномальных traces

## Домашнее задание

1. **Полноценный трейсинг торговой системы**: Создай систему:
   - Трейсинг всех этапов обработки ордера
   - Интеграция с Jaeger (или mock)
   - Алерты на аномальную латентность
   - Dashboard с метриками из traces
   - Поиск по trace_id и order_id

2. **Распределённый трейсинг микросервисов**: Реализуй:
   - 3-4 сервиса (gateway, risk, execution, notification)
   - Передачу контекста между сервисами
   - Единый trace для запроса
   - Sampling для высоконагруженных сценариев
   - Обработку ошибок с сохранением trace

3. **Анализ производительности из traces**: Создай инструмент:
   - Сбор traces за период
   - Расчёт percentiles (p50, p95, p99)
   - Выявление медленных операций
   - Сравнение производительности по времени
   - Рекомендации по оптимизации

4. **Трейсинг стратегии**: Реализуй:
   - Trace для каждого сигнала стратегии
   - Связывание сигнала с исполнением
   - Анализ времени от сигнала до исполнения
   - Корреляция задержек с проскальзыванием
   - Оптимизация на основе trace данных

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../342-*/ru.md)
