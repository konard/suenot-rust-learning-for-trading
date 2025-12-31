# День 163: RwLock — много читателей, один писатель

## Торговая аналогия

Представьте торговый терминал, отображающий рыночные данные в реальном времени:
- **Сотни трейдеров** одновременно смотрят на книгу заявок — они только **читают** цены
- **Один маркетмейкер** периодически обновляет котировки bid/ask — ему нужно **записывать** новые цены
- Чтение не блокирует других читателей — все могут видеть цены одновременно
- Запись требует эксклюзивного доступа — никто не должен читать устаревшие данные во время обновления цен

Именно это обеспечивает `RwLock` (Read-Write Lock, блокировка чтения-записи):
- **Множество одновременных читателей** — отлично для аналитики, графиков, отображения
- **Эксклюзивный доступ для писателя** — гарантирует целостность данных при обновлении
- **Более высокая пропускная способность** чем у Mutex, когда чтений намного больше, чем записей

В торговых системах нагрузка с преобладанием чтения — обычное дело:
- Ценовые потоки читаются многими компонентами, но обновляются одним
- Балансы портфеля проверяются постоянно, но изменяются редко
- Книги заявок отображаются многим пользователям, но обновляются движком сопоставления

## Что такое RwLock?

`RwLock<T>` — это примитив синхронизации, который позволяет:
- **Много одновременных читателей** (блокировки `read()`)
- **Одного эксклюзивного писателя** (блокировка `write()`)

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // Чтение (общий доступ)
    {
        let read_guard = price.read().unwrap();
        println!("Текущая цена: ${:.2}", *read_guard);
    } // блокировка чтения освобождается здесь

    // Запись (эксклюзивный доступ)
    {
        let mut write_guard = price.write().unwrap();
        *write_guard = 42500.0;
        println!("Цена обновлена до: ${:.2}", *write_guard);
    } // блокировка записи освобождается здесь
}
```

## RwLock vs Mutex

| Особенность | `Mutex<T>` | `RwLock<T>` |
|-------------|------------|-------------|
| Читатели | По одному | Много одновременно |
| Писатели | По одному | По одному |
| Применение | Частые записи | Редкие записи, много чтений |
| Накладные расходы | Ниже | Выше |
| Пример в трейдинге | Исполнение ордеров | Отображение цен |

```rust
use std::sync::{Arc, RwLock, Mutex};
use std::thread;

