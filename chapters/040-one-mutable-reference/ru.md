# День 40: Правило одной изменяемой ссылки

## Аналогия из трейдинга

Представь торговый терминал, где открыт ордер на редактирование. Если **два трейдера одновременно** начнут менять параметры одного ордера — один ставит цену 42000, другой 43000 — результат будет непредсказуем. Какая цена окажется в итоге?

В трейдинговых системах это называется **race condition** (состояние гонки), и это одна из самых опасных ошибок. Rust **на уровне компилятора** запрещает такие ситуации: в один момент времени только **один** может редактировать данные.

## Правило Rust

> **В любой момент времени может существовать либо одна изменяемая ссылка `&mut T`, либо любое количество неизменяемых ссылок `&T`, но не оба варианта одновременно.**

Это правило предотвращает:
- **Data races** (гонки данных) — одновременное чтение и запись
- **Undefined behavior** — неопределённое поведение программы
- **Трудноуловимые баги** — ошибки, которые появляются "иногда"

## Базовый пример: изменение цены ордера

```rust
fn main() {
    let mut order_price = 42000.0;

    // Создаём одну изменяемую ссылку — всё ok
    let price_ref = &mut order_price;
    *price_ref = 42500.0;

    println!("Новая цена: {}", order_price);
}
```

## Ошибка: две изменяемые ссылки

```rust
fn main() {
    let mut order_price = 42000.0;

    let ref1 = &mut order_price;
    let ref2 = &mut order_price;  // ОШИБКА КОМПИЛЯЦИИ!

    *ref1 = 42500.0;
    *ref2 = 43000.0;
}
```

Компилятор выдаст ошибку:
```
error[E0499]: cannot borrow `order_price` as mutable more than once at a time
```

## Почему это важно для трейдинга

### Сценарий 1: Управление портфелем

```rust
struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

fn main() {
    let mut portfolio = Portfolio {
        balance: 100_000.0,
        positions: vec![],
    };

    // Один "редактор" портфеля — безопасно
    let editor = &mut portfolio;
    editor.balance -= 21_000.0;
    editor.positions.push(Position {
        symbol: String::from("BTC"),
        quantity: 0.5,
        entry_price: 42000.0,
    });

    println!("Баланс: ${:.2}", portfolio.balance);
    println!("Позиций: {}", portfolio.positions.len());
}
```

### Сценарий 2: Модификация ордера

```rust
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    status: String,
}

fn update_order_price(order: &mut Order, new_price: f64) {
    println!("Ордер #{}: цена {} -> {}", order.id, order.price, new_price);
    order.price = new_price;
}

fn update_order_quantity(order: &mut Order, new_quantity: f64) {
    println!("Ордер #{}: количество {} -> {}", order.id, order.quantity, new_quantity);
    order.quantity = new_quantity;
}

fn main() {
    let mut order = Order {
        id: 12345,
        symbol: String::from("ETH/USDT"),
        price: 2500.0,
        quantity: 10.0,
        status: String::from("PENDING"),
    };

    // Последовательное редактирование — безопасно
    update_order_price(&mut order, 2550.0);
    update_order_quantity(&mut order, 15.0);

    println!("Итоговый ордер: {} @ ${}", order.quantity, order.price);
}
```

## Область действия изменяемых ссылок

Ссылка "живёт" до последнего использования, не до конца блока:

```rust
fn main() {
    let mut price = 42000.0;

    let ref1 = &mut price;
    *ref1 = 42500.0;
    println!("После ref1: {}", ref1);  // Последнее использование ref1

    // ref1 больше не используется — можно создать новую изменяемую ссылку
    let ref2 = &mut price;
    *ref2 = 43000.0;
    println!("После ref2: {}", ref2);
}
```

Это называется **Non-Lexical Lifetimes (NLL)** — Rust понимает, когда ссылка реально используется.

## Практический пример: Анализ цен

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    // Анализируем цены (только читаем)
    let avg = calculate_average(&prices);
    let max = find_max(&prices);
    println!("Среднее: {:.2}, Максимум: {:.2}", avg, max);

    // Теперь модифицируем — добавляем новую цену
    add_price(&mut prices, 42300.0);

    // Снова можем читать
    let new_avg = calculate_average(&prices);
    println!("Новое среднее: {:.2}", new_avg);
}

fn calculate_average(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}

fn find_max(prices: &[f64]) -> f64 {
    prices.iter().cloned().fold(f64::MIN, f64::max)
}

fn add_price(prices: &mut Vec<f64>, price: f64) {
    prices.push(price);
    println!("Добавлена цена: {}", price);
}
```

## Риск-менеджмент: безопасное обновление лимитов

```rust
struct RiskLimits {
    max_position_size: f64,
    max_daily_loss: f64,
    max_leverage: f64,
}

fn main() {
    let mut limits = RiskLimits {
        max_position_size: 100_000.0,
        max_daily_loss: 5_000.0,
        max_leverage: 10.0,
    };

    // Один риск-менеджер обновляет лимиты
    update_risk_limits(&mut limits, 2.0);

    println!("Новый размер позиции: ${:.2}", limits.max_position_size);
    println!("Новый дневной лимит убытка: ${:.2}", limits.max_daily_loss);
}

fn update_risk_limits(limits: &mut RiskLimits, multiplier: f64) {
    limits.max_position_size *= multiplier;
    limits.max_daily_loss *= multiplier;
    println!("Лимиты обновлены с множителем {}", multiplier);
}
```

## Управление ордерами: очередь на исполнение

```rust
struct OrderQueue {
    orders: Vec<Order>,
    total_volume: f64,
}

struct Order {
    id: u64,
    price: f64,
    quantity: f64,
}

