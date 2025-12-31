# День 329: Батчинг операций

## Аналогия из трейдинга

Представь, что ты управляешь торговым ботом, который отслеживает 100 криптовалютных пар. Каждую секунду бот получает обновления цен и должен:
- Рассчитать новую стоимость портфеля
- Обновить trailing stop-loss для каждой позиции
- Проверить условия входа/выхода
- Записать данные в лог

Без батчинга каждое обновление цены вызывает 4 операции. При 100 парах и 10 обновлениях в секунду это 4000 операций в секунду!

**Батчинг операций** — это как система расчётов на бирже:
- Вместо мгновенного исполнения каждого ордера, биржа накапливает ордера
- Раз в определённый интервал (например, каждые 100мс) происходит "матчинг"
- Все накопленные ордера обрабатываются одним пакетом

В результате:
- Снижается нагрузка на систему
- Уменьшается количество блокировок
- Оптимизируется использование сети и диска

## Когда использовать батчинг?

| Сценарий | Пример из трейдинга | Выгода |
|----------|---------------------|--------|
| **Множество мелких записей** | Логирование каждого тика | Снижение I/O операций |
| **Сетевые запросы** | Отправка ордеров на биржу | Уменьшение latency и overhead |
| **Обновление базы данных** | Сохранение истории сделок | Меньше транзакций |
| **Расчёты по множеству активов** | Пересчёт портфеля | Эффективное использование CPU |

## Простой батчер для торговых операций

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Торговая операция для батчинга
#[derive(Debug, Clone)]
enum TradeOperation {
    UpdatePrice { symbol: String, price: f64 },
    PlaceOrder { symbol: String, side: String, quantity: f64 },
    CancelOrder { order_id: u64 },
    UpdateStopLoss { symbol: String, new_price: f64 },
}

/// Батчер операций с настраиваемыми условиями сброса
struct OperationBatcher {
    buffer: VecDeque<TradeOperation>,
    max_size: usize,
    max_wait: Duration,
    last_flush: Instant,
    total_batches: u64,
    total_operations: u64,
}

impl OperationBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        OperationBatcher {
            buffer: VecDeque::new(),
            max_size,
            max_wait: Duration::from_millis(max_wait_ms),
            last_flush: Instant::now(),
            total_batches: 0,
            total_operations: 0,
        }
    }

    /// Добавить операцию в буфер
    fn add(&mut self, op: TradeOperation) -> Option<Vec<TradeOperation>> {
        self.buffer.push_back(op);

        // Проверяем условия сброса
        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Проверить, нужно ли сбросить буфер
    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.max_size
            || self.last_flush.elapsed() >= self.max_wait
    }

    /// Сбросить буфер и вернуть накопленные операции
    fn flush(&mut self) -> Vec<TradeOperation> {
        let ops: Vec<_> = self.buffer.drain(..).collect();
        self.last_flush = Instant::now();
        self.total_batches += 1;
        self.total_operations += ops.len() as u64;
        ops
    }

    /// Принудительный сброс (например, при завершении работы)
    fn force_flush(&mut self) -> Vec<TradeOperation> {
        if !self.buffer.is_empty() {
            self.flush()
        } else {
            Vec::new()
        }
    }

    /// Статистика
    fn stats(&self) -> (u64, u64, f64) {
        let avg = if self.total_batches > 0 {
            self.total_operations as f64 / self.total_batches as f64
        } else {
            0.0
        };
        (self.total_batches, self.total_operations, avg)
    }
}

fn process_batch(batch: &[TradeOperation]) {
    println!("  Обработка пакета из {} операций:", batch.len());
    for op in batch {
        match op {
            TradeOperation::UpdatePrice { symbol, price } => {
                println!("    - Цена {}: ${:.2}", symbol, price);
            }
            TradeOperation::PlaceOrder { symbol, side, quantity } => {
                println!("    - Ордер: {} {} {:.4}", side, quantity, symbol);
            }
            TradeOperation::CancelOrder { order_id } => {
                println!("    - Отмена ордера #{}", order_id);
            }
            TradeOperation::UpdateStopLoss { symbol, new_price } => {
                println!("    - Stop-loss {}: ${:.2}", symbol, new_price);
            }
        }
    }
}

