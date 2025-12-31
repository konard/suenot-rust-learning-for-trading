# День 48: Строковые срезы — часть названия биржи

## Аналогия из трейдинга

Представь, что ты работаешь с биржевыми данными:
- Полное название биржи: **"Binance Futures"** → нужно извлечь **"Binance"**
- Торговая пара: **"BTC/USDT"** → нужно получить только **"BTC"** или **"USDT"**
- Order ID: **"ORD-2024-001234"** → извлечь год **"2024"** или номер **"001234"**
- Лог: **"[ERROR] Connection failed"** → получить уровень **"ERROR"**

В трейдинге мы постоянно работаем с частями строк — срезами. Rust предоставляет безопасный и эффективный способ работы с такими данными через **строковые срезы** (`&str`).

## Что такое строковый срез?

Строковый срез — это **ссылка на часть строки** без копирования данных:

```rust
fn main() {
    let exchange = String::from("Binance Futures");

    // Срез первых 7 символов (байт)
    let name: &str = &exchange[0..7];
    println!("Exchange: {}", name);  // Binance

    // Срез с 8-го до конца
    let suffix: &str = &exchange[8..];
    println!("Type: {}", suffix);  // Futures

    // Вся строка как срез
    let full: &str = &exchange[..];
    println!("Full: {}", full);  // Binance Futures
}
```

**Аналогия:** Представь длинный свиток с данными. Срез — это "окно" в определённую часть свитка. Ты видишь данные, но не копируешь их.

## Синтаксис срезов

```rust
fn main() {
    let pair = String::from("ETH/USDT");

    // [start..end] - от start до end (не включая end)
    let base = &pair[0..3];      // "ETH"

    // [start..] - от start до конца
    let from_slash = &pair[3..]; // "/USDT"

    // [..end] - от начала до end
    let to_slash = &pair[..3];   // "ETH"

    // [..] - вся строка
    let full = &pair[..];        // "ETH/USDT"

    println!("Base: {}", base);
    println!("From slash: {}", from_slash);
    println!("To slash: {}", to_slash);
    println!("Full: {}", full);
}
```

## Срезы работают с байтами, не символами!

**Важно:** Индексы в срезах — это **байты**, а не символы. Для ASCII это не проблема, но для Unicode может привести к панике:

```rust
fn main() {
    // ASCII - всё работает (1 символ = 1 байт)
    let ticker = "BTC";
    let first_two = &ticker[0..2];
    println!("First two: {}", first_two);  // "BT"

    // Unicode - нужно быть осторожным!
    let currency = "₿itcoin";  // ₿ занимает 3 байта

    // Это вызовет панику!
    // let wrong = &currency[0..1];  // ПАНИКА!

    // Правильно - по границам символов
    let symbol = &currency[0..3];  // "₿" (3 байта)
    println!("Symbol: {}", symbol);

    let rest = &currency[3..];  // "itcoin"
    println!("Rest: {}", rest);
}
```

## Безопасное извлечение срезов

Для безопасной работы используем методы `get()`:

```rust
fn main() {
    let exchange = "Binance";

    // get() возвращает Option<&str>
    if let Some(slice) = exchange.get(0..3) {
        println!("First 3: {}", slice);  // "Bin"
    }

    // Безопасно - не паникует при неверных индексах
    let result = exchange.get(0..100);
    println!("Out of bounds: {:?}", result);  // None

    // Безопасно - не паникует при невалидных UTF-8 границах
    let unicode = "₿TC";
    let invalid = unicode.get(0..1);  // None (не на границе символа)
    println!("Invalid UTF-8 boundary: {:?}", invalid);
}
```

## Практический пример: парсинг торговой пары

```rust
fn main() {
    let pairs = ["BTC/USDT", "ETH/BTC", "SOL/USDC", "INVALID"];

    for pair in pairs {
        match parse_trading_pair(pair) {
            Some((base, quote)) => {
                println!("{}: base={}, quote={}", pair, base, quote);
            }
            None => {
                println!("{}: invalid format", pair);
            }
        }
    }
}

fn parse_trading_pair(pair: &str) -> Option<(&str, &str)> {
    // Находим позицию разделителя
    let slash_pos = pair.find('/')?;

    // Извлекаем срезы до и после разделителя
    let base = pair.get(..slash_pos)?;
    let quote = pair.get(slash_pos + 1..)?;

    // Проверяем, что обе части не пустые
    if base.is_empty() || quote.is_empty() {
        return None;
    }

    Some((base, quote))
}
```

