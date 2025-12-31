# Day 341: Distributed Tracing

## Trading Analogy

Imagine you're managing a large trading platform where an order passes through multiple services: order reception → validation → risk management → routing → exchange execution → confirmation.

**Without tracing:**
A client complains: "My order took 5 seconds to execute!" You look through logs of each service separately, trying to understand where the delay occurred. It's like finding a needle in a haystack.

**With tracing:**
Each order gets a unique `trace_id`. As the order passes through services, each step is recorded as a `span` (segment). You see the complete order path: reception (2ms) → validation (5ms) → risk management (3ms) → routing (1ms) → execution (4890ms) → confirmation (3ms). The problem is immediately clear — it's the exchange execution!

| Component | Description | Trading Analogy |
|-----------|-------------|-----------------|
| **Trace** | Complete request path | Order lifecycle |
| **Span** | Individual operation | Order processing stage |
| **Context** | Passed metadata | Order ID, Client ID |
| **Propagation** | Context passing between services | Order handoff between departments |

## OpenTelemetry Basics

OpenTelemetry is a standard for collecting telemetry (traces, metrics, logs). In Rust, it's used through the `tracing` crate:

```rust
use tracing::{info, instrument, span, Level};
use tracing_subscriber::fmt;
use std::collections::HashMap;

/// Order structure
#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: String,      // "BUY" or "SELL"
    price: f64,
    quantity: f64,
    client_id: String,
}

/// Execution result
#[derive(Debug)]
struct ExecutionResult {
    order_id: String,
    status: String,
    executed_price: f64,
    executed_qty: f64,
    latency_ms: u64,
}

/// Order reception — first span in trace
#[instrument(level = "info", fields(order_id = %order.id, symbol = %order.symbol))]
fn receive_order(order: &Order) -> Result<(), String> {
    info!("Received order from client {}", order.client_id);

    // Simulating processing
    std::thread::sleep(std::time::Duration::from_millis(2));

    info!("Order accepted for processing");
    Ok(())
}

/// Order validation
#[instrument(level = "info", skip(order), fields(order_id = %order.id))]
fn validate_order(order: &Order) -> Result<(), String> {
    info!("Checking order parameters");

    // Validate correctness
    if order.price <= 0.0 {
        return Err("Invalid price".to_string());
    }
    if order.quantity <= 0.0 {
        return Err("Invalid quantity".to_string());
    }
    if order.symbol.is_empty() {
        return Err("Symbol not specified".to_string());
    }

    std::thread::sleep(std::time::Duration::from_millis(5));
    info!("Validation passed");
    Ok(())
}

/// Risk check
#[instrument(level = "info", fields(order_id = %order.id, exposure = %order.price * order.quantity))]
fn check_risk(order: &Order, max_position: f64) -> Result<(), String> {
    info!("Checking risk limits");

    let order_value = order.price * order.quantity;

    if order_value > max_position {
        return Err(format!(
            "Position limit exceeded: {} > {}",
            order_value, max_position
        ));
    }

    std::thread::sleep(std::time::Duration::from_millis(3));
    info!(order_value = order_value, "Risk check passed");
    Ok(())
}

/// Routing to exchange
#[instrument(level = "info", fields(order_id = %order.id))]
fn route_order(order: &Order) -> Result<String, String> {
    info!("Selecting optimal route");

    // Simple routing logic
    let exchange = if order.symbol.ends_with("USDT") {
        "binance"
    } else if order.symbol.ends_with("USD") {
        "kraken"
    } else {
        "default"
    };

    std::thread::sleep(std::time::Duration::from_millis(1));
    info!(exchange = exchange, "Route selected");
    Ok(exchange.to_string())
}

/// Execution on exchange
#[instrument(level = "info", fields(order_id = %order.id, exchange = %exchange))]
fn execute_on_exchange(order: &Order, exchange: &str) -> Result<ExecutionResult, String> {
    info!("Sending order to exchange");

    // Simulating exchange latency
    let latency = match exchange {
        "binance" => 15,
        "kraken" => 25,
        _ => 50,
    };

    std::thread::sleep(std::time::Duration::from_millis(latency));

    // Simulating slippage
    let slippage = if order.side == "BUY" { 1.001 } else { 0.999 };
    let executed_price = order.price * slippage;

    info!(
        executed_price = executed_price,
        latency_ms = latency,
        "Order executed"
    );

    Ok(ExecutionResult {
        order_id: order.id.clone(),
        status: "FILLED".to_string(),
        executed_price,
        executed_qty: order.quantity,
        latency_ms: latency,
    })
}

/// Complete order processing cycle
#[instrument(level = "info", name = "order_lifecycle", fields(trace_id = %trace_id))]
fn process_order(order: Order, trace_id: &str, max_position: f64) -> Result<ExecutionResult, String> {
    info!("=== Starting order processing ===");

    // Each step creates its own span
    receive_order(&order)?;
    validate_order(&order)?;
    check_risk(&order, max_position)?;

    let exchange = route_order(&order)?;
    let result = execute_on_exchange(&order, &exchange)?;

    info!(
        status = %result.status,
        total_latency_ms = result.latency_ms,
        "=== Order processed ==="
    );

    Ok(result)
}

fn main() {
    // Initialize tracing
    fmt::init();

    println!("=== Distributed Tracing Systems ===\n");

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
            println!("\nExecution result:");
            println!("  Status: {}", result.status);
            println!("  Executed price: ${:.2}", result.executed_price);
            println!("  Quantity: {}", result.executed_qty);
            println!("  Latency: {}ms", result.latency_ms);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

## Distributed Tracing Between Services

In real systems, tracing must cross service boundaries:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Tracing context passed between services
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

/// Span — unit of work
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

/// Span collector for analysis
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

        // Build span tree
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

/// Simulating Order Gateway Service
fn order_gateway(ctx: TraceContext, order_data: &str, collector: &mut SpanCollector) -> Result<TraceContext, String> {
    let mut span = Span::new("order_gateway.receive", ctx.clone());
    span.set_attribute("order.data", order_data);
    span.add_event("order_received");

    // Order parsing
    std::thread::sleep(Duration::from_millis(2));
    span.add_event("order_parsed");

    span.end();
    collector.collect(span);

    Ok(ctx.child())
}

/// Simulating Risk Service
fn risk_service(ctx: TraceContext, order_value: f64, collector: &mut SpanCollector) -> Result<TraceContext, String> {
    let mut span = Span::new("risk_service.check", ctx.clone());
    span.set_attribute("order.value", &order_value.to_string());

    // Limit checking
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

/// Simulating Execution Service
fn execution_service(ctx: TraceContext, symbol: &str, collector: &mut SpanCollector) -> Result<(TraceContext, f64), String> {
    let mut span = Span::new("execution_service.execute", ctx.clone());
    span.set_attribute("symbol", symbol);

    // Exchange connection
    let child_ctx = ctx.child();
    let mut connect_span = Span::new("exchange.connect", child_ctx.clone());
    std::thread::sleep(Duration::from_millis(3));
    connect_span.end();
    collector.collect(connect_span);

    // Order submission
    let child_ctx2 = child_ctx.child();
    let mut send_span = Span::new("exchange.send_order", child_ctx2.clone());
    std::thread::sleep(Duration::from_millis(10));
    send_span.set_attribute("exchange", "binance");
    send_span.add_event("order_sent");

    // Confirmation receipt
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

/// Complete order path through distributed system
fn distributed_order_flow(order_id: &str, symbol: &str, value: f64) {
    let mut collector = SpanCollector::new();
    let trace_id = format!("trace-{}", order_id);

    println!("Starting distributed tracing for order {}", order_id);

    // Create root context
    let ctx = TraceContext::new(&trace_id)
        .with_baggage("order_id", order_id)
        .with_baggage("client_id", "CLIENT-456");

    // Root span
    let mut root_span = Span::new("order.process", ctx.clone());
    root_span.set_attribute("order_id", order_id);
    root_span.set_attribute("symbol", symbol);

    // Pass through all services
    let result = order_gateway(ctx.child(), &format!("{}|{}|{}", order_id, symbol, value), &mut collector)
        .and_then(|ctx| risk_service(ctx, value, &mut collector))
        .and_then(|ctx| execution_service(ctx, symbol, &mut collector));

    match result {
        Ok((_, price)) => {
            root_span.set_attribute("status", "success");
            root_span.set_attribute("executed_price", &price.to_string());
            println!("Order executed at price: ${:.2}", price);
        }
        Err(e) => {
            root_span.set_error(&e);
            println!("Error: {}", e);
        }
    }

    root_span.end();
    collector.collect(root_span);

    // Print trace
    collector.print_trace(&trace_id);

    println!(
        "\nTotal processing time: {:?}",
        collector.get_critical_path(&trace_id)
    );
}

fn main() {
    println!("=== Distributed Tracing for Trading System ===\n");

    distributed_order_flow("ORD-001", "BTCUSDT", 50000.0);

    println!("\n--- Second order (with risk error) ---\n");

    distributed_order_flow("ORD-002", "ETHUSDT", 2_000_000.0);
}
```

