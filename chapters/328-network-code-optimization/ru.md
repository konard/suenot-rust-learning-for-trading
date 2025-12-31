# День 328: Оптимизация сетевого кода

## Аналогия из трейдинга

Представь, что ты арбитражный трейдер. Ты видишь разницу в цене BTC между биржами — на Binance $50,000, на Kraken $50,100. Прибыль $100 за монету! Но пока твой ордер летит через интернет, цена уже изменилась. **Сетевая задержка** — это налог на каждую твою сделку.

В высокочастотной торговле (HFT) каждая миллисекунда на счету:
- **1 мс** задержки = потерянные сделки
- **10 мс** = конкуренты забрали лучшие цены
- **100 мс** = ты торгуешь вчерашними ценами

Оптимизация сетевого кода — это как переезд офиса ближе к бирже: ты получаешь информацию быстрее и реагируешь раньше конкурентов.

## Основные метрики сетевой производительности

```rust
use std::time::{Duration, Instant};

/// Метрики сетевой производительности для торговой системы
struct NetworkMetrics {
    /// Время установки соединения
    connection_time: Duration,
    /// Время до первого байта (TTFB)
    time_to_first_byte: Duration,
    /// Общая задержка запроса
    total_latency: Duration,
    /// Пропускная способность (байт/сек)
    throughput: f64,
}

impl NetworkMetrics {
    fn display(&self) {
        println!("=== Сетевые метрики ===");
        println!("Время соединения: {:?}", self.connection_time);
        println!("Время до первого байта: {:?}", self.time_to_first_byte);
        println!("Общая задержка: {:?}", self.total_latency);
        println!("Пропускная способность: {:.2} KB/s", self.throughput / 1024.0);
    }
}

fn main() {
    // Симуляция метрик для двух конфигураций
    let unoptimized = NetworkMetrics {
        connection_time: Duration::from_millis(50),
        time_to_first_byte: Duration::from_millis(120),
        total_latency: Duration::from_millis(200),
        throughput: 512_000.0,
    };

    let optimized = NetworkMetrics {
        connection_time: Duration::from_millis(5),
        time_to_first_byte: Duration::from_millis(15),
        total_latency: Duration::from_millis(25),
        throughput: 2_048_000.0,
    };

    println!("До оптимизации:");
    unoptimized.display();

    println!("\nПосле оптимизации:");
    optimized.display();

    let latency_improvement =
        unoptimized.total_latency.as_micros() as f64 /
        optimized.total_latency.as_micros() as f64;
    println!("\nУлучшение задержки: {:.1}x", latency_improvement);
}
```

## Повторное использование соединений (Connection Pooling)

Создание нового TCP-соединения — дорогая операция. Для торговли критично держать соединения открытыми:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Пул соединений для торговых API
struct ConnectionPool {
    /// Активные соединения к биржам
    connections: HashMap<String, Connection>,
    /// Максимальное время жизни соединения
    max_idle_time: Duration,
    /// Максимум соединений на биржу
    max_connections_per_host: usize,
}

struct Connection {
    host: String,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
}

impl Connection {
    fn new(host: &str) -> Self {
        let now = Instant::now();
        println!("[Соединение] Создано новое соединение к {}", host);
        Connection {
            host: host.to_string(),
            created_at: now,
            last_used: now,
            request_count: 0,
        }
    }

    fn is_valid(&self, max_idle: Duration) -> bool {
        self.last_used.elapsed() < max_idle
    }

    fn use_connection(&mut self) {
        self.last_used = Instant::now();
        self.request_count += 1;
    }
}

impl ConnectionPool {
    fn new(max_idle_time: Duration, max_per_host: usize) -> Self {
        ConnectionPool {
            connections: HashMap::new(),
            max_idle_time,
            max_connections_per_host: max_per_host,
        }
    }

    /// Получить соединение (из пула или создать новое)
    fn get_connection(&mut self, host: &str) -> &mut Connection {
        // Проверяем, есть ли валидное соединение
        if let Some(conn) = self.connections.get(host) {
            if conn.is_valid(self.max_idle_time) {
                println!("[Пул] Переиспользуем соединение к {}", host);
                let conn = self.connections.get_mut(host).unwrap();
                conn.use_connection();
                return conn;
            }
        }

        // Создаём новое соединение
        let conn = Connection::new(host);
        self.connections.insert(host.to_string(), conn);
        self.connections.get_mut(host).unwrap()
    }

