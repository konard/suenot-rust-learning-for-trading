# 🦀 Rust для Алготрейдинга: От Нуля до Героя за 365 Дней

> **From Zero to Hero: Learn Rust Through Algorithmic Trading in One Year**

## 🎯 О проекте / About

Это практический курс изучения языка программирования Rust через создание алгоритмических торговых систем. Каждый день — одна тема, одна глава. Всё объясняется простым языком, как для школьника, с аналогиями из мира трейдинга.

This is a practical course for learning Rust programming language through building algorithmic trading systems. One day — one topic, one chapter. Everything explained simply, like for a beginner, with trading-world analogies.

## 🧠 Философия / Philosophy

- **Учимся на реальных задачах** — никакой абстрактной теории без практики
- **Трейдинг как мотивация** — каждый концепт Rust применяется к биржам, ценам, ордерам
- **Объяснения для школьника** — сложное через простое, с понятными аналогиями
- **Двуязычность** — каждая глава на русском и английском

---

## 📚 365 Глав / 365 Chapters

### 🌱 Месяц 1: Первые шаги / Month 1: First Steps (Days 1-31)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 001 | Установка Rust: готовим рабочее место трейдера | Installing Rust: Setting Up Your Trading Workstation |
| 002 | Hello, Trading World! Первая программа | Hello, Trading World! Your First Program |
| 003 | Cargo: менеджер проектов как управляющий портфелем | Cargo: Project Manager Like a Portfolio Manager |
| 004 | Переменные: храним цену актива | Variables: Storing Asset Prices |
| 005 | Неизменяемость: цена сделки зафиксирована | Immutability: The Deal Price is Locked |
| 006 | Типы данных: цена, количество, название тикера | Data Types: Price, Quantity, Ticker Symbol |
| 007 | Целые числа: считаем количество акций | Integers: Counting Shares |
| 008 | Числа с плавающей точкой: точная цена биткоина | Floating Point: Precise Bitcoin Price |
| 009 | Булевы значения: рынок открыт или закрыт? | Booleans: Is the Market Open or Closed? |
| 010 | Символы и строки: тикеры AAPL, BTC, ETH | Characters and Strings: Tickers AAPL, BTC, ETH |
| 011 | Кортежи: цена покупки и продажи вместе | Tuples: Bid and Ask Price Together |
| 012 | Массивы: последние 10 цен закрытия | Arrays: Last 10 Closing Prices |
| 013 | Функции: рассчитываем прибыль сделки | Functions: Calculating Trade Profit |
| 014 | Параметры функций: передаём цену входа и выхода | Function Parameters: Entry and Exit Price |
| 015 | Возврат значений: функция возвращает PnL | Return Values: Function Returns PnL |
| 016 | Комментарии: документируем торговую логику | Comments: Documenting Trading Logic |
| 017 | Условия if: покупать или продавать? | If Conditions: To Buy or To Sell? |
| 018 | else и else if: три сценария рынка | else and else if: Three Market Scenarios |
| 019 | Цикл loop: бесконечный мониторинг цены | loop: Endless Price Monitoring |
| 020 | Цикл while: ждём пока цена достигнет цели | while: Waiting for Target Price |
| 021 | Цикл for: проходим по всем сделкам дня | for: Iterating Through Daily Trades |
| 022 | Range: цены от 100 до 200 | Range: Prices from 100 to 200 |
| 023 | break: выходим когда достигли тейк-профита | break: Exit at Take Profit |
| 024 | continue: пропускаем убыточные сделки в отчёте | continue: Skip Losing Trades in Report |
| 025 | match: определяем тип ордера | match: Determining Order Type |
| 026 | Константы: фиксированная комиссия биржи | Constants: Fixed Exchange Fee |
| 027 | Shadowing: обновляем цену тем же именем | Shadowing: Updating Price with Same Name |
| 028 | Ввод пользователя: трейдер вводит размер позиции | User Input: Trader Enters Position Size |
| 029 | Парсинг строк: преобразуем ввод в число | String Parsing: Converting Input to Number |
| 030 | Простой калькулятор прибыли | Simple Profit Calculator |
| 031 | Проект: Калькулятор размера позиции | Project: Position Size Calculator |

