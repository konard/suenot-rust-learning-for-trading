# День 45: Lifetime Elision — когда Rust сам определяет время жизни

## Аналогия из трейдинга

Представь опытного трейдера, который смотрит на ордер и сразу понимает, к какой сделке он относится — без необходимости каждый раз указывать это явно. Rust работает так же: компилятор достаточно умён, чтобы в большинстве случаев **автоматически вывести** времена жизни ссылок, не требуя явных аннотаций.

Это называется **lifetime elision** (опускание времён жизни) — набор правил, которые компилятор применяет автоматически.

## Зачем нужен lifetime elision?

Без elision нам пришлось бы писать так:

```rust
// Без elision — очень многословно
fn get_ticker<'a>(trade: &'a Trade) -> &'a str {
    &trade.ticker
}

fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}
```

С elision код становится чище:

```rust
// С elision — компилятор сам выводит времена жизни
fn get_ticker(trade: &Trade) -> &str {
    &trade.ticker
}

// Это работает, потому что Rust применяет правила elision
```

## Три правила Lifetime Elision

Компилятор применяет три правила по порядку:

### Правило 1: Каждая ссылка-параметр получает свой lifetime

```rust
// Что мы пишем:
fn analyze_price(price: &f64) -> bool { *price > 0.0 }

// Что компилятор видит:
fn analyze_price<'a>(price: &'a f64) -> bool { *price > 0.0 }
```

```rust
// Два параметра — два разных lifetime
fn compare_prices(bid: &f64, ask: &f64) -> bool { bid < ask }

// Компилятор видит:
fn compare_prices<'a, 'b>(bid: &'a f64, ask: &'b f64) -> bool { bid < ask }
```

### Правило 2: Если ровно один входной lifetime — он назначается всем выходным ссылкам

```rust
// Один входной параметр
fn get_ticker(trade: &Trade) -> &str {
    &trade.ticker
}

// Компилятор выводит:
fn get_ticker<'a>(trade: &'a Trade) -> &'a str {
    &trade.ticker
}
```

```rust
// Практический пример с ценой
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: String,
}

fn get_symbol(data: &PriceData) -> &str {
    &data.symbol
}

fn get_timestamp(data: &PriceData) -> &str {
    &data.timestamp
}

fn main() {
    let data = PriceData {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        timestamp: String::from("2024-01-15 10:30:00"),
    };

    println!("Symbol: {}", get_symbol(&data));
    println!("Time: {}", get_timestamp(&data));
}
```

### Правило 3: Для методов с &self — lifetime self назначается выходным ссылкам

```rust
struct OrderBook {
    symbol: String,
    bids: Vec<f64>,
    asks: Vec<f64>,
}

impl OrderBook {
    // &self есть, поэтому возвращаемая ссылка получает его lifetime
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn best_bid(&self) -> Option<&f64> {
        self.bids.first()
    }

    fn best_ask(&self) -> Option<&f64> {
        self.asks.first()
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
}

fn main() {
    let book = OrderBook {
        symbol: String::from("ETH/USDT"),
        bids: vec![2000.0, 1999.5, 1999.0],
        asks: vec![2001.0, 2001.5, 2002.0],
    };

    println!("Order Book: {}", book.symbol());
    println!("Best Bid: {:?}", book.best_bid());
    println!("Best Ask: {:?}", book.best_ask());
    println!("Spread: {:?}", book.spread());
}
```

## Когда elision НЕ работает

Иногда компилятор не может вывести lifetime автоматически:

### Несколько входных ссылок и возврат ссылки

```rust
// Ошибка компиляции! Компилятор не знает, какой lifetime использовать
// fn get_better_price(bid: &f64, ask: &f64) -> &f64 {
//     if bid > ask { bid } else { ask }
// }

// Нужно указать явно
fn get_better_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42050.0;
    let better = get_better_price(&bid, &ask);
    println!("Better price: {}", better);
}
```

### Разные lifetime для разных параметров

