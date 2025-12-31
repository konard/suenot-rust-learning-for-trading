# День 318: Размер бинарника: уменьшаем

## Аналогия из трейдинга

Представь, что ты разворачиваешь торгового бота на маленьком VPS-сервере или встроенном устройстве в удалённой локации (например, на колокейшен-сервере рядом с биржей). Каждый мегабайт твоего бинарника стоит денег:

- **Затраты на хранение**: Облачные провайдеры берут плату за дисковое пространство
- **Время деплоя**: Большие бинарники дольше передаются по сети
- **Накладные расходы памяти**: Большие бинарники часто потребляют больше RAM во время работы
- **Время запуска**: Загрузка бинарника в 100 МБ медленнее, чем в 10 МБ

Это похоже на то, как трейдеры оптимизируют свои портфели:
- **Лёгкие портфели**: Держать только те активы, которые служат цели
- **Эффективное использование капитала**: Не замораживать деньги в ненужных позициях
- **Быстрое исполнение**: Меньшие, сфокусированные позиции можно быстрее корректировать

В высокочастотном трейдинге каждая миллисекунда на счету. Раздутый бинарник может означать разницу между захватом и упущением сделки. При деплое на граничные устройства или в контейнеры размер напрямую влияет на затраты и производительность.

## Почему бинарники Rust могут быть большими

По умолчанию Rust приоритезирует:
- **Отладочную информацию** для разработки
- **Мономорфизацию** (генерация специализированного кода для каждого generic типа)
- **Статическую линковку** (включение всех зависимостей в один бинарник)
- **Обработку паники** с полными stack trace

Это приводит к бо́льшим, но более быстрым и удобным для отладки бинарникам. Для продакшен торговых систем нам часто нужно балансировать эти компромиссы.

## Базовые техники уменьшения размера

### 1. Компиляция в режиме Release

Самая фундаментальная оптимизация:

```bash
# Debug сборка (по умолчанию) - большая, медленная, отлаживаемая
cargo build

# Release сборка - меньше, быстрее, оптимизирована
cargo build --release
```

```rust
fn main() {
    let prices = vec![100.0, 105.0, 102.0, 108.0, 110.0];

    // Этот код оптимизируется по-разному в debug и release
    let sma: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

    println!("SMA(5): ${:.2}", sma);
}
```

**Типичная разница в размере:**
- Debug: ~10-50 МБ
- Release: ~2-10 МБ

### 2. Настройки оптимизации в Cargo.toml

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[profile.release]
# Уровень оптимизации (0-3, "s" для размера, "z" для минимального размера)
opt-level = "z"         # Оптимизация по размеру

# Link-Time Optimization - позволяет оптимизацию через границы crate
lto = true              # Включить LTO

# Уменьшить параллельные единицы кодогенерации (лучшая оптимизация, медленнее компиляция)
codegen-units = 1       # Лучшая оптимизация

# Удалить символы из бинарника
strip = true            # Удалить отладочные символы

# Обработка паники - "abort" меньше чем "unwind"
panic = "abort"         # Меньший размер обработки паники

[dependencies]
# Использовать минимальные наборы фич
serde = { version = "1.0", default-features = false, features = ["derive"] }
```

### 3. Понимание opt-level

```toml
[profile.release]
# Значения opt-level:
# 0 = без оптимизации (быстрая компиляция, большой бинарник)
# 1 = базовая оптимизация
# 2 = умеренная оптимизация (по умолчанию для release)
# 3 = агрессивная оптимизация (может увеличить размер!)
# "s" = оптимизация по размеру
# "z" = оптимизация для минимального размера (самая агрессивная)
opt-level = "z"
```

## Пример: оптимизация размера торгового бота

Давай создадим простого торгового бота и оптимизируем его размер:

```rust
use std::collections::HashMap;

/// Простой трекер цен для мониторинга портфеля
struct PortfolioTracker {
    positions: HashMap<String, Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl PortfolioTracker {
    fn new() -> Self {
        PortfolioTracker {
            positions: HashMap::new(),
        }
    }

    fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        let position = Position {
            symbol: symbol.to_string(),
            quantity,
            avg_price: price,
        };
        self.positions.insert(symbol.to_string(), position);
    }