impl OrderQueue {
    fn new() -> Self {
        OrderQueue {
            orders: vec![],
            total_volume: 0.0,
        }
    }

    fn add_order(&mut self, order: Order) {
        self.total_volume += order.price * order.quantity;
        self.orders.push(order);
    }

    fn remove_order(&mut self, id: u64) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            let order = self.orders.remove(pos);
            self.total_volume -= order.price * order.quantity;
            Some(order)
        } else {
            None
        }
    }

    fn get_stats(&self) -> (usize, f64) {
        (self.orders.len(), self.total_volume)
    }
}

fn main() {
    let mut queue = OrderQueue::new();

    // Добавляем ордера (изменяемый доступ)
    queue.add_order(Order { id: 1, price: 42000.0, quantity: 0.5 });
    queue.add_order(Order { id: 2, price: 42100.0, quantity: 1.0 });
    queue.add_order(Order { id: 3, price: 41900.0, quantity: 0.25 });

    // Читаем статистику (неизменяемый доступ)
    let (count, volume) = queue.get_stats();
    println!("Ордеров в очереди: {}, Объём: ${:.2}", count, volume);

    // Удаляем ордер (изменяемый доступ)
    if let Some(removed) = queue.remove_order(2) {
        println!("Удалён ордер #{} на ${:.2}", removed.id, removed.price);
    }

    let (count, volume) = queue.get_stats();
    println!("После удаления: {} ордеров, ${:.2}", count, volume);
}
```

## Паттерн: разделение чтения и записи

```rust
struct TradingAccount {
    balance: f64,
    equity: f64,
    margin_used: f64,
}

impl TradingAccount {
    // Методы только для чтения — &self
    fn available_margin(&self) -> f64 {
        self.equity - self.margin_used
    }

    fn margin_level(&self) -> f64 {
        if self.margin_used > 0.0 {
            (self.equity / self.margin_used) * 100.0
        } else {
            f64::INFINITY
        }
    }

    // Методы для изменения — &mut self
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
        self.equity += amount;
        println!("Пополнение: +${:.2}", amount);
    }

    fn use_margin(&mut self, amount: f64) -> bool {
        if amount <= self.available_margin() {
            self.margin_used += amount;
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 10_000.0,
        equity: 10_500.0,
        margin_used: 2_000.0,
    };

    // Читаем данные
    println!("Доступная маржа: ${:.2}", account.available_margin());
    println!("Уровень маржи: {:.1}%", account.margin_level());

    // Модифицируем
    account.deposit(5_000.0);

    // Снова читаем
    println!("Новая доступная маржа: ${:.2}", account.available_margin());
}
```

## Стратегия трейдинга: безопасное обновление состояния

```rust
struct TradingStrategy {
    name: String,
    is_active: bool,
    current_position: f64,
    total_pnl: f64,
    trade_count: u32,
}

impl TradingStrategy {
    fn new(name: &str) -> Self {
        TradingStrategy {
            name: String::from(name),
            is_active: false,
            current_position: 0.0,
            total_pnl: 0.0,
            trade_count: 0,
        }
    }

    fn activate(&mut self) {
        self.is_active = true;
        println!("Стратегия '{}' активирована", self.name);
    }

    fn execute_trade(&mut self, quantity: f64, pnl: f64) {
        if !self.is_active {
            println!("Стратегия не активна!");
            return;
        }
        self.current_position += quantity;
        self.total_pnl += pnl;
        self.trade_count += 1;
        println!("Сделка #{}: кол-во {}, PnL ${:.2}",
                 self.trade_count, quantity, pnl);
    }

    fn get_summary(&self) -> String {
        format!("{}: {} сделок, PnL ${:.2}",
                self.name, self.trade_count, self.total_pnl)
    }
}

fn main() {
    let mut strategy = TradingStrategy::new("SMA Crossover");

    strategy.activate();
    strategy.execute_trade(0.5, 150.0);
    strategy.execute_trade(-0.5, 0.0);
    strategy.execute_trade(1.0, -50.0);

    println!("\n{}", strategy.get_summary());
}
```

## Что мы узнали

| Правило | Описание | Аналогия |
|---------|----------|----------|
| Одна `&mut` | Только одна изменяемая ссылка одновременно | Один редактор ордера |
| Много `&` | Сколько угодно неизменяемых ссылок | Много зрителей терминала |
| Не смешивать | `&mut` и `&` не могут сосуществовать | Нельзя смотреть пока редактируют |
| NLL | Ссылка заканчивается при последнем использовании | Редактор освобождает доступ |

## Домашнее задание

1. **Портфель с проверкой**: Создай структуру `Portfolio` с методами `add_position(&mut self, ...)` и `get_total_value(&self) -> f64`. Убедись, что компилятор правильно отслеживает ссылки.

2. **Журнал сделок**: Напиши функцию `record_trade(journal: &mut TradeJournal, trade: Trade)` и функцию `analyze_journal(journal: &TradeJournal) -> Stats`. Продемонстрируй последовательное использование `&mut` и `&`.

3. **Риск-калькулятор**: Создай структуру `RiskCalculator`, которая хранит настройки риска. Напиши метод `update_settings(&mut self, ...)` и метод `calculate_position_size(&self, ...) -> f64`. Покажи, как правило одной изменяемой ссылки защищает от одновременного изменения настроек.

4. **Многопользовательский доступ** (продвинуто): Попробуй создать ситуацию, где нужны две изменяемые ссылки. Какую ошибку выдаёт компилятор? Как можно переписать код, чтобы он компилировался?

## Навигация

[← Предыдущий день](../039-mutable-references/ru.md) | [Следующий день →](../041-no-mixing-references/ru.md)
