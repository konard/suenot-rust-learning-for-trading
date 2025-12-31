# День 358: Горизонтальное масштабирование

## Аналогия из трейдинга

Представь, что ты управляешь высокочастотной торговой компанией. В начале у тебя один мощный сервер, обрабатывающий все ордера. Но по мере роста бизнеса ты сталкиваешься с проблемами:

- В пиковые часы торговли сервер не справляется с нагрузкой
- Если сервер выходит из строя — все ордера останавливаются
- Добавление мощности к одному серверу становится слишком дорогим

**Горизонтальное масштабирование в трейдинге:**
Вместо покупки одного суперкомпьютера за $1 миллион ты можешь:

1. **Разделить по инструментам**: Один кластер серверов для криптовалют, другой — для акций
2. **Разделить по функциям**: Отдельные кластеры для исполнения ордеров, анализа рынка, управления рисками
3. **Географическое распределение**: Серверы ближе к биржам для минимальной задержки

| Подход | Стоимость | Отказоустойчивость | Сложность |
|--------|-----------|-------------------|-----------|
| **Вертикальное** | Высокая | Низкая (один сервер) | Низкая |
| **Горизонтальное** | Средняя | Высокая (много серверов) | Высокая |
| **Гибридное** | Оптимальная | Высокая | Средняя |

**Почему горизонтальное масштабирование?**
В торговле горизонтальное масштабирование критически важно:
- **Отказоустойчивость**: Если один сервер падает, другие продолжают работать
- **Эластичность**: Легко добавить мощности во время высокой волатильности
- **Экономия**: 10 серверов за $10k дешевле одного за $1M с той же мощностью

## Основы горизонтального масштабирования

### Балансировка нагрузки для ордеров

