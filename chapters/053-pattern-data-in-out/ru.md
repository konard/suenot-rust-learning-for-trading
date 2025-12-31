# День 53: Паттерн "Data In and Out" — Данные на входе и выходе

## Аналогия из трейдинга

Представь работу трейдинговой компании. Когда клиент отправляет ордер на исполнение, происходит одно из трёх:

1. **Передача владения** — клиент полностью передаёт ордер брокеру. Ордер теперь принадлежит брокеру, клиент больше не может его изменить.

2. **Показать для анализа** — аналитик смотрит на ордер, но не забирает его. Клиент остаётся владельцем и может продолжать с ним работать.

3. **Передать для модификации** — риск-менеджер получает ордер, корректирует объём позиции, и возвращает обратно.

В Rust это три способа передачи данных в функции: **by value** (передача владения), **by reference** (заимствование для чтения), и **by mutable reference** (заимствование для изменения).

## Теория: Как данные текут в Rust

### Правило потока данных

Когда данные попадают в функцию, есть три сценария:

```
┌─────────────────────────────────────────────────────────────┐
│                    ДАННЫЕ НА ВХОДЕ                          │
├─────────────────────────────────────────────────────────────┤
│  fn process(data: T)      → Забрать владение (move)        │
│  fn analyze(data: &T)     → Посмотреть (borrow)            │
│  fn modify(data: &mut T)  → Изменить (mutable borrow)      │
├─────────────────────────────────────────────────────────────┤
│                    ДАННЫЕ НА ВЫХОДЕ                         │
├─────────────────────────────────────────────────────────────┤
│  fn create() -> T         → Создать и отдать владение      │
│  fn get(&self) -> &T      → Дать посмотреть                │
│  fn get_mut(&mut self) -> &mut T → Дать изменить           │
└─────────────────────────────────────────────────────────────┘
```

## Паттерн 1: Принять владение и вернуть результат

Функция забирает данные, обрабатывает их и возвращает новый результат.

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: 42000.0,
    };

    // Передаём ордер в функцию — order больше недоступен!
    let executed = execute_order(order);

    // println!("{}", order.symbol); // Ошибка! order перемещён
    println!("Executed: {} at ${}", executed.symbol, executed.fill_price);
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

struct ExecutedOrder {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    fill_price: f64,
    commission: f64,
}

fn execute_order(order: Order) -> ExecutedOrder {
    // Функция владеет order, может использовать его поля
    let slippage = 0.001; // 0.1% проскальзывание
    let commission_rate = 0.0004; // 0.04% комиссия

    let fill_price = match order.side {
        OrderSide::Buy => order.price * (1.0 + slippage),
        OrderSide::Sell => order.price * (1.0 - slippage),
    };

    let commission = fill_price * order.quantity * commission_rate;

    ExecutedOrder {
        symbol: order.symbol,  // Перемещаем String
        side: order.side,
        quantity: order.quantity,
        fill_price,
        commission,
    }
}
```

**Когда использовать:** Когда функция полностью "потребляет" данные и создаёт что-то новое.

## Паттерн 2: Заимствовать для анализа

Функция только читает данные, не забирая владение.

```rust
fn main() {
    let portfolio = Portfolio {
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: 1.5, avg_price: 40000.0 },
            Position { symbol: String::from("ETH"), quantity: 10.0, avg_price: 2500.0 },
            Position { symbol: String::from("SOL"), quantity: 100.0, avg_price: 100.0 },
        ],
    };

    // Передаём ссылку — portfolio остаётся доступным
    let total = calculate_portfolio_value(&portfolio, &get_current_prices());
    println!("Portfolio value: ${:.2}", total);

    // portfolio всё ещё доступен!
    let risk = analyze_portfolio_risk(&portfolio);
    println!("Portfolio risk score: {:.2}", risk);

    // Можем вызвать несколько функций с одними данными
    let diversification = calculate_diversification(&portfolio);
    println!("Diversification index: {:.2}", diversification);
}

struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

struct Portfolio {
    positions: Vec<Position>,
}

