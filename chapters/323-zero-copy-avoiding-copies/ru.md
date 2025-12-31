# День 323: Zero-copy: избегаем копирования

## Аналогия из трейдинга

Представь, что ты управляешь высокочастотным торговым столом, где каждая миллисекунда на счету. Твой поток рыночных данных доставляет тысячи обновлений цен в секунду, и каждое обновление должно быть обработано немедленно.

**Традиционный подход (с копированием):**
- Получение обновления цены — это как курьер, приносящий тебе документ
- Ты делаешь ксерокопию для аналитика, ещё одну для риск-менеджера, ещё одну для системы исполнения ордеров
- Каждая копия занимает время и бумагу (память)
- К тому времени, когда у всех есть копия, цена могла уже измениться

**Подход Zero-copy:**
- Вместо ксерокопирования ты выставляешь документ на центральный дисплей
- Все читают из одного источника одновременно
- Никакой задержки на копирование, никакой траты бумаги
- Все видят данные в одно и то же время

В торговых системах техники zero-copy могут означать разницу между:
- **Захватом прибыльного арбитража** и упущением его на микросекунды
- **Обработкой 10,000 ордеров/секунду** против только 1,000
- **Работой на маленьком VPS** против необходимости в дорогом железе

## Что такое Zero-Copy?

**Zero-copy** — это техника, которая минимизирует или исключает копирование данных во время операций. Вместо копирования данных из одного места памяти в другое ты работаешь со ссылками или представлениями (views) на оригинальные данные.

### Стоимость копирования

```rust
use std::time::Instant;

fn demonstrate_copy_cost() {
    let iterations = 1_000_000;

    // Большой массив исторических цен
    let price_data: Vec<f64> = (0..1000).map(|i| 50000.0 + i as f64 * 0.1).collect();

    // С КОПИРОВАНИЕМ: Каждая итерация копирует весь вектор
    let start = Instant::now();
    for _ in 0..iterations {
        let copied_data = price_data.clone();  // Полная копия!
        let _sum: f64 = copied_data.iter().sum();
    }
    let copy_time = start.elapsed();

    // БЕЗ КОПИРОВАНИЯ: Используем ссылку на оригинальные данные
    let start = Instant::now();
    for _ in 0..iterations {
        let _sum: f64 = price_data.iter().sum();  // Без копии, только ссылка
    }
    let zerocopy_time = start.elapsed();

    println!("=== Анализ стоимости копирования ===");
    println!("С копированием:    {:?}", copy_time);
    println!("Без копирования:   {:?}", zerocopy_time);
    println!("Ускорение:         {:.1}x", copy_time.as_nanos() as f64 / zerocopy_time.as_nanos() as f64);
}

fn main() {
    demonstrate_copy_cost();
}
```

**Ожидаемый вывод:**
```
=== Анализ стоимости копирования ===
С копированием:    1.2s
Без копирования:   45ms
Ускорение:         26.7x
```

## Техники Zero-Copy в Rust

### Техника 1: Срезы вместо владеющих коллекций

Срезы (`&[T]`) — это представления последовательных данных без владения ими.

```rust
/// OHLCV свечные данные
#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Вычисление Simple Moving Average с использованием среза (zero-copy)
fn calculate_sma(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }

    // Берём последние `period` свечей как срез — без копирования!
    let window = &candles[candles.len() - period..];
    let sum: f64 = window.iter().map(|c| c.close).sum();
    Some(sum / period as f64)
}

/// ПЛОХО: Забирает владение, заставляет вызывающего клонировать
fn calculate_sma_bad(candles: Vec<Candle>, period: usize) -> Option<f64> {
    if candles.len() < period {
        return None;
    }
    let sum: f64 = candles[candles.len() - period..].iter().map(|c| c.close).sum();
    Some(sum / period as f64)
}

fn main() {
    let candles: Vec<Candle> = (0..100)
        .map(|i| Candle {
            timestamp: 1700000000 + i * 60,
            open: 50000.0 + i as f64,
            high: 50100.0 + i as f64,
            low: 49900.0 + i as f64,
            close: 50050.0 + i as f64,
            volume: 100.0,
        })
        .collect();

    // ХОРОШО: Zero-copy, можно вызывать много раз
    let sma_20 = calculate_sma(&candles, 20);
    let sma_50 = calculate_sma(&candles, 50);  // Те же данные, без копии

    println!("SMA(20): ${:.2}", sma_20.unwrap_or(0.0));
    println!("SMA(50): ${:.2}", sma_50.unwrap_or(0.0));

    // ПЛОХО: Потребовалось бы клонирование для каждого вызова
    // let sma_bad = calculate_sma_bad(candles.clone(), 20);  // Нужен clone!
}
```

