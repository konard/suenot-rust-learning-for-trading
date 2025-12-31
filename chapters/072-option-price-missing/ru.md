# День 72: Option — цена может отсутствовать

## Аналогия из трейдинга

Представь: ты запрашиваешь цену актива у биржи. Что получишь в ответ? Иногда — число (цену), а иногда... ничего. Торги могли быть приостановлены, актив делистирован, или данных просто нет за этот период.

В реальном трейдинге это происходит постоянно:
- **Цена закрытия** — может не быть, если в этот день не было торгов
- **Последняя сделка** — может отсутствовать для неликвидного актива
- **Stop-loss** — может быть не установлен
- **Позиция в портфеле** — актива может не быть в портфеле

Rust решает эту проблему элегантно с помощью типа `Option<T>`.

## Что такое Option?

`Option` — это перечисление (enum), которое может содержать значение (`Some`) или быть пустым (`None`):

```rust
enum Option<T> {
    Some(T),  // Есть значение типа T
    None,     // Значения нет
}
```

Это заставляет программиста **явно** обрабатывать случай отсутствия значения — никаких `null` и неожиданных падений программы!

## Базовое использование

```rust
fn main() {
    // Цена может быть, а может не быть
    let btc_price: Option<f64> = Some(42000.0);
    let delisted_price: Option<f64> = None;

    // Проверка наличия значения
    if btc_price.is_some() {
        println!("BTC торгуется");
    }

    if delisted_price.is_none() {
        println!("Актив не торгуется");
    }
}
```

## Извлечение значения

### match — самый надёжный способ

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    match price {
        Some(p) => println!("Цена: ${:.2}", p),
        None => println!("Цена недоступна"),
    }
}
```

### if let — когда важен только один вариант

```rust
fn main() {
    let stop_loss: Option<f64> = Some(41000.0);

    if let Some(sl) = stop_loss {
        println!("Stop-loss установлен на ${:.2}", sl);
    }

    let take_profit: Option<f64> = None;

    if let Some(tp) = take_profit {
        println!("Take-profit: ${:.2}", tp);
    } else {
        println!("Take-profit не установлен");
    }
}
```

### unwrap_or — значение по умолчанию

```rust
fn main() {
    let bid: Option<f64> = Some(42000.0);
    let ask: Option<f64> = None;

    // Если None — используем значение по умолчанию
    let bid_price = bid.unwrap_or(0.0);
    let ask_price = ask.unwrap_or(0.0);

    println!("Bid: {}, Ask: {}", bid_price, ask_price);
}
```

### unwrap_or_else — ленивое вычисление по умолчанию

```rust
fn main() {
    let cached_price: Option<f64> = None;

    // Функция вызывается только если None
    let price = cached_price.unwrap_or_else(|| {
        println!("Загружаем цену с биржи...");
        fetch_price_from_exchange()
    });

    println!("Цена: ${:.2}", price);
}

fn fetch_price_from_exchange() -> f64 {
    42500.0  // Имитация запроса к бирже
}
```

## Option в функциях — анализ цен

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Поиск максимальной цены
    match find_max_price(&prices) {
        Some(max) => println!("Максимум: ${:.2}", max),
        None => println!("Нет данных для анализа"),
    }

    // Пустой массив
    let empty: Vec<f64> = vec![];
    match find_max_price(&empty) {
        Some(max) => println!("Максимум: ${:.2}", max),
        None => println!("Массив пуст — нет максимума"),
    }
}

fn find_max_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let mut max = prices[0];
    for &price in &prices[1..] {
        if price > max {
            max = price;
        }
    }
    Some(max)
}
```

## Option для управления ордерами

```rust
fn main() {
    // Создаём ордер без stop-loss
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 0.5,
        stop_loss: None,
        take_profit: None,
    };

    print_order(&order);

    // Устанавливаем stop-loss
    order.stop_loss = Some(41000.0);
    order.take_profit = Some(45000.0);

    print_order(&order);

    // Проверяем риск
    if let Some(risk) = calculate_risk(&order) {
        println!("Риск на сделку: ${:.2}", risk);
    }
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

fn print_order(order: &Order) {
    println!("\n=== Ордер ===");
    println!("Символ: {}", order.symbol);
    println!("Сторона: {:?}", order.side);
    println!("Цена: ${:.2}", order.price);
    println!("Количество: {}", order.quantity);

    match order.stop_loss {
        Some(sl) => println!("Stop-Loss: ${:.2}", sl),
        None => println!("Stop-Loss: не установлен"),
    }

    match order.take_profit {
        Some(tp) => println!("Take-Profit: ${:.2}", tp),
        None => println!("Take-Profit: не установлен"),
    }
}

fn calculate_risk(order: &Order) -> Option<f64> {
    // Риск можно рассчитать только если есть stop-loss
    let stop_loss = order.stop_loss?;  // Ранний выход если None

    let risk_per_unit = (order.price - stop_loss).abs();
    Some(risk_per_unit * order.quantity)
}
```

