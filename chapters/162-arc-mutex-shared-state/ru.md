# День 162: Arc<Mutex<T>>: общая изменяемая структура

## Аналогия из трейдинга

Представь торговый терминал, где несколько стратегий одновременно работают с **общим портфелем**. Каждая стратегия — это отдельный поток, и все они должны:
- Читать текущий баланс
- Изменять позиции
- Обновлять общую статистику

`Arc<Mutex<T>>` — это как **сейф в офисе трейдеров**:
- `Arc` — множество ключей от кабинета (каждый трейдер может войти)
- `Mutex` — замок на сейфе (только один может открыть сейф одновременно)
- `T` — содержимое сейфа (портфель, баланс, позиции)

## Зачем нужен Arc<Mutex<T>>?

| Примитив | Назначение |
|----------|------------|
| `Arc<T>` | Разделяемое владение между потоками (только чтение) |
| `Mutex<T>` | Эксклюзивный доступ для изменения (один поток) |
| `Arc<Mutex<T>>` | Разделяемый доступ + возможность изменения |

## Базовый пример: общий баланс

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Общий баланс для всех потоков
    let balance = Arc::new(Mutex::new(10000.0_f64));
    let mut handles = vec![];

    // Три стратегии торгуют параллельно
    for strategy_id in 1..=3 {
        let balance_clone = Arc::clone(&balance);

        let handle = thread::spawn(move || {
            // Блокируем мьютекс для изменения
            let mut bal = balance_clone.lock().unwrap();
            let profit = strategy_id as f64 * 100.0;
            *bal += profit;
            println!("Стратегия {}: добавила ${:.2}, баланс: ${:.2}",
                     strategy_id, profit, *bal);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Итоговый баланс: ${:.2}", *balance.lock().unwrap());
}
```

## Структура портфеля

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Portfolio {
    balance: f64,
    positions: HashMap<String, f64>,  // тикер -> количество
    total_pnl: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: HashMap::new(),
            total_pnl: 0.0,
        }
    }

    fn buy(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;
        if cost > self.balance {
            return Err(format!("Недостаточно средств: нужно ${:.2}, есть ${:.2}",
                              cost, self.balance));
        }

        self.balance -= cost;
        *self.positions.entry(ticker.to_string()).or_insert(0.0) += quantity;
        Ok(())
    }

    fn sell(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<f64, String> {
        let position = self.positions.get(ticker).copied().unwrap_or(0.0);
        if quantity > position {
            return Err(format!("Недостаточно {}: нужно {}, есть {}",
                              ticker, quantity, position));
        }

        let revenue = quantity * price;
        self.balance += revenue;
        *self.positions.get_mut(ticker).unwrap() -= quantity;

        // Убираем позицию если она стала нулевой
        if self.positions[ticker] == 0.0 {
            self.positions.remove(ticker);
        }

        Ok(revenue)
    }
}

fn main() {
    let portfolio = Arc::new(Mutex::new(Portfolio::new(100000.0)));
    let mut handles = vec![];

    // Стратегия 1: покупает BTC
    let p1 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p1.lock().unwrap();
        match port.buy("BTC", 0.5, 42000.0) {
            Ok(_) => println!("Стратегия 1: купила 0.5 BTC"),
            Err(e) => println!("Стратегия 1: ошибка - {}", e),
        }
    }));

    // Стратегия 2: покупает ETH
    let p2 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p2.lock().unwrap();
        match port.buy("ETH", 5.0, 2200.0) {
            Ok(_) => println!("Стратегия 2: купила 5 ETH"),
            Err(e) => println!("Стратегия 2: ошибка - {}", e),
        }
    }));

    // Стратегия 3: покупает SOL
    let p3 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p3.lock().unwrap();
        match port.buy("SOL", 100.0, 95.0) {
            Ok(_) => println!("Стратегия 3: купила 100 SOL"),
            Err(e) => println!("Стратегия 3: ошибка - {}", e),
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let final_portfolio = portfolio.lock().unwrap();
    println!("\nИтоговый портфель:");
    println!("Баланс: ${:.2}", final_portfolio.balance);
    println!("Позиции: {:?}", final_portfolio.positions);
}
```

## Паттерн: минимизация времени блокировки

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct OrderBook {
    bids: Vec<(f64, f64)>,  // (цена, объём)
    asks: Vec<(f64, f64)>,
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        bids: vec![(42000.0, 1.5), (41990.0, 2.0)],
        asks: vec![(42010.0, 1.0), (42020.0, 3.0)],
    }));

    let ob = Arc::clone(&order_book);
    let analyzer = thread::spawn(move || {
        // ПЛОХО: долгая блокировка
        // let book = ob.lock().unwrap();
        // thread::sleep(Duration::from_secs(1)); // Анализ
        // println!("Spread: {}", book.asks[0].0 - book.bids[0].0);

        // ХОРОШО: копируем данные и освобождаем блокировку
        let (best_bid, best_ask) = {
            let book = ob.lock().unwrap();
            (book.bids[0].0, book.asks[0].0)
        }; // Блокировка снята!

        // Теперь можем долго анализировать
        thread::sleep(Duration::from_millis(100));
        let spread = best_ask - best_bid;
        println!("Спред: ${:.2}", spread);
    });

    analyzer.join().unwrap();
}
```

## Обработка ошибок при блокировке

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![42000.0, 42100.0, 41900.0]));

    let d1 = Arc::clone(&data);
    let handle = thread::spawn(move || {
        // Использование lock() с обработкой потенциальной паники
        match d1.lock() {
            Ok(mut prices) => {
                prices.push(42050.0);
                println!("Добавлена новая цена");
            }
            Err(poisoned) => {
                // Mutex отравлен из-за паники в другом потоке
                println!("Mutex отравлен, восстанавливаем данные");
                let mut prices = poisoned.into_inner();
                prices.clear();
                prices.push(42000.0);
            }
        }
    });

    handle.join().unwrap();

    // try_lock() - неблокирующая попытка захвата
    match data.try_lock() {
        Ok(prices) => println!("Цены: {:?}", *prices),
        Err(_) => println!("Mutex занят, попробуем позже"),
    }
}
```