```rust
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// Состояние рабочего узла обработки ордеров
#[derive(Debug, Clone)]
pub struct WorkerNode {
    pub id: String,
    pub address: String,
    pub capacity: u64,
    pub current_load: Arc<AtomicU64>,
    pub is_healthy: bool,
    pub latency_ms: f64,
}

impl WorkerNode {
    pub fn new(id: &str, address: &str, capacity: u64) -> Self {
        WorkerNode {
            id: id.to_string(),
            address: address.to_string(),
            capacity,
            current_load: Arc::new(AtomicU64::new(0)),
            is_healthy: true,
            latency_ms: 0.0,
        }
    }

    pub fn load_percentage(&self) -> f64 {
        let load = self.current_load.load(Ordering::Relaxed);
        (load as f64 / self.capacity as f64) * 100.0
    }

    pub fn available_capacity(&self) -> u64 {
        let load = self.current_load.load(Ordering::Relaxed);
        self.capacity.saturating_sub(load)
    }
}

/// Стратегия балансировки нагрузки
#[derive(Debug, Clone, Copy)]
pub enum LoadBalancingStrategy {
    /// Циклический перебор
    RoundRobin,
    /// Наименьшее количество соединений
    LeastConnections,
    /// Взвешенный по возможностям
    WeightedCapacity,
    /// На основе задержки
    LowestLatency,
    /// Привязка по хешу (для консистентности)
    HashBased,
}

/// Балансировщик нагрузки для торговых ордеров
pub struct OrderLoadBalancer {
    workers: Vec<WorkerNode>,
    strategy: LoadBalancingStrategy,
    round_robin_counter: AtomicUsize,
    orders_routed: AtomicU64,
}

impl OrderLoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        OrderLoadBalancer {
            workers: Vec::new(),
            strategy,
            round_robin_counter: AtomicUsize::new(0),
            orders_routed: AtomicU64::new(0),
        }
    }

    pub fn add_worker(&mut self, worker: WorkerNode) {
        println!("Добавлен рабочий узел: {} ({})", worker.id, worker.address);
        self.workers.push(worker);
    }

    pub fn remove_worker(&mut self, worker_id: &str) {
        self.workers.retain(|w| w.id != worker_id);
        println!("Удалён рабочий узел: {}", worker_id);
    }

    /// Выбрать рабочий узел для ордера
    pub fn route_order(&self, order_id: &str, symbol: &str) -> Option<&WorkerNode> {
        let healthy_workers: Vec<_> = self.workers.iter()
            .filter(|w| w.is_healthy && w.available_capacity() > 0)
            .collect();

        if healthy_workers.is_empty() {
            return None;
        }

        self.orders_routed.fetch_add(1, Ordering::Relaxed);

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let idx = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
                healthy_workers[idx % healthy_workers.len()]
            }
            LoadBalancingStrategy::LeastConnections => {
                healthy_workers.iter()
                    .min_by_key(|w| w.current_load.load(Ordering::Relaxed))
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::WeightedCapacity => {
                healthy_workers.iter()
                    .max_by(|a, b| {
                        a.available_capacity().cmp(&b.available_capacity())
                    })
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::LowestLatency => {
                healthy_workers.iter()
                    .min_by(|a, b| {
                        a.latency_ms.partial_cmp(&b.latency_ms).unwrap()
                    })
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::HashBased => {
                // Консистентное хеширование по символу для кеш-локальности
                let hash = symbol.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
                healthy_workers[hash % healthy_workers.len()]
            }
        };

        // Увеличиваем нагрузку выбранного узла
        selected.current_load.fetch_add(1, Ordering::Relaxed);

        Some(selected)
    }

    /// Освободить ресурс на узле после обработки ордера
    pub fn release_order(&self, worker_id: &str) {
        if let Some(worker) = self.workers.iter().find(|w| w.id == worker_id) {
            worker.current_load.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Получить статистику балансировщика
    pub fn stats(&self) -> LoadBalancerStats {
        let total_capacity: u64 = self.workers.iter().map(|w| w.capacity).sum();
        let total_load: u64 = self.workers.iter()
            .map(|w| w.current_load.load(Ordering::Relaxed))
            .sum();
        let healthy_count = self.workers.iter().filter(|w| w.is_healthy).count();

        LoadBalancerStats {
            total_workers: self.workers.len(),
            healthy_workers: healthy_count,
            total_capacity,
            total_load,
            utilization_pct: if total_capacity > 0 {
                (total_load as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
            orders_routed: self.orders_routed.load(Ordering::Relaxed),
        }
    }

    pub fn print_status(&self) {
        println!("\n=== Статус балансировщика нагрузки ===");
        println!("Стратегия: {:?}", self.strategy);

        for worker in &self.workers {
            let status = if worker.is_healthy { "✓" } else { "✗" };
            println!("  {} {} - Нагрузка: {:.1}% ({}/{}), Задержка: {:.1}мс",
                status,
                worker.id,
                worker.load_percentage(),
                worker.current_load.load(Ordering::Relaxed),
                worker.capacity,
                worker.latency_ms);
        }

        let stats = self.stats();
        println!("\nОбщая утилизация: {:.1}%", stats.utilization_pct);
        println!("Всего направлено ордеров: {}", stats.orders_routed);
    }
}

#[derive(Debug)]
pub struct LoadBalancerStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_capacity: u64,
    pub total_load: u64,
    pub utilization_pct: f64,
    pub orders_routed: u64,
}

fn main() {
    println!("=== Балансировка нагрузки для ордеров ===\n");

    let mut balancer = OrderLoadBalancer::new(LoadBalancingStrategy::LeastConnections);

    // Добавление рабочих узлов
    balancer.add_worker(WorkerNode {
        id: "worker-1".to_string(),
        address: "10.0.0.1:8080".to_string(),
        capacity: 1000,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 1.5,
    });

    balancer.add_worker(WorkerNode {
        id: "worker-2".to_string(),
        address: "10.0.0.2:8080".to_string(),
        capacity: 1500,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 2.0,
    });

    balancer.add_worker(WorkerNode {
        id: "worker-3".to_string(),
        address: "10.0.0.3:8080".to_string(),
        capacity: 800,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 1.2,
    });

    // Симуляция маршрутизации ордеров
    println!("\nМаршрутизация ордеров...");
    for i in 0..20 {
        let order_id = format!("order-{}", i);
        let symbol = if i % 2 == 0 { "BTCUSDT" } else { "ETHUSDT" };

        if let Some(worker) = balancer.route_order(&order_id, symbol) {
            println!("Ордер {} ({}) -> {}", order_id, symbol, worker.id);
        }
    }

    balancer.print_status();

    // Освобождение некоторых ордеров
    println!("\nОсвобождение ордеров...");
    for i in 0..10 {
        let worker_id = format!("worker-{}", (i % 3) + 1);
        balancer.release_order(&worker_id);
    }

    balancer.print_status();
}
```

## Партиционирование данных

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Стратегия партиционирования
#[derive(Debug, Clone, Copy)]
pub enum PartitionStrategy {
    /// По хешу ключа
    Hash,
    /// По диапазону значений
    Range,
    /// По списку значений
    List,
    /// Консистентное хеширование
    ConsistentHash,
}

