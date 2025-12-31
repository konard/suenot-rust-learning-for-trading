# День 59: Проект — Менеджер торговых позиций

## Обзор проекта

Сегодня мы создадим полноценный **Менеджер торговых позиций** — мини-приложение, которое объединит все концепции владения (ownership), изученные за месяц. Этот проект демонстрирует, как система владения Rust помогает создавать безопасные и надёжные торговые системы.

## Аналогия из трейдинга

Представьте биржевой торговый терминал:
- **Позиция** — это актив, которым вы владеете. Как и в Rust, у каждой позиции есть один владелец
- **Передача позиции** — когда вы закрываете позицию, право собственности передаётся (move)
- **Просмотр позиции** — вы можете показать позицию аналитику без передачи владения (borrow)
- **Изменение позиции** — для изменения размера позиции нужен эксклюзивный доступ (mutable borrow)

## Архитектура проекта

```
trading_position_manager/
├── Cargo.toml
└── src/
    └── main.rs
```

## Шаг 1: Определение структур данных

```rust
/// Торговая позиция — базовая единица нашей системы
#[derive(Debug, Clone)]
struct Position {
    symbol: String,      // Тикер актива (владеет строкой)
    quantity: f64,       // Количество
    entry_price: f64,    // Цена входа
    side: Side,          // Направление
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Long,   // Покупка
    Short,  // Продажа
}

/// Портфель — владеет коллекцией позиций
struct Portfolio {
    name: String,
    positions: Vec<Position>,  // Вектор владеет позициями
    balance: f64,
}
```

**Концепция Ownership:** `Portfolio` владеет вектором `positions`, который в свою очередь владеет каждой `Position`. Когда `Portfolio` выходит из области видимости, все позиции автоматически освобождаются.

## Шаг 2: Реализация Position

```rust
impl Position {
    /// Создаёт новую позицию
    fn new(symbol: String, quantity: f64, entry_price: f64, side: Side) -> Self {
        Position {
            symbol,       // Move: String перемещается в структуру
            quantity,
            entry_price,
            side,
        }
    }

    /// Рассчитывает текущую стоимость позиции
    fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    /// Рассчитывает нереализованный P&L
    fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let price_diff = current_price - self.entry_price;
        match self.side {
            Side::Long => price_diff * self.quantity,
            Side::Short => -price_diff * self.quantity,
        }
    }

    /// Рассчитывает P&L в процентах
    fn pnl_percent(&self, current_price: f64) -> f64 {
        let pnl = self.unrealized_pnl(current_price);
        let cost = self.entry_price * self.quantity;
        if cost == 0.0 { 0.0 } else { (pnl / cost) * 100.0 }
    }

    /// Возвращает ссылку на символ (borrowing)
    fn symbol(&self) -> &str {
        &self.symbol  // Возвращаем ссылку, не передавая владение
    }

    /// Изменяет количество (mutable borrow)
    fn adjust_quantity(&mut self, delta: f64) {
        self.quantity += delta;
    }
}
```

**Концепции Ownership:**
- `&self` — иммутабельное заимствование, позволяет читать данные
- `&mut self` — мутабельное заимствование, позволяет изменять данные
- `&str` возврат — заимствуем строку вместо клонирования

## Шаг 3: Реализация Portfolio

```rust
impl Portfolio {
    /// Создаёт новый портфель
    fn new(name: String, initial_balance: f64) -> Self {
        Portfolio {
            name,
            positions: Vec::new(),  // Создаём пустой вектор
            balance: initial_balance,
        }
    }

    /// Открывает новую позицию (перемещает Position в портфель)
    fn open_position(&mut self, position: Position) {
        let cost = position.entry_price * position.quantity;
        if cost <= self.balance {
            self.balance -= cost;
            self.positions.push(position);  // Move: position перемещается в вектор
            println!("Позиция открыта!");
        } else {
            println!("Недостаточно средств!");
            // position будет освобождена здесь, так как не была перемещена
        }
    }

    /// Закрывает позицию по индексу и возвращает её (transfer ownership)
    fn close_position(&mut self, index: usize, current_price: f64) -> Option<Position> {
        if index < self.positions.len() {
            let position = self.positions.remove(index);  // Извлекаем и передаём владение
            let value = position.market_value(current_price);
            self.balance += value;
            Some(position)  // Передаём владение вызывающему коду
        } else {
            None
        }
    }

    /// Возвращает ссылку на позицию (borrowing)
    fn get_position(&self, index: usize) -> Option<&Position> {
        self.positions.get(index)  // Возвращаем ссылку
    }

    /// Возвращает мутабельную ссылку на позицию
    fn get_position_mut(&mut self, index: usize) -> Option<&mut Position> {
        self.positions.get_mut(index)
    }

    /// Итерирует по всем позициям (borrowing)
    fn iter_positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.iter()
    }

    /// Рассчитывает общий P&L портфеля
    fn total_unrealized_pnl(&self, prices: &[(String, f64)]) -> f64 {
        self.positions.iter().map(|pos| {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            pos.unrealized_pnl(current_price)
        }).sum()
    }

    /// Общая стоимость портфеля
    fn total_value(&self, prices: &[(String, f64)]) -> f64 {
        let positions_value: f64 = self.positions.iter().map(|pos| {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            pos.market_value(current_price)
        }).sum();

        self.balance + positions_value
    }
}
```

