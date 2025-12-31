# День 209: Параллельные WebSocket подписки

## Аналогия из трейдинга

Представь, что ты мониторишь сразу несколько криптобирж: Binance, Kraken и Coinbase. На каждой бирже цены на BTC могут отличаться, и арбитражные возможности появляются на миллисекунды. Если ты будешь подключаться к биржам последовательно — сначала Binance, потом Kraken, потом Coinbase — ты потеряешь драгоценное время и упустишь прибыльные сделки.

**Параллельные WebSocket подписки** позволяют подключиться ко всем биржам одновременно и получать обновления цен в реальном времени. Это как иметь трёх трейдеров, каждый из которых следит за своей биржей и мгновенно сообщает об изменениях.

В реальном алготрейдинге это критически важно для:
- **Арбитража** — мониторинг разницы цен между биржами
- **Агрегации ликвидности** — объединение стаканов с разных площадок
- **Мультивалютных стратегий** — отслеживание корреляций между активами
- **Риск-менеджмента** — мониторинг позиций на разных биржах

## Теория: Параллельный запуск задач в Tokio

Tokio предоставляет несколько способов параллельного выполнения задач:

| Метод | Описание | Использование |
|-------|----------|---------------|
| `tokio::spawn` | Запуск независимой задачи | Фоновые операции |
| `tokio::join!` | Ожидание всех задач | Параллельный запуск с ожиданием |
| `tokio::select!` | Ожидание первой задачи | Гонка между задачами |
| `FuturesUnordered` | Динамическое управление | Изменяющийся набор задач |

## Базовый пример: Мониторинг нескольких пар

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    symbol: String,
    buy_exchange: String,
    sell_exchange: String,
    buy_price: f64,
    sell_price: f64,
    spread_percent: f64,
}

// Симуляция WebSocket подключения к бирже
async fn subscribe_to_exchange(
    exchange: &str,
    symbols: Vec<&str>,
    tx: mpsc::Sender<PriceUpdate>,
) {
    println!("[{}] Подключение к WebSocket...", exchange);

    // Симуляция задержки подключения
    sleep(Duration::from_millis(100)).await;
    println!("[{}] WebSocket подключён!", exchange);

    let mut price_interval = interval(Duration::from_millis(500));
    let mut counter = 0u64;

    loop {
        price_interval.tick().await;
        counter += 1;

        for symbol in &symbols {
            // Симуляция получения цены (в реальности это парсинг WebSocket сообщения)
            let base_price = match *symbol {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2200.0,
                "SOL/USDT" => 95.0,
                _ => 100.0,
            };

            // Добавляем случайное отклонение для каждой биржи
            let exchange_offset = match exchange {
                "Binance" => 0.0,
                "Kraken" => 15.0,
                "Coinbase" => -10.0,
                _ => 0.0,
            };

            let price = base_price + exchange_offset + (counter as f64 % 20.0) - 10.0;
            let spread = price * 0.001; // 0.1% спред

            let update = PriceUpdate {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                bid: price - spread / 2.0,
                ask: price + spread / 2.0,
                timestamp: counter,
            };

            if tx.send(update).await.is_err() {
                println!("[{}] Получатель отключён, завершаем", exchange);
                return;
            }
        }
    }
}

// Агрегатор цен со всех бирж
async fn price_aggregator(mut rx: mpsc::Receiver<PriceUpdate>) {
    // Храним последние цены: exchange -> symbol -> PriceUpdate
    let mut prices: HashMap<String, HashMap<String, PriceUpdate>> = HashMap::new();
    let mut update_count = 0;

    println!("\n=== Агрегатор цен запущен ===\n");

    while let Some(update) = rx.recv().await {
        update_count += 1;

        // Сохраняем обновление
        prices
            .entry(update.exchange.clone())
            .or_insert_with(HashMap::new)
            .insert(update.symbol.clone(), update.clone());

        // Каждые 10 обновлений проверяем арбитраж
        if update_count % 10 == 0 {
            check_arbitrage(&prices);
        }

        // Для демонстрации выводим каждое 5-е обновление
        if update_count % 5 == 0 {
            println!(
                "[{}] {}: bid={:.2}, ask={:.2}",
                update.exchange, update.symbol, update.bid, update.ask
            );
        }

        // Останавливаемся после 50 обновлений для демонстрации
        if update_count >= 50 {
            println!("\n=== Демонстрация завершена (50 обновлений) ===");
            break;
        }
    }
}

