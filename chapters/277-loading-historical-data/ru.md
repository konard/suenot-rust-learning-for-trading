# День 277: Загрузка исторических данных

## Аналогия из трейдинга

Прежде чем пилот полетит на новом самолёте, он проводит часы в симуляторе полётов. Аналогично, прежде чем трейдер запустит стратегию с реальными деньгами, ему нужно протестировать её на исторических рыночных данных. Этот процесс называется **бэктестинг** — воспроизведение прошлых рыночных условий, чтобы увидеть, как бы стратегия себя показала.

Представь, что у тебя есть блестящая идея: "Покупать, когда цена падает на 5%, и продавать, когда она растёт на 3%". Прежде чем рисковать своими сбережениями, ты захочешь узнать: как бы эта стратегия показала себя во время обвала 2020 года? Во время бычьего рынка 2021 года? Загрузка исторических данных — это первый шаг к ответу на эти вопросы.

В этой главе мы научимся:
- Загружать рыночные данные из различных форматов файлов (CSV, JSON)
- Парсить и валидировать данные OHLCV (Open, High, Low, Close, Volume)
- Обрабатывать временные метки и часовые пояса
- Строить надёжные пайплайны загрузки данных для бэктестинга

## Что такое OHLCV данные?

OHLCV — это стандартный формат представления ценовых данных за период времени (свеча):

| Поле   | Описание                                          |
|--------|---------------------------------------------------|
| Open   | Первая цена в начале периода                      |
| High   | Максимальная цена за период                       |
| Low    | Минимальная цена за период                        |
| Close  | Последняя цена в конце периода                    |
| Volume | Общий объём торгов за период                      |

```rust
use chrono::{DateTime, Utc};

/// Представляет одну OHLCV свечу (ценовой бар)
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Self, String> {
        // Валидация OHLCV данных
        if high < low {
            return Err("High не может быть меньше low".to_string());
        }
        if high < open || high < close {
            return Err("High должен быть >= open и close".to_string());
        }
        if low > open || low > close {
            return Err("Low должен быть <= open и close".to_string());
        }
        if volume < 0.0 {
            return Err("Volume не может быть отрицательным".to_string());
        }

        Ok(Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        })
    }

    /// Вычислить размер тела свечи (абсолютная разница между open и close)
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Вычислить полный диапазон (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Проверить, является ли свеча бычьей (зелёной)
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Проверить, является ли свеча медвежьей (красной)
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}
```

## Загрузка данных из CSV

CSV (Comma-Separated Values) — самый распространённый формат для исторических рыночных данных. Давайте построим надёжный загрузчик CSV:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, TimeZone, Utc};

/// Ошибки, которые могут возникнуть при загрузке данных
#[derive(Debug)]
pub enum DataLoadError {
    FileNotFound(String),
    IoError(std::io::Error),
    ParseError { line: usize, message: String },
    ValidationError { line: usize, message: String },
    EmptyFile,
}

impl std::fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataLoadError::FileNotFound(path) => write!(f, "Файл не найден: {}", path),
            DataLoadError::IoError(e) => write!(f, "Ошибка ввода/вывода: {}", e),
            DataLoadError::ParseError { line, message } => {
                write!(f, "Ошибка парсинга в строке {}: {}", line, message)
            }
            DataLoadError::ValidationError { line, message } => {
                write!(f, "Ошибка валидации в строке {}: {}", line, message)
            }
            DataLoadError::EmptyFile => write!(f, "Файл не содержит данных"),
        }
    }
}

impl std::error::Error for DataLoadError {}

