# День 69: Геттеры и сеттеры — контролируемый доступ к цене

## Аналогия из трейдинга

Представь биржу: ты не можешь напрямую изменить цену актива в системе. Все изменения проходят через **контролируемые каналы** — ордера, валидацию, проверки лимитов. Геттеры и сеттеры в программировании работают так же: они обеспечивают **контролируемый доступ** к данным, позволяя добавлять валидацию, логирование и защиту.

## Зачем нужны геттеры и сеттеры?

В Rust поля структур по умолчанию приватны вне модуля. Это хорошо — мы защищаем внутреннее состояние. Но как тогда читать и изменять данные? Через специальные методы:

- **Геттер** — метод для чтения значения поля
- **Сеттер** — метод для изменения значения поля с валидацией

```rust
fn main() {
    let mut price = Price::new(42000.0);

    // Геттер — читаем цену
    println!("Current price: ${:.2}", price.value());

    // Сеттер — изменяем с валидацией
    if price.set_value(43500.0) {
        println!("Price updated to: ${:.2}", price.value());
    }

    // Попытка установить отрицательную цену
    if !price.set_value(-100.0) {
        println!("Invalid price rejected!");
    }
}

struct Price {
    value: f64,
}

impl Price {
    fn new(value: f64) -> Self {
        Price { value: value.max(0.0) }
    }

    // Геттер
    fn value(&self) -> f64 {
        self.value
    }

    // Сеттер с валидацией
    fn set_value(&mut self, new_value: f64) -> bool {
        if new_value >= 0.0 {
            self.value = new_value;
            true
        } else {
            false
        }
    }
}
```

## Базовые паттерны геттеров

### Простой геттер — возврат копии

```rust
struct TradingPair {
    base: String,
    quote: String,
    price: f64,
    volume: f64,
}

impl TradingPair {
    fn new(base: &str, quote: &str, price: f64, volume: f64) -> Self {
        TradingPair {
            base: base.to_string(),
            quote: quote.to_string(),
            price,
            volume,
        }
    }

    // Геттеры для примитивов — возвращаем копию
    fn price(&self) -> f64 {
        self.price
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    // Геттеры для String — возвращаем ссылку
    fn base(&self) -> &str {
        &self.base
    }

    fn quote(&self) -> &str {
        &self.quote
    }

    // Вычисляемый геттер
    fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }

    fn market_cap(&self) -> f64 {
        self.price * self.volume
    }
}

fn main() {
    let btc = TradingPair::new("BTC", "USD", 42000.0, 1500.0);

    println!("Pair: {}", btc.symbol());
    println!("Price: ${:.2}", btc.price());
    println!("Volume: {:.2}", btc.volume());
    println!("Market Cap: ${:.0}", btc.market_cap());
}
```

### Геттер с ссылкой — избегаем копирования

```rust
fn main() {
    let order = Order::new(
        "ORD-12345",
        "BTC/USD",
        42000.0,
        0.5,
        OrderSide::Buy,
    );

    println!("Order ID: {}", order.id());
    println!("Symbol: {}", order.symbol());
    println!("Side: {:?}", order.side());
    println!("Total: ${:.2}", order.total_value());
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    id: String,
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
}

impl Order {
    fn new(id: &str, symbol: &str, price: f64, quantity: f64, side: OrderSide) -> Self {
        Order {
            id: id.to_string(),
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
        }
    }

    // Возвращаем ссылку на строку
    fn id(&self) -> &str {
        &self.id
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    // Copy типы можно возвращать по значению
    fn price(&self) -> f64 {
        self.price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn side(&self) -> OrderSide {
        self.side
    }

    // Вычисляемое значение
    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}
```

## Сеттеры с валидацией

### Валидация цены и количества

