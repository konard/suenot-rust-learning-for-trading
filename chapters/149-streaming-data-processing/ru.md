# День 149: Потоковая обработка данных

## Аналогия из трейдинга

Представь биржевой терминал, через который идёт поток котировок — тысячи обновлений цен в секунду. Ты не можешь загрузить все данные в память и только потом начать анализ — система захлебнётся. Вместо этого ты обрабатываешь данные **по мере их поступления**: получил котировку → проанализировал → принял решение → перешёл к следующей. Это и есть **потоковая обработка данных** (streaming).

## Зачем нужна потоковая обработка?

В трейдинге мы часто работаем с:
- **Бесконечными потоками** — реальные котировки никогда не заканчиваются
- **Большими файлами** — история торгов за годы может весить гигабайты
- **Ограниченной памятью** — нельзя загрузить 100 ГБ данных в 16 ГБ RAM

Потоковая обработка позволяет работать с данными **по частям**, используя минимум памяти.

## Итераторы — основа потоковой обработки

В Rust итераторы — это "ленивые" потоки данных. Они вычисляют следующий элемент только когда он нужен.

```rust
fn main() {
    // Симуляция потока котировок
    let price_stream = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // Ленивая обработка — ничего не вычисляется до .collect() или .for_each()
    let signals: Vec<&str> = price_stream
        .windows(2)  // Скользящее окно из 2 элементов
        .map(|window| {
            if window[1] > window[0] {
                "BUY"
            } else {
                "SELL"
            }
        })
        .collect();

    println!("Trading signals: {:?}", signals);
}
```

## Чтение файла построчно

Не загружаем весь файл — читаем строку за строкой:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    // Открываем файл с историей торгов
    let file = File::open("trades.csv")?;
    let reader = BufReader::new(file);

    let mut total_volume = 0.0;
    let mut trade_count = 0u64;

    // Читаем построчно — в памяти только одна строка!
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
                trade_count += 1;
            }
        }
    }

    println!("Total trades: {}", trade_count);
    println!("Total volume: {:.2}", total_volume);
    println!("Average volume: {:.2}", total_volume / trade_count as f64);

    Ok(())
}
```

## Потоковый расчёт скользящей средней

Рассчитываем SMA, не храня все данные в памяти:

```rust
use std::collections::VecDeque;

/// Скользящее среднее с ограниченным буфером
struct StreamingSMA {
    window: VecDeque<f64>,
    period: usize,
    sum: f64,
}

impl StreamingSMA {
    fn new(period: usize) -> Self {
        StreamingSMA {
            window: VecDeque::with_capacity(period),
            period,
            sum: 0.0,
        }
    }

    /// Добавляет новую цену и возвращает SMA (если достаточно данных)
    fn update(&mut self, price: f64) -> Option<f64> {
        // Добавляем новую цену
        self.window.push_back(price);
        self.sum += price;

        // Если окно переполнено — удаляем старую цену
        if self.window.len() > self.period {
            if let Some(old) = self.window.pop_front() {
                self.sum -= old;
            }
        }

        // Возвращаем SMA только если достаточно данных
        if self.window.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut sma = StreamingSMA::new(3);

    let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0, 42300.0];

    for price in prices {
        match sma.update(price) {
            Some(avg) => println!("Price: {:.0} -> SMA-3: {:.2}", price, avg),
            None => println!("Price: {:.0} -> SMA-3: Накапливаем данные...", price),
        }
    }
}
```

## Потоковый расчёт VWAP

VWAP (Volume Weighted Average Price) — ключевой индикатор для институциональных трейдеров:

```rust
/// Потоковый расчёт VWAP
struct StreamingVWAP {
    cumulative_volume: f64,
    cumulative_pv: f64,  // Price * Volume
}

impl StreamingVWAP {
    fn new() -> Self {
        StreamingVWAP {
            cumulative_volume: 0.0,
            cumulative_pv: 0.0,
        }
    }

    /// Добавляет сделку и возвращает текущий VWAP
    fn update(&mut self, price: f64, volume: f64) -> f64 {
        self.cumulative_volume += volume;
        self.cumulative_pv += price * volume;

        if self.cumulative_volume > 0.0 {
            self.cumulative_pv / self.cumulative_volume
        } else {
            0.0
        }
    }

    /// Сбрасывает VWAP (обычно в начале торгового дня)
    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_pv = 0.0;
    }
}

