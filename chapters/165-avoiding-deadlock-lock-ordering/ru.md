# День 165: Избегаем deadlock: порядок блокировок

## Аналогия из трейдинга

В предыдущей главе мы познакомились с deadlock — взаимной блокировкой потоков. Представь крупную биржу, где торгуются тысячи активов: BTC, ETH, SOL, USDT и другие. Каждый актив имеет свой баланс, и для безопасности операций каждый баланс защищён мьютексом.

Теперь представь двух трейдеров:
- Трейдер A хочет обменять BTC на ETH: сначала блокирует BTC, потом ETH
- Трейдер B хочет обменять ETH на BTC: сначала блокирует ETH, потом BTC

Результат — deadlock! Но есть простое решение: **все должны блокировать активы в одинаковом порядке**. Например, всегда сначала BTC, потом ETH — независимо от направления обмена.

Это похоже на правило дорожного движения: если все едут по правой стороне дороги, столкновений не будет. Порядок блокировки — это "правила дорожного движения" для потоков.

## Что такое Lock Ordering?

**Lock Ordering** (порядок блокировки) — это техника предотвращения deadlock, при которой все потоки захватывают мьютексы в строго определённом, заранее согласованном порядке.

### Почему это работает?

Вспомним четыре условия deadlock:
1. **Mutual Exclusion** — ресурс может быть захвачен только одним потоком
2. **Hold and Wait** — поток удерживает один ресурс и ждёт другой
3. **No Preemption** — ресурс нельзя отобрать принудительно
4. **Circular Wait** — существует цикл ожидания между потоками

Порядок блокировки **устраняет Circular Wait**. Если все потоки блокируют ресурсы A, B, C в порядке A → B → C, то невозможна ситуация, когда один поток держит C и ждёт A (это нарушило бы порядок).

## Проблема: неупорядоченные блокировки

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Портфели для торговли криптовалютой
struct CryptoPortfolio {
    btc: f64,
    eth: f64,
    sol: f64,
}

