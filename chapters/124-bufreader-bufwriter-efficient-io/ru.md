# День 124: BufReader и BufWriter — эффективный I/O

## Аналогия из трейдинга

Представь, что ты трейдер, который анализирует исторические данные. У тебя есть файл с миллионами тиков (сделок) за последний год. Если читать каждую сделку отдельным запросом к диску — это как звонить брокеру для каждой сделки вместо того, чтобы получить пакет ордеров сразу.

**BufReader** — это как получать данные пакетами: вместо одного тика за раз, ты получаешь сразу 8KB данных в память и потом читаешь из этого буфера. Это намного быстрее!

**BufWriter** работает аналогично для записи: вместо того чтобы записывать каждую строку отдельно на диск, ты накапливаешь данные в буфере и записываешь их одним блоком.

## Почему это важно?

Каждое обращение к диску — дорогая операция. Сравни:

| Подход | Операций с диском | Скорость |
|--------|-------------------|----------|
| Без буфера | 1 на каждый байт | Очень медленно |
| С буфером | 1 на ~8KB данных | Быстро |

В трейдинге, где каждая миллисекунда может стоить денег, эффективный I/O критически важен.

## Базовый пример: чтение торговых данных

```rust
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() -> std::io::Result<()> {
    // Открываем файл с историческими ценами
    let file = File::open("prices.csv")?;

    // Оборачиваем в BufReader для эффективного чтения
    let reader = BufReader::new(file);

    let mut total_volume = 0.0;
    let mut count = 0;

    // Читаем построчно — каждая строка буферизована
    for line in reader.lines() {
        let line = line?;

        // Пропускаем заголовок
        if line.starts_with("timestamp") {
            continue;
        }

        // Парсим: timestamp,price,volume
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            if let Ok(volume) = parts[2].parse::<f64>() {
                total_volume += volume;
                count += 1;
            }
        }
    }

    println!("Обработано {} записей", count);
    println!("Общий объём: {:.2}", total_volume);

    Ok(())
}
```

## BufWriter: запись торгового журнала

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() -> std::io::Result<()> {
    let file = File::create("trade_log.csv")?;
    let mut writer = BufWriter::new(file);

    // Записываем заголовок
    writeln!(writer, "timestamp,symbol,side,price,quantity,value")?;

    // Симулируем сделки
    let trades = vec![
        Trade { timestamp: 1703980800, symbol: "BTC/USDT".to_string(),
                side: "BUY".to_string(), price: 42150.50, quantity: 0.5 },
        Trade { timestamp: 1703980860, symbol: "ETH/USDT".to_string(),
                side: "SELL".to_string(), price: 2250.75, quantity: 2.0 },
        Trade { timestamp: 1703980920, symbol: "BTC/USDT".to_string(),
                side: "SELL".to_string(), price: 42200.00, quantity: 0.5 },
    ];

    for trade in &trades {
        let value = trade.price * trade.quantity;
        writeln!(
            writer,
            "{},{},{},{:.2},{:.4},{:.2}",
            trade.timestamp,
            trade.symbol,
            trade.side,
            trade.price,
            trade.quantity,
            value
        )?;
    }

    // Важно: данные гарантированно записаны на диск после flush
    writer.flush()?;

    println!("Записано {} сделок в журнал", trades.len());

    Ok(())
}
```

## Настройка размера буфера

По умолчанию размер буфера — 8KB. Для больших файлов можно увеличить:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

fn main() -> std::io::Result<()> {
    // Буфер 64KB для чтения больших файлов с ценами
    let file = File::open("large_price_history.csv")?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    // Буфер 32KB для записи отчётов
    let output = File::create("analysis_report.csv")?;
    let mut writer = BufWriter::with_capacity(32 * 1024, output);

    writeln!(writer, "date,open,high,low,close,volume")?;

    for line in reader.lines() {
        let line = line?;
        // Обработка и запись...
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;
    Ok(())
}
```

## Практический пример: анализ OHLCV данных

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

