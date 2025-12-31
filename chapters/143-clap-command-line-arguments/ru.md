# День 143: Аргументы командной строки: clap

## Аналогия из трейдинга

Представь, что ты управляешь торговым терминалом. Каждый раз, когда открываешь его, ты можешь указать разные настройки: какую биржу использовать, какой тикер отслеживать, какой период для анализа. Вместо того чтобы менять конфиг-файл каждый раз — ты просто пишешь команду:

```bash
./trading-bot --exchange binance --ticker BTCUSDT --period 1h
```

Библиотека `clap` — это как умный парсер твоих команд, который понимает аргументы и проверяет их корректность.

## Подключение clap

Добавляем в `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

## Базовый пример: параметры для анализа цены

```rust
use clap::Parser;

/// Инструмент анализа криптовалют
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Тикер для анализа (например: BTCUSDT)
    #[arg(short, long)]
    ticker: String,

    /// Количество последних свечей для анализа
    #[arg(short, long, default_value_t = 100)]
    count: u32,
}

fn main() {
    let args = Args::parse();

    println!("Анализирую {} за последние {} свечей", args.ticker, args.count);

    // Здесь была бы реальная логика анализа
    analyze_ticker(&args.ticker, args.count);
}

fn analyze_ticker(ticker: &str, count: u32) {
    println!("╔═══════════════════════════════════════╗");
    println!("║       CRYPTO ANALYSIS TOOL            ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Ticker:      {:>24} ║", ticker);
    println!("║ Candles:     {:>24} ║", count);
    println!("╚═══════════════════════════════════════╝");
}
```

Использование:

```bash
./crypto-analyzer --ticker BTCUSDT --count 200
./crypto-analyzer -t ETHUSDT -c 50
./crypto-analyzer --ticker BTCUSDT  # count будет 100 (по умолчанию)
```

## Обязательные и опциональные аргументы

```rust
use clap::Parser;

/// Торговый бот с конфигурацией через CLI
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct TradingConfig {
    /// Биржа для торговли (обязательный параметр)
    #[arg(short, long)]
    exchange: String,

    /// Торговая пара
    #[arg(short, long)]
    pair: String,

    /// Размер позиции в процентах от баланса (по умолчанию 10%)
    #[arg(short, long, default_value_t = 10.0)]
    size: f64,

    /// Включить режим бумажной торговли
    #[arg(long, default_value_t = false)]
    paper: bool,

    /// API ключ (опциональный, можно через env)
    #[arg(long)]
    api_key: Option<String>,
}

