# День 71: Enum с данными — OrderType с параметрами

## Аналогия из трейдинга

В прошлой главе мы создали простой enum `OrderSide` для направления сделки — Buy или Sell. Но реальные типы ордеров сложнее:

- **Market ордер** — просто исполняется по рыночной цене, параметров нет
- **Limit ордер** — нужна цена, по которой хотим исполнить
- **Stop ордер** — нужна триггер-цена и возможно лимит-цена
- **Trailing Stop** — нужен процент или величина отступа

Каждый тип ордера требует **разные данные**! В Rust enum-ы могут хранить данные внутри вариантов — это идеально подходит для моделирования типов ордеров.

## Enum с данными

В Rust каждый вариант enum-а может содержать данные:

```rust
enum OrderType {
    Market,                           // Без данных
    Limit(f64),                       // Цена
    Stop { trigger: f64, limit: Option<f64> },  // Именованные поля
    TrailingStop { percent: f64 },    // Процент отступа
}

fn main() {
    let order1 = OrderType::Market;
    let order2 = OrderType::Limit(42000.0);
    let order3 = OrderType::Stop {
        trigger: 41000.0,
        limit: Some(40950.0)
    };
    let order4 = OrderType::TrailingStop { percent: 2.5 };

    println!("Создали 4 разных типа ордеров!");
}
```

## Три формы данных в enum

### 1. Unit-вариант (без данных)

```rust
enum Signal {
    Hold,      // Ничего не делать
    Exit,      // Закрыть позицию
}

fn main() {
    let signal = Signal::Hold;

    match signal {
        Signal::Hold => println!("Держим позицию"),
        Signal::Exit => println!("Закрываем позицию"),
    }
}
```

### 2. Tuple-вариант (данные как кортеж)

```rust
enum OrderType {
    Market,
    Limit(f64),                    // Одно значение
    StopLimit(f64, f64),           // Два значения: stop, limit
    Iceberg(f64, f64, u32),        // Цена, общий размер, видимый размер
}

fn main() {
    let limit_order = OrderType::Limit(42000.0);
    let stop_limit = OrderType::StopLimit(41000.0, 40950.0);
    let iceberg = OrderType::Iceberg(42100.0, 10.0, 1);

    match limit_order {
        OrderType::Market => println!("Market ордер"),
        OrderType::Limit(price) => println!("Limit @ {}", price),
        OrderType::StopLimit(stop, limit) => {
            println!("Stop-Limit: trigger={}, limit={}", stop, limit)
        }
        OrderType::Iceberg(price, total, visible) => {
            println!("Iceberg: price={}, total={}, show={}", price, total, visible)
        }
    }
}
```

### 3. Struct-вариант (именованные поля)

```rust
enum Order {
    Market {
        symbol: String,
        quantity: f64,
    },
    Limit {
        symbol: String,
        quantity: f64,
        price: f64,
    },
    StopLoss {
        symbol: String,
        quantity: f64,
        trigger_price: f64,
        limit_price: Option<f64>,
    },
}

fn main() {
    let order = Order::Limit {
        symbol: String::from("BTC/USDT"),
        quantity: 0.5,
        price: 42000.0,
    };

    match order {
        Order::Market { symbol, quantity } => {
            println!("Market {} {} шт.", symbol, quantity);
        }
        Order::Limit { symbol, quantity, price } => {
            println!("Limit {} {} @ {}", symbol, quantity, price);
        }
        Order::StopLoss { symbol, quantity, trigger_price, limit_price } => {
            println!("Stop {} {} trigger={}", symbol, quantity, trigger_price);
            if let Some(limit) = limit_price {
                println!("  с лимитом: {}", limit);
            }
        }
    }
}
```

## Паттерн-матчинг с данными

### Извлечение данных через match

```rust
enum TradeResult {
    Profit(f64),
    Loss(f64),
    BreakEven,
}

fn main() {
    let results = vec![
        TradeResult::Profit(150.0),
        TradeResult::Loss(50.0),
        TradeResult::Profit(200.0),
        TradeResult::BreakEven,
        TradeResult::Loss(75.0),
    ];

    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    for result in &results {
        match result {
            TradeResult::Profit(amount) => {
                total_pnl += amount;
                wins += 1;
            }
            TradeResult::Loss(amount) => {
                total_pnl -= amount;
                losses += 1;
            }
            TradeResult::BreakEven => {
                // PnL не меняется
            }
        }
    }

    println!("Итого PnL: {:+.2}", total_pnl);
    println!("Победы: {}, Поражения: {}", wins, losses);
}
```

### if let для одного варианта

