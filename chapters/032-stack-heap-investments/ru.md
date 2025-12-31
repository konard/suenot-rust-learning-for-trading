# День 32: Стек и куча — краткосрочные и долгосрочные инвестиции

## Аналогия из трейдинга

Представь, что у тебя есть два типа инвестиций:

- **Стек (Stack)** — это как **дневная торговля (day trading)**. Ты быстро открываешь позицию, закрываешь её в тот же день и освобождаешь капитал. Всё происходит быстро, предсказуемо, и ты точно знаешь, когда позиция закроется.

- **Куча (Heap)** — это как **долгосрочные инвестиции**. Ты покупаешь акции и держишь их неопределённое время. Размер позиции может меняться (докупаешь или продаёшь частично), и ты не знаешь заранее, когда закроешь позицию полностью.

## Что такое стек?

Стек — это область памяти, которая работает по принципу **LIFO** (Last In, First Out — последним пришёл, первым ушёл):

```rust
fn main() {
    let btc_price = 42000.0;        // Кладём на стек
    let eth_price = 2200.0;         // Кладём на стек
    calculate_ratio(btc_price, eth_price);
}   // eth_price удаляется, затем btc_price

fn calculate_ratio(btc: f64, eth: f64) -> f64 {
    let ratio = btc / eth;          // ratio на стеке
    ratio                           // Возвращаем, ratio удаляется
}
```

**Характеристики стека:**
- Очень быстрое выделение и освобождение памяти
- Фиксированный размер данных (известен при компиляции)
- Автоматическая очистка при выходе из scope

## Что такое куча?

Куча — это область памяти для данных **динамического размера**:

```rust
fn main() {
    // String хранит данные в куче
    let ticker = String::from("BTC/USDT");

    // Vec хранит элементы в куче
    let mut prices: Vec<f64> = Vec::new();
    prices.push(42000.0);
    prices.push(42100.0);
    prices.push(42050.0);

    println!("Ticker: {}, Prices: {:?}", ticker, prices);
}
```

**Характеристики кучи:**
- Медленнее стека (нужно искать свободное место)
- Гибкий размер данных (может расти и уменьшаться)
- Требует явного или автоматического освобождения

## Визуализация: стек vs куча

```
┌─────────────────────────────────────────────────────────────────┐
│                           ПАМЯТЬ                                 │
├──────────────────────────────┬──────────────────────────────────┤
│           СТЕК               │              КУЧА                │
│    (Stack - Day Trading)     │     (Heap - Long Positions)      │
├──────────────────────────────┼──────────────────────────────────┤
│                              │                                   │
│  ┌────────────────────┐      │    ┌─────────────────────────┐   │
│  │ ratio: f64         │      │    │ "BTC/USDT"              │   │
│  │ 19.09              │      │    └─────────────────────────┘   │
│  └────────────────────┘      │              ▲                   │
│  ┌────────────────────┐      │              │ ptr               │
│  │ eth_price: f64     │      │    ┌─────────────────────────┐   │
│  │ 2200.0             │      │    │ prices: [42000, 42100,  │   │
│  └────────────────────┘      │    │          42050]         │   │
│  ┌────────────────────┐      │    └─────────────────────────┘   │
│  │ btc_price: f64     │      │              ▲                   │
│  │ 42000.0            │      │              │ ptr               │
│  └────────────────────┘      │                                   │
│  ┌────────────────────┐      │                                   │
│  │ ticker: String     │──────┼──────────────┘                   │
│  │ (ptr, len, cap)    │      │                                   │
│  └────────────────────┘      │                                   │
│  ┌────────────────────┐      │                                   │
│  │ prices: Vec<f64>   │──────┼──────────────┘                   │
│  │ (ptr, len, cap)    │      │                                   │
│  └────────────────────┘      │                                   │
│                              │                                   │
│  Быстро, фиксированный       │  Медленнее, динамический размер  │
│  размер                      │                                   │
└──────────────────────────────┴──────────────────────────────────┘
```

## Типы данных: где хранятся?

### На стеке (фиксированный размер)

