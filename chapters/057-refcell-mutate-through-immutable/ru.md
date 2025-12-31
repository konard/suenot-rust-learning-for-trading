# День 57: RefCell — изменяй через неизменяемую ссылку

## Аналогия из трейдинга

Представь торговый терминал в брокерской компании. Терминал установлен на рабочем месте трейдера и **выглядит как обычное приложение** (неизменяемая ссылка — все видят одинаковый интерфейс). Однако **внутри** терминала происходит постоянное обновление: котировки меняются, позиции открываются и закрываются, баланс обновляется.

Снаружи терминал выглядит "неизменным" — это просто окно на экране. Но **внутренняя мутация** происходит постоянно. Это и есть суть `RefCell` — возможность изменять данные через неизменяемую ссылку, проверяя правила заимствования **во время выполнения**, а не во время компиляции.

## Проблема: Rust слишком строг?

Обычные правила заимствования Rust проверяются **во время компиляции**:
- Либо одна изменяемая ссылка (`&mut`)
- Либо много неизменяемых ссылок (`&`)
- Но не одновременно

Иногда эти правила слишком строги. Например:

```rust
// Это НЕ скомпилируется!
struct Portfolio {
    balance: f64,
}

impl Portfolio {
    fn get_balance(&self) -> f64 {
        self.balance
    }

    fn update_balance(&self, new_balance: f64) {
        // Ошибка! self — неизменяемая ссылка
        // self.balance = new_balance;
    }
}
```

## Что такое RefCell?

`RefCell<T>` — это умный указатель, который:
1. Хранит данные типа `T`
2. Позволяет **динамически** заимствовать данные
3. Проверяет правила заимствования **во время выполнения**
4. Вызывает панику при нарушении правил

```rust
use std::cell::RefCell;

fn main() {
    let price = RefCell::new(42000.0);

    // Неизменяемое заимствование
    println!("Цена: {}", *price.borrow());

    // Изменяемое заимствование
    *price.borrow_mut() += 100.0;

    println!("Новая цена: {}", *price.borrow());
}
```

## Основные методы RefCell

```rust
use std::cell::RefCell;

fn main() {
    let balance = RefCell::new(10000.0);

    // borrow() — неизменяемое заимствование (Ref<T>)
    {
        let b = balance.borrow();
        println!("Баланс: {}", *b);
    } // Ref выходит из области видимости

    // borrow_mut() — изменяемое заимствование (RefMut<T>)
    {
        let mut b = balance.borrow_mut();
        *b += 500.0;
        println!("Новый баланс: {}", *b);
    } // RefMut выходит из области видимости

    // into_inner() — извлечение значения
    let final_balance = balance.into_inner();
    println!("Финальный баланс: {}", final_balance);
}
```

## Паника при нарушении правил

```rust
use std::cell::RefCell;

fn main() {
    let price = RefCell::new(42000.0);

    let borrow1 = price.borrow();

    // ПАНИКА! Уже есть неизменяемое заимствование
    // let borrow2 = price.borrow_mut();

    println!("Цена: {}", *borrow1);

    // После того как borrow1 выйдет из области видимости,
    // можно снова заимствовать
    drop(borrow1);

    let mut borrow2 = price.borrow_mut();
    *borrow2 = 43000.0;
    println!("Новая цена: {}", *borrow2);
}
```

## Безопасная проверка: try_borrow

```rust
use std::cell::RefCell;

fn main() {
    let balance = RefCell::new(10000.0);

    let borrow1 = balance.borrow();

    // try_borrow_mut вернёт Err вместо паники
    match balance.try_borrow_mut() {
        Ok(mut b) => {
            *b += 100.0;
        }
        Err(_) => {
            println!("Не удалось получить изменяемую ссылку — данные уже заимствованы");
        }
    }

    println!("Баланс: {}", *borrow1);
}
```

## Практический пример: торговый аккаунт

