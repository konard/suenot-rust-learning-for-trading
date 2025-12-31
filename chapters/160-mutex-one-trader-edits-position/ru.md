# День 160: Mutex: один трейдер редактирует позицию

## Аналогия из трейдинга

Представь торговый зал, где несколько трейдеров работают с одним общим портфелем. Если двое одновременно попытаются изменить позицию — будет хаос: один покупает, другой продаёт, и никто не знает актуальный баланс.

**Mutex (Mutual Exclusion)** — это как ключ от сейфа с документами позиции. Только один трейдер может держать ключ в один момент времени. Остальные ждут, пока он закончит и вернёт ключ.

## Зачем нужен Mutex

В многопоточных программах несколько потоков могут одновременно пытаться изменить одни и те же данные. Без синхронизации это приводит к **гонке данных (data race)** — непредсказуемым результатам.

```rust
use std::sync::Mutex;

fn main() {
    // Создаём Mutex, защищающий баланс счёта
    let balance = Mutex::new(10000.0_f64);

    // Получаем доступ через lock()
    {
        let mut guard = balance.lock().unwrap();
        *guard -= 1500.0; // Списываем средства на покупку
        println!("Баланс после покупки: ${:.2}", *guard);
    } // guard выходит из области видимости — блокировка снимается

    // Теперь другой код может получить доступ
    {
        let guard = balance.lock().unwrap();
        println!("Текущий баланс: ${:.2}", *guard);
    }
}
```

**Ключевые моменты:**
- `Mutex::new(value)` — создаёт Mutex с начальным значением
- `lock()` — блокирует Mutex и возвращает `MutexGuard`
- `MutexGuard` — умный указатель с автоматическим снятием блокировки

## Почему lock() возвращает Result

```rust
use std::sync::Mutex;

fn main() {
    let position = Mutex::new(100);

    // lock() возвращает Result<MutexGuard, PoisonError>
    match position.lock() {
        Ok(guard) => println!("Позиция: {} акций", *guard),
        Err(poisoned) => {
            // Mutex "отравлен" — предыдущий поток запаниковал, держа блокировку
            println!("Предупреждение: Mutex был отравлен!");
            let guard = poisoned.into_inner();
            println!("Восстановленная позиция: {}", *guard);
        }
    }
}
```

## Изменение данных через Mutex

```rust
use std::sync::Mutex;

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: i32,
    avg_price: f64,
}

fn main() {
    let position = Mutex::new(Position {
        symbol: String::from("AAPL"),
        quantity: 100,
        avg_price: 150.0,
    });

    // Добавляем к позиции
    {
        let mut pos = position.lock().unwrap();
        let add_qty = 50;
        let add_price = 155.0;

        // Пересчитываем среднюю цену
        let total_value = pos.avg_price * pos.quantity as f64
                        + add_price * add_qty as f64;
        pos.quantity += add_qty;
        pos.avg_price = total_value / pos.quantity as f64;

        println!("Позиция обновлена: {:?}", *pos);
    }

    // Читаем итоговую позицию
    let pos = position.lock().unwrap();
    println!("Итого: {} акций {} по ${:.2}",
             pos.quantity, pos.symbol, pos.avg_price);
}
```

## Практический пример: торговый баланс

