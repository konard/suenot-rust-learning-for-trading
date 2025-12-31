# День 144: Логирование: log и env_logger

## Аналогия из трейдинга

Представь торговый журнал трейдера. В нём записываются все события: открытие позиций, изменения цен, срабатывания стоп-лоссов, ошибки подключения к бирже. Без такого журнала невозможно понять, почему стратегия потеряла деньги ночью или почему ордер не исполнился.

**Логирование** в программировании — это тот же журнал, только автоматический. Крейт `log` предоставляет уровни важности (от отладки до критических ошибок), а `env_logger` выводит эти записи в консоль с возможностью фильтрации.

## Подключение зависимостей

```toml
# Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

## Базовое использование

```rust
use log::{debug, info, warn, error};

fn main() {
    // Инициализация логгера (один раз в начале программы)
    env_logger::init();

    info!("Торговый бот запущен");
    debug!("Загружаем конфигурацию...");

    let balance = 10000.0;
    info!("Начальный баланс: ${:.2}", balance);

    // Симуляция торговли
    if balance < 1000.0 {
        warn!("Низкий баланс! Рекомендуется пополнение");
    }

    // Симуляция ошибки
    let api_response: Result<f64, &str> = Err("Connection timeout");
    if let Err(e) = api_response {
        error!("Ошибка API: {}", e);
    }

    info!("Торговый бот остановлен");
}
```

**Запуск с разными уровнями:**

```bash
# Показать только ошибки
RUST_LOG=error cargo run

# Показать warnings и выше
RUST_LOG=warn cargo run

# Показать info и выше (рекомендуется для продакшена)
RUST_LOG=info cargo run

# Показать всё, включая debug
RUST_LOG=debug cargo run
```

## Уровни логирования

```rust
use log::{trace, debug, info, warn, error};

fn main() {
    env_logger::init();

    // От самого детального к самому важному:
    trace!("Детали расчёта: step=1, value=0.0001");  // Для глубокой отладки
    debug!("Получены котировки BTC: 42000.0");       // Для разработки
    info!("Ордер размещён: BUY 0.5 BTC @ 42000");    // Обычные операции
    warn!("Спред слишком широкий: 0.5%");            // Предупреждения
    error!("Не удалось подключиться к бирже");       // Ошибки
}
```

| Уровень | Назначение | Пример в трейдинге |
|---------|------------|-------------------|
| `trace` | Мельчайшие детали | Каждый тик котировок |
| `debug` | Отладочная информация | Расчёты индикаторов |
| `info` | Важные события | Открытие/закрытие сделок |
| `warn` | Предупреждения | Высокая волатильность |
| `error` | Ошибки | Отклонённый ордер |

## Логирование торговых операций

```rust
use log::{info, warn, error, debug};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // Симуляция торгового дня
    let trades = simulate_trading_day();

    info!("Торговый день завершён. Всего сделок: {}", trades.len());
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn simulate_trading_day() -> Vec<Trade> {
    let mut trades = Vec::new();

    info!("=== Начало торговой сессии ===");

    // Попытка открытия позиции
    let btc_price = 42000.0;
    let quantity = 0.5;

    debug!("Анализ рынка BTC...");
    debug!("Текущая цена: ${:.2}", btc_price);

    if btc_price > 40000.0 && btc_price < 50000.0 {
        info!("Сигнал на покупку: BTC @ ${:.2}", btc_price);

        let trade = Trade {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity,
            price: btc_price,
        };

        info!("Ордер исполнен: {} {} {} @ ${:.2}",
              trade.side, trade.quantity, trade.symbol, trade.price);
        trades.push(trade);
    }

    // Проверка риска
    let portfolio_value = 10000.0;
    let position_value = btc_price * quantity;
    let risk_percent = (position_value / portfolio_value) * 100.0;

    debug!("Размер позиции: ${:.2} ({:.1}% портфеля)", position_value, risk_percent);

    if risk_percent > 20.0 {
        warn!("Превышен лимит риска! Позиция: {:.1}% > 20%", risk_percent);
    }

    // Симуляция ошибки подключения
    let exchange_status = check_exchange_connection();
    if let Err(e) = exchange_status {
        error!("Потеряно соединение с биржей: {}", e);
    }

    info!("=== Торговая сессия завершена ===");

    trades
}

