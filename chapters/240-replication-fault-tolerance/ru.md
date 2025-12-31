# День 240: Репликация и отказоустойчивость

## Аналогия из трейдинга

Представь, что ты управляешь торговым деском в крупном инвестиционном банке. Ты не можешь позволить себе единую точку отказа — если твоя основная торговая система упадёт во время торговой сессии, ты можешь потерять миллионы на упущенных возможностях или, что ещё хуже, не сможешь закрыть позиции во время обвала рынка.

Профессиональные торговые фирмы решают эту проблему, запуская **несколько идентичных систем** параллельно. Если один сервер выходит из строя, другой немедленно берёт на себя его функции. Это **репликация** — хранение нескольких копий данных и систем. В сочетании с **отказоустойчивостью** — способностью продолжать работу при поломках — это гарантирует, что торговые операции никогда не остановятся.

В мире баз данных:
- **Основной сервер** = Твой главный торговый деск, обрабатывающий все ордера
- **Серверы-реплики** = Резервные дески, готовые мгновенно взять управление
- **Failover** = Когда резервный деск становится основным после сбоя
- **Синхронизация данных** = Обеспечение одинаковой информации о портфеле и ордерах на всех десках

## Почему репликация важна для торговых систем

Торговые системы требуют:
1. **Высокая доступность** — Рынок не ждёт. Если система недоступна, ты упускаешь сделки
2. **Сохранность данных** — Нельзя терять записи о сделках или данные о позициях
3. **Масштабируемость чтения** — Несколько сервисов должны одновременно читать ценовые данные
4. **Географическое распределение** — Торгуй на биржах по всему миру с низкой задержкой

## Стратегии репликации

### 1. Репликация Master-Slave (Primary-Replica)

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// Представляет запись о сделке, которую нужно реплицировать
#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

/// Симулирует узел базы данных (основной или реплика)
#[derive(Debug)]
struct DatabaseNode {
    name: String,
    is_primary: bool,
    trades: RwLock<HashMap<u64, Trade>>,
    replication_lag_ms: u64,
}

impl DatabaseNode {
    fn new(name: &str, is_primary: bool, lag_ms: u64) -> Self {
        DatabaseNode {
            name: name.to_string(),
            is_primary,
            trades: RwLock::new(HashMap::new()),
            replication_lag_ms: lag_ms,
        }
    }

    fn write_trade(&self, trade: Trade) -> Result<(), String> {
        if !self.is_primary {
            return Err("Нельзя писать напрямую в реплику!".to_string());
        }

        let mut trades = self.trades.write().unwrap();
        println!("[{}] Записываю сделку: {:?}", self.name, trade);
        trades.insert(trade.id, trade);
        Ok(())
    }

    fn replicate_trade(&self, trade: Trade) {
        // Симулируем задержку репликации
        thread::sleep(Duration::from_millis(self.replication_lag_ms));

        let mut trades = self.trades.write().unwrap();
        println!("[{}] Реплицирована сделка {} (задержка: {}мс)",
            self.name, trade.id, self.replication_lag_ms);
        trades.insert(trade.id, trade);
    }

    fn read_trade(&self, id: u64) -> Option<Trade> {
        let trades = self.trades.read().unwrap();
        trades.get(&id).cloned()
    }

    fn trade_count(&self) -> usize {
        self.trades.read().unwrap().len()
    }
}

/// Торговая система с репликацией
struct ReplicatedTradingSystem {
    primary: Arc<DatabaseNode>,
    replicas: Vec<Arc<DatabaseNode>>,
    next_trade_id: RwLock<u64>,
}

impl ReplicatedTradingSystem {
    fn new() -> Self {
        let primary = Arc::new(DatabaseNode::new("Primary", true, 0));
        let replicas = vec![
            Arc::new(DatabaseNode::new("Replica-1", false, 10)),
            Arc::new(DatabaseNode::new("Replica-2", false, 25)),
            Arc::new(DatabaseNode::new("Replica-3", false, 50)),
        ];

        ReplicatedTradingSystem {
            primary,
            replicas,
            next_trade_id: RwLock::new(1),
        }
    }

