# День 321: Кеширование результатов

## Аналогия из трейдинга

Представь, что ты управляешь торговым терминалом, который показывает данные по сотням активов. Каждый раз, когда трейдер открывает страницу актива, система запрашивает:
- Текущую цену с биржи
- Историю сделок за последний час
- Расчёт технических индикаторов
- Новости по активу

Без кеширования каждый из 1000 трейдеров, открывающих страницу BTC, создаёт 4 запроса. Это 4000 запросов в секунду на одни и те же данные!

**Кеширование результатов** — это как информационное табло в торговом зале:
- Данные обновляются централизованно раз в секунду
- Все трейдеры смотрят на одно табло
- Нет нужды каждому звонить на биржу отдельно

В отличие от мемоизации (кеширование на уровне функции), **кеширование результатов** работает на уровне системы — кешируются ответы API, результаты сложных запросов, агрегированные данные.

## Когда использовать кеширование результатов?

| Сценарий | Пример из трейдинга | Выгода |
|----------|---------------------|--------|
| **Частые одинаковые запросы** | Цена BTC запрашивается 1000 раз/сек | Снижение нагрузки на биржу |
| **Дорогие вычисления** | Расчёт риска портфеля | Экономия CPU |
| **Медленные внешние сервисы** | Запрос к API брокера | Ускорение отклика |
| **Редко меняющиеся данные** | Список доступных активов | Минимизация сетевых запросов |

