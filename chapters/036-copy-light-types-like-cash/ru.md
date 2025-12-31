# День 36: Copy — лёгкие типы как наличные

## Аналогия из трейдинга

Представь разницу между **наличными деньгами** и **банковским переводом**:

**Наличные (Copy типы):**
- Когда ты даёшь кому-то 100 рублей наличными, ты просто достаёшь купюру из кармана
- Операция мгновенная и дешёвая
- Можно легко "скопировать" — снять ещё денег в банкомате
- Цена BTC = 42000.0 — это просто число, его можно копировать бесконечно

**Банковский перевод (Move типы):**
- Перевод денег требует подтверждения, времени, иногда комиссии
- После перевода деньги уходят с твоего счёта — ты их "переместил"
- История транзакций, строка с названием биржи — это сложные данные

В Rust типы с трейтом `Copy` работают как наличные — они копируются автоматически и дёшево.

## Что такое Copy?

`Copy` — это специальный маркерный трейт в Rust, который говорит компилятору:
> "Этот тип можно безопасно копировать побитово, просто дублируя байты в памяти"

```rust
fn main() {
    // Copy типы — копируются автоматически
    let price = 42000.0_f64;
    let price_copy = price;  // Копирование!

    println!("Original: {}", price);      // Работает!
    println!("Copy: {}", price_copy);     // Тоже работает!

    // Non-Copy типы — перемещаются
    let ticker = String::from("BTC");
    let ticker_moved = ticker;  // Перемещение!

    // println!("{}", ticker);  // ОШИБКА! ticker перемещён
    println!("Moved: {}", ticker_moved);
}
```

## Какие типы имеют Copy?

### Все примитивные числовые типы

```rust
fn main() {
    // Целые числа — Copy
    let shares: i32 = 100;
    let shares_copy = shares;
    println!("Shares: {} and {}", shares, shares_copy);

    // Числа с плавающей точкой — Copy
    let btc_price: f64 = 42000.0;
    let eth_price: f64 = btc_price;  // Копия значения
    println!("BTC: {}, ETH tracking: {}", btc_price, eth_price);

    // Беззнаковые — тоже Copy
    let volume: u64 = 1_000_000;
    process_volume(volume);
    println!("Original volume: {}", volume);  // Всё ещё доступен!
}

fn process_volume(v: u64) {
    println!("Processing volume: {}", v);
}
```

### Bool и Char

```rust
fn main() {
    // bool — Copy
    let is_bull_market = true;
    let market_status = is_bull_market;
    println!("Bull? {} / {}", is_bull_market, market_status);

    // char — Copy
    let trend: char = '↑';
    let saved_trend = trend;
    println!("Trend: {} and saved: {}", trend, saved_trend);
}
```

### Кортежи из Copy типов

```rust
fn main() {
    // Кортеж из Copy типов — тоже Copy!
    let bid_ask: (f64, f64) = (41999.0, 42001.0);
    let spread_data = bid_ask;  // Копирование кортежа

    println!("Original: bid={}, ask={}", bid_ask.0, bid_ask.1);
    println!("Copy: bid={}, ask={}", spread_data.0, spread_data.1);

    // OHLC данные
    let candle: (f64, f64, f64, f64) = (42000.0, 42500.0, 41800.0, 42300.0);
    analyze_candle(candle);
    println!("Candle still available: {:?}", candle);
}

fn analyze_candle(ohlc: (f64, f64, f64, f64)) {
    let (open, high, low, close) = ohlc;
    let range = high - low;
    let body = (close - open).abs();
    println!("Range: {:.2}, Body: {:.2}", range, body);
}
```

### Массивы из Copy типов

