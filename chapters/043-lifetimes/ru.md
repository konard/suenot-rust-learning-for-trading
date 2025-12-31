# День 43: Lifetimes — как долго живёт ордер?

## Аналогия из трейдинга

Представь биржу как город с ордерами-жителями. Каждый ордер имеет **срок жизни** — от момента создания до исполнения или отмены. Лимитный ордер может жить секунды (скальпинг) или дни (свинг-трейдинг). GTC (Good Till Cancelled) живёт пока его не отменят, а IOC (Immediate Or Cancel) умирает мгновенно, если не исполнился.

В Rust **lifetimes** — это способ компилятора отслеживать, как долго ссылки остаются валидными. Как трейдер должен убедиться, что ордер ещё активен перед его модификацией, так Rust проверяет, что ссылка указывает на живые данные.

## Проблема: dangling references

```rust
fn main() {
    let order_ref;

    {
        let order = String::from("BUY BTC 42000");
        order_ref = &order;  // order_ref заимствует order
    }  // order уничтожается здесь!

    // println!("{}", order_ref);  // ОШИБКА! order уже не существует
}
```

Это как пытаться отменить ордер, который уже исполнен — его больше нет в системе.

## Базовый синтаксис lifetimes

```rust
fn main() {
    let ticker = String::from("BTCUSDT");
    let result = get_first_char(&ticker);
    println!("Первый символ: {}", result);
}

// 'a — это параметр времени жизни
// Говорит: возвращаемая ссылка живёт столько же, сколько входная
fn get_first_char<'a>(s: &'a str) -> &'a str {
    &s[0..1]
}
```

**'a** (произносится "лайфтайм а") — это аннотация, связывающая время жизни входных и выходных ссылок.

## Lifetimes в структурах

```rust
// Структура, содержащая ссылки, должна иметь lifetime параметры
struct OrderBook<'a> {
    symbol: &'a str,
    best_bid: f64,
    best_ask: f64,
}

struct TradeContext<'a> {
    account_id: &'a str,
    active_orders: &'a [Order],
    market_data: &'a MarketData,
}

#[derive(Debug)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
}

struct MarketData {
    last_price: f64,
    volume: f64,
}

fn main() {
    let symbol = String::from("ETHUSDT");

    let book = OrderBook {
        symbol: &symbol,
        best_bid: 2350.00,
        best_ask: 2350.50,
    };

    println!("Книга ордеров {}: bid={}, ask={}",
             book.symbol, book.best_bid, book.best_ask);
}
```

## Несколько lifetimes

```rust
fn main() {
    let ticker1 = String::from("BTCUSDT");
    let ticker2 = String::from("ETHUSDT");

    let longer = longest_ticker(&ticker1, &ticker2);
    println!("Более длинный тикер: {}", longer);
}

// Обе ссылки должны жить как минимум 'a
fn longest_ticker<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### Разные lifetimes для разных параметров

```rust
fn main() {
    let market = String::from("BINANCE");

    {
        let symbol = String::from("BTCUSDT");
        let result = format_ticker(&market, &symbol);
        println!("{}", result);
    }  // symbol умирает, но market продолжает жить

    println!("Рынок {} всё ещё доступен", market);
}

// Разные lifetimes для разных параметров
fn format_ticker<'a, 'b>(market: &'a str, symbol: &'b str) -> String {
    format!("{}:{}", market, symbol)
}
```

## Elision — когда lifetime'ы можно опустить

Rust автоматически выводит lifetimes в простых случаях:

```rust
// Эти объявления эквивалентны:

// С явным lifetime
fn get_symbol_explicit<'a>(ticker: &'a str) -> &'a str {
    ticker
}

// Lifetime опущен (elision)
fn get_symbol(ticker: &str) -> &str {
    ticker
}

// Правила elision:
// 1. Каждая ссылка-параметр получает свой lifetime
// 2. Если один входной lifetime — он применяется к выходу
// 3. Если есть &self или &mut self — его lifetime применяется к выходу
```

## 'static — вечный lifetime

```rust
// 'static значит: данные живут всё время работы программы
const EXCHANGE: &'static str = "BINANCE";
static DEFAULT_SYMBOL: &'static str = "BTCUSDT";