fn main() {
    // RwLock - лучше, когда много потоков читают, мало пишут
    let market_data = Arc::new(RwLock::new(42000.0));

    // Mutex - проще, когда соотношение чтения/записи примерно равное
    let order_count = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // Много читателей получают доступ к цене
    for i in 0..5 {
        let data = Arc::clone(&market_data);
        handles.push(thread::spawn(move || {
            let price = data.read().unwrap();
            println!("Читатель {}: цена = ${:.2}", i, *price);
        }));
    }

    // Один писатель обновляет цену
    let data = Arc::clone(&market_data);
    handles.push(thread::spawn(move || {
        let mut price = data.write().unwrap();
        *price = 42100.0;
        println!("Писатель: обновил цену до ${:.2}", *price);
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Практический пример: монитор цен в реальном времени

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

struct MarketPrice {
    symbol: String,
    bid: f64,
    ask: f64,
    last_update: u64,
}

impl MarketPrice {
    fn new(symbol: &str, bid: f64, ask: f64) -> Self {
        MarketPrice {
            symbol: symbol.to_string(),
            bid,
            ask,
            last_update: 0,
        }
    }

    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║   МОНИТОР ЦЕН В РЕАЛЬНОМ ВРЕМЕНИ      ║");
    println!("╚═══════════════════════════════════════╝\n");

    let btc_price = Arc::new(RwLock::new(MarketPrice::new("BTC/USD", 41950.0, 42050.0)));

    let mut handles = vec![];

    // Симулятор ценового потока (писатель)
    let price_feed = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for tick in 1..=3 {
            thread::sleep(Duration::from_millis(100));
            let mut price = price_feed.write().unwrap();
            price.bid += 25.0;
            price.ask += 25.0;
            price.last_update = tick;
            println!("[ПОТОК] Тик {}: Обновлено до bid=${:.2}, ask=${:.2}",
                     tick, price.bid, price.ask);
        }
    }));

    // Отображение графика (читатель 1)
    let chart_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(50));
            let price = chart_data.read().unwrap();
            println!("[ГРАФИК] Средняя цена: ${:.2}", price.mid_price());
        }
    }));

    // Анализатор спреда (читатель 2)
    let spread_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(60));
            let price = spread_data.read().unwrap();
            println!("[СПРЕД] Текущий спред: ${:.2}", price.spread());
        }
    }));

    // Система оповещений (читатель 3)
    let alert_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(70));
            let price = alert_data.read().unwrap();
            if price.mid_price() > 42000.0 {
                println!("[АЛЕРТ] Цена выше $42,000!");
            }
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n--- Финальное состояние ---");
    let final_price = btc_price.read().unwrap();
    println!("Символ: {}", final_price.symbol);
    println!("Bid: ${:.2}", final_price.bid);
    println!("Ask: ${:.2}", final_price.ask);
    println!("Спред: ${:.2}", final_price.spread());
}
```

## Практический пример: трекер портфеля

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;

struct Portfolio {
    holdings: HashMap<String, f64>,
    total_value: f64,
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            holdings: HashMap::new(),
            total_value: 0.0,
        }
    }

    fn add_position(&mut self, symbol: &str, value: f64) {
        *self.holdings.entry(symbol.to_string()).or_insert(0.0) += value;
        self.recalculate_total();
    }

    fn recalculate_total(&mut self) {
        self.total_value = self.holdings.values().sum();
    }

    fn get_allocation(&self, symbol: &str) -> f64 {
        if self.total_value == 0.0 {
            return 0.0;
        }
        let holding = self.holdings.get(symbol).unwrap_or(&0.0);
        (holding / self.total_value) * 100.0
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║         ТРЕКЕР ПОРТФЕЛЯ               ║");
    println!("╚═══════════════════════════════════════╝\n");

    let portfolio = Arc::new(RwLock::new(Portfolio::new()));

    // Инициализация портфеля
    {
        let mut p = portfolio.write().unwrap();
        p.add_position("BTC", 50000.0);
        p.add_position("ETH", 30000.0);
        p.add_position("SOL", 20000.0);
    }

    let mut handles = vec![];

    // Множество читателей дашборда
    for i in 1..=3 {
        let p = Arc::clone(&portfolio);
        handles.push(thread::spawn(move || {
            let portfolio = p.read().unwrap();
            println!("[Дашборд {}] Общая стоимость: ${:.2}", i, portfolio.total_value);
            println!("[Дашборд {}] Доля BTC: {:.1}%", i, portfolio.get_allocation("BTC"));
        }));
    }

    // Анализатор рисков (читатель)
    let risk_portfolio = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let p = risk_portfolio.read().unwrap();
        let btc_alloc = p.get_allocation("BTC");
        if btc_alloc > 40.0 {
            println!("[РИСК] Внимание: концентрация BTC составляет {:.1}%", btc_alloc);
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Практический пример: просмотр книги заявок

```rust
use std::sync::{Arc, RwLock};
use std::thread;

