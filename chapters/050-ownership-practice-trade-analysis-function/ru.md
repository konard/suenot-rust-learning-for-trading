# День 50: Практика владения — функция анализа сделок

## Аналогия из трейдинга

Представь, что у тебя есть **уникальный торговый отчёт** — единственный экземпляр документа с детальным анализом сделки. Когда ты передаёшь этот отчёт аналитику:

1. **Передача владения (move)**: Ты отдаёшь отчёт — теперь он у аналитика, а у тебя его нет
2. **Заимствование (borrow)**: Ты даёшь посмотреть отчёт — аналитик читает, но отчёт остаётся твоим
3. **Мутабельное заимствование**: Ты даёшь карандаш для пометок — аналитик может делать заметки, но только один человек за раз

В Rust эти концепции применяются к данным в памяти — и это критически важно для безопасной обработки торговых данных!

## Теория: Правила владения в Rust

### Три главных правила

```rust
// 1. У каждого значения есть владелец (owner)
let trade_data = String::from("BTC/USDT: +$1500");

// 2. В каждый момент времени может быть только один владелец
let analysis = trade_data;  // Владение перешло к analysis
// println!("{}", trade_data);  // Ошибка! trade_data больше не владеет данными

// 3. Когда владелец выходит из области видимости, значение удаляется
{
    let temp_report = String::from("Temporary");
}  // temp_report удаляется здесь
```

### Почему это важно для трейдинга?

В торговых системах критически важно:
- **Не дублировать ордера** — случайная копия может привести к двойному исполнению
- **Не использовать устаревшие данные** — цены меняются каждую миллисекунду
- **Гарантировать освобождение ресурсов** — соединения с биржей должны закрываться корректно

## Move: Передача владения данными сделки

```rust
fn main() {
    // Создаём данные сделки
    let trade = create_trade("BTC/USDT", 42000.0, 0.5, "BUY");

    // Передаём владение в функцию анализа
    let report = analyze_trade(trade);

    // trade больше недоступна — владение передано!
    // println!("{:?}", trade);  // Ошибка компиляции

    println!("{}", report);
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn create_trade(symbol: &str, price: f64, quantity: f64, side: &str) -> Trade {
    Trade {
        symbol: String::from(symbol),
        price,
        quantity,
        side: String::from(side),
    }
}

fn analyze_trade(trade: Trade) -> String {
    let value = trade.price * trade.quantity;
    format!(
        "Trade Analysis:\n  Symbol: {}\n  Side: {}\n  Value: ${:.2}",
        trade.symbol, trade.side, value
    )
}
```

## Borrow: Заимствование для чтения

```rust
fn main() {
    let portfolio = create_portfolio();

    // Заимствуем для чтения — можно создавать много ссылок
    print_portfolio_summary(&portfolio);
    let total = calculate_total_value(&portfolio);
    let risk = assess_risk(&portfolio);

    // portfolio всё ещё доступна!
    println!("\nTotal: ${:.2}, Risk: {}", total, risk);
    println!("Positions count: {}", portfolio.len());
}

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

fn create_portfolio() -> Vec<Position> {
    vec![
        Position {
            symbol: String::from("BTC"),
            quantity: 0.5,
            entry_price: 42000.0,
            current_price: 43500.0,
        },
        Position {
            symbol: String::from("ETH"),
            quantity: 5.0,
            entry_price: 2800.0,
            current_price: 2650.0,
        },
        Position {
            symbol: String::from("SOL"),
            quantity: 100.0,
            entry_price: 95.0,
            current_price: 110.0,
        },
    ]
}

fn print_portfolio_summary(portfolio: &Vec<Position>) {
    println!("=== Portfolio Summary ===");
    for pos in portfolio {
        let pnl = (pos.current_price - pos.entry_price) * pos.quantity;
        let status = if pnl >= 0.0 { "+" } else { "" };
        println!("  {}: {}${:.2}", pos.symbol, status, pnl);
    }
}

fn calculate_total_value(portfolio: &Vec<Position>) -> f64 {
    portfolio.iter()
        .map(|pos| pos.current_price * pos.quantity)
        .sum()
}

fn assess_risk(portfolio: &Vec<Position>) -> &str {
    let losing_positions = portfolio.iter()
        .filter(|pos| pos.current_price < pos.entry_price)
        .count();

    let ratio = losing_positions as f64 / portfolio.len() as f64;

    if ratio > 0.5 {
        "HIGH"
    } else if ratio > 0.25 {
        "MEDIUM"
    } else {
        "LOW"
    }
}
```

