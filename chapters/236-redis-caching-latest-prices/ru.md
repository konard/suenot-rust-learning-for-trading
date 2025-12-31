# День 236: Redis: кэширование актуальных цен

## Аналогия из трейдинга

Представь себя трейдером на бирже. Каждую секунду ты получаешь тысячи обновлений цен по разным инструментам — акциям, криптовалютам, фьючерсам. Если каждый раз обращаться напрямую к бирже за последней ценой, ты получишь:
- Огромные задержки (latency)
- Перегрузку API биржи
- Возможные блокировки за превышение лимитов запросов

Вместо этого опытные трейдеры используют **локальный кэш цен** — быстрое хранилище, где постоянно обновляются актуальные котировки. Redis идеально подходит для этой задачи: он хранит данные в памяти и обеспечивает время доступа в микросекундах.

Это как иметь экран с котировками прямо перед глазами вместо того, чтобы каждый раз звонить брокеру и спрашивать текущую цену.

## Почему Redis для кэширования цен?

| Характеристика | Значение для трейдинга |
|----------------|------------------------|
| In-memory хранение | Микросекундный доступ к ценам |
| TTL (Time To Live) | Автоматическое устаревание старых цен |
| Атомарные операции | Безопасное обновление из нескольких источников |
| Pub/Sub | Уведомления об изменении цен (будет в следующей главе) |
| Структуры данных | Hash для хранения нескольких полей цены |

## Базовая структура кэша цен

```rust
use redis::{Client, Commands, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PriceData {
    symbol: String,
    bid: f64,           // Лучшая цена покупки
    ask: f64,           // Лучшая цена продажи
    last: f64,          // Последняя сделка
    volume_24h: f64,    // Объём за 24 часа
    timestamp: u64,     // Время обновления (Unix timestamp)
}

impl PriceData {
    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

fn main() -> RedisResult<()> {
    // Подключение к Redis
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // Создаём данные о цене
    let btc_price = PriceData {
        symbol: "BTC/USDT".to_string(),
        bid: 42150.50,
        ask: 42155.00,
        last: 42152.75,
        volume_24h: 15234.56,
        timestamp: 1704067200,
    };

    // Сериализуем в JSON и сохраняем
    let price_json = serde_json::to_string(&btc_price).unwrap();
    let _: () = con.set_ex("price:BTC/USDT", &price_json, 60)?; // TTL 60 секунд

    println!("Цена BTC/USDT сохранена в кэш");
    println!("Спред: {:.2} ({:.4}%)", btc_price.spread(), btc_price.spread_percent());

    // Читаем из кэша
    let cached: String = con.get("price:BTC/USDT")?;
    let restored: PriceData = serde_json::from_str(&cached).unwrap();

    println!("Из кэша: {:?}", restored);

    Ok(())
}
```

## Использование Hash для хранения цен

Hash в Redis позволяет хранить и обновлять отдельные поля цены без перезаписи всего объекта:

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;

struct PriceCache {
    con: redis::Connection,
}

