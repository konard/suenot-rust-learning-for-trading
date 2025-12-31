# День 279: Эмуляция исполнения ордеров

## Аналогия из трейдинга

Представь, что ты разрабатываешь торговую стратегию и хочешь проверить, как она работала бы на исторических данных. Но есть проблема: в реальности, когда ты отправляешь ордер на биржу, происходит множество вещей — задержки сети, проскальзывание цены, частичное исполнение, очередь в стакане заявок. Если в бэктесте просто "исполнять" ордера мгновенно по желаемой цене, результаты будут нереалистично хорошими.

**Эмуляция исполнения ордеров** — это симуляция того, как ордера были бы исполнены в реальных рыночных условиях. Это ключевой компонент качественного бэктестинга, который делает разницу между "бумажной прибылью" и реальными результатами.

Основные факторы, которые нужно эмулировать:
- **Проскальзывание (Slippage)** — разница между ожидаемой и фактической ценой исполнения
- **Частичное исполнение (Partial Fill)** — когда объём заявки больше доступной ликвидности
- **Задержка исполнения (Latency)** — время между отправкой ордера и его исполнением
- **Позиция в очереди (Queue Position)** — для лимитных ордеров важно, кто был раньше

## Модели исполнения ордеров

### 1. Наивная модель (мгновенное исполнение)

Самая простая, но нереалистичная модель:

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>, // None для рыночных ордеров
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

// Наивная модель — просто исполняем по текущей цене
fn naive_execute(order: &Order, current_price: f64) -> Fill {
    Fill {
        order_id: order.id,
        price: current_price, // Нереалистично!
        quantity: order.quantity,
        timestamp: 0,
    }
}

fn main() {
    let order = Order {
        id: 1,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 10.0,
        price: None,
    };

    let fill = naive_execute(&order, 42000.0);
    println!("Наивное исполнение: {:?}", fill);
    // Проблема: в реальности покупка 10 BTC сдвинет цену!
}
```

### 2. Модель с проскальзыванием

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    slippage: f64,
}

#[derive(Debug, Clone)]
struct SlippageModel {
    // Базовое проскальзывание в процентах
    base_slippage_pct: f64,
    // Дополнительное проскальзывание за каждые N единиц объёма
    volume_impact_pct: f64,
    volume_unit: f64,
}

impl SlippageModel {
    fn new(base_slippage: f64, volume_impact: f64, volume_unit: f64) -> Self {
        SlippageModel {
            base_slippage_pct: base_slippage,
            volume_impact_pct: volume_impact,
            volume_unit: volume_unit,
        }
    }

    fn calculate_slippage(&self, order: &Order) -> f64 {
        let volume_multiplier = order.quantity / self.volume_unit;
        let total_slippage = self.base_slippage_pct +
            (volume_multiplier * self.volume_impact_pct);

        // Проскальзывание всегда против нас
        match order.side {
            OrderSide::Buy => total_slippage,   // Покупаем дороже
            OrderSide::Sell => -total_slippage, // Продаём дешевле
        }
    }

    fn execute(&self, order: &Order, mid_price: f64) -> Fill {
        let slippage_pct = self.calculate_slippage(order);
        let execution_price = mid_price * (1.0 + slippage_pct / 100.0);

        Fill {
            order_id: order.id,
            price: execution_price,
            quantity: order.quantity,
            slippage: slippage_pct,
        }
    }
}

fn main() {
    // Модель: 0.05% базовое проскальзывание + 0.01% за каждый 1 BTC
    let slippage_model = SlippageModel::new(0.05, 0.01, 1.0);

    let small_order = Order {
        id: 1,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 0.1, // Маленький ордер
        price: None,
    };

    let large_order = Order {
        id: 2,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 50.0, // Большой ордер
        price: None,
    };

    let mid_price = 42000.0;

    let fill_small = slippage_model.execute(&small_order, mid_price);
    let fill_large = slippage_model.execute(&large_order, mid_price);

    println!("Маленький ордер (0.1 BTC):");
    println!("  Цена исполнения: ${:.2}", fill_small.price);
    println!("  Проскальзывание: {:.4}%", fill_small.slippage);

    println!("\nБольшой ордер (50 BTC):");
    println!("  Цена исполнения: ${:.2}", fill_large.price);
    println!("  Проскальзывание: {:.4}%", fill_large.slippage);
}
```

