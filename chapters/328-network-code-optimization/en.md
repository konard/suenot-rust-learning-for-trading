# Day 328: Network Code Optimization

## Trading Analogy

Imagine you're an arbitrage trader. You spot a price difference for BTC between exchanges — $50,000 on Binance, $50,100 on Kraken. That's $100 profit per coin! But while your order travels across the internet, the price has already changed. **Network latency** is a tax on every trade you make.

In high-frequency trading (HFT), every millisecond counts:
- **1 ms** delay = missed trades
- **10 ms** = competitors grabbed the best prices
- **100 ms** = you're trading on yesterday's prices

Network code optimization is like moving your office closer to the exchange: you receive information faster and react before your competitors.

## Key Network Performance Metrics

```rust
use std::time::{Duration, Instant};

/// Network performance metrics for a trading system
struct NetworkMetrics {
    /// Time to establish connection
    connection_time: Duration,
    /// Time to first byte (TTFB)
    time_to_first_byte: Duration,
    /// Total request latency
    total_latency: Duration,
    /// Throughput (bytes/sec)
    throughput: f64,
}

impl NetworkMetrics {
    fn display(&self) {
        println!("=== Network Metrics ===");
        println!("Connection time: {:?}", self.connection_time);
        println!("Time to first byte: {:?}", self.time_to_first_byte);
        println!("Total latency: {:?}", self.total_latency);
        println!("Throughput: {:.2} KB/s", self.throughput / 1024.0);
    }
}

fn main() {
    // Simulate metrics for two configurations
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

    println!("Before optimization:");
    unoptimized.display();

    println!("\nAfter optimization:");
    optimized.display();

    let latency_improvement =
        unoptimized.total_latency.as_micros() as f64 /
        optimized.total_latency.as_micros() as f64;
    println!("\nLatency improvement: {:.1}x", latency_improvement);
}
```

## Connection Pooling

Creating a new TCP connection is an expensive operation. For trading, it's critical to keep connections open:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Connection pool for trading APIs
struct ConnectionPool {
    /// Active connections to exchanges
    connections: HashMap<String, Connection>,
    /// Maximum connection idle time
    max_idle_time: Duration,
    /// Maximum connections per host
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
        println!("[Connection] Created new connection to {}", host);
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

    /// Get a connection (from pool or create new)
    fn get_connection(&mut self, host: &str) -> &mut Connection {
        // Check if we have a valid connection
        if let Some(conn) = self.connections.get(host) {
            if conn.is_valid(self.max_idle_time) {
                println!("[Pool] Reusing connection to {}", host);
                let conn = self.connections.get_mut(host).unwrap();
                conn.use_connection();
                return conn;
            }
        }

        // Create a new connection
        let conn = Connection::new(host);
        self.connections.insert(host.to_string(), conn);
        self.connections.get_mut(host).unwrap()
    }

    /// Cleanup stale connections
    fn cleanup(&mut self) {
        let max_idle = self.max_idle_time;
        self.connections.retain(|host, conn| {
            let valid = conn.is_valid(max_idle);
            if !valid {
                println!("[Pool] Closing stale connection to {}", host);
            }
            valid
        });
    }