    fn get_total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.values().map(|pos| {
            let current_price = current_prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * current_price
        }).sum()
    }

    fn calculate_pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions.values().map(|pos| {
            let current_price = current_prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * (current_price - pos.avg_price)
        }).sum()
    }
}

fn main() {
    let mut tracker = PortfolioTracker::new();

    // Добавляем позиции
    tracker.add_position("BTCUSD", 0.5, 45000.0);
    tracker.add_position("ETHUSD", 2.0, 2800.0);
    tracker.add_position("SOLUSD", 10.0, 95.0);

    // Текущие рыночные цены
    let mut prices = HashMap::new();
    prices.insert("BTCUSD".to_string(), 48000.0);
    prices.insert("ETHUSD".to_string(), 3100.0);
    prices.insert("SOLUSD".to_string(), 105.0);

    let total_value = tracker.get_total_value(&prices);
    let pnl = tracker.calculate_pnl(&prices);

    println!("=== Сводка по портфелю ===");
    println!("Общая стоимость: ${:.2}", total_value);
    println!("Нереализованный PnL: ${:+.2}", pnl);
}
```

## Link-Time Optimization (LTO)

LTO позволяет компилятору оптимизировать через границы crate:

```toml
[profile.release]
# Варианты LTO:
# false = без LTO (быстрая компиляция)
# true = полный LTO (медленная компиляция, лучшая оптимизация)
# "thin" = тонкий LTO (баланс между скоростью и оптимизацией)
# "fat" = то же что true
lto = true
```

### Как LTO помогает торговым системам:

```rust
// crate: indicators
pub fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let sum: f64 = prices.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

// crate: trading-bot (использует indicators)
fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];

    // С LTO компилятор может встроить (inline) calculate_sma
    // и оптимизировать всю цепочку вызовов
    if let Some(sma) = calculate_sma(&prices, 3) {
        println!("SMA(3): {:.2}", sma);
    }
}
```

Без LTO компилятор не может оптимизировать через границы crate. С включённым LTO он может встраивать и оптимизировать всю программу как единое целое.

## Удаление символов и отладочной информации

Отладочные символы добавляют значительный размер, но необходимы для отладки:

```toml
[profile.release]
# Варианты strip:
# "none" = сохранить все символы
# "debuginfo" = удалить отладочную информацию, сохранить имена символов
# "symbols" = удалить всё
strip = "symbols"
```

### Альтернатива: использование команды strip

```bash
# Собрать release бинарник
cargo build --release

# Проверить размер до strip
ls -lh target/release/trading-bot

# Удалить символы вручную (если не используется Cargo strip)
strip target/release/trading-bot

# Проверить размер после strip
ls -lh target/release/trading-bot
```

## Стратегия паники: Abort vs Unwind

```toml
[profile.release]
# Поведение при панике:
# "unwind" = раскрутка стека, позволяет catch_unwind, больший бинарник
# "abort" = немедленное завершение, меньший бинарник
panic = "abort"
```

### Компромиссы для торговых систем:

```rust
use std::panic;

fn risky_calculation(data: &[f64]) -> f64 {
    // С panic = "unwind" мы можем ловить паники
    let result = panic::catch_unwind(|| {
        data.iter().sum::<f64>() / data.len() as f64
    });

    match result {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Вычисление не удалось, возвращаем значение по умолчанию");
            0.0
        }
    }
}

fn main() {
    let prices: Vec<f64> = vec![];

    // С panic = "abort" любая паника немедленно завершает программу
    // Это часто приемлемо для торговых ботов, которые должны
    // перезапускаться чисто, а не продолжать в неопределённом состоянии
    let avg = risky_calculation(&prices);
    println!("Среднее: {}", avg);
}
```

**Рекомендация для торговых систем:** Используйте `panic = "abort"` в продакшене. Лучше, если торговый бот упадёт и перезапустится чисто, чем продолжит работу в потенциально повреждённом состоянии.

## Feature Flags: используем только нужное

Многие crate имеют опциональные фичи. Включайте только то, что нужно:

```toml
[dependencies]
# Полный serde (большой)
# serde = "1.0"

