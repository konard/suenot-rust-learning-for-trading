# День 317: PGO: Profile Guided Optimization

## Аналогия из трейдинга

Представь, что ты разрабатываешь высокочастотную торговую систему (HFT). В обычной разработке ты оптимизируешь код "вслепую" — догадываясь, какие функции будут вызываться чаще всего. Это как пытаться оптимизировать торговую стратегию без реальных рыночных данных.

**Profile Guided Optimization (PGO)** — это как бэктестинг для компилятора:
1. **Сначала собираешь профиль** — запускаешь программу с реальными торговыми данными, компилятор записывает, какие ветки кода выполняются чаще
2. **Потом оптимизируешь на основе данных** — компилятор перестраивает код, зная реальные паттерны выполнения

Это аналогично тому, как опытный трейдер:
- Сначала анализирует историю сделок (профилирование)
- Потом оптимизирует стратегию на основе реальных данных (PGO-компиляция)
- Получает лучший результат, чем при оптимизации "вслепую"

В контексте трейдинга PGO особенно важен для:
- Высокочастотных систем, где каждая микросекунда имеет значение
- Парсеров рыночных данных с миллионами сообщений в секунду
- Расчёта индикаторов в реальном времени
- Систем риск-менеджмента с критичной латентностью

## Что такое PGO?

**Profile Guided Optimization (PGO)** — это техника оптимизации, при которой компилятор использует информацию о реальном поведении программы для принятия более эффективных решений.

### Как это работает?

```
┌─────────────────────────────────────────────────────────────────┐
│                    Процесс PGO                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Обычная компиляция    2. Профилирование    3. PGO-сборка   │
│  ┌───────────────────┐    ┌─────────────────┐  ┌─────────────┐ │
│  │  Исходный код     │───▶│ Запуск с        │─▶│ Финальная   │ │
│  │  + инструменты    │    │ реальными       │  │ оптимизация │ │
│  │  профилирования   │    │ данными         │  │ на основе   │ │
│  └───────────────────┘    └─────────────────┘  │ профиля     │ │
│         │                        │             └─────────────┘ │
│         ▼                        ▼                    │        │
│  ┌───────────────────┐    ┌─────────────────┐         │        │
│  │ Инструментиро-    │    │ Файл профиля    │─────────┘        │
│  │ ванный бинарник   │    │ (.profdata)     │                  │
│  └───────────────────┘    └─────────────────┘                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Какие оптимизации делает PGO?

| Оптимизация | Описание | Влияние на трейдинг |
|-------------|----------|---------------------|
| **Inlining** | Встраивание часто вызываемых функций | Быстрее расчёт индикаторов |
| **Branch prediction** | Оптимизация условных переходов | Быстрее обработка ордеров |
| **Code layout** | Размещение горячего кода рядом | Меньше cache misses |
| **Register allocation** | Лучшее распределение регистров | Быстрее вычисления |
| **Virtual call speculation** | Оптимизация виртуальных вызовов | Быстрее полиморфный код |

## Включение PGO в Rust

### Шаг 1: Компиляция с инструментами профилирования

```bash
# Создаём папку для профилей
mkdir -p /tmp/pgo-data

# Компилируем с инструментами профилирования
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release --target=x86_64-unknown-linux-gnu
```

### Шаг 2: Запуск программы с реальными данными

```bash
# Запускаем программу с типичными торговыми данными
./target/release/trading-bot --data historical_btc_2024.csv

# Можно запустить несколько раз с разными сценариями
./target/release/trading-bot --data historical_eth_2024.csv
./target/release/trading-bot --data high_volatility_market.csv
```

### Шаг 3: Объединение профилей

```bash
# Устанавливаем llvm-tools, если ещё не установлены
rustup component add llvm-tools-preview

# Находим llvm-profdata
LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata | head -1)