    fn stats(&self) {
        println!("\n=== Pool Statistics ===");
        println!("Active connections: {}", self.connections.len());
        for (host, conn) in &self.connections {
            println!(
                "  {} - requests: {}, age: {:?}",
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

    // Simulate trading requests
    let exchanges = ["api.binance.com", "api.kraken.com", "api.coinbase.com"];

    for i in 0..10 {
        let exchange = exchanges[i % exchanges.len()];
        println!("\nRequest #{} to {}", i + 1, exchange);
        pool.get_connection(exchange);
    }

    pool.stats();
}
```

## Buffering and Batch Processing

Sending many small requests is inefficient. Group data together:

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Buffer for batch order sending
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

    /// Add an order to the buffer
    fn add(&mut self, order: Order) -> Option<Vec<Order>> {
        self.orders.push_back(order);

        // Send if buffer is full or too much time has passed
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

    /// Extract all orders for sending
    fn flush(&mut self) -> Vec<Order> {
        self.last_flush = Instant::now();
        self.orders.drain(..).collect()
    }

    /// Force flush the entire buffer
    fn force_flush(&mut self) -> Vec<Order> {
        if self.orders.is_empty() {
            Vec::new()
        } else {
            self.flush()
        }
    }
}

/// Simulate sending a batch of orders
fn send_batch(orders: Vec<Order>) {
    println!("\n=== Sending batch of {} orders ===", orders.len());
    for order in &orders {
        println!(
            "  {:?} {} {} @ {:.2}",
            order.side, order.quantity, order.symbol, order.price
        );
    }
    // In reality: one HTTP request instead of N
    println!("  Sent with a single network call!");
}

fn main() {
    let mut buffer = OrderBuffer::new(
        5,                             // Send when 5 orders accumulate
        Duration::from_millis(100),    // Or after 100ms
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

    // Send the remainder
    let remaining = buffer.force_flush();
    if !remaining.is_empty() {
        send_batch(remaining);
    }
}
```

## Non-blocking I/O and Multiplexing

Synchronous calls block the thread. Trading requires a non-blocking approach:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Simulation of a non-blocking network client
struct AsyncPriceClient {
    /// Requests awaiting response
    pending_requests: HashMap<u64, PendingRequest>,
    /// Request counter
    request_counter: u64,
    /// Received prices
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

    /// Send a request (non-blocking)
    fn request_price(&mut self, symbol: &str) -> u64 {
        self.request_counter += 1;
        let request_id = self.request_counter;

        self.pending_requests.insert(request_id, PendingRequest {
            symbol: symbol.to_string(),
            sent_at: Instant::now(),
        });

        println!("[Request #{}] Sent price request for {}", request_id, symbol);
        request_id
    }

    /// Process response (called when data is received)
    fn handle_response(&mut self, request_id: u64, price: f64) {
        if let Some(request) = self.pending_requests.remove(&request_id) {
            let latency = request.sent_at.elapsed();
            self.prices.insert(request.symbol.clone(), price);

            println!(
                "[Response #{}] {} = ${:.2} (latency: {:?})",
                request_id, request.symbol, price, latency
            );
        }
    }

    /// Check for timeouts
    fn check_timeouts(&mut self, timeout: Duration) -> Vec<u64> {
        let now = Instant::now();
        let timed_out: Vec<u64> = self.pending_requests
            .iter()
            .filter(|(_, req)| now.duration_since(req.sent_at) > timeout)
            .map(|(&id, _)| id)
            .collect();

        for id in &timed_out {
            if let Some(req) = self.pending_requests.remove(id) {
                println!("[Timeout #{}] Request for {} exceeded timeout", id, req.symbol);
            }
        }

        timed_out
    }

    /// Get all prices
    fn get_prices(&self) -> &HashMap<String, f64> {
        &self.prices
    }
}

fn main() {
    let mut client = AsyncPriceClient::new();

    // Send multiple requests simultaneously (don't wait for response)
    println!("=== Sending requests (non-blocking) ===");
    let btc_req = client.request_price("BTCUSDT");
    let eth_req = client.request_price("ETHUSDT");
    let sol_req = client.request_price("SOLUSDT");

    println!("\n=== Can do other work while waiting ===");
    println!("Calculating indicators...");
    println!("Checking risks...");

    println!("\n=== Responses received ===");
    // Simulate receiving responses (in different order!)
    client.handle_response(eth_req, 3000.0);
    client.handle_response(btc_req, 50000.0);
    client.handle_response(sol_req, 100.0);

    println!("\n=== Final prices ===");
    for (symbol, price) in client.get_prices() {
        println!("  {}: ${:.2}", symbol, price);
    }
}
```

## Data Compression

When transferring large volumes of data (trade history, order books), compression is critical:

```rust
use std::time::Instant;

/// Simulation of data compression effects
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

/// Order book data
struct OrderBookData {
    symbol: String,
    bids: Vec<(f64, f64)>,  // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBookData {
    /// Generate test order book data
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

    /// Size in bytes (approximate)
    fn size_bytes(&self) -> usize {
        self.symbol.len() + (self.bids.len() + self.asks.len()) * 16
    }

    /// Serialize to JSON (for comparison)
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

    /// Compact binary serialization
    fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Symbol (length + bytes)
        data.push(self.symbol.len() as u8);
        data.extend(self.symbol.as_bytes());

        // Number of levels
        data.extend(&(self.bids.len() as u16).to_le_bytes());

        // Bids (using delta-encoding for prices)
        let mut prev_price = 0i64;
        for (price, qty) in &self.bids {
            let price_int = (*price * 100.0) as i64;
            let delta = price_int - prev_price;
            prev_price = price_int;

            // Encode delta as varint (simplified)
            data.extend(&(delta as i32).to_le_bytes());
            data.extend(&((*qty * 10000.0) as u32).to_le_bytes());
        }

        // Asks similarly
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
    println!("=== Comparing Order Book Serialization Formats ===\n");

    for depth in [10, 100, 1000] {
        let order_book = OrderBookData::generate("BTCUSDT", depth);

        let json = order_book.to_json();
        let binary = order_book.to_binary();

        println!("Order book depth: {} levels", depth);
        println!("  JSON size: {} bytes", json.len());
        println!("  Binary size: {} bytes", binary.len());
        println!(
            "  Savings: {:.1}%",
            (1.0 - binary.len() as f64 / json.len() as f64) * 100.0
        );
        println!();
    }

    // Demonstrate impact on throughput
    println!("=== Impact on Throughput ===");
    let bandwidth_mbps = 100.0; // 100 Mbps
    let bytes_per_second = bandwidth_mbps * 1_000_000.0 / 8.0;

    let order_book = OrderBookData::generate("BTCUSDT", 100);
    let json_size = order_book.to_json().len();
    let binary_size = order_book.to_binary().len();

    let json_updates_per_sec = bytes_per_second / json_size as f64;
    let binary_updates_per_sec = bytes_per_second / binary_size as f64;

    println!("At {} Mbps bandwidth:", bandwidth_mbps);
    println!("  JSON: {:.0} updates/sec", json_updates_per_sec);
    println!("  Binary: {:.0} updates/sec", binary_updates_per_sec);
    println!("  Improvement: {:.1}x", binary_updates_per_sec / json_updates_per_sec);
}
```

## TCP Optimization for Trading

TCP settings significantly affect performance:

```rust
use std::collections::HashMap;

/// TCP configuration for a trading system
#[derive(Debug, Clone)]
struct TcpConfig {
    /// Disable Nagle's algorithm (important for low latency!)
    tcp_nodelay: bool,
    /// Send buffer size
    send_buffer_size: usize,
    /// Receive buffer size
    recv_buffer_size: usize,
    /// Keep-alive interval
    keepalive_interval_secs: Option<u64>,
    /// Connection timeout
    connect_timeout_ms: u64,
}

impl Default for TcpConfig {
    fn default() -> Self {
        TcpConfig {
            tcp_nodelay: true,  // CRITICAL for trading!
            send_buffer_size: 64 * 1024,
            recv_buffer_size: 64 * 1024,
            keepalive_interval_secs: Some(30),
            connect_timeout_ms: 5000,
        }
    }
}

impl TcpConfig {
    /// Configuration for high-frequency trading
    fn hft() -> Self {
        TcpConfig {
            tcp_nodelay: true,
            send_buffer_size: 256 * 1024,   // Larger buffer
            recv_buffer_size: 256 * 1024,
            keepalive_interval_secs: Some(10),
            connect_timeout_ms: 1000,        // Fast timeout
        }
    }

    /// Configuration for historical data loading
    fn bulk_data() -> Self {
        TcpConfig {
            tcp_nodelay: false,              // Nagle OK for bulk
            send_buffer_size: 1024 * 1024,   // 1MB buffer
            recv_buffer_size: 1024 * 1024,
            keepalive_interval_secs: Some(60),
            connect_timeout_ms: 30000,
        }
    }

    fn display(&self) {
        println!("TCP configuration:");
        println!("  TCP_NODELAY: {}", self.tcp_nodelay);
        println!("  Send buffer: {} KB", self.send_buffer_size / 1024);
        println!("  Receive buffer: {} KB", self.recv_buffer_size / 1024);
        if let Some(interval) = self.keepalive_interval_secs {
            println!("  Keep-alive: {} sec", interval);
        }
        println!("  Connection timeout: {} ms", self.connect_timeout_ms);
    }
}

fn explain_tcp_nodelay() {
    println!("=== Nagle's Algorithm and TCP_NODELAY ===\n");

    println!("Nagle's Algorithm:");
    println!("  - Buffers small packets");
    println!("  - Sends when enough data accumulates");
    println!("  - Reduces number of packets on the network");
    println!("  - Adds up to 200ms delay!\n");

    println!("TCP_NODELAY = true:");
    println!("  - Sends data IMMEDIATELY");
    println!("  - No buffering");
    println!("  - Critical for trading!");
    println!("  - Every millisecond counts\n");

    // Impact simulation
    let message_size = 100; // bytes
    let messages_per_second = 100;

    println!("Example: {} messages/sec at {} bytes", messages_per_second, message_size);
    println!("  With Nagle: up to 200ms delay = missed signals");
    println!("  With NODELAY: instant sending = fast reaction");
}

fn main() {
    println!("=== TCP Configurations for Different Scenarios ===\n");

    println!("1. Standard configuration:");
    TcpConfig::default().display();

    println!("\n2. High-frequency trading (HFT):");
    TcpConfig::hft().display();

    println!("\n3. Historical data loading:");
    TcpConfig::bulk_data().display();

    println!();
    explain_tcp_nodelay();
}
```

## DNS Caching

DNS queries add latency. Cache the results:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// DNS cache for a trading system
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

    /// Get IP for a host
    fn resolve(&mut self, hostname: &str) -> ResolveResult {
        // Check cache
        if let Some(entry) = self.cache.get(hostname) {
            if entry.is_valid() {
                return ResolveResult::Cached(entry.ip_addresses.clone());
            }
        }

        // Simulate DNS query
        let ips = self.do_dns_lookup(hostname);

        // Save to cache
        self.cache.insert(hostname.to_string(), DnsEntry {
            ip_addresses: ips.clone(),
            resolved_at: Instant::now(),
            ttl: self.default_ttl,
        });

        ResolveResult::Fresh(ips)
    }

    /// Simulate a DNS query
    fn do_dns_lookup(&self, hostname: &str) -> Vec<String> {
        // In reality: system call gethostbyname
        println!("[DNS] Performing lookup for {}", hostname);

        match hostname {
            "api.binance.com" => vec!["52.84.71.1".into(), "52.84.71.2".into()],
            "api.kraken.com" => vec!["104.20.48.1".into()],
            "api.coinbase.com" => vec!["104.18.6.1".into(), "104.18.7.1".into()],
            _ => vec!["127.0.0.1".into()],
        }
    }

    /// Pre-resolve critical hosts
    fn warmup(&mut self, hostnames: &[&str]) {
        println!("=== Warming up DNS cache ===");
        for hostname in hostnames {
            self.resolve(hostname);
        }
        println!("Cached {} hosts\n", hostnames.len());
    }

    fn stats(&self) {
        println!("\n=== DNS Cache Statistics ===");
        println!("Entries in cache: {}", self.cache.len());
        for (hostname, entry) in &self.cache {
            let age = entry.resolved_at.elapsed();
            let remaining = entry.ttl.saturating_sub(age);
            println!(
                "  {}: {} IPs, TTL remaining {:?}",
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
    let mut dns = DnsCache::new(Duration::from_secs(300)); // 5 minute TTL

    // Warm up cache at startup
    dns.warmup(&["api.binance.com", "api.kraken.com", "api.coinbase.com"]);

    // Subsequent requests use the cache
    println!("=== Requests after warmup ===");

    for _ in 0..3 {
        match dns.resolve("api.binance.com") {
            ResolveResult::Cached(ips) => {
                println!("[Cache] api.binance.com -> {:?}", ips);
            }
            ResolveResult::Fresh(ips) => {
                println!("[DNS] api.binance.com -> {:?}", ips);
            }
        }
    }

    dns.stats();
}
```

## Measuring and Monitoring Network Performance

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Network latency monitoring
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

    /// Record a measurement
    fn record(&mut self, latency: Duration) {
        if self.measurements.len() >= self.max_samples {
            self.measurements.pop_front();
        }
        self.measurements.push_back(latency);

        // Check thresholds
        let ms = latency.as_millis() as u64;
        if ms >= self.thresholds.critical_ms {
            println!("[CRITICAL] Latency {} ms!", ms);
        } else if ms >= self.thresholds.warning_ms {
            println!("[Warning] High latency: {} ms", ms);
        }
    }

    /// Average latency
    fn average(&self) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        let total: Duration = self.measurements.iter().sum();
        Some(total / self.measurements.len() as u32)
    }

    /// Latency percentile
    fn percentile(&self, p: f64) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = self.measurements.iter().cloned().collect();
        sorted.sort();

        let index = ((sorted.len() as f64 * p / 100.0) as usize).min(sorted.len() - 1);
        Some(sorted[index])
    }