# Минимальный serde (меньше)
serde = { version = "1.0", default-features = false, features = ["derive"] }

# Полный tokio (большой)
# tokio = { version = "1.0", features = ["full"] }

# Минимальный tokio для торгового бота (меньше)
tokio = { version = "1.0", features = ["rt", "net", "time"] }

# reqwest без фич по умолчанию
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }
```

### Пример: минимальный HTTP клиент для получения цен

```rust
use std::collections::HashMap;

// Используем reqwest с минимальными фичами
// reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }

async fn fetch_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Имитация ответа API
    let mock_prices: HashMap<&str, f64> = [
        ("BTCUSD", 48000.0),
        ("ETHUSD", 3100.0),
    ].iter().cloned().collect();

    mock_prices.get(symbol)
        .copied()
        .ok_or_else(|| format!("Цена не найдена для {}", symbol).into())
}

#[tokio::main]
async fn main() {
    match fetch_price("BTCUSD").await {
        Ok(price) => println!("Цена BTC: ${:.2}", price),
        Err(e) => eprintln!("Ошибка: {}", e),
    }
}
```

## Codegen Units

Уменьшение codegen units позволяет лучшую оптимизацию, но увеличивает время компиляции:

```toml
[profile.release]
# По умолчанию 16, что позволяет параллельную компиляцию
# Установка в 1 включает лучшую оптимизацию
codegen-units = 1
```

## Продвинутое: UPX сжатие

UPX (Ultimate Packer for eXecutables) может дополнительно сжать бинарники:

```bash
# Установка UPX
# Ubuntu/Debian: sudo apt install upx
# macOS: brew install upx

# Сжать бинарник
upx --best target/release/trading-bot

# Или с агрессивным сжатием (медленнее распаковка)
upx --ultra-brute target/release/trading-bot
```

**Примечание:** UPX добавляет небольшие накладные расходы при распаковке во время запуска. Для HFT систем, где важно время запуска, проверьте компромисс.

## Инструменты анализа размера

### Использование cargo-bloat

```bash
# Установить cargo-bloat
cargo install cargo-bloat

# Анализ размера бинарника по функциям
cargo bloat --release -n 20

# Анализ по crate
cargo bloat --release --crates
```

### Пример вывода

```
File  .text     Size Crate
0.8%   4.5%  43.5KiB std
0.5%   2.8%  27.1KiB serde_json
0.3%   1.7%  16.4KiB regex
0.2%   1.2%  11.5KiB trading_bot
...
```

### Использование cargo-size

```bash
# Проверить размер бинарника
cargo size --release

# Детальный анализ секций
cargo size --release -- -A
```

## Полный профиль оптимизации для торговых систем

```toml
[package]
name = "high-performance-trading-bot"
version = "1.0.0"
edition = "2021"

# Оптимизировать зависимости в debug сборках тоже
[profile.dev.package."*"]
opt-level = 2

[profile.release]
# Оптимизация по размеру (используйте "3" если скорость важнее размера)
opt-level = "z"

# Link-Time Optimization
lto = true

# Одна единица кодогенерации для лучшей оптимизации
codegen-units = 1

# Удалить все символы
strip = "symbols"

# Abort при панике (меньше, чище для торговых систем)
panic = "abort"

# Уменьшить отладочную информацию в release
debug = false

# Инкрементальная компиляция выключена для release (лучшая оптимизация)
incremental = false

[dependencies]
# Использовать минимальные наборы фич
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false, features = ["std"] }
tokio = { version = "1.0", default-features = false, features = ["rt", "macros", "time"] }
```

## Измерение влияния

Вот скрипт для измерения влияния оптимизации:

```rust
//! Скрипт сборки для отчёта о размере бинарника
//! Сохранить как: build_and_measure.rs

use std::process::Command;
use std::fs;

fn get_file_size(path: &str) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.2} МБ", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} КБ", bytes as f64 / 1_000.0)
    } else {
        format!("{} байт", bytes)
    }
}

