# День 46: Множественные параметры времени жизни

## Аналогия из трейдинга

Представьте, что вы анализируете сделку, используя данные из двух разных источников:
- **Биржевые данные** (котировки, объёмы) — обновляются в реальном времени
- **Исторические данные** (архив сделок) — хранятся долгосрочно

Эти данные имеют **разное время жизни**: биржевые данные живут пока открыто соединение с биржей, а исторические — пока работает программа. Когда функция работает с обоими источниками одновременно, Rust требует явно указать, какие данные живут дольше.

## Зачем нужны несколько параметров времени жизни?

Когда функция принимает несколько ссылок и возвращает ссылку, компилятор должен знать: возвращаемая ссылка связана с каким входным параметром?

```rust
// Один параметр времени жизни — достаточно, когда все ссылки связаны одинаково
fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42050.0;
    let best = get_best_price(&bid, &ask);
    println!("Best price: {}", best);
}
```

Но что если ссылки имеют **разное время жизни**?

## Два параметра времени жизни

```rust
// Функция сравнивает текущую цену с исторической
// Возвращает ссылку ТОЛЬКО на текущую цену
fn compare_with_history<'current, 'history>(
    current_price: &'current f64,
    historical_price: &'history f64,
) -> &'current f64 {
    println!("Historical: ${:.2}, Current: ${:.2}", historical_price, current_price);
    current_price  // Возвращаем только текущую цену
}

fn main() {
    let current = 42500.0;

    let result = {
        let historical = 41000.0;  // Живёт только в этом блоке
        compare_with_history(&current, &historical)
    };  // historical уничтожена здесь

    // result всё ещё валиден, потому что связан с current, а не с historical
    println!("Result: ${:.2}", result);
}
```

## Практический пример: Анализатор ордеров

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct MarketData {
    bid: f64,
    ask: f64,
    last_price: f64,
}

// Функция принимает ордер и рыночные данные с разным временем жизни
// Возвращает ссылку на цену из рыночных данных
fn get_execution_price<'order, 'market>(
    order: &'order Order,
    market: &'market MarketData,
) -> &'market f64 {
    // Логика определения цены исполнения
    if order.quantity > 0.0 {
        // Покупка исполняется по ask
        &market.ask
    } else {
        // Продажа исполняется по bid
        &market.bid
    }
}

fn main() {
    let market_data = MarketData {
        bid: 41950.0,
        ask: 42050.0,
        last_price: 42000.0,
    };

    let execution_price = {
        let buy_order = Order {
            symbol: String::from("BTC/USDT"),
            price: 42000.0,
            quantity: 0.5,
        };

        get_execution_price(&buy_order, &market_data)
    };  // buy_order уничтожен, но execution_price валиден

    println!("Execution price: ${:.2}", execution_price);
}
```

## Выбор возвращаемого времени жизни

```rust
struct PriceLevel {
    price: f64,
    volume: f64,
}

struct OrderBook {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

// Возвращаем ссылку из первого ИЛИ второго аргумента
// Оба должны жить достаточно долго
fn get_best_level<'a>(
    order_book: &'a OrderBook,
    side: &str,
) -> Option<&'a PriceLevel> {
    match side {
        "buy" => order_book.asks.first(),
        "sell" => order_book.bids.first(),
        _ => None,
    }
}

// Сравниваем два стакана, возвращаем лучшую цену
// Здесь нужны ДВА параметра времени жизни
fn compare_order_books<'a, 'b>(
    book_a: &'a OrderBook,
    book_b: &'b OrderBook,
) -> (Option<&'a PriceLevel>, Option<&'b PriceLevel>) {
    let best_a = book_a.bids.first();
    let best_b = book_b.bids.first();
    (best_a, best_b)
}

fn main() {
    let book1 = OrderBook {
        bids: vec![PriceLevel { price: 42000.0, volume: 1.5 }],
        asks: vec![PriceLevel { price: 42050.0, volume: 2.0 }],
    };

    let book2 = OrderBook {
        bids: vec![PriceLevel { price: 41990.0, volume: 3.0 }],
        asks: vec![PriceLevel { price: 42060.0, volume: 1.0 }],
    };

    let (best1, best2) = compare_order_books(&book1, &book2);

    if let (Some(b1), Some(b2)) = (best1, best2) {
        println!("Book1 best bid: ${:.2}", b1.price);
        println!("Book2 best bid: ${:.2}", b2.price);
    }
}
```

## Структуры с несколькими временами жизни

```rust
// Анализатор хранит ссылки на данные с РАЗНЫМ временем жизни
struct TradeAnalyzer<'current, 'historical> {
    current_data: &'current [f64],
    historical_data: &'historical [f64],
}

impl<'current, 'historical> TradeAnalyzer<'current, 'historical> {
    fn new(
        current: &'current [f64],
        historical: &'historical [f64],
    ) -> Self {
        TradeAnalyzer {
            current_data: current,
            historical_data: historical,
        }
    }

    // Возвращает ссылку только на текущие данные
    fn get_current_high(&self) -> Option<&'current f64> {
        self.current_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    // Возвращает ссылку только на исторические данные
    fn get_historical_average(&self) -> f64 {
        if self.historical_data.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.historical_data.iter().sum();
        sum / self.historical_data.len() as f64
    }

