# День 34: Move — продажа актива

## Аналогия из трейдинга

Представь: ты держишь 1 BTC на своём кошельке. Решил продать — перевёл его покупателю. **Теперь у тебя нет этого биткоина**. Он переместился (move) к новому владельцу. Ты не можешь потратить его ещё раз — это физически невозможно.

В Rust переменные работают так же. Когда ты передаёшь значение куда-то, оно **перемещается**, и оригинальная переменная больше недействительна. Это называется **move semantics**.

## Почему это важно?

В трейдинге двойная трата (double spending) — это катастрофа. Если бы можно было потратить один и тот же BTC дважды, вся система рухнула бы.

Rust защищает от аналогичных ошибок в коде. Если данные "переехали", ты не сможешь использовать старую переменную — компилятор не даст.

## Move в действии

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Продаём (передаём) портфель новому владельцу
    let new_owner = portfolio;

    // ОШИБКА! portfolio больше не существует
    // println!("Мой портфель: {}", portfolio);

    // Теперь только new_owner владеет данными
    println!("Портфель нового владельца: {}", new_owner);
}
```

Компилятор скажет:
```
error[E0382]: borrow of moved value: `portfolio`
 --> src/main.rs:8:35
  |
2 |     let portfolio = String::from("BTC: 1.5, ETH: 10.0");
  |         --------- move occurs because `portfolio` has type `String`
3 |     let new_owner = portfolio;
  |                     --------- value moved here
...
8 |     println!("Мой портфель: {}", portfolio);
  |                                  ^^^^^^^^^ value borrowed here after move
```

**Аналогия:** Это как попытка показать другу свой биткоин после того, как ты его продал. Его у тебя уже нет!

## Move при передаче в функцию

Когда передаёшь значение в функцию, оно тоже перемещается:

```rust
fn sell_asset(asset: String) {
    println!("Продаём актив: {}", asset);
    // asset уничтожается в конце функции
}

fn main() {
    let my_btc = String::from("1.0 BTC");

    sell_asset(my_btc);  // my_btc переместился в функцию

    // ОШИБКА! my_btc больше не существует
    // println!("У меня есть: {}", my_btc);
}
```

**Аналогия:** Ты отдал BTC брокеру для продажи. Брокер продал его и закрыл сделку. Биткоин ушёл навсегда.

## Практический пример: Передача ордера

```rust
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn execute_order(order: Order) {
    println!("=== Исполнение ордера ===");
    println!("Тикер: {}", order.symbol);
    println!("Направление: {}", order.side);
    println!("Количество: {}", order.quantity);
    println!("Цена: {} USDT", order.price);
    println!("Ордер исполнен!");
    // order уничтожается здесь
}

fn main() {
    let buy_order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 0.5,
        price: 42000.0,
    };

    // Передаём ордер на исполнение
    execute_order(buy_order);

    // ОШИБКА! buy_order уже не существует
    // println!("Статус: {}", buy_order.side);
}
```

## Move и возврат значения

Функция может вернуть владение обратно:

```rust
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

fn open_position(symbol: String, size: f64, price: f64) -> Position {
    Position {
        symbol,  // symbol перемещается в структуру
        size,
        entry_price: price,
    }
}

fn close_position(position: Position) -> f64 {
    let pnl = (42500.0 - position.entry_price) * position.size;
    println!("Закрываем позицию по {}", position.symbol);
    println!("PnL: {} USDT", pnl);
    pnl  // Возвращаем PnL, position уничтожается
}

fn main() {
    let ticker = String::from("BTCUSDT");

    // ticker перемещается в функцию, возвращается Position
    let position = open_position(ticker, 0.5, 42000.0);

    // position перемещается в close_position
    let profit = close_position(position);

    println!("Итоговая прибыль: {} USDT", profit);
}
```

## Почему String перемещается, а числа — нет?

```rust
fn main() {
    // Числа КОПИРУЮТСЯ, не перемещаются
    let price = 42000.0;
    let copy_price = price;

    println!("Оригинал: {}", price);      // Работает!
    println!("Копия: {}", copy_price);     // Работает!

    // String ПЕРЕМЕЩАЕТСЯ
    let ticker = String::from("BTC");
    let moved_ticker = ticker;

    // println!("{}", ticker);  // ОШИБКА!
    println!("{}", moved_ticker);  // Работает
}
```

Простые типы (числа, bool, char) реализуют **Copy** — они маленькие и их быстро скопировать. Это как наличные деньги — можно легко отсчитать такую же сумму.

**String** и другие сложные типы живут в куче (heap). Копирование было бы дорогим, поэтому они перемещаются. Это как перевод криптовалюты — актив физически переходит к новому владельцу.

## Move в коллекциях

```rust
fn main() {
    let mut portfolio = Vec::new();

    let btc = String::from("BTC");
    let eth = String::from("ETH");

    portfolio.push(btc);  // btc перемещается в вектор
    portfolio.push(eth);  // eth перемещается в вектор

    // ОШИБКА! btc и eth больше не существуют
    // println!("{}, {}", btc, eth);

    // Но можно получить ссылку из вектора
    println!("Первый актив: {}", portfolio[0]);
    println!("Второй актив: {}", portfolio[1]);
}
```

## Паттерн: Приём и возврат владения

```rust
struct Portfolio {
    assets: Vec<String>,
    total_value: f64,
}

