# День 213: Зачем БД: персистентность данных

## Аналогия из трейдинга

Представь, что ты запускаешь торгового бота. Он работает целый день, совершает сотни сделок, зарабатывает прибыль. А потом... ты перезагружаешь компьютер. И что? Все данные — история сделок, текущие позиции, статистика — исчезли.

Это как если бы брокер каждое утро забывал, сколько акций у тебя есть. Представь: ты владеешь 100 акциями Apple, выключаешь компьютер на ночь, а утром брокер говорит: "Какие акции? У вас ничего нет!"

**Персистентность данных** — это способность сохранять данные между запусками программы. База данных — это как сейф для твоей торговой информации: надёжно, организованно и всегда доступно.

## Проблема: данные в памяти временны

Когда программа работает, все данные хранятся в оперативной памяти (RAM). Но RAM — это временное хранилище:

```rust
fn main() {
    // Эти данные живут только пока программа работает
    let mut trades = Vec::new();

    trades.push(Trade {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
    });

    trades.push(Trade {
        id: 2,
        symbol: "ETH".to_string(),
        price: 2800.0,
        quantity: 10.0,
    });

    println!("У нас {} сделок", trades.len());

    // Когда программа завершится — все данные исчезнут!
}

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}
```

После завершения программы переменная `trades` полностью исчезает. При следующем запуске — пустой вектор.

## Что мы теряем без персистентности

### 1. История сделок

```rust
// Без базы данных: каждый запуск начинаем с нуля
fn main() {
    let mut trade_history: Vec<Trade> = Vec::new();

    // Добавляем сегодняшние сделки
    trade_history.push(Trade {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
        timestamp: 1699000000,
    });

    // Завтра при перезапуске — история пуста!
    // Не можем анализировать прошлые результаты
    // Не можем считать статистику
    // Не можем отчитаться перед налоговой
}

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
}
```

### 2. Позиции и балансы

```rust
// Опасная ситуация: забыли открытые позиции
struct Portfolio {
    cash: f64,
    positions: std::collections::HashMap<String, f64>,
}

fn main() {
    let mut portfolio = Portfolio {
        cash: 10_000.0,
        positions: std::collections::HashMap::new(),
    };

    // Купили 0.5 BTC
    portfolio.positions.insert("BTC".to_string(), 0.5);
    portfolio.cash -= 21_000.0;

    println!("Баланс: ${}, BTC: {}",
        portfolio.cash,
        portfolio.positions.get("BTC").unwrap_or(&0.0));

    // Программа упала или перезапустилась...
    // При перезапуске: cash = 10_000, positions = {}
    // Мы "забыли" что потратили $21k и владеем BTC!
}
```

### 3. Настройки и конфигурация

```rust
struct TradingConfig {
    max_position_size: f64,
    stop_loss_percent: f64,
    api_keys: std::collections::HashMap<String, String>,
    enabled_pairs: Vec<String>,
}

fn main() {
    // Настроили бота час, а потом он перезапустился...
    let config = TradingConfig {
        max_position_size: 1000.0,
        stop_loss_percent: 2.0,
        api_keys: std::collections::HashMap::new(), // Добавляли ключи вручную
        enabled_pairs: vec![
            "BTC/USDT".to_string(),
            "ETH/USDT".to_string(),
            // ... ещё 20 пар
        ],
    };

    // После перезапуска — всё настраивать заново!
}
```

## Решение: персистентность

Персистентность означает сохранение данных на постоянное хранилище (диск, SSD, облако), откуда их можно восстановить при перезапуске.

### Простейший способ: файлы

```rust
use std::fs;
use std::io::{Write, BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl Trade {
    // Сохраняем в CSV формате
    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.symbol, self.price, self.quantity)
    }

    // Парсим из CSV строки
    fn from_csv_line(line: &str) -> Result<Trade, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Trade {
            id: parts[0].parse()?,
            symbol: parts[1].to_string(),
            price: parts[2].parse()?,
            quantity: parts[3].parse()?,
        })
    }
}

fn save_trades(trades: &[Trade], filename: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(filename)?;
    for trade in trades {
        writeln!(file, "{}", trade.to_csv_line())?;
    }
    Ok(())
}

fn load_trades(filename: &str) -> std::io::Result<Vec<Trade>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(trade) = Trade::from_csv_line(&line) {
            trades.push(trade);
        }
    }

    Ok(trades)
}

fn main() -> std::io::Result<()> {
    let filename = "trades.csv";

    // Загружаем существующие сделки (если файл есть)
    let mut trades = load_trades(filename).unwrap_or_else(|_| Vec::new());

    println!("Загружено {} сделок из прошлого", trades.len());

    // Добавляем новую сделку
    let new_trade = Trade {
        id: trades.len() as u64 + 1,
        symbol: "BTC".to_string(),
        price: 42500.0,
        quantity: 0.1,
    };

    trades.push(new_trade);
    println!("Добавлена новая сделка, всего: {}", trades.len());

    // Сохраняем обратно
    save_trades(&trades, filename)?;
    println!("Сделки сохранены в {}", filename);

    Ok(())
}
```