/// Загрузить OHLCV данные из CSV файла
/// Ожидаемый формат: timestamp,open,high,low,close,volume
pub fn load_csv(path: &Path) -> Result<Vec<Candle>, DataLoadError> {
    if !path.exists() {
        return Err(DataLoadError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    let file = File::open(path).map_err(DataLoadError::IoError)?;
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(DataLoadError::IoError)?;

        // Пропускаем заголовок
        if line_num == 0 && line.to_lowercase().contains("timestamp") {
            continue;
        }

        // Пропускаем пустые строки
        if line.trim().is_empty() {
            continue;
        }

        let candle = parse_csv_line(&line, line_num + 1)?;
        candles.push(candle);
    }

    if candles.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    // Сортируем по времени для хронологического порядка
    candles.sort_by_key(|c| c.timestamp);

    Ok(candles)
}

fn parse_csv_line(line: &str, line_num: usize) -> Result<Candle, DataLoadError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() < 6 {
        return Err(DataLoadError::ParseError {
            line: line_num,
            message: format!("Ожидалось 6 полей, найдено {}", parts.len()),
        });
    }

    // Парсим временную метку (поддержка Unix timestamp или ISO формата)
    let timestamp = parse_timestamp(parts[0]).map_err(|e| DataLoadError::ParseError {
        line: line_num,
        message: format!("Неверная временная метка: {}", e),
    })?;

    // Парсим числовые поля
    let open = parse_f64(parts[1], "open", line_num)?;
    let high = parse_f64(parts[2], "high", line_num)?;
    let low = parse_f64(parts[3], "low", line_num)?;
    let close = parse_f64(parts[4], "close", line_num)?;
    let volume = parse_f64(parts[5], "volume", line_num)?;

    Candle::new(timestamp, open, high, low, close, volume).map_err(|e| {
        DataLoadError::ValidationError {
            line: line_num,
            message: e,
        }
    })
}

fn parse_f64(s: &str, field: &str, line_num: usize) -> Result<f64, DataLoadError> {
    s.parse::<f64>().map_err(|_| DataLoadError::ParseError {
        line: line_num,
        message: format!("Неверное значение {}: '{}'", field, s),
    })
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, String> {
    // Пробуем Unix timestamp (секунды)
    if let Ok(ts) = s.parse::<i64>() {
        return Utc
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| "Неверный Unix timestamp".to_string());
    }

    // Пробуем Unix timestamp (миллисекунды)
    if let Ok(ts_ms) = s.parse::<i64>() {
        if ts_ms > 1_000_000_000_000 {
            return Utc
                .timestamp_millis_opt(ts_ms)
                .single()
                .ok_or_else(|| "Неверный Unix timestamp (мс)".to_string());
        }
    }

    // Пробуем формат ISO 8601
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Пробуем распространённый формат: YYYY-MM-DD HH:MM:SS
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| Utc.from_utc_datetime(&ndt))
        })
        .map_err(|e| format!("Не удалось разобрать временную метку '{}': {}", s, e))
}
```

## Загрузка данных из JSON

Многие API предоставляют данные в формате JSON. Вот как с ним работать:

```rust
use serde::{Deserialize, Serialize};
use std::fs;

/// JSON представление OHLCV данных (распространённый формат API)
#[derive(Debug, Deserialize, Serialize)]
pub struct CandleJson {
    #[serde(alias = "t", alias = "time", alias = "timestamp")]
    pub timestamp: i64,

    #[serde(alias = "o")]
    pub open: f64,

    #[serde(alias = "h")]
    pub high: f64,

    #[serde(alias = "l")]
    pub low: f64,

    #[serde(alias = "c")]
    pub close: f64,

    #[serde(alias = "v", alias = "vol")]
    pub volume: f64,
}