```rust
use std::sync::Mutex;

struct TradingAccount {
    balance: Mutex<f64>,
    trades_count: Mutex<u32>,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        TradingAccount {
            balance: Mutex::new(initial_balance),
            trades_count: Mutex::new(0),
        }
    }

    fn buy(&self, symbol: &str, price: f64, quantity: u32) -> Result<(), String> {
        let cost = price * quantity as f64;

        let mut balance = self.balance.lock().unwrap();
        if *balance < cost {
            return Err(format!("Недостаточно средств: нужно ${:.2}, есть ${:.2}",
                              cost, *balance));
        }

        *balance -= cost;

        let mut trades = self.trades_count.lock().unwrap();
        *trades += 1;

        println!("Куплено {} {} по ${:.2}. Остаток: ${:.2}",
                 quantity, symbol, price, *balance);
        Ok(())
    }

    fn sell(&self, symbol: &str, price: f64, quantity: u32) {
        let revenue = price * quantity as f64;

        let mut balance = self.balance.lock().unwrap();
        *balance += revenue;

        let mut trades = self.trades_count.lock().unwrap();
        *trades += 1;

        println!("Продано {} {} по ${:.2}. Баланс: ${:.2}",
                 quantity, symbol, price, *balance);
    }

    fn get_stats(&self) -> (f64, u32) {
        let balance = *self.balance.lock().unwrap();
        let trades = *self.trades_count.lock().unwrap();
        (balance, trades)
    }
}

fn main() {
    let account = TradingAccount::new(10000.0);

    // Серия сделок
    account.buy("AAPL", 150.0, 10).unwrap();
    account.buy("GOOGL", 140.0, 5).unwrap();
    account.sell("AAPL", 155.0, 5);

    let (balance, trades) = account.get_stats();
    println!("\n=== Статистика ===");
    println!("Баланс: ${:.2}", balance);
    println!("Сделок: {}", trades);
}
```

## try_lock: неблокирующая попытка захвата

```rust
use std::sync::Mutex;

fn main() {
    let order_book = Mutex::new(vec!["BUY 100 AAPL @ 150"]);

    // Захватываем блокировку
    let _guard = order_book.lock().unwrap();

    // Попытка захвата без ожидания
    match order_book.try_lock() {
        Ok(guard) => println!("Ордера: {:?}", *guard),
        Err(_) => println!("Стакан заявок занят другим трейдером"),
    }

    // _guard всё ещё держит блокировку, поэтому try_lock не удался
}
```

## Mutex в структуре портфеля

```rust
use std::sync::Mutex;
use std::collections::HashMap;

struct Portfolio {
    positions: Mutex<HashMap<String, i32>>,
    cash: Mutex<f64>,
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio {
            positions: Mutex::new(HashMap::new()),
            cash: Mutex::new(initial_cash),
        }
    }

    fn update_position(&self, symbol: &str, delta: i32, price: f64) {
        let cost = price * delta.abs() as f64;

        // Блокируем cash
        {
            let mut cash = self.cash.lock().unwrap();
            if delta > 0 {
                *cash -= cost; // Покупка
            } else {
                *cash += cost; // Продажа
            }
        }

        // Блокируем positions
        {
            let mut positions = self.positions.lock().unwrap();
            let current = positions.entry(symbol.to_string()).or_insert(0);
            *current += delta;

            // Удаляем нулевые позиции
            if *current == 0 {
                positions.remove(symbol);
            }
        }
    }

    fn print_portfolio(&self) {
        let positions = self.positions.lock().unwrap();
        let cash = self.cash.lock().unwrap();

        println!("\n╔════════════════════════════╗");
        println!("║       ПОРТФЕЛЬ             ║");
        println!("╠════════════════════════════╣");
        for (symbol, qty) in positions.iter() {
            println!("║ {:6} {:>18} шт ║", symbol, qty);
        }
        println!("╠════════════════════════════╣");
        println!("║ Кэш:    ${:>15.2} ║", *cash);
        println!("╚════════════════════════════╝");
    }
}

fn main() {
    let portfolio = Portfolio::new(50000.0);

    portfolio.update_position("AAPL", 100, 150.0);
    portfolio.update_position("GOOGL", 50, 140.0);
    portfolio.update_position("MSFT", 75, 380.0);
    portfolio.update_position("AAPL", -30, 155.0);

    portfolio.print_portfolio();
}
```

## Важные правила работы с Mutex

### 1. Минимизируй время блокировки

```rust
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(vec![1, 2, 3, 4, 5]);

    // Плохо: долгая блокировка
    // let guard = data.lock().unwrap();
    // expensive_calculation(&guard);  // Долгая операция
    // another_calculation(&guard);

    // Хорошо: быстрое копирование и освобождение
    let local_copy = {
        let guard = data.lock().unwrap();
        guard.clone()
    }; // Блокировка снята

    // Теперь работаем с копией без блокировки
    let sum: i32 = local_copy.iter().sum();
    println!("Сумма: {}", sum);
}
```

### 2. Избегай вложенных блокировок (риск deadlock)

