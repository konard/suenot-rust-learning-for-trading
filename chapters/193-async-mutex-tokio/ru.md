# День 193: Async Mutex: tokio::sync::Mutex

## Аналогия из трейдинга

Представь себе торговую площадку, где сотни трейдеров одновременно пытаются обновить общий портфель. В синхронном мире каждый трейдер был бы вынужден ждать в очереди, блокируя весь свой поток работы. Это как если бы трейдер, ожидая доступа к терминалу, не мог даже проверять новости или анализировать графики.

В асинхронном мире с `tokio::sync::Mutex` трейдер **отдаёт ожидание системе**: пока он ждёт доступа к общему ресурсу, система может выполнять другие задачи — получать рыночные данные, обрабатывать сигналы или отправлять уведомления. Как только ресурс освобождается, трейдер мгновенно продолжает работу.

**Ключевое отличие от `std::sync::Mutex`:**
- `std::sync::Mutex` — блокирует весь поток (thread) при ожидании
- `tokio::sync::Mutex` — освобождает поток для других задач, пока ждёт блокировку

## Зачем нужен Async Mutex?

В async-приложениях использование `std::sync::Mutex` может привести к серьёзным проблемам:

```rust
// ПЛОХО: std::sync::Mutex в async-коде
use std::sync::Mutex;

async fn bad_example(mutex: &Mutex<i32>) {
    let guard = mutex.lock().unwrap(); // Блокирует весь поток!

    // Если здесь вызвать .await, другие задачи на этом потоке замрут
    some_async_operation().await; // ОПАСНО!

    // guard всё ещё удерживается
}
```

**Проблема:** когда `std::sync::Mutex` заблокирован, весь поток (thread) блокируется. В async-рантайме типа tokio, где много задач выполняются на ограниченном числе потоков, это может привести к:
- Зависанию других задач на том же потоке
- Потенциальным дедлокам
- Снижению производительности

## Основы tokio::sync::Mutex

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
struct Portfolio {
    cash: f64,
    btc_amount: f64,
    last_update: String,
}