```rust
enum Alert {
    PriceAbove(f64),
    PriceBelow(f64),
    VolumeSpike(f64),
    Custom(String),
}

fn main() {
    let alert = Alert::PriceAbove(45000.0);

    // Если нас интересует только один вариант
    if let Alert::PriceAbove(target) = alert {
        println!("Алерт: цена выше {}", target);
    }

    // Или с else
    let alert2 = Alert::VolumeSpike(3.5);

    if let Alert::PriceAbove(target) = alert2 {
        println!("Цена выше {}", target);
    } else {
        println!("Это не ценовой алерт");
    }
}
```

### while let для итерации

```rust
enum OrderEvent {
    Filled { price: f64, quantity: f64 },
    PartialFill { price: f64, quantity: f64, remaining: f64 },
    Cancelled,
    Rejected(String),
}

fn main() {
    let mut events = vec![
        OrderEvent::PartialFill { price: 42000.0, quantity: 0.3, remaining: 0.2 },
        OrderEvent::Filled { price: 42010.0, quantity: 0.2 },
    ];

    // Обрабатываем события пока есть fill-ы
    while let Some(event) = events.pop() {
        match event {
            OrderEvent::Filled { price, quantity } => {
                println!("Полное исполнение: {} @ {}", quantity, price);
            }
            OrderEvent::PartialFill { price, quantity, remaining } => {
                println!("Частичное: {} @ {}, осталось: {}", quantity, price, remaining);
            }
            _ => break, // Другие события прекращают обработку
        }
    }
}
```

## Практический пример: Торговая стратегия

```rust
#[derive(Debug)]
enum Signal {
    Buy { price: f64, size: f64, reason: String },
    Sell { price: f64, size: f64, reason: String },
    Hold,
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit(f64),
    StopLimit { stop: f64, limit: f64 },
}

#[derive(Debug)]
struct TradeOrder {
    symbol: String,
    side: String,
    order_type: OrderType,
    quantity: f64,
}

fn generate_signal(price: f64, sma_20: f64, sma_50: f64) -> Signal {
    if sma_20 > sma_50 && price > sma_20 {
        Signal::Buy {
            price,
            size: 0.1,
            reason: String::from("Golden cross + price above SMA20"),
        }
    } else if sma_20 < sma_50 && price < sma_20 {
        Signal::Sell {
            price,
            size: 0.1,
            reason: String::from("Death cross + price below SMA20"),
        }
    } else {
        Signal::Hold
    }
}

fn signal_to_order(signal: Signal, symbol: &str) -> Option<TradeOrder> {
    match signal {
        Signal::Buy { price, size, reason } => {
            println!("Сигнал на покупку: {}", reason);
            Some(TradeOrder {
                symbol: symbol.to_string(),
                side: String::from("BUY"),
                order_type: OrderType::Limit(price * 0.999), // Чуть ниже текущей
                quantity: size,
            })
        }
        Signal::Sell { price, size, reason } => {
            println!("Сигнал на продажу: {}", reason);
            Some(TradeOrder {
                symbol: symbol.to_string(),
                side: String::from("SELL"),
                order_type: OrderType::Limit(price * 1.001), // Чуть выше текущей
                quantity: size,
            })
        }
        Signal::Hold => {
            println!("Сигнала нет, держим позицию");
            None
        }
    }
}

fn main() {
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let sma_50 = 41500.0;

    let signal = generate_signal(current_price, sma_20, sma_50);
    println!("Сигнал: {:?}", signal);

    if let Some(order) = signal_to_order(signal, "BTC/USDT") {
        println!("\nСоздан ордер: {:?}", order);

        match &order.order_type {
            OrderType::Market => println!("Тип: рыночный"),
            OrderType::Limit(price) => println!("Тип: лимитный @ {:.2}", price),
            OrderType::StopLimit { stop, limit } => {
                println!("Тип: стоп-лимит, триггер={}, лимит={}", stop, limit)
            }
        }
    }
}
```

## Практический пример: Анализ портфеля