use std::collections::HashMap;

fn get_current_prices() -> HashMap<String, f64> {
    let mut prices = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2800.0);
    prices.insert(String::from("SOL"), 120.0);
    prices
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &HashMap<String, f64>) -> f64 {
    portfolio.positions.iter()
        .map(|pos| {
            let current_price = prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * current_price
        })
        .sum()
}

fn analyze_portfolio_risk(portfolio: &Portfolio) -> f64 {
    // Простой расчёт риска на основе концентрации
    let total_value: f64 = portfolio.positions.iter()
        .map(|p| p.quantity * p.avg_price)
        .sum();

    if total_value == 0.0 {
        return 0.0;
    }

    let max_position_pct = portfolio.positions.iter()
        .map(|p| (p.quantity * p.avg_price) / total_value)
        .fold(0.0, f64::max);

    // Риск выше, если одна позиция доминирует
    max_position_pct * 100.0
}

fn calculate_diversification(portfolio: &Portfolio) -> f64 {
    // Индекс диверсификации: 1 / сумму квадратов долей
    let total_value: f64 = portfolio.positions.iter()
        .map(|p| p.quantity * p.avg_price)
        .sum();

    if total_value == 0.0 {
        return 0.0;
    }

    let sum_of_squares: f64 = portfolio.positions.iter()
        .map(|p| {
            let weight = (p.quantity * p.avg_price) / total_value;
            weight * weight
        })
        .sum();

    if sum_of_squares == 0.0 {
        0.0
    } else {
        1.0 / sum_of_squares
    }
}
```

**Когда использовать:** Когда нужно только прочитать данные, не изменяя их.

## Паттерн 3: Изменяемое заимствование

Функция получает возможность изменить данные.

```rust
fn main() {
    let mut trading_account = TradingAccount {
        balance: 10000.0,
        positions: Vec::new(),
        trade_history: Vec::new(),
    };

    println!("Initial balance: ${:.2}", trading_account.balance);

    // Передаём мутабельную ссылку — функция может изменять
    open_position(&mut trading_account, "BTC/USDT", 0.1, 42000.0);
    println!("After opening BTC: ${:.2}", trading_account.balance);

    open_position(&mut trading_account, "ETH/USDT", 2.0, 2800.0);
    println!("After opening ETH: ${:.2}", trading_account.balance);

    // Закрываем позицию
    close_position(&mut trading_account, "BTC/USDT", 43500.0);
    println!("After closing BTC: ${:.2}", trading_account.balance);

    // Смотрим историю
    print_trade_history(&trading_account);
}

struct TradingAccount {
    balance: f64,
    positions: Vec<AccountPosition>,
    trade_history: Vec<Trade>,
}

struct AccountPosition {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    pnl: f64,
}

fn open_position(account: &mut TradingAccount, symbol: &str, quantity: f64, price: f64) {
    let cost = quantity * price;

    if account.balance >= cost {
        account.balance -= cost;
        account.positions.push(AccountPosition {
            symbol: String::from(symbol),
            quantity,
            entry_price: price,
        });
        account.trade_history.push(Trade {
            symbol: String::from(symbol),
            side: String::from("BUY"),
            quantity,
            price,
            pnl: 0.0,
        });
    }
}

fn close_position(account: &mut TradingAccount, symbol: &str, exit_price: f64) {
    if let Some(pos_index) = account.positions.iter().position(|p| p.symbol == symbol) {
        let position = account.positions.remove(pos_index);
        let revenue = position.quantity * exit_price;
        let pnl = (exit_price - position.entry_price) * position.quantity;

        account.balance += revenue;
        account.trade_history.push(Trade {
            symbol: String::from(symbol),
            side: String::from("SELL"),
            quantity: position.quantity,
            price: exit_price,
            pnl,
        });
    }
}

