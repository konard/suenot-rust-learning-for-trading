# День 83: Методы HashMap — insert, get, remove

## Аналогия из трейдинга

Представь, что твой портфель — это таблица с активами. Ты можешь:
- **insert** — добавить новый актив в портфель или обновить количество существующего
- **get** — посмотреть, сколько у тебя конкретного актива
- **remove** — полностью продать актив и убрать его из портфеля

Это три базовые операции, которые ты делаешь каждый день как трейдер!

## Метод insert — добавление или обновление

### Базовое использование

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Добавляем активы в портфель
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("SOL"), 100.0);

    println!("Портфель: {:?}", portfolio);
}
```

### insert возвращает старое значение

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Первая покупка BTC
    let old_value = portfolio.insert(String::from("BTC"), 0.5);
    println!("Предыдущее значение BTC: {:?}", old_value); // None

    // Докупаем BTC — insert перезаписывает!
    let old_value = portfolio.insert(String::from("BTC"), 1.5);
    println!("Предыдущее значение BTC: {:?}", old_value); // Some(0.5)

    println!("Текущее количество BTC: {:?}", portfolio.get("BTC")); // Some(1.5)
}
```

**Важно:** `insert` полностью заменяет значение, а не добавляет к нему!

### Правильное накопление позиции

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Функция для покупки актива
    fn buy_asset(portfolio: &mut HashMap<String, f64>, asset: &str, quantity: f64) {
        let current = portfolio.get(asset).copied().unwrap_or(0.0);
        portfolio.insert(String::from(asset), current + quantity);
    }

    buy_asset(&mut portfolio, "BTC", 0.5);
    buy_asset(&mut portfolio, "BTC", 0.3);
    buy_asset(&mut portfolio, "ETH", 5.0);

    println!("BTC: {}", portfolio.get("BTC").unwrap()); // 0.8
    println!("ETH: {}", portfolio.get("ETH").unwrap()); // 5.0
}
```

## Метод get — получение значения

### Базовое использование

```rust
use std::collections::HashMap;

fn main() {
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2200.0);

    // get возвращает Option<&V>
    let btc_price = prices.get("BTC");
    println!("Цена BTC: {:?}", btc_price); // Some(42000.0)

    let unknown = prices.get("UNKNOWN");
    println!("Неизвестный актив: {:?}", unknown); // None
}
```

### Обработка Option

```rust
use std::collections::HashMap;

fn main() {
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);

    // Способ 1: match
    match prices.get("BTC") {
        Some(price) => println!("BTC стоит ${}", price),
        None => println!("BTC не найден"),
    }

    // Способ 2: if let
    if let Some(price) = prices.get("ETH") {
        println!("ETH стоит ${}", price);
    } else {
        println!("ETH не найден в списке цен");
    }

    // Способ 3: unwrap_or
    let sol_price = prices.get("SOL").unwrap_or(&0.0);
    println!("SOL стоит ${}", sol_price);

    // Способ 4: copied() + unwrap_or для копирования значения
    let ada_price: f64 = prices.get("ADA").copied().unwrap_or(0.0);
    println!("ADA стоит ${}", ada_price);
}
```

### get vs get_mut

```rust
use std::collections::HashMap;

fn main() {
    let mut balances: HashMap<String, f64> = HashMap::new();
    balances.insert(String::from("USD"), 10000.0);
    balances.insert(String::from("BTC"), 0.5);

    // get — только для чтения
    let usd = balances.get("USD");
    println!("USD баланс: {:?}", usd);

    // get_mut — для изменения значения
    if let Some(btc_balance) = balances.get_mut("BTC") {
        *btc_balance += 0.1; // Докупили BTC
    }

    println!("BTC после покупки: {:?}", balances.get("BTC")); // Some(0.6)
}
```

## Метод remove — удаление элемента

### Базовое использование

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("DOGE"), 1000.0);

    println!("До продажи: {:?}", portfolio);

    // Продаём весь DOGE
    let removed = portfolio.remove("DOGE");
    println!("Продано DOGE: {:?}", removed); // Some(1000.0)

    println!("После продажи: {:?}", portfolio);

    // Попытка удалить несуществующий актив
    let not_found = portfolio.remove("SHIB");
    println!("SHIB не найден: {:?}", not_found); // None
}
```

