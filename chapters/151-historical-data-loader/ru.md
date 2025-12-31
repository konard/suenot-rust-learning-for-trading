# День 151: Проект: Загрузчик исторических данных

## Аналогия из трейдинга

Представь, что ты аналитик в торговой фирме. Каждое утро тебе нужно:
- Загрузить исторические данные с разных бирж (CSV, JSON)
- Преобразовать их в единый формат (OHLCV)
- Сохранить в локальную базу данных
- Логировать все операции для аудита
- Обработать огромные файлы без переполнения памяти

Это именно то, что делает наш **Historical Data Loader** — он объединяет все знания этого месяца в один полезный инструмент!

## Обзор проекта

В этом проекте мы создадим CLI-приложение для загрузки и обработки исторических торговых данных.

### Возможности

- Загрузка данных из CSV и JSON файлов
- Поддержка разных форматов бирж (Binance, Coinbase)
- Кэширование результатов для повторных запросов
- Потоковая обработка больших файлов
- Логирование с разными уровнями
- Конфигурация через файлы и переменные окружения

## Структура проекта

```
historical_data_loader/
├── Cargo.toml
├── config.toml
├── .env
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── loader.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── csv_parser.rs
│   │   └── json_parser.rs
│   ├── cache.rs
│   └── models.rs
└── data/
    ├── btc_hourly.csv
    └── eth_trades.json
```

## Cargo.toml — зависимости

```toml
[package]
name = "historical_data_loader"
version = "0.1.0"
edition = "2021"

[dependencies]
# Сериализация
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
csv = "1.3"

# Дата и время
chrono = { version = "0.4", features = ["serde"] }

# Конфигурация
toml = "0.8"
dotenvy = "0.15"

# CLI аргументы
clap = { version = "4.4", features = ["derive"] }

# Логирование
log = "0.4"
env_logger = "0.10"

# Утилиты
thiserror = "1.0"
```

## Модели данных — models.rs

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OHLCV (Open, High, Low, Close, Volume) — стандартный формат свечей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn price_change(&self) -> f64 {
        self.close - self.open
    }

    pub fn price_change_percent(&self) -> f64 {
        if self.open == 0.0 {
            return 0.0;
        }
        ((self.close - self.open) / self.open) * 100.0
    }

    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

/// Формат данных из Binance CSV
#[derive(Debug, Deserialize)]
pub struct BinanceKline {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_volume: String,
    pub trades: u64,
    pub taker_buy_base: String,
    pub taker_buy_quote: String,
    pub ignore: String,
}

/// Формат данных из Coinbase JSON
#[derive(Debug, Deserialize)]
pub struct CoinbaseTrade {
    pub time: String,
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub side: String,
}

/// Статистика по загруженным данным
#[derive(Debug, Default)]
pub struct LoadStats {
    pub records_loaded: usize,
    pub errors: usize,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
    pub load_time_ms: u128,
}

impl LoadStats {
    pub fn update(&mut self, candle: &Candle) {
        self.records_loaded += 1;

        match self.min_timestamp {
            Some(min) if candle.timestamp < min => {
                self.min_timestamp = Some(candle.timestamp);
            }
            None => self.min_timestamp = Some(candle.timestamp),
            _ => {}
        }

        match self.max_timestamp {
            Some(max) if candle.timestamp > max => {
                self.max_timestamp = Some(candle.timestamp);
            }
            None => self.max_timestamp = Some(candle.timestamp),
            _ => {}
        }
    }
}
```

## Конфигурация — config.rs

```rust
use serde::Deserialize;
use std::path::PathBuf;
use std::env;
use std::fs;

/// Основная конфигурация приложения
#[derive(Debug, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_level: String,
    pub exchange: ExchangeConfig,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeConfig {
    pub default_exchange: String,
    pub binance_api_key: Option<String>,
    pub coinbase_api_key: Option<String>,
}