fn print_trade_history(account: &TradingAccount) {
    println!("\n=== Trade History ===");
    for trade in &account.trade_history {
        println!(
            "{} {} {} @ ${:.2} (PnL: ${:.2})",
            trade.side, trade.quantity, trade.symbol, trade.price, trade.pnl
        );
    }
}
```

**Когда использовать:** Когда функция должна изменить данные.

## Паттерн 4: Принять и вернуть владение

Иногда нужно забрать данные, обработать и вернуть обратно (трансформация).

```rust
fn main() {
    let mut candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42200.0, volume: 100.0 },
        Candle { open: 42200.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 150.0 },
        Candle { open: 42600.0, high: 43000.0, low: 42400.0, close: 42900.0, volume: 120.0 },
    ];

    // Передаём владение и получаем его обратно с вычисленными индикаторами
    candles = add_sma_indicator(candles, 2);

    // Теперь candles содержит SMA
    for (i, candle) in candles.iter().enumerate() {
        println!("Candle {}: close={}, SMA={:?}", i, candle.close, candle.sma);
    }
}

struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    #[allow(dead_code)]
    sma: Option<f64>,
}

// Неявно определяем структуру с sma по умолчанию
impl Default for Candle {
    fn default() -> Self {
        Candle {
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            sma: None,
        }
    }
}