fn check_exchange_connection() -> Result<(), String> {
    // Симуляция случайной ошибки
    Ok(())
}
```

## Фильтрация по модулям

```rust
use log::{info, debug};

mod order_manager {
    use log::{info, debug, warn};

    pub fn place_order(symbol: &str, qty: f64, price: f64) {
        debug!("Подготовка ордера...");
        info!("Ордер размещён: {} {:.4} @ ${:.2}", symbol, qty, price);
    }

    pub fn cancel_order(order_id: u64) {
        warn!("Отмена ордера #{}", order_id);
    }
}

mod price_feed {
    use log::{debug, trace};

    pub fn on_price_update(symbol: &str, price: f64) {
        trace!("Тик: {} = ${:.2}", symbol, price);
    }

    pub fn calculate_sma(prices: &[f64]) -> f64 {
        debug!("Расчёт SMA для {} значений", prices.len());
        prices.iter().sum::<f64>() / prices.len() as f64
    }
}

fn main() {
    env_logger::init();

    info!("Запуск системы");

    order_manager::place_order("BTC/USDT", 0.5, 42000.0);
    price_feed::on_price_update("BTC/USDT", 42100.0);

    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];
    let sma = price_feed::calculate_sma(&prices);
    info!("SMA: ${:.2}", sma);

    order_manager::cancel_order(12345);
}
```

**Фильтрация по модулям:**

```bash
# Только ордера
RUST_LOG=order_manager=info cargo run

# Price feed в режиме trace, остальное — info
RUST_LOG=info,price_feed=trace cargo run

# Отключить шумный модуль
RUST_LOG=info,price_feed=off cargo run
```

## Настройка формата вывода

```rust
use log::{info, warn};
use std::io::Write;

fn main() {
    // Кастомный формат с временными метками
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    info!("Торговый бот запущен");
    warn!("Тестовое предупреждение");
}
```

**Примечание:** Для использования `chrono` добавьте в `Cargo.toml`:

```toml
[dependencies]
chrono = "0.4"
```

## Структурированное логирование

```rust
use log::{info, warn, error};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // Логирование с контекстом
    log_trade_event("BTC/USDT", "BUY", 0.5, 42000.0, "market");
    log_trade_event("ETH/USDT", "SELL", 2.0, 2200.0, "limit");

    // Логирование ошибки с деталями
    log_order_error(12345, "Insufficient balance", 1000.0, 5000.0);
}

fn log_trade_event(symbol: &str, side: &str, qty: f64, price: f64, order_type: &str) {
    info!(
        "trade_event | symbol={} side={} qty={:.4} price={:.2} type={} value={:.2}",
        symbol, side, qty, price, order_type, qty * price
    );
}

fn log_order_error(order_id: u64, reason: &str, available: f64, required: f64) {
    error!(
        "order_error | order_id={} reason=\"{}\" available={:.2} required={:.2}",
        order_id, reason, available, required
    );
}
```

## Практический пример: торговый бот с логированием

```rust
use log::{info, warn, error, debug, trace};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("=== Trading Bot v1.0 ===");

    let mut bot = TradingBot::new(10000.0);

    // Симуляция рыночных данных
    let price_updates = vec![42000.0, 42100.0, 41950.0, 42200.0, 42150.0];

    for price in price_updates {
        bot.on_price_update("BTC/USDT", price);
    }

    bot.print_summary();
}

