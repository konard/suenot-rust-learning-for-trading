# Day 1: Installing Rust — Setting Up Your Trading Workstation

## Trading Analogy

Imagine you want to start trading on an exchange. First, you need to:
1. Open a brokerage account
2. Install a trading terminal
3. Set up your workspace

Similarly, to start programming in Rust, you need to:
1. Install the Rust compiler
2. Get the Cargo build tool
3. Set up your code editor

**Rust is like your trading terminal**, except instead of buying stocks, you'll be creating programs that trade for you!

## What is Rust?

Rust is a programming language that is:
- **Fast** like C++ (crucial for trading where milliseconds matter)
- **Safe** — prevents dangerous mistakes (like risk management in trading)
- **Modern** — convenient syntax and excellent tools

## Installation

### On Linux and macOS

Open terminal and run one command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This command downloads and installs:
- **rustc** — the compiler (translates your code into a program)
- **cargo** — project and dependency manager
- **rustup** — Rust update tool

### On Windows

1. Download installer from [rustup.rs](https://rustup.rs)
2. Run and follow instructions
3. You may need to install Visual Studio Build Tools

### Verify Installation

After installation, open a **new** terminal and check:

```bash
rustc --version
```

You should see something like:
```
rustc 1.75.0 (82e1608df 2023-12-21)
```

Also check Cargo:
```bash
cargo --version
```

Result:
```
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

## Code Editor

For writing code, I recommend **VS Code** with **rust-analyzer** extension:

1. Download VS Code: https://code.visualstudio.com
2. Install "rust-analyzer" extension (find in Extensions)

Rust-analyzer is like an analyst looking over your shoulder. It:
- Shows errors while you type
- Displays variable types
- Helps with autocomplete

## First Test

Create a file `hello.rs` and write:

```rust
fn main() {
    println!("Ready to trade!");
}
```

Compile and run:

```bash
rustc hello.rs
./hello
```

If you see "Ready to trade!" — congratulations! Your workstation is ready.

## What We Learned

| Tool | Trading Analogy |
|------|-----------------|
| rustc | Trading terminal — executes your commands |
| cargo | Portfolio manager — organizes projects |
| rustup | Terminal update to new version |
| rust-analyzer | Personal analyst — points out errors |

## Homework

1. Install Rust on your computer
2. Check compiler version with `rustc --version`
3. Install VS Code with rust-analyzer
4. Create a file `test.rs`, write a program that prints your favorite cryptocurrency name

## Navigation

[Next day →](../002-hello-trading-world/en.md)