struct OrderBook {
    bids: Vec<(f64, f64)>, // (цена, количество)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: vec![
                (41990.0, 1.5),
                (41980.0, 2.3),
                (41970.0, 0.8),
            ],
            asks: vec![
                (42010.0, 1.2),
                (42020.0, 1.8),
                (42030.0, 2.1),
            ],
        }
    }

    fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.first().copied()
    }

    fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.first().copied()
    }

    fn spread(&self) -> f64 {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => ask - bid,
            _ => 0.0,
        }
    }

    fn update_bid(&mut self, price: f64, quantity: f64) {
        if let Some(pos) = self.bids.iter().position(|(p, _)| *p == price) {
            self.bids[pos].1 = quantity;
        } else {
            self.bids.push((price, quantity));
            self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        }
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       ПРОСМОТР КНИГИ ЗАЯВОК           ║");
    println!("╚═══════════════════════════════════════╝\n");

    let order_book = Arc::new(RwLock::new(OrderBook::new()));

    let mut handles = vec![];

    // Множество UI-читателей
    for i in 1..=4 {
        let book = Arc::clone(&order_book);
        handles.push(thread::spawn(move || {
            let ob = book.read().unwrap();
            if let Some((price, qty)) = ob.best_bid() {
                println!("[UI {}] Лучший Bid: ${:.2} x {:.2}", i, price, qty);
            }
            if let Some((price, qty)) = ob.best_ask() {
                println!("[UI {}] Лучший Ask: ${:.2} x {:.2}", i, price, qty);
            }
            println!("[UI {}] Спред: ${:.2}", i, ob.spread());
        }));
    }

    // Движок сопоставления (писатель) - обновляет книгу заявок
    let book = Arc::clone(&order_book);
    handles.push(thread::spawn(move || {
        let mut ob = book.write().unwrap();
        ob.update_bid(41995.0, 3.0);
        println!("[ДВИЖОК] Обновлен bid: $41995.00 x 3.00");
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## try_read и try_write

Неблокирующие альтернативы, которые возвращаются немедленно:

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // Получаем блокировку записи
    let _write_guard = price.write().unwrap();

    // try_read возвращает Err, если удерживается блокировка записи
    match price.try_read() {
        Ok(guard) => println!("Цена: ${:.2}", *guard),
        Err(_) => println!("Нельзя читать - идёт запись"),
    }

    // try_write возвращает Err, если удерживается любая блокировка
    match price.try_write() {
        Ok(mut guard) => *guard = 42500.0,
        Err(_) => println!("Нельзя писать - блокировка удерживается"),
    }
}
```

## Практический пример: кэш торговых сигналов

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
struct Signal {
    symbol: String,
    direction: String,
    strength: f64,
    timestamp: u64,
}

struct SignalCache {
    signals: HashMap<String, Signal>,
    last_update: u64,
}

impl SignalCache {
    fn new() -> Self {
        SignalCache {
            signals: HashMap::new(),
            last_update: 0,
        }
    }

    fn get_signal(&self, symbol: &str) -> Option<&Signal> {
        self.signals.get(symbol)
    }

    fn update_signal(&mut self, signal: Signal) {
        self.last_update += 1;
        self.signals.insert(signal.symbol.clone(), signal);
    }

    fn all_signals(&self) -> Vec<&Signal> {
        self.signals.values().collect()
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     КЭШ ТОРГОВЫХ СИГНАЛОВ             ║");
    println!("╚═══════════════════════════════════════╝\n");

    let cache = Arc::new(RwLock::new(SignalCache::new()));

    // Инициализируем несколькими сигналами
    {
        let mut c = cache.write().unwrap();
        c.update_signal(Signal {
            symbol: "BTC".to_string(),
            direction: "LONG".to_string(),
            strength: 0.8,
            timestamp: 1,
        });
        c.update_signal(Signal {
            symbol: "ETH".to_string(),
            direction: "SHORT".to_string(),
            strength: 0.6,
            timestamp: 1,
        });
    }

    let mut handles = vec![];

    // Генератор сигналов (писатель)
    let writer_cache = Arc::clone(&cache);
    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let mut c = writer_cache.write().unwrap();
        c.update_signal(Signal {
            symbol: "SOL".to_string(),
            direction: "LONG".to_string(),
            strength: 0.9,
            timestamp: 2,
        });
        println!("[ГЕНЕРАТОР] Новый сигнал: SOL LONG (0.9)");
    }));

    // Множество читателей стратегий
    for i in 1..=3 {
        let reader_cache = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            let c = reader_cache.read().unwrap();
            for signal in c.all_signals() {
                println!("[Стратегия {}] {} {} (сила: {:.1})",
                         i, signal.symbol, signal.direction, signal.strength);
            }
        }));
    }

    // Монитор рисков (читатель)
    let risk_cache = Arc::clone(&cache);
    handles.push(thread::spawn(move || {
        let c = risk_cache.read().unwrap();
        let strong_signals: Vec<_> = c.all_signals()
            .into_iter()
            .filter(|s| s.strength > 0.7)
            .collect();
        println!("[РИСК] Количество сильных сигналов: {}", strong_signals.len());
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Распространённые паттерны

### Паттерн 1: конфигурация с преобладанием чтения

```rust
use std::sync::{Arc, RwLock};

struct TradingConfig {
    max_position_size: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
}

fn main() {
    let config = Arc::new(RwLock::new(TradingConfig {
        max_position_size: 10000.0,
        risk_per_trade: 0.02,
        allowed_symbols: vec!["BTC".to_string(), "ETH".to_string()],
    }));

    // Много читателей проверяют конфигурацию
    let cfg = config.read().unwrap();
    println!("Макс. позиция: ${:.2}", cfg.max_position_size);
    println!("Риск на сделку: {:.1}%", cfg.risk_per_trade * 100.0);

    // Редкое обновление конфигурации
    drop(cfg); // Сначала освобождаем блокировку чтения
    {
        let mut cfg = config.write().unwrap();
        cfg.allowed_symbols.push("SOL".to_string());
        println!("SOL добавлен в разрешённые символы");
    }
}
```

### Паттерн 2: снимок для длительных вычислений

```rust
use std::sync::{Arc, RwLock};

struct MarketData {
    prices: Vec<f64>,
}

fn main() {
    let data = Arc::new(RwLock::new(MarketData {
        prices: vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0],
    }));

    // Делаем снимок для вычислений
    let prices_snapshot: Vec<f64>;
    {
        let d = data.read().unwrap();
        prices_snapshot = d.prices.clone(); // Клонируем и освобождаем блокировку
    }

    // Длительные вычисления без удержания блокировки
    let avg: f64 = prices_snapshot.iter().sum::<f64>() / prices_snapshot.len() as f64;
    let variance: f64 = prices_snapshot.iter()
        .map(|p| (p - avg).powi(2))
        .sum::<f64>() / prices_snapshot.len() as f64;
    let std_dev = variance.sqrt();

    println!("Среднее: ${:.2}", avg);
    println!("Станд. откл.: ${:.2}", std_dev);
}
```

## Потенциальные дедлоки с RwLock

Будьте осторожны с порядком блокировок:

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // ОПАСНО: Не повышайте чтение до записи!
    // Это приведёт к дедлоку:
    // let read_guard = price.read().unwrap();
    // let write_guard = price.write().unwrap(); // ДЕДЛОК!

    // Правильный подход: освободить чтение, затем получить запись
    {
        let read_guard = price.read().unwrap();
        let current = *read_guard;
        drop(read_guard); // Освобождаем блокировку чтения

        let mut write_guard = price.write().unwrap();
        *write_guard = current + 100.0;
    }

    println!("Цена: ${:.2}", *price.read().unwrap());
}
```

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| `RwLock<T>` | Блокировка, позволяющая много читателей ИЛИ одного писателя |
| `read()` | Получить общую блокировку чтения |
| `write()` | Получить эксклюзивную блокировку записи |
| `try_read()` | Неблокирующая попытка чтения |
| `try_write()` | Неблокирующая попытка записи |
| `Arc<RwLock<T>>` | Потокобезопасный общий RwLock |

## Когда использовать RwLock vs Mutex

- Используйте `RwLock`, когда чтений значительно больше, чем записей (10:1 или более)
- Используйте `Mutex`, когда соотношение чтения/записи сбалансировано
- `RwLock` имеет более высокие накладные расходы на операцию
- `Mutex` проще и избегает голодания писателей

## Домашнее задание

1. **Агрегатор цен**: Создайте систему, где один поток обновляет цены для нескольких активов, а несколько аналитических потоков читают и вычисляют корреляции. Используйте `RwLock<HashMap<String, f64>>` для кэша цен.

2. **Менеджер конфигурации стратегии**: Создайте торговую стратегию, которая часто читает свою конфигурацию (лимиты риска, размеры позиций), но позволяет обновлять конфигурацию во время работы. Реализуйте безопасную горячую перезагрузку конфигурации.

3. **Кэш глубины книги заявок**: Реализуйте кэш глубины книги заявок, который читается несколькими компонентами отображения (веб-интерфейс, CLI, алерты) и обновляется единственным потоком рыночных данных. Включите методы для получения топ-N уровней и расчёта общей ликвидности.

4. **Трекер состояния сессии**: Создайте трекер торговой сессии, который читается несколькими компонентами (калькулятор P&L, монитор рисков, отчётность), пока единственный менеджер ордеров обновляет его. Отслеживайте открытые позиции, реализованный P&L и количество сделок.

## Навигация

[← Предыдущий день](../162-arc-mutex-shared-mutable-structure/ru.md) | [Следующий день →](../164-deadlock-when-threads-block/ru.md)