#[tokio::main]
async fn main() {
    // Создаём async mutex с портфелем
    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        btc_amount: 0.0,
        last_update: "init".to_string(),
    }));

    // lock() возвращает Future — нужен .await
    let mut guard = portfolio.lock().await;
    guard.cash -= 42_000.0;
    guard.btc_amount += 1.0;
    guard.last_update = "bought BTC".to_string();

    println!("Портфель: {:?}", *guard);
    // guard освобождается здесь при выходе из области видимости
}
```

## Сравнение std::sync::Mutex и tokio::sync::Mutex

```rust
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // std::sync::Mutex — синхронная блокировка
    let std_mutex = Arc::new(StdMutex::new(100.0_f64));
    {
        let guard = std_mutex.lock().unwrap(); // Блокирует поток!
        println!("std::sync::Mutex: {}", *guard);
    }

    // tokio::sync::Mutex — асинхронная блокировка
    let tokio_mutex = Arc::new(TokioMutex::new(100.0_f64));
    {
        let guard = tokio_mutex.lock().await; // Освобождает поток при ожидании
        println!("tokio::sync::Mutex: {}", *guard);
    }
}
```

## Практический пример: Асинхронный торговый движок

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,      // "buy" или "sell"
    price: f64,
    quantity: f64,
    status: String,    // "pending", "filled", "cancelled"
}

#[derive(Debug)]
struct TradingEngine {
    orders: Mutex<HashMap<u64, Order>>,
    next_order_id: Mutex<u64>,
    balances: Mutex<HashMap<String, f64>>,
}

impl TradingEngine {
    fn new() -> Self {
        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 100_000.0);
        balances.insert("BTC".to_string(), 0.0);
        balances.insert("ETH".to_string(), 0.0);

        TradingEngine {
            orders: Mutex::new(HashMap::new()),
            next_order_id: Mutex::new(1),
            balances: Mutex::new(balances),
        }
    }

    async fn place_order(&self, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Генерируем ID ордера
        let order_id = {
            let mut id_guard = self.next_order_id.lock().await;
            let id = *id_guard;
            *id_guard += 1;
            id
        };

        // Проверяем баланс
        {
            let balances = self.balances.lock().await;
            if side == "buy" {
                let usd_balance = balances.get("USD").unwrap_or(&0.0);
                let required = price * quantity;
                if *usd_balance < required {
                    return Err(format!("Недостаточно USD: нужно {}, есть {}", required, usd_balance));
                }
            } else {
                let asset_balance = balances.get(symbol).unwrap_or(&0.0);
                if *asset_balance < quantity {
                    return Err(format!("Недостаточно {}: нужно {}, есть {}", symbol, quantity, asset_balance));
                }
            }
        }

        // Создаём и сохраняем ордер
        let order = Order {
            id: order_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            status: "pending".to_string(),
        };

        {
            let mut orders = self.orders.lock().await;
            orders.insert(order_id, order);
        }

        println!("Ордер #{} создан: {} {} {} @ ${}", order_id, side, quantity, symbol, price);
        Ok(order_id)
    }

    async fn execute_order(&self, order_id: u64) -> Result<(), String> {
        // Получаем ордер
        let order = {
            let orders = self.orders.lock().await;
            orders.get(&order_id).cloned().ok_or("Ордер не найден")?
        };

        // Обновляем балансы
        {
            let mut balances = self.balances.lock().await;
            let total = order.price * order.quantity;

            if order.side == "buy" {
                *balances.get_mut("USD").unwrap() -= total;
                *balances.entry(order.symbol.clone()).or_insert(0.0) += order.quantity;
            } else {
                *balances.get_mut("USD").unwrap() += total;
                *balances.get_mut(&order.symbol).unwrap() -= order.quantity;
            }
        }

        // Обновляем статус ордера
        {
            let mut orders = self.orders.lock().await;
            if let Some(o) = orders.get_mut(&order_id) {
                o.status = "filled".to_string();
            }
        }

        println!("Ордер #{} исполнен", order_id);
        Ok(())
    }

    async fn get_balance(&self, asset: &str) -> f64 {
        let balances = self.balances.lock().await;
        *balances.get(asset).unwrap_or(&0.0)
    }

    async fn get_all_balances(&self) -> HashMap<String, f64> {
        let balances = self.balances.lock().await;
        balances.clone()
    }
}

#[tokio::main]
async fn main() {
    let engine = Arc::new(TradingEngine::new());

    // Запускаем несколько параллельных торговых операций
    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);
    let e3 = Arc::clone(&engine);

    let trader1 = tokio::spawn(async move {
        for i in 0..3 {
            let price = 42000.0 + i as f64 * 100.0;
            match e1.place_order("BTC", "buy", price, 0.1).await {
                Ok(id) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let _ = e1.execute_order(id).await;
                }
                Err(e) => println!("Ошибка трейдера 1: {}", e),
            }
        }
    });

    let trader2 = tokio::spawn(async move {
        for i in 0..3 {
            let price = 2500.0 + i as f64 * 50.0;
            match e2.place_order("ETH", "buy", price, 1.0).await {
                Ok(id) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
                    let _ = e2.execute_order(id).await;
                }
                Err(e) => println!("Ошибка трейдера 2: {}", e),
            }
        }
    });

    let monitor = tokio::spawn(async move {
        for _ in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let balances = e3.get_all_balances().await;
            println!("Балансы: {:?}", balances);
        }
    });

    // Ждём завершения всех задач
    let _ = tokio::join!(trader1, trader2, monitor);

    println!("\nИтоговые балансы: {:?}", engine.get_all_balances().await);
}
```

## try_lock() — неблокирующая попытка

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
struct MarketData {
    last_price: f64,
    volume_24h: f64,
}

