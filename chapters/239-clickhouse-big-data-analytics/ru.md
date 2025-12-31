# День 239: ClickHouse: аналитика больших данных

## Аналогия из трейдинга

Представь, что ты управляешь количественным торговым деском, который обрабатывает миллионы рыночных событий в день: сделки, котировки, обновления стакана заявок и рыночные индикаторы. В конце каждого дня твоим аналитикам нужно отвечать на вопросы вроде:

- "Какой был средний спред по BTC/USDT между 14:00 и 16:00 на всех биржах?"
- "У каких торговых пар был максимальный всплеск объёма за последние 30 дней?"
- "Рассчитай VWAP (средневзвешенную по объёму цену) для каждой минуты за последний год"

Традиционная строковая база данных будет медленно ползти по этим запросам, читая целые строки, когда тебе нужны только несколько колонок. **ClickHouse** — это как специализированный аналитический отдел, который организует все твои торговые данные по колонкам: когда ты спрашиваешь о ценах, он читает только цены, а не временные метки, объёмы или другие поля. Такой колоночный подход в сочетании с мощным сжатием и параллельной обработкой позволяет ClickHouse сканировать миллиарды строк за секунды.

В реальных торговых операциях ClickHouse используется для:
- Анализа исторических тиковых данных
- Мониторинга рынка в реальном времени
- Атрибуции доходности и анализа P&L
- Расчёта факторов риска по большим портфелям
- Бэктестинга стратегий на данных за годы

## Что такое ClickHouse?

ClickHouse — это колоночная СУБД с открытым исходным кодом, разработанная для аналитической обработки данных (OLAP). Ключевые характеристики:

| Особенность | Описание |
|-------------|----------|
| **Колоночное хранение** | Данные хранятся по колонкам, а не по строкам — читаются только нужные колонки |
| **Сжатие** | Достигает степени сжатия 10-20x на рыночных данных |
| **Векторное выполнение** | Обрабатывает данные пакетами с использованием SIMD-инструкций |
| **Вставка в реальном времени** | Обрабатывает миллионы вставок в секунду |
| **Поддержка SQL** | Полный SQL с расширениями для анализа временных рядов |
| **Распределённость** | Масштабируется горизонтально на несколько узлов |

## Настройка ClickHouse с Rust

Добавь клиент ClickHouse в `Cargo.toml`:

```toml
[dependencies]
clickhouse = { version = "0.11", features = ["time", "uuid"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
time = { version = "0.3", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## Базовое подключение и создание таблицы

```rust
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct Trade {
    #[serde(with = "clickhouse::serde::time::datetime64::micros")]
    timestamp: OffsetDateTime,
    symbol: String,
    exchange: String,
    price: f64,
    quantity: f64,
    side: String,  // "buy" или "sell"
    trade_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключение к ClickHouse
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("trading");

    // Создание таблицы, оптимизированной для торговых временных рядов
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                timestamp DateTime64(6),
                symbol LowCardinality(String),
                exchange LowCardinality(String),
                price Float64,
                quantity Float64,
                side LowCardinality(String),
                trade_id String
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, timestamp)
            "#,
        )
        .execute()
        .await?;

    println!("Таблица trades успешно создана!");
    Ok(())
}
```

### Понимание дизайна таблицы

```
Типы колонок для торговых данных:
┌─────────────────────────────────────────────────────────────┐
│ DateTime64(6)     → Микросекундная точность временных меток │
│ LowCardinality    → Оптимизация для колонок с малым числом  │
│                     значений (символы, биржи, стороны)      │
│ Float64           → Стандартная точность для цен/объёмов    │
│ String            → Текст переменной длины (ID сделок)      │
├─────────────────────────────────────────────────────────────┤
│ ENGINE = MergeTree()                                        │
│ ├── PARTITION BY toYYYYMM(timestamp)                        │
│ │   └── Месячные партиции для эффективного управления       │
│ └── ORDER BY (symbol, timestamp)                            │
│     └── Оптимизирует запросы с фильтрацией по символу+время │
└─────────────────────────────────────────────────────────────┘
```

## Вставка торговых данных

### Одиночная вставка

```rust
use clickhouse::Client;
use time::OffsetDateTime;
use uuid::Uuid;

