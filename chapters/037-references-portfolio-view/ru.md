# День 37: Ссылки — смотрим на чужой портфель

## Аналогия из трейдинга

Представь, что коллега показывает тебе свой торговый портфель на экране. Ты **видишь** все его позиции, можешь анализировать, но **не можешь** ничего изменить — это **его** портфель. Ты просто смотришь на него через ссылку.

В Rust **ссылка** (`&T`) работает так же: она позволяет "посмотреть" на данные без владения ими. Оригинал остаётся у владельца.

## Что такое ссылка?

Ссылка — это адрес данных в памяти. Вместо того чтобы копировать или передавать владение, мы передаём "указатель" на данные.

```rust
fn main() {
    let portfolio_value = 100_000.0;

    // Создаём ссылку на значение
    let reference = &portfolio_value;

    println!("Значение портфеля: ${}", portfolio_value);
    println!("Через ссылку: ${}", reference);

    // Оба указывают на одни и те же данные!
}
```

## Зачем нужны ссылки?

### Без ссылок — передаём владение

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    let avg = calculate_average(prices);  // prices перемещён!
    println!("Среднее: {}", avg);

    // println!("{:?}", prices);  // ОШИБКА! prices больше не доступен
}

fn calculate_average(data: Vec<f64>) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

### Со ссылками — заимствуем для чтения

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    let avg = calculate_average(&prices);  // Передаём ссылку!
    println!("Среднее: {}", avg);

    println!("Цены: {:?}", prices);  // prices всё ещё доступен!
}

fn calculate_average(data: &Vec<f64>) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

## Синтаксис ссылок

```rust
fn main() {
    let btc_price = 42000.0;

    // Создание ссылки: &
    let price_ref = &btc_price;

    // Разыменование ссылки: *
    let value = *price_ref;

    println!("Оригинал: {}", btc_price);
    println!("Через ссылку: {}", price_ref);    // Автоматическое разыменование
    println!("Разыменовано: {}", value);
}
```

## Ссылки на структуры

```rust
struct Portfolio {
    name: String,
    total_value: f64,
    positions_count: usize,
}

fn main() {
    let my_portfolio = Portfolio {
        name: String::from("Main Trading Account"),
        total_value: 150_000.0,
        positions_count: 12,
    };

    // Передаём ссылку на портфель
    display_portfolio(&my_portfolio);

    // Портфель всё ещё наш!
    println!("\nМой портфель: {} - ${:.2}",
             my_portfolio.name,
             my_portfolio.total_value);
}

fn display_portfolio(portfolio: &Portfolio) {
    println!("╔═══════════════════════════════════╗");
    println!("║         PORTFOLIO VIEW            ║");
    println!("╠═══════════════════════════════════╣");
    println!("║ Name: {:>25} ║", portfolio.name);
    println!("║ Value: ${:>23.2} ║", portfolio.total_value);
    println!("║ Positions: {:>20} ║", portfolio.positions_count);
    println!("╚═══════════════════════════════════╝");
}
```

## Множественные ссылки для чтения

Можно создать сколько угодно ссылок для чтения одновременно:

```rust
fn main() {
    let market_data = vec![42000.0, 42100.0, 41900.0, 42300.0, 42200.0];

    // Несколько ссылок на одни данные — это нормально!
    let ref1 = &market_data;
    let ref2 = &market_data;
    let ref3 = &market_data;

    // Все могут читать одновременно
    println!("Аналитик 1 видит: {:?}", ref1);
    println!("Аналитик 2 видит: {:?}", ref2);
    println!("Аналитик 3 видит: {:?}", ref3);

    // Это как несколько трейдеров смотрят на один график
}
```

## Ссылки в функциях анализа

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