## Шаг 4: Отображение информации

```rust
/// Форматирует отчёт по позиции (принимает ссылку)
fn format_position_report(position: &Position, current_price: f64) -> String {
    let pnl = position.unrealized_pnl(current_price);
    let pnl_pct = position.pnl_percent(current_price);
    let side_str = match position.side {
        Side::Long => "LONG",
        Side::Short => "SHORT",
    };

    format!(
        "{} {} | Кол-во: {:.4} | Вход: ${:.2} | Текущая: ${:.2} | P&L: ${:.2} ({:.2}%)",
        position.symbol(),
        side_str,
        position.quantity,
        position.entry_price,
        current_price,
        pnl,
        pnl_pct
    )
}

/// Выводит отчёт по портфелю
fn print_portfolio_report(portfolio: &Portfolio, prices: &[(String, f64)]) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  ПОРТФЕЛЬ: {:<54} ║", portfolio.name);
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Свободный баланс: ${:<44.2} ║", portfolio.balance);
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if portfolio.positions.is_empty() {
        println!("║  Нет открытых позиций                                            ║");
    } else {
        for position in portfolio.iter_positions() {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == position.symbol())
                .map(|(_, p)| *p)
                .unwrap_or(position.entry_price);

            let report = format_position_report(position, current_price);
            println!("║  {}  ║", report);
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    let total_pnl = portfolio.total_unrealized_pnl(prices);
    let total_value = portfolio.total_value(prices);
    println!("║  Общий P&L: ${:<53.2} ║", total_pnl);
    println!("║  Общая стоимость: ${:<47.2} ║", total_value);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}
```

## Шаг 5: Главная функция

```rust
fn main() {
    println!("=== Менеджер торговых позиций ===\n");

    // Создаём портфель
    let mut portfolio = Portfolio::new(
        String::from("Основной портфель"),
        100_000.0
    );

    // Текущие цены (в реальной системе — из API)
    let mut prices: Vec<(String, f64)> = vec![
        (String::from("BTC"), 43500.0),
        (String::from("ETH"), 2650.0),
        (String::from("SOL"), 98.0),
    ];

    // Открываем позиции
    println!("--- Открытие позиций ---");

    let btc_position = Position::new(
        String::from("BTC"),
        0.5,
        42000.0,
        Side::Long
    );
    portfolio.open_position(btc_position);  // Move: btc_position перемещена
    // btc_position больше недоступна!

    let eth_position = Position::new(
        String::from("ETH"),
        5.0,
        2500.0,
        Side::Long
    );
    portfolio.open_position(eth_position);

    let sol_position = Position::new(
        String::from("SOL"),
        100.0,
        95.0,
        Side::Short
    );
    portfolio.open_position(sol_position);

    // Отчёт по портфелю
    print_portfolio_report(&portfolio, &prices);

    // Изменяем позицию (mutable borrow)
    println!("\n--- Увеличение позиции ETH ---");
    if let Some(eth_pos) = portfolio.get_position_mut(1) {
        eth_pos.adjust_quantity(2.5);  // Добавляем к позиции
        println!("ETH позиция увеличена до {} единиц", eth_pos.quantity);
    }

    // Обновляем цены
    println!("\n--- Изменение рыночных цен ---");
    prices = vec![
        (String::from("BTC"), 45000.0),
        (String::from("ETH"), 2800.0),
        (String::from("SOL"), 92.0),
    ];

    print_portfolio_report(&portfolio, &prices);

    // Закрываем позицию
    println!("\n--- Закрытие позиции BTC ---");
    if let Some(closed_position) = portfolio.close_position(0, 45000.0) {
        println!(
            "Позиция {} закрыта. Реализованная прибыль: ${:.2}",
            closed_position.symbol(),
            closed_position.unrealized_pnl(45000.0)
        );
        // closed_position будет освобождена в конце этого блока
    }

    // Финальный отчёт
    println!("\n--- Финальный отчёт ---");
    // Обновляем индексы цен после закрытия BTC
    print_portfolio_report(&portfolio, &prices);

    println!("\n=== Программа завершена ===");
}
```