```rust
use std::sync::Mutex;

struct Account {
    balance: Mutex<f64>,
}

// Потенциальный deadlock при переводе между счетами!
// fn transfer_bad(from: &Account, to: &Account, amount: f64) {
//     let mut from_balance = from.balance.lock().unwrap();
//     let mut to_balance = to.balance.lock().unwrap(); // Может заблокироваться
//     ...
// }

// Решение: всегда блокировать в одном порядке
fn transfer_safe(from: &Account, to: &Account, amount: f64) {
    // Используем адреса для определения порядка
    let (first, second, is_from_first) = {
        let from_ptr = from as *const _ as usize;
        let to_ptr = to as *const _ as usize;
        if from_ptr < to_ptr {
            (&from.balance, &to.balance, true)
        } else {
            (&to.balance, &from.balance, false)
        }
    };

    let mut first_guard = first.lock().unwrap();
    let mut second_guard = second.lock().unwrap();

    if is_from_first {
        *first_guard -= amount;
        *second_guard += amount;
    } else {
        *second_guard -= amount;
        *first_guard += amount;
    }

    println!("Перевод ${:.2} выполнен", amount);
}

fn main() {
    let account1 = Account { balance: Mutex::new(1000.0) };
    let account2 = Account { balance: Mutex::new(500.0) };

    transfer_safe(&account1, &account2, 200.0);

    println!("Счёт 1: ${:.2}", *account1.balance.lock().unwrap());
    println!("Счёт 2: ${:.2}", *account2.balance.lock().unwrap());
}
```

## Шаблон: защищённый ресурс

```rust
use std::sync::Mutex;

// Обёртка для потокобезопасного ресурса
struct Protected<T> {
    data: Mutex<T>,
}

impl<T> Protected<T> {
    fn new(value: T) -> Self {
        Protected {
            data: Mutex::new(value),
        }
    }

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.data.lock().unwrap();
        f(&mut *guard)
    }
}

fn main() {
    let balance = Protected::new(10000.0_f64);

    // Удобный API для работы с защищёнными данными
    balance.with(|b| {
        *b -= 1500.0;
        println!("Новый баланс: ${:.2}", b);
    });

    let current = balance.with(|b| *b);
    println!("Текущий баланс: ${:.2}", current);
}
```

## Что мы узнали

| Концепт | Описание | Применение в трейдинге |
|---------|----------|------------------------|
| `Mutex::new()` | Создание мьютекса | Защита общих данных |
| `lock()` | Блокировка с ожиданием | Безопасное обновление позиции |
| `try_lock()` | Попытка без ожидания | Проверка занятости ресурса |
| `MutexGuard` | RAII-страж блокировки | Автоматическое освобождение |
| Poison | Отравление при панике | Обработка ошибок потоков |

## Практические задания

1. **Счётчик сделок**: Создай структуру `TradeCounter` с Mutex, которая считает количество BUY и SELL сделок отдельно

2. **Лимит позиции**: Реализуй структуру `PositionManager`, которая не позволяет позиции превысить заданный лимит

3. **Логгер сделок**: Создай потокобезопасный `TradeLogger`, который записывает все сделки в Vec

4. **Стакан заявок**: Реализуй простой `OrderBook` с Mutex, где можно добавлять и удалять ордера

## Домашнее задание

1. Создай структуру `RiskManager`, которая:
   - Хранит максимальный дневной убыток в Mutex
   - Отслеживает текущий PnL
   - Возвращает ошибку при попытке сделки, если дневной лимит исчерпан

2. Реализуй `PriceCache` — кеш последних цен для нескольких тикеров:
   - Используй `Mutex<HashMap<String, f64>>`
   - Добавь методы `update_price`, `get_price`, `get_all_prices`

3. Напиши функцию, которая безопасно переводит средства между двумя `TradingAccount`, избегая deadlock

4. Создай `ExecutionQueue` — очередь ордеров на исполнение:
   - Методы `push_order`, `pop_order`, `peek_order`
   - Подсчёт общего объёма в очереди

## Навигация

[← Предыдущий день](../159-sync-channel-bounded-queue/ru.md) | [Следующий день →](../161-arc-shared-access/ru.md)