# Объединяем все профили в один
$LLVM_PROFDATA merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data/
```

### Шаг 4: Финальная PGO-компиляция

```bash
# Компилируем с использованием профиля
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release --target=x86_64-unknown-linux-gnu
```

## Практический пример: оптимизация торговой системы

### Исходный код торговой системы

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Данные свечи OHLCV
#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Торговый сигнал
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Позиция
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    entry_time: i64,
}

/// Калькулятор индикаторов
struct IndicatorCalculator {
    sma_cache: HashMap<(String, usize), Vec<f64>>,
    ema_cache: HashMap<(String, usize), Vec<f64>>,
}

impl IndicatorCalculator {
    fn new() -> Self {
        Self {
            sma_cache: HashMap::new(),
            ema_cache: HashMap::new(),
        }
    }

    /// Расчёт SMA (Simple Moving Average)
    /// Эта функция вызывается очень часто — PGO оптимизирует её
    fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Расчёт EMA (Exponential Moving Average)
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for &price in &prices[1..] {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }

        Some(ema)
    }

    /// Расчёт RSI (Relative Strength Index)
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period {
            let change = prices[prices.len() - period - 1 + i] - prices[prices.len() - period - 2 + i];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Расчёт волатильности (стандартное отклонение)
    fn calculate_volatility(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let slice = &prices[prices.len() - period..];
        let mean: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / period as f64;

        Some(variance.sqrt())
    }
}

/// Торговая стратегия
struct TradingStrategy {
    calculator: IndicatorCalculator,
    fast_period: usize,
    slow_period: usize,
    rsi_period: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,
}

impl TradingStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            calculator: IndicatorCalculator::new(),
            fast_period,
            slow_period,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
        }
    }

    /// Генерация сигнала на основе индикаторов
    /// Это горячий путь — PGO оптимизирует ветвления
    fn generate_signal(&self, prices: &[f64]) -> Signal {
        // Проверка достаточности данных — часто true
        if prices.len() < self.slow_period {
            return Signal::Hold;
        }

        // Расчёт индикаторов
        let fast_sma = match self.calculator.calculate_sma(prices, self.fast_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        let slow_sma = match self.calculator.calculate_sma(prices, self.slow_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        let rsi = match self.calculator.calculate_rsi(prices, self.rsi_period) {
            Some(v) => v,
            None => return Signal::Hold,
        };

        // Логика стратегии — PGO оптимизирует на основе реальных паттернов
        // Если рынок чаще растёт, компилятор оптимизирует ветку Buy
        if fast_sma > slow_sma && rsi < self.rsi_oversold {
            Signal::Buy
        } else if fast_sma < slow_sma && rsi > self.rsi_overbought {
            Signal::Sell
        } else {
            Signal::Hold
        }
    }
}

/// Бэктестер для сбора профиля
struct Backtester {
    strategy: TradingStrategy,
    positions: Vec<Position>,
    equity_curve: Vec<f64>,
    initial_capital: f64,
    current_capital: f64,
}

impl Backtester {
    fn new(strategy: TradingStrategy, initial_capital: f64) -> Self {
        Self {
            strategy,
            positions: Vec::new(),
            equity_curve: vec![initial_capital],
            initial_capital,
            current_capital: initial_capital,
        }
    }

    /// Обработка свечи — вызывается миллионы раз
    fn process_candle(&mut self, candle: &Candle, price_history: &[f64]) {
        let signal = self.strategy.generate_signal(price_history);

        match signal {
            Signal::Buy if self.positions.is_empty() => {
                // Открываем позицию
                let quantity = self.current_capital * 0.95 / candle.close;
                self.positions.push(Position {
                    symbol: "BTC".to_string(),
                    quantity,
                    entry_price: candle.close,
                    entry_time: candle.timestamp,
                });
                self.current_capital *= 0.05; // Оставляем 5% на комиссии
            }
            Signal::Sell if !self.positions.is_empty() => {
                // Закрываем позицию
                if let Some(pos) = self.positions.pop() {
                    let pnl = (candle.close - pos.entry_price) * pos.quantity;
                    self.current_capital += pos.quantity * candle.close * 0.999; // -0.1% комиссия
                }
            }
            _ => {} // Hold — ничего не делаем
        }

        // Обновляем equity curve
        let total_equity = self.current_capital +
            self.positions.iter()
                .map(|p| p.quantity * candle.close)
                .sum::<f64>();
        self.equity_curve.push(total_equity);
    }

    /// Расчёт финальных метрик
    fn calculate_metrics(&self) -> BacktestMetrics {
        let final_equity = *self.equity_curve.last().unwrap_or(&self.initial_capital);
        let total_return = (final_equity / self.initial_capital - 1.0) * 100.0;

        // Расчёт максимальной просадки
        let mut max_drawdown = 0.0;
        let mut peak = self.initial_capital;
        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        BacktestMetrics {
            total_return,
            max_drawdown: max_drawdown * 100.0,
            final_equity,
        }
    }
}

#[derive(Debug)]
struct BacktestMetrics {
    total_return: f64,
    max_drawdown: f64,
    final_equity: f64,
}

/// Генерация тестовых данных для профилирования
fn generate_test_data(num_candles: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(num_candles);
    let mut price = 40000.0;

    for i in 0..num_candles {
        // Симуляция рыночной волатильности
        let change = ((i as f64 * 0.1).sin() * 0.02 + (i as f64 * 0.03).cos() * 0.015) * price;
        price += change;

        let high = price * 1.01;
        let low = price * 0.99;
        let open = price - change * 0.3;
        let close = price;

        candles.push(Candle {
            timestamp: 1704067200 + (i as i64 * 3600), // Почасовые свечи
            open,
            high,
            low,
            close,
            volume: 1000.0 + (i as f64 * 10.0).sin().abs() * 500.0,
        });
    }

    candles
}

fn main() {
    println!("=== PGO Benchmark: Торговая система ===\n");

    // Генерируем тестовые данные (как при реальной торговле)
    let num_candles = 100_000;
    println!("Генерация {} свечей для бэктеста...", num_candles);
    let candles = generate_test_data(num_candles);

    // Создаём стратегию
    let strategy = TradingStrategy::new(10, 50);
    let mut backtester = Backtester::new(strategy, 100_000.0);

    // Собираем историю цен
    let mut price_history: Vec<f64> = Vec::with_capacity(num_candles);

    // Бенчмарк
    let start = Instant::now();

    for candle in &candles {
        price_history.push(candle.close);
        backtester.process_candle(candle, &price_history);
    }

    let duration = start.elapsed();

    // Результаты
    let metrics = backtester.calculate_metrics();

    println!("\n=== Результаты бэктеста ===");
    println!("Время выполнения: {:?}", duration);
    println!("Свечей обработано: {}", num_candles);
    println!("Скорость: {:.2} свечей/сек", num_candles as f64 / duration.as_secs_f64());
    println!("\n=== Метрики стратегии ===");
    println!("Общая доходность: {:.2}%", metrics.total_return);
    println!("Макс. просадка: {:.2}%", metrics.max_drawdown);
    println!("Финальный капитал: ${:.2}", metrics.final_equity);
}
```