```rust
// Возвращаем ссылку только на один из параметров
fn get_longer_symbol<'a, 'b>(s1: &'a str, s2: &'b str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s1 } // Всегда возвращаем s1
}

// Или с одинаковым lifetime, если можем вернуть любой
fn get_longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s2 }
}

fn main() {
    let btc = "BTC/USDT";
    let eth = "ETH";

    println!("Longer: {}", get_longer(btc, eth));
}
```

## Практические примеры для трейдинга

### Анализатор портфеля

```rust
struct Portfolio {
    name: String,
    assets: Vec<Asset>,
    base_currency: String,
}

struct Asset {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Portfolio {
    fn new(name: &str, base_currency: &str) -> Self {
        Portfolio {
            name: name.to_string(),
            assets: Vec::new(),
            base_currency: base_currency.to_string(),
        }
    }

    // Правило 3: lifetime от &self
    fn name(&self) -> &str {
        &self.name
    }

    fn base_currency(&self) -> &str {
        &self.base_currency
    }

    fn get_asset(&self, symbol: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.symbol == symbol)
    }

    fn largest_position(&self) -> Option<&Asset> {
        self.assets.iter()
            .max_by(|a, b| {
                let value_a = a.quantity * a.avg_price;
                let value_b = b.quantity * b.avg_price;
                value_a.partial_cmp(&value_b).unwrap()
            })
    }

    fn add_asset(&mut self, symbol: &str, quantity: f64, price: f64) {
        self.assets.push(Asset {
            symbol: symbol.to_string(),
            quantity,
            avg_price: price,
        });
    }

    fn total_value(&self) -> f64 {
        self.assets.iter()
            .map(|a| a.quantity * a.avg_price)
            .sum()
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading", "USDT");

    portfolio.add_asset("BTC", 0.5, 42000.0);
    portfolio.add_asset("ETH", 5.0, 2200.0);
    portfolio.add_asset("SOL", 100.0, 95.0);

    println!("Portfolio: {}", portfolio.name());
    println!("Base currency: {}", portfolio.base_currency());
    println!("Total value: ${:.2}", portfolio.total_value());

    if let Some(largest) = portfolio.largest_position() {
        println!("Largest position: {} ({} units)",
                 largest.symbol, largest.quantity);
    }

    if let Some(btc) = portfolio.get_asset("BTC") {
        println!("BTC position: {} @ ${:.2}", btc.quantity, btc.avg_price);
    }
}
```

### Парсер торговых данных

```rust
struct TradeRecord<'a> {
    raw_data: &'a str,
}

impl<'a> TradeRecord<'a> {
    fn new(data: &'a str) -> Self {
        TradeRecord { raw_data: data }
    }

    // Elision работает — один входной lifetime от &self
    fn get_field(&self, index: usize) -> Option<&str> {
        self.raw_data.split(',').nth(index)
    }

    fn symbol(&self) -> Option<&str> {
        self.get_field(0)
    }

    fn side(&self) -> Option<&str> {
        self.get_field(1)
    }

    fn price(&self) -> Option<f64> {
        self.get_field(2)?.parse().ok()
    }

    fn quantity(&self) -> Option<f64> {
        self.get_field(3)?.parse().ok()
    }
}

fn main() {
    let csv_line = "BTC/USDT,BUY,42150.50,0.5";
    let record = TradeRecord::new(csv_line);

    println!("Symbol: {:?}", record.symbol());
    println!("Side: {:?}", record.side());
    println!("Price: {:?}", record.price());
    println!("Quantity: {:?}", record.quantity());
}
```

### Кэш рыночных данных

