# День 58: Weak — слабые ссылки на связанные ордера

## Аналогия из трейдинга

В трейдинге ордера часто связаны между собой:
- **Основной ордер** на покупку BTC
- **Take Profit ордер** — автоматически продаёт при достижении целевой цены
- **Stop Loss ордер** — автоматически продаёт при достижении уровня убытка

Эти связи создают потенциальную проблему: если основной ордер исполнен и удалён, связанные ордера должны "знать" об этом и не пытаться обращаться к уже несуществующим данным.

**Weak (слабые ссылки)** решают эту проблему: они позволяют ссылаться на данные, не предотвращая их удаление.

## Проблема циклических ссылок

```rust
use std::rc::Rc;
use std::cell::RefCell;

// ❌ ПРОБЛЕМА: циклические ссылки с Rc
struct Order {
    id: String,
    price: f64,
    related_orders: Vec<Rc<RefCell<Order>>>,  // Связанные ордера
}

fn main() {
    let order1 = Rc::new(RefCell::new(Order {
        id: "ORD-001".to_string(),
        price: 42000.0,
        related_orders: vec![],
    }));

    let order2 = Rc::new(RefCell::new(Order {
        id: "ORD-002".to_string(),
        price: 43000.0,
        related_orders: vec![],
    }));

    // Создаём циклическую ссылку
    order1.borrow_mut().related_orders.push(Rc::clone(&order2));
    order2.borrow_mut().related_orders.push(Rc::clone(&order1));

    // ⚠️ Память никогда не освободится!
    // Счётчик ссылок обоих ордеров никогда не станет 0
}
```

## Решение с Weak

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ✅ РЕШЕНИЕ: слабые ссылки не увеличивают счётчик
struct Order {
    id: String,
    price: f64,
    quantity: f64,
    // Слабые ссылки на связанные ордера
    take_profit: Option<Weak<RefCell<Order>>>,
    stop_loss: Option<Weak<RefCell<Order>>>,
    // Слабая ссылка на родительский ордер
    parent_order: Option<Weak<RefCell<Order>>>,
}

impl Order {
    fn new(id: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Order {
            id: id.to_string(),
            price,
            quantity,
            take_profit: None,
            stop_loss: None,
            parent_order: None,
        }))
    }

    fn set_take_profit(&mut self, tp: &Rc<RefCell<Order>>) {
        self.take_profit = Some(Rc::downgrade(tp));
    }

    fn set_stop_loss(&mut self, sl: &Rc<RefCell<Order>>) {
        self.stop_loss = Some(Rc::downgrade(sl));
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<Order>>) {
        self.parent_order = Some(Rc::downgrade(parent));
    }
}

fn main() {
    // Основной ордер на покупку
    let main_order = Order::new("BUY-001", 42000.0, 0.5);

    // Take Profit ордер
    let tp_order = Order::new("TP-001", 45000.0, 0.5);

    // Stop Loss ордер
    let sl_order = Order::new("SL-001", 40000.0, 0.5);

    // Связываем ордера
    {
        let mut main = main_order.borrow_mut();
        main.set_take_profit(&tp_order);
        main.set_stop_loss(&sl_order);
    }

    {
        tp_order.borrow_mut().set_parent(&main_order);
        sl_order.borrow_mut().set_parent(&main_order);
    }

    println!("Ордера созданы и связаны");
    println!("Main order refs: {}", Rc::strong_count(&main_order));
    println!("TP order refs: {}", Rc::strong_count(&tp_order));
    // Weak ссылки не увеличивают счётчик!
}
```

## Rc::downgrade и Weak::upgrade

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

#[derive(Debug)]
struct RiskMonitor {
    // Слабая ссылка на позицию
    position: Weak<RefCell<Position>>,
    max_loss_percent: f64,
}

impl RiskMonitor {
    fn new(position: &Rc<RefCell<Position>>, max_loss: f64) -> Self {
        RiskMonitor {
            // downgrade: Rc -> Weak
            position: Rc::downgrade(position),
            max_loss_percent: max_loss,
        }
    }

    fn check_risk(&self, current_price: f64) {
        // upgrade: Weak -> Option<Rc>
        match self.position.upgrade() {
            Some(pos) => {
                let position = pos.borrow();
                let pnl_percent = (current_price - position.entry_price)
                    / position.entry_price * 100.0;

                if pnl_percent < -self.max_loss_percent {
                    println!(
                        "⚠️ RISK ALERT: {} loss {:.2}% exceeds limit {:.1}%",
                        position.symbol, pnl_percent.abs(), self.max_loss_percent
                    );
                } else {
                    println!(
                        "✅ {} P&L: {:.2}% (limit: {:.1}%)",
                        position.symbol, pnl_percent, self.max_loss_percent
                    );
                }
            }
            None => {
                // Позиция уже закрыта/удалена
                println!("Position no longer exists");
            }
        }
    }
}

fn main() {
    let position = Rc::new(RefCell::new(Position {
        symbol: "BTC/USDT".to_string(),
        size: 0.5,
        entry_price: 42000.0,
    }));

    let monitor = RiskMonitor::new(&position, 5.0);

    // Проверяем риск при разных ценах
    monitor.check_risk(41000.0);  // -2.38%
    monitor.check_risk(39000.0);  // -7.14% - превышает лимит!

    // Удаляем позицию
    drop(position);

    // Мониторинг безопасно обрабатывает удалённую позицию
    monitor.check_risk(40000.0);  // Position no longer exists
}
```