```rust
fn main() {
    // Примитивные типы — всегда на стеке
    let price: f64 = 42000.0;           // 8 байт
    let quantity: i32 = 100;             // 4 байта
    let is_bullish: bool = true;         // 1 байт
    let symbol: char = '₿';              // 4 байта

    // Кортежи фиксированного размера — на стеке
    let trade: (f64, f64, bool) = (42000.0, 0.5, true);

    // Массивы фиксированного размера — на стеке
    let last_5_prices: [f64; 5] = [41900.0, 42000.0, 42100.0, 42050.0, 42080.0];

    println!("Price: {}, Qty: {}, Bullish: {}", price, quantity, is_bullish);
}
```

### В куче (динамический размер)

```rust
fn main() {
    // String — динамическая строка, данные в куче
    let ticker = String::from("ETH/USDT");

    // Vec — динамический массив, данные в куче
    let mut order_book: Vec<(f64, f64)> = Vec::new();
    order_book.push((42000.0, 1.5));
    order_book.push((41999.0, 2.3));
    order_book.push((41998.0, 0.8));

    // Box — явное размещение в куче
    let big_order = Box::new(Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 10.0,
        side: OrderSide::Buy,
    });

    println!("Ticker: {}", ticker);
    println!("Order book depth: {}", order_book.len());
    println!("Big order price: {}", big_order.price);
}

struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
}

enum OrderSide {
    Buy,
    Sell,
}
```

## Практический пример: Order Book

```rust
fn main() {
    let mut order_book = OrderBook::new("BTC/USDT");

    // Добавляем ордера (данные растут в куче)
    order_book.add_bid(41990.0, 1.5);
    order_book.add_bid(41980.0, 2.3);
    order_book.add_bid(41970.0, 0.8);

    order_book.add_ask(42000.0, 1.0);
    order_book.add_ask(42010.0, 2.0);
    order_book.add_ask(42020.0, 1.5);

    order_book.print_book();

    // Рассчитываем спред (использует стек для вычислений)
    if let Some(spread) = order_book.calculate_spread() {
        println!("\nSpread: ${:.2} ({:.4}%)", spread.0, spread.1);
    }
}

struct OrderBook {
    symbol: String,           // Хранит данные в куче
    bids: Vec<(f64, f64)>,   // Хранит данные в куче
    asks: Vec<(f64, f64)>,   // Хранит данные в куче
}

impl OrderBook {
    fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: String::from(symbol),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, quantity: f64) {
        self.bids.push((price, quantity));
        // Сортируем по убыванию цены
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    fn add_ask(&mut self, price: f64, quantity: f64) {
        self.asks.push((price, quantity));
        // Сортируем по возрастанию цены
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    fn calculate_spread(&self) -> Option<(f64, f64)> {
        // Эти переменные на стеке
        let best_bid = self.bids.first()?.0;
        let best_ask = self.asks.first()?.0;
        let spread = best_ask - best_bid;
        let spread_pct = (spread / best_bid) * 100.0;
        Some((spread, spread_pct))  // Возвращаем кортеж (копируется)
    }

    fn print_book(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║     ORDER BOOK: {:^20} ║", self.symbol);
        println!("╠═══════════════════════════════════════╣");
        println!("║  ASKS (Sell orders)                   ║");
        for (price, qty) in self.asks.iter().rev().take(3) {
            println!("║  ${:>10.2} | {:>8.4} BTC         ║", price, qty);
        }
        println!("╠═══════════════════════════════════════╣");
        println!("║  BIDS (Buy orders)                    ║");
        for (price, qty) in self.bids.iter().take(3) {
            println!("║  ${:>10.2} | {:>8.4} BTC         ║", price, qty);
        }
        println!("╚═══════════════════════════════════════╝");
    }
}
```

## Почему это важно для трейдинга?

### 1. Производительность критична

```rust
fn main() {
    // Быстро: данные на стеке
    let prices: [f64; 1000] = [42000.0; 1000];
    let sum_stack: f64 = prices.iter().sum();

    // Медленнее: данные в куче (но гибко)
    let prices_vec: Vec<f64> = vec![42000.0; 1000];
    let sum_heap: f64 = prices_vec.iter().sum();

    println!("Stack sum: {}", sum_stack);
    println!("Heap sum: {}", sum_heap);
}
```

### 2. Выбор структуры данных

