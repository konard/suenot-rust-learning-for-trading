# День 41: Правило — нельзя смешивать ссылки

## Аналогия из трейдинга

Представь торговый терминал, который показывает стакан заявок. **Множество трейдеров** могут одновременно **смотреть** на стакан — это безопасно, данные не меняются.

Но что если один трейдер начнёт **редактировать** стакан, пока остальные его читают? Хаос! Кто-то увидит старые данные, кто-то новые, кто-то — частично обновлённые. Это гонка данных (data race).

Rust **запрещает** такие ситуации на этапе компиляции. Правило простое:

> **Либо много читателей, ЛИБО один писатель — но не одновременно.**

## Правило заимствования

В Rust действует строгое правило:

```
В любой момент времени может существовать:
- Либо ОДНА изменяемая ссылка (&mut T)
- Либо ЛЮБОЕ количество неизменяемых ссылок (&T)
- Но НЕ оба типа одновременно!
```

### Почему это важно?

```rust
fn main() {
    let mut portfolio_value = 100000.0;

    // Создаём неизменяемую ссылку
    let reader = &portfolio_value;

    // Пытаемся создать изменяемую ссылку
    let writer = &mut portfolio_value;  // ОШИБКА!

    println!("Значение: {}", reader);
}
```

Компилятор скажет:
```
error[E0502]: cannot borrow `portfolio_value` as mutable
              because it is also borrowed as immutable
```

**Аналогия:** Нельзя редактировать документ, пока другие его читают. Или закрой документ для всех читателей, или жди.

## Проблема: гонка данных

Представь код без защиты Rust:

```rust
fn main() {
    let mut balance = 10000.0;

    // Поток 1: читает баланс для проверки
    let check = &balance;  // Видит 10000

    // Поток 2: списывает деньги
    let modify = &mut balance;
    *modify -= 5000.0;  // Теперь 5000

    // Поток 1: принимает решение на основе СТАРЫХ данных!
    println!("Проверка видит: {}", check);  // Всё ещё думает что 10000!
}
```

В реальном боте это могло бы привести к:
- Открытию позиции, которую уже нельзя себе позволить
- Неправильному расчёту риска
- Потере денег!

**Rust предотвращает это на этапе компиляции.**

## Область действия ссылок (NLL)

Ссылка "живёт" до момента её **последнего использования**, не до конца блока. Это называется Non-Lexical Lifetimes (NLL):

```rust
fn main() {
    let mut order_price = 42000.0;

    // Неизменяемая ссылка
    let snapshot = &order_price;
    println!("Снимок цены: {}", snapshot);
    // snapshot больше не используется — ссылка "умерла"

    // Теперь можно создать изменяемую ссылку!
    let update = &mut order_price;
    *update = 42500.0;
    println!("Обновлённая цена: {}", update);
}
```

Этот код **компилируется**, потому что `snapshot` не используется после `println!`.

### Когда это НЕ работает:

```rust
fn main() {
    let mut order_price = 42000.0;

    let snapshot = &order_price;
    let update = &mut order_price;  // ОШИБКА!

    // snapshot используется ПОСЛЕ создания update
    println!("Снимок: {}", snapshot);
}
```

## Практические примеры

### Пример 1: Чтение и обновление позиции

```rust
fn main() {
    let mut position_size = 1.5;  // 1.5 BTC

    // Сначала читаем
    let current = &position_size;
    println!("Текущая позиция: {} BTC", current);
    // current больше не нужен

    // Теперь обновляем
    let updater = &mut position_size;
    *updater += 0.5;
    println!("После добавления: {} BTC", updater);
}
```

### Пример 2: Множественные читатели

```rust
fn main() {
    let portfolio_value = 150000.0;

    // Много неизменяемых ссылок — это OK!
    let reader1 = &portfolio_value;
    let reader2 = &portfolio_value;
    let reader3 = &portfolio_value;

    println!("Риск-менеджер видит: {}", reader1);
    println!("Отчёт показывает: {}", reader2);
    println!("Dashboard отображает: {}", reader3);
}
```

**Аналогия:** Все могут одновременно смотреть котировки на экране. Это безопасно — никто ничего не меняет.

### Пример 3: Последовательные изменения

```rust
fn main() {
    let mut balance = 10000.0;

    println!("Начальный баланс: {}", balance);

    // Первое изменение
    {
        let trade1 = &mut balance;
        *trade1 += 500.0;
        println!("После сделки 1: {}", trade1);
    }  // trade1 выходит из области видимости

    // Второе изменение
    {
        let trade2 = &mut balance;
        *trade2 -= 200.0;
        println!("После сделки 2: {}", trade2);
    }

    // Можем снова читать
    println!("Итоговый баланс: {}", balance);
}
```

### Пример 4: Ошибка при одновременном доступе

```rust
fn main() {
    let mut orders = vec!["BTC-USDT", "ETH-USDT", "SOL-USDT"];

    // Получаем ссылку на первый элемент
    let first = &orders[0];

    // Пытаемся добавить новый ордер
    orders.push("DOGE-USDT");  // ОШИБКА!

    println!("Первый ордер: {}", first);
}
```

