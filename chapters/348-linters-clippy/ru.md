# День 348: Линтеры: clippy

## Аналогия из трейдинга

Представь, что ты управляешь торговым деском, и у тебя есть опытный старший трейдер, который проверяет все ордера перед их отправкой на биржу. Он не просто проверяет синтаксис (правильно ли указан тикер), но и смотрит на паттерны:

- "Ты уверен, что хочешь купить актив по цене выше рыночной на 5%?"
- "Этот ордер дублирует предыдущий — возможно, это ошибка?"
- "Ты проверяешь значение, но не используешь результат — это подозрительно"
- "Этот код можно написать проще и эффективнее"

**Clippy** — это такой же старший разработчик для Rust. Он анализирует твой код и находит:
- Потенциальные баги
- Неэффективные паттерны
- Устаревший стиль кода
- Места, где код можно упростить

| Инструмент | Аналогия в трейдинге |
|------------|----------------------|
| **Компилятор** | Проверка, что ордер синтаксически корректен |
| **Clippy** | Старший трейдер проверяет логику ордера |
| **rustfmt** | Форматирование отчётов по стандарту |

## Установка и запуск Clippy

Clippy устанавливается вместе с Rust через rustup:

```bash
# Clippy обычно уже установлен, но можно обновить
rustup component add clippy

# Запуск clippy
cargo clippy

# Запуск с более строгими проверками
cargo clippy -- -W clippy::pedantic

# Запуск с запретом всех предупреждений (CI mode)
cargo clippy -- -D warnings
```

## Категории линтов Clippy

Clippy организует свои проверки в категории:

```rust
// Настройка уровней линтов в коде
#![warn(clippy::all)]           // Стандартные проверки
#![warn(clippy::pedantic)]      // Более строгие проверки
#![warn(clippy::nursery)]       // Экспериментальные проверки
#![warn(clippy::cargo)]         // Проверки Cargo.toml
#![deny(clippy::correctness)]   // Критичные ошибки — запретить
```

### Уровни строгости

| Категория | Описание | Применение |
|-----------|----------|------------|
| `clippy::all` | Основные проверки | Рекомендуется всегда |
| `clippy::pedantic` | Строгие проверки стиля | Для чистого кода |
| `clippy::nursery` | Новые/экспериментальные | Для энтузиастов |
| `clippy::restriction` | Ограничительные правила | Для специфичных случаев |
| `clippy::correctness` | Вероятные баги | Критически важно |

## Типичные предупреждения и их исправление

### 1. Ненужное клонирование (Performance)

```rust
// Clippy предупреждает: unnecessary clone
fn process_trade_bad(trade: Trade) {
    let trade_copy = trade.clone();  // Предупреждение!
    println!("Processing: {:?}", trade_copy);
}

// Правильно: используем ссылку
fn process_trade_good(trade: &Trade) {
    println!("Processing: {:?}", trade);
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}
```

### 2. Неиспользуемый Result (Correctness)

```rust
use std::collections::HashMap;

struct OrderBook {
    bids: HashMap<u64, f64>,
    asks: HashMap<u64, f64>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    fn add_bid(&mut self, price: u64, quantity: f64) {
        // Clippy предупреждает: игнорируем Result от insert
        // Здесь это нормально, но clippy хочет явности
        let _ = self.bids.insert(price, quantity);  // Правильно: явно игнорируем
    }

    // Плохой пример:
    fn add_ask_bad(&mut self, price: u64, quantity: f64) {
        self.asks.insert(price, quantity);  // Предупреждение!
    }
}
```

### 3. Неоптимальные итераторы

```rust
fn calculate_total_volume_bad(prices: &[f64], quantities: &[f64]) -> f64 {
    // Clippy предупреждает: можно использовать zip
    let mut total = 0.0;
    for i in 0..prices.len() {
        total += prices[i] * quantities[i];  // Предупреждение!
    }
    total
}

// Правильно: идиоматичный Rust с zip
fn calculate_total_volume_good(prices: &[f64], quantities: &[f64]) -> f64 {
    prices
        .iter()
        .zip(quantities.iter())
        .map(|(p, q)| p * q)
        .sum()
}
```