    // Сравнивает текущие данные с историческими
    fn analyze(&self) {
        let current_high = self.get_current_high().unwrap_or(&0.0);
        let historical_avg = self.get_historical_average();

        println!("Current High: ${:.2}", current_high);
        println!("Historical Avg: ${:.2}", historical_avg);

        let diff_percent = ((current_high - historical_avg) / historical_avg) * 100.0;
        println!("Difference: {:.2}%", diff_percent);
    }
}

fn main() {
    // Исторические данные живут всё время программы
    let historical_prices = vec![40000.0, 41000.0, 39500.0, 42000.0, 41500.0];

    // Текущие данные могут обновляться
    let current_prices = vec![42500.0, 42600.0, 42400.0, 42550.0];

    let analyzer = TradeAnalyzer::new(&current_prices, &historical_prices);
    analyzer.analyze();
}
```

## Практический пример: Сравнение портфелей

```rust
#[derive(Debug)]
struct Portfolio {
    name: String,
    assets: Vec<(String, f64)>,  // (тикер, сумма)
}

#[derive(Debug)]
struct PortfolioComparison<'a, 'b> {
    portfolio_a: &'a Portfolio,
    portfolio_b: &'b Portfolio,
}

impl<'a, 'b> PortfolioComparison<'a, 'b> {
    fn new(a: &'a Portfolio, b: &'b Portfolio) -> Self {
        PortfolioComparison {
            portfolio_a: a,
            portfolio_b: b,
        }
    }

    fn total_value(portfolio: &Portfolio) -> f64 {
        portfolio.assets.iter().map(|(_, value)| value).sum()
    }

    fn get_larger(&self) -> &str {
        let value_a = Self::total_value(self.portfolio_a);
        let value_b = Self::total_value(self.portfolio_b);

        if value_a >= value_b {
            &self.portfolio_a.name
        } else {
            &self.portfolio_b.name
        }
    }

    fn compare(&self) {
        let value_a = Self::total_value(self.portfolio_a);
        let value_b = Self::total_value(self.portfolio_b);

        println!("=== Portfolio Comparison ===");
        println!("{}: ${:.2}", self.portfolio_a.name, value_a);
        println!("{}: ${:.2}", self.portfolio_b.name, value_b);
        println!("Larger: {}", self.get_larger());

        let diff = (value_a - value_b).abs();
        let diff_percent = (diff / value_a.max(value_b)) * 100.0;
        println!("Difference: ${:.2} ({:.2}%)", diff, diff_percent);
    }
}

fn main() {
    let conservative = Portfolio {
        name: String::from("Conservative"),
        assets: vec![
            (String::from("BTC"), 50000.0),
            (String::from("ETH"), 30000.0),
            (String::from("USDT"), 20000.0),
        ],
    };

    let aggressive = Portfolio {
        name: String::from("Aggressive"),
        assets: vec![
            (String::from("BTC"), 80000.0),
            (String::from("SOL"), 15000.0),
            (String::from("DOGE"), 5000.0),
        ],
    };

    let comparison = PortfolioComparison::new(&conservative, &aggressive);
    comparison.compare();
}
```

## Когда использовать разные времена жизни

| Ситуация | Решение |
|----------|---------|
| Все ссылки связаны одинаково | Один параметр `'a` |
| Возврат связан только с одним аргументом | Разные параметры |
| Структура хранит независимые ссылки | Разные параметры |
| Сравнение двух независимых источников | Разные параметры |

## Частые ошибки

```rust
// ОШИБКА: Возвращаем ссылку на локальную переменную
fn bad_example<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    let local = String::from("local");
    // &local  // Нельзя! local уничтожится
    x  // Правильно: возвращаем входной параметр
}

// ОШИБКА: Неправильное время жизни возврата
// fn wrong_lifetime<'a, 'b>(x: &'a str, y: &'b str) -> &'b str {
//     x  // Нельзя! x имеет время жизни 'a, а не 'b
// }

// ПРАВИЛЬНО: Время жизни соответствует возвращаемой ссылке
fn correct<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x  // x живёт 'a, возвращаем 'a — всё совпадает
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `<'a, 'b>` | Два параметра времени жизни |
| Независимые ссылки | Разные источники данных |
| Связь с возвратом | Возврат связан с конкретным параметром |
| Структуры | Могут хранить ссылки с разным временем жизни |

## Упражнения

1. **Сравнение котировок**: Напишите функцию, которая принимает котировки с двух бирж (с разным временем жизни) и возвращает лучшую цену.

2. **Анализатор спреда**: Создайте структуру `SpreadAnalyzer<'bid, 'ask>`, которая хранит ссылки на массивы bid и ask цен с разным временем жизни.

3. **Менеджер риска**: Реализуйте структуру, которая сравнивает текущую позицию с лимитами риска, используя разные времена жизни.

4. **Калькулятор PnL**: Напишите функцию с двумя параметрами времени жизни для расчёта PnL, используя текущие и исторические цены.

## Домашнее задание

1. Создайте структуру `TradeMatcher<'orders, 'market>` для сопоставления ордеров с рыночными данными:
   ```rust
   struct TradeMatcher<'orders, 'market> {
       pending_orders: &'orders [Order],
       market_data: &'market MarketData,
   }
   ```

2. Реализуйте функцию `find_arbitrage<'a, 'b>` для поиска арбитражных возможностей между двумя биржами.

3. Создайте анализатор, который сравнивает производительность двух стратегий с разным временем жизни данных.

## Навигация

[← Предыдущий день](../chapter-045-lifetimes-basics/ru.md) | [Следующий день →](../chapter-047-lifetime-elision/ru.md)