fn main() {
    let mut batcher = OperationBatcher::new(5, 100); // max 5 ops или 100ms

    println!("=== Батчинг торговых операций ===\n");

    // Симулируем поток операций
    let operations = vec![
        TradeOperation::UpdatePrice { symbol: "BTC".to_string(), price: 42500.0 },
        TradeOperation::UpdatePrice { symbol: "ETH".to_string(), price: 2500.0 },
        TradeOperation::UpdateStopLoss { symbol: "BTC".to_string(), new_price: 42000.0 },
        TradeOperation::PlaceOrder { symbol: "SOL".to_string(), side: "BUY".to_string(), quantity: 10.0 },
        TradeOperation::UpdatePrice { symbol: "SOL".to_string(), price: 100.0 },
        TradeOperation::CancelOrder { order_id: 12345 },
        TradeOperation::UpdatePrice { symbol: "BTC".to_string(), price: 42550.0 },
    ];

    for op in operations {
        if let Some(batch) = batcher.add(op) {
            process_batch(&batch);
        }
    }

    // Сбрасываем оставшиеся операции
    let remaining = batcher.force_flush();
    if !remaining.is_empty() {
        println!("\nОставшиеся операции:");
        process_batch(&remaining);
    }

    let (batches, ops, avg) = batcher.stats();
    println!("\n=== Статистика ===");
    println!("Всего пакетов: {}", batches);
    println!("Всего операций: {}", ops);
    println!("Среднее на пакет: {:.1}", avg);
}
```

## Батчинг с приоритетами

В трейдинге некоторые операции важнее других:

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::Instant;

/// Приоритет операции
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    Critical = 3,  // Стоп-лосс, ликвидация
    High = 2,      // Размещение ордера
    Normal = 1,    // Обновление данных
    Low = 0,       // Логирование
}

impl Priority {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Операция с приоритетом
#[derive(Debug, Clone)]
struct PrioritizedOp {
    priority: Priority,
    operation: String,
    created_at: Instant,
    sequence: u64,  // Для стабильной сортировки
}

impl Eq for PrioritizedOp {}

impl PartialEq for PrioritizedOp {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Ord for PrioritizedOp {
    fn cmp(&self, other: &Self) -> Ordering {
        // Сначала по приоритету (выше = лучше)
        // При равном приоритете — по времени (раньше = лучше)
        match self.priority.as_u8().cmp(&other.priority.as_u8()) {
            Ordering::Equal => other.sequence.cmp(&self.sequence),
            other => other,
        }
    }
}

impl PartialOrd for PrioritizedOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Батчер с поддержкой приоритетов
struct PriorityBatcher {
    heap: BinaryHeap<PrioritizedOp>,
    max_size: usize,
    sequence: u64,
}

impl PriorityBatcher {
    fn new(max_size: usize) -> Self {
        PriorityBatcher {
            heap: BinaryHeap::new(),
            max_size,
            sequence: 0,
        }
    }

    fn add(&mut self, priority: Priority, operation: String) {
        self.sequence += 1;
        self.heap.push(PrioritizedOp {
            priority,
            operation,
            created_at: Instant::now(),
            sequence: self.sequence,
        });
    }

    fn flush(&mut self) -> Vec<PrioritizedOp> {
        let count = self.heap.len().min(self.max_size);
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(op) = self.heap.pop() {
                result.push(op);
            }
        }
        result
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}

fn main() {
    let mut batcher = PriorityBatcher::new(5);

    println!("=== Батчинг с приоритетами ===\n");

    // Добавляем операции в случайном порядке
    batcher.add(Priority::Normal, "Обновить цену BTC".to_string());
    batcher.add(Priority::Low, "Записать лог".to_string());
    batcher.add(Priority::Critical, "Исполнить стоп-лосс!".to_string());
    batcher.add(Priority::High, "Разместить ордер".to_string());
    batcher.add(Priority::Normal, "Обновить цену ETH".to_string());
    batcher.add(Priority::Critical, "Предупреждение о ликвидации!".to_string());
    batcher.add(Priority::Low, "Отправить метрики".to_string());

    println!("Всего операций в очереди: {}\n", batcher.len());

    // Обрабатываем пакет (в порядке приоритета)
    let batch = batcher.flush();
    println!("Обработка пакета ({} операций):", batch.len());
    for (i, op) in batch.iter().enumerate() {
        println!("  {}. [{:?}] {}", i + 1, op.priority, op.operation);
    }

    println!("\nОсталось в очереди: {}", batcher.len());
}
```