## Система связанных ордеров

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderType {
    Market,
    Limit,
    TakeProfit,
    StopLoss,
}

struct LinkedOrder {
    id: String,
    order_type: OrderType,
    symbol: String,
    price: f64,
    quantity: f64,
    status: OrderStatus,
    // Слабые ссылки для избежания циклов
    parent: Option<Weak<RefCell<LinkedOrder>>>,
    children: Vec<Weak<RefCell<LinkedOrder>>>,
}

impl LinkedOrder {
    fn new(id: &str, order_type: OrderType, symbol: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(LinkedOrder {
            id: id.to_string(),
            order_type,
            symbol: symbol.to_string(),
            price,
            quantity,
            status: OrderStatus::Pending,
            parent: None,
            children: vec![],
        }))
    }

    fn link_child(&mut self, child: &Rc<RefCell<LinkedOrder>>) {
        self.children.push(Rc::downgrade(child));
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<LinkedOrder>>) {
        self.parent = Some(Rc::downgrade(parent));
    }

    fn fill(&mut self) {
        self.status = OrderStatus::Filled;
        println!("✅ Order {} filled at ${:.2}", self.id, self.price);
    }

    fn cancel(&mut self) {
        self.status = OrderStatus::Cancelled;
        println!("❌ Order {} cancelled", self.id);
    }

    fn cancel_children(&self) {
        for weak_child in &self.children {
            if let Some(child) = weak_child.upgrade() {
                let mut child_order = child.borrow_mut();
                if child_order.status == OrderStatus::Pending {
                    child_order.cancel();
                }
            }
        }
    }

    fn check_parent_status(&self) -> Option<OrderStatus> {
        self.parent.as_ref().and_then(|weak_parent| {
            weak_parent.upgrade().map(|parent| {
                parent.borrow().status.clone()
            })
        })
    }
}