```rust
fn main() {
    // Массив из Copy типов — Copy (если размер известен)
    let prices: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let backup = prices;  // Копирование всего массива

    println!("Original: {:?}", prices);
    println!("Backup: {:?}", backup);

    // Передача в функцию — тоже копия
    let avg = calculate_average(prices);
    println!("Average: {:.2}, original: {:?}", avg, prices);
}

fn calculate_average(data: [f64; 5]) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

## Copy vs Clone

Важно понимать разницу:

```rust
fn main() {
    // Copy — неявное, автоматическое, побитовое
    let price = 42000.0;
    let price2 = price;  // Неявная копия

    // Clone — явное, может быть дорогим
    let ticker = String::from("BTC/USDT");
    let ticker2 = ticker.clone();  // Явное клонирование

    println!("Price: {} / {}", price, price2);
    println!("Ticker: {} / {}", ticker, ticker2);
}
```

| Характеристика | Copy | Clone |
|---------------|------|-------|
| Вызов | Автоматический | Явный (.clone()) |
| Стоимость | Всегда дешёвый | Может быть дорогим |
| Глубина | Побитовое копирование | Глубокое копирование |
| Требования | Только простые типы | Любые типы |

## Создание собственного Copy типа

```rust
// Для Copy нужен также Clone
#[derive(Debug, Copy, Clone)]
struct Price {
    value: f64,
    timestamp: u64,
}

#[derive(Debug, Copy, Clone)]
struct OrderLevel {
    price: f64,
    quantity: f64,
    side: bool,  // true = buy, false = sell
}

fn main() {
    let btc_price = Price {
        value: 42000.0,
        timestamp: 1703980800,
    };

    // Теперь Price — Copy!
    let saved_price = btc_price;
    println!("Current: {:?}", btc_price);
    println!("Saved: {:?}", saved_price);

    // OrderLevel тоже Copy
    let bid = OrderLevel {
        price: 41999.0,
        quantity: 0.5,
        side: true,
    };

    process_order(bid);
    println!("Bid still available: {:?}", bid);
}

fn process_order(order: OrderLevel) {
    println!("Processing: {} @ {}",
        if order.side { "BUY" } else { "SELL" },
        order.price
    );
}
```

## Когда Copy невозможен

```rust
// НЕ может быть Copy — содержит String
struct Trade {
    symbol: String,  // String не Copy!
    price: f64,
    quantity: f64,
}

// НЕ может быть Copy — содержит Vec
struct Portfolio {
    positions: Vec<f64>,  // Vec не Copy!
    total_value: f64,
}

// МОЖЕТ быть Copy — только примитивы
#[derive(Copy, Clone)]
struct SimplePosition {
    entry_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let pos = SimplePosition {
        entry_price: 42000.0,
        quantity: 0.5,
        is_long: true,
    };

    let backup = pos;  // Copy работает
    println!("Entry: {} / {}", pos.entry_price, backup.entry_price);
}
```

## Практический пример: система отслеживания цен

```rust
#[derive(Debug, Copy, Clone)]
struct PricePoint {
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug, Copy, Clone)]
struct PriceAlert {
    target_price: f64,
    is_above: bool,  // true = alert when price goes above
    triggered: bool,
}

fn main() {
    // Текущая цена — Copy, можно передавать везде
    let current = PricePoint {
        price: 42500.0,
        volume: 100.0,
        timestamp: 1703980800,
    };

    // Создаём алерты
    let mut alerts = [
        PriceAlert { target_price: 43000.0, is_above: true, triggered: false },
        PriceAlert { target_price: 42000.0, is_above: false, triggered: false },
        PriceAlert { target_price: 45000.0, is_above: true, triggered: false },
    ];

    // Проверяем алерты — current копируется в каждый вызов
    for alert in &mut alerts {
        check_alert(alert, current);
    }

    // current всё ещё доступен!
    println!("\nCurrent price: ${:.2}", current.price);
    println!("\nAlert status:");
    for (i, alert) in alerts.iter().enumerate() {
        println!("  Alert {}: ${:.2} {} - {}",
            i + 1,
            alert.target_price,
            if alert.is_above { "above" } else { "below" },
            if alert.triggered { "TRIGGERED!" } else { "waiting" }
        );
    }
}

fn check_alert(alert: &mut PriceAlert, price: PricePoint) {
    if alert.triggered {
        return;
    }

    let should_trigger = if alert.is_above {
        price.price >= alert.target_price
    } else {
        price.price <= alert.target_price
    };

    if should_trigger {
        alert.triggered = true;
        println!("ALERT: Price ${:.2} {} ${:.2}!",
            price.price,
            if alert.is_above { "crossed above" } else { "dropped below" },
            alert.target_price
        );
    }
}
```

## Copy в функциях: когда это важно

```rust
#[derive(Copy, Clone)]
struct RiskMetrics {
    max_position_size: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
    max_daily_loss: f64,
}

