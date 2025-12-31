# День 181: Проект: Мульти-биржевой сборщик данных

## Обзор проекта

Добро пожаловать в проектную главу месяца многопоточности! Мы создадим **мульти-биржевой сборщик данных** — систему, которая одновременно собирает рыночные данные с нескольких криптовалютных бирж, агрегирует их и предоставляет унифицированный поток данных для анализа.

### Аналогия из трейдинга

Представь, что ты профессиональный трейдер, работающий на нескольких биржах одновременно: Binance, Kraken, Coinbase. Каждая биржа — это отдельный источник данных с разными ценами, разной скоростью обновления и разными форматами. Чтобы найти лучшую цену или арбитражную возможность, тебе нужно:

1. **Параллельно получать данные** со всех бирж (потоки)
2. **Безопасно агрегировать** их в единое хранилище (синхронизация)
3. **Обрабатывать ошибки** без остановки всей системы (отказоустойчивость)
4. **Анализировать спреды** между биржами (обработка данных)

Именно это мы и реализуем!

## Архитектура системы

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Exchange Data Collector                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Binance  │  │  Kraken  │  │ Coinbase │  │  Bybit   │        │
│  │ Fetcher  │  │ Fetcher  │  │ Fetcher  │  │ Fetcher  │        │
│  │ (Thread) │  │ (Thread) │  │ (Thread) │  │ (Thread) │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │               │
│       └─────────────┴──────┬──────┴─────────────┘               │
│                            │                                    │
│                            ▼                                    │
│                 ┌──────────────────┐                           │
│                 │  Price Channel   │                           │
│                 │  (mpsc::channel) │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│                          ▼                                      │
│                 ┌──────────────────┐                           │
│                 │   Aggregator     │                           │
│                 │    (Thread)      │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│                          ▼                                      │
│                 ┌──────────────────┐                           │
│                 │  Price Storage   │                           │
│                 │ (Arc<RwLock<>>)  │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│           ┌──────────────┼──────────────┐                      │
│           ▼              ▼              ▼                      │
│    ┌───────────┐  ┌───────────┐  ┌───────────┐                │
│    │ Analyzer  │  │ Arbitrage │  │   Logger  │                │
│    │ (Thread)  │  │ Detector  │  │  (Thread) │                │
│    └───────────┘  └───────────┘  └───────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Часть 1: Базовые структуры данных

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Идентификатор биржи
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exchange {
    Binance,
    Kraken,
    Coinbase,
    Bybit,
}

impl Exchange {
    /// Возвращает все поддерживаемые биржи
    pub fn all() -> Vec<Exchange> {
        vec![
            Exchange::Binance,
            Exchange::Kraken,
            Exchange::Coinbase,
            Exchange::Bybit,
        ]
    }

    /// Имя биржи для отображения
    pub fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => "Binance",
            Exchange::Kraken => "Kraken",
            Exchange::Coinbase => "Coinbase",
            Exchange::Bybit => "Bybit",
        }
    }

    /// Симулированная задержка API (в миллисекундах)
    pub fn api_latency(&self) -> u64 {
        match self {
            Exchange::Binance => 50,
            Exchange::Kraken => 80,
            Exchange::Coinbase => 60,
            Exchange::Bybit => 70,
        }
    }
}

/// Торговая пара
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: String,   // BTC
    pub quote: String,  // USDT
}

impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        TradingPair {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

/// Тиковые данные с биржи
#[derive(Debug, Clone)]
pub struct TickData {
    pub exchange: Exchange,
    pub pair: TradingPair,
    pub bid: f64,           // Лучшая цена покупки
    pub ask: f64,           // Лучшая цена продажи
    pub bid_volume: f64,    // Объём на лучшей цене покупки
    pub ask_volume: f64,    // Объём на лучшей цене продажи
    pub last_price: f64,    // Последняя цена сделки
    pub timestamp: u64,     // Unix timestamp в миллисекундах
    pub sequence: u64,      // Порядковый номер обновления
}

impl TickData {
    /// Спред между bid и ask
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Спред в процентах
    pub fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    /// Средняя цена
    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

/// Агрегированные данные по паре со всех бирж
#[derive(Debug, Clone)]
pub struct AggregatedPrice {
    pub pair: TradingPair,
    pub best_bid: (Exchange, f64),      // Лучшая цена покупки и биржа
    pub best_ask: (Exchange, f64),      // Лучшая цена продажи и биржа
    pub prices: HashMap<Exchange, TickData>,
    pub last_update: u64,
}

impl AggregatedPrice {
    pub fn new(pair: TradingPair) -> Self {
        AggregatedPrice {
            pair,
            best_bid: (Exchange::Binance, 0.0),
            best_ask: (Exchange::Binance, f64::MAX),
            prices: HashMap::new(),
            last_update: 0,
        }
    }

    /// Обновление данных с биржи
    pub fn update(&mut self, tick: TickData) {
        // Обновляем лучший bid (ищем максимум)
        if tick.bid > self.best_bid.1 {
            self.best_bid = (tick.exchange, tick.bid);
        }

        // Обновляем лучший ask (ищем минимум)
        if tick.ask < self.best_ask.1 {
            self.best_ask = (tick.exchange, tick.ask);
        }

        self.last_update = tick.timestamp;
        self.prices.insert(tick.exchange, tick);
    }

    /// Проверка арбитражной возможности
    /// Арбитраж возможен, когда best_bid > best_ask на разных биржах
    pub fn arbitrage_opportunity(&self) -> Option<ArbitrageOpportunity> {
        if self.best_bid.0 != self.best_ask.0 && self.best_bid.1 > self.best_ask.1 {
            let profit_percent = ((self.best_bid.1 - self.best_ask.1) / self.best_ask.1) * 100.0;
            Some(ArbitrageOpportunity {
                pair: self.pair.clone(),
                buy_exchange: self.best_ask.0,
                buy_price: self.best_ask.1,
                sell_exchange: self.best_bid.0,
                sell_price: self.best_bid.1,
                profit_percent,
            })
        } else {
            None
        }
    }
}

/// Арбитражная возможность
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: TradingPair,
    pub buy_exchange: Exchange,
    pub buy_price: f64,
    pub sell_exchange: Exchange,
    pub sell_price: f64,
    pub profit_percent: f64,
}

impl ArbitrageOpportunity {
    pub fn display(&self) -> String {
        format!(
            "ARBITRAGE: {} - Buy on {} @ {:.2}, Sell on {} @ {:.2}, Profit: {:.4}%",
            self.pair.symbol(),
            self.buy_exchange.name(),
            self.buy_price,
            self.sell_exchange.name(),
            self.sell_price,
            self.profit_percent
        )
    }
}
```

## Часть 2: Симулятор биржи

В реальном проекте здесь были бы HTTP/WebSocket клиенты. Для обучения создадим симулятор:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Генератор последовательностей для симуляции
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Симулятор биржевого API
pub struct ExchangeSimulator {
    exchange: Exchange,
    base_prices: HashMap<String, f64>,
    volatility: f64,
}

impl ExchangeSimulator {
    pub fn new(exchange: Exchange) -> Self {
        let mut base_prices = HashMap::new();

        // Базовые цены немного отличаются для каждой биржи
        // (симуляция реальных расхождений)
        let offset = match exchange {
            Exchange::Binance => 0.0,
            Exchange::Kraken => 0.15,
            Exchange::Coinbase => -0.10,
            Exchange::Bybit => 0.05,
        };

        base_prices.insert("BTC".to_string(), 42000.0 + offset * 100.0);
        base_prices.insert("ETH".to_string(), 2200.0 + offset * 10.0);
        base_prices.insert("SOL".to_string(), 95.0 + offset);

        ExchangeSimulator {
            exchange,
            base_prices,
            volatility: 0.001, // 0.1% волатильность
        }
    }

    /// Симуляция получения тиковых данных
    pub fn fetch_tick(&self, pair: &TradingPair) -> Result<TickData, String> {
        // Симуляция задержки API
        thread::sleep(Duration::from_millis(self.exchange.api_latency()));

        // Симуляция случайных ошибок (5% вероятность)
        if rand_simple() < 0.05 {
            return Err(format!("{}: Connection timeout", self.exchange.name()));
        }

        let base_price = self.base_prices
            .get(&pair.base)
            .copied()
            .unwrap_or(100.0);

        // Добавляем случайное отклонение
        let random_factor = 1.0 + (rand_simple() - 0.5) * 2.0 * self.volatility;
        let mid_price = base_price * random_factor;

        // Спред зависит от биржи
        let spread_percent = match self.exchange {
            Exchange::Binance => 0.0005,  // 0.05%
            Exchange::Kraken => 0.0008,   // 0.08%
            Exchange::Coinbase => 0.0010, // 0.10%
            Exchange::Bybit => 0.0006,    // 0.06%
        };

        let half_spread = mid_price * spread_percent / 2.0;
        let bid = mid_price - half_spread;
        let ask = mid_price + half_spread;

        Ok(TickData {
            exchange: self.exchange,
            pair: pair.clone(),
            bid,
            ask,
            bid_volume: rand_simple() * 10.0,
            ask_volume: rand_simple() * 10.0,
            last_price: mid_price,
            timestamp: current_timestamp_ms(),
            sequence: SEQUENCE.fetch_add(1, Ordering::SeqCst),
        })
    }
}

/// Простой генератор случайных чисел (без внешних зависимостей)
fn rand_simple() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    thread::current().id().hash(&mut hasher);

    let hash = hasher.finish();
    (hash as f64) / (u64::MAX as f64)
}

/// Текущий timestamp в миллисекундах
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
```

## Часть 3: Сборщик данных с каналами

```rust
/// Сообщение в канал данных
#[derive(Debug)]
pub enum DataMessage {
    Tick(TickData),
    Error { exchange: Exchange, error: String },
    Shutdown,
}

/// Статистика сборщика
#[derive(Debug, Default)]
pub struct CollectorStats {
    pub ticks_received: u64,
    pub errors: u64,
    pub last_tick_time: Option<Instant>,
}

/// Конфигурация сборщика
pub struct CollectorConfig {
    pub pairs: Vec<TradingPair>,
    pub update_interval: Duration,
    pub max_retries: u32,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        CollectorConfig {
            pairs: vec![
                TradingPair::new("BTC", "USDT"),
                TradingPair::new("ETH", "USDT"),
            ],
            update_interval: Duration::from_millis(500),
            max_retries: 3,
        }
    }
}

/// Сборщик данных с одной биржи
pub struct ExchangeFetcher {
    exchange: Exchange,
    simulator: ExchangeSimulator,
    config: Arc<CollectorConfig>,
    sender: mpsc::Sender<DataMessage>,
    running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<CollectorStats>>,
}

impl ExchangeFetcher {
    pub fn new(
        exchange: Exchange,
        config: Arc<CollectorConfig>,
        sender: mpsc::Sender<DataMessage>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        ExchangeFetcher {
            exchange,
            simulator: ExchangeSimulator::new(exchange),
            config,
            sender,
            running,
            stats: Arc::new(RwLock::new(CollectorStats::default())),
        }
    }

    /// Основной цикл сбора данных
    pub fn run(&self) {
        println!("[{}] Fetcher started", self.exchange.name());

        while *self.running.read().unwrap() {
            for pair in &self.config.pairs {
                if !*self.running.read().unwrap() {
                    break;
                }

                self.fetch_with_retry(pair);
            }

            thread::sleep(self.config.update_interval);
        }

        println!("[{}] Fetcher stopped", self.exchange.name());
    }

    /// Получение данных с повторными попытками
    fn fetch_with_retry(&self, pair: &TradingPair) {
        let mut retries = 0;

        loop {
            match self.simulator.fetch_tick(pair) {
                Ok(tick) => {
                    // Обновляем статистику
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.ticks_received += 1;
                        stats.last_tick_time = Some(Instant::now());
                    }

                    // Отправляем в канал
                    if self.sender.send(DataMessage::Tick(tick)).is_err() {
                        println!("[{}] Channel closed", self.exchange.name());
                        return;
                    }
                    break;
                }
                Err(error) => {
                    retries += 1;

                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.errors += 1;
                    }

                    if retries >= self.config.max_retries {
                        let _ = self.sender.send(DataMessage::Error {
                            exchange: self.exchange,
                            error: error.clone(),
                        });
                        println!(
                            "[{}] Failed after {} retries: {}",
                            self.exchange.name(),
                            retries,
                            error
                        );
                        break;
                    }

                    // Экспоненциальная задержка перед повтором
                    thread::sleep(Duration::from_millis(100 * retries as u64));
                }
            }
        }
    }

    pub fn get_stats(&self) -> CollectorStats {
        self.stats.read().unwrap().clone()
    }
}
```

## Часть 4: Агрегатор данных

```rust
/// Хранилище агрегированных цен
pub type PriceStorage = Arc<RwLock<HashMap<String, AggregatedPrice>>>;

/// Агрегатор данных со всех бирж
pub struct DataAggregator {
    receiver: mpsc::Receiver<DataMessage>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
    arbitrage_callback: Option<Box<dyn Fn(ArbitrageOpportunity) + Send>>,
}

impl DataAggregator {
    pub fn new(
        receiver: mpsc::Receiver<DataMessage>,
        storage: PriceStorage,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        DataAggregator {
            receiver,
            storage,
            running,
            arbitrage_callback: None,
        }
    }

    /// Установка callback для арбитражных возможностей
    pub fn set_arbitrage_callback<F>(&mut self, callback: F)
    where
        F: Fn(ArbitrageOpportunity) + Send + 'static,
    {
        self.arbitrage_callback = Some(Box::new(callback));
    }

    /// Основной цикл агрегации
    pub fn run(&self) {
        println!("[Aggregator] Started");

        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(DataMessage::Tick(tick)) => {
                    self.process_tick(tick);
                }
                Ok(DataMessage::Error { exchange, error }) => {
                    println!("[Aggregator] Error from {}: {}", exchange.name(), error);
                }
                Ok(DataMessage::Shutdown) => {
                    println!("[Aggregator] Received shutdown signal");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !*self.running.read().unwrap() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Aggregator] Channel disconnected");
                    break;
                }
            }
        }

        println!("[Aggregator] Stopped");
    }

    fn process_tick(&self, tick: TickData) {
        let symbol = tick.pair.symbol();

        let mut storage = self.storage.write().unwrap();
        let aggregated = storage
            .entry(symbol.clone())
            .or_insert_with(|| AggregatedPrice::new(tick.pair.clone()));

        aggregated.update(tick);

        // Проверяем арбитражные возможности
        if let Some(opportunity) = aggregated.arbitrage_opportunity() {
            if let Some(ref callback) = self.arbitrage_callback {
                callback(opportunity);
            }
        }
    }
}
```

## Часть 5: Анализатор и мониторинг

```rust
/// Статистика системы
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_ticks: u64,
    pub ticks_per_second: f64,
    pub active_pairs: usize,
    pub active_exchanges: usize,
    pub arbitrage_opportunities: u64,
    pub uptime_seconds: u64,
}

/// Анализатор рыночных данных
pub struct MarketAnalyzer {
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
    analysis_interval: Duration,
}

impl MarketAnalyzer {
    pub fn new(storage: PriceStorage, running: Arc<RwLock<bool>>) -> Self {
        MarketAnalyzer {
            storage,
            running,
            analysis_interval: Duration::from_secs(5),
        }
    }

    pub fn run(&self) {
        println!("[Analyzer] Started");

        while *self.running.read().unwrap() {
            self.analyze();
            thread::sleep(self.analysis_interval);
        }

        println!("[Analyzer] Stopped");
    }

    fn analyze(&self) {
        let storage = self.storage.read().unwrap();

        println!("\n========== Market Analysis ==========");

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!("  Best Bid: {} @ {:.2}", agg.best_bid.0.name(), agg.best_bid.1);
            println!("  Best Ask: {} @ {:.2}", agg.best_ask.0.name(), agg.best_ask.1);

            // Показываем цены со всех бирж
            for (exchange, tick) in &agg.prices {
                println!(
                    "  {} - Bid: {:.2}, Ask: {:.2}, Spread: {:.4}%",
                    exchange.name(),
                    tick.bid,
                    tick.ask,
                    tick.spread_percent()
                );
            }

            // Проверяем арбитраж
            if let Some(opp) = agg.arbitrage_opportunity() {
                println!("  *** {} ***", opp.display());
            }
        }

        println!("\n======================================\n");
    }
}
```

## Часть 6: Основная программа

```rust
use std::sync::atomic::AtomicU64;

/// Счётчик арбитражных возможностей
static ARBITRAGE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Главная структура сборщика данных
pub struct MultiExchangeCollector {
    config: Arc<CollectorConfig>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MultiExchangeCollector {
    pub fn new(config: CollectorConfig) -> Self {
        MultiExchangeCollector {
            config: Arc::new(config),
            storage: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(true)),
        }
    }

    pub fn run(&self, duration: Duration) {
        println!("Starting Multi-Exchange Data Collector...");
        println!("Collecting data for {:?}", duration);
        println!("Pairs: {:?}", self.config.pairs.iter().map(|p| p.symbol()).collect::<Vec<_>>());
        println!("Exchanges: {:?}", Exchange::all().iter().map(|e| e.name()).collect::<Vec<_>>());
        println!();

        // Создаём канал для передачи данных
        let (sender, receiver) = mpsc::channel::<DataMessage>();

        let mut handles = vec![];

        // Запускаем fetcher для каждой биржи
        for exchange in Exchange::all() {
            let fetcher = ExchangeFetcher::new(
                exchange,
                Arc::clone(&self.config),
                sender.clone(),
                Arc::clone(&self.running),
            );

            let handle = thread::spawn(move || {
                fetcher.run();
            });

            handles.push(handle);
        }

        // Запускаем агрегатор
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);

        let aggregator_handle = thread::spawn(move || {
            let mut aggregator = DataAggregator::new(receiver, storage_clone, running_clone);

            // Устанавливаем callback для арбитража
            aggregator.set_arbitrage_callback(|opp| {
                ARBITRAGE_COUNT.fetch_add(1, Ordering::SeqCst);
                println!("!!! ARBITRAGE DETECTED: {}", opp.display());
            });

            aggregator.run();
        });

        // Запускаем анализатор
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);

        let analyzer_handle = thread::spawn(move || {
            let analyzer = MarketAnalyzer::new(storage_clone, running_clone);
            analyzer.run();
        });

        // Ждём указанное время
        thread::sleep(duration);

        // Останавливаем все потоки
        println!("\nStopping collector...");
        *self.running.write().unwrap() = false;

        // Отправляем сигнал завершения
        let _ = sender.send(DataMessage::Shutdown);

        // Ожидаем завершения всех потоков
        for handle in handles {
            let _ = handle.join();
        }
        let _ = aggregator_handle.join();
        let _ = analyzer_handle.join();

        self.print_final_stats();
    }

    fn print_final_stats(&self) {
        let storage = self.storage.read().unwrap();

        println!("\n========== Final Statistics ==========");
        println!("Total pairs tracked: {}", storage.len());
        println!(
            "Arbitrage opportunities found: {}",
            ARBITRAGE_COUNT.load(Ordering::SeqCst)
        );

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!("  Exchanges with data: {}", agg.prices.len());

            let total_volume: f64 = agg.prices.values().map(|t| t.bid_volume + t.ask_volume).sum();
            println!("  Total volume: {:.2}", total_volume);

            if let Some(opp) = agg.arbitrage_opportunity() {
                println!("  Current arbitrage: {:.4}%", opp.profit_percent);
            }
        }

        println!("\n======================================");
    }
}

fn main() {
    let config = CollectorConfig {
        pairs: vec![
            TradingPair::new("BTC", "USDT"),
            TradingPair::new("ETH", "USDT"),
            TradingPair::new("SOL", "USDT"),
        ],
        update_interval: Duration::from_millis(200),
        max_retries: 3,
    };

    let collector = MultiExchangeCollector::new(config);

    // Запускаем на 30 секунд
    collector.run(Duration::from_secs(30));
}
```

## Часть 7: Полный рабочий пример

Вот полный файл `main.rs`, который можно скомпилировать и запустить:

```rust
//! Multi-Exchange Data Collector
//!
//! Пример многопоточного сборщика данных с нескольких бирж.
//! Демонстрирует: потоки, каналы, синхронизацию, RwLock, Arc.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ==================== Базовые структуры ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exchange {
    Binance,
    Kraken,
    Coinbase,
    Bybit,
}

impl Exchange {
    pub fn all() -> Vec<Exchange> {
        vec![
            Exchange::Binance,
            Exchange::Kraken,
            Exchange::Coinbase,
            Exchange::Bybit,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => "Binance",
            Exchange::Kraken => "Kraken",
            Exchange::Coinbase => "Coinbase",
            Exchange::Bybit => "Bybit",
        }
    }

    pub fn api_latency(&self) -> u64 {
        match self {
            Exchange::Binance => 50,
            Exchange::Kraken => 80,
            Exchange::Coinbase => 60,
            Exchange::Bybit => 70,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
}

impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        TradingPair {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

#[derive(Debug, Clone)]
pub struct TickData {
    pub exchange: Exchange,
    pub pair: TradingPair,
    pub bid: f64,
    pub ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub last_price: f64,
    pub timestamp: u64,
    pub sequence: u64,
}

impl TickData {
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    pub fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedPrice {
    pub pair: TradingPair,
    pub best_bid: (Exchange, f64),
    pub best_ask: (Exchange, f64),
    pub prices: HashMap<Exchange, TickData>,
    pub last_update: u64,
}

impl AggregatedPrice {
    pub fn new(pair: TradingPair) -> Self {
        AggregatedPrice {
            pair,
            best_bid: (Exchange::Binance, 0.0),
            best_ask: (Exchange::Binance, f64::MAX),
            prices: HashMap::new(),
            last_update: 0,
        }
    }

    pub fn update(&mut self, tick: TickData) {
        if tick.bid > self.best_bid.1 {
            self.best_bid = (tick.exchange, tick.bid);
        }
        if tick.ask < self.best_ask.1 {
            self.best_ask = (tick.exchange, tick.ask);
        }
        self.last_update = tick.timestamp;
        self.prices.insert(tick.exchange, tick);
    }

    pub fn arbitrage_opportunity(&self) -> Option<ArbitrageOpportunity> {
        if self.best_bid.0 != self.best_ask.0 && self.best_bid.1 > self.best_ask.1 {
            let profit_percent =
                ((self.best_bid.1 - self.best_ask.1) / self.best_ask.1) * 100.0;
            Some(ArbitrageOpportunity {
                pair: self.pair.clone(),
                buy_exchange: self.best_ask.0,
                buy_price: self.best_ask.1,
                sell_exchange: self.best_bid.0,
                sell_price: self.best_bid.1,
                profit_percent,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: TradingPair,
    pub buy_exchange: Exchange,
    pub buy_price: f64,
    pub sell_exchange: Exchange,
    pub sell_price: f64,
    pub profit_percent: f64,
}

impl ArbitrageOpportunity {
    pub fn display(&self) -> String {
        format!(
            "{} - Buy {} @ {:.2}, Sell {} @ {:.2}, Profit: {:.4}%",
            self.pair.symbol(),
            self.buy_exchange.name(),
            self.buy_price,
            self.sell_exchange.name(),
            self.sell_price,
            self.profit_percent
        )
    }
}

// ==================== Симулятор ====================

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static RAND_STATE: AtomicU64 = AtomicU64::new(12345);

fn rand_simple() -> f64 {
    let mut state = RAND_STATE.load(Ordering::Relaxed);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    RAND_STATE.store(state, Ordering::Relaxed);
    (state >> 33) as f64 / (u32::MAX as f64)
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct ExchangeSimulator {
    exchange: Exchange,
    base_prices: HashMap<String, f64>,
    volatility: f64,
}

impl ExchangeSimulator {
    pub fn new(exchange: Exchange) -> Self {
        let mut base_prices = HashMap::new();
        let offset = match exchange {
            Exchange::Binance => 0.0,
            Exchange::Kraken => 0.15,
            Exchange::Coinbase => -0.10,
            Exchange::Bybit => 0.05,
        };
        base_prices.insert("BTC".to_string(), 42000.0 + offset * 100.0);
        base_prices.insert("ETH".to_string(), 2200.0 + offset * 10.0);
        base_prices.insert("SOL".to_string(), 95.0 + offset);

        ExchangeSimulator {
            exchange,
            base_prices,
            volatility: 0.002,
        }
    }

    pub fn fetch_tick(&self, pair: &TradingPair) -> Result<TickData, String> {
        thread::sleep(Duration::from_millis(self.exchange.api_latency()));

        if rand_simple() < 0.02 {
            return Err(format!("{}: Connection timeout", self.exchange.name()));
        }

        let base_price = self.base_prices.get(&pair.base).copied().unwrap_or(100.0);
        let random_factor = 1.0 + (rand_simple() - 0.5) * 2.0 * self.volatility;
        let mid_price = base_price * random_factor;

        let spread_percent = match self.exchange {
            Exchange::Binance => 0.0005,
            Exchange::Kraken => 0.0008,
            Exchange::Coinbase => 0.0010,
            Exchange::Bybit => 0.0006,
        };

        let half_spread = mid_price * spread_percent / 2.0;

        Ok(TickData {
            exchange: self.exchange,
            pair: pair.clone(),
            bid: mid_price - half_spread,
            ask: mid_price + half_spread,
            bid_volume: rand_simple() * 10.0,
            ask_volume: rand_simple() * 10.0,
            last_price: mid_price,
            timestamp: current_timestamp_ms(),
            sequence: SEQUENCE.fetch_add(1, Ordering::SeqCst),
        })
    }
}

// ==================== Сообщения и сборщик ====================

#[derive(Debug)]
pub enum DataMessage {
    Tick(TickData),
    Error { exchange: Exchange, error: String },
    Shutdown,
}

pub struct CollectorConfig {
    pub pairs: Vec<TradingPair>,
    pub update_interval: Duration,
    pub max_retries: u32,
}

pub struct ExchangeFetcher {
    exchange: Exchange,
    simulator: ExchangeSimulator,
    config: Arc<CollectorConfig>,
    sender: mpsc::Sender<DataMessage>,
    running: Arc<RwLock<bool>>,
}

impl ExchangeFetcher {
    pub fn new(
        exchange: Exchange,
        config: Arc<CollectorConfig>,
        sender: mpsc::Sender<DataMessage>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        ExchangeFetcher {
            exchange,
            simulator: ExchangeSimulator::new(exchange),
            config,
            sender,
            running,
        }
    }

    pub fn run(&self) {
        println!("[{}] Fetcher started", self.exchange.name());

        while *self.running.read().unwrap() {
            for pair in &self.config.pairs {
                if !*self.running.read().unwrap() {
                    break;
                }
                self.fetch_with_retry(pair);
            }
            thread::sleep(self.config.update_interval);
        }

        println!("[{}] Fetcher stopped", self.exchange.name());
    }

    fn fetch_with_retry(&self, pair: &TradingPair) {
        let mut retries = 0;
        loop {
            match self.simulator.fetch_tick(pair) {
                Ok(tick) => {
                    if self.sender.send(DataMessage::Tick(tick)).is_err() {
                        return;
                    }
                    break;
                }
                Err(error) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        let _ = self.sender.send(DataMessage::Error {
                            exchange: self.exchange,
                            error,
                        });
                        break;
                    }
                    thread::sleep(Duration::from_millis(50 * retries as u64));
                }
            }
        }
    }
}

// ==================== Агрегатор ====================

pub type PriceStorage = Arc<RwLock<HashMap<String, AggregatedPrice>>>;

static ARBITRAGE_COUNT: AtomicU64 = AtomicU64::new(0);

pub struct DataAggregator {
    receiver: mpsc::Receiver<DataMessage>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl DataAggregator {
    pub fn new(
        receiver: mpsc::Receiver<DataMessage>,
        storage: PriceStorage,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        DataAggregator {
            receiver,
            storage,
            running,
        }
    }

    pub fn run(&self) {
        println!("[Aggregator] Started");

        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(DataMessage::Tick(tick)) => {
                    self.process_tick(tick);
                }
                Ok(DataMessage::Error { exchange, error }) => {
                    println!("[Aggregator] Error from {}: {}", exchange.name(), error);
                }
                Ok(DataMessage::Shutdown) => {
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !*self.running.read().unwrap() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        println!("[Aggregator] Stopped");
    }

    fn process_tick(&self, tick: TickData) {
        let symbol = tick.pair.symbol();
        let mut storage = self.storage.write().unwrap();
        let aggregated = storage
            .entry(symbol)
            .or_insert_with(|| AggregatedPrice::new(tick.pair.clone()));

        aggregated.update(tick);

        if let Some(opp) = aggregated.arbitrage_opportunity() {
            ARBITRAGE_COUNT.fetch_add(1, Ordering::SeqCst);
            println!(">>> ARBITRAGE: {}", opp.display());
        }
    }
}

// ==================== Анализатор ====================

pub struct MarketAnalyzer {
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MarketAnalyzer {
    pub fn new(storage: PriceStorage, running: Arc<RwLock<bool>>) -> Self {
        MarketAnalyzer { storage, running }
    }

    pub fn run(&self) {
        println!("[Analyzer] Started");

        while *self.running.read().unwrap() {
            thread::sleep(Duration::from_secs(5));
            self.analyze();
        }

        println!("[Analyzer] Stopped");
    }

    fn analyze(&self) {
        let storage = self.storage.read().unwrap();
        println!("\n========== Market Analysis ==========");

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!(
                "  Best Bid: {} @ {:.2}",
                agg.best_bid.0.name(),
                agg.best_bid.1
            );
            println!(
                "  Best Ask: {} @ {:.2}",
                agg.best_ask.0.name(),
                agg.best_ask.1
            );

            for (exchange, tick) in &agg.prices {
                println!(
                    "    {} - Bid: {:.2}, Ask: {:.2}, Spread: {:.4}%",
                    exchange.name(),
                    tick.bid,
                    tick.ask,
                    tick.spread_percent()
                );
            }
        }

        println!("\n======================================\n");
    }
}

// ==================== Главный сборщик ====================

pub struct MultiExchangeCollector {
    config: Arc<CollectorConfig>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MultiExchangeCollector {
    pub fn new(config: CollectorConfig) -> Self {
        MultiExchangeCollector {
            config: Arc::new(config),
            storage: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(true)),
        }
    }

    pub fn run(&self, duration: Duration) {
        println!("=== Multi-Exchange Data Collector ===");
        println!("Duration: {:?}", duration);
        println!(
            "Pairs: {:?}",
            self.config.pairs.iter().map(|p| p.symbol()).collect::<Vec<_>>()
        );
        println!(
            "Exchanges: {:?}\n",
            Exchange::all().iter().map(|e| e.name()).collect::<Vec<_>>()
        );

        let (sender, receiver) = mpsc::channel::<DataMessage>();
        let mut handles = vec![];

        // Запуск fetcher'ов
        for exchange in Exchange::all() {
            let fetcher = ExchangeFetcher::new(
                exchange,
                Arc::clone(&self.config),
                sender.clone(),
                Arc::clone(&self.running),
            );
            handles.push(thread::spawn(move || fetcher.run()));
        }

        // Запуск агрегатора
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);
        let aggregator_handle = thread::spawn(move || {
            let aggregator = DataAggregator::new(receiver, storage_clone, running_clone);
            aggregator.run();
        });

        // Запуск анализатора
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);
        let analyzer_handle = thread::spawn(move || {
            let analyzer = MarketAnalyzer::new(storage_clone, running_clone);
            analyzer.run();
        });

        // Ожидание
        thread::sleep(duration);

        // Остановка
        println!("\nStopping...");
        *self.running.write().unwrap() = false;
        let _ = sender.send(DataMessage::Shutdown);

        for handle in handles {
            let _ = handle.join();
        }
        let _ = aggregator_handle.join();
        let _ = analyzer_handle.join();

        self.print_stats();
    }

    fn print_stats(&self) {
        let storage = self.storage.read().unwrap();
        println!("\n========== Final Statistics ==========");
        println!("Pairs tracked: {}", storage.len());
        println!(
            "Arbitrage opportunities: {}",
            ARBITRAGE_COUNT.load(Ordering::SeqCst)
        );

        for (symbol, agg) in storage.iter() {
            println!("\n{}: {} exchanges", symbol, agg.prices.len());
        }
        println!("\n======================================");
    }
}

fn main() {
    let config = CollectorConfig {
        pairs: vec![
            TradingPair::new("BTC", "USDT"),
            TradingPair::new("ETH", "USDT"),
            TradingPair::new("SOL", "USDT"),
        ],
        update_interval: Duration::from_millis(200),
        max_retries: 3,
    };

    let collector = MultiExchangeCollector::new(config);
    collector.run(Duration::from_secs(15));
}
```

## Что мы узнали

| Концепция | Применение в проекте |
|-----------|---------------------|
| `thread::spawn` | Создание отдельных потоков для каждой биржи |
| `mpsc::channel` | Передача тиковых данных от fetcher'ов к агрегатору |
| `Arc<RwLock<T>>` | Разделяемое хранилище цен с множественным чтением |
| `Arc<Mutex<T>>` | Потокобезопасные счётчики и статистика |
| `AtomicU64` | Лёгкие атомарные счётчики без блокировок |
| Graceful shutdown | Корректное завершение всех потоков |
| Error handling | Повторные попытки и обработка ошибок API |
| Callback pattern | Уведомления об арбитражных возможностях |

## Домашнее задание

### Упражнение 1: Добавление новой биржи

Добавь поддержку ещё одной биржи (например, OKX или Huobi):
- Добавь новый вариант в enum `Exchange`
- Настрой параметры симуляции (latency, spread, base prices)
- Убедись, что новая биржа участвует в арбитражных расчётах

### Упражнение 2: Улучшенный детектор арбитража

Модифицируй `ArbitrageOpportunity` для учёта:
- Комиссий биржи (maker/taker fees)
- Минимального объёма для сделки
- Времени жизни возможности (staleness check)

```rust
pub struct ImprovedArbitrage {
    // ... базовые поля ...
    pub maker_fee: f64,      // 0.1%
    pub taker_fee: f64,      // 0.1%
    pub min_volume: f64,     // минимальный объём
    pub net_profit: f64,     // прибыль после комиссий
    pub is_profitable: bool, // выгодно ли после комиссий?
}
```

### Упражнение 3: Персистентное хранение

Добавь сохранение истории цен в файл:
- Каждые N секунд записывай текущие цены в CSV
- Формат: `timestamp,exchange,pair,bid,ask,volume`
- Используй отдельный поток для записи

### Упражнение 4: Rate Limiting

Реализуй ограничение частоты запросов к биржам:
- Не более N запросов в секунду на биржу
- Используй `std::sync::Condvar` для ожидания
- Добавь метрики: сколько запросов было отложено

```rust
pub struct RateLimiter {
    max_requests_per_second: u32,
    requests: Arc<Mutex<VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn acquire(&self) {
        // Подожди, если превышен лимит
    }
}
```

### Упражнение 5: Мониторинг здоровья системы

Создай компонент для мониторинга:
- Отслеживай, сколько времени прошло с последнего тика от каждой биржи
- Если биржа не отвечает > 5 секунд — выводи предупреждение
- Веди статистику: uptime каждой биржи, средняя latency

## Навигация

[← День 180: Профилирование потоков](../180-thread-profiling/ru.md) | [День 182: Sync vs Async →](../182-sync-vs-async/ru.md)
