# День 126: Path и PathBuf — пути к файлам

## Аналогия из трейдинга

Представь, что у тебя есть офис с множеством папок для разных документов: исторические данные в одной папке, конфигурации стратегий в другой, логи сделок в третьей. **Путь к файлу** — это как точный адрес документа: "шкаф → полка → папка → документ".

В трейдинге мы постоянно работаем с файлами:
- Исторические данные цен (CSV, JSON)
- Конфигурации торговых стратегий
- Логи сделок и ошибок
- Файлы состояния портфеля

`Path` — это неизменяемая ссылка на путь (как `&str` для строк), а `PathBuf` — это владеющий путь (как `String`).

## Базовое использование Path и PathBuf

```rust
use std::path::{Path, PathBuf};

fn main() {
    // Path — неизменяемая ссылка на путь
    let config_path = Path::new("config/strategy.json");
    println!("Config path: {:?}", config_path);

    // PathBuf — владеющий путь, можно изменять
    let mut data_path = PathBuf::new();
    data_path.push("data");
    data_path.push("historical");
    data_path.push("btc_usdt.csv");
    println!("Data path: {:?}", data_path);

    // Преобразования
    let path_ref: &Path = data_path.as_path();  // PathBuf -> &Path
    let owned: PathBuf = config_path.to_path_buf();  // &Path -> PathBuf
}
```

## Создание путей для торговых данных

```rust
use std::path::{Path, PathBuf};

fn main() {
    // Путь к файлу с историческими данными
    let historical_data = Path::new("data/historical/btc_usdt_1h.csv");

    // Динамическое построение пути
    let ticker = "eth_usdt";
    let timeframe = "4h";

    let mut price_data_path = PathBuf::from("data");
    price_data_path.push("historical");
    price_data_path.push(format!("{}_{}.csv", ticker, timeframe));

    println!("Price data: {:?}", price_data_path);
    // Output: "data/historical/eth_usdt_4h.csv"
}
```

## Компоненты пути

```rust
use std::path::Path;

fn main() {
    let log_path = Path::new("/var/log/trading/trades_2024.log");

    // Имя файла
    if let Some(file_name) = log_path.file_name() {
        println!("File name: {:?}", file_name);  // "trades_2024.log"
    }

    // Расширение файла
    if let Some(extension) = log_path.extension() {
        println!("Extension: {:?}", extension);  // "log"
    }

    // Имя без расширения
    if let Some(stem) = log_path.file_stem() {
        println!("File stem: {:?}", stem);  // "trades_2024"
    }

    // Родительская директория
    if let Some(parent) = log_path.parent() {
        println!("Parent: {:?}", parent);  // "/var/log/trading"
    }
}
```

## Проверки существования и типа

```rust
use std::path::Path;

fn main() {
    let config_path = Path::new("config/strategy.json");
    let data_dir = Path::new("data/historical");

    // Проверка существования
    if config_path.exists() {
        println!("Config file exists");
    } else {
        println!("Config file not found - using defaults");
    }

    // Проверка типа: файл или директория
    if data_dir.is_dir() {
        println!("Data directory exists");
    }

    if config_path.is_file() {
        println!("Config is a file");
    }

    // Абсолютный или относительный путь
    println!("Is absolute: {}", config_path.is_absolute());
    println!("Is relative: {}", config_path.is_relative());
}
```

## Работа с путями в торговой системе

```rust
use std::path::{Path, PathBuf};

struct TradingDataManager {
    base_dir: PathBuf,
}

impl TradingDataManager {
    fn new(base_dir: &str) -> Self {
        TradingDataManager {
            base_dir: PathBuf::from(base_dir),
        }
    }

    fn get_historical_data_path(&self, ticker: &str, timeframe: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("historical");
        path.push(format!("{}_{}.csv", ticker, timeframe));
        path
    }

    fn get_strategy_config_path(&self, strategy_name: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("strategies");
        path.push(format!("{}.json", strategy_name));
        path
    }

    fn get_trade_log_path(&self, date: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("logs");
        path.push("trades");
        path.push(format!("trades_{}.log", date));
        path
    }

    fn get_portfolio_snapshot_path(&self, timestamp: u64) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("snapshots");
        path.push(format!("portfolio_{}.json", timestamp));
        path
    }
}

fn main() {
    let manager = TradingDataManager::new("/home/trader/trading_system");

    println!("BTC 1h data: {:?}", manager.get_historical_data_path("btc_usdt", "1h"));
    println!("SMA strategy: {:?}", manager.get_strategy_config_path("sma_crossover"));
    println!("Today's trades: {:?}", manager.get_trade_log_path("2024-01-15"));
    println!("Portfolio: {:?}", manager.get_portfolio_snapshot_path(1705312800));
}
```

## Объединение путей с join

