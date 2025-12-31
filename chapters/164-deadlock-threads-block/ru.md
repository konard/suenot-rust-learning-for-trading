# День 164: Deadlock: когда потоки заблокировали друг друга

## Аналогия из трейдинга

Представь ситуацию на бирже: трейдер A хочет обменять BTC на ETH, а трейдер B хочет обменять ETH на BTC. Трейдер A сначала блокирует свои BTC и ждёт, пока появятся ETH для обмена. В то же время трейдер B блокирует свои ETH и ждёт BTC. Оба ждут друг друга — и никто не может завершить сделку. Это и есть **deadlock** (взаимная блокировка) — ситуация, когда два или более потоков навечно заблокированы, ожидая ресурсы, которые держит другой поток.

В реальном трейдинге это может произойти, когда:
- Один поток блокирует стакан заявок и ждёт доступа к балансам
- Другой поток блокирует балансы и ждёт доступа к стакану заявок
- Оба застряли навсегда!

## Что такое Deadlock?

Deadlock возникает при выполнении четырёх условий одновременно:

1. **Mutual Exclusion** — ресурс может быть захвачен только одним потоком
2. **Hold and Wait** — поток удерживает один ресурс и ждёт другой
3. **No Preemption** — ресурс нельзя отобрать принудительно
4. **Circular Wait** — существует цикл ожидания между потоками

## Простой пример Deadlock

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Два ресурса: баланс BTC и баланс ETH
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let btc2 = Arc::clone(&btc_balance);
    let eth2 = Arc::clone(&eth_balance);

    // Поток 1: сначала блокирует BTC, потом ETH
    let handle1 = thread::spawn(move || {
        println!("Поток 1: Пытаюсь заблокировать BTC...");
        let _btc = btc1.lock().unwrap();
        println!("Поток 1: BTC заблокирован!");

        // Имитируем работу
        thread::sleep(Duration::from_millis(100));

        println!("Поток 1: Пытаюсь заблокировать ETH...");
        let _eth = eth1.lock().unwrap(); // DEADLOCK! Поток 2 уже держит ETH
        println!("Поток 1: ETH заблокирован!");

        println!("Поток 1: Выполняю обмен BTC -> ETH");
    });

    // Поток 2: сначала блокирует ETH, потом BTC
    let handle2 = thread::spawn(move || {
        println!("Поток 2: Пытаюсь заблокировать ETH...");
        let _eth = eth2.lock().unwrap();
        println!("Поток 2: ETH заблокирован!");

        // Имитируем работу
        thread::sleep(Duration::from_millis(100));

        println!("Поток 2: Пытаюсь заблокировать BTC...");
        let _btc = btc2.lock().unwrap(); // DEADLOCK! Поток 1 уже держит BTC
        println!("Поток 2: BTC заблокирован!");

        println!("Поток 2: Выполняю обмен ETH -> BTC");
    });

    // Эти join никогда не завершатся!
    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Программа завершена"); // Эта строка никогда не выполнится
}
```

**Важно:** Этот код зависнет навсегда! Не запускайте его в продакшене.

## Визуализация Deadlock

```
Поток 1                     Поток 2
   |                           |
   v                           v
Блокирует BTC ----+    +---- Блокирует ETH
   |              |    |       |
   v              |    |       v
Ждёт ETH... <-----+----+--> Ждёт BTC...
   |                           |
   X DEADLOCK X           X DEADLOCK X
