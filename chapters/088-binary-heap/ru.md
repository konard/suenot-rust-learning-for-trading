# День 88: BinaryHeap — приоритетная очередь

## Аналогия из трейдинга

Представьте биржевой стакан: ордера на покупку сортируются по цене от высокой к низкой (лучшая цена — наверху), а ордера на продажу — от низкой к высокой. Это и есть **приоритетная очередь** — структура данных, где элемент с наивысшим приоритетом всегда доступен первым.

В алготрейдинге приоритетные очереди используются для:
- **Выбора лучших ордеров** — ордер с лучшей ценой обрабатывается первым
- **Приоритизации сигналов** — важные сигналы обрабатываются раньше
- **Управления событиями** — события с более ранним временем исполняются первыми
- **Ранжирования стратегий** — выбор стратегии с лучшей доходностью

## Что такое BinaryHeap?

`BinaryHeap` в Rust — это реализация приоритетной очереди на основе двоичной кучи. По умолчанию это **max-heap**: наибольший элемент всегда на вершине.

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut prices = BinaryHeap::new();

    prices.push(42000);
    prices.push(42500);
    prices.push(41800);
    prices.push(42200);

    // Наибольшая цена извлекается первой
    while let Some(price) = prices.pop() {
        println!("Price: {}", price);
    }
    // Вывод: 42500, 42200, 42000, 41800
}
```

## Основные операции

### Создание и добавление элементов

```rust
use std::collections::BinaryHeap;

fn main() {
    // Создание пустой кучи
    let mut heap: BinaryHeap<i32> = BinaryHeap::new();

    // Создание с заданной ёмкостью
    let mut heap_with_capacity: BinaryHeap<i32> = BinaryHeap::with_capacity(100);

    // Создание из вектора
    let prices = vec![42000, 42500, 41800, 42200];
    let heap_from_vec: BinaryHeap<i32> = BinaryHeap::from(prices);

    // Добавление элементов
    heap.push(100);
    heap.push(200);
    heap.push(150);

    println!("Heap: {:?}", heap);
    println!("Size: {}", heap.len());
}
```

### Извлечение элементов

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut order_priorities = BinaryHeap::from(vec![5, 10, 3, 8, 1]);

    // Посмотреть максимальный элемент (без извлечения)
    if let Some(&top) = order_priorities.peek() {
        println!("Highest priority: {}", top);
    }

    // Извлечь максимальный элемент
    while let Some(priority) = order_priorities.pop() {
        println!("Processing priority: {}", priority);
    }
}
```

## Торговые сигналы с приоритетами

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct TradingSignal {
    priority: u32,      // Приоритет сигнала (чем выше, тем важнее)
    symbol: String,
    signal_type: String,
    price: u64,         // Цена в центах для точности
}

// Реализуем Ord для сравнения по приоритету
impl Ord for TradingSignal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for TradingSignal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut signal_queue = BinaryHeap::new();

    // Добавляем сигналы с разными приоритетами
    signal_queue.push(TradingSignal {
        priority: 5,
        symbol: "BTC/USDT".to_string(),
        signal_type: "BUY".to_string(),
        price: 4200000, // $42,000.00
    });

    signal_queue.push(TradingSignal {
        priority: 10,  // Высокий приоритет — стоп-лосс!
        symbol: "ETH/USDT".to_string(),
        signal_type: "STOP_LOSS".to_string(),
        price: 220000, // $2,200.00
    });

    signal_queue.push(TradingSignal {
        priority: 3,
        symbol: "SOL/USDT".to_string(),
        signal_type: "BUY".to_string(),
        price: 10000, // $100.00
    });

    // Обрабатываем сигналы по приоритету
    println!("=== Processing Signals ===");
    while let Some(signal) = signal_queue.pop() {
        println!(
            "[Priority {}] {} {} @ ${:.2}",
            signal.priority,
            signal.signal_type,
            signal.symbol,
            signal.price as f64 / 100.0
        );
    }
}
```

## Min-Heap: инверсия приоритета

По умолчанию `BinaryHeap` — max-heap. Для min-heap используем `Reverse`:

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn main() {
    // Min-heap для цен Ask (хотим минимальную цену)
    let mut ask_prices: BinaryHeap<Reverse<u64>> = BinaryHeap::new();

    ask_prices.push(Reverse(42010));
    ask_prices.push(Reverse(42005));
    ask_prices.push(Reverse(42020));
    ask_prices.push(Reverse(42008));

    println!("=== Best Ask Prices ===");
    while let Some(Reverse(price)) = ask_prices.pop() {
        println!("Ask: ${:.2}", price as f64 / 100.0);
    }
    // Вывод: 42005, 42008, 42010, 42020 (от меньшего к большему)
}
```

