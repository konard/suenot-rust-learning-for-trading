# День 55: Отладка проблем владения

## Аналогия из трейдинга

Представь, что ты управляющий торговым фондом и обнаружил аномалию: одна и та же акция числится сразу у двух трейдеров. Или ещё хуже — кто-то пытается продать актив, который уже был продан. Для расследования нужно: посмотреть логи операций, найти момент ошибки, понять кто и когда передал права. В Rust компилятор — это твой аудитор, который ловит такие проблемы **до** того, как они попадут в продакшн.

## Типичные ошибки владения и как их читать

### 1. Use after move — использование после перемещения

```rust
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = portfolio;  // Владение перемещено

    // Ошибка! portfolio больше не владеет данными
    println!("Активное портфолио: {:?}", portfolio);
}
```

**Сообщение компилятора:**
```
error[E0382]: borrow of moved value: `portfolio`
 --> src/main.rs:6:43
  |
2 |     let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
  |         --------- move occurs because `portfolio` has type `Vec<&str>`
3 |     let archived = portfolio;
  |                    --------- value moved here
...
6 |     println!("Активное портфолио: {:?}", portfolio);
  |                                          ^^^^^^^^^ value borrowed here after move
```

**Как читать:**
- `move occurs because` — тип не реализует Copy, поэтому происходит перемещение
- `value moved here` — здесь значение было перемещено
- `value borrowed here after move` — здесь пытаемся использовать после перемещения

**Решения:**
```rust
// Решение 1: Clone — создаём копию
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = portfolio.clone();  // Копируем
    println!("Активное: {:?}", portfolio);
    println!("Архив: {:?}", archived);
}

// Решение 2: Ссылка — смотрим, не владеем
fn main() {
    let portfolio = vec!["AAPL", "GOOGL", "MSFT"];
    let archived = &portfolio;  // Только ссылка
    println!("Активное: {:?}", portfolio);
    println!("Архив: {:?}", archived);
}
```

### 2. Заимствование после перемещения в функцию

```rust
fn archive_portfolio(portfolio: Vec<String>) {
    println!("Архивируем: {:?}", portfolio);
}

fn main() {
    let portfolio = vec!["BTC".to_string(), "ETH".to_string()];
    archive_portfolio(portfolio);  // Владение передано

    // Ошибка! portfolio больше не наше
    println!("Размер: {}", portfolio.len());
}
```

**Решение: принимаем ссылку вместо владения**
```rust
fn archive_portfolio(portfolio: &Vec<String>) {
    println!("Архивируем: {:?}", portfolio);
}

fn main() {
    let portfolio = vec!["BTC".to_string(), "ETH".to_string()];
    archive_portfolio(&portfolio);  // Передаём ссылку
    println!("Размер: {}", portfolio.len());  // Работает!
}
```

### 3. Одновременное изменяемое и неизменяемое заимствование

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    let first = &prices[0];  // Неизменяемое заимствование
    prices.push(42200.0);    // Изменяемое заимствование

    println!("Первая цена: {}", first);  // Ошибка!
}
```

**Сообщение компилятора:**
```
error[E0502]: cannot borrow `prices` as mutable because it is also borrowed as immutable
 --> src/main.rs:5:5
  |
4 |     let first = &prices[0];
  |                  ------ immutable borrow occurs here
5 |     prices.push(42200.0);
  |     ^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
6 |
7 |     println!("Первая цена: {}", first);
  |                                 ----- immutable borrow later used here
```

**Как читать:**
- `cannot borrow as mutable because it is also borrowed as immutable` — нельзя изменять, пока есть неизменяемая ссылка
- `immutable borrow occurs here` — здесь взяли неизменяемую ссылку
- `immutable borrow later used here` — и здесь она ещё используется

**Решение: разделяем во времени**
```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // Сначала используем ссылку
    let first = &prices[0];
    println!("Первая цена: {}", first);
    // Ссылка больше не нужна

    // Теперь можем изменять
    prices.push(42200.0);
    println!("Цены после добавления: {:?}", prices);
}
```

### 4. Dangling reference — висячая ссылка

```rust
fn get_best_price() -> &f64 {
    let price = 42000.0;
    &price  // Ошибка! price уничтожится
}

fn main() {
    let best = get_best_price();
    println!("Лучшая цена: {}", best);
}
```

**Сообщение компилятора:**
```
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:24
  |