impl Config {
    /// Загружает конфигурацию из TOML файла и переменных окружения
    pub fn load(config_path: &str) -> Result<Self, ConfigError> {
        // Загружаем .env файл
        dotenvy::dotenv().ok();

        // Читаем TOML файл
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;

        let mut config: Config = toml::from_str(&config_content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        // Переопределяем значения из переменных окружения
        if let Ok(data_dir) = env::var("DATA_DIR") {
            config.data_dir = PathBuf::from(data_dir);
        }

        if let Ok(cache_dir) = env::var("CACHE_DIR") {
            config.cache_dir = PathBuf::from(cache_dir);
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }

        // Секретные ключи только из окружения!
        config.exchange.binance_api_key = env::var("BINANCE_API_KEY").ok();
        config.exchange.coinbase_api_key = env::var("COINBASE_API_KEY").ok();

        Ok(config)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileRead(String),
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::Parse(e) => write!(f, "Failed to parse config: {}", e),
        }
    }
}
```

Файл `config.toml`:

```toml
data_dir = "./data"
cache_dir = "./cache"
log_level = "info"

[exchange]
default_exchange = "binance"
```

Файл `.env`:

```bash
# Не коммитьте этот файл в git!
DATA_DIR=./data
CACHE_DIR=./cache
LOG_LEVEL=debug
BINANCE_API_KEY=your_binance_key_here
COINBASE_API_KEY=your_coinbase_key_here
```

## CSV парсер — parser/csv_parser.rs

```rust
use crate::models::{BinanceKline, Candle, LoadStats};
use chrono::{DateTime, TimeZone, Utc};
use csv::ReaderBuilder;
use log::{debug, error, info, warn};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

/// Загружает данные из CSV файла Binance
pub fn load_binance_csv(
    path: &Path,
    symbol: &str,
) -> Result<(Vec<Candle>, LoadStats), CsvError> {
    let start = Instant::now();
    info!("Loading CSV file: {:?}", path);

    let file = File::open(path)
        .map_err(|e| CsvError::FileOpen(e.to_string()))?;

    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);

    let mut candles = Vec::new();
    let mut stats = LoadStats::default();

    for (line_num, result) in csv_reader.deserialize().enumerate() {
        match result {
            Ok(kline) => {
                match convert_binance_kline(kline, symbol) {
                    Ok(candle) => {
                        stats.update(&candle);
                        candles.push(candle);

                        if candles.len() % 10000 == 0 {
                            debug!("Processed {} records...", candles.len());
                        }
                    }
                    Err(e) => {
                        warn!("Line {}: conversion error: {}", line_num + 1, e);
                        stats.errors += 1;
                    }
                }
            }
            Err(e) => {
                error!("Line {}: parse error: {}", line_num + 1, e);
                stats.errors += 1;
            }
        }
    }

    stats.load_time_ms = start.elapsed().as_millis();

    info!(
        "Loaded {} candles in {}ms ({} errors)",
        candles.len(),
        stats.load_time_ms,
        stats.errors
    );

    Ok((candles, stats))
}

/// Потоковая загрузка для больших файлов
pub fn load_binance_csv_streaming<F>(
    path: &Path,
    symbol: &str,
    mut processor: F,
) -> Result<LoadStats, CsvError>
where
    F: FnMut(Candle) -> bool, // Возвращает false чтобы остановить
{
    let start = Instant::now();
    info!("Streaming CSV file: {:?}", path);

    let file = File::open(path)
        .map_err(|e| CsvError::FileOpen(e.to_string()))?;

    let reader = BufReader::with_capacity(64 * 1024, file); // 64KB буфер
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);

    let mut stats = LoadStats::default();

    for result in csv_reader.deserialize::<BinanceKline>() {
        match result {
            Ok(kline) => {
                match convert_binance_kline(kline, symbol) {
                    Ok(candle) => {
                        stats.update(&candle);
                        if !processor(candle) {
                            break;
                        }
                    }
                    Err(_) => stats.errors += 1,
                }
            }
            Err(_) => stats.errors += 1,
        }
    }

    stats.load_time_ms = start.elapsed().as_millis();
    Ok(stats)
}