fn main() {
    let mut vwap = StreamingVWAP::new();

    // Поток сделок: (цена, объём)
    let trades = [
        (42000.0, 1.5),
        (42050.0, 2.0),
        (42100.0, 0.8),
        (42080.0, 1.2),
        (42150.0, 3.0),
    ];

    println!("╔════════════════════════════════════════════╗");
    println!("║          STREAMING VWAP CALCULATION        ║");
    println!("╠════════════════════════════════════════════╣");

    for (price, volume) in trades {
        let current_vwap = vwap.update(price, volume);
        println!("║ Trade: {:.0} x {:.1} BTC -> VWAP: {:.2} ║", price, volume, current_vwap);
    }

    println!("╚════════════════════════════════════════════╝");
}
```

## Каналы для обработки данных между потоками

Используем `std::sync::mpsc` для передачи данных между потоками:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Создаём канал: tx — отправитель, rx — получатель
    let (tx, rx) = mpsc::channel();

    // Поток-производитель: генерирует котировки
    let producer = thread::spawn(move || {
        let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

        for price in prices {
            println!("[Producer] Sending price: {}", price);
            tx.send(price).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        println!("[Producer] Done sending");
    });

    // Поток-потребитель: обрабатывает котировки
    let consumer = thread::spawn(move || {
        let mut sum = 0.0;
        let mut count = 0;

        // Получаем данные пока канал открыт
        while let Ok(price) = rx.recv() {
            sum += price;
            count += 1;
            println!("[Consumer] Received: {}, Running avg: {:.2}",
                     price, sum / count as f64);
        }

        println!("[Consumer] Final average: {:.2}", sum / count as f64);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Потоковый расчёт статистики торгов

```rust
/// Потоковая статистика с использованием алгоритма Уэлфорда
struct StreamingStats {
    count: u64,
    mean: f64,
    m2: f64,  // Сумма квадратов отклонений
    min: f64,
    max: f64,
}