impl PriceCache {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceCache { con })
    }

    // Обновление всех полей цены
    fn update_price(&mut self, symbol: &str, bid: f64, ask: f64, last: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Используем HSET для установки нескольких полей
        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(bid)
            .arg("ask").arg(ask)
            .arg("last").arg(last)
            .arg("timestamp").arg(timestamp)
            .query(&mut self.con)?;

        // Устанавливаем TTL для ключа
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(60) // 60 секунд
            .query(&mut self.con)?;

        Ok(())
    }

    // Обновление только bid/ask (для стакана заявок)
    fn update_quote(&mut self, symbol: &str, bid: f64, ask: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);

        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(bid)
            .arg("ask").arg(ask)
            .query(&mut self.con)?;

        Ok(())
    }

    // Обновление только последней цены (для сделок)
    fn update_last(&mut self, symbol: &str, last: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);

        redis::cmd("HSET")
            .arg(&key)
            .arg("last").arg(last)
            .query(&mut self.con)?;

        Ok(())
    }

    // Получение всех данных о цене
    fn get_price(&mut self, symbol: &str) -> RedisResult<Option<HashMap<String, String>>> {
        let key = format!("price:{}", symbol);
        let result: HashMap<String, String> = self.con.hgetall(&key)?;

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    // Получение конкретного поля
    fn get_last_price(&mut self, symbol: &str) -> RedisResult<Option<f64>> {
        let key = format!("price:{}", symbol);
        let result: Option<String> = self.con.hget(&key, "last")?;
        Ok(result.map(|s| s.parse().unwrap_or(0.0)))
    }

    // Получение bid/ask для расчёта спреда
    fn get_spread(&mut self, symbol: &str) -> RedisResult<Option<(f64, f64)>> {
        let key = format!("price:{}", symbol);
        let values: Vec<Option<String>> = redis::cmd("HMGET")
            .arg(&key)
            .arg("bid")
            .arg("ask")
            .query(&mut self.con)?;

        match (values.get(0), values.get(1)) {
            (Some(Some(bid)), Some(Some(ask))) => {
                let bid: f64 = bid.parse().unwrap_or(0.0);
                let ask: f64 = ask.parse().unwrap_or(0.0);
                Ok(Some((bid, ask)))
            }
            _ => Ok(None),
        }
    }
}

fn main() -> RedisResult<()> {
    let mut cache = PriceCache::new("redis://127.0.0.1/")?;

    // Обновляем цены
    cache.update_price("BTC/USDT", 42150.50, 42155.00, 42152.75)?;
    cache.update_price("ETH/USDT", 2250.25, 2251.00, 2250.50)?;

    // Получаем данные
    if let Some(btc_data) = cache.get_price("BTC/USDT")? {
        println!("BTC/USDT данные: {:?}", btc_data);
    }

    if let Some(last) = cache.get_last_price("ETH/USDT")? {
        println!("ETH/USDT последняя цена: {}", last);
    }

    if let Some((bid, ask)) = cache.get_spread("BTC/USDT")? {
        println!("BTC/USDT спред: {} - {} = {}", ask, bid, ask - bid);
    }

    Ok(())
}
```

## Кэширование цен нескольких инструментов

```rust
use redis::{Client, Commands, RedisResult, Pipeline};
use std::collections::HashMap;

struct MultiPriceCache {
    con: redis::Connection,
}

impl MultiPriceCache {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(MultiPriceCache { con })
    }

    // Пакетное обновление цен (эффективно для множества инструментов)
    fn batch_update(&mut self, prices: &[(String, f64, f64, f64)]) -> RedisResult<()> {
        let mut pipe = redis::pipe();

        for (symbol, bid, ask, last) in prices {
            let key = format!("price:{}", symbol);

            pipe.cmd("HSET")
                .arg(&key)
                .arg("bid").arg(*bid)
                .arg("ask").arg(*ask)
                .arg("last").arg(*last);

            pipe.cmd("EXPIRE").arg(&key).arg(60);
        }

        pipe.query(&mut self.con)?;
        Ok(())
    }

    // Получение цен для списка инструментов
    fn get_multiple_prices(&mut self, symbols: &[&str]) -> RedisResult<HashMap<String, f64>> {
        let mut result = HashMap::new();

        // Используем pipeline для эффективного получения
        let mut pipe = redis::pipe();

        for symbol in symbols {
            let key = format!("price:{}", symbol);
            pipe.cmd("HGET").arg(&key).arg("last");
        }

        let prices: Vec<Option<String>> = pipe.query(&mut self.con)?;

        for (symbol, price) in symbols.iter().zip(prices.iter()) {
            if let Some(p) = price {
                if let Ok(val) = p.parse::<f64>() {
                    result.insert(symbol.to_string(), val);
                }
            }
        }

        Ok(result)
    }

    // Поиск инструментов по паттерну
    fn find_symbols(&mut self, pattern: &str) -> RedisResult<Vec<String>> {
        let search_pattern = format!("price:{}*", pattern);
        let keys: Vec<String> = self.con.keys(&search_pattern)?;

        Ok(keys.into_iter()
            .map(|k| k.replace("price:", ""))
            .collect())
    }

    // Получение всех кэшированных символов
    fn get_all_symbols(&mut self) -> RedisResult<Vec<String>> {
        self.find_symbols("")
    }
}