## Почему файлов недостаточно?

Файлы — хорошее начало, но у них есть серьёзные ограничения:

### 1. Производительность

```rust
// Проблема: чтобы найти сделку, нужно читать весь файл
fn find_trade_by_id(filename: &str, target_id: u64) -> Option<Trade> {
    let trades = load_trades(filename).ok()?;

    // O(n) — читаем ВСЕ записи, даже если нужна первая
    for trade in trades {
        if trade.id == target_id {
            return Some(trade);
        }
    }
    None
}

// Если у нас 1 миллион сделок — это медленно!
```

### 2. Одновременный доступ

```rust
// Проблема: два потока пишут одновременно
use std::thread;

fn main() {
    let filename = "trades.csv";

    // Поток 1: добавляет сделку
    let handle1 = thread::spawn(move || {
        // Читаем...
        // ... в это время поток 2 тоже читает
        // Добавляем сделку
        // Сохраняем
        // ... поток 2 перезаписывает наши изменения!
    });

    // Поток 2: добавляет другую сделку
    let handle2 = thread::spawn(move || {
        // Те же действия — race condition!
    });

    // Результат: одна из сделок потеряна
}
```

### 3. Сложные запросы

```rust
// Нужно найти: все сделки по BTC за последний месяц с прибылью > 100$
// С файлами — это кошмар:

fn find_profitable_btc_trades(filename: &str) -> Vec<Trade> {
    let trades = load_trades(filename).unwrap();
    let month_ago = current_timestamp() - 30 * 24 * 60 * 60;

    trades.into_iter()
        .filter(|t| t.symbol == "BTC")
        .filter(|t| t.timestamp > month_ago)
        .filter(|t| calculate_profit(t) > 100.0)
        .collect()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn calculate_profit(_trade: &Trade) -> f64 {
    // Нужно загружать ещё данные о текущих ценах...
    0.0
}

// С базой данных это один SQL запрос!
```

### 4. Целостность данных

```rust
// Проблема: программа упала посередине записи
fn save_trade_and_update_portfolio(trade: &Trade, portfolio: &mut Portfolio) {
    // Шаг 1: сохраняем сделку
    save_trade_to_file(trade);

    // ПРОГРАММА УПАЛА ЗДЕСЬ!

    // Шаг 2: обновляем баланс — не выполнился
    portfolio.cash -= trade.price * trade.quantity;
    save_portfolio_to_file(portfolio);

    // Результат: сделка записана, но баланс не обновлён
    // Данные в несогласованном состоянии!
}

fn save_trade_to_file(_trade: &Trade) {}
fn save_portfolio_to_file(_portfolio: &Portfolio) {}

struct Portfolio {
    cash: f64,
}
```

## Что даёт база данных

### 1. Индексы — быстрый поиск

База данных создаёт специальные структуры (индексы), которые позволяют искать данные мгновенно:

```rust
// С базой данных: поиск по id — O(log n) или O(1)
// Вместо просмотра всех 1 миллион записей — 20 операций!

// Псевдокод
fn find_trade_with_db(db: &Database, id: u64) -> Option<Trade> {
    // Индекс по id — мгновенный поиск
    db.query("SELECT * FROM trades WHERE id = ?", &[id])
}
```

### 2. Транзакции — атомарные операции

```rust
// База данных гарантирует: либо всё выполнится, либо ничего
fn execute_trade_atomically(db: &Database, trade: &Trade, portfolio: &mut Portfolio) {
    db.transaction(|tx| {
        // Шаг 1: записываем сделку
        tx.execute("INSERT INTO trades ...", trade)?;

        // Шаг 2: обновляем баланс
        tx.execute("UPDATE portfolio SET cash = cash - ?",
                   trade.price * trade.quantity)?;

        // Если любой шаг провалился — откатываем всё
        Ok(())
    });
    // Гарантия: данные всегда согласованы!
}

struct Database;
impl Database {
    fn transaction<F, R>(&self, _f: F) -> R
    where F: FnOnce(&Transaction) -> R {
        unimplemented!()
    }
}

struct Transaction;
impl Transaction {
    fn execute(&self, _query: &str, _params: impl std::fmt::Debug) -> Result<(), ()> {
        Ok(())
    }
}
```

### 3. SQL — мощный язык запросов

