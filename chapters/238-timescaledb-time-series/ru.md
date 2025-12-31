# День 238: TimescaleDB: Временные ряды — Хранение рыночной истории

## Торговая аналогия

Каждый трейдер знает, что рынки имеют память. Движения цен, всплески объёмов, поток ордеров — все эти данные привязаны к временной метке. График свечей — это не просто визуализация; это временной ряд данных OHLCV (Open, High, Low, Close, Volume). Для анализа рынка, бэктестинга стратегий или обнаружения паттернов необходимо эффективно хранить и запрашивать миллионы таких записей с временными метками.

Традиционные базы данных плохо справляются с временными рядами в больших масштабах. Представьте попытку выполнить запрос «Какой был средний объём BTC/USDT в 5-минутных интервалах за последний год?» к таблице со 100 миллионами строк. Именно здесь блистает **TimescaleDB** — это PostgreSQL, оптимизированный для временных рядов, автоматически партиционирующий ваши данные по времени и обеспечивающий молниеносные агрегации.

## Что такое TimescaleDB?

TimescaleDB — это база данных временных рядов с открытым исходным кодом, построенная на PostgreSQL. Она предоставляет:

- **Гипертаблицы**: Автоматическое партиционирование по времени
- **Непрерывные агрегаты**: Предвычисленные свёртки, обновляющиеся автоматически
- **Сжатие**: До 95% уменьшение объёма хранения для исторических данных
- **Полный SQL**: Все возможности PostgreSQL, включая джойны и индексы
- **Функции временных интервалов**: Простая агрегация по временным периодам

Для торговых приложений это означает:
- Эффективное хранение тиковых данных
- Запросы исторических OHLCV-свечей за миллисекунды
- Расчёт скользящих средних, VWAP и других индикаторов на лету
- Объединение ценовых данных с историей ордеров и снимками портфеля

## Настройка TimescaleDB с Rust

Сначала добавьте необходимые зависимости в `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = "1.33"
serde = { version = "1.0", features = ["derive"] }
```

## Подключение к TimescaleDB

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Подключение к TimescaleDB (аналогично подключению к PostgreSQL)
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    // Запускаем обработчик соединения
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    // Проверяем, что расширение TimescaleDB включено
    let row = client
        .query_one("SELECT extversion FROM pg_extension WHERE extname = 'timescaledb'", &[])
        .await?;

    let version: &str = row.get(0);
    println!("Версия TimescaleDB: {}", version);

    Ok(())
}
```

## Создание гипертаблицы для данных OHLCV

Гипертаблица — это ключевая концепция TimescaleDB. Она выглядит как обычная таблица, но автоматически партиционирует данные по времени:

```rust
use tokio_postgres::{NoTls, Error};

async fn create_ohlcv_table(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Создаём базовую таблицу
    client.execute(
        "CREATE TABLE IF NOT EXISTS ohlcv (
            time        TIMESTAMPTZ NOT NULL,
            symbol      VARCHAR(20) NOT NULL,
            open        DECIMAL(18, 8) NOT NULL,
            high        DECIMAL(18, 8) NOT NULL,
            low         DECIMAL(18, 8) NOT NULL,
            close       DECIMAL(18, 8) NOT NULL,
            volume      DECIMAL(24, 8) NOT NULL,
            trade_count INTEGER DEFAULT 0
        )",
        &[],
    ).await?;

    // Преобразуем в гипертаблицу с чанками по 1 дню
    // Если таблица уже является гипертаблицей, будет ошибка (обработайте соответственно)
    match client.execute(
        "SELECT create_hypertable('ohlcv', 'time',
            chunk_time_interval => INTERVAL '1 day',
            if_not_exists => TRUE
        )",
        &[],
    ).await {
        Ok(_) => println!("Гипертаблица успешно создана"),
        Err(e) => println!("Гипертаблица может уже существовать: {}", e),
    }

    // Создаём индексы для частых паттернов запросов
    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_ohlcv_symbol_time
         ON ohlcv (symbol, time DESC)",
        &[],
    ).await?;

    println!("Таблица OHLCV готова для временных рядов!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    create_ohlcv_table(&client).await?;
    Ok(())
}
```

## Вставка OHLCV-свечей

```rust
use chrono::{DateTime, Utc};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
struct Candle {
    time: DateTime<Utc>,
    symbol: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    trade_count: i32,
}

