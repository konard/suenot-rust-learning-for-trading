# День 278: Итерация по свечам

## Аналогия из трейдинга

Представь, что ты анализируешь график биткоина за последний год. У тебя есть 365 дневных свечей, и тебе нужно пройти по каждой из них, чтобы найти паттерны, рассчитать индикаторы или проверить торговую стратегию. Это и есть **итерация по свечам** — последовательный обход каждого элемента данных.

В реальном трейдинге итерация используется везде:
- Бэктестинг стратегий — проход по историческим данным
- Расчёт скользящих средних — суммирование последних N свечей
- Поиск экстремумов — нахождение максимальных/минимальных цен
- Анализ объёмов — подсчёт общего объёма за период

## Что такое итератор в Rust?

Итератор — это объект, который позволяет последовательно обходить элементы коллекции. В Rust итераторы:

1. **Ленивые** — не выполняют вычисления, пока не потребуются результаты
2. **Безопасные** — компилятор проверяет границы на этапе компиляции
3. **Эффективные** — часто оптимизируются до уровня ручных циклов
4. **Комбинируемые** — можно соединять множество операций в цепочку

```rust
// Простой пример итерации
let prices = vec![42000.0, 42500.0, 41800.0, 43000.0];

for price in prices.iter() {
    println!("Цена: {}", price);
}
```

## Структура свечи (Candle)

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,      // Unix timestamp
    open: f64,           // Цена открытия
    high: f64,           // Максимальная цена
    low: f64,            // Минимальная цена
    close: f64,          // Цена закрытия
    volume: f64,         // Объём торгов
}

impl Candle {
    fn new(timestamp: u64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Candle { timestamp, open, high, low, close, volume }
    }

    // Свеча бычья (зелёная)?
    fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    // Свеча медвежья (красная)?
    fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    // Размер тела свечи
    fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    // Полный диапазон свечи
    fn range(&self) -> f64 {
        self.high - self.low
    }
}
```

## Основные методы итерации

### 1. iter() — неизменяемая итерация

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Подсчёт бычьих свечей
    let mut bullish_count = 0;
    for candle in candles.iter() {
        if candle.is_bullish() {
            bullish_count += 1;
        }
    }
    println!("Бычьих свечей: {}", bullish_count);

    // candles всё ещё доступен!
    println!("Всего свечей: {}", candles.len());
}
```

### 2. iter_mut() — изменяемая итерация

```rust
fn main() {
    let mut candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
    ];

    // Корректировка объёма на коэффициент
    let adjustment_factor = 1.1;
    for candle in candles.iter_mut() {
        candle.volume *= adjustment_factor;
    }

    for candle in candles.iter() {
        println!("Скорректированный объём: {:.2}", candle.volume);
    }
}
```

### 3. into_iter() — итерация с владением

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
    ];

    // Потребляем вектор
    let closes: Vec<f64> = candles.into_iter()
        .map(|c| c.close)
        .collect();

    println!("Цены закрытия: {:?}", closes);

    // candles больше недоступен — владение передано!
    // println!("{:?}", candles); // Ошибка компиляции!
}
```

## Адаптеры итераторов для трейдинга

### map() — преобразование данных

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Извлекаем только цены закрытия
    let closes: Vec<f64> = candles.iter()
        .map(|c| c.close)
        .collect();

    println!("Цены закрытия: {:?}", closes);

    // Вычисляем типичную цену (typical price)
    let typical_prices: Vec<f64> = candles.iter()
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();

    println!("Типичные цены: {:?}", typical_prices);
}
```

### filter() — фильтрация свечей

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
    ];

    // Только бычьи свечи
    let bullish: Vec<&Candle> = candles.iter()
        .filter(|c| c.is_bullish())
        .collect();

    println!("Бычьих свечей: {}", bullish.len());

    // Свечи с высоким объёмом
    let high_volume: Vec<&Candle> = candles.iter()
        .filter(|c| c.volume > 130.0)
        .collect();

    println!("Свечей с высоким объёмом: {}", high_volume.len());
}
```

### enumerate() — итерация с индексом

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    for (index, candle) in candles.iter().enumerate() {
        println!(
            "Свеча {}: Open={}, Close={}, {}",
            index + 1,
            candle.open,
            candle.close,
            if candle.is_bullish() { "Бычья" } else { "Медвежья" }
        );
    }
}
```