async fn insert_single_trade(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let trade = Trade {
        timestamp: OffsetDateTime::now_utc(),
        symbol: "BTC/USDT".to_string(),
        exchange: "Binance".to_string(),
        price: 42150.50,
        quantity: 0.5,
        side: "buy".to_string(),
        trade_id: Uuid::new_v4().to_string(),
    };

    let mut insert = client.insert("trades")?;
    insert.write(&trade).await?;
    insert.end().await?;

    println!("Сделка вставлена: {:?}", trade);
    Ok(())
}
```

### Пакетная вставка для высокой пропускной способности

```rust
use clickhouse::Client;
use time::OffsetDateTime;
use uuid::Uuid;
use std::time::Duration;

async fn insert_trades_batch(
    client: &Client,
    trades: Vec<Trade>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();

    let mut insert = client.insert("trades")?;

    for trade in &trades {
        insert.write(trade).await?;
    }

    insert.end().await?;

    let elapsed = start.elapsed();
    let rate = trades.len() as f64 / elapsed.as_secs_f64();

    println!(
        "Вставлено {} сделок за {:?} ({:.0} сделок/сек)",
        trades.len(),
        elapsed,
        rate
    );

    Ok(())
}

// Генерация примеров торговых данных
fn generate_sample_trades(count: usize) -> Vec<Trade> {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT", "XRP/USDT"];
    let exchanges = vec!["Binance", "Coinbase", "Kraken", "OKX"];
    let base_prices = vec![42000.0, 2200.0, 95.0, 0.62];

    (0..count)
        .map(|i| {
            let symbol_idx = i % symbols.len();
            let exchange_idx = (i / 2) % exchanges.len();
            let price_variance = (i as f64 * 0.001).sin() * 100.0;

            Trade {
                timestamp: OffsetDateTime::now_utc(),
                symbol: symbols[symbol_idx].to_string(),
                exchange: exchanges[exchange_idx].to_string(),
                price: base_prices[symbol_idx] + price_variance,
                quantity: 0.1 + (i as f64 % 10.0) * 0.1,
                side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                trade_id: Uuid::new_v4().to_string(),
            }
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("trading");

    // Вставка 100,000 примеров сделок
    let trades = generate_sample_trades(100_000);
    insert_trades_batch(&client, trades).await?;

    Ok(())
}
```

## Аналитические запросы для трейдинга

### Расчёт VWAP

```rust
use clickhouse::{Client, Row};
use serde::Deserialize;

#[derive(Debug, Row, Deserialize)]
struct VwapResult {
    symbol: String,
    vwap: f64,
    total_volume: f64,
    trade_count: u64,
}

async fn calculate_vwap(
    client: &Client,
    symbol: &str,
    hours: u32,
) -> Result<VwapResult, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            symbol,
            sum(price * quantity) / sum(quantity) AS vwap,
            sum(quantity) AS total_volume,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL ? HOUR
        GROUP BY symbol
    "#;

    let result = client
        .query(query)
        .bind(symbol)
        .bind(hours)
        .fetch_one::<VwapResult>()
        .await?;

    println!(
        "{} VWAP ({}ч): ${:.2} | Объём: {:.4} | Сделок: {}",
        result.symbol, hours, result.vwap, result.total_volume, result.trade_count
    );

    Ok(result)
}
```

### OHLCV (свечные) данные по временным интервалам

```rust
#[derive(Debug, Row, Deserialize)]
struct Candle {
    bucket: String,
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_count: u64,
}

async fn get_ohlcv_candles(
    client: &Client,
    symbol: &str,
    interval_minutes: u32,
    limit: u32,
) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            toStartOfInterval(timestamp, INTERVAL ? MINUTE) AS bucket,
            symbol,
            argMin(price, timestamp) AS open,
            max(price) AS high,
            min(price) AS low,
            argMax(price, timestamp) AS close,
            sum(quantity) AS volume,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
        GROUP BY bucket, symbol
        ORDER BY bucket DESC
        LIMIT ?
    "#;

    let candles = client
        .query(query)
        .bind(interval_minutes)
        .bind(symbol)
        .bind(limit)
        .fetch_all::<Candle>()
        .await?;

    for candle in &candles {
        println!(
            "{} | O:{:.2} H:{:.2} L:{:.2} C:{:.2} | Объём:{:.4}",
            candle.bucket, candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }

    Ok(candles)
}
```

### Сравнение объёмов по биржам

```rust
#[derive(Debug, Row, Deserialize)]
struct ExchangeVolume {
    exchange: String,
    symbol: String,
    total_volume: f64,
    buy_volume: f64,
    sell_volume: f64,
    avg_trade_size: f64,
    trade_count: u64,
}

async fn compare_exchange_volumes(
    client: &Client,
    symbol: &str,
) -> Result<Vec<ExchangeVolume>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            exchange,
            symbol,
            sum(quantity) AS total_volume,
            sumIf(quantity, side = 'buy') AS buy_volume,
            sumIf(quantity, side = 'sell') AS sell_volume,
            avg(quantity) AS avg_trade_size,
            count() AS trade_count
        FROM trades
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL 24 HOUR
        GROUP BY exchange, symbol
        ORDER BY total_volume DESC
    "#;

    let volumes = client
        .query(query)
        .bind(symbol)
        .fetch_all::<ExchangeVolume>()
        .await?;

    println!("\n=== Объём {} по биржам (24ч) ===", symbol);
    for vol in &volumes {
        let buy_ratio = vol.buy_volume / vol.total_volume * 100.0;
        println!(
            "{}: {:.4} всего | Покупки: {:.1}% | Сред. размер: {:.6} | {} сделок",
            vol.exchange, vol.total_volume, buy_ratio, vol.avg_trade_size, vol.trade_count
        );
    }

    Ok(volumes)
}
```

## Продвинутая аналитика: анализ спреда и проскальзывания

```rust
#[derive(Debug, Row, Deserialize)]
struct SpreadAnalysis {
    time_bucket: String,
    symbol: String,
    avg_spread_bps: f64,
    max_spread_bps: f64,
    min_spread_bps: f64,
    quote_count: u64,
}