fn check_arbitrage(prices: &HashMap<String, HashMap<String, PriceUpdate>>) {
    let symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    for symbol in symbols {
        let mut best_bid: Option<(&str, f64)> = None;
        let mut best_ask: Option<(&str, f64)> = None;

        for (exchange, symbol_prices) in prices {
            if let Some(price) = symbol_prices.get(symbol) {
                // Лучший bid — самый высокий (можем продать)
                if best_bid.is_none() || price.bid > best_bid.unwrap().1 {
                    best_bid = Some((exchange.as_str(), price.bid));
                }
                // Лучший ask — самый низкий (можем купить)
                if best_ask.is_none() || price.ask < best_ask.unwrap().1 {
                    best_ask = Some((exchange.as_str(), price.ask));
                }
            }
        }

        if let (Some((sell_ex, sell_price)), Some((buy_ex, buy_price))) = (best_bid, best_ask) {
            if sell_price > buy_price && sell_ex != buy_ex {
                let spread_percent = (sell_price - buy_price) / buy_price * 100.0;
                if spread_percent > 0.05 {
                    println!(
                        "\n!!! АРБИТРАЖ {} !!!\n    Купить на {} по {:.2}\n    Продать на {} по {:.2}\n    Спред: {:.3}%\n",
                        symbol, buy_ex, buy_price, sell_ex, sell_price, spread_percent
                    );
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Параллельные WebSocket подписки ===\n");

    // Канал для передачи обновлений цен
    let (tx, rx) = mpsc::channel::<PriceUpdate>(100);

    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    // Запускаем подписки на все биржи параллельно
    let binance = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Binance", symbols, tx).await;
        })
    };

    let kraken = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Kraken", symbols, tx).await;
        })
    };

    let coinbase = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Coinbase", symbols, tx).await;
        })
    };

    // Важно: закрываем оригинальный sender, чтобы receiver мог завершиться
    drop(tx);

    // Запускаем агрегатор
    let aggregator = tokio::spawn(async move {
        price_aggregator(rx).await;
    });

    // Ждём завершения агрегатора (он остановится после 50 обновлений)
    let _ = aggregator.await;

    // Отменяем подписки
    binance.abort();
    kraken.abort();
    coinbase.abort();

    println!("\nВсе подписки закрыты");
}
```

## Использование tokio::select! для обработки событий

`select!` позволяет ожидать первое готовое событие из нескольких:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, timeout};

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    RiskAlert { message: String },
    Heartbeat,
}

async fn event_processor(
    mut price_rx: mpsc::Receiver<TradingEvent>,
    mut order_rx: mpsc::Receiver<TradingEvent>,
    mut risk_rx: mpsc::Receiver<TradingEvent>,
) {
    let heartbeat_interval = Duration::from_secs(5);
    let mut heartbeat = tokio::time::interval(heartbeat_interval);

    loop {
        tokio::select! {
            // Приоритет 1: Риск-алерты (всегда обрабатываем первыми)
            Some(event) = risk_rx.recv() => {
                match event {
                    TradingEvent::RiskAlert { message } => {
                        println!("!!! РИСК-АЛЕРТ: {} !!!", message);
                        // В реальности здесь могла бы быть экстренная остановка торговли
                    }
                    _ => {}
                }
            }

            // Приоритет 2: Исполнение ордеров
            Some(event) = order_rx.recv() => {
                match event {
                    TradingEvent::OrderFilled { order_id, price } => {
                        println!("Ордер #{} исполнен по цене {:.2}", order_id, price);
                    }
                    _ => {}
                }
            }

            // Приоритет 3: Обновления цен
            Some(event) = price_rx.recv() => {
                match event {
                    TradingEvent::PriceUpdate { symbol, price } => {
                        println!("Цена {}: {:.2}", symbol, price);
                    }
                    _ => {}
                }
            }

            // Heartbeat для проверки соединения
            _ = heartbeat.tick() => {
                println!("[Heartbeat] Система работает");
            }

            // Все каналы закрыты
            else => {
                println!("Все источники событий закрыты");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (price_tx, price_rx) = mpsc::channel(100);
    let (order_tx, order_rx) = mpsc::channel(100);
    let (risk_tx, risk_rx) = mpsc::channel(100);

    // Запускаем обработчик событий
    let processor = tokio::spawn(async move {
        event_processor(price_rx, order_rx, risk_rx).await;
    });

    // Симулируем события
    let price_tx_clone = price_tx.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            sleep(Duration::from_millis(200)).await;
            let _ = price_tx_clone.send(TradingEvent::PriceUpdate {
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            }).await;
        }
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        let _ = order_tx.send(TradingEvent::OrderFilled {
            order_id: 12345,
            price: 42050.0,
        }).await;
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(800)).await;
        let _ = risk_tx.send(TradingEvent::RiskAlert {
            message: "Превышен дневной лимит убытков".to_string(),
        }).await;
    });

    // Ждём немного и закрываем
    sleep(Duration::from_secs(2)).await;
    drop(price_tx);

    let _ = processor.await;
}
```

