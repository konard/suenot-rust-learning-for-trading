# День 315: Компиляция в release mode

## Аналогия из трейдинга

Представь, что ты разработал торгового бота, который анализирует тысячи свечей в реальном времени. В режиме разработки (debug mode) твой бот работает медленно — это как торговать на симуляторе с задержками: удобно для отладки, но непригодно для реальных торгов.

**Release mode** — это переход от симулятора к реальной бирже:

| Debug mode (разработка) | Release mode (продакшн) |
|-------------------------|-------------------------|
| Много проверок и отладочной информации | Максимальная скорость |
| Медленное исполнение | Оптимизированный код |
| Большой размер бинарника | Компактный бинарник |
| Детальная информация об ошибках | Минимальные накладные расходы |

Это как разница между тестовым запуском стратегии на исторических данных (где скорость не критична) и реальной торговлей, где каждая миллисекунда может стоить денег.

## Что такое Release Mode?

**Release mode** — это режим компиляции в Rust, который включает оптимизации компилятора для максимальной производительности.

### Сравнение режимов компиляции

```bash
# Debug mode (по умолчанию)
cargo build

# Release mode (с оптимизациями)
cargo build --release
```

### Ключевые различия

| Аспект | Debug | Release |
|--------|-------|---------|
| **Оптимизация** | Минимальная (opt-level = 0) | Максимальная (opt-level = 3) |
| **Время компиляции** | Быстрее | Медленнее |
| **Скорость выполнения** | Медленная | До 10-100x быстрее |
| **Размер бинарника** | Больше | Меньше |
| **Отладочная информация** | Полная | Отсутствует |
| **Проверки границ** | Полные | Могут быть оптимизированы |
| **Панические сообщения** | Подробные | Минимальные |

## Практический пример: Расчёт SMA

Давай измерим разницу в производительности на реальной задаче — расчёт скользящей средней для большого объёма данных:

```rust
use std::time::Instant;

/// Расчёт Simple Moving Average
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    // Первое значение SMA
    let first_sum: f64 = prices[..period].iter().sum();
    sma_values.push(first_sum / period as f64);

    // Оптимизированный скользящий расчёт
    for i in period..prices.len() {
        let prev_sma = sma_values.last().unwrap();
        let new_sma = prev_sma + (prices[i] - prices[i - period]) / period as f64;
        sma_values.push(new_sma);
    }

    sma_values
}

/// Генерация тестовых данных (имитация цен BTC)
fn generate_price_data(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 42000.0;

    for i in 0..count {
        // Имитация случайного движения цены
        let change = ((i * 17 + 13) % 200) as f64 / 100.0 - 1.0;
        price += change * 50.0;
        prices.push(price);
    }

    prices
}

fn main() {
    let data_sizes = [1_000, 10_000, 100_000, 1_000_000];
    let period = 20;

    println!("=== Бенчмарк расчёта SMA(20) ===\n");
    println!("Режим компиляции: {}",
             if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });
    println!();

    for &size in &data_sizes {
        let prices = generate_price_data(size);

        let start = Instant::now();
        let sma = calculate_sma(&prices, period);
        let elapsed = start.elapsed();

        println!("Данные: {:>10} точек", size);
        println!("  Время: {:>10.3} мс", elapsed.as_secs_f64() * 1000.0);
        println!("  SMA значений: {}", sma.len());
        println!("  Последнее SMA: ${:.2}", sma.last().unwrap_or(&0.0));
        println!();
    }
}
```

### Результаты сравнения

Запустите этот код в обоих режимах:

```bash
# Debug mode
cargo run

# Release mode
cargo run --release
```

Типичные результаты:

| Данные | Debug | Release | Ускорение |
|--------|-------|---------|-----------|
| 1,000 | 0.5 мс | 0.02 мс | 25x |
| 10,000 | 5 мс | 0.15 мс | 33x |
| 100,000 | 50 мс | 1.2 мс | 42x |
| 1,000,000 | 500 мс | 10 мс | 50x |

## Настройка профилей в Cargo.toml

Rust позволяет тонко настраивать параметры компиляции через `Cargo.toml`:

```toml
[package]
name = "trading-bot"
version = "1.0.0"

# Настройки для debug сборки
[profile.dev]
opt-level = 0          # Без оптимизаций (быстрая компиляция)
debug = true           # Полная отладочная информация
overflow-checks = true # Проверки переполнения

# Настройки для release сборки
[profile.release]
opt-level = 3          # Максимальные оптимизации
lto = true             # Link-Time Optimization
codegen-units = 1      # Один кодген для лучшей оптимизации
panic = "abort"        # Abort вместо unwind (меньший бинарник)
strip = true           # Удаление символов отладки

# Кастомный профиль для бенчмарков
[profile.bench]
opt-level = 3
lto = true
debug = false

# Профиль для тестирования с оптимизациями
[profile.test-release]
inherits = "release"
debug = true           # Отладочная информация для профилирования
```

