# День 51: Практика ссылок — передаём портфель на анализ

## Аналогия из трейдинга

Представь, что у тебя есть инвестиционный портфель стоимостью миллионы долларов. Когда тебе нужна консультация аналитика, ты не передаёшь ему все свои активы физически — ты даёшь ему **доступ для просмотра**. Аналитик смотрит на твой портфель, анализирует его, но сами активы остаются у тебя. Это и есть **ссылки** в Rust — передача доступа без передачи владения.

## Зачем нужны ссылки в трейдинге?

Когда мы анализируем портфель, историю цен или список ордеров, мы не хотим:
- Копировать огромные объёмы данных (дорого по памяти)
- Терять владение данными после анализа (нужны для дальнейшей работы)
- Давать полный доступ на изменение, когда достаточно чтения

## Базовая структура портфеля

```rust
#[derive(Debug)]
struct Position {
    ticker: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn new(ticker: &str, quantity: f64, entry_price: f64, current_price: f64) -> Self {
        Position {
            ticker: ticker.to_string(),
            quantity,
            entry_price,
            current_price,
        }
    }

    fn market_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    fn unrealized_pnl(&self) -> f64 {
        self.quantity * (self.current_price - self.entry_price)
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[derive(Debug)]
struct Portfolio {
    name: String,
    positions: Vec<Position>,
    cash_balance: f64,
}

impl Portfolio {
    fn new(name: &str, cash_balance: f64) -> Self {
        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
            cash_balance,
        }
    }

    fn add_position(&mut self, position: Position) {
        self.positions.push(position);
    }
}
```

## Неизменяемые ссылки: анализ без изменений

### Расчёт общей стоимости портфеля

```rust
fn calculate_total_value(portfolio: &Portfolio) -> f64 {
    let positions_value: f64 = portfolio.positions
        .iter()
        .map(|p| p.market_value())
        .sum();

    positions_value + portfolio.cash_balance
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading Account", 10000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2500.0));
    portfolio.add_position(Position::new("AAPL", 50.0, 150.0, 175.0));

    // Передаём ссылку — портфель остаётся у нас
    let total = calculate_total_value(&portfolio);
    println!("Total portfolio value: ${:.2}", total);

    // Можем использовать portfolio дальше!
    println!("Portfolio name: {}", portfolio.name);
}
```

### Анализ всех позиций

```rust
fn analyze_positions(portfolio: &Portfolio) {
    println!("\n=== Portfolio Analysis: {} ===", portfolio.name);
    println!("{:<8} {:>10} {:>12} {:>10} {:>8}",
             "Ticker", "Quantity", "Value", "PnL", "PnL%");
    println!("{}", "-".repeat(52));

    for position in &portfolio.positions {
        println!("{:<8} {:>10.2} {:>12.2} {:>10.2} {:>7.2}%",
                 position.ticker,
                 position.quantity,
                 position.market_value(),
                 position.unrealized_pnl(),
                 position.pnl_percent());
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Crypto Portfolio", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2500.0));

    analyze_positions(&portfolio);

    // Портфель всё ещё наш
    println!("\nCash balance: ${:.2}", portfolio.cash_balance);
}
```

## Множественные неизменяемые ссылки

В Rust можно иметь сколько угодно неизменяемых ссылок одновременно:

```rust
fn calculate_total_pnl(portfolio: &Portfolio) -> f64 {
    portfolio.positions
        .iter()
        .map(|p| p.unrealized_pnl())
        .sum()
}

fn calculate_win_rate(portfolio: &Portfolio) -> f64 {
    let total = portfolio.positions.len() as f64;
    if total == 0.0 { return 0.0; }

    let winners = portfolio.positions
        .iter()
        .filter(|p| p.unrealized_pnl() > 0.0)
        .count() as f64;

    (winners / total) * 100.0
}

fn find_best_performer(portfolio: &Portfolio) -> Option<&Position> {
    portfolio.positions
        .iter()
        .max_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
}

fn find_worst_performer(portfolio: &Portfolio) -> Option<&Position> {
    portfolio.positions
        .iter()
        .min_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
}

fn main() {
    let mut portfolio = Portfolio::new("Mixed Portfolio", 10000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));  // Profit
    portfolio.add_position(Position::new("ETH", 5.0, 2800.0, 2500.0));   // Loss
    portfolio.add_position(Position::new("SOL", 100.0, 20.0, 35.0));     // Profit

    // Все функции получают ссылки — никаких конфликтов!
    let total_pnl = calculate_total_pnl(&portfolio);
    let win_rate = calculate_win_rate(&portfolio);
    let best = find_best_performer(&portfolio);
    let worst = find_worst_performer(&portfolio);

    println!("Total PnL: ${:.2}", total_pnl);
    println!("Win Rate: {:.1}%", win_rate);

    if let Some(pos) = best {
        println!("Best: {} ({:+.2}%)", pos.ticker, pos.pnl_percent());
    }
    if let Some(pos) = worst {
        println!("Worst: {} ({:+.2}%)", pos.ticker, pos.pnl_percent());
    }
}
```

