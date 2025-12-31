# День 148: Сжатие данных — работа с большими файлами

## Аналогия из трейдинга

Представь, что ты хранишь историю тиков за год — миллионы записей. Без сжатия файл занимает 10 ГБ, а со сжатием — всего 1 ГБ. Это как упаковать огромный склад в компактный контейнер для транспортировки. Сжатие данных — критически важная техника для трейдеров, работающих с историческими данными, бэктестингом и хранением логов торговых операций.

## Зачем сжимать данные в трейдинге

1. **Экономия дискового пространства** — история тиков за годы занимает терабайты
2. **Быстрая передача по сети** — сжатые данные быстрее скачиваются с бирж
3. **Архивирование** — старые логи и бэкапы в сжатом виде
4. **Стриминг данных** — сжатие в реальном времени для высокочастотного трейдинга

## Подключаем зависимости

Добавь в `Cargo.toml`:

```toml
[dependencies]
flate2 = "1.0"
```

Крейт `flate2` предоставляет сжатие gzip, deflate и zlib — самые распространённые алгоритмы.

## Базовое сжатие строки

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

fn main() {
    let original = "BTC,42000.50,1.5,buy\nETH,2800.00,10.0,sell\n".repeat(1000);
    println!("Оригинал: {} байт", original.len());

    // Сжимаем
    let compressed = compress_data(original.as_bytes()).unwrap();
    println!("Сжато: {} байт", compressed.len());
    println!("Коэффициент сжатия: {:.2}x", original.len() as f64 / compressed.len() as f64);

    // Распаковываем
    let decompressed = decompress_data(&compressed).unwrap();
    let restored = String::from_utf8(decompressed).unwrap();

    assert_eq!(original, restored);
    println!("Данные восстановлены корректно!");
}

fn compress_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

fn decompress_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}
```

## Уровни сжатия

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::time::Instant;

fn main() {
    // Генерируем тестовые данные: история сделок
    let trades: String = (0..100_000)
        .map(|i| format!("BTCUSD,{:.2},{:.4},buy,{}\n",
            42000.0 + (i as f64 * 0.01).sin() * 100.0,
            0.001 + (i as f64 * 0.1).cos().abs(),
            1700000000 + i
        ))
        .collect();

    let data = trades.as_bytes();
    println!("Исходный размер: {} байт\n", data.len());

    let levels = [
        (Compression::none(), "Без сжатия"),
        (Compression::fast(), "Быстрое (уровень 1)"),
        (Compression::default(), "Стандартное (уровень 6)"),
        (Compression::best(), "Максимальное (уровень 9)"),
    ];

    for (level, name) in levels {
        let start = Instant::now();

        let mut encoder = GzEncoder::new(Vec::new(), level);
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();

        let elapsed = start.elapsed();
        let ratio = data.len() as f64 / compressed.len() as f64;

        println!("{}: {} байт ({:.2}x) за {:?}",
            name, compressed.len(), ratio, elapsed);
    }
}
```

## Работа с файлами: сжатие истории цен

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read, BufRead};

fn main() -> std::io::Result<()> {
    // Создаём тестовый файл с историей OHLCV
    create_sample_ohlcv("price_history.csv")?;

    // Сжимаем файл
    compress_file("price_history.csv", "price_history.csv.gz")?;

    // Сравниваем размеры
    let original_size = std::fs::metadata("price_history.csv")?.len();
    let compressed_size = std::fs::metadata("price_history.csv.gz")?.len();

    println!("Оригинал: {} байт", original_size);
    println!("Сжатый: {} байт", compressed_size);
    println!("Экономия: {:.1}%",
        (1.0 - compressed_size as f64 / original_size as f64) * 100.0);

    // Читаем сжатый файл напрямую
    println!("\nПервые 5 строк из сжатого файла:");
    read_compressed_lines("price_history.csv.gz", 5)?;

    Ok(())
}

fn create_sample_ohlcv(path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "timestamp,open,high,low,close,volume")?;

    let mut price = 42000.0_f64;
    for i in 0..50_000 {
        let volatility = (i as f64 * 0.1).sin() * 100.0;
        let open = price;
        let high = price + volatility.abs() + 50.0;
        let low = price - volatility.abs() - 50.0;
        let close = price + volatility;
        let volume = 100.0 + (i as f64 * 0.05).cos().abs() * 1000.0;

        writeln!(writer, "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            1700000000 + i * 60, open, high, low, close, volume)?;

        price = close;
    }

    Ok(())
}

