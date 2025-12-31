# День 39: Изменяемые ссылки — редактируем чужой ордер

## Аналогия из трейдинга

Представьте, что вы работаете в трейдинговой компании. У вашего коллеги есть ордер, который нужно модифицировать — например, изменить цену или объём. Есть два варианта:

1. **Скопировать ордер** — сделать копию, изменить её и вернуть (неэффективно)
2. **Получить доступ к редактированию** — коллега даёт вам временное право изменить его ордер напрямую

В Rust второй подход реализуется через **изменяемые ссылки** (`&mut`). Это как получить ключ редактирования к чужому документу — вы можете его менять, но только пока вам разрешили.

## Обычные ссылки vs изменяемые

```rust
fn main() {
    let mut price = 42000.0;

    // Обычная ссылка — только чтение
    let price_ref = &price;
    println!("Текущая цена: ${}", price_ref);
    // price_ref = 43000.0;  // ОШИБКА! Нельзя изменить через &

    // Изменяемая ссылка — можно менять
    let price_mut = &mut price;
    *price_mut = 43000.0;  // OK! Меняем значение через &mut
    println!("Новая цена: ${}", price_mut);
}
```

## Синтаксис изменяемых ссылок

```rust
fn main() {
    let mut balance = 10000.0;

    // Создаём изменяемую ссылку
    let balance_ref = &mut balance;

    // Используем * для доступа к значению (разыменование)
    *balance_ref += 500.0;
    *balance_ref -= 100.0;

    println!("Баланс: ${}", balance_ref);
}
```

- `&mut` — создаёт изменяемую ссылку
- `*` — разыменование (доступ к значению)
- Исходная переменная должна быть `mut`

## Изменяемые ссылки в функциях

```rust
fn main() {
    let mut order_price = 42000.0;
    let mut order_quantity = 0.5;

    println!("До: цена=${}, объём={}", order_price, order_quantity);

    // Передаём изменяемые ссылки в функцию
    modify_order(&mut order_price, &mut order_quantity);

    println!("После: цена=${}, объём={}", order_price, order_quantity);
}

fn modify_order(price: &mut f64, quantity: &mut f64) {
    *price = 42500.0;     // Меняем цену
    *quantity = 0.75;     // Меняем объём
}
```

## Практический пример: обновление позиции

```rust
fn main() {
    let mut position_size = 1.0;  // Текущий размер позиции
    let mut average_price = 42000.0;  // Средняя цена

    println!("Позиция: {} BTC @ ${:.2}", position_size, average_price);

    // Докупаем ещё
    add_to_position(&mut position_size, &mut average_price, 0.5, 43000.0);
    println!("После докупки: {} BTC @ ${:.2}", position_size, average_price);

    // Частичная продажа
    reduce_position(&mut position_size, 0.3);
    println!("После продажи: {} BTC @ ${:.2}", position_size, average_price);
}

fn add_to_position(
    size: &mut f64,
    avg_price: &mut f64,
    add_size: f64,
    add_price: f64
) {
    // Рассчитываем новую среднюю цену
    let total_value = (*size * *avg_price) + (add_size * add_price);
    *size += add_size;
    *avg_price = total_value / *size;
}

fn reduce_position(size: &mut f64, sell_size: f64) {
    *size -= sell_size;
}
```

## Изменяемые ссылки на структуры

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    is_active: bool,
}

fn main() {
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        is_active: true,
    };

    println!("Ордер: {} {} @ ${}", order.symbol, order.quantity, order.price);

    // Передаём изменяемую ссылку на структуру
    update_order_price(&mut order, 42500.0);
    println!("После обновления: {} @ ${}", order.symbol, order.price);

    cancel_order(&mut order);
    println!("Статус: активен = {}", order.is_active);
}

fn update_order_price(order: &mut Order, new_price: f64) {
    order.price = new_price;  // Автоматическое разыменование для полей
}