// Таблица для снимков стакана заявок
async fn create_orderbook_table(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS orderbook_snapshots (
                timestamp DateTime64(6),
                symbol LowCardinality(String),
                exchange LowCardinality(String),
                best_bid Float64,
                best_ask Float64,
                bid_size Float64,
                ask_size Float64,
                mid_price Float64,
                spread_bps Float64
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, exchange, timestamp)
            "#,
        )
        .execute()
        .await?;

    Ok(())
}

async fn analyze_spreads(
    client: &Client,
    symbol: &str,
    interval_minutes: u32,
) -> Result<Vec<SpreadAnalysis>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            toStartOfInterval(timestamp, INTERVAL ? MINUTE) AS time_bucket,
            symbol,
            avg(spread_bps) AS avg_spread_bps,
            max(spread_bps) AS max_spread_bps,
            min(spread_bps) AS min_spread_bps,
            count() AS quote_count
        FROM orderbook_snapshots
        WHERE symbol = ?
          AND timestamp >= now() - INTERVAL 24 HOUR
        GROUP BY time_bucket, symbol
        ORDER BY time_bucket
    "#;

    let analysis = client
        .query(query)
        .bind(interval_minutes)
        .bind(symbol)
        .fetch_all::<SpreadAnalysis>()
        .await?;

    println!("\n=== Анализ спреда {} (интервал {}мин) ===", symbol, interval_minutes);
    for row in &analysis {
        println!(
            "{} | Сред: {:.2}bps | Диапазон: {:.2}-{:.2}bps | {} котировок",
            row.time_bucket, row.avg_spread_bps, row.min_spread_bps, row.max_spread_bps, row.quote_count
        );
    }

    Ok(analysis)
}
```

## Агрегация в реальном времени с материализованными представлениями

```rust
async fn create_materialized_views(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // Создание целевой таблицы для минутных агрегаций
    client
        .query(
            r#"
            CREATE TABLE IF NOT EXISTS trades_1m (
                timestamp DateTime,
                symbol LowCardinality(String),
                open Float64,
                high Float64,
                low Float64,
                close Float64,
                volume Float64,
                buy_volume Float64,
                sell_volume Float64,
                trade_count UInt64,
                vwap Float64
            )
            ENGINE = SummingMergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (symbol, timestamp)
            "#,
        )
        .execute()
        .await?;

    // Создание материализованного представления для авто-агрегации
    client
        .query(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS trades_1m_mv
            TO trades_1m
            AS SELECT
                toStartOfMinute(timestamp) AS timestamp,
                symbol,
                argMin(price, timestamp) AS open,
                max(price) AS high,
                min(price) AS low,
                argMax(price, timestamp) AS close,
                sum(quantity) AS volume,
                sumIf(quantity, side = 'buy') AS buy_volume,
                sumIf(quantity, side = 'sell') AS sell_volume,
                count() AS trade_count,
                sum(price * quantity) / sum(quantity) AS vwap
            FROM trades
            GROUP BY timestamp, symbol
            "#,
        )
        .execute()
        .await?;

    println!("Материализованное представление создано - сделки будут авто-агрегироваться в минутные свечи!");
    Ok(())
}
```

## Аналитика портфеля

```rust
#[derive(Debug, Clone, Row, Serialize, Deserialize)]
struct PortfolioTrade {
    #[serde(with = "clickhouse::serde::time::datetime64::micros")]
    timestamp: OffsetDateTime,
    portfolio_id: String,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    fees: f64,
    pnl: f64,
}