impl StreamingStats {
    fn new() -> Self {
        StreamingStats {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    /// Добавляет новое значение (алгоритм Уэлфорда для численной стабильности)
    fn update(&mut self, value: f64) {
        self.count += 1;

        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;

        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
    }

    fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║       STREAMING TRADE STATISTICS      ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Count:    {:>25} ║", self.count);
        println!("║ Mean:     {:>25.2} ║", self.mean);
        println!("║ Std Dev:  {:>25.2} ║", self.std_dev());
        println!("║ Min:      {:>25.2} ║", self.min);
        println!("║ Max:      {:>25.2} ║", self.max);
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut stats = StreamingStats::new();

    // Поток PnL по сделкам
    let pnl_stream = [150.0, -50.0, 200.0, -30.0, 180.0, -80.0, 220.0, 100.0];

    for pnl in pnl_stream {
        stats.update(pnl);
        println!("PnL: {:>7.2} | Running mean: {:>7.2} | Std: {:>7.2}",
                 pnl, stats.mean, stats.std_dev());
    }

    println!();
    stats.print_summary();
}
```

## Потоковое обнаружение аномалий

Детектируем необычные ценовые движения в реальном времени:

```rust
/// Потоковый детектор аномалий на основе Z-score
struct AnomalyDetector {
    stats: StreamingStats,
    threshold: f64,  // Порог в стандартных отклонениях
}

/// Статистика для детектора (упрощённая версия)
struct StreamingStats {
    count: u64,
    mean: f64,
    m2: f64,
}

impl StreamingStats {
    fn new() -> Self {
        StreamingStats { count: 0, mean: 0.0, m2: 0.0 }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    fn std_dev(&self) -> f64 {
        if self.count < 2 { 0.0 }
        else { (self.m2 / (self.count - 1) as f64).sqrt() }
    }
}

impl AnomalyDetector {
    fn new(threshold: f64) -> Self {
        AnomalyDetector {
            stats: StreamingStats::new(),
            threshold,
        }
    }

    /// Проверяет, является ли значение аномалией
    fn check(&mut self, value: f64) -> (bool, f64) {
        let std_dev = self.stats.std_dev();
        let mean = self.stats.mean;

        // Рассчитываем Z-score
        let z_score = if std_dev > 0.0 && self.stats.count > 10 {
            (value - mean) / std_dev
        } else {
            0.0
        };

        // Обновляем статистику
        self.stats.update(value);

        // Аномалия, если Z-score превышает порог
        let is_anomaly = z_score.abs() > self.threshold;

        (is_anomaly, z_score)
    }
}

fn main() {
    let mut detector = AnomalyDetector::new(2.0);  // 2 стандартных отклонения

    // Поток изменений цены (%)
    let price_changes = [
        0.1, 0.2, -0.1, 0.15, -0.2, 0.1, -0.15, 0.2,
        0.1, -0.1, 0.15, -0.2,
        5.0,   // <- Аномалия! Резкий скачок
        0.1, -0.1, 0.2, -0.15,
        -4.5,  // <- Аномалия! Резкое падение
        0.1, 0.15
    ];

    println!("Streaming Anomaly Detection (threshold: 2σ)");
    println!("═══════════════════════════════════════════");

    for (i, &change) in price_changes.iter().enumerate() {
        let (is_anomaly, z_score) = detector.check(change);

        if is_anomaly {
            println!("⚠️  [{}] Change: {:>6.2}% | Z-score: {:>6.2} | ANOMALY!",
                     i, change, z_score);
        } else {
            println!("   [{:>2}] Change: {:>6.2}% | Z-score: {:>6.2}",
                     i, change, z_score);
        }
    }
}
```

## Агрегация свечей в реальном времени

Собираем OHLCV свечи из потока сделок:

```rust
/// Свеча OHLCV
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_count: u32,
}

/// Билдер свечи из потока сделок
struct CandleBuilder {
    current: Option<Candle>,
}

impl CandleBuilder {
    fn new() -> Self {
        CandleBuilder { current: None }
    }

    /// Добавляет сделку к текущей свече
    fn add_trade(&mut self, price: f64, volume: f64) {
        match &mut self.current {
            Some(candle) => {
                if price > candle.high {
                    candle.high = price;
                }
                if price < candle.low {
                    candle.low = price;
                }
                candle.close = price;
                candle.volume += volume;
                candle.trade_count += 1;
            }
            None => {
                self.current = Some(Candle {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    trade_count: 1,
                });
            }
        }
    }

    /// Закрывает текущую свечу и начинает новую
    fn close_candle(&mut self) -> Option<Candle> {
        self.current.take()
    }
}

fn main() {
    let mut builder = CandleBuilder::new();

    // Поток сделок: (цена, объём)
    let trades = [
        (42000.0, 1.0),
        (42050.0, 0.5),
        (42100.0, 2.0),  // High
        (41950.0, 1.5),  // Low
        (42020.0, 0.8),  // Close
    ];

    println!("Building candle from trade stream...\n");

    for (i, &(price, volume)) in trades.iter().enumerate() {
        builder.add_trade(price, volume);
        println!("Trade {}: price={}, volume={}", i + 1, price, volume);
    }

    if let Some(candle) = builder.close_candle() {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║          COMPLETED CANDLE             ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Open:        {:>20.2} ║", candle.open);
        println!("║ High:        {:>20.2} ║", candle.high);
        println!("║ Low:         {:>20.2} ║", candle.low);
        println!("║ Close:       {:>20.2} ║", candle.close);
        println!("║ Volume:      {:>20.2} ║", candle.volume);
        println!("║ Trades:      {:>20} ║", candle.trade_count);
        println!("╚═══════════════════════════════════════╝");
    }
}
```

## Паттерны потоковой обработки

```rust
// 1. Map — преобразование каждого элемента
let prices = [42000.0, 42100.0, 42050.0];
let returns: Vec<f64> = prices
    .windows(2)
    .map(|w| (w[1] - w[0]) / w[0] * 100.0)
    .collect();

// 2. Filter — отбор по условию
let large_trades: Vec<f64> = volumes
    .iter()
    .filter(|&&v| v > 10.0)
    .cloned()
    .collect();

// 3. Fold/Reduce — агрегация
let total: f64 = prices.iter().fold(0.0, |acc, &x| acc + x);

// 4. Take/Skip — ограничение потока
let first_10: Vec<_> = prices.iter().take(10).collect();
let after_warmup: Vec<_> = prices.iter().skip(100).collect();

// 5. Scan — накопительное преобразование
let cumulative: Vec<f64> = prices
    .iter()
    .scan(0.0, |state, &x| {
        *state += x;
        Some(*state)
    })
    .collect();
```

## Что мы узнали

| Концепция | Описание | Применение |
|-----------|----------|------------|
| Итераторы | Ленивые потоки данных | Обработка больших файлов |
| BufReader | Буферизованное чтение | Построчное чтение логов |
| VecDeque | Очередь с двумя концами | Скользящие окна |
| Каналы (mpsc) | Передача между потоками | Разделение производителя и потребителя |
| Алгоритм Уэлфорда | Потоковое среднее/дисперсия | Статистика в реальном времени |

## Домашнее задание

1. Реализуй `StreamingEMA` — потоковую экспоненциальную скользящую среднюю с параметром сглаживания

2. Создай `StreamingBollingerBands` — потоковый расчёт полос Боллинджера (средняя ± 2 стандартных отклонения)

3. Напиши программу, которая читает большой CSV файл с историей торгов и строит свечи с интервалом 1 минута, используя только потоковую обработку

4. Реализуй потоковый детектор паттерна "двойная вершина" на основе скользящего окна последних N свечей

## Навигация

[← Предыдущий день](../148-data-compression/ru.md) | [Следующий день →](../150-memoization/ru.md)
