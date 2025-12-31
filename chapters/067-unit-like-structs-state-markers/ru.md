# День 67: Unit-like структуры — маркеры состояния

## Аналогия из трейдинга

В торговле каждый ордер имеет **статус**:
- **Pending** — ожидает исполнения
- **Filled** — исполнен полностью
- **PartiallyFilled** — исполнен частично
- **Cancelled** — отменён
- **Rejected** — отклонён

Эти статусы не содержат данных — они просто **маркеры состояния**. В Rust для таких случаев используются **unit-like структуры** — структуры без полей.

## Что такое unit-like структура?

Unit-like структура — это структура без полей, которая занимает **ноль байт** в памяти:

```rust
// Unit-like структуры — не содержат данных
struct Pending;
struct Filled;
struct Cancelled;

fn main() {
    let status = Pending;

    // Размер = 0 байт!
    println!("Size of Pending: {} bytes", std::mem::size_of::<Pending>());
}
```

## Зачем нужны пустые структуры?

### 1. Маркеры типов (Type Markers)

```rust
// Статусы рынка
struct MarketOpen;
struct MarketClosed;
struct PreMarket;
struct AfterHours;

fn main() {
    let current_status = MarketOpen;

    println!("Market is now open!");
    println!("Type: {}", std::any::type_name::<MarketOpen>());
}
```

### 2. Состояния в обобщённых типах

```rust
// Маркеры состояний ордера
struct New;
struct Submitted;
struct Executed;
struct Cancelled;

// Ордер с состоянием как параметр типа
struct Order<State> {
    symbol: String,
    quantity: f64,
    price: f64,
    _state: std::marker::PhantomData<State>,
}

impl Order<New> {
    fn new(symbol: &str, quantity: f64, price: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            quantity,
            price,
            _state: std::marker::PhantomData,
        }
    }

    fn submit(self) -> Order<Submitted> {
        println!("Submitting order for {} {} @ {}",
                 self.quantity, self.symbol, self.price);
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }
}

impl Order<Submitted> {
    fn execute(self) -> Order<Executed> {
        println!("Order executed!");
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }

    fn cancel(self) -> Order<Cancelled> {
        println!("Order cancelled!");
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }
}

fn main() {
    // Компилятор проверяет правильность переходов!
    let order = Order::<New>::new("BTC/USDT", 0.5, 42000.0);
    let submitted = order.submit();
    let executed = submitted.execute();

    // Ошибка компиляции! Нельзя отменить уже исполненный ордер
    // executed.cancel(); // не скомпилируется
}
```

## Практический пример: торговые сигналы

```rust
// Маркеры сигналов
struct BuySignal;
struct SellSignal;
struct HoldSignal;

// Универсальная функция для обработки сигналов
trait TradingSignal {
    fn action(&self) -> &'static str;
    fn emoji(&self) -> &'static str;
}

impl TradingSignal for BuySignal {
    fn action(&self) -> &'static str { "BUY" }
    fn emoji(&self) -> &'static str { "🟢" }
}

impl TradingSignal for SellSignal {
    fn action(&self) -> &'static str { "SELL" }
    fn emoji(&self) -> &'static str { "🔴" }
}

impl TradingSignal for HoldSignal {
    fn action(&self) -> &'static str { "HOLD" }
    fn emoji(&self) -> &'static str { "🟡" }
}

fn analyze_market(price: f64, sma: f64) -> Box<dyn TradingSignal> {
    if price > sma * 1.02 {
        Box::new(SellSignal)  // Цена выше SMA на 2%
    } else if price < sma * 0.98 {
        Box::new(BuySignal)   // Цена ниже SMA на 2%
    } else {
        Box::new(HoldSignal)  // Цена около SMA
    }
}

fn main() {
    let current_price = 42000.0;
    let sma_20 = 41500.0;

    let signal = analyze_market(current_price, sma_20);

    println!("=== Market Analysis ===");
    println!("Price: ${:.2}", current_price);
    println!("SMA(20): ${:.2}", sma_20);
    println!("Signal: {} {}", signal.emoji(), signal.action());
}
```

## Состояния торговой стратегии