## Mutable Borrow: Изменение данных

```rust
fn main() {
    let mut order_book = OrderBook::new();

    // Добавляем ордера — мутабельное заимствование
    add_bid(&mut order_book, 41900.0, 1.5);
    add_bid(&mut order_book, 41800.0, 2.0);
    add_ask(&mut order_book, 42100.0, 1.0);
    add_ask(&mut order_book, 42200.0, 3.0);

    // Читаем — иммутабельное заимствование
    print_order_book(&order_book);

    // Исполняем ордер — снова мутабельное
    execute_market_buy(&mut order_book, 0.5);

    println!("\nAfter execution:");
    print_order_book(&order_book);
}

struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

fn add_bid(book: &mut OrderBook, price: f64, quantity: f64) {
    book.bids.push((price, quantity));
    // Сортируем по убыванию цены (лучший bid сверху)
    book.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
}

fn add_ask(book: &mut OrderBook, price: f64, quantity: f64) {
    book.asks.push((price, quantity));
    // Сортируем по возрастанию цены (лучший ask сверху)
    book.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
}

fn print_order_book(book: &OrderBook) {
    println!("╔══════════════════════════╗");
    println!("║      ORDER BOOK          ║");
    println!("╠══════════════════════════╣");
    println!("║ ASKS (Sell orders)       ║");
    for (price, qty) in book.asks.iter().rev() {
        println!("║   ${:.2} x {:.4}         ║", price, qty);
    }
    println!("╠══════════════════════════╣");
    println!("║ BIDS (Buy orders)        ║");
    for (price, qty) in &book.bids {
        println!("║   ${:.2} x {:.4}         ║", price, qty);
    }
    println!("╚══════════════════════════╝");
}

fn execute_market_buy(book: &mut OrderBook, quantity: f64) {
    let mut remaining = quantity;

    while remaining > 0.0 && !book.asks.is_empty() {
        let (price, available) = book.asks[0];

        if available <= remaining {
            println!("Filled {:.4} @ ${:.2}", available, price);
            remaining -= available;
            book.asks.remove(0);
        } else {
            println!("Filled {:.4} @ ${:.2}", remaining, price);
            book.asks[0].1 -= remaining;
            remaining = 0.0;
        }
    }

    if remaining > 0.0 {
        println!("Warning: {:.4} unfilled (no liquidity)", remaining);
    }
}
```

## Практический пример: Полный анализатор сделок

