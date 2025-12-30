# Day 3: Cargo — Project Manager Like a Portfolio Manager

## Trading Analogy

Imagine a portfolio manager:
- They **create** portfolios for clients
- **Add** assets (stocks, bonds, crypto)
- **Manage** dependencies between assets
- **Monitor** to ensure everything works properly
- **Optimize** the portfolio for best results

Cargo does the same thing for your Rust projects!

## What Cargo Can Do

| Command | What it does | Analogy |
|---------|-------------|---------|
| `cargo new` | Creates a project | Open a new portfolio |
| `cargo build` | Compiles code | Assemble assets |
| `cargo run` | Runs the program | Start trading |
| `cargo test` | Runs tests | Test the strategy |
| `cargo add` | Adds a library | Add asset to portfolio |

## Creating a Project

```bash
cargo new trading_bot
```

This creates the structure:
```
trading_bot/
├── Cargo.toml      # Project passport
├── Cargo.lock      # Exact dependency versions
└── src/
    └── main.rs     # Main code file
```

## Cargo.toml — Project Passport

```toml
[package]
name = "trading_bot"
version = "0.1.0"
edition = "2021"

[dependencies]
```

- `name` — project name (like fund name)
- `version` — version (like quarterly report: v1, v2...)
- `edition` — Rust version (2021 is the latest stable)
- `[dependencies]` — list of external libraries

## Adding Dependencies

Let's say we need a library for working with time:

```bash
cargo add chrono
```

Cargo.toml now looks like:
```toml
[dependencies]
chrono = "0.4"
```

**Analogy:** This is like adding an ETF to your portfolio. `chrono` is a "package" of functions for working with dates that someone already wrote for us.

## Popular Libraries for Trading

```toml
[dependencies]
# Time handling
chrono = "0.4"

# HTTP requests to API
reqwest = { version = "0.11", features = ["json"] }

# JSON handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Decimal numbers (precise prices)
rust_decimal = "1.33"
```

## Building the Project

### Development mode (fast compilation)
```bash
cargo build
```

Result in `target/debug/trading_bot`

### Release mode (optimized code)
```bash
cargo build --release
```

Result in `target/release/trading_bot`

**Analogy:**
- `debug` — demo account where we test
- `release` — live account with real money

## Running

```bash
cargo run              # debug mode
cargo run --release    # release mode (runs faster)
```

## Useful Commands

```bash
cargo check     # Quick error check (no build)
cargo clean     # Delete built files
cargo update    # Update dependencies
cargo doc       # Generate documentation
```

## Cargo.lock — Version Pinning

This file is created automatically and contains **exact** versions of all libraries.

**Analogy:** This is like a portfolio snapshot on a specific date. If chrono updates from 0.4.31 to 0.4.32, your project will still use 0.4.31 until you update.

Important: add `Cargo.lock` to git for executable programs (bots), but not for libraries.

## Workspaces

When a project grows, you can split it into parts:

```
trading_system/
├── Cargo.toml           # Main file
├── bot/                  # Trading bot
│   ├── Cargo.toml
│   └── src/
├── indicators/           # Indicators library
│   ├── Cargo.toml
│   └── src/
└── backtest/            # Backtesting
    ├── Cargo.toml
    └── src/
```

Main `Cargo.toml`:
```toml
[workspace]
members = ["bot", "indicators", "backtest"]
```

**Analogy:** This is like splitting a fund into multiple strategies that can be developed independently.

## Practical Example

Let's create a trading bot project:

```bash
cargo new my_trading_bot
cd my_trading_bot
cargo add chrono serde serde_json
```

Update `src/main.rs`:

```rust
use chrono::Utc;

fn main() {
    let now = Utc::now();
    println!("Trading bot started!");
    println!("Current time (UTC): {}", now);
    println!("Version: 0.1.0");
}
```

Run:
```bash
cargo run
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Cargo.toml | Project configuration |
| dependencies | External libraries |
| cargo build | Compilation |
| cargo run | Execution |
| --release | Optimized build |

## Homework

1. Create a project `price_tracker`
2. Add dependencies: `chrono` and `rust_decimal`
3. Write a program that outputs current time and BTC "price" (just a number for now)
4. Build the project in release mode
5. Compare file sizes in `target/debug` and `target/release`

## Navigation

[← Previous day](../002-hello-trading-world/en.md) | [Next day →](../004-variables-asset-prices/en.md)