fn convert_binance_kline(kline: BinanceKline, symbol: &str) -> Result<Candle, String> {
    let timestamp = Utc
        .timestamp_millis_opt(kline.open_time)
        .single()
        .ok_or("Invalid timestamp")?;

    Ok(Candle {
        symbol: symbol.to_string(),
        timestamp,
        open: kline.open.parse().map_err(|_| "Invalid open price")?,
        high: kline.high.parse().map_err(|_| "Invalid high price")?,
        low: kline.low.parse().map_err(|_| "Invalid low price")?,
        close: kline.close.parse().map_err(|_| "Invalid close price")?,
        volume: kline.volume.parse().map_err(|_| "Invalid volume")?,
    })
}

#[derive(Debug)]
pub enum CsvError {
    FileOpen(String),
    Parse(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::FileOpen(e) => write!(f, "Failed to open file: {}", e),
            CsvError::Parse(e) => write!(f, "Failed to parse CSV: {}", e),
        }
    }
}
```

## JSON парсер — parser/json_parser.rs

```rust
use crate::models::{Candle, CoinbaseTrade, LoadStats};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

/// Загружает трейды из JSON файла Coinbase
pub fn load_coinbase_json(
    path: &Path,
    symbol: &str,
) -> Result<(Vec<CoinbaseTrade>, LoadStats), JsonError> {
    let start = Instant::now();
    info!("Loading JSON file: {:?}", path);

    let file = File::open(path)
        .map_err(|e| JsonError::FileOpen(e.to_string()))?;

    let reader = BufReader::new(file);
    let trades: Vec<CoinbaseTrade> = serde_json::from_reader(reader)
        .map_err(|e| JsonError::Parse(e.to_string()))?;

    let mut stats = LoadStats::default();
    stats.records_loaded = trades.len();
    stats.load_time_ms = start.elapsed().as_millis();

    info!(
        "Loaded {} trades in {}ms",
        trades.len(),
        stats.load_time_ms
    );

    Ok((trades, stats))
}

/// Агрегирует трейды в свечи по временному интервалу
pub fn aggregate_trades_to_candles(
    trades: &[CoinbaseTrade],
    symbol: &str,
    interval_minutes: u32,
) -> Vec<Candle> {
    let mut candle_map: HashMap<i64, CandleBuilder> = HashMap::new();
    let interval_secs = (interval_minutes * 60) as i64;

    for trade in trades {
        let trade_time: DateTime<Utc> = trade.time.parse().unwrap_or_else(|_| Utc::now());
        let bucket = (trade_time.timestamp() / interval_secs) * interval_secs;

        let price: f64 = trade.price.parse().unwrap_or(0.0);
        let size: f64 = trade.size.parse().unwrap_or(0.0);

        candle_map
            .entry(bucket)
            .or_insert_with(|| CandleBuilder::new(price, trade_time))
            .update(price, size);
    }

    let mut candles: Vec<Candle> = candle_map
        .into_iter()
        .map(|(_, builder)| builder.build(symbol))
        .collect();

    candles.sort_by_key(|c| c.timestamp);

    debug!("Aggregated {} candles from {} trades", candles.len(), trades.len());
    candles
}

struct CandleBuilder {
    timestamp: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl CandleBuilder {
    fn new(price: f64, timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
    }

    fn build(self, symbol: &str) -> Candle {
        Candle {
            symbol: symbol.to_string(),
            timestamp: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }
}

/// Парсит вложенный JSON ответ биржи
pub fn parse_exchange_response(json: &str) -> Result<Vec<Value>, JsonError> {
    let response: Value = serde_json::from_str(json)
        .map_err(|e| JsonError::Parse(e.to_string()))?;

    // Обрабатываем разные структуры ответов
    if let Some(data) = response.get("data") {
        if let Some(arr) = data.as_array() {
            return Ok(arr.clone());
        }
    }

    if let Some(result) = response.get("result") {
        if let Some(arr) = result.as_array() {
            return Ok(arr.clone());
        }
    }

    if let Some(arr) = response.as_array() {
        return Ok(arr.clone());
    }

    Err(JsonError::Parse("Unknown response structure".to_string()))
}

#[derive(Debug)]
pub enum JsonError {
    FileOpen(String),
    Parse(String),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::FileOpen(e) => write!(f, "Failed to open file: {}", e),
            JsonError::Parse(e) => write!(f, "Failed to parse JSON: {}", e),
        }
    }
}
```

## Модуль парсеров — parser/mod.rs

```rust
pub mod csv_parser;
pub mod json_parser;

