# День 26: Константы — фиксированная комиссия биржи

## Аналогия из трейдинга

На каждой бирже есть **комиссия за сделку** — фиксированный процент, который не меняется во время работы бота. Binance берёт 0.1%, Bybit — 0.1%, Kraken — 0.16%. Эти значения известны заранее и остаются постоянными.

В Rust для таких значений есть специальное ключевое слово — `const`. Константы **никогда не меняются** и вычисляются во время компиляции.

## Объявление констант

```rust
const BINANCE_FEE: f64 = 0.001;        // 0.1%
const BYBIT_FEE: f64 = 0.001;          // 0.1%
const MAX_LEVERAGE: u32 = 125;          // Максимальное плечо
const MIN_ORDER_USDT: f64 = 10.0;       // Минимальный ордер

fn main() {
    println!("Комиссия Binance: {}%", BINANCE_FEE * 100.0);
    println!("Максимальное плечо: {}x", MAX_LEVERAGE);
    println!("Минимальный ордер: {} USDT", MIN_ORDER_USDT);
}
```

## Правила именования констант

Константы пишутся **ЗАГЛАВНЫМИ_БУКВАМИ_С_ПОДЧЁРКИВАНИЕМ**:

```rust
const TRADING_FEE: f64 = 0.001;          // Правильно
const MAX_POSITION_SIZE: f64 = 100000.0; // Правильно
const minOrderSize: f64 = 10.0;          // Неправильно! (но скомпилируется)
```

## Константы vs. let

| Характеристика | `const` | `let` (без mut) |
|---------------|---------|-----------------|
| Изменяемость | Никогда | Нет, но можно shadowing |
| Тип | Обязателен | Может выводиться |
| Вычисление | При компиляции | При выполнении |
| Область видимости | Глобальная возможна | Только локальная |
| Именование | UPPER_SNAKE_CASE | snake_case |

```rust
const MAKER_FEE: f64 = 0.0002;  // Известна при компиляции

fn main() {
    let taker_fee = 0.0004;     // Вычисляется при выполнении

    // Константу нельзя изменить
    // MAKER_FEE = 0.0003;      // ОШИБКА!

    // let без mut тоже нельзя, но можно shadowing
    let taker_fee = 0.0005;     // Создаёт новую переменную
}
```

## Константы для параметров стратегии

```rust
// Параметры риск-менеджмента
const MAX_RISK_PER_TRADE: f64 = 0.02;    // 2% от депозита
const MAX_DAILY_LOSS: f64 = 0.06;        // 6% макс. убыток в день
const MAX_OPEN_POSITIONS: u32 = 5;       // Макс. открытых позиций

// Параметры торговли
const DEFAULT_LEVERAGE: u32 = 10;
const STOP_LOSS_PERCENT: f64 = 0.02;     // 2% стоп-лосс
const TAKE_PROFIT_PERCENT: f64 = 0.04;   // 4% тейк-профит

fn main() {
    let deposit = 10000.0;

    let max_loss_per_trade = deposit * MAX_RISK_PER_TRADE;
    let max_daily_loss_amount = deposit * MAX_DAILY_LOSS;

    println!("=== Риск-менеджмент ===");
    println!("Депозит: {} USDT", deposit);
    println!("Макс. убыток на сделку: {} USDT ({}%)",
        max_loss_per_trade, MAX_RISK_PER_TRADE * 100.0);
    println!("Макс. убыток в день: {} USDT ({}%)",
        max_daily_loss_amount, MAX_DAILY_LOSS * 100.0);
    println!("Макс. открытых позиций: {}", MAX_OPEN_POSITIONS);
}
```

## Глобальные константы

Константы можно объявлять **вне функций**:

```rust
// Глобальные константы доступны везде
const EXCHANGE_NAME: &str = "Binance";
const API_RATE_LIMIT: u32 = 1200;        // Запросов в минуту
const WEBSOCKET_PING_INTERVAL: u32 = 30; // Секунд

fn print_exchange_info() {
    println!("Биржа: {}", EXCHANGE_NAME);
    println!("Лимит API: {} req/min", API_RATE_LIMIT);
}

fn print_websocket_info() {
    println!("Ping каждые {} секунд", WEBSOCKET_PING_INTERVAL);
}

fn main() {
    print_exchange_info();
    print_websocket_info();
}
```

