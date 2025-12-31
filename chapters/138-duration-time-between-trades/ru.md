# День 138: Duration — время между сделками

## Аналогия из трейдинга

Представь, что ты анализируешь свои сделки. Тебе важно знать: сколько времени прошло между входом и выходом из позиции? Держал ты BTC 5 минут (скальпинг) или 3 дня (свинг-трейдинг)? `Duration` в Rust — это именно такой "отрезок времени". Это не конкретная дата, а **промежуток** — как время удержания позиции.

## Что такое Duration

`Duration` — это тип из стандартной библиотеки Rust (`std::time::Duration`), представляющий промежуток времени. Он хранит время в секундах и наносекундах, обеспечивая высокую точность.

```rust
use std::time::Duration;

fn main() {
    // Создание Duration разными способами
    let five_seconds = Duration::from_secs(5);
    let half_second = Duration::from_millis(500);
    let microseconds = Duration::from_micros(1000);
    let nanoseconds = Duration::from_nanos(1_000_000);

    println!("5 секунд: {:?}", five_seconds);
    println!("500 мс: {:?}", half_second);
    println!("1000 мкс: {:?}", microseconds);
    println!("1_000_000 нс: {:?}", nanoseconds);
}
```

## Duration в трейдинге

### Время удержания позиции

```rust
use std::time::Duration;

fn main() {
    // Время удержания разных типов сделок
    let scalp_trade = Duration::from_secs(45);           // 45 секунд — скальп
    let day_trade = Duration::from_secs(4 * 60 * 60);    // 4 часа — дейтрейд
    let swing_trade = Duration::from_secs(3 * 24 * 60 * 60); // 3 дня — свинг

    println!("Скальп: {} сек", scalp_trade.as_secs());
    println!("Дейтрейд: {} часов", day_trade.as_secs() / 3600);
    println!("Свинг: {} дней", swing_trade.as_secs() / 86400);

    // Классификация сделки по времени удержания
    classify_trade(scalp_trade);
    classify_trade(day_trade);
    classify_trade(swing_trade);
}

fn classify_trade(holding_time: Duration) {
    let minutes = holding_time.as_secs() / 60;

    let trade_type = if minutes < 5 {
        "Скальп"
    } else if minutes < 60 {
        "Краткосрочная"
    } else if minutes < 24 * 60 {
        "Дейтрейд"
    } else {
        "Свинг/Позиционная"
    };

    println!("Сделка длительностью {} мин = {}", minutes, trade_type);
}
```

### Время между сделками

```rust
use std::time::Duration;

fn main() {
    // Время между последовательными сделками
    let trade_intervals = vec![
        Duration::from_secs(120),   // 2 минуты после первой сделки
        Duration::from_secs(45),    // 45 секунд после второй
        Duration::from_secs(300),   // 5 минут после третьей
        Duration::from_secs(60),    // 1 минута после четвёртой
    ];

    // Анализ частоты торговли
    let total_time: Duration = trade_intervals.iter().sum();
    let avg_interval = total_time / trade_intervals.len() as u32;

    println!("Общее время торговой сессии: {} сек", total_time.as_secs());
    println!("Среднее время между сделками: {} сек", avg_interval.as_secs());
    println!("Количество сделок: {}", trade_intervals.len() + 1);

    // Сделок в час (если бы торговали с такой частотой)
    let trades_per_hour = 3600.0 / avg_interval.as_secs_f64();
    println!("Темп: {:.1} сделок в час", trades_per_hour);
}
```

## Операции с Duration

### Арифметика времени

```rust
use std::time::Duration;

fn main() {
    let entry_to_sl = Duration::from_secs(30);    // До стоп-лосса
    let sl_to_exit = Duration::from_secs(15);     // После стоп-лосса до закрытия

    // Сложение
    let total_time = entry_to_sl + sl_to_exit;
    println!("Общее время в сделке: {} сек", total_time.as_secs());

    // Умножение
    let three_trades = total_time * 3;
    println!("Время на 3 таких сделки: {} сек", three_trades.as_secs());

    // Деление
    let half_time = total_time / 2;
    println!("Половина времени: {} сек", half_time.as_secs());

    // Вычитание (с проверкой)
    if let Some(diff) = entry_to_sl.checked_sub(sl_to_exit) {
        println!("Разница: {} сек", diff.as_secs());
    }

    // saturating_sub — безопасное вычитание (не уходит в минус)
    let safe_diff = sl_to_exit.saturating_sub(entry_to_sl);
    println!("Безопасная разница: {} сек", safe_diff.as_secs());
}
```