```rust
use std::collections::HashMap;

struct MarketCache {
    prices: HashMap<String, f64>,
    volumes: HashMap<String, f64>,
}

impl MarketCache {
    fn new() -> Self {
        MarketCache {
            prices: HashMap::new(),
            volumes: HashMap::new(),
        }
    }

    fn update_price(&mut self, symbol: &str, price: f64) {
        self.prices.insert(symbol.to_string(), price);
    }

    fn update_volume(&mut self, symbol: &str, volume: f64) {
        self.volumes.insert(symbol.to_string(), volume);
    }

    // Elision: &self -> возвращаемая ссылка
    fn get_price(&self, symbol: &str) -> Option<&f64> {
        self.prices.get(symbol)
    }

    fn get_volume(&self, symbol: &str) -> Option<&f64> {
        self.volumes.get(symbol)
    }

    fn symbols(&self) -> Vec<&String> {
        self.prices.keys().collect()
    }
}

fn main() {
    let mut cache = MarketCache::new();

    cache.update_price("BTC/USDT", 42000.0);
    cache.update_price("ETH/USDT", 2200.0);
    cache.update_volume("BTC/USDT", 1500000.0);
    cache.update_volume("ETH/USDT", 800000.0);

    println!("Cached symbols: {:?}", cache.symbols());

    if let Some(btc_price) = cache.get_price("BTC/USDT") {
        println!("BTC price: ${}", btc_price);
    }

    if let Some(btc_vol) = cache.get_volume("BTC/USDT") {
        println!("BTC volume: ${}", btc_vol);
    }
}
```

## Elision в замыканиях

Замыкания тоже используют elision:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // Elision в замыкании
    let above_threshold: Vec<&f64> = prices.iter()
        .filter(|p| **p > 42000.0)
        .collect();

    println!("Prices above 42000: {:?}", above_threshold);

    // Более сложный пример
    let trades = vec![
        ("BTC", 0.5, 42000.0),
        ("ETH", 5.0, 2200.0),
        ("SOL", 100.0, 95.0),
    ];

    let large_positions: Vec<_> = trades.iter()
        .filter(|(_, qty, price)| qty * price > 5000.0)
        .collect();

    println!("Large positions: {:?}", large_positions);
}
```

## Шпаргалка по Lifetime Elision

| Ситуация | Elision работает? | Пример |
|----------|-------------------|--------|
| Один вход, без выхода ссылки | Да | `fn log(msg: &str)` |
| Один вход, выход ссылка | Да | `fn first(s: &str) -> &str` |
| Метод с &self, выход ссылка | Да | `fn name(&self) -> &str` |
| Два входа, выход ссылка | Нет | `fn pick<'a>(a: &'a T, b: &'a T) -> &'a T` |
| Нет входных ссылок, выход ссылка | Нет | Нужен 'static или другой источник |

## Что мы узнали

1. **Lifetime elision** — автоматический вывод времён жизни компилятором
2. **Три правила**: каждый вход получает свой lifetime → один вход = один выход → &self определяет выход
3. **Когда не работает**: несколько входных ссылок с возвратом ссылки
4. **На практике**: большинство функций не требуют явных аннотаций

## Упражнения

1. Определи, где нужны явные аннотации lifetime:
```rust
fn get_symbol(trade: &Trade) -> &str { ... }
fn longer(a: &str, b: &str) -> &str { ... }
fn process(data: &Data) { ... }
fn best_of_two(x: &f64, y: &f64) -> &f64 { ... }
```

2. Исправь ошибку компиляции:
```rust
fn get_max_price(prices: &[f64], threshold: &f64) -> &f64 {
    prices.iter()
        .filter(|p| *p > threshold)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(threshold)
}
```

3. Напиши структуру `TradeAnalyzer` с методами, использующими elision.

## Домашнее задание

1. Создай структуру `MarketDataProvider` с методами:
   - `get_last_price(&self, symbol: &str) -> Option<&f64>`
   - `get_symbol_info(&self, symbol: &str) -> Option<&SymbolInfo>`
   - `all_symbols(&self) -> Vec<&String>`

2. Реализуй функции для работы с торговой историей:
   - `find_trade_by_id(trades: &[Trade], id: &str) -> Option<&Trade>` (elision работает)
   - `find_best_trade<'a>(t1: &'a Trade, t2: &'a Trade) -> &'a Trade` (нужна явная аннотация)

3. Создай парсер конфигурации с методами, возвращающими ссылки на внутренние строки.

4. Напиши unit-тесты, проверяющие, что время жизни ссылок корректно.

## Навигация

[← Предыдущий день](../044-lifetimes-and-functions/ru.md) | [Следующий день →](../046-static-lifetime/ru.md)