fn main() {
    let config = TradingConfig::parse();

    println!("Конфигурация торгового бота:");
    println!("  Биржа: {}", config.exchange);
    println!("  Пара: {}", config.pair);
    println!("  Размер позиции: {}%", config.size);
    println!("  Бумажная торговля: {}", config.paper);

    if let Some(key) = &config.api_key {
        println!("  API ключ: {}...", &key[..8.min(key.len())]);
    } else {
        println!("  API ключ: не указан (используем переменную окружения)");
    }
}
```

## Подкоманды: разные режимы работы бота

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Анализ рынка
    Analyze {
        /// Тикер для анализа
        #[arg(short, long)]
        ticker: String,

        /// Период: 1m, 5m, 15m, 1h, 4h, 1d
        #[arg(short, long, default_value = "1h")]
        period: String,
    },

    /// Торговля
    Trade {
        /// Действие: buy или sell
        #[arg(short, long)]
        action: String,

        /// Тикер
        #[arg(short, long)]
        ticker: String,

        /// Количество
        #[arg(short, long)]
        quantity: f64,

        /// Лимитная цена (опционально)
        #[arg(short, long)]
        price: Option<f64>,
    },

    /// Показать портфель
    Portfolio {
        /// Показать детали по каждой позиции
        #[arg(short, long)]
        detailed: bool,
    },

    /// Показать историю сделок
    History {
        /// Количество последних сделок
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { ticker, period } => {
            println!("Анализирую {} на периоде {}", ticker, period);
            run_analysis(&ticker, &period);
        }
        Commands::Trade { action, ticker, quantity, price } => {
            println!("Торговая операция:");
            println!("  Действие: {}", action);
            println!("  Тикер: {}", ticker);
            println!("  Количество: {}", quantity);
            match price {
                Some(p) => println!("  Тип: Лимитный ордер по цене {}", p),
                None => println!("  Тип: Рыночный ордер"),
            }
        }
        Commands::Portfolio { detailed } => {
            if detailed {
                show_detailed_portfolio();
            } else {
                show_portfolio_summary();
            }
        }
        Commands::History { limit } => {
            println!("Последние {} сделок:", limit);
            show_trade_history(limit);
        }
    }
}

fn run_analysis(ticker: &str, period: &str) {
    println!("╔═══════════════════════════════════════╗");
    println!("║         MARKET ANALYSIS               ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Ticker:    {:>26} ║", ticker);
    println!("║ Period:    {:>26} ║", period);
    println!("║ RSI:       {:>26} ║", "65.4 (Neutral)");
    println!("║ MACD:      {:>26} ║", "Bullish crossover");
    println!("║ Trend:     {:>26} ║", "Uptrend");
    println!("╚═══════════════════════════════════════╝");
}

fn show_portfolio_summary() {
    println!("Портфель: $45,230.50 (+5.2% за день)");
}

fn show_detailed_portfolio() {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║              PORTFOLIO DETAILS                ║");
    println!("╠═══════════════════════════════════════════════╣");
    println!("║ Asset     │ Quantity  │ Value      │ PnL     ║");
    println!("╠═══════════════════════════════════════════════╣");
    println!("║ BTC       │ 0.5       │ $21,500    │ +12.3%  ║");
    println!("║ ETH       │ 5.0       │ $11,250    │ +8.7%   ║");
    println!("║ USDT      │ 12,480    │ $12,480    │ 0.0%    ║");
    println!("╚═══════════════════════════════════════════════╝");
}

fn show_trade_history(limit: u32) {
    let trades = vec![
        ("BTC", "BUY", 0.1, 42000.0, "+$120"),
        ("ETH", "SELL", 2.0, 2300.0, "+$85"),
        ("BTC", "BUY", 0.05, 41500.0, "-$15"),
    ];

    for (i, (ticker, action, qty, price, pnl)) in trades.iter().enumerate() {
        if i >= limit as usize {
            break;
        }
        println!("  {}. {} {} {} @ ${} → {}", i + 1, action, qty, ticker, price, pnl);
    }
}
```

Использование:

```bash
./trading-cli analyze --ticker BTCUSDT --period 4h
./trading-cli trade --action buy --ticker ETHUSDT --quantity 0.5
./trading-cli trade -a sell -t BTCUSDT -q 0.1 --price 45000
./trading-cli portfolio --detailed
./trading-cli history --limit 5
```

## Валидация входных данных

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct OrderArgs {
    /// Тикер торговой пары
    #[arg(short, long)]
    ticker: String,

    /// Количество для покупки/продажи (должно быть положительным)
    #[arg(short, long, value_parser = validate_quantity)]
    quantity: f64,

    /// Цена входа (должна быть положительной)
    #[arg(short, long, value_parser = validate_price)]
    price: f64,

    /// Риск на сделку в процентах (0.1 - 5.0)
    #[arg(short, long, default_value_t = 1.0, value_parser = validate_risk)]
    risk: f64,
}

fn validate_quantity(s: &str) -> Result<f64, String> {
    let qty: f64 = s.parse().map_err(|_| "Количество должно быть числом")?;
    if qty <= 0.0 {
        Err("Количество должно быть положительным".to_string())
    } else {
        Ok(qty)
    }
}

fn validate_price(s: &str) -> Result<f64, String> {
    let price: f64 = s.parse().map_err(|_| "Цена должна быть числом")?;
    if price <= 0.0 {
        Err("Цена должна быть положительной".to_string())
    } else {
        Ok(price)
    }
}

fn validate_risk(s: &str) -> Result<f64, String> {
    let risk: f64 = s.parse().map_err(|_| "Риск должен быть числом")?;
    if risk < 0.1 || risk > 5.0 {
        Err("Риск должен быть от 0.1% до 5.0%".to_string())
    } else {
        Ok(risk)
    }
}