fn main() {
    let s: &'static str = "Это строковый литерал — всегда 'static";

    println!("Биржа: {}", EXCHANGE);
    println!("Символ по умолчанию: {}", DEFAULT_SYMBOL);
}

// Функция, возвращающая 'static ссылку
fn get_default_market() -> &'static str {
    "SPOT"
}
```

## Практический пример: система управления ордерами

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
}

// OrderManager хранит ссылку на список ордеров
struct OrderManager<'a> {
    orders: &'a mut Vec<Order>,
    max_orders: usize,
}

impl<'a> OrderManager<'a> {
    fn new(orders: &'a mut Vec<Order>, max_orders: usize) -> Self {
        OrderManager { orders, max_orders }
    }

    // Возвращаемая ссылка живёт столько же, сколько self
    fn find_order(&self, id: u64) -> Option<&Order> {
        self.orders.iter().find(|o| o.id == id)
    }

    fn find_order_mut(&mut self, id: u64) -> Option<&mut Order> {
        self.orders.iter_mut().find(|o| o.id == id)
    }

    fn get_active_orders(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| o.status == OrderStatus::New || o.status == OrderStatus::PartiallyFilled)
            .collect()
    }

    fn cancel_order(&mut self, id: u64) -> Result<(), &'static str> {
        match self.find_order_mut(id) {
            Some(order) => {
                if order.status == OrderStatus::Filled {
                    Err("Cannot cancel filled order")
                } else {
                    order.status = OrderStatus::Cancelled;
                    Ok(())
                }
            }
            None => Err("Order not found"),
        }
    }

    fn add_order(&mut self, order: Order) -> Result<(), &'static str> {
        if self.orders.len() >= self.max_orders {
            return Err("Maximum orders reached");
        }
        self.orders.push(order);
        Ok(())
    }
}

fn main() {
    let mut orders = vec![
        Order {
            id: 1,
            symbol: String::from("BTCUSDT"),
            side: OrderSide::Buy,
            price: 42000.0,
            quantity: 0.5,
            status: OrderStatus::New,
        },
        Order {
            id: 2,
            symbol: String::from("ETHUSDT"),
            side: OrderSide::Sell,
            price: 2400.0,
            quantity: 10.0,
            status: OrderStatus::PartiallyFilled,
        },
    ];

    let mut manager = OrderManager::new(&mut orders, 100);

    // Найти ордер
    if let Some(order) = manager.find_order(1) {
        println!("Найден ордер: {:?}", order);
    }

    // Получить активные ордера
    let active = manager.get_active_orders();
    println!("Активных ордеров: {}", active.len());

    // Отменить ордер
    match manager.cancel_order(1) {
        Ok(()) => println!("Ордер отменён"),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Lifetime bounds в generics

```rust
use std::fmt::Display;

// T должен жить как минимум 'a
fn print_with_context<'a, T: Display + 'a>(value: &'a T, context: &str) {
    println!("[{}] {}", context, value);
}

// Структура с generic типом и lifetime
struct PriceCache<'a, T> {
    prices: &'a [T],
    last_update: u64,
}

impl<'a, T: Display + Copy> PriceCache<'a, T> {
    fn new(prices: &'a [T]) -> Self {
        PriceCache {
            prices,
            last_update: 0,
        }
    }

    fn get_latest(&self) -> Option<&T> {
        self.prices.last()
    }

    fn print_all(&self) {
        for price in self.prices {
            println!("Price: {}", price);
        }
    }
}

fn main() {
    let prices = [42000.0, 42100.0, 42050.0, 42200.0];
    let cache = PriceCache::new(&prices);

    if let Some(latest) = cache.get_latest() {
        println!("Последняя цена: {}", latest);
    }

    cache.print_all();
}
```

## Анализатор рыночных данных с lifetimes

```rust
#[derive(Debug)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct MarketAnalyzer<'a> {
    candles: &'a [Candle],
    symbol: &'a str,
}

impl<'a> MarketAnalyzer<'a> {
    fn new(candles: &'a [Candle], symbol: &'a str) -> Self {
        MarketAnalyzer { candles, symbol }
    }