fn cancel_order(order: &mut Order) {
    order.is_active = false;
}
```

## Изменение Vec через ссылку

```rust
fn main() {
    let mut portfolio: Vec<(&str, f64)> = vec![
        ("BTC", 0.5),
        ("ETH", 2.0),
        ("SOL", 10.0),
    ];

    println!("Портфель до:");
    print_portfolio(&portfolio);

    // Добавляем новый актив
    add_asset(&mut portfolio, "AVAX", 5.0);

    // Обновляем количество BTC
    update_asset_quantity(&mut portfolio, "BTC", 0.75);

    println!("\nПортфель после:");
    print_portfolio(&portfolio);
}

fn add_asset(portfolio: &mut Vec<(&str, f64)>, symbol: &str, quantity: f64) {
    portfolio.push((symbol, quantity));
}

fn update_asset_quantity(portfolio: &mut Vec<(&str, f64)>, symbol: &str, new_qty: f64) {
    for asset in portfolio.iter_mut() {
        if asset.0 == symbol {
            asset.1 = new_qty;
            return;
        }
    }
}

fn print_portfolio(portfolio: &Vec<(&str, f64)>) {
    for (symbol, qty) in portfolio {
        println!("  {}: {}", symbol, qty);
    }
}
```

## Практический пример: риск-менеджмент

```rust
struct RiskManager {
    max_position_size: f64,
    current_exposure: f64,
    daily_loss_limit: f64,
    current_daily_loss: f64,
}

fn main() {
    let mut risk = RiskManager {
        max_position_size: 10.0,
        current_exposure: 0.0,
        daily_loss_limit: 500.0,
        current_daily_loss: 0.0,
    };

    println!("=== РИСК-МЕНЕДЖМЕНТ ===\n");

    // Пытаемся открыть позицию
    if try_open_position(&mut risk, 5.0) {
        println!("Позиция 5.0 открыта");
    }

    // Пытаемся открыть ещё
    if try_open_position(&mut risk, 7.0) {
        println!("Позиция 7.0 открыта");
    } else {
        println!("Позиция 7.0 отклонена — превышение лимита");
    }

    // Фиксируем убыток
    record_loss(&mut risk, 200.0);
    println!("Зафиксирован убыток: $200");

    // Проверяем, можем ли торговать
    if can_trade(&risk) {
        println!("Торговля разрешена");
    }

    // Большой убыток
    record_loss(&mut risk, 350.0);
    println!("Зафиксирован убыток: $350");

    if !can_trade(&risk) {
        println!("СТОП! Достигнут дневной лимит убытков");
    }
}

fn try_open_position(risk: &mut RiskManager, size: f64) -> bool {
    if risk.current_exposure + size <= risk.max_position_size {
        risk.current_exposure += size;
        true
    } else {
        false
    }
}

fn record_loss(risk: &mut RiskManager, amount: f64) {
    risk.current_daily_loss += amount;
}