fn add_asset(mut portfolio: Portfolio, asset: String, value: f64) -> Portfolio {
    println!("Добавляем {} стоимостью {} USDT", asset, value);
    portfolio.assets.push(asset);
    portfolio.total_value += value;
    portfolio  // Возвращаем владение
}

fn main() {
    let portfolio = Portfolio {
        assets: Vec::new(),
        total_value: 0.0,
    };

    // Каждый раз передаём и получаем обратно
    let portfolio = add_asset(portfolio, String::from("BTC"), 42000.0);
    let portfolio = add_asset(portfolio, String::from("ETH"), 2500.0);
    let portfolio = add_asset(portfolio, String::from("SOL"), 100.0);

    println!("\n=== Итоговый портфель ===");
    for asset in &portfolio.assets {
        println!("- {}", asset);
    }
    println!("Общая стоимость: {} USDT", portfolio.total_value);
}
```

## Симуляция торговой сессии

```rust
struct Trade {
    id: u32,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

struct TradingSession {
    trades: Vec<Trade>,
    balance: f64,
}

fn execute_trade(mut session: TradingSession, trade: Trade) -> TradingSession {
    let trade_value = trade.quantity * trade.price;

    if trade.side == "BUY" {
        session.balance -= trade_value;
        println!("КУПИЛИ {} {} по {} = -{} USDT",
            trade.quantity, trade.symbol, trade.price, trade_value);
    } else {
        session.balance += trade_value;
        println!("ПРОДАЛИ {} {} по {} = +{} USDT",
            trade.quantity, trade.symbol, trade.price, trade_value);
    }

    session.trades.push(trade);  // trade перемещается в вектор
    session
}

fn close_session(session: TradingSession) {
    println!("\n=== Закрытие сессии ===");
    println!("Всего сделок: {}", session.trades.len());
    println!("Финальный баланс: {} USDT", session.balance);

    println!("\nИстория сделок:");
    for trade in &session.trades {
        println!("  #{}: {} {} {} @ {}",
            trade.id, trade.side, trade.quantity, trade.symbol, trade.price);
    }
    // session уничтожается здесь
}

fn main() {
    let session = TradingSession {
        trades: Vec::new(),
        balance: 100000.0,
    };

    println!("=== Начало торговой сессии ===");
    println!("Начальный баланс: {} USDT\n", session.balance);

    let trade1 = Trade {
        id: 1,
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 1.0,
        price: 42000.0,
    };

    let trade2 = Trade {
        id: 2,
        symbol: String::from("ETHUSDT"),
        side: String::from("BUY"),
        quantity: 10.0,
        price: 2500.0,
    };

    let trade3 = Trade {
        id: 3,
        symbol: String::from("BTCUSDT"),
        side: String::from("SELL"),
        quantity: 0.5,
        price: 43000.0,
    };

    let session = execute_trade(session, trade1);
    let session = execute_trade(session, trade2);
    let session = execute_trade(session, trade3);

    close_session(session);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Move | Передача владения данными |
| После move | Оригинальная переменная недействительна |
| Защита | Компилятор не даст использовать перемещённое значение |
| Copy типы | Числа, bool, char копируются, не перемещаются |
| Возврат | Функция может вернуть владение |

## Домашнее задание

1. **Система ордеров**: Создай структуру `Order` и функцию `submit_order(order: Order)`, которая "отправляет" ордер. Убедись, что после отправки использовать ордер нельзя.

2. **Портфельный менеджер**: Реализуй:
   - Структуру `Asset` с полями `symbol: String` и `quantity: f64`
   - Структуру `Portfolio` с `Vec<Asset>` и `total_value: f64`
   - Функцию `add_to_portfolio(portfolio: Portfolio, asset: Asset) -> Portfolio`
   - Функцию `remove_from_portfolio(portfolio: Portfolio, index: usize) -> (Portfolio, Asset)`

3. **Торговый движок**: Создай симуляцию где:
   - Есть 3 трейдера, каждый со своим балансом
   - Один актив может принадлежать только одному трейдеру
   - Реализуй функцию `transfer_asset(from: Trader, to: Trader, asset: Asset) -> (Trader, Trader)`

4. **Эксперимент с ошибками**: Напиши код который:
   - Пытается использовать переменную после move
   - Прочитай сообщение об ошибке компилятора
   - Исправь код тремя разными способами

## Навигация

[← Предыдущий день](../033-ownership-who-holds/ru.md) | [Следующий день →](../035-clone-copying-portfolio/ru.md)