### Сравнение Duration

```rust
use std::time::Duration;

fn main() {
    let max_holding_time = Duration::from_secs(300); // 5 минут максимум
    let current_holding = Duration::from_secs(180);  // 3 минуты держим

    if current_holding < max_holding_time {
        let remaining = max_holding_time - current_holding;
        println!("Можно держать ещё {} сек", remaining.as_secs());
    } else {
        println!("Пора закрывать позицию!");
    }

    // Проверка на превышение лимита
    let trades = vec![
        ("BTCUSD", Duration::from_secs(240)),
        ("ETHUSD", Duration::from_secs(600)),
        ("SOLUSD", Duration::from_secs(180)),
    ];

    for (symbol, holding) in &trades {
        if *holding > max_holding_time {
            println!("{}: ПРЕВЫШЕН лимит времени!", symbol);
        } else {
            println!("{}: в пределах лимита", symbol);
        }
    }
}
```

## Методы Duration

### Извлечение компонентов

```rust
use std::time::Duration;

fn main() {
    let trade_duration = Duration::new(3723, 500_000_000); // 1 час 2 мин 3.5 сек

    // Общее время в разных единицах
    println!("В секундах: {}", trade_duration.as_secs());
    println!("В миллисекундах: {}", trade_duration.as_millis());
    println!("В микросекундах: {}", trade_duration.as_micros());
    println!("В наносекундах: {}", trade_duration.as_nanos());

    // Как дробное число
    println!("В секундах (f64): {:.3}", trade_duration.as_secs_f64());
    println!("В секундах (f32): {:.3}", trade_duration.as_secs_f32());

    // Отдельные компоненты
    println!("Секунды (целая часть): {}", trade_duration.as_secs());
    println!("Наносекунды (дробная часть): {}", trade_duration.subsec_nanos());
    println!("Миллисекунды (дробная часть): {}", trade_duration.subsec_millis());
}
```

### Форматирование для трейдинга

```rust
use std::time::Duration;

fn main() {
    let holding_times = vec![
        Duration::from_secs(45),
        Duration::from_secs(3661),
        Duration::from_secs(86400 + 7200 + 180),
    ];

    for duration in holding_times {
        println!("{}", format_trading_duration(duration));
    }
}

fn format_trading_duration(d: Duration) -> String {
    let total_secs = d.as_secs();

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{}д {}ч {}м {}с", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}ч {}м {}с", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}м {}с", minutes, seconds)
    } else {
        format!("{}с", seconds)
    }
}
```

## Duration + chrono

Крейт `chrono` расширяет возможности работы со временем:

```rust
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::time::Duration as StdDuration;

fn main() {
    // Время входа и выхода из сделки
    let entry_time: DateTime<Utc> = "2024-01-15T10:30:00Z".parse().unwrap();
    let exit_time: DateTime<Utc> = "2024-01-15T14:45:30Z".parse().unwrap();

    // Разница — это chrono::Duration
    let holding_time: ChronoDuration = exit_time - entry_time;

    println!("Вход: {}", entry_time);
    println!("Выход: {}", exit_time);
    println!("Время в позиции: {} часов {} минут {} секунд",
        holding_time.num_hours(),
        holding_time.num_minutes() % 60,
        holding_time.num_seconds() % 60
    );

    // Конвертация в std::time::Duration
    if let Ok(std_duration) = holding_time.to_std() {
        println!("std::time::Duration: {:?}", std_duration);
    }

    // Создание chrono::Duration
    let max_hold = ChronoDuration::hours(8);
    if holding_time < max_hold {
        println!("Сделка в пределах дневного лимита");
    }
}
```

## Практический пример: Анализ сделок

