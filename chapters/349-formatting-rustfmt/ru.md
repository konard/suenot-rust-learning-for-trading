# День 349: Форматирование кода: rustfmt

## Аналогия из трейдинга

Представь торговый зал крупного хедж-фонда. Каждый трейдер ведёт свои записи о сделках, но если каждый будет записывать их по-своему — один использует запятые для разделения тысяч, другой пробелы, третий вообще ничего — то при анализе общей позиции возникнет хаос.

Поэтому в профессиональных торговых организациях существуют **стандарты документации**:
- Единый формат записи цен (например, всегда 2 знака после запятой)
- Стандартные обозначения тикеров (BTCUSDT, а не btc/usdt или Bitcoin-USDT)
- Единообразное оформление отчётов

**rustfmt** — это автоматический "стандартизатор" для кода Rust. Он обеспечивает единый стиль форматирования во всём проекте, так же как стандарты документации обеспечивают единообразие в торговых отчётах.

| Трейдинг | Программирование |
|----------|------------------|
| Стандарт записи цен | Стиль отступов |
| Формат тикеров | Именование переменных |
| Шаблон отчёта | Структура кода |
| Аудит документов | Code review |

## Что такое rustfmt?

**rustfmt** — это официальный инструмент форматирования кода Rust. Он автоматически приводит код к единому стилю согласно рекомендациям Rust Style Guide.

### Установка и проверка

rustfmt обычно устанавливается вместе с Rust:

```bash
# Проверка установки
rustfmt --version

# Если не установлен
rustup component add rustfmt
```

## Базовое использование

### Форматирование файла

```bash
# Форматирование одного файла
rustfmt src/main.rs

# Форматирование с выводом изменений
rustfmt --check src/main.rs

# Форматирование всего проекта через Cargo
cargo fmt

# Проверка без изменения файлов
cargo fmt --check
```

### Пример: До и после форматирования

Рассмотрим код торгового модуля до форматирования:

```rust
// До форматирования — хаотичный стиль
use std::collections::HashMap;use std::time::{SystemTime,UNIX_EPOCH};

#[derive(Debug,Clone)]
struct Order{symbol:String,side:OrderSide,price:f64,quantity:f64,timestamp:u64}

#[derive(Debug,Clone,Copy)]
enum OrderSide{Buy,Sell}

impl Order{
fn new(symbol:&str,side:OrderSide,price:f64,quantity:f64)->Self{
let timestamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
Order{symbol:symbol.to_string(),side,price,quantity,timestamp}
}

fn total_value(&self)->f64{self.price*self.quantity}

fn is_buy(&self)->bool{matches!(self.side,OrderSide::Buy)}
}

fn calculate_portfolio_value(orders:&[Order],prices:&HashMap<String,f64>)->f64{
orders.iter().filter(|o|o.is_buy()).map(|o|{
let current_price=prices.get(&o.symbol).unwrap_or(&o.price);
o.quantity*current_price}).sum()
}

fn main(){
let mut prices=HashMap::new();
prices.insert("BTCUSDT".to_string(),50000.0);
prices.insert("ETHUSDT".to_string(),3000.0);

let orders=vec![
Order::new("BTCUSDT",OrderSide::Buy,49000.0,0.5),
Order::new("ETHUSDT",OrderSide::Buy,2900.0,2.0),
Order::new("BTCUSDT",OrderSide::Sell,51000.0,0.2),
];

let value=calculate_portfolio_value(&orders,&prices);
println!("Portfolio value: ${:.2}",value);
}
```

После запуска `cargo fmt`:

```rust
// После форматирования — чистый, читаемый код
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

impl Order {
    fn new(symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Order {
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            timestamp,
        }
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }

    fn is_buy(&self) -> bool {
        matches!(self.side, OrderSide::Buy)
    }
}

fn calculate_portfolio_value(orders: &[Order], prices: &HashMap<String, f64>) -> f64 {
    orders
        .iter()
        .filter(|o| o.is_buy())
        .map(|o| {
            let current_price = prices.get(&o.symbol).unwrap_or(&o.price);
            o.quantity * current_price
        })
        .sum()
}

fn main() {
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 50000.0);
    prices.insert("ETHUSDT".to_string(), 3000.0);

    let orders = vec![
        Order::new("BTCUSDT", OrderSide::Buy, 49000.0, 0.5),
        Order::new("ETHUSDT", OrderSide::Buy, 2900.0, 2.0),
        Order::new("BTCUSDT", OrderSide::Sell, 51000.0, 0.2),
    ];

    let value = calculate_portfolio_value(&orders, &prices);
    println!("Portfolio value: ${:.2}", value);
}
```