fn can_trade(risk: &RiskManager) -> bool {
    risk.current_daily_loss < risk.daily_loss_limit
}
```

## Изменение через метод

```rust
struct TradingAccount {
    balance: f64,
    positions: Vec<String>,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        TradingAccount {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    // &self — неизменяемый доступ
    fn get_balance(&self) -> f64 {
        self.balance
    }

    // &mut self — изменяемый доступ
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: f64) -> bool {
        if amount <= self.balance {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    fn open_position(&mut self, symbol: &str) {
        self.positions.push(String::from(symbol));
    }

    fn close_position(&mut self, symbol: &str) {
        self.positions.retain(|s| s != symbol);
    }
}

fn main() {
    let mut account = TradingAccount::new(10000.0);

    println!("Начальный баланс: ${}", account.get_balance());

    account.deposit(5000.0);
    println!("После депозита: ${}", account.get_balance());

    account.open_position("BTC/USDT");
    account.open_position("ETH/USDT");
    println!("Позиции: {:?}", account.positions);

    account.close_position("BTC/USDT");
    println!("После закрытия BTC: {:?}", account.positions);

    if account.withdraw(20000.0) {
        println!("Вывод успешен");
    } else {
        println!("Недостаточно средств");
    }
}
```

## Изменяемая ссылка на часть данных

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    println!("Цены до: {:?}", prices);

    // Получаем изменяемую ссылку на последнюю цену
    if let Some(last) = prices.last_mut() {
        *last = 42300.0;
    }

    // Получаем изменяемую ссылку на первую цену
    if let Some(first) = prices.first_mut() {
        *first = 41800.0;
    }

    println!("Цены после: {:?}", prices);

    // Изменяем все цены
    for price in prices.iter_mut() {
        *price *= 1.01;  // Увеличиваем на 1%
    }

    println!("После роста на 1%: {:?}", prices);
}
```

## Swap — обмен значениями

```rust
fn main() {
    let mut bid = 41950.0;
    let mut ask = 42050.0;

    println!("До swap: bid={}, ask={}", bid, ask);

    // Меняем местами через изменяемые ссылки
    swap_prices(&mut bid, &mut ask);

    println!("После swap: bid={}, ask={}", bid, ask);

    // Есть встроенная функция std::mem::swap
    std::mem::swap(&mut bid, &mut ask);
    println!("После std::mem::swap: bid={}, ask={}", bid, ask);
}

fn swap_prices(a: &mut f64, b: &mut f64) {
    let temp = *a;
    *a = *b;
    *b = temp;
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&mut T` | Изменяемая ссылка |
| `*ref` | Разыменование для доступа к значению |
| `fn f(x: &mut T)` | Функция принимает изменяемую ссылку |
| `&mut self` | Метод с изменяемым доступом к self |
| `.iter_mut()` | Итератор с изменяемыми ссылками |
| `last_mut()`, `first_mut()` | Изменяемый доступ к элементам |

## Упражнения

1. **Управление балансом**
   Напиши функции для работы с балансом:
   ```rust
   fn deposit(balance: &mut f64, amount: f64) { ... }
   fn withdraw(balance: &mut f64, amount: f64) -> bool { ... }
   fn apply_fee(balance: &mut f64, fee_percent: f64) { ... }
   ```

2. **Обновление ордера**
   Создай структуру `LimitOrder` и функции для её изменения:
   ```rust
   fn update_price(order: &mut LimitOrder, new_price: f64) { ... }
   fn update_quantity(order: &mut LimitOrder, new_qty: f64) { ... }
   fn cancel(order: &mut LimitOrder) { ... }
   ```

3. **Нормализация портфеля**
   Напиши функцию, которая приводит веса активов в портфеле к сумме 100%:
   ```rust
   fn normalize_weights(weights: &mut Vec<f64>) { ... }
   ```

4. **Stop-loss менеджер**
   Реализуй систему с trailing stop-loss:
   ```rust
   fn update_trailing_stop(
       stop_price: &mut f64,
       current_price: f64,
       trail_percent: f64
   ) { ... }
   ```

## Домашнее задание

1. Создай структуру `Portfolio` с методами, использующими `&mut self`:
   - `add_asset()` — добавить актив
   - `remove_asset()` — удалить актив
   - `update_quantity()` — изменить количество
   - `rebalance()` — ребалансировать по целевым весам

2. Реализуй функцию пересчёта средней цены позиции при добавлении:
   ```rust
   fn add_to_position(
       current_qty: &mut f64,
       avg_price: &mut f64,
       add_qty: f64,
       add_price: f64
   )
   ```

3. Напиши функцию, которая применяет комиссию к списку сделок:
   ```rust
   fn apply_commission(trades: &mut Vec<Trade>, commission_rate: f64)
   ```

4. Создай систему управления рисками с методами:
   - `check_and_update_exposure()` — проверить и обновить экспозицию
   - `record_pnl()` — записать результат сделки
   - `reset_daily_stats()` — сбросить дневную статистику

## Навигация

[← Предыдущий день](../038-borrowing/ru.md) | [Следующий день →](../040-one-mutable-reference-rule/ru.md)