```rust
use std::time::Duration;

fn main() {
    let trades = vec![
        Trade::new("BTCUSD", 42000.0, 42500.0, Duration::from_secs(180)),
        Trade::new("ETHUSD", 2200.0, 2150.0, Duration::from_secs(3600)),
        Trade::new("BTCUSD", 42100.0, 42800.0, Duration::from_secs(45)),
        Trade::new("SOLUSD", 95.0, 98.5, Duration::from_secs(7200)),
    ];

    let analysis = analyze_trades(&trades);
    print_analysis(&analysis);
}

struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    holding_time: Duration,
}

impl Trade {
    fn new(symbol: &str, entry: f64, exit: f64, holding: Duration) -> Self {
        Trade {
            symbol: symbol.to_string(),
            entry_price: entry,
            exit_price: exit,
            holding_time: holding,
        }
    }

    fn pnl_percent(&self) -> f64 {
        ((self.exit_price - self.entry_price) / self.entry_price) * 100.0
    }

    fn is_profitable(&self) -> bool {
        self.exit_price > self.entry_price
    }
}

struct TradeAnalysis {
    total_trades: usize,
    profitable_trades: usize,
    total_holding_time: Duration,
    avg_holding_time: Duration,
    fastest_trade: Duration,
    slowest_trade: Duration,
    avg_pnl_percent: f64,
}

fn analyze_trades(trades: &[Trade]) -> TradeAnalysis {
    let total_trades = trades.len();
    let profitable_trades = trades.iter().filter(|t| t.is_profitable()).count();

    let total_holding_time: Duration = trades.iter()
        .map(|t| t.holding_time)
        .sum();

    let avg_holding_time = total_holding_time / total_trades as u32;

    let fastest_trade = trades.iter()
        .map(|t| t.holding_time)
        .min()
        .unwrap_or(Duration::ZERO);

    let slowest_trade = trades.iter()
        .map(|t| t.holding_time)
        .max()
        .unwrap_or(Duration::ZERO);

    let avg_pnl_percent = trades.iter()
        .map(|t| t.pnl_percent())
        .sum::<f64>() / total_trades as f64;

    TradeAnalysis {
        total_trades,
        profitable_trades,
        total_holding_time,
        avg_holding_time,
        fastest_trade,
        slowest_trade,
        avg_pnl_percent,
    }
}

fn print_analysis(a: &TradeAnalysis) {
    println!("╔══════════════════════════════════════╗");
    println!("║        АНАЛИЗ СДЕЛОК                 ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Всего сделок:        {:>14} ║", a.total_trades);
    println!("║ Прибыльных:          {:>14} ║", a.profitable_trades);
    println!("║ Винрейт:             {:>13.1}% ║",
        (a.profitable_trades as f64 / a.total_trades as f64) * 100.0);
    println!("║ Средний PnL:         {:>13.2}% ║", a.avg_pnl_percent);
    println!("╠══════════════════════════════════════╣");
    println!("║ Общее время:         {:>11} сек ║", a.total_holding_time.as_secs());
    println!("║ Среднее время:       {:>11} сек ║", a.avg_holding_time.as_secs());
    println!("║ Самая быстрая:       {:>11} сек ║", a.fastest_trade.as_secs());
    println!("║ Самая долгая:        {:>11} сек ║", a.slowest_trade.as_secs());
    println!("╚══════════════════════════════════════╝");
}
```

## Измерение времени выполнения

```rust
use std::time::{Duration, Instant};

fn main() {
    // Замеряем время расчёта индикатора
    let prices: Vec<f64> = (0..10000).map(|i| 42000.0 + (i as f64 * 0.1)).collect();

    let start = Instant::now();
    let sma = calculate_sma(&prices, 20);
    let elapsed: Duration = start.elapsed();

    println!("SMA-20 рассчитан за {:?}", elapsed);
    println!("Последнее значение: {:.2}", sma.last().unwrap_or(&0.0));

    // Сравнение разных реализаций
    benchmark_implementations(&prices);
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();
    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

fn benchmark_implementations(prices: &[f64]) {
    // Простая реализация
    let start = Instant::now();
    for _ in 0..100 {
        let _: Vec<f64> = prices.iter()
            .enumerate()
            .filter(|(i, _)| *i >= 19)
            .map(|(i, _)| prices[i-19..=i].iter().sum::<f64>() / 20.0)
            .collect();
    }
    let simple_time = start.elapsed();

    // Оптимизированная реализация
    let start = Instant::now();
    for _ in 0..100 {
        let _ = calculate_sma(prices, 20);
    }
    let optimized_time = start.elapsed();

    println!("\nСравнение (100 итераций):");
    println!("Простая: {:?}", simple_time);
    println!("Оптимизированная: {:?}", optimized_time);
    println!("Ускорение: {:.1}x",
        simple_time.as_secs_f64() / optimized_time.as_secs_f64());
}
```

