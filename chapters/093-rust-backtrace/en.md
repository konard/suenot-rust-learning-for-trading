# Day 93: RUST_BACKTRACE — Investigating the Trading System Crash

## Trading Analogy

Imagine: your trading bot suddenly crashes in the middle of the night, and you lose a profitable position. What happened? Without an **investigation log**, you're like a detective without clues — you can only guess.

`RUST_BACKTRACE` is your trading system's **black box**. Just like an airplane's black box records all actions before a crash, backtrace shows the **complete chain of calls** that led to the program's crash.

## What is a Backtrace?

A backtrace (call stack) is a list of all functions that were executing at the moment of program panic, in the order they were called. This allows you to:

1. **Find the exact error location** — which line in which file
2. **Understand the context** — which functions were called before the error
3. **Reconstruct the logic** — how data flowed through the system

## Enabling Backtrace

### Basic Usage

```rust
fn main() {
    // Run with: RUST_BACKTRACE=1 cargo run
    let prices: Vec<f64> = vec![42000.0, 42500.0, 41800.0];

    // This will panic — index out of bounds!
    let price = prices[10];
    println!("Price: {}", price);
}
```

Running without backtrace:
```bash
cargo run
# thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 10'
```

Running with backtrace:
```bash
RUST_BACKTRACE=1 cargo run
# Shows full call stack with line numbers!
```

### Detail Levels

```bash
# Brief backtrace — main functions only
RUST_BACKTRACE=1 cargo run

# Full backtrace — including Rust internal functions
RUST_BACKTRACE=full cargo run
```

## Practical Example: Debugging a Trading Strategy

```rust
fn main() {
    let portfolio = Portfolio::new(10000.0);

    // Trading simulation — there's an error somewhere here
    run_trading_simulation(portfolio);
}

struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) {
        let cost = quantity * price;
        if cost > self.balance {
            panic!("Insufficient balance! Need {} but have {}", cost, self.balance);
        }

        self.balance -= cost;
        self.positions.push(Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
        });
    }

    fn close_position(&mut self, index: usize, exit_price: f64) -> f64 {
        // Potential panic if index >= positions.len()
        let position = self.positions.remove(index);
        let pnl = (exit_price - position.entry_price) * position.quantity;
        self.balance += position.quantity * exit_price;
        pnl
    }
}

fn run_trading_simulation(mut portfolio: Portfolio) {
    let signals = generate_trading_signals();

    for (i, signal) in signals.iter().enumerate() {
        process_signal(&mut portfolio, signal, i);
    }
}

fn generate_trading_signals() -> Vec<TradeSignal> {
    vec![
        TradeSignal { action: "BUY", symbol: "BTC", price: 42000.0, quantity: 0.1 },
        TradeSignal { action: "BUY", symbol: "ETH", price: 2500.0, quantity: 1.0 },
        TradeSignal { action: "SELL", symbol: "BTC", price: 43000.0, quantity: 0.1 },
        TradeSignal { action: "SELL", symbol: "ETH", price: 2600.0, quantity: 1.0 },
        TradeSignal { action: "SELL", symbol: "SOL", price: 100.0, quantity: 5.0 }, // Error! No SOL position
    ]
}

struct TradeSignal {
    action: &'static str,
    symbol: &'static str,
    price: f64,
    quantity: f64,
}

fn process_signal(portfolio: &mut Portfolio, signal: &TradeSignal, _signal_index: usize) {
    match signal.action {
        "BUY" => {
            portfolio.open_position(signal.symbol, signal.quantity, signal.price);
            println!("Opened {} position: {} @ {}", signal.symbol, signal.quantity, signal.price);
        }
        "SELL" => {
            // Find position to close
            let pos_index = find_position_index(portfolio, signal.symbol);
            let pnl = portfolio.close_position(pos_index, signal.price);
            println!("Closed {} position with PnL: ${:.2}", signal.symbol, pnl);
        }
        _ => println!("Unknown signal: {}", signal.action),
    }
}

fn find_position_index(portfolio: &Portfolio, symbol: &str) -> usize {
    // Dangerous! Panics if position not found
    portfolio.positions
        .iter()
        .position(|p| p.symbol == symbol)
        .expect(&format!("Position {} not found!", symbol))
}
```

When running with `RUST_BACKTRACE=1`, you'll see the complete chain:
```
thread 'main' panicked at 'Position SOL not found!'

stack backtrace:
   0: find_position_index
             at ./src/main.rs:95
   1: process_signal
             at ./src/main.rs:82
   2: run_trading_simulation
             at ./src/main.rs:58
   3: main
             at ./src/main.rs:4
```

## Fixing with Result

```rust
fn find_position_index(portfolio: &Portfolio, symbol: &str) -> Result<usize, String> {
    portfolio.positions
        .iter()
        .position(|p| p.symbol == symbol)
        .ok_or_else(|| format!("Position {} not found!", symbol))
}

fn process_signal(portfolio: &mut Portfolio, signal: &TradeSignal, _signal_index: usize) -> Result<(), String> {
    match signal.action {
        "BUY" => {
            portfolio.open_position(signal.symbol, signal.quantity, signal.price)?;
            println!("Opened {} position: {} @ {}", signal.symbol, signal.quantity, signal.price);
        }
        "SELL" => {
            let pos_index = find_position_index(portfolio, signal.symbol)?;
            let pnl = portfolio.close_position(pos_index, signal.price);
            println!("Closed {} position with PnL: ${:.2}", signal.symbol, pnl);
        }
        _ => return Err(format!("Unknown signal: {}", signal.action)),
    }
    Ok(())
}
```