```

## Пример с торговым движком

Рассмотрим более реалистичный пример — торговый движок с ордерами:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Debug)]
struct Portfolio {
    cash: f64,
    positions: Vec<(String, f64)>, // (symbol, quantity)
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        bids: vec![],
        asks: vec![],
    }));

    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        positions: vec![("BTC".to_string(), 5.0)],
    }));

    let ob1 = Arc::clone(&order_book);
    let pf1 = Arc::clone(&portfolio);

    let ob2 = Arc::clone(&order_book);
    let pf2 = Arc::clone(&portfolio);

    // Поток 1: Добавление ордера
    // Сначала проверяет портфель, потом обновляет стакан
    let handle1 = thread::spawn(move || {
        println!("Поток 1 (Добавление ордера): Блокирую портфель...");
        let portfolio_guard = pf1.lock().unwrap();
        println!("Поток 1: Портфель заблокирован, проверяю баланс...");

        if portfolio_guard.cash >= 50_000.0 {
            thread::sleep(Duration::from_millis(50));

            println!("Поток 1: Пытаюсь заблокировать стакан заявок...");
            let mut ob_guard = ob1.lock().unwrap();

            ob_guard.bids.push(Order {
                id: 1,
                symbol: "BTC".to_string(),
                price: 42000.0,
                quantity: 1.0,
            });
            println!("Поток 1: Ордер добавлен!");
        }
    });

    // Поток 2: Исполнение ордера
    // Сначала смотрит стакан, потом обновляет портфель
    let handle2 = thread::spawn(move || {
        println!("Поток 2 (Исполнение ордера): Блокирую стакан заявок...");
        let ob_guard = ob2.lock().unwrap();
        println!("Поток 2: Стакан заблокирован, ищу ордера...");

        if !ob_guard.bids.is_empty() {
            thread::sleep(Duration::from_millis(50));

            println!("Поток 2: Пытаюсь заблокировать портфель...");
            let mut portfolio_guard = pf2.lock().unwrap();

            portfolio_guard.cash -= 42000.0;
            println!("Поток 2: Ордер исполнен!");
        }
    });

    // Потенциальный DEADLOCK!
    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Обнаружение Deadlock

Rust не предотвращает deadlock автоматически. Вот как можно обнаружить проблему:

### 1. try_lock — неблокирующая попытка захвата

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let handle = thread::spawn(move || {
        // Блокируем BTC
        let btc_guard = btc1.lock().unwrap();
        println!("BTC заблокирован: {}", *btc_guard);

        // Пробуем заблокировать ETH без ожидания
        match eth1.try_lock() {
            Ok(eth_guard) => {
                println!("ETH тоже заблокирован: {}", *eth_guard);
                // Выполняем операцию
            }
            Err(_) => {
                println!("Не удалось заблокировать ETH, ресурс занят");
                // Можно повторить позже или освободить BTC
            }
        }
    });

    handle.join().unwrap();
}
```

### 2. Таймаут с использованием parking_lot

```rust
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let balance = Arc::new(Mutex::new(10.0_f64));
    let balance_clone = Arc::clone(&balance);

    // Поток 1 захватывает блокировку надолго
    let handle1 = thread::spawn(move || {
        let _guard = balance_clone.lock();
        println!("Поток 1: Держу блокировку...");
        thread::sleep(Duration::from_secs(2));
        println!("Поток 1: Освобождаю блокировку");
    });

    // Небольшая задержка, чтобы поток 1 успел захватить блокировку
    thread::sleep(Duration::from_millis(100));

    // Поток 2 пробует с таймаутом
    let handle2 = thread::spawn(move || {
        println!("Поток 2: Пробую захватить с таймаутом...");

        if let Some(guard) = balance.try_lock_for(Duration::from_millis(500)) {
            println!("Поток 2: Успех! Баланс: {}", *guard);
        } else {
            println!("Поток 2: Таймаут! Возможный deadlock.");
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Паттерны предотвращения Deadlock

### 1. Единый порядок блокировки

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let btc2 = Arc::clone(&btc_balance);
    let eth2 = Arc::clone(&eth_balance);

    // Оба потока блокируют в ОДИНАКОВОМ порядке: сначала BTC, потом ETH
    let handle1 = thread::spawn(move || {
        let btc = btc1.lock().unwrap();
        let eth = eth1.lock().unwrap();
        println!("Поток 1: BTC={}, ETH={}", *btc, *eth);
    });

    let handle2 = thread::spawn(move || {
        let btc = btc2.lock().unwrap();
        let eth = eth2.lock().unwrap();
        println!("Поток 2: BTC={}, ETH={}", *btc, *eth);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Программа успешно завершена!");
}
```