### Cargo.toml для проекта

```toml
[package]
name = "trading-pgo-example"
version = "0.1.0"
edition = "2021"

[dependencies]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

# Для PGO сборки создаём отдельный профиль
[profile.release-pgo]
inherits = "release"
# Дополнительные оптимизации для PGO
```

### Скрипт автоматизации PGO

```bash
#!/bin/bash
# pgo_build.sh - Скрипт для PGO-сборки торговой системы

set -e

PROJECT_DIR=$(pwd)
PGO_DATA_DIR="$PROJECT_DIR/pgo-data"
TARGET="x86_64-unknown-linux-gnu"

echo "=== PGO Build Script для торговой системы ==="
echo ""

# Шаг 1: Очистка
echo "[1/5] Очистка предыдущих данных..."
rm -rf "$PGO_DATA_DIR"
mkdir -p "$PGO_DATA_DIR"
cargo clean

# Шаг 2: Обычная сборка (для сравнения)
echo ""
echo "[2/5] Сборка обычной версии..."
cargo build --release --target=$TARGET 2>/dev/null
cp target/$TARGET/release/trading-pgo-example target/release/trading-normal

# Шаг 3: Сборка с инструментами профилирования
echo ""
echo "[3/5] Сборка с инструментами профилирования..."
RUSTFLAGS="-Cprofile-generate=$PGO_DATA_DIR" \
    cargo build --release --target=$TARGET 2>/dev/null

# Шаг 4: Сбор профиля
echo ""
echo "[4/5] Сбор профиля на тестовых данных..."
echo "  Запуск 1: Стандартные данные..."
./target/$TARGET/release/trading-pgo-example > /dev/null

echo "  Запуск 2: Высокая волатильность (другой seed)..."
./target/$TARGET/release/trading-pgo-example > /dev/null

echo "  Запуск 3: Низкая волатильность..."
./target/$TARGET/release/trading-pgo-example > /dev/null

# Объединяем профили
LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata | head -1)
if [ -z "$LLVM_PROFDATA" ]; then
    echo "Ошибка: llvm-profdata не найден"
    echo "Установите: rustup component add llvm-tools-preview"
    exit 1
fi

$LLVM_PROFDATA merge -o "$PGO_DATA_DIR/merged.profdata" "$PGO_DATA_DIR/"

# Шаг 5: Финальная PGO сборка
echo ""
echo "[5/5] Финальная PGO-оптимизированная сборка..."
cargo clean 2>/dev/null
RUSTFLAGS="-Cprofile-use=$PGO_DATA_DIR/merged.profdata" \
    cargo build --release --target=$TARGET 2>/dev/null
cp target/$TARGET/release/trading-pgo-example target/release/trading-pgo

# Сравнение
echo ""
echo "=== Сравнение производительности ==="
echo ""

echo "Обычная сборка:"
time ./target/release/trading-normal

echo ""
echo "PGO-оптимизированная сборка:"
time ./target/release/trading-pgo

echo ""
echo "=== Сборка завершена ==="
echo "Бинарники находятся в target/release/"
echo "  - trading-normal: обычная сборка"
echo "  - trading-pgo: PGO-оптимизированная сборка"
```

