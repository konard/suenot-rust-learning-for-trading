# День 207: Реконнект: восстанавливаем соединение

## Аналогия из трейдинга

Представь, что ты запустил алготрейдингового бота, который получает данные о ценах в реальном времени через WebSocket. Внезапно соединение обрывается — сервер биржи перезапустился, сеть дала сбой или превышен таймаут соединения. Если твой бот не переподключится автоматически, ты пропустишь критические движения цен и торговые возможности. В реальном трейдинге **логика реконнекта** — это не опция, а необходимость для выживания.

Надёжная торговая система должна:
- Определять, когда соединение потеряно
- Автоматически пытаться переподключиться
- Использовать экспоненциальную задержку, чтобы не перегружать сервер
- Восстанавливать подписки после переподключения
- Корректно обрабатывать частичные сбои

## Что такое реконнект?

Реконнект — это процесс автоматического восстановления сетевого соединения после его потери. В асинхронном Rust с tokio мы реализуем реконнект с помощью:

1. **Отслеживания состояния соединения** — знаем, подключены мы или нет
2. **Логики повторных попыток** — пытаемся переподключиться с подходящими задержками
3. **Экспоненциальной задержки** — увеличиваем паузы между попытками, чтобы не перегружать сервер
4. **Ограничения количества попыток** — сдаёмся после слишком многих неудач
5. **Восстановления состояния** — переподписываемся на каналы после реконнекта

## Простой паттерн реконнекта

```rust
use std::time::Duration;
use tokio::time::sleep;

/// Конфигурация поведения реконнекта
#[derive(Debug, Clone)]
struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    max_retries: u32,
    backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            max_retries: 10,
            backoff_multiplier: 2.0,
        }
    }
}

/// Симулирует подключение к бирже (может не удаться)
async fn connect_to_exchange(url: &str) -> Result<String, String> {
    // Имитируем сетевую задержку
    sleep(Duration::from_millis(50)).await;

    // Симулируем случайные сбои подключения (30% неудач)
    if rand::random::<f32>() < 0.3 {
        Err(format!("Не удалось подключиться к {}", url))
    } else {
        Ok(format!("Подключено к {}", url))
    }
}

/// Цикл реконнекта с экспоненциальной задержкой
async fn connect_with_retry(url: &str, config: &ReconnectConfig) -> Result<String, String> {
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        attempts += 1;
        println!("Попытка подключения {} к {}...", attempts, url);

        match connect_to_exchange(url).await {
            Ok(connection) => {
                println!("Успешно подключились после {} попыток!", attempts);
                return Ok(connection);
            }
            Err(e) => {
                println!("Попытка {} не удалась: {}", attempts, e);

                if attempts >= config.max_retries {
                    return Err(format!(
                        "Не удалось подключиться после {} попыток",
                        config.max_retries
                    ));
                }

                println!("Повторная попытка через {:?}...", delay);
                sleep(delay).await;

                // Экспоненциальная задержка
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_multiplier)
                        .min(config.max_delay.as_secs_f64())
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = ReconnectConfig::default();

    match connect_with_retry("wss://exchange.example.com/ws", &config).await {
        Ok(conn) => println!("Итоговый результат: {}", conn),
        Err(e) => println!("Подключение не удалось: {}", e),
    }
}
```

## Фид цен с реконнектом

Вот более реалистичный пример — клиент фида цен с автоматическим реконнектом:

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Instant};

/// Представляет обновление цены с биржи
#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: Instant,
}

/// Состояние соединения
#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
}

/// Клиент фида цен с автоматическим реконнектом
struct PriceFeedClient {
    url: String,
    state: Arc<RwLock<ConnectionState>>,
    subscriptions: Arc<RwLock<Vec<String>>>,
    config: ReconnectConfig,
}

#[derive(Debug, Clone)]
struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    max_retries: u32,
    backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        ReconnectConfig {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            max_retries: 5,
            backoff_multiplier: 2.0,
        }
    }
}