```rust
fn main() {
    // Если размер известен заранее — используй массив (стек)
    let ohlc: [f64; 4] = [42000.0, 42500.0, 41800.0, 42200.0];

    // Если размер меняется — используй Vec (куча)
    let mut trade_history: Vec<Trade> = Vec::new();
    trade_history.push(Trade::new(42000.0, 0.5, true));
    trade_history.push(Trade::new(42100.0, 0.3, false));

    // Если нужна фиксированная строка — &str (стек/статическая память)
    let symbol: &str = "BTC";

    // Если строка создаётся динамически — String (куча)
    let full_symbol = format!("{}/USDT", symbol);

    println!("OHLC: {:?}", ohlc);
    println!("Symbol: {}", full_symbol);
}

struct Trade {
    price: f64,
    quantity: f64,
    is_buy: bool,
}

impl Trade {
    fn new(price: f64, quantity: f64, is_buy: bool) -> Self {
        Trade { price, quantity, is_buy }
    }
}
```

### 3. Работа с большими данными

```rust
fn main() {
    // Для больших структур используй Box (размещение в куче)
    let market_data = Box::new(MarketData::new("BTC/USDT"));

    println!("Symbol: {}", market_data.symbol);
    println!("Candle count: {}", market_data.candles.len());
}

struct MarketData {
    symbol: String,
    candles: Vec<Candle>,
    order_book: OrderBookSnapshot,
    trades: Vec<TradeRecord>,
}

struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct OrderBookSnapshot {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

struct TradeRecord {
    price: f64,
    quantity: f64,
    is_buyer_maker: bool,
    timestamp: u64,
}

impl MarketData {
    fn new(symbol: &str) -> Self {
        MarketData {
            symbol: String::from(symbol),
            candles: Vec::new(),
            order_book: OrderBookSnapshot {
                bids: Vec::new(),
                asks: Vec::new(),
            },
            trades: Vec::new(),
        }
    }
}
```

## Сравнение: когда что использовать

```rust
fn main() {
    // ✅ Стек: простые расчёты
    let entry_price: f64 = 42000.0;
    let exit_price: f64 = 43000.0;
    let pnl = exit_price - entry_price;

    // ✅ Стек: фиксированные данные
    let ohlc: [f64; 4] = [42000.0, 42500.0, 41800.0, 42200.0];

    // ✅ Куча: динамический список
    let mut positions: Vec<Position> = Vec::new();
    positions.push(Position { symbol: String::from("BTC"), size: 0.5 });

    // ✅ Куча: строки, создаваемые в runtime
    let report = generate_report(&positions);

    println!("PnL: {}", pnl);
    println!("Report: {}", report);
}

struct Position {
    symbol: String,
    size: f64,
}

fn generate_report(positions: &[Position]) -> String {
    let mut report = String::from("Portfolio:\n");
    for pos in positions {
        report.push_str(&format!("  {} : {}\n", pos.symbol, pos.size));
    }
    report
}
```

## Что мы узнали

| Характеристика | Стек | Куча |
|----------------|------|------|
| Аналогия | Day Trading | Долгосрочные позиции |
| Скорость | Очень быстро | Медленнее |
| Размер данных | Известен при компиляции | Динамический |
| Время жизни | До конца scope | Пока есть владелец |
| Типы | i32, f64, bool, [T; N] | String, Vec<T>, Box<T> |
| Управление | Автоматическое | Через ownership |

## Домашнее задание

1. Создай структуру `Portfolio`, которая хранит список позиций в куче (Vec) и рассчитывает общую стоимость, используя локальные переменные на стеке

2. Напиши функцию, которая принимает массив цен фиксированного размера `[f64; 10]` (стек) и возвращает `Vec<f64>` с скользящими средними (куча)

3. Реализуй `TradeJournal`, который:
   - Хранит историю сделок в `Vec<Trade>` (куча)
   - Использует локальные переменные для расчётов (стек)
   - Возвращает `String` с отчётом (куча)

4. Сравни производительность: создай массив `[f64; 10000]` на стеке и `Vec<f64>` с 10000 элементами в куче, измерь время суммирования

## Навигация

[← Предыдущий день](../031-ownership-asset-ownership/ru.md) | [Следующий день →](../033-references-borrowing/ru.md)