```rust
fn main() {
    // Создаём историю сделок
    let mut trade_history = TradeHistory::new();

    // Добавляем сделки
    record_trade(&mut trade_history, "BTC/USDT", 42000.0, 43500.0, 0.5);
    record_trade(&mut trade_history, "ETH/USDT", 2800.0, 2650.0, 5.0);
    record_trade(&mut trade_history, "BTC/USDT", 43000.0, 44200.0, 0.3);
    record_trade(&mut trade_history, "SOL/USDT", 95.0, 88.0, 100.0);
    record_trade(&mut trade_history, "BTC/USDT", 44000.0, 43800.0, 0.2);

    // Анализируем — передаём по ссылке
    let analysis = analyze_history(&trade_history);
    print_analysis(&analysis);

    // Фильтруем прибыльные — ещё одно заимствование
    let profitable = filter_profitable(&trade_history);
    println!("\nProfitable trades: {}", profitable.len());

    // Группируем по символу
    let by_symbol = group_by_symbol(&trade_history);
    print_grouped_stats(&by_symbol);
}

struct TradeRecord {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

struct TradeHistory {
    trades: Vec<TradeRecord>,
}

impl TradeHistory {
    fn new() -> Self {
        TradeHistory { trades: Vec::new() }
    }
}

struct HistoryAnalysis {
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    total_pnl: f64,
    largest_win: f64,
    largest_loss: f64,
    win_rate: f64,
    average_pnl: f64,
}

fn record_trade(
    history: &mut TradeHistory,
    symbol: &str,
    entry: f64,
    exit: f64,
    quantity: f64,
) {
    let pnl = (exit - entry) * quantity;
    history.trades.push(TradeRecord {
        symbol: String::from(symbol),
        entry_price: entry,
        exit_price: exit,
        quantity,
        pnl,
    });
}

fn analyze_history(history: &TradeHistory) -> HistoryAnalysis {
    let total_trades = history.trades.len();

    if total_trades == 0 {
        return HistoryAnalysis {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_pnl: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            win_rate: 0.0,
            average_pnl: 0.0,
        };
    }

    let winning_trades = history.trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing_trades = history.trades.iter().filter(|t| t.pnl < 0.0).count();
    let total_pnl: f64 = history.trades.iter().map(|t| t.pnl).sum();

    let largest_win = history.trades.iter()
        .map(|t| t.pnl)
        .fold(0.0_f64, |a, b| a.max(b));

    let largest_loss = history.trades.iter()
        .map(|t| t.pnl)
        .fold(0.0_f64, |a, b| a.min(b));

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    let average_pnl = total_pnl / total_trades as f64;

    HistoryAnalysis {
        total_trades,
        winning_trades,
        losing_trades,
        total_pnl,
        largest_win,
        largest_loss,
        win_rate,
        average_pnl,
    }
}

fn print_analysis(analysis: &HistoryAnalysis) {
    println!("╔════════════════════════════════════╗");
    println!("║      TRADING HISTORY ANALYSIS      ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Total trades:      {:>15} ║", analysis.total_trades);
    println!("║ Winning trades:    {:>15} ║", analysis.winning_trades);
    println!("║ Losing trades:     {:>15} ║", analysis.losing_trades);
    println!("║ Win rate:          {:>14.1}% ║", analysis.win_rate);
    println!("╠════════════════════════════════════╣");
    println!("║ Total PnL:        ${:>14.2} ║", analysis.total_pnl);
    println!("║ Average PnL:      ${:>14.2} ║", analysis.average_pnl);
    println!("║ Largest win:      ${:>14.2} ║", analysis.largest_win);
    println!("║ Largest loss:     ${:>14.2} ║", analysis.largest_loss);
    println!("╚════════════════════════════════════╝");
}

fn filter_profitable(history: &TradeHistory) -> Vec<&TradeRecord> {
    history.trades.iter().filter(|t| t.pnl > 0.0).collect()
}

fn group_by_symbol(history: &TradeHistory) -> std::collections::HashMap<String, Vec<&TradeRecord>> {
    let mut groups: std::collections::HashMap<String, Vec<&TradeRecord>> = std::collections::HashMap::new();

    for trade in &history.trades {
        groups.entry(trade.symbol.clone())
            .or_insert_with(Vec::new)
            .push(trade);
    }

    groups
}

fn print_grouped_stats(groups: &std::collections::HashMap<String, Vec<&TradeRecord>>) {
    println!("\n=== Performance by Symbol ===");

    for (symbol, trades) in groups {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (wins as f64 / trades.len() as f64) * 100.0;

        let status = if total_pnl >= 0.0 { "+" } else { "" };
        println!(
            "  {}: {} trades, {}${:.2}, {:.0}% win rate",
            symbol,
            trades.len(),
            status,
            total_pnl,
            win_rate
        );
    }
}
```

## Упражнение 1: Трекер позиций с владением

```rust
fn main() {
    // Создай структуру PositionTracker
    // Реализуй:
    // - open_position(&mut self, ...) -> добавляет позицию
    // - close_position(&mut self, symbol) -> закрывает и возвращает PnL
    // - get_position(&self, symbol) -> возвращает &Position или None
    // - calculate_exposure(&self) -> общий размер позиций

    let mut tracker = PositionTracker::new();

    tracker.open_position("BTC", 42000.0, 0.5);
    tracker.open_position("ETH", 2800.0, 5.0);

    println!("BTC position: {:?}", tracker.get_position("BTC"));
    println!("Total exposure: ${:.2}", tracker.calculate_exposure());

    if let Some(pnl) = tracker.close_position("BTC", 43000.0) {
        println!("Closed BTC with PnL: ${:.2}", pnl);
    }
}

// Твоя реализация здесь...
```

## Упражнение 2: Валидатор ордеров

```rust
fn main() {
    // Создай функции:
    // - validate_order(&Order) -> Result<(), String>
    // - enrich_order(&mut Order) -> добавляет timestamp и ID
    // - submit_order(Order) -> OrderReceipt (передаёт владение)

    let mut order = Order::new("BTC/USDT", 42000.0, 0.5, "LIMIT", "BUY");

    // Валидация не меняет ордер
    match validate_order(&order) {
        Ok(()) => println!("Order valid"),
        Err(e) => println!("Invalid: {}", e),
    }

    // Обогащение меняет ордер
    enrich_order(&mut order);

    // Отправка забирает владение
    let receipt = submit_order(order);
    // order больше недоступен!

    println!("Receipt: {:?}", receipt);
}

// Твоя реализация здесь...
```