### 🌿 Месяц 2: Владение — сердце Rust / Month 2: Ownership — Heart of Rust (Days 32-59)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 032 | Стек и куча: быстрые и долгосрочные инвестиции | Stack and Heap: Short and Long-term Investments |
| 033 | Владение: кто держит актив | Ownership: Who Holds the Asset |
| 034 | Передача владения: продажа актива | Move: Selling the Asset |
| 035 | Clone: копируем портфель | Clone: Copying the Portfolio |
| 036 | Copy: лёгкие типы как наличные | Copy: Light Types Like Cash |
| 037 | Ссылки: смотрим на чужой портфель | References: Looking at Someone's Portfolio |
| 038 | Заимствование: временный доступ к данным | Borrowing: Temporary Data Access |
| 039 | Изменяемые ссылки: редактируем чужой ордер | Mutable References: Editing Someone's Order |
| 040 | Правило одной изменяемой ссылки | One Mutable Reference Rule |
| 041 | Правило: нельзя смешивать ссылки | Rule: No Mixing References |
| 042 | Dangling References: висячие указатели | Dangling References |
| 043 | Время жизни: сколько живёт ордер | Lifetimes: How Long Does an Order Live |
| 044 | Аннотации времени жизни: 'a в функциях | Lifetime Annotations: 'a in Functions |
| 045 | Lifetime elision: когда Rust сам разберётся | Lifetime Elision: When Rust Figures It Out |
| 046 | Несколько параметров времени жизни | Multiple Lifetime Parameters |
| 047 | 'static: данные живут вечно как история рынка | 'static: Data Lives Forever Like Market History |
| 048 | Слайсы строк: часть названия биржи | String Slices: Part of Exchange Name |
| 049 | Слайсы массивов: часть ценового ряда | Array Slices: Part of Price Series |
| 050 | Практика владения: функция анализа сделки | Ownership Practice: Trade Analysis Function |
| 051 | Практика ссылок: передаём портфель на анализ | Reference Practice: Portfolio Analysis |
| 052 | Практика слайсов: анализ части истории | Slice Practice: Partial History Analysis |
| 053 | Паттерн: данные входят и выходят | Pattern: Data In and Out |
| 054 | Паттерн: заимствуем, не владеем | Pattern: Borrow, Don't Own |
| 055 | Отладка проблем владения | Debugging Ownership Issues |
| 056 | Rc: несколько владельцев одного актива | Rc: Multiple Owners of One Asset |
| 057 | RefCell: изменяем через неизменяемую ссылку | RefCell: Mutate Through Immutable Reference |
| 058 | Weak: слабые ссылки на связанные ордера | Weak: Weak References to Related Orders |
| 059 | Проект: Менеджер торговых позиций | Project: Trading Position Manager |

### 🌳 Месяц 3: Структуры данных / Month 3: Data Structures (Days 60-90)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 060 | Структуры: создаём тип Order | Structs: Creating Order Type |
| 061 | Поля структур: цена, объём, направление | Struct Fields: Price, Volume, Direction |
| 062 | Создание экземпляра: новый ордер | Creating Instance: New Order |
| 063 | Методы: order.execute() | Methods: order.execute() |
| 064 | Ассоциированные функции: Order::new() | Associated Functions: Order::new() |
| 065 | Несколько блоков impl | Multiple impl Blocks |
| 066 | Tuple structs: Price(f64) | Tuple Structs: Price(f64) |
| 067 | Unit-like structs: маркеры состояний | Unit-like Structs: State Markers |
| 068 | Приватность полей: скрываем внутреннее состояние | Field Privacy: Hiding Internal State |
| 069 | Геттеры и сеттеры: контролируем доступ к цене | Getters and Setters: Controlling Price Access |
| 070 | Enum: OrderSide — Buy или Sell | Enum: OrderSide — Buy or Sell |
| 071 | Enum с данными: OrderType с параметрами | Enum with Data: OrderType with Parameters |
| 072 | Option: цена может отсутствовать | Option: Price Might Be Missing |
| 073 | Option методы: unwrap, expect, unwrap_or | Option Methods: unwrap, expect, unwrap_or |
| 074 | Option с map и and_then | Option with map and and_then |
| 075 | Result: операция может провалиться | Result: Operation Can Fail |
| 076 | Result методы: работаем с ошибками | Result Methods: Handling Errors |
| 077 | Оператор ?: пробрасываем ошибку вверх | ? Operator: Propagating Errors Up |
| 078 | Собственные типы ошибок: TradingError | Custom Error Types: TradingError |
| 079 | Vec: динамический список сделок | Vec: Dynamic List of Trades |
| 080 | Vec методы: push, pop, get, len | Vec Methods: push, pop, get, len |
| 081 | Итерация по Vec: проходим все ордера | Iterating Vec: Going Through All Orders |
| 082 | HashMap: портфель актив → количество | HashMap: Portfolio Asset → Quantity |
| 083 | HashMap методы: insert, get, remove | HashMap Methods: insert, get, remove |
| 084 | Entry API: обновляем или вставляем | Entry API: Update or Insert |
| 085 | HashSet: уникальные тикеры в портфеле | HashSet: Unique Tickers in Portfolio |
| 086 | BTreeMap: отсортированные цены | BTreeMap: Sorted Prices |
| 087 | VecDeque: очередь ордеров | VecDeque: Order Queue |
| 088 | BinaryHeap: приоритетная очередь | BinaryHeap: Priority Queue |
| 089 | Комбинируем структуры: Exchange содержит Orders | Combining Structs: Exchange Contains Orders |
| 090 | Проект: Стакан заявок (Order Book) | Project: Order Book |