### windows() — скользящее окно

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
        Candle::new(5, 42350.0, 42600.0, 42000.0, 42500.0, 180.0),
    ];

    // Простая скользящая средняя (SMA) с периодом 3
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    println!("SMA(3):");
    for (i, window) in closes.windows(3).enumerate() {
        let sma: f64 = window.iter().sum::<f64>() / window.len() as f64;
        println!("  Период {}-{}: {:.2}", i + 1, i + 3, sma);
    }
}
```

### zip() — объединение данных

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Сравниваем текущую свечу с предыдущей
    for (prev, curr) in candles.iter().zip(candles.iter().skip(1)) {
        let change = ((curr.close - prev.close) / prev.close) * 100.0;
        println!(
            "Свеча {} -> {}: изменение {:.2}%",
            prev.timestamp, curr.timestamp, change
        );
    }
}
```

## Потребляющие адаптеры

### sum() и product()

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Общий объём
    let total_volume: f64 = candles.iter()
        .map(|c| c.volume)
        .sum();

    println!("Общий объём: {}", total_volume);

    // Средний объём
    let avg_volume = total_volume / candles.len() as f64;
    println!("Средний объём: {:.2}", avg_volume);
}
```

### fold() — аккумулирование

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
    ];

    // Находим максимальную и минимальную цену за период
    let (min_low, max_high) = candles.iter().fold(
        (f64::MAX, f64::MIN),
        |(min, max), candle| {
            (min.min(candle.low), max.max(candle.high))
        }
    );

    println!("Диапазон периода: Low={}, High={}", min_low, max_high);
}
```

### find() и position()

```rust
fn main() {
    let candles = vec![
        Candle::new(1, 42000.0, 42500.0, 41800.0, 42300.0, 100.0),
        Candle::new(2, 42300.0, 43000.0, 42200.0, 42800.0, 150.0),
        Candle::new(3, 42800.0, 43200.0, 42600.0, 42100.0, 120.0),
        Candle::new(4, 42100.0, 42400.0, 41900.0, 42350.0, 200.0),
    ];

    // Найти первую свечу с объёмом больше 180
    if let Some(candle) = candles.iter().find(|c| c.volume > 180.0) {
        println!("Найдена свеча с высоким объёмом: timestamp={}", candle.timestamp);
    }

    // Найти позицию первой медвежьей свечи
    if let Some(pos) = candles.iter().position(|c| c.is_bearish()) {
        println!("Первая медвежья свеча на позиции: {}", pos);
    }
}
```

## Практический пример: бэктестинг простой стратегии

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn new(timestamp: u64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Candle { timestamp, open, high, low, close, volume }
    }

    fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit: f64,
}

