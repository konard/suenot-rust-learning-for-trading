# День 44: Аннотации времени жизни — `'a` в функциях

## Аналогия из трейдинга

Представь, что ты работаешь с **торговыми сессиями**. Когда ты анализируешь ордера внутри сессии, эти ордера существуют только пока открыта сессия. Нельзя использовать ордер из закрытой сессии — он уже недействителен.

В Rust **lifetime (время жизни)** — это способ сказать компилятору: "Эта ссылка действительна, пока жив исходный объект". Аннотация `'a` — это метка, которая связывает время жизни ссылок между собой.

## Зачем нужны lifetimes?

Rust гарантирует безопасность памяти без сборщика мусора. Компилятор должен знать, что все ссылки действительны. Когда функция принимает или возвращает ссылки, нужно указать, как они связаны.

```rust
// Эта функция не скомпилируется без lifetime аннотаций!
// fn get_best_price(bid: &f64, ask: &f64) -> &f64 {
//     if bid > ask { bid } else { ask }
// }

// Правильно: связываем lifetimes
fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42100.0;
    let best = get_best_price(&bid, &ask);
    println!("Best price: ${}", best);
}
```

## Синтаксис аннотаций

```rust
// 'a — это имя lifetime (может быть любым: 'b, 'price, 'session)
fn function_name<'a>(param: &'a Type) -> &'a Type {
    // ...
}
```

**Расшифровка:**
- `<'a>` — объявляем lifetime параметр (как обобщённый тип)
- `&'a Type` — ссылка с lifetime `'a`
- Возвращаемая ссылка живёт столько же, сколько входной параметр

## Пример: выбор лучшей цены

```rust
fn main() {
    let btc_price = 42000.0;
    let eth_price = 2800.0;

    let higher = get_higher_price(&btc_price, &eth_price);
    println!("Higher price: ${}", higher);

    // higher всё ещё действителен, пока btc_price и eth_price в области видимости
}

fn get_higher_price<'a>(price1: &'a f64, price2: &'a f64) -> &'a f64 {
    if price1 > price2 {
        price1
    } else {
        price2
    }
}
```

## Разные lifetimes для разных параметров

Иногда нужно различать время жизни разных параметров:

```rust
fn main() {
    let session_name = String::from("NYSE Morning");
    let prices = vec![42000.0, 42100.0, 42050.0];

    let report = create_price_report(&session_name, &prices);
    println!("{}", report);
}

// Здесь 'a и 'b — разные lifetimes
// Возвращаемая строка живёт столько же, сколько session
fn create_price_report<'a, 'b>(session: &'a str, prices: &'b [f64]) -> &'a str {
    println!("Prices in session: {:?}", prices);
    session  // Возвращаем ссылку на session
}
```

## Lifetime в структурах

Если структура содержит ссылки, нужно указать их lifetime:

```rust
// Структура с ссылкой на данные о цене
struct PriceSnapshot<'a> {
    symbol: &'a str,
    price: &'a f64,
    timestamp: u64,
}

fn main() {
    let symbol = String::from("BTC/USDT");
    let price = 42000.0;

    let snapshot = PriceSnapshot {
        symbol: &symbol,
        price: &price,
        timestamp: 1704067200,
    };

    println!("{} @ ${}", snapshot.symbol, snapshot.price);
}
```

## Практический пример: анализ торговых данных

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];
    let volumes = [100.0, 150.0, 80.0, 120.0, 90.0];

    // Найдём OHLC данные
    let (open, high, low, close) = get_ohlc(&prices);
    println!("Open: {}, High: {}, Low: {}, Close: {}", open, high, low, close);

    // Найдём цену с максимальным объёмом
    if let Some((price, volume)) = find_highest_volume_price(&prices, &volumes) {
        println!("Highest volume: {} at price ${}", volume, price);
    }
}

fn get_ohlc<'a>(prices: &'a [f64]) -> (&'a f64, &'a f64, &'a f64, &'a f64) {
    let open = &prices[0];
    let close = &prices[prices.len() - 1];

    let mut high = &prices[0];
    let mut low = &prices[0];

    for price in prices {
        if price > high { high = price; }
        if price < low { low = price; }
    }

    (open, high, low, close)
}