### 🔧 Месяц 4: Обработка ошибок / Month 4: Error Handling (Days 91-120)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 091 | panic!: что-то пошло совсем не так | panic!: Something Went Very Wrong |
| 092 | Когда паниковать: невосстановимые ошибки | When to Panic: Unrecoverable Errors |
| 093 | RUST_BACKTRACE: расследуем крах | RUST_BACKTRACE: Investigating the Crash |
| 094 | Result глубже: Ok и Err | Result Deeper: Ok and Err |
| 095 | match на Result: обрабатываем оба случая | match on Result: Handling Both Cases |
| 096 | if let с Result: когда нужен только успех | if let with Result: When You Only Need Success |
| 097 | while let: обрабатываем пока успешно | while let: Process While Successful |
| 098 | Цепочки методов Result | Result Method Chaining |
| 099 | map_err: преобразуем тип ошибки | map_err: Transforming Error Type |
| 100 | ok_or: превращаем Option в Result | ok_or: Converting Option to Result |
| 101 | Множественные типы ошибок в функции | Multiple Error Types in Function |
| 102 | Box<dyn Error>: любая ошибка | Box<dyn Error>: Any Error |
| 103 | thiserror: создаём красивые ошибки | thiserror: Creating Beautiful Errors |
| 104 | anyhow: простая обработка ошибок | anyhow: Simple Error Handling |
| 105 | Контекст ошибок: with_context() | Error Context: with_context() |
| 106 | Логирование ошибок при торговле | Logging Trading Errors |
| 107 | Восстановление после ошибок | Recovering from Errors |
| 108 | Retry логика: повторяем неудавшийся запрос | Retry Logic: Repeating Failed Requests |
| 109 | Circuit breaker: защита от каскадных сбоев | Circuit Breaker: Cascade Failure Protection |
| 110 | Graceful degradation: работаем без части данных | Graceful Degradation: Working Without Some Data |
| 111 | Валидация входных данных: проверяем ордер | Input Validation: Checking Order |
| 112 | Паттерн newtype для валидации | Newtype Pattern for Validation |
| 113 | Паттерн builder для сложных структур | Builder Pattern for Complex Structs |
| 114 | Тестирование ошибок: проверяем обработку | Testing Errors: Verifying Handling |
| 115 | Моки ошибок в тестах | Mocking Errors in Tests |
| 116 | Документируем возможные ошибки | Documenting Possible Errors |
| 117 | Ошибки в асинхронном коде (превью) | Errors in Async Code (Preview) |
| 118 | Паттерн: fail fast | Pattern: Fail Fast |
| 119 | Паттерн: ошибка как значение | Pattern: Error as Value |
| 120 | Проект: Робастный API клиент биржи | Project: Robust Exchange API Client |