```rust
fn main() {
    let mut position = Position::new("BTC/USD");

    // Устанавливаем цену входа
    match position.set_entry_price(42000.0) {
        Ok(_) => println!("Entry price set"),
        Err(e) => println!("Error: {}", e),
    }

    // Устанавливаем количество
    match position.set_quantity(0.5) {
        Ok(_) => println!("Quantity set to {}", position.quantity()),
        Err(e) => println!("Error: {}", e),
    }

    // Попытка установить отрицательное количество
    match position.set_quantity(-1.0) {
        Ok(_) => println!("Quantity set"),
        Err(e) => println!("Validation failed: {}", e),
    }

    // Устанавливаем стоп-лосс
    match position.set_stop_loss(41000.0) {
        Ok(_) => println!("Stop loss set to ${:.2}", position.stop_loss().unwrap()),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nPosition summary:");
    println!("  Symbol: {}", position.symbol());
    println!("  Entry: ${:.2}", position.entry_price().unwrap_or(0.0));
    println!("  Quantity: {:.4}", position.quantity());
    println!("  Stop Loss: ${:.2}", position.stop_loss().unwrap_or(0.0));
}

struct Position {
    symbol: String,
    entry_price: Option<f64>,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Position {
    fn new(symbol: &str) -> Self {
        Position {
            symbol: symbol.to_string(),
            entry_price: None,
            quantity: 0.0,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Геттеры
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn entry_price(&self) -> Option<f64> {
        self.entry_price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn stop_loss(&self) -> Option<f64> {
        self.stop_loss
    }

    fn take_profit(&self) -> Option<f64> {
        self.take_profit
    }

    // Сеттеры с валидацией
    fn set_entry_price(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Entry price must be positive".to_string());
        }
        self.entry_price = Some(price);
        Ok(())
    }

    fn set_quantity(&mut self, qty: f64) -> Result<(), String> {
        if qty < 0.0 {
            return Err("Quantity cannot be negative".to_string());
        }
        self.quantity = qty;
        Ok(())
    }

    fn set_stop_loss(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(entry) = self.entry_price {
            if price >= entry {
                return Err("Stop loss must be below entry price for long position".to_string());
            }
        }
        self.stop_loss = Some(price);
        Ok(())
    }

    fn set_take_profit(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if let Some(entry) = self.entry_price {
            if price <= entry {
                return Err("Take profit must be above entry price for long position".to_string());
            }
        }
        self.take_profit = Some(price);
        Ok(())
    }
}
```

## Паттерн Builder с сеттерами

```rust
fn main() {
    // Fluent interface — цепочка сеттеров
    let order = OrderBuilder::new("BTC/USD")
        .price(42000.0)
        .quantity(0.5)
        .side(OrderSide::Buy)
        .stop_loss(41000.0)
        .take_profit(45000.0)
        .build();

    match order {
        Ok(o) => {
            println!("Order created:");
            println!("  Symbol: {}", o.symbol());
            println!("  Price: ${:.2}", o.price());
            println!("  Quantity: {:.4}", o.quantity());
            println!("  Total: ${:.2}", o.total_value());
        }
        Err(e) => println!("Failed to create order: {}", e),
    }
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Order {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn side(&self) -> OrderSide {
        self.side
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}

struct OrderBuilder {
    symbol: String,
    price: Option<f64>,
    quantity: Option<f64>,
    side: Option<OrderSide>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl OrderBuilder {
    fn new(symbol: &str) -> Self {
        OrderBuilder {
            symbol: symbol.to_string(),
            price: None,
            quantity: None,
            side: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Каждый сеттер возвращает self для цепочки
    fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    fn quantity(mut self, qty: f64) -> Self {
        self.quantity = Some(qty);
        self
    }

    fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    fn stop_loss(mut self, price: f64) -> Self {
        self.stop_loss = Some(price);
        self
    }

    fn take_profit(mut self, price: f64) -> Self {
        self.take_profit = Some(price);
        self
    }

    fn build(self) -> Result<Order, String> {
        let price = self.price.ok_or("Price is required")?;
        let quantity = self.quantity.ok_or("Quantity is required")?;
        let side = self.side.ok_or("Side is required")?;

        if price <= 0.0 {
            return Err("Price must be positive".to_string());
        }
        if quantity <= 0.0 {
            return Err("Quantity must be positive".to_string());
        }

        Ok(Order {
            symbol: self.symbol,
            price,
            quantity,
            side,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
        })
    }
}
```