```rust
use std::cell::RefCell;

struct TradingAccount {
    name: String,
    balance: RefCell<f64>,
    open_positions: RefCell<Vec<Position>>,
}

#[derive(Clone, Debug)]
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

impl TradingAccount {
    fn new(name: &str, initial_balance: f64) -> Self {
        TradingAccount {
            name: name.to_string(),
            balance: RefCell::new(initial_balance),
            open_positions: RefCell::new(Vec::new()),
        }
    }

    // &self — неизменяемая ссылка, но мы можем изменять balance
    fn deposit(&self, amount: f64) {
        *self.balance.borrow_mut() += amount;
        println!("Депозит: +${:.2}", amount);
    }

    fn withdraw(&self, amount: f64) -> bool {
        let mut balance = self.balance.borrow_mut();
        if *balance >= amount {
            *balance -= amount;
            println!("Вывод: -${:.2}", amount);
            true
        } else {
            println!("Недостаточно средств!");
            false
        }
    }

    fn open_position(&self, symbol: &str, size: f64, price: f64) {
        let cost = size * price;

        // Сначала проверяем баланс
        {
            let balance = self.balance.borrow();
            if *balance < cost {
                println!("Недостаточно средств для открытия позиции");
                return;
            }
        }

        // Теперь изменяем баланс и добавляем позицию
        *self.balance.borrow_mut() -= cost;

        self.open_positions.borrow_mut().push(Position {
            symbol: symbol.to_string(),
            size,
            entry_price: price,
        });

        println!("Открыта позиция: {} x {} @ {}", symbol, size, price);
    }

    fn get_balance(&self) -> f64 {
        *self.balance.borrow()
    }

    fn print_status(&self) {
        println!("\n=== {} ===", self.name);
        println!("Баланс: ${:.2}", self.balance.borrow());
        println!("Открытые позиции:");
        for pos in self.open_positions.borrow().iter() {
            println!("  {} x {} @ ${:.2}", pos.symbol, pos.size, pos.entry_price);
        }
    }
}

fn main() {
    let account = TradingAccount::new("Мой аккаунт", 100000.0);

    account.print_status();

    account.deposit(5000.0);
    account.open_position("BTC/USDT", 0.5, 42000.0);
    account.open_position("ETH/USDT", 2.0, 2500.0);

    account.print_status();

    println!("\nТекущий баланс: ${:.2}", account.get_balance());
}
```

## RefCell + Rc: общие изменяемые данные

Комбинация `Rc<RefCell<T>>` позволяет иметь **несколько владельцев** изменяемых данных:

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, size)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, size: f64) {
        self.bids.push((price, size));
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    fn add_ask(&mut self, price: f64, size: f64) {
        self.asks.push((price, size));
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.first().copied()
    }

    fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.first().copied()
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.0 - bid.0),
            _ => None,
        }
    }
}

struct MarketDataFeed {
    order_book: Rc<RefCell<OrderBook>>,
}

struct TradingStrategy {
    order_book: Rc<RefCell<OrderBook>>,
    name: String,
}

impl MarketDataFeed {
    fn new(order_book: Rc<RefCell<OrderBook>>) -> Self {
        MarketDataFeed { order_book }
    }

    fn update_bid(&self, price: f64, size: f64) {
        self.order_book.borrow_mut().add_bid(price, size);
    }

    fn update_ask(&self, price: f64, size: f64) {
        self.order_book.borrow_mut().add_ask(price, size);
    }
}

impl TradingStrategy {
    fn new(name: &str, order_book: Rc<RefCell<OrderBook>>) -> Self {
        TradingStrategy {
            order_book,
            name: name.to_string(),
        }
    }

    fn analyze(&self) {
        let book = self.order_book.borrow();
        println!("\n[{}] Анализ стакана:", self.name);

        if let Some((price, size)) = book.best_bid() {
            println!("  Лучший bid: {} x {}", price, size);
        }
        if let Some((price, size)) = book.best_ask() {
            println!("  Лучший ask: {} x {}", price, size);
        }
        if let Some(spread) = book.spread() {
            println!("  Спред: {:.2}", spread);
        }
    }
}

