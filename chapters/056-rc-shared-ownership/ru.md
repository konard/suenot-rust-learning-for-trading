# День 56: Rc — несколько владельцев одного актива

## Аналогия из трейдинга

Представь ситуацию: у тебя есть **рыночные данные** (котировки, история цен), которые одновременно используют несколько торговых стратегий. Каждая стратегия — независимый модуль, но все они работают с одними и теми же данными. Кто "владеет" этими данными? Все стратегии одновременно!

В Rust по умолчанию у данных может быть только один владелец. Но `Rc<T>` (Reference Counting) позволяет иметь **несколько владельцев** одних данных. Это как совместный доступ к биржевому терминалу — каждый трейдер может смотреть котировки, но терминал один.

## Что такое Rc?

`Rc` — это "умный указатель" со счётчиком ссылок:
- При клонировании `Rc` данные не копируются — увеличивается счётчик
- Когда счётчик становится 0, данные освобождаются
- `Rc` работает только в однопоточном коде (для многопоточности есть `Arc`)

```rust
use std::rc::Rc;

fn main() {
    // Рыночные данные, которые будут использовать несколько стратегий
    let market_data = Rc::new(vec![42000.0, 42100.0, 41950.0, 42200.0]);

    println!("Счётчик ссылок: {}", Rc::strong_count(&market_data));  // 1

    let strategy_a = Rc::clone(&market_data);
    println!("После клонирования для стратегии A: {}", Rc::strong_count(&market_data));  // 2

    let strategy_b = Rc::clone(&market_data);
    println!("После клонирования для стратегии B: {}", Rc::strong_count(&market_data));  // 3

    // Все три переменные указывают на одни и те же данные
    println!("Данные из strategy_a: {:?}", strategy_a);
    println!("Данные из strategy_b: {:?}", strategy_b);
}
```

**Важно:** Используй `Rc::clone(&x)` вместо `x.clone()`. Это явно показывает, что мы увеличиваем счётчик, а не копируем данные.

## Когда использовать Rc в трейдинге

### 1. Общие рыночные данные для нескольких стратегий

```rust
use std::rc::Rc;

struct MarketData {
    symbol: String,
    prices: Vec<f64>,
    volumes: Vec<f64>,
}

struct MomentumStrategy {
    data: Rc<MarketData>,
    period: usize,
}

struct MeanReversionStrategy {
    data: Rc<MarketData>,
    threshold: f64,
}

impl MomentumStrategy {
    fn analyze(&self) -> &str {
        if self.data.prices.len() < self.period {
            return "INSUFFICIENT_DATA";
        }
        let recent = &self.data.prices[self.data.prices.len() - self.period..];
        let first = recent[0];
        let last = recent[recent.len() - 1];

        if last > first * 1.02 {
            "BUY"
        } else if last < first * 0.98 {
            "SELL"
        } else {
            "HOLD"
        }
    }
}

impl MeanReversionStrategy {
    fn analyze(&self) -> &str {
        if self.data.prices.is_empty() {
            return "INSUFFICIENT_DATA";
        }
        let avg: f64 = self.data.prices.iter().sum::<f64>() / self.data.prices.len() as f64;
        let current = self.data.prices.last().unwrap();
        let deviation = (current - avg) / avg;

        if deviation > self.threshold {
            "SELL"  // Цена выше среднего — ожидаем возврат
        } else if deviation < -self.threshold {
            "BUY"   // Цена ниже среднего — ожидаем возврат
        } else {
            "HOLD"
        }
    }
}

fn main() {
    let data = Rc::new(MarketData {
        symbol: String::from("BTC/USDT"),
        prices: vec![42000.0, 42500.0, 43000.0, 43500.0, 44000.0],
        volumes: vec![100.0, 150.0, 200.0, 180.0, 220.0],
    });

    let momentum = MomentumStrategy {
        data: Rc::clone(&data),
        period: 3,
    };

    let mean_reversion = MeanReversionStrategy {
        data: Rc::clone(&data),
        threshold: 0.02,
    };

    println!("Символ: {}", data.symbol);
    println!("Momentum сигнал: {}", momentum.analyze());
    println!("Mean Reversion сигнал: {}", mean_reversion.analyze());
    println!("Активных ссылок на данные: {}", Rc::strong_count(&data));
}
```