fn main() -> RedisResult<()> {
    let mut cache = MultiPriceCache::new("redis://127.0.0.1/")?;

    // Симуляция обновления цен от биржи
    let market_data = vec![
        ("BTC/USDT".to_string(), 42150.50, 42155.00, 42152.75),
        ("ETH/USDT".to_string(), 2250.25, 2251.00, 2250.50),
        ("SOL/USDT".to_string(), 98.50, 98.60, 98.55),
        ("DOGE/USDT".to_string(), 0.0850, 0.0851, 0.0850),
    ];

    cache.batch_update(&market_data)?;
    println!("Обновлено {} инструментов", market_data.len());

    // Получаем цены для портфеля
    let portfolio_symbols = ["BTC/USDT", "ETH/USDT"];
    let prices = cache.get_multiple_prices(&portfolio_symbols)?;

    println!("\nЦены портфеля:");
    for (symbol, price) in &prices {
        println!("  {}: ${:.2}", symbol, price);
    }

    // Поиск всех USDT пар
    let usdt_pairs = cache.find_symbols("*USDT")?;
    println!("\nНайдено USDT пар: {:?}", usdt_pairs);

    Ok(())
}
```

## Практический пример: Торговый монитор с кэшированием

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
    timestamp: u64,
}

struct TradingMonitor {
    con: redis::Connection,
    price_ttl: usize,
}

impl TradingMonitor {
    fn new(redis_url: &str, price_ttl_seconds: usize) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(TradingMonitor {
            con,
            price_ttl: price_ttl_seconds,
        })
    }

    // Обработка тика с биржи
    fn process_tick(&mut self, tick: &MarketTick) -> RedisResult<()> {
        let key = format!("price:{}", tick.symbol);

        // Сохраняем текущую цену
        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(tick.bid)
            .arg("ask").arg(tick.ask)
            .arg("last").arg(tick.last)
            .arg("timestamp").arg(tick.timestamp)
            .query(&mut self.con)?;

        // Устанавливаем TTL
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(self.price_ttl)
            .query(&mut self.con)?;

        // Обновляем историю цен (последние 100 значений)
        let history_key = format!("price_history:{}", tick.symbol);
        let price_entry = format!("{}:{}", tick.timestamp, tick.last);

        let _: () = self.con.lpush(&history_key, &price_entry)?;
        let _: () = self.con.ltrim(&history_key, 0, 99)?; // Храним только 100 записей

        Ok(())
    }

    // Получение текущей цены
    fn get_current_price(&mut self, symbol: &str) -> RedisResult<Option<MarketTick>> {
        let key = format!("price:{}", symbol);
        let data: HashMap<String, String> = self.con.hgetall(&key)?;

        if data.is_empty() {
            return Ok(None);
        }

        Ok(Some(MarketTick {
            symbol: symbol.to_string(),
            bid: data.get("bid").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            ask: data.get("ask").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            last: data.get("last").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            timestamp: data.get("timestamp").and_then(|s| s.parse().ok()).unwrap_or(0),
        }))
    }

    // Проверка свежести цены
    fn is_price_fresh(&mut self, symbol: &str, max_age_seconds: u64) -> RedisResult<bool> {
        let key = format!("price:{}", symbol);
        let timestamp: Option<u64> = self.con.hget(&key, "timestamp")?;

        match timestamp {
            Some(ts) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(now - ts <= max_age_seconds)
            }
            None => Ok(false),
        }
    }

    // Получение истории цен
    fn get_price_history(&mut self, symbol: &str, count: isize) -> RedisResult<Vec<(u64, f64)>> {
        let key = format!("price_history:{}", symbol);
        let entries: Vec<String> = self.con.lrange(&key, 0, count - 1)?;

        let history: Vec<(u64, f64)> = entries
            .iter()
            .filter_map(|e| {
                let parts: Vec<&str> = e.split(':').collect();
                if parts.len() == 2 {
                    let ts = parts[0].parse().ok()?;
                    let price = parts[1].parse().ok()?;
                    Some((ts, price))
                } else {
                    None
                }
            })
            .collect();

        Ok(history)
    }

    // Расчёт средней цены за период
    fn get_average_price(&mut self, symbol: &str, count: isize) -> RedisResult<Option<f64>> {
        let history = self.get_price_history(symbol, count)?;

        if history.is_empty() {
            return Ok(None);
        }

        let sum: f64 = history.iter().map(|(_, p)| p).sum();
        Ok(Some(sum / history.len() as f64))
    }
}

fn main() -> RedisResult<()> {
    let mut monitor = TradingMonitor::new("redis://127.0.0.1/", 60)?;

    // Симуляция потока рыночных данных
    let ticks = vec![
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42150.0,
            ask: 42155.0,
            last: 42152.0,
            timestamp: 1704067200,
        },
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42155.0,
            ask: 42160.0,
            last: 42158.0,
            timestamp: 1704067201,
        },
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42148.0,
            ask: 42153.0,
            last: 42150.0,
            timestamp: 1704067202,
        },
        MarketTick {
            symbol: "ETH/USDT".to_string(),
            bid: 2250.0,
            ask: 2251.0,
            last: 2250.5,
            timestamp: 1704067200,
        },
    ];

    // Обрабатываем тики
    for tick in &ticks {
        monitor.process_tick(tick)?;
        println!("Обработан тик: {} @ {}", tick.symbol, tick.last);
    }

    // Получаем текущие цены
    println!("\n--- Текущие цены ---");
    if let Some(btc) = monitor.get_current_price("BTC/USDT")? {
        println!("BTC/USDT: bid={}, ask={}, last={}", btc.bid, btc.ask, btc.last);
    }

    // Получаем историю
    println!("\n--- История BTC/USDT ---");
    let history = monitor.get_price_history("BTC/USDT", 10)?;
    for (ts, price) in &history {
        println!("  {}: ${:.2}", ts, price);
    }

    // Средняя цена
    if let Some(avg) = monitor.get_average_price("BTC/USDT", 10)? {
        println!("\nСредняя цена BTC/USDT: ${:.2}", avg);
    }

    Ok(())
}
```