## Изменяемые ссылки: модификация портфеля

Когда нужно изменить портфель, используем `&mut`:

```rust
fn update_prices(portfolio: &mut Portfolio, updates: &[(String, f64)]) {
    for (ticker, new_price) in updates {
        if let Some(position) = portfolio.positions
            .iter_mut()
            .find(|p| &p.ticker == ticker)
        {
            position.current_price = *new_price;
        }
    }
}

fn apply_deposit(portfolio: &mut Portfolio, amount: f64) {
    portfolio.cash_balance += amount;
    println!("Deposited ${:.2}. New balance: ${:.2}",
             amount, portfolio.cash_balance);
}

fn close_position(portfolio: &mut Portfolio, ticker: &str) -> Option<f64> {
    if let Some(idx) = portfolio.positions
        .iter()
        .position(|p| p.ticker == ticker)
    {
        let position = portfolio.positions.remove(idx);
        let realized_pnl = position.unrealized_pnl();
        let proceeds = position.market_value();

        portfolio.cash_balance += proceeds;

        println!("Closed {} position. Proceeds: ${:.2}, PnL: ${:.2}",
                 ticker, proceeds, realized_pnl);

        Some(realized_pnl)
    } else {
        println!("Position {} not found", ticker);
        None
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Active Trading", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 42000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2200.0));

    // Обновляем цены
    let price_updates = vec![
        ("BTC".to_string(), 45000.0),
        ("ETH".to_string(), 2500.0),
    ];
    update_prices(&mut portfolio, &price_updates);

    // Добавляем средства
    apply_deposit(&mut portfolio, 2000.0);

    analyze_positions(&portfolio);  // Неизменяемая ссылка для анализа

    // Закрываем позицию
    close_position(&mut portfolio, "BTC");

    println!("\nFinal cash balance: ${:.2}", portfolio.cash_balance);
}
```

## Правило: одна изменяемая ИЛИ много неизменяемых

```rust
fn main() {
    let mut portfolio = Portfolio::new("Test", 1000.0);
    portfolio.add_position(Position::new("BTC", 0.1, 40000.0, 42000.0));

    // ОК: несколько неизменяемых ссылок
    let ref1 = &portfolio;
    let ref2 = &portfolio;
    println!("Value via ref1: ${:.2}", calculate_total_value(ref1));
    println!("Value via ref2: ${:.2}", calculate_total_value(ref2));

    // ОК: одна изменяемая ссылка (после окончания жизни неизменяемых)
    let ref_mut = &mut portfolio;
    apply_deposit(ref_mut, 500.0);

    // Это НЕ скомпилируется:
    // let ref3 = &portfolio;      // неизменяемая ссылка
    // apply_deposit(&mut portfolio, 100.0);  // изменяемая ссылка
    // println!("{:?}", ref3);     // использование неизменяемой
}
```

## Практический пример: полный анализатор портфеля