    // Находит свечу с максимальным объёмом
    fn find_highest_volume(&self) -> Option<&Candle> {
        self.candles
            .iter()
            .max_by(|a, b| a.volume.partial_cmp(&b.volume).unwrap())
    }

    // Находит свечи выше заданной цены
    fn find_above_price(&self, price: f64) -> Vec<&Candle> {
        self.candles
            .iter()
            .filter(|c| c.close > price)
            .collect()
    }

    // Рассчитывает SMA для последних N свечей
    fn calculate_sma(&self, period: usize) -> Option<f64> {
        if self.candles.len() < period {
            return None;
        }

        let sum: f64 = self.candles
            .iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }

    // Определяет тренд
    fn get_trend(&self) -> &'static str {
        if self.candles.len() < 2 {
            return "UNDEFINED";
        }

        let first = self.candles.first().unwrap().close;
        let last = self.candles.last().unwrap().close;

        if last > first * 1.01 {
            "UPTREND"
        } else if last < first * 0.99 {
            "DOWNTREND"
        } else {
            "SIDEWAYS"
        }
    }
}

fn main() {
    let candles = vec![
        Candle { timestamp: 1, open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 1000.0 },
        Candle { timestamp: 2, open: 42300.0, high: 42800.0, low: 42200.0, close: 42700.0, volume: 1500.0 },
        Candle { timestamp: 3, open: 42700.0, high: 43000.0, low: 42500.0, close: 42900.0, volume: 1200.0 },
        Candle { timestamp: 4, open: 42900.0, high: 43200.0, low: 42800.0, close: 43100.0, volume: 2000.0 },
    ];

    let symbol = "BTCUSDT";
    let analyzer = MarketAnalyzer::new(&candles, symbol);

    println!("=== Анализ {} ===", symbol);

    if let Some(candle) = analyzer.find_highest_volume() {
        println!("Свеча с макс. объёмом: close={}, volume={}",
                 candle.close, candle.volume);
    }

    if let Some(sma) = analyzer.calculate_sma(3) {
        println!("SMA(3): {:.2}", sma);
    }

    println!("Тренд: {}", analyzer.get_trend());

    let above_42500 = analyzer.find_above_price(42500.0);
    println!("Свечей выше 42500: {}", above_42500.len());
}
```

## Распространённые ошибки и решения

### Ошибка 1: возврат ссылки на локальные данные

```rust
// НЕПРАВИЛЬНО — не компилируется
// fn create_order_ref() -> &Order {
//     let order = Order { ... };
//     &order  // order умрёт при выходе из функции!
// }