fn compress_file(input: &str, output: &str) -> std::io::Result<()> {
    let input_file = File::open(input)?;
    let mut reader = BufReader::new(input_file);

    let output_file = File::create(output)?;
    let mut encoder = GzEncoder::new(BufWriter::new(output_file), Compression::default());

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
    }

    encoder.finish()?;
    Ok(())
}

fn read_compressed_lines(path: &str, count: usize) -> std::io::Result<()> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    for line in reader.lines().take(count) {
        println!("{}", line?);
    }

    Ok(())
}
```

## Потоковое сжатие для больших файлов

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read, BufRead};

const CHUNK_SIZE: usize = 64 * 1024; // 64 КБ

fn main() -> std::io::Result<()> {
    // Создаём большой файл с тиками
    println!("Создаём большой файл с тиками...");
    create_large_tick_file("ticks.csv", 1_000_000)?;

    let original_size = std::fs::metadata("ticks.csv")?.len();
    println!("Размер файла тиков: {} МБ", original_size / 1_000_000);

    // Потоковое сжатие
    println!("Сжимаем потоково...");
    stream_compress("ticks.csv", "ticks.csv.gz")?;

    let compressed_size = std::fs::metadata("ticks.csv.gz")?.len();
    println!("Сжатый размер: {} МБ", compressed_size / 1_000_000);
    println!("Коэффициент сжатия: {:.2}x", original_size as f64 / compressed_size as f64);

    // Потоковое чтение и анализ
    println!("\nАнализируем сжатый файл...");
    let stats = analyze_compressed_ticks("ticks.csv.gz")?;
    println!("Всего тиков: {}", stats.count);
    println!("Мин. цена: {:.2}", stats.min_price);
    println!("Макс. цена: {:.2}", stats.max_price);
    println!("Общий объём: {:.4}", stats.total_volume);

    Ok(())
}

fn create_large_tick_file(path: &str, count: usize) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "timestamp,price,volume,side")?;

    let mut price = 42000.0_f64;
    for i in 0..count {
        price += ((i as f64 * 0.001).sin() * 10.0) as f64;
        let volume = 0.001 + (i as f64 * 0.0001).cos().abs();
        let side = if i % 2 == 0 { "buy" } else { "sell" };

        writeln!(writer, "{},{:.2},{:.6},{}",
            1700000000000_u64 + i as u64, price, volume, side)?;
    }

    Ok(())
}

fn stream_compress(input: &str, output: &str) -> std::io::Result<()> {
    let input_file = File::open(input)?;
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, input_file);

    let output_file = File::create(output)?;
    let mut encoder = GzEncoder::new(
        BufWriter::with_capacity(CHUNK_SIZE, output_file),
        Compression::default()
    );

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_read = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
        total_read += bytes_read as u64;

        // Прогресс каждые 10 МБ
        if total_read % (10 * 1024 * 1024) == 0 {
            print!("\rОбработано: {} МБ", total_read / 1_000_000);
        }
    }
    println!();

    encoder.finish()?;
    Ok(())
}

struct TickStats {
    count: usize,
    min_price: f64,
    max_price: f64,
    total_volume: f64,
}

fn analyze_compressed_ticks(path: &str) -> std::io::Result<TickStats> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    let mut stats = TickStats {
        count: 0,
        min_price: f64::MAX,
        max_price: f64::MIN,
        total_volume: 0.0,
    };

    for line in reader.lines().skip(1) { // Пропускаем заголовок
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 3 {
            if let (Ok(price), Ok(volume)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                stats.count += 1;
                stats.min_price = stats.min_price.min(price);
                stats.max_price = stats.max_price.max(price);
                stats.total_volume += volume;
            }
        }
    }

    Ok(stats)
}
```

## Сжатие в памяти для API ответов

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

fn main() {
    // Симулируем ответ от биржи — список ордеров
    let orders = generate_order_book_json();
    println!("Размер JSON ордербука: {} байт", orders.len());

    // Сжимаем для передачи
    let compressed = compress_for_transfer(&orders);
    println!("Сжатый размер: {} байт", compressed.len());
    println!("Экономия трафика: {:.1}%",
        (1.0 - compressed.len() as f64 / orders.len() as f64) * 100.0);

    // Распаковываем на клиенте
    let restored = decompress_from_transfer(&compressed);
    assert_eq!(orders, restored);
    println!("Данные восстановлены корректно!");
}

