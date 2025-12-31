# День 54: Паттерн — заимствуем, не владеем

## Аналогия из трейдинга

Представь, что ты аналитик в инвестиционной компании. Когда тебе нужно проанализировать портфель клиента, ты не забираешь его активы себе — ты просто **смотришь** на них и делаешь выводы. После анализа портфель остаётся у клиента в целости и сохранности.

Это и есть паттерн **"Заимствуй, не владей"**:
- Аналитик **заимствует** данные для анализа
- Клиент **владеет** своим портфелем
- После работы данные возвращаются владельцу

В Rust этот паттерн — основа эффективного и безопасного кода.

## Теория: зачем заимствовать?

### Проблема с владением

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // Передаём владение — portfolio больше недоступен!
    let total = count_assets(portfolio);

    // Ошибка! portfolio перемещён
    // println!("{:?}", portfolio);
}

fn count_assets(assets: Vec<&str>) -> usize {
    assets.len()
}
```

### Решение: заимствование

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // Заимствуем — portfolio остаётся у нас
    let total = count_assets(&portfolio);

    // Работает! portfolio всё ещё наш
    println!("Portfolio: {:?}, count: {}", portfolio, total);
}

fn count_assets(assets: &Vec<&str>) -> usize {
    assets.len()
}
```

## Паттерн в действии: анализ рынка

### Пример 1: Анализ цен без владения

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 43200.0, 42900.0];

    // Все функции заимствуют данные
    let avg = calculate_average(&prices);
    let max = find_max(&prices);
    let min = find_min(&prices);
    let volatility = calculate_volatility(&prices);

    // prices всё ещё доступен для дальнейшего использования
    println!("Price analysis for {} data points:", prices.len());
    println!("  Average: ${:.2}", avg);
    println!("  Max: ${:.2}", max);
    println!("  Min: ${:.2}", min);
    println!("  Volatility: {:.2}%", volatility);
}

fn calculate_average(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}

fn find_max(prices: &[f64]) -> f64 {
    prices.iter().copied().fold(f64::MIN, f64::max)
}

fn find_min(prices: &[f64]) -> f64 {
    prices.iter().copied().fold(f64::MAX, f64::min)
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let avg = calculate_average(prices);
    let variance: f64 = prices.iter()
        .map(|p| (p - avg).powi(2))
        .sum::<f64>() / prices.len() as f64;

    (variance.sqrt() / avg) * 100.0
}
```

### Пример 2: Анализ ордера

```rust
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        price: 42000.0,
        quantity: 0.5,
    };

    // Каждая функция заимствует ордер
    let value = calculate_order_value(&order);
    let fee = estimate_fee(&order, 0.1);
    let is_valid = validate_order(&order);

    // order всё ещё доступен
    println!("Order: {} {} {} @ ${}",
        order.side, order.quantity, order.symbol, order.price);
    println!("Value: ${:.2}", value);
    println!("Fee: ${:.2}", fee);
    println!("Valid: {}", is_valid);
}

fn calculate_order_value(order: &Order) -> f64 {
    order.price * order.quantity
}

fn estimate_fee(order: &Order, fee_percent: f64) -> f64 {
    calculate_order_value(order) * (fee_percent / 100.0)
}

fn validate_order(order: &Order) -> bool {
    order.price > 0.0 && order.quantity > 0.0 && !order.symbol.is_empty()
}
```

### Пример 3: Цепочка анализов

```rust
struct Portfolio {
    name: String,
    assets: Vec<Asset>,
    total_value: f64,
}

struct Asset {
    symbol: String,
    quantity: f64,
    price: f64,
}

fn main() {
    let portfolio = Portfolio {
        name: String::from("Main Portfolio"),
        assets: vec![
            Asset { symbol: String::from("BTC"), quantity: 1.5, price: 42000.0 },
            Asset { symbol: String::from("ETH"), quantity: 10.0, price: 2500.0 },
            Asset { symbol: String::from("SOL"), quantity: 100.0, price: 95.0 },
        ],
        total_value: 97500.0,
    };

    // Цепочка анализов — каждый заимствует
    let report = generate_report(&portfolio);
    let allocation = calculate_allocation(&portfolio);
    let risk = assess_risk(&portfolio);

    // portfolio остаётся доступен
    println!("=== {} ===", portfolio.name);
    println!("{}", report);
    println!("\nAllocation:");
    for (symbol, percent) in allocation {
        println!("  {}: {:.1}%", symbol, percent);
    }
    println!("\nRisk level: {}", risk);
}