### Полная продажа актива

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);

    fn sell_all(portfolio: &mut HashMap<String, f64>, asset: &str) -> Option<f64> {
        portfolio.remove(asset)
    }

    match sell_all(&mut portfolio, "ETH") {
        Some(quantity) => println!("Продано {} ETH", quantity),
        None => println!("ETH нет в портфеле"),
    }

    println!("Портфель после продажи: {:?}", portfolio);
}
```

## Практический пример: Система управления портфелем

```rust
use std::collections::HashMap;

struct Portfolio {
    holdings: HashMap<String, f64>,
    prices: HashMap<String, f64>,
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            holdings: HashMap::new(),
            prices: HashMap::new(),
        }
    }

    fn update_price(&mut self, asset: &str, price: f64) {
        self.prices.insert(String::from(asset), price);
    }

    fn buy(&mut self, asset: &str, quantity: f64) {
        let current = self.holdings.get(asset).copied().unwrap_or(0.0);
        self.holdings.insert(String::from(asset), current + quantity);
        println!("Куплено {} {}", quantity, asset);
    }

    fn sell(&mut self, asset: &str, quantity: f64) -> Result<f64, String> {
        match self.holdings.get_mut(asset) {
            Some(holding) if *holding >= quantity => {
                *holding -= quantity;
                if *holding == 0.0 {
                    self.holdings.remove(asset);
                }
                Ok(quantity)
            }
            Some(holding) => Err(format!(
                "Недостаточно {}: есть {}, нужно {}",
                asset, holding, quantity
            )),
            None => Err(format!("{} нет в портфеле", asset)),
        }
    }

    fn get_holding(&self, asset: &str) -> f64 {
        self.holdings.get(asset).copied().unwrap_or(0.0)
    }

    fn get_value(&self, asset: &str) -> f64 {
        let quantity = self.get_holding(asset);
        let price = self.prices.get(asset).copied().unwrap_or(0.0);
        quantity * price
    }

    fn total_value(&self) -> f64 {
        self.holdings.iter().map(|(asset, qty)| {
            let price = self.prices.get(asset).copied().unwrap_or(0.0);
            qty * price
        }).sum()
    }

    fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║           ПОРТФЕЛЬ                    ║");
        println!("╠═══════════════════════════════════════╣");

        for (asset, quantity) in &self.holdings {
            let price = self.prices.get(asset).copied().unwrap_or(0.0);
            let value = quantity * price;
            println!("║ {:6} {:>10.4} @ ${:>10.2} = ${:>10.2} ║",
                     asset, quantity, price, value);
        }

        println!("╠═══════════════════════════════════════╣");
        println!("║ ИТОГО:                     ${:>10.2} ║", self.total_value());
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut portfolio = Portfolio::new();

    // Обновляем цены
    portfolio.update_price("BTC", 42000.0);
    portfolio.update_price("ETH", 2200.0);
    portfolio.update_price("SOL", 95.0);

    // Покупаем активы
    portfolio.buy("BTC", 0.5);
    portfolio.buy("ETH", 5.0);
    portfolio.buy("SOL", 50.0);
    portfolio.buy("BTC", 0.3); // Докупаем BTC

    portfolio.print_summary();

    // Продаём часть ETH
    match portfolio.sell("ETH", 2.0) {
        Ok(qty) => println!("\nУспешно продано {} ETH", qty),
        Err(e) => println!("\nОшибка: {}", e),
    }

    portfolio.print_summary();
}
```

## Практический пример: Книга ордеров

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    side: String,
}

struct OrderBook {
    orders: HashMap<u64, Order>,
    next_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: HashMap::new(),
            next_id: 1,
        }
    }

    fn place_order(&mut self, price: f64, quantity: f64, side: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let order = Order {
            id,
            price,
            quantity,
            side: String::from(side),
        };

        self.orders.insert(id, order);
        println!("Создан ордер #{}: {} {} @ ${}", id, side, quantity, price);
        id
    }

    fn get_order(&self, id: u64) -> Option<&Order> {
        self.orders.get(&id)
    }

    fn cancel_order(&mut self, id: u64) -> Option<Order> {
        let removed = self.orders.remove(&id);
        if removed.is_some() {
            println!("Ордер #{} отменён", id);
        }
        removed
    }

    fn modify_order(&mut self, id: u64, new_price: f64, new_quantity: f64) -> bool {
        if let Some(order) = self.orders.get_mut(&id) {
            order.price = new_price;
            order.quantity = new_quantity;
            println!("Ордер #{} изменён: {} @ ${}", id, new_quantity, new_price);
            true
        } else {
            false
        }
    }

    fn print_orders(&self) {
        println!("\n=== Активные ордера ===");
        for (id, order) in &self.orders {
            println!("#{}: {} {} @ ${}", id, order.side, order.quantity, order.price);
        }
        println!("Всего ордеров: {}", self.orders.len());
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Размещаем ордера
    let order1 = book.place_order(42000.0, 0.5, "BUY");
    let order2 = book.place_order(42500.0, 0.3, "SELL");
    let order3 = book.place_order(41500.0, 1.0, "BUY");

    book.print_orders();

    // Проверяем ордер
    if let Some(order) = book.get_order(order1) {
        println!("\nОрдер #{}: {:?}", order1, order);
    }

    // Изменяем ордер
    book.modify_order(order2, 42800.0, 0.5);

    // Отменяем ордер
    book.cancel_order(order3);

    book.print_orders();
}
```