// ПРАВИЛЬНО — возвращаем владение
fn create_order() -> Order {
    Order {
        id: 1,
        symbol: String::from("BTCUSDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 1.0,
        status: OrderStatus::New,
    }
}
```

### Ошибка 2: конфликтующие lifetimes

```rust
fn main() {
    let ticker1 = String::from("BTCUSDT");
    let result;

    {
        let ticker2 = String::from("ETHUSDT");
        // result = longest_ticker(&ticker1, &ticker2);
        // ОШИБКА! result не может пережить ticker2
    }

    // Решение: убедиться, что обе ссылки живут достаточно долго
    let ticker2 = String::from("ETHUSDT");
    result = longest_ticker(&ticker1, &ticker2);
    println!("{}", result);
}

fn longest_ticker<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

## Упражнения

### Упражнение 1: Парсер торговых сигналов

```rust
// Реализуй структуру SignalParser, которая хранит ссылку на строку сигнала
// и предоставляет методы для извлечения частей сигнала

struct SignalParser<'a> {
    signal: &'a str,
}

impl<'a> SignalParser<'a> {
    fn new(signal: &'a str) -> Self {
        // Реализуй
        todo!()
    }

    // Извлекает действие (BUY/SELL) из начала строки
    fn get_action(&self) -> &str {
        // Реализуй
        todo!()
    }

    // Извлекает символ (например, BTCUSDT)
    fn get_symbol(&self) -> &str {
        // Реализуй
        todo!()
    }
}

fn main() {
    let signal = "BUY BTCUSDT 42000";
    let parser = SignalParser::new(signal);

    println!("Action: {}", parser.get_action());
    println!("Symbol: {}", parser.get_symbol());
}
```

### Упражнение 2: Кэш котировок

```rust
// Создай структуру QuoteCache, хранящую ссылки на bid и ask цены
// Реализуй методы для расчёта спреда и mid-price

struct QuoteCache<'a> {
    bids: &'a [f64],
    asks: &'a [f64],
}

impl<'a> QuoteCache<'a> {
    fn new(bids: &'a [f64], asks: &'a [f64]) -> Self {
        // Реализуй
        todo!()
    }

    fn best_bid(&self) -> Option<f64> {
        // Реализуй: вернуть максимальный bid
        todo!()
    }

    fn best_ask(&self) -> Option<f64> {
        // Реализуй: вернуть минимальный ask
        todo!()
    }

    fn spread(&self) -> Option<f64> {
        // Реализуй: best_ask - best_bid
        todo!()
    }

    fn mid_price(&self) -> Option<f64> {
        // Реализуй: (best_bid + best_ask) / 2
        todo!()
    }
}
```

### Упражнение 3: Журнал сделок

```rust
// Создай структуру TradeLog с методом, возвращающим
// ссылку на последнюю прибыльную сделку

#[derive(Debug)]
struct Trade {
    id: u64,
    pnl: f64,
}

struct TradeLog<'a> {
    trades: &'a [Trade],
}

impl<'a> TradeLog<'a> {
    fn new(trades: &'a [Trade]) -> Self {
        // Реализуй
        todo!()
    }

    // Возвращает ссылку на последнюю прибыльную сделку
    fn last_profitable(&self) -> Option<&Trade> {
        // Реализуй
        todo!()
    }

    // Возвращает все прибыльные сделки
    fn all_profitable(&self) -> Vec<&Trade> {
        // Реализуй
        todo!()
    }
}
```

### Упражнение 4: Фильтр позиций

```rust
// Создай функцию, которая принимает slice позиций и возвращает
// только открытые позиции

#[derive(Debug)]
struct Position {
    symbol: String,
    size: f64,
    is_open: bool,
}

// Реализуй функцию с правильными lifetime аннотациями
fn filter_open_positions(/* параметры */) -> Vec<&Position> {
    // Реализуй
    todo!()
}
```

## Что мы узнали

| Концепция | Синтаксис | Назначение |
|-----------|-----------|------------|
| Lifetime параметр | `'a` | Связывает время жизни ссылок |
| Lifetime в функции | `fn foo<'a>(x: &'a T) -> &'a T` | Связь входа и выхода |
| Lifetime в структуре | `struct Foo<'a> { x: &'a T }` | Структура с ссылками |
| Static lifetime | `'static` | Живёт всё время программы |
| Lifetime bounds | `T: 'a` | T должен жить как минимум 'a |
| Elision | Опущение `'a` | Автоматический вывод |

## Домашнее задание

1. **Система мониторинга ордеров**: Создай структуру `OrderMonitor<'a>`, которая хранит ссылку на список ордеров и предоставляет методы для:
   - Поиска ордеров по символу
   - Подсчёта общего объёма по стороне (buy/sell)
   - Нахождения ордера с максимальной ценой

2. **Анализатор стакана**: Реализуй `OrderBookAnalyzer<'a>` с методами:
   - `imbalance()` — отношение объёма bid к ask
   - `spread_percent()` — спред в процентах
   - `depth_at_level(n)` — суммарный объём на первых n уровнях

3. **Портфельный трекер**: Создай `PortfolioView<'a>`, который:
   - Принимает ссылку на список позиций
   - Рассчитывает total PnL
   - Находит самую прибыльную/убыточную позицию
   - Возвращает позиции по заданному символу

4. **Валидатор сигналов**: Напиши функцию `validate_signals<'a>(signals: &'a [Signal], rules: &Rules) -> Vec<&'a Signal>`, которая возвращает только валидные сигналы.

## Навигация

[← Предыдущий день](../042-borrowing-data-access/ru.md) | [Следующий день →](../044-lifetime-elision/ru.md)
