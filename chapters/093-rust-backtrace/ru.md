# День 93: RUST_BACKTRACE — расследование краха торговой системы

## Аналогия из трейдинга

Представь: твой торговый бот внезапно упал посреди ночи, и ты потерял выгодную позицию. Что произошло? Без **журнала расследования** ты как детектив без улик — можешь только гадать.

`RUST_BACKTRACE` — это твой **чёрный ящик** торговой системы. Как в самолёте чёрный ящик записывает все действия перед катастрофой, так и backtrace показывает **полную цепочку вызовов**, которая привела к краху программы.

## Что такое Backtrace?

Backtrace (стек вызовов) — это список всех функций, которые выполнялись в момент паники программы, в порядке их вызова. Это позволяет:

1. **Найти точное место ошибки** — в какой строке какого файла
2. **Понять контекст** — какие функции вызывались до ошибки
3. **Восстановить логику** — как данные проходили через систему

## Включение Backtrace

### Базовое использование

```rust
fn main() {
    // Запусти с: RUST_BACKTRACE=1 cargo run
    let prices: Vec<f64> = vec![42000.0, 42500.0, 41800.0];

    // Это вызовет панику — индекс вне диапазона!
    let price = prices[10];
    println!("Price: {}", price);
}
```

Запуск без backtrace:
```bash
cargo run
# thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 10'
```

Запуск с backtrace:
```bash
RUST_BACKTRACE=1 cargo run
# Показывает полный стек вызовов с номерами строк!
```

### Уровни детализации

```bash
# Краткий backtrace — основные функции
RUST_BACKTRACE=1 cargo run

# Полный backtrace — включая внутренние функции Rust
RUST_BACKTRACE=full cargo run
```

## Практический пример: отладка торговой стратегии

```rust
fn main() {
    let portfolio = Portfolio::new(10000.0);

    // Симуляция торговли — где-то здесь ошибка
    run_trading_simulation(portfolio);
}

struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        let cost = quantity * price;
        if cost > self.balance {
            panic!("Insufficient balance! Need {} but have {}", cost, self.balance);
        }

        self.balance -= cost;
        self.positions.push(Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
        });
    }

    fn close_position(&mut self, index: usize, exit_price: f64) -> f64 {
        // Потенциальная паника, если index >= positions.len()
        let position = self.positions.remove(index);
        let pnl = (exit_price - position.entry_price) * position.quantity;
        self.balance += position.quantity * exit_price;
        pnl
    }
}

fn run_trading_simulation(mut portfolio: Portfolio) {
    let signals = generate_trading_signals();

    for (i, signal) in signals.iter().enumerate() {
        process_signal(&mut portfolio, signal, i);
    }
}

fn generate_trading_signals() -> Vec<TradeSignal> {
    vec![
        TradeSignal { action: "BUY", symbol: "BTC", price: 42000.0, quantity: 0.1 },
        TradeSignal { action: "BUY", symbol: "ETH", price: 2500.0, quantity: 1.0 },
        TradeSignal { action: "SELL", symbol: "BTC", price: 43000.0, quantity: 0.1 },
        TradeSignal { action: "SELL", symbol: "ETH", price: 2600.0, quantity: 1.0 },
        TradeSignal { action: "SELL", symbol: "SOL", price: 100.0, quantity: 5.0 }, // Ошибка! Нет позиции SOL
    ]
}

struct TradeSignal {
    action: &'static str,
    symbol: &'static str,
    price: f64,
    quantity: f64,
}

fn process_signal(portfolio: &mut Portfolio, signal: &TradeSignal, _signal_index: usize) {
    match signal.action {
        "BUY" => {
            portfolio.open_position(signal.symbol, signal.quantity, signal.price);
            println!("Opened {} position: {} @ {}", signal.symbol, signal.quantity, signal.price);
        }
        "SELL" => {
            // Ищем позицию для закрытия
            let pos_index = find_position_index(portfolio, signal.symbol);
            let pnl = portfolio.close_position(pos_index, signal.price);
            println!("Closed {} position with PnL: ${:.2}", signal.symbol, pnl);
        }
        _ => println!("Unknown signal: {}", signal.action),
    }
}

fn find_position_index(portfolio: &Portfolio, symbol: &str) -> usize {
    // Опасно! Паникует, если позиция не найдена
    portfolio.positions
        .iter()
        .position(|p| p.symbol == symbol)
        .expect(&format!("Position {} not found!", symbol))
}
```

При запуске с `RUST_BACKTRACE=1` вы увидите полную цепочку:
```
thread 'main' panicked at 'Position SOL not found!'

stack backtrace:
   0: find_position_index
             at ./src/main.rs:95
   1: process_signal
             at ./src/main.rs:82
   2: run_trading_simulation
             at ./src/main.rs:58
   3: main
             at ./src/main.rs:4
```

## Исправление с использованием Result

```rust
fn find_position_index(portfolio: &Portfolio, symbol: &str) -> Result<usize, String> {
    portfolio.positions
        .iter()
        .position(|p| p.symbol == symbol)
        .ok_or_else(|| format!("Position {} not found!", symbol))
}

fn process_signal(portfolio: &mut Portfolio, signal: &TradeSignal, _signal_index: usize) -> Result<(), String> {
    match signal.action {
        "BUY" => {
            portfolio.open_position(signal.symbol, signal.quantity, signal.price)?;
            println!("Opened {} position: {} @ {}", signal.symbol, signal.quantity, signal.price);
        }
        "SELL" => {
            let pos_index = find_position_index(portfolio, signal.symbol)?;
            let pnl = portfolio.close_position(pos_index, signal.price);
            println!("Closed {} position with PnL: ${:.2}", signal.symbol, pnl);
        }
        _ => return Err(format!("Unknown signal: {}", signal.action)),
    }
    Ok(())
}
```