## Геттеры для вложенных структур

```rust
fn main() {
    let portfolio = Portfolio::new("Main Trading Account");

    // Добавляем позиции через метод
    let mut portfolio = portfolio;
    portfolio.add_position("BTC/USD", 42000.0, 0.5);
    portfolio.add_position("ETH/USD", 2200.0, 5.0);
    portfolio.add_position("SOL/USD", 100.0, 50.0);

    println!("Portfolio: {}", portfolio.name());
    println!("Positions: {}", portfolio.position_count());
    println!("Total Value: ${:.2}", portfolio.total_value());

    // Доступ к позициям через геттер
    for pos in portfolio.positions() {
        println!("  {} - ${:.2}", pos.symbol(), pos.value());
    }
}

struct PortfolioPosition {
    symbol: String,
    entry_price: f64,
    quantity: f64,
}

impl PortfolioPosition {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn entry_price(&self) -> f64 {
        self.entry_price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn value(&self) -> f64 {
        self.entry_price * self.quantity
    }
}

struct Portfolio {
    name: String,
    positions: Vec<PortfolioPosition>,
}

impl Portfolio {
    fn new(name: &str) -> Self {
        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    // Геттер возвращает срез — неизменяемый доступ к коллекции
    fn positions(&self) -> &[PortfolioPosition] {
        &self.positions
    }

    fn position_count(&self) -> usize {
        self.positions.len()
    }

    fn total_value(&self) -> f64 {
        self.positions.iter().map(|p| p.value()).sum()
    }

    // Метод для добавления (не классический сеттер)
    fn add_position(&mut self, symbol: &str, price: f64, qty: f64) {
        self.positions.push(PortfolioPosition {
            symbol: symbol.to_string(),
            entry_price: price,
            quantity: qty,
        });
    }
}
```

## Защита через типы — альтернатива сеттерам

```rust
fn main() {
    // Цена не может быть отрицательной — гарантировано типом!
    let price = match ValidatedPrice::new(42000.0) {
        Some(p) => p,
        None => {
            println!("Invalid price!");
            return;
        }
    };

    println!("Price: ${:.2}", price.value());

    // Количество с гарантией положительного значения
    let qty = match PositiveQuantity::new(0.5) {
        Some(q) => q,
        None => {
            println!("Invalid quantity!");
            return;
        }
    };

    // Теперь можем безопасно использовать
    let order = SafeOrder::new("BTC/USD", price, qty);
    println!("Order value: ${:.2}", order.total_value());
}

// Новый тип с гарантией валидности
struct ValidatedPrice(f64);

impl ValidatedPrice {
    fn new(value: f64) -> Option<Self> {
        if value > 0.0 {
            Some(ValidatedPrice(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

struct PositiveQuantity(f64);

impl PositiveQuantity {
    fn new(value: f64) -> Option<Self> {
        if value > 0.0 {
            Some(PositiveQuantity(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

struct SafeOrder {
    symbol: String,
    price: ValidatedPrice,
    quantity: PositiveQuantity,
}

impl SafeOrder {
    // Принимаем уже валидированные типы — сеттер не нужен!
    fn new(symbol: &str, price: ValidatedPrice, quantity: PositiveQuantity) -> Self {
        SafeOrder {
            symbol: symbol.to_string(),
            price,
            quantity,
        }
    }

    fn total_value(&self) -> f64 {
        self.price.value() * self.quantity.value()
    }
}
```