#[derive(Debug)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct DailyStats {
    date: String,
    avg_price: f64,
    volatility: f64,
    total_volume: f64,
    price_change_pct: f64,
}

fn parse_ohlcv(line: &str) -> Option<OHLCV> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 6 {
        return None;
    }

    Some(OHLCV {
        timestamp: parts[0].parse().ok()?,
        open: parts[1].parse().ok()?,
        high: parts[2].parse().ok()?,
        low: parts[3].parse().ok()?,
        close: parts[4].parse().ok()?,
        volume: parts[5].parse().ok()?,
    })
}

fn calculate_stats(candles: &[OHLCV]) -> Option<DailyStats> {
    if candles.is_empty() {
        return None;
    }

    let first = candles.first()?;
    let last = candles.last()?;

    let sum_price: f64 = candles.iter().map(|c| (c.high + c.low) / 2.0).sum();
    let avg_price = sum_price / candles.len() as f64;

    let total_volume: f64 = candles.iter().map(|c| c.volume).sum();

    // Простая волатильность: (max high - min low) / avg price
    let max_high = candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let min_low = candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let volatility = (max_high - min_low) / avg_price * 100.0;

    let price_change_pct = (last.close - first.open) / first.open * 100.0;

    Some(DailyStats {
        date: format!("{}", first.timestamp),
        avg_price,
        volatility,
        total_volume,
        price_change_pct,
    })
}

fn main() -> std::io::Result<()> {
    let input_file = File::open("btc_1m_candles.csv")?;
    let reader = BufReader::new(input_file);

    let output_file = File::create("daily_stats.csv")?;
    let mut writer = BufWriter::new(output_file);

    writeln!(writer, "date,avg_price,volatility_pct,total_volume,change_pct")?;

    let mut candles: Vec<OHLCV> = Vec::new();
    let mut current_day: Option<u64> = None;

    for line in reader.lines().skip(1) {  // Пропускаем заголовок
        let line = line?;

        if let Some(candle) = parse_ohlcv(&line) {
            let day = candle.timestamp / 86400;  // Группируем по дням

            match current_day {
                None => {
                    current_day = Some(day);
                    candles.push(candle);
                }
                Some(d) if d == day => {
                    candles.push(candle);
                }
                Some(_) => {
                    // Новый день — записываем статистику предыдущего
                    if let Some(stats) = calculate_stats(&candles) {
                        writeln!(
                            writer,
                            "{},{:.2},{:.2},{:.0},{:.2}",
                            stats.date,
                            stats.avg_price,
                            stats.volatility,
                            stats.total_volume,
                            stats.price_change_pct
                        )?;
                    }

                    candles.clear();
                    candles.push(candle);
                    current_day = Some(day);
                }
            }
        }
    }

    // Записываем последний день
    if let Some(stats) = calculate_stats(&candles) {
        writeln!(
            writer,
            "{},{:.2},{:.2},{:.0},{:.2}",
            stats.date,
            stats.avg_price,
            stats.volatility,
            stats.total_volume,
            stats.price_change_pct
        )?;
    }

    writer.flush()?;
    println!("Анализ завершён!");

    Ok(())
}
```

## Чтение бинарных данных

Для максимальной производительности торговые данные часто хранят в бинарном формате:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

#[repr(C, packed)]
struct Tick {
    timestamp: u64,
    price: f64,
    volume: f64,
}

fn write_ticks(filename: &str, ticks: &[Tick]) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for tick in ticks {
        // Записываем структуру как байты
        let bytes: [u8; 24] = unsafe {
            std::mem::transmute_copy(tick)
        };
        writer.write_all(&bytes)?;
    }

    writer.flush()?;
    Ok(())
}

fn read_ticks(filename: &str) -> std::io::Result<Vec<Tick>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let mut ticks = Vec::new();
    let mut buffer = [0u8; 24];

    while reader.read_exact(&mut buffer).is_ok() {
        let tick: Tick = unsafe {
            std::mem::transmute_copy(&buffer)
        };
        ticks.push(tick);
    }

    Ok(ticks)
}

fn main() -> std::io::Result<()> {
    // Создаём тестовые тики
    let ticks = vec![
        Tick { timestamp: 1703980800000, price: 42150.50, volume: 1.5 },
        Tick { timestamp: 1703980800100, price: 42151.00, volume: 0.3 },
        Tick { timestamp: 1703980800200, price: 42149.75, volume: 2.1 },
    ];

    write_ticks("ticks.bin", &ticks)?;
    println!("Записано {} тиков", ticks.len());

    let loaded = read_ticks("ticks.bin")?;
    println!("Загружено {} тиков", loaded.len());

    for tick in &loaded {
        println!(
            "Время: {}, Цена: {:.2}, Объём: {:.2}",
            tick.timestamp, tick.price, tick.volume
        );
    }

    Ok(())
}
```

