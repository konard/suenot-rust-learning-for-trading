# День 89: Комбинирование структур — биржа содержит ордера

## Аналогия из трейдинга

Представь криптобиржу. Она не существует сама по себе — она **содержит** ордера, торговые пары, историю сделок и балансы пользователей. Это как матрёшка: большая структура (биржа) содержит внутри себя другие структуры (ордера, сделки). В программировании это называется **композиция** — один из фундаментальных паттернов проектирования.

## Базовая концепция: структура внутри структуры

```rust
fn main() {
    // Создаём ордер
    let order = Order {
        id: 1,
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 0.5,
        status: OrderStatus::Open,
    };

    println!("Order #{}: {} {} @ ${}",
        order.id,
        order.side_str(),
        order.symbol,
        order.price
    );
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
enum OrderStatus {
    Open,
    Filled,
    Cancelled,
    PartiallyFilled,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

impl Order {
    fn side_str(&self) -> &str {
        match self.side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    fn value(&self) -> f64 {
        self.price * self.quantity
    }
}
```

## Биржа, содержащая ордера

```rust
fn main() {
    let mut exchange = Exchange::new("CryptoExchange");

    // Добавляем ордера
    exchange.place_order("BTC/USDT", OrderSide::Buy, 42000.0, 0.5);
    exchange.place_order("BTC/USDT", OrderSide::Sell, 43000.0, 0.3);
    exchange.place_order("ETH/USDT", OrderSide::Buy, 2800.0, 2.0);

    // Выводим статистику
    exchange.print_summary();

    // Находим ордера по символу
    println!("\nBTC orders:");
    for order in exchange.get_orders_by_symbol("BTC/USDT") {
        println!("  #{}: {} {} @ ${}",
            order.id, order.side_str(), order.quantity, order.price);
    }
}

#[derive(Debug)]
struct Exchange {
    name: String,
    orders: Vec<Order>,
    next_order_id: u64,
}

impl Exchange {
    fn new(name: &str) -> Self {
        Exchange {
            name: String::from(name),
            orders: Vec::new(),
            next_order_id: 1,
        }
    }

    fn place_order(&mut self, symbol: &str, side: OrderSide, price: f64, quantity: f64) -> u64 {
        let order = Order {
            id: self.next_order_id,
            symbol: String::from(symbol),
            side,
            price,
            quantity,
            status: OrderStatus::Open,
        };

        let order_id = order.id;
        self.orders.push(order);
        self.next_order_id += 1;

        println!("Order #{} placed on {}", order_id, self.name);
        order_id
    }

    fn get_orders_by_symbol(&self, symbol: &str) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| o.symbol == symbol)
            .collect()
    }

    fn get_open_orders(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| matches!(o.status, OrderStatus::Open))
            .collect()
    }

    fn total_order_value(&self) -> f64 {
        self.orders.iter().map(|o| o.value()).sum()
    }

    fn print_summary(&self) {
        println!("\n╔══════════════════════════════════════╗");
        println!("║  Exchange: {:^25} ║", self.name);
        println!("╠══════════════════════════════════════╣");
        println!("║  Total orders: {:>21} ║", self.orders.len());
        println!("║  Open orders:  {:>21} ║", self.get_open_orders().len());
        println!("║  Total value:  ${:>19.2} ║", self.total_order_value());
        println!("╚══════════════════════════════════════╝");
    }
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
enum OrderStatus {
    Open,
    Filled,
    Cancelled,
    PartiallyFilled,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

impl Order {
    fn side_str(&self) -> &str {
        match self.side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }

    fn value(&self) -> f64 {
        self.price * self.quantity
    }
}
```

## Многоуровневая композиция: биржа → книга ордеров → ордера