## Батчинг ордеров для биржи

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

/// Результат отправки пакета ордеров
#[derive(Debug)]
struct BatchResult {
    successful: Vec<u64>,
    failed: Vec<(u64, String)>,
    latency_ms: u64,
}

/// Батчер ордеров с группировкой по символу
struct OrderBatcher {
    orders: HashMap<String, Vec<Order>>,
    max_orders_per_symbol: usize,
    max_total_orders: usize,
    total_orders: usize,
    last_send: Instant,
    send_interval: Duration,
}

impl OrderBatcher {
    fn new(max_per_symbol: usize, max_total: usize, interval_ms: u64) -> Self {
        OrderBatcher {
            orders: HashMap::new(),
            max_orders_per_symbol: max_per_symbol,
            max_total_orders: max_total,
            total_orders: 0,
            last_send: Instant::now(),
            send_interval: Duration::from_millis(interval_ms),
        }
    }

    /// Добавить ордер в пакет
    fn add_order(&mut self, order: Order) -> Option<HashMap<String, Vec<Order>>> {
        let symbol = order.symbol.clone();

        let orders = self.orders.entry(symbol.clone()).or_insert_with(Vec::new);
        orders.push(order);
        self.total_orders += 1;

        // Проверяем условия отправки
        if self.should_send(&symbol) {
            Some(self.flush())
        } else {
            None
        }
    }

    fn should_send(&self, symbol: &str) -> bool {
        let symbol_count = self.orders.get(symbol).map(|v| v.len()).unwrap_or(0);

        symbol_count >= self.max_orders_per_symbol
            || self.total_orders >= self.max_total_orders
            || self.last_send.elapsed() >= self.send_interval
    }

    fn flush(&mut self) -> HashMap<String, Vec<Order>> {
        let result = std::mem::take(&mut self.orders);
        self.total_orders = 0;
        self.last_send = Instant::now();
        result
    }

    /// Получить статистику буфера
    fn buffer_stats(&self) -> (usize, usize) {
        (self.orders.len(), self.total_orders)
    }
}

/// Симуляция отправки пакета на биржу
fn send_order_batch(orders: &HashMap<String, Vec<Order>>) -> BatchResult {
    let start = Instant::now();
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (symbol, order_list) in orders {
        println!("  Отправка {} ордеров для {}:", order_list.len(), symbol);
        for order in order_list {
            // Симуляция: 95% успеха
            if order.id % 20 != 0 {
                successful.push(order.id);
                let price_str = order.price
                    .map(|p| format!(" @ ${:.2}", p))
                    .unwrap_or_else(|| " (market)".to_string());
                println!("    ✓ #{}: {:?} {:.4} {}{}",
                    order.id, order.side, order.quantity, symbol, price_str);
            } else {
                failed.push((order.id, "Insufficient balance".to_string()));
                println!("    ✗ #{}: Ошибка - Insufficient balance", order.id);
            }
        }
    }

    BatchResult {
        successful,
        failed,
        latency_ms: start.elapsed().as_millis() as u64,
    }
}