pub use csv_parser::*;
pub use json_parser::*;
```

## Кэширование — cache.rs

```rust
use crate::models::Candle;
use chrono::{DateTime, Utc, Duration};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Кэш для результатов загрузки
pub struct DataCache {
    cache_dir: PathBuf,
    memory_cache: HashMap<String, CacheEntry>,
    ttl_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    data: Vec<Candle>,
    created_at: DateTime<Utc>,
    source_path: String,
    source_modified: i64,
}

impl DataCache {
    pub fn new(cache_dir: PathBuf, ttl_minutes: i64) -> Self {
        // Создаём директорию кэша если не существует
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).ok();
        }

        Self {
            cache_dir,
            memory_cache: HashMap::new(),
            ttl_minutes,
        }
    }

    /// Генерирует ключ кэша для файла
    fn cache_key(&self, source_path: &Path, symbol: &str) -> String {
        let mut hasher = DefaultHasher::new();
        source_path.to_string_lossy().hash(&mut hasher);
        symbol.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Путь к файлу кэша
    fn cache_file_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache.json", key))
    }

    /// Проверяет и возвращает данные из кэша
    pub fn get(&mut self, source_path: &Path, symbol: &str) -> Option<Vec<Candle>> {
        let key = self.cache_key(source_path, symbol);

        // Сначала проверяем память
        if let Some(entry) = self.memory_cache.get(&key) {
            if self.is_valid(entry, source_path) {
                debug!("Cache hit (memory): {}", key);
                return Some(entry.data.clone());
            }
        }

        // Затем проверяем диск
        let cache_file = self.cache_file_path(&key);
        if cache_file.exists() {
            if let Ok(entry) = self.load_from_disk(&cache_file) {
                if self.is_valid(&entry, source_path) {
                    debug!("Cache hit (disk): {}", key);
                    // Загружаем в память для быстрого доступа
                    let data = entry.data.clone();
                    self.memory_cache.insert(key, entry);
                    return Some(data);
                }
            }
        }

        debug!("Cache miss: {}", key);
        None
    }

    /// Сохраняет данные в кэш
    pub fn set(&mut self, source_path: &Path, symbol: &str, data: Vec<Candle>) {
        let key = self.cache_key(source_path, symbol);

        let source_modified = fs::metadata(source_path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64)
            .unwrap_or(0);

        let entry = CacheEntry {
            data: data.clone(),
            created_at: Utc::now(),
            source_path: source_path.to_string_lossy().to_string(),
            source_modified,
        };

        // Сохраняем в память
        self.memory_cache.insert(key.clone(), entry.clone());

        // Сохраняем на диск
        let cache_file = self.cache_file_path(&key);
        if let Err(e) = self.save_to_disk(&cache_file, &entry) {
            warn!("Failed to save cache to disk: {}", e);
        } else {
            info!("Cached {} candles for {}", data.len(), symbol);
        }
    }

    /// Проверяет валидность записи кэша
    fn is_valid(&self, entry: &CacheEntry, source_path: &Path) -> bool {
        // Проверяем TTL
        let age = Utc::now() - entry.created_at;
        if age > Duration::minutes(self.ttl_minutes) {
            return false;
        }

        // Проверяем, не изменился ли исходный файл
        let current_modified = fs::metadata(source_path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64)
            .unwrap_or(0);

        entry.source_modified == current_modified
    }

    fn load_from_disk(&self, path: &Path) -> Result<CacheEntry, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn save_to_disk(&self, path: &Path, entry: &CacheEntry) -> Result<(), std::io::Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Очищает устаревший кэш
    pub fn cleanup(&self) -> Result<usize, std::io::Error> {
        let mut removed = 0;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(cache_entry) = self.load_from_disk(&path) {
                    let age = Utc::now() - cache_entry.created_at;
                    if age > Duration::minutes(self.ttl_minutes) {
                        fs::remove_file(&path)?;
                        removed += 1;
                    }
                }
            }
        }

        info!("Cleaned up {} stale cache entries", removed);
        Ok(removed)
    }
}

