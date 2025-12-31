# День 215: rusqlite: подключение к SQLite

## Аналогия из трейдинга

Представь, что ты трейдер, который ведёт журнал всех своих сделок в блокноте. Каждый раз, когда нужно найти конкретную сделку или посчитать статистику, приходится перелистывать все страницы. А если блокнот потеряется — все данные пропадут навсегда.

**SQLite** — это как умный электронный журнал, который:
- Хранит все твои сделки в одном файле
- Мгновенно находит нужную информацию по любому критерию
- Автоматически создаёт резервные копии
- Работает без отдельного сервера (встроенная база данных)

**rusqlite** — это мост между Rust и SQLite. Он позволяет твоему торговому боту:
- Сохранять историю сделок для анализа
- Хранить настройки стратегий
- Кешировать рыночные данные
- Вести журнал ордеров

## Что такое rusqlite?

`rusqlite` — это Rust-обёртка над библиотекой SQLite. Она предоставляет:

| Возможность | Описание |
|-------------|----------|
| Типобезопасность | Rust-типы при работе с SQL |
| Безопасность памяти | Автоматическое управление ресурсами |
| Параметризованные запросы | Защита от SQL-инъекций |
| Транзакции | Атомарные операции с данными |

## Добавление rusqlite в проект

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
```

Флаг `bundled` включает SQLite прямо в ваш бинарник — не нужно устанавливать SQLite отдельно.

## Создание подключения

### Базовое подключение к файловой базе

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Создаём или открываем базу данных
    let conn = Connection::open("trading_bot.db")?;

    println!("База данных успешно открыта!");

    // Connection автоматически закроется при выходе из области видимости
    Ok(())
}
```

### База данных в памяти (для тестирования)

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // База в памяти — данные исчезнут при завершении программы
    let conn = Connection::open_in_memory()?;

    println!("In-memory база создана для тестов");

    Ok(())
}
```

## Практический пример: Подключение торгового бота

```rust
use rusqlite::{Connection, Result};
use std::path::Path;

/// Менеджер базы данных для торгового бота
struct TradingDatabase {
    conn: Connection,
}

impl TradingDatabase {
    /// Создаёт новое подключение к базе данных
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Включаем режим WAL для лучшей производительности
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Включаем проверку внешних ключей
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        Ok(TradingDatabase { conn })
    }

    /// Создаёт in-memory базу для тестирования
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(TradingDatabase { conn })
    }

    /// Проверяет, что подключение активно
    fn is_connected(&self) -> bool {
        self.conn.execute_batch("SELECT 1").is_ok()
    }
}

fn main() -> Result<()> {
    // Создаём базу данных для торгового бота
    let db = TradingDatabase::new("my_trading_bot.db")?;

    if db.is_connected() {
        println!("Торговый бот успешно подключен к базе данных!");
    }

    // Тестовая база в памяти
    let test_db = TradingDatabase::new_in_memory()?;
    println!("Тестовая база создана: {}", test_db.is_connected());

    Ok(())
}
```

## Режимы открытия базы данных

```rust
use rusqlite::{Connection, OpenFlags, Result};