```rust
#[derive(Debug)]
struct PortfolioAnalysis {
    total_value: f64,
    total_invested: f64,
    total_pnl: f64,
    total_pnl_percent: f64,
    win_rate: f64,
    best_performer: Option<String>,
    worst_performer: Option<String>,
    largest_position: Option<String>,
    risk_concentration: f64,
}

fn analyze_portfolio_full(portfolio: &Portfolio) -> PortfolioAnalysis {
    let total_value = calculate_total_value(portfolio);

    let total_invested: f64 = portfolio.positions
        .iter()
        .map(|p| p.quantity * p.entry_price)
        .sum();

    let total_pnl = calculate_total_pnl(portfolio);

    let total_pnl_percent = if total_invested > 0.0 {
        (total_pnl / total_invested) * 100.0
    } else {
        0.0
    };

    let win_rate = calculate_win_rate(portfolio);

    let best = find_best_performer(portfolio)
        .map(|p| p.ticker.clone());

    let worst = find_worst_performer(portfolio)
        .map(|p| p.ticker.clone());

    let largest = portfolio.positions
        .iter()
        .max_by(|a, b| a.market_value().partial_cmp(&b.market_value()).unwrap())
        .map(|p| p.ticker.clone());

    // Концентрация риска: доля самой большой позиции
    let risk_concentration = if let Some(max_pos) = portfolio.positions
        .iter()
        .max_by(|a, b| a.market_value().partial_cmp(&b.market_value()).unwrap())
    {
        (max_pos.market_value() / total_value) * 100.0
    } else {
        0.0
    };

    PortfolioAnalysis {
        total_value,
        total_invested,
        total_pnl,
        total_pnl_percent,
        win_rate,
        best_performer: best,
        worst_performer: worst,
        largest_position: largest,
        risk_concentration,
    }
}

fn print_analysis(analysis: &PortfolioAnalysis) {
    println!("\n╔════════════════════════════════════════╗");
    println!("║       PORTFOLIO ANALYSIS REPORT        ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Total Value:     ${:>18.2} ║", analysis.total_value);
    println!("║ Total Invested:  ${:>18.2} ║", analysis.total_invested);
    println!("║ Total PnL:       ${:>18.2} ║", analysis.total_pnl);
    println!("║ Return:          {:>18.2}% ║", analysis.total_pnl_percent);
    println!("║ Win Rate:        {:>18.2}% ║", analysis.win_rate);
    println!("╠════════════════════════════════════════╣");

    if let Some(ref best) = analysis.best_performer {
        println!("║ Best Performer:  {:>20} ║", best);
    }
    if let Some(ref worst) = analysis.worst_performer {
        println!("║ Worst Performer: {:>20} ║", worst);
    }
    if let Some(ref largest) = analysis.largest_position {
        println!("║ Largest Position:{:>20} ║", largest);
    }

    println!("╠════════════════════════════════════════╣");
    println!("║ Risk Concentration:{:>17.2}% ║", analysis.risk_concentration);

    if analysis.risk_concentration > 50.0 {
        println!("║ ⚠️  WARNING: High concentration risk!   ║");
    }

    println!("╚════════════════════════════════════════╝");
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading Portfolio", 15000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 48000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2800.0));
    portfolio.add_position(Position::new("SOL", 200.0, 25.0, 35.0));
    portfolio.add_position(Position::new("AAPL", 30.0, 170.0, 165.0));
    portfolio.add_position(Position::new("NVDA", 20.0, 450.0, 520.0));

    // Полный анализ с использованием ссылок
    let analysis = analyze_portfolio_full(&portfolio);
    print_analysis(&analysis);

    // Портфель всё ещё доступен для дальнейшей работы
    println!("\nPositions count: {}", portfolio.positions.len());
}
```

## Ссылки в замыканиях

```rust
fn filter_profitable_positions(portfolio: &Portfolio) -> Vec<&Position> {
    portfolio.positions
        .iter()
        .filter(|p| p.unrealized_pnl() > 0.0)
        .collect()
}

fn filter_positions_by_pnl_threshold(
    portfolio: &Portfolio,
    threshold: f64
) -> Vec<&Position> {
    portfolio.positions
        .iter()
        .filter(|p| p.pnl_percent().abs() > threshold)
        .collect()
}

fn main() {
    let mut portfolio = Portfolio::new("Test", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 40000.0, 45000.0));  // +12.5%
    portfolio.add_position(Position::new("ETH", 10.0, 2800.0, 2500.0));  // -10.7%
    portfolio.add_position(Position::new("SOL", 100.0, 30.0, 31.0));     // +3.3%

    let profitable = filter_profitable_positions(&portfolio);
    println!("Profitable positions: {:?}",
             profitable.iter().map(|p| &p.ticker).collect::<Vec<_>>());

    // Позиции с движением более 5%
    let significant_moves = filter_positions_by_pnl_threshold(&portfolio, 5.0);
    println!("Significant moves (>5%): {:?}",
             significant_moves.iter().map(|p| &p.ticker).collect::<Vec<_>>());
}
```

## Что мы узнали

| Концепция | Синтаксис | Когда использовать |
|-----------|-----------|-------------------|
| Неизменяемая ссылка | `&T` | Чтение без изменения |
| Изменяемая ссылка | `&mut T` | Чтение и модификация |
| Множественные `&T` | Допускается | Параллельное чтение |
| Одна `&mut T` | Обязательно | Эксклюзивный доступ |
| Возврат ссылки | `-> &T` | Часть данных владельца |

## Домашнее задание

1. Напиши функцию `calculate_sector_allocation(portfolio: &Portfolio, sectors: &HashMap<String, String>) -> HashMap<String, f64>`, которая считает распределение по секторам (например: "BTC" -> "Crypto", "AAPL" -> "Tech")

2. Создай функцию `rebalance_suggestions(portfolio: &Portfolio, target_weights: &HashMap<String, f64>) -> Vec<(String, f64)>`, которая возвращает рекомендации по ребалансировке (тикер, сумма к покупке/продаже)

3. Реализуй функцию `risk_metrics(portfolio: &Portfolio, benchmark: &[f64]) -> RiskMetrics`, которая вычисляет метрики риска: волатильность, бета, максимальную просадку

4. Напиши систему оповещений `check_alerts(portfolio: &Portfolio, alerts: &[Alert]) -> Vec<String>`, которая проверяет условия (цена > X, PnL < Y) и возвращает сработавшие алерты

## Навигация

[← Предыдущий день](../050-ownership-practice-trade-analysis/ru.md) | [Следующий день →](../052-slice-practice-partial-history/ru.md)
