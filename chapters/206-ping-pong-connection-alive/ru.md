# День 206: Пинг-понг: поддержание соединения

## Аналогия из трейдинга

Представь, что ты подключен к бирже через WebSocket для получения котировок в реальном времени. Если соединение внезапно прервётся — ты можешь пропустить важное движение цены и потерять деньги. Но как биржа и твой клиент узнают, что соединение ещё живо?

Это как охранник на посту: он должен периодически нажимать кнопку "Я здесь", чтобы центральный пульт знал, что он не уснул. Если сигнал не приходит — срабатывает тревога. В сетевых протоколах эта кнопка называется **Ping**, а ответ — **Pong**.

В реальном трейдинге ping-pong механизм критически важен:
- **Биржи отключают неактивные соединения** через 30-60 секунд без пинга
- **Ты должен мониторить задержку** — если pong приходит слишком долго, возможно, сеть перегружена
- **При обрыве соединения** нужно быстро переподключиться, чтобы не пропустить сделки

## Что такое Ping-Pong?

Ping-Pong — это механизм **heartbeat** (сердцебиение) для TCP/WebSocket соединений:

1. **Ping** — сообщение от клиента серверу: "Ты ещё там?"
2. **Pong** — ответ от сервера: "Да, я здесь!"

```
Клиент                          Сервер
   |                               |
   |-------- PING --------------->|
   |                               |
   |<------- PONG ----------------|
   |                               |
   |-------- PING --------------->|
   |                               |
   |<------- PONG ----------------|
   |                               |
   ...
```

### Зачем нужен Ping-Pong?

| Проблема | Решение с Ping-Pong |
|----------|---------------------|
| Соединение "зависло" | Таймаут на pong — переподключение |
| Сервер перегружен | Увеличение latency пингов — предупреждение |
| NAT/Firewall закрывает соединение | Периодические пинги держат соединение открытым |
| Сбой на стороне сервера | Отсутствие pong = нужно переподключиться |

## Простой Ping-Pong с TCP

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, timeout, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct ConnectionStats {
    pings_sent: u64,
    pongs_received: u64,
    last_latency_ms: u64,
}

