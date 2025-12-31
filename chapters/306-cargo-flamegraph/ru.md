# День 306: cargo flamegraph: визуализация производительности

## Аналогия из трейдинга

Представь, что ты запускаешь высокочастотную торговую систему, которая обрабатывает тысячи рыночных ордеров в секунду. Всё работает нормально при тестировании, но на продакшене ты замечаешь периодические задержки, из-за которых упускаешь прибыльные возможности. Где узкое место?

Это как попытка найти самого медленного трейдера на большой торговой площадке в час пик — нужно видеть, кто тратит больше всего времени на каждую операцию. **cargo flamegraph** — это как тепловая карта твоей торговой площадки, показывающая точно, где тратится время.

Flamegraph — это визуализация, где:
- **Ширина** показывает, сколько времени тратится в функции (шире = больше времени)
- **Высота** показывает стек вызовов (насколько глубоко вложены функции)
- **Горячие цвета** (красный, оранжевый) выделяют самые времязатратные операции

Так же, как ты бы определил того одного трейдера, который обрабатывает ордера вручную вместо использования автоматизированных инструментов, flamegraph помогает выявить медленные функции в коде, которые нуждаются в оптимизации.

## Почему профилирование производительности критично в трейдинге?

В алгоритмическом трейдинге миллисекунды имеют значение:

| Проблема | Последствие | Решение |
|----------|-------------|---------|
| Медленная обработка ордеров | Упущенные арбитражные возможности | Профилировать и оптимизировать горячие пути |
| Неэффективные расчеты цен | Задержка торговых сигналов | Выявить вычислительные узкие места |
| Аллокации памяти в циклах | Увеличенная задержка | Найти точки частых аллокаций |
| Избыточный парсинг данных | Потерянные циклы CPU | Визуализировать ненужную работу |
| Блокирующие I/O операции | Замедление всей системы | Обнаружить блокирующие вызовы |

## Что такое cargo flamegraph?

`cargo flamegraph` — это инструмент Rust, который генерирует flame-графики, показывающие, где твоя программа тратит время во время выполнения. Он:

1. **Профилирует** твоё приложение, используя системные инструменты (perf на Linux, DTrace на macOS)
2. **Семплирует** стек вызовов сотни раз в секунду
3. **Генерирует** интерактивную SVG визуализацию
4. **Показывает**, какие функции потребляют больше всего времени CPU

### Установка

```bash
# Установить инструмент
cargo install flamegraph

# На Linux может потребоваться настроить права для perf
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

### Базовое использование

```bash
# Профилировать приложение
cargo flamegraph

# Профилировать релизную сборку (рекомендуется)
cargo flamegraph --release

# Профилировать с конкретными аргументами
cargo flamegraph --release -- --data-file market_data.csv

