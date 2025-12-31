# День 38: Заимствование — временный доступ к данным

## Аналогия из трейдинга

Представь, что ты управляешь торговым фондом. У тебя есть **портфель** (данные), и разные люди хотят с ним работать:

- **Аналитики** хотят **посмотреть** портфель, чтобы оценить риски
- **Аудитор** хочет **проверить** состав позиций
- **Риск-менеджер** хочет **изменить** лимиты

Ключевой момент: ты **не передаёшь владение** портфелем — ты лишь **даёшь временный доступ**. При этом:
- Смотреть могут **многие одновременно** (read-only доступ)
- Изменять может только **один человек** (эксклюзивный доступ)

В Rust это называется **заимствование** (borrowing).

## Что такое заимствование?

Заимствование — это создание **ссылки** на данные без передачи владения. Владелец остаётся прежним, но другой код получает временный доступ.

```rust
fn main() {
    let portfolio = String::from("BTC: 2.5, ETH: 10.0, SOL: 100.0");

    // Создаём ссылку — заимствуем данные
    let portfolio_ref = &portfolio;

    println!("Просмотр: {}", portfolio_ref);
    println!("Оригинал: {}", portfolio);  // portfolio всё ещё доступен!
}
```

Символ `&` создаёт **ссылку** (reference) — это как дать кому-то посмотреть документ, не отдавая его.

## Два типа ссылок

### 1. Неизменяемая ссылка `&T`

Позволяет только **читать** данные:

```rust
fn main() {
    let btc_price = 42000.0;

    // Несколько неизменяемых ссылок одновременно — это OK
    let ref1 = &btc_price;
    let ref2 = &btc_price;
    let ref3 = &btc_price;

    println!("Терминал 1 видит: {}", ref1);
    println!("Терминал 2 видит: {}", ref2);
    println!("Терминал 3 видит: {}", ref3);
}
```

**Аналогия:** Несколько трейдеров смотрят на одну книгу ордеров — каждый видит одни и те же данные.

### 2. Изменяемая ссылка `&mut T`

Позволяет **изменять** данные:

```rust
fn main() {
    let mut balance = 10000.0;

    // Создаём изменяемую ссылку
    let balance_ref = &mut balance;

    // Через ссылку можем изменить значение
    *balance_ref += 500.0;  // Добавляем прибыль

    println!("Новый баланс: {}", balance);
}
```

Оператор `*` (разыменование) позволяет обратиться к значению по ссылке.

## Правила заимствования

Rust строго следит за безопасностью. Вот главные правила:

### Правило 1: Много читателей ИЛИ один писатель

```rust
fn main() {
    let mut order_book = String::from("Bid: 42000, Ask: 42010");

    // Можно: много неизменяемых ссылок
    let view1 = &order_book;
    let view2 = &order_book;
    println!("{} | {}", view1, view2);

    // Можно: одна изменяемая ссылка
    let editor = &mut order_book;
    editor.push_str(", Spread: 10");
    println!("{}", editor);
}
```

### Правило 2: Нельзя совмещать изменяемую и неизменяемую ссылки

```rust
fn main() {
    let mut price = 42000.0;

    let reader = &price;        // неизменяемая ссылка
    // let writer = &mut price; // ОШИБКА! Нельзя создать &mut пока есть &

    println!("Цена: {}", reader);

    // После последнего использования reader, можно создать &mut
    let writer = &mut price;
    *writer = 42500.0;
}
```

**Аналогия:** Пока аудиторы проверяют отчёт (читают), никто не может его редактировать.

## Заимствование в функциях

Самое частое использование — передача данных в функции без потери владения:

```rust
// Функция принимает ссылку — только читает
fn calculate_position_value(price: &f64, quantity: &f64) -> f64 {
    price * quantity
}

// Функция принимает изменяемую ссылку — может изменить
fn apply_fee(balance: &mut f64, fee_percent: f64) {
    let fee = *balance * fee_percent / 100.0;
    *balance -= fee;
}

fn main() {
    let btc_price = 42000.0;
    let btc_quantity = 0.5;
    let mut balance = 10000.0;

    // Передаём ссылки — владение остаётся у нас
    let value = calculate_position_value(&btc_price, &btc_quantity);
    println!("Стоимость позиции: {} USDT", value);

    // Передаём изменяемую ссылку
    apply_fee(&mut balance, 0.1);
    println!("Баланс после комиссии: {} USDT", balance);

    // Переменные всё ещё наши!
    println!("Цена BTC: {}", btc_price);
}
```

## Практический пример: Анализатор портфеля

```rust
struct Portfolio {
    btc: f64,
    eth: f64,
    usdt: f64,
}

// Только чтение — неизменяемая ссылка
fn total_value(portfolio: &Portfolio, btc_price: f64, eth_price: f64) -> f64 {
    portfolio.btc * btc_price + portfolio.eth * eth_price + portfolio.usdt
}

// Только чтение — проверка рисков
fn check_concentration(portfolio: &Portfolio, btc_price: f64, eth_price: f64) -> bool {
    let total = total_value(portfolio, btc_price, eth_price);
    let btc_share = (portfolio.btc * btc_price) / total * 100.0;

    if btc_share > 50.0 {
        println!("Внимание! BTC составляет {:.1}% портфеля", btc_share);
        return false;
    }
    true
}

// Изменение — ребалансировка
fn rebalance(portfolio: &mut Portfolio, sell_btc: f64, btc_price: f64) {
    portfolio.btc -= sell_btc;
    portfolio.usdt += sell_btc * btc_price;
    println!("Продано {} BTC по цене {}", sell_btc, btc_price);
}

fn main() {
    let mut my_portfolio = Portfolio {
        btc: 2.0,
        eth: 10.0,
        usdt: 5000.0,
    };

    let btc_price = 42000.0;
    let eth_price = 2200.0;

    // Читаем несколько раз — OK
    let value = total_value(&my_portfolio, btc_price, eth_price);
    println!("Стоимость портфеля: {} USDT", value);

    let is_balanced = check_concentration(&my_portfolio, btc_price, eth_price);

    if !is_balanced {
        // Изменяем портфель
        rebalance(&mut my_portfolio, 0.5, btc_price);

        // Проверяем результат
        let new_value = total_value(&my_portfolio, btc_price, eth_price);
        println!("Новая стоимость: {} USDT", new_value);
    }
}
```