Почему ошибка? `push` может переаллоцировать Vec в памяти, и тогда `first` будет указывать на недействительную память!

**Правильное решение:**

```rust
fn main() {
    let mut orders = vec!["BTC-USDT", "ETH-USDT", "SOL-USDT"];

    // Сначала читаем и сохраняем значение
    let first = orders[0].to_string();

    // Теперь безопасно добавляем
    orders.push("DOGE-USDT");

    println!("Первый ордер был: {}", first);
    println!("Все ордера: {:?}", orders);
}
```

## Функции и смешивание ссылок

### Ошибка: читаем и пишем одновременно

```rust
fn update_and_log(value: &mut f64, log: &f64) {
    println!("Было: {}", log);
    *value += 100.0;
    println!("Стало: {}", value);
}

fn main() {
    let mut price = 42000.0;

    // Нельзя передать одну переменную как &mut и & одновременно!
    update_and_log(&mut price, &price);  // ОШИБКА!
}
```

### Правильное решение:

```rust
fn update_and_log(value: &mut f64) {
    println!("Было: {}", *value);
    *value += 100.0;
    println!("Стало: {}", *value);
}

fn main() {
    let mut price = 42000.0;
    update_and_log(&mut price);  // OK!
}
```

## Паттерн: сначала читай, потом пиши

```rust
fn analyze_and_update_position(
    current_price: f64,
    position: &mut f64,
    entry_price: f64,
) {
    // Сначала вычисляем на основе текущего значения
    let current_pnl = (*position) * (current_price - entry_price);
    println!("Текущий PnL: {:.2} USDT", current_pnl);

    // Потом обновляем
    if current_pnl > 100.0 {
        *position *= 0.5;  // Фиксируем часть прибыли
        println!("Позиция уменьшена вдвое");
    }
}

fn main() {
    let mut position = 2.0;  // 2 BTC
    let entry_price = 40000.0;
    let current_price = 42000.0;

    analyze_and_update_position(current_price, &mut position, entry_price);
    println!("Итоговая позиция: {} BTC", position);
}
```

## Реальный пример: торговая система

```rust
struct TradingAccount {
    balance: f64,
    open_positions: i32,
    total_pnl: f64,
}

fn display_account(account: &TradingAccount) {
    println!("=== Состояние счёта ===");
    println!("Баланс: {:.2} USDT", account.balance);
    println!("Открытых позиций: {}", account.open_positions);
    println!("Общий PnL: {:.2} USDT", account.total_pnl);
}

fn execute_trade(account: &mut TradingAccount, profit: f64) {
    account.balance += profit;
    account.total_pnl += profit;
    if profit > 0.0 {
        println!("Прибыльная сделка: +{:.2}", profit);
    } else {
        println!("Убыточная сделка: {:.2}", profit);
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 10000.0,
        open_positions: 0,
        total_pnl: 0.0,
    };

    // Читаем состояние
    display_account(&account);
    println!();

    // Выполняем сделки
    execute_trade(&mut account, 150.0);
    execute_trade(&mut account, -50.0);
    execute_trade(&mut account, 300.0);
    println!();

    // Снова читаем
    display_account(&account);
}
```

Вывод:
```
=== Состояние счёта ===
Баланс: 10000.00 USDT
Открытых позиций: 0
Общий PnL: 0.00 USDT

Прибыльная сделка: +150.00
Убыточная сделка: -50.00
Прибыльная сделка: +300.00

=== Состояние счёта ===
Баланс: 10400.00 USDT
Открытых позиций: 0
Общий PnL: 400.00 USDT
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Правило смешивания | Нельзя иметь `&mut` и `&` одновременно |
| Гонка данных | Проблема, которую Rust предотвращает |
| NLL | Ссылки живут до последнего использования |
| Множество `&` | Много неизменяемых ссылок — OK |
| Один `&mut` | Только одна изменяемая ссылка за раз |

## Домашнее задание

1. **Исправь ошибку:**
   ```rust
   fn main() {
       let mut prices = vec![100.0, 200.0, 300.0];
       let first = &prices[0];
       prices.push(400.0);
       println!("Первая цена: {}", first);
   }
   ```

2. **Перепиши код корректно:**
   ```rust
   fn main() {
       let mut balance = 5000.0;
       let reader = &balance;
       let writer = &mut balance;
       *writer += 100.0;
       println!("Читатель видит: {}", reader);
   }
   ```

3. **Создай структуру `Portfolio`** с полями `cash` и `positions`. Напиши функции:
   - `display(portfolio: &Portfolio)` — выводит состояние
   - `deposit(portfolio: &mut Portfolio, amount: f64)` — пополняет cash
   - Продемонстрируй последовательный вызов обеих функций

4. **Дополнительное задание:** Объясни, почему этот код компилируется:
   ```rust
   fn main() {
       let mut value = 42;
       let r1 = &value;
       let r2 = &value;
       println!("{} и {}", r1, r2);
       let r3 = &mut value;
       *r3 += 1;
       println!("{}", r3);
   }
   ```

## Навигация

[← День 40: Правило одной изменяемой ссылки](../040-one-mutable-reference/ru.md) | [День 42: Dangling References →](../042-dangling-references/ru.md)