## Продвинутый пример: Мультибиржевой торговый бот

```rust
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct MarketData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    bid_volume: f64,
    ask_volume: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    exchange: String,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
enum Command {
    PlaceOrder(Order),
    CancelOrder(u64),
    Shutdown,
}

struct TradingBot {
    // Текущие рыночные данные со всех бирж
    market_data: Arc<RwLock<HashMap<String, HashMap<String, MarketData>>>>,
    // Активные ордера
    orders: Arc<RwLock<HashMap<u64, Order>>>,
    // Счётчик ордеров
    order_counter: Arc<RwLock<u64>>,
}

impl TradingBot {
    fn new() -> Self {
        TradingBot {
            market_data: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            order_counter: Arc::new(RwLock::new(0)),
        }
    }

    // Обработчик рыночных данных
    async fn market_data_handler(
        &self,
        mut rx: mpsc::Receiver<MarketData>,
        strategy_tx: broadcast::Sender<MarketData>,
    ) {
        while let Some(data) = rx.recv().await {
            // Сохраняем данные
            {
                let mut market = self.market_data.write().await;
                market
                    .entry(data.exchange.clone())
                    .or_insert_with(HashMap::new)
                    .insert(data.symbol.clone(), data.clone());
            }

            // Уведомляем стратегию
            let _ = strategy_tx.send(data);
        }
    }

    // Простая арбитражная стратегия
    async fn arbitrage_strategy(
        &self,
        mut data_rx: broadcast::Receiver<MarketData>,
        command_tx: mpsc::Sender<Command>,
    ) {
        let min_spread_percent = 0.1; // Минимальный спред для арбитража

        while let Ok(data) = data_rx.recv().await {
            let market = self.market_data.read().await;

            // Ищем арбитражные возможности
            for (other_exchange, symbols) in market.iter() {
                if *other_exchange == data.exchange {
                    continue;
                }

                if let Some(other_data) = symbols.get(&data.symbol) {
                    // Проверяем: можем купить на одной бирже и продать на другой?

                    // Вариант 1: Купить на data.exchange, продать на other_exchange
                    let spread1 = (other_data.bid - data.ask) / data.ask * 100.0;
                    if spread1 > min_spread_percent {
                        println!(
                            "\n>>> Арбитраж найден! {} <<<",
                            data.symbol
                        );
                        println!(
                            "    Купить на {} по {:.2}",
                            data.exchange, data.ask
                        );
                        println!(
                            "    Продать на {} по {:.2}",
                            other_exchange, other_data.bid
                        );
                        println!("    Прибыль: {:.3}%\n", spread1);

                        // Размещаем ордера (в реальности нужна атомарность)
                        let quantity = 0.01; // Минимальный объём

                        let mut counter = self.order_counter.write().await;
                        *counter += 1;
                        let buy_order_id = *counter;
                        *counter += 1;
                        let sell_order_id = *counter;

                        // Ордер на покупку
                        let _ = command_tx.send(Command::PlaceOrder(Order {
                            id: buy_order_id,
                            exchange: data.exchange.clone(),
                            symbol: data.symbol.clone(),
                            side: OrderSide::Buy,
                            price: data.ask,
                            quantity,
                            status: OrderStatus::Pending,
                        })).await;

                        // Ордер на продажу
                        let _ = command_tx.send(Command::PlaceOrder(Order {
                            id: sell_order_id,
                            exchange: other_exchange.clone(),
                            symbol: data.symbol.clone(),
                            side: OrderSide::Sell,
                            price: other_data.bid,
                            quantity,
                            status: OrderStatus::Pending,
                        })).await;
                    }
                }
            }
        }
    }

    // Обработчик команд (исполнение ордеров)
    async fn order_handler(&self, mut rx: mpsc::Receiver<Command>) {
        while let Some(command) = rx.recv().await {
            match command {
                Command::PlaceOrder(order) => {
                    println!(
                        "Размещаем ордер #{}: {:?} {} {} по {:.2}",
                        order.id, order.side, order.quantity, order.symbol, order.price
                    );

                    // Симулируем отправку на биржу
                    let mut orders = self.orders.write().await;
                    orders.insert(order.id, order.clone());

                    // В реальности здесь был бы REST API запрос к бирже
                    sleep(Duration::from_millis(10)).await;

                    // Симулируем исполнение
                    if let Some(o) = orders.get_mut(&order.id) {
                        o.status = OrderStatus::Filled;
                        println!("Ордер #{} исполнен!", order.id);
                    }
                }
                Command::CancelOrder(order_id) => {
                    let mut orders = self.orders.write().await;
                    if let Some(order) = orders.get_mut(&order_id) {
                        order.status = OrderStatus::Cancelled;
                        println!("Ордер #{} отменён", order_id);
                    }
                }
                Command::Shutdown => {
                    println!("Получена команда завершения");
                    break;
                }
            }
        }
    }
}

// Симулятор биржевого WebSocket
async fn exchange_simulator(
    exchange: &str,
    symbols: Vec<&str>,
    tx: mpsc::Sender<MarketData>,
) {
    let mut tick_interval = interval(Duration::from_millis(100));
    let mut counter = 0u64;

    println!("[{}] Симулятор запущен", exchange);

    loop {
        tick_interval.tick().await;
        counter += 1;

        for symbol in &symbols {
            let base_price = match *symbol {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2200.0,
                _ => 100.0,
            };

            // Разные цены на разных биржах
            let exchange_modifier = match exchange {
                "Binance" => 1.0,
                "Kraken" => 1.002,  // На 0.2% дороже
                "Coinbase" => 0.998, // На 0.2% дешевле
                _ => 1.0,
            };

            // Добавляем шум
            let noise = ((counter as f64).sin() * 10.0) +
                       ((counter as f64 * 0.7).cos() * 5.0);

            let mid_price = base_price * exchange_modifier + noise;
            let spread = mid_price * 0.0005; // 0.05% спред

            let data = MarketData {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                bid: mid_price - spread,
                ask: mid_price + spread,
                bid_volume: 1.0 + (counter as f64 % 5.0),
                ask_volume: 1.0 + ((counter + 2) as f64 % 5.0),
                timestamp: counter,
            };

            if tx.send(data).await.is_err() {
                println!("[{}] Канал закрыт, завершаем", exchange);
                return;
            }
        }

        // Для демонстрации ограничиваем количество обновлений
        if counter >= 30 {
            println!("[{}] Симуляция завершена", exchange);
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Мультибиржевой торговый бот ===\n");

    let bot = Arc::new(TradingBot::new());

    // Каналы
    let (market_tx, market_rx) = mpsc::channel::<MarketData>(1000);
    let (strategy_tx, _) = broadcast::channel::<MarketData>(1000);
    let strategy_rx = strategy_tx.subscribe();
    let (command_tx, command_rx) = mpsc::channel::<Command>(100);

    let symbols = vec!["BTC/USDT", "ETH/USDT"];

    // Запускаем обработчик рыночных данных
    let bot_clone = Arc::clone(&bot);
    let strategy_tx_clone = strategy_tx.clone();
    let market_handler = tokio::spawn(async move {
        bot_clone.market_data_handler(market_rx, strategy_tx_clone).await;
    });

    // Запускаем стратегию
    let bot_clone = Arc::clone(&bot);
    let strategy = tokio::spawn(async move {
        bot_clone.arbitrage_strategy(strategy_rx, command_tx).await;
    });

    // Запускаем обработчик ордеров
    let bot_clone = Arc::clone(&bot);
    let order_handler = tokio::spawn(async move {
        bot_clone.order_handler(command_rx).await;
    });

    // Запускаем симуляторы бирж параллельно
    let exchanges = vec![
        ("Binance", symbols.clone()),
        ("Kraken", symbols.clone()),
        ("Coinbase", symbols.clone()),
    ];

    let mut exchange_handles = vec![];
    for (exchange, syms) in exchanges {
        let tx = market_tx.clone();
        let handle = tokio::spawn(async move {
            exchange_simulator(exchange, syms, tx).await;
        });
        exchange_handles.push(handle);
    }

    // Закрываем оригинальный sender
    drop(market_tx);

    // Ждём завершения симуляторов
    for handle in exchange_handles {
        let _ = handle.await;
    }

    // Даём время на обработку оставшихся данных
    sleep(Duration::from_millis(500)).await;

    // Выводим статистику
    let orders = bot.orders.read().await;
    println!("\n=== Статистика ===");
    println!("Всего ордеров: {}", orders.len());
    println!(
        "Исполнено: {}",
        orders.values().filter(|o| o.status == OrderStatus::Filled).count()
    );

    println!("\nБот завершил работу");
}
```