## Практический пример: агрегатор цен

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceData {
    ticker: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

struct PriceAggregator {
    prices: HashMap<String, PriceData>,
    update_count: u64,
}

use std::collections::HashMap;

impl PriceAggregator {
    fn new() -> Self {
        PriceAggregator {
            prices: HashMap::new(),
            update_count: 0,
        }
    }

    fn update_price(&mut self, data: PriceData) {
        self.prices.insert(data.ticker.clone(), data);
        self.update_count += 1;
    }

    fn get_price(&self, ticker: &str) -> Option<f64> {
        self.prices.get(ticker).map(|d| d.price)
    }

    fn get_all_prices(&self) -> Vec<(String, f64)> {
        self.prices
            .iter()
            .map(|(k, v)| (k.clone(), v.price))
            .collect()
    }
}

fn main() {
    let aggregator = Arc::new(Mutex::new(PriceAggregator::new()));
    let mut handles = vec![];

    // Поток обновления BTC
    let agg1 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for i in 0..5 {
            let price = 42000.0 + (i as f64 * 10.0);
            {
                let mut agg = agg1.lock().unwrap();
                agg.update_price(PriceData {
                    ticker: "BTC".to_string(),
                    price,
                    volume: 1.5,
                    timestamp: i as u64,
                });
            }
            thread::sleep(Duration::from_millis(50));
        }
    }));

    // Поток обновления ETH
    let agg2 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for i in 0..5 {
            let price = 2200.0 + (i as f64 * 5.0);
            {
                let mut agg = agg2.lock().unwrap();
                agg.update_price(PriceData {
                    ticker: "ETH".to_string(),
                    price,
                    volume: 10.0,
                    timestamp: i as u64,
                });
            }
            thread::sleep(Duration::from_millis(50));
        }
    }));

    // Поток чтения цен
    let agg3 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(30));
            let agg = agg3.lock().unwrap();
            println!("Текущие цены: {:?}", agg.get_all_prices());
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let final_agg = aggregator.lock().unwrap();
    println!("\nВсего обновлений: {}", final_agg.update_count);
}
```

## Паттерн: статистика сделок

```rust
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Default)]
struct TradeStats {
    total_trades: u64,
    winning_trades: u64,
    losing_trades: u64,
    total_pnl: f64,
    max_profit: f64,
    max_loss: f64,
}