## Оператор ? для Option

Оператор `?` позволяет элегантно обрабатывать цепочки Option:

```rust
fn main() {
    let portfolio = Portfolio {
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: Some(0.5), avg_price: Some(42000.0) },
            Position { symbol: String::from("ETH"), quantity: Some(10.0), avg_price: Some(2200.0) },
            Position { symbol: String::from("DOGE"), quantity: None, avg_price: None },
        ],
    };

    for pos in &portfolio.positions {
        match calculate_position_value(pos) {
            Some(value) => println!("{}: ${:.2}", pos.symbol, value),
            None => println!("{}: нет позиции", pos.symbol),
        }
    }
}

struct Position {
    symbol: String,
    quantity: Option<f64>,
    avg_price: Option<f64>,
}

struct Portfolio {
    positions: Vec<Position>,
}

fn calculate_position_value(position: &Position) -> Option<f64> {
    let qty = position.quantity?;      // Вернёт None если quantity = None
    let price = position.avg_price?;   // Вернёт None если avg_price = None
    Some(qty * price)
}
```

## Методы трансформации Option

### map — преобразование значения

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Преобразуем цену в строку
    let formatted: Option<String> = price.map(|p| format!("${:.2}", p));
    println!("{:?}", formatted);  // Some("$42000.00")

    // Для None map возвращает None
    let no_price: Option<f64> = None;
    let formatted_none: Option<String> = no_price.map(|p| format!("${:.2}", p));
    println!("{:?}", formatted_none);  // None
}
```

### and_then — цепочка Option

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    // Цепочка: найти максимум -> рассчитать комиссию
    let max_fee = find_max_price(&prices)
        .and_then(|max| calculate_fee(max, 0.1));

    match max_fee {
        Some(fee) => println!("Комиссия от максимума: ${:.2}", fee),
        None => println!("Невозможно рассчитать"),
    }
}

fn find_max_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }
    prices.iter().cloned().reduce(f64::max)
}

fn calculate_fee(amount: f64, fee_percent: f64) -> Option<f64> {
    if amount <= 0.0 {
        return None;
    }
    Some(amount * fee_percent / 100.0)
}
```

### filter — условная фильтрация

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Оставляем только если цена выше порога
    let valid_price = price.filter(|&p| p > 40000.0);
    println!("Валидная цена: {:?}", valid_price);  // Some(42000.0)

    let low_price: Option<f64> = Some(35000.0);
    let filtered = low_price.filter(|&p| p > 40000.0);
    println!("Отфильтровано: {:?}", filtered);  // None
}
```

## Практический пример: система риск-менеджмента

```rust
fn main() {
    let trade_params = TradeParams {
        entry_price: Some(42000.0),
        stop_loss: Some(41000.0),
        take_profit: Some(45000.0),
        position_size: Some(0.5),
        account_balance: Some(10000.0),
    };

    match analyze_trade_risk(&trade_params) {
        Some(analysis) => {
            println!("╔════════════════════════════════════╗");
            println!("║       АНАЛИЗ РИСКА СДЕЛКИ          ║");
            println!("╠════════════════════════════════════╣");
            println!("║ Риск на сделку:    ${:>13.2} ║", analysis.risk_amount);
            println!("║ Риск от баланса:   {:>13.2}% ║", analysis.risk_percent);
            println!("║ Потенциал прибыли: ${:>13.2} ║", analysis.potential_profit);
            println!("║ Risk/Reward:       {:>13.2}  ║", analysis.risk_reward_ratio);
            println!("║ Рекомендация:      {:>13}  ║", analysis.recommendation);
            println!("╚════════════════════════════════════╝");
        }
        None => println!("Недостаточно данных для анализа риска"),
    }
}

struct TradeParams {
    entry_price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    position_size: Option<f64>,
    account_balance: Option<f64>,
}

struct RiskAnalysis {
    risk_amount: f64,
    risk_percent: f64,
    potential_profit: f64,
    risk_reward_ratio: f64,
    recommendation: String,
}