## Программное получение Backtrace

```rust
use std::backtrace::Backtrace;

fn execute_trade(symbol: &str, quantity: f64, price: f64) -> Result<(), TradeError> {
    if price <= 0.0 {
        return Err(TradeError::new(
            "Invalid price",
            format!("Price must be positive, got {}", price),
        ));
    }

    if quantity <= 0.0 {
        return Err(TradeError::new(
            "Invalid quantity",
            format!("Quantity must be positive, got {}", quantity),
        ));
    }

    println!("Executing trade: {} {} @ {}", symbol, quantity, price);
    Ok(())
}

#[derive(Debug)]
struct TradeError {
    kind: String,
    message: String,
    backtrace: Backtrace,
}

impl TradeError {
    fn new(kind: &str, message: String) -> Self {
        TradeError {
            kind: kind.to_string(),
            message,
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}\n\nBacktrace:\n{}", self.kind, self.message, self.backtrace)
    }
}

fn main() {
    // Нужно установить переменную для захвата backtrace
    std::env::set_var("RUST_BACKTRACE", "1");

    match execute_trade("BTC", -0.5, 42000.0) {
        Ok(()) => println!("Trade executed successfully"),
        Err(e) => eprintln!("Trade failed:\n{}", e),
    }
}
```

## Логирование краша торговой системы

```rust
use std::panic;
use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;

fn setup_crash_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::capture();
        let timestamp = chrono_lite_timestamp();

        let crash_report = format!(
            "=== TRADING SYSTEM CRASH REPORT ===\n\
             Timestamp: {}\n\
             Panic: {}\n\
             \n\
             Backtrace:\n{}\n\
             ================================\n\n",
            timestamp,
            panic_info,
            backtrace
        );

        // Записываем в лог-файл
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash_log.txt")
        {
            let _ = file.write_all(crash_report.as_bytes());
        }

        // Также выводим в stderr
        eprintln!("{}", crash_report);
    }));
}

fn chrono_lite_timestamp() -> String {
    // Упрощённая версия — в реальном коде используйте chrono
    "2024-01-15 14:30:45 UTC".to_string()
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    setup_crash_handler();

    // Торговая логика
    run_trading_bot();
}

fn run_trading_bot() {
    let prices: Vec<f64> = vec![42000.0, 42500.0];

    // Симулируем ошибку
    for i in 0..5 {
        let price = prices[i]; // Паника на i=2!
        println!("Processing price: {}", price);
    }
}
```

## Анализ Backtrace: практические советы

### 1. Читайте снизу вверх
Backtrace показывает стек от места паники до main. Читайте снизу вверх, чтобы понять последовательность вызовов.

### 2. Фильтруйте шум
В полном backtrace много внутренних функций Rust. Ищите строки с путями к вашему коду (`src/`).

### 3. Обращайте внимание на номера строк
```
at ./src/trading/strategy.rs:142
```
Это точное место, где произошла ошибка.

### 4. Используйте для профилактики
Регулярно запускайте тесты с `RUST_BACKTRACE=1`, чтобы находить скрытые проблемы.

## Таблица команд

| Команда | Описание |
|---------|----------|
| `RUST_BACKTRACE=1` | Краткий backtrace |
| `RUST_BACKTRACE=full` | Полный backtrace |
| `Backtrace::capture()` | Программный захват |
| `panic::set_hook()` | Кастомный обработчик паники |

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|------------------------|
| Backtrace | Стек вызовов при панике | Отладка падений бота |
| RUST_BACKTRACE=1 | Переменная окружения | Быстрая диагностика |
| Backtrace::capture() | Программный захват | Логирование ошибок |
| panic::set_hook() | Кастомный обработчик | Запись crash-логов |

## Практические упражнения

### Упражнение 1: Отладка стратегии
Напишите торговую стратегию с намеренной ошибкой (деление на ноль при расчёте среднего). Используйте backtrace для локализации проблемы.

### Упражнение 2: Crash Reporter
Создайте систему логирования крашей, которая:
- Сохраняет backtrace в файл
- Включает информацию о последних сделках
- Отправляет уведомление (симуляция)

### Упражнение 3: Defensive Trading
Перепишите код с паниками на Result, сохраняя возможность получения backtrace при ошибках.

### Упражнение 4: Анализ реального краша
Создайте многопоточный торговый симулятор, который иногда падает. Используйте backtrace для определения race condition.

## Домашнее задание

1. **Crash Logger**: Создайте полноценную систему логирования крашей для торгового бота с ротацией логов и уровнями детализации

2. **Error Context**: Реализуйте структуру ошибки, которая автоматически захватывает backtrace и контекст (текущая позиция, баланс, последние сделки)

3. **Debug Mode**: Создайте торгового бота с режимом отладки, который при ошибке:
   - Сохраняет backtrace
   - Выводит состояние всех переменных
   - Создаёт snapshot для воспроизведения проблемы

4. **Post-mortem Analyzer**: Напишите утилиту, которая читает crash-логи и создаёт статистику: какие функции чаще всего падают, в какое время суток, при каких условиях

## Навигация

[← Предыдущий день](../092-panic-macro/ru.md) | [Следующий день →](../094-custom-error-types/ru.md)