    /// Очистка устаревших соединений
    fn cleanup(&mut self) {
        let max_idle = self.max_idle_time;
        self.connections.retain(|host, conn| {
            let valid = conn.is_valid(max_idle);
            if !valid {
                println!("[Пул] Закрываем устаревшее соединение к {}", host);
            }
            valid
        });
    }

    fn stats(&self) {
        println!("\n=== Статистика пула ===");
        println!("Активных соединений: {}", self.connections.len());
        for (host, conn) in &self.connections {
            println!(
                "  {} - запросов: {}, возраст: {:?}",
                host,
                conn.request_count,
                conn.created_at.elapsed()
            );
        }
    }
}

fn main() {
    let mut pool = ConnectionPool::new(
        Duration::from_secs(30),
        5,
    );

    // Симуляция торговых запросов
    let exchanges = ["api.binance.com", "api.kraken.com", "api.coinbase.com"];

    for i in 0..10 {
        let exchange = exchanges[i % exchanges.len()];
        println!("\nЗапрос #{} к {}", i + 1, exchange);
        pool.get_connection(exchange);
    }

    pool.stats();
}
```

## Буферизация и пакетная обработка

Отправка множества мелких запросов неэффективна. Группируем данные:

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Буфер для пакетной отправки ордеров
struct OrderBuffer {
    orders: VecDeque<Order>,
    max_batch_size: usize,
    max_delay: Duration,
    last_flush: Instant,
}

#[derive(Clone, Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Clone, Debug)]
enum OrderSide {
    Buy,
    Sell,
}

impl OrderBuffer {
    fn new(max_batch_size: usize, max_delay: Duration) -> Self {
        OrderBuffer {
            orders: VecDeque::new(),
            max_batch_size,
            max_delay,
            last_flush: Instant::now(),
        }
    }

    /// Добавить ордер в буфер
    fn add(&mut self, order: Order) -> Option<Vec<Order>> {
        self.orders.push_back(order);

        // Отправляем если буфер полон или прошло много времени
        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }

    fn should_flush(&self) -> bool {
        self.orders.len() >= self.max_batch_size ||
        self.last_flush.elapsed() >= self.max_delay
    }

    /// Извлечь все ордера для отправки
    fn flush(&mut self) -> Vec<Order> {
        self.last_flush = Instant::now();
        self.orders.drain(..).collect()
    }

    /// Принудительная отправка всего буфера
    fn force_flush(&mut self) -> Vec<Order> {
        if self.orders.is_empty() {
            Vec::new()
        } else {
            self.flush()
        }
    }
}

/// Симуляция отправки пакета ордеров
fn send_batch(orders: Vec<Order>) {
    println!("\n=== Отправка пакета из {} ордеров ===", orders.len());
    for order in &orders {
        println!(
            "  {:?} {} {} @ {:.2}",
            order.side, order.quantity, order.symbol, order.price
        );
    }
    // В реальности: один HTTP-запрос вместо N
    println!("  Отправлено одним сетевым вызовом!");
}

fn main() {
    let mut buffer = OrderBuffer::new(
        5,                             // Отправляем когда накопится 5 ордеров
        Duration::from_millis(100),    // Или через 100мс
    );

    let orders = vec![
        Order { symbol: "BTCUSDT".into(), side: OrderSide::Buy, price: 50000.0, quantity: 0.1 },
        Order { symbol: "ETHUSDT".into(), side: OrderSide::Buy, price: 3000.0, quantity: 1.0 },
        Order { symbol: "BTCUSDT".into(), side: OrderSide::Sell, price: 50100.0, quantity: 0.1 },
        Order { symbol: "SOLUSDT".into(), side: OrderSide::Buy, price: 100.0, quantity: 10.0 },
        Order { symbol: "ETHUSDT".into(), side: OrderSide::Sell, price: 3050.0, quantity: 0.5 },
        Order { symbol: "BTCUSDT".into(), side: OrderSide::Buy, price: 49900.0, quantity: 0.2 },
    ];

    for order in orders {
        if let Some(batch) = buffer.add(order) {
            send_batch(batch);
        }
    }

    // Отправляем остаток
    let remaining = buffer.force_flush();
    if !remaining.is_empty() {
        send_batch(remaining);
    }
}
```