## Streaming обработка больших файлов

Когда файл не помещается в память:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

struct TradeSummary {
    total_trades: u64,
    total_volume: f64,
    total_value: f64,
    max_price: f64,
    min_price: f64,
}

impl TradeSummary {
    fn new() -> Self {
        TradeSummary {
            total_trades: 0,
            total_volume: 0.0,
            total_value: 0.0,
            max_price: f64::MIN,
            min_price: f64::MAX,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.total_trades += 1;
        self.total_volume += volume;
        self.total_value += price * volume;
        self.max_price = self.max_price.max(price);
        self.min_price = self.min_price.min(price);
    }

    fn vwap(&self) -> f64 {
        if self.total_volume > 0.0 {
            self.total_value / self.total_volume
        } else {
            0.0
        }
    }
}

fn process_large_file(input: &str, output: &str) -> std::io::Result<()> {
    let file = File::open(input)?;
    let reader = BufReader::with_capacity(128 * 1024, file);  // 128KB буфер

    let out_file = File::create(output)?;
    let mut writer = BufWriter::new(out_file);

    let mut summary = TradeSummary::new();
    let mut processed_lines = 0u64;

    writeln!(writer, "Processing started...")?;

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 3 {
            if let (Ok(price), Ok(volume)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                summary.update(price, volume);
            }
        }

        processed_lines += 1;

        // Промежуточный отчёт каждый миллион строк
        if processed_lines % 1_000_000 == 0 {
            writeln!(
                writer,
                "Processed {} M lines, current VWAP: {:.2}",
                processed_lines / 1_000_000,
                summary.vwap()
            )?;
            writer.flush()?;  // Сбрасываем буфер для немедленного вывода
        }
    }

    // Финальный отчёт
    writeln!(writer, "\n=== FINAL SUMMARY ===")?;
    writeln!(writer, "Total trades: {}", summary.total_trades)?;
    writeln!(writer, "Total volume: {:.4}", summary.total_volume)?;
    writeln!(writer, "VWAP: {:.2}", summary.vwap())?;
    writeln!(writer, "Price range: {:.2} - {:.2}", summary.min_price, summary.max_price)?;

    writer.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    process_large_file("all_trades_2023.csv", "summary_report.txt")
}
```

## Обработка ошибок при I/O

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

fn safe_process_trades(input_path: &str, output_path: &str) -> Result<usize, String> {
    // Открываем файл с обработкой ошибки
    let input_file = File::open(input_path)
        .map_err(|e| format!("Не удалось открыть {}: {}", input_path, e))?;

    let output_file = File::create(output_path)
        .map_err(|e| format!("Не удалось создать {}: {}", output_path, e))?;

    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    let mut processed = 0;
    let mut errors = 0;

    for (line_num, line_result) in reader.lines().enumerate() {
        match line_result {
            Ok(line) => {
                // Обрабатываем строку
                if let Err(e) = writeln!(writer, "Processed: {}", line) {
                    return Err(format!("Ошибка записи на строке {}: {}", line_num, e));
                }
                processed += 1;
            }
            Err(e) => {
                eprintln!("Ошибка чтения строки {}: {}", line_num, e);
                errors += 1;
                // Продолжаем обработку
            }
        }
    }

    writer.flush()
        .map_err(|e| format!("Ошибка при сбросе буфера: {}", e))?;

    if errors > 0 {
        eprintln!("Предупреждение: {} строк с ошибками", errors);
    }

    Ok(processed)
}

fn main() {
    match safe_process_trades("trades.csv", "processed.csv") {
        Ok(count) => println!("Успешно обработано {} записей", count),
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}
```