## Обработка переподключений и ошибок

В реальных условиях WebSocket соединения могут разрываться. Важно уметь их восстанавливать:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug)]
struct ConnectionState {
    is_connected: AtomicBool,
    reconnect_count: AtomicU32,
    max_reconnects: u32,
}

impl ConnectionState {
    fn new(max_reconnects: u32) -> Self {
        ConnectionState {
            is_connected: AtomicBool::new(false),
            reconnect_count: AtomicU32::new(0),
            max_reconnects,
        }
    }

    fn set_connected(&self, connected: bool) {
        self.is_connected.store(connected, Ordering::SeqCst);
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    fn increment_reconnect(&self) -> bool {
        let count = self.reconnect_count.fetch_add(1, Ordering::SeqCst);
        count < self.max_reconnects
    }

    fn reset_reconnect_count(&self) {
        self.reconnect_count.store(0, Ordering::SeqCst);
    }
}

async fn resilient_websocket_connection(
    exchange: &str,
    state: Arc<ConnectionState>,
    tx: mpsc::Sender<String>,
) {
    loop {
        println!("[{}] Попытка подключения...", exchange);

        // Симулируем подключение
        match simulate_connect(exchange).await {
            Ok(_) => {
                println!("[{}] Подключено!", exchange);
                state.set_connected(true);
                state.reset_reconnect_count();

                // Симулируем получение данных
                if let Err(e) = simulate_receive_data(exchange, &tx).await {
                    println!("[{}] Ошибка получения данных: {}", exchange, e);
                    state.set_connected(false);
                }
            }
            Err(e) => {
                println!("[{}] Ошибка подключения: {}", exchange, e);
            }
        }

        // Пробуем переподключиться
        if !state.increment_reconnect() {
            println!("[{}] Превышено максимальное количество переподключений", exchange);
            break;
        }

        let reconnect_delay = Duration::from_secs(
            2u64.pow(state.reconnect_count.load(Ordering::SeqCst).min(5))
        );
        println!(
            "[{}] Переподключение через {:?}...",
            exchange, reconnect_delay
        );
        sleep(reconnect_delay).await;
    }
}

async fn simulate_connect(exchange: &str) -> Result<(), String> {
    sleep(Duration::from_millis(100)).await;

    // Симулируем случайные ошибки подключения
    if exchange == "Kraken" && rand_bool(0.3) {
        return Err("Connection refused".to_string());
    }

    Ok(())
}

async fn simulate_receive_data(
    exchange: &str,
    tx: &mpsc::Sender<String>,
) -> Result<(), String> {
    for i in 0..10 {
        sleep(Duration::from_millis(200)).await;

        // Симулируем случайный разрыв соединения
        if rand_bool(0.1) {
            return Err("Connection reset by peer".to_string());
        }

        let msg = format!("[{}] Tick {}: BTC = {:.2}", exchange, i, 42000.0 + i as f64 * 10.0);
        if tx.send(msg).await.is_err() {
            return Err("Channel closed".to_string());
        }
    }

    Ok(())
}

// Простой генератор случайных чисел для демонстрации
fn rand_bool(probability: f64) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f64 / u32::MAX as f64) < probability
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    let state = Arc::new(ConnectionState::new(5));

    let handle = {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            resilient_websocket_connection("Binance", state, tx).await;
        })
    };

    // Получаем сообщения
    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }

    let _ = handle.await;
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio::spawn` | Запуск асинхронной задачи в фоне |
| `mpsc::channel` | Канал для передачи данных между задачами |
| `broadcast::channel` | Канал с множеством получателей |
| `tokio::select!` | Ожидание первого готового события |
| `RwLock` | Блокировка для чтения/записи в async |
| Экспоненциальный backoff | Увеличивающаяся задержка при переподключении |

## Домашнее задание

1. **Мониторинг спреда**: Создай программу, которая подключается к нескольким "биржам" и в реальном времени отслеживает спред (разницу между bid и ask) для каждой пары. Выводи предупреждение, если спред превышает 0.5%.

2. **Детектор аномалий**: Реализуй систему, которая:
   - Получает цены с нескольких бирж
   - Вычисляет среднюю цену
   - Определяет, если цена на какой-то бирже отклоняется более чем на 1% от средней
   - Логирует все аномалии

3. **Синхронизатор стаканов**: Напиши программу, которая:
   - Получает данные стакана (bid/ask с объёмами) с трёх бирж
   - Объединяет их в единый агрегированный стакан
   - Показывает топ-5 лучших bid и ask со всех бирж

4. **Система оповещений**: Создай систему с приоритетными уровнями оповещений:
   - Критические (price spike > 5%) — немедленная остановка торговли
   - Важные (spread > 1%) — предупреждение
   - Информационные (обычные обновления цен) — логирование

   Используй `tokio::select!` с приоритетной обработкой.

## Навигация

[← Предыдущий день](../208-processing-websocket-messages/ru.md) | [Следующий день →](../210-graceful-shutdown-async-tasks/ru.md)