```rust
fn main() {
    let mut exchange = Exchange::new("ProExchange");

    // Добавляем торговые пары
    exchange.add_trading_pair("BTC/USDT");
    exchange.add_trading_pair("ETH/USDT");

    // Размещаем ордера
    exchange.place_order("BTC/USDT", OrderSide::Buy, 41500.0, 1.0);
    exchange.place_order("BTC/USDT", OrderSide::Buy, 41800.0, 0.5);
    exchange.place_order("BTC/USDT", OrderSide::Sell, 42500.0, 0.8);
    exchange.place_order("BTC/USDT", OrderSide::Sell, 42800.0, 1.2);

    exchange.place_order("ETH/USDT", OrderSide::Buy, 2750.0, 5.0);
    exchange.place_order("ETH/USDT", OrderSide::Sell, 2850.0, 3.0);

    // Показываем книгу ордеров
    exchange.print_order_book("BTC/USDT");
}

#[derive(Debug)]
struct OrderBook {
    symbol: String,
    bids: Vec<Order>,  // Ордера на покупку
    asks: Vec<Order>,  // Ордера на продажу
}

impl OrderBook {
    fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: String::from(symbol),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        match order.side {
            OrderSide::Buy => {
                self.bids.push(order);
                // Сортируем по убыванию цены (лучшие bid сверху)
                self.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
            }
            OrderSide::Sell => {
                self.asks.push(order);
                // Сортируем по возрастанию цены (лучшие ask сверху)
                self.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
            }
        }
    }

    fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|o| o.price)
    }

    fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|o| o.price)
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }

    fn print(&self) {
        println!("\n📊 Order Book: {}", self.symbol);
        println!("═══════════════════════════════════════");

        println!("ASKS (Sell orders):");
        for ask in self.asks.iter().rev().take(5) {
            println!("  ${:>10.2} | {:>8.4}", ask.price, ask.quantity);
        }

        if let Some(spread) = self.spread() {
            println!("───────────── Spread: ${:.2} ─────────────", spread);
        }

        println!("BIDS (Buy orders):");
        for bid in self.bids.iter().take(5) {
            println!("  ${:>10.2} | {:>8.4}", bid.price, bid.quantity);
        }
    }
}

#[derive(Debug)]
struct Exchange {
    name: String,
    order_books: Vec<OrderBook>,
    next_order_id: u64,
}

impl Exchange {
    fn new(name: &str) -> Self {
        Exchange {
            name: String::from(name),
            order_books: Vec::new(),
            next_order_id: 1,
        }
    }

    fn add_trading_pair(&mut self, symbol: &str) {
        if !self.order_books.iter().any(|ob| ob.symbol == symbol) {
            self.order_books.push(OrderBook::new(symbol));
            println!("Trading pair {} added to {}", symbol, self.name);
        }
    }

    fn get_order_book_mut(&mut self, symbol: &str) -> Option<&mut OrderBook> {
        self.order_books.iter_mut().find(|ob| ob.symbol == symbol)
    }

    fn place_order(&mut self, symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Option<u64> {
        let order = Order {
            id: self.next_order_id,
            symbol: String::from(symbol),
            side,
            price,
            quantity,
            status: OrderStatus::Open,
        };

        let order_id = order.id;

        if let Some(order_book) = self.get_order_book_mut(symbol) {
            order_book.add_order(order);
            self.next_order_id += 1;
            Some(order_id)
        } else {
            println!("Trading pair {} not found!", symbol);
            None
        }
    }

    fn print_order_book(&self, symbol: &str) {
        if let Some(order_book) = self.order_books.iter().find(|ob| ob.symbol == symbol) {
            order_book.print();
        }
    }
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
enum OrderStatus {
    Open,
    Filled,
    Cancelled,
    PartiallyFilled,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}
```

## Композиция для управления портфелем