### 2. Граф зависимостей ордеров

В трейдинге ордера могут зависеть друг от друга (OCO, bracket orders). `Rc` позволяет моделировать такие связи:

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    linked_orders: Vec<Rc<RefCell<Order>>>,
}

impl Order {
    fn new(id: u64, symbol: &str, side: &str, price: f64, quantity: f64) -> Self {
        Order {
            id,
            symbol: String::from(symbol),
            side: String::from(side),
            price,
            quantity,
            linked_orders: Vec::new(),
        }
    }

    fn link_order(&mut self, order: Rc<RefCell<Order>>) {
        self.linked_orders.push(order);
    }

    fn cancel_linked(&self) {
        for order in &self.linked_orders {
            println!("Отмена связанного ордера #{}", order.borrow().id);
        }
    }
}

fn main() {
    // Bracket order: основной ордер + take profit + stop loss
    let main_order = Rc::new(RefCell::new(Order::new(
        1, "BTC/USDT", "BUY", 42000.0, 0.5
    )));

    let take_profit = Rc::new(RefCell::new(Order::new(
        2, "BTC/USDT", "SELL", 45000.0, 0.5
    )));

    let stop_loss = Rc::new(RefCell::new(Order::new(
        3, "BTC/USDT", "SELL", 40000.0, 0.5
    )));

    // Связываем take profit и stop loss с основным ордером
    main_order.borrow_mut().link_order(Rc::clone(&take_profit));
    main_order.borrow_mut().link_order(Rc::clone(&stop_loss));

    // Take profit и stop loss связаны друг с другом (OCO)
    take_profit.borrow_mut().link_order(Rc::clone(&stop_loss));
    stop_loss.borrow_mut().link_order(Rc::clone(&take_profit));

    println!("Основной ордер: {:?}", main_order.borrow().id);
    println!("При исполнении take profit, отменяем связанные:");
    take_profit.borrow().cancel_linked();
}
```

### 3. Общий конфиг для компонентов системы

```rust
use std::rc::Rc;

struct TradingConfig {
    max_position_size: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
    api_endpoint: String,
}

struct RiskManager {
    config: Rc<TradingConfig>,
}

struct OrderExecutor {
    config: Rc<TradingConfig>,
}

struct PortfolioTracker {
    config: Rc<TradingConfig>,
}

impl RiskManager {
    fn check_position(&self, size: f64) -> bool {
        size <= self.config.max_position_size
    }

    fn calculate_position_size(&self, balance: f64) -> f64 {
        balance * (self.config.risk_per_trade / 100.0)
    }
}

impl OrderExecutor {
    fn can_trade(&self, symbol: &str) -> bool {
        self.config.allowed_symbols.contains(&symbol.to_string())
    }
}

impl PortfolioTracker {
    fn get_api(&self) -> &str {
        &self.config.api_endpoint
    }
}