async fn insert_candle(client: &tokio_postgres::Client, candle: &Candle) -> Result<(), Error> {
    client.execute(
        "INSERT INTO ohlcv (time, symbol, open, high, low, close, volume, trade_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &candle.time,
            &candle.symbol,
            &candle.open,
            &candle.high,
            &candle.low,
            &candle.close,
            &candle.volume,
            &candle.trade_count,
        ],
    ).await?;

    Ok(())
}

async fn insert_candles_batch(
    client: &tokio_postgres::Client,
    candles: &[Candle]
) -> Result<u64, Error> {
    // Для массовых вставок используйте COPY или пакетный INSERT
    let mut count = 0u64;

    for candle in candles {
        client.execute(
            "INSERT INTO ohlcv (time, symbol, open, high, low, close, volume, trade_count)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT DO NOTHING",
            &[
                &candle.time,
                &candle.symbol,
                &candle.open,
                &candle.high,
                &candle.low,
                &candle.close,
                &candle.volume,
                &candle.trade_count,
            ],
        ).await?;
        count += 1;
    }

    println!("Вставлено {} свечей", count);
    Ok(count)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    // Пример: вставка одной свечи
    let candle = Candle {
        time: Utc::now(),
        symbol: "BTC/USDT".to_string(),
        open: Decimal::new(4250000, 2),    // 42500.00
        high: Decimal::new(4275000, 2),    // 42750.00
        low: Decimal::new(4240000, 2),     // 42400.00
        close: Decimal::new(4268000, 2),   // 42680.00
        volume: Decimal::new(15234567, 4), // 1523.4567
        trade_count: 4521,
    };

    insert_candle(&client, &candle).await?;
    println!("Свеча вставлена: {:?}", candle);

    Ok(())
}
```

## Запросы к временным рядам

### Базовые запросы

```rust
use chrono::{DateTime, Utc, Duration};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

async fn get_latest_candles(
    client: &tokio_postgres::Client,
    symbol: &str,
    limit: i64,
) -> Result<Vec<(DateTime<Utc>, Decimal, Decimal, Decimal, Decimal)>, Error> {
    let rows = client.query(
        "SELECT time, open, high, low, close
         FROM ohlcv
         WHERE symbol = $1
         ORDER BY time DESC
         LIMIT $2",
        &[&symbol, &limit],
    ).await?;

    let candles: Vec<_> = rows.iter().map(|row| {
        (
            row.get::<_, DateTime<Utc>>(0),
            row.get::<_, Decimal>(1),
            row.get::<_, Decimal>(2),
            row.get::<_, Decimal>(3),
            row.get::<_, Decimal>(4),
        )
    }).collect();

    Ok(candles)
}

async fn get_price_range(
    client: &tokio_postgres::Client,
    symbol: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<(Decimal, Decimal), Error> {
    let row = client.query_one(
        "SELECT MIN(low) as period_low, MAX(high) as period_high
         FROM ohlcv
         WHERE symbol = $1 AND time >= $2 AND time < $3",
        &[&symbol, &start, &end],
    ).await?;

    Ok((row.get(0), row.get(1)))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    // Получить последние 10 свечей для BTC/USDT
    let candles = get_latest_candles(&client, "BTC/USDT", 10).await?;
    println!("Последние свечи:");
    for (time, open, high, low, close) in candles {
        println!("  {} | O:{} H:{} L:{} C:{}",
            time.format("%Y-%m-%d %H:%M"),
            open, high, low, close
        );
    }

    Ok(())
}
```

## Агрегации по временным интервалам

Функция `time_bucket` в TimescaleDB идеально подходит для создания свечей произвольных таймфреймов:

```rust
use chrono::{DateTime, Utc};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug)]
struct AggregatedCandle {
    bucket: DateTime<Utc>,
    symbol: String,
    open: Decimal,
    high: Decimal,
    low: Decimal,
    close: Decimal,
    volume: Decimal,
    trade_count: i64,
}

