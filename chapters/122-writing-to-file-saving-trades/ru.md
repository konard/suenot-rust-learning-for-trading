# День 122: Запись в файл — сохраняем сделки

## Аналогия из трейдинга

Каждый трейдер ведёт журнал сделок. В конце торгового дня ты записываешь все совершённые операции: время, актив, цену входа, цену выхода, объём, результат. Без этого журнала невозможно анализировать свою торговлю, находить ошибки и улучшать стратегию.

В программировании запись в файл — это цифровой аналог такого журнала. Мы **сохраняем** данные на диск, чтобы они не пропали после закрытия программы.

## Базовая запись в файл

```rust
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut file = File::create("trades.txt")?;

    writeln!(file, "BTCUSDT,BUY,42000.00,0.5")?;
    writeln!(file, "ETHUSDT,SELL,2500.00,2.0")?;

    println!("Trades saved to trades.txt");
    Ok(())
}
```

**Важно:**
- `File::create` создаёт новый файл или перезаписывает существующий
- `?` оператор передаёт ошибку вверх
- `writeln!` записывает строку с переводом строки

## Запись структуры сделки

```rust
use std::fs::File;
use std::io::Write;

struct Trade {
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

impl Trade {
    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.4},{:.2}",
            self.symbol, self.side, self.entry_price,
            self.exit_price, self.quantity, self.pnl
        )
    }
}

fn main() -> std::io::Result<()> {
    let trades = vec![
        Trade {
            symbol: String::from("BTCUSDT"),
            side: String::from("LONG"),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.5,
            pnl: 750.0,
        },
        Trade {
            symbol: String::from("ETHUSDT"),
            side: String::from("SHORT"),
            entry_price: 2500.0,
            exit_price: 2400.0,
            quantity: 2.0,
            pnl: 200.0,
        },
    ];

    let mut file = File::create("trade_log.csv")?;

    // Заголовок CSV
    writeln!(file, "symbol,side,entry_price,exit_price,quantity,pnl")?;

    // Записываем каждую сделку
    for trade in &trades {
        writeln!(file, "{}", trade.to_csv_line())?;
    }

    println!("Saved {} trades to trade_log.csv", trades.len());
    Ok(())
}
```

## Добавление в существующий файл (Append)

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // Открываем файл для добавления (создаём, если не существует)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("trade_journal.txt")?;

    let new_trade = "2024-01-15,BTCUSDT,LONG,43000.00,43800.00,0.3,+240.00";
    writeln!(file, "{}", new_trade)?;

    println!("Trade appended to journal");
    Ok(())
}
```

## Запись с буферизацией

Для большого количества записей используй буферизованный вывод:

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let file = File::create("price_history.csv")?;
    let mut writer = BufWriter::new(file);

    // Заголовок
    writeln!(writer, "timestamp,price,volume")?;

    // Симуляция записи большого количества данных
    let prices = vec![
        (1705305600, 42150.50, 125.5),
        (1705305660, 42165.75, 89.2),
        (1705305720, 42140.00, 210.8),
        (1705305780, 42180.25, 156.3),
        (1705305840, 42200.00, 178.9),
    ];

    for (ts, price, volume) in prices {
        writeln!(writer, "{},{:.2},{:.1}", ts, price, volume)?;
    }

    // Буфер автоматически сбрасывается при drop,
    // но можно сделать это явно
    writer.flush()?;

    println!("Price history saved with buffering");
    Ok(())
}
```

## Сохранение в JSON формате