## Вычисляемые константы

Константы могут использовать **константные выражения**:

```rust
const MINUTES_PER_HOUR: u32 = 60;
const HOURS_PER_DAY: u32 = 24;
const MINUTES_PER_DAY: u32 = MINUTES_PER_HOUR * HOURS_PER_DAY;

const TRADING_FEE_PERCENT: f64 = 0.1;
const TRADING_FEE: f64 = TRADING_FEE_PERCENT / 100.0;

// Используем в расчётах позиции
const DEFAULT_POSITION_USDT: f64 = 1000.0;
const FEE_FOR_ROUND_TRIP: f64 = DEFAULT_POSITION_USDT * TRADING_FEE * 2.0;

fn main() {
    println!("Минут в дне: {}", MINUTES_PER_DAY);
    println!("Комиссия: {}%", TRADING_FEE_PERCENT);
    println!("Комиссия за открытие+закрытие {} USDT: {} USDT",
        DEFAULT_POSITION_USDT, FEE_FOR_ROUND_TRIP);
}
```

## Типы данных в константах

Тип **обязательно** нужно указывать:

```rust
const LEVERAGE: u32 = 10;              // u32
const FEE: f64 = 0.001;                // f64
const IS_TESTNET: bool = true;         // bool
const TICKER: &str = "BTCUSDT";        // &str (строковый срез)
const DECIMALS: usize = 8;             // usize

fn main() {
    println!("Тикер: {}", TICKER);
    println!("Плечо: {}x", LEVERAGE);
    println!("Тестовая сеть: {}", IS_TESTNET);
}
```

## Практический пример: Калькулятор комиссий

```rust
// Комиссии разных бирж
const BINANCE_SPOT_FEE: f64 = 0.001;
const BINANCE_FUTURES_FEE: f64 = 0.0004;
const BYBIT_FEE: f64 = 0.001;
const OKX_FEE: f64 = 0.0008;

// Скидки для VIP
const VIP1_DISCOUNT: f64 = 0.1;   // 10% скидка
const VIP2_DISCOUNT: f64 = 0.2;   // 20% скидка
const VIP3_DISCOUNT: f64 = 0.3;   // 30% скидка

fn calculate_fee(volume: f64, base_fee: f64, discount: f64) -> f64 {
    let effective_fee = base_fee * (1.0 - discount);
    volume * effective_fee
}

fn main() {
    let trade_volume = 50000.0;  // 50,000 USDT

    println!("=== Калькулятор комиссий ===");
    println!("Объём сделки: {} USDT\n", trade_volume);

    // Без скидки
    let binance_fee = calculate_fee(trade_volume, BINANCE_SPOT_FEE, 0.0);
    let bybit_fee = calculate_fee(trade_volume, BYBIT_FEE, 0.0);

    println!("Binance Spot: {} USDT", binance_fee);
    println!("Bybit: {} USDT", bybit_fee);

    // Со скидкой VIP2
    let binance_vip2 = calculate_fee(trade_volume, BINANCE_SPOT_FEE, VIP2_DISCOUNT);
    println!("\nBinance с VIP2 скидкой: {} USDT", binance_vip2);
    println!("Экономия: {} USDT", binance_fee - binance_vip2);
}
```

## Практический пример: Торговый бот с константами