    fn execute_trade(&self, symbol: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Получаем следующий ID сделки
        let trade_id = {
            let mut id = self.next_trade_id.write().unwrap();
            let current = *id;
            *id += 1;
            current
        };

        let trade = Trade {
            id: trade_id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Пишем в основной сервер
        self.primary.write_trade(trade.clone())?;

        // Асинхронно реплицируем на все реплики
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let trade = trade.clone();
            thread::spawn(move || {
                replica.replicate_trade(trade);
            });
        }

        Ok(trade_id)
    }

    fn read_trade_from_replica(&self, id: u64, replica_index: usize) -> Option<Trade> {
        if replica_index < self.replicas.len() {
            self.replicas[replica_index].read_trade(id)
        } else {
            None
        }
    }
}

fn main() {
    let system = ReplicatedTradingSystem::new();

    println!("=== Выполняем сделки на Primary ===\n");

    // Выполняем несколько сделок
    let trade1 = system.execute_trade("BTC/USD", 42000.0, 0.5).unwrap();
    let trade2 = system.execute_trade("ETH/USD", 2800.0, 5.0).unwrap();
    let trade3 = system.execute_trade("SOL/USD", 95.0, 100.0).unwrap();

    println!("\nID сделок: {}, {}, {}", trade1, trade2, trade3);

    // Ждём репликации
    println!("\n=== Ожидаем репликацию ===\n");
    thread::sleep(Duration::from_millis(100));

    // Проверяем репликацию
    println!("\n=== Проверяем репликацию ===\n");
    println!("Сделок на Primary: {}", system.primary.trade_count());
    for (i, replica) in system.replicas.iter().enumerate() {
        println!("Сделок на Replica-{}: {}", i + 1, replica.trade_count());
    }
}
```

### 2. Синхронная и асинхронная репликация

```rust
use std::sync::{Arc, Mutex, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct OrderUpdate {
    order_id: u64,
    status: String,
    filled_quantity: f64,
}

/// Синхронная репликация — ждёт подтверждения от всех реплик
struct SyncReplication {
    replicas: Vec<Arc<Mutex<Vec<OrderUpdate>>>>,
}

impl SyncReplication {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect();
        SyncReplication { replicas }
    }

    fn replicate(&self, update: OrderUpdate) -> Duration {
        let start = Instant::now();

        // Используем барьер для синхронизации записей на всех репликах
        let barrier = Arc::new(Barrier::new(self.replicas.len() + 1));
        let mut handles = vec![];

        for (i, replica) in self.replicas.iter().enumerate() {
            let replica = Arc::clone(replica);
            let update = update.clone();
            let barrier = Arc::clone(&barrier);

            handles.push(thread::spawn(move || {
                // Симулируем сетевую задержку (варьируется по репликам)
                thread::sleep(Duration::from_millis(10 + (i as u64 * 15)));

                let mut data = replica.lock().unwrap();
                data.push(update);

                // Сигнализируем о завершении
                barrier.wait();
            }));
        }

        // Ждём все реплики
        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        start.elapsed()
    }
}

/// Асинхронная репликация — возвращается сразу, реплицирует в фоне
struct AsyncReplication {
    replicas: Vec<Arc<Mutex<Vec<OrderUpdate>>>>,
    pending_count: Arc<Mutex<u64>>,
}

impl AsyncReplication {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect();
        AsyncReplication {
            replicas,
            pending_count: Arc::new(Mutex::new(0)),
        }
    }

    fn replicate(&self, update: OrderUpdate) -> Duration {
        let start = Instant::now();

        {
            let mut pending = self.pending_count.lock().unwrap();
            *pending += self.replicas.len() as u64;
        }

        for (i, replica) in self.replicas.iter().enumerate() {
            let replica = Arc::clone(replica);
            let update = update.clone();
            let pending_count = Arc::clone(&self.pending_count);

            thread::spawn(move || {
                // Симулируем сетевую задержку
                thread::sleep(Duration::from_millis(10 + (i as u64 * 15)));

                let mut data = replica.lock().unwrap();
                data.push(update);

                let mut pending = pending_count.lock().unwrap();
                *pending -= 1;
            });
        }

        // Возвращаемся сразу без ожидания
        start.elapsed()
    }

    fn pending_replications(&self) -> u64 {
        *self.pending_count.lock().unwrap()
    }
}