/// Загрузить OHLCV данные из JSON файла
pub fn load_json(path: &Path) -> Result<Vec<Candle>, DataLoadError> {
    if !path.exists() {
        return Err(DataLoadError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(path).map_err(DataLoadError::IoError)?;

    let json_candles: Vec<CandleJson> =
        serde_json::from_str(&content).map_err(|e| DataLoadError::ParseError {
            line: 0,
            message: format!("Ошибка парсинга JSON: {}", e),
        })?;

    let mut candles = Vec::with_capacity(json_candles.len());

    for (idx, jc) in json_candles.iter().enumerate() {
        // Определяем, timestamp в секундах или миллисекундах
        let timestamp = if jc.timestamp > 1_000_000_000_000 {
            Utc.timestamp_millis_opt(jc.timestamp)
                .single()
                .ok_or_else(|| DataLoadError::ValidationError {
                    line: idx + 1,
                    message: "Неверная временная метка".to_string(),
                })?
        } else {
            Utc.timestamp_opt(jc.timestamp, 0)
                .single()
                .ok_or_else(|| DataLoadError::ValidationError {
                    line: idx + 1,
                    message: "Неверная временная метка".to_string(),
                })?
        };

        let candle = Candle::new(timestamp, jc.open, jc.high, jc.low, jc.close, jc.volume)
            .map_err(|e| DataLoadError::ValidationError {
                line: idx + 1,
                message: e,
            })?;

        candles.push(candle);
    }

    if candles.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    candles.sort_by_key(|c| c.timestamp);
    Ok(candles)
}
```

## Создание универсального загрузчика данных

Давайте создадим единый интерфейс для работы с разными форматами:

```rust
use std::path::Path;

/// Поддерживаемые форматы данных
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Csv,
    Json,
    Auto, // Определить по расширению файла
}

/// Универсальный загрузчик исторических данных
pub struct HistoricalDataLoader {
    format: DataFormat,
}

impl HistoricalDataLoader {
    pub fn new(format: DataFormat) -> Self {
        HistoricalDataLoader { format }
    }

    /// Автоопределение формата по расширению файла
    pub fn auto() -> Self {
        HistoricalDataLoader {
            format: DataFormat::Auto,
        }
    }

    /// Загрузить данные из файла
    pub fn load(&self, path: &Path) -> Result<Vec<Candle>, DataLoadError> {
        let format = match self.format {
            DataFormat::Auto => Self::detect_format(path)?,
            other => other,
        };

        match format {
            DataFormat::Csv => load_csv(path),
            DataFormat::Json => load_json(path),
            DataFormat::Auto => unreachable!(),
        }
    }

    /// Загрузить данные из нескольких файлов и объединить
    pub fn load_multiple(&self, paths: &[&Path]) -> Result<Vec<Candle>, DataLoadError> {
        let mut all_candles = Vec::new();

        for path in paths {
            let candles = self.load(path)?;
            all_candles.extend(candles);
        }

        // Сортируем и удаляем дубликаты
        all_candles.sort_by_key(|c| c.timestamp);
        all_candles.dedup_by_key(|c| c.timestamp);

        Ok(all_candles)
    }

    fn detect_format(path: &Path) -> Result<DataFormat, DataLoadError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension.as_deref() {
            Some("csv") => Ok(DataFormat::Csv),
            Some("json") => Ok(DataFormat::Json),
            _ => Err(DataLoadError::ParseError {
                line: 0,
                message: "Не удалось определить формат файла. Используйте расширение .csv или .json".to_string(),
            }),
        }
    }
}
```

## Валидация и очистка данных

Реальные данные часто содержат ошибки. Добавим валидацию:

```rust
/// Статистика качества загруженных данных
#[derive(Debug, Default)]
pub struct DataQualityReport {
    pub total_records: usize,
    pub valid_records: usize,
    pub gaps_detected: usize,
    pub duplicates_removed: usize,
    pub outliers_detected: usize,
}