## Таймауты и задержки

```rust
use std::time::Duration;
use std::thread;

fn main() {
    println!("Ожидаем подтверждения ордера...");

    // Имитация ожидания с таймаутом
    let timeout = Duration::from_secs(5);
    let check_interval = Duration::from_millis(500);
    let mut elapsed = Duration::ZERO;
    let mut confirmed = false;

    while elapsed < timeout {
        // Имитация проверки статуса
        if check_order_status() {
            confirmed = true;
            break;
        }

        thread::sleep(check_interval);
        elapsed += check_interval;
        println!("  Прошло: {:?}", elapsed);
    }

    if confirmed {
        println!("Ордер подтверждён!");
    } else {
        println!("Таймаут: ордер не подтверждён за {:?}", timeout);
    }
}

fn check_order_status() -> bool {
    // Имитация: подтверждается на 3-й проверке
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER >= 3
    }
}
```

## Константы Duration

```rust
use std::time::Duration;

fn main() {
    // Полезные константы
    println!("ZERO: {:?}", Duration::ZERO);
    println!("MAX: {:?}", Duration::MAX);

    // Проверка на ноль
    let no_delay = Duration::ZERO;
    if no_delay.is_zero() {
        println!("Нет задержки");
    }

    // Безопасные операции
    let result = Duration::MAX.checked_add(Duration::from_secs(1));
    match result {
        Some(d) => println!("Результат: {:?}", d),
        None => println!("Переполнение!"),
    }
}
```

## Что мы узнали

| Метод | Описание | Пример |
|-------|----------|--------|
| `Duration::from_secs(n)` | Из секунд | `Duration::from_secs(60)` |
| `Duration::from_millis(n)` | Из миллисекунд | `Duration::from_millis(500)` |
| `Duration::from_secs_f64(f)` | Из дробных секунд | `Duration::from_secs_f64(1.5)` |
| `.as_secs()` | В секунды | `d.as_secs()` |
| `.as_millis()` | В миллисекунды | `d.as_millis()` |
| `.as_secs_f64()` | В дробные секунды | `d.as_secs_f64()` |
| `+`, `-`, `*`, `/` | Арифметика | `d1 + d2` |
| `.checked_add()` | Безопасное сложение | `d.checked_add(d2)` |
| `.saturating_sub()` | Вычитание без минуса | `d.saturating_sub(d2)` |
| `Duration::ZERO` | Нулевая длительность | Сравнение |
| `.is_zero()` | Проверка на ноль | `d.is_zero()` |

## Домашнее задание

1. Напиши функцию `average_trade_duration(trades: &[Trade]) -> Option<Duration>`, которая возвращает среднее время удержания позиции

2. Создай функцию `classify_by_duration(trades: &[Trade]) -> HashMap<String, Vec<&Trade>>`, которая группирует сделки по типам: "scalp" (< 5 мин), "intraday" (5 мин - 24 ч), "swing" (> 24 ч)

3. Реализуй функцию `calculate_time_weighted_pnl(trades: &[Trade]) -> f64`, которая рассчитывает PnL, взвешенный по времени удержания (более длинные сделки имеют больший вес)

4. Напиши структуру `TradingSession` с методами:
   - `start()` — начало сессии
   - `record_trade(trade: Trade)` — запись сделки
   - `finish()` — конец сессии
   - `report()` — отчёт со временем сессии, количеством сделок, средним интервалом между сделками

## Навигация

[← Предыдущий день](../137-timestamp-unix-time/ru.md) | [Следующий день →](../139-time-formatting/ru.md)