fn main() {
    let update = OrderUpdate {
        order_id: 12345,
        status: "FILLED".to_string(),
        filled_quantity: 100.0,
    };

    println!("=== Синхронная репликация ===\n");
    let sync_replication = SyncReplication::new(3);
    let sync_time = sync_replication.replicate(update.clone());
    println!("Синхронная репликация заняла: {:?}", sync_time);
    println!("Все реплики подтвердили до возврата\n");

    println!("=== Асинхронная репликация ===\n");
    let async_replication = AsyncReplication::new(3);
    let async_time = async_replication.replicate(update.clone());
    println!("Асинхронная репликация вернулась за: {:?}", async_time);
    println!("Ожидающих репликаций: {}", async_replication.pending_replications());

    // Ждём и проверяем снова
    thread::sleep(Duration::from_millis(100));
    println!("После ожидания - Ожидающих: {}", async_replication.pending_replications());

    println!("\n=== Компромиссы ===");
    println!("Синхронная: Медленнее ({:?}), но гарантирует сохранность", sync_time);
    println!("Асинхронная: Быстрее ({:?}), но может потерять данные при сбое", async_time);
}
```

## Паттерны отказоустойчивости

### 1. Проверки здоровья и Failover

```rust
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum NodeStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug)]
struct TradingNode {
    id: String,
    is_alive: AtomicBool,
    last_heartbeat: RwLock<Instant>,
    processed_orders: AtomicU64,
    status: RwLock<NodeStatus>,
}

impl TradingNode {
    fn new(id: &str) -> Self {
        TradingNode {
            id: id.to_string(),
            is_alive: AtomicBool::new(true),
            last_heartbeat: RwLock::new(Instant::now()),
            processed_orders: AtomicU64::new(0),
            status: RwLock::new(NodeStatus::Healthy),
        }
    }

    fn heartbeat(&self) {
        *self.last_heartbeat.write().unwrap() = Instant::now();
    }

    fn process_order(&self) -> Result<u64, String> {
        if !self.is_alive.load(Ordering::SeqCst) {
            return Err(format!("Узел {} недоступен!", self.id));
        }

        let order_num = self.processed_orders.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(order_num)
    }

    fn simulate_failure(&self) {
        self.is_alive.store(false, Ordering::SeqCst);
        *self.status.write().unwrap() = NodeStatus::Failed;
    }

    fn recover(&self) {
        self.is_alive.store(true, Ordering::SeqCst);
        self.heartbeat();
        *self.status.write().unwrap() = NodeStatus::Healthy;
    }
}

struct FaultTolerantCluster {
    nodes: Vec<Arc<TradingNode>>,
    primary_index: RwLock<usize>,
    heartbeat_timeout: Duration,
}

impl FaultTolerantCluster {
    fn new(node_count: usize) -> Self {
        let nodes = (0..node_count)
            .map(|i| Arc::new(TradingNode::new(&format!("node-{}", i))))
            .collect();

        FaultTolerantCluster {
            nodes,
            primary_index: RwLock::new(0),
            heartbeat_timeout: Duration::from_millis(100),
        }
    }

    fn get_primary(&self) -> Arc<TradingNode> {
        let index = *self.primary_index.read().unwrap();
        Arc::clone(&self.nodes[index])
    }