#[derive(Debug, Row, Deserialize)]
struct PortfolioPerformance {
    portfolio_id: String,
    total_trades: u64,
    winning_trades: u64,
    win_rate: f64,
    total_pnl: f64,
    avg_pnl_per_trade: f64,
    max_drawdown: f64,
    sharpe_approx: f64,
}

async fn analyze_portfolio_performance(
    client: &Client,
    portfolio_id: &str,
) -> Result<PortfolioPerformance, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT
            portfolio_id,
            count() AS total_trades,
            countIf(pnl > 0) AS winning_trades,
            countIf(pnl > 0) / count() * 100 AS win_rate,
            sum(pnl) AS total_pnl,
            avg(pnl) AS avg_pnl_per_trade,
            min(runningAccumulate(sumState(pnl))) AS max_drawdown,
            avg(pnl) / stddevPop(pnl) AS sharpe_approx
        FROM portfolio_trades
        WHERE portfolio_id = ?
        GROUP BY portfolio_id
    "#;

    let perf = client
        .query(query)
        .bind(portfolio_id)
        .fetch_one::<PortfolioPerformance>()
        .await?;

    println!("\n=== Производительность портфеля {} ===", portfolio_id);
    println!("Всего сделок: {}", perf.total_trades);
    println!("Процент выигрышных: {:.1}%", perf.win_rate);
    println!("Общий P&L: ${:.2}", perf.total_pnl);
    println!("Средний P&L/сделка: ${:.2}", perf.avg_pnl_per_trade);
    println!("Коэффициент Шарпа (прибл.): {:.2}", perf.sharpe_approx);

    Ok(perf)
}
```

## Потоковые вставки с асинхронными каналами

```rust
use tokio::sync::mpsc;
use std::time::Duration;

struct TradingDataPipeline {
    client: Client,
    buffer_size: usize,
}

impl TradingDataPipeline {
    fn new(client: Client, buffer_size: usize) -> Self {
        Self { client, buffer_size }
    }