## Мутабельный доступ через геттеры

```rust
fn main() {
    let mut account = TradingAccount::new("ACC-001", 10000.0);

    println!("Initial balance: ${:.2}", account.balance());

    // Получаем мутабельную ссылку на настройки
    account.settings_mut().max_position_size = 5000.0;
    account.settings_mut().max_leverage = 5.0;

    println!("Max position: ${:.2}", account.settings().max_position_size);
    println!("Max leverage: {}x", account.settings().max_leverage);
}

struct AccountSettings {
    max_position_size: f64,
    max_leverage: f64,
    risk_per_trade: f64,
}

impl Default for AccountSettings {
    fn default() -> Self {
        AccountSettings {
            max_position_size: 1000.0,
            max_leverage: 1.0,
            risk_per_trade: 2.0,
        }
    }
}

struct TradingAccount {
    id: String,
    balance: f64,
    settings: AccountSettings,
}

impl TradingAccount {
    fn new(id: &str, balance: f64) -> Self {
        TradingAccount {
            id: id.to_string(),
            balance,
            settings: AccountSettings::default(),
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn balance(&self) -> f64 {
        self.balance
    }

    // Иммутабельный геттер для настроек
    fn settings(&self) -> &AccountSettings {
        &self.settings
    }

    // Мутабельный геттер — позволяет изменять настройки
    fn settings_mut(&mut self) -> &mut AccountSettings {
        &mut self.settings
    }
}
```

## Практический пример: риск-менеджер

```rust
fn main() {
    let mut risk_manager = RiskManager::new(10000.0);

    // Настраиваем параметры риска
    risk_manager.set_max_risk_per_trade(2.0).unwrap();
    risk_manager.set_max_daily_loss(5.0).unwrap();
    risk_manager.set_max_open_positions(5).unwrap();

    println!("Risk Manager Settings:");
    println!("  Max risk per trade: {}%", risk_manager.max_risk_per_trade());
    println!("  Max daily loss: {}%", risk_manager.max_daily_loss());
    println!("  Max positions: {}", risk_manager.max_open_positions());

    // Проверяем сделку
    let trade_risk = 150.0; // $150 риска
    if risk_manager.can_take_trade(trade_risk) {
        println!("\nTrade approved!");
        risk_manager.record_trade(trade_risk);
    }

    // Рассчитываем размер позиции
    let entry = 42000.0;
    let stop_loss = 41000.0;
    let position_size = risk_manager.calculate_position_size(entry, stop_loss);
    println!("Recommended position size: {:.4} units", position_size);

    // Статус дня
    println!("\nDaily Stats:");
    println!("  Trades taken: {}", risk_manager.trades_today());
    println!("  Risk used: ${:.2}", risk_manager.daily_risk_used());
    println!("  Remaining risk: ${:.2}", risk_manager.remaining_daily_risk());
}

struct RiskManager {
    account_balance: f64,
    max_risk_per_trade: f64,     // процент
    max_daily_loss: f64,         // процент
    max_open_positions: usize,
    daily_risk_used: f64,
    trades_today: usize,
}

impl RiskManager {
    fn new(balance: f64) -> Self {
        RiskManager {
            account_balance: balance,
            max_risk_per_trade: 1.0,
            max_daily_loss: 3.0,
            max_open_positions: 3,
            daily_risk_used: 0.0,
            trades_today: 0,
        }
    }

    // Геттеры
    fn account_balance(&self) -> f64 {
        self.account_balance
    }

    fn max_risk_per_trade(&self) -> f64 {
        self.max_risk_per_trade
    }

    fn max_daily_loss(&self) -> f64 {
        self.max_daily_loss
    }

    fn max_open_positions(&self) -> usize {
        self.max_open_positions
    }

    fn daily_risk_used(&self) -> f64 {
        self.daily_risk_used
    }

    fn trades_today(&self) -> usize {
        self.trades_today
    }

    // Вычисляемые геттеры
    fn max_risk_amount(&self) -> f64 {
        self.account_balance * (self.max_risk_per_trade / 100.0)
    }

    fn max_daily_loss_amount(&self) -> f64 {
        self.account_balance * (self.max_daily_loss / 100.0)
    }

    fn remaining_daily_risk(&self) -> f64 {
        self.max_daily_loss_amount() - self.daily_risk_used
    }

    // Сеттеры с валидацией
    fn set_max_risk_per_trade(&mut self, percent: f64) -> Result<(), String> {
        if percent <= 0.0 || percent > 10.0 {
            return Err("Risk per trade must be between 0 and 10%".to_string());
        }
        self.max_risk_per_trade = percent;
        Ok(())
    }

    fn set_max_daily_loss(&mut self, percent: f64) -> Result<(), String> {
        if percent <= 0.0 || percent > 20.0 {
            return Err("Daily loss limit must be between 0 and 20%".to_string());
        }
        if percent < self.max_risk_per_trade {
            return Err("Daily loss must be >= risk per trade".to_string());
        }
        self.max_daily_loss = percent;
        Ok(())
    }

    fn set_max_open_positions(&mut self, count: usize) -> Result<(), String> {
        if count == 0 || count > 20 {
            return Err("Position count must be between 1 and 20".to_string());
        }
        self.max_open_positions = count;
        Ok(())
    }

    // Бизнес-логика
    fn can_take_trade(&self, risk_amount: f64) -> bool {
        risk_amount <= self.max_risk_amount() &&
        self.daily_risk_used + risk_amount <= self.max_daily_loss_amount() &&
        self.trades_today < self.max_open_positions
    }

    fn record_trade(&mut self, risk_amount: f64) {
        self.daily_risk_used += risk_amount;
        self.trades_today += 1;
    }

    fn calculate_position_size(&self, entry: f64, stop_loss: f64) -> f64 {
        let risk_per_unit = (entry - stop_loss).abs();
        if risk_per_unit == 0.0 {
            return 0.0;
        }
        self.max_risk_amount() / risk_per_unit
    }

    fn reset_daily_stats(&mut self) {
        self.daily_risk_used = 0.0;
        self.trades_today = 0;
    }
}
```