    fn check_health(&self) -> HashMap<String, NodeStatus> {
        let mut health = HashMap::new();
        let now = Instant::now();

        for node in &self.nodes {
            let last_hb = *node.last_heartbeat.read().unwrap();
            let status = if !node.is_alive.load(Ordering::SeqCst) {
                NodeStatus::Failed
            } else if now.duration_since(last_hb) > self.heartbeat_timeout {
                NodeStatus::Degraded
            } else {
                NodeStatus::Healthy
            };

            *node.status.write().unwrap() = status.clone();
            health.insert(node.id.clone(), status);
        }

        health
    }

    fn failover(&self) -> Result<(), String> {
        let current_primary = *self.primary_index.read().unwrap();

        // Ищем следующий здоровый узел
        for i in 1..self.nodes.len() {
            let candidate_index = (current_primary + i) % self.nodes.len();
            let candidate = &self.nodes[candidate_index];

            if candidate.is_alive.load(Ordering::SeqCst) {
                *self.primary_index.write().unwrap() = candidate_index;
                println!("FAILOVER: {} теперь основной", candidate.id);
                return Ok(());
            }
        }

        Err("Нет здоровых узлов для failover!".to_string())
    }

    fn process_order(&self) -> Result<u64, String> {
        let primary = self.get_primary();

        match primary.process_order() {
            Ok(order_num) => Ok(order_num),
            Err(_) => {
                println!("Основной узел недоступен! Запускаем failover...");
                self.failover()?;
                // Повторяем на новом основном узле
                self.get_primary().process_order()
            }
        }
    }
}