```rust
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("/data/trading");

    // join создаёт новый PathBuf
    let historical = base.join("historical");
    let btc_data = base.join("historical").join("btc_usdt_1h.csv");

    println!("Historical dir: {:?}", historical);
    println!("BTC data: {:?}", btc_data);

    // Если второй путь абсолютный, он заменяет первый
    let absolute = Path::new("/etc/config.json");
    let result = base.join(absolute);
    println!("Joined absolute: {:?}", result);  // "/etc/config.json"
}
```

## Изменение путей

```rust
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::from("data/historical/btc_usdt_1h.csv");

    // Изменение расширения
    path.set_extension("json");
    println!("Changed extension: {:?}", path);  // "data/historical/btc_usdt_1h.json"

    // Изменение имени файла
    path.set_file_name("eth_usdt_4h.json");
    println!("Changed file name: {:?}", path);  // "data/historical/eth_usdt_4h.json"

    // Удаление последнего компонента
    path.pop();
    println!("After pop: {:?}", path);  // "data/historical"

    // Добавление нового компонента
    path.push("sol_usdt_1d.csv");
    println!("After push: {:?}", path);  // "data/historical/sol_usdt_1d.csv"
}
```

## Итерация по компонентам пути

```rust
use std::path::Path;

fn main() {
    let path = Path::new("/data/trading/historical/btc_usdt/2024/01/candles.csv");

    println!("Path components:");
    for component in path.components() {
        println!("  {:?}", component);
    }

    // Извлечение информации из пути к данным
    let parts: Vec<_> = path.components().collect();
    println!("\nTotal components: {}", parts.len());
}

fn parse_data_path(path: &Path) -> Option<DataPathInfo> {
    let components: Vec<_> = path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Ожидаем формат: .../historical/{ticker}/{year}/{month}/{filename}
    if components.len() >= 5 {
        let len = components.len();
        Some(DataPathInfo {
            ticker: components[len - 4].to_string(),
            year: components[len - 3].to_string(),
            month: components[len - 2].to_string(),
            filename: components[len - 1].to_string(),
        })
    } else {
        None
    }
}

struct DataPathInfo {
    ticker: String,
    year: String,
    month: String,
    filename: String,
}
```

## Кросс-платформенные пути

```rust
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

fn main() {
    // Разделитель пути зависит от ОС
    println!("Path separator: '{}'", MAIN_SEPARATOR);
    // Windows: '\', Unix: '/'

    // Path автоматически работает с правильным разделителем
    let path = Path::new("data").join("historical").join("btc.csv");
    println!("Cross-platform path: {:?}", path);

    // Получение строки из пути
    if let Some(path_str) = path.to_str() {
        println!("As string: {}", path_str);
    }

    // Для случаев, когда нужен String
    let path_string: String = path.display().to_string();
    println!("Display: {}", path_string);
}
```

## Практический пример: Менеджер файлов торговой системы

```rust
use std::path::{Path, PathBuf};

struct TradingFileManager {
    root: PathBuf,
}

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    file_type: FileType,
    ticker: Option<String>,
    timeframe: Option<String>,
}

#[derive(Debug)]
enum FileType {
    HistoricalData,
    StrategyConfig,
    TradeLog,
    PortfolioSnapshot,
    Unknown,
}

impl TradingFileManager {
    fn new(root: &str) -> Self {
        TradingFileManager {
            root: PathBuf::from(root),
        }
    }

    fn classify_file(&self, path: &Path) -> FileInfo {
        let path_buf = path.to_path_buf();
        let extension = path.extension().and_then(|e| e.to_str());

        // Определяем тип файла по расширению и пути
        let file_type = match extension {
            Some("csv") => {
                if path.to_string_lossy().contains("historical") {
                    FileType::HistoricalData
                } else {
                    FileType::Unknown
                }
            }
            Some("json") => {
                if path.to_string_lossy().contains("strateg") {
                    FileType::StrategyConfig
                } else if path.to_string_lossy().contains("portfolio") {
                    FileType::PortfolioSnapshot
                } else {
                    FileType::Unknown
                }
            }
            Some("log") => FileType::TradeLog,
            _ => FileType::Unknown,
        };

        // Извлекаем тикер и таймфрейм из имени файла
        let (ticker, timeframe) = self.extract_ticker_info(path);

        FileInfo {
            path: path_buf,
            file_type,
            ticker,
            timeframe,
        }
    }

    fn extract_ticker_info(&self, path: &Path) -> (Option<String>, Option<String>) {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            // Формат: ticker_timeframe (например: btc_usdt_1h)
            let parts: Vec<&str> = stem.split('_').collect();
            if parts.len() >= 3 {
                let ticker = format!("{}_{}", parts[0], parts[1]);
                let timeframe = parts[2].to_string();
                return (Some(ticker), Some(timeframe));
            }
        }
        (None, None)
    }

    fn create_backup_path(&self, original: &Path) -> PathBuf {
        let mut backup = self.root.join("backups");

        if let Some(file_name) = original.file_name() {
            let timestamp = 1705312800u64; // В реальности - текущее время
            let backup_name = format!("{}_{}", timestamp, file_name.to_string_lossy());
            backup.push(backup_name);
        }

        backup
    }

    fn get_all_data_files(&self, ticker: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let historical_dir = self.root.join("historical");

        // В реальности здесь был бы обход директории
        // Демонстрация построения путей для разных таймфреймов
        for timeframe in &["1m", "5m", "15m", "1h", "4h", "1d"] {
            files.push(historical_dir.join(format!("{}_{}.csv", ticker, timeframe)));
        }

        files
    }
}

fn main() {
    let manager = TradingFileManager::new("/home/trader/trading");

    // Классификация файлов
    let files = vec![
        Path::new("data/historical/btc_usdt_1h.csv"),
        Path::new("config/strategies/sma_crossover.json"),
        Path::new("logs/trades_2024-01-15.log"),
        Path::new("snapshots/portfolio_1705312800.json"),
    ];

    println!("File Classification:");
    println!("════════════════════════════════════════");
    for file in &files {
        let info = manager.classify_file(file);
        println!("Path: {:?}", info.path);
        println!("Type: {:?}", info.file_type);
        if let Some(ticker) = &info.ticker {
            println!("Ticker: {}", ticker);
        }
        if let Some(tf) = &info.timeframe {
            println!("Timeframe: {}", tf);
        }
        println!("────────────────────────────────────────");
    }

    // Создание путей для бэкапов
    let original = Path::new("config/strategies/sma_crossover.json");
    let backup = manager.create_backup_path(original);
    println!("Backup path: {:?}", backup);

    // Получение всех файлов для тикера
    println!("\nAll BTC/USDT data files:");
    for file in manager.get_all_data_files("btc_usdt") {
        println!("  {:?}", file);
    }
}
```