/// Партиция данных
#[derive(Debug)]
pub struct Partition<V> {
    pub id: u32,
    pub data: RwLock<HashMap<String, V>>,
    pub item_count: RwLock<usize>,
}

impl<V: Clone> Partition<V> {
    pub fn new(id: u32) -> Self {
        Partition {
            id,
            data: RwLock::new(HashMap::new()),
            item_count: RwLock::new(0),
        }
    }

    pub fn insert(&self, key: String, value: V) {
        let mut data = self.data.write().unwrap();
        let is_new = !data.contains_key(&key);
        data.insert(key, value);
        if is_new {
            *self.item_count.write().unwrap() += 1;
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        self.data.read().unwrap().get(key).cloned()
    }

    pub fn count(&self) -> usize {
        *self.item_count.read().unwrap()
    }
}

/// Менеджер партиционирования торговых данных
pub struct TradingDataPartitioner<V> {
    partitions: Vec<Arc<Partition<V>>>,
    strategy: PartitionStrategy,
    symbol_to_partition: RwLock<HashMap<String, u32>>,
}

impl<V: Clone> TradingDataPartitioner<V> {
    pub fn new(partition_count: u32, strategy: PartitionStrategy) -> Self {
        let partitions = (0..partition_count)
            .map(|id| Arc::new(Partition::new(id)))
            .collect();

        TradingDataPartitioner {
            partitions,
            strategy,
            symbol_to_partition: RwLock::new(HashMap::new()),
        }
    }

