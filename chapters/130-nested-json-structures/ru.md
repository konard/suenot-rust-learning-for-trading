# День 130: Вложенные структуры JSON

## Аналогия из трейдинга

Представь ответ биржевого API как отчёт о твоём портфеле. В нём есть:
- Общая информация (баланс, время обновления)
- Список позиций, где каждая позиция — это отдельный объект
- В каждой позиции — вложенные данные об активе, прибыли, марже

Это как **матрёшка**: внутри большого объекта находятся объекты поменьше, а в них — ещё данные. В JSON такие структуры встречаются постоянно, и Rust с serde отлично с ними справляется.

## Что такое вложенные структуры?

Вложенная структура — это когда одна структура содержит поле, которое само является структурой:

```rust
use serde::{Deserialize, Serialize};

// Вложенная структура — актив
#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    symbol: String,
    name: String,
}

// Основная структура содержит вложенную
#[derive(Debug, Serialize, Deserialize)]
struct Position {
    asset: Asset,  // Вложенная структура
    quantity: f64,
    entry_price: f64,
}

fn main() {
    let json = r#"
    {
        "asset": {
            "symbol": "BTC",
            "name": "Bitcoin"
        },
        "quantity": 0.5,
        "entry_price": 42000.0
    }
    "#;

    let position: Position = serde_json::from_str(json).unwrap();

    println!("Asset: {} ({})", position.asset.name, position.asset.symbol);
    println!("Quantity: {}", position.quantity);
    println!("Entry: ${}", position.entry_price);
}
```

## Многоуровневая вложенность

