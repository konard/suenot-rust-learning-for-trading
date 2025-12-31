# День 97: while let — обрабатываем пока успешно

## Аналогия из трейдинга

Представь, что ты работаешь с очередью ордеров. Пока в очереди есть ордера — ты их обрабатываешь. Как только очередь пуста — останавливаешься. Это точно как `while let`: **продолжай выполнять действие, пока паттерн успешно совпадает**.

Другой пример: ты получаешь котировки от биржи. Пока соединение активно и данные приходят — обрабатываешь. Как только `None` или ошибка — выходишь из цикла.

## Базовый синтаксис while let

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // Извлекаем цены пока вектор не пуст
    while let Some(price) = prices.pop() {
        println!("Обработка цены: ${:.2}", price);
    }

    println!("Все цены обработаны");
}
```

**Важно:** `while let` — это сахар для цикла `loop` с `match`. Цикл продолжается пока паттерн `Some(price)` совпадает.

## Сравнение с loop + match

```rust
fn main() {
    let mut orders = vec![100.0, 200.0, 150.0];

    // Эквивалентный код с loop + match
    loop {
        match orders.pop() {
            Some(order) => println!("Ордер: ${:.2}", order),
            None => break,
        }
    }

    // То же самое с while let — гораздо чище!
    let mut orders2 = vec![100.0, 200.0, 150.0];
    while let Some(order) = orders2.pop() {
        println!("Ордер: ${:.2}", order);
    }
}
```

## Обработка очереди ордеров

```rust
fn main() {
    let mut order_queue: Vec<Order> = vec![
        Order { id: 1, symbol: "BTC", quantity: 0.5, price: 42000.0 },
        Order { id: 2, symbol: "ETH", quantity: 10.0, price: 2200.0 },
        Order { id: 3, symbol: "BTC", quantity: 0.25, price: 42100.0 },
    ];

    println!("Начинаем обработку очереди ордеров...\n");

    while let Some(order) = order_queue.pop() {
        process_order(&order);
    }

    println!("\nОчередь пуста. Все ордера обработаны.");
}

struct Order {
    id: u32,
    symbol: &'static str,
    quantity: f64,
    price: f64,
}

fn process_order(order: &Order) {
    let value = order.quantity * order.price;
    println!(
        "Ордер #{}: {} {} @ ${:.2} = ${:.2}",
        order.id, order.quantity, order.symbol, order.price, value
    );
}
```

## Чтение данных из итератора

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let mut iter = prices.iter();

    let mut sum = 0.0;
    let mut count = 0;

    while let Some(&price) = iter.next() {
        sum += price;
        count += 1;
        println!("Цена {}: ${:.2}, сумма: ${:.2}", count, price, sum);
    }

    let average = sum / count as f64;
    println!("\nСредняя цена: ${:.2}", average);
}
```

## Парсинг потока котировок

```rust
fn main() {
    // Симуляция потока данных от биржи
    let raw_data = vec!["42000.50", "42100.75", "invalid", "42050.25"];
    let mut data_iter = raw_data.iter();

    let mut valid_prices = Vec::new();

    while let Some(raw) = data_iter.next() {
        // Пробуем распарсить, пропускаем невалидные
        if let Ok(price) = raw.parse::<f64>() {
            valid_prices.push(price);
            println!("Получена цена: ${:.2}", price);
        } else {
            println!("Пропуск невалидных данных: {}", raw);
        }
    }

    println!("\nВалидных цен: {}", valid_prices.len());
}
```

## Вложенные while let

```rust
fn main() {
    let portfolios = vec![
        vec!["BTC", "ETH", "SOL"],
        vec!["AAPL", "GOOGL"],
        vec!["EUR/USD", "GBP/USD", "USD/JPY"],
    ];

    let mut portfolio_iter = portfolios.iter();
    let mut portfolio_num = 0;

    while let Some(portfolio) = portfolio_iter.next() {
        portfolio_num += 1;
        println!("Портфель {}:", portfolio_num);

        let mut asset_iter = portfolio.iter();
        while let Some(asset) = asset_iter.next() {
            println!("  - {}", asset);
        }
    }
}
```

## Обработка Result в цикле

```rust
fn main() {
    let trade_strings = vec!["BTC:0.5:42000", "ETH:10:2200", "invalid", "SOL:100:25"];
    let mut iter = trade_strings.iter();

    let mut successful_trades = 0;
    let mut total_value = 0.0;

    while let Some(trade_str) = iter.next() {
        match parse_trade(trade_str) {
            Ok((symbol, qty, price)) => {
                let value = qty * price;
                println!("{}: {} @ ${:.2} = ${:.2}", symbol, qty, price, value);
                successful_trades += 1;
                total_value += value;
            }
            Err(e) => {
                println!("Ошибка парсинга '{}': {}", trade_str, e);
            }
        }
    }

    println!("\nУспешных сделок: {}", successful_trades);
    println!("Общая стоимость: ${:.2}", total_value);
}

fn parse_trade(s: &str) -> Result<(&str, f64, f64), &'static str> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err("неверный формат");
    }

    let symbol = parts[0];
    let qty = parts[1].parse::<f64>().map_err(|_| "неверное количество")?;
    let price = parts[2].parse::<f64>().map_err(|_| "неверная цена")?;

    Ok((symbol, qty, price))
}
```

## Практический пример: симуляция торгового потока