### Уровни оптимизации

| opt-level | Описание | Применение |
|-----------|----------|------------|
| 0 | Без оптимизаций | Быстрая разработка |
| 1 | Базовые оптимизации | Баланс скорость/размер |
| 2 | Большинство оптимизаций | Хороший компромисс |
| 3 | Все оптимизации | Максимальная скорость |
| "s" | Оптимизация размера | Встраиваемые системы |
| "z" | Минимальный размер | Критичен размер |

## Link-Time Optimization (LTO)

LTO — это мощная техника оптимизации, которая анализирует весь код проекта на этапе линковки:

```toml
[profile.release]
lto = true          # Полная LTO (самая медленная компиляция, лучший результат)
# lto = "thin"      # Thin LTO (быстрее компиляция, немного хуже результат)
# lto = false       # Без LTO (по умолчанию)
```

### Пример: оптимизация торгового расчёта

```rust
/// Расчёт максимальной просадки (Maximum Drawdown)
/// LTO позволяет инлайнить эту функцию в вызывающий код
#[inline(always)]
fn calculate_running_max(prices: &[f64]) -> Vec<f64> {
    let mut running_max = Vec::with_capacity(prices.len());
    let mut max = f64::MIN;

    for &price in prices {
        if price > max {
            max = price;
        }
        running_max.push(max);
    }

    running_max
}

/// Расчёт просадки от пика
#[inline(always)]
fn calculate_drawdown(prices: &[f64], running_max: &[f64]) -> Vec<f64> {
    prices.iter()
        .zip(running_max.iter())
        .map(|(price, max)| (price - max) / max * 100.0)
        .collect()
}

/// Максимальная просадка портфеля
pub fn max_drawdown(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    let running_max = calculate_running_max(prices);
    let drawdowns = calculate_drawdown(prices, &running_max);

    drawdowns.iter()
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0)
}

fn main() {
    // Имитация equity curve торговой стратегии
    let equity: Vec<f64> = vec![
        10000.0, 10200.0, 10500.0, 10300.0, 10100.0,
        9800.0, 9500.0, 9700.0, 10000.0, 10500.0,
        11000.0, 10800.0, 10600.0, 11200.0, 11500.0,
    ];

    let mdd = max_drawdown(&equity);
    println!("Максимальная просадка: {:.2}%", mdd);
}
```

## Условная компиляция: Debug vs Release

Rust позволяет писать разный код для debug и release режимов:

```rust
/// Структура для логирования сделок
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: TradeSide,
}

#[derive(Debug)]
enum TradeSide {
    Buy,
    Sell,
}

impl Trade {
    fn execute(&self) {
        // Детальное логирование только в debug режиме
        #[cfg(debug_assertions)]
        println!(
            "[DEBUG] Исполнение сделки: {} {} {} @ ${:.2}",
            match self.side { TradeSide::Buy => "BUY", TradeSide::Sell => "SELL" },
            self.quantity,
            self.symbol,
            self.price
        );

        // Реальная логика исполнения
        self.send_to_exchange();

        // Верификация только в debug
        #[cfg(debug_assertions)]
        self.verify_execution();
    }

    fn send_to_exchange(&self) {
        // Отправка на биржу
        println!("Ордер отправлен: {} {}", self.symbol, self.quantity);
    }

    #[cfg(debug_assertions)]
    fn verify_execution(&self) {
        println!("[DEBUG] Верификация исполнения...");
        // Дополнительные проверки для разработки
    }
}

/// Функция проверки, доступная только в debug режиме
#[cfg(debug_assertions)]
fn debug_assert_valid_price(price: f64) {
    assert!(price > 0.0, "Цена должна быть положительной!");
    assert!(price < 1_000_000.0, "Цена слишком высокая!");
}

/// Расчёт позиции с дополнительными проверками в debug
fn calculate_position_size(capital: f64, price: f64, risk_percent: f64) -> f64 {
    #[cfg(debug_assertions)]
    debug_assert_valid_price(price);

    let position_value = capital * risk_percent / 100.0;
    position_value / price
}

fn main() {
    println!("Режим: {}",
             if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" });

    let trade = Trade {
        symbol: "BTC/USD".to_string(),
        price: 42500.0,
        quantity: 0.5,
        side: TradeSide::Buy,
    };

    trade.execute();

    let position = calculate_position_size(10000.0, 42500.0, 2.0);
    println!("Размер позиции: {} BTC", position);
}
```

## Оптимизация для конкретной платформы

Для максимальной производительности можно указать целевой процессор:

```bash
# Использовать все возможности текущего CPU
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Указать конкретную архитектуру
RUSTFLAGS="-C target-cpu=skylake" cargo build --release
```