## Ключевые концепции Ownership в проекте

### 1. Владение (Ownership)

```rust
let btc_position = Position::new(...);
portfolio.open_position(btc_position);  // Move
// btc_position недоступна — владение передано portfolio
```

### 2. Заимствование (Borrowing)

```rust
// Иммутабельное заимствование — можно читать
fn print_portfolio_report(portfolio: &Portfolio, prices: &[(String, f64)])

// Мутабельное заимствование — можно изменять
fn adjust_quantity(&mut self, delta: f64)
```

### 3. Время жизни (Lifetimes)

```rust
// Ссылка на строку внутри Position
fn symbol(&self) -> &str {
    &self.symbol  // Ссылка живёт пока жива Position
}
```

### 4. Передача владения (Move)

```rust
fn close_position(&mut self, index: usize, price: f64) -> Option<Position> {
    let position = self.positions.remove(index);  // Move из вектора
    Some(position)  // Move вызывающему коду
}
```

## Упражнения

### Упражнение 1: Стоп-лосс

Добавьте поле `stop_loss: Option<f64>` в `Position` и метод проверки срабатывания стопа.

```rust
impl Position {
    fn is_stopped_out(&self, current_price: f64) -> bool {
        // Реализуйте логику
        todo!()
    }
}
```

### Упражнение 2: История сделок

Создайте структуру `TradeHistory`, которая хранит закрытые позиции:

```rust
struct TradeHistory {
    trades: Vec<ClosedTrade>,
}

struct ClosedTrade {
    position: Position,  // Владеет позицией
    close_price: f64,
    close_time: String,
}
```

### Упражнение 3: Поиск позиции

Реализуйте метод поиска позиции по символу:

```rust
impl Portfolio {
    fn find_by_symbol(&self, symbol: &str) -> Option<&Position> {
        // Верните ссылку на позицию, не передавая владение
        todo!()
    }
}
```

### Упражнение 4: Риск-менеджмент

Добавьте проверку максимального риска при открытии позиции:

```rust
impl Portfolio {
    fn open_position_with_risk_check(
        &mut self,
        position: Position,
        max_risk_percent: f64
    ) -> Result<(), String> {
        // Проверьте, не превышает ли позиция допустимый риск
        todo!()
    }
}
```

## Домашнее задание

1. **Расширьте Portfolio:**
   - Добавьте метод `close_all_positions()`, который закрывает все позиции
   - Реализуйте `clone_positions()`, который возвращает клон вектора позиций

2. **Создайте систему ордеров:**
   - Структура `Order` с типом (Market, Limit)
   - Метод `place_order()` в Portfolio
   - Логика исполнения ордеров

3. **Реализуйте статистику:**
   - Общее количество сделок
   - Средний P&L
   - Винрейт (процент прибыльных сделок)

4. **Добавьте сериализацию:**
   - Сохранение портфеля в строку (формат на ваш выбор)
   - Загрузка портфеля из строки

## Полный код проекта

Соберите все части в файл `src/main.rs` и запустите:

```bash
cargo new trading_position_manager
cd trading_position_manager
# Скопируйте код в src/main.rs
cargo run
```

## Что мы изучили

| Концепция | Применение в проекте |
|-----------|---------------------|
| Ownership | Position принадлежит Portfolio |
| Move | Передача позиции при открытии/закрытии |
| Borrow | Получение ссылок на позиции для чтения |
| Mutable Borrow | Изменение количества в позиции |
| Lifetimes | Ссылка на symbol живёт пока жива Position |
| Vec ownership | Вектор владеет всеми своими элементами |

## Навигация

[← Предыдущий день](../058-ownership-review/ru.md) | [Следующий день →](../060-month2-summary/ru.md)
