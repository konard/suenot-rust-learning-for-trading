# День 121: Чтение файла — загружаем историю цен

## Аналогия из трейдинга

Представь, что ты пришёл в офис и тебе нужно проанализировать исторические данные за последний год. Эти данные хранятся в файле на диске — CSV с ценами, JSON с настройками стратегии или текстовый лог сделок. Чтение файла в программировании — это как открытие документа для анализа: ты получаешь доступ к данным, которые были сохранены ранее.

## Зачем читать файлы в трейдинге?

1. **Загрузка исторических данных** — бэктестинг стратегий
2. **Конфигурации** — настройки API ключей, лимитов
3. **Логи сделок** — анализ прошлых операций
4. **Кеширование** — локальное хранение данных с биржи

## Основы чтения файлов в Rust

### Чтение всего файла в строку

```rust
use std::fs;

fn main() {
    // Самый простой способ — прочитать весь файл
    match fs::read_to_string("prices.txt") {
        Ok(content) => {
            println!("Файл содержит {} байт", content.len());
            println!("Содержимое:\n{}", content);
        }
        Err(e) => println!("Ошибка чтения файла: {}", e),
    }
}
```

### Обработка ошибок при чтении

```rust
use std::fs;
use std::io;

fn main() {
    match load_price_file("btc_prices.txt") {
        Ok(prices) => {
            println!("Загружено {} цен", prices.len());
            if let Some(last) = prices.last() {
                println!("Последняя цена: ${:.2}", last);
            }
        }
        Err(e) => eprintln!("Не удалось загрузить цены: {}", e),
    }
}

fn load_price_file(path: &str) -> io::Result<Vec<f64>> {
    let content = fs::read_to_string(path)?;  // ? пробрасывает ошибку

    let prices: Vec<f64> = content
        .lines()                              // Разбиваем на строки
        .filter(|line| !line.is_empty())      // Пропускаем пустые
        .filter_map(|line| line.parse().ok()) // Парсим в f64
        .collect();

    Ok(prices)
}
```

## Чтение файла построчно

Для больших файлов лучше читать построчно, чтобы не загружать весь файл в память:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    match load_ohlcv_data("btc_1h.csv") {
        Ok(candles) => {
            println!("Загружено {} свечей", candles.len());

            // Найдём максимальную цену
            if let Some(max_high) = candles.iter().map(|c| c.high).reduce(f64::max) {
                println!("Исторический максимум: ${:.2}", max_high);
            }
        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}

struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn load_ohlcv_data(path: &str) -> std::io::Result<Vec<Candle>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Пропускаем заголовок
        if line_num == 0 && line.contains("timestamp") {
            continue;
        }

        // Пропускаем пустые строки
        if line.trim().is_empty() {
            continue;
        }

        // Парсим CSV: timestamp,open,high,low,close,volume
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 6 {
            if let (Ok(ts), Ok(o), Ok(h), Ok(l), Ok(c), Ok(v)) = (
                parts[0].parse::<u64>(),
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
                parts[5].parse::<f64>(),
            ) {
                candles.push(Candle {
                    timestamp: ts,
                    open: o,
                    high: h,
                    low: l,
                    close: c,
                    volume: v,
                });
            }
        }
    }

    Ok(candles)
}
```

## Проверка существования файла

```rust
use std::path::Path;
use std::fs;

fn main() {
    let config_path = "trading_config.json";

    if Path::new(config_path).exists() {
        println!("Конфигурация найдена, загружаем...");
        match fs::read_to_string(config_path) {
            Ok(config) => println!("Конфиг: {}", config),
            Err(e) => eprintln!("Ошибка чтения: {}", e),
        }
    } else {
        println!("Конфигурация не найдена, используем значения по умолчанию");
    }
}
```

## Чтение с указанием пути

```rust
use std::fs;
use std::path::PathBuf;