### 2. Блокировка на короткое время

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct TradingEngine {
    btc_balance: Arc<Mutex<f64>>,
    eth_balance: Arc<Mutex<f64>>,
}

impl TradingEngine {
    fn new(btc: f64, eth: f64) -> Self {
        TradingEngine {
            btc_balance: Arc::new(Mutex::new(btc)),
            eth_balance: Arc::new(Mutex::new(eth)),
        }
    }

    // Короткие, атомарные операции
    fn get_btc_balance(&self) -> f64 {
        *self.btc_balance.lock().unwrap()
    }

    fn get_eth_balance(&self) -> f64 {
        *self.eth_balance.lock().unwrap()
    }

    fn swap(&self, btc_amount: f64, eth_amount: f64) -> Result<(), String> {
        // Проверяем балансы (короткие блокировки)
        let current_btc = self.get_btc_balance();
        let current_eth = self.get_eth_balance();

        if current_btc < btc_amount {
            return Err("Недостаточно BTC".to_string());
        }

        // Выполняем обмен (отдельные короткие блокировки)
        {
            let mut btc = self.btc_balance.lock().unwrap();
            *btc -= btc_amount;
        }

        {
            let mut eth = self.eth_balance.lock().unwrap();
            *eth += eth_amount;
        }

        Ok(())
    }
}