### 📁 Месяц 5: Работа с данными / Month 5: Working with Data (Days 121-151)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 121 | Чтение файла: загружаем историю цен | Reading File: Loading Price History |
| 122 | Запись в файл: сохраняем сделки | Writing to File: Saving Trades |
| 123 | Построчное чтение: парсим CSV вручную | Line by Line: Parsing CSV Manually |
| 124 | BufReader и BufWriter: эффективный I/O | BufReader and BufWriter: Efficient I/O |
| 125 | std::fs: операции с файлами | std::fs: File Operations |
| 126 | Path и PathBuf: пути к файлам | Path and PathBuf: File Paths |
| 127 | Сериализация: serde введение | Serialization: serde Introduction |
| 128 | JSON: формат биржевых API | JSON: Exchange API Format |
| 129 | serde_json: парсим ответ биржи | serde_json: Parsing Exchange Response |
| 130 | Вложенные структуры JSON | Nested JSON Structures |
| 131 | Опциональные поля в JSON | Optional JSON Fields |
| 132 | #[serde(rename)]: разные имена полей | #[serde(rename)]: Different Field Names |
| 133 | CSV: загружаем исторические данные | CSV: Loading Historical Data |
| 134 | csv crate: читаем OHLCV | csv Crate: Reading OHLCV |
| 135 | Парсинг дат: chrono crate | Date Parsing: chrono Crate |
| 136 | Таймзоны: UTC и локальное время | Timezones: UTC and Local Time |
| 137 | Timestamp: unix время для бирж | Timestamp: Unix Time for Exchanges |
| 138 | Duration: время между сделками | Duration: Time Between Trades |
| 139 | Форматирование времени | Time Formatting |
| 140 | TOML: конфигурация торгового бота | TOML: Trading Bot Configuration |
| 141 | Переменные окружения: секреты API | Environment Variables: API Secrets |
| 142 | dotenv: удобные конфиги разработки | dotenv: Convenient Dev Configs |
| 143 | Аргументы командной строки: clap | Command Line Arguments: clap |
| 144 | Логирование: log и env_logger | Logging: log and env_logger |
| 145 | Уровни логирования для трейдинга | Logging Levels for Trading |
| 146 | Структурированные логи: tracing | Structured Logs: tracing |
| 147 | Ротация логов: логи по дням | Log Rotation: Daily Logs |
| 148 | Сжатие данных: работа с большими файлами | Data Compression: Large Files |
| 149 | Потоковая обработка данных | Streaming Data Processing |
| 150 | Мемоизация: кешируем результаты | Memoization: Caching Results |
| 151 | Проект: Загрузчик исторических данных | Project: Historical Data Loader |

### ⚡ Месяц 6: Многопоточность / Month 6: Concurrency (Days 152-181)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 152 | Потоки: параллельно следим за биржами | Threads: Watching Exchanges in Parallel |
| 153 | std::thread::spawn: запускаем поток | std::thread::spawn: Starting a Thread |
| 154 | join: ждём завершения потока | join: Waiting for Thread Completion |
| 155 | move closures: передаём данные в поток | move Closures: Passing Data to Thread |
| 156 | Каналы mpsc: очередь ордеров | Channels mpsc: Order Queue |
| 157 | Sender и Receiver: кто отправляет, кто получает | Sender and Receiver: Who Sends, Who Receives |
| 158 | Множественные отправители | Multiple Senders |
| 159 | sync_channel: ограниченная очередь | sync_channel: Bounded Queue |
| 160 | Mutex: один трейдер редактирует позицию | Mutex: One Trader Edits Position |
| 161 | Arc: общий доступ между потоками | Arc: Shared Access Between Threads |
| 162 | Arc<Mutex<T>>: общая изменяемая структура | Arc<Mutex<T>>: Shared Mutable Structure |
| 163 | RwLock: много читателей, один писатель | RwLock: Many Readers, One Writer |
| 164 | Deadlock: когда потоки заблокировали друг друга | Deadlock: When Threads Block Each Other |
| 165 | Избегаем deadlock: порядок блокировок | Avoiding Deadlock: Lock Ordering |
| 166 | Atomics: атомарные операции | Atomics: Atomic Operations |
| 167 | AtomicBool: флаг остановки бота | AtomicBool: Bot Stop Flag |
| 168 | AtomicU64: счётчик сделок | AtomicU64: Trade Counter |
| 169 | Ordering: гарантии видимости | Ordering: Visibility Guarantees |
| 170 | crossbeam: продвинутая многопоточность | crossbeam: Advanced Concurrency |
| 171 | crossbeam channels: быстрее mpsc | crossbeam Channels: Faster Than mpsc |
| 172 | crossbeam scope: потоки с заимствованием | crossbeam Scope: Threads with Borrowing |
| 173 | rayon: параллельные итераторы | rayon: Parallel Iterators |
| 174 | par_iter: параллельная обработка сделок | par_iter: Parallel Trade Processing |
| 175 | Пул потоков: ограничиваем параллелизм | Thread Pool: Limiting Parallelism |
| 176 | Отмена операций: graceful shutdown | Operation Cancellation: Graceful Shutdown |
| 177 | Паттерн: producer-consumer | Pattern: Producer-Consumer |
| 178 | Паттерн: fan-out fan-in | Pattern: Fan-out Fan-in |
| 179 | Бенчмарки многопоточности | Concurrency Benchmarks |
| 180 | Профилирование потоков | Thread Profiling |
| 181 | Проект: Мульти-биржевой сборщик данных | Project: Multi-Exchange Data Collector |