async fn get_candles_by_timeframe(
    client: &tokio_postgres::Client,
    symbol: &str,
    interval: &str,  // например, "5 minutes", "1 hour", "1 day"
    limit: i64,
) -> Result<Vec<AggregatedCandle>, Error> {
    // Используем функции time_bucket и first/last TimescaleDB
    let query = format!(
        "SELECT
            time_bucket('{}', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count
         FROM ohlcv
         WHERE symbol = $1
         GROUP BY bucket, symbol
         ORDER BY bucket DESC
         LIMIT $2",
        interval
    );

    let rows = client.query(&query, &[&symbol, &limit]).await?;

    let candles: Vec<AggregatedCandle> = rows.iter().map(|row| {
        AggregatedCandle {
            bucket: row.get(0),
            symbol: row.get(1),
            open: row.get(2),
            high: row.get(3),
            low: row.get(4),
            close: row.get(5),
            volume: row.get(6),
            trade_count: row.get(7),
        }
    }).collect();

    Ok(candles)
}

async fn calculate_vwap(
    client: &tokio_postgres::Client,
    symbol: &str,
    interval: &str,
) -> Result<Vec<(DateTime<Utc>, Decimal)>, Error> {
    // Расчёт средневзвешенной по объёму цены (VWAP)
    let query = format!(
        "SELECT
            time_bucket('{}', time) AS bucket,
            SUM((high + low + close) / 3 * volume) / SUM(volume) AS vwap
         FROM ohlcv
         WHERE symbol = $1
         GROUP BY bucket
         ORDER BY bucket DESC
         LIMIT 100",
        interval
    );

    let rows = client.query(&query, &[&symbol]).await?;

    let vwap_data: Vec<(DateTime<Utc>, Decimal)> = rows.iter().map(|row| {
        (row.get(0), row.get(1))
    }).collect();

    Ok(vwap_data)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    // Получить часовые свечи
    println!("=== Часовые свечи ===");
    let hourly = get_candles_by_timeframe(&client, "BTC/USDT", "1 hour", 24).await?;
    for candle in hourly.iter().take(5) {
        println!("{}: O={} H={} L={} C={} V={}",
            candle.bucket.format("%Y-%m-%d %H:%M"),
            candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }

    // Рассчитать VWAP
    println!("\n=== Часовой VWAP ===");
    let vwap = calculate_vwap(&client, "BTC/USDT", "1 hour").await?;
    for (time, price) in vwap.iter().take(5) {
        println!("{}: VWAP = {}", time.format("%Y-%m-%d %H:%M"), price);
    }

    Ok(())
}
```

## Непрерывные агрегаты для дашбордов реального времени

Непрерывные агрегаты — это материализованные представления, которые автоматически обновляются:

```rust
use tokio_postgres::{NoTls, Error};

async fn create_continuous_aggregates(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Создаём непрерывный агрегат для часовых данных
    client.execute(
        "CREATE MATERIALIZED VIEW IF NOT EXISTS ohlcv_hourly
         WITH (timescaledb.continuous) AS
         SELECT
            time_bucket('1 hour', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count,
            count(*) AS candle_count
         FROM ohlcv
         GROUP BY bucket, symbol
         WITH NO DATA",
        &[],
    ).await?;

    // Создаём дневной агрегат
    client.execute(
        "CREATE MATERIALIZED VIEW IF NOT EXISTS ohlcv_daily
         WITH (timescaledb.continuous) AS
         SELECT
            time_bucket('1 day', time) AS bucket,
            symbol,
            first(open, time) AS open,
            max(high) AS high,
            min(low) AS low,
            last(close, time) AS close,
            sum(volume) AS volume,
            sum(trade_count) AS trade_count
         FROM ohlcv
         GROUP BY bucket, symbol
         WITH NO DATA",
        &[],
    ).await?;

    // Настраиваем автоматическое обновление (каждый час для часовых данных)
    client.execute(
        "SELECT add_continuous_aggregate_policy('ohlcv_hourly',
            start_offset => INTERVAL '3 hours',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour'
        )",
        &[],
    ).await?;

    println!("Непрерывные агрегаты созданы!");
    Ok(())
}

async fn query_hourly_aggregate(
    client: &tokio_postgres::Client,
    symbol: &str,
    limit: i64,
) -> Result<(), Error> {
    // Запрос к предвычисленному агрегату — намного быстрее!
    let rows = client.query(
        "SELECT bucket, open, high, low, close, volume
         FROM ohlcv_hourly
         WHERE symbol = $1
         ORDER BY bucket DESC
         LIMIT $2",
        &[&symbol, &limit],
    ).await?;

    println!("Часовые данные из непрерывного агрегата:");
    for row in rows {
        let bucket: chrono::DateTime<chrono::Utc> = row.get(0);
        let close: rust_decimal::Decimal = row.get(4);
        let volume: rust_decimal::Decimal = row.get(5);
        println!("  {} | Close: {} | Volume: {}",
            bucket.format("%Y-%m-%d %H:%M"), close, volume
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    create_continuous_aggregates(&client).await?;
    query_hourly_aggregate(&client, "BTC/USDT", 24).await?;

    Ok(())
}
```

## Практический пример: Торговый аналитический дашборд

```rust
use chrono::{DateTime, Utc, Duration};
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug)]
struct MarketStats {
    symbol: String,
    current_price: Decimal,
    price_change_24h: Decimal,
    high_24h: Decimal,
    low_24h: Decimal,
    volume_24h: Decimal,
    vwap_24h: Decimal,
    trade_count_24h: i64,
}

#[derive(Debug)]
struct MovingAverages {
    symbol: String,
    sma_20: Decimal,
    sma_50: Decimal,
    ema_12: Decimal,
    ema_26: Decimal,
}

async fn get_market_stats(
    client: &tokio_postgres::Client,
    symbol: &str,
) -> Result<MarketStats, Error> {
    let now = Utc::now();
    let yesterday = now - Duration::hours(24);

    let row = client.query_one(
        "WITH latest AS (
            SELECT close as current_price
            FROM ohlcv
            WHERE symbol = $1
            ORDER BY time DESC
            LIMIT 1
        ),
        day_ago AS (
            SELECT close as price_24h_ago
            FROM ohlcv
            WHERE symbol = $1 AND time <= $2
            ORDER BY time DESC
            LIMIT 1
        ),
        stats AS (
            SELECT
                MAX(high) as high_24h,
                MIN(low) as low_24h,
                SUM(volume) as volume_24h,
                SUM((high + low + close) / 3 * volume) / NULLIF(SUM(volume), 0) as vwap,
                SUM(trade_count) as trade_count
            FROM ohlcv
            WHERE symbol = $1 AND time >= $2
        )
        SELECT
            l.current_price,
            l.current_price - COALESCE(d.price_24h_ago, l.current_price) as change,
            s.high_24h,
            s.low_24h,
            s.volume_24h,
            COALESCE(s.vwap, l.current_price) as vwap,
            COALESCE(s.trade_count, 0) as trade_count
        FROM latest l
        CROSS JOIN stats s
        LEFT JOIN day_ago d ON true",
        &[&symbol, &yesterday],
    ).await?;

    Ok(MarketStats {
        symbol: symbol.to_string(),
        current_price: row.get(0),
        price_change_24h: row.get(1),
        high_24h: row.get(2),
        low_24h: row.get(3),
        volume_24h: row.get(4),
        vwap_24h: row.get(5),
        trade_count_24h: row.get(6),
    })
}

async fn calculate_moving_averages(
    client: &tokio_postgres::Client,
    symbol: &str,
    timeframe: &str,
) -> Result<MovingAverages, Error> {
    // Рассчитываем SMA с использованием time_bucket
    let row = client.query_one(
        &format!(
            "WITH candles AS (
                SELECT
                    time_bucket('{}', time) as bucket,
                    last(close, time) as close
                FROM ohlcv
                WHERE symbol = $1
                GROUP BY bucket
                ORDER BY bucket DESC
                LIMIT 50
            ),
            numbered AS (
                SELECT close, ROW_NUMBER() OVER (ORDER BY bucket DESC) as rn
                FROM candles
            )
            SELECT
                (SELECT AVG(close) FROM numbered WHERE rn <= 20) as sma_20,
                (SELECT AVG(close) FROM numbered WHERE rn <= 50) as sma_50,
                (SELECT AVG(close) FROM numbered WHERE rn <= 12) as ema_12_approx,
                (SELECT AVG(close) FROM numbered WHERE rn <= 26) as ema_26_approx
            ", timeframe
        ),
        &[&symbol],
    ).await?;

    Ok(MovingAverages {
        symbol: symbol.to_string(),
        sma_20: row.get(0),
        sma_50: row.get(1),
        ema_12: row.get(2),
        ema_26: row.get(3),
    })
}