## Эмуляция стакана заявок

Для более реалистичной симуляции нужно моделировать стакан заявок:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct PriceLevel {
    price: f64,
    quantity: f64,
    order_count: u32,
}

#[derive(Debug, Clone)]
struct OrderBook {
    // Биды отсортированы по убыванию цены (лучшая цена = самая высокая)
    bids: BTreeMap<i64, PriceLevel>, // Используем i64 для ключа (цена * 100)
    // Аски отсортированы по возрастанию цены (лучшая цена = самая низкая)
    asks: BTreeMap<i64, PriceLevel>,
    tick_size: f64,
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    fills: Vec<(f64, f64)>, // (цена, количество)
    average_price: f64,
    total_quantity: f64,
    unfilled_quantity: f64,
}

impl OrderBook {
    fn new(tick_size: f64) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            tick_size,
        }
    }

    fn price_to_key(&self, price: f64) -> i64 {
        (price / self.tick_size).round() as i64
    }

    fn add_level(&mut self, side: OrderSide, price: f64, quantity: f64, order_count: u32) {
        let key = self.price_to_key(price);
        let level = PriceLevel {
            price,
            quantity,
            order_count,
        };

        match side {
            OrderSide::Buy => {
                self.bids.insert(-key, level); // Отрицательный ключ для сортировки по убыванию
            }
            OrderSide::Sell => {
                self.asks.insert(key, level);
            }
        }
    }

    fn best_bid(&self) -> Option<f64> {
        self.bids.values().next().map(|l| l.price)
    }

    fn best_ask(&self) -> Option<f64> {
        self.asks.values().next().map(|l| l.price)
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    // Симуляция исполнения рыночного ордера
    fn execute_market_order(&self, side: OrderSide, quantity: f64) -> ExecutionResult {
        let mut fills = Vec::new();
        let mut remaining = quantity;
        let mut total_cost = 0.0;

        // Выбираем нужную сторону стакана
        let levels: Vec<&PriceLevel> = match side {
            OrderSide::Buy => self.asks.values().collect(),
            OrderSide::Sell => self.bids.values().collect(),
        };

        for level in levels {
            if remaining <= 0.0 {
                break;
            }

            let fill_qty = remaining.min(level.quantity);
            fills.push((level.price, fill_qty));
            total_cost += level.price * fill_qty;
            remaining -= fill_qty;
        }

        let total_filled = quantity - remaining;
        let average_price = if total_filled > 0.0 {
            total_cost / total_filled
        } else {
            0.0
        };

        ExecutionResult {
            fills,
            average_price,
            total_quantity: total_filled,
            unfilled_quantity: remaining,
        }
    }
}