## Сравнение производительности

```rust
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, BufRead};
use std::time::Instant;

fn benchmark_unbuffered_read(filename: &str) -> std::io::Result<(usize, u128)> {
    let start = Instant::now();
    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 1];
    let mut count = 0;

    while file.read(&mut buffer)? > 0 {
        count += 1;
    }

    Ok((count, start.elapsed().as_millis()))
}

fn benchmark_buffered_read(filename: &str) -> std::io::Result<(usize, u128)> {
    let start = Instant::now();
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        line?;
        count += 1;
    }

    Ok((count, start.elapsed().as_millis()))
}

fn main() -> std::io::Result<()> {
    // Создаём тестовый файл
    {
        let file = File::create("benchmark_data.csv")?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "timestamp,price,volume")?;
        for i in 0..100_000 {
            writeln!(writer, "{},{:.2},{:.4}", 1700000000 + i, 42000.0 + (i as f64 * 0.01), 0.1)?;
        }
        writer.flush()?;
    }

    println!("Бенчмарк чтения файла:");
    println!("========================");

    let (lines, time) = benchmark_buffered_read("benchmark_data.csv")?;
    println!("С буфером: {} строк за {} мс", lines, time);

    // Примечание: unbuffered будет очень медленным для больших файлов
    // Раскомментируйте только для маленьких тестов
    // let (bytes, time) = benchmark_unbuffered_read("benchmark_data.csv")?;
    // println!("Без буфера: {} байт за {} мс", bytes, time);

    Ok(())
}
```

## Что мы узнали

| Тип | Назначение | Когда использовать |
|-----|------------|-------------------|
| `BufReader` | Буферизованное чтение | Чтение файлов построчно или частями |
| `BufWriter` | Буферизованная запись | Множественная запись в файл |
| `with_capacity` | Настройка размера буфера | Большие файлы, оптимизация |
| `flush()` | Принудительная запись | Гарантия записи данных на диск |
| `lines()` | Итератор по строкам | Чтение текстовых файлов |

## Практические задания

1. **Конвертер форматов**: Напиши программу, которая читает CSV файл с ценами и конвертирует его в JSON формат. Используй `BufReader` и `BufWriter`.

2. **Агрегатор тиков**: Создай программу, которая читает файл с тиковыми данными (timestamp, price, volume) и агрегирует их в минутные свечи (OHLCV).

3. **Поиск паттернов**: Напиши программу, которая ищет в файле все сделки объёмом больше заданного порога и записывает их в отдельный файл.

4. **Слияние файлов**: Реализуй программу, которая читает несколько CSV файлов с ценами разных активов и объединяет их в один файл с сортировкой по времени.

## Домашнее задание

1. Напиши функцию `calculate_rolling_vwap(input: &str, output: &str, window: usize)` — считает скользящий VWAP и записывает результат в файл.

2. Создай `LogRotator` — структуру, которая автоматически создаёт новый файл для записи, когда текущий превышает заданный размер (важно для торговых логов).

3. Реализуй `TradeFilter` — читает сделки из файла и записывает только те, которые соответствуют критериям (symbol, min_volume, time_range).

4. Напиши бенчмарк, сравнивающий скорость чтения одного файла: без буфера, с буфером 8KB, 64KB, 512KB. Найди оптимальный размер для твоей системы.

## Навигация

[← Предыдущий день](../123-file-io-price-history/ru.md) | [Следующий день →](../125-serde-json-api-data/ru.md)