fn display_dashboard(stats: &MarketStats, ma: &MovingAverages) {
    let change_pct = if stats.current_price != Decimal::ZERO {
        (stats.price_change_24h / stats.current_price) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let trend = if stats.price_change_24h > Decimal::ZERO { "+" } else { "" };

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    ТОРГОВЫЙ ДАШБОРД                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Символ: {:<55} ║", stats.symbol);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ Текущая цена:   ${:<46} ║", stats.current_price);
    println!("║ Изменение 24ч:  {}{} ({}{:.2}%)                              ║",
        trend, stats.price_change_24h, trend, change_pct);
    println!("║ Макс. 24ч:      ${:<46} ║", stats.high_24h);
    println!("║ Мин. 24ч:       ${:<46} ║", stats.low_24h);
    println!("║ Объём 24ч:      {:<47} ║", stats.volume_24h);
    println!("║ VWAP 24ч:       ${:<46} ║", stats.vwap_24h);
    println!("║ Сделок 24ч:     {:<47} ║", stats.trade_count_24h);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                   СКОЛЬЗЯЩИЕ СРЕДНИЕ                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ SMA(20):        ${:<46} ║", ma.sma_20);
    println!("║ SMA(50):        ${:<46} ║", ma.sma_50);
    println!("║ EMA(12):        ${:<46} ║", ma.ema_12);
    println!("║ EMA(26):        ${:<46} ║", ma.ema_26);
    println!("╠══════════════════════════════════════════════════════════════╣");

    // Простой анализ тренда
    let trend_signal = if stats.current_price > ma.sma_20 && ma.sma_20 > ma.sma_50 {
        "БЫЧИЙ - Цена выше SMA, краткосрочный восходящий тренд"
    } else if stats.current_price < ma.sma_20 && ma.sma_20 < ma.sma_50 {
        "МЕДВЕЖИЙ - Цена ниже SMA, краткосрочный нисходящий тренд"
    } else {
        "НЕЙТРАЛЬНЫЙ - Смешанные сигналы, фаза консолидации"
    };

    println!("║ Тренд:          {:<46} ║", trend_signal);
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    let symbol = "BTC/USDT";

    // Получаем статистику рынка и скользящие средние
    let stats = get_market_stats(&client, symbol).await?;
    let ma = calculate_moving_averages(&client, symbol, "1 hour").await?;

    // Отображаем дашборд
    display_dashboard(&stats, &ma);

    Ok(())
}
```

## Сжатие исторических данных

TimescaleDB может сжимать старые данные для экономии места:

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_compression(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Включаем сжатие для гипертаблицы
    client.execute(
        "ALTER TABLE ohlcv SET (
            timescaledb.compress,
            timescaledb.compress_segmentby = 'symbol',
            timescaledb.compress_orderby = 'time DESC'
        )",
        &[],
    ).await?;

    // Добавляем политику сжатия: сжимать чанки старше 7 дней
    client.execute(
        "SELECT add_compression_policy('ohlcv', INTERVAL '7 days')",
        &[],
    ).await?;

    println!("Политика сжатия включена!");

    // Проверяем статистику сжатия
    let rows = client.query(
        "SELECT
            hypertable_name,
            total_chunks,
            compressed_chunks,
            pg_size_pretty(before_compression_total_bytes) as before,
            pg_size_pretty(after_compression_total_bytes) as after
         FROM timescaledb_information.compression_stats",
        &[],
    ).await?;

    for row in rows {
        let table: &str = row.get(0);
        let total: i64 = row.get(1);
        let compressed: i64 = row.get(2);
        let before: &str = row.get(3);
        let after: &str = row.get(4);

        println!("Таблица: {} | Чанки: {}/{} сжато | Размер: {} -> {}",
            table, compressed, total, before, after
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    setup_compression(&client).await?;
    Ok(())
}
```