## Стратегии инвалидации кэша

```rust
use redis::{Client, Commands, RedisResult};

struct PriceCacheWithInvalidation {
    con: redis::Connection,
}

impl PriceCacheWithInvalidation {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceCacheWithInvalidation { con })
    }

    // Установка цены с TTL
    fn set_price_with_ttl(&mut self, symbol: &str, price: f64, ttl_seconds: usize) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        let _: () = self.con.set_ex(&key, price, ttl_seconds)?;
        Ok(())
    }

    // Условное обновление — только если цена изменилась значительно
    fn update_if_significant_change(
        &mut self,
        symbol: &str,
        new_price: f64,
        threshold_percent: f64
    ) -> RedisResult<bool> {
        let key = format!("price:{}", symbol);

        // Получаем текущую цену
        let current: Option<f64> = self.con.get(&key)?;

        match current {
            Some(current_price) => {
                let change_percent = ((new_price - current_price) / current_price).abs() * 100.0;

                if change_percent >= threshold_percent {
                    let _: () = self.con.set_ex(&key, new_price, 60)?;
                    println!(
                        "{}: Цена обновлена {} -> {} (изменение {:.2}%)",
                        symbol, current_price, new_price, change_percent
                    );
                    Ok(true)
                } else {
                    // Только продлеваем TTL
                    let _: () = self.con.expire(&key, 60)?;
                    Ok(false)
                }
            }
            None => {
                // Цены нет — устанавливаем новую
                let _: () = self.con.set_ex(&key, new_price, 60)?;
                Ok(true)
            }
        }
    }

    // Инвалидация по паттерну (осторожно — может быть медленной!)
    fn invalidate_by_pattern(&mut self, pattern: &str) -> RedisResult<usize> {
        let search_pattern = format!("price:{}*", pattern);
        let keys: Vec<String> = self.con.keys(&search_pattern)?;

        let count = keys.len();
        for key in keys {
            let _: () = self.con.del(&key)?;
        }

        Ok(count)
    }

    // Инвалидация всех цен биржи
    fn invalidate_exchange(&mut self, exchange: &str) -> RedisResult<usize> {
        self.invalidate_by_pattern(&format!("{}:", exchange))
    }

    // Проверка и обновление устаревших цен
    fn refresh_stale_prices<F>(
        &mut self,
        symbols: &[&str],
        max_age_seconds: i64,
        fetch_price: F
    ) -> RedisResult<Vec<String>>
    where
        F: Fn(&str) -> Option<f64>
    {
        let mut refreshed = Vec::new();

        for symbol in symbols {
            let key = format!("price:{}", symbol);
            let ttl: i64 = self.con.ttl(&key)?;

            // Если TTL меньше порога — обновляем
            if ttl < max_age_seconds || ttl < 0 {
                if let Some(new_price) = fetch_price(symbol) {
                    let _: () = self.con.set_ex(&key, new_price, 60)?;
                    refreshed.push(symbol.to_string());
                }
            }
        }

        Ok(refreshed)
    }
}

fn main() -> RedisResult<()> {
    let mut cache = PriceCacheWithInvalidation::new("redis://127.0.0.1/")?;

    // Установка начальных цен
    cache.set_price_with_ttl("BTC/USDT", 42000.0, 60)?;
    cache.set_price_with_ttl("ETH/USDT", 2200.0, 60)?;

    // Условное обновление
    println!("--- Условное обновление ---");
    cache.update_if_significant_change("BTC/USDT", 42010.0, 0.1)?; // Не обновится (< 0.1%)
    cache.update_if_significant_change("BTC/USDT", 42100.0, 0.1)?; // Обновится (> 0.1%)

    // Инвалидация
    println!("\n--- Инвалидация ---");
    let count = cache.invalidate_by_pattern("ETH")?;
    println!("Инвалидировано {} ключей с ETH", count);

    Ok(())
}
```

