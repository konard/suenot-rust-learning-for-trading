# День 85: HashSet — уникальные тикеры в портфеле

## Аналогия из трейдинга

В портфеле трейдера каждый актив встречается **только один раз**:
- Нельзя добавить AAPL дважды — либо он есть, либо нет
- Список отслеживаемых тикеров не должен дублироваться
- Уникальные биржи, на которых мы торгуем

`HashSet` — это коллекция **уникальных** значений. Как список тикеров в watchlist: каждый тикер либо в списке, либо нет.

## Создание HashSet

```rust
use std::collections::HashSet;

fn main() {
    // Пустой HashSet
    let mut watchlist: HashSet<String> = HashSet::new();

    // HashSet с начальными значениями
    let exchanges: HashSet<&str> = HashSet::from(["Binance", "Coinbase", "Kraken"]);

    // Создание через collect()
    let tickers: HashSet<&str> = vec!["BTC", "ETH", "BTC", "SOL", "ETH"]
        .into_iter()
        .collect();

    println!("Watchlist: {:?}", watchlist);
    println!("Exchanges: {:?}", exchanges);
    println!("Unique tickers: {:?}", tickers);  // BTC и ETH только по одному разу!
}
```

## Добавление элементов: insert()

```rust
use std::collections::HashSet;

fn main() {
    let mut portfolio_tickers: HashSet<String> = HashSet::new();

    // insert() возвращает bool: true если элемент был добавлен
    let added = portfolio_tickers.insert("BTC".to_string());
    println!("BTC added: {}", added);  // true

    portfolio_tickers.insert("ETH".to_string());
    portfolio_tickers.insert("SOL".to_string());

    // Попытка добавить дубликат
    let duplicate = portfolio_tickers.insert("BTC".to_string());
    println!("BTC added again: {}", duplicate);  // false — уже есть!

    println!("Portfolio: {:?}", portfolio_tickers);
    println!("Unique assets: {}", portfolio_tickers.len());  // 3
}
```

## Проверка наличия: contains()

```rust
use std::collections::HashSet;

fn main() {
    let allowed_pairs: HashSet<&str> = HashSet::from([
        "BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT"
    ]);

    // Проверяем разрешённые пары для торговли
    let pair_to_trade = "BTC/USDT";

    if allowed_pairs.contains(pair_to_trade) {
        println!("Trading {} is allowed", pair_to_trade);
    } else {
        println!("Trading {} is NOT allowed", pair_to_trade);
    }

    // Проверка нескольких пар
    let orders = vec!["BTC/USDT", "DOGE/USDT", "ETH/USDT"];

    for pair in orders {
        if allowed_pairs.contains(pair) {
            println!("  [OK] {} - order accepted", pair);
        } else {
            println!("  [REJECT] {} - pair not in whitelist", pair);
        }
    }
}
```

## Удаление элементов: remove()

```rust
use std::collections::HashSet;

fn main() {
    let mut active_positions: HashSet<String> = HashSet::from([
        "BTC".to_string(),
        "ETH".to_string(),
        "SOL".to_string(),
    ]);

    println!("Active positions: {:?}", active_positions);

    // Закрываем позицию по ETH
    let closed = active_positions.remove("ETH");
    println!("ETH position closed: {}", closed);  // true

    // Попытка закрыть несуществующую позицию
    let not_found = active_positions.remove("DOGE");
    println!("DOGE position closed: {}", not_found);  // false

    println!("Remaining positions: {:?}", active_positions);
}
```

## Операции над множествами

HashSet поддерживает математические операции над множествами — очень полезно для анализа портфелей.

### Объединение (Union)

```rust
use std::collections::HashSet;

fn main() {
    // Активы на Binance
    let binance: HashSet<&str> = HashSet::from(["BTC", "ETH", "BNB", "SOL"]);

    // Активы на Coinbase
    let coinbase: HashSet<&str> = HashSet::from(["BTC", "ETH", "MATIC", "AVAX"]);

    // Все уникальные активы, которые можем торговать
    let all_assets: HashSet<_> = binance.union(&coinbase).collect();

    println!("Binance: {:?}", binance);
    println!("Coinbase: {:?}", coinbase);
    println!("All tradeable assets: {:?}", all_assets);
}
```

### Пересечение (Intersection)

```rust
use std::collections::HashSet;

fn main() {
    // Топ-10 по объёму на Binance
    let binance_top: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "BNB", "SOL", "XRP"
    ]);

    // Топ-10 по объёму на Coinbase
    let coinbase_top: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "MATIC", "SOL", "DOGE"
    ]);

    // Общие лидеры на обеих биржах — потенциально самые ликвидные
    let common_leaders: HashSet<_> = binance_top.intersection(&coinbase_top).collect();

    println!("Common top assets: {:?}", common_leaders);
    // {"BTC", "ETH", "SOL"}
}
```