impl TradeStats {
    fn record_trade(&mut self, pnl: f64) {
        self.total_trades += 1;
        self.total_pnl += pnl;

        if pnl > 0.0 {
            self.winning_trades += 1;
            if pnl > self.max_profit {
                self.max_profit = pnl;
            }
        } else if pnl < 0.0 {
            self.losing_trades += 1;
            if pnl < self.max_loss {
                self.max_loss = pnl;
            }
        }
    }

    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.total_trades as f64) * 100.0
    }
}

fn main() {
    let stats = Arc::new(Mutex::new(TradeStats::default()));
    let mut handles = vec![];

    // Симуляция нескольких торговых потоков
    for thread_id in 0..3 {
        let stats_clone = Arc::clone(&stats);

        handles.push(thread::spawn(move || {
            let trades = vec![150.0, -50.0, 200.0, -30.0, 100.0];

            for pnl in trades {
                let adjusted_pnl = pnl * (thread_id as f64 + 1.0);
                let mut s = stats_clone.lock().unwrap();
                s.record_trade(adjusted_pnl);
                println!("Поток {}: сделка ${:.2}", thread_id, adjusted_pnl);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_stats = stats.lock().unwrap();
    println!("\n=== Итоговая статистика ===");
    println!("Всего сделок: {}", final_stats.total_trades);
    println!("Выигрышных: {}", final_stats.winning_trades);
    println!("Убыточных: {}", final_stats.losing_trades);
    println!("Win Rate: {:.1}%", final_stats.win_rate());
    println!("Общий PnL: ${:.2}", final_stats.total_pnl);
    println!("Макс. прибыль: ${:.2}", final_stats.max_profit);
    println!("Макс. убыток: ${:.2}", final_stats.max_loss);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Arc::new(Mutex::new(T))` | Создание разделяемых изменяемых данных |
| `Arc::clone(&arc)` | Получение новой ссылки для потока |
| `.lock().unwrap()` | Блокировка и получение доступа |
| `try_lock()` | Неблокирующая попытка захвата |
| Минимизация блокировки | Копирование данных перед освобождением |
| Poisoned Mutex | Обработка паники в другом потоке |

## Важные правила

1. **Минимизируйте время блокировки** — копируйте данные и освобождайте мьютекс
2. **Избегайте вложенных блокировок** — это ведёт к deadlock
3. **Используйте `try_lock()`** когда не критично получить доступ немедленно
4. **Обрабатывайте poisoned mutex** в production-коде

## Домашнее задание

1. Создай структуру `SharedOrderManager` с методами `place_order()`, `cancel_order()`, `get_orders()` для многопоточного управления ордерами

2. Реализуй `RiskManager` с `Arc<Mutex<T>>`, который проверяет лимиты перед каждой сделкой и обновляет использованный риск

3. Напиши симулятор маркет-мейкера, где один поток обновляет котировки, а другие потоки "торгуют" по этим котировкам

4. Создай систему логирования сделок, где несколько стратегий пишут в общий лог через `Arc<Mutex<Vec<TradeLog>>>`

## Навигация

[← Предыдущий день](../161-arc-shared-access/ru.md) | [Следующий день →](../163-rwlock-readers-writer/ru.md)