fn main() {
    // Общий стакан заявок
    let order_book = Rc::new(RefCell::new(OrderBook::new()));

    // Создаём фид данных и стратегии — все используют один стакан
    let feed = MarketDataFeed::new(Rc::clone(&order_book));
    let strategy1 = TradingStrategy::new("Арбитраж", Rc::clone(&order_book));
    let strategy2 = TradingStrategy::new("Маркет-мейкинг", Rc::clone(&order_book));

    // Фид обновляет стакан
    feed.update_bid(42000.0, 1.5);
    feed.update_bid(41990.0, 2.0);
    feed.update_bid(41980.0, 3.5);

    feed.update_ask(42010.0, 1.0);
    feed.update_ask(42020.0, 2.5);
    feed.update_ask(42030.0, 1.8);

    // Стратегии анализируют те же данные
    strategy1.analyze();
    strategy2.analyze();

    // Добавляем ещё данных
    feed.update_bid(42005.0, 0.5);

    println!("\n--- После обновления ---");
    strategy1.analyze();
}
```

## Практический пример: кеш рыночных данных

```rust
use std::cell::RefCell;
use std::collections::HashMap;

struct MarketDataCache {
    prices: RefCell<HashMap<String, f64>>,
    access_count: RefCell<u32>,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            prices: RefCell::new(HashMap::new()),
            access_count: RefCell::new(0),
        }
    }

    // &self — можем вызывать через неизменяемую ссылку
    fn update_price(&self, symbol: &str, price: f64) {
        self.prices.borrow_mut().insert(symbol.to_string(), price);
    }

    fn get_price(&self, symbol: &str) -> Option<f64> {
        *self.access_count.borrow_mut() += 1;
        self.prices.borrow().get(symbol).copied()
    }

    fn get_access_count(&self) -> u32 {
        *self.access_count.borrow()
    }

    fn print_all_prices(&self) {
        println!("\n=== Кеш рыночных данных ===");
        println!("Обращений к кешу: {}", self.access_count.borrow());
        for (symbol, price) in self.prices.borrow().iter() {
            println!("  {}: ${:.2}", symbol, price);
        }
    }
}

fn main() {
    let cache = MarketDataCache::new();

    // Обновляем цены
    cache.update_price("BTC/USDT", 42000.0);
    cache.update_price("ETH/USDT", 2500.0);
    cache.update_price("SOL/USDT", 95.0);

    // Получаем цены (счётчик увеличивается)
    if let Some(btc) = cache.get_price("BTC/USDT") {
        println!("BTC: ${:.2}", btc);
    }

    if let Some(eth) = cache.get_price("ETH/USDT") {
        println!("ETH: ${:.2}", eth);
    }

    // Обновляем цену BTC
    cache.update_price("BTC/USDT", 42500.0);

    if let Some(btc) = cache.get_price("BTC/USDT") {
        println!("BTC (обновлено): ${:.2}", btc);
    }

    cache.print_all_prices();
}
```

## Практический пример: журнал сделок

```rust
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    size: f64,
    pnl: Option<f64>,
}

struct TradeJournal {
    trades: RefCell<Vec<Trade>>,
    next_id: RefCell<u64>,
}

impl TradeJournal {
    fn new() -> Self {
        TradeJournal {
            trades: RefCell::new(Vec::new()),
            next_id: RefCell::new(1),
        }
    }

    fn record_trade(&self, symbol: &str, side: &str, price: f64, size: f64) -> u64 {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;

        let trade = Trade {
            id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            size,
            pnl: None,
        };

        self.trades.borrow_mut().push(trade);

        println!("Записана сделка #{}: {} {} @ {:.2}", id, side, symbol, price);
        id
    }

    fn set_pnl(&self, trade_id: u64, pnl: f64) {
        let mut trades = self.trades.borrow_mut();
        if let Some(trade) = trades.iter_mut().find(|t| t.id == trade_id) {
            trade.pnl = Some(pnl);
            println!("Установлен PnL для сделки #{}: {:+.2}", trade_id, pnl);
        }
    }