## Измерение влияния PGO

### Бенчмарк для сравнения

```rust
use std::time::{Duration, Instant};

/// Результат бенчмарка
struct BenchmarkResult {
    name: String,
    iterations: usize,
    total_time: Duration,
    avg_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("=== {} ===", self.name);
        println!("  Итераций: {}", self.iterations);
        println!("  Общее время: {:?}", self.total_time);
        println!("  Среднее: {:?}", self.avg_time);
        println!("  Мин: {:?}", self.min_time);
        println!("  Макс: {:?}", self.max_time);
    }
}

/// Запуск бенчмарка функции
fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    let mut times = Vec::with_capacity(iterations);

    // Прогрев
    for _ in 0..10 {
        f();
    }

    // Измерения
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }

    let total_time: Duration = times.iter().sum();
    let avg_time = total_time / iterations as u32;
    let min_time = *times.iter().min().unwrap();
    let max_time = *times.iter().max().unwrap();

    BenchmarkResult {
        name: name.to_string(),
        iterations,
        total_time,
        avg_time,
        min_time,
        max_time,
    }
}

fn main() {
    let prices: Vec<f64> = (0..10000)
        .map(|i| 40000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    let calculator = IndicatorCalculator::new();

    // Бенчмарк SMA
    let sma_result = benchmark("SMA(50)", 10000, || {
        let _ = calculator.calculate_sma(&prices, 50);
    });
    sma_result.print();

    println!();

    // Бенчмарк EMA
    let ema_result = benchmark("EMA(50)", 10000, || {
        let _ = calculator.calculate_ema(&prices, 50);
    });
    ema_result.print();

    println!();

    // Бенчмарк RSI
    let rsi_result = benchmark("RSI(14)", 10000, || {
        let _ = calculator.calculate_rsi(&prices, 14);
    });
    rsi_result.print();
}
```

## Ожидаемые результаты PGO

Типичные улучшения производительности с PGO:

| Компонент | Улучшение | Причина |
|-----------|-----------|---------|
| **Расчёт индикаторов** | 5-15% | Лучшее встраивание, предсказание веток |
| **Парсинг рыночных данных** | 10-25% | Оптимизация горячих путей парсинга |
| **Обработка ордеров** | 5-10% | Оптимизация условных переходов |
| **Общая пропускная способность** | 10-20% | Комбинированный эффект |

### Реальные примеры улучшений

```
=== Результаты без PGO ===
Время выполнения: 1.234s
Скорость: 81,037 свечей/сек

=== Результаты с PGO ===
Время выполнения: 1.052s
Скорость: 95,057 свечей/сек

Улучшение: 17.3%
```

## Продвинутые техники PGO

### 1. Инструментирование только критичных функций

```rust
/// Атрибут для принудительного встраивания горячих функций
#[inline(always)]
fn hot_path_calculation(price: f64, factor: f64) -> f64 {
    // Критичный код, который должен быть встроен
    price * factor * 1.001
}

/// Атрибут для предотвращения встраивания редких функций
#[inline(never)]
fn cold_path_error_handling(error: &str) {
    // Редко вызываемый код обработки ошибок
    eprintln!("Error: {}", error);
}
```

### 2. Подсказки компилятору о вероятности веток

```rust
use std::hint::black_box;

/// Функция с подсказками о вероятности
fn process_market_data(data: &[u8]) -> Result<f64, &'static str> {
    // Проверка валидности — обычно успешна
    if data.is_empty() {
        // unlikely! макрос (в nightly Rust)
        return Err("Empty data");
    }

    // Основная логика — hot path
    let price = parse_price(data);

    // Проверка аномалий — редко срабатывает
    if price < 0.0 || price > 1_000_000.0 {
        return Err("Invalid price");
    }

    Ok(price)
}

fn parse_price(data: &[u8]) -> f64 {
    // Парсинг цены из байтов
    // PGO оптимизирует этот код на основе реальных данных
    42000.0 // Заглушка
}
```