## Конфигурация rustfmt

### Файл rustfmt.toml

Создай файл `rustfmt.toml` в корне проекта для настройки форматирования:

```toml
# rustfmt.toml — конфигурация для торговой системы

# Максимальная ширина строки
max_width = 100

# Использовать табы вместо пробелов
hard_tabs = false

# Размер отступа
tab_spaces = 4

# Стиль отступа для цепочек методов
chain_width = 60

# Форматирование импортов
imports_granularity = "Module"
group_imports = "StdExternalCrate"

# Переносы в объявлении функций
fn_args_layout = "Tall"

# Стиль фигурных скобок
brace_style = "SameLineWhere"

# Использовать сокращённую инициализацию полей
use_field_init_shorthand = true

# Использовать try! или ?
use_try_shorthand = true
```

### Пример с конфигурацией

```rust
// С настройками imports_granularity = "Module" и group_imports = "StdExternalCrate"
// Импорты группируются и сортируются автоматически

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::TradingError;
use crate::models::{Order, Position, Trade};

/// Торговый движок с автоматическим форматированием
#[derive(Debug)]
pub struct TradingEngine {
    orders: Arc<RwLock<HashMap<String, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    trade_history: Vec<Trade>,
    started_at: Instant,
}

impl TradingEngine {
    /// Создаёт новый торговый движок
    pub fn new() -> Self {
        // use_field_init_shorthand = true позволяет писать так:
        Self {
            orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            trade_history: Vec::new(),
            started_at: Instant::now(),
        }
    }

    /// Размещает новый ордер
    pub fn place_order(
        &mut self,
        symbol: &str,
        side: OrderSide,
        price: f64,
        quantity: f64,
    ) -> Result<String, TradingError> {
        // fn_args_layout = "Tall" — аргументы на отдельных строках
        // когда не помещаются в одну строку

        let order_id = self.generate_order_id();

        let order = Order {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
        };

        self.orders
            .write()
            .map_err(|_| TradingError::LockError)?
            .insert(order_id.clone(), order);

        Ok(order_id)
    }

    /// Получает все открытые позиции
    pub fn get_open_positions(&self) -> Result<Vec<Position>, TradingError> {
        // chain_width = 60 определяет, когда разбивать цепочку методов
        let positions = self
            .positions
            .read()
            .map_err(|_| TradingError::LockError)?;

        Ok(positions
            .values()
            .filter(|p| p.quantity != 0.0)
            .cloned()
            .collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}
```

## Интеграция с рабочим процессом

### Pre-commit hook

Создай файл `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Pre-commit hook для проверки форматирования

echo "Проверка форматирования кода..."

# Проверяем форматирование
if ! cargo fmt --check; then
    echo "Ошибка: код не отформатирован!"
    echo "Запустите 'cargo fmt' перед коммитом."
    exit 1
fi

echo "Форматирование OK!"
```

### Интеграция в CI/CD

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  format:
    name: Check formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --check

  build:
    name: Build and test
    runs-on: ubuntu-latest
    needs: format
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test
```

## Продвинутые возможности

### Атрибут #[rustfmt::skip]

Иногда автоформатирование ухудшает читаемость. Используй `#[rustfmt::skip]`:

```rust
use std::collections::HashMap;

/// Таблица комиссий бирж
/// Формат: (maker_fee, taker_fee)
#[rustfmt::skip]
const EXCHANGE_FEES: &[(&str, (f64, f64))] = &[
    ("Binance",  (0.001, 0.001)),
    ("Coinbase", (0.004, 0.006)),
    ("Kraken",   (0.002, 0.005)),
    ("Bybit",    (0.001, 0.001)),
    ("OKX",      (0.002, 0.005)),
];

/// Матрица корреляции активов — форматирование нарушило бы структуру
#[rustfmt::skip]
const CORRELATION_MATRIX: [[f64; 4]; 4] = [
    // BTC    ETH    SOL    ADA
    [ 1.00,  0.85,  0.72,  0.68],  // BTC
    [ 0.85,  1.00,  0.78,  0.75],  // ETH
    [ 0.72,  0.78,  1.00,  0.82],  // SOL
    [ 0.68,  0.75,  0.82,  1.00],  // ADA
];

/// Уровни поддержки и сопротивления
#[rustfmt::skip]
fn get_price_levels(symbol: &str) -> Vec<f64> {
    match symbol {
        "BTCUSDT" => vec![
            45000.0, 47500.0, 50000.0,  // Поддержка
            52500.0, 55000.0, 60000.0,  // Сопротивление
        ],
        "ETHUSDT" => vec![
            2800.0, 3000.0, 3200.0,
            3500.0, 3800.0, 4000.0,
        ],
        _ => vec![],
    }
}

fn main() {
    // Вывод комиссий в удобном формате
    println!("=== Комиссии бирж ===");
    for (exchange, (maker, taker)) in EXCHANGE_FEES {
        println!("{:12} Maker: {:.2}% Taker: {:.2}%", exchange, maker * 100.0, taker * 100.0);
    }

    println!("\n=== Матрица корреляции ===");
    let assets = ["BTC", "ETH", "SOL", "ADA"];
    print!("     ");
    for asset in &assets {
        print!("{:>6}", asset);
    }
    println!();

    for (i, row) in CORRELATION_MATRIX.iter().enumerate() {
        print!("{:>4} ", assets[i]);
        for val in row {
            print!("{:>6.2}", val);
        }
        println!();
    }
}
```