```rust
use std::marker::PhantomData;

// Состояния стратегии
struct Backtesting;
struct PaperTrading;
struct LiveTrading;

struct Strategy<Mode> {
    name: String,
    capital: f64,
    _mode: PhantomData<Mode>,
}

impl Strategy<Backtesting> {
    fn new(name: &str, capital: f64) -> Self {
        println!("[BACKTEST] Strategy '{}' created", name);
        Strategy {
            name: name.to_string(),
            capital,
            _mode: PhantomData,
        }
    }

    fn run_backtest(&self, data: &[f64]) {
        println!("[BACKTEST] Running on {} candles", data.len());
        // Логика бэктеста
    }

    fn to_paper(self) -> Strategy<PaperTrading> {
        println!("[PAPER] Switching to paper trading");
        Strategy {
            name: self.name,
            capital: self.capital,
            _mode: PhantomData,
        }
    }
}

impl Strategy<PaperTrading> {
    fn simulate_trade(&self, symbol: &str, side: &str, amount: f64) {
        println!("[PAPER] {} {} {} (simulated)", side, amount, symbol);
    }

    fn to_live(self) -> Strategy<LiveTrading> {
        println!("[LIVE] ⚠️ Going LIVE with ${:.2}!", self.capital);
        Strategy {
            name: self.name,
            capital: self.capital,
            _mode: PhantomData,
        }
    }
}

impl Strategy<LiveTrading> {
    fn execute_trade(&self, symbol: &str, side: &str, amount: f64) {
        println!("[LIVE] 🚨 EXECUTING: {} {} {}", side, amount, symbol);
    }
}

fn main() {
    // Чёткий жизненный цикл стратегии
    let historical_data = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // 1. Сначала бэктест
    let strategy = Strategy::<Backtesting>::new("SMA Crossover", 10000.0);
    strategy.run_backtest(&historical_data);

    // 2. Затем paper trading
    let paper = strategy.to_paper();
    paper.simulate_trade("BTC/USDT", "BUY", 0.1);

    // 3. Только потом live
    let live = paper.to_live();
    live.execute_trade("BTC/USDT", "BUY", 0.1);

    // Нельзя сразу перейти в live из backtest!
    // let strategy2 = Strategy::<Backtesting>::new("Test", 1000.0);
    // strategy2.to_live(); // Ошибка компиляции!
}
```

## Маркеры направления позиции

```rust
struct Long;
struct Short;

struct Position<Direction> {
    symbol: String,
    entry_price: f64,
    size: f64,
    _direction: std::marker::PhantomData<Direction>,
}

impl<D> Position<D> {
    fn value(&self) -> f64 {
        self.entry_price * self.size
    }
}

impl Position<Long> {
    fn open_long(symbol: &str, price: f64, size: f64) -> Self {
        println!("Opening LONG {} @ {} x {}", symbol, price, size);
        Position {
            symbol: symbol.to_string(),
            entry_price: price,
            size,
            _direction: std::marker::PhantomData,
        }
    }

    fn calculate_pnl(&self, current_price: f64) -> f64 {
        // Long: прибыль при росте цены
        (current_price - self.entry_price) * self.size
    }
}

impl Position<Short> {
    fn open_short(symbol: &str, price: f64, size: f64) -> Self {
        println!("Opening SHORT {} @ {} x {}", symbol, price, size);
        Position {
            symbol: symbol.to_string(),
            entry_price: price,
            size,
            _direction: std::marker::PhantomData,
        }
    }

    fn calculate_pnl(&self, current_price: f64) -> f64 {
        // Short: прибыль при падении цены
        (self.entry_price - current_price) * self.size
    }
}

fn main() {
    let long_pos = Position::<Long>::open_long("BTC/USDT", 42000.0, 0.5);
    let short_pos = Position::<Short>::open_short("ETH/USDT", 2500.0, 2.0);

    let btc_current = 43000.0;
    let eth_current = 2400.0;

    println!("\n=== PnL Report ===");
    println!("BTC Long:  ${:+.2}", long_pos.calculate_pnl(btc_current));
    println!("ETH Short: ${:+.2}", short_pos.calculate_pnl(eth_current));
}
```

## Состояния подключения к бирже

```rust
struct Disconnected;
struct Connecting;
struct Connected;
struct Authenticated;

struct Exchange<State> {
    name: String,
    _state: std::marker::PhantomData<State>,
}

impl Exchange<Disconnected> {
    fn new(name: &str) -> Self {
        Exchange {
            name: name.to_string(),
            _state: std::marker::PhantomData,
        }
    }

    fn connect(self) -> Exchange<Connecting> {
        println!("[{}] Connecting...", self.name);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Connecting> {
    fn on_connected(self) -> Exchange<Connected> {
        println!("[{}] Connected!", self.name);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Connected> {
    fn authenticate(self, api_key: &str) -> Exchange<Authenticated> {
        println!("[{}] Authenticating with key {}...", self.name, &api_key[..8]);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Authenticated> {
    fn get_balance(&self) -> f64 {
        println!("[{}] Fetching balance...", self.name);
        10000.0  // Симуляция
    }

    fn place_order(&self, symbol: &str, side: &str, amount: f64) {
        println!("[{}] Placing {} {} {}", self.name, side, amount, symbol);
    }
}

fn main() {
    let exchange = Exchange::<Disconnected>::new("Binance");

    // Правильная последовательность
    let connecting = exchange.connect();
    let connected = connecting.on_connected();
    let authenticated = connected.authenticate("sk-1234567890abcdef");

    // Теперь можем торговать!
    let balance = authenticated.get_balance();
    println!("Balance: ${:.2}", balance);

    authenticated.place_order("BTC/USDT", "BUY", 0.1);

    // Нельзя торговать без аутентификации!
    // let ex = Exchange::<Connected>::new("Test");
    // ex.place_order(...); // Ошибка: метод не существует для Connected
}
```