/// Валидировать и очистить исторические данные
pub fn validate_and_clean(
    candles: Vec<Candle>,
    expected_interval_secs: i64,
) -> (Vec<Candle>, DataQualityReport) {
    let mut report = DataQualityReport {
        total_records: candles.len(),
        ..Default::default()
    };

    let mut cleaned: Vec<Candle> = Vec::with_capacity(candles.len());
    let mut prev_timestamp: Option<DateTime<Utc>> = None;

    for candle in candles {
        // Проверяем пропуски
        if let Some(prev) = prev_timestamp {
            let gap = candle.timestamp.signed_duration_since(prev).num_seconds();
            if gap > expected_interval_secs * 2 {
                report.gaps_detected += 1;
                println!(
                    "Обнаружен пропуск: {} секунд между {} и {}",
                    gap, prev, candle.timestamp
                );
            }
        }

        // Проверяем дубликаты
        if Some(candle.timestamp) == prev_timestamp {
            report.duplicates_removed += 1;
            continue;
        }

        // Проверяем выбросы (изменение цены > 50% за одну свечу)
        if let Some(last) = cleaned.last() {
            let price_change = ((candle.close - last.close) / last.close).abs();
            if price_change > 0.5 {
                report.outliers_detected += 1;
                println!(
                    "Обнаружен выброс в {}: {:.2}% изменение цены",
                    candle.timestamp,
                    price_change * 100.0
                );
            }
        }

        prev_timestamp = Some(candle.timestamp);
        cleaned.push(candle);
        report.valid_records += 1;
    }

    (cleaned, report)
}
```

## Практический пример: Полный пайплайн данных для бэктестинга

Давайте соберём всё вместе в полном примере:

```rust
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::path::Path;

/// Представляет торговый символ с его историческими данными
pub struct SymbolData {
    pub symbol: String,
    pub timeframe: String,
    pub candles: Vec<Candle>,
}

impl SymbolData {
    /// Получить свечи в диапазоне дат
    pub fn get_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&Candle> {
        self.candles
            .iter()
            .filter(|c| c.timestamp >= start && c.timestamp <= end)
            .collect()
    }

    /// Вычислить простую статистику
    pub fn statistics(&self) -> DataStatistics {
        if self.candles.is_empty() {
            return DataStatistics::default();
        }

        let prices: Vec<f64> = self.candles.iter().map(|c| c.close).collect();
        let volumes: Vec<f64> = self.candles.iter().map(|c| c.volume).collect();

        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
        let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        DataStatistics {
            count: self.candles.len(),
            start_date: self.candles.first().map(|c| c.timestamp),
            end_date: self.candles.last().map(|c| c.timestamp),
            avg_price,
            min_price,
            max_price,
            avg_volume,
        }
    }
}

#[derive(Debug, Default)]
pub struct DataStatistics {
    pub count: usize,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub avg_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub avg_volume: f64,
}

/// Пример: Загрузка и анализ данных BTC/USDT
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаём примерные данные для демонстрации
    let sample_csv = r#"timestamp,open,high,low,close,volume