fn main() {
    // Строим путь программно
    let mut data_path = PathBuf::from("data");
    data_path.push("historical");
    data_path.push("btc_usdt_2024.csv");

    println!("Читаем файл: {:?}", data_path);

    match fs::read_to_string(&data_path) {
        Ok(content) => {
            let line_count = content.lines().count();
            println!("Файл содержит {} строк данных", line_count);
        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}
```

## Практический пример: загрузка истории сделок

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    match load_trade_history("trades.csv") {
        Ok(trades) => {
            println!("═══════════════════════════════════════");
            println!("       АНАЛИЗ ИСТОРИИ СДЕЛОК           ");
            println!("═══════════════════════════════════════");
            println!("Всего сделок: {}", trades.len());

            let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
            let winning: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
            let losing: Vec<_> = trades.iter().filter(|t| t.pnl < 0.0).collect();

            println!("Прибыльных: {} ({:.1}%)",
                winning.len(),
                winning.len() as f64 / trades.len() as f64 * 100.0
            );
            println!("Убыточных: {} ({:.1}%)",
                losing.len(),
                losing.len() as f64 / trades.len() as f64 * 100.0
            );
            println!("Общий PnL: ${:.2}", total_pnl);

            if !winning.is_empty() {
                let avg_win: f64 = winning.iter().map(|t| t.pnl).sum::<f64>()
                    / winning.len() as f64;
                println!("Средняя прибыль: ${:.2}", avg_win);
            }

            if !losing.is_empty() {
                let avg_loss: f64 = losing.iter().map(|t| t.pnl).sum::<f64>()
                    / losing.len() as f64;
                println!("Средний убыток: ${:.2}", avg_loss);
            }
            println!("═══════════════════════════════════════");
        }
        Err(e) => eprintln!("Ошибка загрузки: {}", e),
    }
}

struct Trade {
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

fn load_trade_history(path: &str) -> std::io::Result<Vec<Trade>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (i, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Пропускаем заголовок
        if i == 0 {
            continue;
        }

        // symbol,side,entry,exit,qty,pnl
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 6 {
            if let (Ok(entry), Ok(exit), Ok(qty), Ok(pnl)) = (
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
                parts[5].parse::<f64>(),
            ) {
                trades.push(Trade {
                    symbol: parts[0].to_string(),
                    side: parts[1].to_string(),
                    entry_price: entry,
                    exit_price: exit,
                    quantity: qty,
                    pnl,
                });
            }
        }
    }

    Ok(trades)
}
```

## Чтение бинарных данных

Для эффективного хранения ценовых данных иногда используют бинарный формат:

```rust
use std::fs::File;
use std::io::{Read, BufReader};

fn main() {
    match load_binary_prices("prices.bin") {
        Ok(prices) => {
            println!("Загружено {} цен из бинарного файла", prices.len());
            if let Some(last) = prices.last() {
                println!("Последняя цена: ${:.2}", last);
            }
        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}

fn load_binary_prices(path: &str) -> std::io::Result<Vec<f64>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut prices = Vec::new();

    // Читаем количество записей (u32)
    let mut count_bytes = [0u8; 4];
    reader.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes) as usize;

    // Читаем цены (f64 = 8 байт каждая)
    for _ in 0..count {
        let mut price_bytes = [0u8; 8];
        reader.read_exact(&mut price_bytes)?;
        let price = f64::from_le_bytes(price_bytes);
        prices.push(price);
    }

    Ok(prices)
}
```

## Работа с разными форматами данных

### Загрузка конфигурации из текстового файла

```rust
use std::fs;
use std::collections::HashMap;

fn main() {
    match load_config("strategy.conf") {
        Ok(config) => {
            println!("Конфигурация стратегии:");
            for (key, value) in &config {
                println!("  {} = {}", key, value);
            }

            // Используем значения
            if let Some(risk) = config.get("risk_percent") {
                if let Ok(risk_pct) = risk.parse::<f64>() {
                    println!("\nРиск на сделку: {}%", risk_pct);
                }
            }
        }
        Err(e) => eprintln!("Ошибка загрузки конфига: {}", e),
    }
}

fn load_config(path: &str) -> std::io::Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut config = HashMap::new();

    for line in content.lines() {
        // Пропускаем комментарии и пустые строки
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Парсим key=value
        if let Some((key, value)) = line.split_once('=') {
            config.insert(
                key.trim().to_string(),
                value.trim().to_string()
            );
        }
    }

    Ok(config)
}
```

## Обработка ошибок чтения

```rust
use std::fs;
use std::io::{self, ErrorKind};

fn main() {
    let files = ["prices.csv", "backup_prices.csv", "default_prices.csv"];

    match load_first_available(&files) {
        Ok((filename, prices)) => {
            println!("Загружено из '{}': {} записей", filename, prices.len());
        }
        Err(e) => {
            eprintln!("Не удалось загрузить данные ни из одного файла: {}", e);
        }
    }
}

fn load_first_available(paths: &[&str]) -> io::Result<(String, Vec<f64>)> {
    for path in paths {
        match fs::read_to_string(path) {
            Ok(content) => {
                let prices: Vec<f64> = content
                    .lines()
                    .filter_map(|l| l.parse().ok())
                    .collect();
                return Ok((path.to_string(), prices));
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("Файл '{}' не найден, пробуем следующий...", path);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ни один файл не найден"
    ))
}
```

## Что мы узнали

| Метод | Использование | Когда применять |
|-------|--------------|-----------------|
| `fs::read_to_string` | Весь файл в строку | Маленькие файлы |
| `BufReader::lines()` | Построчное чтение | Большие файлы |
| `Path::exists()` | Проверка наличия | Перед чтением |
| `PathBuf` | Построение пути | Динамические пути |
| `read_exact` | Бинарные данные | Компактное хранение |

## Практические задания

1. **Загрузчик цен**: Напиши функцию, которая читает файл с ценами (одна цена на строку) и возвращает `Vec<f64>`. Обработай случай пустого файла.

2. **Парсер OHLCV**: Создай функцию для чтения CSV с колонками `date,open,high,low,close,volume`. Пропускай некорректные строки, но логируй их номера.

3. **Умный загрузчик**: Реализуй функцию, которая пытается загрузить данные из нескольких источников по очереди: сначала из основного файла, затем из бэкапа, затем из архива.

4. **Валидатор данных**: Напиши функцию, которая читает файл с ценами и проверяет данные на корректность (цены положительные, нет пропусков).

## Домашнее задание

1. Создай функцию `load_portfolio(path: &str) -> Result<Portfolio, String>`, которая загружает портфель из файла формата:
   ```
   BTC,0.5,42000.0
   ETH,10.0,2500.0
   SOL,100.0,95.0
   ```

2. Напиши функцию `find_data_files(dir: &str) -> Vec<PathBuf>`, которая находит все CSV файлы в директории.

3. Реализуй функцию `merge_price_files(files: &[&str]) -> Vec<(u64, f64)>`, которая объединяет несколько файлов с ценами (timestamp, price) в один отсортированный вектор.

4. Создай структуру `PriceDataLoader` с методами:
   - `new(path: &str)` — создание загрузчика
   - `load()` — загрузка данных
   - `get_range(start: u64, end: u64)` — получение данных за период

## Навигация

[← Предыдущий день](../120-project-robust-api-client/ru.md) | [Следующий день →](../122-writing-file-saving-trades/ru.md)