    /// Вычислить хеш для ключа
    fn hash_key(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Определить партицию для ключа
    pub fn get_partition(&self, key: &str) -> Arc<Partition<V>> {
        let partition_id = match self.strategy {
            PartitionStrategy::Hash => {
                let hash = Self::hash_key(key);
                (hash % self.partitions.len() as u64) as u32
            }
            PartitionStrategy::Range => {
                // Партиционирование по первой букве символа
                let first_char = key.chars().next().unwrap_or('A');
                let range = (first_char as u32 - 'A' as u32) % self.partitions.len() as u32;
                range
            }
            PartitionStrategy::List => {
                // Используем кеш для привязки символов к партициям
                let cache = self.symbol_to_partition.read().unwrap();
                if let Some(&partition_id) = cache.get(key) {
                    partition_id
                } else {
                    drop(cache);
                    // Назначаем по хешу, если не в списке
                    let hash = Self::hash_key(key);
                    let partition_id = (hash % self.partitions.len() as u64) as u32;
                    self.symbol_to_partition.write().unwrap().insert(key.to_string(), partition_id);
                    partition_id
                }
            }
            PartitionStrategy::ConsistentHash => {
                // Консистентное хеширование с виртуальными узлами
                let hash = Self::hash_key(key);
                // Упрощённая версия: используем модуль
                (hash % self.partitions.len() as u64) as u32
            }
        };

        Arc::clone(&self.partitions[partition_id as usize])
    }

    /// Вставить данные
    pub fn insert(&self, key: String, value: V) {
        let partition = self.get_partition(&key);
        partition.insert(key, value);
    }

    /// Получить данные
    pub fn get(&self, key: &str) -> Option<V> {
        let partition = self.get_partition(key);
        partition.get(key)
    }

    /// Статистика по партициям
    pub fn stats(&self) -> PartitionStats {
        let counts: Vec<usize> = self.partitions.iter()
            .map(|p| p.count())
            .collect();

        let total: usize = counts.iter().sum();
        let max = *counts.iter().max().unwrap_or(&0);
        let min = *counts.iter().min().unwrap_or(&0);
        let avg = if counts.is_empty() { 0.0 } else { total as f64 / counts.len() as f64 };

        // Коэффициент неравномерности (стандартное отклонение)
        let variance = counts.iter()
            .map(|&c| (c as f64 - avg).powi(2))
            .sum::<f64>() / counts.len() as f64;
        let std_dev = variance.sqrt();

        PartitionStats {
            partition_count: self.partitions.len(),
            total_items: total,
            items_per_partition: counts,
            max_items: max,
            min_items: min,
            avg_items: avg,
            imbalance_ratio: if min > 0 { max as f64 / min as f64 } else { 0.0 },
            std_deviation: std_dev,
        }
    }

    pub fn print_stats(&self) {
        let stats = self.stats();

        println!("\n=== Статистика партиционирования ===");
        println!("Стратегия: {:?}", self.strategy);
        println!("Количество партиций: {}", stats.partition_count);
        println!("Всего элементов: {}", stats.total_items);
        println!("Среднее на партицию: {:.1}", stats.avg_items);
        println!("Мин/Макс: {} / {}", stats.min_items, stats.max_items);
        println!("Коэффициент неравномерности: {:.2}", stats.imbalance_ratio);
        println!("Стандартное отклонение: {:.2}", stats.std_deviation);

        println!("\nРаспределение:");
        for (i, count) in stats.items_per_partition.iter().enumerate() {
            let bar_len = (*count as f64 / stats.max_items.max(1) as f64 * 20.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("  Партиция {}: {:>5} {}", i, count, bar);
        }
    }
}

#[derive(Debug)]
pub struct PartitionStats {
    pub partition_count: usize,
    pub total_items: usize,
    pub items_per_partition: Vec<usize>,
    pub max_items: usize,
    pub min_items: usize,
    pub avg_items: f64,
    pub imbalance_ratio: f64,
    pub std_deviation: f64,
}

/// Торговая сделка для примера
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

fn main() {
    println!("=== Партиционирование торговых данных ===\n");

    // Создание партиционера с 4 партициями
    let partitioner: TradingDataPartitioner<Trade> =
        TradingDataPartitioner::new(4, PartitionStrategy::Hash);

    // Симуляция вставки сделок
    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "XRPUSDT",
                   "SOLUSDT", "DOTUSDT", "MATICUSDT", "LINKUSDT", "AVAXUSDT"];

    println!("Вставка сделок...");
    for i in 0..1000 {
        let symbol = symbols[i % symbols.len()];
        let trade = Trade {
            id: format!("trade-{}", i),
            symbol: symbol.to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.1 + (i as f64 * 0.001),
            timestamp: 1700000000 + i as u64,
        };

        partitioner.insert(trade.id.clone(), trade);
    }

    partitioner.print_stats();

    // Тест получения данных
    println!("\nТест получения данных:");
    for i in [0, 100, 500, 999] {
        let key = format!("trade-{}", i);
        if let Some(trade) = partitioner.get(&key) {
            println!("  {} -> {} @ ${:.2}", key, trade.symbol, trade.price);
        }
    }

    // Сравнение стратегий
    println!("\n=== Сравнение стратегий партиционирования ===");

    for strategy in [PartitionStrategy::Hash, PartitionStrategy::Range,
                     PartitionStrategy::ConsistentHash] {
        let part: TradingDataPartitioner<Trade> =
            TradingDataPartitioner::new(4, strategy);

        for i in 0..1000 {
            let symbol = symbols[i % symbols.len()];
            let trade = Trade {
                id: format!("trade-{}", i),
                symbol: symbol.to_string(),
                price: 50000.0,
                quantity: 0.1,
                timestamp: 1700000000,
            };
            part.insert(trade.id.clone(), trade);
        }

        let stats = part.stats();
        println!("\n{:?}:", strategy);
        println!("  Неравномерность: {:.2}, Станд. откл.: {:.2}",
            stats.imbalance_ratio, stats.std_deviation);
    }
}
```

## Распределённая обработка рыночных данных

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Тик рыночных данных
#[derive(Debug, Clone)]
pub struct MarketTick {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub timestamp: u64,
}

/// Рабочий узел обработки данных
pub struct DataProcessingNode {
    pub id: String,
    pub assigned_symbols: Vec<String>,
    pub ticks_processed: AtomicU64,
    pub last_tick_time: AtomicU64,
    processor: Box<dyn Fn(&MarketTick) -> ProcessingResult + Send + Sync>,
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub symbol: String,
    pub signal: Option<TradeSignal>,
    pub latency_us: u64,
}

#[derive(Debug, Clone)]
pub enum TradeSignal {
    Buy { price: f64, size: f64 },
    Sell { price: f64, size: f64 },
}

impl DataProcessingNode {
    pub fn new<F>(id: &str, processor: F) -> Self
    where
        F: Fn(&MarketTick) -> ProcessingResult + Send + Sync + 'static,
    {
        DataProcessingNode {
            id: id.to_string(),
            assigned_symbols: Vec::new(),
            ticks_processed: AtomicU64::new(0),
            last_tick_time: AtomicU64::new(0),
            processor: Box::new(processor),
        }
    }

    pub fn assign_symbol(&mut self, symbol: String) {
        if !self.assigned_symbols.contains(&symbol) {
            self.assigned_symbols.push(symbol);
        }
    }

    pub fn process_tick(&self, tick: &MarketTick) -> ProcessingResult {
        self.ticks_processed.fetch_add(1, Ordering::Relaxed);
        self.last_tick_time.store(tick.timestamp, Ordering::Relaxed);
        (self.processor)(tick)
    }

    pub fn stats(&self) -> NodeStats {
        NodeStats {
            id: self.id.clone(),
            symbols_count: self.assigned_symbols.len(),
            ticks_processed: self.ticks_processed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub struct NodeStats {
    pub id: String,
    pub symbols_count: usize,
    pub ticks_processed: u64,
}

/// Координатор распределённой обработки
pub struct DistributedProcessor {
    nodes: Vec<Arc<DataProcessingNode>>,
    symbol_to_node: HashMap<String, usize>,
    total_ticks: AtomicU64,
    start_time: Instant,
}

impl DistributedProcessor {
    pub fn new() -> Self {
        DistributedProcessor {
            nodes: Vec::new(),
            symbol_to_node: HashMap::new(),
            total_ticks: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn add_node(&mut self, node: DataProcessingNode) {
        let node_idx = self.nodes.len();
        let node = Arc::new(node);

        // Регистрируем символы узла
        for symbol in &node.assigned_symbols {
            self.symbol_to_node.insert(symbol.clone(), node_idx);
        }

        self.nodes.push(node);
    }

    /// Назначить символы узлам равномерно
    pub fn distribute_symbols(&mut self, symbols: &[String]) {
        if self.nodes.is_empty() {
            return;
        }

        for (i, symbol) in symbols.iter().enumerate() {
            let node_idx = i % self.nodes.len();
            self.symbol_to_node.insert(symbol.clone(), node_idx);

            // Нужен unsafe для модификации Arc
            // В реальном коде используйте RwLock
            println!("Символ {} назначен узлу {}", symbol, self.nodes[node_idx].id);
        }
    }

    /// Обработать тик на соответствующем узле
    pub fn process_tick(&self, tick: &MarketTick) -> Option<ProcessingResult> {
        self.total_ticks.fetch_add(1, Ordering::Relaxed);

        if let Some(&node_idx) = self.symbol_to_node.get(&tick.symbol) {
            let node = &self.nodes[node_idx];
            Some(node.process_tick(tick))
        } else {
            // Символ не назначен, используем первый доступный узел
            if !self.nodes.is_empty() {
                Some(self.nodes[0].process_tick(tick))
            } else {
                None
            }
        }
    }

    /// Получить статистику всех узлов
    pub fn get_stats(&self) -> ClusterStats {
        let node_stats: Vec<NodeStats> = self.nodes.iter()
            .map(|n| n.stats())
            .collect();

        let total_ticks = self.total_ticks.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        let throughput = if elapsed.as_secs_f64() > 0.0 {
            total_ticks as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        ClusterStats {
            node_count: self.nodes.len(),
            total_symbols: self.symbol_to_node.len(),
            total_ticks_processed: total_ticks,
            throughput_per_second: throughput,
            uptime: elapsed,
            nodes: node_stats,
        }
    }

    pub fn print_stats(&self) {
        let stats = self.get_stats();

        println!("\n=== Статистика распределённого процессора ===");
        println!("Узлов: {}", stats.node_count);
        println!("Символов: {}", stats.total_symbols);
        println!("Всего тиков: {}", stats.total_ticks_processed);
        println!("Пропускная способность: {:.0} тиков/сек", stats.throughput_per_second);
        println!("Время работы: {:.2}с", stats.uptime.as_secs_f64());

        println!("\nСтатистика по узлам:");
        for node in &stats.nodes {
            println!("  {} - Символов: {}, Тиков: {}",
                node.id, node.symbols_count, node.ticks_processed);
        }
    }
}

#[derive(Debug)]
pub struct ClusterStats {
    pub node_count: usize,
    pub total_symbols: usize,
    pub total_ticks_processed: u64,
    pub throughput_per_second: f64,
    pub uptime: Duration,
    pub nodes: Vec<NodeStats>,
}

fn main() {
    println!("=== Распределённая обработка рыночных данных ===\n");

    let mut processor = DistributedProcessor::new();

    // Создание узлов обработки с разной логикой
    let node1 = DataProcessingNode::new("crypto-processor", |tick| {
        let start = Instant::now();

        // Простая логика моментума
        let spread = tick.ask - tick.bid;
        let signal = if spread < 0.0001 * tick.bid {
            Some(TradeSignal::Buy {
                price: tick.ask,
                size: tick.bid_size.min(1.0)
            })
        } else {
            None
        };

        ProcessingResult {
            symbol: tick.symbol.clone(),
            signal,
            latency_us: start.elapsed().as_micros() as u64,
        }
    });

    let node2 = DataProcessingNode::new("forex-processor", |tick| {
        let start = Instant::now();

        // Логика для форекса
        let mid_price = (tick.bid + tick.ask) / 2.0;
        let signal = if tick.bid_size > tick.ask_size * 1.5 {
            Some(TradeSignal::Buy {
                price: tick.ask,
                size: 10000.0
            })
        } else if tick.ask_size > tick.bid_size * 1.5 {
            Some(TradeSignal::Sell {
                price: tick.bid,
                size: 10000.0
            })
        } else {
            None
        };

        ProcessingResult {
            symbol: tick.symbol.clone(),
            signal,
            latency_us: start.elapsed().as_micros() as u64,
        }
    });

    processor.add_node(node1);
    processor.add_node(node2);

    // Распределение символов
    let symbols: Vec<String> = vec![
        "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT",
        "EURUSD", "GBPUSD", "USDJPY", "AUDUSD"
    ].into_iter().map(String::from).collect();

    processor.distribute_symbols(&symbols);

    // Симуляция обработки тиков
    println!("\nОбработка тиков...");
    for i in 0..1000 {
        let symbol = &symbols[i % symbols.len()];
        let tick = MarketTick {
            symbol: symbol.clone(),
            bid: 50000.0 + (i as f64 * 0.01),
            ask: 50001.0 + (i as f64 * 0.01),
            bid_size: 1.5 + (i as f64 % 10.0) * 0.1,
            ask_size: 1.2 + (i as f64 % 8.0) * 0.1,
            timestamp: 1700000000 + i as u64,
        };

        if let Some(result) = processor.process_tick(&tick) {
            if result.signal.is_some() && i < 10 {
                println!("Тик {} -> {:?}", tick.symbol, result.signal);
            }
        }
    }

    processor.print_stats();
}
```

## Синхронизация состояния в распределённой системе

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Версия состояния для контроля согласованности
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateVersion(pub u64);

impl StateVersion {
    pub fn new() -> Self {
        StateVersion(0)
    }