## Интеграция с торговой стратегией

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;

struct PriceFeed {
    con: redis::Connection,
}

impl PriceFeed {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceFeed { con })
    }

    fn get_price(&mut self, symbol: &str) -> RedisResult<Option<f64>> {
        let key = format!("price:{}", symbol);
        self.con.get(&key)
    }

    fn set_price(&mut self, symbol: &str, price: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        self.con.set_ex(&key, price, 60)
    }
}

struct Portfolio {
    positions: HashMap<String, f64>, // symbol -> quantity
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            positions: HashMap::new(),
        }
    }

    fn add_position(&mut self, symbol: &str, quantity: f64) {
        *self.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;
    }

    // Расчёт стоимости портфеля используя кэшированные цены
    fn calculate_value(&self, price_feed: &mut PriceFeed) -> RedisResult<f64> {
        let mut total = 0.0;

        for (symbol, quantity) in &self.positions {
            if let Some(price) = price_feed.get_price(symbol)? {
                total += price * quantity;
            } else {
                println!("Предупреждение: нет цены для {}", symbol);
            }
        }

        Ok(total)
    }

    // Расчёт экспозиции по каждому активу
    fn get_exposures(&self, price_feed: &mut PriceFeed) -> RedisResult<HashMap<String, f64>> {
        let total_value = self.calculate_value(price_feed)?;
        let mut exposures = HashMap::new();

        for (symbol, quantity) in &self.positions {
            if let Some(price) = price_feed.get_price(symbol)? {
                let position_value = price * quantity;
                let exposure = if total_value > 0.0 {
                    (position_value / total_value) * 100.0
                } else {
                    0.0
                };
                exposures.insert(symbol.clone(), exposure);
            }
        }

        Ok(exposures)
    }
}

struct RiskManager {
    max_position_percent: f64,
    max_drawdown_percent: f64,
}

impl RiskManager {
    fn new(max_position: f64, max_drawdown: f64) -> Self {
        RiskManager {
            max_position_percent: max_position,
            max_drawdown_percent: max_drawdown,
        }
    }