fn main() {
    let trades = vec![
        Trade { symbol: String::from("BTC"), entry_price: 42000.0, exit_price: 43500.0, quantity: 0.5 },
        Trade { symbol: String::from("ETH"), entry_price: 2200.0, exit_price: 2350.0, quantity: 5.0 },
        Trade { symbol: String::from("BTC"), entry_price: 43000.0, exit_price: 42500.0, quantity: 0.3 },
    ];

    // Передаём ссылки — не забираем владение
    let total_pnl = calculate_total_pnl(&trades);
    let profitable = count_profitable_trades(&trades);
    let win_rate = calculate_win_rate(&trades);

    println!("Total PnL: ${:.2}", total_pnl);
    println!("Profitable trades: {}", profitable);
    println!("Win rate: {:.1}%", win_rate * 100.0);

    // trades всё ещё доступны для дальнейшего анализа!
    for trade in &trades {
        let pnl = (trade.exit_price - trade.entry_price) * trade.quantity;
        println!("{}: ${:.2}", trade.symbol, pnl);
    }
}

fn calculate_total_pnl(trades: &Vec<Trade>) -> f64 {
    trades.iter()
        .map(|t| (t.exit_price - t.entry_price) * t.quantity)
        .sum()
}

fn count_profitable_trades(trades: &Vec<Trade>) -> usize {
    trades.iter()
        .filter(|t| t.exit_price > t.entry_price)
        .count()
}

fn calculate_win_rate(trades: &Vec<Trade>) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }
    let profitable = count_profitable_trades(trades);
    profitable as f64 / trades.len() as f64
}
```

## Ссылки на срезы (slices)

Вместо `&Vec<T>` часто лучше использовать `&[T]` — срез:

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Полный срез
    let sma5 = calculate_sma(&prices);
    println!("SMA-5: {:.2}", sma5);

    // Последние 3 цены
    let sma3 = calculate_sma(&prices[2..]);
    println!("SMA-3 (последние): {:.2}", sma3);

    // Первые 3 цены
    let sma3_first = calculate_sma(&prices[..3]);
    println!("SMA-3 (первые): {:.2}", sma3_first);
}

// Принимает срез — работает и с Vec, и с массивами, и с частями
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Ссылки на строки: &str vs &String

```rust
fn main() {
    let ticker_owned = String::from("BTCUSDT");
    let ticker_literal = "ETHUSDT";  // Это уже &str

    // Обе работают с функцией, принимающей &str
    display_ticker(&ticker_owned);   // &String автоматически преобразуется в &str
    display_ticker(ticker_literal);  // &str остаётся &str
}

fn display_ticker(ticker: &str) {
    println!("Trading: {}", ticker);
}
```

## Практический пример: анализ портфеля

```rust
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

struct Portfolio {
    name: String,
    positions: Vec<Position>,
}

fn main() {
    let portfolio = Portfolio {
        name: String::from("Crypto Portfolio"),
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: 1.5, entry_price: 40000.0, current_price: 42000.0 },
            Position { symbol: String::from("ETH"), quantity: 10.0, entry_price: 2000.0, current_price: 2200.0 },
            Position { symbol: String::from("SOL"), quantity: 50.0, entry_price: 100.0, current_price: 95.0 },
        ],
    };

    // Все функции принимают ссылки — портфель не перемещается
    println!("Portfolio: {}", portfolio.name);
    println!("Total Value: ${:.2}", calculate_portfolio_value(&portfolio));
    println!("Total PnL: ${:.2}", calculate_portfolio_pnl(&portfolio));
    println!("Unrealized PnL%: {:.2}%", calculate_portfolio_pnl_percent(&portfolio));

    print_portfolio_summary(&portfolio);
}

fn calculate_portfolio_value(portfolio: &Portfolio) -> f64 {
    portfolio.positions.iter()
        .map(|p| p.quantity * p.current_price)
        .sum()
}

fn calculate_portfolio_pnl(portfolio: &Portfolio) -> f64 {
    portfolio.positions.iter()
        .map(|p| (p.current_price - p.entry_price) * p.quantity)
        .sum()
}

fn calculate_portfolio_pnl_percent(portfolio: &Portfolio) -> f64 {
    let cost: f64 = portfolio.positions.iter()
        .map(|p| p.entry_price * p.quantity)
        .sum();

    if cost == 0.0 {
        return 0.0;
    }

    let pnl = calculate_portfolio_pnl(portfolio);
    (pnl / cost) * 100.0
}