fn main() -> Result<()> {
    // Только для чтения — безопасно для анализа данных
    let read_only = Connection::open_with_flags(
        "trading_data.db",
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    println!("Открыта база только для чтения");

    // Чтение и запись (по умолчанию)
    let read_write = Connection::open_with_flags(
        "trading_data.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE,
    )?;
    println!("Открыта база для чтения и записи");

    // Создать, если не существует
    let create_new = Connection::open_with_flags(
        "new_trading.db",
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    println!("База создана или открыта");

    Ok(())
}
```

## Обработка ошибок подключения

```rust
use rusqlite::{Connection, Error, Result};
use std::path::Path;

/// Результат проверки базы данных
enum DatabaseStatus {
    Ready,
    Created,
    Error(String),
}

fn check_database(path: &str) -> DatabaseStatus {
    // Проверяем, существует ли файл
    let exists = Path::new(path).exists();

    match Connection::open(path) {
        Ok(conn) => {
            // Проверяем целостность базы
            match conn.execute_batch("PRAGMA integrity_check;") {
                Ok(_) => {
                    if exists {
                        DatabaseStatus::Ready
                    } else {
                        DatabaseStatus::Created
                    }
                }
                Err(e) => DatabaseStatus::Error(format!("Повреждена база: {}", e)),
            }
        }
        Err(e) => DatabaseStatus::Error(format!("Не удалось открыть: {}", e)),
    }
}

fn main() {
    let paths = vec![
        "trading_bot.db",
        "/invalid/path/database.db",
        ":memory:",
    ];

    for path in paths {
        print!("Проверка '{}': ", path);
        match check_database(path) {
            DatabaseStatus::Ready => println!("Готова к работе"),
            DatabaseStatus::Created => println!("Создана новая база"),
            DatabaseStatus::Error(e) => println!("Ошибка: {}", e),
        }
    }
}
```

## Конфигурация для торгового бота

```rust
use rusqlite::{Connection, Result};

/// Настройки базы данных для высокопроизводительного бота
fn configure_for_trading(conn: &Connection) -> Result<()> {
    // WAL режим — позволяет читать во время записи
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    // Синхронизация — баланс между скоростью и надёжностью
    // NORMAL — хороший компромисс для торговли
    conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

    // Кеш в памяти — ускоряет частые запросы
    conn.execute_batch("PRAGMA cache_size = -64000;")?; // 64MB

    // Размер страницы — оптимально для SSD
    conn.execute_batch("PRAGMA page_size = 4096;")?;

    // Временные таблицы в памяти
    conn.execute_batch("PRAGMA temp_store = MEMORY;")?;

    println!("База данных оптимизирована для торговли");

    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("fast_trading.db")?;
    configure_for_trading(&conn)?;

    Ok(())
}
```

## Пул подключений (для многопоточного бота)

Для многопоточных приложений используйте `r2d2` с `rusqlite`:

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
```

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::thread;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаём менеджер подключений
    let manager = SqliteConnectionManager::file("trading_pool.db");

    // Создаём пул с 4 подключениями
    let pool = Arc::new(Pool::builder()
        .max_size(4)
        .build(manager)?);

    println!("Пул подключений создан");

    // Симулируем многопоточную работу
    let mut handles = vec![];

    for i in 0..4 {
        let pool = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            let conn = pool.get().expect("Не удалось получить подключение");
            println!("Поток {} получил подключение", i);

            // Симулируем работу с базой
            conn.execute_batch("SELECT 1").expect("Ошибка запроса");

            println!("Поток {} завершил работу", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Все потоки завершены");

    Ok(())
}
```

## Структура торгового приложения с базой данных

```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

/// Торговый бот с подключением к базе данных
struct TradingBot {
    name: String,
    db: Arc<Mutex<Connection>>,
}

impl TradingBot {
    fn new(name: &str, db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Базовая конфигурация
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        Ok(TradingBot {
            name: name.to_string(),
            db: Arc::new(Mutex::new(conn)),
        })
    }

    fn log_message(&self, message: &str) {
        println!("[{}] {}", self.name, message);
    }

    fn check_db_connection(&self) -> bool {
        match self.db.lock() {
            Ok(conn) => conn.execute_batch("SELECT 1").is_ok(),
            Err(_) => false,
        }
    }
}

fn main() -> Result<()> {
    let bot = TradingBot::new("CryptoTrader", "crypto_trades.db")?;

    bot.log_message("Бот запущен");

    if bot.check_db_connection() {
        bot.log_message("Подключение к базе данных активно");
    } else {
        bot.log_message("ОШИБКА: База данных недоступна!");
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `rusqlite` | Rust-обёртка для работы с SQLite |
| `Connection::open()` | Открытие файловой базы данных |
| `Connection::open_in_memory()` | База в памяти для тестов |
| `OpenFlags` | Флаги режима открытия |
| PRAGMA | Настройки производительности SQLite |
| WAL режим | Улучшенный режим журналирования |
| Пул подключений | r2d2 для многопоточных приложений |

## Практические задания

1. **Простое подключение**: Создай программу, которая:
   - Открывает базу данных `my_trades.db`
   - Проверяет, что подключение работает
   - Выводит версию SQLite (`SELECT sqlite_version()`)

2. **Обработка ошибок**: Напиши функцию, которая:
   - Пытается открыть базу по указанному пути
   - Возвращает понятное сообщение об ошибке
   - Обрабатывает случаи: файл не существует, нет прав доступа, повреждённая база

3. **Конфигуратор базы**: Создай структуру `DatabaseConfig` с методами:
   - `with_wal_mode()` — включает WAL
   - `with_cache_size(mb: u32)` — устанавливает размер кеша
   - `apply(conn: &Connection)` — применяет настройки

4. **Многопоточный тест**: Используя пул подключений:
   - Создай 4 потока
   - Каждый поток выполняет 100 запросов `SELECT 1`
   - Измерь общее время выполнения

## Домашнее задание

1. **Менеджер баз данных**: Реализуй структуру `DatabaseManager`, которая:
   - Хранит несколько подключений к разным базам (trades, analytics, logs)
   - Предоставляет метод `get_connection(db_name: &str)` для получения нужного подключения
   - Автоматически создаёт базы, если они не существуют

2. **Мониторинг подключения**: Создай компонент, который:
   - Периодически проверяет состояние подключения (каждые 5 секунд)
   - Автоматически переподключается при потере связи
   - Логирует все события (подключение, отключение, переподключение)

3. **Бенчмарк подключений**: Напиши тест производительности, который:
   - Сравнивает скорость работы с WAL и без него
   - Измеряет влияние размера кеша на производительность
   - Выводит таблицу с результатами

4. **Миграция данных**: Реализуй функцию, которая:
   - Открывает две базы данных (старую и новую)
   - Копирует все данные из одной в другую
   - Проверяет целостность после копирования

## Навигация

[← Предыдущий день](../214-sqlite-embedded-database/ru.md) | [Следующий день →](../216-create-table-trades/ru.md)