```rust
fn main() {
    let mut portfolio = Portfolio::new("My Trading Portfolio");

    // Добавляем позиции
    portfolio.add_position("BTC", 0.5, 42000.0);
    portfolio.add_position("ETH", 5.0, 2800.0);
    portfolio.add_position("SOL", 100.0, 95.0);

    // Обновляем цены
    portfolio.update_price("BTC", 43500.0);
    portfolio.update_price("ETH", 2950.0);
    portfolio.update_price("SOL", 105.0);

    // Показываем портфель
    portfolio.print_summary();
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn new(symbol: &str, quantity: f64, entry_price: f64) -> Self {
        Position {
            symbol: String::from(symbol),
            quantity,
            entry_price,
            current_price: entry_price,
        }
    }

    fn cost_basis(&self) -> f64 {
        self.quantity * self.entry_price
    }

    fn current_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    fn unrealized_pnl(&self) -> f64 {
        self.current_value() - self.cost_basis()
    }

    fn pnl_percent(&self) -> f64 {
        if self.cost_basis() > 0.0 {
            (self.unrealized_pnl() / self.cost_basis()) * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
struct Portfolio {
    name: String,
    positions: Vec<Position>,
}

impl Portfolio {
    fn new(name: &str) -> Self {
        Portfolio {
            name: String::from(name),
            positions: Vec::new(),
        }
    }

    fn add_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        // Проверяем, есть ли уже позиция
        if let Some(pos) = self.positions.iter_mut().find(|p| p.symbol == symbol) {
            // Усредняем цену входа
            let total_cost = pos.cost_basis() + (quantity * price);
            let total_qty = pos.quantity + quantity;
            pos.quantity = total_qty;
            pos.entry_price = total_cost / total_qty;
        } else {
            self.positions.push(Position::new(symbol, quantity, price));
        }
    }

    fn update_price(&mut self, symbol: &str, price: f64) {
        if let Some(pos) = self.positions.iter_mut().find(|p| p.symbol == symbol) {
            pos.current_price = price;
        }
    }

    fn total_value(&self) -> f64 {
        self.positions.iter().map(|p| p.current_value()).sum()
    }

    fn total_cost(&self) -> f64 {
        self.positions.iter().map(|p| p.cost_basis()).sum()
    }

    fn total_pnl(&self) -> f64 {
        self.total_value() - self.total_cost()
    }

    fn print_summary(&self) {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║  Portfolio: {:^50} ║", self.name);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  Symbol │   Qty   │  Entry  │ Current │    PnL   │   PnL%  ║");
        println!("╠════════════════════════════════════════════════════════════════╣");

        for pos in &self.positions {
            let pnl_sign = if pos.unrealized_pnl() >= 0.0 { "+" } else { "" };
            println!("║  {:^6} │ {:>7.3} │ ${:>6.0} │ ${:>6.0} │ {}{:>7.2} │ {:>+6.2}% ║",
                pos.symbol,
                pos.quantity,
                pos.entry_price,
                pos.current_price,
                pnl_sign,
                pos.unrealized_pnl(),
                pos.pnl_percent()
            );
        }

        println!("╠════════════════════════════════════════════════════════════════╣");
        let total_pnl_sign = if self.total_pnl() >= 0.0 { "+" } else { "" };
        println!("║  TOTAL  │         │ ${:>6.0} │ ${:>6.0} │ {}{:>7.2} │ {:>+6.2}% ║",
            self.total_cost(),
            self.total_value(),
            total_pnl_sign,
            self.total_pnl(),
            (self.total_pnl() / self.total_cost()) * 100.0
        );
        println!("╚════════════════════════════════════════════════════════════════╝");
    }
}
```

## Риск-менеджмент с вложенными структурами