## Простой кеш с инвалидацией

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Кешированное значение с временной меткой
struct CachedValue<T> {
    value: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CachedValue<T> {
    fn new(value: T, ttl: Duration) -> Self {
        CachedValue {
            value,
            cached_at: Instant::now(),
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }

    fn get(&self) -> Option<T> {
        if self.is_valid() {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

/// Кеш цен активов с автоматической инвалидацией
struct PriceCache {
    cache: HashMap<String, CachedValue<f64>>,
    default_ttl: Duration,
    hits: u64,
    misses: u64,
}

impl PriceCache {
    fn new(ttl_seconds: u64) -> Self {
        PriceCache {
            cache: HashMap::new(),
            default_ttl: Duration::from_secs(ttl_seconds),
            hits: 0,
            misses: 0,
        }
    }

    /// Получить цену из кеша или вычислить
    fn get_or_fetch<F>(&mut self, symbol: &str, fetch_fn: F) -> f64
    where
        F: FnOnce() -> f64,
    {
        // Пробуем получить из кеша
        if let Some(cached) = self.cache.get(symbol) {
            if let Some(price) = cached.get() {
                self.hits += 1;
                return price;
            }
        }

        // Кеш пуст или устарел — получаем свежие данные
        self.misses += 1;
        let price = fetch_fn();

        // Сохраняем в кеш
        self.cache.insert(
            symbol.to_string(),
            CachedValue::new(price, self.default_ttl),
        );

        price
    }

    /// Принудительная инвалидация
    fn invalidate(&mut self, symbol: &str) {
        self.cache.remove(symbol);
    }

    /// Очистка устаревших записей
    fn cleanup(&mut self) {
        self.cache.retain(|_, v| v.is_valid());
    }

    /// Статистика кеша
    fn stats(&self) -> (u64, u64, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }
}

fn main() {
    let mut cache = PriceCache::new(5); // TTL = 5 секунд

    // Симулируем получение цены с биржи
    let fetch_btc_price = || {
        println!("  [API] Запрос цены BTC с биржи...");
        42500.0
    };

    println!("=== Тест кеширования цен ===\n");

    // Первый запрос — промах кеша
    let price1 = cache.get_or_fetch("BTC", fetch_btc_price);
    println!("Цена BTC: ${:.2}\n", price1);

    // Повторные запросы — попадание в кеш
    for i in 2..=5 {
        let price = cache.get_or_fetch("BTC", fetch_btc_price);
        println!("Запрос #{}: ${:.2} (из кеша)", i, price);
    }

    let (hits, misses, rate) = cache.stats();
    println!("\n=== Статистика ===");
    println!("Попаданий: {}", hits);
    println!("Промахов: {}", misses);
    println!("Эффективность: {:.1}%", rate);
}
```

## Многоуровневый кеш

В торговых системах часто используется многоуровневое кеширование:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Уровень кеша
#[derive(Debug, Clone, Copy)]
enum CacheLevel {
    L1Memory,    // Быстрый, маленький (локальная память)
    L2Shared,    // Средний (Redis, общая память)
    L3Database,  // Медленный, большой (база данных)
}

struct CacheEntry {
    value: f64,
    created_at: Instant,
    ttl: Duration,
    level: CacheLevel,
}

impl CacheEntry {
    fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }
}

struct MultiLevelCache {
    l1: HashMap<String, CacheEntry>,
    l2: HashMap<String, CacheEntry>,
    l1_ttl: Duration,
    l2_ttl: Duration,
}

impl MultiLevelCache {
    fn new() -> Self {
        MultiLevelCache {
            l1: HashMap::new(),
            l2: HashMap::new(),
            l1_ttl: Duration::from_millis(100),  // 100ms для горячих данных
            l2_ttl: Duration::from_secs(5),      // 5 секунд для тёплых данных
        }
    }

    fn get(&mut self, key: &str) -> Option<(f64, CacheLevel)> {
        // Проверяем L1 (самый быстрый)
        if let Some(entry) = self.l1.get(key) {
            if entry.is_valid() {
                return Some((entry.value, CacheLevel::L1Memory));
            }
        }

        // Проверяем L2
        if let Some(entry) = self.l2.get(key) {
            if entry.is_valid() {
                // Продвигаем в L1
                self.l1.insert(key.to_string(), CacheEntry {
                    value: entry.value,
                    created_at: Instant::now(),
                    ttl: self.l1_ttl,
                    level: CacheLevel::L1Memory,
                });
                return Some((entry.value, CacheLevel::L2Shared));
            }
        }

        None
    }

    fn set(&mut self, key: &str, value: f64) {
        // Сохраняем в оба уровня
        let now = Instant::now();

        self.l1.insert(key.to_string(), CacheEntry {
            value,
            created_at: now,
            ttl: self.l1_ttl,
            level: CacheLevel::L1Memory,
        });

        self.l2.insert(key.to_string(), CacheEntry {
            value,
            created_at: now,
            ttl: self.l2_ttl,
            level: CacheLevel::L2Shared,
        });
    }

    fn get_or_fetch<F>(&mut self, key: &str, fetch: F) -> (f64, CacheLevel)
    where
        F: FnOnce() -> f64,
    {
        if let Some((value, level)) = self.get(key) {
            return (value, level);
        }

        let value = fetch();
        self.set(key, value);
        (value, CacheLevel::L3Database)
    }
}

fn main() {
    let mut cache = MultiLevelCache::new();

    println!("=== Многоуровневый кеш ===\n");

    // Первый запрос — L3 (база данных)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || {
        println!("[L3] Загрузка из базы данных...");
        42500.0
    });
    println!("BTC: ${:.2} (уровень: {:?})\n", price, level);

    // Второй запрос — L1 (память)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || 0.0);
    println!("BTC: ${:.2} (уровень: {:?})", price, level);

    // Ждём истечения L1
    println!("\nОжидаем 150ms...");
    std::thread::sleep(Duration::from_millis(150));

    // Третий запрос — L2 (после истечения L1)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || 0.0);
    println!("BTC: ${:.2} (уровень: {:?})", price, level);
}
```

## Кеширование с учётом версий

Для торговых данных важно отслеживать версии:

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Версионированная запись в кеше
#[derive(Clone)]
struct VersionedEntry<T> {
    value: T,
    version: u64,
    updated_at: Instant,
}

/// Кеш книги ордеров с версионированием
struct OrderBookCache {
    cache: HashMap<String, VersionedEntry<OrderBook>>,
    current_versions: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
struct OrderBook {
    symbol: String,
    bids: Vec<(f64, f64)>,  // (цена, объём)
    asks: Vec<(f64, f64)>,
}

impl OrderBookCache {
    fn new() -> Self {
        OrderBookCache {
            cache: HashMap::new(),
            current_versions: HashMap::new(),
        }
    }

    /// Обновить книгу ордеров (только если версия новее)
    fn update(&mut self, symbol: &str, book: OrderBook, version: u64) -> bool {
        let current = self.current_versions.get(symbol).copied().unwrap_or(0);

        if version > current {
            self.cache.insert(symbol.to_string(), VersionedEntry {
                value: book,
                version,
                updated_at: Instant::now(),
            });
            self.current_versions.insert(symbol.to_string(), version);
            true
        } else {
            false  // Пропускаем устаревшее обновление
        }
    }

    /// Получить книгу ордеров
    fn get(&self, symbol: &str) -> Option<&OrderBook> {
        self.cache.get(symbol).map(|e| &e.value)
    }

    /// Получить с проверкой версии
    fn get_if_newer(&self, symbol: &str, min_version: u64) -> Option<&OrderBook> {
        self.cache.get(symbol).and_then(|entry| {
            if entry.version > min_version {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    /// Текущая версия
    fn version(&self, symbol: &str) -> u64 {
        self.current_versions.get(symbol).copied().unwrap_or(0)
    }
}

fn main() {
    let mut cache = OrderBookCache::new();

    println!("=== Версионированный кеш книги ордеров ===\n");

    // Первое обновление
    let book_v1 = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42500.0, 1.5), (42490.0, 2.0)],
        asks: vec![(42510.0, 1.0), (42520.0, 3.0)],
    };

    let updated = cache.update("BTCUSDT", book_v1, 1);
    println!("Обновление v1: {}", if updated { "принято" } else { "отклонено" });

    // Попытка обновить старой версией (должна быть отклонена)
    let book_old = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42000.0, 1.0)],
        asks: vec![(43000.0, 1.0)],
    };

    let updated = cache.update("BTCUSDT", book_old, 0);
    println!("Обновление v0: {}", if updated { "принято" } else { "отклонено" });

    // Новое обновление с большей версией
    let book_v2 = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42505.0, 2.0), (42495.0, 1.5)],
        asks: vec![(42515.0, 1.5), (42525.0, 2.5)],
    };

    let updated = cache.update("BTCUSDT", book_v2, 2);
    println!("Обновление v2: {}", if updated { "принято" } else { "отклонено" });

    // Получаем актуальные данные
    if let Some(book) = cache.get("BTCUSDT") {
        println!("\nАктуальная книга ордеров:");
        println!("  Лучший bid: ${:.2}", book.bids[0].0);
        println!("  Лучший ask: ${:.2}", book.asks[0].0);
        println!("  Версия: {}", cache.version("BTCUSDT"));
    }
}
```

## Стратегии инвалидации кеша

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Стратегия инвалидации
#[derive(Debug, Clone)]
enum InvalidationStrategy {
    /// Инвалидация по времени (TTL)
    TimeToLive(Duration),
    /// Инвалидация по количеству обращений
    MaxHits(u32),
    /// Инвалидация по событию (например, новая сделка)
    EventBased,
    /// Никогда не инвалидировать (для статических данных)
    Never,
}

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    hits: u32,
    strategy: InvalidationStrategy,
}

impl<T: Clone> CacheEntry<T> {
    fn is_valid(&self) -> bool {
        match &self.strategy {
            InvalidationStrategy::TimeToLive(ttl) => {
                self.created_at.elapsed() < *ttl
            }
            InvalidationStrategy::MaxHits(max) => {
                self.hits < *max
            }
            InvalidationStrategy::EventBased => {
                true  // Управляется внешне
            }
            InvalidationStrategy::Never => {
                true
            }
        }
    }

    fn access(&mut self) -> Option<T> {
        if self.is_valid() {
            self.hits += 1;
            Some(self.value.clone())
        } else {
            None
        }
    }
}

struct TradingCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
}

impl<T: Clone> TradingCache<T> {
    fn new() -> Self {
        TradingCache {
            entries: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: T, strategy: InvalidationStrategy) {
        self.entries.insert(key, CacheEntry {
            value,
            created_at: Instant::now(),
            hits: 0,
            strategy,
        });
    }

    fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            return entry.access();
        }
        None
    }

    /// Инвалидация по событию (для стратегии EventBased)
    fn invalidate_by_event(&mut self, pattern: &str) {
        self.entries.retain(|key, _| !key.contains(pattern));
    }

    fn cleanup(&mut self) {
        self.entries.retain(|_, entry| entry.is_valid());
    }
}

fn main() {
    let mut cache: TradingCache<f64> = TradingCache::new();

    println!("=== Стратегии инвалидации ===\n");

    // 1. TTL стратегия для цен (обновляются каждые 100ms)
    cache.set(
        "price:BTCUSDT".to_string(),
        42500.0,
        InvalidationStrategy::TimeToLive(Duration::from_millis(100)),
    );

    // 2. MaxHits для дорогих вычислений (кеш на 10 обращений)
    cache.set(
        "risk:portfolio".to_string(),
        0.15,
        InvalidationStrategy::MaxHits(10),
    );

    // 3. EventBased для данных, зависящих от сделок
    cache.set(
        "volume:BTCUSDT:1h".to_string(),
        1500.5,
        InvalidationStrategy::EventBased,
    );

    // 4. Never для статических данных
    cache.set(
        "config:min_order_size".to_string(),
        0.001,
        InvalidationStrategy::Never,
    );

    // Тестируем TTL
    println!("1. Тест TTL:");
    if let Some(price) = cache.get("price:BTCUSDT") {
        println!("   Цена (свежая): ${:.2}", price);
    }
    std::thread::sleep(Duration::from_millis(150));
    match cache.get("price:BTCUSDT") {
        Some(p) => println!("   Цена (после TTL): ${:.2}", p),
        None => println!("   Цена (после TTL): кеш истёк"),
    }

    // Тестируем MaxHits
    println!("\n2. Тест MaxHits:");
    for i in 1..=12 {
        match cache.get("risk:portfolio") {
            Some(risk) => println!("   Запрос #{}: риск = {:.2}%", i, risk * 100.0),
            None => println!("   Запрос #{}: кеш исчерпан", i),
        }
    }

    // Тестируем EventBased
    println!("\n3. Тест EventBased:");
    if let Some(volume) = cache.get("volume:BTCUSDT:1h") {
        println!("   Объём до сделки: {:.2}", volume);
    }

    // Симулируем новую сделку — инвалидируем связанные данные
    println!("   [EVENT] Новая сделка по BTCUSDT");
    cache.invalidate_by_event("BTCUSDT");

    match cache.get("volume:BTCUSDT:1h") {
        Some(v) => println!("   Объём после сделки: {:.2}", v),
        None => println!("   Объём после сделки: кеш инвалидирован"),
    }

    // Тестируем Never
    println!("\n4. Тест Never:");
    for _ in 0..100 {
        let _ = cache.get("config:min_order_size");
    }
    if let Some(size) = cache.get("config:min_order_size") {
        println!("   Мин. размер ордера (после 100 запросов): {}", size);
    }
}
```

## Кеширование с Write-Through и Write-Back

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Режим записи в кеш
#[derive(Debug, Clone, Copy)]
enum WriteMode {
    /// Синхронная запись в хранилище
    WriteThrough,
    /// Отложенная запись (буферизация)
    WriteBack,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct PositionCache {
    cache: HashMap<String, Position>,
    dirty: HashMap<String, Instant>,  // Изменённые записи для WriteBack
    mode: WriteMode,
}

impl PositionCache {
    fn new(mode: WriteMode) -> Self {
        PositionCache {
            cache: HashMap::new(),
            dirty: HashMap::new(),
            mode,
        }
    }