## Полезные методы HashMap

```rust
use std::collections::HashMap;

fn main() {
    let mut data: HashMap<String, f64> = HashMap::new();
    data.insert(String::from("BTC"), 42000.0);
    data.insert(String::from("ETH"), 2200.0);
    data.insert(String::from("SOL"), 95.0);

    // contains_key — проверка наличия ключа
    if data.contains_key("BTC") {
        println!("BTC есть в данных");
    }

    // len — количество элементов
    println!("Количество активов: {}", data.len());

    // is_empty — проверка на пустоту
    println!("Пустой: {}", data.is_empty());

    // keys — итератор по ключам
    print!("Активы: ");
    for key in data.keys() {
        print!("{} ", key);
    }
    println!();

    // values — итератор по значениям
    let total: f64 = data.values().sum();
    println!("Сумма всех цен: ${}", total);

    // iter — итератор по парам
    for (asset, price) in data.iter() {
        println!("{}: ${}", asset, price);
    }

    // clear — очистка
    data.clear();
    println!("После очистки: {} элементов", data.len());
}
```

## Что мы узнали

| Метод | Возвращает | Описание |
|-------|-----------|----------|
| `insert(k, v)` | `Option<V>` | Вставляет, возвращает старое значение |
| `get(&k)` | `Option<&V>` | Ссылка на значение (только чтение) |
| `get_mut(&k)` | `Option<&mut V>` | Мутабельная ссылка (для изменения) |
| `remove(&k)` | `Option<V>` | Удаляет и возвращает значение |
| `contains_key(&k)` | `bool` | Проверка наличия ключа |
| `len()` | `usize` | Количество элементов |

## Упражнения

1. **Трекер цен:** Создай структуру, которая хранит историю цен для каждого актива и позволяет получать последнюю цену

2. **Кэш баланса:** Реализуй кэш балансов для биржи с методами `deposit`, `withdraw`, `get_balance`

3. **Счётчик сделок:** Создай систему, которая считает количество сделок по каждому активу

## Домашнее задание

1. Напиши функцию `merge_portfolios(p1: &HashMap<String, f64>, p2: &HashMap<String, f64>) -> HashMap<String, f64>`, которая объединяет два портфеля

2. Создай структуру `RiskManager`, которая отслеживает позиции и не позволяет превысить лимит на один актив

3. Реализуй `OrderMatcher`, который сопоставляет ордера на покупку и продажу по цене

4. Напиши функцию поиска арбитражных возможностей между двумя HashMap с ценами на разных биржах

## Навигация

[← Предыдущий день](../082-hashmap-portfolio-asset/ru.md) | [Следующий день →](../084-entry-api-update-insert/ru.md)