### 🌐 Месяц 7: Async и сети / Month 7: Async and Networking (Days 182-212)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 182 | Sync vs Async: блокирующий vs неблокирующий код | Sync vs Async: Blocking vs Non-Blocking |
| 183 | async/await синтаксис | async/await Syntax |
| 184 | Future: обещание результата | Future: Promise of Result |
| 185 | tokio runtime: движок асинхронности | tokio Runtime: Async Engine |
| 186 | #[tokio::main]: входная точка | #[tokio::main]: Entry Point |
| 187 | tokio::spawn: асинхронные задачи | tokio::spawn: Async Tasks |
| 188 | tokio::select!: первый кто ответит | tokio::select!: First to Respond |
| 189 | tokio::join!: ждём всех | tokio::join!: Wait for All |
| 190 | tokio::time: таймеры и задержки | tokio::time: Timers and Delays |
| 191 | tokio::timeout: ограничиваем ожидание | tokio::timeout: Limiting Wait Time |
| 192 | tokio::interval: периодические задачи | tokio::interval: Periodic Tasks |
| 193 | Async mutex: tokio::sync::Mutex | Async Mutex: tokio::sync::Mutex |
| 194 | Async channels: tokio::sync::mpsc | Async Channels: tokio::sync::mpsc |
| 195 | broadcast channel: всем подписчикам | Broadcast Channel: To All Subscribers |
| 196 | watch channel: последнее значение | Watch Channel: Latest Value |
| 197 | HTTP basics: reqwest GET | HTTP Basics: reqwest GET |
| 198 | HTTP POST: отправляем ордер | HTTP POST: Sending Order |
| 199 | HTTP headers: авторизация API | HTTP Headers: API Authorization |
| 200 | HTTP client: connection pooling | HTTP Client: Connection Pooling |
| 201 | Rate limiting: ограничение запросов | Rate Limiting: Request Throttling |
| 202 | Retry с backoff: повторяем запросы | Retry with Backoff: Repeating Requests |
| 203 | WebSocket: потоковые данные | WebSocket: Streaming Data |
| 204 | tokio-tungstenite: WebSocket клиент | tokio-tungstenite: WebSocket Client |
| 205 | Подключение к стриму цен | Connecting to Price Stream |
| 206 | Пинг-понг: поддержание соединения | Ping-Pong: Keeping Connection Alive |
| 207 | Реконнект: восстанавливаем соединение | Reconnect: Restoring Connection |
| 208 | Обработка WebSocket сообщений | Processing WebSocket Messages |
| 209 | Параллельные WebSocket подписки | Parallel WebSocket Subscriptions |
| 210 | Graceful shutdown асинхронных задач | Graceful Shutdown of Async Tasks |
| 211 | Отладка асинхронного кода | Debugging Async Code |
| 212 | Проект: Real-time монитор цен | Project: Real-time Price Monitor |