fn find_highest_volume_price<'a>(
    prices: &'a [f64],
    volumes: &'a [f64]
) -> Option<(&'a f64, &'a f64)> {
    if prices.is_empty() || volumes.is_empty() {
        return None;
    }

    let mut max_idx = 0;
    let mut max_vol = volumes[0];

    for (idx, &vol) in volumes.iter().enumerate() {
        if vol > max_vol {
            max_vol = vol;
            max_idx = idx;
        }
    }

    Some((&prices[max_idx], &volumes[max_idx]))
}
```

## Elision rules — когда lifetimes можно опустить

Rust имеет правила "элизии" — когда компилятор сам выводит lifetimes:

```rust
// Правило 1: Один входной параметр — lifetime копируется на выход
fn get_first_price(prices: &[f64]) -> &f64 {
    &prices[0]
}
// Эквивалентно: fn get_first_price<'a>(prices: &'a [f64]) -> &'a f64

// Правило 2: Метод &self — lifetime self копируется на выход
struct Portfolio {
    name: String,
    balance: f64,
}

impl Portfolio {
    fn get_name(&self) -> &str {
        &self.name
    }
    // Эквивалентно: fn get_name<'a>(&'a self) -> &'a str
}

fn main() {
    let prices = [42000.0, 42100.0];
    println!("First: {}", get_first_price(&prices));

    let portfolio = Portfolio {
        name: String::from("Main"),
        balance: 10000.0,
    };
    println!("Portfolio: {}", portfolio.get_name());
}
```

## Статический lifetime `'static`

`'static` означает, что данные живут всё время работы программы:

```rust
fn main() {
    // Строковые литералы имеют 'static lifetime
    let default_symbol: &'static str = "BTC/USDT";

    let symbol = get_default_symbol();
    println!("Default symbol: {}", symbol);
}

fn get_default_symbol() -> &'static str {
    "BTC/USDT"  // Строковый литерал — всегда 'static
}

// Пример с константами
const EXCHANGE_NAME: &str = "Binance";  // Неявно 'static
static MAX_LEVERAGE: f64 = 100.0;       // Статическая переменная
```

## Сложный пример: фильтрация ордеров

```rust
#[derive(Debug)]
struct Order<'a> {
    id: u64,
    symbol: &'a str,
    price: f64,
    quantity: f64,
    side: &'a str,
}

fn main() {
    let btc = "BTC/USDT";
    let eth = "ETH/USDT";
    let buy = "BUY";
    let sell = "SELL";

    let orders = vec![
        Order { id: 1, symbol: btc, price: 42000.0, quantity: 0.5, side: buy },
        Order { id: 2, symbol: eth, price: 2800.0, quantity: 5.0, side: sell },
        Order { id: 3, symbol: btc, price: 42100.0, quantity: 0.3, side: buy },
        Order { id: 4, symbol: btc, price: 41900.0, quantity: 0.2, side: sell },
    ];

    let btc_orders = filter_by_symbol(&orders, btc);
    println!("BTC orders: {:?}", btc_orders);

    let buy_orders = filter_by_side(&orders, buy);
    println!("Buy orders: {:?}", buy_orders);

    if let Some(best) = find_best_bid(&orders, btc) {
        println!("Best BTC bid: ${} for {} units", best.price, best.quantity);
    }
}

fn filter_by_symbol<'a, 'b>(
    orders: &'a [Order<'b>],
    symbol: &str
) -> Vec<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.symbol == symbol)
        .collect()
}

fn filter_by_side<'a, 'b>(
    orders: &'a [Order<'b>],
    side: &str
) -> Vec<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.side == side)
        .collect()
}

fn find_best_bid<'a, 'b>(
    orders: &'a [Order<'b>],
    symbol: &str
) -> Option<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.symbol == symbol && o.side == "BUY")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}
```

## Типичные ошибки и их решения

### Ошибка: возврат ссылки на локальную переменную

```rust
// НЕ СКОМПИЛИРУЕТСЯ!
// fn calculate_and_return() -> &f64 {
//     let result = 42.0;
//     &result  // result уничтожится после выхода из функции!
// }

// Решение: возвращать значение, а не ссылку
fn calculate_and_return() -> f64 {
    let result = 42.0;
    result
}

fn main() {
    let value = calculate_and_return();
    println!("Value: {}", value);
}
```