## Неблокирующий I/O и мультиплексирование

Синхронные вызовы блокируют поток. Для торговли нужен неблокирующий подход:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Симуляция неблокирующего сетевого клиента
struct AsyncPriceClient {
    /// Ожидающие ответа запросы
    pending_requests: HashMap<u64, PendingRequest>,
    /// Счётчик запросов
    request_counter: u64,
    /// Полученные цены
    prices: HashMap<String, f64>,
}

struct PendingRequest {
    symbol: String,
    sent_at: Instant,
}

impl AsyncPriceClient {
    fn new() -> Self {
        AsyncPriceClient {
            pending_requests: HashMap::new(),
            request_counter: 0,
            prices: HashMap::new(),
        }
    }

    /// Отправить запрос (неблокирующий)
    fn request_price(&mut self, symbol: &str) -> u64 {
        self.request_counter += 1;
        let request_id = self.request_counter;

        self.pending_requests.insert(request_id, PendingRequest {
            symbol: symbol.to_string(),
            sent_at: Instant::now(),
        });

        println!("[Запрос #{}] Отправлен запрос цены {}", request_id, symbol);
        request_id
    }

    /// Обработать ответ (вызывается когда данные получены)
    fn handle_response(&mut self, request_id: u64, price: f64) {
        if let Some(request) = self.pending_requests.remove(&request_id) {
            let latency = request.sent_at.elapsed();
            self.prices.insert(request.symbol.clone(), price);

            println!(
                "[Ответ #{}] {} = ${:.2} (задержка: {:?})",
                request_id, request.symbol, price, latency
            );
        }
    }

    /// Проверить таймауты
    fn check_timeouts(&mut self, timeout: Duration) -> Vec<u64> {
        let now = Instant::now();
        let timed_out: Vec<u64> = self.pending_requests
            .iter()
            .filter(|(_, req)| now.duration_since(req.sent_at) > timeout)
            .map(|(&id, _)| id)
            .collect();

        for id in &timed_out {
            if let Some(req) = self.pending_requests.remove(id) {
                println!("[Таймаут #{}] Запрос {} превысил время ожидания", id, req.symbol);
            }
        }

        timed_out
    }

    /// Получить все цены
    fn get_prices(&self) -> &HashMap<String, f64> {
        &self.prices
    }
}

fn main() {
    let mut client = AsyncPriceClient::new();

    // Отправляем несколько запросов одновременно (не ждём ответа)
    println!("=== Отправка запросов (неблокирующая) ===");
    let btc_req = client.request_price("BTCUSDT");
    let eth_req = client.request_price("ETHUSDT");
    let sol_req = client.request_price("SOLUSDT");

    println!("\n=== Можем делать другую работу пока ждём ===");
    println!("Расчёт индикаторов...");
    println!("Проверка рисков...");

    println!("\n=== Получены ответы ===");
    // Симуляция получения ответов (в разном порядке!)
    client.handle_response(eth_req, 3000.0);
    client.handle_response(btc_req, 50000.0);
    client.handle_response(sol_req, 100.0);

    println!("\n=== Итоговые цены ===");
    for (symbol, price) in client.get_prices() {
        println!("  {}: ${:.2}", symbol, price);
    }
}
```

## Сжатие данных

При передаче больших объёмов данных (история сделок, стаканы) сжатие критично:

```rust
use std::time::Instant;

/// Симуляция эффекта сжатия данных
struct CompressionStats {
    original_size: usize,
    compressed_size: usize,
    compression_time: std::time::Duration,
}

impl CompressionStats {
    fn compression_ratio(&self) -> f64 {
        self.original_size as f64 / self.compressed_size as f64
    }

    fn space_saved_percent(&self) -> f64 {
        (1.0 - (self.compressed_size as f64 / self.original_size as f64)) * 100.0
    }
}