/// Мемоизация для дорогих вычислений
pub fn memoize<F, K, V>(cache: &mut HashMap<K, V>, key: K, compute: F) -> V
where
    K: Eq + Hash + Clone,
    V: Clone,
    F: FnOnce() -> V,
{
    if let Some(value) = cache.get(&key) {
        return value.clone();
    }

    let value = compute();
    cache.insert(key, value.clone());
    value
}
```

## Загрузчик — loader.rs

```rust
use crate::cache::DataCache;
use crate::config::Config;
use crate::models::{Candle, LoadStats};
use crate::parser::{csv_parser, json_parser};
use log::{error, info, warn};
use std::path::{Path, PathBuf};

/// Поддерживаемые форматы файлов
#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    BinanceCsv,
    CoinbaseJson,
    GenericCsv,
}

impl FileFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        let filename = path.file_name()?.to_str()?.to_lowercase();

        match extension.to_lowercase().as_str() {
            "csv" => {
                if filename.contains("binance") {
                    Some(FileFormat::BinanceCsv)
                } else {
                    Some(FileFormat::GenericCsv)
                }
            }
            "json" => Some(FileFormat::CoinbaseJson),
            _ => None,
        }
    }
}

/// Основной загрузчик данных
pub struct DataLoader {
    config: Config,
    cache: DataCache,
}

impl DataLoader {
    pub fn new(config: Config) -> Self {
        let cache = DataCache::new(
            config.cache_dir.clone(),
            60, // TTL 60 минут
        );

        Self { config, cache }
    }

    /// Загружает данные из файла с автоопределением формата
    pub fn load(&mut self, path: &Path, symbol: &str) -> Result<Vec<Candle>, LoadError> {
        // Проверяем кэш
        if let Some(cached) = self.cache.get(path, symbol) {
            info!("Using cached data for {}", symbol);
            return Ok(cached);
        }

        // Определяем формат файла
        let format = FileFormat::from_path(path)
            .ok_or_else(|| LoadError::UnsupportedFormat(
                path.to_string_lossy().to_string()
            ))?;

        // Загружаем данные
        let candles = match format {
            FileFormat::BinanceCsv | FileFormat::GenericCsv => {
                let (candles, stats) = csv_parser::load_binance_csv(path, symbol)
                    .map_err(|e| LoadError::Parse(e.to_string()))?;
                self.log_stats(&stats);
                candles
            }
            FileFormat::CoinbaseJson => {
                let (trades, stats) = json_parser::load_coinbase_json(path, symbol)
                    .map_err(|e| LoadError::Parse(e.to_string()))?;
                self.log_stats(&stats);
                json_parser::aggregate_trades_to_candles(&trades, symbol, 60)
            }
        };

        // Сохраняем в кэш
        self.cache.set(path, symbol, candles.clone());

        Ok(candles)
    }

    /// Загружает несколько файлов и объединяет данные
    pub fn load_multiple(
        &mut self,
        paths: &[PathBuf],
        symbol: &str,
    ) -> Result<Vec<Candle>, LoadError> {
        let mut all_candles = Vec::new();

        for path in paths {
            match self.load(path, symbol) {
                Ok(candles) => {
                    info!("Loaded {} candles from {:?}", candles.len(), path);
                    all_candles.extend(candles);
                }
                Err(e) => {
                    warn!("Failed to load {:?}: {}", path, e);
                }
            }
        }

        // Сортируем по времени и удаляем дубликаты
        all_candles.sort_by_key(|c| c.timestamp);
        all_candles.dedup_by_key(|c| c.timestamp);

        info!("Total: {} unique candles", all_candles.len());
        Ok(all_candles)
    }