fn main() {
    let cluster = Arc::new(FaultTolerantCluster::new(3));

    println!("=== Отказоустойчивый торговый кластер ===\n");

    // Обрабатываем ордера в нормальном режиме
    for i in 1..=5 {
        match cluster.process_order() {
            Ok(num) => println!("Ордер {} обработан (всего: {})", i, num),
            Err(e) => println!("Ордер {} не удался: {}", i, e),
        }
    }

    println!("\n=== Проверка здоровья ===");
    let health = cluster.check_health();
    for (node, status) in &health {
        println!("{}: {:?}", node, status);
    }

    // Симулируем сбой основного узла
    println!("\n=== Симулируем сбой основного узла ===\n");
    cluster.get_primary().simulate_failure();

    // Пробуем обработать ещё ордера (должен сработать failover)
    for i in 6..=10 {
        match cluster.process_order() {
            Ok(num) => println!("Ордер {} обработан (всего: {})", i, num),
            Err(e) => println!("Ордер {} не удался: {}", i, e),
        }
    }

    println!("\n=== Финальная проверка здоровья ===");
    let health = cluster.check_health();
    for (node, status) in &health {
        println!("{}: {:?}", node, status);
    }
}
```

### 2. Write-Ahead Log (WAL) для надёжности

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::path::Path;

#[derive(Debug, Clone)]
struct TradeEntry {
    sequence: u64,
    trade_id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

impl TradeEntry {
    fn to_wal_format(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}\n",
            self.sequence, self.trade_id, self.symbol,
            self.side, self.price, self.quantity
        )
    }

    fn from_wal_format(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() != 6 {
            return None;
        }

        Some(TradeEntry {
            sequence: parts[0].parse().ok()?,
            trade_id: parts[1].parse().ok()?,
            symbol: parts[2].to_string(),
            side: parts[3].to_string(),
            price: parts[4].parse().ok()?,
            quantity: parts[5].parse().ok()?,
        })
    }
}

struct WriteAheadLog {
    file: BufWriter<File>,
    sequence: u64,
}

impl WriteAheadLog {
    fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Подсчитываем существующие записи для получения номера последовательности
        let sequence = if Path::new(path).exists() {
            let reader = BufReader::new(File::open(path)?);
            reader.lines().count() as u64
        } else {
            0
        };

        Ok(WriteAheadLog {
            file: BufWriter::new(file),
            sequence,
        })
    }

    fn append(&mut self, mut entry: TradeEntry) -> std::io::Result<u64> {
        self.sequence += 1;
        entry.sequence = self.sequence;

        let data = entry.to_wal_format();
        self.file.write_all(data.as_bytes())?;
        self.file.flush()?; // Гарантируем запись на диск

        Ok(self.sequence)
    }

    fn recover(path: &str) -> std::io::Result<Vec<TradeEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let entries: Vec<TradeEntry> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| TradeEntry::from_wal_format(&line))
            .collect();

        Ok(entries)
    }
}

struct DurableTradingEngine {
    wal: WriteAheadLog,
    trades: Vec<TradeEntry>,
    next_trade_id: u64,
}

impl DurableTradingEngine {
    fn new(wal_path: &str) -> std::io::Result<Self> {
        // Сначала пробуем восстановиться из существующего WAL
        let trades = if Path::new(wal_path).exists() {
            println!("Восстанавливаемся из WAL...");
            WriteAheadLog::recover(wal_path)?
        } else {
            vec![]
        };

        let next_trade_id = trades.iter()
            .map(|t| t.trade_id)
            .max()
            .unwrap_or(0) + 1;

        println!("Восстановлено {} сделок, следующий ID: {}", trades.len(), next_trade_id);

        Ok(DurableTradingEngine {
            wal: WriteAheadLog::new(wal_path)?,
            trades,
            next_trade_id,
        })
    }

    fn execute_trade(&mut self, symbol: &str, side: &str, price: f64, quantity: f64)
        -> std::io::Result<u64>
    {
        let trade_id = self.next_trade_id;
        self.next_trade_id += 1;

        let entry = TradeEntry {
            sequence: 0, // Будет установлено WAL
            trade_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
        };

        // Пишем в WAL СНАЧАЛА (до обновления в памяти)
        let seq = self.wal.append(entry.clone())?;
        println!("Сделка {} записана в WAL (seq: {})", trade_id, seq);

        // Затем обновляем состояние в памяти
        self.trades.push(entry);

        Ok(trade_id)
    }

    fn get_trades(&self) -> &[TradeEntry] {
        &self.trades
    }
}

fn main() -> std::io::Result<()> {
    let wal_path = "/tmp/trading_wal.log";

    // Чистый старт для демонстрации
    if Path::new(wal_path).exists() {
        std::fs::remove_file(wal_path)?;
    }

    println!("=== Демонстрация Write-Ahead Log ===\n");

    // Первый запуск — создаём сделки
    {
        let mut engine = DurableTradingEngine::new(wal_path)?;

        engine.execute_trade("BTC/USD", "BUY", 42000.0, 1.0)?;
        engine.execute_trade("ETH/USD", "SELL", 2800.0, 10.0)?;
        engine.execute_trade("SOL/USD", "BUY", 95.0, 50.0)?;

        println!("\nСделок в памяти: {}", engine.get_trades().len());
    }

    println!("\n=== Симулируем сбой и восстановление ===\n");

    // Второй запуск — восстанавливаемся из WAL
    {
        let engine = DurableTradingEngine::new(wal_path)?;

        println!("\nВосстановленные сделки:");
        for trade in engine.get_trades() {
            println!("  {:?}", trade);
        }
    }

    // Очистка
    std::fs::remove_file(wal_path)?;

    Ok(())
}
```

### 3. Консистентность на основе кворума

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    version: u64,
}

#[derive(Debug)]
struct QuorumNode {
    id: String,
    prices: RwLock<HashMap<String, PriceUpdate>>,
    latency_ms: u64,
}

impl QuorumNode {
    fn new(id: &str, latency_ms: u64) -> Self {
        QuorumNode {
            id: id.to_string(),
            prices: RwLock::new(HashMap::new()),
            latency_ms,
        }
    }

    fn write(&self, update: PriceUpdate) -> bool {
        thread::sleep(Duration::from_millis(self.latency_ms));

        let mut prices = self.prices.write().unwrap();

        // Принимаем только если версия выше
        if let Some(existing) = prices.get(&update.symbol) {
            if existing.version >= update.version {
                return false;
            }
        }

        prices.insert(update.symbol.clone(), update);
        true
    }

    fn read(&self, symbol: &str) -> Option<PriceUpdate> {
        thread::sleep(Duration::from_millis(self.latency_ms));

        let prices = self.prices.read().unwrap();
        prices.get(symbol).cloned()
    }
}