```rust
#[derive(Debug)]
enum PositionStatus {
    Open {
        entry_price: f64,
        quantity: f64,
        unrealized_pnl: f64,
    },
    Closed {
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        realized_pnl: f64,
    },
    Pending {
        target_price: f64,
        quantity: f64,
    },
}

struct Position {
    symbol: String,
    status: PositionStatus,
}

fn analyze_portfolio(positions: &[Position]) {
    let mut total_unrealized = 0.0;
    let mut total_realized = 0.0;
    let mut open_count = 0;
    let mut closed_count = 0;

    println!("=== Анализ портфеля ===\n");

    for pos in positions {
        print!("{}: ", pos.symbol);

        match &pos.status {
            PositionStatus::Open { entry_price, quantity, unrealized_pnl } => {
                println!("OPEN {} @ {} (PnL: {:+.2})",
                    quantity, entry_price, unrealized_pnl);
                total_unrealized += unrealized_pnl;
                open_count += 1;
            }
            PositionStatus::Closed { entry_price, exit_price, quantity, realized_pnl } => {
                println!("CLOSED {} @ {} -> {} (PnL: {:+.2})",
                    quantity, entry_price, exit_price, realized_pnl);
                total_realized += realized_pnl;
                closed_count += 1;
            }
            PositionStatus::Pending { target_price, quantity } => {
                println!("PENDING {} @ {}", quantity, target_price);
            }
        }
    }

    println!("\n=== Итоги ===");
    println!("Открытых позиций: {}", open_count);
    println!("Закрытых позиций: {}", closed_count);
    println!("Нереализованный PnL: {:+.2}", total_unrealized);
    println!("Реализованный PnL: {:+.2}", total_realized);
    println!("Общий PnL: {:+.2}", total_unrealized + total_realized);
}

fn main() {
    let portfolio = vec![
        Position {
            symbol: String::from("BTC/USDT"),
            status: PositionStatus::Open {
                entry_price: 42000.0,
                quantity: 0.5,
                unrealized_pnl: 250.0,
            },
        },
        Position {
            symbol: String::from("ETH/USDT"),
            status: PositionStatus::Closed {
                entry_price: 2200.0,
                exit_price: 2350.0,
                quantity: 2.0,
                realized_pnl: 300.0,
            },
        },
        Position {
            symbol: String::from("SOL/USDT"),
            status: PositionStatus::Open {
                entry_price: 100.0,
                quantity: 10.0,
                unrealized_pnl: -50.0,
            },
        },
        Position {
            symbol: String::from("DOGE/USDT"),
            status: PositionStatus::Pending {
                target_price: 0.15,
                quantity: 1000.0,
            },
        },
    ];

    analyze_portfolio(&portfolio);
}
```

## Практический пример: Обработка событий биржи

```rust
#[derive(Debug)]
enum ExchangeEvent {
    // Рыночные данные
    Trade {
        symbol: String,
        price: f64,
        quantity: f64,
        is_buyer_maker: bool,
    },
    OrderBookUpdate {
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    },
    Ticker {
        symbol: String,
        last_price: f64,
        volume_24h: f64,
        change_24h: f64,
    },

    // События аккаунта
    OrderUpdate {
        order_id: u64,
        status: String,
        filled_qty: f64,
    },
    BalanceUpdate {
        asset: String,
        free: f64,
        locked: f64,
    },

    // Системные
    Heartbeat,
    Error(String),
}

fn process_event(event: ExchangeEvent) {
    match event {
        ExchangeEvent::Trade { symbol, price, quantity, is_buyer_maker } => {
            let side = if is_buyer_maker { "SELL" } else { "BUY" };
            println!("[TRADE] {} {} {} @ {}", symbol, side, quantity, price);
        }

        ExchangeEvent::OrderBookUpdate { symbol, bids, asks } => {
            println!("[BOOK] {} | Best Bid: {:?} | Best Ask: {:?}",
                symbol,
                bids.first(),
                asks.first()
            );
        }

        ExchangeEvent::Ticker { symbol, last_price, volume_24h, change_24h } => {
            let emoji = if change_24h >= 0.0 { "📈" } else { "📉" };
            println!("[TICK] {} {} {:.2} ({:+.2}%) Vol: {:.0}",
                emoji, symbol, last_price, change_24h, volume_24h);
        }

        ExchangeEvent::OrderUpdate { order_id, status, filled_qty } => {
            println!("[ORDER] #{} -> {} (filled: {})", order_id, status, filled_qty);
        }

        ExchangeEvent::BalanceUpdate { asset, free, locked } => {
            println!("[BAL] {} Free: {:.4}, Locked: {:.4}", asset, free, locked);
        }

        ExchangeEvent::Heartbeat => {
            // Тихо игнорируем heartbeat
        }

        ExchangeEvent::Error(msg) => {
            println!("[ERROR] {}", msg);
        }
    }
}

fn main() {
    let events = vec![
        ExchangeEvent::Ticker {
            symbol: String::from("BTC/USDT"),
            last_price: 42500.0,
            volume_24h: 15000.0,
            change_24h: 2.5,
        },
        ExchangeEvent::Trade {
            symbol: String::from("BTC/USDT"),
            price: 42510.0,
            quantity: 0.5,
            is_buyer_maker: false,
        },
        ExchangeEvent::OrderUpdate {
            order_id: 12345,
            status: String::from("FILLED"),
            filled_qty: 0.5,
        },
        ExchangeEvent::BalanceUpdate {
            asset: String::from("BTC"),
            free: 1.5,
            locked: 0.0,
        },
        ExchangeEvent::Heartbeat,
        ExchangeEvent::Error(String::from("Rate limit exceeded")),
    ];

    println!("=== Обработка событий биржи ===\n");
    for event in events {
        process_event(event);
    }
}
```