1609459200,29000.0,29500.0,28800.0,29400.0,1500.5
1609462800,29400.0,30000.0,29300.0,29800.0,2000.0
1609466400,29800.0,30200.0,29600.0,30100.0,1800.0
1609470000,30100.0,30500.0,29900.0,30300.0,2200.0
1609473600,30300.0,30800.0,30100.0,30600.0,1900.0
"#;

    // Записываем примерные данные во временный файл
    std::fs::write("/tmp/btc_sample.csv", sample_csv)?;

    // Загружаем данные
    let loader = HistoricalDataLoader::auto();
    let candles = loader.load(Path::new("/tmp/btc_sample.csv"))?;

    println!("Загружено {} свечей", candles.len());

    // Валидируем и очищаем
    let (cleaned, report) = validate_and_clean(candles, 3600); // Часовые свечи
    println!("\nОтчёт о качестве данных:");
    println!("  Всего записей: {}", report.total_records);
    println!("  Валидных записей: {}", report.valid_records);
    println!("  Обнаружено пропусков: {}", report.gaps_detected);
    println!("  Удалено дубликатов: {}", report.duplicates_removed);

    // Создаём данные по символу
    let btc_data = SymbolData {
        symbol: "BTC/USDT".to_string(),
        timeframe: "1h".to_string(),
        candles: cleaned,
    };

    // Выводим статистику
    let stats = btc_data.statistics();
    println!("\nСтатистика BTC/USDT:");
    println!("  Точек данных: {}", stats.count);
    if let (Some(start), Some(end)) = (stats.start_date, stats.end_date) {
        println!("  Период: {} - {}", start, end);
    }
    println!("  Средняя цена: ${:.2}", stats.avg_price);
    println!("  Мин. цена: ${:.2}", stats.min_price);
    println!("  Макс. цена: ${:.2}", stats.max_price);
    println!("  Средний объём: {:.2} BTC", stats.avg_volume);

    // Анализируем отдельные свечи
    println!("\nАнализ свечей:");
    for (i, candle) in btc_data.candles.iter().enumerate() {
        let trend = if candle.is_bullish() {
            "БЫЧЬЯ"
        } else if candle.is_bearish() {
            "МЕДВЕЖЬЯ"
        } else {
            "ДОЖИ"
        };

        println!(
            "  Свеча {}: {} - Open: ${:.2}, Close: ${:.2}, Диапазон: ${:.2} ({})",
            i + 1,
            candle.timestamp.format("%Y-%m-%d %H:%M"),
            candle.open,
            candle.close,
            candle.range(),
            trend
        );
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| OHLCV | Стандартный формат: Open, High, Low, Close, Volume |
| Candle (Свеча) | Один ценовой бар, представляющий временной период |
| Загрузка CSV | Парсинг значений, разделённых запятыми, с обработкой ошибок |
| Загрузка JSON | Парсинг структурированных данных с помощью serde |
| Парсинг временных меток | Обработка Unix timestamp и форматов ISO 8601 |
| Валидация данных | Проверка пропусков, дубликатов и выбросов |
| Очистка данных | Удаление невалидных записей, хронологическая сортировка |
| Универсальный загрузчик | Поддержка нескольких форматов с автоопределением |

## Упражнения

1. **Добавить поддержку Parquet**: Расширь `HistoricalDataLoader` для поддержки формата Apache Parquet, который часто используется для больших наборов данных.

2. **Реализовать заполнение пропусков**: Создай функцию, которая заполняет пропущенные свечи интерполяцией или переносом последнего известного значения.

3. **Обработка часовых поясов**: Модифицируй загрузчик для приёма исходного часового пояса и конвертации всех временных меток в UTC.

4. **Потоковый загрузчик**: Реализуй загрузчик на основе итератора, который обрабатывает данные построчно без загрузки всего файла в память.

## Домашнее задание

1. **Загрузчик нескольких символов**: Создай структуру `MarketDataManager`, которая может:
   - Загружать данные по нескольким символам
   - Выравнивать временные метки между символами
   - Обрабатывать разные таймфреймы (1m, 5m, 1h, 1d)
   - Возвращать синхронизированные данные для бэктестинга

2. **Абстракция источника данных**: Спроектируй трейт `DataSource` с реализациями для:
   - Локальной файловой системы
   - HTTP API (mock-реализация)
   - Кэша в памяти
   Это позволит бэктестеру переключаться между источниками данных бесшовно.

3. **Определение свечных паттернов**: Используя структуру `Candle`, реализуй функции для определения:
   - Дожи (open == close, с допуском)
   - Молот (длинная нижняя тень, маленькое тело вверху)
   - Поглощение (текущая свеча поглощает предыдущую)
   - Утренняя/Вечерняя звезда (паттерн из трёх свечей)

4. **Дашборд качества данных**: Создай комплексную функцию `analyze_data_quality`, которая возвращает:
   - Процент пропущенных данных
   - Среднюю продолжительность пропусков
   - Метрики волатильности цены
   - Аномалии объёма
   - Проверку согласованности временных меток

## Навигация

[← Предыдущий день](../276-backtesting-framework-design/ru.md) | [Следующий день →](../278-simulating-order-execution/ru.md)