fn main() {
    println!("=== Система связанных ордеров ===\n");

    // Создаём основной ордер на покупку
    let buy_order = LinkedOrder::new(
        "BUY-001",
        OrderType::Market,
        "BTC/USDT",
        42000.0,
        0.5
    );

    // Создаём Take Profit
    let tp_order = LinkedOrder::new(
        "TP-001",
        OrderType::TakeProfit,
        "BTC/USDT",
        45000.0,
        0.5
    );

    // Создаём Stop Loss
    let sl_order = LinkedOrder::new(
        "SL-001",
        OrderType::StopLoss,
        "BTC/USDT",
        40000.0,
        0.5
    );

    // Связываем ордера
    {
        let mut buy = buy_order.borrow_mut();
        buy.link_child(&tp_order);
        buy.link_child(&sl_order);
    }
    tp_order.borrow_mut().set_parent(&buy_order);
    sl_order.borrow_mut().set_parent(&buy_order);

    // Основной ордер исполнен
    buy_order.borrow_mut().fill();

    // Симуляция: цена достигла Take Profit
    println!("\n--- Цена достигла Take Profit ---");
    tp_order.borrow_mut().fill();

    // При исполнении TP, отменяем SL
    buy_order.borrow().cancel_children();

    // Проверяем статус родительского ордера из дочернего
    if let Some(status) = sl_order.borrow().check_parent_status() {
        println!("\nParent order status: {:?}", status);
    }
}
```

## OCO (One-Cancels-Other) ордера

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum OcoStatus {
    Active,
    OneFilled,
    AllCancelled,
}

struct OcoOrder {
    id: String,
    take_profit: Option<Weak<RefCell<LimitOrder>>>,
    stop_loss: Option<Weak<RefCell<LimitOrder>>>,
    status: OcoStatus,
}

struct LimitOrder {
    id: String,
    price: f64,
    quantity: f64,
    is_filled: bool,
    oco_group: Option<Weak<RefCell<OcoOrder>>>,
}

impl LimitOrder {
    fn new(id: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(LimitOrder {
            id: id.to_string(),
            price,
            quantity,
            is_filled: false,
            oco_group: None,
        }))
    }

    fn fill(&mut self) {
        self.is_filled = true;
        println!("Order {} filled at ${:.2}", self.id, self.price);

        // Уведомляем OCO группу
        if let Some(ref oco_weak) = self.oco_group {
            if let Some(oco) = oco_weak.upgrade() {
                oco.borrow_mut().on_order_filled(&self.id);
            }
        }
    }
}

impl OcoOrder {
    fn new(
        id: &str,
        tp: &Rc<RefCell<LimitOrder>>,
        sl: &Rc<RefCell<LimitOrder>>
    ) -> Rc<RefCell<Self>> {
        let oco = Rc::new(RefCell::new(OcoOrder {
            id: id.to_string(),
            take_profit: Some(Rc::downgrade(tp)),
            stop_loss: Some(Rc::downgrade(sl)),
            status: OcoStatus::Active,
        }));

        // Связываем ордера с OCO группой
        tp.borrow_mut().oco_group = Some(Rc::downgrade(&oco));
        sl.borrow_mut().oco_group = Some(Rc::downgrade(&oco));

        oco
    }

    fn on_order_filled(&mut self, filled_id: &str) {
        self.status = OcoStatus::OneFilled;

        // Отменяем другой ордер
        let other = if self.take_profit.as_ref()
            .and_then(|w| w.upgrade())
            .map(|o| o.borrow().id.clone())
            .as_deref() == Some(filled_id)
        {
            &self.stop_loss
        } else {
            &self.take_profit
        };

        if let Some(ref weak_order) = other {
            if let Some(order) = weak_order.upgrade() {
                println!("OCO: Cancelling order {}", order.borrow().id);
            }
        }
    }
}

fn main() {
    println!("=== OCO (One-Cancels-Other) ===\n");

    let tp = LimitOrder::new("TP-001", 45000.0, 0.5);
    let sl = LimitOrder::new("SL-001", 40000.0, 0.5);

    let _oco = OcoOrder::new("OCO-001", &tp, &sl);

    println!("OCO группа создана:");
    println!("- Take Profit: ${:.2}", tp.borrow().price);
    println!("- Stop Loss: ${:.2}\n", sl.borrow().price);

    // Цена достигла Take Profit
    tp.borrow_mut().fill();
}
```

## Портфель с отслеживанием позиций

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

struct Portfolio {
    name: String,
    positions: HashMap<String, Rc<RefCell<TrackedPosition>>>,
}

struct TrackedPosition {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    portfolio: Weak<RefCell<Portfolio>>,
}

impl Portfolio {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Portfolio {
            name: name.to_string(),
            positions: HashMap::new(),
        }))
    }

    fn add_position(
        portfolio: &Rc<RefCell<Self>>,
        symbol: &str,
        quantity: f64,
        price: f64
    ) {
        let position = Rc::new(RefCell::new(TrackedPosition {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
            portfolio: Rc::downgrade(portfolio),
        }));

        portfolio.borrow_mut().positions.insert(
            symbol.to_string(),
            position
        );
    }

    fn total_value(&self, prices: &HashMap<String, f64>) -> f64 {
        self.positions.iter().map(|(symbol, pos)| {
            let position = pos.borrow();
            let price = prices.get(symbol).unwrap_or(&position.entry_price);
            position.quantity * price
        }).sum()
    }

    fn print_summary(&self, prices: &HashMap<String, f64>) {
        println!("\n=== Portfolio: {} ===", self.name);
        for (symbol, pos) in &self.positions {
            let position = pos.borrow();
            let current_price = prices.get(symbol).unwrap_or(&position.entry_price);
            let pnl = (current_price - position.entry_price) * position.quantity;
            let pnl_percent = (current_price - position.entry_price)
                / position.entry_price * 100.0;

            println!(
                "{}: {:.4} @ ${:.2} | P&L: ${:.2} ({:.2}%)",
                symbol, position.quantity, current_price, pnl, pnl_percent
            );
        }
        println!("Total Value: ${:.2}", self.total_value(prices));
    }
}

impl TrackedPosition {
    fn get_portfolio_name(&self) -> Option<String> {
        self.portfolio.upgrade().map(|p| p.borrow().name.clone())
    }
}