/// Данные стакана (order book)
struct OrderBookData {
    symbol: String,
    bids: Vec<(f64, f64)>,  // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBookData {
    /// Генерация тестовых данных стакана
    fn generate(symbol: &str, depth: usize) -> Self {
        let base_price = 50000.0;
        let mut bids = Vec::with_capacity(depth);
        let mut asks = Vec::with_capacity(depth);

        for i in 0..depth {
            let offset = i as f64 * 0.1;
            bids.push((base_price - offset, 0.1 + (i as f64 * 0.01)));
            asks.push((base_price + offset, 0.1 + (i as f64 * 0.01)));
        }

        OrderBookData { symbol: symbol.to_string(), bids, asks }
    }

    /// Размер в байтах (приблизительно)
    fn size_bytes(&self) -> usize {
        self.symbol.len() + (self.bids.len() + self.asks.len()) * 16
    }

    /// Сериализация в JSON (для сравнения)
    fn to_json(&self) -> String {
        let mut json = format!(r#"{{"symbol":"{}","bids":["#, self.symbol);
        for (i, (price, qty)) in self.bids.iter().enumerate() {
            if i > 0 { json.push(','); }
            json.push_str(&format!("[{:.2},{:.4}]", price, qty));
        }
        json.push_str(r#"],"asks":["#);
        for (i, (price, qty)) in self.asks.iter().enumerate() {
            if i > 0 { json.push(','); }
            json.push_str(&format!("[{:.2},{:.4}]", price, qty));
        }
        json.push_str("]}");
        json
    }

    /// Компактная бинарная сериализация
    fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Символ (длина + байты)
        data.push(self.symbol.len() as u8);
        data.extend(self.symbol.as_bytes());

        // Количество уровней
        data.extend(&(self.bids.len() as u16).to_le_bytes());

        // Bids (используем delta-encoding для цен)
        let mut prev_price = 0i64;
        for (price, qty) in &self.bids {
            let price_int = (*price * 100.0) as i64;
            let delta = price_int - prev_price;
            prev_price = price_int;

            // Кодируем delta как varint (упрощённо)
            data.extend(&(delta as i32).to_le_bytes());
            data.extend(&((*qty * 10000.0) as u32).to_le_bytes());
        }

        // Asks аналогично
        prev_price = 0;
        for (price, qty) in &self.asks {
            let price_int = (*price * 100.0) as i64;
            let delta = price_int - prev_price;
            prev_price = price_int;
            data.extend(&(delta as i32).to_le_bytes());
            data.extend(&((*qty * 10000.0) as u32).to_le_bytes());
        }

        data
    }
}

fn main() {
    println!("=== Сравнение форматов сериализации стакана ===\n");

    for depth in [10, 100, 1000] {
        let order_book = OrderBookData::generate("BTCUSDT", depth);

        let json = order_book.to_json();
        let binary = order_book.to_binary();

        println!("Глубина стакана: {} уровней", depth);
        println!("  JSON размер: {} байт", json.len());
        println!("  Binary размер: {} байт", binary.len());
        println!(
            "  Экономия: {:.1}%",
            (1.0 - binary.len() as f64 / json.len() as f64) * 100.0
        );
        println!();
    }

    // Демонстрация влияния на пропускную способность
    println!("=== Влияние на пропускную способность ===");
    let bandwidth_mbps = 100.0; // 100 Мбит/с
    let bytes_per_second = bandwidth_mbps * 1_000_000.0 / 8.0;

    let order_book = OrderBookData::generate("BTCUSDT", 100);
    let json_size = order_book.to_json().len();
    let binary_size = order_book.to_binary().len();

    let json_updates_per_sec = bytes_per_second / json_size as f64;
    let binary_updates_per_sec = bytes_per_second / binary_size as f64;

    println!("При пропускной способности {} Мбит/с:", bandwidth_mbps);
    println!("  JSON: {:.0} обновлений/сек", json_updates_per_sec);
    println!("  Binary: {:.0} обновлений/сек", binary_updates_per_sec);
    println!("  Выигрыш: {:.1}x", binary_updates_per_sec / json_updates_per_sec);
}
```

## Оптимизация TCP для торговли

Настройки TCP сильно влияют на производительность:

```rust
use std::collections::HashMap;

/// Конфигурация TCP для торговой системы
#[derive(Debug, Clone)]
struct TcpConfig {
    /// Отключить алгоритм Nagle (важно для низкой задержки!)
    tcp_nodelay: bool,
    /// Размер буфера отправки
    send_buffer_size: usize,
    /// Размер буфера приёма
    recv_buffer_size: usize,
    /// Keep-alive интервал
    keepalive_interval_secs: Option<u64>,
    /// Таймаут соединения
    connect_timeout_ms: u64,
}

impl Default for TcpConfig {
    fn default() -> Self {
        TcpConfig {
            tcp_nodelay: true,  // ВАЖНО для торговли!
            send_buffer_size: 64 * 1024,
            recv_buffer_size: 64 * 1024,
            keepalive_interval_secs: Some(30),
            connect_timeout_ms: 5000,
        }
    }
}

impl TcpConfig {
    /// Конфигурация для высокочастотной торговли
    fn hft() -> Self {
        TcpConfig {
            tcp_nodelay: true,
            send_buffer_size: 256 * 1024,   // Больший буфер
            recv_buffer_size: 256 * 1024,
            keepalive_interval_secs: Some(10),
            connect_timeout_ms: 1000,        // Быстрый таймаут
        }
    }