fn add_sma_indicator(mut candles: Vec<Candle>, period: usize) -> Vec<Candle> {
    for i in 0..candles.len() {
        if i + 1 >= period {
            let sum: f64 = candles[i + 1 - period..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            candles[i].sma = Some(sum / period as f64);
        }
    }
    candles  // Возвращаем владение
}
```

**Когда использовать:** Когда нужно трансформировать данные целиком.

## Паттерн 5: Строитель (Builder Pattern)

Создание сложных объектов через цепочку вызовов.

```rust
fn main() {
    let strategy = StrategyBuilder::new("SMA Crossover")
        .with_symbol("BTC/USDT")
        .with_timeframe("1h")
        .with_fast_period(10)
        .with_slow_period(20)
        .with_risk_percent(2.0)
        .build();

    println!("Strategy: {}", strategy.name);
    println!("Symbol: {}", strategy.symbol);
    println!("Timeframe: {}", strategy.timeframe);
    println!("Fast SMA: {}", strategy.fast_period);
    println!("Slow SMA: {}", strategy.slow_period);
    println!("Risk: {}%", strategy.risk_percent);
}

struct Strategy {
    name: String,
    symbol: String,
    timeframe: String,
    fast_period: usize,
    slow_period: usize,
    risk_percent: f64,
}

struct StrategyBuilder {
    name: String,
    symbol: String,
    timeframe: String,
    fast_period: usize,
    slow_period: usize,
    risk_percent: f64,
}

impl StrategyBuilder {
    fn new(name: &str) -> Self {
        StrategyBuilder {
            name: String::from(name),
            symbol: String::from("BTC/USDT"),
            timeframe: String::from("1h"),
            fast_period: 10,
            slow_period: 20,
            risk_percent: 1.0,
        }
    }

    fn with_symbol(mut self, symbol: &str) -> Self {
        self.symbol = String::from(symbol);
        self  // Возвращаем self для цепочки
    }

    fn with_timeframe(mut self, timeframe: &str) -> Self {
        self.timeframe = String::from(timeframe);
        self
    }

    fn with_fast_period(mut self, period: usize) -> Self {
        self.fast_period = period;
        self
    }

    fn with_slow_period(mut self, period: usize) -> Self {
        self.slow_period = period;
        self
    }

    fn with_risk_percent(mut self, percent: f64) -> Self {
        self.risk_percent = percent;
        self
    }

    fn build(self) -> Strategy {
        Strategy {
            name: self.name,
            symbol: self.symbol,
            timeframe: self.timeframe,
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            risk_percent: self.risk_percent,
        }
    }
}
```

**Когда использовать:** Для создания сложных объектов с множеством опциональных параметров.

## Паттерн 6: Обработка с коллбэком

Передача функции для обработки каждого элемента.

```rust
fn main() {
    let trades = vec![
        TradeRecord { symbol: String::from("BTC"), pnl: 500.0 },
        TradeRecord { symbol: String::from("ETH"), pnl: -200.0 },
        TradeRecord { symbol: String::from("BTC"), pnl: 300.0 },
        TradeRecord { symbol: String::from("SOL"), pnl: -50.0 },
        TradeRecord { symbol: String::from("BTC"), pnl: 150.0 },
    ];

    // Обрабатываем каждую прибыльную сделку
    process_trades(&trades, |trade| {
        if trade.pnl > 0.0 {
            println!("Profit on {}: ${:.2}", trade.symbol, trade.pnl);
        }
    });

    // Считаем общий PnL по BTC
    let btc_pnl = aggregate_trades(&trades, |trade| {
        if trade.symbol == "BTC" {
            trade.pnl
        } else {
            0.0
        }
    });
    println!("\nTotal BTC PnL: ${:.2}", btc_pnl);
}

struct TradeRecord {
    symbol: String,
    pnl: f64,
}

fn process_trades<F>(trades: &[TradeRecord], mut processor: F)
where
    F: FnMut(&TradeRecord),
{
    for trade in trades {
        processor(trade);
    }
}

fn aggregate_trades<F>(trades: &[TradeRecord], selector: F) -> f64
where
    F: Fn(&TradeRecord) -> f64,
{
    trades.iter().map(|t| selector(t)).sum()
}
```

**Когда использовать:** Когда нужна гибкая обработка данных с разной логикой.

## Практический пример: Торговый движок

```rust
fn main() {
    let mut engine = TradingEngine::new(10000.0);

    // Получаем рыночные данные
    let market_data = MarketData {
        symbol: String::from("BTC/USDT"),
        bid: 41990.0,
        ask: 42010.0,
        last: 42000.0,
    };

    // Генерируем сигнал (анализируем данные, не забирая владение)
    if let Some(signal) = engine.generate_signal(&market_data) {
        println!("Signal: {:?} {} at ${}", signal.side, signal.symbol, signal.price);

        // Создаём ордер из сигнала (сигнал перемещается)
        let order = engine.create_order(signal);
        println!("Created order: {} {} @ ${}", order.side, order.quantity, order.price);

        // Исполняем ордер (изменяем состояние движка)
        if let Ok(execution) = engine.execute(&mut order.clone()) {
            println!("Executed: {} {} @ ${}", execution.side, execution.quantity, execution.fill_price);
        }
    }

    // Смотрим текущее состояние
    engine.print_status();
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
}

#[derive(Debug)]
struct Signal {
    symbol: String,
    side: Side,
    price: f64,
    strength: f64,
}

#[derive(Clone)]
struct TradeOrder {
    symbol: String,
    side: Side,
    quantity: f64,
    price: f64,
}

struct Execution {
    symbol: String,
    side: Side,
    quantity: f64,
    fill_price: f64,
}

struct EnginePosition {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

struct TradingEngine {
    balance: f64,
    positions: Vec<EnginePosition>,
    risk_per_trade: f64,
}

impl TradingEngine {
    fn new(initial_balance: f64) -> Self {
        TradingEngine {
            balance: initial_balance,
            positions: Vec::new(),
            risk_per_trade: 0.02, // 2% риска на сделку
        }
    }

    // Анализирует рынок, не забирая владение данных
    fn generate_signal(&self, data: &MarketData) -> Option<Signal> {
        // Простая логика: покупаем если спред узкий
        let spread_pct = (data.ask - data.bid) / data.last * 100.0;

        if spread_pct < 0.1 {
            Some(Signal {
                symbol: data.symbol.clone(),
                side: Side::Buy,
                price: data.ask,
                strength: 1.0 - spread_pct,
            })
        } else {
            None
        }
    }

    // Создаёт ордер из сигнала (забирает владение сигналом)
    fn create_order(&self, signal: Signal) -> TradeOrder {
        let risk_amount = self.balance * self.risk_per_trade;
        let quantity = risk_amount / signal.price;

        TradeOrder {
            symbol: signal.symbol,
            side: signal.side,
            quantity,
            price: signal.price,
        }
    }

    // Исполняет ордер, изменяя состояние движка
    fn execute(&mut self, order: &mut TradeOrder) -> Result<Execution, String> {
        let cost = order.quantity * order.price;

        match order.side {
            Side::Buy => {
                if self.balance < cost {
                    return Err(String::from("Insufficient balance"));
                }

                self.balance -= cost;
                self.positions.push(EnginePosition {
                    symbol: order.symbol.clone(),
                    quantity: order.quantity,
                    avg_price: order.price,
                });
            }
            Side::Sell => {
                // Логика продажи...
            }
        }

        Ok(Execution {
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            quantity: order.quantity,
            fill_price: order.price,
        })
    }

    fn print_status(&self) {
        println!("\n=== Engine Status ===");
        println!("Balance: ${:.2}", self.balance);
        println!("Positions: {}", self.positions.len());
        for pos in &self.positions {
            println!("  {} {} @ ${:.2}", pos.symbol, pos.quantity, pos.avg_price);
        }
    }
}
```

## Упражнения

### Упражнение 1: Обработчик котировок

Создай функцию, которая принимает вектор котировок и возвращает обработанный вектор с добавленными индикаторами (SMA, RSI).

```rust
fn process_quotes(quotes: Vec<Quote>) -> Vec<ProcessedQuote> {
    // Твой код здесь
}
```

### Упражнение 2: Риск-менеджер

Создай структуру `RiskManager` с методами:
- `check_position(&self, position: &Position) -> RiskReport` — анализ позиции
- `adjust_position(&mut self, position: &mut Position)` — корректировка
- `close_if_needed(self, position: Position) -> Option<Position>` — закрытие по условию

### Упражнение 3: Конвейер обработки данных

Реализуй цепочку обработки рыночных данных:

```rust
let result = market_data
    .validate()     // Проверка валидности
    .normalize()    // Нормализация
    .calculate()    // Расчёт индикаторов
    .filter()       // Фильтрация
    .collect();     // Сбор результатов
```

### Упражнение 4: Генератор отчётов

Создай функцию-строитель для отчёта по торговле:

```rust
let report = ReportBuilder::new()
    .with_trades(&trades)
    .with_period("2024-01")
    .with_metrics(&["pnl", "win_rate", "sharpe"])
    .build()?;
```

## Что мы узнали

| Паттерн | Сигнатура | Когда использовать |
|---------|-----------|-------------------|
| Забрать владение | `fn(T) -> U` | Потребление/трансформация |
| Читать | `fn(&T) -> U` | Анализ без изменения |
| Изменять | `fn(&mut T)` | Модификация на месте |
| Трансформировать | `fn(T) -> T` | Изменить и вернуть |
| Строитель | `fn(self) -> Self` | Цепочка вызовов |
| Коллбэк | `fn(&T, F) where F: Fn(&T)` | Гибкая обработка |

## Домашнее задание

1. **Торговый журнал**: Создай структуру `TradeJournal` с методами:
   - `record(&mut self, trade: Trade)` — запись сделки
   - `analyze(&self) -> JournalStats` — анализ статистики
   - `export(self) -> String` — экспорт в JSON (потребляет журнал)

2. **Оптимизатор позиций**: Напиши функцию `optimize_positions(portfolio: &mut Portfolio, target_allocation: &Allocation)`, которая перебалансирует портфель.

3. **Генератор сигналов**: Реализуй трейт `SignalGenerator` с методом `fn generate(&self, data: &MarketData) -> Option<Signal>` и создай несколько реализаций (SMA crossover, RSI, Bollinger Bands).

4. **Бэктест-движок**: Создай структуру `Backtest` с паттерном Builder:
   ```rust
   let result = Backtest::new()
       .with_strategy(strategy)      // Забирает стратегию
       .with_data(&historical_data)  // Заимствует данные
       .with_initial_capital(10000.0)
       .run()?;                       // Запуск и результат
   ```

## Навигация

[← Предыдущий день](../052-ownership-move-semantics/ru.md) | [Следующий день →](../054-pattern-borrowed-vs-owned/ru.md)