    async fn start_ingestion(
        &self,
        mut receiver: mpsc::Receiver<Trade>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer: Vec<Trade> = Vec::with_capacity(self.buffer_size);
        let mut last_flush = std::time::Instant::now();
        let flush_interval = Duration::from_secs(1);

        loop {
            tokio::select! {
                Some(trade) = receiver.recv() => {
                    buffer.push(trade);

                    // Сброс при заполнении буфера
                    if buffer.len() >= self.buffer_size {
                        self.flush_buffer(&mut buffer).await?;
                        last_flush = std::time::Instant::now();
                    }
                }
                _ = tokio::time::sleep(flush_interval) => {
                    // Периодический сброс даже если буфер не полон
                    if !buffer.is_empty() && last_flush.elapsed() >= flush_interval {
                        self.flush_buffer(&mut buffer).await?;
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        }
    }

    async fn flush_buffer(
        &self,
        buffer: &mut Vec<Trade>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut insert = self.client.insert("trades")?;
        for trade in buffer.iter() {
            insert.write(trade).await?;
        }
        insert.end().await?;

        println!("Сброшено {} сделок в ClickHouse", buffer.len());
        buffer.clear();

        Ok(())
    }
}

// Пример использования с симулированными рыночными данными
async fn simulate_market_feed(sender: mpsc::Sender<Trade>) {
    let symbols = vec!["BTC/USDT", "ETH/USDT"];
    let mut counter = 0u64;

    loop {
        for symbol in &symbols {
            let trade = Trade {
                timestamp: OffsetDateTime::now_utc(),
                symbol: symbol.to_string(),
                exchange: "Binance".to_string(),
                price: 42000.0 + (counter as f64 * 0.1).sin() * 100.0,
                quantity: 0.1 + (counter % 10) as f64 * 0.05,
                side: if counter % 2 == 0 { "buy" } else { "sell" }.to_string(),
                trade_id: format!("T{}", counter),
            };

            if sender.send(trade).await.is_err() {
                return;
            }
            counter += 1;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Колоночное хранение | ClickHouse хранит данные по колонкам для быстрых аналитических запросов |
| Движок MergeTree | Основной движок таблиц с сортировкой, партиционированием и слиянием |
| LowCardinality | Оптимизация для колонок с малым числом уникальных значений |
| DateTime64 | Микросекундная точность временных меток для тиковых данных |
| Материализованные представления | Авто-агрегация данных при вставке для аналитики в реальном времени |
| Пакетные вставки | Буферизация и пакетная запись для высокой пропускной способности |
| VWAP/OHLCV | Стандартные торговые метрики, вычисляемые эффективно |

## Упражнения

1. **Детектор межбиржевого арбитража**: Напиши запрос, который находит расхождения цен более 0.1% между биржами для одного символа в окнах по 1 секунде.

2. **Анализ профиля объёма**: Создай запрос, который рассчитывает распределение объёма по ценовым уровням (Price Volume Profile) для заданного символа за последние 24 часа.

3. **Дисбаланс потока сделок**: Реализуй потоковый расчёт, который отслеживает дисбаланс объёма покупок/продаж в реальном времени и сохраняет оповещения при превышении порога.

4. **Атрибуция производительности**: Спроектируй таблицы и запросы для отслеживания атрибуции P&L по символам, стратегиям и временным периодам для мультистратегийной торговой системы.

## Домашнее задание

1. **Хранилище тиковых данных**: Спроектируй полную схему для хранения:
   - Тиков сделок
   - Снимков стакана заявок (топ-10 уровней)
   - Ставок финансирования
   - Событий ликвидации

   Включи соответствующее партиционирование, ключи сортировки и материализованные представления для частых запросов.

2. **Пайплайн данных для бэктестинга**: Построй сервис на Rust, который:
   - Читает исторические данные сделок из CSV-файлов
   - Трансформирует и валидирует данные
   - Массово загружает в ClickHouse с отображением прогресса
   - Обрабатывает обнаружение дубликатов и проверку качества данных

3. **Бэкенд для дашборда реального времени**: Создай асинхронный сервис на Rust, который:
   - Предоставляет REST-эндпоинты для типичной торговой аналитики
   - Реализует кэширование часто запрашиваемых данных
   - Поддерживает WebSocket-стриминг живых агрегаций
   - Эффективно обрабатывает параллельные запросы

4. **Инструмент кросс-биржевого анализа**: Реализуй CLI-инструмент, который:
   - Сравнивает качество исполнения на разных биржах
   - Рассчитывает эффективные спреды и влияние на рынок
   - Генерирует отчёты о лучших площадках для исполнения
   - Поддерживает различные временные гранулярности

## Навигация

[← Предыдущий день](../238-timescaledb-time-series/ru.md) | [Следующий день →](../240-questdb-market-data/ru.md)
