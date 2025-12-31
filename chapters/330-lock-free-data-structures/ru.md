# День 330: Lock-free структуры данных

## Аналогия из трейдинга

Представь себе биржевой стакан (order book) в высокочастотной торговой системе. Тысячи ордеров приходят каждую секунду, и каждый из них должен быть обработан максимально быстро.

В традиционном подходе с блокировками (мьютексами) всё выглядит так:
- Трейдер А хочет добавить ордер — берёт блокировку стакана
- Трейдер Б хочет отменить ордер — ждёт, пока А освободит блокировку
- Трейдер В хочет прочитать лучшую цену — тоже ждёт в очереди

Это как если бы на бирже была только одна касса, и все выстраивались в очередь!

**Lock-free структуры данных** — это как современный электронный стакан:
- Множество трейдеров могут одновременно смотреть текущие цены
- Новые ордера добавляются атомарно, без остановки всей системы
- Отмены обрабатываются параллельно с новыми ордерами
- Никто не ждёт — каждый продвигается вперёд

В lock-free подходе потоки не блокируют друг друга. Даже если один поток "завис", остальные продолжают работать. Это критически важно для торговых систем, где каждая микросекунда задержки стоит денег.

## Основные концепции

### Атомарные операции

Атомарные операции — это неделимые операции, которые выполняются полностью или не выполняются вовсе:

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Атомарный счётчик сделок
struct TradeCounter {
    count: AtomicU64,
    total_volume: AtomicU64,
}

impl TradeCounter {
    fn new() -> Self {
        TradeCounter {
            count: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
        }
    }

    /// Добавить сделку (потокобезопасно, без блокировок)
    fn add_trade(&self, volume: u64) {
        // fetch_add — атомарная операция "прочитать и увеличить"
        self.count.fetch_add(1, Ordering::SeqCst);
        self.total_volume.fetch_add(volume, Ordering::SeqCst);
    }

    fn get_stats(&self) -> (u64, u64) {
        (
            self.count.load(Ordering::SeqCst),
            self.total_volume.load(Ordering::SeqCst),
        )
    }
}

fn main() {
    let counter = Arc::new(TradeCounter::new());
    let mut handles = vec![];

    // 4 потока симулируют торговлю
    for trader_id in 0..4 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let volume = (trader_id * 100 + i) as u64;
                counter.add_trade(volume);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (count, volume) = counter.get_stats();
    println!("Всего сделок: {}", count);
    println!("Общий объём: {}", volume);
}
```

### Compare-and-Swap (CAS)

CAS — это фундаментальная операция для lock-free алгоритмов:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free обновление лучшей цены
struct BestPrice {
    // Храним цену как целое число (в копейках/сатоши)
    price: AtomicU64,
}

impl BestPrice {
    fn new(initial: u64) -> Self {
        BestPrice {
            price: AtomicU64::new(initial),
        }
    }

    /// Обновить цену, если новая лучше (меньше для ask, больше для bid)
    /// Возвращает true, если обновление успешно
    fn update_if_better_ask(&self, new_price: u64) -> bool {
        loop {
            let current = self.price.load(Ordering::SeqCst);

            // Новая цена не лучше текущей
            if new_price >= current {
                return false;
            }

            // Пробуем атомарно заменить
            // compare_exchange вернёт Ok, если current не изменился
            match self.price.compare_exchange(
                current,
                new_price,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(_) => {
                    // Кто-то другой изменил цену, пробуем снова
                    continue;
                }
            }
        }
    }

    fn get(&self) -> u64 {
        self.price.load(Ordering::SeqCst)
    }
}

fn main() {
    let best_ask = Arc::new(BestPrice::new(u64::MAX));
    let mut handles = vec![];

    // Множество потоков пытаются обновить лучшую цену
    for i in 0..10 {
        let best_ask = Arc::clone(&best_ask);
        let handle = thread::spawn(move || {
            // Симулируем разные цены от разных маркет-мейкеров
            let prices = [42500, 42490, 42510, 42480, 42495];
            for &price in &prices {
                let adjusted_price = price + (i * 5); // Разные источники
                if best_ask.update_if_better_ask(adjusted_price) {
                    println!("Поток {} обновил цену на {}", i, adjusted_price);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nЛучший ask: {}", best_ask.get());
}
```