## Практический пример: извлечение данных из Order ID

```rust
fn main() {
    let order_ids = [
        "ORD-2024-001234",
        "ORD-2023-999999",
        "INVALID",
    ];

    for order_id in order_ids {
        if let Some(info) = parse_order_id(order_id) {
            println!("Order {}: year={}, number={}",
                     order_id, info.0, info.1);
        } else {
            println!("Invalid order ID: {}", order_id);
        }
    }
}

fn parse_order_id(order_id: &str) -> Option<(&str, &str)> {
    // Формат: ORD-YYYY-NNNNNN
    // Проверяем префикс
    if !order_id.starts_with("ORD-") {
        return None;
    }

    // Извлекаем год (позиции 4-8)
    let year = order_id.get(4..8)?;

    // Проверяем, что год - это цифры
    if !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Извлекаем номер (после второго дефиса)
    let number = order_id.get(9..)?;

    if number.is_empty() {
        return None;
    }

    Some((year, number))
}
```

## Практический пример: парсинг лог-сообщений

```rust
fn main() {
    let logs = [
        "[INFO] Trade executed: BTC/USDT @ 42000",
        "[ERROR] Connection timeout",
        "[WARN] Low balance: 0.001 BTC",
        "Invalid log format",
    ];

    for log in logs {
        if let Some((level, message)) = parse_log(log) {
            println!("Level: {:6} | Message: {}", level, message);
        } else {
            println!("Invalid log: {}", log);
        }
    }
}

fn parse_log(log: &str) -> Option<(&str, &str)> {
    // Формат: [LEVEL] message

    // Проверяем, начинается ли с [
    if !log.starts_with('[') {
        return None;
    }

    // Находим закрывающую скобку
    let end_bracket = log.find(']')?;

    // Извлекаем уровень (между [ и ])
    let level = log.get(1..end_bracket)?;

    // Извлекаем сообщение (после ] и пробела)
    let message_start = end_bracket + 2;  // ] + пробел
    let message = log.get(message_start..)?;

    Some((level, message.trim()))
}
```

## Срезы и функции

Функции, принимающие `&str`, работают как со строковыми литералами, так и со срезами `String`:

```rust
fn main() {
    // Строковый литерал
    let literal: &str = "BINANCE";

    // String
    let owned = String::from("COINBASE");

    // Оба работают с одной функцией
    let binance_short = get_exchange_short_name(literal);
    let coinbase_short = get_exchange_short_name(&owned);

    println!("Binance: {} -> {}", literal, binance_short);
    println!("Coinbase: {} -> {}", owned, coinbase_short);

    // Срез String тоже работает
    let full_name = String::from("Kraken Exchange");
    let kraken_short = get_exchange_short_name(&full_name[0..6]);
    println!("Kraken: {}", kraken_short);
}

fn get_exchange_short_name(name: &str) -> &str {
    // Возвращаем первые 3 символа или всё имя, если короче
    if name.len() >= 3 {
        &name[0..3]
    } else {
        name
    }
}
```

## Практический пример: извлечение цен из текста

```rust
fn main() {
    let messages = [
        "BTC price: $42,500.00",
        "ETH at $2,800.50 now",
        "Current rate: 1.0850",
    ];

    for msg in messages {
        if let Some(price_str) = extract_price_string(msg) {
            println!("Found price string: '{}' in '{}'", price_str, msg);
        } else {
            println!("No price found in: {}", msg);
        }
    }
}

fn extract_price_string(text: &str) -> Option<&str> {
    // Ищем начало числа (цифра или $)
    let start = text.find(|c: char| c.is_ascii_digit() || c == '$')?;

    // Определяем реальное начало числа (пропускаем $)
    let num_start = if text.get(start..start+1) == Some("$") {
        start + 1
    } else {
        start
    };

    // Находим конец числа
    let slice = text.get(num_start..)?;
    let end_offset = slice
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != ',')
        .unwrap_or(slice.len());

    text.get(num_start..num_start + end_offset)
}
```

## Практический пример: нормализация тикеров

```rust
fn main() {
    let tickers = [
        "  btc  ",
        "ETH/USDT",
        "sol",
        " DOGE ",
    ];

    for ticker in tickers {
        let normalized = normalize_ticker(ticker);
        println!("'{}' -> '{}'", ticker, normalized);
    }
}

fn normalize_ticker(ticker: &str) -> String {
    // Убираем пробелы и приводим к верхнему регистру
    let trimmed = ticker.trim();

    // Если есть /, берём только базовую валюту
    if let Some(slash_pos) = trimmed.find('/') {
        trimmed[..slash_pos].to_uppercase()
    } else {
        trimmed.to_uppercase()
    }
}
```