## Упражнение 3: Анализатор ликвидности

```rust
fn main() {
    // Реализуй функции, правильно используя заимствование:
    // - calculate_spread(&OrderBook) -> f64
    // - calculate_depth(&OrderBook, levels: usize) -> (f64, f64)
    // - find_price_impact(&OrderBook, quantity: f64) -> f64
    // - merge_order_books(&OrderBook, &OrderBook) -> OrderBook

    let book1 = create_order_book_1();
    let book2 = create_order_book_2();

    let spread = calculate_spread(&book1);
    println!("Spread: ${:.2}", spread);

    let (bid_depth, ask_depth) = calculate_depth(&book1, 3);
    println!("Depth - Bids: ${:.2}, Asks: ${:.2}", bid_depth, ask_depth);

    let impact = find_price_impact(&book1, 2.0);
    println!("Price impact for 2 BTC: ${:.2}", impact);

    // Объединение не потребляет исходные книги
    let merged = merge_order_books(&book1, &book2);
    println!("Merged book has {} bids", merged.bids.len());

    // book1 и book2 всё ещё доступны
    println!("Original book1 spread: ${:.2}", calculate_spread(&book1));
}

// Твоя реализация здесь...
```

## Упражнение 4: Менеджер рисков

```rust
fn main() {
    // Создай RiskManager с правилами владения:
    // - new(max_position: f64, max_loss: f64) -> RiskManager
    // - check_order(&self, &Order, &Portfolio) -> Result<(), RiskError>
    // - update_limits(&mut self, new_max: f64)
    // - consume_and_report(self) -> RiskReport (потребляет менеджер)

    let mut risk_manager = RiskManager::new(100000.0, 5000.0);
    let portfolio = create_test_portfolio();
    let order = create_test_order();

    // Проверка не меняет ничего
    match risk_manager.check_order(&order, &portfolio) {
        Ok(()) => println!("Order passed risk check"),
        Err(e) => println!("Risk violation: {:?}", e),
    }

    // Обновляем лимиты
    risk_manager.update_limits(150000.0);

    // Финальный отчёт потребляет менеджер
    let report = risk_manager.consume_and_report();
    // risk_manager больше недоступен!

    println!("Report: {:?}", report);
}

// Твоя реализация здесь...
```

## Что мы узнали

| Концепция | Синтаксис | Применение в трейдинге |
|-----------|-----------|------------------------|
| Move | `fn process(data: Data)` | Передача ордера на исполнение |
| Borrow | `fn analyze(data: &Data)` | Чтение портфеля для отчёта |
| Mut Borrow | `fn update(data: &mut Data)` | Обновление позиции |
| Возврат владения | `fn create() -> Data` | Создание нового ордера |
| Lifetime | `fn get<'a>(&'a self) -> &'a T` | Ссылки на данные структуры |

## Частые ошибки и как их избежать

```rust
// ❌ Ошибка: использование после move
fn bad_example() {
    let order = Order::new();
    submit(order);
    println!("{:?}", order);  // Ошибка!
}

// ✅ Правильно: клонировать или использовать ссылку
fn good_example() {
    let order = Order::new();
    submit(order.clone());  // Отправляем копию
    println!("{:?}", order);  // Оригинал доступен
}

// ❌ Ошибка: одновременное мутабельное заимствование
fn bad_borrow() {
    let mut book = OrderBook::new();
    let ref1 = &mut book;
    let ref2 = &mut book;  // Ошибка!
}

// ✅ Правильно: последовательные заимствования
fn good_borrow() {
    let mut book = OrderBook::new();
    {
        let ref1 = &mut book;
        update(ref1);
    }  // ref1 освобождается
    let ref2 = &mut book;  // Теперь можно
}
```

## Домашнее задание

1. **Трекер портфеля**: Создай `PortfolioTracker`, который хранит позиции и правильно управляет владением при добавлении/удалении/обновлении позиций

2. **Система алертов**: Реализуй `AlertSystem`, где алерты передаются по владению при срабатывании, но проверяются по ссылке

3. **Агрегатор данных**: Напиши функции для агрегации рыночных данных, используя только заимствование (без копирования больших массивов)

4. **Конвейер анализа**: Создай цепочку функций анализа, где каждая функция принимает данные по ссылке и возвращает новую структуру с результатами

## Навигация

[← Предыдущий день](../049-ownership-borrowing-trading-data/ru.md) | [Следующий день →](../051-references-market-data-sharing/ru.md)