### Cargo.toml для кросс-компиляции

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-cpu=native"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "target-cpu=native"]
```

## Измерение производительности: Бенчмарки

Для корректного измерения производительности используйте встроенные бенчмарки:

```rust
#![feature(test)]

extern crate test;

use test::Bencher;

/// Наивная реализация SMA
fn sma_naive(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..=prices.len() - period {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

/// Оптимизированная реализация SMA
fn sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return Vec::new();
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

#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    fn generate_prices(n: usize) -> Vec<f64> {
        (0..n).map(|i| 42000.0 + (i as f64) * 0.1).collect()
    }

    #[bench]
    fn bench_sma_naive_1000(b: &mut Bencher) {
        let prices = generate_prices(1000);
        b.iter(|| sma_naive(&prices, 20));
    }

    #[bench]
    fn bench_sma_optimized_1000(b: &mut Bencher) {
        let prices = generate_prices(1000);
        b.iter(|| sma_optimized(&prices, 20));
    }

    #[bench]
    fn bench_sma_naive_10000(b: &mut Bencher) {
        let prices = generate_prices(10000);
        b.iter(|| sma_naive(&prices, 20));
    }

    #[bench]
    fn bench_sma_optimized_10000(b: &mut Bencher) {
        let prices = generate_prices(10000);
        b.iter(|| sma_optimized(&prices, 20));
    }
}

fn main() {
    println!("Запустите бенчмарки командой: cargo bench");
}
```

Запуск бенчмарков:

```bash
cargo bench
```

## Уменьшение размера бинарника

Для продакшн-деплоя важен размер бинарника:

```toml
[profile.release]
opt-level = "z"        # Оптимизация размера
lto = true             # LTO также уменьшает размер
codegen-units = 1      # Лучшая оптимизация
panic = "abort"        # Убираем код для unwind
strip = true           # Удаляем символы отладки
```

### Дополнительные шаги

```bash
# Сборка с минимальным размером
cargo build --release

# Проверка размера
ls -lh target/release/trading-bot

# Дополнительное сжатие (Linux)
strip target/release/trading-bot
upx --best target/release/trading-bot
```

## Отладка в Release Mode

Иногда нужно отладить проблему, которая проявляется только в release:

```toml
# Профиль для отладки release
[profile.release-with-debug]
inherits = "release"
debug = true           # Добавляем отладочную информацию
```

```bash
# Сборка с этим профилем
cargo build --profile release-with-debug
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Debug mode** | Режим для разработки с проверками и отладочной информацией |
| **Release mode** | Оптимизированный режим для продакшна |
| **opt-level** | Уровень оптимизации (0-3, s, z) |
| **LTO** | Link-Time Optimization для межмодульной оптимизации |
| **codegen-units** | Количество параллельных единиц кодогенерации |
| **panic = "abort"** | Упрощённая обработка паники для меньшего бинарника |
| **cfg!(debug_assertions)** | Условная компиляция для разных режимов |
| **target-cpu=native** | Оптимизация под конкретный процессор |

## Практические задания

1. **Бенчмарк стратегии**: Создай торговую стратегию (например, пересечение SMA) и измерь разницу производительности между debug и release режимами на 1 миллионе свечей.

2. **Профиль для тестирования**: Создай кастомный профиль в `Cargo.toml` для тестирования с частичными оптимизациями (opt-level = 1) и отладочной информацией.

3. **Условная компиляция**: Добавь в торгового бота детальное логирование всех операций, которое работает только в debug режиме, и измерь разницу в производительности.

4. **Оптимизация размера**: Собери торгового бота с минимальным размером бинарника и сравни размер с обычной release сборкой. Измерь влияние на производительность.

## Домашнее задание

1. **Trading Bot Benchmark Suite**: Создай набор бенчмарков для основных операций торгового бота:
   - Расчёт индикаторов (SMA, EMA, RSI, MACD)
   - Парсинг рыночных данных
   - Сериализация/десериализация ордеров
   - Расчёт рисков портфеля

   Сравни результаты debug vs release.

2. **Профили для разных сценариев**: Настрой разные профили компиляции для:
   - Разработка (быстрая компиляция)
   - Тестирование (оптимизации + отладка)
   - Staging (почти продакшн)
   - Продакшн (максимальная производительность)

   Документируй время компиляции и размер бинарника для каждого.

3. **Cross-Platform Optimization**: Настрой кросс-компиляцию торгового бота для:
   - Linux (сервер)
   - macOS (разработка)
   - Windows (клиенты)

   С оптимизациями под конкретные процессоры.

4. **Performance Regression Testing**: Создай систему автоматического тестирования производительности:
   - Бенчмарки критических путей
   - Сравнение с базовыми показателями
   - Автоматические предупреждения при деградации производительности

   Интегрируй в CI/CD пайплайн.

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../316-*/ru.md)