## Срезы и владение (Ownership)

Срезы **заимствуют** данные, а не владеют ими:

```rust
fn main() {
    let exchange = String::from("Binance Futures");

    // Создаём срез - заимствование
    let name = &exchange[0..7];

    // exchange всё ещё доступен!
    println!("Full: {}", exchange);
    println!("Name: {}", name);

    // Но нельзя изменить exchange, пока существует срез
    // exchange.push_str("!");  // ОШИБКА!

    // После последнего использования среза можно изменять
    drop(name);  // Явно "уничтожаем" срез (обычно не нужно)

    let mut exchange = exchange;  // Делаем mutable
    exchange.push_str(" Pro");
    println!("Modified: {}", exchange);
}
```

## Сравнение: срезы vs копирование

```rust
fn main() {
    let data = String::from("BINANCE:BTC/USDT:SPOT:1000.50");

    // Срезы - эффективно, без копирования
    let parts: Vec<&str> = data.split(':').collect();
    println!("Exchange (slice): {}", parts[0]);

    // Копирование - создаём новые String
    let exchange_owned: String = parts[0].to_string();
    println!("Exchange (owned): {}", exchange_owned);

    // Когда использовать что:
    // - Срезы: для чтения, временной обработки
    // - String: когда нужно хранить/модифицировать данные
}
```

## Упражнения

### Упражнение 1: Парсер биржевого сообщения
```rust
// Реализуй функцию, которая извлекает данные из сообщения биржи
// Формат: "EXCHANGE:PAIR:SIDE:PRICE:QUANTITY"
// Пример: "BINANCE:BTC/USDT:BUY:42000.50:0.5"

fn parse_exchange_message(msg: &str) -> Option<TradeInfo> {
    // TODO: реализуй
    todo!()
}

struct TradeInfo<'a> {
    exchange: &'a str,
    pair: &'a str,
    side: &'a str,
    price: &'a str,
    quantity: &'a str,
}
```

### Упражнение 2: Извлечение домена из URL биржи
```rust
// Извлеки домен из URL
// "https://api.binance.com/v3/ticker" -> "binance.com"
// "https://ftx.com/api/markets" -> "ftx.com"

fn extract_domain(url: &str) -> Option<&str> {
    // TODO: реализуй
    todo!()
}
```

### Упражнение 3: Маскирование API ключа
```rust
// Замаскируй API ключ, показывая только первые и последние 4 символа
// "abcd1234efgh5678" -> "abcd********5678"

fn mask_api_key(key: &str) -> String {
    // TODO: реализуй
    todo!()
}
```

### Упражнение 4: Парсер торговой команды
```rust
// Распарси команду вида "buy 0.5 BTC at 42000"
// Верни структуру с полями: action, amount, asset, price

fn parse_trade_command(cmd: &str) -> Option<TradeCommand> {
    // TODO: реализуй
    todo!()
}

struct TradeCommand<'a> {
    action: &'a str,
    amount: &'a str,
    asset: &'a str,
    price: &'a str,
}
```

## Домашнее задание

1. **Парсер WebSocket сообщений**: Напиши функцию, которая парсит JSON-подобные сообщения:
   ```
   {"event":"trade","symbol":"BTC/USDT","price":"42000.50","side":"buy"}
   ```
   Извлеки значения полей используя только срезы (без serde).

2. **Анализатор исторических данных**: Создай функцию для парсинга CSV-строки:
   ```
   2024-01-15,42000.00,42500.00,41800.00,42300.00,1500.5
   ```
   (date,open,high,low,close,volume)

3. **Фильтр тикеров**: Напиши функцию, которая из списка строк извлекает только валидные тикеры (2-5 символов, только буквы).

4. **Маршрутизатор API**: Создай функцию, которая из URL извлекает endpoint и параметры:
   ```
   "/api/v1/ticker?symbol=BTCUSDT" -> (endpoint: "ticker", params: "symbol=BTCUSDT")
   ```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&str` | Строковый срез — ссылка на часть строки |
| `[start..end]` | Синтаксис среза |
| `get(range)` | Безопасное извлечение среза |
| Байты vs символы | Индексы — это байты, не символы |
| Заимствование | Срезы заимствуют данные, не копируют |

## Навигация

[← День 47](../047-ownership-rules/ru.md) | [День 49 →](../049-string-methods/ru.md)
