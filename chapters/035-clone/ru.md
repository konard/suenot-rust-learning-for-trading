# День 35: Clone — копируем портфель

## Аналогия из трейдинга

Представь, что у тебя есть торговый портфель с позициями: BTC, ETH, SOL. Ты хочешь создать **точную копию** этого портфеля для тестирования новой стратегии, не затрагивая оригинал.

В реальном мире:
- **Оригинальный портфель** остаётся нетронутым
- **Копия** — это независимый портфель с теми же активами
- Изменения в копии не влияют на оригинал

В Rust это реализуется через trait **Clone** — явное, глубокое копирование данных.

## Проблема: Move забирает владение

Вспомним из предыдущего дня — передача владения (move) делает оригинал недоступным:

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Move — portfolio больше не доступен
    let backup = portfolio;

    // println!("{}", portfolio);  // ОШИБКА! Значение перемещено
    println!("{}", backup);
}
```

Но что если нам нужны **оба** — и оригинал, и копия?

## Решение: Clone

`Clone` позволяет создать полную копию данных:

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Clone — создаём копию, оригинал остаётся
    let backup = portfolio.clone();

    println!("Оригинал: {}", portfolio);  // Работает!
    println!("Копия: {}", backup);         // Тоже работает!
}
```

## Как работает Clone

```rust
fn main() {
    // String хранится в куче
    let original = String::from("AAPL");

    // .clone() создаёт новую строку в куче
    // с копией всех данных
    let cloned = original.clone();

    // Теперь у нас ДВА независимых String
    // Каждый владеет своей памятью

    println!("Original addr: {:p}", original.as_ptr());
    println!("Cloned addr: {:p}", cloned.as_ptr());
    // Адреса разные — это разные участки памяти
}
```

## Clone для торговых данных

```rust
fn main() {
    // Позиция в портфеле
    let position = String::from("LONG BTC @ 42000");

    // Сохраняем историю — нужна копия
    let history_entry = position.clone();

    // Отправляем в аналитику — ещё копия
    let for_analytics = position.clone();

    // Оригинал всё ещё наш
    println!("Текущая позиция: {}", position);
    println!("В истории: {}", history_entry);
    println!("Для аналитики: {}", for_analytics);
}
```

## Clone с Vec (списком сделок)

```rust
fn main() {
    // Список ордеров
    let orders = vec![
        String::from("BUY BTC 0.5"),
        String::from("SELL ETH 2.0"),
        String::from("BUY SOL 100"),
    ];

    // Клонируем весь вектор
    let orders_backup = orders.clone();

    // Оба вектора независимы
    println!("Активные ордера: {:?}", orders);
    println!("Бэкап: {:?}", orders_backup);
}
```

**Важно:** При клонировании `Vec<String>` копируется и сам вектор, и все строки внутри него — это **глубокое копирование**.

## Когда использовать Clone

### 1. Сохранение снимка состояния

```rust
fn main() {
    let mut portfolio_value = String::from("$100,000");

    // Сохраняем состояние на начало дня
    let morning_snapshot = portfolio_value.clone();

    // В течение дня значение меняется
    portfolio_value = String::from("$105,000");

    println!("Утром: {}", morning_snapshot);
    println!("Сейчас: {}", portfolio_value);
    println!("Прибыль: +$5,000");
}
```

### 2. Передача в функцию без потери владения

```rust
fn analyze_order(order: String) {
    println!("Анализ: {}", order);
    // order уничтожится после функции
}

fn main() {
    let order = String::from("LIMIT BUY BTC @ 40000");

    // Передаём клон — оригинал остаётся
    analyze_order(order.clone());

    // Можем продолжить использовать
    println!("Ордер активен: {}", order);
}
```

### 3. Работа с несколькими потоками данных

```rust
fn main() {
    let market_data = String::from("BTC=42000,ETH=2800,SOL=100");

    // Для графика
    let for_chart = market_data.clone();

    // Для уведомлений
    let for_alerts = market_data.clone();

    // Для записи в лог
    let for_log = market_data.clone();

    process_chart(for_chart);
    check_alerts(for_alerts);
    write_log(for_log);

    // Оригинал для текущего использования
    println!("Данные: {}", market_data);
}

fn process_chart(data: String) {
    println!("[CHART] {}", data);
}

fn check_alerts(data: String) {
    println!("[ALERT] {}", data);
}

fn write_log(data: String) {
    println!("[LOG] {}", data);
}
```

## Clone vs Move: стоимость операции

```rust
fn main() {
    let big_data = "X".repeat(1_000_000);  // 1 миллион символов

    // Move — мгновенно (просто передаём указатель)
    let moved = big_data;

    // Создадим заново для демонстрации
    let big_data2 = "Y".repeat(1_000_000);

    // Clone — копирует 1 миллион символов!
    // Это затратная операция
    let cloned = big_data2.clone();

    println!("Moved len: {}", moved.len());
    println!("Cloned len: {}", cloned.len());
}
```