// Сервер: отвечает PONG на каждый PING
async fn run_price_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Сервер котировок запущен на 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Новое подключение: {}", addr);

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        println!("Клиент {} отключился", addr);
                        break;
                    }
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]);

                        if message.trim() == "PING" {
                            println!("Получен PING от {}", addr);
                            if let Err(e) = socket.write_all(b"PONG\n").await {
                                eprintln!("Ошибка отправки PONG: {}", e);
                                break;
                            }
                        } else if message.starts_with("SUBSCRIBE:") {
                            // Обработка подписки на котировки
                            let symbol = message.trim().strip_prefix("SUBSCRIBE:").unwrap();
                            println!("Клиент {} подписался на {}", addr, symbol);
                            socket.write_all(format!("SUBSCRIBED:{}\n", symbol).as_bytes()).await.ok();
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка чтения от {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}

// Клиент: отправляет PING и ждёт PONG
async fn run_trading_client() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Подключились к серверу котировок");

    let stats = Arc::new(Mutex::new(ConnectionStats {
        pings_sent: 0,
        pongs_received: 0,
        last_latency_ms: 0,
    }));

    let stats_ping = Arc::clone(&stats);

    // Задача для отправки пингов
    let (mut read_half, mut write_half) = stream.into_split();

    let ping_task = tokio::spawn(async move {
        let mut ping_interval = interval(Duration::from_secs(5));

        loop {
            ping_interval.tick().await;

            let start = std::time::Instant::now();

            if let Err(e) = write_half.write_all(b"PING\n").await {
                eprintln!("Ошибка отправки PING: {}", e);
                break;
            }

            let mut stats = stats_ping.lock().await;
            stats.pings_sent += 1;
            println!("Отправлен PING #{}", stats.pings_sent);
        }
    });

    // Задача для получения ответов
    let stats_pong = Arc::clone(&stats);
    let pong_task = tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        loop {
            match read_half.read(&mut buffer).await {
                Ok(0) => {
                    println!("Сервер закрыл соединение");
                    break;
                }
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);

                    if message.trim() == "PONG" {
                        let mut stats = stats_pong.lock().await;
                        stats.pongs_received += 1;
                        println!("Получен PONG #{}", stats.pongs_received);
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка чтения: {}", e);
                    break;
                }
            }
        }
    });

    // Ждём завершения
    tokio::select! {
        _ = ping_task => {},
        _ = pong_task => {},
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Запускаем сервер в отдельной задаче
    tokio::spawn(run_price_server());

    // Даём серверу время запуститься
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Запускаем клиента
    if let Err(e) = run_trading_client().await {
        eprintln!("Ошибка клиента: {}", e);
    }
}
```

## Ping-Pong с таймаутами для трейдинга

В реальном трейдинге критически важно быстро реагировать на потерю соединения:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{interval, timeout, Duration, Instant};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PONG_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_MISSED_PONGS: u32 = 3;

#[derive(Debug, Clone)]
enum ConnectionState {
    Connected,
    Degraded { missed_pongs: u32 },
    Disconnected,
}

#[derive(Debug)]
struct ExchangeConnection {
    state: ConnectionState,
    last_pong: Instant,
    latency_ms: Vec<u64>,
}

impl ExchangeConnection {
    fn new() -> Self {
        ExchangeConnection {
            state: ConnectionState::Connected,
            last_pong: Instant::now(),
            latency_ms: Vec::new(),
        }
    }

    fn record_pong(&mut self, latency: u64) {
        self.last_pong = Instant::now();
        self.latency_ms.push(latency);

        // Храним только последние 100 измерений
        if self.latency_ms.len() > 100 {
            self.latency_ms.remove(0);
        }

        self.state = ConnectionState::Connected;
    }

    fn miss_pong(&mut self) {
        match &mut self.state {
            ConnectionState::Connected => {
                self.state = ConnectionState::Degraded { missed_pongs: 1 };
            }
            ConnectionState::Degraded { missed_pongs } => {
                *missed_pongs += 1;
                if *missed_pongs >= MAX_MISSED_PONGS {
                    self.state = ConnectionState::Disconnected;
                }
            }
            ConnectionState::Disconnected => {}
        }
    }

    fn average_latency(&self) -> Option<f64> {
        if self.latency_ms.is_empty() {
            return None;
        }
        let sum: u64 = self.latency_ms.iter().sum();
        Some(sum as f64 / self.latency_ms.len() as f64)
    }

    fn is_healthy(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }
}

async fn exchange_client_with_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let connection = Arc::new(Mutex::new(ExchangeConnection::new()));

    let (mut read_half, mut write_half) = stream.into_split();
    let (pong_tx, mut pong_rx) = mpsc::channel::<Instant>(10);

    // Задача чтения
    let read_task = tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        loop {
            match read_half.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    if message.trim() == "PONG" {
                        pong_tx.send(Instant::now()).await.ok();
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Задача ping-pong с таймаутом
    let conn_clone = Arc::clone(&connection);
    let heartbeat_task = tokio::spawn(async move {
        let mut ping_interval = interval(PING_INTERVAL);

        loop {
            ping_interval.tick().await;

            let ping_time = Instant::now();

            // Отправляем PING
            if write_half.write_all(b"PING\n").await.is_err() {
                break;
            }

            // Ждём PONG с таймаутом
            match timeout(PONG_TIMEOUT, pong_rx.recv()).await {
                Ok(Some(pong_time)) => {
                    let latency = pong_time.duration_since(ping_time).as_millis() as u64;
                    let mut conn = conn_clone.lock().await;
                    conn.record_pong(latency);

                    println!(
                        "PONG получен: latency={}ms, avg={:.2}ms",
                        latency,
                        conn.average_latency().unwrap_or(0.0)
                    );

                    // Предупреждение при высокой задержке
                    if latency > 100 {
                        println!("⚠️  Высокая задержка! Возможны проблемы с исполнением ордеров");
                    }
                }
                Ok(None) => {
                    println!("Канал закрыт");
                    break;
                }
                Err(_) => {
                    let mut conn = conn_clone.lock().await;
                    conn.miss_pong();

                    match &conn.state {
                        ConnectionState::Degraded { missed_pongs } => {
                            println!(
                                "⚠️  PONG не получен! Пропущено: {}/{}",
                                missed_pongs, MAX_MISSED_PONGS
                            );
                        }
                        ConnectionState::Disconnected => {
                            println!("❌ Соединение потеряно! Инициируем переподключение...");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Мониторинг состояния соединения
    let conn_monitor = Arc::clone(&connection);
    let monitor_task = tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(1));

        loop {
            check_interval.tick().await;

            let conn = conn_monitor.lock().await;
            if !conn.is_healthy() {
                println!("Соединение нездорово: {:?}", conn.state);
            }
        }
    });

    tokio::select! {
        _ = read_task => println!("Read task завершён"),
        _ = heartbeat_task => println!("Heartbeat task завершён"),
        _ = monitor_task => println!("Monitor task завершён"),
    }

    Ok(())
}
```

