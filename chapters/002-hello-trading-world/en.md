# Day 2: Hello, Trading World! — Your First Program

## Trading Analogy

When a trader first opens a trading terminal, they usually make a test trade — buying the minimum lot to make sure everything works. This isn't for profit, but to understand the mechanics.

Our first program is the same kind of "test trade." We'll write the simplest code to understand how Rust works.

## Anatomy of a Rust Program

Here's the simplest program:

```rust
fn main() {
    println!("Hello, Trading World!");
}
```

Let's break down each part:

### `fn main()`

```rust
fn main() {
```

- `fn` — short for "function"
- `main` — the function name
- `()` — empty parentheses mean "no input parameters"
- `{` — start of function body

**Analogy:** `main` is like the "Start" button in a trading bot. When you run the program, Rust looks for the `main` function and starts executing from there.

### `println!`

```rust
    println!("Hello, Trading World!");
```

- `println!` — is a macro (notice the `!` at the end) that prints text to screen
- Text in quotes — is the string we're printing
- `;` — semicolon ends the statement

**Analogy:** `println!` is like a log in your trading terminal. You see a message about what happened.

### Closing Brace

```rust
}
```

Closes the function body.

## Creating a Project

Instead of manual compilation, let's use Cargo — it's more convenient:

```bash
cargo new trading_hello
cd trading_hello
```

Cargo will create this structure:
```
trading_hello/
├── Cargo.toml    # Project settings
└── src/
    └── main.rs   # Our code
```

Open `src/main.rs` — Hello World is already there!

## Running

```bash
cargo run
```

You'll see:
```
   Compiling trading_hello v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/trading_hello`
Hello, world!
```

## Modifying the Program

Let's make it more "trading-like":

```rust
fn main() {
    println!("=== Trading Bot Started ===");
    println!("Connecting to exchange...");
    println!("Ready to trade!");
    println!("BTC/USDT: $42,000");
}
```

Run again:
```bash
cargo run
```

## Formatted Output

`println!` can insert values:

```rust
fn main() {
    println!("BTC price: {} USD", 42000);
    println!("ETH price: {} USD", 2500);
    println!("Profit: {}%", 15.5);
}
```

`{}` — is a placeholder where the value gets inserted.

**Analogy:** This is like a message template in a bot: "Bought {} BTC at price {} USDT".

## Multiple Placeholders

```rust
fn main() {
    println!("Bought {} {} at price {} USDT", 0.5, "BTC", 42000);
}
```

Output: `Bought 0.5 BTC at price 42000 USDT`

## Named Placeholders

```rust
fn main() {
    println!(
        "{symbol}: bought {amount} at {price}",
        symbol = "BTC/USDT",
        amount = 0.5,
        price = 42000
    );
}
```

This is easier to read!

## Special Characters

```rust
fn main() {
    println!("Line 1\nLine 2");       // \n — newline
    println!("Price:\t42000");         // \t — tab
    println!("He said: \"Buy!\"");     // \" — quotes inside string
}
```

## Practical Example

```rust
fn main() {
    println!("╔════════════════════════════╗");
    println!("║     TRADING BOT v0.1       ║");
    println!("╠════════════════════════════╣");
    println!("║ Balance: {} USDT        ║", 10000);
    println!("║ Open positions: {}         ║", 0);
    println!("║ Daily profit: {}%        ║", 0.0);
    println!("╚════════════════════════════╝");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `fn main()` | Entry point — program starts here |
| `println!` | Print text to screen |
| `{}` | Placeholder for values |
| `;` | End of statement |
| `cargo run` | Compiles and runs the project |

## Homework

1. Create a new project `my_trading_bot` using `cargo new`
2. Write a program that outputs:
   - Your bot's name
   - A list of 3 cryptocurrencies you "trade"
   - Starting balance
3. Use formatted output with `{}`

## Navigation

[← Previous day](../001-installing-rust/en.md) | [Next day →](../003-cargo-project-manager/en.md)