## Моделирование книги ордеров

```rust
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Order {
    price: u64,      // Цена в центах
    quantity: u64,   // Количество
    timestamp: u64,  // Время размещения
    order_id: u64,
}

// Для Bid: сначала высокая цена, при равной — раннее время
impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => other.timestamp.cmp(&self.timestamp), // Раннее время выше
            other => other, // Большая цена выше
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Обёртка для Ask ордеров (min-heap по цене)
#[derive(Debug, Clone, Eq, PartialEq)]
struct AskOrder(Order);

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.0.price.cmp(&self.0.price) { // Инвертируем для min-heap
            Ordering::Equal => other.0.timestamp.cmp(&self.0.timestamp),
            other_ord => other_ord,
        }
    }
}

impl PartialOrd for AskOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut bids: BinaryHeap<Order> = BinaryHeap::new();
    let mut asks: BinaryHeap<AskOrder> = BinaryHeap::new();

    // Добавляем ордера на покупку (Bid)
    bids.push(Order { price: 4200000, quantity: 100, timestamp: 1, order_id: 1 });
    bids.push(Order { price: 4199500, quantity: 200, timestamp: 2, order_id: 2 });
    bids.push(Order { price: 4200000, quantity: 150, timestamp: 3, order_id: 3 });
    bids.push(Order { price: 4198000, quantity: 300, timestamp: 4, order_id: 4 });

    // Добавляем ордера на продажу (Ask)
    asks.push(AskOrder(Order { price: 4201000, quantity: 80, timestamp: 1, order_id: 5 }));
    asks.push(AskOrder(Order { price: 4202500, quantity: 120, timestamp: 2, order_id: 6 }));
    asks.push(AskOrder(Order { price: 4201000, quantity: 90, timestamp: 3, order_id: 7 }));

    println!("╔═══════════════════════════════════════════╗");
    println!("║           ORDER BOOK (BTC/USDT)           ║");
    println!("╠═══════════════════════════════════════════╣");

    // Показываем лучшие Ask (сверху вниз)
    println!("║ {:^10} {:>12} {:>10} {:>6} ║", "Side", "Price", "Qty", "ID");
    println!("╠═══════════════════════════════════════════╣");

    // Клонируем для отображения (чтобы не разрушить очередь)
    let asks_display: Vec<_> = asks.clone().into_sorted_vec();
    for ask in asks_display.iter().rev().take(3) {
        println!(
            "║ {:^10} {:>12.2} {:>10} {:>6} ║",
            "ASK",
            ask.0.price as f64 / 100.0,
            ask.0.quantity,
            ask.0.order_id
        );
    }

    println!("╠═══════════════════════════════════════════╣");

    let bids_display: Vec<_> = bids.clone().into_sorted_vec();
    for bid in bids_display.iter().rev().take(3) {
        println!(
            "║ {:^10} {:>12.2} {:>10} {:>6} ║",
            "BID",
            bid.price as f64 / 100.0,
            bid.quantity,
            bid.order_id
        );
    }

    println!("╚═══════════════════════════════════════════╝");

    // Спред
    if let (Some(best_bid), Some(best_ask)) = (bids.peek(), asks.peek()) {
        let spread = best_ask.0.price - best_bid.price;
        println!("\nSpread: ${:.2}", spread as f64 / 100.0);
    }
}
```

## Очередь событий по времени

```rust
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

#[derive(Debug, Eq, PartialEq)]
struct ScheduledEvent {
    execute_at: u64,    // Время исполнения (timestamp)
    event_type: String,
    data: String,
}

// Min-heap по времени (раннее время — выше приоритет)
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other.execute_at.cmp(&self.execute_at) // Инвертируем для min-heap
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut event_queue = BinaryHeap::new();

    // Планируем события
    event_queue.push(ScheduledEvent {
        execute_at: 1000,
        event_type: "CHECK_STOP_LOSS".to_string(),
        data: "BTC/USDT".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 500,
        event_type: "PLACE_ORDER".to_string(),
        data: "ETH/USDT BUY 1.5".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 750,
        event_type: "CANCEL_ORDER".to_string(),
        data: "Order #12345".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 500,
        event_type: "UPDATE_PRICE".to_string(),
        data: "SOL/USDT".to_string(),
    });

    // Обрабатываем события в порядке времени
    println!("=== Event Timeline ===");
    while let Some(event) = event_queue.pop() {
        println!(
            "[T={}] {}: {}",
            event.execute_at,
            event.event_type,
            event.data
        );
    }
}
```