### Разность (Difference)

```rust
use std::collections::HashSet;

fn main() {
    // Текущий портфель
    let portfolio: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL", "AVAX"]);

    // Целевой портфель
    let target: HashSet<&str> = HashSet::from(["BTC", "ETH", "MATIC", "DOT"]);

    // Что нужно продать (есть в портфеле, но нет в целевом)
    let to_sell: HashSet<_> = portfolio.difference(&target).collect();

    // Что нужно купить (нет в портфеле, но есть в целевом)
    let to_buy: HashSet<_> = target.difference(&portfolio).collect();

    println!("Current portfolio: {:?}", portfolio);
    println!("Target portfolio: {:?}", target);
    println!("Assets to SELL: {:?}", to_sell);   // {"SOL", "AVAX"}
    println!("Assets to BUY: {:?}", to_buy);     // {"MATIC", "DOT"}
}
```

### Симметрическая разность (Symmetric Difference)

```rust
use std::collections::HashSet;

fn main() {
    let yesterday: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL"]);
    let today: HashSet<&str> = HashSet::from(["BTC", "ETH", "AVAX"]);

    // Активы, которые изменились (были вчера, но нет сегодня + есть сегодня, но не было вчера)
    let changed: HashSet<_> = yesterday.symmetric_difference(&today).collect();

    println!("Portfolio changes: {:?}", changed);  // {"SOL", "AVAX"}
}
```

## Итерация по HashSet

```rust
use std::collections::HashSet;

fn main() {
    let watchlist: HashSet<&str> = HashSet::from([
        "BTC", "ETH", "SOL", "AVAX", "MATIC"
    ]);

    println!("=== Watchlist ===");
    for ticker in &watchlist {
        println!("  Monitoring: {}", ticker);
    }

    // Фильтрация
    let eth_based: Vec<_> = watchlist
        .iter()
        .filter(|t| **t == "ETH" || t.starts_with("ETH"))
        .collect();

    println!("ETH-related: {:?}", eth_based);
}
```

## Практический пример: фильтр дубликатов ордеров

```rust
use std::collections::HashSet;

#[derive(Debug)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    quantity: f64,
}

fn main() {
    // Поток ордеров (могут быть дубликаты id из-за ретраев)
    let incoming_orders = vec![
        Order { id: "ORD001".to_string(), symbol: "BTC".to_string(), side: "BUY".to_string(), quantity: 0.5 },
        Order { id: "ORD002".to_string(), symbol: "ETH".to_string(), side: "BUY".to_string(), quantity: 2.0 },
        Order { id: "ORD001".to_string(), symbol: "BTC".to_string(), side: "BUY".to_string(), quantity: 0.5 }, // Дубликат!
        Order { id: "ORD003".to_string(), symbol: "SOL".to_string(), side: "SELL".to_string(), quantity: 10.0 },
        Order { id: "ORD002".to_string(), symbol: "ETH".to_string(), side: "BUY".to_string(), quantity: 2.0 }, // Дубликат!
    ];

    let mut processed_ids: HashSet<String> = HashSet::new();
    let mut unique_orders: Vec<&Order> = Vec::new();

    for order in &incoming_orders {
        // Если id ещё не обработан
        if processed_ids.insert(order.id.clone()) {
            unique_orders.push(order);
            println!("[PROCESS] Order {}: {} {} {}",
                order.id, order.side, order.quantity, order.symbol);
        } else {
            println!("[SKIP] Duplicate order {}", order.id);
        }
    }

    println!("\nTotal incoming: {}", incoming_orders.len());
    println!("Unique processed: {}", unique_orders.len());
}
```

## Практический пример: анализ активности по тикерам

```rust
use std::collections::HashSet;

fn main() {
    // Сделки за неделю (символы)
    let monday_trades = vec!["BTC", "ETH", "SOL", "BTC", "ETH"];
    let tuesday_trades = vec!["ETH", "AVAX", "ETH", "BTC"];
    let wednesday_trades = vec!["SOL", "MATIC", "DOT", "SOL"];

    // Уникальные тикеры за каждый день
    let monday: HashSet<_> = monday_trades.into_iter().collect();
    let tuesday: HashSet<_> = tuesday_trades.into_iter().collect();
    let wednesday: HashSet<_> = wednesday_trades.into_iter().collect();

    println!("Monday tickers: {:?}", monday);
    println!("Tuesday tickers: {:?}", tuesday);
    println!("Wednesday tickers: {:?}", wednesday);

    // Тикеры, которые торговались каждый день
    let all_days: HashSet<_> = monday
        .intersection(&tuesday)
        .cloned()
        .collect::<HashSet<_>>()
        .intersection(&wednesday)
        .cloned()
        .collect();

    println!("\nTraded every day: {:?}", all_days);

    // Все уникальные тикеры за неделю
    let all_week: HashSet<_> = monday
        .union(&tuesday)
        .cloned()
        .collect::<HashSet<_>>()
        .union(&wednesday)
        .cloned()
        .collect();

    println!("All tickers this week: {:?}", all_week);
    println!("Total unique tickers: {}", all_week.len());
}
```