fn analyze_trade_risk(params: &TradeParams) -> Option<RiskAnalysis> {
    // Извлекаем все необходимые параметры
    let entry = params.entry_price?;
    let stop = params.stop_loss?;
    let target = params.take_profit?;
    let size = params.position_size?;
    let balance = params.account_balance?;

    // Расчёты
    let risk_per_unit = (entry - stop).abs();
    let profit_per_unit = (target - entry).abs();

    let risk_amount = risk_per_unit * size;
    let risk_percent = (risk_amount / balance) * 100.0;
    let potential_profit = profit_per_unit * size;

    let risk_reward_ratio = if risk_amount > 0.0 {
        potential_profit / risk_amount
    } else {
        0.0
    };

    let recommendation = if risk_percent > 5.0 {
        String::from("ВЫСОКИЙ РИСК")
    } else if risk_reward_ratio < 1.5 {
        String::from("НИЗКИЙ R/R")
    } else {
        String::from("ОДОБРЕНО")
    };

    Some(RiskAnalysis {
        risk_amount,
        risk_percent,
        potential_profit,
        risk_reward_ratio,
        recommendation,
    })
}
```

## Работа с коллекциями и Option

```rust
fn main() {
    let trades: Vec<Option<f64>> = vec![
        Some(150.0),   // Прибыльная сделка
        Some(-50.0),   // Убыточная
        None,          // Данных нет
        Some(200.0),   // Прибыльная
        None,          // Данных нет
        Some(-30.0),   // Убыточная
    ];

    // Фильтруем None и суммируем
    let total_pnl: f64 = trades
        .iter()
        .filter_map(|&t| t)  // Убирает None, извлекает значения
        .sum();

    println!("Общий PnL: ${:.2}", total_pnl);

    // Считаем только прибыльные сделки
    let profitable_count = trades
        .iter()
        .filter_map(|&t| t)
        .filter(|&pnl| pnl > 0.0)
        .count();

    println!("Прибыльных сделок: {}", profitable_count);

    // Средний PnL (только по известным сделкам)
    let known_trades: Vec<f64> = trades
        .iter()
        .filter_map(|&t| t)
        .collect();

    if !known_trades.is_empty() {
        let avg_pnl = known_trades.iter().sum::<f64>() / known_trades.len() as f64;
        println!("Средний PnL: ${:.2}", avg_pnl);
    }
}
```

## Что мы узнали

| Метод/Конструкция | Описание | Пример |
|-------------------|----------|--------|
| `Some(value)` | Создаёт Option со значением | `Some(42000.0)` |
| `None` | Пустой Option | `let price: Option<f64> = None` |
| `is_some()` | Проверка наличия значения | `price.is_some()` |
| `is_none()` | Проверка отсутствия | `price.is_none()` |
| `unwrap_or(default)` | Значение или дефолт | `price.unwrap_or(0.0)` |
| `unwrap_or_else(f)` | Значение или результат функции | `price.unwrap_or_else(\|\| calc())` |
| `map(f)` | Преобразование значения | `price.map(\|p\| p * 2.0)` |
| `and_then(f)` | Цепочка Option | `price.and_then(calc_fee)` |
| `filter(pred)` | Условная фильтрация | `price.filter(\|&p\| p > 0.0)` |
| `?` оператор | Ранний выход при None | `let p = price?;` |

## Упражнения

1. **Получение цены**: Напиши функцию `get_price(symbol: &str, prices: &HashMap<String, f64>) -> Option<f64>`, которая возвращает цену актива если он есть в словаре.

2. **Расчёт спреда**: Создай функцию `calculate_spread(bid: Option<f64>, ask: Option<f64>) -> Option<f64>`, которая возвращает спред только если оба значения доступны.

3. **Поиск в портфеле**: Напиши функцию `find_position(portfolio: &[Position], symbol: &str) -> Option<&Position>`, которая находит позицию по символу.

4. **Цепочка проверок**: Создай функцию `validate_order(order: &Order) -> Option<ValidatedOrder>`, которая проверяет все поля и возвращает валидный ордер только если всё корректно.

## Домашнее задание

1. Реализуй систему кэширования цен, где `get_cached_price` возвращает `Option<f64>` — цену из кэша или None если кэш устарел.

2. Создай функцию `find_best_entry(candles: &[Candle]) -> Option<EntrySignal>`, которая анализирует свечи и возвращает сигнал на вход только при определённых условиях.

3. Напиши функцию `calculate_portfolio_stats(positions: &[Position]) -> Option<PortfolioStats>`, которая возвращает статистику портфеля или None если портфель пуст.

4. Реализуй функцию `get_trading_recommendation(price: f64, indicators: &Indicators) -> Option<Recommendation>`, где `Indicators` содержит `Option<f64>` для различных индикаторов (SMA, RSI, MACD). Рекомендация выдаётся только при наличии всех необходимых индикаторов.

## Навигация

[← Предыдущий день](../071-result-order-execution/ru.md) | [Следующий день →](../073-result-error-handling/ru.md)