fn main() {
    let portfolio = Portfolio::new("Main Trading Account");

    Portfolio::add_position(&portfolio, "BTC", 0.5, 42000.0);
    Portfolio::add_position(&portfolio, "ETH", 5.0, 2500.0);
    Portfolio::add_position(&portfolio, "SOL", 100.0, 95.0);

    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 43500.0);
    prices.insert("ETH".to_string(), 2650.0);
    prices.insert("SOL".to_string(), 102.0);

    portfolio.borrow().print_summary(&prices);

    // Позиция знает свой портфель
    if let Some(btc_pos) = portfolio.borrow().positions.get("BTC") {
        if let Some(name) = btc_pos.borrow().get_portfolio_name() {
            println!("\nBTC position belongs to: {}", name);
        }
    }
}
```

## Weak с Arc для многопоточности

```rust
use std::sync::{Arc, Weak, Mutex};
use std::thread;

struct SharedMarketData {
    symbol: String,
    price: Mutex<f64>,
}

struct PriceObserver {
    name: String,
    data_source: Weak<SharedMarketData>,
}

impl PriceObserver {
    fn new(name: &str, source: &Arc<SharedMarketData>) -> Self {
        PriceObserver {
            name: name.to_string(),
            data_source: Arc::downgrade(source),
        }
    }

    fn check_price(&self) {
        match self.data_source.upgrade() {
            Some(data) => {
                let price = data.price.lock().unwrap();
                println!("{} sees {}: ${:.2}", self.name, data.symbol, *price);
            }
            None => {
                println!("{}: Data source no longer available", self.name);
            }
        }
    }
}

fn main() {
    let market_data = Arc::new(SharedMarketData {
        symbol: "BTC/USDT".to_string(),
        price: Mutex::new(42000.0),
    });

    let observer1 = PriceObserver::new("Strategy A", &market_data);
    let observer2 = PriceObserver::new("Strategy B", &market_data);

    // Обновляем цену
    *market_data.price.lock().unwrap() = 42500.0;

    observer1.check_price();
    observer2.check_price();

    // Освобождаем источник данных
    drop(market_data);

    // Наблюдатели безопасно обрабатывают отсутствие данных
    observer1.check_price();
    observer2.check_price();
}
```

## Методы Weak

```rust
use std::rc::{Rc, Weak};

fn main() {
    let strong = Rc::new(42);
    let weak: Weak<i32> = Rc::downgrade(&strong);

    // strong_count - количество Rc
    println!("Strong count: {}", Rc::strong_count(&strong));

    // weak_count - количество Weak
    println!("Weak count: {}", Rc::weak_count(&strong));

    // upgrade - попытка получить Rc
    if let Some(value) = weak.upgrade() {
        println!("Value: {}", value);
    }

    // После drop
    drop(strong);

    // upgrade возвращает None
    assert!(weak.upgrade().is_none());
    println!("Weak reference is now invalid");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Weak<T>` | Слабая ссылка, не владеет данными |
| `Rc::downgrade(&rc)` | Создаёт Weak из Rc |
| `weak.upgrade()` | Пытается получить Rc (Option) |
| `Rc::strong_count()` | Количество сильных ссылок |
| `Rc::weak_count()` | Количество слабых ссылок |
| `Arc`/`Weak` | Многопоточные версии |

## Практические упражнения

### Упражнение 1: Граф стратегий
Создайте систему торговых стратегий, где стратегии могут зависеть друг от друга без циклических ссылок.

### Упражнение 2: Кэш ордеров
Реализуйте кэш исполненных ордеров с использованием Weak ссылок для автоматической очистки.

### Упражнение 3: Event система
Создайте систему событий, где подписчики хранятся как Weak ссылки.

### Упражнение 4: Дерево портфелей
Реализуйте иерархию портфелей (главный -> суб-портфели) с обратными ссылками.

## Домашнее задание

1. **Система OCO ордеров**: Расширьте пример OCO для поддержки нескольких связанных ордеров (OSO - One-Sends-Other).

2. **Мониторинг позиций**: Создайте систему мониторинга, где несколько мониторов отслеживают одну позицию через Weak ссылки.

3. **Bracket Order**: Реализуйте Bracket Order (вход + TP + SL) с корректными связями через Weak.

4. **Cleanup система**: Напишите функцию, которая проверяет все Weak ссылки и удаляет недействительные из коллекций.

## Навигация

[← Предыдущий день](../057-rc-refcell/ru.md) | [Следующий день →](../059-interior-mutability/ru.md)