fn generate_order_book_json() -> String {
    let mut json = String::from(r#"{"bids":["#);

    for i in 0..1000 {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            r#"{{"price":{:.2},"qty":{:.4}}}"#,
            42000.0 - i as f64 * 0.5,
            1.0 + (i as f64 * 0.1).sin().abs()
        ));
    }

    json.push_str(r#"],"asks":["#);

    for i in 0..1000 {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            r#"{{"price":{:.2},"qty":{:.4}}}"#,
            42001.0 + i as f64 * 0.5,
            1.0 + (i as f64 * 0.1).cos().abs()
        ));
    }

    json.push_str("]}");
    json
}

fn compress_for_transfer(data: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

fn decompress_from_transfer(data: &[u8]) -> String {
    let mut decoder = GzDecoder::new(data);
    let mut result = String::new();
    decoder.read_to_string(&mut result).unwrap();
    result
}
```

## Архивирование торговых логов

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    // Создаём архиватор логов
    let mut archiver = LogArchiver::new("trading_logs")?;

    // Симулируем запись логов в течение "дня"
    for hour in 0..24 {
        for minute in 0..60 {
            let log_entry = format!(
                "[2024-01-15 {:02}:{:02}:00] TRADE executed: BTC/USD {} @ {:.2}\n",
                hour, minute,
                if (hour + minute) % 2 == 0 { "BUY" } else { "SELL" },
                42000.0 + (hour * 60 + minute) as f64 * 0.5
            );
            archiver.write_log(&log_entry)?;
        }
    }

    // Завершаем и закрываем архив
    let stats = archiver.finish()?;

    println!("Архивирование завершено:");
    println!("  Записей: {}", stats.entries);
    println!("  Байт записано: {}", stats.bytes_written);
    println!("  Размер архива: {}", stats.archive_size);
    println!("  Сжатие: {:.1}x", stats.bytes_written as f64 / stats.archive_size as f64);

    Ok(())
}

struct LogArchiver {
    encoder: GzEncoder<BufWriter<File>>,
    entries: usize,
    bytes_written: usize,
}

struct ArchiveStats {
    entries: usize,
    bytes_written: usize,
    archive_size: usize,
}

impl LogArchiver {
    fn new(base_name: &str) -> std::io::Result<Self> {
        let filename = format!("{}.log.gz", base_name);
        let file = File::create(&filename)?;
        let encoder = GzEncoder::new(
            BufWriter::new(file),
            Compression::default()
        );

        Ok(LogArchiver {
            encoder,
            entries: 0,
            bytes_written: 0,
        })
    }

    fn write_log(&mut self, entry: &str) -> std::io::Result<()> {
        self.encoder.write_all(entry.as_bytes())?;
        self.entries += 1;
        self.bytes_written += entry.len();
        Ok(())
    }

    fn finish(self) -> std::io::Result<ArchiveStats> {
        let entries = self.entries;
        let bytes_written = self.bytes_written;

        let inner = self.encoder.finish()?;
        inner.into_inner()?.sync_all()?;

        let archive_size = std::fs::metadata("trading_logs.log.gz")?.len() as usize;

        Ok(ArchiveStats {
            entries,
            bytes_written,
            archive_size,
        })
    }
}
```

## Сравнение алгоритмов сжатия

```rust
use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder, DeflateEncoder};
use std::io::Write;
use std::time::Instant;

fn main() {
    // Генерируем типичные торговые данные
    let data = generate_trading_data();
    println!("Исходный размер: {} байт\n", data.len());

    // Тестируем разные алгоритмы
    println!("{:<15} {:>12} {:>10} {:>12}", "Алгоритм", "Размер", "Сжатие", "Время");
    println!("{}", "-".repeat(52));

    // Gzip
    let (size, time) = test_gzip(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}мс",
        "Gzip", size, data.len() as f64 / size as f64, time);

    // Zlib
    let (size, time) = test_zlib(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}мс",
        "Zlib", size, data.len() as f64 / size as f64, time);

    // Deflate
    let (size, time) = test_deflate(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}мс",
        "Deflate", size, data.len() as f64 / size as f64, time);
}

fn generate_trading_data() -> Vec<u8> {
    let mut data = String::new();

    // OHLCV данные имеют высокую повторяемость — хорошо сжимаются
    for i in 0..10_000 {
        data.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.4}\n",
            1700000000 + i * 60,
            42000.0 + (i as f64 * 0.1).sin() * 100.0,
            42100.0 + (i as f64 * 0.1).sin() * 100.0,
            41900.0 + (i as f64 * 0.1).sin() * 100.0,
            42050.0 + (i as f64 * 0.1).sin() * 100.0,
            100.0 + (i as f64 * 0.05).cos().abs() * 50.0
        ));
    }

    data.into_bytes()
}

fn test_gzip(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}

fn test_zlib(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}

fn test_deflate(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}
```

## Практический пример: архив исторических данных

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    // Создаём архив с несколькими торговыми парами
    let pairs = vec!["BTCUSD", "ETHUSD", "SOLUSD"];

    for pair in &pairs {
        create_pair_archive(pair, 50_000)?;
    }

    // Читаем и анализируем все архивы
    println!("\n{:<10} {:>15} {:>15} {:>15}", "Пара", "Свечей", "Мин. цена", "Макс. цена");
    println!("{}", "-".repeat(60));

    for pair in &pairs {
        let stats = read_pair_archive(pair)?;
        println!("{:<10} {:>15} {:>15.2} {:>15.2}",
            pair, stats.count, stats.min_price, stats.max_price);
    }

    Ok(())
}