fn main() {
    let config = Rc::new(TradingConfig {
        max_position_size: 10000.0,
        risk_per_trade: 2.0,
        allowed_symbols: vec![
            String::from("BTC/USDT"),
            String::from("ETH/USDT"),
        ],
        api_endpoint: String::from("https://api.exchange.com"),
    });

    let risk_mgr = RiskManager { config: Rc::clone(&config) };
    let executor = OrderExecutor { config: Rc::clone(&config) };
    let tracker = PortfolioTracker { config: Rc::clone(&config) };

    println!("Позиция 5000 допустима: {}", risk_mgr.check_position(5000.0));
    println!("Можно торговать BTC: {}", executor.can_trade("BTC/USDT"));
    println!("API endpoint: {}", tracker.get_api());
    println!("Компонентов с конфигом: {}", Rc::strong_count(&config));
}
```

## Rc + RefCell: изменяемые данные с несколькими владельцами

`Rc` даёт неизменяемый доступ. Для изменяемых данных комбинируй `Rc<RefCell<T>>`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Portfolio {
    balance: f64,
    positions: Vec<(String, f64)>,  // (symbol, quantity)
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn buy(&mut self, symbol: &str, quantity: f64, price: f64) -> bool {
        let cost = quantity * price;
        if cost > self.balance {
            return false;
        }
        self.balance -= cost;
        self.positions.push((symbol.to_string(), quantity));
        true
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }
}

struct Strategy {
    name: String,
    portfolio: Rc<RefCell<Portfolio>>,
}

impl Strategy {
    fn execute_buy(&self, symbol: &str, quantity: f64, price: f64) {
        let success = self.portfolio.borrow_mut().buy(symbol, quantity, price);
        if success {
            println!("{}: Купил {} {} по ${}", self.name, quantity, symbol, price);
        } else {
            println!("{}: Недостаточно средств", self.name);
        }
    }

    fn check_balance(&self) {
        println!("{}: Текущий баланс ${:.2}",
            self.name,
            self.portfolio.borrow().get_balance()
        );
    }
}

fn main() {
    let portfolio = Rc::new(RefCell::new(Portfolio::new(10000.0)));

    let strategy_a = Strategy {
        name: String::from("Momentum"),
        portfolio: Rc::clone(&portfolio),
    };

    let strategy_b = Strategy {
        name: String::from("Scalping"),
        portfolio: Rc::clone(&portfolio),
    };

    strategy_a.check_balance();
    strategy_a.execute_buy("BTC/USDT", 0.1, 42000.0);
    strategy_b.check_balance();  // Баланс уже изменился!
    strategy_b.execute_buy("ETH/USDT", 1.0, 2500.0);
    strategy_a.check_balance();

    println!("\nИтоговый баланс: ${:.2}", portfolio.borrow().get_balance());
}
```

## Подсчёт ссылок

```rust
use std::rc::Rc;

fn main() {
    let data = Rc::new(vec![42000.0, 42100.0]);
    println!("После создания: {}", Rc::strong_count(&data));  // 1

    {
        let ref1 = Rc::clone(&data);
        println!("Внутри блока: {}", Rc::strong_count(&data));  // 2

        let ref2 = Rc::clone(&data);
        println!("Ещё одна ссылка: {}", Rc::strong_count(&data));  // 3
    }  // ref1 и ref2 выходят из области видимости

    println!("После блока: {}", Rc::strong_count(&data));  // 1
}
```

## Weak ссылки — предотвращение циклов

`Rc::downgrade` создаёт слабую ссылку, которая не увеличивает счётчик:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TradingBot {
    name: String,
    parent: Option<Weak<RefCell<TradingBot>>>,  // Слабая ссылка на родителя
    children: Vec<Rc<RefCell<TradingBot>>>,     // Сильные ссылки на детей
}

impl TradingBot {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TradingBot {
            name: name.to_string(),
            parent: None,
            children: Vec::new(),
        }))
    }

    fn add_child(parent: &Rc<RefCell<Self>>, child: &Rc<RefCell<Self>>) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(Rc::clone(child));
    }
}