    pub fn increment(&self) -> Self {
        StateVersion(self.0 + 1)
    }
}

/// Состояние торгового портфеля
#[derive(Debug, Clone)]
pub struct PortfolioState {
    pub positions: HashMap<String, Position>,
    pub cash_balance: f64,
    pub version: StateVersion,
    pub last_updated: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub unrealized_pnl: f64,
}

/// Изменение состояния
#[derive(Debug, Clone)]
pub struct StateChange {
    pub change_id: u64,
    pub version: StateVersion,
    pub operation: StateOperation,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum StateOperation {
    UpdatePosition { symbol: String, quantity: f64, price: f64 },
    UpdateCash { amount: f64 },
    ClosePosition { symbol: String, price: f64 },
}

/// Реплика состояния на узле
pub struct StateReplica {
    pub node_id: String,
    pub state: RwLock<PortfolioState>,
    pub pending_changes: RwLock<Vec<StateChange>>,
    pub applied_version: AtomicU64,
}

impl StateReplica {
    pub fn new(node_id: &str) -> Self {
        StateReplica {
            node_id: node_id.to_string(),
            state: RwLock::new(PortfolioState {
                positions: HashMap::new(),
                cash_balance: 100_000.0,
                version: StateVersion::new(),
                last_updated: 0,
            }),
            pending_changes: RwLock::new(Vec::new()),
            applied_version: AtomicU64::new(0),
        }
    }