# Профилировать конкретный бинарник
cargo flamegraph --bin trading_engine
```

## Пример 1: Поиск узких мест в расчете цен

Создадим торговую систему, которая рассчитывает различные ценовые метрики, и используем flamegraph для поиска проблем производительности.

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
struct PriceData {
    timestamp: u64,
    price: f64,
    volume: f64,
}

/// Расчет простой скользящей средней (медленная версия)
fn calculate_sma_slow(prices: &[f64], window: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in window..=prices.len() {
        let sum: f64 = prices[i - window..i].iter().sum();
        result.push(sum / window as f64);
    }

    result
}

/// Расчет экспоненциальной скользящей средней (медленная версия)
fn calculate_ema_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Первая EMA — это SMA
    let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);

    for i in period..prices.len() {
        let ema = (prices[i] - result.last().unwrap()) * multiplier + result.last().unwrap();
        result.push(ema);
    }

    result
}

/// Расчет RSI (медленная версия с повторными аллокациями)
fn calculate_rsi_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsi_values = Vec::new();

    for i in period..prices.len() {
        let mut gains = Vec::new();  // Аллокация в горячем цикле!
        let mut losses = Vec::new(); // Аллокация в горячем цикле!

        for j in i - period + 1..=i {
            let change = prices[j] - prices[j - 1];
            if change > 0.0 {
                gains.push(change);
            } else {
                losses.push(change.abs());
            }
        }

        let avg_gain = if !gains.is_empty() {
            gains.iter().sum::<f64>() / gains.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };

        rsi_values.push(rsi);
    }

    rsi_values
}

/// Обработка рыночных данных с множественными индикаторами
fn analyze_market(prices: &[f64]) {
    let start = Instant::now();

    println!("Расчет индикаторов для {} ценовых точек...", prices.len());

    // Эти расчеты будут видны во flamegraph
    let sma_20 = calculate_sma_slow(prices, 20);
    let sma_50 = calculate_sma_slow(prices, 50);
    let ema_12 = calculate_ema_slow(prices, 12);
    let ema_26 = calculate_ema_slow(prices, 26);
    let rsi_14 = calculate_rsi_slow(prices, 14);

    println!("SMA(20): {} значений", sma_20.len());
    println!("SMA(50): {} значений", sma_50.len());
    println!("EMA(12): {} значений", ema_12.len());
    println!("EMA(26): {} значений", ema_26.len());
    println!("RSI(14): {} значений", rsi_14.len());

    println!("Анализ завершен за {:?}", start.elapsed());
}

/// Генерация тестовых ценовых данных
fn generate_price_data(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 50000.0;

    for i in 0..count {
        // Симуляция движения цены
        let change = ((i * 7) % 100) as f64 - 50.0;
        price += change;
        prices.push(price);
    }

    prices
}

fn main() {
    // Генерируем большой датасет для видимости в профилировании
    let prices = generate_price_data(10000);

    // Запускаем анализ несколько раз для получения четких данных профилирования
    for iteration in 1..=5 {
        println!("\n=== Итерация {} ===", iteration);
        analyze_market(&prices);
    }
}
```

**Запуск с flamegraph:**

```bash
# Сохрани код в src/main.rs
cargo flamegraph --release

# Это:
# 1. Соберет код в релизном режиме
# 2. Запустит его с профилированием
# 3. Сгенерирует flamegraph.svg в корне проекта
# 4. Откроет его в браузере
```

**Что ты увидишь во flamegraph:**
- `calculate_rsi_slow` скорее всего будет самой широкой (больше всего времени)
- Ты увидишь повторные аллокации в горячем цикле
- `calculate_sma_slow` может показать неэффективные паттерны итерации

## Пример 2: Оптимизированная версия после профилирования

После анализа flamegraph мы можем оптимизировать горячие пути:

```rust
/// Расчет SMA (оптимизированная версия)
fn calculate_sma_optimized(prices: &[f64], window: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len() - window + 1);

    if prices.len() < window {
        return result;
    }

    // Рассчитываем первую SMA
    let mut sum: f64 = prices[..window].iter().sum();
    result.push(sum / window as f64);

    // Используем скользящее окно: убираем старейшую, добавляем новейшую
    for i in window..prices.len() {
        sum = sum - prices[i - window] + prices[i];
        result.push(sum / window as f64);
    }

    result
}

/// Расчет RSI (оптимизированная версия - предварительно выделенные буферы)
fn calculate_rsi_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsi_values = Vec::with_capacity(prices.len() - period);

    // Предварительно рассчитываем все изменения цены
    let mut changes: Vec<f64> = Vec::with_capacity(prices.len() - 1);
    for i in 1..prices.len() {
        changes.push(prices[i] - prices[i - 1]);
    }

    // Используем скользящие средние вместо перерасчета
    for i in period - 1..changes.len() {
        let window = &changes[i - period + 1..=i];

        let (gain_sum, loss_sum) = window.iter().fold((0.0, 0.0), |(g, l), &change| {
            if change > 0.0 {
                (g + change, l)
            } else {
                (g, l + change.abs())
            }
        });

        let avg_gain = gain_sum / period as f64;
        let avg_loss = loss_sum / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };

        rsi_values.push(rsi);
    }

    rsi_values
}

/// Сравнение производительности
fn compare_performance(prices: &[f64]) {
    use std::time::Instant;

    println!("=== Сравнение производительности ===\n");

    // Тест медленной версии
    let start = Instant::now();
    let _rsi_slow = calculate_rsi_slow(prices, 14);
    let slow_time = start.elapsed();
    println!("RSI (медленная):        {:?}", slow_time);

    // Тест оптимизированной версии
    let start = Instant::now();
    let _rsi_fast = calculate_rsi_optimized(prices, 14);
    let fast_time = start.elapsed();
    println!("RSI (оптимизированная): {:?}", fast_time);

    let speedup = slow_time.as_secs_f64() / fast_time.as_secs_f64();
    println!("\nУскорение: {:.2}x быстрее", speedup);
}

fn main() {
    let prices = generate_price_data(10000);
    compare_performance(&prices);
}
```