struct QuorumCluster {
    nodes: Vec<Arc<QuorumNode>>,
    write_quorum: usize,  // W
    read_quorum: usize,   // R
}

impl QuorumCluster {
    fn new(node_count: usize, write_quorum: usize, read_quorum: usize) -> Self {
        assert!(write_quorum + read_quorum > node_count,
            "W + R должно быть больше N для консистентности!");

        let nodes = (0..node_count)
            .map(|i| Arc::new(QuorumNode::new(
                &format!("node-{}", i),
                10 + (i as u64 * 5), // Разные задержки
            )))
            .collect();

        QuorumCluster {
            nodes,
            write_quorum,
            read_quorum,
        }
    }

    fn write(&self, update: PriceUpdate) -> Result<usize, String> {
        let mut handles = vec![];

        for node in &self.nodes {
            let node = Arc::clone(node);
            let update = update.clone();
            handles.push(thread::spawn(move || {
                node.write(update)
            }));
        }

        let mut success_count = 0;
        for handle in handles {
            if handle.join().unwrap() {
                success_count += 1;
            }
        }

        if success_count >= self.write_quorum {
            Ok(success_count)
        } else {
            Err(format!("Кворум записи не достигнут: {} < {}",
                success_count, self.write_quorum))
        }
    }

    fn read(&self, symbol: &str) -> Option<PriceUpdate> {
        let mut handles = vec![];

        for node in &self.nodes {
            let node = Arc::clone(node);
            let symbol = symbol.to_string();
            handles.push(thread::spawn(move || {
                node.read(&symbol)
            }));
        }

        let mut results: Vec<PriceUpdate> = vec![];
        for handle in handles {
            if let Some(update) = handle.join().unwrap() {
                results.push(update);
            }

            // Возвращаем как только достигли кворума чтения
            if results.len() >= self.read_quorum {
                break;
            }
        }

        if results.len() >= self.read_quorum {
            // Возвращаем значение с максимальной версией
            results.into_iter().max_by_key(|u| u.version)
        } else {
            None
        }
    }
}

fn main() {
    println!("=== Консистентность на основе кворума ===\n");
    println!("N=5, W=3, R=3 (W+R > N гарантирует консистентность)\n");

    let cluster = QuorumCluster::new(5, 3, 3);

    // Записываем обновления цен
    let updates = vec![
        PriceUpdate { symbol: "BTC/USD".into(), price: 42000.0, version: 1 },
        PriceUpdate { symbol: "BTC/USD".into(), price: 42100.0, version: 2 },
        PriceUpdate { symbol: "ETH/USD".into(), price: 2800.0, version: 1 },
    ];

    for update in updates {
        println!("Записываем: {} = ${} (v{})", update.symbol, update.price, update.version);
        match cluster.write(update) {
            Ok(count) => println!("  Успех: {} узлов подтвердили\n", count),
            Err(e) => println!("  Ошибка: {}\n", e),
        }
    }

    // Читаем цены
    println!("=== Читаем цены ===\n");

    for symbol in &["BTC/USD", "ETH/USD", "SOL/USD"] {
        match cluster.read(symbol) {
            Some(update) => {
                println!("{}: ${} (версия {})", symbol, update.price, update.version);
            }
            None => println!("{}: Не найден или кворум не достигнут", symbol),
        }
    }

    println!("\n=== Свойства кворума ===");
    println!("- Кворум чтения (R=3) пересекается с кворумом записи (W=3)");
    println!("- Это гарантирует чтение последнего записанного значения");
    println!("- Можем пережить отказ 2 узлов для чтения и записи");
}
```

## Практический пример: Отказоустойчивая система управления ордерами

```rust
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    status: OrderStatus,
    created_at: u64,
}

#[derive(Debug)]
struct ReplicaState {
    orders: RwLock<HashMap<u64, Order>>,
    last_sync: RwLock<Instant>,
    is_healthy: AtomicBool,
}