    /// Применить изменение к локальному состоянию
    pub fn apply_change(&self, change: &StateChange) -> Result<(), &'static str> {
        let current_version = self.applied_version.load(Ordering::SeqCst);

        // Проверка порядка версий
        if change.version.0 != current_version + 1 {
            return Err("Пропущена версия, требуется синхронизация");
        }

        let mut state = self.state.write().unwrap();

        match &change.operation {
            StateOperation::UpdatePosition { symbol, quantity, price } => {
                let position = state.positions.entry(symbol.clone()).or_insert(Position {
                    symbol: symbol.clone(),
                    quantity: 0.0,
                    avg_price: 0.0,
                    unrealized_pnl: 0.0,
                });

                // Пересчёт средней цены
                let total_value = position.quantity * position.avg_price + quantity * price;
                let new_quantity = position.quantity + quantity;

                if new_quantity.abs() > 0.0001 {
                    position.avg_price = total_value / new_quantity;
                    position.quantity = new_quantity;
                } else {
                    state.positions.remove(symbol);
                }
            }
            StateOperation::UpdateCash { amount } => {
                state.cash_balance += amount;
            }
            StateOperation::ClosePosition { symbol, price } => {
                if let Some(position) = state.positions.remove(symbol) {
                    let pnl = (price - position.avg_price) * position.quantity;
                    state.cash_balance += position.quantity * price;
                    println!("  Позиция {} закрыта, P&L: ${:.2}", symbol, pnl);
                }
            }
        }