fn main() {
    let engine = Arc::new(TradingEngine::new(10.0, 100.0));

    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);

    let h1 = thread::spawn(move || {
        for i in 0..5 {
            match e1.swap(1.0, 10.0) {
                Ok(_) => println!("Обмен {} успешен", i + 1),
                Err(e) => println!("Ошибка обмена {}: {}", i + 1, e),
            }
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..5 {
            println!("BTC: {}, ETH: {}",
                e2.get_btc_balance(),
                e2.get_eth_balance());
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
}
```

### 3. Использование единой блокировки для связанных данных

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Группируем связанные данные в одну структуру
#[derive(Debug, Clone)]
struct TradingState {
    btc_balance: f64,
    eth_balance: f64,
    pending_orders: u32,
}

fn main() {
    // Одна блокировка для всего состояния
    let state = Arc::new(Mutex::new(TradingState {
        btc_balance: 10.0,
        eth_balance: 100.0,
        pending_orders: 0,
    }));

    let state1 = Arc::clone(&state);
    let state2 = Arc::clone(&state);

    let handle1 = thread::spawn(move || {
        let mut s = state1.lock().unwrap();
        s.btc_balance -= 1.0;
        s.eth_balance += 15.0;
        s.pending_orders += 1;
        println!("Поток 1: {:?}", *s);
    });

    let handle2 = thread::spawn(move || {
        let mut s = state2.lock().unwrap();
        s.eth_balance -= 15.0;
        s.btc_balance += 1.0;
        s.pending_orders -= 1;
        println!("Поток 2: {:?}", *s);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Практический пример: Безопасный торговый движок

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug)]
struct SafeTradingEngine {
    // Все данные под одной блокировкой — нет deadlock
    state: Mutex<EngineState>,
}

#[derive(Debug)]
struct EngineState {
    cash: f64,
    positions: HashMap<String, Position>,
    order_count: u64,
}

impl SafeTradingEngine {
    fn new(initial_cash: f64) -> Self {
        SafeTradingEngine {
            state: Mutex::new(EngineState {
                cash: initial_cash,
                positions: HashMap::new(),
                order_count: 0,
            }),
        }
    }

    fn buy(&self, symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap();

        let cost = quantity * price;
        if state.cash < cost {
            return Err(format!(
                "Недостаточно средств: нужно {}, есть {}",
                cost, state.cash
            ));
        }

        state.cash -= cost;
        state.order_count += 1;
        let order_id = state.order_count;

        state.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                let total_qty = pos.quantity + quantity;
                let total_cost = pos.avg_price * pos.quantity + price * quantity;
                pos.avg_price = total_cost / total_qty;
                pos.quantity = total_qty;
            })
            .or_insert(Position {
                symbol: symbol.to_string(),
                quantity,
                avg_price: price,
            });

        Ok(order_id)
    }

    fn sell(&self, symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap();

        let position = state.positions.get(symbol)
            .ok_or_else(|| format!("Нет позиции по {}", symbol))?;

        if position.quantity < quantity {
            return Err(format!(
                "Недостаточно {}: есть {}, нужно {}",
                symbol, position.quantity, quantity
            ));
        }

        let revenue = quantity * price;
        state.cash += revenue;
        state.order_count += 1;
        let order_id = state.order_count;

        if let Some(pos) = state.positions.get_mut(symbol) {
            pos.quantity -= quantity;
            if pos.quantity <= 0.0 {
                state.positions.remove(symbol);
            }
        }

        Ok(order_id)
    }

    fn get_status(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "Баланс: ${:.2}, Позиций: {}, Ордеров: {}",
            state.cash,
            state.positions.len(),
            state.order_count
        )
    }
}

fn main() {
    let engine = Arc::new(SafeTradingEngine::new(100_000.0));

    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);
    let e3 = Arc::clone(&engine);

    let buyer = thread::spawn(move || {
        for i in 0..10 {
            match e1.buy("BTC", 0.1, 42000.0 + i as f64 * 100.0) {
                Ok(id) => println!("Покупка #{}: ордер {}", i + 1, id),
                Err(e) => println!("Ошибка покупки: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let seller = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(200)); // Ждём накопления позиции
        for i in 0..5 {
            match e2.sell("BTC", 0.1, 43000.0 + i as f64 * 100.0) {
                Ok(id) => println!("Продажа #{}: ордер {}", i + 1, id),
                Err(e) => println!("Ошибка продажи: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    let monitor = thread::spawn(move || {
        for _ in 0..10 {
            println!("Статус: {}", e3.get_status());
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    buyer.join().unwrap();
    seller.join().unwrap();
    monitor.join().unwrap();

    println!("\nИтоговый статус: {}", engine.get_status());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Deadlock | Взаимная блокировка потоков, ожидающих друг друга |
| 4 условия deadlock | Mutual exclusion, Hold and wait, No preemption, Circular wait |
| `try_lock()` | Неблокирующая попытка захвата мьютекса |
| Единый порядок | Всегда блокируем ресурсы в одном порядке |
| Короткие блокировки | Минимизируем время удержания блокировки |
| Группировка данных | Связанные данные под одной блокировкой |

## Домашнее задание

1. **Создание deadlock**: Напиши программу с тремя мьютексами и тремя потоками, которая гарантированно создаёт deadlock. Добавь логирование, чтобы увидеть, в какой момент происходит блокировка.

2. **Исправление deadlock**: Возьми программу из задания 1 и исправь её, используя:
   - Единый порядок блокировки
   - `try_lock()` с повторными попытками

3. **Безопасная биржа**: Реализуй структуру `Exchange` с методами:
   - `place_order(order: Order)` — размещение ордера
   - `cancel_order(order_id: u64)` — отмена ордера
   - `get_order_book()` — получение стакана заявок

   Убедись, что несколько потоков могут безопасно вызывать эти методы без риска deadlock.

4. **Детектор deadlock**: Используя `try_lock()` и счётчик попыток, создай функцию-обёртку, которая:
   - Пытается захватить несколько мьютексов
   - Если не удаётся за N попыток — логирует предупреждение о возможном deadlock
   - Возвращает `Result` с информацией об успехе или ошибке

## Навигация

[← Предыдущий день](../163-rwlock-readers-writer/ru.md) | [Следующий день →](../165-avoiding-deadlock-lock-ordering/ru.md)