fn main() {
    let mut batcher = OrderBatcher::new(3, 10, 500);

    println!("=== Батчинг ордеров ===\n");

    // Создаём поток ордеров
    let orders = vec![
        Order { id: 1, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, quantity: 0.1, price: Some(42500.0) },
        Order { id: 2, symbol: "ETHUSDT".to_string(), side: OrderSide::Buy, quantity: 2.0, price: Some(2500.0) },
        Order { id: 3, symbol: "BTCUSDT".to_string(), side: OrderSide::Sell, quantity: 0.05, price: Some(43000.0) },
        Order { id: 4, symbol: "SOLUSDT".to_string(), side: OrderSide::Buy, quantity: 10.0, price: None },
        Order { id: 5, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, quantity: 0.2, price: Some(42000.0) },
        Order { id: 20, symbol: "ETHUSDT".to_string(), side: OrderSide::Sell, quantity: 1.0, price: Some(2600.0) }, // будет ошибка
        Order { id: 7, symbol: "BTCUSDT".to_string(), side: OrderSide::Sell, quantity: 0.15, price: Some(43500.0) },
    ];

    for order in orders {
        println!("Добавлен ордер #{} для {}", order.id, order.symbol);

        if let Some(batch) = batcher.add_order(order) {
            println!("\n--- Отправка пакета на биржу ---");
            let result = send_order_batch(&batch);
            println!("\nРезультат:");
            println!("  Успешно: {} ордеров", result.successful.len());
            println!("  Ошибки: {} ордеров", result.failed.len());
            println!("  Задержка: {}ms\n", result.latency_ms);
        }
    }

    // Проверяем оставшиеся ордера
    let (symbols, total) = batcher.buffer_stats();
    if total > 0 {
        println!("\nОсталось в буфере: {} ордеров по {} символам", total, symbols);
    }
}
```

## Батчинг записи в базу данных

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

/// Статистика записи
#[derive(Debug, Default)]
struct WriteStats {
    total_writes: u64,
    total_records: u64,
    total_time_ms: u64,
}

impl WriteStats {
    fn add(&mut self, records: u64, time_ms: u64) {
        self.total_writes += 1;
        self.total_records += records;
        self.total_time_ms += time_ms;
    }

    fn avg_batch_size(&self) -> f64 {
        if self.total_writes > 0 {
            self.total_records as f64 / self.total_writes as f64
        } else {
            0.0
        }
    }

    fn avg_write_time(&self) -> f64 {
        if self.total_writes > 0 {
            self.total_time_ms as f64 / self.total_writes as f64
        } else {
            0.0
        }
    }
}

/// Батчер для записи сделок в БД
struct TradeBatcher {
    buffer: Vec<Trade>,
    max_size: usize,
    max_wait: Duration,
    last_flush: Instant,
    stats: WriteStats,
}

impl TradeBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        TradeBatcher {
            buffer: Vec::with_capacity(max_size),
            max_size,
            max_wait: Duration::from_millis(max_wait_ms),
            last_flush: Instant::now(),
            stats: WriteStats::default(),
        }
    }

    fn add(&mut self, trade: Trade) {
        self.buffer.push(trade);

        if self.should_flush() {
            self.flush();
        }
    }

    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.max_size
            || (!self.buffer.is_empty() && self.last_flush.elapsed() >= self.max_wait)
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let start = Instant::now();
        let count = self.buffer.len();

        // Симуляция batch insert в БД
        self.simulate_db_write();

        let elapsed = start.elapsed().as_millis() as u64;
        self.stats.add(count as u64, elapsed);

        println!(
            "[DB] Записано {} сделок за {}ms (всего: {})",
            count, elapsed, self.stats.total_records
        );

        self.buffer.clear();
        self.last_flush = Instant::now();
    }

    fn simulate_db_write(&self) {
        // В реальности здесь был бы batch INSERT
        // INSERT INTO trades (id, symbol, price, quantity, timestamp) VALUES
        //   ($1, $2, $3, $4, $5), ($6, $7, $8, $9, $10), ...

        // Симулируем задержку: 5ms базовая + 0.1ms на запись
        let delay_ms = 5 + (self.buffer.len() as u64 / 10);
        std::thread::sleep(Duration::from_millis(delay_ms));
    }

    fn force_flush(&mut self) {
        self.flush();
    }

    fn get_stats(&self) -> &WriteStats {
        &self.stats
    }
}

fn main() {
    let mut batcher = TradeBatcher::new(100, 1000); // max 100 записей или 1 секунда

    println!("=== Батчинг записи сделок ===\n");

    // Симулируем поток сделок
    println!("Генерация 500 сделок...\n");

    for i in 0..500 {
        let trade = Trade {
            id: i,
            symbol: if i % 3 == 0 { "BTCUSDT" } else if i % 3 == 1 { "ETHUSDT" } else { "SOLUSDT" }.to_string(),
            price: 42500.0 + (i as f64 * 0.1),
            quantity: 0.1 + (i as f64 * 0.001),
            timestamp: 1700000000 + i,
        };
        batcher.add(trade);
    }

    // Сбрасываем оставшееся
    batcher.force_flush();

    let stats = batcher.get_stats();
    println!("\n=== Статистика ===");
    println!("Всего записей в БД: {}", stats.total_writes);
    println!("Всего сделок: {}", stats.total_records);
    println!("Средний размер пакета: {:.1}", stats.avg_batch_size());
    println!("Среднее время записи: {:.1}ms", stats.avg_write_time());

    // Сравнение: без батчинга было бы 500 записей по ~5ms = 2500ms
    // С батчингом: 5 записей по ~10ms = 50ms
    println!("\nЭкономия времени: ~{}x",
        (stats.total_records as f64 * 5.0) / stats.total_time_ms as f64);
}
```

