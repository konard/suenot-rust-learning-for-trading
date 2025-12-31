# День 109: Circuit Breaker — защита от каскадных сбоев

## Аналогия из трейдинга

Представь, что ты торгуешь через API биржи. Внезапно биржа начинает отвечать с задержкой или возвращать ошибки. Если ты продолжишь слать запросы, произойдёт несколько плохих вещей:

1. **Перегрузка системы** — твои повторные попытки только усугубляют проблему
2. **Каскадный сбой** — другие части твоей системы (риск-менеджмент, логирование) тоже начинают падать
3. **Потеря денег** — ордера застревают, позиции не закрываются

**Circuit Breaker** работает как автоматический выключатель в электрощитке: при перегрузке он "размыкает цепь", давая системе время восстановиться.

## Теоретические основы

### Три состояния Circuit Breaker

```
     ┌─────────────────────────────────────────────────────────┐
     │                                                         │
     ▼                                                         │
┌─────────┐   failure_threshold   ┌────────┐   timeout   ┌─────────────┐
│ CLOSED  │ ─────────────────────►│  OPEN  │────────────►│ HALF-OPEN   │
│(работаем)│                      │(ждём)  │             │(проверяем)  │
└─────────┘                       └────────┘             └─────────────┘
     ▲                                 ▲                       │
     │         success                 │      failure          │
     └─────────────────────────────────┴───────────────────────┘
```

- **Closed (Замкнут)**: Нормальная работа, запросы проходят
- **Open (Разомкнут)**: Сбой обнаружен, запросы блокируются
- **Half-Open (Полуоткрыт)**: Пробуем один запрос для проверки восстановления

### Ключевые параметры

| Параметр | Описание | Типичное значение для трейдинга |
|----------|----------|--------------------------------|
| `failure_threshold` | Сколько сбоев до размыкания | 3-5 |
| `success_threshold` | Сколько успехов для восстановления | 2-3 |
| `timeout` | Время ожидания в открытом состоянии | 30-60 сек |

## Базовая реализация

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Проверяем, прошёл ли таймаут
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0; // Сброс счётчика ошибок
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    println!("Circuit CLOSED: Сервис восстановлен");
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                    println!("Circuit OPEN: Слишком много ошибок!");
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
                println!("Circuit OPEN: Проверка провалена");
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.state
    }
}

fn main() {
    let mut cb = CircuitBreaker::new(
        3,                      // 3 ошибки для размыкания
        2,                      // 2 успеха для восстановления
        Duration::from_secs(5), // 5 секунд таймаут
    );

    // Симуляция работы
    println!("Состояние: {:?}", cb.state());

    // Имитируем сбои
    for i in 1..=4 {
        if cb.can_execute() {
            println!("Попытка {}: выполняем запрос...", i);
            cb.record_failure();
        } else {
            println!("Попытка {}: Circuit открыт, пропускаем", i);
        }
        println!("Состояние: {:?}\n", cb.state());
    }
}
```

## Применение для API биржи

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }
}

#[derive(Debug)]
enum ExchangeError {
    Timeout,
    RateLimit,
    ServerError,
    CircuitOpen,
}

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
struct OrderResult {
    order_id: String,
    status: String,
}

struct ExchangeClient {
    circuit_breaker: CircuitBreaker,
    request_count: u32,
}

impl ExchangeClient {
    fn new() -> Self {
        ExchangeClient {
            circuit_breaker: CircuitBreaker::new(
                3,                       // 3 ошибки — размыкаем
                2,                       // 2 успеха — восстанавливаем
                Duration::from_secs(30), // 30 секунд ожидания
            ),
            request_count: 0,
        }
    }

    fn place_order(&mut self, order: &Order) -> Result<OrderResult, ExchangeError> {
        // Проверяем состояние circuit breaker
        if !self.circuit_breaker.can_execute() {
            return Err(ExchangeError::CircuitOpen);
        }

        // Выполняем запрос (симуляция)
        let result = self.execute_request(order);

        // Обновляем состояние circuit breaker
        match &result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(_) => self.circuit_breaker.record_failure(),
        }

        result
    }

    fn execute_request(&mut self, order: &Order) -> Result<OrderResult, ExchangeError> {
        self.request_count += 1;

        // Симуляция: каждый 3-й запрос — ошибка
        if self.request_count % 3 == 0 {
            Err(ExchangeError::Timeout)
        } else {
            Ok(OrderResult {
                order_id: format!("ORD-{}", self.request_count),
                status: "FILLED".to_string(),
            })
        }
    }

    fn get_circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state
    }
}

fn main() {
    let mut client = ExchangeClient::new();

    let order = Order {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.1,
        price: 42000.0,
    };

    // Попытки отправить ордера
    for i in 1..=10 {
        println!("--- Попытка {} ---", i);
        println!("Circuit state: {:?}", client.get_circuit_state());

        match client.place_order(&order) {
            Ok(result) => println!("Успех: {:?}", result),
            Err(e) => println!("Ошибка: {:?}", e),
        }
        println!();
    }
}
```