fn main() {
    let mut order_book = OrderBook::new(0.01);

    // Добавляем уровни в стакан (типичный стакан BTC/USDT)
    order_book.add_level(OrderSide::Sell, 42010.0, 2.5, 15);
    order_book.add_level(OrderSide::Sell, 42005.0, 1.2, 8);
    order_book.add_level(OrderSide::Sell, 42001.0, 0.5, 3);

    order_book.add_level(OrderSide::Buy, 41999.0, 0.8, 5);
    order_book.add_level(OrderSide::Buy, 41995.0, 1.5, 10);
    order_book.add_level(OrderSide::Buy, 41990.0, 3.0, 20);

    println!("=== Стакан заявок ===");
    println!("Лучший бид: ${:.2}", order_book.best_bid().unwrap());
    println!("Лучший аск: ${:.2}", order_book.best_ask().unwrap());
    println!("Спред: ${:.2}", order_book.spread().unwrap());

    // Симулируем покупку 2 BTC
    println!("\n=== Покупка 2 BTC рыночным ордером ===");
    let result = order_book.execute_market_order(OrderSide::Buy, 2.0);

    println!("Исполнения:");
    for (price, qty) in &result.fills {
        println!("  {:.4} BTC @ ${:.2}", qty, price);
    }
    println!("Средняя цена: ${:.2}", result.average_price);
    println!("Исполнено: {:.4} BTC", result.total_quantity);

    // Симулируем покупку 10 BTC (больше ликвидности)
    println!("\n=== Покупка 10 BTC рыночным ордером ===");
    let result_large = order_book.execute_market_order(OrderSide::Buy, 10.0);

    println!("Исполнения:");
    for (price, qty) in &result_large.fills {
        println!("  {:.4} BTC @ ${:.2}", qty, price);
    }
    println!("Средняя цена: ${:.2}", result_large.average_price);
    println!("Исполнено: {:.4} BTC", result_large.total_quantity);
    println!("Не исполнено: {:.4} BTC", result_large.unfilled_quantity);
}
```

## Эмуляция лимитных ордеров

Лимитные ордера требуют особой логики — они исполняются только когда цена достигает указанного уровня:

```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
struct LimitOrder {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
    filled_quantity: f64,
    timestamp: u64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
struct MarketTick {
    timestamp: u64,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct LimitOrderEmulator {
    pending_orders: HashMap<u64, LimitOrder>,
    fills: Vec<Fill>,
    fill_probability: f64, // Вероятность исполнения при касании цены
    next_order_id: u64,
}

impl LimitOrderEmulator {
    fn new(fill_probability: f64) -> Self {
        LimitOrderEmulator {
            pending_orders: HashMap::new(),
            fills: Vec::new(),
            fill_probability,
            next_order_id: 1,
        }
    }

    fn place_order(&mut self, side: OrderSide, price: f64, quantity: f64, timestamp: u64) -> u64 {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let order = LimitOrder {
            id: order_id,
            side,
            price,
            quantity,
            filled_quantity: 0.0,
            timestamp,
            status: OrderStatus::Pending,
        };

        self.pending_orders.insert(order_id, order);
        order_id
    }

    fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some(order) = self.pending_orders.get_mut(&order_id) {
            order.status = OrderStatus::Cancelled;
            self.pending_orders.remove(&order_id);
            true
        } else {
            false
        }
    }

    fn process_tick(&mut self, tick: &MarketTick) -> Vec<Fill> {
        let mut new_fills = Vec::new();
        let mut orders_to_remove = Vec::new();

        for (order_id, order) in self.pending_orders.iter_mut() {
            let should_fill = match order.side {
                // Покупка исполняется, когда аск <= цены ордера
                OrderSide::Buy => tick.ask <= order.price,
                // Продажа исполняется, когда бид >= цены ордера
                OrderSide::Sell => tick.bid >= order.price,
            };

            if should_fill {
                // Симулируем вероятность исполнения
                // В реальности это зависит от позиции в очереди
                let random_value = (tick.timestamp % 100) as f64 / 100.0;

                if random_value < self.fill_probability {
                    // Определяем объём исполнения на основе объёма тика
                    let available_volume = tick.volume * 0.1; // Предполагаем 10% объёма доступно
                    let fill_quantity = (order.quantity - order.filled_quantity)
                        .min(available_volume);

                    if fill_quantity > 0.0 {
                        let fill = Fill {
                            order_id: *order_id,
                            price: order.price,
                            quantity: fill_quantity,
                            timestamp: tick.timestamp,
                        };

                        order.filled_quantity += fill_quantity;
                        new_fills.push(fill);

                        if order.filled_quantity >= order.quantity {
                            order.status = OrderStatus::Filled;
                            orders_to_remove.push(*order_id);
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                        }
                    }
                }
            }
        }

        // Удаляем полностью исполненные ордера
        for order_id in orders_to_remove {
            self.pending_orders.remove(&order_id);
        }

        self.fills.extend(new_fills.clone());
        new_fills
    }