fn generate_report(portfolio: &Portfolio) -> String {
    format!(
        "Assets: {}, Total Value: ${:.2}",
        portfolio.assets.len(),
        portfolio.total_value
    )
}

fn calculate_allocation(portfolio: &Portfolio) -> Vec<(String, f64)> {
    portfolio.assets.iter()
        .map(|asset| {
            let value = asset.quantity * asset.price;
            let percent = (value / portfolio.total_value) * 100.0;
            (asset.symbol.clone(), percent)
        })
        .collect()
}

fn assess_risk(portfolio: &Portfolio) -> &'static str {
    let btc_allocation = portfolio.assets.iter()
        .find(|a| a.symbol == "BTC")
        .map(|a| (a.quantity * a.price) / portfolio.total_value)
        .unwrap_or(0.0);

    if btc_allocation > 0.5 {
        "High (>50% in BTC)"
    } else if btc_allocation > 0.3 {
        "Medium (30-50% in BTC)"
    } else {
        "Low (<30% in BTC)"
    }
}
```

## Когда использовать заимствование

### Используй `&T` (неизменяемую ссылку) когда:

```rust
// 1. Нужно только прочитать данные
fn print_order(order: &Order) {
    println!("{}: {} @ {}", order.symbol, order.quantity, order.price);
}

// 2. Нужно передать в несколько функций
fn analyze_order(order: &Order) {
    validate_order(order);
    calculate_order_value(order);
    estimate_fee(order, 0.1);
}

// 3. Работаешь с большими структурами
fn process_large_history(history: &[Trade]) -> Summary {
    // Не копируем тысячи сделок — просто смотрим
    Summary {
        count: history.len(),
        total_volume: history.iter().map(|t| t.volume).sum(),
    }
}
```

### Используй `&mut T` (изменяемую ссылку) когда:

```rust
// 1. Нужно изменить данные, но не забрать владение
fn update_price(order: &mut Order, new_price: f64) {
    order.price = new_price;
}

// 2. Нужно добавить элементы в коллекцию
fn add_asset(portfolio: &mut Portfolio, asset: Asset) {
    portfolio.total_value += asset.quantity * asset.price;
    portfolio.assets.push(asset);
}