## Пример 3: Профилирование движка сопоставления ордеров

Давай профилируем простую систему сопоставления ордеров:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

struct OrderBook {
    bids: BTreeMap<u64, Order>,  // Ордера на покупку (цена как ключ * 100)
    asks: BTreeMap<u64, Order>,  // Ордера на продажу (цена как ключ * 100)
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Добавить ордер и попытаться сопоставить
    fn add_order(&mut self, order: Order) -> Vec<(u64, u64, f64, f64)> {
        let mut matches = Vec::new();

        match order.side {
            OrderSide::Buy => {
                // Пытаемся сопоставить с asks
                let price_key = (order.price * 100.0) as u64;

                // Находим подходящие ордера на продажу
                let matching_asks: Vec<_> = self.asks
                    .iter()
                    .filter(|(_, ask)| ask.price <= order.price)
                    .map(|(_, ask)| ask.clone())
                    .collect();

                for ask in matching_asks {
                    let ask_key = (ask.price * 100.0) as u64;
                    matches.push((order.id, ask.id, ask.price, ask.quantity.min(order.quantity)));
                    self.asks.remove(&ask_key);
                }

                // Добавляем в книгу ордеров, если не полностью сопоставлен
                if matches.is_empty() {
                    self.bids.insert(price_key, order);
                }
            }
            OrderSide::Sell => {
                // Аналогичная логика для ордеров на продажу
                let price_key = (order.price * 100.0) as u64;

                let matching_bids: Vec<_> = self.bids
                    .iter()
                    .filter(|(_, bid)| bid.price >= order.price)
                    .map(|(_, bid)| bid.clone())
                    .collect();

                for bid in matching_bids {
                    let bid_key = (bid.price * 100.0) as u64;
                    matches.push((order.id, bid.id, bid.price, bid.quantity.min(order.quantity)));
                    self.bids.remove(&bid_key);
                }

                if matches.is_empty() {
                    self.asks.insert(price_key, order);
                }
            }
        }

        matches
    }
}

fn simulate_trading_day() {
    let mut order_book = OrderBook::new();
    let mut order_id = 1u64;

    // Симулируем 100,000 ордеров
    for i in 0..100_000 {
        let side = if i % 2 == 0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let base_price = 50000.0;
        let price_variation = ((i * 7) % 100) as f64 - 50.0;
        let price = base_price + price_variation;

        let order = Order {
            id: order_id,
            side,
            price,
            quantity: 0.1,
        };

        let matches = order_book.add_order(order);

        if !matches.is_empty() && i % 10000 == 0 {
            println!("Ордер {} сопоставлен с {} ордерами", order_id, matches.len());
        }

        order_id += 1;
    }

    println!("Симуляция завершена!");
    println!("Осталось bid-ордеров: {}", order_book.bids.len());
    println!("Осталось ask-ордеров: {}", order_book.asks.len());
}