    fn get_order_status(&self, order_id: u64) -> Option<&LimitOrder> {
        self.pending_orders.get(&order_id)
    }
}

fn main() {
    let mut emulator = LimitOrderEmulator::new(0.7); // 70% вероятность исполнения

    // Размещаем лимитные ордера
    let buy_order = emulator.place_order(OrderSide::Buy, 41500.0, 1.0, 0);
    let sell_order = emulator.place_order(OrderSide::Sell, 42500.0, 0.5, 0);

    println!("Размещены ордера:");
    println!("  Покупка: id={}, 1 BTC @ $41,500", buy_order);
    println!("  Продажа: id={}, 0.5 BTC @ $42,500", sell_order);

    // Симулируем рыночные тики
    let ticks = vec![
        MarketTick { timestamp: 1, bid: 42000.0, ask: 42010.0, last_price: 42005.0, volume: 5.0 },
        MarketTick { timestamp: 2, bid: 41600.0, ask: 41610.0, last_price: 41605.0, volume: 3.0 },
        MarketTick { timestamp: 3, bid: 41480.0, ask: 41490.0, last_price: 41485.0, volume: 8.0 },
        MarketTick { timestamp: 4, bid: 41700.0, ask: 41710.0, last_price: 41705.0, volume: 2.0 },
        MarketTick { timestamp: 5, bid: 42400.0, ask: 42410.0, last_price: 42405.0, volume: 4.0 },
        MarketTick { timestamp: 6, bid: 42510.0, ask: 42520.0, last_price: 42515.0, volume: 6.0 },
    ];

    println!("\nОбработка рыночных тиков:");
    for tick in &ticks {
        let fills = emulator.process_tick(tick);

        println!("\nTick {}: bid=${:.0}, ask=${:.0}, volume={:.1}",
                 tick.timestamp, tick.bid, tick.ask, tick.volume);

        for fill in fills {
            println!("  >>> ИСПОЛНЕНИЕ: ордер {}, {:.4} @ ${:.2}",
                     fill.order_id, fill.quantity, fill.price);
        }
    }

    println!("\n=== Итоговый отчёт ===");
    println!("Всего исполнений: {}", emulator.fills.len());
    for fill in &emulator.fills {
        println!("  Ордер {}: {:.4} @ ${:.2} (t={})",
                 fill.order_id, fill.quantity, fill.price, fill.timestamp);
    }
}
```

## Полная система эмуляции исполнения

Объединим все компоненты в единую систему:

```rust
use std::collections::HashMap;

// ============ Типы данных ============

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Rejected,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    limit_price: Option<f64>,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    commission: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// ============ Конфигурация эмуляции ============

#[derive(Debug, Clone)]
struct ExecutionConfig {
    // Проскальзывание
    base_slippage_bps: f64,      // В базисных пунктах (1 bp = 0.01%)
    volume_impact_bps: f64,      // Дополнительное проскальзывание за объём

    // Комиссии
    maker_fee_pct: f64,          // Комиссия мейкера
    taker_fee_pct: f64,          // Комиссия тейкера

    // Исполнение лимитных ордеров
    limit_fill_probability: f64, // Вероятность исполнения лимитника
    partial_fill_enabled: bool,  // Разрешить частичное исполнение

    // Задержка
    execution_delay_ms: u64,     // Задержка исполнения в мс
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            base_slippage_bps: 5.0,        // 0.05%
            volume_impact_bps: 1.0,        // 0.01% за единицу
            maker_fee_pct: 0.02,           // 0.02%
            taker_fee_pct: 0.04,           // 0.04%
            limit_fill_probability: 0.8,
            partial_fill_enabled: true,
            execution_delay_ms: 50,
        }
    }
}

// ============ Движок эмуляции ============

#[derive(Debug)]
struct ExecutionEmulator {
    config: ExecutionConfig,
    pending_orders: HashMap<u64, Order>,
    fills: Vec<Fill>,
    next_order_id: u64,
}