impl ReplicaState {
    fn new() -> Self {
        ReplicaState {
            orders: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(Instant::now()),
            is_healthy: AtomicBool::new(true),
        }
    }
}

struct ResilientOrderSystem {
    primary: Arc<ReplicaState>,
    replicas: Vec<Arc<ReplicaState>>,
    next_order_id: AtomicU64,
    sync_interval: Duration,
}

impl ResilientOrderSystem {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(ReplicaState::new()))
            .collect();

        ResilientOrderSystem {
            primary: Arc::new(ReplicaState::new()),
            replicas,
            next_order_id: AtomicU64::new(1),
            sync_interval: Duration::from_millis(50),
        }
    }

    fn place_order(&self, symbol: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Проверяем здоровье основного узла
        if !self.primary.is_healthy.load(Ordering::SeqCst) {
            return Err("Основной узел недоступен, отклоняем новые ордера".to_string());
        }

        let order_id = self.next_order_id.fetch_add(1, Ordering::SeqCst);

        let order = Order {
            id: order_id,
            symbol: symbol.to_string(),
            price,
            quantity,
            status: OrderStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Пишем в основной узел
        {
            let mut orders = self.primary.orders.write().unwrap();
            orders.insert(order_id, order.clone());
        }

        // Асинхронная репликация
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let order = order.clone();
            thread::spawn(move || {
                if replica.is_healthy.load(Ordering::SeqCst) {
                    let mut orders = replica.orders.write().unwrap();
                    orders.insert(order.id, order);
                    *replica.last_sync.write().unwrap() = Instant::now();
                }
            });
        }

        Ok(order_id)
    }

    fn update_order_status(&self, order_id: u64, status: OrderStatus) -> Result<(), String> {
        // Обновляем основной узел
        {
            let mut orders = self.primary.orders.write().unwrap();
            if let Some(order) = orders.get_mut(&order_id) {
                order.status = status.clone();
            } else {
                return Err(format!("Ордер {} не найден", order_id));
            }
        }

        // Реплицируем обновление статуса
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let status = status.clone();
            thread::spawn(move || {
                if replica.is_healthy.load(Ordering::SeqCst) {
                    let mut orders = replica.orders.write().unwrap();
                    if let Some(order) = orders.get_mut(&order_id) {
                        order.status = status;
                    }
                }
            });
        }

        Ok(())
    }

    fn get_order(&self, order_id: u64) -> Option<Order> {
        // Сначала пробуем основной узел
        if self.primary.is_healthy.load(Ordering::SeqCst) {
            let orders = self.primary.orders.read().unwrap();
            if let Some(order) = orders.get(&order_id) {
                return Some(order.clone());
            }
        }

        // Fallback на реплики
        for replica in &self.replicas {
            if replica.is_healthy.load(Ordering::SeqCst) {
                let orders = replica.orders.read().unwrap();
                if let Some(order) = orders.get(&order_id) {
                    return Some(order.clone());
                }
            }
        }

        None
    }

    fn get_replication_status(&self) -> Vec<(usize, bool, Duration)> {
        self.replicas.iter().enumerate().map(|(i, replica)| {
            let is_healthy = replica.is_healthy.load(Ordering::SeqCst);
            let lag = replica.last_sync.read().unwrap().elapsed();
            (i, is_healthy, lag)
        }).collect()
    }

    fn simulate_replica_failure(&self, index: usize) {
        if index < self.replicas.len() {
            self.replicas[index].is_healthy.store(false, Ordering::SeqCst);
        }
    }

    fn recover_replica(&self, index: usize) {
        if index < self.replicas.len() {
            // Копируем все данные с основного узла на реплику
            let primary_orders = self.primary.orders.read().unwrap();
            let mut replica_orders = self.replicas[index].orders.write().unwrap();

            *replica_orders = primary_orders.clone();

            self.replicas[index].is_healthy.store(true, Ordering::SeqCst);
            *self.replicas[index].last_sync.write().unwrap() = Instant::now();
        }
    }
}