```rust
use std::fs::File;
use std::io::Write;

struct Portfolio {
    account_id: String,
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }
}

fn portfolio_to_json(portfolio: &Portfolio) -> String {
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"account_id\": \"{}\",\n", portfolio.account_id));
    json.push_str(&format!("  \"balance\": {:.2},\n", portfolio.balance));
    json.push_str("  \"positions\": [\n");

    for (i, pos) in portfolio.positions.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"symbol\": \"{}\",\n", pos.symbol));
        json.push_str(&format!("      \"quantity\": {:.4},\n", pos.quantity));
        json.push_str(&format!("      \"entry_price\": {:.2},\n", pos.entry_price));
        json.push_str(&format!("      \"current_price\": {:.2},\n", pos.current_price));
        json.push_str(&format!("      \"unrealized_pnl\": {:.2}\n", pos.unrealized_pnl()));
        json.push_str("    }");
        if i < portfolio.positions.len() - 1 {
            json.push_str(",");
        }
        json.push_str("\n");
    }

    json.push_str("  ]\n");
    json.push_str("}");
    json
}

fn main() -> std::io::Result<()> {
    let portfolio = Portfolio {
        account_id: String::from("ACC-001"),
        balance: 50000.0,
        positions: vec![
            Position {
                symbol: String::from("BTCUSDT"),
                quantity: 0.5,
                entry_price: 42000.0,
                current_price: 43500.0,
            },
            Position {
                symbol: String::from("ETHUSDT"),
                quantity: 5.0,
                entry_price: 2400.0,
                current_price: 2550.0,
            },
        ],
    };

    let json = portfolio_to_json(&portfolio);

    let mut file = File::create("portfolio.json")?;
    file.write_all(json.as_bytes())?;

    println!("Portfolio saved to portfolio.json");
    Ok(())
}
```

## Обработка ошибок записи

```rust
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn save_trades(trades: &[String], filename: &str) -> Result<usize, String> {
    // Проверяем, что есть что сохранять
    if trades.is_empty() {
        return Err(String::from("No trades to save"));
    }

    // Проверяем директорию
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        if !parent.exists() && !parent.as_os_str().is_empty() {
            return Err(format!("Directory does not exist: {:?}", parent));
        }
    }

    // Пытаемся создать и записать файл
    let mut file = File::create(filename)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut count = 0;
    for trade in trades {
        writeln!(file, "{}", trade)
            .map_err(|e| format!("Failed to write trade: {}", e))?;
        count += 1;
    }

    Ok(count)
}

fn main() {
    let trades = vec![
        String::from("BTCUSDT,BUY,42000,0.5"),
        String::from("ETHUSDT,SELL,2500,2.0"),
        String::from("SOLUSDT,BUY,95,10.0"),
    ];

    match save_trades(&trades, "my_trades.csv") {
        Ok(count) => println!("Successfully saved {} trades", count),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Пример с пустым списком
    let empty: Vec<String> = vec![];
    match save_trades(&empty, "empty.csv") {
        Ok(count) => println!("Saved {} trades", count),
        Err(e) => eprintln!("Expected error: {}", e),
    }
}
```

## Практический пример: полный торговый журнал

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Clone)]
struct TradeEntry {
    timestamp: String,
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    commission: f64,
}

impl TradeEntry {
    fn gross_pnl(&self) -> f64 {
        let multiplier = if self.side == "LONG" { 1.0 } else { -1.0 };
        (self.exit_price - self.entry_price) * self.quantity * multiplier
    }

    fn net_pnl(&self) -> f64 {
        self.gross_pnl() - self.commission
    }

    fn to_csv(&self) -> String {
        format!(
            "{},{},{},{:.2},{:.2},{:.4},{:.2},{:.2},{:.2}",
            self.timestamp,
            self.symbol,
            self.side,
            self.entry_price,
            self.exit_price,
            self.quantity,
            self.commission,
            self.gross_pnl(),
            self.net_pnl()
        )
    }
}

struct TradeJournal {
    filename: String,
    trades: Vec<TradeEntry>,
}