    /// Потоковая загрузка для обработки больших файлов
    pub fn stream_process<F>(
        &self,
        path: &Path,
        symbol: &str,
        processor: F,
    ) -> Result<LoadStats, LoadError>
    where
        F: FnMut(Candle) -> bool,
    {
        csv_parser::load_binance_csv_streaming(path, symbol, processor)
            .map_err(|e| LoadError::Parse(e.to_string()))
    }

    fn log_stats(&self, stats: &LoadStats) {
        info!("Load stats:");
        info!("  Records: {}", stats.records_loaded);
        info!("  Errors: {}", stats.errors);
        info!("  Time: {}ms", stats.load_time_ms);

        if let (Some(min), Some(max)) = (stats.min_timestamp, stats.max_timestamp) {
            info!("  Date range: {} to {}",
                min.format("%Y-%m-%d %H:%M"),
                max.format("%Y-%m-%d %H:%M")
            );
        }
    }

    /// Очистка кэша
    pub fn cleanup_cache(&self) -> Result<usize, std::io::Error> {
        self.cache.cleanup()
    }
}

#[derive(Debug)]
pub enum LoadError {
    UnsupportedFormat(String),
    FileNotFound(String),
    Parse(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::UnsupportedFormat(p) => write!(f, "Unsupported format: {}", p),
            LoadError::FileNotFound(p) => write!(f, "File not found: {}", p),
            LoadError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}
```

## Главный файл — main.rs

```rust
mod cache;
mod config;
mod loader;
mod models;
mod parser;

use crate::config::Config;
use crate::loader::DataLoader;
use crate::models::Candle;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use log::{error, info, LevelFilter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "historical-data-loader")]
#[command(about = "Load and process historical trading data")]
#[command(version = "1.0")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load data from a file
    Load {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Trading symbol (e.g., BTCUSDT)
        #[arg(short, long)]
        symbol: String,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Stream process a large file
    Stream {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Trading symbol
        #[arg(short, long)]
        symbol: String,

        /// Maximum records to process
        #[arg(short, long)]
        max_records: Option<usize>,
    },

    /// Show statistics for a file
    Stats {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Trading symbol
        #[arg(short, long)]
        symbol: String,
    },

    /// Clean up stale cache entries
    CacheClean,
}

fn main() {
    // Инициализируем логирование
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();

    // Загружаем конфигурацию
    let config = match Config::load(cli.config.to_str().unwrap_or("config.toml")) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let mut loader = DataLoader::new(config);

    match cli.command {
        Commands::Load { input, symbol, output } => {
            cmd_load(&mut loader, &input, &symbol, output.as_deref());
        }
        Commands::Stream { input, symbol, max_records } => {
            cmd_stream(&loader, &input, &symbol, max_records);
        }
        Commands::Stats { input, symbol } => {
            cmd_stats(&mut loader, &input, &symbol);
        }
        Commands::CacheClean => {
            cmd_cache_clean(&loader);
        }
    }
}