fn main() {
    let btc_wallet = Arc::new(Mutex::new(10.0_f64));
    let eth_wallet = Arc::new(Mutex::new(100.0_f64));
    let sol_wallet = Arc::new(Mutex::new(500.0_f64));

    let btc1 = Arc::clone(&btc_wallet);
    let eth1 = Arc::clone(&eth_wallet);
    let sol1 = Arc::clone(&sol_wallet);

    let btc2 = Arc::clone(&btc_wallet);
    let eth2 = Arc::clone(&eth_wallet);
    let sol2 = Arc::clone(&sol_wallet);

    // Поток 1: Ребалансировка BTC → ETH → SOL
    let handle1 = thread::spawn(move || {
        println!("Поток 1: Начинаю ребалансировку портфеля...");

        let _btc = btc1.lock().unwrap();
        println!("Поток 1: BTC заблокирован");
        thread::sleep(Duration::from_millis(50));

        let _eth = eth1.lock().unwrap();
        println!("Поток 1: ETH заблокирован");
        thread::sleep(Duration::from_millis(50));

        let _sol = sol1.lock().unwrap();
        println!("Поток 1: SOL заблокирован");

        println!("Поток 1: Ребалансировка завершена!");
    });

    // Поток 2: Ребалансировка SOL → ETH → BTC (ОБРАТНЫЙ порядок!)
    let handle2 = thread::spawn(move || {
        println!("Поток 2: Начинаю ребалансировку портфеля...");

        let _sol = sol2.lock().unwrap();
        println!("Поток 2: SOL заблокирован");
        thread::sleep(Duration::from_millis(50));

        let _eth = eth2.lock().unwrap();
        println!("Поток 2: ETH заблокирован");
        thread::sleep(Duration::from_millis(50));

        let _btc = btc2.lock().unwrap();
        println!("Поток 2: BTC заблокирован");

        println!("Поток 2: Ребалансировка завершена!");
    });

    // DEADLOCK неизбежен!
    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

**Проблема:** Поток 1 блокирует BTC и ждёт SOL, а поток 2 блокирует SOL и ждёт BTC.

## Решение 1: Фиксированный порядок по алфавиту

Самый простой способ — упорядочить ресурсы по алфавиту или другому статическому критерию:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Порядок: BTC < ETH < SOL (по алфавиту)
    let btc_wallet = Arc::new(Mutex::new(10.0_f64));
    let eth_wallet = Arc::new(Mutex::new(100.0_f64));
    let sol_wallet = Arc::new(Mutex::new(500.0_f64));

    let btc1 = Arc::clone(&btc_wallet);
    let eth1 = Arc::clone(&eth_wallet);
    let sol1 = Arc::clone(&sol_wallet);

    let btc2 = Arc::clone(&btc_wallet);
    let eth2 = Arc::clone(&eth_wallet);
    let sol2 = Arc::clone(&sol_wallet);

    // Поток 1: BTC → ETH → SOL (правильный порядок)
    let handle1 = thread::spawn(move || {
        println!("Поток 1: Блокирую в порядке BTC → ETH → SOL");

        let btc = btc1.lock().unwrap();
        let eth = eth1.lock().unwrap();
        let sol = sol1.lock().unwrap();

        println!("Поток 1: BTC={}, ETH={}, SOL={}", *btc, *eth, *sol);
    });

    // Поток 2: BTC → ETH → SOL (ТОТ ЖЕ порядок!)
    let handle2 = thread::spawn(move || {
        println!("Поток 2: Блокирую в порядке BTC → ETH → SOL");

        let btc = btc2.lock().unwrap();
        let eth = eth2.lock().unwrap();
        let sol = sol2.lock().unwrap();

        println!("Поток 2: BTC={}, ETH={}, SOL={}", *btc, *eth, *sol);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Программа успешно завершена — без deadlock!");
}
```

## Решение 2: Порядок по идентификатору (для динамических ресурсов)

В реальных торговых системах активы добавляются динамически. Используем уникальные ID:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::cmp::Ordering;

/// Торговый актив с уникальным ID для упорядочивания
struct TradingAsset {
    id: u64,            // Уникальный ID для порядка блокировки
    symbol: String,
    balance: Mutex<f64>,
}

impl TradingAsset {
    fn new(id: u64, symbol: &str, balance: f64) -> Self {
        TradingAsset {
            id,
            symbol: symbol.to_string(),
            balance: Mutex::new(balance),
        }
    }
}

/// Безопасно блокирует два актива в правильном порядке
fn lock_two_assets<'a>(
    asset1: &'a TradingAsset,
    asset2: &'a TradingAsset,
) -> (
    std::sync::MutexGuard<'a, f64>,
    std::sync::MutexGuard<'a, f64>,
) {
    // Всегда блокируем по возрастанию ID
    match asset1.id.cmp(&asset2.id) {
        Ordering::Less => {
            let guard1 = asset1.balance.lock().unwrap();
            let guard2 = asset2.balance.lock().unwrap();
            (guard1, guard2)
        }
        Ordering::Greater => {
            let guard2 = asset2.balance.lock().unwrap();
            let guard1 = asset1.balance.lock().unwrap();
            (guard1, guard2)
        }
        Ordering::Equal => {
            panic!("Нельзя блокировать один и тот же актив дважды!");
        }
    }
}

/// Безопасный обмен между двумя активами
fn safe_swap(
    from_asset: &TradingAsset,
    to_asset: &TradingAsset,
    amount: f64,
    rate: f64,
) -> Result<(), String> {
    let (mut from_guard, mut to_guard) = lock_two_assets(from_asset, to_asset);

    // Проверяем баланс
    if *from_guard < amount {
        return Err(format!(
            "Недостаточно {}: есть {}, нужно {}",
            from_asset.symbol, *from_guard, amount
        ));
    }

    // Выполняем обмен
    *from_guard -= amount;
    *to_guard += amount * rate;

    println!(
        "Обмен: {} {} → {} {} (курс: {})",
        amount, from_asset.symbol,
        amount * rate, to_asset.symbol,
        rate
    );

    Ok(())
}

fn main() {
    // Создаём активы с уникальными ID
    let btc = Arc::new(TradingAsset::new(1, "BTC", 10.0));
    let eth = Arc::new(TradingAsset::new(2, "ETH", 100.0));
    let sol = Arc::new(TradingAsset::new(3, "SOL", 500.0));

    let btc1 = Arc::clone(&btc);
    let eth1 = Arc::clone(&eth);

    let eth2 = Arc::clone(&eth);
    let sol2 = Arc::clone(&sol);

    let btc3 = Arc::clone(&btc);
    let sol3 = Arc::clone(&sol);

    // Три потока выполняют обмены одновременно
    let h1 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&btc1, &eth1, 0.1, 15.0);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&eth2, &sol2, 1.0, 5.0);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let h3 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&sol3, &btc3, 10.0, 0.002);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();

    println!("\nИтоговые балансы:");
    println!("BTC: {}", *btc.balance.lock().unwrap());
    println!("ETH: {}", *eth.balance.lock().unwrap());
    println!("SOL: {}", *sol.balance.lock().unwrap());
}
```

## Решение 3: Порядок по адресу памяти

Когда у ресурсов нет естественного ID, можно использовать адрес в памяти:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct OrderBook {
    symbol: String,
    bids: Vec<(f64, f64)>, // (цена, количество)
    asks: Vec<(f64, f64)>,
}

struct Portfolio {
    cash: f64,
    positions: Vec<(String, f64)>,
}

/// Блокирует два мьютекса по адресу указателя
fn lock_by_address<'a, T, U>(
    m1: &'a Mutex<T>,
    m2: &'a Mutex<U>,
) -> (std::sync::MutexGuard<'a, T>, std::sync::MutexGuard<'a, U>) {
    let ptr1 = m1 as *const Mutex<T> as usize;
    let ptr2 = m2 as *const Mutex<U> as usize;

    if ptr1 < ptr2 {
        let g1 = m1.lock().unwrap();
        let g2 = m2.lock().unwrap();
        (g1, g2)
    } else {
        let g2 = m2.lock().unwrap();
        let g1 = m1.lock().unwrap();
        (g1, g2)
    }
}

fn execute_market_order(
    order_book: &Mutex<OrderBook>,
    portfolio: &Mutex<Portfolio>,
    symbol: &str,
    quantity: f64,
    is_buy: bool,
) -> Result<f64, String> {
    // Блокируем в порядке адресов — deadlock невозможен!
    let (mut ob, mut pf) = lock_by_address(order_book, portfolio);

    // Находим лучшую цену
    let price = if is_buy {
        ob.asks.first().map(|(p, _)| *p).unwrap_or(0.0)
    } else {
        ob.bids.first().map(|(p, _)| *p).unwrap_or(0.0)
    };

    if price == 0.0 {
        return Err("Нет ордеров в стакане".to_string());
    }

    let cost = price * quantity;

    if is_buy {
        if pf.cash < cost {
            return Err(format!("Недостаточно средств: {} < {}", pf.cash, cost));
        }
        pf.cash -= cost;
        pf.positions.push((symbol.to_string(), quantity));
    } else {
        pf.cash += cost;
        // Удаляем позицию (упрощённо)
    }

    println!(
        "{} {} {} по цене {} = {}",
        if is_buy { "Покупка" } else { "Продажа" },
        quantity,
        symbol,
        price,
        cost
    );

    Ok(price)
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        symbol: "BTC/USDT".to_string(),
        bids: vec![(41000.0, 5.0), (40900.0, 10.0)],
        asks: vec![(41100.0, 3.0), (41200.0, 7.0)],
    }));

    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        positions: vec![],
    }));

    let ob1 = Arc::clone(&order_book);
    let pf1 = Arc::clone(&portfolio);

    let ob2 = Arc::clone(&order_book);
    let pf2 = Arc::clone(&portfolio);

    // Параллельные торговые операции
    let buyer = thread::spawn(move || {
        for _ in 0..3 {
            let _ = execute_market_order(&ob1, &pf1, "BTC", 0.5, true);
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let seller = thread::spawn(move || {
        for _ in 0..3 {
            let _ = execute_market_order(&ob2, &pf2, "BTC", 0.3, false);
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    buyer.join().unwrap();
    seller.join().unwrap();

    println!("\nИтоговый портфель:");
    let pf = portfolio.lock().unwrap();
    println!("Кэш: ${:.2}", pf.cash);
    for (sym, qty) in &pf.positions {
        println!("{}: {}", sym, qty);
    }
}
```

## Решение 4: Иерархическая блокировка

Для сложных систем удобно использовать иерархию уровней:

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Уровни иерархии блокировок (блокируем от низшего к высшему)
/// Level 1: Индивидуальные балансы
/// Level 2: Стаканы заявок
/// Level 3: Риск-менеджмент
/// Level 4: Глобальные настройки

struct TradingSystem {
    // Level 1: Балансы (блокируем первыми)
    btc_balance: Mutex<f64>,
    usdt_balance: Mutex<f64>,

    // Level 2: Стакан заявок
    order_book: Mutex<Vec<(f64, f64, bool)>>, // (цена, кол-во, is_bid)

    // Level 3: Риск-менеджмент
    risk_limits: RwLock<RiskLimits>,

    // Level 4: Глобальные настройки (блокируем последними)
    global_config: RwLock<GlobalConfig>,
}

struct RiskLimits {
    max_position_size: f64,
    max_daily_loss: f64,
    current_daily_pnl: f64,
}

struct GlobalConfig {
    trading_enabled: bool,
    maintenance_mode: bool,
}

impl TradingSystem {
    fn new() -> Self {
        TradingSystem {
            btc_balance: Mutex::new(10.0),
            usdt_balance: Mutex::new(100_000.0),
            order_book: Mutex::new(vec![]),
            risk_limits: RwLock::new(RiskLimits {
                max_position_size: 5.0,
                max_daily_loss: 10_000.0,
                current_daily_pnl: 0.0,
            }),
            global_config: RwLock::new(GlobalConfig {
                trading_enabled: true,
                maintenance_mode: false,
            }),
        }
    }

    /// Размещение ордера с соблюдением иерархии
    fn place_order(&self, price: f64, quantity: f64, is_buy: bool) -> Result<(), String> {
        // Level 4: Проверяем глобальные настройки
        {
            let config = self.global_config.read().unwrap();
            if !config.trading_enabled || config.maintenance_mode {
                return Err("Торговля временно недоступна".to_string());
            }
        }

        // Level 3: Проверяем риск-лимиты
        {
            let risk = self.risk_limits.read().unwrap();
            if quantity > risk.max_position_size {
                return Err(format!(
                    "Превышен лимит позиции: {} > {}",
                    quantity, risk.max_position_size
                ));
            }
        }

        // Level 1: Блокируем балансы
        let cost = price * quantity;
        if is_buy {
            let mut usdt = self.usdt_balance.lock().unwrap();
            if *usdt < cost {
                return Err(format!("Недостаточно USDT: {} < {}", *usdt, cost));
            }
            *usdt -= cost;
        } else {
            let mut btc = self.btc_balance.lock().unwrap();
            if *btc < quantity {
                return Err(format!("Недостаточно BTC: {} < {}", *btc, quantity));
            }
            *btc -= quantity;
        }

        // Level 2: Добавляем в стакан
        {
            let mut book = self.order_book.lock().unwrap();
            book.push((price, quantity, is_buy));
            println!(
                "Ордер размещён: {} {} BTC @ {}",
                if is_buy { "BUY" } else { "SELL" },
                quantity,
                price
            );
        }

        Ok(())
    }

    /// Обновление риск-лимитов (требует write lock)
    fn update_risk_limits(&self, new_max_position: f64) {
        // Level 4 → Level 3 (сначала config, потом risk)
        let config = self.global_config.read().unwrap();
        if config.maintenance_mode {
            println!("В режиме обслуживания — лимиты не изменены");
            return;
        }
        drop(config); // Освобождаем до следующей блокировки

        let mut risk = self.risk_limits.write().unwrap();
        risk.max_position_size = new_max_position;
        println!("Риск-лимиты обновлены: max_position = {}", new_max_position);
    }

    fn get_status(&self) -> String {
        let btc = self.btc_balance.lock().unwrap();
        let usdt = self.usdt_balance.lock().unwrap();
        let orders = self.order_book.lock().unwrap();

        format!(
            "BTC: {:.4}, USDT: {:.2}, Активных ордеров: {}",
            *btc, *usdt, orders.len()
        )
    }
}

fn main() {
    let system = Arc::new(TradingSystem::new());

    let s1 = Arc::clone(&system);
    let s2 = Arc::clone(&system);
    let s3 = Arc::clone(&system);

    let trader1 = thread::spawn(move || {
        for i in 0..5 {
            let price = 42000.0 + i as f64 * 100.0;
            match s1.place_order(price, 0.5, true) {
                Ok(_) => {}
                Err(e) => println!("Ошибка: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    let trader2 = thread::spawn(move || {
        for i in 0..5 {
            let price = 43000.0 - i as f64 * 100.0;
            match s2.place_order(price, 0.3, false) {
                Ok(_) => {}
                Err(e) => println!("Ошибка: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    let admin = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(50));
        s3.update_risk_limits(3.0);
        println!("Статус: {}", s3.get_status());
    });

    trader1.join().unwrap();
    trader2.join().unwrap();
    admin.join().unwrap();

    println!("\nИтоговый статус: {}", system.get_status());
}
```

## Практический пример: Мультивалютная биржа

Создадим полноценный пример биржи с множеством валютных пар:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::BTreeMap;

/// Торговая пара на бирже
#[derive(Debug)]
struct TradingPair {
    id: u64,
    base: String,    // Например: BTC
    quote: String,   // Например: USDT
    price: f64,
    volume_24h: f64,
}

/// Мультивалютная биржа с упорядоченными блокировками
struct MultiCurrencyExchange {
    // BTreeMap автоматически сортирует ключи — используем для порядка
    balances: BTreeMap<String, Arc<Mutex<f64>>>,
    pairs: Vec<Arc<Mutex<TradingPair>>>,
}

impl MultiCurrencyExchange {
    fn new() -> Self {
        let mut balances = BTreeMap::new();

        // Балансы автоматически отсортированы по алфавиту
        balances.insert("BTC".to_string(), Arc::new(Mutex::new(10.0)));
        balances.insert("ETH".to_string(), Arc::new(Mutex::new(100.0)));
        balances.insert("SOL".to_string(), Arc::new(Mutex::new(500.0)));
        balances.insert("USDT".to_string(), Arc::new(Mutex::new(100_000.0)));

        let pairs = vec![
            Arc::new(Mutex::new(TradingPair {
                id: 1,
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
                price: 42000.0,
                volume_24h: 0.0,
            })),
            Arc::new(Mutex::new(TradingPair {
                id: 2,
                base: "ETH".to_string(),
                quote: "USDT".to_string(),
                price: 2200.0,
                volume_24h: 0.0,
            })),
            Arc::new(Mutex::new(TradingPair {
                id: 3,
                base: "SOL".to_string(),
                quote: "USDT".to_string(),
                price: 100.0,
                volume_24h: 0.0,
            })),
        ];

        MultiCurrencyExchange { balances, pairs }
    }

    /// Безопасный обмен между любыми двумя валютами
    fn swap(&self, from: &str, to: &str, amount: f64, rate: f64) -> Result<f64, String> {
        // BTreeMap гарантирует порядок — итерируем и блокируем по порядку
        let from_lock = self.balances.get(from)
            .ok_or_else(|| format!("Неизвестная валюта: {}", from))?;
        let to_lock = self.balances.get(to)
            .ok_or_else(|| format!("Неизвестная валюта: {}", to))?;

        // Блокируем в алфавитном порядке (BTreeMap гарантирует)
        let (mut from_guard, mut to_guard) = if from < to {
            let f = from_lock.lock().unwrap();
            let t = to_lock.lock().unwrap();
            (f, t)
        } else {
            let t = to_lock.lock().unwrap();
            let f = from_lock.lock().unwrap();
            (f, t)
        };

        // Проверяем баланс
        if *from_guard < amount {
            return Err(format!(
                "Недостаточно {}: {} < {}",
                from, *from_guard, amount
            ));
        }

        // Выполняем обмен
        let received = amount * rate;
        *from_guard -= amount;
        *to_guard += received;

        Ok(received)
    }

    /// Получить все балансы (блокирует в правильном порядке)
    fn get_all_balances(&self) -> Vec<(String, f64)> {
        // BTreeMap итерирует в отсортированном порядке
        self.balances
            .iter()
            .map(|(name, lock)| {
                let balance = lock.lock().unwrap();
                (name.clone(), *balance)
            })
            .collect()
    }

    /// Обновить цену пары
    fn update_price(&self, pair_id: u64, new_price: f64) {
        for pair in &self.pairs {
            let mut p = pair.lock().unwrap();
            if p.id == pair_id {
                p.price = new_price;
                break;
            }
        }
    }
}

fn main() {
    let exchange = Arc::new(MultiCurrencyExchange::new());

    let ex1 = Arc::clone(&exchange);
    let ex2 = Arc::clone(&exchange);
    let ex3 = Arc::clone(&exchange);

    // Трейдер 1: Покупает BTC за USDT
    let trader1 = thread::spawn(move || {
        for i in 0..5 {
            match ex1.swap("USDT", "BTC", 4200.0, 1.0 / 42000.0) {
                Ok(received) => println!(
                    "Трейдер 1: Купил {} BTC (операция {})",
                    received, i + 1
                ),
                Err(e) => println!("Трейдер 1: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    // Трейдер 2: Покупает ETH за USDT
    let trader2 = thread::spawn(move || {
        for i in 0..5 {
            match ex2.swap("USDT", "ETH", 2200.0, 1.0 / 2200.0) {
                Ok(received) => println!(
                    "Трейдер 2: Купил {} ETH (операция {})",
                    received, i + 1
                ),
                Err(e) => println!("Трейдер 2: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    // Трейдер 3: Арбитраж BTC ↔ ETH
    let trader3 = thread::spawn(move || {
        for i in 0..5 {
            // Сначала BTC → ETH
            match ex3.swap("BTC", "ETH", 0.1, 15.0) {
                Ok(received) => println!(
                    "Трейдер 3: Обменял 0.1 BTC → {} ETH (операция {})",
                    received, i + 1
                ),
                Err(e) => println!("Трейдер 3: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    trader1.join().unwrap();
    trader2.join().unwrap();
    trader3.join().unwrap();

    println!("\n=== Итоговые балансы ===");
    for (currency, balance) in exchange.get_all_balances() {
        println!("{}: {:.4}", currency, balance);
    }
}
```

## Лучшие практики порядка блокировок

| Практика | Описание | Пример |
|----------|----------|--------|
| Алфавитный порядок | Блокируем ресурсы по алфавиту | BTC → ETH → SOL → USDT |
| По ID | Используем уникальный числовой ID | id=1 → id=2 → id=3 |
| По адресу | Сравниваем адреса указателей | ptr1 < ptr2 |
| Иерархия | Определяем уровни важности | Балансы → Ордера → Риски |
| BTreeMap | Используем сортированную коллекцию | Автоматический порядок |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Lock Ordering | Фиксированный порядок захвата блокировок |
| Circular Wait | Цикл ожидания — одно из условий deadlock |
| Порядок по ID | Использование уникальных идентификаторов |
| Порядок по адресу | Сравнение указателей для определения порядка |
| Иерархическая блокировка | Уровни приоритета ресурсов |
| BTreeMap | Автоматическая сортировка ключей |

## Домашнее задание

1. **Трёхсторонний обмен**: Реализуй функцию `triangular_swap`, которая безопасно обменивает три актива одновременно (A → B → C → A). Используй порядок блокировки по ID.

2. **Торговый движок с очередью**: Создай систему, где:
   - Есть 5 валютных пар
   - Несколько потоков размещают ордера
   - Один поток исполняет ордера из очереди
   - Используй иерархическую блокировку: Orders → Balances → Statistics

3. **Детектор нарушения порядка**: Напиши wrapper для Mutex, который:
   - Запоминает thread-local порядок блокировок
   - В debug-режиме проверяет, что новая блокировка не нарушает порядок
   - Логирует предупреждение при потенциальном deadlock

4. **Биржевой арбитраж**: Реализуй систему для поиска арбитражных возможностей между тремя биржами:
   - Каждая биржа имеет свои балансы
   - Арбитражный бот одновременно работает на всех биржах
   - Используй порядок блокировки по имени биржи (алфавитный)

## Навигация

[← Предыдущий день](../164-deadlock-threads-block/ru.md) | [Следующий день →](../166-condition-variables/ru.md)