    // Проверка рисков на основе текущих цен
    fn check_risks(
        &self,
        portfolio: &Portfolio,
        price_feed: &mut PriceFeed
    ) -> RedisResult<Vec<String>> {
        let mut warnings = Vec::new();
        let exposures = portfolio.get_exposures(price_feed)?;

        for (symbol, exposure) in &exposures {
            if *exposure > self.max_position_percent {
                warnings.push(format!(
                    "Позиция {} превышает лимит: {:.1}% > {:.1}%",
                    symbol, exposure, self.max_position_percent
                ));
            }
        }

        Ok(warnings)
    }
}

fn main() -> RedisResult<()> {
    let mut price_feed = PriceFeed::new("redis://127.0.0.1/")?;

    // Устанавливаем текущие цены
    price_feed.set_price("BTC/USDT", 42000.0)?;
    price_feed.set_price("ETH/USDT", 2200.0)?;
    price_feed.set_price("SOL/USDT", 95.0)?;

    // Создаём портфель
    let mut portfolio = Portfolio::new();
    portfolio.add_position("BTC/USDT", 2.5);   // 2.5 BTC
    portfolio.add_position("ETH/USDT", 15.0);  // 15 ETH
    portfolio.add_position("SOL/USDT", 100.0); // 100 SOL

    // Рассчитываем стоимость
    let total_value = portfolio.calculate_value(&mut price_feed)?;
    println!("Общая стоимость портфеля: ${:.2}", total_value);

    // Получаем экспозиции
    let exposures = portfolio.get_exposures(&mut price_feed)?;
    println!("\nЭкспозиции:");
    for (symbol, exposure) in &exposures {
        println!("  {}: {:.1}%", symbol, exposure);
    }

    // Проверка рисков
    let risk_manager = RiskManager::new(50.0, 10.0);
    let warnings = risk_manager.check_risks(&portfolio, &mut price_feed)?;

    if !warnings.is_empty() {
        println!("\nПредупреждения о рисках:");
        for warning in &warnings {
            println!("  ⚠️ {}", warning);
        }
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SET/GET с TTL | Базовое кэширование цен с автоматическим устареванием |
| HSET/HGET | Хранение структурированных данных о цене в Hash |
| Pipeline | Пакетные операции для эффективности |
| TTL | Автоматическая инвалидация устаревших цен |
| История цен | Хранение в списках Redis (LPUSH/LRANGE) |
| Условное обновление | Обновление только при значительных изменениях |

## Упражнения

1. **Кэш OHLCV**: Реализуй кэширование данных свечей (Open, High, Low, Close, Volume) с использованием Hash. Добавь методы для получения данных за период.

2. **Агрегатор цен**: Создай систему, которая принимает цены от нескольких бирж и вычисляет средневзвешенную цену. Храни цены каждой биржи отдельно.

3. **Детектор аномалий**: Реализуй функцию, которая сравнивает новую цену с историей и логирует предупреждение при резком изменении (более 5% от средней).

## Домашнее задание

1. **Полноценный ценовой кэш**: Создай структуру `AdvancedPriceCache` с методами:
   - `update_price(symbol, bid, ask, last, volume)` — обновление цены
   - `get_vwap(symbol, period)` — расчёт VWAP за период
   - `get_volatility(symbol, period)` — расчёт волатильности
   - `subscribe_to_updates(callback)` — подписка на обновления (подготовка к Pub/Sub)

2. **Мультибиржевой арбитраж**: Реализуй систему, которая:
   - Хранит цены одного инструмента от разных бирж
   - Находит арбитражные возможности (разница цен > 0.1%)
   - Логирует потенциальные сделки

3. **Мониторинг портфеля**: Создай веб-сервис (используя `actix-web` или `axum`), который:
   - Принимает обновления цен через REST API
   - Сохраняет их в Redis
   - Отдаёт текущую стоимость портфеля по GET-запросу

4. **Стресс-тест**: Напиши benchmark, который:
   - Записывает 10,000 обновлений цен в секунду
   - Одновременно читает случайные цены
   - Измеряет latency операций

## Навигация

[← Предыдущий день](../235-redis-rs-connection/ru.md) | [Следующий день →](../237-redis-pubsub-notifications/ru.md)