fn main() {
    println!("=== Отчёт об оптимизации размера бинарника ===\n");

    // Собрать debug
    println!("Сборка debug...");
    Command::new("cargo")
        .args(["build"])
        .status()
        .expect("Не удалось собрать debug");

    // Собрать release
    println!("Сборка release...");
    Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Не удалось собрать release");

    // Измерить размеры
    let debug_size = get_file_size("target/debug/trading-bot").unwrap_or(0);
    let release_size = get_file_size("target/release/trading-bot").unwrap_or(0);

    println!("\n=== Результаты ===");
    println!("Debug бинарник:   {}", format_size(debug_size));
    println!("Release бинарник: {}", format_size(release_size));

    if debug_size > 0 && release_size > 0 {
        let reduction = (1.0 - release_size as f64 / debug_size as f64) * 100.0;
        println!("Уменьшение размера: {:.1}%", reduction);
    }
}
```

## Что мы узнали

| Техника | Уменьшение размера | Компромисс |
|---------|-------------------|------------|
| Режим Release | 70-90% | Более долгое время компиляции |
| opt-level = "z" | 10-30% | Немного медленнее выполнение |
| LTO = true | 10-20% | Намного дольше компиляция |
| codegen-units = 1 | 5-15% | Дольше компиляция |
| strip = true | 20-50% | Нет отладочных символов |
| panic = "abort" | 5-10% | Нет восстановления после паники |
| Минимальные фичи | Варьируется | Может потерять функциональность |
| UPX сжатие | 30-60% | Накладные расходы при запуске |

## Практические задания

1. **Сравнение размеров**: Создай простого торгового бота и измерь размер бинарника с:
   - Debug сборкой по умолчанию
   - Release сборкой по умолчанию
   - Release с `opt-level = "z"`
   - Release с полным профилем оптимизации
   - После UPX сжатия

   Запиши все размеры и вычисли процентное уменьшение.

2. **Аудит фич**: Возьми существующий Rust проект и:
   - Выведи все зависимости с `cargo tree`
   - Определи какие фичи можно отключить
   - Создай минимальный набор фич
   - Измерь уменьшение размера

3. **Анализ компромиссов**: Собери калькулятор торговых сигналов с:
   - Полным профилем оптимизации (минимальный размер)
   - Профилем производительности (opt-level = 3)
   - Измерь и сравни:
     - Размер бинарника
     - Время выполнения для 1 миллиона вычислений
     - Использование памяти

4. **Симуляция деплоя**: Создай Docker контейнер для торгового бота:
   - Один с неоптимизированным бинарником
   - Один с полностью оптимизированным бинарником
   - Сравни размеры контейнеров и время запуска

## Домашнее задание

1. **Оптимизатор торгового бота**: Создай CLI инструмент, который:
   - Анализирует файл Cargo.toml
   - Предлагает настройки оптимизации
   - Запускает сборки для сравнения размеров
   - Генерирует отчёт с рекомендациями
   - Включает специфичные советы для торговых приложений

2. **Дашборд размера бинарника**: Построй систему мониторинга, которая:
   - Отслеживает размер бинарника по git коммитам
   - Оповещает когда размер значительно увеличивается
   - Показывает разбивку размера по crate
   - Сравнивает разные профили оптимизации
   - Генерирует исторические графики размера

3. **Минимизатор фич**: Напиши инструмент, который:
   - Парсит зависимости Cargo.toml
   - Определяет неиспользуемые фичи через статический анализ
   - Предлагает минимальные наборы фич
   - Тестирует компиляцию с уменьшенными фичами
   - Отчитывается об экономии размера

4. **Пайплайн деплоя**: Создай CI/CD пайплайн, который:
   - Собирает с несколькими профилями оптимизации
   - Запускает бенчмарки производительности для каждого
   - Сравнивает размеры бинарников
   - Выбирает оптимальный профиль на основе ограничений (размер vs скорость)
   - Деплоит в разные окружения (HFT = скорость, edge = размер)

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../319-*/ru.md)