## Circuit Breaker с метриками

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
struct CircuitMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    rejected_requests: u64,  // Отклонено из-за открытого circuit
    state_changes: Vec<(Instant, CircuitState)>,
    response_times: VecDeque<Duration>,  // Скользящее окно
}

impl CircuitMetrics {
    fn new() -> Self {
        CircuitMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rejected_requests: 0,
            state_changes: Vec::new(),
            response_times: VecDeque::with_capacity(100),
        }
    }

    fn record_request(&mut self, success: bool, response_time: Duration) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // Храним последние 100 времён ответа
        if self.response_times.len() >= 100 {
            self.response_times.pop_front();
        }
        self.response_times.push_back(response_time);
    }

    fn record_rejection(&mut self) {
        self.rejected_requests += 1;
    }

    fn record_state_change(&mut self, new_state: CircuitState) {
        self.state_changes.push((Instant::now(), new_state));
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    fn avg_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.response_times.iter().sum();
        total / self.response_times.len() as u32
    }
}

struct MonitoredCircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
    metrics: CircuitMetrics,
}

impl MonitoredCircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        MonitoredCircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
            metrics: CircuitMetrics::new(),
        }
    }

    fn execute<F, T, E>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Проверяем, можно ли выполнять
        if !self.can_execute() {
            self.metrics.record_rejection();
            return Err(CircuitBreakerError::CircuitOpen);
        }

        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        match &result {
            Ok(_) => {
                self.metrics.record_request(true, elapsed);
                self.record_success();
                result.map_err(CircuitBreakerError::ServiceError)
            }
            Err(_) => {
                self.metrics.record_request(false, elapsed);
                self.record_failure();
                result.map_err(CircuitBreakerError::ServiceError)
            }
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.transition_to(CircuitState::HalfOpen);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn transition_to(&mut self, new_state: CircuitState) {
        if self.state != new_state {
            self.state = new_state;
            self.metrics.record_state_change(new_state);
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.transition_to(CircuitState::Closed);
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.transition_to(CircuitState::Open);
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.transition_to(CircuitState::Open);
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn get_metrics(&self) -> &CircuitMetrics {
        &self.metrics
    }

    fn print_status(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║       CIRCUIT BREAKER STATUS          ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ State:          {:>20?} ║", self.state);
        println!("║ Total requests: {:>20} ║", self.metrics.total_requests);
        println!("║ Successful:     {:>20} ║", self.metrics.successful_requests);
        println!("║ Failed:         {:>20} ║", self.metrics.failed_requests);
        println!("║ Rejected:       {:>20} ║", self.metrics.rejected_requests);
        println!("║ Success rate:   {:>19.1}% ║", self.metrics.success_rate());
        println!("║ Avg response:   {:>17.2?} ║", self.metrics.avg_response_time());
        println!("╚═══════════════════════════════════════╝");
    }
}

#[derive(Debug)]
enum CircuitBreakerError<E> {
    CircuitOpen,
    ServiceError(E),
}

fn main() {
    let mut cb = MonitoredCircuitBreaker::new(
        3,
        2,
        Duration::from_secs(5),
    );

    // Симуляция запросов
    let mut fail_next = false;
    for i in 1..=15 {
        let result: Result<String, &str> = cb.execute(|| {
            if i % 4 == 0 {
                Err("Timeout")
            } else {
                Ok(format!("Response {}", i))
            }
        });

        match result {
            Ok(v) => println!("#{}: Успех - {}", i, v),
            Err(CircuitBreakerError::CircuitOpen) => println!("#{}: CIRCUIT OPEN", i),
            Err(CircuitBreakerError::ServiceError(e)) => println!("#{}: Ошибка - {}", i, e),
        }
    }

    println!();
    cb.print_status();
}
```

## Circuit Breaker для нескольких сервисов

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            timeout,
            last_failure_time: None,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.state
    }
}

/// Менеджер Circuit Breaker для множества сервисов
struct CircuitBreakerRegistry {
    breakers: HashMap<String, CircuitBreaker>,
    default_failure_threshold: u32,
    default_success_threshold: u32,
    default_timeout: Duration,
}

impl CircuitBreakerRegistry {
    fn new() -> Self {
        CircuitBreakerRegistry {
            breakers: HashMap::new(),
            default_failure_threshold: 5,
            default_success_threshold: 2,
            default_timeout: Duration::from_secs(30),
        }
    }

    fn register(&mut self, name: &str, failure_threshold: u32, success_threshold: u32, timeout: Duration) {
        self.breakers.insert(
            name.to_string(),
            CircuitBreaker::new(failure_threshold, success_threshold, timeout),
        );
    }

    fn get_or_create(&mut self, name: &str) -> &mut CircuitBreaker {
        if !self.breakers.contains_key(name) {
            self.breakers.insert(
                name.to_string(),
                CircuitBreaker::new(
                    self.default_failure_threshold,
                    self.default_success_threshold,
                    self.default_timeout,
                ),
            );
        }
        self.breakers.get_mut(name).unwrap()
    }

    fn print_all_states(&self) {
        println!("╔════════════════════════════════════════════╗");
        println!("║         CIRCUIT BREAKER REGISTRY           ║");
        println!("╠════════════════════╦═══════════════════════╣");
        println!("║ Service            ║ State                 ║");
        println!("╠════════════════════╬═══════════════════════╣");
        for (name, cb) in &self.breakers {
            println!("║ {:18} ║ {:21?} ║", name, cb.state());
        }
        println!("╚════════════════════╩═══════════════════════╝");
    }
}

/// Трейдинг система с несколькими биржами
struct TradingSystem {
    registry: CircuitBreakerRegistry,
}

impl TradingSystem {
    fn new() -> Self {
        let mut registry = CircuitBreakerRegistry::new();

        // Разные настройки для разных бирж
        registry.register("binance", 3, 2, Duration::from_secs(30));
        registry.register("bybit", 5, 3, Duration::from_secs(60));
        registry.register("kraken", 4, 2, Duration::from_secs(45));

        TradingSystem { registry }
    }

    fn execute_on_exchange<F, T>(&mut self, exchange: &str, operation: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let cb = self.registry.get_or_create(exchange);

        if !cb.can_execute() {
            return Err(format!("{} circuit is OPEN", exchange));
        }

        let result = operation();

        match &result {
            Ok(_) => cb.record_success(),
            Err(_) => cb.record_failure(),
        }

        result
    }

    fn get_price(&mut self, exchange: &str, symbol: &str) -> Result<f64, String> {
        self.execute_on_exchange(exchange, || {
            // Симуляция API вызова
            match (exchange, symbol) {
                ("binance", "BTC/USDT") => Ok(42000.0),
                ("bybit", "BTC/USDT") => Ok(42010.0),
                _ => Err("Symbol not found".to_string()),
            }
        })
    }

    fn print_status(&self) {
        self.registry.print_all_states();
    }
}

fn main() {
    let mut system = TradingSystem::new();

    // Получаем цены с разных бирж
    let exchanges = ["binance", "bybit", "kraken"];
    let symbol = "BTC/USDT";

    for exchange in &exchanges {
        match system.get_price(exchange, symbol) {
            Ok(price) => println!("{}: {} = ${:.2}", exchange, symbol, price),
            Err(e) => println!("{}: Ошибка - {}", exchange, e),
        }
    }

    println!();
    system.print_status();
}
```

## Практические упражнения

### Упражнение 1: Circuit Breaker с экспоненциальным backoff

Добавь логику, при которой timeout увеличивается с каждым переходом в Open состояние:
- 1-й раз: 30 сек
- 2-й раз: 60 сек
- 3-й раз: 120 сек
- Максимум: 5 минут

### Упражнение 2: Sliding Window Circuit Breaker

Вместо простого счётчика ошибок, реализуй Circuit Breaker на основе скользящего окна:
- Храни результаты последних N запросов (например, 10)
- Если процент ошибок превышает threshold (например, 50%), открывай circuit

### Упражнение 3: Circuit Breaker для WebSocket

Адаптируй Circuit Breaker для WebSocket соединений:
- Отслеживай потерю соединения
- Автоматически переподключайся с backoff
- Переключайся на резервный endpoint при длительном сбое

### Упражнение 4: Health Check Endpoint

Добавь к Circuit Breaker специальный health check:
- В состоянии HalfOpen отправляй сначала health check запрос, а не реальный
- Только если health check успешен, разрешай реальные запросы

## Домашнее задание

1. **Реализуй Bulkhead pattern**: Ограничь количество одновременных запросов к каждому сервису (например, max 10 concurrent requests)

2. **Добавь алертинг**: При переходе в Open состояние отправляй уведомление (симулируй через print)

3. **Создай Fallback механизм**: Если основная биржа недоступна, автоматически переключайся на резервную

4. **Реализуй Retry с Circuit Breaker**: До размыкания circuit делай 3 повторные попытки с exponential backoff

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Circuit Breaker | Паттерн защиты от каскадных сбоев |
| Три состояния | Closed → Open → HalfOpen → Closed |
| failure_threshold | Количество ошибок для размыкания |
| success_threshold | Количество успехов для восстановления |
| timeout | Время ожидания перед проверкой |
| Метрики | Мониторинг состояния и производительности |
| Registry | Управление несколькими Circuit Breaker |

## Навигация

[← Предыдущий день](../108-retry-patterns/ru.md) | [Следующий день →](../110-fallback-strategies/ru.md)