fn calculate_sma(closes: &[f64], period: usize) -> Vec<f64> {
    closes
        .windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

fn backtest_sma_crossover(candles: &[Candle], fast_period: usize, slow_period: usize) -> Vec<Trade> {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let fast_sma = calculate_sma(&closes, fast_period);
    let slow_sma = calculate_sma(&closes, slow_period);

    // Выравниваем массивы (slow_sma короче)
    let offset = slow_period - fast_period;
    let fast_sma: Vec<f64> = fast_sma.iter().skip(offset).cloned().collect();

    let mut trades = Vec::new();
    let mut position: Option<f64> = None;

    for (i, (fast, slow)) in fast_sma.iter().zip(slow_sma.iter()).enumerate() {
        let prev_fast = if i > 0 { fast_sma[i - 1] } else { *fast };
        let prev_slow = if i > 0 { slow_sma[i - 1] } else { *slow };

        // Пересечение снизу вверх — покупаем
        if prev_fast <= prev_slow && fast > slow && position.is_none() {
            let price = closes[i + slow_period - 1];
            position = Some(price);
            println!("Покупка по цене: {:.2}", price);
        }

        // Пересечение сверху вниз — продаём
        if prev_fast >= prev_slow && fast < slow && position.is_some() {
            let entry_price = position.unwrap();
            let exit_price = closes[i + slow_period - 1];
            let profit = exit_price - entry_price;

            trades.push(Trade {
                entry_price,
                exit_price,
                profit,
            });

            println!("Продажа по цене: {:.2}, Прибыль: {:.2}", exit_price, profit);
            position = None;
        }
    }

    trades
}

fn main() {
    // Генерируем тестовые данные
    let candles: Vec<Candle> = (0..50)
        .map(|i| {
            let base = 42000.0 + (i as f64 * 50.0).sin() * 1000.0;
            Candle::new(
                i,
                base,
                base + 100.0,
                base - 100.0,
                base + 50.0 * (i as f64 * 0.3).cos(),
                100.0 + (i as f64 * 10.0) % 50.0,
            )
        })
        .collect();

    println!("=== Бэктестинг SMA Crossover ===\n");

    let trades = backtest_sma_crossover(&candles, 5, 10);

    println!("\n=== Результаты ===");
    println!("Количество сделок: {}", trades.len());

    if !trades.is_empty() {
        let total_profit: f64 = trades.iter().map(|t| t.profit).sum();
        let winning_trades = trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = (winning_trades as f64 / trades.len() as f64) * 100.0;

        println!("Общая прибыль: {:.2}", total_profit);
        println!("Прибыльных сделок: {} ({:.1}%)", winning_trades, win_rate);
    }
}
```

## Практические упражнения

### Упражнение 1: Поиск паттернов

Реализуй функцию, которая находит паттерн "три белых солдата" (три последовательные бычьи свечи):

```rust
fn find_three_white_soldiers(candles: &[Candle]) -> Vec<usize> {
    // Твой код здесь
    // Верни индексы начала каждого паттерна
    todo!()
}
```

### Упражнение 2: Расчёт ATR

Реализуй расчёт Average True Range (ATR):

```rust
fn calculate_atr(candles: &[Candle], period: usize) -> Vec<f64> {
    // True Range = max(high - low, |high - prev_close|, |low - prev_close|)
    // ATR = SMA(True Range, period)
    todo!()
}
```

### Упражнение 3: Детектор аномалий

Найди свечи с аномально высоким объёмом (больше 2 стандартных отклонений от среднего):

```rust
fn find_volume_anomalies(candles: &[Candle]) -> Vec<&Candle> {
    // Твой код здесь
    todo!()
}
```

### Упражнение 4: Группировка по дням

Преобразуй часовые свечи в дневные:

```rust
fn aggregate_to_daily(hourly_candles: &[Candle]) -> Vec<Candle> {
    // Группируй по 24 часа
    // Open = первый open, Close = последний close
    // High = max(highs), Low = min(lows)
    // Volume = sum(volumes)
    todo!()
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `iter()` | Неизменяемая итерация, оставляет владение |
| `iter_mut()` | Изменяемая итерация |
| `into_iter()` | Итерация с передачей владения |
| `map()` | Преобразование каждого элемента |
| `filter()` | Фильтрация по условию |
| `enumerate()` | Добавляет индекс к элементам |
| `windows()` | Скользящее окно для расчёта индикаторов |
| `zip()` | Объединение двух итераторов |
| `fold()` | Аккумулирование с начальным значением |
| `find()` | Поиск первого элемента по условию |

## Домашнее задание

1. **RSI Calculator**: Реализуй расчёт индекса относительной силы (RSI) с использованием итераторов. RSI = 100 - (100 / (1 + RS)), где RS = средний рост / среднее падение за период.

2. **Bollinger Bands**: Реализуй расчёт полос Боллинджера:
   - Средняя линия = SMA(close, 20)
   - Верхняя полоса = SMA + 2 * стандартное отклонение
   - Нижняя полоса = SMA - 2 * стандартное отклонение

3. **Pattern Scanner**: Создай сканер, который находит различные свечные паттерны (молот, доджи, поглощение) используя цепочки итераторов.

4. **Performance Analyzer**: Напиши анализатор производительности торговой стратегии, который вычисляет:
   - Максимальную просадку (Max Drawdown)
   - Коэффициент Шарпа
   - Профит-фактор (сумма прибыльных / сумма убыточных)

## Навигация

[← Предыдущий день](../277-backtesting-data-structures/ru.md) | [Следующий день →](../279-candle-pattern-detection/ru.md)