## Programmatic Backtrace Capture

```rust
use std::backtrace::Backtrace;

fn execute_trade(symbol: &str, quantity: f64, price: f64) -> Result<(), TradeError> {
    if price <= 0.0 {
        return Err(TradeError::new(
            "Invalid price",
            format!("Price must be positive, got {}", price),
        ));
    }

    if quantity <= 0.0 {
        return Err(TradeError::new(
            "Invalid quantity",
            format!("Quantity must be positive, got {}", quantity),
        ));
    }

    println!("Executing trade: {} {} @ {}", symbol, quantity, price);
    Ok(())
}

#[derive(Debug)]
struct TradeError {
    kind: String,
    message: String,
    backtrace: Backtrace,
}

impl TradeError {
    fn new(kind: &str, message: String) -> Self {
        TradeError {
            kind: kind.to_string(),
            message,
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}\n\nBacktrace:\n{}", self.kind, self.message, self.backtrace)
    }
}

fn main() {
    // Need to set the variable to capture backtrace
    std::env::set_var("RUST_BACKTRACE", "1");

    match execute_trade("BTC", -0.5, 42000.0) {
        Ok(()) => println!("Trade executed successfully"),
        Err(e) => eprintln!("Trade failed:\n{}", e),
    }
}
```

## Logging Trading System Crashes

```rust
use std::panic;
use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;

fn setup_crash_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::capture();
        let timestamp = chrono_lite_timestamp();

        let crash_report = format!(
            "=== TRADING SYSTEM CRASH REPORT ===\n\
             Timestamp: {}\n\
             Panic: {}\n\
             \n\
             Backtrace:\n{}\n\
             ================================\n\n",
            timestamp,
            panic_info,
            backtrace
        );

        // Write to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash_log.txt")
        {
            let _ = file.write_all(crash_report.as_bytes());
        }

        // Also print to stderr
        eprintln!("{}", crash_report);
    }));
}

fn chrono_lite_timestamp() -> String {
    // Simplified version — use chrono crate in real code
    "2024-01-15 14:30:45 UTC".to_string()
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    setup_crash_handler();

    // Trading logic
    run_trading_bot();
}

fn run_trading_bot() {
    let prices: Vec<f64> = vec![42000.0, 42500.0];

    // Simulate error
    for i in 0..5 {
        let price = prices[i]; // Panics at i=2!
        println!("Processing price: {}", price);
    }
}
```

## Analyzing Backtraces: Practical Tips

### 1. Read Bottom to Top
Backtrace shows the stack from panic location to main. Read bottom to top to understand the call sequence.

### 2. Filter the Noise
Full backtrace contains many Rust internal functions. Look for lines with paths to your code (`src/`).

### 3. Pay Attention to Line Numbers
```
at ./src/trading/strategy.rs:142
```
This is the exact location where the error occurred.

### 4. Use for Prevention
Regularly run tests with `RUST_BACKTRACE=1` to find hidden problems.

## Command Reference

| Command | Description |
|---------|-------------|
| `RUST_BACKTRACE=1` | Brief backtrace |
| `RUST_BACKTRACE=full` | Full backtrace |
| `Backtrace::capture()` | Programmatic capture |
| `panic::set_hook()` | Custom panic handler |

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| Backtrace | Call stack at panic | Debugging bot crashes |
| RUST_BACKTRACE=1 | Environment variable | Quick diagnostics |
| Backtrace::capture() | Programmatic capture | Error logging |
| panic::set_hook() | Custom handler | Crash log recording |

## Practice Exercises

### Exercise 1: Strategy Debugging
Write a trading strategy with an intentional error (division by zero when calculating average). Use backtrace to localize the problem.

### Exercise 2: Crash Reporter
Create a crash logging system that:
- Saves backtrace to a file
- Includes information about recent trades
- Sends a notification (simulation)

### Exercise 3: Defensive Trading
Rewrite panic-prone code using Result while maintaining the ability to get backtrace on errors.

### Exercise 4: Real Crash Analysis
Create a multi-threaded trading simulator that occasionally crashes. Use backtrace to identify the race condition.

## Homework

1. **Crash Logger**: Create a full-featured crash logging system for a trading bot with log rotation and detail levels

2. **Error Context**: Implement an error structure that automatically captures backtrace and context (current position, balance, recent trades)

3. **Debug Mode**: Create a trading bot with debug mode that on error:
   - Saves backtrace
   - Outputs all variable states
   - Creates a snapshot for problem reproduction

4. **Post-mortem Analyzer**: Write a utility that reads crash logs and creates statistics: which functions crash most often, at what time of day, under what conditions

## Navigation

[← Previous day](../092-panic-macro/en.md) | [Next day →](../094-custom-error-types/en.md)