## Политики хранения данных

Автоматическое удаление старых данных для управления хранилищем:

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_retention_policy(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Храним только последние 365 дней минутных данных
    client.execute(
        "SELECT add_retention_policy('ohlcv', INTERVAL '365 days')",
        &[],
    ).await?;

    println!("Политика хранения: последние 365 дней данных");

    // Для непрерывных агрегатов храним дольше
    // Часовые данные: 2 года
    client.execute(
        "SELECT add_retention_policy('ohlcv_hourly', INTERVAL '730 days')",
        &[],
    ).await?;

    // Дневные данные: 5 лет
    client.execute(
        "SELECT add_retention_policy('ohlcv_daily', INTERVAL '1825 days')",
        &[],
    ).await?;

    println!("Хранение агрегатов: часовые=2 года, дневные=5 лет");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader password=secret dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    setup_retention_policy(&client).await?;
    Ok(())
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|------------------------|
| Гипертаблица | Автоматически партиционируемая таблица временных рядов | Хранение миллионов OHLCV-свечей эффективно |
| `time_bucket()` | Агрегация данных по временным интервалам | Создание свечей произвольных таймфреймов (5м, 1ч, 1д) |
| `first()` / `last()` | Получение первого/последнего значения в интервале | Расчёт цен открытия/закрытия в агрегациях |
| Непрерывные агрегаты | Автообновляемые материализованные представления | Дашборды реального времени, предвычисленные индикаторы |
| Сжатие | Уменьшение объёма для исторических данных | Хранение многолетних тиковых данных экономично |
| Политика хранения | Автоматическая очистка данных | Гранулярные данные на X дней, агрегаты дольше |

## Домашнее задание

1. **Хранение тиковых данных**: Создайте гипертаблицу для хранения отдельных сделок (тиковые данные) с полями: `time`, `symbol`, `price`, `quantity`, `side` (покупка/продажа). Напишите функции для:
   - Вставки тиков пакетами по 1000
   - Агрегации тиков в 1-минутные OHLCV-свечи с использованием `time_bucket`
   - Расчёта спреда bid-ask из тиковых данных

2. **Мультисимвольный дашборд**: Расширьте пример торгового дашборда:
   - Отслеживание нескольких символов одновременно (BTC, ETH, SOL)
   - Показ корреляции между символами за последние 24 часа
   - Определение символа с наибольшей волатильностью (используя `stddev()` на доходностях)

3. **Пайплайн данных для бэктестинга**: Постройте пайплайн данных, который:
   - Хранит данные OHLCV для бэктестинга
   - Предоставляет функцию получения данных за диапазон дат с заданным таймфреймом
   - Реализует простой генератор сигналов пересечения скользящих средних
   - Сохраняет сигналы в отдельную гипертаблицу с временными метками

4. **Сравнение производительности**: Создайте бенчмарк, который:
   - Вставляет 1 миллион свечей в обычную таблицу PostgreSQL и в гипертаблицу TimescaleDB
   - Сравнивает производительность запросов: последние N свечей, запросы по диапазону времени, агрегации
   - Измеряет коэффициент сжатия и экономию хранилища
   - Документирует результаты со статистикой по времени выполнения

## Навигация

[← Предыдущий день](../237-redis-pub-sub-notifications/ru.md) | [Следующий день →](../239-clickhouse-big-data-analytics/ru.md)