    /// Latency report
    fn report(&self) {
        println!("\n=== Network Latency Report ===");
        println!("Measurements: {}", self.measurements.len());

        if let Some(avg) = self.average() {
            println!("Average: {:?}", avg);
        }

        if let Some(p50) = self.percentile(50.0) {
            println!("P50 (median): {:?}", p50);
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

/// Simulate measuring request latency
fn measure_request_latency() -> Duration {
    // Simulation: random latency from 5 to 50 ms
    // with occasional spikes to 200 ms
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
        1000,   // Store 1000 measurements
        50,     // Warning > 50 ms
        100,    // Critical > 100 ms
    );

    println!("=== Simulating Trading Request Monitoring ===\n");

    // Simulate 100 requests
    for i in 0..100 {
        let latency = measure_request_latency();

        if i % 20 == 0 {
            println!("Request #{}: {:?}", i, latency);
        }

        monitor.record(latency);
    }

    monitor.report();
}
```

## What We Learned

| Technique | Description | Benefit |
|-----------|-------------|---------|
| **Connection pooling** | Reusing connections | Avoid 100+ ms handshake |
| **TCP_NODELAY** | Disable Nagle's algorithm | Remove 200 ms buffering |
| **Batch processing** | Group requests | Fewer network calls |
| **Data compression** | Binary format instead of JSON | 50-80% bandwidth savings |
| **DNS caching** | Store resolution results | Remove 10-50 ms per query |
| **Monitoring** | Track latencies | Early problem detection |

## Practical Exercises

1. **Priority Connection Pool**: Create a pool that:
   - Allocates more connections to critical exchanges
   - Auto-reconnects on errors
   - Balances load between connections
   - Collects metrics for each connection

2. **Intelligent Batcher**: Implement an order buffer that:
   - Groups orders by exchange
   - Considers priority (market orders over limit orders)
   - Sends critical orders immediately
   - Dynamically optimizes batch size

3. **Adaptive Compression**: Create a system that:
   - Selects compression algorithm by data type
   - Disables compression for small messages
   - Measures compression benefit in real-time
   - Adapts to channel bandwidth

## Homework

1. **Optimized WebSocket Client**: Write a client for trading data:
   - Support multiple exchanges simultaneously
   - Automatic reconnection
   - Ping/pong for connection health
   - Metrics per connection
   - Message buffering during reconnection

2. **Early Warning System**: Create monitoring that:
   - Tracks latency to each exchange
   - Compares with historical data
   - Alerts on anomalies
   - Visualizes latency trends
   - Integrates with trading strategy

3. **Network Stack Benchmark**: Develop a tool that:
   - Measures actual latency to exchanges
   - Tests different TCP settings
   - Compares HTTP/1.1 vs HTTP/2
   - Evaluates VPN/proxy impact
   - Generates report with recommendations

4. **Minimal Latency Protocol**: Design a protocol:
   - Binary message format
   - Delta-encoding for price updates
   - Built-in integrity checking
   - Message priority support
   - Compare with JSON for latency and bandwidth

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../329-*/en.md)