### 🗄️ Месяц 8: Базы данных / Month 8: Databases (Days 213-243)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 213 | Зачем БД: персистентность данных | Why DB: Data Persistence |
| 214 | SQLite: встроенная база для бота | SQLite: Embedded Database for Bot |
| 215 | rusqlite: подключение к SQLite | rusqlite: Connecting to SQLite |
| 216 | CREATE TABLE: таблица сделок | CREATE TABLE: Trades Table |
| 217 | INSERT: записываем сделку | INSERT: Recording a Trade |
| 218 | SELECT: читаем историю сделок | SELECT: Reading Trade History |
| 219 | UPDATE: обновляем статус ордера | UPDATE: Updating Order Status |
| 220 | DELETE: удаляем отменённый ордер | DELETE: Removing Cancelled Order |
| 221 | Prepared statements: безопасные запросы | Prepared Statements: Safe Queries |
| 222 | Транзакции: атомарные операции | Transactions: Atomic Operations |
| 223 | Индексы: быстрый поиск по дате | Indexes: Fast Search by Date |
| 224 | Миграции: эволюция схемы | Migrations: Schema Evolution |
| 225 | PostgreSQL: продакшн база | PostgreSQL: Production Database |
| 226 | tokio-postgres: асинхронный клиент | tokio-postgres: Async Client |
| 227 | sqlx: компилируемые запросы | sqlx: Compile-time Checked Queries |
| 228 | sqlx миграции | sqlx Migrations |
| 229 | Connection pool: пул соединений | Connection Pool |
| 230 | ORM vs raw SQL | ORM vs Raw SQL |
| 231 | Diesel ORM: введение | Diesel ORM: Introduction |
| 232 | Diesel: модели и схемы | Diesel: Models and Schemas |
| 233 | SeaORM: асинхронный ORM | SeaORM: Async ORM |
| 234 | Redis: кеш и очереди | Redis: Cache and Queues |
| 235 | redis-rs: подключение | redis-rs: Connection |
| 236 | Redis: кешируем последние цены | Redis: Caching Latest Prices |
| 237 | Redis pub/sub: уведомления | Redis Pub/Sub: Notifications |
| 238 | TimescaleDB: временные ряды | TimescaleDB: Time Series |
| 239 | ClickHouse: аналитика больших данных | ClickHouse: Big Data Analytics |
| 240 | Репликация и отказоустойчивость | Replication and Fault Tolerance |
| 241 | Бэкапы: сохраняем историю | Backups: Preserving History |
| 242 | Мониторинг производительности БД | Database Performance Monitoring |
| 243 | Проект: Хранилище торговой истории | Project: Trading History Storage |

### 📊 Месяц 9: Алгоритмы трейдинга / Month 9: Trading Algorithms (Days 244-274)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 244 | Свечи OHLCV: базовая структура | OHLCV Candles: Basic Structure |
| 245 | Расчёт свечей из тиков | Calculating Candles from Ticks |
| 246 | Moving Average: скользящая средняя | Moving Average |
| 247 | SMA: простая скользящая средняя | SMA: Simple Moving Average |
| 248 | EMA: экспоненциальная средняя | EMA: Exponential Moving Average |
| 249 | RSI: индекс относительной силы | RSI: Relative Strength Index |
| 250 | MACD: схождение/расхождение средних | MACD: Moving Average Convergence Divergence |
| 251 | Bollinger Bands: полосы Боллинджера | Bollinger Bands |
| 252 | ATR: средний истинный диапазон | ATR: Average True Range |
| 253 | Stochastic: стохастический осциллятор | Stochastic Oscillator |
| 254 | Volume: анализ объёмов | Volume Analysis |
| 255 | VWAP: средневзвешенная по объёму цена | VWAP: Volume Weighted Average Price |
| 256 | Паттерн: индикатор как trait | Pattern: Indicator as Trait |
| 257 | Комбинирование индикаторов | Combining Indicators |
| 258 | Сигналы: когда покупать/продавать | Signals: When to Buy/Sell |
| 259 | Стратегия: SMA crossover | Strategy: SMA Crossover |
| 260 | Стратегия: Mean reversion | Strategy: Mean Reversion |
| 261 | Стратегия: Momentum | Strategy: Momentum |
| 262 | Стратегия: Breakout | Strategy: Breakout |
| 263 | Риск-менеджмент: размер позиции | Risk Management: Position Sizing |
| 264 | Stop-Loss: ограничение убытков | Stop-Loss: Limiting Losses |
| 265 | Take-Profit: фиксация прибыли | Take-Profit: Locking Profits |
| 266 | Trailing Stop: скользящий стоп | Trailing Stop |
| 267 | Risk/Reward ratio | Risk/Reward Ratio |
| 268 | Kelly Criterion: оптимальный размер ставки | Kelly Criterion: Optimal Bet Size |
| 269 | Портфельная аллокация | Portfolio Allocation |
| 270 | Корреляция активов | Asset Correlation |
| 271 | Volatility: измеряем волатильность | Volatility: Measuring Volatility |
| 272 | Drawdown: просадка портфеля | Drawdown: Portfolio Decline |
| 273 | Sharpe Ratio: доходность с учётом риска | Sharpe Ratio: Risk-Adjusted Returns |
| 274 | Проект: Библиотека индикаторов | Project: Indicators Library |