```rust
fn main() {
    let mut risk_manager = RiskManager::new(100000.0); // Баланс $100,000

    // Устанавливаем лимиты
    risk_manager.set_max_position_size(0.1); // 10% на позицию
    risk_manager.set_max_daily_loss(0.02);   // 2% макс. убыток в день

    // Добавляем сделки
    let trade1 = Trade::new("BTC/USDT", OrderSide::Buy, 42000.0, 0.2, 41000.0);
    let trade2 = Trade::new("ETH/USDT", OrderSide::Buy, 2800.0, 3.0, 2700.0);

    println!("Trade 1 allowed: {}", risk_manager.check_trade(&trade1));
    println!("Trade 2 allowed: {}", risk_manager.check_trade(&trade2));

    risk_manager.add_trade(trade1);
    risk_manager.print_risk_report();
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    side: OrderSide,
    entry_price: f64,
    quantity: f64,
    stop_loss: f64,
}

impl Trade {
    fn new(symbol: &str, side: OrderSide, entry: f64, qty: f64, stop: f64) -> Self {
        Trade {
            symbol: String::from(symbol),
            side,
            entry_price: entry,
            quantity: qty,
            stop_loss: stop,
        }
    }

    fn position_value(&self) -> f64 {
        self.entry_price * self.quantity
    }

    fn risk_amount(&self) -> f64 {
        (self.entry_price - self.stop_loss).abs() * self.quantity
    }
}

#[derive(Debug)]
struct RiskManager {
    balance: f64,
    max_position_pct: f64,
    max_daily_loss_pct: f64,
    active_trades: Vec<Trade>,
    daily_pnl: f64,
}

impl RiskManager {
    fn new(balance: f64) -> Self {
        RiskManager {
            balance,
            max_position_pct: 0.1,
            max_daily_loss_pct: 0.02,
            active_trades: Vec::new(),
            daily_pnl: 0.0,
        }
    }

    fn set_max_position_size(&mut self, pct: f64) {
        self.max_position_pct = pct;
    }

    fn set_max_daily_loss(&mut self, pct: f64) {
        self.max_daily_loss_pct = pct;
    }

    fn check_trade(&self, trade: &Trade) -> bool {
        // Проверка размера позиции
        let max_position = self.balance * self.max_position_pct;
        if trade.position_value() > max_position {
            println!("❌ Position too large: ${:.2} > ${:.2}",
                trade.position_value(), max_position);
            return false;
        }

        // Проверка риска
        let max_risk = self.balance * self.max_daily_loss_pct;
        let current_risk: f64 = self.active_trades.iter().map(|t| t.risk_amount()).sum();

        if current_risk + trade.risk_amount() > max_risk {
            println!("❌ Risk limit exceeded: ${:.2} + ${:.2} > ${:.2}",
                current_risk, trade.risk_amount(), max_risk);
            return false;
        }

        println!("✅ Trade approved");
        true
    }

    fn add_trade(&mut self, trade: Trade) {
        if self.check_trade(&trade) {
            self.active_trades.push(trade);
        }
    }

    fn total_exposure(&self) -> f64 {
        self.active_trades.iter().map(|t| t.position_value()).sum()
    }

    fn total_risk(&self) -> f64 {
        self.active_trades.iter().map(|t| t.risk_amount()).sum()
    }

    fn print_risk_report(&self) {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║           RISK MANAGEMENT REPORT         ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║  Account Balance:      ${:>15.2} ║", self.balance);
        println!("║  Active Trades:        {:>16} ║", self.active_trades.len());
        println!("║  Total Exposure:       ${:>15.2} ║", self.total_exposure());
        println!("║  Exposure %:           {:>15.2}% ║",
            (self.total_exposure() / self.balance) * 100.0);
        println!("║  Total Risk:           ${:>15.2} ║", self.total_risk());
        println!("║  Risk %:               {:>15.2}% ║",
            (self.total_risk() / self.balance) * 100.0);
        println!("║  Max Position Limit:   {:>15.2}% ║", self.max_position_pct * 100.0);
        println!("║  Max Daily Loss Limit: {:>15.2}% ║", self.max_daily_loss_pct * 100.0);
        println!("╚══════════════════════════════════════════╝");
    }
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}
```

## Паттерны композиции

| Паттерн | Описание | Пример |
|---------|----------|--------|
| Один-ко-многим | Контейнер содержит коллекцию | `Exchange` → `Vec<Order>` |
| Иерархия | Многоуровневое вложение | `Exchange` → `OrderBook` → `Order` |
| Агрегация | Слабая связь, независимые объекты | `Portfolio` → `Vec<Position>` |
| Композиция | Сильная связь, зависимые объекты | `RiskManager` → `Vec<Trade>` |

## Что мы узнали

1. **Композиция** — основной способ организации сложных данных
2. **Вложенные структуры** позволяют моделировать реальные системы
3. **Методы** могут работать с внутренними структурами
4. **Итераторы** помогают обрабатывать коллекции внутри структур
5. **Многоуровневая иерархия** — биржа содержит книги ордеров, которые содержат ордера

## Упражнения

1. Добавь в `Exchange` метод `cancel_order(order_id: u64) -> bool`
2. Реализуй метод `match_orders()` для простого сопоставления ордеров
3. Добавь в `Portfolio` метод `close_position(symbol: &str, price: f64) -> Option<f64>`, возвращающий реализованный PnL

## Домашнее задание

1. Создай структуру `TradingStrategy`, которая содержит:
   - Список сигналов (`Vec<Signal>`)
   - Параметры стратегии (`StrategyParams`)
   - Историю сделок (`Vec<Trade>`)

2. Реализуй `MultiExchangeManager`, который управляет несколькими биржами и может находить лучшую цену среди всех

3. Создай систему алертов `AlertSystem` с вложенными структурами:
   - `Alert` (тип, условие, действие)
   - `PriceAlert` (срабатывает при достижении цены)
   - `VolumeAlert` (срабатывает при объёме)

4. Напиши `BacktestEngine`, содержащий:
   - Исторические данные (`Vec<Candle>`)
   - Стратегию (`Strategy`)
   - Результаты (`BacktestResult`)

## Навигация

[← Предыдущий день](../088-struct-methods-order-actions/ru.md) | [Следующий день →](../090-tuple-structs-bid-ask/ru.md)