## Что мы узнали

| Паттерн | Пример | Когда использовать |
|---------|--------|-------------------|
| Простой геттер | `fn value(&self) -> f64` | Чтение примитивов |
| Геттер-ссылка | `fn name(&self) -> &str` | Строки, большие объекты |
| Геттер Option | `fn price(&self) -> Option<f64>` | Опциональные поля |
| Вычисляемый геттер | `fn total(&self) -> f64` | Производные значения |
| Сеттер с bool | `fn set(&mut self, v: T) -> bool` | Простая валидация |
| Сеттер с Result | `fn set(&mut self, v: T) -> Result<(), E>` | Детальные ошибки |
| Builder сеттер | `fn price(mut self, v: f64) -> Self` | Fluent interface |
| Мутабельный геттер | `fn data_mut(&mut self) -> &mut T` | Доступ к вложенным данным |

## Домашнее задание

1. Создай структуру `TradeJournal` с геттерами для статистики (win rate, average profit, total trades) и сеттером для добавления сделок с валидацией

2. Реализуй `PriceAlert` с геттерами для текущей цены, целевой цены и направления (above/below), сеттером для изменения целевой цены с проверкой

3. Напиши `PositionSizer` с настраиваемыми параметрами (риск на сделку, максимальный размер позиции) через сеттеры с валидацией и методом расчёта размера позиции

4. Создай `OrderBook` с геттерами для лучших bid/ask, спреда и глубины, методами для добавления и удаления ордеров

## Навигация

[← Предыдущий день](../068-privacy-visibility/ru.md) | [Следующий день →](../070-mutable-references-in-structs/ru.md)