impl TradeJournal {
    fn new(filename: &str) -> Self {
        TradeJournal {
            filename: String::from(filename),
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: TradeEntry) {
        self.trades.push(trade);
    }

    fn save(&self) -> std::io::Result<()> {
        let file = File::create(&self.filename)?;
        let mut writer = BufWriter::new(file);

        // Заголовок
        writeln!(writer, "timestamp,symbol,side,entry,exit,qty,commission,gross_pnl,net_pnl")?;

        // Данные
        for trade in &self.trades {
            writeln!(writer, "{}", trade.to_csv())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn save_summary(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        let total_trades = self.trades.len();
        let winning: Vec<_> = self.trades.iter().filter(|t| t.net_pnl() > 0.0).collect();
        let losing: Vec<_> = self.trades.iter().filter(|t| t.net_pnl() < 0.0).collect();

        let total_pnl: f64 = self.trades.iter().map(|t| t.net_pnl()).sum();
        let total_commission: f64 = self.trades.iter().map(|t| t.commission).sum();

        let win_rate = if total_trades > 0 {
            (winning.len() as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        writeln!(file, "╔════════════════════════════════════╗")?;
        writeln!(file, "║       TRADING JOURNAL SUMMARY      ║")?;
        writeln!(file, "╠════════════════════════════════════╣")?;
        writeln!(file, "║ Total Trades:      {:>15} ║", total_trades)?;
        writeln!(file, "║ Winning Trades:    {:>15} ║", winning.len())?;
        writeln!(file, "║ Losing Trades:     {:>15} ║", losing.len())?;
        writeln!(file, "║ Win Rate:          {:>14.1}% ║", win_rate)?;
        writeln!(file, "║ Total PnL:        ${:>14.2} ║", total_pnl)?;
        writeln!(file, "║ Total Commission: ${:>14.2} ║", total_commission)?;
        writeln!(file, "╚════════════════════════════════════╝")?;

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut journal = TradeJournal::new("trading_journal.csv");

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 09:30:00"),
        symbol: String::from("BTCUSDT"),
        side: String::from("LONG"),
        entry_price: 42000.0,
        exit_price: 43200.0,
        quantity: 0.5,
        commission: 42.6,
    });

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 11:45:00"),
        symbol: String::from("ETHUSDT"),
        side: String::from("SHORT"),
        entry_price: 2500.0,
        exit_price: 2450.0,
        quantity: 2.0,
        commission: 9.9,
    });

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 14:20:00"),
        symbol: String::from("SOLUSDT"),
        side: String::from("LONG"),
        entry_price: 95.0,
        exit_price: 92.0,
        quantity: 10.0,
        commission: 1.87,
    });

    journal.save()?;
    journal.save_summary("summary.txt")?;

    println!("Journal and summary saved successfully!");
    Ok(())
}
```

## Атомарная запись (безопасное сохранение)

```rust
use std::fs::{self, File};
use std::io::Write;

fn save_atomically(data: &str, filename: &str) -> std::io::Result<()> {
    let temp_filename = format!("{}.tmp", filename);

    // Записываем во временный файл
    {
        let mut file = File::create(&temp_filename)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;  // Убеждаемся, что данные записаны на диск
    }

    // Атомарно переименовываем
    fs::rename(&temp_filename, filename)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let portfolio_data = "BTC: 0.5\nETH: 5.0\nSOL: 100.0";

    save_atomically(portfolio_data, "portfolio_backup.txt")?;

    println!("Portfolio saved atomically");
    Ok(())
}
```

## Что мы узнали

| Операция | Способ | Когда использовать |
|----------|--------|-------------------|
| Создание файла | `File::create` | Новый файл или перезапись |
| Добавление | `OpenOptions::append` | Журнал, лог сделок |
| Буферизация | `BufWriter` | Много мелких записей |
| Формат CSV | `writeln!` с форматом | Экспорт данных |
| Формат JSON | Ручной или serde | API, веб-сервисы |
| Атомарная запись | temp + rename | Критичные данные |

## Практические задания

1. **Экспорт сделок в CSV**
   Напиши функцию, которая принимает вектор сделок и сохраняет их в CSV файл с заголовками.

2. **Журнал ордеров**
   Создай функцию `append_order`, которая добавляет новый ордер в конец файла без перезаписи существующих данных.

3. **Резервное копирование**
   Реализуй функцию, которая сохраняет состояние портфеля и автоматически создаёт резервную копию предыдущей версии.

4. **Отчёт по торговому дню**
   Напиши функцию, которая генерирует текстовый отчёт с итогами дня: количество сделок, общий PnL, лучшая и худшая сделки.

## Домашнее задание

1. Создай структуру `PriceLogger`, которая записывает цены в файл в реальном времени с буферизацией, сбрасывая буфер каждые N записей.

2. Реализуй систему ротации логов: когда файл превышает определённый размер, переименуй его с добавлением timestamp и начни новый файл.

3. Напиши функцию `export_trades_report(trades: &[Trade], format: &str)`, которая экспортирует сделки в указанном формате (csv, json, txt).

4. Создай механизм автосохранения: каждые N операций или через определённый интервал данные автоматически сохраняются на диск.

## Навигация

[← Предыдущий день](../121-reading-from-file-parsing-market-data/ru.md) | [Следующий день →](../123-file-error-handling-corrupted-data/ru.md)