fn main() {
    println!("=== Отказоустойчивая система управления ордерами ===\n");

    let system = Arc::new(ResilientOrderSystem::new(3));

    // Размещаем несколько ордеров
    println!("Размещаем ордера...\n");
    let order1 = system.place_order("BTC/USD", 42000.0, 1.0).unwrap();
    let order2 = system.place_order("ETH/USD", 2800.0, 5.0).unwrap();
    let order3 = system.place_order("SOL/USD", 95.0, 100.0).unwrap();

    thread::sleep(Duration::from_millis(50));

    println!("Размещённые ордера: {}, {}, {}", order1, order2, order3);

    // Проверяем статус репликации
    println!("\nСтатус репликации:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Реплика {}: здорова={}, задержка={:?}", i, healthy, lag);
    }

    // Обновляем статус ордеров
    println!("\nОбновляем статусы ордеров...");
    system.update_order_status(order1, OrderStatus::Filled).unwrap();
    system.update_order_status(order2, OrderStatus::Cancelled).unwrap();

    thread::sleep(Duration::from_millis(50));

    // Симулируем сбой
    println!("\n=== Симулируем сбой Реплики 0 ===\n");
    system.simulate_replica_failure(0);

    // Размещаем ещё ордера (должно работать)
    let order4 = system.place_order("DOGE/USD", 0.08, 10000.0).unwrap();
    println!("Ордер {} размещён несмотря на сбой реплики", order4);

    // Снова проверяем статус репликации
    println!("\nСтатус репликации после сбоя:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Реплика {}: здорова={}, задержка={:?}", i, healthy, lag);
    }

    // Восстанавливаем сбойную реплику
    println!("\n=== Восстанавливаем Реплику 0 ===\n");
    system.recover_replica(0);

    println!("Статус репликации после восстановления:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Реплика {}: здорова={}, задержка={:?}", i, healthy, lag);
    }

    // Проверяем, что все ордера доступны
    println!("\n=== Проверяем данные ордеров ===\n");
    for id in [order1, order2, order3, order4] {
        if let Some(order) = system.get_order(id) {
            println!("Ордер {}: {} {} @ ${} - {:?}",
                order.id, order.quantity, order.symbol, order.price, order.status);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Репликация | Хранение нескольких копий данных на разных узлах |
| Primary-Replica | Один узел обрабатывает записи, остальные — чтения |
| Синхронная репликация | Ждём подтверждения всех реплик перед подтверждением записи |
| Асинхронная репликация | Запись возвращается сразу, репликация в фоне |
| Отказоустойчивость | Способность продолжать работу при сбоях |
| Failover | Автоматическое переключение на резерв при сбое основного узла |
| Write-Ahead Log (WAL) | Журнал записей для восстановления после сбоя |
| Кворум | Требование согласия большинства для консистентности |
| Проверки здоровья | Постоянный мониторинг доступности узлов |

## Домашнее задание

1. **Мульти-региональная репликация**: Реализуй торговую систему с репликами в трёх "регионах" (симулируй разными задержками). Обеспечь eventual consistency между всеми регионами. Добавь логику предпочтения чтения из ближайшей реплики.

2. **Разрешение конфликтов**: Создай систему, где две реплики могут получить конфликтующие обновления (например, разные цены для одного символа). Реализуй подход "last-write-wins" с использованием временных меток и подход "version vector". Сравни результаты.

3. **Автоматический Failover с выбором лидера**: Реализуй простой алгоритм выбора лидера, где узлы голосуют за нового основного, когда текущий выходит из строя. Используй heartbeat для обнаружения сбоев и гарантируй, что только один узел становится новым лидером.

4. **Consistent Hashing для распределения ордеров**: Построй систему, распределяющую ордера по нескольким узлам с использованием consistent hashing. При выходе узла из строя ордера должны перераспределяться на оставшиеся узлы с минимальным перемещением данных.

## Навигация

[← Предыдущий день](../239-clickhouse-big-data/ru.md) | [Следующий день →](../241-backups-preserving-history/ru.md)