## Jaeger Integration

Jaeger is a popular system for storing and visualizing traces:

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Jaeger Thrift format for sending spans
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

/// Span exporter in Jaeger format
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
        // In reality, we'd send via HTTP/UDP
        // Here we format as JSON for demonstration
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

/// Span builder for trading operations
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

/// Demonstrating Jaeger integration
fn jaeger_integration_demo() {
    println!("=== Jaeger Integration ===\n");

    let mut exporter = JaegerExporter::new("trading-gateway");
    let builder = TradingSpanBuilder::new("trading-gateway");

    // Create spans for trading operation
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

## Metrics from Traces

Traces can be used to generate metrics:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metrics extracted from traces
#[derive(Debug, Default)]
struct TraceMetrics {
    // Latency
    latency_sum: Duration,
    latency_count: u64,
    latency_min: Option<Duration>,
    latency_max: Option<Duration>,

    // Errors
    error_count: u64,

    // Distribution by operations
    operation_counts: HashMap<String, u64>,
    operation_latencies: HashMap<String, Vec<Duration>>,
}

impl TraceMetrics {
    fn new() -> Self {
        Self::default()
    }

    fn record_span(&mut self, operation: &str, duration: Duration, is_error: bool) {
        // Total latency
        self.latency_sum += duration;
        self.latency_count += 1;

        self.latency_min = Some(
            self.latency_min.map_or(duration, |min| min.min(duration))
        );
        self.latency_max = Some(
            self.latency_max.map_or(duration, |max| max.max(duration))
        );

        // Errors
        if is_error {
            self.error_count += 1;
        }

        // By operations
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
        println!("=== Metrics from Traces ===\n");

        println!("Overall statistics:");
        println!("  Total spans: {}", self.latency_count);
        println!("  Average latency: {:?}", self.avg_latency());
        println!("  Min latency: {:?}", self.latency_min.unwrap_or_default());
        println!("  Max latency: {:?}", self.latency_max.unwrap_or_default());
        println!("  Errors: {} ({:.2}%)", self.error_count, self.error_rate());

        println!("\nBy operations:");
        for (op, count) in &self.operation_counts {
            let p50 = self.percentile(op, 50.0);
            let p99 = self.percentile(op, 99.0);

            println!(
                "  {}: {} calls, p50={:?}, p99={:?}",
                op,
                count,
                p50.unwrap_or_default(),
                p99.unwrap_or_default()
            );
        }
    }
}

/// Tracing trading operations with metrics
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
        // Validation
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(2));
        self.metrics.record_span("validate", start.elapsed(), false);

        // Risk check
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(3));
        let order_value = price * qty;
        let is_risk_error = order_value > 100_000.0;
        self.metrics.record_span("risk_check", start.elapsed(), is_risk_error);

        if is_risk_error {
            return Err("Risk limit exceeded".to_string());
        }

        // Execution
        let start = Instant::now();
        // Simulating varying latency
        let execution_time = if symbol.contains("BTC") { 10 } else { 15 };
        std::thread::sleep(Duration::from_millis(execution_time));
        self.metrics.record_span("execute", start.elapsed(), false);

        // Slightly adjust price (slippage)
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
    println!("=== Metrics from Trading System Traces ===\n");

    let mut trader = InstrumentedTrader::new();

    // Simulate a series of orders
    let orders = vec![
        ("BTCUSDT", "BUY", 50000.0, 0.1),
        ("ETHUSDT", "SELL", 3000.0, 1.0),
        ("BTCUSDT", "BUY", 50100.0, 0.5),
        ("SOLUSDT", "BUY", 100.0, 10.0),
        ("BTCUSDT", "SELL", 50050.0, 0.2),
        ("ETHUSDT", "BUY", 2990.0, 2.0),
        // Order exceeding limit
        ("BTCUSDT", "BUY", 50000.0, 10.0),
    ];

    for (symbol, side, price, qty) in orders {
        match trader.execute_order(symbol, side, price, qty) {
            Ok(executed_price) => {
                println!("{} {} {} @ ${:.2} -> ${:.2}",
                    side, qty, symbol, price, executed_price);
            }
            Err(e) => {
                println!("{} {} {} @ ${:.2} -> ERROR: {}",
                    side, qty, symbol, price, e);
            }
        }
    }

    println!();
    trader.get_metrics().print_summary();
}
```

## WebSocket Connection Tracing

For trading systems, WebSocket connection tracing is important:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// WebSocket connection state
#[derive(Debug, Clone)]
enum WsConnectionState {
    Connecting,
    Connected,
    Authenticated,
    Subscribed,
    Disconnected,
}

/// Span for WebSocket events
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

/// Tracer for WebSocket market data
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

            // Simulating subscription
            std::thread::sleep(Duration::from_millis(5));

            span.set_state(WsConnectionState::Subscribed);
            span.add_event("subscribe_completed");
        }
    }

    fn trace_message(&mut self, msg_type: &str, symbol: &str, latency_us: u64) {
        self.message_count += 1;

        // Log every 100th message
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
        println!("\n=== WebSocket Tracing ===\n");

        for span in &self.connection_spans {
            println!("Connection: {}", span.operation);
            println!("  ID: {}", span.id);
            println!("  State: {:?}", span.state);
            println!("  Duration: {:?}", span.duration());

            println!("  Attributes:");
            for (k, v) in &span.attributes {
                println!("    {}: {}", k, v);
            }

            println!("  Events:");
            for (event, time) in &span.events {
                let offset = time.duration_since(span.start_time);
                println!("    +{:?}: {}", offset, event);
            }
        }

        println!("\nStatistics:");
        println!("  Messages processed: {}", self.message_count);
        println!("  Errors: {}", self.error_count);
    }
}

/// Demonstrating WebSocket tracing
fn ws_tracing_demo() {
    println!("=== WebSocket Market Data Tracing ===\n");

    let mut tracer = WsMarketDataTracer::new();

    // Connection
    println!("Connecting to Binance...");
    let span = tracer.trace_connection("binance", "wss://stream.binance.com:9443");
    std::thread::sleep(Duration::from_millis(50));
    span.set_state(WsConnectionState::Connected);

    // Authentication
    std::thread::sleep(Duration::from_millis(10));
    span.set_state(WsConnectionState::Authenticated);
    span.add_event("auth_success");

    // Subscription
    tracer.trace_subscription(&["BTCUSDT", "ETHUSDT", "SOLUSDT"]);

    // Simulating message reception
    println!("\nReceiving market data...");
    for i in 0..350 {
        let symbol = match i % 3 {
            0 => "BTCUSDT",
            1 => "ETHUSDT",
            _ => "SOLUSDT",
        };

        // Random latency
        let latency = 50 + (i % 200) as u64;
        tracer.trace_message("trade", symbol, latency);

        // Occasional errors
        if i == 150 {
            tracer.trace_error("message_parse_error");
        }
    }

    // Disconnect
    tracer.trace_disconnect("normal_closure");

    tracer.print_summary();
}

fn main() {
    ws_tracing_demo();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Trace** | Complete request path through the system |
| **Span** | Individual operation within a trace |
| **Context Propagation** | Passing context between services |
| **Baggage** | User data passed with context |
| **Jaeger** | System for storing and visualizing traces |
| **OpenTelemetry** | Standard for collecting telemetry |
| **Span Attributes** | Operation metadata |
| **Span Events** | Events within an operation |

## Practical Exercises

1. **Order Tracing Through System**: Create a system that:
   - Tracks an order from reception to execution
   - Records time for each stage
   - Shows bottlenecks in processing
   - Generates alerts when latency is exceeded

2. **Correlation ID for Arbitrage**: Implement:
   - Linking orders across different exchanges
   - Common trace for arbitrage trade
   - Total arbitrage latency calculation
   - Success analysis from trace data

3. **WebSocket Stream Monitoring**: Create:
   - Tracing connections to exchanges
   - Tracking data delays
   - Connection break detection
   - Data quality metrics

4. **Trace Visualization**: Implement:
   - Export to Jaeger format
   - Building operation timeline
   - Grouping spans by services
   - Searching for anomalous traces

## Homework

1. **Full Trading System Tracing**: Create a system:
   - Trace all order processing stages
   - Integration with Jaeger (or mock)
   - Alerts on abnormal latency
   - Dashboard with metrics from traces
   - Search by trace_id and order_id

2. **Distributed Microservices Tracing**: Implement:
   - 3-4 services (gateway, risk, execution, notification)
   - Context passing between services
   - Single trace for a request
   - Sampling for high-load scenarios
   - Error handling with trace preservation

3. **Performance Analysis from Traces**: Create a tool:
   - Collect traces for a period
   - Calculate percentiles (p50, p95, p99)
   - Identify slow operations
   - Compare performance over time
   - Optimization recommendations

4. **Strategy Tracing**: Implement:
   - Trace for each strategy signal
   - Linking signal to execution
   - Signal-to-execution time analysis
   - Correlating delays with slippage
   - Optimization based on trace data

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../342-*/en.md)