impl ExecutionEmulator {
    fn new(config: ExecutionConfig) -> Self {
        ExecutionEmulator {
            config,
            pending_orders: HashMap::new(),
            fills: Vec::new(),
            next_order_id: 1,
        }
    }

    fn submit_order(&mut self, order: Order) -> u64 {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let mut order = order;
        order.id = order_id;

        self.pending_orders.insert(order_id, order);
        order_id
    }

    fn calculate_slippage(&self, order: &Order, base_price: f64) -> f64 {
        let slippage_bps = self.config.base_slippage_bps +
            (order.quantity * self.config.volume_impact_bps);

        let slippage_multiplier = slippage_bps / 10000.0;

        match order.side {
            OrderSide::Buy => base_price * (1.0 + slippage_multiplier),
            OrderSide::Sell => base_price * (1.0 - slippage_multiplier),
        }
    }

    fn calculate_commission(&self, order: &Order, fill_price: f64, quantity: f64) -> f64 {
        let fee_pct = match order.order_type {
            OrderType::Market => self.config.taker_fee_pct,
            OrderType::Limit => self.config.maker_fee_pct,
        };

        fill_price * quantity * (fee_pct / 100.0)
    }

    fn process_candle(&mut self, candle: &OHLCV) -> Vec<Fill> {
        let mut new_fills = Vec::new();
        let mut orders_to_remove = Vec::new();

        for (order_id, order) in self.pending_orders.iter() {
            match order.order_type {
                OrderType::Market => {
                    // Рыночные ордера исполняются сразу
                    let execution_price = self.calculate_slippage(order, candle.open);
                    let commission = self.calculate_commission(order, execution_price, order.quantity);

                    new_fills.push(Fill {
                        order_id: *order_id,
                        price: execution_price,
                        quantity: order.quantity,
                        commission,
                        timestamp: candle.timestamp + self.config.execution_delay_ms,
                    });

                    orders_to_remove.push(*order_id);
                }

                OrderType::Limit => {
                    if let Some(limit_price) = order.limit_price {
                        let price_touched = match order.side {
                            OrderSide::Buy => candle.low <= limit_price,
                            OrderSide::Sell => candle.high >= limit_price,
                        };

                        if price_touched {
                            // Проверяем вероятность исполнения
                            let random = ((candle.timestamp * 7 + *order_id * 13) % 100) as f64 / 100.0;

                            if random < self.config.limit_fill_probability {
                                let fill_quantity = if self.config.partial_fill_enabled {
                                    // Частичное исполнение на основе объёма свечи
                                    order.quantity.min(candle.volume * 0.1)
                                } else {
                                    order.quantity
                                };

                                let commission = self.calculate_commission(order, limit_price, fill_quantity);

                                new_fills.push(Fill {
                                    order_id: *order_id,
                                    price: limit_price,
                                    quantity: fill_quantity,
                                    commission,
                                    timestamp: candle.timestamp + self.config.execution_delay_ms,
                                });

                                if fill_quantity >= order.quantity {
                                    orders_to_remove.push(*order_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        for order_id in orders_to_remove {
            self.pending_orders.remove(&order_id);
        }

        self.fills.extend(new_fills.clone());
        new_fills
    }

    fn get_statistics(&self) -> ExecutionStatistics {
        let total_volume: f64 = self.fills.iter().map(|f| f.quantity * f.price).sum();
        let total_commission: f64 = self.fills.iter().map(|f| f.commission).sum();

        ExecutionStatistics {
            total_fills: self.fills.len(),
            total_volume,
            total_commission,
            average_fill_size: if self.fills.is_empty() {
                0.0
            } else {
                total_volume / self.fills.len() as f64
            },
        }
    }
}

#[derive(Debug)]
struct ExecutionStatistics {
    total_fills: usize,
    total_volume: f64,
    total_commission: f64,
    average_fill_size: f64,
}

fn main() {
    let config = ExecutionConfig {
        base_slippage_bps: 5.0,
        volume_impact_bps: 2.0,
        maker_fee_pct: 0.02,
        taker_fee_pct: 0.04,
        limit_fill_probability: 0.75,
        partial_fill_enabled: true,
        execution_delay_ms: 100,
    };

    let mut emulator = ExecutionEmulator::new(config);

    // Отправляем ордера
    let market_buy = Order {
        id: 0,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 0.5,
        limit_price: None,
        timestamp: 0,
    };

    let limit_sell = Order {
        id: 0,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Sell,
        order_type: OrderType::Limit,
        quantity: 0.5,
        limit_price: Some(43000.0),
        timestamp: 0,
    };

    let id1 = emulator.submit_order(market_buy);
    let id2 = emulator.submit_order(limit_sell);

    println!("Отправлены ордера: Market Buy (id={}), Limit Sell (id={})", id1, id2);

    // Симулируем свечи
    let candles = vec![
        OHLCV { timestamp: 1000, open: 42000.0, high: 42500.0, low: 41800.0, close: 42200.0, volume: 100.0 },
        OHLCV { timestamp: 2000, open: 42200.0, high: 42800.0, low: 42100.0, close: 42700.0, volume: 80.0 },
        OHLCV { timestamp: 3000, open: 42700.0, high: 43200.0, low: 42600.0, close: 43100.0, volume: 120.0 },
    ];

    println!("\n=== Обработка свечей ===");
    for candle in &candles {
        println!("\nСвеча: O={:.0} H={:.0} L={:.0} C={:.0} V={:.0}",
                 candle.open, candle.high, candle.low, candle.close, candle.volume);

        let fills = emulator.process_candle(candle);
        for fill in fills {
            println!("  FILL: ордер {} | {:.4} @ ${:.2} | комиссия: ${:.4}",
                     fill.order_id, fill.quantity, fill.price, fill.commission);
        }
    }

    let stats = emulator.get_statistics();
    println!("\n=== Статистика исполнения ===");
    println!("Всего исполнений: {}", stats.total_fills);
    println!("Общий объём: ${:.2}", stats.total_volume);
    println!("Общая комиссия: ${:.4}", stats.total_commission);
    println!("Средний размер: ${:.2}", stats.average_fill_size);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Проскальзывание | Разница между ожидаемой и фактической ценой исполнения |
| Частичное исполнение | Когда ордер исполняется не полностью из-за недостатка ликвидности |
| Стакан заявок | Модель для симуляции исполнения по нескольким ценовым уровням |
| Лимитные ордера | Требуют проверки достижения цены и позиции в очереди |
| Комиссии | Maker/Taker модель комиссий биржи |
| Задержка исполнения | Время между отправкой и исполнением ордера |

## Практические упражнения

### Упражнение 1: Модель влияния на рынок
Реализуй модель, где большие ордера влияют на цену:
```rust
// Подсказка: используй формулу квадратного корня
// impact = sqrt(order_size / average_volume) * impact_coefficient
```

### Упражнение 2: Эмуляция очереди
Добавь симуляцию очереди для лимитных ордеров — ордера, размещённые раньше, исполняются первыми.

### Упражнение 3: Стоп-ордера
Реализуй поддержку стоп-ордеров (Stop-Loss и Take-Profit), которые превращаются в рыночные при достижении триггерной цены.

## Домашнее задание

1. **Реалистичный стакан**: Создай генератор синтетического стакана заявок с реалистичным распределением ликвидности (больше ликвидности около текущей цены, меньше — дальше).

2. **Сравнение моделей**: Прогони одну и ту же стратегию через разные модели исполнения (наивная, с проскальзыванием, с полным стаканом) и сравни результаты PnL.

3. **Latency Arbitrage**: Симулируй ситуацию, когда задержка исполнения влияет на прибыльность стратегии. Найди порог задержки, при котором стратегия становится убыточной.

4. **Fill Ratio анализ**: Реализуй трекинг и анализ коэффициента исполнения (какой процент лимитных ордеров исполняется) для разных расстояний от текущей цены.

## Навигация

[← Предыдущий день](../278-backtesting-framework-core/ru.md) | [Следующий день →](../280-slippage-models-realistic-fills/ru.md)