struct TradingBot {
    balance: f64,
    position: f64,
    entry_price: Option<f64>,
    trades_count: u32,
    pnl: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        info!("Инициализация бота с балансом: ${:.2}", initial_balance);
        TradingBot {
            balance: initial_balance,
            position: 0.0,
            entry_price: None,
            trades_count: 0,
            pnl: 0.0,
        }
    }

    fn on_price_update(&mut self, symbol: &str, price: f64) {
        trace!("Получена цена: {} = ${:.2}", symbol, price);

        // Простая стратегия: покупаем при падении на 0.1%+
        if let Some(entry) = self.entry_price {
            let change = (price - entry) / entry * 100.0;
            debug!("Изменение позиции: {:.2}%", change);

            if change >= 0.2 {
                self.close_position(symbol, price);
            } else if change <= -0.5 {
                warn!("Стоп-лосс сработал при изменении {:.2}%", change);
                self.close_position(symbol, price);
            }
        } else {
            // Нет открытой позиции — ищем вход
            self.try_open_position(symbol, price);
        }
    }

    fn try_open_position(&mut self, symbol: &str, price: f64) {
        let position_size = self.balance * 0.1; // 10% от баланса
        let quantity = position_size / price;

        if position_size < 100.0 {
            warn!("Недостаточно средств для открытия позиции");
            return;
        }

        debug!("Анализ входа: размер=${:.2}, количество={:.4}", position_size, quantity);

        self.position = quantity;
        self.entry_price = Some(price);
        self.balance -= position_size;
        self.trades_count += 1;

        info!("OPEN {} | qty={:.4} @ ${:.2} | value=${:.2}",
              symbol, quantity, price, position_size);
    }

    fn close_position(&mut self, symbol: &str, price: f64) {
        if let Some(entry) = self.entry_price {
            let exit_value = self.position * price;
            let entry_value = self.position * entry;
            let trade_pnl = exit_value - entry_value;

            self.balance += exit_value;
            self.pnl += trade_pnl;

            let pnl_percent = (trade_pnl / entry_value) * 100.0;

            if trade_pnl >= 0.0 {
                info!("CLOSE {} | qty={:.4} @ ${:.2} | PnL=${:.2} ({:+.2}%)",
                      symbol, self.position, price, trade_pnl, pnl_percent);
            } else {
                warn!("CLOSE {} | qty={:.4} @ ${:.2} | PnL=${:.2} ({:+.2}%)",
                      symbol, self.position, price, trade_pnl, pnl_percent);
            }

            self.position = 0.0;
            self.entry_price = None;
        }
    }

    fn print_summary(&self) {
        info!("=== Итоги торговли ===");
        info!("Сделок: {}", self.trades_count);
        info!("Баланс: ${:.2}", self.balance);
        info!("Общий PnL: ${:.2}", self.pnl);

        if self.pnl >= 0.0 {
            info!("Результат: ПРИБЫЛЬ");
        } else {
            warn!("Результат: УБЫТОК");
        }
    }
}
```

## Логирование в файл

```rust
use log::{info, LevelFilter};
use std::fs::File;
use std::io::Write;

fn main() {
    // Настройка логирования в файл
    let target = Box::new(File::create("trading.log").expect("Can't create file"));

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    info!("Логирование в файл trading.log");
    info!("Это сообщение будет записано в файл");
}
```

## Что мы узнали

| Концепция | Описание | Применение |
|-----------|----------|------------|
| `log` крейт | Фасад для логирования | Макросы уровней |
| `env_logger` | Реализация логгера | Вывод и фильтрация |
| Уровни | trace → debug → info → warn → error | Разная детализация |
| RUST_LOG | Переменная окружения | Настройка фильтрации |
| Модули | Фильтрация по пути | Изоляция шума |

## Практические задания

1. **Журнал ордеров**: Создай систему логирования для книги ордеров с уровнями:
   - `trace`: каждое изменение стакана
   - `debug`: агрегированные обновления
   - `info`: значимые изменения (>1% от объёма)
   - `warn`: аномально большие ордера
   - `error`: несоответствия в данных

2. **Мониторинг риска**: Реализуй модуль контроля рисков с логированием:
   - Превышение лимитов позиции
   - Приближение к margin call
   - Высокая волатильность

3. **Анализ сделок**: Добавь логирование статистики:
   - Винрейт за сессию
   - Средний PnL
   - Максимальная просадка

## Домашнее задание

1. Напиши торговый бот с полным логированием всех операций и возможностью анализа лог-файла после сессии

2. Создай модуль `risk_manager` с отдельными настройками логирования и алертами при превышении лимитов

3. Реализуй парсер лог-файла, который извлекает статистику торгов: общий PnL, количество прибыльных/убыточных сделок, время самой долгой позиции

4. Добавь ротацию логов: новый файл каждый день с именем `trading_YYYY-MM-DD.log`

## Навигация

[← Предыдущий день](../143-clap-command-line/ru.md) | [Следующий день →](../145-logging-levels-trading/ru.md)