    fn total_pnl(&self) -> f64 {
        self.trades
            .borrow()
            .iter()
            .filter_map(|t| t.pnl)
            .sum()
    }

    fn print_summary(&self) {
        let trades = self.trades.borrow();

        println!("\n=== Журнал сделок ===");
        println!("Всего сделок: {}", trades.len());

        for trade in trades.iter() {
            let pnl_str = match trade.pnl {
                Some(pnl) => format!("{:+.2}", pnl),
                None => "N/A".to_string(),
            };
            println!(
                "  #{}: {} {} x {} @ {:.2} | PnL: {}",
                trade.id, trade.side, trade.symbol, trade.size, trade.price, pnl_str
            );
        }

        println!("Общий PnL: {:+.2}", self.total_pnl());
    }
}

fn main() {
    let journal = TradeJournal::new();

    // Записываем сделки
    let trade1 = journal.record_trade("BTC/USDT", "BUY", 42000.0, 0.5);
    let trade2 = journal.record_trade("ETH/USDT", "BUY", 2500.0, 2.0);
    let trade3 = journal.record_trade("BTC/USDT", "SELL", 42500.0, 0.5);

    // Устанавливаем PnL
    journal.set_pnl(trade1, 250.0);
    journal.set_pnl(trade2, -100.0);
    journal.set_pnl(trade3, 0.0);  // Это была закрывающая сделка

    journal.print_summary();
}
```

## Cell vs RefCell

| Характеристика | Cell<T> | RefCell<T> |
|---------------|---------|------------|
| Копирование | Требует `Copy` | Любой тип |
| Доступ к данным | Копирует значение | Возвращает ссылку |
| Проверка | Нет проверок | Проверка в runtime |
| Паника | Не паникует | Может паниковать |
| Использование | Простые типы (i32, f64) | Сложные типы (Vec, String) |

```rust
use std::cell::{Cell, RefCell};

fn main() {
    // Cell — для Copy-типов
    let price = Cell::new(42000.0);
    price.set(42100.0);
    println!("Цена: {}", price.get());

    // RefCell — для не-Copy типов
    let prices = RefCell::new(vec![42000.0, 42100.0]);
    prices.borrow_mut().push(42200.0);
    println!("Цены: {:?}", prices.borrow());
}
```

## Когда использовать RefCell

**Используй RefCell когда:**
- Нужна внутренняя изменяемость (interior mutability)
- Правила заимствования не могут быть проверены во время компиляции
- Данные изменяются через неизменяемые ссылки
- Нужно изменять поля структуры в методах с `&self`

**Не используй RefCell когда:**
- Достаточно обычного `&mut`
- Работаешь в многопоточном контексте (используй `Mutex`)
- Простые Copy-типы (используй `Cell`)

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `RefCell<T>` | Контейнер с проверкой заимствования в runtime |
| `borrow()` | Неизменяемое заимствование → `Ref<T>` |
| `borrow_mut()` | Изменяемое заимствование → `RefMut<T>` |
| `try_borrow()` | Безопасная попытка заимствования |
| `Rc<RefCell<T>>` | Общие изменяемые данные |

## Домашнее задание

1. Создай структуру `Portfolio` с `RefCell<HashMap<String, Position>>` для хранения позиций. Реализуй методы `add_position`, `update_position` и `get_total_value` через `&self`.

2. Реализуй систему подписки на обновления цен: несколько `TradingStrategy` используют один `Rc<RefCell<MarketData>>`. При обновлении данных все стратегии должны иметь доступ к актуальным ценам.

3. Создай `TransactionLog` с `RefCell<Vec<Transaction>>`. Реализуй методы для добавления транзакций, расчёта баланса и отката последней транзакции.

4. Напиши торгового бота с `RefCell`-полями для баланса, открытых ордеров и истории. Бот должен уметь открывать/закрывать позиции, обновлять стоп-лоссы и тейк-профиты через методы с `&self`.

## Навигация

[← Предыдущий день](../056-cell-interior-mutability/ru.md) | [Следующий день →](../058-mutex-thread-safe-interior-mutability/ru.md)