fn create_pair_archive(pair: &str, candles: usize) -> std::io::Result<()> {
    let filename = format!("{}_1m.csv.gz", pair.to_lowercase());
    let file = File::create(&filename)?;
    let mut encoder = GzEncoder::new(BufWriter::new(file), Compression::default());

    writeln!(encoder, "timestamp,open,high,low,close,volume")?;

    let base_price = match pair {
        "BTCUSD" => 42000.0,
        "ETHUSD" => 2800.0,
        "SOLUSD" => 100.0,
        _ => 1000.0,
    };

    let mut price = base_price;
    for i in 0..candles {
        let volatility = (i as f64 * 0.1).sin() * base_price * 0.001;
        let open = price;
        let high = price + volatility.abs() + base_price * 0.0005;
        let low = price - volatility.abs() - base_price * 0.0005;
        let close = price + volatility;
        let volume = 100.0 + (i as f64 * 0.05).cos().abs() * 1000.0;

        writeln!(encoder, "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            1700000000 + i * 60, open, high, low, close, volume)?;

        price = close;
    }

    let inner = encoder.finish()?;
    inner.into_inner()?.sync_all()?;

    let size = std::fs::metadata(&filename)?.len();
    println!("Создан {}: {} КБ", filename, size / 1024);

    Ok(())
}

struct PairStats {
    count: usize,
    min_price: f64,
    max_price: f64,
}

fn read_pair_archive(pair: &str) -> std::io::Result<PairStats> {
    let filename = format!("{}_1m.csv.gz", pair.to_lowercase());
    let file = File::open(&filename)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    let mut stats = PairStats {
        count: 0,
        min_price: f64::MAX,
        max_price: f64::MIN,
    };

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 5 {
            if let (Ok(low), Ok(high)) = (
                parts[3].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                stats.count += 1;
                stats.min_price = stats.min_price.min(low);
                stats.max_price = stats.max_price.max(high);
            }
        }
    }

    Ok(stats)
}
```

## Что мы узнали

| Техника | Применение | Преимущество |
|---------|------------|--------------|
| `GzEncoder` | Сжатие файлов/данных | Универсальный формат .gz |
| `GzDecoder` | Распаковка gzip | Потоковое чтение |
| `Compression::fast()` | Быстрое сжатие | Минимальная задержка |
| `Compression::best()` | Максимальное сжатие | Минимальный размер |
| Потоковая обработка | Большие файлы | Низкое потребление памяти |

## Советы по выбору сжатия для трейдинга

1. **Для real-time данных**: `Compression::fast()` — минимальная задержка
2. **Для архивов**: `Compression::best()` — максимальная экономия места
3. **Для бэктестинга**: `Compression::default()` — баланс скорости и размера
4. **Для API**: gzip — поддерживается всеми браузерами и библиотеками

## Домашнее задание

1. Создай функцию `compress_trade_history(trades: &[Trade]) -> Vec<u8>`, которая сериализует список сделок в JSON и сжимает

2. Реализуй `CompressedDataReader`, который читает сжатые CSV файлы построчно и парсит их в структуры `Candle`

3. Напиши утилиту для архивирования логов торгового бота: ротация по дням, автоматическое сжатие файлов старше 24 часов

4. Создай бенчмарк сравнения разных уровней сжатия на реальных торговых данных. Определи оптимальный уровень для твоего случая

## Навигация

[← Предыдущий день](../147-log-rotation/ru.md) | [Следующий день →](../149-streaming-data-processing/ru.md)