**Правило:** Используй `clone()` только когда **действительно нужна** независимая копия. Не злоупотребляй — это затратная операция.

## Типы с Clone

Многие стандартные типы реализуют Clone:

```rust
fn main() {
    // String
    let s1 = String::from("BTC");
    let s2 = s1.clone();

    // Vec
    let v1 = vec![1, 2, 3];
    let v2 = v1.clone();

    // Box
    let b1 = Box::new(42000);
    let b2 = b1.clone();

    println!("String: {} -> {}", s1, s2);
    println!("Vec: {:?} -> {:?}", v1, v2);
    println!("Box: {} -> {}", b1, b2);
}
```

## Практический пример: система ордеров

```rust
fn main() {
    // Создаём ордер
    let order = String::from("LIMIT BUY BTC 0.5 @ 42000");

    // Отправляем на биржу (нужна копия для истории)
    let for_exchange = order.clone();
    send_to_exchange(for_exchange);

    // Сохраняем в историю
    let for_history = order.clone();
    save_to_history(for_history);

    // Показываем пользователю
    display_order(&order);  // Здесь можно использовать ссылку
}

fn send_to_exchange(order: String) {
    println!("[EXCHANGE] Отправлен: {}", order);
}

fn save_to_history(order: String) {
    println!("[HISTORY] Сохранён: {}", order);
}

fn display_order(order: &String) {
    println!("[UI] Отображается: {}", order);
}
```

## Практический пример: торговый журнал

```rust
fn main() {
    let mut trade_log: Vec<String> = Vec::new();

    // Совершаем сделки
    let trade1 = String::from("10:00 BUY BTC 0.1 @ 42000");
    trade_log.push(trade1.clone());  // Копия в лог
    println!("Выполнено: {}", trade1);

    let trade2 = String::from("10:15 SELL ETH 1.0 @ 2800");
    trade_log.push(trade2.clone());  // Копия в лог
    println!("Выполнено: {}", trade2);

    let trade3 = String::from("10:30 BUY SOL 10 @ 100");
    trade_log.push(trade3.clone());  // Копия в лог
    println!("Выполнено: {}", trade3);

    // Выводим журнал
    println!("\n=== Торговый журнал ===");
    for (i, trade) in trade_log.iter().enumerate() {
        println!("#{}: {}", i + 1, trade);
    }
}
```

## Практический пример: бэкап портфеля

```rust
fn main() {
    // Текущий портфель
    let mut portfolio = vec![
        String::from("BTC: 1.5"),
        String::from("ETH: 10.0"),
        String::from("SOL: 100.0"),
    ];

    // Создаём бэкап перед рискованной операцией
    let backup = portfolio.clone();

    println!("Портфель до операции: {:?}", portfolio);

    // Рискованная операция — продаём всё
    portfolio.clear();
    portfolio.push(String::from("USDT: 150000"));

    println!("Портфель после операции: {:?}", portfolio);

    // Откатываемся к бэкапу
    portfolio = backup.clone();
    println!("Откат к бэкапу: {:?}", portfolio);
}
```

## Clone и функции

```rust
// Функция забирает владение
fn process_data(data: String) -> String {
    format!("Processed: {}", data)
}

// Функция работает с ссылкой (предпочтительнее)
fn analyze_data(data: &String) -> usize {
    data.len()
}

fn main() {
    let price_data = String::from("42000.50");

    // Способ 1: клонируем для process_data
    let result = process_data(price_data.clone());
    println!("Result: {}", result);
    println!("Original: {}", price_data);  // Всё ещё доступен

    // Способ 2: используем ссылку
    let length = analyze_data(&price_data);
    println!("Length: {}", length);
    println!("Original: {}", price_data);  // Не тронут
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Clone` | Trait для явного глубокого копирования |
| `.clone()` | Метод создания копии |
| Глубокое копирование | Копируются все вложенные данные |
| Стоимость | Clone может быть затратной операцией |
| Когда использовать | Когда нужны две независимые копии |

## Домашнее задание

1. **Снимок портфеля**
   Создай функцию, которая принимает портфель (Vec<String>) и создаёт его снимок. Затем измени оригинальный портфель и убедись, что снимок не изменился.

2. **Система бэкапов**
   Реализуй простую систему с тремя бэкапами:
   ```
   backup1 = portfolio на момент 10:00
   backup2 = portfolio на момент 12:00
   backup3 = portfolio на момент 14:00
   ```
   Должна быть возможность откатиться к любому бэкапу.

3. **Торговый симулятор**
   Создай список из 5 ордеров. Клонируй его для "симуляции" — выполни все ордера в копии (очисти список), но оригинальный список должен остаться нетронутым.

4. **Анализ производительности**
   Напиши программу, которая создаёт строку из 10,000 символов и клонирует её 100 раз. Подумай: в каких случаях лучше использовать ссылки вместо клонирования?

## Навигация

[← Предыдущий день](../034-move-ownership/ru.md) | [Следующий день →](../036-copy-trait/ru.md)
