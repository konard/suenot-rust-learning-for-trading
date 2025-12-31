# Day 143: Command Line Arguments: clap

## Trading Analogy

Imagine you're running a trading terminal. Every time you open it, you can specify different settings: which exchange to use, which ticker to track, which period for analysis. Instead of editing a config file every time — you simply type a command:

```bash
./trading-bot --exchange binance --ticker BTCUSDT --period 1h
```

The `clap` library is like a smart parser for your commands that understands arguments and validates their correctness.

## Adding clap

Add to `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

## Basic Example: Price Analysis Parameters

```rust
use clap::Parser;

/// Cryptocurrency analysis tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Ticker to analyze (e.g.: BTCUSDT)
    #[arg(short, long)]
    ticker: String,

    /// Number of recent candles to analyze
    #[arg(short, long, default_value_t = 100)]
    count: u32,
}

fn main() {
    let args = Args::parse();

    println!("Analyzing {} for the last {} candles", args.ticker, args.count);

    // Here would be the actual analysis logic
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

Usage:

```bash
./crypto-analyzer --ticker BTCUSDT --count 200
./crypto-analyzer -t ETHUSDT -c 50
./crypto-analyzer --ticker BTCUSDT  # count defaults to 100
```

## Required and Optional Arguments

```rust
use clap::Parser;

/// Trading bot with CLI configuration
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct TradingConfig {
    /// Exchange to trade on (required)
    #[arg(short, long)]
    exchange: String,

    /// Trading pair
    #[arg(short, long)]
    pair: String,

    /// Position size as percentage of balance (default 10%)
    #[arg(short, long, default_value_t = 10.0)]
    size: f64,

    /// Enable paper trading mode
    #[arg(long, default_value_t = false)]
    paper: bool,

    /// API key (optional, can use env)
    #[arg(long)]
    api_key: Option<String>,
}

fn main() {
    let config = TradingConfig::parse();

    println!("Trading bot configuration:");
    println!("  Exchange: {}", config.exchange);
    println!("  Pair: {}", config.pair);
    println!("  Position size: {}%", config.size);
    println!("  Paper trading: {}", config.paper);

    if let Some(key) = &config.api_key {
        println!("  API key: {}...", &key[..8.min(key.len())]);
    } else {
        println!("  API key: not provided (using environment variable)");
    }
}
```

## Subcommands: Different Bot Modes

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
    /// Analyze the market
    Analyze {
        /// Ticker to analyze
        #[arg(short, long)]
        ticker: String,

        /// Period: 1m, 5m, 15m, 1h, 4h, 1d
        #[arg(short, long, default_value = "1h")]
        period: String,
    },

    /// Execute a trade
    Trade {
        /// Action: buy or sell
        #[arg(short, long)]
        action: String,

        /// Ticker
        #[arg(short, long)]
        ticker: String,

        /// Quantity
        #[arg(short, long)]
        quantity: f64,

        /// Limit price (optional)
        #[arg(short, long)]
        price: Option<f64>,
    },

    /// Show portfolio
    Portfolio {
        /// Show details for each position
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show trade history
    History {
        /// Number of recent trades
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { ticker, period } => {
            println!("Analyzing {} on {} timeframe", ticker, period);
            run_analysis(&ticker, &period);
        }
        Commands::Trade { action, ticker, quantity, price } => {
            println!("Trade operation:");
            println!("  Action: {}", action);
            println!("  Ticker: {}", ticker);
            println!("  Quantity: {}", quantity);
            match price {
                Some(p) => println!("  Type: Limit order at ${}", p),
                None => println!("  Type: Market order"),
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
            println!("Last {} trades:", limit);
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
    println!("Portfolio: $45,230.50 (+5.2% today)");
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

Usage:

```bash
./trading-cli analyze --ticker BTCUSDT --period 4h
./trading-cli trade --action buy --ticker ETHUSDT --quantity 0.5
./trading-cli trade -a sell -t BTCUSDT -q 0.1 --price 45000
./trading-cli portfolio --detailed
./trading-cli history --limit 5
```

## Input Validation

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct OrderArgs {
    /// Trading pair ticker
    #[arg(short, long)]
    ticker: String,

    /// Quantity to buy/sell (must be positive)
    #[arg(short, long, value_parser = validate_quantity)]
    quantity: f64,

    /// Entry price (must be positive)
    #[arg(short, long, value_parser = validate_price)]
    price: f64,

    /// Risk per trade in percent (0.1 - 5.0)
    #[arg(short, long, default_value_t = 1.0, value_parser = validate_risk)]
    risk: f64,
}

fn validate_quantity(s: &str) -> Result<f64, String> {
    let qty: f64 = s.parse().map_err(|_| "Quantity must be a number")?;
    if qty <= 0.0 {
        Err("Quantity must be positive".to_string())
    } else {
        Ok(qty)
    }
}

fn validate_price(s: &str) -> Result<f64, String> {
    let price: f64 = s.parse().map_err(|_| "Price must be a number")?;
    if price <= 0.0 {
        Err("Price must be positive".to_string())
    } else {
        Ok(price)
    }
}

fn validate_risk(s: &str) -> Result<f64, String> {
    let risk: f64 = s.parse().map_err(|_| "Risk must be a number")?;
    if risk < 0.1 || risk > 5.0 {
        Err("Risk must be between 0.1% and 5.0%".to_string())
    } else {
        Ok(risk)
    }
}

fn main() {
    let args = OrderArgs::parse();

    let position_value = args.quantity * args.price;
    let risk_amount = position_value * (args.risk / 100.0);

    println!("Order:");
    println!("  Ticker: {}", args.ticker);
    println!("  Quantity: {}", args.quantity);
    println!("  Price: ${}", args.price);
    println!("  Position value: ${:.2}", position_value);
    println!("  Risk: {}% (${:.2})", args.risk, risk_amount);
}
```

## Enums as Arguments

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
    /// Exchange to trade on
    #[arg(short, long, value_enum)]
    exchange: Exchange,

    /// Order type
    #[arg(short = 't', long, value_enum)]
    order_type: OrderType,

    /// Direction: buy or sell
    #[arg(short, long, value_enum)]
    side: OrderSide,

    /// Trading pair
    #[arg(short, long)]
    pair: String,

    /// Quantity
    #[arg(short, long)]
    quantity: f64,

    /// Price (required for limit, stop_loss, take_profit)
    #[arg(long)]
    price: Option<f64>,
}

fn main() {
    let cmd = TradeCommand::parse();

    // Check that price is specified for limit orders
    match cmd.order_type {
        OrderType::Limit | OrderType::StopLoss | OrderType::TakeProfit => {
            if cmd.price.is_none() {
                eprintln!("Error: {:?} order type requires a price (--price)", cmd.order_type);
                std::process::exit(1);
            }
        }
        OrderType::Market => {}
    }

    println!("Creating order:");
    println!("  Exchange: {:?}", cmd.exchange);
    println!("  Type: {:?}", cmd.order_type);
    println!("  Side: {:?}", cmd.side);
    println!("  Pair: {}", cmd.pair);
    println!("  Quantity: {}", cmd.quantity);

    if let Some(price) = cmd.price {
        println!("  Price: ${}", price);
    }
}
```

Usage:

```bash
./order --exchange binance --order-type market --side buy --pair BTCUSDT --quantity 0.01
./order -e bybit -t limit -s sell -p ETHUSDT -q 0.5 --price 2500
```

## Combining with Environment Variables

```rust
use clap::Parser;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct BotConfig {
    /// API key (can also use TRADING_API_KEY)
    #[arg(long, env = "TRADING_API_KEY")]
    api_key: String,

    /// API secret (can also use TRADING_API_SECRET)
    #[arg(long, env = "TRADING_API_SECRET")]
    api_secret: String,

    /// Exchange (can also use TRADING_EXCHANGE)
    #[arg(short, long, env = "TRADING_EXCHANGE", default_value = "binance")]
    exchange: String,

    /// Mode: live or paper
    #[arg(short, long, env = "TRADING_MODE", default_value = "paper")]
    mode: String,
}

fn main() {
    let config = BotConfig::parse();

    println!("Bot configuration:");
    println!("  API key: {}***", &config.api_key[..4.min(config.api_key.len())]);
    println!("  Exchange: {}", config.exchange);
    println!("  Mode: {}", config.mode);

    if config.mode == "live" {
        println!("\n⚠️  WARNING: Live trading mode enabled!");
    } else {
        println!("\n📝 Paper trading mode (no real money)");
    }
}
```

You can run:

```bash
# Via arguments
./bot --api-key abc123 --api-secret xyz789 --exchange bybit --mode live

# Via environment variables
export TRADING_API_KEY=abc123
export TRADING_API_SECRET=xyz789
./bot --exchange binance

# Combination: env for secrets, args for the rest
export TRADING_API_KEY=abc123
export TRADING_API_SECRET=xyz789
./bot --exchange okx --mode paper
```

## Practical Example: Full-Featured Backtesting CLI

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "backtest")]
#[command(author = "Trading Bot Team")]
#[command(version = "1.0")]
#[command(about = "Tool for backtesting trading strategies")]
struct Cli {
    /// Logging level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a backtest
    Run {
        /// Strategy file
        #[arg(short, long)]
        strategy: String,

        /// Trading pair
        #[arg(short, long)]
        pair: String,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// Initial capital
        #[arg(short, long, default_value_t = 10000.0)]
        capital: f64,

        /// Timeframe
        #[arg(short, long, value_enum, default_value_t = Timeframe::H1)]
        timeframe: Timeframe,

        /// Fee percentage
        #[arg(long, default_value_t = 0.1)]
        fee: f64,
    },

    /// Optimize strategy parameters
    Optimize {
        /// Strategy file
        #[arg(short, long)]
        strategy: String,

        /// Trading pair
        #[arg(short, long)]
        pair: String,

        /// Number of iterations
        #[arg(short, long, default_value_t = 100)]
        iterations: u32,
    },

    /// Show available strategies
    List,

    /// Compare results of multiple backtests
    Compare {
        /// Backtest IDs to compare
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

    println!("Log level: {}", cli.log_level);

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
            println!("\n🚀 Running backtest:");
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

            // Simulated results
            println!("\n📊 Results:");
            println!("  Total trades: 47");
            println!("  Profitable: 28 (59.6%)");
            println!("  Net profit: $2,340.50 (+23.4%)");
            println!("  Max drawdown: -8.3%");
            println!("  Sharpe Ratio: 1.85");
        }

        Commands::Optimize {
            strategy,
            pair,
            iterations,
        } => {
            println!("\n🔧 Optimizing strategy:");
            println!("  Strategy: {}", strategy);
            println!("  Pair: {}", pair);
            println!("  Iterations: {}", iterations);
            println!("\nBest parameters found after {} iterations", iterations);
        }

        Commands::List => {
            println!("\n📋 Available strategies:");
            println!("  1. sma_crossover - Moving Average Crossover");
            println!("  2. rsi_oversold - RSI Oversold");
            println!("  3. macd_divergence - MACD Divergence");
            println!("  4. bollinger_bounce - Bollinger Bands Bounce");
        }

        Commands::Compare { ids } => {
            println!("\n📈 Comparing backtests:");
            for id in &ids {
                println!("  - {}", id);
            }
        }
    }
}
```

## Shell Autocompletion

`clap` can generate autocompletion scripts:

```rust
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Shell};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Generate completion script for shell
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

    // Main logic
    if let Some(ticker) = args.ticker {
        println!("Ticker: {}", ticker);
    }
}
```

Add to `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
```

## What We Learned

| Feature | Syntax | Description |
|---------|--------|-------------|
| Simple argument | `#[arg(short, long)]` | Flags -t and --ticker |
| Default value | `default_value_t = 100` | If not specified |
| Optional | `Option<String>` | Can be absent |
| Subcommands | `#[command(subcommand)]` | analyze, trade, etc. |
| Validation | `value_parser = fn` | Input validation |
| Enum | `#[arg(value_enum)]` | Choice from list |
| From environment | `env = "VAR_NAME"` | Environment variables |

## Homework

1. Create a CLI for a position size calculator with arguments:
   - `--balance` — current balance
   - `--risk` — risk per trade in percent
   - `--entry` — entry price
   - `--stop-loss` — stop loss level

2. Write a CLI with subcommands for portfolio management:
   - `add --ticker --quantity --price` — add position
   - `remove --ticker` — remove position
   - `show` — show all positions
   - `pnl` — calculate total PnL

3. Create a tool for downloading historical data:
   - Arguments: exchange, pair, period, dates
   - Date validation (start < end)
   - Enum for output format selection (json, csv)

4. Implement a CLI wrapper for a backtester with:
   - Autocompletion for bash/zsh
   - Configuration via file and/or arguments
   - Beautiful table output for results

## Navigation

[← Previous day](../142-dotenv-configs/en.md) | [Next day →](../144-logging-env-logger/en.md)