    fn update_position(&mut self, position: Position) {
        let symbol = position.symbol.clone();

        match self.mode {
            WriteMode::WriteThrough => {
                // Сначала записываем в "базу данных"
                println!("[WRITE-THROUGH] Сохраняем {} в БД", symbol);
                self.simulate_db_write(&position);
                // Затем обновляем кеш
                self.cache.insert(symbol, position);
            }
            WriteMode::WriteBack => {
                // Только обновляем кеш
                println!("[WRITE-BACK] Обновляем {} в кеше", symbol);
                self.cache.insert(symbol.clone(), position);
                // Помечаем как "грязную" запись
                self.dirty.insert(symbol, Instant::now());
            }
        }
    }

    fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.cache.get(symbol)
    }

    /// Сброс "грязных" записей в хранилище (для WriteBack)
    fn flush(&mut self) {
        if matches!(self.mode, WriteMode::WriteBack) {
            println!("\n[FLUSH] Сохраняем {} изменённых записей", self.dirty.len());
            for (symbol, _) in self.dirty.drain() {
                if let Some(position) = self.cache.get(&symbol) {
                    self.simulate_db_write(position);
                }
            }
        }
    }

    fn simulate_db_write(&self, position: &Position) {
        println!(
            "  -> БД: {} qty={:.4} @ ${:.2}",
            position.symbol, position.quantity, position.entry_price
        );
    }