## Практическое применение в торговом коде

### Пример: Система управления позициями с clippy

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Отключаем некоторые педантичные проверки для читаемости
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

use std::collections::HashMap;

/// Торговая позиция
#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    pub fn new(symbol: &str, quantity: f64, entry_price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            entry_price,
            current_price: entry_price,
        }
    }

    /// Рассчитывает нереализованную прибыль/убыток
    pub fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    /// Обновляет текущую цену
    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
    }

    /// Возвращает рыночную стоимость позиции
    pub fn market_value(&self) -> f64 {
        self.current_price * self.quantity.abs()
    }
}

/// Портфель трейдера
pub struct Portfolio {
    positions: HashMap<String, Position>,
    cash: f64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_cash,
        }
    }

    /// Добавляет новую позицию или увеличивает существующую
    pub fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        // Clippy одобряет: используем entry API
        self.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                // Средневзвешенная цена входа
                let total_quantity = pos.quantity + quantity;
                if total_quantity.abs() > f64::EPSILON {
                    pos.entry_price = (pos.entry_price * pos.quantity + price * quantity)
                        / total_quantity;
                }
                pos.quantity = total_quantity;
            })
            .or_insert_with(|| Position::new(symbol, quantity, price));

        self.cash -= quantity * price;
    }

    /// Закрывает позицию полностью
    pub fn close_position(&mut self, symbol: &str) -> Option<f64> {
        // Clippy одобряет: используем remove вместо get + remove
        self.positions.remove(symbol).map(|pos| {
            let pnl = pos.unrealized_pnl();
            self.cash += pos.market_value() + pnl;
            pnl
        })
    }

    /// Обновляет цены всех позиций
    pub fn update_prices(&mut self, prices: &HashMap<String, f64>) {
        // Clippy одобряет: values_mut для изменения на месте
        for position in self.positions.values_mut() {
            if let Some(&price) = prices.get(&position.symbol) {
                position.update_price(price);
            }
        }
    }

    /// Рассчитывает общий нереализованный PnL
    pub fn total_unrealized_pnl(&self) -> f64 {
        // Clippy одобряет: использование sum()
        self.positions.values().map(Position::unrealized_pnl).sum()
    }

    /// Возвращает общую стоимость портфеля
    pub fn total_value(&self) -> f64 {
        self.cash + self.positions.values().map(Position::market_value).sum::<f64>()
    }

    /// Возвращает позиции, отсортированные по PnL
    pub fn positions_by_pnl(&self) -> Vec<&Position> {
        // Clippy может предложить sorted_by вместо sort_by на клоне
        let mut positions: Vec<_> = self.positions.values().collect();
        positions.sort_by(|a, b| {
            b.unrealized_pnl()
                .partial_cmp(&a.unrealized_pnl())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        positions
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100_000.0);

    // Открываем позиции
    portfolio.add_position("BTCUSDT", 0.5, 50_000.0);
    portfolio.add_position("ETHUSDT", 5.0, 3_000.0);
    portfolio.add_position("SOLUSDT", 100.0, 100.0);

    println!("=== Начальный портфель ===");
    println!("Общая стоимость: ${:.2}", portfolio.total_value());
    println!("Наличные: ${:.2}", portfolio.cash);

    // Обновляем цены
    let mut new_prices = HashMap::new();
    new_prices.insert("BTCUSDT".to_string(), 52_000.0);
    new_prices.insert("ETHUSDT".to_string(), 3_200.0);
    new_prices.insert("SOLUSDT".to_string(), 95.0);

    portfolio.update_prices(&new_prices);

    println!("\n=== После обновления цен ===");
    println!("Общая стоимость: ${:.2}", portfolio.total_value());
    println!("Нереализованный PnL: ${:.2}", portfolio.total_unrealized_pnl());

    println!("\n=== Позиции по PnL ===");
    for pos in portfolio.positions_by_pnl() {
        println!(
            "{}: quantity={:.2}, PnL=${:.2}",
            pos.symbol,
            pos.quantity,
            pos.unrealized_pnl()
        );
    }

    // Закрываем прибыльную позицию
    if let Some(pnl) = portfolio.close_position("ETHUSDT") {
        println!("\nЗакрыта позиция ETHUSDT с PnL: ${:.2}", pnl);
    }

    println!("\n=== Финальный портфель ===");
    println!("Общая стоимость: ${:.2}", portfolio.total_value());
    println!("Наличные: ${:.2}", portfolio.cash);
}
```

## Конфигурация Clippy через файл

Создайте файл `clippy.toml` в корне проекта:

```toml
# clippy.toml

# Максимальная сложность функции
cognitive-complexity-threshold = 25

# Минимальная длина для предупреждения о магических числах
trivial-copy-size-limit = 8

# Разрешённые имена переменных (для отключения предупреждений)
allowed-idents-below-min-chars = ["i", "j", "x", "y", "id"]

# Порог для слишком большого количества аргументов
too-many-arguments-threshold = 7

# Порог для слишком большого количества строк в функции
too-many-lines-threshold = 100
```

## Автоматическое исправление с cargo clippy --fix

```bash
# Автоматически исправить простые проблемы
cargo clippy --fix

# Исправить даже если есть uncommitted changes
cargo clippy --fix --allow-dirty

# Исправить с разрешением на unstaged files
cargo clippy --fix --allow-staged
```

## Интеграция с CI/CD

### GitHub Actions

```yaml
# .github/workflows/clippy.yml
name: Clippy

on: [push, pull_request]

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Run Clippy
        run: cargo clippy -- -D warnings
```

## Специфичные линты для торгового кода

### Проверки точности чисел с плавающей точкой

```rust
#![warn(clippy::float_cmp)]

fn check_price_equal_bad(price1: f64, price2: f64) -> bool {
    price1 == price2  // Предупреждение! Небезопасное сравнение float
}

fn check_price_equal_good(price1: f64, price2: f64) -> bool {
    (price1 - price2).abs() < f64::EPSILON * 100.0  // Правильно
}

// Или использовать специальный тип для денег
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Price(i64);  // Храним в центах/сатоши

impl Price {
    fn from_float(value: f64, precision: u32) -> Self {
        Price((value * 10_f64.powi(precision as i32)).round() as i64)
    }

    fn to_float(self, precision: u32) -> f64 {
        self.0 as f64 / 10_f64.powi(precision as i32)
    }
}
```

### Проверки переполнения для больших объёмов

```rust
#![warn(clippy::integer_arithmetic)]
#![warn(clippy::cast_possible_truncation)]

fn calculate_total_value_bad(quantity: u64, price: u64) -> u64 {
    quantity * price  // Предупреждение! Возможно переполнение
}

fn calculate_total_value_good(quantity: u64, price: u64) -> Option<u64> {
    quantity.checked_mul(price)  // Правильно: явная обработка переполнения
}

// Или использовать saturating операции
fn calculate_total_value_safe(quantity: u64, price: u64) -> u64 {
    quantity.saturating_mul(price)  // Возвращает MAX при переполнении
}
```

## Подавление предупреждений

Иногда нужно отключить конкретные предупреждения:

```rust
// Для конкретной строки
#[allow(clippy::needless_return)]
fn get_price() -> f64 {
    return 100.0;  // Нужен явный return для ясности
}

// Для функции
#[allow(clippy::too_many_arguments)]
fn create_complex_order(
    symbol: &str,
    side: &str,
    order_type: &str,
    price: f64,
    quantity: f64,
    stop_price: f64,
    take_profit: f64,
    time_in_force: &str,
) -> Order {
    // ...
    Order::default()
}

#[derive(Default)]
struct Order;

// Для модуля
#[allow(clippy::module_inception)]
mod order {
    pub mod order {
        // ...
    }
}
```

## Кастомные линты для торговых систем

Хотя Clippy не позволяет создавать кастомные линты напрямую, можно использовать комбинацию существующих:

```rust
// Рекомендуемая конфигурация для торговых систем
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

// Критичные для финансов
#![deny(clippy::float_cmp)]           // Запретить небезопасное сравнение float
#![deny(clippy::integer_arithmetic)]   // Предупреждать о возможных переполнениях
#![deny(clippy::unwrap_used)]         // Запретить .unwrap() в продакшн коде
#![deny(clippy::expect_used)]         // Запретить .expect() в продакшн коде
#![deny(clippy::panic)]               // Запретить panic! в продакшн коде

// Отключаем слишком строгие для нашего случая
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **clippy::all** | Базовые проверки качества кода |
| **clippy::pedantic** | Строгие стилистические проверки |
| **clippy::correctness** | Проверки на вероятные баги |
| **cargo clippy --fix** | Автоматическое исправление проблем |
| **#[allow(clippy::...)]** | Подавление конкретных предупреждений |
| **clippy.toml** | Конфигурация порогов и настроек |
| **CI интеграция** | Автоматические проверки в пайплайне |

## Практические задания

1. **Аудит существующего кода**: Возьми один из своих проектов и:
   - Запусти `cargo clippy -- -W clippy::pedantic`
   - Исправь все предупреждения категории `correctness`
   - Проанализируй предупреждения `pedantic` и реши, какие стоит исправить
   - Добавь подавление для тех, которые не применимы

2. **Настройка CI**: Создай GitHub Actions workflow:
   - Запускает clippy на каждый PR
   - Блокирует мёрж при наличии предупреждений
   - Отправляет отчёт в комментарий к PR

3. **Рефакторинг торгового модуля**: Возьми код из примера и:
   - Добавь обработку переполнения для всех арифметических операций
   - Замени сравнения float на безопасные альтернативы
   - Убери все .unwrap() и замени на proper error handling

4. **Кастомная конфигурация**: Создай `clippy.toml` для торгового проекта:
   - Настрой пороги сложности под специфику финансового кода
   - Определи список разрешённых коротких идентификаторов
   - Задокументируй причины отключения определённых линтов

## Домашнее задание

1. **Система мониторинга качества кода**: Реализуй систему, которая:
   - Запускает clippy с разными уровнями строгости
   - Группирует предупреждения по категориям
   - Отслеживает динамику (увеличение/уменьшение предупреждений)
   - Генерирует отчёт с приоритетами исправления
   - Интегрируется с системой тикетов

2. **Безопасный калькулятор для трейдинга**: Напиши библиотеку:
   - Все арифметические операции с проверкой переполнения
   - Типобезопасные представления денег и цен
   - Нулевые предупреждения от `clippy::pedantic`
   - Документация всех публичных функций
   - 100% покрытие тестами

3. **Линтер для торговой логики**: Используя clippy и кастомные проверки:
   - Найди паттерны, специфичные для торговых ошибок
   - Реализуй проверки через атрибуты и макросы
   - Добавь проверки бизнес-логики (валидация ордеров, риск-лимиты)
   - Интегрируй с существующим CI/CD

4. **Миграция legacy кода**: Возьми старый торговый код и:
   - Проведи полный аудит с clippy
   - Создай план миграции с приоритетами
   - Реализуй автоматические исправления где возможно
   - Напиши документацию по изменениям
   - Добавь регрессионные тесты

## Навигация

[← Предыдущий день](../347-testing-in-ci/ru.md) | [Следующий день →](../349-formatting-rustfmt/ru.md)