fn main() {
    let risk = RiskMetrics {
        max_position_size: 10000.0,
        stop_loss_percent: 2.0,
        take_profit_percent: 6.0,
        max_daily_loss: 500.0,
    };

    // Благодаря Copy, risk можно использовать многократно
    let position_ok = validate_position_size(risk, 5000.0);
    let stop_price = calculate_stop_loss(risk, 42000.0);
    let take_price = calculate_take_profit(risk, 42000.0);
    let daily_ok = check_daily_risk(risk, 300.0);

    println!("Position valid: {}", position_ok);
    println!("Stop loss: ${:.2}", stop_price);
    println!("Take profit: ${:.2}", take_price);
    println!("Daily risk OK: {}", daily_ok);

    // risk всё ещё доступен после всех вызовов!
    println!("\nMax position: ${:.2}", risk.max_position_size);
}

fn validate_position_size(config: RiskMetrics, size: f64) -> bool {
    size <= config.max_position_size
}

fn calculate_stop_loss(config: RiskMetrics, entry: f64) -> f64 {
    entry * (1.0 - config.stop_loss_percent / 100.0)
}

fn calculate_take_profit(config: RiskMetrics, entry: f64) -> f64 {
    entry * (1.0 + config.take_profit_percent / 100.0)
}

fn check_daily_risk(config: RiskMetrics, current_loss: f64) -> bool {
    current_loss < config.max_daily_loss
}
```

## Упражнения

### Упражнение 1: Определи Copy типы
```rust
// Какие из этих типов Copy?
// 1. i32
// 2. String
// 3. (f64, f64)
// 4. Vec<f64>
// 5. [f64; 3]
// 6. &str
// 7. bool
// 8. (String, i32)

fn main() {
    // Проверь свои предположения!
    let a: i32 = 42;
    let _b = a;
    println!("i32 is Copy: {}", a);  // Компилируется?

    // Добавь проверки для остальных типов...
}
```

### Упражнение 2: Создай Copy структуру

```rust
// Создай структуру Tick с полями:
// - price: f64
// - bid: f64
// - ask: f64
// - volume: f64
// Сделай её Copy и напиши функцию calculate_spread

fn main() {
    // Твой код здесь
}
```

### Упражнение 3: Copy vs Move

```rust
// Исправь код, чтобы он компилировался,
// используя знания о Copy

fn main() {
    let price = 42000.0_f64;
    let ticker = String::from("BTC");

    print_price(price);
    print_price(price);  // Должно работать

    print_ticker(ticker);
    // print_ticker(ticker);  // Как сделать, чтобы работало?
}

fn print_price(p: f64) {
    println!("Price: {}", p);
}

fn print_ticker(t: String) {
    println!("Ticker: {}", t);
}
```

### Упражнение 4: Калькулятор рисков

```rust
// Создай Copy структуру TradeSetup и функции для:
// 1. Расчёта размера позиции
// 2. Расчёта Risk/Reward ratio
// 3. Расчёта максимального убытка

#[derive(Copy, Clone)]
struct TradeSetup {
    // Заполни поля
}

fn main() {
    // Твой код здесь
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Copy трейт | Маркер для типов с дешёвым побитовым копированием |
| Copy типы | i32, f64, bool, char, кортежи и массивы из Copy типов |
| Non-Copy | String, Vec, Box — требуют явного clone() |
| #[derive(Copy, Clone)] | Автоматическая реализация для своих структур |
| Ограничение | Все поля структуры должны быть Copy |

## Домашнее задание

1. **Структура Quote**: Создай Copy структуру для биржевой котировки с полями bid, ask, bid_size, ask_size, timestamp. Напиши функции для расчёта спреда и mid-price.

2. **Система сигналов**: Создай Copy структуру для торгового сигнала (цена входа, стоп, тейк, направление). Напиши функцию, которая принимает сигнал и текущую цену и возвращает действие.

3. **Мульти-таймфрейм**: Создай функцию, которая принимает одну и ту же цену (Copy) и анализирует её относительно разных уровней (массив уровней). Убедись, что цена остаётся доступной после всех проверок.

4. **Copy vs Clone бенчмарк**: Создай две версии структуры — одну Copy (только примитивы), другую с String. Измерь время 1000 копирований и клонирований. Какая разница?

## Навигация

[← Предыдущий день](../035-clone-duplicating-orders/ru.md) | [Следующий день →](../037-drop-cleaning-up-positions/ru.md)