### Техника 2: Заимствующие итераторы

Использование итераторов, которые заимствуют данные вместо их потребления.

```rust
struct OrderBook {
    bids: Vec<(f64, f64)>,  // (цена, количество)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, quantity: f64) {
        self.bids.push((price, quantity));
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); // По убыванию
    }

    fn add_ask(&mut self, price: f64, quantity: f64) {
        self.asks.push((price, quantity));
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap()); // По возрастанию
    }

    /// Zero-copy: возвращает итератор по ссылкам
    fn top_bids(&self, n: usize) -> impl Iterator<Item = &(f64, f64)> {
        self.bids.iter().take(n)
    }

    /// Zero-copy: вычисляет общий объём бидов без копирования
    fn total_bid_volume(&self) -> f64 {
        self.bids.iter().map(|(_, qty)| qty).sum()
    }

    /// Zero-copy: находит лучший спред bid/ask
    fn spread(&self) -> Option<f64> {
        let best_bid = self.bids.first().map(|(p, _)| p)?;
        let best_ask = self.asks.first().map(|(p, _)| p)?;
        Some(best_ask - best_bid)
    }

    /// Zero-copy: вычисляет дисбаланс стакана
    fn imbalance(&self, depth: usize) -> f64 {
        let bid_volume: f64 = self.bids.iter().take(depth).map(|(_, q)| q).sum();
        let ask_volume: f64 = self.asks.iter().take(depth).map(|(_, q)| q).sum();

        if bid_volume + ask_volume > 0.0 {
            (bid_volume - ask_volume) / (bid_volume + ask_volume)
        } else {
            0.0
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Добавляем ордера
    book.add_bid(49990.0, 1.5);
    book.add_bid(49980.0, 2.0);
    book.add_bid(49970.0, 3.0);
    book.add_ask(50010.0, 1.0);
    book.add_ask(50020.0, 2.5);

    // Все эти операции — zero-copy
    println!("=== Анализ стакана ===");
    println!("Спред: ${:.2}", book.spread().unwrap_or(0.0));
    println!("Общий объём бидов: {:.2} BTC", book.total_bid_volume());
    println!("Дисбаланс (глубина 3): {:.2}", book.imbalance(3));

    println!("\nТоп-2 бида:");
    for (price, qty) in book.top_bids(2) {
        println!("  ${:.2} x {:.2}", price, qty);
    }
}
```

### Техника 3: Cow (Clone-on-Write)

`Cow` (Clone on Write) откладывает копирование до момента, когда необходима мутация.