## Асинхронный батчинг с каналами

```rust
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum BatchCommand {
    Add(String),
    Flush,
    Shutdown,
}

#[derive(Debug)]
struct BatchProcessor {
    sender: Sender<BatchCommand>,
}

impl BatchProcessor {
    fn new(batch_size: usize, flush_interval_ms: u64) -> Self {
        let (sender, receiver) = mpsc::channel();

        // Запускаем фоновый обработчик
        thread::spawn(move || {
            Self::worker(receiver, batch_size, flush_interval_ms);
        });

        BatchProcessor { sender }
    }

    fn worker(receiver: Receiver<BatchCommand>, batch_size: usize, flush_interval_ms: u64) {
        let mut buffer: Vec<String> = Vec::with_capacity(batch_size);
        let flush_interval = Duration::from_millis(flush_interval_ms);

        loop {
            // Пробуем получить команду с таймаутом
            match receiver.recv_timeout(flush_interval) {
                Ok(BatchCommand::Add(item)) => {
                    buffer.push(item);
                    if buffer.len() >= batch_size {
                        Self::process_batch(&mut buffer);
                    }
                }
                Ok(BatchCommand::Flush) => {
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                }
                Ok(BatchCommand::Shutdown) => {
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                    println!("[Worker] Завершение работы");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Таймаут — сбрасываем по времени
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Worker] Канал закрыт");
                    break;
                }
            }
        }
    }

    fn process_batch(buffer: &mut Vec<String>) {
        println!("[Worker] Обработка пакета из {} элементов:", buffer.len());
        for item in buffer.iter() {
            println!("  - {}", item);
        }
        buffer.clear();
    }

    fn add(&self, item: String) {
        let _ = self.sender.send(BatchCommand::Add(item));
    }

    fn flush(&self) {
        let _ = self.sender.send(BatchCommand::Flush);
    }

    fn shutdown(&self) {
        let _ = self.sender.send(BatchCommand::Shutdown);
    }
}

fn main() {
    println!("=== Асинхронный батчинг ===\n");

    let processor = BatchProcessor::new(3, 500); // batch=3, interval=500ms

    // Добавляем элементы
    processor.add("Обновление BTC: $42500".to_string());
    processor.add("Обновление ETH: $2500".to_string());
    processor.add("Обновление SOL: $100".to_string()); // Триггер по размеру

    thread::sleep(Duration::from_millis(100));

    processor.add("Ордер BUY BTC".to_string());

    // Ждём flush по таймауту
    thread::sleep(Duration::from_millis(600));

    processor.add("Ордер SELL ETH".to_string());

    // Принудительный flush
    processor.flush();

    thread::sleep(Duration::from_millis(100));

    // Завершение
    processor.shutdown();
    thread::sleep(Duration::from_millis(100));

    println!("\nГотово!");
}
```

## Батчинг с дедупликацией

В трейдинге часто приходят дублирующиеся данные:

