# День 87: VecDeque — очередь ордеров

## Аналогия из трейдинга

На бирже ордера обрабатываются в определённом порядке:
- **Очередь ордеров**: первым пришёл — первым исполнен (FIFO)
- **Стакан заявок**: нужен быстрый доступ к лучшим ценам с обеих сторон
- **История сделок**: новые сделки добавляются в конец, старые удаляются с начала

`VecDeque` (двусторонняя очередь) — идеальная структура для таких сценариев:
- O(1) добавление/удаление с обоих концов
- Эффективнее `Vec` когда нужны операции с началом коллекции

## Создание VecDeque

```rust
use std::collections::VecDeque;

fn main() {
    // Пустая очередь ордеров
    let mut order_queue: VecDeque<String> = VecDeque::new();

    // С начальной ёмкостью (для высокочастотной торговли)
    let mut hft_queue: VecDeque<f64> = VecDeque::with_capacity(10000);

    // Из вектора
    let prices = vec![42000.0, 42100.0, 42050.0];
    let price_buffer: VecDeque<f64> = VecDeque::from(prices);

    // Макрос (nightly) или через итератор
    let quick_queue: VecDeque<i32> = [1, 2, 3, 4, 5].into_iter().collect();

    println!("Order queue len: {}", order_queue.len());
    println!("HFT queue capacity: {}", hft_queue.capacity());
    println!("Price buffer: {:?}", price_buffer);
}
```

## Добавление элементов

```rust
use std::collections::VecDeque;

fn main() {
    let mut orders: VecDeque<&str> = VecDeque::new();

    // push_back — добавить в конец (новый ордер в очередь)
    orders.push_back("BUY BTC 0.1");
    orders.push_back("SELL ETH 2.0");
    orders.push_back("BUY BTC 0.5");

    println!("Order queue: {:?}", orders);
    // ["BUY BTC 0.1", "SELL ETH 2.0", "BUY BTC 0.5"]

    // push_front — добавить в начало (приоритетный ордер)
    orders.push_front("URGENT: SELL BTC 1.0");

    println!("With priority order: {:?}", orders);
    // ["URGENT: SELL BTC 1.0", "BUY BTC 0.1", "SELL ETH 2.0", "BUY BTC 0.5"]
}
```

## Удаление элементов

```rust
use std::collections::VecDeque;

fn main() {
    let mut orders: VecDeque<&str> = VecDeque::new();
    orders.push_back("Order 1");
    orders.push_back("Order 2");
    orders.push_back("Order 3");
    orders.push_back("Order 4");

    // pop_front — извлечь первый (FIFO обработка)
    if let Some(order) = orders.pop_front() {
        println!("Processing: {}", order);  // Order 1
    }

    // pop_back — извлечь последний (отмена последнего ордера)
    if let Some(order) = orders.pop_back() {
        println!("Cancelled: {}", order);  // Order 4
    }

    println!("Remaining: {:?}", orders);  // ["Order 2", "Order 3"]
}
```

## Скользящее окно цен

Один из главных паттернов использования VecDeque — скользящее окно:

```rust
use std::collections::VecDeque;

fn main() {
    // Скользящее окно для расчёта SMA-5
    let mut price_window: VecDeque<f64> = VecDeque::with_capacity(5);
    let window_size = 5;

    // Поток новых цен
    let incoming_prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
                           42300.0, 42250.0, 42400.0];

    for price in incoming_prices {
        // Добавляем новую цену в конец
        price_window.push_back(price);

        // Если окно переполнено — удаляем старую цену
        if price_window.len() > window_size {
            price_window.pop_front();
        }

        // Рассчитываем SMA когда окно заполнено
        if price_window.len() == window_size {
            let sma: f64 = price_window.iter().sum::<f64>() / window_size as f64;
            println!("Price: {:.0} | SMA-5: {:.2}", price, sma);
        } else {
            println!("Price: {:.0} | Collecting data ({}/{})",
                     price, price_window.len(), window_size);
        }
    }
}
```

## Доступ к элементам

```rust
use std::collections::VecDeque;

fn main() {
    let mut prices: VecDeque<f64> = VecDeque::new();
    prices.push_back(42000.0);
    prices.push_back(42100.0);
    prices.push_back(42200.0);
    prices.push_back(42300.0);

    // Доступ по индексу
    println!("First price: {}", prices[0]);
    println!("Last price: {}", prices[prices.len() - 1]);

    // Безопасный доступ
    if let Some(price) = prices.get(2) {
        println!("Price at index 2: {}", price);
    }

    // front() и back() — доступ к концам
    println!("Front (oldest): {:?}", prices.front());
    println!("Back (newest): {:?}", prices.back());

    // Изменяемый доступ
    if let Some(price) = prices.front_mut() {
        *price = 41900.0;  // Корректируем первую цену
    }

    println!("Updated prices: {:?}", prices);
}
```

## Книга ордеров (Order Book)

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