## WebSocket Ping-Pong для криптобирж

Большинство криптобирж используют WebSocket с встроенным ping-pong:

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct PingMessage {
    op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PongMessage {
    op: String,
    #[serde(default)]
    id: Option<u64>,
}

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn binance_ws_client() -> Result<(), Box<dyn std::error::Error>> {
    // Подключаемся к WebSocket API Binance
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";

    let (ws_stream, _) = connect_async(url).await?;
    println!("Подключились к Binance WebSocket");

    let (mut write, mut read) = ws_stream.split();

    let mut last_message_time = Instant::now();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Отправляем ping каждые 30 секунд
            _ = ping_interval.tick() => {
                // Проверяем, давно ли мы получали данные
                let silence_duration = last_message_time.elapsed();

                if silence_duration > Duration::from_secs(60) {
                    println!("⚠️  Нет данных более 60 секунд!");
                }

                // WebSocket ping (на уровне протокола)
                write.send(Message::Ping(vec![])).await?;
                println!("Отправлен WebSocket PING");
            }

            // Читаем входящие сообщения
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_message_time = Instant::now();

                        // Парсим торговые данные
                        if let Ok(trade) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(price) = trade.get("p").and_then(|p| p.as_str()) {
                                println!("BTC/USDT: ${}", price);
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_message_time = Instant::now();
                        println!("Получен PONG от Binance");
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // Отвечаем на ping от сервера
                        write.send(Message::Pong(data)).await?;
                        println!("Ответили PONG на PING от сервера");
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Сервер закрыл соединение");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("Ошибка WebSocket: {}", e);
                        break;
                    }
                    None => {
                        println!("Поток закрыт");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

## Автоматическое переподключение

В трейдинге переподключение должно быть автоматическим и быстрым:

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout, Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
}

struct TradingClient {
    status: Arc<RwLock<ConnectionStatus>>,
    server_addr: String,
    max_reconnect_attempts: u32,
    base_reconnect_delay: Duration,
}

impl TradingClient {
    fn new(server_addr: &str) -> Self {
        TradingClient {
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            server_addr: server_addr.to_string(),
            max_reconnect_attempts: 10,
            base_reconnect_delay: Duration::from_secs(1),
        }
    }

    async fn connect_with_retry(&self) -> Result<TcpStream, String> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Reconnecting { attempt };
            }

            println!("Попытка подключения #{}", attempt);

            match timeout(
                Duration::from_secs(5),
                TcpStream::connect(&self.server_addr)
            ).await {
                Ok(Ok(stream)) => {
                    let mut status = self.status.write().await;
                    *status = ConnectionStatus::Connected;
                    println!("✅ Подключение успешно!");
                    return Ok(stream);
                }
                Ok(Err(e)) => {
                    eprintln!("Ошибка подключения: {}", e);
                }
                Err(_) => {
                    eprintln!("Таймаут подключения");
                }
            }

            if attempt >= self.max_reconnect_attempts {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Disconnected;
                return Err("Превышено максимальное количество попыток".to_string());
            }

            // Экспоненциальная задержка с jitter
            let delay = self.calculate_backoff(attempt);
            println!("Следующая попытка через {:?}", delay);
            sleep(delay).await;
        }
    }

    fn calculate_backoff(&self, attempt: u32) -> Duration {
        // Экспоненциальный backoff: 1s, 2s, 4s, 8s, ... до 60s
        let base = self.base_reconnect_delay.as_secs_f64();
        let exp_delay = base * 2.0_f64.powi((attempt - 1) as i32);
        let capped_delay = exp_delay.min(60.0);

        // Добавляем случайный jitter (±20%)
        let jitter = capped_delay * 0.2 * (rand::random::<f64>() - 0.5);
        Duration::from_secs_f64(capped_delay + jitter)
    }

    async fn run_with_heartbeat(self: Arc<Self>) -> Result<(), String> {
        loop {
            let stream = self.connect_with_retry().await?;
            let (mut read_half, mut write_half) = stream.into_split();

            let status = Arc::clone(&self.status);

            // Ping-pong loop
            let ping_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                let mut missed_pongs = 0;

                loop {
                    interval.tick().await;

                    if write_half.write_all(b"PING\n").await.is_err() {
                        return false; // Нужно переподключение
                    }

                    // Простая проверка состояния
                    let current_status = status.read().await;
                    if !matches!(*current_status, ConnectionStatus::Connected) {
                        return false;
                    }
                }
            });

            let read_handle = tokio::spawn(async move {
                let mut buffer = [0u8; 1024];

                loop {
                    match read_half.read(&mut buffer).await {
                        Ok(0) => return false,
                        Ok(_) => {
                            // Обрабатываем сообщения
                        }
                        Err(_) => return false,
                    }
                }
            });

            // Ждём завершения любой задачи
            tokio::select! {
                result = ping_handle => {
                    if let Ok(false) = result {
                        println!("Соединение потеряно, переподключаемся...");
                    }
                }
                result = read_handle => {
                    if let Ok(false) = result {
                        println!("Соединение закрыто сервером, переподключаемся...");
                    }
                }
            }

            // Небольшая пауза перед переподключением
            sleep(Duration::from_millis(500)).await;
        }
    }
}