## Топ-N лучших сделок

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    pnl: f64,
    entry_price: f64,
    exit_price: f64,
}

fn find_top_n_trades(trades: &[Trade], n: usize) -> Vec<Trade> {
    // Используем min-heap размера n для эффективного поиска топ-N
    let mut min_heap: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

    for (idx, trade) in trades.iter().enumerate() {
        // Конвертируем f64 в i64 для сравнения (умножаем на 100 для центов)
        let pnl_cents = (trade.pnl * 100.0) as i64;

        if min_heap.len() < n {
            min_heap.push(Reverse((pnl_cents, idx)));
        } else if let Some(&Reverse((min_pnl, _))) = min_heap.peek() {
            if pnl_cents > min_pnl {
                min_heap.pop();
                min_heap.push(Reverse((pnl_cents, idx)));
            }
        }
    }

    // Извлекаем индексы и сортируем по убыванию PnL
    let mut result: Vec<_> = min_heap
        .into_iter()
        .map(|Reverse((_, idx))| trades[idx].clone())
        .collect();

    result.sort_by(|a, b| b.pnl.partial_cmp(&a.pnl).unwrap());
    result
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC/USDT".to_string(), pnl: 150.50, entry_price: 42000.0, exit_price: 42150.50 },
        Trade { symbol: "ETH/USDT".to_string(), pnl: -30.25, entry_price: 2200.0, exit_price: 2169.75 },
        Trade { symbol: "SOL/USDT".to_string(), pnl: 85.00, entry_price: 100.0, exit_price: 185.00 },
        Trade { symbol: "BTC/USDT".to_string(), pnl: 200.00, entry_price: 41800.0, exit_price: 42000.0 },
        Trade { symbol: "ADA/USDT".to_string(), pnl: 45.75, entry_price: 0.50, exit_price: 0.55 },
        Trade { symbol: "DOT/USDT".to_string(), pnl: -15.00, entry_price: 7.50, exit_price: 7.35 },
        Trade { symbol: "ETH/USDT".to_string(), pnl: 120.00, entry_price: 2100.0, exit_price: 2220.0 },
    ];

    let top_3 = find_top_n_trades(&trades, 3);

    println!("=== Top 3 Profitable Trades ===");
    println!("{:<12} {:>12} {:>12} {:>12}", "Symbol", "Entry", "Exit", "PnL");
    println!("{}", "-".repeat(50));

    for trade in top_3 {
        println!(
            "{:<12} {:>12.2} {:>12.2} {:>+12.2}",
            trade.symbol, trade.entry_price, trade.exit_price, trade.pnl
        );
    }
}
```

## Практический пример: планировщик ребалансировки

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct RebalanceTask {
    urgency: u32,           // Срочность (1-10)
    deviation_percent: u32, // Отклонение от целевого % (в сотых долях)
    symbol: String,
    current_weight: u32,    // Текущий вес в портфеле (в сотых долях %)
    target_weight: u32,     // Целевой вес (в сотых долях %)
}

impl Ord for RebalanceTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Сначала по срочности, затем по отклонению
        match self.urgency.cmp(&other.urgency) {
            Ordering::Equal => self.deviation_percent.cmp(&other.deviation_percent),
            other => other,
        }
    }
}

impl PartialOrd for RebalanceTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut rebalance_queue = BinaryHeap::new();

    // Анализируем портфель и создаём задачи на ребалансировку
    let portfolio = vec![
        ("BTC", 4500, 4000),   // (symbol, current_weight, target_weight) в сотых %
        ("ETH", 2800, 3000),
        ("SOL", 1500, 1500),
        ("ADA", 800, 1000),
        ("DOT", 400, 500),
    ];

    for (symbol, current, target) in portfolio {
        let deviation = if current > target {
            current - target
        } else {
            target - current
        };

        // Срочность зависит от величины отклонения
        let urgency = match deviation {
            0..=100 => 1,
            101..=300 => 3,
            301..=500 => 5,
            501..=1000 => 7,
            _ => 10,
        };

        if deviation > 0 {
            rebalance_queue.push(RebalanceTask {
                urgency,
                deviation_percent: deviation,
                symbol: symbol.to_string(),
                current_weight: current,
                target_weight: target,
            });
        }
    }

    println!("=== Portfolio Rebalancing Queue ===");
    println!("{:<8} {:>10} {:>10} {:>10} {:>10}",
             "Symbol", "Current%", "Target%", "Dev%", "Urgency");
    println!("{}", "-".repeat(55));

    while let Some(task) = rebalance_queue.pop() {
        let action = if task.current_weight > task.target_weight {
            "SELL"
        } else {
            "BUY"
        };

        println!(
            "{:<8} {:>9.2}% {:>9.2}% {:>9.2}% {:>10} → {}",
            task.symbol,
            task.current_weight as f64 / 100.0,
            task.target_weight as f64 / 100.0,
            task.deviation_percent as f64 / 100.0,
            task.urgency,
            action
        );
    }
}
```