impl PriceFeedClient {
    fn new(url: &str, config: ReconnectConfig) -> Self {
        PriceFeedClient {
            url: url.to_string(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Подписаться на торговую пару
    async fn subscribe(&self, symbol: &str) {
        let mut subs = self.subscriptions.write().await;
        if !subs.contains(&symbol.to_string()) {
            subs.push(symbol.to_string());
            println!("Подписка на {}", symbol);
        }
    }

    /// Симуляция установки соединения
    async fn establish_connection(&self) -> Result<(), String> {
        // Симуляция попытки подключения
        sleep(Duration::from_millis(100)).await;

        // 20% вероятность неудачи
        if rand::random::<f32>() < 0.2 {
            Err("Соединение отклонено".to_string())
        } else {
            Ok(())
        }
    }

    /// Восстановление подписок после реконнекта
    async fn restore_subscriptions(&self) -> Result<(), String> {
        let subs = self.subscriptions.read().await;
        for symbol in subs.iter() {
            println!("Восстановление подписки: {}", symbol);
            // Симуляция запроса на подписку
            sleep(Duration::from_millis(50)).await;
        }
        println!("Все {} подписок восстановлены", subs.len());
        Ok(())
    }

    /// Основной цикл соединения с автоматическим реконнектом
    async fn run(&self, price_tx: mpsc::Sender<PriceUpdate>) -> Result<(), String> {
        loop {
            // Обновляем состояние на Connecting
            {
                let mut state = self.state.write().await;
                *state = ConnectionState::Connecting;
            }

            // Пробуем подключиться с повторными попытками
            let mut attempts = 0;
            let mut delay = self.config.initial_delay;

            let connected = loop {
                attempts += 1;

                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Reconnecting { attempt: attempts };
                }

                println!("Попытка подключения {}...", attempts);

                match self.establish_connection().await {
                    Ok(()) => {
                        println!("Подключено к {}", self.url);
                        break true;
                    }
                    Err(e) => {
                        println!("Подключение не удалось: {}", e);

                        if attempts >= self.config.max_retries {
                            println!("Достигнуто максимальное количество попыток");
                            break false;
                        }

                        println!("Ожидание {:?} перед повторной попыткой...", delay);
                        sleep(delay).await;

                        delay = Duration::from_secs_f64(
                            (delay.as_secs_f64() * self.config.backoff_multiplier)
                                .min(self.config.max_delay.as_secs_f64())
                        );
                    }
                }
            };

            if !connected {
                return Err("Не удалось установить соединение".to_string());
            }

            // Обновляем состояние на Connected
            {
                let mut state = self.state.write().await;
                *state = ConnectionState::Connected;
            }

            // Восстанавливаем подписки
            self.restore_subscriptions().await?;

            // Симуляция получения обновлений цен
            let result = self.receive_prices(&price_tx).await;

            match result {
                Ok(()) => {
                    // Корректное завершение
                    println!("Соединение закрыто корректно");
                    return Ok(());
                }
                Err(e) => {
                    // Соединение потеряно, будем переподключаться
                    println!("Соединение потеряно: {}. Переподключение...", e);

                    {
                        let mut state = self.state.write().await;
                        *state = ConnectionState::Disconnected;
                    }

                    // Небольшая пауза перед попыткой реконнекта
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Симуляция получения обновлений цен (может завершиться ошибкой для симуляции отключения)
    async fn receive_prices(&self, price_tx: &mpsc::Sender<PriceUpdate>) -> Result<(), String> {
        let subs = self.subscriptions.read().await;
        let symbols: Vec<String> = subs.clone();
        drop(subs);

        for i in 0..10 {
            // Симуляция случайного отключения (10% на каждую итерацию)
            if rand::random::<f32>() < 0.1 {
                return Err("Соединение сброшено сервером".to_string());
            }

            // Генерируем обновления цен для всех подписанных символов
            for symbol in &symbols {
                let base_price = match symbol.as_str() {
                    "BTC/USDT" => 42000.0,
                    "ETH/USDT" => 2500.0,
                    _ => 100.0,
                };

                let price_update = PriceUpdate {
                    symbol: symbol.clone(),
                    bid: base_price + (i as f64 * 10.0) - 5.0,
                    ask: base_price + (i as f64 * 10.0) + 5.0,
                    timestamp: Instant::now(),
                };

                if price_tx.send(price_update).await.is_err() {
                    return Ok(()); // Получатель закрыт, корректное завершение
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let config = ReconnectConfig::default();
    let client = Arc::new(PriceFeedClient::new("wss://exchange.example.com/ws", config));

    // Подписываемся на торговые пары
    client.subscribe("BTC/USDT").await;
    client.subscribe("ETH/USDT").await;

    // Канал для обновлений цен
    let (price_tx, mut price_rx) = mpsc::channel(100);

    // Запускаем задачу соединения
    let client_clone = Arc::clone(&client);
    let connection_task = tokio::spawn(async move {
        client_clone.run(price_tx).await
    });

    // Обрабатываем обновления цен
    let processing_task = tokio::spawn(async move {
        while let Some(update) = price_rx.recv().await {
            println!(
                "Цена: {} bid={:.2} ask={:.2}",
                update.symbol, update.bid, update.ask
            );
        }
    });

    // Ждём завершения задачи соединения
    match connection_task.await {
        Ok(Ok(())) => println!("Задача соединения завершена успешно"),
        Ok(Err(e)) => println!("Задача соединения завершилась ошибкой: {}", e),
        Err(e) => println!("Паника в задаче соединения: {}", e),
    }

    processing_task.abort();
}
```

## Реконнект с паттерном Circuit Breaker

Для продакшен-систем часто комбинируют реконнект с circuit breaker для предотвращения каскадных сбоев:

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};

/// Состояния circuit breaker
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,      // Нормальная работа
    Open,        // Сбой, отклоняем запросы
    HalfOpen,    // Тестируем, восстановился ли сервис
}

/// Circuit breaker для управления соединениями
struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
            failure_threshold,
            reset_timeout,
        }
    }

    /// Проверить, стоит ли пробовать подключение
    async fn should_attempt(&self) -> bool {
        let state = self.state.read().await;

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Проверяем, прошёл ли таймаут сброса
                let last = self.last_failure.read().await;
                if let Some(last_time) = *last {
                    if last_time.elapsed() >= self.reset_timeout {
                        drop(state);
                        drop(last);
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Записать успешное подключение
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut count = self.failure_count.write().await;

        *state = CircuitState::Closed;
        *count = 0;

        println!("Circuit breaker: Подключение успешно, circuit CLOSED");
    }

    /// Записать неудачное подключение
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut count = self.failure_count.write().await;
        let mut last = self.last_failure.write().await;

        *count += 1;
        *last = Some(Instant::now());

        if *count >= self.failure_threshold {
            *state = CircuitState::Open;
            println!(
                "Circuit breaker: {} сбоев, circuit OPEN на {:?}",
                *count, self.reset_timeout
            );
        }
    }
}

/// Соединение с биржей с circuit breaker
struct ExchangeConnection {
    url: String,
    circuit_breaker: CircuitBreaker,
}

impl ExchangeConnection {
    fn new(url: &str) -> Self {
        ExchangeConnection {
            url: url.to_string(),
            circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(10)),
        }
    }

    /// Попытка подключения с защитой circuit breaker
    async fn connect(&self) -> Result<(), String> {
        if !self.circuit_breaker.should_attempt().await {
            return Err("Circuit breaker в состоянии OPEN, подключение отклонено".to_string());
        }

        // Симуляция попытки подключения
        sleep(Duration::from_millis(100)).await;

        // 40% вероятность сбоя для демонстрации
        if rand::random::<f32>() < 0.4 {
            self.circuit_breaker.record_failure().await;
            Err("Подключение не удалось".to_string())
        } else {
            self.circuit_breaker.record_success().await;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let connection = ExchangeConnection::new("wss://exchange.example.com/ws");

    // Пробуем несколько подключений
    for i in 1..=10 {
        println!("\n--- Попытка {} ---", i);

        match connection.connect().await {
            Ok(()) => println!("Подключено успешно!"),
            Err(e) => println!("Сбой: {}", e),
        }

        sleep(Duration::from_secs(2)).await;
    }
}
```

## Менеджер реконнекта для нескольких бирж

Для торговых систем, подключающихся к нескольким биржам:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Статус одного соединения с биржей
#[derive(Debug, Clone)]
struct ExchangeStatus {
    name: String,
    connected: bool,
    last_message: Option<std::time::Instant>,
    reconnect_attempts: u32,
}

/// Менеджер для нескольких соединений с биржами
struct MultiExchangeManager {
    exchanges: Arc<RwLock<HashMap<String, ExchangeStatus>>>,
}

impl MultiExchangeManager {
    fn new() -> Self {
        MultiExchangeManager {
            exchanges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Добавить биржу для управления
    async fn add_exchange(&self, name: &str) {
        let mut exchanges = self.exchanges.write().await;
        exchanges.insert(name.to_string(), ExchangeStatus {
            name: name.to_string(),
            connected: false,
            last_message: None,
            reconnect_attempts: 0,
        });
    }

    /// Симуляция подключения к бирже
    async fn connect_exchange(&self, name: &str) -> Result<(), String> {
        sleep(Duration::from_millis(100)).await;

        // Разные вероятности сбоя для разных бирж
        let failure_rate = match name {
            "Binance" => 0.1,
            "Kraken" => 0.2,
            "Coinbase" => 0.15,
            _ => 0.3,
        };

        if rand::random::<f32>() < failure_rate {
            Err(format!("{}: соединение отклонено", name))
        } else {
            Ok(())
        }
    }

    /// Подключиться ко всем биржам с индивидуальным реконнектом
    async fn connect_all(&self) {
        let exchanges = self.exchanges.read().await;
        let exchange_names: Vec<String> = exchanges.keys().cloned().collect();
        drop(exchanges);

        let mut handles = Vec::new();

        for name in exchange_names {
            let manager = Arc::new(self.exchanges.clone());
            let exchange_name = name.clone();

            let handle = tokio::spawn(async move {
                let mut attempts = 0;
                let max_attempts = 5;
                let mut delay = Duration::from_millis(500);

                loop {
                    attempts += 1;
                    println!("[{}] Попытка подключения {}...", exchange_name, attempts);

                    // Симуляция подключения
                    sleep(Duration::from_millis(100)).await;
                    let success = rand::random::<f32>() > 0.3;

                    if success {
                        let mut exchanges = manager.write().await;
                        if let Some(status) = exchanges.get_mut(&exchange_name) {
                            status.connected = true;
                            status.last_message = Some(std::time::Instant::now());
                            status.reconnect_attempts = attempts;
                        }
                        println!("[{}] Подключено после {} попыток!", exchange_name, attempts);
                        break;
                    } else {
                        println!("[{}] Сбой, повтор через {:?}...", exchange_name, delay);

                        if attempts >= max_attempts {
                            println!("[{}] Достигнуто максимум попыток!", exchange_name);
                            break;
                        }

                        sleep(delay).await;
                        delay = std::cmp::min(delay * 2, Duration::from_secs(10));
                    }
                }
            });

            handles.push(handle);
        }

        // Ждём все подключения
        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Получить сводку статуса
    async fn get_status_summary(&self) -> String {
        let exchanges = self.exchanges.read().await;
        let connected: Vec<_> = exchanges.values()
            .filter(|e| e.connected)
            .map(|e| e.name.as_str())
            .collect();
        let disconnected: Vec<_> = exchanges.values()
            .filter(|e| !e.connected)
            .map(|e| e.name.as_str())
            .collect();

        format!(
            "Подключены: {:?}, Отключены: {:?}",
            connected, disconnected
        )
    }
}

#[tokio::main]
async fn main() {
    let manager = MultiExchangeManager::new();

    // Добавляем биржи для управления
    manager.add_exchange("Binance").await;
    manager.add_exchange("Kraken").await;
    manager.add_exchange("Coinbase").await;
    manager.add_exchange("FTX").await;

    println!("Запуск подключения к нескольким биржам...\n");

    // Подключаемся ко всем биржам параллельно
    manager.connect_all().await;

    println!("\n{}", manager.get_status_summary().await);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Реконнект | Автоматическое восстановление потерянного соединения |
| Экспоненциальная задержка | Увеличение пауз между повторными попытками |
| Circuit Breaker | Паттерн для предотвращения каскадных сбоев |
| Восстановление состояния | Переподписка на каналы после реконнекта |
| Состояние соединения | Отслеживание: подключены, отключены или переподключаемся |
| Мульти-подключение | Управление соединениями к нескольким биржам одновременно |

## Домашнее задание

1. **Базовый реконнект**: Реализуй простой клиент фида цен, который:
   - Подключается к симулированной бирже
   - Автоматически переподключается при потере соединения
   - Использует экспоненциальную задержку с максимумом 30 секунд
   - Логирует все изменения состояния соединения

2. **Восстановление подписок**: Расширь клиент из задания 1, чтобы он:
   - Хранил список подписанных торговых пар
   - Автоматически переподписывался на все пары после реконнекта
   - Корректно обрабатывал ошибки подписки

3. **Торговая система с Circuit Breaker**: Создай торговую систему, которая:
   - Подключается к 3 биржам (симулированным)
   - Использует circuit breaker для каждого соединения
   - Прекращает торговлю на бирже, когда её circuit открывается
   - Возобновляет торговлю, когда circuit закрывается
   - Логирует все изменения состояния circuit

4. **Монитор здоровья**: Построй монитор здоровья соединений, который:
   - Отслеживает время работы соединения для нескольких бирж
   - Вычисляет процент надёжности соединения
   - Оповещает, когда надёжность биржи падает ниже 95%
   - Предоставляет сводную панель всех метрик здоровья соединений

## Навигация

[← Предыдущий день](../206-ping-pong-keeping-connection/ru.md) | [Следующий день →](../208-processing-websocket-messages/ru.md)