## Практический пример: whitelist/blacklist

```rust
use std::collections::HashSet;

struct TradingFilter {
    whitelist: HashSet<String>,
    blacklist: HashSet<String>,
}

impl TradingFilter {
    fn new() -> Self {
        TradingFilter {
            whitelist: HashSet::new(),
            blacklist: HashSet::new(),
        }
    }

    fn allow(&mut self, symbol: &str) {
        self.whitelist.insert(symbol.to_string());
        self.blacklist.remove(symbol);
    }

    fn block(&mut self, symbol: &str) {
        self.blacklist.insert(symbol.to_string());
        self.whitelist.remove(symbol);
    }

    fn can_trade(&self, symbol: &str) -> bool {
        // Если есть whitelist — только из него
        // Если нет whitelist — всё кроме blacklist
        if !self.whitelist.is_empty() {
            self.whitelist.contains(symbol)
        } else {
            !self.blacklist.contains(symbol)
        }
    }
}

fn main() {
    let mut filter = TradingFilter::new();

    // Разрешаем только определённые активы
    filter.allow("BTC");
    filter.allow("ETH");
    filter.allow("SOL");

    let orders = vec!["BTC", "DOGE", "ETH", "SHIB", "SOL"];

    println!("=== Trading Filter ===");
    for symbol in orders {
        if filter.can_trade(symbol) {
            println!("  [ALLOW] {} - order accepted", symbol);
        } else {
            println!("  [BLOCK] {} - not in whitelist", symbol);
        }
    }
}
```

## HashSet vs Vec: когда что использовать

```rust
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    // Создаём большой набор тикеров
    let tickers: Vec<String> = (0..10000)
        .map(|i| format!("TICKER{}", i))
        .collect();

    let ticker_set: HashSet<String> = tickers.iter().cloned().collect();

    let search_for = "TICKER9999";

    // Поиск в Vec — O(n)
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = tickers.contains(&search_for.to_string());
    }
    let vec_time = start.elapsed();

    // Поиск в HashSet — O(1)
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = ticker_set.contains(search_for);
    }
    let set_time = start.elapsed();

    println!("Vec search (1000x): {:?}", vec_time);
    println!("HashSet search (1000x): {:?}", set_time);
    println!("HashSet is ~{:.0}x faster",
        vec_time.as_nanos() as f64 / set_time.as_nanos() as f64);
}
```

## Полезные методы

```rust
use std::collections::HashSet;

fn main() {
    let mut set: HashSet<&str> = HashSet::from(["BTC", "ETH", "SOL"]);

    // Размер и проверка пустоты
    println!("Size: {}", set.len());         // 3
    println!("Is empty: {}", set.is_empty()); // false

    // Очистка
    set.clear();
    println!("After clear: {:?}", set);       // {}

    // Создание с заданной ёмкостью
    let big_set: HashSet<String> = HashSet::with_capacity(1000);
    println!("Capacity: {}", big_set.capacity());

    // Проверка подмножества
    let small: HashSet<i32> = HashSet::from([1, 2]);
    let large: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5]);

    println!("small is subset of large: {}", small.is_subset(&large));     // true
    println!("large is superset of small: {}", large.is_superset(&small)); // true
    println!("Are disjoint: {}", small.is_disjoint(&HashSet::from([6, 7]))); // true
}
```

## Что мы узнали

| Метод | Описание |
|-------|----------|
| `HashSet::new()` | Создаёт пустой HashSet |
| `insert(value)` | Добавляет элемент, возвращает `true` если новый |
| `remove(&value)` | Удаляет элемент |
| `contains(&value)` | Проверяет наличие |
| `union(&other)` | Объединение множеств |
| `intersection(&other)` | Пересечение множеств |
| `difference(&other)` | Разность множеств |
| `len()`, `is_empty()` | Размер и проверка пустоты |

## Домашнее задание

1. Создай функцию `find_common_assets`, которая принимает портфели нескольких трейдеров и возвращает активы, которые есть у всех

2. Реализуй систему отслеживания уникальных торговых пар: добавление новых пар, проверка существующих, статистика

3. Напиши функцию ребалансировки портфеля: принимает текущий и целевой портфели, возвращает списки активов для покупки и продажи

4. Создай детектор "новых" активов: сравнивает вчерашний и сегодняшний списки торгуемых активов на бирже, находит новые листинги

## Навигация

[← Предыдущий день](../084-entry-api/ru.md) | [Следующий день →](../086-btreemap-sorted-prices/ru.md)