// 3. Нужно модифицировать состояние
fn execute_trade(position: &mut Position, trade: &Trade) {
    if trade.side == "buy" {
        position.quantity += trade.quantity;
    } else {
        position.quantity -= trade.quantity;
    }
}
```

## Практический пример: торговый анализатор

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct TradeStats {
    total_trades: usize,
    buy_trades: usize,
    sell_trades: usize,
    total_volume: f64,
    average_price: f64,
    pnl: f64,
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 42000.0, quantity: 0.5, timestamp: 1000 },
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 41500.0, quantity: 0.3, timestamp: 2000 },
        Trade { symbol: "BTC".into(), side: "sell".into(), price: 43000.0, quantity: 0.4, timestamp: 3000 },
        Trade { symbol: "BTC".into(), side: "sell".into(), price: 42500.0, quantity: 0.2, timestamp: 4000 },
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 42200.0, quantity: 0.1, timestamp: 5000 },
    ];

    println!("=== Trade Analysis ===\n");

    // Все функции заимствуют trades
    let stats = calculate_stats(&trades);
    let best_trade = find_best_trade(&trades);
    let worst_trade = find_worst_trade(&trades);

    // trades всё ещё доступен
    println!("Statistics:");
    println!("  Total trades: {}", stats.total_trades);
    println!("  Buy trades: {}", stats.buy_trades);
    println!("  Sell trades: {}", stats.sell_trades);
    println!("  Total volume: {:.4} BTC", stats.total_volume);
    println!("  Average price: ${:.2}", stats.average_price);
    println!("  Estimated PnL: ${:.2}", stats.pnl);

    if let Some(trade) = best_trade {
        println!("\nBest trade (highest sell):");
        println!("  {} {} @ ${}", trade.side, trade.quantity, trade.price);
    }

    if let Some(trade) = worst_trade {
        println!("\nWorst trade (highest buy):");
        println!("  {} {} @ ${}", trade.side, trade.quantity, trade.price);
    }

    // Можем продолжить использовать trades
    println!("\nAll {} trades are still accessible!", trades.len());
}

fn calculate_stats(trades: &[Trade]) -> TradeStats {
    let total_trades = trades.len();
    let buy_trades = trades.iter().filter(|t| t.side == "buy").count();
    let sell_trades = trades.iter().filter(|t| t.side == "sell").count();

    let total_volume: f64 = trades.iter().map(|t| t.quantity).sum();

    let total_value: f64 = trades.iter().map(|t| t.price * t.quantity).sum();
    let average_price = if total_volume > 0.0 { total_value / total_volume } else { 0.0 };

    // Простой расчёт PnL
    let buy_cost: f64 = trades.iter()
        .filter(|t| t.side == "buy")
        .map(|t| t.price * t.quantity)
        .sum();

    let sell_revenue: f64 = trades.iter()
        .filter(|t| t.side == "sell")
        .map(|t| t.price * t.quantity)
        .sum();

    TradeStats {
        total_trades,
        buy_trades,
        sell_trades,
        total_volume,
        average_price,
        pnl: sell_revenue - buy_cost,
    }
}

fn find_best_trade<'a>(trades: &'a [Trade]) -> Option<&'a Trade> {
    trades.iter()
        .filter(|t| t.side == "sell")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}

fn find_worst_trade<'a>(trades: &'a [Trade]) -> Option<&'a Trade> {
    trades.iter()
        .filter(|t| t.side == "buy")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}
```

## Сравнение подходов

```rust
// ПЛОХО: забираем владение без необходимости
fn bad_calculate_total(prices: Vec<f64>) -> f64 {
    prices.iter().sum()
    // prices уничтожен, вызывающий код не может его использовать
}

// ХОРОШО: заимствуем данные
fn good_calculate_total(prices: &[f64]) -> f64 {
    prices.iter().sum()
    // prices остаётся у вызывающего кода
}

// ПЛОХО: клонируем без необходимости
fn bad_find_symbol(orders: Vec<Order>, symbol: &str) -> Option<Order> {
    orders.into_iter().find(|o| o.symbol == symbol)
}

// ХОРОШО: возвращаем ссылку
fn good_find_symbol<'a>(orders: &'a [Order], symbol: &str) -> Option<&'a Order> {
    orders.iter().find(|o| o.symbol == symbol)
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&T` | Неизменяемое заимствование — только чтение |
| `&mut T` | Изменяемое заимствование — чтение и запись |
| `&[T]` | Слайс — заимствование части коллекции |
| Паттерн | Передавай `&T` если не нужно владение |
| Преимущество | Данные остаются у владельца |

## Домашнее задание

1. **Анализатор портфеля**:
   Создай функции, которые принимают `&Portfolio` и возвращают:
   - Общую стоимость
   - Количество активов
   - Самый дорогой актив
   - Процент в криптовалюте

2. **Валидатор ордеров**:
   Напиши функцию `validate_orders(&[Order]) -> Vec<&Order>`, которая возвращает ссылки только на валидные ордера.

3. **Сравнение цен**:
   Создай функции для сравнения двух временных рядов цен:
   - `compare_averages(&[f64], &[f64]) -> f64`
   - `find_correlation(&[f64], &[f64]) -> f64`
   - `find_divergence(&[f64], &[f64]) -> Vec<(usize, f64)>`

4. **Менеджер рисков**:
   Напиши структуру `RiskManager` с методами, принимающими `&Portfolio`:
   - `check_exposure(&self, portfolio: &Portfolio) -> RiskLevel`
   - `suggest_rebalance(&self, portfolio: &Portfolio) -> Vec<Suggestion>`
   - `generate_report(&self, portfolio: &Portfolio) -> String`

## Навигация

[← Предыдущий день](../053-data-in-out/ru.md) | [Следующий день →](../055-debugging-ownership/ru.md)