fn main() {
    println!("Запуск симуляции сопоставления ордеров...\n");
    simulate_trading_day();
}
```

## Чтение Flamegraph

Когда ты откроешь `flamegraph.svg`, ищи:

### 1. **Широкие полосы** = Горячие функции
Чем шире полоса, тем больше времени CPU потребила функция:
```
[────── calculate_rsi_slow ──────]  ← Это медленно!
[─ calculate_sma ─]                 ← Это быстрее
```

### 2. **Высокие стеки** = Глубокие цепочки вызовов
Высота показывает глубину вызовов. Очень высокие стеки могут указывать на:
- Рекурсивные функции
- Глубокие слои абстракций
- Потенциал для инлайнинга

### 3. **Цвета** = Разные модули/крейты
- Твой код vs библиотечный код
- Разные модули в твоем крейте
- Функции стандартной библиотеки

### 4. **Информация в подсказках**
Наведи курсор на полосу, чтобы увидеть:
- Имя функции
- Процент от общего времени
- Абсолютное время
- Количество семплов

## Распространенные проблемы производительности

| Паттерн во Flamegraph | Проблема | Решение |
|----------------------|----------|----------|
| Широкие полосы аллокаций | Слишком много heap-аллокаций | Предварительное выделение или stack |
| Повторные маленькие функции | Отсутствие инлайнинга | Используй `#[inline]` или LTO |
| Большие clone операции | Ненужное копирование | Используй ссылки или `Cow` |
| Функции format/parse | Конвертации строк | Кэширование или бинарные форматы |
| Конкуренция за locks | Блокировка Mutex | Lock-free структуры или более тонкая блокировка |

## Практические советы

### 1. Всегда профилируй релизные сборки
```bash
cargo flamegraph --release
```
Debug-сборки включают множество дополнительных проверок, которые искажают результаты.

### 2. Включи отладочные символы в релизе
Добавь в `Cargo.toml`:
```toml
[profile.release]
debug = true  # Включить отладочные символы для профилирования
```

### 3. Запускай достаточно долго для хороших данных
Короткие запуски могут не предоставить репрезентативных семплов:
```rust
// Запускай операции несколько раз
for _ in 0..1000 {
    analyze_market(&prices);
}
```

### 4. Профилируй реальные нагрузки
Используй реальные рыночные данные или реалистичные симуляции:
```bash
cargo flamegraph --release -- --data-file real_market_data.csv
```

### 5. Сравнивай до/после
Генерируй flamegraph до и после оптимизации:
```bash
# До
cargo flamegraph --release -o flamegraph_before.svg

# После оптимизации
cargo flamegraph --release -o flamegraph_after.svg
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Flamegraph** | Визуальное представление использования времени CPU |
| **Семплирование** | Периодический захват стеков вызовов во время выполнения |
| **Горячий путь** | Секции кода, потребляющие больше всего времени |
| **Профилирование** | Измерение того, где программа тратит время |
| **Стек вызовов** | Цепочка вызовов функций в любой точке |
| **cargo flamegraph** | Инструмент для генерации flamegraph в Rust |
| **Узкое место** | Самая медленная часть, ограничивающая общую производительность |

## Домашнее задание

1. **Профилирование собственного кода**: Возьми любой предыдущий торговый пример и:
   - Добавь профилирование `cargo flamegraph`
   - Определи самую медленную функцию
   - Сгенерируй flamegraph отчет, показывающий до/после оптимизации

2. **Оптимизация книги ордеров**: Профилируй пример сопоставления ордеров:
   - Найди самую горячую функцию в логике сопоставления
   - Оптимизируй её (рассмотри использование лучших структур данных)
   - Измерь ускорение с бенчмарками

3. **Панель множественных индикаторов**: Создай систему, рассчитывающую 10+ индикаторов:
   - SMA с 5 разными периодами
   - EMA с 3 разными периодами
   - RSI, MACD, полосы Боллинджера
   - Профилируй, чтобы найти, какой индикатор самый медленный
   - Оптимизируй топ-3 узких места

4. **Отчет анализа Flamegraph**: Напиши инструмент, который:
   - Автоматически запускает cargo flamegraph
   - Парсит сгенерированные данные
   - Определяет функции, занимающие > 10% времени
   - Генерирует текстовый отчет с предложениями по оптимизации

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