```rust
use std::borrow::Cow;

/// Данные сделки, которые могут потребовать модификации
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

/// Обработка данных сделки — копирует только при необходимости модификации
fn normalize_symbol<'a>(symbol: &'a str) -> Cow<'a, str> {
    if symbol.chars().all(|c| c.is_uppercase()) {
        // Уже нормализован — возвращаем ссылку (без копии)
        Cow::Borrowed(symbol)
    } else {
        // Нужна нормализация — выделяем новую строку
        Cow::Owned(symbol.to_uppercase())
    }
}

/// Валидация и возможная модификация цены
fn validate_price<'a>(price: f64, min_tick: f64) -> f64 {
    // Округление до ближайшего размера тика
    (price / min_tick).round() * min_tick
}

/// Обработка сделок с zero-copy где возможно
fn process_trades<'a>(trades: &'a [Trade]) -> Vec<Cow<'a, Trade>> {
    trades
        .iter()
        .map(|trade| {
            let normalized_symbol = normalize_symbol(&trade.symbol);

            // Клонируем только если нужно модифицировать
            if matches!(normalized_symbol, Cow::Owned(_)) {
                Cow::Owned(Trade {
                    symbol: normalized_symbol.into_owned(),
                    price: trade.price,
                    quantity: trade.quantity,
                    side: trade.side.clone(),
                })
            } else {
                Cow::Borrowed(trade)
            }
        })
        .collect()
}

fn main() {
    let trades = vec![
        Trade {
            symbol: "BTCUSDT".to_string(),  // Уже в верхнем регистре
            price: 50000.0,
            quantity: 1.0,
            side: "buy".to_string(),
        },
        Trade {
            symbol: "ethusdt".to_string(),  // Нужна нормализация
            price: 3000.0,
            quantity: 2.0,
            side: "sell".to_string(),
        },
        Trade {
            symbol: "SOLUSDT".to_string(),  // Уже в верхнем регистре
            price: 100.0,
            quantity: 10.0,
            side: "buy".to_string(),
        },
    ];

    let processed = process_trades(&trades);

    println!("=== Обработанные сделки ===");
    for (i, trade) in processed.iter().enumerate() {
        let copy_status = if matches!(trade, Cow::Borrowed(_)) {
            "zero-copy"
        } else {
            "скопировано"
        };
        println!("Сделка {}: {} ({}) - {}", i + 1, trade.symbol, trade.price, copy_status);
    }
}
```

### Техника 4: Memory Mapping для исторических данных

Для больших файлов исторических данных memory mapping избегает загрузки всего в RAM.

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Симулированный memory-mapped читатель ценовых данных
/// В продакшене ты бы использовал crate `memmap2`
struct PriceDataReader {
    prices: Vec<f64>,  // В реальной реализации это было бы memory-mapped
}

impl PriceDataReader {
    /// Загрузка ценовых данных (симулировано — реальная имплементация использовала бы mmap)
    fn new(data: Vec<f64>) -> Self {
        PriceDataReader { prices: data }
    }

    /// Zero-copy доступ к окну цен
    fn get_window(&self, start: usize, end: usize) -> Option<&[f64]> {
        if end <= self.prices.len() && start < end {
            Some(&self.prices[start..end])
        } else {
            None
        }
    }

    /// Вычисление индикатора по окну без копирования
    fn calculate_volatility(&self, window_start: usize, window_size: usize) -> Option<f64> {
        let window = self.get_window(window_start, window_start + window_size)?;

        // Вычисляем стандартное отклонение (волатильность)
        let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
        let variance: f64 = window.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / window.len() as f64;

        Some(variance.sqrt())
    }

    /// Анализ скользящего окна без копирования данных
    fn analyze_volatility_series(&self, window_size: usize) -> Vec<f64> {
        (0..self.prices.len().saturating_sub(window_size))
            .filter_map(|i| self.calculate_volatility(i, window_size))
            .collect()
    }
}