// Простой random для примера (в реальном коде используйте crate rand)
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T: From<f64>>() -> T {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        T::from((nanos % 1000) as f64 / 1000.0)
    }
}
```

## Мониторинг качества соединения

Для алготрейдинга важно не только поддерживать соединение, но и мониторить его качество:

```rust
use std::collections::VecDeque;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct LatencyStats {
    samples: VecDeque<u64>,
    max_samples: usize,
}

impl LatencyStats {
    fn new(max_samples: usize) -> Self {
        LatencyStats {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    fn add_sample(&mut self, latency_ms: u64) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(latency_ms);
    }

    fn average(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: u64 = self.samples.iter().sum();
        Some(sum as f64 / self.samples.len() as f64)
    }

    fn percentile(&self, p: f64) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.samples.iter().copied().collect();
        sorted.sort();

        let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[index])
    }

    fn min(&self) -> Option<u64> {
        self.samples.iter().copied().min()
    }

    fn max(&self) -> Option<u64> {
        self.samples.iter().copied().max()
    }
}

#[derive(Debug)]
struct ConnectionQualityMonitor {
    latency_stats: LatencyStats,
    missed_pongs: u32,
    total_pings: u64,
    total_pongs: u64,
    last_pong_time: Option<Instant>,
}

impl ConnectionQualityMonitor {
    fn new() -> Self {
        ConnectionQualityMonitor {
            latency_stats: LatencyStats::new(100),
            missed_pongs: 0,
            total_pings: 0,
            total_pongs: 0,
            last_pong_time: None,
        }
    }

    fn record_ping(&mut self) {
        self.total_pings += 1;
    }

    fn record_pong(&mut self, latency_ms: u64) {
        self.total_pongs += 1;
        self.missed_pongs = 0;
        self.last_pong_time = Some(Instant::now());
        self.latency_stats.add_sample(latency_ms);
    }

    fn record_missed_pong(&mut self) {
        self.missed_pongs += 1;
    }

    fn success_rate(&self) -> f64 {
        if self.total_pings == 0 {
            return 100.0;
        }
        (self.total_pongs as f64 / self.total_pings as f64) * 100.0
    }