## Полезные методы BinaryHeap

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut heap = BinaryHeap::from(vec![3, 1, 4, 1, 5, 9, 2, 6]);

    // Размер и ёмкость
    println!("Length: {}", heap.len());
    println!("Capacity: {}", heap.capacity());
    println!("Is empty: {}", heap.is_empty());

    // Просмотр максимального элемента
    if let Some(&max) = heap.peek() {
        println!("Max element: {}", max);
    }

    // Изменение максимального элемента
    if let Some(max) = heap.peek_mut() {
        *max = 100;
    }
    println!("After peek_mut: {:?}", heap.peek());

    // Очистка
    heap.clear();
    println!("After clear, is empty: {}", heap.is_empty());

    // Создание из итератора
    let prices = vec![42000, 42100, 41900];
    let heap2: BinaryHeap<_> = prices.into_iter().collect();
    println!("From iterator: {:?}", heap2);

    // Преобразование в отсортированный вектор
    let sorted: Vec<_> = heap2.into_sorted_vec();
    println!("Sorted (ascending): {:?}", sorted);
}
```

## Сравнение BinaryHeap с другими коллекциями

| Операция | BinaryHeap | Vec (sorted) | VecDeque |
|----------|------------|--------------|----------|
| push | O(log n) | O(n) | O(1) |
| pop (max/min) | O(log n) | O(1) | O(1) |
| peek | O(1) | O(1) | O(1) |
| Поиск | O(n) | O(log n) | O(n) |
| Применение | Приоритетные очереди | Отсортированные данные | FIFO/LIFO |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `BinaryHeap::new()` | Создание пустой кучи (max-heap) |
| `heap.push(x)` | Добавление элемента |
| `heap.pop()` | Извлечение максимального |
| `heap.peek()` | Просмотр максимального |
| `Reverse(x)` | Инверсия для min-heap |
| `impl Ord` | Кастомный порядок сортировки |

## Домашнее задание

1. **Очередь ордеров**: Создай структуру `LimitOrder` с полями `price`, `quantity`, `timestamp` и `is_buy`. Реализуй две очереди: для Bid (max-heap по цене) и Ask (min-heap по цене). При равной цене приоритет у более раннего ордера.

2. **Топ-5 волатильных активов**: Дан массив структур `Asset { symbol, volatility }`. Используя BinaryHeap, найди 5 активов с наибольшей волатильностью за O(n log 5) времени.

3. **Планировщик сигналов**: Создай систему, где торговые сигналы имеют `priority` (1-10) и `expire_at` (timestamp). Сигнал с истёкшим сроком должен игнорироваться при извлечении. Реализуй методы `add_signal()` и `get_next_valid_signal()`.

4. **Медианный трекер**: Используя два BinaryHeap (max-heap и min-heap), реализуй структуру `MedianTracker` с методами:
   - `add_price(price: f64)` — добавить цену
   - `get_median() -> f64` — получить текущую медиану

   Подсказка: max-heap хранит меньшую половину, min-heap — большую.

## Навигация

[← Предыдущий день](../087-vecdeque-order-queue/ru.md) | [Следующий день →](../089-combining-structs/ru.md)