fn main() {
    let args = OrderArgs::parse();

    let position_value = args.quantity * args.price;
    let risk_amount = position_value * (args.risk / 100.0);

    println!("Ордер:");
    println!("  Тикер: {}", args.ticker);
    println!("  Количество: {}", args.quantity);
    println!("  Цена: ${}", args.price);
    println!("  Стоимость позиции: ${:.2}", position_value);
    println!("  Риск: {}% (${:.2})", args.risk, risk_amount);
}
```

## Перечисления как аргументы

```rust
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Exchange {
    Binance,
    Bybit,
    Okx,
    Kraken,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct TradeCommand {
    /// Биржа для торговли
    #[arg(short, long, value_enum)]
    exchange: Exchange,

    /// Тип ордера
    #[arg(short = 't', long, value_enum)]
    order_type: OrderType,

    /// Направление: buy или sell
    #[arg(short, long, value_enum)]
    side: OrderSide,

    /// Торговая пара
    #[arg(short, long)]
    pair: String,

    /// Количество
    #[arg(short, long)]
    quantity: f64,

    /// Цена (обязательна для limit, stop_loss, take_profit)
    #[arg(long)]
    price: Option<f64>,
}

fn main() {
    let cmd = TradeCommand::parse();

    // Проверяем, что цена указана для лимитных ордеров
    match cmd.order_type {
        OrderType::Limit | OrderType::StopLoss | OrderType::TakeProfit => {
            if cmd.price.is_none() {
                eprintln!("Ошибка: для ордера типа {:?} необходимо указать цену (--price)", cmd.order_type);
                std::process::exit(1);
            }
        }
        OrderType::Market => {}
    }

    println!("Создаём ордер:");
    println!("  Биржа: {:?}", cmd.exchange);
    println!("  Тип: {:?}", cmd.order_type);
    println!("  Направление: {:?}", cmd.side);
    println!("  Пара: {}", cmd.pair);
    println!("  Количество: {}", cmd.quantity);

    if let Some(price) = cmd.price {
        println!("  Цена: ${}", price);
    }
}
```

Использование:

```bash
./order --exchange binance --order-type market --side buy --pair BTCUSDT --quantity 0.01
./order -e bybit -t limit -s sell -p ETHUSDT -q 0.5 --price 2500
```

## Комбинирование с переменными окружения

```rust
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct BotConfig {
    /// API ключ (можно также через TRADING_API_KEY)
    #[arg(long, env = "TRADING_API_KEY")]
    api_key: String,

    /// API секрет (можно также через TRADING_API_SECRET)
    #[arg(long, env = "TRADING_API_SECRET")]
    api_secret: String,

    /// Биржа (можно также через TRADING_EXCHANGE)
    #[arg(short, long, env = "TRADING_EXCHANGE", default_value = "binance")]
    exchange: String,

    /// Режим: live или paper
    #[arg(short, long, env = "TRADING_MODE", default_value = "paper")]
    mode: String,
}

fn main() {
    let config = BotConfig::parse();

    println!("Конфигурация бота:");
    println!("  API ключ: {}***", &config.api_key[..4.min(config.api_key.len())]);
    println!("  Биржа: {}", config.exchange);
    println!("  Режим: {}", config.mode);

    if config.mode == "live" {
        println!("\n⚠️  ВНИМАНИЕ: Запущен режим реальной торговли!");
    } else {
        println!("\n📝 Режим бумажной торговли (без реальных денег)");
    }
}
```

Можно запускать:

```bash
# Через аргументы
./bot --api-key abc123 --api-secret xyz789 --exchange bybit --mode live

# Через переменные окружения
export TRADING_API_KEY=abc123
export TRADING_API_SECRET=xyz789
./bot --exchange binance

# Комбинация: env для секретов, аргументы для остального
export TRADING_API_KEY=abc123
export TRADING_API_SECRET=xyz789
./bot --exchange okx --mode paper
```

## Практический пример: полноценный CLI для бэктестинга

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "backtest")]
#[command(author = "Trading Bot Team")]
#[command(version = "1.0")]
#[command(about = "Инструмент для бэктестинга торговых стратегий")]
struct Cli {
    /// Уровень логирования
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Запустить бэктест
    Run {
        /// Файл со стратегией
        #[arg(short, long)]
        strategy: String,

        /// Торговая пара
        #[arg(short, long)]
        pair: String,

        /// Начальная дата (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// Конечная дата (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// Начальный капитал
        #[arg(short, long, default_value_t = 10000.0)]
        capital: f64,

        /// Таймфрейм
        #[arg(short, long, value_enum, default_value_t = Timeframe::H1)]
        timeframe: Timeframe,

        /// Комиссия в процентах
        #[arg(long, default_value_t = 0.1)]
        fee: f64,
    },

    /// Оптимизировать параметры стратегии
    Optimize {
        /// Файл со стратегией
        #[arg(short, long)]
        strategy: String,

        /// Торговая пара
        #[arg(short, long)]
        pair: String,

        /// Количество итераций
        #[arg(short, long, default_value_t = 100)]
        iterations: u32,
    },

    /// Показать доступные стратегии
    List,

    /// Сравнить результаты нескольких бэктестов
    Compare {
        /// ID бэктестов для сравнения
        #[arg(short, long, num_args = 2..)]
        ids: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

fn main() {
    let cli = Cli::parse();

    println!("Уровень логирования: {}", cli.log_level);

    match cli.command {
        Commands::Run {
            strategy,
            pair,
            start,
            end,
            capital,
            timeframe,
            fee,
        } => {
            println!("\n🚀 Запускаем бэктест:");
            println!("╔═══════════════════════════════════════════════╗");
            println!("║            BACKTEST CONFIGURATION             ║");
            println!("╠═══════════════════════════════════════════════╣");
            println!("║ Strategy:    {:>32} ║", strategy);
            println!("║ Pair:        {:>32} ║", pair);
            println!("║ Period:      {:>16} to {:>11} ║", start, end);
            println!("║ Capital:     {:>31}$ ║", capital);
            println!("║ Timeframe:   {:>32?} ║", timeframe);
            println!("║ Fee:         {:>31}% ║", fee);
            println!("╚═══════════════════════════════════════════════╝");

            // Симуляция результата
            println!("\n📊 Результаты:");
            println!("  Всего сделок: 47");
            println!("  Прибыльных: 28 (59.6%)");
            println!("  Чистая прибыль: $2,340.50 (+23.4%)");
            println!("  Макс. просадка: -8.3%");
            println!("  Sharpe Ratio: 1.85");
        }

        Commands::Optimize {
            strategy,
            pair,
            iterations,
        } => {
            println!("\n🔧 Оптимизация стратегии:");
            println!("  Стратегия: {}", strategy);
            println!("  Пара: {}", pair);
            println!("  Итераций: {}", iterations);
            println!("\nЛучшие параметры найдены после {} итераций", iterations);
        }

        Commands::List => {
            println!("\n📋 Доступные стратегии:");
            println!("  1. sma_crossover - Пересечение скользящих средних");
            println!("  2. rsi_oversold - RSI перепроданность");
            println!("  3. macd_divergence - Дивергенция MACD");
            println!("  4. bollinger_bounce - Отскок от Bollinger Bands");
        }

        Commands::Compare { ids } => {
            println!("\n📈 Сравнение бэктестов:");
            for id in &ids {
                println!("  - {}", id);
            }
        }
    }
}
```

## Автодополнение в терминале

`clap` может генерировать скрипты автодополнения:

```rust
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Shell};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Генерировать скрипт автодополнения для shell
    #[arg(long, value_enum)]
    generate_completion: Option<Shell>,

    #[arg(short, long)]
    ticker: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(shell) = args.generate_completion {
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "trading-cli", &mut std::io::stdout());
        return;
    }

    // Основная логика
    if let Some(ticker) = args.ticker {
        println!("Тикер: {}", ticker);
    }
}
```

Добавь в `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
```

## Что мы узнали

| Возможность | Синтаксис | Описание |
|-------------|-----------|----------|
| Простой аргумент | `#[arg(short, long)]` | Флаги -t и --ticker |
| Значение по умолчанию | `default_value_t = 100` | Если не указано |
| Опциональный | `Option<String>` | Может отсутствовать |
| Подкоманды | `#[command(subcommand)]` | analyze, trade, etc. |
| Валидация | `value_parser = fn` | Проверка входных данных |
| Перечисление | `#[arg(value_enum)]` | Выбор из списка |
| Из окружения | `env = "VAR_NAME"` | Переменные окружения |

## Домашнее задание

1. Создай CLI для калькулятора размера позиции с аргументами:
   - `--balance` — текущий баланс
   - `--risk` — риск на сделку в процентах
   - `--entry` — цена входа
   - `--stop-loss` — уровень стоп-лосса

2. Напиши CLI с подкомандами для управления портфелем:
   - `add --ticker --quantity --price` — добавить позицию
   - `remove --ticker` — удалить позицию
   - `show` — показать все позиции
   - `pnl` — рассчитать общий PnL

3. Создай инструмент для загрузки исторических данных:
   - Аргументы: биржа, пара, период, даты
   - Валидация дат (начало < конец)
   - Перечисление для выбора формата вывода (json, csv)

4. Реализуй CLI-обёртку над backtester с:
   - Автодополнением для bash/zsh
   - Конфигурацией через файл и/или аргументы
   - Красивым выводом результатов в таблице

## Навигация

[← Предыдущий день](../142-dotenv-configs/ru.md) | [Следующий день →](../144-logging-env-logger/ru.md)