fn print_portfolio_summary(portfolio: &Portfolio) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║                PORTFOLIO SUMMARY                 ║");
    println!("╠══════════════════════════════════════════════════╣");

    for position in &portfolio.positions {
        let pnl = (position.current_price - position.entry_price) * position.quantity;
        let pnl_symbol = if pnl >= 0.0 { "+" } else { "" };
        println!("║ {:6} | {:>6.2} units | PnL: {}{:>10.2} ║",
                 position.symbol, position.quantity, pnl_symbol, pnl);
    }

    println!("╚══════════════════════════════════════════════════╝");
}
```

## Правила ссылок

1. **Ссылка не может жить дольше данных**
```rust
fn main() {
    let reference;
    {
        let price = 42000.0;
        reference = &price;
    }  // price уничтожен
    // println!("{}", reference);  // ОШИБКА! Данных больше нет
}
```

2. **Нельзя изменять данные через обычную ссылку**
```rust
fn main() {
    let price = 42000.0;
    let reference = &price;

    // *reference = 43000.0;  // ОШИБКА! Ссылка только для чтения
}
```

3. **Много читателей — это нормально**
```rust
fn main() {
    let data = vec![1, 2, 3];
    let r1 = &data;
    let r2 = &data;
    let r3 = &data;
    println!("{:?} {:?} {:?}", r1, r2, r3);  // OK
}
```

## Упражнения

### Упражнение 1: Анализ массива цен
```rust
// Реализуй функции, используя ссылки

fn find_min_price(prices: &[f64]) -> f64 {
    // Найди минимальную цену
    todo!()
}

fn find_max_price(prices: &[f64]) -> f64 {
    // Найди максимальную цену
    todo!()
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    // Рассчитай волатильность: (max - min) / min * 100
    todo!()
}

fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    println!("Min: ${:.2}", find_min_price(&prices));
    println!("Max: ${:.2}", find_max_price(&prices));
    println!("Volatility: {:.2}%", calculate_volatility(&prices));
}
```

### Упражнение 2: Фильтрация сделок
```rust
struct Trade {
    symbol: String,
    pnl: f64,
}

fn filter_by_symbol<'a>(trades: &'a [Trade], symbol: &str) -> Vec<&'a Trade> {
    // Верни ссылки на сделки с указанным символом
    todo!()
}

fn calculate_symbol_pnl(trades: &[Trade], symbol: &str) -> f64 {
    // Рассчитай PnL для конкретного символа
    todo!()
}
```

### Упражнение 3: Поиск лучшей сделки
```rust
struct Trade {
    id: u32,
    pnl: f64,
}

fn find_best_trade(trades: &[Trade]) -> Option<&Trade> {
    // Верни ссылку на сделку с максимальным PnL
    todo!()
}

fn find_worst_trade(trades: &[Trade]) -> Option<&Trade> {
    // Верни ссылку на сделку с минимальным PnL
    todo!()
}
```

## Что мы узнали

| Концепция | Синтаксис | Описание |
|-----------|-----------|----------|
| Создание ссылки | `&value` | Получаем адрес значения |
| Ссылка в параметре | `fn foo(x: &T)` | Функция заимствует данные |
| Срез | `&[T]` | Ссылка на часть коллекции |
| Разыменование | `*reference` | Получаем значение по ссылке |
| String slice | `&str` | Ссылка на строковые данные |

## Домашнее задание

1. Напиши функцию `analyze_order_book(bids: &[f64], asks: &[f64]) -> (f64, f64, f64)`, которая возвращает лучший bid, лучший ask и спред

2. Создай структуру `MarketData` с историей цен и напиши функции для технического анализа, использующие ссылки:
   - `calculate_ema(data: &MarketData, period: usize) -> f64`
   - `find_support_resistance(data: &MarketData) -> (f64, f64)`

3. Реализуй функцию `compare_portfolios(p1: &Portfolio, p2: &Portfolio) -> PortfolioComparison`, которая сравнивает два портфеля без владения ими

4. Напиши функцию `get_top_performers(positions: &[Position], n: usize) -> Vec<&Position>`, которая возвращает ссылки на N лучших позиций по PnL

## Навигация

[← Предыдущий день](../036-copy-lightweight-types/ru.md) | [Следующий день →](../038-borrowing-temporary-access/ru.md)