#[tokio::main]
async fn main() {
    let market_data = Arc::new(Mutex::new(MarketData {
        last_price: 42000.0,
        volume_24h: 1_000_000.0,
    }));

    let md1 = Arc::clone(&market_data);
    let md2 = Arc::clone(&market_data);

    // Задача обновления данных
    let updater = tokio::spawn(async move {
        loop {
            {
                let mut data = md1.lock().await;
                data.last_price += 10.0;
                data.volume_24h += 100.0;
                println!("Обновлено: цена = {}", data.last_price);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Задача чтения данных (неблокирующая)
    let reader = tokio::spawn(async move {
        for i in 0..10 {
            // try_lock() не блокирует — сразу возвращает результат
            match md2.try_lock() {
                Ok(data) => {
                    println!("Прочитано (попытка {}): цена = {}, объём = {}",
                             i + 1, data.last_price, data.volume_24h);
                }
                Err(_) => {
                    println!("Попытка {}: данные заблокированы, пропускаем", i + 1);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    // Ждём только reader, updater работает бесконечно
    let _ = reader.await;
    updater.abort();
}
```

## Паттерн: Удержание блокировки через .await

**Важно:** `tokio::sync::Mutex` безопасно удерживать через `.await`:

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

async fn fetch_price(symbol: &str) -> f64 {
    // Имитация HTTP-запроса
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 0.0,
    }
}

#[derive(Debug)]
struct PriceCache {
    prices: std::collections::HashMap<String, f64>,
}

#[tokio::main]
async fn main() {
    let cache = Arc::new(Mutex::new(PriceCache {
        prices: std::collections::HashMap::new(),
    }));

    // Безопасно удерживать tokio::sync::Mutex через .await
    let mut guard = cache.lock().await;

    // Можем делать async-операции, удерживая блокировку
    let btc_price = fetch_price("BTC").await;
    guard.prices.insert("BTC".to_string(), btc_price);

    let eth_price = fetch_price("ETH").await;
    guard.prices.insert("ETH".to_string(), eth_price);

    println!("Кэш цен: {:?}", guard.prices);
    // guard освобождается здесь
}
```

## Когда использовать std::sync::Mutex vs tokio::sync::Mutex

| Сценарий | Рекомендация |
|----------|--------------|
| Короткие операции без `.await` внутри критической секции | `std::sync::Mutex` (быстрее) |
| Нужен `.await` внутри критической секции | `tokio::sync::Mutex` (обязательно) |
| Много конкурентных async-задач | `tokio::sync::Mutex` |
| Синхронный код или минимум async | `std::sync::Mutex` |

```rust
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

// Хорошо: std::sync::Mutex для быстрых операций
async fn quick_update(counter: &Arc<StdMutex<u64>>) {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    // Нет .await внутри — это нормально
}

// Хорошо: tokio::sync::Mutex когда нужен .await
async fn slow_update(cache: &Arc<TokioMutex<String>>) {
    let mut guard = cache.lock().await;
    // Асинхронная операция внутри критической секции
    let new_data = fetch_data().await;
    *guard = new_data;
}

async fn fetch_data() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "updated".to_string()
}
```

## Практический пример: Асинхронный менеджер рисков

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[derive(Debug)]
struct RiskManager {
    positions: Mutex<HashMap<String, Position>>,
    max_loss_percent: f64,
    max_position_size: f64,
}

impl RiskManager {
    fn new(max_loss_percent: f64, max_position_size: f64) -> Self {
        RiskManager {
            positions: Mutex::new(HashMap::new()),
            max_loss_percent,
            max_position_size,
        }
    }

    async fn add_position(&self, symbol: &str, quantity: f64, price: f64) -> Result<(), String> {
        if quantity > self.max_position_size {
            return Err(format!(
                "Превышен максимальный размер позиции: {} > {}",
                quantity, self.max_position_size
            ));
        }

        let mut positions = self.positions.lock().await;
        positions.insert(symbol.to_string(), Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
            current_price: price,
        });

        println!("Позиция открыта: {} {} @ ${}", quantity, symbol, price);
        Ok(())
    }

    async fn update_price(&self, symbol: &str, new_price: f64) -> Option<String> {
        let mut positions = self.positions.lock().await;

        if let Some(pos) = positions.get_mut(symbol) {
            pos.current_price = new_price;
            let pnl_pct = pos.pnl_percent();

            // Проверяем риск-лимит
            if pnl_pct < -self.max_loss_percent {
                let warning = format!(
                    "РИСК-АЛЕРТ: {} потеря {:.2}% превышает лимит -{:.2}%",
                    symbol, pnl_pct.abs(), self.max_loss_percent
                );
                return Some(warning);
            }
        }
        None
    }

    async fn get_total_pnl(&self) -> f64 {
        let positions = self.positions.lock().await;
        positions.values().map(|p| p.pnl()).sum()
    }

    async fn get_positions_report(&self) -> String {
        let positions = self.positions.lock().await;
        let mut report = String::from("=== Отчёт по позициям ===\n");

        for pos in positions.values() {
            report.push_str(&format!(
                "{}: {} @ ${:.2} -> ${:.2} | PnL: ${:.2} ({:+.2}%)\n",
                pos.symbol, pos.quantity, pos.entry_price,
                pos.current_price, pos.pnl(), pos.pnl_percent()
            ));
        }

        report.push_str(&format!("Общий PnL: ${:.2}",
            positions.values().map(|p| p.pnl()).sum::<f64>()));
        report
    }
}

async fn simulate_price_feed(symbol: &str, base_price: f64) -> f64 {
    // Имитация получения цены с биржи
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let change = ((seed % 1000) as f64 - 500.0) / 100.0;
    base_price * (1.0 + change / 100.0)
}

#[tokio::main]
async fn main() {
    let risk_manager = Arc::new(RiskManager::new(5.0, 10.0)); // 5% макс. убыток, 10 макс. размер

    // Открываем позиции
    let rm1 = Arc::clone(&risk_manager);
    rm1.add_position("BTC", 1.0, 42000.0).await.unwrap();
    rm1.add_position("ETH", 5.0, 2500.0).await.unwrap();

    // Запускаем симуляцию ценового фида
    let rm2 = Arc::clone(&risk_manager);
    let price_feed = tokio::spawn(async move {
        for _ in 0..10 {
            let btc_price = simulate_price_feed("BTC", 42000.0).await;
            let eth_price = simulate_price_feed("ETH", 2500.0).await;

            if let Some(alert) = rm2.update_price("BTC", btc_price).await {
                println!("{}", alert);
            }
            if let Some(alert) = rm2.update_price("ETH", eth_price).await {
                println!("{}", alert);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    });

    // Мониторинг позиций
    let rm3 = Arc::clone(&risk_manager);
    let monitor = tokio::spawn(async move {
        for _ in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
            println!("\n{}\n", rm3.get_positions_report().await);
        }
    });

    let _ = tokio::join!(price_feed, monitor);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::sync::Mutex` | Асинхронный мьютекс для async-кода |
| `.lock().await` | Асинхронное получение блокировки |
| `try_lock()` | Неблокирующая попытка получить блокировку |
| vs `std::sync::Mutex` | Можно удерживать через `.await` |
| Освобождение потока | При ожидании поток свободен для других задач |

## Практические задания

1. **Асинхронный кэш котировок**: Реализуй структуру `QuoteCache`, которая:
   - Хранит последние цены для списка символов
   - Обновляет цены асинхронно с разными интервалами
   - Предоставляет метод `get_quote(symbol)` без блокировки основного потока

2. **Параллельный загрузчик ордеров**: Создай систему, которая:
   - Загружает ордера из нескольких "источников" параллельно
   - Сохраняет их в общую структуру с использованием `tokio::sync::Mutex`
   - Показывает прогресс загрузки в реальном времени

3. **Rate Limiter для API**: Реализуй ограничитель запросов:
   - Отслеживает количество запросов за последнюю минуту
   - Блокирует новые запросы при превышении лимита
   - Работает асинхронно и не блокирует другие задачи

## Домашнее задание

1. **Сравнение производительности**: Напиши бенчмарк, который сравнивает `std::sync::Mutex` и `tokio::sync::Mutex` в разных сценариях:
   - Много коротких операций
   - Мало длинных операций с `.await` внутри
   - Высокая конкуренция (много задач, один ресурс)

2. **Асинхронная биржа**: Расширь пример торгового движка:
   - Добавь книгу ордеров (order book) с bid/ask
   - Реализуй матчинг ордеров
   - Добавь WebSocket-подобные уведомления о сделках

3. **Deadlock-детектор**: Создай обёртку над `tokio::sync::Mutex`, которая:
   - Логирует время ожидания блокировки
   - Предупреждает, если блокировка удерживается слишком долго
   - Выводит стек вызовов при подозрении на deadlock

## Навигация

[← Предыдущий день](../192-async-channels-tokio-mpsc/ru.md) | [Следующий день →](../194-async-rwlock-tokio/ru.md)