### 🧪 Месяц 10: Бэктестинг / Month 10: Backtesting (Days 275-304)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 275 | Что такое бэктестинг | What is Backtesting |
| 276 | Event-driven архитектура бэктестера | Event-Driven Backtester Architecture |
| 277 | Загрузка исторических данных | Loading Historical Data |
| 278 | Итерация по свечам | Iterating Through Candles |
| 279 | Эмуляция исполнения ордеров | Order Execution Emulation |
| 280 | Проскальзывание: slippage модель | Slippage Model |
| 281 | Комиссии: учитываем издержки | Commissions: Accounting for Costs |
| 282 | Equity curve: кривая капитала | Equity Curve |
| 283 | Метрики: общая прибыль | Metrics: Total Profit |
| 284 | Метрики: количество сделок | Metrics: Number of Trades |
| 285 | Метрики: win rate | Metrics: Win Rate |
| 286 | Метрики: profit factor | Metrics: Profit Factor |
| 287 | Метрики: максимальная просадка | Metrics: Maximum Drawdown |
| 288 | Генерация отчёта | Report Generation |
| 289 | Визуализация результатов | Results Visualization |
| 290 | Walk-forward анализ | Walk-Forward Analysis |
| 291 | Out-of-sample тестирование | Out-of-Sample Testing |
| 292 | Оптимизация параметров | Parameter Optimization |
| 293 | Grid search: перебор параметров | Grid Search: Parameter Sweep |
| 294 | Overfitting: переобучение стратегии | Overfitting: Strategy Over-Optimization |
| 295 | Cross-validation для стратегий | Cross-Validation for Strategies |
| 296 | Monte Carlo симуляция | Monte Carlo Simulation |
| 297 | Bootstrap: оценка стабильности | Bootstrap: Stability Assessment |
| 298 | Мульти-таймфрейм тестирование | Multi-Timeframe Testing |
| 299 | Мульти-инструмент тестирование | Multi-Instrument Testing |
| 300 | Stress testing: экстремальные условия | Stress Testing: Extreme Conditions |
| 301 | Paper trading: виртуальная торговля | Paper Trading |
| 302 | Сравнение стратегий | Comparing Strategies |
| 303 | Документирование результатов | Documenting Results |
| 304 | Проект: Бэктестинг движок | Project: Backtesting Engine |

### 🚀 Месяц 11: Оптимизация / Month 11: Optimization (Days 305-334)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 305 | Профилирование: где тратится время | Profiling: Where Time is Spent |
| 306 | cargo flamegraph: визуализация производительности | cargo flamegraph: Performance Visualization |
| 307 | Бенчмарки: criterion crate | Benchmarks: criterion Crate |
| 308 | Микрооптимизации: алокации | Micro-Optimizations: Allocations |
| 309 | String vs &str в горячих путях | String vs &str in Hot Paths |
| 310 | Vec с предаллокацией | Vec with Pre-allocation |
| 311 | Итераторы vs циклы: что быстрее | Iterators vs Loops: What's Faster |
| 312 | SIMD: параллельные вычисления | SIMD: Parallel Computations |
| 313 | unsafe: когда оправдано | unsafe: When Justified |
| 314 | FFI: интеграция с C библиотеками | FFI: C Library Integration |
| 315 | Компиляция в release mode | Compiling in Release Mode |
| 316 | LTO: Link Time Optimization | LTO: Link Time Optimization |
| 317 | PGO: Profile Guided Optimization | PGO: Profile Guided Optimization |
| 318 | Размер бинарника: уменьшаем | Binary Size: Reducing |
| 319 | Память: отслеживание утечек | Memory: Tracking Leaks |
| 320 | valgrind и heaptrack | valgrind and heaptrack |
| 321 | Кеширование результатов | Result Caching |
| 322 | Lazy evaluation: ленивые вычисления | Lazy Evaluation |
| 323 | Zero-copy: избегаем копирования | Zero-Copy: Avoiding Copies |
| 324 | Custom allocators: свои аллокаторы | Custom Allocators |
| 325 | jemalloc и mimalloc | jemalloc and mimalloc |
| 326 | Async vs threading: выбор модели | Async vs Threading: Choosing Model |
| 327 | Оптимизация запросов к БД | Database Query Optimization |
| 328 | Оптимизация сетевого кода | Network Code Optimization |
| 329 | Батчинг операций | Operation Batching |
| 330 | Lock-free структуры данных | Lock-Free Data Structures |
| 331 | Latency vs Throughput | Latency vs Throughput |
| 332 | Tail latency: p99 оптимизация | Tail Latency: p99 Optimization |
| 333 | Нагрузочное тестирование | Load Testing |
| 334 | Проект: Высокопроизводительный матчер | Project: High-Performance Matcher |