```rust
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

/// Батчер с дедупликацией — хранит только последнее значение для каждого ключа
struct DeduplicatingBatcher {
    // Для каждого символа храним только последнее обновление
    updates: HashMap<String, PriceUpdate>,
    // Порядок добавления для сохранения очерёдности
    order: Vec<String>,
    seen: HashSet<String>,
    max_size: usize,
    last_flush: Instant,
    max_wait: Duration,

    // Статистика
    total_received: u64,
    total_deduplicated: u64,
}

impl DeduplicatingBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        DeduplicatingBatcher {
            updates: HashMap::new(),
            order: Vec::new(),
            seen: HashSet::new(),
            max_size,
            last_flush: Instant::now(),
            max_wait: Duration::from_millis(max_wait_ms),
            total_received: 0,
            total_deduplicated: 0,
        }
    }

    fn add(&mut self, update: PriceUpdate) -> Option<Vec<PriceUpdate>> {
        self.total_received += 1;

        let symbol = update.symbol.clone();

        // Если уже есть обновление для этого символа — это дубликат
        if self.updates.contains_key(&symbol) {
            self.total_deduplicated += 1;
        } else {
            // Новый символ — добавляем в порядок
            self.order.push(symbol.clone());
            self.seen.insert(symbol.clone());
        }

        // Всегда перезаписываем последним значением
        self.updates.insert(symbol, update);

        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }

    fn should_flush(&self) -> bool {
        self.updates.len() >= self.max_size
            || self.last_flush.elapsed() >= self.max_wait
    }

    fn flush(&mut self) -> Vec<PriceUpdate> {
        // Возвращаем в порядке добавления
        let result: Vec<_> = self.order.drain(..)
            .filter_map(|s| self.updates.remove(&s))
            .collect();

        self.seen.clear();
        self.last_flush = Instant::now();
        result
    }

    fn dedup_rate(&self) -> f64 {
        if self.total_received > 0 {
            (self.total_deduplicated as f64 / self.total_received as f64) * 100.0
        } else {
            0.0
        }
    }
}

fn main() {
    let mut batcher = DeduplicatingBatcher::new(5, 100);

    println!("=== Батчинг с дедупликацией ===\n");

    // Симулируем поток обновлений цен (много дубликатов)
    let updates = vec![
        PriceUpdate { symbol: "BTC".to_string(), price: 42500.0, timestamp: 1 },
        PriceUpdate { symbol: "ETH".to_string(), price: 2500.0, timestamp: 1 },
        PriceUpdate { symbol: "BTC".to_string(), price: 42510.0, timestamp: 2 }, // дубликат
        PriceUpdate { symbol: "BTC".to_string(), price: 42520.0, timestamp: 3 }, // дубликат
        PriceUpdate { symbol: "SOL".to_string(), price: 100.0, timestamp: 1 },
        PriceUpdate { symbol: "ETH".to_string(), price: 2510.0, timestamp: 2 }, // дубликат
        PriceUpdate { symbol: "BTC".to_string(), price: 42530.0, timestamp: 4 }, // дубликат
        PriceUpdate { symbol: "DOGE".to_string(), price: 0.08, timestamp: 1 },
        PriceUpdate { symbol: "XRP".to_string(), price: 0.5, timestamp: 1 },
    ];

    println!("Входящий поток ({} обновлений):\n", updates.len());

    for update in updates {
        println!("  Получено: {} @ ${:.2} (ts={})",
            update.symbol, update.price, update.timestamp);

        if let Some(batch) = batcher.add(update) {
            println!("\n--- Обработка пакета ---");
            for u in &batch {
                println!("  Финальное значение: {} @ ${:.2}", u.symbol, u.price);
            }
            println!();
        }
    }

    println!("\n=== Статистика ===");
    println!("Всего получено: {}", batcher.total_received);
    println!("Дедуплицировано: {}", batcher.total_deduplicated);
    println!("Коэффициент дедупликации: {:.1}%", batcher.dedup_rate());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Батчинг** | Группировка операций для пакетной обработки |
| **Условия сброса** | По размеру, по времени, или по событию |
| **Приоритеты** | Критичные операции обрабатываются первыми |
| **Дедупликация** | Устранение дублирующихся данных в пакете |
| **Асинхронность** | Фоновая обработка через каналы |
| **Write batching** | Группировка записей в БД для оптимизации I/O |

## Практические задания

1. **Адаптивный батчер**: Реализуй батчер, который автоматически увеличивает размер пакета при высокой нагрузке и уменьшает при низкой.

2. **Батчинг с backpressure**: Создай систему, которая замедляет приём новых данных, если обработчик не успевает.

3. **Метрики батчинга**: Добавь сбор метрик: среднее время в буфере, процент заполнения, количество forced flush.

## Домашнее задание

1. **Умный батчер ордеров**: Реализуй батчер, который группирует ордера не только по времени, но и по символу и направлению (BUY/SELL), чтобы минимизировать количество API-вызовов.

2. **Батчинг с retry**: Создай систему, которая при ошибке отправки пакета автоматически повторяет попытку с экспоненциальным backoff.

3. **Распределённый батчинг**: Реализуй батчер, который работает на нескольких узлах и координирует отправку через leader election.

4. **Батчинг для WebSocket**: Создай систему, которая группирует исходящие сообщения WebSocket для уменьшения количества фреймов.

## Навигация

[← Предыдущий день](../328-*/ru.md) | [Следующий день →](../330-*/ru.md)