    /// Конфигурация для загрузки исторических данных
    fn bulk_data() -> Self {
        TcpConfig {
            tcp_nodelay: false,              // Nagle OK для bulk
            send_buffer_size: 1024 * 1024,   // 1MB буфер
            recv_buffer_size: 1024 * 1024,
            keepalive_interval_secs: Some(60),
            connect_timeout_ms: 30000,
        }
    }

    fn display(&self) {
        println!("TCP конфигурация:");
        println!("  TCP_NODELAY: {}", self.tcp_nodelay);
        println!("  Буфер отправки: {} KB", self.send_buffer_size / 1024);
        println!("  Буфер приёма: {} KB", self.recv_buffer_size / 1024);
        if let Some(interval) = self.keepalive_interval_secs {
            println!("  Keep-alive: {} сек", interval);
        }
        println!("  Таймаут соединения: {} мс", self.connect_timeout_ms);
    }
}

fn explain_tcp_nodelay() {
    println!("=== Алгоритм Nagle и TCP_NODELAY ===\n");

    println!("Алгоритм Nagle:");
    println!("  - Буферизует маленькие пакеты");
    println!("  - Отправляет когда накопится достаточно данных");
    println!("  - Снижает количество пакетов в сети");
    println!("  - Добавляет задержку до 200мс!\n");

    println!("TCP_NODELAY = true:");
    println!("  - Отправляет данные СРАЗУ");
    println!("  - Нет буферизации");
    println!("  - Критично для торговли!");
    println!("  - Каждая миллисекунда на счету\n");

    // Симуляция влияния
    let message_size = 100; // байт
    let messages_per_second = 100;

    println!("Пример: {} сообщений/сек по {} байт", messages_per_second, message_size);
    println!("  С Nagle: задержка до 200мс = пропущены сигналы");
    println!("  С NODELAY: мгновенная отправка = быстрая реакция");
}

fn main() {
    println!("=== Конфигурации TCP для разных сценариев ===\n");

    println!("1. Стандартная конфигурация:");
    TcpConfig::default().display();

    println!("\n2. Высокочастотная торговля (HFT):");
    TcpConfig::hft().display();

    println!("\n3. Загрузка исторических данных:");
    TcpConfig::bulk_data().display();

    println!();
    explain_tcp_nodelay();
}
```

## Кэширование DNS

DNS-запросы добавляют задержку. Кэшируем результаты:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Кэш DNS для торговой системы
struct DnsCache {
    cache: HashMap<String, DnsEntry>,
    default_ttl: Duration,
}

struct DnsEntry {
    ip_addresses: Vec<String>,
    resolved_at: Instant,
    ttl: Duration,
}

impl DnsEntry {
    fn is_valid(&self) -> bool {
        self.resolved_at.elapsed() < self.ttl
    }
}

impl DnsCache {
    fn new(default_ttl: Duration) -> Self {
        DnsCache {
            cache: HashMap::new(),
            default_ttl,
        }
    }

    /// Получить IP для хоста
    fn resolve(&mut self, hostname: &str) -> ResolveResult {
        // Проверяем кэш
        if let Some(entry) = self.cache.get(hostname) {
            if entry.is_valid() {
                return ResolveResult::Cached(entry.ip_addresses.clone());
            }
        }

        // Симуляция DNS-запроса
        let ips = self.do_dns_lookup(hostname);

        // Сохраняем в кэш
        self.cache.insert(hostname.to_string(), DnsEntry {
            ip_addresses: ips.clone(),
            resolved_at: Instant::now(),
            ttl: self.default_ttl,
        });

        ResolveResult::Fresh(ips)
    }

    /// Симуляция DNS-запроса
    fn do_dns_lookup(&self, hostname: &str) -> Vec<String> {
        // В реальности: системный вызов gethostbyname
        println!("[DNS] Выполняем запрос для {}", hostname);

        match hostname {
            "api.binance.com" => vec!["52.84.71.1".into(), "52.84.71.2".into()],
            "api.kraken.com" => vec!["104.20.48.1".into()],
            "api.coinbase.com" => vec!["104.18.6.1".into(), "104.18.7.1".into()],
            _ => vec!["127.0.0.1".into()],
        }
    }

    /// Предварительное разрешение критичных хостов
    fn warmup(&mut self, hostnames: &[&str]) {
        println!("=== Прогрев DNS кэша ===");
        for hostname in hostnames {
            self.resolve(hostname);
        }
        println!("Закэшировано {} хостов\n", hostnames.len());
    }

    fn stats(&self) {
        println!("\n=== Статистика DNS кэша ===");
        println!("Записей в кэше: {}", self.cache.len());
        for (hostname, entry) in &self.cache {
            let age = entry.resolved_at.elapsed();
            let remaining = entry.ttl.saturating_sub(age);
            println!(
                "  {}: {} IP, осталось {:?}",
                hostname,
                entry.ip_addresses.len(),
                remaining
            );
        }
    }
}

#[derive(Debug)]
enum ResolveResult {
    Cached(Vec<String>),
    Fresh(Vec<String>),
}

fn main() {
    let mut dns = DnsCache::new(Duration::from_secs(300)); // TTL 5 минут

    // Прогреваем кэш при старте
    dns.warmup(&["api.binance.com", "api.kraken.com", "api.coinbase.com"]);

    // Последующие запросы используют кэш
    println!("=== Запросы после прогрева ===");

    for _ in 0..3 {
        match dns.resolve("api.binance.com") {
            ResolveResult::Cached(ips) => {
                println!("[Кэш] api.binance.com -> {:?}", ips);
            }
            ResolveResult::Fresh(ips) => {
                println!("[DNS] api.binance.com -> {:?}", ips);
            }
        }
    }

    dns.stats();
}
```

## Измерение и мониторинг сетевой производительности

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Мониторинг сетевой задержки
struct LatencyMonitor {
    measurements: VecDeque<Duration>,
    max_samples: usize,
    thresholds: LatencyThresholds,
}

struct LatencyThresholds {
    warning_ms: u64,
    critical_ms: u64,
}

impl LatencyMonitor {
    fn new(max_samples: usize, warning_ms: u64, critical_ms: u64) -> Self {
        LatencyMonitor {
            measurements: VecDeque::with_capacity(max_samples),
            max_samples,
            thresholds: LatencyThresholds { warning_ms, critical_ms },
        }
    }