    fn dirty_count(&self) -> usize {
        self.dirty.len()
    }
}

fn main() {
    println!("=== Write-Through vs Write-Back ===\n");

    // Write-Through: каждое изменение сразу идёт в БД
    println!("--- Write-Through режим ---");
    let mut wt_cache = PositionCache::new(WriteMode::WriteThrough);

    wt_cache.update_position(Position {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.5,
        entry_price: 42500.0,
    });

    wt_cache.update_position(Position {
        symbol: "ETHUSDT".to_string(),
        quantity: 5.0,
        entry_price: 2500.0,
    });

    // Write-Back: изменения накапливаются в кеше
    println!("\n--- Write-Back режим ---");
    let mut wb_cache = PositionCache::new(WriteMode::WriteBack);

    wb_cache.update_position(Position {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.5,
        entry_price: 42500.0,
    });

    wb_cache.update_position(Position {
        symbol: "ETHUSDT".to_string(),
        quantity: 5.0,
        entry_price: 2500.0,
    });

    wb_cache.update_position(Position {
        symbol: "SOLUSDT".to_string(),
        quantity: 100.0,
        entry_price: 100.0,
    });

    println!("\nНакоплено изменений: {}", wb_cache.dirty_count());

    // Периодический сброс в БД
    wb_cache.flush();
}
```

## Практический пример: кеш рыночных данных

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last_trade: f64,
    volume_24h: f64,
    timestamp: u64,
}

#[derive(Clone, Debug)]
struct OHLCV {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct MarketDataCache {
    ticks: HashMap<String, (MarketTick, Instant)>,
    candles: HashMap<String, (Vec<OHLCV>, Instant)>,
    tick_ttl: Duration,
    candle_ttl: Duration,

    // Статистика
    tick_hits: u64,
    tick_misses: u64,
    candle_hits: u64,
    candle_misses: u64,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            ticks: HashMap::new(),
            candles: HashMap::new(),
            tick_ttl: Duration::from_millis(50),   // Тики устаревают быстро
            candle_ttl: Duration::from_secs(60),   // Свечи живут дольше
            tick_hits: 0,
            tick_misses: 0,
            candle_hits: 0,
            candle_misses: 0,
        }
    }

    /// Получить последний тик
    fn get_tick(&mut self, symbol: &str) -> Option<MarketTick> {
        if let Some((tick, cached_at)) = self.ticks.get(symbol) {
            if cached_at.elapsed() < self.tick_ttl {
                self.tick_hits += 1;
                return Some(tick.clone());
            }
        }
        self.tick_misses += 1;
        None
    }

    /// Обновить тик
    fn update_tick(&mut self, tick: MarketTick) {
        let symbol = tick.symbol.clone();
        self.ticks.insert(symbol, (tick, Instant::now()));
    }

    /// Получить свечи
    fn get_candles(&mut self, symbol: &str, timeframe: &str) -> Option<Vec<OHLCV>> {
        let key = format!("{}:{}", symbol, timeframe);
        if let Some((candles, cached_at)) = self.candles.get(&key) {
            if cached_at.elapsed() < self.candle_ttl {
                self.candle_hits += 1;
                return Some(candles.clone());
            }
        }
        self.candle_misses += 1;
        None
    }

    /// Обновить свечи
    fn update_candles(&mut self, symbol: &str, timeframe: &str, candles: Vec<OHLCV>) {
        let key = format!("{}:{}", symbol, timeframe);
        self.candles.insert(key, (candles, Instant::now()));
    }

    /// Инвалидация при новой сделке
    fn on_trade(&mut self, symbol: &str) {
        // Инвалидируем все связанные свечи
        let prefix = format!("{}:", symbol);
        self.candles.retain(|k, _| !k.starts_with(&prefix));
    }

    fn print_stats(&self) {
        let tick_total = self.tick_hits + self.tick_misses;
        let candle_total = self.candle_hits + self.candle_misses;

        println!("\n=== Статистика кеша ===");
        println!("Тики:");
        println!("  Попаданий: {}", self.tick_hits);
        println!("  Промахов: {}", self.tick_misses);
        if tick_total > 0 {
            println!("  Эффективность: {:.1}%",
                     self.tick_hits as f64 / tick_total as f64 * 100.0);
        }

        println!("Свечи:");
        println!("  Попаданий: {}", self.candle_hits);
        println!("  Промахов: {}", self.candle_misses);
        if candle_total > 0 {
            println!("  Эффективность: {:.1}%",
                     self.candle_hits as f64 / candle_total as f64 * 100.0);
        }
    }
}

fn main() {
    let mut cache = MarketDataCache::new();

    println!("=== Кеш рыночных данных ===\n");

    // Симулируем поток тиков
    for i in 0..5 {
        let tick = MarketTick {
            symbol: "BTCUSDT".to_string(),
            bid: 42500.0 + i as f64,
            ask: 42501.0 + i as f64,
            last_trade: 42500.5 + i as f64,
            volume_24h: 15000.0,
            timestamp: 1000 + i,
        };
        cache.update_tick(tick);
    }

    // Запросы к кешу тиков
    println!("Запросы тиков:");
    for _ in 0..10 {
        match cache.get_tick("BTCUSDT") {
            Some(tick) => println!("  BTC bid: ${:.2}", tick.bid),
            None => println!("  Кеш устарел, нужен запрос к бирже"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    // Загружаем свечи
    let candles = vec![
        OHLCV { open: 42000.0, high: 42500.0, low: 41800.0, close: 42400.0, volume: 100.0 },
        OHLCV { open: 42400.0, high: 42600.0, low: 42200.0, close: 42500.0, volume: 150.0 },
    ];
    cache.update_candles("BTCUSDT", "1h", candles);

    println!("\nЗапросы свечей:");
    for i in 0..5 {
        match cache.get_candles("BTCUSDT", "1h") {
            Some(c) => println!("  Запрос #{}: {} свечей", i + 1, c.len()),
            None => println!("  Запрос #{}: промах", i + 1),
        }
    }

    // Симулируем новую сделку
    println!("\n[EVENT] Новая сделка по BTCUSDT");
    cache.on_trade("BTCUSDT");

    match cache.get_candles("BTCUSDT", "1h") {
        Some(_) => println!("Свечи: в кеше"),
        None => println!("Свечи: инвалидированы"),
    }

    cache.print_stats();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Кеширование результатов** | Хранение вычисленных значений для повторного использования |
| **TTL (Time To Live)** | Автоматическая инвалидация по времени |
| **Многоуровневый кеш** | L1 (память) → L2 (shared) → L3 (БД) |
| **Версионирование** | Отслеживание актуальности данных |
| **Write-Through** | Синхронная запись в кеш и хранилище |
| **Write-Back** | Отложенная запись с буферизацией |
| **Event-based инвалидация** | Сброс кеша по событиям |

## Практические задания

1. **Кеш ордеров с приоритетами**: Реализуй кеш, где ордера с большим объёмом имеют больший TTL.

2. **Кеш с прогревом**: Создай систему предзагрузки часто используемых данных при старте.

3. **Распределённый кеш**: Реализуй простой протокол синхронизации кеша между несколькими узлами.

## Домашнее задание

1. **Адаптивный TTL**: Реализуй кеш, который автоматически подстраивает TTL под волатильность рынка — при высокой волатильности TTL уменьшается.

2. **Кеш с предсказанием**: Создай систему, которая предугадывает, какие данные понадобятся трейдеру, и загружает их заранее.

3. **Метрики кеша**: Реализуй dashboard со статистикой: hit rate, latency, memory usage, eviction rate.

4. **Гибридный кеш**: Объедини несколько стратегий инвалидации (TTL + Event + Version) в единый умный кеш для торговой системы.

## Навигация

[← Предыдущий день](../320-*/ru.md) | [Следующий день →](../322-*/ru.md)