    fn report(&self) -> String {
        format!(
            "📊 Статистика соединения:\n\
             • Успешность: {:.1}%\n\
             • Пропущено pong: {}\n\
             • Latency avg: {:.2}ms\n\
             • Latency p50: {}ms\n\
             • Latency p99: {}ms\n\
             • Latency min/max: {}/{}ms",
            self.success_rate(),
            self.missed_pongs,
            self.latency_stats.average().unwrap_or(0.0),
            self.latency_stats.percentile(50.0).unwrap_or(0),
            self.latency_stats.percentile(99.0).unwrap_or(0),
            self.latency_stats.min().unwrap_or(0),
            self.latency_stats.max().unwrap_or(0),
        )
    }

    fn should_reconnect(&self) -> bool {
        // Переподключаемся, если:
        // 1. Пропущено 3+ pong подряд
        // 2. Успешность ниже 90%
        // 3. Latency p99 > 1000ms
        self.missed_pongs >= 3
            || self.success_rate() < 90.0
            || self.latency_stats.percentile(99.0).unwrap_or(0) > 1000
    }

    fn is_suitable_for_trading(&self) -> bool {
        // Для активной торговли нужно:
        // 1. Latency p99 < 100ms
        // 2. Успешность > 99%
        // 3. Нет пропущенных pong
        self.latency_stats.percentile(99.0).unwrap_or(u64::MAX) < 100
            && self.success_rate() > 99.0
            && self.missed_pongs == 0
    }
}

fn main() {
    let mut monitor = ConnectionQualityMonitor::new();

    // Симуляция работы
    for latency in [15, 18, 22, 19, 45, 21, 17, 120, 23, 19] {
        monitor.record_ping();
        monitor.record_pong(latency);
    }

    println!("{}", monitor.report());
    println!("\nПодходит для торговли: {}", monitor.is_suitable_for_trading());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Ping-Pong | Механизм heartbeat для проверки живости соединения |
| Таймаут pong | Максимальное время ожидания ответа на ping |
| Экспоненциальный backoff | Увеличение задержки между попытками переподключения |
| Jitter | Случайное смещение для предотвращения thundering herd |
| Latency мониторинг | Отслеживание задержки для оценки качества соединения |
| Автопереподключение | Автоматическое восстановление потерянного соединения |

## Практические задания

1. **Базовый heartbeat**: Реализуй простой TCP-сервер и клиент с ping-pong механизмом. Сервер должен отключать клиентов, не отправивших ping более 30 секунд.

2. **Мониторинг latency**: Добавь к клиенту сбор статистики задержки и вывод предупреждения, если p95 latency превышает 100ms.

3. **Умное переподключение**: Реализуй клиента с экспоненциальным backoff и jitter. Добавь максимальное количество попыток и уведомление при невозможности подключиться.

4. **Мультиплексирование**: Создай клиента, который поддерживает соединения с несколькими биржами одновременно и отслеживает качество каждого соединения отдельно.

## Домашнее задание

1. **Симулятор биржевого подключения**: Создай программу, которая:
   - Подключается к "серверу котировок" (можешь использовать свой TCP-сервер)
   - Отправляет ping каждые 5 секунд
   - Отслеживает latency и выводит статистику каждую минуту
   - Автоматически переподключается при потере соединения

2. **Детектор проблем сети**: Расширь программу из задания 1:
   - Добавь обнаружение "тихих" обрывов (когда данные просто перестают приходить)
   - Реализуй алерты при деградации качества соединения
   - Добавь лог всех проблем для последующего анализа

3. **Балансировщик подключений**: Создай клиента, который:
   - Поддерживает подключения к нескольким серверам
   - Выбирает сервер с наименьшей latency для отправки ордеров
   - Переключается на резервный сервер при проблемах с основным

4. **Анализатор стабильности**: Напиши программу, которая:
   - Собирает статистику ping-pong за длительный период
   - Определяет паттерны деградации (например, проблемы в определённое время суток)
   - Выводит рекомендации по оптимизации подключения

## Навигация

[← Предыдущий день](../205-websocket-streaming-data/ru.md) | [Следующий день →](../207-graceful-shutdown/ru.md)