## Lock-free очередь ордеров

Реализуем простую lock-free очередь для торговых ордеров:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

struct Node {
    order: Option<Order>,
    next: AtomicPtr<Node>,
}

impl Node {
    fn new(order: Option<Order>) -> *mut Node {
        Box::into_raw(Box::new(Node {
            order,
            next: AtomicPtr::new(ptr::null_mut()),
        }))
    }
}

/// Lock-free очередь (упрощённая реализация Michael-Scott queue)
pub struct LockFreeOrderQueue {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
    len: AtomicUsize,
}

impl LockFreeOrderQueue {
    pub fn new() -> Self {
        // Создаём пустой узел-заглушку
        let dummy = Node::new(None);
        LockFreeOrderQueue {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
            len: AtomicUsize::new(0),
        }
    }

    /// Добавить ордер в очередь (lock-free)
    pub fn enqueue(&self, order: Order) {
        let new_node = Node::new(Some(order));

        loop {
            let tail = self.tail.load(Ordering::SeqCst);
            let tail_ref = unsafe { &*tail };
            let next = tail_ref.next.load(Ordering::SeqCst);

            if next.is_null() {
                // Пробуем добавить новый узел
                if tail_ref
                    .next
                    .compare_exchange(
                        ptr::null_mut(),
                        new_node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    // Обновляем tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        new_node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    self.len.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            } else {
                // Помогаем обновить tail
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Извлечь ордер из очереди (lock-free)
    pub fn dequeue(&self) -> Option<Order> {
        loop {
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            let head_ref = unsafe { &*head };
            let next = head_ref.next.load(Ordering::SeqCst);

            if head == tail {
                if next.is_null() {
                    return None; // Очередь пуста
                }
                // Tail отстал, помогаем обновить
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            } else {
                if next.is_null() {
                    continue;
                }
                let next_ref = unsafe { &*next };
                let order = next_ref.order.clone();

                if self
                    .head
                    .compare_exchange(head, next, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    self.len.fetch_sub(1, Ordering::SeqCst);
                    // Освобождаем старый head
                    unsafe {
                        drop(Box::from_raw(head));
                    }
                    return order;
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Для упрощённого примера добавляем Clone для Order
impl Clone for Order {
    fn clone(&self) -> Self {
        Order {
            id: self.id,
            symbol: self.symbol.clone(),
            price: self.price,
            quantity: self.quantity,
        }
    }
}

fn main() {
    let queue = Arc::new(LockFreeOrderQueue::new());
    let mut handles = vec![];

    // Продюсеры — добавляют ордера
    for producer_id in 0..3 {
        let queue = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let order = Order {
                    id: producer_id * 1000 + i,
                    symbol: "BTCUSDT".to_string(),
                    price: 42500.0 + i as f64,
                    quantity: 0.1,
                };
                queue.enqueue(order);
            }
            println!("Продюсер {} добавил 100 ордеров", producer_id);
        });
        handles.push(handle);
    }

    // Консьюмеры — обрабатывают ордера
    for consumer_id in 0..2 {
        let queue = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            let mut processed = 0;
            loop {
                match queue.dequeue() {
                    Some(_order) => {
                        processed += 1;
                    }
                    None => {
                        if processed > 0 {
                            break;
                        }
                        thread::yield_now();
                    }
                }
            }
            println!("Консьюмер {} обработал {} ордеров", consumer_id, processed);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Осталось в очереди: {}", queue.len());
}
```

## Lock-free стек цен

Стек полезен для хранения истории цен или откатов:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug)]
struct PricePoint {
    timestamp: u64,
    price: f64,
}

struct StackNode {
    data: PricePoint,
    next: *mut StackNode,
}

/// Lock-free стек (Treiber stack)
pub struct LockFreePriceStack {
    head: AtomicPtr<StackNode>,
    len: AtomicUsize,
}

impl LockFreePriceStack {
    pub fn new() -> Self {
        LockFreePriceStack {
            head: AtomicPtr::new(ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    /// Добавить цену в стек (lock-free)
    pub fn push(&self, price_point: PricePoint) {
        let new_node = Box::into_raw(Box::new(StackNode {
            data: price_point,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::SeqCst);
            unsafe {
                (*new_node).next = head;
            }

            if self
                .head
                .compare_exchange(head, new_node, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.len.fetch_add(1, Ordering::SeqCst);
                return;
            }
        }
    }

    /// Извлечь последнюю цену из стека (lock-free)
    pub fn pop(&self) -> Option<PricePoint> {
        loop {
            let head = self.head.load(Ordering::SeqCst);
            if head.is_null() {
                return None;
            }

            let next = unsafe { (*head).next };

            if self
                .head
                .compare_exchange(head, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.len.fetch_sub(1, Ordering::SeqCst);
                let data = unsafe {
                    let node = Box::from_raw(head);
                    node.data.clone()
                };
                return Some(data);
            }
        }
    }

    /// Посмотреть последнюю цену без извлечения
    pub fn peek(&self) -> Option<PricePoint> {
        let head = self.head.load(Ordering::SeqCst);
        if head.is_null() {
            None
        } else {
            unsafe { Some((*head).data.clone()) }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }
}

fn main() {
    let stack = Arc::new(LockFreePriceStack::new());
    let mut handles = vec![];

    // Несколько потоков записывают цены
    for thread_id in 0..4 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            for i in 0..50 {
                let price_point = PricePoint {
                    timestamp: thread_id * 1000 + i,
                    price: 42000.0 + (thread_id * 100 + i) as f64,
                };
                stack.push(price_point);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Записей в стеке: {}", stack.len());

    // Читаем последние 10 цен
    println!("\nПоследние цены:");
    for _ in 0..10 {
        if let Some(point) = stack.pop() {
            println!("  ts={}: ${:.2}", point.timestamp, point.price);
        }
    }
}
```

## Атомарные флаги для состояния рынка

```rust
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum MarketState {
    PreOpen = 0,
    Open = 1,
    Halt = 2,
    Closed = 3,
}

impl From<u8> for MarketState {
    fn from(v: u8) -> Self {
        match v {
            0 => MarketState::PreOpen,
            1 => MarketState::Open,
            2 => MarketState::Halt,
            3 => MarketState::Closed,
            _ => MarketState::Closed,
        }
    }
}

/// Атомарное состояние рынка
struct AtomicMarketState {
    state: AtomicU8,
    trading_enabled: AtomicBool,
}

impl AtomicMarketState {
    fn new() -> Self {
        AtomicMarketState {
            state: AtomicU8::new(MarketState::PreOpen as u8),
            trading_enabled: AtomicBool::new(false),
        }
    }

    fn get_state(&self) -> MarketState {
        MarketState::from(self.state.load(Ordering::SeqCst))
    }

    fn set_state(&self, new_state: MarketState) {
        self.state.store(new_state as u8, Ordering::SeqCst);

        // Автоматически управляем флагом торговли
        let trading = matches!(new_state, MarketState::Open);
        self.trading_enabled.store(trading, Ordering::SeqCst);
    }

    fn is_trading_enabled(&self) -> bool {
        self.trading_enabled.load(Ordering::SeqCst)
    }

    /// Попытка перевести рынок в режим halt (атомарно)
    fn try_halt(&self) -> bool {
        let current = MarketState::Open as u8;
        let new = MarketState::Halt as u8;

        match self.state.compare_exchange(
            current,
            new,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                self.trading_enabled.store(false, Ordering::SeqCst);
                true
            }
            Err(_) => false,
        }
    }
}

fn main() {
    let market = Arc::new(AtomicMarketState::new());
    let mut handles = vec![];

    // Контроллер рынка
    {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            println!("[Контроллер] Рынок в состоянии PreOpen");
            thread::sleep(Duration::from_millis(100));

            market.set_state(MarketState::Open);
            println!("[Контроллер] Рынок открыт!");
            thread::sleep(Duration::from_millis(300));

            market.set_state(MarketState::Closed);
            println!("[Контроллер] Рынок закрыт");
        });
        handles.push(handle);
    }

    // Торговые боты проверяют состояние
    for bot_id in 0..3 {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            let mut trades = 0;
            for _ in 0..50 {
                if market.is_trading_enabled() {
                    trades += 1;
                }
                thread::sleep(Duration::from_millis(10));
            }
            println!("[Бот {}] Совершено сделок: {}", bot_id, trades);
        });
        handles.push(handle);
    }

    // Монитор рисков может остановить торги
    {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            if market.try_halt() {
                println!("[Риск-монитор] Торги приостановлены!");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nФинальное состояние: {:?}", market.get_state());
}
```

## Lock-free счётчик позиций

```rust
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free трекер позиции
struct PositionTracker {
    // Позиция в базовых единицах (может быть отрицательной для шортов)
    position: AtomicI64,
    // Общий объём сделок (всегда положительный)
    total_volume: AtomicU64,
    // Количество сделок
    trade_count: AtomicU64,
}

impl PositionTracker {
    fn new() -> Self {
        PositionTracker {
            position: AtomicI64::new(0),
            total_volume: AtomicU64::new(0),
            trade_count: AtomicU64::new(0),
        }
    }

    /// Покупка (увеличение позиции)
    fn buy(&self, quantity: i64) {
        self.position.fetch_add(quantity, Ordering::SeqCst);
        self.total_volume.fetch_add(quantity.unsigned_abs(), Ordering::SeqCst);
        self.trade_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Продажа (уменьшение позиции)
    fn sell(&self, quantity: i64) {
        self.position.fetch_sub(quantity, Ordering::SeqCst);
        self.total_volume.fetch_add(quantity.unsigned_abs(), Ordering::SeqCst);
        self.trade_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Получить текущую позицию
    fn get_position(&self) -> i64 {
        self.position.load(Ordering::SeqCst)
    }

    /// Получить статистику
    fn get_stats(&self) -> (i64, u64, u64) {
        (
            self.position.load(Ordering::SeqCst),
            self.total_volume.load(Ordering::SeqCst),
            self.trade_count.load(Ordering::SeqCst),
        )
    }

    /// Атомарно проверить и обновить позицию
    /// Возвращает true, если обновление прошло успешно
    fn try_update_position(&self, expected: i64, new: i64) -> bool {
        self.position
            .compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

fn main() {
    let tracker = Arc::new(PositionTracker::new());
    let mut handles = vec![];

    // Покупатели
    for _ in 0..3 {
        let tracker = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                tracker.buy(10);
            }
        });
        handles.push(handle);
    }

    // Продавцы
    for _ in 0..2 {
        let tracker = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                tracker.sell(10);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (position, volume, trades) = tracker.get_stats();
    println!("=== Результаты ===");
    println!("Финальная позиция: {} единиц", position);
    println!("Общий объём: {} единиц", volume);
    println!("Количество сделок: {}", trades);

    // Проверка: 3 покупателя * 100 * 10 - 2 продавца * 100 * 10 = 1000
    println!("\nОжидаемая позиция: {}", (3 - 2) * 100 * 10);
}
```

## SeqLock для данных тикера

SeqLock позволяет читателям работать без блокировок:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
struct TickerData {
    bid: f64,
    ask: f64,
    last: f64,
    volume: f64,
}

/// SeqLock для данных тикера
/// Один писатель, много читателей, без блокировок для читателей
struct SeqLockTicker {
    sequence: AtomicU64,
    data: std::cell::UnsafeCell<TickerData>,
}

unsafe impl Sync for SeqLockTicker {}

impl SeqLockTicker {
    fn new(data: TickerData) -> Self {
        SeqLockTicker {
            sequence: AtomicU64::new(0),
            data: std::cell::UnsafeCell::new(data),
        }
    }

    /// Записать новые данные (только один писатель!)
    fn write(&self, new_data: TickerData) {
        // Увеличиваем sequence на 1 (нечётное = запись)
        self.sequence.fetch_add(1, Ordering::SeqCst);

        // Записываем данные
        unsafe {
            *self.data.get() = new_data;
        }

        // Увеличиваем sequence на 1 (чётное = данные стабильны)
        self.sequence.fetch_add(1, Ordering::SeqCst);
    }

    /// Прочитать данные (lock-free для читателей)
    fn read(&self) -> TickerData {
        loop {
            let seq1 = self.sequence.load(Ordering::SeqCst);

            // Если sequence нечётный, писатель активен — ждём
            if seq1 % 2 != 0 {
                std::hint::spin_loop();
                continue;
            }

            // Читаем данные
            let data = unsafe { (*self.data.get()).clone() };

            // Проверяем, что sequence не изменился
            let seq2 = self.sequence.load(Ordering::SeqCst);

            if seq1 == seq2 {
                return data;
            }
            // Если изменился — повторяем чтение
        }
    }
}

fn main() {
    let ticker = Arc::new(SeqLockTicker::new(TickerData {
        bid: 42500.0,
        ask: 42501.0,
        last: 42500.5,
        volume: 0.0,
    }));

    let mut handles = vec![];

    // Писатель — обновляет данные
    {
        let ticker = Arc::clone(&ticker);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let data = TickerData {
                    bid: 42500.0 + i as f64,
                    ask: 42501.0 + i as f64,
                    last: 42500.5 + i as f64,
                    volume: i as f64 * 10.0,
                };
                ticker.write(data);
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Читатели — читают данные без блокировок
    for reader_id in 0..4 {
        let ticker = Arc::clone(&ticker);
        let handle = thread::spawn(move || {
            let mut reads = 0;
            for _ in 0..500 {
                let data = ticker.read();
                reads += 1;
                if reads % 100 == 0 {
                    println!(
                        "[Читатель {}] bid={:.2} ask={:.2} volume={:.1}",
                        reader_id, data.bid, data.ask, data.volume
                    );
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_data = ticker.read();
    println!("\nФинальные данные: {:?}", final_data);
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|----------------------|
| **Атомарные операции** | Неделимые операции с гарантией | Счётчики сделок, объёмов |
| **Compare-and-Swap** | Условное атомарное обновление | Обновление лучшей цены |
| **Lock-free очередь** | Очередь без блокировок | Очередь ордеров |
| **Lock-free стек** | Стек без блокировок | История цен, откаты |
| **Атомарные флаги** | Булевы флаги для состояний | Состояние рынка |
| **SeqLock** | Оптимизация для частых чтений | Данные тикера |

## Практические задания

1. **Lock-free книга ордеров**: Реализуйте упрощённую книгу ордеров с lock-free добавлением и отменой ордеров.

2. **Атомарный агрегатор цен**: Создайте структуру, которая атомарно отслеживает min, max, avg цену за период.

3. **Lock-free кольцевой буфер**: Реализуйте кольцевой буфер для хранения последних N тиков без блокировок.

4. **Атомарный rate limiter**: Создайте ограничитель скорости запросов к API биржи с использованием атомарных операций.

## Домашнее задание

1. **Lock-free агрегатор книги ордеров**: Реализуйте систему, которая объединяет книги ордеров с нескольких бирж в единую view без использования мьютексов.

2. **Атомарный трекер PnL**: Создайте трекер прибыли/убытка, который обновляется из множества потоков с использованием только атомарных операций.

3. **Сравнение производительности**: Напишите бенчмарк, сравнивающий производительность lock-free очереди с `Mutex<VecDeque>` при разном количестве продюсеров и консьюмеров.

4. **Lock-free кеш с TTL**: Реализуйте кеш цен с автоматической инвалидацией, используя только атомарные операции.

## Навигация

[← Предыдущий день](../321-result-caching/ru.md) | [Следующий день →](../331-*/ru.md)