### 3. BOLT — оптимизация после линковки

BOLT (Binary Optimization and Layout Tool) может дополнительно оптимизировать бинарник после PGO:

```bash
# Сборка с сохранением символов для BOLT
RUSTFLAGS="-Clink-arg=-Wl,--emit-relocs" cargo build --release

# Сбор профиля с perf
perf record -e cycles:u -o perf.data ./target/release/trading-bot

# Преобразование в формат BOLT
perf2bolt -p perf.data ./target/release/trading-bot -o perf.fdata

# Оптимизация с BOLT
llvm-bolt ./target/release/trading-bot -o ./target/release/trading-bot-bolt \
    -data=perf.fdata -reorder-blocks=ext-tsp -reorder-functions=hfsort
```

## Автоматизация PGO в CI/CD

### GitHub Actions workflow

```yaml
name: PGO Build

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  pgo-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-unknown-linux-gnu
          components: llvm-tools-preview

      - name: Create PGO data directory
        run: mkdir -p pgo-data

      - name: Build instrumented binary
        run: |
          RUSTFLAGS="-Cprofile-generate=pgo-data" \
            cargo build --release --target=x86_64-unknown-linux-gnu

      - name: Run profiling workload
        run: |
          ./target/x86_64-unknown-linux-gnu/release/trading-bot \
            --data test_data/btc_2024.csv

      - name: Merge profile data
        run: |
          LLVM_PROFDATA=$(find $(rustc --print sysroot) -name llvm-profdata)
          $LLVM_PROFDATA merge -o pgo-data/merged.profdata pgo-data/

      - name: Build PGO-optimized binary
        run: |
          cargo clean
          RUSTFLAGS="-Cprofile-use=pgo-data/merged.profdata" \
            cargo build --release --target=x86_64-unknown-linux-gnu

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: trading-bot-pgo
          path: target/x86_64-unknown-linux-gnu/release/trading-bot
```

## Когда использовать PGO

### PGO рекомендуется когда:

| Сценарий | Причина |
|----------|---------|
| **HFT системы** | Каждая микросекунда имеет значение |
| **Парсеры рыночных данных** | Много условных переходов |
| **Расчёт индикаторов** | Интенсивные вычисления в циклах |
| **Production системы** | Стабильные паттерны использования |
| **Критичные по латентности сервисы** | Предсказуемое время отклика |

### PGO не рекомендуется когда:

| Сценарий | Причина |
|----------|---------|
| **Прототипирование** | Overhead на сборку не окупается |
| **Разнообразные нагрузки** | Профиль не репрезентативен |
| **Редко запускаемый код** | Улучшения незаметны |
| **Быстро меняющийся код** | Профиль устаревает |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **PGO** | Profile Guided Optimization — оптимизация на основе профиля |
| **Инструментирование** | Сборка с записью информации о выполнении |
| **Профиль** | Данные о реальном поведении программы |
| **Branch prediction** | Оптимизация условных переходов |
| **Inlining** | Встраивание часто вызываемых функций |
| **Code layout** | Размещение горячего кода для лучшего кэширования |
| **BOLT** | Дополнительная оптимизация после линковки |

## Домашнее задание

1. **Базовая PGO сборка**: Возьми свой торговый бот или используй пример из этой главы:
   - Собери обычную release версию
   - Собери PGO-оптимизированную версию
   - Сравни производительность на одинаковых данных
   - Запиши результаты в таблицу

2. **Профилирование разных сценариев**: Создай несколько наборов тестовых данных:
   - Бычий рынок (постоянный рост)
   - Медвежий рынок (постоянное падение)
   - Боковик (низкая волатильность)
   - Высокая волатильность
   Собери профиль на каждом и сравни, как это влияет на оптимизацию.

3. **Автоматизация**: Напиши скрипт, который:
   - Автоматически определяет, изменился ли код с последней PGO-сборки
   - Если изменился — пересобирает с новым профилем
   - Сохраняет историю производительности
   - Отправляет уведомление если производительность ухудшилась

4. **Интеграция с бенчмарками**: Добавь в свой проект:
   - Criterion бенчмарки для критичных функций
   - Сравнение производительности обычной и PGO сборки
   - Автоматический отчёт с графиками

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../318-*/ru.md)