fn main() {
    // Симулируем загрузку 1 миллиона исторических цен
    let prices: Vec<f64> = (0..1_000_000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    let reader = PriceDataReader::new(prices);

    // Zero-copy доступ к окну
    if let Some(recent) = reader.get_window(999_990, 1_000_000) {
        println!("Последние 10 цен (zero-copy доступ):");
        for (i, price) in recent.iter().enumerate() {
            println!("  {}: ${:.2}", i + 1, price);
        }
    }

    // Вычисляем волатильность по небольшой выборке
    let start = std::time::Instant::now();
    let volatilities = reader.analyze_volatility_series(100);
    let elapsed = start.elapsed();

    println!("\nВычислено {} точек волатильности за {:?}", volatilities.len(), elapsed);
    println!("Средняя волатильность: ${:.2}",
        volatilities.iter().sum::<f64>() / volatilities.len() as f64);
}
```

### Техника 5: Байты и повторное использование буферов

При парсинге потоков рыночных данных переиспользуй буферы вместо выделения новых.

```rust
/// Переиспользуемый буфер для парсинга рыночных данных
struct MarketDataParser {
    buffer: Vec<u8>,
    parsed_prices: Vec<f64>,
}

impl MarketDataParser {
    fn new(capacity: usize) -> Self {
        MarketDataParser {
            buffer: Vec::with_capacity(capacity),
            parsed_prices: Vec::with_capacity(1000),
        }
    }

    /// Парсинг ценовых данных с переиспользованием внутренних буферов
    fn parse_prices(&mut self, data: &[u8]) -> &[f64] {
        self.parsed_prices.clear();  // Переиспользуем буфер, не реаллоцируем

        // Симуляция парсинга (в реальности ты бы парсил актуальный формат рыночных данных)
        for chunk in data.chunks(8) {
            if chunk.len() == 8 {
                let price = f64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
                self.parsed_prices.push(price);
            }
        }

        &self.parsed_prices  // Возвращаем ссылку на внутренний буфер
    }

    /// Обработка потоковых данных с нулём аллокаций в hot path
    fn process_stream(&mut self, messages: &[Vec<u8>]) -> f64 {
        let mut total_volume = 0.0;

        for message in messages {
            let prices = self.parse_prices(message);
            total_volume += prices.iter().sum::<f64>();
        }

        total_volume
    }
}

fn main() {
    let mut parser = MarketDataParser::new(1024);

    // Симулируем входящие сообщения рыночных данных
    let messages: Vec<Vec<u8>> = (0..100)
        .map(|i| {
            let price = 50000.0 + i as f64;
            price.to_le_bytes().to_vec()
        })
        .collect();

    let start = std::time::Instant::now();
    let iterations = 10_000;

    for _ in 0..iterations {
        let _total = parser.process_stream(&messages);
    }

    println!("Обработано {} итераций за {:?}", iterations, start.elapsed());
    println!("Повторное использование буфера исключает {} аллокаций", iterations * messages.len());
}
```

## Пример реальной торговой системы

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Тик рыночных данных
#[derive(Debug, Clone)]
struct Tick {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: i64,
}

/// Позиция портфеля
struct Position {
    quantity: f64,
    avg_price: f64,
}

/// Высокопроизводительный торговый движок с использованием zero-copy техник
struct TradingEngine {
    positions: HashMap<String, Position>,
    price_cache: HashMap<String, (f64, f64)>,  // (bid, ask)
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine {
            positions: HashMap::new(),
            price_cache: HashMap::new(),
        }
    }

    /// Обновление кеша цен — использует &str чтобы избежать аллокации
    fn update_price(&mut self, symbol: &str, bid: f64, ask: f64) {
        // Аллоцирует String только если символ новый
        if let Some(cache) = self.price_cache.get_mut(symbol) {
            *cache = (bid, ask);  // Обновление на месте
        } else {
            self.price_cache.insert(symbol.to_string(), (bid, ask));
        }
    }

    /// Получение текущей цены — zero-copy поиск
    fn get_price(&self, symbol: &str) -> Option<(f64, f64)> {
        self.price_cache.get(symbol).copied()
    }

    /// Вычисление стоимости портфеля — zero-copy итерация
    fn calculate_portfolio_value(&self) -> f64 {
        self.positions.iter()
            .map(|(symbol, pos)| {
                let mid_price = self.price_cache
                    .get(symbol)
                    .map(|(bid, ask)| (bid + ask) / 2.0)
                    .unwrap_or(pos.avg_price);
                pos.quantity * mid_price
            })
            .sum()
    }

    /// Вычисление нереализованного PnL — zero-copy
    fn calculate_unrealized_pnl(&self) -> f64 {
        self.positions.iter()
            .map(|(symbol, pos)| {
                let mid_price = self.price_cache
                    .get(symbol)
                    .map(|(bid, ask)| (bid + ask) / 2.0)
                    .unwrap_or(pos.avg_price);
                pos.quantity * (mid_price - pos.avg_price)
            })
            .sum()
    }

    /// Поиск торговых возможностей — возвращает ссылки
    fn find_arbitrage_opportunities(&self, threshold: f64) -> Vec<(&str, f64)> {
        self.price_cache
            .iter()
            .filter_map(|(symbol, (bid, ask))| {
                let spread = (ask - bid) / bid;
                if spread > threshold {
                    Some((symbol.as_str(), spread))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Обработка пакета тиков — минимальное копирование
    fn process_ticks(&mut self, ticks: &[Tick]) -> usize {
        let mut updates = 0;
        for tick in ticks {
            self.update_price(&tick.symbol, tick.bid, tick.ask);
            updates += 1;
        }
        updates
    }

    /// Добавление или обновление позиции
    fn update_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        self.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                let total_cost = pos.avg_price * pos.quantity + price * quantity;
                pos.quantity += quantity;
                if pos.quantity != 0.0 {
                    pos.avg_price = total_cost / pos.quantity;
                }
            })
            .or_insert(Position {
                quantity,
                avg_price: price,
            });
    }
}

fn main() {
    let mut engine = TradingEngine::new();

    // Настройка позиций
    engine.update_position("BTCUSDT", 0.5, 50000.0);
    engine.update_position("ETHUSDT", 5.0, 3000.0);
    engine.update_position("SOLUSDT", 100.0, 100.0);

    // Симуляция потока рыночных данных
    let symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT", "DOTUSDT"];
    let ticks: Vec<Tick> = symbols
        .iter()
        .enumerate()
        .map(|(i, &symbol)| Tick {
            symbol: symbol.to_string(),
            bid: 50000.0 / (i + 1) as f64,
            ask: 50010.0 / (i + 1) as f64,
            timestamp: 1700000000 + i as i64,
        })
        .collect();

    // Бенчмарк zero-copy обработки
    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        engine.process_ticks(&ticks);
        let _value = engine.calculate_portfolio_value();
        let _pnl = engine.calculate_unrealized_pnl();
    }

    let elapsed = start.elapsed();

    println!("=== Zero-Copy торговый движок ===");
    println!("Обработано {} итераций за {:?}", iterations, elapsed);
    println!("Пропускная способность: {:.0} итераций/сек", iterations as f64 / elapsed.as_secs_f64());
    println!("\nСтоимость портфеля: ${:.2}", engine.calculate_portfolio_value());
    println!("Нереализованный PnL: ${:+.2}", engine.calculate_unrealized_pnl());

    // Поиск возможностей
    if let opportunities = engine.find_arbitrage_opportunities(0.0001) {
        println!("\nАрбитражные возможности (спред > 0.01%):");
        for (symbol, spread) in opportunities.iter().take(3) {
            println!("  {}: {:.4}%", symbol, spread * 100.0);
        }
    }
}
```

## Когда использовать Zero-Copy

### Используй Zero-Copy когда:

| Сценарий | Почему |
|----------|--------|
| **Hot paths** | Код, выполняющийся миллионы раз в секунду |
| **Большие данные** | Обработка гигабайтов исторических данных |
| **Потоковые данные** | Рыночные потоки в реальном времени |
| **Ограниченная память** | VPS или встроенные системы |
| **Критичная латентность** | HFT, арбитраж |

### Когда копирование допустимо:

| Сценарий | Почему |
|----------|--------|
| **Нужна модификация данных** | Клонируй перед мутацией |
| **Межпоточное разделение** | Может потребоваться владеющие данные |
| **Сложность лайфтаймов** | Иногда проще клонировать |
| **Маленькие данные** | Стоимость незначительна |
| **Редкие операции** | Не является узким местом |

## Сравнение производительности

```rust
use std::time::Instant;

fn benchmark_approaches() {
    let data: Vec<f64> = (0..10_000).map(|i| i as f64).collect();
    let iterations = 100_000;

    // Подход 1: Клонировать весь вектор каждый раз
    let start = Instant::now();
    for _ in 0..iterations {
        let cloned = data.clone();
        let _sum: f64 = cloned.iter().sum();
    }
    let clone_time = start.elapsed();

    // Подход 2: Использовать ссылку (zero-copy)
    let start = Instant::now();
    for _ in 0..iterations {
        let _sum: f64 = data.iter().sum();
    }
    let ref_time = start.elapsed();

    // Подход 3: Использовать срез
    let start = Instant::now();
    for _ in 0..iterations {
        let slice = &data[..];
        let _sum: f64 = slice.iter().sum();
    }
    let slice_time = start.elapsed();

    println!("=== Сравнение производительности ===");
    println!("Клонировать каждый раз: {:?}", clone_time);
    println!("Использовать ссылку:    {:?}", ref_time);
    println!("Использовать срез:      {:?}", slice_time);
    println!("\nУскорение (clone vs ref): {:.1}x",
        clone_time.as_nanos() as f64 / ref_time.as_nanos() as f64);
}

fn main() {
    benchmark_approaches();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Zero-copy** | Работа со ссылками вместо копирования данных |
| **Срезы (Slices)** | Представления данных без владения |
| **Cow** | Clone-on-write для условного копирования |
| **Переиспользование буферов** | Повторное использование выделенной памяти для нескольких операций |
| **Memory mapping** | Доступ к файлам без загрузки в RAM |
| **Заимствующие итераторы** | Итерация без потребления данных |

## Практические задания

1. **Оптимизация стакана ордеров**: Реализуй стакан ордеров, который:
   - Использует срезы для доступа к уровням цен
   - Переиспользует внутренние буферы для агрегации
   - Измеряет использование памяти vs. подход с копированием

2. **Парсер потока цен**: Создай парсер рыночных данных, который:
   - Переиспользует один буфер для всех сообщений
   - Использует срезы для ссылок на распарсенные данные
   - Бенчмаркает vs. аллокация на каждое сообщение

3. **Калькулятор портфеля**: Построй анализатор портфеля, который:
   - Хранит позиции со ссылками где возможно
   - Использует Cow для нормализации символов
   - Вычисляет метрики без копирования данных позиций

4. **Движок бэктестинга**: Реализуй бэктестер, который:
   - Использует memory-mapped файлы для исторических данных
   - Обрабатывает свечи через срезы
   - Измеряет улучшение пропускной способности vs. загрузка всех данных

## Домашнее задание

1. **Zero-Copy поток рыночных данных**: Построй систему, которая:
   - Получает симулированные WebSocket сообщения
   - Парсит без аллокации на каждое сообщение
   - Обновляет стакан ордеров используя ссылки
   - Вычисляет VWAP используя срезы
   - Бенчмаркает: измерь аллокации в секунду

2. **Анализатор исторических данных**: Создай инструмент, который:
   - Читает многогигабайтные CSV файлы
   - Использует memory mapping или стриминг
   - Вычисляет скользящие индикаторы со срезами
   - Генерирует отчёты без загрузки всех данных
   - Сравнивает использование памяти: полная загрузка vs. стриминг

3. **Генератор сигналов с низкой латентностью**: Реализуй систему, которая:
   - Обрабатывает 100,000+ тиков в секунду
   - Использует пулы буферов для парсинга сообщений
   - Генерирует сигналы используя zero-copy паттерны
   - Измеряет и отчитывается по перцентилям латентности (p50, p99, p999)

4. **Память-эффективный бэктестер**: Построй фреймворк бэктестинга, который:
   - Обрабатывает 10+ лет тиковых данных
   - Использует итераторы вместо загрузки всего
   - Реализует стратегии с заимствованными данными
   - Отчитывается по максимальному использованию памяти
   - Сравнивает время выполнения с разными подходами

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../324-*/ru.md)