### Форматирование макросов

```rust
/// Макрос для создания ордера с форматированием
macro_rules! order {
    ($symbol:expr, $side:ident, $price:expr, $qty:expr) => {
        Order {
            symbol: $symbol.to_string(),
            side: OrderSide::$side,
            price: $price,
            quantity: $qty,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    };
}

/// Макрос для логирования сделок
macro_rules! log_trade {
    ($($arg:tt)*) => {
        println!(
            "[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            format!($($arg)*)
        );
    };
}

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    // Использование макросов с красивым форматированием
    let orders = vec![
        order!("BTCUSDT", Buy, 50000.0, 0.1),
        order!("ETHUSDT", Sell, 3000.0, 1.0),
        order!("SOLUSDT", Buy, 100.0, 10.0),
    ];

    for order in &orders {
        println!("{:?}", order);
    }
}
```

## Сравнение с другими форматтерами

| Функция | rustfmt | prettier (JS) | black (Python) |
|---------|---------|---------------|----------------|
| Официальный | Да | Нет | Нет |
| Конфигурация | rustfmt.toml | .prettierrc | pyproject.toml |
| Стиль | Rust Style Guide | Свой | PEP 8 |
| Интеграция с IDE | Отличная | Отличная | Хорошая |
| Скорость | Быстрый | Быстрый | Средний |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **rustfmt** | Официальный форматтер кода Rust |
| **cargo fmt** | Команда для форматирования всего проекта |
| **rustfmt.toml** | Файл конфигурации форматирования |
| **#[rustfmt::skip]** | Атрибут для пропуска форматирования |
| **--check** | Флаг для проверки без изменения файлов |
| **Pre-commit hook** | Автоматическая проверка перед коммитом |

## Практические задания

1. **Настройка проекта**: Настрой rustfmt для своего торгового проекта:
   - Создай `rustfmt.toml` с оптимальными настройками
   - Настрой группировку импортов
   - Установи максимальную ширину строки 100 символов
   - Добавь pre-commit hook

2. **Форматирование существующего кода**: Возьми неформатированный код:
   - Запусти `cargo fmt --check` для анализа
   - Примени форматирование
   - Сравни разницу с `git diff`
   - Убедись, что код стал более читаемым

3. **Использование #[rustfmt::skip]**: Создай модуль с:
   - Таблицей констант, где форматирование важно
   - Матрицей данных для анализа
   - Документируй причины использования skip

4. **Интеграция в CI**: Добавь проверку форматирования:
   - Создай GitHub Actions workflow
   - Добавь шаг проверки форматирования
   - Настрой fail на неформатированный код

## Домашнее задание

1. **Стандартизация торгового проекта**: Создай полную конфигурацию форматирования:
   - Настрой все параметры `rustfmt.toml`
   - Добавь примеры с `#[rustfmt::skip]` для таблиц
   - Создай скрипт для автоформатирования
   - Документируй выбранные настройки

2. **Автоматизация**: Настрой полную автоматизацию:
   - Pre-commit hook для проверки
   - Pre-push hook для форматирования
   - Интеграция с VS Code (format on save)
   - CI/CD pipeline с проверкой

3. **Сравнительный анализ**: Сравни читаемость кода:
   - Возьми сложный торговый модуль
   - Измерь время понимания кода до форматирования
   - Измерь время после форматирования
   - Подготовь отчёт с выводами

4. **Командные стандарты**: Создай гайд для команды:
   - Опиши все настройки форматирования
   - Объясни когда использовать skip
   - Добавь примеры хорошего и плохого кода
   - Создай чеклист для code review

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../350-*/ru.md)