    /// Записать измерение
    fn record(&mut self, latency: Duration) {
        if self.measurements.len() >= self.max_samples {
            self.measurements.pop_front();
        }
        self.measurements.push_back(latency);

        // Проверяем пороги
        let ms = latency.as_millis() as u64;
        if ms >= self.thresholds.critical_ms {
            println!("[КРИТИЧНО] Задержка {} мс!", ms);
        } else if ms >= self.thresholds.warning_ms {
            println!("[Предупреждение] Высокая задержка: {} мс", ms);
        }
    }

    /// Средняя задержка
    fn average(&self) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        let total: Duration = self.measurements.iter().sum();
        Some(total / self.measurements.len() as u32)
    }

    /// Перцентиль задержки
    fn percentile(&self, p: f64) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = self.measurements.iter().cloned().collect();
        sorted.sort();

        let index = ((sorted.len() as f64 * p / 100.0) as usize).min(sorted.len() - 1);
        Some(sorted[index])
    }

    /// Отчёт по задержкам
    fn report(&self) {
        println!("\n=== Отчёт по сетевой задержке ===");
        println!("Измерений: {}", self.measurements.len());

        if let Some(avg) = self.average() {
            println!("Средняя: {:?}", avg);
        }

        if let Some(p50) = self.percentile(50.0) {
            println!("P50 (медиана): {:?}", p50);
        }

        if let Some(p95) = self.percentile(95.0) {
            println!("P95: {:?}", p95);
        }

        if let Some(p99) = self.percentile(99.0) {
            println!("P99: {:?}", p99);
        }

        if let (Some(min), Some(max)) = (
            self.measurements.iter().min(),
            self.measurements.iter().max()
        ) {
            println!("Min: {:?}, Max: {:?}", min, max);
        }
    }
}