## Преобразования между типами

```rust
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

fn main() {
    // String/&str -> Path/PathBuf
    let s = "data/prices.csv";
    let path = Path::new(s);
    let path_buf = PathBuf::from(s);

    // Path/PathBuf -> &str (может не получиться для не-UTF8 путей)
    if let Some(str_ref) = path.to_str() {
        println!("As &str: {}", str_ref);
    }

    // Path/PathBuf -> String
    let string = path.display().to_string();
    println!("As String: {}", string);

    // PathBuf <-> Path
    let path_ref: &Path = path_buf.as_path();
    let owned: PathBuf = path.to_path_buf();

    // Работа с OsStr для имён файлов
    let file_name: &OsStr = path.file_name().unwrap();
    println!("File name (OsStr): {:?}", file_name);
}
```

## Паттерны работы с путями

```rust
use std::path::{Path, PathBuf};

// 1. Функция принимает &Path — работает и с Path, и с PathBuf
fn process_data_file(path: &Path) -> bool {
    path.exists() && path.extension().map(|e| e == "csv").unwrap_or(false)
}

// 2. Функция возвращает PathBuf — создаёт новый путь
fn create_output_path(input: &Path) -> PathBuf {
    let mut output = input.to_path_buf();
    output.set_extension("processed.csv");
    output
}

// 3. Функция модифицирует PathBuf на месте
fn add_timestamp(path: &mut PathBuf, timestamp: u64) {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        path.set_file_name(format!("{}_{}.{}", stem, timestamp, ext));
    }
}

fn main() {
    let csv_path = Path::new("data/btc.csv");
    let txt_path = Path::new("readme.txt");

    println!("Is CSV data file: {}", process_data_file(csv_path));
    println!("Is CSV data file: {}", process_data_file(txt_path));

    let output = create_output_path(csv_path);
    println!("Output: {:?}", output);

    let mut path = PathBuf::from("data/prices.csv");
    add_timestamp(&mut path, 1705312800);
    println!("With timestamp: {:?}", path);
}
```

## Что мы узнали

| Тип | Владение | Аналог | Использование |
|-----|----------|--------|---------------|
| `Path` | Заимствование | `&str` | Неизменяемые операции |
| `PathBuf` | Владение | `String` | Создание и модификация путей |

| Метод | Описание |
|-------|----------|
| `new()` | Создание Path из строки |
| `push()` | Добавление компонента к PathBuf |
| `join()` | Объединение путей (новый PathBuf) |
| `parent()` | Родительская директория |
| `file_name()` | Имя файла |
| `extension()` | Расширение файла |
| `exists()` | Проверка существования |
| `is_file()` / `is_dir()` | Проверка типа |

## Домашнее задание

1. Напиши функцию `organize_data_files(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>>`, которая группирует файлы по тикеру

2. Создай структуру `DataPathBuilder` с методами-билдерами для создания путей к различным типам торговых данных

3. Реализуй функцию `validate_data_directory(path: &Path) -> Result<DataDirInfo, String>`, которая проверяет структуру директории с торговыми данными

4. Напиши функцию `find_latest_snapshot(snapshots_dir: &Path) -> Option<PathBuf>`, которая находит самый последний файл снимка портфеля по имени файла (timestamp в имени)

## Навигация

[← Предыдущий день](../125-file-system-basics/ru.md) | [Следующий день →](../127-reading-files/ru.md)