### Ошибка: несовместимые lifetimes

```rust
// НЕ СКОМПИЛИРУЕТСЯ!
// fn wrong_lifetime<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
//     if x.len() > y.len() { x } else { y }  // y имеет lifetime 'b, не 'a!
// }

// Решение: использовать одинаковый lifetime
fn correct_lifetime<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let x = "Bitcoin";
    let y = "Ethereum";
    println!("Longer: {}", correct_lifetime(x, y));
}
```

## Практические упражнения

### Упражнение 1: Найди минимальную и максимальную цену

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0];

    // Реализуй функцию
    let (min, max) = find_min_max(&prices);
    println!("Min: {}, Max: {}", min, max);
}

// TODO: Добавь lifetime аннотации
fn find_min_max(prices: &[f64]) -> (&f64, &f64) {
    let mut min = &prices[0];
    let mut max = &prices[0];

    for price in prices {
        if price < min { min = price; }
        if price > max { max = price; }
    }

    (min, max)
}
```

### Упражнение 2: Структура с ссылками

```rust
// TODO: Добавь lifetime параметр
struct Trade {
    symbol: &str,
    entry_price: &f64,
    exit_price: &f64,
}

impl Trade {
    // TODO: Добавь lifetime аннотации
    fn pnl(&self) -> f64 {
        self.exit_price - self.entry_price
    }

    fn get_symbol(&self) -> &str {
        self.symbol
    }
}

fn main() {
    let sym = String::from("BTC/USDT");
    let entry = 42000.0;
    let exit = 43500.0;

    let trade = Trade {
        symbol: &sym,
        entry_price: &entry,
        exit_price: &exit,
    };

    println!("{}: PnL = ${}", trade.get_symbol(), trade.pnl());
}
```

### Упражнение 3: Фильтрация с lifetimes

```rust
// TODO: Реализуй функцию filter_profitable_trades
// Должна вернуть вектор ссылок на прибыльные сделки

struct TradeResult<'a> {
    id: u64,
    symbol: &'a str,
    pnl: f64,
}

fn main() {
    let btc = "BTC";
    let eth = "ETH";

    let trades = vec![
        TradeResult { id: 1, symbol: btc, pnl: 500.0 },
        TradeResult { id: 2, symbol: eth, pnl: -200.0 },
        TradeResult { id: 3, symbol: btc, pnl: 300.0 },
        TradeResult { id: 4, symbol: eth, pnl: -50.0 },
    ];

    // TODO: Реализуй эту функцию
    // let profitable = filter_profitable_trades(&trades);
    // println!("Profitable trades: {}", profitable.len());
}
```

## Что мы узнали

| Концепция | Синтаксис | Описание |
|-----------|-----------|----------|
| Lifetime параметр | `<'a>` | Объявление lifetime в сигнатуре |
| Аннотация ссылки | `&'a T` | Ссылка с определённым lifetime |
| Несколько lifetimes | `<'a, 'b>` | Разные lifetime для разных данных |
| Struct lifetime | `struct Foo<'a>` | Структура содержит ссылки |
| Static lifetime | `'static` | Данные живут вечно |
| Elision | (автоматически) | Компилятор выводит lifetime |

## Домашнее задание

1. **Напиши функцию** `get_best_order<'a>(orders: &'a [Order], side: &str) -> Option<&'a Order>` — возвращает лучший ордер (максимальная цена для BUY, минимальная для SELL)

2. **Создай структуру** `TradingSession<'a>` с полями `name: &'a str`, `orders: Vec<&'a Order>`. Добавь методы для работы с сессией

3. **Реализуй функцию** `merge_price_data<'a>(prices1: &'a [f64], prices2: &'a [f64]) -> Vec<&'a f64>` — объединяет две коллекции ссылок на цены

4. **Сложное задание:** создай парсер торгового лога, который возвращает ссылки на найденные символы и цены, не копируя данные

## Навигация

[← Предыдущий день](../043-lifetime-basics/ru.md) | [Следующий день →](../045-lifetime-structs/ru.md)