struct OrderBook {
    buy_orders: VecDeque<Order>,   // Заявки на покупку
    sell_orders: VecDeque<Order>,  // Заявки на продажу
    next_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            buy_orders: VecDeque::new(),
            sell_orders: VecDeque::new(),
            next_id: 1,
        }
    }

    fn add_order(&mut self, side: &str, price: f64, quantity: f64, timestamp: u64) {
        let order = Order {
            id: self.next_id,
            side: side.to_string(),
            price,
            quantity,
            timestamp,
        };
        self.next_id += 1;

        match side {
            "BUY" => self.buy_orders.push_back(order),
            "SELL" => self.sell_orders.push_back(order),
            _ => println!("Unknown order side"),
        }
    }

    fn process_next_buy(&mut self) -> Option<Order> {
        self.buy_orders.pop_front()
    }

    fn process_next_sell(&mut self) -> Option<Order> {
        self.sell_orders.pop_front()
    }

    fn cancel_last_order(&mut self, side: &str) -> Option<Order> {
        match side {
            "BUY" => self.buy_orders.pop_back(),
            "SELL" => self.sell_orders.pop_back(),
            _ => None,
        }
    }

    fn display(&self) {
        println!("=== Order Book ===");
        println!("Buy orders ({}):", self.buy_orders.len());
        for order in &self.buy_orders {
            println!("  #{}: {} {} @ {:.2}",
                     order.id, order.quantity, order.side, order.price);
        }
        println!("Sell orders ({}):", self.sell_orders.len());
        for order in &self.sell_orders {
            println!("  #{}: {} {} @ {:.2}",
                     order.id, order.quantity, order.side, order.price);
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Добавляем ордера
    book.add_order("BUY", 41950.0, 0.5, 1000);
    book.add_order("BUY", 41900.0, 1.0, 1001);
    book.add_order("SELL", 42000.0, 0.3, 1002);
    book.add_order("SELL", 42050.0, 0.7, 1003);

    book.display();

    // Исполняем первый ордер на покупку
    if let Some(order) = book.process_next_buy() {
        println!("\nExecuted: {:?}", order);
    }

    // Отменяем последний ордер на продажу
    if let Some(order) = book.cancel_last_order("SELL") {
        println!("Cancelled: {:?}", order);
    }

    println!();
    book.display();
}
```

## История сделок с лимитом

```rust
use std::collections::VecDeque;

#[derive(Debug)]
struct Trade {
    price: f64,
    quantity: f64,
    side: String,
    timestamp: u64,
}

struct TradeHistory {
    trades: VecDeque<Trade>,
    max_size: usize,
}

impl TradeHistory {
    fn new(max_size: usize) -> Self {
        TradeHistory {
            trades: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add_trade(&mut self, price: f64, quantity: f64, side: &str, timestamp: u64) {
        let trade = Trade {
            price,
            quantity,
            side: side.to_string(),
            timestamp,
        };

        self.trades.push_back(trade);

        // Удаляем старые сделки если превышен лимит
        while self.trades.len() > self.max_size {
            self.trades.pop_front();
        }
    }

    fn get_recent(&self, count: usize) -> Vec<&Trade> {
        self.trades.iter().rev().take(count).collect()
    }

    fn average_price(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.trades.iter().map(|t| t.price).sum();
        sum / self.trades.len() as f64
    }

    fn total_volume(&self) -> f64 {
        self.trades.iter().map(|t| t.quantity).sum()
    }
}

fn main() {
    let mut history = TradeHistory::new(5);  // Хранить только 5 последних сделок

    // Добавляем сделки
    history.add_trade(42000.0, 0.5, "BUY", 1000);
    history.add_trade(42010.0, 0.3, "BUY", 1001);
    history.add_trade(42005.0, 0.8, "SELL", 1002);
    history.add_trade(42020.0, 0.2, "BUY", 1003);
    history.add_trade(42015.0, 0.6, "SELL", 1004);
    history.add_trade(42025.0, 0.4, "BUY", 1005);  // Первая сделка будет удалена
    history.add_trade(42030.0, 0.1, "BUY", 1006);  // Вторая сделка будет удалена

    println!("Trade history (max 5):");
    for trade in &history.trades {
        println!("  {:?}", trade);
    }

    println!("\nRecent 3 trades:");
    for trade in history.get_recent(3) {
        println!("  {} {:.1} @ {:.2}", trade.side, trade.quantity, trade.price);
    }

    println!("\nAverage price: {:.2}", history.average_price());
    println!("Total volume: {:.2}", history.total_volume());
}
```

## Итерация и преобразование

```rust
use std::collections::VecDeque;

fn main() {
    let mut prices: VecDeque<f64> = VecDeque::new();
    prices.push_back(42000.0);
    prices.push_back(42100.0);
    prices.push_back(42050.0);
    prices.push_back(42200.0);

    // Итерация
    println!("All prices:");
    for price in &prices {
        println!("  {}", price);
    }

    // С индексом
    println!("\nWith index:");
    for (i, price) in prices.iter().enumerate() {
        println!("  [{}] {}", i, price);
    }

    // Изменяемая итерация — применить комиссию
    for price in prices.iter_mut() {
        *price *= 0.999;  // -0.1% комиссия
    }
    println!("\nAfter fees: {:?}", prices);

    // Преобразование в Vec
    let price_vec: Vec<f64> = prices.clone().into_iter().collect();
    println!("\nAs Vec: {:?}", price_vec);

    // make_contiguous — гарантирует непрерывность в памяти
    let slice = prices.make_contiguous();
    println!("As slice: {:?}", slice);
}
```

## Rotate и другие операции

```rust
use std::collections::VecDeque;

fn main() {
    let mut queue: VecDeque<i32> = (1..=5).collect();
    println!("Original: {:?}", queue);

    // rotate_left — сдвинуть элементы влево
    queue.rotate_left(2);
    println!("Rotate left 2: {:?}", queue);  // [3, 4, 5, 1, 2]

    // rotate_right — сдвинуть элементы вправо
    queue.rotate_right(2);
    println!("Rotate right 2: {:?}", queue);  // [1, 2, 3, 4, 5]

    // swap — поменять элементы местами
    queue.swap(0, 4);
    println!("After swap(0,4): {:?}", queue);  // [5, 2, 3, 4, 1]

    // retain — оставить только элементы, удовлетворяющие условию
    queue.retain(|&x| x > 2);
    println!("After retain(>2): {:?}", queue);  // [5, 3, 4]

    // clear — очистить очередь
    queue.clear();
    println!("After clear: {:?}", queue);  // []
}
```

## VecDeque vs Vec: когда что использовать

```rust
use std::collections::VecDeque;

fn main() {
    // VecDeque лучше для:
    // 1. Очереди (FIFO) — добавление в конец, удаление с начала
    let mut fifo: VecDeque<i32> = VecDeque::new();
    fifo.push_back(1);   // O(1)
    fifo.pop_front();    // O(1) — у Vec это было бы O(n)!

    // 2. Скользящее окно
    // 3. Двусторонние операции (deque)

    // Vec лучше для:
    // 1. Случайный доступ (оба O(1), но Vec быстрее из-за кэша)
    // 2. Только добавление/удаление с конца
    // 3. Работа со срезами (slices)

    println!("VecDeque: оптимален для очередей и скользящих окон");
    println!("Vec: оптимален для стеков и случайного доступа");
}
```

## Практический пример: Rate Limiter

```rust
use std::collections::VecDeque;

struct RateLimiter {
    requests: VecDeque<u64>,  // timestamps запросов
    max_requests: usize,
    window_ms: u64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_ms: u64) -> Self {
        RateLimiter {
            requests: VecDeque::with_capacity(max_requests),
            max_requests,
            window_ms,
        }
    }

    fn allow_request(&mut self, current_time: u64) -> bool {
        // Удаляем устаревшие запросы
        while let Some(&oldest) = self.requests.front() {
            if current_time - oldest > self.window_ms {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Проверяем лимит
        if self.requests.len() < self.max_requests {
            self.requests.push_back(current_time);
            true
        } else {
            false
        }
    }
}

fn main() {
    // Максимум 5 запросов в 1000мс
    let mut limiter = RateLimiter::new(5, 1000);

    let timestamps = [100, 200, 300, 400, 500, 600, 700, 1200, 1300];

    for ts in timestamps {
        let allowed = limiter.allow_request(ts);
        println!("Time {}: {}", ts, if allowed { "ALLOWED" } else { "BLOCKED" });
    }
}
```

## Что мы узнали

| Метод | Описание | Сложность |
|-------|----------|-----------|
| `push_back(x)` | Добавить в конец | O(1) |
| `push_front(x)` | Добавить в начало | O(1) |
| `pop_back()` | Удалить с конца | O(1) |
| `pop_front()` | Удалить с начала | O(1) |
| `front()` / `back()` | Доступ к концам | O(1) |
| `get(i)` | Доступ по индексу | O(1) |
| `len()` | Размер | O(1) |
| `rotate_left(n)` | Сдвиг влево | O(n) |

## Домашнее задание

1. **Очередь ордеров с приоритетом**: создай систему где VIP-клиенты могут добавлять ордера в начало очереди через `push_front`, а обычные — через `push_back`

2. **Скользящее окно волатильности**: реализуй расчёт стандартного отклонения цен в скользящем окне размером N

3. **Буфер тиков**: создай структуру, которая хранит последние 1000 тиков и позволяет получить:
   - Последние N тиков
   - Среднюю цену за окно
   - Максимальный спред за период

4. **Matching Engine**: реализуй простой движок сопоставления ордеров, используя две VecDeque для bid и ask ордеров. При добавлении нового ордера проверяй возможность исполнения.

## Навигация

[← Предыдущий день](../086-btreemap-sorted-prices/ru.md) | [Следующий день →](../088-binaryheap-priority-queue/ru.md)