        state.version = change.version;
        state.last_updated = change.timestamp;
        self.applied_version.store(change.version.0, Ordering::SeqCst);

        Ok(())
    }

    /// Получить текущую версию
    pub fn current_version(&self) -> StateVersion {
        StateVersion(self.applied_version.load(Ordering::SeqCst))
    }

    /// Получить снимок состояния
    pub fn snapshot(&self) -> PortfolioState {
        self.state.read().unwrap().clone()
    }
}

/// Координатор синхронизации состояния
pub struct StateCoordinator {
    replicas: Vec<Arc<StateReplica>>,
    change_log: RwLock<Vec<StateChange>>,
    next_change_id: AtomicU64,
    next_version: AtomicU64,
}

impl StateCoordinator {
    pub fn new() -> Self {
        StateCoordinator {
            replicas: Vec::new(),
            change_log: RwLock::new(Vec::new()),
            next_change_id: AtomicU64::new(1),
            next_version: AtomicU64::new(1),
        }
    }

    pub fn add_replica(&mut self, replica: Arc<StateReplica>) {
        println!("Добавлена реплика: {}", replica.node_id);
        self.replicas.push(replica);
    }

    /// Предложить изменение состояния
    pub fn propose_change(&self, operation: StateOperation) -> Result<StateVersion, &'static str> {
        let change_id = self.next_change_id.fetch_add(1, Ordering::SeqCst);
        let version = StateVersion(self.next_version.fetch_add(1, Ordering::SeqCst));

        let change = StateChange {
            change_id,
            version,
            operation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Применяем ко всем репликам
        let mut success_count = 0;
        for replica in &self.replicas {
            if replica.apply_change(&change).is_ok() {
                success_count += 1;
            }
        }

        // Требуем кворум (большинство)
        let quorum = (self.replicas.len() / 2) + 1;
        if success_count >= quorum {
            self.change_log.write().unwrap().push(change);
            Ok(version)
        } else {
            Err("Не удалось достичь кворума")
        }
    }

    /// Синхронизировать отставшую реплику
    pub fn sync_replica(&self, replica: &StateReplica) -> Result<u64, &'static str> {
        let current_version = replica.current_version();
        let changes = self.change_log.read().unwrap();

        let mut applied = 0;
        for change in changes.iter() {
            if change.version.0 > current_version.0 {
                replica.apply_change(change)?;
                applied += 1;
            }
        }

        Ok(applied)
    }

    /// Проверить согласованность всех реплик
    pub fn check_consistency(&self) -> ConsistencyReport {
        let versions: Vec<(String, StateVersion)> = self.replicas.iter()
            .map(|r| (r.node_id.clone(), r.current_version()))
            .collect();

        let max_version = versions.iter().map(|(_, v)| v.0).max().unwrap_or(0);
        let min_version = versions.iter().map(|(_, v)| v.0).min().unwrap_or(0);

        ConsistencyReport {
            replica_count: self.replicas.len(),
            versions: versions.clone(),
            is_consistent: max_version == min_version,
            version_lag: max_version - min_version,
        }
    }

    pub fn print_status(&self) {
        println!("\n=== Статус синхронизации состояния ===");

        let report = self.check_consistency();
        println!("Реплик: {}", report.replica_count);
        println!("Согласованность: {}", if report.is_consistent { "✓" } else { "✗" });
        println!("Отставание версий: {}", report.version_lag);

        println!("\nВерсии по репликам:");
        for (node_id, version) in &report.versions {
            println!("  {} -> v{}", node_id, version.0);
        }

        // Показать состояние первой реплики
        if let Some(replica) = self.replicas.first() {
            let state = replica.snapshot();
            println!("\nТекущее состояние портфеля:");
            println!("  Кеш: ${:.2}", state.cash_balance);
            println!("  Позиции:");
            for (symbol, pos) in &state.positions {
                println!("    {} - {} @ ${:.2}", symbol, pos.quantity, pos.avg_price);
            }
        }
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub replica_count: usize,
    pub versions: Vec<(String, StateVersion)>,
    pub is_consistent: bool,
    pub version_lag: u64,
}

fn main() {
    println!("=== Синхронизация состояния ===\n");

    let mut coordinator = StateCoordinator::new();

    // Создание реплик
    let replica1 = Arc::new(StateReplica::new("node-us-east"));
    let replica2 = Arc::new(StateReplica::new("node-eu-west"));
    let replica3 = Arc::new(StateReplica::new("node-asia"));

    coordinator.add_replica(Arc::clone(&replica1));
    coordinator.add_replica(Arc::clone(&replica2));
    coordinator.add_replica(Arc::clone(&replica3));

    // Симуляция торговых операций
    println!("\nВыполнение торговых операций...");

    // Покупка BTC
    coordinator.propose_change(StateOperation::UpdateCash { amount: -50000.0 }).unwrap();
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.0,
        price: 50000.0,
    }).unwrap();