fn main() {
    let master_bot = TradingBot::new("MasterBot");
    let btc_bot = TradingBot::new("BTCBot");
    let eth_bot = TradingBot::new("ETHBot");

    TradingBot::add_child(&master_bot, &btc_bot);
    TradingBot::add_child(&master_bot, &eth_bot);

    // Дочерний бот может получить доступ к родителю
    if let Some(parent_weak) = &btc_bot.borrow().parent {
        if let Some(parent) = parent_weak.upgrade() {
            println!("Родитель BTCBot: {}", parent.borrow().name);
        }
    }

    println!("Ссылок на MasterBot: {}", Rc::strong_count(&master_bot));  // 1
    println!("Дочерних ботов: {}", master_bot.borrow().children.len());  // 2
}
```

## Практический пример: система мониторинга портфеля

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct PriceData {
    symbol: String,
    current_price: f64,
    history: Vec<f64>,
}

impl PriceData {
    fn new(symbol: &str, initial_price: f64) -> Self {
        PriceData {
            symbol: symbol.to_string(),
            current_price: initial_price,
            history: vec![initial_price],
        }
    }

    fn update(&mut self, new_price: f64) {
        self.current_price = new_price;
        self.history.push(new_price);
    }

    fn get_change_percent(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let first = self.history[0];
        ((self.current_price - first) / first) * 100.0
    }
}

struct Position {
    price_data: Rc<RefCell<PriceData>>,
    quantity: f64,
    entry_price: f64,
}

impl Position {
    fn get_pnl(&self) -> f64 {
        let current = self.price_data.borrow().current_price;
        (current - self.entry_price) * self.quantity
    }

    fn get_pnl_percent(&self) -> f64 {
        let current = self.price_data.borrow().current_price;
        ((current - self.entry_price) / self.entry_price) * 100.0
    }
}

struct RiskMonitor {
    price_data: Rc<RefCell<PriceData>>,
    alert_threshold: f64,
}

impl RiskMonitor {
    fn check_alerts(&self) -> Option<String> {
        let data = self.price_data.borrow();
        let change = data.get_change_percent();

        if change.abs() > self.alert_threshold {
            Some(format!(
                "ALERT: {} изменился на {:.2}%!",
                data.symbol, change
            ))
        } else {
            None
        }
    }
}

fn main() {
    // Общие ценовые данные
    let btc_data = Rc::new(RefCell::new(PriceData::new("BTC/USDT", 42000.0)));

    // Позиция использует те же данные
    let position = Position {
        price_data: Rc::clone(&btc_data),
        quantity: 0.5,
        entry_price: 42000.0,
    };

    // Монитор рисков тоже
    let risk_monitor = RiskMonitor {
        price_data: Rc::clone(&btc_data),
        alert_threshold: 5.0,
    };

    println!("=== Начальное состояние ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    // Обновляем цену — все компоненты видят изменения
    btc_data.borrow_mut().update(44000.0);

    println!("\n=== После роста цены ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    if let Some(alert) = risk_monitor.check_alerts() {
        println!("{}", alert);
    }

    btc_data.borrow_mut().update(40000.0);

    println!("\n=== После падения цены ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    if let Some(alert) = risk_monitor.check_alerts() {
        println!("{}", alert);
    }
}
```

## Rc vs другие подходы

| Подход | Когда использовать |
|--------|-------------------|
| Ownership | Один владелец, данные перемещаются |
| References | Временное заимствование, известное время жизни |
| `Rc<T>` | Несколько владельцев, неизменяемые данные |
| `Rc<RefCell<T>>` | Несколько владельцев, нужно изменять данные |
| `Arc<T>` | Несколько владельцев в многопоточном коде |

## Ограничения Rc

1. **Только однопоточный код** — для многопоточности используй `Arc`
2. **Возможны циклы** — используй `Weak` для предотвращения утечек памяти
3. **Накладные расходы** — подсчёт ссылок требует памяти и времени

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Rc<T>` | Умный указатель со счётчиком ссылок |
| `Rc::clone()` | Увеличивает счётчик, не копирует данные |
| `Rc::strong_count()` | Возвращает количество сильных ссылок |
| `Rc<RefCell<T>>` | Комбинация для изменяемых данных |
| `Rc::downgrade()` | Создаёт слабую ссылку (Weak) |
| `Weak::upgrade()` | Пытается получить Rc из Weak |

## Домашнее задание

1. Создай систему с несколькими торговыми стратегиями, использующими общий `Rc<MarketData>`. Реализуй обновление данных и проверь, что все стратегии видят актуальные цены.

2. Реализуй граф OCO-ордеров (One-Cancels-Other), где при исполнении одного ордера автоматически отменяются связанные. Используй `Rc<RefCell<Order>>`.

3. Создай иерархию торговых ботов (мастер-бот и дочерние боты для разных пар). Используй `Weak` для ссылок от детей к родителю.

4. Реализуй систему подписок на ценовые обновления: несколько подписчиков (`Rc`) на один источник данных, с возможностью отписки (удаление ссылки).

## Навигация

[← Предыдущий день](../055-box-heap-allocation/ru.md) | [Следующий день →](../057-refcell-interior-mutability/ru.md)