```sql
-- Сложный запрос, который с файлами занял бы 50 строк кода:

SELECT
    symbol,
    COUNT(*) as trade_count,
    SUM(quantity * price) as total_volume,
    AVG(price) as avg_price
FROM trades
WHERE timestamp > datetime('now', '-30 days')
GROUP BY symbol
HAVING total_volume > 10000
ORDER BY total_volume DESC;
```

### 4. Параллельный доступ

```rust
// База данных сама управляет блокировками
use std::thread;

fn main() {
    // Оба потока могут безопасно работать с базой одновременно
    let handle1 = thread::spawn(|| {
        let db = connect_to_database();
        db.insert_trade(/* ... */);  // БД сама заблокирует нужные строки
    });

    let handle2 = thread::spawn(|| {
        let db = connect_to_database();
        db.insert_trade(/* ... */);  // Безопасно работает параллельно
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

fn connect_to_database() -> Database {
    Database
}

impl Database {
    fn insert_trade(&self) {}
}
```

## Практический пример: торговый журнал

Вот полный пример системы, которая сохраняет торговые данные персистентно:

```rust
use std::fs;
use std::collections::HashMap;

/// Торговый журнал с персистентностью в файле
/// (В реальном проекте используй базу данных!)
#[derive(Debug)]
struct TradingJournal {
    trades: Vec<Trade>,
    portfolio: Portfolio,
    filename: String,
}

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: TradeSide,
    price: f64,
    quantity: f64,
    timestamp: i64,
}

#[derive(Debug, Clone)]
enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Portfolio {
    cash: f64,
    positions: HashMap<String, f64>,
}

impl TradingJournal {
    /// Создаёт журнал или загружает существующий
    fn new(filename: &str) -> Self {
        // Попробуем загрузить существующие данные
        if let Ok(data) = fs::read_to_string(filename) {
            if let Some(journal) = Self::deserialize(&data) {
                println!("Загружен журнал: {} сделок, баланс ${:.2}",
                    journal.trades.len(), journal.portfolio.cash);
                return journal;
            }
        }

        // Создаём новый журнал
        println!("Создан новый журнал");
        TradingJournal {
            trades: Vec::new(),
            portfolio: Portfolio {
                cash: 100_000.0,  // Стартовый капитал
                positions: HashMap::new(),
            },
            filename: filename.to_string(),
        }
    }

    /// Добавляет сделку и сохраняет
    fn add_trade(&mut self, symbol: &str, side: TradeSide,
                 price: f64, quantity: f64) -> Result<u64, String> {
        let cost = price * quantity;

        // Проверяем возможность сделки
        match &side {
            TradeSide::Buy => {
                if self.portfolio.cash < cost {
                    return Err(format!("Недостаточно средств: нужно ${:.2}, есть ${:.2}",
                        cost, self.portfolio.cash));
                }
            }
            TradeSide::Sell => {
                let position = self.portfolio.positions.get(symbol).unwrap_or(&0.0);
                if *position < quantity {
                    return Err(format!("Недостаточно {}: нужно {}, есть {}",
                        symbol, quantity, position));
                }
            }
        }

        // Создаём сделку
        let trade = Trade {
            id: self.trades.len() as u64 + 1,
            symbol: symbol.to_string(),
            side: side.clone(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        // Обновляем портфель
        match side {
            TradeSide::Buy => {
                self.portfolio.cash -= cost;
                *self.portfolio.positions
                    .entry(symbol.to_string())
                    .or_insert(0.0) += quantity;
            }
            TradeSide::Sell => {
                self.portfolio.cash += cost;
                if let Some(pos) = self.portfolio.positions.get_mut(symbol) {
                    *pos -= quantity;
                    if *pos <= 0.0 {
                        self.portfolio.positions.remove(symbol);
                    }
                }
            }
        }

        let trade_id = trade.id;
        self.trades.push(trade);

        // ВАЖНО: сохраняем после каждой операции
        self.save()?;

        Ok(trade_id)
    }

    /// Сохраняет журнал в файл
    fn save(&self) -> Result<(), String> {
        let data = self.serialize();
        fs::write(&self.filename, data)
            .map_err(|e| format!("Ошибка сохранения: {}", e))
    }

    /// Сериализация в простой текстовый формат
    fn serialize(&self) -> String {
        let mut lines = Vec::new();

        // Заголовок портфеля
        lines.push(format!("CASH:{}", self.portfolio.cash));
        for (symbol, qty) in &self.portfolio.positions {
            lines.push(format!("POS:{}:{}", symbol, qty));
        }

        // Сделки
        lines.push("TRADES".to_string());
        for trade in &self.trades {
            let side_str = match trade.side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            };
            lines.push(format!("{}:{}:{}:{}:{}:{}",
                trade.id, trade.symbol, side_str,
                trade.price, trade.quantity, trade.timestamp));
        }

        lines.join("\n")
    }

    /// Десериализация из текста
    fn deserialize(data: &str) -> Option<Self> {
        let mut cash = 100_000.0;
        let mut positions = HashMap::new();
        let mut trades = Vec::new();
        let mut reading_trades = false;

        for line in data.lines() {
            if line.starts_with("CASH:") {
                cash = line[5..].parse().ok()?;
            } else if line.starts_with("POS:") {
                let parts: Vec<&str> = line[4..].split(':').collect();
                if parts.len() == 2 {
                    positions.insert(
                        parts[0].to_string(),
                        parts[1].parse().ok()?
                    );
                }
            } else if line == "TRADES" {
                reading_trades = true;
            } else if reading_trades {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 6 {
                    let side = match parts[2] {
                        "BUY" => TradeSide::Buy,
                        "SELL" => TradeSide::Sell,
                        _ => continue,
                    };
                    trades.push(Trade {
                        id: parts[0].parse().ok()?,
                        symbol: parts[1].to_string(),
                        side,
                        price: parts[3].parse().ok()?,
                        quantity: parts[4].parse().ok()?,
                        timestamp: parts[5].parse().ok()?,
                    });
                }
            }
        }

        Some(TradingJournal {
            trades,
            portfolio: Portfolio { cash, positions },
            filename: String::new(),
        })
    }

    /// Показывает статус
    fn status(&self) {
        println!("\n=== Статус портфеля ===");
        println!("Денежные средства: ${:.2}", self.portfolio.cash);
        println!("Позиции:");
        for (symbol, qty) in &self.portfolio.positions {
            println!("  {}: {:.4}", symbol, qty);
        }
        println!("Всего сделок: {}", self.trades.len());

        if let Some(last) = self.trades.last() {
            let side = match last.side {
                TradeSide::Buy => "Покупка",
                TradeSide::Sell => "Продажа",
            };
            println!("Последняя сделка: {} {} {} @ ${:.2}",
                side, last.quantity, last.symbol, last.price);
        }
    }
}

fn main() {
    // Журнал сохраняется в файл и загружается при перезапуске
    let mut journal = TradingJournal::new("trading_journal.dat");

    journal.status();

    // Симулируем торговлю
    println!("\n=== Новые сделки ===");

    match journal.add_trade("BTC", TradeSide::Buy, 42000.0, 0.5) {
        Ok(id) => println!("Сделка #{}: Купили 0.5 BTC @ $42000", id),
        Err(e) => println!("Ошибка: {}", e),
    }

    match journal.add_trade("ETH", TradeSide::Buy, 2800.0, 5.0) {
        Ok(id) => println!("Сделка #{}: Купили 5 ETH @ $2800", id),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Попробуем продать то, чего нет
    match journal.add_trade("SOL", TradeSide::Sell, 100.0, 10.0) {
        Ok(id) => println!("Сделка #{}: Продали 10 SOL", id),
        Err(e) => println!("Ошибка: {}", e),
    }

    journal.status();

    println!("\n=== При следующем запуске программы данные сохранятся! ===");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Персистентность | Сохранение данных между запусками программы |
| RAM vs Диск | RAM — временно, диск — постоянно |
| Файлы | Простой способ, но с ограничениями |
| База данных | Мощное решение: индексы, транзакции, параллельность |
| Транзакции | Атомарные операции — всё или ничего |
| Индексы | Быстрый поиск без просмотра всех записей |

## Домашнее задание

1. **Сохранение конфигурации**: Создай структуру `BotConfig` с полями для API ключей, торговых пар и лимитов. Реализуй сохранение/загрузку в JSON файл. Проверь, что настройки сохраняются между запусками.

2. **История сделок с поиском**: Расширь пример `TradingJournal`:
   - Добавь метод `find_trades_by_symbol(symbol: &str)` — поиск сделок по тикеру
   - Добавь метод `get_total_pnl()` — расчёт общего P&L
   - Добавь метод `get_trades_after(timestamp: i64)` — сделки после определённого времени

3. **Атомарное сохранение**: Модифицируй метод `save()`:
   - Сначала пиши во временный файл `.tmp`
   - Затем переименовывай в основной файл
   - Это защитит от потери данных при сбое во время записи

4. **Подумай о базе данных**: Какие операции в твоём примере были бы проще с SQL базой данных? Запиши 3-5 примеров запросов, которые ты хотел бы выполнять.

## Навигация

[← Предыдущий день](../212-project-realtime-price-monitor/ru.md) | [Следующий день →](../214-sqlite-embedded-database/ru.md)