    // Покупка ETH
    coordinator.propose_change(StateOperation::UpdateCash { amount: -3000.0 }).unwrap();
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "ETHUSDT".to_string(),
        quantity: 1.0,
        price: 3000.0,
    }).unwrap();

    // Частичная продажа BTC
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "BTCUSDT".to_string(),
        quantity: -0.5,
        price: 52000.0,
    }).unwrap();
    coordinator.propose_change(StateOperation::UpdateCash { amount: 26000.0 }).unwrap();

    coordinator.print_status();

    // Проверка согласованности
    let report = coordinator.check_consistency();
    println!("\nПроверка согласованности:");
    if report.is_consistent {
        println!("✓ Все реплики согласованы на версии v{}",
            report.versions.first().map(|(_, v)| v.0).unwrap_or(0));
    } else {
        println!("✗ Обнаружено расхождение версий!");
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Балансировка нагрузки** | Распределение запросов между рабочими узлами |
| **Партиционирование** | Разделение данных на части для параллельной обработки |
| **Распределённая обработка** | Обработка данных на нескольких узлах одновременно |
| **Синхронизация состояния** | Поддержание согласованности данных между репликами |
| **Кворум** | Минимальное количество узлов для подтверждения операции |
| **Версионирование** | Отслеживание изменений для обеспечения порядка |
| **Консистентное хеширование** | Распределение данных с минимальным перераспределением |

## Практические задания

1. **Балансировщик с приоритетами**: Реализуй систему, которая:
   - Назначает приоритеты разным типам ордеров
   - Направляет VIP-клиентов на выделенные узлы
   - Обрабатывает экстренные ордера без очереди
   - Балансирует нагрузку с учётом типа инструмента

2. **Партиционер по времени**: Создай систему партиционирования:
   - Разделяет данные по временным окнам (секунды, минуты, часы)
   - Поддерживает горячие и холодные партиции
   - Архивирует старые данные автоматически
   - Оптимизирует запросы по временному диапазону

3. **Отказоустойчивый кластер**: Построй кластер обработки:
   - Обнаруживает отказы узлов через heartbeat
   - Автоматически перераспределяет нагрузку
   - Восстанавливает узлы после сбоя
   - Логирует все события для аудита

4. **Гео-распределённая система**: Реализуй систему:
   - Реплицирует данные между регионами
   - Направляет запросы к ближайшему дата-центру
   - Обрабатывает сетевые разрывы между регионами
   - Разрешает конфликты при расхождении данных

## Домашнее задание

1. **Полный торговый кластер**: Построй систему, которая:
   - Масштабируется горизонтально при росте нагрузки
   - Поддерживает добавление узлов без простоя
   - Балансирует нагрузку между биржами
   - Обеспечивает согласованность портфеля
   - Мониторит здоровье всех компонентов
   - Автоматически восстанавливается после сбоев

2. **Система потоковой обработки**: Создай платформу:
   - Обрабатывает миллионы тиков в секунду
   - Распределяет вычисления по узлам
   - Агрегирует результаты в реальном времени
   - Поддерживает backpressure при перегрузке
   - Масштабируется по запросу

3. **Распределённый ордер-бук**: Реализуй:
   - Партиционирование ордер-бука по символам
   - Репликацию для отказоустойчивости
   - Быстрый мэтчинг ордеров на каждом узле
   - Синхронизацию между партициями
   - Атомарные кросс-партиционные операции

4. **Оркестратор авто-масштабирования**: Спроектируй систему:
   - Мониторит метрики нагрузки кластера
   - Принимает решения о масштабировании
   - Добавляет/удаляет узлы автоматически
   - Прогнозирует нагрузку на основе исторических данных
   - Оптимизирует затраты на инфраструктуру

## Навигация

[← Предыдущий день](../357-disaster-recovery/ru.md) | [Следующий день →](../359-kubernetes-orchestration/ru.md)