```rust
fn main() {
    let mut trading_session = TradingSession::new(10000.0);

    // Симуляция сигналов
    let signals = vec![
        Some(Signal::Buy { symbol: "BTC", qty: 0.1, price: 42000.0 }),
        Some(Signal::Buy { symbol: "ETH", qty: 5.0, price: 2200.0 }),
        Some(Signal::Sell { symbol: "BTC", qty: 0.1, price: 42500.0 }),
        None, // Конец сессии
    ];

    let mut signal_iter = signals.into_iter();

    println!("Начало торговой сессии");
    println!("Баланс: ${:.2}\n", trading_session.balance);

    while let Some(Some(signal)) = signal_iter.next() {
        trading_session.execute(signal);
    }

    println!("\nСессия завершена");
    println!("Итоговый баланс: ${:.2}", trading_session.balance);
    println!("PnL: ${:.2}", trading_session.pnl);
}

struct TradingSession {
    balance: f64,
    pnl: f64,
}

enum Signal {
    Buy { symbol: &'static str, qty: f64, price: f64 },
    Sell { symbol: &'static str, qty: f64, price: f64 },
}

impl TradingSession {
    fn new(balance: f64) -> Self {
        TradingSession { balance, pnl: 0.0 }
    }

    fn execute(&mut self, signal: Signal) {
        match signal {
            Signal::Buy { symbol, qty, price } => {
                let cost = qty * price;
                self.balance -= cost;
                println!("BUY {} {} @ ${:.2} (cost: ${:.2})", qty, symbol, price, cost);
            }
            Signal::Sell { symbol, qty, price } => {
                let revenue = qty * price;
                self.balance += revenue;
                self.pnl += revenue - (qty * 42000.0); // упрощённый PnL
                println!("SELL {} {} @ ${:.2} (revenue: ${:.2})", qty, symbol, price, revenue);
            }
        }
    }
}
```

## Комбинация с break и continue

```rust
fn main() {
    let orders = vec![
        Order { id: 1, amount: 500.0, is_valid: true },
        Order { id: 2, amount: 150.0, is_valid: false },
        Order { id: 3, amount: 10000.0, is_valid: true },  // слишком большой
        Order { id: 4, amount: 800.0, is_valid: true },
    ];

    let mut iter = orders.iter();
    let max_order_size = 5000.0;
    let mut processed = 0;

    while let Some(order) = iter.next() {
        // Пропускаем невалидные
        if !order.is_valid {
            println!("Ордер #{} невалиден, пропуск", order.id);
            continue;
        }

        // Прерываем при слишком большом ордере
        if order.amount > max_order_size {
            println!("Ордер #{} превышает лимит ${:.2}, остановка", order.id, max_order_size);
            break;
        }

        println!("Обработан ордер #{}: ${:.2}", order.id, order.amount);
        processed += 1;
    }

    println!("\nОбработано ордеров: {}", processed);
}

struct Order {
    id: u32,
    amount: f64,
    is_valid: bool,
}
```

## Паттерны использования while let

```rust
fn main() {
    // 1. Опустошение очереди
    let mut queue = vec![1, 2, 3];
    while let Some(item) = queue.pop() {
        println!("Элемент: {}", item);
    }

    // 2. Итерация по итератору
    let data = [10, 20, 30];
    let mut iter = data.iter();
    while let Some(value) = iter.next() {
        println!("Значение: {}", value);
    }

    // 3. Деструктуризация кортежей
    let pairs = vec![(1, "a"), (2, "b")];
    let mut pair_iter = pairs.iter();
    while let Some((num, letter)) = pair_iter.next() {
        println!("{}: {}", num, letter);
    }

    // 4. Вложенный Option
    let nested: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let mut nested_iter = nested.iter();
    while let Some(opt) = nested_iter.next() {
        if let Some(val) = opt {
            println!("Найдено: {}", val);
        }
    }
}
```

## Что мы узнали

| Конструкция | Описание | Когда использовать |
|-------------|----------|-------------------|
| `while let Some(x) = expr` | Цикл пока Option содержит значение | Обработка очередей, итераторов |
| `while let Ok(x) = expr` | Цикл пока Result успешен | Чтение данных, парсинг |
| `while let Pattern = expr` | Общий паттерн | Деструктуризация в цикле |

## Практические упражнения

### Упражнение 1: Обработка стакана ордеров
Создай функцию, которая обрабатывает стакан ордеров (order book) с помощью `while let`, пока не встретит ордер с нулевым количеством.

### Упражнение 2: Парсинг торговой истории
Напиши парсер, который читает строки торговой истории и останавливается при первой ошибке парсинга.

### Упражнение 3: Агрегация свечей
Реализуй функцию, которая собирает тиковые данные в свечи, используя `while let` для чтения потока тиков.

### Упражнение 4: Фильтрация позиций
Создай функцию, которая итерирует по позициям и закрывает убыточные, используя `while let` с условиями `continue` и `break`.

## Домашнее задание

1. Напиши функцию `process_market_data(stream: &mut Iterator<Item = MarketTick>) -> Summary`, которая использует `while let` для обработки потока рыночных данных и возвращает суммарную статистику.

2. Создай симулятор очереди ордеров с приоритетами. Используй `while let` для обработки ордеров в порядке приоритета.

3. Реализуй функцию поиска арбитражных возможностей, которая сканирует поток котировок с разных бирж с помощью `while let`.

4. Напиши парсер CSV-файла с историей сделок, который использует `while let` для построчного чтения и останавливается при первой критической ошибке.

## Навигация

[← Предыдущий день](../096-if-let-order-matching/ru.md) | [Следующий день →](../098-advanced-matching/ru.md)