```rust
// === КОНФИГУРАЦИЯ БОТА ===
const BOT_NAME: &str = "TrendFollower v1.0";
const EXCHANGE: &str = "Binance Futures";

// Торговые параметры
const TRADING_PAIR: &str = "BTCUSDT";
const LEVERAGE: u32 = 10;
const POSITION_SIZE_USDT: f64 = 1000.0;

// Риск-менеджмент
const STOP_LOSS_PERCENT: f64 = 0.015;     // 1.5%
const TAKE_PROFIT_PERCENT: f64 = 0.03;    // 3%
const MAX_TRADES_PER_DAY: u32 = 10;

// Комиссии
const TAKER_FEE: f64 = 0.0004;
const MAKER_FEE: f64 = 0.0002;

fn main() {
    println!("=== {} ===", BOT_NAME);
    println!("Биржа: {}", EXCHANGE);
    println!("Пара: {}", TRADING_PAIR);
    println!();

    // Расчёт параметров сделки
    let entry_price = 42000.0;
    let position_btc = POSITION_SIZE_USDT / entry_price;

    let stop_loss_price = entry_price * (1.0 - STOP_LOSS_PERCENT);
    let take_profit_price = entry_price * (1.0 + TAKE_PROFIT_PERCENT);

    println!("=== Параметры сделки ===");
    println!("Размер позиции: {} USDT ({:.6} BTC)", POSITION_SIZE_USDT, position_btc);
    println!("Плечо: {}x", LEVERAGE);
    println!("Цена входа: {} USDT", entry_price);
    println!("Стоп-лосс: {} USDT ({:.1}%)", stop_loss_price, STOP_LOSS_PERCENT * 100.0);
    println!("Тейк-профит: {} USDT ({:.1}%)", take_profit_price, TAKE_PROFIT_PERCENT * 100.0);

    // Расчёт потенциального P&L
    let max_loss = POSITION_SIZE_USDT * STOP_LOSS_PERCENT * LEVERAGE as f64;
    let max_profit = POSITION_SIZE_USDT * TAKE_PROFIT_PERCENT * LEVERAGE as f64;

    // Учитываем комиссию (вход + выход)
    let total_fee = POSITION_SIZE_USDT * TAKER_FEE * 2.0;

    println!();
    println!("=== Потенциальный P&L ===");
    println!("Макс. убыток: -{:.2} USDT", max_loss);
    println!("Макс. прибыль: +{:.2} USDT", max_profit);
    println!("Комиссия (туда-обратно): {:.2} USDT", total_fee);
    println!("Чистая макс. прибыль: +{:.2} USDT", max_profit - total_fee);
}
```

## static vs const

Есть также `static`, но для простых значений используйте `const`:

```rust
const TRADING_FEE: f64 = 0.001;     // Рекомендуется для простых значений
static EXCHANGE: &str = "Binance"; // Имеет фиксированный адрес в памяти

fn main() {
    println!("Комиссия: {}", TRADING_FEE);
    println!("Биржа: {}", EXCHANGE);
}
```

**Когда использовать `static`:**
- Когда нужен фиксированный адрес в памяти
- Для изменяемых глобальных данных (`static mut`, небезопасно)
- Для больших массивов данных

**Для большинства случаев в трейдинге используйте `const`.**

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `const` | Константа, известная при компиляции |
| UPPER_SNAKE_CASE | Стиль именования констант |
| Глобальные константы | Доступны во всех функциях |
| Обязательный тип | Константы требуют указания типа |
| Константные выражения | Можно вычислять на основе других констант |

## Упражнения

### Упражнение 1: Конфигурация биржи

Создайте набор констант для разных бирж:

```rust
// Создайте константы для:
// - Названия бирж
// - Комиссии maker/taker
// - API лимиты
// - Минимальные размеры ордеров

fn main() {
    // Выведите конфигурацию для каждой биржи
}
```

### Упражнение 2: Калькулятор позиции

Используя константы, создайте калькулятор размера позиции:

```rust
const MAX_RISK_PERCENT: f64 = 0.02;  // 2% риска
const LEVERAGE: u32 = 10;

fn main() {
    let deposit = 5000.0;
    let entry_price = 42000.0;
    let stop_loss_price = 41500.0;

    // Рассчитайте:
    // 1. Максимальный риск в USDT
    // 2. Размер позиции для данного стоп-лосса
    // 3. Количество BTC в позиции
}
```

### Упражнение 3: Таблица комиссий

```rust
// Создайте константы для разных уровней VIP
// Выведите таблицу комиссий для объёмов:
// 10,000 / 50,000 / 100,000 / 500,000 USDT
```

## Домашнее задание

1. **Создайте файл конфигурации для своего торгового бота:**
   - Параметры риск-менеджмента (максимальный убыток, размер позиции)
   - Параметры торговли (плечо, стоп-лосс, тейк-профит)
   - Параметры биржи (комиссии, лимиты)

2. **Рассчитайте break-even:**
   - При какой прибыли сделка покрывает комиссию?
   - Создайте калькулятор минимального движения цены

3. **Сравните биржи:**
   - Создайте константы для 3-4 бирж
   - Рассчитайте комиссию за 100 сделок по 1000 USDT
   - Определите самую выгодную биржу

## Навигация

[← Предыдущий день](../025-statements-expressions/ru.md) | [Следующий день →](../027-shadowing/ru.md)