## Пример: Мониторинг сделок

```rust
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

// Неизменяемая ссылка — анализ сделки
fn analyze_trade(trade: &Trade) {
    let value = trade.price * trade.quantity;
    println!(
        "Анализ: {} {} {} по {} = {} USDT",
        trade.side, trade.quantity, trade.symbol, trade.price, value
    );
}

// Неизменяемая ссылка — проверка лимитов
fn check_limits(trade: &Trade, max_value: f64) -> bool {
    let value = trade.price * trade.quantity;
    if value > max_value {
        println!("Превышен лимит! {} > {}", value, max_value);
        return false;
    }
    true
}

// Изменяемая ссылка — корректировка объёма
fn adjust_quantity(trade: &mut Trade, max_value: f64) {
    let current_value = trade.price * trade.quantity;
    if current_value > max_value {
        trade.quantity = max_value / trade.price;
        println!("Объём скорректирован до {}", trade.quantity);
    }
}

fn main() {
    let mut trade = Trade {
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 1.0,
    };

    let max_trade_value = 30000.0;

    // Анализируем (читаем)
    analyze_trade(&trade);

    // Проверяем лимиты (читаем)
    if !check_limits(&trade, max_trade_value) {
        // Корректируем (изменяем)
        adjust_quantity(&mut trade, max_trade_value);

        // Повторно анализируем
        analyze_trade(&trade);
    }
}
```

## Время жизни ссылок

Ссылка не может жить дольше, чем данные, на которые она указывает:

```rust
fn main() {
    let reference;

    {
        let price = 42000.0;
        reference = &price;
        println!("Внутри блока: {}", reference);
    } // price уничтожается здесь

    // println!("{}", reference);  // ОШИБКА! price уже не существует
}
```

Rust проверяет это на этапе компиляции — dangling references невозможны!

## Срезы — особый вид заимствования

Срезы (slices) позволяют заимствовать часть коллекции:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0];

    // Срез — ссылка на часть вектора
    let last_three = &prices[2..5];

    println!("Последние 3 цены: {:?}", last_three);
    println!("Все цены: {:?}", prices);  // Оригинал доступен
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&T` | Неизменяемая ссылка (много одновременно) |
| `&mut T` | Изменяемая ссылка (только одна) |
| `*` | Разыменование ссылки |
| Правила | Много & ИЛИ один &mut, но не вместе |
| Функции | Заимствование для передачи без потери владения |

## Практические задания

### Задание 1: Калькулятор риска
Напиши функцию `calculate_risk`, которая принимает неизменяемую ссылку на структуру `Position` и возвращает размер риска:

```rust
struct Position {
    entry_price: f64,
    stop_loss: f64,
    quantity: f64,
}

// Реализуй функцию
fn calculate_risk(position: &Position) -> f64 {
    // Риск = (entry_price - stop_loss) * quantity
    todo!()
}
```

### Задание 2: Обновление стоп-лосса
Напиши функцию `update_stop_loss`, которая принимает изменяемую ссылку и обновляет стоп-лосс:

```rust
fn update_stop_loss(position: &mut Position, new_stop: f64) {
    // Обнови stop_loss позиции
    todo!()
}
```

### Задание 3: Множественный анализ
Создай несколько функций, которые анализируют портфель через неизменяемые ссылки:

```rust
fn get_largest_position(portfolio: &Portfolio) -> &str {
    // Верни название самой большой позиции
    todo!()
}

fn count_positions(portfolio: &Portfolio) -> usize {
    // Посчитай количество позиций > 0
    todo!()
}
```

### Задание 4: Безопасная модификация
Напиши функцию, которая увеличивает позицию только если хватает свободных средств:

```rust
fn add_to_position(portfolio: &mut Portfolio, amount: f64, price: f64) -> bool {
    // Проверь, хватает ли usdt
    // Если да — добавь к btc и вычти из usdt
    // Верни true при успехе, false при неудаче
    todo!()
}
```

## Домашнее задание

1. **Торговый журнал**: Создай структуру `TradeLog` с вектором сделок. Напиши функции:
   - `add_trade(&mut TradeLog, Trade)` — добавить сделку
   - `total_pnl(&TradeLog) -> f64` — посчитать общий PnL
   - `best_trade(&TradeLog) -> &Trade` — вернуть ссылку на лучшую сделку

2. **Система лимитов**: Создай структуру `RiskLimits` с лимитами. Напиши:
   - `check_trade(&RiskLimits, &Trade) -> bool` — проверка сделки
   - `update_limit(&mut RiskLimits, limit_name: &str, new_value: f64)` — обновление лимита

3. **Эксперимент**: Попробуй создать одновременно `&` и `&mut` ссылку на одну переменную. Прочитай сообщение об ошибке и объясни, почему Rust это запрещает.

## Навигация

[← Предыдущий день](../037-ownership-basics/ru.md) | [Следующий день →](../039-lifetimes-intro/ru.md)