## Маркеры валидации данных

```rust
struct Unvalidated;
struct Validated;

struct MarketData<State> {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
    _state: std::marker::PhantomData<State>,
}

impl MarketData<Unvalidated> {
    fn new(symbol: &str, price: f64, volume: f64, timestamp: u64) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            price,
            volume,
            timestamp,
            _state: std::marker::PhantomData,
        }
    }

    fn validate(self) -> Result<MarketData<Validated>, String> {
        // Проверки
        if self.price <= 0.0 {
            return Err("Price must be positive".to_string());
        }
        if self.volume < 0.0 {
            return Err("Volume cannot be negative".to_string());
        }
        if self.symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }

        Ok(MarketData {
            symbol: self.symbol,
            price: self.price,
            volume: self.volume,
            timestamp: self.timestamp,
            _state: std::marker::PhantomData,
        })
    }
}

impl MarketData<Validated> {
    // Только валидированные данные можно использовать для торговли
    fn calculate_notional(&self) -> f64 {
        self.price * self.volume
    }

    fn to_csv(&self) -> String {
        format!("{},{},{},{}", self.symbol, self.price, self.volume, self.timestamp)
    }
}

fn process_data(data: MarketData<Validated>) {
    println!("Processing: {} @ ${:.2}", data.symbol, data.price);
    println!("Notional: ${:.2}", data.calculate_notional());
}

fn main() {
    // Создаём невалидированные данные
    let raw_data = MarketData::<Unvalidated>::new("BTC/USDT", 42000.0, 1.5, 1704067200);

    // Валидируем
    match raw_data.validate() {
        Ok(valid_data) => {
            process_data(valid_data);
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    // Невалидные данные
    let bad_data = MarketData::<Unvalidated>::new("", -100.0, 1.0, 0);
    if let Err(e) = bad_data.validate() {
        println!("Expected error: {}", e);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `struct Name;` | Unit-like структура без полей |
| Zero-sized type | Не занимает память |
| Type marker | Маркировка типов на уровне компиляции |
| PhantomData | Фантомный параметр для обобщённых типов |
| State machine | Машина состояний с проверкой компилятором |

## Упражнения

### Упражнение 1: Статусы заявки

Создайте unit-like структуры для всех возможных статусов биржевой заявки и реализуйте переходы между ними.

```rust
struct OrderNew;
struct OrderPending;
struct OrderPartiallyFilled;
struct OrderFilled;
struct OrderCancelled;
struct OrderRejected;

// Реализуйте структуру BrokerOrder<Status> с методами для переходов
```

### Упражнение 2: Состояния торговой сессии

Реализуйте машину состояний для торговой сессии:
- PreMarket → MarketOpen → MarketClose → AfterHours → PreMarket

### Упражнение 3: Риск-менеджмент

Создайте систему с состояниями:
- RiskNormal — нормальный режим
- RiskWarning — предупреждение
- RiskCritical — критический уровень
- TradingHalted — торговля остановлена

### Упражнение 4: Валидация ордера

Реализуйте цепочку валидации ордера:
1. Unvalidated → SizeValidated → PriceValidated → RiskChecked → ReadyToSubmit

## Домашнее задание

1. Реализуйте полную машину состояний для торговой стратегии: Development → Testing → Staging → Production с соответствующими ограничениями для каждого состояния

2. Создайте систему управления портфелем с состояниями позиций: Opening → Open → Closing → Closed, где каждый переход требует определённых действий

3. Реализуйте WebSocket клиент для биржи с состояниями: Idle → Connecting → Connected → Subscribing → Streaming → Disconnecting

4. Создайте систему обработки рыночных данных с валидацией: RawTick → ParsedTick → ValidatedTick → EnrichedTick → StoredTick

## Навигация

[← Предыдущий день](../066-tuple-structs/ru.md) | [Следующий день →](../068-generic-structs/ru.md)