1 | fn get_best_price() -> &f64 {
  |                        ^ expected named lifetime parameter
```

**Решение: возвращаем владение**
```rust
fn get_best_price() -> f64 {
    let price = 42000.0;
    price  // Возвращаем значение, не ссылку
}

fn main() {
    let best = get_best_price();
    println!("Лучшая цена: {}", best);
}
```

## Инструменты отладки

### 1. Команда cargo check

Быстрая проверка без полной компиляции:

```bash
cargo check
```

Показывает ошибки владения без траты времени на генерацию кода.

### 2. Cargo clippy — расширенный анализ

```bash
cargo clippy
```

Clippy находит не только ошибки, но и неоптимальные паттерны:

```rust
// Clippy предупредит: лишнее клонирование
fn process_order(order: Order) {
    let backup = order.clone();  // Clippy: "unnecessary clone"
    // используем только order, backup не нужен
}
```

### 3. Аннотации типов для понимания

Когда не понимаешь что происходит, добавь явные типы:

```rust
fn analyze_trades(trades: Vec<Trade>) -> AnalysisResult {
    let filtered: Vec<&Trade> = trades  // Явный тип помогает понять
        .iter()                          // iter() даёт &Trade
        .filter(|t| t.is_profitable())
        .collect();

    // Теперь видно: filtered содержит ссылки на trades
    // значит trades должен жить дольше filtered

    process(&filtered)
}
```

### 4. Комментарии о владении

```rust
struct TradingEngine {
    // Владеет списком ордеров
    orders: Vec<Order>,

    // Владеет конфигурацией (клонируется при создании)
    config: Config,

    // НЕ владеет — только ссылка на внешний логгер
    // Время жизни: engine не может пережить logger
    logger: &'static Logger,
}
```

## Практические паттерны решения

### Паттерн 1: Раннее освобождение ссылок

```rust
fn update_portfolio(portfolio: &mut Portfolio, market_data: &MarketData) {
    // Проблемный код:
    // let price = portfolio.get_price("BTC");  // &f64
    // portfolio.update("BTC", new_price);       // &mut — конфликт!
    // println!("Было: {}", price);

    // Решение: копируем значение
    let price = *portfolio.get_price("BTC");  // f64, не &f64
    portfolio.update("BTC", market_data.get("BTC"));
    println!("Было: {}", price);  // Работает!
}
```

### Паттерн 2: Индексы вместо ссылок

```rust
fn find_best_trade(trades: &mut Vec<Trade>) -> Option<usize> {
    // Вместо хранения ссылки &Trade, храним индекс
    let mut best_idx = None;
    let mut best_profit = 0.0;

    for (idx, trade) in trades.iter().enumerate() {
        if trade.profit > best_profit {
            best_profit = trade.profit;
            best_idx = Some(idx);
        }
    }

    // Теперь можем модифицировать trades
    if let Some(idx) = best_idx {
        trades[idx].mark_as_best();
    }

    best_idx
}
```

### Паттерн 3: Разделение структуры

```rust
// Проблема: хотим читать orders и писать в stats одновременно
struct TradingSystem {
    orders: Vec<Order>,
    stats: Statistics,
}

// Решение: разделяем на части
struct TradingSystem {
    orders: OrderBook,
    stats: Statistics,
}

impl TradingSystem {
    fn process(&mut self) {
        // Можем заимствовать разные поля независимо
        let orders = &self.orders;
        let stats = &mut self.stats;

        for order in orders.iter() {
            stats.record(order);
        }
    }
}
```

### Паттерн 4: Временные переменные

```rust
fn calculate_metrics(trades: &mut Vec<Trade>) {
    // Проблема:
    // for trade in trades.iter() {
    //     trades.push(trade.generate_hedge());  // Ошибка!
    // }

    // Решение: собираем изменения отдельно
    let hedges: Vec<Trade> = trades
        .iter()
        .map(|t| t.generate_hedge())
        .collect();

    trades.extend(hedges);
}
```

## Практические упражнения

### Упражнение 1: Исправь ошибку владения

```rust
fn main() {
    let orders = vec!["BUY BTC", "SELL ETH", "BUY SOL"];
    process_orders(orders);
    println!("Обработано {} ордеров", orders.len());  // Ошибка!
}

fn process_orders(orders: Vec<&str>) {
    for order in orders {
        println!("Обработка: {}", order);
    }
}
```

### Упражнение 2: Исправь конфликт заимствований

```rust
fn main() {
    let mut balances = vec![1000.0, 2000.0, 500.0];
    let first = &balances[0];
    balances[0] = 1500.0;
    println!("Первый баланс: {}", first);
}
```

### Упражнение 3: Исправь висячую ссылку

```rust
fn get_ticker() -> &str {
    let ticker = String::from("BTC/USDT");
    &ticker
}
```

### Упражнение 4: Сложный случай

```rust
struct Portfolio {
    assets: Vec<String>,
    total_value: f64,
}

impl Portfolio {
    fn get_asset(&self, idx: usize) -> &String {
        &self.assets[idx]
    }

    fn update_value(&mut self, new_value: f64) {
        self.total_value = new_value;
    }
}

fn main() {
    let mut portfolio = Portfolio {
        assets: vec!["BTC".to_string(), "ETH".to_string()],
        total_value: 50000.0,
    };

    let first_asset = portfolio.get_asset(0);
    portfolio.update_value(55000.0);
    println!("Актив: {}", first_asset);
}
```

## Домашнее задание

1. **Анализатор ошибок:** Создай функцию, которая принимает строку с кодом ошибки компилятора Rust (E0382, E0502, E0106) и возвращает объяснение проблемы и типичное решение

2. **Безопасный кеш цен:** Реализуй структуру `PriceCache`, которая хранит последние N цен и позволяет:
   - Добавлять новые цены
   - Получать среднюю цену
   - Получать последнюю цену
   Убедись, что все операции безопасны с точки зрения владения

3. **Рефакторинг:** Возьми следующий код с ошибками и исправь его тремя разными способами:

```rust
fn analyze_and_update(data: Vec<f64>) -> Vec<f64> {
    let avg = data.iter().sum::<f64>() / data.len() as f64;
    let normalized = data;  // Проблема здесь

    normalized.iter().map(|x| x - avg).collect()
}
```

4. **Документирование:** Добавь к своему решению комментарии, объясняющие почему каждое изменение решает проблему владения

## Что мы узнали

| Ошибка | Код | Типичная причина | Решение |
|--------|-----|------------------|---------|
| Use after move | E0382 | Значение перемещено | Clone или ссылка |
| Double borrow | E0502 | &mut + & одновременно | Разделить во времени |
| Dangling ref | E0106 | Ссылка на локальное | Вернуть владение |
| Lifetime | E0597 | Ссылка живёт дольше | Продлить жизнь источника |

## Навигация

[← Предыдущий день](../054-borrow-dont-own/ru.md) | [Следующий день →](../056-rc-multiple-owners/ru.md)