API бирж часто возвращают глубоко вложенные структуры:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Exchange {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Market {
    symbol: String,
    base: String,
    quote: String,
    exchange: Exchange,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticker {
    market: Market,
    last_price: f64,
    volume_24h: f64,
    change_24h: f64,
}

fn main() {
    let json = r#"
    {
        "market": {
            "symbol": "BTC/USDT",
            "base": "BTC",
            "quote": "USDT",
            "exchange": {
                "name": "Binance",
                "url": "https://binance.com"
            }
        },
        "last_price": 43250.50,
        "volume_24h": 125000000.0,
        "change_24h": 2.35
    }
    "#;

    let ticker: Ticker = serde_json::from_str(json).unwrap();

    println!("=== Ticker Info ===");
    println!("Exchange: {}", ticker.market.exchange.name);
    println!("Market: {}", ticker.market.symbol);
    println!("Price: ${:.2}", ticker.last_price);
    println!("24h Volume: ${:.0}", ticker.volume_24h);
    println!("24h Change: {:.2}%", ticker.change_24h);
}
```

## Массивы вложенных структур

Очень часто API возвращает массив объектов:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    id: u64,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeHistory {
    symbol: String,
    trades: Vec<Trade>,  // Массив вложенных структур
}

fn main() {
    let json = r#"
    {
        "symbol": "ETH/USDT",
        "trades": [
            {"id": 1001, "price": 2250.0, "quantity": 1.5, "side": "buy", "timestamp": 1700000001},
            {"id": 1002, "price": 2251.5, "quantity": 0.8, "side": "sell", "timestamp": 1700000002},
            {"id": 1003, "price": 2249.0, "quantity": 2.0, "side": "buy", "timestamp": 1700000003}
        ]
    }
    "#;

    let history: TradeHistory = serde_json::from_str(json).unwrap();

    println!("=== Trade History: {} ===", history.symbol);
    println!("{:<6} {:>10} {:>10} {:>6}", "ID", "Price", "Qty", "Side");
    println!("{}", "-".repeat(36));

    for trade in &history.trades {
        println!("{:<6} {:>10.2} {:>10.2} {:>6}",
            trade.id, trade.price, trade.quantity, trade.side);
    }

    // Анализ
    let total_volume: f64 = history.trades.iter().map(|t| t.quantity).sum();
    let avg_price: f64 = history.trades.iter().map(|t| t.price).sum::<f64>()
                         / history.trades.len() as f64;

    println!("{}", "-".repeat(36));
    println!("Total Volume: {:.2}", total_volume);
    println!("Average Price: ${:.2}", avg_price);
}
```

## Практический пример: Стакан заявок

Стакан заявок (Order Book) — классический пример вложенных структур:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct PriceLevel {
    price: f64,
    quantity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderBook {
    symbol: String,
    timestamp: u64,
    bids: Vec<PriceLevel>,  // Заявки на покупку
    asks: Vec<PriceLevel>,  // Заявки на продажу
}

fn main() {
    let json = r#"
    {
        "symbol": "BTC/USDT",
        "timestamp": 1700000000,
        "bids": [
            {"price": 42000.0, "quantity": 1.5},
            {"price": 41995.0, "quantity": 2.3},
            {"price": 41990.0, "quantity": 0.8}
        ],
        "asks": [
            {"price": 42005.0, "quantity": 1.2},
            {"price": 42010.0, "quantity": 3.1},
            {"price": 42015.0, "quantity": 0.5}
        ]
    }
    "#;

    let order_book: OrderBook = serde_json::from_str(json).unwrap();

    println!("╔═══════════════════════════════════╗");
    println!("║   ORDER BOOK: {}          ║", order_book.symbol);
    println!("╠═══════════════════════════════════╣");
    println!("║          ASKS (Sell)              ║");
    println!("╠═══════════════════════════════════╣");

    // Asks в обратном порядке (сверху — дальние)
    for ask in order_book.asks.iter().rev() {
        println!("║  ${:>10.2}  |  {:>8.4} BTC   ║", ask.price, ask.quantity);
    }

    // Спред
    let best_bid = &order_book.bids[0];
    let best_ask = &order_book.asks[0];
    let spread = best_ask.price - best_bid.price;

    println!("╠═══════════════════════════════════╣");
    println!("║     SPREAD: ${:.2}                ║", spread);
    println!("╠═══════════════════════════════════╣");
    println!("║          BIDS (Buy)               ║");
    println!("╠═══════════════════════════════════╣");

    for bid in &order_book.bids {
        println!("║  ${:>10.2}  |  {:>8.4} BTC   ║", bid.price, bid.quantity);
    }

    println!("╚═══════════════════════════════════╝");

    // Расчёт глубины рынка
    let bid_depth: f64 = order_book.bids.iter().map(|b| b.quantity).sum();
    let ask_depth: f64 = order_book.asks.iter().map(|a| a.quantity).sum();

    println!("\nMarket Depth:");
    println!("  Bid Depth: {:.4} BTC", bid_depth);
    println!("  Ask Depth: {:.4} BTC", ask_depth);
    println!("  Imbalance: {:.1}% bids", (bid_depth / (bid_depth + ask_depth)) * 100.0);
}
```

## Практический пример: Ответ API портфеля

Реальный ответ биржевого API обычно содержит множество вложенных уровней:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AssetBalance {
    asset: String,
    free: f64,
    locked: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenPosition {
    symbol: String,
    side: String,
    entry_price: f64,
    quantity: f64,
    unrealized_pnl: f64,
    leverage: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountInfo {
    total_equity: f64,
    available_margin: f64,
    used_margin: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PortfolioResponse {
    success: bool,
    timestamp: u64,
    account: AccountInfo,
    balances: Vec<AssetBalance>,
    positions: Vec<OpenPosition>,
}

fn main() {
    let json = r#"
    {
        "success": true,
        "timestamp": 1700000000,
        "account": {
            "total_equity": 50000.0,
            "available_margin": 35000.0,
            "used_margin": 15000.0
        },
        "balances": [
            {"asset": "USDT", "free": 25000.0, "locked": 5000.0},
            {"asset": "BTC", "free": 0.5, "locked": 0.1},
            {"asset": "ETH", "free": 5.0, "locked": 0.0}
        ],
        "positions": [
            {
                "symbol": "BTC/USDT",
                "side": "long",
                "entry_price": 42000.0,
                "quantity": 0.3,
                "unrealized_pnl": 375.0,
                "leverage": 10
            },
            {
                "symbol": "ETH/USDT",
                "side": "short",
                "entry_price": 2300.0,
                "quantity": 2.0,
                "unrealized_pnl": -50.0,
                "leverage": 5
            }
        ]
    }
    "#;

    let portfolio: PortfolioResponse = serde_json::from_str(json).unwrap();

    if !portfolio.success {
        println!("Error: Failed to fetch portfolio");
        return;
    }

    println!("╔════════════════════════════════════════════╗");
    println!("║           PORTFOLIO SUMMARY                ║");
    println!("╠════════════════════════════════════════════╣");
    println!("║ Total Equity:     ${:>20.2} ║", portfolio.account.total_equity);
    println!("║ Available Margin: ${:>20.2} ║", portfolio.account.available_margin);
    println!("║ Used Margin:      ${:>20.2} ║", portfolio.account.used_margin);
    println!("╠════════════════════════════════════════════╣");
    println!("║                BALANCES                    ║");
    println!("╠════════════════════════════════════════════╣");

    for balance in &portfolio.balances {
        let total = balance.free + balance.locked;
        println!("║ {:>6}: {:>12.4} (free: {:>8.4})     ║",
            balance.asset, total, balance.free);
    }

    println!("╠════════════════════════════════════════════╣");
    println!("║              OPEN POSITIONS                ║");
    println!("╠════════════════════════════════════════════╣");

    let mut total_pnl = 0.0;
    for pos in &portfolio.positions {
        total_pnl += pos.unrealized_pnl;
        let pnl_sign = if pos.unrealized_pnl >= 0.0 { "+" } else { "" };
        println!("║ {} {} x{:<2}                              ║",
            pos.symbol, pos.side.to_uppercase(), pos.leverage);
        println!("║   Entry: ${:.2}, Qty: {:.4}              ║",
            pos.entry_price, pos.quantity);
        println!("║   PnL: {}${:.2}                          ║",
            pnl_sign, pos.unrealized_pnl);
    }

    println!("╠════════════════════════════════════════════╣");
    let total_sign = if total_pnl >= 0.0 { "+" } else { "" };
    println!("║ Total Unrealized PnL: {}${:.2}             ║", total_sign, total_pnl);
    println!("╚════════════════════════════════════════════╝");
}
```

## Сериализация вложенных структур

Создание JSON из вложенных структур работает симметрично:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderRequest {
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiRequest {
    action: String,
    timestamp: u64,
    order: OrderRequest,
}

fn main() {
    // Создаём вложенную структуру
    let request = ApiRequest {
        action: "create_order".to_string(),
        timestamp: 1700000000,
        order: OrderRequest {
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            quantity: 0.1,
            price: Some(42000.0),
        },
    };

    // Сериализуем в JSON
    let json = serde_json::to_string_pretty(&request).unwrap();
    println!("Request JSON:\n{}", json);

    // Парсим обратно
    let parsed: ApiRequest = serde_json::from_str(&json).unwrap();
    println!("\nParsed back:");
    println!("Action: {}", parsed.action);
    println!("Order: {} {} {:.4} @ ${:.2}",
        parsed.order.side,
        parsed.order.symbol,
        parsed.order.quantity,
        parsed.order.price.unwrap_or(0.0));
}
```

## Работа с serde_json::Value для динамических структур

Иногда структура JSON неизвестна заранее:

```rust
use serde_json::{Value, json};

fn main() {
    // Неизвестная структура — используем Value
    let json_str = r#"
    {
        "exchange": "binance",
        "data": {
            "ticker": {
                "symbol": "BTC/USDT",
                "prices": {
                    "bid": 42000.0,
                    "ask": 42005.0,
                    "last": 42002.5
                }
            }
        }
    }
    "#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Навигация по вложенной структуре
    let exchange = value["exchange"].as_str().unwrap_or("unknown");
    let symbol = value["data"]["ticker"]["symbol"].as_str().unwrap_or("unknown");
    let bid = value["data"]["ticker"]["prices"]["bid"].as_f64().unwrap_or(0.0);
    let ask = value["data"]["ticker"]["prices"]["ask"].as_f64().unwrap_or(0.0);

    println!("Exchange: {}", exchange);
    println!("Symbol: {}", symbol);
    println!("Bid: ${:.2}, Ask: ${:.2}", bid, ask);
    println!("Spread: ${:.2}", ask - bid);

    // Создаём динамический JSON
    let response = json!({
        "status": "ok",
        "data": {
            "spread": ask - bid,
            "mid_price": (bid + ask) / 2.0
        }
    });

    println!("\nGenerated response:");
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
```

## Практические упражнения

### Упражнение 1: Парсинг свечей

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CandleResponse {
    symbol: String,
    timeframe: String,
    candles: Vec<Candle>,
}

fn main() {
    let json = r#"
    {
        "symbol": "BTC/USDT",
        "timeframe": "1h",
        "candles": [
            {"timestamp": 1700000000, "open": 42000.0, "high": 42500.0, "low": 41800.0, "close": 42300.0, "volume": 1500.5},
            {"timestamp": 1700003600, "open": 42300.0, "high": 42800.0, "low": 42100.0, "close": 42600.0, "volume": 1200.3},
            {"timestamp": 1700007200, "open": 42600.0, "high": 42700.0, "low": 42000.0, "close": 42100.0, "volume": 1800.7}
        ]
    }
    "#;

    let response: CandleResponse = serde_json::from_str(json).unwrap();

    println!("=== {} ({}) ===", response.symbol, response.timeframe);

    for candle in &response.candles {
        let is_bullish = candle.close > candle.open;
        let emoji = if is_bullish { "🟢" } else { "🔴" };
        println!("{} O:{:.0} H:{:.0} L:{:.0} C:{:.0} V:{:.1}",
            emoji, candle.open, candle.high, candle.low, candle.close, candle.volume);
    }
}
```

### Упражнение 2: Многоуровневая позиция

Создай структуры для парсинга:

```json
{
    "position": {
        "instrument": {
            "symbol": "BTC/USDT",
            "type": "perpetual",
            "contract_size": 1.0
        },
        "details": {
            "side": "long",
            "size": 0.5,
            "entry_price": 42000.0,
            "mark_price": 42500.0
        },
        "risk": {
            "leverage": 10,
            "liquidation_price": 38000.0,
            "margin_ratio": 0.15
        }
    }
}
```

### Упражнение 3: Создание вложенного JSON

Напиши функцию, которая создаёт структуру ордера со всеми вложенными объектами и сериализует в JSON.

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Вложенные структуры | Структура содержит поле-структуру |
| `Vec<T>` в JSON | Массив вложенных объектов |
| Многоуровневая вложенность | Доступ через `obj.field.subfield` |
| `serde_json::Value` | Динамический парсинг неизвестных структур |
| `json!{}` макрос | Создание динамического JSON |

## Домашнее задание

1. Создай структуры для парсинга ответа с информацией о нескольких биржах, где каждая биржа содержит список рынков

2. Напиши функцию, которая принимает JSON стакана заявок и возвращает структуру с расчётами: спред, глубина bid/ask, дисбаланс

3. Реализуй структуры для API ответа с историей сделок пользователя, включая комиссии и общую статистику

4. Создай функцию, которая конвертирует между двумя форматами JSON (например, разные биржи возвращают данные в разном формате)

## Навигация

[← Предыдущий день](../129-serde-json-parsing/ru.md) | [Следующий день →](../131-optional-json-fields/ru.md)