## Методы для enum с данными

```rust
#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit(f64),
    StopLimit { stop: f64, limit: f64 },
    TrailingStop { percent: f64 },
}

impl OrderType {
    fn description(&self) -> String {
        match self {
            OrderType::Market => String::from("Market: исполнение по рыночной цене"),
            OrderType::Limit(price) => format!("Limit @ {:.2}: исполнение по указанной цене", price),
            OrderType::StopLimit { stop, limit } => {
                format!("Stop-Limit: триггер {:.2}, лимит {:.2}", stop, limit)
            }
            OrderType::TrailingStop { percent } => {
                format!("Trailing Stop: отступ {:.1}%", percent)
            }
        }
    }

    fn is_conditional(&self) -> bool {
        matches!(self, OrderType::StopLimit { .. } | OrderType::TrailingStop { .. })
    }

    fn get_limit_price(&self) -> Option<f64> {
        match self {
            OrderType::Limit(price) => Some(*price),
            OrderType::StopLimit { limit, .. } => Some(*limit),
            _ => None,
        }
    }

    fn with_price(self, new_price: f64) -> Self {
        match self {
            OrderType::Limit(_) => OrderType::Limit(new_price),
            OrderType::StopLimit { stop, .. } => OrderType::StopLimit { stop, limit: new_price },
            other => other,
        }
    }
}

fn main() {
    let orders = vec![
        OrderType::Market,
        OrderType::Limit(42000.0),
        OrderType::StopLimit { stop: 41000.0, limit: 40950.0 },
        OrderType::TrailingStop { percent: 2.0 },
    ];

    for order in &orders {
        println!("{}", order.description());
        println!("  Условный: {}", order.is_conditional());
        if let Some(price) = order.get_limit_price() {
            println!("  Лимит цена: {:.2}", price);
        }
        println!();
    }

    // Изменяем цену
    let original = OrderType::Limit(42000.0);
    let updated = original.with_price(42500.0);
    println!("Обновили цену: {:?} -> {:?}", OrderType::Limit(42000.0), updated);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Variant(T)` | Tuple-вариант с данными |
| `Variant { field: T }` | Struct-вариант с именованными полями |
| `match` с извлечением | Получаем данные из варианта |
| `if let` | Проверяем один вариант |
| `matches!` | Проверка варианта без извлечения данных |

## Домашнее задание

1. **Типы ордеров**: Создай enum `AdvancedOrderType` с вариантами:
   - `Market` (без данных)
   - `Limit(price: f64)`
   - `StopLoss { trigger: f64, size: f64 }`
   - `TakeProfit { target: f64, size: f64 }`
   - `OCO { stop_loss: f64, take_profit: f64 }` (One Cancels Other)

   Реализуй методы `describe()` и `requires_trigger()`.

2. **События кошелька**: Создай enum `WalletEvent`:
   - `Deposit { asset: String, amount: f64, from: String }`
   - `Withdrawal { asset: String, amount: f64, to: String, fee: f64 }`
   - `Transfer { asset: String, amount: f64, from_wallet: String, to_wallet: String }`
   - `Swap { from_asset: String, from_amount: f64, to_asset: String, to_amount: f64 }`

   Напиши функцию, которая обрабатывает вектор событий и считает итоговое изменение баланса для каждого актива.

3. **Риск-менеджмент**: Создай enum `RiskCheckResult`:
   - `Approved`
   - `ApprovedWithWarning(String)`
   - `Rejected { reason: String, max_allowed: f64 }`
   - `RequiresManualApproval { reason: String }`

   Напиши функцию `check_order_risk(order_size: f64, account_balance: f64, max_risk_percent: f64)`, которая возвращает соответствующий результат.

4. **Стратегия с данными**: Создай enum `StrategyState`:
   - `Idle`
   - `WaitingForEntry { target_price: f64 }`
   - `InPosition { entry_price: f64, quantity: f64, stop_loss: f64, take_profit: f64 }`
   - `Exiting { reason: String }`

   Реализуй функции перехода между состояниями и вывод текущего состояния.

## Навигация

[← Предыдущий день](../070-enum-order-side/ru.md) | [Следующий день →](../072-option-price-missing/ru.md)