/// Симуляция измерения задержки запроса
fn measure_request_latency() -> Duration {
    // Симуляция: случайная задержка от 5 до 50 мс
    // с редкими выбросами до 200 мс
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let now = Instant::now();
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let random = hasher.finish();

    let base_ms = 5 + (random % 45);
    let spike = if random % 20 == 0 { 150 } else { 0 };

    Duration::from_millis(base_ms + spike)
}

fn main() {
    let mut monitor = LatencyMonitor::new(
        1000,   // Хранить 1000 измерений
        50,     // Предупреждение > 50 мс
        100,    // Критично > 100 мс
    );

    println!("=== Симуляция мониторинга торговых запросов ===\n");

    // Симуляция 100 запросов
    for i in 0..100 {
        let latency = measure_request_latency();

        if i % 20 == 0 {
            println!("Запрос #{}: {:?}", i, latency);
        }

        monitor.record(latency);
    }

    monitor.report();
}
```

## Что мы узнали

| Техника | Описание | Выигрыш |
|---------|----------|---------|
| **Connection pooling** | Повторное использование соединений | Избегаем 100+ мс на handshake |
| **TCP_NODELAY** | Отключение алгоритма Nagle | Убираем 200 мс буферизации |
| **Пакетная обработка** | Группировка запросов | Меньше сетевых вызовов |
| **Сжатие данных** | Бинарный формат вместо JSON | 50-80% экономии трафика |
| **Кэширование DNS** | Сохранение результатов разрешения | Убираем 10-50 мс на запрос |
| **Мониторинг** | Отслеживание задержек | Раннее обнаружение проблем |

## Практические упражнения

1. **Пул соединений с приоритетами**: Создайте пул, который:
   - Выделяет больше соединений критичным биржам
   - Автоматически переподключается при ошибках
   - Балансирует нагрузку между соединениями
   - Собирает метрики по каждому соединению

2. **Интеллектуальный батчер**: Реализуйте буфер ордеров, который:
   - Группирует ордера по бирже
   - Учитывает приоритет (рыночные ордера важнее лимитных)
   - Отправляет критичные ордера немедленно
   - Оптимизирует размер пакета динамически

3. **Адаптивное сжатие**: Создайте систему, которая:
   - Выбирает алгоритм сжатия по типу данных
   - Отключает сжатие для маленьких сообщений
   - Измеряет выигрыш от сжатия в реальном времени
   - Адаптируется к пропускной способности канала

## Домашнее задание

1. **Оптимизированный WebSocket-клиент**: Напишите клиент для торговых данных:
   - Поддержка нескольких бирж одновременно
   - Автоматическое переподключение
   - Пинг/понг для проверки соединения
   - Метрики по каждому соединению
   - Буферизация сообщений при переподключении

2. **Система раннего оповещения**: Создайте мониторинг, который:
   - Отслеживает задержку к каждой бирже
   - Сравнивает с историческими данными
   - Предупреждает об аномалиях
   - Визуализирует тренды задержек
   - Интегрируется с торговой стратегией

3. **Бенчмарк сетевого стека**: Разработайте инструмент, который:
   - Измеряет реальную задержку до биржи
   - Тестирует разные TCP-настройки
   - Сравнивает HTTP/1.1 vs HTTP/2
   - Оценивает влияние VPN/прокси
   - Генерирует отчёт с рекомендациями

4. **Протокол с минимальной задержкой**: Спроектируйте протокол:
   - Бинарный формат сообщений
   - Delta-кодирование для обновлений цен
   - Встроенная проверка целостности
   - Поддержка приоритетов сообщений
   - Сравните с JSON по задержке и трафику

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../329-*/ru.md)