fn cmd_load(loader: &mut DataLoader, input: &PathBuf, symbol: &str, output: Option<&PathBuf>) {
    info!("Loading data from {:?}", input);

    match loader.load(input, symbol) {
        Ok(candles) => {
            info!("Successfully loaded {} candles", candles.len());

            if let Some(output_path) = output {
                save_candles(&candles, output_path);
            } else {
                // Выводим первые 5 и последние 5 свечей
                print_candle_summary(&candles);
            }
        }
        Err(e) => {
            error!("Load failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_stream(loader: &DataLoader, input: &PathBuf, symbol: &str, max_records: Option<usize>) {
    info!("Streaming data from {:?}", input);

    let mut count = 0;
    let max = max_records.unwrap_or(usize::MAX);

    let mut bullish = 0;
    let mut bearish = 0;
    let mut total_volume = 0.0;

    let result = loader.stream_process(input, symbol, |candle| {
        count += 1;

        if candle.is_bullish() {
            bullish += 1;
        } else {
            bearish += 1;
        }
        total_volume += candle.volume;

        count < max
    });

    match result {
        Ok(stats) => {
            println!("\n=== Streaming Results ===");
            println!("Processed: {} candles", count);
            println!("Bullish: {} ({:.1}%)", bullish, 100.0 * bullish as f64 / count as f64);
            println!("Bearish: {} ({:.1}%)", bearish, 100.0 * bearish as f64 / count as f64);
            println!("Total volume: {:.2}", total_volume);
            println!("Time: {}ms", stats.load_time_ms);
        }
        Err(e) => {
            error!("Stream failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_stats(loader: &mut DataLoader, input: &PathBuf, symbol: &str) {
    match loader.load(input, symbol) {
        Ok(candles) => {
            calculate_and_print_stats(&candles, symbol);
        }
        Err(e) => {
            error!("Failed to load: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_cache_clean(loader: &DataLoader) {
    match loader.cleanup_cache() {
        Ok(count) => {
            info!("Cleaned {} cache entries", count);
        }
        Err(e) => {
            error!("Cache cleanup failed: {}", e);
        }
    }
}

fn print_candle_summary(candles: &[Candle]) {
    if candles.is_empty() {
        println!("No candles loaded");
        return;
    }

    println!("\n=== First 5 candles ===");
    for candle in candles.iter().take(5) {
        print_candle(candle);
    }

    if candles.len() > 10 {
        println!("\n... {} more candles ...", candles.len() - 10);
    }

    println!("\n=== Last 5 candles ===");
    for candle in candles.iter().rev().take(5).collect::<Vec<_>>().iter().rev() {
        print_candle(candle);
    }
}

fn print_candle(candle: &Candle) {
    let direction = if candle.is_bullish() { "+" } else { "-" };
    println!(
        "{} {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2} {}",
        candle.timestamp.format("%Y-%m-%d %H:%M"),
        candle.symbol,
        candle.open,
        candle.high,
        candle.low,
        candle.close,
        candle.volume,
        direction
    );
}

fn calculate_and_print_stats(candles: &[Candle], symbol: &str) {
    if candles.is_empty() {
        println!("No data for statistics");
        return;
    }

    let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let returns: Vec<f64> = prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect();

    let min_price = prices.iter().cloned().fold(f64::MAX, f64::min);
    let max_price = prices.iter().cloned().fold(f64::MIN, f64::max);
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    let total_volume: f64 = candles.iter().map(|c| c.volume).sum();
    let avg_volume = total_volume / candles.len() as f64;

    let bullish_count = candles.iter().filter(|c| c.is_bullish()).count();
    let bearish_count = candles.len() - bullish_count;

    let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|r| (r - avg_return).powi(2)).sum::<f64>()
        / returns.len() as f64;
    let volatility = variance.sqrt();

    println!("\n╔══════════════════════════════════════════╗");
    println!("║         HISTORICAL DATA STATS            ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║ Symbol:        {:>25} ║", symbol);
    println!("║ Candles:       {:>25} ║", candles.len());
    println!("║ Date range:    {:>25} ║",
        format!("{} - {}",
            candles.first().unwrap().timestamp.format("%Y-%m-%d"),
            candles.last().unwrap().timestamp.format("%Y-%m-%d")
        )
    );
    println!("╠══════════════════════════════════════════╣");
    println!("║ Min price:     {:>25.2} ║", min_price);
    println!("║ Max price:     {:>25.2} ║", max_price);
    println!("║ Avg price:     {:>25.2} ║", avg_price);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Total volume:  {:>25.2} ║", total_volume);
    println!("║ Avg volume:    {:>25.2} ║", avg_volume);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Bullish:       {:>18} ({:>4.1}%) ║", bullish_count,
        100.0 * bullish_count as f64 / candles.len() as f64);
    println!("║ Bearish:       {:>18} ({:>4.1}%) ║", bearish_count,
        100.0 * bearish_count as f64 / candles.len() as f64);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Avg return:    {:>24.4}% ║", avg_return);
    println!("║ Volatility:    {:>24.4}% ║", volatility);
    println!("╚══════════════════════════════════════════╝");
}

fn save_candles(candles: &[Candle], path: &PathBuf) {
    use std::fs::File;
    use std::io::BufWriter;

    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create output file: {}", e);
            return;
        }
    };

    let writer = BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, candles) {
        Ok(_) => info!("Saved {} candles to {:?}", candles.len(), path),
        Err(e) => error!("Failed to save: {}", e),
    }
}
```

## Запуск проекта

### Создание тестовых данных

Создадим файл `data/btc_hourly.csv` (формат Binance):

```csv
1609459200000,29000.50,29100.00,28950.00,29050.00,100.5,1609462800000,2910250.00,1500,50.2,1455125.00,0
1609462800000,29050.00,29200.00,29000.00,29150.00,120.3,1609466400000,3505485.00,1800,60.1,1752742.50,0
1609466400000,29150.00,29300.00,29100.00,29250.00,95.7,1609470000000,2801475.00,1200,47.8,1400737.50,0
```

### Команды запуска

```bash
# Загрузка данных
cargo run -- load -i data/btc_hourly.csv -s BTCUSDT

# Сохранение результата в JSON
cargo run -- load -i data/btc_hourly.csv -s BTCUSDT -o output/btc.json

# Потоковая обработка большого файла
cargo run -- stream -i data/large_file.csv -s BTCUSDT -m 10000

# Статистика
cargo run -- stats -i data/btc_hourly.csv -s BTCUSDT

# Очистка кэша
cargo run -- cache-clean

# С debug логированием
RUST_LOG=debug cargo run -- load -i data/btc_hourly.csv -s BTCUSDT
```

## Практические задания

### Задание 1: Добавьте поддержку нового формата

Добавьте парсер для формата Kraken (JSON с другой структурой):

```json
{
  "result": {
    "XXBTZUSD": [
      [1609459200, "29000.5", "29100.0", "28950.0", "29050.0", "29025.0", "100.5", 1500]
    ]
  }
}
```

### Задание 2: Реализуйте фильтрацию по дате

Добавьте параметры `--from` и `--to` для фильтрации свечей по временному диапазону:

```bash
cargo run -- load -i data/btc.csv -s BTCUSDT --from 2024-01-01 --to 2024-06-30
```

### Задание 3: Добавьте индикаторы

Расширьте команду `stats` для расчёта технических индикаторов:
- SMA (20, 50, 200)
- RSI (14)
- Bollinger Bands

### Задание 4: Реализуйте экспорт

Добавьте команду `export` для сохранения в разных форматах:

```bash
cargo run -- export -i data/btc.csv -s BTCUSDT -f parquet -o output/btc.parquet
cargo run -- export -i data/btc.csv -s BTCUSDT -f csv -o output/btc_normalized.csv
```

## Домашнее задание

1. **Обработка ошибок**: Замените все `unwrap()` на правильную обработку ошибок с помощью `Result` и `?`

2. **Параллельная загрузка**: Используйте `rayon` для параллельной загрузки нескольких файлов

3. **Compression**: Добавьте поддержку сжатых файлов (.gz, .zip) используя знания из главы 148

4. **Трейсинг**: Замените `log` на `tracing` для структурированного логирования с полями:
   ```rust
   tracing::info!(symbol = %symbol, candles = candles.len(), "Data loaded");
   ```

5. **Тесты**: Напишите unit-тесты для парсеров и интеграционные тесты для загрузчика

## Что мы применили

| Концепция | Глава | Применение |
|-----------|-------|------------|
| File I/O | 122-126 | Чтение/запись файлов |
| Serde | 127-132 | JSON сериализация |
| CSV | 133-134 | Парсинг исторических данных |
| Chrono | 135-139 | Работа с timestamps |
| TOML | 140 | Конфигурация |
| Env vars | 141-142 | Секретные ключи |
| Clap | 143 | CLI аргументы |
| Logging | 144-147 | Логирование операций |
| Streaming | 149 | Обработка больших файлов |
| Memoization | 150 | Кэширование результатов |

## Навигация

[← Предыдущий день](../150-memoization-caching/ru.md) | [Следующий день →](../152-tcp-basics/ru.md)