### 🏭 Месяц 12: Продакшн / Month 12: Production (Days 335-365)

| День | Тема (RU) | Topic (EN) |
|------|-----------|------------|
| 335 | Архитектура торгового бота | Trading Bot Architecture |
| 336 | Конфигурация для разных сред | Environment Configuration |
| 337 | Feature flags: включаем функции | Feature Flags: Enabling Features |
| 338 | Graceful shutdown: корректное завершение | Graceful Shutdown |
| 339 | Health checks: проверка здоровья | Health Checks |
| 340 | Metrics: prometheus метрики | Metrics: Prometheus Metrics |
| 341 | Трейсинг распределённых систем | Distributed Tracing |
| 342 | Алерты: уведомления о проблемах | Alerts: Problem Notifications |
| 343 | Docker: контейнеризация бота | Docker: Bot Containerization |
| 344 | Docker Compose: локальное окружение | Docker Compose: Local Environment |
| 345 | CI/CD: автоматический деплой | CI/CD: Automatic Deployment |
| 346 | GitHub Actions для Rust | GitHub Actions for Rust |
| 347 | Тестирование в CI | Testing in CI |
| 348 | Линтеры: clippy | Linters: clippy |
| 349 | Форматирование: rustfmt | Formatting: rustfmt |
| 350 | Security audit: cargo audit | Security Audit: cargo audit |
| 351 | Документация: rustdoc | Documentation: rustdoc |
| 352 | Публикация на crates.io | Publishing to crates.io |
| 353 | Мониторинг в продакшене | Production Monitoring |
| 354 | Логи в продакшене | Production Logging |
| 355 | Ротация секретов | Secret Rotation |
| 356 | Резервное копирование | Backups |
| 357 | Disaster recovery | Disaster Recovery |
| 358 | Масштабирование горизонтальное | Horizontal Scaling |
| 359 | Kubernetes: оркестрация | Kubernetes: Orchestration |
| 360 | Canary deployments | Canary Deployments |
| 361 | A/B тестирование стратегий | A/B Strategy Testing |
| 362 | Пост-мортем: анализ инцидентов | Post-Mortem: Incident Analysis |
| 363 | Compliance: соответствие требованиям | Compliance |
| 364 | Рефакторинг: эволюция системы | Refactoring: System Evolution |
| 365 | Финал: собираем всё вместе | Final: Putting It All Together |

---

## 📁 Структура проекта / Project Structure

```
rust-learning-for-trading/
├── README.md
├── chapters/
│   ├── 001-installing-rust/
│   │   ├── ru.md (русская версия)
│   │   └── en.md (english version)
│   ├── 002-hello-trading-world/
│   │   ├── ru.md
│   │   └── en.md
│   └── ... (365 глав)
└── examples/
    └── ... (примеры кода)
```

## 🚀 Как использовать / How to Use

1. **Каждый день** — изучай одну главу
2. **Пиши код** — все примеры в главах рабочие
3. **Экспериментируй** — меняй код, смотри что получится
4. **Практикуй** — каждый месяц завершается проектом

## 🤝 Контрибуция / Contributing

Pull requests приветствуются! Если нашёл ошибку или хочешь улучшить объяснение — создай PR.

---

**🦀 Happy Trading & Happy Coding!**

*Изучаем Rust, создавая торговых ботов — от первой программы до продакшн системы.*
